# Escribe tu propio constraint: el trait AccountConstraint

La lección pasada endureciste el vault con los constraints que entrega el framework: `address` como la barrera de autoridad, la trampa de `owner` y su workaround con `UncheckedAccount`, `close` para el reembolso del rent, y el camino de reuso de `init_if_needed` con su riesgo de reinit que sobrevive. Cada uno de esos fue un keyword que eligió otra persona. Lo cual está bien, justo hasta que la regla que de verdad necesitas no está en la lista.

Acá está la regla que quiero en R2, el quarter-vault que construiste en este módulo. Un jugador no debería poder correr ciertas operaciones contra un vault cuyo `credit` guardado está por debajo de algún piso mínimo. Llámalo `quarters::min_balance = 100`: por debajo de 100, rechaza. Fíjate en *de qué* se trata la regla, porque el nombre te va a confundir si no. Lee el contador `credit` de la cuenta, no el saldo de lamports de la cuenta, y tal como quedó la lección pasada ese contador todavía no está respaldado por ningún lamport. El módulo 4 es donde los dos se vuelven el mismo número. El piso mínimo de hoy es un piso mínimo en los libros. Antes de V2 esa regla tenía dos casas y ninguna era buena. La mayoría de las veces era código de handler: un `if` arriba de la instrucción, más la esperanza de que toda otra instrucción que tocara el vault escribiera el mismo `if`, más el rezo de que nadie agregara un call site seis meses después y se olvidara. A veces era un `constraint = <expr>` inline, que al menos corría en la macro pero era un booleano anónimo que no podías nombrar, reusar ni leer de un IDL. De cualquier manera el invariante vivía en tu disciplina en vez de en el tipo.

Antes de que argumente por qué ese es un mal lugar para guardar un invariante, haz la cosa que vuelve concreto todo lo demás. Mete esto en un archivo scratch y córrelo:

```bash
rustc --version   # any stable rustc at or above Anchor V2's MSRV, Rust 1.89.0
```

```rust
// scratch.rs - the check hook, distilled to plain Rust.
// Deliberately NOT named AccountConstraint / MinBalanceConstraint: the real trait has a
// different shape (static methods, an associated Value, a program error), and
// this file exists only to get the comparison right before the wiring.
pub trait BalanceGate {
    fn check(&self, balance: u64) -> Result<(), String>;
}

pub struct MinBalanceRule {
    pub min: u64,
}

impl MinBalanceRule {
    // The condition, split out as a plain `const fn` this file owns. Not a hook:
    // the trait has one method, `check`. It sits here so the compiler can
    // evaluate it at build time, which the graded version leans on.
    const fn meets_floor(&self, _balance: u64) -> bool {
        // This is the starter: it never rejects anything. That is the bug.
        true
    }
}

impl BalanceGate for MinBalanceRule {
    fn check(&self, balance: u64) -> Result<(), String> {
        if self.meets_floor(balance) {
            Ok(())
        } else {
            Err(format!(
                "quarters::min_balance violated: {} < {}",
                balance, self.min
            ))
        }
    }
}

fn run_constraint(balance: u64, min: u64) -> bool {
    MinBalanceRule { min }.check(balance).is_ok()
}

fn main() {
    // A 50-lamport vault against a 100 floor should be REJECTED.
    println!("50 vs 100 -> {}", run_constraint(50, 100)); // prints true. wrong.
    println!("100 vs 100 -> {}", run_constraint(100, 100)); // prints true. correct, by accident.
}
```

```bash
rustc scratch.rs -o scratch && ./scratch
```

Vas a ver `50 vs 100 -> true`. Un vault que guarda la mitad del piso mínimo pasa sin problema, porque la condición a la que el hook delega no hace nada. Guárdate esa línea que falla en la cabeza. Para el final del Lab este mismo `check` es un constraint real y visible en el IDL sobre R2, que se dispara antes de que tu handler llegue a correr. El problema Completion te devuelve este cuerpo de hook para llenar; el problema Solo pide un segundo constraint sin ningún apoyo. En el Lab mismo recorro cada línea.

## Resumen

- Anchor V2 despacha cualquier constraint con namespace que no sea de token (`ns::key = value`) a través de un trait público, `AccountConstraint<A>`. Lo implementas para un tipo marcador y obtienes un keyword nuevo de `#[account(...)]`, sin ningún fork y sin ningún cambio en la macro de derive.
- El trait expone cuatro hooks de ciclo de vida: `init`, `check`, `update`, `exit`. Codegen llama a cada uno en una fase específica. Un piso mínimo de saldo es una barrera en tiempo de lectura sobre la cuenta cargada, así que va en `check`, no en `init` (solo creación) ni en `exit` (post-handler).
- ¿Por qué un trait y no más keywords integrados? Porque ningún autor de framework puede enumerar por adelantado los invariantes de cada programa. Un trait abierto traslada esa decisión a ti, para siempre.
- El costo es real y vale la pena nombrarlo de entrada: un hook `check` corre en cada instrucción que calza, dentro del presupuesto de cómputo que tiene esa instrucción. Un hook equivocado o un hook caro es CU que pagas en un camino caliente para siempre, y puede esconder un bug de lógica detrás de un verde "pasa los constraints."

El repliegue de la ayuda de hoy, y es el tercer giro de este loop en el módulo, así que corre un punto más lejos que el de la lección pasada: yo escribo el impl del trait contigo en el Lab, pero el vault, sus seeds, su bump guardado y su banco de pruebas de LiteSVM ahora te toca reproducirlos a ti sin narración. En el problema Completion vuelves a llenar el cuerpo de `check` de memoria. En el problema Solo recibes una especificación, ningún apoyo, y diseñas tú mismo un segundo namespace de constraint.

## Por qué la superficie de constraints es un trait que puedes implementar

### Una rampa de entrada de 30 segundos: traits, tipos marcadores, elementos asociados

No necesitas hablar Rust con fluidez para leer lo que sigue, solo reconocer tres formas.

Un **trait** es un conjunto con nombre de métodos que un tipo promete proveer. Si un tipo "implementa `AccountConstraint`", aporta cuerpos para los métodos de ese trait, y cualquier código escrito contra el trait ahora puede llamarlos. Esa es toda la idea: el código depende de la promesa, no del tipo concreto.

Un **tipo marcador** es una struct sin campos, `pub struct MinBalanceConstraint;`, que existe solo para que le implementen un trait encima. No lleva datos. Es un nombre del que puedes colgar comportamiento. Cuando ves `impl AccountConstraint<Account<Vault>> for MinBalanceConstraint`, léelo así: "esto es lo que hace la regla de min-balance cuando se aplica a un `Vault` cargado."

Un **elemento asociado** es un tipo o una constante que pertenece a una implementación de trait en vez de pasarse por parámetro. `AccountConstraint` tiene un tipo asociado `Value`, el tipo del valor a la derecha del `=`. Para `quarters::min_balance = 100`, `Value` es `u64` y el `100` es ese valor. Esa es genuinamente toda la maquinaria de Rust en la que se apoya esta lección. Trait, tipo marcador, tipo asociado. Todo lo demás es el argumento de por qué están acá.

![Una referencia de tres tarjetas que mapea el trait AccountConstraint, el tipo marcador sin campos MinBalanceConstraint y el tipo asociado Value a la única línea de impl que ata los tres juntos.](assets/v01-diagram.png)

### El statu quo, y el lugar exacto donde se rompe

Arranca por cómo funcionaban los constraints antes de V2, porque el límite es toda la motivación. Un constraint como `has_one` o `address` es un keyword que el autor del framework coció dentro de la macro de derive. La macro parsea tu atributo `#[account(...)]`, calza el keyword contra una lista fija que ya conoce, y emite la verificación correspondiente. La lista es cerrada. Es un diccionario que el autor del framework terminó de imprimir antes de conocer tu programa.

Para los constraints que están en esa lista, esto es excelente. `has_one`, `owner`, `seeds`: son casi universales, y tenerlos como keywords de primera clase y visibles en el IDL quiere decir que un cliente que lee tu IDL puede verlos, y que el compilador los aplica igual en cada instrucción. El único problema, siempre, es el constraint que *no* está en la lista. Y siempre hay uno, porque tu programa tiene invariantes que ningún autor de framework podría haber predicho: un piso mínimo de vault, una regla de dueño-es-un-rol-específico, una regla de "este contador nunca pasa de un tope". Reglas de dominio. En el momento en que necesitas una, la lista cerrada no tiene nada para ti, y recurres a código de handler.

Acá está la pregunta filosa que fuerza el diseño. Si `has_one` llega a ser un constraint declarativo, de cada instrucción, visible en el IDL, ¿por qué *tu* invariante tendría que ser un `if` escrito a mano arriba de un solo handler que cada otro call site tiene que acordarse de copiar?

### Descartando primero las respuestas fáciles

Los arreglos obvios fallan uno por uno, y verlos fallar es lo que construye la necesidad del arreglo de verdad.

La primera respuesta fácil: "escribe la verificación en el handler, y listo." Acá arranca todo el mundo, y tiene tres modos de falla específicos, no uno. No es visible en el IDL, así que un cliente no tiene manera de saber que la regla existe. No la aplica el tipo, así que una segunda instrucción que toca el mismo vault no la hereda. Y corre *después* de la carga de cuentas y a menudo después de otra lógica, así que un bug de orden puede dejar pasar una cuenta mala por una verificación parcial. La regla es real pero vive en tu memoria, y la memoria no sobrevive a un compañero de equipo nuevo que agrega un call site.

La segunda respuesta fácil: "entonces agrega mi keyword al framework." Haz un fork de Anchor, agrega `min_balance` al parser y al codegen, listo. Solo que ahora eres dueño de un fork de un framework, para siempre, y lo vuelves a mergear en cada release. Peor, tu regla es específica de *tu* vault; no tiene nada que hacer en el Anchor de todo el mundo. Un framework que aceptara el invariante privado de cada programa como keyword integrado se derrumbaría bajo una lista de keywords que nadie podría leer. Escala la idea y es absurdo: un keyword por programa no es un lenguaje, es un basurero.

La tercera respuesta, más sutil: "hazlo un `constraint = <expr>` genérico." V2 sí entrega `constraint = <expr>` para exactamente el caso de una sola vez, y para una condición desechable es la herramienta correcta. Pero es un booleano inline, no una cosa con nombre, reusable, legible en el IDL. No puedes aplicar `constraint = vault.credit >= 100` a una segunda cuenta y hacer que un lector vea "ah, esa es la regla de min-balance." No compone, no se nombra a sí mismo, y no aparece como un constraint distinto en la interfaz. Es un parche, no una primitiva.

Así que la pregunta de verdad se angosta a esto: ¿cómo dejas que un programa agregue un keyword de constraint *con nombre, reusable y visible en el IDL* sin tocar el framework y sin hacerle un fork? Una vez que la pregunta es así de precisa, el mecanismo es casi forzoso.

![Una matriz que compara los if de handler, hacerle un fork a Anchor, las expresiones de constraint inline e implementar AccountConstraint; solo el trait es a la vez visible en el IDL, aplicado por el tipo en cada call site, libre de fork y reusable.](assets/v02-comparison.png)

### El mecanismo: despacho a través de un trait

La respuesta de V2 es dejar de tratar la lista de constraints como una tabla cerrada de keywords y empezar a tratarla como un trait abierto. Cualquier constraint con namespace, cualquier cosa de la forma `ns::key = value` donde `ns` no sea un integrado como `token`, se despacha a través de `AccountConstraint<A>`. Un crate downstream, o sea tu programa o cualquier librería de la que dependas, implementa ese trait para un tipo marcador, y la macro de derive enruta el keyword nuevo hacia él. Nada en la macro central cambia. Esto tampoco es una escotilla especial atornillada para un solo caso. Es la misma filosofía que hace públicos los traits `AnchorAccount`, `Id` y `Discriminator` en V2: el framework está deliberadamente abierto en las costuras, así que los crates downstream pueden entregar wrappers de cuenta nuevos, program ids conocidos nuevos y esquemas de discriminador nuevos sin ningún fork (superficie de extensibilidad de lang-v2 y docs-v2, verificada en 2.0.0-rc.1). Una frontera para mantener clara: el namespace `token::*` es la excepción integrada, el único namespace que el derive central maneja él mismo por el bien de `anchor-spl`. Todo *otro* namespace, el tuyo incluido, se despacha a través del trait, y ese despacho abierto es la puerta por la que estás a punto de pasar.

Acá está el trait, verificado contra el fuente de docs-v2 en la release candidate 2.0.0-rc.1 (crates.io, publicada el 2026-08-12). Trata la forma exacta como un blanco en movimiento: esta es una release candidate, la superficie de extensibilidad es de alta confianza pero todavía se está asentando, así que vuelve a leerla contra el crate cuando construyas:

```rust
pub trait AccountConstraint<A> {
    type Value;
    fn init(_account: &mut A, _value: &Self::Value) -> Result<()> { Ok(()) }
    fn check(_account: &A, _value: &Self::Value) -> Result<()> { Ok(()) }
    fn update(_account: &mut A, _value: &Self::Value) -> Result<()> { Ok(()) }
    fn exit(_account: &mut A, _value: &Self::Value) -> Result<()> { Ok(()) }
}
```

`A` es el tipo de cuenta *cargada* al que se aplica el constraint — el wrapper, `Account<Vault>` para nosotros, no el `Vault` desnudo, porque lo que codegen tiene en la mano cuando un hook se dispara es el wrapper cargado, y el wrapper hace deref a tu struct, así que el cuerpo se lee igual de cualquier manera. `Value` es el tipo del lado derecho, `u64` para un piso mínimo de lamports. Fíjate que cada hook entrega un cuerpo por defecto de `Ok(())`: un impl sobrescribe solo las fases que le importan, y las que dejas sin escribir son no-ops por construcción. Y los cuatro métodos son las cuatro fases de la vida de un constraint. Esa última parte es la pieza que tienes que sacar bien, así que se gana su propia sección.

### Los cuatro hooks, y en cuál vive un piso mínimo

Codegen no llama a los cuatro métodos cada vez. Llama al que calza con cómo se escribió el constraint. Esta es la tabla de enrutamiento, y es el hecho estructural de la lección:

![Un mapa de enrutamiento que muestra que un ns::key desnudo enruta a check, que las formas con prefijo init enrutan a init, que init_if_needed se bifurca a init y después a check, que update(...) enruta a update, y que exit se dispara en cualquier camino exitoso.](assets/v03-diagram.png)

Ahora razona en voz alta dónde va un piso mínimo de saldo, porque el razonamiento es el punto y es lo que la evaluación te pide defender.

`init` corre una vez, después de que la cuenta se crea. Si pones el piso mínimo ahí, lo aplicas exactamente en la creación y nunca más. Un vault creado por encima del piso mínimo podría caer por debajo en la instrucción siguiente y nada lo atraparía. Un piso mínimo que solo vale al nacer no es un piso mínimo. Descártalo.

`exit` corre al final, durante la serialización de la cuenta, solo para las cuentas que fueron mutadas y se escriben de vuelta. Poner una validación ahí es tentador porque "verifica el estado final" suena seguro. Pero `exit` corre *después* de la lógica de tu handler, así que es una aserción post-hoc, no una barrera. Además no corre para nada en una cuenta de solo lectura que nunca serializa, que es precisamente la cuenta que le importa a un piso mínimo en tiempo de lectura. Si tu objetivo es rechazar una cuenta mala de entrada, antes de que pase cualquier trabajo, `exit` es la fase equivocada. Descártalo también.

`update` solo se dispara dentro de una cláusula `update(...)` explícita. Es para el caso en que el constraint mismo muta la cuenta durante un flujo de update. Un piso mínimo es una validación, no una mutación. Este tampoco.

Eso deja `check`, y `check` es exactamente lo correcto, no por eliminación sino por ajuste. `check` corre sobre la cuenta cargada, en cada instrucción que calza, antes del handler. Esa es la definición de una barrera en tiempo de lectura: el vault ya tiene que satisfacer el invariante para que la instrucción siga, y se vuelve a verificar en todas y cada una de las llamadas, sin cachear, sin ser una sola vez. Un piso mínimo de saldo es una barrera en tiempo de lectura. Así que vive en `check`. Cuando la evaluación pregunta cuál hook y por qué, esta es toda la respuesta: `check`, porque un piso mínimo es un invariante sobre la cuenta cargada que tiene que valer antes de que corra el handler y en cada llamada.

![Un árbol de decisión que enruta una regla a update cuando muta, a check cuando tiene que valer en cada carga, a init solo en la creación, o a exit para aserciones post-handler.](assets/v04-flowchart.png)

### El tradeoff, dicho sin rodeos

Un trait de constraint abierto te compra algo real. Tu invariante ahora vive en la macro, es visible en el IDL, y no puede olvidarse en un call site, porque está pegado al campo de la cuenta mismo, no a las líneas de apertura de un handler. Agrega una instrucción nueva que cargue el vault con el mismo constraint, y el piso mínimo viene incluido gratis. Esa es la reutilización sin disciplina que unos `if` de handler nunca te pueden dar.

Pero ahora eres el autor de código que corre, en cada instrucción que calza, dentro de su presupuesto de cómputo. Esto no es gratis, y pretender lo contrario es así como entregas un programa lento. Comparado con un `if` de handler que escribiste una vez, un hook `check` corre la misma comparación, así que el costo de una verificación *barata* es un empate. El peligro es una verificación que no es barata. Si un compañero de equipo escribe un `check` que vuelve a leer y a hashear un pedazo grande de datos en cada llamada, eso es cómputo que ahora pagas en cada instrucción que calza, en un camino caliente, para siempre. Y hay un segundo costo, más silencioso: porque la cuenta entonces se lee como "válida", una verificación pesada o sutilmente equivocada puede esconder un bug de lógica detrás de un verde "pasa los constraints." La regla práctica es corta. Mantén los hooks baratos, mantenlos en la fase correcta, y nunca dejes que un constraint haga trabajo que le corresponde al handler. La extensibilidad te pasa código pegado al framework para que te hagas cargo; hazte cargo con cuidado.

![Una comparación fila por fila que muestra que el hook check gana en reutilización y en visibilidad en el IDL, empata en el costo de una comparación barata, y pierde fuerte cuando el hook es caro o sutilmente equivocado.](assets/v05-comparison.png)

Hay un poco de linaje que vale la pena llevarse al Lab, porque explica por qué esta puerta existe siquiera. La presión no fue académica. Vino de los que construyen. En la discusión #3742, ChewingGlass planteó sin rodeos el problema de ergonomía del framework, "Boilerplate kills new devs because they don't know the sacred incantations," y, en un sub-hilo de Codama de esa misma discusión, "But borsh is kind of terrible." El issue de diseño #4390 lleva la misma presión con sus propias palabras, que "the default serialization should probably behave more like zero-copy but with better UX." Ese es el argumento de la comunidad, comprimido, que empujó a V2 hacia una superficie de constraints que puedes extender en vez de una lista de keywords que solo puedes aceptar. El trait abierto es cómo se ve "menos boilerplate" una vez que deja de ser un reclamo y se vuelve una API.

![Una línea de tiempo desde la lista cerrada de keywords de v1, pasando por la presión de la comunidad para recortar boilerplate, hasta la reescritura no_std de V2 que hizo que los constraints se despacharan a través de traits públicos e implementables.](assets/v06-timeline.png)

## Lab: entrega `quarters::min_balance` en R2

Estás extendiendo R2, el programa `quarter_vault`, con un constraint propio. Cuando termines, `#[account(quarters::min_balance = 100)]` es un keyword de verdad en el campo del vault, su hook `check` rechaza un vault subfinanciado en tiempo de constraint, y una prueba de LiteSVM demuestra tanto el rechazo como el pase. La barra de `verify` para este artefacto es una sola cosa: `anchor test` está verde, un vault bajo el piso mínimo es rechazado por la capa de constraints, y un vault en el piso mínimo o por encima pasa.

**1. Confirma el toolchain de V2.** La misma verificación que el arranque de m03-l1, una línea. Si la versión está mal, el comando para volver a fijar está debajo; no construyas contenido de V2 sobre un binario `anchor` de V1:

```bash
anchor --version         # must report a 2.0.0 RC line, not 1.x; rc.1 as of 2026-08-12
# Only if it does not, re-pin (still no release binary for the v2 tag, so build from
# the tag: it is a fixed point, unlike the anchor-next branch tip it sits on):
cargo install --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1 anchor-cli --locked --force
# macOS, if the build trips on LTO: prefix that line with CARGO_PROFILE_RELEASE_LTO=off
```

**2. Demuestra la lógica en aislamiento primero.** Antes de tocar la macro, deja correcta la lógica de `check` como Rust puro, exactamente el archivo scratch del arranque de la lección. Esta es la misma destilación que califica el coding challenge, y vale la pena pasarla antes de cablearla en el framework, porque un constraint que falla y que no puedes aislar es miserable de depurar. Llena `meets_floor` para que rechace por debajo del piso mínimo y acepte en él o por encima. El piso mínimo es inclusivo: un saldo igual al mínimo pasa. El peldaño calificado entrega este mismo archivo con la misma división y le agrega debajo un bloque de aserciones `const` — como `meets_floor` es un `const fn`, esas corren en el compilador, así que una regla equivocada hace fallar el build con un mensaje que nombra el caso que sacó mal.

```rust
// scratch.rs - now with the condition filled in
pub trait BalanceGate {
    fn check(&self, balance: u64) -> Result<(), String>;
}

pub struct MinBalanceRule {
    pub min: u64,
}

impl MinBalanceRule {
    const fn meets_floor(&self, balance: u64) -> bool {
        balance >= self.min
    }
}

impl BalanceGate for MinBalanceRule {
    fn check(&self, balance: u64) -> Result<(), String> {
        if self.meets_floor(balance) {
            Ok(())
        } else {
            Err(format!(
                "quarters::min_balance violated: {} < {}",
                balance, self.min
            ))
        }
    }
}

fn run_constraint(balance: u64, min: u64) -> bool {
    MinBalanceRule { min }.check(balance).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_is_inclusive_and_rejects_below() {
        assert!(run_constraint(500, 100)); // above passes
        assert!(run_constraint(100, 100)); // exactly at the floor passes
        assert!(!run_constraint(50, 100)); // below is rejected
        assert!(!run_constraint(0, 1)); // an empty vault fails a 1-lamport floor
    }
}
```

```bash
rustc --test scratch.rs -o scratch_test && ./scratch_test
```

Esperado: `test result: ok. 1 passed; 0 failed`. Con eso la lógica queda cerrada. Todo lo que sigue es cablearlo en V2 para que se dispare desde la macro en vez de desde una función que te acordaste de llamar.

**3. Escribe el tipo marcador y su impl de `AccountConstraint`.** Abre el `lib.rs` de R2. Acuérdate del vault de antes en el módulo: una cuenta Pod, porque el `Account<T>` de V2 exige `T: Pod`, que guarda el dueño, un saldo de `credit` en lamports nativos por ahora, y el bump canónico.

```rust
use anchor_lang::prelude::*;

// Already in your file from earlier in the module, reproduced here for context;
// do not re-paste them. The id stays the one anchor init generated for you, and
// `_pad` is the explicit tail padding that keeps Vault Pod.
declare_id!("<your generated program id>");

#[account]
#[repr(C)]
#[derive(InitSpace)]
pub struct Vault {
    pub owner: Address,  // 32 bytes: the player who owns this vault
    pub credit: u64,     //  8 bytes: prepaid credit, native lamports for now
    pub bump: u8,        //  1 byte: the canonical bump, persisted for reuse
    pub _pad: [u8; 7],   //  7 bytes: explicit tail padding, zeroed, never read
}
```

Ahora el constraint, y primero la única regla mecánica que hace que todo esto funcione, porque nada más en esta lección te la va a decir y no puedes escribir el problema Solo sin ella. La macro resuelve `ns::key = value` por path, no por registro. Toma el namespace `ns` tal cual, como un path de módulo que tiene que estar en scope, y convierte la `key` en snake_case en un nombre de tipo en PascalCase dentro de él, y después le agrega un sufijo `Constraint` obligatorio. Así que `quarters::min_balance = 100` compila a una llamada sobre `quarters::MinBalanceConstraint`, y `quarters::max_balance` resolvería a `quarters::MaxBalanceConstraint`. Tres consecuencias que vale la pena retener: el módulo tiene que ser alcanzable desde donde está escrito el struct del derive, el nombre del tipo marcador no es una etiqueta que elijas libremente (es la `key` en PascalCase más el sufijo `Constraint`, o nada), y el `100` de la derecha tiene que chequear de tipo como el `Self::Value` de ese impl, que es por lo que el tipo asociado es `u64` y no algo que te toque inferir.

Con esa regla enunciada: el tipo marcador `MinBalanceConstraint` vive en un módulo `quarters`, e implementar `AccountConstraint<Account<Vault>>` para él es lo que le da a `quarters::min_balance` un cuerpo que llamar. El impl apunta al wrapper cargado, `Account<Vault>`, no al `Vault` desnudo, porque el wrapper es lo que codegen tiene en la mano cuando el hook se dispara — y como el wrapper hace deref a tu struct, `vault.credit` se lee exactamente como se leería en el tipo desnudo. El cuerpo de `check` es la lógica que acabas de demostrar, traducida al trait de verdad: `check` toma `&Account<Vault>` y `&Self::Value`, y rechaza con un error de programa en vez de con un `String`. Los otros tres hooks se quedan sin escribir para un piso mínimo puro en tiempo de lectura: el trait pone por defecto cada hook en `Ok(())`, así que dejar `init`, `update` y `exit` afuera *es* la declaración de que esta regla no hace nada en esas fases:

```rust
pub mod quarters {
    use super::*;

    /// Marker type. Implementing AccountConstraint<Account<Vault>> for it makes
    /// `#[account(quarters::min_balance = N)]` a real, IDL-visible constraint.
    pub struct MinBalanceConstraint;

    impl AccountConstraint<Account<Vault>> for MinBalanceConstraint {
        type Value = u64;

        fn check(vault: &Account<Vault>, floor: &u64) -> Result<()> {
            // The read-time gate: the loaded vault must already hold the floor.
            require_gte!(vault.credit, *floor, VaultError::BelowFloor);
            Ok(())
        }
    }
}

// You already have this enum from last lesson. Anchor allows exactly one
// #[error_code] per program, so do not add a second: just add the BelowFloor
// variant to the one that is already there.
#[error_code]
pub enum VaultError {
    #[msg("caller is not the configured arcade authority")]
    Unauthorized,
    #[msg("credit addition overflowed")]
    Overflow,
    #[msg("account is owned by the wrong program")]
    WrongOwner,
    #[msg("vault credit is below the quarters::min_balance floor")]
    BelowFloor,
}
```

`require_gte!(a, b, err)` es la macro de V2 que dice "a debe ser mayor o igual que b, si no devuelve err". Es la manera nativa del framework de escribir el piso mínimo inclusivo; echar mano de un `if` crudo con un `return Err(...)` manual también compilaría, pero la macro es el estilo de la casa y mantiene uniforme el camino de error.

![La función check lee el vault cargado en solo lectura, dereferencia el piso mínimo u64, y usa require_gte para rechazar cualquier crédito por debajo de ese piso mínimo inclusivo.](assets/v07-annotated-code.png)

Esperado después de este paso: `anchor build` compila el impl aunque todavía nada use el constraint. Un error de compilación que nombre `AccountConstraint` acá quiere decir que la forma del trait se movió bajo la RC, así que vuelve a leerla contra el crate antes de seguir.

**4. Aplica el constraint a una instrucción con guarda.** Agrega dos handlers. `set_credit` es un fixture que fija directo el `credit` guardado del vault, haciendo de suplente del camino real de depósito que llega en el módulo 4, cuando el vault empieza a mover lamports de verdad. Sé honesto contigo mismo sobre lo que es: un setter sin barrera, una lección después de toda una lección sobre condicionar las escrituras. Se entrega a propósito sin un `address = config.authority`, así la prueba puede llevar el vault a cualquier crédito en una línea, y es exactamente el handler que borrarías antes de entregar. El constraint es el artefacto; el fixture es herramienta de taller, y la tabla de disposición de m05-l1 es donde lo borras de verdad. `require_funded` es la operación con guarda: su cuerpo está vacío a propósito, así que cualquier pase o falla solo puede venir de la capa de constraints, nunca de la lógica del handler. La guarda es la única línea nueva, `quarters::min_balance = 100`, en el campo del vault:

```rust
#[program]
pub mod quarter_vault {
    use super::*;

    /// Fixture: set stored credit directly. Real lamport movement is module 4.
    pub fn set_credit(ctx: &mut Context<SetCredit>, amount: u64) -> Result<()> {
        ctx.accounts.vault.credit = amount;
        Ok(())
    }

    /// Guarded no-op: reaching this body at all proves the floor was satisfied.
    pub fn require_funded(_ctx: &mut Context<RequireFunded>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetCredit {
    pub player: Signer,
    #[account(
        mut,
        seeds = [b"vault", player.address().as_ref()],
        bump = vault.bump, // reuse the stored canonical bump, no runtime search
    )]
    pub vault: Account<Vault>,
}

#[derive(Accounts)]
pub struct RequireFunded {
    pub player: Signer,
    #[account(
        seeds = [b"vault", player.address().as_ref()],
        bump = vault.bump,
        quarters::min_balance = 100, // the custom constraint: check hook fires here
    )]
    pub vault: Account<Vault>,
}
```

Mira dónde queda el constraint. Está en el campo del vault en `RequireFunded`, justo al lado de `seeds` y `bump`, que son keywords integrados, y se lee exactamente como ellos. Esa es la recompensa: `quarters::min_balance` ahora es un constraint de primera clase, indistinguible en uso de los que entregó el framework. El vault acá ni siquiera es `mut`, que es la señal honesta de que esta es una barrera de solo lectura y por lo tanto `exit` nunca corre para él, solo `check`.

Esperado después de este paso: `anchor build` está limpio y el IDL de `require_funded` lleva el constraint en su cuenta `vault`. Esa línea del IDL es la diferencia entre una regla que el framework aplica y una regla que te acordaste de escribir.

**5. Escribe la prueba de LiteSVM.** LiteSVM corre el programa en proceso sin ningún validador, así que el loop es rápido. Agrega las dev-dependencies:

```toml
# Same one row as last lesson, and for the same reason: the harness owns the SVM
# version so you cannot drift off it. At tag v2.0.0-rc.1 anchor-v2-testing carries
# litesvm 0.11.0. Naming litesvm yourself is how you end up with two of them.
[dev-dependencies]
anchor-v2-testing = { git = "https://github.com/otter-sec/anchor.git", tag = "v2.0.0-rc.1" }
```

La prueba, en `tests/min_balance.rs`, hace cuatro cosas: inicializa el vault, pone su crédito por debajo del piso mínimo y demuestra que `require_funded` es rechazado, y después lo pone en el piso mínimo y demuestra que `require_funded` pasa. El rechazo y el pase son todo el artefacto:

```rust
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{
    Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction,
};

fn require_funded_tx(
    svm: &LiteSVM,
    player: &Keypair,
    program_id: Address,
    vault_pda: Address,
) -> VersionedTransaction {
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::RequireFunded {
            player: player.pubkey(),
            vault: vault_pda,
        }
        .to_account_metas(None),
        data: quarter_vault::instruction::RequireFunded {}.data(),
    };
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&player.pubkey()), &blockhash);
    VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[player]).unwrap()
}

#[test]
fn min_balance_rejects_below_and_passes_at_floor() {
    let mut svm = anchor_v2_testing::svm();
    let program_id = quarter_vault::ID;
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    svm.add_program_from_file(program_id, vault_so).unwrap();

    let player = Keypair::new();
    svm.airdrop(&player.pubkey(), 1_000_000_000).unwrap();
    let (vault_pda, _bump) =
        Address::find_program_address(&[b"vault", player.pubkey().as_ref()], &program_id);

    // Init the vault (init_vault was built in the earlier lesson; credit starts at 0).
    let init_ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::InitVault {
            player: player.pubkey(),
            vault: vault_pda,
            system_program: System::id(),
        }
        .to_account_metas(None),
        data: quarter_vault::instruction::InitVault {}.data(),
    };
    let blockhash = svm.latest_blockhash();
    let init_msg = Message::new_with_blockhash(&[init_ix], Some(&player.pubkey()), &blockhash);
    let init_tx =
        VersionedTransaction::try_new(VersionedMessage::Legacy(init_msg), &[&player]).unwrap();
    svm.send_transaction(init_tx).unwrap();

    // Helper to set stored credit.
    let set_credit = |svm: &LiteSVM, amount: u64| {
        let ix = Instruction {
            program_id,
            accounts: quarter_vault::accounts::SetCredit {
                player: player.pubkey(),
                vault: vault_pda,
            }
            .to_account_metas(None),
            data: quarter_vault::instruction::SetCredit { amount }.data(),
        };
        let blockhash = svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&player.pubkey()), &blockhash);
        VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&player]).unwrap()
    };

    // Below the floor: the constraint layer must REJECT require_funded.
    svm.send_transaction(set_credit(&svm, 50)).unwrap();
    let below = svm.send_transaction(require_funded_tx(&svm, &player, program_id, vault_pda));
    assert!(below.is_err(), "under-floor vault must be rejected by the constraint");

    // At the floor: the constraint layer must PASS require_funded.
    svm.send_transaction(set_credit(&svm, 100)).unwrap();
    // LiteSVM never advances its blockhash on its own, so a second require_funded
    // transaction built right now would be byte-identical to the rejected one above --
    // same signature -- and the SVM would refuse it as a duplicate (AlreadyProcessed)
    // instead of re-running it. Expire the blockhash so the retry is genuinely new.
    svm.expire_blockhash();
    let at_floor = svm.send_transaction(require_funded_tx(&svm, &player, program_id, vault_pda));
    assert!(at_floor.is_ok(), "at-or-above-floor vault must pass the constraint");
}
```

**6. Construye y corre.**

```bash
anchor test
```

Salida esperada, la única prueba que pasa y que cumple la barra:

```
running 1 test
test min_balance_rejects_below_and_passes_at_floor ... ok

test result: ok. 1 passed; 0 failed
```

La demostración está en el handler vacío. `require_funded` no hace nada, así que el `is_err()` del vault con 50 de crédito y el `is_ok()` del vault con 100 de crédito solo pueden ser el hook `check` hablando. El invariante se disparó en tiempo de constraint, antes de que corriera tu código, que es exactamente donde argumentaste que debía. Si en cambio ves que la llamada por debajo del piso mínimo *pasa*, la causa habitual es que el constraint lee el campo equivocado o que se coló de vuelta un `>` donde querías `>=`; vuelve a correr la prueba scratch del paso 2 para aislar la lógica del cableado.

## Challenge

Dos peldaños. El primero te da el trait y te quita el cuerpo; el segundo no te da nada.

**Completion.** Vuelve a abrir el hook `check` en tu impl de `MinBalanceConstraint` y borra el cuerpo, dejando `fn check(vault: &Account<Vault>, floor: &u64) -> Result<()> { /* TODO */ }`. Vuelve a llenarlo de memoria para que el piso mínimo sea inclusivo: un vault cuyo crédito iguala al piso mínimo pasa, uno por debajo devuelve `VaultError::BelowFloor`. Dos corridas lo aceptan: el peldaño calificado, que es el archivo scratch del paso 2 con las aserciones de tiempo de compilación pegadas debajo, y el `anchor test` del paso 6. Si echas mano de `init` o `exit` para hacer esto, detente: una barrera en tiempo de lectura vive en `check`, y ponerla en cualquier otra parte o se pierde instrucciones posteriores o se dispara en la fase equivocada.

**Solo.** Agrega un *segundo* constraint con namespace y demuestra que compone con el primero en una sola cuenta. Elige uno: `quarters::max_balance = N`, que rechaza un vault cuyo crédito está *por encima* de un techo, o `quarters::owner_is = <expr>`, que rechaza un vault cuyo dueño guardado no es una dirección dada. Impleméntalo como su propio tipo marcador con su propio impl de `AccountConstraint<Account<Vault>>`, eligiendo el hook correcto (los dos son barreras en tiempo de lectura, así que los dos son `check`, y razonar por qué es la mitad del ejercicio). Después aplica *los dos* en un solo campo, `#[account(quarters::min_balance = 100, quarters::max_balance = 10_000)]`, y escribe una prueba de LiteSVM que demuestre que un vault dentro de la banda pasa mientras que uno a cualquiera de los dos lados es rechazado. Aceptación: el segundo constraint se dispara desde la macro de derive sin ningún cambio en el framework, los dos constraints componen en una sola cuenta, y el coding challenge de Rust puro sigue pasando. Una cosa que vale la pena mirar: un vault que falla el primer constraint nunca debería llegar al segundo, porque los hooks `check` hacen cortocircuito en el primer error, igual que cualquier resultado propagado con `?`.

Ahora tienes toda la historia del constraint: puedes derivar PDAs, manejar el catálogo integrado completo y, desde hoy, extender ese catálogo con keywords que los autores del framework nunca escribieron. El vault está validado tan estrictamente como puedas describirlo. El vault puede guardar crédito y resguardar crédito. Lo que nunca ha hecho es pagarle de vuelta a nadie. A continuación, el módulo 4 lo pone a trabajar: firmando un withdraw real de lamports *como* la PDA, por el modelo de borrow de `CpiHandle` que trae V2, donde el compilador, no una llamada a `.reload()` que te acordaste de hacer, es lo que te mantiene a salvo. Los libros dejan de ser libros.
