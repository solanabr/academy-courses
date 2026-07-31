/**
 * Plan the vault deposit the way the kit bot assembles it: fee payer = owner,
 * lifetime = the recent blockhash, exactly one appended deposit instruction, and a
 * "confirmed" commitment. Pure — no RPC, no signing, no imports — so it grades
 * deterministically. In the real bot these are the `pipe(...)` steps around
 * `getDepositInstructionAsync` in deposit.ts.
 */
function planVaultDeposit(
  owner: string,
  amountLamports: bigint,
  recentBlockhash: string
): {
  feePayer: string;
  lifetime: string;
  commitment: string;
  instructions: { program: string; accounts: { address: string; role: string }[]; data: { amount: bigint } }[];
} {
  // Subgoal 1 — the fee payer is the owner (done for you).
  const feePayer = owner;

  // Subgoal 2 — the transaction lifetime is the recent blockhash.
  const lifetime = recentBlockhash;

  // Subgoal 3 — append exactly the deposit instruction from your generated client.
  const instructions = [depositInstruction(owner, amountLamports)];

  // Subgoal 4 — confirm at the "confirmed" commitment.
  const commitment = "confirmed";

  return { feePayer, lifetime, commitment, instructions };
}

// --- your Codama-generated client's builder — treat as given ----------------
// (stands in for getDepositInstructionAsync, which derives the vault + vault_state
//  PDAs from the owner and lists them with the right roles).
function depositInstruction(
  owner: string,
  amount: bigint
): { program: string; accounts: { address: string; role: string }[]; data: { amount: bigint } } {
  const vaultState = "VauSt" + owner.slice(0, 38).padEnd(38, "1");
  const vault = "Vau1t" + owner.slice(0, 38).padEnd(38, "1");
  return {
    program: "VauLtPr0gram1111111111111111111111111111111",
    accounts: [
      { address: owner, role: "writable-signer" },
      { address: vaultState, role: "writable" },
      { address: vault, role: "writable" },
      { address: "11111111111111111111111111111111", role: "readonly" },
    ],
    data: { amount },
  };
}
