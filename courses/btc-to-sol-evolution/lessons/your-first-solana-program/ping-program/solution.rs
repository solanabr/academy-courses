// Your first Solana program — assembled. Four parts: the crate prelude, the program
// id, the #[program] module with one instruction, and the accounts struct it names.
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"); // replace with `anchor keys list`

#[program]
pub mod first_program {
    use super::*;

    pub fn ping(_ctx: Context<Ping>) -> Result<()> {
        msg!("pong from {}", ID);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Ping {}

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
// It is a TYPE check: it compiles only if `ping` and `Ping` exist with exactly the
// names and signatures the lesson asks for. It never runs your instruction.
// ─────────────────────────────────────────────────────────────────────────────
#[doc(hidden)]
#[allow(dead_code)]
mod verify {
    use super::*;

    // `Ping` exists, is empty, and carries #[derive(Accounts)] (the derive supplies
    // the `Bumps` impl that `Context` demands).
    fn accounts() -> Ping {
        Ping {}
    }

    // a handler named `ping` exists inside #[program], takes `Context<Ping>` and
    // returns `Result<()>`.
    const PING: for<'info> fn(Context<'info, Ping>) -> Result<()> = first_program::ping;
}
