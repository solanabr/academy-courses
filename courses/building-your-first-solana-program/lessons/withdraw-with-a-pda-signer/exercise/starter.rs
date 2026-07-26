// Complete `withdraw`.
//
// The accounts struct is written. The two `AccountInfo` bindings are written.
// Below are ten candidate lines. EIGHT of them belong in this file, in an order
// you have to work out. The other two compile — or all but compile — and are
// still wrong, and a compile-only grader cannot tell the difference. That is the
// point of asking you.
//
//   (1)  ctx.accounts.vault.withdraw(amount)?;
//   (2)  let rent_floor = Rent::get()?.minimum_balance(vault_info.data_len());
//   (3)  let remaining = vault_info.lamports().checked_sub(amount).ok_or(VaultError::InsufficientFunds)?;
//   (4)  require!(remaining >= rent_floor, VaultError::InsufficientFunds);
//   (5)  **vault_info.try_borrow_mut_lamports()? = remaining;
//   (6)  let credited = user_info.lamports().checked_add(amount).ok_or(VaultError::Overflow)?;
//   (7)  **user_info.try_borrow_mut_lamports()? = credited;
//   (8)  Ok(())
//   (9)  transfer(CpiContext::new_with_signer(System::id(), Transfer { from: vault_info.clone(), to: user_info.clone() }, signer_seeds), amount)?;
//  (10)  **vault_info.try_borrow_mut_lamports()? -= amount;
//
// Order matters. Three of the eight read a value another one of them has to have
// produced first, and the guard has to run before any lamport is touched.
//
// This file does NOT compile as shipped. Build it first — an empty body
// evaluates to `()`, and the signature promises `Result<()>`.

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

        // Eight lines from the pool, in order.
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
