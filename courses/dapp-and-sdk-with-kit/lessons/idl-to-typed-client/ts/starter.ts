/**
 * WORKED EXAMPLE — a hand-written stand-in for one Codama-generated builder.
 *
 * This is what `getDepositInstruction(...)` assembles for you from the IDL:
 * a program address, an ORDERED account list where each entry carries a role,
 * and a data payload holding the 8-byte instruction discriminator plus the
 * argument.
 *
 * It is complete except for ONE value: the data payload hardcodes `0n` instead
 * of passing the amount through. Wire the `amountLamports` parameter into the
 * payload, then run it and read the returned object.
 *
 * Note the Kit conventions the real generated code uses:
 *   - addresses are strings (Kit `Address`), never `PublicKey` objects
 *   - the amount is a `bigint`, never a `number`
 *   - the account order is fixed by the IDL and must not be shuffled
 */
function buildDepositInstruction(
  programAddress: string,
  vaultPda: string,
  user: string,
  amountLamports: bigint
): {
  programAddress: string;
  accounts: { address: string; role: string }[];
  data: { discriminator: number[]; amountLamports: bigint };
} {
  // The System Program — deposit moves lamports through a System CPI, so it
  // must be present and read-only.
  const SYSTEM_PROGRAM = "11111111111111111111111111111111";

  // The first 8 bytes Anchor writes to identify the `deposit` instruction.
  // The generated client carries this so you never hand-write it.
  const DEPOSIT_DISCRIMINATOR = [242, 35, 198, 137, 82, 225, 242, 182];

  return {
    programAddress,
    accounts: [
      { address: vaultPda, role: "writable" }, // the vault — receives the lamports
      { address: user, role: "writable-signer" }, // the user — pays the lamports, signs
      { address: SYSTEM_PROGRAM, role: "readonly" }, // the CPI target
    ],
    data: {
      discriminator: DEPOSIT_DISCRIMINATOR,
      amountLamports: 0n, // TODO: pass `amountLamports` through instead of 0n
    },
  };
}
