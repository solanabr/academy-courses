// Exercise 1 of 2 — complete the impl block.
//
// VaultState, VaultError and the two method signatures are already written.
// Below are seven candidate lines. FIVE of them belong in this file. The other
// two compile and are still wrong — the grader cannot tell the difference, which
// is the point of asking you. Two of the five are used TWICE. Order matters
// inside a body.
//
//   (1)  Ok(())
//   (2)  self.balance = self.balance.checked_sub(amount).unwrap();
//   (3)  self.balance = self.balance.checked_add(amount).ok_or(VaultError::Overflow)?;
//   (4)  pub use vault::{VaultError, VaultState};
//   (5)  self.balance = self.balance - amount;
//   (6)  require!(amount > 0, VaultError::ZeroAmount);
//   (7)  self.balance = self.balance.checked_sub(amount).ok_or(VaultError::InsufficientFunds)?;
//
// This file does NOT compile as shipped. Build it first. The errors arrive in
// two groups, and each group points at a different missing line.

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
            // Three lines from the pool, in order.
        }

        pub fn withdraw(&mut self, amount: u64) -> Result<()> {
            // Three lines from the pool, in order.
        }
    }
}

// One line from the pool goes here.

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
