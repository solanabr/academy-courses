// From Rust core to Anchor — nothing to write. Click Build.
//
// This file is your Course 2 `vault_core.rs`, unchanged, with the Anchor wrapper
// added underneath it. The two halves are separated by a labeled line so you can
// see exactly where your code stops and this course's material starts.
//
// If you did not take Course 2, this is the reference version — it is a correct
// vault core, and starting here is a supported path.

// ─────────────────────────────────────────────────────────────────────────────
// YOUR COURSE 2 FILE — copied in verbatim, from the `use` line to the re-export.
// Not one character of it changes in this course.
// ─────────────────────────────────────────────────────────────────────────────
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
// EVERYTHING BELOW THIS LINE IS WHAT COURSE 3 ADDS.
//
// Two items, and they are the two your Course 2 file was missing:
//
//   #[program]          — turns a plain Rust module into a set of instructions
//                         the Solana runtime can dispatch to. This is what makes
//                         the crate a *program* and not a library.
//   #[derive(Accounts)] — declares which accounts an instruction may touch.
//                         `DryRun` declares none, and that is the entire reason
//                         nothing below persists.
//
// Read the body of `dry_run`: it builds a `VaultState`, calls the `deposit` you
// wrote in Course 2, and logs the result. The method is imported, not retyped.
// That is the pattern for the rest of the course — a handler validates accounts
// and then calls core logic that was already correct.
// ─────────────────────────────────────────────────────────────────────────────
#[program]
pub mod vault_program {
    use super::*;

    pub fn dry_run(_ctx: Context<DryRun>) -> Result<()> {
        // Not an account — a value on this handler's stack, alive for exactly
        // as long as this instruction runs.
        let mut vault = VaultState {
            owner: Pubkey::default(),
            balance: 0,
            bump: 0,
        };

        // Your Course 2 method, called unmodified. `?` needs no conversion here
        // because `deposit` already returns `anchor_lang::Result<()>`.
        vault.deposit(1_000)?;

        msg!("program id: {}", ID);
        msg!("balance after deposit: {}", vault.balance);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct DryRun {}
