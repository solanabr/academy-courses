// token<->ticket swap: quote the output amount from the pool's own reserves.
//
// Uniswap-v2 constant-product invariant with a 0.3% fee (997/1000). A u128
// intermediate keeps the multiplication from overflowing on u64 reserves, and
// every step is checked so an overflow degrades to a 0 quote rather than panicking
// inside an on-chain instruction.
//
//   amount_in_with_fee = amount_in * 997
//   out = (amount_in_with_fee * reserve_out) / (reserve_in * 1000 + amount_in_with_fee)
pub fn swap_out(reserve_in: u64, reserve_out: u64, amount_in: u64) -> u64 {
    if amount_in == 0 || reserve_in == 0 || reserve_out == 0 {
        return 0;
    }
    let fee_adjusted = match (amount_in as u128).checked_mul(997) {
        Some(v) => v,
        None => return 0,
    };
    let numerator = match fee_adjusted.checked_mul(reserve_out as u128) {
        Some(v) => v,
        None => return 0,
    };
    let denominator = match (reserve_in as u128)
        .checked_mul(1000)
        .and_then(|v| v.checked_add(fee_adjusted))
    {
        Some(v) => v,
        None => return 0,
    };
    (numerator / denominator) as u64
}
