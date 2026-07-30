function priorityFeeLamports(
  computeUnits: number,
  microLamportsPerCu: number
): number {
  const MICRO_LAMPORTS_PER_LAMPORT = 1_000_000;
  return Math.ceil((computeUnits * microLamportsPerCu) / MICRO_LAMPORTS_PER_LAMPORT);
}
