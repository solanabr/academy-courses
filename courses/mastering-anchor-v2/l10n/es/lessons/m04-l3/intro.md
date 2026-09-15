# Composición: construye el prize-escrow

La lección pasada el modelo de borrow hizo algo calladamente radical: mató la trampa de `.reload()` en tiempo de compilación. Una vez que un `CpiHandle` estaba vivo, el compilador se negaba a dejarte leer la cuenta tipada de la que venía, así que los datos obsoletos-después-de-CPI dejaron de ser una disciplina que tenías que recordar y pasaron a ser una cosa que no compilaba. Te apoyaste en el compilador en vez de en tu propia atención. Esa es toda la personalidad de V2, y esta lección es donde se paga a sí misma.

Acá está el dolor. Un vault que solo firma por sí mismo es una alcancía. Útil, claro, pero nunca tiene que confiar en nadie. En el momento en que aparece una segunda parte (un operador que fondea un premio, un jugador que lo reclama solo si se lo ganó) ya no estás escribiendo un programa. Estás cableando dos programas juntos y apostando tu dinero a la costura que hay entre ellos. Esa costura se llama composición, y equivocarse en ella es donde los escrows tienen fugas.

Así que antes de teorizar, cablea la costura. R3 es un segundo programa en el workspace del vault, así que créalo ahí y apúntalo al vault. Desde la raíz de tu workspace `quarter-vault`:

```bash
anchor new quarter-prize        # adds programs/quarter-prize and registers it in Anchor.toml
```

Después cablea R3 para que consuma R2 de la forma en que los programas V2 se consumen entre sí: a través de la **interfaz** del vault, no de su fuente. `declare_program!` lee el IDL de un programa — la descripción JSON de sus instrucciones y cuentas que el toolchain deriva del fuente — y genera toda la superficie de CPI a partir de él en tiempo de compilación. Dos jugadas. Primero, cosecha el IDL del vault dentro de un directorio `idls/` en la raíz del workspace (la macro sube desde el crate que consume y usa el primer directorio `idls/` que encuentra):

```bash
mkdir -p idls
( cd programs/quarter-vault && anchor idl build -o ../../idls/quarter_vault.json )
```

Segundo, abre `programs/quarter-prize/Cargo.toml` — y fíjate en lo que *no* está ahí. Ninguna fila para el vault: la interfaz llega como JSON, no como crate.

```toml
[dependencies]
# crates.io, the same release your tag-pinned CLI was built from (see m01-l2)
anchor-lang = "2.0.0-rc.1"
# The pins from m01-l2 — every program crate in this course carries them (issue #4937's class).
wincode = { version = "0.5", features = ["derive"] }
# The arcade-workspace row from m02-l1, verbatim. R3 lands in R2's workspace, and the
# two share one lock: the members have to agree on solana-address or nothing resolves.
solana-address = ">=2.6.1, <2.7"
# NO `quarter-vault = { path = "../quarter-vault", features = ["cpi"] }` row. The
# scaffold's feature table ships a `cpi = ["no-entrypoint"]` hook for source-level
# consumption, and this course deliberately never uses it — the IDL path below has
# none of its rc.1 sharp edges, and it is V2's own cross-program mechanism.
```

Después, arriba de `programs/quarter-prize/src/lib.rs`, una línea trae a la existencia el módulo generado del vault:

```rust
declare_program!(quarter_vault);
```

Después `anchor build`. Todavía no hay nada que reclamar, y el build va a empezar a fallar en el momento en que escribas código real contra R2, porque R2 sigue siendo de auto-custodia y no puede tomar una segunda parte. Arreglar eso es el Paso 1 del Lab y viene antes que todo lo demás. Lo que tienes ahora mismo es el módulo `cpi` del callee en scope — generado desde el IDL, y rastreado por `include_bytes!`, así que una edición del IDL obliga a quien llama a recompilar y un desajuste en la forma de un tipo es un error de compilación — aunque un IDL obsoleto de una manera que todavía pasa el chequeo de tipos compila en verde y solo desencaja en tiempo de ejecución, así que regenera después de cada cambio del callee en vez de confiar en el rastreo — y un compilador que te va a decir exactamente qué handles espera el vault. Ese loop de feedback es la lección.

> Nota de frescura: esto está escrito contra la release candidate de Anchor V2 en la línea 2.x (el árbol de docs publicado bajo `v2`), 2026-08-22, tag más nuevo `2.0.0-rc.1` (publicado en crates.io el 2026-08-12). `avm install` no puede traerla, como mostró m01-l2: no se cortó ningún GitHub Release para el tag v2, así que el binario precompilado que descarga da 404. Construye el CLI desde el canal documentado en su lugar, fijado al tag: `cargo install --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1 anchor-cli --locked --force`, y vuelve a revisar si hay una rc más nueva o un tag estable antes de compilar. El `anchor-cli 1.1.2` default de la máquina es la línea V1 y no va a compilar los features `unsafe(dup)`, `CpiHandle` ni Pod-`Account` de abajo.

## Resumen

R3, el prize-escrow. Dos instrucciones, dos partes, una condición. `reserve` toma los lamports de un operador y los estaciona dentro de una instancia real del quarter-vault a través de una llamada entre programas, y después registra para quién es el premio y qué hace falta para ganarlo. `redeem` libera esos lamports, pero solo cuando se cumple la condición de victoria y solo hacia el que llama que el escrow nombró, exactamente ese. Un reclamo prematuro falla. Un reclamo del que llama equivocado falla. El dinero vive en el vault que construiste en m04-l1 todo el tiempo, que es justamente el punto: R3 no reimplementa la custodia, se *compone* sobre la de R2.

La ayuda se repliega a propósito. La CPI que deposita en el vault está trabajada completa abajo, con cada handle explicitado, porque un depósito entre programas es el músculo nuevo y deberías verlo moverse una vez. Las dos líneas que lo hacen *seguro*, la verificación de la condición de victoria y el constraint de quien llama, te toca llenarlas: están marcadas `TODO(you)` y el checkpoint muestra la respuesta. Después el challenge te suelta por completo: una segunda forma de ganar, independiente, demostrada en la prueba por ti mismo.

La ruta de la lección: primero cómo funciona de verdad la composición en V2 (el borde del vault, la verificación de quien llama, el default de cuenta duplicada, y la única regla de orden que separa un escrow de un exploit), después el build, después un checkpoint que lo demuestra, después lo extiendes tú.

## Cómo un programa construye sobre otro

La composición es un programa invocando una instrucción en otro y construyendo sobre el estado de ese programa. Esa es la idea entera, y también es donde tu superficie de confianza deja de ser solo tuya. Cuando R3 llama a R2, la corrección de R3 ahora *incluye* la corrección de R2. Ya no estás confiando solo en tu propio código.

Concretamente, el premio nunca se queda en el escrow. El escrow es un registro: dice "50,000,000 lamports para este jugador, liberados bajo esta condición, custodiados allá". Los lamports mismos viven en una instancia del quarter-vault, y R3 llega a ese vault solo por CPI. Esa única flecha, R3 depositando dentro de un vault de R2 y después liberando desde él, es la razón por la que esta lección declara que R3 consume R2.

![R3 hace CPI hacia R2 para depositar los lamports del operador en reserve y, después de verificar a quien llama y el puntaje, para liberarlos en redeem, sin sostener ningún premio él mismo.](assets/v01-flowchart.png)

Pero ¿comparado con qué? El diseño obvio y más simple es dejar que el escrow sostenga los lamports directamente: saltarse R2 por completo, acreditar la cuenta del escrow, debitarla en redeem. Funciona, y para una sola vez es menos código. Pero mira lo que entregas a cambio. Estarías reimplementando la custodia (los movimientos de lamports, la matemática del rent, las verificaciones de autoridad) dentro de R3, una segunda copia de lógica que ya vive en R2 y que ya probaste. Dos copias se separan. El día que arregles un bug de custodia en el vault, la copia privada del escrow todavía lo tiene. Componer sobre R2 en cambio quiere decir que el escrow es dueño de exactamente una cosa, la *decisión* de liberar, y delega el *sostener* al programa construido para eso. Ese es el canje por el que argumenta la lección entera: una cosa más chica sobre la que puedes razonar, atornillada a una cosa probada en la que ya confías.

Para que esa delegación funcione, la instancia del vault tiene que responderle al escrow. R2 deriva un par de vault desde su dueño, `[b"vault", owner]` para el libro mayor y `[b"sol", owner]` para los lamports, y la instancia que usa este escrow se crea con el PDA del escrow como ese dueño. Así que hay exactamente un par de vault por escrow y solo el escrow puede mover sus lamports. Por eso `redeem` puede firmar el withdraw con las seeds del escrow y nadie más puede. La relación de autoridad es el contrato entre los dos programas; equivócate en ella y o el depósito aterriza en algún lugar al que no puedes llegar o el withdraw se niega a firmar.

Esa delegación le cuesta a R2 exactamente tres cuentas extra, y vale la pena nombrarlo en vez de esconderlo. El vault que construiste en m04-l1 es un vault de *auto-custodia*: `deposit` saca lamports de la misma `authority` que es dueña del libro mayor, y `withdraw` le paga de vuelta a esa misma authority. Un escrow necesita esos dos roles divididos, porque el operador fondea un vault del que el escrow es dueño, y el jugador, no el escrow, recibe el pago. Así que el `deposit` de R2 gana una cuenta `funder` que provee los lamports, `withdraw` gana una cuenta `destination` que los recibe, y `init_vault` gana un `funder` que paga el rent, todas distintas de la `authority` de la que se derivan las seeds. Tres cuentas extra, ninguna lógica de custodia nueva, y R3 construye sobre el mismo código en vez de copiarlo. Eso es lo que "componible" cuesta de verdad y compra de verdad. El Paso 1 del Lab es esa edición, y es lo primero que haces, porque nada de R3 compila hasta que R2 pueda tomar un funder y pagarle a un destination.

### La verificación de quien llama: address, no has_one

Allá en m03-l2, cuando recorrimos el catálogo de constraints, uno de los renames mecánicos fue la verificación de autoridad. `has_one = maker` está obsoleto en V2. Todavía parsea, el compilador nada más lo subraya y avisa, que es exactamente la migración guiada y de hacerla una sola vez sobre la que trataba toda esa lección. El reemplazo es `address = parent.field`:

```rust
// V1 idiom, deprecated in V2 (parses, warns):
#[account(has_one = maker)]
pub escrow: Account<Escrow>,

// V2 idiom: reads as what it does, this account's address must equal that stored field
#[account(mut, address = escrow.maker)]
pub maker: UncheckedAccount,
```

El cambio es más que cosmético. `has_one` estaba atado a un campo cuyo nombre coincidía con el nombre de la cuenta. `address` toma cualquier expresión, así que `address = escrow.player` en el que reclama dice, sin rodeos, "la cuenta que se pasa acá tiene que ser la pubkey que este escrow registró como el player". Esa es la verificación de quien llama para toda la liberación condicional, y es una línea.

![has_one = maker se vuelve address = escrow.player; la misma verificación de igualdad, ahora escrita como una expresión, y el compilador avisa sobre la forma obsoleta.](assets/v02-annotated-code.png)

### El default de duplicados mutables: distintas está bien, con alias no

Acá hay un default que hace tropezar a la gente la primera vez y después nunca más. V2 rechaza *cuentas mutables duplicadas*. Entrega la misma cuenta bajo dos nombres en una instrucción, con aunque sea uno de esos slots marcado `mut`, y la validación falla antes de que corra tu handler, con `ConstraintDuplicateMutableAccount`.

Lee eso con cuidado, porque la mala lectura común es cara. *No* quiere decir "dos cuentas mutables están prohibidas". Tu `redeem` toma el escrow (mutable, se cierra) y el vault (mutable, sus lamports se mueven), las dos mutables, y V2 está perfectamente contento, porque son dos cuentas diferentes. Lo que V2 rechaza es el *aliasing*: la misma cuenta entregada dos veces bajo dos nombres, donde una escritura mutable pisa en silencio la otra. Esa es una clase de bug real, y ahora es un error de compilación y de carga en vez de un incidente a las 2 de la mañana.

¿Por qué el default no te cuesta nada en tiempo de ejecución para el caso común? Porque la verificación está dividida en dos lugares. La macro `#[derive(Accounts)]` calcula, en tiempo de compilación, una máscara de 256 bits de qué campos son mutables: este es el `MUT_MASK`, una const asociada cocinada dentro de tu struct de cuentas. Después, mientras el dispatcher carga las cuentas de la instrucción, recorre esa máscara contra un bitvec en tiempo de ejecución de las direcciones vistas hasta ahí. La *forma* de la verificación (qué campos son mutables) se decide cuando compilas. Los *valores* (si dos de esas direcciones son iguales) se deciden cuando corre la transacción. Const de tiempo de compilación, despacho en tiempo de ejecución. Esa división es la razón por la que es barata y por la que no se la puede engañar.

![La macro derive emite un MUT_MASK de 256 bits en tiempo de compilación con los campos mutables; el dispatcher verifica las direcciones de esos campos contra un bitvec en tiempo de ejecución mientras carga las cuentas, fallando ante un alias.](assets/v03-diagram.png)

Cuando de verdad quieres pasar una cuenta dos veces como mutable (una instrucción batch que toca dos pools de premios que resultan resolver al mismo vault, digamos) haces opt-out por campo, y el opt-out está escrito para que lo sientas: `unsafe(dup)`. El `dup` a secas sin el wrapper `unsafe` es un error de compilación en V2, a propósito. El keyword es la luz del cinturón: estás apagando una verificación de alias, así que ahora el riesgo de aliasing es tuyo y tienes que escribir el handler para que nunca sostenga dos referencias mutables en conflicto a esa cuenta.

| Tu `redeem` toma... | Veredicto de V2 | Lo que escribes |
|---|---|---|
| escrow (mut) + vault (mut), cuentas diferentes | aceptado | nada extra, esto es composición normal |
| el mismo vault dos veces, cualquiera de los slots mut | rechazado | `ConstraintDuplicateMutableAccount` en la carga |
| el mismo vault dos veces, y era a propósito | aceptado | `#[account(mut, unsafe(dup))]` en las dos, y el aliasing es tuyo |
| `dup` a secas sin `unsafe` | error de compilación | el compilador te dice que escribas `unsafe(dup)` |

### La única regla de orden: verifica antes de pagar

Ahora la regla que separa un escrow de una máquina de donaciones. Una liberación condicional es apenas tan segura como el *cuándo* de su verificación. La guarda tiene que correr y pasar antes de que se mueva un solo lamport.

Vale la pena descartar las alternativas ingenuas, porque cada una se ve bien hasta que ya no. La primera jugada ingenua es "págale al jugador, después verifica la condición, y si era falsa, revierte". Sé honesto sobre la mecánica primero: en Solana eso *sí* revierte — una instrucción que termina en error deshace cada cambio de cuenta de la transacción, la CPI del pago incluida, exactamente como enseñó m04-l1. Nada puede comprometerse parcialmente antes de tu revert. Así que la objeción no es que pagar-y-después-verificar pierda lamports hoy. Es que su seguridad pende por completo de que la verificación siga siendo fatal y siga dentro de esta instrucción, para siempre — y las refactorizaciones erosionan exactamente eso. Un `require!` se ablanda hasta volverse una rama que solo loguea. La condición se va corriendo hacia un helper que retorna en vez de dar error. La verificación empieza a leer estado que la propia CPI del pago acaba de cambiar, así que mide la cosa equivocada. El día que pase cualquiera de esas cosas, la transferencia que ya compusiste sale adelante con una condición falsa, y ningún compilador lo marca. La guarda primero no tiene ningún invariante de ese tipo que mantener: si la guarda nunca pasa, el código del pago ni siquiera corre. La segunda jugada ingenua es "confía en la atomicidad, verifica en cualquier parte de la instrucción". La misma dependencia frágil, más un costo más sutil: el orden es documentación. Un auditor que lee `redeem` debería ver la condición haciendo de barrera para el pago por *posición*, no tener que demostrar que algún revert posterior te salva.

Así que la forma real queda forzada: verifica a quien llama y la condición, y solo entonces arma la CPI de liberación. En el handler, las líneas `require!` van primero y la llamada `quarter_vault::cpi::withdraw` va al final. Nada se mueve hasta que las guardas hayan pasado.

![El redeem seguro verifica a quien llama y la condición antes de la CPI de withdraw; pagar primero y verificar después, o verificar a mitad de la instrucción y confiar en la atomicidad, fallan las dos.](assets/v04-flowchart.png)

### El trade-off que estás comprando

La composición no es gratis, y nombrar la cuenta es la parte honesta. Vienen tres costos pegados, y o los demuestras seguros o haces opt-in y los asumes como tuyos.

Primero, tu superficie de confianza se multiplicó. R3 depende de que R2 sea correcto; un bug en el withdraw del vault ahora es un bug en tu escrow. Segundo, la pila de CPI está acotada, en esa altura de pila de 5 a la que m04-l1 ya le puso un número y una salvedad de SIMD pendiente. Tu escrow llamando al vault se sienta en la altura 2, lejísimo de ahí, pero un protocolo que se compone cinco niveles de profundidad es un protocolo que algún día va a chocar contra la pared, y esta es la lección donde empiezas a gastar ese presupuesto. Tercero, el riesgo de aliasing: dos cuentas mutables en una instrucción se rechazan por defecto, y el día que escribas `unsafe(dup)` firmaste las consecuencias tú mismo.

![La altura de pila 1 es la instrucción de nivel superior y cada CPI suma uno a un techo vivo de 5, así que la llamada de escrow a vault de esta lección se sienta en la altura 2.](assets/v05-table.png)

Hay una tesis debajo de todo esto. El issue manifiesto de Anchor que arrancó V2, el número 4390, "Zero-copy account deserialization by default," argumentó a favor de exactamente una idea: hacer el modelo de cuentas seguro por defecto y dejar que las cosas no sólidas fallen al compilar. La composición con borrow rastreado es esa tesis aplicada al caso más difícil, un programa construyendo sobre el estado de otro. El `CpiHandle` que peleaste la lección pasada y el default de duplicados mutables que acabas de conocer son el mismo principio con dos sombreros puestos.

## Lab: construye R3

Terminal nueva. Esta es la máquina de premios de verdad del barcade: un operador carga un premio de peluche detrás de un puntaje máximo, y la máquina arcade solo paga cuando un jugador de verdad lo supera.

### Paso 1: divide los roles de R2 para que una segunda parte pueda usarlo

Haz esto primero, porque nada de abajo compila hasta que esté hecho. El vault que construiste en m04-l1 es de auto-custodia: `deposit` saca lamports de la misma `authority` de la que se derivan las seeds, y `withdraw` le paga de vuelta a esa misma `authority`. Un escrow necesita esos roles divididos. Abre R2 y agrega una cuenta a cada struct, dejando cada seed, bump y guarda exactamente como estaba:

```rust
// quarter-vault: Deposit gains a funder (who pays) alongside authority (who owns).
#[derive(Accounts)]
pub struct Deposit {
    /// CHECK: seeds derive from this; a top-up needs no permission from the owner
    pub authority: UncheckedAccount,   // was Signer: the owner no longer has to sign a deposit
    #[account(mut)]
    pub funder: Signer,                // NEW: sources the lamports, and signs for them
    #[account(mut, seeds = [b"vault", authority.address().as_ref()], bump = state.bump)]
    pub state: Account<Vault>,
    #[account(mut, seeds = [b"sol", authority.address().as_ref()], bump = state.sol_bump)]
    pub sol_vault: SystemAccount,
    pub system_program: Program<System>,
}

// quarter-vault: Withdraw gains a destination (who receives) alongside authority (who owns).
#[derive(Accounts)]
pub struct Withdraw {
    #[account(address = state.owner @ VaultError::NotVaultOwner)]
    pub authority: Signer,
    #[account(mut)]
    pub destination: UncheckedAccount, // NEW: receives the payout
    #[account(mut, seeds = [b"vault", authority.address().as_ref()], bump = state.bump)]
    pub state: Account<Vault>,
    #[account(mut, seeds = [b"sol", authority.address().as_ref()], bump = state.sol_bump)]
    pub sol_vault: SystemAccount,
    pub system_program: Program<System>,
}
```

`InitVault` necesita el mismo tratamiento, y esta es la que la gente se salta. Ahora mismo dice `authority: Signer` con `payer = authority`, que dice que el dueño del vault autoriza su creación y además fondea su rent. Un PDA de escrow no puede ninguna de las dos: no tiene clave con la que firmar ni lamports con los que pagar. Divide esos dos trabajos de la misma forma:

```rust
// quarter-vault: InitVault gains a funder (who pays rent) alongside authority (who owns).
#[derive(Accounts)]
pub struct InitVault {
    /// CHECK: seeds derive from this; creating someone's vault needs no permission
    pub authority: UncheckedAccount,   // was Signer
    #[account(mut)]
    pub funder: Signer,                // NEW: pays rent for both accounts
    #[account(init, payer = funder, space = Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE,
              seeds = [b"vault", authority.address().as_ref()], bump)]
    pub state: Account<Vault>,
    #[account(init, payer = funder, space = 0, owner = System::id(),
              seeds = [b"sol", authority.address().as_ref()], bump)]
    /// CHECK: UncheckedAccount because SystemAccount has no init path in V2, and
    /// only UncheckedAccount may `init` with a foreign owner. `owner = System::id()`
    /// does that handoff — without it the account stays owned by THIS program and
    /// every later SystemAccount read fails at load with IllegalOwner.
    pub sol_vault: UncheckedAccount,
    pub system_program: Program<System>,
}
```

Crear un vault para una dirección es inofensivo: le cuesta rent al funder y le da al dueño una cuenta vacía de la que solo él puede sacar. Por eso soltar la firma acá es seguro y no lo sería en `Withdraw`.

Después apunta las dos CPIs de `Transfer` a las cuentas nuevas: el `from` de `deposit` se vuelve `funder`, y el `to` de `withdraw` se vuelve `destination`. Nada más cambia, y ese es justamente el punto. Tres cuentas extra repartidas en tres structs, ninguna lógica de custodia nueva, y R3 puede construir sobre el mismo código en vez de copiarlo.

![La auto-custodia mantiene cada rol en manos del jugador a través de cuentas distintas, ya que el default de duplicados mutables rechaza una sola clave con alias repartida entre slots, mientras que el vault del que el escrow es dueño divide los roles entre el PDA del escrow, el operador y el jugador con la lógica de custodia de R2 sin cambios.](assets/v06-comparison.png)

Fíjate en la única degradación: la `authority` de `Deposit` deja de ser un `Signer`. Recargar el vault de alguien nunca necesitó su permiso, solo su dirección para derivar las seeds, y el PDA del escrow no puede firmar un depósito del que es apenas el dueño. La `authority` de `Withdraw` sigue siendo un `Signer`, que es exactamente la cuenta que el PDA del escrow va a satisfacer a través de `invoke_signed` en el Paso 4.

Checkpoint: `anchor build` en el workspace del vault compila — si en cambio falla, moviste una seed o un bump; ponlo de vuelta. Después vuelve a correr la prueba de withdraw de m04-l1, y córrela primero de la forma tentadora: pasa la clave del jugador a `authority`, `funder` y `destination`, como la vieja prueba de auto-custodia colapsaba todo en una sola billetera. Falla en la primerísima instrucción con `Custom(2040)` — `ConstraintDuplicateMutableAccount`. Una clave en dos slots es aliasing, y la guarda rechaza una cuenta con alias en el momento en que *cualquiera* de sus slots es `mut`; una `authority` de solo lectura no la excusa, porque en `InitVault` el `funder` mut le hace alias y en `Withdraw` lo hace el `destination` mut. Ese 2040 es el default de duplicados mutables de la vista general haciendo su trabajo sobre tu propio programa, una lección antes. El arreglo es la misma división de roles que acabas de hacer en los structs, aplicada a la prueba: dale a `funder` y a `destination` sus propios keypairs, como el escrow va a sostener partes distintas en el salón. Claves distintas, corrida en verde. Una última jugada antes de que te vayas de este paso: vuelve a correr la cosecha del IDL de la apertura. Acabas de cambiar la interfaz de R2, y `declare_program!` compila contra el JSON, no contra el fuente — el archivo obsoleto es exactamente el error de compilación que nombra el checkpoint de abajo.

### Paso 2: el registro del escrow

El escrow es chico y de tamaño fijo, así que es un `Account` Pod: cada campo es un escalar plano, ningún `Vec` ni `String`, que es lo que le deja a V2 respaldarlo con un `Account<T>` zero-copy en vez de un `BorshAccount<T>`. Crea `src/state.rs`:

```rust
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub maker: Address,      // the operator who funded the prize
    pub player: Address,     // the only caller allowed to redeem
    pub vault: Address,      // the R2 vault ledger holding this prize
    pub amount: u64,         // prize size, in lamports
    pub winning_score: u64,  // the bar the player must clear
    pub bump: u8,            // canonical bump, stored so we never re-derive
    pub _pad: [u8; 7],   // explicit tail padding: V2 rejects implicit pad bytes
}
```

Una cosa que *no* escribes acá, y la omisión merece un momento. `Program<T>` necesita un tipo marcador con un impl de `Id`, y `#[program]` no emite ningún marcador — genera exactamente tres módulos hermanos, `instruction`, `accounts` y `cpi` — así que un consumidor a nivel de fuente tendría que escribir uno a mano, con `IDL_ADDRESS` y todo, y un impl que se salta `IDL_ADDRESS` igual compila mientras el IDL pierde en silencio la dirección del callee. `declare_program!` genera el marcador en su lugar, en `quarter_vault::program::QuarterVault`, con `IDL_ADDRESS` ya llenado desde el JSON. Una cosa menos que mantener a mano, una cosa menos en la que equivocarse en silencio.

Checkpoint: `anchor build` compila. Si se queja del relleno o de un campo que no es Pod, agregaste algo de largo variable; mantén el registro escalar.

### Paso 3: reserve, con la CPI de depósito trabajada completa

`reserve` crea el registro del escrow y, en la misma instrucción, mueve los lamports del operador dentro de una instancia del quarter-vault llamando a R2. Este es el ejemplo trabajado: lee cada handle.

Una nota de ubicación que vale para este listado y para el de `redeem` en el paso 4, porque pegar cualquiera de los dos en el nivel superior de `lib.rs` compila como código muerto y después falla el paso 5 con un `quarter_prize::instruction::Reserve` sin resolver. Las líneas `use` y el struct `#[derive(Accounts)]` son items de nivel superior. La `pub fn` va **adentro** del módulo `#[program] pub mod quarter_prize { use super::*; … }`, exactamente donde fue el `init_vault` de m03-l1 — ese módulo es lo que genera los builders `instruction::` y `accounts::` que la prueba busca.

```rust
use anchor_lang::prelude::*;
use quarter_vault::cpi as vault_cpi;
use quarter_vault::program::QuarterVault; // generated by declare_program!
use quarter_vault::Vault;                 // the vault's account type, also generated
use crate::state::Escrow;

#[derive(Accounts)]
pub struct Reserve {
    #[account(
        init,
        payer = maker,
        space = Escrow::DISCRIMINATOR.len() + Escrow::INIT_SPACE,
        seeds = [b"escrow", maker.address().as_ref(), player.address().as_ref()],
        bump
    )]
    pub escrow: Account<Escrow>,

    #[account(mut)]
    pub maker: Signer,

    /// CHECK: recorded as the future claimant; never signs here
    pub player: UncheckedAccount,

    // R2's vault pair, both derived from the escrow PDA as owner. Neither exists
    // yet: `reserve` creates them by CPI, which is why vault_state is unchecked
    // here rather than a typed Account<Vault>.
    /// CHECK: created by the init_vault CPI below and validated by R2's own seeds
    #[account(mut)]
    pub vault_state: UncheckedAccount,
    #[account(mut)]
    pub vault_sol: SystemAccount,      // zero data, holds the actual lamports

    pub quarter_vault_program: Program<QuarterVault>,
    pub system_program: Program<System>,
}

pub fn reserve(ctx: &mut Context<Reserve>, amount: u64, winning_score: u64) -> Result<()> {
    // Copy scalars out BEFORE any handle borrows ctx.accounts (the borrow model,
    // again). The leading `*` is what makes each one a copy: .address() returns
    // &Address, and a bare binding would hold a borrow into ctx.accounts instead.
    let maker = *ctx.accounts.maker.address();
    let player = *ctx.accounts.player.address();
    let vault_ledger = *ctx.accounts.vault_state.address();

    // FIRST: bring the escrow's vault pair into existence. The escrow PDA owns it
    // (the seeds derive from the escrow), but the maker funds the rent, which is
    // exactly the split you just made to R2's InitVault. No PDA signature needed:
    // creating a vault for an address is not an authority action.
    let init_accounts = vault_cpi::accounts::InitVault {
        authority: ctx.accounts.escrow.cpi_handle(),
        funder: ctx.accounts.maker.cpi_handle_mut(),
        state: ctx.accounts.vault_state.cpi_handle_mut(),
        sol_vault: ctx.accounts.vault_sol.cpi_handle_mut(),
        system_program: ctx.accounts.system_program.cpi_handle(),
    };
    vault_cpi::init_vault(CpiContext::new(
        ctx.accounts.quarter_vault_program.address(),
        init_accounts,
    ))?;

    // THEN: deposit the operator's lamports INTO that vault (the R3 -> R2 edge).
    let cpi_accounts = vault_cpi::accounts::Deposit {
        authority: ctx.accounts.escrow.cpi_handle(),        // owns the vault; signs nothing here
        funder: ctx.accounts.maker.cpi_handle_mut(),        // the operator's lamports
        state: ctx.accounts.vault_state.cpi_handle_mut(),
        sol_vault: ctx.accounts.vault_sol.cpi_handle_mut(),
        system_program: ctx.accounts.system_program.cpi_handle(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.quarter_vault_program.address(), cpi_accounts);
    vault_cpi::deposit(cpi_ctx, amount)?;

    // Only after the money is custodied, and after the handles have dropped, do we write the record.
    let escrow = &mut ctx.accounts.escrow;
    escrow.maker = maker;
    escrow.player = player;
    escrow.vault = vault_ledger;
    escrow.amount = amount;
    escrow.winning_score = winning_score;
    escrow.bump = ctx.bumps.escrow;
    Ok(())
}
```

Tres cosas para notar, porque son la gramática de CPI de V2. El callee expone `vault_cpi::accounts::Deposit`, un struct en el que cada campo es un `CpiHandle`. Lo llenas con `.cpi_handle_mut()` para las cuentas que el callee va a escribir (el par de vault, el funder) y con `.cpi_handle()` para el resto. `CpiContext::new` toma el callee como un `&Address`, que es exactamente lo que devuelve el `.address()` de la cuenta del programa — así que se pasa derecho, sin envolverlo en un `AccountInfo`. Y el wrapper generado `vault_cpi::deposit(cpi_ctx, amount)` empaca los args e invoca. Ese es el mismo handle con el que peleaste la lección pasada: una vez que `cpi_handle_mut()` toma prestado el vault, no puedes además tocarlo como cuenta tipada hasta que la llamada retorne, que es precisamente cómo el compilador te mantiene honesto.

Checkpoint: `anchor build`. El compilador resuelve `quarter_vault::cpi::*` porque `declare_program!` generó el módulo entero desde `idls/quarter_vault.json`. Si muere con `` `idls` directory not found ``, el paso de cosecha de la apertura nunca corrió; si no encuentra un *campo* como `funder`, tu JSON es anterior a la división de roles — vuelve a correr la cosecha, porque la interfaz cambió y el archivo tiene que cambiar con ella; y si el módulo mismo queda sin resolver desde un archivo como `src/instructions/reserve.rs`, escribe la ruta desde la raíz del crate — `use crate::quarter_vault::cpi as vault_cpi;` — porque la macro genera el módulo en la raíz, y el `use quarter_vault::…` a secas de un submódulo no puede verlo.

### Paso 4: redeem, y las dos líneas que son tuyas

`redeem` es la liberación condicional. Las cuentas y el esqueleto están acá; dos lugares son `TODO(you)`. Llénalos, después verifica contra la solución.

```rust
use anchor_lang::prelude::*;
use quarter_vault::cpi as vault_cpi;
use quarter_vault::program::QuarterVault; // generated by declare_program!
use quarter_vault::Vault;                 // the vault's account type, also generated
use crate::state::Escrow;

#[derive(Accounts)]
pub struct Redeem {
    #[account(
        mut,
        close = maker,
        seeds = [b"escrow", escrow.maker.as_ref(), escrow.player.as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<Escrow>,

    // Bound to the exact vault the escrow recorded: the record's vault, not any vault.
    #[account(mut, address = escrow.vault @ EscrowError::WrongVault)]
    pub vault_state: Account<Vault>,   // mutable AND distinct from escrow: no unsafe(dup) needed
    #[account(mut)]
    pub vault_sol: SystemAccount,      // the lamports actually leave from here

    // TODO(you): constrain this to the recorded player. One line.
    #[account(mut)]
    pub player: Signer,

    #[account(mut, address = escrow.maker)]
    pub maker: UncheckedAccount,       // close destination: rent returns to the operator

    pub quarter_vault_program: Program<QuarterVault>,
    pub system_program: Program<System>,
}

pub fn redeem(ctx: &mut Context<Redeem>, final_score: u64) -> Result<()> {
    // Copy scalars out BEFORE any handle borrows the escrow (the borrow model, again).
    let amount = ctx.accounts.escrow.amount;
    let winning = ctx.accounts.escrow.winning_score;
    let maker_key = ctx.accounts.escrow.maker;
    let player_key = ctx.accounts.escrow.player;
    let bump = ctx.accounts.escrow.bump;

    // TODO(you): check the win condition BEFORE the payout CPI. One line.

    let seeds: &[&[u8]] = &[b"escrow", maker_key.as_ref(), player_key.as_ref(), &[bump]];
    let signer: &[&[&[u8]]] = &[seeds];

    let cpi_accounts = vault_cpi::accounts::Withdraw {
        authority: ctx.accounts.escrow.cpi_handle(),   // escrow PDA owns the vault and signs for it
        state: ctx.accounts.vault_state.cpi_handle_mut(),
        sol_vault: ctx.accounts.vault_sol.cpi_handle_mut(),
        destination: ctx.accounts.player.cpi_handle_mut(),
        system_program: ctx.accounts.system_program.cpi_handle(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.quarter_vault_program.address(), cpi_accounts)
        .with_signer(signer);
    vault_cpi::withdraw(cpi_ctx, amount)?;
    Ok(())
}

#[error_code]
pub enum EscrowError {
    #[msg("The escrow does not custody this vault")]
    WrongVault,
    #[msg("The win condition has not been met")]
    ConditionNotMet,
}
```

Las dos respuestas. En la cuenta `player`, la verificación de quien llama es una línea: `#[account(mut, address = escrow.player)]`. Eso es `has_one` retirado a favor de una expresión, exactamente el momento de migración de más arriba. Y la condición, puesta donde se sienta el `TODO` para que le haga de barrera a la CPI por posición:

```rust
require!(final_score >= winning, EscrowError::ConditionNotMet);
```

Fíjate en que el escrow (mutable, se cierra), el libro mayor del vault y el vault de SOL (mutable, sus lamports se mueven) se sientan lado a lado, los tres `mut`, y V2 nunca pide `unsafe(dup)`, porque son cuentas distintas.

Fíjate también en lo que R3 *no* verifica. Solo `vault_state` lleva un constraint `address =`, que lo ata al vault que este escrow registró. `vault_sol` no tiene ningún constraint de seeds acá, y eso es deliberado y no descuidado: el propio struct `Withdraw` de R2 ya valida el par, derivando el vault de SOL desde `[b"sol", authority]` contra el `sol_bump` almacenado del libro mayor. Volver a derivarlo en R3 pagaría la misma verificación dos veces y, peor, recalcularía un bump que R2 ya almacenó. Componer quiere decir confiar en que los constraints del callee son el trabajo del callee. La verificación que te quedas es la única que solo *tú* puedes hacer: que este vault es el vault que este escrow nombró. Todo lo demás es R2 volviendo a ganarse la confianza que depositaste en él al depender de él.

Checkpoint: `anchor build` está limpio, no queda ningún aviso de obsolescencia de `has_one` (lo reemplazaste), y `redeem` se lee de arriba a abajo como copiar-escalares, guarda, firmar, pagar.

Aléjate un paso y mira toda la vida de un premio. Existe en exactamente dos estados, y el modelo de cuentas hace que las transiciones sean totales: un escrow está o abierto (fondeado, esperando) o liberado (condición cumplida, lamports idos al jugador, registro cerrado). Cada reclamo rechazado lo deja abierto, sin cambios. No hay un tercer estado donde el dinero esté a medio mover, porque la liberación es una sola instrucción y las guardas le hacen de barrera.

![Un escrow está o ABIERTO y fondeado en el vault o LIBERADO y cerrado, y solo el que llama correcto con un puntaje que pasa hace esa transición.](assets/v07-diagram.png)

### Paso 5: demuéstralo con una prueba de LiteSVM

La barrera es una prueba de LiteSVM, que es la plantilla de prueba de Rust default de V2. Agrégala con la misma única dev-dependency que usó m04-l1 — `anchor-v2-testing` en `tag = "v2.0.0-rc.1"`, que trae litesvm 0.11.0 y re-exporta todo lo que el archivo de prueba necesita, así que nunca nombras litesvm tú mismo — y levanta los dos programas en proceso. La prueba carga R2 y R3, fondea un operador y un jugador, reserva un premio de 0.05 SOL detrás de un puntaje de 5000, y después prueba tres redeems: uno con el que llama equivocado, uno prematuro con puntaje bajo, y el de verdad.

```rust
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{
    Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction,
};
// The vault's generated module rides inside the escrow crate now, so the test
// borrows its ID from there instead of naming a quarter-vault dependency.
use quarter_prize::quarter_vault;

const PRIZE: u64 = 50_000_000; // 0.05 SOL
const WIN: u64 = 5_000;

fn escrow_pda(maker: &Address, player: &Address) -> (Address, u8) {
    Address::find_program_address(
        &[b"escrow", maker.as_ref(), player.as_ref()],
        &quarter_prize::ID,
    )
}

// One place that signs an instruction into a sendable transaction.
fn tx(svm: &LiteSVM, payer: &Keypair, instruction: Instruction) -> VersionedTransaction {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap()
}

#[test]
fn conditional_release() {
    let mut svm = anchor_v2_testing::svm();
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    let prize_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_prize.so");
    svm.add_program_from_file(quarter_vault::ID, vault_so).unwrap();
    svm.add_program_from_file(quarter_prize::ID, prize_so).unwrap();

    let maker = Keypair::new();
    let player = Keypair::new();
    let stranger = Keypair::new();
    for kp in [&maker, &player, &stranger] {
        svm.airdrop(&kp.pubkey(), 1_000_000_000).unwrap();
    }

    let (escrow, _b) = escrow_pda(&maker.pubkey(), &player.pubkey());
    // R2 derives its vault pair from the owner, which here is the escrow PDA.
    let (vault_state, _sb) = Address::find_program_address(
        &[b"vault", escrow.as_ref()], &quarter_vault::ID,
    );
    let (vault_sol, _lb) = Address::find_program_address(
        &[b"sol", escrow.as_ref()], &quarter_vault::ID,
    );

    // reserve: operator funds the prize into the vault via CPI
    let reserve_ix = Instruction {
        program_id: quarter_prize::ID,
        accounts: quarter_prize::accounts::Reserve {
            escrow,
            maker: maker.pubkey(),
            player: player.pubkey(),
            vault_state,
            vault_sol,
            quarter_vault_program: quarter_vault::ID,
            system_program: System::id(),
        }.to_account_metas(None),
        data: quarter_prize::instruction::Reserve { amount: PRIZE, winning_score: WIN }.data(),
    };
    let reserve_tx = tx(&svm, &maker, reserve_ix);
    svm.send_transaction(reserve_tx).unwrap();
    // the lamports live in the zero-data SOL vault, never in the escrow record
    assert!(svm.get_account(&vault_sol).unwrap().lamports >= PRIZE);
    assert!(svm.get_account(&escrow).unwrap().lamports < PRIZE);

    let redeem = |caller: &Keypair, score: u64| -> Instruction {
        Instruction {
            program_id: quarter_prize::ID,
            accounts: quarter_prize::accounts::Redeem {
                escrow,
                vault_state,
                vault_sol,
                player: caller.pubkey(),
                maker: maker.pubkey(),
                quarter_vault_program: quarter_vault::ID,
                system_program: System::id(),
            }.to_account_metas(None),
            data: quarter_prize::instruction::Redeem { final_score: score }.data(),
        }
    };

    // wrong caller: the address = escrow.player constraint rejects it
    let bad = tx(&svm, &stranger, redeem(&stranger, 9_999));
    assert!(svm.send_transaction(bad).is_err());

    // premature: right player, score below the bar -> ConditionNotMet
    let early = tx(&svm, &player, redeem(&player, 4_200));
    assert!(svm.send_transaction(early).is_err());

    // the real thing: right player, score clears the bar
    let before = svm.get_account(&player.pubkey()).unwrap().lamports;
    let good = tx(&svm, &player, redeem(&player, 5_200));
    svm.send_transaction(good).unwrap();
    let after = svm.get_account(&player.pubkey()).unwrap().lamports;
    assert!(after > before, "the prize should have landed with the player");
}
```

Córrela. Esta suite es la única prueba de LiteSVM en proceso de arriba, y eso importa para lo que hace `anchor test`: para la plantilla de LiteSVM la RC se salta levantar un validador local por completo, exactamente como dijeron m01-l2 y m01-l3 que pasaría, así que nada de acá necesita Surfpool instalado. (`anchor test` sobre una plantilla respaldada por validador es donde entra Surfpool, y m09-l3 lo instala ahí para la corrida de integración del capstone.) Espera:

```text
running 1 test
test conditional_release ... ok

test result: ok. 1 passed; 0 failed
```

Checkpoint, y este es el que importa: el que llama equivocado falla, el reclamo prematuro falla, y solo la superación de verdad mueve los lamports. Si el redeem del extraño *tiene éxito*, te falta el constraint de quien llama. Si el redeem de 4,200 tiene éxito, te falta el `require!` o está sentado después de la CPI. Esas dos fallas son la lección entera, atrapada por la prueba.

## Challenge: una segunda forma de ganar

Demostraste un camino de liberación. Ahora suéltate. Un premio de barcade real no siempre es "supera el puntaje máximo". A veces es "el operador dice que ganaste" (una anulación manual para una final de torneo, juzgada offline). Agrega a R3 una segunda condición de liberación, independiente, y demuestra los dos caminos en la prueba.

La forma te toca elegirla, pero el criterio de aceptación es fijo: agrega un campo al escrow (un flag `won` que el operador pueda fijar es el obvio), una instrucción chica o un argumento para que el operador lo fije, y un camino de redeem que libere con *cualquiera* de los dos, el criterio del puntaje o el flag. Demuestra todo: el camino del flag libera cuando está fijado y se niega cuando no, el camino del puntaje sigue funcionando, y ninguno de los dos caminos deja pasar al que llama equivocado. Cuando las dos condiciones pueden abrir la misma custodia de forma independiente y segura, construiste composición real, no un demo.

Una salvedad honesta antes de que corras. El `final_score` de esta lección lo auto-reporta el jugador, que está bien para una prueba y está mal para producción. Una máquina arcade real tendría el puntaje *atestiguado*, firmado por el programa contador que construiste en el módulo 2 (R1), así que el jugador no puede nada más pasar 9,999. Esa atestación es un problema de composición también, y está deliberadamente fuera de alcance acá: esta lección es sobre la *mecánica* de la liberación, no sobre el oráculo. Nombra ese hueco en tu propio código con un comentario para que el próximo lector sepa que es una elección, no un descuido.

## ¿Funcionó?

Ahora deberías tener un programa `quarter-prize` que nunca toca los lamports del premio él mismo. Los reserva dentro de una instancia real del quarter-vault a través de una CPI, y los libera solo cuando se cumple la condición y solo hacia el que llama que él mismo nombró, con una prueba de LiteSVM que pasa y demuestra que un reclamo del que llama equivocado y un reclamo prematuro fallan los dos. La única línea que reemplazó a `has_one` es `address = escrow.player`, y la guarda que le hace de barrera al pago corre antes de la CPI de withdraw, por posición.

Si te trabaste, los dos culpables de siempre son los que atrapa la prueba: un constraint `address` que falta (le pagan al extraño) o un `require!` en el lugar equivocado (le pagan al puntaje bajo). Los dos son arreglos de una línea, y los dos son exactamente por qué verificamos antes de pagar.

La lección que viene, el suelo se mueve debajo de todo esto. El escrow y el vault hoy mueven lamports los dos, SOL en crudo, custodiado a mano. Los Quarters están por volverse tokens SPL de verdad, y acá está la buena noticia que te compró el modelo de borrow: cada línea de custodia que escribiste (la CPI de depósito, la CPI de liberación, el balance que sigues) cambia en exactamente un lugar, porque compusiste sobre el vault en vez de copiarlo. Construye sobre estado, y pagas por un cambio una sola vez.
