// Anchor derives every 8-byte discriminator by hashing a NAMESPACED preimage:
// sha256("<namespace>:<Name>")[..8]. The bytes come later; the preimage STRING is
// the part you have to get right, and one of the three namespaces is a classic trap.
//
// Return the exact preimage string Anchor hashes for each item kind:
//   - an account struct       -> "account:<Name>"
//   - an instruction handler  -> "global:<Name>"     <-- NOT "instruction:"
//   - an event struct         -> "event:<Name>"
//
// The starter below just pastes the item kind in front of the name, so it emits
// "instruction:increment" instead of "global:increment" and fails that case.
fn discriminator_preimage(item_kind: &str, name: &str) -> String {
    // TODO: map each item_kind to its real Anchor namespace prefix before the name.
    format!("{item_kind}:{name}")
}
