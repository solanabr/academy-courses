/**
 * Assemble the deposit transaction message by hand, in four subgoals.
 *
 * `buildDepositInstruction` below is your published wrapper's output — treat it
 * as given. Build the message with the owner as fee payer, the given blockhash
 * as its lifetime, and the deposit instruction appended. Do NOT set a compute
 * unit limit — that overpay is the next lesson's exercise.
 */
function assembleDeposit(
  owner: string,
  amountLamports: bigint,
  blockhash: string
): {
  feePayer: string;
  lifetimeBlockhash: string;
  instructions: { programAddress: string; accounts: { address: string; role: string }[]; data: { amountLamports: bigint } }[];
  computeUnitLimitSet: boolean;
} {
  // Subgoal 1 — set the fee payer to the owner (done for you).
  const feePayer = owner;

  // Subgoal 2 — set the transaction lifetime to the given blockhash.
  // const lifetimeBlockhash = ...

  // Subgoal 3 — append the deposit instruction built from your wrapper.
  // const instructions = [ buildDepositInstruction(owner, amountLamports) ];

  // Subgoal 4 — return the message; leave computeUnitLimitSet as false.
  return {
    feePayer,
    lifetimeBlockhash: "",
    instructions: [],
    computeUnitLimitSet: false,
  };
}

// --- your published wrapper's instruction builder — treat as given -------
function buildDepositInstruction(
  owner: string,
  amountLamports: bigint
): { programAddress: string; accounts: { address: string; role: string }[]; data: { amountLamports: bigint } } {
  return {
    programAddress: "VauLtPr0gram1111111111111111111111111111111",
    accounts: [
      { address: owner, role: "writable-signer" },
      { address: "Vau1t" + owner.slice(0, 38).padEnd(38, "1"), role: "writable" },
      { address: "11111111111111111111111111111111", role: "readonly" },
    ],
    data: { amountLamports },
  };
}
