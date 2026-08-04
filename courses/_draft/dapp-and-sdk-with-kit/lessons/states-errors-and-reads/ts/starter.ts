/**
 * The UI state reducer. INDEPENDENT-WRITE — write it from the spec; no pattern
 * is shown, and it is graded on hidden fixtures.
 *
 * Return the next UI state given the current state and one event.
 *
 * Lifecycle events advance the ladder:
 *   "submit"    -> "pending"
 *   "processed" -> "processed"
 *   "confirmed" -> "confirmed"
 *   "finalized" -> "finalized"
 *
 * Error events map to a state that is HONEST about what happened:
 *   "user-rejected"        -> "idle"       (never reached the network; NOT a failure)
 *   "blockhash-expired"    -> "retryable"  (aged out; rebuild and resend)
 *   "preflight-failure"    -> "failed"     (your bug; surface the logs)
 *   "program-error"        -> "failed"     (decoded VaultError; a real reason)
 *   "insufficient-lamports"-> "failed"
 *
 * Any unrecognized event leaves the state unchanged.
 */
function nextUiState(current: string, event: string): string {
  // Write it.
  return current;
}
