# El vault paga: firmar una CPI como PDA

La lección pasada le diste a R2 una regla propia. Escribiste `quarters::min_balance` como un `AccountConstraint` real, y ahora `#[account(quarters::min_balance = 100)]` rechaza un vault con fondos insuficientes en tiempo de constraint, antes de que tu handler llegue a correr. El vault puede tener crédito y proteger crédito. Lo que nunca ha hecho, ni una vez en todo este módulo, es devolverle el dinero a nadie.

Esa es la brecha que cerramos hoy. Y hay un acertijo escondido en ella. El vault es un PDA. Un PDA no tiene clave privada, por construcción, así que no puede firmar nada por la vía normal. Entonces, cuando un retiro mueve lamports fuera del vault, ¿quién firma? El jugador no: no es su cuenta para gastar desde ahí. Tú tampoco, que guardas algún secreto: no hay secreto que guardar. La respuesta es el programa mismo, y hacer que el programa firme por su propio PDA es la lección entera.

Antes de todo eso, haz lo que vuelve concreto el peligro. El retiro debita un saldo, y un débito es una resta, y una resta sobre un `u64` es la trampa más sondeada de todas en un camino de custodia. Mete esto en un archivo scratch y córrelo:

```rust
// scratch.rs - the debit an attacker will aim at
use std::hint::black_box;

fn main() {
    // black_box hides the values from const-eval, exactly as a real instruction's
    // arguments would; without it rustc sees the overflow at compile time and
    // refuses to build, which is not the failure we are hunting.
    let balance: u64 = black_box(50);
    let amount: u64 = black_box(100); // someone asks to withdraw more than they have
    let new_balance = balance - amount; // the naive debit
    println!("new balance: {new_balance}");
}
```

```bash
rustc --version              # any stable rustc; 1.93.1 on my machine
rustc scratch.rs -o scratch && ./scratch
```

Un build por defecto de `rustc` entra en panic: `attempt to subtract with overflow`. Eso parece el resultado seguro, y es el *menos* malo. Construye el mismo archivo con `rustc -O` y el panic desaparece: imprime `new balance: 18446744073709551566`, con la resta habiendo hecho wrap en silencio hasta un `u64` cerca de su techo. On-chain, un panic de debug aborta tu instrucción con un log confuso, y un wrap de release le entrega al llamante un vault que ahora cree ser dueño de dieciocho trillones de lamports. Ten presente esa línea que falla. Cada guarda del camino de retiro existe para asegurar que esa resta nunca se alcance con `amount > balance`.

## Resumen

- Un PDA está fuera de la curva ed25519 y no tiene clave privada. El runtime deja que el **programa dueño** firme por él presentando los seeds exactos del PDA más su bump canónico a `invoke_signed`. Eso es la firma por PDA: una firma sintética que el programa autoriza, ningún par de claves en ninguna parte.
- En Anchor V2 la forma de la CPI cambió. `CpiContext::new(program: &Address, accounts)` toma el programa de destino como un `&Address`, y las cuentas llegan como handles **`CpiHandle`** con seguimiento de borrow desde `.cpi_handle()` / `.cpi_handle_mut()`. El signer del PDA lo adjuntas con `.with_signer(signer_seeds)`.
- Hay una regla dura de propiedad debajo de todo esto: el System Program solo puede hacer `transfer` de lamports desde una cuenta de la que es *dueño*. Un PDA que carga datos (tu `Account<Vault>`) es propiedad de tu programa, así que una transferencia del System desde él falla. Los lamports tienen que vivir en un PDA aparte, propiedad del System, por el que el programa firma.
- El bump con el que firmas es el bump canónico **guardado**, leído del estado de la cuenta, nunca recalculado. Recalcularlo en cada llamada es a la vez un costo de CU y un riesgo de corrección.
- El débito es `checked_sub`, y el retiro se controla *antes* de que la CPI se dispare: ninguna solicitud de cero, ningún retiro excesivo, y nunca una caída por debajo del piso exento de renta del vault.

El repliegue de la ayuda: en el Lab escribo contigo toda la llamada `CpiContext::new` más `invoke_signed`, cada línea. En el problema Completion rellenas apenas dos líneas de memoria, el array de signer seeds y el débito verificado. En el problema Solo implementas `resolve_withdrawal`, la guarda pre-CPI, desde una especificación, sin apoyo.

## Firmar por una cuenta que no tiene clave

### Quién firma cuando no hay clave privada

Arranca desde el modelo que ya tienes. Una cuenta normal de Solana es un par de claves: quien tiene la clave privada puede firmar una transacción que gaste desde ella. Así funciona la billetera del jugador. Un PDA rompe ese modelo a propósito. Es una dirección derivada del ID de tu programa y de un conjunto de seeds, empujada deliberadamente *fuera* de la curva ed25519 para que ninguna clave privada pueda corresponderle nunca. Esa es la característica, no una limitación: un vault sin clave es un vault cuya clave no se puede robar, ni sacar con phishing, ni filtrar.

Pero un vault que puede guardar fondos y nunca moverlos es una alcancía que tienes que romper. Así que el runtime ofrece un canje. Cuando tu programa llama a `invoke_signed` y entrega los seeds exactos más el bump canónico que se usó para derivar una de las cuentas de la llamada, el runtime vuelve a derivar la dirección desde esos seeds y el ID de tu programa. Si coincide, el runtime marca esa cuenta como signer por la duración de la llamada. No pasa nada de criptografía. Es un permiso que el runtime otorga porque solo el programa dueño de esos seeds pudo haberlos presentado. El PDA "firma" como firma un gerente por la cuenta de una empresa: no con su propia identidad, sino demostrando que está autorizado a actuar por ella.

![Una billetera firma con su clave privada, mientras que por un PDA firma su programa dueño a través de invoke_signed, y el intento falsificado de otro programa falla en la re-derivación.](assets/v01-diagram.png)

La última fila de ese diagrama es todo el modelo de seguridad en una línea: una CPI firmada por PDA solo puede llegar a firmar por seeds de los que es dueño *este* programa. No puedes firmar por el PDA de otro programa, y nadie puede firmar por el tuyo. Quédate con esa oración. Es también la frontera exacta de lo que el retiro puede y no puede hacer.

### La trampa que decide todo el diseño

Acá es donde se quema la mayoría de la gente que ya hizo esto en v1, así que lo enfrentamos de frente antes de construir. El plan obvio es: el PDA del vault guarda lamports, el programa firma como el vault, y hace una CPI al System Program para hacer `transfer` de esos lamports al jugador. Limpio. Además no funciona, y la razón vale la pena entenderla porque dicta la forma de todo lo que viene abajo.

El System Program solo va a mover lamports desde una cuenta de la que el System Program es *dueño*. Tu `Account<Vault>` es una cuenta que carga datos: lleva el struct `#[account]` con el dueño, el crédito, el bump. En el momento en que una cuenta carga los datos de tu programa, es propiedad de tu programa, no del System Program. Así que un `transfer` del System con tu vault de datos como origen falla en tiempo de ejecución con un error muy específico: `Transfer: from must not carry data`. De todos los errores con PDA, es el más común, y falla *después* de que escribiste y desplegaste la versión ingenua, que es el peor momento para aprenderlo.

Hay dos formas correctas de mover lamports, y cuál uses depende por completo de quién es dueño de la cuenta de origen:

![Un PDA de datos propiedad del programa no puede ser origen de una transferencia del System y tiene que mover lamports directamente, mientras que un PDA sin datos propiedad del System sí puede, firmado vía invoke_signed.](assets/v02-comparison.png)

Lee la fila de la conclusión, porque fuerza nuestra arquitectura. Esta lección es sobre firmar una CPI como PDA. Ese mecanismo, `invoke_signed` hacia el System Program, solo existe para el caso de propiedad del System. Así que R2 no puede mantener sus lamports dentro del vault de datos. Necesita un segundo PDA: un `SystemAccount` sin datos que guarda el SOL de verdad, uno por el que el programa puede firmar una transferencia del System. El vault de datos se queda como el libro mayor; el nuevo vault de SOL guarda el dinero. Esa división no es complejidad incidental, es la decisión de custodia hacia la que el módulo viene construyendo, y es la razón por la que el vault no podía simplemente "pagar" hasta ahora.

### A R2 le crece un segundo PDA

Así que el quarter-vault después de hoy son dos cuentas bajo cada jugador, derivadas del mismo dueño pero con seeds distintos:

![R2 le da a cada jugador un PDA de estado propiedad del programa que guarda el libro mayor y un PDA de SOL sin datos propiedad del System que guarda los lamports que el programa firma para mover.](assets/v03-diagram.png)

Dos seeds, `b"vault"` y `b"sol"`, mantienen las direcciones distintas para que un jugador tenga las dos. El PDA de estado guarda dos bumps ahora: su propio `bump` (sin cambios respecto de lecciones anteriores) y `sol_bump`, el bump canónico del vault de SOL, que capturamos una vez en la inicialización y reutilizamos para siempre. ¿Por qué guardar el bump del vault de SOL en la cuenta de estado y no recalcularlo en `withdraw`? Esa es la próxima sección, y es donde entran los números de CU.

### La forma de la CPI en V2, recorrida

La CPI de V2 es la forma que vas a escribir toda la lección, así que nombremos cada parte antes de cablearla en un handler. Anchor V2 reconstruyó esta superficie. Dos cosas cambiaron frente a la forma pre-1.0 y frente a la línea 1.0 también: el argumento del programa ahora es un `&Address`, no un `AccountInfo` y no un `Pubkey` por valor, y las cuentas se pasan como handles `CpiHandle` con seguimiento de borrow en vez de structs tipados planos. Esos handles los consigues con `.cpi_handle()` para una lectura y `.cpi_handle_mut()` para una escritura. El signer del PDA viaja junto a través de `.with_signer(signer_seeds)`, que es el wrapper ergonómico sobre el syscall crudo `invoke_signed`.

![Una llamada de transfer de V2 anotada que etiqueta address(), los signer seeds con el bump guardado, el argumento de programa &Address, los handles cpi_handle_mut, y with_signer.](assets/v04-annotated-code.png)

Una advertencia sobre ese bloque, la misma que llevaban las lecciones anteriores: esto es una release candidate, `2.0.0-rc.1` en crates.io al momento de escribir esto. La ergonomía de la CPI es de alta confianza pero todavía se está asentando, así que trata las formas exactas de los tokens como un blanco en movimiento y vuelve a leerlas contra el crate cuando construyas. Los conceptos debajo de ellas, un programa `&Address`, cuentas `CpiHandle`, y `with_signer` para el PDA, son la parte estable.

### invoke o invoke_signed: la bifurcación entre deposit y withdraw

Hay dos formas de hacer una CPI, y la diferencia es una sola cosa: de quién necesita la firma el programa llamado. `invoke` es para una CPI donde cada signer que exige ya firmó la transacción externa. `invoke_signed` es para el caso donde el programa que llama tiene que firmar en nombre de un PDA del que es dueño. Por debajo son el mismo syscall, `sol_invoke_signed_rust`; `invoke` es literalmente `invoke_signed` con un array de seeds vacío. Así que el modelo mental no es "dos funciones", es "una función, y si le pasas seeds o no".

Los dos handlers de dinero de R2 caen en lados opuestos de esa bifurcación, y por eso construir los dos en una lección vale el handler extra. Un `deposit` mueve lamports del jugador *hacia* el vault de SOL. El jugador es dueño del origen y ya firmó la transacción, así que la transferencia del System no necesita firma extra: `invoke` pelado, un `CpiContext::new` sin nada adjunto. Un `withdraw` mueve lamports *fuera* del vault de SOL, cuyo origen es un PDA sin clave que no firmó nada, así que el programa tiene que aportar los seeds y dejar que el runtime le otorgue al PDA su firma sintética: `invoke_signed`, expresado como `.with_signer(signer_seeds)`.

![Deposit usa invoke porque el jugador ya firmó, mientras que withdraw usa invoke_signed porque su origen es un PDA sin clave por el que el programa firma con seeds.](assets/v05-comparison.png)

Hay una regla de privilegios que vale la pena interiorizar mientras estás acá, porque es la barrera de protección del runtime que hace que la firma por PDA sea segura de exponer. Una CPI nunca puede escalar privilegios: si el que llama no tenía una cuenta como escribible o como signer, el programa llamado no puede inventar ese privilegio. La única excepción sancionada es exactamente la firma por PDA: el runtime va a agregar un PDA al conjunto de signers, pero solo cuando el programa que llama presenta seeds que se re-derivan a ese PDA bajo el ID propio del programa que llama. Esa es toda la razón por la que una cuenta sin clave es segura para darle poder de gasto. La autoridad no es un secreto que puede filtrarse, es una derivación que solo el programa dueño puede producir.

### El bump se guarda, no se recalcula

Los signer seeds llevan `ctx.accounts.state.sol_bump`, un byte que leemos del estado de la cuenta. No llamamos a `find_program_address` dentro de `withdraw` para volver a descubrirlo. Vale la pena ser preciso acá, porque "funciona de las dos formas" es verdad y engañoso al mismo tiempo.

Un bump canónico lo encuentra `find_program_address`, que arranca en el bump `255` y camina hacia abajo, llamando a `create_program_address` en cada paso hasta caer en una dirección que está fuera de la curva. Cada uno de esos intentos de derivación es un syscall con un precio real de CU, y la caminata puede tomar un intento o muchos. Guarda el bump una vez en el init y `withdraw` se salta la búsqueda por completo: presenta el byte conocido como bueno y el runtime hace una sola re-derivación para verificarlo. Recalcula el bump en cada llamada y pagas por la búsqueda entera, en cada llamada, para siempre, en un camino caliente. El ahorro es real pero es un *rango*, no un número fijo, porque depende de cuántos bumps tenga que probar la búsqueda y de los costos por syscall que el runtime tiene hoy.

Ese último punto no es palabrería, y es la razón por la que este curso cita los ahorros de CU como rangos y nunca los congela. Los benchmarks de Anchor V2 se volvieron más honestos con el tiempo:

![Una línea de tiempo donde las afirmaciones de titular de Anchor V2, el 95 por ciento y 9.9x, se revisan a la baja hasta el 94 por ciento y 8.8x por el PR 4914 el 2026-08-13.](assets/v06-timeline.png)

La lección del PR #4914 no es que Anchor se puso más lento. Es que un número de titular que un mantenedor corrigió una vez va a volver a ser corregido, así que la forma honesta de hablar de una ganancia de CU es como un rango que vuelves a medir en tu propio programa, no como un trofeo que citas. Guardar el bump es sin ambigüedad más barato que recalcularlo; el delta exacto te toca perfilarlo a ti.

Hay también un filo de corrección, no solo costo. `find_program_address` siempre devuelve el bump *canónico* (el más alto), pero una derivación hecha a mano con el bump equivocado puede caer en una dirección válida-pero-no-canónica, y un programa que a veces firma por una dirección distinta de la que guarda es un bug sutil, horrible. Guardar el único bump canónico en el init y reutilizarlo elimina la clase entera. Guarda el bump.

### El trade-off, dicho sin vueltas

La firma por PDA hace del programa la autoridad sobre el dinero del vault. Eso es exactamente tan poderoso, y exactamente tan peligroso, como suena. Cada camino de retiro que expones es un camino que un atacante va a sondear: una llamada de monto cero para ver qué pasa, un retiro excesivo para cazar esa resta sin verificar, una solicitud dimensionada para drenar el vault de SOL por debajo de su piso exento de renta y cerrar la cuenta en silencio por debajo del libro mayor. El programa es lo único que se interpone entre "el vault le paga a la persona correcta el monto correcto" y "el vault le paga a quien pregunte". Así que las guardas no son cortesía. Son la frontera de seguridad.

Hay una propiedad que juega a tu favor acá, y vale la pena nombrarla para que te apoyes en ella correctamente. Una instrucción es atómica: si el handler devuelve un error en cualquier punto, cada cambio que hizo, incluida una CPI que ya corrió, se revierte. Así que el orden en `withdraw`, primero la transferencia y después debitar el libro mayor, es seguro aunque parezca riesgoso. Si el `checked_sub` sobre el libro mayor falla de algún modo después de que la transferencia salió bien, toda la instrucción aborta y la transferencia se desenrolla con ella. Nunca terminas en el estado a medias donde los lamports salieron pero el libro mayor no lo registró. Lo que la atomicidad *no* hace es salvarte de un débito *sin verificar* que hace wrap en vez de dar error: un wrap silencioso no es una falla, así que nada se revierte, y el vault se queda creyendo una mentira. La atomicidad te protege de errores, no de bugs que nunca levantan uno. Eso es todo el caso a favor de `checked_sub` sobre `-` en tres palabras: convierte el bug en un error.

![checked_sub levanta un error así que toda la instrucción se revierte y el vault se mantiene correcto, mientras que la resta pelada, en un build con overflow-checks apagados, hace wrap en silencio y deja la transferencia hecha.](assets/v07-comparison.png)

Dos límites duros viajan junto, y los dos son restricciones que tomas como dadas en vez de pelear. Primero, la frontera de seeds que muestra el diagrama de apertura: una CPI firmada por PDA solo puede firmar por seeds de los que es dueño este programa, así que `withdraw` puede mover los lamports del vault de SOL y nada más. Segundo, la profundidad de la CPI. La altura máxima de la pila de instrucciones es 5, lo que quiere decir que un programa puede anidar CPIs hasta 4 niveles de profundidad. SIMD-0268 (estado Accepted) sube el límite de anidamiento de 4 a 8, una altura de pila de 9, pero su feature gate `6TkHkRmP7JZy1fdM6fg5uXn76wChQBWGokHBJzrLB3mj` todavía no tiene cuenta en mainnet al 2026-08-22, así que trata el 5 como la ley y vuelve a sondear el feature gate en tiempo de build en vez de confiar en un estado que cacheaste. Nuestro retiro es de una CPI de profundidad, ni cerca del techo, pero el número importa en el momento en que tu programa llama a un programa que llama a un programa.

Vale la pena alejarse un compás antes de construir, porque esta instrucción es la línea donde todo el módulo deja de ser un juguete. Un programa que solo puede recibir deposits y leer saldos es un demo, y un demo es algo que muestras una vez y a lo que nunca le confías nada que importe; en el momento en que un programa puede mover valor hacia afuera bajo su propia autoridad, se vuelve algo en lo que un desconocido puede apoyarse sin haberte conocido nunca, y esa es toda la promesa de poner la custodia en una blockchain en vez de en una empresa. Todo lo que construiste hasta acá, el contador, el vault, los bumps guardados, el constraint personalizado, viene ensamblando en silencio la única capacidad que hace que algo de esto valga la pena desplegar: la capacidad de guardar el dinero de alguien y devolverlo correctamente, demostrablemente, sin un humano en el loop que pudiera tomarlo o perderlo. Esa capacidad es también precisamente la que un atacante más quiere romper, y por eso la guarda sin glamour que estás por escribir, tres comparaciones y una resta verificada, lleva más del peso real del programa que cualquier feature que le agregues encima.

## Lab: paga desde el vault

Estás extendiendo R2, el programa `quarter_vault`, con una instrucción `withdraw` que firma una transferencia del System como el PDA del vault de SOL y debita el libro mayor con matemática verificada. Cuando terminas, `anchor test` está verde: un retiro firmado por PDA mueve lamports del vault de SOL al jugador y debita `credit`, y un retiro excesivo devuelve un error en vez de entrar en panic. Acá está la forma del handler que estás construyendo, para que los pasos tengan dónde aterrizar:

![Un diagrama de flujo de withdraw donde la guarda rechaza las solicitudes de cero, las de retiro excesivo y las que caen por debajo del piso de rent antes de que corra la CPI de transfer firmada por PDA y se debite el libro mayor.](assets/v08-flowchart.png)

**1. Fija el toolchain de V2.** La release candidate de V2 no llega por `avm install`: ese comando descarga un binario precompilado desde el GitHub Release del tag, no se cortó ningún Release para el tag v2, y el fetch da 404, exactamente como lo mostró la lección de toolchain (m01-l2). El canal documentado es una instalación por cargo git, fijada al tag `v2.0.0-rc.1` en vez de a la punta de la rama `anchor-next` sobre la que se apoya. Si hiciste las lecciones anteriores de este módulo ya lo tienes; si no, instálalo y confírmalo. No construyas contenido de V2 sobre un binario `anchor` de V1:

```bash
# macOS, if the build trips on LTO: prefix with CARGO_PROFILE_RELEASE_LTO=off
cargo install --git https://github.com/otter-sec/anchor.git \
  --tag v2.0.0-rc.1 anchor-cli --locked --force
anchor --version   # must report 2.0.0-rc.1 (the RC as of 2026-08-12; re-check for a newer rc/stable), not a 1.x line
```

**2. Extiende el estado del vault con el bump del vault de SOL, y renombra dos campos mientras estás ahí.** Abre el `lib.rs` de R2.

Dos renames mecánicos primero, porque el vault ya no es solo de un jugador. Ahora tiene un dueño que hoy podría ser un jugador y mañana un programa de escrow, y ahora tiene dos cuentas en vez de una, así que "el vault" es ambiguo. En cada struct de cuentas del programa, renombra el campo `player: Signer` a `authority`, y el campo `vault: Account<Vault>` a `state`. El *tipo* de cuenta se queda como `Vault`; solo se mueven los nombres de los campos. Los builders de tu archivo de pruebas nombran esos campos, así que va a dejar de compilar hasta que renombres ahí también, y eso es el compilador haciendo tu migración por ti.

Después el estado mismo. El `Vault` de las lecciones anteriores guardaba el dueño, un saldo `credit`, su propio `bump`, y el relleno de cola explícito que lo mantiene Pod. Agrega un campo, `sol_bump`, el bump canónico del PDA que guarda el SOL, para que `withdraw` pueda reconstruir los signer seeds sin una búsqueda. Sale del relleno, así que el tamaño de la cuenta no cambia:

```rust
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

// Still the id anchor init generated for you.
declare_id!("<your generated program id>");

#[account]
#[repr(C)]
#[derive(InitSpace)]
pub struct Vault {
    pub owner: Address, // 32 bytes: whoever owns this vault
    pub credit: u64,    //  8 bytes: the withdrawable ledger balance
    pub bump: u8,       //  1 byte: this state PDA's canonical bump
    pub sol_bump: u8,   //  1 byte: the SOL vault PDA's canonical bump, stored at init
    pub _pad: [u8; 6],  //  6 bytes: explicit tail padding (was 7, sol_bump took one)
}
```

**2b. Enséñale a `init_vault` sobre el segundo PDA.** Todavía nada escribe `sol_bump`, y un cero ahí es el peor tipo de bug: cada `withdraw` reconstruye los signer seeds contra un bump que nunca fue canónico, el runtime se niega a marcar el PDA como signer, y el error no dice nada sobre por qué. `init_vault` también tiene que traer el vault de SOL a la existencia, porque un PDA `SystemAccount` que ninguna instrucción creó nunca es apenas una dirección vacía. Los dos trabajos son una sola edición:

```rust
pub fn init_vault(ctx: &mut Context<InitVault>) -> Result<()> {
    let bump = ctx.bumps.state;
    let sol_bump = ctx.bumps.sol_vault;   // the second PDA's canonical bump
    let state = &mut ctx.accounts.state;
    state.owner = *ctx.accounts.authority.address();
    state.credit = 0;
    state.bump = bump;
    state.sol_bump = sol_bump;            // store it once, sign with it forever
    Ok(())
}

#[derive(Accounts)]
pub struct InitVault {
    #[account(mut)]
    pub authority: Signer,
    #[account(
        init,
        payer = authority,
        space = Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE,
        seeds = [b"vault", authority.address().as_ref()],
        bump,
    )]
    pub state: Account<Vault>,
    // The money side: zero data, System-owned, created here so it exists to be
    // signed for later. `space = 0` plus the explicit `owner` are what keep it a
    // legal System transfer source.
    #[account(
        init,
        payer = authority,
        space = 0,
        owner = System::id(),
        seeds = [b"sol", authority.address().as_ref()],
        bump,
    )]
    /// CHECK: typed UncheckedAccount because SystemAccount has no init path in V2,
    /// and UncheckedAccount is the one wrapper `init` may hand to a foreign owner.
    /// The `owner = System::id()` line does that handoff; without it, `init`
    /// defaults the owner to THIS program, and every later SystemAccount read of
    /// this PDA fails at load with IllegalOwner.
    pub sol_vault: UncheckedAccount,
    pub system_program: Program<System>,
}
```

Fíjate de dónde viene cada bump: `ctx.bumps.state` y `ctx.bumps.sol_vault`, un campo tipado por PDA en el struct, los dos provistos por la macro porque los dos escribieron un `bump` pelado. Ese es el único lugar de este programa donde alguna vez se busca un bump. Todo lo que viene aguas abajo lee el byte guardado.

**3. Demuestra la matemática del débito en aislamiento.** Antes de tocar la CPI, deja correcta la versión *segura* del scratch de apertura, porque el débito verificado es una de las dos líneas que el problema Completion te devuelve. El `balance - amount` ingenuo entraba en panic o hacía wrap; `checked_sub` devuelve `None` en el underflow así que puedes convertirlo en un error limpio:

```rust
// scratch.rs - the safe debit
fn debit(balance: u64, amount: u64) -> Result<u64, &'static str> {
    balance.checked_sub(amount).ok_or("underflow: over-withdraw")
}

fn main() {
    assert_eq!(debit(100, 40), Ok(60)); // normal
    assert_eq!(debit(50, 100), Err("underflow: over-withdraw")); // rejected, no panic
    println!("checked debit ok");
}
```

```bash
rustc scratch.rs -o scratch && ./scratch   # prints: checked debit ok
```

Con eso el débito queda cerrado. `None` en el underflow, mapeado a un error, nunca un panic y nunca un wrap. En el handler real el error es un error de programa, no un `&str`, pero la lógica es exactamente esta.

**4. Agrega las variantes de error.** Un programa recibe un solo espacio de errores basado en 6000, y un segundo enum `#[error_code]` compila en verde mientras numera en silencio sus variantes dentro de ese mismo rango — una colisión de offset, no un error de compilación — así que extiende el `VaultError` que ya tienes en vez de agregar un segundo enum. Conserva cada variante de las últimas dos lecciones, en orden, y agrega las nuevas al final: agregar al final importa, porque las variantes se numeran desde 6000 por posición y reordenarlas renumera en silencio errores sobre los que tus pruebas ya afirman. `withdraw` necesita cinco razones nuevas para negarse:

```rust
#[error_code]
pub enum VaultError {
    // --- already yours, from m03-l2 and m03-l3. Do not reorder. ---
    #[msg("caller is not the configured arcade authority")]
    Unauthorized,
    #[msg("credit addition overflowed")]
    Overflow,
    #[msg("account is owned by the wrong program")]
    WrongOwner,
    #[msg("vault credit is below the quarters::min_balance floor")]
    BelowFloor,
    // --- new today ---
    #[msg("withdrawal amount must be greater than zero")]
    ZeroWithdrawal,
    #[msg("withdrawal exceeds the vault's lamport balance")]
    Overdraw,
    #[msg("withdrawal would drop the SOL vault below its rent-exempt floor")]
    WouldCloseVault,
    #[msg("arithmetic underflow while debiting the ledger")]
    Underflow,
    #[msg("caller is not the owner this vault recorded")]
    NotVaultOwner,
}
```

**5. Escribe el handler `withdraw`.** Este es el núcleo resuelto. La guarda corre primero y rechaza cada solicitud insegura antes de que se mueva un solo lamport. Después los signer seeds se construyen desde los seeds propios del vault de SOL más el `sol_bump` *guardado*. Después la CPI de V2 firma la transferencia del System como el PDA. Después el libro mayor se debita con `checked_sub`. Fíjate que el handler toma `&mut Context<T>`, la firma de V2:

```rust
#[program]
pub mod quarter_vault {
    use super::*;

    pub fn withdraw(ctx: &mut Context<Withdraw>, amount: u64) -> Result<()> {
        // --- guard: refuse every unsafe request before touching lamports ---
        let vault_lamports = ctx.accounts.sol_vault.lamports();
        let rent_exempt_min = Rent::get()?.minimum_balance(0); // SOL vault carries zero data
        require!(amount > 0, VaultError::ZeroWithdrawal);
        require!(amount <= vault_lamports, VaultError::Overdraw);
        // checked_sub even here, where the line above already proved it cannot
        // underflow. The proof is one refactor away from being wrong, and this
        // lesson opened on what a bare `-` does in a release build.
        let remaining = vault_lamports
            .checked_sub(amount)
            .ok_or(VaultError::Overdraw)?;
        require!(remaining >= rent_exempt_min, VaultError::WouldCloseVault);

        // --- the PDA-signed CPI: sign the System transfer AS the SOL vault ---
        // Copy these out of ctx.accounts BEFORE any CPI handle is built. The `*`
        // matters: .address() returns &Address, and the deref makes `owner` an
        // owned copy instead of a live borrow of ctx.accounts.
        let owner = *ctx.accounts.authority.address();
        let sol_bump = ctx.accounts.state.sol_bump;
        let signer_seeds: &[&[&[u8]]] = &[&[b"sol", owner.as_ref(), &[sol_bump]]];
        let cpi = CpiContext::new(
            ctx.accounts.system_program.address(),
            Transfer {
                from: ctx.accounts.sol_vault.cpi_handle_mut(),
                to: ctx.accounts.authority.cpi_handle_mut(),
            },
        )
        .with_signer(signer_seeds);
        transfer(cpi, amount)?;

        // --- debit the ledger with checked math ---
        ctx.accounts.state.credit = ctx
            .accounts
            .state
            .credit
            .checked_sub(amount)
            .ok_or(VaultError::Underflow)?;

        Ok(())
    }

    /// Contrast handler: a deposit needs NO invoke_signed. The player owns the
    /// source, so the player signs the System transfer the ordinary way.
    pub fn deposit(ctx: &mut Context<Deposit>, amount: u64) -> Result<()> {
        let cpi = CpiContext::new(
            ctx.accounts.system_program.address(),
            Transfer {
                from: ctx.accounts.authority.cpi_handle_mut(),
                to: ctx.accounts.sol_vault.cpi_handle_mut(),
            },
        );
        transfer(cpi, amount)?;
        ctx.accounts.state.credit = ctx
            .accounts
            .state
            .credit
            .checked_add(amount)
            .ok_or(VaultError::Overflow)?;
        Ok(())
    }
}
```

Mira los dos handlers lado a lado, porque el contraste es la columna vertebral de la lección. `deposit` mueve lamports *hacia* el vault de SOL desde el jugador, y no necesita `with_signer`: el jugador es dueño del origen y firma la transacción externa, así que un `invoke` ordinario (un `CpiContext::new` sin signer adjunto) alcanza. `withdraw` mueve lamports *fuera* del vault de SOL, cuyo origen es un PDA sin clave, así que tiene que adjuntar `signer_seeds` y dejar que el programa firme. La misma transferencia del System, dirección opuesta, y la dirección es lo que decide quién firma.

Un detalle más que vale la pena notar ahora: `owner` y `sol_bump` se copian de `ctx.accounts` hacia locales planas *antes* de que se construya cualquier `cpi_handle`. Ese ordenamiento es estructural en V2, no una elección de estilo, y la próxima lección es enteramente sobre el por qué.

**6. Cablea las cuentas.** `withdraw` necesita el libro mayor de estado, el vault de SOL propiedad del System, el jugador, y el System Program. El vault de SOL es un `SystemAccount` (cero datos, así que una transferencia del System puede tomarlo como origen), y su constraint `bump` reutiliza el `sol_bump` guardado, no una búsqueda fresca. Fíjate que ni el estado ni el vault de SOL se validan recalculando un bump: los dos usan `bump = ...` con el byte guardado.

```rust
#[derive(Accounts)]
pub struct Withdraw {
    #[account(mut, address = state.owner @ VaultError::NotVaultOwner)]
    pub authority: Signer,

    #[account(
        mut,
        seeds = [b"vault", authority.address().as_ref()],
        bump = state.bump,
    )]
    pub state: Account<Vault>,

    #[account(
        mut,
        seeds = [b"sol", authority.address().as_ref()],
        bump = state.sol_bump, // reuse the stored canonical bump, no runtime search
    )]
    pub sol_vault: SystemAccount,

    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct Deposit {
    #[account(mut)]
    pub authority: Signer,
    #[account(
        mut,
        seeds = [b"vault", authority.address().as_ref()],
        bump = state.bump,
    )]
    pub state: Account<Vault>,
    #[account(
        mut,
        seeds = [b"sol", authority.address().as_ref()],
        bump = state.sol_bump,
    )]
    pub sol_vault: SystemAccount,
    pub system_program: Program<System>,
}
```

El `address = state.owner` en la authority ata el signer a la clave exacta que el libro mayor guardó en el init, así que quien llama no puede presentar el libro mayor de otra persona. Este es el reemplazo basado en expresiones de V2 para el `has_one` deprecado de v1, y es la misma disciplina de "atar cada clave guardada" de las lecciones de constraints; en un camino de custodia no es opcional.

Fíjate qué constraint *no* está en `Withdraw`, porque la lección pasada hizo un punto de eso. `quarters::min_balance = 100` sigue en tu programa y sigue en `require_funded`, y está deliberadamente ausente acá. Un piso que bloqueara los retiros atraparía para siempre los últimos 100 créditos de un jugador dentro del vault, que es lo contrario de una garantía de custodia. Esa es la mitad honesta de "el constraint viene gratis en cualquier instrucción que carga el vault": viene en las instrucciones donde tú lo pones, y decidir dónde sigue siendo cosa de tu criterio. `require_funded` controla el *gasto*; `withdraw` devuelve el dinero propio del jugador y solo controla contra los lamports reales del vault.

Checkpoint para los pasos 2 a 6: corre `anchor build`. Compila limpio, sin advertencia de deprecación de `has_one`, sin error de campo faltante en `sol_bump`, y sin nombres de campo `player`/`vault` que queden en los derive structs o en los builders de las pruebas. Una nota de tiempo de ejecución para después: cualquier vault que inicializaste en una lección anterior es anterior a `sol_bump` y es anterior al PDA de SOL por completo, así que su `sol_bump` guardado es lo que hubiera en esos bytes de relleno y su vault de SOL no existe. El tamaño y el contenido de una cuenta quedan fijos en el init, así que esos vaults viejos no se pueden actualizar acá. Las pruebas de abajo crean unos frescos, que es el único camino que esta lección soporta.

**7. Escribe la prueba de LiteSVM.** LiteSVM corre el programa en proceso sin ningún validador, así que el loop es rápido. Agrega las dev-dependencies:

```toml
# The same one row as module 3, for the same reason: anchor-v2-testing owns the SVM
# version. At tag v2.0.0-rc.1 that is litesvm 0.11.0, and you never say so yourself.
[dev-dependencies]
anchor-v2-testing = { git = "https://github.com/otter-sec/anchor.git", tag = "v2.0.0-rc.1" }
```

La prueba financia el vault de SOL a través de `deposit`, retira parte de eso y demuestra que los lamports se movieron y que el libro mayor bajó, después demuestra que un retiro excesivo se rechaza limpiamente en vez de entrar en panic. El movimiento y el rechazo son todo el artefacto:

```rust
use anchor_lang::{
    prelude::Address,
    programs::System,
    solana_program::instruction::{AccountMeta, Instruction},
    Id, InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction};

fn ix(program_id: Address, accounts: Vec<AccountMeta>, data: Vec<u8>) -> Instruction {
    Instruction { program_id, accounts, data }
}

// One place that turns an instruction into a signed, sendable transaction.
fn tx(svm: &LiteSVM, payer: &Keypair, instruction: Instruction) -> VersionedTransaction {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap()
}

#[test]
fn withdraw_moves_lamports_and_rejects_overdraw() {
    let mut svm = anchor_v2_testing::svm();
    let program_id = quarter_vault::ID;
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    svm.add_program_from_file(program_id, vault_so).unwrap();

    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 5_000_000_000).unwrap();
    let (state_pda, _) =
        Address::find_program_address(&[b"vault", authority.pubkey().as_ref()], &program_id);
    let (sol_pda, _) =
        Address::find_program_address(&[b"sol", authority.pubkey().as_ref()], &program_id);

    // init_vault (rewritten in step 2b) creates BOTH PDAs and stores both bumps.
    let init = tx(
        &svm,
        &authority,
        ix(
            program_id,
            quarter_vault::accounts::InitVault {
                authority: authority.pubkey(),
                state: state_pda,
                sol_vault: sol_pda,
                system_program: System::id(),
            }
            .to_account_metas(None),
            quarter_vault::instruction::InitVault {}.data(),
        ),
    );
    svm.send_transaction(init).unwrap();

    // deposit 2 SOL into the SOL vault (player signs; no PDA signature needed).
    let deposit = tx(
        &svm,
        &authority,
        ix(
            program_id,
            quarter_vault::accounts::Deposit {
                authority: authority.pubkey(),
                state: state_pda,
                sol_vault: sol_pda,
                system_program: System::id(),
            }
            .to_account_metas(None),
            quarter_vault::instruction::Deposit { amount: 2_000_000_000 }.data(),
        ),
    );
    svm.send_transaction(deposit).unwrap();
    let funded = svm.get_account(&sol_pda).unwrap().lamports;

    // withdraw 1 SOL: the PDA-signed CPI must move lamports OUT of the SOL vault.
    let withdraw = |svm: &LiteSVM, amount: u64| {
        tx(
            svm,
            &authority,
            ix(
                program_id,
                quarter_vault::accounts::Withdraw {
                    authority: authority.pubkey(),
                    state: state_pda,
                    sol_vault: sol_pda,
                    system_program: System::id(),
                }
                .to_account_metas(None),
                quarter_vault::instruction::Withdraw { amount }.data(),
            ),
        )
    };
    svm.send_transaction(withdraw(&svm, 1_000_000_000)).unwrap();
    let after = svm.get_account(&sol_pda).unwrap().lamports;
    assert_eq!(funded - after, 1_000_000_000, "1 SOL must leave the vault");

    // over-withdraw: asking for more than the vault holds must ERROR, not panic.
    let overdraw = svm.send_transaction(withdraw(&svm, 100_000_000_000));
    assert!(overdraw.is_err(), "an over-withdraw must be rejected cleanly");
}
```

**8. Construye y corre.**

```bash
anchor test
```

Salida esperada, la única prueba que pasa y supera el criterio:

```
running 1 test
test withdraw_moves_lamports_and_rejects_overdraw ... ok

test result: ok. 1 passed; 0 failed
```

La demostración está en las dos aserciones. La primera muestra que salió exactamente un SOL del vault, lo que solo puede pasar si el programa firmó la transferencia como el PDA, porque el vault de SOL no tiene clave y el jugador no es su dueño. La segunda muestra que el retiro excesivo volvió como `is_err()`, con la guarda rechazándolo antes de la CPI, no un panic en el log. Si en cambio el retiro excesivo *entra en panic* en vez de dar error, la causa habitual es el orden de la guarda: tienes que demostrar `amount <= vault_lamports` antes del `vault_lamports - amount`, o la resta hace underflow dentro de la guarda misma. Vuelve a correr el scratch del paso 3 para aislar la matemática del cableado.

## Challenge

Un peldaño lo vuelves a llenar de memoria, otro lo construyes en frío.

**Completion.** Vuelve a abrir `withdraw` y borra las dos líneas que el Lab escribió por ti: el array `signer_seeds` y el débito `checked_sub`. Deja `let signer_seeds: &[&[&[u8]]] = /* TODO */;` y `ctx.accounts.state.credit = /* TODO */;`. Vuelve a llenar las dos de memoria. Los seeds tienen que ser los seeds propios del vault de SOL más su bump *guardado* copiado primero a una local, `&[&[b"sol", owner.as_ref(), &[sol_bump]]]`, y el débito tiene que ser `checked_sub(amount).ok_or(VaultError::Underflow)?`. Si echas mano de `find_program_address` para conseguir el bump, detente: todo el punto es el byte guardado, y recalcularlo cuesta CU y arriesga un bump no canónico. La verificación de aceptación es el `anchor test` del paso 8, todavía en verde.

**Solo.** Extrae la guarda a una función independiente y testeable, `resolve_withdrawal`, y demuéstrala en Rust puro antes de volver a cablearla en el handler. Esta es la guarda pre-CPI, destilada para que se pueda probar unitariamente sin ningún framework:

![Un diagrama de flujo de decisión que devuelve menos uno para una solicitud de cero, menos dos para un retiro excesivo, menos tres por debajo del piso de rent, y el monto solicitado en cualquier otro caso.](assets/v09-flowchart.png)

El starter, la solución, y los vectores de prueba viven en `lessons/m04-l1/resolve-withdrawal/`, junto a los otros challenges de este curso. El starter ignora cada guarda y devuelve `requested` sin condiciones — y porque la función es una `const fn` con aserciones de tiempo de compilación debajo (el dispositivo de m03-l3, que es lo que la calificación solo-por-compilación de verdad exige), la versión sin guardas ni llega a construir: el primer error nombra el caso de la solicitud de cero. Una verruga deliberada para notar en vez de copiar: la firma devuelve centinelas `i64` porque una función pura sin ningún framework en alcance no tiene `VaultError` que devolver, y los vectores se quedan lo bastante chicos como para que el cast `as i64` sea exacto. En el handler se vuelve un `Result` con los errores tipados del paso 4, y si algún día te encuentras entregando códigos centinela desde código de programa real, ese es el olor del que hablaba la sección de errores tipados del módulo 1. Aceptación: los casos de verificación pasan en orden, el retiro excesivo devuelve `-2` en vez de hacer underflow, el piso de rent es *inclusivo* así que un retiro que deja exactamente `rent_exempt_min` se permite y uno con un lamport menos es `-3`, un retiro excesivo que además rompería el piso sigue siendo `-2` porque la verificación del saldo se alcanza primero, y — la parte que importa — tu resta es `checked_sub` en vez de un `-` pelado, exactamente como en el handler — aunque la guarda de arriba ya demostró `requested <= balance`, porque esa demostración está a un refactor de estar equivocada y un `-` pelado hace wrap en silencio en un build de release. Después cambia los tres `require!` de `withdraw` por una llamada a tu monto resuelto y confirma que `anchor test` sigue en verde. Una cosa que vale la pena vigilar: `resolve_withdrawal` protege el movimiento de *lamports* contra el saldo del vault de SOL, mientras que el `checked_sub` protege el *libro mayor*. Son dos saldos distintos haciendo dos trabajos distintos, y un bug de custodia real es dejar que se separen.

Hiciste que el vault haga la única cosa que no podía hacer antes: devolverle el dinero a alguien, bajo la autoridad propia del programa, con una firma que ningún atacante puede falsificar porque no hay clave que robar. Construiste la guarda, firmaste la CPI como el PDA con un bump guardado, y debitaste el libro mayor con matemática que se niega a hacer underflow. Ese es el loop de custodia, cerrado.

Tu retiro funciona. Pero hay una trampa dentro que v1 le puso a miles de programas, y no la pisaste solo porque el Lab nunca volvió a leer el saldo del vault justo después de la CPI. En v1, leer los campos de una cuenta deserializada *después* de que una CPI la mutó te daba datos viejos a menos que te acordaras de llamar a `.reload()`, y olvidarse era una forma clásica de entregar un bug. En V2 el modelo de borrow de `CpiHandle` que usaste hoy es lo que hace que ese mismo error se niegue a compilar. Próxima lección: exactamente cómo, y por qué el compilador es ahora la cosa que te mantiene seguro en vez de una llamada a `.reload()` que tenías que recordar.
