// ---------------------------------------------------------------------------
// REFERENCE SOLUTION — read a 402 challenge and decide how to pay it.
//
// Blocks 5, 2, 7, 1, 8, 4 in that order. Blocks 3 and 6 were the wrong ones:
//   3 — retries before decoding: assumes v2, assumes the body, and takes
//       accepts[0] sight unseen, which on every capture here is the EVM entry.
//   6 — selects on the eip155 namespace: a chain you cannot pay from a Solana
//       wallet.
//
// Pinned 2026-07-25: @x402/core 2.19.0, @x402/svm 2.19.0, @x402/fetch 2.19.0.
// ---------------------------------------------------------------------------

const SOLANA_DEVNET_CAIP2 = "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1";
const USDC_DEVNET_ADDRESS = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";

interface Accept {
  scheme: string;
  network: string;
  asset: string;
  payTo: string;
  maxAmountRequired: string;
  resource: string;
  description: string;
  mimeType: string;
  maxTimeoutSeconds: number;
}

interface Report {
  paid: boolean;
  x402Version: number | null;
  envelopeFrom: "body" | "header" | null;
  paymentHeader: "X-PAYMENT" | "PAYMENT" | null;
  selected: Accept | null;
}

interface CapturedResponse {
  status: number;
  headers: Record<string, string>;
  body: string;
}

const CAPTURES: Record<string, CapturedResponse> = {
  "plain-200": {
    status: 200,
    headers: { "content-type": "application/json" },
    body: `{"pair":"USDC/BRL","rate":"5.41"}`,
  },

  "v1-body": {
    status: 402,
    headers: { "content-type": "application/json" },
    body: `{"x402Version":1,"error":"X-PAYMENT header is required","accepts":[{"scheme":"exact","network":"eip155:8453","asset":"0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913","payTo":"0x1a2B3c4D5e6F708192a3B4c5D6e7F8091A2b3C4d","maxAmountRequired":"50000","resource":"https://api.example.dev/digest","description":"Daily market digest","mimeType":"application/json","maxTimeoutSeconds":60},{"scheme":"exact","network":"solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1","asset":"4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU","payTo":"B7o8NfV81HzjuZFWQTTx3Xdvh77Dqoajwib3kWEnvzJF","maxAmountRequired":"10000","resource":"https://api.example.dev/quote","description":"One FX quote","mimeType":"application/json","maxTimeoutSeconds":60}]}`,
  },

  "v2-header": {
    status: 402,
    headers: {
      "payment-required":
        "eyJ4NDAyVmVyc2lvbiI6MiwiZXJyb3IiOiJwYXltZW50IHJlcXVpcmVkIiwiYWNjZXB0cyI6W3sic2NoZW1lIjoiZXhhY3QiLCJuZXR3b3JrIjoiZWlwMTU1Ojg0NTMiLCJhc3NldCI6IjB4ODMzNTg5ZkNENmVEYjZFMDhmNGM3QzMyRDRmNzFiNTRiZEEwMjkxMyIsInBheVRvIjoiMHgxYTJCM2M0RDVlNkY3MDgxOTJhM0I0YzVENmU3RjgwOTFBMmIzQzRkIiwibWF4QW1vdW50UmVxdWlyZWQiOiI1MDAwMCIsInJlc291cmNlIjoiaHR0cHM6Ly9hcGkuZXhhbXBsZS5kZXYvZGlnZXN0IiwiZGVzY3JpcHRpb24iOiJEYWlseSBtYXJrZXQgZGlnZXN0IiwibWltZVR5cGUiOiJhcHBsaWNhdGlvbi9qc29uIiwibWF4VGltZW91dFNlY29uZHMiOjYwfSx7InNjaGVtZSI6ImV4YWN0IiwibmV0d29yayI6InNvbGFuYTo1ZXlrdDRVc0Z2OFA4TkpkVFJFcFkxdnpxS3FaS3ZkcCIsImFzc2V0IjoiRVBqRldkZDVBdWZxU1NxZU0ycU4xeHp5YmFwQzhHNHdFR0drWnd5VER0MXYiLCJwYXlUbyI6IkI3bzhOZlY4MUh6anVaRldRVFR4M1hkdmg3N0Rxb2Fqd2liM2tXRW52ekpGIiwibWF4QW1vdW50UmVxdWlyZWQiOiI1MDAwMCIsInJlc291cmNlIjoiaHR0cHM6Ly9hcGkuZXhhbXBsZS5kZXYvZGlnZXN0IiwiZGVzY3JpcHRpb24iOiJEYWlseSBtYXJrZXQgZGlnZXN0IiwibWltZVR5cGUiOiJhcHBsaWNhdGlvbi9qc29uIiwibWF4VGltZW91dFNlY29uZHMiOjYwfSx7InNjaGVtZSI6ImV4YWN0IiwibmV0d29yayI6InNvbGFuYTpFdFdUUkFCWmFZcTZpTWZlWUtvdVJ1MTY2VlUyeHFhMSIsImFzc2V0IjoiNHpNTUM5c3J0NVJpNVgxNEdBZ1hoYUhpaTNHblBBRUVSWVBKZ1pKRG5jRFUiLCJwYXlUbyI6IkI3bzhOZlY4MUh6anVaRldRVFR4M1hkdmg3N0Rxb2Fqd2liM2tXRW52ekpGIiwibWF4QW1vdW50UmVxdWlyZWQiOiI1MDAwMCIsInJlc291cmNlIjoiaHR0cHM6Ly9hcGkuZXhhbXBsZS5kZXYvZGlnZXN0IiwiZGVzY3JpcHRpb24iOiJEYWlseSBtYXJrZXQgZGlnZXN0IiwibWltZVR5cGUiOiJhcHBsaWNhdGlvbi9qc29uIiwibWF4VGltZW91dFNlY29uZHMiOjYwfV19",
    },
    body: "",
  },

  "v2-evm-only": {
    status: 402,
    headers: {
      "payment-required":
        "eyJ4NDAyVmVyc2lvbiI6MiwiZXJyb3IiOiJwYXltZW50IHJlcXVpcmVkIiwiYWNjZXB0cyI6W3sic2NoZW1lIjoiZXhhY3QiLCJuZXR3b3JrIjoiZWlwMTU1Ojg0NTMiLCJhc3NldCI6IjB4ODMzNTg5ZkNENmVEYjZFMDhmNGM3QzMyRDRmNzFiNTRiZEEwMjkxMyIsInBheVRvIjoiMHgxYTJCM2M0RDVlNkY3MDgxOTJhM0I0YzVENmU3RjgwOTFBMmIzQzRkIiwibWF4QW1vdW50UmVxdWlyZWQiOiI1MDAwMCIsInJlc291cmNlIjoiaHR0cHM6Ly9hcGkuZXhhbXBsZS5kZXYvZGlnZXN0IiwiZGVzY3JpcHRpb24iOiJEYWlseSBtYXJrZXQgZGlnZXN0IiwibWltZVR5cGUiOiJhcHBsaWNhdGlvbi9qc29uIiwibWF4VGltZW91dFNlY29uZHMiOjYwfV19",
    },
    body: "",
  },
};

function handle402(captureId: string): Report {
  const res = CAPTURES[captureId];

  // ---- BLOCK 5 — detect -------------------------------------------------
  // Nothing else runs unless the server actually asked for payment.
  if (res.status !== 402) {
    return { paid: false, x402Version: null, envelopeFrom: null, paymentHeader: null, selected: null };
  }

  // ---- BLOCK 2 — locate -------------------------------------------------
  // The envelope is in the body, EXCEPT when the body is empty, in which case
  // it is base64 in the payment-required header. Check both, always.
  const headerEnvelope = res.headers["payment-required"];
  const fromHeader = res.body.length === 0 && typeof headerEnvelope === "string";

  // ---- BLOCK 7 — decode -------------------------------------------------
  const raw = fromHeader ? base64ToUtf8(headerEnvelope) : res.body;
  const envelope = JSON.parse(raw);

  // ---- BLOCK 1 — version ------------------------------------------------
  // The envelope names its own wire version, and the version decides the
  // request header the retry has to use.
  const paymentHeader: "X-PAYMENT" | "PAYMENT" =
    envelope.x402Version === 1 ? "X-PAYMENT" : "PAYMENT";
  const envelopeFrom: "header" | "body" = fromHeader ? "header" : "body";

  // ---- BLOCK 8 — select -------------------------------------------------
  // accepts[] is a menu. Filter on scheme, network AND asset. Never index it.
  const selected = envelope.accepts.find(
    (entry: Accept) =>
      entry.scheme === "exact" &&
      entry.network === SOLANA_DEVNET_CAIP2 &&
      entry.asset === USDC_DEVNET_ADDRESS
  );
  if (!selected) {
    return { paid: false, x402Version: envelope.x402Version, envelopeFrom, paymentHeader, selected: null };
  }

  // ---- BLOCK 4 — retry --------------------------------------------------
  // In the live client this is where you sign the payment for `selected` and
  // re-issue the request carrying it in `paymentHeader`.
  return { paid: true, x402Version: envelope.x402Version, envelopeFrom, paymentHeader, selected };
}

// --- plumbing -------------------------------------------------------------

const B64_ALPHABET =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

function base64ToUtf8(encoded: string): string {
  const bytes: number[] = [];
  let buffer = 0;
  let bits = 0;
  for (let i = 0; i < encoded.length; i++) {
    const value = B64_ALPHABET.indexOf(encoded[i]);
    if (value < 0) continue;
    buffer = (buffer << 6) | value;
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      bytes.push((buffer >> bits) & 0xff);
    }
  }
  return new TextDecoder().decode(new Uint8Array(bytes));
}
