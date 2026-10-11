//! Slot metadata for the 20-step tournament draft, shared by the pages
//! that store drafts as `(order, side, phase)` actions. The board itself
//! is `components::ui::DraftBoard`.

/// Returns (side, kind, label) for each of the 20 draft slots in official LoL order.
pub fn slot_meta(idx: usize) -> (&'static str, &'static str, &'static str) {
    match idx {
        0 => ("blue", "ban", "Ban 1"),
        1 => ("red", "ban", "Ban 1"),
        2 => ("blue", "ban", "Ban 2"),
        3 => ("red", "ban", "Ban 2"),
        4 => ("blue", "ban", "Ban 3"),
        5 => ("red", "ban", "Ban 3"),
        6 => ("blue", "pick", "Pick 1"),
        7 => ("red", "pick", "Pick 1"),
        8 => ("red", "pick", "Pick 2"),
        9 => ("blue", "pick", "Pick 2"),
        10 => ("blue", "pick", "Pick 3"),
        11 => ("red", "pick", "Pick 3"),
        12 => ("red", "ban", "Ban 4"),
        13 => ("blue", "ban", "Ban 4"),
        14 => ("red", "ban", "Ban 5"),
        15 => ("blue", "ban", "Ban 5"),
        16 => ("red", "pick", "Pick 4"),
        17 => ("blue", "pick", "Pick 4"),
        18 => ("blue", "pick", "Pick 5"),
        19 => ("red", "pick", "Pick 5"),
        _ => ("blue", "ban", "?"),
    }
}
