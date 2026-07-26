// The finished vault — the deploy artifact, in one file. Nothing here is new.
// Read it once to confirm you recognise all of it, then click Build, then
// Deploy to Devnet.
//
// WHAT SHIPS, AND WHAT DOES NOT. Three instructions go on-chain:
// `initialize_vault`, `deposit` and `withdraw` — the three that move lamports
// or create state, and the three Course 4 calls against this program id. Your
// `transfer_between_vaults` from `harden-the-vault` is deliberately NOT here:
// it moves recorded balance between two vaults and touches no lamports, so it
// adds nothing to the artifact the next course consumes. It was an exercise in
// writing constraints, not a feature of the vault. If you want it deployed,
// add it back along with its entry in the IDL — nothing stops you.
//
// Version stamp — checked 2026-07-26: anchor-lang 1.1.2, borsh resolves to
// 1.8.0, edition 2021, Agave >= 3.1.10, rustc 1.89+.

use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

// The one line that looks optional and is not: `#[account]` expands to an
// `impl Owner for VaultState` whose body reads `crate::ID`, and this is what
// defines `ID`. Deploying replaces this placeholder with your own program id.
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ─────────────────────────────────────────────────────────────────────────────
// YOUR COURSE 2 CORE — imported, not retyped.
//
// This module is the file you wrote in Course 2, unchanged. It has no accounts,
// no signers, no CPI: it is the half of a program that does not depend on the
// runtime. Every checked_add / checked_sub / named-error decision in it is one
// you already made and already defended.
// ─────────────────────────────────────────────────────────────────────────────
pub mod vault {
    use super::*;

    #[account]
    #[derive(InitSpace)]
    pub struct VaultState {
        pub owner: Pubkey,
        pub balance: u64,
        pub bump: u8,
    }

    // Exactly ONE `#[error_code]` enum per program — Anchor 1.0 permits no more
    // than one, and this is it. Module 3 added no new error type; it added a
    // constraint that raises `NotOwner`, which Course 2 declared and left unused
    // for precisely that reason.
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
// THE HALF COURSE 3 ADDED: the runtime-facing program.
// ─────────────────────────────────────────────────────────────────────────────
#[program]
pub mod vault_program {
    use super::*;

    /// Module 2. Creates the per-user vault PDA and stores the canonical bump.
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = ctx.accounts.user.key();
        vault.balance = 0;
        // `ctx.bumps` is keyed by the FIELD name in the accounts struct, not by
        // the seed prefix. Store it now so no later instruction has to search.
        vault.bump = ctx.bumps.vault;
        msg!("Vault initialized for {}", vault.owner);
        Ok(())
    }

    /// Module 3. Lamports IN move via a System Program CPI, because the debited
    /// account is the user's wallet and System owns that. The recorded balance
    /// moves via your Course 2 method. Two writes that have to agree.
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // Anchor 1.0: `CpiContext::new` takes the program's ID, not its
        // AccountInfo. Every pre-1.0 CPI tutorial online has the old shape.
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
        ctx.accounts.vault.deposit(amount)?;
        Ok(())
    }

    /// Module 3, and the instruction with no CPI in it. The debited account is
    /// the vault, which OUR program owns — so our program may move its lamports
    /// directly. The System Program could not do it for us at any price: it
    /// refuses to debit an account carrying data, and it does not own this one.
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault_info = ctx.accounts.vault.to_account_info();
        let user_info = ctx.accounts.user.to_account_info();

        // Bookkeeping first, through the Course 2 method: the zero guard, the
        // `checked_sub` and `InsufficientFunds` all live inside it.
        ctx.accounts.vault.withdraw(amount)?;

        // The rent floor, read from the account's real allocated size so it stays
        // correct if `VaultState` ever grows. Never hardcode 49.
        let rent_floor = Rent::get()?.minimum_balance(vault_info.data_len());
        let remaining = vault_info
            .lamports()
            .checked_sub(amount)
            .ok_or(VaultError::InsufficientFunds)?;
        require!(remaining >= rent_floor, VaultError::InsufficientFunds);

        **vault_info.try_borrow_mut_lamports()? = remaining;
        let credited = user_info
            .lamports()
            .checked_add(amount)
            .ok_or(VaultError::Overflow)?;
        **user_info.try_borrow_mut_lamports()? = credited;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    // 8 (discriminator) + 32 (owner) + 8 (balance) + 1 (bump) = 49, derived in
    // module 2 rather than copied. No `rent` constraint — removed in 0.31.
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
    // `bump = vault.bump` uses the stored canonical bump: one hash instead of a
    // search from 255 down for a value you already wrote to the account.
    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, VaultState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // The `constraint` is what finally spends the `owner` field carried since
    // Course 2, and it raises `NotOwner` — the fourth variant, previously unused.
    // Note what is absent: no `system_program`, because `withdraw` makes no CPI
    // and there is no callee program for the runtime to load.
    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump,
        constraint = vault.owner == user.key() @ VaultError::NotOwner
    )]
    pub vault: Account<'info, VaultState>,
    #[account(mut)]
    pub user: Signer<'info>,
}
