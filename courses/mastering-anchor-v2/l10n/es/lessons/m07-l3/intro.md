# La checklist de auditoría como lab, y el fuzz como la evaluación

La lección pasada drenaste el escrow con tus propias pruebas de exploit, y después lo parcheaste: un pin de `address` sobre quien llama, un pin de `address` sobre el `UncheckedAccount` sustituible, y `checked_sub` sobre el débito del vault, hasta que falló cada ataque que se te pudo ocurrir. No atacaste el swap. Mapeaste las mismas siete clases sobre sus campos y lo dejaste ahí. Esa última cláusula es la trampa. "Cada ataque que se te pudo ocurrir" es una cerca construida exactamente a la altura de tu propia imaginación, y la imaginación de un atacante no es la tuya.

Así que antes de leer otro párrafo, haz esto. Abre `programs/token-ticket-swap/src/lib.rs`, busca `swap_arcade_for_tickets`, y responde una pregunta sobre su struct de cuentas: ¿hay una línea, una línea de verdad que puedas señalar, que demuestre que `reserve_ticket` es la reserva propia del pool y no una cuenta de token que eligió quien llama? No "Anchor probablemente lo maneja." Un número de línea, o la palabra `FAIL`. Escríbelo en un archivo de borrador. Esa es la primera fila de tu checklist de auditoría, y acabas de empezar el lab.

```bash
# Find the swap handler you are about to audit, and start the checklist file.
grep -n "fn swap_arcade_for_tickets" programs/token-ticket-swap/src/lib.rs
: > audit-checklist.txt   # one row per check: write a line number, or the word FAIL
```

Si la línea que buscaste es un constraint `token::authority = pool`, mira de nuevo, porque esa es la trampa de la pregunta. Cualquiera puede crear una cuenta de token de SPL cuya autoridad sea el PDA del pool — `InitializeAccount` toma al dueño como un argumento pelado, sin ninguna firma del dueño — y las dos reservas comparten esa autoridad de todos modos. La autoridad dice quién puede gastar desde una cuenta, no cuál cuenta quiso decir el pool, que es precisamente la clase de sustitución de la lección pasada. La línea que *sí* respondería la pregunta es el pin que el mapeo de la lección pasada exigía para R4: la *dirección* de la reserva verificada contra una reserva que el pool registró en el init. Ve a buscarla y encuentras algo más filoso que un constraint faltante. `Pool` almacena los dos mints y su bump y nada más — m05-l2 dijo justamente eso cuando notó que las reservas se sientan "en direcciones que nadie deriva" — así que no hay ninguna dirección registrada on-chain contra la cual verificar. La línea no existe, y no se puede escribir hasta que el pool empiece a registrar una. Escribe `FAIL`. Acabas de hacer el primer hallazgo real de la checklist antes de que la checklist siquiera empezara, y es un arreglo de dos partes en vez de algo de una línea, que es exactamente el tipo de cosa por la que una lectura de corrido pasa de largo y una checklist no. El punto no es la dificultad. El punto es que miraste, y que ahora tienes una fila de un archivo que lo dice.

Esta lección convierte la seguridad de una sensación en un procedimiento de dos partes. La parte uno es una checklist que corres a mano, fila por fila, contra el swap: hace que tu revisión sea repetible y te obliga a nombrar la línea que satisface cada garantía. La parte dos es `anchor fuzz`, que corre el ataque por ti. Lo apuntas al swap, te vas, y vuelves a un artefacto de crash por un input que nunca habrías escrito. El reencuadre que carga la lección entera: una corrida de fuzz limpia no es tranquilidad, es silencio. Un crash es la victoria, porque un crash que encontraste tú es un bug que el atacante no.

![Una tarjeta de dos columnas que compara qué atrapa, qué se le pierde y qué cuesta la checklist manual de auditoría contra las mismas tres filas para corridas automatizadas de anchor fuzz.](assets/v01-comparison.png)

Acá está lo que te entrego y lo que no. Yo corro la checklist completa contigo y monto el primer loop de fuzz paso a paso, incluyendo sembrar un bug a propósito para que veas al fuzzer atraparlo. La aserción del invariante que está en el centro del banco de pruebas la terminas tú mismo a partir de un scaffold con un agujero. Después sembrar un bug totalmente nuevo, predecir si el fuzzer lo va a encontrar, y repetir el crash para confirmar es enteramente tuyo. El entregable al final es una checklist completada, un artefacto de crash y su repetición, un parche, y un re-fuzz limpio con un reporte de cobertura.

## Convertir la seguridad en un procedimiento

### Correr la checklist como un lab

Una revisión de código que vive en tu cabeza no es repetible, y nadie más la puede revisar. El arreglo es aburrido y funciona: un conjunto fijo de filas, y para cada fila escribes el número de línea que la satisface o escribes `FAIL` y vas a arreglarla. Siete filas cubren las clases de bug que explican la mayor parte de lo que encuentran las auditorías reales de Solana. No son las mismas siete que las clases sobrevivientes de la lección pasada, y la diferencia es deliberada: la lista de la lección pasada estaba organizada por *clase de bug*, esta por *cosa que miras en un archivo*. Dos de las clases de la lección pasada se pliegan en la fila 1 de acá, el reúso de `init_if_needed` no tiene fila porque el swap no tiene ningún `init_if_needed`, y dos filas de abajo (bumps canónicos, discriminadores) son verificaciones que el compilador mayormente hace por ti pero que no cuesta nada confirmar. Córrelas contra el swap en orden.

| # | Verificación | Condición de aprobación | Línea del swap |
|---|-------|----------------|-----------|
| 1 | Firmante y dueño presentes | Cada autoridad es un `Signer`; cada cuenta tipada tiene una verificación de dueño (implícita en `Account<T>`, explícita para `UncheckedAccount`) | ? |
| 2 | Cada `UncheckedAccount` fijado | Cada cuenta cruda lleva un `address`, un `owner`, o un `constraint` del lado del handler | ? |
| 3 | Targets de CPI validados | El programa de token que invocas es un `Interface`/`Program` tipado, no una clave que entregó quien llama | ? |
| 4 | Matemática verificada en todos lados | Nada de `+ - * /` pelados sobre balances o reservas; solo `checked_*` con un error en `None` | ? |
| 5 | Cerrar-y-poner-en-cero al desmontar | Cerrar una cuenta pone sus datos en ceros y recupera los lamports para que no se pueda revivir | ? |
| 6 | Bumps canónicos almacenados | Los bumps de PDA se leen del estado almacenado, nunca se vuelven a buscar en cada llamada | ? |
| 7 | Discriminadores sanos | Los discriminadores de cuenta son distintos y no triviales para que la confusión de tipos sea imposible | ? |

![Una fila de checklist trabajada que cita la línea 41 de swap.rs, al lado del UncheckedAccount que pasa fijado por un constraint address y la versión que falla y no fija nada.](assets/v02-annotated-code.png)

El punto de escribir el número de línea es que te defiende del autoengaño más común en una revisión, que es suponer que el framework hizo algo que no hizo. Anchor V2 sí mata varias de estas en tiempo de compilación. `Account<T>` exige `T: Pod` con un layout sin relleno, así que una lectura con confusión de tipos no compila en vez de leer mal los bytes en silencio. Las cuentas mutables duplicadas se rechazan durante la validación de cuentas a menos que hagas opt-in con el deliberadamente feo `unsafe(dup)`. Esas son garantías de verdad que puedes citar. Pero la fila 2 es exactamente aquella en la que el framework no te va a salvar: los docs de V2 son tajantes en que `UncheckedAccount` sigue sin hacer ninguna validación, y tú mismo tienes que acompañarlo con un `address`, un `owner` o un `constraint`. La checklist existe para hacerte mirar esa línea y confirmar que está.

Unas cuantas filas merecen una mirada específica en el swap, porque son las que la gente pasa por alto. Fila 3, targets de CPI: el swap mueve tokens a través de una CPI, y el programa de token que invoca tiene que ser un `Interface` o `Program` tipado, nunca una dirección pelada que pasó quien llama, o un atacante te entrega un programa parecido y tu "transfer" corre el código de ellos. Fila 5, cerrar-y-poner-en-cero: si el pool se puede desmontar, cerrarlo tiene que poner los datos en ceros además de recuperar los lamports, si no un ataque de resurrección reinicializa bytes obsoletos en una cuenta nueva con balances viejos. Fila 6, bumps canónicos: almacenaste el bump del pool en el estado cuando construiste el pool; la fila 6 confirma que el handler de `swap` lee ese bump almacenado en vez de llamar a `find_program_address` de nuevo, que es a la vez un costo de CU y una trampa sutil de corrección si las seeds llegan a cambiar. Escribe la línea, o escribe `FAIL`. Una fila de la que "estás bastante seguro" es un `FAIL` que todavía no admitiste.

![Una tabla que marca en cuáles de las siete filas de auditoría ayuda el compilador de V2 y cuáles quedan enteramente como responsabilidad del desarrollador.](assets/v03-comparison.png)

### Por qué el fuzzer atrapa lo que tu revisión no puede

Arranca desde el límite honesto de lo que acabas de hacer. La checklist es determinista y rápida, pero solo puede verificar clases de bug que ya nombraste. El bug que se entrega es, casi por definición, el que nadie nombró. Así que la pregunta es: ¿cómo encuentras un bug que no puedes imaginar? Recorre primero las respuestas ingenuas, porque cada una falla de una forma que apunta a la herramienta de verdad.

La primera respuesta ingenua es "escribe más pruebas." Falla por la misma razón por la que la checklist tiene un techo: una prueba afirma un comportamiento en el que pensaste, así que tu suite de pruebas es exactamente igual de imaginativa que tú, y nada más. La segunda respuesta ingenua es "tírale inputs aleatorios a cada instrucción." Mejor, porque la aleatoriedad no está acotada por tu imaginación, pero falla en cualquier cosa que necesite una secuencia: dispara un `swap` aleatorio contra un pool recién creado diez millones de veces y nunca vas a llegar al estado en el que el pool quedó drenado hasta una astilla y un canje redondea para el lado equivocado, porque ese estado solo existe después de una cadena específica de canjes anteriores. La tercera respuesta ingenua, "secuencias aleatorias de instrucciones," está más cerca, pero la aleatoriedad pura deambula: se pasa casi todo su tiempo re-explorando estados superficiales y casi nunca tropieza con el profundo y angosto donde vive el bug.

La herramienta que sobrevive a las tres fallas es el fuzzing stateful guiado por cobertura, y cada palabra es estructural. Stateful, para que el mundo persista a través de una cadena de acciones y los estados profundos se vuelvan siquiera alcanzables. Guiado por cobertura, para que el fuzzer no ande deambulando: mira qué ramas de tu programa compilado alcanzó cada input y dirige hacia inputs que alcancen ramas nuevas, convirtiendo una caminata aleatoria en una búsqueda dirigida. Esa combinación es lo que te da `anchor fuzz`, y es por eso que una máquina encuentra inputs que tú nunca encontrarías.

![Una comparación que descarta por turnos más pruebas, inputs aleatorios y secuencias aleatorias, y deja el fuzzing stateful guiado por cobertura como la herramienta sobreviviente.](assets/v04-comparison.png)

### Qué corre de verdad cuando escribes `anchor fuzz`

Antes de confiarle a una herramienta la seguridad de tu programa, sabe qué es. `anchor fuzz` no es un wrapper delgado alrededor de bytes aleatorios. Corre Crucible, un fuzzer guiado por cobertura construido por Asymmetric Research y cableado en el CLI de Anchor como un subcomando. Debajo de Crucible se sienta un motor de fuzzing LibAFL que maneja un runtime LiteSVM en proceso, con cobertura de aristas sBPF realimentando la selección de inputs. Esa última parte es la diferencia entre un fuzzer que manotea y uno que aprende: la cobertura de aristas quiere decir que el fuzzer ve qué ramas de tu programa compilado alcanzó cada input, y dirige hacia inputs que alcancen ramas nuevas. Lo aleatorio se vuelve dirigido.

![Un diagrama de stack que pone anchor fuzz sobre Crucible, un motor LibAFL y un runtime LiteSVM, con cobertura de aristas sBPF realimentando la mutación.](assets/v05-diagram.png)

El motor tiene un trabajo que una prueba unitaria no puede hacer: genera secuencias de acciones, no inputs sueltos. Tú describes las acciones que tu programa soporta (deposit, swap, withdraw) y las propiedades que siempre tienen que cumplirse (los invariantes), y el fuzzer elige qué acciones disparar, en qué orden, con qué argumentos, y después verifica cada invariante después de cada acción. Un bug que necesita tres llamadas específicas en un orden específico para aparecer es un bug que tus pruebas escritas a mano casi nunca alcanzan, porque tendrías que imaginarlo primero para escribirlo.

### Stateless contra stateful, y por qué importa la flag

El default de Crucible es *stateless*, y el nombre es más preciso de lo que suena. Stateless no quiere decir una acción por corrida. Cada iteración clona la instantánea posterior al `setup` y ejecuta una secuencia mutada entera contra esa copia nueva, hasta `--max-actions` (default 8), y después tira el mundo a la basura. Así que las secuencias sí están sobre la mesa por defecto. Lo que *no* está sobre la mesa es la profundidad: cada iteración vuelve a arrancar desde la misma instantánea superficial, así que un estado que toma cuarenta canjes alcanzar está cuarenta veces más lejos de lo que alcanza el presupuesto, y la búsqueda vuelve a pagar los primeros ocho pasos en todos y cada uno de los intentos.

Así que la pregunta de verdad es más angosta que "¿el input lo rompe?" Es: ¿algún *estado* alcanzable lo rompe, incluyendo estados que solo existen bien al fondo de una cadena de llamadas por lo demás válidas? Un pool drenado y después rellenado, una posición inicializada a medias, un residuo de redondeo que se acumula a lo largo de cuarenta canjes. Eso es lo que prende `--stateful`. En modo stateful Crucible mantiene un pool indexado por cobertura de estados vivos del programa (`--pool-size`, default 256,000) y aplica una acción mutada por iteración a un estado que sacó de ese pool, así que el progreso se acumula en vez de reiniciarse: las cadenas crecen hasta `--max-depth` (default 15) y los estados profundos se vuelven siquiera alcanzables. Asymmetric Research reporta una ganancia de throughput de alrededor de un orden de magnitud por eso, pagada en memoria a medida que el pool crece con la cobertura.

![Una comparación del fuzzing stateless, que descarta su instantánea en cada iteración, contra el fuzzing stateful, que mantiene un pool de estados vivos y los extiende.](assets/v06-comparison.png)

Olvidarse de `--stateful` es el modo de falla callado. Tu corrida vuelve limpia, te sientes seguro, y la mitad profunda del espacio de estados nunca estuvo en el presupuesto. Limpia sin `--stateful` quiere decir "ningún bug encontrado dentro de ocho acciones de un pool recién creado," que es una afirmación mucho más chica que la que crees que estás haciendo.

## Lab: el loop de crash-a-limpio

Todo lo de abajo corre contra el swap, el único programa que la lección pasada clasificó pero nunca atacó de verdad. Recórrelo en orden. Los primeros seis pasos los hago contigo; el agujero del invariante en el paso cinco es donde empieza el repliegue.

### 1. Instala el CLI que de verdad lleva `anchor fuzz`

Ten bien este dato del toolchain antes de escribir nada, porque tenerlo mal cuesta una tarde. Anchor tiene dos líneas vivas que este curso no deja de nombrar: la release candidate 2.0.0-rc.1 contra la que has estado construyendo, y la línea V1, que pasó de 1.1.2 a **1.2.0, publicada el 2026-09-04**. **`anchor fuzz` va sobre la línea V1, no sobre la RC.** Verificado el 2026-09-07 contra el crate publicado: el manifiesto de `anchor-cli` 1.2.0 depende de `crucible-fuzz-cli = "0.2.1"` y su enum de comandos despacha `Command::Fuzz` hacia él, mientras que el CLI 2.0.0-rc.1 entrega `anchor test --profile`, `anchor debugger` y `anchor coverage` y no tiene ningún subcomando `fuzz`. El 1.1.2 que llevan muchas máquinas tampoco lo tiene: Crucible aterrizó después de ese tag.

Así que instalas un segundo CLI. Hasta que se entregó 1.2.0 esto quería decir un build de git con `--branch master`; ahora es un release fijado, que es estrictamente mejor porque una versión publicada no se puede mover debajo de ti. Los dos builds instalan un binario llamado `anchor`, así que manda este a su propia raíz en vez de dejar que sobrescriba tu RC, y pon esa raíz primero en el `PATH` mientras dure esta lección:

```bash
# The CLI that carries `anchor fuzz` (Crucible) is the V1 line, not the V2 RC.
cargo install anchor-cli --version 1.2.0 --locked --root ~/.anchor-fuzz
export PATH="$HOME/.anchor-fuzz/bin:$PATH"   # this shell only; drop it to get the RC back
anchor fuzz --help   # unknown-subcommand error = you are running the wrong CLI
```

Nota de frescura, y léela antes de fijar: las dos líneas se mueven, y cuál lleva el fuzzer es exactamente el tipo de cosa que cambia entre releases. Vuelve a correr `anchor fuzz --help` contra el CLI que tengas antes de concluir que falta el subcomando; cuando el árbol de V2 levante Crucible, este baile de dos CLI se colapsa de vuelta a uno. Nada río abajo en esta lección depende de cuál CLI lo hospeda, porque el banco de pruebas, los invariantes y los artefactos de crash son todos de Crucible. Si tu CLI no tiene `anchor fuzz`, instala el CLI propio de Crucible y sustituye `anchor fuzz` por `crucible` en cada comando de abajo:

```bash
git clone https://github.com/asymmetric-research/crucible
cd crucible && cargo install --path crates/crucible-fuzz-cli
```

Una cosa que *no* cambia con el CLI que instales: Crucible traza las aristas sBPF sobre el `.so` compilado y genera sus bindings de llamada tipados a partir de un IDL estándar de Anchor, así que no le importa cuál línea de Anchor construyó el programa al que lo apuntas. Lo que te da la regla de trabajo para el resto de esta lección: construye el swap en un shell *sin* el override de `PATH`, para que `anchor build` siga siendo la RC de V2, y corre cada comando `anchor fuzz` en el shell que lo tiene.

### 2. Corre la checklist de auditoría y registra cada fila

Vuelve a la tabla de arriba y llena la última columna. Varias filas ya deberían pasar en el swap porque el módulo 5 las construyó así: el programa de token es un `Interface` tipado, la matemática de las reservas es `checked_*`, el bump del pool está almacenado. La lección pasada solo *mapeó* las clases restantes sobre los campos de R4 — parcheó el escrow, no el swap. No tomes nada de eso por confianza, que es la disciplina entera de la fila.

La fila 1 es el `FAIL` al que la apertura ya te llevó, y acá es donde lo arreglas, antes de fuzzear, porque el fuzzer está a punto de apoyarse exactamente en este tipo de hueco. Son tres ediciones, todas formas que ya escribiste antes:

```rust
// 1. programs/token-ticket-swap/src/lib.rs - the pool records what it owns.
#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub arcade_mint: Address,     // 32
    pub ticket_mint: Address,     // 32
    pub arcade_reserve: Address,  // 32  NEW
    pub ticket_reserve: Address,  // 32  NEW
    pub bump: u8,                 //  1
    pub _pad: [u8; 7],            //  7  explicit Pod padding (129 -> 136)
}

// 2. in init_pool, beside the two mints and ctx.bumps.pool:
pool.arcade_reserve = *ctx.accounts.reserve_arcade.address();
pool.ticket_reserve = *ctx.accounts.reserve_ticket.address();

// 3. in SwapArcadeForTickets, pin each reserve to the record. Keep the token::
//    lines: they say what the account holds, the address says WHICH one it is.
#[account(
    mut,
    address = pool.arcade_reserve @ SwapError::WrongReserve,
    token::mint = mint_arcade,
    token::authority = pool,
)]
pub reserve_arcade: InterfaceAccount<TokenAccount>,
#[account(
    mut,
    address = pool.ticket_reserve @ SwapError::WrongReserve,
    token::mint = mint_ticket,
    token::authority = pool,
)]
pub reserve_ticket: InterfaceAccount<TokenAccount>,
```

Agrégale una variante `WrongReserve` a `SwapError` mientras estás ahí. Una consecuencia para esperar en vez de descubrir: `Pool` acaba de crecer 64 bytes, así que cualquier cuenta de pool creada antes de esta edición ya no calza con `INIT_SPACE` y no va a cargar. Cada pool que hiciste hasta ahora vive adentro de una prueba de LiteSVM que construye uno nuevo en cada corrida, así que acá eso no te cuesta nada — pero fíjate en la forma de la trampa para después, porque el PDA del pool deriva solo de `[POOL_SEED]` y por lo tanto es una sola dirección fija por programa: pon un pool en un cluster y las únicas vueltas atrás son cerrarlo o desplegar a un id de programa nuevo. No hay resize en el lugar por este camino. El lab de cliente del módulo 8 decodifica este registro con `fetchPool`, así que las dos direcciones de reserva que empiezas a almacenar acá son las que él lee de vuelta.

Ahora el resto. Escribe el número de línea que demuestra cada fila restante. La fila 2 es la siguiente a la que hay que mirar fuerte: encuentra cada `UncheckedAccount` en el struct de cuentas y confirma que cada uno tenga un `address`, un `owner` o un `constraint`. Si uno está pelado, eso también es un `FAIL`, y se arregla acá también.

Checkpoint: `audit-checklist.txt` tiene siete filas y cada fila lleva un número de línea, ni un blanco ni un quizás. Cualquier `FAIL` que hayas escrito está arreglado y vuelto a verificar antes del paso 3.

Una checklist verde es el ticket de entrada, no la línea de llegada. Demuestra que las clases estructurales están manejadas. No dice nada sobre si la matemática de tu swap preserva el producto constante, y para eso es el fuzzer.

### 3. Genera el scaffold del banco de pruebas de fuzz

```bash
# Generate a fuzz harness template for the `swap` program.
anchor fuzz init token_ticket_swap
```

Esto escribe un workspace de fuzz independiente en `fuzz/token_ticket_swap/` (el banco de pruebas en `src/main.rs`, el IDL del programa en `idls/`, los artefactos de crash después en `crashes/`) con un `Cargo.toml` que depende de la librería de banco de pruebas de Crucible. Dos cosas en ese archivo para verificar a mano, porque las dos muerden en silencio:

```toml
# fuzz/token_ticket_swap/Cargo.toml
[dependencies]
crucible-fuzzer = "0.2.1"    # harness library; version-locked with the CLI (crucible-fuzz-cli 0.2.1)

[features]
constant_product_holds = []  # ONE feature per fuzz test, named EXACTLY like the test function
```

La línea de feature es en la que la gente pierde una hora: cada prueba de fuzz se tiene que declarar como un feature cuyo nombre calce con el nombre de la función de prueba carácter por carácter, o el CLI no va a encontrar la prueba que estás tratando de correr.

Nota de frescura: `0.2.1` es el estable actual tanto de `crucible-fuzzer` como de `crucible-fuzz-cli` en crates.io al 2026-08-22, y una línea `0.3.0-alpha.1` ya está publicada. Se mueven juntos, así que vuelve a revisar qué escribe el scaffold después de cualquier rebuild del CLI en vez de asumir este pin.

Generar el scaffold también desbloquea el resto de la familia de comandos `anchor fuzz`, y vale la pena ver el mapa entero ahora para que sepas para qué es cada uno cuando lo necesites después en el loop.

![Una tabla de los subcomandos de anchor fuzz (init, run, list, show, cmin, tmin) con sus flags, que anota anchor coverage como una lectura aparte.](assets/v07-table.png)

### 4. Siembra un overflow conocido para que veas al fuzzer ganarse el sueldo

No fuzzees un programa limpio primero. Siembra un bug que entiendas, confirma que el fuzzer lo atrapa, y solo entonces confía en una corrida limpia. Esta es la misma disciplina que ver una prueba fallar antes de hacerla pasar.

Acá está la matemática de producto constante del swap. Este es `swap_out`, la misma función que vienes cargando desde que construiste R4, movida a su propio `src/math.rs` para esta lección para que la edición sembrada sea un diff de una línea en un archivo que nada más toca. El invariante es `k = reserve_in * reserve_out`, y un canje nunca tiene que dejar que `k` se encoja. Como el swap cobra 0.3%, la comisión se queda en el pool, así que en la práctica `k` crece un poco en cada canje; `k_now >= k_before` es la aserción que es verdadera de cualquier manera.

Esa aserción también cobra una deuda del módulo 5. m05-l3 advirtió que un mint con tarifa de transferencia anula calladamente la suposición de recibiste-lo-que-mandaste, así que el pool acredita menos de lo que mandó el trader "y tu invariante se corre" — y después soltó el hilo, porque todavía no tenías ningún invariante. Este es. Apunta el swap a un mint de arcade que lleve una tarifa de transferencia y la reserva crece por menos que `amount_in` mientras el lado del ticket paga un `out` cotizado desde el `amount_in` completo: `k` se encoge genuinamente, y `k_now >= k_before` es la línea que se pone en rojo. La advertencia huérfana del módulo 5 y la aserción que estás a punto de escribir son el mismo hecho, con dos módulos de distancia.

```rust
// programs/token-ticket-swap/src/math.rs  (correct: the swap_out you built, with its fee)
pub fn swap_out(reserve_in: u64, reserve_out: u64, amount_in: u64) -> Result<u64> {
    // 997/1000 is the 0.3% fee: the withheld 3/1000 stays in the pool, which is
    // exactly why k grows rather than staying equal.
    let amount_in_with_fee = (amount_in as u128)
        .checked_mul(997)
        .ok_or(SwapError::Overflow)?;

    // The product is held in u128 so it cannot wrap a u64.
    let numerator = amount_in_with_fee
        .checked_mul(reserve_out as u128)
        .ok_or(SwapError::Overflow)?;

    let denominator = (reserve_in as u128)
        .checked_mul(1000)
        .ok_or(SwapError::Overflow)?
        .checked_add(amount_in_with_fee)
        .ok_or(SwapError::Overflow)?;

    let out = numerator
        .checked_div(denominator)
        .ok_or(SwapError::DivByZero)?;

    u64::try_from(out).map_err(|_| SwapError::Overflow.into())
}
```

Ahora siembra el bug. Reemplaza la línea `numerator` verificada con una multiplicación `u64` pelada:

```rust
// programs/token-ticket-swap/src/math.rs  (seeded bug - DO NOT SHIP)
let numerator = ((amount_in * 997) * reserve_out) as u128; // u64 math wraps FIRST; the cast launders it
```

Un `u64 * u64` que excede `u64::MAX` hace wrap en vez de promover, así que para reservas grandes pero plausibles el numerador se colapsa a un valor chico, `out` vuelve mal, y la `k` del pool cae. El `as u128` del final es lo que mantiene el crimen callado: corre *después* del daño, convirtiendo el valor que ya hizo wrap al tipo que espera el `checked_div(denominator)` sobreviviente, así que la edición se queda en una línea y el build se queda verde. Un humano que lee esta línea ve una multiplicación y un cast. El fuzzer ve una recta numérica y se va a caminar justo por el borde.

Un detalle del build decide si esto hace wrap o entra en panic, y es el mismo de la lección pasada: el workspace generado de Anchor pone `overflow-checks = true` en el profile de release, así que sobre un scaffold intacto esto entra en panic. Un panic igual dispara al fuzzer, así que el ejercicio funciona de cualquier manera, pero el crash que sacas es un abort en vez de una violación de invariante. Para ver la versión de wrap silencioso, la que da genuinamente más miedo, pon `overflow-checks = false` en el `[profile.release]` del workspace antes de construir, y déjalo como estaba después. Anota eso como su propia línea en el archivo de la checklist: cuál de las dos viste es un hecho sobre tu build, no sobre el bug.

Checkpoint: `anchor build` tiene éxito y tus pruebas de swap existentes siguen pasando, porque todas canjean contra un pool de 1,000,000 / 1,000,000 donde nada se acerca a `u64::MAX`. Esa es la parte inquietante y la razón por la que lo sembraste: el bug está adentro, la suite está verde, y nada de lo que ya escribiste lo notó. Si en cambio el build falla, también cambiaste los casts en las líneas de alrededor, y el bug sembrado tiene que ser exactamente una línea.

![Una tarjeta de código anotada que muestra una multiplicación pelada de reservas en u64 haciendo wrap en release, colapsando el producto constante, al lado del arreglo verificado en u128.](assets/v08-annotated-code.png)

### 5. Completa el invariante y córrelo (el repliegue empieza acá)

Abre el banco de pruebas que escribió el scaffold. Tiene acciones ya descubiertas a partir de tu programa y un invariante con un agujero. La forma de Crucible es chica: un struct de fixture, un bloque `impl` donde cualquier método llamado `action_*` se vuelve una transición de estado que el fuzzer puede disparar, y una función `#[invariant_test]` que corre después de cada acción.

<!-- verify: expect-fail fuzz scaffold with a deliberate TODO - the reader adds prev_k and its initializer -->
```rust
// fuzz/token_ticket_swap/src/main.rs
use crucible_fuzzer::*;

#[derive(Clone)]
struct SwapFixture {
    ctx: TestContext,
    pool: Pubkey,
    reserve_arcade: Pubkey,
    reserve_ticket: Pubkey,
    prev_k: u128,          // YOU add this field; nothing else in the scaffold needs it
}

#[fuzz_fixture]
impl SwapFixture {
    pub fn setup() -> Self {
        // Deploys the swap, creates a pool with starting reserves, funds traders,
        // and returns the fixture. (scaffolded, except the last line.)
        let mut f = Self { /* scaffolded */ };
        f.prev_k = f.k();      // YOU add this: seed the baseline before any trade
        f
    }

    // Any `action_*` method is auto-discovered as an action the fuzzer can choose.
    pub fn action_swap(&mut self, #[range(0..4)] trader: usize, amount_in: u64) {
        // Fires one swap with a fuzzer-chosen trader and amount. (scaffolded.)
        // The invariant below runs AFTER this returns, so do not update prev_k here;
        // the invariant updates it once it has compared.
    }

    // Reads the two reserve token accounts back and returns their amounts.
    pub fn reserves(&self) -> (u64, u64) {
        let a = self.ctx.token_amount(&self.reserve_arcade);
        let b = self.ctx.token_amount(&self.reserve_ticket);
        (a, b)
    }

    pub fn k(&self) -> u128 {
        let (a, b) = self.reserves();
        (a as u128) * (b as u128)
    }
}

#[invariant_test]
fn constant_product_holds(fixture: &mut SwapFixture) {
    let k_now = fixture.k();

    // TODO (yours): assert the constant product never SHRINKS across a trade,
    // then update prev_k so the next action compares against this one.
    // Use the fuzz_assert_* macros, NOT assert!: a bare assert! panics the whole
    // fuzzer process, while fuzz_assert_* records the violation as a crash and
    // lets the run continue.
    //
    //   fuzz_assert_ge!(k_now, fixture.prev_k);
    //   fixture.prev_k = k_now;
    let _ = k_now;
}
```

El agujero es la aserción, y es el punto entero del banco de pruebas, así que piensa en qué quiere decir "correcto" antes de escribirla. Un swap nunca tiene que dejar que `k` se encoja. Tu swap cobra 0.3%, así que `k` normalmente va a crecer; uno sin comisión la mantendría exactamente igual. La aserción que es verdadera bajo las dos es `k_now >= k_before`, que es `fuzz_assert_ge!`. Fíjate dónde se actualiza `prev_k`: en el invariante, después de la comparación, no en `action_swap`. Actualízalo en la acción y comparas un valor contra sí mismo y la aserción no puede fallar nunca, que es la forma más común de todas en la que un banco de pruebas de fuzz vuelve limpio sin hacer nada. Escribe esas dos líneas, y después corre el build sembrado:

```bash
anchor fuzz run token_ticket_swap constant_product_holds --release --stateful
```

Estás esperando que la corrida se detenga y reporte un crash. Con la multiplicación `u64` sembrada en su lugar, lo va a hacer, y rápido, porque el fuzzer está guiado por cobertura hacia la rama donde las reservas se hacen lo bastante grandes para hacer wrap. Te entrega un artefacto de crash: una secuencia de inputs concreta, minimizada, repetible que violó tu invariante.

![Un diagrama de flujo del loop de crash-a-limpio, de la siembra y el invariante al artefacto de crash, la repetición, el parche y el re-fuzz limpio con export de LCOV.](assets/v09-flowchart.png)

### 6. Repite el crash, parchea, y vuelve a fuzzear hasta limpio

Repetir no es opcional. Un crash que no puedes reproducir es un rumor. Crucible escribió la secuencia que falla en `fuzz/token_ticket_swap/crashes/constant_product_holds/`, así que lista lo que guardó, y después repite una exactamente:

```bash
anchor fuzz show token_ticket_swap                          # list the saved crashes
anchor fuzz show token_ticket_swap <crash_file> --replay    # re-run that exact sequence
```

Míralo volver a correr la misma secuencia de trader/monto y disparar la misma aserción. Esa es tu demostración de que el artefacto es real y determinista. Ahora parchea: pon de vuelta el numerador `u128` verificado exactamente como la versión correcta de arriba, y restaura `overflow-checks` si lo apagaste. Vuelve a correr el mismo comando del paso cinco:

```bash
anchor fuzz run token_ticket_swap constant_product_holds --release --stateful
```

Esta vez debería correr sin producir un crash. Y acá está la disciplina que la lección entera está construida para instalar: esa corrida limpia no quiere decir "seguro." Quiere decir "los invariantes que escribí, sobre las acciones que definí, durante todo el tiempo que la dejé correr, no encontraron nada." Dite esa oración a ti mismo cada vez que una corrida vuelve verde. Después vuelve a correrla con la cobertura prendida, para que veas cuánto del programa ejercitó de verdad el fuzzer:

```bash
# Same run, with LCOV coverage written out.
anchor fuzz run token_ticket_swap constant_product_holds --release --stateful --coverage \
  --lcov-out ./fuzz-coverage.lcov
```

Ese archivo LCOV es una lectura de qué líneas alcanzó el fuzzer, y `genhtml` lo va a convertir en algo navegable. Fíjate qué comando lo produjo: la cobertura de fuzz sale de `run --coverage`, mientras que el comando aparte `anchor coverage` reporta sobre las trazas de `anchor test`, no sobre las del fuzzer. De cualquier manera, la cobertura te dice dónde ha estado y dónde no ha estado la búsqueda; una cobertura baja en una rama crítica quiere decir que no la exploraste, no que la rama sea segura. La cobertura es el mapa, `--stateful` es el vehículo.

## Challenge

Dos partes. La primera termina el loop; la segunda eres tú solo.

**Completion.** `k_now >= k_before` es una aserción débil. Tu swap cobra 0.3%, así que no es meramente verdad que `k` no se encoja, es verdad que `k` crece al menos por la contribución de la comisión en cualquier canje distinto de cero. Aprieta el invariante para que diga eso: haz assert de `k_now > k_before` cada vez que la acción movió tokens de verdad, y de `k_now == k_before` cuando no. Vas a necesitar que `action_swap` registre si el canje tuvo éxito, ya que un revert por slippage es un no-op legítimo.

Después corre `anchor fuzz run token_ticket_swap constant_product_holds --release --stateful` y manéjalo hasta un crash que repitas y parchees, o hasta una corrida limpia con un reporte LCOV. Fíjate en el modo de falla interesante de acá: una aserción más apretada puede crashear en un canje *legítimo*, porque la división entera quiere decir que un `amount_in` lo bastante chico redondea la comisión hasta hacerla desaparecer del todo y `k` genuinamente no se mueve. Si eso pasa, el fuzzer encontró un bug en tu invariante, no en tu programa, y saber cuál de los dos estás mirando es la habilidad.

**Solo.** Siembra exactamente un bug nuevo en el swap. Elige algo que una checklist no atraparía: un paso de redondeo que trunca a favor del pool en cada canje, o una comisión aplicada a `amount_out` en vez de a `amount_in`. Antes de correr nada, escribe tu predicción: ¿lo va a encontrar el fuzzer, y si lo hace, va a necesitar `--stateful`? Después corre `anchor fuzz run token_ticket_swap constant_product_holds --release --stateful`, y si crashea, repite con `anchor fuzz show token_ticket_swap <crash_file> --replay` para confirmar la secuencia exacta. Parchéalo. Vuelve a fuzzear hasta limpio. Compara lo que pasó con tu predicción. El bug de redondeo en particular es un buen maestro: un solo canje pierde una fracción de un lamport, invisible para una prueba de un solo canje, pero encadenado a lo largo de docenas de canjes stateful el residuo se acumula hasta que tu invariante se dispara.

Acéptalo cuando: se produce y se repite un artefacto de crash para tu bug sembrado, el bug está parcheado, el target vuelve a fuzzear limpio con un reporte LCOV, y tu checklist de auditoría está completamente verde con un número de línea en cada fila.

## ¿Funcionó, y qué no demuestra?

Terminaste con esta lección cuando puedes mostrar cuatro cosas: una checklist verde, un artefacto de crash repetido, un re-fuzz limpio y un reporte LCOV. Si tu corrida nunca crasheó con el bug sembrado, la causa habitual es un `--stateful` faltante o un invariante que no hace assert de nada de verdad (un `assert!(true)` disfrazado). Si crashea y no puedes repetirlo, parcheaste antes de guardar el artefacto. Arregla el orden: crash, repetición, parche, re-fuzz.

![Una checklist de cuatro filas que empareja cada artefacto requerido con el error que explica su ausencia, bajo una tira que fija el orden como crash, repetición, parche, y después re-fuzz.](assets/v10-table.png)

Ahora la parte que te mantiene honesto, porque es fácil alejarse de una corrida verde sintiéndote terminado. El fuzzing y una checklist suben tu confianza. Nunca demuestran la ausencia de bugs. Una corrida limpia quiere decir "no encontrado todavía," que es una afirmación real, útil, y estrictamente más débil que "seguro." Vale la pena ser preciso sobre la brecha. Una corrida de fuzz limpia es una afirmación de caso promedio: sobre los inputs y las secuencias que el fuzzer llegó a explorar en el tiempo que le diste, ningún invariante se rompió. El bug que te arruina normalmente es un objeto de peor caso, un solo input angosto en un rincón al que la búsqueda no llegó antes de que dieras el día por terminado. El fuzzing guiado por cobertura angosta esa brecha dirigiendo hacia ramas inexploradas, pero no la cierra, y no hay largo de corrida que convierta un "limpio de caso promedio" en un "seguro de peor caso."

El argumento más fuerte para esa humildad viene del framework sobre el que estás parado. La propia suite de pruebas de Anchor lleva testigos de Miri, que verifican comportamiento indefinido en código unsafe, y configs de Kani, que hacen model-check de propiedades específicas. Y el fuzzing encontró cuatro bugs de corrección en el propio Anchor, rastreados como la issue #4431. El framework está fuzzeado y verificado contra comportamiento indefinido tan fuerte como te pide que verifiques tu programa, y *aun así* encontró cuatro cosas. Si eso es verdad de código escrito y revisado por la gente que construyó el framework, asume que es verdad del tuyo.

![Un diagrama vertical en capas de la superficie de confianza, que va desde tu programa hacia abajo pasando por las verificaciones de Miri y Kani de Anchor y por OtterSec como guardián hasta llegar a una advertencia de revisión que este curso aporta por su propia autoridad, porque el tag fijado no entrega ninguna.](assets/v11-timeline.png)

Ese único guardián es en sí mismo un hecho que vale la pena rumiar. OtterSec custodia el framework, publica los crates, corre el registry de builds verificados contra el que verifica `anchor verify`, y firma el tag v2 con una clave GPG (trixter-osec). Una sola organización sostiene un montón de la cadena de suministro, que es eficiente y también una concentración que deberías conocer. Se empareja con una advertencia que este curso tiene que aportar por su propia autoridad, porque el proyecto no lo hace: ve a buscar en el tag fijado y no vas a encontrar ninguna página de advertencias — el README de lang-v2 dice "v2 is secure by default for users" y se detiene. Así que toma la oración de la auditoría que acabas de correr, no de una cita: los defaults no son un sustituto de la revisión, el fuzzing y el modelado de amenazas específico de producción. Un solo guardián, una release candidate no auditada, y cuatro bugs encontrados por el fuzzer en el framework mismo son el argumento entero, y alcanzan.

Dos notas honestas más para cerrar la superficie de confianza. Primero, los guardrails que trataste de apagar allá en la lección de CU — el flip que se tragó la unificación de features de cargo — son una red de seguridad de runtime default-on: atrapan cosas como un despacho con id de programa equivocado o el acceso mutable a una cuenta de solo lectura en runtime. Apagarlos ahorra un poco de tamaño de binario y de CU, y quita una red justo cuando un fuzzer tiene más probabilidad de estar empujando tu programa hacia un estado malo. Fuzzea con los guardrails prendidos. Entrega con ellos apagados solo después de que el fuzzing y la revisión hayan mostrado que nada de lo que habrían atrapado sigue vivo. Esa es una decisión de seguridad contra velocidad, y ahora puedes tomarla deliberadamente en vez de por defecto.

Segundo, sobre tooling: vas a oír hablar de Trident, el fuzzer de Ackee, y es un proyecto real. Pero no es el camino incorporado, y su cadencia de releases se estancó, el último estable es 0.12.0 del 2025-11-27, con un pre-release 0.13.0-rc.4 sentado desde el 2026-05-14. Anchor eligió Crucible y lo cableó en el CLI. Echa mano de `anchor fuzz` primero; Trident es una opción de respaldo para evaluar, no el default.

El programa ahora está tan endurecido como puedes ponerlo a mano y a máquina, con la checklist y el fuzzer los dos en verde y los dos honestamente etiquetados como "ningún bug encontrado todavía." Ese es el lugar correcto para dejarlo, porque la amenaza que sigue no está en el código. El módulo que viene, el swap deja tu máquina: generas un cliente tipado para que otras personas lo puedan llamar, demuestras un build verificable para que puedan confiar en que el bytecode calza con la fuente, y razonas sobre quién tiene la clave de upgrade, que es la única superficie de ataque que ninguna cantidad de fuzzing verde puede cerrar.
