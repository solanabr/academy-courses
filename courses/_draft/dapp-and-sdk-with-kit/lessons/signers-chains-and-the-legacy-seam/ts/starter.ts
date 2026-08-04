/**
 * Choose the signer for the features a wallet advertises.
 *
 * `features` is a pipe-joined list of Wallet Standard feature ids, e.g.
 *   "solana:signTransaction|solana:signMessage"
 * Empty string means the wallet advertises no signing feature.
 *
 * Return { signer, broadcaster }:
 *   - "solana:signTransaction"        -> signer "modifying", broadcaster "app"
 *   - "solana:signAndSendTransaction" -> signer "sending",   broadcaster "wallet"
 *   - "solana:signMessage" only       -> signer "message",   broadcaster "none"
 *   - nothing usable                  -> signer "none",      broadcaster "none"
 *
 * PARSONS: the branches are all here but scrambled, plus one decoy. Order them
 * so THIS app's preference wins — a modifying signer whenever the wallet can
 * offer one, because module 3 must still hold the transaction to size and price
 * it before broadcast.
 */
function selectSigner(features: string): { signer: string; broadcaster: string } {
  const has = (f: string): boolean =>
    features === "" ? false : features.split("|").indexOf(f) !== -1;

  // --- reorder these branches ---------------------------------------------
  if (has("solana:signAndSendTransaction")) return { signer: "sending", broadcaster: "wallet" }; // (A)
  if (has("solana:signTransaction")) return { signer: "modifying", broadcaster: "app" }; // (B)
  if (has("solana:signMessage")) return { signer: "message", broadcaster: "none" }; // (C)
  // if (has("solana:signMessage")) return { signer: "sending", broadcaster: "wallet" }; // (D) DECOY — a message signer cannot send
  return { signer: "none", broadcaster: "none" }; // (E)
}
