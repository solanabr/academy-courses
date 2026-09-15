# Construye el swap de token-a-ticket

La lección pasada moviste el quarter-vault y el prize-escrow a `token_interface`. Los dos ahora custodian tokens SPL de verdad en vez de lamports, y sus pruebas de withdraw y de release están verdes. Esa es toda la razón por la que esta lección es posible: tienes dos programas que pueden sostener un balance de token y firmar transferencias que salen de él con un PDA. Cablea dos de esos juntos y tienes un mercado.

Así que acá está el dolor. Los jugadores ganan tokens de arcade en la máquina. Quieren tickets para gastar en el mostrador de premios. Alguien tiene que fijar la tasa de cambio. Hardcodea `1 token = 1 ticket` en una constante y construiste un faucet, no un swap: el primer jugador que note que el pool tiene más tickets que tokens canjea el lado barato hasta que las reservas quedan vacías, y lo pagaste tú. La tasa no puede ser una constante. Tiene que salir de las reservas propias del pool y moverse cada vez que alguien canjea.

Antes de que leas otro párrafo, haz la aritmética que un swap hace en cada canje. Córrela, no te fíes de mi palabra. Guarda esto como `curve.py` y corre `python3 curve.py`, sin paquetes, sin instalación, solo el `python3` que ya tienes en la máquina. Cuatro líneas son el canje entero:

```python
def swap_out(reserve_in, reserve_out, amount_in):
    fee_in = amount_in * 997                                  # 0.3% fee, taken before the curve
    return (fee_in * reserve_out) // (reserve_in * 1000 + fee_in)

print("naive:", 10_000 * 1_000_000 // 1_000_000)             # 10000
print("real :", swap_out(1_000_000, 1_000_000, 10_000))      # 9871
```

Ahí está tu brecha de 129 tickets, impresa: una tasa ingenua cotiza 10,000, la curva paga 9,871. Quédate con esa brecha. Para el final de esta lección vas a saber exactamente adónde se fue cada uno de esos tickets, y por qué devolvérselos al trader es como se drenan los pools.

## Resumen

Qué construyes, y la forma de cada pieza, para que puedas hojear antes de excavar:

- **El artefacto es R4, un swap de producto constante.** Dos reservas de token, una cotización computada a partir de esas reservas, y dos transferencias firmadas por PDA por canje. Es el patrón canónico de composición de Anchor: un programa manejando dos cuentas de token que controla.
- **El precio es el invariante.** `x * y = k`. El producto de las dos reservas se mantiene (casi) constante a través de un canje, lo que quiere decir que cuanto más compras, peor se te pone la tasa. Sin oráculo, sin admin, sin ningún número hardcoded. Las reservas *son* el precio.
- **La comisión es de 0.3%, expresada como `997/1000`.** Se resta de la entrada antes de que la curva la vea, y se queda en el pool, lo que empuja `k` hacia arriba en cada canje.
- **La matemática necesita un intermedio `u128`.** Dos reservas `u64` multiplicadas pueden pasarse de `u64`. Promueve, multiplica, divide, haz cast de vuelta hacia abajo. Este es el único lugar donde un swap entra en panic si te equivocas.
- **La guarda de slippage es `min_out`.** El trader declara el peor fill que va a aceptar; el programa revierte si la cotización entra por debajo. Reutiliza la forma exacta del release condicional del escrow en R3. Es la única protección para el trader que ofrece este swap.
- **Las dos reservas son cuentas distintas.** Así que el default de duplicate-mutable de Anchor V2 se satisface gratis. No vas a recurrir a `unsafe(dup)`, y esta lección te muestra por qué recurrir a él acá sería un bug de diseño.

El repliegue de esta lección: yo derivo la curva y te entrego la transferencia de token-in trabajada completa. La transferencia de token-out es un relleno que la refleja. La guarda de slippage es tuya para escribirla en solo, porque para entonces ya habrás visto su gemela en el escrow y no deberías necesitarme para eso.

Una línea de alcance antes de empezar. Este es un swap de enseñanza, un patrón de Anchor, no un protocolo DeFi. Si quieres creación de mercado automatizada en un venue real con profundidad viva de proveedores de liquidez, ese es territorio del curso de DeFi and RWA Engineering. Acá el swap existe para enseñar composición de PDA de dos lados, no para canjear contra él.

## El precio vive en las reservas

Una sola idea genera todo lo que sigue. Un pool de producto constante sostiene dos activos y trata el producto de sus balances como un número que tiene que proteger. Llama a las reservas `x` e `y`. La ley del pool es `x * y = k`, y `k` es (casi) sagrado: todo canje tiene que dejar `k` al menos tan grande como lo encontró.

Esa única regla es el mecanismo de precio. Cuando un trader agrega `dx` del primer activo, el pool tiene que devolver suficiente del segundo activo, `dy`, para que el producto siga valiendo. Despeja `dy` y la tasa no es un número guardado en ningún lado. Es lo que sea que mantenga la curva intacta. Cuanto más profundo el pool, menos lo mueve un canje dado; cuanto menos profundo el pool, más cuesta cada canje. Ese no es un bug que tengas que vigilar. Es la geometría vigilando por ti.

![Un canje jala los tokens de arcade del trader hacia la reserva del pool, cotiza una salida, la verifica contra min_out, y después empuja tickets de vuelta bajo la firma del PDA del pool.](assets/v01-diagram.png)

¿Por qué cotizar desde las reservas en vez de desde una tasa que fijas tú? Porque una tasa que fijas tú es una tasa de la que un atacante puede elegir un lado. Imagínate el pool como un sube y baja curvo con un área fija debajo. Cada canje desliza un peso a lo largo de la viga, y la viga tiene que inclinarse para mantener el área constante. Un trader puede empujar el peso, pero no puede cambiar el área, y el área es de donde tendría que robar. La curva no está protegiendo un precio que elegiste. Es el precio, y se repreció en el instante en que aterrizó el último canje. Esa es toda la razón por la que un AMM no necesita oráculo: el pool se cotiza a sí mismo.

Ahora la fórmula exacta. Sin comisión, mantener `k` constante da:

```
out = (amount_in * reserve_out) / (reserve_in + amount_in)
```

Léela y fíjate en lo que hace por sí sola. A medida que `amount_in` crece en relación con `reserve_in`, el denominador crece también, así que cada token adicional que entra compra menos tokens de salida. Intenta comprar el `reserve_out` entero y el denominador se te escapa; nunca puedes drenarlo con una entrada finita. La curva se niega.

La comisión es un recorte sobre la entrada antes de que la curva la vea. Una comisión de 0.3% quiere decir que el pool se queda con 3 de cada 1000 unidades de entrada y solo 997 de ellas cuentan para el canje:

```
amount_in_with_fee = amount_in * 997
out = (amount_in_with_fee * reserve_out) / (reserve_in * 1000 + amount_in_with_fee)
```

El `1000` del denominador está ahí para mantener todo en la misma escala entera que el `997`, así nunca tocas un float. La matemática on-chain es matemática entera, siempre. Los tokens de la comisión se quedan en `reserve_in`, así que `k` después del canje es apenas más grande que `k` antes. El pool crece. Ese crecimiento es lo que ganaría un proveedor de liquidez de verdad; acá solo quiere decir que la prueba de invariante afirma `k_after >= k_before`, nunca igualdad.

La curva de producto constante como dibujo, porque la forma es la intuición:

![Una cotización lineal ingenua y la curva de producto constante coinciden en canjes chicos pero divergen fuerte a medida que crece el tamaño, con la curva doblándose muy por debajo de la línea.](assets/v02-chart.png)

Esa brecha entre la línea recta y la curva son los 129 tickets de la intro, a mayor escala. En un canje de 10,000 tokens es chica. En un canje de 500,000 tokens contra una reserva de 1,000,000 la tasa ingenua entregaría 500,000 tickets mientras que la curva da 332,665. Un pool que cotizara la línea recta lo drenaría la primera ballena que hiciera la aritmética. La curva no está siendo tacaña. Se está negando a venderte el pool entero al precio marginal.

### El único lugar donde esto entra en panic: `u64` por `u64`

Mira el numerador: `amount_in_with_fee * reserve_out`. Los dos derivan de valores `u64`. Multiplica dos números cerca del tope de `u64` y el producto necesita hasta 128 bits. Hazlo en `u64` y el programa entra en panic en un canje grande contra un pool profundo, que es exactamente el canje que menos quieres que falle. El arreglo no es ponerle tope a tus reservas. Es promover a `u128`, multiplicar ahí, dividir de vuelta hacia abajo, y hacer cast del resultado a `u64` solo una vez que sabes que entra (y siempre entra, porque la salida nunca puede exceder `reserve_out`, que ya es un `u64`).

La función de cotización, recorrida línea por línea. Esta es la interfaz de artefacto que va a llamar un lab posterior, así que la firma está congelada: `swap_out(reserve_in, reserve_out, amount_in) -> u64`.

```rust
/// Quote a constant-product swap output with a 0.3% fee.
///
/// Uses a u128 intermediate so the product of two u64 reserves cannot
/// overflow. Integer division truncates, and truncation favors the pool
/// (the trader is never rounded up). Returns 0 for a zero input or an
/// empty reserve, so the caller can treat 0 as "no trade".
pub fn swap_out(reserve_in: u64, reserve_out: u64, amount_in: u64) -> u64 {
    // Nothing to trade, or a side of the pool is empty: no quote.
    if amount_in == 0 || reserve_in == 0 || reserve_out == 0 {
        return 0;
    }

    // 0.3% fee: only 997 of every 1000 input units reach the curve.
    // amount_in <= u64::MAX, so * 997 stays well inside u128.
    let amount_in_with_fee = (amount_in as u128) * 997;

    // The one multiply that can exceed u64. Guard it; on overflow, no quote.
    let numerator = match amount_in_with_fee.checked_mul(reserve_out as u128) {
        Some(n) => n,
        None => return 0,
    };

    // reserve_in > 0 here, so the denominator is never zero.
    let denominator = (reserve_in as u128) * 1000 + amount_in_with_fee;

    // Output is bounded by reserve_out (a u64), so this cast never truncates.
    (numerator / denominator) as u64
}
```

![Las cinco líneas estructurales de swap_out, cada una emparejada con la falla específica que previene: cotización de pool vacío, comisión antes de la curva, overflow de u128 en la multiplicación, división por cero, y truncamiento seguro a favor del pool.](assets/v03-annotated-code.png)

Fíjate en la dirección del redondeo. La división entera tira el resto, así que el trader siempre se lleva el redondeo hacia abajo, nunca hacia arriba. Eso es deliberado. El redondeo tiene que favorecer al pool en cada canje, porque un swap corre millones de veces y una fracción redondeada para el lado equivocado, repetida, es una fuga lenta. Que el pool se quede con el polvo es correcto. Que se lo quede el trader es un bug que encontrarías meses después como un faltante que no puedes explicar.

Acá es donde la economía se gana una frase, y solo una, porque esta es una lección de Anchor y no de mercados. La razón por la que un pool puede cotizarse a sí mismo sin oráculo y sin operador es que la curva convierte la liquidez en una función de precio: la profundidad se vuelve estabilidad, y cada canje le paga al pool por el privilegio de moverlo. Eso es una pieza de diseño de mecanismos genuinamente elegante, y es la razón por la que el mismo invariante de dos líneas aparece debajo de Uniswap, debajo de una bonding curve de pump.fun, y debajo del juguete que estás construyendo ahora mismo. No lo estás inventando. Lo estás cableando dentro de Anchor.

### Adónde se fueron los 129 tickets

Te prometí que ibas a saber adónde se fue cada uno de esos 129 tickets, así que acá está la contabilidad completa de ese canje de 10,000 tokens contra el pool balanceado de 1,000,000 / 1,000,000. Se parte limpiamente en dos piezas, y ninguna de las dos es una fuga.

Corre la curva sin ninguna comisión y sacas 9,900 tickets, no los 10,000 ingenuos. Esos 100 que faltan son impacto de precio. En el instante en que tus 10,000 tokens aterrizan en la reserva el pool ya no está balanceado uno a uno, así que la curva vuelve a ponerles precio a los tickets que estás comprando mientras los compras. Moviste el mercado y pagaste por moverlo. Nadie se embolsó esos 100 tickets; la cotización lineal ingenua nunca hizo más que fingir que estaban sobre la mesa.

Ahora vuelve a poner la comisión de 0.3% y la salida cae de 9,900 a 9,871. Esos últimos 29 tickets son la comisión, y se quedan atrás como reserva extra. Cien al impacto de precio, veintinueve a la comisión, ciento veintinueve en total. Los dos son la curva haciendo precisamente aquello para lo que fue diseñada, y los dos son números que un trader querría tener enfrente antes de firmar, que es toda la razón por la que existe `min_out`.

Esos mismos 29 tickets de comisión son los que levantan el invariante. Antes del canje, `k = 1,000,000 * 1,000,000 = 1,000,000,000,000`. Después, `reserve_in = 1,010,000` y `reserve_out = 990,129`, así que `k = 1,000,030,290,000`, un pelo por encima de donde empezó. `k` nunca cae. La comisión es lo que lo empuja hacia arriba, y la prueba de invariante de la barrera afirma exactamente eso: `k_after >= k_before`, nunca igualdad.

![Una cascada que baja desde la cotización ingenua de 10,000 tickets, 100 tickets por impacto de precio y 29 por la comisión, y aterriza en el fill real de 9,871 tickets.](assets/v04-chart.png)

### Mover los tokens: dos transferencias, dos firmantes

La cotización es una función pura. No toca ninguna cuenta. El canje de verdad son dos transferencias SPL en direcciones opuestas, y la parte interesante es quién firma cada una.

La transferencia de entrada es fácil: el trader está gastando sus propios tokens, así que firma el trader. Los tokens de arcade se mueven de `trader_arcade` a `reserve_arcade`.

La transferencia de salida es el patrón que hace de esto una lección de Anchor. Los tickets viven en `reserve_ticket`, y la autoridad de esa cuenta es el PDA del pool, una dirección sin clave privada. Nadie puede firmar por ella salvo el programa que es dueño de sus seeds. Así que firma el programa, usando `with_signer` y los seeds del pool, exactamente como tu quarter-vault firmó sus propios withdrawals en la lección pasada. Los tickets se mueven de `reserve_ticket` a `trader_ticket` bajo la firma del pool.

El handler corre una secuencia fija, y el orden no es cosmético: las lecturas tienen que pasar antes de que existan los handles, y la guarda va delante de las dos transferencias. Equivócate en lo primero y el compilador te frena (una lectura después de un handle). Equivócate en lo segundo y — sé preciso acá — la atomicidad igual salva al trader: un `require!` que falla después de las transferencias las revierte las dos, así que un fill malo nunca llega a liquidarse de verdad. Lo que te cuesta una guarda tardía es otra cosa. Gastas dos CPIs completas para enterarte de lo que una sola comparación podía haber dicho de entrada, y escribes un handler donde la única protección del trader se lee como una ocurrencia tardía sobre la que un revisor tiene que razonar hacia atrás. Las guardas van antes del dinero por costo y por legibilidad, no porque el runtime fuera a dejar en pie un fill revertido.

![El handler lee primero las dos reservas, cotiza la salida, revierte si queda por debajo de min_out, y después corre la CPI de pull firmada por el trader y la CPI de push firmada por el PDA del pool, en ese orden fijo.](assets/v05-flowchart.png)

Acá está el handler de swap completo. La dirección de token-in está trabajada; la dirección de token-out es el relleno y se muestra acá para que veas el espejo, pero en el lab la vas a tipear tú mismo contra un stub.

```rust
use anchor_lang::prelude::*;
use anchor_spl::{
    // `token` rides along as a MODULE, not for a type: the `token::mint = ...`
    // constraints below expand to code that names the module by path, so it has
    // to be in scope or the derive fails to resolve — m05-l1's import rule.
    token,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

// Placeholder. Keep the id `anchor new token-ticket-swap` generated for you.
declare_id!("<your generated program id>");

pub const POOL_SEED: &[u8] = b"pool";

#[program]
pub mod token_ticket_swap {
    use super::*;

    pub fn swap_arcade_for_tickets(
        ctx: &mut Context<SwapArcadeForTickets>,
        amount_in: u64,
        min_out: u64,
    ) -> Result<()> {
        // 1. Read the reserves BEFORE opening any CPI handle. In V2 you cannot
        //    hold a typed reference to a token account while a CpiHandle from it
        //    is live, so the read happens here, up front, once.
        let reserve_in = ctx.accounts.reserve_arcade.amount();
        let reserve_out = ctx.accounts.reserve_ticket.amount();

        // 2. Quote the output from the pre-trade reserves.
        let out = swap_out(reserve_in, reserve_out, amount_in);
        require!(out > 0, SwapError::ZeroOutput);

        // 3. Slippage guard (this is the SOLO piece in the lab).
        require!(out >= min_out, SwapError::SlippageExceeded);

        // 4. Pull the arcade tokens IN. The trader signs for their own account.
        let pull = TransferChecked {
            from: ctx.accounts.trader_arcade.cpi_handle_mut(),
            mint: ctx.accounts.mint_arcade.cpi_handle(),
            to: ctx.accounts.reserve_arcade.cpi_handle_mut(),
            authority: ctx.accounts.trader.cpi_handle(),
        };
        token_interface::transfer_checked(
            CpiContext::new(ctx.accounts.token_program.address(), pull),
            amount_in,
            ctx.accounts.mint_arcade.decimals(),
        )?;

        // 5. Push the tickets OUT. The pool PDA signs with its own seeds.
        //    Copy the bump into an owned local FIRST, exactly as the vault did:
        //    `signer_seeds` borrows this array, so it has to outlive the CPI
        //    below. That is a lifetime requirement, not a borrow conflict --
        //    `pool` goes into `push` as a shared `cpi_handle()`, which leaves
        //    reads of `pool` legal. The exclusion applies to the accounts handed
        //    over with `cpi_handle_mut()`.
        let bump = [ctx.accounts.pool.bump];
        let signer_seeds: &[&[&[u8]]] = &[&[POOL_SEED, &bump]];
        let push = TransferChecked {
            from: ctx.accounts.reserve_ticket.cpi_handle_mut(),
            mint: ctx.accounts.mint_ticket.cpi_handle(),
            to: ctx.accounts.trader_ticket.cpi_handle_mut(),
            authority: ctx.accounts.pool.cpi_handle(),
        };
        token_interface::transfer_checked(
            CpiContext::new(ctx.accounts.token_program.address(), push)
                .with_signer(signer_seeds),
            out,
            ctx.accounts.mint_ticket.decimals(),
        )?;

        Ok(())
    }
}

pub fn swap_out(reserve_in: u64, reserve_out: u64, amount_in: u64) -> u64 {
    if amount_in == 0 || reserve_in == 0 || reserve_out == 0 {
        return 0;
    }
    let amount_in_with_fee = (amount_in as u128) * 997;
    let numerator = match amount_in_with_fee.checked_mul(reserve_out as u128) {
        Some(n) => n,
        None => return 0,
    };
    let denominator = (reserve_in as u128) * 1000 + amount_in_with_fee;
    (numerator / denominator) as u64
}

#[derive(Accounts)]
pub struct SwapArcadeForTickets {
    pub trader: Signer,

    #[account(seeds = [POOL_SEED], bump = pool.bump)]
    pub pool: Account<Pool>,

    // Pinned to what the pool recorded at init. Without these two lines the mint
    // fields on Pool would be decoration: a caller could hand you any mint whose
    // reserves happen to be pool-owned, and the token:: constraints below would
    // happily agree with them. `address = parent.field` is the has_one replacement,
    // and this is the second place in two lessons it does real work.
    #[account(address = pool.arcade_mint @ SwapError::WrongMint)]
    pub mint_arcade: InterfaceAccount<Mint>,
    #[account(address = pool.ticket_mint @ SwapError::WrongMint)]
    pub mint_ticket: InterfaceAccount<Mint>,

    #[account(mut, token::mint = mint_arcade, token::authority = pool)]
    pub reserve_arcade: InterfaceAccount<TokenAccount>,
    #[account(mut, token::mint = mint_ticket, token::authority = pool)]
    pub reserve_ticket: InterfaceAccount<TokenAccount>,

    #[account(mut, token::mint = mint_arcade, token::authority = trader)]
    pub trader_arcade: InterfaceAccount<TokenAccount>,
    #[account(mut, token::mint = mint_ticket, token::authority = trader)]
    pub trader_ticket: InterfaceAccount<TokenAccount>,

    pub token_program: Interface<'static, TokenInterface>,
}

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub arcade_mint: Address,  // 32
    pub ticket_mint: Address,  // 32
    pub bump: u8,              //  1
    pub _pad: [u8; 7],         //  7  explicit Pod padding (65 -> 72)
}

#[error_code]
pub enum SwapError {
    #[msg("Swap output is zero")]
    ZeroOutput,
    #[msg("Slippage tolerance exceeded: output below min_out")]
    SlippageExceeded,
    #[msg("Mint does not match the one this pool was initialized with")]
    WrongMint,
}
```

Una elección de constraint merece una nota antes de los detalles de V2. Las cuatro cuentas de token llevan la familia `token::` de la lección pasada, no `associated_token::`. La razón es la misma que puso `token::` en el destinatario del escrow: un constraint de ATA *deriva* una dirección a partir de un dueño y un mint, y las reservas no son ATAs. El pool sostiene dos cuentas de token que creó y se asignó a sí mismo, en direcciones que nadie deriva, así que no hay contra qué derivar. `token::mint` y `token::authority` restringen qué sostiene la cuenta y quién la controla, que es exactamente la afirmación que necesitas acá.

Unas cuantas cosas de ese código son detalles de Anchor V2 en los que vale la pena detenerse, porque son nuevos desde la línea 0.x que quizá escribiste antes, y este es un curso de framework.

La forma de la CPI cambió. `CpiContext::new` ahora toma el programa como un `&Address`, que sacas de `ctx.accounts.token_program.address()`. En la línea 0.x pasabas un `AccountInfo` (un `.to_account_info()`); en V2 eso es un error de compilación, `expected Address, found AccountInfo`. Las cuentas que pasas al struct `TransferChecked` tampoco son `AccountInfo`s. Son valores `CpiHandle`, producidos por `cpi_handle()` para una cuenta de solo lectura y `cpi_handle_mut()` para una mutable. El handle es el ticket de la cuenta hacia la CPI, y lleva un borrow de Rust sobre el wrapper tipado del que salió.

Ese borrow es el punto de la próxima sección, y es la trampa que antes mordía a todo el mundo.

![Lado a lado: en la línea 0.x una cuenta deserializada quedaba obsoleta a menos que llamaras a reload, mientras que en V2 el handle de CPI verificado por el borrow checker convierte esa lectura obsoleta en un error de compilación.](assets/v06-comparison.png)

### Por qué V2 no te deja leer un balance a mitad de una CPI

En la línea 0.x, este era el bug clásico. Hacías una CPI que movía tokens, después leías `token_account.amount` y actuabas sobre eso, olvidándote de que Anchor deserializó esa cuenta *una sola vez*, arriba de todo de la instrucción. La CPI cambió el balance on-chain, pero tu copia en memoria todavía tenía el número viejo. Tenías que llamar a `.reload()` para refrescarlo, y si te olvidabas, tomabas una decisión sobre datos obsoletos. Era silencioso, era fácil, y se entregaba.

V2 mata la clase. Un `CpiHandle` sostiene un borrow de Rust sobre el wrapper tipado de la cuenta. Mientras ese handle está vivo, el borrow checker no te deja tocar la cuenta tipada, así que `reserve_ticket.amount()` durante un handle en vuelo desde `reserve_ticket` no es una sorpresa de runtime. No compila. El arreglo no es hacer `.reload()`. El arreglo es estructural: lee cada reserva que necesites *antes* de abrir un handle, que es exactamente por qué el paso 1 del handler lee los dos balances arriba de todo, antes de que exista ningún `cpi_handle_mut()`. No hay ninguna lectura a mitad de CPI que puedas equivocar, porque el lenguaje quitó la capacidad de escribir una.

Así que no recurras a `.reload()` acá por memoria muscular de 0.x. Si te descubres queriéndolo, estructuraste las lecturas en el orden equivocado. Muévelas más arriba.

### La trampa de duplicate-mutable en la que no vas a caer

Un default más de V2 que vale la pena nombrar, porque un swap es exactamente la forma que lo dispara. V2 rechaza una instrucción que recibe la misma cuenta mutable en dos campos. Pasa la misma cuenta de token como origen `mut` y como destino `mut` y la validación falla antes de que corra tu handler. Esa verificación existe para atrapar bugs de aliasing donde por accidente lees y escribes la misma cuenta a través de dos nombres y la corrompes.

Un swap tiene dos reservas, y la tentación, si estás pensando en el pool como una sola cosa, es rutear las dos direcciones a través de una cuenta. Haz eso y V2 te frena. La reacción equivocada es callar la verificación con `unsafe(dup)`, la salida de emergencia para cuentas genuinamente duplicadas. La reacción correcta es notar que un pool de producto constante tiene dos reservas por definición, así que cada lado es su propia cuenta de token distinta. `reserve_arcade` y `reserve_ticket` son cuentas distintas que sostienen mints distintos. El default de duplicate-mutable se satisface gratis, y nunca tocas `unsafe(dup)`. Recurrir a él acá no sería un opt-out. Sería tapar un diseño donde colapsaste dos reservas en una, que es un bug que la verificación acaba de atrapar por ti.

![El arreglo equivocado colapsa las dos reservas en una sola cuenta callada con unsafe(dup); el diseño correcto mantiene dos cuentas de reserva distintas, así no existe ninguna duplicación ni ningún opt-out.](assets/v07-comparison.png)

El equipo de Anchor no agregó estos defaults por puntos de estilo. Cuando hicieron benchmark de V2 contra Quasar y Pinocchio antes de la conferencia Accelerate a principios de mayo de 2026 (el encuadre está ahí mismo en el issue #4355, donde el esfuerzo entero de V2 se justificó como existencial), los programas que registraron las mayores reducciones de cómputo fueron los programas de la familia AMM, el benchmark `prop-amm` específicamente, con una reducción máxima reportada de 50.4x. Por eso un swap es la pieza de exhibición: el patrón que estás construyendo es el que V2 afinó para hacer barato. No voy a imprimir un número de cómputo para este programa exacto, porque esas cifras se movieron a medida que el proyecto las afinaba y lo honesto es que midas el tuyo, pero la dirección era el pitch entero.

![Una línea de tiempo desde la justificación por benchmark del issue #4355 hasta las corridas previas a Accelerate de principios de mayo de 2026, donde el programa prop-amm registró la mayor reducción reportada.](assets/v08-timeline.png)

### La guarda de slippage, y por qué es todo el punto

Hay una brecha entre el momento en que un trader lee una cotización y el momento en que su transacción aterriza. En esa brecha, otros canjes pueden pegarle al pool y mover las reservas. Al trader al que se le cotizaron 9,871 tickets hace un momento, sobre un pool de 1,000,000 / 1,000,000, le puede tocar aterrizar en un pool que una venta grande ya movió, y sacar 9,400. Si tu programa llena cualquier canje que la curva produzca, el trader se come esa diferencia, y un actor hostil puede *fabricar* esa diferencia ordenando un canje delante del suyo y otro detrás. Eso es un sandwich, y la única defensa que tiene contra eso un swap así de simple es `min_out`.

`min_out` es el trader diciendo "revierte a menos que consiga al menos esto". El programa computa `out` a partir de las reservas vivas, lo compara con `min_out`, y revierte si queda corto. Esa es la línea `require!(out >= min_out, SwapError::SlippageExceeded)`, y su forma es el mismo release condicional que escribiste dentro del prize-escrow en R3: computa un valor, compáralo con una cota provista por el llamador, revierte si la cota no se cumple. Ya escribiste este flujo de control antes. Por eso es tu solo.

Déjala afuera y todo canje se puede llenar a cualquier precio, que es como decir que todo canje es sandwicheable. Es una línea, y es la diferencia entre un swap que un jugador puede usar y un swap que un bot farmea.

## Lab: construye R4, el swap de token-a-ticket

R4 es un programa nuevo. Di con claridad qué reusa y qué no, porque la tentación es recurrir al vault: las reservas del pool **no** son instancias de vault de R2. La cuenta de token de un vault es una ATA cuya autoridad es el PDA del vault, y la autoridad de una reserva tiene que ser el PDA del pool, así que un vault no puede ser una reserva sin dejar de ser un vault. R4 no compone sobre nada. Es el primer peldaño desde el módulo 1 que se sostiene solo, y esa es su forma honesta: lo que se traslada es el patrón, el `transfer_checked` firmado por PDA que aprendiste en el vault, no el artefacto. El vault y el escrow siguen haciendo su trabajo en otra parte del salón, y el capstone del módulo 9 es donde los cuatro peldaños por fin se encuentran.

El repliegue de la ayuda es explícito: el paso 1 es una especificación que implementas, el paso 2 lo tipeas desde la fórmula, los pasos 3 y 4 están trabajados, la segunda CPI del paso 5 es un completion que tipeas contra un stub, y los pasos 6 y 7 son solo.

Primero, el toolchain, una línea:

```bash
anchor --version       # confirm you are on the V2 line (2.0.0-rc.1 as of 2026-08-22), not 1.1.2
```

Si reporta 1.x, vuelve a fijar con el bloque de instalación de m01-l2 — `--tag v2.0.0-rc.1`, `--locked`, el canal de git, ya que `avm` todavía no puede traer la RC. No verifiques lecciones de V2 sobre la línea 1.1.2; las APIs de CPI y de cuentas difieren y tu código no va a compilar contra la vieja.

1. **Levanta el estado del pool (especificación, sin código dado).** Escribe una instrucción `init_pool`. Crea la cuenta `Pool` en `seeds = [POOL_SEED]` con un `bump` pelado, guarda las dos direcciones de mint y `ctx.bumps.pool`, y hace `init` de dos cuentas de token cuya `token::authority` es el PDA del pool, una por mint, pagadas por el llamador. Ya escribiste cada una de esas líneas antes: `init` más `seeds` más `bump` es el módulo 3, guardar el bump canónico es el módulo 3, y crear una cuenta de token propiedad del programa es el `Initialize` de la lección pasada con otra autoridad. Checkpoint: `anchor build` compila. R4 no compone sobre nada, así que todavía no hay ninguna suite heredada que correr ni nada contra qué afirmar hasta que el paso 7 escriba una — momento en el que lo primero que esa prueba demuestra es exactamente esto: la cuenta del pool existe y las dos cuentas de token de reserva reportan el PDA del pool como su autoridad. Anota el paso ahora para que el paso 7 tenga una afirmación esperándolo.

2. **Agrega la función de cotización.** Tipea `swap_out` tú mismo a partir de las dos líneas de fórmula de arriba, con la firma congelada; vuelve a la versión trabajada solo después de que compile la tuya. Escribe una prueba unitaria que la llame con `reserve_in = 1_000_000`, `reserve_out = 1_000_000`, `amount_in = 10_000` y afirme que devuelve `9_871`. Si te sale `10_000`, te olvidaste de la comisión. Si entra en panic en un caso de reserva grande, multiplicaste en `u64`. Checkpoint: la prueba unitaria está verde y el 9,871 computado a mano coincide.

3. **Lee las reservas por adelantado.** En el handler del swap, lee `reserve_arcade.amount()` y `reserve_ticket.amount()` hacia locales antes de construir ninguna cuenta de CPI. Esto no es estilo opcional. Es el único lugar donde el compilador te va a dejar leerlas, porque una vez que existe un `cpi_handle_mut()` desde una reserva, el acceso tipado `.amount()` sobre esa reserva no compila. Checkpoint: demuestra esa afirmación en vez de confiar en ella. Mueve las dos lecturas para que queden *entre* el binding `let pull = TransferChecked { .. };` y la llamada a `transfer_checked` que lo consume, que es la única ventana donde el handle está genuinamente vivo, y corre `anchor build`. Deberías sacar un error de borrow que nombra `reserve_arcade`, no una sorpresa de runtime. Ponlas en cambio en cualquier lado después de la llamada a `transfer_checked` y el build se pone verde, porque el handle ya se soltó, lo que también vale la pena ver: la regla es sobre el lifetime del handle, no sobre el número de línea. Muévelas de vuelta arriba y sigue.

4. **Cablea la CPI de token-in (trabajada).** Construye el `TransferChecked` para `trader_arcade -> reserve_arcade` con el trader como autoridad, e invócala con `token_interface::transfer_checked` sobre `CpiContext::new(token_program.address(), pull)`. Acá no va `with_signer`: el trader es un firmante real en la transacción. Checkpoint: después de esta CPI la reserva de arcade del pool creció en `amount_in`.

5. **Completa la CPI de token-out (relleno).** Se te da el stub:

```rust
// TODO(you): move `out` tickets from reserve_ticket to trader_ticket,
// signed by the pool PDA. Mirror the token-in CPI, but:
//   - from/to are the ticket accounts, not the arcade accounts
//   - the mint is mint_ticket
//   - the authority is the pool PDA, so you must attach with_signer
let bump = [ctx.accounts.pool.bump];             // read out first, same as the vault
let signer_seeds: &[&[&[u8]]] = &[&[POOL_SEED, &bump]];
let push = TransferChecked {
    // fill in the four accounts using cpi_handle_mut() / cpi_handle()
};
// invoke transfer_checked over CpiContext::new(...).with_signer(signer_seeds)
// with `out` and mint_ticket.decimals()
```

Rellénalo contra la versión trabajada de arriba. Lo único que no te puedes saltar es `.with_signer(signer_seeds)`. Sin eso el runtime no tiene firma para el PDA del pool y la transferencia falla con un error de firmante faltante. Checkpoint: un canje mueve balances de ticket reales hacia `trader_ticket`.

6. **Agrega la guarda de slippage (solo).** Antes de cualquiera de las dos CPIs, después de computar `out`, revierte cuando `out < min_out`. Tienes la variante de error (`SwapError::SlippageExceeded`) y ya escribiste esta forma exacta de release condicional en el escrow. Checkpoint: un canje con `min_out` puesto uno por encima de la salida cotizada revierte con el error de slippage, y un canje con un `min_out` razonable se llena.

7. **Escribe la barrera (solo).** Dos pruebas de LiteSVM en `programs/token-ticket-swap/tests/swap_invariant.rs`, construidas sobre la forma `spl_setup` de la lección pasada: crea dos mints, inicializa el pool, fondea las dos reservas, fondea un trader. Después:

   - `invariant_holds`: lee `k_before = reserve_in * reserve_out` como `u128`, manda un swap con un `min_out` permisivo, vuelve a leer las dos reservas, y afirma `k_after >= k_before`. Mayor-o-igual, no igual: la división entera redondea hacia abajo la salida del trader, así que el pool se queda con el resto y `k` solo crece.
   - `slippage_reverts`: cotiza el canje con `swap_out` en la prueba, mándalo con `min_out = quote + 1`, y afirma que la transacción da error.

Cuando los siete estén hechos, corre la barrera:

```bash
anchor build && cargo test --test swap_invariant
```

Las dos en verde. Si `invariant_holds` falla con `k_after < k_before`, tu redondeo está favoreciendo al trader en algún lado. Si `slippage_reverts` se llena en cambio, tu guarda está comparando en la dirección equivocada o falta.

## Challenge: cotiza un swap de producto constante

El lab cableó el swap dentro de un programa, donde una cotización equivocada sale a la superficie como una afirmación de balance que falla tres capas más allá. El challenge saca `swap_out` del framework por completo para que la aritmética sea lo único que pueda estar mal. El starter, en `lessons/m05-l2/cp-swap-out/`, es la cotización lineal ingenua: ignora tanto la comisión como el corrimiento de la reserva, sobrecotiza, y dejaría que un trader drene el pool.

Implementa `swap_out(reserve_in, reserve_out, amount_in) -> u64` de modo que:

- aplique la comisión de 0.3% (`997/1000`) bajo el invariante de producto constante, no la razón de precio ingenua;
- use un intermedio `u128` para que dos reservas `u64` no puedan hacer overflow en la multiplicación;
- devuelva `0` para una entrada en cero o una reserva vacía.

Los ocho casos que afirma el banco de pruebas:

| reserve_in | reserve_out | amount_in | esperado |
|---|---|---|---|
| 1_000 | 1_000 | 100 | 90 |
| 1_000_000 | 1_000_000 | 1_000 | 996 |
| 5_000 | 10_000 | 500 | 906 |
| 1_000 | 1_000 | 0 | 0 |
| 1_000_000 | 1_000_000 | 10_000 | 9_871 |
| 0 | 1_000_000 | 10_000 | 0 |
| u64::MAX | u64::MAX | u64::MAX | 0 — sin panic, sin overflow |
| 1e18 | 1e18 | 1e12 | 996_999_005_991 |

Tres empujoncitos si te trabas:

- `amount_in_with_fee = amount_in * 997`
- `out = (amount_in_with_fee * reserve_out) / (reserve_in * 1000 + amount_in_with_fee)`
- promueve a `u128` antes de multiplicar para que las reservas `u64` no puedan hacer overflow

La función es un `const fn` con afirmaciones en tiempo de compilación debajo (el recurso de m03-l3, y lo que la calificación solo-de-compilación de verdad exige), así que el starter anuncia sus propios bugs en tiempo de build: las filas de cotización equivocada fallan afirmaciones cuyos mensajes nombran el caso, y las filas grandes fallan más fuerte — la evaluación const rechaza de plano la multiplicación `u64` ingenua con `attempt to multiply with overflow`, que es rustc haciendo el argumento de la lección por ti. Las últimas dos filas son las que separan una cotización que funciona de una correcta. La fila de 1e18 hace overflow de plano en una multiplicación `u64`: en runtime eso entra en panic en debug y hace wrap en release, y una cotización que hizo wrap es un withdrawal gratis. La fila de `u64::MAX` después hace overflow de `u128` también, y vale la pena ser exacto sobre por qué, porque la razón *no* es que dos factores `u64` dejen de entrar. Siempre entran: `(u64::MAX)²` es `2¹²⁸ − 2⁶⁵ + 1`, que aterriza justo debajo del techo de `u128` con menos de un solo bit de sobra. El numerador acá es `amount_in * 997 * reserve_out` — esos dos factores del tamaño de `u64` *más* el multiplicador de la comisión — y el `× 997` es lo que gasta la astilla que quedaba. Así que promover es necesario y aun así no suficiente: la multiplicación tiene que quedarse `checked` y degradar a `0`. Un `u128 *` pelado pasa todas las demás filas y muere en esa. Computa la primera fila a mano antes de programarla. Cuando tu aritmética y el programa coinciden, entiendes la curva, no solo el código.

## Dónde te deja esto

Tienes R4: un pool de dos lados que se cotiza a sí mismo desde sus reservas, mueve tokens en las dos direcciones bajo la firma de un programa, y rechaza un canje que se llenaría peor de lo que el trader acepta. Derivaste la curva, viste por qué el intermedio `u128` no es opcional, y miraste cómo la vieja trampa de `.reload()` se convierte en algo que el compilador simplemente se niega a dejarte escribir. Esa última parte es el tema de este curso entero: V2 mueve clases de bugs de runtime a tiempo de compilación, y el swap es donde sentiste tres de ellas a la vez.

Construiste todo eso contra un mint SPL simple. Acá está la pregunta que abre la próxima lección. ¿Qué pasa en el momento en que un jugador trae un mint Token-2022 que cobra una tarifa de transferencia y corre un transfer hook? Tu `transfer_checked` igual llama, pero ¿el número que cotizaste sigue coincidiendo con el número que llega, y las cuentas extra del hook siquiera entran por la CPI que acabas de escribir? La próxima lección aprendes a responder eso desde el mint mismo, leyendo uno vivo antes de confiar en él. Arreglar un swap para un mint con hook es trabajo de estándares que le corresponde al curso de Digital Assets; saber que tendrías que hacerlo es la parte que es tuya, y está a una lectura de distancia.
