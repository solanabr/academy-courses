# Dos líneas paralelas, y la instalación que te pelea

La lección pasada confirmaste dos transacciones que alguien más ya había aterrizado en devnet, una contra el gemelo de v1 y otra contra el gemelo de v2, y leíste el delta de unidades de cómputo directo de los logs. Miraste la brecha. No construiste ni desplegaste nada propio.

Eso cambia ahora. Pero antes de ganarte tu primer deploy, abre una terminal y corre esto:

```bash
anchor --version
which anchor
```

Lo que imprima de vuelta es casi seguro Anchor **1.1.2**, la línea estable actual, instalada por `avm` y viviendo en tu PATH. Ese binario es la herramienta equivocada para este curso, y no te lo va a decir. Va a construir alegremente un Lab de V2 contra semántica de V1 y te va a entregar errores que no tienen sentido. Así que lo primerísimo que aprendes sobre Anchor V2 no tiene nada que ver con macros: la versión que ya tienes es una trampa, y que la versión que quieres devuelve la pelea cuando intentas instalarla.

Esa es la lección: la fricción misma, no un desvío que la rodea. Instalar una release candidate desde una rama, cuando el instalador oficial no tiene un binario que darte, es lo que se siente de verdad vivir en la frontera. Quiero que lo sientas una vez, conmigo narrando cada pared para que sepas que es la herramienta y no tú.

## Resumen

Anchor se entrega ahora mismo en dos líneas paralelas: la estable **1.1.2**, y **2.0.0-rc.1** montada sobre una rama sin mergear llamada `anchor-next`. Este curso vive en la segunda línea. Instalas esa RC en un toolchain aislado desde su canal git documentado, aprendes por qué `avm install 2.0.0-rc.1` no puede bajártela, registras todo el asunto en un archivo central de pins con fechas de frescura, y después generas el scaffold, construyes y despliegas el greeter (R0) a devnet como tu primer deploy independiente. R0 es el programa borrador: se sienta por debajo del primer peldaño en la escalera de Quarters, y lo sigues extendiendo el resto de este módulo antes de que empiecen los peldaños de verdad.

El repliegue de la ayuda acá es deliberado y superficial. Esta es una lección de toolchain, así que la instalación y el scaffold están **completamente trabajados**: yo muestro cada comando, tú sigues exacto, todavía nada de solo. El único paso que es solo tuyo es el deploy final. Corres `anchor deploy` contra devnet, lees de vuelta un program id, y lo pegas en el archivo de pins. Esa es toda la graduación.

Una nota honesta por adelantado. Cada número de versión de esta página es una instantánea con una fecha pegada, y la RC se va a mover. Eso es el costo de llegar semanas antes, no descuido. La disciplina de reverificar que construyes acá es la habilidad de verdad.

## Por qué la RC vive en su propia casa

Arranca desde la cosa que ya puedes ver. Hay dos líneas de Anchor, y corren en paralelo en vez de como una escalera de beta-y-después-estable.

La línea estable es **1.1.2**. Es lo que instala `avm`, lo que crates.io sirve como `anchor-lang`, y contra lo que construye hoy la mayor parte del ecosistema. La línea de frontera es **2.0.0-rc.1**. No vive sobre un release publicado y bendecido como sí lo hace 1.1.2. Vive en una rama de desarrollo llamada `anchor-next`, y la única forma documentada de sacar de ahí un CLI que funcione es construir esa rama tú mismo con cargo.

![Una comparación lado a lado de Anchor 1.1.2 estable (avm/crates, ya instalado) contra la frontera 2.0.0-rc.1 (construida desde la rama de git anchor-next, y etiquetada tanto "rc" como "alpha").](assets/v01-comparison.webp)

Acá está el por qué debajo del qué, porque vale la pena derivarlo una vez. Una release candidate en una rama sin mergear no es una promesa, es un trabajo en curso que resulta tener un número de versión. Si la dejas sobrescribir la 1.1.2 de tu PATH, ahora tienes exactamente un Anchor, y es el que no para de cambiar. En el momento en que `anchor-next` se rompa (y las RC se rompen, ese es su trabajo), cada proyecto de tu máquina se rompe con ella. El aislamiento acá es práctico, no quisquilloso: una herramienta estable para tu trabajo estable y una herramienta de frontera para tu trabajo de frontera, lado a lado, cada una honesta sobre lo que es.

La buena noticia es que "aislado" acá no quiere decir un contenedor ni una máquina virtual. Es más simple y más físico que eso. El `cargo install` que estás por correr deja un solo binario en `~/.cargo/bin/anchor`. `avm`, mientras tanto, maneja tu 1.1.2 a través de su propio shim. Los dos quieren contestar cuando escribes `anchor`, y cuál gana lo decide nada más exótico que el orden del PATH. Ese es todo el modelo de aislamiento: dos binarios en disco, un nombre, y tu shell eligiendo la primera coincidencia. Es también por qué la confusión más común de toda esta instalación es un build que se comporta como V1 cuando estabas seguro de haber instalado V2. La RC está ahí. Tu PATH te entregó el otro. Vas a confirmar cuál binario contesta en el Lab, y vale la pena interiorizar desde ahora que en la frontera, `which anchor` es un comando de debugging, no una formalidad.

### Por qué `avm install` no puede hacer esto por ti

Tu instinto, y con razón, es echar mano de `avm`. El **Anchor Version Manager** es la herramienta que instala y cambia entre versiones del CLI de Anchor, como hace `rustup` para Rust. Casi seguro lo usaste para conseguir tu 1.1.2. Si no lo tienes instalado, la vía documentada es un cargo install desde el repositorio de Anchor:

```bash
# avm - the Anchor Version Manager. The install docs still publish the
# solana-foundation URL; it 301-redirects to otter-sec/anchor, which is where
# the repo actually lives now (custody went coral-xyz -> solana-foundation ->
# otter-sec during 2026). Either URL resolves. Re-check before you run this.
# (freshness 2026-08-22)
cargo install --git https://github.com/otter-sec/anchor avm --locked --force
avm install 1.1.2
avm use 1.1.2
```

Así que intentas la cosa obvia:

```bash
avm install 2.0.0-rc.1
```

Y se niega. Lee la falla con cuidado, porque la razón es más aburrida y más útil de lo que parece:

```
Failed to download the binary for version `2.0.0-rc.1` (status code: 404 Not Found)
```

Eso no es un rechazo por política. `avm` parsea `2.0.0-rc.1` perfectamente bien; tiene todo un canal de prerelease (`avm install latest-pre-release`, `avm list --pre-release`). La falla es mecánica. Para cualquier versión en 0.31 o más arriba, `avm install` no construye nada: baja un **binario precompilado** desde los assets del GitHub Release del tag, en `releases/download/v2.0.0-rc.1/anchor-2.0.0-rc.1-<your-target>`. Esos assets solo existen si alguien cortó un **Release object** para el tag, y nadie lo hizo. Anda a buscar `releases/tags/v2.0.0-rc.1` y te da un 404 también. Sin Release, sin asset, sin descarga.

Fíjate en lo que *no* está pasando acá, porque es una historia que vas a escuchar mal contada. `avm` no verifica ninguna atestación criptográfica del release, y nunca lo hizo: no hay código de atestación adentro. La pared es un archivo que falta, no una verificación de firma que falló. Vale saberlo con precisión, porque una verificación de seguridad que no puedes satisfacer y un artefacto de build que nadie subió piden respuestas completamente distintas.

![avm install baja un binario precompilado desde los assets de release del tag v2, se topa con un 404 porque nunca se cortó ningún Release, y aborta, así que toma el relevo el cargo git install documentado.](assets/v02-flowchart.webp)

Para que quede completo: `avm install` sí carga un flag `--from-source`, que se salta la descarga y le pasa el trabajo a `cargo install --git https://github.com/otter-sec/anchor --tag v2.0.0-rc.1` — el mismo build que estás por correr a mano, con el canal elegido por ti.

Hay una trampa de nombres que vale la pena señalar antes de que te muerda. Si te vas a buscar `avm` en crates.io, vas a encontrar uno, y **no es esta herramienta**. El crate `avm` de crates.io es un paquete sin relación, de 2016 (schultyy/avm). El gestor de versiones de Anchor no se distribuye bajo ese nombre de crate, se instala desde el repo de Anchor. Instala el `avm` equivocado y te vas a pasar una hora confundido sobre por qué ninguno de los comandos existe.

### El canal de instalación, y la publicación que es una pista falsa

El canal documentado es un build de git. Acá está el comando exacto, y lo vas a correr de verdad en el Lab:

```bash
cargo install --git https://github.com/otter-sec/anchor.git \
  --branch anchor-next anchor-cli --locked --force
```

Léelo de izquierda a derecha, porque cada flag es estructural. `--git` más la URL del fork dice "construye desde el código fuente en este repositorio, no desde crates.io." `--branch anchor-next` fija el código fuente a la rama de frontera específicamente. `anchor-cli` es el crate dentro de ese repo que de verdad quieres como binario. `--locked` dice "respeta el Cargo.lock commiteado, no resuelvas dependencias más nuevas en silencio," lo que en una RC es la diferencia entre un build reproducible y un misterio. `--force` sobrescribe cualquier `anchor-cli` que cargo ya haya puesto en `~/.cargo/bin`.

`--branch anchor-next` es el flag que esta lección elige a propósito y que cada lección posterior sobrescribe, así que zanja la diferencia acá. La punta de una rama se mueve; un tag no. Al 2026-08-22 la punta de `anchor-next` va por delante de `v2.0.0-rc.1` y ya movió los dos crates que estás por fijar a mano: el `anchor-lang` de la punta pide `wincode 0.6` y `solana-address 2.7.0`, mientras que el tag — y el crate `2.0.0-rc.1` publicado desde él — pide `wincode 0.5` y la línea `solana-address 2.x` por debajo de 2.7. La línea de falla del issue #4937 corre exactamente ahí, y por eso los pins del Lab de abajo llevan una versión *y* una fecha.

Hoy te paras en la rama porque el canal *es* el tema de esta lección: es lo que publican los docs de V2, y un canal que se mueve no es algo sobre lo que puedas razonar desde afuera. Cada lección después de esta reimprime el comando con **`--tag v2.0.0-rc.1`** en lugar de `--branch anchor-next`, y m08-l2, cuyo trabajo entero es un build reproducible byte por byte, aprieta una vez más hasta `--rev e4878b6d`, el commit al que apunta ese tag. Tres formas de escribirlo, dos cosas distintas: párate en la rama acá para ver el canal, en el tag en todo lo que sigue, para que los pins que escribes sigan queriendo decir lo que querían decir cuando los escribiste.

Ahora, una cosa que te va a tentar. La RC **sí** se publicó en crates.io el 2026-08-12 como `2.0.0-rc.1`. Así que podrías pensar, con razón, que puedes saltarte el baile de git y hacer nada más `cargo install anchor-cli --version 2.0.0-rc.1`. No confíes en eso como tu vía de instalación. La publicación en crates.io existe, pero no está documentada ni probada para instalar el CLI. El canal sancionado y reproducible para el **binario** es el build de git. Cuando los docs y el registro no coinciden sobre qué es seguro instalar, los docs van atrasados de la realidad todo el tiempo, pero un crate publicado-pero-sin-probar es una apuesta peor que el build documentado. Confiar en el registro por encima del proceso documentado es la trampa número tres, retomando la cuenta de la lección pasada.

Lee eso tan estrechamente como está escrito, porque el Lab de abajo hace lo *contrario* para la librería. `anchor-cli` es un binario que el proyecto construye y prueba a través de su canal git; `anchor-lang` es una librería que el proyecto publica en crates.io a propósito, y su propio scaffold te dice que dependas de la versión publicada en cuanto exista una. Un artefacto por canal, cada uno en el canal que sus mantenedores de verdad soportan.

### La tensión rc-contra-alpha, enseñada en voz alta

Acá está una cosa chica que te dice mucho sobre dónde está de verdad este release. crates.io lo etiqueta `rc`. La propia página de benchmarks del proyecto llama `alpha` a ese mismo build. Mismo código, dos etiquetas de madurez, del propio proyecto.

No voy a elegir una por ti y pretender que el conflicto no existe. Eso lavaría exactamente la señal que necesitas. Un `rc` se supone que quiere decir "creemos que esto está casi entregable." Un `alpha` quiere decir "esto está temprano, espera que se rompa." Cuando el proyecto usa las dos palabras para un mismo build, la lectura honesta es: está en algún lugar en medio, y te conviene fijar la versión exacta que instalaste y reverificar con un cronograma en vez de confiar en la etiqueta. El conflicto de etiquetas no es ruido que haya que resolver, es la señal de madurez, y la respuesta correcta a él es un archivo de pins, y por eso estamos por construir uno.

La primera historia de guerra real de la RC vuelve concreto el punto. El issue #4937, abierto el 2026-08-16 y cerrado el 2026-08-20, era un desajuste de dependencias: `anchor-lang` fijaba `wincode` en 0.5 mientras `solana-address` 2.7.0 subía su propio requisito de `wincode` a 0.6, y el desajuste de trait bounds rompió `#[account(borsh)]`. (La versión que se movió es la de `wincode`; `solana-address` nunca entregó más que una línea 2.x, como deja claro el bloque de pins al final de esta lección.) Esa es la disciplina de fijar dependencias en la era RC, agarrada en vivo. Es exactamente por qué `--locked` está en tu comando de instalación y exactamente por qué cada pin que escribes lleva una fecha al lado.

### Los incrementos fechados de 1.0, para que V2 tenga un "antes"

Una pieza más de contexto, y esta es para todo lector sin importar cuál Anchor hayas tocado antes. Para entender por qué V2 cambió cosas, necesitas el mapa de lo que Anchor **1.0** ya había cambiado. Estos son los incrementos que aterrizaron con Anchor 1.0.0 el **2026-04-02**, y las lecciones posteriores van a volver a esta lista cada vez que digamos "V2 se quedó con esto" o "V2 fue más allá."

![Una línea de tiempo que marca Anchor 1.0.0 el 2026-04-02 con sus cinco incrementos (el rename del paquete, CpiContext tomando un Pubkey, transfer_checked por defecto, LiteSVM, Surfpool) y la publicación de 2.0.0-rc.1 en crates.io el 2026-08-12.](assets/v03-timeline.webp)

Recórrelos una vez, despacio, porque cada uno es un callback esperando a pasar.

El **rename del paquete** es el primero. El cliente de TypeScript que antes vivía en `@coral-xyz/anchor` ahora se publica como `@anchor-lang/core`. Eso no es un cambio cosmético. Cada línea de import en cada cliente que escribas contra un programa de Anchor apunta al nombre nuevo, y el día que cablees un cliente en m08, vas a pesar `@anchor-lang/core` a propósito — y lo vas a dejar de lado a propósito, porque todavía va montado sobre web3.js v1, así que generas un cliente nativo de kit en su lugar — mientras que la mayoría de los tutoriales online todavía muestran el nombre viejo.

Segundo, **`CpiContext` cambió de forma**. Cuando un programa llama a otro (una invocación entre programas), construyes un `CpiContext`, y en 1.0 su constructor toma el programa destino como un `Pubkey` (vía `program.key()`), no como un `AccountInfo` como hacía el Anchor más viejo. Este trae una trampa de fábrica: la página de documentación de anchor-lang.com para CPI todavía muestra la forma vieja con `AccountInfo`. Cuando m04 te meta en llamadas entre programas de verdad, el compilador es la autoridad, no esa página. Pasa el tipo equivocado y te va a decir `expected Pubkey, found AccountInfo`.

Tercero, **`transfer_checked` es el movimiento de tokens por defecto**. El `transfer` a secas en el que se apoyaba el código más viejo de SPL token está deprecado, y la CPI a la que recurres ahora es `transfer_checked`, que además toma el mint y sus decimales para que el runtime pueda agarrar un desajuste de decimales antes de mover valor. Cuando m05 cablee los flujos de tokens, vas a escribir `transfer_checked` sin pensar, y la razón de que no sea el `transfer` a secas empieza el 2026-04-02.

Cuarto, **LiteSVM es la plantilla de prueba por defecto**. Las pruebas que genera Anchor ya no asumen que levantas un validador completo para correr una sola afirmación. LiteSVM corre tu programa en proceso, y por eso el hilo de pruebas que se abre en m02 es lo bastante rápido para correr en cada guardado.

Quinto, **Surfpool es el validador local por defecto**. Cuando corres `anchor test` o `anchor localnet`, el validador de abajo es Surfpool, no el viejo `solana-test-validator`. Hoy no corres ninguna prueba, pero el scaffold ya escribió una para ti en `programs/greeter/tests/test_initialize.rs`, y la corres la próxima lección. Como la plantilla por defecto es LiteSVM, esa prueba corre en proceso y no necesita Surfpool para nada; en el momento en que recurras a una plantilla que sí habla con un validador, esta es la máquina del otro lado.

Ese rename tiene una sobrevida llamativa. Como ocho meses después de que `@coral-xyz/anchor` se volviera `@anchor-lang/core`, el paquete viejo todavía le gana en descargas al nuevo por cerca de **40 a 1** (601,707 contra 14,745 en una sola semana). "Actual" y "de uso común" se separaron fuerte. Esa brecha es tu recordatorio de que el ecosistema se mueve más lento que los números de versión, y de que cuando escribas un cliente más adelante en este curso, eliges el nombre que es correcto, no el nombre que es popular.

Ninguno de estos cinco cambios de 1.0 es algo que toques en el Lab de abajo. El programa que te genera el scaffold abre una cuenta y pone un counter en cero, nada más. Pero el punto de narrarlos ahora es que cuando m05 te muestre `transfer_checked` y te pregunte "¿por qué desapareció el `transfer` a secas?", la respuesta empieza acá, el 2026-04-02, no en V2 para nada.

### Las cuatro paredes, y cómo no chocar contra ellas

Antes de abrir una terminal, mantén los modos de falla a la vista. La frontera tiene exactamente cuatro paredes que atrapan a casi todos, y cada una es un caso de una herramienta siendo honesta mientras tú esperabas otra herramienta. Ninguna de ellas es tu código.

![Una tabla tipo runbook que empareja cada una de las cuatro paredes de la instalación en la frontera, más la trampa de nombres de avm, con la única línea correctiva que la resuelve.](assets/v04-table.webp)

Ten esa tabla cerca durante el Lab. Cuando algo se rompa, y en la frontera normalmente algo se rompe, empareja el síntoma con una fila antes de asumir que hiciste algo mal.

## Lab: instala la RC, genera el scaffold del greeter, despliega R0

Vas a construir un archivo central de pins, instalar la RC aislada, generar el scaffold de un greeter, construirlo, y desplegarlo a devnet. Los pasos 1 al 6 están completamente trabajados. El paso 7, el deploy, es tuyo.

**1. Confirma las dos herramientas debajo de Anchor.** Anchor se sienta encima de Rust y del CLI de Solana (Agave), así que fija esos primero. Si no tienes Rust, instálalo y fija el MSRV que la RC requiere, que es **1.89.0** (frescura 2026-08-22):

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install 1.89.0
rustc +1.89.0 --version   # expect: rustc 1.89.0
```

Instálalo, no lo pongas por defecto en tu máquina. El mismo argumento de aislamiento de la sección de teoría aplica una capa más abajo: fijar tu Rust global al MSRV de la RC arrastra todos los demás proyectos que tienes a ese toolchain. En vez de eso lo acotas. El scaffold que generas en el paso 4 escribe un `rust-toolchain.toml` que nombra 1.89.0, que rustup honra automáticamente dentro de ese directorio; si alguna vez necesitas forzarlo a mano, `rustup override set 1.89.0` desde la raíz del workspace hace el mismo trabajo para ese directorio solamente.

El **MSRV**, la versión mínima de Rust soportada, es el Rust más viejo sobre el que el crate promete compilar. Para una RC no es una sugerencia. Construye con algo más viejo y te salen errores que parecen decir que tu código está mal cuando el que está mal es el toolchain.

Después el CLI de Solana, que instalas a través del instalador de Anza:

```bash
# Agave (Solana) CLI
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
solana --version
```

Una palabra precisa sobre versiones acá, porque importa para todo el curso. La imagen de integración continua de este curso fija el CLI de Solana en **3.1.10**. Ese número es un **pin de toolchain de CI local**, el CLI exacto contra el que corre el verificador del Lab, y nada más. No es una afirmación sobre qué es "la Solana actual". La línea estable actual de Agave es **v4.2.1** mientras escribo esto (agosto de 2026; reverifícalo, se mueve). Si alguna vez ves 3.1.10 y piensas "entonces Solana está en 3.x", esa es la trampa número cuatro. Es un build fijado para labs reproducibles, punto.

**2. Crea el archivo central de pins.** Esto es infraestructura del curso, no algo desechable. Haz un archivo en la raíz de donde vayas a guardar el trabajo de este curso, llamado `PINS.md`, y siémbralo:

```markdown
# Course toolchain pins (re-verify on a schedule; the RC moves)

| pin                         | value                        | channel                          | verified   |
|-----------------------------|------------------------------|----------------------------------|------------|
| anchor-cli (this lesson)    | 2.0.0-rc.1                   | git anchor-next (otter-sec fork) | 2026-08-22 |
| anchor-cli (rest of course) | 2.0.0-rc.1                   | git tag v2.0.0-rc.1 = e4878b6d   | 2026-08-22 |
| anchor-lang (library)       | 2.0.0-rc.1                   | crates.io (immutable)            | 2026-08-22 |
| wincode                     | 0.5 (features = ["derive"])  | crates.io                        | 2026-08-22 |
| solana-address              | =2.6.0                       | crates.io                        | 2026-08-22 |
| Rust (MSRV)                 | 1.89.0                       | rustup                           | 2026-08-22 |
| macOS build workaround      | CARGO_PROFILE_RELEASE_LTO=off| env var (release profile)        | 2026-08-22 |
| Solana CLI (CI pin)         | 3.1.10                       | agave-install (LOCAL-CI ONLY)    | 2026-08-22 |
| R0 greeter program id       | <fill after deploy>          | devnet                           | <fill>     |

Note: 3.1.10 is the local-CI pin, NOT "current Solana" (current stable Agave: v4.2.1, Aug 2026).
Note: the two anchor-cli rows report the SAME version string. Only the ref distinguishes them.
Note: solana-address is =2.6.0 for this greeter, a workspace of one. From m02-l1 the arcade
programs share a workspace, and every member of it reads ">=2.6.1, <2.7" instead. Same ceiling.
Label tension: crates.io says "rc", the benchmarks page says "alpha". Pinned + re-verified on purpose.
```

Una **nota de frescura** es solo esa columna de fecha `verified`. Un pin sin fecha es una mentira esperando a pasar, porque la cosa a la que apunta puede moverse al día siguiente de que lo escribiste. En la frontera la fecha es la mitad del pin.

**3. Instala la RC aislada.** Este es el comando de la sección de teoría, corrido de verdad. En macOS, ponle adelante la solución alternativa de LTO (voy a explicar la solución alternativa justo después):

```bash
# macOS: the RC build dies during link-time optimization without this
CARGO_PROFILE_RELEASE_LTO=off \
cargo install --git https://github.com/otter-sec/anchor.git \
  --branch anchor-next anchor-cli --locked --force
```

En Linux el prefijo `CARGO_PROFILE_RELEASE_LTO=off` es inofensivo, así que dejarlo puesto mantiene un solo comando que funciona en todas partes. En macOS es obligatorio: sin él el build de la RC muere de forma confiable durante el **LTO** (link-time optimization, el paso final de optimización entre crates), y la falla parece un crash del linker antes que un problema de Anchor. Poner la variable de entorno de cargo para el perfil de release apaga ese paso y el build se completa. Esa única línea pertenece a tu `PINS.md`, que es exactamente por qué ya está en la tabla de arriba. Fíjate en el nombre: es `CARGO_PROFILE_RELEASE_LTO`, una variable estándar de perfil de cargo, no una invención tipo `ANCHOR_LTO`.

![El comando de instalación de la RC partido en sus partes, con cada flag glosado: la variable de entorno de LTO, --git y --branch anchor-next, --locked, y --force.](assets/v05-annotated-code.webp)

Cuando termine, verifica que te quedó la RC y no tu binario viejo:

```bash
anchor --version   # expect: anchor-cli 2.0.0-rc.1
```

Si eso todavía imprime 1.1.2, tu shell resolvió primero el binario viejo. Revisa `which anchor` y asegúrate de que `~/.cargo/bin` esté temprano en tu PATH. Este es el aislamiento funcionando: cargo puso la RC en `~/.cargo/bin/anchor`, y el shim de `avm`, si gana la carrera del PATH, te va a seguir sirviendo 1.1.2. Lo que sea que imprima, registra el de verdad en `PINS.md`.

**4. Genera el scaffold del greeter.** Ahora haz tu primer programa de V2. `anchor init` genera un workspace completo y construible:

```bash
anchor init greeter
cd greeter
```

Ese único comando escribe un proyecto entero. Acá está lo que aterriza, para que el árbol no sea una caja negra:

![El árbol del workspace de greeter generado, con programs/greeter/src/lib.rs resaltado como el programa de verdad, una prueba de Rust con LiteSVM generada al lado, y app/ y migrations/ marcados como scaffolding que todavía no se usa.](assets/v06-diagram.webp)

Abre `programs/greeter/src/lib.rs`. Acá está lo que la plantilla de V2 escribe de verdad, textual salvo por tu program id generado:

```rust
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod greeter {
    use super::*;

    pub fn initialize(ctx: &mut Context<Initialize>) -> Result<()> {
        ctx.accounts.counter.count = 0;
        ctx.accounts.counter.authority = *ctx.accounts.payer.address();
        msg!("Counter initialized");
        Ok(())
    }
}

pub mod state {
    use super::*;

    #[account]
    pub struct Counter {
        pub count: u64,
        pub authority: Address,
    }
}

use state::Counter;

#[derive(Accounts)]
pub struct Initialize {
    #[account(mut)]
    pub payer: Signer,
    #[account(init, payer = payer)]
    pub counter: Account<Counter>,
    pub system_program: Program<System>,
}
```

Este no es el "hello world" vacío que escribían las plantillas de 0.x. V2 genera el scaffold de un counter chico: una instrucción que crea una cuenta y la pone en cero. `declare_id!` declara la dirección on-chain del programa. `#[program]` marca el módulo de los handlers de instrucciones. `initialize` abre un `Counter`, pone su count en cero, y estampa al payer como su authority. Eso es R0: no porque haga nada interesante, sino porque es la cosa completa más chica que tu toolchain puede construir, desplegar y demostrar.

Cuatro detalles de ahí adentro van a parecer equivocados si cargas memoria muscular de 0.x o de 1.0, y cada uno es un cambio real de V2: el handler toma `&mut Context<T>` en vez de un context por valor; la struct de accounts y sus wrappers no cargan ningún lifetime `<'info>`; el tipo de dirección es `Address`, no `Pubkey`, y lo lees con `.address()`; y `init` no nombra ningún `space`, porque V2 dimensiona la cuenta a partir de su tipo. Hoy es una caja negra a propósito: la próxima lección destapas estas macros y lees exactamente qué generan.

Ahora arregla cómo el crate del programa consigue la RC. Abre `programs/greeter/Cargo.toml`. La plantilla **no** fija una versión de crates.io; apunta a la misma rama desde la que instalaste el CLI, y se deja una nota al respecto (frescura 2026-08-22):

```toml
[dependencies]
# Once anchor-lang is published to crates.io, swap to: anchor-lang = "2.0.0-rc.1"
anchor-lang = { git = "https://github.com/otter-sec/anchor.git", branch = "anchor-next" }
solana-program-log = { version = "1.1", features = ["macro"] }
```

Ese comentario generado es una pequeña cápsula de tiempo que vale la pena leer, y también es tu instrucción. La plantilla dice "once anchor-lang is published to crates.io," y *sí* lo fue, el 2026-08-12. Haz exactamente lo que dice el comentario. Cambia esa línea a:

```toml
[dependencies]
anchor-lang = "2.0.0-rc.1"     # crates.io; a published version is immutable
solana-program-log = { version = "1.1", features = ["macro"] }
```

Dos líneas que la plantilla **no** escribe todavía, y sin las cuales nada en este curso compila. Agrégalas a la misma tabla `[dependencies]` antes de tu primer build:

```toml
# #[program] expands absolute ::wincode:: paths, so the serializer must be a direct dep
wincode = { version = "0.5", features = ["derive"] }
solana-address = "=2.6.0"      # rc.1 pins wincode 0.5; solana-address 2.7.0 moved to 0.6
```

Esas tres filas son el conjunto de pins que carga cada crate de programa en este curso, y son la razón de que la fila de `anchor-lang` tuviera que salir de la rama. El greeter es un workspace de uno, así que el `=2.6.0` exacto enuncia bien la restricción acá; desde m02-l1 en adelante, donde los programas comparten un workspace y cargo tiene que resolver un solo `solana-address` para todos, esa fila se escribe como el rango `">=2.6.1, <2.7"` en su lugar. El mismo techo, enunciado para que más de un crate pueda estar de acuerdo con él — m02-l1 hace el argumento. Intenta la otra forma y cargo no va a llegar ni a compilar:

```
error: failed to select a version for `solana-address`.
    ... required by package `anchor-lang v2.0.0-rc.1 (https://github.com/otter-sec/anchor.git?branch=anchor-next#a6510ad7)`
versions that meet the requirements `^2.7.0` are: 2.7.0
all possible versions conflict with previously selected packages.
  previously selected package `solana-address v2.6.0`
```

Esa es la clase de bug de #4937 otra vez, en vivo sobre un resolve fresco al momento de escribir esto. El `anchor-lang` de la punta de la rama ahora exige `solana-address 2.7.0`, que arrastra `wincode 0.6`; el crate `2.0.0-rc.1` del registro — construido desde el tag `v2.0.0-rc.1` — no exige ninguno de los dos, así que `wincode 0.5` y `solana-address 2.6.0` se mantienen. Dos majors de wincode en un solo grafo es una pared de errores `SchemaRead`/`SchemaWrite` "is not satisfied" cuando llega a resolver, y saltarse la dep directa de `wincode` quiere decir que la expansión de `#[program]` no puede ni nombrar su serializador (`error[E0433]: could not find wincode in the list of imported crates`).

Así que la división es deliberada, y vale la pena enunciarla como una regla antes que como una solución alternativa. **El CLI viene de git; la librería viene de crates.io.** Son grafos de dependencias separados — el binario `anchor` de `~/.cargo/bin` se linkeó una vez y nunca participa en el resolve de tu programa — así que fijarlos a refs distintos del mismo release no es desalineación, es precisión. Y el pin del registro es el más fuerte de los dos: crates.io prohíbe republicar una versión, así que `2.0.0-rc.1` son bytes que no pueden cambiar, mientras que una rama es un nombre que apunta a donde alguien empujó por última vez. Y `anchor --version` imprime `2.0.0-rc.1` desde la punta de la rama *y* desde el tag, así que la cadena de versión nunca te va a decir en cuál de las dos estás. El ref es el pin. La versión es solo una etiqueta.

Carga las tres filas en cada crate de programa que escribas en este curso, y reverifícalas como reverificas cada pin de la RC: dejan de ser necesarias el día que la RC reconcilie su propio grafo.

**5. Constrúyelo.** Desde la raíz del workspace:

```bash
anchor build
```

En macOS, si el build del programa mismo se topa con la misma pared de LTO, ponle el prefijo igual: `CARGO_PROFILE_RELEASE_LTO=off anchor build`. Un build limpio escribe el programa compilado en `target/deploy/greeter.so` y un par de claves en `target/deploy/greeter-keypair.json`. Dos artefactos, dos trabajos. El `.so` es tu programa compilado al formato de bytecode on-chain, la cosa de verdad que va a correr dentro del runtime; desplegar no es nada más que subir esos bytes a una cuenta y marcarla como ejecutable. El par de claves es la identidad on-chain de tu programa: su clave pública es la dirección a la que van a llamar otras transacciones, y su clave secreta es la autoridad que te deja hacerle upgrade después a los bytes desplegados. Cuida el par de claves. Piérdelo y nunca vas a poder volver a hacerle upgrade a este programa, solo desplegar uno nuevo en una dirección nueva.

El primer `anchor build` sobre la RC es también el paso más lento del setup, porque cargo está compilando el framework entero de Anchor desde el código fuente, no bajando un crate precompilado. Ese es el costo del canal git. Los builds siguientes son rápidos; solo el primero paga el precio completo.

**6. Apunta Anchor a devnet y fondea una billetera.** Pon el CLI en devnet, asegúrate de tener un par de claves, y hazte un airdrop de algo de SOL de devnet para pagar el deploy:

```bash
solana config set --url devnet
solana address                 # your deployer wallet; solana-keygen new if you have none
solana airdrop 2               # devnet SOL; retry if the faucet is rate-limited
solana balance
```

Después sincroniza el program id para que `declare_id!` y `Anchor.toml` coincidan con el par de claves que acabas de construir:

```bash
anchor keys sync
```

`anchor keys sync` lee `target/deploy/greeter-keypair.json`, deriva su clave pública, y reescribe tanto el `declare_id!` de `lib.rs` como la dirección bajo `[programs.devnet]` en `Anchor.toml` para que coincidan. Si te lo saltas, `declare_id!` sigue guardando el id placeholder de la plantilla y el deploy no va a cuadrar. Corre `anchor build` una vez más después de sincronizar para que el binario compilado cargue el id corregido.

**7. Despliega R0 a devnet. Este paso es tuyo.** Todo lo de arriba te lo fui guiando paso a paso. Este lo corres y lo lees por tu cuenta:

```bash
anchor deploy --provider.cluster devnet
```

![El build emite el .so, keys sync alinea los program ids, deploy imprime un Program Id, y un explorador de devnet confirma que resuelve como ejecutable.](assets/v07-flowchart.webp)

El éxito se ve como las palabras **Deploy success** y una línea que dice `Program Id:` seguida de una cadena base58. Esa cadena es la dirección de tu greeter en devnet. Cópiala a la fila `R0 greeter program id` de `PINS.md`, con la fecha de hoy en la columna verified. Después pégala en cualquier explorador de devnet y confirma que la cuenta resuelve como un programa ejecutable. Esa resolución es tu Checkpoint. Si el explorador muestra un programa ejecutable en tu id, R0 está vivo y tu toolchain aislado de la RC funciona de punta a punta.

Si el deploy falla por falta de fondos, el airdrop no aterrizó o fue muy chico, así que vuelve a correr `solana airdrop 2` y revisa `solana balance` antes de intentar de nuevo. Si falla por un program id que no coincide, te saltaste `anchor keys sync` o no reconstruiste después. Arregla esa única cosa y vuelve a desplegar. Todo lo demás que podría salir mal en este punto rastrea de vuelta a la cuestión del PATH del paso 3: el `anchor` equivocado es el que está desplegando.

## Challenge

Tu criterio es simple de enunciar y o pasa o no pasa.

**Completion, trabajado conmigo:** `PINS.md` existe y las cuatro filas que el recorrido de la instalación acaba de verificar — el CLI de anchor, Rust, la solución alternativa de macOS, y el CLI de Solana — están llenas con los valores que de verdad instalaste, no con los de esta página. Eso quiere decir que `anchor --version` de verdad imprimió `2.0.0-rc.1`, que tu Rust de verdad es 1.89.0, que la línea de la solución alternativa de macOS está registrada si estás en una Mac, y que la fila del CLI de Solana está etiquetada como un pin de CI. El greeter genera su scaffold y `anchor build` produce un `greeter.so`.

**Solo, solamente tuyo:** despliega R0 a devnet, pega su program id de vuelta en la última fila de `PINS.md`, y ponle la fecha de frescura a cada pin.

**Aceptación:** `anchor --version` imprime la RC, el greeter se despliega, y el program id resuelve como un programa ejecutable en un explorador de devnet. Tres hechos, todos verificables. Si las tres se cumplen, te ganaste tu primer deploy de V2 sobre un toolchain que casi nadie en el ecosistema está corriendo todavía.

Una cosa para quedarte pensando mientras construye. Ahora estás siguiendo dos líneas de Anchor a la vez, 1.1.2 y `anchor-next`, y la de frontera se te va a ir corriendo por debajo de tus pins. Esa deriva es el trato que hiciste, no un bug en tu setup. La comodidad a la que renunciaste, un solo `avm install` bendecido que simplemente funciona, la cambiaste por llegar semanas antes a V2. El precio de ese canje es la columna `verified`, y lo pagas volviendo a correr `anchor --version` y releyendo tus pins con un cronograma en vez de confiar en ellos para siempre.

![Dos disparadores alimentan un loop de observar-y-después-estampar que reescribe la fecha verified en PINS.md cada vez que un humano vuelve a chequear la RC en movimiento.](assets/v08-flowchart.webp)

Vuelve real ese cronograma, porque una intención vaga de "chequear a veces" es cómo se pudre un archivo de pins. Una cadencia que funciona sobre una RC: volver a correr `anchor --version` al inicio de cualquier sesión donde un build de golpe se comporte distinto de como se comportaba ayer, y reconstruir la RC desde `anchor-next` cuando las notas de release del proyecto o un build roto te digan que la rama se movió. Cuando reverificas, no confías en la fecha que ya está en el archivo. Vuelves a observar el valor y estampas la fecha de hoy, aun si el valor no cambió, porque una fecha fresca sobre un valor sin cambios es información en sí misma: dice que alguien miró. El issue #4937, el desajuste entre `wincode` y `solana-address` que rompió `#[account(borsh)]` y se cerró el 2026-08-20, es todo el argumento en un solo bug. Una dependencia dos niveles más abajo se movió, y la única defensa fue `--locked` más un humano que volvió a chequear. En la estable puedes ser flojo con esto. En la frontera el re-chequeo es el trabajo.

La razón más profunda para fijar exacto, en vez de seguir un "latest" flotante, es una de cadena de suministro que vale la pena nombrar, ya que un artefacto de release que falta es lo que arrancó todo este desvío. Cada vez que instalas desde un blanco en movimiento estás confiando en lo que sea que ese blanco resulte ser en ese instante. Una versión fijada con un canal registrado y una fecha es una afirmación que puedes auditar después: este build exacto, desde esta rama exacta, verificado este día. Un release cortado por lo menos te daría un artefacto inmutable al que apuntar; una rama no te da nada más que el commit que te tocó bajar. Así que te quedas con la versión humana de la misma garantía: escríbelo, féchalo, reverifica.

## Dónde aterriza esto

Instalaste una release candidate que el instalador oficial se niega a tocar, escribiste cada pared con una fecha al lado, y desplegaste un programa que abre una cuenta en devnet. La instalación te peleó y ganaste, que es la única forma en que termina esa pelea una vez que sabes que la RC vive en su propia casa.

El greeter es una caja negra ahora mismo. Construye y se despliega, y no tienes idea, mecánicamente, de cómo `declare_id!` y `#[program]` convirtieron una docena de líneas de Rust en una cuenta ejecutable en devnet. Eso es lo que viene. En la lección que sigue justo después destapas esas dos macros y lees qué generan, forma por forma, y el greeter deja de ser magia. Hiciste la parte difícil. El toolchain es real, el deploy es real, y el id de tu archivo de pins es tuyo.

Nos vemos la próxima lección, con el id en la mano. Entrégalo primero.
