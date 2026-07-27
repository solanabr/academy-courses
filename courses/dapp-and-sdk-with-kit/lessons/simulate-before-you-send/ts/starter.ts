/**
 * WORKED EXAMPLE — the compute-budget overpay, in arithmetic.
 *
 * A priority fee is charged on the REQUESTED compute-unit limit, not on what the
 * transaction consumes. This compares the fee at the 200,000-CU default against
 * a tightly-sized limit (consumed + a 10% margin).
 *
 * It is complete except for ONE line: `savedLamports` is stubbed to 0. Wire it
 * to the difference between the two fees, then run it on your lesson-7 numbers.
 *
 * @param consumedUnits      compute units the transaction actually used
 * @param microLamportsPerCu the priority-fee price, in micro-lamports per CU
 */
function feeComparison(
  consumedUnits: number,
  microLamportsPerCu: number
): {
  defaultFeeLamports: number;
  tightLimit: number;
  tightFeeLamports: number;
  savedLamports: number;
} {
  const DEFAULT_LIMIT = 200000; // the per-instruction default when you set none
  const tightLimit = Math.ceil(consumedUnits * 1.1); // consumed + ~10% margin

  // fee (lamports) = requested CU * price (micro-lamports/CU) / 1e6
  const defaultFeeLamports = Math.ceil((DEFAULT_LIMIT * microLamportsPerCu) / 1_000_000);
  const tightFeeLamports = Math.ceil((tightLimit * microLamportsPerCu) / 1_000_000);

  const savedLamports = 0; // TODO: defaultFeeLamports - tightFeeLamports

  return { defaultFeeLamports, tightLimit, tightFeeLamports, savedLamports };
}
