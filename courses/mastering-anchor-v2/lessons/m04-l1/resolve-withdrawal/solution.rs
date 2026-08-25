// Guard the vault's payout: decide how many lamports may safely leave.
//
// The three failure cases are checked BEFORE computing the safe remainder, and
// the subtraction is only reached once `requested <= balance` is proven, so it
// can never underflow.
pub fn resolve_withdrawal(balance: u64, rent_exempt_min: u64, requested: u64) -> i64 {
    if requested == 0 {
        return -1; // nothing to withdraw
    }
    if requested > balance {
        return -2; // would underflow the vault
    }
    let remaining = balance - requested; // safe: requested <= balance
    if remaining < rent_exempt_min {
        return -3; // would drop the PDA below rent-exempt and risk closing it
    }
    requested as i64
}
