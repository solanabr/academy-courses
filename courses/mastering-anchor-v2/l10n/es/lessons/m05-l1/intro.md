# token_interface y transfer_checked: mueve el vault y el escrow a tokens de verdad

La lección pasada construiste el prize-escrow, R3. Nunca tiene el premio en sí. Lo estaciona en una instancia de quarter-vault (R2) por medio de una llamada entre programas, registra quién puede reclamarlo y el puntaje que tiene que superar, y solo lo suelta cuando la condición se dispara y el jugador correcto lo pide. Los dos programas funcionan de punta a punta. Y los dos custodian lamports nativos.

Que es el problema, porque nadie en un arcade paga en SOL crudo. Los jugadores tienen tokens de arcade, mints SPL de verdad, y toda la economía que estás cableando corre sobre eso. Así que acá está la pregunta que de verdad se siente, la que decide si las dos lecciones pasadas fueron un calentamiento o lo de verdad: ¿pasar de lamports a tokens de verdad quiere decir una reescritura, o cambia un conjunto chico y contable de líneas mientras el esqueleto de PDA y CPI se queda exactamente como lo construiste?

Antes de leer mi respuesta, ve a buscar la tuya. Abre el vault de R2 y cuenta las líneas que de verdad pertenecen a la custodia:

```bash
# In your R2 quarter-vault program, list every line that truly touches custody:
grep -n 'b"sol"\|sol_vault\|sol_bump\|lamports\|system_program' programs/quarter-vault/src/lib.rs
```

Todo lo que vuelve pertenece a una sola capa: el PDA de custodia `[b"sol", owner]`, el `sol_bump` guardado que firma por él, y la transferencia del System Program que mueve los lamports. Lo que el grep *no* imprime es la mitad más interesante: los seeds `[b"vault", authority]` propios del PDA de estado, su `bump` guardado, el pin de autoridad `address =`, el `checked_sub` sobre los libros. Esos son agnósticos a la custodia. Las líneas que sí volvieron son toda la lección, y las vamos a cambiar.

## Resumen

Vas a actualizar dos programas de lamports que funcionan, llevándolos a custodia de tokens SPL: el quarter-vault (R2) y el prize-escrow (R3). Las formas de las instrucciones se quedan igual. `deposit`, `withdraw`, `release` en el vault; `reserve` y `redeem` en el escrow. Lo que cambia es la capa de custodia: el PDA aparte que sostenía los lamports se retira, la transferencia del System Program se vuelve un `transfer_checked` firmado por el PDA, y las cuentas ganan un mint más constraints de cuenta de token. El PDA de estado y su bump guardado, la verificación `address = vault.owner`, el `checked_sub` sobre los libros y la barrera de condición del escrow no se mueven.

La ayuda se repliega como lo hace en toda lección de build. Recorro el upgrade del vault totalmente resuelto, `withdraw` incluido, porque esa forma es la que quiero en tus dedos. El release de `redeem` del escrow lo terminas tú: el cableado del mint y los decimals hacia la CPI del vault es el único hueco que dejo. Después vuelves a correr las dos verificaciones provisionales desde cero en SPL, en Solo. Terminaste cuando un `transfer_checked` firmado por el PDA mueve un balance de tokens bajo la firma del vault y el escrow libera solo con la condición correcta, las dos en verde.

Un límite por delante, porque es fácil cruzarlo sin darse cuenta. Esta lección es tokens *desde el asiento del framework*: cómo los mueve tu programa. Lo que Token-2022 de verdad cambia sobre un mint, las comisiones y los hooks y los balances confidenciales, es material del curso de Digital Assets, y este curso solo vuelve sobre el borde de eso que se ve desde el asiento del programa, en m05-l3. Acá nos importa exactamente una cosa: el mismo camino de código sirviendo al SPL Token clásico y a Token-2022 sin que tú le dejes hardcoded ninguno de los dos.

## Lo que de verdad cuesta la custodia cuando se va a SPL

Primero la forma de la respuesta; los artefactos vienen después.

La custodia es una capa, no el programa. La identidad de tu vault, el PDA derivado de `[b"vault", authority]` con su bump guardado, es quién *es* el vault. La custodia es lo que el vault *tiene*. En la versión de lamports, tener quería decir que los lamports se sentaban en un segundo PDA, propiedad del System, y se movían por una transferencia del System Program firmada por los seeds de ese PDA. En la versión SPL, tener quiere decir una cuenta de token cuya autoridad es el PDA de estado mismo, y mover quiere decir una CPI de `transfer_checked` al token program, firmada por los seeds del PDA de estado. Identidad sin tocar. La cuenta de custodia y el verbo son lo que cambia.

Esa es la espina dorsal de todo el upgrade, y vale la pena verla como un diff antes de que toquemos una sola línea.

![Dos columnas: los seeds del PDA, el bump guardado, la verificación de autoridad, el débito y las formas de las instrucciones quedan idénticos, mientras que solo la capa de custodia cambia a una cuenta de token y transfer_checked.](assets/v01-comparison.png)

Lee esa columna derecha otra vez. Cuatro ítems. Eso es lo que cuesta una migración de custodia. Todo lo de la izquierda es el trabajo que ya hiciste en R2 y R3, y sobrevive al traslado intacto. Por esto exactamente el camino del upgrade enseña tokens mejor de lo que lo haría un build recién hecho. Un build recién hecho esconde la costura, porque todo es nuevo de una vez. El upgrade aísla la costura, y la costura es la lección.

### El asiento de tokens: Interface, InterfaceAccount y un solo camino de código

Anchor V2 le da a tu programa un asiento específico para tokens, y vive en `token_interface`. Dos tipos lo cargan.

La cuenta de programa es `Interface<'static, TokenInterface>`. Compárala con `Program<Token>`, a la que irías por reflejo. `Program<Token>` amarra tu instrucción a exactamente un programa, el SPL Token program clásico en su única dirección. `Interface<'static, TokenInterface>` acepta o el Token program clásico o el Token-2022 program. El mismo espacio, dos inquilinos válidos.

Los mints y las cuentas de token son `InterfaceAccount<Mint>` e `InterfaceAccount<TokenAccount>`. Y acá está la cosa que hace tropezar a todo el que aprendió Anchor hace un año: `InterfaceAccount<T>` en V2 es literalmente un alias de `Account<T>`. No un primo, no un wrapper. El mismo tipo.

Lo que quiere decir que el wrapper no es donde vive el comportamiento, y esta es la oración que hay que retener: la diferencia entre una cuenta que acepta los dos token programs y una que no, es *de qué módulo vino `T`*. `anchor_spl::token::TokenAccount` lleva el id del Token program clásico como su dueño esperado. `anchor_spl::token_interface::TokenAccount` acepta cualquiera. Escribe `Account<TokenAccount>` con un import de `token_interface` y te queda el comportamiento de interface; escribe `InterfaceAccount<TokenAccount>` con un import de `token` y te queda el comportamiento de solo-clásico, con alias o sin alias. La convención de abajo es emparejar `InterfaceAccount` con imports de `token_interface` porque leer el wrapper es más rápido que rastrear un import, pero es una convención, no el mecanismo.

¿Por qué un *alias*, y no el tipo aparte y más pesado que era antes? Este merece un compás, porque la respuesta explica toda la dirección de V2.

![Una línea de tiempo de tres paradas: a Account<T> se lo llama el camino lento de Anchor, la issue #4390 argumenta zero-copy por defecto, y V2 hace de InterfaceAccount<T> un alias del ahora rápido Account<T>.](assets/v02-timeline.png)

Así que cuando escribes `InterfaceAccount<TokenAccount>` en V2, te queda el `Account<TokenAccount>` zero-copy de hoy más la propiedad de que va a validar un mint o una cuenta de token de la que sea dueño *cualquiera* de los dos token programs. No pagas nada extra por la capacidad de interface. Eso es el diseño rindiendo: el camino rápido y el camino compatible ahora son el mismo camino. Es la misma palanca detrás de la mejora promedio de 8.8x en unidades de cómputo que Anchor reporta en su propio banco de pruebas, la cifra que el PR #4914 revisó a la baja desde 9.9x el 2026-08-13. Fechada y atribuida, no medida acá; el módulo 6 es donde mides la tuya.

Lo que levanta la pregunta obvia de quien migra: si `InterfaceAccount<T>` es nada más `Account<T>`, ¿por qué no seguir escribiendo `Account<TokenAccount>`? Porque en el camino reflejo, `Account<TokenAccount>` viene con un `use anchor_spl::token::TokenAccount`, y esa `T` deja hardcoded el Token program clásico como el dueño. En el momento en que aparece un mint Token-2022, esa cuenta falla al cargar. Emparejar el wrapper `InterfaceAccount` con el módulo `token_interface` es lo que se queda agnóstico al dueño en los dos programas. Las combinaciones no son intercambiables, y las que de verdad te vas a encontrar valen la pena verlas lado a lado.

![Dos tipados de cuenta, uno de solo-clásico y uno que acepta los dos token programs, más la elección de cuenta de programa que le corresponde entre Program<Token> e Interface<'static, TokenInterface>.](assets/v03-comparison.png)

Hay una trampa de verdad escondida en esa primera tarjeta, lo bastante sutil para quemarte una tarde. Si tipas una cuenta como `Account<T>` y esperas un error de dueño *propio*, no te va a llegar ninguno. Las verificaciones de dueño y discriminador corren durante la carga de la cuenta, que pasa antes de cualquiera de tus constraint hooks. Así que un dueño que no calza aparece como un `IllegalOwner` genérico, y tu mensaje propio, tan bien redactado, nunca se dispara. Si de verdad necesitas un error de dueño propio, bajas a `UncheckedAccount` y afirmas el dueño tú mismo en el handler. Guárdate esa.

### Los constraints que colocan la cuenta de token del vault

Dos familias de constraint aparecen sobre las cuentas de token en el lab, y vale la pena glosarlas una vez para que se lean como intención y no como conjuro cuando te las encuentres.

La familia `associated_token::` dice "esta cuenta es la Associated Token Account para este mint y este dueño". Una ATA es la única cuenta de token canónica que un dueño dado tiene para un mint dado, en una dirección determinista derivada del dueño, del mint y del token program. Cuando escribes `associated_token::mint = mint`, `associated_token::authority = vault` y `associated_token::token_program = token_program` sobre un campo, Anchor deriva esa dirección canónica y verifica que la cuenta que te entregaron esté ahí. Empareja las tres con `init` y Anchor *crea* la ATA si todavía no existe, pagando el rent desde el `payer`; empareja las tres con `mut` a secas y Anchor nada más valida una ATA que espera que ya esté. Esa es toda la diferencia entre el `initialize` del vault (que hace init de la ATA del vault) y su `deposit` (que valida una que existe).

La familia `mint::`, en cambio, restringe el mint mismo: `mint::decimals`, `mint::authority`, `mint::freeze_authority`. No los vas a necesitar en este upgrade, porque estás custodiando un mint de token de arcade *existente*, no creando uno. Pero son la misma forma, y saber que la familia existe te evita reinventar a mano una verificación de decimals más adelante. La razón por la que la línea `associated_token::token_program = token_program` importa siquiera es la historia del camino de código único que vimos antes: como `token_program` es un `Interface`, la dirección de la ATA derivada se computa contra el token program que de verdad es dueño del mint, así que el mismo constraint resuelve bien para un mint clásico y para un mint Token-2022. Deja hardcoded el programa clásico ahí y derivarías en silencio la dirección equivocada el día en que llegue un mint Token-2022.

![Dos familias de constraint lado a lado: associated_token coloca y deriva una cuenta de token para un mint y un dueño, mientras que mint restringe los decimals y las autoridades de un mint existente.](assets/v04-comparison.png)

### transfer_checked: la primitiva que lleva el mint

La versión de lamports movía valor con una transferencia del System Program. La versión SPL mueve valor con `transfer_checked`, y el nombre está haciendo trabajo de verdad. Un `transfer` pelado toma un monto y le cree. `transfer_checked` toma el monto *más la cuenta del mint más los decimals del mint*, y se niega a correr si los decimals que afirmas no concuerdan con los decimals que están en el mint. Es el token program verificando dos veces que tú y él estén de acuerdo en qué quiere decir una "unidad" antes de mover cualquier cosa.

Esta es la única línea que la mayoría de quienes migran se equivoca primero, porque la memoria muscular echa mano del `transfer` pelado que usaba hace dos años, y en V2 ese `transfer` pelado está deprecado y no va a compilar como lo recuerdan. `transfer_checked` es la primitiva ahora. Dilo una vez, en voz alta: el mint y sus decimals viajan con cada transferencia.

Vale preguntar por qué el token program se molesta, dado que el monto ya es un entero crudo de unidades base y la transferencia movería exactamente esa cantidad de cualquier modo. La respuesta es la clase de bug que cierra la verificación. Un monto de tokens no significa nada sin sus decimals: `1_000_000` es un token entero con seis decimals y una milésima de token con nueve. Un cliente que computa un monto contra los decimals equivocados, o un programa que deja hardcoded un valor de decimals que después se desvía del mint, mueve la cantidad de valor equivocada mientras el entero crudo se ve perfectamente razonable. El `transfer` pelado no puede atrapar eso, porque nunca ve el mint. `transfer_checked` ve los dos, y aborta antes de mover cualquier cosa si los decimals que aseveras no concuerdan con los decimals registrados en la cuenta del mint. Es una verificación de consistencia barata parada exactamente donde antes vivía un error silencioso y caro.

Ahora el trade-off, porque es la parte honesta de este asiento. `InterfaceAccount<T>` te compra un solo camino de código para los dos token programs, y eso es de verdad valioso. Pero lo pagas en dos monedas. Primero, `transfer_checked` mete a la fuerza el mint y los decimals en cada transferencia, así que el mint tiene que estar *presente* en instrucciones que antes no lo necesitaban. Segundo, esa cuenta de mint extra es una cuenta más por instrucción, una cosa más que pasar desde el cliente, una línea más en el struct de cuentas. Ninguno de los dos costos es grande. Los dos son reales. La forma de sentir su tamaño es hacer el upgrade y contar, que es exactamente lo que hace el lab.

## Lab: actualiza el quarter-vault a SPL

Pasos numerados. Los interesantes traen su porqué; los rutinarios corren escuetos. Los checkpoints te dicen cómo se ve el éxito para que nunca adivines si funcionó.

### 1. Fija el toolchain de la RC de V2

Tu máquina viene con anchor-cli en la línea 1.x. Este curso es V2, una superficie de compilador distinta, y la CLI de V2 se consume desde git, no desde `avm`. Esto lo armaste allá en m01-l2, así que es un re-pin, no una instalación desde cero: `avm install` no puede traer el tag de V2 porque no se cortó ningún GitHub Release para él y el binario precompilado que descarga da 404, así que el canal documentado es un build desde el código fuente fijado al tag `v2.0.0-rc.1`.

```bash
# The documented V2 channel. `avm install` 404s on the RC (see m01-l2); build from git.
# macOS, if the build trips on LTO: prefix with CARGO_PROFILE_RELEASE_LTO=off
cargo install --git https://github.com/otter-sec/anchor.git \
  --tag v2.0.0-rc.1 anchor-cli --locked --force

anchor --version   # confirm you are on the V2 line, not your old 1.1.2
```

Nota de frescura (verificada 2026-08-22): V2 se entrega como `2.0.0-rc.1` desde la rama `anchor-next`, y `avm list` no carga nada arriba de `1.1.2`, así que el build desde git es el único canal. Una vez que `anchor --version` imprima la cadena exacta de la RC, fija *esa* cadena en `Anchor.toml` bajo `[toolchain]` y fija el commit en tu Dockerfile de CI, porque las APIs de V2 pueden moverse entre commits. En macOS, ponle al install el prefijo `CARGO_PROFILE_RELEASE_LTO=off` si el paso de link se queda sin memoria. El Rust mínimo soportado por V2 es 1.89.0; si `rustc --version` es más viejo, corre `rustup update` antes de construir.

Checkpoint: `anchor --version` imprime la cadena de la RC de V2, no `1.1.2`. Si todavía imprime la versión vieja, tu shell está resolviendo un binario más viejo antes en el `PATH`; arregla eso antes de seguir, porque todo error de compilación después de este punto sería una mentira.

### 2. Agrega la dependencia de token

El vault necesita la superficie de SPL. En `programs/quarter-vault/Cargo.toml`, agrega `anchor-spl` al lado de `anchor-lang`, las dos en la misma versión de V2. Nada que configurar más allá de eso: `token`, `token_interface`, `associated_token` y `token_2022` son todos módulos incondicionales del crate de V2.

```toml
[dependencies]
# The V2 crates are named plainly: the repo's lang-v2/ directory publishes `anchor-lang` and
# spl-v2/ publishes `anchor-spl` (its V1 directories are the ones suffixed -v1). There is no
# token-2022 feature to opt into — Token-2022 support IS token_interface.
anchor-lang = "2.0.0-rc.1"
anchor-spl  = "2.0.0-rc.1"
# The pins from m01-l2 — every program crate in this course carries them (issue #4937's class).
# They are already in this file from m03-l1; keep them when you add the SPL rows above.
wincode = { version = "0.5", features = ["derive"] }
solana-address = ">=2.6.1, <2.7"   # the arcade-workspace row from m02-l1, unchanged
```

Nota de frescura: `anchor-lang` y `anchor-spl` se mueven juntos en la línea de V2, así que fija los dos a la *misma* cadena de versión y cámbialos juntos. La trampa acá es la versión, no la fuente: los nombres de crate son idénticos en las dos líneas, así que `anchor-spl = "1"` — o un `anchor-spl = "*"` pelado que resuelve a la línea 1.x — te entrega el crate de V1 bajo el nombre de V2, los tipos de account handle no van a calzar, y te salen errores de tipos justo en la CPI. El `2.0.0-rc.1` explícito es lo que te mantiene en la línea de V2, y como el registry prohíbe republicar una versión, no puede desviarse como se desvía la punta de la rama `anchor-next`.

Checkpoint: `cargo check` resuelve los dos crates y el build falla solo en tu propio código, no en el grafo de dependencias. Si se queja de que `token_interface` no existe, resolviste el `anchor-spl` 1.x — revisa la cadena de versión, no una lista de features.

### 3. Cambia las cuentas: el PDA de lamports gana una cuenta de token

Este es el primer lugar donde se ve el diff. En R2, el valor vivía en un *segundo* PDA: el `sol_vault` propiedad del System, sembrado sobre `[b"sol", owner]`, sosteniendo los lamports. Ahora el valor vive en una cuenta de token cuya *autoridad* es el PDA de estado mismo. El PDA de estado `Vault` se queda exactamente donde estaba, seeds y bump sin cambios; el `sol_vault` se retira, y un nuevo `vault_token_account` (una ATA cuya autoridad es el PDA del vault) se hace cargo de la tenencia.

![Un diff de struct de cuentas: la cuenta del vault conserva sus seeds y su bump, el PDA sol-vault aparte se elimina, y se agregan líneas para el mint, la cuenta de token y el token program.](assets/v05-annotated-code.png)

El **camino de custodia** completo, de `initialize` a `release`. Lee `withdraw` con atención: esa es la verificación provisional que vuelves a correr.

Una tabla de disposición antes de que pegues, porque "el programa del vault" ahora quiere decir más que estos cuatro handlers, y un pegado encima borraría en silencio el resto del módulo 3 sin que nadie lo diga. La tesis de esta lección es que solo cambia la capa de custodia; acá está esa tesis desglosada artefacto por artefacto.

| artefacto | de | qué le pasa |
|---|---|---|
| el PDA `Config` y la barrera `address = config.authority` sobre `admin_set_credit` | m03-l2 | **Conservar, sin cambios.** Controla quién puede escribir los libros, y los libros no cambiaron de forma. |
| `close_vault` | m03-l2 | **Conservar, y fijarse en lo que ahora implica.** El valor del vault ya no vive en la cuenta que estás cerrando. Cierra el PDA de estado mientras su ATA todavía tiene tokens y nada en el programa los va a mover otra vez, porque todo camino que firma por el vault carga el estado que acabas de cerrar. Resguárdalo — niégate a cerrar un vault cuyo `credit` no sea cero — o drena primero. |
| `quarters::min_balance` y `require_funded` | m03-l3 | **Conservar, sin cambios.** El constraint lee `vault.credit`, que sigue ahí y sigue siendo un `u64`. |
| `set_credit` | m03-l3 | **Bórralo acá.** m03-l3 lo llamó "exactamente el handler que borrarías antes de entregar", y esta es la lección donde el vault empieza a tener valor de verdad. Un setter de credit sin barrera sentado al lado de custodia de verdad contradice el camino de depósito que estás por escribir; `deposit` es dueño de los libros de acá en adelante. |
| el PDA `sol_vault` y el campo `sol_bump` | m04-l1 | **Se va.** Esta es la única eliminación genuina que el cambio de custodia obliga: el valor se mudó a una cuenta de token, así que el PDA de lamports propiedad del System y el bump que guardaste para él no tienen más trabajo. |

Solo las últimas dos filas son eliminaciones, y solo una de ellas es obra del cambio de custodia. Esa es la tesis aguantando.

```rust
use anchor_lang::prelude::*;
use anchor_spl::{
    // `token` and `associated_token` are imported as MODULES, not just for their types: a
    // `token::mint = ...` or `associated_token::authority = ...` constraint expands to code that
    // names the module by path, so it has to be in scope or the derive fails to resolve.
    associated_token::{self, AssociatedToken},
    token,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};
// InterfaceAccount comes from anchor_lang::prelude — the alias inside
// anchor_spl::token_interface is private and cannot be imported.

declare_id!("3pX5NKLru1UBDVckynWQxsgnJeUN3N1viy36Gk9TSn8d");

#[program]
pub mod quarter_vault {
    use super::*;

    pub fn initialize(ctx: &mut Context<Initialize>) -> Result<()> {
        let authority = *ctx.accounts.authority.address();
        let vault = &mut ctx.accounts.vault;
        vault.owner = authority;
        vault.credit = 0;
        vault.bump = ctx.bumps.vault;
        Ok(())
    }

    pub fn deposit(ctx: &mut Context<Deposit>, amount: u64) -> Result<()> {
        // Deposit is signed by the depositor, a real keypair. No PDA signing here.
        let decimals = ctx.accounts.mint.decimals();
        let accounts = TransferChecked {
            from: ctx.accounts.depositor_token_account.cpi_handle_mut(),
            mint: ctx.accounts.mint.cpi_handle(),
            to: ctx.accounts.vault_token_account.cpi_handle_mut(),
            authority: ctx.accounts.depositor.cpi_handle(),
        };
        let cpi = CpiContext::new(ctx.accounts.token_program.address(), accounts);
        token_interface::transfer_checked(cpi, amount, decimals)?;

        let vault = &mut ctx.accounts.vault;
        vault.credit = vault.credit.checked_add(amount).ok_or(VaultError::Overflow)?;
        Ok(())
    }

    pub fn withdraw(ctx: &mut Context<Withdraw>, amount: u64) -> Result<()> {
        // Withdraw is signed by the vault PDA, with the SAME seeds and stored bump
        // the lamport version used. Only the verb changed.
        let decimals = ctx.accounts.mint.decimals();
        let authority_key = ctx.accounts.vault.owner;
        let bump = [ctx.accounts.vault.bump];
        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", authority_key.as_ref(), &bump]];

        let accounts = TransferChecked {
            from: ctx.accounts.vault_token_account.cpi_handle_mut(),
            mint: ctx.accounts.mint.cpi_handle(),
            to: ctx.accounts.authority_token_account.cpi_handle_mut(),
            authority: ctx.accounts.vault.cpi_handle(),
        };
        let cpi = CpiContext::new(ctx.accounts.token_program.address(), accounts)
            .with_signer(signer_seeds);
        token_interface::transfer_checked(cpi, amount, decimals)?;

        // The books debit is byte-for-byte the lamport version: checked_sub, never `-`.
        let vault = &mut ctx.accounts.vault;
        vault.credit = vault.credit.checked_sub(amount).ok_or(VaultError::Underflow)?;
        Ok(())
    }

    pub fn release(ctx: &mut Context<Release>, amount: u64) -> Result<()> {
        // Release is the composition path: R3 (the escrow) is this vault's recorded
        // authority and signs the CPI, then the vault PDA signs the token move.
        let decimals = ctx.accounts.mint.decimals();
        let authority_key = ctx.accounts.vault.owner;
        let bump = [ctx.accounts.vault.bump];
        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", authority_key.as_ref(), &bump]];

        let accounts = TransferChecked {
            from: ctx.accounts.vault_token_account.cpi_handle_mut(),
            mint: ctx.accounts.mint.cpi_handle(),
            to: ctx.accounts.recipient_token_account.cpi_handle_mut(),
            authority: ctx.accounts.vault.cpi_handle(),
        };
        let cpi = CpiContext::new(ctx.accounts.token_program.address(), accounts)
            .with_signer(signer_seeds);
        token_interface::transfer_checked(cpi, amount, decimals)?;

        let vault = &mut ctx.accounts.vault;
        vault.credit = vault.credit.checked_sub(amount).ok_or(VaultError::Underflow)?;
        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub owner: Address,  // 32  the only key allowed to authorize a move
    pub credit: u64,     //  8  the books, still debited with checked_sub
    pub bump: u8,        //  1  stored canonical bump, never re-derived
    pub _pad: [u8; 7],   //  7  explicit Pod padding (32+8+1 -> 48)
}

#[error_code]
pub enum VaultError {
    #[msg("Arithmetic overflow on the vault books")]
    Overflow,
    #[msg("Withdraw exceeds the vault balance")]
    Underflow,
}

#[derive(Accounts)]
pub struct Initialize {
    // Carried over from m04-l3's role split: the vault's owner does not have to sign
    // or pay for its creation, because an escrow PDA can do neither. The seeds derive
    // from `authority`; the rent comes from `funder`.
    /// CHECK: seeds derive from this; creating a vault for an address is not an authority action
    pub authority: UncheckedAccount,
    #[account(mut)]
    pub funder: Signer,
    #[account(
        init,
        payer = funder,
        space = Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE,
        seeds = [b"vault", authority.address().as_ref()],
        bump
    )]
    pub vault: Account<Vault>,
    pub mint: InterfaceAccount<Mint>,
    #[account(
        init,
        payer = funder,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
    pub associated_token_program: Program<AssociatedToken>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct Deposit {
    #[account(mut)]
    pub depositor: Signer,
    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<Vault>,
    pub mint: InterfaceAccount<Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_token_account: InterfaceAccount<TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
}

#[derive(Accounts)]
pub struct Withdraw {
    // has_one is deprecated in V2. The replacement is address = parent.field:
    // the signer's address must equal the authority the vault stored at init.
    #[account(mut, address = vault.owner)]
    pub authority: Signer,
    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<Vault>,
    pub mint: InterfaceAccount<Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = authority,
        associated_token::token_program = token_program,
    )]
    pub authority_token_account: InterfaceAccount<TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
}

#[derive(Accounts)]
pub struct Release {
    // The recorded authority must sign to authorize a release. In composition this
    // is the escrow PDA, which signs via its own seeds from R3.
    #[account(address = vault.owner)]
    pub authority: Signer,
    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<Vault>,
    pub mint: InterfaceAccount<Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<TokenAccount>,
    // NOT an ATA constraint: the recipient's owner is whoever the caller names, so
    // there is no owner to derive an address from. The token:: family constrains an
    // existing token account's fields instead of placing it: token::mint pins which
    // mint it holds, token::token_program pins which token program owns it. Reach
    // for token:: whenever you accept a token account you did not derive.
    #[account(
        mut,
        token::mint = mint,
        token::token_program = token_program,
    )]
    pub recipient_token_account: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
}
```

Tres líneas se ganan un comentario. El cálculo de space es `Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE`, con el `_pad` explícito manteniendo el struct en un múltiplo limpio de 8 para que el `INIT_SPACE` derivado y el layout Pod concuerden exacto. El campo `owner` es `Address`, no `Pubkey`; V2 le cambió el nombre al tipo de clave, y `.address()` es cómo lees una clave de un firmante o de una cuenta (reemplaza a `.key()`). Y la línea de autoridad de `withdraw`, `address = vault.owner`, es el reemplazo congelado de `has_one`: la misma garantía, una forma nueva de escribirla.

Checkpoint: `anchor build` compila el programa con las cuentas nuevas. Todavía no va a hacer nada útil, pero debería estar verde.

### 4. Cambia la transferencia: la cuenta de custodia se mueve, la jugada de firma no

Este es el corazón del asunto, y quiero que te fijes con precisión en qué se movió y qué no. En la versión de lamports firmabas con los seeds propios de la cuenta de *custodia*, `[b"sol", owner, &[state.sol_bump]]`, porque los lamports se sentaban en ese segundo PDA. Bajo SPL no hay segundo PDA. Los tokens se sientan en una ATA cuya autoridad es el PDA de estado, así que firmas con los seeds del PDA de estado, `[b"vault", authority_key.as_ref(), &bump]`, y `sol_bump` se retira junto con la cuenta que describía.

El array de seeds cambió. La *jugada* no, y esa es la parte transferible: reconstruye los seeds a partir de un bump que guardaste en el init, nunca lo vuelvas a derivar, engancha `.with_signer`, y el runtime le otorga al PDA el privilegio de signer sin importar si la llamada interna es una transferencia del System Program o un `transfer_checked` de token. Los seeds son la firma, y ese mecanismo es agnóstico a la custodia.

![Un antes y después de la CPI de withdraw: la jugada de firma no cambia, mientras que la transferencia de lamports del System Program se vuelve un transfer_checked que lleva el mint y un argumento decimals al final.](assets/v06-annotated-code.png)

Cuatro cosas en ese bloque `AFTER` son específicas de V2 y vale la pena nombrarlas, porque la memoria muscular de 1.x (la versión que todavía está en tu máquina) te va a pelear en cada una.

- `cpi_handle()` y `cpi_handle_mut()` reemplazan a `.to_account_info()` cuando construyes las cuentas de la CPI. V2 rutea las CPIs de token por un camino de handle más barato y con borrow rastreado; usa `_mut` para las cuentas cuyos balances cambian (`from`, `to`) y el simple para las de solo lectura (`mint`, `authority`).
- `CpiContext::new` toma el programa como una dirección, `token_program.address()`, no como un `AccountInfo`. Ese es el mismo cambio de V2 que ya conociste en la CPI del System Program.
- El handler toma `&mut Context<T>`, no `Context<T>`. Las firmas de handler de V2 son de contexto mutable por defecto.
- `.with_signer(signer_seeds)` es lo que convierte una CPI simple en una firmada por PDA. Omítelo y esta llamada exacta se vuelve una transferencia sin firma que el runtime rechaza, porque el PDA del vault nunca la autorizó.

![El programa reconstruye sus signer seeds a partir del bump guardado y llama a transfer_checked; el token program verifica que esas seeds reproducen el PDA del vault antes de mover el balance.](assets/v07-diagram.png)

Una cosa más para tener en la cabeza antes de correrlo: el orden en que el runtime hace todo esto, porque conocer la secuencia es cómo ubicas una falla en la línea correcta en vez de adivinar.

![Un diagrama de flujo vertical que recorre el withdraw SPL en ocho pasos, desde la carga de cuentas y el constraint address, pasando por el transfer_checked firmado por el PDA, hasta el débito con checked_sub, con avisos de falla por paso.](assets/v08-flowchart.png)

Checkpoint: `anchor build` está verde, y puedes leer `withdraw` de arriba a abajo y nombrar cada línea como identidad (sin cambios) o custodia (cambiada), y apuntar en cuál de los ocho pasos vive.

### 5. Vuelve a correr la verificación provisional de R2 en SPL

La verificación de R2 siempre hizo la misma afirmación: el valor sale del vault *solo* bajo la firma del PDA del vault. En la versión de lamports afirmabas sobre balances de lamports. Ahora afirmas sobre el balance de tokens SPL. La misma prueba, una unidad nueva. La plantilla de prueba default de V2 es LiteSVM en Rust, así que nos quedamos ahí.

```bash
# The test surface. anchor-v2-testing is the V2 harness the scaffold generates
# against; it wraps LiteSVM and pins its own LiteSVM version, so add it rather
# than a bare litesvm that could resolve to a different one. Pin the TAG, not the
# branch: the tag carries litesvm 0.11.0 and the anchor-next tip has already moved
# to 0.13.1, and two SVM crates in one graph is the skew this pin exists to avoid.
cargo add anchor-v2-testing --dev \
  --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1
# The fixture's two SPL crates, pinned like everything else here: these are the
# versions that resolve against the rc.1 pin set (verified 2026-09-01).
cargo add spl-token@9 spl-associated-token-account@8 --dev
```

El setup es más que un comentario, así que acá está completo. Ponlo en `programs/quarter-vault/tests/spl_setup.rs`, al lado del crate del programa, que es lo que hace alcanzable el `.so` construido:

```rust
// tests/spl_setup.rs - the fixture. Creates a 6-decimal mint, initializes the
// vault and its ATA, mints into the authority's ATA, and deposits 1.0 token.
// Imports ride anchor_lang and anchor_v2_testing and reach past neither: the
// instruction plumbing is anchor_lang's, the signing and SVM plumbing is the
// harness's, and nothing here names a solana crate the Cargo.toml never declared.
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction};

// Cargo compiles every tests/*.rs as its own crate root, so a sibling `mod spl_helpers`
// in the test file is not in scope here. Pull the helpers in by path instead; the test
// file then declares only `mod spl_setup;`.
#[path = "spl_helpers.rs"]
mod spl_helpers;

pub const DECIMALS: u8 = 6;
pub const ONE_TOKEN: u64 = 1_000_000;

pub struct Ctx {
    pub authority: Keypair,
    pub mint: Address,
    pub vault: Address,
    pub vault_ata: Address,
    pub authority_ata: Address,
}

// LiteSVM's own failure type is not among anchor_v2_testing's re-exports, and adding
// litesvm by name is exactly the skew this lesson's pin avoids, so the error is
// flattened to a String at the boundary. Callers only ever ask "did it fail, and why".
pub fn send(svm: &mut LiteSVM, payer: &Keypair, signers: &[&Keypair], ixs: &[Instruction])
    -> Result<(), String>
{
    let bh = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &bh);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx).map(|_| ()).map_err(|e| format!("{:?}", e.err))
}

pub fn token_balance(svm: &LiteSVM, ata: &Address) -> u64 {
    let acct = svm.get_account(ata).expect("token account exists");
    // SPL token account layout: amount is a little-endian u64 at offset 64.
    u64::from_le_bytes(acct.data[64..72].try_into().unwrap())
}

pub fn setup(svm: &mut LiteSVM) -> Ctx {
    let program_id = quarter_vault::ID;
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    svm.add_program_from_file(program_id, vault_so).unwrap();

    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 5_000_000_000).unwrap();

    // The rent gets its own payer. Initialize takes authority AND funder — the
    // m04-l3 role split — and funder is the mut one, so handing one keypair to
    // both slots is exactly the aliasing the duplicate-mutable default rejects
    // at runtime (ConstraintDuplicateMutableAccount). Distinct keys, no friction.
    let funder = Keypair::new();
    svm.airdrop(&funder.pubkey(), 5_000_000_000).unwrap();

    // 1. the mint, created and minted with the spl-token helpers
    let mint = spl_helpers::create_mint(svm, &authority, DECIMALS);
    let authority_ata = spl_helpers::create_ata(svm, &authority, &mint, &authority.pubkey());
    spl_helpers::mint_to(svm, &authority, &mint, &authority_ata, ONE_TOKEN);

    // 2. the vault PDA and its ATA, created by the program's own initialize
    let (vault, _b) = Address::find_program_address(
        &[b"vault", authority.pubkey().as_ref()], &program_id);
    let vault_ata = spl_helpers::ata_address(&mint, &vault);
    let init = Instruction {
        program_id,
        accounts: quarter_vault::accounts::Initialize {
            authority: authority.pubkey(),
            funder: funder.pubkey(),
            vault,
            mint,
            vault_token_account: vault_ata,
            token_program: spl_token::ID,
            associated_token_program: spl_associated_token_account::ID,
            system_program: System::id(),
        }.to_account_metas(None),
        data: quarter_vault::instruction::Initialize {}.data(),
    };
    send(svm, &funder, &[&funder], &[init]).unwrap();

    // 3. deposit 1.0 token into the vault ATA
    let dep = Instruction {
        program_id,
        accounts: quarter_vault::accounts::Deposit {
            depositor: authority.pubkey(),
            vault,
            mint,
            depositor_token_account: authority_ata,
            vault_token_account: vault_ata,
            token_program: spl_token::ID,
        }.to_account_metas(None),
        data: quarter_vault::instruction::Deposit { amount: ONE_TOKEN }.data(),
    };
    send(svm, &authority, &[&authority], &[dep]).unwrap();

    Ctx { authority, mint, vault, vault_ata, authority_ata }
}

pub fn send_withdraw(svm: &mut LiteSVM, ctx: &Ctx, amount: u64) -> Result<(), String> {
    let ix = Instruction {
        program_id: quarter_vault::ID,
        accounts: quarter_vault::accounts::Withdraw {
            authority: ctx.authority.pubkey(),
            vault: ctx.vault,
            mint: ctx.mint,
            authority_token_account: ctx.authority_ata,
            vault_token_account: ctx.vault_ata,
            token_program: spl_token::ID,
        }.to_account_metas(None),
        data: quarter_vault::instruction::Withdraw { amount }.data(),
    };
    send(svm, &ctx.authority, &[&ctx.authority], &[ix])
}
```

Ese `spl_helpers` es el wrapper fino sobre `spl_token` y `spl_associated_token_account` que construye las cuatro instrucciones de setup (`create_mint`, `create_ata`, `mint_to`, `ata_address`). Es código de cliente SPL ordinario, sin nada específico de V2 adentro, así que se entrega junto a esta lección como [spl-helpers/spl_helpers.rs](spl-helpers/spl_helpers.rs); ponlo como `tests/spl_helpers.rs`.

Ahora la verificación en sí, en `programs/quarter-vault/tests/vault_spl_withdraw.rs`:

```rust
mod spl_setup;   // it pulls spl_helpers in itself, by #[path]

use anchor_v2_testing::svm;

#[test]
fn withdraw_moves_the_spl_balance_under_the_vault_signature() {
    let mut svm = svm();
    let ctx = spl_setup::setup(&mut svm); // 1.0 token (6 decimals) now sits in the vault ATA

    let before = spl_setup::token_balance(&svm, &ctx.vault_ata);
    assert_eq!(before, spl_setup::ONE_TOKEN, "vault holds 1.0 token before withdraw");

    // the vault PDA signs the move out; the human authority only authorizes it
    spl_setup::send_withdraw(&mut svm, &ctx, spl_setup::ONE_TOKEN)
        .expect("PDA-signed withdraw should succeed");

    let after = spl_setup::token_balance(&svm, &ctx.vault_ata);
    assert_eq!(after, 0, "the vault PDA signed the tokens out");

    // an over-withdraw is rejected, not panicked. Be precise about who refuses:
    // the token program fails the transfer CPI on insufficient funds before your
    // checked_sub ever runs; the ledger guard exists for drift the token balance
    // cannot see, and this assert only proves the refusal path is an Err.
    assert!(
        spl_setup::send_withdraw(&mut svm, &ctx, 1_000_000_000).is_err(),
        "over-withdraw must return an error, never a wrap or a panic"
    );
}
```

Córrelo:

```bash
anchor build && cargo test --test vault_spl_withdraw
```

Checkpoint: verde. El balance de tokens salió de la ATA del vault, y lo único que autorizó el movimiento fueron los seeds del PDA del vault. Si comentas `.with_signer(signer_seeds)` en el handler y vuelves a correr, esta prueba debería fallar con un error de firma faltante. Esa falla es la demostración de que el PDA es lo que autoriza el movimiento. Descoméntalo y sigue.

## Challenge: enhebra el release del escrow, después vuelve a correr los dos en SPL

Acá entra el repliegue. El vault está resuelto. El escrow es tuyo, y es un cambio más chico de lo que podrías temer, por una razón que es toda la recompensa del diseño de la lección pasada.

El prize-escrow (R3), el programa `quarter-prize` que construiste la lección pasada, nunca custodió el premio en sí. Delegó la custodia a una instancia de vault de R2 y la alcanzó por una CPI. Así que cuando la custodia de R2 se fue a SPL, la *política* del escrow no cambió para nada: `reserve` sigue registrando el maker, el player, el vault, el amount y el winning score; `redeem` sigue verificando `final_score >= escrow.winning_score` antes de que se mueva cualquier cosa; el llamador sigue fijado con `address = escrow.player`; el escrow sigue cerrando hacia el `maker` registrado para que el rent vuelva al operador que lo pagó; el escrow sigue firmando como su propio PDA con `[b"escrow", maker, player, bump]`, seeds y orden sin cambios. Lo único que cambió es lo que el escrow *pasa hacia abajo* al vault: el mint y las cuentas de token ahora viajan en las CPIs de `deposit` y de `release`.

Esa es la oración con la que hay que quedarse un rato. Porque construiste sobre un vault en vez de meter la custodia en línea, la migración a SPL nunca sale del borde de CPI del escrow: cambian las cuentas que enhebra hacia la CPI del vault, y `reserve` sigue un solo renombre mecánico, la instrucción de init del vault pasando de `init_vault` a `initialize`. Política, condición, pin del llamador, firma por PDA: sin tocar.

![Los campos de política del escrow, la verificación de condición, el pin del llamador y la firma por PDA no cambian; solo cambia el borde de CPI, que ahora lleva el mint y las cuentas de token.](assets/v09-diagram.png)

Acá está `redeem`, con la guarda de condición y la firma del escrow ya puestas, y el cableado de cuentas para la CPI de release dejado como tu hueco. La respuesta está impresa dentro del bloque marcado en vez de retenida, porque seis nombres de campo sin un compilador delante es un juego de adivinanzas, no un ejercicio. Tápalo con la mano, escribe el struct a partir de lo que te enseñó el vault, después destapa y compara. El hueco que es genuinamente tuyo es la sección Solo de abajo.

```rust
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::state::Escrow;
use crate::error::EscrowError;
// The vault's module is the one declare_program! generated back in m04-l3 —
// marker included. Re-harvest idls/quarter_vault.json after this lesson's SPL
// upgrade lands, because R2's interface just changed shape. And if this handler
// lives in an instructions submodule rather than lib.rs, spell the path from the
// crate root — `use crate::quarter_vault::…` — the macro generates the module at
// the root, and a submodule's bare `use quarter_vault::…` cannot see it.
use quarter_vault::cpi as vault_cpi;
use quarter_vault::program::QuarterVault;

pub fn handler(ctx: &mut Context<Redeem>, final_score: u64) -> Result<()> {
    // GUARD (unchanged from the lamport escrow): condition before payout.
    require!(
        final_score >= ctx.accounts.escrow.winning_score,
        EscrowError::ConditionNotMet
    );

    // Read escrow state out before any CpiHandle goes live (the borrow model).
    let amount = ctx.accounts.escrow.amount;
    let maker = ctx.accounts.escrow.maker;
    let player = ctx.accounts.escrow.player;
    let bump = ctx.accounts.escrow.bump;

    // The escrow signs as ITS OWN PDA to satisfy the vault's address = vault.owner.
    // Seeds unchanged from the lamport version, order included.
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"escrow",
        maker.as_ref(),
        player.as_ref(),
        &[bump],
    ]];

    // === YOUR LINE ===
    // Build vault_cpi::accounts::Release. In the lamport version it was
    //   { authority, vault, recipient, system_program }.
    // Now the vault moves TOKENS, so thread the SPL custody accounts through:
    // the mint, the vault's token account (from), the winner's token account (to),
    // and the token program. That mint account is how decimals reach transfer_checked.
    let accounts = vault_cpi::accounts::Release {
        authority: ctx.accounts.escrow.cpi_handle(),
        vault: ctx.accounts.vault.cpi_handle_mut(),
        mint: ctx.accounts.mint.cpi_handle(),
        vault_token_account: ctx.accounts.vault_token_account.cpi_handle_mut(),
        recipient_token_account: ctx.accounts.winner_token_account.cpi_handle_mut(),
        token_program: ctx.accounts.token_program.cpi_handle(),
    };
    // === END YOUR LINE ===

    let cpi = CpiContext::new(ctx.accounts.quarter_vault_program.address(), accounts)
        .with_signer(signer_seeds);
    vault_cpi::release(cpi, amount)?;
    Ok(())
}
```

Fíjate en lo que *no* está ahí: ningún `transfer_checked`, ningún argumento decimals, ninguna búsqueda de decimals del mint. Todo eso vive en el `release` de R2, que trabajaste en el lab. El trabajo del escrow es solamente entregarle al vault las cuentas correctas y firmar como la autoridad. Los decimals viajan dentro de la cuenta `mint` que enhebraste. Este es el dividendo de la composición: la primitiva vive en un solo lugar, y el llamador nada más apunta a los tokens correctos.

El struct de cuentas `Redeem` gana las mismas cuatro cuentas de custodia, y conserva cada línea de política:

```rust
#[derive(Accounts)]
pub struct Redeem {
    // caller guard, unchanged: only the recorded player may redeem
    #[account(address = escrow.player)]
    pub player: Signer,
    #[account(
        mut,
        close = maker,
        seeds = [b"escrow", escrow.maker.as_ref(), escrow.player.as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<Escrow>,
    /// CHECK: close destination only, pinned to the maker the escrow recorded.
    #[account(mut, address = escrow.maker)]
    pub maker: UncheckedAccount,
    /// CHECK: address fixed by seeds; the quarter_vault program validates and signs it.
    #[account(
        mut,
        seeds = [b"vault", escrow.address().as_ref()],
        bump,
        seeds::program = quarter_vault_program.address(),
    )]
    pub vault: UncheckedAccount,
    pub mint: InterfaceAccount<Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = player,
        associated_token::token_program = token_program,
    )]
    pub winner_token_account: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
    pub quarter_vault_program: Program<QuarterVault>,
    pub system_program: Program<System>,
}
```

Dos líneas de ahí vienen de la versión de lamports y son fáciles de perder en una reescritura, así que nómbralas: `close = maker` en el escrow, y el lugar de `maker` que necesita como destino de close, fijado con `address = escrow.maker`. Omite cualquiera de las dos y un escrow redimido se queda abierto con su rent varado — la línea de política sobrevive al cambio de custodia exactamente como el resto.

Un constraint de ahí es nuevo, así que llévate la versión de treinta segundos ahora en vez de adivinar. `seeds::program = quarter_vault_program.address()` le dice a Anchor que derive ese PDA contra el id de *otro* programa en vez del tuyo. Lo necesitas porque el PDA del vault pertenece a `quarter_vault`, no al escrow: sin esa línea Anchor derivaría `[b"vault"...]` bajo el id de programa del escrow, conseguiría una dirección distinta, y rechazaría la cuenta que de verdad querías. Cada vez que restrinjas un PDA del que es dueño un programa al que estás llamando, `seeds::program` es la línea que apunta la derivación al dueño correcto.

Después la parte Solo: las dos verificaciones provisionales en SPL, desde cero, sin ningún handler resuelto delante de ti.

1. **R2 en SPL, el withdraw.** Esto lo corriste en el lab. Hazlo una vez más sin mirar: crea el mint, inicializa el vault y su ATA, haz deposit, haz withdraw, y afirma que el balance de tokens del vault se fue a cero *y* que quitar `.with_signer` lo hace fallar. Esa falla es el punto de la verificación.
2. **R3 en SPL, el release condicional.** Haz que el operador haga `reserve` de un premio en la instancia de vault del escrow. Una cosa que hay que acertar antes de empezar, porque es el paso que traba a la gente: el vault del escrow todavía no existe, y el PDA del escrow no puede traerlo a la existencia por su cuenta. No porque esté quebrado — un PDA exento de alquiler tiene lamports por definición, y el propio vault de lamports de este curso era un PDA cuyo único trabajo era tenerlos — sino por quién puede gastarlos: al que paga el rent en un `init` lo debita el System Program, y el System Program solo debita cuentas de las que es dueño. El PDA del escrow carga datos y pertenece al programa del escrow, así que nunca puede sentarse en el lugar del payer. Por eso exactamente `Initialize` ganó un `funder` en el paso 3 del lab. Así que `reserve` primero le hace CPI al `initialize` del vault — cuidado con el nombre: el upgrade a SPL le cambió el nombre a la instrucción de init del vault, del `init_vault` de la versión de lamports a `initialize`, así que el módulo que genera tu IDL recosechado expone `initialize`, y el cableado `vault_cpi::accounts::InitVault` de la era de m04-l3 se lleva el nombre nuevo con él — con el **maker** como `funder` y el **PDA del escrow** como `authority`, después le hace CPI a `deposit`, la misma forma de dos llamadas que la versión de lamports. Después llama a `redeem` con un `final_score` debajo de `winning_score` y afirma que da error con `ConditionNotMet` (nada se mueve). Después `redeem` con un puntaje ganador y afirma que el premio aterrizó en la cuenta de token del ganador. Verde quiere decir que la barrera de condición aguanta y que el PDA del escrow es lo único que puede firmar el premio hacia afuera.

Aceptación: `withdraw` mueve el balance de tokens solo bajo la firma del PDA del vault; el escrow libera solo con la condición correcta, solo hacia el player registrado; las dos en verde. Y puedes decir, en una oración, qué líneas cambiaron respecto de la versión de lamports. Si tu oración corre más larga que "la capa de custodia del vault, el mint y las cuentas de token que el escrow enhebra hacia la CPI de release, y el renombre de `init_vault` a `initialize` que `reserve` sigue", cambiaste más de lo que necesitabas.

## Dónde te deja esto

Acá está el loop de feedback, honesto. Si tu `redeem` del escrow compiló en el primer intento, el vault te enseñó el patrón y lo transferiste. Bien. Si no, la falla fue casi seguro una de tres, en el orden en que suelen morder: echaste mano de un `transfer` pelado en vez de `transfer_checked` en algún lugar del vault, te olvidaste de la cuenta del mint (así que los decimals nunca llegaron a `transfer_checked`), o omitiste un `.with_signer` y el runtime rechazó un movimiento de PDA sin firmar. Cada una de esas es un error de la capa de custodia, no un error de identidad, que es la lección aterrizando: el esqueleto que construiste en R2 y R3 estaba correcto, y cambiar lo que tiene no rompió quién es.

![Una tabla de diagnóstico de tres filas que mapea un transfer pelado, una cuenta de mint faltante y una signer seed omitida a sus arreglos, siendo las tres errores de custodia y no errores de identidad.](assets/v10-table.png)

Ese es un hito de verdad, así que nómbralo por lo que costó. Acabas de demostrar que el diseño de un programa que funciona sobrevive a su custodia. El prototipo de lamports no era descartable. Era el esqueleto, y el esqueleto aguantó. Y el escrow demostró la segunda afirmación, la más filosa: porque delegó la custodia al vault en vez de meterla en línea, la migración a tokens nunca salió de su borde de CPI. Eso no es suerte. Eso es lo que te compra construir sobre un vault, y es la misma razón por la que un protocolo de verdad separa la política de la custodia.

Ahora hay dos programas SPL en tu workspace: un vault que no cotiza nada y un escrow que no cotiza nada. Solo mueven tokens que ya tienen. Pero los jugadores tienen tokens de arcade y quieren tickets, y ninguno de estos dos programas tiene una opinión sobre el precio. La próxima lección construyes un tercer programa, un pool que cotiza su propia tasa desde sus reservas e intercambia tokens por tickets, sosteniendo cada lado en una cuenta de token propia. Ni el vault ni el escrow se suman a ese build, y eso es deliberado: la autoridad de una reserva tiene que ser el pool, así que no puede ser también un vault. Lo que se traspasa es la forma y no el artefacto. El `transfer_checked` firmado por el PDA que acabas de escribir es el pago del swap, y el release condicional que escribiste en R3 es exactamente el flujo de control que necesita la guarda de slippage del swap. El mismo asiento de PDA-y-CPI que acabas de aprender, un trabajo nuevo: poner precio.
