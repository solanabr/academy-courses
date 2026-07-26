// Anatomy of an Anchor program — the four parts, with the second handler and
// its account list filled in.
//
// The `msg!` string is yours; nothing checks it. What is fixed is the pair of
// names: the handler is `greet` and the accounts struct is `Greet`, because
// `Context<Greet>` in the signature is what ties them together.

use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// PART 1 of 4 — `declare_id!`: the program's address.
// ─────────────────────────────────────────────────────────────────────────────
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ─────────────────────────────────────────────────────────────────────────────
// PART 2 of 4 — `#[program]`: the instruction handlers.
// ─────────────────────────────────────────────────────────────────────────────
#[program]
pub mod vault_program {
    use super::*;

    pub fn version(_ctx: Context<Version>) -> Result<()> {
        msg!("vault program, id {}", ID);
        Ok(())
    }

    pub fn greet(_ctx: Context<Greet>) -> Result<()> {
        msg!("Hello, Solana!");
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PART 3 of 4 — `#[derive(Accounts)]`: one account list per handler.
//
// Both are empty because neither handler touches an account. Empty is a claim,
// not an omission: it says "this instruction may read and write nothing".
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Accounts)]
pub struct Version {}

#[derive(Accounts)]
pub struct Greet {}

// ─────────────────────────────────────────────────────────────────────────────
// PART 4 of 4 — `#[account]`: the state, unchanged from Course 2.
// ─────────────────────────────────────────────────────────────────────────────
#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub owner: Pubkey,
    pub balance: u64,
    pub bump: u8,
}

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
//
// It compiles only if both items exist with the exact names and signatures the
// lesson asks for. This is a TYPE check, not a behaviour check: it cannot see
// what your `msg!` string says, and it cannot see whether the handler does
// anything at all.
// ─────────────────────────────────────────────────────────────────────────────
#[doc(hidden)]
#[allow(dead_code)]
mod verify {
    use super::*;

    // Part 3: `Greet` exists, has no fields, and carries `#[derive(Accounts)]`
    // — the derive is what supplies the `Bumps` impl `Context` requires below.
    fn accounts() -> Greet {
        Greet {}
    }

    // Part 2: a handler named `greet` exists inside the `#[program]` module,
    // takes `Context<Greet>` and returns `Result<()>`.
    const GREET: for<'info> fn(Context<'info, Greet>) -> Result<()> = vault_program::greet;

    // Part 4 is unchanged from Course 2: three fields, 41 bytes of data.
    const _: () = assert!(VaultState::INIT_SPACE == 41);
}
