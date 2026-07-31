use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ─────────────────────────────────────────────────────────────────────────────
// Your Course 2 vault core, imported and not retyped. Nothing in here is broken.
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

        // TASK 2 — the handler body. Three lines, no scaffolding this time.
        //
        // By the time this runs the account exists: allocated, program-owned,
        // discriminator written, every field zero. What it does not have is
        // meaning. Give it the three things `deposit` and `withdraw` will need —
        // who it belongs to, what it holds, and the bump that proves its address.
        //
        // Names and types are in `VaultState` above. The bump Anchor just derived
        // is on `ctx.bumps`, under this account's own field name.

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
    // BUG: Something is missing here...
}

#[derive(Accounts)]
pub struct VaultInfo {}

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
//
// A TYPE check, not a behaviour check. It names the three accounts the struct
// must hold, so that "make the build pass" cannot be satisfied by deleting the
// constraint that caused the failure.
//
// It cannot see the handler body at all. Whether you assigned the three fields,
// and whether you assigned them the right values, is invisible to a compile-only
// grader — there are no hidden tests on the Rust path. Module 4 proves that part
// against a real runtime.
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
