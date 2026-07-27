# Deploy It

The app is not real until it has a URL someone else can open.

1. Push the repo to GitHub (browser only — you already did this for the package).
2. On vercel.com, **Import** the repo. Vercel detects the framework; you add one thing.
3. Set an environment variable for the **devnet** RPC endpoint — the same endpoint your module-scope client is built with. Do not hardcode it in the bundle; a redeploy to another cluster should be an env change, not a code change.
4. Deploy. Vercel gives you a public URL.
5. Open it, connect your wallet, and send the deposit. The transaction you just assembled by hand goes out — signed by your wallet, against your vault, using the client you published.

Record the URL. You will keep shipping to it: the next three lessons make this same app correct, cheap and honest, and Course 5 adds a paid tier to it.
