# El pago: Rust en el mismo edge

## Resumen

m07-l1 entregó el worker de TS: la lógica pura de pulse-core corriendo en un cron en cientos de ciudades, el último estado conocido estacionado en KV, el secreto manejado como un adulto, y el getHealth de Solana ya sentado en la lista de sondas. Ese fue el motor uno. La estación tiene dos. Hoy el motor de Rust recibe el mismo tratamiento: compilas `pulse-engine` a WebAssembly, lo envuelves en un proyecto de workers-rs, y aterrizas la segunda URL de workers.dev en vivo de la estación con el mismo comando de deploy exacto que corriste ayer. El contrato de repliegue, en voz alta, con su precio dicho con honestidad: la plantilla generada llega funcionando, el handler de verdad llega completo en la página, y tus TODOs con las manos son el cableado alrededor, la dependencia del motor, el brazo classify de la CLI, y el par get y put de KV; la ruta de la tabla de transiciones del cierre es el único build totalmente solo de la lección. Más delgado que algunos labs, a propósito, porque la tesis es que el port en sí es chico. Hiciste este deploy exacto hace una lección. El delta es el lenguaje. Que el delta sea SOLO el lenguaje es toda la lección.

## La apuesta paga

Allá en M4 te hice mantener puro el motor de Rust: el clasificador, la máquina de estados, los tipos de serde, todos ellos funciones de valores a valores, ningún socket ni file handle en ninguna parte del crate. En m05-l2 partimos esa pureza en su propio crate del workspace y yo enumeré qué puede alimentar un núcleo puro, terminando con "incluso un worker WASM en el tier de deploy". Ya es más tarde en el tier de deploy. Dos comandos empiezan la cobranza:

```bash
rustup target add wasm32-unknown-unknown
cargo install cargo-generate
```

El primero le enseña a tu toolchain existente a emitir WebAssembly en vez de código máquina nativo. El mismo rustc, backend nuevo, un build más de la librería estándar. El segundo instala `cargo-generate`, una herramienta de scaffolding que estampa proyectos a partir de repos de plantilla de Git, y existe en esta lección porque el camino oficial de Rust de Cloudflare empieza con ella:

```bash
cargo generate cloudflare/workers-rs
```

Elige la plantilla `hello-world` cuando pregunte, nombra el proyecto `pulse-edge-rs`, y córrelo en la raíz del repo de tu estación para que aterrice al lado de `pulse-rs/` y del worker de TS. La plantilla también hace una pregunta sobre la que este párrafo, si no, te dejaría adivinando, "Enable panic=unwind and abort recovery?": toma el default, false. No lo despliegues todavía. Primero, diez minutos de entender qué acabas de poner como target, porque `wasm32-unknown-unknown` es el nombre de target más honesto de todo el toolchain.

### Un OS llamado unknown

Un target de compilación de Rust nombra tres cosas: arquitectura, vendor, sistema operativo. `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`. Lee el de WASM igual: arquitectura WASM de 32 bits, vendor unknown, sistema operativo unknown. Un OS unknown no es un placeholder esperando un valor. Es la especificación. El módulo compilado no puede asumir ningún thread, ningún socket, ningún sistema de archivos, ningún reloj que no haya pedido, ninguna variable de entorno. Es computación pura en una caja sellada, y cualquier cosa del mundo de afuera tiene que pasarse hacia adentro por una interfaz que el host elige exponer.

Eso debería sonarte conocido, porque es la forma exacta del crate de tu motor. `classify_latency` toma una latencia y devuelve un `Verdict`. `next_state` toma un estado, un desenlace y el conteo de fallas consecutivas, y devuelve un estado. Los tipos de serde convierten bytes en valores y de vuelta. Nada de eso abrió nunca una conexión; la CLI, el poller y el worker de TS hicieron todo el I/O y le dieron valores al motor. En el edge, el host es workerd, el mismo runtime de m07-l1, y lo que le pasa a la caja es el contrato de plataforma que ya conoces: fetch, bindings de KV, triggers de cron, que le llegan a Rust por una capa de pegamento de JS que las herramientas te generan.

![El crate puro del motor cruza la costura hacia WebAssembly mientras la CLI y el poller se quedan nativos, con la plataforma proveyendo fetch, KV y cron del lado de WASM.](assets/v01-diagram.webp)

### El ejercicio de rechazo

Las afirmaciones sobre lo que "no se puede portar" son baratas, así que compremos el error de verdad. Desde adentro del crate de tu motor, primero haz commit, después sabotéalo a propósito:

```bash
cargo add tokio --features full
cargo build --target wasm32-unknown-unknown
```

El build muere adentro de mio, la capa de event loop del OS de tokio, y el error es inusualmente cortés sobre por qué:

```text
error: This wasm target is unsupported by mio. If using Tokio, disable the net feature.
  --> mio-1.2.2/src/lib.rs:44:1
   |
44 | compile_error!("This wasm target is unsupported by mio. If using Tokio, disable the net feature.");
```

Corrí este sabotaje exacto mientras escribía la lección; cada bloque de error de esta página está pegado desde mi terminal, no escrito de memoria. Y mira lo que el error está diciendo de verdad: el trabajo de mio es envolver epoll y kqueue, las facilidades del OS para esperar en sockets. En un target cuyo OS se llama unknown no hay nada que envolver, así que el crate se rehúsa en tiempo de compilación. reqwest falla de manera más solapada, y vale la pena saber cómo se escabulle: el crate en sí compila en wasm32 porque carga un backend de navegador, pero el módulo `blocking` que tu CLI usa desde m05-l3 se compila condicionalmente afuera, así que en el momento en que tu código lo toca te llevas `error[E0433]: could not find 'blocking' in 'reqwest'`. La misma ley, distinto mensajero: el arsenal async no está prohibido en WASM por política. Está anclado a lo nativo por las llamadas al OS que tiene debajo, y el compilador hace cumplir el ancla. Ahora hazle `git checkout` al Cargo.toml Y al Cargo.lock, porque `cargo add` reescribió los dos y el build fallido jaló los pins de tokio adentro del lockfile, y deja que el motor vuelva a ser portable.

Esta es la costura de núcleo puro y cascarón de I/O completando su arco. m07-l1 lo demostró en TypeScript, donde quien lo hacía cumplir era el runtime de workerd rehusándose a arrancar ante una lectura del sistema de archivos: el módulo con shim, el sistema operativo ausente. Rust lo demuestra en tiempo de compilación, antes de que se entregue nada. Dos lenguajes, una ley: la lógica que nunca tocó el OS va a cualquier parte; el I/O es del cascarón, y cada host recibe su propio cascarón.

![Una línea de tiempo desde las primeras decisiones de pureza en los módulos tres y cuatro hasta el port a WebAssembly de hoy, mostrando que el deploy fue planeado y no suerte.](assets/v02-timeline.webp)

### Leer la plantilla que acabas de generar

Abre `pulse-edge-rs/`. cargo-generate te dejó un proyecto real y desplegable, y el hábito que este curso aplica a cada scaffold aplica acá: lee el artefacto antes de correrlo. `Cargo.toml` primero:

```toml
[package]
name = "pulse-edge-rs"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
worker = { version = "0.8" }
worker-macros = { version = "0.8" }
```

Tres observaciones campo por campo. `crate-type = ["cdylib"]` le dice a cargo que produzca una librería dinámica al estilo C en vez de un binario, que es la forma que wasm-bindgen sabe consumir. El crate `worker` es el SDK de Rust para la plataforma Workers: tus tipos para `Request`, `Response`, `Env`, y el binding de KV. Y el pin dice `"0.8"`, no un latest pelado, porque worker es pre-1.0. Di la regla de m05-l2 en voz alta: bajo semver, una minor 0.x tiene permitido romperte como lo haría una 2.0 en otro lado, así que el pin sostiene la línea 0.8 a propósito y lees las notas de release antes de elegir 0.9. Mientras escribo, sondeado en crates.io el 2026-09-02, la línea está en 0.8.5, publicada el 2026-06-12. Nota de frescura: vuelve a chequear ese dígito cuando hagas este lab; un SDK pre-1.0 es exactamente el tipo de dependencia cuya minor actual importa. (La plantilla también fija su propia edition en 2021 mientras tu workspace corre 2024; esa es la revisión de m05-l2 jugándose en un solo manifest, el ecosistema subiéndose tarde mientras tus propios crates del lado del host van en la edition vigente bajo la excepción fechada de esa lección, y su regla cubre esto también: la edition de la plantilla es el contrato de la plantilla. Editions distintas a través de una dependencia por path están bien; las editions son por crate, que es precisamente por qué pueden existir siquiera.)

El `src/lib.rs` de la plantilla tiene ocho líneas y ya puedes leer cada una de ellas:

```rust
use worker::*;

#[event(fetch)]
async fn fetch(
    _req: Request,
    _env: Env,
    _ctx: Context,
) -> Result<Response> {
    Response::ok("Hello World!")
}
```

`#[event(fetch)]` es la macro del crate worker que marca esta función como el handler de fetch, el mismo slot del contrato que tu worker de TS llenó con un método `fetch` exportado. Hay un gemelo `#[event(scheduled)]` para cron, que toma un `ScheduledEvent` en vez de un `Request`, y vale la pena saber que este worker podría correr en un calendario con una función y una línea de wrangler.toml. A propósito no lo hacemos: el worker de TS ya corre por cron, la topología de hub del capstone quiere que este responda a demanda, y darles a los dos workers el mismo trabajo te enseñaría copy-paste, no arquitectura. Sí, la función es `async`, y eso vale un beat dado el ejercicio de rechazo que acabas de correr: el Rust async funciona bien en un worker. Lo que falta en el target unknown es el I/O respaldado por el OS de tokio, no la feature de lenguaje async. workerd maneja estos futures él mismo, a través del event loop de JS del que ya es dueño.

Ahora `wrangler.toml`, donde se esconde el truco:

```toml
name = "pulse-edge-rs"
main = "build/index.js"
compatibility_date = "2026-09-02"

[build]
command = "cargo install -q \"worker-build@^0.8\" && worker-build --release"
```

`main` apunta a un archivo JavaScript que todavía no existe. El bloque `[build]` es el porqué: cada `wrangler deploy` corre primero `worker-build`, que compila tu crate para wasm32-unknown-unknown, corre wasm-bindgen para generar el pegamento de JS que traslada los tipos a través de la frontera, corre wasm-opt para achicar el módulo, y emite `build/index.js` como el shim de entrada. Tres herramientas apiladas bajo un solo comando. Las nombro para que la salida del build no se lea como ruido, y eso es todo lo que hacemos con ellas; worker-build, wasm-bindgen y wasm-opt son plomería que este curso no enseña. Una consecuencia práctica igual vale la pena guardarla: cuando un error de build mencione pegamento de wasm-bindgen y se vea exótico, corre `cargo check` en tu workspace nativo primero. Los errores de tipos de verdad salen a la superficie ahí con spans normales, y depuras Rust en Rust en vez de a través del pegamento.

![Un solo comando de deploy se abre en una compilación de Rust, generación de pegamento y optimización antes de que wrangler suba el resultado e imprima una URL en vivo.](assets/v03-flowchart.webp)

### KV a través de tipos de Rust

Una pieza más del crate worker antes de la síntesis, porque el lab se apoya en ella: el binding de KV. Todo lo que aprendiste ayer sobre KV sigue en pie y no se vuelve a enseñar acá: es el almacén clave-valor persistente de la plataforma, eventualmente consistente, limitado en escrituras en el tier gratis, y la razón por la que un worker sin estado puede recordar algo siquiera. Lo que cambia en Rust es puramente la historia de los tipos, y la historia de los tipos es buena. `env.kv("PULSE_STATUS")` busca el binding por nombre y te da un `KvStore`. Las lecturas vuelven a través de un builder que termina en `.json::<T>().await?`, lo que quiere decir que KV te da un `Option<T>` de tu propio tipo del motor, ya deserializado, o `None` cuando la clave nunca se escribió. Las escrituras van al revés: serializa con `serde_json::to_string`, después `kv.put(key, value)?.execute().await?`, donde el `.execute()` es el builder disparando de verdad (olvidarlo es el bug clásico de la primera semana, porque la línea sin él compila bien y no hace nada). Serde parado en los dos extremos de la tubería es el punto a notar. Las mismas líneas de derive que alimentaron la salida JSON de la CLI y el endpoint `/status` del poller ahora definen el formato de cable de tu almacenamiento en el edge, en un tercer runtime, sin código nuevo. Cuando la gente dice que el ecosistema de Rust converge fuerte en serde, esto es lo que compra converger.

### Un contrato, dos lenguajes

Acá va la síntesis hacia la que ha venido caminando todo el módulo. Ayer desplegaste TypeScript con `npx wrangler deploy`. En el lab de abajo vas a desplegar Rust con `npx wrangler deploy`. No un comando análogo. El mismo comando, y la plataforma no puede notar la diferencia, porque lo que la plataforma define es un contrato: una forma de handler de fetch, una forma de handler scheduled, bindings declarados en la config, un verbo de deploy. Cualquier cosa que satisfaga el contrato es un worker. Hazte la pregunta discriminante: ¿qué, exactamente, necesitaba saber wrangler sobre tu lenguaje para correr el deploy de ayer? Nada. Corrió un comando de build desde un archivo de config y subió lo que salió, y va a hacer precisamente eso otra vez hoy. Así que el artefacto que estás entregando a Cloudflare nunca fue de verdad "una app de TypeScript" o "una app de Rust"; es una implementación de contrato, y el lenguaje es un proveedor parado detrás de ella. Esa es la lección durable para llevarte de este módulo, porque te la vas a volver a encontrar en todos lados donde vivan las plataformas: al contrato de contenedor de M6 no le importó que la caja tuviera Rust adentro, solo que un proceso escuchara en un puerto; al contrato JSON-RPC del próximo módulo no le va a importar quién escribió el request, solo que los bytes parseen. Cuando una plataforma define la costura, el lenguaje deja de ser una decisión arquitectónica y se vuelve una elección por componente, hecha sobre los méritos reales de cada componente.

Y dicho eso, la tabla de costos honesta, porque la síntesis corta para los dos lados. El camino de Rust apila tres herramientas de build bajo el deploy, se monta en un SDK pre-1.0 cuyas minors pueden romper entre sesiones, y renuncia a tokio y reqwest en la costura. Para un worker que es mayormente pegamento alrededor de fetch, el worker de TS que construiste ayer es el default de menor fricción, punto. Pagas el peaje de WASM cuando el valor es el núcleo de Rust compartido en sí: un clasificador, una máquina de estados, un conjunto de tipos de serde, ya probados, ya confiables para la CLI y el poller, ahora respondiendo desde el edge sin una reescritura. Para el motor de la estación ese trade-off vale la pena. La heurística para llevarte: cuenta las líneas que son tuyas. Si la sustancia del worker son llamadas a la plataforma con una pizca delgada de lógica, escríbelo en el lenguaje de casa de la plataforma y sigue; si la sustancia es un núcleo que mantienes, pruebas y en el que confías en otro lado en Rust, porta el núcleo y quédate con una sola implementación de la verdad. Saber de QUÉ lado de ese criterio cae un servicio dado es precisamente la habilidad que este curso está vendiendo.

![Una comparación de seis filas de los workers de TypeScript y Rust compartiendo un solo comando de deploy mientras difieren en herramientas de build, madurez del SDK, y cuándo gana cada elección.](assets/v04-comparison.webp)

**Profundiza (el 20%).** esta lección enseña el camino de workers-rs a profundidad de trabajo: el target, la plantilla, las macros de handler, KV desde Rust, y el deploy. El catálogo completo de bindings, las clases de Durable Object en Rust, los wrappers de send-safety, y el resto de la superficie del SDK viven en los docs de lenguaje Rust de Cloudflare para Workers: [https://developers.cloudflare.com/workers/languages/rust/](https://developers.cloudflare.com/workers/languages/rust/) (URL chequeada el 2026-09-02). Guárdalo como bookmark; el lab de abajo no necesita nada del material del bookmark.

### La barrera del tier: lo que salteamos, y dónde vive

M7 cierra el tier del edge, así que antes del lab, el mapa de la familia de plataformas sobre la que deliberadamente no construimos. Esto quiere ser un catálogo; lo voy a mantener en un mapa, unas pocas líneas honestas por producto, porque saber qué salteaste y por qué es una habilidad de verdad y fingir que la plataforma es solo Workers más KV sería una mentira por omisión.

**R2** es almacenamiento de objetos, con forma de S3 y compatible con la API de S3: archivos, imágenes, volcados de historial de sondas, cualquier cosa de tamaño blob. El tier gratis es genuinamente generoso, 10 GB-mes almacenados y egress gratis, y el egress gratis es todo el argumento de venta de R2 contra S3. Pero varios hilos de la comunidad de Cloudflare reportan que habilitar R2 requiere vincular un método de pago incluso para uso del tier gratis, y eso falla la regla de este curso de no-tarjeta. Así que R2 es una extensión opcional claramente etiquetada para quienes aprenden y eligen vincular una tarjeta, nunca un paso del camino central; KV carga nuestro estado. Si algún día sí vinculas una tarjeta, guardar el JSON crudo de cada corrida de sondas en R2 es el primer uso natural, y el crate worker le habla con el mismo patrón de binding que estás por usar para KV.

**D1** es SQL en el edge, SQLite abajo del capó, 5,000,000 de filas leídas por día gratis. Responde la pregunta que KV no puede: consultas. En el momento en que quieres "cada target que estuvo Degraded en la última hora", las búsquedas por clave dejan de ser la forma correcta y D1 es a donde te manda la plataforma, sin tarjeta exigida.

**Durable Objects** son coordinación con estado: un objeto de un solo thread con su propio almacenamiento, donde cada request para una clave dada se rutea a la misma instancia, que es como la plataforma hace "existe exactamente uno de estos" sin que tú corras un servidor. 100,000 requests por día gratis, y ahora están en el plan gratis, lo que no siempre fue cierto. Si la estación alguna vez necesitara un rate limiter por target o un contador en vivo que no pueda tener condiciones de carrera, esta es la herramienta.

**Queues** compran desacople async entre workers, productor de un lado y consumidor del otro, 10,000 operaciones por día gratis. El trabajo que hacen para una flota en el edge es el mismo trabajo que hace una fila de mensajes en cualquier lado: absorber una ráfaga ahora, procesarla con calma después.

**Cloudflare Containers**, GA el 2026-04-13, es la plataforma admitiendo en voz alta que algunas cargas de trabajo solo quieren una caja Linux: tus imágenes de Docker de M6, gestionadas, al lado de tus workers. Va a parecer la pieza faltante de la historia de este curso, y arquitectónicamente casi lo es. También es solo para Workers Paid, un prerrequisito de $5/mo, así que bajo la regla de no-tarjeta se queda como un letrero acá, no como un lab. Cuando tengas cinco dólares al mes y una razón, las imágenes de M6 que ya empujas a GHCR son exactamente lo que corre.

Ningún curso hermano de este catálogo es dueño de estos productos, así que este mapa más los docs oficiales son el traspaso honesto: la línea en negrita es la descripción del trabajo, y la documentación de la plataforma misma es a donde vas el día que la estación necesite uno.

![Cinco productos de Cloudflare listados con sus trabajos y asignaciones gratis, donde R2 y Containers cargan requisitos de pago que los dejan fuera del camino central de este curso.](assets/v05-table.webp)

## Lab: pulse-edge-rs

Este es el segundo ship al edge de la estación, el gemelo de Rust del worker de m07-l1. El handler de fetch acepta fixtures de sondas o las últimas muestras guardadas en KV, las corre por el MISMO clasificador y la MISMA máquina de estados que usan la CLI y el poller de Docker, y devuelve JSON clasificado desde el edge.

1. **Despliega primero el hello world.** Desde `pulse-edge-rs/`, antes de tocar nada:

   ```bash
   npx wrangler deploy
   ```

   La autenticación se hereda del login de navegador de m07-l1 (en una máquina nueva, `npx wrangler login` primero). wrangler mismo no se hereda, porque el cuidadoso pin v4 de ayer vive en `pulse-edge-ts/package.json` y este proyecto de cargo-generate no tiene package.json, así que el `npx wrangler` pelado de acá trae wrangler fresco; acepta el prompt de npx, o mantén el hábito de fijar con `npx wrangler@4 deploy`. De cualquier manera, la primera ejecución instala worker-build, compila la plantilla, e imprime tu segunda URL de workers.dev en vivo. Hazle curl, mira `Hello World!`, y aprecia lo que acaba de pasar: tu primer pipeline de Rust-a-WASM-a-edge funcionó antes de que lo entendieras, que es el orden correcto para los pipelines. Ahora hacemos que se gane la URL.

2. **Cablea el crate del motor.** Una pregunta de cableado que vale la pena resolver antes de cualquier código: ¿dependencia por path o dependencia por git? Path, y verifiqué la cadena entera antes de escribir esta página: un crate fuera del workspace puede depender de `pulse-engine` por path, cargo resuelve las suscripciones de dependencias `workspace = true` del motor contra el workspace propio del motor, y el resultado compila para wasm32-unknown-unknown limpio. Agrega las dependencias a `pulse-edge-rs/Cargo.toml`:

   ```toml
   [dependencies]
   worker = { version = "0.8" }
   worker-macros = { version = "0.8" }
   serde = { version = "1.0.229", features = ["derive"] }
   serde_json = "1.0.151"
   pulse-engine = { path = "../pulse-rs/crates/pulse-engine" }
   ```

   Después dale al motor el único módulo nuevo que los dos consumidores van a compartir. Los fixtures están por volverse un formato de cable entre lenguajes, así que van en el núcleo puro: crea `crates/pulse-engine/src/fixture.rs`:

   ```rust
   use crate::engine::{classify_latency, LatencyMs, Verdict};
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Serialize, Deserialize)]
   pub struct FixtureSample {
       pub name: String,
       pub latency_ms: u64,
   }

   #[derive(Debug, Serialize)]
   pub struct Classified {
       pub name: String,
       pub latency_ms: u64,
       pub verdict: Verdict,
   }

   pub fn classify_fixtures(samples: &[FixtureSample]) -> Vec<Classified> {
       samples
           .iter()
           .map(|s| Classified {
               name: s.name.clone(),
               latency_ms: s.latency_ms,
               verdict: classify_latency(LatencyMs(s.latency_ms)),
           })
           .collect()
   }
   ```

   Decláralo y re-expórtalo en el `lib.rs` del motor (`pub mod fixture;` más `pub use fixture::{Classified, FixtureSample, classify_fixtures};`), y haz exhaustiva la auditoría de derives mientras estás ahí adentro, porque el código dado del paso 4 depende de cada ítem: `Verdict` necesita `Serialize` (y un lugar en los re-exports de la raíz de lib.rs, ya que el worker va a nombrar `pulse_engine::Verdict` directo), y a `ProbeState` hay que agregarle `Deserialize` al lado del `Serialize` que levantó en m06-l1, porque KV está por hacerle ida y vuelta, Y todavía tiene que cargar el canon completo de m04-l3, `Debug, Clone, Copy, PartialEq, Eq`. El `Copy` en particular es estructural: el `StoredStatus` del paso 4 deriva `Clone, Copy` mientras guarda un `ProbeState`, que solo compila cuando el enum de estado es `Copy` él mismo, así que si tu lista de derives alguna vez se desvió, restáurala ahora en vez de encontrarte el error de derive adentro de código que esta página llamó completo. Si tus nombres de módulo o de campo se desviaron de los míos a lo largo de los módulos, quédate con los tuyos y adapta; la interfaz que importa son los nombres de función exportados y la forma del JSON.

3. **Dale a la CLI la otra mitad de la barrera.** La prueba de aceptación de esta lección es el mismo fixture produciendo veredictos idénticos desde Rust nativo y WASM en el edge, así que la CLI necesita un subcomando `classify`. En el enum de comandos de `pulse-cli`, una variante nueva y un brazo nuevo:

   ```rust
   /// Classify a JSON fixture set from stdin and print the verdicts as JSON
   Classify,
   ```

   ```rust
   Command::Classify => {
       use pulse_engine::{FixtureSample, classify_fixtures};
       let raw = std::io::read_to_string(std::io::stdin())?;
       let samples: Vec<FixtureSample> = serde_json::from_str(&raw)?;
       println!("{}", serde_json::to_string(&classify_fixtures(&samples))?);
   }
   ```

   Una línea de cableado antes de que ese brazo compile, porque la CLI nunca necesitó salida JSON hasta ahora: dale a `pulse-cli` la suscripción a serde_json del workspace en las dependencias de su `Cargo.toml`, `serde_json = { workspace = true }`. El `use` adentro del brazo trae los dos nombres del motor al scope; nada más cambia.

   Escribe un fixture compartido en la raíz del repo de la estación como `fixture.json`:

   ```json
   [{"name":"solana-rpc","latency_ms":180},{"name":"demo-api","latency_ms":740},{"name":"dead-host","latency_ms":4000}]
   ```

   Y córrelo desde el directorio del workspace `pulse-rs/`, dicho porque las dos mitades del comando dependen de eso: `-p` necesita el workspace de cargo como su cwd, y `../fixture.json` alcanza la raíz del repo desde exactamente un nivel más abajo:

   ```bash
   cd pulse-rs
   cargo run -p pulse-cli -- classify < ../fixture.json
   ```

   ```text
   [{"name":"solana-rpc","latency_ms":180,"verdict":"Up"},{"name":"demo-api","latency_ms":740,"verdict":"Degraded"},{"name":"dead-host","latency_ms":4000,"verdict":"Down"}]
   ```

   Esa línea es tu verdad de base nativa. El edge tiene que reproducirla exactamente.

![Tres fixtures con latencias fijas mapean a Up, Degraded y Down, y el worker del edge tiene que devolver la misma línea serializada que Rust nativo.](assets/v06-table.webp)

4. **Reemplaza la lógica de juguete.** Cambia el `src/lib.rs` de la plantilla por el handler de verdad. Quedan dos TODOs donde va el par de KV; todo lo demás está completo:

   ```rust
   use pulse_engine::{classify_fixtures, next_state, FixtureSample, ProbeState};
   use serde::{Deserialize, Serialize};
   use worker::*;

   #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
   struct StoredStatus {
       state: ProbeState,
       consecutive_failures: u32,
   }

   #[event(fetch)]
   async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
       let url = req.url()?;
       match url.path() {
           "/" => classify_handler(req, env).await,
           _ => Response::error("not found", 404),
       }
   }

   async fn classify_handler(mut req: Request, env: Env) -> Result<Response> {
       let kv = env.kv("PULSE_STATUS")?;
       let samples: Vec<FixtureSample> = if req.method() == Method::Post {
           req.json().await?
       } else {
           // TODO 1: read the "latest-samples" key from KV as Vec<FixtureSample>,
           // defaulting to an empty Vec when the key has never been written.
           Vec::new()
       };
       let classified = classify_fixtures(&samples);

       if req.method() == Method::Post {
           for c in &classified {
               let key = format!("status:{}", c.name);
               let prev: StoredStatus = kv
                   .get(&key)
                   .json()
                   .await?
                   .unwrap_or(StoredStatus { state: ProbeState::Pending, consecutive_failures: 0 });
               let ok = matches!(c.verdict, pulse_engine::Verdict::Up);
               let failures = if ok { 0 } else { prev.consecutive_failures + 1 };
               let next = next_state(prev.state, ok, failures);
               let stored = StoredStatus { state: next, consecutive_failures: failures };
               // TODO 2: write `stored` back to KV under `key`, serialized with serde_json.
           }
       }

       Response::from_json(&classified)
   }
   ```

   Lee la forma antes de llenar los huecos. POST quiere decir "acá hay muestras frescas, clasifícalas y avanza el estado por target". GET quiere decir "clasifica lo último que vio KV", y es deliberadamente una lectura pura: los veredictos se recomputan, pero el loop `for` que avanza el estado está detrás de la barrera del POST, así que mirar el worker no mueve nada y no escribe nada. Esa barrera es estructural dos veces. m04-l3 definió `consecutive_failures` como fallas de SONDA consecutivas, y una sonda pasa cuando llegan muestras, no cada vez que alguien mira; un loop sin barrera dejaría que tres GETs ociosos, o un crawler que anda paseando por la URL pública de workers.dev, caminaran un target Degraded hasta Down con cero datos nuevos de sondeo. Y cada put a KV gasta el presupuesto diario de escrituras que m07-l1 dimensionó con casi nada de margen; las lecturas no deben gastarlo. Los veredictos vienen de `classify_fixtures` y el camino POST avanza a través de `next_state`: las mismas dos funciones, la misma máquina de cuatro estados, los mismos umbrales que vienen respondiendo en la CLI desde M4 y en el poller desde M6. El worker no escribe ninguna lógica. Es un cascarón alrededor del motor, que ha sido la definición de un buen cascarón de este curso desde m03-l1.

5. **Completa el par de KV.** Esta es la pieza enseñada del lab, así que acá están las dos líneas, con el razonamiento. TODO 1:

   ```rust
   kv.get("latest-samples")
       .json::<Vec<FixtureSample>>()
       .await?
       .unwrap_or_default()
   ```

   Y el TODO 2, más un put extra al final de la rama POST para que GET tenga algo que leer la próxima vez (ponlo justo antes de `Response::from_json`, con barrera sobre el método al que ya le hiciste match):

   ```rust
   kv.put(&key, serde_json::to_string(&stored)?)?.execute().await?;
   ```

   ```rust
   if req.method() == Method::Post {
       kv.put("latest-samples", serde_json::to_string(&samples)?)?.execute().await?;
   }
   ```

   Fíjate en serde parado en los dos extremos de la tubería: `.json::<T>()` deserializa lo que KV guardó, `serde_json::to_string` serializa lo que pones de vuelta, y los tipos que cruzan la tubería son los propios del motor. Este es el mismo KV que tu worker de TS usó conceptualmente, pero un namespace separado en la práctica, así que crea uno y dale un binding:

   ```bash
   npx wrangler kv namespace create PULSE_STATUS
   ```

   Pega en `wrangler.toml` el bloque de id que imprime el comando:

   ```toml
   [[kv_namespaces]]
   binding = "PULSE_STATUS"
   id = "<the-id-wrangler-printed>"
   ```

   El nombre del binding es lo que `env.kv("PULSE_STATUS")` busca en tiempo de ejecución; el id es cuál namespace real responde. Dos workers, dos namespaces, cero estado compartido: según la topología de hub del capstone, este worker sondea independientemente y nunca consume al poller.

![Un request POST fluye a través de la deserialización, el clasificador puro y la máquina de estados, y lecturas y escrituras de KV, mientras que GET repite las últimas muestras guardadas por el mismo camino.](assets/v07-flowchart.webp)

6. **Córrelo local antes de entregarlo.** El loop de dev que tenías para el worker de TS existe para Rust también, el mismo comando:

   ```bash
   npx wrangler dev
   ```

   wrangler corre el pipeline de worker-build y sirve tu worker en `localhost:8787`, con el binding de KV apuntado a una simulación local así que nada de lo que hagas POST acá toca el namespace real. Aliméntalo con el fixture y échale un ojo a los veredictos:

   ```bash
   curl -s -X POST -H "content-type: application/json" --data @../fixture.json http://localhost:8787/
   ```

   El loop es más lento que el de TS, porque cada cambio de código repite una compilación de Rust antes del reload, y ese es un costo honesto del peaje que elegiste. Sigue siendo un loop de compilar y probar en tu propia máquina, que le gana a depurar a través de un deploy todas y cada una de las veces.

7. **Despliega y corre la barrera.** El mismo verbo que ayer, después las dos líneas que cierran el módulo:

   ```bash
   npx wrangler deploy
   ```

   ```bash
   cargo run -p pulse-cli -- classify < ../fixture.json
   curl -s -X POST -H "content-type: application/json" --data @../fixture.json https://pulse-edge-rs.<your-subdomain>.workers.dev/
   ```

   Los mismos veredictos JSON. Si quieres el recibo a nivel de bytes, cuida el newline final de la CLI y deja que diff no diga nada:

   ```bash
   diff <(cargo run -q -p pulse-cli -- classify < ../fixture.json) \
        <(curl -s -X POST -H "content-type: application/json" --data @../fixture.json https://pulse-edge-rs.<your-subdomain>.workers.dev/; echo)
   ```

   Un binario corrió en tu laptop. El otro corrió como WebAssembly en la ciudad de Cloudflare que estuviera más cerca de ti. El clasificador ni sabe ni le importa. Después demuestra que la ida y vuelta tiene memoria: hazle un GET pelado con `curl -s` a la URL y mira volver clasificadas las últimas muestras que POSTeaste, `npx wrangler deploy` otra vez, GET otra vez. KV sobrevive al redeploy porque nunca estuvo adentro del worker; el worker no tiene estado y es reemplazable, el namespace persiste.

![Tanto el motor de TypeScript como el de Rust se abren hacia sus superficies de deployment, con los dos workers del edge alineados bajo un solo comando de deploy y contrato de plataforma compartidos.](assets/v08-diagram.webp)

## Challenge

Totalmente solo, datos puros del motor, sin I/O nuevo: expón `GET /transitions` devolviendo la tabla de transiciones legales de la máquina ProbeState como JSON. Tienes todo: las reglas de transición de m04-l3 viven en `next_state`, el router es el `match` sobre `url.path()` que llegó escrito en el paso 4 (extenderlo es una edición de un solo brazo), y `Response::from_json` serializa cualquier cosa `Serialize`. Sé preciso sobre la forma de la tabla antes de codearla, porque `next_state` toma TRES entradas, y una tabla con clave de par `(from, probe_ok, to)` no puede expresar para nada la escalera de Degraded: para `(Degraded, false)` la respuesta depende del conteo de fallas. Así que las filas de la tabla son `(from, probe_ok, consecutive_failures, to)`, y enumerar `consecutive_failures` de 0 a 3 cubre cada cambio de comportamiento, ya que la única guarda de la máquina está en tres. Deriva las filas llamando a `next_state` en tres loops anidados en vez de escribirlas a mano; una tabla derivada nunca puede desviarse del código. Aceptación: la ruta responde en la URL en vivo, las filas muestran las reglas de m04-l3 (las filas `Degraded, false` voltean a Down exactamente cuando el conteo llega a tres, y cualquier éxito, incluso desde Down, se recupera derecho a Up), y `cargo check` en el proyecto del worker se mantiene limpio. Si quieres la verificación espejo: la tabla que sirve tu ruta debería coincidir con los brazos del match que escribiste en m04-l3, brazo por brazo, con la guarda visible como el conteo donde cambian las filas Degraded.

## Checkpoint

Barrera sobre hacer, dos pegados: el lado a lado de dos líneas del paso 7 del lab mostrando veredictos idénticos desde `pulse-cli classify` y el curl, y la salida del GET-después-del-redeploy demostrando que la ida y vuelta de KV sobrevivió a un deploy. (Si tomaste el challenge, la URL de tu ruta `/transitions` respondiendo es el tercero de bonus; como cada challenge de este curso, es evidencia extra, no la barrera.) Ese primer pegado es la tesis del módulo comprimida en dos líneas de terminal, y es un triunfo genuino de 30 segundos para mostrarle a alguien.

Lo que ahora puedes hacer, concretamente: compilar un crate puro de Rust a wasm32-unknown-unknown y explicar desde el nombre propio del target por qué tokio y reqwest no pueden venir; hacer scaffold, cablear y desplegar un proyecto de workers-rs cuya lógica es un crate del workspace en el que ya confiabas; leer un pin pre-1.0 como una decisión y no como desactualización; correr KV desde Rust con serde en los dos extremos de la tubería; y poner R2, D1, Durable Objects, Queues y Containers en un mapa con su realidad de tier gratis adjunta.

La pregunta de recuperación antes de que cierres la terminal: tu compañero de equipo agrega reqwest al crate del motor "solo para un helper rápido de sondeo" y el build de WASM se rompe. ¿Cuál es el comentario de review de una sola oración? (El I/O es del cascarón; el motor se queda puro para que cada host, nativo o unknown, pueda cargarlo.)

Si worker-build o el target de wasm pelearon con tu máquina, dime el OS y el error en el feedback del curso. El toolchain de Rust para el edge es lo más joven que entrega este curso, ese pin pre-1.0 es honesto al respecto, y los reportes de falla reales deciden si el cuadro de triage de esta lección crece.

La estación ahora corre en dos lenguajes sobre cuatro plataformas, y cada una de esas superficies ya está sondeando un endpoint RPC de Solana, en una blockchain cuya meta de tiempo de slot bajó de 400ms a 300ms, un cuarto más corta, la misma semana en que se investigó este curso; el próximo módulo te da los instrumentos para chequear ese número tú mismo. El módulo 8 deja de tratar ese endpoint como una URL más: lecturas con kit desde TypeScript, JSON-RPC crudo desde Rust, y una transferencia firmada en devnet para demostrar que puedes escribir, no solo mirar. Los motores están listos. Hora de apuntarlos a la blockchain de verdad.
