// Patch the withdraw guard: the class the V2 compiler does NOT catch for you.
//
// `Address` is 32 bytes, the shape `pinocchio::address::Address` really has. The
// authority gate is a full-width equality check: comparing a prefix, a single
// byte, or (worse) an ordering would let a near-miss address through, and a
// near-miss address is an attacker's address.
//
// Return convention:
//   >= 0  -> the new balance after a successful withdraw
//     -1  -> rejected: caller is not the authority
//     -2  -> rejected: amount would underflow the balance
type Address = [u8; 32];

fn settle_withdraw(balance: u64, amount: u64, caller: Address, authority: Address) -> i64 {
    if caller != authority {
        return -1;
    }
    match balance.checked_sub(amount) {
        Some(remaining) => remaining as i64,
        None => -2,
    }
}
