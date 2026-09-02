/// Admit a new score to a fixed-capacity cabinet high-score board and return
/// the LOWEST score still on the board afterwards, the leaderboard "cutoff".
///
/// The board is a bounded, fixed-size list: exactly the discipline a Pod
/// `Slab<Header, TailItem>` enforces on-chain, where you cannot heap-grow a
/// `Vec` inside an account. Rules:
///   * while the board has fewer than `cap` entries, always admit the score;
///   * once the board is full, admit the score ONLY if it strictly beats the
///     current cutoff (ties do not evict);
///   * keep the board sorted highest-first and never let it exceed `cap`,
///     including when the board handed to you already exceeds it.
///
/// Return the cutoff (the minimum retained score), or 0 for an empty board.
fn admit(board: Vec<u64>, score: u64, cap: usize) -> u64 {
    // TODO: actually admit `score` under the `cap` limit and keep only the
    // top `cap` entries. For now this ignores the new score and the cap, so a
    // real high score never makes the board.
    let _ = (score, cap);
    *board.iter().min().unwrap_or(&0)
}
