# 0.3x -> 1.0: los dolores que quien migra todavía encuentra

Acabas de entregar el capstone en m09-l3: el floor-registry haciéndole CPI al counter, al quarter-vault, al prize-escrow y al swap de token-a-ticket, llevado hasta el final por prueba, fuzz, profile, una corrida de localnet en Surfpool, un deploy en devnet, y una pasada local de verify-desde-el-repo. Cada línea de eso salió de un archivo en blanco. Nada de lo que tocaste fue heredado.

Ahora nos damos vuelta hacia las bases de código que no empezaron en blanco.

Acá está el escenario, y no es hipotético. Heredas un programa que compiló e hizo deploy limpio sobre Anchor 0.32 ocho meses atrás. Tu trabajo es chico: subir el toolchain a la línea 1.x y seguir. Así que haces la cosa obvia. Apunta `avm` a la línea actual y recompila.

```bash
# avm ships with the Anchor installer; if you don't have it:
#   cargo install --git https://github.com/otter-sec/anchor avm --force
# (otter-sec/anchor is the repo's current home; the coral-xyz and
#  solana-foundation URLs still redirect there.)
# Toolchain is the 1.x line. 1.1.2 is the version this recon is written against;
# the line moved on 2026-09-04, when 1.2.0 shipped. Pin 1.1.2 here so the recon
# output matches the lesson, and re-check `avm list` before you pin anything else.
avm install 1.1.2
avm use 1.1.2
anchor build
```

No va a compilar. `#[interface]` es un atributo desconocido. `CpiContext::new` rechaza el `AccountInfo` de programa que le vienes pasando desde hace dos años. `anchor login` desapareció por completo. Y acá está la parte que importa: nada de lo que escribiste está mal. El framework se movió debajo de ti, y el compilador te va a decir *qué* se quebró sin decirte nunca *por qué*. Esta lección es el mapa de lo que se movió entre 0.3x y 1.0, y la razón detrás de cada movimiento. Agarra el *por qué* y el port deja de ser un juego de adivinanzas.

## Resumen

Este es el primero de dos deltas de migración. Es un tour de por-qué-no-solo-qué sobre el release de Anchor 1.0.0 (entregado el 2026-04-02) y las roturas de 0.32-a-1.0 que todavía siguen vivas hoy en bases de código reales. Acá no hay build que completar. Ese es un trato deliberado: gastas la hora de manos a la obra de una lección normal en una meseta en cambio, leyendo el delta en frío, para que el port de verdad en m10-l3 se vuelva un checklist en vez de una pelea con el compilador. Esta lección abre la pista de quien migra del módulo, las tres lecciones de m10-l1 a m10-l3, que existen para lectores que cargan una base de código más vieja. Si eres nuevo de nuevo en Anchor y nunca escribiste una línea de 0.32, puedes pasarle el ojo a la pista e ir derecho a la conclusión en m10-l4. El costo de pasarle el ojo es perder el contexto de trayectoria sobre el que se apoya el resto del módulo.

Vamos a recorrer seis cambios. Para cada uno: la rotura exacta, la razón por la que el framework la hizo, y la única edición que la arregla. Al final deberías poder mirar un fragmento de 0.32 y nombrar de memoria el cambio de 1.0 con el que se pega. Ese reconocimiento es el punto entero.

Una palabra sobre cómo entregan responsabilidad estas lecciones. Al principio de este curso te recorrí comando por comando. Acá te entrego la subida del toolchain y un grep, y lees el compilador tú mismo. En m10-l2 recibes el código y una tabla de delta y manejas un port chico tú mismo. Para m10-l3 recibes un repositorio roto cuyas ediciones mecánicas están marcadas y cuyas dos ediciones más difíciles no, porque para entonces la salida propia del compilador es la marca. Las rueditas se van saliendo a lo largo del módulo, y esta lección es donde sale la primera.

## El mapa de lo que se movió, y por qué

La forma honesta de leer una lista de cambios que rompen es preguntar, por cada ítem, *¿qué problema estaba causando la forma vieja que resuelve la forma nueva?* Un framework no rompe código por valor de un millón de descargas por diversión. Cada uno de estos tenía un límite que lo motivaba. Así que empezamos cada cambio desde ese límite, como lo harías si fueras quien decide si entregar la rotura.

### 1. El rename que nadie terminó de hacer

El cambio más visible es también el que te enseña más sobre la migración como práctica. El paquete de cliente se movió de `@coral-xyz/anchor` a `@anchor-lang/core` (PR #4141), y 1.0 es donde el nombre viejo dejó de ser el que te entrega la documentación. Lee las fechas en orden, porque son la pista: el paquete `@anchor-lang/core` se creó en npm el 2025-12-19, su primer publish de verdad aterrizó el 2026-01-06 bajo el número de versión de la línea vieja `0.32.1`, y 1.0.0 en sí no se entregó hasta el 2026-04-02. El nombre se movió en la línea 0.32, meses antes del release bajo el que normalmente se lo archiva.

Así que el rename es noticia vieja. Tuvo casi un año para propagarse. Acá está la pregunta motivadora que quien migra debería hacerse: *si el nombre canónico cambió ocho meses atrás, ¿el nombre viejo está muerto?*

La respuesta ingenua es sí, claro, la documentación dice que uses el nuevo. Esa respuesta va a romper calladamente tu migración. Porque "canónico" y "lo que de verdad vas a leer" divergieron, y divergieron fuerte.

![El paquete viejo @coral-xyz/anchor tiró alrededor de 602k descargas semanales contra unas 15k del nuevo @anchor-lang/core, una brecha cerca de cuarenta a uno.](assets/v01-chart.webp)

Más o menos ocho meses después del rename, el nombre viejo todavía le gana en descargas al nuevo por algo cerca de cuarenta a uno. El número exacto no importa y cambia cada semana. La forma de eso es lo que te llevas: el paquete que te dicen que importes no es el paquete que el ecosistema está importando. La mayor parte del código de ejemplo que copias de un blog, la mayor parte de las respuestas de Stack Overflow, la mayor parte de los repositorios a medio migrar que heredas, siguen recurriendo a `@coral-xyz/anchor`.

Por eso el compañero que dice "el rename es cosmético, solo actualiza el import" te está entregando una trampa. El rename en sí de verdad es solo un movimiento de nombre, la API que hay en la caja no cambió *por el rename*. La trampa es la realidad del ecosistema alrededor. Si asumes un solo import canónico y grepeas un solo scope, te vas a perder la mitad de los call sites, porque el código que estás portando se escribió contra el nombre que todavía gana el conteo de descargas. El arreglo es un hábito, no una edición: grepea los dos scopes antes de asumir nada.

```bash
# Before you touch a line, inventory BOTH names.
# ripgrep (rg) ships with most dev setups; else: brew install ripgrep
rg -l "@coral-xyz/anchor" .
rg -l "@anchor-lang/core" .
```

Hay una pieza de color chica y lúgubre que vuelve esto concreto. La ruta de aprendizaje oficial del propio ecosistema, la que está en solana.com/developers/courses, ahora sirve un redirect 308 hacia un repositorio de GitHub archivado que se congeló el 2025-01-24. El curso canónico de Anchor es contenido más o menos de la era 0.30 sentado en un árbol de solo lectura. Así que cuando quien migra va a buscar guía autoritativa de migración y encuentra una lápida, eso no es un accidente de tu búsqueda. Es el estado del mundo, y es exactamente por eso que esta lección existe en un curso pago y esencialmente en ningún otro lado.

### 2. CpiContext::new dejó de tomar un AccountInfo

Acá está una rotura que detiene el build, no solo el linter. En 0.32 armabas una llamada entre programas entregándole a `CpiContext::new` el programa como un `AccountInfo`:

![En 0.32 CpiContext::new tomaba el token program como un AccountInfo y usaba Transfer; en 1.0 toma un Pubkey vía .key() y usa TransferChecked con decimals.](assets/v02-annotated-code.webp)

Anchor 1.0 cambió `CpiContext::new` para que tome el programa como un `Pubkey` (PR #2762). Pásale un `.to_account_info()` ahí ahora y falla al compilar con un desajuste de tipos seco: esperaba `Pubkey`, encontró `AccountInfo`.

¿Por qué hacer esta rotura? Razona desde lo que una CPI de verdad necesita. Cuando armaste la transferencia de token allá en m05-l1, el runtime identificaba al programa llamado por su dirección y nada más, porque la identidad de un programa en Solana simplemente *es* su clave pública. Cuando le entregabas al constructor un `AccountInfo` entero, estabas pasando un handle gordo donde una sola clave era la única parte estructural, y Anchor se daba vuelta y sacaba la clave de vuelta internamente de todos modos. Mover el argumento a `Pubkey` saca esa indirección redundante y alinea el constructor con cómo ya piensa el runtime sobre el programa llamado. Es un apriete chico, y es la forma sobre la que después construye el modelo de borrow de 2.0. El arreglo son exactamente dos caracteres de intención: cambia `.to_account_info()` por `.key()`.

Un lector afilado retruca acá, y el retruque vale responderlo porque es la objeción que vas a escuchar en code review. Si Anchor iba a extraer la clave de todos modos, ¿qué costaba de verdad pasar el `AccountInfo` entero, más allá de unos bytes en la pila? La respuesta honesta es que en 0.32 costaba casi nada en runtime, y si el costo de runtime fuera la historia entera esta rotura no habría valido el vaivén de código por valor de un millón de descargas. La motivación de verdad es que el handle gordo te dejaba pasar una cuenta que no era el programa para nada, y el constructor la tomaba, difiriendo el desajuste a una falla de runtime en vez de un error de compilación. Angostar el tipo a `Pubkey` mueve una clase entera de errores de "pasé la cuenta equivocada acá" desde un error on-chain confuso hacia un mensaje seco de tu propio compilador, que es exactamente el trato para el que existe un framework tipado.

Viajando junto a esta va la forma idiomática de transferencia de SPL. El `transfer` simple está deprecado a favor de `transfer_checked`, que toma el mint y los decimals para que el programa de token pueda verificar que estás moviendo lo que crees que estás moviendo con la precisión que crees que tiene. Así que la migración mecánica son dos jugadas de una vez: el argumento de programa va a un `Pubkey`, y la llamada de transferencia gana una cuenta de mint y un valor de decimals. Sáltate la segunda mitad y arreglaste el error de tipos solo para entregar una llamada deprecada.

Una precaución para que no confundas dos deltas que se parecen. El modelo de borrow de `CpiHandle`, donde el argumento de cuentas mismo cambia de forma, es un cambio de 2.0, el territorio de la próxima lección. En 1.0 la rotura es específicamente y solamente el argumento de programa moviéndose de `AccountInfo` a `Pubkey`. Si ves consejos sobre `CpiHandle`, estás leyendo sobre otro mundo.

### 3. El literal de space se volvió una expresión

En 0.32, la mitad de los bloques `init` del ecosistema llevaba un cálculo de space hecho a mano que empezaba con un `8` mágico:

![El literal de space contado a mano de 0.32, 8 más los tamaños de campo, se vuelve la expresión derivada DISCRIMINATOR.len() más INIT_SPACE en 1.0.](assets/v03-annotated-code.webp)

El `8` era el discriminator de la cuenta, y todo lo que venía después eras tú, contando bytes de campo a mano y esperando haber acertado el relleno. El límite que lo motiva es obvio una vez que entregaste un bug por eso: un literal contado a mano se corre. Agrega un `u64` a la struct, olvídate de subir el literal, y obtienes una falla de runtime que no tiene nada que ver con el código que acabas de cambiar.

1.0 reemplaza el literal por `DISCRIMINATOR.len() + INIT_SPACE`. `INIT_SPACE` viene de `#[derive(InitSpace)]` sobre tu struct de cuenta y se computa a partir de los campos mismos, así que sigue a la struct automáticamente. `DISCRIMINATOR.len()` reemplaza el `8` mágico por el largo real del discriminator real, que importa porque 1.0 también te deja definir discriminators propios que no tienen ocho bytes. El arreglo es borrar la aritmética y dejar que el framework la derive. Esta es la rotura rara que es puro lado positivo: estás sacando una clase de bug, no cambiando una forma por otra.

### 4. Un enum #[error_code] por programa

0.32 te dejaba desparramar definiciones de error por varios enums `#[error_code]`, uno por módulo si querías. La regla de 1.0 (PR #4300) es un enum por programa — pero escucha bien la imposición, porque esta es la trampa: un segundo enum `#[error_code]` todavía *compila verde* sobre la línea 1.x. Ningún error, ninguna advertencia, verificado contra el toolchain fijado. Si el programa que heredaste dividió sus errores en un `VaultError` y un `EscrowError`, el build que debería haberse quejado se entrega bien, y el riesgo aterriza calladamente en runtime en cambio.

El riesgo es colisión de discriminante, y vale recorrer una instancia concreta para sentirlo. Anchor le asigna a cada error un código numérico por su posición en el enum, desplazado dentro de un espacio compartido de códigos de error que empieza en `6000`. Imagina el programa heredado: `VaultError` declara `Overflow` primero, así que se vuelve `6000`, y `EscrowError` en otro módulo también declara su primera variante, que *también* quiere ser `6000`. Ahora un cliente agarra el error `6000` de una transacción que falló y no tiene forma de saber si el vault se desbordó o el escrow rechazó, porque dos enums independientes contaron los dos desde la misma base y produjeron el mismo código para significados distintos. Colapsar a exactamente un enum por programa vuelve al código de error un índice único y sin ambigüedad hacia una lista única, así que `6000` quiere decir una cosa para siempre. El arreglo es un merge que impones tú mismo, porque el compilador no lo va a imponer por ti: mueve cada variante hacia un solo enum, y si dos subsistemas entregaron una variante con el mismo nombre, renombra una de ellas. Es tedioso más que difícil, y la recompensa es que cada uno de tus códigos de error finalmente quiere decir exactamente una cosa, que es lo que quien los decodifica off-chain necesitaba desde el principio.

### 5. #[interface] y las instrucciones de interfaz desaparecieron

Esta es la rotura en el hook. En 0.32, una instrucción de interfaz de SPL, el caso clásico siendo el `execute` de un transfer hook, se declaraba con la macro de atributo `#[interface]`. En 1.0 esa macro y la maquinaria entera de instrucciones de interfaz se sacaron. No hay ningún `#[program_interface]` al que lo renombres, y no hay ninguna feature flag que lo traiga de vuelta. Desapareció.

¿Qué lo reemplazó, y por qué? La razón es unificación. Cada instrucción común de Anchor ya se despacha haciendo coincidir su discriminator, los bytes iniciales de los datos de instrucción. Las instrucciones de interfaz necesitaban lo *mismo*, despacho por un discriminator específico y definido desde afuera, pero tenían una macro hecha a medida para hacerlo. 1.0 colapsa el caso especial hacia el general. El `execute` de un transfer hook ahora se declara como cualquier otra instrucción, salvo que le dices a Anchor qué discriminator hacer coincidir:

![La macro de interfaz de 0.32 sobre un transfer hook se vuelve un atributo de instrucción de 1.0 que lleva un slice de discriminator de SPL explícito, usando despacho común por discriminator.](assets/v04-annotated-code.webp)

El discriminator viene de la interfaz de SPL misma, expuesto como una constante `SPL_DISCRIMINATOR_SLICE`, así que estás haciendo coincidir los bytes exactos que define la interfaz en vez de confiar en que una macro los sepa por ti. El arreglo: borra el atributo `#[interface]` y declara la instrucción normalmente con `#[instruction(discriminator = ...SPL_DISCRIMINATOR_SLICE)]`.

Una nota de frontera, porque acá es donde los cursos se solapan y quiero mantener las líneas limpias. Esta lección enseña la *migración* de la declaración del hook, el atributo que cambió. No enseña la interfaz de transfer hook en sí — las cuentas que reenvía, el contrato de execute, el programa entero. Ese es territorio del curso de Digital Assets, Tokenización y Token Extensions, y si te estás encontrando con transfer hooks por primera vez acá, ese es el curso que los enseña. Acá solo nos importa qué atributo cambias.

### 6. El toolchain cambió de forma debajo del código

Los primeros cinco cambios son cosas que editas en el código. El sexto no es una edición de código para nada, sino el suelo sobre el que se para el código, y es el que embosca a la gente a la hora del deploy y no a la hora del build.

Empieza por la emboscada. Tu programa de 0.32 compila sobre 1.x después de que arreglas el código, lo apuntas a devnet, haces el deploy, y el *deploy* da error en el IDL on-chain. Nada de tu Rust está mal. El problema es una cuenta obsoleta del mundo viejo.

![Un build de 1.x pasa pero el deploy tropieza con una cuenta de IDL on-chain heredada; cerrarla una vez con el CLI 0.32.1 despeja el camino.](assets/v05-flowchart.webp)

1.0 sacó las instrucciones heredadas de IDL on-chain (PR #3798). Los IDL ahora van on-chain a través del Program Metadata Program en cambio. Pero un programa que heredaste probablemente tuvo deploy con una cuenta de IDL creada a la vieja forma, y el camino de deploy de 1.x no sabe cómo esquivarla. El arreglo es preciso y es una jugada de una sola vez: cambia al CLI 0.32.1, cierra la cuenta de IDL heredada con `anchor idl close`, cambia de vuelta a 1.x, y haz el deploy. Usas el CLI viejo exactamente una vez, para exactamente esto. No es un downgrade y no es permanente. 1.x escribe IDL perfectamente bien, solo a través de otro programa.

Mientras estamos acá abajo, varias otras piezas del toolchain cambiaron de forma, y saber que se movieron te salva de perseguir fantasmas:

- **`anchor login` y `[registry]` desaparecieron.** El flujo viejo donde te logueabas en un registry para publicar un IDL ya no existe. Los IDL van on-chain vía el Program Metadata Program. Si tu memoria muscular recurre a `anchor login`, detente, esa puerta está tapiada.
- **LiteSVM es la plantilla de prueba default** (PR #4316). Los scaffolds nuevos de `anchor init` levantan pruebas de Rust basadas en LiteSVM en vez de la forma vieja de validador-en-un-loop.
- **Surfpool es el validador default** (PR #4106), y necesita por lo menos 1.1.2. `anchor test` y `anchor localnet` manejan Surfpool ahora, no `solana-test-validator`.
- **El CLI se desacopló de un CLI solana externo** (PR #4099). El toolchain de Anchor empaqueta lo que necesita ahora en vez de llamar a un binario solana instalado por separado, que es por lo que el instalador de 1.x ya no te molesta para que hagas coincidir una versión específica de solana primero. Nota la dirección de esto con cuidado, porque cambia cómo lees un número de versión. La versión del CLI de Anchor y la versión del CLI de Agave que tienes instalada ahora son hechos independientes, así que un pin como "Solana CLI 3.1.10" sentado en el bloque de toolchain de un proyecto es una declaración sobre el entorno de integración continua de ese proyecto, nunca una afirmación sobre cuál es el release actual de Solana. Léelo como un pin local, no como un titular.
- **`declare_program!` movió sus helpers generados** de `utils` a `parsers`. `declare_program!` es cómo un programa consume el IDL on-chain de otro programa para generar un módulo de CPI y de cliente, y en 0.32 los parsers de cuenta generados vivían bajo un submódulo `utils`. 1.0 los movió a `parsers`, que se lee como un cambio cosmético de ruta hasta que te das cuenta de que es el tipo de rotura que el compilador agarra al instante y un grep agarra más rápido. Si el código que heredaste consume otro programa vía `declare_program!` y alcanza hacia adentro del módulo `utils` generado, actualiza la ruta a `parsers` y sigue.

![Un lado a lado de seis preocupaciones de toolchain que muestra la herramienta de 0.3x y su reemplazo en 1.0, desde la publicación de IDL hasta las rutas de módulo de declare_program!.](assets/v06-comparison.webp)

### El delta como evidencia, no como trivia

Da un paso atrás de los seis ítems y pregunta a qué suman. Cada rotura rastrea hasta un límite con el que la forma vieja se estaba pegando: un handle gordo de CPI donde alcanzaba una clave, un literal contado a mano que se corre, códigos de error colisionando, una macro a medida duplicando un despacho que ya existía, un flujo de IDL que le quedó chico a un registry. Ninguna es arbitraria. Esa es la lectura que vuelve legible en vez de espantosa una decisión de migración.

![Una tabla de referencia de seis filas que mapea cada rotura de 0.32-a-1.0 a la razón que la motiva y a su arreglo exacto, cubriendo el rename, CpiContext, el cálculo de space, el enum de error-code, la remoción de interface, y el IDL heredado.](assets/v07-table.webp)

Vale atar esto de vuelta a la trayectoria que armamos en m01-l2, porque eso es lo que deja que quien migra y un lector nuevo de nuevo en Anchor compartan una sola historia en vez de dos. Allá atrás el arco entero del framework se enmarcó como un apriete lento: cada versión cambia un poco de la soltura vieja por un compilador que agarra más de tus errores antes de que lleguen a un validador. El delta de 0.32-a-1.0 es ese mismo arco, visto desde adentro del único salto donde el apriete casualmente rompió código. Un lector que nunca escribió una línea de 0.32 igual se beneficia de leerlo así, porque las *razones* son los principios de diseño del framework que está aprendiendo, no trivia de migración que puede olvidar. Quien migra recibe los mismos principios más un plan de port. Una narrativa, dos audiencias.

También demuestra algo que la conclusión de este módulo (m10-l4) va a formalizar en un árbol de decisión: los cambios que rompen cuestan horas de verdad. Acabas de contar las horas. Quien migra se pega con cada una de estas en un programa no trivial, y el dato de que el nombre viejo del paquete todavía le gana en descargas al nuevo por cuarenta a uno demuestra que la audiencia para este trabajo es real y grande. Hay gente corriendo código de 0.32 en producción ahora mismo y va a estar portándolo mucho después de que esta lección esté vieja. El punto de sostener el *por qué* de cada cambio es que, cuando portes en m10-l3, el "esto se rompió" seco del compilador se vuelve tu "cierto, ese es el cambio tres, acá está la edición", sin un desvío por documentación que puede ella misma ser una lápida.

![Una línea de tiempo desde la creación del paquete nuevo en npm el 2025-12-19, pasando por el release de 1.0.0 el 2026-04-02, terminando donde el nombre viejo todavía lidera por cerca de cuarenta a uno.](assets/v08-timeline.webp)

## Lab: reconocimiento de port

Nada se construye. La actividad es diagnóstico, y es trabajo de verdad: vas a hacer que un programa de 0.32 falle sobre la línea 1.x a propósito, después leer cada rotura que produce y mapearla a los seis cambios de arriba antes de arreglar una sola línea. Este es el reconocimiento que harías el primer día de un port de verdad, y hacerlo una vez acá es lo que vuelve m10-l3 un checklist.

Las lecciones anteriores te entregaron cada comando con su salida. Acá recibes las jugadas y lees el compilador tú mismo. Ese es el repliegue de la ayuda en acción: para m10-l3 la única guía que queda es el texto de error del propio compilador.

1. **Ponte un programa de 0.32 adelante.** Cualquier programa de Anchor 0.32 no trivial sirve, y no va a ser uno de los tuyos: cada peldaño de la escalera de Quarters se escribió contra la RC de V2 desde un archivo en blanco. Clona cualquier programa público de Anchor de la era 0.32 que haga transferencias de token. Confirma la versión que tiene como objetivo antes de empezar:

   ```bash
   # Look at Anchor.toml [toolchain] and the anchor-lang pin in Cargo.toml
   rg "anchor_version|anchor-lang" Anchor.toml Cargo.toml
   ```

   Espera un pin de 0.3x en las dos líneas. Si ya lee 1.x, este programa fue portado y es el sujeto equivocado para el ejercicio; encuentra uno que no.

2. **Inventaría los dos scopes de paquete.** Antes de tocar Rust, encuentra cada import del lado del cliente, en los dos nombres:

   ```bash
   rg -l "@coral-xyz/anchor" .
   rg -l "@anchor-lang/core" .
   ```

   Anota qué scope usa el código. Si es el viejo, eso es normal, esa es la realidad del cuarenta a uno. Todavía no estás arreglando esto, estás contando.

3. **Sube el toolchain y forza la rotura.** Apunta avm a la línea 1.x y compila:

   ```bash
   avm install 1.1.2
   avm use 1.1.2
   anchor --version   # confirm you are on the 1.x line
   anchor build
   ```

   Espera que el build falle, fuerte y en varios lugares a la vez. Esa falla es el entregable de este paso, no un problema para resolver todavía.

4. **Cataloga cada error contra los seis cambios.** No arregles nada. Por cada error de compilador, escribe el número del cambio al que mapea. Estás buscando las huellas: un atributo `#[interface]` desconocido (cambio 5), un desajuste de `Pubkey` contra `AccountInfo` en `CpiContext::new` (cambio 2). Dos de los seis no dejan ningún error de compilador, así que tienen que agarrarse por grep, no por el build. Un literal de space hecho a mano con `8 + ...` (cambio 3) no siempre va a dar error fuerte; y un segundo enum `#[error_code]` (cambio 4) nunca da error — dos enums compilan verde, que es exactamente la trampa sobre la que advirtió la sección del cambio 4:

   ```bash
   rg "space\s*=\s*8\s*\+" .
   rg -n "#\[error_code\]" .   # more than one hit inside a single program crate: change 4
   ```

5. **Haz un dry-run de la trampa del deploy en tu cabeza, o en devnet.** No vas a arreglar el deploy hoy, pero ubica el riesgo. Si el programa alguna vez tuvo deploy con un IDL on-chain a la vieja forma, anota que un deploy de v1 va a tropezar con la cuenta de IDL obsoleta hasta que la cierres con el CLI 0.32.1. Escribe el comando exacto de close que *correrías*:

   ```bash
   # the one-time close, run on the 0.32.1 CLI, NOT 1.x:
   #   avm use 0.32.1
   #   anchor idl close <program-id> --provider.cluster devnet
   #   avm use 1.1.2
   ```

**Checkpoint.** Terminaste cuando tienes una lista escrita: cada rotura que produjo el build *más* las dos silenciosas que sacaron a la superficie los greps, cada una etiquetada con su número de cambio y su arreglo de una línea, más una nota sobre si aplica el close del IDL heredado. Esa lista es un plan de port. No escribiste una línea del port, y ya sabes exactamente lo que va a llevar. Ese es el trato entero que esta lección hizo por ti.

## Challenge

Cierra el loop de memoria. Acá hay tres fragmentos sacados de un programa de 0.32. Para cada uno, sin mirar de vuelta hacia arriba en la página, nombra el cambio que rompe de 1.0 con el que se pega, la razón por la que existe el cambio, y la edición exacta que lo arregla. Tres triples de cambio-a-razón-a-arreglo.

**Fragmento A:**
```
let cpi_ctx = CpiContext::new(
    ctx.accounts.token_program.to_account_info(),
    Transfer { from, to, authority },
);
transfer(cpi_ctx, amount)?;
```

**Fragmento B:**
```
#[interface(spl_transfer_hook_interface::execute)]
pub fn execute(ctx: Context<Execute>, amount: u64) -> Result<()> { Ok(()) }
```

**Fragmento C:**
```
#[account(init, payer = authority, space = 8 + 32 + 8)]
pub vault: Account<'info, Vault>,
```

Para ir más lejos: tu programa heredado hizo deploy bien sobre 0.32 pero su primer deploy en 1.x da error en el IDL on-chain, y no hay ningún bloque `#[interface]` ni ningún enum `#[error_code]` extra en ninguna parte del código. ¿Cuál es el paso de migración que falta, y por qué "corre `anchor login` y vuelve a registrar" es el instinto equivocado? Escribe la respuesta antes de chequearla contra el cambio 6.

## Antes de seguir adelante

No necesitas haber arreglado nada para haber hecho bien esta lección. Necesitas la lista de reconocimiento del Lab y tres triples limpios del Challenge. Si algún triple salió difuso, la pista más rápida es que nombraste el *arreglo* pero no la *razón*, esa es la brecha exacta que hace que un port se sienta como adivinar, así que vuelve a ese cambio y vuelve a leer por qué se movió el framework, no solo hacia qué se movió. Si tu lista de errores de build no incluyó un desajuste de tipos de `CpiContext` o una queja de `#[interface]`, tu programa de prueba probablemente no ejercitaba CPI ni hooks; corre el reconocimiento una vez más contra uno que sí, porque esas son las dos roturas que se comen más tiempo en un port de verdad. No necesitas cargar ese programa hacia adelante: m10-l3 pone un vault de v1 en el banco y lo porta fila por fila.

Ese fue el primer delta: los dolores de llegar *a* 1.0. Pero la línea 1.x y `anchor-next` son dos mundos paralelos, y el salto de 1.x a 2.0 es una reescritura desde el suelo, no un rename. El modelo de borrow de `CpiHandle` que te dije que dejaras de lado dos veces en esta lección vive allá. La próxima lección cruzamos esa línea y recorremos cada lugar donde el código cambia cuando vas de 1.x hacia V2. Feliz port.
