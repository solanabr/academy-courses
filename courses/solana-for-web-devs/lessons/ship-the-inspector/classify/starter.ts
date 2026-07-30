// ─────────────────────────────────────────────────────────────────────────────
// TEST HARNESS — do not edit. Grading calls runCase with a fixture name and
// hands classifyAccount the recorded account, in the exact shape an RPC
// getAccountInfo read returns: { value: null } for a missing account,
// { value: { lamports, ownerProgram, executable, data } } otherwise.
// ─────────────────────────────────────────────────────────────────────────────
function runCase(fixtureName: string) {
  const acc = ACCOUNTS[fixtureName];
  if (acc === undefined) throw new Error("unknown fixture: " + fixtureName);
  return classifyAccount(acc);
}

type AccountInfo = {
  value: {
    lamports: bigint;
    ownerProgram: string;
    executable: boolean;
    data: number[];
  } | null;
};

/**
 * RUNG 1 — the branch table. Return exactly one of:
 * "missing" | "system-owned" | "program" | "vault" | "other".
 *
 * The seven candidate branches are in the PARTS BIN, shuffled. FIVE belong,
 * in the right order. Two are decoys:
 *   - one trusts ownership alone and skips the shape check — the exact bug
 *     that makes an inspector report garbage as a vault;
 *   - one calls an existing empty account "missing" — an account with no
 *     data still exists.
 * Uncomment the branches you want, in order. Do not write branches that are
 * not in the bin.
 *
 * ┌─ PARTS BIN ──────────────────────────────────────────────────────────┐
 * │ (A)  if (acc.value.ownerProgram === VAULT_PROGRAM) return "vault";   │
 * │ (B)  if (acc.value === null) return "missing";                       │
 * │ (C)  return "other";                                                 │
 * │ (D)  if (acc.value.data.length === 0) return "missing";              │
 * │ (E)  if (acc.value.executable) return "program";                     │
 * │ (F)  if (acc.value.ownerProgram === SYSTEM_PROGRAM)                  │
 * │        return "system-owned";                                        │
 * │ (G)  if (acc.value.ownerProgram === VAULT_PROGRAM &&                 │
 * │          hasVaultShape(acc.value.data)) return "vault";              │
 * └──────────────────────────────────────────────────────────────────────┘
 */
function classifyAccount(
  acc: AccountInfo
): "missing" | "system-owned" | "program" | "vault" | "other" {
  // Assemble five branches from the parts bin here, then delete the throw.
  throw new Error("assemble the branch table from the parts bin");
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

// ── RECORDED FIXTURES — real devnet accounts, captured 2026-07-28, except
// "second-vault" (constructed byte-for-byte to the frozen VaultState layout
// for a real derived PDA) and "truncated-vault" (a synthetic layout probe:
// vault-program-owned bytes that are NOT a VaultState) ──────────────────────
const ACCOUNTS: Record<string, AccountInfo> = {
  // FY86s1fAwUiFQTjVFYprsiV6fwNH7e955MSUBo73FP4j — the reference vault.
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
  // EMTqK9Cfac1X7c86GUYG5wwGkqR9HpeYkqUDvjLmhgKi — a different vault.
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
  // 6JFH1dxqiw6Dc81CbWdx4TUT8CvAfgwZg33wQrSytZsU — a plain wallet: exists,
  // zero-length data, system-owned.
  wallet: {
    value: {
      lamports: 3331870864n,
      ownerProgram: "11111111111111111111111111111111",
      executable: false,
      data: [],
    },
  },
  // D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd — the frozen program itself.
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
  // SysvarC1ock11111111111111111111111111111111 — the clock, a real account
  // owned by a program that is neither System nor the vault program.
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
  // Synthetic: owned by the vault program, but 12 bytes — NOT a VaultState.
  // The account your inspector must refuse to decode.
  "truncated-vault": {
    value: {
      lamports: 900000n,
      ownerProgram: "D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd",
      executable: false,
      data: [228, 196, 82, 165, 98, 210, 235, 152, 78, 181, 115, 75],
    },
  },
  // An address nobody has ever funded: the RPC answer is { value: null }.
  missing: { value: null },
};
