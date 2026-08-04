// ─────────────────────────────────────────────────────────────────────────────
// TEST HARNESS — do not edit. Grading calls runCase with an owner address and
// injects `derive`: a lookup over REAL getProgramDerivedAddress outputs,
// recorded on 2026-07-28 (kit 7.0.0). No hashing happens in the sandbox; the
// recorded map holds the correct derivation for two owners, the wrong-seed-
// order result, and the wrong-program result — all real.
// ─────────────────────────────────────────────────────────────────────────────
function runCase(ownerAddress: string) {
  return deriveVaultPda(ownerAddress, derive);
}

/**
 * Reproduce the vault PDA for `owner`.
 *
 * The five candidate lines are in the PARTS BIN below, commented out and
 * shuffled. THREE belong, in the right order. Two are decoys:
 *   - one flips the seed order — it derives a real address that is not the
 *     vault (seed order is part of the address);
 *   - one passes the owner where the PROGRAM address belongs — a category
 *     error the lookup rejects loudly.
 * Uncomment the lines you want inside the function, in order. Do not write
 * lines that are not in the bin.
 *
 * ┌─ PARTS BIN ──────────────────────────────────────────────────────────┐
 * │ (A)  const seeds = [utf8("vault"), addressBytes(owner)];             │
 * │ (B)  const [address, bump] = derive({ programAddress: owner, seeds });│
 * │ (C)  const seeds = [addressBytes(owner), utf8("vault")];             │
 * │ (D)  return { address, bump };                                       │
 * │ (E)  const [address, bump] = derive({                                │
 * │        programAddress: VAULT_PROGRAM, seeds });                      │
 * └──────────────────────────────────────────────────────────────────────┘
 */
function deriveVaultPda(
  owner: string,
  derive: (input: { programAddress: string; seeds: string[] }) => [
    string,
    number,
  ]
): { address: string; bump: number } {
  // (A) — the program's seed order: the constant "vault" first, then the owner.
  const seeds = [utf8("vault"), addressBytes(owner)];
  // (E) — derive under the PROGRAM's address. (B) was the decoy: the owner is
  // a wallet, not a program, and the lookup rejects it as a category error.
  // (C) was the other decoy: [owner, "vault"] derives a real address that is
  // simply not the vault — seed order is part of the address.
  const [address, bump] = derive({ programAddress: VAULT_PROGRAM, seeds });
  // (D)
  return { address, bump };
}

// ── PROVIDED — seed encoders (stand-ins for getUtf8Encoder/getAddressEncoder;
// the real ones return byte arrays, these return tagged strings the recorded
// lookup understands) ────────────────────────────────────────────────────────
const VAULT_PROGRAM = "D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd";
function utf8(s: string): string {
  return "utf8:" + s;
}
function addressBytes(addr: string): string {
  return "address:" + addr;
}

// ── RECORDED DERIVATIONS — real getProgramDerivedAddress outputs ────────────
const DERIVATIONS: Record<string, [string, number]> = {
  // correct: ["vault", owner] under the vault program
  "D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd|utf8:vault,address:6JFH1dxqiw6Dc81CbWdx4TUT8CvAfgwZg33wQrSytZsU":
    ["FY86s1fAwUiFQTjVFYprsiV6fwNH7e955MSUBo73FP4j", 255],
  "D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd|utf8:vault,address:Hr99MUHbQksoGpkSagrhqzvTgCo1BVEHjnACTSsaEJh8":
    ["EMTqK9Cfac1X7c86GUYG5wwGkqR9HpeYkqUDvjLmhgKi", 252],
  // wrong seed order: [owner, "vault"] — a real address, just not the vault
  "D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd|address:6JFH1dxqiw6Dc81CbWdx4TUT8CvAfgwZg33wQrSytZsU,utf8:vault":
    ["6jBWSvMq5a1qiS61uUajhmp1KQYYZWriHc2fo6S4jYjJ", 254],
  "D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd|address:Hr99MUHbQksoGpkSagrhqzvTgCo1BVEHjnACTSsaEJh8,utf8:vault":
    ["7dpNqGM87gD1cVfnmwjc12rpeNiDdJStK4Jo4rtPs9R4", 254],
  // wrong program: same seeds under the System Program — different address
  "11111111111111111111111111111111|utf8:vault,address:6JFH1dxqiw6Dc81CbWdx4TUT8CvAfgwZg33wQrSytZsU":
    ["9tk79Wfc9fQZBo5pggDqEnTRyhRu2pajB9VcgGe6tH3m", 255],
};

function derive(input: {
  programAddress: string;
  seeds: string[];
}): [string, number] {
  const key = input.programAddress + "|" + input.seeds.join(",");
  const hit = DERIVATIONS[key];
  if (!hit) {
    throw new Error(
      "no recorded derivation for programAddress=" +
        input.programAddress +
        " seeds=[" +
        input.seeds.join(", ") +
        "] — is the program address actually a program, and are the seeds the ones the program defined?"
    );
  }
  return [hit[0], hit[1]];
}
