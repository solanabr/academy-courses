// ---------------------------------------------------------------------------
// The agent's call loop. You write this one.
//
// Pinned 2026-07-25: @solana/subscriptions 0.4.0 (exactly), @solana/kit 7.0.0,
// @x402/{core,svm,fetch} 2.19.0.
//
// In your own project each iteration is: call the metered endpoint through the
// wrapped fetch from lesson 6, and settle the 402 with a `transferFixed` draw
// against the Fixed Delegation you opened in lesson 3. Here the network round
// trips are already captured — what you implement is the part that is actually
// yours: the spend / halt / report decision logic.
// ---------------------------------------------------------------------------

/** One settlement attempt as the delegation program answered it. */
interface Settlement {
  ok: boolean;
  /** Present when ok. */
  signature?: string;
  /** Present when not ok. The program's error, verbatim. */
  error?: { code: string };
}

interface Scenario {
  /** `remaining_amount` on the Fixed Delegation. Base units, decimal string. */
  remainingBaseUnits: string;
  /** What one call costs. Base units, decimal string. */
  pricePerCall: string;
  /** The agent's task: at most this many calls. */
  maxCalls: number;
  /** Settlement responses, consumed in order — one per call actually attempted. */
  settlements: Settlement[];
}

interface AgentReport {
  /** Calls that completed AND settled. */
  calls: number;
  /** Total drawn from the delegation. Base units, decimal string. */
  spent: string;
  /** What is left on the delegation. Base units, decimal string. */
  remaining: string;
  /** Settlement signatures, in order, one per successful call. */
  signatures: string[];
  /** True when the loop stopped before finishing its task. */
  halted: boolean;
  reason: "completed" | "budget-exhausted" | "refused";
  /** The program's error code when reason is "refused", otherwise null. */
  errorCode: string | null;
}

const SCENARIOS: Record<string, Scenario> = {
  // Budget comfortably covers the task.
  "s-completed": {
    remainingBaseUnits: "1000000",
    pricePerCall: "10000",
    maxCalls: 3,
    settlements: [
      { ok: true, signature: "sig-c1" },
      { ok: true, signature: "sig-c2" },
      { ok: true, signature: "sig-c3" },
    ],
  },

  // The task needs 5 calls. The allowance covers 2 and a half.
  // Note that the fixtures would happily settle all five.
  "s-exhausted": {
    remainingBaseUnits: "25000",
    pricePerCall: "10000",
    maxCalls: 5,
    settlements: [
      { ok: true, signature: "sig-e1" },
      { ok: true, signature: "sig-e2" },
      { ok: true, signature: "sig-e3" },
      { ok: true, signature: "sig-e4" },
      { ok: true, signature: "sig-e5" },
    ],
  },

  // The program refuses the third draw. The fourth response is a success —
  // it is there to catch a loop that keeps going after a refusal.
  "s-refused": {
    remainingBaseUnits: "1000000",
    pricePerCall: "250000",
    maxCalls: 4,
    settlements: [
      { ok: true, signature: "sig-r1" },
      { ok: true, signature: "sig-r2" },
      { ok: false, error: { code: "AmountExceedsTotalLimit" } },
      { ok: true, signature: "sig-r4" },
    ],
  },

  // Nothing left at all.
  "s-broke": {
    remainingBaseUnits: "0",
    pricePerCall: "10000",
    maxCalls: 2,
    settlements: [{ ok: true, signature: "sig-b1" }],
  },

  // A u64-sized allowance spent to exactly zero. Base units are u64; Number
  // stops being exact above 2^53, so this one only comes out right if the
  // arithmetic never becomes a Number.
  "s-precision": {
    remainingBaseUnits: "18446744073709551615",
    pricePerCall: "6148914691236517205",
    maxCalls: 3,
    settlements: [
      { ok: true, signature: "sig-p1" },
      { ok: true, signature: "sig-p2" },
      { ok: true, signature: "sig-p3" },
    ],
  },
};

/**
 * Run the agent's call loop against a scenario and report what happened.
 *
 * SPEC — the loop performs at most `maxCalls` calls, and before each one:
 *
 *   1. If `remaining` is less than `pricePerCall`, STOP. Do not attempt the
 *      call and do not consume a settlement. Report halted: true, reason
 *      "budget-exhausted", errorCode null. A transaction you already know the
 *      program will refuse still costs you a fee.
 *   2. Otherwise consume the next settlement, in order.
 *      - ok: count the call, add the price to `spent`, subtract it from
 *        `remaining`, append the signature, continue.
 *      - not ok: STOP IMMEDIATELY. Do not retry, do not skip it, do not touch
 *        `spent` or `remaining`. Report halted: true, reason "refused", and
 *        errorCode set to the program's code verbatim.
 *   3. If all `maxCalls` calls settle, report halted: false, reason
 *      "completed", errorCode null.
 *
 * ACCEPTANCE CRITERIA
 *   - `spent`, `remaining` and `pricePerCall` are base units. Do the arithmetic
 *     in BigInt and return decimal strings. A u64 allowance does not survive
 *     Number.
 *   - `signatures.length` always equals `calls`.
 *   - `errorCode` is non-null only when reason is "refused", and it is the code
 *     the program returned — never a string you invented.
 *   - The function returns a report in every case. It never throws.
 */
function runAgentLoop(scenarioId: string): AgentReport {
  const scenario = SCENARIOS[scenarioId];

  // Your code here.

  return {
    calls: 0,
    spent: "0",
    remaining: scenario.remainingBaseUnits,
    signatures: [],
    halted: false,
    reason: "completed",
    errorCode: null,
  };
}
