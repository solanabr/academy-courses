/**
 * Derive a priority-fee price from recently observed fees.
 *
 * `recentFeesPipe` is a pipe-joined list of observed micro-lamport-per-CU fees
 * for your writable accounts, e.g. "0|100|300|500". Build the price in four
 * numbered subgoals.
 *
 * @param recentFeesPipe    observed fees, pipe-joined
 * @param percentile        0..100 — which observed fee to take
 * @param floorMicroLamports minimum price so a quiet slot does not yield 0
 */
function priceFee(
  recentFeesPipe: string,
  percentile: number,
  floorMicroLamports: number
): { microLamports: number } {
  // Subgoal 1 — parse the pipe-joined observed fees (done for you).
  const fees = recentFeesPipe === "" ? [] : recentFeesPipe.split("|").map(Number);

  // Subgoal 2 — sort ascending and pick the value at `percentile`.
  //   index = Math.floor((percentile / 100) * (fees.length - 1))
  // const picked = ...

  // Subgoal 3 — floor it so a quiet slot (all zeros) does not produce 0.
  // const floored = Math.max(picked, floorMicroLamports)

  // Subgoal 4 — return the price.
  return { microLamports: 0 };
}
