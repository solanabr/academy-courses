// ─────────────────────────────────────────────────────────────────────────────
// TEST HARNESS — do not edit. Grading calls runCase with the name of a legacy
// call sequence (below) and checks the Kit equivalent you produce for every
// step. The right-hand strings are exactly the ones in THE MAP in the lesson —
// this is open book by design.
// ─────────────────────────────────────────────────────────────────────────────
function runCase(planName: string) {
  const plan = LEGACY_PLANS[planName];
  if (!plan) throw new Error("unknown legacy plan: " + planName);
  return modernizePlan(plan);
}

/**
 * TRANSLATION DRILL — fill in THE MAP.
 *
 * `legacy` is a sequence of calls lifted from a v1 codebase. For each one,
 * answer with the Kit equivalent, exactly as written in the lesson's map.
 * Two rows are done for you.
 */
function modernizePlan(
  legacy: string[]
): { legacy: string; kit: string }[] {
  const MAP: Record<string, string> = {
    // Done for you — classes become functions:
    "new Connection(endpoint)": "createSolanaRpc(endpoint)",
    // Done for you — magic endpoints become explicit:
    "clusterApiUrl('devnet')": "an explicit endpoint string",
    // Fill in the rest from THE MAP:
    "new PublicKey(base58)": "address(base58)",
    "Keypair.generate()": "generateKeyPairSigner()",
    "SystemProgram.transfer({...})": "getTransferSolInstruction({...})",
    "sendAndConfirmTransaction(connection, tx, [payer])": "sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })",
    "connection.getAccountInfo(pubkey)": "rpc.getAccountInfo(address).send()",
    "PublicKey.findProgramAddressSync(seeds, programId)": "await getProgramDerivedAddress({ programAddress, seeds })",
    "useWallet() from @solana/wallet-adapter-react": "@solana/kit-plugin-wallet (Wallet Standard)",
  };

  return legacy.map((call) => {
    const kit = MAP[call];
    if (kit === undefined || kit === "") {
      throw new Error("no Kit equivalent recorded for: " + call);
    }
    return { legacy: call, kit };
  });
}

// ── LEGACY CALL SEQUENCES — what you will actually read in the wild ─────────
const LEGACY_PLANS: Record<string, string[]> = {
  // A v1 transfer flow, top to bottom — the shape you studied in lesson 4.
  transfer: [
    "new Connection(endpoint)",
    "clusterApiUrl('devnet')",
    "Keypair.generate()",
    "SystemProgram.transfer({...})",
    "sendAndConfirmTransaction(connection, tx, [payer])",
  ],
  // A v1 account read — lesson 2's territory, in old clothes.
  "account-read": [
    "new Connection(endpoint)",
    "new PublicKey(base58)",
    "connection.getAccountInfo(pubkey)",
  ],
  // A v1 dApp fragment: derive a PDA, reach for the wallet — lessons 3 and 6.
  "pda-dapp": [
    "new PublicKey(base58)",
    "PublicKey.findProgramAddressSync(seeds, programId)",
    "useWallet() from @solana/wallet-adapter-react",
  ],
};
