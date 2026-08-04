/**
 * Choose the signer for the features a wallet advertises.
 *
 * `features` is a pipe-joined list of Wallet Standard feature ids, e.g.
 *   "solana:signTransaction|solana:signMessage"
 * Empty string means the wallet advertises no signing feature.
 *
 * This app prefers a modifying signer (it must hold the transaction to size and
 * price it in module 3), so signTransaction is checked before the sending
 * signer, and a message-only wallet cannot send at all.
 */
function selectSigner(features: string): { signer: string; broadcaster: string } {
  const has = (f: string): boolean =>
    features === "" ? false : features.split("|").indexOf(f) !== -1;

  if (has("solana:signTransaction")) return { signer: "modifying", broadcaster: "app" }; // (B) preferred
  if (has("solana:signAndSendTransaction")) return { signer: "sending", broadcaster: "wallet" }; // (A) fallback
  if (has("solana:signMessage")) return { signer: "message", broadcaster: "none" }; // (C) cannot send
  return { signer: "none", broadcaster: "none" }; // (E) nothing usable
}
