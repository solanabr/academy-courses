// The native quarter-vault's withdraw guard — reference solution.
//
// Every check Anchor V2 would have generated (data validation, checked arithmetic) is
// now hand-written. Zero-amount and over-withdraw are both rejected before any lamport
// moves; the subtraction uses checked_sub so an underflow can never wrap.
pub fn vault_withdraw(balance: u64, amount: u64) -> i128 {
    if amount == 0 {
        return -2;
    }
    match balance.checked_sub(amount) {
        Some(remaining) => remaining as i128,
        None => -1,
    }
}
