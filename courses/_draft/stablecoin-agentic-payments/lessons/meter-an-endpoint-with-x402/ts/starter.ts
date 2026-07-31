// ---------------------------------------------------------------------------
// WORKED EXAMPLE — an x402 v2 challenge builder.
//
// This file is complete and correct except for ONE value. Read it, then change
// PAY_TO to your own devnet address. Nothing else needs to move.
//
// Pinned 2026-07-25: @x402/core 2.19.0, @x402/svm 2.19.0, @solana/kit 7.0.0.
// The grader has no module resolution, so the two constants below are written
// out. In your own server you IMPORT them:
//   import { SOLANA_DEVNET_CAIP2, USDC_DEVNET_ADDRESS } from "@x402/svm";
// ---------------------------------------------------------------------------

/** CAIP-2 chain id: "solana:" + the first 32 chars of the cluster genesis hash. */
const SOLANA_DEVNET_CAIP2 = "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1";

/** Circle's devnet USDC mint. 6 decimals. Paired with the network above. */
const USDC_DEVNET_ADDRESS = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";

// >>> THE ONE VALUE YOU CHANGE <<<
// The wallet that receives the tokens. Use a real base58 devnet address.
const PAY_TO = "REPLACE_WITH_YOUR_DEVNET_WALLET";

interface Route {
  /** Absolute URL of the priced route. Binds the payment to this resource. */
  resource: string;
  /** Human-readable, shown by wallets and agent logs. */
  description: string;
  /** Base units, decimal string. 0.01 USDC at 6 decimals = "10000". */
  priceBaseUnits: string;
}

/** Your price list. Two routes so the builder is driven by data, not by copy-paste. */
const ROUTES: Record<string, Route> = {
  quote: {
    resource: "https://your-app.vercel.app/api/quote",
    description: "One FX quote",
    priceBaseUnits: "10000",
  },
  digest: {
    resource: "https://your-app.vercel.app/api/digest",
    description: "Daily market digest",
    priceBaseUnits: "250000",
  },
};

/**
 * Build the full HTTP 402 response for a priced route.
 *
 * Returns the body (the challenge envelope), the headers, and the base64
 * encoding of the envelope — because some hosts strip bodies from error
 * responses, so a correct server emits the envelope BOTH ways.
 */
function buildPaymentRequired(routeId: string) {
  const route = ROUTES[routeId];
  if (!route) throw new Error("unknown route: " + routeId);

  // The menu. One entry here; a real server offers several so the client can
  // pick a (network, asset) pair it can actually satisfy.
  const envelope = {
    x402Version: 2,
    error: "payment required",
    accepts: [
      {
        // Pay the exact amount. No quoting, no conversion, no partial fills.
        scheme: "exact",
        // network and asset are a PAIR. A mint lives on one cluster.
        network: SOLANA_DEVNET_CAIP2,
        asset: USDC_DEVNET_ADDRESS,
        payTo: PAY_TO,
        // Decimal STRING of base units. Never a float, never a JSON number:
        // u64 amounts stop being exact above 2^53.
        maxAmountRequired: route.priceBaseUnits,
        resource: route.resource,
        description: route.description,
        mimeType: "application/json",
        // How long you will hold the door open between challenge and settlement.
        maxTimeoutSeconds: 60,
      },
    ],
  };

  const encoded = utf8ToBase64(JSON.stringify(envelope));

  return {
    status: 402,
    headers: {
      "content-type": "application/json",
      // Same bytes as the body. Clients behind edge layers that drop error
      // bodies read the envelope from here instead.
      "payment-required": encoded,
    },
    body: envelope,
    encoded,
  };
}

// --- plumbing -------------------------------------------------------------
// In Node or a browser you would use Buffer.from(s).toString("base64") or
// btoa(). Neither exists in this sandbox, so here is base64 in twelve lines.

const B64_ALPHABET =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

function utf8ToBase64(text: string): string {
  const bytes = new TextEncoder().encode(text);
  let out = "";
  for (let i = 0; i < bytes.length; i += 3) {
    const b0 = bytes[i];
    const b1 = i + 1 < bytes.length ? bytes[i + 1] : -1;
    const b2 = i + 2 < bytes.length ? bytes[i + 2] : -1;
    out += B64_ALPHABET[b0 >> 2];
    out += B64_ALPHABET[((b0 & 3) << 4) | (b1 < 0 ? 0 : b1 >> 4)];
    out += b1 < 0 ? "=" : B64_ALPHABET[((b1 & 15) << 2) | (b2 < 0 ? 0 : b2 >> 6)];
    out += b2 < 0 ? "=" : B64_ALPHABET[b2 & 63];
  }
  return out;
}
