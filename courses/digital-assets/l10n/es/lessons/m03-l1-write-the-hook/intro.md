# Escribe el hook: la interfaz + tu único programa en Rust

## Resumen

La lección pasada terminó SPROUT: comisión de transferencia, harvest de comisiones retenidas y el TLV de metadatos nativos, todo cableado sobre un mint, el peldaño R3 en la escalera de artefactos de este curso. Cada extensión hasta ahora rehace el token de forma pasiva. Las comisiones se acumulan. Los metadatos se quedan en el TLV y esperan a que alguien los lea. Hoy conoces la única extensión que corre TU código en cada movimiento del token, y escribes ese código: `harvest-hook`, un programa chico de Anchor que rechaza cualquier destino que no esté en su allowlist y, una vez que termines el challenge en solitario, registra cada transferencia en un libro de tesorería. (El nombre va por lo que protege, los flujos de harvest de Overgrowth; el harvest de comisiones en sí se queda con la secuencia de harvest de m02-l1, porque un hook nunca puede mover fondos.) Tú escribes la interfaz de transfer hook de tres instrucciones, inicializas la cuenta de validación que le dice al runtime qué cuentas extra reenviar, acuñas una variante nueva de SPROUT con hook y lo demuestras en un harness de LiteSVM donde una transferencia entra y otra revierte con tu propio error. Chequeo del repliegue: la interfaz, la PDA de validación y la lista de account-metas se construyen contigo; el gate de allowlist dentro de `Execute` es un TODO que rellenas tú; el log de tesorería es completamente en solitario.

Antes de cualquier teoría, pon las tres cadenas de bytes en tu pantalla. La interfaz nombra sus instrucciones hasheando cadenas, así que un shell deriva las tres:

```bash
# macOS ships shasum, most Linux boxes ship sha256sum; take whichever is there.
sha256() { if command -v shasum >/dev/null; then shasum -a 256; else sha256sum; fi; }

for s in execute initialize-extra-account-metas update-extra-account-metas; do
  printf '%-32s ' "$s"
  printf '%s' "spl-transfer-hook-interface:$s" | sha256 | head -c 16
  echo
done
```

Deberías ver exactamente esto:

```
execute                          692565c54bfb661a
initialize-extra-account-metas   2b220d31a758ebeb
update-extra-account-metas       9d692a926655f1ae
```

Ese es el contrato entero que estás por implementar. No un archivo ABI, no un registro, no una interfaz que heredas: tres prefijos de ocho bytes derivados de tres cadenas ASCII, y cualquier programa de Solana que responda a ellos es un transfer hook. Mantén `692565c54bfb661a` a la vista durante el resto de la lección. Es el timbre que Token-2022 toca en tu programa, y para el final lo vas a haber visto sonar en un log real.

Ya leíste una extensión TransferHook que no apuntaba a nada: la de PYUSD, cuya línea `transferHook.programId: null` imprimiste por primera vez allá en el script de apertura de m01-l1, configurada al nacer el mint con un program id nulo. Vale la pena mirar esa forma otra vez antes de construir, porque reconcilia dos hechos que suenan contradictorios. La extensión en sí es de creación: un mint o lleva el slot de hook desde su primer bloque o no lo llevará nunca. El program id dentro del slot no lo es: la autoridad del hook puede apuntarlo, o anularlo, después. Así que un slot dormido es un arma cargada con la recámara vacía, el slot es el arma y el program id es la bala, y un emisor como Paxos instala el arma vacía a propósito. En el momento en que SÍ hay un programa en la recámara, Token-2022 deja de ser una librería y se vuelve un llamador: hace CPI hacia ese programa en cada transferencia, para siempre, y si el programa devuelve un error la transferencia se muere con él.

## La interfaz, la PDA de validación y los 35 bytes

### Tres cadenas, hasheadas en ocho bytes

Empieza por la restricción que aprieta todo el diseño. Token-2022 tiene que invocar un programa que nunca ha visto, escrito por alguien que nunca ha conocido, sin un crate compartido, sin un registro y sin una negociación de versión. Lo único que los dos lados pueden acordar de antemano es un nombre. Así que la interfaz hashea nombres: `sha256("spl-transfer-hook-interface:execute")[0..8]` prefija los datos de la instrucción, tu programa hace match sobre esos ocho bytes, y el dispatch funciona. Eso es un discriminador de cadena hasheada, y es el mismo truco que hace Anchor con `sha256("global:<method_name>")[0..8]` para sus propias instrucciones. Dos convenciones independientes, un mecanismo, y en un minuto vas a tener un programa que responde a las dos.

Las tres instrucciones se dividen limpiamente por quién las llama:

![Comparación de las tres instrucciones que forman la interfaz de transfer hook, mostrando cuál llama Token-2022 en cada transferencia y cuáles dos llama el emisor.](assets/v01-comparison.png)

Fíjate en la asimetría. Dos de las tres instrucciones son llamadas de gestión comunes que un emisor corre desde un script, en un buen día dos veces en la vida de un mint. La tercera corre en el camino caliente de cada transferencia que llegue a tocar el token, y es la única cuyo costo paga alguien más. Esa asimetría es todo el argumento ético sobre los hooks, y vamos a volver a ella con números.

`Execute` recibe el monto de la transferencia como datos de instrucción y un prefijo fijo de cuentas: source en el índice 0, mint en 1, destination en 2, el owner o delegate del source en 3, después la cuenta de validación, después cada cuenta extra que el runtime resolvió por ti. Quédate con esos índices. No son documentación, son posiciones direccionables sobre las que hashea la codificación de account-metas.

Vale la pena ser preciso sobre qué quiere decir registro aquí, porque es menos de lo que la gente espera. No hay un registro global de hooks. Token-2022 no lleva una tabla de programas aprobados, no hay ninguna allowlist a la que entrar, y nada valida que el program id en la extensión TransferHook de un mint sea siquiera un programa. La extensión del propio mint nombrando tu program id es todo el cableado. Lo cual también quiere decir que el modo de falla cuando tu programa no responde a la interfaz no es un error útil: Token-2022 le pasa a tu programa ocho bytes que no reconoce, el dispatch se cae por el final de tu match, y te llevas un error de fallback de un programa que parece que nunca llegó a ser llamado. Si alguna vez ves una transferencia con hook morir dentro de tu propio programa sin nada en el log más que una queja de fallback, tienes un problema de discriminador, no un problema de lógica.

![La tabla de dispatch de un programa, con tres discriminadores del namespace de Anchor y tres del namespace de la interfaz, y los prefijos sin coincidencia cayendo a un error de fallback.](assets/v02-diagram.png)

### La cuenta de validación vive en TU programa

Aquí está la pieza que hace tropezar a casi todo el mundo la primera vez, incluido yo la primera vez que cableé uno de estos. El hook necesita cuentas extra. Token-2022 no tiene forma de saber cuáles, porque tu programa es arbitrario. Los programas de Solana no pueden buscar cuentas en runtime, así que nadie aguas abajo puede descubrirlas tampoco. La respuesta de la interfaz es un manifiesto publicado: una cuenta on-chain, por mint, cuyos datos listan exactamente qué cuentas extra necesita una llamada a `Execute` y cómo derivar cada una.

Esa cuenta es el `ExtraAccountMetaList`, vive en una PDA sembrada con el literal `extra-account-metas` y el mint, y es propiedad de TU programa de hook. No del mint. No de Token-2022.

![La extensión TransferHook del mint apunta al programa del hook, y la PDA de validación cuelga de ese programa del hook y no del mint ni de Token-2022.](assets/v03-diagram.png)

La implementación de referencia te da la derivación como una función, `get_extra_account_metas_address(&mint, &program_id)`, y el argumento `program_id` es el que la gente rellena mal. Pasa Token-2022 ahí y te sale una dirección perfectamente válida que ninguna cuenta va a ocupar jamás, así que cada transferencia falla en la resolución con un error que no dice nada del error real. Si te llevas una sola regla de derivación de esta lección, llévate esta: el manifiesto pertenece al programa que necesita las cuentas, porque es la única parte que sabe cuáles son.

¿Por qué una por mint y no una por cuenta de token? Porque las necesidades son una propiedad de la política, no del tenedor. Un hook que condiciona el acceso a una allowlist necesita la cuenta de la allowlist tanto si el destino es una ballena como si es una billetera nueva. Por mint mantiene la lista chica, mantiene las actualizaciones atómicas, y mantiene el camino de lectura del cliente en exactamente una cuenta.

### 35 bytes para una cuenta que nadie ha creado todavía

Cada entrada de esa lista es un `ExtraAccountMeta`, un struct fijo de 35 bytes. El tamaño fijo hace trabajo real aquí: un cliente puede recorrer la lista sin un esquema, y el lado on-chain puede indexarla sin asignar memoria. Un byte de discriminador elige cómo se encuentra la dirección, treinta y dos bytes llevan o bien la dirección misma o bien un conjunto empaquetado de configuraciones de seeds, y dos bytes llevan los flags de signer y writable.

El caso interesante es el que usa tu hook. Las dos cuentas que necesita `harvest-hook` son PDAs del programa del hook derivadas del mint, y el mint no se conoce cuando escribes la lista, se conoce cuando ocurre la transferencia. Así que en vez de una dirección, la entrada guarda una receta: un seed literal, y después "la key de la cuenta en el índice 1 de la lista de cuentas de Execute," que es el mint.

![Las dos entradas de account-meta del hook codifican una receta de PDA, un seed literal más la key de la cuenta de Execute en el índice 1, el mint, con la entrada de tesorería como writable.](assets/v04-annotated-code.png)

Dos consecuencias se siguen de los seeds basados en índice, y las dos muerden en producción. Primero, la resolución es posicional: si un cliente resuelve la lista fuera de orden o se salta una entrada, cada seed posterior basado en índice deriva una dirección distinta, en silencio, y la transferencia revierte con un desajuste que no apunta a nada útil. Segundo, el flag writable de la entrada es propio de la entrada, no heredado de la transferencia, que es la razón por la que se puede escribir en tu log de tesorería aunque todo lo que llega de la transferencia sea de solo lectura. Esa afirmación de solo lectura la demostramos en la próxima lección, sacada del constructor de instrucciones que trae el propio crate de la interfaz; hoy, tómala como la razón por la que el diseño es lo bastante seguro como para entregarlo siquiera.

### Lo que el hook puede hacer, y lo que le cuesta a todos los demás

Antes del precio, el límite, porque es lo que hace que el precio sea pagable siquiera. Tu hook recibe la cuenta source, la cuenta destination y el owner. No puede gastar de ninguna de ellas. No porque se niegue con educación, sino porque Token-2022 despoja a esas cuentas antes de la CPI: llegan de solo lectura y no firmantes, así que una escritura es un error de runtime y no hay firma disponible para reutilizar. El inventario completo de poderes del hook son tres ítems. Puede leer cualquier cosa en las cuentas que le dieron. Puede escribir en sus propios extras declarados, que es por lo que funciona el log de tesorería. Y puede devolver un error, que aborta la transferencia entera de forma atómica.

Deriva esa restricción en vez de memorizarla, porque el razonamiento generaliza. Un hook es código elegido por el emisor, corriendo dentro de una transacción escrita por alguien más, con las cuentas de ese alguien en alcance. Si el hook pudiera firmar o escribir en esas cuentas, tener un token querría decir darle al emisor permiso permanente para mover tus otros saldos a mitad de transferencia, y cada integración tendría que auditar un programa arbitrario antes de cotizar un swap. La única versión de esta funcionalidad que puede existir sin esa auditoría es un asiento con veto: el hook lo ve todo y no toca nada. Por eso "¿un hook me puede vaciar?" tiene una respuesta de una línea, y es la forma que deberías buscar en cualquier diseño parecido a un hook. Observa, registra en tu propio terreno, rechaza. Las cuatro líneas del crate de la interfaz que ponen esos flags de cuenta se leen línea por línea en la próxima lección.

El canje, entonces, es filoso y vale la pena ponerle precio antes de que escribas una línea. Lo que compra el emisor es real: control por transferencia, expresable como lógica de programa arbitraria, impuesto por el propio programa de token en vez de por una app alrededor de la cual un usuario puede enrutar. Lo que paga todo el mundo también es real, y viene en dos monedas.

La primera es el cómputo. Medí este harness por los dos caminos. Un `TransferChecked` simple de Token-2022 sobre un mint sin extensiones quemó 1,790 CU. La misma transferencia a través del mint con hook quedó entre unos 23,000 y 35,000 CU de una corrida a otra, con el propio `Execute` del hook dando cuenta de unos 9,400 a 13,500 de eso. Diez a veinte veces el costo de la transferencia que protege, para un hook cuya lógica entera es un escaneo booleano de un array de ocho entradas.

![Un TransferChecked simple cuesta 1,790 unidades de cómputo contra 23,108 a 35,292 el que lleva hook, de las cuales Execute son 9,448 a 13,448.](assets/v05-chart.png)

Vale la pena pausar en ese rango, porque es una lección en sí. La varianza no es el escaneo de la allowlist, que no cuesta nada. Es `find_program_address`: cada constraint de seeds que no lleva un bump guardado recorre la búsqueda, y cada iteración cuesta cómputo real. Guardar los bumps canónicos es el arreglo estándar y el curso Master Anchor V2 lo cubre como un patrón del framework. Estoy dejando un bump sin guardar en este programa a propósito para que la varianza aparezca en tus propios logs.

La segunda moneda es la coordinación, y es la cara. Como las cuentas extra tienen que estar en la transacción antes de que se envíe, cada billetera, cada DEX, cada integración de pagos que llegue a tocar tu token tiene que buscar tu cuenta de validación, decodificar esas entradas de 35 bytes, resolver cada una y agregarlas en orden. Para siempre. Un hook no solo gasta el cómputo del emisor; le empuja una obligación permanente de reenvío a extraños que nunca la aceptaron. Ese es el hecho con el que abre la próxima lección, y es por lo que una porción seria del ecosistema simplemente rechaza los tokens con hook.

![Antes de cada transferencia de un token con hook un cliente debe buscar la cuenta de validación, decodificar sus entradas, resolver cada una y agregarlas en orden.](assets/v06-flowchart.png)

Dos recibos para ubicar la funcionalidad en el mundo real antes de construir. PYUSD, el lanzamiento emblemático de Token-2022 en Solana en mayo de 2024 de PayPal y Paxos, trae un conjunto de ocho extensiones TLV con forma de compliance, y una de ellas es un transferHook cuyo `programId` es null. Configurado, dormido, reservado. Los emisores recurren a este slot en el momento en que compliance está sobre la mesa, incluso cuando no están listos para usarlo. Y el estado del material oficial, para que sepas qué existe antes de construir: solana.com aloja una guía de transfer hooks — un tutorial de Anchor con build, deploy y tests — más una guía de integración para el camino de envío del cliente, solana-program.com documenta la interfaz al lado de una implementación de referencia, y solana-developers/program-examples trae ejemplos de transfer hook. Existen tutoriales para copiar, en otras palabras. Lo que agrega esta lección es la parte que copiar no te da: versiones fijadas, cómputo medido, un hook cableado en el token SPROUT que has cargado durante dos módulos, y un gate que escribes tú mismo y defiendes contra una suite de pruebas en rojo.

## Lab: de un crate vacío a una transferencia revertida

El plan: un crate, un programa, un archivo de pruebas. Vas a construir el hook, inicializar su manifiesto, acuñar una variante de SPROUT con hook dentro del harness y pasarle dos transferencias. Todo lo de este lab se construyó y se corrió en esta máquina el 2026-08-22 con los pins exactos de abajo, incluidos los números de cómputo que acabas de leer.

Una nota de alcance antes del primer comando, porque cambia cómo deberías leer el código. Este lab usa Anchor y no lo enseña. La capa del framework, las macros, los constraints de cuentas, la mecánica de CPI, los patrones de pruebas, todo eso es del curso Master Anchor V2 (`mastering-anchor-v2`), y si un bloque `#[derive(Accounts)]` de aquí te da ganas de una explicación más completa de lo que hace el sistema de constraints, ese es el curso al que llevarla; si nunca has leído ningún programa de Anchor, su módulo inicial de anatomía es lo que hay que leer primero antes de este lab. Lo que estás aprendiendo aquí es la interfaz y la extensión, no el framework. El programa son unas cuarenta líneas de lógica con un abrigo fino de Anchor encima, y todo lo específico de hooks se vería igual en Rust puro con más ceremonia.

La escalera de autonomía del lab, dicha sin rodeos para que sepas cuándo estás por tu cuenta: los pasos del 1 al 6 se trabajan contigo, el paso 7 deja la función `gate` deliberadamente vacía para que la escribas tú, el paso 9 es donde la suite de pruebas se pone roja contra ese gate vacío, y el Challenge va sin apoyo.

**1. Consigue el toolchain de build.** Necesitas Rust y el compilador SBF del toolchain de Solana. ¿Todavía no tienes ningún toolchain de Rust? El curso Rust & TypeScript Fundamentals lo instala desde cero en su m04-l1, y su módulo 4 en general es de donde sale la fluidez para leer Rust en la que se apoya esta lección. Si `cargo-build-sbf` no está ya en tu path de trabajos anteriores:

```bash
# Rust, if you do not have it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# The Agave toolchain, which brings cargo-build-sbf and the solana CLI
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"

cargo-build-sbf --version
```

No necesitas la CLI de `anchor` para este lab y no está en la lista de instalación de arriba. Si ya la tienes vía avm de otros trabajos, `anchor build` produce el mismo `.so`; todo aquí usa `cargo build-sbf` directamente para que el crate siga siendo un paquete Cargo común sin ningún scaffolding de workspace alrededor.

**2. Crea el crate.** Un directorio, un paquete. Llámalo `hook/` al lado de la carpeta `labs/` que has venido usando.

```bash
mkdir -p hook/src hook/tests && cd hook
```

`hook/Cargo.toml`, completo:

```toml
[package]
name = "harvest-hook"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
name = "harvest_hook"

[dependencies]
anchor-lang = "=1.1.2"
spl-tlv-account-resolution = "=0.11.1"
spl-transfer-hook-interface = "=2.1.0"
spl-discriminator = "0.5"

[workspace]

[dev-dependencies]
litesvm = "=0.15.2"
solana-program-runtime = "=4.1.1"
solana-builtins = "=4.1.1"
solana-instruction = "3"
solana-keypair = "3"
solana-signer = "3"
solana-system-interface = "2"
solana-transaction = "4"
spl-token-2022 = "=11.0.0"
```

Cuatro notas sobre esos pins, todas verificadas el 2026-08-22 en vez de recordadas, con una re-verificación fechada el 2026-09-01 donde está marcado.

`anchor-lang` está fijado en 1.1.2, que es contra lo que se verificó el código de este lab y ya NO es el release más nuevo de la línea V1: 1.2.0 se entregó el 2026-09-04, después de que esta lección se escribiera y después de que se tomara su propio versionStamp. El pin sigue en pie, porque un pin es una afirmación sobre qué se probó y no sobre qué es más nuevo, pero no le repitas "1.1.2 es el actual" a nadie. También existe un 2.0.0-rc.1 y es asunto del curso Master Anchor V2, no nuestro; V2 cambia la superficie de tipos de cuenta lo suficiente como para que este archivo no compilara contra él sin cambios. `spl-transfer-hook-interface` 2.1.0 es el release actual de la interfaz. `spl-tlv-account-resolution` está fijado en 0.11.1, la versión contra la que se verificaron las afirmaciones a nivel de bytes de este lab; crates.io ha entregado desde entonces 0.11.2/0.11.3 (re-verificado el 2026-09-01), que rehacen los internos del resolvedor sin tocar el formato serializado. La tabla vacía `[workspace]` no es decoración: impide que el workspace de un directorio padre adopte este crate y arrastre una resolución de dependencias incompatible.

Los dos pins de dev-dependency que se ven raros son del tipo feo honesto. LiteSVM 0.15.2 no compila contra la línea 4.2 de los crates de runtime Agave que Cargo elegiría para él si no, así que `solana-program-runtime` y `solana-builtins` están fijados hacia atrás en 4.1.1 para sostener el resolvedor en una versión contra la que se construyó LiteSVM. Si un release futuro de LiteSVM arregla eso, elimina las dos líneas. Esto es exactamente el tipo de cosa que se pudre, así que verifícalo en vez de confiar en un curso.

**3. Instala el harness, y conócelo.** LiteSVM es la dev-dependency que acabas de agregar, así que ya está instalado. Vale diez segundos decir qué es, porque es nuevo en este curso: LiteSVM es una VM de Solana en proceso. Te da un runtime SBF real con programas reales y medición de cómputo real, en una librería, sin proceso de validador, sin RPC y sin ledger. Las pruebas corren en milisegundos en vez de en decenas de segundos. También trae los programas SPL, que es por lo que el harness puede crear un mint de Token-2022 sin que despliegues nada. El canje es que no es un cluster: ni leader schedule, ni mercado de comisiones, ni red. Para un hook, que es lógica pura a nivel de instrucción, es el instrumento correcto.

**4. Escribe el estado y los errores.** Crea `hook/src/lib.rs` y empieza con los imports, el program id, los seeds y las cuentas que tu hook posee:

```rust
use anchor_lang::prelude::*;
use spl_discriminator::SplDiscriminate;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::{
    ExecuteInstruction, InitializeExtraAccountMetaListInstruction,
    UpdateExtraAccountMetaListInstruction,
};

declare_id!("HookH1FQuTU21GVAjJZDLXPjXWLQFPJ5FLpwGKZLkYQ");

pub const CONFIG_SEED: &[u8] = b"hook-config";
pub const TREASURY_SEED: &[u8] = b"treasury";
pub const META_LIST_SEED: &[u8] = b"extra-account-metas";
pub const MAX_ALLOWED: usize = 8;

pub const TOKEN_2022_ID: Pubkey =
    Pubkey::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

#[account]
#[derive(InitSpace)]
pub struct HookConfig {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub paused: bool,
    pub bump: u8,
    pub allowed_len: u8,
    pub allowed: [Pubkey; MAX_ALLOWED],
}

#[account]
#[derive(InitSpace)]
pub struct TreasuryLog {
    pub mint: Pubkey,
    pub bump: u8,
    pub transfers: u64,
    pub total_amount: u128,
    pub last_amount: u64,
    pub last_destination: Pubkey,
}

#[error_code]
pub enum HarvestHookError {
    #[msg("Transfers are paused by the hook authority")]
    Paused,
    #[msg("Destination is not on the hook allowlist")]
    NotAllowed,
    #[msg("The allowlist is full")]
    AllowlistFull,
    #[msg("Only the hook authority may call this")]
    Unauthorized,
    #[msg("Source account is not owned by Token-2022")]
    NotTokenAccount,
}
```

Un array fijo de ocho slots en vez de un `Vec` es una elección deliberada: `Execute` corre en cada transferencia, y un layout fijo quiere decir nada de realocación, nada de sorpresas de longitud, y un escaneo cuyo costo puedes razonar. Un emisor real con miles de destinos en la allowlist usaría una PDA por destino y dejaría que la meta list la derive de la key del destination en el índice 2, que la codificación soporta. Ocho slots mantienen honesto el conteo de cuentas de la lección.

**5. Publica el manifiesto.** Esta es la función que escribe esas dos entradas de 35 bytes. Agrega a `lib.rs`:

```rust
fn harvest_metas() -> Result<Vec<ExtraAccountMeta>> {
    Ok(vec![
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal { bytes: CONFIG_SEED.to_vec() },
                Seed::AccountKey { index: 1 },
            ],
            false,
            false,
        )?,
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal { bytes: TREASURY_SEED.to_vec() },
                Seed::AccountKey { index: 1 },
            ],
            false,
            true,
        )?,
    ])
}
```

Lee los dos argumentos booleanos por lo que son: `is_signer` y después `is_writable`. La config es de solo lectura porque `Execute` solo la lee. El log de tesorería es writable porque el ejercicio en solitario va a escribir en él, y declararlo ahora quiere decir que la cuenta ya está en cada transferencia para cuando la necesites.

**6. Las tres instrucciones de la interfaz, más las dos de gestión.** Ahora el módulo del programa. La línea que hay que mirar fijo es el atributo `#[instruction(discriminator = ...)]`: es cómo un programa de Anchor responde a la convención de nombres de otro en vez de a la suya.

```rust
#[program]
pub mod harvest_hook {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.mint = ctx.accounts.mint.key();
        config.paused = false;
        config.bump = ctx.bumps.config;
        config.allowed_len = 0;
        config.allowed = [Pubkey::default(); MAX_ALLOWED];

        let log = &mut ctx.accounts.treasury_log;
        log.mint = ctx.accounts.mint.key();
        log.bump = ctx.bumps.treasury_log;
        log.transfers = 0;
        log.total_amount = 0;
        log.last_amount = 0;
        log.last_destination = Pubkey::default();
        Ok(())
    }

    pub fn allow_destination(ctx: Context<Manage>, destination: Pubkey) -> Result<()> {
        let config = &mut ctx.accounts.config;
        let len = config.allowed_len as usize;
        require!(len < MAX_ALLOWED, HarvestHookError::AllowlistFull);
        config.allowed[len] = destination;
        config.allowed_len = config
            .allowed_len
            .checked_add(1)
            .ok_or(HarvestHookError::AllowlistFull)?;
        Ok(())
    }

    pub fn set_paused(ctx: Context<Manage>, paused: bool) -> Result<()> {
        ctx.accounts.config.paused = paused;
        Ok(())
    }

    #[instruction(discriminator = InitializeExtraAccountMetaListInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn initialize_extra_account_metas(ctx: Context<InitializeMetas>) -> Result<()> {
        let metas = harvest_metas()?;
        let mut data = ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?;
        ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &metas)?;
        Ok(())
    }

    #[instruction(discriminator = UpdateExtraAccountMetaListInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn update_extra_account_metas(ctx: Context<UpdateMetas>) -> Result<()> {
        let metas = harvest_metas()?;
        let mut data = ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?;
        ExtraAccountMetaList::update::<ExecuteInstruction>(&mut data, &metas)?;
        Ok(())
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn execute(ctx: Context<Execute>, amount: u64) -> Result<()> {
        require_keys_eq!(
            *ctx.accounts.source.owner,
            TOKEN_2022_ID,
            HarvestHookError::NotTokenAccount
        );
        let config = &ctx.accounts.config;
        gate(config, &ctx.accounts.destination.key())?;
        msg!(
            "harvest-hook: allowed {} to {}",
            amount,
            ctx.accounts.destination.key()
        );
        Ok(())
    }
}
```

Tres cosas que pasan ahí adentro merecen una oración cada una.

`SPL_DISCRIMINATOR_SLICE` es una const en los tipos marcador del crate de la interfaz, y su valor es el prefijo sha256 que imprimiste en tu shell al comienzo de la lección. No estás copiando un literal hex, estás referenciando la misma constante que el programa de token va a calcular. Esa es la diferencia entre una interfaz y una coincidencia.

`ExtraAccountMetaList::init::<ExecuteInstruction>` escribe una entrada TLV cuyo tipo es el discriminador de execute. La lista no es solo "unas cuantas cuentas," es "las cuentas que `execute` necesita," y la etiqueta de tipo lo dice. Eso es lo que hace que el manifiesto se auto-describa del lado del cliente.

La primera línea de `execute` verifica que la cuenta source sea propiedad de Token-2022. Tu programa es públicamente invocable: cualquiera puede llamarlo directo con cuatro cuentas cualesquiera y afirmar que está ocurriendo una transferencia. La verificación de owner es la guarda más barata que impide que un extraño le meta basura a tu log. No es una defensa completa, y la interfaz ofrece una más fuerte: `TransferHookAccount`, la pequeña extensión de cuenta que un mint con hook fuerza sobre cada cuenta de tenedor (vas a dimensionar cuentas para ella en el paso 8), lleva un flag de transferencia que se pone solo mientras una transferencia real está en vuelo, así que un hook puede revisarlo y rechazar llamadas directas de plano. Para un hook de allowlist-y-log la verificación de owner es proporcionada; para un hook que mueve valor basándose en lo que observó, no lo sería.

**7. Cablea las cuentas, y deja el gate vacío.** El struct de cuentas de `Execute` tiene que coincidir con el orden de la interfaz exactamente, porque esa lista la construye Token-2022, no tú.

```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: read as a key only; the mint is validated by Token-2022 at transfer time.
    pub mint: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = HookConfig::DISCRIMINATOR.len() + HookConfig::INIT_SPACE,
        seeds = [CONFIG_SEED, mint.key().as_ref()],
        bump
    )]
    pub config: Account<'info, HookConfig>,
    #[account(
        init,
        payer = authority,
        space = TreasuryLog::DISCRIMINATOR.len() + TreasuryLog::INIT_SPACE,
        seeds = [TREASURY_SEED, mint.key().as_ref()],
        bump
    )]
    pub treasury_log: Account<'info, TreasuryLog>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Manage<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [CONFIG_SEED, config.mint.as_ref()],
        bump = config.bump,
        has_one = authority @ HarvestHookError::Unauthorized
    )]
    pub config: Account<'info, HookConfig>,
}

#[derive(Accounts)]
pub struct InitializeMetas<'info> {
    #[account(
        init,
        payer = authority,
        space = ExtraAccountMetaList::size_of(2)?,
        seeds = [META_LIST_SEED, mint.key().as_ref()],
        bump
    )]
    /// CHECK: written as raw TLV by spl-tlv-account-resolution.
    pub extra_account_meta_list: UncheckedAccount<'info>,
    /// CHECK: key only.
    pub mint: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateMetas<'info> {
    #[account(
        mut,
        seeds = [META_LIST_SEED, mint.key().as_ref()],
        bump
    )]
    /// CHECK: written as raw TLV by spl-tlv-account-resolution.
    pub extra_account_meta_list: UncheckedAccount<'info>,
    /// CHECK: key only.
    pub mint: UncheckedAccount<'info>,
    #[account(
        seeds = [CONFIG_SEED, mint.key().as_ref()],
        bump = config.bump,
        has_one = authority @ HarvestHookError::Unauthorized
    )]
    pub config: Account<'info, HookConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Execute<'info> {
    /// CHECK: source token account, read-only by interface contract.
    pub source: UncheckedAccount<'info>,
    /// CHECK: mint, read-only by interface contract.
    pub mint: UncheckedAccount<'info>,
    /// CHECK: destination token account, read-only by interface contract.
    pub destination: UncheckedAccount<'info>,
    /// CHECK: source owner or delegate, read-only by interface contract.
    pub owner: UncheckedAccount<'info>,
    #[account(
        seeds = [META_LIST_SEED, mint.key().as_ref()],
        bump
    )]
    /// CHECK: the validation account Token-2022 resolved for us.
    pub extra_account_meta_list: UncheckedAccount<'info>,
    #[account(
        seeds = [CONFIG_SEED, mint.key().as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, HookConfig>,
    #[account(
        mut,
        seeds = [TREASURY_SEED, mint.key().as_ref()],
        bump = treasury_log.bump
    )]
    pub treasury_log: Account<'info, TreasuryLog>,
}
```

`ExtraAccountMetaList::size_of(2)` es el espacio para exactamente dos entradas, que es el largo de `harvest_metas`. Cambia uno y tienes que cambiar el otro, y un desajuste aquí aparece como una falla de serialización en el init y no a la hora de la transferencia, que es el orden misericordioso.

Una decisión de diseño en `InitializeMetas` vale la pena señalarla en vez de saltarla, porque vas a tener que responder por ella en revisión. La documentación de la propia interfaz lista la tercera cuenta de initialize-extra-account-metas como la autoridad de mint, firmando. Este programa toma cualquier signer y lo hace el payer, y no lo verifica contra el mint. Para una lección está bien y para un mint que controlas está bien, porque la cuenta es una PDA de tu programa con el mint como clave, así que se puede crear exactamente una vez y quien la cree publica la misma lista fija de cualquier forma. Deja de estar bien en el momento en que tu programa sirve a mints que no son tuyos: entonces el primero que llame decide para siempre qué dice el manifiesto, que es un problema de ocupación sin más arreglo que un redespliegue. Si entregas un hook de propósito general, restringe esa cuenta contra la autoridad del mint y guarda la autoridad en tu config. Costo de la versión honesta: una cuenta más y una verificación más.

Ahora el gate, que es la parte que escribes tú. Agrega esta función fuera del módulo `#[program]`, exactamente como está escrita:

```rust
fn gate(config: &HookConfig, destination: &Pubkey) -> Result<()> {
    // TODO(you): the pause kill-switch first, then the allowlist.
    //   1. if config.paused is true, fail with HarvestHookError::Paused
    //   2. if `destination` is not among the first config.allowed_len entries
    //      of config.allowed, fail with HarvestHookError::NotAllowed
    let _ = (config, destination);
    Ok(())
}
```

Eso compila y está mal, deliberadamente. El orden importa de una forma que vale la pena decir en voz alta: la pausa es un interruptor de apagado global y tiene que ganarle a la allowlist, porque la situación en la que recurres a la pausa es la situación en la que ya no confías en tu propia allowlist.

**8. Construye el harness.** Crea `hook/tests/hook.rs`. Este es el archivo más largo de la lección y está haciendo algo específico: levantar un mint de Token-2022 CON la extensión TransferHook usando constructores de instrucciones en crudo, no constraints de Anchor. Las extensiones son de creación y a nivel de instrucción, que es el patrón que has usado todo el curso desde TypeScript; aquí son las mismas instrucciones desde Rust.

```rust
use anchor_lang::prelude::Pubkey;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use harvest_hook::{HookConfig, CONFIG_SEED, META_LIST_SEED, TREASURY_SEED};
use litesvm::{types::TransactionMetadata, LiteSVM};
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_token_2022::{
    extension::{transfer_hook, ExtensionType},
    state::{Account as TokenAccount, Mint},
    ID as TOKEN_2022_ID,
};

const SO_PATH: &str = "target/deploy/harvest_hook.so";

struct Fixture {
    svm: LiteSVM,
    payer: Keypair,
    mint: Pubkey,
    source: Pubkey,
    destination: Pubkey,
    config: Pubkey,
    treasury: Pubkey,
    validation: Pubkey,
}

fn send(
    svm: &mut LiteSVM,
    payer: &Keypair,
    ixs: &[Instruction],
    signers: &[&Keypair],
) -> Result<TransactionMetadata, String> {
    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(ixs, Some(&payer.pubkey()), signers, blockhash);
    svm.send_transaction(tx).map_err(|e| {
        for line in &e.meta.logs {
            println!("{line}");
        }
        format!("{:?}", e.err)
    })
}

fn setup() -> Fixture {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    svm.add_program_from_file(harvest_hook::ID, SO_PATH).unwrap();

    let mint_kp = Keypair::new();
    let mint = mint_kp.pubkey();
    let mint_len =
        ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::TransferHook]).unwrap();
    let create_mint = solana_system_interface::instruction::create_account(
        &payer.pubkey(),
        &mint,
        svm.minimum_balance_for_rent_exemption(mint_len),
        mint_len as u64,
        &TOKEN_2022_ID,
    );
    let init_hook = transfer_hook::instruction::initialize(
        &TOKEN_2022_ID,
        &mint,
        Some(payer.pubkey()),
        Some(harvest_hook::ID),
    )
    .unwrap();
    let init_mint =
        spl_token_2022::instruction::initialize_mint2(&TOKEN_2022_ID, &mint, &payer.pubkey(), None, 0)
            .unwrap();
    send(&mut svm, &payer, &[create_mint, init_hook, init_mint], &[&payer, &mint_kp]).unwrap();

    let acc_len = ExtensionType::try_calculate_account_len::<TokenAccount>(&[
        ExtensionType::TransferHookAccount,
    ])
    .unwrap();
    let mut token_accounts = Vec::new();
    for _ in 0..2 {
        let kp = Keypair::new();
        let create = solana_system_interface::instruction::create_account(
            &payer.pubkey(),
            &kp.pubkey(),
            svm.minimum_balance_for_rent_exemption(acc_len),
            acc_len as u64,
            &TOKEN_2022_ID,
        );
        let init = spl_token_2022::instruction::initialize_account3(
            &TOKEN_2022_ID,
            &kp.pubkey(),
            &mint,
            &payer.pubkey(),
        )
        .unwrap();
        send(&mut svm, &payer, &[create, init], &[&payer, &kp]).unwrap();
        token_accounts.push(kp.pubkey());
    }
    let (source, destination) = (token_accounts[0], token_accounts[1]);

    let mint_to = spl_token_2022::instruction::mint_to(
        &TOKEN_2022_ID,
        &mint,
        &source,
        &payer.pubkey(),
        &[],
        1_000,
    )
    .unwrap();
    send(&mut svm, &payer, &[mint_to], &[&payer]).unwrap();

    let (config, _) = Pubkey::find_program_address(&[CONFIG_SEED, mint.as_ref()], &harvest_hook::ID);
    let (treasury, _) =
        Pubkey::find_program_address(&[TREASURY_SEED, mint.as_ref()], &harvest_hook::ID);
    let (validation, _) =
        Pubkey::find_program_address(&[META_LIST_SEED, mint.as_ref()], &harvest_hook::ID);

    let init = Instruction {
        program_id: harvest_hook::ID,
        accounts: harvest_hook::accounts::Initialize {
            authority: payer.pubkey(),
            mint,
            config,
            treasury_log: treasury,
            system_program: solana_system_interface::program::ID,
        }
        .to_account_metas(None),
        data: harvest_hook::instruction::Initialize {}.data(),
    };
    let init_metas = Instruction {
        program_id: harvest_hook::ID,
        accounts: harvest_hook::accounts::InitializeMetas {
            extra_account_meta_list: validation,
            mint,
            authority: payer.pubkey(),
            system_program: solana_system_interface::program::ID,
        }
        .to_account_metas(None),
        data: harvest_hook::instruction::InitializeExtraAccountMetas {}.data(),
    };
    send(&mut svm, &payer, &[init, init_metas], &[&payer]).unwrap();

    Fixture { svm, payer, mint, source, destination, config, treasury, validation }
}
```

Cosa chica con un gran rendimiento al final de esa función: `harvest_hook::instruction::InitializeExtraAccountMetas {}.data()` emite el discriminador de la interfaz, no uno de Anchor. La macro generó ese tipo a partir de tu handler, vio el override, y horneó `2b220d31a758ebeb` en su serialización. Así que la prueba construye una instrucción estándar de la interfaz usando los propios tipos generados de tu programa, y si alguna vez cambias el override la prueba se rompe en tiempo de compilación en vez de en runtime. Consistencia gratis, y vale la pena saber que está ahí.

El mint se crea con `ExtensionType::TransferHook` en su cálculo de largo de cuenta y con `transfer_hook::instruction::initialize` antes de `initialize_mint2`. Ese orden no es estilístico. Las extensiones tienen que inicializarse después de que la cuenta existe y antes de que el mint se inicialice, y la extensión TransferHook es solo de creación, una de las cuatro trampas que el Checkpoint junta en una tabla, y la razón por la que estamos acuñando una variante nueva de SPROUT en vez de retroadaptar el mint que terminaste la lección pasada. A un mint existente sin la extensión no le puede crecer una nunca. Una honestidad de alcance sobre esta variante: lleva TransferHook sola, decimals 0, sin comisiones, sin metadatos. Es el doble de pruebas de SPROUT con el acceso restringido, no un re-mint del stack R3 completo; componer las extensiones de R3 sobre un mint con hook es el mismo orden de creación con más inicializadores, y nada en este módulo depende de esa composición. Si quieres restringirle el acceso a tu SPROUT en vivo, acuñas una variante nueva y migras a los tenedores, y ninguna cantidad de `update-extra-account-metas` cambia eso.

Fíjate también en que las dos cuentas de token reservan espacio para `ExtensionType::TransferHookAccount`. Token-2022 exige esa extensión en cada cuenta de tenedor de un mint con hook, y si las dimensionas como cuentas simples, `initialize_account3` falla antes de que llegues al hook.

Ahora las dos transferencias y los asserts:

```rust
fn hooked_transfer(f: &Fixture, amount: u64) -> Instruction {
    let mut ix = spl_token_2022::instruction::transfer_checked(
        &TOKEN_2022_ID,
        &f.source,
        &f.mint,
        &f.destination,
        &f.payer.pubkey(),
        &[],
        amount,
        0,
    )
    .unwrap();
    ix.accounts.extend_from_slice(&[
        AccountMeta::new_readonly(f.config, false),
        AccountMeta::new(f.treasury, false),
        AccountMeta::new_readonly(harvest_hook::ID, false),
        AccountMeta::new_readonly(f.validation, false),
    ]);
    ix
}

fn allow(f: &mut Fixture, destination: Pubkey) {
    let ix = Instruction {
        program_id: harvest_hook::ID,
        accounts: harvest_hook::accounts::Manage {
            authority: f.payer.pubkey(),
            config: f.config,
        }
        .to_account_metas(None),
        data: harvest_hook::instruction::AllowDestination { destination }.data(),
    };
    let payer = f.payer.insecure_clone();
    send(&mut f.svm, &payer, &[ix], &[&payer]).unwrap();
}

#[test]
fn allowlisted_transfer_passes_the_hook() {
    let mut f = setup();
    let destination = f.destination;
    allow(&mut f, destination);
    let ix = hooked_transfer(&f, 100);
    let payer = f.payer.insecure_clone();
    let meta = send(&mut f.svm, &payer, &[ix], &[&payer]).expect("allowlisted transfer should land");
    for line in meta.logs.iter().filter(|l| l.contains("harvest-hook") || l.contains("consumed")) {
        println!("{line}");
    }

    let raw = f.svm.get_account(&f.config).unwrap();
    let config = HookConfig::try_deserialize(&mut raw.data.as_slice()).unwrap();
    assert_eq!(config.allowed_len, 1);
}

#[test]
fn stranger_transfer_fails_the_hook() {
    let mut f = setup();
    let ix = hooked_transfer(&f, 100);
    let payer = f.payer.insecure_clone();
    let err = send(&mut f.svm, &payer, &[ix], &[&payer])
        .expect_err("a non-allowlisted destination must be rejected by the hook");
    assert!(err.contains("Custom"), "expected the hook's own error, got {err}");
}
```

`hooked_transfer` es donde el harness te está mintiendo con discreción, y vale la pena nombrarlo ahora para que la próxima lección aterrice. Esas cuatro cuentas agregadas al final, los dos extras en el orden de la lista, después el programa del hook, después la cuenta de validación, son exactamente lo que un cliente debe aportar, en exactamente ese orden. Aquí las escribí a mano porque conozco mi propio hook. Una billetera no. Esa brecha es el tema entero de la próxima lección.

![Una transferencia con hook corre a través de Token-2022 hacia el Execute del hook a profundidad dos, con un borde de falla antes de que corra tu lógica y otro dentro del gate mismo.](assets/v07-flowchart.png)

**9. Córrelo, y lee el rojo.** Primero construye el programa a bytecode SBF, porque el harness carga el `.so` compilado:

```bash
cargo build-sbf
cargo test -p harvest-hook -- --nocapture
```

Cargo corre dos binarios de prueba: primero las pruebas unitarias del propio crate, que según tu toolchain pueden estar vacías o llevar una prueba generada trivial, y de cualquier forma no te dicen nada, y después `tests/hook.rs`, que es el que hay que leer. Con el gate todavía vacío, esa segunda suite vuelve así (las dos pruebas corren en paralelo, así que el orden cambia de corrida en corrida):

```
running 2 tests
test allowlisted_transfer_passes_the_hook ... ok
test stranger_transfer_fails_the_hook ... FAILED

failures:
    stranger_transfer_fails_the_hook

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

Una verde, una roja, y la roja es tu tarea. El hook está cableado correctamente: Token-2022 lo encontró, resolvió las cuentas, llamó a `Execute`, y tu código le dijo que sí a todo el mundo. Ahora ve a implementar `gate` para que diga que no. Dos líneas de `require!`, la pausa primero. Cuando lo tengas, el mismo comando imprime:

```
running 2 tests
Program log: harvest-hook: allowed 100 to BtqSbosaGCZZczgs6oAVRoRkMYLTi3v8qs6t8DzyG88Y
Program HookH1FQuTU21GVAjJZDLXPjXWLQFPJ5FLpwGKZLkYQ consumed 11948 of 184840 compute units
Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb consumed 27792 of 200000 compute units
test allowlisted_transfer_passes_the_hook ... ok
test stranger_transfer_fails_the_hook ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Tus números de cómputo van a diferir de los míos por unos miles, por la razón que di arriba sobre la búsqueda de bump. Y cuando la transferencia del extraño falla, el log es lo que vale la pena capturar en pantalla, porque es el nombre de tu programa en medio de una transferencia de token:

```
Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb invoke [1]
Program log: Instruction: TransferChecked
Program HookH1FQuTU21GVAjJZDLXPjXWLQFPJ5FLpwGKZLkYQ invoke [2]
Program log: Instruction: Execute
Program log: AnchorError thrown in src/lib.rs:261. Error Code: NotAllowed. Error Number: 6001.
    Error Message: Destination is not on the hook allowlist.
Program HookH1FQuTU21GVAjJZDLXPjXWLQFPJ5FLpwGKZLkYQ failed: custom program error: 0x1771
Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb failed: custom program error: 0x1771
```

Lee otra vez las dos últimas líneas. Tu error se propagó fuera de `Execute` a profundidad de invocación 2 y mató el `TransferChecked` de Token-2022 a profundidad 1. 0x1771 es 6001, que es el offset de errores de Anchor más el índice de `NotAllowed` en tu enum. Una nota de calibración antes de que compares este log con el tuyo: el número de línea `src/lib.rs:261` sigue TU archivo, no el mío, porque el gate lo escribiste tú, y la pubkey del destination es el keypair que haya generado tu corrida. Lo que tiene que coincidir exactamente es el par de nombres de instrucción, `Error Code: NotAllowed. Error Number: 6001.`, y el 0x1771 en las dos líneas de cierre. El programa de token no tiene ninguna opinión sobre tu allowlist ni forma de anularla. Preguntó, dijiste que no, la transferencia se fue.

## Challenge

Solo, sin apoyo: haz que el log de tesorería sea real.

Ahora mismo `Execute` escribe un `msg!` y sigue adelante. Tu hook ya reenvía la cuenta del log de tesorería en cada transferencia, declarada writable en el manifiesto, sin hacer nada. Cambia eso. En cada transferencia permitida, actualiza `TreasuryLog` en el lugar: incrementa `transfers`, suma `amount` a `total_amount` con aritmética verificada, y registra `last_amount` y `last_destination`. Después extiende el harness para demostrarlo: tras dos transferencias permitidas de tamaños distintos, deserializa la cuenta del log y haz assert de los cuatro campos. Después corre una transferencia no permitida y haz assert de que el log NO se movió, que es el assert que de verdad importa, porque demuestra que el revert deshizo tu escritura junto con la transferencia.

Se acepta cuando: `cargo test -p harvest-hook` está verde con tus asserts nuevos, el contador sobrevive dos transferencias, y una transferencia rechazada deja cada campo intacto. Rómpelo una vez a propósito para estar seguro, poniendo el destination en la allowlist y haciendo assert del total equivocado.

También hay un ejercicio enfocado solo en el núcleo de la decisión: `hook-execute-gate`, el coding challenge de esta lección en el panel de challenges que tiene la plataforma del curso, donde implementas `hook_execute(destination_allowed, is_paused, amount)` contra cinco casos. Dos de ellos existen específicamente para atrapar el bug de ordenamiento: un hook pausado tiene que rechazar un destino de la allowlist, y la pausa tiene que ganar cuando las dos condiciones son hostiles. Si tu `gate` pasó el harness pero quieres estar seguro de que la precedencia está bien en tu cabeza, haz ese primero; toma dos minutos.

## Checkpoint

El criterio de esta lección es un comando y una afirmación.

```bash
cargo build-sbf && cargo test -p harvest-hook -- --nocapture
```

Verde en las dos pruebas, con `harvest-hook: allowed` en el log que pasa y `Error Code: NotAllowed` en el que falla. Si hiciste el challenge, más tus cuatro asserts del log. La afirmación que deberías poder hacer sin consultar nada: la cuenta de validación para el mint M y el programa de hook H es la PDA de los seeds `["extra-account-metas", M]` en H, y hay exactamente una por mint.

Cuatro formas en que este build sale mal, juntas en un solo lugar porque son las que cuestan horas en vez de minutos:

![Cuatro trampas de hooks emparejadas con causa y arreglo: program id equivocado en la PDA, cuentas de tenedor sin dimensionar, esperar que Execute mueva fondos, y retroadaptar una extensión de creación.](assets/v08-comparison.png)

Dos fallas que espero durante la corrida misma, para que puedas autodiagnosticarte en vez de bisecar.

Si la transferencia revierte antes de que `Execute` llegue a loguear algo, estás en el borde de falla A: la lista de cuentas está mal. Revisa primero la lista en `hooked_transfer` — cada extra del manifiesto presente, más el programa del hook y la cuenta de validación; lo que importa es la presencia, ya que Token-2022 resuelve los extras por pubkey, no por posición — y revisa que `ExtraAccountMetaList::size_of` coincida con el número de entradas que devuelve `harvest_metas`. Un desajuste de tamaño corrompe la lista sin ruido en el init y solo aparece aquí.

Si `cargo build-sbf` tiene éxito pero el harness no encuentra el programa, revisa `SO_PATH`. `cargo test` corre con la raíz del paquete como directorio de trabajo, así que `target/deploy/harvest_hook.so` es correcto para el layout de arriba y está mal si anidaste el crate dentro de un workspace con un directorio target compartido. Si lo anidaste, apunta `SO_PATH` al target del workspace en su lugar.

![La escalera de artefactos va de decode-mint al mint SPROUT terminado al harvest-hook de esta lección, y después al resolvedor del cliente, la enrutabilidad y el enrutamiento de comisiones.](assets/v09-timeline.png)

Tómate el hito. Has escrito un programa de Solana que el software de otras personas ahora está obligado a llamar, y lo demostraste contra un programa de token real con una transferencia real. Ese es un tipo de artefacto distinto de todo lo demás en este curso: SPROUT es una configuración, `harvest-hook` es código con una dirección, y la diferencia es que el código puede decir que no.

Y ahí es donde se pone incómodo. Tu hook pasa su harness, pero el harness le entregó cada cuenta en bandeja. En el momento en que una billetera real o un DEX arma una transferencia simple de cuatro cuentas de tu SPROUT con hook, las cuentas que tu programa necesita simplemente no están en la transacción, y la transferencia revierte antes de que tu lógica llegue a correr. Entonces, ¿quién se supone que las pone ahí, y por qué tanta parte del ecosistema decidió que la respuesta es "no nosotros"? La próxima lección te sientas en la silla del integrador, ves ese revert ocurrir, y escribes el resolvedor que lo arregla.
