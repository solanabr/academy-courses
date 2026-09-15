# Una lista que vive en los propios bytes de la cuenta

En m02-l1 construiste R1, el cabinet-counter: un `Account<T>` Pod con `play_count` y `high_score`, y levantaste el banco de pruebas de LiteSVM que ahora es el criterio de cada peldaño. Viste el estado aterrizar sin ninguna llamada a deserialize en todo el camino. El cast de bytes era todo el truco.

Acá está la cosa sobre la que ese contador miente en silencio. Un solo `high_score` no es lo que guarda una máquina arcade. Una máquina arcade guarda una *tabla*: el top ten, con iniciales y todo, en orden, para que la persona que acaba de perder por 400 puntos pueda ver exactamente qué nombre tiene que superar. Un número es un marcador para un juego que nadie juega solo.

Así que le vamos a dar a R1 un leaderboard de verdad. Y antes de cualquier teoría, haz la única edición que lo arranca. Abre el proyecto R1 y busca la struct de accounts detrás de `increment`. Esta lección la renombra `PostScore`, porque publicar un puntaje en un board es lo que está por hacer. Mira la línea que declara la cuenta cabinet:

```rust
pub cabinet: Account<Cabinet>,
```

Cámbiala por esto:

```rust
pub cabinet: Slab<Cabinet, Score>,
```

Ese es todo el movimiento estructural de esta lección en una sola línea. `Account<Cabinet>` siempre fue `Slab<Cabinet, HeaderOnly>` por debajo: un header, y después una cola de nada. Acabas de cambiar la cola vacía por una serie de ítems `Score`. La cuenta ahora lleva una lista acotada en sus propios bytes, y sigue sin deserializar nunca. El compilador se va a quejar de que `Score` todavía no existe y de que nadie dimensiona la cola. Bien. Esas dos quejas son la lección.

![En v1 cada toque deserializa y vuelve a serializar todo el Vec<Score>; en V2 el Slab es una vista de bytes mutada en el lugar, sin nada que volver a serializar.](assets/v01-comparison.png)

## La versión corta

Estás convirtiendo el contador de R1 en una tabla de puntajes acotada. Tres cosas la sostienen. Primero, el toolkit de campos Pod: `PodU64`, `PodVec<T, MAX>`, `#[derive(bytemuck::Pod)]` y `Nested<T>`, los wrappers que mantienen una struct casteable directo desde bytes. Segundo, `Slab<Header, TailItem>`, la primitiva de lista-en-cuenta, donde a un header fijo le sigue una serie acotada de ítems. Tercero, la recompensa que m01-l4 te prometió: `#[event(bytemuck)]`, un evento zero-copy que emites y vuelves a leer de los logs de la transacción.

El pero honesto atraviesa todo esto, así que escúchalo una vez de entrada: la capacidad de un Slab es fija en tiempo de compilación. Dimensionas para el peor caso, pagas rent por los slots vacíos, y una escritura más allá de `MAX` es un error duro, nunca un resize automático. Ese tope fijo no es un defecto. Es el precio exacto de una lista que nunca tienes que serializar, y elegir `MAX` es una decisión de diseño real.

Sobre la autonomía: el Lab te entrega el layout del Slab terminado y la struct del evento. La lógica de admisión y desalojo y la aserción de orden las escribes tú. El Challenge en solitario del final, el cutoff del leaderboard, lo haces sin ningún apoyo. Esta lección es donde las rueditas del layout de datos se quitan y no vuelven.

## Construir un leaderboard dentro de una sola cuenta

### El toolkit de campos Pod

Recuerda la regla de m02-l1: `Account<T>` exige `T: Pod`, plain-old-data, un layout fijo sin sorpresas de relleno, así el framework puede castear los bytes de la cuenta directo a `&T` con cero copia. Esa regla no se ablanda porque tus datos se pusieron más interesantes. Un `Score` tiene que ser Pod. Una tabla de `Score` tiene que ser Pod. Así que la primera pregunta es mecánica: ¿cómo construyes una struct Pod con los campos que de verdad quieres?

Un `u64` desnudo estaba bien en el header de R1, porque el header se sienta en un offset con alineación garantizada de 8 bytes. Un ítem de cola no recibe esa promesa: aterriza donde lo dejen el header y el campo de longitud, con un stride de `size_of::<Score>()`, y ninguna de esas dos cosas es algo sobre lo que quieras razonar campo por campo. Así que en un ítem de cola recurres al wrapper de campo en su lugar. `PodU64` es un `u64` guardado como un array de bytes con métodos de acceso, así que tiene alineación 1 y se lee correctamente desde cualquier offset. Misma historia para `PodI128`, `PodBool`, y el resto de la familia. No son tipos nuevos en los que tengas que pensar. Son los tipos viejos con el impuesto de alineación pre-pagado.

El costo que pagas por eso es un poquito de ceremonia en el punto de uso: pasas por accesores en vez de tocar el valor directamente.

```rust
let mut plays = cabinet.play_count.get();      // read: byte array -> u64
plays += 1;
cabinet.play_count = PodU64::from(plays);      // write: u64 -> byte array
// A Slab Derefs to its header, so header fields are plain field access.
// .get() reads; PodU64::from(v) / v.into() writes. There is no .set().
```

Ese `.get()` a la salida y una conversión `From` a la entrada es todo el impuesto. A cambio, el campo se lee correctamente sin importar en qué offset aterrice dentro de la cuenta, que es lo que hace casteable todo el registro.

```rust
use anchor_lang::prelude::*;

// Deriving bytemuck::Pod makes a fixed-size struct castable straight from bytes:
// every field is itself Pod, so the whole struct has a defined layout and no
// padding. This is one leaderboard entry. (#[pod_wrapper] is for ENUMS — structs
// take the derive directly, and V2 rejects the attribute on a struct outright.)
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Score {
    pub player: Address,   // 32 bytes: who set it
    pub points: PodU64,    //  8 bytes: the score
    pub slot: PodU64,      //  8 bytes: when, for tie-breaking display
}                          // 48 bytes, fixed, no padding
```

`#[derive(bytemuck::Pod)]` (en pareja con `bytemuck::Zeroable`) es lo que certifica tu propia struct fija como Pod: verifica que cada campo sea Pod y le da al tipo la bendición del cast de bytes. Recurre a `#[pod_wrapper]` solo en un *enum* — V2 rechaza el atributo en una struct de plano, con un mensaje que te dice que uses el derive en su lugar. Cuando un campo Pod es a su vez una struct que quieres anidar, la envuelves en `Nested<T>` para que su alineación siga definida dentro del padre en vez de abrir un agujero en el layout. Y cuando quieres una lista acotada como *campo* en vez de como toda la cola de la cuenta, eso es `PodVec<T, MAX>`: un vector con capacidad en tiempo de compilación, su longitud guardada inline, sin heap en ninguna parte.

![Una tabla que mapea cada wrapper Pod (PodU64, el derive bytemuck::Pod, Nested, PodVec, Slab) al tipo plano que reemplaza y a cuándo usarlo.](assets/v02-table.png)

¿Por qué el framework te hizo pasar por esto en vez de dejarte escribir `Vec<Score>` y listo? Porque no hay almuerzo gratis en el cast de bytes. Un `Vec` es un puntero, una longitud, y una capacidad que apunta a memoria del heap que no existe dentro de una cuenta. Para hacer una lista casteable tienes que disponerla plana y fija, en el lugar, y los wrappers son la forma de hacerlo sin escribir a mano la aritmética de offsets. Lo que nos lleva a la primitiva que sostiene toda la tabla.

### La alineación es el impuesto de Pod

Vale la pena ver la falla exacta que los wrappers previenen, porque es la que aparece como una advertencia que te da tentación de silenciar de la forma equivocada. Supón que te saltas el toolkit y escribes la entrada a mano como una struct desnuda con campos nativos:

```rust
// DON'T: bare multi-byte fields inside a byte-cast struct.
#[repr(C)]
pub struct Score {
    pub player: Address,  // 32 bytes, fine
    pub points: u64,      // native u64: alignment 8
    pub slot: u64,        // native u64: alignment 8
}
```

Un `u64` nativo quiere sentarse en un límite de 8 bytes. Un `Score` en la cola vive en el offset al que lo empujen el header y el campo de longitud, así que esa alineación no está garantizada, y el cast tiene que abortar en vez de entregarte una referencia desalineada, que es comportamiento indefinido en Rust. El instinto es recurrir a `#[repr(packed)]` para quitar el relleno del que la advertencia parece culpar. Eso es exactamente al revés. `repr(packed)` es lo que *crea* el peligro de referencia desalineada: tomar una referencia a un campo packed es la trampa, no el arreglo. El movimiento correcto es el toolkit. `PodU64` guarda el valor como un `[u8; 8]` que se lee por `.get()` y se escribe por una conversión `From`, así que su alineación es 1 y se lee correctamente desde *cualquier* offset, y `#[derive(bytemuck::Pod)]` (o `Nested<T>` para un campo de struct anidada) mantiene definida la alineación de todo el registro para que el cast de bytes directo siga siendo sólido.

![Un campo u64 desnudo causa una referencia desalineada en el cast; repr(packed) lo empeora al garantizar la desalineación; los campos PodU64 más el derive bytemuck::Pod dan alineación 1 y un cast sólido.](assets/v03-annotated-code.png)

### Slab: una lista que vive en la cuenta

Acá está la definición, justo a tiempo. `Slab<Header, TailItem>` es un layout de cuenta que es una struct `Header` fija seguida por una serie acotada, in-place, de registros `TailItem`. Eso es todo. El header son tus campos fijos, esos de los que cada máquina arcade tiene exactamente uno. La cola es la lista. Y la identidad que ya conociste lo hace encajar: `Account<T>` es literalmente `Slab<T, HeaderOnly>`, un header con una cola de un marcador de tamaño cero. El contador de R1 fue un Slab todo el tiempo. Solo que no tenía nada en la cola.

Así que `Cabinet` de m02-l1 se vuelve el header. Mantiene sus dos contadores y agarra dos campos nuevos que el peldaño del leaderboard necesita de todos modos: un `authority` para que los seeds del PDA ya no dependan de un solo jugador, y el `bump` guardado del que vuelves a derivar la validación:

```rust
// The HEADER: the fixed part every cabinet has exactly one of.
// R1's Cabinet, plus two new fields: authority (for the seeds) and bump.
#[account]
#[repr(C)]
pub struct Cabinet {
    pub authority: Address,
    pub play_count: PodU64,
    pub high_score: PodU64,   // still here: the single top score, for quick reads
    pub bump: u8,
}
```

Y la cuenta en tu handler es el Slab que empareja ese header con una cola de `Score`. El Slab lleva la cuenta de su propia longitud de cola y deriva su capacidad de cuánto space se le asignó a la cuenta en el init. Lees la cola como un slice y la mutas en el lugar:

```rust
#[derive(Accounts)]
pub struct PostScore {
    #[account(mut)]
    pub player: Signer,

    #[account(
        mut,
        seeds = [b"cabinet", cabinet.authority.as_ref()],
        bump = cabinet.bump,
    )]
    pub cabinet: Slab<Cabinet, Score>,
}

// The Slab surface you work against:
//   *cabinet                    -> Deref/DerefMut to Cabinet, so the header's
//                                  fields are plain field access:
//                                  cabinet.play_count.get()
//                                  cabinet.play_count = PodU64::from(n)
//   cabinet.as_slice()          -> &[Score]      (the live list, a byte view)
//   cabinet.as_mut_slice()      -> &mut [Score]
//   cabinet.len() / .capacity() -> usize         (live items / MAX at allocation)
//   cabinet.is_full()           -> bool
//   cabinet.get(i) / .get_mut(i) / .first() / .last() / .iter()
//   cabinet.try_push(Score)     -> Result<(), ProgramError>
//                                  (Err past capacity; there is no infallible push
//                                   and no implicit growth)
//   cabinet.address()           -> &Address       (the account's OWN address; this
//                                  one lives on the Slab, not on the Cabinet header)
//   cabinet.resize_to_capacity(n) -> Result<()>   (the ONLY growth path: reallocs
//                                  the account and settles the rent difference)
//   Slab::<Cabinet, Score>::space_for(MAX) -> usize   (const, for `space =`)
```

Fíjate que `cabinet.as_mut_slice()` te entrega un `&mut [Score]` a secas. Una vez que tienes ese slice, ordenar y comparar son Rust ordinario sobre memoria ordinaria, excepto que la memoria es la cuenta y cada escritura que le haces al slice ya está persistida. No hay paso de serialización al final del handler porque no hay nada que volver a serializar. El slice *son* los bytes de la cuenta.

¿Cómo sabe el Slab cuántos ítems `Score` están activos frente a cuántos slots están asignados pero vacíos? Guarda su propia longitud como un `u32` little-endian en la cuenta, justo entre el header y los ítems, igual que un `Vec` lleva la longitud separada de la capacidad, excepto que las dos viven dentro de la cuenta y ninguna puede apuntar a un heap. `capacity()` se deriva de la longitud de datos que tiene la cuenta: bytes totales, menos el discriminador, menos el header, menos ese campo de longitud, dividido por `size_of::<Score>()`. Por eso el space que asignas en el init es el techo hasta que lo cambies deliberadamente: `try_push` más allá de la capacidad devuelve un error en vez de crecer, y la única forma de que la cuenta se haga más grande es una llamada explícita a `resize_to_capacity(n)` que realoca el buffer y liquida la diferencia de rent. Nada crece a tus espaldas. Hay una primitiva hermana que vale nombrar acá para que recurras a la correcta: `PodVec<T, MAX>` es la lista acotada a *nivel de campo*, la que metes dentro de un header cuando una struct necesita su propia lista inline chica, mientras que `Slab<Header, TailItem>` es la de *nivel de cuenta*, donde la lista es toda la cola de la cuenta. Regla general: una lista acotada que es el punto de la cuenta es una cola de Slab, una lista acotada chica que cuelga de un registro más grande es un campo `PodVec`.

![La cuenta es un discriminador, después el header Cabinet fijo, después un campo de longitud activa de 4 bytes, después diez slots Score fijos de 48 bytes; los slots sin llenar siguen asignados y pagan rent.](assets/v04-diagram.png)

Ese diagrama es también el tradeoff mirándote de vuelta. Diez slots de 48 bytes son 480 bytes de cola, asignados y con el rent pagado en el momento en que inicializas la cuenta, tenga la máquina arcade un puntaje encima o diez. Que es el momento honesto sobre el que gira todo este diseño.

### Por qué MAX es fijo, y por qué eso es todo el punto

Vale la pena ir más despacio acá, porque "hazlo dinámico y ya" es el instinto obvio y vale la pena ver exactamente por qué el Slab se niega.

Arranca de lo que de verdad quieres: agregar un puntaje al final, mantener la lista ordenada, nunca perder las entradas de arriba. La respuesta ingenua es un `Vec<Score>` que crece en el heap y realoca cuando se llena. Eso falla dentro de una cuenta por una razón plana: una cuenta es una región fija de bytes con un dueño y un balance de rent, no un heap que puedas hacer crecer. No hay ningún lado *hacia* donde el `Vec` pueda crecer sin una instrucción `realloc` aparte y explícita que mueva rent y cambie el tamaño de la cuenta. El crecimiento nunca es gratis y nunca es automático.

La siguiente respuesta ingenua mantiene el `Vec` pero paga el impuesto de serialización: deserializar toda la lista al leer, volver a serializar al escribir, dejar que borsh maneje el largo variable. Ese es precisamente el modelo de v1, y es exactamente aquello de lo que m02-l1 te enseñó que el cast Pod existe para escapar. Estarías comprando de vuelta la flexibilidad al re-introducir el costo que todo el framework se construyó para eliminar. Para una lista que tocas en todas y cada una de las jugadas, ese canje está al revés.

Así que el requisito se afina: quieres mutación en el lugar con cero serialización, sobre una lista cuya longitud cambia. La única forma de tener una lista casteable desde bytes es disponerla plana y fija, lo que quiere decir que la capacidad tiene que conocerse antes de que escribas un solo byte. Eso es `MAX`. El Slab sí tiene una válvula de escape, `resize_to_capacity(n)`, que realoca la cuenta y liquida la diferencia de rent, pero fíjate que es una instrucción que corres deliberadamente, no algo que un `try_push` haga a tus espaldas. El crecimiento *implícito* y un cast de bytes con cero serialización son las dos cosas que no puedes tener a la vez, y el Slab elige el cast.

Lo que aterriza la oración para guardar: un `MAX` fijo es el precio de una lista sin serialización, porque una lista que puedes castear directo desde bytes tiene que tener su tamaño conocido antes de que se escriba el primer byte, y cambiarlo después es un realloc explícito, no un efecto secundario de insertar. Pagas rent por los slots vacíos y manejas el desalojo tú mismo, y a cambio cada toque cuesta el ítem que tocaste en vez del largo de la lista. Elegir `MAX` es ahora tu trabajo, y es uno real. Diez es un leaderboard. Diez mil es una cuenta de rent que vas a lamentar.

Recórrelo una vez en concreto, en un board diminuto con `MAX = 3`, y la disciplina de desalojo deja de ser abstracta. Arranca vacío. Entra un puntaje de `50`: el board está bajo capacidad, así que aterriza, y el cutoff, el puntaje activo más bajo, es `50`. Después `90`: sigue bajo capacidad, así que admítelo sin comparación, y el cutoff se queda en `50`, porque `50` sigue siendo el más chico de `[90, 50]`. Después `70`: el board se llena a `[90, 70, 50]`, cutoff `50`. Ahora el board está lleno y llega un `60`. Es estrictamente mayor que el cutoff `50`, así que `50` se sobrescribe en el lugar y el board queda `[90, 70, 60]`, cutoff nuevo `60`. Después llega otro `60`: *empata* el cutoff, así que se rechaza, el board queda sin cambios. Por último un `40`: abajo del cutoff, rechazado. Esa secuencia, admitir-bajo-capacidad, desalojar-solo-si-es-estrictamente-mayor, los-empates-pierden, es exactamente la lógica que escribes sobre la cola del Slab en el Lab y otra vez desde cero en el Challenge. Las mismas reglas, una vez que las ves moverse.

![Vec-con-realloc paga serialización y crecimiento manual; el Vec borsh re-introduce el impuesto de serialización; el Slab de MAX fijo castea en el lugar pero te hace pagar rent por los slots vacíos y desalojar tú mismo.](assets/v05-comparison.png)

Esta no es una preferencia abstracta que el framework inventó en el vacío. Cuando el diseño de V2 se estaba discutiendo en público, la nota más fuerte de la comunidad era exactamente esta fricción. ChewingGlass lo dijo sin rodeos en la discusión #3742, el hilo "What do you want to see in Anchor V2?": "la serialización por defecto probablemente debería comportarse más como zero-copy pero con mejor UX (o sea, no tener que intentar tener una alineación de bytes perfecta, etc). No estoy seguro de si eso es posible. Pero borsh es medio terrible." El issue de diseño #4390 cita ese comentario de vuelta, con sus propias palabras: "Como lo puso el feedback de la discusión #3742: *la serialización por defecto probablemente debería comportarse más como zero-copy pero con mejor UX*," y lista #3742 en sus referencias. En otra parte de la misma discusión el mismo comentarista aterriza la otra mitad del reclamo, sobre la ergonomía del lado del cliente: "El boilerplate mata a los devs nuevos porque no conocen los encantamientos sagrados." El Slab y el toolkit de Pod son la respuesta entregada a la primera mitad: zero-copy por defecto, con wrappers que pagan el impuesto de alineación de bytes por ti en vez de hacerlo tu problema. Voz de la comunidad convertida en estructura de datos.

![El comentario de ChewingGlass sobre zero-copy-con-mejor-UX en la discusión #3742 es citado de vuelta por el issue de diseño #4390, que alimentó el empuje de benchmarks de #4355 y se entregó como el toolkit de Pod.](assets/v06-timeline.png)

### El dividendo de Pod: eventos zero-copy

Ahora la recompensa que m01-l4 anticipó. Tienes un leaderboard que se muta en el lugar. Cuando un puntaje nuevo entra al board, quieres contarle al mundo de afuera: un indexador, un frontend, un bot de Discord que publica el nuevo top ten. En Solana eso lo haces escribiendo en los logs de la transacción, y el `emit!` de Anchor baja a `sol_log_data`, el syscall que suelta en los logs un blob con prefijo de longitud para que lo recoja cualquiera que esté leyendo la transacción.

El blob no es solo payload crudo. `emit!` le pone adelante el discriminador de 8 bytes del evento, el mismo mecanismo de tag que conociste en m01-l4, así un lector puede distinguir un tipo de evento de otro antes de intentar decodificar. Cuando sacas los logs de una transacción confirmada, cada evento emitido aparece como una línea `Program data:` que lleva ese blob, codificado en base64. Un consumidor compara los primeros 8 bytes contra el discriminador que le importa, y después decodifica el resto. Ese paso de decodificación es donde las dos variantes se separan.

El `#[event]` por defecto serializa ese blob con **wincode**. Sé preciso sobre qué es wincode, porque el nombre se usa a la ligera y en la próxima lección importa: es una *implementación* distinta, no un *formato* distinto. Su `BORSH_CONFIG` produce bytes byte-idénticos a borsh (dos excepciones documentadas, el orden de `HashMap`/`HashSet` y NaN, que conoces en m02-l3), así que todo lo que va en el wire sigue siendo compatible con borsh mientras el código que hace el trabajo es propio de V2. Lo que quiere decir que el ahorro de CU es real y la codificación sigue siendo una codificación: para una struct que ya te tomaste el trabajo de hacer Pod, ese es el impuesto de serialización colándose de vuelta por la puerta de al lado. Así que V2 te da `#[event(bytemuck)]`: el mismo camino de `sol_log_data`, pero el payload es un `memcpy` zero-copy de los bytes de la struct en vez de una serialización campo por campo. Nada de borsh, los mismos logs.

```rust
// A zero-copy event: emitted via sol_log_data as a raw memcpy of the struct.
// Same rules as any Pod type - fixed layout, Pod fields, defined alignment.
#[event(bytemuck)]
#[repr(C)]
pub struct HighScorePosted {
    pub cabinet: Address,
    pub player: Address,
    pub points: PodU64,
    pub cutoff: PodU64,   // the lowest score still on the board after this post
}
```

¿Cuánto más barato es todo esto? El único número que el proyecto publica vive en su propio `event.rs`, y se compara contra v1: los eventos wincode, el `#[event]` simple de V2, consumen de 3 a 10 veces menos unidades de cómputo que los eventos borsh de v1, según el payload. Los mismos bytes en el wire, de 3 a 10 veces menos trabajo para producirlos; eso es lo que te compra una implementación más rápida de un formato idéntico. Lo cito como la afirmación del proyecto, no lo lavo para convertirlo en un hecho medido mío, y fíjate en lo que *no* dice: no le pone ningún número a `#[event(bytemuck)]` frente al `#[event]` simple. Esa brecha la mides tú mismo con `anchor test --profile` cuando llegues al módulo de instrumentación, y deberías. El mecanismo, eso sí, no está en duda: un `memcpy` de una struct fija es estrictamente menos trabajo que recorrer sus campos por cualquier serializador, wincode incluido.

Hay una trampa que viene gratis con la velocidad, y es del tipo que falla en silencio en producción. `#[event]` y `#[event(bytemuck)]` escriben *bytes distintos* en el log. Uno es formato wincode, uno es un memcpy crudo. Las dos salen por `sol_log_data`, así que el evento está genuinamente en los logs de cualquiera de las dos maneras. Pero un lector construido para decodificar la variante borsh va a sacar basura de los bytes bytemuck, y al revés también. Todo consumidor aguas abajo tiene que decodificar con la misma variante que emite el programa. Elige una, anótala, y asegúrate de que tu indexador se enteró.

![Las dos variantes de evento emiten por sol_log_data, pero una lleva bytes borsh y la otra un memcpy, así que un lector tiene que decodificar con la variante que usó el programa.](assets/v07-diagram.png)

Ese es el toolkit. Un tipo de entrada Pod, un Slab para contener una serie acotada de ellas, un `MAX` fijo que elegiste a propósito, y un evento zero-copy para anunciar los cambios. Hora de conectarlo a R1 y verlo correr.

## Lab: atorníllale una tabla de puntajes a R1

Vas a extender el handler `PostScore` de R1 para que cada puntaje se admita en un board acotado y ordenado, y para que un `#[event(bytemuck)]` se dispare cuando el board cambia. El layout del Slab y la struct del evento de arriba te los dan. La lógica de admisión y desalojo, y la aserción de la prueba de que el board sigue ordenado y acotado, te toca escribirlas a ti. Esa división es deliberada: la forma se muestra, la disciplina se practica.

Primero, el toolchain. Instalaste la RC allá en m01-l2 y la confirmaste otra vez al comienzo de m02-l1; si falta, acá está el mismo build documentado desde git, porque Anchor V2 es una release candidate que se instala desde git, no desde el canal de binarios precompilados de `avm`:

```bash
# Anchor V2 2.0.0-rc.1 - git otter-sec/anchor, tag v2.0.0-rc.1 (commit e4878b6d).
# RC/alpha, and no release binary is published for it. The tag is a fixed point;
# the anchor-next branch tip it sits on is not. The Docker verify gate in m08-l2
# names the same commit outright. Re-verify the tag before you rely on it.
# macOS, if the build trips on LTO: prefix that line with CARGO_PROFILE_RELEASE_LTO=off
cargo install --git https://github.com/otter-sec/anchor.git \
  --tag v2.0.0-rc.1 anchor-cli --locked --force
```

El banco de pruebas es el mismo que levantaste en m02-l1: `anchor-v2-testing`, que envuelve LiteSVM y re-exporta las piezas que el archivo de pruebas necesita. Sigues sin depender de `litesvm` por nombre. La única dev-dependency nueva que agrega esta lección es `base64`, para la decodificación del evento de más abajo:

```toml
# programs/cabinet-counter/Cargo.toml
# anchor-v2-testing at tag v2.0.0-rc.1 pins litesvm 0.11.0 internally (crates.io
# latest is 0.15.2 as of 2026-08-22; do not float ahead of the harness by pulling
# litesvm in yourself). This is why the tag and not the branch: the anchor-next tip
# has already moved that pin to =0.13.1, and a harness that changes SVM majors
# under a green test suite is the exact failure the pin exists to prevent.
# base64 0.22 is pinned on purpose (0.23.1 is current as of 2026-08-22):
# re-verify the Engine API before bumping it.
[dev-dependencies]
anchor-v2-testing = { git = "https://github.com/otter-sec/anchor.git", tag = "v2.0.0-rc.1" }
base64 = "0.22"
```

Ahora los pasos.

1. **Cambia el tipo de cuenta, y renombra lo que al peldaño le quedó chico.** Ya hiciste el cambio de tipo en la apertura: `pub cabinet: Account<Cabinet>` se vuelve `pub cabinet: Slab<Cabinet, Score>`. Ahora termina el rename que viene con eso, porque R1 ya no está contando, está publicando: `Increment` se vuelve `PostScore` y `increment` se vuelve `post_score`; `Init` se vuelve `InitCabinet` e `init` se vuelve `init_cabinet`. Agrega la struct `Score` con `#[derive(bytemuck::Pod, bytemuck::Zeroable)]` y el evento `HighScorePosted` con `#[event(bytemuck)]`, los dos exactamente como se muestran en la teoría de arriba, y dale al header `Cabinet` sus dos campos nuevos, `authority` y `bump`.

   `init_cabinet` es más que un rename, porque esos dos campos nuevos del header los tiene que escribir alguien y nada más lo va a hacer. Agrega las dos líneas al cuerpo del handler:

   ```rust
   pub fn init_cabinet(ctx: &mut Context<InitCabinet>) -> Result<()> {
       let bump = ctx.bumps.cabinet;               // the canonical bump the macro found
       let cabinet = &mut ctx.accounts.cabinet;
       cabinet.authority = *ctx.accounts.authority.address();
       cabinet.bump = bump;
       cabinet.play_count = PodU64::from(0);
       cabinet.high_score = PodU64::from(0);
       Ok(())
   }
   ```

   Sáltate esas dos asignaciones y todo sigue compilando, que es la parte peligrosa: `authority` se queda todo en cero y `bump` se queda en `0`, así que cada `post_score` posterior falla su constraint de `seeds` y de `bump` contra una dirección que nunca fue derivable, y el error no te dice nada sobre por qué.

   Resultado esperado después de este paso: todavía no compila, y la prueba `cabinet_round_trips` de m02-l1 tampoco. Eso es correcto y vale nombrarlo en vez de descubrirlo. La prueba llama a `instruction::Init` y `accounts::Init`, que ya no existen con esos nombres, y comprueba `raw.len() == 24`, que era cierto de una cuenta con solo header y es falso en el momento en que hay una cola. Vas a reemplazar esa aserción en el paso 4. Reapunta los builders de la prueba a `InitCabinet`/`PostScore` ahora, así lo único que queda en rojo es la lógica que estás por escribir. Si te trajiste a este workspace la prueba de reset que venía en el Challenge de m02-l1, se pone roja por la misma razón — los mismos renames, el mismo arreglo de reapuntado.

2. **Dimensiona la cola en el init.** En los constraints de cuentas de tu handler `init_cabinet`, el `space` del cabinet ahora tiene que cubrir el header más los slots de puntajes. Define `MAX_SCORES = 10` como `const` y deja que el Slab haga la aritmética: el discriminador, más el header `Cabinet`, más el campo de longitud activa de 4 bytes, más `MAX_SCORES * size_of::<Score>()`. Eso es exactamente lo que calcula `space_for`, que es por lo que nunca lo escribes a mano:

   ```rust
   pub const MAX_SCORES: usize = 10;

   #[derive(Accounts)]
   pub struct InitCabinet {
       #[account(mut)]
       pub authority: Signer,
       #[account(
           init,
           payer = authority,
           // discriminator + Cabinet header + len field + MAX_SCORES slots.
           // space_for is a const fn on the Slab, so you never hand-roll the math.
           space = Slab::<Cabinet, Score>::space_for(MAX_SCORES),
           seeds = [b"cabinet", authority.address().as_ref()],
           bump,
       )]
       pub cabinet: Slab<Cabinet, Score>,
       pub system_program: Program<System>,
   }
   ```

   Eso son 480 bytes de cola (`10 * 48`), más el header y su campo de longitud de 4 bytes, asignados en el instante en que el cabinet existe, llenos o no. Acá es donde comprometes rent a los slots vacíos, así que vale la pena leer el número de la cuenta y sentirlo: una máquina arcade vacía y una llena cuestan lo mismo. Ese es el tradeoff, hecho concreto. Si hubieras elegido `MAX_SCORES = 1000`, estarías pagando rent por 48,000 bytes de cola para un leaderboard que casi ninguna máquina arcade va a llenar nunca.

3. **Escribe la lógica de admisión y desalojo.** Esta es la parte que implementas tú. En `post_score`, después de incrementar `play_count`, inserta el `Score` nuevo en la cola y mantén el board acotado y ordenado de mayor a menor. Las reglas:
   - Mientras `cabinet.len() < cabinet.capacity()`, siempre haz `try_push` del puntaje.
   - Cuando el board está lleno, encuentra el cutoff actual (el puntaje activo más bajo). Admite solo si el puntaje nuevo es *estrictamente* mayor que el cutoff, y cuando lo es, sobrescribe el slot del cutoff en el lugar por `cabinet.as_mut_slice()`. Un empate no desaloja.
   - Ordena la cola activa de forma descendente para que el slot 0 sea siempre el puntaje más alto, y mantén el `high_score` del header sincronizado con el slot 0.

   ```rust
   pub fn post_score(ctx: &mut Context<PostScore>, points: u64) -> Result<()> {
       let cabinet = &mut ctx.accounts.cabinet;
       let player = *ctx.accounts.player.address();
       let cabinet_address = *cabinet.address();

       // routine: one more play recorded (Slab derefs to the Cabinet header)
       let plays = cabinet.play_count.get().checked_add(1)
           .ok_or(CabinetError::Overflow)?;
       cabinet.play_count = PodU64::from(plays);

       let entry = Score {
           player,
           points: PodU64::from(points),
           slot: PodU64::from(Clock::get()?.slot),
       };

       // TODO(you): admit `entry` under the fixed capacity.
       //   - under capacity  -> cabinet.try_push(entry)?;
       //   - full + strictly beats cutoff -> overwrite the cutoff slot in place
       //   - full + ties or loses -> the board is unchanged. Do NOT return early:
       //     every call emits, so a rejected post reports the unchanged cutoff and
       //     an indexer can still see that the attempt happened.
       //   - then sort cabinet.as_mut_slice() descending and sync high_score
       // The cutoff you compute here is the value you emit below.
       let cutoff = todo!("return the lowest live score after admitting");

       emit!(HighScorePosted {
           cabinet: cabinet_address,
           player,
           points: PodU64::from(points),
           cutoff: PodU64::from(cutoff),
       });
       Ok(())
   }
   ```

   La lógica de acá es la misma disciplina que el Challenge en solitario de más abajo, solo que operando sobre una cola de Slab en vez de un `Vec`. Hazla funcionar acá donde puedes ver la cuenta, y después hazla en frío en el Challenge.

4. **Escribe la prueba de LiteSVM.** Inserta *más de* `MAX` puntajes en un mismo cabinet, provocando overflow del board a propósito, y después comprueba dos cosas. Primero, que la cola contiene exactamente `MAX` entradas y que están en orden descendente, con solo los puntajes más altos retenidos. Segundo, vuelve a leer el evento `HighScorePosted` desde los logs de la transacción y comprueba que su `cutoff` coincide con el puntaje activo más bajo del board. La decodificación del evento es la única pieza de plomería de pruebas que no has visto, así que acá está, completa:

   ```rust
   // Pull a #[event(bytemuck)] payload back out of the transaction logs.
   // The program wrote it via sol_log_data as: [8-byte discriminator][raw struct].
   // We match the discriminator, then bytemuck-cast the rest. Decoding with the
   // SAME variant the program emitted is the rule from the theory - here it is bytemuck.
   use base64::Engine as _; // decode() is a trait method on Engine

   fn decode_highscore(logs: &[String]) -> HighScorePosted {
       for line in logs {
           let Some(b64) = line.strip_prefix("Program data: ") else { continue };
           let bytes = base64::engine::general_purpose::STANDARD
               .decode(b64).expect("valid base64 program data");
           if bytes.len() >= 8 && &bytes[..8] == HighScorePosted::DISCRIMINATOR {
               return *bytemuck::from_bytes::<HighScorePosted>(&bytes[8..]);
           }
       }
       panic!("no HighScorePosted event in logs");
   }
   ```

   ```rust
   // TODO(you): the overflow-and-order assertion.
   //   - post MAX_SCORES + 3 scores with distinct values through post_score
   //   - fetch the cabinet, read its Slab tail as &[Score] via as_slice()
   //   - assert tail.len() == MAX_SCORES
   //   - assert the tail is sorted descending (non-increasing: equal scores are
   //     legal side by side, because a tie only loses against a FULL board's cutoff)
   //   - assert the smallest retained score equals the last event's cutoff
   ```

5. **Córrelo.** Con el toolchain instalado y la prueba escrita:

   ```bash
   anchor test
   ```

   Una corrida verde demuestra la forma que construiste: el board admitió más puntajes que `MAX`, se quedó solo con los `MAX` más altos en orden descendente, desalojó al resto, y el evento zero-copy hizo el ida y vuelta por los logs con un `cutoff` que concuerda con el board. Si la aserción de la cola falla con entradas fuera de orden, tu ordenamiento corrió antes de la inserción o ordenaste de forma ascendente. Si `decode_highscore` entra en panic, o emitiste el `#[event]` borsh por error o tu lectura de indexador está buscando el discriminador equivocado. Las dos son la trampa de "decodifica la variante que emitiste" apareciendo exactamente donde la teoría dijo que aparecería.

![Si el board está bajo MAX, haz push; si está lleno, admite solo por encima del cutoff, sobrescribiendo ese slot; los empates se rechazan, los puntajes admitidos se ordenan de forma descendente, y todo camino igual emite el evento.](assets/v08-flowchart.png)

## Challenge: el cutoff del leaderboard

Ahora el peldaño en solitario, sin Slab y sin cuenta, solo la disciplina destilada en una función pura sobre la que puedes razonar en aislamiento. Esta es la lógica de inserción acotada que un Slab Pod te impone on-chain, levantada a un lugar donde nada más se interpone.

Implementa `admit`:

```rust
/// Admit a new score to a fixed-capacity cabinet high-score board and return
/// the LOWEST score still on the board afterwards - the leaderboard "cutoff".
///
/// Rules:
///   * while the board has fewer than `cap` entries, always admit the score;
///   * once the board is full, the board retains the `cap` highest scores, so a
///     score that only ties the current cutoff cannot raise it;
///   * keep the board bounded and highest-first, including when the board you
///     were handed already exceeds `cap`.
///
/// Return the cutoff (the minimum retained score), or 0 for an empty board.
fn admit(board: Vec<u64>, score: u64, cap: usize) -> u64 {
    todo!()
}
```

La barra de aceptación, los siete casos:

- Un puntaje admitido bajo capacidad aparece en el board y el cutoff lo refleja.
- En un board lleno, solo un puntaje estrictamente mayor que el cutoff desaloja al mínimo.
- Un puntaje que solo empata el cutoff deja el cutoff exactamente donde estaba.
- El board nunca excede `cap` — y dos casos te entregan un board que *arranca* sobre capacidad, así que el recorte tiene que pasar sea que el puntaje nuevo se admita o no.
- El valor devuelto es el puntaje retenido mínimo, y `0` cuando el board está vacío.

Tres pistas, en el orden en que las vas a querer:

1. Mientras `board.len() < cap`, todo puntaje se admite sin ninguna comparación.
2. Cuando el board está lleno, la decisión de admisión es `beats_cutoff`, la `const fn` chica arriba de `admit`: estrictamente mayor que el mínimo actual entra, un empate no. Impleméntala ahí y enruta por ella la rama de board lleno.
3. Ordena de mayor a menor, trunca a `cap`, y devuelve el último puntaje retenido, el más chico. El truncado es el paso que los casos sobre capacidad existen para atrapar — devuelve el mínimo del board *retenido*, no del board que te entregaron.

El starter y las pruebas están en `lessons/m02-l2/high-score-cutoff/`. La regla de admisión está factorizada en `beats_cutoff` para que el compilador pueda demostrarla mientras construye — un dispositivo de aserción en tiempo de compilación que vas a volver a encontrar en el Challenge de constraint de m03-l3, y lo que la calificación de producción solo-por-compilación realmente impone: una regla sin arreglar no compila en absoluto. Corre también los siete vectores, hasta que los siete pasen. La función es chica. El punto no es el volumen de código, es internalizar que on-chain no puedes salir de esto creciendo en el heap. El límite es todo el juego, así que la inserción tiene que respetarlo todas y cada una de las veces.

## Antes del próximo peldaño

Responde esto en una sola oración antes de seguir, porque es el concepto que tiene que quedarse: ¿por qué un `MAX` fijo en tiempo de compilación es el precio de una lista sin serialización? Si tu oración aterriza en "porque una lista que casteas directo desde bytes tiene que saber su tamaño antes de que se escriba el primer byte, así que cambias crecimiento dinámico por cero serialización y manejas el desalojo tú mismo", la tienes. Si no, vuelve a leer la sección del tradeoff, porque cada decisión con forma de lista que tomes en este framework de acá en adelante pasa por ahí.

Acá te ganaste un Checkpoint de verdad. R1 pasó de un solo número a un leaderboard acotado, ordenado, in-place, que nunca deserializa, y anuncia sus cambios con un evento zero-copy que puedes leer de los logs. Esa es una estructura de datos on-chain genuinamente no trivial, y construiste la mitad difícil tú mismo. Hay una razón por la que V2 tuvo que demostrar que estas primitivas valían su cómputo: el issue #4355 planteó los benchmarks de V2 contra Quasar y Pinocchio antes de la conferencia Accelerate a principios de mayo de 2026, como tarea de ruta crítica, precisamente porque las listas y los eventos sin serialización tienen que ganarse sus números de CU o toda la tesis del zero-copy es puro discurso. Acabas de correr la tesis.

Las listas Slab son rápidas porque todo en ellas es de tamaño fijo. Pero algunos datos son genuinamente de largo variable: una etiqueta libre de máquina arcade, un conjunto sin cota de algo que no puedes acotar en tiempo de compilación. Cuando la rigidez de Pod deja de calzar con la forma de tus datos, V2 te entrega una escotilla de escape, y te cuesta el cast de bytes recuperar la flexibilidad. Después, en m02-l3, esa escotilla: borsh, `BorshAccount<T>`, y exactamente cuándo recurrir a ella es la decisión correcta en vez de una falta de agallas.
