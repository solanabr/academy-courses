/**
 * One collection attempt against a recurring delegation.
 *
 * The whole model is in the `available` expression: on a period rollover the
 * cap is the full period limit again (semantic 2), and it is the full period
 * limit no matter how many periods were skipped (semantic 3).
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
  const available =
    currentPeriod > lastChargedPeriod
      ? periodLimitBaseUnits
      : periodLimitBaseUnits - chargedInCurrentPeriod;

  const overCap = requestBaseUnits > available;

  // The guard runs first, or it does not run at all.
  if (overCap) {
    return {
      ok: false,
      error: "AMOUNT_EXCEEDS_PERIOD_LIMIT",
      charged: 0,
      remainingThisPeriod: available,
    };
  }

  return {
    ok: true,
    error: "",
    charged: requestBaseUnits,
    remainingThisPeriod: available - requestBaseUnits,
  };
}
