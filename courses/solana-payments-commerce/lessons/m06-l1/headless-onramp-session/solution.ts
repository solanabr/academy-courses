// Headless Coinbase Onramp: server-side session request + client onramp URL.
//
// The session token binds the destination address and receivable assets on
// the server. The client URL then carries ONLY the token: a tampered URL
// cannot redirect the funded USDC to a different wallet, because the address
// never travels in the query string.

export interface OnrampInit {
  requestBody: {
    addresses: { address: string; blockchains: string[] }[];
    assets: string[];
  };
  onrampUrl: string;
}

export function initHeadlessOnramp(
  destinationAddress: string,
  sessionToken: string,
  fiatAmount: number
): OnrampInit {
  // The destination will receive USDC on Solana; bind it server-side.
  const requestBody = {
    addresses: [{ address: destinationAddress, blockchains: ['solana'] }],
    assets: ['USDC'],
  };

  // The client URL carries the session token only. The address is bound to
  // the token server-side and is deliberately absent here.
  const query = new URLSearchParams({
    sessionToken,
    defaultNetwork: 'solana',
    defaultAsset: 'USDC',
    fiatCurrency: 'USD',
    presetFiatAmount: String(fiatAmount),
  });
  const onrampUrl = `https://pay.coinbase.com/buy/select-asset?${query.toString()}`;

  return { requestBody, onrampUrl };
}
