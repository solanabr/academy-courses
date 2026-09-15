# El loop medido: recompra CU un cambio a la vez

La lección pasada apuntaste cuatro instrumentos al swap: una aserción de CU en Mollusk, un flamegraph con `--profile`, el debugger de anchor, y un reporte de cobertura. Te fuiste con un número de CU de línea base para la instrucción `swap_arcade_for_tickets` y el nombre del frame más caliente del flamegraph. Ese número es la cosa que ahora atacas.

Acá está la tentación, y quiero nombrarla antes de que la sientas. Tienes un número y un frame gordo. La jugada obvia es cambiar cinco cosas a la vez, volver a correr la prueba, ver caer el número, y festejar. Haz eso y no aprendiste nada. No vas a saber cuál de los cinco cambios ayudó, cuál hizo daño, cuál canceló a otro, y vas a entregar los cinco a ciegas. Un número más chico que no puedes explicar no es una optimización. Es una coincidencia a la que te encariñaste.

Así que antes de cualquier teoría, haz la única cosa sobre la que se apoya toda esta lección: vuelve a leer tu línea base. No de memoria, no de la nota que escribiste la lección pasada. Léela recién salida de la máquina, ahora mismo, porque es el "antes" de cada medición que sigue.

```bash
# Build FIRST, then read. Mollusk measures whatever .so SBF_OUT_DIR points it at,
# so a build with different flags is a different measurement wearing the same name.
cargo build-sbf
export SBF_OUT_DIR=$PWD/target/deploy   # a fresh shell needs it again; see m06-l1
cargo test -p token-ticket-swap trade_cu_baseline -- --nocapture
```

Anota el entero que imprime. Ese es el único número de esta lección en el que tienes permiso de confiar sin volver a medirlo, y hasta ese lo acabas de volver a medir. Todo de acá en adelante es: cambia exactamente una cosa, corre esto otra vez, y deja que la diferencia entre los dos números sea el argumento entero.

## Resumen

No estás escribiendo lógica nueva de programa esta lección. El swap conserva su comportamiento. Lo que atacas es su costo, con la misma medición de la lección pasada corrida dos veces, una antes y una después de una sola edición — y la primera palanca que acciones se va a negar a mover el número para nada, lo que termina enseñando más de lo que habría enseñado una ganancia.

Acá está la forma de esto:

- **El método es un loop, no una bolsa de trucos.** Línea base (ya la tienes) después cambia exactamente una cosa después vuelve a medir la misma instrucción después quédate o revierte según el delta. La disciplina es la lección. Las tres palancas de abajo, dos feature flags y una refactorización, son nada más cosas para pasar por él.
- **Palanca uno: `guardrails` apagado — y el cero que enseña.** `guardrails` es un conjunto de redes de seguridad de runtime prendidas por defecto en Anchor V2, y apagarlo *debería* devolver CU y unos 300 bytes de binario. Sobre R4 no devuelve exactamente nada, y el trabajo del lab es cazar ese cero y leer la razón directo del grafo de dependencias: la unificación de features de cargo, con `anchor-spl` como el borde que vuelve a prender el flag. Un cambio que nunca llegó al binario es el otro modo de falla del loop, y es igual de medible que una ganancia.
- **Palanca dos: `const-rent` prendido.** `const-rent` pliega la constante de rent en tiempo de compilación, y ahorra alrededor de 85 a 90 CU por CPI de creación de cuenta. Su letra chica, escrita en el propio Cargo.toml de Anchor, es que la constante plegada queda obsoleta si la fórmula de rent cambia. Ese comentario cita SIMD-0194, que es exactamente una propuesta para cambiar la fórmula de rent.
- **El artefacto es una prueba de regresión.** Llevas el swap por un ciclo completo en el lab, cazas el cero, lo atribuyes, y codificas el presupuesto medido en una prueba llamada `cu_swap_regression` para que el día en que algo saque el canje por encima de él, el build falle.

El repliegue de esta lección: yo corro un ciclo completo de punta a punta en el lab, guardrails apagado, mido, y atribuyo un delta de cero al borde de dependencia exacto que se tragó el flag. Después tú corres el loop para una segunda palanca, `const-rent`, con el banco de pruebas entregado, y reportas el delta atribuido — uno real esta vez. El challenge de código es el peldaño solo: escribes desde cero la función de cotización optimizada y la haces pasar.

## El loop medido

Primero un momento rápido de desmitificación, porque dos palabras de esta lección suenan más pesadas de lo que son. Un feature flag de Cargo es nada más compilación condicional. `#[cfg(feature = "guardrails")]` se sienta delante de un bloque de código, y que ese bloque esté en tu binario depende de si el feature estaba prendido cuando compilaste. `guardrails` y `const-rent` son dos de esos flags que entrega Anchor V2. Prender o apagar uno no cambia tu fuente. Cambia qué líneas se queda el compilador. Ese es el mecanismo entero.

Ahora el método, que es el contenido de verdad.

Piensa en cómo establecerías que un solo cambio al diseño de un puente lo hizo más liviano. No cambiarías el acero, el tablero, y el cableado todos a la vez y después lo pesarías. Cambiarías un solo elemento, lo pesarías, lo volverías a dejar como estaba si quedó más pesado. La razón no es ser quisquilloso. Es que un delta solo tiene sentido causal cuando se movió exactamente una variable. Cambia dos y el número que sacas es una suma que no puedes descomponer. Esta es la idea más vieja del método experimental, y es exactamente igual de cierta para unidades de cómputo que para cualquier cosa que puedas pesar.

Así que el loop son cuatro pasos, y el paso dos es estructural:

1. **Mide** la instrucción. Esto ya lo tienes: `trade_cu_baseline` imprimió un número.
2. **Cambia exactamente una cosa.** Un feature flag, o una refactorización. No dos.
3. **Vuelve a medir** la misma instrucción, de la misma forma, con la misma fixture.
4. **Quédate o revierte** según el delta, y anota qué único cambio lo causó.

![Un loop cíclico de medir, cambia exactamente una cosa, volver a medir, y quedarte-o-revertir, con la regla de una sola variable marcada como el paso estructural.](assets/v01-flowchart.png)

¿Por qué esto vale una lección entera en vez de una oración? Porque el modo de falla es seductor. Tres cambios simultáneos y una caída total de CU se sienten como progreso. Pero ese total podría esconder fácil una regresión: un cambio ahorró 400 CU, otro costó 200, un tercero no hizo nada, y entregaste la regresión de 200 CU porque la suma igual bajó. La cargarías para siempre, invisible, porque nunca la aislaste. El loop es la cosa que hace real una ganancia. El delta es la evidencia, y la evidencia exige un experimento controlado.

Con el método en la mano, acá están tres palancas concretas para pasar por él. Las dos primeras son feature flags, deliberadamente distintas de carácter: una saca algo, una pliega algo adentro, y sus trade-offs apuntan en direcciones opuestas. La tercera no es un flag para nada, que es el punto de incluirla.

### Palanca uno: apagar guardrails

`guardrails` es un feature prendido por defecto. Ese "prendido por defecto" importa: quiere decir que tu línea base de la lección pasada ya se midió con las redes de seguridad puestas. Son verificaciones de runtime que el framework inserta de tu parte, y cuestan CU en cada corrida porque se ejecutan en cada corrida.

Compila con `guardrails` apagado, sobre un crate donde el flip de verdad aterriza, y pasan dos cosas. El binario se encoge (la cifra propia de Anchor es alrededor de 300 bytes, medida sobre su programa de benchmark), y la instrucción sale más barata, porque esas verificaciones ya no corren adentro de ella. Sostén esa expectativa con cuidado, porque el lab está por violarla: sobre R4 el flip aterriza en nada, el medidor no se mueve, y la razón es un borde de dependencia que vas a leer con tus propios ojos.

Acá está la parte que no te voy a dejar saltear, porque las lecturas equivocadas de esto son las tentadoras. `guardrails` no es un lint. No es una gentileza de solo-compilación. No es una configuración de Mollusk. Cambia el programa compilado — *cuando de verdad se apaga*. Lo que estarías entregando a cambio son las verificaciones mismas, redes de seguridad de runtime de verdad, con la CU volviendo porque el trabajo salió. Y la precondición que el lab existe para grabarte: un feature de cargo está apagado solo cuando *nada en ningún lugar de tu grafo* lo vuelve a prender.

¿Y qué son las redes, concretamente? Esto importa, porque no puedes argumentar un invariante que no puedes nombrar. La familia guardrails es la clase de verificaciones defensivas que el framework inserta alrededor de tu handler para que una llamada malformada falle limpio en vez de hacer algo peor. Piensa en las garantías sobre las que te has estado apoyando sin escribirlas: que una cuenta que te entregan sea de verdad propiedad del programa que crees que es su dueño, que un discriminador coincida con el tipo de cuenta como el que la deserializaste, que un camino aritmético que podría hacer wrap quede cazado en vez de truncar en silencio, que un límite que diste por cumplido de verdad se haya cumplido. En todas y cada una de las llamadas corren esas verificaciones, y cada una de ellas debita alguna CU. Esa es la forma de la ganancia cuando las apagas, y es también la forma exacta del riesgo. No hiciste imposible la llamada malformada. Hiciste que el framework dejara de verificarla.

Así que el verdicto honesto sobre guardrails apagado es: es defendible solo para código cuyos invariantes puedas argumentar tú mismo. Si puedes mirar el handler `swap_arcade_for_tickets` y decir, en voz alta y correctamente, "las reservas son siempre cuentas distintas, la escala de la comisión es fija, la salida está acotada por `reserve_out`, y nada acá puede hacer underflow porque la cotización devuelve 0 en un pool vacío", entonces hiciste el argumento que el framework estaba haciendo por ti, y puedes bajar las redes. Si no puedes hacer ese argumento, déjalas prendidas. Entregar guardrails apagado sin un argumento de invariantes escrito no es una optimización. Es una apuesta que no sabías que hiciste.

![Una comparación de guardrails apagado, que saca verificaciones del camino caliente a cambio de CU y 300 bytes pero te hace dueño de los invariantes, contra const-rent prendido, que pliega la constante de rent y puede quedar obsoleta.](assets/v02-comparison.png)

### Palanca dos: plegar const-rent

`const-rent` va para el otro lado. En vez de quitar una verificación, quita un cómputo, plegando una constante.

Cada vez que un programa crea una cuenta a través de una CPI, tiene que fondear esa cuenta hasta el mínimo exento de alquiler, o la cuenta no puede sobrevivir. Ese mínimo es una función del tamaño en bytes de la cuenta, y derivarlo quiere decir correr la fórmula de rent: un costo por byte más un overhead fijo, multiplicado para el tamaño que estás asignando. El runtime puede computar eso en cada creación de cuenta, o, si el tamaño se conoce en tiempo de compilación, la respuesta puede quedar incrustada como un literal. Ese literal incrustado es lo que `const-rent` pliega adentro, así que el runtime se saltea el cómputo. El ahorro es alrededor de 85 a 90 CU por CPI de creación de cuenta.

Fíjate que escribí un rango, no un número solo, y voy a ser terco con eso. Esta es la misma disciplina que hizo que este curso se negara a congelar el multiplicador del propio benchmark de Anchor la lección pasada. El ahorro aparece como cerca de 85 en el comentario del propio feature en el `Cargo.toml` de `anchor-lang` y cerca de 90 en la entrada del changelog de V2 que lo introdujo, y arriba de eso puede irse a la deriva, así que congelar un dígito sería falsa precisión disfrazada de rigor. Ve a leer las dos antes de citar cualquiera de ellas; están a cuatro líneas de distancia en un repo del que ya tienes un checkout. La forma honesta es el rango más una nota que diga volver a verificar. Un número de CU es determinístico para un programa, una entrada y un toolchain dados, así que este rango no es ruido de corrida a corrida. Es desacuerdo entre fuentes más riesgo de deriva, que es una cosa distinta y más interesante.

Y la deriva es la lección de verdad acá. `const-rent` pliega la constante de rent, que es correcta solo mientras la fórmula de rent que la produjo se quede quieta. El propio comentario del Cargo.toml de Anchor dice exactamente esto, y cita SIMD-0194 para decirlo. SIMD-0194 se titula "Deprecate Rent Exemption Threshold". Es una propuesta Core, Accepted, presentada allá por noviembre de 2024, que cambiaría cómo se deriva el mínimo exento de alquiler. Si se activa, la constante que plegaste se vuelve equivocada, en silencio, y tu creación de cuenta ahora está computando rent contra un número obsoleto.

![Una línea de tiempo desde la presentación de SIMD-0194 en noviembre de 2024 en adelante, que muestra que la constante plegada de const-rent sigue válida solo hasta que la fórmula de rent cambie, lo que vuelve al flag un ítem de volver a verificar.](assets/v03-timeline.png)

Quédate un segundo con lo raro que es eso. Recurriste a un feature flag para raspar menos de cien unidades de cómputo de una creación de cuenta, y la letra chica te devolvió una pregunta viva de gobernanza de protocolo. Una edición de una línea en Cargo metió en tu build una dependencia del estado de activación de una SIMD. Esa es genuinamente la cosa más interesante de `const-rent`, y es por eso que el rango importa: no estás nada más citando un ahorro, estás citando un ahorro con una fecha de vencimiento que no controlas.

![Una barra de rango que abarca alrededor de 85 a 90 CU a lo largo de dos citas de fuentes, con una extensión punteada de deriva que muestra por qué un dígito congelado sería falsa precisión.](assets/v04-chart.png)

### Palanca tres: una refactorización, no un flag

Las dos primeras palancas eran feature flags, una edición en una línea de build. Al loop no le importa de dónde viene el cambio. Una refactorización a nivel de fuente pasa por él exactamente igual, y el swap tiene una sentada a la vista: la función de cotización misma.

Acá es donde vence el otro entregable de la lección pasada. Anotaste el frame de código propio más caliente debajo de la instrucción de canje, y en el swap de Quarters ese frame es `swap_out`, la cotización. Un flamegraph no te dice qué hacer con un frame gordo; te dice dónde apuntar el loop. Así que apúntalo ahí.

La cotización que entregaste, `swap_out`, ya promueve a `u128` antes de multiplicar, porque el módulo 5 puso eso como el criterio de aceptación. Lo que no hace es exponer la comisión, y hay una pregunta de diseño de verdad escondida en cómo se escribe una versión general. ¿Recurres a `checked_mul` sobre `u64` y ramificas según el overflow, que es seguro pero pone una rama en el camino caliente? ¿O promueves primero a `u128`, donde la multiplicación no puede hacer overflow por construcción, así que no se necesita ninguna verificación? Esas dos no son igual de baratas, y no son igual de seguras, y la única forma de saber el canje real para tus reservas es pasar las dos por el loop. ("Por construcción" lleva un calificador que derivaste en m05-l2 — dos factores `u64`. El challenge del final de esta lección agrega un tercero.)

Eso es lo que es el challenge de código del final de esta lección: una cotización generalizada, `get_amount_out`, con la comisión levantada a un parámetro, escrita desde cero y después medida. Es una función aparte del `swap_out` que entregaste, no una edición de él, así que puedes tener las dos y comparar. Escríbela, métela en el handler detrás de un cambio de una línea en la llamada, y vuelve a medir el canje. Si el delta es una ganancia y la función sigue coincidiendo con sus salidas de referencia, quédate con ella. Si tu versión de `checked_mul` más rama salió más barata sobre tus reservas, para eso está el loop, y el número decide, no tu intuición sobre cuál se lee más rápido.

![Una comparación lado a lado de una multiplicación verificada con una rama contra promover a u128, que muestra que las dos son seguras y que solo una medición sobre reservas de verdad decide cuál cuesta menos.](assets/v05-comparison.png)

El punto que generaliza más allá de esta única función: una palanca es cualquier cosa que puedas accionar y volver a medir en aislamiento. Un feature flag es la clase más limpia porque no mueve nada en tu fuente. Una refactorización también es una palanca, siempre que hagas una y solo una, y después midas. El método no cambia. Solo cambia la cosa que estás cambiando.

### El trade-off, dicho sin adornos

Cada CU que compras tiene un precio, y las palancas le ponen precio distinto.

Guardrails apagado, donde aterriza, devuelve CU y unos 300 bytes al precio de las redes de seguridad de runtime — nunca lo entregues sin un argumento de invariantes que puedas defender, porque el riesgo que tomas es la corrección: ahora la verificación eres tú. Sobre R4 no aterriza para nada, y el riesgo se invierte en algo más callado: una línea de build que *dice* redes apagadas mientras el binario todavía las carga documenta una optimización que nunca pasó, y se queda ahí esperando el día en que el grafo cambie por debajo.

Const-rent cambia un cómputo de runtime por una constante de tiempo de compilación que puede irse a la deriva si la fórmula de rent cambia. El riesgo que tomaste es la obsolescencia. Necesita una nota de volver a verificar amarrada a SIMD-0194, no un commit de tirar-y-olvidar.

Y hay un tercer trade-off que no es sobre ninguno de los dos flags. Es sobre dónde apuntas el loop. Optimizar una instrucción fría es esfuerzo desperdiciado. 50 CU medidas raspadas de un camino al que nadie le pega son ruido, no una ganancia, y peor, son ruido que te costó tiempo de verdad y muchas veces compró un riesgo de verdad. Esto es directamente relevante para las palancas, porque apuntan a instrucciones distintas. Guardrails apagado apunta al camino caliente del canje, el que corre en cada swap, miles de veces — donde aterriza, al menos. Const-rent ayuda a las CPI de creación de cuenta, que en este programa quiere decir el montaje del pool, una instrucción que corre una vez cuando levantas el pool y nunca más mientras haya canjes.

Los dos objetivos son legítimos, pero el peso no es el mismo. Una CU ahorrada en el camino del canje se ahorra en cada canje para siempre, así que se acumula con el volumen. Una CU ahorrada en el init de una sola vez se ahorra exactamente una vez. Eso no vuelve inútil la optimización del init, levantar una cuenta más barato sigue siendo más barato, pero sí quiere decir que no deberías gastar una tarde raspando el camino frío mientras el caliente todavía tiene un frame gordo que no tocaste. Ordena las palancas por frecuencia por delta, no por delta solo. El instrumento que te dice la frecuencia no es el flamegraph, es tu propio conocimiento de cómo se llama al programa en realidad.

Lo que vale decir sin adornos contra lo que esta lección te pide después, porque parece una contradicción. El Completion de abajo corre el loop sobre `const-rent`, una palanca de camino frío, mientras el frame caliente sigue sin tocarse hasta el challenge. Ese orden es pedagógico, no una recomendación: `const-rent` es la segunda vuelta al loop más limpia porque su trade-off es deriva y no corrección, así que llegas a practicar el método sin tener además que defender un argumento de invariantes — y, después del cero del lab, es la primera vuelta donde el número de verdad se mueve. En tu propio programa harías primero el camino caliente. Acá estás aprendiendo el loop, y el loop es más barato de aprender sobre la palanca que no puede lastimarte.

![Una comparación de dos columnas que pesa el camino caliente del canje contra el camino frío del init del pool, y muestra que el mismo delta de CU debería ordenarse por frecuencia por delta.](assets/v06-comparison.png)

La otra mitad de apuntar bien el loop es el aislamiento, y es donde el método entero vive o muere.

![Un diagrama de dos paneles que contrasta un cambio atribuible contra tres cambios simultáneos cuyo total neto esconde una regresión que se entrega invisible porque la suma igual bajó.](assets/v07-diagram.png)

## Lab: corre un ciclo completo sobre el swap

Ejemplo trabajado, con rueditas. Yo corro una pasada completa del loop contra la instrucción `swap_arcade_for_tickets` usando `guardrails` como la palanca, y aterrizamos en un presupuesto medido codificado en una prueba de regresión. Aviso de antemano sobre la forma del final: la respuesta del loop acá es un cero, y el cero — cazado, atribuido, y nombrado — es el hallazgo. Corre cada comando en tu propio checkout mientras lo lees; los números que te salgan no van a ser los míos, y ese es el punto.

El repliegue de la ayuda es explícito. Yo corro el ciclo de guardrails de punta a punta acá, con el delta y la decisión dichos en voz alta. Después tú vuelves a correr el loop para `const-rent` sobre la instrucción de init del pool, con el banco de pruebas provisto, en la sección de Completion de abajo. Y el challenge de código después de eso, la cotización optimizada escrita desde cero, es solo tuyo.

### 1. Confirma que la línea base sigue siendo la línea base

No confíes en el número de tus notas. Vuelve a leerlo, porque una línea base obsoleta envenena cada delta que venga después.

```bash
# The "before". Build the defaults configuration, then measure THAT build.
cargo build-sbf
ls -l target/deploy/token_ticket_swap.so     # write the byte size down too
cargo test -p token-ticket-swap trade_cu_baseline -- --nocapture
```

Anota el entero como `BEFORE`, y el tamaño en bytes del `.so` al lado. Los dos son con `guardrails` prendido, porque ese es el default. La línea de build importa más de lo que parece: `cargo test` no vuelve a construir el artefacto on-chain, solo corre el banco de pruebas contra el `.so` que ya esté en el disco, así que cada medición de esta lección va precedida por el build cuyo costo está reportando. Sáltate un build y vas a medir la configuración anterior y atribuir el delta al cambio equivocado.

### 2. Cambia exactamente una cosa: guardrails apagado

La forma limpia de voltear un feature del framework es reenviarlo a través del `Cargo.toml` de tu propio crate de programa, así el interruptor es un flag en el comando de build y nada en tu fuente se mueve. Cablea el reenvío del feature una vez:

```toml
# programs/token-ticket-swap/Cargo.toml
# RC tags move fast: check the crate's Cargo.toml for the exact feature names on the
# branch you pin, then re-verify. anchor-lang's DEFAULT features are `alloc` +
# `guardrails`; const-rent is opt-in.
[features]
default = ["anchor-lang/guardrails"]      # safety nets ship ON by default
const-rent = ["anchor-lang/const-rent"]   # opt in to the compile-time rent fold

[dependencies]
# Program deps come from crates.io at 2.0.0-rc.1 (published 2026-08-12, re-verified
# 2026-08-23), exactly as m02-l1 pinned them: a published version is immutable, and
# it is the same release the tag-pinned CLI was built from.
# default-features = false makes guardrails toggleable, but it also drops `alloc`,
# the OTHER v2 default. Re-enable alloc here explicitly: otherwise your
# --no-default-features build moves TWO variables (and your allocator), not one.
anchor-lang = { version = "2.0.0-rc.1", default-features = false, features = ["alloc"] }
# The SPL surface R4 has carried since module 5 — and, as this lab is about to
# measure, the row that quietly decides the whole guardrails story.
anchor-spl  = "2.0.0-rc.1"
# The pins from m01-l2 — every program crate in this course carries them (issue #4937's class).
wincode = { version = "0.5", features = ["derive"] }
# The arcade-workspace row, unchanged since m02-l1 and confirmed in m06-l1 when Mollusk
# arrived: this is that same crate. The ceiling below 2.7 is the constraint that matters;
# 2.6.1 is still on wincode 0.5. Every member of the workspace carries this same row.
solana-address = ">=2.6.1, <2.7"

[dev-dependencies]
# Unchanged from m06-l1, and listed here so the whole crate is on one page: Mollusk,
# its token-program companion, plus the two rows that hold its graph on wincode 0.5.
mollusk-svm = "0.15.1"
mollusk-svm-programs-token = "0.15.1"
solana-sdk = "4"
solana-short-vec = ">=3.2.2, <3.3"
solana-signature = ">=3.4.1, <3.5"
spl-token = "9"   # the fixture's state types and spl_token::ID (m05-l1's pin)
```

Ahora el único cambio. Compila el programa con los features default apagados, lo que elimina `guardrails`:

```bash
# ONE change: build with guardrails off. Nothing else moves.
cargo build-sbf --no-default-features
```

Resultado esperado: un build limpio — y un tamaño en bytes que **no se movió**. Corre `ls -l` otra vez y compara contra el paso 1: idéntico, byte por byte. En la mayoría de los crates un tamaño que no se movió querría decir que el reenvío del feature no está cableado y que la medición que estás por hacer no mide nada. Tu reenvío está cableado exactamente bien, y el tamaño que no se movió te está diciendo igual que el flag nunca llegó al binario — esa contradicción es el hallazgo de verdad de este lab, y el paso 3 corre la medición igual antes de explicarla, porque la regla del loop es medir primero, explicar después.

Eso es todo. Cambiaste una cosa. Resiste agregar `--features const-rent` en la misma línea, porque entonces habrías movido dos variables y el delta sería una suma.

### 3. Vuelve a medir la misma instrucción, de la misma forma

```bash
# The "after" number. Identical command, identical fixture.
cargo test -p token-ticket-swap trade_cu_baseline -- --nocapture
```

Ese comando imprimió el número — y la prueba se quedó **verde**. Lee la línea: `trade consumed <N> CU`, el mismo entero que `BEFORE`, y `Check::compute_units(BEFORE)`, una igualdad exacta, sigue pasando. La alarma de la lección pasada está funcionando exactamente como se diseñó: fija el número, el número no se movió, así que nada se dispara. Anota `AFTER` igual, porque el loop lo exige, y computa el delta: `AFTER - BEFORE = 0`. Cambiaste una cosa y el medidor no se movió.

Un delta de cero tiene exactamente dos lecturas honestas, y el loop te obliga a elegir la correcta. O el cambio genuinamente no cuesta nada — o el cambio nunca llegó al binario. El tamaño en bytes que no se movió del paso 2 ya votó por la segunda. Ahora lee al culpable directo, con el instrumento construido para exactamente esta pregunta:

```bash
cargo tree -e features -i anchor-lang --no-default-features
```

```text
anchor-lang v2.0.0-rc.1
├── anchor-lang feature "alloc"
│   └── token-ticket-swap v0.1.0 (…/programs/token-ticket-swap)
│   └── anchor-lang feature "default"
│       └── anchor-spl v2.0.0-rc.1
│           ├── anchor-spl feature "default"
│           │   └── token-ticket-swap v0.1.0 (…/programs/token-ticket-swap)
│           └── anchor-spl feature "guardrails"
│               └── anchor-spl feature "default" (*)
├── anchor-lang feature "default" (*)
└── anchor-lang feature "guardrails"
    └── anchor-lang feature "default" (*)
```

Lee la última estrofa de abajo para arriba: `anchor-lang feature "guardrails"` lo prende `anchor-lang feature "default"`, y la flecha debajo de *eso* apunta directo a `anchor-spl v2.0.0-rc.1`. Tu fila `default-features = false` hizo su trabajo — tu propio borde dejó de pedir los defaults — pero `anchor-spl` declara un `anchor-lang = "=2.0.0-rc.1"` a secas, defaults y todo, y cargo **unifica los features a lo largo de cada borde del grafo**: un crate se compila para todos, así que cualquier borde suelto que pida un feature lo prende para todos ellos. Los features son aditivos por diseño; tu `false` no puede restar lo que agrega un borde hermano. Volteaste el flag, y el grafo lo volvió a voltear antes de que el compilador lo viera.

Así que la oración de atribución de este ciclo, dicha en voz alta y verdadera: "apagar guardrails no cambió nada, porque el borde de dependencia de anchor-spl mantiene el feature prendido". Esa oración es el punto entero del loop — un cero que puedes explicar le gana a una ganancia que no.

### 4. Quédate o revierte, con el argumento dicho

Acá está la decisión, y el cero la toma por ti: **revierte**. Saca `--no-default-features` de vuelta de la línea de build. No porque las redes tengan que quedarse — porque el flag no hace nada acá, y una línea de build que dice redes apagadas mientras el binario todavía las carga es peor que cualquiera de los dos estados honestos. Documenta una optimización que nunca pasó, y se queda ahí armada: el día en que el borde de anchor-spl cambie, tu flag "no-op" se vuelve en silencio un build real con redes apagadas que nadie nunca argumentó.

La disciplina de invariantes que exige el canje de redes apagadas no se desperdicia, igual — guárdala donde el cero la dejó. Sobre un crate que no está debajo de `anchor-spl` — un programa de pura lógica con solo `anchor-lang` en su grafo — este flip exacto aterriza, el binario se encoge, y la regla aplica completa: nunca entregues redes apagadas sin un argumento escrito que defienda cada invariante que las verificaciones estaban cubriendo. Para este swap el párrafo hasta sería escribible: las dos reservas son cuentas de token distintas por construcción, así que no hay aliasing que cazar; la cotización devuelve 0 en un pool vacío o de entrada cero y el handler revierte sobre una salida 0; la escala de la comisión es una constante fija; y la salida está acotada por `reserve_out`, un `u64`, así que el cast final no puede truncar. Escríbelo el día en que el flip pueda aterrizar. Hoy el grafo vetó el canje antes de que pudieras hacerlo.

![Una tabla de decisión que le hace de barrera a un build con guardrails apagado exigiendo un delta real, un argumento de invariantes escrito completo para el handler, y el argumento de verdad commiteado; si no, revierte.](assets/v08-comparison.png)

### 5. Codifica el número medido como una prueba de regresión

El ciclo del lab terminó en un revert, y igual deja un artefacto — este. Un número que mediste una vez es una historia; un número codificado en una prueba es una alarma, y el costo real del canje, con las redes prendidas, merece una. El paso de verificación de esta lección es una prueba llamada `cu_swap_regression`, y su trabajo es hacer fallar el build el día en que algo saque el canje de vuelta por encima de su presupuesto.

También reemplaza a `trade_cu_baseline`, deliberadamente, y no porque algo esté rojo — nada lo está. Un pin de igualdad exacta era la herramienta correcta para establecer una línea base una vez; como prueba permanente se pone roja en cada mejora igual que en cada regresión, lo que entrena a la gente a ignorarla. Borra `tests/cu_baseline.rs` una vez que la prueba nueva esté verde, y quédate con el módulo de fixture que usaba, porque esta también lo necesita.

```rust
// programs/token-ticket-swap/tests/cu_swap_regression.rs
use mollusk_svm::{result::Check, Mollusk};
use solana_sdk::{account::Account, instruction::Instruction, pubkey::Pubkey};

mod swap_fixture;   // the same module cu_baseline.rs used last lesson

// The same swap fixture: program, accounts, one swap ix.
fn fixture() -> (Mollusk, Instruction, Vec<(Pubkey, Account)>) {
    let program_id = token_ticket_swap::ID;
    let mut mollusk = Mollusk::new(&program_id, "token_ticket_swap");
    // The trade's CPI target, registered exactly as in m06-l1.
    mollusk_svm_programs_token::token::add_program(&mut mollusk);
    let keys = swap_fixture::keys(&program_id);
    let accounts = swap_fixture::build_swap_accounts(&keys);
    let ix = swap_fixture::build_swap_ix(&program_id, &keys);
    (mollusk, ix, accounts)
}

// The budget from this lab's measurement: your BEFORE (which is also your AFTER —
// that zero was the finding), plus a little headroom. This is not a wish. It is the
// number you measured, rounded up so a future toolchain bump that shifts the number
// a little does not flap the build.
// Set this before you run the test; the println below tells you what to set it to.
const TRADE_CU_BUDGET: u64 = 0; // <- set to your measured AFTER + a small headroom

#[test]
fn cu_swap_regression() {
    let (mollusk, ix, accounts) = fixture();

    // Assert the trade still succeeds, then assert it costs AT OR BELOW the budget.
    // Check::compute_units asserts EQUALITY, which is why the bound is a comparison
    // here instead: a change that gets cheaper should pass, not fail.
    let result = mollusk.process_and_validate_instruction(&ix, &accounts, &[Check::success()]);

    // Print before you assert, same as the baseline test did, so a red run still
    // hands you the number you need to set the budget to.
    println!("trade consumed {} CU (budget {})", result.compute_units_consumed, TRADE_CU_BUDGET);

    assert!(
        result.compute_units_consumed <= TRADE_CU_BUDGET,
        "trade regressed: {} CU consumed, budget is {} CU",
        result.compute_units_consumed,
        TRADE_CU_BUDGET
    );
}
```

![Un panel anotado que explica las cuatro líneas estructurales de la prueba de regresión: el presupuesto ganado más holgura, la verificación de éxito primero, una comparación de igual-o-por-debajo, y los dos números impresos cuando falla.](assets/v09-annotated-code.png)

Córrela:

```bash
cargo test cu_swap_regression -- --nocapture
```

Con `TRADE_CU_BUDGET` todavía en `0` falla e imprime el número que hay que poner. Ponlo, vuelve a correr, verde. De ahí en adelante, el día en que una refactorización saque el canje de vuelta por encima de ese presupuesto, esta prueba se pone roja y te dice en qué dirección se movió. Un flamegraph es algo que vas y miras. Esto es algo que mira por ti.

## Completion: corre el loop para const-rent

Rueditas a medio quitar. Una vuelta más al loop antes del peldaño solo, esta vez con `const-rent` como la palanca y el banco de pruebas entregado. Hay un detalle que es en sí mismo una lección: `const-rent` ayuda a las CPI de creación de cuenta, y `swap_arcade_for_tickets` no crea cuentas. Así que no midas el canje para esta. Mide la instrucción que levanta el pool, el init, porque esa es la instrucción que la palanca de verdad toca. Medir el canje acá te mostraría un delta de cero y te enseñaría la cosa equivocada.

El ciclo es idéntico en forma:

1. **Mide** la CU de la instrucción de init del pool. Esa es la instrucción que escribiste en el lab de swap del módulo 5, la que hace `init` de la cuenta `Pool` y crea las dos cuentas de token de reserva debajo del PDA del pool; cada uno de esos `init` es una CPI de create-account del System Program, que es precisamente la llamada para la que `const-rent` pliega la constante.

   No hay banco de pruebas provisto para ella, y escribirlo es el punto de este peldaño: ya tienes el patrón dos veces. Copia `cu_swap_regression.rs` a `tests/init_cu_baseline.rs` y cambia cuatro cosas. Renombra la fn `#[test]` misma de `cu_swap_regression` a `init_cu_baseline` — el filtro posicional de cargo coincide con nombres de *función*, nunca con nombres de archivo, así que sin este renombre los dos comandos de abajo filtran hasta `running 0 tests` y no imprimen ningún número. Construye la instrucción de init en vez de la instrucción de swap (`swap_fixture::build_init_ix`, que pertenece al mismo módulo de fixture que metiste la lección pasada). Pasa la lista de cuentas del init, que la fixture expone como `swap_fixture::build_init_accounts(&keys)`, y que difiere de la del swap porque el pool y las dos reservas tienen que llegar *sin inicializar*. Y elimina la aserción del presupuesto por completo; quédate solo con `Check::success()` y el `println!`, porque para este peldaño quieres el número impreso dos veces, no fijado.

   ```bash
   cargo build-sbf                                                 # build first, always
   cargo test -p token-ticket-swap init_cu_baseline -- --nocapture
   ```
2. **Cambia exactamente una cosa** respecto de tu build del paso 1: prende `const-rent` y no muevas nada más. El lab terminó con los defaults de vuelta prendidos — el flag de guardrails era un no-op y lo sacaste de la línea — así que esto es un flag sobre un build que por lo demás es default:

```bash
cargo build-sbf --features const-rent
```

Una asimetría que vale notar mientras lo tipeas: la unificación vetó la *quita* de guardrails, pero no puede vetar este *agregado*. Los features son aditivos, así que prender uno solo necesita que tu propio borde lo pida — que es exactamente por qué esta palanca puede aterrizar donde la anterior no pudo. La única variable que se mueve entre el paso 1 y el paso 3 es `const-rent`.

3. **Vuelve a medir** la misma instrucción de init, la misma fixture, el mismo comando. La línea de build de arriba ya volvió a construir con el flag nuevo; corre `cargo test -p token-ticket-swap init_cu_baseline -- --nocapture` otra vez y lee el segundo número.
4. **Reporta** la CU de antes, la CU de después, y la atribución de una línea: "prender const-rent ahorró N CU en la creación de cuenta del init del pool".

Tu delta debería aterrizar en el vecindario de la cifra de 85 a 90 CU por CPI de creación de cuenta, escalada por cuántas cuentas crea el init. Si vuelve cerca de cero, verifica que mediste la instrucción de init y no el canje. Esa es la forma más común en que este Completion sale mal, y es la misma trampa que perseguir un camino frío: mediste la instrucción que la palanca no toca.

## Challenge: la cotización optimizada de producto constante

Rueditas del todo quitadas. Este es el peldaño solo, y es el único donde escribes código y además corres el loop sobre él.

Allá en el módulo 5 construiste `swap_out`, la función de cotización con una comisión del 0.3% hardcoded, y la lección pasada la viste sentada ahí como el frame de código propio más caliente debajo de la instrucción de swap. Ahora generalízala. `get_amount_out` es la misma curva con la comisión levantada afuera a un parámetro, `fee_bps`, en basis points. Acá está su starter, y está mal a propósito:

```rust
/// Quote a constant-product swap output. THIS STARTER IS BROKEN:
/// it ignores fee_bps, multiplies in u64, and checks nothing, so its
/// quotes are wrong whenever fee_bps > 0, it can overflow on large
/// reserves, and it quotes the whole pool on an empty reserve_in.
const fn get_amount_out(reserve_in: u64, reserve_out: u64, amount_in: u64, fee_bps: u64) -> u64 {
    let _ = fee_bps; // the fee is being ignored
    let numerator = reserve_out * amount_in;
    let denominator = reserve_in + amount_in;
    numerator / denominator
}
```

Una palabra de esa firma es nueva desde que se entregó el `swap_out` del módulo 5, y está haciendo trabajo de verdad: `const fn`. El archivo calificado lleva afirmaciones en tiempo de compilación debajo de la función — el recurso de m03-l3, y lo que la calificación solo-de-compilación de verdad exige — así que el starter roto no construye: las filas ciegas a la comisión fallan afirmaciones cuyos mensajes nombran el caso, y la fila de pool profundo falla en la evaluación const misma con `attempt to multiply with overflow`, con rustc apuntando a la multiplicación `u64` exacta que estás acá para arreglar.

Dos cosas están mal en ella, y son las mismas dos que el challenge del módulo 5 te hizo arreglar en `swap_out`. Nunca aplica la comisión, así que toda cotización con `fee_bps > 0` le paga de más al trader. Y multiplica dos valores `u64`, así que un `reserve_out * amount_in` grande puede hacer overflow. Sobre el scaffold en el que de verdad estás parado eso es un panic de cualquier manera — el workspace que genera Anchor entrega `overflow-checks = true` en el perfil de release, y `cargo build-sbf` construye release, un hecho en el que se apoya m07-l2 — así que el modo de falla acá es un abort sobre el pool más profundo que tengas, que es el canje en el que menos quieres fallar. Saca esa línea del perfil, o hereda un crate que nunca la tuvo, y la misma multiplicación hace wrap en silencio en vez de eso, que es peor. Los dos ya los arreglaste una vez sobre una curva sin comisión. Esta es la misma reparación sobre la general — y el día en que estés sobre un crate donde un build con redes apagadas de verdad aterriza, una multiplicación que hizo wrap no tiene nada detrás que la sostenga, así que la versión verificada es la que quieres en tus dedos ahora.

Una tercera cosa está mal en ella que el módulo 5 ya te enseñó y que este starter elimina calladamente: no verifica sus entradas. Pon `reserve_in` en cero y el denominador se colapsa a `amount_in`, el `amount_in` se cancela, y la función cotiza todo `reserve_out` — el pool entero — a quien pregunte primero. Esa es la misma guarda con la que abre `swap_out`, y la versión general la necesita también.

Tu trabajo: reescribe `get_amount_out` para que aplique la comisión, rechace las entradas sobre las que la curva no está definida, y pase cada producto intermedio por un solo `u128` antes de una única división final de vuelta a `u64`. Mantén la firma exactamente como está congelada arriba, porque los casos de referencia de abajo la llaman por esa interfaz.

Tres empujoncitos, la misma forma que ya viste antes, generalizada a basis points:

- `amount_in_with_fee = amount_in * (10_000 - fee_bps)`
- `out = (reserve_out * amount_in_with_fee) / (reserve_in * 10_000 + amount_in_with_fee)`
- castea a `u128` antes de las multiplicaciones, castea de vuelta a `u64` después de la única división

Las salidas de referencia con las que tu solución tiene que coincidir:

```
get_amount_out(1_000_000, 1_000_000,  10_000, 30) == 9_871    // 0.3% fee, balanced pool
get_amount_out(5_000_000, 2_000_000, 100_000, 30) == 39_100   // 0.3% fee, uneven reserves
get_amount_out(1_000_000, 1_000_000,   1_000,  0) == 999      // zero fee = plain constant product
get_amount_out(1_000_000, 1_000_000, 500_000, 30) == 332_665  // large trade, missing fee is obvious

// ...and the inputs the curve is not defined on, which must degrade
// rather than panic or over-quote:
get_amount_out(0, 1_000_000, 10_000, 30)             == 0  // empty reserve_in: unguarded, quotes 1_000_000
get_amount_out(0, 0, 0, 30)                          == 0  // drained pool, zero quote: unguarded, 0 / 0
get_amount_out(1_000_000, 1_000_000, 10_000, 10_001) == 0  // fee past the scale: 10_000 - fee_bps underflows
get_amount_out(1_000_000, 1_000_000, 10_000, 10_000) == 0  // 100% fee: here 0 is the arithmetic answer
get_amount_out(u64::MAX, u64::MAX, u64::MAX, 30)     == 0  // numerator exceeds u128, the check degrades it

// and one that is not degenerate at all: it fits u128 with room to spare
// but overflows a u64 multiply, so it is the case that forces the promotion
get_amount_out(1_000_000_000_000_000_000, 1_000_000_000_000_000_000, 1_000_000_000_000, 30) == 996_999_005_991
```

Ese último par es con el que hay que quedarse pensando, porque es donde la comparación de arriba paga su calificador. Esta aritmética ya la hiciste en m05-l2, sobre la escala de comisión `× 997`: dos factores `u64` entran en `u128` con menos de un bit de sobra, y un tercer factor gasta la astilla. La misma forma acá con `10_000 - fee_bps` como el tercero — reservas de `u64::MAX` ponen el numerador alrededor de 3.4e42 contra un techo cerca de 3.4e38. La fila de 1e18 es el contrapeso, que pica cerca de 1.0e34 y libra ese techo por cuatro órdenes y medio de magnitud. Un espacio enorme, entonces, y todavía no una garantía, que es por lo que los productos se quedan verificados y la verificación degrada a 0 en vez de entrar en panic a mitad de instrucción.

Ahora el límite que la firma congelada no puede expresar: `10_000 - fee_bps` hace underflow para cualquier `fee_bps` arriba de 10,000, y el tipo de retorno es un `u64` pelado sin ningún lugar donde poner un error. Ponle una guarda igual y devuelve 0, de la forma en que le pones guarda a la reserva vacía — un 0 equivocado-pero-callado le gana a una instrucción que entra en panic, y sobre un crate cuyo perfil de release de verdad tiene overflow-checks apagados — no el de este scaffold, que los fija prendidos — el underflow hace wrap del intermedio `u128` a un número enorme en vez de eso, lo que cotiza un pago que drena el pool. Pero ten claro contigo mismo qué quiere decir ese 0. Para `fee_bps == 10_000` es aritmética: una comisión del 100% se come la entrada entera. Para todo lo demás es un centinela que hace de suplente de un error que la firma no puede devolver, y los AMM de verdad no degradan así — el `getAmountOut` de Uniswap V2 revierte con INSUFFICIENT_LIQUIDITY sobre una reserva vacía. Así que quien llama sigue siendo dueño del límite. En el handler, `fee_bps` es una constante de tiempo de compilación que controlas; en el momento en que se vuelve un parámetro que un usuario puede fijar, necesita un `require!` antes de llegar a esta función, y el handler se niega a liquidar un canje que cotiza 0 de cualquier manera. Escribe ese razonamiento como un comentario arriba de la fn para que el próximo lector sepa que fue una decisión y no un accidente.

Después mídela, porque una cotización que no pasaste por el loop es nada más una reescritura. Mete `get_amount_out(reserve_in, reserve_out, amount_in, 30)` en el handler en lugar de `swap_out`, vuelve a construir, y vuelve a correr `cu_swap_regression`. Reporta el delta de la misma forma en que reportaste los otros dos: un número, un cambio nombrado, una oración.

El criterio de aceptación: la comisión se aplica antes de cotizar, las entradas están guardadas antes de que se compute nada, cada producto intermedio se computa en un `u128` verificado, así que reservas grandes no pueden hacer overflow, hay exactamente una división en el camino caliente, cada salida de referencia coincide — las degeneradas incluidas — y puedes decir el delta de CU medido contra `swap_out`. El starter ya cumple exactamente un ítem de esa lista — hace una sola división — y falla todos los otros; tu solución los pasa todos. Computa el primer caso a mano, saca 9,871, y después haz que el código coincida con tu aritmética.

## Antes de seguir adelante

Cuatro formas en que esta lección sale mal en la práctica. Chequéate contra cada una.

¿Cambiaste exactamente una cosa entre cada par de mediciones? Si volteaste dos flags en una línea de build, tu delta es una suma que no puedes atribuir, y la respuesta honesta es volver y aislarlos.

¿Atribuiste el cero en vez de encogerte de hombros? Un delta de cero tiene dos lecturas, y solo una de ellas es "este cambio es gratis": la tuya era "el cambio nunca llegó al binario", y `cargo tree -e features` nombró el borde de anchor-spl que se lo tragó. Carga también el corolario: el día en que estés sobre un crate donde el flip aterriza, el argumento de invariantes vence completo — un build con redes apagadas es apenas tan seguro como el párrafo que escribiste al lado, y ningún párrafo quiere decir revertir.

¿Citaste el ahorro de const-rent como un rango con una nota de volver a verificar, y no como un solo dígito congelado? El ahorro es alrededor de 85 a 90 CU, abarca dos citas de fuentes, y puede irse a la deriva si la fórmula de rent cambia. SIMD-0194 es la razón por la que el número lleva una fecha de vencimiento que no controlas.

¿Y mediste cada palanca sobre la instrucción que de verdad toca? Guardrails apagado apuntaba al camino caliente del canje — y el grafo lo vetó antes de que llegara; const-rent aterriza en el init de creación de cuenta. Una ganancia de 50 CU sobre un camino al que nadie le pega es ruido. Pesa la ganancia por cuán seguido corre la instrucción.

Corriste el loop de verdad y puedes demostrar cada resultado que produjo: un cero atribuido a un borde de dependencia nombrado, un ahorro del init atribuido a un flag, una cotización reescrita medida por sus propios méritos, y una prueba de regresión sosteniendo la línea del canje por ti. El próximo módulo hace una pregunta más difícil que "¿es rápido?" Pregunta "¿es seguro?" Vas a escribir los exploits clásicos de Anchor contra tu propio programa, las sustituciones de cuenta y las verificaciones de firmante faltantes, y ver cuáles V2 simplemente se niega a compilar y cuáles sobreviven a todo framework y siguen siendo tuyas para defender. El loop medido te enseñó a comprar CU un cambio a la vez. El módulo de seguridad te enseña qué es lo que nunca debes cambiar por ella.

Nos vemos en el módulo de seguridad.
