# Desmonta el framework I: reconstruye el vault nativo

La lección pasada demostraste un build. Corriste `solana-verify` localmente, después verificaste-desde-el-repo contra tu programa en devnet, y los dos hashes coincidieron. Viste el flujo de autoridad solo-de-mainnet pasar por OtterSec y Squads, demostrado y claramente etiquetado como la cosa que no tocas en devnet. El swap, R4, está entregado y es demostrable. Ese es un hito de verdad: cualquiera puede verificar ahora que los bytes on-chain vinieron de tu código.

Así que acá está la pregunta incómoda. Cada línea de `#[account(...)]` en la que te apoyaste para llegar ahí escribió una verificación por ti. Una verificación de dueño. Una verificación de discriminator. Una verificación de firmante. Nunca las viste, porque la macro de derive las emitió adentro de código que nunca leíste. Borra el framework y esas verificaciones no desaparecen. Se vuelven líneas que o escribes a mano o te olvidas. Y una verificación olvidada en un programa que custodia lamports no es un bug, es un exploit.

Esa es esta lección: vas a reconstruir R2, el quarter-vault, sin ningún Anchor, sobre `pinocchio` crudo, manteniendo el mismo comportamiento y la misma barrera de aceptación. Los lamports salen de un PDA bajo autoridad del programa, y un over-withdraw se rechaza. Ninguna macro a la vista.

Haz el scaffold ahora, para que el proyecto exista mientras lees:

```bash
cargo new native-quarter-vault --lib
cd native-quarter-vault
# These three pin as a set: pinocchio-system 0.4 requires pinocchio ^0.9, and
# pinocchio-pubkey 0.3 requires ^0.9 too. Checked 2026-08-22; see the pins note
# at the end of the lesson before you bump any of them.
cargo add pinocchio@0.9 pinocchio-system@0.4 pinocchio-pubkey@0.3
# This crate has no anchor-lang in it, so it is the one place in the course where you
# pin litesvm by name: there is no anchor-v2-testing here to own the version for you.
# Checked 2026-09-01: litesvm 0.16 and solana-sdk 4 build clean on the pinocchio 0.9 line.
cargo add --dev litesvm@0.16 solana-sdk@4
```

Una edición en `Cargo.toml` antes de que cualquier cosa compile a algo desplegable. `cargo new --lib` te da un rlib, y `cargo build-sbf` no va a emitir un `.so` desde eso. Agrega el tipo de crate:

```toml
[lib]
crate-type = ["cdylib", "lib"]
```

`cdylib` es lo que produce el `.so` que carga el runtime; `lib` mantiene el crate usable como una biblioteca de Rust normal — eso sirve para pruebas unitarias adentro del crate y para rust-analyzer, no para la prueba de integración del paso 5, que nunca linkea tu crate para nada: carga el `.so` compilado adentro de LiteSVM con `add_program_from_file` y lo maneja por el wire, exactamente como lo haría un validador. Anchor escribe esta línea por ti en cada scaffold, que es por lo que nunca la tipeaste.

Una cosa sobre cómo corre esta lección. Esos cuatro comandos son toda la tipeada que pide la vista general: de acá hasta el Lab yo recorro la forma y tú lees. En el Lab codificas junto conmigo, paso a paso. El Challenge del final lo haces solo, en frío, sin respuesta en la página. El framework se va saliendo en etapas, y el apoyo debajo de ti también.

## Resumen

Vas a construir `native-quarter-vault`: un programa pinocchio `no_std` con dos instrucciones, `init` y un `withdraw` firmado por PDA, despachadas desde un discriminator de un byte. Vas a escribir a mano la validación que el `Account<T>` de Anchor generaba por ti, firmar una transferencia de lamports fuera de un PDA con `invoke_signed` usando seeds que armas tú mismo, y pasar exactamente el mismo criterio de aceptación que pasó la versión de framework, a través de un banco de pruebas de LiteSVM que escribes desde cero.

`no_std` quiere decir que la biblioteca estándar está apagada. Ningún asignador de heap que no pediste, nada de `std::`, solo `core` y lo que elijas traer. Sé preciso sobre qué es y qué no es esto, porque la versión del folclore exagera: los programas de Solana comunes — cada programa Anchor 1.x incluido — están linkeados con `std`. El toolchain SBF entrega un `std` funcional, aunque recortado, y el entrypoint de fábrica instala un asignador bump default; esa es exactamente la maquinaria escondida que `no_std` rechaza. Así que este no es "el modo en el que los programas de Solana corren de verdad". Es el modo que V2 y pinocchio *eligieron*, y prenderlo tú mismo no es una elección estética: es lo que mantiene el binario chico y el compute predecible, porque no se trae nada que no pidieras explícitamente. Acá das vuelta esa llave en la primera línea del archivo.

El trade-off es el punto entero del módulo: el pinocchio nativo compra de vuelta compute y te entrega control total, pero ahora escribes a mano y nunca puedes olvidar cada verificación que generaba el derive. Discriminator, dueño, duplicate-mutable, firmante, aritmética verificada. Omite una y entregaste una vulnerabilidad. Es precisamente por esto que existe el framework. No estás aprendiendo que Anchor es malo, sino exactamente lo que cuesta ganárselo, para que puedas decidir cuándo la CU vale el riesgo.

## Qué estaba haciendo el derive por ti

Acá está la resonancia que vuelve este ejercicio más que una acrobacia. Anchor V2, la RC sobre la que has estado construyendo todo el curso, `anchor-next` en `2.0.0-rc.1`, es él mismo una reescritura `no_std` desde cero construida sobre pinocchio: su crate `lang-v2` depende de `pinocchio` y de `pinocchio-system 0.6` directamente. Los crates que acabas de agregar a mano son esa misma fundación, dos líneas de minor atrás (el árbol de V2 corre la línea pinocchio 0.11, donde los tipos centrales ya se renombraron; más sobre eso en la nota de pins del final). Cuando reconstruyes el vault crudo, no estás haciendo un juguete sin relación: estás escribiendo a mano exactamente la capa que el framework ahora genera.

¿Por qué V2 fue por este camino? Lee la issue #4390 de GitHub, el manifiesto de zero-copy que dio forma a la reescritura. Llama al `Account<T>` de hoy "the slow path" y "the #1 performance complaint from Anchor developers." La tesis entera de V2 es que el wrapper ergonómico de cuenta hace trabajo de deserialización y de asignación que muchas veces no necesitas, y que una fundación más flaca y zero-copy debería ser el default. Desmontar el framework a mano rima con la propia decisión de diseño de V2. Estás siguiendo el mismo razonamiento que siguieron los mantenedores, una capa más abajo.

Déjame anclar el mecanismo nuevo al que ya conoces. En Anchor, esta era tu cuenta de vault:

```rust
#[account(
    mut,
    seeds = [b"vault", authority.address().as_ref()],
    bump = vault.bump,
    constraint = vault.authority == *authority.address(),
)]
pub vault: Account<Vault>,
```

Cada atributo de esa struct es una verificación que la macro convierte en código de runtime antes de que corra el cuerpo de tu instrucción. `Account<Vault>` solo son tres verificaciones: la cuenta es propiedad de tu programa, sus primeros 8 bytes coinciden con el discriminator de `Vault`, y los bytes restantes se castean a un `Vault`. `seeds` más `bump` re-deriva el PDA y confirma la dirección. El hook `constraint` confirma que el campo de authority guardado es igual a la cuenta de authority pasada. Eso es bastante seguridad empacada en seis líneas.

(Dos escrituras de V2 que vale volver a notar, porque son exactamente las que mapea el módulo de migración: los wrappers soltaron sus lifetimes `<'info>`, y `.key()` se volvió `.address()`. La verificación de clave guardada en sí solía ser la keyword `has_one = authority`; V2 la deprecia a favor de las formas de expresión `address = ...` y `constraint = ...` que encontraste en el módulo 3; la keyword todavía parsea, con una advertencia.)

![Una tabla que mapea cada atributo de cuenta de Anchor a la verificación explícita de pinocchio que lo reemplaza y al bug específico que aparece si omites esa verificación.](assets/v01-comparison.webp)

Lee la columna de la derecha una vez más, porque esas no son hipótesis: cada una es una clase de exploit que drenó programas reales, y cada una es un solo `if` que Anchor escribía y tú no. Ahora las escribes tú.

### El modelo de cuentas, y por qué son dos PDA

R2 en Anchor era lo bastante ordenado para sentirse como una sola cosa. Nativo, ves la costura que el framework estaba tapando: un vault de custodia de lamports y su registro de autoridad son dos cuentas distintas, propiedad de dos programas distintos, y eso no es una complicación, es la verdad que Anchor estaba escondiendo.

Acá está el constraint que lo fuerza. El System Program solo va a mover lamports fuera de una cuenta de la que es dueño, y esa cuenta tiene que tener datos cero. Así que la cuenta que de verdad sostiene el SOL tiene que ser propiedad del System y estar vacía. Pero también necesitas algún lugar para guardar tu propio estado: el discriminator, la authority, el bump canónico. Eso tiene que ser una cuenta de la que tu programa es dueño y puede escribir. Una cuenta no puede ser las dos. Así que el vault es un par.

- El PDA de **config**, seeds `[b"config", authority]`, propiedad de tu programa. Guarda `[discriminator][authority][vault_bump]`. Este es tu análogo de `Account<Vault>`, la cosa que validas.
- El PDA de **vault**, seeds `[b"vault", authority]`, propiedad del System Program, que sostiene el SOL custodiado. Tu programa nunca escribe sus datos (no hay ninguno). Firma para mover sus lamports.

![Un diagrama de dos PDA desde una authority: un config propiedad del programa que sostiene estado y un vault propiedad del System que sostiene SOL, retirado firmando con invoke_signed.](assets/v02-diagram.webp)

Si te estás imaginando R2 como un único `Account<Vault>` que tanto guardaba el bump como sostenía lamports, estaba haciendo el truco del lamport directo: debitando el saldo de su propia cuenta porque el programa era su dueño. Eso funciona, pero no es una transferencia firmada, y no es lo que queremos enseñar acá. El camino del invoke_signed, donde un PDA presenta sus seeds para autorizar una transferencia de System de verdad, es el patrón al que vas a recurrir constantemente (vaults de token, escrows, cualquier cosa donde el PDA tenga que ser un firmante de CPI). Así que construimos la versión que firma.

### El discriminator: un byte, estructural

Anchor gasta 8 bytes en un discriminator de cuenta, la etiqueta derivada de SHA-256 que dice "esto es un `Vault`, no un `Config` ni un `Pool`". Nativo, puedes gastar uno. Un solo `u8` te da 255 tipos de cuenta, que es bastante, y el layout es simple hasta decir basta: el byte 0 es la etiqueta, el resto son datos.

![Una franja de layout de 34 bytes para la cuenta de config: byte 0 discriminator, bytes 1 a 32 la pubkey de authority, byte 33 el bump de vault guardado.](assets/v03-annotated-code.webp)

La regla de la casa importa acá y no es opcional: lee y escribe estos campos como slices de array de bytes con lógica de accesor, nunca casteando el buffer crudo a una struct packed con un puntero desalineado. Un `&*(ptr as *const Config)` sobre una struct `#[repr(C, packed)]` produce referencias desalineadas, que es comportamiento indefinido en Rust y una de las trampas más afiladas del ecosistema nativo entero. Los slices con `copy_from_slice` y `from_le_bytes` son seguros, obvios, y solo un pelo más lentos, así que eso es lo que vamos a usar en todas partes.

Ahora imagina el ataque exacto que detiene el discriminator, porque una clase de bug que puedes ver es una clase de bug que vas a recordar. Supón que Mallory encuentra alguna otra instrucción de tu programa que por casualidad crea una cuenta propiedad del programa de 34 bytes para un propósito sin relación, una fila del marcador, digamos, del mismo largo que tu config. Si tu withdraw se salteara la verificación del discriminator, ella podría pasar esa fila del marcador al slot al que pertenece el config. La verificación de dueño pasaría, porque tu programa genuinamente es dueño de la fila. La verificación de largo pasaría también, porque tiene 34 bytes. Tu código entonces leería los bytes 1 a 32 como una authority y el byte 33 como un bump, desde datos que nunca fueron un config de vault para empezar. Ahora sé honesto sobre qué pasa después *en este programa en particular*, porque es más sutil que un drenaje. La barrera 5 compara esos bytes contra la clave que de verdad firmó, así que una fila que Mallory pueda usar tiene que llevar su propia pubkey en los bytes 1 a 32 — y entonces las seeds del withdraw, `[b"vault", her key, bump]`, derivan su propio vault, el mismo del que ya podía retirar legítimamente. Las otras barreras por casualidad contienen el daño acá. Pero esa contención es acoplamiento, no diseño: se sostiene solo porque el esquema de seeds de este programa amarra cada vault a la misma clave que verifica la barrera 5. Reusa esta forma de despacho en un programa donde el config nombra a un beneficiario, un destino de tarifa, o cualquier authority que no sea quien firma, y la fila fabricada se vuelve un robo de verdad. El único byte en el offset 0 es lo que vuelve irrelevante la pregunta entera: una fila del marcador lleva otra etiqueta, la verificación falla, y la transacción revierte antes de que se mueva un lamport. Eso es type cosplay, y el discriminator es la revisión de disfraces en la puerta — la verificación que mantiene a cada barrera de más adelante queriendo decir lo que dice en vez de apoyarse en sus vecinas. Sostén el mapeo: el discriminator es la etiqueta de tipo de la cuenta, y la verificación de que la etiqueta coincide es la línea exacta que Anchor escribe antes de siquiera entregarte datos tipados.

### Firmar como un PDA: seeds más el bump guardado

Un PDA no tiene clave privada. "Firma" una CPI presentando, a la hora de la llamada, las seeds exactas de las que fue derivado más su bump, y el runtime reconstruye la dirección para confirmar que el programa tiene permitido firmar por él. Ese es el truco entero, y es la parte que Anchor arma por ti. Sé preciso sobre *cuándo*, porque es la misma distinción que trazó m03-l1: Anchor precomputa el bump canónico en tiempo de macro solo cuando cada seed es un literal de byte. Las seeds de este vault llevan la clave de la authority, así que del lado de Anchor el bump se derivaba durante la validación y después se guardaba — que es exactamente el byte que estás a punto de leer de vuelta a mano.

Nativo, armas el firmante tú mismo. Y usas el bump canónico **guardado**, el que salvaste en el init, no uno recién derivado. Volver a correr `find_program_address` adentro de cada instrucción quema alrededor de 1500 unidades de cómputo por llamada, porque muele candidatos de bump desde 255 hacia abajo buscando el que está fuera de la curva. Pagaste por eso una vez en el init. Guárdalo, reúsalo. Anchor lo guarda en la cuenta y lo lee de vuelta a través de `bump = vault.bump`; tú haces lo mismo a mano.

![Un diagrama de flujo que muestra el withdraw leyendo el bump guardado, armando las seeds, llamando invoke_signed, el runtime re-derivando la dirección, y ejecutando la transferencia solo si coincide con la clave del vault.](assets/v04-flowchart.webp)

### La verificación que agregó V2: sin mutables duplicadas

Hay una verificación más en la lista, y es la más nueva, así que vale destacarla por su cuenta. Anchor V2 no permite cuentas mutables duplicadas por defecto, una guarda que las versiones más viejas te dejaban por completo a ti. Acá está por qué se gana su lugar. Tu withdraw toma un `vault` escribible y un destino `authority` escribible, y nada de lo que escribiste hasta acá detiene a un llamador de pasar la misma cuenta para las dos. Si `vault` y `authority` resuelven a la misma clave, estás moviendo lamports de una cuenta hacia ella misma, y dependiendo de la lógica envuelta alrededor de una transferencia así puedes terminar contando un saldo dos veces o pasando de largo una guarda que calladamente asumió que las dos cuentas eran distintas. El arreglo nativo es una comparación de claves que agregas a mano, `if self.vault.key() == self.authority.key() { return Err(ProgramError::InvalidArgument); }`, puesta en el mismo corredor de barreras del `TryFrom` que las otras cinco. Anchor V2 escribe esa guarda a través de tu struct entera, calladamente, en tiempo de compilación — y su regla es más amplia que la verificación de pares hecha a mano: una cuenta aliasada falla la validación en el momento en que *cualquiera* de sus slots es mutable, no solo cuando dos slots mutables colisionan. Está en la lista precisamente porque olvidarla mordió a suficiente gente para que los mantenedores la prendieran para todos.

Esa es la forma. Vista general lista. Ahora constrúyelo.

## Lab: construye el quarter-vault nativo

Codifica junto conmigo de acá en adelante. Cada paso es una edición de verdad; los checkpoints te dicen cómo se ve "funcionando".

### Paso 1: apaga std y arma el despacho

Abre `src/lib.rs` y reemplázalo por entero. La primera línea es la que Anchor nunca te dejó ver.

```rust
// Not a bare `#![no_std]`. The on-chain build has no std; the host test harness in
// step 5 does, and needs it. `cfg_attr` turns no_std on for every build except the
// test one. Anchor's entrypoint does the same thing behind your back.
#![cfg_attr(not(test), no_std)]

use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::{CreateAccount, Transfer};

// Generate the keypair, print its pubkey, and paste that string BOTH here and in
// the test's PROGRAM_ID const in step 5. They have to be the same or the test loads
// your program at an address `crate::ID` does not recognize, and gate 2 rejects
// every account with IncorrectProgramId for a reason that has nothing to do with
// your code:
//   mkdir -p target/deploy
//   solana-keygen new --no-bip39-passphrase \
//     -o target/deploy/native_quarter_vault-keypair.json
//   solana address -k target/deploy/native_quarter_vault-keypair.json
pinocchio_pubkey::declare_id!("<paste your generated pubkey>");

/// Account type tag. Anchor spends 8 bytes on this; we spend one.
const VAULT_DISCRIMINATOR: u8 = 1;

/// [disc:1][authority:32][vault_bump:1]
const CONFIG_LEN: usize = 1 + 32 + 1;

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    match data.split_first() {
        Some((0, rest)) => Init::try_from((rest, accounts))?.process(),
        Some((1, rest)) => Withdraw::try_from((rest, accounts))?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
```

`split_first` pela el primer byte de los datos de instrucción. Ese byte es la **etiqueta de instrucción**: `0` es init, `1` es withdraw. Mantenlo separado en tu cabeza del **discriminator de cuenta** dos líneas arriba, `VAULT_DISCRIMINATOR`, que etiqueta el *tipo* de la cuenta en vez de la llamada. Anchor gasta ocho bytes en cada uno y a los dos les dice discriminator; acá tienen un byte cada uno y los dos por casualidad son números chicos, así que nombrarlos aparte es la única cosa que los mantiene aparte. Esta es tu tabla de despacho, la cosa que la macro `#[program]` de Anchor generaba a partir de los nombres de tus funciones. Checkpoint: `cargo build-sbf` **falla**, y debería. `process_instruction` nombra `Init` y `Withdraw`, que todavía no existen, así que obtienes dos errores de `cannot find type in this scope` y nada más. Ese es el estado esperado después del paso 1; lo que estás confirmando es que los errores son esos dos y no algo sobre `no_std`, el entrypoint, o un import que falta. Los pasos 2 a 4 llenan esos dos nombres, y el primer build verde llega al final del paso 4.

### Paso 2: valida con TryFrom (esta es la parte trabajada, lee cada línea)

Esta es la capa que reemplaza el derive de Anchor, y la estoy mostrando por entero porque olvidar una línea acá es el peligro entero. Agrega la struct `Withdraw` y su `TryFrom`:

```rust
struct Withdraw<'a> {
    authority: &'a AccountInfo,
    vault: &'a AccountInfo,
    vault_bump: u8,
    amount: u64,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Withdraw<'a> {
    type Error = ProgramError;

    fn try_from(
        (data, accounts): (&'a [u8], &'a [AccountInfo]),
    ) -> Result<Self, ProgramError> {
        let [authority, config, vault, _system_program, ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 1. signer check     (Anchor: Signer)
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        // 2. owner check      (Anchor: Account<T> proves program ownership)
        if !config.is_owned_by(&crate::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }

        let cfg = config.try_borrow_data()?;

        // 3. data-length check (Anchor: deserialization would fail on short data)
        if cfg.len() != CONFIG_LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        // 4. discriminator check (Anchor: the account type tag)
        if cfg[0] != VAULT_DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        // 5. stored-authority check (Anchor: constraint = vault.authority)
        if &cfg[1..33] != authority.key().as_ref() {
            return Err(ProgramError::InvalidAccountData);
        }
        // 6. duplicate-mutable check (Anchor V2: on by default)
        if vault.key() == authority.key() {
            return Err(ProgramError::InvalidArgument);
        }

        // read the stored canonical bump before the borrow drops
        let vault_bump = cfg[33];

        let amount = u64::from_le_bytes(
            data.try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        Ok(Withdraw { authority, vault, vault_bump, amount })
    }
}
```

Seis verificaciones. Las primeras cinco se alinean una a una con la tabla de comparación de arriba, y la sexta es la guarda de duplicate-mutable que V2 prende por defecto, agregada acá al mismo corredor. El patrón `let [authority, config, vault..] = accounts else` es tu ordenamiento de cuentas, la cosa que `#[derive(Accounts)]` imponía por el orden de los campos de la struct. Equivócate en el orden acá y todo lo de más adelante lee la cuenta equivocada, que es en sí una trampa que el framework sacó.

![Un diagrama de flujo de falla-rápida de seis barreras del TryFrom, cada una etiquetada con el error que devuelve y el exploit que bloquea, convergiendo en una struct Withdraw validada.](assets/v05-flowchart.webp)

Nota lo que te compra TryFrom: para el momento en que corre `process()`, la validación está hecha y la lógica de negocio nunca vuelve a verificar. Esa separación, validar-después-actuar, es exactamente lo que te da Anchor separando la struct de cuentas del cuerpo de la instrucción, y acabas de construirla a mano.

Checkpoint: `cargo build-sbf` todavía falla, y la lista de errores debería haber encogido en uno: `Withdraw` ahora existe, así que lo que queda es el `Init` que falta más un `Withdraw::process` sin resolver, que escribes a continuación. Fíjate en un error que no está en esa lista: una queja del borrow checker sobre `cfg` quiere decir que tu lectura de `let vault_bump = cfg[33];` se escapó del alcance donde la guarda de `try_borrow_data` está viva. Copia el byte afuera mientras el borrow está sostenido, exactamente donde lo pone el comentario, en vez de intentar leer a través de la guarda después de que se cae.

### Paso 3: el cuerpo del withdraw, y el completion que llenas

Ahora la firma. Este es el paso de Completion: la guarda y las seeds del invoke_signed son las líneas que armas. Agrega el `impl`:

```rust
impl Withdraw<'_> {
    fn process(&self) -> ProgramResult {
        // --- the guard (you harden this cold in the Challenge) ---
        if self.amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }
        // Bound only. The transfer below does the actual debit; this line exists
        // to turn an over-withdraw into a clean error instead of a failed CPI.
        let _remaining = self
            .vault
            .lamports()
            .checked_sub(self.amount)
            .ok_or(ProgramError::InsufficientFunds)?;

        // --- the completion: assemble the signer from seeds + STORED bump ---
        let bump = [self.vault_bump];
        let seeds = [
            Seed::from(b"vault"),
            Seed::from(self.authority.key().as_ref()),
            Seed::from(&bump),
        ];
        let signer = [Signer::from(&seeds)];

        Transfer {
            from: self.vault,
            to: self.authority,
            lamports: self.amount,
        }
        .invoke_signed(&signer)?;

        Ok(())
    }
}
```

El array de seeds es el conjunto exacto del que se derivó el PDA: el literal `b"vault"`, los bytes de pubkey de la authority, y el bump guardado envuelto como su propia seed de un byte. Ese último `Seed::from(&bump)` es el slice del bump guardado. Equivócate en este array, en orden o en contenido, e `invoke_signed` deriva otra dirección, el runtime se niega a firmar, y obtienes un `InvalidSeeds`. Acá no hay crédito parcial del runtime: las seeds están bien o la transferencia no pasa.

La guarda usa `checked_sub`, nunca `self.vault.lamports() - self.amount`. Una resta ingenua hace underflow y panic (o peor, hace wrap) en un over-withdraw. `checked_sub` devuelve `None`, que mapeas a un error limpio. Esta es la verificación de aritmética verificada que Anchor nunca te hizo pensar, porque rara vez restabas saldos de cuenta directo adentro de un constraint.

Checkpoint: queda un error, el `Init` que falta, que provee el paso 4. Un `InvalidSeeds` en esta etapa es imposible, porque todavía no corrió nada; un error de compilación que nombre `Seed` o `Signer` quiere decir que falta la línea de import del Paso 1.

### Paso 4: el init, la creación de cuentas hecha a mano

El init crea la cuenta de config y guarda los bumps. Esta es tu CPI de `CreateAccount`, fondeando los lamports exentos de alquiler, y la firman las propias seeds del PDA de config porque un PDA tiene que autorizar su propia creación.

```rust
struct Init<'a> {
    authority: &'a AccountInfo,
    config: &'a AccountInfo,
    config_bump: u8,
    vault_bump: u8,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Init<'a> {
    type Error = ProgramError;

    fn try_from(
        (data, accounts): (&'a [u8], &'a [AccountInfo]),
    ) -> Result<Self, ProgramError> {
        let [authority, config, _system_program, ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let [config_bump, vault_bump] = data else {
            return Err(ProgramError::InvalidInstructionData);
        };
        Ok(Init { authority, config, config_bump: *config_bump, vault_bump: *vault_bump })
    }
}

impl Init<'_> {
    fn process(&self) -> ProgramResult {
        // Blunt reinit guard: rejects ANY lamports at the address, including a
        // stranger's 1-lamport pre-fund (a griefing seam Anchor's init tolerates
        // by funding the shortfall instead -- see the note below the code).
        if self.config.lamports() > 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let bump = [self.config_bump];
        let seeds = [
            Seed::from(b"config"),
            Seed::from(self.authority.key().as_ref()),
            Seed::from(&bump),
        ];
        let signer = [Signer::from(&seeds)];

        let rent = Rent::get()?;
        CreateAccount {
            from: self.authority,
            to: self.config,
            lamports: rent.minimum_balance(CONFIG_LEN),
            space: CONFIG_LEN as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&signer)?;

        // write [disc][authority][vault_bump] as bytes, no packed-struct cast
        let mut cfg = self.config.try_borrow_mut_data()?;
        cfg[0] = VAULT_DISCRIMINATOR;
        cfg[1..33].copy_from_slice(self.authority.key().as_ref());
        cfg[33] = self.vault_bump;
        Ok(())
    }
}
```

Esa primera guarda está haciendo un trabajo que el framework hacía con más cuidado, y la diferencia vale nombrarla con precisión. El `init` simple de Anchor se niega a correr sobre una cuenta que ya está *inicializada* — datos asignados, o un dueño ajeno — pero deliberadamente acepta una meramente pre-fondeada y sin asignar: su código generado completa el faltante de alquiler, asigna y adjudica, precisamente para que un extraño tirando un lamport por airdrop a la dirección de tu PDA no pueda inutilizar tu init para siempre. La barrera bruta de `lamports() > 0` de arriba compra protección de reinit al costo de volver a abrir exactamente esa costura de griefing. Para un vault didáctico el trato es aceptable, dicho en voz alta: la señal en la que Anchor de verdad se apoya es la asignación y la propiedad, no el saldo, y un init nativo endurecido verifica `data_len() == 0` más propiedad del System y *fondea el faltante* en vez de negarse. `init_if_needed` es una trampa completamente distinta — permite calladamente la re-inicialización de estado vivo, que es por lo que las reglas de la casa lo prohíben. Nativo, cada una de estas distinciones es un `if`, y cada una es tuya para recordar.

Los bumps llegan en los datos de instrucción, computados del lado del cliente por `find_program_address`. Eso mantiene la derivación carísima fuera de la cadena y fuera de tu presupuesto de compute, y es por eso que el cliente, no el programa, hace la molienda. Checkpoint: `cargo build-sbf` compila limpio ahora, primer build verde de la lección, y `target/deploy/native_quarter_vault.so` existe. Si no hay `.so`, falta la línea de `crate-type` del paso del scaffold.

### Paso 5: la barrera de aceptación

Ahora el mismo *criterio* de aceptación que pasó R2, a través de un banco nuevo. No el mismo archivo: la prueba de R2 armaba datos de instrucción y structs de cuenta generados por Anchor, y ninguno de los dos existe acá, así que las aserciones pasan adelante y el banco se escribe desde cero. LiteSVM es la misma VM de Solana in-process que has usado desde el módulo 2: sin validador, sin ledger, solo tu programa cargado en un bank que manejas desde Rust. La agregaste con `cargo add --dev litesvm@0.16 solana-sdk@4` al principio. Crea `tests/withdraw.rs`:

```rust
use litesvm::LiteSVM;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

// The SAME string you pasted into declare_id! in step 1.
const PROGRAM_ID: Pubkey = pubkey!("<paste your generated pubkey>");

#[test]
fn withdraw_signed() {
    let mut svm = LiteSVM::new();
    let program_so = concat!(env!("CARGO_MANIFEST_DIR"), "/target/deploy/native_quarter_vault.so");
    svm.add_program_from_file(PROGRAM_ID, program_so).unwrap();

    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();

    let (config, config_bump) =
        Pubkey::find_program_address(&[b"config", authority.pubkey().as_ref()], &PROGRAM_ID);
    let (vault, vault_bump) =
        Pubkey::find_program_address(&[b"vault", authority.pubkey().as_ref()], &PROGRAM_ID);

    // init
    let init_ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(config, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: vec![0, config_bump, vault_bump],
    };
    let bh = svm.latest_blockhash();
    let tx =
        Transaction::new_signed_with_payer(&[init_ix], Some(&authority.pubkey()), &[&authority], bh);
    svm.send_transaction(tx).unwrap();

    // fund the SOL vault PDA (this teaching build has no deposit instruction)
    svm.airdrop(&vault, 2_000_000_000).unwrap();

    // a valid PDA-signed withdraw succeeds
    let mut data = vec![1u8];
    data.extend_from_slice(&500_000_000u64.to_le_bytes());
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    };
    let before = svm.get_balance(&vault).unwrap();
    let bh = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority.pubkey()), &[&authority], bh);
    svm.send_transaction(tx).unwrap();
    assert_eq!(before - svm.get_balance(&vault).unwrap(), 500_000_000);

    // an over-withdraw is rejected
    let mut data = vec![1u8];
    data.extend_from_slice(&999_000_000_000u64.to_le_bytes());
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    };
    let bh = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority.pubkey()), &[&authority], bh);
    assert!(svm.send_transaction(tx).is_err());
}
```

Haz el build del programa a un `.so`, y después corre la prueba:

```bash
cargo build-sbf
cargo test -p native-quarter-vault
```

Quieres ver:

```
test withdraw_signed ... ok
```

Esa es la misma barrera. Los lamports salieron del PDA bajo autoridad del programa, y el over-withdraw fue rechazado. Tu vault nativo hace exactamente lo que hacía el vault de framework. Quédate con eso un segundo: sin `#[account]`, sin `#[program]`, sin magia de `declare_id!` más allá de un const, y la prueba de aceptación no nota la diferencia.

![Una línea de tiempo que va desde la queja de zero-copy de la #4390, a pinocchio como una fundación flaca, a la reescritura no_std de Anchor V2, a la reconstrucción a mano de esta lección.](assets/v06-timeline.webp)

Antes de que salgas del Lab, mira de vuelta el trade-off del modelo de cuentas. Dos PDA y unas ochenta líneas te compraron lo que seis atributos de Anchor daban gratis, más el compute que ahorraste por no deserializar a través de `Account<T>`. Ese es el trato que ofrece el nativo, y es un trato de verdad, no un reto. El próximo visual es el que hay que capturar, porque es la respuesta a "cuándo vale esto".

![Una tabla de decisión que compara Anchor V2 y el pinocchio nativo a través de verificaciones, compute, auditabilidad, modo de falla, y cuándo elegir cada uno.](assets/v07-table.webp)

## Challenge: la guarda de withdraw del vault nativo

Ahora vas Solo. Sin respuesta en la página.

En el Lab, la guarda viajaba adentro de `process()`. El Challenge te hace escribirla en frío, como una función pura autónoma con códigos de retorno explícitos, de la forma en que la picotearía un fuzzer o una prueba unitaria. Esta es la lógica exacta que está entre tu vault y un underflow, extraída para que puedas verla con claridad.

El starter, en `lessons/m09-l1/native-vault-withdraw/starter.rs`:

```rust
/// Compute the vault's remaining balance after a withdraw, or an error code.
///
/// Contract (an i128 so the harness can signal failure without a Result):
///   -2  : the withdraw amount is zero (invalid input)
///   -1  : the withdraw exceeds the balance (over-withdraw)
///   >=0 : the remaining balance after a valid withdraw
///
/// The starter skips BOTH guards and subtracts naively. It happens to return the
/// right number for a normal withdraw, and is wrong (and unsafe) for both rejects.
const fn vault_withdraw(balance: u64, amount: u64) -> i128 {
    // TODO: reject a zero-amount withdraw with -2
    // TODO: reject an over-withdraw with -1 using balance.checked_sub(amount)
    (balance as i128) - (amount as i128)
}
```

El `const fn` está haciendo trabajo sin framework: el archivo calificado lleva aserciones de tiempo de compilación debajo de la función — el dispositivo de m03-l3, y lo que la calificación solo-de-compilación impone de verdad — así que el starter sin guarda no compila, y cada aserción que falla nombra el caso de rechazo que representa. Apropiado, para la lección donde escribes cada verificación a mano: acá hasta el banco no es nada más que el compilador.

Criterios de aceptación:

- un withdraw de monto cero devuelve `-2`, y se rechaza antes de la verificación de saldo
- un over-withdraw devuelve `-1`, computado vía `checked_sub`, nunca una resta ingenua — incluido `(100, 102)`, donde un `balance as i128 - amount as i128` ingenuo produce `-2`, que es el código de *monto cero*. Los centinelas son decisiones, no aritmética; si tu resta puede producir uno, no tienes una guarda
- retirar de un vault *vacío* es un over-withdraw, `-1`, no un rechazo de monto cero: la guarda del cero está sobre el monto, nunca sobre el saldo
- un withdraw válido devuelve el saldo restante, a través del rango entero de `u64` — `(u64::MAX, 1)` devuelve `u64::MAX - 1`, que es por lo que el retorno es `i128` y por lo que hacer la resta en `i64` falla
- drenar hasta exactamente cero está permitido (el PDA de SOL de este vault didáctico no lleva datos y el challenge es deliberadamente agnóstico del alquiler; un withdraw de producción también pondría un piso en el mínimo exento de alquiler, que es lo que hacía la propia guarda de R2)

Dos pistas, y después es tuyo. Primera: un withdraw de monto cero es entrada inválida, así que recházalo antes de tocar el saldo. Segunda: `balance.checked_sub(amount)` devuelve `None` exactamente cuando el withdraw excede el saldo, que es tu señal de over-withdraw. Compila hasta que las aserciones dejen de dispararse, y después corre los vectores de `tests.json` hasta que cada caso esté verde.

Si te equivocas en el orden, mira qué aserción falla. Un monto cero que devuelve `0` en vez de `-2` quiere decir que restaste antes de rechazar. Ese bug de orden es el mismo que se entrega en programas reales, así que aprender a verlo acá es el punto.

## Dónde aterrizaste, y qué viene

Acabas de borrar el framework y el vault sigue funcionando. Escribiste las seis verificaciones que el derive escribía por ti, en el orden que importa, y ahora puedes apuntar a cada línea y nombrar el exploit que bloquea. Firmaste una transferencia de lamports fuera de un PDA con seeds que armaste a mano y un bump que guardaste en vez de re-derivar. Y pasaste la misma barrera de LiteSVM sobre pinocchio crudo que pasaste sobre Anchor V2. Esa es la fundación sobre la que está construido el framework, en tus propias manos.

Una nota honesta sobre los pins, y es la lección de disciplina de versiones más afilada de este módulo, así que léela en vez de pasarle el ojo. Todo lo de arriba está verificado contra `pinocchio 0.9` / `pinocchio-system 0.4` / `pinocchio-pubkey 0.3`, Rust `1.89.0`, y Anchor V2 `2.0.0-rc.1`, verificado el 2026-08-22. Esos tres crates de pinocchio tienen que moverse como un conjunto: `pinocchio-system 0.4` declara `pinocchio ^0.9`, y `pinocchio-pubkey 0.3` también. Mezcla un minor tipo-major y cargo va a resolver felizmente dos copias de `pinocchio` adentro de tu árbol, momento en el que el `AccountInfo` que te entrega tu entrypoint es un tipo distinto del que quiere `Transfer`, y el mensaje de error va a ser sobre trait bounds en vez de sobre versiones.

El crate es pre-1.0 y la API está genuinamente moviéndose, y ya se movió una vez más allá de donde estás parado. El rename no es un rumor sobre una rama main: **se entregó**. Sobre la línea `pinocchio 0.11` (0.11.0, 2026-04-08), `AccountInfo` se volvió `AccountView`, `Pubkey` se volvió `Address` (de `solana-address`), `is_owned_by` se volvió `owned_by`, `try_borrow_data`/`try_borrow_mut_data` se volvieron `try_borrow`/`try_borrow_mut`, y `Seed`/`Signer` se movieron de `instruction` hacia `cpi`. Esa línea 0.11 es exactamente sobre lo que se sienta el propio `lang-v2` de Anchor V2, vía `pinocchio-system 0.6`, que es por lo que el código V2 que has estado escribiendo todo el curso dice `.address()` y no `.key()`. Este lab fija la línea 0.9 deliberadamente, porque es la última línea donde `pinocchio-pubkey` todavía resuelve y donde los nombres pre-rename vuelven legible el mapeo uno-a-uno con tus instintos viejos de la era Anchor V1. Pórtalo a 0.11 como ejercicio y vas a sentir el rename dos veces: una en los nombres de tipo, una en `try_borrow_mut` tomando `&mut self`, que es la API diciéndote que dejó de fingir que la mutabilidad interior era gratis.

Así que: fija exacto, mueve los tres como un conjunto, y vuelve a leer la nota de pins antes de subir. Seguir un latest flotante en un crate pre-1.0 es cómo te enteras por las malas de que un nombre de tipo es una versión.

Tu vault nativo funciona. ¿Pero cuáles de tus líneas escritas a mano generaba Anchor gratis, y cuáles volvía imposibles de equivocar para empezar? La próxima lección ponemos tu `lib.rs` lado a lado con la expansión de macro de verdad y leemos el diff línea por línea. Vas a descubrir exactamente qué tan cerca llegó tu reconstrucción a mano, y dónde el framework te protege calladamente de formas que no replicaste.
