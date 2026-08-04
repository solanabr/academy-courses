// Exercise complete.
//
// Pool order is (1), (2), (3), (4), (5), (6), (7), (8). Lines (9) and (10)
// belong nowhere:
//
//   (9)  is the CPI everyone reaches for. Give it a `signer_seeds` binding and
//        it compiles, deploys, and then returns `Transfer: 'from' must not carry
//        data` — the System Program refuses to debit any account with a
//        non-empty data field, and it does not own our vault anyway. Signer
//        seeds solve authorisation; authorisation was never the obstacle.
//
//  (10)  drops the rent floor and does bare subtraction. Under this crate's
//        `overflow-checks = true` it panics instead of returning
//        `InsufficientFunds`, and even when it does not underflow it will happily
//        take the vault below its rent-exempt minimum, failing the entire
//        transaction with `InsufficientFundsForRent`.
//
// Note what is NOT in this file: no `checked_sub` on `balance` (that is the
// Course 2 method), no second `#[error_code]` enum, and no `system_program` in
// `Withdraw`, because there is no CPI to make.

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

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault_info = ctx.accounts.vault.to_account_info();
        let user_info = ctx.accounts.user.to_account_info();

        // (1) Bookkeeping first, and it is the Course 2 method — the zero guard,
        // the `checked_sub` and `InsufficientFunds` all live inside it. If the
        // vault's recorded balance cannot cover `amount`, nothing below runs.
        ctx.accounts.vault.withdraw(amount)?;

        // (2) The floor. `data_len()` reads the account's real allocated size,
        // so this stays correct if `VaultState` ever grows. Never hardcode 49.
        let rent_floor = Rent::get()?.minimum_balance(vault_info.data_len());

        // (3) What the vault would hold afterwards, computed without trusting
        // subtraction not to underflow.
        let remaining = vault_info
            .lamports()
            .checked_sub(amount)
            .ok_or(VaultError::InsufficientFunds)?;

        // (4) The guard, before a single lamport moves. Below the floor, the
        // runtime refuses the whole transaction with `InsufficientFundsForRent`
        // — so we refuse it first, with an error the caller can read.
        require!(remaining >= rent_floor, VaultError::InsufficientFunds);

        // (5) Our program owns the vault, so our program may debit it. No CPI,
        // nothing to sign.
        **vault_info.try_borrow_mut_lamports()? = remaining;

        // (6) and (7) The credit, also checked.
        let credited = user_info
            .lamports()
            .checked_add(amount)
            .ok_or(VaultError::Overflow)?;
        **user_info.try_borrow_mut_lamports()? = credited;

        // (8)
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

// GIVEN. Two things to notice before you start.
//
//   * `constraint = vault.owner == user.key()` is what finally spends the
//     `owner` field you have been carrying since Course 2, and it returns
//     `NotOwner` — the fourth variant, previously unused.
//   * there is no `system_program` here. `Withdraw` makes no CPI, so there is no
//     callee program for the runtime to load. If your answer needs it, your
//     answer is the wrong one.
#[derive(Accounts)]
pub struct Withdraw<'info> {
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

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
//
// It pins the signature of `withdraw` and the fields of `Withdraw`, and nothing
// else. It is a TYPE check, not a behaviour check. Ways to be wrong and still be
// green, each one confirmed by compiling it:
//   * pool line (9), which needs no more than a `signer_seeds` binding to
//     compile, deploys fine, and returns `Transfer: 'from' must not carry data`
//     the first time anyone calls it
//   * pool line (10), which drops the rent floor and panics under
//     `overflow-checks = true` instead of returning `InsufficientFunds`
//   * crediting the user without debiting the vault, which the runtime catches
//     at the end of the instruction — but not until it runs
// The build server compiles; it does not run. Module 4 shows you the LiteSVM
// test that would execute this program and catch the above, and why compiling
// cannot — read it before any devnet SOL is spent.
// ─────────────────────────────────────────────────────────────────────────────
#[doc(hidden)]
#[allow(dead_code)]
mod verify {
    use super::*;

    const WITHDRAW: for<'info> fn(Context<'info, Withdraw<'info>>, u64) -> Result<()> =
        vault_program::withdraw;

    fn pin_withdraw_accounts(a: &Withdraw) {
        let _: &Account<VaultState> = &a.vault;
        let _: &Signer = &a.user;
    }

    // The Course 2 core is intact and still owns the arithmetic.
    const CORE_WITHDRAW: fn(&mut VaultState, u64) -> Result<()> = VaultState::withdraw;

    // All four variants still exist, including the one this lesson finally uses.
    fn errors() -> [anchor_lang::error::Error; 4] {
        [
            VaultError::Overflow.into(),
            VaultError::InsufficientFunds.into(),
            VaultError::ZeroAmount.into(),
            VaultError::NotOwner.into(),
        ]
    }
}
