/**
 * Anchor lays an account out as:
 *
 *   8 bytes   discriminator (always present on an Anchor account)
 *   8 bytes   per u64 field
 *   1 byte    for the PDA bump, when the account stores one
 *
 * Return the total number of bytes to request at creation.
 *
 * Examples:
 *   accountSize(3, true)  -> 8 + 24 + 1 = 33
 *   accountSize(0, false) -> 8
 */
function accountSize(numU64Fields: number, hasBump: boolean): number {
  return 0;
}
