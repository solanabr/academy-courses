# Anatomía del programa: en qué se expanden declare_id! y #[program]

La lección pasada peleaste con la instalación, fijaste el toolchain RC en tu archivo central de pins, y desplegaste R0 a devnet. Compila, se despliega, abre una cuenta. Y sigue siendo una caja negra: dos macros, `declare_id!` y `#[program]`, dos docenas de líneas de Rust, y ninguna idea real de en qué se convierten.

Déjame arreglar eso ahora mismo, antes de cualquier teoría. Desde tu workspace de greeter, corre la prueba que el scaffold ya escribió:

```bash
anchor test
```

Ese comando construye el programa y corre la prueba de Rust que `anchor init` dejó al lado, la que inicializa el counter del scaffold y afirma sus campos. Mira en la cola de la salida el `test result: ok. 1 passed`. Acabas de invocar tu propio handler en una VM en proceso, y la razón de que esa prueba sea Rust y no TypeScript es lo primero que vale la pena entender hoy. Guárdate esa idea. Vamos a leer exactamente qué generan esas dos macros, y después cerrar a mano el loop de invocación sobre un programa que habrás pelado hasta el hueso.

## Resumen

Esta es la ruta. Ya sabes que el greeter compila y se despliega. Lo que todavía no sabes es la *forma* en la que compila: la firma del handler, los tipos de cuenta, el tipo de dirección, el camino de despacho desde los bytes crudos hasta tu código. Así que vamos hacia atrás, desde el código que escribiste hasta la superficie que generan las macros, en el orden en que el framework realmente la usa. Después llevas esa superficie a un Lab: reescribes la prueba LiteSVM del scaffold alrededor de tu handler pelado, llenas el único TODO que tiene para que llegue a `greet`, y confirmas desde el CLI que la copia que desplegaste la lección pasada sigue viva en el id de tu archivo de pins.

Una advertencia por adelantado, y es la honesta. Esto es Anchor V2, una RC. La enorme mayoría del código, los tutoriales y las respuestas de Stack Overflow con las que te vas a topar todavía importan `@coral-xyz/anchor` y la forma de v1. Todo lo que sigue es la expansión de V2, no la de v1. Donde difieren, te muestro las dos, porque leer el ecosistema te va a costar un impuesto de traducción y más vale que aprendas el tipo de cambio desde ahora.

## Leer la expansión

Empieza con el greeter completo en una sola pantalla. Esto es el código fuente, nada generado todavía:

```rust
use anchor_lang::prelude::*;

declare_id!("3ynNB373Q3VAzKp7m4x238po36hjAGFXFJB4ybN2iTyg");

#[program]
pub mod greeter {
    use super::*;

    pub fn greet(_ctx: &mut Context<Greet>) -> Result<()> {
        msg!("gm, a player just tapped in");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Greet {
    pub player: Signer,
}
```

Tu scaffold de la lección pasada sigue siendo el counter generado: un handler `initialize` que abre un `Account<Counter>`, más su módulo `state`. Reemplaza ahora tanto el handler como el struct `Initialize` por esta forma `greet`, y borra el módulo `state` y su línea `use state::Counter;` junto con ellos. Quieres el programa más chico que todavía demuestre algo: un firmante, una línea de log, ningún estado. Cambia también el `declare_id!` por tu propio id de programa; el de arriba es un sustituto.

En el momento en que borras `initialize`, el workspace deja de compilar, y vale la pena saber por qué antes de que veas el error. La prueba del scaffold en `programs/greeter/tests/test_initialize.rs` nombra `greeter::instruction::Initialize` y `greeter::accounts::Initialize`, y las dos se generan *a partir* del handler y del struct que acabas de sacar. Esos símbolos ya no existen, así que la prueba no compila. Esa prueba no es algo que recortas; es algo que reescribes, y el Lab de abajo la reescribe. Haz la edición del programa y la de la prueba juntas, después reconstruye, para que el `.so` y los constructores generados coincidan con lo que vas a leer enseguida.

Una cosa que esa línea de log *no* hace es imprimir la dirección del player, y la razón es una restricción genuina de V2 con la que conviene encontrarse temprano y no como un error de compilación misterioso. `Address` solo implementa `Display` y `Debug` cuando el feature `decode` del crate `solana-address` está activado, porque la codificación base58 es exactamente el tipo de peso que un framework `no_std` se niega a cargar por defecto, y anchor-lang no lo activa. Así que `msg!("gm, {}", ctx.accounts.player.address())` no compila en un programa V2 de fábrica. Sé preciso con lo que falta ahí: `msg!` formatea bien, y `msg!("lit at {} plays", plays)` sobre un `u64` es código V2 ordinario que vas a escribir la lección que viene. El hueco es `Display` sobre `Address` específicamente, así que son las direcciones las que no puedes interpolar, no los valores en general. Si de verdad necesitas una dirección base58 en un log, activas ese feature a propósito (el feature `compat` de anchor-lang lo trae consigo, junto con una macro `debug!`) y lo pagas en tamaño del binario y en unidades de cómputo. El valor por defecto es el silencio sobre las direcciones, y el valor por defecto es el punto.

![Una tabla de cuatro filas que muestra que los literales y los valores u64 formatean bien en msg! mientras que un Address no, hasta que el feature compat se activa a propósito, a costa de tamaño y de cómputo.](assets/v01-table.webp)

La lección pasada nombró los cambios de superficie al pasar y prometió que los ibas a abrir acá. Esto es lo prometido. Tres de ellos son visibles en las once líneas de arriba: `&mut Context`, no `Context`. `Signer`, no `Signer<'info>`. Y ningún `Pubkey` en ninguna parte. El cuarto venía montado en la línea del scaffold que estás reemplazando, `ctx.accounts.counter.authority = *ctx.accounts.payer.address();`: `.address()`, no `.key()`. Nombrarlos era el trabajo de la lección pasada. Derivar por qué cada uno tiene esa forma, y qué genera la macro a su alrededor, es el de esta. Tómalos uno por uno.

### declare_id! todavía solo declara la dirección

`declare_id!` es la tranquila. Toma la dirección base58 del programa y genera, más o menos, esto:

```rust
use anchor_lang::prelude::*;

// what declare_id! expands to, in spirit. The real macro decodes your base58
// string into these 32 bytes at compile time; zeros stand in for them here.
pub static ID: Address = Address::new_from_array([0u8; 32]);
pub fn id() -> Address { ID }
pub fn check_id(id: &Address) -> bool { id == &ID }
```

Una constante `ID`, un accesor `id()` verificado, y un helper `check_id`, para que el resto del programa y el punto de entrada puedan comparar la dirección con la que fue invocado contra la dirección para la que fue compilado. Ese trabajo no cambió entre v1 y V2, y tampoco cambió el nombre de la macro. Lo único que sí cambió va montado calladamente en los tipos de arriba: la constante es un `Address`, no un `Pubkey`. Guárdate eso, volvemos a eso dos secciones más abajo.

El valor en el tuyo es lo que sea que `anchor keys sync` escribió después de tu deploy de m01-l2; el de arriba es un sustituto. Mete el id de programa que tienes en tu archivo de pins cuando sigas los pasos, o el `.so` construido va a cargar la dirección equivocada y cada invocación va a rebotar en la verificación del id.

La verificación del id es lo primerísimo que hace el punto de entrada generado en cada llamada: `check_id` del id de programa declarado contra el id de programa de entrada, rechazar si difieren. Tenlo en mente, porque es el paso uno del camino de despacho al que llegamos en un rato. También es la razón por la que un `.so` construido con el `declare_id!` de otra persona llega muerto: el programa se niega a correr como una dirección para la que no fue compilado.

### #[program]: el handler, y el &mut que te sorprende

`#[program]` marca el módulo que guarda tus handlers de instrucción. Cada función pública adentro se vuelve una instrucción a la que el programa puede despachar. Eso también es v1. Lo que V2 cambió es la firma del handler, y es el cambio con el que vas a tropezar primero.

En v1 un handler tomaba su contexto *por valor*: `pub fn greet(ctx: Context<Greet>)`. En V2 toma una *referencia mutable*: `pub fn greet(ctx: &mut Context<Greet>)`. Todo handler, de manera uniforme, incluso uno como `greet` que solo lee. El framework construye el `Context`, te entrega un `&mut` a él, y lo reutiliza a lo largo de la rutina de salida que escribe de vuelta tus cambios de cuenta. No lo construyes y no lo devuelves. Lo tomas prestado, mutas a través de él, devuelves `Ok(())`.

![La misma instrucción greet en v1 y V2, marcando cuatro cambios: una referencia mutable a Context, .address() reemplazando a .key(), Address reemplazando a Pubkey, y los lifetimes info eliminados.](assets/v02-annotated-code.webp)

¿Por qué `&mut` siquiera, si `greet` nunca escribe? La respuesta honesta es que el contexto por valor era un costo chico que se pagaba en cada instrucción: el framework movía un contexto hacia adentro de tu función, hacías tu trabajo, volvía a mover el estado de la cuenta hacia afuera. Una referencia elimina el movimiento y deja que el mismo contexto atraviese la validación, tu handler y la rutina de salida como una sola cosa prestada. Es una ganancia chica en ergonomía y en costo, y encaja con toda la tesis de V2 desde la lección uno: deja de copiar lo que puedes tomar prestado o castear en el lugar. No tienes que amar la sintaxis. Sí tienes que reconocerla, porque un handler escrito `Context<T>` por valor no va a compilar contra la RC, y el mensaje de error apunta a la firma, no a la causa.

### Pubkey ahora es Address, y .key() ahora es .address()

Mira otra vez la línea del scaffold que acabas de borrar: `ctx.accounts.counter.authority = *ctx.accounts.payer.address();`. En v1 esa línea habría dicho `ctx.accounts.payer.key()` y el tipo habría sido `Pubkey`. En V2 cambiaron los dos nombres. El tipo es `Address` (del crate `solana-address` que trae pinocchio), y el accesor es `.address()`, que te entrega una referencia, de ahí el `*` adelante cuando quieres una copia propia para guardar.

Esto es un rename directo de dos cosas que usas todo el tiempo, y es exactamente por eso que muerde. Cada clave de cuenta que lees, cada dirección que comparas, cada pubkey que guardaste en un campo de struct: `Pubkey` se vuelve `Address`, `.key()` se vuelve `.address()`. No hay `.pubkey()` ni `.to_address()` en la superficie de los wrappers; el accesor es `.address()`, y punto. Tus dedos van a tipear `.key()` por pura memoria muscular las primeras tres veces. Los míos lo hicieron. El compilador te va a detener todas las veces, así que es un hábito barato de romper, pero es un hábito.

El rename no es puro cambio cosmético, y vale la pena entender de dónde viene para que deje de sentirse arbitrario. V2 es una reescritura no_std construida sobre pinocchio, y pinocchio trae su propio tipo de dirección a través del crate `solana-address` en vez del más viejo `solana-program::Pubkey`. Así que cuando Anchor V2 se apoya en ese cimiento, el tipo que te entrega arriba es el que habla el cimiento: `Address`. El paso de `.key()` a `.address()` es el accesor siguiendo al tipo. Léelo así y el patrón se generaliza: la mayor parte de lo que parece nuevo en una firma de V2 es el cimiento de pinocchio asomando a través del framework en vez de quedar tapado. Esa es la misma tesis de la lección uno, vista desde el lado de los tipos en vez del lado del cómputo.

![Una tabla de cinco filas que mapea v1 a V2, cubriendo la firma del handler, Pubkey a Address, .key() a .address(), el lifetime eliminado del wrapper de cuenta, y la lectura del bump que es idéntica en los dos lados porque el mapa de cadenas murió en 0.29.](assets/v03-comparison.webp)

### Los lifetimes <'info> desaparecieron de los wrappers

Ahora el struct de cuentas. En v1 cada wrapper de cuenta cargaba un lifetime explícito (su tiempo de vida) y el struct también: `pub struct Greet<'info>` con `pub player: Signer<'info>` adentro. Ese `<'info>` se enhebraba por todo y era lo que los principiantes hacían mal más que cualquier otra cosa, porque el borrow checker (el verificador de préstamos) tenía opiniones al respecto y los mensajes de error eran densos.

En V2 la superficie de los wrappers no carga ningún `<'info>`. Tu struct es `pub struct Greet`, tu campo es `pub player: Signer`. El lifetime no se movió al módulo `#[program]` ni se escondió en otra parte. Desapareció de la superficie que escribes. El framework todavía rastrea internamente los lifetimes de los datos de cuenta que están debajo, pero la contabilidad a nivel de tipos que antes escribías a mano ahora es problema de la macro, no tuyo. Para alguien que viene de v1, este es el cambio que hace que un struct de cuentas de V2 se vea casi demasiado limpio, como si faltara algo. No falta nada. Siempre fue trabajo del framework; V2 por fin dejó de hacerte tipearlo.

Hay un quinto rename que no vas a ver en el greeter porque todavía no tiene PDAs, pero pertenece a la misma lista para que no te embosque más adelante: el Anchor más viejo leía un bump guardado con `ctx.bumps.get("vault")`, una búsqueda en un mapa que devolvía un `Option`. La línea 0.2x-a-0.3x ya movió eso a acceso por campo, y V2 lo mantiene: `ctx.bumps.vault`. Es la única fila de esa tabla que no es un cambio de V2, y está ahí porque alguien con memoria muscular vieja todavía va a echar mano del mapa. Aterriza en el momento en que escribes tu primera cuenta con seeds, que es el módulo de PDAs, a dos módulos de acá.

### Cómo llegan realmente los bytes crudos a greet

Ya leíste las piezas. Ahora míralas correr, porque "la macro genera un punto de entrada" no es una explicación hasta que puedas trazar una invocación a través de ella. Cuando alguien invoca tu programa, el código generado hace cuatro cosas en orden.

Primero, el punto de entrada verifica que el id de programa declarado (el de `declare_id!`) coincida con el id de programa con el que realmente fue invocado, y lanza error si no. Segundo, lee el frente de los datos de instrucción y lo compara contra el discriminador de cada handler, la etiqueta chica que dice "esta llamada es para `greet`, no para algún otro handler." Tercero, el wrapper del handler que coincidió deserializa las cuentas nombradas en la transacción hacia tu struct `Greet`, corriendo cada constraint y cada verificación a medida que avanza, y construye el `Context`. Cuarto, llama al código que realmente escribiste, `greet`, con un `&mut` a ese contexto, y después corre una rutina de salida que escribe de vuelta cualquier cambio de cuenta.

![Un flujo de seis pasos desde el punto de entrada hasta la salida: verificación del id de programa, coincidencia del discriminador, deserialización de cuentas hacia un Context, el cuerpo del handler greet, y después la rutina de salida que escribe de vuelta los cambios.](assets/v04-flowchart.webp)

Todo menos el cuerpo del handler es generado: la verificación del id, el despacho, la deserialización, y la rutina de salida que escribe de vuelta los cambios. El único código que escribiste es el cuerpo de `greet`. Ese es todo el canje de un framework: escribe el despacho, la deserialización, las verificaciones de constraints, y la persistencia, y a cambio esconde cableado sobre el que igual tienes que poder razonar cuando algo sale mal. Leer la expansión es cómo te quedas con el razonamiento aunque la macro se quede con el tipeo.

Un detalle del paso dos se gana un nombre ahora y un tratamiento completo la lección que viene. La etiqueta con la que el dispatcher hace la coincidencia es un discriminador: los bytes iniciales de tus datos de instrucción que dicen para qué handler es esta llamada. Anchor lo calcula a partir del nombre del handler, así que `greet` y `greet_high_score` reciben discriminadores distintos y nunca chocan, y el constructor generado `greeter::instruction::Greet` que vas a usar en el Lab te estampa el correcto sobre los bytes. Eso es todo lo que necesitas hoy: entran los bytes, el dispatcher lee el discriminador, corre el wrapper que coincide. Cómo se deriva realmente el discriminador, por qué un handler de instrucción se hashea desde un namespace que sorprende a la gente, y dónde viven los códigos de error personalizados, es todo el contenido de la próxima lección sobre `#[derive(Accounts)]`. Un concepto por lección; esta es la del lado `#[program]`.

## Lab: cierra el loop de invocación

Hora de hacer correr el handler. El vehículo es la prueba de Rust con LiteSVM que `anchor init` ya generó como scaffold, reescrita alrededor del programa que acabas de pelar. El recorrido acá está completamente resuelto; lo sigues exactamente. Las dos cosas que llenas tú mismo vienen en el Challenge, justo después.

Una nota de alcance, para que no te quedes esperando un zapato que no va a caer. Invocar tu programa desplegado desde la línea de comandos no es algo que el CLI `solana` a secas pueda hacer: no puede calcular un discriminador de Anchor ni empacar account metas, así que no hay conjuro de `solana` que llame a `greet`. Construir un cliente que sí pueda es todo el trabajo del módulo 8, una vez que tengas un cliente generado del cual construirlo. Hoy el loop de invocación se cierra en LiteSVM, y el deploy a devnet queda confirmado como desplegado, no como llamado.

Un paréntesis rápido sobre por qué la prueba por defecto es Rust y no TypeScript, ya que llevas un rato mirándola. `anchor init` en V2 ofrece cinco plantillas de prueba: Mocha, Jest, Rust, Mollusk y Litesvm. Litesvm es el `#[default]`. Así que el scaffold te entrega una prueba de integración en Rust que carga tu `.so` compilado en una VM en proceso y lo invoca, sin TypeScript escrito y sin validador local levantado.

![Una tabla con las cinco plantillas de prueba de anchor init, que separa las opciones de TypeScript-contra-un-validador de las de Rust en proceso, con Litesvm marcada como la opción por defecto.](assets/v05-comparison.webp)

¿Por qué ese valor por defecto, y no uno de TypeScript como todos los tutoriales de v1 que leíste? Porque en V2 la superficie de Rust es el cliente principal, y todavía no hay ningún paquete oficial de TypeScript para V2 al que echar mano. Pero la elección también es una apuesta, y Jacob Creech la nombró en su memo de unificación: "I expect Anchor V2 to unify tools around using Litesvm, using the solana-verify standard, potentially surfpool, Gill." El loop de invocación con LiteSVM por defecto que estás por correr es ese memo entregado. Rima con los incrementos fechados de Anchor 1.0 que narraste la lección pasada, la misma ola de decisiones que renombró el paquete de TypeScript y cambió el validador por defecto.

![Cinco incrementos de Anchor 1.0 mostrados como un solo conjunto: el rename del paquete, CpiContext tomando un Pubkey, transfer_checked, LiteSVM como plantilla de prueba por defecto, y Surfpool como validador por defecto.](assets/v06-timeline.webp)

También es la razón por la que todo este módulo cierra el loop de deploy e invocación en Rust en vez de entregarte un harness de TypeScript que todavía no existe.

**Paso 1. Reescribe la prueba del scaffold.** La plantilla de LiteSVM escribe su prueba al lado del crate del programa, no en la raíz del workspace: abre `programs/greeter/tests/` y busca `test_initialize.rs`. Renombra el archivo a `test_greet.rs` y reemplaza su cuerpo con la versión de abajo. Esto es una reescritura, no un recorte: cada símbolo `Initialize` del archivo viejo se generó a partir del handler que acabas de borrar, así que nada de lo que hay adentro sobrevive al reemplazo. Se lee así:

```rust
use {
    anchor_lang::{
        solana_program::instruction::Instruction, InstructionData, ToAccountMetas,
    },
    anchor_v2_testing::{Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction},
};

#[test]
fn test_greet() {
    let program_id = greeter::id();
    let payer = Keypair::new();
    let mut svm = anchor_v2_testing::svm();

    let bytes = include_bytes!("../../../target/deploy/greeter.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let ix = Instruction::new_with_bytes(
        program_id,
        &greeter::instruction::Greet {}.data(),
        greeter::accounts::Greet {
            // TODO: name the accounts greet needs
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "send_transaction failed: {:?}", res);
}
```

Una cosa en ese bloque de imports merece una nota antes de que leas el cuerpo, porque parece contradecir todo lo de arriba. La prueba echa mano de `anchor_lang::solana_program::instruction::Instruction` y llama a `payer.pubkey()`, exactamente los dos nombres que esta lección te acaba de decir que V2 reemplazó. Las dos son correctas. Los renames son un cambio *del lado del programa*: adentro de tu programa, sobre los wrappers que genera el derive, el tipo es `Address` y el accesor es `.address()`. El código off-chain, que es lo que una prueba es, todavía habla los tipos `solana-program` / `solana-sdk` que el runtime y el SDK usaron siempre, así que un `Keypair` todavía te entrega un `Pubkey` desde `.pubkey()`. Estás mirando la costura entre las dos superficies, y se queda ahí durante todo el curso: on-chain es `Address`, off-chain es `Pubkey`.

Ahora lee lo que hace la prueba contra el flujo de despacho que acabas de trazar. `anchor_v2_testing::svm()` levanta la VM en proceso. `add_program` inserta tu `.so` construido en su id de programa. `greeter::instruction::Greet {}.data()` produce los bytes de instrucción, discriminador incluido, con los que el paso dos hace la coincidencia. `greeter::accounts::Greet { ... }.to_account_metas(None)` produce los account metas que el wrapper deserializa en el paso tres. Estos módulos `instruction::` y `accounts::` los generan las mismas macros a partir de tu programa; la prueba no lee un IDL en runtime, usa los constructores tipados directamente.

![Un solo greeter.so construido desde lib.rs alimenta dos demostraciones, una corrida local en proceso del handler en LiteSVM y una cuenta de devnet que resuelve como ejecutable.](assets/v07-diagram.webp)

**Paso 2. Construye y corre, y espera rojo.** Desde la raíz del workspace:

```bash
anchor test
```

`anchor test` construye el programa, corre la prueba de LiteSVM, y se salta el arranque de un validador porque la VM en proceso no necesita ninguno. Debería **fallar en tiempo de compilación**, en el archivo de prueba, con `error[E0063]: missing field 'player' in initializer of 'Greet'` — el constructor `accounts::Greet` sigue siendo un TODO vacío, y un literal de struct en Rust tiene que nombrar todos los campos, así que el compilador te detiene antes de que se mande nada. Ese rojo es el estado correcto en el que estar ahora mismo. Se queda rojo hasta que llenes el TODO en el Challenge, y ponerlo en verde es la mitad calificada de esta lección. Lo que quieres confirmar en este paso es más angosto: el crate del programa en sí compila después de la reescritura, y el único error que queda nombra exactamente el campo que estás por llenar — no un símbolo faltante por los renames de V2.

**Paso 3. Confirma que el deploy sigue vivo.** La prueba de LiteSVM es donde corre `greet`. Aparte, demuestra que la copia que subiste en m01-l2 sigue ahí y sigue siendo tuya. Apunta el CLI al id de programa en devnet que tienes en tu archivo de pins:

```bash
solana program show <YOUR_GREETER_PROGRAM_ID> --url devnet
```

Eso imprime la cuenta del programa, su largo de datos, y su autoridad de upgrade. Tres hechos que vale la pena leer en vez de hojear: la cuenta existe, es ejecutable, y la autoridad de upgrade es tu billetera, que es lo que te deja volver a desplegar encima más adelante. Fíjate en lo que esto *no* hace. No llama a `greet`, y nada en esta lección lo hace, en devnet. Desplegado e invocable son dos afirmaciones distintas, y hoy demuestras la primera desde el CLI y la segunda en LiteSVM.

## Challenge

Dos tareas. La primera es una completion, la segunda corre por tu cuenta. Acá es donde se empiezan a quitar las rueditas, una rueda a la vez.

**Completion, el TODO.** En `test_greet.rs`, llena el constructor `accounts::Greet { ... }` con las cuentas que `greet` realmente necesita. Mira tu `#[derive(Accounts)] pub struct Greet` para la respuesta: tiene un solo campo, `player: Signer`, y el firmante es el `payer` de la prueba. Una línea. Después `anchor test` debería convertir la falla anterior en `1 passed`. Si sigues en rojo, vuelve a leer el nombre del campo en tu struct de cuentas contra la clave que pusiste en el constructor; un desajuste ahí es toda la falla.

**Solo, la segunda variante.** Agrega una segunda instrucción de saludo al módulo `#[program]`, reusando las mismas cuentas `Greet`, y demuestra que despacha. La forma a la que apuntar:

```rust
pub fn greet_high_score(_ctx: &mut Context<Greet>) -> Result<()> {
    msg!("someone is going for the high score");
    Ok(())
}
```

Reconstruye, después escribe un segundo `#[test]` al lado del primero que mande `greet_high_score` en vez de `greet`. Ninguna pista más allá de lo que el Paso 1 ya te mostró: las únicas dos líneas que cambian son el constructor de la instrucción y el nombre de la prueba, y descifrar en qué constructor aparece la instrucción nueva es el punto. Si puedes leer el flujo de despacho, sabes por qué una segunda función pública en el módulo se vuelve una segunda instrucción despachable con su propio discriminador.

Aceptación: `anchor test` reporta `test result: ok. 2 passed`.

## Checkpoint

Terminaste cuando tres cosas son verdad al mismo tiempo: el workspace compila después de la reescritura, `anchor test` reporta las dos pruebas de saludo pasando, y `solana program show` resuelve tu greeter en devnet como una cuenta ejecutable de la que tienes la autoridad de upgrade. Si falta cualquiera de esas, el loop no está cerrado y la próxima lección se va a sentir como si se hubiera saltado un paso.

Vale la pena nombrar lo que cambió en tu cabeza, no solo en tu terminal. El greeter es apenas una docena de líneas, más chico que el scaffold con el que empezaste la hora, y ya no es una caja negra. Puedes trazar una invocación desde los bytes crudos, pasando por la verificación del id, la coincidencia del discriminador, el wrapper, tu handler, y la rutina de salida, y puedes nombrar cada rename de V2 que te cruces leyendo el programa de otra persona: `&mut Context<T>`, `Address` y `.address()`, sin `<'info>`, `ctx.bumps.name`. Un último recordatorio, porque te va a ahorrar una tarde: el ecosistema sigue siendo abrumadoramente v1. Cuando un tutorial escrito el año pasado muestra `Context<T>` por valor y `.key()`, ya sabes que no está mal, es solo la otra línea. Incluso las fuentes cuidadosas se atrasan acá. Helius, un referente de calidad en documentación, todavía entrega su introducción estrella a Anchor con una instalación de 0.29.0 en 2026. Leer tutoriales viejos de anatomía te engaña precisamente porque la anatomía se movió.

Ahora puedes leer el lado `#[program]` de cualquier programa V2. Pero el lado de las cuentas, `#[derive(Accounts)]`, es donde viven de verdad los constraints, el despacho, los discriminadores y los errores, y esconde el más filoso de todos los detalles de V2: una máscara de tiempo de compilación aplicada por una verificación en runtime. Eso es lo que viene, y es lo más filoso de este módulo.
