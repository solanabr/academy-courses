/**
 * The ergonomic wrapper around the generated client.
 *
 * `deriveVaultPda` and `getDepositInstruction` below are the GENERATED surface —
 * treat them as read-only (in a real package they live in `src/generated`, which
 * codegen overwrites). Your job is the wrapper `deposit`, in four subgoals.
 */
function deposit(
  owner: string,
  amountLamports: bigint
): {
  ok: boolean;
  vaultPda: string;
  instruction: {
    programAddress: string;
    accounts: { address: string; role: string }[];
    data: { discriminator: number[]; amountLamports: bigint };
  } | null;
} {
  // Subgoal 1 — derive the vault PDA from the owner (done for you).
  const vaultPda = deriveVaultPda(owner);

  // Subgoal 2 — narrow the input: a deposit must be a positive bigint (done for you).
  if (typeof amountLamports !== "bigint" || amountLamports <= 0n) {
    return { ok: false, vaultPda, instruction: null };
  }

  // Subgoal 3 — build the instruction from the generated builder.
  const instruction = getDepositInstruction({ owner, vaultPda, amountLamports });

  // Subgoal 4 — return the ergonomic shape: { ok: true, vaultPda, instruction }.
  return { ok: true, vaultPda, instruction };
}

// --- generated surface — do not edit -------------------------------------
function deriveVaultPda(owner: string): string {
  // Stands in for the generated `findVaultPda`. Deterministic in the owner:
  // the same owner always yields the same vault address, a different owner a
  // different one.
  return "Vau1t" + owner.slice(0, 38).padEnd(38, "1");
}

function getDepositInstruction(args: {
  owner: string;
  vaultPda: string;
  amountLamports: bigint;
}): {
  programAddress: string;
  accounts: { address: string; role: string }[];
  data: { discriminator: number[]; amountLamports: bigint };
} {
  return {
    programAddress: "VauLtPr0gram1111111111111111111111111111111",
    accounts: [
      { address: args.owner, role: "writable-signer" },
      { address: args.vaultPda, role: "writable" },
      { address: "11111111111111111111111111111111", role: "readonly" },
    ],
    data: { discriminator: [242, 35, 198, 137, 82, 225, 242, 182], amountLamports: args.amountLamports },
  };
}
