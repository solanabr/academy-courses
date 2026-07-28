/**
 * Convert a human-facing token amount into base units.
 *
 *   baseUnits = uiAmount * 10^decimals
 *
 * Decimals are NOT conventionally 18 on Solana: USDC uses 6, wrapped SOL uses 9,
 * and plenty of NFTs use 0. Always read `decimals` from the mint.
 *
 * Return a whole number of base units.
 *
 * Examples:
 *   toBaseUnits(1.5, 6) -> 1500000
 *   toBaseUnits(1, 0)   -> 1
 */
function toBaseUnits(uiAmount: number, decimals: number): number {
  return uiAmount * 10 ** 18;
}
