use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ─────────────────────────────────────────────────────────────────────────────
// Your Course 2 vault core, imported and not retyped. Nothing in this module
// needs to change in this lesson — `bump: u8` is already here, waiting for a
// reason.
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

        // ── SUBGOAL 3 ────────────────────────────────────────────────────────
        // Store the canonical bump, so that no later instruction has to run the
        // derivation search again. Anchor hands you the bump it just computed on
        // `ctx.bumps`, under the field's own name.
        //
        // One line, right here. It only compiles once subgoal 2 is done.

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
        space = 8 + 32 + 8 + 1
        // ── SUBGOAL 1 ────────────────────────────────────────────────────────
        // Derive this account's address instead of accepting whatever the client
        // passed. Add a `seeds` constraint: a `b"vault"` namespace prefix, then
        // the user's key as raw bytes.
        //
        // Remember the trailing comma on the line above once you add a line
        // below it.
        //
        // ── SUBGOAL 2 ────────────────────────────────────────────────────────
        // Ask Anchor for the canonical bump and check that the account handed in
        // really is at that address. One bare word. `seeds` without it does not
        // compile, and neither does it without `seeds`.
    )]
    pub vault: Account<'info, VaultState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VaultInfo {}

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
//
// This is a TYPE check, not a behaviour check. Grading on this platform is
// compile-success: the build server compiles your file and reports whether it
// built. Nothing inspects your source, and there are no hidden tests on the Rust
// path. So the harness names, in Rust, the things a passing file must have —
// and an absent or wrongly-typed item becomes a compile error the grader
// already catches.
//
// What it reaches: subgoals 1 and 2, via `InitializeVaultBumps`. `ctx.bumps`
// only grows a field for an account that carries a `bump` constraint, and
// Anchor only accepts `bump` alongside `seeds`.
//
// What it cannot reach: whether your seeds are the *right* seeds, and whether
// you actually stored the bump in subgoal 3. Those are proven in module 4,
// against a real runtime.
// ─────────────────────────────────────────────────────────────────────────────
#[doc(hidden)]
#[allow(dead_code)]
mod verify {
    use super::*;

    // `#[account]` is present: 8 bytes of discriminator.
    const _: () = assert!(VaultState::DISCRIMINATOR.len() == 8);

    // `#[derive(InitSpace)]` is present and the fields are still exactly
    // Pubkey (32) + u64 (8) + u8 (1) = 41. `space` is 8 + this.
    const _: () = assert!(VaultState::INIT_SPACE == 41);

    // The vault account is a PDA: this field exists only when `vault` carries a
    // `bump` constraint, which Anchor accepts only alongside `seeds`.
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
