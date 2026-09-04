// Patch the withdraw guard: the class the V2 compiler does NOT catch for you.
//
// This is the escrow/swap payout logic, distilled to a pure function so it grades
// deterministically. Two vulnerability classes from this lesson live here:
//   1. Access control: only the vault authority may withdraw.
//   2. Arithmetic: an over-withdraw must be rejected, never wrapped or panicked.
//
// `Address` is the real shape of an on-chain address: 32 bytes, which is exactly
// what `pinocchio::address::Address` is. Two addresses are the same only if all
// 32 bytes match, and an address has no meaningful ordering — the only comparison
// an access-control check is allowed to make is equality.
//
// Return convention (so the grader can value-compare):
//   >= 0  -> the new balance after a successful withdraw
//     -1  -> rejected: caller is not the authority
//     -2  -> rejected: amount would underflow the balance
//
// The starter ships the vulnerable version: no authority check, raw subtraction.
// Make it enforce both guards.
type Address = [u8; 32];

fn settle_withdraw(balance: u64, amount: u64, caller: Address, authority: Address) -> i64 {
    // TODO: reject callers who are not the authority (return -1).
    // TODO: use checked arithmetic so an over-withdraw returns -2 instead of underflowing.
    (balance - amount) as i64
}
