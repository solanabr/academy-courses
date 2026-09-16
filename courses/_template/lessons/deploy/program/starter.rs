// TODO: add a public `ping` handler.
//
// Everything below the marker is the verification harness. The grader strips
// whatever harness your submission carries and re-attaches this one before
// compiling, so deleting it does not make the exercise go away.

// ─────────────────────────────────────────────────────────────────────────────
// VERIFICATION HARNESS — DO NOT EDIT ANYTHING BELOW THIS LINE.
// ─────────────────────────────────────────────────────────────────────────────
#[allow(dead_code)]
mod verify {
    const PING: fn() = super::ping;
}
