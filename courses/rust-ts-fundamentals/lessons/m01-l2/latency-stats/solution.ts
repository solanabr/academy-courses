// Latency stats: summarize a batch of probe samples.
// Reference solution.

function latencyStats(csv: string): { min: number; max: number; mean: number; p95: number } {
  const samples = csv
    .split(',')
    .map((s) => Number(s.trim()))
    .sort((a, b) => a - b);

  const n = samples.length;
  const min = samples[0];
  const max = samples[n - 1];

  const sum = samples.reduce((acc, v) => acc + v, 0);
  const mean = Math.round((sum / n) * 100) / 100;

  // Nearest-rank 95th percentile: sort ascending, take index ceil(0.95 * n) - 1.
  const p95 = samples[Math.max(0, Math.ceil(0.95 * n) - 1)];

  return { min, max, mean, p95 };
}
