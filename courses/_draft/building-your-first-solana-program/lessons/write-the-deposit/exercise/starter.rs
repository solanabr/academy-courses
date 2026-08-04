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

    // Finished in Module 2. Leave it alone.
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = ctx.accounts.user.key();
        vault.balance = 0;
        vault.bump = ctx.bumps.vault;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // ── SUBGOAL 2: move the lamports ────────────────────────────────────
        //
        // Ask the System Program to transfer `amount` from the user to the
        // vault. Two things to get right:
        //
        //   * the first argument to `CpiContext::new` is a program id — a
        //     `Pubkey`. Pass an `AccountInfo` and you get
        //     `error[E0308]: expected Pubkey, found AccountInfo<'_>`.
        //   * `Transfer { from, to }` takes `AccountInfo`s, so both accounts
        //     need `.to_account_info()`. The user is the source.
        //
        // Propagate failure with `?` — never `.unwrap()`.

        // ── SUBGOAL 3: update the balance using YOUR Course 2 method ────────
        //
        // One line. `ctx.accounts.vault` derefs to your `VaultState`, so the
        // method you wrote and reasoned about in Course 2 is callable on it directly.
        //
        // Do NOT retype the arithmetic. If you are writing `checked_add` or
        // `+= amount` in this file, you are shipping a second copy of a function
        // you already own — one more place for the guard to go missing.

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

// ── SUBGOAL 1: the accounts struct ──────────────────────────────────────────
//
// Three accounts. One is pre-filled. Add the other two, in this order:
//
//   1a. `vault` — `Account<'info, VaultState>`. Mutable (both its lamports and
//       its data change), re-derived from `[b"vault", user.key().as_ref()]`,
//       and using the bump you STORED rather than searching for it again.
//
//   1b. `user`  — `Signer<'info>`. Mutable: no rent is paid here, but lamports
//       leave this wallet, and any account whose lamports change must be `mut`.
//
#[derive(Accounts)]
pub struct Deposit<'info> {
    // Still required even though `CpiContext::new` no longer takes it: the
    // runtime must have the callee's program account loaded, and declaring it
    // here is what makes the client include it in the transaction.
    pub system_program: Program<'info, System>,
}

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
//
// This compiles only if `deposit` and `Deposit` exist with exactly the names,
// field names and types the subgoals ask for. A missing field is
// `error[E0609]: no field ... on type &Deposit<'_>`; a wrong type is
// `error[E0308]: mismatched types`; a missing item is `error[E0433]`.
//
// It is a TYPE check, not a behaviour check. Ways to be wrong and still be
// green, each one confirmed by compiling it:
//   * an empty `deposit` body — only the signature is pinned
//   * `from` and `to` swapped, so the vault pays the user
//   * `ctx.accounts.vault.balance += amount` instead of the Course 2 method
//   * a bare `bump` instead of `bump = vault.bump`, which costs compute units
// The build server compiles; it does not run, and neither does anything else in
// this course. Module 4 shows you the LiteSVM test that would catch these three,
// and why compiling cannot.
// ─────────────────────────────────────────────────────────────────────────────
#[doc(hidden)]
#[allow(dead_code)]
mod verify {
    use super::*;

    // `deposit` takes a `Deposit` context plus a `u64` and returns `Result<()>`.
    const DEPOSIT: for<'info> fn(Context<'info, Deposit<'info>>, u64) -> Result<()> =
        vault_program::deposit;

    // `Deposit` carries all three accounts, each with the right type.
    fn pin_deposit_accounts(a: &Deposit) {
        let _: &Account<VaultState> = &a.vault;
        let _: &Signer = &a.user;
        let _: &Program<System> = &a.system_program;
    }

    // The Course 2 core is intact and still has the method subgoal 3 calls.
    const CORE_DEPOSIT: fn(&mut VaultState, u64) -> Result<()> = VaultState::deposit;
    const _: () = assert!(VaultState::INIT_SPACE == 41);
}
