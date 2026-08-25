/// Admit a new score to a fixed-capacity cabinet high-score board and return
/// the LOWEST score still on the board afterwards, the leaderboard "cutoff".
///
/// The board is a bounded, fixed-size list: exactly the discipline a Pod
/// `Slab<Header, TailItem>` enforces on-chain, where you cannot heap-grow a
/// `Vec` inside an account. Rules:
///   * while the board has fewer than `cap` entries, always admit the score;
///   * once the board is full, admit the score ONLY if it strictly beats the
///     current cutoff (ties do not evict);
///   * keep the board sorted highest-first and never let it exceed `cap`.
///
/// Return the cutoff (the minimum retained score), or 0 for an empty board.
pub fn admit(mut board: Vec<u64>, score: u64, cap: usize) -> u64 {
    if cap == 0 {
        return 0;
    }

    if board.len() < cap {
        board.push(score);
    } else {
        // Board full: only a score that strictly beats the cutoff gets in.
        let cutoff = *board.iter().min().unwrap_or(&0);
        if score > cutoff {
            let pos = board.iter().position(|&s| s == cutoff).unwrap();
            board[pos] = score;
        }
    }

    board.sort_unstable_by(|a, b| b.cmp(a));
    board.truncate(cap);
    *board.last().unwrap_or(&0)
}
