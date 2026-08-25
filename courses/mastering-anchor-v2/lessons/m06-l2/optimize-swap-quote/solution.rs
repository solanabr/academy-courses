/// Quote the output of a constant-product swap with a trading fee.
///
/// The whole quote runs through a single `u128` intermediate so a large
/// `reserve_out * amount_in_with_fee` product cannot overflow, and there is
/// exactly one division on the hot path.
///
///   amount_in_with_fee = amount_in * (10_000 - fee_bps)
///   out = (reserve_out * amount_in_with_fee)
///         / (reserve_in * 10_000 + amount_in_with_fee)
pub fn get_amount_out(reserve_in: u64, reserve_out: u64, amount_in: u64, fee_bps: u64) -> u64 {
    let amount_in_with_fee = (amount_in as u128) * (10_000u128 - fee_bps as u128);
    let numerator = amount_in_with_fee * (reserve_out as u128);
    let denominator = (reserve_in as u128) * 10_000u128 + amount_in_with_fee;
    (numerator / denominator) as u64
}
