# Pre-Flight Check

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` · `borsh` resolves to **1.8.0** · `litesvm 0.15.x` (minor pinned; patch moves weekly) · Agave ≥ 3.1.10 · rustc 1.89+ · edition 2021.

Everything after this page costs something. The next lesson puts real devnet SOL in your wallet, the one after that spends it on 200-odd transactions, and the program id that comes out is the artifact your credential is keyed to. This is the last page where being wrong is free.

So before you spend a lamport: what do you actually know, and what have you only compiled?

## A green build is one bit of information

The build server compiles your file and grades on exactly that: **did it compile.** There is no test step on the Rust path. The grader never reads your code.

That bit is worth having — a program that does not compile cannot be deployed, and Anchor's macros catch a large class of account-wiring mistakes at compile time, which is most of why this course uses Anchor at all. But it is one bit, and Course 2 already showed you four ways to spend it on a vault that does not work:

| What you wrote                                       | Build | What happens on-chain                              |
| ---------------------------------------------------- | ----- | -------------------------------------------------- |
| `withdraw` calling `checked_add`                     | green | the vault pays you for withdrawing                 |
| the `require!(amount > 0, …)` guard dropped          | green | zero-amount instructions succeed and write nothing |
| `.unwrap()` where `.ok_or(…)?` belongs               | green | a panic — aborted instruction, no error code       |
| `self.balance = self.balance - amount`               | green | a panic, for the same reason                       |

None of those changes a type, and nothing type-level can see them.

Course 3 adds four more of its own:

- **`deposit` that moves the lamports but forgets `vault.deposit(amount)`.** The CPI lands; the stored `balance` stays at zero. Two numbers that are supposed to agree, and only one of them got written.
- **`withdraw` written as a System Program CPI with signer seeds.** It compiles. It deploys. It fails the first time anyone calls it, with `Transfer: 'from' must not carry data` — because System will not debit an account that carries data, and does not own your vault in any case. Signer seeds solve authorisation, and authorisation was never the obstacle.
- **A dropped rent floor.** Debit the vault below its rent-exempt minimum and the runtime kills the whole transaction with `InsufficientFundsForRent` — a failure your caller cannot act on, in place of the `InsufficientFunds` you would have returned.
- **A bump that is re-derived instead of read.** `bump` (bare) and `bump = vault.bump` both compile and both usually resolve to the same address. They diverge in compute units, and they diverge in intent: one trusts what you stored, the other recomputes it every call.

The compiler is indifferent to all eight. So is the grader.

## The tool that is not indifferent: LiteSVM

**LiteSVM** is the SVM — the same runtime a validator executes your program in — as a Rust library you construct in-process. No validator to boot, no RPC, no ports, no keypair files. You build a machine, load your `.so` into it, fund an address, send a transaction, and read the account back. Milliseconds, not tens of seconds.

That is the whole reason it matters here: the thing that catches a swapped operator is a test that **runs** `withdraw` and looks at the number afterwards. Nothing else does.

```rust
// litesvm 0.15.x — reading material, not a graded exercise.
let mut svm = LiteSVM::new();
svm.add_program_from_file(program_id, "target/deploy/academy_program.so")
    .unwrap();
svm.airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

// initialize → deposit 2 SOL → withdraw 1 SOL
svm.send_transaction(initialize_vault_tx(&svm, &user)).unwrap();
svm.send_transaction(deposit_tx(&svm, &user, 2 * LAMPORTS_PER_SOL)).unwrap();
svm.send_transaction(withdraw_tx(&svm, &user, 1 * LAMPORTS_PER_SOL)).unwrap();

// The assertion the compiler cannot make.
let raw = svm.get_account(&vault_pda).unwrap().data;
let state = VaultState::try_deserialize(&mut raw.as_slice()).unwrap();
assert_eq!(state.balance, 1 * LAMPORTS_PER_SOL);
```

Three things to notice, because they are the point rather than the syntax.

**The `unwrap()`s are fine here.** Course 2 spent a lesson on why a panic inside a program is unacceptable: the caller gets an aborted instruction with no code and no message. A test is the opposite situation — a panic *is* the failure report, and it is the shortest one available. The rule was never "never `unwrap`"; it was "never `unwrap` where someone else has to read the error."

**`assert_eq!(state.balance, …)` is the assertion nothing else in this course can make.** It compares a number the runtime produced against a number you predicted. Every green build up to now has compared a shape against a shape.

**The last line reads the account back rather than trusting the return value.** `send_transaction` succeeding means the instruction did not error. It does not mean it wrote what you meant. Those are different claims and the second one is the one you care about.

LiteSVM is Apache-2.0, which makes it the one major testing corpus in this ecosystem whose examples you can adapt into your own repo with attribution rather than reimplement from scratch. Pin the minor — it ships roughly weekly, and `0.15.x` is what the snippet above was written against.

## What this platform runs, stated plainly

The code block on a lesson has two modes, `standard` and `buildable`. `buildable` shells out to the Anchor toolchain and compiles. **There is no test mode.** Adding one is a two-stage build-then-run pipeline on the build server, not a flag, and it is tracked separately.

Which means: the LiteSVM file above is not something you submit here today. It is something you read, and then write for real the first time you have a repo of your own — which, if you follow this path, is Course 4.

It also means the honest description of what you are about to deploy is *"compiled against the real Anchor 1.1 toolchain,"* and never *"unit tested."* If you write up this build — and the last lesson asks you to — use the first one. The distinction is the kind of thing a reviewer notices.

## So the pre-flight check is you

No new material below. The checkpoint asks about decisions you made across the last eleven lessons, cold, out of the order you made them in, plus two from Course 2 that Course 3 has been quietly leaning on the whole time.

Getting one wrong here costs nothing. Getting it wrong after the deploy costs an upgrade transaction and the walk back through this module to find out which of the eight silent failures you shipped.
