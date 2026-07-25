/**
 * The payment-rail decision map.
 *
 * WORKED EXAMPLE — this code is complete and every branch below is correct.
 * There is exactly one thing wrong with it, and it is not a branch: it is the
 * ORDER. See the note at the bottom of the function.
 */
function chooseRail(
  cadence: string,
  settlement: string,
  merchantPublishesTerms: boolean
): string {
  // Priced per HTTP request, no account relationship, caller may be a machine.
  // -> x402. Module 2 builds this end to end.
  if (cadence === "per-request") {
    return "x402";
  }

  // Charged again every period. If the merchant publishes terms that many
  // customers subscribe to, that is a subscription plan; if you are charging
  // your own users on your own schedule, it is a recurring delegation whose
  // cap resets each period.
  if (cadence === "periodic") {
    return merchantPublishesTerms ? "subscription-plan" : "recurring-delegation";
  }

  // One capped total, drawn down over many pulls, no refill. An allowance.
  // This is the agent budget in lesson 7.
  if (cadence === "capped-total") {
    return "fixed-delegation";
  }

  // A human approving a single transfer at a checkout. No cap and no schedule
  // to authorise, because the payer is present. -> Solana Pay (as a decision;
  // @solana/pay@1.0.23 peer-depends on @solana/kit ^6.9.0 and this course
  // pins @solana/kit@7.0.0, so it is not installed here).
  if (cadence === "one-off") {
    return "solana-pay";
  }

  // ⚠️ THIS GUARD IS IN THE WRONG PLACE.
  //
  // BCB Resolution 561 (published 2026-04-30, effective 2026-10-01) closes the
  // pattern where a regulated eFX provider takes reais, converts to a
  // stablecoin and settles the obligation abroad on-chain. That is a stop, not
  // a rail — and a stop that runs after the rails have already answered never
  // stops anything.
  //
  // YOUR EDIT: move this block so it runs before any rail is chosen.
  if (settlement === "cross-border-fx") {
    return "blocked-by-bcb-561";
  }

  // A cadence the map does not model. Fail loudly instead of guessing a rail.
  return "unsupported";
}
