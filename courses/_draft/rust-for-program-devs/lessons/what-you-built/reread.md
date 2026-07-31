# What You Built

> **Version stamp — checked 2026-07-26.** `anchor-lang 1.1.2` · `borsh 1.8.0` (resolved) · edition 2021 · Agave ≥ 3.1.10. No new code in this lesson; every claim below was compiled against that toolchain on that date.

Open your file from the last exercise. Not the reference below — yours. Read it once, top to bottom, before you read anything else on this page.

Here is the reference version, so you can check yours item for item:

```rust
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub mod vault {
    use super::*;

    #[account]
    #[derive(InitSpace)]
    pub struct VaultState {
        pub owner: Pubkey,
        pub balance: u64,
        pub bump: u8,
    }

    #[error_code]
    pub enum VaultError {
        #[msg("Deposit would overflow the vault balance")]
        Overflow,
        #[msg("Withdrawal is larger than the vault balance")]
        InsufficientFunds,
        #[msg("Amount must be greater than zero")]
        ZeroAmount,
        #[msg("Signer is not the vault owner")]
        NotOwner,
    }

    impl VaultState {
        pub fn deposit(&mut self, amount: u64) -> Result<()> {
            require!(amount > 0, VaultError::ZeroAmount);
            self.balance = self
                .balance
                .checked_add(amount)
                .ok_or(VaultError::Overflow)?;
            Ok(())
        }

        pub fn withdraw(&mut self, amount: u64) -> Result<()> {
            require!(amount > 0, VaultError::ZeroAmount);
            self.balance = self
                .balance
                .checked_sub(amount)
                .ok_or(VaultError::InsufficientFunds)?;
            Ok(())
        }
    }
}

pub use vault::{VaultError, VaultState};
```

Forty-odd lines. Thirteen lessons. Every line came from somewhere specific, and being able to say where is the difference between having finished a course and being able to write the next file without one.

## Line by line

| Line                                        | What it actually is                                                                                                                                                                                                    | Where you got it                                    |
| ------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| `use anchor_lang::prelude::*;`              | One glob import supplies `Pubkey`, `Result`, `require!`, `msg!`. Anchor re-exports the Solana crates, which is why you never named a `solana-*` dependency.                                                              | L1 · Your First Rust Build                          |
| `declare_id!("Fg6Pa…")`                     | Not decoration. `#[account]` expands to an `impl Owner` whose body reads `crate::ID`. Delete the macro and the error appears at your struct: `cannot find value ID in the crate root`.                                   | L1, and paid for at L10                             |
| `pub mod vault { use super::*; }`            | A module is a namespace, not a file. `use super::*` is what makes the crate root's imports visible inside the module body.                                                                                              | L13 · Build the Vault Core                          |
| `#[account]`                                | A macro that writes trait impls you would otherwise hand-write: `AnchorSerialize`, `AnchorDeserialize`, `Discriminator`, `Owner`. The discriminator is the 8 bytes your Course 1 decoder sliced off before reading `owner`. | L10 · Structs, Traits, and What `#[derive]` Really Does |
| `#[derive(InitSpace)]`                      | Generates the associated const `VaultState::INIT_SPACE`. Here it is **41**: 32 + 8 + 1. Course 3 spends it as `space = 8 + VaultState::INIT_SPACE`.                                                                      | L10                                                 |
| `pub owner: Pubkey`                         | 32 bytes. The same 32 bytes you read with `getAddressDecoder()` in Course 1, now on the writing side.                                                                                                                    | L2 · C1 L2                                          |
| `pub balance: u64`                          | Lamports stop being exact in a JavaScript `Number` above 2⁵³. This is the type for which that is not a question.                                                                                                        | L2 · Types, Mutability, and Why the Compiler Argues |
| `pub bump: u8`                              | One byte, and a *stored* bump rather than a recomputed one. It is the value the derivation in Course 1 found by counting down from 255 until the result fell off the curve.                                               | L2 · C1 L3                                          |
| `#[error_code]`                             | Turns a plain enum into program errors: attaches the `#[msg]` strings and generates the conversion into `anchor_lang::error::Error`, offsetting the codes by `ERROR_CODE_OFFSET` = 6000. Exactly one of these per program. | L11 · Enums, `Option`, and `Result`                 |
| the four variants                           | On the wire: `Overflow` = 6000, `InsufficientFunds` = 6001, `ZeroAmount` = 6002, `NotOwner` = 6003. Custom errors start at 6000 so they cannot collide with Anchor's own.                                                | L11                                                 |
| `impl VaultState`                           | A method is just a function whose first parameter is a form of `self`.                                                                                                                                                  | L10                                                 |
| `&mut self`                                 | One mutable borrow, and no shared borrows alongside it — the same shape as one writable account at a time. `self` by value would consume the account.                                                                    | L6 · Borrowing, L5 · Moves                          |
| `-> Result<()>`                             | Anchor's alias for `Result<(), anchor_lang::error::Error>`. The reason the function can fail without panicking.                                                                                                          | L12 · Error Plumbing with `?`                       |
| `require!(amount > 0, …)`                    | Guard first, mutate second. It expands to an early `return Err(...)`, which is why it reads like an assertion and behaves like a branch.                                                                                | L12                                                 |
| `.checked_add(…)` / `.checked_sub(…)`        | Returns `Option<u64>` — `None` instead of wrapping or panicking. This is lesson 4's `add_lamports` / `sub_lamports`, moved onto a method.                                                                                | L4 · Checked Lamport Math                           |
| `.ok_or(VaultError::Overflow)?`             | `ok_or` converts `Option` into `Result`; `?` either unwraps the `Ok` or returns early, applying the `From` conversion `#[error_code]` generated. Two lessons meeting inside one expression.                              | L11 (`Option`) + L12 (`?`)                          |
| `Ok(())`                                    | A tail expression: no `return`, no semicolon. Add the semicolon and the body evaluates to `()`, and the error is `mismatched types`.                                                                                     | L3 · Functions, Expressions, and the Missing `return` |
| `pub use vault::{VaultError, VaultState};`   | A re-export. Without it the items exist and nothing outside the module can name them.                                                                                                                                   | L13                                                 |

## Three absences, which are also the point

**`.clone()` appears zero times.** Lesson 5 was explicit that cloning is a legitimate escape hatch and not a moral failing. It is also true that in a `&mut self` method you never reach for it, because you never moved anything: you borrowed the account, wrote through the borrow, and gave it back.

**`unwrap()` appears zero times.** Every tutorial you will read on the way to your first real program uses it. In a program, a panic is an aborted instruction with no code and no message — the caller learns that something failed and nothing else. Your file returns 6001 with the string "Withdrawal is larger than the vault balance". That difference is the whole of lessons 4, 11 and 12.

**Lifetimes appear zero times**, and lesson 8 was not wasted on you. `VaultState` owns all three of its fields — a `Pubkey`, a `u64` and a `u8` are all owned values — so nothing in the struct borrows and there is no lifetime to name. What lesson 8 bought you is the *next* file: `Account<'info, VaultState>` in Course 3 is a borrowed view over these bytes, and `'info` is the same annotation you wrote by hand on `discriminator(data: &'a [u8]) -> Option<&'a [u8]>`. Graduates who skip lifetimes read `Account<'info, Vault>` as noise for years. Question 6 below is that payoff, asked before you meet it.

## Copy the file out. Now.

Say this plainly, because the platform does not: **no block type on this platform persists your source file.** The build server graded a submission; it did not keep a copy you can come back to. Select everything in the editor, save it locally as `vault_core.rs`, and put it somewhere you will find it next week.

If you lose it, Course 3's first lesson ships a known-good reference copy — the file above — so nothing dead-ends. But it will be *a* vault core, not yours, and the first thing Course 3 does is replace that placeholder in `declare_id!` with the program id of your own deployment.
