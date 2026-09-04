// The native quarter-vault's withdraw guard.
//
// In the framework version, Anchor generated the account checks for you. Stripping
// the framework, YOU write the validation by hand. This is the load-bearing branch of
// the native withdraw instruction: given the vault's current lamport `balance` and a
// requested `amount`, decide the outcome.
//
// Contract (return an i128 so we can signal failure without a Result in the harness):
//   - a zero-amount withdraw is invalid          -> return -2
//   - an over-withdraw (amount > balance)        -> return -1   (use checked_sub, no naive `-`)
//   - otherwise                                  -> return the remaining balance
//
// The starter below skips BOTH guards and subtracts naively. It happens to return the
// right number for a normal withdraw, but it is wrong (and unsafe) for the reject cases.
fn vault_withdraw(balance: u64, amount: u64) -> i128 {
    // TODO: reject a zero-amount withdraw with -2
    // TODO: reject an over-withdraw with -1 using balance.checked_sub(amount)
    (balance as i128) - (amount as i128)
}
