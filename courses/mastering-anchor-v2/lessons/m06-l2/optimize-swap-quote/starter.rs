/// Quote the output of a constant-product swap.
///
/// This is the hot function on the swap instruction you have been profiling.
/// The optimized version must do THREE things the version below does not:
///   1. apply the trading fee (`fee_bps`, in basis points) before quoting,
///   2. compute the intermediate products in `u128` so large reserves cannot
///      silently overflow a `u64` multiply, and
///   3. refuse the inputs the curve is not defined on — an empty reserve, or a
///      `fee_bps` above the 10_000 bps scale — by returning 0, because the
///      frozen `u64` signature has nowhere to put an error.
///
/// Right now it ignores the fee entirely, multiplies in `u64`, and checks
/// nothing: its quotes are wrong whenever `fee_bps > 0`, it panics on a pool
/// deep enough to overflow the multiply, and on an empty `reserve_in` it quotes
/// the entire `reserve_out` — the whole pool — to whoever asks first. Fix it.
///
/// Reference formula (Uniswap-style constant product with fee):
///   amount_in_with_fee = amount_in * (10_000 - fee_bps)
///   out = (reserve_out * amount_in_with_fee)
///         / (reserve_in * 10_000 + amount_in_with_fee)
fn get_amount_out(reserve_in: u64, reserve_out: u64, amount_in: u64, fee_bps: u64) -> u64 {
    let _ = fee_bps; // TODO: the fee is being ignored
    let numerator = reserve_out * amount_in;
    let denominator = reserve_in + amount_in;
    numerator / denominator
}
