# Build the Vault Core

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` (published 2026-06-26, latest on crates.io) · `borsh` resolves to **1.8.0** from the manifest's `borsh = "1.5.7"` · edition 2021 · Agave ≥ 3.1.10 · rustc 1.89+. Both exercises below were compiled clean against exactly that manifest on that date.

Twelve lessons of parts. This is the assembly.

You have already written every piece of this file, each one somewhere else: `checked_sub` returning `None` in lesson 4, a `&mut self` receiver in lesson 6, a `#[derive]`d struct in lesson 10, a `#[error_code]` enum in lesson 11, `?` carrying an error up a chain in lesson 12. What you have not done is put them in one file and hand it to the build server.

You do it twice. The first exercise gives you the struct, the enum and the two method signatures, and scrambles the seven lines that go inside them — five belong, two do not. The second gives you this spec, a `mod vault` with nothing in it, and a verification harness you may not edit.

## The spec

`declare_id!` first, because it is the one line that looks optional and is not.

```rust
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
```

That address is a placeholder — Course 3 replaces it with the program id of your own deployment. It has to be there today anyway: `#[account]` expands to an `impl Owner for VaultState` whose body reads `crate::ID`, and `declare_id!` is what defines `ID`. Delete the line and the compiler does not complain about the missing macro. It complains at your struct:

```
error[E0425]: cannot find value `ID` in the crate root
```

Which is a nice illustration of how macro-generated code fails: the error lands where the expansion happened, not where you made the mistake.

Then the state, the errors and the two methods.

| Item        | Exactly                                                                                                                |
| ----------- | ---------------------------------------------------------------------------------------------------------------------- |
| **Module**  | `pub mod vault { … }` — everything below lives inside it, and is re-exported from the crate root                        |
| **State**   | `#[account]` + `#[derive(InitSpace)]` on `pub struct VaultState { pub owner: Pubkey, pub balance: u64, pub bump: u8 }`  |
| **Errors**  | `#[error_code] pub enum VaultError` with exactly four variants: `Overflow`, `InsufficientFunds`, `ZeroAmount`, `NotOwner` |
| **Deposit** | `pub fn deposit(&mut self, amount: u64) -> Result<()>`                                                                  |
| **Withdraw**| `pub fn withdraw(&mut self, amount: u64) -> Result<()>`                                                                 |

Field names, field types, method names, parameter type and return type are all fixed. Course 3 wraps around this exact surface, and a client written against `balance` cannot read a field you decided to call `amount_held`.

### Invariants

1. **A zero amount is rejected before anything is written.** Both methods. `VaultError::ZeroAmount`.
2. **`deposit` adds with `checked_add`** and returns `VaultError::Overflow` when it gets `None`.
3. **`withdraw` subtracts with `checked_sub`** and returns `VaultError::InsufficientFunds` when it gets `None`.
4. **No `unwrap()`, no `expect()`, no bare `+` or `-` on `balance`.** A panic inside a program is an aborted instruction carrying no error code — the caller gets a generic failure and no message. Turning `None` into a named variant is the entire point of the last nine lessons.
5. **`balance` is only ever written through these two methods.** Nothing else in the file touches it.

`NotOwner` is declared and never used, deliberately. It is the error Course 3's ownership constraint returns, and declaring the enum complete now means Course 3 adds a constraint rather than editing your error type. The harness checks only that the variant **exists** and converts — nothing in a compile-time check can observe which errors your code raises, and a `deposit` that returned `err!(VaultError::NotOwner)` compiles identically.

## What is deliberately missing

Your file has **no `#[program]` module and no `#[derive(Accounts)]` struct**. It is not an unfinished program. It is the half of a program that does not depend on the runtime — no accounts, no signers, no clock, no CPI — which is exactly the half you can check by reading. Course 3 supplies the other half and the tests that exercise this one. Lesson 14 spells the split out line by line.

## One file, so modules are inline

The code block ships exactly one starter, and the manifest points at `src/lib.rs`, so everything you write is in one file. That is why the module is `pub mod vault { … }` written out inline rather than `mod vault;`. In a one-file crate `mod vault;` sends the compiler looking for `src/vault.rs`:

```
error[E0583]: file not found for module `vault`
```

Inline modules are not a workaround for the platform. They are how you namespace inside a file, and `use super::*;` at the top of the module body is what makes the crate root's imports — `Pubkey`, `Result`, `require!` — visible inside it. The crate root then needs one line to name the items back out:

```rust
pub use vault::{VaultError, VaultState};
```

Without it the items exist and nothing outside the module can say their names. The harness lives outside the module, so this is not a style point — it is a compile error.

## How this is graded, precisely

The build server compiles your file and grades on one bit: **did it compile.** There is no `cargo test` step on the Rust path, and the grader never reads your code or this description.

So the checking is done by the `mod verify` block at the bottom of both starters, and its reach is exact.

**It catches:**

- a missing or renamed item — `VaultState`, `VaultError`, `deposit`, `withdraw`
- a renamed or re-typed field, because `assert!(VaultState::INIT_SPACE == 41)` only holds for `Pubkey` (32) + `u64` (8) + `u8` (1)
- a missing `#[account]`, because that is what supplies the 8-byte discriminator the harness measures
- a missing `#[derive(InitSpace)]`, because that is what generates `INIT_SPACE`
- a wrong receiver (`self` instead of `&mut self`), a wrong parameter type, a wrong return type — the harness binds each method to a typed function pointer
- a missing `VaultError` variant, or an enum without `#[error_code]`
- items you left unreachable from the crate root

**It does not catch — verified, by compiling each of these clean:**

- a `withdraw` that calls `checked_add`. Every signature still matches and the build goes green with a vault that pays you for withdrawing.
- a dropped `require!(amount > 0, …)`. The zero guard is invariant 1 and no type changes when it is gone.
- `self.balance = self.balance.checked_sub(amount).unwrap();` — invariant 4, and a panic is not a type.
- `self.balance = self.balance - amount;` — bare arithmetic, same reason.

Which means exercise 1's pool is not policed either: two of its seven lines do not belong, and both of the two compile. "Two do not belong" is a claim about the spec, not about the build. Picking them out is the exercise; the grader is not scoring it.

That is a property of compile-only grading, not something to work around, and pretending otherwise would be worse than saying it. The thing that catches a swapped operator is a test that runs `withdraw` and looks at the number — which is Course 3, with LiteSVM. Until then it is you, rereading the file, which is what lesson 14 is for.

## Order of work

Exercise 1: assemble the two bodies and the re-export from the pool. Exercise 2: write all of it from this page and the harness, with nothing above the harness line but your own typing.

One warning about that second block, because the platform will not give it to you. Exercise 1 is on the same page, one block above, and by the time you reach exercise 2 it contains the finished `VaultState`, the finished `VaultError`, and — in its pool — every line of both method bodies. **If you scroll up, you are typing, not writing.** Nothing stops you and nothing detects it; the surface Course 3 needs is fixed, so exercise 2 cannot ask for anything exercise 1 did not show you.

The version of this that is worth your time: read the spec and the harness, close the lesson, write the file from the signatures, and only then scroll up to compare. Getting it wrong first and finding out where is the entire mechanism by which the second block is worth more than the first. Copying it is a green build and nothing else.
