use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ─── Your Course 2 core, imported unchanged. Do not edit it. ─────────────────
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

#[program]
pub mod vault_program {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = ctx.accounts.user.key();
        vault.balance = 0;
        vault.bump = ctx.bumps.vault;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // SUBGOAL 2 — the lamports. Program id first (a `Pubkey`), accounts
        // second, amount last. The user's transaction signature is what the
        // System Program checks; this program signs nothing.
        transfer(
            CpiContext::new(
                System::id(),
                Transfer {
                    from: ctx.accounts.user.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                },
            ),
            amount,
        )?;

        // SUBGOAL 3 — the bookkeeping, in one line, by calling Course 2.
        // The zero-amount guard, the `checked_add` and `VaultError::Overflow`
        // all live inside this method. Nothing is retyped here.
        ctx.accounts.vault.deposit(amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8 + 1,
        seeds = [b"vault", user.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, VaultState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// SUBGOAL 1 — the accounts struct.
#[derive(Accounts)]
pub struct Deposit<'info> {
    // `mut`: both the lamports and the data change.
    // `bump = vault.bump`: one hash, using the byte stored at initialize time,
    // instead of searching from 255 down for a canonical bump we already have.
    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, VaultState>,

    // `mut` because lamports leave this wallet — not because rent is paid.
    // `Signer` because the System Program will not debit an account that did
    // not sign, and that signature is the only authorisation in the instruction.
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
//
// A TYPE check, not a behaviour check. Ways to be wrong and still be green,
// each one confirmed by compiling it: an empty `deposit` body; `from` and `to`
// swapped; `balance += amount` instead of the Course 2 method; a bare `bump`
// instead of `bump = vault.bump`. The build server compiles, it does not run.
// ─────────────────────────────────────────────────────────────────────────────
#[doc(hidden)]
#[allow(dead_code)]
mod verify {
    use super::*;

    const DEPOSIT: for<'info> fn(Context<'info, Deposit<'info>>, u64) -> Result<()> =
        vault_program::deposit;

    fn pin_deposit_accounts(a: &Deposit) {
        let _: &Account<VaultState> = &a.vault;
        let _: &Signer = &a.user;
        let _: &Program<System> = &a.system_program;
    }

    const CORE_DEPOSIT: fn(&mut VaultState, u64) -> Result<()> = VaultState::deposit;
    const _: () = assert!(VaultState::INIT_SPACE == 41);
}
