/**
 * Assemble the deposit plan — four labeled subgoals.
 *
 * A pure function: the wallet's address, a recent blockhash, and an amount
 * in, a complete unsigned transaction plan out. The wallet signs it; your
 * code never touches a private key. Helpers and recorded constants are
 * provided BELOW the function — read them before you start.
 */
function buildDepositPlan(
  wallet: string,
  blockhash: string,
  lamports: bigint
): {
  feePayer: string;
  lifetimeBlockhash: string;
  instructions: {
    programAddress: string;
    accounts: { address: string; role: string }[];
    data: number[];
  }[];
} {
  // Subgoal 1 (done for you) — the connected wallet pays the fee.
  const feePayer = wallet;

  // Subgoal 2 — the transaction's lifetime is the given recent blockhash.
  const lifetimeBlockhash = blockhash;

  // Subgoal 3 — the account list, in the program's IDL order:
  //   1. the wallet's vault PDA (use vaultFor(wallet)), role "writable"
  //   2. the wallet itself,                      role "writable-signer"
  //   3. SYSTEM_PROGRAM,                         role "readonly"
  // The program reads accounts BY POSITION — the vault must be first.
  const accounts: { address: string; role: string }[] = [
    { address: vaultFor(wallet), role: "writable" },
    { address: wallet, role: "writable-signer" },
    { address: SYSTEM_PROGRAM, role: "readonly" },
  ];

  // Subgoal 4 — the instruction data: the 8-byte DEPOSIT_DISCRIMINATOR,
  // then the amount as u64 little-endian (u64le(lamports) gives the 8
  // bytes — the write-side of your lesson-2 read).
  const data: number[] = [...DEPOSIT_DISCRIMINATOR, ...u64le(lamports)];

  return {
    feePayer,
    lifetimeBlockhash,
    instructions: [{ programAddress: VAULT_PROGRAM, accounts, data }],
  };
}

// ── PROVIDED ────────────────────────────────────────────────────────────────
const VAULT_PROGRAM = "D7ZFoWvEG5NBnkJy6iC98rhwj2qhgq8xhSD42cdTRAQd";
const SYSTEM_PROGRAM = "11111111111111111111111111111111";
// The `deposit` instruction's 8-byte tag, from the IDL you read in lesson 1.
const DEPOSIT_DISCRIMINATOR = [242, 35, 198, 137, 82, 225, 242, 182];

/** The amount as 8 little-endian bytes — lesson 2's DataView, writing. */
function u64le(value: bigint): number[] {
  const bytes = new Uint8Array(8);
  new DataView(bytes.buffer).setBigUint64(0, value, true);
  return Array.from(bytes);
}

/**
 * The wallet's vault PDA — recorded real derivations (lesson 3's math;
 * captured 2026-07-28). In production this is
 * `await getProgramDerivedAddress({ programAddress, seeds })`.
 */
function vaultFor(wallet: string): string {
  const RECORDED: Record<string, string> = {
    "6JFH1dxqiw6Dc81CbWdx4TUT8CvAfgwZg33wQrSytZsU":
      "FY86s1fAwUiFQTjVFYprsiV6fwNH7e955MSUBo73FP4j",
    Hr99MUHbQksoGpkSagrhqzvTgCo1BVEHjnACTSsaEJh8:
      "EMTqK9Cfac1X7c86GUYG5wwGkqR9HpeYkqUDvjLmhgKi",
  };
  const vault = RECORDED[wallet];
  if (!vault) throw new Error("no recorded vault derivation for " + wallet);
  return vault;
}
