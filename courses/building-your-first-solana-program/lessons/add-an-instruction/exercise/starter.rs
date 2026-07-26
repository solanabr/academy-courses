// Add an instruction — assemble it from the fragments below.
//
// Nothing here is for you to invent. Every line you need is already in this
// file, commented out and out of order, in two pools. Uncomment the ones that
// belong, put them in the right place in the right order, and leave the rest
// commented.
//
// THREE of the nine pool lines belong nowhere. All three are mistakes people
// actually make. TWO of them the compiler catches; ONE builds green and is
// wrong anyway. Decide which is which before you build.
//
// Target, in full:
//
//     pub fn vault_info(_ctx: Context<VaultInfo>) -> Result<()>
//     #[derive(Accounts)] pub struct VaultInfo {}
//
// This file does NOT compile as shipped.

use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod vault_program {
    use super::*;

    pub fn version(_ctx: Context<Version>) -> Result<()> {
        msg!("vault program, id {}", ID);
        Ok(())
    }

    // ── POOL A — five lines. FOUR of them make one complete handler, here,
    //    in some order. ONE belongs nowhere. Indentation is not graded; order
    //    inside the block is the whole exercise.
    //
    // (a1)     Ok(())
    // (a2) }
    // (a3)     msg!("vault program id: {}", ID);
    // (a4) pub fn vault_info(_ctx: Context<VaultInfo>) -> ProgramResult {
    // (a5) pub fn vault_info(_ctx: Context<VaultInfo>) -> Result<()> {
}

#[derive(Accounts)]
pub struct Version {}

// ── POOL B — four lines. TWO of them belong here, in order. TWO belong nowhere.
//
// (b1) pub struct VaultInfo {}
// (b2) #[derive(Accounts)]
// (b3) #[account]
// (b4) #[error_code] pub enum InfoError { #[msg("no vault at that address")] NoVault }

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
