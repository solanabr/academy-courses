// ─────────────────────────────────────────────────────────────────────────────
// TEST HARNESS — do not edit. Grading calls runCase with the name of a
// RECORDED simulation (run for real against devnet on 2026-07-28) and a
// priority-fee price in micro-lamports per compute unit.
// ─────────────────────────────────────────────────────────────────────────────
function runCase(simName: string, microLamportsPerCu: bigint) {
  const sim = SIMULATIONS[simName];
  if (!sim) throw new Error("unknown simulation: " + simName);
  return planBudget(sim, microLamportsPerCu);
}

/**
 * WORKED EXAMPLE — complete except for ONE line.
 *
 * Given a simulation result and a priority-fee price, produce the full v0
 * cost plan:
 *   - computeUnitLimit — what the transaction will REQUEST: the simulated
 *     consumption plus a 10% margin (integer bigint math).
 *   - signatureFee     — 5,000 lamports per signature; this plan has one.
 *   - priorityFee      — price × requested limit ÷ 1,000,000, rounded UP.
 *   - totalFeeLamports — the sum.
 */
function planBudget(
  sim: { unitsConsumed: bigint },
  microLamportsPerCu: bigint
): {
  computeUnitLimit: bigint;
  signatureFee: bigint;
  priorityFee: bigint;
  totalFeeLamports: bigint;
} {
  // ── THE BLANK ──────────────────────────────────────────────────────────
  // Size the requested limit from what simulation measured, plus a 10%
  // margin: consumed + consumed / 10n. (Why request MORE than consumed?
  // Compute varies slightly run to run. Why not just leave the default?
  // Because the default is 200,000 and you pay the priority fee on it.)
  const computeUnitLimit = sim.unitsConsumed + sim.unitsConsumed / 10n;

  // One signer pays 5,000 lamports, fixed. (Half is burned.)
  const signatureFee = 5_000n;

  // The priority fee is charged on the REQUESTED limit, not on consumption.
  // Micro-lamports → lamports is a ÷ 1,000,000, rounded up: the network
  // never rounds in your favor.
  const priorityFee =
    (computeUnitLimit * microLamportsPerCu + 999_999n) / 1_000_000n;

  const totalFeeLamports = signatureFee + priorityFee;
  return { computeUnitLimit, signatureFee, priorityFee, totalFeeLamports };
}

// ── RECORDED SIMULATIONS — real devnet runs, 2026-07-28 ─────────────────────
// "vault-deposit" is the reference vault's deposit instruction — the exact
// transaction you assemble and send in lesson 6. "sol-transfer" is a bare
// System Program transfer, for scale.
const SIMULATIONS: Record<string, { unitsConsumed: bigint }> = {
  "vault-deposit": { unitsConsumed: 6_697n },
  "sol-transfer": { unitsConsumed: 150n },
};
