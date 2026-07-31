/**
 * The publish decision logic. INDEPENDENT-WRITE — no pattern is shown; write it
 * from the spec.
 *
 * Decide the auth method and pre-empt the scoped-first-publish 402.
 *
 * @param isScoped            is the package name scoped, e.g. "@scope/name"
 * @param isFirstPublish      is this the first publish of this package
 * @param accessPublicFlag    did the command pass `--access public`
 * @param hasTrustedPublisher is a trusted publisher configured on npmjs.com
 * @param npmCliOk            is the npm CLI >= 11.5.1
 *
 * Return `{ ok, error, auth }` where:
 *   - auth is "oidc" when a trusted publisher is configured AND the CLI is new
 *     enough; otherwise "token" (the fallback route).
 *   - a scoped package's FIRST publish WITHOUT the public-access flag must be
 *     stopped: return ok=false and error="E402_SCOPED_NEEDS_ACCESS_PUBLIC".
 *   - every other case returns ok=true and error="".
 */
function publishGate(
  isScoped: boolean,
  isFirstPublish: boolean,
  accessPublicFlag: boolean,
  hasTrustedPublisher: boolean,
  npmCliOk: boolean
): { ok: boolean; error: string; auth: string } {
  // Write it.
  return { ok: true, error: "", auth: "token" };
}
