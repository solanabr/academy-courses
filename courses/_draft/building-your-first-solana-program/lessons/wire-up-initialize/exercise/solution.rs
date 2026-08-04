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

        // TASK 2. Three assignments, and every one of them is load-bearing later:
        //   · `owner` is what module 3's `has_one = owner` check reads to reject
        //     somebody else's withdrawal.
        //   · `balance` starts at zero explicitly. The bytes are already zero, so
        //     this line changes nothing — and it is still worth writing, because
        //     the next person to read the handler should not have to know that.
        //   · `bump` is stored once here so that `deposit` and `withdraw` can use
        //     `bump = vault.bump` instead of re-running the derivation search.
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
    // TASK 1. `init` creates the account by calling the System Program, so the
    // System Program has to be in the account list. Anchor never injects it.
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
