// ─────────────────────────────────────────────────────────────────────────────
// TEST HARNESS — do not edit. Grading calls runCase with a fixture name and
// hands your decodeVault the pinned bytes of that fixture (the grader has no
// network). Provenance of each fixture is stated where it is defined below.
// ─────────────────────────────────────────────────────────────────────────────
function runCase(fixtureName: string) {
  const bytes = FIXTURES[fixtureName];
  if (!bytes) throw new Error("unknown fixture: " + fixtureName);
  return decodeVault(new Uint8Array(bytes));
}

/**
 * Decode a 49-byte VaultState account.
 *
 * Layout (from the lesson):
 *   offset 0,  8 bytes  — discriminator (already checked by the caller here)
 *   offset 8,  32 bytes — owner, a raw public key; display form is base58
 *   offset 40, 8 bytes  — balance, u64 LITTLE-endian
 *   offset 48, 1 byte   — bump
 */
function decodeVault(data: Uint8Array): {
  owner: string;
  balance: bigint;
  bump: number;
} {
  // Subgoal 1 (done for you) — a DataView over exactly these bytes.
  // data.byteOffset/byteLength matter: a Uint8Array can be a window into a
  // larger buffer, and the view must cover the same window.
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);

  // Subgoal 2 — owner: slice the 32 bytes at offsets 8..39 and base58-encode
  // them with the provided toBase58 helper.
  const owner = toBase58(data.slice(8, 40));

  // Subgoal 3 — balance: the unsigned 64-bit little-endian integer at
  // offset 40. Use the view; keep it a bigint.
  const balance = view.getBigUint64(40, true);

  // Subgoal 4 — bump: the single byte at offset 48.
  const bump = view.getUint8(48);

  return { owner, balance, bump };
}

// ── PROVIDED — a real base58 encoder (the display encoding of public keys) ──
const BASE58_ALPHABET =
  "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
function toBase58(bytes: Uint8Array): string {
  let n = 0n;
  for (const b of bytes) n = n * 256n + BigInt(b);
  let out = "";
  while (n > 0n) {
    out = BASE58_ALPHABET[Number(n % 58n)] + out;
    n /= 58n;
  }
  // Leading zero bytes encode as leading '1's.
  for (const b of bytes) {
    if (b !== 0) break;
    out = "1" + out;
  }
  return out;
}

// ── PINNED FIXTURES ─────────────────────────────────────────────────────────
// "reference-vault" is FY86s1fAwUiFQTjVFYprsiV6fwNH7e955MSUBo73FP4j — real
// bytes captured over devnet RPC on 2026-07-28 (the balance field is as
// recorded at capture time; the live value keeps moving, which is exactly why
// the bytes are pinned here — re-fetch them yourself with the lesson snippet).
// "second-vault" is constructed byte-for-byte to the frozen VaultState layout
// for a real derived PDA of a different owner — different owner, different
// balance, different bump; same 49-byte layout, which is all your decoder may
// rely on.
const FIXTURES: Record<string, number[]> = {
  "reference-vault": [
    228, 196, 82, 165, 98, 210, 235, 152, 78, 181, 115, 75, 97, 15, 248, 39,
    115, 50, 94, 194, 229, 144, 50, 163, 140, 21, 194, 79, 145, 31, 240, 203,
    132, 30, 178, 177, 89, 213, 11, 199, 0, 225, 245, 5, 0, 0, 0, 0, 255,
  ],
  "second-vault": [
    228, 196, 82, 165, 98, 210, 235, 152, 250, 81, 168, 175, 243, 7, 13, 139,
    135, 233, 111, 129, 165, 52, 141, 25, 59, 70, 254, 154, 56, 95, 224, 164,
    40, 67, 110, 98, 15, 29, 43, 19, 160, 37, 38, 0, 0, 0, 0, 0, 252,
  ],
};
