/** USDC has 6 decimals. 1 USDC = 1_000_000 base units. */
const USDC_DECIMALS = 6;

/**
 * Plan the calls that open a Fixed Delegation for one (user, mint) pair.
 *
 * @param authorityExists does the SubscriptionAuthority PDA already exist?
 * @param amountUsdc      the cap, in human USDC
 * @param sponsored       true when a sponsor signs as `payer` for the rent
 */
function planFixedDelegation(
  authorityExists: boolean,
  amountUsdc: number,
  sponsored: boolean
): { steps: string[]; amountBaseUnits: bigint; payer: string } {
  const steps: string[] = [];

  // 1. Derive the user's associated token account for the payment mint.
  //    findAssociatedTokenPda({ mint, owner, tokenProgram }) -> [address, bump]
  steps.push("findAssociatedTokenPda");

  // 2. Derive the SubscriptionAuthority PDA for this (user, tokenMint) pair.
  //    findSubscriptionAuthorityPda({ tokenMint, user }) -> [address, bump]
  steps.push("findSubscriptionAuthorityPda");

  // 3. Read the authority before touching it. This step is ALWAYS in the plan —
  //    reading is how you find out whether step 4 is needed.
  //    fetchMaybeSubscriptionAuthority(rpc, pda) -> { exists, data }
  // TODO

  // 4. Initialise the authority ONLY when it does not exist yet. Calling
  //    initSubscriptionAuthority unconditionally is the classic error here.
  // TODO

  // 5. Convert the human amount to base units as a bigint.
  //    There is no decimals conversion anywhere in the SDK — this is it.
  // TODO (and replace the 0n in the return below)

  // 6. Open the delegation, then draw the first payment down from its cap.
  //    createFixedDelegation, then transferFixed.
  // TODO

  return {
    steps,
    amountBaseUnits: 0n,
    payer: sponsored ? "sponsor" : "user",
  };
}
