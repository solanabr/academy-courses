/**
 * Compute the priority fee, in lamports, for a transaction.
 *
 * The bid is expressed in MICRO-lamports per compute unit, so:
 *
 *   lamports = computeUnits * microLamportsPerCu / 1_000_000
 *
 * Round UP — a partial lamport is still charged as a whole one.
 *
 * Examples:
 *   priorityFeeLamports(200000, 5000) -> 1000
 *   priorityFeeLamports(200000, 0)    -> 0
 */
function priorityFeeLamports(
  computeUnits: number,
  microLamportsPerCu: number
): number {
  return computeUnits * microLamportsPerCu;
}
