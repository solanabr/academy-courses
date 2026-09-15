# Capstone de migración: porta un programa 0.31/1.0 a V2

En m10-l2 terminaste el segundo mapa. Cada delta de reescritura de 1.x a 2.0, desde `Pubkey` volviéndose `Address` hasta el `AccountLoader` repropuesto. También manejaste dos ports chicos: un programa de config de cincuenta líneas en el lab, con la tabla de delta abierta al lado, y un cálculo de space en el challenge. Así que: dos mapas y dos ejercicios de borrador. Lo que no hiciste es llevar una base de código que no escribiste al otro lado de la línea. Eso cambia ahora.

Porque acá está la cosa sobre un mapa: no es el manejo. Puedes leer los dos mapas de delta de punta a punta, asentir con cada flecha, y todavía congelarte la primera vez que una base de código de verdad te tira una pared de texto de error rojo. Así que vamos a tomar un programa de verdad, uno que no compila en V2, y manejarlo hasta el verde juntos. No admirar los deltas. Aplicarlos.

Antes de que leas otro párrafo, corre esto y mira el número que imprime:

```bash
anchor --version
```

En la máquina de referencia de este curso eso dice `anchor-cli 1.1.2`. Agárrate de ese número — no porque el CLI decida qué es tu build (no lo hace, y el hecho más importante de esta lección es *qué sí lo hace*), sino porque una máquina que imprime 1.x es una máquina cuyos hábitos, y cuyo vault entregado, siguen fijados a los crates 1.x. El pin es la historia. Volvemos a él en el paso 1.

## Resumen

Te entregan un vault de lamports de Anchor 0.31/1.0 funcionando: una cuenta de estado, un PDA que sostiene SOL, y tres instrucciones (`initialize`, `deposit`, y un `withdraw` firmado por PDA). Compila bien en 1.x. No compila en la RC de Anchor V2. Tu trabajo es hacer que el compilador se ponga verde y que la prueba de LiteSVM pase, trabajando los dos mapas de delta como un checklist.

El vault se eligió para ser aburrido a propósito. Hace eco del quarter-vault que construiste allá en m03 y m04, así que casi nada de tu atención va a "qué hace este programa". Toda va a la migración en sí. Ese es el diseño entero: carga de dominio baja, foco de migración alto.

La mayor parte de los deltas es mecánica, y el programa provisto los marca por ti con comentarios `// TODO(migrate):`. Los dos que más importan no llevan ninguna marca, porque para el final no vas a necesitar una. El compilador te lo va a decir. Una advertencia de deprecación subraya el constraint exacto que hay que cambiar. Un método que falta apunta a la línea exacta donde un hábito de v1 ya no tiene nada a lo que llamar. Ese es el núcleo emocional de este capstone: aprendiste lo suficiente para que la propia salida de las herramientas sea una guía lo bastante buena. Le decimos *dejar que el compilador maneje*, y es una facilidad de verdad de V2, no un eslogan motivacional. Vas a ver por qué.

Una reserva honesta de entrada, porque le da forma a todo: este port compila hoy contra un blanco en movimiento. La línea de Anchor V2 es `2.0.0-rc.1`, y la rama anchor-next en la que vive está etiquetada de alpha por sus propios mantenedores: no auditada, las API pueden romperse entre commits, y la documentación va detrás del código (los crates de rc.1 llegaron a crates.io el 2026-08-12, y aun así la documentación de instalación sigue describiendo el mundo pre-publicación). Así que construimos esto no como un artefacto eterno sino como uno re-verificable. Cuando una RC posterior renombre un constraint debajo de ti, vuelves a correr el checklist. Esa es la realidad de la migración, y la lección final de este curso pregunta si deberías anotarte en eso para nada.

## La migración, delta por delta

Pongamos el toolchain bien primero, porque cada otro delta viene después de él.

### El toolchain es el partido entero

¿Te acuerdas de ese `1.1.2` de hace un minuto? Acá está la trampa que arma, y es más sutil que "binario equivocado". El vault entregado compila bien en 1.x, que quiere decir que su `Cargo.toml` fija `anchor-lang` en la línea 1.x — y *ese pin, no el CLI de tu PATH, es lo que selecciona el major del framework*. `anchor build` es un wrapper; debajo, cargo resuelve tu grafo de crates idénticamente sin importar qué anchor-cli lo invocó. Así que si clonas el vault, empiezas a arreglar nombres de tipo, y nunca tocas el manifiesto, no obtienes un artefacto silencioso de v1 — obtienes una falla fuerte: `Address`, `.address()`, `&mut Context` no existen en ninguna parte de los crates 1.x, y el compilador lo dice en cada sitio que acabas de editar. Lo inverso también vale: sube el pin a `2.0.0-rc.1` y hasta el CLI viejo del host saca a la superficie las deprecaciones de V2 y el error de método que falta, porque esos diagnósticos vienen de las macros del grafo de dependencias, no del binario que llamó a cargo. Ya te encontraste con esta inversión dos veces — el reconocimiento de m10-l1 te hizo correr `rg "anchor_version|anchor-lang"` precisamente porque el pin es el hecho que importa, y m10-l2 te dijo que fijaras la versión exacta en `Anchor.toml` y en `Cargo.toml`. La versión que el *build* es, es la versión que dice el *manifiesto*.

Así que la primera jugada no es una edición de handler. Son dos pins, hechos juntos: la fila `anchor-lang = "2.0.0-rc.1"` del `Cargo.toml` del programa, que es la llave que de verdad da vuelta el major, y un CLI V2 aislado, que mantiene cada comportamiento de nivel de wrapper — scaffolds, el banco de pruebas, el manejo de IDL — en la misma línea que los crates, para que `anchor --version` siga siendo una etiqueta veraz del toolchain entero.

La instalación te pelea un poco, y vale saber por qué. V2 no tiene objeto de GitHub Release. Hay un tag de git, `v2.0.0-rc.1` sobre la rama anchor-next, pero ningún release publicado para ese tag, que quiere decir que `avm install` no puede descargar un binario precompilado para él como lo hace para las versiones estables — la URL del asset simplemente da 404. Los crates de rc.1 sí aterrizaron en crates.io el 2026-08-12, pero la documentación va detrás de esa publicación y el camino documentado es una instalación directa por git (la documentación apunta a la punta de la rama `anchor-next`; este curso fija el tag que está sobre esa rama, por la razón de reproducibilidad que expuso m01-l2):

```bash
# Anchor V2 RC - installed straight from the anchor-next repo by tag.
# No GitHub Release cut for the tag, so `avm install` finds no binary to fetch.
# macOS needs LTO off or the release build blows up; harmless elsewhere.
CARGO_PROFILE_RELEASE_LTO=off \
cargo install --git https://github.com/otter-sec/anchor \
  --tag v2.0.0-rc.1 anchor-cli --locked --force
```

Nota de frescura: `v2.0.0-rc.1` es el pin al 2026-08-22, y está sobre una rama que su propia documentación llama alpha con API que pueden romperse entre commits. Antes de confiar en un build, vuelve a verificar el tag actual en anchor-next y actualiza el pin. Esta no es una versión para memorizar; es una para re-verificar.

Dos hechos de toolchain más que necesitas. El Rust mínimo soportado de V2 es `1.89.0`, hardcoded como `ANCHOR_MSRV` en el CLI y escrito adentro del `rust-toolchain.toml` del que hace scaffold, así que confirma tu compilador y actualiza si estás atrás:

```bash
rustc --version      # need >= 1.89.0 for V2
rustup update        # if you are below it
```

Y no puedes dejar que el 1.1.2 del ambiente se filtre de vuelta durante la verificación. La forma de garantizar eso es fijar la RC en el contenedor de verify para que el build nunca toque el toolchain del host:

```dockerfile
# verify/Dockerfile - the port builds ONLY against the pinned RC.
FROM rust:1.89
ENV CARGO_PROFILE_RELEASE_LTO=off
RUN cargo install --git https://github.com/otter-sec/anchor \
      --tag v2.0.0-rc.1 anchor-cli --locked --force
WORKDIR /work
COPY . .
CMD ["anchor", "test"]
```

![Construir el vault editado contra un Cargo.toml que todavía fija anchor-lang 1.x falla en cada línea editada bajo cualquier CLI; solo un grafo fijado en 2.0.0-rc.1 emite las deprecaciones de V2 y el error de método que falta, y un build V2 de verdad.](assets/v01-flowchart.png)

Ese es el armado sobre el que se para todo lo demás. Equivócate y cada edición de código de abajo es teatro. Hazlo bien y el compilador empieza a hacer tu trabajo por ti.

### El mapa de delta, en una página

Acá está cada delta de reescritura que el vault toca, lado a lado. Este es el checklist. Mantenlo abierto mientras trabajas.

| # | v1 (0.31/1.0) | V2 (2.0.0-rc.1) | cómo lo encuentras |
|---|---|---|---|
| 1 | `Pubkey` | `Address` | error de tipo en el campo |
| 2 | `account.key()` | `account.address()` | error de método-no-encontrado |
| 3 | `struct Foo<'info>` + `ctx: Context<Foo>` | suelta el `<'info>`, el handler toma `&mut Context<Foo>` | error de lifetime / de firma |
| 4 | `space = 8 + T::INIT_SPACE` | `space = T::DISCRIMINATOR.len() + T::INIT_SPACE` | todavía compila, pero el mapa dice que lo arregles |
| 5 | `CpiContext::new(prog.key()..)` | `CpiContext::new(prog.address()..)`, las cuentas de CPI se vuelven `.cpi_handle_mut()` | error de tipo: esperaba `&Address` |
| 6 | `has_one = authority` | `address = state.authority` sobre la cuenta de authority | **una advertencia de deprecación lo subraya** |
| 7 | `account.reload()?` después de una CPI | bórralo; nada cambia debajo de una cuenta tipada cargada | **E0599: ningún método llamado `reload`** |

Una nota sobre la fila 5 antes de que uses la tabla. El vault que te entregan compila en 1.1.2, así que su CPI ya pasa el programa como un `Pubkey` con `.key()`; ese salto fue el cambio dos de m10-l1. Si la base de código que traes a un port de verdad todavía está en 0.31 va a leer `.to_account_info()` ahí en cambio, y haces los dos saltos a la vez.

Dos filas faltan a propósito, y las dos valen una frase para que tu mapa quede completo aunque este vault no las haga tropezar.

`zero_copy` ahora es el layout default en V2, así que el atributo simplemente desapareció. Nuestro vault nunca lo usó, así que no hay nada que arrancar. En un programa que sí lo usaba, borrarías el atributo y la cuenta sigue funcionando, porque lo que antes era un opt-in ahora es solo cómo se disponen las cuentas.

`unsafe(dup)` es el más interesante, y el challenge te va a hacer usarlo, así que entiéndelo ahora. V2 no permite cuentas mutables duplicadas por defecto. La razón es una trampa de verdad: si la misma cuenta llega en dos slots mutables, tu handler termina sosteniendo dos referencias `&mut` a una cuenta, y las ediciones a través de una calladamente pisan las ediciones a través de la otra. v1 te dejaba hacer esto y esperaba que supieras lo que hacías. V2 lo rechaza, en la validación, antes de que corra tu handler. Cuando genuinamente quieres que te entreguen una cuenta bajo dos nombres mutables, te vuelves a anotar por campo escribiendo el constraint `unsafe(dup)`. La palabra `unsafe` está haciendo trabajo honesto: es tú diciéndole al compilador que verificaste el invariante que él ya no puede verificar por ti, y asumiendo la obligación de escribir el handler para que nunca sostenga dos referencias mutables en conflicto. Nota lo que *no* necesita el opt-out: dos slots mutables que siempre resuelven a dos direcciones distintas, como hacen las dos reservas de un swap, satisfacen la verificación gratis. Nuestro vault de lab tiene exactamente una cuenta de cada tipo, así que nunca aparece. La barrida de consolidación del challenge sí.

![Una tabla agrupada de antes/después de siete deltas: cinco que el compilador marca como errores de tipo, dos que saca a la superficie como una advertencia de deprecación y un error de método que falta.](assets/v02-comparison.png)

### Por qué `.reload()` desapareció (y por qué eso es bueno)

Las filas 1 a 5 son find-and-replace con un compilador verificando tu trabajo. Las filas 6 y 7 son las que vale entender, porque son donde el port deja de ser mecánico.

Empieza por `.reload()`, porque es el que hace trastabillar a cada dev de v1 con experiencia, y el razonamiento detrás de su remoción es la idea más interesante de la migración entera.

Acá está el patrón de v1, y no es ni malo:

```rust
// v1 withdraw tail: build the transfer, read state while it is pending, run it, reload.
let cpi_ctx = CpiContext::new_with_signer(sys, Transfer { from, to }, signer);
let before = ctx.accounts.state.total_withdrawn; // legal in v1: `from`/`to` are AccountInfo clones
system_program::transfer(cpi_ctx, amount)?;
ctx.accounts.state.reload()?;                    // re-deserialize after the CPI
let bal = ctx.accounts.state.total_deposited;    // read the "fresh" value
```

Derivaste la remoción la lección pasada: las lecturas tipadas obsoletas a través de una frontera de CPI eran un bug con forma de parche, y el rastreo de borrow de `CpiHandle` las vuelve inexpresables — un borrow por cuenta entregada a la CPI, y nada más ancho, exactamente como lo recorrió m04-l2. Ninguna re-derivación acá. Lo que esta lección agrega es el *borde* de esa regla, porque es más angosta de lo que suena al principio y este vault se para justo en él. Porta esa cola línea por línea y exactamente una línea detiene el build: `.reload()`, que ya no existe como método al que llamar. La lectura de `total_withdrawn` una línea más arriba sobrevive, porque los handles de esta transferencia están sobre `vault` y `authority` mientras `state` es una cuenta disjunta que el llamado nunca podría haber escrito — nada que excluir ahí, y nada que pudiera haber quedado obsoleto. Eso es el modelo haciendo su trabajo, no fallando: la exclusión aterriza sobre las cuentas que la CPI de verdad puede cambiar. Entrégale una cuenta tipada a una CPI, como hizo el sondeo de token de m04-l2, y una lectura de *esa* cuenta en medio de la CPI es el error de compilación.

Quédate con eso un segundo, porque es una filosofía genuinamente distinta — y mantén dos mecanismos aparte, porque el borrow checker es solo la mitad. El primero es el modelo de cuentas en sí. El `Account<T>` default de V2 es una *vista* zero-copy sobre los bytes de la cuenta, no una copia decodificada una vez arriba de la instrucción, así que no hay una segunda copia que pudiera correrse y quedar desactualizada. La capa borsh que estás a punto de usar acá, `BorshAccount<T>`, sí decodifica en una copia — pero sostiene el borrow de datos de la cuenta por todo el tiempo en que está cargada, y tienes que entregar ese borrow explícitamente antes de que una CPI pueda escribir esos bytes. De cualquier forma, nada cambia debajo de una cuenta tipada cargada sin tu palabra, así que no hay nada que una llamada de re-deserializar pueda re-leer. Por eso el método no existe. El segundo mecanismo es el modelo de borrow, y cubre la ventana que queda: mientras una CPI sostiene un handle a una cuenta, el acceso tipado a *esa* cuenta no va a compilar. v1 te daba una herramienta para evitar una trampa. V2 sacó los lugares donde la trampa podía estar. La clase de bug desapareció, no está guardada.

![En v1 una copia tipada decodificada una vez queda obsoleta a través de una CPI y .reload() la vuelve a leer; en V2 la cuenta tipada cargada sostiene el borrow de datos, así que nada cambia debajo de ella y no hay método de reload al que llamar.](assets/v03-diagram.png)

Así que el arreglo no es "encuentra el nombre V2 de reload". No hay ninguno. El arreglo es estructural: no sostengas datos tipados de una cuenta que esta CPI toma a través de la CPI. Lee los escalares que necesitas (el bump, la clave de estado) hacia locales antes de la transferencia, corre la transferencia, y después toma un borrow tipado fresco después de que complete para actualizar tus contadores — y esa lectura post-CPI ya está viva, que es por lo que no queda nada para que haga un `.reload()`. El error no es un obstáculo. Es la instrucción. Eso es dejar que el compilador maneje.

![El build de V2 tira exactamente un error sobre el withdraw de v1 copiado, ningún método llamado reload, mientras la lectura tipada una línea más arriba compila porque state no es una cuenta que toque esta transferencia.](assets/v04-annotated-code.png)

### Por qué `has_one` todavía compila pero igual lo arreglas

La fila 6 es la otra edición sin marca, y enseña otro reflejo.

`has_one = authority` todavía funciona en V2. Parsea, verifica, la prueba pasa con él puesto. ¿Entonces por qué tocarlo? Porque cuando compilas, recibes esto:

```text
warning: use of deprecated function `__deprecated_has_one`: `has_one` is
         deprecated; on the sibling field, use
         `#[account(address = owner.field)]` instead.
   --> programs/quarter_vault/src/lib.rs:126:9
    |
126 |         has_one = authority,
    |         ^^^^^^^
```

Dos cosas de esa advertencia valen notarse. Primera, nombra el reemplazo exactamente, y te dice dónde ponerlo: en el campo hermano, como `#[account(address = owner.field)]`. Para nuestro vault, eso es `address = state.authority` puesto en la cuenta `authority`, que verifica que la dirección de la authority pasada es igual al campo `authority` guardado en `state`. La misma garantía, escritura nueva. Segunda, y este es el toque de color que quiero que sostengas: ese subrayado no es un accidente. Allá abajo en el parser, `parse.rs` deliberadamente mantiene el span de código de la keyword `has_one` alrededor para que la generación de código pueda emitir una advertencia que apunte derecho de vuelta a esos caracteres exactos. Nadie subraya un token que no planeó deprecar. Las herramientas se construyeron para guiar la migración que crearon. La advertencia es una feature, no ruido.

![La advertencia de deprecación de has_one nombra su propio reemplazo, subraya el token exacto que hay que sacar, y no hace fallar la prueba, lo que la vuelve un ítem de checklist.](assets/v05-annotated-code.png)

Y sobre una RC en movimiento, la sintaxis deprecada es precisamente lo que una versión posterior tiene más probabilidad de sacar. Resolver las deprecaciones hasta cero no es prolijidad. Es cómo mantienes el port compilando contra el tag del mes que viene. La advertencia es un ítem de checklist que el framework te entrega gratis.

Acá está el antes y el después de ese único constraint:

```rust
// v1: has_one lives on the state account (seeds elided).
#[account(mut, has_one = authority)]
pub state: Account<'info, VaultState>,
#[account(mut)]
pub authority: Signer<'info>,
```

```rust
// V2: the equivalence check moves onto the authority account as an address constraint.
#[account(mut)]
pub state: BorshAccount<VaultState>,
#[account(address = state.authority)]
pub authority: Signer,
```

Nota el orden de declaración: `state` antes de `authority`, para que la expresión `address = state.authority` pueda resolver — el vault entregado ya los lista así, y la escritura de V2 es lo que vuelve estructural ese orden. Nota también el `<'info>` soltado en la cuenta tipada. Esas son las filas 1 a 3 viajando junto. Los deltas se agrupan; arreglar uno muchas veces aterriza tres.

## Lab: maneja el vault hasta el verde

Hora de construir. Tienes el mapa de delta y entiendes las dos filas difíciles. Ahora aplícalas. El programa entregado está impreso por entero abajo. Haz el scaffold de un proyecto con el CLI 1.1.2 que ya está en tu máquina (`anchor init quarter_vault` — este es el único uso legítimo que le queda al toolchain viejo en este curso), reemplaza el `programs/quarter_vault/src/lib.rs` generado por el listado, y asegúrate de que el manifiesto del programa lleve la fila de v1 que escribió el scaffold:

```toml
# programs/quarter_vault/Cargo.toml — the row step 1 will flip.
[dependencies]
anchor-lang = "1.1.2"
```

Acá está el vault, entero. Las marcas `// TODO(migrate):` están en los sitios mecánicos; trabaja de arriba para abajo.

```rust
// programs/quarter_vault/src/lib.rs — the handed 0.31/1.0 vault.
// Builds clean on anchor-lang 1.1.2. Does NOT build on the V2 RC.
// The `// TODO(migrate):` markers flag the mechanical sites (rows 1-5).
// Rows 6 and 7 carry no marker on purpose: the compiler finds them for you.

use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

declare_id!("Quart3rVau1t1111111111111111111111111111111");

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub authority: Pubkey, // TODO(migrate): row 1 — Pubkey -> Address
    pub bump: u8,
    pub vault_bump: u8,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
}

#[error_code]
pub enum VaultError {
    #[msg("counter overflow")]
    Overflow,
}

#[program]
pub mod quarter_vault {
    use super::*;

    // TODO(migrate): row 3 — handlers take `&mut Context<T>` in V2.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let authority = ctx.accounts.authority.key(); // TODO(migrate): row 2 — .key() -> .address()
        let state = &mut ctx.accounts.state;
        state.authority = authority;
        state.bump = ctx.bumps.state; // canonical bumps, stored once
        state.vault_bump = ctx.bumps.vault;
        state.total_deposited = 0;
        state.total_withdrawn = 0;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.key(), // TODO(migrate): row 5 — &Address + cpi handles
                Transfer {
                    from: ctx.accounts.depositor.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                },
            ),
            amount,
        )?;
        let state = &mut ctx.accounts.state;
        state.total_deposited = state
            .total_deposited
            .checked_add(amount)
            .ok_or(VaultError::Overflow)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let state_key = ctx.accounts.state.key(); // TODO(migrate): row 2
        let vault_bump = ctx.accounts.state.vault_bump;

        let seeds: &[&[u8]] = &[b"vault", state_key.as_ref(), &[vault_bump]];
        let signer = &[seeds];

        // v1 tail: build the transfer, read state while it is pending, run it, reload.
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.key(), // TODO(migrate): row 5
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.authority.to_account_info(),
            },
            signer,
        );
        let already_out = ctx.accounts.state.total_withdrawn; // legal in v1: the CPI holds AccountInfo clones
        system_program::transfer(cpi_ctx, amount)?;

        ctx.accounts.state.reload()?; // v1 habit: re-deserialize after the CPI

        let state = &mut ctx.accounts.state;
        state.total_withdrawn = already_out
            .checked_add(amount)
            .ok_or(VaultError::Overflow)?;
        Ok(())
    }
}

// TODO(migrate): row 3 — drop the <'info> lifetimes on every struct below.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + VaultState::INIT_SPACE, // TODO(migrate): row 4 — no magic 8 in V2
        seeds = [b"state", authority.key().as_ref()],
        bump
    )]
    pub state: Account<'info, VaultState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(seeds = [b"vault", state.key().as_ref()], bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"state", state.authority.as_ref()], bump = state.bump)]
    pub state: Account<'info, VaultState>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(mut, seeds = [b"vault", state.key().as_ref()], bump = state.vault_bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"state", authority.key().as_ref()],
        bump = state.bump,
        has_one = authority,
    )]
    pub state: Account<'info, VaultState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"vault", state.key().as_ref()], bump = state.vault_bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
```

Ese archivo pasa `cargo check` limpio contra `anchor-lang 1.1.2` — cero errores, cero advertencias de deprecación, porque las deprecaciones son una historia de V2 y sacarlas a la superficie es para lo que existe el paso 6. Y si hiciste From Bitcoin to Solana, este es tu propio vault de su lección an-anchor-vault — la misma división de registro-más-vault, los mismos bumps canónicos guardados, la misma compuerta de authority, con el único campo `balance` crecido en los dos contadores — así que eres bienvenido a portar el que construyeron tus propias manos en vez de este.

**1. Da vuelta el pin, y después levanta el toolchain aislado.** Primero la edición que de verdad selecciona V2 — la jugada de manifiesto que la sección de teoría acaba de volver estructural. Abre `programs/quarter_vault/Cargo.toml` y cambia la fila de `anchor-lang` de su versión 1.x a la RC:

```toml
[dependencies]
anchor-lang = "2.0.0-rc.1"   # was a 1.x row; THIS line is what selects the framework major
```

Sin esta edición, ninguno de los pasos 2 a 7 puede ni empezar a compilar: cada rename de abajo apunta a nombres que no existen en los crates 1.x. Después instala el CLI de la RC exactamente como arriba, y confirma que la etiqueta es veraz:

```bash
CARGO_PROFILE_RELEASE_LTO=off \
cargo install --git https://github.com/otter-sec/anchor \
  --tag v2.0.0-rc.1 anchor-cli --locked --force

anchor --version    # must now report 2.0.0-rc.1, NOT 1.1.2
```

Si eso todavía dice 1.1.2, tu PATH está resolviendo el binario viejo primero. Eso no va a cambiar contra qué framework compila tu grafo de crates — el pin de arriba gobierna eso — pero un toolchain mal etiquetado es cómo el comportamiento de scaffold, de banco de pruebas y de IDL se corre fuera de la línea en la que están tus crates, así que arréglalo antes de escribir una sola línea. Este es el paso 1 por una razón.

**2. Renombra los tipos (filas 1 y 2).** Cambia cada `Pubkey` a `Address` y cada `.key()` a `.address()`. Compila. El compilador va a listar los que te perdiste como errores de tipo y de método. Déjalo. Acá está la struct de estado después de esta pasada:

```rust
use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

declare_id!("Quart3rVau1t1111111111111111111111111111111");

#[account(borsh)]
#[derive(InitSpace)]
pub struct VaultState {
    pub authority: Address,   // was Pubkey
    pub bump: u8,
    pub vault_bump: u8,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
}
```

Nota el `(borsh)` que tuviste que agregar, porque este es el delta que el mapa no lista y con el que el port tropieza de inmediato. `#[account]` pelado en V2 quiere decir zero-copy Pod, y Pod quiere decir `#[repr(C)]` con una aserción de tiempo de compilación de que el tamaño de la struct es igual a la suma de los tamaños de sus campos. Los campos de `VaultState` suman 50 bytes, pero los dos `u64` fuerzan alineación de 8 bytes, así que `repr(C)` redondea la struct a 56 y la aserción se dispara: "account struct has padding bytes." Tienes dos salidas legales. Vuelve a disponer el estado con campos de alineación 1 (el preludio entrega `PodU64`, `PodI128`, `PodBool` para exactamente esto) y mantén el camino zero-copy, o manda esta única cuenta por el camino borsh con `#[account(borsh)]` y el wrapper `BorshAccount<T>`. Un port 1:1 toma la segunda salida, que es también la que está documentada contra `#[derive(InitSpace)]`. Re-arquitecturar para Pod es el proyecto separado del que hablamos al final.

**3. Arranca los lifetimes y arregla las firmas de handler (fila 3).** Suelta el `<'info>` de cada struct de cuentas y de los tipos de sus campos. Cambia cada handler de `ctx: Context<T>` a `ctx: &mut Context<T>`. El handler `initialize` después de esta pasada:

```rust
#[program]
pub mod quarter_vault {
    use super::*;

    pub fn initialize(ctx: &mut Context<Initialize>) -> Result<()> {
        let authority = *ctx.accounts.authority.address();   // .address() returns &Address
        let state = &mut ctx.accounts.state;
        state.authority = authority;
        state.bump = ctx.bumps.state;
        state.vault_bump = ctx.bumps.vault;
        state.total_deposited = 0;
        state.total_withdrawn = 0;
        Ok(())
    }
    // deposit, withdraw below
}
```

Compila otra vez. Espera que cada error de lifetime y de firma se limpie, y espera que las filas de CPI y de constraint de abajo sigan en rojo. Esa lista de errores encogiéndose es tu barra de progreso.

**4. Arregla el cálculo de space (fila 4).** Este todavía compila como `8 + VaultState::INIT_SPACE`, así que el compilador no te va a forzar. El mapa sí. Reemplaza el `8` mágico por el largo real del discriminator:

```rust
#[account(
    init,
    payer = authority,
    space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE,  // was 8 +
    seeds = [b"state", authority.address().as_ref()],
    bump
)]
pub state: BorshAccount<VaultState>,
```

**5. Arregla la CPI de deposit (fila 5).** Dos ediciones viajan juntas acá. `CpiContext::new` ahora toma el programa como `&Address`, y `.address()` ya te entrega uno, así que no hay ningún `&` que agregar. Las cuentas de adentro de `Transfer` tampoco son más `AccountInfo`: V2 las declara como `CpiHandleMut`, que es el handle con borrow rastreado del que habla la fila 7, así que las armas con `.cpi_handle_mut()`. La transferencia de deposit, desde quien deposita hacia el PDA de vault:

```rust
pub fn deposit(ctx: &mut Context<Deposit>, amount: u64) -> Result<()> {
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.address(),   // was .key()
            Transfer {
                from: ctx.accounts.depositor.cpi_handle_mut(),
                to: ctx.accounts.vault.cpi_handle_mut(),
            },
        ),
        amount,
    )?;
    let state = &mut ctx.accounts.state;
    state.total_deposited = state
        .total_deposited
        .checked_add(amount)
        .ok_or(VaultError::Overflow)?;
    Ok(())
}
```

Compila. Las filas mecánicas están listas. Ahora las dos que maneja el compilador.

Una palabra rápida sobre lo que probablemente estás viendo ahora, porque dos fallas son comunes exactamente en este punto y las dos se ven más aterradoras de lo que son. Si el build vomita docenas de errores de tipo de `Address`-contra-`Pubkey` en archivos que nunca tocaste, te perdiste un `.key()` en algún lugar más arriba y el tipo equivocado se está propagando. Arregla el más viejo de la lista del compilador primero, no el más fuerte, porque los errores posteriores normalmente son solo consecuencia de él. Y si vomita errores de nombre no resuelto exactamente en las líneas que ya arreglaste — `Address` desconocido, `.address()` faltando — el culpable es la otra mitad del paso 1: el pin de `anchor-lang` sigue en 1.x, así que los nombres *hacia* los que renombraste no existen en el grafo contra el que estás compilando. El CLI de tu PATH no puede ni causar ni curar ninguno de los dos síntomas; los diagnósticos vienen de los crates que resolvió cargo, exactamente como dijo el paso 1. Ese es el manifiesto mordiéndote, precisamente como se prometió.

**6. Solo: resuelve la advertencia de deprecación (fila 6).** No hay ningún TODO para esto. Compila y lee la advertencia. Subraya `has_one = authority` y nombra el reemplazo. Mueve la verificación a un constraint `address` sobre la cuenta de authority, exactamente como se mostró antes. Recompila hasta que el conteo de deprecaciones sea cero. No te detengas en "la prueba pasa". Detente en "la advertencia desapareció".

**7. Solo: resuelve el método que falta (fila 7).** También sin TODO. El `withdraw` provisto copia el patrón de v1: arma el `CpiContext` firmado en un local, lee `state` mientras ese valor todavía está ahí sentado, corre la transferencia, llama `.reload()`, y después lee otra vez. En V2 exactamente una de esas líneas detiene el build: `.reload()` no existe. La lectura de `state` una línea más arriba está bien acá, porque los handles de esta transferencia están sobre `vault` y `authority` y `state` es una cuenta disjunta que el llamado nunca toca. Borra el reload, y reestructura para que la actualización del contador tome un borrow tipado fresco después de la llamada — y para que nada de lo que los signer seeds toman prestado se salga de alcance antes de que la CPI las use. Acá está la forma a la que apuntas; escríbela antes de leerla, porque el paso 6 y el paso 7 son los dos que se supone que maneje el compilador:

```rust
pub fn withdraw(ctx: &mut Context<Withdraw>, amount: u64) -> Result<()> {
    // Copy the scalars into locals BEFORE the CPI: `seeds` borrows `state_key`,
    // so the local has to outlive the call that uses `signer`.
    let state_key = *ctx.accounts.state.address();
    let vault_bump = ctx.accounts.state.vault_bump;

    let seeds: &[&[u8]] = &[b"vault", state_key.as_ref(), &[vault_bump]];
    let signer = &[seeds];

    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.address(),
            Transfer {
                from: ctx.accounts.vault.cpi_handle_mut(),
                to: ctx.accounts.authority.cpi_handle_mut(),
            },
            signer,
        ),
        amount,
    )?;

    // NO .reload(). Take a FRESH typed borrow only after the CPI has completed.
    let state = &mut ctx.accounts.state;
    state.total_withdrawn = state
        .total_withdrawn
        .checked_add(amount)
        .ok_or(VaultError::Overflow)?;
    Ok(())
}
```

La struct de cuentas `Withdraw` lleva el arreglo de la fila 6:

```rust
#[derive(Accounts)]
pub struct Withdraw {
    #[account(mut, seeds = [b"state", authority.address().as_ref()], bump = state.bump)]
    pub state: BorshAccount<VaultState>,
    #[account(mut, address = state.authority)]     // replaces has_one = authority
    pub authority: Signer,
    #[account(mut, seeds = [b"vault", state.address().as_ref()], bump = state.vault_bump)]
    pub vault: SystemAccount,
    pub system_program: Program<System>,
}
```

**8. Corre la barrera.** La prueba de aceptación es la misma forma que usó cada peldaño de este curso: una prueba de LiteSVM que pasa. LiteSVM es la plantilla de prueba default de Anchor, así que `anchor init` hizo scaffold de un banco de Rust debajo de `tests/`. Alcanza LiteSVM como lo alcanzó el resto de este curso, a través del wrapper del scaffold en vez de un pin directo, para que tu banco no pueda correrse fuera de la versión que espera el toolchain (el `anchor-v2-testing` de rc.1 fija `litesvm 0.11`; el latest de litesvm en crates.io es 0.16.0 al 2026-09-07, cuatro minors adelante, que es exactamente por lo que no lo fijas tú mismo):

```toml
# programs/quarter_vault/Cargo.toml - dev-dependencies
[dev-dependencies]
anchor-v2-testing = { git = "https://github.com/otter-sec/anchor", tag = "v2.0.0-rc.1" }
```

El programa provisto entrega tres helpers de send en `tests/helpers.rs`, uno por instrucción, todos de la misma forma. Acá está `send_initialize` para que veas qué hacen los otros dos con un `amount: u64` agregado en sus datos de instrucción:

```rust
// tests/helpers.rs (provided)
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{
    Keypair, LiteSVM, Message, Signer as _, VersionedMessage, VersionedTransaction,
};

pub fn send_initialize(svm: &mut LiteSVM, authority: &Keypair, state: Address, vault: Address) {
    let ix = Instruction {
        program_id: quarter_vault::ID,
        accounts: quarter_vault::accounts::Initialize {
            state,
            authority: authority.pubkey(),
            vault,
            system_program: System::id(),
        }
        .to_account_metas(None),
        data: quarter_vault::instruction::Initialize {}.data(),
    };
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&authority.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[authority]).unwrap();
    svm.send_transaction(tx).unwrap();
}
```

La prueba en sí maneja el ciclo de vida entero, init y después deposit y después withdraw firmado por PDA, y hace la aserción de que el saldo del vault se movió:

```rust
mod helpers;
use helpers::{send_deposit, send_initialize, send_withdraw};
use anchor_lang::prelude::Address;
use anchor_v2_testing::{svm, Keypair, Signer as _};

#[test]
fn init_deposit_withdraw_roundtrip() {
    let mut svm = svm();
    let program_id = quarter_vault::ID;
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    svm.add_program_from_file(program_id, vault_so).unwrap();

    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 5_000_000_000).unwrap();

    let (state, _) =
        Address::find_program_address(&[b"state", authority.pubkey().as_ref()], &program_id);
    let (vault, _) =
        Address::find_program_address(&[b"vault", state.as_ref()], &program_id);

    // init -> deposit(1 SOL) -> withdraw(0.4 SOL), each through the helper above.
    send_initialize(&mut svm, &authority, state, vault);
    send_deposit(&mut svm, &authority, state, vault, 1_000_000_000);
    send_withdraw(&mut svm, &authority, state, vault, 400_000_000);

    let vault_lamports = svm.get_account(&vault).unwrap().lamports;
    assert_eq!(vault_lamports, 600_000_000, "vault should hold 0.6 SOL after the round-trip");
}
```

Y después la barrera en sí:

```bash
anchor test                              # LiteSVM suite: init, deposit, PDA-signed withdraw
touch programs/quarter_vault/src/lib.rs  # belt and braces: force a genuine recompile before the grep
cargo build 2>&1 | rg "deprecat"         # must print nothing (rg exits 1 on zero matches)
```

Ese `touch` es precaución redundante en vez de estructural, y la distinción vale una frase: el cargo moderno cachea los diagnósticos de un crate y los *repite* en builds calientes, así que el grep agarraría un `has_one` remanente hasta contra un build que no recompiló nada. Hacerle touch al archivo solo vuelve la línea que grepeas demostrablemente la salida fresca de este build en vez de una repetición — un seguro barato cuando estás a punto de reportar un número como final.

Y después corre los mismos dos comandos adentro del contenedor de verify, para que el resultado que reportas lo haya producido la RC fijada y nunca lo que esté en tu PATH:

```bash
docker build -t v2-port verify/ && docker run --rm v2-port
```

Prueba verde, cero advertencias de deprecación, sobre el toolchain de la RC, reproducido en el contenedor. Ese es el port. Esa es la demostración.

![Un loop de seis pasos: fija el toolchain de la RC, aplica los deltas mecánicos marcados, y después compila y arregla lo que imprima el compilador hasta que anchor test esté verde.](assets/v06-timeline.png)

## Challenge

El lab te entregó el vault. El challenge saca las rueditas.

El **segundo** programa 0.31/1.0 lo armas tú mismo, en `challenge/`, antes de portarlo — diez minutos de la memoria muscular de 1.x que acabas de retirar, y te compra una base de código sin ninguna marca de TODO: una instrucción `sweep` de dos vaults que mueve lamports desde un PDA de vault de origen hacia un PDA de vault de destino en una llamada, y actualiza un contador compartido después de la transferencia, escrita en idioma v1 completo — `has_one` gateando los dos estados de vault, y el contador leído otra vez a través de un `.reload()` después de la CPI (el vault del lab es tu plantilla de plomería; confirma que el original compila limpio en la línea 1.x antes de tocarlo). Sus operadores también lo usan para ajustar el contador de un vault único haciendo sweep de ese vault hacia sí mismo, así que escribe la prueba de LiteSVM para hacer exactamente eso — una cuenta de verdad llega en los dos slots mutables. Después pórtalo a V2 y haz que esa prueba pase con cero advertencias de deprecación.

Tres cosas lo vuelven más difícil que el lab, y cada una mapea a algo que ahora sabes:

1. Usa `has_one` en dos lugares. Resuelve los dos solo desde las advertencias de deprecación.
2. Su handler lee un contador, hace la CPI de transferencia, y después lee el contador otra vez con un `.reload()` en el medio. Mata el reload y reestructura las lecturas alrededor de la llamada. El error de método que falta es tu mapa.
3. El self-sweep entrega **una cuenta a dos slots mutables** (origen y destino). V2 rechaza cuentas mutables duplicadas por defecto, así que esa prueba falla la validación con `ConstraintDuplicateMutableAccount` antes de que corra tu handler. Aplica `unsafe(dup)` a los dos campos de vault, y, porque el nombre dice `unsafe`, escribe una frase en un comentario justificando el aliasing: el handler tiene que computar el movimiento una vez y aplicar una única actualización verificada, para que nunca sostenga dos referencias mutables en conflicto a la única cuenta. Si te encuentras recurriendo a `unsafe(dup)` sobre el contador también, detente: esa es una cuenta en un slot, y el opt-out estaría escondiendo otro bug.

![Una tabla de tres filas que empareja cada obstáculo del challenge con la advertencia, el error de método que falta, o la falla de validación que lo encuentra, más una precaución contra aplicar de más el opt-out de cuenta duplicada.](assets/v07-table.png)

Acéptalo cuando `anchor test` pase sobre el toolchain de la RC y `cargo build` emita cero advertencias de deprecación. Sin pistas más allá de tus dos mapas y el compilador. Ese es el punto.

## Antes de seguir adelante

Nota lo que le acaba de pasar a la forma de esta lección. Los deltas iniciales venían con marcas de TODO en el código y código trabajado que podías leer derecho de la página. Los dos últimos pasos del lab no tenían ninguna marca en el código: la advertencia y el error de método que falta los ubicaban por ti, y la respuesta impresa estaba ahí para que te verificaras después de escribir la tuya. El challenge suelta hasta eso. Ese repliegue fue deliberado. Coincide con dónde estás: al principio de la pista de migración necesitabas el delta nombrado y ubicado por ti; a esta altura las herramientas los nombran y los ubican mejor de lo que podría un comentario. Si el paso 7 se sintió menos como seguir instrucciones y más como leerle la mente al compilador, esa es la habilidad hacia la que esta pista entera estaba construyendo. Eso vale más que cualquier rename de constraint aislado.

Y sé honesto contigo mismo sobre lo que este port es y no es. Una migración manejada por checklist es rápida y mecánica, y convierte un programa uno a uno. Tu vault V2 mantiene su forma de v1. No está re-arquitecturado para las fortalezas de V2, no es el layout Pod más flaco que podría ser, es el diseño viejo que ahora compila sobre el framework nuevo. Ese es un trade-off de verdad, no una falla: el 1:1 es exactamente lo que quieres cuando la meta es "hacer que esto compile con seguridad", y re-arquitecturar es un proyecto separado que asumes después, deliberadamente, no contrabandeado adentro de una migración. La otra mitad del trade-off es el suelo moviéndose debajo de ti. Esto compila hoy contra `2.0.0-rc.1` sobre una rama alpha cuya propia documentación advierte que las API pueden romperse entre commits. Una RC posterior puede renombrar `address` o cambiar cómo se escribe `unsafe(dup)`. No pelees con eso. Vuelves a correr el checklist.

El port compila y la prueba está verde. Llevaste una base de código v1 de verdad todo el camino hasta V2, a mano, dejando que el compilador manejara la última milla. Queda una pregunta, y no es técnica: dada la tensión de RC-y-alpha, la fecha estable no comprometida, y el estado no auditado, ¿*deberías* de verdad moverte a V2 hoy? Esa es una decisión de juicio, no un error de compilación, y la próxima lección, la conclusión del curso, la responde honestamente.
