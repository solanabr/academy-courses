/**
 * Cold-restart resume. The ops bot persists a cursor — the id of the last
 * operation it confirmed. On restart it re-reads the full, ordered operation log
 * and must apply ONLY what it hasn't already done (id > cursor), in order, then
 * advance the cursor to the highest id it just applied. If nothing is new, the
 * cursor is unchanged. This is what "wakes up where it left off" means, and why
 * every op past the cursor must be idempotent or guarded on-chain.
 *
 * The log arrives as a space-separated string of `id:kind` entries, e.g.
 * "3:sol 4:btc" — the parsing into ops is already done for you.
 *
 * As shipped this replays the entire log on every restart and never advances the
 * cursor — a double-execution bug. Fix both.
 */
function resumeFrom(
  cursor: number,
  log: string
): { toApply: { id: number; kind: string }[]; nextCursor: number } {
  const ops = log
    .split(" ")
    .filter(Boolean)
    .map((entry) => {
      const [id, kind] = entry.split(":");
      return { id: Number(id), kind: kind ?? "" };
    });

  // TODO: keep only the operations after the cursor (id > cursor).
  const toApply = ops;

  // TODO: advance the cursor to the highest id actually applied.
  const nextCursor = cursor;

  return { toApply, nextCursor };
}
