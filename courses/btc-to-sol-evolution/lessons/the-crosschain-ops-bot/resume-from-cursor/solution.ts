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

  const toApply = ops.filter((op) => op.id > cursor);
  const nextCursor = toApply.reduce((max, op) => (op.id > max ? op.id : max), cursor);
  return { toApply, nextCursor };
}
