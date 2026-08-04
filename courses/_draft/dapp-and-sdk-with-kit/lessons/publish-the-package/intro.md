# Publish It

This is the module milestone: a real, versioned client on the public npm registry, published from CI with build provenance, and installed back from the registry to prove it works for someone who is not you.

> **Milestone honesty.** No block on this platform can verify "this package is on npm" yet, so the milestone is self-reported — you paste the package URL at the end. The work is real; the check is manual.

## Trusted publishing, not a token

The modern path is npm **trusted publishing (OIDC)** from GitHub Actions — no `NPM_TOKEN` secret anywhere. The workflow proves its identity to npm with a short-lived OIDC token GitHub mints for the run, and npm attaches build provenance automatically. A provenance-attested CI publish is exactly what a bounty reviewer wants to see: proof the bytes on the registry were built from the commit you point at.

It needs three things: the workflow requests `permissions: { id-token: write, contents: read }`, the runner has npm CLI ≥ 11.5.1, and you configured the trusted publisher once on the package's page at npmjs.com. The whole flow is browser-only — you create the repo, edit files, and commit the workflow on github.com, and Actions runs `npx codama run js` and `npm publish` for you.

```yaml
# .github/workflows/publish.yml  (ships as prose — you author the decision logic below)
name: publish
on:
  release:
    types: [published]
permissions:
  id-token: write
  contents: read
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: "22", registry-url: "https://registry.npmjs.org" }
      - run: npm ci
      - run: npx codama run js   # regenerate the client from the IDL
      - run: npm publish         # OIDC provenance is automatic; no token
```

The fallback, only if trusted publishing is not set up, is a `NPM_TOKEN` secret plus `npm publish --provenance --access public`.

## The one failure everyone hits

A **scoped** package (`@your-scope/vault-client`) is private by default, and the very first publish needs `--access public` or npm rejects it with a **402 Payment Required** — npm thinks you are trying to publish a private package without a paid plan. It is the single most common first-publish failure. Set access to public on that first publish and it never recurs.

## The exercise

You author the decision logic a publish step runs before it calls `npm publish`: given whether the package is scoped, whether this is the first publish, whether you passed the public-access flag, whether a trusted publisher is configured, and whether the npm CLI is new enough — decide the auth method and catch the 402 before it happens. Signature and rules are in the starter; there is no pattern shown. Write it.
