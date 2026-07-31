/**
 * The payment-rail decision map.
 *
 * The compliance gate runs FIRST. A requirement whose settlement leg is a
 * regulated cross-border FX conversion has no rail — it has a stop.
 */
function chooseRail(
  cadence: string,
  settlement: string,
  merchantPublishesTerms: boolean
): string {
  // BCB Resolution 561 (published 2026-04-30, effective 2026-10-01): a
  // regulated eFX provider may not take reais, convert to USDT/USDC/BTC and
  // settle abroad on-chain. Gate, not fallback.
  if (settlement === "cross-border-fx") {
    return "blocked-by-bcb-561";
  }

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

  // A cadence the map does not model. Fail loudly instead of guessing a rail.
  return "unsupported";
}
