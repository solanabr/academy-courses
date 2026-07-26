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

        // Finished in the previous lesson. Bookkeeping via the Course 2 method,
        // then a rent-floor check, then a direct debit — no CPI, because a
        // program may move the lamports of an account it owns.
        ctx.accounts.vault.withdraw(amount)?;

        let rent_floor = Rent::get()?.minimum_balance(vault_info.data_len());
        let remaining = vault_info
            .lamports()
            .checked_sub(amount)
            .ok_or(VaultError::InsufficientFunds)?;
        require!(remaining >= rent_floor, VaultError::InsufficientFunds);

        let credited = user_info
            .lamports()
            .checked_add(amount)
            .ok_or(VaultError::Overflow)?;
        **vault_info.try_borrow_mut_lamports()? = remaining;
        **user_info.try_borrow_mut_lamports()? = credited;

        Ok(())
    }

    pub fn transfer_between_vaults(
        ctx: Context<TransferBetweenVaults>,
        amount: u64,
    ) -> Result<()> {
        // Debit first. If the signer's vault cannot cover `amount`, the Course 2
        // method returns `InsufficientFunds` and nothing is credited.
        ctx.accounts.from_vault.withdraw(amount)?;
        ctx.accounts.to_vault.deposit(amount)?;
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

// Finished in the previous lesson. No `system_program`: no CPI is made here.
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

// Attack (a): `seeds` re-derives the address from the SIGNER's key, so the only
// account that can satisfy it is the signer's own vault, and the explicit
// `constraint` re-asks the same question against the stored `owner` field —
// returning our error rather than Anchor's generic 2001/2006.
//
// Attack (b): `Account<'info, VaultState>` checks program ownership, the 8-byte
// discriminator and initialisation on every deserialisation.
//
// Attack (c): no `dup`. Both vaults are mutable, so if `recipient` is the signer
// the two slots resolve to one key and Anchor 1.0 refuses the instruction with
// error 2040 before the body runs. Adding `dup` here would re-open exactly the
// double-count bug 2040 is reporting: two copies of one account, one
// subtraction, one addition, last write wins.
#[derive(Accounts)]
pub struct TransferBetweenVaults<'info> {
    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = from_vault.bump,
        constraint = from_vault.owner == user.key() @ VaultError::NotOwner
    )]
    pub from_vault: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", recipient.key().as_ref()],
        bump = to_vault.bump
    )]
    pub to_vault: Account<'info, VaultState>,

    pub user: Signer<'info>,

    /// CHECK: read only as a PDA seed. Never written, never deserialised, and
    /// never trusted for authorisation — `user` is the only signer here.
    pub recipient: UncheckedAccount<'info>,
}

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
//
// This is the acceptance criteria in Rust. It compiles only if the instruction
// and the accounts struct exist with the names, order and types the spec gives.
// A missing item is `error[E0433]` or `error[E0412]`; a missing field is
// `error[E0609]`; a wrong type is `error[E0308]`.
//
// It is a TYPE check. It proves attack (b) is handled — `Account<'info, VaultState>`
// carries the owner, discriminator and initialisation checks, and an
// `UncheckedAccount` in either vault slot fails to compile here. It can see
// nothing about attacks (a) or (c): seeds, the ownership constraint and the
// duplicate-mutable-account default are all runtime behaviour. Confirmed ways to
// be green and still be wrong: no `seeds`/`bump` at all, no owner constraint, a
// bare `bump` instead of the stored one, `dup` added to silence error 2040, and
// the two body calls in the wrong order.
// ─────────────────────────────────────────────────────────────────────────────
#[doc(hidden)]
#[allow(dead_code)]
mod verify {
    use super::*;

    const TRANSFER_BETWEEN: for<'info> fn(
        Context<'info, TransferBetweenVaults<'info>>,
        u64,
    ) -> Result<()> = vault_program::transfer_between_vaults;

    fn pin_accounts(a: &TransferBetweenVaults) {
        // Both vault slots are typed accounts, not unchecked ones. This is the
        // whole of attack (b), enforced by the compiler.
        let _: &Account<VaultState> = &a.from_vault;
        let _: &Account<VaultState> = &a.to_vault;
        let _: &Signer = &a.user;
        let _: &UncheckedAccount = &a.recipient;
    }

    // The earlier instructions are untouched and the Course 2 core still owns
    // the arithmetic.
    const DEPOSIT: for<'info> fn(Context<'info, Deposit<'info>>, u64) -> Result<()> =
        vault_program::deposit;
    const WITHDRAW: for<'info> fn(Context<'info, Withdraw<'info>>, u64) -> Result<()> =
        vault_program::withdraw;
    const CORE_DEPOSIT: fn(&mut VaultState, u64) -> Result<()> = VaultState::deposit;
    const CORE_WITHDRAW: fn(&mut VaultState, u64) -> Result<()> = VaultState::withdraw;
    const _: () = assert!(VaultState::INIT_SPACE == 41);
}
