function accountSize(numU64Fields: number, hasBump: boolean): number {
  const DISCRIMINATOR = 8;
  const U64 = 8;
  return DISCRIMINATOR + numU64Fields * U64 + (hasBump ? 1 : 0);
}
