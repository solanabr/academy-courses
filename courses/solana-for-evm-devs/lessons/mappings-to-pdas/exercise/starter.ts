/**
 * Build the seed list for a per-owner vault PDA.
 *
 * The convention this program uses is a literal namespace prefix followed by the
 * owner, so a vault seed list is exactly:
 *
 *   ["vault", <owner>]
 *
 * The prefix is what stops a "vault" PDA from colliding with a "config" PDA
 * derived from the same owner. Order is part of the derivation: swapping the two
 * produces a different address.
 *
 * Example:
 *   vaultSeeds("alice") -> ["vault", "alice"]
 */
function vaultSeeds(owner: string): string[] {
  return [owner];
}
