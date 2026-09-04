// Guard the vault's payout: decide how many lamports may safely leave.
//
// A program-owned vault PDA can sign its own withdrawal. Before the CPI fires,
// though, the program must decide whether the withdrawal is even legal.
// Implement `resolve_withdrawal` so it:
//   * returns -1 if `requested` is 0               (nothing to withdraw)
//   * returns -2 if `requested` exceeds `balance`  (would underflow the vault)
//   * returns -3 if the remainder would fall below `rent_exempt_min`
//                                                  (would risk closing the PDA)
//   * otherwise returns `requested` as i64         (safe to sign for)
//
// The starter below ignores every guard and just hands back what was asked,
// so it fails the zero, over-withdraw, and rent-floor cases.
fn resolve_withdrawal(balance: u64, rent_exempt_min: u64, requested: u64) -> i64 {
    requested as i64
}
