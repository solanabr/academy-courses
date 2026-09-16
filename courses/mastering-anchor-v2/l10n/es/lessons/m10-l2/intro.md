# 1.x a 2.0: los deltas de la reescritura

La lección pasada mapeaste el salto de 0.3x a 1.0: los renames, el único `#[error_code]` por programa, el paso de close del IDL on-chain, `CpiContext::new` aprendiendo a tomar un `Pubkey`. Se puede sobrevivir a cada uno de esos con un find-and-replace y una tarde. Forzaste la rotura sobre un programa 0.32 y la catalogaste contra los seis cambios; el port en sí es subir un número en `Cargo.toml`, una persecución a través de los errores de compilación, y el mismo programa saliendo del otro lado. Así se siente una subida de versión.

El salto de 1.x a 2.0 no es eso: es una reescritura `no_std` desde cero sobre pinocchio, y la sintaxis que ya conoces deja de pasar la verificación de tipos. `Pubkey` ya no es un tipo. Los lifetimes `<'info>` que escribiste en cada struct de cuentas desde el módulo 2 desaparecen. `has_one` compila pero se subraya a sí mismo. El `dup` pelado se niega a compilar y te dice que escribas `unsafe(dup)` en su lugar. `.reload()` no está más, y no porque alguien lo haya renombrado. Así que antes de razonar sobre nada de esto, mira la superficie que estás a punto de cruzar. En un programa v1 tuyo, corre esto:

```bash
rg -n 'Pubkey|\.key\(\)|has_one|zero_copy|<'"'"'info>|\.reload\(\)|realloc::payer|LazyAccount|AccountLoader|Migration<' src/
```

Cada línea que se imprime es una línea que va a cambiar. Algunas son renames de una palabra. Unas pocas son errores de compilación con opinión. Una de ellas, `AccountLoader`, se imprime como una coincidencia ordinaria más que vas a estar tentado de saltarte, porque el nombre sobrevive hasta V2 y el build nunca se queja de él. Esa es la más peligrosa del conjunto. Ese grep es tu lista de trabajo para esta lección.

## Resumen

Este es el segundo delta de migración, y el más difícil. En m10-l1 el modelo mental era "mismo programa, otra ortografía." Acá el modelo honesto es "misma intención, otro programa," porque una base de código 1.x no se actualiza a 2.0, se reescribe línea por línea contra una candidata a release que vive en una rama aparte y que se sigue moviendo.

La recompensa por el dolor es real, y es el hilo conductor de todo este curso: V2 mata clases enteras de bugs en tiempo de compilación. La trampa de datos obsoletos que `.reload()` parchaba, el alias de duplicados mutables, la matemática de space hecha a mano que cuenta de menos en silencio. Cada una de esas se vuelve algo que no puedes escribir, no algo que tienes que acordarte de verificar. Vamos a ganarnos esa afirmación derivando cada cambio de sintaxis de vuelta hasta la decisión de diseño que lo forzó, la mayoría de las cuales ya conociste antes en el curso. Después vas a portar un programa v1 chico tú mismo, con el compilador de compañero, y cerrar sobre un challenge de código que arregla el único bug de cálculo de space que sobrevive a un port descuidado.

Acá el llevarte de la mano baja un escalón, a propósito. Los módulos tempranos narraban cada tecla. A esta altura, el lab te entrega el código v1 y la tabla de delta y espera que manejes tú, leyendo las advertencias del compilador como instrucciones en vez de esperar las mías.

## Los deltas, y las decisiones detrás de ellos

Empieza por la pregunta que de verdad importa, porque es la que mantiene honesto al resto de la lección: ¿por qué 1.x a 2.0 rompe cosas que 0.3x a 1.0 no rompía?

La respuesta ingenua es "se acumularon más cambios que rompen." Tentadora, y equivocada. Si fuera solo volumen, el arreglo sería el mismo que la vez pasada, solo que más largo: sube la versión, muele los errores, entrega. Ese enfoque falla en el primer archivo, y falla por una razón específica. De 0.3x a 1.0 era el mismo framework con nombres nuevos. V2 es un framework distinto al que le pasa que se queda con la mayoría de los nombres. Es una reescritura `no_std` construida sobre pinocchio, la capa de runtime zero-copy y liviana en dependencias. Anchor no editó su código viejo para llegar acá. Reconstruyó sobre cimientos nuevos.

Ese solo hecho es el generador. Casi todo delta que viene adelante es consecuencia de una de tres decisiones de diseño cocinadas en esa reconstrucción, y si cargas las tres decisiones en la cabeza puedes predecir los deltas en vez de memorizarlos.

![Un árbol que muestra tres decisiones de diseño raíz (reescritura no_std, default zero-copy, CPI con borrow rastreado) cada una ramificándose hacia los cambios de sintaxis específicos que causan, más un grupo transversal para los deltas narrados por el compilador.](assets/v01-diagram.webp)

Antes de las decisiones, una pieza de vocabulario y un mapa, porque el piso se mueve mientras estás parado encima. La línea v1 es `anchor-lang` 1.1.2 — o mejor dicho lo era: la v1.2.0 se entregó el 2026-09-04, así que la línea estable es 1.2.0 ahora y sigue tomando commits. V2 vive en la rama `anchor-next` y se entrega como `2.0.0-rc.1`, publicada en crates.io el 2026-08-12 bajo el tag de git `v2.0.0-rc.1`. Esas son dos líneas paralelas, no un antes y un después. Al momento de escribir esto, 2026-08-22, la documentación de referencia de V2 todavía carga lenguaje de instalación anterior a la publicación en crates.io, advirtiéndote que consumas los crates desde git. No leas la redacción de alpha que va al lado como el mismo tipo de retraso: el proyecto etiqueta este único release tanto de `rc` como de `alpha` a propósito, y ese par es actual, no obsoleto. Las RC se mueven rápido. Vuelve a verificar la versión de crates.io y el tag de `anchor-next` antes de fijar nada, y trata cada número de versión de esta lección como una instantánea con la fecha encima.

![Una línea de tiempo de dos vías con la línea v1 estable y la línea candidata a release V2 de anchor-next corriendo en paralelo a lo largo de 2026, con rc.1 llegando a crates.io el 2026-08-12.](assets/v02-timeline.webp)

### Decisión 1: la reescritura no_std renombra las primitivas

`no_std` quiere decir que el framework no puede apoyarse en la biblioteca estándar de Rust, así que saca sus tipos fundacionales de crates construidos para ese mundo. Las direcciones ahora vienen de `solana-address` a través de pinocchio. Por eso `Pubkey` se vuelve `Address` y `.key()` se vuelve `.address()`. No es estético. El tipo viejo vivía en un grafo de dependencias dentro del cual V2 ya no se sienta.

Los lifetimes se van por una razón emparentada. En v1 cada struct de cuentas cargaba `<'info>` porque el framework enhebraba a mano un borrow del slice de cuentas de la transacción por tus tipos, y pagabas esa plomería en cada firma que escribiste en tu vida. El modelo de cuentas de V2 rastrea esos borrows de otra forma, así que la anotación de lifetime deja de ser algo que escribas. Los handlers toman `&mut Context<T>`, los wrappers pierden `<'info>`, y una columna entera de paréntesis angulares desaparece de tu código. ¿Comparado con qué? Comparado con un struct v1 donde `pub struct Initialize<'info>` y `Account<'info, Config>` repetían el mismo lifetime una docena de veces para decir una cosa que el compilador ahora infiere.

Si ir una capa más abajo de "el framework se encarga" es la picazón que no dejas de rascarte, ahí es exactamente donde vive el curso de Low-Level Solana: enteramente debajo del framework, sobre la maquinaria sobre la que V2 ahora está sentado.

![Una tabla de comparación que empareja cada ortografía de Anchor v1 con su reemplazo en V2 y la razón de una línea, desde Pubkey-a-Address hasta la remoción de reload().](assets/v03-comparison.webp)

### Decisión 2: zero-copy es el default, así que la ceremonia a su alrededor desaparece

En m02 conociste la ceremonia de v1 como historia que nunca tuviste que correr: `#[account(zero_copy)]`, `AccountLoader`, `load()` y `load_mut()`, todo para evitar deserializar una cuenta grande dentro del stack. V2 vuelve zero-copy el camino ordinario, que es por lo que solo llegaste a leer sobre la forma vieja. `Account<T>` es zero-copy por defecto, lo que requiere `T: Pod` con un layout sin relleno. **Pod** es plain old data, del módulo 2: una struct de tamaño fijo y limpia de alineación donde todo patrón de bits es un valor válido, así el framework puede tender una vista tipada directamente sobre los bytes de la cuenta en vez de decodificarlos. Las consecuencias se propagan hacia afuera, y acá es donde un port descuidado se rompe calladamente.

Primero, la fácil: el atributo `zero_copy` no está más. No hay nada a lo que optar por entrar porque ya estás adentro. Bórralo.

Ahora la complejidad que escondí, devuelta en voz alta. "Todo es zero-copy" solo es cierto para datos que de verdad son Pod, y el criterio para Pod es más alto de lo que quien migra espera. Dos reglas muerden en el primer port. Primera, cada campo tiene que ser Pod él mismo, y un `bool` pelado no lo es: solo dos de sus 256 patrones de bits son legales, así que V2 entrega `PodBool` para eso. Segunda, la struct no puede tener *nada de relleno*. `#[account]` emite `#[repr(C)]` más una aserción de tiempo de compilación de que `size_of::<T>()` es igual a la suma de los tamaños de los campos, y cuando no lo es recibes esto, textual:

```text
account struct has padding bytes; reorder fields from largest to smallest
alignment to eliminate padding (e.g. u64 before u32 before u8)
```

Así que una struct v1 copiada al otro lado normalmente no llega legal para Pod. Tienes dos salidas honestas, y el port elige una por cuenta. Vuelve a disponer la struct con campos de alineación 1 (`PodU64`, `PodBool`, arrays de bytes) para que la suma calce con el tamaño, o manda la cuenta por `#[account(borsh)]` y el wrapper `BorshAccount<T>`, que es también donde vive cualquier cosa genuinamente de largo variable: un `Vec`, un `String`. La seguridad acá no es una convención. Si un layout volviera no sólida la lectura zero-copy, el programa falla al compilar en vez de leer bytes ambiguos en silencio. El diseño no sólido es irrepresentable, que es la tesis entera de V2 en una oración.

Después la trampa. `AccountLoader` todavía existe en V2. Tu grep lo marca como una línea más entre muchas, tu build no se va a quejar para nada, y eso es precisamente el peligro. No quiere decir lo que quería decir en v1. El rol zero-copy de v1 que `AccountLoader` solía llenar se movió a `Account<T>` (el default nuevo). El nombre `AccountLoader` se repropuso como un cursor secuencial de cuentas, una cosa completamente distinta, y la documentación advierte explícitamente que ahora "quiere decir otra cosa". Este es el delta con más probabilidad de compilar y después portarse mal en vez de fallar fuerte. Tratarlo como "desapareció, bórralo" está mal. Tratarlo como "igual que v1, quédatelo" es peor. Es un falso amigo: la misma cara, otro trabajo.

![Una tabla que muestra el rol zero-copy del AccountLoader de v1 moviéndose a Account-de-T, el nombre AccountLoader repropuesto como un cursor secuencial, y LazyAccount quedándose sin equivalente en V2.](assets/v04-comparison.webp)

Última consecuencia de la Decisión 2, y la que vas a arreglar a mano en el challenge: la matemática de space. En 0.32 escribías a mano `space = 8 + 32 + 8`, donde el `8` de adelante era el discriminador de cuenta que agregabas tú mismo. La forma derivada llegó con 1.0, como el cambio tres de m10-l1, y V2 la mantiene sin cambios: `space = T::DISCRIMINATOR.len() + T::INIT_SPACE`. La razón por la que todavía muerde a quien migra en 2.0 es que el conteo a mano es aritmética de Rust legal, así que un programa que se saltó la edición de 1.0 compila en 1.1.2 con el `8` mágico intacto y llega acá cargándolo todavía. El discriminador sigue siendo de 8 bytes (default de sha256, sin cambios y compatible con v1), así que `DISCRIMINATOR.len()` es 8. La trampa es que `INIT_SPACE` es la suma de los tamaños de los campos Pod solamente. Nunca incluye el discriminador. Un port a medio terminar que borra el `8` mágico pero se olvida de que `INIT_SPACE` lo excluye va a contar de menos cada cuenta por exactamente 8 bytes, dimensionar cada cuenta demasiado chica, y desbordar el buffer en la primera escritura.

Desglósalo, porque el número es el argumento entero. Toma el `Config` del lab: un `Address` de 32 bytes, un `u64` de 8, un `bool` de 1. `INIT_SPACE` es `32 + 8 + 1 = 41`. El largo completo on-chain es `DISCRIMINATOR.len() + INIT_SPACE = 8 + 41 = 49`. El port descuidado computa `41` y asigna `41`, así que a la cuenta le falta exactamente un discriminador, y el primerísimo byte de tu campo `authority` aterriza donde el runtime esperaba que la cuenta terminara. El bug no va a crashear en ningún lado que puedas leer: es un off-by-8 que dimensiona bien en tu cabeza y mal en la cadena. Guarda ese layout. Es el challenge.

![Una tira de layout de bytes para una cuenta Config de 49 bytes: el discriminador de 8 bytes más 41 bytes de INIT_SPACE, al lado de una asignación corta cuyas escrituras se desbordan por ocho bytes.](assets/v05-diagram.webp)

### Decisión 3: la CPI se rastrea por borrow, así que .reload() no puede existir

Esta es la derivación insignia de toda la migración, y es un llamado directo de vuelta a m04-l2, donde ya viste el acceso tipado durante un handle vivo volverse un error de compilación. Aplica esa misma idea a la migración y `.reload()` se explica solo.

Rebobina hasta por qué existía `.reload()` en v1. Sostenías una copia deserializada de una cuenta. Hacías una CPI que mutaba esa cuenta on-chain. Tu copia en memoria quedaba obsoleta, mostrando el balance de antes de la transferencia. Si tomabas una decisión sobre esa copia obsoleta tenías un bug de verdad, así que v1 te daba `.reload()` para volver a leer la cuenta después de la CPI y refrescar tu copia. Era un parche para una trampa que el sistema de tipos permitía.

El instinto de quien migra es: "V2 sacó `.reload()`, bueno, lo llamo a mano después de cada CPI como antes." Eso es a la vez imposible e innecesario, y la razón es un solo mecanismo. En V2, las cuentas de CPI son `CpiHandle`s con borrow rastreado. Un handle ata la CPI a un borrow de Rust del wrapper tipado del que vino, lo que le deja a Anchor usar el camino rápido de CPI sin verificar de pinocchio mientras el borrow checker prohíbe el aliasing tipado en tiempo de compilación. Mientras un handle está vivo, el acceso tipado a esa cuenta es un error de compilación. Así que la situación de datos obsoletos no se puede escribir. No hay ningún momento en el que sostengas una copia tipada obsoleta a través de una mutación, porque el borrow checker no va a dejar que la copia y el handle vivo coexistan. Nada que recargar. El método se sacó en vez de deprecarse porque el bug que parchaba ya no es expresable.

Fíjate qué clase de bug mata esto, porque es la peor clase. Un `.reload()` olvidado en v1 no falla el caso promedio. La transferencia igual pasa, la CPI igual tiene éxito, las pruebas que no dependen del balance post-CPI igual pasan. Falla solo cuando tu handler lee el valor mutado y bifurca según él: un withdrawal que verifica un balance que cree que sigue siendo 100 cuando la CPI acaba de moverlo a 0. Esa es la firma de los bugs más caros que hay en la cadena. Correcto en el camino común, equivocado exactamente cuando hay dinero en juego, invisible hasta que llega el peor caso. Deprecar `.reload()` habría dejado esa ventana de peor caso abierta para cualquiera que se olvidara de llamarlo. Sacar por completo la posibilidad de sostener la copia obsoleta cierra la ventana para todos, incluido quien migra y nunca leyó esta lección. Esa asimetría, caso-promedio-bien contra peor-caso-catastrófico, es la razón exacta por la que V2 eligió "irrepresentable" antes que "documentado."

![Dos paneles de código: en v1 una lectura post-CPI queda obsoleta y reload() la parcha; en V2 el acceso tipado durante un CpiHandle vivo es un error de compilación.](assets/v06-annotated-code.webp)

Un delta chico más de esta decisión, porque es donde quienes migran nuevos tropiezan con la sintaxis después de entender el concepto: `CpiContext::new` ahora toma el programa como `&Address`. En m10-l1 aprendiste la forma de 1.0, donde `CpiContext::new` tomaba un `Pubkey`. V2 toma un `Address` prestado. La misma idea, un rename de tipo más siguiendo a la Decisión 1 río abajo hasta la API de CPI.

### Los deltas transversales: deja que el compilador narre

Dos cambios no pertenecen a una sola decisión. Pertenecen a una filosofía: hacer del camino seguro el camino ordinario, y cuando te desvías, obligarte a decirlo en el punto exacto. El compilador es el maestro acá, y fue diseñado para serlo.

Toma `has_one`. Porta `#[account(mut, has_one = authority)]` a V2 y compila, pero emite una advertencia de deprecación que subraya específicamente el keyword `has_one`. Ese subrayado no es incidental. El parser del framework guarda el span del keyword `has_one` a propósito para que el codegen pueda apuntar de vuelta hacia él. La advertencia te está diciendo la edición exacta, en sus propias palabras: "on the sibling field, use `#[account(address = owner.field)]` instead." El lado derecho es cualquier expresión, la mayoría de las veces el campo de la cuenta padre. Es una deprecación con un mapa adjunto. No recurras a `#[allow(deprecated)]` para callarla. `has_one` está en camino a la remoción, y una RC posterior puede llevárselo. La advertencia te está haciendo un favor.

![Un panel de código que muestra una advertencia de build de V2 que subraya el keyword has_one vía un span de parser guardado deliberadamente, con la ortografía corregida de address-igual-a-campo-del-padre debajo, todavía sobre un Signer.](assets/v07-annotated-code.webp)

Ahora la más filosa, la bifurcación línea por línea. Tu código v1 saca una cuenta mutable duplicada de la verificación de duplicados mutables con `#[account(mut, dup)]`. En V2 esa línea no advierte. Falla al compilar. El `dup` pelado es un error de compilación, y el texto del error nombra el arreglo: escribe `unsafe(dup)`. Esto importa más de lo que parece. V2 todavía detecta duplicados mutables, y la verificación todavía corre durante la validación de cuentas contra el bitvec recorrido. Lo que cambió es que la salida de emergencia ahora tiene que escribirse `unsafe`, en el call site, cada vez que la uses. (El bitvec recorrido es la pasada de tiempo de ejecución del dispatcher de m01-l4: marca cada dirección que llega dos veces, y después hace AND de eso contra el `MUT_MASK` de tiempo de compilación del struct. El atributo es un evento de tiempo de compilación; la colisión contra la que protege es de tiempo de ejecución.) La palabra está haciendo trabajo: te hace reconocer, justo donde te desvías, que asumiste la obligación de escribir el handler de forma que nunca forme referencias mutables en conflicto. La documentación dice que la salida se llama `unsafe(dup)` "a propósito." Un `dup` que compilaba en silencio era un riesgo que podías olvidar. Un `unsafe(dup)` que tuviste que escribir es un riesgo que elegiste.

![Un diagrama de flujo para portar una cuenta mutable duplicada: el dup pelado falla al compilar, y dos preguntas sobre el alias te rutean a sacar el opt-out o a escribir unsafe(dup).](assets/v08-flowchart.webp)

### Renames mecánicos y las cosas que tienes que dejar atrás

Unos pocos deltas son pura ortografía, nada de filosofía. `realloc::payer` se vuelve `realloc_payer`, el namespace de doble dos-puntos colapsando en un solo token plano. Grepea, reemplaza, sigue. Estas son las ediciones con sabor a 0.3x-a-1.0 que todavía viven dentro de la migración más difícil, y vale nombrarlas precisamente porque te adormecen: si los primeros diez cambios fueron así de fáciles vas a asumir que el once también, y el once es `AccountLoader`.

Después las remociones. `LazyAccount` y `Migration<From, To>` son solo de v1. No hay ortografía de V2 a la que portarlos. `LazyAccount` era el wrapper de solo lectura, alojado en el heap y de carga bajo demanda de v1; en un mundo zero-copy-por-defecto su nicho lo absorbe mayormente `Account<T>`, así que el wrapper no sobrevive. `Migration<From, To>` simplemente no existe en la línea V2. Si tu programa v1 se apoya en cualquiera de los dos, eso no es un rename, es un rediseño de esa pieza.

Una nota de seguridad para la audiencia de v1, ya que algunos van a mantener un programa en la línea v1 por un rato más. `LazyAccount` solía saltearse su re-verificación de propiedad en el reload, así que después de una CPI una cuenta que sostenías de forma perezosa podía tener su dueño cambiado por debajo sin que el camino perezoso lo volviera a verificar. El PR #4784, "re-run ownership checks when reloading `LazyAccount`," arregló exactamente eso, y se mergeó el 2026-07-16, que es *después* de que 1.1.2 se entregara el 2026-06-26. Lee las fechas en ese orden, porque son el punto: si estás fijado a 1.1.2 no tienes el arreglo. No es un asunto de V2, ya que `LazyAccount` no cruza, pero es una razón para auditar tu uso de v1, y tu pin de v1, antes de decidir qué portar y qué reescribir.

### ¿Deberías portar siquiera, ahora mismo? ¿Comparado con qué?

Vale detenerse en la pregunta que un ingeniero cuidadoso hace antes de tocar un programa que funciona: ¿deberías migrar a V2 hoy, o esperar? El caso a favor de esperar es real, y lo voy a plantear en su forma más fuerte antes de responderlo. V2 es una candidata a release. Explícitamente no está auditada, la documentación de referencia todavía carga salvedades de alpha, y, como mostró el #4937, los pins de crates en sí mismos pueden romper un programa correcto. Un programa que sostiene fondos de verdad en la línea v1, estable en 1.1.2, tiene todas las razones para quedarse ahí hasta que 2.0 entregue un release estable y auditado. Eso es hacer calzar el riesgo del tooling con el valor que protege, no timidez. Si tu programa está en producción, el default honesto es: no portes fondos vivos a una RC.

¿Entonces comparado con qué? Comparado con la alternativa de aprender los deltas después de que 2.0 sea estable, con fecha límite encima, sobre una base de código que medio olvidaste. El port es una reescritura, y una reescritura que haces con calma sobre una copia scratch, mientras el compilador te enseña cada delta, es una tarea completamente distinta de la misma reescritura hecha a las apuradas porque una dependencia finalmente dejó de soportar v1. Aprender los deltas ahora es barato. Portar fondos de producción ahora no lo es. Esas son dos decisiones distintas, y el error es tratarlas como una. Esta lección es la primera: construye el mapa, porta algo descartable, hazte dueño del razonamiento. La segunda, cuándo mover un programa de verdad, es una decisión que tomas después con un release estable y una auditoría en la mano.

![Una tabla de decisión que separa la opción barata, aprender los deltas y portar algo descartable ahora, de la cara de mover fondos de verdad a una RC no auditada.](assets/v09-comparison.webp)

### El trade-off, y la disciplina que fuerza la RC

Acá está el balance honesto. Del lado de las ganancias, V2 saca clases de bugs al nivel de los tipos: la ventana de datos obsoletos de `.reload()`, el alias de duplicados mutables, el layout zero-copy no sólido, el discriminador contado a mano. Esas dejan de ser cosas que verificas y se vuelven cosas que no puedes expresar. Del lado de los costos, una base de código 1.x no se actualiza, se porta, línea por línea, contra una candidata a release en una rama aparte. Cambias una subida de versión mecánica por una reescritura de verdad, y asumes el vaivén de la era RC: el toolchain, los pins de crates, hasta la versión del serializador pueden moverse por debajo entre una escritura y la siguiente.

Esa última cláusula no es hipotética. La primera historia de guerra de la era RC es el issue #4937, abierto el 2026-08-16 y cerrado cuatro días después el 2026-08-20. `anchor-lang` fijaba `wincode` (el serializador de V2) en `0.5` mientras dependía de `solana-address` sin cota superior, y `solana-address` subió su propio requisito de `wincode` a `0.6`. El desajuste de trait bounds entre los dos rompió `#[account(borsh)]` con un error pelado de `SchemaRead is not satisfied`. Nadie escribió código malo. El grafo de dependencias en sí mordió. Esa es la forma del riesgo sobre una RC: código correcto, pins incompatibles. Así que la disciplina no es opcional. Fija versiones exactas, no rangos de caret. Vuelve a verificar el tag de `anchor-next` y las versiones de los crates antes de cada sesión de trabajo. Trata un build verde de hoy como evidencia sobre hoy solamente. La recompensa por la reescritura es un programa que falla en tiempo de compilación en vez de en mainnet. El alquiler que pagas por eso, hasta que 2.0 se estabilice, es vigilancia de versiones.

Vigilancia, hecha concreta, porque ese desajuste exacto está vivo otra vez mientras lees esto. El #4937 se cerró cuando los pins se reconciliaron, pero el `2.0.0-rc.1` publicado todavía fija `wincode 0.5` mientras deja `solana-address` sin cota, y `solana-address 2.7.0` se movió desde entonces a `wincode 0.6` — así que una resolución nueva rearma exactamente el grafo que el issue describía, y el lab de abajo entra derecho en él en el `#[account(borsh)] Config`. Lleva dos caras, una sola causa: `error[E0277]` sobre el tipo de cuenta, `SchemaWrite`/`SchemaRead` "is not satisfied" con una nota sobre múltiples versiones de `wincode` en el grafo de dependencias, y `error[E0433]: could not find wincode`, porque la expansión de `#[program]` nombra `::wincode::` de forma absoluta y tu crate no depende de él directamente. La solución alterna es el par de pins que cada `Cargo.toml` de programa en este curso carga desde m01-l2. Tu port es un crate independiente, así que toma la forma exacta de abajo; el workspace del arcade enuncia el mismo techo de `< 2.7` como un rango, porque cinco miembros tienen que ponerse de acuerdo en una sola versión resuelta — m02-l1 hace ese argumento.

```toml
[dependencies]
anchor-lang = "2.0.0-rc.1"
wincode = { version = "0.5", features = ["derive"] }
solana-address = "=2.6.0"      # rc.1 pins wincode 0.5; solana-address 2.7.0 moved to 0.6
```

Los pins van en el crate del programa sin importar qué canal monte `anchor-lang` en sí, rama de git o crates.io. Ponlos en el `Cargo.toml` del port antes del paso 1 del lab, o encuéntrate con los dos errores en el paso 6 y agrégalos ahí; de cualquier forma, vuelve a verificarlos cada sesión, porque este es exactamente el tipo de solución alterna que una rc posterior te borra por debajo.

## Lab: porta un programa v1 a V2, guiado por el compilador

Vas a portar un programa v1 diminuto pero completo: una cuenta de config con una authority, inicializada una vez, después actualizada por esa authority. Ejercita el rename del modelo de cuentas, la eliminación del lifetime, la deprecación de `has_one`, y el cálculo de space, que es la mayor parte de la superficie de delta en cincuenta líneas. Trabaja en un crate scratch, no en un proyecto de verdad.

Este lab hace que la ayuda se repliegue. Te entrego el código v1 y la tabla de delta de arriba. Tú manejas el port y dejas que el compilador te diga qué queda.

1. **Instala y fija el toolchain de V2.** Una trampa antes de que escribas nada: la RC no pasa por `avm`. No se cortó ningún GitHub Release para el tag de v2, así que el binario preconstruido que `avm install` descarga no está ahí y el fetch da 404. Los crates de rc.1 sí aterrizaron en crates.io el 2026-08-12, pero la instalación documentada sigue siendo una instalación de git desde la rama `anchor-next`, fijada acá al tag:

```bash
# no GitHub Release cut for the v2 tag -> no binary to download; use the git install
CARGO_PROFILE_RELEASE_LTO=off \
cargo install --git https://github.com/otter-sec/anchor \
  --tag v2.0.0-rc.1 anchor-cli --locked --force

anchor --version           # confirm 2.0.0-rc.1 before you touch code
```

Fija esa versión exacta en tu `Anchor.toml` y `Cargo.toml`. Las RC se mueven; vuelve a verificar el tag en `anchor-next` primero (este lab se escribió contra `2.0.0-rc.1`, 2026-08-22).

2. **Lee el código v1 que estás portando.** Esto compila en la línea v1 (1.1.2), con literal de space contado a mano y todo, porque es un programa de la era 0.32 que solo llegó a recibir las ediciones que el compilador forzó. Cada línea que el grep de antes marcaría es una línea que vas a tocar:

<!-- verify: expect-fail the V1 'before' program in the migration module; it is not meant to build on V2 -->
```rust
use anchor_lang::prelude::*;

declare_id!("Cfg1111111111111111111111111111111111111111");

#[program]
pub mod config_v1 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, seed: u64) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.seed = seed;
        config.active = true;
        Ok(())
    }

    pub fn set_active(ctx: Context<SetActive>, active: bool) -> Result<()> {
        ctx.accounts.config.active = active;
        Ok(())
    }
}

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub seed: u64,
    pub active: bool,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 8 + 1)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetActive<'info> {
    #[account(mut, has_one = authority)]
    pub config: Account<'info, Config>,
    pub authority: Signer<'info>,
}
```

3. **Renombra las primitivas (Decisión 1).** `Pubkey` se vuelve `Address`. `authority.key()` se vuelve `authority.address()`. Un pero que el compilador te va a entregar: `.address()` devuelve un `&Address`, no un `Address`, así que la asignación necesita un deref. El scaffold de V2 lo escribe exactamente así:

```rust
config.authority = *ctx.accounts.authority.address();
```

   Después elimina todo `<'info>` de las definiciones de struct y de `Account<'info, Config>`.

4. **Arregla el modelo de cuentas y el space (Decisión 2).** Acá es donde muere el port ingenuo. `Config` sostiene un `Address` (32), un `u64` (8) y un `bool` (1). El `bool` no es Pod para nada, y aunque lo cambiaras por un `u8` el `u64` fuerza alineación de 8 bytes, así que `repr(C)` rellena la struct hasta 48 bytes mientras los campos solo dan cuenta de 41: exactamente la aserción de relleno de antes. Toma la segunda salida y manda esta cuenta por borsh, que es también para lo que está documentado `#[derive(InitSpace)]`. Después reescribe el space a la forma idiomática de V2, y nota que no es `8 + 32 + 8 + 1` copiado hacia adelante:

```rust
#[account(borsh)]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Address,
    pub seed: u64,
    pub active: bool,
}

#[account(
    init,
    payer = authority,
    space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
)]
pub config: BorshAccount<Config>,
```

5. **Arregla el constraint deprecado (el delta transversal).** `has_one = authority` compila pero se subraya a sí mismo. Reemplázalo con la verificación explícita de address a la que apunta la advertencia. Nota dónde aterriza el constraint: se muda de `config` a la cuenta `authority`, afirmando que la address de la authority que se pasa es igual al campo guardado en `config`:

```rust
#[account(mut)]
pub config: BorshAccount<Config>,
#[account(address = config.authority)]
pub authority: Signer,
```

6. **Compila, y lee el compilador como tu checklist.** Corre `anchor build`. Cada error o advertencia es un delta que queda. Arregla el de arriba, vuelve a compilar, repite. Deberías llegar a un build limpio sin `<'info>`, sin `Pubkey`, sin advertencia de `has_one`, y con una línea de space que lea `DISCRIMINATOR.len() + INIT_SPACE`.

**Checkpoint.** `anchor build` tiene éxito y `rg 'Pubkey|<.info>|has_one' src/` no imprime nada. Si el build todavía se queja de un lifetime, te salteaste un `<'info>` en algún struct. Si se queja de que un campo no es `Pod`, o de que la struct "has padding bytes," dejaste una cuenta en el camino zero-copy default que no puede sentarse ahí legalmente: o la vuelves a disponer con campos de alineación 1 o la mandas a `#[account(borsh)]` más `BorshAccount<T>`, como hizo el paso 4 con `Config`. Un programa V2 viene con un camino de pruebas de LiteSVM (`anchor_v2_testing::svm()`); una prueba de humo que inicializa la config y da vuelta `active` es la demostración de que el port de verdad corre, no solo compila.

## Challenge: porta el cálculo de space sin perder el discriminador

Este es el único bug que sobrevive a un port de apariencia cuidadosa, aislado para que puedas matarlo limpio. Una migración barrió un archivo v1 buscando el `8` suelto de la forma idiomática `space = 8 + ...` — instinto correcto, porque V2 no tiene número mágico — y se llevó cada ocho aditivo que encontró. El `* 8` que dimensiona un campo `u64` quedó intacto, correctamente: ese es un ancho de campo, no un número mágico. Pero el barrido no pudo distinguirlos del `checked_add(8)` dentro de `with_discriminator`, el último eslabón de la cadena de dimensionado, que era el paso que volvía a poner el discriminador, y `INIT_SPACE` no lo vuelve a poner por ti. El eslabón sobrevive como una función con nombre que ahora pasa su entrada derecho a través, y todo lo que el helper dimensiona sale 8 bytes corto.

Tu trabajo es restaurar ese eslabón para que `account_len` devuelva el largo completo de datos on-chain: el discriminador de 8 bytes (default de sha256, sin cambios en V2) más los tamaños de campo sumados, donde un `Address` son 32 bytes, un `u64` son 8, y un `bool` es 1. Nombra el nivel, porque esa suma solo vale para uno de ellos: estos son los tamaños de `#[account(borsh)]`, campos escritos uno pegado al otro sin relleno de alineación y sin prefijos de largo, que es el nivel al que el paso 4 mandó `Config`. Bajo el default de Pod los mismos tres campos nunca llegan a un largo siquiera — el `u64` fuerza alineación de 8 bytes, la struct necesita relleno de cola, y el build se detiene en `error[E0080]: account struct has padding bytes`. Nota el nombre también: la función no se llama `init_space`, porque `INIT_SPACE` es exactamente la mitad que excluye el discriminador, y nombrarla así es cómo se escribió el bug en primer lugar.

La firma está congelada, porque las pruebas la llaman por exactamente esta interfaz:

```rust
/// The chain's last link: from the INIT_SPACE half (field bytes only) to the
/// full on-chain data length. A `const fn`, so the compiler can prove it while
/// it builds (the m03-l3 device — what compile-only grading actually enforces).
const fn with_discriminator(init_space: u64) -> Option<u64> {
    // TODO: the 8 goes here -- checked, so an overflowing count stays a
    // refusal. Right now this link passes its input through unchanged.
    Some(init_space)
}

/// Returns the FULL on-chain data length for a `#[account(borsh)]` account:
/// T::DISCRIMINATOR.len() (8) + T::INIT_SPACE (the field bytes only).
fn account_len(address_fields: u64, u64_fields: u64, bool_fields: u64) -> u64 {
    // the field sums are all here; the last link is the gutted one above
    address_fields
        .checked_mul(32)
        .and_then(|bytes| bytes.checked_add(u64_fields.checked_mul(8)?))
        .and_then(|bytes| bytes.checked_add(bool_fields))
        .and_then(with_discriminator)
        .unwrap_or(0)
}
```

Deja verificado el resto de la cadena. Esos conteos llegan de quien llama, y el contrato del helper es que un conteo al que no le puede encontrar sentido vuelve como `0` — un largo que ningún asignador va a aceptar — en vez de como un número que hizo wrap y se ve bien. Aceptación: `account_len` devuelve `8 + 32*address_fields + 8*u64_fields + 1*bool_fields`; una struct vacía `(0, 0, 0)` devuelve `8`, no `0`; y la guarda sobrevive a tu edición. Cinco pruebas: `(1,1,1)` da `49`, `(2,3,0)` da `96`, `(0,0,0)` da `8`, `(1,0,2)` da `42`, y `(u64::MAX,0,0)` da `0`. Ese último vector no es una cuenta — ninguna struct tiene dieciocho trillones de campos — es el conteo basura que está en lugar de lo que sea que salió mal río arriba, y está ahí para fijar *dónde* va tu 8. Atorníllalo después de la guarda como `...unwrap_or(0) + 8` y la negativa vuelve como `8`, que se lee exactamente como una cuenta vacía legítima.

Tres pistas, en orden de cuánto revelan. La forma idiomática de V2 es `T::DISCRIMINATOR.len() + T::INIT_SPACE`, y `DISCRIMINATOR.len()` es `8`. `INIT_SPACE` son bytes de campo solamente, así que vuelves a sumar el 8 exactamente una vez, nunca por campo. Y el caso de la struct vacía es la señal: si `(0,0,0)` devuelve `0` no sumaste nada; si devuelve `16` sumaste el discriminador dos veces. Restaura el 8 como un `checked_add` dentro de `with_discriminator`, en la misma cadena que las sumas de campos, no como un `+ 8` atornillado después de `unwrap_or` — y como el eslabón es un `const fn` con aserciones debajo, un eslabón destripado o sin verificar ni siquiera compila.

## Antes de seguir adelante

La barrera de esta lección tiene dos mitades. Primero, haz que el challenge pase: starter en rojo, tu arreglo en verde, las cinco pruebas. Segundo, y esta es la que vale hacer lejos del teclado, explica de memoria por qué V2 sacó `.reload()` en vez de meramente deprecarlo. Si tu respuesta apunta al modelo de borrow de `CpiHandle`, a que el acceso tipado durante un handle vivo es un error de compilación, así que la ventana de datos obsoletos es irrepresentable y no queda nada que recargar, eres dueño de la derivación, no solo del hecho. Esa es la idea de m04-l2 aplicada a la migración, y es el ejemplo más claro que hay de la tesis del curso: el arreglo más seguro para una trampa es volver la trampa imposible de sostener.

Ahora tienes los dos mapas, 0.3x a 1.0 y 1.x a 2.0. Pero un mapa no es una migración. Después tomas un programa 0.31 o 1.0 de verdad y lo manejas todo el camino hasta un build de V2 que compila y pasa en LiteSVM, usando estos deltas como tu checklist y las advertencias del compilador como tu guía. Trae el grep. Buen port.
