# Lo que todavía te muerde (explota y después parchea)

Acabas de ver morir en tiempo de compilación a type cosplay, a duplicate-mutable y al aliasing de CpiHandle: tres ataques que no llegaban a compilar. El cuarto, el recálculo de bump, compiló limpio y después no hizo nada, porque el framework vuelve a derivar el bump canónico durante la validación y firma con ese, así que tu byte recomputado no tenía adónde ir. Tres rechazados, uno hueco. El compilador era tu guardaespaldas, e hizo el trabajo. Esta lección el guardaespaldas se va a su casa.

Así que todavía no leas. Vas a hacer la rama vulnerable tú mismo, a partir del escrow que ya construiste, porque las guardas que estás por sacar son guardas que escribiste *tú* y sacarlas es lo primero que vale la pena sentir.

Desde el mismo workspace `quarter-vault` de la lección pasada, sobre tu R3/R4 limpio:

```bash
git checkout -b vuln/prize-escrow
```

Después abre `programs/quarter-prize/src/lib.rs` y sácale tres pins al struct de cuentas `Redeem` — el de SPL que terminaste en m05-l1, no el borrador de lamports que lo precedió. Borra el `address = escrow.player` de `player`, borra el `address = escrow.maker` de `maker`, y reduce `vault` a un `#[account(mut)]` pelado sacándole toda su derivación `seeds` / `bump` / `seeds::program`. Deja todo lo demás, los dos constraints de ATA incluidos. Eso es un primer borrador apurado, y es como se entregó la mitad de los escrows de esta cadena.

El registro `Escrow` en sí no cambió desde m04-l3, y los exploits de abajo lo leen, así que mantenlo a la vista:

```rust
#[account]
#[repr(C)]
#[derive(InitSpace)]
pub struct Escrow {
    pub maker: Address,      // the operator who funded the prize
    pub player: Address,     // the only caller allowed to redeem
    pub vault: Address,      // the R2 vault ledger holding this prize
    pub amount: u64,         // prize size
    pub winning_score: u64,  // the bar the player must clear
    pub bump: u8,
    pub _pad: [u8; 7],
}
```

Ahora escribe `drain_as_stranger` (impreso completo abajo, en la clase 1 — todo menos su helper de montaje `reserve_prize`, que el código marca como tuyo para escribir en el mismo archivo) dentro de `programs/quarter-prize/tests/exploits.rs` y córrelo:

```bash
anchor build && cargo test --test exploits drain_as_stranger
```

```text
running 1 test
test drain_as_stranger ... ok

test result: ok. 1 passed; 0 failed
```

Lee ese resultado por lo que es. Un extraño, no el jugador que el escrow nombró, se acaba de ir con el premio entero, y la prueba que lo demuestra dice `ok`. Nada en ese programa es un error de tipos. Compiló limpio sobre el toolchain de V2 que mató tres clases de ataque la lección pasada. Se entrega. Y es drenable. Esa brecha, entre "compila" y "seguro," es la lección entera, y vas a atacarla con exploits que funcionan antes de cerrarla.

## Lo que esta lección resuelve

La lección pasada respondió "qué mata V2 gratis." Esta responde la pregunta más difícil: qué espera V2 que escribas todavía, y qué no puede escribir por ti ningún compilador de ningún framework. La taxonomía de seguridad de programas de la Solana Foundation tiene dos mitades. Ya retiraste la mitad que se volvió errores de compilación. Acá está la mitad que sobrevive, corrida contra tu propio escrow (R3) y swap (R4). Tres de ellas las aterrizas como exploits que funcionan y después las parcheas: la verificación de firmante/dueño, la sustitución de cuentas y el underflow aritmético. Las otras cuatro aprendes a *detectarlas*, porque cada una o ya está cerrada por un constraint que el escrow usa por casualidad o necesita un programa con otra forma para demostrarse. Cuáles cuatro son cuáles vale la pena notarlo mientras lees; el Lab explota exactamente las tres:

1. **Verificaciones de firmante y de dueño.** V2 todavía espera que afirmes quién tiene permitido actuar. Sáltate la afirmación y cualquiera actúa como la autoridad.
2. **Sustitución de UncheckedAccount.** `UncheckedAccount` hace opt-out de toda verificación del framework por su propio nombre, así que tiene que venir acompañado de un `address`, un `owner` o un `constraint` explícito. Sin uno, un atacante sustituye su propia cuenta. Esta es la clase de sustitución de cuentas, y es la que sobrevive a todos los frameworks jamás escritos.
3. **La trampa del error de dueño.** `#[account(owner = X @ MyErr)]` sobre un `Account<T>` no hace aflorar `MyErr`. Hace aflorar `ProgramError::IllegalOwner`, porque las verificaciones de dueño y de discriminador corren adentro de `load`, antes de tus hooks de constraint.
4. **Reúso de init_if_needed.** V2 entrega `init_if_needed` sin feature gate y valida espacio, dueño y discriminador en el reúso, pero un bug de lógica de reinicializar-sobre-estado-vivo todavía lo sobrevive.
5. **CPI arbitraria.** Invoca un programa que te entregó quien llama e invocaste lo que ellos quisieran. Valida el id del programa target.
6. **Cerrar-y-revivir.** Una cuenta cerrada se puede revivir adentro de la misma transacción a menos que la pongas en ceros y le pongas una guarda.
7. **Overflow aritmético.** La resta sin signo hace underflow. Usa matemática verificada.

La ayuda se repliega como siempre. Yo recorro la clase de firmante/dueño de punta a punta, exploit y parche, porque quiero el drenaje en tu salida de pruebas y el arreglo en tus dedos una vez. La sustitución de UncheckedAccount baja a un problema Completion: yo te entrego el parche, tú escribes el exploit que demuestra que el agujero era real. La guarda de withdraw es el último peldaño, un Challenge de código en Rust puro despojado de Anchor por completo, donde recibes la firma y la convención de retorno y nada más. Aterriza el exploit, cierra el agujero, demuestra que se queda cerrado.

![Una comparación de dos columnas que pone las tres clases retiradas en tiempo de compilación, más la clase de bump que compila pero no tiene ninguna costura, al lado de las siete clases que sobreviven y que esta lección tiene que parchear a mano.](assets/v01-comparison.webp)

Una trampa para nombrar antes de empezar, porque es toda la razón por la que esta mitad es peligrosa. Cada guarda que agregas cuesta cómputo y código que tienes que mantener, y el framework nunca te va a decir qué guarda *falta*. Solo rechaza las formas de escribirlo que ya conoce. Así que "secure by default" es una afirmación sobre las clases de arriba de esa tabla, no las de abajo. Confiar de más en eso sobre la mitad de abajo es exactamente cómo se drenan los escrows.

## Las clases que sobreviven, un ataque a la vez

El método es fijo y es el punto: para cada clase, aterriza un exploit que tenga éxito contra el código actual, nombra con precisión por qué funciona, y después parchea hasta que el exploit falle y el camino legítimo siga pasando. Lee, rompe, arregla, demuestra.

![Un diagrama de loop de siete pasos: elige una clase, escribe un exploit que pase, nombra la guarda que falta, agrégala, y vuelve a correr hasta que el exploit falle.](assets/v02-flowchart.webp)

### Clase 1: la verificación de firmante y de dueño (trabajada completa)

Empieza con el exploit que ya corriste. Acá está el struct de cuentas de redeem sobre la rama `vuln`, con las guardas sacadas hasta donde las deja un primer borrador apurado:

```rust
#[derive(Accounts)]
pub struct Redeem {
    pub player: Signer,                    // VULN: any signer, not the recorded player

    #[account(
        mut,
        close = maker,
        seeds = [b"escrow", escrow.maker.as_ref(), escrow.player.as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<Escrow>,

    /// CHECK: pinned by the address constraint the patch adds below
    #[account(mut)]                        // VULN: any account, substitutable rent recipient
    pub maker: UncheckedAccount,

    /// CHECK: pinned by the constraint the patch adds below
    #[account(mut)]                        // VULN: any vault, not the one the escrow recorded
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

Lee las dos líneas de ATA antes de seguir, porque son lo que hace que el exploit *pague*. `winner_token_account` se deriva de quien sea que se siente en el lugar de `player`. Sustituye a quien llama y sustituyes el destino del pago junto con él — una edición, las dos mitades del robo.

Nada acá es un error de compilación. `player` es un `Signer` de verdad, así que *alguien* firmó. Pero el programa nunca verifica que ese alguien sea `escrow.player`. Toda la liberación condicional gira sobre "solo el jugador nombrado puede reclamar," y esa oración no aparece en ninguna parte del código. El exploit se escribe solo. Un extraño firma un redeem, supera la barra de puntaje (autoreportado, acuérdate, este cabinet todavía no atestigua puntajes), y el vault le paga. Las pruebas de exploit corren sobre LiteSVM, el banco de pruebas de Rust en proceso default de V2, así que agrégalo al programa si todavía no está:

```bash
# Reach LiteSVM through the harness, never by name. anchor-v2-testing owns the litesvm
# version (0.11.0 at tag v2.0.0-rc.1; the anchor-next head has already moved it to 0.13.1),
# so pinning the tag pins the SVM too. Add litesvm yourself at another version and the
# mismatch surfaces as a baffling compile error — two versions of one crate, identical-looking
# types refusing to unify — instead of never happening at all.
cargo add anchor-v2-testing --dev \
  --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1
# The escrow crate needs the same two SPL client crates the vault's fixture used in
# m05-l1, at the same pins, because the prize is tokens now and the setup has to mint
# and move them.
cargo add spl-token@9 spl-associated-token-account@8 --dev
```

```rust
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{
    Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction,
};

// The SPL client helpers m05-l1 shipped, reached by path instead of copied.
// Each tests/*.rs is its own crate root, so you cannot `use` an item out of a
// sibling test binary — but `#[path]` compiles any file you point it at straight
// into THIS crate, and it can point across packages. Same trick m05-l1's
// spl_setup.rs used to reach spl_helpers.
#[path = "../../quarter-vault/tests/spl_helpers.rs"]
mod spl_helpers;

const ONE_TOKEN: u64 = 1_000_000;   // 6 decimals, same mint shape as m05-l1

// The offset-64 read from m05-l1's spl_setup, restated here because that file
// belongs to the vault's crate: a token account's `amount` is a little-endian
// u64 at byte 64. (spl_helpers above builds instructions; it reads nothing.)
fn token_balance(svm: &LiteSVM, ata: &Address) -> u64 {
    let acct = svm.get_account(ata).expect("token account exists");
    u64::from_le_bytes(acct.data[64..72].try_into().unwrap())
}

#[test]
fn drain_as_stranger() {
    let mut svm = anchor_v2_testing::svm();
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    let prize_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_prize.so");
    svm.add_program_from_file(quarter_vault::ID, vault_so).unwrap();
    svm.add_program_from_file(quarter_prize::ID, prize_so).unwrap();

    let (maker, player, stranger) = (Keypair::new(), Keypair::new(), Keypair::new());
    for kp in [&maker, &player, &stranger] {
        svm.airdrop(&kp.pubkey(), 1_000_000_000).unwrap();
    }

    let (escrow, _b) = Address::find_program_address(
        &[b"escrow", maker.pubkey().as_ref(), player.pubkey().as_ref()],
        &quarter_prize::ID,
    );
    let (vault, _vb) =
        Address::find_program_address(&[b"vault", escrow.as_ref()], &quarter_vault::ID);

    // Setup seam, and it is YOURS to write in this same file: adapt the SPL reserve
    // flow you built for m05-l1's solo into a local `fn reserve_prize`. It creates a
    // 6-decimal mint, gives the maker, the player and the stranger each an ATA on it,
    // then reserves a 1.0-token prize behind a 5_000 winning score — which means the
    // two-call flow into the escrow's own vault instance, `initialize` then `deposit`.
    // Have it hand back the mint and the ATA addresses so the assertion can read them.
    let f = reserve_prize(&mut svm, &maker, &player, &stranger, escrow, vault, ONE_TOKEN, 5_000);

    // The stranger, NOT escrow.player, redeems with a passing score — and because
    // winner_token_account derives from the player seat, the payout follows them.
    let ix = Instruction {
        program_id: quarter_prize::ID,
        accounts: quarter_prize::accounts::Redeem {
            player: stranger.pubkey(),     // substitute the caller
            escrow,
            maker: maker.pubkey(),
            vault,
            mint: f.mint,
            vault_token_account: f.vault_ata,
            winner_token_account: f.stranger_ata,
            token_program: spl_token::ID,
            quarter_vault_program: quarter_vault::ID,
            system_program: System::id(),
        }
        .to_account_metas(None),
        data: quarter_prize::instruction::Redeem { final_score: 9_999 }.data(),
    };
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&stranger.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&stranger]).unwrap();

    // On the vuln branch this SUCCEEDS. That is the bug.
    svm.send_transaction(tx).unwrap();
    assert_eq!(
        token_balance(&svm, &f.stranger_ata),
        ONE_TOKEN,
        "the stranger walked off with the whole prize"
    );
}
```

Eso es un drenaje que funciona, y el parche es una línea. Sobre el campo `player`, fija a quien llama a la pubkey que el escrow registró:

```rust
#[account(address = escrow.player)]
pub player: Signer,
```

`address = escrow.player` dice, sin rodeos, que la cuenta que se pasa acá tiene que ser igual al jugador que este escrow nombró. Es la forma idiomática de V2 que reemplazó a `has_one`, y toma cualquier expresión, así que se lee como lo que hace. Vuelve a correr `drain_as_stranger` y se da vuelta: la transacción ahora falla en la carga de cuentas, antes de que tu handler corra una sola línea, porque la clave del extraño no coincide con `escrow.player`. Vuelve a correr la prueba legítima `conditional_release` y sigue verde. Exploit muerto, feature intacto. Ese es el loop entero, y cada clase de abajo es una variación de él.

![Una tarjeta de código anotada que muestra el campo player sin fijar que un extraño puede drenar, y el constraint address que rechaza al extraño en la carga de cuentas.](assets/v03-annotated-code.webp)

### Clase 2: sustitución de UncheckedAccount (la que sobrevive a todos los frameworks)

Vuelve a mirar ese campo `maker`. Es un `UncheckedAccount`, y sobre la rama vuln lleva solo `mut`. Esa palabra `UncheckedAccount` es el tipo haciendo opt-out de toda verificación del framework que exista: ninguna verificación de dueño, ninguna de discriminador, ninguna de identidad. Le estás diciendo a Anchor "yo voy a validar esto por mi cuenta," y después no lo haces.

Acá está por qué esa es la clase más profunda de la lección. El escrow cierra hacia `maker`, devolviendo el rent. Sobre la rama vuln, `maker` es cualquier cuenta que pase quien llama. Así que un atacante pasa su *propia* cuenta como `maker`, y los lamports de rent de `close = maker` aterrizan en su billetera en vez de la del operador. Plata chica en un escrow, plata de verdad a lo largo de mil. Y nada lo atrapa, porque no hay nada que atrapar: la cuenta es válida, es mutable, es escribible. Simplemente no es la cuenta que el escrow quería decir.

Esta es la clase de sustitución de cuentas, y quiero que te quedes un rato con una afirmación: ningún compilador de ningún framework, en ningún lenguaje, puede atrapar esta por ti. Type cosplay era atrapable porque los tipos diferían. Duplicate-mutable era atrapable porque las direcciones hacían alias. La sustitución no tiene ninguna señal. Una cuenta correcta y la cuenta de un atacante tienen tipos idénticos, dueños idénticos en el caso general, todo idéntico excepto *cuál de las dos quería la lógica de negocio*, y la intención no está en el sistema de tipos. Es la única clase de la que eres dueño para siempre.

![Una comparación lado a lado de un UncheckedAccount que lleva solo mut contra uno fijado por address, owner o constraint, mostrando qué sustituciones permite cada uno.](assets/v04-comparison.webp)

El parche sobre el escrow restaura el pin que la versión congelada siempre tuvo:

```rust
#[account(mut, address = escrow.maker)]
pub maker: UncheckedAccount,
```

Ahora el rent solo puede volver al maker registrado. Haz lo mismo con `vault`, que sobre la rama vuln lleva solo `mut` — cualquier cuenta, la que sea — así que un atacante sustituye un vault *distinto* que él controla, y la línea `associated_token::authority = vault` amablemente deriva el ATA del vault desde *su* vault. Una cosa que el parche *no* tiene que hacer es promover el campo a un `Account<Vault>` tipado: el vault pertenece a `quarter_vault`, un programa distinto, así que nunca podría cargarse como una cuenta tipada adentro de `quarter_prize` — la clase 3 de abajo deriva exactamente por qué, y es la razón por la que R3 declaró el slot `UncheckedAccount` en primer lugar. Dos formas de escribirlo lo cierran, y las dos ya están en tu vocabulario. El escrow congelado lo fija por derivación, `seeds` + `bump` + `seeds::program`, que es lo que escribiste en m05-l1. La más corta lo fija contra el registro, con un error de verdad ya que está, y es la que usa esta lección porque hace visible la clase de sustitución en una sola línea:

```rust
/// CHECK: pinned to the exact vault this escrow recorded
#[account(mut, address = escrow.vault @ EscrowError::WrongVault)]
pub vault: UncheckedAccount,
```

**Problema Completion.** Te di los dos parches de arriba. Ahora escribes el exploit que demuestra que el agujero de `maker` era real. Bifurca `drain_as_stranger` en una prueba `steal_rent_on_close`: el jugador legítimo hace el redeem correctamente, pero pasa `maker: attacker.pubkey()` en vez del maker verdadero, y afirmas que el balance del atacante creció en más o menos el rent del escrow. Aterrízala en rojo sobre el campo con la guarda sacada, aplica el pin `address = escrow.maker`, mírala ponerse verde. El criterio de aceptación es exactamente el loop: la prueba pasa contra el campo vuln y falla contra el parche.

Un puntero, porque deberías saber dónde vive la versión insignia de esta clase: el drenaje de Cashio, donde una verificación `.mint` que faltaba dejó a un atacante acuñar colateral de la nada, es territorio del curso de DeFi y RWA Engineering. Este curso no lo desarrolla; ve allá por la historia de guerra. Acá, hazte dueño de la clase.

### Clase 3: la trampa del error de dueño (por qué la verificación que esperabas no dispara)

Una trampa que parece un arreglo: digamos que quieres un error personalizado cuando alguien pasa un vault del que es dueño el programa equivocado. La forma natural de escribirlo es:

```rust
#[account(owner = quarter_vault::ID @ EscrowError::WrongVault)]
pub vault: Account<Vault>,
```

Escribes tu prueba, pasas un vault del que es dueño algún otro programa, y esperas `WrongVault`. En vez de eso recibes `ProgramError::IllegalOwner`. Tu `@ EscrowError::WrongVault` nunca disparó. ¿Por qué?

(Hay una segunda razón por la que esa forma de escribirlo está mal para *este* campo en particular, y vale la pena detectarla antes que el mecanismo: el vault del escrow pertenece a `quarter_vault`, un programa distinto, así que nunca podría cargarse como un `Account<Vault>` tipado adentro de `quarter_prize`, para nada. Por eso R3 lo declara `UncheckedAccount`. La trampa de abajo es la que muerde cuando la cuenta genuinamente es tuya y lo único que querías era un mensaje más lindo.)

Esto vale derivarlo, no solo memorizarlo, porque la razón generaliza. Un `Account<T>` es un wrapper tipado, y antes de que corran tus hooks de constraint, Anchor tiene que *cargarlo*: leer sus bytes, verificar que el programa declarante sea dueño de la cuenta, y verificar que el discriminador coincida con `T`. Esas dos verificaciones, dueño y discriminador, son estructurales. Pasan adentro de `load`, primero, porque el framework no te puede entregar un `T` tipado que no verificó que sea un `T`. Tu constraint `owner = ... @ MyErr` es un *hook*, y los hooks corren después de la carga. Así que sobre una cuenta con el dueño equivocado, la carga falla primero con `IllegalOwner`, y el control nunca llega al hook que lleva tu error personalizado.

![Un diagrama del orden de verificación para Account<T>, con dueño y discriminador corriendo adentro de load antes de cualquier hook de constraint, así que IllegalOwner le gana al error personalizado.](assets/v05-diagram.webp)

El arreglo, cuando genuinamente necesitas el error personalizado, es dejar de pedirle a `Account<T>` que lo lleve. Toma un `UncheckedAccount`, que no hace ninguna verificación de dueño en tiempo de carga (exactamente el tipo de la clase 2), y ponle encima el *mismo* constraint `owner = X @ MyErr`. Ahora no hay ningún `load` que cortocircuitar, así que el hook de constraint es lo único que verifica al dueño, y lleva tu error:

```rust
#[account(owner = quarter_vault::ID @ EscrowError::WrongVault)]
pub vault: UncheckedAccount,
```

Esa es la prescripción del propio framework: el comentario de doc sobre el alias `Account<T>` de V2 dice, con todas las letras, que para un error personalizado usas `UncheckedAccount` con un `owner = X @ MyErr` a nivel de derive. El canje es que entregaste la vista tipada, así que si el handler necesita los campos del vault ahora los cargas y los validas tú mismo.

Fíjate en la forma: las dos clases se componen. El tipo que hace opt-out de las verificaciones del framework es el mismo tipo que te deja escribir las tuyas. Eso no es una coincidencia, es el framework diciéndote qué hace y qué no hace por ti.

### Clase 4: reúso de init_if_needed (sin feature gate no es lo mismo que seguro)

Un compañero se acuerda de `init_if_needed` de la línea de v1 como el que tenía feature gate, el que tenías que habilitar explícitamente porque era peligroso. Corrige el registro, porque V2 lo cambió y acordarse a medias del cambio es su propio riesgo.

En V2, `init_if_needed` ya no está detrás de un feature flag. Verifiqué el conjunto de features de V2 contra los propios docs del framework: la release entrega seis feature flags, `alloc`, `guardrails`, `idl-build`, `compat`, `const-rent` y `testing`, y `init-if-needed` no está entre ellos, así que no tiene feature gate. Encima de eso, las cuentas de `init_if_needed` están plegadas dentro de la verificación de duplicate-mutable desde la línea 1.0 (#4239) y lo siguen estando bajo V2, y la rama de reúso vuelve a validar el espacio, el dueño y el discriminador de la cuenta, lo que cierra los trucos de reinicialización más crudos.

La parte que sobrevive a todo eso: El framework puede verificar que la cuenta tenga la *forma* correcta. No puede verificar que reinicializar *esta* cuenta sea *lo correcto que hacer*. Si tu instrucción cae en la rama `init` sobre una cuenta que ya sostiene estado vivo, la validación de reúso pasa (el espacio coincide, el dueño coincide, el discriminador coincide) y alegremente sobrescribes un escrow fondeado de vuelta a ceros.

![Una tabla que parte la validación de reúso de init_if_needed en las verificaciones estructurales que V2 hace y la intención de negocio que no puede juzgar, donde sobrevive un bug de reinit.](assets/v06-table.webp)

Nota de frescura: esto refleja la release candidate de Anchor V2 al 2026-08-22, verificada contra los docs de feature flags y el changelog del propio framework. V2 sigue siendo una RC sin tag estable, así que si fijas una RC más nueva, vuelve a leer su comportamiento de validación de reúso de `init_if_needed` antes de confiar en la semántica exacta. La mitigación no cambia: si una cuenta puede sostener estado vivo, ponle tú mismo una guarda al reinit. Verifica una flag guardada o un campo distinto de cero antes de dejar que corra la rama `init`, y rechaza cuando la cuenta ya esté viva.

### Clase 5: CPI arbitraria (valida el id del programa target)

Tu escrow llama al vault. Declara al callee como `quarter_vault_program: Program<QuarterVault>`, y ese `Program<T>` tipado está haciendo trabajo de seguridad callado: verifica que la clave de la cuenta sea igual a `QuarterVault::id()` — el marcador que `declare_program!` generó allá en m04-l3 — y, con el feature `guardrails` default prendido, que la cuenta sea ejecutable. No te pueden engañar para que llames otra cosa, porque el tipo fija el target.

Ahora imagina que te ganó la pereza y lo tipaste como un `AccountInfo` o un `UncheckedAccount`, y después lo invocaste:

```rust
// VULN: the callee is whatever the caller passed.
let cpi_ctx = CpiContext::new(ctx.accounts.some_program.address(), cpi_accounts);
// ... invoke ...
```

Un atacante pasa su propio programa como `some_program`, tu escrow firma la CPI con las seeds del PDA del escrow, y ahora el programa del atacante corre *con la firma de tu PDA*. Esa es la clase de CPI arbitraria: no elegiste el programa que corrió, lo eligió quien llama, y le entregaste tu autoridad.

El parche es dejar que el tipo fije el target, exactamente como hace el escrow congelado. Para el swap, la misma regla aplica al token program: `token_program: Interface<'static, TokenInterface>` fija al callee a un token program de verdad (clásico o Token-2022) en vez de aceptar uno arbitrario.

![Un diagrama de flujo que contrasta una cuenta de programa sin tipar que un atacante puede elegir con un Program tipado que fija al callee al id del programa del vault.](assets/v07-flowchart.webp)

### Clase 6: cerrar-y-revivir (ponla en ceros, o ponle una guarda)

Cerrar una cuenta no es solo sacarle los lamports. En Solana, una cuenta con cero lamports al final de una transacción se barre, pero *adentro* de la transacción, una cuenta que drenaste y encogiste se puede volver a llenar y reusar antes del barrido. Si tu cierre está hecho a mano, transferir los lamports, hacer realloc a nada, y parar, dejaste la puerta abierta: una instrucción posterior en la misma transacción la refondea y el discriminador de tu programa sigue sentado en los datos (ahora revividos), así que la cuenta pasa la validación otra vez en una segunda llamada. Eso es cerrar-y-revivir.

El escrow lo evita porque usa el constraint `close = maker`, y el close de V2 pone los datos en ceros, escribe el centinela de cuenta cerrada, y asigna la cuenta al system program. No queda nada que revivir. La clase solo muerde cuando alguien pasa por encima del constraint y cierra a mano. Si alguna vez lo haces, la regla es: pon el discriminador en ceros y ponle una guarda contra la forma revivida, no muevas solamente los lamports.

![Una línea de tiempo de una sola transacción donde un cierre de solo lamports deja el discriminador intacto y la cuenta se revive, al lado de un cierre que pone en ceros y bloquea la revivida.](assets/v08-timeline.webp)

### Clase 7: overflow aritmético (el bug de verdad de la guarda de withdraw)

La última clase es la más chica de enunciar y la más fácil de entregar. El `withdraw` del vault debita su libro mayor, `vault.credit`, después del movimiento de tokens. Sobre la rama vuln lo hace con resta cruda:

```rust
// VULN: unsigned subtraction underflows.
vault.credit = vault.credit - amount;
```

Si `amount > credit`, esto no da error. En builds de debug entra en panic; en un build con las verificaciones de overflow apagadas hace *wrap*, así que un credit de 30 menos un withdraw de 100 se vuelve un número positivo gigantesco y los libros del vault creen que sostiene mucho más de lo que sostiene. Ninguno de los dos resultados es "el withdraw fue rechazado," que es el único correcto.

Llegar a esa línea toma una jugada extra, y la jugada es la parte interesante. `withdraw` transfiere los tokens primero y debita el libro mayor segundo, así que mientras el libro mayor y la custodia coinciden, el token program rechaza un over-withdraw en la CPI de transferencia y la resta nunca corre — m05-l1 dijo exactamente eso cuando te hizo afirmar que el over-withdraw vuelve como un `Err`. La guarda del libro mayor existe para el caso en que los dos números *no coinciden*, y dejan de coincidir en el momento en que cualquiera transfiere tokens directo al ATA del vault. Una cuenta de token asociada es un buzón público: `deposit` no es la única manera de que entren tokens, y nada on-chain hace que una transferencia no solicitada suba `vault.credit`. Así que sobre-fondea el ATA del vault a mano, y después haz un withdraw por más que `credit` pero no más de lo que el ATA de verdad sostiene. La transferencia tiene éxito, la resta cruda hace wrap, y los libros ahora reportan un balance que nadie puso ahí nunca.

Ten claro dónde está parado tu build en eso, porque decide cuál de los dos te toca. El `Cargo.toml` de workspace que genera Anchor pone `overflow-checks = true` sobre el perfil de release, y `cargo build-sbf` usa release, así que sobre un scaffold sin tocar esto entra en panic en vez de hacer wrap. Dos cosas vuelven real el wrap igual. Alguien saca esa línea, que es lo que pasa la primera vez que un equipo persigue CU. O alguien apaga `guardrails` — el flip que corriste tú mismo en m06-l2, que aterrizó en nada solo porque el borde de anchor-spl mantuvo el feature prendido; sobre un crate sin ese borde, o después de un cambio en el grafo, aterriza. De cualquiera de las dos formas el wrap está a una edición de Cargo de distancia, y una guarda que solo se sostiene por un ajuste de perfil no es una guarda.

El control de acceso no te salva acá. Un jugador perfectamente autorizado puede igual pedir más de lo que el vault sostiene. Este no es un bug de "quién", es un bug de "cuánto", y el arreglo es aritmética verificada:

```rust
// PATCH: checked_sub returns None exactly when amount > credit.
vault.credit = vault
    .credit
    .checked_sub(amount)
    .ok_or(VaultError::Underflow)?;
```

`checked_sub` devuelve `None` exactamente en el caso que haría underflow, así que conviertes ese `None` en un error de verdad y rechazas el over-withdraw. Es un método y un `?`. Es también, y no por casualidad, el parche exacto que te pide tu Challenge de código.

![Una tarjeta de código anotada que compara la resta cruda y checked_sub sobre un withdraw de 100 contra un credit de 30 del libro mayor, una haciendo wrap y la otra rechazando.](assets/v09-annotated-code.webp)

Un dato de mantenimiento para tus parches. Los rechazos de constraint del propio Anchor viven mayormente en los 2000 — `ConstraintAddress`, el que levantan tus pins, es Custom(2012) — pero no todos: un puñado mapea directo sobre los errores builtin del runtime en vez de eso, y `ConstraintOwner` es el caso filoso, aflorando como `ProgramError::IllegalOwner` en vez de cualquier número de los 2000, exactamente como te mostró la clase 3. Tus variantes personalizadas de `#[error_code]` empiezan en 6000 y cuentan hacia arriba. Así que un error en los 6000 es uno tuyo, y *cuál* depende del programa: `quarter_prize` y `quarter_vault` tienen cada uno su propio enum `#[error_code]`, cada uno numerado desde 6000 por orden de declaración, así que 6001 quiere decir una cosa en un rechazo de redeem y otra en un rechazo de withdraw. Lee el programa del que vino el error antes de leer el número. Saber en qué banda vive un error te dice de un vistazo si la transacción la rechazó el framework o tu propia guarda.

### El mismo loop sobre el swap

El escrow era la taxonomía entera sobre un solo programa. El swap (R4) son las mismas clases aplicadas a cuentas de token en vez de vaults de lamports, y correr el loop contra él es lo que te convence de que estas son *clases*, no trivia del escrow. `swap_arcade_for_tickets(amount_in, min_out)` jala los tokens de arcade del trader hacia la reserva de arcade del pool y empuja tickets de vuelta afuera. Dos de las clases que sobreviven mapean directo sobre él.

Primero, sustitución, la clase 2 otra vez. El `reserve_arcade` y el `reserve_ticket` del swap son las cuentas de token propias del pool, aquellas contra las que los canjes cotizan. Llevan `token::mint` y `token::authority = pool`, y ninguna de esas dos dice *cuál* cuenta quería decir el pool: cualquiera puede crear una cuenta de token sobre el mint correcto con el pool como su autoridad, porque el `InitializeAccount` de SPL toma al dueño como un argumento común y nunca le pide al dueño que firme. Así que un atacante pasa su propio par como las reservas, la matemática de producto constante cotiza contra balances que él controla, y se cotizan a sí mismos un fill que el pool de verdad nunca ofrecería. La misma forma que el extraño drenando el escrow: una cuenta válida del tipo correcto, simplemente no la que el programa quería decir.

Ahora la mitad incómoda. El parche es de la misma *forma* que el del escrow — fijar cada reserva contra lo que el pool registró — salvo que R4 como lo construiste no registra nada contra lo que fijar: `Pool` sostiene los dos mints y su bump, y las reservas se sientan en direcciones que nadie deriva. Así que cerrar esta son dos jugadas, no una: el pool tiene que empezar a registrar las dos direcciones de reserva en el init, y el swap tiene que fijar cada campo contra ese registro. Ese es un hallazgo real y abierto contra tu propio programa. No lo parchees acá por corazonada — la lección que viene abre haciéndote buscar exactamente esta línea, y arreglarla es la primera fila de la checklist de auditoría.

Segundo, CPI arbitraria, la clase 5. El swap hace CPI de `transfer_checked` a través de `token_program`, y el swap congelado lo tipa como `Interface<'static, TokenInterface>`, que fija al callee a un token program de verdad (clásico o Token-2022) y a nada más. Típalo como un `UncheckedAccount` en vez de eso y el trader elige qué programa mueve los tokens, con la autoridad del pool detrás de la llamada. El tipo es la guarda.

![Una comparación que mapea cada clase de vulnerabilidad del escrow que sobrevive sobre los campos propios del swap, con la guarda que la cierra nombrada en la columna final.](assets/v10-comparison.webp)

No vuelves a derivar nada para atacar el swap. Llevas las mismas siete preguntas y se las haces a una lista de cuentas distinta. Esa portabilidad es la razón por la que vale la pena aprender la taxonomía como clases en vez de como una checklist para un solo programa.

## Lab: explota y después parchea el escrow

Ya viste cada clase. Ahora corre el loop tú mismo contra el escrow, de punta a punta, hasta que cada prueba de exploit falle y la suite esté verde.

**Paso 1. Fija el toolchain de V2 y ponte sobre la rama vulnerable.** Este curso corre sobre la release candidate de Anchor V2, no sobre la línea V1 1.1.2 que muchas máquinas entregan por defecto. `avm install` no puede traer la RC de V2: descarga un binario preconstruido desde un release publicado de GitHub, y no se cortó ningún release para el tag v2, así que la descarga da 404. Instala el CLI directo desde el tag `v2.0.0-rc.1`:

```bash
# Anchor V2 RC CLI, built from the tag (no release binary for the v2 tag to download).
cargo install --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1 anchor-cli --locked --force
# macOS: prefix with CARGO_PROFILE_RELEASE_LTO=off if the release build fails to link.
anchor --version              # confirm the V2 line, not 1.1.2

# You cut this branch and peeled the three constraints back at the top of the lesson.
git checkout vuln/prize-escrow
```

Nota de frescura: al 2026-08-22 la línea de V2 se entrega solo como release candidates (2.0.0-rc.1, tagueada sobre `anchor-next`), así que no hay ninguna versión estable para hardcodear. La punta de la rama avanza, así que registra el commit exacto con el que construiste en `Anchor.toml` y en CI para que un compañero construya el mismo bytecode. Cuando V2 taguee estable, fija eso en cambio.

**Paso 2. Aterriza los tres exploits.** Tu rama `vuln` tiene `Redeem` con las guardas sacadas: `player` sin `address`, `maker` sin `address`, `vault` sin pin de derivación. Saca la cuarta guarda ahora, en el vault, y esta toma dos ediciones — que es en sí misma la lección: reemplaza el `checked_sub` en `withdraw` por un `vault.credit - amount` crudo, *y* comenta la línea `overflow-checks = true` bajo el perfil de release en el `Cargo.toml` del workspace. La clase 4 te dijo por qué la segunda edición es obligatoria: sobre un scaffold de Anchor sin tocar, los builds de release mantienen prendidas las verificaciones de overflow, así que la resta cruda haría panic-abort de la transacción — un crash, no un drenaje — y `over_withdraw` fallaría por la razón equivocada. El wrap está a una edición de Cargo de distancia, dijo la clase 4; para este ejercicio, tú eres ese alguien que la hace. Después pon las tres pruebas de exploit en `programs/quarter-prize/tests/exploits.rs`: `drain_as_stranger` de la clase 1, `steal_rent_on_close` del problema Completion de la clase 2, y `over_withdraw`, que manda tokens directo al ATA del vault para empujar la custodia por encima del libro mayor, y después hace un withdraw por más que `credit`. Corre las tres y míralas pasar, que es el resultado equivocado y todo el punto:

```bash
anchor build && cargo test --test exploits
```

```text
test drain_as_stranger   ... ok
test steal_rent_on_close ... ok
test over_withdraw       ... ok

test result: ok. 3 passed; 0 failed
```

Tres exploits verdes son tres agujeros reales. `drain_as_stranger` es la clase de firmante/dueño del recorrido. `steal_rent_on_close` es el problema Completion de sustitución de cuentas que escribiste en la clase 2. `over_withdraw` pide más de lo que el *libro mayor* dice que sostiene el vault, respaldado por un ATA que la prueba sobre-fondeó a mano, y — con la edición de perfil desarmando las verificaciones de overflow — la resta cruda hace wrap y lo deja pasar. Checkpoint: las tres reportan `ok`. Si en cambio `steal_rent_on_close` falla, tu prueba está afirmando la cosa equivocada, no demostrando que el agujero está cerrado, así que vuelve a leer el criterio de aceptación de la clase 2 antes de seguir.

**Paso 3. Parchea, una clase a la vez.** Aplica los cuatro constraints y la única operación verificada, exactamente como se derivó arriba:

```rust
// in Redeem accounts:
/// CHECK: pinned to the exact vault this escrow recorded
#[account(mut, address = escrow.vault @ EscrowError::WrongVault)]
pub vault: UncheckedAccount,

#[account(address = escrow.player)]
pub player: Signer,

#[account(mut, address = escrow.maker)]
pub maker: UncheckedAccount,
```

```rust
// in the vault's withdraw handler:
vault.credit = vault
    .credit
    .checked_sub(amount)
    .ok_or(VaultError::Underflow)?;
```

Y mantén la guarda de la condición de victoria por delante de la CPI del pago, donde le hace de barrera a la liberación por posición:

```rust
require!(final_score >= ctx.accounts.escrow.winning_score, EscrowError::ConditionNotMet);
```

Y restaura la línea `overflow-checks = true` del release que comentaste en el paso 2. El `checked_sub` ya no necesita que el perfil lo salve — ese es el punto del parche — pero el perfil es el respaldo del workspace para todas las *demás* restas, y vuelve a estar prendido.

Checkpoint: `anchor build` está verde. Tres constraints, una operación verificada, y la línea de perfil restaurada son todo el conjunto de parches, así que si el build falla es un problema de escritura, no un problema de diseño, y el compilador nombra el campo.

**Paso 4. Demuestra que los exploits están muertos y el feature vive.** Corre los exploits y el camino legítimo juntos:

```bash
anchor build && cargo test
```

```text
test drain_as_stranger    ... FAILED (rejected: ConstraintAddress)
test steal_rent_on_close  ... FAILED (rejected: ConstraintAddress)
test over_withdraw        ... FAILED (rejected: custom error 0x1771)
test conditional_release  ... ok

test result: FAILED. 1 passed; 3 failed
```

Lee ese resultado invertido con cuidado, porque una prueba de exploit que falla es éxito acá. `drain_as_stranger` y `steal_rent_on_close` ahora fallan con `ConstraintAddress`, el rechazo de la banda de los 2000 del framework en la carga de cuentas: quien llama equivocado y el maker sustituido nunca llegan a tu handler. `over_withdraw` falla con un error personalizado en los 6000, sea cual sea el número en el que cayó tu variante `Underflow` dada su posición en `VaultError` (las variantes se numeran desde 6000 por orden de declaración, así que cuenta las tuyas en vez de copiar las mías). El runtime lo imprime en hex, así que una variante en 6001 se muestra como `0x1771`. Y `conditional_release`, el jugador de verdad superando la barra de verdad, sigue pasando. Checkpoint: los tres exploits pasan de pasar a fallar, y la liberación legítima se queda verde. Si algún exploit todavía pasa, el culpable es la guarda que no agregaste todavía, y el nombre de la prueba te dice qué clase.

![Un diagrama de flujo de arriba abajo del redeem con guardas, desde la carga de cuentas fijada por address, pasando por la verificación de victoria, el pago firmado, el débito verificado, y el cierre que pone en ceros.](assets/v11-flowchart.webp)

## Challenge: parchea la guarda de withdraw como una función pura

Ahora suéltate. La lógica de pago del escrow y del vault, destilada a una función pura para que se califique determinísticamente, se entrega vulnerable: ninguna verificación de autoridad y resta cruda. Dos clases de esta lección viven adentro, la verificación de firmante/dueño y el underflow aritmético. Tu trabajo es agregar las dos guardas, y después portar esas mismas dos guardas de vuelta a la instrucción del escrow para que la destilación y el programa de verdad coincidan.

El starter, en un proyecto `cargo` común (sin Anchor, sin toolchain, compila en cualquier lado donde compile `rustc`). Cada guarda es su propia `const fn` — el dispositivo de aserción en tiempo de compilación de m03-l3, y lo que la calificación de solo-compilar de verdad impone — con `settle_withdraw` ya cableado a través de las dos, así que los dos cuerpos vulnerables son todo el ejercicio:

```rust
// `Address` is 32 bytes, the shape `pinocchio::address::Address` really has.
//
// Return convention (so the grader can value-compare):
//   >= 0  -> the new balance after a successful withdraw
//     -1  -> rejected: caller is not the authority
//     -2  -> rejected: amount would underflow the balance
type Address = [u8; 32];

/// Gate the withdraw: is `caller` the vault authority?
const fn is_authority(caller: &Address, authority: &Address) -> bool {
    // TODO: true only when ALL 32 bytes match. Right now every caller passes.
    let _ = (caller, authority);
    true
}

/// Settle the arithmetic: the balance left after the withdraw, or -2.
const fn checked_remaining(balance: u64, amount: u64) -> i64 {
    // TODO: checked arithmetic, so an over-withdraw returns -2. This raw `-`
    // is the drain.
    (balance - amount) as i64
}

fn settle_withdraw(balance: u64, amount: u64, caller: Address, authority: Address) -> i64 {
    if !is_authority(&caller, &authority) {
        return -1;
    }
    checked_remaining(balance, amount)
}
```

Dos guardas, en orden. El control de acceso viene primero: `settle_withdraw` ya devuelve `-1` en el momento en que `is_authority` dice que no, antes de que corra cualquier aritmética en nombre de quien llama — tu trabajo es hacer que `is_authority` de verdad diga que no. Después la aritmética: `u64::checked_sub` devuelve `None` exactamente cuando `amount > balance`, así que haz match sobre eso en `checked_remaining`, devuelve `-2` en `None`, y el balance nuevo en `Some`.

Las direcciones son de ancho completo a propósito. Una dirección son 32 bytes y no lleva ningún orden con significado, así que la única comparación legal es la igualdad sobre todos los 32 — en un handler de verdad eso es un solo `caller != authority`. Acá la barrera es una `const fn` para que el compilador pueda demostrarla mientras construye, y `==` sobre arrays es una llamada a trait que una `const fn` no puede hacer en Rust estable, así que escribes la igualdad de la forma en que la máquina la corre igual: recorre los bytes, los 32, y rechaza en el primer desajuste. Tres de los casos del calificador existen para demostrar que comparaste todo y nada más barato: uno donde quien llama ordena *por debajo* de la autoridad, que una comparación de orden deja pasar, y dos casi-coincidencias que coinciden con la autoridad en 31 de 32 bytes — una que difiere en el primer byte, otra en el último — que cualquier comparación de prefijo o de un solo byte deja pasar. La razón para que te importe no es que alguien vaya a moler 31 bytes que coincidan — eso es 2²⁴⁸ de trabajo, y nadie lo está haciendo. Es que una barrera que compara un prefijo es una barrera cuyo prefijo se puede moler, y los prefijos cortos son baratos: la búsqueda de vanity los vende por carácter.

Criterios de aceptación que el calificador verifica directamente:

- quien llama sin ser la autoridad es rechazado con `-1`, sin importar si su dirección ordena por encima o por debajo de la de la autoridad
- una dirección que coincide con la autoridad en 31 de 32 bytes igual es rechazada con `-1`, sea cual sea el byte que difiere
- un over-withdraw que haría underflow es rechazado con `-2`
- un withdraw de la autoridad dentro del balance devuelve el balance nuevo (`100, 30, [7u8; 32], [7u8; 32]` devuelve 70; un `50, 50, [7u8; 32], [7u8; 32]` de balance exacto devuelve 0)
- el starter ni siquiera compila — la barrera abierta falla aserciones de tiempo de compilación cuyos mensajes nombran a quien llama que dejó pasar, y el `-` crudo hace overflow en la evaluación const sobre el caso de over-withdraw; tu solución compila limpio y pasa cada caso

Cuando la función pura esté verde, pórtala: el recorrido de bytes en `is_authority` es un solo `caller != authority` en código de handler de verdad — el constraint `address = escrow.player` que ya agregaste — y el `checked_sub` es el débito del vault que ya parcheaste. La destilación y la instrucción imponen las mismas dos guardas. Ese es el punto del ejercicio, que la guarda es la guarda ya sea que viva en un constraint, en un `require!`, o en una función pura.

## ¿Funcionó?

Ahora deberías tener un prize-escrow cuyas tres pruebas de exploit fallan todas, una liberación legítima que sigue pasando, y una guarda de withdraw que rechaza tanto a quien llama equivocado como al over-withdraw, en forma de función pura sobre la que puedes razonar aislada. El extraño no lo puede drenar. El maker sustituido no puede robar el rent. El over-withdraw no puede hacer wrap del libro mayor. Y cada uno de esos arreglos fue un solo constraint o una sola operación verificada, agregado porque *supiste agregarlo*, no porque el compilador te forzara la mano.

Mantén afilado el filo de esta lección, porque es la parte que la gente entiende exactamente al revés. Las victorias de tiempo de compilación de la lección pasada son reales, y las clases que sobreviven de esta lección son igual de reales, y el segundo conjunto es más peligroso precisamente porque el primero te entrena a confiar en el framework. La clase de sustitución de cuentas en particular es tuya para siempre: ningún compilador, en este framework ni en ningún otro, puede distinguir una cuenta correcta de la de un atacante cuando sus tipos coinciden y solo difiere la intención. Si una prueba de exploit todavía te pasa, no es un misterio, es una guarda que falta, y el nombre de la prueba es la clase.

Una última cosa, y debería dejarte un poco incómodo de una forma útil. Si escarbas en el campo `repository` de npm de Anchor a lo largo de los releases de este año, la custodia del framework pasó de `coral-xyz` en enero de 2026 a `solana-foundation` desde marzo hasta mayo y a `otter-sec` de junio en adelante, y ni el README del repositorio, ni su changelog, ni ninguna nota de release anuncia ninguno de los dos traslados. El framework insignia del ecosistema cambió de manos dos veces en seis meses y los metadatos del registro son donde te enteras. No te estoy diciendo eso para espantarte de Anchor, es excelente y deberías usarlo. Te lo estoy diciendo porque "el framework me cubre la espalda" es una *suposición* de seguridad, y esta es una lección sobre nunca dejar que una suposición ocupe el lugar de una verificación. La gente que mantiene a tu guardaespaldas puede cambiar sin que lo notes. La guarda que escribiste tú mismo no.

Parcheaste todo lo que se te ocurrió atacar. Pero "todo lo que se te podía ocurrir" es exactamente la estrategia de auditoría equivocada, porque los inputs que drenan escrows son los que nadie pensó en probar. La lección que viene dejas de adivinar y sueltas al fuzzer, generando los inputs que nunca imaginaste y dejándolo encontrar la guarda que todavía te falta.
