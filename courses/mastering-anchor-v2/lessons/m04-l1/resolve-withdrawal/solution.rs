// Guard the vault's payout: decide how many lamports may safely leave.
//
// The three failure cases are checked BEFORE computing the safe remainder, and
// the subtraction still goes through `checked_sub` even though the guard above
// already proved it cannot underflow: the proof is one refactor away from being
// wrong, and a bare `-` wraps silently in a release build.
pub fn resolve_withdrawal(balance: u64, rent_exempt_min: u64, requested: u64) -> i64 {
    if requested == 0 {
        return -1; // nothing to withdraw
    }
    if requested > balance {
        return -2; // would underflow the vault
    }
    let Some(remaining) = balance.checked_sub(requested) else {
        return -2; // unreachable while the guard above holds; still not a bare `-`
    };
    if remaining < rent_exempt_min {
        return -3; // would drop the PDA below rent-exempt and risk closing it
    }
    requested as i64
}
