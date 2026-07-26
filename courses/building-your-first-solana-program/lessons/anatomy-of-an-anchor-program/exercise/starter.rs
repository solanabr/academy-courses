// Anatomy of an Anchor program — the same vault program, reduced to its four
// parts and nothing else. Each part is labeled below.
//
// Parts 1 and 4 are complete. Parts 2 and 3 each already contain one worked
// example, `version` and `Version`. Your job is the second pair: `greet` and
// `Greet`, following the pattern that is already there.
//
// This file does NOT compile as shipped. The harness at the bottom names both
// items you owe, and every error you see comes from one of them.

use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// PART 1 of 4 — `declare_id!`: the program's address.  (done)
//
// One address per program, baked into the binary. At deploy time the runtime
// compares this value against the address it is deploying to.
// ─────────────────────────────────────────────────────────────────────────────
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ─────────────────────────────────────────────────────────────────────────────
// PART 2 of 4 — `#[program]`: the instruction handlers.
//
// Every `pub fn` in here becomes a callable instruction. First parameter is
// always `Context<T>`; return type is always `Result<()>`.
//
// TODO (part 2): add a second handler, `greet`, that logs "Hello, Solana!"
//                and returns `Ok(())`. Copy the shape of `version`.
// ─────────────────────────────────────────────────────────────────────────────
#[program]
pub mod vault_program {
    use super::*;

    pub fn version(_ctx: Context<Version>) -> Result<()> {
        msg!("vault program, id {}", ID);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PART 3 of 4 — `#[derive(Accounts)]`: one account list per handler.
//
// The struct name must match the `Context<T>` generic exactly. `Version`
// declares no fields, because `version` reads and writes no accounts.
//
// TODO (part 3): add the `Greet` accounts struct. `greet` only logs, so it too
//                needs no fields — but it does need the derive.
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Accounts)]
pub struct Version {}

// ─────────────────────────────────────────────────────────────────────────────
// PART 4 of 4 — `#[account]`: the state.  (done — this is your Course 2 struct)
//
// `#[account]` marks a struct as the contents of a program-owned account: it
// generates the serialization and the 8-byte discriminator that identifies the
// type on-chain. Nothing here creates an account. Module 2 does that.
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
