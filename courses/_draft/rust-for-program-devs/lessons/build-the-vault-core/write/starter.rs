// Exercise 2 of 2 — write the vault core.
//
// Nothing above the harness line is written for you. The spec is on the lesson
// page; the harness below is the same spec expressed in Rust, and it is the
// only thing the build server can check. Read it before you type.
//
// Exercise 1 is one block above this one and it contains the answer, pool lines
// and all. Nothing stops you scrolling up and nothing detects it — but a file you
// transcribed teaches you nothing a file you wrote does not. Write it from the
// harness first, compare afterwards.
//
// This file does NOT compile as shipped. Every error you see names something
// you still owe.

use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub mod vault {
    use super::*;

    // TODO 1 — `VaultState`: an `#[account]` struct that also derives
    //          `InitSpace`, with three public fields.
    //
    // TODO 2 — `VaultError`: an `#[error_code]` enum with four variants, each
    //          carrying a `#[msg("…")]` a caller can read.
    //
    // TODO 3 — `impl VaultState`, with `deposit` and `withdraw`. Reject a zero
    //          amount before writing anything, do the arithmetic with
    //          `checked_add` / `checked_sub`, and turn `None` into the right
    //          named variant. No `unwrap()`, no `expect()`, no bare `+`/`-`.
}

// TODO 4 — one line: make `VaultState` and `VaultError` nameable from the
//          crate root, where the harness can see them.

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
//
// This module compiles only if every item the spec asks for exists with the
// name, shape and signature the spec gives. It is a TYPE check, not a behaviour
// check. Four ways to be wrong and still green, each one confirmed by compiling
// it: `checked_add` where `checked_sub` belongs, a dropped `require!(amount > 0)`,
// `.unwrap()` instead of `.ok_or(…)?`, and bare `+`/`-` on `self.balance`.
// Nothing below can see any of them.
// ─────────────────────────────────────────────────────────────────────────────
#[doc(hidden)]
#[allow(dead_code)]
mod verify {
    use super::*;

    // `VaultState` exists with exactly these three field names and types.
    fn state(owner: Pubkey, balance: u64, bump: u8) -> VaultState {
        VaultState {
            owner,
            balance,
            bump,
        }
    }

    // `#[account]` is present — it is what supplies the 8-byte discriminator.
    const _: () = assert!(VaultState::DISCRIMINATOR.len() == 8);

    // `#[derive(InitSpace)]` is present, and the fields are exactly
    // Pubkey (32) + u64 (8) + u8 (1). Widen any of them and this trips.
    const _: () = assert!(VaultState::INIT_SPACE == 41);

    // Both methods take `&mut self` and a `u64`, and return `Result<()>`.
    const DEPOSIT: fn(&mut VaultState, u64) -> Result<()> = VaultState::deposit;
    const WITHDRAW: fn(&mut VaultState, u64) -> Result<()> = VaultState::withdraw;

    // All four variants exist and the enum carries `#[error_code]`, which is
    // what gives it a conversion into `anchor_lang::error::Error`.
    fn errors() -> [anchor_lang::error::Error; 4] {
        [
            VaultError::Overflow.into(),
            VaultError::InsufficientFunds.into(),
            VaultError::ZeroAmount.into(),
            VaultError::NotOwner.into(),
        ]
    }
}
