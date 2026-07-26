use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// Nothing in this file needs changing. Press Build, then read the log.
pub fn vault_core_banner() -> Result<()> {
    msg!("vault core reporting in from {}", ID);
    Ok(())
}
