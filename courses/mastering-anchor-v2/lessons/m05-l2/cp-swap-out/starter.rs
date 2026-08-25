// token<->ticket swap: quote the output amount from the pool's own reserves.
//
// The arcade pool holds `reserve_in` of the token you pay with and `reserve_out`
// of the token you want. Given `amount_in`, return how many output tokens the
// trader receives, applying a 0.3% fee (multiply the input by 997/1000) under the
// constant-product invariant (x * y = k).
//
// TODO: replace the naive body below. This version just scales by the *current*
// price ratio: it ignores the 0.3% fee AND the fact that adding `amount_in` shifts
// the reserves, so it over-quotes and lets a trader drain the pool.
pub fn swap_out(reserve_in: u64, reserve_out: u64, amount_in: u64) -> u64 {
    if reserve_in == 0 {
        return 0;
    }
    amount_in * reserve_out / reserve_in
}
