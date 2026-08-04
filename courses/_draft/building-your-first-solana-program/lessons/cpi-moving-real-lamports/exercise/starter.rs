// ─────────────────────────────────────────────────────────────────────────────
// WORKED EXAMPLE — nothing to fix. Build it, then read it.
//
// This is your vault after Module 2, plus one new instruction: `deposit`.
// It compiles as shipped. The point of the exercise is the reading, so if you
// change something and it breaks, `git`-less as you are, just reload the lesson.
// ─────────────────────────────────────────────────────────────────────────────

use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ─── Your Course 2 core, imported unchanged ──────────────────────────────────
// Not one line of this module is new. It is the file you finished Course 2 with:
// a struct that knows its own size, two methods that cannot silently overflow,
// and one error enum. Nothing in it knows what an account is.
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
        // ── 1. The lamports. This is the CPI. ────────────────────────────────
        //
        // `transfer` here is `anchor_lang::system_program::transfer` — a thin
        // wrapper that builds a System Program instruction and invokes it.
        //
        // First argument to `CpiContext::new` is a `Pubkey`: WHICH program.
        // `System::id()` is a compile-time constant, so no account is read to
        // build this. Anchor 1.0 changed this argument from an `AccountInfo`;
        // passing one now is `error[E0308]: expected Pubkey, found AccountInfo`.
        //
        // The user signed the transaction, and that signature is still valid
        // this far down the call stack. Our program signs nothing.
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

        // ── 2. The bookkeeping. This is your Course 2 method. ────────────────
        //
        // Imported, not retyped. `checked_add`, the `ZeroAmount` guard and the
        // `Overflow` error all live in `vault::VaultState`, already written and
        // already reasoned about. If `amount` is 0 this returns before the
        // balance changes — and because the CPI above already ran, the whole
        // transaction is rolled back. Failure is all-or-nothing per instruction.
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

#[derive(Accounts)]
pub struct Deposit<'info> {
    // `mut` because the lamports AND the data both change.
    //
    // `bump = vault.bump` re-derives the address using the bump you stored at
    // initialize time — one hash. A bare `bump` would make Anchor search from
    // 255 downwards for the canonical bump, which costs compute units for an
    // answer the account is already carrying.
    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, VaultState>,

    // `mut` because lamports leave this account. `Signer` because the System
    // Program will not debit an account that did not sign — and that signature
    // is the only authorisation in the whole instruction.
    #[account(mut)]
    pub user: Signer<'info>,

    // Required even though `CpiContext::new` no longer takes it. The runtime
    // must have the callee's program account loaded; declaring it here is what
    // makes the client include it in the transaction.
    pub system_program: Program<'info, System>,
}
