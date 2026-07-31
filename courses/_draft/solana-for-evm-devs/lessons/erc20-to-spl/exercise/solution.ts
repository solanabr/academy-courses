function toBaseUnits(uiAmount: number, decimals: number): number {
  return Math.round(uiAmount * 10 ** decimals);
}
