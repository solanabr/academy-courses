# Prende los instrumentos: perfilar, depurar y cubrir el swap

La lección pasada te sentaste en el asiento del framework y viste qué cambia cuando un mint llega como Token-2022 en vez de SPL simple. Apuntaste un lector chico del lado del cliente a un mint provisto, leíste su largo real y su transfer hook dormido directo del wire, y después saliste a cazar un mint cuyo hook estuviera vivo. No se escribió código nuevo de programa. Tu programa de swap, R4 en el barcade de Quarters, sigue moviendo tokens de arcade para un lado y tickets para el otro. Funciona. Y todavía no mediste cuánto cuesta un solo canje.

Haz la pregunta honesta: ¿cuántas unidades de cómputo quema un canje? Toda respuesta que puedas dar hoy es un encogimiento de hombros con una estimación pegada. Este módulo entero es sobre canjear ese encogimiento de hombros por un número que midas tú mismo, usando el tooling first-party de V2 en vez de un multiplicador de marketing del benchmark de otra persona.

Así que antes de cualquier teoría, haz la cosa. Si todavía no pusiste V2 en esta máquina, haz el build del release candidate desde su canal git documentado, la misma instalación que corriste en m01. Acuérdate de esa lección: `avm install` no puede traer la RC, porque descarga binarios precompilados de GitHub Releases y no se cortó ningún Release para el tag v2. El camino sancionado es un build desde el código fuente del hogar actual del repositorio, otter-sec/anchor (las URLs viejas coral-xyz y solana-foundation redirigen ahí), fijado al tag `v2.0.0-rc.1`:

```bash
# The documented V2 RC install: build from source at the v2.0.0-rc.1 tag.
# Pin verified live on crates.io 2026-08-23: anchor-cli 2.0.0-rc.1 (published 2026-08-12).
# RC pins move fast: re-verify the tag and version before you pin a Dockerfile.
# The LTO prefix is needed on macOS when the link step dies, harmless on Linux
# (m01-l2 has the why). It is also why the earlier install blocks in this course
# print it: leave it on and one command works everywhere.
# Why git and not `cargo install anchor-cli@2.0.0-rc.1`? The crates.io publish is
# real but undocumented for CLI installs; the git build is the sanctioned channel
# for the BINARY. (The library is the opposite: your program crate takes
# anchor-lang from crates.io, because a published version cannot move.)
CARGO_PROFILE_RELEASE_LTO=off \
cargo install --git https://github.com/otter-sec/anchor.git \
  --tag v2.0.0-rc.1 anchor-cli --locked --force
anchor --version
```

Ahora, desde tu workspace de R4, corre el profiler contra las pruebas de swap que ya tienes:

```bash
anchor test --profile
```

Cuando termine en verde, mira dentro de `target/anchor-v2-profile/`. Hay un SVG de flamegraph sentado ahí, uno por prueba, que no existía cinco minutos atrás. Déjalo abierto en una pestaña del navegador; el lab lo retoma en el paso 2 y lo lee como se debe. Al final vas a saber cómo leerlo, sobre qué miente, y el único número que vale anotar.

## Resumen

Vas a instrumentar el swap que ya existe, no a extenderlo. No se escribe lógica nueva de programa. R4 gana una capa de observabilidad construida con cuatro instrumentos. Tres de ellos, el profiler, el debugger y la cobertura, corren contra las pruebas de swap en LiteSVM que ya tienes: ningún montaje nuevo, tres lecturas de una sola corrida que ya existe. El cuarto, Mollusk, es un segundo montaje y vale decirlo, porque te cuesta sus propias dev-dependencies, su propia fixture de cuentas y su propia invocación de `cargo test`. Ese es el precio de un número de CU lo bastante preciso como para convertirse en aserción.

Tres de ellos son de Anchor mismo: `anchor test --profile` para flamegraphs, `anchor debugger` para dar step a una instrucción que falla, y `anchor coverage` para encontrar ramas sin prueba. El cuarto es `anza-xyz/mollusk`, un crate de terceros sin ninguna afiliación con Anchor, que es de donde sale una aserción precisa en unidades de cómputo sobre una sola instrucción. Tres instrumentos first-party más uno prestado, y el prestado es el único que hace de barrera. El lab recorre los cuatro contra el swap como ejemplo trabajado. Después vuelves a correr cada uno contra tu propio swap sin apoyo y registras dos cosas: el costo de línea base en unidades de cómputo de un canje, y el nombre del frame más caliente del flamegraph.

Esa división es el repliegue de la ayuda de esta lección. Yo demuestro las cuatro herramientas sobre R4 contigo mirando. Tú las vuelves a correr sobre tu propio programa sin las rueditas. Y la interpretación, leer el ancho del flamegraph y detectar el hueco de cobertura, es solo tuya al final. No hay código de programa para escribir. La única cosa que escribes es una sola constante en una prueba, y el resto del hacer es medición.

Un término de glosario, porque está en cada línea de abajo. Una unidad de cómputo, o CU, es la medición que hace Solana del trabajo on-chain: cada instrucción corre contra un presupuesto de compute, y cada operación que ejecuta el runtime le debita alguna CU. "¿Cuánto cuesta un canje?" quiere decir "¿cuántas CU consume la instrucción de canje?" Más barato quiere decir holgura para más trabajo en la misma transacción y una tarifa más chica al aterrizar.

## Los cuatro instrumentos

Acá está la forma del toolkit entero antes de manejarlo. Cuatro herramientas, cuatro preguntas distintas, una sola fixture de prueba compartida debajo. Lee esta tabla una vez y vuelve a ella durante el lab.

![Una tarjeta de cuatro filas que compara el profiler, el debugger, la cobertura y la prueba de Mollusk por la pregunta que responde cada uno, su salida, su tipo de build, y si hace de barrera para la corrida.](assets/v01-comparison.webp)

La cosa que hay que internalizar es la última columna. Tres de estos cuatro reportan: te entregan un artefacto y te dejan decidir qué quiere decir. Solo la prueba de Mollusk decide por ti, porque una prueba es un contrato de pasa-o-falla. Esa diferencia es la razón por la que el número de línea base con el que terminas comprometiéndote vive en la prueba de Mollusk y en ningún otro lado.

Ahora cada instrumento, en el orden en que de verdad vas a agarrarlos.

### El profiler: a dónde se fue la CU

`anchor test --profile` corre tu suite normal de pruebas, pero compila el programa de una forma específica y captura un artefacto específico. Hace el build en DEBUG para que el binario se quede con sus símbolos DWARF. DWARF es el formato de debug-info que mapea instrucciones compiladas crudas de vuelta a los nombres de tus funciones y a las líneas de código. Sin él, un profile es una pared de direcciones hex. Con él, el profiler puede rotular cada frame con la función a la que pertenece.

El artefacto capturado es un flamegraph. Un flamegraph es un gráfico de barras apiladas de dónde se acumuló el tiempo de ejecución, o acá el costo en compute: cada caja es una función, su ancho es el costo que se le atribuye, y las cajas se apilan para mostrar quién llamó a quién. Una nota de orientación, porque decide a dónde miras: estos SVG se dibujan en estilo icicle, con la raíz arriba y los llamados apilándose hacia abajo, así que la caja de tu instrucción queda arriba y todo lo que ella llamó cuelga *debajo* de ella. La caja más ancha debajo de la raíz de tu instrucción que sea código tuyo es, a grandes rasgos, "a dónde se fue la CU." Se escribe un SVG por prueba dentro de `target/anchor-v2-profile/`.

![Un diagrama de flujo de cinco etapas desde la compilación en debug pasando por la resolución de frames DWARF hasta un SVG por prueba, que advierte que la CU en debug muestra forma relativa en vez de costo en release.](assets/v02-flowchart.webp)

Lee la figura. Acá está lo que un frame de flamegraph te está diciendo y lo que no.

![Un flamegraph estilizado donde los frames anchos de matemática y de deserialización de la instrucción de swap son los hotspots de verdad mientras un frame de montaje de prueba igual de ancho aparece en gris como ruido del banco de pruebas.](assets/v03-annotated-code.webp)

Ese frame del banco de pruebas es la primera trampa y la más común. Tu prueba de LiteSVM acuña y fondea cuentas en sus propias transacciones antes de llamar a `swap_arcade_for_tickets`, y el profiler traza cada instrucción de la corrida, así que esas instrucciones de montaje sacan sus propias raíces en el mismo SVG, muchas veces más gordas que el canje. Son costo de verdad, pero no son el costo de tu instrucción. Persigue una y vas a optimizar tu fixture de prueba mientras el canje queda exactamente igual de caro que antes. Todo lo que te importa cuelga debajo de la raíz `swap_arcade_for_tickets` específicamente.

### El debugger: instrucción por instrucción

A veces el flamegraph no es la pregunta. A veces una instrucción falla y necesitas verla morir. `anchor debugger` es una TUI al estilo foundry, una interfaz de terminal en ratatui, que le da step a tu programa una instrucción sBPF a la vez y te muestra los registros a medida que cambian. sBPF es el sabor Solana del bytecode eBPF al que baja compilado tu Rust, la cosa de verdad que ejecuta el runtime.

Para trabajo más profundo se conecta a un debugger de verdad. Pasa `--gdb` y Anchor expone el gdb stub de solana-sbpf, un servidor chico que habla el protocolo remoto de gdb para que puedas adjuntar gdb y poner breakpoints contra el programa corriendo. En el lab vas a apuntar el debugger a un canje roto a propósito y a dar step hasta la instrucción exacta donde revierte.

### Cobertura: qué ramas nunca corrieron

`anchor coverage` responde una pregunta que las otras tres no pueden: ¿qué no tocaron nunca tus pruebas? Reconstruye cobertura de línea y de rama a partir de trazas de registros SBF y la emite como LCOV, el formato estándar de reporte de cobertura de línea que los editores y las herramientas de CI ya saben mostrar. Apúntalo al swap y te va a mostrar, por ejemplo, que la rama de tu guarda de slippage o tu retorno temprano de monto cero nunca se ejecutó bajo ninguna prueba.

Acá está la trampa, dicha sin adornos para que no la esperes: `anchor coverage` reporta, no hace de barrera. Te entrega un archivo LCOV y se va, en vez de hacer fallar tu build cuando la cobertura caiga. Si quieres un piso de cobertura impuesto, eso es una política de CI que escribes encima del reporte, no una cosa que la herramienta haga por ti.

### Mollusk: exactamente cuántas CU

Las tres primeras herramientas describen. Mollusk hace aserciones. Mollusk (`anza-xyz/mollusk`) es un banco de pruebas liviano e in-process que corre una sola instrucción en una SVM minificada y te deja hacer verificaciones duras sobre el resultado, incluida una verificación precisa en unidades de cómputo. Sin validador, sin localnet, sin async. Armas una instrucción, le entregas cuentas, y haces una aserción tanto de que tuvo éxito como de que consumió un número específico de CU.

Acá es donde escala el hilo de pruebas del módulo. En m02 escribiste una prueba de LiteSVM: rápida, in-process, buenísima para comportamiento. LiteSVM responde "¿hizo lo correcto?" Mollusk responde "¿hizo lo correcto por exactamente esta cantidad de unidades de cómputo?" La misma velocidad in-process, un peldaño más afilado. Más adelante, en el capstone, entra Surfpool para integración en localnet de salón entero contra estado real del cluster. Para una aserción precisa en CU sobre una instrucción, hoy, Mollusk es la herramienta.

![Un diagrama de eje y radios donde tres instrumentos de Anchor leen la misma corrida de swap en LiteSVM que ya existe, con la prueba de CU de Mollusk dibujada aparte como una segunda fixture propia.](assets/v04-diagram.webp)

### El trade-off, antes de que confíes en nada de esto

La instrumentación no es gratis y no es la verdad. Nombra los costos ahora para que ningún número te sorprenda después.

`--profile` hace el build en DEBUG para sacar esos símbolos DWARF. Un build de debug no es tu build de release, así que sus cifras de CU están infladas y tienen otra forma que lo que se entrega. Usa el flamegraph para forma relativa, qué frame está gordo en relación a los otros, nunca como un costo absoluto que le cites a alguien. El debugger y la cobertura los dos agregan tiempo de build y de montaje que pagas en cada corrida, así que los agarras cuando tienes una pregunta en vez de dejarlos prendidos en cada corrida. Y el límite más profundo de todos: un flamegraph te dice DÓNDE está el costo, nunca POR QUÉ. Las herramientas miden. La próxima lección decide qué hacer al respecto.

Esa honestidad no es solo mía. El propio titular de benchmark de V2 de Anchor se volvió más honesto con el tiempo. En el PR #4914, mergeado el 2026-08-13, los números de marketing se revisaron a la baja: la afirmación de "95% smaller bytecode" pasó a 94%, y la de "9.9x average CU reduction" pasó a 8.8x.

![Una línea de tiempo de dos puntos que muestra el PR #4914 el 2026-08-13 revisando el titular de V2 de Anchor de 95 por ciento a 94 por ciento de bytecode y de 9.9x a 8.8x de CU promedio, motivando medir tu propio programa.](assets/v05-timeline.webp)

Esa es la razón por la que este curso nunca te entrega un multiplicador para repetir. Un titular de benchmark es el programa de otra persona sobre la carga de trabajo de otra persona. Tu canje es tuyo. Mídelo.

## Lab: instrumenta el swap

Ejemplo trabajado. Corremos los cuatro instrumentos contra R4, el swap, y aterrizamos en un número de CU de línea base para el canje más el frame nombrado más caliente. Acompáñame en tu propio checkout de R4. Cada comando de abajo es real.

### 1. Confirma tu toolchain

No fijes una versión de memoria. Esa es la última trampa y muerde en silencio, porque un pin obsoleto compila bien y solo mide la cosa equivocada.

```bash
anchor --version          # expect anchor-cli 2.0.0-rc.1 (the tag build; pin verified 2026-08-23)
solana --version          # expect 3.1.10, the course's local-CI pin from m01-l2.
                          # A different number is not an error, it is a note to yourself:
                          # your CU readings are against a different runtime than mine.
```

### 2. Genera el flamegraph

```bash
anchor test --profile
ls target/anchor-v2-profile/
```

Ahora deberías ver un `.svg` por prueba. Abre el de tu prueba de swap en un navegador. El gráfico es en estilo icicle, así que las raíces quedan arriba: encuentra la caja raíz `swap_arcade_for_tickets` allá arriba, ignora las raíces hermanas que son las transacciones de montaje de tu prueba, y busca la caja más ancha que cuelgue debajo de ella. En el swap de Quarters tal como yo lo armé, ese frame de código propio más ancho es `swap_out`, la matemática de producto constante, con la deserialización de cuentas pisándole los talones. Anota lo que de verdad diga el tuyo. Esa es la mitad de la forma de tu respuesta.

### 3. Dale step al caso que falla en el debugger

Queremos algo que revierta, así que rompe el swap temporalmente: en tu prueba de slippage, pon `min_out` uno por encima de la salida cotizada para que dispare la guarda. Después apunta el debugger a esa sola prueba por nombre, desde la raíz del workspace:

```bash
anchor build                                   # the debugger steps the built .so
anchor debugger --test slippage_reverts        # names the test whose transaction to step
```

Dos cosas para saber antes de que abra la TUI, porque un lanzamiento pelado es confuso. El debugger no corre tu suite; hace el build de la prueba nombrada, repite su transacción, y se detiene en la primera instrucción de tu programa, esperándote. Y le da step al sBPF *de tu programa*, no al banco de pruebas, así que la primera instrucción que ves es el punto de entrada, no `main`.

La TUI abre con el flujo de instrucciones a la izquierda y los registros a la derecha. Dale step hacia adelante y míralos. Estás buscando el momento en que el programa pega en tu guarda `require!` y salta al retorno de error. Cuando lo encuentres, localizaste la rama que falla al nivel de la instrucción, no la adivinaste desde un log.

Si quieres gdb propiamente dicho, relanza con el stub. Escucha en `127.0.0.1:9001` y espera una conexión antes de correr cualquier cosa:

```bash
# gdb itself is not part of the Anchor toolchain. Install it once if you do not have it:
#   macOS: brew install gdb   Debian/Ubuntu: sudo apt install gdb
anchor debugger --test slippage_reverts --gdb
```

Después, en una segunda terminal, adjúntate al stub y apunta gdb al binario sin stripping para que pueda resolver símbolos:

```bash
gdb target/deploy/token_ticket_swap.so
(gdb) target remote 127.0.0.1:9001
(gdb) break swap_arcade_for_tickets
(gdb) continue
```

Deshaz tu rotura deliberada antes de seguir. El trabajo del debugger acá era demostrar que puedes caminar una instrucción hasta su falla. Deja el swap funcionando antes de continuar.

### 4. Encuentra las ramas sin prueba

```bash
anchor coverage
ls target/anchor-v2-coverage/       # lcov.info lands here
```

Eso escribe `target/anchor-v2-coverage/lcov.info`, y el LCOV crudo es un formato de máquina: líneas `DA:` para aciertos de línea, líneas `BRDA:` para aciertos de rama, ni un código a la vista. No lo lees directo. Renderízalo:

```bash
# genhtml ships with lcov. macOS: brew install lcov   Debian/Ubuntu: sudo apt install lcov
genhtml target/anchor-v2-coverage/lcov.info -o target/anchor-v2-coverage/html
open target/anchor-v2-coverage/html/index.html    # xdg-open on Linux
```

Ahora tienes una vista del código con cada línea coloreada por cantidad de aciertos. Entra a `lib.rs` y lee qué ramas del swap nunca corrieron. Muy probablemente tu camino feliz esté verde y una rama de borde, el revert de slippage o la guarda de salida cero, salga en rojo. Ese rojo no es una falla de build. Acuérdate: la cobertura reporta, no hace de barrera, y te está diciendo a dónde debería ir una prueba futura.

Si prefieres quedarte en tu editor, la mayoría de las extensiones de cobertura leen `lcov.info` directo; apunta una a esa ruta y sáltate `genhtml`.

### 5. Fija la línea base con Mollusk

Este es el que queda, y empieza con un build, no con una prueba. Mollusk carga un `.so` compilado del disco por nombre, y nada de ese archivo dice qué configuración lo compiló — el `anchor test --profile` del paso 2 escribió un build DEBUG ahí, y pasos posteriores escribieron encima del directorio desde entonces. Mide lo que por casualidad esté tirado en el disco y no puedes ni decir qué build fijaste como tu línea base. Así que la regla, que vale cada vez que toques Mollusk de acá en adelante: **haz el build exactamente de la configuración que pretendes medir, inmediatamente antes de medirla.**

```bash
cargo build-sbf              # release, defaults on: the configuration you actually ship
# Mollusk searches tests/fixtures, $SBF_OUT_DIR, and the current directory for the .so —
# NOT target/deploy. `anchor test` sets this var for you; a bare `cargo test` does not,
# and the miss reads `[MOLLUSK]: Program file not found`. Export it once per shell.
export SBF_OUT_DIR=$PWD/target/deploy
```

Después agrega Mollusk al crate de tu programa como dev-dependency. Cuesta seis filas de dev, más una verificación sobre un pin que el workspace entero ya carga:

```toml
# programs/token-ticket-swap/Cargo.toml
[dependencies]
# This row is the reason m02-l1 wrote the arcade pin as a ceiling instead of an equality.
# Mollusk's SVM stack reaches solana-address ^2.6.1, and `=2.6.0` refuses that resolve
# before anything compiles; 2.6.1 is still on wincode 0.5, so the real constraint — below
# 2.7 — still holds. Confirm it reads this way HERE AND IN EVERY SIBLING RUNG. Cargo
# resolves one solana-address for the whole workspace, so a single member still holding
# `=2.6.0` fails the entire workspace, not just its own crate.
solana-address = ">=2.6.1, <2.7"

[dev-dependencies]
# Pins verified live on crates.io 2026-09-01: mollusk-svm 0.15.1 (published 2026-08-29).
# A 0.15.0-agave-4.3.0-beta.0 also exists (2026-08-18); stay on the stable line unless
# you are tracking the agave 4.3 beta SVM. Mollusk 0.15 builds on the agave 4.2 SVM
# crates, so your solana dev-deps must be the 4.x line: a 2.x solana-sdk will not
# type-check against Mollusk's Pubkey/Account/Instruction types.
mollusk-svm = "0.15.1"
# Mollusk's minified SVM loads no program you don't register, and the trade CPIs
# into SPL Token — this companion crate (same 0.15 line, same publish batch)
# ships the real token-program ELF plus the two helpers the fixture and test
# use: token::add_program() and token::keyed_account().
mollusk-svm-programs-token = "0.15.1"
solana-sdk = "4"
# The two rows below hold Mollusk's own graph on the wincode 0.5 line. Without them the
# resolve succeeds and the BUILD dies, in solana-message and then in solana-transaction.
solana-short-vec = ">=3.2.2, <3.3"
solana-signature = ">=3.4.1, <3.5"
# The fixture builds real SPL mint and token-account state, and the test names
# spl_token::ID — the same pin m05-l1 installed.
spl-token = "9"
```

Las dos filas de rango de `solana-*` son otra vez la clase de bug de la issue #4937, una capa más abajo, y vale entenderlas en vez de pegarlas. `solana-short-vec 3.3.0` y `solana-signature 3.5.0` los dos pasaron a `wincode 0.6` mientras todavía satisfacen lo que pide `solana-message 4.4.0`, así que un resolve fresco pone dos majors de `wincode` en el grafo y `solana-message` deja de compilar contra cualquiera de los dos que elija cargo. Fijar los dos de vuelta por debajo de esos majors sostiene toda la línea solana 4.x en `wincode 0.5`, que es la línea que `anchor-lang 2.0.0-rc.1` ya quiere. Trata estos tres pins como una sola decisión: cuando V2 cruce a `wincode 0.6`, se van todos de una vez.

Fíjate en la diferencia de radio de impacto entre las filas de dev y la fila `[dependencies]` de arriba, porque es la lección práctica acá. Las dos filas de rango son dev-dependencies de este crate — le dan forma al lock del workspace, y ningún hermano tiene que declararlas nunca. `solana-address` es lo opuesto, por la razón que da su propio comentario, y eso no es una rareza de Mollusk; es lo que un workspace *es*. Un pin que es tuyo sigue siendo una decisión por crate hasta el momento exacto en que un hermano no está de acuerdo con él.

Ahora la prueba precisa en CU. Arma la instrucción `swap_arcade_for_tickets` usando los tipos que Anchor generó para tu programa, le entrega a Mollusk la fixture de cuentas, y hace aserciones tanto sobre el éxito como sobre las unidades de cómputo. En el ejemplo trabajado el banco de pruebas y el montaje de cuentas se te entregan. Acá está la cosa entera, con las dos líneas que llenas durante el Challenge marcadas:

```rust
// programs/token-ticket-swap/tests/cu_baseline.rs
mod swap_fixture;

use anchor_lang::{InstructionData, ToAccountMetas};
use mollusk_svm::{result::Check, Mollusk};
use solana_sdk::{account::Account, instruction::Instruction, pubkey::Pubkey};

// The Anchor-generated instruction args + accounts for R4's swap handler.
// Anchor names both after the handler: `swap_arcade_for_tickets` -> `SwapArcadeForTickets`.
use token_ticket_swap::accounts::SwapArcadeForTickets as SwapAccounts;
use token_ticket_swap::instruction::SwapArcadeForTickets as SwapArgs;

/// Builds the swap fixture: the program, the accounts, and one swap instruction.
/// (Provided for you in the worked example. In your own swap you adapt the account list
/// to R4's actual `SwapArcadeForTickets` context.)
fn swap_fixture() -> (Mollusk, Instruction, Vec<(Pubkey, Account)>) {
    let program_id = token_ticket_swap::ID;
    // Mollusk loads the compiled .so by name, from wherever SBF_OUT_DIR points.
    let mut mollusk = Mollusk::new(&program_id, "token_ticket_swap");
    // A minified SVM runs only the programs you register, and the trade CPIs
    // into SPL Token. Put the real token program in the cache; the fixture
    // supplies the matching account row.
    mollusk_svm_programs_token::token::add_program(&mut mollusk);

    // `build_swap_accounts` builds the trader, the pool PDA, both mints, and the four
    // token accounts (two reserves, two trader-side), funds them, and returns them as
    // Mollusk's (Pubkey, Account) pairs. It is ordinary SPL fixture construction with
    // nothing V2-specific in it, so it ships beside this lesson at
    // `lessons/m06-l1/swap-fixture/`. Drop it in as `tests/swap_fixture.rs`
    // and `mod swap_fixture;` at the top of this file. Its full surface, which the
    // next lesson also leans on: keys(), build_swap_accounts(), build_swap_ix(),
    // build_init_accounts(), build_init_ix().
    let keys = swap_fixture::keys(&program_id);
    let accounts: Vec<(Pubkey, Account)> = swap_fixture::build_swap_accounts(&keys);

    let metas = SwapAccounts {
        trader: keys.trader,
        pool: keys.pool,
        mint_arcade: keys.mint_arcade,
        mint_ticket: keys.mint_ticket,
        reserve_arcade: keys.reserve_arcade,
        reserve_ticket: keys.reserve_ticket,
        trader_arcade: keys.trader_arcade,
        trader_ticket: keys.trader_ticket,
        token_program: spl_token::ID,
    }
    .to_account_metas(None);
    // The handler's own args: amount_in, and a min_out of 0 so the slippage guard
    // never decides the measurement for you.
    let data = SwapArgs { amount_in: 100, min_out: 0 }.data();
    let ix = Instruction { program_id, accounts: metas, data };

    (mollusk, ix, accounts)
}

#[test]
fn trade_cu_baseline() {
    let (mollusk, ix, accounts) = swap_fixture();

    // Read the raw number FIRST, so this test always prints what one trade costs
    // right now. `process_instruction` runs the trade and reports; it asserts nothing.
    let measured = mollusk.process_instruction(&ix, &accounts);
    println!("trade consumed {} CU", measured.compute_units_consumed);

    // YOU FILL THIS IN THE CHALLENGE: the CU bound you measured for one trade.
    // The assertion below is written for you; the number is the exercise.
    const TRADE_CU_BASELINE: u64 = /* your measured baseline */ 0;

    mollusk.process_and_validate_instruction(
        &ix,
        &accounts,
        &[Check::success(), Check::compute_units(TRADE_CU_BASELINE)],
    );
}
```

El módulo de fixture en sí se entrega junto a esta lección como [swap-fixture/swap_fixture.rs](swap-fixture/swap_fixture.rs) — layouts reales de mint y de cuenta de token de SPL, el pool en su PDA, nada que no hayas visto ya.

Córrelo ahora, antes de tener número alguno para fijar. El `println!` se dispara antes que la aserción, así que la cota placeholder `0` hace fallar la prueba y de todos modos te vas con tu medición:

```bash
cargo build-sbf && cargo test -p token-ticket-swap trade_cu_baseline -- --nocapture
```

Resultado esperado: una línea que diga `trade consumed <N> CU`, seguida de una falla en `Check::compute_units(0)`. Esa `N` es tu línea base. Ponla en `TRADE_CU_BASELINE` y corre el mismo comando otra vez; esta vez se pone en verde, y de acá en adelante `Check::compute_units` hace fallar el build el día en que un cambio saque el canje de ese número. Ese es el punto de fijarlo en una prueba y no en un flamegraph: el flamegraph es una foto que miras, la aserción de Mollusk es una alarma que vigila por ti. Deja el leer-e-imprimir arriba de la prueba, porque así vas a tomar la lectura del "antes" la próxima lección.

Un número de referencia para escala. Helius publicó conteos de CU de V1 para un programa contador trivial, alrededor de 5,095 para inicializar y 1,162 para incrementar: sin fecha, V1, un programa distinto. Úsalos para una sola cosa, una noción de orden de magnitud. Una instrucción de verdad vive en los miles de CU, no en las decenas y no en los millones. Si tu lectura queda muy afuera de esa banda, sospecha de tu fixture antes de festejar.

![Un gráfico de barras de un contador V1 sin fecha en 5095 y 1162 CU al lado de una barra fantasma para el propio canje del lector, con epígrafe de solo-escala en vez de meta.](assets/v06-chart.webp)

## Challenge: mide tu propio canje

Corriste los comandos a mi lado. Ahora córrelos en frío, sin la página abierta, y produce dos hechos tuyos. La diferencia no son las teclas, es que nada acá te dice qué estás a punto de ver.

Completion primero, la única línea que escribes acá. Llena `TRADE_CU_BASELINE` en `trade_cu_baseline` con el número que mediste, y mira la prueba pasar de rojo a verde con esa sola edición.

Después la corrida Solo:

1. `anchor test --profile` sobre tu swap. Abre el SVG, encuentra la raíz `swap_arcade_for_tickets`, y nombra el frame de código propio más ancho debajo de ella. Lo que diga es tu respuesta, incluso si no es el frame que me salió a mí: mi swap y el tuyo divergieron por cuatro lecciones de ediciones, y un frame caliente distinto es un hallazgo, no un error. La única respuesta equivocada es un frame que no esté debajo de esa raíz para nada.
2. `anchor debugger` sobre un canje fallado a propósito. Dale step hasta la instrucción que falla, después deshaz la rotura.
3. `anchor coverage`. Abre el LCOV y nombra una rama que tus pruebas nunca ejercitaron.
4. Lee la CU de Mollusk, fíjala en `TRADE_CU_BASELINE`, y confirma que la prueba se pone en verde.

La forma de tu respuesta son exactamente dos cosas: un entero de CU para un solo canje, y el nombre del frame más caliente del flamegraph. Anótalos en algún lado donde los vas a encontrar la próxima lección.

![Una tarjeta de registro con espacios en blanco para la línea base de CU del canje, el frame más caliente, las herramientas y los tipos de build usados, la rama sin prueba encontrada, y la fecha de la medición.](assets/v07-table.webp)

El criterio para pasar es simple y estricto. Las cuatro herramientas corren limpias. El número de línea base existe y vive en una aserción de Mollusk que pasa. Y el frame que nombraste es uno que cuelga debajo de la raíz de tu instrucción, así que es trabajo de instrucción y no trabajo de fixture, como sea que termine llamándose.

## Antes de seguir adelante

Chequéate contra tres preguntas, porque estos son los lugares exactos donde esta lección sale mal en la práctica.

¿Tu CU de línea base viene de Mollusk, no del flamegraph? Ese es el único lugar del que tiene permitido venir, porque el flamegraph es un build de debug y sus números son forma y no costo. El número con el que te comprometes es el de Mollusk.

¿Tu frame más caliente está debajo de la raíz `swap_arcade_for_tickets`, y no debajo de una de las raíces hermanas que produjeron las transacciones de montaje de tu prueba? Solo los frames debajo de tu instrucción son el costo de tu instrucción.

¿Notaste que `anchor coverage` ni una sola vez amenazó con hacer fallar tu build? Bien. Reporta. Nada acá hace de barrera, salvo la prueba que escribiste tú mismo.

Tienes un número de verdad ahora, medido por ti, sobre tu programa, sin ningún multiplicador prestado de nadie. Ese número es una línea de partida, no una llegada. El flamegraph te está mostrando un frame gordo sentado sobre tu instrucción de canje. Puedes ver el costo. La próxima lección lo haces más chico, un cambio medido a la vez, con CU de antes-y-después como la única demostración que cuenta.

Anda a buscar tu número.
