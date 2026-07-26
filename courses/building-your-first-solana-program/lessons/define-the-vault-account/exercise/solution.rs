use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ─────────────────────────────────────────────────────────────────────────────
// Your Course 2 vault core, imported and not retyped.
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
        msg!("Vault initialized for {}", vault.owner);
        Ok(())
    }

    pub fn vault_info(_ctx: Context<VaultInfo>) -> Result<()> {
        msg!("Vault program online");
        Ok(())
    }
}

// One correct assembly. The three fields may be declared in any order, and the
// constraints inside `#[account(...)]` in any order too. What is fixed is which
// lines are here at all.
//
// Pool lines (5) `rent = rent` and (12) `mut,` are not used:
//   · `rent = rent` was removed in Anchor 0.31. Anchor reads the rent parameters
//     itself and derives the deposit from `space`.
//   · `mut` on the vault would be asking to write an account that already
//     exists. `init` is the request to create it — allocate `space` bytes, set
//     the owner to this program, and fund the rent-exempt deposit from `payer`.
//     `init` implies the write.
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
    // `mut` because `payer = user` debits this account's lamports for the rent
    // deposit. `Signer` because someone has to authorize spending them. Two
    // separate facts, two separate pieces of syntax.
    #[account(mut)]
    pub user: Signer<'info>,
    // `init` creates the account by calling the System Program, so the System
    // Program has to be in the account list. Anchor does not add it for you.
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VaultInfo {}

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
// A type check, not a behaviour check. See the starter for its exact reach.
// ─────────────────────────────────────────────────────────────────────────────
#[doc(hidden)]
#[allow(dead_code)]
mod verify {
    use super::*;

    const _: () = assert!(VaultState::DISCRIMINATOR.len() == 8);
    const _: () = assert!(VaultState::INIT_SPACE == 41);

    fn pin_bump(bumps: &InitializeVaultBumps) -> u8 {
        bumps.vault
    }

    fn pin_vault<'info>(a: &'info InitializeVault<'info>) -> &'info Account<'info, VaultState> {
        &a.vault
    }
    fn pin_user<'info>(a: &'info InitializeVault<'info>) -> &'info Signer<'info> {
        &a.user
    }
    fn pin_system<'info>(a: &'info InitializeVault<'info>) -> &'info Program<'info, System> {
        &a.system_program
    }
}
