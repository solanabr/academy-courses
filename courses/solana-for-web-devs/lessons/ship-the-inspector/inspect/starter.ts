// ─────────────────────────────────────────────────────────────────────────────
// TEST HARNESS — do not edit. Grading calls runCase with a fixture name and
// hands your inspect the recorded account plus minRentFor — the injected
// rent-exemption lookup (getMinimumBalanceForRentExemption, recorded from
// devnet 2026-07-28: 0 bytes → 890,880 lamports; 49 bytes → 1,231,920).
// ─────────────────────────────────────────────────────────────────────────────
function runCase(fixtureName: string) {
  const acc = ACCOUNTS[fixtureName];
  if (acc === undefined) throw new Error("unknown fixture: " + fixtureName);
  return inspect(acc, minRentFor);
}

type AccountInfo = {
  value: {
    lamports: bigint;
    ownerProgram: string;
    executable: boolean;
    data: number[];
  } | null;
};

type Report = {
  type: "missing" | "system-owned" | "program" | "vault" | "other";
  lamports: bigint | null;
  sol: string | null;
  ownerProgram: string | null;
  rentExempt: boolean | null;
  vault: { owner: string; balance: bigint; bump: number } | null;
};

/**
 * RUNG 2 — the whole inspector, written by you from the spec. No pattern
 * this time; the spec from the lesson is the contract:
 *
 *   - type: your rung-1 classification, same five values, same order of
 *     checks (existence → executable → system-owned → vault-with-shape →
 *     other).
 *   - For a MISSING account every other field is null.
 *   - lamports: the exact balance (bigint). sol: lamportsToSol(lamports).
 *   - ownerProgram: as recorded.
 *   - rentExempt: lamports >= minRentFor(data.length) — the minimum for
 *     THIS account's data length, not a constant.
 *   - vault: the decoded { owner, balance, bump } — your lesson-2 decoder —
 *     but ONLY when type is "vault". Everything else gets null, including
 *     accounts owned by the vault program whose bytes are not a VaultState.
 *
 * Helpers provided below: hasVaultShape, toBase58, lamportsToSol, and the
 * constants. The byte offsets are the ones you have known since lesson 2:
 * owner at 8..39, balance u64 LE at 40, bump at 48.
 */
function inspect(
  acc: AccountInfo,
  minRentFor: (dataLength: number) => bigint
): Report {
  // Write the inspector from the spec, then delete the throw.
  throw new Error("write inspect from the spec above");
}

// ── PROVIDED ────────────────────────────────────────────────────────────────
const VAULT_PROGRAM = "D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd";
const SYSTEM_PROGRAM = "11111111111111111111111111111111";
const VAULT_DISCRIMINATOR = [228, 196, 82, 165, 98, 210, 235, 152];

/** True iff the bytes have the exact VaultState shape: 49 bytes, tag first. */
function hasVaultShape(data: number[]): boolean {
  if (data.length !== 49) return false;
  for (let i = 0; i < 8; i++) {
    if (data[i] !== VAULT_DISCRIMINATOR[i]) return false;
  }
  return true;
}

/** Lamports → "whole.fraction" SOL string, all nine fractional digits. */
function lamportsToSol(lamports: bigint): string {
  const whole = lamports / 1_000_000_000n;
  const frac = (lamports % 1_000_000_000n).toString().padStart(9, "0");
  return whole.toString() + "." + frac;
}

/**
 * getMinimumBalanceForRentExemption, injected. The recorded devnet anchors
 * (0 → 890,880; 49 → 1,231,920) pin the live rent parameters this linear
 * formula reproduces: (128 + dataLength) × 6,960 lamports.
 */
function minRentFor(dataLength: number): bigint {
  return (128n + BigInt(dataLength)) * 6_960n;
}

/** A real base58 encoder — same helper you used in lesson 2. */
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
  for (const b of bytes) {
    if (b !== 0) break;
    out = "1" + out;
  }
  return out;
}

// ── RECORDED FIXTURES — the same set rung 1 classified: real devnet
// accounts captured 2026-07-28, except "second-vault" (constructed
// byte-for-byte to the frozen VaultState layout for a real derived PDA) and
// "truncated-vault" (a synthetic layout probe, deliberately NOT rent-exempt:
// 900,000 lamports < minRentFor(12) = 974,400) ──────────────────────────────
const ACCOUNTS: Record<string, AccountInfo> = {
  "reference-vault": {
    value: {
      lamports: 101231920n,
      ownerProgram: "D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd",
      executable: false,
      data: [
        228, 196, 82, 165, 98, 210, 235, 152, 78, 181, 115, 75, 97, 15, 248,
        39, 115, 50, 94, 194, 229, 144, 50, 163, 140, 21, 194, 79, 145, 31,
        240, 203, 132, 30, 178, 177, 89, 213, 11, 199, 0, 225, 245, 5, 0, 0,
        0, 0, 255,
      ],
    },
  },
  "second-vault": {
    value: {
      lamports: 3731920n,
      ownerProgram: "D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd",
      executable: false,
      data: [
        228, 196, 82, 165, 98, 210, 235, 152, 250, 81, 168, 175, 243, 7, 13,
        139, 135, 233, 111, 129, 165, 52, 141, 25, 59, 70, 254, 154, 56, 95,
        224, 164, 40, 67, 110, 98, 15, 29, 43, 19, 160, 37, 38, 0, 0, 0, 0,
        0, 252,
      ],
    },
  },
  wallet: {
    value: {
      lamports: 3331870864n,
      ownerProgram: "11111111111111111111111111111111",
      executable: false,
      data: [],
    },
  },
  program: {
    value: {
      lamports: 1141440n,
      ownerProgram: "BPFLoaderUpgradeab1e11111111111111111111111",
      executable: true,
      data: [
        2, 0, 0, 0, 68, 241, 198, 255, 117, 99, 71, 101, 249, 222, 102, 97,
        219, 215, 40, 216, 232, 31, 153, 137, 220, 115, 36, 234, 236, 38, 120,
        161, 231, 188, 130, 216,
      ],
    },
  },
  "clock-sysvar": {
    value: {
      lamports: 1169280n,
      ownerProgram: "Sysvar1111111111111111111111111111111111111",
      executable: false,
      data: [
        23, 157, 149, 28, 0, 0, 0, 0, 112, 187, 104, 106, 0, 0, 0, 0, 86, 4,
        0, 0, 0, 0, 0, 0, 87, 4, 0, 0, 0, 0, 0, 0, 244, 254, 104, 106, 0, 0,
        0, 0,
      ],
    },
  },
  "truncated-vault": {
    value: {
      lamports: 900000n,
      ownerProgram: "D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd",
      executable: false,
      data: [228, 196, 82, 165, 98, 210, 235, 152, 78, 181, 115, 75],
    },
  },
  missing: { value: null },
};
