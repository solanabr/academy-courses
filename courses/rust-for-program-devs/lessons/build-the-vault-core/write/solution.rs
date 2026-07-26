// Exercise 2 of 2 — the vault core, written from the spec.
//
// This is one correct shape, not the only one: the fields may be declared in
// any order, the `#[msg]` strings are yours, and `require!` could be a plain
// `if amount == 0 { return err!(VaultError::ZeroAmount); }`. What is fixed is
// the surface the harness names — and the operator in each method, which the
// harness cannot see and Course 3's tests can.

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
