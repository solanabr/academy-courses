// Patch the withdraw guard: the class the V2 compiler does NOT catch for you.
//
// Return convention:
//   >= 0  -> the new balance after a successful withdraw
//     -1  -> rejected: caller is not the authority
//     -2  -> rejected: amount would underflow the balance
pub fn settle_withdraw(balance: u64, amount: u64, caller: u8, authority: u8) -> i64 {
    if caller != authority {
        return -1;
    }
    match balance.checked_sub(amount) {
        Some(remaining) => remaining as i64,
        None => -2,
    }
}
