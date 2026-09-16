# El catálogo de constraints (y init_if_needed, in situ)

La lección pasada derivaste el PDA del quarter-vault y guardaste su bump canónico. Crea y vuelve a derivar, que es progreso real. Pero mira lo que todavía hace: `init_vault` confía en cada cuenta que le pasa quien llama. Nada verifica que la cuenta que *crees* que es el config sea el config, que el firmante que paga una acción de admin sea de verdad el operador del arcade, ni que una cuenta cualquiera pasada donde va una cuenta de token tenga como dueño al programa que supones que es su dueño. La macro deriva la dirección y después tu handler se encoge de hombros y confía en el resto.

En v1, ese encogimiento de hombros era, de lejos, la forma más común de que a un programa lo vaciaran. Alguien se olvida de una verificación, un atacante pasa su propia cuenta donde se esperaba la tuya, y el handler opera alegremente sobre ella. Así que antes de cualquier teoría, siente el agujero tú mismo. Abre el `programs/quarter-vault/src/lib.rs` de la lección pasada y agrega este handler de admin ingenuo, del tipo que se ve bien en una revisión:

```rust
// Looks fine. Is exploitable. There is no check that `authority` is anyone in particular.
pub fn admin_set_credit(ctx: &mut Context<AdminSetCredit>, new_credit: u64) -> Result<()> {
    ctx.accounts.vault.credit = new_credit;
    Ok(())
}

#[derive(Accounts)]
pub struct AdminSetCredit {
    pub authority: Signer,          // ANY signer. That is the bug.
    #[account(mut, seeds = [b"vault", vault.owner.as_ref()], bump = vault.bump)]
    pub vault: Account<Vault>,
}
```

Corre `anchor build`. Compila limpio. También deja que cualquier billetera de Solana le ponga a cualquier jugador el crédito que quiera, porque `Signer` demuestra que alguien firmó, no *quién*. Guarda esa idea. Esta lección es donde haces que la macro `#[derive(Accounts)]` haga de policía, así una cuenta mala queda rechazada antes de que llegue a correr una línea de tu handler. Y es donde un keyword, `init_if_needed`, vuelve a abrir en silencio una puerta que el framework pasó años atrancando.

## Resumen

Acá está toda la lección como índice de hallazgos, cada línea una conclusión sobre la que puedes actuar:

- Los constraints viven en la **macro derive**, así que la validación corre *antes* de tu handler y aparece en el IDL. Lo que la macro rechaza, tu código nunca lo ve.
- Dos fases importan y su orden es todo el truco: **`load`** (verificaciones de dueño + discriminador) corre *primero*, y los **constraint hooks** (`seeds`, `address`, `owner`, `constraint`, `close`) corren *después*. Una verificación que pones en la fase equivocada nunca se dispara, en silencio.
- El `address = parent.field` es el reemplazo de V2 para el ahora deprecado **`has_one`**. Acepta *cualquier expresión*, no solo una clave guardada, y por eso el keyword más viejo se quedó sin trabajo.
- La **trampa del error de owner**: `#[account(owner = X @ MyErr)]` sobre `Account<T>` *no* va a mostrar `MyErr`. El owner corre en `load`, antes de tu hook, así que en su lugar te llega el `IllegalOwner` del framework. Para un error de owner propio, baja a `UncheckedAccount` y afirma la propiedad con un `constraint` explícito.
- El `close = destination` devuelve la **reserva exenta de renta** de la cuenta al destino e invalida la cuenta, atómicamente, dentro de la macro. Es la forma correcta de devolverle su rent a un jugador.
- Los sub-keywords de realloc se aplanaron: los `realloc::payer` / `realloc::zero` de v1 ahora son **`realloc_payer` / `realloc_zero`**. Mismo comportamiento, forma nueva. La forma vieja no compila.
- El `init_if_needed` se entrega en V2 **sin feature gate** y con validación de reuso real, pero el **ataque de reinicialización** le sobrevive. La validación de reuso vuelve a verificar la estructura, nunca tu estado de negocio. Esa guarda te toca escribirla a ti.

El repliegue corre un punto más afuera que la lección pasada: en el Lab te sigo entregando el programa completo resguardado por constraints y dos pruebas que pasan, pero el vault en sí, sus seeds y su bump guardado, ahora llega sin volver a explicarse. En el problema de completion vuelvo a sacar los constraints `address` y `close` como TODOs y tú los vuelves a llenar. En el Solo construyes tú mismo el camino de `init_if_needed` y escribes la prueba que demuestra que no se puede abusar de él.

## El catálogo de constraints, keyword por keyword

Arranca por la máquina, no por los keywords, porque la máquina explica por qué uno de los keywords es una trampa. Cuando una transacción llega a tu programa, Anchor no salta directo a tu handler. Primero *construye* el struct de cuentas, y esa construcción pasa en dos fases ordenadas.

La primera fase es **`load`**. Para cada campo `Account<T>`, el framework lee la cuenta cruda, verifica que su **dueño** sea tu programa, y verifica que los primeros bytes calcen con el **discriminador** de `T` (la etiqueta que dice "esto es un `Vault`, no un `Config`"). Si cualquiera de las dos verificaciones falla, `load` aborta de inmediato con un error incorporado. La segunda fase son los **constraint hooks**: `seeds`, `bump`, `address`, `owner`, `constraint`, `has_one`, `close`, y compañía. Estos corren *después* de que cada campo cargó. Un "constraint hook" es solo el código generado que emite la macro para aplicar una cláusula `#[account(...)]`, y corre en la segunda fase, nunca en la primera.

Glosario, porque se paga solo en treinta segundos: el **constraint hook** es el paso de aplicación de un solo constraint, que corre después de `load`. Mantén ese orden en la cabeza. Casi todo filo de este catálogo es consecuencia de él.

![El struct derive valida en dos fases: el load verifica el dueño y el discriminador y puede salir con errores incorporados, y después corren los constraint hooks antes del handler, que es por qué un constraint owner que lleva un error propio solo se dispara sobre una cuenta sin verificar.](assets/v01-flowchart.webp)

### address = parent.field: la barrera de autoridad, y por qué has_one se quedó sin trabajo

Tu `admin_set_credit` ingenuo necesita exactamente una cosa: la demostración de que el firmante es el operador del arcade y no un transeúnte cualquiera. Anchor ofrece dos formas de escribir eso desde hace mucho. La forma a la que recurre todo tutorial de v1 es `has_one`: guardas la clave del operador en una cuenta `Config` y escribes `has_one = authority` sobre ese config, que quiere decir "la cuenta llamada `authority` en este struct tiene que ser igual a `config.authority`". Funcionaba, pero era rígida — `has_one` solo puede comparar contra un *campo de clave guardado* con un *nombre de campo que calce*. La general es `address = <expression>`: la pones sobre la cuenta que quieres restringir y le das cualquier expresión que evalúe a un `Address`, y la macro verifica que la clave de la cuenta sea igual a esa expresión, en la fase de constraint hooks, con un `@ CustomError` opcional. Sé preciso con el linaje, porque es fácil equivocarse: el `address = <expr>` *no* es un invento de V2 — la línea v1 ya lo entrega, con expresión y error propio incluidos (0.30 lo extendió a expresiones de campo; 0.31 arregló la regresión que rompió brevemente las no-const).

Lo que V2 cambia de verdad es el veredicto entre las dos: retira `has_one`. El keyword estrecho que solo hacía clave-guardada-igual-a-cuenta-del-mismo-nombre queda deprecado a favor de la verificación general de expresiones que hace eso y todo lo demás, así que el caso especial se vuelve peso muerto. La barrera del operador, en la forma en la que V2 se estandariza, es una línea sobre el firmante:

```rust
#[account(address = config.authority @ VaultError::Unauthorized)]
pub authority: Signer,
```

Ese es todo el arreglo. Si la dirección del firmante no es `config.authority`, la macro levanta `VaultError::Unauthorized` antes de que corra `admin_set_credit`. Sin rama en el handler, sin `require!`, nada que olvidar. Y acá está el hilo de trayectoria de la lección pasada apareciendo otra vez, en espíritu: V2 no para de borrar casos especiales. La lección pasada fue el bump de seeds literales plegándose en una const de tiempo de macro; acá es el keyword estrecho cediéndole el lugar a la verificación general que lo subsume. El `has_one` solo hacía clave-guardada-igual-a-cuenta-del-mismo-nombre. El `address = expr` hace eso *y todo lo demás*, así que el estrecho se vuelve peso muerto.

Igual no desapareció, y la forma en que persiste es una linda pieza de arqueología de frameworks. El `has_one` todavía parsea en V2. Solo emite una advertencia de deprecación. El parser guarda el span de fuente del keyword justamente para que el codegen te lo pueda subrayar (lang-v2 `derive/src/parse.rs`, al 2026-08). Alguien se guardó la ubicación a propósito solo para dibujarte una rayita ondulada debajo. Así que las migraciones no se rompen, el código viejo compila, y el compilador te insiste con `address =` de a una advertencia por vez.

![has_one compara un campo de clave guardado contra una cuenta del mismo nombre y está deprecado, mientras que address = expr compara la clave de la cuenta contra cualquier expresión y es la forma en la que V2 se estandariza.](assets/v02-comparison.webp)

### owner: la trampa que se esconde a plena vista

Ahora el constraint que el catálogo más quiere que uses mal. Digamos que aceptas una cuenta cuyo dueño es un programa *distinto*, algún registro que tu vault lee pero que no es suyo, y quieres un error amable cuando alguien pasa la cosa equivocada. El código obvio se escribe solo:

```rust
// Reads correctly. Does NOT do what you think.
#[account(owner = REGISTRY_PROGRAM_ID @ VaultError::WrongOwner)]
pub registry: Account<Registry>,
```

Lo pruebas con una cuenta mala, esperando `WrongOwner`, y la prueba reporta un `IllegalOwner` genérico en su lugar. Tu error propio nunca se dispara. ¿Por qué?

Recorre primero las explicaciones ingenuas, porque descartarlas es lo que hace que la razón real se te quede. ¿Tal vez la sintaxis de error `@` solo funciona sobre `constraint =`? No, `@` liga errores en varios constraints, `address` y `owner` incluidos. ¿Tal vez la constante es una expresión que la macro no puede evaluar? No, evalúa bien. La razón es el modelo de fases de dos secciones atrás, y nada más. Sobre `Account<T>`, la verificación de dueño corre en **`load`**, la primera fase, y el dueño contra el que verifica es *tu programa*, hardcodeado, porque eso es lo que quiere decir un wrapper tipado. El `load` falla la cuenta antes de que exista ningún constraint hook que levante tu error, y te llega `IllegalOwner`.

Léelo un paso más allá de lo que lo lee el mensaje de error, porque ese es el hecho más importante. El `owner = <some other program>` sobre un `Account<T>` no está meramente tapado por otro error, es inalcanzable: el wrapper tipado ya fijó el dueño a tu propio programa, así que una cuenta cuyo dueño sea cualquier otro nunca puede cargar como `Account<Registry>`, de entrada. El constraint `owner =` solo tiene algo que decir sobre un wrapper que *no* fijó el dueño durante el load.

Así que el arreglo no es "mover el error", es "usar el wrapper que deja la pregunta abierta". Baja a una cuenta cruda y afirma la propiedad tú mismo en la fase de hooks:

```rust
/// CHECK: ownership is asserted explicitly below so a custom error can fire.
#[account(
    constraint = registry.owner() == &REGISTRY_PROGRAM_ID @ VaultError::WrongOwner
)]
pub registry: UncheckedAccount,
```

El `UncheckedAccount` se saltea el `load` tipado, así que nada fijó ningún dueño y no hay ninguna verificación temprana que se te adelante. El `constraint =` corre en la fase de hooks, evalúa tu booleano, y levanta `WrongOwner` si falla. Canjeaste la verificación automática de dueño del framework por una manual, a propósito, para comprarte tanto un error propio como la posibilidad siquiera de nombrar un dueño ajeno. (El registro de acá es ilustrativo; R2 todavía no lee ninguna cuenta ajena, y el caso real de dueño ajeno llega con los tokens en el módulo 5. La trampa es la lección.)

El tradeoff honesto: que `Account<T>` fije el dueño a tu programa durante `load` es una *feature* noventa y nueve veces de cada cien. Quiere decir que casi nunca escribes verificaciones de dueño a mano, y la que sí escribiste y olvidaste no es un bug porque el framework la hizo por ti. La trampa es solo el caso de borde en el que quieres un *mensaje propio* sobre esa verificación automática. No te pongas a reemplazar cada `Account<T>` por `UncheckedAccount` para tener errores bonitos. Estarías desactivando el cinturón de seguridad para cambiarle el color.

![Un constraint owner con un error propio sobre Account<T> corre durante el load y da IllegalOwner, mientras que la misma aserción sobre UncheckedAccount corre en la fase de hooks y muestra WrongOwner.](assets/v03-comparison.webp)

### close = destination: devolver el rent

Un jugador que ya no quiere un vault tendría que recuperar su dinero. Cuando creaste el vault la lección pasada, el jugador fondeó su **reserva exenta de renta**, el balance de lamports que toda cuenta tiene que mantener para que el runtime no la purgue. Esa reserva es un depósito y no una comisión, que es toda la razón por la que puede volver: se queda en la cuenta todo lo que la cuenta viva, y tendría que volver al jugador en el momento en que la cuenta muere.

El `close = destination` hace exactamente eso, y lo hace como constraint de macro así nunca armas a mano el movimiento de lamports:

```rust
#[account(
    mut,
    close = player,          // rent goes to `player`; the account is invalidated
    seeds = [b"vault", player.address().as_ref()],
    bump = vault.bump,
)]
pub vault: Account<Vault>,
```

Tres cosas pasan atómicamente cuando esta instrucción tiene éxito. El balance entero de lamports de la cuenta, reserva exenta de renta incluida, se mueve a `player`. Los datos de la cuenta quedan en cero y su discriminador queda borrado, así nunca puede revivirse en silencio y confundirse con un vault vivo. Y todo eso es visible en el IDL, así que un indexador o un cliente sabe que esta instrucción cierra una cuenta sin leer el cuerpo de tu handler. De hecho el cuerpo del handler puede estar vacío, porque el constraint carga con toda la operación él solo. Compara eso con la versión hecha a mano, donde debitarías lamports manualmente, pondrías los datos en cero, y esperarías no haber dejado un camino para revivirla, y la diferencia de verdad no es la concisión: la forma con constraint no puede olvidarse de un paso y la hecha a mano sí.

![Antes del close el vault guarda su reserva de rent; el constraint close mueve cada lamport al jugador, pone los datos en cero y borra el discriminador para que la cuenta no se pueda revivir.](assets/v04-diagram.webp)

### realloc_payer y realloc_zero: la misma idea, una forma nueva de escribirla

Si el vault alguna vez necesita crecer, digamos que le agregas un slab de puntajes altos recientes, realocas su space. V2 se quedó con realloc pero aplanó la sintaxis de sub-keywords, así que las formas de v1 anidadas con dos puntos, `realloc::payer` y `realloc::zero`, ya no están, y las formas de V2 son los identificadores planos y únicos `realloc_payer` y `realloc_zero`. Escribe la forma vieja con dos puntos y no compila:

```rust
#[account(
    mut,
    realloc = Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE + EXTRA_SLAB,
    realloc_payer = player,   // v1 was realloc::payer
    realloc_zero = false,     // v1 was realloc::zero; false = keep existing bytes on grow
)]
pub vault: Account<Vault>,
```

El `realloc_payer` nombra quién fondea el rent extra cuando la cuenta crece (crecer necesita más rent; encogerse lo devuelve). El `realloc_zero` decide si el buffer recién dimensionado queda borrado: `true` cuando estás encogiendo y quieres que los datos viejos de la cola desaparezcan, `false` cuando estás creciendo y quieres que los bytes existentes se preserven. Esa es toda la migración. Si estás portando un programa v1 y el compilador rechaza `realloc::payer`, este rename es la razón, y el arreglo es mecánico.

### init_if_needed, in situ: la puerta que se volvió a abrir

Ahora el keyword que marcó el resumen de esta lección. El `init_if_needed` deja que una instrucción diga "crea esta cuenta si no existe, y si no usa la que sí". Es genuinamente cómodo para un flujo de "recargar o abrir": el jugador corre una instrucción tenga o no tenga ya un vault.

En v1, este keyword estaba detrás de un feature gate y venía envuelto en advertencias, porque es la casa clásica del **ataque de reinicialización**: un atacante fuerza tu camino de crear-o-reusar por la rama de *reuso* sobre una cuenta que ya guarda estado vivo, y tu handler, creyendo que acaba de crear una cuenta nueva, resetea ese estado. Balance vivo, desaparecido. En V2 el keyword se entrega **sin ningún feature gate** y con un archivo de 604 líneas de pruebas que validan el reuso y lo respaldan (lang-v2 al 2026-08). Es un ablandamiento real, tanto frente a la postura de v1 con su feature gate como frente a la regla de la casa que antes decía "nunca". Alguien escribió un montón de pruebas para volver esto defendible por defecto.

Acá está la parte que no puedes malinterpretar. La validación de reuso de V2 vuelve a verificar la **estructura** de la cuenta cuando ya existe: el space está bien, el dueño es tu programa, el discriminador calza con `Vault`. Eso vale la pena tenerlo. Lo que *no* hace, lo que *no puede* hacer, es conocer tus invariantes. No tiene ni idea de que `credit` es un balance vivo que fondeó un jugador. Así que el ataque de reinicialización sobrevive, en exactamente una forma acotada: la validación de reuso resguarda la *forma*, y el *estado* te toca resguardarlo a ti.

![La validación de reuso cubre el space, el dueño y el discriminador en una cuenta init_if_needed existente pero no el estado de negocio vivo; la guarda consiste en ramificar según si el vault es nuevo antes de resetear cualquier campo.](assets/v05-diagram.webp)

La guarda es una sola rama. En una cuenta recién creada cada byte es cero, así que `owner == Address::default()` te dice que es nueva. Inicializa solo en ese caso, y solo *suma* al balance, nunca lo asignes:

```rust
pub fn top_up_or_open(ctx: &mut Context<TopUpOrOpen>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    if vault.owner == Address::default() {
        // Fresh: reuse-validation confirmed shape, but the fields are still zeroed.
        vault.owner = *ctx.accounts.player.address();
        vault.bump = ctx.bumps.vault;
        vault.credit = 0;
    }
    // Existing OR fresh: only ADD. Never `vault.credit = amount`, that is the clobber.
    vault.credit = vault.credit.checked_add(amount).ok_or(VaultError::Overflow)?;
    Ok(())
}
```

Una nota honesta de contabilidad, porque la lección pasada fue enfática en que los créditos son fichas y las fichas son lamports. Ni este handler ni `admin_set_credit` mueven un solo lamport. El `credit` es un contador que flota libre durante toda esta lección, a propósito: mover valor quiere decir una transferencia firmada por PDA, y el módulo 4 se dedica enteramente a eso. Así que el vault de hoy tiene libros honestos y nada de dinero en ellos. El módulo 4 es donde el número empieza a estar respaldado por el balance de la cuenta, y donde un `credit` sin respaldo se vuelve un bug y no una simplificación.

Ese `if` es toda la defensa, y es la disciplina para la que el framework nunca te va a dar un keyword. El trade-off de todo el catálogo aterriza justo acá. Los constraints mueven la validación adentro de la macro, donde no se puede olvidar y donde aparece en el IDL, que es una victoria real y la mayor parte de esta lección. Pero dos filos siguen afilados: la trampa del error de owner quiere decir que un mensaje de owner propio necesita `UncheckedAccount`, y el `init_if_needed` canjea una rama de conveniencia por una superficie de reinicialización que ahora es tuya. Ergonomía de un lado, del otro una disciplina de verificación explícita que no puedes tercerizar a los keywords. Los keywords hacen mucho. Ese `if` no lo hacen.

## Lab: ponle un portero al vault

Estás extendiendo el `quarter_vault` de la lección pasada hacia un programa resguardado por constraints. Gana tres cosas en este Lab: una cuenta `Config` que guarda la clave del operador del arcade, un `admin_set_credit` con barrera de autoridad, y un `close_vault` que devuelve el rent. El camino `top_up_or_open` de la sección de teoría a propósito no está acá; es el Challenge Solo, y construirlo en frío es el punto. Dos pruebas de LiteSVM superan el criterio: una llamada de admin con la autoridad equivocada queda rechazada *por el constraint*, y `close_vault` devuelve el rent e invalida la cuenta.

**1. Agrega el estado Config y su init.** La clave del operador necesita una casa. Pon un PDA `Config` en un seed fijo, de instancia única. En `programs/quarter-vault/src/lib.rs`:

```rust
#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Address, // 32: the arcade operator
    pub bump: u8,           // 1: canonical bump, stored per last lesson's discipline
}

#[derive(Accounts)]
pub struct InitConfig {
    #[account(mut)]
    pub authority: Signer,
    #[account(
        init,
        payer = authority,
        space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<Config>,
    pub system_program: Program<System>,
}
```

Y el handler, que es el mismo patrón de guardar-el-bump que ya conoces:

```rust
pub fn init_config(ctx: &mut Context<InitConfig>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.authority = *ctx.accounts.authority.address();
    config.bump = ctx.bumps.config;
    Ok(())
}
```

Resultado esperado después de este paso: `anchor build` compila limpio y el programa ahora expone dos caminos de init, el `init_vault` de la lección pasada y `init_config`. Todavía no hay ninguna barrera puesta, que es el punto del paso siguiente.

**2. Endurece el camino de admin.** Reemplaza el struct `AdminSetCredit` ingenuo de la apertura por el resguardado. El cambio es una sola línea de constraint, y es todo el punto de la lección:

```rust
#[derive(Accounts)]
pub struct AdminSetCredit {
    // The gate: this signer MUST equal config.authority, enforced in the hook phase.
    #[account(address = config.authority @ VaultError::Unauthorized)]
    pub authority: Signer,

    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<Config>,

    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<Vault>,
}
```

El cuerpo del handler no cambia respecto del ingenuo. Ese es el mensaje en el que vale la pena detenerse: la seguridad se movió *fuera* del handler y *dentro* del struct derive, donde no se puede olvidar y donde el IDL la publica.

![El struct AdminSetCredit condiciona al firmante de autoridad con address = config.authority, vuelve a derivar el config de solo lectura desde su bump guardado, y vuelve a derivar el vault objetivo mutable de la misma forma.](assets/v06-annotated-code.webp)

Resultado esperado después de este paso: el build *no* compila todavía, y el error vale la pena leerlo y no temerle. El constraint nombra `VaultError::Unauthorized`, un enum que no existe hasta el paso 4, así que `anchor build` se detiene con un E0433 `failed to resolve` sobre `VaultError`. Déjalo en rojo hasta el paso 3; el paso 4 lo salda. Una vez que el enum aterriza, el IDL generado para `admin_set_credit` va a listar una cuenta `config` que no listaba un minuto atrás. Esa cuenta nueva en la interfaz *es* la barrera, visible para cualquiera que lea el IDL sin leer tu Rust.

**3. Agrega close_vault.** El handler está vacío. El constraint carga con el trabajo:

```rust
pub fn close_vault(_ctx: &mut Context<CloseVault>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct CloseVault {
    #[account(mut)]
    pub player: Signer,
    #[account(
        mut,
        close = player,   // rent-exempt reserve returns to the player; account invalidated
        seeds = [b"vault", player.address().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<Vault>,
}
```

La línea de `seeds` está haciendo control de acceso en silencio: solo el jugador cuya clave deriva este vault exacto puede pasar la verificación de seed, así que nadie puede cerrar un vault que no es suyo. No necesitaste un constraint de propiedad aparte, el esquema de seeds ya es uno.

**4. Agrega el enum de errores.** Anchor permite exactamente un enum `#[error_code]` por programa, así que todo vive acá:

```rust
#[error_code]
pub enum VaultError {
    #[msg("caller is not the configured arcade authority")]
    Unauthorized,
    #[msg("credit addition overflowed")]
    Overflow,
    #[msg("account is owned by the wrong program")]
    WrongOwner,
}
```

Resultado esperado después de este paso: `anchor build` compila limpio. Pero ojo con un segundo enum `#[error_code]` que haya quedado dando vueltas: compila verde, y los dos enums numeran sus variantes desde la misma base 6000, colisionando en silencio en tiempo de ejecución. Cada variante que el programa vaya a levantar alguna vez tiene que aterrizar en este.

**5. Demuestra la barrera con una prueba de autoridad equivocada.** Este es el criterio de evaluación: el rechazo tiene que venir del *constraint*, no de una rama del handler. Agrega esto al `tests/quarter_vault.rs` que escribiste la lección pasada, conservando la prueba que ese archivo ya tiene y descartando las líneas `use` duplicadas en vez de pegarlas dos veces:

```rust
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{
    Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction,
};

fn setup() -> (LiteSVM, Address) {
    let mut svm = anchor_v2_testing::svm();
    let program_id = quarter_vault::ID;
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    svm.add_program_from_file(program_id, vault_so).unwrap();
    (svm, program_id)
}

// New here, because this file now sends three transactions instead of one: fold the
// blockhash-fetch-and-sign into one helper rather than repeating it at every send.
fn tx(svm: &LiteSVM, payer: &Keypair, instruction: Instruction) -> VersionedTransaction {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap()
}

#[test]
fn wrong_authority_is_rejected_at_the_constraint() {
    let (mut svm, program_id) = setup();

    let operator = Keypair::new();   // the real authority
    let attacker = Keypair::new();   // a random signer
    let player = Keypair::new();
    for kp in [&operator, &attacker, &player] {
        svm.airdrop(&kp.pubkey(), 1_000_000_000).unwrap();
    }

    // init_config with the real operator
    let (config_pda, _) = Address::find_program_address(&[b"config"], &program_id);
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::InitConfig {
            authority: operator.pubkey(),
            config: config_pda,
            system_program: System::id(),
        }.to_account_metas(None),
        data: quarter_vault::instruction::InitConfig {}.data(),
    };
    let init_config = tx(&svm, &operator, ix);
    svm.send_transaction(init_config).unwrap();

    // init a player vault (uses last lesson's init_vault)
    let (vault_pda, _) =
        Address::find_program_address(&[b"vault", player.pubkey().as_ref()], &program_id);
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::InitVault {
            player: player.pubkey(),
            vault: vault_pda,
            system_program: System::id(),
        }.to_account_metas(None),
        data: quarter_vault::instruction::InitVault {}.data(),
    };
    let init_vault = tx(&svm, &player, ix);
    svm.send_transaction(init_vault).unwrap();

    // ATTACK: attacker signs admin_set_credit, presenting themselves as authority.
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::AdminSetCredit {
            authority: attacker.pubkey(),
            config: config_pda,
            vault: vault_pda,
        }.to_account_metas(None),
        data: quarter_vault::instruction::AdminSetCredit { new_credit: 9_999 }.data(),
    };
    let attack = tx(&svm, &attacker, ix);

    // The macro rejects it in the constraint-hook phase, before the handler runs.
    assert!(svm.send_transaction(attack).is_err(),
        "attacker must be rejected by address = config.authority");
}
```

**6. Demuestra que close devuelve el rent.** La segunda prueba muestra la reserva volviendo a casa y la cuenta yéndose:

```rust
#[test]
fn close_returns_rent_and_invalidates() {
    let (mut svm, program_id) = setup();
    let player = Keypair::new();
    svm.airdrop(&player.pubkey(), 1_000_000_000).unwrap();

    let (vault_pda, _) =
        Address::find_program_address(&[b"vault", player.pubkey().as_ref()], &program_id);

    // init_vault first
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::InitVault {
            player: player.pubkey(),
            vault: vault_pda,
            system_program: System::id(),
        }.to_account_metas(None),
        data: quarter_vault::instruction::InitVault {}.data(),
    };
    let init = tx(&svm, &player, ix);
    svm.send_transaction(init).unwrap();

    let before = svm.get_balance(&player.pubkey()).unwrap();

    // close_vault
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::CloseVault {
            player: player.pubkey(),
            vault: vault_pda,
        }.to_account_metas(None),
        data: quarter_vault::instruction::CloseVault {}.data(),
    };
    let close = tx(&svm, &player, ix);
    svm.send_transaction(close).unwrap();

    let after = svm.get_balance(&player.pubkey()).unwrap();
    assert!(after > before, "rent-exempt reserve should return to the player");
    assert!(svm.get_account(&vault_pda).is_none(), "closed account is invalidated");
}
```

**7. Córrelo.**

```bash
anchor test
```

Salida esperada, los dos criterios en verde:

```
running 2 tests
test wrong_authority_is_rejected_at_the_constraint ... ok
test close_returns_rent_and_invalidates ... ok

test result: ok. 2 passed; 0 failed
```

Si la prueba de autoridad equivocada *falla* (la transacción tiene éxito cuando no debería), la causa habitual es que dejaste el struct `Signer` ingenuo en su lugar y nunca agregaste `address = config.authority`. El constraint es toda la barrera. Sin él, `Signer` demuestra que alguien firmó, nunca quién.

## Challenge

Dos peldaños otra vez, y esta vez el segundo no tiene nada de código en la página.

**Completion.** Abre los structs `AdminSetCredit` y `CloseVault` que acabas de escribir y borra dos constraints: reemplaza la línea `address = ...` por `// TODO: gate the signer` y la línea `close = ...` por `// TODO: refund + invalidate`. Ahora vuelve a llenarlos de memoria. Aceptación: la prueba de autoridad equivocada rechaza en el constraint (no en el handler), y `close_returns_rent_and_invalidates` pasa. Si te encuentras agregando una verificación `if ctx.accounts.authority.address() != ...` *adentro* del handler, para. Ese es el hábito de v1 que el catálogo existe para borrar. La verificación va en el struct derive.

**Solo.** Construye el camino de `top_up_or_open` con `init_if_needed`, usando el handler resguardado de la sección de teoría, y después escribe la prueba que demuestra que tu rama de reuso no puede sobrescribir un balance vivo. La forma: inicializa un vault, recárgalo hasta algún crédito distinto de cero, después llama a `top_up_or_open` *otra vez* con un segundo monto y afirma que el crédito final es la *suma*, no el segundo monto solo. Esa única aserción es la demostración de que tu guarda `if vault.owner == Address::default()` aguantó y de que el riesgo de reinicialización no mordió. Aceptación: la llamada de reuso preserva el balance existente y le suma, una llamada nueva inicializa limpiamente, y ninguno de los dos caminos resetea `credit` de forma incondicional. Si tu prueba ve el balance igual a solo la última recarga, tu guarda falta o está invertida, y escribiste la vulnerabilidad exacta de la que la sección advirtió, que es algo genuinamente útil de haber visto fallar una vez, a propósito, en una prueba.

![Una línea de tiempo que arranca en el init_if_needed que v1 tenía detrás de un feature gate, pasa por la reescritura que incorporó la validación de reuso por diseño, y llega a V2 entregándolo sin feature gate mientras tus invariantes de estado de negocio siguen siendo tu propia guarda.](assets/v07-timeline.webp)

Cuando los dos peldaños pasan, siéntate con lo que se volvió el vault. Un firmante equivocado rebota contra la macro derive antes de que corra tu código. Un jugador recupera su rent con un handler vacío y sin ningún camino para revivirla. Existe una instrucción de crear-o-reusar que *no* deja que nadie sobrescriba un balance fondeado, porque escribiste el único `if` que ningún keyword va a escribir por ti. Cada una de esas garantías es ahora visible en el IDL, lo que quiere decir que la próxima persona que lea tu programa ve las reglas sin leer la lógica. Ese es el canje que ofrecía el catálogo, y te quedaste con el lado bueno: validación que no puedes olvidar, menos dos filos que ahora conoces por su nombre.

El catálogo que acabas de recorrer es el conjunto que entrega el *framework*. Pero te vas a topar con una regla para la que el framework no tiene keyword, algo como "el crédito de este vault nunca puede bajar de un piso mínimo", y no vas a querer desparramar ese `require!` por nueve handlers. La próxima lección escribes tu propio namespace de constraints, un `quarters::min_balance` que la macro aplica exactamente igual que `address` o `close`, implementando el trait `AccountConstraint` que el framework deja abierto precisamente para esto. El portero aprende una regla de la casa que inventaste tú. Mantenla estricta.
