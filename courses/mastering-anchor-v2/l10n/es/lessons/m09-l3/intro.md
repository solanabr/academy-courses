# Capstone: pon a correr el salón de arcade entero

Acabas de diffear tu vault nativo contra la expansión de macro de V2, línea por línea, y te diste la mirada de un solo compás a asm-v2. Ahora puedes predecir qué escribe el derive y por qué, lo que quiere decir que el framework dejó de ser una caja negra la última vez que corriste `cargo expand`. Bien. Aférrate a eso, porque esta lección lo gasta.

Acá está el dolor, dicho sin vueltas. Cada peldaño que construiste está solo. El cabinet-counter cuenta. El quarter-vault guarda. El prize-escrow liquida. El swap de token-a-ticket cotiza. Cuatro programas, cuatro suites de prueba en verde, uno de ellos — el swap — ya vivo en devnet, y ninguno de ellos sabe que los otros existen. Un arcade no son cuatro máquinas en cuatro cuartos. Es un salón: una jugada sube un counter, el counter mete un crédito en un vault, una victoria libera un premio de un escrow, y una pila de tickets se canjea por algo en el counter. Todavía nadie cableó el salón. Ese es el capstone, y es casi enteramente tuyo.

Un asunto de orden primero, porque todo lo de abajo lo da por hecho. R1 viene viviendo por su cuenta desde m02-l1: `anchor init cabinet-counter` lo hizo un workspace de uno, mientras que R2, R3 y R4 crecieron todos adentro del workspace `quarter-vault` que empezaste en m03-l1. El salón construye, prueba y despliega como un solo workspace — un `Anchor.toml`, un `target/deploy` del que carga cada banco de pruebas, un directorio `idls/` — así que copia R1 adentro antes de generar ningún scaffold. El workspace original se queda donde está — esto es una copia, no un traslado, y puedes borrar el árbol viejo una vez que el registry compile. Desde la raíz de ese workspace de arcade:

```bash
cp -R ../cabinet-counter/programs/cabinet-counter programs/cabinet-counter
cp ../cabinet-counter/target/deploy/cabinet_counter-keypair.json target/deploy/
```

Después regístralo como lo habría hecho `anchor new`: agrégalo a `members` en el `Cargo.toml` del workspace si esa lista nombra los programas uno por uno en vez de usar el glob `programs/*`, y agrega su fila bajo `[programs.localnet]` en `Anchor.toml`. Deja el `declare_id!` con el que llega el crate — ese id coincide con el keypair con el que desplegaste R1 en m02-l1, y editarlo a mano deja huérfano el deploy. El segundo `cp` de arriba se llevó su keypair, así que `anchor keys sync` va a confirmar que el par sigue coincidiendo. Una última revisión ya que estás adentro: su fila `solana-address` tiene que leer el rango de m02-l1, porque R1 es miembro de este workspace ahora y un solo miembro que sostenga una igualdad hace fallar el resolve entero.

Ahora vamos a hacer que el salón exista antes que la teoría. Genera el scaffold del último programa y apúntalo a los cuatro que ya entregaste. Nada de instalar nada nuevo para esta parte:

```bash
anchor new floor-registry   # adds programs/floor-registry to the workspace
```

Después cablea el registry a los cuatro peldaños de la misma forma en que el escrow alcanzó el vault en m04-l3 — a través de sus interfaces, cuatro veces. Primero, cosecha el IDL de cada peldaño hacia el directorio `idls/` de la raíz del workspace. Los peldaños tienen que estar nombrados *antes* de que el registry compile siquiera, porque `declare_program!` lee el JSON en tiempo de expansión de macro:

```bash
mkdir -p idls
for rung in cabinet-counter quarter-vault quarter-prize token-ticket-swap; do
  ( cd programs/$rung && anchor idl build -o ../../idls/$(echo $rung | tr '-' '_').json )
done
```

(Tu `idls/quarter_vault.json` ya existe de la última re-cosecha del módulo 5; el loop simplemente refresca los cuatro a lo que digan los peldaños hoy, que es el único estado de IDL contra el que vale la pena construir.)

El `Cargo.toml` del registry entonces no lleva **ninguna fila de peldaño** — este es el diff contra cada workspace multi-crate que viste antes, y es una eliminación:

```toml
[dependencies]
# The rc.1 crates landed on crates.io (2026-08-12), and that is where the LIBRARY comes
# from course-wide: a published version is immutable. The CLI is the git build (see m01-l2).
anchor-lang = "2.0.0-rc.1"
# The pins from m01-l2 — every program crate in this course carries them (issue #4937's class).
wincode = { version = "0.5", features = ["derive"] }
# The arcade-workspace row, identical to the one every rung has carried since m02-l1.
# Step 4 puts Mollusk in this crate, and Mollusk's SVM stack reaches solana-address
# ^2.6.1; the ceiling is what the pin was always for — 2.6.1 is still wincode 0.5, and
# 2.7.0 is the version that moved. The registry and the four rungs share one
# workspace and one lock, so all five members must read this row, not just this one.
solana-address = ">=2.6.1, <2.7"
# No `{ path = "../<rung>", features = ["cpi"] }` rows — the rungs arrive as IDLs.
```

y la parte de arriba de `programs/floor-registry/src/lib.rs` nombra los cuatro peldaños (R3 es el crate `quarter-prize`, el scaffold de m04-l3 — "prize-escrow" es el papel que juega en el salón, no el nombre con el que lo conoce cargo):

```rust
use anchor_lang::prelude::*;

declare_program!(cabinet_counter);
declare_program!(quarter_vault);
declare_program!(quarter_prize);
declare_program!(token_ticket_swap);
```

¿Por qué no cuatro filas de path-dep con el hook `features = ["cpi"]` del scaffold, como lo hace la mitad de los tutoriales de Anchor que leíste? Porque en rc.1 ese camino termina físicamente en un peldaño. `cpi` prende `no-entrypoint`, y bajo `no-entrypoint` cada programa consumido exporta su función de dispatch como un símbolo sin manglear — así que en el momento en que un programa referencia los módulos `cpi` de *dos* peldaños así, el link de SBF muere con `duplicate symbol: __anchor_dispatch`, y un registry que compone cuatro peldaños no linkea nunca. El mismo hook tiene una segunda falla, más callada: un `cargo build-sbf` desde la raíz del workspace unifica el feature `no-entrypoint` sobre el peldaño consumido mismo y emite un `.so` con *ningún símbolo de entrypoint* — compila limpio, se sienta en `target/deploy/` con pinta de desplegable, y el loader lo rechaza. `declare_program!` no es una solución alterna adoptada a regañadientes; es el mecanismo entre programas propio de V2, genera la superficie de CPI en modo interfaz donde no existe ningún símbolo de dispatch que colisione, y el curso retiró las filas del feature `cpi` el día en que el salón de cuatro peldaños se volvió la meta.

Corre `anchor build`. Va a compilar un registry que todavía no hace nada, pero los cuatro módulos generados ya están en scope, y el compilador va a empezar a decirte exactamente qué handles espera cada peldaño. Ese loop de feedback es el lab entero.

**Resumen.** El floor-registry es un solo programa de Anchor V2 que compone los cuatro peldaños por CPI: incrementa el counter de un cabinet (R1), rutea créditos a través del quarter-vault (R2), liquida premios a través del prize-escrow (R3), y cotiza a través del swap (R4). Vas a cablear el borde de R1 como un paso trabajado, después construyes el resto solo, y después llevas la cosa entera por el ciclo de vida de producción completo que este curso viene enseñando: una suite de LiteSVM más Mollusk, un caso de fuzz en verde, una pasada del checklist de seguridad, un profile de CU con una optimización medida, una corrida de localnet en Surfpool con el salón de cinco programas junto, un deploy en devnet, y un verify-desde-el-repo local que demuestra que tu build reproduce el bytecode on-chain. Cuando `anchor test` imprima `floor-registry ... passing` contra el salón de localnet y tu verify coincida, terminaste.

El repliegue de la ayuda, dicho en voz alta para que sepas qué es tuyo. La CPI del counter de R1 está trabajada para ti entera, porque la acumulación en este curso siempre se demuestra, nunca se entrega terminada. La gramática de CPI que reutilizas para los otros tres peldaños está en la página. Todo lo que viene después (las instrucciones del vault, del escrow y del swap, y después cada paso del ciclo de vida) es el capstone. Es solo. Te voy a mostrar la forma de cada jugada y el comando que la demuestra, y la vas a correr contra tu propio código.

## El salón como un solo programa

Empieza por la imagen, porque el registry es más fácil de sostener como un hub. Casi no tiene estado propio. Lo que sí tiene son las decisiones sobre *cuándo* llamar a cada peldaño y *en qué orden*, y delega cada cambio de estado real al programa construido para eso. Ese es el argumento entero a favor de la composición: el registry es una cosa chica sobre la que puedes razonar, atornillada a cuatro cosas probadas en las que ya confías.

![El floor-registry se sienta en el centro con una flecha de CPI hacia cada uno de los cuatro peldaños, más una segunda flecha que muestra la liquidación del premio como una llamada de dos saltos.](assets/v01-diagram.png)

### Por qué un programa y cuatro CPI, no un programa grande

Detente en la decisión de diseño antes de la mecánica, porque es la decisión a favor de la que argumenta el capstone entero. Podrías escribir un solo programa monolítico que cuente jugadas, guarde créditos, liquide premios y cotice swaps, todo en un solo crate. Se desplegaría como un solo `.so`, no necesitaría ninguna CPI, y correría a una sola profundidad de invocación. Para un proyecto de fin de semana es menos código. ¿Entonces por qué el registry es un hub de cuatro llamadas en vez de eso?

La respuesta es la deriva, y es la misma razón por la que el escrow no reimplementó la custodia. Cada peldaño es una cosa acotada que ya probaste, endureciste, fuzzeaste y perfilaste. Mete su lógica adentro de un monolito y ahora eres dueño de una segunda copia de esa lógica, una que no comparte nada con el peldaño desplegado y que empieza a divergir el día que arreglas un bug en una y te olvidas de la otra. Dos copias de la matemática de custodia son dos bugs de custodia esperando a desincronizarse. Componer sobre los peldaños quiere decir que el registry es dueño de exactamente una responsabilidad, la decisión sobre qué llamar y cuándo, y cada cambio de estado real se queda detrás de la interfaz del programa construido para eso. Cuando parchas el vault, el salón recibe el parche gratis, porque el salón nunca tuvo su propio vault. Eso no es una preferencia de estilo. Es la diferencia entre una base de código que se vuelve más segura a medida que crece y una que acumula copias del mismo error.

Hay una segunda razón que solo aparece en la costura: una interfaz a la que le haces CPI es un contrato que puedes verificar de forma independiente. Las pruebas del vault demuestran el vault. Las pruebas del registry demuestran que el registry llama al vault correctamente. Ninguna tiene que volver a demostrar la otra, y un auditor puede leer cada una por separado. Un monolito colapsa las dos en un solo bloque donde la lógica de conteo y la lógica de custodia pueden meterse calladas en el estado de la otra, y ahora nada se puede demostrar solo. Cosas chicas atornilladas a cosas probadas, cada una verificable por sí sola, es como mantienes auditable un programa que crece. La CPI es el tornillo.

### La composición es una CPI, y la gramática es la que ya escribiste

No hay nada nuevo que aprender sobre cómo un programa llama a otro. Lo hiciste en el escrow. El registry hace lo mismo cuatro veces. La gramática de CPI de V2 tiene tres partes y ya usaste las tres. Primero, el callee expone una struct de accounts generada, un `CpiHandle` por cuenta, que llenas con `.cpi_handle_mut()` para las cuentas que el callee va a escribir y `.cpi_handle()` para el resto. Segundo, `CpiContext::new` toma el id de programa del callee a través de `.address()` sobre su cuenta `Program`, que en V2 entrega un `&Address`, no un clon de `AccountInfo`. Tercero, el wrapper generado empaqueta tus argumentos e invoca.

Vale la pena detenerse en el contraste, porque es la diferencia entre la línea que borraste y la línea que dejaste.

![Una tabla de tres columnas que compara cada pieza de una llamada de CPI entre las líneas de Anchor, subrayando que las lecturas obsoletas-después-de-CPI se volvieron un error de compilación en V2.](assets/v02-comparison.png)

El único hábito que pasa derecho al capstone: lee cualquier estado que necesites *antes* de abrir un handle. Una vez que `.cpi_handle_mut()` toma prestada una cuenta, no puedes tocar esa cuenta a través de su vista tipada hasta que la llamada consuma el `CpiContext` y el handle se suelte. Esa ya no es una regla que sigues tú. Es una regla que el compilador sigue por ti, y es exactamente por eso que el swap del vault leyó sus reservas por adelantado, una sola vez, antes de cotizar. Sigue haciendo eso y el borrow checker va a seguir atrapando tus lecturas obsoletas antes de que un validador las vea.

### El costo de la confianza, y el techo de profundidad

La composición es poderosa, pero nombra el canje con honestidad, porque es el punto de la lección. Cada peldaño al que le haces CPI es un peldaño en el que confías. El registry confía en que el counter suba el cabinet correcto, en que el vault mueva los lamports correctos, en que el escrow libere solo sobre una condición verdadera. Esa confianza es código real que puedes leer y pruebas que ya corriste, que es muchísimo mejor que confiar en el programa de un extraño. Sigue siendo confianza, y sigue costando.

Dos costos son concretos. El primero es la profundidad de invocación. El runtime de Solana le pone un tope a qué tan hondo puede anidarse una cadena de CPI: la altura máxima de la pila de invocación es **5**, que es tu instrucción de nivel superior más cuatro CPI anidadas. Hay un aumento especificado, SIMD-0268, "Raise CPI Nesting Limit", estado Accepted, que subiría el anidamiento de 4 a 8, pero su feature gate, `6TkHkRmP7JZy1fdM6fg5uXn76wChQBWGokHBJzrLB3mj`, no tenía cuenta en mainnet cuando se escribió esta lección (sondeado 2026-08-22), así que 5 es el número en vigor. Vuelve a sondear el feature gate en tiempo de build en vez de confiar en esta frase para siempre; un gate pendiente es exactamente la clase de hecho que se da vuelta entre que un curso se escribe y un curso se lee. Importa acá porque `settle_prize` es una llamada de tres saltos: el registry le hace CPI al escrow, el escrow le hace CPI al vault, y el vault le hace CPI al token program. Cuéntalo: tu instrucción de nivel superior más tres llamadas anidadas es altura de pila 4, así que te queda exactamente una llamada anidada de margen. Esa es la clase de número que no podías computar para nada cuando cada programa vivía solo.

![Una pila vertical que muestra settle_prize anidándose a través de floor-registry, prize-escrow, quarter-vault y el token program, llegando a altura de pila 4 sobre el máximo de 5.](assets/v03-diagram.png)

El segundo costo es la disciplina de borrow en sí, pero ese es un regalo disfrazado de costo: que el compilador te obligue a secuenciar tus lecturas es la razón por la que una liquidación de dos saltos no paga en silencio contra un saldo obsoleto. Pagas con un poco de rigidez por adelantado y eso te compra una clase de incidente de las 2am que nunca vas a tener.

Una propiedad juega a tu favor en todos los peldaños, y vale la pena nombrarla porque cambia cómo razonas sobre las fallas. Una transacción es atómica. Si cualquier CPI de la cadena devuelve un error, la transacción entera revierte, y cada cambio de estado arriba de ella se revierte con ella. Así que `settle_prize` no puede liquidar a medias: si la verificación de condición del escrow falla, la CPI de redeem da error, y el depósito o la subida del counter que vinieron antes en la misma transacción también se deshacen. Eso es una red de seguridad de verdad, y también es una trampa si te apoyas en ella como tu única guarda. La atomicidad te salva cuando una llamada *da error*. No hace nada cuando una llamada *tiene éxito contra una premisa falsa*, que es exactamente por qué el escrow verifica la condición antes de construir la CPI de release en vez de pagar primero y confiar en el revert. El orden sigue siendo tu guarda. La atomicidad es el respaldo, no el plan.

## Lab: cablea el salón y córrelo de punta a punta

Este es el lab del capstone. El borde de R1 está trabajado. El resto es tuyo. Voy a mantener los pasos del ciclo de vida cortos, un comando y la salida que lo demuestra, porque a esta altura ya corriste cada una de estas herramientas al menos una vez y el capstone se trata de ensamblarlas, no de volver a enseñarlas.

> Nota de frescura: esto está escrito contra la release candidate de Anchor V2 en la línea 2.x (el árbol de docs publicado bajo `v2`), `2.0.0-rc.1` a fecha de 2026-08-22. Instala el toolchain desde el canal de git documentado (Paso 0, `avm` no puede traer la RC). El `anchor-cli 1.1.2` default de la máquina es la línea V1 y no va a compilar la gramática de `CpiHandle` ni de `&Address` de abajo. Los pins de versión en este lab llevan la fecha en que se revisaron; vuelve a verificar antes de compilar.

**Paso 0. Fija el toolchain.** Una línea, y la trampa detrás de ella es una que ya te sabes de memoria — `avm install` todavía da 404 con la RC, nunca se cortó ningún GitHub Release para el tag — así que si el check falla, vuelve a correr la instalación por git de m01-l2 (`--tag v2.0.0-rc.1`, `--locked`):

```bash
anchor --version           # expect: anchor-cli 2.0.0-rc.1
```

Una nota de línea de versión para que nadie tropiece: este curso fija el CLI de Solana `3.1.10` como toolchain de build local y de CI, que es lo que usa el contenedor de build verificable. Ese pin es una decisión de reproducibilidad, no una afirmación sobre la red actual. El release estable actual de Agave es una cosa aparte, que se mueve más rápido (v4.2.1 a fecha de 2026-08-22; revisa `agave-install info` o `solana --version` en tiempo de build). Nunca leas el pin `3.1.10` como "la versión actual de Solana".

**Paso 1. Las cuatro interfaces ya están adentro.** Cosechaste los IDL y escribiste las cuatro líneas de `declare_program!` en la apertura. Confirma que `anchor build` todavía compila el registry vacío con los cuatro módulos generados resueltos. `` error: `idls` directory not found `` quiere decir que la cosecha nunca corrió — los peldaños tienen que estar nombrados antes de que el registry compile. Un *item* faltante adentro de un módulo (una función o campo que el compilador no encuentra) quiere decir un JSON obsoleto: vuelve a correr el loop de cosecha, porque `declare_program!` compila contra el archivo, no contra la fuente.

**Paso 2. Cablea R1, el incremento del counter (trabajado para ti).** Una jugada en un cabinet es una CPI: el registry llama al `post_score` del counter — `increment` como m02-l1 lo escribió primero, renombrado en m02-l2 cuando el cabinet creció un board. Acá está entero. Lee cada línea, porque esta es la plantilla que vas a copiar tres veces.

```rust
use anchor_lang::prelude::*;

declare_program!(cabinet_counter);
declare_program!(quarter_vault);
declare_program!(quarter_prize);
declare_program!(token_ticket_swap);

// Everything a caller needs comes out of the generated module: the `cpi`
// builders, the rung's account types (`Cabinet`, straight from the IDL), and —
// unlike source-level consumption, where the caller hand-writes it — the
// Program<T> marker, at cabinet_counter::program::CabinetCounter, with its
// IDL_ADDRESS already filled from the JSON.
use cabinet_counter::cpi as counter_cpi;
use cabinet_counter::program::CabinetCounter;
use cabinet_counter::Cabinet;

declare_id!("F1oorReg1stry111111111111111111111111111111");

#[program]
pub mod floor_registry {
    use super::*;

    // A play bumps the cabinet's counter by CPI-ing into R1.
    // This is the accretion edge, wired as a worked step, not handed to you finished.
    pub fn record_play(ctx: &mut Context<RecordPlay>, score: u64) -> Result<()> {
        // Build the callee's accounts struct from HANDLES, not AccountInfos.
        // cpi_handle_mut() takes a live borrow of `cabinet` for the callee; while it is
        // held you cannot also touch `cabinet` through its typed view. That borrow IS the
        // reload discipline, enforced by the compiler instead of your memory.
        // Named after the INSTRUCTION, and m02-l2 renamed it: `increment` became
        // `post_score` when R1 grew a leaderboard, so the generated struct is
        // `accounts::PostScore`, not `accounts::Increment`.
        let cpi_accounts = counter_cpi::accounts::PostScore {
            cabinet: ctx.accounts.cabinet.cpi_handle_mut(),
            player: ctx.accounts.player.cpi_handle(),
        };

        // .address() hands the callee's program id as &Address (V2), not an AccountInfo.
        let cpi_ctx = CpiContext::new(
            ctx.accounts.cabinet_counter_program.address(),
            cpi_accounts,
        );

        // The generated wrapper packs the score and invokes R1.post_score, whose
        // argument m02-l2 named `points`.
        counter_cpi::post_score(cpi_ctx, score)?;
        Ok(())
    }
}

// V2 wrappers carry no <'info> lifetime, and handlers take &mut Context<T>.
// If you catch yourself typing Account<'info, Cabinet>, you are on the V1 line.
#[derive(Accounts)]
pub struct RecordPlay {
    // Owner-checked to the cabinet-counter program; R1's own increment context
    // re-validates the [b"cabinet", player] seeds when the CPI lands.
    #[account(mut)]
    pub cabinet: Account<Cabinet>,
    pub player: Signer,
    pub cabinet_counter_program: Program<CabinetCounter>,
}
```

El jugador firma la transacción externa, y ese privilegio de firmante se extiende hacia abajo a través de la CPI, así que el counter ve un `player` firmado sin que el registry firme nada por su cuenta. Nada de esto es nuevo. Es el depósito `reserve` del escrow con otros nombres.

Resultado esperado: `anchor build` compila el registry con una instrucción y sin advertencias sobre rutas `counter_cpi` sin resolver. Un error "no method named `cpi_handle_mut`" quiere decir que estás en el CLI V1 default de la máquina, no en la RC del Paso 0; un `cabinet_counter::cpi` sin resolver quiere decir que falta la línea `declare_program!` o su `idls/cabinet_counter.json`. Una regla de nombres para llevar en el bolsillo para los bordes solo: las structs de accounts de CPI generadas se nombran según la **instrucción** (`accounts::PostScore` para `post_score`), no según como sea que el callee llamó a su propio tipo de contexto — el IDL lleva nombres de instrucción, y en los peldaños los dos coinciden por casualidad. Que es también por qué un rename que hiciste hace dos módulos te alcanza acá: un `unresolved import counter_cpi::accounts::Increment` es un *nombre* obsoleto, no una cosecha obsoleta, y volver a correr la cosecha no lo va a arreglar.

![Una tarjeta de código anotada que aísla las tres partes reutilizables de una CPI de V2, con la regla de leer el estado tipado antes de abrir un handle.](assets/v04-annotated-code.png)

**Paso 3. Cablea R2, R4 y R3 (solo).** Estos son el capstone. Cada uno es la misma gramática de tres partes apuntada a un peldaño distinto. Constrúyelos de a uno y deja que `anchor build` te diga qué handles faltan.

- `route_credit` llama a `quarter_vault::cpi::deposit(cpi_ctx, amount)`. Ese es R2 tal como queda después del módulo 5: el vault actualizado a SPL, cuyo `deposit` mueve tokens con `transfer_checked`, no la versión de lamports del módulo 4. Así que las cuentas que llenas son el estado del vault, el depositante, el mint y las dos cuentas de token. El jugador es el depositante y firma, así que esto es un `CpiContext::new` simple, sin signer seeds. La misma forma que el `reserve` del escrow, un peldaño más afuera.
- `quote_swap` llama a `token_ticket_swap::cpi::swap_arcade_for_tickets(cpi_ctx, amount_in, min_out)`. Lee las reservas que necesites antes de abrir cualquier handle, después pasa las cuentas del swap. La guarda de slippage ya vive adentro de R4; el registry solo rutea.
- `settle_prize` llama a `quarter_prize::cpi::redeem(cpi_ctx, final_score)` — R3, el prize-escrow, cuyo crate cargo conoce como `quarter-prize`. Esta es la llamada más profunda del salón, así que cuida la profundidad: el escrow mismo le va a hacer CPI al vault para liberar. El registry no firma por el PDA del escrow. El escrow firma por sí mismo, como siempre lo hizo.

Dos de estos tienen una arruga que vale la pena marcar antes de que te la topes. `quote_swap` lee las reservas del pool para dimensionar el canje, y esa lectura tiene que pasar antes de que abras cualquier handle desde esas mismas cuentas de reserva, o el borrow checker te frena en seco. Esta es la disciplina de leer-antes-del-handle del swap mismo, ahora una capa más afuera: el registry lee, después rutea. Y `settle_prize` es el camino más profundo del salón, así que ten el diagrama de profundidad en mente. Cuéntalo con precisión, porque el número es el punto: tu instrucción de nivel superior es altura 1, la llamada del registry hacia el escrow es 2, la llamada del escrow hacia el vault es 3, y el `transfer_checked` del vault hacia el token program es 4. Cuatro de las cinco que el runtime te deja. Queda una llamada anidada de margen. Agrega un peldaño entre el registry y el escrow y ya la gastaste.

Si una llamada se niega a compilar con un error de borrow, casi siempre es una lectura tipada sentada arriba de la línea que suelta un handle. Mueve la lectura hacia arriba, antes de que el handle se abra, y prueba de nuevo. Ese error es el compilador haciendo tu disciplina de reload por ti.

**Paso 4. La suite de unidad: LiteSVM más Mollusk.** Cada peldaño ya tiene pruebas. El registry necesita las suyas, ejercitando cada borde por separado contra un runtime en proceso. LiteSVM corre tu programa compilado entero en un validador liviano en memoria; Mollusk maneja una sola instrucción y reporta la CU que quemó. Agrégalos como dev-dependencies, y fíjate que los dos llegan por caminos distintos:

```bash
# LiteSVM comes through the V2 harness, never by name. anchor-v2-testing owns the
# litesvm version (0.11.0 at tag v2.0.0-rc.1; the anchor-next head has already moved
# it to 0.13.1), so pinning the tag pins the SVM. A bare `cargo add --dev litesvm`
# resolves the crates.io latest against your rc.1 program: two SVM majors, one graph.
cargo add anchor-v2-testing --dev \
  --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1

# Mollusk is a separate stack and carries its own solana pins, exactly as in m06-l1:
# 0.15 builds on the agave 4.x SVM crates, so the measurement tests need solana-sdk 4
# for their Pubkey/Account/Instruction types. Those rows are Mollusk's, not LiteSVM's.
cargo add mollusk-svm@0.15.1 --dev
# SPL Token's cache entry + account row for Mollusk, exactly as m06-l1 used it.
cargo add mollusk-svm-programs-token@0.15.1 --dev
cargo add solana-sdk@4 --dev
# And the two rows that keep Mollusk's graph on wincode 0.5, straight out of m06-l1.
# Quote them: the shell would read < and > as redirects.
cargo add 'solana-short-vec@>=3.2.2, <3.3' --dev
cargo add 'solana-signature@>=3.4.1, <3.5' --dev

cargo build-sbf                      # both harnesses load the .so; build before you measure
export SBF_OUT_DIR=$PWD/target/deploy   # Mollusk reads this, not target/deploy (m06-l1)
cargo test -p floor-registry         # runs BOTH suites
```

> Nota de pin, y es la razón por la que la fila `solana-address` al inicio de esta lección — y en los cuatro peldaños que arrastra — lee `">=2.6.1, <2.7"` en vez de un `=2.6.0` exacto. El alcance es el workspace, no este crate. `cargo` resuelve un solo `solana-address` para todos los miembros a la vez, así que un solo peldaño que todavía sostenga `=2.6.0` hace fallar el resolve entero con `all possible versions conflict`, y el registry ni siquiera llega a compilar. El stack de SVM de Mollusk alcanza `solana-address ^2.6.1`; un pin exacto en cualquier parte del workspace lo rechaza. Las dos filas de rango de abajo son el mismo peligro un nivel más abajo, y se comportan distinto: son dev-dependencies de este crate, así que le dan forma al lock sin que cada hermano tenga que declararlas — `solana-short-vec 3.3.0` y `solana-signature 3.5.0` se movieron a `wincode 0.6` sin dejar de satisfacer `solana-message`, así que sin ellas el resolve funciona y el *build* muere. Las tres filas dicen una sola cosa: sostén este grafo en `wincode 0.5`, la línea que quiere rc.1. Ninguna sobrevive a que V2 cruce a 0.6, y ninguna va antes de eso.

Escribe dos clases de prueba en ese crate, porque las dos herramientas responden preguntas distintas y el Paso 7 necesita la segunda. Las pruebas de LiteSVM son la suite de comportamiento: una por borde, `record_play`, `route_credit`, `settle_prize`, `quote_swap`, cada una afirmando que la CPI aterrizó y que el estado del callee se movió; sus imports montan sobre `anchor_lang` y `anchor_v2_testing` y no alcanzan más allá de ninguno de los dos, la misma forma que usó cada prueba de LiteSVM de este curso. Las pruebas de Mollusk son la suite de medición, la misma forma que construiste en el módulo 6: una instrucción, una fixture, `process_instruction`, y un `println!` de `compute_units_consumed`, importando `Account`, `Instruction` y `Pubkey` desde `solana_sdk`. Una arruga específica del capstone que la forma del módulo 6 no tenía: un SVM minificado corre solo los programas que registras, y `settle_prize` invoca una cadena entera de ellos. Registra cada peldaño local en el banco de pruebas — `mollusk.add_program(&quarter_prize::ID, "quarter_prize")`, y lo mismo para el vault — que carga cada `.so` por nombre desde el `SBF_OUT_DIR` que exportaste arriba; los ids salen de los módulos generados del propio registry (`use floor_registry::quarter_prize;` en la prueba — no hay ningún extern crate `quarter_prize` desde el que importar). Registra SPL Token a través de su crate compañero, `mollusk_svm_programs_token::token::add_program(&mut mollusk)`. Después dale también su fila de cuenta a cada programa al que le haces CPI: `mollusk_svm::program::create_program_account_loader_v3(&quarter_prize::ID)` construye la cuenta ejecutable propiedad del loader que el runtime exige, y la fila del token program es `token::keyed_account()`. Las dos mitades fallan de dos formas distintas, las dos vale la pena reconocerlas de una: una *entrada de caché* faltante deja correr tu instrucción externa y después mata la CPI con `Unsupported program id`, mientras que una *fila de cuenta* faltante no llega a tu programa para nada — el banco de pruebas mismo entra en panic con `[MOLLUSK]: An account required by the instruction was not provided`. Mantén las dos suites en archivos de prueba separados: hablan dos stacks de SVM distintos, y un archivo que mezcle sus tipos no va a compilar — archivos separados es el requisito entero, y las dos suites después conviven felices en un solo crate. Necesitas al menos una prueba de Mollusk para `settle_prize`, porque ese entero impreso es el número "antes" que el Paso 7 te pide registrar.

Verde acá quiere decir que cada borde funciona solo. Eso es necesario y no suficiente, que es la razón entera por la que existe el Paso 8.

**Paso 5. Un caso de fuzz en verde.** Anchor V2 trae empaquetado un banco de pruebas de fuzzing (Crucible). Apúntalo al registry y déjalo tirarle entradas generadas a una instrucción hasta que tengas un caso que sobreviva:

```bash
anchor fuzz init floor-registry              # scaffold a Crucible target for the registry
anchor fuzz run floor-registry --release     # run it; --stateful for sequences of instructions
```

No estás persiguiendo cobertura total en un capstone. Estás demostrando que el banco de pruebas corre contra tu composición y que un target vuelve en verde.

**Paso 6. El checklist de seguridad.** Recorre el checklist por instrucción que este curso viene construyendo: cada cuenta validada por dueño, firmante y PDA; aritmética verificada en todas partes; nada de `unwrap()` en código de programa; targets de CPI fijados al `Program<T>` correcto; y el específico de la composición, la clase de sustitución de cuenta que sobrevive a cualquier framework. El módulo 7 te mostró qué clases de vulnerabilidad mata V2 en tiempo de compilación; la sustitución de cuenta a través de una CPI es la clase que no muere sola, así que confirma que cada cuenta de peldaño es la que querías, por tipo y por seed.

**Paso 7. Profile de CU más una optimización.** Perfila el borde más pesado, `settle_prize`, porque tres saltos son los que más queman. Lee las compute units de la prueba de Mollusk que escribiste en el Paso 4 y anota tu número — para escala, el montaje de verificación propio del curso mide su cadena de registry-a-escrow-a-vault con handlers de stub en los miles-y-pico de CU, y tus handlers de verdad aterrizan más arriba; el número es tuyo, la forma de miles-no-decenas es el chequeo de sanidad. Después vuelve a compilar, haz un solo cambio medido, y anótalo de nuevo. Un cambio concreto que rinde: si tu handler lee una cuenta antes y después de una CPI, y la segunda lectura solo necesita un valor de lamports o de bytes en vez de la vista tipada, elimina la lectura tipada redundante. No fabriques la ganancia; mídela. La regla es la misma que este curso viene sosteniendo desde el módulo 1: reporta el número que viste, no el número que esperabas.

![Un pipeline de ocho etapas que va desde anchor build pasando por la suite de unidad, fuzz, harden, profile de CU, localnet de Surfpool, deploy en devnet, y verify-desde-el-repo local.](assets/v05-flowchart.png)

**Paso 8. La corrida de integración en el localnet de Surfpool.** Este es el paso que atrapa lo que ninguna prueba de unidad de arriba puede. Tus pruebas de LiteSVM demuestran que cada peldaño funciona solo. Nunca paran el salón entero junto, así que una CPI que pasa la cuenta equivocada, o una seed que deriva un vault en la prueba y otro en el salón, pasa de largo las pruebas de unidad y falla solo cuando los programas componen de verdad. `anchor test` en V2 levanta un localnet de Surfpool por defecto, despliega el workspace entero, y corre tus pruebas contra el salón entero de cinco programas corriendo junto, antes de que un solo byte toque devnet. Surfpool es un binario aparte que `anchor test` maneja; si hiciste Digital Assets, este es el mismo Surfpool que vienes manejando desde su módulo 2 — allá forkeaba el estado de mainnet debajo de tus pruebas, acá levanta tu salón de localnet de cinco programas. Instálalo una vez para que el validador default quede en tu PATH:

```bash
# Surfpool's documented installer. The repo moved from txtx to the Solana Foundation
# (the old URL redirects); latest release v1.5.0, checked 2026-08-22. `anchor test`
# needs surfpool >= 1.1.2.
curl -sL https://run.surfpool.run/ | bash
surfpool --version
anchor test                         # V2 default validator is surfpool; runs the floor together
# expect the registry suite line:
#   floor-registry ... passing
```

Haz concreto el modo de falla, porque es el que muerde. Digamos que el `settle_prize` del registry deriva el vault del escrow desde `[b"vault", escrow.key()]` pero tu helper de prueba creó el vault del escrow desde `[b"vault", operator.key()]`. Cada prueba de unidad pasa: la prueba del registry construye sus propias cuentas y nunca cruza la costura, la prueba del escrow también construye las suyas. Después el salón corre junto, el registry le entrega al escrow una dirección de vault que el escrow no reconoce como propia, y la CPI de release falla sobre una cuenta por la que no puede firmar. Ese bug no tiene casa en ninguna prueba de un solo programa. Vive enteramente en la costura, y la corrida de localnet es el único paso antes de devnet que para los dos programas sobre las mismas cuentas al mismo tiempo. Si existen fallas entre programas, salen a la superficie acá, en tu máquina, gratis. Ese es el punto de una corrida de integración en localnet y es por qué saltearla para "solo desplegar y ver" es el atajo más caro de esta lista. Vale saber que este pipeline no son tres herramientas que alguien atornilló juntas. El memo de Jacob Creech sobre la unificación de Anchor (discusión #3742) lo nombró de antemano: "I expect Anchor V2 to unify tools around using Litesvm, using the solana-verify standard, potentially surfpool." LiteSVM para la suite de unidad, Surfpool para la corrida de integración, `solana-verify` para la demostración. Tu capstone es esa frase, ejecutada.

**Paso 9. Despliega a devnet.** Apunta `Anchor.toml` a devnet, fondea la billetera, y despliega el registry junto con los peldaños a los que llama:

```bash
solana config set --url devnet
solana airdrop 2                    # devnet SOL for the deploy
# One callback before you deploy: m08-l2 handed the swap's upgrade authority to
# /tmp/new-authority.json as a rehearsal. `anchor deploy` upgrades the whole
# workspace signed by your workspace wallet, which is no longer the swap's
# authority, so take it back first (and if a reboot already wiped /tmp, that
# program is frozen at its current bytes and you deploy the rest without it).
# Both sides are keypair FILES, which is m08-l2's own rule: a bare pubkey needs
# --skip-new-upgrade-authority-signer-check, and that is not a flag to rehearse.
solana program set-upgrade-authority <SWAP_PROGRAM_ID> -u devnet \
  --upgrade-authority /tmp/new-authority.json \
  --new-upgrade-authority ~/.config/solana/id.json
anchor deploy                       # deploys the workspace to devnet
# expect, per program:
#   Deploy success
#   Program Id: <FLOOR_REGISTRY_PROGRAM_ID>
```

Anota ese program id. El Paso 10 lo necesita dos veces, y la lista de cierre lo pide como evidencia. Si el deploy falla por falta de fondos, vuelve a hacer airdrop; devnet le pone un tope a un solo airdrop bastante por debajo de lo que cuesta desplegar cinco programas de una pasada.

**Paso 10. Verifica desde el repo, localmente, contra el programa en devnet.** Esta es la barrera del capstone. `solana-verify` vuelve a compilar tu programa desde la fuente adentro de una imagen de Docker fijada para que el bytecode sea determinístico, y después compara ese hash con el programa desplegado on-chain. Instálalo, compila, despliega el artefacto verificable, y verifica contra tu programa de devnet:

```bash
cargo install solana-verify --locked   # v0.5.1 (solana-foundation/solana-verifiable-build; the old Ellipsis-Labs URL redirects); re-check the latest release
solana-verify build --library-name floor_registry
solana-verify get-executable-hash target/deploy/floor_registry.so
# solana-verify build just overwrote target/deploy with the deterministic artifact.
# Step 9's anchor deploy shipped a non-deterministic build, so redeploy NOW — skip
# this and the two hashes below will not match:
solana program deploy target/deploy/floor_registry.so \
  --program-id target/deploy/floor_registry-keypair.json
# then compare against the on-chain program:
solana-verify get-program-hash -u devnet <FLOOR_REGISTRY_PROGRAM_ID>
solana-verify verify-from-repo -u devnet \
  --program-id <FLOOR_REGISTRY_PROGRAM_ID> \
  --mount-path programs/floor-registry \
  --library-name floor_registry \
  https://github.com/<you>/quarter-vault
```

Cuando los dos hashes coinciden, demostraste que tu fuente pública reproduce el bytecode exacto que corre en devnet. Esa es una demostración de verdad, y vale la pena ser preciso sobre qué es y qué no es.

![Una línea de tiempo de dos carriles que separa la demostración local de reproducibilidad contra devnet de los pasos de distribución y autoridad exclusivos de mainnet, que se demuestran pero nunca se corren acá.](assets/v06-timeline.png)

El job remoto de OtterSec (la flag `--remote`) manda tu build a un registry público, y la verificación remota solo corre contra mainnet. Squads v4 ejecutando un upgrade bajo un multisig es el flujo de autoridad para un lanzamiento de verdad. Los dos se demuestran en este curso y están etiquetados como exclusivos de mainnet, porque están más allá del cluster de este curso. Ninguno de los dos es la *demostración* de la verificación. La demostración es que el rebuild local coincida con el hash on-chain, y acabas de correrla contra devnet. La reproducibilidad es reproducibilidad en el cluster al que la apuntes.

Sé preciso sobre qué te compra y qué no te compra un hash que coincide, porque acá es donde la gente sobreinterpreta el check verde. Un build verificado demuestra exactamente una cosa: el bytecode que corre on-chain lo produjo la fuente en ese commit, byte por byte, así que nadie metió un programa distinto detrás de la dirección que auditaste. Esa es la propiedad que hace que una auditoría on-chain valga algo, y no es poca cosa. También es estrictamente una afirmación sobre la *procedencia*, no sobre la *corrección*. Un build verificado de un programa con bugs es un bug fielmente reproducido. La verificación les dice a tus usuarios "el código que puedes leer es el código que corre". No les dice que el código esté bien; para eso están tus pruebas, tu caso de fuzz, tu checklist de seguridad, y una auditoría de verdad. Entrega la escalera entera, no solo el último peldaño, y el hash verde quiere decir lo que la gente cree que quiere decir.

## Challenge: produce el salón y llévalo por el ciclo de vida

Nada de scaffold nuevo. La barrera es la cosa entera, ensamblada por ti.

Construye las tres instrucciones restantes del registry (`route_credit`, `settle_prize`, `quote_swap`) usando la gramática del Paso 2, después lleva el salón por cada etapa. Acéptalo como terminado cuando se cumpla todo lo siguiente:

![Una tabla de dos columnas con la lista de cierre, que enumera cada etapa del capstone y su señal concreta de aprobado, desde la suite de unidad en verde pasando por la corrida de localnet en Surfpool, el deploy en devnet, y el verify-desde-el-repo que coincide.](assets/v07-comparison.png)

Vas a saber que lo tienes cuando tres cosas sean verdad al mismo tiempo: `anchor test` imprime `floor-registry ... passing`, el programa está vivo en una dirección de devnet que puedes buscar, y `solana-verify verify-from-repo` contra esa dirección coincide con tu build local. Si la corrida de localnet falla pero cada prueba de unidad pasó, no vayas a devnet. La falla es un bug de composición, que es exactamente lo que el Paso 8 existe para atrapar, y es más barato arreglarlo en tu máquina que depurarlo cruzando un cluster. Si el verify no coincide, tu artefacto desplegado y tu fuente se fueron a la deriva; vuelve a compilar con `solana-verify build`, vuelve a desplegar ese `.so` exacto, y verifica de nuevo.

Esa es la escalera, de arriba abajo. Construiste un counter y sentiste desaparecer el impuesto de deserialización. Le diste custodia, después una condición, después un precio. Lo endureciste, lo fuzzeaste, lo perfilaste y lo entregaste. Y ahora compusiste todo eso en un solo programa que corre el salón entero y demostraste, desde tu propia fuente, que la cosa que está en devnet es la cosa que escribiste. Vale la pena sentarse con eso un segundo. Cinco programas, cuatro de los cuales escribiste desde un archivo en blanco, compuestos por un quinto, demostrados byte por byte contra la fuente que puedes publicar. Eso no es un artefacto de tutorial. Esa es la forma de un deploy de verdad.

El salón corre, verificado, en devnet. Queda una pregunta, y es la que decide si algo de esto importa para el código que ya tienes: ¿deberías mover una base de código de verdad a V2 hoy? El módulo final mapea los dos deltas de versión desde fuentes primarias y porta un programa 0.31/1.0 de verdad a un V2 que compila y está probado. Demostraste que puedes construir V2 desde cero. Ahora demuestras que puedes traer el mundo viejo contigo.
