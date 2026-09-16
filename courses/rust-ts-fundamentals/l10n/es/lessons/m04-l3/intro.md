# Los enums son máquinas de estados

## Resumen

m04-l2 convirtió cada desenlace de sondeo en un Result: un motor tipado con thiserror, un binario con anyhow, tu primer closure en map_err, y aritmética de lamports verificada. La corrida ahora sobrevive fixtures sucios. Lo que no sobrevive es el tiempo. Un target no es solo "ok en este instante"; tiene una historia, y nada en la estación la sigue: la flota de TS clasifica cada sonda en aislamiento (los veredictos de m02-l1, up, degraded, down) y se olvida. Hoy el motor de Rust escribe la mitad faltante del canon: la vida de un target como cuatro estados, Pending, Up, Degraded, Down, un enum cuyo match ES la tabla de transiciones, y las transiciones ilegales dejan de ser bugs que atrapas y se vuelven programas que no existen. Después aterriza el otro hilo del módulo: cargo test, clippy y fmt se suman al pipeline de Actions como barrera #3, y los dos lenguajes terminan con barrera sobre un solo workflow. La pregunta que guía toda la lección: ¿cómo cambias código al que le tienes miedo?

## Rompe la máquina primero

Empieza haciendo que te rechacen, a propósito, en menos de tres minutos. Abre `pulse-rs`, crea un archivo borrador `src/bin/scratch.rs`, y pega exactamente esto, brazo faltante y todo:

```rust
#[derive(Debug, Clone, Copy)]
enum ProbeState {
    Pending,
    Up,
    Down,
}

fn next_state(state: ProbeState, probe_ok: bool) -> ProbeState {
    use ProbeState::*;
    match (state, probe_ok) {
        (Pending, true) => Up,
        (Pending, false) => Down,
        (Up, true) => Up,
        (Up, false) => Down,
        (Down, false) => Down,
    }
}

fn main() {
    println!("{:?}", next_state(ProbeState::Pending, true));
}
```

```bash
cargo check
```

El compilador responde con el caso faltante, por su nombre:

```text
error[E0004]: non-exhaustive patterns: `(ProbeState::Down, true)` not covered
  --> src/bin/scratch.rs:10:11
   |
   |     match (state, probe_ok) {
   |           ^^^^^^^^^^^^^^^^^ pattern `(ProbeState::Down, true)` not covered
```

Lee ese error otra vez, despacio, porque es la lección. No escribiste una prueba para el camino de recuperación. No te acordaste del camino de recuperación. Lo olvidaste, como todo el mundo olvida un caso, y el compilador imprimió el caso olvidado por su nombre y se negó a compilar hasta que decidas qué significa el sondeo exitoso de un target Down. En m02-l1 compraste esta garantía exacta para la flota de TS con el truco de `assertNever`: una función ingeniosa que tenías que conocer, cablear y recordar en cada switch. Acá es el comportamiento por defecto de `match`. Nadie opta por entrar. La pluma está en la mano del compilador.

Agrega el brazo `(Down, true) => Up,` y `cargo check` se queda callado. Borra el archivo borrador cuando termines de romper cosas; la máquina de verdad va en el motor.

## La pluma está en la mano del compilador

### Estados como datos, transiciones como brazos

La vida de un target necesita cuatro estados, así que el motor recibe un enum de cuatro variantes. Esto va en `src/engine.rs`, al lado del trabajo de ProbeError de la lección pasada:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeState {
    Pending,
    Up,
    Degraded,
    Down,
}
```

El atributo `derive` es generación de código que te sale gratis: `Debug` te da impresión con `{:?}`, `Clone` y `Copy` hacen que el tipo sea barato de pasar por valor (es un byte, copiarlo sale más barato que pensar en pedirlo prestado), y `PartialEq`/`Eq` dejan que `assert_eq!` compare estados en las pruebas que estás por escribir. Vienes usando derive desde m04-l1 sin ceremonia. Esa es la cantidad correcta de ceremonia.

Ahora la máquina en sí. Las reglas son canon nuevo, escrito hoy; extienden los veredictos por sonda de la flota de TS (que juzgan una sola respuesta) hacia una política sobre la vida de un target a través de varias respuestas, y la mitad de TS va a adoptar los mismos estados cuando las dos mitades se encuentren. Un primer éxito lleva a Up un target Pending, una falla degrada un target Up en vez de matarlo, un target Degraded muere solo después de tres fallas consecutivas, y cualquier éxito lo recupera directo a Up. Escribe las reglas como un match sobre el par:

```rust
pub fn next_state(state: ProbeState, probe_ok: bool, consecutive_failures: u32) -> ProbeState {
    use ProbeState::*;
    match (state, probe_ok) {
        (Pending, true) => Up,
        (Pending, false) => Down,
        (Up, true) => Up,
        (Up, false) => Degraded,
        (Degraded, true) => Up,
        (Degraded, false) if consecutive_failures >= 3 => Down,
        (Degraded, false) => Degraded,
        (Down, true) => Up,
        (Down, false) => Down,
    }
}
```

![Cuatro estados de sondeo conectados por aristas de ok y fail, con la arista faltante de Pending a Degraded tachada porque ningún brazo de match la crea.](assets/v01-flowchart.webp)

Dos cosas en ese match son nuevas, y las dos se ganan su lugar. La tupla `(state, probe_ok)` deja que un solo match cubra la grilla completa de estado-por-desenlace, que es exactamente la forma de una tabla de transiciones. Y `if consecutive_failures >= 3` es una guarda de match: una condición extra atornillada a un solo brazo. Las guardas vienen con una regla que vale la pena decir en voz alta, porque te va a morder la primera vez que te apoyes en ellas: el compilador no puede ver adentro del booleano de una guarda, así que un brazo con guarda no cuenta para la exhaustividad. Por eso `(Degraded, false)` aparece dos veces, una con guarda y una pelada. Borra la pelada y E0004 vuelve, diciéndote que el brazo con guarda solo no es una promesa.

Acá va la parte que me tomó vergonzosamente mucho internalizar cuando aprendí esto: fíjate en lo que NO está en la función. Ninguna verificación `if state is valid`. Ninguna rama de error para transiciones ilegales. La regla "Pending nunca va directo a Degraded" no vive en ninguna parte, porque no necesita vivir en ninguna parte. No hay brazo que la produzca, así que no hay camino de código que la ejecute, así que el motor no puede hacerla, de la misma forma en que tu unión de m02-l1 no podía representar una sonda exitosa sin una latencia. Irrepresentable le gana a validado. Validación que nunca tienes que recordar le gana a validación que alguien va a olvidar con el tiempo.

### La unión que ya entregaste, vistiendo una bandera

Pon los dos artefactos lado a lado, porque ya diseñaste esta máquina una vez y me niego a pretender lo contrario. Desde tu `pulse-core`, m02-l1:

```typescript
type ProbeResult =
  | { kind: 'ok'; latencyMs: number }
  | { kind: 'timeout'; budgetMs: number }
  | { kind: 'http-error'; status: number };

function assertNever(value: never): never {
  throw new Error(`unhandled variant: ${JSON.stringify(value)}`);
}

switch (result.kind) {
  case 'ok': /* ... */ break;
  case 'timeout': /* ... */ break;
  case 'http-error': /* ... */ break;
  default:
    assertNever(result); // compile error here if a variant is unhandled
}
```

Un match sobre un enum de Rust ES ese switch exhaustivo, con una diferencia que cambia la experiencia diaria: el never-hack desapareció. En TS, la exhaustividad era un patrón que aplicabas; sáltate `assertNever` y el switch compila feliz con un caso faltante. En Rust es la semántica de `match`; no hay una versión sin chequeo en la que recaer por accidente. La misma garantía, pero en un lenguaje la cargas tú y en el otro el lenguaje te carga a ti. Y las variantes de Rust cargan datos directamente (`Ok { latency: LatencyMs }` de m04-l1) donde TS lo deletrea como formas de objeto con un campo discriminante. Sintaxis distinta, el mismo tipo suma, y una de las costuras TS-a-Rust de este curso se cierra justo acá: todo lo que aprendiste sobre modelar con uniones transfiere uno a uno.

![Una unión discriminada de TypeScript con su switch exhaustivo al lado del enum y el match equivalentes de Rust, mostrando que la garantía es opt-in de un lado y por defecto del otro.](assets/v02-annotated-code.webp)

### Structs, impl, derive: el trío que ya sabes a medias

Vienes usando los tres desde m04-l1, así que esto es nombrar, no enseñar. Un `struct` es tu tipo registro. Un bloque `impl` cuelga funciones de un tipo; un método que toma `&self` lee, `&mut self` muta. Acá va el que esta lección de verdad necesita, una comodidad que el código que renderiza el estado va a llamar:

```rust
impl ProbeState {
    pub fn is_alerting(&self) -> bool {
        matches!(self, ProbeState::Degraded | ProbeState::Down)
    }
}
```

`matches!` es una macro abreviada: se expande a un match que devuelve true para los patrones listados y false para todo lo demás. Lo que quiere decir, y guárdate esto para el lab, que contiene un brazo `_ => false` escondido. Es un catch-all disfrazado. Eso va a importar en unos veinte minutos.

### Un trait, porque una fuente debería ser intercambiable

El motor todavía lee latencias de datos de fixture, y en m05-l3 le crece un brazo de sonda HTTP de verdad. Esas son dos fuentes para la misma pregunta: ¿cuál es la próxima latencia? En TS irías a buscar una interfaz. La palabra de Rust es trait:

```rust
pub trait ProbeSource {
    fn next_latency(&mut self) -> Option<u64>;
}
```

Esa firma es un contrato congelado en este curso: cópiala textualmente, porque `drive` más abajo está escrito contra ella y nada de lo que viene después puede reformarla. `&mut self` porque una fuente avanza a medida que jalas de ella; `Option<u64>` porque toda fuente con el tiempo se seca, y ya sabes de m04-l2 que "quizás un valor" se deletrea Option, no un centinela como `-1`. Una nota honesta hacia adelante, para que este socket nunca se vuelva una promesa que el curso deja caer calladamente: el brazo HTTP de m05-l3 es una llamada suelta que devuelve una sola medición, no una implementación de `ProbeSource`, y esa lección dice en voz alta por qué mantiene a los dos separados. El trait es la costura que deja que `drive` corra sobre fixtures hoy y deja que una segunda fuente entre el día que se gane su lugar; ese día es m06-l1, cuando una fuente en vivo respaldada por reqwest finalmente toma el socket. La implementación de hoy es la respaldada por fixtures:

```rust
pub struct FixtureSource {
    latencies: Vec<u64>,
    cursor: usize,
}

impl FixtureSource {
    pub fn new(latencies: Vec<u64>) -> Self {
        Self {
            latencies,
            cursor: 0,
        }
    }
}

impl ProbeSource for FixtureSource {
    fn next_latency(&mut self) -> Option<u64> {
        let latency = self.latencies.get(self.cursor).copied();
        self.cursor += 1;
        latency
    }
}
```

`impl Trait for Type` es toda la ceremonia: declara que FixtureSource cumple el contrato de ProbeSource, y el compilador verifica que la firma coincida al pie de la letra. El código que consume una fuente nombra el trait, no el struct:

```rust
pub fn drive<S: ProbeSource>(source: &mut S, budget_ms: u64) -> ProbeState {
    let mut state = ProbeState::Pending;
    let mut consecutive_failures: u32 = 0;
    while let Some(latency) = source.next_latency() {
        let probe_ok = latency <= budget_ms;
        if probe_ok {
            consecutive_failures = 0;
        } else {
            consecutive_failures += 1;
        }
        state = next_state(state, probe_ok, consecutive_failures);
    }
    state
}
```

Ese `<S: ProbeSource>` es un bound genérico: "cualquier tipo S, mientras implemente ProbeSource." Estás leyendo genéricos-en-firmas ahora mismo, y leer es todo lo que este curso te pide; escribir tus propias abstracciones genéricas, trait objects y el vocabulario de bounds son exactamente la profundidad que el cuadro de abajo deja como bookmark. `while let` es el hermanito de match: haz loop mientras el patrón coincida, destructura el Some, para en None.

![Una función drive conectada a un socket de fuente de sondeo que acepta un enchufe de fixture ahora y un enchufe de HTTP en un módulo posterior.](assets/v03-diagram.webp)

### El beat con sabor: un conjunto de instrucciones es el partido de local de un enum

Un desvío antes de que aterrice el emparejamiento, porque este patrón exacto es por qué Rust es dueño del nicho de web3. Una transacción de Solana carga instrucciones, y una instrucción es una de un conjunto cerrado de operaciones, cada una con su propio payload: transfiere tantos lamports allá, delega autoridad a esa clave, cierra esta cuenta. Conjunto cerrado. Datos por variante. Dispatch exhaustivo. Vienes mirando fijo esa forma toda la lección:

```rust
#[derive(Debug)]
pub enum StationInstruction {
    Transfer { lamports: u64, to: [u8; 32] },
    Delegate { authority: [u8; 32] },
    Close,
}

pub fn describe_instruction(ix: &StationInstruction) -> String {
    match ix {
        StationInstruction::Transfer { lamports, to } => {
            format!("move {lamports} lamports to the address ending {:02x}", to[31])
        }
        StationInstruction::Delegate { authority } => {
            format!("hand probe authority to the key starting {:02x}", authority[0])
        }
        StationInstruction::Close => "tear the account down".to_string(),
    }
}
```

Tres variantes, tres formas de payload (campos nombrados, campos nombrados, ninguno en absoluto), un solo match que tiene que manejar cada operación o no compila. (La función es `describe_instruction`, no `describe`, porque el motor ya es dueño de un `describe` para `ProbeResult`, y dos funciones no pueden compartir un nombre en un módulo.) Un diseño de un struct por instrucción más un string `kind` no te da nada de eso: el conjunto cerrado se vuelve abierto, el dispatch se vuelve esperanza tipada con cadenas. Esto es práctica de modelado, dicho sin adornos: acá no se despliega nada, y los arrays `[u8; 32]` son arrays de bytes que hacen las veces de direcciones. Lo que los programas HACEN con las instrucciones es asunto del curso Master Anchor V2; por qué las transacciones cargan instrucciones siquiera le pertenece al curso de Bitcoin a Solana. Pedimos prestada la forma porque es el mejor argumento del mundo real a favor de los enums que cargan datos, y así el enum grande de instrucciones de un programa de Solana de verdad se lee como jugar de local, no como algo impresionante.

### El emparejamiento: cargo test es vitest vistiendo una bandera distinta

Ahora el segundo hilo del módulo. Allá en m02-l4 le diste a la flota de TS una suite de pruebas y la cableaste al pipeline como barrera #2. El motor de Rust viene viviendo sin nada de eso, y "¿cómo cambias código al que le tienes miedo?" tiene una respuesta de dos partes en este curso: haz irrepresentables los estados ilegales, después ponle barrera a todo lo demás. Acá va toda la historia del testing en Rust, y la versión honesta es que ya la sabes:

| pagas por esto, en TS | esto te sale gratis, en Rust | mismo trabajo |
|---|---|---|
| vitest | `cargo test` | corre las aserciones, falla el build |
| prettier | `cargo fmt` | termina para siempre con los diffs de estilo |
| eslint | `cargo clippy` | marca código que compila pero huele mal |
| tsc --noEmit | `cargo check` | ¿esto es siquiera un programa? |

Ninguna línea de install en esta sección y ese es el punto: las tres herramientas ya vienen en el toolchain estable que instalaste en m04-l1, al lado de rust-analyzer. Cero paquetes, cero archivos de config, cero debates de runner. Los mismos trabajos que la flota de TS le paga a cuatro dev-dependencies para que haga, en la caja. Misma idea, capataz más estricto.

Las pruebas viven en el mismo archivo que el código, adentro de un módulo que solo existe para los builds de prueba:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ProbeState::*;

    #[test]
    fn one_failure_degrades_instead_of_killing() {
        assert_eq!(next_state(Up, false, 1), Degraded);
    }

    #[test]
    fn third_consecutive_failure_goes_down() {
        assert_eq!(next_state(Degraded, false, 3), Down);
    }
}
```

`#[cfg(test)]` compila el módulo solo cuando pruebas, `use super::*` trae los ítems del archivo que lo envuelve, `#[test]` marca una función como prueba, `assert_eq!` es tu `expect(x).toBe(y)`. Esa es toda la superficie de API que necesitas este módulo. Pruebas guiadas por tabla, fixtures, los patrones de m02-l4: todos ellos traducen, y el lab escribe la suite de transiciones completa. Una diferencia cultural que vale la pena registrar sobre la marcha: tu flota de TS guarda las pruebas en archivos hermanos `*.test.ts`, mientras que la convención de Rust pone las pruebas unitarias en el MISMO archivo que el código que fijan, lo que a mí me pareció mal por como una semana y después se volvió la cosa que extraño en todos lados, porque la prueba y el match que ella cuida pasan al lado uno del otro en una sola pantalla. Rust también tiene un segundo tier, pruebas de integración en un directorio `tests/` de nivel superior que ejercitan tu crate desde afuera; el motor se gana ese tier cuando pase la partición del workspace el módulo que viene, así que déjalo estacionado.

clippy se merece una oración más, porque "el compilador ya verifica los tipos" es la objeción que escucha cada equipo cuando entra el step de lint. El verificador de tipos demuestra que tu código está bien formado. clippy discute sobre si es sabio: el cast `as` con pérdida, el Result ignorado, el clone innecesario del que aprendiste a desconfiar en m04-l1. Es el generador de comentarios de review, el asiento de eslint en la tabla, y con `-D warnings` (deny: promueve cada warning a error duro) deja de ser un consejo y se vuelve una barrera.

![Cuatro herramientas de TypeScript cada una emparejada con el comando de cargo que hace el mismo trabajo, convergiendo en un solo pipeline compartido.](assets/v04-comparison.webp)

Ya que estamos siendo honestos sobre el ecosistema: enséñale a tu editor a correr clippy, pero enséñale a tu Cargo.toml el ritmo real del ecosistema. El Rust actual es edition 2024, entregada en Rust 1.85.0 el 2025-02-20 (`cargo new` viene escribiendo `edition = "2024"` en tus manifests todo el módulo), y el rustc estable está en 1.98.0 al 2026-08-20, verificado el 2026-09-02, con una estable nueva cada seis semanas. Pero de cinco repos de Rust cercanos a Solana que este curso revisó en su investigación, cuatro todavía declaran edition 2021. Solo agave migró. El tren de releases corre a horario; el ecosistema se sube tarde, y eso es normal, no negligencia. Así que cuando un tutorial muestra `edition = "2021"`, no está equivocado, está más viejo, y cuando contribuyas a un repo de verdad, coincide con SU edition en vez de subírsela servicialmente en un PR drive-by. (Una nota al pie fechada para que nadie se sorprenda después: los cursos de Solana on-chain de la Academia escriben 2021 por regla, porque su Rust calificado construye sobre un toolchain que todavía no acepta edition 2024. Todo lo Rust de ESTE curso compila en tu propia máquina, donde 2024 es simplemente lo actual; m05-l2 es dueño de esa división completa cuando las editions reciban su tratamiento como corresponde.)

![Una línea de tiempo desde el release de edition 2024 hasta una revisión de 2026 donde cuatro de cinco repos de Solana todavía declaran la edition más vieja.](assets/v05-timeline.webp)

Un pedazo más de honestidad, mi hecho favorito de este módulo porque corta para los dos lados. El 2025-11-19, mientras el equipo del compilador de TypeScript portaba tsc a un lenguaje nativo por velocidad, Prisma fue en la dirección contraria: Prisma 7 borró su motor de queries en Rust a cambio de un compilador de queries en TypeScript, bundles alrededor de un 90% más chicos, un 3x en queries afirmado por el proveedor. Lee las dos jugadas juntas y la guerra de lenguajes se disuelve en la única pregunta real: la herramienta correcta para la capa. Un compilador quiere la envolvente de rendimiento de Rust; una capa de queries adentro de un proceso de Node quiere dejar de pagar el impuesto de frontera. Tu estación corre los dos lenguajes porque cada mitad se sienta en la capa en la que es mejor, y que esta lección les ponga barrera sobre un solo pipeline es la tesis del curso en miniatura.

**Profundiza (el 20%).** esta lección enseñó enums, match y un trait de la forma en que la flota los necesita a diario. La profundidad está dejada como bookmark a propósito: el capítulo completo de enums, el zoológico de métodos de Option (`map`, `and_then`, `unwrap_or` y amigos), la sintaxis de patrones más allá de tuplas y guardas, viven en The Rust Book cap. 6 ([https://doc.rust-lang.org/book/ch06-00-enums.html](https://doc.rust-lang.org/book/ch06-00-enums.html)), y escribir tus propios genéricos, trait bounds y trait objects en el cap. 10 ([https://doc.rust-lang.org/book/ch10-00-generics.html](https://doc.rust-lang.org/book/ch10-00-generics.html)), los dos verificados en vivo el 2026-09-02. Cuando este módulo termine, Rustlings es el patio de ejercicios: sus sets de ejercicios de enums y traits mapean uno a uno sobre el material de hoy. El lab de abajo no depende de nada de la profundidad dejada como bookmark.

### La parte honesta

La exhaustividad es un contrato con un precio, y vas a sentir el precio antes de amar el contrato. Cada variante nueva rompe cada match, en cada archivo, hasta que cada uno decide qué significa la variante. Magnífico para la corrección, ruidoso para la velocidad, y ese ruido es por qué tientan los catch-all de `_ =>`: un brazo comodín y los errores paran. Pero corre el trade-off hasta el final. Un catch-all te compra silencio de compilación hoy vendiendo la garantía misma por la que modelaste el enum; la próxima variante pasa de largo, clasificada en silencio como lo que diga el comodín, y estás de vuelta en el registro falsificado de m02-l1, en el lenguaje al que viniste por la garantía. La regla honesta: `_ =>` solo en fronteras de verdadero no-me-importa, y trata uno sobre una máquina de estados como un mal olor en el código. El mismo trade-off sobre la barrera de CI: `-D warnings` mantiene honesto al motor y de vez en cuando deja de rehén un merge inocente por un lint pedante. Esa fricción no es una falla. Esa fricción ES el code review.

## Lab: la máquina, el trait y la barrera #3

Chequeo de autonomía antes de empezar, porque este es el último bastión del loop de completion: los pasos 1 y 2 son reparaciones de código dado, el paso 5 está trabajado contigo manejando el push, y el challenge es enteramente tuyo. M5 vuelve al molde estándar de overview-lab-challenge; hoy te gradúas de las rueditas.

### 1. Termina el match

A `src/engine.rs` va la máquina, exactamente como se entrega acá, o sea: rota. El enum y los derives de la sección de teoría, `is_alerting`, y este next_state, con dos brazos de menos:

```rust
pub fn next_state(state: ProbeState, probe_ok: bool, consecutive_failures: u32) -> ProbeState {
    use ProbeState::*;
    match (state, probe_ok) {
        (Pending, true) => Up,
        (Pending, false) => Down,
        (Up, true) => Up,
        (Degraded, true) => Up,
        (Degraded, false) if consecutive_failures >= 3 => Down,
        (Degraded, false) => Degraded,
        (Down, false) => Down,
    }
}
```

Corre `cargo check` y usa el error como hoja de trabajo: E0004 nombra los dos patrones faltantes. Decide cada uno desde las reglas de la flota (una falla degrada, no mata; la recuperación es inmediata), escribe los dos brazos, y llega al silencio. Después fija la máquina con la suite de transiciones. Una regla de ubicación antes de que pegues: `engine.rs` ya termina en un bloque `#[cfg(test)] mod tests`, el que construyó m04-l2, y Rust admite exactamente un módulo por nombre, así que pegar este bloque textualmente debajo de él es E0428, "the name `tests` is defined multiple times." Haz merge en vez de eso: agrega las cinco funciones `#[test]` adentro del módulo existente, y agrega su línea `use ProbeState::*;` al lado del `use super::*;` que ya está ahí.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ProbeState::*;

    #[test]
    fn pending_first_success_goes_up() {
        assert_eq!(next_state(Pending, true, 0), Up);
    }

    #[test]
    fn one_failure_degrades_instead_of_killing() {
        assert_eq!(next_state(Up, false, 1), Degraded);
    }

    #[test]
    fn degraded_holds_below_three_failures() {
        assert_eq!(next_state(Degraded, false, 2), Degraded);
    }

    #[test]
    fn third_consecutive_failure_goes_down() {
        assert_eq!(next_state(Degraded, false, 3), Down);
    }

    #[test]
    fn down_recovers_straight_to_up() {
        assert_eq!(next_state(Down, true, 0), Up);
    }
}
```

```bash
cargo test
```

Checkpoint: las cinco pruebas de transición pasan, junto con todo lo que m04-l2 ya tenía en ese módulo, así que espera un total en el rango de diez-o-más dependiendo de cuántas pruebas de camino de parseo agregaste la lección pasada, con los cinco nombres de arriba en verde en la lista. Si una prueba de transición falla, llenaste un brazo con el estado target equivocado; el nombre de la prueba te dice qué regla releer.

### 2. Agrega la variante, sigue los errores

El ejercicio que responde la pregunta que guía. La estación va a necesitar un estado Maintenance algún día: sondas suspendidas a propósito, sin alertas. Agrégalo ahora y mira qué hace el compilador con tu miedo. En el enum:

```rust
    Down,
    Maintenance,
```

`cargo check`. E0004, en next_state, nombrando `(ProbeState::Maintenance, _)`. Cada match sin un catch-all ahora exige una decisión, y esa lista de errores es un inventario completo, escrito por el compilador, de cada lugar del motor que tiene que aprender qué significa Maintenance. Tus instintos de TS dicen que acabas de romper el proyecto. Replantéalo: le hiciste una pregunta al proyecto y te devolvió cada sitio relevante, por su nombre, con números de línea. Este es el beat de refactorizar-sin-miedo, y es la respuesta concreta a cómo cambias código al que le tienes miedo: haces que el compilador enumere el radio de daño, después recorres la lista. Dale sus brazos a Maintenance (las sondas suspendidas ignoran los desenlaces, así que tanto `(Maintenance, true)` como `(Maintenance, false)` se quedan quietos en Maintenance).

![Un error del compilador nombrando la nueva variante Maintenance en el match exacto que tiene que manejarla, mientras un sitio con comodín escondido se queda callado.](assets/v06-annotated-code.webp)

Ahora la trampa que te prometieron. `cargo check` está callado, pero pregúntate: ¿`is_alerting` aprendió sobre Maintenance? No lo hizo, y nunca se quejó, porque `matches!` esconde un brazo `_ => false`. El comodín decidió en silencio que Maintenance no está alertando, que resulta ser lo que queremos, por suerte, no por decisión. Eso es un catch-all haciendo exactamente lo que advirtió la sección del trade-off: absorber variantes nuevas sin avisarte. En una máquina de verdad ese silencio tiene dientes. Una vez agregué un estado detrás de un comodín en un pipeline de estado y me tomó dos días de paneles equivocados encontrar dónde se había tomado la decisión por mí, por un brazo `_` escrito meses antes. Quédate con `matches!` acá si aceptas conscientemente el comportamiento de false-por-defecto, y ahora sentiste los dos lados del trade-off del comodín en un solo ejercicio.

Termina el ejercicio revirtiendo: el canon de la flota es cuatro estados, y las pruebas del challenge están escritas contra exactamente esos cuatro. `git restore src/engine.rs` desde la raíz de `pulse-rs` si commiteaste antes del ejercicio (commiteaste antes del ejercicio, ¿sí?), o borra la variante y sus brazos a mano y deja que un `cargo check` limpio confirme la cirugía. Y como ese commit es el primero de `pulse-rs`, una jugada de higiene va delante de él: cargo se saltea escribir un `.gitignore` cuando hace scaffold adentro de un repo existente, y el propio archivo ignore de la estación es solo de node, así que agrega `target/` al final del `.gitignore` del repo de la estación antes de hacer `git add`, o vas a stagear el árbol de build entero, cientos de megabytes de salida del compilador que nadie revisa.

### 3. Aterriza el trait

Trabajado, con menos apoyo que el que te dio la sección de teoría. Agrega `ProbeSource`, `FixtureSource` y `drive` de la sección de teoría a `src/engine.rs`, textualmente, después enséñale la máquina a `main.rs`. El reporte de m04-l2 SE QUEDA: cada función del motor que ejercita se volvería código muerto en el momento en que lo borraras, y el código muerto es exactamente lo que la barrera de `-D warnings` del paso 5 se niega a entregar. Las líneas de la máquina van debajo del reporte, todavía adentro de `main`, justo arriba del `Ok(())` final. Dos pegados separados en dos lugares separados, así que mantenlos aparte. Primero, extiende los imports en el tope de `main.rs`, al lado de las líneas `use` de m04-l2:

```rust
use engine::{drive, parse_state, FixtureSource};
```

(`StationInstruction` deliberadamente todavía no se importa; no existe hasta el paso 4, e importarlo ahora es un error de import sin resolver.) Segundo, agrega al final adentro de `main`, debajo del println de fondeo de la estación:

```rust
    let boundary_input = "Pending";
    let Some(start) = parse_state(boundary_input) else {
        eprintln!("unknown state in fixture: {boundary_input}");
        return Ok(());
    };
    println!("starting from {start:?}");

    let mut source = FixtureSource::new(vec![212, 487, 1600, 1700, 1800, 90]);
    let state = drive(&mut source, 1500);
    println!("station state: {state:?} (alerting: {})", state.is_alerting());
```

Un indicio chico de que estás adentro del `main` de la lección pasada y no de uno fresco: la salida temprana es `return Ok(());`, porque `main` todavía devuelve `anyhow::Result<()>` y un `return;` pelado se negaría a compilar.

Una función referenciada ahí todavía no existe: `parse_state`. La regla de M2, parse, don't validate, viste su bandera de Rust acá. Las cadenas del mundo exterior se parsean al enum UNA sola vez en la frontera, y todo tierra adentro habla ProbeState:

```rust
pub fn parse_state(raw: &str) -> Option<ProbeState> {
    match raw {
        "Pending" => Some(ProbeState::Pending),
        "Up" => Some(ProbeState::Up),
        "Degraded" => Some(ProbeState::Degraded),
        "Down" => Some(ProbeState::Down),
        _ => None,
    }
}
```

Y ahí tienes tu `_ =>` legítimo: una frontera de verdadero no-me-importa, donde toda cadena desconocida significa exactamente una cosa, no-es-un-estado. Este es el hogar honesto del comodín. Si alguna vez los estados tipados con cadenas se cuelan de vuelta tierra adentro, si atrapas un estado `&str` en lo profundo del motor, arrastra el parseo de vuelta al borde.

Una nota de honestidad sobre el cableado que acabas de hacer, antes de que alguien que lea con cuidado pregunte: `start` se parsea, se imprime, y después nunca se le pasa a `drive`, porque `drive` hardcodea su propio arranque `ProbeState::Pending`. La rep de parseo es real (una cadena cruzó la frontera y se volvió un estado tipado o un rechazo a gritos), pero la plomería deliberadamente no está cerrada: la firma de `drive` es parte del contrato congelado de m05, así que hoy el estado parseado se detiene en el println. Si la variable colgando te molesta, buen instinto; una variante `drive_from(start...)` es un ejercicio de cinco líneas, solo no le cambies el nombre al `drive` congelado para hacerle lugar.

Checkpoint: `cargo run` imprime primero el reporte de fixtures de m04-l2, sin cambios, después `starting from Pending`, después `station state: Up (alerting: false)`. Antes de creerle a la salida impresa, recorre los seis fixtures a mano contra las reglas de transición: dos sondas limpias lo mantienen Up, después 1600 lo degrada, 1700 es la segunda falla consecutiva así que Degraded se mantiene, 1800 es la tercera así que la guarda se dispara y el target se va a Down, y la sonda final de 90ms lo recupera directo a Up. Si tu salida impresa dice Down en vez de eso, el brazo de recuperación es el sospechoso: un brazo `(Down, _) => Down`, o cualquier comodín que se trague `(Down, true)`, hace que Down sea terminal, y entonces el éxito final de 90ms no puede rescatar al target. Verifica que `(Down, true) => Up` exista y sea alcanzable. (Olvidarse de resetear `consecutive_failures` en un éxito también es un bug de verdad, pero este fixture no puede exponerlo: el éxito final recupera al target sin importar lo que diga el contador, lo que es en sí mismo una lección sobre lo que un solo fixture puede y no puede demostrar.)

### 4. El desvío de instrucciones, entregado

Agrega `StationInstruction` y `describe_instruction` de la sección de teoría al motor, extiende la línea de import del paso 3 a `use engine::{drive, parse_state, FixtureSource, StationInstruction};`, después despacha una fila en main, después del reporte de drive:

```rust
    let queue = vec![
        StationInstruction::Transfer {
            lamports: 2_000_000,
            to: [7u8; 32],
        },
        StationInstruction::Delegate {
            authority: [9u8; 32],
        },
        StationInstruction::Close,
    ];
    for ix in &queue {
        println!("instruction: {}", engine::describe_instruction(ix));
    }
```

Esa cifra de lamports es el sustituto de rent de la lección pasada haciendo un turno más como valor trabajado, nada más. Pequeña nota de honestidad: si agregas el enum sin usar cada variante y cada campo, `-D warnings` va a hacer fallar el build por lints de código muerto en el paso 5. Esa es estrictez de grado clippy del propio rustc, y es por eso que la fila de arriba construye las tres variantes. El código muerto en un binario es un warning; detrás de la flag de deny, los warnings son la ley.

### 5. Barrera #3: el pipeline aprende Rust

La re-entrega. Mismo repo, mismo `pulse.yml` que vienes creciendo desde m01-l3; asegúrate de que `pulse-rs/` viva adentro del repo de la estación (si le hiciste scaffold en otro lado en m04-l1, mueve el directorio adentro y commitea antes de cablear, con `target/` en el `.gitignore` de la estación según el paso 2). Después el diff del workflow, un job nuevo y una línea cambiada:

```yaml
  rust: # NEW: the engine's triple gate
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: pulse-rs
    steps:
      - uses: actions/checkout@v7
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo test
      - run: cargo clippy -- -D warnings
      - run: cargo fmt --check

  probe:
    needs: [typecheck, test, rust] # CHANGED: was [typecheck, test]
```

El step del toolchain se gana su porqué. La imagen ubuntu-latest de GitHub resulta que trae Rust 1.98.0 preinstalado al momento de escribir esto (sondeado el 2026-09-02), pero la imagen se actualiza según el calendario de GitHub, no el tuyo, y no lleva clippy. `dtolnay/rust-toolchain@stable` (la action de toolchain estándar de la comunidad, verificada en vivo el 2026-09-02) instala la estable actual más exactamente los componentes que nombres, así que el toolchain de la barrera es tu decisión en vez de un accidente de la imagen. `working-directory` apunta cada step `run` a la mitad de Rust del repo. Y `needs` gana un tercer nombre, que es el punto entero del segundo hilo de la lección: el cron que publica status.json ahora espera a los dos lenguajes.

Mira el tercer comando adentro del job antes de commitear, porque su flag carga todo el diseño. El `cargo fmt` local reescribe archivos en el lugar; `cargo fmt --check` no reescribe nada y falla si algo CAMBIARÍA. CI recibe la forma check, siempre, por la misma razón por la que el workflow de m02-l4 corrió `npx vitest run` en vez de modo watch: el trabajo de una barrera es rechazar, no arreglar. Es el step más barato del pipeline y compra la mayor paz social por segundo de cualquier barrera que vayas a agregar: con la máquina resolviendo el estilo, ningún pull request vuelve a gastar tres comentarios de review en dónde va una llave, y cada diff muestra solo lógica. Cuando se pone en rojo, el arreglo es un comando, sin decisiones de criterio: `cargo fmt`, commit, listo. El asiento de prettier de tu pipeline de TS, mismo razonamiento, bandera distinta. No lo suavices a solo-warning; una barrera de estilo que solo avisa se pudre hasta no ser barrera en un mes.

![Los triggers de push y de calendario alimentan typecheck, test y una nueva barrera rust cuyas aristas de needs cuidan todas el job probe que publica el estado.](assets/v07-flowchart.webp)

Rojo primero, siempre. Planta el lint del mundo de m04-l1, un clone innecesario sobre un tipo Copy, en `drive`:

```rust
        let probe_ok = latency.clone() <= budget_ms;
```

Commitea, empuja, mira la ejecución: el job rust falla en su step de clippy (`-D warnings` promoviendo el lint a error duro) con `using clone on type u64 which implements the Copy trait`, el job probe aparece como skipped, status.json no recibe commit. El motor escribió un comentario de review y bloqueó su propio merge. Fíjate en lo que NO lo atrapó: `cargo test` pasó, porque el clone es código correcto, solo que imprudente. Ese es el asiento de clippy haciendo un trabajo que el asiento de test no puede. Revierte el clone, corre el triple local antes de empujar, porque la tercera trampa de esta lección es cablear `-D warnings` al CI sin nunca correr clippy localmente, y después descubrir lints de a un push por vez como un linter que funciona con monedas:

```bash
cargo test && cargo clippy -- -D warnings && cargo fmt --check
```

Empuja, y mira la secuencia verde: tres barreras, después la sonda.

![Una ejecución de pipeline fallida detenida por el step de lint arriba de una segunda ejecución totalmente verde que llega a la publicación del estado.](assets/v08-diagram.webp)

Checkpoint, y es el del módulo: la ejecución de Actions del commit empujado muestra el job rust en verde AL LADO del job vitest. Sácale una captura. Dos lenguajes, un solo pipeline, tres barreras, y nada se entrega a status.json que los dos compiladores y las dos suites no hayan firmado.

## Challenge

La rep sin guía: el challenge de la máquina de estados de sondeo, en el panel de coding-challenge de esta lección. El starter compila y está mal de las dos formas que ahora sabes arreglar: Degraded falta por completo en la máquina, y las cadenas de estado desconocidas se filtran como Pending en vez de ser rechazadas. Para ser preciso sobre la forma, porque `"Invalid"` NO es un quinto estado: el `next_state` que ve el corrector toma el estado actual como cadena y devuelve el NOMBRE del próximo estado como cadena. Así que modela los cuatro estados canónicos como un enum de verdad, parsea la cadena entrante una sola vez en la frontera (el brazo `_ =>` del parseo es su hogar honesto, exactamente como `parse_state` en el lab), devuelve la cadena `"Invalid"` cuando ese parseo falle, y despacha cada estado parseado con éxito por un match exhaustivo sobre estado y desenlace con la guarda de fallas-consecutivas en tres. Ningún `_ =>` adentro de ese match interno, y el enum se queda en cuatro variantes, y acá la honestidad importa sobre quién hace cumplir eso: nadie más que tú. El corrector corre pares de entrada-salida y nunca lee tu código fuente, así que un atajo con comodín pasa cada prueba mientras esquiva la lección entera; grepea tu propia solución buscando `_ =>` antes de darla por terminada y asegúrate de que el único hit sea el parseo de frontera. Seis pruebas, incluyendo el camino de recuperación y la frontera entre la segunda y la tercera falla. Todo lo que necesitas está arriba; las pistas del starter escalan desde la forma del enum hasta la sintaxis de la guarda, gástalas en orden.

## Dónde deja esto al motor

Di la recuperación en voz alta antes de cerrar la terminal, triunfo de treinta segundos: vitest es a cargo test lo que prettier es a QUÉ lo que eslint es a QUÉ. Si las dos respuestas llegaron al instante, el emparejamiento aterrizó; es la bisagra sobre la que gira el quiz del módulo. Localmente, tu barrera triple corre los mismos tres comandos que el workflow ahora hace cumplir, lo que quiere decir que "funciona en mi máquina" y "va a pasar CI" se volvieron la misma oración, y esa identidad vale más que cualquier prueba individual.

Un pedido mientras la plantilla del módulo está fresca: esta fue la última lección con la textura más fina, reparaciones y reps trabajadas todo el camino hacia abajo, y el módulo que viene te vuelve a dar archivos en blanco. Si el ejercicio de agregar-una-variante fue el momento en que la exhaustividad hizo clic, o si todavía se siente como ceremonia, dilo en el feedback; ese ejercicio es la apuesta de la lección, y quiero saber si pagó.

El motor de Rust ahora coincide con la spec de la flota de TS: tipado, honesto con los errores, hecho máquina de estados, y con barrera en CI. Pero todavía lee latencias de un vector hardcodeado, y su config vive en el código fuente. El módulo que viene: serde parsea el MISMO archivo de config que parsea tu schema de zod, un solo archivo alimentando dos lenguajes, el crate se divide en un workspace de verdad, y la CLI le crece un brazo de sonda HTTP real, primero una llamada suelta con reqwest; enchufar HTTP en vivo detrás del trait mismo que congelaste hoy aterriza un módulo después, en la lección del poller de larga duración, desde el lado de la CLI, donde una fuente bloqueante sigue siendo legal. Un solo archivo de config está por servir a dos amos.
