# dominio de cargo: editions, workspaces, features, MSRV

## Resumen

m05-l1 le dio serde al motor: `pulse-rs` parsea el mismo archivo de config que la flota de TS parsea con zod, los tipos de sonda viven en un enum etiquetado, y tu primer pipeline de iteradores filtra y mapea la lista de targets. Todo eso sigue sentado en un solo crate. Esta lección arregla la forma, no el código: partes `pulse-rs` en un workspace de cargo de dos crates en los primeros diez minutos, y después pasas el resto de la sesión aprendiendo a leer el archivo que produjo esa partición, porque Cargo.toml es donde viven las editions, el MSRV, las features y los pins de versión de otra gente. Cómo corren las reps esta vez: la partición misma es trabajada conmigo, elevar las dependencias va guiado por los errores del propio compilador, declarar tu MSRV es cosa tuya, y el ejercicio final de leer pins es totalmente sin guía. Ese último es el músculo que esta lección existe para construir.

## La negociación

Cargo.toml parece configuración. En realidad es una negociación con cada máquina que alguna vez vaya a construir tu código: tu laptop, CI, el toolchain de tres años de un contribuidor, el resolver eligiendo versiones en una máquina que nunca vas a ver. Cada línea adentro es un término de ese contrato. Vamos a escribir uno que valga la pena firmar, empezando ahora, primero la partición, después la teoría.

Desde la raíz de `pulse-rs`:

```bash
cargo new crates/pulse-engine --lib
cargo new crates/pulse-cli
```

Dos crates nuevos, uno librería, uno binario. Ahora reemplaza el `Cargo.toml` raíz entero (el que viene cargando el proyecto entero desde m04-l1) con tres líneas:

```toml
[workspace]
resolver = "3"
members = ["crates/pulse-engine", "crates/pulse-cli"]
```

Esa línea del medio se gana su explicación más adelante en esta lección. Mueve el código: todo módulo excepto `main.rs` va al motor, `main.rs` va a la CLI. Una trampa primero: `cargo new` ya dejó caer un `main.rs` de hello-world adentro de `crates/pulse-cli/src/`, y `git mv` se niega a sobrescribir un destino existente (`fatal: destination exists`), así que borra el stub antes del movimiento:

```bash
git mv src/config.rs src/engine.rs crates/pulse-engine/src/
rm crates/pulse-cli/src/main.rs
git mv src/main.rs crates/pulse-cli/src/main.rs
rm -rf src
```

Tus nombres de archivo pueden diferir de los míos según cómo hayas cortado m05-l1; la regla no: todo lo puro va al motor, el punto de entrada va a la CLI. Dale al motor sus dependencias pegando el viejo bloque `[dependencies]` en `crates/pulse-engine/Cargo.toml`, menos una línea: `anyhow` se queda afuera del motor, porque es maquinaria del lado del binario según el canon de m04-l2 (thiserror en la librería, anyhow en el binario) y en cambio se mueve al manifest de la CLI de abajo:

```toml
[package]
name = "pulse-engine"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.151"
thiserror = "2.0.20"
```

(Esos son los dígitos vigentes en crates.io mientras escribo, sondeados el 2026-09-02; los tuyos son los que te haya dejado m05-l1, y por ahora está bien, el lab los revisa.) El `src/lib.rs` del motor declara los módulos y re-exporta los nombres que el mundo de afuera puede usar:

```rust
pub mod config;
pub mod engine;

pub use config::{Config, ProbeKind, ProbeTarget, Target, parse_config};
pub use engine::{
    FixtureSource, LatencyMs, ProbeError, ProbeState, drive, next_state, total_latency,
};
```

Cura esa lista contra sus consumidores, no contra el `main.rs` de hoy solamente: `next_state`, `LatencyMs` y `total_latency` están ahí porque el brazo de report de m05-l3 y el poller de m06 llaman a los tres por estos nombres pelados de `pulse_engine::`. Una lista de re-exports que solo cubre al llamador actual obliga a todo consumidor futuro a hacer espeleología por los module paths de `pulse_engine::engine::`, lo que anula por completo el punto de nombrar una superficie pública.

La CLI depende del motor por path y se queda solo con lo que necesita un binario delgado:

```toml
[package]
name = "pulse-cli"
version = "0.1.0"
edition = "2024"

[dependencies]
pulse-engine = { path = "../pulse-engine" }
anyhow = "1.0.104"
```

Y `main.rs` se encoge a un consumidor, llamando a la firma exacta de `parse_config` que el Checkpoint de m05-l1 te dijo que no tocaras:

```rust
use pulse_engine::{Config, parse_config};

fn main() -> anyhow::Result<()> {
    let raw = std::fs::read_to_string("pulse.config.json")?;
    let config: Config = parse_config(&raw)?;
    for target in &config.targets {
        println!("{} -> {:?}", target.name, target.kind);
    }
    Ok(())
}
```

Arregla los paths de `use` en los módulos movidos (`crate::engine::ProbeError` sigue funcionando adentro del motor; todo lo que usaba `main.rs` ahora llega por `pulse_engine::`), y después:

```bash
cargo check --workspace
```

Verde. Diez minutos adentro, y tu Rust ya tiene la forma de todo repo serio de Rust que vayas a abrir. agave, el cliente de validador sobre el que corre todo este ecosistema, es esta estructura exacta a mayor escala: un manifest raíz, una lista de members estilo `crates`, librerías en el medio, binarios en el borde.

![Un crate que sostiene tres archivos se convierte en un workspace donde una CLI delgada y los futuros consumidores apuntan todos a una sola librería de motor pura.](assets/v01-diagram.webp)

¿Por qué esta forma, y por qué ahora? Porque M4 te hizo pagar por la pureza: el clasificador, los tipos de config, el enum de errores, todos toman valores y devuelven valores, sin I/O en ninguna parte cerca de ellos. Esa inversión empieza a pagar renta hoy. Un núcleo puro compila a una librería que cualquier cosa puede consumir: la CLI que acabas de hacer, la suite de pruebas, el daemon que este curso construye después, incluso un worker WASM en el tier de deploy. El borde impuro se queda delgado e intercambiable. Si esta canción te suena conocida, debería: es m03-l1 nota por nota, donde extrajiste `pulse-core` a un workspace de pnpm para que el panel y el cron lo pudieran compartir. Monorepo-lite es una sola idea vestida con dos toolchains, y ahora la construiste en los dos.

### Una declaración, muchos suscriptores

Ahora mismo cada member declara sus propias versiones de dependencias en su propio manifest: el motor carga dígitos para serde, serde_json y thiserror, y la CLI carga dígitos para anyhow más su propia escritura con `path` de la dependencia del motor. Todavía no hay nada duplicado, y esa es exactamente la trampa: el primer crate futuro que necesite alguna de estas, y este workspace va a crecer, saca sus dígitos por copiar y pegar del manifest que se te ocurra abrir, y desde ese momento dos crates pueden sentarse en líneas distintas sin que ningún diff lo diga. Dos manifests, dos lugares donde las versiones se pueden desviar, y hoy, mientras la cuenta es chica, es el momento barato para cerrar la puerta. agave tiene cientos de crates internos, y a esa escala la deriva es un incidente de cadena de suministro esperando un diff de lockfile que nadie lee. Su respuesta, y el patrón que adoptas en el lab, es `[workspace.dependencies]`: declara cada dependencia compartida una sola vez en la raíz, con su versión y sus features, y deja que los crates member se suscriban.

Acá está el manifest raíz que vas a tener al final del lab:

```toml
[workspace]
resolver = "3"
members = ["crates/pulse-engine", "crates/pulse-cli"]

[workspace.dependencies]
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.151"
thiserror = "2.0.20"
anyhow = "1.0.104"
pulse-engine = { path = "crates/pulse-engine" }
```

Y el bloque de dependencias de cada member colapsa a suscripciones:

```toml
[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
```

Fíjate en lo que el member NO carga: dígitos de versión. La raíz declara; el member opta con `workspace = true`. Los members que nunca mencionan serde nunca lo reciben, que es el punto, la inyección automática inflaría cada crate con cada dependencia. Un bump de versión se vuelve un diff de una línea en la raíz, y ningún par de crates del workspace puede sentarse en silencio en líneas distintas de serde. Esta es la forma observada de agave, no un invento del curso: su manifest raíz declara cada dependencia compartida exactamente una vez, features incluidas, sondeado en vivo el 2026-09-02.

![El manifest raíz declara serde una vez, el motor se suscribe y lo recibe, la CLI no se suscribe y no recibe nada, un solo lockfile abarca los dos.](assets/v02-flowchart.webp)

Antes de la segunda mitad de la higiene, deja clavado qué es en realidad una feature, porque venimos usando una desde m05-l1 sin nombrarla. Una feature es una flag con nombre que el autor de un crate expone y que deja código opcional y dependencias opcionales detrás de una barrera de compilación condicional. `serde = { version = "1.0.229", features = ["derive"] }` eres tú prendiendo la flag `derive` de serde, que jala el crate proc-macro `serde_derive` y la maquinaria detrás de `#[derive(Deserialize)]`; deja la flag apagada y ese subárbol entero nunca compila. Las features son aditivas por convención: prender una agrega capacidad, nunca la quita, que es lo que le deja a cargo hacer unificación de features, construyendo cada crate una sola vez con la unión de cada feature que cualquier crate del grafo haya pedido. Y todo crate trae una feature especial llamada `default`, el conjunto que el autor prende por ti salvo que digas otra cosa.

Que es donde entra la segunda mitad de la higiene de agave: `default-features = false`. Cada autor de crate elige un conjunto de features por defecto para el consumidor promedio, y un workspace no es promedio. Apagar los defaults y nombrar solo las features que usas quiere decir que cada capacidad de tu build fue elegida a propósito: binarios más chicos, builds más rápidos, y un manifest que documenta para qué sirve en realidad cada dependencia. agave lo aplica con criterio y no con dogma, que también vale la pena copiar. Su serde se queda con los defaults prendidos más `derive`; su reqwest y su clap corren con defaults apagados porque esos crates cargan maquinaria opcional pesada (stacks de TLS, helpers de terminal) que un validador no quiere por accidente.

El pero, y te lo vas a topar a propósito en el lab: cuando apagas una feature por defecto que algo necesitaba, el error no lo dice. Sale a la superficie como un ítem que falta adentro del código de la DEPENDENCIA, un trait no implementado, un módulo que parece haberse esfumado de un crate que definitivamente tiene uno. El compilador está diciendo la verdad, el ítem se compiló afuera, pero apunta a la fuente de ellos, no a tu línea del manifest. El debugger para esto es `cargo tree -e features`, que imprime el grafo de dependencias resuelto con cada arista de feature visible. Para la arqueología de manifests no hay nada mejor, y viene adentro de cargo, nada que instalar:

```bash
cargo tree -e features -p pulse-engine
```

Córrelo ahora contra el workspace que acabas de partir y lee las aristas de serde antes de que el lab te haga necesitarlas.

El trade-off, nombrado antes de que te encariñes: los workspaces cuestan acoplamiento. Cada member resuelve contra el mismo grafo de dependencias, un lockfile, una negociación. Si la CLI algún día quiere una major nueva y brillante de algún crate y el motor depende de algo que fija la vieja, la CLI espera. El apetito de upgrade de un crate puede quedar rehén de la restricción de otro, y en unos minutos vas a ver exactamente hasta dónde puede llegar eso adentro de agave. Versiones compartidas y cero deriva, comprados con destino compartido. Para un proyecto como el nuestro, y para la mayoría, el trade-off vale la pena tomarlo a propósito.

### Editions, con honestidad

Tus dos crates nuevos dicen `edition = "2024"`, porque lo escribió `cargo new`. Hora de saber qué firmaste. Una edition es el mecanismo de Rust para hacer cambios de ruptura en el lenguaje sin romperle a nadie: tu crate declara contra qué conjunto de reglas fue escrito, y el compilador le exige a cada crate que cumpla su propia declaración, para siempre. Crates en editions distintas linkean juntos sin problema. Por eso un crate de edition 2015 todavía compila hoy y por eso las editions son contratos por crate, no eventos del ecosistema.

La edition 2024 es la vigente. Salió en Rust 1.85.0 el 2025-02-20, y a nuestro nivel cambió tres cosas que vas a notar de verdad: el resolver de dependencias pasa a v3 por defecto (próxima sección), los bloques `extern` tienen que marcarse `unsafe`, y tomar referencias a `static mut` queda prohibido. El mismo release también trajo los async closures, y acá va una distinción que la mayoría de los posts de blog se equivoca: los async closures son una feature del lenguaje de Rust 1.85.0, disponible en toda edition, no detrás de una barrera de 2024. "Rust 1.85 trajo la edition 2024 más los async closures" es la oración precisa, e importa porque te dice lo que una edition NO es: las features nuevas llegan con los releases del compilador; las editions solo alojan los cambios de reglas incompatibles.

Ahora la parte honesta. Revisé los manifests de cinco repos de Rust cercanos a Solana mientras investigaba este curso (agave, yellowstone-grpc, photon, jito-relayer, carbon, sondeados el 2026-09-01), y cuatro de los cinco todavía declaran `edition = "2021"`. Solo agave se movió. El tren sale puntual; el ecosistema se sube tarde. Así que enseñamos 2024 y escribimos 2024, pero nadie debería pretender que el ecosistema migró, y la regla del contribuidor se sigue directo: cuando mandas un PR a un repo de 2021, escribes 2021. Las editions son el contrato del repo, no tu preferencia personal, y copiar `edition = "2024"` de un tutorial al crate de otra persona es exactamente el tipo de drive-by que nadie mergea. Para tus PROPIOS crates, cuando con el tiempo llegue la próxima edition, el costo de subirse es chico: `cargo fix --edition` reescribe los patrones incompatibles de forma mecánica, cambias el dígito, revisas el diff. La guía de editions documenta ese baile de punta a punta, que es parte de por qué el tren puede seguir saliendo puntual.

Y una declaración con fecha encima, 2026-09, porque el catálogo en el que vive este curso tiene una regla que parece contradecir esta lección, y deberías escucharla de mí. Los cursos on-chain de Solana de la Academy escriben `edition = "2021"` por regla, y para ellos 2024 no es pasado de moda, está rechazada: su Rust calificado compila en un build server cuyo toolchain todavía no acepta la edition 2024, así que escribir 2024 ahí produce un build roto, no un desacuerdo de estilo. Este curso es una excepción deliberada a esa regla, y es seguro por exactamente dos razones: cada línea de Rust de acá compila en tu propia máquina con tu propio rustc actual, y los challenges calificados son ejercicios de un solo archivo que compilarían idéntico bajo cualquiera de las dos editions. Así que enseñamos la edition vigente a propósito, y cuando después te inscribas en un curso on-chain, ya eres dueño de la regla que resuelve la tensión: el build server es el repo. Sigue SU edition.

![Cuatro editions de Rust salen a intervalos de tres años mientras una revisión de cinco repos muestra que la mayoría del ecosistema sigue en la edition anterior.](assets/v03-timeline.webp)

Una cosa más que tu manifest raíz de tres líneas ya resolvió, y ahora te la puedo explicar. La raíz de un workspace como el nuestro es un manifest virtual: no tiene `[package]`, así que no tiene `edition`, así que cargo no puede inferir qué resolver querías. Deja `resolver = "3"` afuera y cargo te lo dice en cada build:

```text
warning: virtual workspace defaulting to `resolver = "1"` despite one or more
workspace members being on edition 2024 which implies `resolver = "3"`
```

Esa advertencia es cargo pidiéndote que declares un término del contrato de forma explícita. Ya lo hiciste.

### MSRV es un contrato

`rust-version` es el campo del manifest que nadie enseña y que todo repo serio carga. Declara tu Minimum Supported Rust Version: el toolchain más viejo al que se le deja construir tu crate. Se hace cumplir, no es decorativo. Apunta un cargo demasiado viejo a un crate que declara `rust-version = "1.98.0"` y el build se muere de inmediato con:

```text
error: rustc 1.93.1 is not supported by the following package:
  pulse-engine@0.1.0 requires rustc 1.98.0
```

Lee ese error una vez y entiendes el campo: es un portero de entrada, verificado antes de que compile una sola línea de código. Y `cargo add` lo respeta en la otra dirección, eligiendo versiones de dependencia cuyo propio MSRV encaja con el tuyo en vez de pasarte algo que tu piso no puede parsear.

Lo que nos trae de vuelta a `resolver = "3"`. El resolver v3 es el default de la edition 2024 (necesita Rust 1.84 o más nuevo para siquiera correr), y su único trabajo es hacer que la resolución de versiones sea consciente del MSRV. Bajo v2, cargo agarra la versión semver-compatible más nueva de cada dependencia y deja que un toolchain viejo descubra el problema en tiempo de compilación. Bajo v3, cargo compara el `rust-version` declarado de cada candidata contra el tuyo y retrocede más allá de los releases que exigen un compilador más nuevo del que soportas. Mecánicamente cambia una clave de config, `resolver.incompatible-rust-versions`, de `allow` a `fallback`.

Escribe esa clave con cuidado: es plural, `incompatible-rust-versions`. Lo señalo porque la guía de editions misma imprime un typo en singular en su página del resolver, mientras que la referencia de cargo imprime la clave real, cuatro veces. Copia la escritura de la página oficial equivocada y entregas una clave de config que cargo ignora en silencio. Dos fuentes oficiales, una de ellas equivocada: esta es la costumbre de verificar-tus-fuentes de este curso en miniatura, y aplica a rust-lang.org exactamente igual que a un blog cualquiera. Cuando dos docs no coinciden, gana la referencia más cercana a la implementación.

![El resolver v2 elige la dependencia más nueva y rompe un toolchain viejo mientras el resolver v3 compara MSRVs y recurre a una versión que encaja.](assets/v04-flowchart.webp)

¿Entonces qué número escribes? No hay respuesta correcta, solo un contrato que eliges a propósito, y el ecosistema te muestra los dos polos. agave declara `rust-version = "1.98.0"`, que es exactamente el stable vigente (1.98.0 salió el 2026-08-20; los dos datos sondeados el 2026-09-02, y con un release estable cada seis semanas el dígito se va a mover otra vez pronto). carbon, un framework de librería para construir indexadores, declara `rust-version = "1.82"`, dieciséis releases atrás. Las dos están bien. Una aplicación como un validador controla su propio entorno de build, así que sigue la punta y toma cada API nueva de std el día que aterriza. Una librería corre en los toolchains de otra gente, así que va atrás, dándoles la bienvenida a usuarios que nunca conoció al costo de prohibirse APIs más nuevas. Declarar 1.82 les da la bienvenida a usuarios más viejos Y te ata las manos; declarar 1.98 te libera las manos Y excluye gente. Las apps siguen la punta, las librerías van atrás, y el único pecado de verdad es no elegir.

![Una aplicación de validador fija su piso de Rust en el stable vigente mientras una librería de indexador va dieciséis releases atrás, cada postura cambiando libertad por alcance.](assets/v05-comparison.webp)

Para `pulse-rs` vas a declarar `rust-version = "1.85"` en los dos crates en el lab. El razonamiento es la postura de librería: nada en nuestro código necesita algo más nuevo que el piso mismo de la edition 2024, y el motor es una librería por construcción, así que va atrás a propósito. Una trampa antes de que lo escribas: `rust-version` es un piso, no un pin. Detiene a un toolchain viejo de construirte; no te detiene a TI, en un toolchain nuevo, de escribir un idiom que tu piso declarado no puede parsear. La forma honesta de hacerlo cumplir es un job de CI que construya sobre el toolchain del MSRV mismo. Hoy no vamos a agregar uno; las librerías serias sí. Sin él, el campo es una promesa sin prueba.

### Leer pins como un dev que trabaja

Todo hasta acá fue escribir tu propio manifest. La habilidad de ciclo de vida de dev que se esconde acá es leer los de otra gente, porque cada línea de dependencia de un repo real es una decisión que alguien tomó, y los pins son donde se ven las decisiones. Tres artefactos vivos, todos sondeados de los manifests de sus repos el 2026-09-02. Para cada uno, la pregunta de trabajo: ¿qué te está diciendo este pin?

Primero. El manifest raíz de agave:

```toml
reqwest = { version = "0.12.28", default-features = false }
```

El reqwest más nuevo de crates.io es 0.13.4, y 0.13.0 viene afuera desde fines de 2025. El repo más activamente mantenido del ecosistema de Solana está una major entera atrás en su cliente HTTP, y no es un accidente, nadie se olvida de una dependencia en un codebase así de auditado. Una major vieja fijada en un repo vivo quiere decir que alguien evaluó el upgrade y dijo todavía no: costo de migración, riesgo de comportamiento, superficie de review, algo. El pin es una decisión que estás leyendo, no una tarea que nadie hizo.

Segundo, el mismo manifest, más abajo:

```toml
clap = { version = "2.33.1", default-features = false, features = ["suggestions"] }
```

clap hoy es 4.6.6, y la línea 4.x viene estable desde 2022. Este pin es una major de hace una DÉCADA, todavía entregándose en producción, todavía parseando los argumentos del software que corre una red monetaria. Me acuerdo de la primera vez que un pin así me frenó en seco en el repo de otra persona: mi instinto dijo mal mantenido, y mi instinto estaba equivocado. La lectura correcta es más fría. Lo que sea que clap 2 haga por esos binarios, lo hace, y el costo de tocar algo que funciona, multiplicado por cada binario del workspace, perdió la pelea costo-beneficio todos los años durante diez años. La peor respuesta a esta línea es el clásico primer PR rechazado: un upgrade drive-by a clap 4 de alguien que leyó el número de versión pero no el repo. Fíjate también en que este pin es el trade-off del workspace de hace un rato a escala completa: porque agave centraliza cada versión en `[workspace.dependencies]`, el clap de un crate es el clap de todos los crates, así que un upgrade no es una migración, son todas a la vez, y eso es precisamente por qué el pin se sostiene.

Tercero, de photon, el indexador de Helius para cuentas comprimidas:

```toml
sqlx = { version = "0.6.2", features = [
    "macros",
    "runtime-tokio-rustls",
    # ...more features elided
] }
# time pinned because of https://github.com/launchbadge/sqlx/issues/3189
```

sqlx hoy es 0.9.0, tres majors adelante. Pero mira lo que se sienta debajo del bloque: un comentario que enlaza el issue upstream exacto al que se remonta el pin. Este es el estándar de oro, el pin que se explica solo. Un contribuidor que llega a este manifest no tiene que hacer ingeniería inversa de la intención desde el git blame; la razón está a un clic, y cuando el issue upstream se cierre, quien lo vea sabe con precisión qué volver a probar. Cuando fijes algo en `pulse-rs` por una razón que no es obvia, este comentario es la forma a copiar.

![Tres barras miden cuántas majors va atrás cada pin de producción respecto de su release más nuevo, de una para reqwest a tres para sqlx.](assets/v06-chart.webp)

Si el patrón se siente específico de Rust, no lo es. Viste a la mitad de TypeScript de esta misma stack hacerlo más rápido y más fuerte: @solana/kit entregó dos majors en poco más de nueve semanas, entre el 2026-06-16 y el 2026-08-21, justo detrás de una minor, y las lecciones de M3 de este curso te enseñaron a fijar lo que tus dependencias de verdad declaran peer en vez de lo que npm llama latest. Misma regla, los dos toolchains: lee lo que fija tu ecosistema antes de actualizar nada. Es una costumbre de supervivencia, no pedantería.

El entregable profesional de leer un manifest es una línea por pin: qué implica para un contribuidor, qué igualar si mandas un PR. Vas a escribir tres de esas líneas en el challenge, y la costumbre vuelve con apuestas de verdad en el lab de auditoría de dependencias de M9, donde el árbol que lees es el tuyo.

**Profundiza (el 20%).** esta lección enseñó los patrones de workspace, edition, MSRV y features que vas a usar semanalmente, más la habilidad de leer pins. Lo que se salteó a propósito es la referencia completa de campos del manifest, los perfiles y la personalización del build, y cómo funciona la resolución desde adentro. Los capítulos canónicos, los dos sondeados en vivo hoy: la referencia de cargo sobre el resolver en [https://doc.rust-lang.org/cargo/reference/resolver.html](https://doc.rust-lang.org/cargo/reference/resolver.html), y sobre rust-version en [https://doc.rust-lang.org/cargo/reference/rust-version.html](https://doc.rust-lang.org/cargo/reference/rust-version.html). Esa página del resolver es además el lugar correcto de donde copiar la clave `incompatible-rust-versions`. Nada del lab depende de ninguna de las dos páginas; guárdalas como bookmark para el día en que una resolución te sorprenda.

## Lab: elevar, romper, declarar, verificar

La partición ya pasó en la apertura, así que el lab arranca desde un `cargo check --workspace` verde y hace que el workspace se gane su lugar. Los pasos 1 a 3 son guiados; el paso 4, la declaración del MSRV, es el que es tuyo; el paso 5 te da el archivo de pruebas de integración completo, porque lo que se gana su lugar ahí es leer la segunda línea del runner y saber por qué existe el tier, no escribir ocho líneas; el paso 6 le demuestra el asunto entero a CI. La rep sin guía a la que esta lección de verdad le está apostando es el challenge, tres veredictos de pin escritos en frío.

1. Haz commit de la partición tal como está, para que todo diff que siga sea legible:

   ```bash
   git add -A && git commit -m "split pulse-rs into engine + cli workspace"
   ```

2. Eleva las dependencias. Agrega al manifest raíz la tabla `[workspace.dependencies]` de la sección de teoría (serde con `derive`, serde_json, thiserror, anyhow, y la entrada de path de `pulse-engine`), después reescribe los dos bloques `[dependencies]` de los members a suscripciones: `serde = { workspace = true }` y compañía en el motor, `pulse-engine = { workspace = true }` y `anyhow = { workspace = true }` en la CLI. Corre `cargo check --workspace` después de cada manifest que toques, no al final; un error de manifest encontrado de inmediato nombra su propia causa. Y el desliz clásico de acá falla a gritos y de forma útil: suscríbete a algo que te olvidaste de declarar en la raíz y cargo dice exactamente qué falta:

   ```text
   error inheriting `thiserror` from workspace root manifest's
   `workspace.dependencies.thiserror`

   Caused by:
     `dependency.thiserror` was not found in `workspace.dependencies`
   ```

   Guarda ese contraste: los errores de manifest como este nombran su causa en texto plano, mientras que el error de feature con el que te vas a topar en el próximo paso hace cualquier cosa menos eso.

3. Ahora rómpelo a propósito, porque la rep guiada de acá es leer la rotura. En la declaración raíz, cambia serde a defaults apagados:

   ```toml
   serde = { version = "1.0.229", default-features = false, features = ["derive"] }
   ```

   `cargo check --workspace` de nuevo y lee lo que recibes. No una nota amable sobre features. Esto (en cargo 1.98.1; los toolchains más viejos nombran items helper distintos del mismo módulo, `Content` en vez de `TaggedContentVisitor`, y la forma es idéntica):

   ```text
   error[E0433]: failed to resolve: cannot find `TaggedContentVisitor` in `de`
     --> crates/pulse-engine/src/config.rs
   note: found an item that was configured out
     --> .../serde-1.0.229/src/private/de.rs
   ```

   El error apunta adentro de la fuente propia de serde, a tu línea de derive, sobre un ítem que fue "configured out," y se repite para un puñado de nombres hermanos (`ContentDeserializer`, `Content`, `ContentVisitor`). Nada en ninguna parte dice que apagaste `std`. Esta es la trampa de la sección de teoría en vivo en tu pantalla, y el debugger es:

   ```bash
   cargo tree -e features -p pulse-engine
   ```

   En la salida, encuentra serde y lee qué aristas de feature existen. Con los defaults apagados vas a ver la arista `derive` pero ninguna arista `default`, y esa ausencia es el bug entero. Ahora toma la decisión deliberada: nuestro motor parsea archivos con `String`s y `Vec`s por todas partes, necesita `std`, y agave mismo deja los defaults de serde prendidos. Revierte el cambio. Defaults apagados es una herramienta para crates pesados con maquinaria opcional que no quieres; aplicado a serde acá es hacer cargo cult de la higiene sin el criterio. Saber cuándo NO aplicar el patrón es el patrón.

![Cada línea del manifest de workspace terminado lleva una nota al margen que explica la promesa que le hace a quienes construyen y al resolver.](assets/v07-annotated-code.webp)

4. Declara el contrato del MSRV. Este lo lidera quien aprende: agrega `rust-version = "1.85"` a las tablas `[package]` de los dos crates, y sé capaz de decir por qué 1.85 y no 1.98 en una oración antes de seguir (la división app-versus-librería de la sección de teoría es la oración). Demuéstrate que el campo se hace cumplir leyendo, no corriendo: el texto de error que está arriba en la sección de MSRV es lo que un toolchain 1.84 imprimiría a tus usuarios. Después `cargo test --workspace`. Los dos crates construyen, las pruebas de m04 del motor pasan desde la raíz, el mismo verde que antes de la partición, forma nueva.

5. Cobra el tier de integración. m04-l3 nombró el segundo tier de pruebas de Rust, las pruebas de integración en un directorio `tests/` de nivel superior, y te dijo que lo dejaras estacionado hasta que pasara la partición del workspace. Acaba de pasar, así que cóbralo. Crea `crates/pulse-engine/tests/engine_contract.rs`:

   ```rust
   // The integration tier: this file compiles as its own tiny crate, linked
   // against pulse-engine, so it sees exactly what any outside consumer sees.
   // Private items are unreachable from here; pub ones are, either by the
   // short name lib.rs re-exported or by their full module path.
   use pulse_engine::{FixtureSource, ProbeState, drive};

   #[test]
   fn the_fixture_story_survives_the_public_surface() {
       // m04-l3's six-fixture walk: two clean probes, three failures past the
       // budget, one recovery. Ends Up, exactly as you walked it by hand.
       let mut source = FixtureSource::new(vec![212, 487, 1600, 1700, 1800, 90]);
       assert_eq!(drive(&mut source, 1500), ProbeState::Up);
   }
   ```

   Corre `cargo test --workspace` y lee la salida con ojos nuevos: debajo del binario de pruebas unitarias recibes una segunda línea de runner, `Running tests/engine_contract.rs`, porque cargo compiló ese archivo como su propio crate y lo linkeó contra tu librería. Esa es la costura entre los dos tiers, y es visibilidad, no geografía: un bloque `#[cfg(test)] mod tests` vive adentro del módulo y ve items privados, mientras que un archivo en `tests/` consume el crate exactamente como lo hace `pulse-cli`, por la superficie pública que curaste hace diez minutos. Lo que le deja a esta única prueba hacer un trabajo que las cinco pruebas unitarias no pueden: saca un nombre de la lista de re-exports de `lib.rs` y la suite unitaria se queda verde mientras este archivo deja de compilar, el primer consumidor en notar que la puerta de entrada cambió. Una prueba mantiene el tier abierto hoy; la suite de transición de m04-l3 se queda donde está, adentro del módulo al lado del match que fija, porque hurgar en los casos borde de `next_state` es trabajo de tier unitario y la ubicación del archivo debería decirlo.

6. Haz push, y mira correr sin cambios la barrera de CI de m04-l3. Sin ediciones al workflow: los comandos de cargo de la barrera corren contra lo que sea que describa el manifest raíz, y el manifest raíz ahora describe un workspace, así que `cargo test` cubre a los dos members y a los dos tiers, el crate nuevo de `tests/` incluido. Un CI agnóstico al layout es uno de los pagos silenciosos de que cargo sea una sola herramienta en vez de cinco. Cuando la ejecución esté verde, la vara de aceptación para la mitad de build de esta lección está cumplida: deps declaradas una sola vez en la raíz, los dos crates en edition 2024 con `rust-version` declarado, `cargo test --workspace` verde localmente y en CI.

## Challenge

La rep sin guía, en papel, sin compilador en el que apoyarte. Tres extractos de manifest de la sección de teoría: el reqwest 0.12.28 de agave, el clap 2.33.1 de agave, el sqlx 0.6.2 de photon con su comentario que enlaza el issue. Para cada uno, escribe UNA línea que diga qué le dice el pin a un contribuidor que está por abrir un PR contra ese repo: qué implica sobre el codebase, y qué igualarías o evitarías tocar. Sin apoyo, sin pistas, y resiste las ganas de espiar de vuelta mis lecturas; el ejercicio es producir el veredicto tú mismo, en frío. Aceptación: tres líneas escritas, cada una nombrando una implicación concreta (qué idioms de API tiene que usar tu patch, qué no debes relitigar adentro de un PR no relacionado, o qué quiere decir el issue enlazado para volver a probar). No te quedes con las tres líneas en un archivo borrador. Haz commit de ellas, en la raíz del repo de la estación, como `docs/pin-reads.md`, un encabezado por pin y tu veredicto debajo, con fecha. Dos razones, y la segunda es la de verdad. Un veredicto que nada puede volver a encontrar es un veredicto que vas a revisar en silencio; uno del que hiciste commit es uno frente al que puedes estar equivocado. Y el lab de auditoría de dependencias de m09-l1, que pide exactamente esta habilidad contra tu propio árbol y la califica, abre este archivo y te hace leer tus veredictos en frío contra una checklist que no vas a tener hasta entonces. Escrito hoy, calificado en cuatro lecciones.

## Checkpoint

Los músculos nuevos, concretamente: partir un proyecto de Rust en la forma de workspace que el ecosistema usa de verdad, con las dependencias declaradas una sola vez y los members suscribiéndose; decir qué cambió la edition 2024 y qué no (los async closures son una feature del lenguaje de 1.85, en todas las editions); declarar un MSRV como un contrato elegido y explicar el contrato de quién refleja, el de agave o el de carbon; y leer el pin de versión de un desconocido como información en vez de ruido.

La recuperación de 30 segundos antes de cerrar la pestaña: ¿qué hace el resolver v3 que v2 no hacía, en una oración? (Considera el rust-version declarado de cada dependencia al elegir versiones, retrocediendo más allá de los releases que tu MSRV no puede construir, en vez de siempre agarrar la más nueva compatible.) Si esa oración te tomó más de un intento, vuelve a leer el diagrama de flujo de la sección de MSRV, es la única pieza de esta lección que aparece en entrevistas.

Un pedido mientras está fresco: la rotura de defaults apagados en el paso 3 del lab es deliberadamente desorientadora, y quiero saber cuánto. Anota si el error "configured out" tuvo sentido antes o solo después de `cargo tree -e features`, y dímelo en el feedback. Si la mayoría de ustedes lo entendió solo después, la próxima revisión enseña el árbol primero y rompe segundo.

El workspace ya tiene la forma del ecosistema: un crate de motor puro que cualquier cosa puede consumir, una CLI delgada encima que no hace nada más que consumirlo. Lo que quiere decir que la CLI es por ahora un cascarón hueco con un cerebro prestado, y la próxima lección se gana el nombre: clap le da una interfaz de línea de comandos de verdad, reqwest blocking por fin le da a la estación un brazo de sonda DE VERDAD en vez de latencias de fixture, y CI empieza a pasarle a desconocidos un binario que pueden descargar y correr. Cargo.toml fue la negociación; la próxima lección entrega algo que vale la pena negociar.
