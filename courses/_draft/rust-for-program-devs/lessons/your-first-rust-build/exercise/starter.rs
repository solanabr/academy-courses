use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// Nothing in this file needs changing. Press Build. This one passes, so you
/// get one success line and no log — the prose below explains why, and what you
/// get instead when a build fails.
pub fn vault_core_banner() -> Result<()> {
    msg!("vault core reporting in from {}", ID);
    Ok(())
}
