# serde: el parser de la frontera

## Resumen

m04-l3 completó el motor de Rust: la máquina ProbeState, errores tipados de punta a punta, la barrera de cargo test, clippy y fmt en verde al lado de vitest en el único pipeline. Pero la config del motor está hardcodeada y sus latencias son fixtures. Hoy muere el problema de la config. Escribes a mano un parser de config, sientes exactamente lo que cuesta, después lo borras con una sola línea de derive, porque serde genera el parser desde tus tipos igual que zod infería tus tipos desde un schema en M2. El mismo archivo en disco, los dos lenguajes parseándolo, una sola disciplina vistiendo dos banderas. En el camino: enums etiquetados, el mini-lenguaje de atributos, tu primer pipeline de iteradores. Tareas de casa: la textura de completion-loop de M4 se terminó, deliberadamente. M5 vuelve al molde por defecto: el overview trabajado conmigo, los apoyos del lab adelgazando a medida que avanzas, el paso del pipeline tuyo, el challenge totalmente sin guía.

## Siente el dolor primero

Antes de que serde se gane nada, pagas precio de lista. Acá va un target de sondeo como una línea plana de config:

```text
name=api;url=https://example.com/health;timeout_ms=500
```

Parséalo a mano, en `pulse-rs`, usando nada más que `split`, `match` y la plomería de `Result` que construiste en m04-l2. Y hazlo con honestidad, porque la versión deshonesta son diez minutos y una mentira. Honesto quiere decir: partir los pares en `;`, partir cada par solo en el PRIMER `=` (esa URL está a un query string de contener un `=`), rechazar las claves desconocidas como errores en vez de encogerte de hombros, validar el scheme de la url y los dígitos del timeout, y reportar las claves faltantes en un orden fijo. Cada falla vuelve como un valor `Err`, nunca como un panic.

El panel de coding-challenge de esta lección tiene un starter, `kv-config-parser`, que compila y hace trampa con cada una de esas reglas; ábrelo en el editor del navegador y empieza a volverlo honesto ahora. Ponte un cronómetro, de verdad. El núcleo de la versión honesta se ve así:

```rust
let (key, value) = match pair.split_once('=') {
    Some((k, v)) => (k, v),
    None => return Err(format!("bad pair: {pair}")),
};
match key {
    "name" => name = Some(value.to_string()),
    "timeout_ms" => {
        let parsed = value
            .parse::<u64>()
            .map_err(|_| format!("invalid timeout_ms: {value}"))?;
        timeout_ms = Some(parsed);
    }
    other => return Err(format!("unknown key: {other}")),
}
```

Y esa es solo la mitad por par. Cada campo también tiene que rastrearse como un `Option` a lo largo del loop, porque "la clave nunca apareció" es una falla distinta de "la clave apareció rota", y la spec quiere las claves faltantes reportadas en un orden fijo. El final de la versión honesta son tres líneas de la jugada `ok_or_else` de m04-l2:

```rust
let name = name.ok_or_else(|| "missing key: name".to_string())?;
let url = url.ok_or_else(|| "missing key: url".to_string())?;
let timeout_ms = timeout_ms.ok_or_else(|| "missing key: timeout_ms".to_string())?;
```

Veinte minutos, más o menos, de seguimiento de `Option`, llamadas a `ok_or_else` y cadenas de error. Para UN registro plano. Con TRES campos. Ahora escala eso a una config a nivel de flota, un nombre de flota más un array de registros de target, la forma de m02-l2 que esta lección está por resucitar (la historia completa de a dónde fue a parar ese archivo vive unas pantallas más abajo), y haz la cuenta de parsear eso a mano. Ese número es lo que esta lección borra. Igual quédate con tu parser hecho a mano; vuelve como el challenge, y terminarlo es cómo vas a saber exactamente qué te compró el derive.

## El compilador escribe el parser

Dos instalaciones, y fíjate en que los dígitos llevan fecha:

```bash
cargo add serde@1.0.229 --features derive
cargo add serde_json@1.0.151
```

Esas versiones fueron verificadas contra crates.io el 2026-09-02; serde lleva años viviendo en la línea 1.x, así que lo que sea que `cargo add` te resuelva hoy está bien. Anécdota de guerra chica de cuando las verificamos: crates.io rechaza las llamadas de API que no mandan un header User-Agent. Nuestras propias herramientas de investigación chocaron con eso durante el barrido de datos de este curso, un curl pelado recibió algo que no era JSON donde deberían haber estado los datos de versión. Lo primero que parseas desde una frontera de verdad puede ser la página de error de alguien. El parseo de frontera existe porque las fronteras mienten, y eso incluye la frontera que consultas para aprender sobre parsers de frontera.

La flag `--features derive` importa. serde sin la feature derive compila bien como crate y después falla en tu línea `#[derive(Deserialize)]` con un error que apunta al atributo, no a `Cargo.toml`, que es exactamente el lugar equivocado al que mandarte a buscar. Si tu primer build explota en el derive, revisa la lista de features antes que nada.

Ahora el borrado. Este es el reemplazo entero del parser por el que acabas de sudar, cubriendo el archivo de config completo:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub fleet_name: String,
    pub targets: Vec<Target>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub name: String,
    pub url: String,
    pub interval_secs: u64,
    pub timeout_ms: u64,
}
```

Nada de código de parseo. Tú describes la forma; el macro derive genera el parser en tiempo de compilación, verificaciones de campo, verificaciones de tipo, errores de clave faltante, todo. `rename_all = "camelCase"` es la costura con el lado de TypeScript: el archivo en disco dice `intervalSecs` y `timeoutMs` porque lo escribió la flota, los campos de Rust son `snake_case` porque clippy tiene opiniones, y un atributo traduce entre las dos convenciones así ninguno de los dos lenguajes tiene que taparse la nariz.

Usarlo es una sola llamada a función que devuelve, y esto ya debería sentirse familiar, un `Result`:

```rust
pub fn parse_config(raw: &str) -> Result<Config, ProbeError> {
    serde_json::from_str(raw).map_err(ProbeError::BadConfig)
}
```

`serde_json::from_str` te da `Result<Config, serde_json::Error>`, y la falla es un valor que lleva qué salió mal y dónde, hasta la línea y la columna. Sin excepción, sin panic, nada que no hayas tenido antes. `map_err(ProbeError::BadConfig)` es el músculo de m04-l2 haciendo exactamente su trabajo, puenteando el error ajeno hacia la taxonomía de tu motor, y sí, ese es el nombre de la variante usado pelado como función, el truco que te enseñó clippy cuando marcó el closure redundante. Lo que quiere decir que acaba de vencer un préstamo: m04-l2 dejó estacionado `#[allow(clippy::redundant_closure)]` sobre `parse_fixture_line` con un vencimiento escrito que decía "colapsa el closure en M5, borra este allow." Es M5. Abre `engine.rs`, cambia esa línea a `.map_err(ProbeError::BadFixture)?`, borra el atributo y su comentario, y deja que `cargo clippy` confirme que la deuda quedó saldada. `ProbeError` gana una variante para recibir la falla de config:

```rust
#[error("config rejected: {0}")]
BadConfig(serde_json::Error),
```

![Un archivo de config parsea o bien a una config Ok que alimenta el loop de sondeo o bien a un valor de error que lleva línea y columna y que maneja quien llama.](assets/v01-flowchart.webp)

Una nota de resistir-el-reflejo antes de seguir. El punto entero de m04-l2 era que una falla de frontera es un valor que ruteas, así que `from_str(...).unwrap()` en la frontera de la config sería gastar dos lecciones de disciplina para ahorrar nueve caracteres. El caso de la config malformada no es excepcional. Es martes. Recibe una variante, un mensaje y una decisión, como todo lo demás.

### Mismo archivo, los dos parsers, una pantalla

Acá va la parte que llevo esperando mostrarte desde M2, con un pedazo de historia de la estación contado sin adornos primero. En m02-l2 diseñaste una config a nivel de flota, `fleetName` más un array de targets con nombre, y un schema de zod para protegerla. Después el ejercicio de ráfaga de m02-l3 sobrescribió las dos: el `packages/pulse-fleet/pulse.config.json` vivo de la flota tiene la forma de ráfaga desde entonces (`targets` como una lista plana de URLs, `timeoutMs`, `concurrency`, `retry`), y su `configSchema` tiene forma de ráfaga y es estructural para `fleet.ts` y la suite de pruebas. La forma a nivel de flota no sobrevivió en disco. Hoy vuelve a propósito, en una dirección nueva: la raíz del repo de la estación, como la config que el arco de Rust parsea de acá en adelante, mientras la config de ráfaga se queda exactamente donde está. Dos archivos, dos trabajos, honestamente separados: la config de ráfaga dice a qué URLs martillar en este instante; la config raíz dice a qué targets vigila la estación para siempre. Este es el archivo que el lab te hace crear:

```json
{
  "fleetName": "pulse-prod",
  "targets": [
    {
      "name": "docs",
      "url": "https://example.com",
      "intervalSecs": 60,
      "timeoutMs": 3000
    },
    {
      "name": "api",
      "url": "https://example.org/health",
      "intervalSecs": 30,
      "timeoutMs": 2000
    }
  ]
}
```

Y en m02-l2 escribiste el schema para él. El lab reconstruye ese schema en un módulo nuevo, `src/root-config.ts`, al lado del schema de ráfaga en vez de encima de él, para que la flota también pueda firmar este archivo:

```ts
export const rootTargetSchema = z.strictObject({
  name: z.string().min(1),
  url: z.url(),
  intervalSecs: z.number().int().positive(),
  timeoutMs: z.number().int().positive(),
});

export const rootConfigSchema = z.strictObject({
  fleetName: z.string().min(1),
  targets: z.array(rootTargetSchema).min(1),
});

export type RootConfig = z.infer<typeof rootConfigSchema>;
```

Pon eso al lado del struct `Target` de arriba y lee los dos despacio. zod: escribiste un schema, un valor en runtime que recorre la entrada, y `z.infer` derivó el tipo estático DESDE el schema. serde: escribiste un tipo, y el derive generó el parser DESDE el tipo. El mismo archivo. El mismo rechazo en la puerta. El mismo "después de esta línea, los datos son la forma sobre la que razoné." La dirección es opuesta y la disciplina es idéntica: parse, don't validate, cruza la frontera una vez hacia un tipo que no puede representar la basura, y deja que el resto del programa confíe en él.

La síntesis, y no es una metáfora: derive(Deserialize) es zod corriendo en tiempo de compilación. zod paga su flexibilidad con un objeto schema en runtime que recorre tus datos; serde paga su velocidad con una expansión de macro que no puedes ajustar en runtime. Debajo, una sola idea.

Por qué este concepto carga tanto peso en web3 específicamente: del lado Rust de este ecosistema, casi todo lo que cruza una frontera de proceso la cruza a través de serde. Cuando la investigación de este curso relevó cinco repos de producción cercanos a Solana, agave, yellowstone-grpc, photon, jito-relayer y carbon, serde y serde_json estaban en todos y cada uno de los manifests, cinco de cinco, en el mismo tier universal que tokio, thiserror, anyhow y clap. Requests y responses de RPC, archivos de config, payloads de webhook, el JSON en el que se guarda un archivo de par de claves: todo eso entra a Rust tipado a través de la maquinaria que estás aprendiendo ahora mismo. Este no es un capítulo que estás probando. Es infraestructura estructural para el resto de tu vida en Rust.

![El schema de TypeScript infiere un tipo mientras el tipo de Rust deriva un parser, y los dos consumen el mismo archivo de config en disco.](assets/v02-diagram.webp)

Esa disciplina tiene una historia que vale treinta segundos. zod 3.0.0 se entregó el 2021-05-17; la versión 4 no llegó a GA hasta el 2025-07-09. Cuatro años en una sola major, y en esa ventana "parse, don't validate" pasó de slogan de blog post a la cultura de fronteras por defecto de todo un ecosistema. La idea nunca fue específica de TypeScript. Rust solo la hace cumplir más fuerte, porque acá no hay ninguna escotilla de escape con forma de `any` para pasar de contrabando datos sin parsear más allá de la frontera; el único camino a un `Config` es a través del parser que escribió el compilador.

![Una línea de tiempo desde el release de zod 3 en 2021 hasta la GA de la versión 4 en 2025, terminando en el derive de Rust de esta lección cargando la misma disciplina.](assets/v03-timeline.webp)

### Enums etiquetados: volver irrepresentable un kind malo

La config está por crecer, porque las configs de verdad siempre lo hacen. Ahora mismo cada target es una sonda HTTP, pero en una flota de monitoreo la lista de targets nunca se queda con una sola forma por mucho tiempo: un chequeo de socket simple necesita un host y un puerto donde una sonda HTTP necesita una url. Kinds distintos, campos distintos, un conjunto cerrado. En m04-l3 modelaste exactamente esta forma en memoria: un conjunto cerrado de variantes, cada una cargando sus propios datos. La pregunta es cómo se ve eso cuando tiene que sobrevivir un viaje por JSON, y la respuesta es un campo discriminante, la misma jugada de `"kind"` que tu unión `ProbeResult` usa del lado de TS desde M2:

```rust
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ProbeKind {
    Http { url: String },
    Tcp { host: String, port: u16 },
}
```

`tag = "kind"` es la representación etiquetada internamente: serde lee primero el campo `"kind"`, despacha a exactamente una variante, y parsea los campos restantes contra la forma de esa variante. `rename_all = "lowercase"` mapea `Http` a `"http"` en disco. Una entrada de config que dice `"kind": "grpc"` falla con un error nombrado que lista las variantes legales, y vas a leer ese error exacto en el lab. El struct `Target` absorbe el enum en línea:

```rust
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub name: String,
    pub interval_secs: u64,
    pub timeout_ms: u64,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(flatten)]
    pub kind: ProbeKind,
}

fn default_enabled() -> bool {
    true
}
```

Dos atributos nuevos, los dos de uso diario. `#[serde(flatten)]` empalma los campos del enum dentro del objeto JSON del target en vez de anidarlos un nivel más abajo, así el archivo se queda plano y editable por humanos. `#[serde(default = "default_enabled")]` hace que `enabled` sea opcional en el archivo con un default explícito, que es la gentileza de migración que le deja a cada entrada existente seguir funcionando cuando llega un campo; estrictez donde te protege, lenidad donde la pediste, campo por campo.

![Una entrada de sonda JSON de siete líneas con cada clave anotada al campo del struct o a la variante del enum en la que se deserializa.](assets/v04-annotated-code.webp)

Puede que hayas notado que cada línea de derive en esta lección también dice `Serialize`. Ese es el boleto de vuelta, y no es decoración. Los mismos atributos manejan las dos direcciones, así que un `Target` se serializa de vuelta al JSON exacto, plano, en camelCase y etiquetado por kind, del que fue parseado:

```rust
let json = serde_json::to_string_pretty(&target)?;
```

```json
{
  "name": "rpc",
  "intervalSecs": 30,
  "timeoutMs": 1000,
  "enabled": false,
  "kind": "tcp",
  "host": "127.0.0.1",
  "port": 8899
}
```

Un solo set de atributos, dos direcciones, cero deriva entre lo que lees y lo que escribes. Hoy el motor solo lee; el próximo módulo, el poller de larga duración gana un endpoint `/status` que tiene que EMITIR JSON, y este derive es la razón por la que eso te va a costar una sola llamada a función en vez de una sesión de plantillas.

Ahora la trampa, mostrada una vez para que la reconozcas en el mundo real. Borra el tag y serde igual te ofrece una salida:

```rust
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LooseKind {
    Http { url: String },
    Tcp { host: String, port: u16 },
}
```

Untagged quiere decir que serde prueba cada variante en orden y toma la primera que encaja. Dale basura y el error se degrada a `data did not match any variant of untagged enum LooseKind`, sin línea, sin campo, sin pista. Peor es lo que pasa cuando los datos encajan en más de una forma: un objeto que lleva tanto una `url` como un `host` parsea alegremente como `Http` y tira el resto al piso, gana la primera coincidencia, en silencio. Untagged es para consumir formatos que no controlas y no puedes arreglar. En el momento en que eres dueño del formato, y eres dueño de este, etiquétalo.

![Una comparación que muestra que los enums etiquetados fallan con un error de variante nombrada mientras que los enums untagged adivinan por primera coincidencia y producen errores vagos.](assets/v05-comparison.webp)

El costo honesto de todo esto, porque lo hay. El lenguaje de atributos, `tag`, `rename_all`, `default`, `flatten`, es un mini-DSL, y la garantía de tiempo de compilación cubre tus TIPOS, no cómo escribiste los atributos. Escribe mal un valor de tag o apunta `rename_all` para el lado equivocado y el código compila limpio, después falla en runtime. Pruébalo: borra la línea `rename_all` de `Config` y corre contra el archivo de la flota. Build verde, después el parseo muere con ``missing field `fleet_name` ``, nombrando un campo que está JUSTO AHÍ en el archivo, a una convención de nombres de distancia. Los atributos son configuración, no código que el compilador verifica contra tus datos, y un parser derivado es un contrato con la forma exacta de la config: la evolución del schema se vuelve una falla de parseo para la que diseñas a propósito. Las reglas de diseño son cortas: los campos aditivos reciben `#[serde(default)]` para que los archivos desplegados sigan parseando, los conjuntos cerrados reciben enums etiquetados para que una variante nueva sea un error nombrado y ruidoso, y los renombres son cambios de ruptura que planificas. Vas a sentir todo esto en el momento en que el archivo viejo se encuentre con el schema nuevo. Una nota de honestidad más: serde ignora los campos desconocidos por defecto para formatos autodescriptivos como JSON, más laxo que tu `strictObject` de zod; `deny_unknown_fields` existe pero, según los propios docs de serde, no se combina con `flatten`. Trade-offs hasta el fondo; ten claro qué postura tiene cada frontera.

### Tu primer pipeline de iteradores

Concepto justo a tiempo, y el último que esta lección necesita. El `Vec<Target>` parseado es materia prima; el motor quiere una lista de sondas. Solo los targets habilitados, mapeados a la forma del motor:

```rust
#[derive(Debug)]
pub struct ProbeTarget {
    pub name: String,
    pub endpoint: String,
    pub budget_ms: u64,
}
```

Y esta es la transformación, tu primera cadena de iteradores:

```rust
let probes: Vec<ProbeTarget> = config
    .targets
    .iter()
    .filter(|t| t.enabled)
    .map(|t| t.to_probe_target())
    .collect();
```

Ya escribiste este programa antes. `targets.filter(t => t.enabled).map(toProbeTarget)` está en la flota desde M2, y los closures de adentro son los closures `map_err` de m04-l2 con trabajos distintos. Léela como la misma cadena de array vistiendo una bandera de Rust y tienes el 90%. El 10% que queda es una palabra: perezosa. `iter`, `filter` y `map` no hacen trabajo; construyen una descripción del trabajo, y nada corre hasta que un consumidor como `collect` jala los elementos a través. Por eso las cadenas de Rust no asignan una colección intermedia por paso como sí lo hacen los métodos de array encadenados de JS, y esa es toda la teoría de pereza que necesitas hoy; el tratamiento completo vive en el cuadro de Profundiza de abajo.

![Los adaptadores de iterador forman una descripción perezosa del trabajo y el consumidor collect jala cada elemento a través de la cadena entera en una sola pasada.](assets/v06-flowchart.webp)

`collect` es un consumidor entre varios, y cambiar el consumidor cambia la pregunta que responde la misma cadena. Dos que vas a usar esta semana, directo de la config de la flota:

```rust
let enabled = config.targets.iter().filter(|t| t.enabled).count();
let slowest: u64 = config.targets.iter().map(|t| t.timeout_ms).max().unwrap_or(0);
```

`count` responde "cuántos sobreviven al filter", `max` responde "cuál es el presupuesto más grande", y las dos drenan la cadena sin construir ninguna colección. (Ese `unwrap_or` no es un descuido: `max` devuelve un `Option` porque una lista vacía no tiene máximo, y cero es una respuesta defendible para eso, la regla de m04-l2 sobre los defaults que puedes defender en un comentario.)

Dos trampas antes del lab. `collect` necesita un tipo de destino, porque puede construir un Vec, un HashMap, un String y más a partir de la misma cadena; deja el tipo afuera y el compilador te frena con E0282, `type annotations needed`. Anota el binding como arriba, o usa el turbofish, `collect::<Vec<_>>()`. Ese error es el clásico rito de iniciación del primer pipeline, y ahora es información, no obstrucción. Segundo: `iter()` pide prestado. Tus closures ven `&Target`, que es por lo que `to_probe_target` toma `&self` y clona las cadenas que se queda, las reglas de m04-l1 sosteniendo la línea en silencio.

**Profundiza (el 20%).** esta lección enseñó el derive, la representación etiquetada y los cuatro atributos que de verdad vas a escribir este año. El resto de serde, impls `Deserialize` personalizados, el modelo de datos, deserialización zero-copy, cada atributo, vive en [https://serde.rs/](https://serde.rs/), el libro propio del crate; su página de representaciones de enum es el mapa canónico de etiquetado contra untagged y de los dos estilos que nos saltamos. Para closures e iteradores con la historia completa de pereza y rendimiento, el capítulo del Book es [https://doc.rust-lang.org/book/ch13-00-functional-features.html](https://doc.rust-lang.org/book/ch13-00-functional-features.html), y el remate de ahí vale el viaje: los iteradores compilan al mismo código que el loop hecho a mano. Las dos URLs verificadas en vivo el 2026-09-02. El lab no necesita nada del material de bookmark.

## Lab: un archivo, dos parsers

Numerado, los apoyos adelgazando a medida que bajas. Los pasos 1 y 2 los hacemos juntos, el paso 3 te da firmas, el paso 4 es un ejercicio que corres solo.

1. **Crea la config raíz, después parséala (trabajado).** Dos jugadas. Primero el archivo mismo: en la raíz del repo de la estación, un nivel arriba de `pulse-rs/` (la barrera de m04-l3 movió `pulse-rs/` adentro del repo de la estación), crea `pulse.config.json` con exactamente el contenido de dos targets impreso en el overview, `fleetName: "pulse-prod"`, `docs` y `api`. Este es un archivo nuevo en una dirección nueva, la config a nivel de flota que resucitó la sección de teoría; nada en `packages/pulse-fleet` se mueve ni cambia. Después el lado de Rust: en `pulse-rs`, crea `src/config.rs` con los primeros structs `Config` y `Target` del overview, las formas v1 con `url` directo sobre `Target`, más `parse_config` y la variante `BadConfig` agregada a `ProbeError` en `src/engine.rs`. No copies nada adentro de `pulse-rs`: desde `pulse-rs/`, un symlink mantiene funcionando la lectura relativa al cwd de abajo: `ln -s ../pulse.config.json pulse.config.json` (o simplemente lee `"../pulse.config.json"` directo; el punto es un archivo, no dos copias). Main temporal, y fíjate en que pone en línea la misma llamada a serde que `parse_config` envuelve; eso es deliberado, el ejercicio de frontera del paso 4 es donde `parse_config` toma el mando, así que la función que acabas de escribir queda brevemente sin usar en vez de mal puesta:

   ```rust
   mod config;
   mod engine;

   use engine::ProbeError;
   use std::fs;

   fn main() -> anyhow::Result<()> {
       let raw = fs::read_to_string("pulse.config.json")?;
       let config: config::Config =
           serde_json::from_str(&raw).map_err(ProbeError::BadConfig)?;

       println!("fleet \"{}\": {} target(s)", config.fleet_name, config.targets.len());
       for t in &config.targets {
           println!(
               "  {} -> {} every {}s, timeout {}ms",
               t.name, t.url, t.interval_secs, t.timeout_ms
           );
       }
       Ok(())
   }
   ```

   Una expectativa fijada antes de que lo corras: este main temporal deja huérfana toda la superficie del motor de m04, así que el build llega vistiendo un muro de warnings de dead-code, un par de docenas, todos de una lección de largo. Esperado, y temporal: el split de workspace de la próxima lección devuelve el motor a un crate consumido. Simplemente no hagas push hasta entonces, porque el CI de la estación corre `cargo clippy -- -D warnings` y contaría cada uno de ellos como un error. `cargo run` debería imprimir:

   ```text
   fleet "pulse-prod": 2 target(s)
     docs -> https://example.com every 60s, timeout 3000ms
     api -> https://example.org/health every 30s, timeout 2000ms
   ```

   Ahora dale su pluma al lado de TS. El `configSchema` de ráfaga no puede firmar este archivo (forma equivocada, y es estructural para `fleet.ts` y las pruebas, así que queda intacto); la config raíz recibe su propio módulo. Crea `packages/pulse-fleet/src/root-config.ts` con el `rootTargetSchema`, el `rootConfigSchema` y el `RootConfig` del overview (más `import { z } from "zod";` arriba), y un verificador de cuatro líneas al lado, `src/check-root-config.ts`, la jugada de m02-l2 repetida:

   ```ts
   import { readFileSync } from "node:fs";
   import { parseOrExit } from "./config.js";
   import { rootConfigSchema, type RootConfig } from "./root-config.js";

   const path = process.argv[2] ?? "../../pulse.config.json";
   const raw: unknown = JSON.parse(readFileSync(path, "utf8"));
   const config: RootConfig = parseOrExit(rootConfigSchema, raw);
   console.log(`fleet "${config.fleetName}": ${config.targets.length} target(s)`);
   ```

   Córrelo desde `packages/pulse-fleet` con `npx tsx src/check-root-config.ts ../../pulse.config.json`, y deja que el momento aterrice: dos lenguajes, dos sistemas de tipos, un archivo, las dos fronteras rechazando basura. Checkpoint: los dos comandos en verde sobre los mismos bytes.

2. **Evoluciona el contrato (guiado).** Mete el enum `ProbeKind` etiquetado y el `Target` evolucionado del overview, con `flatten` y el default de `enabled`. El compilador objeta antes de que serde tenga su turno: el `main` temporal todavía imprime `t.url`, y `url` ahora vive adentro de la variante `Http`. Cambia ese println para que muestre `t.kind` con `{:?}` en vez de la url, y reconstruye. Ahora `cargo run` otra vez, contra el archivo SIN CAMBIOS, y lee tu primera falla de evolución de schema:

   ```text
   Error: config rejected: missing field `kind` at line 9 column 5
   ```

   (El prefijo `Error:` es de anyhow, imprimiendo la falla que tu `?` devolvió desde `main`.)

   El parser derivado es un contrato con la forma, y acabas de cambiar el contrato sin avisarle al archivo. Así que avísale al archivo: agrega `"kind": "http"` a los dos targets existentes, y agrega un tercer target que la flota de TS nunca vio:

   ```json
   {
     "kind": "tcp",
     "name": "rpc",
     "host": "127.0.0.1",
     "port": 8899,
     "intervalSecs": 30,
     "timeoutMs": 1000,
     "enabled": false
   }
   ```

   Viene con `"enabled": false` porque ningún brazo de la estación puede correr todavía un chequeo de socket, y una config que nombra una sonda que nadie puede correr debería decirlo. (Ese puerto es donde un validador local de Solana responde RPC, una puerta a la que tocamos mucho más adelante en el curso.) Rómpelo a propósito antes de arreglarlo: pon `"kind": "grpc"` en esa entrada y corre:

   ```text
   Error: config rejected: unknown variant `grpc`, expected `http` or `tcp` at line 26 column 5
   ```

   Un rechazo nombrado que lista las variantes legales. Compara eso con el error untagged del overview y tienes el argumento entero de etiquetado contra untagged en dos líneas de salida de terminal. Vuelve a dejar el kind como estaba. Un parser sigue objetando, igual: `check-root-config.ts` ahora rechaza el archivo, `Unrecognized key: "kind"`, y tiene RAZÓN al hacerlo, esa es la estrictez de `strictObject` que pediste haciendo su trabajo sobre un contrato evolucionado. Los dos firmantes vuelven a firmar o nadie entrega. En `src/root-config.ts`, actualiza `rootTargetSchema` a una unión discriminada, la forma que ya conoces de `ProbeResult`:

   ```ts
   const baseFields = {
     name: z.string().min(1),
     intervalSecs: z.number().int().positive(),
     timeoutMs: z.number().int().positive(),
     enabled: z.boolean().default(true),
   };

   export const rootTargetSchema = z.discriminatedUnion("kind", [
     z.strictObject({ kind: z.literal("http"), url: z.url(), ...baseFields }),
     z.strictObject({
       kind: z.literal("tcp"),
       host: z.string().min(1),
       port: z.number().int().min(1).max(65535),
       ...baseFields,
     }),
   ]);
   ```

   `z.discriminatedUnion("kind"...)` y `#[serde(tag = "kind")]` son la misma máquina en lados opuestos del archivo. Checkpoint: `cargo run` imprime config por valor de tres targets, y `check-root-config.ts` acepta el mismo archivo otra vez.

![Evolucionar la config compartida rompe el parser de Rust, después el parser de zod, hasta que los dos schemas vuelven a firmar el contrato nuevo y se ponen en verde.](assets/v07-flowchart.webp)

3. **El pipeline (tuyo).** Conecta `ProbeTarget` y la cadena filter-map-collect del overview a `main`, después alimenta el resultado al motor que construiste en m04-l3. Un retiro primero: `engine.rs` todavía exporta el `ProbeTarget { name, url }` esquelético de m04-l1, y el `ProbeTarget { name, endpoint, budget_ms }` del módulo de config es su reemplazo adulto. Dos tipos pub con un solo nombre en un solo crate es deriva esperando a pasar, así que borra el de m04 de `engine.rs` ahora; nada lo llama desde el main temporal del paso 1, y `cargo check` lo va a respaldar. El tipo nuevo y `Target::to_probe_target` pertenecen a `src/config.rs`, al lado de los structs de los que se derivan, que es exactamente donde la lista de re-exports de la próxima lección los espera. Después el pipeline: para cada sonda, maneja la máquina de estados sobre latencias de fixture con el `timeout_ms` del propio target como presupuesto. Las firmas de los closures son tu apoyo, los cuerpos y el cableado no: `filter` toma `|t: &&Target| -> bool` (referencia doble, `iter` presta y `filter` presta otra vez; `t.enabled` simplemente funciona a través de las dos), `map` toma `|t: &Target| -> ProbeTarget`. Escribe `to_probe_target(&self)` como un `match` sobre el kind: `Http` da la url como endpoint, `Tcp` formatea `host:port`. Mi `main` termina así; escribe el tuyo antes de comparar:

   ```rust
   let probes: Vec<ProbeTarget> = config
       .targets
       .iter()
       .filter(|t| t.enabled)
       .map(|t| t.to_probe_target())
       .collect();

   println!(
       "fleet \"{}\": probing {} of {} targets",
       config.fleet_name,
       probes.len(),
       config.targets.len()
   );

   for probe in &probes {
       let mut source = FixtureSource::new(vec![212, 487, 2400, 2600]);
       let state = drive(&mut source, probe.budget_ms);
       println!(
           "  {} -> {} settles {:?} (budget {} ms, fixture latencies)",
           probe.name, probe.endpoint, state, probe.budget_ms
       );
   }
   ```

   Salida del Checkpoint, vale leerla de cerca:

   ```text
   fleet "pulse-prod": probing 2 of 3 targets
     docs -> https://example.com settles Up (budget 3000 ms, fixture latencies)
     api -> https://example.org/health settles Degraded (budget 2000 ms, fixture latencies)
   ```

   Dos de tres: el filter descartó el target TCP deshabilitado, así que el pipeline es estructural, no decoración. Y las mismas cuatro latencias de fixture se asientan distinto bajo presupuestos distintos, 2400 y 2600 ms pasan por debajo del presupuesto de 3000 que tiene docs y revientan el de 2000 de api dos veces, que es la máquina de m04-l3 consumiendo config de verdad por primera vez. Di los límites en voz alta: las latencias siguen siendo fixtures, el motor todavía no sondea nada, y el brazo HTTP de verdad está a dos lecciones, en m05-l3.

4. **El ejercicio de frontera (solo).** Copia la config a `pulse.config.broken.json` y pon el `timeoutMs` de la entrada api en `"fast"`. Pon la copia donde sea que la lectura de abajo la vaya a encontrar, que depende de cuál de los dos cableados del paso 1 tomaste: si hiciste el symlink, la copia rota va en `pulse-rs/` al lado de él; si estás leyendo `"../pulse.config.json"` directo, la copia va en la raíz del repo y la ruta del snippet se vuelve `"../pulse.config.broken.json"`. Equivócate en esto y `?` te da un NotFound de `anyhow` antes de que se imprima siquiera el mensaje de rechazo que este ejercicio existe para mostrar. Después haz que `main` cargue la copia rota DESPUÉS de la de verdad, reportando la falla sin morirse:

   ```rust
   let broken = fs::read_to_string("pulse.config.broken.json")?;
   if let Err(e) = parse_config(&broken) {
       println!("broken copy refused: {e}");
   }

   println!("run complete");
   ```

   Cola esperada de la ejecución:

   ```text
   broken copy refused: config rejected: invalid type: string "fast", expected u64 at line 16 column 25
   run complete
   ```

   La línea y la columna van a coincidir con donde sea que tu editor haya puesto ese campo; lo que importa es que estén AHÍ, en un valor de error que ruteaste, impreso por una ejecución que después siguió andando. ¿Reconoces la forma? Es el ejercicio de config rota de m02-l2, vistiendo la otra bandera. **Verifica antes de seguir**: `cargo run` imprime el resumen de flota de tres targets con dos sondeados, el rechazo de la copia rota con línea y columna, y `run complete`. Ese es el contrato entero de esta lección en una pantalla.

## Challenge

Ahora anda a terminar lo que empezó la apertura: `kv-config-parser`, totalmente sin guía, en el panel de coding-challenge. El parser del starter es un mentiroso: parte en cada `=`, se traga las claves desconocidas, inventa defaults donde debería rechazar. Vuélvelo honesto: partir en el primer `=` vía `split_once`, una allowlist de claves, validación del scheme de url y del timeout, claves faltantes reportadas en el orden fijo name, url, timeout_ms, cada falla un `Err`, nada de unwrap en el camino del parseo. Ocho pruebas lo califican, incluyendo la URL con query string que castiga el partido perezoso y el punto y coma final que castiga la iteración perezosa. Las pistas escalan desde `split_once` hasta el final de `ok_or_else` para las claves faltantes; gástalas en orden. Este es deliberadamente el último parser hecho a mano que escribes en este curso, que es exactamente por qué vale la pena escribirlo bien: después de él, sabes hasta la línea lo que hace por ti cada derive futuro.

## Checkpoint

Lo que ahora puedes hacer, concretamente: convertir una definición de struct en un parser de JSON con un solo derive y leer los cuatro atributos que cargan el uso diario de serde; modelar un formato discriminado como un enum etiquetado y explicar, con dos mensajes de error como evidencia, por qué etiquetado le gana a untagged en los formatos de los que eres dueño; rutear una falla de parseo a través de tu taxonomía de errores como un valor con línea y columna adjuntas; y transformar una lista parseada con filter, map y collect, sabiendo que nada corre hasta que el consumidor jala. Una nota de interfaz para tu yo futuro: `parse_config(&str) -> Result<Config, ProbeError>` y `Target::to_probe_target` ahora son parte de la superficie pública del motor. La próxima lección los mueve a un crate de librería, y lecciones posteriores los llaman con exactamente estas firmas, así que resiste las ganas de "ordenarlas" de acá a entonces.

La recuperación de 30 segundos antes de cerrar la pestaña, en voz alta: ¿zod infiere el qué desde el schema, y serde deriva el qué desde el tipo? (El tipo; el parser. Direcciones opuestas, una disciplina.) ¿Y qué palabra sola explica por qué tu cadena no hizo trabajo antes de `collect`? (Perezosa.)

Reporte de fricción, mientras está fresco: ¿la falla de evolución de schema en el paso 2 del lab se sintió como la lección rompiéndose o la lección aterrizando? Ese beat está diseñado para picar, la idea del contrato-con-una-forma no se pega sin él, pero hay una versión que se lee como pura vuelta en círculos, y tu reporte es cómo me entero de cuál se entregó. Lo mismo con la referencia doble en `filter`; si `|t: &&Target|` te costó más de un minuto, dilo.

Tu motor ahora lee la config de verdad de la flota a través de un parser que no tuviste que escribir, y todo vive en un solo crate, que está por volverse el problema. La CLI que haces crecer después necesita el motor como librería, y un binario de release no debería arrastrar fixtures de prueba con él. Próxima lección: workspaces de cargo, editions, features y leer los pins de versión de otra gente, Cargo.toml como una negociación con cada máquina que vaya a construir tu código. Trae tu manifest; cada línea de él está por querer decir algo.
