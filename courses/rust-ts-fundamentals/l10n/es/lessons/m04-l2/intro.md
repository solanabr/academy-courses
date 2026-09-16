# Los errores son valores: Result, `?`, y el canon de dos crates

## Resumen

La lección pasada dejó `pulse-rs` compilando: tipos limpios de ownership y un clasificador puro corriendo sobre latencias de fixture. Pero cualquier entrada malformada todavía mata la corrida entera con un panic, y esa es la meta de hoy. No atrapar el panic. BORRARLO. Al final, cada desenlace de sondeo fluye por el motor como un `Result` que quien llama tiene que mirar, las líneas malformadas vuelven como valores `Err` impresos mientras el reporte termina alrededor de ellas, y tu matemática de dinero se niega a dar wrap. En el camino vas a escribir tu primer closure, partir el proyecto en una mitad librería y una mitad binario, y adoptar el canon de errores de dos crates exacto que los repos más grandes de Solana corren en producción. La textura del M4 sigue: el lab y las reps de reparación se trabajan conmigo o son repara-el-código-dado, y las únicas líneas que escribes sin guía adentro de ellos son el closure de `map_err` y los reemplazos de aritmética verificada. El challenge del final entonces rompe la textura de nada-de-archivos-en-blanco una vez, a propósito: una reconstrucción-y-conversión desde cero, porque la conversión solo demuestra que viaja cuando la escena del crimen es tuya.

## Mata el reporte primero

Abre `pulse-rs` de la lección pasada. Los fixtures de verdad no llegan como literales prolijos de `vec![LatencyMs(212)]`; llegan como texto, y el texto miente. Simula eso en dos minutos. Agrega este helper arriba de `main`:

```rust
fn parse_latency(line: &str) -> u64 {
    line.trim().parse().unwrap()
}
```

Y reemplaza el loop de fixture en `main` con una versión de texto crudo:

```rust
    let raw_fixture = ["212", "487", "fast", "1204", "930"];

    println!("target: {} ({})", target.name, target.url);
    for line in raw_fixture {
        let latency = LatencyMs(parse_latency(line));
        println!("{line}ms -> {:?}", classify_latency(latency));
    }
    println!("probes classified: {}", raw_fixture.len());
```

`cargo run`:

```text
target: solana-rpc (https://api.mainnet.solana.com)
212ms -> Up
487ms -> Degraded

thread 'main' panicked at src/main.rs:58:25:
called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
```

(La compilación además te recibe con tres warnings de código muerto, `describe`, `classify_probe`, y `ProbeResult` acaban de perder a los únicos que los llamaban cuando cambiaste el loop. Esperado, inofensivo y temporal: el paso 5 del lab devuelve a los tres a la nómina.)

Dos sondas clasificadas, después la muerte. `"fast"` no es un número, `parse` lo dijo, y `unwrap()` tradujo "lo dijo" a "mata el proceso." Al 1204 y al 930 nunca los miraron. ¿Te acuerdas de la flota v0 sin tipos del módulo 1, la que mentía con cortesía y anotaba los timeouts como cadenas de prosa? Este es su primo de Rust, salvo que más ruidoso: en vez de salida equivocada no tienes salida. Ninguna de las dos es lo que un operador mirando un reporte de flota necesita a las 3 a.m. Deja el panic donde está. Estamos por desarmarlo, y después lo vamos a borrar tan a fondo que un grep de `unwrap` en tu código de producción no devuelva nada.

## Los errores como valores de retorno, derivados

### Qué era ese panic en realidad

Un panic es Rust declarando el estado del programa no confiable y derribando el thread: desenrolla, imprime, muere. Es la herramienta correcta para "esto nunca puede pasar, y si pasó, la memoria es sospechosa." Es catastróficamente la herramienta equivocada para "un archivo de texto tenía un typo," que no es una emergencia, es martes. Y `unwrap()` es el puente de una palabra entre las dos: quiere decir "si esta operación falló, entra en panic." Cada `unwrap` en tu código es una pequeña confesión firmada de que elegiste no pensar en el caso de falla.

¿Entonces qué debería hacer `parse` en cambio cuando la entrada es basura, dado que Rust no tiene excepciones que lanzar? Acá está todo el diseño, y con honestidad apenas necesita derivarse: si una función puede fallar, dilo en el tipo de retorno. Eso es todo. Una excepción, vista en frío, es un segundo canal de retorno que nunca aparece en la firma: `JSON.parse` en tu TypeScript afirma que devuelve `any`, y el throw es una puerta lateral de la que te enteras en producción. Eso lo manejaste bien en M2, con `try/catch` en la frontera y una unión discriminada llevando el desenlace hacia adentro, y ese instinto estaba exactamente bien. Rust simplemente saca la puerta lateral por completo. La falla ES el valor de retorno, de primera clase, tipado, visible en cada firma por la que pasa.

![Una función falla por un canal lateral invisible mientras otra devuelve éxito o falla como un solo valor visible sobre el que quien llama tiene que ramificar.](assets/v01-comparison.webp)

### Result y Option son enums de los que ya eres dueño

Acá está la parte que debería sentirse como una repetición, porque lo es. `Result` está definido en la librería estándar más o menos así:

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Un enum con dos variantes, cada una llevando datos. Tú CONSTRUISTE uno de estos ayer: `ProbeResult` con sus variantes `Ok`, `Timeout` y `HttpError` eras tú haciendo errores-como-valores a mano, en tu propio dominio, antes de que el lenguaje te dijera que tenía una versión de propósito general. `Option<T>` es la misma idea para la ausencia: `Some(T)` o `None`, la respuesta de la librería estándar a `null`, salvo que el compilador te hace mirar antes de tocar. Y como son enums comunes, aplica la herramienta en la que ya confías: haz `match` sobre ellos, exhaustivamente, con el compilador negándose a dejarte olvidar un brazo.

```rust
match parse_latency("212") {
    Ok(ms) => println!("got {ms}"),
    Err(e) => println!("bad line: {e}"),
}
```

Los parámetros genéricos `<T, E>` son los primeros genéricos que conociste en Rust, y hoy solo necesitas la versión a nivel de lectura: `Result<u64, ParseIntError>` quiere decir "Ok lleva un u64, Err lleva un ParseIntError." Ese es todo el entendimiento requerido para este módulo.

Una nota de campo sobre lo universal que es esta forma: mientras construía las herramientas de este curso le pegué a la API de crates.io con un `curl` pelado y me rechazaron, porque crates.io rechaza las llamadas sin un header de User-Agent. El rechazo llegó como datos, un cuerpo que no era JSON y que mi script tuvo que parsear y reportar como un `Err`, no como una excepción que alguien aguas arriba se olvidó de capturar. Allá afuera en el mundo real, la falla es un valor más en el cable. El sistema de tipos de Rust está de acuerdo con la realidad, no inventando ceremonia.

### ? es retorno temprano con buen gusto

Hacer match en cada llamada falible se vuelve viejo rápido. Mira lo que le pasa a una función que hace tres cosas falibles con matches explícitos: se vuelve una escalera de bloques `match` donde la lógica de verdad se esconde en las esquinas. La respuesta de Rust es un carácter. Escribir `?` después de un `Result` quiere decir: si esto es `Ok(v)`, desenvuélvelo a `v` justo acá y sigue; si esto es `Err(e)`, devuelve `Err(e)` desde la función que lo encierra de inmediato. Retorno temprano, en el carril de error, con el camino feliz quedando plano y legible.

![Una llamada falible o bien sigue hacia abajo con su valor desenvuelto o bien sale temprano por una conversión From que lleva el error afuera de la función.](assets/v02-flowchart.webp)

Hay una cosa más escondida en ese carril rojo, y es el detalle que hace que `?` componga entre librerías: en la salida, el error pasa por el trait `From`. Si tu función devuelve `Result<T, ProbeError>` y la llamada interna falló con un `ParseIntError`, `?` va a convertir el error ajeno en el tuyo, siempre que exista una conversión. Cuando no existe, el compilador te frena, y vas a chocar exactamente contra ese muro en el lab, a propósito, porque el arreglo es tu primer closure. Nombra el mecanismo ahora, para que el mensaje de error se lea como información después: ? propaga, y convierte vía From en la salida.

Así que la regla de la casa, y fíjate que es gusto, no ley: nada de `unwrap()` en el camino de producción. En una prueba, `unwrap` está bien, con honestidad es lo correcto: un panic en una prueba ES el reporte de falla, entregado a ti, en tu escritorio. En un spike descartable, bien también. La diferencia es quién paga cuando se dispara. Pagas tú en tu escritorio, o paga un operador a las 3 a.m. mirando un reporte de flota impreso a medias. Escribe el unwrap donde tú seas el que se queda con la cuenta.

### El canon de dos crates: thiserror en el motor, anyhow en el binario

Ahora la capa del ecosistema, porque el manejo de errores de Rust de verdad son dos crates, y DÓNDE va cada uno importa más que cualquiera de los dos crates. La división sale de una sola pregunta: ¿quién consume el error?

Los errores de una librería los consume el código. Quien llama a tu motor quiere hacer `match` sobre qué salió mal, porque una línea malformada y un valor fuera de rango merecen tratamientos distintos. El código necesita variantes, así que una librería les debe a quienes la llaman un TIPO de error de verdad: un enum. Escribir el boilerplate de esos enums (la impl de `Display`, el cableado del trait) es lo que `thiserror` borra: lo derivas, anotas cada variante con una cadena de mensaje `#[error("...")]`, y el crate escribe el resto. Puro trabajo ahorrado, y lo más cercano que tiene Rust a una convención universal.

Los errores de un binario los consume un humano leyendo una terminal. `main` no hace match sobre variantes; reporta, con todo el contexto posible, y sale con código distinto de cero. Eso es `anyhow`: un solo `anyhow::Result` flexible al que se convierte cualquier error, más `.context("...")` para apilar migas de pan legibles por humanos en la subida.

El trade-off que encierra a los dos en sus carriles: la flexibilidad de anyhow viene de BORRAR la información de tipos que thiserror preserva. Usa anyhow en una librería y compila bien, se siente cómodo, y calladamente le roba a quienes te llaman la capacidad de hacer match sobre qué salió mal. Esa inversión es el error más común del mundo real con estos crates, que es exactamente por qué el canon son dos crates y no uno.

Y es el canon, no mi preferencia. Sondeé los archivos Cargo.toml de los repos estructurales de Solana el 2026-09-01: agave (el cliente de validador) y yellowstone-grpc están en thiserror 2.x, mientras que photon y jito-relayer todavía corren la línea 1.x, y anyhow va de acompañante en todos. Dos majors coexistiendo en un ecosistema es en sí mismo una lección chica: antes de subir de versión un crate fundacional, lee qué fija tu ecosistema de verdad. Los repos de los que dependes se mueven más lento que el tag latest de crates.io, y coincidir con tus vecinos le gana a perseguir el frente.

![Un enum de error tipado en el motor fluye por el operador de signo de pregunta hacia un result de anyhow en el binario, con los repos relevados listados debajo.](assets/v03-diagram.webp)

### Matemática de dinero: el compilador no te va a salvar de +

Un modo de falla más va acá porque NO se anuncia solo. Tus latencias y, más adelante en este curso, tus lamports son valores `u64`, y la suma de `u64` puede dar overflow. Acá está la parte fea: los builds de debug dan panic con el overflow, los builds de release dan wrap en silencio por defecto. La misma línea de código, dos comportamientos. Tu compañero de equipo dice "entrega el build de release, ahí no da panic," y tu compañero de equipo tiene técnicamente razón de la peor manera posible, porque un u64 que dio wrap no es un crash, es un número equivocado con cara de póker.

Trabájalo con valores de verdad. Digamos que un contador de balance se sienta cerca del tope del rango, en `u64::MAX - 1_000_000`, que es 18,446,744,073,708,551,615. Suma un depósito de 2,000,000 lamports en un build de release y la suma da wrap pasando el cero hasta 999,999. Dieciocho trillones y medio de lamports se vuelven como una milésima de SOL, sin panic, sin línea de log, y cada cálculo aguas abajo consume el cadáver alegremente. Yo entregué el primo de este bug: un ticker de balance que llegó a 18,446,744,073,709,551,615 en pantalla porque un reembolso aterrizó dos veces y mi resta dio wrap por debajo de cero. Nadie lo agarró en todo un día, porque nada crasheó. "No crasheó" no es "estaba bien."

![Una suma cerca del tope del rango de u64 se sale del final de una recta numérica y reaparece cerca de cero como un total diminuto que dio wrap.](assets/v04-diagram.webp)

El arreglo es la familia de métodos que la librería estándar hizo crecer exactamente para esto: `checked_add` y `checked_sub` devuelven `Option<u64>`, `Some(sum)` normalmente y `None` con el overflow. Y mira lo que eso devuelve: un Option, que alimenta directo de vuelta la maquinaria que acabas de aprender. El overflow deja de ser una corrupción de estado silenciosa y se vuelve un valor de error más subiendo por el mismo pipeline de `?` que una línea de fixture mala. Una sola historia de manejo de errores para todo el programa.

**Profundiza (el 20%).** esta lección enseñó el patrón de uso diario: Result y Option, `?`, la regla de la casa de nada-de-unwrap, la división de dos crates, la aritmética verificada. El tratamiento completo de errores recuperables contra irrecuperables, cuándo un panic es genuinamente correcto, y la superficie de API más profunda de `Result` es el capítulo 9 del Rust Book: [https://doc.rust-lang.org/book/ch09-00-error-handling.html](https://doc.rust-lang.org/book/ch09-00-error-handling.html). Y el closure que estás por escribir tiene un capítulo entero de profundidad detrás (los modos de captura, los traits Fn, closures como argumentos y como retornos) en el capítulo 13: [https://doc.rust-lang.org/book/ch13-00-functional-features.html](https://doc.rust-lang.org/book/ch13-00-functional-features.html) (los dos links verificados en vivo el 2026-09-02). Deja los dos como bookmark, lee ch09 esta semana. El lab no necesita ninguno de los dos.

## Lab: el reporte que sobrevive a sus fixtures

Estado meta, para que sepas cuándo terminaste: `pulse-rs` partido en un módulo engine (la mitad librería, errores tipados, sin imprimir, sin salir) y un binario main (la mitad anyhow, toda la impresión), leyendo de punta a punta un archivo de fixture deliberadamente sucio, con cero unwraps fuera de las pruebas.

1. **Agrega los dos crates.** Desde la raíz de `pulse-rs`:

   ```bash
   cargo add thiserror anyhow
   ```

   `cargo add` es npm-install para Cargo, y escribió las dos dependencias en `Cargo.toml` por ti. Hoy (2026-09-02) eso trae thiserror 2.0.20 y anyhow 1.0.104; tus dígitos de patch pueden ser más nuevos, y para estos dos crates famosamente estables eso está bien.

2. **Parte el proyecto en mitades.** Crea `src/engine.rs` y mueve ahí cada tipo y cada función de `main.rs` EXCEPTO `main` mismo: `ProbeTarget`, `LatencyMs`, `ProbeResult`, `Verdict`, `classify_latency`, `classify_probe`, `describe`, y el condenado `parse_latency`. Marca `pub` cada ítem que moviste (público: visible fuera del módulo; la privacidad de módulo es el default de Rust y la vamos a recorrer como se debe con Cargo en M6), y sé preciso sobre qué quiere decir "cada", porque la privacidad es por ítem, no por archivo: los dos structs necesitan `pub` en sus CAMPOS también, ya que `main.rs` los construye y los lee. `ProbeTarget` se vuelve `pub struct ProbeTarget { pub name: String, pub url: String }`, y el valor interno del newtype lo recibe igual: `pub struct LatencyMs(pub u64);`. Los enums son más fáciles: una variante es exactamente tan pública como su enum, así que `ProbeResult` y `Verdict` solo necesitan el único `pub` adelante de `enum`. Sáltate un campo y `cargo check` te recibe con un error de campo privado en el sitio de construcción en `main`. Después declara el módulo arriba del ahora diminuto `main.rs`:

   ```rust
   mod engine;
   ```

   Esa sola línea le dice a Cargo que `src/engine.rs` existe y es parte de este programa. Tu binario ahora tiene una mitad librería y una mitad binario, que es exactamente la costura que quiere el canon de dos crates.

3. **Deriva el tipo de error del motor.** Arriba de `engine.rs`:

   ```rust
   use std::num::ParseIntError;
   use thiserror::Error;

   /// A latency past this is a corrupted fixture line, not a slow probe.
   pub const MAX_SANE_LATENCY_MS: u64 = 60_000;

   /// Worked-example constant for the checked-math path. The number is a
   /// round synthetic stand-in, not the live rent rate; what an ATA
   /// actually is belongs to the Digital Assets course.
   pub const ATA_RENT_LAMPORTS: u64 = 2_000_000;

   #[derive(Debug, Error)]
   pub enum ProbeError {
       #[error("not a latency reading: {0}")]
       BadFixture(ParseIntError),
       #[error("{0}ms is past the {MAX_SANE_LATENCY_MS}ms sanity ceiling")]
       OutOfRange(u64),
       #[error("u64 arithmetic overflowed")]
       Overflow,
   }
   ```

   Léelo como lo que es: un enum pelado, como `Verdict`, salvo que cada variante es una forma en la que el motor puede fallar, y cada una lleva su evidencia. `BadFixture` guarda el `ParseIntError` de abajo así no se pierde nada; `OutOfRange` guarda el número ofensor. Las cadenas `#[error("...")]` son la representación legible por humanos, y el `{0}` interpola el campo de datos de la variante. Ese derive más esas cadenas reemplazan la docena de líneas de `impl Display` que si no escribirías a mano por cada tipo de error.

4. **Convierte el parseo, y conoce el muro.** Borra la versión con `unwrap` de `parse_latency` y escribe el par honesto. Primer intento, exactamente así:

   ```rust
   pub fn parse_latency(line: &str) -> Result<u64, ParseIntError> {
       line.trim().parse()
   }

   pub fn parse_fixture_line(line: &str) -> Result<LatencyMs, ProbeError> {
       let ms = parse_latency(line)?;
       if ms > MAX_SANE_LATENCY_MS {
           return Err(ProbeError::OutOfRange(ms));
       }
       Ok(LatencyMs(ms))
   }
   ```

   `cargo check`:

   ```text
   error[E0277]: `?` couldn't convert the error to `ProbeError`
      |
      |     let ms = parse_latency(line)?;
      |                                 ^ the trait `From<ParseIntError>` is not
      |                                   implemented for `ProbeError`
      |
      = note: the question mark operation (`?`) implicitly performs a conversion
        on the error value using the `From` trait
   ```

   Ahí está el mecanismo de la sección de teoría, en vivo: `?` intentó convertir `ParseIntError` en `ProbeError` vía `From`, no encontró ninguna conversión, y se detuvo. Existen dos arreglos. El que enseña esta lección es el map explícito en el carril de error, y escribirlo quiere decir escribir tu primer closure. Cambia la línea a:

   ```rust
       let ms = parse_latency(line).map_err(|e| ProbeError::BadFixture(e))?;
   ```

   Ahora frena y mira `|e| ProbeError::BadFixture(e)`, porque este es un momento chico grande: tu primer closure, llegando justo a tiempo, exactamente donde el lenguaje lo hace necesario. Un closure es una función anónima, escrita en línea, que puede capturar variables del scope de alrededor. `|e|` declara su parámetro, la expresión que sigue es su cuerpo, y la cosa entera es un VALOR que le pasas a `map_err` como cualquier otro argumento. `map_err` lo corre solo si el Result es un `Err`, transformando el carril de error y dejando el carril `Ok` intacto (`map` es su gemelo para el carril de valor; mezclar los dos es un tropiezo clásico de la primera semana, así que di en voz alta a cuál carril te refieres). Este todavía no captura nada; los closures que capturan, y las cadenas de iteradores donde los closures viven sus mejores vidas, llegan en el módulo que viene. La profundidad completa quedó como bookmark en el link de ch13 de arriba.

![Una línea de código es diseccionada con etiquetas para la llamada falible, el map del carril de error, el parámetro del closure, el cuerpo que envuelve, y el operador de propagación final.](assets/v05-annotated-code.webp)

   Corre `cargo clippy` antes de seguir y te va a fastidiar con esta línea exacta:

   ```text
   warning: redundant closure
       .map_err(|e| ProbeError::BadFixture(e))?;
                ^ help: replace the closure with the tuple variant itself:
                  `ProbeError::BadFixture`
   ```

   Clippy tiene razón, y el motivo vale el desvío: un constructor de variante tupla como `ProbeError::BadFixture` YA es una función, así que un closure que solo reenvía hacia él no agrega nada. `.map_err(ProbeError::BadFixture)` es idéntico y más corto. Igual nos quedamos con el closure escrito completo este módulo, para que la forma se quede adelante tuyo mientras es nueva, y se lo decimos a clippy explícitamente. Ponle este atributo a la función:

   ```rust
   #[allow(clippy::redundant_closure)]
   ```

   con un comentario que diga por qué y cuándo muere (colapsa el closure en M5, borra el allow). Un `#[allow]` con una fecha de vencimiento escrita es un préstamo de herramienta; un `#[allow]` sin una es cómo nace la deuda de lint. El segundo arreglo para el muro de E0277, por completitud: thiserror puede derivar la conversión `From` él mismo con un atributo `#[from]` en la variante, y a partir de ahí el `?` pelado simplemente funciona. Elegimos el closure explícito hoy porque necesitas VER la conversión una vez antes de dejar que un derive la esconda.

5. **Cablea la mitad anyhow.** Reemplaza `main.rs` debajo de la línea `mod engine;` con la mitad binario completa:

   ```rust
   use anyhow::{Context, Result};
   use engine::{LatencyMs, ProbeResult, ProbeTarget};

   fn main() -> Result<()> {
       let target = ProbeTarget {
           name: String::from("solana-rpc"),
           url: String::from("https://api.mainnet.solana.com"),
       };

       let raw = std::fs::read_to_string("fixture.txt")
           .context("could not read fixture.txt from the pulse-rs root")?;

       let mut clean: Vec<LatencyMs> = Vec::new();
       let mut rejected = 0u32;

       println!("target: {} ({})", target.name, target.url);
       for line in raw.lines() {
           match engine::parse_fixture_line(line) {
               Ok(latency) => {
                   println!("  {line}ms -> {:?}", engine::classify_latency(latency));
                   clean.push(latency);
               }
               Err(e) => {
                   println!("  {line:?} -> Err({e:?}): {e}");
                   rejected += 1;
               }
           }
       }

       // The structured path keeps a heartbeat on fixtures until the M6 daemon
       // feeds real network outcomes into the state machine.
       let structured = [
           ProbeResult::Ok {
               latency: LatencyMs(212),
           },
           ProbeResult::Timeout {
               budget: LatencyMs(3000),
           },
           ProbeResult::HttpError { status: 429 },
       ];
       for probe in &structured {
           println!(
               "  {} -> {:?}",
               engine::describe(probe),
               engine::classify_probe(probe)
           );
       }

       let total = engine::total_latency(&clean).context("summing the latency budget")?;
       let funding = engine::station_funding(1_000_000, engine::ATA_RENT_LAMPORTS, 50_000)
           .context("computing station funding")?;
       println!(
           "{} clean probes, {rejected} rejected, {total}ms total latency",
           clean.len()
       );
       println!("station funding needed: {funding} lamports");
       Ok(())
   }
   ```

   Recorre las costuras, porque cada una es una decisión. `main` devuelve `anyhow::Result<()>`, que es lo que deja que `?` funcione adentro: cualquier error que se escape se imprime con su cadena de contexto y el proceso sale con código distinto de cero, que es todo el trabajo de manejo de errores de un binario. La lectura del archivo lleva puesto `.context("...")`, una miga de pan para el humano. Y el loop del reporte es la tesis de la lección en cuatro líneas: `match` sobre el Result de cada línea, imprime el veredicto en `Ok`, imprime el error en `Err`, y SIGUE. Un valor de error no puede matar un loop; solo un panic puede. Las dos llamadas del bloque de resumen (`total_latency`, `station_funding`) todavía no existen; eso es el paso 6. Esto no va a compilar hasta que existan, y ahora puedes leer ese estado con calma en vez de supersticiosamente.

![Una línea de fixture se bifurca en un carril de error y un carril de valor que terminan los dos en el mismo reporte, que sigue en loop hacia la línea siguiente de cualquiera de las dos formas.](assets/v06-flowchart.webp)

6. **Haz que la matemática se niegue a mentir.** En `engine.rs`, agrega el camino de la suma y el helper de fondeo, los dos construidos sobre aritmética verificada que alimenta el mismo pipeline de errores:

   ```rust
   pub fn total_latency(latencies: &[LatencyMs]) -> Result<u64, ProbeError> {
       let mut total: u64 = 0;
       for latency in latencies {
           total = total.checked_add(latency.0).ok_or(ProbeError::Overflow)?;
       }
       Ok(total)
   }

   pub fn station_funding(base: u64, rent: u64, buffer: u64) -> Result<u64, ProbeError> {
       let subtotal = base.checked_add(rent).ok_or(ProbeError::Overflow)?;
       subtotal.checked_add(buffer).ok_or(ProbeError::Overflow)
   }
   ```

   `ok_or` es el método puente: convierte `Option` en `Result` aportando el error para el caso `None`, y de ahí en adelante `?` toma el control como siempre. Después fija el comportamiento con pruebas al fondo de `engine.rs`, incluyendo una que documenta lo que el modo release HABRÍA hecho, para que el número del wrap de la sección de teoría viva en forma ejecutable:

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn overflow_is_an_error_not_a_wrap() {
           let nearly_full = LatencyMs(u64::MAX - 1_000_000);
           let rent_sized = LatencyMs(ATA_RENT_LAMPORTS);
           assert!(matches!(
               total_latency(&[nearly_full, rent_sized]),
               Err(ProbeError::Overflow)
           ));
       }

       #[test]
       fn what_release_mode_would_have_done() {
           let nearly_full: u64 = u64::MAX - 1_000_000;
           assert_eq!(nearly_full.wrapping_add(ATA_RENT_LAMPORTS), 999_999);
       }

       #[test]
       fn epoch_wall_clock_shrank_with_faster_slots() {
           let at_400ms = 432_000u64.checked_mul(400);
           let at_300ms = 432_000u64.checked_mul(300);
           assert_eq!(at_400ms, Some(172_800_000)); // 48 hours of milliseconds
           assert_eq!(at_300ms, Some(129_600_000)); // 36 hours of milliseconds
       }
   }
   ```

   El atributo `#[cfg(test)]` compila este módulo solo para `cargo test`, que es por qué los unwraps y los panics son ciudadanos legales adentro. Esa última prueba es un precalentamiento de aritmética verificada con los números propios de Solana: un epoch está fijo en 432,000 slots, así que cuando la meta de tiempo de slot de la red bajó de 400ms a 300ms este agosto, los epochs se achicaron de 48 horas a 36 en tiempo de reloj. Mismo conteo de slots, latido más rápido, derivable en una línea de matemática honesta de u64. Agrega dos o tres pruebas propias más para el camino del parseo (una línea basura es `Err(BadFixture(_))`, un día en milisegundos es `Err(OutOfRange(_))`; `matches!` es la forma de una línea de afirmar la forma de un enum).

7. **Dale mugre y verifica.** Crea `fixture.txt` en la raíz del proyecto, deliberadamente inmundo:

   ```text
   212
   487
   fast
   1204
   86400000
   930
   ```

   Después la barrera completa:

   ```bash
   cargo fmt
   cargo clippy
   cargo test
   cargo run
   ```

   fmt en silencio, clippy con cero warnings (el único lint que nos ganamos está explícitamente permitido, con su nota de vencimiento), pruebas en verde, y la corrida imprime:

   ```text
   target: solana-rpc (https://api.mainnet.solana.com)
     212ms -> Up
     487ms -> Degraded
     "fast" -> Err(BadFixture(ParseIntError { kind: InvalidDigit })): not a latency reading: invalid digit found in string
     1204ms -> Down
     "86400000" -> Err(OutOfRange(86400000)): 86400000ms is past the 60000ms sanity ceiling
     930ms -> Degraded
     212ms -> Up
     no answer in 3000ms -> Down
     HTTP 429 -> Degraded
   4 clean probes, 2 rejected, 2833ms total latency
   station funding needed: 3050000 lamports
   ```

   Pon eso frente a la apertura. La misma clase de basura en la entrada, y en vez de dos líneas y un cadáver tienes el reporte entero: veredictos para las cuatro sondas limpias, un motivo tipado e impreso para cada uno de los dos rechazos (la forma de debug Y la representación de `#[error]` una al lado de la otra), y totales honestos. El panic de la apertura ya no está, y un solo grep demuestra qué tan ido está:

   ```bash
   grep -n "unwrap" src/main.rs src/engine.rs
   ```

   El resultado esperado es silencio, y para el código exactamente como está dado, silencio total: el camino de producción tiene cero unwraps, y el módulo de pruebas de esta lección justo afirma con `matches!` y `assert_eq!` en vez de `unwrap`, así que el grep no devuelve absolutamente nada. Si las pruebas extra que escribas echan mano de `unwrap` (legal ahí, y seguido la decisión correcta), la regla de auditoría es: cada coincidencia tiene que estar DEBAJO de la línea `#[cfg(test)]` en `engine.rs`, y `main.rs` no debe mostrar ninguna. Los unwraps del lado de las pruebas son panics funcionando como se pretende, en tu escritorio, como reportes de falla.

### Reps de reparación: tres arreglos de los que ahora eres dueño

El loop de completion, la misma textura que la lección pasada: código roto o feo, una reparación con principios cada uno, en un banco borrador (`cargo new error-reps && cd error-reps && cargo add thiserror`). Una jugada de preparación antes de la rep 1: copia el enum `ProbeError` del lab al `main.rs` del banco, junto con sus líneas `use std::num::ParseIntError;` y `use thiserror::Error;`, porque la rep 1 lo devuelve y el `HeaderError` de la rep 2 viene dado mientras que `ProbeError` se asume. Predice antes de comprobar.

**Rep 1, escribe el closure sin guía.** Esta función no va a compilar. Arréglala escribiendo tú mismo el closure de `map_err`, sin espiar el lab:

```rust
fn checked_line(line: &str) -> Result<u64, ProbeError> {
    let ms = line.trim().parse::<u64>()?;
    Ok(ms)
}
```

**Rep 2, tres unwraps, tres arreglos DISTINTOS.** Este helper funciona justo hasta que cualquiera de sus tres suposiciones se rompe. Conviértelo para que devuelva `Result<String, HeaderError>` (el enum de abajo viene dado; los arreglos son tuyos). El pero: los tres unwraps merecen tres tratamientos distintos, y saber cuál es cuál es la habilidad de verdad. Uno debería propagarse con `?` y un `map_err`, uno debería volverse un `Err` devuelto vía `ok_or`, y uno debería tragarse con `unwrap_or_default` más un comentario que defienda el trago:

```rust
#[derive(Debug, Error)]
enum HeaderError {
    #[error("the fixture is empty")]
    EmptyFixture,
    #[error("the first line is not a probe count: {0}")]
    BadCount(std::num::ParseIntError),
}

fn report_header(raw: &str) -> String {
    let first = raw.lines().next().unwrap();
    let count: u64 = first.trim().parse().unwrap();
    let label = std::env::var("STATION_NAME").unwrap();
    format!("{label}: expecting {count} probes")
}
```

Chequeo de razonamiento, antes de escribir: la variable de entorno que falta es el trago defendible (una estación sin nombre es molesta, un reporte muerto es peor), la entrada vacía es un error de verdad del que quien llama tiene que enterarse, y el conteo malo sube montado en `?` como `BadCount`. Si los asignaste distinto, discútelo conmigo en el feedback; hay espacio honesto en la variable de entorno.

![Tres preguntas de sí o no enrutan una llamada falible hacia uno de cuatro tratamientos, desde quedarse con unwrap en las pruebas hasta propagar, poner un default, o devolver un error.](assets/v07-flowchart.webp)

**Rep 3, el reemplazo de aritmética verificada, sin guía.** De vuelta en `pulse-rs`: esta versión de `station_funding` compila, pasa una prueba de camino feliz, y miente bajo presión. Reemplaza los dos operadores `+` con aritmética verificada que mapee `None` a `ProbeError::Overflow`, después fija la reparación con una prueba tuya, porque la prueba de overflow del paso 6 ejercita `total_latency`, no esta función. Escribe `station_funding_overflow_is_an_error` con la misma forma (alimenta `u64::MAX - 1_000_000` como `base` y `ATA_RENT_LAMPORTS` como `rent`, afirma `Err(ProbeError::Overflow)` con `matches!`) y haz que pase:

```rust
pub fn station_funding(base: u64, rent: u64, buffer: u64) -> Result<u64, ProbeError> {
    Ok(base + rent + buffer)
}
```

Aceptación para las reps: las tres compilan, `cargo test` en verde, y para cada unwrap que sacaste puedes decir en una oración quién habría pagado cuando se disparó.

## Challenge

La rep sin guía es un barrido, y tú mismo construyes la escena del crimen para que la conversión sea honesta. `cargo new no-unwrap-report && cd no-unwrap-report && cargo add thiserror anyhow`, después reconstruye un binario chico de reporte de fixture en el estado en el que estaba tu `pulse-rs` esta mañana: el parseo acribillado de `unwrap` de la apertura y el loop de fixture crudo (ponle cuatro o cinco unwraps en el camino de producción ya que estás, una variable de entorno, una lectura de archivo, un `lines().next()`), un `+` pelado en su total de lamports, y un `fixture.txt` sucio que lo mata tres líneas adentro. Después conviértelo de punta a punta: un enum de thiserror en su módulo engine, anyhow con contexto en su `main`, cada unwrap reemplazado por el tratamiento que merece, aritmética verificada en el total. El grader es la misma barrera que acabas de correr a mano: la corrida tiene que completarse sobre el fixture sucio imprimiendo líneas `Ok` y `Err`, `grep -rn 'unwrap()' src/` no tiene que imprimir nada fuera de los módulos de prueba (fíjate en el `-r`; un `grep -c` pelado sobre un directorio se niega a correr), y la prueba de overflow tiene que pasar. Cuida el patrón exacto: es `'unwrap()'` con los paréntesis, no `unwrap` pelado, porque `unwrap_or_default` y sus hermanos son tratamientos, no confesiones, y el arreglo de la variable de entorno de la rep 2 haría saltar un grep de `unwrap` pelado siendo exactamente correcto. Todo lo que necesitas está arriba; si te trabas, escala siguiendo el orden propio del lab, encuentra qué línea muere primero, después el closure de `map_err`, después el puente de `ok_or`.

## Checkpoint

Lo que ahora puedes hacer, concretamente: leer una firma de `Result` como un contrato en vez de como ceremonia; propagar con `?` y explicar el salto de `From` en el carril rojo; escribir un closure en `map_err` y decir qué lo hace un closure; dividir los errores según el canon de dos crates y defender la división con el trade-off (anyhow borra lo que thiserror preserva); y negarte a la aritmética que da wrap en cualquier lugar donde vivan números con forma de dinero. Tu `pulse-rs` sobrevive un archivo de fixture inmundo y dice exactamente por qué fue rechazada cada línea mala, en un tipo sobre el que quien llama podría hacer match.

La recuperación de 30 segundos antes de que cierres la pestaña, en voz alta: ¿qué hace `?` con un `Err`? (Retorna temprano el error desde la función que lo encierra, convirtiendo vía `From` en la salida.) ¿Y cuál perfil de build da wrap con el overflow? (Release. Debug da panic. Ninguno de los dos es un sustituto de `checked_add`.)

Un pedido mientras está fresco: en la rep 2, ¿la división en tres arreglos distintos se sintió con principios o arbitraria, y cuál unwrap casi arreglas mal? Dímelo en el feedback. Esa rep es la filosofía entera del módulo en nueve líneas, y si el trago de la variable de entorno se sintió como hacer trampa quiero escuchar el argumento, porque "cuándo es aceptable un default" es un debate que los equipos de verdad tienen todas las semanas.

Los desenlaces ahora son valores. Pero la VIDA de una sonda a lo largo del tiempo (pending, up, degraded, down) sigue siendo datos sueltos que tu código simplemente se acuerda de actualizar. La lección que viene: los enums como máquinas de estados, el compilador sosteniendo la pluma en cada transición legal, más la barrera de cargo test, clippy y fmt cableada en tu workflow de Actions, que es la respuesta honesta a "¿cómo cambias código al que le tienes miedo?" Nos vemos ahí.
