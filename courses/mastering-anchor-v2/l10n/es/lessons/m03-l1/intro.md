# PDAs y bumps canónicos en V2

La lección pasada terminaste el modelo de estado: los campos fijos y acotados casteados directo desde bytes como Pod, los genuinamente no acotados aislados detrás de `BorshAccount<T>`. Puedes describir cualquier cuenta que este arcade necesite. Lo que todavía no puedes hacer es decir cómo el runtime *encuentra* una, ni quién tiene derecho a tocarla.

Ya usaste la respuesta sin que te dijeran su nombre. El `Cabinet` que construiste al comienzo del módulo 2 llevaba `seeds = [b"cabinet", player.address().as_ref()]` y un `bump` desnudo, y te dije de frente que copiaras esas dos líneas y esperaras. Para el final de ese módulo al cabinet le había crecido un `authority` y un `bump` guardado y los seeds habían pasado a `cabinet.authority`, que también escribiste por confianza. Todo pasó como un encantamiento: lo escribes, y la cuenta aparece en una dirección que nunca elegiste. Hoy el encantamiento se vuelve un mecanismo sobre el que puedes razonar — y vas a ver con precisión qué tajada de él cambió V2 en realidad, que es una tajada más delgada de lo que afirman los blog posts del ecosistema.

El arcade necesita la versión completa ahora. Los jugadores están por prepagar créditos, y esos créditos se sientan en un vault por jugador. No puedes entregarle a cada jugador un par de claves para su propio vault, porque entonces lo controla el *jugador*, no el programa. Y no puedes mantener una tabla de "jugador -> dirección del vault" en algún lado, porque esa tabla es una cosa más que corromper, migrar y por la que pagar rent. Lo que quieres es una dirección que el programa vuelva a derivar desde cero, a solas, a partir de la clave del jugador, cada vez, para siempre.

Esa es toda la razón por la que existen las direcciones derivadas de programa. Antes del por qué, confirma que el toolchain que va a estar generando tus bumps es el de V2. Esto lo instalaste allá en m01-l2, así que esta es una verificación de re-pin, no una instalación desde cero:

```bash
anchor --version   # must report a 2.0.0 RC line, not 1.x
# If it reports 1.x or nothing, re-pin. avm (the Anchor Version Manager) cannot
# install the V2 RC: it downloads a prebuilt binary from a published GitHub
# release, and no release was cut for the v2 tag, so the fetch 404s;
# so the documented channel is a cargo git install, pinned to the RC's tag:
cargo install --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1 anchor-cli --locked --force
# macOS, if the build trips on LTO: prefix that line with CARGO_PROFILE_RELEASE_LTO=off
```

Una nota de frescura antes de que te apoyes en ese pin: Anchor V2 está en `2.0.0-rc.1` mientras escribo esto, publicado en crates.io el 2026-08-12. Es una *release candidate*, lo que quiere decir que la API todavía puede moverse entre RCs. Fíjate en lo que quiere decir "pin" acá, porque las dos mitades vienen de lugares distintos a propósito. El CLI es una instalación de git fijada a `v2.0.0-rc.1`, un tag, que es un punto fijo en vez de una punta de branch que se mueve; m01-l2 instaló desde la branch `anchor-next` misma porque el canal era el tema de esa lección, y todo lo que viene después fija el tag. La librería en el `Cargo.toml` que escribes en el paso 1 es la versión de crates.io, `anchor-lang = "2.0.0-rc.1"`, que es más fuerte todavía, porque el registry prohíbe republicar una versión. Si quieres nombrar el toolchain con más precisión todavía, el tag resuelve al commit `e4878b6d`, y el Dockerfile de verify de m08-l2 fija exactamente ese. De cualquier manera, vuelve a revisar el tag antes de arrancar una sesión. El MSRV es Rust 1.89.0, así que asegúrate de que tu `rustc` también lo cumpla.

## Resumen

Aquí está la lección entera como un índice de hallazgos, cada línea una conclusión sobre la que puedes actuar:

- Un PDA es una cuenta propiedad del programa cuya dirección se deriva de forma determinista a partir de **los seeds más el ID del programa**, y que cae deliberadamente **fuera** de la curva Ed25519 para que ninguna clave privada pueda firmar nunca por ella.
- El **bump** es el byte extra que empuja fuera de la curva un punto que estaría sobre ella. El bump **canónico** es el primero (buscando hacia abajo desde 255) que cae fuera de la curva. Hay exactamente uno, y no deberías usar nunca ningún otro.
- El bump canónico te llega a través de un **struct `bumps` tipado que la macro genera en tiempo de expansión**, leído como `ctx.bumps.vault` — y eso es *línea base*, no un delta de V2: Anchor 0.29 reemplazó hace dos majors el antiguo mapa indexado por cadena `ctx.bumps.get("vault").unwrap()`, y la línea 1.x lee `ctx.bumps.vault` exactamente como lo hace V2. La novedad real de V2 sobre los bumps es más estrecha: cuando cada seed es un literal de tiempo de compilación, el bump canónico se precalcula en tiempo de macro como una const. Los seeds que incluyen un valor de tiempo de ejecución, como la clave del jugador de este vault, siguen derivándose durante la validación en cualquiera de las dos líneas.
- Para un PDA propiedad del programa, V2 **se salta la verificación de curva**, un ahorro reportado por el proyecto de alrededor de 1,000 CU por verify (changelog de Anchor V2). Vuelve a medirlo contra tu propio build.
- **Persistes el bump canónico** en la cuenta en el init, así las instrucciones posteriores vuelven a derivar la misma dirección con `bump = vault.bump` y nunca pagan una búsqueda en tiempo de ejecución.
- **El esquema de seeds es la frontera de seguridad**. Un seed faltante o controlado por el usuario es un bug de colisión o de suplantación, no un detalle de estilo.

El repliegue de la ayuda de hoy: en el Lab te entrego cada línea del quarter-vault. En el problema Completion vuelvo a sacar el array `seeds` y el binding de `bump` y tú los rellenas. En el Challenge en solitario recibes una especificación y una forma de seeds, y escribes el handler, el struct de derive y la prueba sin ningún código delante.

## Cómo un programa es dueño de una dirección que nunca tuvo

Arranca por la cosa contra la que se define un PDA, porque el contraste hace la mayor parte de la enseñanza. Una cuenta normal de Solana tiene un par de claves: una clave privada y la clave pública derivada de ella. La clave pública es un punto sobre la curva Ed25519, y la clave privada es lo que te deja firmar. Dueño de la clave privada, dueño de la cuenta.

Una dirección derivada de programa es lo opuesto deliberado. Tomas unos **seeds** (slices de bytes arbitrarios que eliges tú) y el ID propio del programa, los hasheas juntos, y revisas si el resultado cae sobre la curva. Si cae, esa dirección *podría* tener por ahí una clave privada que le corresponda, que es exactamente lo que no quieres para una cuenta que tu programa tiene que controlar unilateralmente. Así que la rechazas y vuelves a intentar con un ajuste chico, hasta que consigues una dirección que demostrablemente no tiene ninguna clave privada. Esa es una dirección fuera de la curva, y es todo el truco: a un programa se le puede otorgar autoridad para firmar por una dirección fuera de la curva precisamente *porque* ningún par de claves puede.

Los seeds son la entrada que tú diseñas. El ID del programa acota la derivación a tu programa (otro programa con código distinto no puede derivar dentro de tu namespace). La salida es una dirección de 32 bytes que es una función pura de esas entradas.

![Una comparación de tres filas que muestra que los pares de claves por vault se filtran, que una cuenta de registro agrega costo de rent y de migración, y que la derivación de PDA no guarda nada porque el mapeo es el cómputo.](assets/v01-comparison.webp)

Contrasta eso con las dos cosas que un programa podría hacer en su lugar, porque la comparación es lo que hace que los PDA encajen. Podría generar un par de claves por vault y guardar el secreto en algún lado, lo que quiere decir que el programa es ahora el custodio de miles de secretos y una sola filtración vacía a todos de golpe. O podría mantener una cuenta de registro que mapee cada jugador a la dirección de su vault, lo que quiere decir una cuenta más que asignar, por la que pagar rent, que mantener consistente bajo concurrencia, y que migrar cada vez que el esquema cambia. El PDA colapsa los dos problemas en aritmética. No hay ningún secreto que se filtre porque no hay ningún secreto, y no hay ningún registro que corromper porque el mapeo *es* la derivación. El programa vuelve a computar la dirección de cualquier vault a demanda a partir de entradas que ya tiene.

![Los seeds, el ID del programa y un byte de bump se hashean; un resultado sobre la curva se rechaza y el bump decrementa; un resultado fuera de la curva se vuelve el PDA, y nunca se genera ningún par de claves.](assets/v02-diagram.webp)

### El bump, y por qué solo uno de ellos cuenta

Ese "ajuste chico" es el bump. Es un solo byte que se agrega al final de tus seeds antes de hashear. La derivación arranca en el bump `255` y camina hacia abajo: `255`, `254`, `253`, y así. En cada valor hashea y revisa la curva. Estadísticamente alrededor de la mitad de todos los puntos candidatos caen fuera de la curva, así que casi siempre tienes éxito dentro de los primeros intentos. El **primer** bump que produce una dirección fuera de la curva es el **bump canónico**, y por convención es el único válido.

Ayuda imaginarse qué quiere decir "fuera de la curva" en realidad. Ed25519 es una curva elíptica específica, y una clave pública válida es un punto que se sienta sobre ella. Alrededor de la mitad de todos los valores de 32 bytes son puntos válidos de la curva y la mitad no. Una clave normal de billetera es, por construcción, uno de los puntos *sobre* la curva, porque se generó a partir de un escalar privado que vive en ese punto. Un PDA es lo inverso: sigues hasheando hasta que caes en uno de los valores que *no* es un punto sobre la curva, que es exactamente el conjunto de direcciones que ninguna clave privada puede producir jamás. Toma una jugadora concreta, Ana, cuya dirección es `An4...k2`. El runtime hashea `[b"vault", An4...k2, 255, program_id]` y, digamos, cae sobre la curva. Rechaza, baja a `254`, hashea otra vez, sigue sobre la curva, rechaza, baja a `253`, y esta vez el punto está fuera de la curva. Así que `253` es el bump canónico de Ana, los 32 bytes resultantes son el vault de Ana, y los dos son una función pura de su clave y de nada más. Corre la derivación idéntica la semana que viene, el año que viene, desde una máquina en frío que no guarda ningún estado, y obtienes la misma dirección todas las veces. Esa permanencia es todo el feature, no un efecto secundario de él.

¿Por qué importa tanto que "solo uno cuenta"? Porque otros bumps más abajo en la lista podrían *también* producir direcciones fuera de la curva. Esos son PDA reales y derivables para los mismos seeds. Si tu programa acepta cualquier bump que quien llama le pase, un atacante puede presentar un PDA *distinto*, no canónico, para el mismo vault lógico, sembrarlo con su propio estado, y hacerlo pasar por una verificación que solo comprobó "¿es este un PDA válido para estos seeds?". Esa es la familia de bugs por sustitución de cuentas. La defensa es simplísima: usa siempre el bump canónico, y nunca confíes en un bump que llegó como entrada no confiable.

![La derivación prueba desde el bump 255 hacia abajo; el primer bump que da una dirección fuera de la curva es el canónico, y cualquier bump más bajo fuera de la curva es un PDA no canónico que un atacante podría sustituir.](assets/v03-table.webp)

### El struct `bumps` tipado — un repaso, y el delta que V2 sí agrega

Primero, el crédito donde corresponde, porque atribuir esto mal es el error que comete la mitad de los posts sobre migración que escribe el ecosistema. En el Anchor *viejo* — 0.28 y anteriores — la macro encontraba el bump canónico durante la validación de cuentas y lo metía en un mapa indexado por cadena. Lo pescabas de vuelta con `ctx.bumps.get("vault").unwrap()`. Dos cosas de esa línea envejecieron mal. Primero, `"vault"` es una cadena, así que un typo compila sin problema y explota en tiempo de ejecución. Segundo, `.get(...)` devuelve un `Option`, así que le haces `.unwrap()` a un valor cuya existencia el compilador no puede demostrar.

Anchor 0.29 mató ese mecanismo — dos majors antes de que V2 existiera. Desde entonces, `#[derive(Accounts)]` genera, en tiempo de expansión, un struct `bumps` tipado con un campo por cada cuenta PDA de tu derive, leído como un campo: `ctx.bumps.vault`. Ninguna cadena, ningún `Option`, ningún `.unwrap()`. Escribe mal el nombre del campo y el programa no compila. Sobre la línea base 1.x contra la que este curso mide V2, `ctx.bumps.get("vault")` no es "la vieja forma" — no compila en absoluto. Así que cuando leas `ctx.bumps.vault` en el código de abajo, estás mirando continuidad, no cambio: la RC mantiene el struct tipado exactamente como lo tiene 1.x.

Lo que V2 *sí* agrega se sienta por debajo, y vale la pena ser preciso porque esta es la oración que la gente entiende mal. Cuando los seeds de un PDA son todos literales de bytes en tiempo de compilación, la macro de V2 precalcula el bump canónico en tiempo de expansión como una const, y la validación no busca nunca. Esa optimización no puede aplicarse a este vault: tus seeds incluyen `player.address()`, y ningún compilador sabe qué jugador va a llamar, así que el framework recurre a derivar durante la validación, exactamente como hace 1.x. Lo que quiere decir que la historia de CU acá es enteramente sobre qué constraint escribes, en cualquiera de las dos líneas. Un `bump` desnudo corre la búsqueda. `bump = vault.bump` corre una derivación contra un valor que ya guardaste. Esa diferencia es el cómputo; el struct tipado es la seguridad de tipos, y tenías las dos cosas antes de V2.

Si estás portando un programa 1.x, la línea del bump es la que *no* cambia: `let bump = ctx.bumps.vault;` se lee idéntico en los dos lados. La reescritura mecánica vive en otra parte, y vale la pena hacerla a mano una vez para que se te fije en la memoria muscular. Cada constraint de seeds que decía `player.key().as_ref()` se vuelve `player.address().as_ref()`, ya que `Pubkey` ahora es `Address` y `.key()` ahora es `.address()`. La firma del handler pierde su lifetime `<'info>` y gana un `&mut`, así que `pub fn init(ctx: Context<Init>)` se vuelve `pub fn init(ctx: &mut Context<Init>)`. Los tipos de cuenta también sueltan sus lifetimes, así que `Account<'info, Vault>` colapsa a `Account<Vault>`. Nada de eso es cosmético; cada edición le entrega al compilador una garantía que antes no tenía. (Si el snippet que heredas es de verdad antiguo — de la era 0.28, con mapa de cadenas y todo — entonces `ctx.bumps.get("vault").unwrap()` sí se vuelve `ctx.bumps.vault`, pero esa es una migración de 0.29 que estás pagando con atraso, no una de V2.)

![El Anchor pre-0.29 leía el bump a través de una búsqueda de cadena en tiempo de ejecución, falible y capaz de entrar en panic; de 0.29 en adelante, V2 incluido, lee un campo de struct tipado cuyo cableado se resuelve en tiempo de expansión de macro, así que los typos no compilan, mientras que una derivación con seeds de tiempo de ejecución sigue corriendo durante la validación en todas las líneas.](assets/v04-comparison.webp)

Hay un segundo ahorro escondido debajo de la misma reescritura, y vale la pena nombrarlo con precisión porque es fácil exagerarlo. Para una cuenta de la que tu *propio programa es dueño*, V2 se salta la verificación de curva que correría una verificación de dirección general. El changelog de Anchor V2 reporta esto como alrededor de 1,000 CU ahorrados por verify. Trátalo como una cifra reportada por el proyecto, no como una ley de la naturaleza: mídelo contra tu propio build, y vuelve a revisarlo cuando subas la RC, porque los números de cómputo se mueven entre release candidates. El razonamiento detrás del salto es limpio, que es por lo que es seguro. Si la dirección la derivó tu programa a partir de seeds y un bump, y la estás tratando como propiedad del programa, entonces que resulte estar sobre la curva o no es información que no necesitas. Ya sabes que es tuya. Pagar cómputo para volver a responder una pregunta que ya respondiste es la clase de desperdicio que una reescritura desde cero existe para borrar.

![Una barra de antes/después muestra que saltarse la verificación de curva en un PDA propiedad del programa ahorra alrededor de 1,000 CU por verify, etiquetada como una cifra reportada por el proyecto que hay que volver a medir.](assets/v05-chart.webp)

¿De dónde salió esto? No es una optimización aislada que alguien atornilló encima. Anchor V2 es una reescritura `no_std` desde cero construida sobre pinocchio, el framework de cuentas mínimo y sin dependencias, descrito así en el propio crate lang-v2 al 2026-08. La misma reescritura que hizo posibles los bumps plegados a const es la que queda señalada en el manifiesto `#4390` "zero-copy by default" (otter-sec/anchor#4390), que replantea el viejo `Account<T>` deserializado por borsh como *el camino lento* y lleva el zero-copy al valor por defecto. Los bumps const y los saltos de la verificación de curva solo son posibles porque alguien desarmó el framework hasta los cimientos. Ese es el color que vale la pena llevarse al Lab: el struct Pod que construiste en el módulo 2 ya no es un caso especial. En V2 es la fibra de la madera.

### La decisión de custodia, dicha en voz alta

Una nota de alcance antes del código, porque te va a morder más adelante si se queda implícita. El quarter-vault que estás por construir guarda **SOL nativo**, como lamports, directo en la cuenta. No tokens. Los créditos son fichas, las fichas son lamports, por ahora. Eso mantiene esta lección sobre PDAs y nada más. El upgrade a SPL, donde el vault se gradúa a guardar un balance de token de verdad, aterriza en el módulo 5. Si te encuentras recurriendo a una cuenta de token hoy, detente: esa es una lección posterior filtrándose en esta.

Y el trade-off, dicho sin adornos porque es la parte honesta. Los PDA te dan direccionamiento determinista sin ningún par de claves que cuidar y sin ninguna tabla de búsqueda que mantener. Lo que pagas por eso es *responsabilidad de diseño de seeds, para siempre*. El esquema de seeds no es una convención de nombres, es el namespace y la frontera de control de acceso a la vez. Hazlo bien y cada jugador tiene un vault aislado y re-derivable. Hazlo con flojera, y tienes un bug de colisión que ninguna cantidad de código posterior puede tapar.

![Con solo el seed b"vault" cada jugador deriva un único PDA compartido, pero agregar la dirección del jugador le da a cada jugador un vault distinto y re-derivable.](assets/v06-diagram.webp)

Tres maneras en que esto muerde, nombradas ahora para que las reconozcas antes de que te cuesten nada. Primero, volver a derivar el bump en tiempo de ejecución. Si un handler posterior llama a `find_program_address` para "conseguir un bump fresco", reintrodujiste exactamente la búsqueda que un bump guardado se construyó para borrar, y la pagas en todas y cada una de las llamadas. Ponle un número: la referencia de constantes de Solana le pone precio a un syscall de derivación de PDA en **1,500 CU** (`create_program_address_units`). Validar contra un bump guardado cuesta exactamente uno de esos. `find_program_address` paga uno *por cada bump que prueba* antes de caer fuera de la curva, caminando 255 hacia abajo, así que la cuenta es 1,500 CU multiplicados por la cantidad de candidatos que los seeds resulten necesitar. Vuelve a medir en tu propio build, pero la dirección no está en discusión: cada intento evitado son otros 1,500 CU que te quedas, que es por lo que persistes el bump. Segundo, un seed controlado por el jugador o sub-especificado. Cualquier cosa del conjunto de seeds en la que quien llama pueda influir es una palanca que puede accionar para dirigir una derivación hacia una cuenta que no es suya, o hacia una cuenta compartida que debería haber estado aislada por jugador. El arreglo es amarrar la identidad dentro de los seeds, que es exactamente lo que hace la dirección del jugador. Tercero, y esta es la sutil, tratar el ahorro que da saltarse la verificación de curva como un número fijo contra el que puedes presupuestar. Es una cifra reportada por el proyecto, de una release candidate. Diseña como si el mes que viene pudiera leerse 800 CU o 1,200, porque podría.

## Lab: construye R2, el quarter-vault

Estás construyendo `quarter_vault`, un programa de Anchor V2 con un solo trabajo: crear un vault PDA por jugador, guardar el dueño, el bump canónico y un balance de crédito en cero, y exponer un camino de lectura. Una prueba de Rust con LiteSVM va a demostrar que el PDA es derivable y que el estado se lee de vuelta. El criterio de `verify` para este artefacto es una prueba que pasa: el vault deriva de `[b"vault", player]`, inicializa, y se lee de vuelta.

**1. Genera el scaffold y fija el toolchain.** Desde un directorio vacío:

```bash
anchor init quarter-vault   # the default test template is LiteSVM (Rust tests)
cd quarter-vault
```

Abre el `Cargo.toml` del programa y confirma que la dependencia monta sobre la misma release de V2 que el CLI. El scaffold escribe una fila git que sigue la branch `anchor-next`; reemplázala con la versión de crates.io, la misma edición que m01-l2 recorrió. El CLI es una instalación de git y la librería es una versión de registry, y eso no es un desfase — son grafos de dependencias separados que nombran la misma release, y la versión del registry es la que no se puede mover debajo de ti:

```toml
[dependencies]
anchor-lang = "2.0.0-rc.1"     # crates.io; the branch tip wants solana-address 2.7.0
# The pins from m01-l2 — every program crate in this course carries them (issue #4937's class).
wincode = { version = "0.5", features = ["derive"] }
# The arcade-workspace row from m02-l1, verbatim: a ceiling, not an equality. One
# workspace resolves one solana-address, and module 6's Mollusk dev-dep needs ^2.6.1.
solana-address = ">=2.6.1, <2.7"
```

Resultado esperado después de este paso: `anchor build` funciona sobre la plantilla sin tocar y `target/deploy/quarter_vault.so` existe. Si el build falla acá, es un problema de toolchain, no un problema de PDA, y arreglarlo ahora te ahorra depurar la capa equivocada en el paso 5.

**2. Define el estado del vault.** Este es el struct Pod, la misma forma que conoces del módulo 2, ahora destinado a una dirección derivada de programa. Ponlo en `programs/quarter-vault/src/lib.rs`. Fíjate en los tipos de los campos: en V2, `Pubkey` es `Address`.

```rust
use anchor_lang::prelude::*;

// Leave the id `anchor init` generated here. It matches the keypair in
// target/deploy/, and a hand-typed string gives you an id the deploy cannot
// sign for. `anchor keys sync` re-aligns them if they ever drift.
declare_id!("<your generated program id>");

#[account]
#[repr(C)]
#[derive(InitSpace)]
pub struct Vault {
    pub owner: Address,  // 32 bytes: the player who owns this vault
    pub credit: u64,     //  8 bytes: prepaid credit, native lamports for now
    pub bump: u8,        //  1 byte: the canonical bump, persisted for reuse
    pub _pad: [u8; 7],   //  7 bytes: explicit tail padding, zeroed, never read
}
```

Resultado esperado después de este paso: `anchor build` sigue funcionando. El struct compila por sí mismo, antes de que ningún constraint se refiera a él, que es el lugar más barato posible para detectar un error de tipo de campo.

Ese campo `_pad` es la disciplina del módulo 2 aterrizando en una cuenta de verdad, así que no lo borres. `credit` es un `u64` nativo, que lleva una alineación de 8 bytes, así que Rust redondea el struct entero hacia arriba a un múltiplo de 8: 41 bytes de campos se vuelven 48 bytes de layout, y esos siete bytes existen les pongas nombre o no. Sin nombre, son relleno implícito y el bound de Pod rechaza el struct. Con nombre y en cero, son un campo como cualquier otro y el cast es sólido. Escribe el relleno que ya estás pagando.

`#[derive(InitSpace)]` calcula el tamaño de datos de la cuenta por ti, así que nunca cuentas a mano. Guardar el `bump` en la cuenta es la disciplina sobre la que gira toda la lección: computas el bump canónico exactamente una vez, en el init, y reutilizas el valor guardado en todas partes después.

Un detalle de V2 viaja junto en la línea `space` que estás por escribir. V1 te enseñó a agregar un `8` mágico para el discriminador de cuenta, la etiqueta que Anchor escribe al frente de cada cuenta para poder distinguir un `Vault` de un `Config` cuando lee bytes crudos. V2 te impide hardcodear eso: escribes `Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE`, y si el esquema de discriminador cambia debajo de ti en algún momento, tu aritmética de space cambia con él en vez de ponerse mal en silencio. (V2 incluso va a inferir la línea `space` entera a partir del `INIT_SPACE` del wrapper si la omites; un `space =` explícito sigue siendo aceptado, y escribirlo una vez vale la práctica de ver de dónde sale el número.) Es el mismo instinto que el bump moviéndose a una const. Deja de cargar a mano números que el framework está dispuesto a entregarte.

![El constraint init empareja un array de seeds de b"vault" más la dirección del jugador con un bump desnudo, así que la macro deriva y guarda el bump canónico.](assets/v07-annotated-code.webp)

**3. Escribe el struct de derive y el handler de init.** Los handlers de V2 toman `&mut Context<T>` y el struct de cuentas no lleva ningún lifetime `<'info>`. Las dos cosas son consecuencias de la reescritura de pinocchio. Agrega esto a `lib.rs`:

```rust
#[program]
pub mod quarter_vault {
    use super::*;

    pub fn init_vault(ctx: &mut Context<InitVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = *ctx.accounts.player.address();
        vault.bump = ctx.bumps.vault; // canonical bump, read as a field off the typed bumps struct
        vault.credit = 0;
        Ok(())
    }

    pub fn read_vault(ctx: &mut Context<ReadVault>) -> Result<()> {
        let vault = &ctx.accounts.vault;
        msg!("vault credit={} bump={}", vault.credit, vault.bump);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitVault {
    #[account(mut)]
    pub player: Signer,
    #[account(
        init,
        payer = player,
        space = Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE,
        seeds = [b"vault", player.address().as_ref()],
        bump,
    )]
    pub vault: Account<Vault>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct ReadVault {
    pub player: Signer,
    #[account(
        seeds = [b"vault", player.address().as_ref()],
        bump = vault.bump, // reuse the STORED bump, no runtime search
    )]
    pub vault: Account<Vault>,
}
```

Resultado esperado después de este paso: `anchor build` compila los dos handlers y los dos structs de derive. Sé preciso sobre cuál typo atrapa este paso, porque solo uno de los dos es un error de build. Escribe mal el *campo* del struct de bumps — `ctx.bumps.valut` — y el compilador lo rechaza, porque el derive generó un struct con un campo por cada cuenta PDA y no existe tal campo. Escribe mal el *contenido del seed* — `b"valt"` en vez de `b"vault"` — y compila perfecto, porque una cadena de bytes es tan válida como cualquier otra; esa sí falla en tiempo de ejecución como una violación de constraint de seeds, que es exactamente la falla que el paso 5 te hace leer. El struct de bumps tipado corre el primer typo hacia la izquierda. No tiene nada que decir sobre el segundo.

Mira con atención la diferencia entre las dos líneas `bump`, porque es el punto de todo el build. En `InitVault` escribes un `bump` desnudo, que le dice a la macro que encuentre el bump canónico y te lo entregue a través de `ctx.bumps.vault`. En `ReadVault` escribes `bump = vault.bump`, que le dice a la macro que se salte la búsqueda por completo y valide contra el valor que ya guardaste. El primero es cómputo que pagas una vez. El segundo es cómputo que nunca vuelves a pagar.

![El cliente deriva el PDA y envía init_vault; la macro lo vuelve a derivar, provee el bump canónico, crea y fondea la cuenta, y después el handler guarda owner, bump y credit.](assets/v08-flowchart.webp)

**4. Escribe la prueba de LiteSVM.** LiteSVM corre el programa en proceso, sin validador, así que el loop es rápido. Llegas a él a través de `anchor-v2-testing`, el banco de pruebas contra el que genera el scaffold de V2, y **no** a través de una dependencia directa de `litesvm`:

```toml
# One row, not three. anchor-v2-testing wraps LiteSVM and re-exports the pieces a
# test file needs, so the SVM version is the harness's problem and not yours. At tag
# v2.0.0-rc.1 it carries litesvm 0.11.0; crates.io's latest is 0.16.0 as of
# 2026-09-07, and the anchor-next branch tip has moved it to 0.13.1. Name litesvm
# yourself and you are choosing one of those against a harness that expects
# another — two SVM versions in one graph, failing in a way that reads like your
# test is wrong.
[dev-dependencies]
anchor-v2-testing = { git = "https://github.com/otter-sec/anchor.git", tag = "v2.0.0-rc.1" }
bytemuck = "1.25"     # to cast the account bytes back to the Pod state
```

Después la prueba en sí, en `tests/quarter_vault.rs`. Deriva el PDA de la misma forma que lo hace el programa, envía `init_vault`, y lee la cuenta cruda de vuelta para afirmar el estado guardado. Fíjate de dónde viene cada import: la plomería de instrucciones monta sobre `anchor_lang`, la plomería de firma y de SVM monta sobre `anchor_v2_testing`, y nada alcanza más allá de ninguno de los dos:

```rust
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{Keypair, Message, Signer, VersionedMessage, VersionedTransaction};
use bytemuck::from_bytes;

#[test]
fn quarter_vault_pda_derives_and_reads_back() {
    // `svm()` is LiteSVM::new() plus the profiling hook `anchor test --profile` turns on.
    let mut svm = anchor_v2_testing::svm();
    let program_id = quarter_vault::ID;
    // cargo runs a test binary with its working directory at the package root, so a bare
    // "target/deploy/..." would resolve inside programs/quarter-vault/ and miss. Anchor the
    // path on the crate instead: the artifact lives in the workspace target dir, two up.
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    svm.add_program_from_file(program_id, vault_so).unwrap();

    // A player who will pay rent and own the vault.
    let player = Keypair::new();
    svm.airdrop(&player.pubkey(), 1_000_000_000).unwrap();

    // Derive the PDA exactly as the program does: [b"vault", player].
    let (vault_pda, expected_bump) =
        Address::find_program_address(&[b"vault", player.pubkey().as_ref()], &program_id);

    // Build and send init_vault.
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::InitVault {
            player: player.pubkey(),
            vault: vault_pda,
            system_program: System::id(),
        }
        .to_account_metas(None),
        data: quarter_vault::instruction::InitVault {}.data(),
    };
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&player.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&player]).unwrap();
    svm.send_transaction(tx).unwrap();

    // Read the raw account and cast the Pod state - no deserialize step, same as
    // module 2. `from_bytes` wants EXACTLY size_of::<Vault>() bytes, and it wants
    // Vault to be Pod, which is what the `_pad` field bought you. And yes, the
    // discriminator offset is spelled out rather than hardcoded as 8: same rule as
    // the `space =` line, for the same reason.
    let raw = svm.get_account(&vault_pda).unwrap();
    let start = quarter_vault::Vault::DISCRIMINATOR.len();
    let vault: &quarter_vault::Vault =
        from_bytes(&raw.data[start..start + core::mem::size_of::<quarter_vault::Vault>()]);

    assert_eq!(vault.owner, player.pubkey().into()); // owner is the player
    assert_eq!(vault.credit, 0); // credit starts at zero
    assert_eq!(vault.bump, expected_bump); // stored bump IS the canonical bump
}
```

Esa última aserción es la que demuestra la disciplina: el bump que guardaste es igual al bump canónico que el cliente derivó de forma independiente. Ninguna re-derivación en tiempo de ejecución, el mismo valor.

Ahora ejercita la otra mitad. Agrega una segunda instrucción a la misma prueba, `read_vault`, construida de la misma forma a partir de `accounts::ReadVault { player, vault: vault_pda }` e `instruction::ReadVault {}`, y envíala después del init. No tiene ninguna aserción que hacer más allá de tener éxito, y ese es el punto: `read_vault` es el camino restringido por `bump = vault.bump`, así que un envío verde es la demostración de que el bump guardado valida la misma dirección que encontró la búsqueda. Déjala fuera y la única línea que esta lección existe para enseñar nunca se ejecuta.

Resultado esperado después de este paso: el archivo de prueba compila (`cargo test --no-run` alcanza para revisarlo) incluso antes de que lo corras. Un desajuste entre estos nombres de cuentas y los campos de tu struct de derive es un error de compilación, así que una compilación limpia quiere decir que el cliente y el programa están de acuerdo en la lista de cuentas.

**5. Construye y corre.**

```bash
anchor test
```

Salida esperada, la única prueba que pasa y cumple el criterio:

```
running 1 test
test quarter_vault_pda_derives_and_reads_back ... ok

test result: ok. 1 passed; 0 failed
```

Si en cambio ves una falla de constraint de seeds en el camino `ReadVault`, la causa habitual es derivar con un orden de seeds distinto o con una clave distinta en la prueba que en el programa. Los seeds tienen que coincidir byte por byte en los dos lados. Eso no es un bug en Anchor, es el esquema de seeds siendo exactamente tan estricto como prometió ser.

## Challenge

Dos peldaños, y el apoyo se adelgaza en cada uno.

**Completion.** Abre el struct de derive que acabas de escribir y borra dos cosas: reemplaza el array `seeds = [...]` con `seeds = [/* TODO */]` y la línea `bump` con `bump, // TODO: canonical or stored?` en `InitVault` y en `ReadVault`. Ahora rellénalas de memoria. La verificación de aceptación: `init_vault` usa un `bump` desnudo y deriva de `[b"vault", player.address().as_ref()]`, mientras que `read_vault` usa `bump = vault.bump` y el *mismo* array de seeds. Si recurres a `find_program_address` dentro de un handler, tomaste el camino equivocado: la macro ya hizo ese trabajo.

**Solo.** Dale a un solo jugador más de un vault. Agrega un argumento `slot: u8` a un nuevo handler `init_vault_slot` y enhébralo en los seeds para que el PDA se vuelva `[b"vault", player.address().as_ref(), &[slot]]`. Vas a necesitar `#[instruction(slot: u8)]` en el struct de derive para que el constraint pueda ver el argumento. Demuestra dos cosas en una prueba de LiteSVM: que el slot `0` y el slot `1` para el mismo jugador producen dos direcciones *distintas*, y que llamar a cada uno una segunda vez vuelve a derivar la *misma* dirección que la primera vez (así que los dos son estables, re-derivables, y sus bumps guardados coinciden con el valor que deriva el cliente). Aceptación: los dos vaults inicializan, los dos vuelven a derivar en una segunda llamada, y ningún handler vuelve a derivar un bump en tiempo de ejecución. Un punto extra que vale la pena perseguir: intenta hacer `init_vault` dos veces para los mismos seeds y mira fallar la segunda llamada. Esa falla es la guarda de cuenta-ya-existe haciendo su trabajo, y es la razón por la que un vault no puede reinicializarse en silencio por debajo de un jugador.

![Una línea de tiempo desde los bumps indexados por cadena pre-0.29, pasando por el struct bumps tipado de Anchor 0.29, el manifiesto #4390 zero-copy-by-default y la reescritura no_std de V2 sobre pinocchio, hasta las consts de bump de hoy, calculadas en tiempo de macro cuando los seeds son literales, y el salto de la verificación de curva para los PDA propiedad del programa.](assets/v09-timeline.webp)

Cuando los dos peldaños pasen, siéntate con lo que de verdad demostraste. Dos jugadores reciben dos vaults aislados, un jugador recibe tantos slots como quiera, cada dirección vuelve a derivar a los mismos 32 bytes para siempre, y ninguno de ellos necesitó un par de claves ni una tabla de búsqueda. Tú escribiste el esquema de seeds, y el esquema de seeds *es* el modelo de custodia. Ese es el peso del que advertía la parte honesta, y lo cargaste correctamente.

Fíjate, además, cuánto te compró ya el esquema de seeds. Como el vault se deriva de `player.address()` y `player` tiene que firmar, nadie puede inicializar ni leer un vault que no sea suyo: la derivación *es* la verificación de acceso, gratis, sin ningún constraint escrito. Vale la pena saber eso con precisión, porque es la frontera de lo que los seeds pueden hacer por ti.

Y se detienen justo ahí. En el momento en que a este programa le crezca un *operador* de arcade, alguien que pueda ajustar el crédito en todos los vaults, el esquema de seeds no tiene nada que decir sobre quién es ese, y `Signer` demuestra solo que alguien firmó, nunca quién. La próxima lección empuñas el catálogo de constraints de V2, `address`, `owner`, `constraint`, `close`, `realloc_payer`, y haces que la macro de derive rechace las cuentas equivocadas antes de que corra el código de tu handler. El vault recibe un portero.
