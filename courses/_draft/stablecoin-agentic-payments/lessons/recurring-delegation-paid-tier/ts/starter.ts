/**
 * One collection attempt against a recurring delegation.
 *
 * Every line you need is already below. Two of the three `available` candidates
 * are decoys, and the guard sits below the success return instead of above it.
 * Rearrange; do not rewrite.
 *
 * @param periodLimitBaseUnits   amountPerPeriod, in base units
 * @param currentPeriod          index of the period the clock is in now
 * @param lastChargedPeriod      index of the period of the last successful pull
 * @param chargedInCurrentPeriod already collected inside `currentPeriod`
 * @param requestBaseUnits       what this attempt is asking for
 */
function collectRecurring(
  periodLimitBaseUnits: number,
  currentPeriod: number,
  lastChargedPeriod: number,
  chargedInCurrentPeriod: number,
  requestBaseUnits: number
): {
  ok: boolean;
  error: string;
  charged: number;
  remainingThisPeriod: number;
} {
  // --- candidate 1 ----------------------------------------------------------
  // The cap that never resets: subtracts what was taken this period, but never
  // notices that the clock moved on.
  // const available = periodLimitBaseUnits - chargedInCurrentPeriod;

  // --- candidate 2 (currently active) ---------------------------------------
  // The accumulating cap: agrees with the truth whenever no period was skipped,
  // and hands over several periods' worth the moment one was.
  const available =
    periodLimitBaseUnits * (currentPeriod - lastChargedPeriod + 1) -
    chargedInCurrentPeriod;

  // --- candidate 3 ----------------------------------------------------------
  // const available =
  //   currentPeriod > lastChargedPeriod
  //     ? periodLimitBaseUnits
  //     : periodLimitBaseUnits - chargedInCurrentPeriod;

  const overCap = requestBaseUnits > available;

  // --- the next two blocks are in the wrong order ---------------------------

  return {
    ok: true,
    error: "",
    charged: requestBaseUnits,
    remainingThisPeriod: available - requestBaseUnits,
  };

  if (overCap) {
    return {
      ok: false,
      error: "AMOUNT_EXCEEDS_PERIOD_LIMIT",
      charged: 0,
      remainingThisPeriod: available,
    };
  }
}
