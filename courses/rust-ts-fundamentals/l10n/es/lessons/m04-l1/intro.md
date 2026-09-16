# El ownership es el punto

## Resumen

M3 cerró el tier de TypeScript: `pulse-core` está publicado en npm, el panel está en vivo en Vercel, y la barrera del tier nombró exactamente lo que salteamos. La mitad de TS de la estación está entregada y con barrera. Ahora empieza el segundo lenguaje, y empieza con una pelea. En menos de diez minutos vas a instalar el toolchain de Rust, armar `pulse-rs`, pegar cinco líneas inocentes, y ser rechazado por un compilador por código que TypeScript correría sin pestañear. Toda la lección es por qué ese rechazo es la feature por la que viniste. Un aviso sobre cómo funciona este módulo: M4 corre con la textura más fina de todo el curso. Todo acá es o trabajado conmigo o repara-este-snippet. Nunca escribes desde un archivo en blanco; la única rep sin guía es el challenge del final.

## Rompe algo primero

Instala el toolchain. rustup es el instalador, todo el paquete, un comando en macOS o Linux:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

En Windows, descarga y corre `rustup-init.exe` desde el mismo sitio. Reinicia tu shell, y después confirma:

```bash
rustc --version
cargo --version
```

Deberías ver rustc 1.98.0 o más nuevo (1.98.0 pasó a estable el 2026-08-20; verificado el 2026-09-02, y como cada seis semanas sale una estable nueva, tu dígito puede ser ya más alto, y eso está bien). Ahora arma la mitad de Rust de la estación y córrela. Hazlo desde la raíz del repo de tu estación, el que tiene `packages/` y `pnpm-workspace.yaml`, porque `pulse-rs` es la mitad de Rust de la estación, no un proyecto aparte: vive adentro del repo, al lado de `packages/`, y m04-l3 lo cablea al CI propio de la estación sobre exactamente esa suposición.

```bash
cargo new pulse-rs
cd pulse-rs
cargo run
```

Hello, world. Cargo es npm, tsc y vitest en un solo binario, y lo vamos a recorrer como corresponde en una lección posterior. Hoy existe para compilar tu primer rechazo. Reemplaza todo en `src/main.rs` con esto:

```rust
fn sum_latencies(latencies: Vec<u64>) -> u64 {
    latencies.iter().sum()
}

fn main() {
    let latencies = vec![212, 487, 1204];
    let total = sum_latencies(latencies);
    println!("total: {total} across {} probes", latencies.len());
}
```

Léelo como la persona que desarrolla en TypeScript que ahora eres: haz un array, pásalo a un helper, imprime la suma y el largo. En TS esto es un martes. Corre `cargo check` (compila sin producir un binario, tu loop de feedback rápido de acá en adelante):

```text
error[E0382]: borrow of moved value: `latencies`
 --> src/main.rs:8:49
  |
6 |     let latencies = vec![212, 487, 1204];
  |         --------- move occurs because `latencies` has type `Vec<u64>`,
  |                   which does not implement the `Copy` trait
7 |     let total = sum_latencies(latencies);
  |                               --------- value moved here
8 |     println!("total: {total} across {} probes", latencies.len());
  |                                                 ^^^^^^^^^ value borrowed here after move
```

E0382. Uso de un valor movido. El compilador rechazó un programa que correría correctamente, hoy, en tu máquina, en cualquier lenguaje con GC. Quédate un segundo con lo irrazonable que se siente eso, porque el resto de esta lección es el argumento de que es la cosa más razonable que un compilador te ha hecho jamás. Deja el error donde está. Lo arreglamos en el lab, a propósito, por la vía con principios.

## Las reglas, derivadas del problema

### Alguien tiene que liberarla

Empieza por el hecho con el que vive todo lenguaje: cuando tu programa hace un `Vec` de latencias, se asigna memoria, y en algún momento esa memoria tiene que devolverse. Por alguien. No hay una cuarta opción, solo tres respuestas a "¿quién?"

Respuesta uno: un recolector de basura. Node, Java, Go, Python. Un runtime vigila tus objetos, averigua a cuáles ya no apunta nada, y los libera. Escribes código como si la memoria fuera infinita, y pagas por la ilusión en runtime: el recolector consume CPU, y pausa tu programa según su propio cronograma, no el tuyo. Para el panel de la flota ese costo es invisible. Para un cliente validador o un sistema de trading, una pausa en el milisegundo equivocado es dinero real, que es una razón honesta de por qué tanto del stack de Solana es Rust.

Respuesta dos: tú. C y C++. Llamas a `free` cuando terminas, y el compilador te confía por completo. Los modos de falla de esa confianza tienen nombres que escuchaste incluso si nunca escribiste C: use-after-free, double-free, dangling pointer. Décadas de avisos de seguridad son los recibos.

Respuesta tres, la respuesta de Rust: hacer de "¿quién libera esto?" una propiedad del código mismo, decidida en tiempo de compilación. Cada valor tiene exactamente un dueño, la variable responsable de él. Cuando el dueño sale de scope, el valor se libera, de forma determinista, sin ningún recolector involucrado. Esa es la primera regla del ownership (la "propiedad", si quieres el término en español), y fíjate en que no la memorizaste, la derivaste: si la limpieza tiene que pasar exactamente una vez sin ningún runtime vigilando, exactamente un binding tiene que estar en el anzuelo.

![Tres columnas comparan la recolección de basura, la liberación manual y el ownership en tiempo de compilación como respuestas a quién libera la memoria asignada.](assets/v01-comparison.webp)

### El move, y por qué murió tu snippet

Sigue la regla hasta el snippet. `let latencies = vec![...]` hace de `latencies` el dueño. Después `sum_latencies(latencies)` pasa el `Vec` por valor, y acá Rust hace algo que TS nunca te hizo pensar: el ownership se transfiere. El parámetro adentro del helper es el nuevo dueño; va a liberar la memoria cuando la función termine. ¿Entonces qué es `latencies` en `main` después de esa línea? Si Rust te dejara seguir usándolo, dos bindings creerían los dos que son dueños de la misma asignación. Dos dueños significa o limpieza doble, liberar la misma memoria dos veces, o mutación ambigua, dos lugares con derecho a cambiar una cosa sin que el otro sepa. Así que el acceso del binding viejo simplemente termina. Eso es un move, y E0382 es el compilador diciéndote, con precisión: este valor tiene un dueño nuevo, tu nombre ya no está en él.

Fíjate en lo que el error NO está diciendo. Nada se liberó temprano. Los datos están vivos y sanos adentro del helper. El rechazo no es sobre memoria colgando; es sobre la ambigüedad, sobre que la pregunta "¿quién es dueño de esto?" tenga momentáneamente dos respuestas. Toda la apuesta de Rust es que la ambigüedad, no la asignación, es donde están enterrados los cadáveres.

![Un vector sigue vivo mientras el ownership pasa de un binding a un parámetro de función, y el binding viejo queda tachado.](assets/v02-diagram.webp)

### Pedir prestado: úsalo sin ser su dueño

El helper nunca quiso ser dueño de las latencias. Quería leerlas un momento y devolverlas. Rust tiene una palabra exactamente para eso: un borrow. Escribe `&latencies` y le pasas a la función una referencia, permiso de lectura, ownership sin mover. La firma del helper declara que acepta datos prestados tomando `&[u64]`, un slice (una rebanada), que es "una vista sobre una tira de u64s" y la forma estándar de aceptar datos de lista prestados (un `&Vec<u64>` se convierte a eso automáticamente, así que una sola firma le sirve a todos). Esta es la jugada de uso diario de todo el lenguaje. Cuando miras a quienes programan Rust de verdad, la abrumadora mayoría de los parámetros de función son borrows, porque la mayoría de las funciones son invitadas, no herederas.

Después hay una segunda clase de borrow, y con ella la segunda regla, que también puedes derivar en vez de memorizar. Supón que una parte de tu código tiene `let worst = &latencies[2];`, un lector, mientras otra llama a `latencies.push(90)`, un escritor. Un push puede reasignar el buffer del Vec, moviendo cada elemento a una dirección nueva, y en ese punto `worst` apunta a memoria liberada. En C eso también es un martes, del tipo segfault. ¿Entonces cuál tiene que ser la regla, si esto se va a atrapar en tiempo de compilación? Los lectores y los escritores no pueden superponerse. Cualquier cantidad de borrows compartidos (`&`), O exactamente un borrow exclusivo (`&mut`), nunca los dos a la vez. Intenta la superposición y recibes al hermano de tu apertura:

```text
error[E0502]: cannot borrow `latencies` as mutable because it is also borrowed as immutable
```

Piensa en una planilla compartida: cualquier cantidad de personas puede verla a la vez, pero en el momento en que alguien tiene el lock de edición, o quienes miran ven un estado congelado consistente o nadie edita. La analogía se rompe en un lugar que vale nombrar: la planilla hace cumplir el lock en runtime, mientras Rust lo hace cumplir antes de que el programa exista, que es por qué la misma regla que salva a `worst` de un buffer reasignado es también, en código multithread, la regla que hace irrepresentables las carreras de datos. Un escritor XOR muchos lectores es la regla de carreras de datos vestida de un solo hilo de ejecución, y acabas de conocerla en un archivo de cinco líneas.

![Dos estados permitidos muestran muchos lectores o un escritor sobre un vector, y un tercer estado superpuesto es rechazado por el compilador.](assets/v03-diagram.webp)

Así que las tres reglas, ninguna recitada, todas forzadas por el problema: cada valor tiene un dueño; el ownership se mueve cuando cedes el valor mismo; los borrows te dejan prestar acceso, muchos lectores XOR un escritor. Acá va la síntesis, y es el asa de este módulo, así que quédatela: el verificador de préstamos es el code review que no puedes saltarte. Cada rechazo de esta lección es un comentario que una persona senior y cuidadosa habría dejado en tu PR, "¿quién es dueño de esto después de la línea 7?", "estás mutando una lista que otro está leyendo". El verificador no te está impidiendo hacer la cosa. Te está impidiendo hacer la cosa de forma ambigua. Ya entregas TypeScript que funciona; no te están degradando, te están dando el review más temprano, de parte de alguien que nunca se cansa.

Y este revisor es un proyecto vivo, no una especificación congelada. Polonius-alpha, una formulación de nueva generación del verificador de préstamos que acepta más programas correctos, aterrizó en nightly el 2026-08-04, y el solver de traits de nueva generación siguió el 2026-08-21, diecisiete días después. Las peleas que hoy pierdes en los márgenes son peleas que el verificador está aprendiendo a conceder donde tenías razón desde el principio. El code review que no puedes saltarte se está volviendo más inteligente.

### La caja que te dio rustup

Corto y honesto, porque vas a escuchar mitos. El toolchain estable que acabas de instalar no es un compilador pelado. clippy (el linter) y rustfmt (el formateador) llegan instalados con el perfil default de rustup, y rust-analyzer (el motor de IDE con el que habla tu editor) también es un componente de rustup del canal estable, aunque ese está a un `rustup component add rust-analyzer` de distancia en vez de venir preinstalado, y las extensiones de Rust de la mayoría de los editores se bajan su propia copia igual. Herramientas de primera parte, no add-ons de terceros. La única herramienta que la gente espera en la caja y no recibe es miri, el intérprete que atrapa comportamiento indefinido en código unsafe: miri es solo de nightly. No la vas a necesitar en este curso, y ahora no vas a ir a instalar un sustituto de terceros para herramientas que ya tienes. Dos hábitos empiezan hoy y no paran nunca: `cargo fmt` antes de commitear, `cargo clippy` antes de empujar. Son el Prettier y el ESLint de este lenguaje excepto que nadie debate la config.

![Una línea de tiempo fechada muestra dos mejoras del verificador de préstamos en nightly, el release estable actual, la fecha de verificación de esta lección, y el próximo release esperado.](assets/v04-timeline.webp)

### Lo que esto te cuesta

El trade-off, nombrado sin adornos: pagas en peleas. El verificador de préstamos rechaza programas que un lenguaje con GC correría feliz y correctamente, y quienes empiezan pierden horas reales reestructurando código que "estaba bien". Esas horas te compran cero pausas de GC, cero use-after-free, cero carreras de datos, y una limpieza que pasa en una línea que puedes señalar. Si ese trade-off vale la pena depende de lo que construyas; para la infraestructura sobre la que corre este stack, la industria ya votó.

Y sabe cuándo no pelear. `.clone()` hace una copia independiente con su propio dueño, y un clone en código frío, un struct de config copiado una vez al arrancar, muchas veces es la decisión de ingeniería correcta, no una derrota. El pecado es el clone reflejo: aparece E0382, espolvoreas `.clone()`, compila, y no aprendes nada. Yo cloné mi camino por mis primeras semanas de Rust, a lo grande, y el hábito me costó dos veces, una en asignaciones y una en nunca escuchar lo que el verificador me estaba tratando de decir. En este módulo la escotilla de escape del clone está nombrada y después cerrada con llave: cada reparación de abajo tiene que ser un arreglo basado en borrows, y la regla de no-clone del challenge es una que haces cumplir tú mismo (sus pruebas califican entradas y salidas, así que una solución con clone las pasaría, y no demostraría nada).

**Profundiza (el 20%).** esta lección te enseñó por qué existe el ownership y las jugadas del día a día, move, `&`, `&mut`, y el rol honesto del clone. La progresión completa, stack contra heap, cómo está dispuesto un String, la mecánica de los slices, vive en el capítulo de ownership del Book, y la versión para leer es el fork interactivo de Brown University, con quizzes incorporados y visualizaciones de Aquascope que animan exactamente los moves y los borrows que acabas de conocer: [https://rust-book.cs.brown.edu/ch04-00-understanding-ownership.html](https://rust-book.cs.brown.edu/ch04-00-understanding-ownership.html). Ese fork existe porque el ownership es lo bastante difícil como para haber generado un programa de investigación académica: el Cognitive Engineering Lab de Brown lo construyó sobre investigación revisada por pares de OOPSLA 2023 y 2024 sobre cómo la gente aprende de verdad este modelo. Déjalo como bookmark, haz ch04 con los quizzes esta semana. Y después de cada lección de M4, Rustlings (`cargo install rustlings`, después `rustlings init` y `rustlings`) es el patio de ejercicios: ejercicios chicos de arregla-el-código, la misma forma de loop de completion que este módulo, mantenidos por el propio proyecto Rust. El lab de abajo no necesita nada del material que quedó como bookmark.

## Lab: espeja la flota en Rust

Empieza la reescritura. `pulse-rs` se vuelve el gemelo en Rust de los tipos de tu motor de sondeo, y el TypeScript de M2 va a pantalla justo al lado del Rust nuevo, porque ya diseñaste estos tipos una vez y me niego a pretender lo contrario.

1. **Arregla la apertura, por la vía con principios.** La propia nota del compilador ya te lo dijo: el helper debería pedir prestado. Cambia la firma a un slice y presta en el sitio de la llamada:

   ```rust
   fn sum_latencies(latencies: &[u64]) -> u64 {
       latencies.iter().sum()
   }

   fn main() {
       let latencies = vec![212, 487, 1204];
       let total = sum_latencies(&latencies);
       println!("total: {total} across {} probes", latencies.len());
   }
   ```

   `cargo check` pasa a verde, `cargo run` imprime `total: 1903 across 3 probes`. Dos caracteres de puntuación, y `main` sigue siendo dueño de su Vec, lo presta una vez, y lo usa después. Ese es el patrón de arreglo que vas a aplicar todo el módulo: no "haz que el error desaparezca" sino "di quién es dueño y quién pide prestado".

2. **Pon la especificación de TS en pantalla.** Abre tu `pulse-core` de M2 al lado de `pulse-rs`. Este es el momento lado-a-lado; acá va el TypeScript que entregaste, menos un brazo. La unión real de tu flota carga una cuarta variante, `{ kind: 'dns-error'; host: string }`, la que agregó el ejercicio de m02-l1; el port de hoy espeja las tres de abajo y deja dns-error afuera a conciencia, el mismo recorte que hace cualquier port cuando arranca del núcleo estructural:

   ```ts
   type ProbeResult =
     | { kind: 'ok'; latencyMs: number }
     | { kind: 'timeout'; budgetMs: number }
     | { kind: 'http-error'; status: number };

   type Verdict = 'up' | 'degraded' | 'down';
   ```

3. **Modela la misma verdad en Rust.** Reemplaza `src/main.rs` con los tipos, variante por variante. Dos recién llegados: un struct para el target de sondeo (un tipo registro, campos y nada más) y el patrón newtype, `LatencyMs(u64)`, un struct de un solo campo que no cuesta nada en runtime pero te impide para siempre pasar un número de puerto donde va una latencia. En M2 compraste esta seguridad con tipos literales; acá es un wrapper gratis:

   ```rust
   #[derive(Debug)]
   struct ProbeTarget {
       name: String,
       url: String,
   }

   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   struct LatencyMs(u64);

   #[derive(Debug)]
   enum ProbeResult {
       Ok { latency: LatencyMs },
       Timeout { budget: LatencyMs },
       HttpError { status: u16 },
   }

   #[derive(Debug, PartialEq, Eq)]
   enum Verdict {
       Up,
       Degraded,
       Down,
   }
   ```

   Mira `ProbeResult` al lado de su gemelo de TS. Un enum de Rust ES tu unión discriminada, excepto que el discriminante no es un campo `kind` que mantienes por convención, es el nombre de la variante en sí, hecho cumplir por construcción. Las líneas `#[derive(...)]` le piden al compilador que escriba boilerplate por ti; `Debug` es lo que permite que `{:?}` imprima un valor, y `Copy` sobre `LatencyMs` lo marca como lo bastante barato para copiar en vez de mover, que es por qué un `u64` nunca te da E0382 pero un `Vec` sí. El mensaje de error de la apertura decía exactamente eso, ve a releer su segunda línea. Una arruga honesta que quizá hayas notado: el budget de `Timeout` también lleva `LatencyMs`, aunque un budget es una duración que elegiste, no una latencia que mediste. Esa reutilización es una economía deliberada (un newtype de milisegundos para un archivo de cinco tipos), no una afirmación de categoría; el día en que los dos roles se encuentren en una misma firma, un segundo newtype, `BudgetMs`, es el mismo argumento de puerto-contra-latencia aplicado a nosotros mismos.

![Una unión discriminada de TypeScript y un enum de Rust están lado a lado con líneas que emparejan sus tres variantes coincidentes.](assets/v05-annotated-code.webp)

4. **Porta el clasificador, puro y alimentado por fixtures.** Ahora la función alrededor de la cual orbita toda la estación. Las mismas bandas que congelaste en M2: menos de 400 es up, de 400 a 1000 inclusive es degraded, arriba de 1000 es down, y un 429 significa que el target está vivo pero cansado, así que degraded. Agrega debajo de los tipos:

   ```rust
   fn classify_latency(latency: LatencyMs) -> Verdict {
       let ms = latency.0;
       if ms < 400 {
           Verdict::Up
       } else if ms <= 1000 {
           Verdict::Degraded
       } else {
           Verdict::Down
       }
   }

   fn classify_probe(result: &ProbeResult) -> Verdict {
       match result {
           ProbeResult::Ok { latency } => classify_latency(*latency),
           ProbeResult::Timeout { .. } => Verdict::Down,
           ProbeResult::HttpError { status } => {
               if *status == 429 {
                   Verdict::Degraded
               } else {
                   Verdict::Down
               }
           }
       }
   }
   ```

   Tres cosas se ganan su por qué acá. Sin `return` y sin punto y coma en las líneas de cola: la última expresión de un bloque es su valor, esa es simplemente la forma de Rust. `match` es tu switch exhaustivo con `assertNever` incorporado al lenguaje: borra el brazo `Timeout` y `cargo check` rechaza todo el programa, la demostración negativa que extraías a mano en M2 ahora es el default. Y `classify_probe` toma `&ProbeResult`, un borrow, porque un clasificador es un invitado: lee, responde, no es dueño de nada. Di la pureza en voz alta también: este motor no llama a nada, no hace fetch de nada, y eso no es una disculpa de placeholder. Su brazo de HTTP real llega en M5, y la pureza es exactamente lo que le permite a este mismo motor compilar a WASM en M7 y correr en el edge. Alimentado por fixtures hoy, a propósito.

5. **Aliméntalo con un fixture, y choca con el tercer rechazo del módulo.** Escribe un `main` que espeje la idea del `describe` de M2, y después clasifique un fixture. Tipéalo exactamente así primero, con `for result in fixture`:

   ```rust
   fn describe(result: &ProbeResult) -> String {
       match result {
           ProbeResult::Ok { latency } => format!("{}ms", latency.0),
           ProbeResult::Timeout { budget } => format!("no answer in {}ms", budget.0),
           ProbeResult::HttpError { status } => format!("HTTP {status}"),
       }
   }

   fn main() {
       let target = ProbeTarget {
           name: String::from("solana-rpc"),
           url: String::from("https://api.mainnet.solana.com"),
       };

       let fixture = vec![
           ProbeResult::Ok { latency: LatencyMs(212) },
           ProbeResult::Ok { latency: LatencyMs(487) },
           ProbeResult::Timeout { budget: LatencyMs(3000) },
           ProbeResult::HttpError { status: 429 },
           ProbeResult::Ok { latency: LatencyMs(1204) },
       ];

       println!("target: {} ({})", target.name, target.url);
       for result in fixture {
           println!("{} -> {:?}", describe(&result), classify_probe(&result));
       }
       println!("probes classified: {}", fixture.len());
   }
   ```

   `cargo check`: E0382 otra vez, y este es más furtivo que el de la apertura. `for result in fixture` consume el Vec, el loop toma el ownership y se lo come elemento por elemento, así que el `fixture.len()` de después es uso-después-de-move. El arreglo es la misma idea de siempre, presta en vez de ceder: recorre `&fixture`, y en ese punto cada `result` ya es una referencia y los dos argumentos `&result` se simplifican:

   ```rust
       for result in &fixture {
           println!("{} -> {:?}", describe(result), classify_probe(result));
       }
       println!("probes classified: {}", fixture.len());
   ```

6. **Verifica.** La barrera de aceptación para el artefacto de esta lección:

   ```bash
   cargo fmt
   cargo clippy
   cargo check
   cargo run
   ```

   `cargo check` tiene que pasar con cero errores y, tal como está escrito acá, cero warnings (corrí este archivo exacto hoy, las dos verificaciones limpias). `cargo run` imprime:

   ```text
   target: solana-rpc (https://api.mainnet.solana.com)
   212ms -> Up
   487ms -> Degraded
   no answer in 3000ms -> Down
   HTTP 429 -> Degraded
   1204ms -> Down
   probes classified: 5
   ```

   Pon esa salida al lado de lo que dice tu clasificador de TS para los mismos cinco inputs. Los mismos veredictos, variante por variante. El segundo cuerpo del motor está vivo.

![El núcleo clasificador puro construido hoy fluye sin cambios hacia un futuro servicio HTTP y un build de WASM en el edge.](assets/v06-flowchart.webp)

### Reps de reparación: tres borrows de maldad creciente

El loop de completion, la textura más fina. Cada snippet de abajo no compila, cada uno se arregla con UN cambio con principios, y el clone es la escotilla de escape nombrada que esquiva la lección, así que está prohibido. Trabaja en un archivo borrador (`cargo new borrow-reps` si quieres un banco limpio). Predice el error antes de correr `cargo check`, y después lee lo que el compilador dice de verdad; la brecha de predicción es donde está el aprendizaje.

**Rep 1, el move hacia un helper.** El patrón de la apertura con ropa nueva. Arregla cambiando una firma y un sitio de llamada:

```rust
fn max_latency(latencies: Vec<u64>) -> u64 {
    latencies.iter().copied().max().unwrap_or(0)
}

fn main() {
    let latencies = vec![212, 487, 1204];
    let max = max_latency(latencies);
    println!("max {max} out of {} probes", latencies.len());
}
```

**Rep 2, el choque lector-escritor.** E0502 en el mundo real. Acá no hay firmas que cambiar; el arreglo es un reordenamiento, mover una línea para que el escritor termine antes de que el lector empiece:

```rust
fn main() {
    let mut latencies = vec![212u64, 487, 1204];
    let worst = &latencies[2];
    latencies.push(90);
    println!("worst so far: {worst}ms");
}
```

**Rep 3, el loop que se come su propia lista.** Este lo conociste en el lab; ahora cázalo tú mismo, y destructura mientras estás ahí: iterar un slice prestado te entrega referencias, y `for &l in` las desenvuelve para que el cuerpo trabaje con números pelados:

```rust
fn main() {
    let latencies = vec![212u64, 487, 1204];
    let mut slow = 0;
    for l in latencies {
        if l > 1000 {
            slow += 1;
        }
    }
    println!("{slow} slow out of {}", latencies.len());
}
```

Aceptación para las reps: las tres compilan, cero clones, y para cada una puedes decir en una oración CUÁL regla se disparó y POR QUÉ tu cambio la satisface, prestar, reordenar o reestructurar. Si una rep te llevó tres intentos, bien, esa era la cantidad calibrada de maldad. Una puerta que deliberadamente no estoy abriendo: en algún momento una respuesta de internet va a sugerir anotaciones de tiempo de vida para problemas con esta forma. Los tiempos de vida más allá de simplemente leerlos están fuera del alcance de este curso por diseño, la barrera del tier de M5 nombra dónde viven, y nada de M4 los necesita.

## Challenge

La rep sin guía, y es ella misma una reparación. El starter `fix-the-borrow` vive en el panel interactivo de coding-challenge de la página de esta lección (el editor en el navegador con su propio corrector y sus pistas, el mismo panel que usaron los challenges del lado de TypeScript; la mayoría de los challenges calificados de este curso viven ahí, aunque no todas las reps, y el de la lección que viene es un build local autocorregido). Te entrega un `latency_report` que mueve su Vec hacia `max_latency`, y después trata de pasarlo a `count_over`, que es el crimen de la apertura a escala de producción. Haz que los dos helpers pidan prestado `&[u64]`, presta `&latencies` en los sitios de llamada, y deja a `latency_report` siendo dueño de sus datos todo el camino, todavía imprimiendo el `"max=930,over=1"` esperado del fixture del starter. Ningún `.clone()` en ninguna parte, y sé honesto sobre quién revisa eso: las pruebas son pares de entrada-salida, así que una solución con `.clone()` pasa todas y esquiva la lección entera. Grepea tu propia solución buscando `clone` antes de darla por terminada; esa autoauditoría es la rep. Cinco pruebas, incluyendo la lista vacía (el max es 0, no un panic) y el límite estrictamente-mayor, porque un corrido en uno en un umbral es el tipo de bug que clasifica un RPC degradado como sano. Todo lo que necesitas son las tres reps que acabas de hacer; las pistas del starter escalan desde "qué llamada lo consumió" hasta la destructuración `for &l in`, gástalas en orden.

## Checkpoint

Lo que ahora puedes hacer, concretamente: instalar y verificar un toolchain de Rust y decir qué hay de verdad en la caja estable (y que miri no está); leer E0382 y E0502 como información sobre el ownership, no como obstrucción; y elegir entre mover, pedir prestado y pedir prestado en exclusiva a propósito, con el clone degradado de reflejo a decisión. Tu `pulse-rs` pasa `cargo check` con los tipos centrales de la flota espejados, tres de los cuatro brazos de la unión con dns-error conscientemente postergado, y lo hizo sin un solo clone.

La recuperación de 30 segundos antes de cerrar la pestaña, en voz alta: ¿cuáles son las dos cosas que previene un move? (La limpieza doble y la mutación ambigua, un dueño significa una respuesta a las dos.) ¿Y la regla del borrow en cinco palabras? (Muchos lectores XOR un escritor.)

Un pedido mientras está fresco: anota cuál rep te peleó más fuerte y qué dijo el compilador contra lo que predijiste. Esa brecha es información, para ti y para mí, y si la rep 2 en particular se sintió arbitraria, dímelo en el feedback, porque el arreglo por reordenamiento es el que más merece tiempo al aire y quiero saber si aterrizó. La textura de este módulo es un experimento de enseñar reparando; tus reportes de fricción son cómo se afina.

Tus tipos compilan y tu clasificador corre sobre fixtures. El fixture de hoy está construido en el código fuente, valores de enum tipados que no pueden estar malformados. La lección que viene el fixture se vuelve texto crudo parseado en una frontera, y la pregunta se pone en vivo: en el momento en que una línea esté malformada, ¿qué DEVUELVE el parser? No hay excepción que lanzar; Rust no las tiene. La lección que viene, el error ES el tipo de retorno, y el enum que construiste hoy resulta ser exactamente la máquina que hace que eso funcione. Trae el enum.
