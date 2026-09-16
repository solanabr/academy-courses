# clap, un brazo de sonda de verdad, y un binario que cualquiera puede descargar

## Resumen

m05-l2 le dio al código la forma del ecosistema: un workspace de dos crates con las deps declaradas una sola vez, editions y MSRV elegidos a propósito, y tres pins del mundo real leídos como un dev que trabaja. El motor ahora es una librería. Y nada lo llama excepto las pruebas. Hoy eso se termina por triplicado: la CLI gana una interfaz de verdad con clap, el camino de sonda de la estación por fin toca una URL viva por HTTP (del lado de la CLI; el motor sigue sin I/O, exactamente como se diseñó), y CI empieza a publicar un binario que un desconocido puede descargar y correr sin ver nunca tu código fuente. Este es el último build del tier de Rust, un artefacto entregado más en el estante de la estación, y los apoyos son los más delgados del módulo: el derive de clap es trabajado conmigo, el fetch es guiado, las mediciones y el segundo subcomando son tuyos, y la extensión del workflow al final es totalmente sin guía. Ese es el repliegue terminando lo que M4 empezó.

## Diez minutos hasta una pantalla de ayuda que nunca escribiste

Haz esto antes de leer cualquier otra cosa. En la raíz de tu workspace `pulse-rs`, abre `crates/pulse-cli/Cargo.toml` y agrega una dependencia:

```toml
clap = { version = "4.6", features = ["derive"] }
```

(`cargo add clap --features derive` desde adentro de `crates/pulse-cli` hace la misma edición y escribe el dígito exacto de hoy, 4.6.6. Vigente en crates.io al 2026-09-02; la línea 4.x viene estable desde el 2022-09-28, así que el pin es tranquilo. Y sí, local al member y con versión inline, una lección después del sermón de hoisting: a propósito. La regla de m05-l2 se gana su lugar con las deps COMPARTIDAS, y clap, igual que el reqwest que agregas después, tiene exactamente un consumidor; súbelos a `[workspace.dependencies]` el día que un segundo crate quiera cualquiera de los dos.)

Ahora reemplaza `crates/pulse-cli/src/main.rs` con este esqueleto. Fíjate en lo que el reemplazo deja en pausa, a propósito, no por olvido: el cuerpo de m05-l2 que leía `pulse.config.json` por `parse_config` desaparece, porque hoy los subcomandos de la CLI toman su target de argv. El cableado de la config vuelve cuando el poller de m06 corra flotas enteras en un calendario, y la firma congelada de `parse_config` es exactamente lo que va a llamar; nada de la superficie del motor cambia mientras tanto.

```rust
use clap::{Parser, Subcommand};

/// Pulse Station's Rust probe arm.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Probe one URL and print its status and latency
    Probe {
        /// The target to hit, scheme included
        url: String,
    },
    /// Run the latency-stats pass over a synthetic fixture set
    Report,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Probe { url } => println!("would probe {url}"),
        Command::Report => println!("would report"),
    }
    Ok(())
}
```

(Conserva la firma `anyhow::Result<()>` de `main` que viene de m05-l2 aunque todavía nada falle; el fetch que conectas después usa `?`, que solo compila adentro de una función que devuelve `Result`, y la cola `Ok(())` es todo el costo.)

Córrelo (el doble guion pelado es el separador de cargo: todo lo que va después va a TU binario, no a cargo):

```bash
cargo run -- --help
```

Y mira lo que vuelve:

```text
Pulse Station's Rust probe arm

Usage: pulse-cli <COMMAND>

Commands:
  probe   Probe one URL and print its status and latency
  report  Run the latency-stats pass over a synthetic fixture set
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Una pantalla de ayuda formateada y versionada, con líneas de uso, resúmenes de subcomandos y un `-V` que funciona. No escribiste nada de eso. Allá en M1 el manejo de argumentos en la CLI `pulse` de TS era tu código y tus bugs: el slicing de `process.argv`, la verificación de "¿pasaron una URL?", la cadena de uso que te olvidabas de actualizar. Acá la interfaz es derivada, el mismo truco que sacó serde el módulo pasado: el tipo es el spec, y el macro escribe la maquinaria. Para superficies de CLI, es el mejor trade-off de la caja de herramientas.

## De derivado a descargable

### El tipo es la interfaz

Mira en qué se convirtió cada pieza de ese struct. El doc comment del struct `Cli` se convirtió en la línea del about. El doc comment de cada variante del enum se convirtió en su resumen de una línea en la lista de comandos. El campo `url: String` se convirtió en un argumento posicional requerido, y como está tipado, clap se queda con las quejas:

```bash
cargo run -- probe
```

```text
error: the following required arguments were not provided:
  <URL>

Usage: pulse-cli probe <URL>

For more information, try '--help'.
```

Agrega una flag tipada y el parseo de valores también viene gratis. Dale un timeout a `Probe`:

```rust
    Probe {
        /// The target to hit, scheme included
        url: String,
        /// Request timeout in seconds
        #[arg(long, default_value_t = 10)]
        timeout: u64,
    },
```

(El compilador va a señalar de inmediato que tu brazo de `match` ahora necesita el campo nuevo: `Command::Probe { url, timeout }`. Déjalo guiarte; esa es la exhaustividad trabajando para ti, no contra ti.) Ahora `--timeout 5` se parsea a un `u64`, sin flag quiere decir 10, y la basura se rechaza con un mensaje que nombra la flag, el valor y la razón: `error: invalid value 'abc' for '--timeout <TIMEOUT>': invalid digit found in string`. Cada uno de esos comportamientos es código que no escribiste y pruebas que no mantienes.

![Cada línea de un struct derivado de clap se mapea con una flecha al texto de ayuda, la flag o la validación que genera.](assets/v01-annotated-code.webp)

El mecanismo importa porque ya conociste su inverso. Rust no tiene reflexión en runtime: nada puede inspeccionar tu struct mientras el programa corre. Así que `#[derive(Parser)]` hace su trabajo en tiempo de compilación, leyendo la definición del struct y generando ahí mismo el código de parseo, validación y ayuda, exactamente como `#[derive(Deserialize)]` lo hizo con tu archivo de config en m05-l1. Misma jugada, blanco distinto: serde deriva la frontera de los datos, clap deriva la frontera humana.

Y acá tu músculo de leer pins de la lección pasada hace ejercicio. El clap que acabas de agregar es 4.6.6. agave, el codebase principal del validador de Solana, fija `clap = "2.33.1"`, una major de hace más o menos una década, y lo entrega a producción todos los días. Después de m05-l2 puedes leer ese pin en vez de confundirte con él: una major de hace una década en un repo mantenido activamente es una decisión, sostenida por algo, probablemente la pura superficie de migrar cada flag de CLI que expone un validador. Tu proyecto, tu 4.x. Su repo, su 2.x. Las dos correctas, y ahora sabes cómo debería verse un PR contra cada una.

Una frontera antes de seguir: todo lo de arriba es la API derive de clap, structs adentro, interfaz afuera. clap también tiene una API builder donde construyes el parser a mano en runtime, y capas de dynamic-completion y custom-help debajo de eso. El uso diario es el derive. El resto queda como bookmark al final de esta sección.

### El brazo de sonda, al fin

Dos módulos de Rust, y cada latencia que tu motor clasificó fue un fixture. Ese fue el trato que hicimos en m05-l1, dicho en voz alta: las latencias siguen siendo fixtures, el brazo de HTTP llega en m05-l3. Es m05-l3. Agrega la segunda dependencia a `crates/pulse-cli/Cargo.toml`:

```toml
reqwest = { version = "0.13", features = ["blocking"] }
```

reqwest 0.13.4 es la vigente al 2026-09-02, y técnicamente ya conociste este crate: fue la estrella del ejercicio de leer pins en la lección pasada, donde agave se sienta en 0.12.28, una major atrás. Ahora lo tienes tú. La feature `blocking` no es decoración opcional. La superficie por defecto de reqwest es async, y sin la feature el módulo `reqwest::blocking` simplemente no existe. Olvídala y el error del compilador apunta a un módulo que falta, no a tu Cargo.toml, lo que la vuelve una primera trampa genuinamente malvada: el arreglo vive en un archivo distinto del error.

¿Por qué blocking, cuando cada tutorial de HTTP en Rust de internet va por async? Por lo que este programa es. Una CLI sondea un target, imprime una línea y sale. No hay concurrencia que explotar, así que un runtime async sería puro overhead y un concepto nuevo gastado sin beneficio. Blocking es la decisión de ingeniería correcta para esta forma de programa, elegida a propósito, no un atajo. El costo honesto: en el momento en que quieras cincuenta sondas en vuelo en un calendario, blocking se vuelve el cuello de botella. Esa presión exacta es el problema con el que abre el módulo que viene, y el delta async crece de este mismo crate. Una mina que hay que señalar ahora para que nunca detone después: el cliente blocking da panic si se lo llama desde adentro de un runtime async. Bien hoy, en una CLI pelada sin runtime en ninguna parte. Recuérdalo en m06-l1.

Acá está el fetch trabajado, la forma de todo el asunto:

```rust
use std::time::Instant;

fn probe(url: &str, timeout: u64) -> Result<(), ProbeError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout))
        .build()
        .map_err(|e| ProbeError::Unreachable {
            reason: e.to_string(),
        })?;

    let started = Instant::now();
    let response = client
        .get(url)
        .send()
        .map_err(|e| ProbeError::Unreachable {
            reason: e.to_string(),
        })?;
    let elapsed = started.elapsed();

    println!(
        "{url} -> {} in {} ms",
        response.status(),
        elapsed.as_millis()
    );
    Ok(())
}
```

(Los saltos de línea adentro de los closures `map_err` y del `println!` son de rustfmt, no de gusto: esta es la forma en la que se asienta `cargo fmt`, impresa ya asentada para que el `fmt --check` de la barrera de CI no tenga nada de qué quejarse. Escríbelo más apretado y la barrera va a pedir exactamente esto.)

Veintitantas líneas, y la mitad son el camino del error. Acá está el viaje entero en un dibujo antes de que disequemos esa mitad.

![Una sonda fluye desde el parseo de argumentos por el cliente blocking hasta una línea de estado impresa o un error mapeado.](assets/v02-flowchart.webp)

Lee el camino del error de cerca, porque es tu músculo de m04-l2 disparando en un gimnasio nuevo. `send()` devuelve un `Result` cuyo tipo de error es `reqwest::Error`, un desconocido para la taxonomía `ProbeError` de tu motor. El closure `map_err` es el puente, la misma jugada que usaste para jalar el error de serde hacia `BadConfig` la lección pasada, y el error de parseo de `std` hacia `BadFixture` allá en m04-l2. La variante en la que aterriza es nueva:

```rust
#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("not a latency reading: {0}")]
    BadFixture(ParseIntError),
    #[error("{0}ms is past the {MAX_SANE_LATENCY_MS}ms sanity ceiling")]
    OutOfRange(u64),
    #[error("u64 arithmetic overflowed")]
    Overflow,
    #[error("config rejected: {0}")]
    BadConfig(serde_json::Error),
    #[error("probe could not reach the target: {reason}")]
    Unreachable { reason: String },
}
```

(Las primeras cuatro están reimpresas exactamente como las dejaron m04-l2 y m05-l1, mensajes incluidos, para que puedas hacer diff de esto contra tu propio archivo: solo la última variante es nueva. Si tus cadenas de `#[error]` se leen distinto porque escribiste las tuyas, quédate con las tuyas, las cadenas son tuyas para frasearlas; las FORMAS de las variantes son en lo que se apoya el resto del curso, y `BadFixture(ParseIntError)` en particular es contra lo que compila el colapso `.map_err(ProbeError::BadFixture)?` de m05-l1.)

Fíjate en lo que lleva `Unreachable`: un `String`, no un `reqwest::Error`. Eso es a propósito, y es la decisión arquitectónica de la lección. El enum vive en `pulse-engine`, y si la variante llevara el tipo de error de reqwest, al motor le crecería una dependencia de reqwest, y el núcleo puro que has estado protegiendo desde M4 ya no sería puro. ¿Por qué importa tanto esa pureza como para achatar un error a una cadena? Porque el motor tiene más futuros que esta CLI: m06 lo quiere adentro de un poller de larga vida, y m07-l2 lo quiere compilar a WASM para un Worker de Cloudflare, un entorno donde una stack HTTP nativa no puede seguirlo. Un motor sin I/O porta; un motor con un socket dentro, no. Así que el brazo de HTTP vive en `pulse-cli`, el motor se queda siendo una calculadora, y la costura entre los dos es un `String` cruzando una frontera de crate.

Una promesa mantenida honesta mientras estamos acá: este `probe` es una llamada suelta, y NO implementa el trait `ProbeSource` que congelaste en m04-l3. `drive` nunca ve estas latencias hoy; la línea impresa es todo el producto. Tampoco lo enchufa después el poller de m06: ese daemon llama a `next_state` directo con el resultado de cada sonda, que es el camino más corto cuando hay exactamente un tipo de fuente y ya corre bajo un runtime async. Así que el trait se queda siendo para lo que m04-l3 lo construyó, la costura que deja a `drive` correr contra latencias enlatadas en vez de un socket, que es precisamente lo que hace el `tests/engine_contract.rs` de m05-l2 desde el tier de integración, sentada lista para el día en que aparezca una segunda fuente, y ese día es la próxima lección: m06-l1 cierra el loop con una fuente reqwest blocking enchufada detrás del trait, justo acá en `pulse-cli`, donde el cliente blocking sigue siendo legal. Si te pican los dedos por escribir `impl ProbeSource for` una fuente respaldada por reqwest ahora mismo, son ganas sanas y precisamente la jugada de m06-l1, así que aguántalas una lección; nada en esta lección, el lab, ni el challenge lo espera.

![Un crate de motor puro sin I/O se sienta al lado de un crate de CLI que lleva clap y reqwest, con futuros consumidores poller y WASM pegados al motor.](assets/v03-diagram.webp)

Conecta `probe` al brazo de match, córrelo contra una URL viva, y la era del fixture termina:

```bash
cargo run -- probe https://www.rust-lang.org
```

```text
https://www.rust-lang.org -> 200 OK in 557 ms
```

Ese número es de verdad, y no se va a repetir. Mi propia primera corrida imprimió 557 ms, la siguiente 208 ms, siendo lo que son la reutilización de conexión y el caching de DNS. La latencia es clima, no arquitectura. Lo que arma la pregunta que la próxima sección responde con un cronómetro: si los números tiemblan, ¿cómo haces una afirmación de rendimiento, siquiera?

### Dos binarios, dos personalidades

Todo lo que construiste hasta ahora pasó por `cargo build`, lo que quiere decir el perfil dev: compilaciones rápidas, código lento, fallas ruidosas. `cargo build --release` produce un binario distinto del mismo código fuente, y las diferencias no son cosméticas. Las optimizaciones pasan de esencialmente ninguna a completas. Las aserciones de debug se apagan. Y el overflow de enteros deja de dar panic y empieza a dar wrap, el cambio exacto de personalidad que conociste en m04-l2 como regla; hoy le queda pegado un número de lab.

¿Qué tan distinto? Cronometré la pasada de latency-stats del motor, el recorrido de suma con checked_add de m04-l2, sobre cinco millones de muestras sintéticas de latencia, veinte pasadas, los dos perfiles, en mi propia máquina:

| perfil | 20 pasadas sobre 5M muestras | tamaño del binario |
|---|---|---|
| dev (`cargo build`) | 983 ms | 18M |
| release (`cargo build --release`) | 67 ms | 6.4M |

Quince veces más rápido, y el binario se encogió a como un tercio (el debug info es pesado). Tus números van a diferir, y eso es en parte el punto: el lab te hace producir tu propio par, porque la relación es la lección durable, no mis dígitos.

![Las barras muestran un build de release corriendo la misma carga unas quince veces más rápido que el build dev mientras produce un binario más chico.](assets/v04-chart.webp)

Dos consecuencias, las dos obligatorias. Primera: cualquier afirmación de rendimiento hecha sobre un binario debug es una mentira. No exagerada, una mentira, errada por un orden de magnitud. Mide `--release`, siempre; los propios números de rendimiento de este curso siguen esa regla. Segunda, más sutil: los perfiles no se ponen de acuerdo sobre la aritmética. Los builds dev dan panic con el overflow de enteros, los builds release dan wrap en silencio, así que una suma `u64` que se cae con honestidad en tu laptop puede entregar basura desde CI. Por esto exactamente m04-l2 te hizo escribir `checked_add` en vez de `+`: tu aritmética devuelve `Err(Overflow)` en los dos perfiles, y el cambio de personalidad no te puede tocar. La disciplina nunca fue sobre estilo. Fue sobre hacer que los dos binarios digan la verdad.

![Columnas lado a lado contrastan los perfiles dev y release mientras un pie nota que la aritmética verificada se comporta igual en los dos.](assets/v05-comparison.webp)

El ritmo para interiorizar: desarrolla en dev, mide y entrega en release. Las perillas detrás de las personalidades (`opt-level`, `debug-assertions`, `overflow-checks`) viven en secciones `[profile]` de Cargo.toml y se pueden sobrescribir por proyecto; saber que existen alcanza para este curso.

### Entrégalo a un desconocido

El último movimiento es corto porque la plataforma no es nueva. Tu `.github/workflows/pulse.yml` viene acumulando jobs desde M1: la sonda de TS, la barrera de typecheck, el job `rust` de m04-l3 corriendo test, clippy y fmt. Hoy aprende a repartir binarios. Sin plataforma nueva, un job nuevo.

Tres conceptos, glosados una vez. Un **GitHub Release** es un objeto de primera clase pegado a un tag de git: un título, notas y archivos descargables. Un **release asset** es uno de esos archivos, y se sirve a cualquiera que tenga la URL, sin cuenta, sin toolchain, sin clone. Y un **tag push** (`git tag v0.1.0 && git push origin v0.1.0`) es el evento que va a disparar el nuestro, que es el contrato convencional: los merges a main corren barreras, los tags cortan releases. Las piezas que ya tienes cubren el resto: el workflow necesita `permissions: contents: write` por la misma razón que lo necesitó tu job de commit-back en m01-l3, y la imagen del runner ya trae tanto un toolchain estable de Rust (el job de m04-l3 ya se apoya en él) como la CLI `gh`, que puede crear un release y adjuntar archivos en un comando.

![Un tag pusheado fluye por la barrera de pruebas y el build de release hasta un asset descargable que un desconocido corre en una máquina sin el código fuente.](assets/v06-flowchart.webp)

Honestidad sobre lo que esto entrega: un binario de un solo target, x86_64 Linux, construido en el runner. La distribución de verdad le crece matrices de targets, firma de macOS y Windows, y checksums; esos están nombrados acá y enseñados en ninguna parte de este curso, porque un solo target alcanza para cobrar la afirmación que importa. Y la afirmación vale decirla sin rodeos: cargo convierte "funciona en mi máquina" en un archivo que le puedes pasar a un desconocido. El tier del intérprete nunca pudo decir eso. Tu flota de TS se entrega como código fuente más un lockfile más una versión de Node más una etapa de instalación; esto se entrega como un archivo que ya ES el programa. Vas a construir el job tú mismo en el challenge. No antes.

**Profundiza (el 20%).** esta lección enseñó clap a nivel de derive, un fetch blocking y un pipeline de release de un solo target, que cubre el trabajo diario de CLI. Las capas de abajo quedan como bookmarks: el propio tutorial de derive de clap recorre cada atributo que soporta el derive, incluida la API builder debajo, en [https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html), y los docs del módulo blocking de reqwest cubren las opciones de cliente que nos salteamos, timeouts, headers, política de redirect, en [https://docs.rs/reqwest/latest/reqwest/blocking/index.html](https://docs.rs/reqwest/latest/reqwest/blocking/index.html). Las dos URLs verificadas en vivo el 2026-09-02. El middleware, el tuning del connection pool y la compilación cruzada también quedan como bookmarks. El lab no necesita ninguna.

## Lab: la CLI se gana su nombre

Trabaja desde la raíz del workspace `pulse-rs`. Estimados 60 a 75 minutos.

1. **La interfaz (trabajada, casi lista).** Si hiciste la apertura, tienes clap conectado con los subcomandos `probe` y `report` y la flag `--timeout`. Haz checkpoint de los tres comportamientos gratis, un comando cada uno:

   ```bash
   cargo run -- --help                 # prints the derived help
   cargo run -- probe                  # refuses with the missing-URL error
   cargo run -- probe x --timeout abc  # rejects the value by name
   ```

   Tres comportamientos, cero líneas de tu código de manejo.

2. **La taxonomía gana una variante (guiada).** En `pulse-engine`, agrega `Unreachable { reason: String }` a `ProbeError` con un mensaje `#[error(...)]` como el mío de arriba. El compilador no te va a forzar a actualizar los matches viejos a menos que hagas match exhaustivo en alguna parte; revisa cualquier `match` sobre `ProbeError` que escribiste en M4 y extiéndelo a propósito, no con un comodín.

3. **El fetch (guiado, tú escribes el puente).** Agrega la dependencia de reqwest, después escribe `probe` desde mi versión trabajada pero deja afuera los dos closures `map_err` e intenta compilar. Lee el error resultante completo: nombra `reqwest::Error` y el tipo de error de tu `Result`, y leerlo de punta a punta es la misma disciplina que E0382 te ejercitó en m04-l1, información, no obstrucción. Ahora escribe los dos puentes tú mismo. Checkpoint:

   ```bash
   cargo run -- probe https://www.rust-lang.org
   ```

   Eso imprime un estado de verdad y una latencia de verdad. Después sondea una URL que no pueda resolverse y confirma que recibes tu propio mensaje `Unreachable`, no un panic.

4. **El segundo subcomando (tuyo).** Implementa `report` para que la CLI de Rust refleje la forma de la CLI `pulse` de TS: debería correr la pasada de latency-stats del motor (el recorrido con checked_add de m04-l2; si el tuyo lleva otro nombre, quédatelo) sobre un set de fixtures generado e imprimir la media, el máximo y el conteo de muestras. Los nombres que necesitas, `total_latency` y `LatencyMs`, salen directo de `pulse_engine::`, los dos en la lista de re-exports que m05-l2 curó para exactamente este consumidor. El mío genera cinco millones de muestras con un scrambler de multiplicar-y-sumar con semilla para que las corridas sean comparables, e imprime una línea: `mean=479 ms, max=929 ms over 5000000 samples; 20 passes took 983 ms`. Veinte pasadas existen puramente para hacer medible el próximo paso. Sin apoyo para este, y para ser honesto sobre lo que quiere decir "todo lo que necesita": la pasada de stats ya está en tu motor, y el generador de fixtures es tu decisión, porque nada en el curso enseñó PRNGs y nada acá requiere uno. Cualquier fuente determinista pasa: un scrambler con semilla si te gusta presumir, o simplemente un slice chico de latencias ciclado hasta cinco millones (`[212u64, 487, 930, 479].iter().cycle().take(5_000_000)` alcanza). Tu media y tu máximo impresos van a reflejar tu generador, no el mío; el entregable que califica el paso 5 es la relación entre dev y release, que cualquiera de estos produce.

5. **Mide las dos personalidades (tuyo).** Construye los dos perfiles y corre el mismo report:

   ```bash
   cargo build --workspace
   cargo build --release --workspace
   ./target/debug/pulse-cli report
   ./target/release/pulse-cli report
   ls -lh target/debug/pulse-cli target/release/pulse-cli
   ```

   Anota los cuatro números: los dos tiempos, los dos tamaños. Ese par escrito es un entregable del lab, y la vara de aceptación es honesta: tu relación no va a ser mi 15x, pero un build dev que no es varias veces más lento que release quiere decir que tu report todavía no está haciendo trabajo de verdad.

6. **Mira cómo voltea la personalidad del overflow (2 minutos).** Mete este archivo como `crates/pulse-cli/src/bin/overflow_demo.rs` (cualquier archivo en `src/bin/` se vuelve su propio binario, una convención de cargo que vale conocer):

   ```rust
   fn main() {
       let start: u8 = std::env::args()
           .nth(1)
           .and_then(|s| s.parse().ok())
           .unwrap_or(250);
       let mut v = start;
       for _ in 0..10 {
           v += 1;
       }
       println!("{v}");
   }
   ```

   `cargo run --bin overflow_demo` da panic con `attempt to add with overflow`. `cargo run --release --bin overflow_demo` imprime `4`, en silencio, código de salida cero. Mismo código fuente, 250 más 10 dieron wrap alrededor de un `u8`. Borra el demo después de que te haya perturbado como se debe, y nota que tu subcomando report es inmune: sus sumas pasan por `checked_add`.

7. **Verde antes de entregar.** Corre `cargo fmt` primero, después el triple local (`cargo test && cargo clippy -- -D warnings && cargo fmt --check`); el código de toda una lección escrito a mano casi siempre carga un salto de línea o dos que rustfmt quiere de vuelta, y enterarte localmente cuesta segundos donde enterarte en CI cuesta un push. Después haz push del branch y confirma que el job rust de m04-l3 pasa con el código nuevo: test, clippy, fmt, todo verde. El job de release que estás por escribir va a quedar aguas abajo de esta barrera, que es el punto de haber construido la barrera primero.

## Challenge

Totalmente sin guía, y es el chequeo intermedio de este peldaño. Extiende `pulse.yml` para que un tag push publique tu binario:

- El workflow ahora se dispara con `push` a `main` y con el calendario. Haz que también se dispare con tags con forma `v*`, y mantén el job de commit-back de probe fuera de las ejecuciones de tag (un checkout de tag no es un branch; un push desde ahí falla. La barrera es una condición `if:` a nivel de job, nueva justo acá; los docs de expresiones de GitHub tienen la sintaxis, y el job de referencia de abajo lo muestra, después de tu intento, no antes).
- Agrega un job `release` que corra solo con refs de tag, que haga `needs` del job rust, construya `--release`, y use `gh release create` para publicar un Release con el binario adjunto como `pulse-cli-linux-x86_64`. Todo lo requerido está nombrado en la sección de entrega: el permiso, la variable de entorno del token (`GH_TOKEN: ${{ github.token }}`), el `gh` preinstalado, y el working directory de tu workspace.
- Haz tag `v0.1.0`, haz push del tag, mira el pipeline, y después haz la prueba de aceptación de verdad: descarga el asset en una máquina que nunca vio tu código fuente y corre `./pulse-cli-linux-x86_64 probe https://www.rust-lang.org`. Sirve la máquina Linux de un amigo, o cualquier VM de Linux, o WSL. Si estás en macOS y Docker ya está instalado de casualidad, este one-liner-en-un-contenedor funciona con cero conocimiento de Docker (desmitificamos todo eso el módulo que viene): `docker run --rm -it ubuntu:24.04 bash`, y adentro, `apt-get update && apt-get install -y curl ca-certificates`, descarga la URL del asset con curl, `chmod +x`, corre. Apple Silicon necesita `--platform linux/amd64` en el comando run. El asset es solo para Linux y eso se dice con honestidad, sin disculparse: un solo target, límites enseñados.

Aceptación: el comando de verificación en verde, o sea que

```bash
cargo run --release -- probe https://www.rust-lang.org
```

imprime un estado y una latencia de tu binario; las dos mediciones de perfil anotadas del lab; y el Release asset descargado imprimiendo la misma forma de línea de sonda en una máquina limpia. Cuando el tuyo funcione, compara contra mi job de abajo. Después, no antes; el diff review es donde está el aprendizaje, y una rep sin guía que espiaste es una rep guiada con pasos extra.

```yaml
on:
  push:
    branches: [main]
    tags: ["v*"]

# ...schedule and existing jobs unchanged; probe job gains:
#   if: github.ref == 'refs/heads/main' || github.event_name == 'schedule'

  release:
    if: startsWith(github.ref, 'refs/tags/v')
    needs: rust
    runs-on: ubuntu-latest
    permissions:
      contents: write
    defaults:
      run:
        working-directory: pulse-rs
    steps:
      - uses: actions/checkout@v7
      - run: cargo build --release --workspace
      - name: Attach the binary to a GitHub Release
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          cp target/release/pulse-cli pulse-cli-linux-x86_64
          gh release create "$GITHUB_REF_NAME" pulse-cli-linux-x86_64 --title "$GITHUB_REF_NAME" --generate-notes
```

## Las puertas que dejamos cerradas

Esto cierra el tier de Rust, así que te paso el mapa de lo que a propósito no enseñamos, porque saber dónde está una puerta le gana a pretender que no existe. Tres puertas: los tiempos de vida más allá de leerlos, `unsafe`, y escribir macros (más los internals de async, que m06 va a nombrar otra vez). Las dos primeras viven detrás del Rustonomicon, el libro oficial del Rust de artes oscuras, cuya propia advertencia de apertura voy a citar textualmente, verificada en la página el 2026-09-01: "Si deseas una carrera larga y feliz escribiendo programas en Rust, deberías dar la vuelta ahora y olvidar que alguna vez viste este libro. No es necesario". Que la documentación oficial te diga que no lo leas es todo el argumento 80/20 de este tier, hecho por los propios mantenedores del lenguaje. Vive en [https://doc.rust-lang.org/nomicon/](https://doc.rust-lang.org/nomicon/) para el día en que de verdad lo necesites, y ese día no es prerrequisito de nada de lo que quieras hacer después. La tercera puerta, escribir macros, vive en otro lado, y sorprende a la gente que asume que todo el Rust avanzado se esconde en un solo libro: el capítulo de macros de The Rust Programming Language cubre `macro_rules!` y un primer vistazo a los macros procedurales, y The Little Book of Rust Macros es donde viven los patrones profundos. El Rustonomicon no tiene ningún capítulo de macros; ya usaste macros del lado del consumidor cada vez que escribiste `println!`, y consumir es el único lado que este curso necesita.

![Un mapa muestra al centro las habilidades que enseñó el tier, al lado recursos de ejercicios gratis, y puertas cerradas etiquetadas con dónde viven los temas más profundos de Rust.](assets/v07-diagram.webp)

Lo que de verdad deberías hacer después es hacer ejercicios, no descender. Rustlings (`cargo install rustlings`, después `rustlings init` y `rustlings`) sigue siendo el patio de ejercicios, reps chicas de arreglar-el-código mantenidas por el propio proyecto Rust. Y Comprehensive Rust, en [https://google.github.io/comprehensive-rust/](https://google.github.io/comprehensive-rust/), es el curso que el equipo de Android de Google usa para incorporar a Rust a ingenieros que trabajan, gratis, mantenido activamente (su repo recibió commits el mismísimo día en que corrió la investigación de este curso, 2026-09-01). El curso que una empresa de un billón de dólares usa para reentrenar ingenieros de C++ no te cuesta nada; esa es la vara de recursos gratis en la que se han estado apoyando los bookmarks de este tier todo este tiempo. Una opción paga se gana una mención porque nombró la audiencia exacta de este curso tres años antes que nosotros: "Rust for TypeScript Developers" de ThePrimeagen, 5h19m, publicado el 2023-04-25, paga con un preview gratis. Declarada una vez, acá, al lado del canon gratis.

¿Y por dónde va Rust on-chain? Por una puerta distinta a la del Nomicon, y esto importa: escribir programas de Solana no requiere turismo de unsafe ni brujería de tiempos de vida. Eso sí, lee los letreros con honestidad. El curso Mastering Anchor de este catálogo es la parte honda, no la rampa de entrada: su propia vara es que ya entregues programas de Anchor y quieras el delta de V2. Lo que te compra tu nivel de salida acá es el nivel de lectura para el código de ese mundo, structs, enums, traits, `Result`, pensamiento con forma de serde; la distancia que queda es experiencia de entrega, no vocabulario. Y los conceptos de cuenta y transacción que están debajo de escribir programas, lo que un programa realmente recibe y por qué, pertenecen al curso de evolución de Bitcoin a Solana. Nivel de lectura de acá, conceptos de allá, reps por tu cuenta: esa es la ruta honesta on-chain.

## Checkpoint

Lo que ahora puedes hacer, concretamente: derivar una CLI tipada cuya ayuda, errores de parseo y defaults se generan desde el tipo; hacer requests HTTP de verdad desde Rust y tender un puente desde errores ajenos hacia tu propia taxonomía sin contaminar un crate puro; medir el hueco entre dev y release con tus propios números y decir con precisión qué cambió entre los perfiles; y publicar por CI un binario que corre en una máquina que nunca vio tu código fuente. La mitad Rust de la estación ya no es una librería que solo las pruebas pueden amar. Es una herramienta.

La recuperación de 30 segundos antes de cerrar la pestaña, de memoria: nombra las tres puertas de Rust profundo que este tier dejó cerradas y dónde vive cada una. (Los tiempos de vida más allá de leerlos y unsafe, las dos detrás del Rustonomicon, cuya propia advertencia te dijo que dieras la vuelta; escribir macros, detrás del capítulo de macros de TRPL y The Little Book of Rust Macros. Y el Rust on-chain no está detrás de ninguna de ellas: esa es la puerta del curso de Anchor.)

Un pedido mientras el tier está fresco: de todo lo de M4 y M5, dime en el feedback qué único concepto te costó más tiempo de reloj, y si el pago ya llegó hoy o sigue siendo un pagaré. Toda la apuesta del tier es que el 20% que enseñamos cubre tu 80% diario, y tu reporte de fricción es el único instrumento que mide si la apuesta pagó.

Tu binario de verdad sondea, y después sale. Una estación necesita un latido que no lo haga. El módulo que viene el mismo crate del motor va adentro de un poller tokio de larga vida con un endpoint `/status`, el fetch blocking que escribiste hoy le crece su delta async bajo presión de concurrencia real, y todo entero va a una caja. La próxima vez que arranques el proceso, cuenta con dejarlo corriendo.
