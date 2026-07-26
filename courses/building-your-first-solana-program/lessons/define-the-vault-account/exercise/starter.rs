use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ─────────────────────────────────────────────────────────────────────────────
// Your Course 2 vault core, imported and not retyped. Nothing in here changes.
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

// ─────────────────────────────────────────────────────────────────────────────
// THE FRAGMENT POOL
//
// Every line `InitializeVault` needs is here, out of order. Two of them belong
// nowhere at all. Assemble the struct below from the ones you want; leave the
// rest where they are.
//
//   (1)   pub struct InitializeVault<'info> {
//   (2)       pub system_program: Program<'info, System>,
//   (3)           space = 8 + 32 + 8 + 1,
//   (4)       pub vault: Account<'info, VaultState>,
//   (5)           rent = rent,
//   (6)       #[account(mut)]
//   (7)           seeds = [b"vault", user.key().as_ref()],
//   (8)           init,
//   (9)       pub user: Signer<'info>,
//  (10)           bump
//  (11)           payer = user,
//  (12)           mut,
//
// The two indent levels are a hint: the ones indented further are constraints
// and go inside `#[account( ... )]`. The others are fields of the struct.
//
// On the two lines that belong nowhere:
//   · One is a constraint Anchor deleted. The build refuses it with a bare
//     `error: Invalid attribute` pointing at the line — it will not tell you
//     why, which is the point of knowing before you press the button.
//   · The other is the constraint people reach for out of habit when an account
//     is about to be written to. It is the wrong one here, because this account
//     does not exist yet, and "write to it" and "bring it into existence" are
//     different requests.
//
// One thing is not in the pool and is not optional: line (1) replaces the
// `InitializeVault` below, and the difference between them is the part that
// matters.
// ─────────────────────────────────────────────────────────────────────────────

// TODO: assemble InitializeVault from the pool above, replacing this.
#[derive(Accounts)]
pub struct InitializeVault {}

#[derive(Accounts)]
pub struct VaultInfo {}

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
//
// A TYPE check, not a behaviour check. Grading is compile-success: the build
// server compiles the file and reports whether it built. Nothing reads your
// source and there are no hidden tests on the Rust path — so the harness names,
// in Rust, every item a passing file must have, and an absent or wrongly-typed
// one becomes a compile error the grader already catches.
//
// It reaches: the three field names and their exact types, the lifetime on the
// struct, and that `vault` carries a `bump` constraint (which Anchor accepts
// only alongside `seeds`).
//
// It cannot reach: whether `space` is the right number. `8 + 32 + 8 + 1` and
// `8 + 999` compile identically. That one is on you — which is why you derived
// it two lessons ago instead of copying it.
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
