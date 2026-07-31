// Add an instruction — assembled.
//
// Pool A, in order: (a5) opens the handler, (a3) is the body, (a1) returns, (a2)
// closes the block. (a4) is the same signature with `ProgramResult` as its return
// type; that type was removed from Anchor years ago, so it does not resolve.
//
// Pool B, in order: (b2) then (b1). (b3) is `#[account]`, which marks the
// *contents of an account*, not an account list — pick it and `Context<VaultInfo>`
// fails with `the trait bound VaultInfo: Bumps is not satisfied`, because the
// `Accounts` derive is the only thing that supplies that impl. (b4) is the one
// wrong line the compiler does not catch; see the lesson prose.

use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod vault_program {
    use super::*;

    pub fn version(_ctx: Context<Version>) -> Result<()> {
        msg!("vault program, id {}", ID);
        Ok(())
    }

    pub fn vault_info(_ctx: Context<VaultInfo>) -> Result<()> {
        msg!("vault program id: {}", ID);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Version {}

#[derive(Accounts)]
pub struct VaultInfo {}

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

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
//
// It compiles only if `vault_info` and `VaultInfo` exist with exactly the names
// and signatures the lesson asks for. It is a TYPE check and nothing more: it
// cannot see the body of your handler, and it cannot see a second `#[error_code]`
// enum sitting above it.
// ─────────────────────────────────────────────────────────────────────────────
#[doc(hidden)]
#[allow(dead_code)]
mod verify {
    use super::*;

    // Pool B: `VaultInfo` exists, has no fields, and carries
    // `#[derive(Accounts)]` — the derive is what supplies the `Bumps` impl that
    // `Context` demands on the next line.
    fn accounts() -> VaultInfo {
        VaultInfo {}
    }

    // Pool A: a handler named `vault_info` exists inside the `#[program]`
    // module, takes `Context<VaultInfo>` and returns `Result<()>`.
    const VAULT_INFO: for<'info> fn(Context<'info, VaultInfo>) -> Result<()> =
        vault_program::vault_info;
}
