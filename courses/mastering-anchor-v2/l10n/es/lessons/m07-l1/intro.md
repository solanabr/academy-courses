# Qué mata V2 en tiempo de compilación

La lección pasada corriste el loop de medir-cambiar-remedir sobre el swap. El flip de guardrails midió un delta de exactamente cero — el borde de dependencia de anchor-spl mantuvo las redes prendidas, y le atribuiste el cero en vez de entregar la flag — mientras `const-rent` y la cotización reescrita produjeron deltas que podías quedarte, cada uno demostrado con un número antes y un número después. Confiaste en el compilador para mantener el programa correcto mientras lo hacías más rápido.

Ahora convierte esa confianza en arma. `quarter-vault` es el workspace en el que has estado construyendo desde el módulo 3 — mantiene el nombre que le dio `anchor init` por su primer programa, y ahora sostiene R2, R3 y R4 — y el swap de token-a-ticket que tiene adentro es el cuarto peldaño de la escalera de Quarters. Abre una terminal ahí, corta una rama de exploit desde tu R4 limpio, y escribe el primer ataque clásico de Anchor contra tu propio swap.

```bash
cd quarter-vault
git checkout -b exploit/compile-time-kills
```

Vas a escribir cuatro ataques deliberados, un commit cada uno, contra el swap de token-a-ticket que ya construiste: el pool leído a través de un lente de config, dos slots mutables de reserva marcados con `dup` simple, una reserva tipada leída mientras una CPI está en vuelo, y un bump de pool recomputado a mano. Después le pegas a `anchor build`. Tres de ellos nunca producen un binario. Uno de ellos compila bien y no hace nada. Esa brecha, entre "compila" y "explota", es la lección entera.

Esta es la lección que era genuinamente imposible de enseñar antes de agosto de 2026. Las cuatro clases de vulnerabilidad de abajo necesitaban una verificación en runtime, una prueba, o un auditor con un checklist. Sobre los defaults de V2, tres de ellas mueren donde las escribes, en rojo, en tiempo de compilación. Te toca ver morir a cada una.

Una reserva honesta antes de que escribas una línea de maldad: esto es un alpha no auditado. Todo acá es una ganancia de tiempo de compilación de verdad y nada de esto es una demostración. Sostén las dos cosas.

## Resumen

Al final de esta lección vas a haber re-derivado cuatro clases de la taxonomía archivada de seguridad de programas de la Solana Foundation (el curso de once directorios en el que entrenó una generación entera de auditores) y re-corrido cuatro de ellas contra los defaults de V2. Vas a demostrar que tres ahora son errores de compilación en vez de riesgos de runtime, y que la cuarta compila pero no le queda ninguna costura para tirar.

Las cuatro clases:

- **Type cosplay**: cargar los bytes de una cuenta como un tipo distinto.
- **Duplicate-mutable**: pasar la misma cuenta escribible en dos slots para gastarla dos veces.
- **Aliasing de CPI / obsoleto-después-de-CPI**: leer un campo tipado cuyos bytes una llamada entre programas está a punto de cambiar debajo de ti.
- **Recálculo de bump**: recomputar un bump de PDA para que se cuele uno equivocado.

Tu entregable es concreto: tres errores de compilador nombrados, capturados con el texto de su mensaje, más una corrida verde de `anchor test` una vez que restaures R4. El registro acá es cautela, no celebración. El compilador es un aliado fuerte y una excusa mala.

![Una tabla de cuatro filas que empareja cada clase de vulnerabilidad de Anchor con su riesgo de runtime en v1, su estado sobre los defaults de V2, y el juicio del que el desarrollador sigue siendo dueño.](assets/v01-comparison.webp)

## Las cuatro clases, y por qué tres se vuelven errores de tipo

Acá está el repliegue para el resto de la lección, dicho sin adornos para que sepas qué viene. Yo recorro la primera clase, type cosplay, de punta a punta: el riesgo en v1, los arreglos ingenuos, y la razón exacta por la que el default de V2 la rechaza. Las dos siguientes, duplicate-mutable y aliasing de CPI, las terminas desde stubs en el Lab. La última clase la atacas por tu cuenta, con una variante que nadie te entregó, y predices el resultado antes de compilar. El apoyo cae a propósito. Así es como descubres qué entendiste de verdad.

Una palabra sobre de dónde vienen estas cuatro, para que esto no se lea como una lista que inventé. La Solana Foundation corrió un curso de seguridad de programas cuyo repositorio sostiene once directorios de vulnerabilidad, una clase por directorio, y una generación de auditores aprendió la taxonomía de ahí. Type cosplay, duplicate-mutable, CPI arbitraria, verificaciones de dueño y de firmante que faltan, sustitución de cuentas, y el resto, cada uno tuvo un programa vulnerable y uno parchado. Lo que estamos haciendo esta lección es tomar cuatro de esas once y re-correrlas contra los defaults de V2 para ver cuáles responde ahora el framework por ti. Cuatro de once. Mantén esa razón a la vista: es el alcance honesto de lo que compra un upgrade de compilador.

Empieza con la pregunta que está debajo de las cuatro: ¿qué es una cuenta, para un programa? En Anchor v1, una cuenta llega como bytes crudos más un discriminator de ocho bytes, y `Account<'info, T>` deserializa esos bytes hacia tu struct después de verificar que el discriminator coincide con `T`. Ese paso de deserializar es la costura. Cada una de estas cuatro clases es una forma de hacer que el programa actúe sobre bytes que no son lo que el tipo dice que son. La pregunta interesante no es "¿hay una verificación?", es "¿cuándo corre la verificación, y puedo saltármela por accidente?". Una verificación de runtime corre cuando corre la transacción, que es tarde, y corre solo si el camino de código llega a ella. Una verificación de tiempo de compilación corre antes de que exista un binario. Esa diferencia de momento es el asunto entero de esta lección.

### Type cosplay, recorrido de punta a punta

El statu quo, en v1: tu instrucción espera un `FeeConfig`, el atacante te entrega un `Pool`, y si las dos structs por casualidad se alinean en memoria, tu programa lee los bytes del pool a través del lente de config y confía en campos que quieren decir otra cosa completamente. La verificación del discriminator era lo que detenía la versión cruda. La versión sutil se colaba cuando dos tipos de cuenta compartían un prefijo o cuando un programa usaba `AccountInfo` y deserializaba a mano sin verificar.

La pregunta motivadora: si el tipo está fijo en la definición de la struct, ¿por qué el runtime es libre de entregarme los bytes equivocados?

Descarta los arreglos ingenuos primero, porque son lo que entregó v1. Arreglo ingenuo uno: agrega un discriminator y verifícalo en cada load. Funciona, pero es una verificación de runtime, que quiere decir una verificación que puedes olvidar, apagar o rodear con `AccountInfo`. Arreglo ingenuo dos: compara una string de nombre de tipo guardada en el load. Más lento, todavía runtime, y ahora estás pagando por guardar un nombre. Los dos arreglos comparten la misma falla: detectan el desajuste después de que el programa ya está sosteniendo una referencia tipada a la memoria equivocada.

V2 afila el requisito hacia algo que el compilador puede imponer. Sobre los defaults de V2, `Account<T>` (nota el lifetime que se soltó) es una **vista zero-copy tipada como Pod** de los datos de la cuenta. `T` tiene que implementar `Pod`, que quiere decir que no tiene relleno y tiene un layout completamente determinístico, y los bytes se castean directo a `T` en vez de parsearse campo por campo. Esta es la misma decisión de diseño por la que argumentó la issue #4390 bajo la bandera "zero-copy account deserialization by default", que nombró al viejo `Account<T>` de parseo-en-load como "the slow path" y "the #1 performance complaint". El punto con el que vale quedarse: Pod-por-defecto es una jugada de seguridad tanto como una jugada de velocidad. Un layout determinístico y sin relleno es exactamente lo que vuelve a "estos bytes son un `FeeConfig`" una afirmación que el sistema de tipos puede sostener en vez de una afirmación que re-verificas en runtime.

![Un diagrama que contrasta la verificación del discriminator en runtime de v1, que se puede saltear, con el cast tipado como Pod en tiempo de compilación de V2, donde el tipo de la cuenta está fijo en la struct y lo rastrea el compilador.](assets/v02-diagram.webp)

Así que cuando escribes el cosplay, el desajuste de tipos no tiene dónde esconderse. Una pieza de montaje honesta primero, porque la forma de tu propio programa importa acá: R4 entrega exactamente **un** tipo de cuenta, el `Pool` que escribiste en m05-l2, y el cosplay necesita dos. Así que la rama de exploit agrega un segundo — un `FeeConfig` que tu swap no tiene y no va a hacer crecer. Es un accesorio, y nombrarlo como tal es parte de la lección: lo que demuestra el error de compilación de abajo es un hecho sobre el sistema de tipos, no una afirmación sobre un campo que tu programa de verdad sostenga.

```rust
// programs/token-ticket-swap/src/lib.rs  (R4, clean) - your swap's ONLY account type
#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub arcade_mint: Address,  // 32
    pub ticket_mint: Address,  // 32
    pub bump: u8,              //  1
    pub _pad: [u8; 7],         //  7
}

// programs/token-ticket-swap/src/exploits.rs - the prop, exploit branch only.
// An admin-config shape is the classic cosplay target: it carries an authority
// worth stealing.
#[account]
#[derive(InitSpace)]
pub struct FeeConfig {
    pub authority: Address,  // 32
    pub fee_bps: u16,        //  2
    pub bump: u8,            //  1
    pub _pad: [u8; 5],       //  5
}
```

Ahora el cosplay. Sostienes el pool e intentas leerlo como un `FeeConfig` para levantar la authority:

```rust
// ATTACK 1: type cosplay - read the pool's bytes through the FeeConfig lens
pub fn cosplay(pool: &Account<Pool>) -> Address {
    let stolen: &FeeConfig = pool.as_ref(); // will not compile
    stolen.authority
}
```

`Account<Pool>` es un `Slab`, y un `Slab` implementa `AsRef` para exactamente dos objetivos — el `AccountView` crudo y el `Address` — nunca para algún *otro* tipo Pod. Le pediste un `&FeeConfig`, un impl que no existe. El compilador se detiene en seco:

```text
error[E0277]: the trait bound `Slab<Pool>: AsRef<FeeConfig>` is not satisfied
  --> programs/token-ticket-swap/src/exploits.rs
   |
   |     let stolen: &FeeConfig = pool.as_ref();
   |                                   ^^^^^^ the trait `AsRef<FeeConfig>` is not implemented
   |
   = help: the following other types implement trait `AsRef<T>`:
             `Slab<T, H>` implements `AsRef<AccountView>`
             `Slab<T, H>` implements `AsRef<Address>`
```

Eso es type cosplay convertido en `E0277`: la superficie de traits simplemente se niega a ofrecer un lente que no sea el tipo propio de la cuenta. Los bytes del pool nunca llegan a leerse a través de la struct equivocada, porque la struct equivocada es un tipo Pod distinto y los dos no se interconvierten. Nota qué hizo el trabajo: no una verificación de runtime nueva, sino el sistema de tipos común de Rust, al que se le dio un layout determinístico del que agarrarse.

Ahora sé honesto sobre lo que hizo y no hizo ese fragmento, porque por sí solo demuestra menos de lo que parece. Nadie entregó nunca esa línea. `let x: &FeeConfig = y.as_ref()` sobre un `&Pool` falla al compilar en v1, en 0.29, en cualquier Rust jamás escrito; es un error de tipos, no un exploit. La línea está ahí porque es el intento de *lavado*, la cosa que un atacante prueba primero cuando el wrapper tipado está en el camino, y muestra al wrapper tipado sosteniéndose. El ataque de verdad de v1 nunca escribió esa línea. Rodeó el wrapper tipado por completo:

```rust
// ATTACK 1, the shape it actually shipped in: hand-read raw bytes through
// the wrong struct, so no wrapper and no discriminator is ever consulted.
pub fn cosplay_v1(any_account: &AccountInfo) -> Result<Pubkey> {
    let data = any_account.try_borrow_data()?;
    let cfg = FeeConfig::try_from_slice(&data[8..])?;  // is this REALLY a FeeConfig?
    Ok(cfg.authority)                                  // whatever bytes sat there, read as one
}
```

Entrégale un `Pool` y felizmente devuelve los primeros 32 bytes — `arcade_mint` — como una `authority`, porque `try_from_slice` decodifica lo que sea que se le dé. Nota qué plausible se ve el resultado: un mint es una dirección bien formada, así que nada más adelante huele mal hasta que alguien firma contra ella. Esa es la clase. Dos cosas tienen que sostenerse para que V2 la responda, y la responden en dos relojes distintos. En el *load*, una cuenta `Pool` pasada a un slot declarado `Account<FeeConfig>` es rechazada por la verificación del discriminator, en runtime, exactamente como la rechazaba el `Account<T>` de v1: la etiqueta dice `account:Pool` y el wrapper quería `account:FeeConfig`. Esa mitad no es nueva. Lo que *sí* es nuevo es que la salida de emergencia al nivel de los bytes se cerró: con `Account<T>` como una vista Pod no hay `try_from_slice` sobre un slice suelto al que recurrir, y el cast de bytemuck al que recurrirías en cambio, `from_bytes::<FeeConfig>(&data[8..])`, es un cast que tienes que escribir deliberadamente, sobre bytes de los que no demostraste que son un `FeeConfig`, en código que un revisor puede grepear en una sola pasada. El discriminator siempre fue la guarda de runtime. La contribución de V2 es que el camino común ya no te ofrece una forma de rodearlo, que es por lo que el `E0277` de arriba es la falla interesante en vez de una obvia.

### Duplicate-mutable

El mismo motor, otra costura. El doble gasto clásico: una instrucción toma dos cuentas escribibles y el atacante pasa la *misma* cuenta para las dos. El programa lee un saldo a través de un nombre, lo lee otra vez a través del otro, acredita la primera, debita la segunda, y escribe las dos de vuelta. Las dos copias en memoria divergen, gana la escritura que aterrice última, y el crédito sobrevive mientras el débito se evapora. Eso es acuñar de la nada. En v1 el dispatcher corría una verificación de duplicados para detener exactamente esto, pero era fácil salirse de la verificación por accidente, y muchos programas se salieron, normalmente recurriendo a un tipo de cuenta crudo para raspar un constraint.

Intenta imaginarlo sobre tu swap y pasa algo útil: no puedes. `SwapArcadeForTickets` lleva `token::mint = mint_arcade` en `reserve_arcade` y `token::mint = mint_ticket` en `reserve_ticket`, y ninguna cuenta de token sostiene dos mints, así que los dos slots escribibles de reserva son no-aliasables antes de que la verificación de duplicados tenga voto. Un constraint que escribiste en m05-l2 por una razón de precio cerró esta puerta por una razón de seguridad. Ese es el estado honesto de R4, y es por eso que el ataque de abajo es una struct de cuentas *nueva* que agregas en la rama de exploit en vez de una edición al swap: tienes que arrancarle los pins para que la costura exista.

Sobre los defaults de V2, el conjunto de cuentas escribibles que una instrucción toca es un const asociado de tiempo de compilación, el bitset **`MUT_MASK`**. Salirse de la protección de duplicate-mutable es una cosa de verdad que a veces necesitas, y tiene un nombre: `unsafe(dup)`. Escribir `dup` simple sin `unsafe` es un error de compilación duro cuyo mensaje te dice el arreglo. No puedes ni construir la escritura unsafe por accidente, porque la escritura que parece segura no compila.

Lee esa trampa con cuidado, porque es la que la gente recuerda mal: la verificación de duplicados en runtime sigue corriendo en el dispatcher. V2 no la borró. Lo que V2 agregó es una barrera de compilador delante de la escritura unsafe, así que llegas a la verificación de runtime solo por el camino que marcaste explícitamente como unsafe. "El build está verde" ahora quiere decir "no apagué esto por un typo".

![Una tarjeta de código anotada que muestra dos slots mutables de cuenta marcados con dup pelado, el error de compilación de V2 rechazándolo, y la escritura unsafe(dup) exigida que el error nombra como el arreglo.](assets/v03-annotated-code.webp)

### Aliasing de CPI y la muerte de `.reload()`

Esta es la clase que retira un hábito de v1 para el que tienes memoria muscular. En v1, si leías el saldo de una cuenta de token, después hacías una CPI que cambiaba ese saldo, después leías el campo otra vez, obtenías el valor *obsoleto* a menos que te acordaras de llamar `.reload()`. El bug era invisible: el código se veía correcto, el campo tenía un número plausible adentro, y el número simplemente era viejo. Existían auditorías enteras para encontrar llamadas a `.reload()` que faltaban.

La pregunta motivadora: ¿por qué al programa se le permite sostener una referencia tipada a través de una llamada que muta esos mismos bytes?

Descarta las respuestas de v1 por niveles, porque el ecosistema probó todas. Nivel uno: acuérdate de llamar `.reload()` después de cada CPI. Esto es disciplina, y la disciplina es la cosa que falla a las 2 de la mañana bajo una fecha límite. Nivel dos: documéntalo, pon "always reload after CPI" en la guía de contribución. La documentación agarra al lector que la lee. Nivel tres: escribe un linter que grepee CPIs sin un reload a continuación. Mejor, pero un linter modela un patrón, y en el momento en que la CPI y la lectura están en funciones distintas el patrón se rompe y el linter se queda callado. Los tres niveles comparten una falla: intentan agarrar un error que el sistema de tipos ya estaba en posición de volver imposible.

La respuesta de V2 es un borrow, no un recordatorio. Un **`CpiHandle`** es un handle con borrow rastreado hacia las cuentas que una CPI va a tocar. Mientras el handle está vivo, sostiene un borrow de Rust sobre esas cuentas, y el acceso tipado a esos mismos datos no compila hasta que el handle se dropea. Físicamente no puedes leer el campo obsoleto, porque la lectura no compila mientras la CPI está pendiente. La clase entera de obsoleto-después-de-CPI se colapsa dentro del borrow checker, que es la única parte de Rust que nunca se olvida.

![Una línea de tiempo vertical de la ventana de borrow de un CpiHandle, que marca cada lectura tipada de la reserva de ticket adentro de ella como un error de compilación, contra la lectura obsoleta de v1.](assets/v04-diagram.webp)

### Recálculo de bump, el que compila

La cuarta clase es la interesante, porque no produce un error. En v1, un programa que recomputaba un bump de PDA en cada llamada, en vez de guardar el canónico, podía ser guiado a firmar con un bump no canónico, y la familia de recomputar-el-bump-equivocado vivía en esa costura. Sobre los defaults de V2 la costura es más apretada, pero sé preciso sobre el mecanismo, porque es fácil exagerar: la macro precomputa un bump como un const de tiempo de compilación *solo cuando cada seed es un literal de string de bytes* — un `b"..."` escrito por completo en el derive y nada más. La lista de seeds de tu pool lee `seeds = [POOL_SEED]`, y `POOL_SEED` es un `const` nombrado, no un literal, así que el pool se pierde esa optimización por un pelo y la generación de código cae de vuelta a derivar durante la validación. (Si quieres verificar en vez de creerme: el `lang-v2/derive/src/pda.rs` del tag fijado condiciona la cosa entera a `seeds_as_byte_literals`, que coincide con un literal de string de bytes y devuelve `None` para una expresión de ruta — sus propias pruebas unitarias lo dicen por completo.) Lo que el framework nunca hace, sobre ninguna forma de seed, es aceptar un bump que le entregues: la validación re-deriva el resultado canónico y compara.

Así que cuando recomputas un bump a mano en tu exploit, compila. `Address::find_program_address` es código común. Pero el framework valida y firma contra su propia derivación canónica, así que tu valor recomputado es o idéntico, caso en el que no cambiaste nada, o distinto, caso en el que la validación de PDA lo rechaza en runtime. El ataque compila y no va a ningún lado. Mantén ese resultado cerca, porque es el puente hacia la próxima lección: compilar no es explotar, y hay un conjunto entero de clases donde el código compila *y* drena un escrow.

![Un embudo que muestra cuatro ataques entrando a anchor build, tres saliendo como errores de compilación rechazados, y solo el ataque de bump emergiendo como un binario.](assets/v05-flowchart.webp)

Ese conjunto es donde vive la honestidad, así que déjame nombrar la trampa ahora en vez de al final.

Convertir cuatro clases en errores de compilación angosta la superficie de ataque. No retira la auditoría. V2 es una candidata a release no auditada, y la postura de "los defaults no son sustituto de la revisión" es una que este curso afirma por autoridad propia — el proyecto no lo va a decir por ti. Ve a mirar y encuentras el README del tag fijado dando el tono opuesto ("v2 is secure by default for users"), sin ninguna página de reservas detrás. Esa es exactamente la superficie de marketing que un equipo se cita a sí mismo mientras se saltea la revisión, así que el escepticismo tiene que ser tuyo. La mala lectura cómoda, "secure by default" oído como "secure", es exactamente cómo un equipo se convence de salirse de la revisión que agarra todo en la próxima lección. Una garantía de tiempo de compilación es confiable solo en la medida del compilador que la hace, y este compilador es un alpha. Trata las cuatro muertes como afirmaciones de diseño que verificas contra la RC fijada, no como demostraciones. V2 no es la bala de plata para la seguridad de programas; es un compilador muy bueno con un changelog muy honesto.

Hay un riesgo de segundo orden acá que es peor que cualquier bug aislado. Un equipo que internaliza "el compilador agarra nuestros bugs de seguridad" revisa menos, y revisa menos precisamente en la región donde el compilador está callado, que es la región de donde el dinero de verdad se va. Así que la disciplina está invertida respecto de cómo se siente: las clases que el compilador mata son las que menos atención puedes gastarles en la revisión, y las clases que no puede tocar son a donde debería ir el presupuesto entero de auditoría. Las ganancias de tiempo de compilación son una reasignación de dónde miras, no una razón para mirar menos. Vale mantener la división en algún lado donde puedas verla.

![Una tabla de dos bandas que separa las clases que agarran los defaults de V2 de las clases de firmante, de sustitución y de lógica que compilan, corren y siguen siendo trabajo del desarrollador.](assets/v06-table.webp)

Ese changelog vale una mirada, porque modela la postura. El PR #4914, mergeado el 2026-08-13, revisó los benchmarks del titular *a la baja*: el ahorro de bytecode de 95% a 94%, y la ganancia de compute de 9.9x a 8.8x, con la reserva de que "This version is alpha and exact values can move as codegen, pinocchio, and tooling change." Cita el 8.8x como contexto de cuánto más rápido corre el camino Pod, nunca como un número de seguridad. La misma honestidad que revisa un benchmark a la baja es la honestidad que prohíbe tratar cualquier default de V2 como auditado.

Un nombre para archivar y no desarrollar: la clase de sustitución de cuentas que viste en la banda de abajo tiene una historia de guerra canónica, el drenaje del `.mint` que faltaba en Cashio, y ese es territorio del curso de DeFi y RWA Engineering. Apuntamos hacia allá en vez de volver a contarlo, y retomamos la clase de sustitución de cuentas en sí en la próxima lección.

## Lab: cuatro ataques, una rama

Estás en `exploit/compile-time-kills`. Primero, fija el toolchain, porque nada de esto es real sobre la línea V1.

```bash
# Install the Anchor V2 release candidate. Freshness note (2026-08-22):
# 2.0.0-rc.1 is the pinned RC for this course, tagged on the `anchor-next` branch. It is an
# UNAUDITED alpha. `avm` CANNOT install the V2 RC: it only tracks published GitHub releases,
# and there is no release object for the v2 tag. Install the CLI straight from the tag:
cargo install --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1 anchor-cli --locked --force
# macOS: prefix with CARGO_PROFILE_RELEASE_LTO=off if the release build fails to link.
# The tag is a fixed point (commit e4878b6d) where the branch head is not, which is what keeps
# this in step with m08-l2's verify Dockerfile. Do NOT verify V2 content on the 1.1.x line.
anchor --version   # expect the 2.0.0-rc.1 line, NOT anchor-cli 1.1.x
```

El repliegue de la ayuda empieza acá. El ataque 1 está hecho por ti, para que puedas ver la forma de un error capturado. Los ataques 2 y 3 llegan como stubs que terminas. El ataque 4 ya lo entiendes de la derivación, así que solo lo corres y lees el (no-)resultado.

**Paso 1: aterriza el ataque de type cosplay y captura el error.** Crea `programs/token-ticket-swap/src/exploits.rs`, pega el Ataque 1 de la derivación de arriba, cabléalo en `lib.rs` con `mod exploits;`, y haz el build:

```bash
anchor build 2>&1 | tee /tmp/attack1.log
grep -A6 'E0277' /tmp/attack1.log
```

Deberías ver el bloque de `trait bound` nombrando `Slab<Pool>: AsRef<FeeConfig>` como no satisfecho, con la nota de help listando `AccountView` y `Address` como los únicos objetivos de `AsRef` que ofrece un `Slab`. Commitea el estado que falla para que la rama registre el intento:

```bash
git add -A && git commit -m "attack 1: type cosplay (does not compile)"
```

Checkpoint: `git log --oneline` muestra un commit, y `/tmp/attack1.log` contiene `E0277`. Si el build *tuvo éxito*, accidentalmente dejaste las dos structs siendo el mismo tipo, así que re-verifica que `Pool` y `FeeConfig` son distintos.

**Paso 2: termina el stub de duplicate-mutable.** Esta es una segunda struct de cuentas en `exploits.rs`, no una edición a `SwapArcadeForTickets` — como dijo la derivación, los pins de `token::mint` del swap de verdad vuelven no-aliasables sus slots de reserva, así que el ataque tiene que soltarlos para tener una costura. Completa el segundo slot para que los dos sean mutables y los dos lleven el opt-out de `dup` simple:

<!-- verify: expect-fail the V2 default rejects bare `dup`; that compile error IS this lesson's point -->
```rust
// STUB - finish this so both slots are `mut` and marked plain `dup`
#[derive(Accounts)]
pub struct DrainPool {
    pub trader: Signer,
    #[account(seeds = [POOL_SEED], bump = pool.bump)]
    pub pool: Account<Pool>,
    #[account(mut, dup)]
    pub reserve_a: InterfaceAccount<TokenAccount>,
    // TODO: add reserve_b as a second mutable InterfaceAccount<TokenAccount>,
    //       also marked plain `dup`
}
```

Haz el build, y confirma que el compilador nombra el arreglo. Esta es la línea exacta que busca el paso de verify de la lección:

```bash
anchor build 2>&1 | grep -c 'unsafe(dup)'
# expect: at least 1
```

Checkpoint: el conteo es al menos 1. El error te dijo que escribieras `unsafe(dup)`, y lo vas a dejar como `dup` simple, porque el punto es el rechazo, no el arreglo. Commitealo como un intento que falla.

**Paso 3: termina el stub de aliasing de CPI.** Ya corriste este experimento una vez, en el paso 3 de m05-l2, donde mover las lecturas de reserva hacia adentro de la ventana del handle era un checkpoint. La misma jugada, enmarcada como un ataque: completa el stub para que una lectura tipada de la reserva de ticket quede *entre* la creación del handle y su `invoke`:

```rust
// STUB - read reserve_ticket.amount() while the CpiHandles are still live
pub fn drain(ctx: &mut Context<SwapArcadeForTickets>) -> Result<()> {
    let push = TransferChecked {
        from: ctx.accounts.reserve_ticket.cpi_handle_mut(),
        mint: ctx.accounts.mint_ticket.cpi_handle(),
        to: ctx.accounts.trader_ticket.cpi_handle_mut(),
        authority: ctx.accounts.pool.cpi_handle(),
    };
    // TODO: read ctx.accounts.reserve_ticket.amount() HERE, while `push` still holds the handles
    token_interface::transfer_checked(
        CpiContext::new(ctx.accounts.token_program.address(), push),
        1,
        ctx.accounts.mint_ticket.decimals(),
    )?;
    Ok(())
}
```

Haz el build. El borrow checker rechaza la lectura con un mensaje de clase `E0502`: `reserve_ticket` está prestado mutablemente por el `CpiHandle` adentro de `push`, así que no puedes tomar una segunda referencia para leer su saldo. Captúralo, commitea el intento que falla.

Checkpoint: el build falla por un error de borrow que nombra el handle y `reserve_ticket`. Si *compiló*, tu lectura aterrizó después de que los handles salieron de alcance o después del `transfer_checked`, que es el orden seguro, así que mueve la lectura hacia arriba.

**Paso 4: corre el ataque de bump y lee el no-resultado.** Este compila. Agrega el recálculo a mano y haz el build:

```rust
// ATTACK 4: hand-recompute the bump instead of trusting the one the pool stored
pub fn wrong_bump(ctx: &mut Context<SwapArcadeForTickets>) -> Result<()> {
    let (_pda, bump) = Address::find_program_address(&[POOL_SEED], &crate::ID);
    msg!("recomputed bump = {}, stored bump = {}", bump, ctx.accounts.pool.bump);
    Ok(())
}
```

```bash
anchor build   # this one succeeds
```

Checkpoint: el build está verde, y los dos bumps del log son iguales. No había ninguna costura para explotar, solo el bump canónico para re-derivar a tu propia costa en CU. Ese build verde es el punto del ejercicio entero: compiló, y no drenó nada.

**Paso 5: restaura R4 a limpio y demuéstralo.** Saca los exploits de vuelta afuera y confirma que el swap todavía pasa:

```bash
git checkout main -- programs/token-ticket-swap/src   # restore clean R4 source
rm -f programs/token-ticket-swap/src/exploits.rs
anchor test
```

Checkpoint: `anchor test` está verde. Tu artefacto de evaluación está completo ahora: tres errores de compilación capturados en la rama de exploit (type cosplay, duplicate-mutable, aliasing de CPI) más una corrida verde de pruebas sobre R4 restaurado. El ataque de bump es el cuarto commit registrado que compiló y no hizo nada.

![Una línea de tiempo de commits de cinco nodos: tres ataques fallando al compilar, uno compilando como un no-op de runtime, y un commit final restaurando la suite verde.](assets/v07-timeline.webp)

## Challenge

El trabajo de Completion es el Lab que acabas de terminar: tres ataques expresados desde stubs, tres errores de compilador capturados, R4 restaurado a verde. Ahora el peldaño Solo, donde nadie te entrega el ataque.

Elige una clase y escribe una variante *nueva* de ella contra el swap. Algunos puntos de partida, pero inventa el tuyo si se te ocurre uno:

- Un par distinto de cuentas aliasadas para la clase de CPI: sostén una lectura tipada de `reserve_arcade`, no de `reserve_ticket`, mientras un handle sobre `reserve_arcade` está vivo.
- Un cosplay en la otra dirección: lee un `FeeConfig` como un `Pool` e intenta levantar su `ticket_mint`.
- Un duplicate-mutable a través de tres slots en vez de dos.

Antes de compilar, anota tu predicción: ¿el default de V2 mata esto en tiempo de compilación, o lo deja pasar? Después haz el build y verifícate. La predicción es la parte calificada, no la compilación. Si puedes cantar el resultado antes de pegarle al build, internalizaste el mecanismo en vez de memorizar los cuatro ejemplos. Si tu predicción estuvo equivocada, la pregunta interesante no es "cuál es el arreglo" sino "cuál de los cuatro mecanismos entendí mal", y la sección de derivación es a donde vas a averiguarlo.

## Dónde te deja esto

Acabas de hacer algo que el ecosistema no podía hacer un mes atrás: escribiste cuatro exploits de libro de texto de Anchor contra tu propio programa y viste al compilador rechazar tres de ellos por su nombre. Eso es un angostamiento de verdad de la superficie de ataque, y vale estar genuinamente contento con eso.

Ahora sostén la otra mitad. Tres ataques se negaron a compilar. Pero escribiste cuatro, y uno compiló bien. Las clases de las que el compilador no puede salvarte son las próximas, verificaciones de firmante y de dueño que faltan, sustitución de cuentas, los bugs de lógica y de aritmética, y esas son las que de verdad drenan escrows. Un build verde es un piso, no una línea de llegada. Trae la rama de exploit y el mismo ojo sospechoso a la próxima lección, donde atacamos exactamente lo que todavía muerde, y donde "compiló" deja de ser consuelo alguno.
