/**
 * Derive a priority-fee price from recently observed fees.
 */
function priceFee(
  recentFeesPipe: string,
  percentile: number,
  floorMicroLamports: number
): { microLamports: number } {
  // Subgoal 1 — parse the pipe-joined observed fees (done for you).
  const fees = recentFeesPipe === "" ? [] : recentFeesPipe.split("|").map(Number);

  // Subgoal 2 — sort ascending and pick the value at `percentile`.
  const sorted = fees.slice().sort((a, b) => a - b);
  const index = sorted.length === 0 ? -1 : Math.floor((percentile / 100) * (sorted.length - 1));
  const picked = index === -1 ? 0 : sorted[index];

  // Subgoal 3 — floor it so a quiet slot (all zeros) does not produce 0.
  const floored = Math.max(picked, floorMicroLamports);

  // Subgoal 4 — return the price.
  return { microLamports: floored };
}
