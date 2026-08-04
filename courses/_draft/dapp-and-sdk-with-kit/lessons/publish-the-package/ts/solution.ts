/**
 * The publish decision logic.
 */
function publishGate(
  isScoped: boolean,
  isFirstPublish: boolean,
  accessPublicFlag: boolean,
  hasTrustedPublisher: boolean,
  npmCliOk: boolean
): { ok: boolean; error: string; auth: string } {
  const auth = hasTrustedPublisher && npmCliOk ? "oidc" : "token";

  if (isScoped && isFirstPublish && !accessPublicFlag) {
    return { ok: false, error: "E402_SCOPED_NEEDS_ACCESS_PUBLIC", auth };
  }

  return { ok: true, error: "", auth };
}
