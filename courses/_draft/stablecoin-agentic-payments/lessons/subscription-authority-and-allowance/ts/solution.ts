/** USDC has 6 decimals. 1 USDC = 1_000_000 base units. */
const USDC_DECIMALS = 6;

/**
 * Plan the calls that open a Fixed Delegation for one (user, mint) pair.
 */
function planFixedDelegation(
  authorityExists: boolean,
  amountUsdc: number,
  sponsored: boolean
): { steps: string[]; amountBaseUnits: bigint; payer: string } {
  const steps: string[] = [];

  // 1. Derive the user's associated token account for the payment mint.
  steps.push("findAssociatedTokenPda");

  // 2. Derive the SubscriptionAuthority PDA for this (user, tokenMint) pair.
  steps.push("findSubscriptionAuthorityPda");

  // 3. Read the authority before touching it — always.
  steps.push("fetchMaybeSubscriptionAuthority");

  // 4. Initialise it only when the read said it is not there.
  if (!authorityExists) {
    steps.push("initSubscriptionAuthority");
  }

  // 5. Human amount -> base units, as a bigint, exactly once.
  const amountBaseUnits = toBaseUnits(amountUsdc, USDC_DECIMALS);

  // 6. Open the delegation, then draw the first payment down from its cap.
  steps.push("createFixedDelegation");
  steps.push("transferFixed");

  return {
    steps,
    amountBaseUnits,
    payer: sponsored ? "sponsor" : "user",
  };
}

/**
 * Multiply in integer space and round once. Doing the arithmetic in bigint from
 * the start would be better still, but the input here is a JS number, so round
 * before the conversion rather than trusting float multiplication.
 */
function toBaseUnits(amount: number, decimals: number): bigint {
  let factor = 1;
  for (let i = 0; i < decimals; i++) {
    factor *= 10;
  }
  return BigInt(Math.round(amount * factor));
}
