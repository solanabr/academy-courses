# Destapando `#[derive(Accounts)]`: despacho, discriminadores y errores

La lección pasada leíste en qué se expanden `declare_id!` y `#[program]`, y cerraste el loop de deploy e invocación sobre el greeter. Viste el lado del programa. Pero un programa V2 tiene dos mitades, y la mitad que de verdad decide si una transacción maliciosa llega a correr es la que todavía no abriste: `#[derive(Accounts)]`.

Ese derive es silencioso. Escribes cuatro campos y un par de atributos `#[account(...)]`, y genera la carga de cuentas, las verificaciones de constraints, una guarda de duplicados mutables y el wireado que el dispatcher llama antes de que tu handler llegue a ejecutarse. Esta lección lo abre. Para el final vas a poder trazar el orden generado, derivar a mano las preimágenes de discriminador de Anchor y leer el layout de errores del framework frente a los propios lo bastante bien como para predecir un código de error en el wire antes de correr el programa.

Esta es la forma más rápida de volver concreta una de esas ideas ahora mismo. Un discriminador no es más que los primeros 8 bytes de un hash sha256 sobre una cadena con namespace. Tu instrucción `greet` del greeter tiene uno, y puedes calcularlo en tu terminal:

```bash
# shasum ships with macOS and most Linux distros. If yours lacks it,
# swap in the coreutils equivalent: sha256sum.
echo -n "global:greet" | shasum -a 256 | cut -c1-16
```

Esa cadena hexadecimal es la etiqueta exacta de 8 bytes que un cliente pone delante de los datos de la instrucción para que el dispatcher sepa a qué handler llamar.

## Resumen

La ruta sigue el orden propio del framework. Primero lees lo que el derive genera sobre el greeter que ya tienes, carga después constraints después despacho. Después la guarda de duplicados mutables, donde una máscara de tiempo de compilación se cruza con una verificación en tiempo de ejecución y la mayoría de la gente adivina la equivocada. Después los discriminadores y el layout de códigos de error, que son las dos superficies que tus clientes de verdad ven en el wire. El Lab extiende R0 con una segunda instrucción y un struct de cuentas pequeño, así que cada una de esas superficies se vuelve algo que puedes ver dispararse.

El repliegue de la ayuda de esta lección: el recorrido por la expansión del derive está totalmente resuelto, hecho para ti paso a paso. El Challenge de la preimagen del discriminador es un problema Completion, un starter que falla hasta que lo arreglas. La predicción del error propio y el razonamiento sobre los duplicados mutables son Solo, sin apoyo.

## Destapando el derive

### Qué genera el derive, en orden

Arranca desde el statu quo y su límite. Un programa Solana crudo recibe un slice plano de cuentas y un slice plano de bytes. Cada propiedad de seguridad que te importa, que esta cuenta es un signer, que esta otra es propiedad de tu programa, que esta pubkey de verdad es el PDA que crees que es, hay que verificarla a mano, en el orden correcto, sin ayuda del compilador. Olvida una verificación y tienes una vulnerabilidad. La razón entera por la que existe `#[derive(Accounts)]` es mover esa lista de verificación de algo que recuerdas a algo que genera la macro.

Así que la pregunta natural es: ¿qué genera exactamente, y en qué orden? Porque el orden decide si un constraint te protege o corre demasiado tarde para importar.

El derive genera tres fases, y siempre corren en esta secuencia:

1. **Load.** Cada campo se deserializa desde el slice crudo de cuentas hacia su wrapper tipado. `Account<Marquee>` verifica el dueño y el discriminador de 8 bytes y te entrega una vista tipada. `Signer` verifica el bit de firma. `Program<System>` verifica la dirección y el flag de ejecutable. Si un campo no puede cargar como su tipo declarado, la validación se detiene acá, antes de que corra cualquiera de tus constraints.
2. **Constraints.** Los atributos `#[account(...)]` se disparan como hooks: `mut`, `init`, `seeds` y `bump`, `has_one` (que V2 deprecia en favor de `address = ...`; todavía parsea, con una advertencia), `constraint = ...`. Estos corren después de la carga porque la mayoría necesita los datos ya cargados para verificar algo. Un `has_one = authority` no puede comparar contra un campo que todavía no deserializó.
3. **Despacho.** Solo una vez que la carga y los constraints pasaron, el dispatcher le entrega el `ctx.accounts` validado a tu handler. El cuerpo de tu handler es lo último que corre, no lo primero.

![Un flujo de arriba hacia abajo de tres fases, carga después constraints después despacho, donde cada fase solo corre si la anterior pasó y el cuerpo del handler corre al final.](assets/v01-flowchart.webp)

Ese ordenamiento es el modelo mental que llevas por el resto del curso. Cada constraint que escribas vive en la fase dos, lo que quiere decir que puede suponer que la cuenta ya cargó como su tipo, y corre antes de tu lógica, lo que quiere decir que un constraint que falla te cuesta la comisión de la transacción pero nunca deja que una cuenta mala llegue a tu handler.

### La expansión, mostrada de verdad

Las fases abstractas están bien, pero no tienes que aceptarlas por fe. El derive genera una implementación de trait de verdad, y puedes verla. Si tienes `cargo-expand` instalado, apúntalo a tu programa y lee la salida:

```bash
cargo install cargo-expand
cargo expand --package greeter
```

Deja ese comando para el paso 1 del Lab. Ahora mismo tu programa tiene un solo struct de cuentas, `Greet`, con un único `Signer` adentro, y expandir eso te muestra una función de dos líneas que no demuestra nada. El struct cuya expansión vale la pena leer es `LightMarquee`, que escribes en el Lab: tres campos, un `payer: Signer` con `mut`, un `marquee: Account<Marquee>` con `init`, y un `system_program: Program<System>`. Así que lee el bosquejo de abajo ahora, escribe el struct en el Lab, después corre `cargo expand` y calza la salida real contra él.

Para ese struct, el derive genera una implementación del trait `TryAccounts` cuya función `try_accounts` es, en esencia, las tres fases escritas como código de línea recta. Corre en orden de declaración de campos, que es por qué el orden en que escribes tus campos es el orden en que cargan:

![Un bosquejo del dispatcher comprobando el bitvec de duplicados recorrido contra el MUT_MASK de LightMarquee, y después try_accounts cargando y aplicando constraints a cada campo en orden de declaración antes de que corra el handler.](assets/v02-annotated-code.webp)

Ese bosquejo está simplificado a propósito, pero la estructura es fiel. Vale la pena sacarle tres cosas, porque responden preguntas que la versión abstracta deja abiertas.

Primero, el orden de los campos es el orden de carga. La macro recorre tu struct de arriba abajo. Si el constraint de un campo posterior depende de un campo anterior, por ejemplo un `address = config.authority` que compara contra una cuenta `config` declarada arriba, el campo anterior tiene garantizado haber cargado primero. Reordena tus campos y de verdad puedes cambiar qué verificación corre contra datos cargados versus sin cargar.

Segundo, la guarda de duplicados mutables no está metida dentro de la carga de ningún campo en particular, y ni siquiera vive en `try_accounts`. El dispatcher recorre primero las vistas de cuenta que entran, anota cualquier dirección que aparezca dos veces en un bitvec, y hace AND de ese bitvec contra el `MUT_MASK` que el struct fija en tiempo de compilación, en una única comprobación de cuatro palabras, antes de que arranque la carga tipada. Los compuestos se manejan en tiempo de compilación y no en tiempo de ejecución: un campo `Nested<Inner>` pliega el `MUT_MASK` propio del struct interno, desplazado por el offset de ese campo, dentro de la máscara externa. Así que una sola comprobación cubre todo el árbol de cuentas, y atrapa una colisión incluso cuando la misma cuenta se pasa a un campo directo y a un campo enterrado dentro de un compuesto.

Tercero, el handler no aparece en `try_accounts` en absoluto. La carga y la validación son una función generada; tu handler es una función aparte que el dispatcher llama solo después de que `try_accounts` devuelve `Ok`. Esa separación es la razón estructural de que un constraint nunca pueda correr "demasiado tarde": físicamente no puede, porque vive en una función que termina antes de que tu código arranque.

### La guarda de duplicados mutables: máscara en tiempo de compilación, verificación en tiempo de ejecución

Ahora el detalle más filoso de V2, y el que vale la pena frenar para mirar, porque vive exactamente sobre la costura entre lo que el compilador sabe y lo que solo el runtime puede saber.

Considera una instrucción que toma dos cuentas del mismo tipo, las dos mutables:

```rust
#[derive(Accounts)]
pub struct TallyTwo {
    #[account(mut)]
    pub first: Account<Marquee>,
    #[account(mut)]
    pub second: Account<Marquee>,
}
```

Quien llama controla qué pubkeys aterrizan en `first` y `second`. Nada le impide pasar la *misma* cuenta para los dos. Y eso es genuinamente peligroso. Tanto `first` como `second` deserializarían los mismos bytes de abajo hacia dos copias mutables separadas. Tu handler muta `first`, después muta `second`, y al salir Anchor serializa las dos de vuelta. La segunda escritura pisa la primera. Le sumaste uno a un contador y subió en uno en vez de dos, en silencio, sin error. Este es un bug clásico de aliasing de cuentas, y V2 lo prohíbe por defecto.

Esta es la pregunta que importa: ¿*dónde* se atrapa esa colisión? La respuesta ingenua es "en tiempo de compilación, el compilador sabe que hay dos campos mutables." Eso es cierto a medias, y es la mitad que hace tropezar a la gente. El compilador sí sabe la *forma*: qué campos son mutables y se serializan al salir. El derive codifica eso como una const asociada de 256 bits, `MUT_MASK`, un bit por slot de cuenta, prendido para cada campo mutable que se serializa. Esa máscara es fija en tiempo de compilación y no cuesta nada en tiempo de ejecución.

Pero el compilador no puede saber los *valores*. Que `first` y `second` guarden la misma dirección depende por completo de lo que manda quien llama, y eso solo se puede saber cuando llega la transacción. Así que el runtime hace la otra mitad: el dispatcher recorre las vistas de cuenta que entran, prende un bit para cada slot cuya dirección ya vio, y hace AND de ese bitvec contra `MUT_MASK`. Si algún bit sobrevive, dos slots mutables llevan la misma dirección, y la llamada devuelve `ConstraintDuplicateMutableAccount` desde el dispatcher, en tiempo de ejecución, antes de que corra tu handler.

![Un diagrama en dos partes de MUT_MASK: una máscara de bits fija en tiempo de compilación sobre los campos mutables, comparada contra un bitvec de direcciones repetidas en tiempo de ejecución, donde cualquier bit que sobreviva levanta un error.](assets/v03-diagram.webp)

Hay una segunda cosa, aparte, que la gente confunde con esta, y fijarla es todo el punto del Checkpoint más adelante. Si de verdad *quieres* pasar la misma cuenta mutable dos veces, porque tu handler está escrito para nunca mantener referencias mutables en conflicto, optas por salirte campo por campo:

```rust
#[derive(Accounts)]
pub struct TouchTwice {
    #[account(mut, unsafe(dup))]
    pub first: Account<Marquee>,
    #[account(mut, unsafe(dup))]
    pub second: Account<Marquee>,
}
```

La salida se escribe `unsafe(dup)`. El `unsafe` es deliberado: hacer aliasing sobre los datos de una cuenta mutable es una trampa, y V2 te obliga a nombrarla. Ahora la distinción. Escribir `dup` a secas — la forma de escribir este atributo en v1 — sin `unsafe` es un *error de compilación*, y el compilador te dice que escribas `unsafe(dup)` en su lugar. Ese es un evento de tiempo de compilación sobre el *atributo que escribiste*. No tiene nada que ver con que dos cuentas choquen de verdad. La colisión en sí, el mismo pubkey llegando a los dos slots, se atrapa en *tiempo de ejecución*, en el dispatcher, contra ese bitvec recorrido. Dos eventos distintos, dos momentos distintos. No dejes que la palabra compartida "dup" los difumine.

Una nota sobre el alcance, porque te ahorra una tarde de confusión. La guarda se activa según el atributo `mut`, no según el tipo de wrapper. Cualquier campo marcado con `mut` prende su bit, tanto `Account<T>` como `Signer`; un campo sin `mut` no prende ninguno, así que pasar la misma cuenta de solo lectura a dos slots siempre está bien. Dos excepciones recortan la máscara: un campo con `unsafe(dup)` queda excluido por diseño, y un campo `Option<_>` queda excluido de la máscara de tiempo de compilación (un slot `None` se codifica como el id del programa, que de otro modo se leería como una colisión) y a cambio recibe una verificación más angosta, campo por campo. Por esto nuestro `TallyTwo` marca los dos campos con `mut`: sin ese atributo no habría nada que chocara.

Eso sí, nada de esto es creerle a Anchor sin más. El dispatcher que estás leyendo está sometido a fuzzing él mismo. El changelog de V2 le acredita al fuzzing el hallazgo de 4 bugs de corrección en el framework, registrados en el issue #4431, y la suite de pruebas de Anchor lleva witnesses de Miri y configuraciones de Kani. El pegamento generado que recorre tus cuentas está verificado contra comportamiento indefinido de la misma forma en que verificarías un programa al que estás por ponerle dinero detrás. Eso es algo razonable en lo que confiar, precisamente porque está verificado y no afirmado.

### Los discriminadores reciben su hogar con nombre

Ya calculaste uno al principio de la lección. Ahora démosle a la idea su forma completa, porque hay tres namespaces y exactamente uno de ellos sorprende a la gente.

Un discriminador es una etiqueta de 8 bytes que Anchor antepone para que el runtime pueda distinguir una cosa de otra sin parsear todo el payload. Las cuentas reciben uno para que una deserialización pueda rechazar el tipo de cuenta equivocado a primera vista. Las instrucciones reciben uno para que el dispatcher pueda enrutar al handler correcto. Los eventos reciben uno para que un indexador pueda decir cuál evento decodificó. Por defecto todos y cada uno son los primeros 8 bytes de `sha256` sobre una *cadena de preimagen con namespace*, y el namespace es la parte que carga la trampa:

- un struct de cuenta hashea desde `account:<Name>`, por ejemplo `sha256("account:Marquee")[..8]`
- un struct de evento hashea desde `event:<Name>`, por ejemplo `sha256("event:MarqueeLit")[..8]`
- un handler de instrucción hashea desde `global:<Name>`, por ejemplo `sha256("global:greet")[..8]`

![Una comparación en tres filas de los namespaces de discriminador que muestra que los structs de cuenta usan account, los eventos usan event y los handlers de instrucción usan global, con el namespace global marcado como la trampa común.](assets/v04-comparison.webp)

¿Por qué `global:`? Historia. El Anchor temprano puso los handlers de instrucción bajo un namespace de estado `global`, un diseño que en gran parte desapareció pero dejó atrás la convención de la preimagen. No hay ningún namespace `instruction:` y nunca lo hubo. Si alguna vez armas a mano una etiqueta de instrucción desde `instruction:<Name>`, tus bytes no van a calzar con los que generó el programa, y el dispatcher va a rechazar la llamada como una instrucción desconocida.

Unos cuantos hechos más fijan la superficie, y cuentan en el momento en que te importa la compatibilidad de wire. Los discriminadores de V2 están sin cambios por defecto: el mismo esquema sha256 de 8 bytes, así que el formato de wire de un programa V2 se mantiene compatible con un cliente v1 que ya sabe armar estas etiquetas. V2 sí endurece una regla: un discriminador de cuenta todo en ceros se rechaza, porque todo en ceros es como se lee una cuenta sin inicializar, así que permitirlo dejaría que una cuenta vacía se hiciera pasar por una real. Ese rechazo está especificado para los discriminadores de cuenta en particular, la confusión que evita solo existe para las cuentas.

Y hay una compactación opcional. Si 8 bytes al frente de cada instrucción te parecen mucho, V2 te deja anotar un handler con `#[discrim = N]`, que cambia su prefijo sha256 de 8 bytes por una etiqueta entera pequeña. Es ingeniería honesta, pero lee el canje antes de echarle mano, porque no es la sobrescritura por ítem que podrías esperar.

![Una tabla comparativa de los discriminadores sha256 de 8 bytes por defecto contra discriminadores de instrucción compactos, que muestra que lo compacto ahorra bytes pero es todo-o-nada por programa, agrega validación de ambigüedad de prefijos y deja de lado la compatibilidad de wire con v1 por defecto.](assets/v05-table.webp)

Fíjate en la forma de ese canje. El valor por defecto te cuesta 8 bytes en el wire más las unidades de cómputo para compararlos, y te compra legibilidad y compatibilidad con v1 gratis. La opción compacta ahorra los bytes y el cómputo, pero es todo-o-nada por programa, hay que validarla contra la ambigüedad de prefijos para que dos etiquetas no puedan hacer alias, y deja de lado la compatibilidad con v1 por defecto. Compatibilidad y legibilidad frente a eficiencia pura. Esa es toda la decisión, y para la mayoría de los programas gana el valor por defecto, que es exactamente por qué el camino compacto sigue siendo un caso de borde.

Jacob Creech, describiendo el rediseño del discriminador, puso la realidad del uso sin adornos: los discriminadores personalizados "being considered ... almost have 0 usage today." Ese es el contexto de por qué `#[discrim = N]` es un caso de borde opcional y no el valor por defecto. El valor por defecto es el valor por defecto porque es lo que, en esencia, todo el mundo termina entregando.

### El layout de errores: el framework en los 2000, los propios en 6000

La última pieza de la superficie es lo que ve un cliente cuando algo falla. Los códigos de error de Anchor están particionados en rangos, y la partición no es arbitraria. Existe para que un error del framework y un error tuyo nunca puedan chocar en el wire.

Cada constraint que el framework aplica devuelve un código en su propia banda. La que vas a encontrar todo el tiempo es la banda de constraints, que arranca en 2000. `ConstraintHasOne`, el error que lanza un `has_one` violado, es 2001. `ConstraintDuplicateMutableAccount`, la guarda que acabamos de trazar, vive en ese mismo territorio del framework. Tus propios errores, los que declaras con `#[error_code]`, arrancan en 6000 y suben desde ahí, indexados desde cero por variante.

![Un gráfico por bandas que muestra los rangos de códigos de error en Anchor, que va desde los errores de instrucción en 100, pasando por los errores de constraints en 2000, hasta los errores propios que empiezan en 6000.](assets/v06-comparison.webp)

Esto te da una herramienta predictiva de verdad útil. Toma un programa cuyo enum `#[error_code]` propio tenga, digamos, tres variantes, y ninguna sobrescritura de `offset`. La primera variante es 6000, la segunda 6001, la tercera 6002. Si sabes la posición de una variante contada desde cero, sabes su número en el wire sin correr nada. El error clásico es ver los 2000 en un error decodificado y suponer que ahí viven los errores propios. No es así. Los 2000 son la banda de constraints del framework. Tus errores arrancan en 6000. Cuando quieres mover esa base, por ejemplo para dejar espacio o para calzar con una convención externa, `#[error_code(offset = N)]` la desplaza.

Esa es toda la superficie de errores: el framework es dueño de todo lo que está debajo de 6000, tú eres dueño de 6000 y para arriba, y la frontera es fija para que los dos nunca se puedan confundir.

### Dónde encajan los eventos, y dónde arranca el dispatcher

El tercer namespace, `event:`, merece su propio momento, porque el greeter que extiendes en el Lab de abajo emite uno y deberías saber exactamente qué pasa en el wire cuando lo hace. El handler `light_marquee` que estás por escribir llama a `emit!(MarqueeLit { plays })`. Esa macro serializa el struct del evento con **wincode**, el serializador de V2 (conociste el nombre en m01-l2 como el crate en el centro de la ruptura de dependencias #4937; esto es lo que hace de verdad), corriéndolo bajo una configuración de wire cuya salida es byte-idéntica a borsh. Después antepone el discriminador `event:MarqueeLit` de 8 bytes y escribe todo eso a través del syscall `sol_log_data`. No se guarda en una cuenta y no se devuelve a quien llama. Aterriza en los datos de log de la transacción, donde un indexador off-chain suscrito a tu programa puede leerlo, calzar los 8 bytes de adelante contra `sha256("event:MarqueeLit")[..8]`, y decodificar el resto como un `MarqueeLit`. El discriminador es lo que le deja al indexador distinguir tu `MarqueeLit` de cualquier otro evento que cualquier programa haya emitido en el mismo bloque.

Hay una variante de evento más rápida, `#[event(bytemuck)]`, que se saltea el serializador por completo y acomoda los campos como un struct Pod fijo, así decodificar es un cast en lugar de un parseo. Lo dejamos para el próximo módulo a propósito, porque solo tiene sentido una vez que conoces Pod y el layout zero-copy. Échale mano ahora y estarías wireando una herramienta cuyos cimientos todavía no colaste. Para esta lección, el `emit!` simple es exactamente lo correcto: muestra al namespace `event:` haciendo su trabajo sin arrastrar maquinaria que todavía no te enseñaron.

Lo que cierra el loop de vuelta a donde arrancó todo el capítulo, el dispatcher. Traza una llamada completa y cada namespace aparece en su lugar. Un cliente arma una transacción, antepone la etiqueta `global:light_marquee` de 8 bytes a los datos de la instrucción, y la manda. El dispatcher lee esos primeros 8 bytes, los calza contra el discriminador de instrucción de cada handler, y enruta a `light_marquee`. Después corre `try_accounts`: carga cada `Account<Marquee>` verificando su discriminador `account:`, corre los constraints, recorre la guarda de duplicados mutables. Solo entonces corre el cuerpo de tu handler, y cuando llama a `emit!`, sale el discriminador `event:` en el log. Tres namespaces, una invocación, cada uno haciendo el único trabajo para el que fue hasheado.

![Una línea temporal de seis paradas de una invocación, con el namespace global enrutando la instrucción, account: validando la cuenta cargada y event: etiquetando el log emitido.](assets/v07-timeline.webp)

## Lab: extiende R0 y observa la superficie

Todavía estás trabajando sobre R0, el greeter. Este Lab lo extiende con una segunda y una tercera instrucción y un struct de cuenta pequeño, puramente para que puedas observar, sobre código que escribiste tú, la carga generada, los constraints, el despacho, los namespaces de discriminador y el layout de errores. El estado real y probado llega el próximo módulo. Por ahora, el punto es ver la superficie.

Primero, el toolchain. Ya construiste la RC de V2 en m01-l2, y ahí aprendiste por qué `avm install` no puede traértela: no se cortó ningún GitHub Release para el tag v2, así que el binario precompilado que descarga da 404. La RC vive en el canal de git (`cargo install --git ... --tag v2.0.0-rc.1 anchor-cli --locked --force` — m01-l2 se construyó a partir de la rama `anchor-next` misma porque el canal era su tema; de aquí en adelante el curso fija el tag), y el binario ya está en tu máquina. Confirma que es el que responde antes de tocar código:

```bash
anchor --version   # the real check: expect anchor-cli 2.0.0-rc.1, not 1.1.2
which anchor       # ~/.cargo/bin/anchor either way — the avm shim lives at that same
                   # path, so this only proves something is on PATH, never which build
```

Si eso imprime `1.1.2`, tu PATH te entregó la línea estable otra vez; vuelve a leer la tabla de las cuatro paredes en m01-l2 y arregla el orden antes que nada. Mientras estás ahí, vuelve a estampar la fecha `verified` en la fila de anchor-cli de `PINS.md`. El pin es `2.0.0-rc.1` al momento de escribir esto (agosto de 2026), las RC se mueven, y una fecha fresca sobre un valor que no cambió es el registro de que un humano miró.

**Paso 1: abre el greeter y agrega el struct de cuenta.** En el `lib.rs` de tu programa, junto al handler `greet` de la lección pasada, agrega un tipo de cuenta diminuto y dos instrucciones nuevas. Este es el programa extendido completo:

```rust
use anchor_lang::prelude::*;

declare_id!("3ynNB373Q3VAzKp7m4x238po36hjAGFXFJB4ybN2iTyg");

#[program]
pub mod greeter {
    use super::*;

    // From m01-l3: the greeter. Logs and returns.
    pub fn greet(_ctx: &mut Context<Greet>) -> Result<()> {
        msg!("gm, a player just tapped in");
        Ok(())
    }

    // New: open a marquee account and set its play count.
    pub fn light_marquee(ctx: &mut Context<LightMarquee>, plays: u64) -> Result<()> {
        require!(plays > 0, BarcadeError::DeadMachine);
        ctx.accounts.marquee.plays = plays;
        emit!(MarqueeLit { plays });
        msg!("marquee lit at {} plays", plays);
        Ok(())
    }

    // New: bump two marquee accounts. Two mut Account<Marquee>, no unsafe(dup):
    // the duplicate-mutable guard is armed.
    pub fn tally_two(ctx: &mut Context<TallyTwo>) -> Result<()> {
        ctx.accounts.first.plays = ctx
            .accounts
            .first
            .plays
            .checked_add(1)
            .ok_or(BarcadeError::Overflow)?;
        ctx.accounts.second.plays = ctx
            .accounts
            .second
            .plays
            .checked_add(1)
            .ok_or(BarcadeError::Overflow)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Greet {
    pub player: Signer,
}

#[derive(Accounts)]
pub struct LightMarquee {
    #[account(mut)]
    pub payer: Signer,
    #[account(init, payer = payer)]
    pub marquee: Account<Marquee>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct TallyTwo {
    #[account(mut)]
    pub first: Account<Marquee>,
    #[account(mut)]
    pub second: Account<Marquee>,
}

#[account]
pub struct Marquee {
    pub plays: u64,
}

#[event]
pub struct MarqueeLit {
    pub plays: u64,
}

#[error_code]
pub enum BarcadeError {
    #[msg("a marquee cannot open on zero plays")]
    DeadMachine,
    #[msg("play counter overflowed")]
    Overflow,
}
```

En ese programa aparecen dos formas de escribir errores y vale la pena saber por qué las dos compilan, porque la segunda parece equivocada la primera vez que te la cruzas. `require!(cond, BarcadeError::DeadMachine)` toma la variante desnuda; la macro construye el error por ti. `.ok_or(BarcadeError::Overflow)?` le entrega a `ok_or` una variante desnuda también, produciendo un `Result<_, BarcadeError>`, y después el `?` lo convierte, porque `#[error_code]` genera el impl `From<BarcadeError>` hacia el tipo de error de Anchor. Dos formas, un enum, y este es el par que vas a ver en cada programa de este curso. La única vez que echas mano de otra cosa es cuando quieres la ubicación del código fuente estampada en el error, que es lo que agrega `error!(BarcadeError::Overflow)`.

Fíjate en la superficie V2 que estás mirando directo. Sin lifetimes `<'info>` en los structs de cuentas. Los handlers toman `&mut Context<T>`. `LightMarquee` usa `init` sin `space` explícito, porque V2 infiere el tamaño a partir del tipo de cuenta. Cada campo de cada derive es una carga de fase uno seguida de constraints de fase dos, exactamente el orden del diagrama.

![Un struct de cuentas LightMarquee anotado que etiqueta cada campo con el trabajo de carga y de constraints que genera, y el cuerpo del handler como la fase de despacho que corre al final.](assets/v08-annotated-code.webp)

**Paso 2: mira un discriminador con tus propios ojos.** No hace falta que adivines qué etiqueta escribe `init` al frente de una cuenta `Marquee`. Calcúlalo:

```bash
echo -n "account:Marquee" | shasum -a 256 | cut -c1-16
```

Esos son los 8 bytes exactos que `Account<Marquee>` verifica en cada carga. Haz lo mismo para `global:light_marquee` y `event:MarqueeLit` y ya derivaste a mano cada discriminador de tu propio programa. Este es el músculo que pide el Challenge.

**Paso 3: arma y observa la guarda de duplicados mutables.** La instrucción `tally_two` toma dos campos `Account<Marquee>` mutables sin `unsafe(dup)`. Eso quiere decir que la guarda está viva. Esta es una prueba que abre un marquee, después llama a `tally_two` pasando esa única cuenta a *los dos*, `first` y `second`, y afirma que la llamada se rechaza. Es la misma forma de Rust con LiteSVM que la prueba que reescribiste la lección pasada, con dos instrucciones en vez de una. Agrégala al lado de esa prueba:

```rust
use {
    anchor_lang::{
        programs::System, solana_program::instruction::Instruction, Id, InstructionData,
        ToAccountMetas,
    },
    anchor_v2_testing::{Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction},
};

#[test]
fn rejects_same_mut_account_twice_without_unsafe_dup() {
    let program_id = greeter::id();
    let payer = Keypair::new();
    let marquee = Keypair::new();
    let mut svm = anchor_v2_testing::svm();

    let bytes = include_bytes!("../../../target/deploy/greeter.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    // Open the marquee: light_marquee inits it with plays = 1.
    let init_ix = Instruction::new_with_bytes(
        program_id,
        &greeter::instruction::LightMarquee { plays: 1 }.data(),
        greeter::accounts::LightMarquee {
            payer: payer.pubkey(),
            marquee: marquee.pubkey(),
            system_program: System::id(),
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[init_ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer, &marquee]).unwrap();
    svm.send_transaction(tx).unwrap();

    // Now pass the SAME account into both mutable slots. No unsafe(dup): the guard is armed.
    let dup_ix = Instruction::new_with_bytes(
        program_id,
        &greeter::instruction::TallyTwo {}.data(),
        greeter::accounts::TallyTwo {
            first: marquee.pubkey(),
            second: marquee.pubkey(),
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[dup_ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_err(), "expected the duplicate-mutable guard to fire");
}
```

Córrela:

```bash
anchor test
```

La llamada a `tally_two` nunca llega a tu handler. El dispatcher recorre las vistas de cuenta, marca la dirección repetida, hace AND de ese bitvec contra `MUT_MASK`, ve un bit que sobrevive y devuelve `ConstraintDuplicateMutableAccount` en tiempo de ejecución, antes de que los dos campos siquiera carguen. Tu lógica de `checked_add` es irrelevante acá, porque la guarda se dispara antes del cuerpo. Ese es el Checkpoint: una cuenta mutable duplicada, pasada sin `unsafe(dup)`, es un rechazo en tiempo de ejecución desde el dispatcher. Deberías ver pasar la prueba porque el error fue lanzado, que es la guarda haciendo su trabajo.

![Un flujo de invocación de izquierda a derecha donde el dispatcher encuentra la misma dirección dos veces y rechaza con ConstraintDuplicateMutableAccount, así que no corre ni la carga de cuentas ni el cuerpo del handler.](assets/v09-flowchart.webp)

Si quieres demostrarte la salida a ti mismo, agrega `unsafe(dup)` a los dos campos de `TallyTwo` y vuelve a correr. Ahora la misma llamada se acepta, las dos escrituras apuntan a la misma cuenta, y el segundo `checked_add` ve el valor que escribió el primero. Ese es el aliasing del que V2 te protege por defecto, hecho visible a pedido.

## Challenge: construye la preimagen del discriminador

Esto es un problema Completion, y es el Challenge de código del módulo. Te dan una función que se supone que devuelve la cadena exacta de preimagen con namespace que Anchor hashea para cada tipo de ítem. El starter hace trampa: su mapeo `namespace` simplemente repite el tipo de ítem, así que la preimagen sale `instruction:increment` donde Anchor en realidad quiere `global:increment`, y el build falla en ese caso exacto.

```rust
// Anchor derives every 8-byte discriminator by hashing a NAMESPACED preimage:
// sha256("<namespace>:<Name>")[..8]. The bytes come later. The preimage STRING is
// the part you have to get right, and one of the three namespaces is a classic trap.
//
// `namespace` maps an item kind to the namespace Anchor actually hashes from:
//   - an account struct       -> "account"
//   - an instruction handler  -> "global"     <-- NOT "instruction"
//   - an event struct         -> "event"
const fn namespace<'a>(item_kind: &'a str) -> &'a str {
    // TODO: map each item_kind to its real Anchor namespace. Only one of the
    // three differs from its item kind -- that one is the whole exercise.
    item_kind
}

fn discriminator_preimage(item_kind: &str, name: &str) -> String {
    format!("{}:{name}", namespace(item_kind))
}
```

Tu trabajo es mapear los tres tipos de ítem a sus namespaces reales dentro de `namespace`. Dos de los tres se mapean a sí mismos. Uno no, y los criterios de aceptación de abajo te dicen cuál. El mapeo es un `const fn` a propósito — un dispositivo de tiempo de compilación que vas a volver a encontrar en el Challenge de constraints de m03-l3 — así que el bloque de verificación que viene debajo del starter corre en el compilador mismo: un mapeo sin arreglar no compila, y el mensaje de error nombra el namespace en el que te equivocaste. (Una consecuencia de `const fn`: `match` no puede comparar `&str` directamente ahí, porque la igualdad de cadenas es una llamada a trait y las llamadas a trait no son `const` en el Rust estable — haz el match sobre `item_kind.as_bytes()` con patrones de byte-string como `b"instruction"` en su lugar.)

Los criterios de aceptación son exactos:

- `discriminator_preimage("account", "CabinetCounter")` devuelve `account:CabinetCounter`
- `discriminator_preimage("instruction", "increment")` devuelve `global:increment`
- `discriminator_preimage("event", "HighScore")` devuelve `event:HighScore`
- `discriminator_preimage("account", "HighScore")` devuelve `account:HighScore` — el mismo nombre que el caso de arriba, namespace distinto, porque el prefijo es una función del *tipo*
- `discriminator_preimage("instruction", "initialize")` devuelve `global:initialize`

Los cinco casos de arriba son los vectores que vienen con el Challenge, y documentan el contrato — pero la calificación de producción para Rust verifica que tu código *compile*, y las aserciones de tiempo de compilación debajo del starter son las que lo hacen cumplir: el build mismo falla hasta que el mapeo `instruction -> global` esté bien. Los últimos dos vectores están ahí a propósito: hacen fallar una búsqueda indexada por el *nombre*, que es el atajo que de otro modo pasa los primeros tres — un atajo que la división `namespace` también cierra estructuralmente, ya que el mapeo jamás ve el nombre. Es una función simple sin framework en el medio, así que si prefieres trabajar localmente, métela en cualquier crate de borrador y hazla andar desde un `#[test]`:

```bash
cargo test
```

El mapeo sin arreglar del starter falla el build en el caso instruction; tu solución compila limpio y pasa los cinco vectores.

**Solo, sin apoyo, dos partes.** Primero, agrega una tercera variante al enum `BarcadeError` de tu programa y, antes de correr nada, escribe el número exacto en el wire que esperas que vea un cliente cuando se dispare. La sección del layout de errores tiene todo lo que necesitas para derivarlo. Después dispáralo y verifícate contra el wire. Segundo, razónalo en una o dos oraciones propias: ¿por qué escribir `dup` a secas es un error de compilación, mientras que pasar la misma cuenta mutable dos veces es un rechazo en tiempo de ejecución? Dos eventos distintos en dos momentos distintos, y nombrar qué sabe cada uno es todo el ejercicio. No hay respuesta acá; si tu oración se sostiene cuando vuelves a leer la sección de `MUT_MASK`, se sostiene.

## Dónde te deja esto

El criterio de esta lección es pequeño y concreto. Completa el Challenge de la preimagen del discriminador, el starter fallando, la solución pasando, y enuncia en una oración en qué se diferencian el `MUT_MASK`, que es de tiempo de compilación, y la verificación que el dispatcher hace en tiempo de ejecución. Si tu 6002 predicho calzó con el wire y tu oración se sostiene, terminaste acá.

Ahora puedes leer la superficie completa de cualquier programa V2, tanto el lado del programa que abriste la lección pasada como el lado de las cuentas que abriste hoy: el orden generado carga-constraints-despacho, la guarda de duplicados mutables sobre su costura de máscara-en-compilación-más-verificación-en-ejecución, los tres namespaces de discriminador con `global:` como el que muerde, y el layout de errores con el framework debajo de 6000 y tu código ahí mismo y para arriba. Ese es todo el wireado de seguridad de un programa V2, y ya nada de eso es magia para ti.

El próximo módulo le das estado real a las ideas del greeter. El `Account<Marquee>` que escribiste hoy ya era zero-copy, porque en V2 eso es simplemente lo que `Account<T>` es; un solo `u64` resulta ser Pod-legal sin que lo pienses, que es exactamente por qué la disciplina quedó invisible. El próximo módulo deja de ser invisible. Conoces las reglas de layout que `Account<T>` ha estado aplicando en silencio todo este tiempo, descubres qué pasa la primera vez que intentas poner algo en un campo que no es de bytes planos, y ves por qué `#[event(bytemuck)]` solo tiene sentido una vez que puedes leer un struct como bytes. Después construyes R1, el cabinet-counter: el primer peldaño con datos que vale la pena probar. Ya leíste la superficie. Ahora la haces guardar algo.
