// Solution: the helpers BORROW a slice; the caller lends `&latencies` twice.
// Ownership never moves, so `latency_report` can keep using its Vec after
// both calls, no clone, no fight with the borrow checker.
//
// The helpers are nested inside `latency_report` so the graded entry point
// is this file's one top-level `fn`. A nested fn is an ordinary function,
// no captures, the same borrowing rules apply.

fn latency_report(latencies: Vec<u64>, threshold_ms: u64) -> String {
    fn max_latency(latencies: &[u64]) -> u64 {
        let mut max = 0;
        for &l in latencies {
            if l > max {
                max = l;
            }
        }
        max
    }

    fn count_over(latencies: &[u64], threshold_ms: u64) -> u64 {
        let mut n = 0;
        for &l in latencies {
            if l > threshold_ms {
                n += 1;
            }
        }
        n
    }

    let max = max_latency(&latencies);
    let over = count_over(&latencies, threshold_ms);
    format!("max={},over={}", max, over)
}
