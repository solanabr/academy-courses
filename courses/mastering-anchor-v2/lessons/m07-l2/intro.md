# What still bites you (exploit then patch)

You just watched type cosplay, duplicate-mutable, and CpiHandle aliasing die at compile time: three attacks that would not build. The fourth, bump recalculation, built cleanly and then did nothing, because the framework signs with a macro-time const and your recomputed byte had nowhere to go. Three refused, one hollow. The compiler was your bodyguard, and it did the job. This lesson the bodyguard goes home.

So do not read yet. You are going to make the vulnerable branch yourself, from the escrow you already built, because the guards you are about to remove are guards *you* wrote and the removal is the first thing worth feeling.

From the same `quarters` workspace as last lesson, on your clean R3/R4:

```bash
git checkout -b vuln/prize-escrow
```

Then open `programs/quarter-prize/src/lib.rs` and delete three constraints from the `Redeem` accounts struct: the `address = escrow.player` on `player`, the `address = escrow.vault` on `vault_state`, and the `address = escrow.maker` on `maker`. Leave everything else. That is a rushed first draft, and it is what half the escrows on this chain shipped as.

The `Escrow` record itself is unchanged from m04-l3, and the exploits below read it, so keep it in view:

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

Now write `drain_as_stranger` (it is printed in full below, in class 1) into `programs/quarter-prize/tests/exploits.rs` and run it:

```bash
anchor build && cargo test --test exploits drain_as_stranger
```

```text
running 1 test
test drain_as_stranger ... ok

test result: ok. 1 passed; 0 failed
```

Read that result for what it is. A stranger, not the player the escrow named, just walked off with the whole prize, and the test that proves it says `ok`. Nothing in that program is a type error. It compiled clean on the V2 toolchain that killed three attack classes last lesson. It ships. And it is drainable. That gap, between "compiles" and "safe," is the entire lesson, and you are going to attack it with working exploits before you close it.

## What this lesson settles

Last lesson answered "what does V2 kill for free." This one answers the harder question: what does V2 still expect you to write, and what can no compiler in any framework ever write for you. The Solana Foundation program-security taxonomy has two halves. You already retired the half that became compile errors. Here is the surviving half, run against your own escrow (R3) and swap (R4). Three of them you land as working exploits and then patch: the signer/owner check, the account substitution, and the arithmetic underflow. The other four you learn to *spot*, because each one is either already closed by a constraint the escrow happens to use or needs a program shaped differently than this one to demonstrate. Which four are which is worth noticing as you read; the lab exploits exactly the three:

1. **Signer and owner checks.** V2 still expects you to assert who is allowed to act. Miss the assertion and anyone acts as the authority.
2. **UncheckedAccount substitution.** `UncheckedAccount` opts out of every framework check by name, so it must be paired with an explicit `address`, `owner`, or `constraint`. Without one, an attacker substitutes their own account. This is the account-substitution class, and it is the one that survives every framework ever written.
3. **The owner-error footgun.** `#[account(owner = X @ MyErr)]` on an `Account<T>` does not surface `MyErr`. It surfaces `ProgramError::IllegalOwner`, because owner and discriminator checks run inside `load`, before your constraint hooks.
4. **init_if_needed reuse.** V2 ships `init_if_needed` ungated and validates space, owner, and discriminator on reuse, but a re-initialize-over-live-state logic bug still survives it.
5. **Arbitrary CPI.** Invoke a program the caller handed you and you invoked whatever they wanted. Validate the target program id.
6. **Close-and-revival.** A closed account can be revived inside the same transaction unless you zero it and guard it.
7. **Arithmetic overflow.** Unsigned subtraction underflows. Use checked math.

The autonomy fades the usual way. I work the signer/owner class end to end, exploit and patch, because I want the drain in your test output and the fix in your fingers once. UncheckedAccount substitution drops to a completion problem: I hand you the patch, you write the exploit that proves the hole was real. The withdraw guard is the last rung, a pure-Rust coding challenge stripped of Anchor entirely, where you get the signature and the return convention and nothing else. Land the exploit, close the hole, prove it stays closed.

![A two-column comparison putting the three classes retired at compile time, plus the bump class that compiles but has no seam, beside the seven surviving classes this lesson must patch by hand.](assets/v01-comparison.png)

One trap to name before we start, because it is the whole reason this half is dangerous. Every guard you add costs compute and code you have to maintain, and the framework will never tell you which guard is *missing*. It only rejects the spellings it already knows. So "secure by default" is a claim about the classes on the top of that table, not the bottom. Over-trusting it on the bottom half is exactly how escrows get drained.

## The surviving classes, one attack at a time

The method is fixed and it is the point: for each class, land an exploit that succeeds against the current code, name precisely why it works, then patch until the exploit fails and the legitimate path still passes. Read, break, fix, prove.

![A seven-step loop diagram: pick a class, write a passing exploit, name the missing guard, add it, and re-run until the exploit fails.](assets/v02-flowchart.png)

### Class 1: the signer and owner check (worked in full)

Start with the exploit you already ran. Here is the redeem accounts struct on the `vuln` branch, with the guards peeled back to where a rushed first draft leaves them:

```rust
#[derive(Accounts)]
pub struct Redeem {
    #[account(
        mut,
        close = maker,
        seeds = [b"escrow", escrow.maker.as_ref(), escrow.player.as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<Escrow>,

    /// CHECK: pinned by the address constraint the patch adds below
    #[account(mut)]                        // VULN: any vault, not the one the escrow recorded
    pub vault_state: UncheckedAccount,

    #[account(mut)]                        // VULN: any signer, not the recorded player
    pub player: Signer,

    #[account(mut)]                        // VULN: any account, substitutable rent recipient
    pub maker: UncheckedAccount,

    pub quarter_vault_program: Program<QuarterVault>,
    pub system_program: Program<System>,
}
```

Nothing here is a compile error. `player` is a real `Signer`, so *someone* signed. But the program never checks that the someone is `escrow.player`. The whole conditional release turns on "only the named player may claim," and that sentence appears nowhere in the code. The exploit writes itself. A stranger signs a redeem, clears the score bar (self-reported, remember, this cabinet does not attest scores yet), and the vault pays them. The exploit tests run on LiteSVM, V2's default in-process Rust test harness, so add it to the program if it is not already there:

```bash
# Reach LiteSVM through the harness, never by name. anchor-v2-testing owns the litesvm
# version (0.11.0 at tag v2.0.0-rc.1; the anchor-next head has already moved it to 0.13.1),
# so pinning the tag pins the SVM too. Add litesvm yourself and a mismatch shows up as a
# runtime type error rather than a build error.
cargo add anchor-v2-testing --dev \
  --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1
```

```rust
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{Keypair, Message, Signer, VersionedMessage, VersionedTransaction};

#[test]
fn drain_as_stranger() {
    let mut svm = anchor_v2_testing::svm();
    svm.add_program_from_file(quarter_vault::ID, "target/deploy/quarter_vault.so").unwrap();
    svm.add_program_from_file(quarter_prize::ID, "target/deploy/quarter_prize.so").unwrap();

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

    reserve_prize(&mut svm, &maker, &player, escrow, vault, 50_000_000, 5_000);

    // The stranger, NOT escrow.player, redeems with a passing score.
    let before = svm.get_account(&stranger.pubkey()).unwrap().lamports;
    let ix = Instruction {
        program_id: quarter_prize::ID,
        accounts: quarter_prize::accounts::Redeem {
            escrow,
            vault,
            player: stranger.pubkey(),     // substitute the caller
            maker: maker.pubkey(),
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
    let after = svm.get_account(&stranger.pubkey()).unwrap().lamports;
    assert!(after > before, "the stranger drained the prize");
}
```

That is a working drain, and it is 30 lines. The patch is one. On the `player` field, pin the caller to the pubkey the escrow recorded:

```rust
#[account(mut, address = escrow.player)]
pub player: Signer,
```

`address = escrow.player` says, plainly, the account passed here must equal the player this escrow named. It is the V2 idiom that replaced `has_one`, and it takes any expression, so it reads as what it does. Re-run `drain_as_stranger` and it flips: the transaction now fails at account load, before your handler runs a single line, because the stranger's key does not match `escrow.player`. Re-run the legitimate `conditional_release` test and it is still green. Exploit dead, feature intact. That is the whole loop, and every class below is a variation on it.

![An annotated code card showing the unpinned player field a stranger can drain, and the address constraint that rejects the stranger at account load.](assets/v03-annotated-code.png)

### Class 2: UncheckedAccount substitution (the one that survives every framework)

Look back at that `maker` field. It is an `UncheckedAccount`, and on the vuln branch it carries only `mut`. That word `UncheckedAccount` is not decoration. It is the type opting out of every framework check there is: no owner check, no discriminator check, no identity check. You are telling Anchor "I will validate this myself," and then not doing it.

Here is why that is the deepest class in the lesson. The escrow closes to `maker`, returning the rent. On the vuln branch, `maker` is any account the caller passes. So an attacker passes *their own* account as `maker`, and the rent lamports from `close = maker` land in their wallet instead of the operator's. Small money on one escrow, real money across a thousand. And nothing catches it, because there is nothing to catch: the account is valid, it is mutable, it is writable. It is simply not the account the escrow meant.

This is the account-substitution class, and I want you to sit with a claim: no compiler in any framework, in any language, can catch this one for you. Type cosplay was catchable because the types differed. Duplicate-mutable was catchable because the addresses aliased. Substitution has no signal. A correct account and an attacker's account have identical types, identical owners in the general case, identical everything except *which one the business logic intended*, and intent is not in the type system. It is the one class you own forever.

![A side-by-side comparison of an UncheckedAccount carrying only mut against one pinned by address, owner, or constraint, showing which substitutions each allows.](assets/v04-comparison.png)

The patch on the escrow restores the pin the frozen version always had:

```rust
#[account(mut, address = escrow.maker)]
pub maker: UncheckedAccount,
```

Now the rent can only return to the recorded maker. Do the same to `vault`, which on the vuln branch is any `Account<Vault>`, so an attacker substitutes a *different* vault they control and redirects the withdraw. The patch pins it to the escrow's recorded vault and gives a real error while it is at it:

```rust
#[account(mut, address = escrow.vault @ EscrowError::WrongVault)]
pub vault: Account<Vault>,
```

**Completion problem.** I gave you the two patches above. Now you write the exploit that proves the `maker` hole was real. Fork `drain_as_stranger` into a `steal_rent_on_close` test: the legitimate player redeems correctly, but passes `maker: attacker.pubkey()` instead of the true maker, and you assert the attacker's balance grew by roughly the escrow's rent. Land it red on the peeled-back field, apply the `address = escrow.maker` pin, watch it go green. The accept bar is exactly the loop: the test passes against the vuln field and fails against the patch.

One pointer, because you should know where the flagship version of this class lives: the Cashio drain, where a missing `.mint` check let an attacker mint collateral from nothing, is written and patched end to end in the DeFi and RWA Engineering course. This course does not develop it; go there for the war story. Here, own the class.

### Class 3: the owner-error footgun (why the check you expected does not fire)

Here is a trap that looks like a fix. Say you want a custom error when someone passes a vault owned by the wrong program. The natural spelling is:

```rust
#[account(owner = quarter_vault::ID @ EscrowError::WrongVault)]
pub vault: Account<Vault>,
```

You write your test, pass a vault owned by some other program, and expect `WrongVault`. You get `ProgramError::IllegalOwner` instead. Your `@ EscrowError::WrongVault` never fired. Why?

(There is a second reason that spelling is wrong for *this* field specifically, and it is worth spotting before the mechanism: the escrow's vault is owned by `quarter_vault`, a different program, so it could never load as a typed `Account<Vault>` inside `quarter_prize` at all. That is why R3 declares it `UncheckedAccount`. The footgun below is what bites when the account genuinely is yours and you just wanted a nicer message.)

This is worth deriving, not just memorizing, because the reason generalizes. An `Account<T>` is a typed wrapper, and before your constraint hooks run, Anchor has to *load* it: read its bytes, check the account is owned by the declaring program, and check the discriminator matches `T`. Those two checks, owner and discriminator, are structural. They happen inside `load`, first, because the framework cannot hand you a typed `T` it has not verified is a `T`. Your `owner = ... @ MyErr` constraint is a *hook*, and hooks run after load. So on a wrong-owner account, load fails first with `IllegalOwner`, and control never reaches the hook that carries your custom error.

![A diagram of check ordering for Account<T>, with owner and discriminator running inside load before any constraint hook, so IllegalOwner wins over the custom error.](assets/v05-diagram.png)

The fix, when you genuinely need the custom error, is to stop asking `Account<T>` to carry it. Take an `UncheckedAccount`, which does no load-time owner check (exactly the type from class 2), and put the *same* `owner = X @ MyErr` constraint on it. Now there is no `load` to short-circuit, so the constraint hook is the only thing checking the owner, and it carries your error:

```rust
#[account(owner = quarter_vault::ID @ EscrowError::WrongVault)]
pub vault: UncheckedAccount,
```

That is the framework's own prescription: the doc comment on V2's `Account<T>` alias says, in as many words, that for a custom error you use `UncheckedAccount` with a derive-level `owner = X @ MyErr`. The trade is that you gave up the typed view, so if the handler needs the vault's fields you now load and validate them yourself.

Notice the shape: the two classes compose. The type that opts out of the framework's checks is the same type that lets you write your own. That is not a coincidence, it is the framework being honest about what it does and does not do for you.

### Class 4: init_if_needed reuse (ungated is not the same as safe)

A teammate remembers `init_if_needed` from the v1 line as the feature-gated one, the one you had to explicitly enable because it was dangerous. Correct the record, because V2 changed it and half-remembering the change is its own hazard.

In V2, `init_if_needed` is no longer behind a feature flag. I checked the V2 feature set against the framework's own docs: the release ships six feature flags, `alloc`, `guardrails`, `idl-build`, `compat`, `const-rent`, and `testing`, and `init-if-needed` is not among them. It is ungated. On top of that, `init_if_needed` accounts have been folded into the duplicate-mutable check since the 1.0 line (#4239) and still are under V2, and the reuse branch re-validates the account's space, owner, and discriminator, which closes the crudest reinitialize tricks.

Here is the part that survives all of that. The framework can verify the account is the right *shape*. It cannot verify that reinitializing *this* account is the right *thing to do*. If your instruction hits the `init` branch over an account that already holds live state, the reuse-validation passes (space matches, owner matches, discriminator matches) and you cheerfully overwrite a funded escrow back to zeros.

![A table splitting init_if_needed reuse-validation into the structural checks V2 performs and the business intent it cannot judge, where a reinit bug survives.](assets/v06-table.png)

Freshness note: this reflects the Anchor V2 release candidate as of 2026-08-22, verified against the framework's feature-flag docs and changelog. V2 is still an RC with no stable tag, so if you pin a newer RC, re-read its `init_if_needed` reuse-validation behavior before you trust the exact semantics. The mitigation does not change: if an account can hold live state, guard the reinit yourself. Check a stored flag or a nonzero field before you let the `init` branch run, and reject when the account is already live.

### Class 5: arbitrary CPI (validate the target program id)

Your escrow calls the vault. It declares the callee as `quarter_vault_program: Program<QuarterVault>`, and that typed `Program<T>` is doing quiet security work: it checks the account's key equals `QuarterVault::id()` — the marker impl you declared back in m04-l3 — and, with the default `guardrails` feature on, that the account is executable. You cannot be tricked into calling something else, because the type pins the target.

Now imagine you got lazy and typed it as an `AccountInfo` or `UncheckedAccount`, then invoked it:

```rust
// VULN: the callee is whatever the caller passed.
let cpi_ctx = CpiContext::new(ctx.accounts.some_program.address(), cpi_accounts);
// ... invoke ...
```

An attacker passes their own program as `some_program`, your escrow signs the CPI with the escrow PDA's seeds, and now the attacker's program runs *with your PDA's signature*. That is the arbitrary-CPI class: you did not choose the program that ran, the caller did, and you handed it your authority.

The patch is to let the type pin the target, exactly as the frozen escrow does. For the swap, the same rule applies to the token program: `token_program: Interface<'static, TokenInterface>` pins the callee to a real token program (classic or Token-2022) rather than accepting an arbitrary one.

![A flowchart contrasting an untyped program account an attacker can choose with a typed Program that pins the callee to the vault's program id.](assets/v07-flowchart.png)

### Class 6: close-and-revival (zero it, or guard it)

Closing an account is not just moving its lamports out. On Solana, an account with zero lamports at the end of a transaction is swept, but *within* the transaction, an account you drained and shrank can be topped back up and reused before the sweep. If your close is hand-rolled, transfer the lamports, realloc to nothing, and stop, you left the door open: a later instruction in the same transaction refunds it and your program's discriminator is still sitting in the (now revived) data, so the account passes validation again on a second call. That is close-and-revival.

The escrow avoids it because it uses the `close = maker` constraint, and V2's close zeroes the data, writes the closed-account sentinel, and assigns the account to the system program. There is nothing left to revive. The class only bites when someone reaches past the constraint and closes by hand. If you ever do, the rule is: zero the discriminator and guard against the revived shape, do not just move the lamports.

![A single-transaction timeline where a lamports-only close leaves the discriminator intact and the account is revived, beside a zeroing close that blocks revival.](assets/v08-timeline.png)

### Class 7: arithmetic overflow (the withdraw guard's real bug)

The last class is the smallest to state and the easiest to ship. The vault's withdraw debits a balance. On the vuln branch it does it with raw subtraction:

```rust
// VULN: unsigned subtraction underflows.
vault.balance = vault.balance - amount;
```

If `amount > balance`, this does not error. In debug builds it panics; in a build with overflow checks off it *wraps*, so a balance of 30 minus a withdraw of 100 becomes a gigantic positive number and your vault believes it holds far more than it does. Neither outcome is "the withdraw was rejected," which is the only correct one.

Know where your build sits on that, because it decides which of the two you get. Anchor's generated workspace `Cargo.toml` sets `overflow-checks = true` on the release profile, and `cargo build-sbf` uses release, so on an untouched scaffold this panics rather than wraps. Two things make the wrap real anyway. Someone removes that line, which happens the first time a team chases CU. Or someone turns `guardrails` off, which you did yourself last lesson. Either way the wrap is one Cargo edit away, and a guard that only holds because of a profile setting is not a guard.

Access control does not save you here. A perfectly authorized player can still request more than the vault holds. This is not a "who" bug, it is a "how much" bug, and the fix is checked arithmetic:

```rust
// PATCH: checked_sub returns None exactly when amount > balance.
vault.balance = vault
    .balance
    .checked_sub(amount)
    .ok_or(VaultError::Underflow)?;
```

`checked_sub` returns `None` in precisely the case that would underflow, so you convert that `None` into a real error and reject the over-withdraw. It is one method and one `?`. It is also, not coincidentally, the exact patch your coding challenge asks for.

![An annotated code card comparing raw subtraction and checked_sub on a withdraw of 100 against a balance of 30, one wrapping and one rejecting.](assets/v09-annotated-code.png)

One housekeeping fact for your patches. Anchor's own error codes live in the 2000s (constraint failures like `ConstraintAddress` and `ConstraintOwner` are there). Your custom `#[error_code]` variants start at 6000 and count up. So an error in the 6000s is one of yours, and *which* one depends on the program: `quarter_prize` and `quarter_vault` each have their own `#[error_code]` enum, each numbered from 6000 by declaration order, so 6001 means one thing in a redeem rejection and another in a withdraw rejection. Read the program the error came from before you read the number. Knowing which band an error lives in tells you at a glance whether the framework rejected the transaction or your own guard did.

### The same loop on the swap

The escrow was the whole taxonomy on one program. The swap (R4) is the same classes wearing token accounts instead of lamport vaults, and running the loop against it is what convinces you these are *classes*, not escrow trivia. `swap_arcade_for_tickets(amount_in, min_out)` pulls the trader's arcade tokens into the pool's arcade reserve and pushes tickets back out. Two of the surviving classes map straight onto it.

First, substitution, class 2 again. The swap's `reserve_arcade` and `reserve_ticket` are the pool's own token accounts, the ones the trades price against. If those fields are typed as bare `InterfaceAccount<TokenAccount>` with no pin to the pool that owns them, an attacker passes *their own* token accounts as the reserves. The constant-product math then prices against balances the attacker controls, so they quote themselves a fill the real pool would never offer and walk the ticket reserve out. Same shape as the stranger draining the escrow: a valid account of the right type, simply not the one the program meant. The patch is the same too, pin each reserve to the pool's recorded reserve so a substituted account is rejected at load.

Second, arbitrary CPI, class 5. The swap CPIs `transfer_checked` through `token_program`, and the frozen swap types it as `Interface<'static, TokenInterface>`, which pins the callee to a real token program (classic or Token-2022) and nothing else. Type it as an `UncheckedAccount` instead and the trader chooses which program moves the tokens, with the pool's authority behind the call. The type is the guard.

![A comparison mapping each surviving escrow vulnerability class onto the swap's own fields, with the guard that closes it named in the final column.](assets/v10-comparison.png)

You do not re-derive anything to attack the swap. You carry the same seven questions over and ask them of a different account list. That portability is the reason the taxonomy is worth learning as classes rather than as a checklist for one program.

## Lab: exploit then patch the escrow

You have seen every class. Now run the loop yourself against the escrow, end to end, until every exploit test fails and the suite is green.

**Step 1. Pin the V2 toolchain and get on the vulnerable branch.** This course runs on the Anchor V2 release candidate, not the 1.1.2 V1 line many machines ship by default. `avm install` cannot fetch the V2 RC: it downloads a prebuilt binary from a published GitHub release, and no release was cut for the v2 tag, so the download 404s. Install the CLI directly from the `v2.0.0-rc.1` tag:

```bash
# Anchor V2 RC CLI, built from the tag (no release binary for the v2 tag to download).
cargo install --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1 anchor-cli --locked --force
# macOS: prefix with CARGO_PROFILE_RELEASE_LTO=off if the release build fails to link.
anchor --version              # confirm the V2 line, not 1.1.2

# You cut this branch and peeled the three constraints back at the top of the lesson.
git checkout vuln/prize-escrow
```

Freshness note: as of 2026-08-22 the V2 line ships only as release candidates (2.0.0-rc.1, tagged on `anchor-next`), so there is no stable version to hardcode. The branch head advances, so record the exact commit you built in `Anchor.toml` and CI so a teammate builds the same bytecode. When V2 tags stable, pin that instead.

**Step 2. Land the three exploits.** Your `vuln` branch has `Redeem` with the guards peeled back: `player` with no `address`, `maker` with no `address`, `vault_state` with no `address`. Add the fourth peel now, in the vault: replace the `checked_sub` in `withdraw` with a raw `vault.balance - amount`. Then put all three exploit tests in `programs/quarter-prize/tests/exploits.rs`: `drain_as_stranger` from class 1, `steal_rent_on_close` from the class-2 completion problem, and `over_withdraw`, which reserves a small prize and then redeems for more than the vault holds. Run all three and watch them pass, which is the wrong result and the whole point:

```bash
anchor build && cargo test --test exploits
```

```text
test drain_as_stranger   ... ok
test steal_rent_on_close ... ok
test over_withdraw       ... ok

test result: ok. 3 passed; 0 failed
```

Three green exploits is three real holes. `drain_as_stranger` is the signer/owner class from the walkthrough. `steal_rent_on_close` is the account-substitution completion problem you wrote in class 2. `over_withdraw` requests more than the vault holds and the raw subtraction lets it through. Checkpoint: all three report `ok`. If `steal_rent_on_close` fails instead, your test is asserting the wrong thing, not proving the hole is closed, so re-read the accept bar in class 2 before you move on.

**Step 3. Patch, one class at a time.** Apply the four constraints and the one checked op, exactly as derived above:

```rust
// in Redeem accounts:
/// CHECK: pinned to the exact vault this escrow recorded
#[account(mut, address = escrow.vault @ EscrowError::WrongVault)]
pub vault_state: UncheckedAccount,

#[account(mut, address = escrow.player)]
pub player: Signer,

#[account(mut, address = escrow.maker)]
pub maker: UncheckedAccount,
```

```rust
// in the vault's withdraw handler:
vault.balance = vault
    .balance
    .checked_sub(amount)
    .ok_or(VaultError::Underflow)?;
```

And keep the win-condition guard ahead of the payout CPI, where it gates the release by position:

```rust
require!(final_score >= ctx.accounts.escrow.winning_score, EscrowError::ConditionNotMet);
```

Checkpoint: `anchor build` is green. Four constraints and one checked op is the entire patch set, so if the build fails it is a spelling problem, not a design problem, and the compiler names the field.

**Step 4. Prove the exploits are dead and the feature lives.** Run the exploits and the legitimate path together:

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

Read that inverted result carefully, because a failing exploit test is success here. `drain_as_stranger` and `steal_rent_on_close` now fail with `ConstraintAddress`, the framework's 2000s-band rejection at account load: the wrong caller and the substituted maker never reach your handler. `over_withdraw` fails with a custom error in the 6000s, whichever number your `Underflow` variant landed on given its position in `VaultError` (variants are numbered from 6000 by declaration order, so count yours rather than copying mine). The runtime prints it in hex, so a variant at 6001 shows as `0x1771`. And `conditional_release`, the real player clearing the real bar, still passes. Checkpoint: the three exploits flip from pass to fail, and the legitimate release stays green. If any exploit still passes, the culprit is the guard you have not added yet, and the test name tells you which class.

![A top-to-bottom flowchart of the guarded redeem, from address-pinned account loading through the win check, signed payout, checked debit, and zeroing close.](assets/v11-flowchart.png)

## Challenge: patch the withdraw guard as a pure function

Now cut loose. The escrow and vault payout logic, distilled to a pure function so it grades deterministically, ships vulnerable: no authority check and raw subtraction. Two classes from this lesson live in it, the signer/owner check and the arithmetic underflow. Your job is to add both guards, then port the same two guards back into the escrow instruction so the distillation and the real program agree.

The starter, in a plain `cargo` project (no Anchor, no toolchain, it compiles anywhere `rustc` does):

```rust
// `Address` is 32 bytes, the shape `pinocchio::address::Address` really has.
//
// Return convention (so the grader can value-compare):
//   >= 0  -> the new balance after a successful withdraw
//     -1  -> rejected: caller is not the authority
//     -2  -> rejected: amount would underflow the balance
type Address = [u8; 32];

fn settle_withdraw(balance: u64, amount: u64, caller: Address, authority: Address) -> i64 {
    // TODO: reject callers who are not the authority (return -1).
    // TODO: use checked arithmetic so an over-withdraw returns -2 instead of underflowing.
    (balance - amount) as i64
}
```

Two guards, in order. Access control comes first: if `caller != authority`, return `-1` before you touch the balance, because you should reject an unauthorized caller without doing any arithmetic on their behalf. Then the arithmetic: `u64::checked_sub` returns `None` exactly when `amount > balance`, so match on it, return `-2` on `None`, and the new balance on `Some`.

The addresses are full-width on purpose. An address is 32 bytes and carries no meaningful ordering, so the only legal comparison is `==` over all 32 of them. Two of the grader's cases exist to prove you did that and nothing cheaper: one where the caller sorts *below* the authority, which an ordering comparison waves through, and two near-misses that match the authority in 31 of 32 bytes — one differing in the first byte, one in the last — which any prefix or single-byte comparison waves through. A near-miss address is not a rounding error. It is an attacker who ground out a vanity key.

Acceptance criteria the grader checks directly:

- a non-authority caller is rejected with `-1`, whether their address sorts above or below the authority's
- an address matching the authority in 31 of 32 bytes is still rejected with `-1`, whichever byte differs
- an over-withdraw that would underflow is rejected with `-2`
- an authority withdraw within balance returns the new balance (`100, 30, [7u8; 32], [7u8; 32]` returns 70; an exact-balance `50, 50, [7u8; 32], [7u8; 32]` returns 0)
- the starter fails at least one case; your solution passes every case

When the pure function is green, port it: the `caller != authority` check is the `address = escrow.player` constraint you already added, and the `checked_sub` is the vault debit you already patched. The distillation and the instruction enforce the same two guards. That is the point of the exercise, that the guard is the guard whether it lives in a constraint, a `require!`, or a pure function.

## Did it work?

You should now have a prize-escrow whose three exploit tests all fail, a legitimate release that still passes, and a withdraw guard that rejects both the wrong caller and the over-withdraw as a pure function you can reason about in isolation. The stranger cannot drain it. The substituted maker cannot steal the rent. The over-withdraw cannot wrap the balance. And every one of those fixes was a single constraint or a single checked operation, added because you *knew to add it*, not because the compiler forced your hand.

Keep the edge from this lesson sharp, because it is the part people get exactly backwards. Last lesson's compile-time wins are real, and this lesson's surviving classes are equally real, and the second set is more dangerous precisely because the first set trains you to trust the framework. The account-substitution class in particular is yours forever: no compiler, in this framework or any other, can tell a correct account from an attacker's when their types match and only the intent differs. If an exploit test still passes for you, it is not a mystery, it is a missing guard, and the test name is the class.

One last thing, and it should leave you a little uneasy in a useful way. If you dig into Anchor's npm `repository` field across this year's releases, custody of the framework moved from `coral-xyz` in January 2026 to `solana-foundation` from March through May to `otter-sec` from June onward, and neither the repository's README, its changelog, nor any release note announces either move. The ecosystem's flagship framework changed hands twice in six months and the registry metadata is where you find out. I am not telling you that to spook you off Anchor, it is excellent and you should use it. I am telling you because "the framework has my back" is a security *assumption*, and this is a lesson about never letting an assumption stand in for a check. The people maintaining your bodyguard can change without you noticing. The guard you wrote yourself does not.

You patched everything you could think to attack. But "everything you could think of" is exactly the wrong audit strategy, because the inputs that drain escrows are the ones nobody thought to test. Next lesson you stop guessing and turn the fuzzer loose, generating the inputs you never imagined and letting it find the guard you are still missing.
