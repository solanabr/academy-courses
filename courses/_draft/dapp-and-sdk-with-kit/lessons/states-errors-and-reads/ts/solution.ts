/**
 * The UI state reducer.
 */
function nextUiState(current: string, event: string): string {
  switch (event) {
    case "submit":
      return "pending";
    case "processed":
      return "processed";
    case "confirmed":
      return "confirmed";
    case "finalized":
      return "finalized";
    case "user-rejected":
      return "idle"; // never reached the network — not a failure
    case "blockhash-expired":
      return "retryable"; // aged out — rebuild and resend
    case "preflight-failure":
    case "program-error":
    case "insufficient-lamports":
      return "failed";
    default:
      return current; // unrecognized event — no change
  }
}
