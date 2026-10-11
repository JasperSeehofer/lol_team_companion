//! Keyboard helpers shared by the interactive primitives (Menu,
//! SegmentedControl, CommandBar, TimelineScrub). Pure functions, so the
//! keyboard contracts are unit-tested without a browser.

/// Index a list moves to for a key, or `None` for other keys.
/// `vertical` lists use Up/Down, horizontal ones Left/Right; both wrap.
/// Home and End jump to the ends. `current = None` means nothing is
/// active yet: Down/Right go to the first item, Up/Left to the last.
pub fn roving_index(
    key: &str,
    current: Option<usize>,
    len: usize,
    vertical: bool,
) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let (next_key, prev_key) = if vertical {
        ("ArrowDown", "ArrowUp")
    } else {
        ("ArrowRight", "ArrowLeft")
    };
    match key {
        k if k == next_key => Some(current.map_or(0, |c| (c + 1) % len)),
        k if k == prev_key => Some(current.map_or(len - 1, |c| (c + len - 1) % len)),
        "Home" => Some(0),
        "End" => Some(len - 1),
        _ => None,
    }
}

/// Typeahead: the next label after `current` (wrapping) that starts with
/// `ch`, ignoring case. `None` when the key is not a single character or
/// nothing matches.
pub fn typeahead(labels: &[String], current: Option<usize>, key: &str) -> Option<usize> {
    let mut chars = key.chars();
    let ch = chars.next()?;
    if chars.next().is_some() || ch.is_whitespace() {
        return None;
    }
    let ch = ch.to_lowercase().to_string();
    let len = labels.len();
    let start = current.map_or(0, |c| c + 1);
    (0..len)
        .map(|i| (start + i) % len)
        .find(|&i| labels[i].to_lowercase().starts_with(&ch))
}

/// Focus the element with this DOM id (browser only; no-op on the server).
pub fn focus_id(id: &str) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        if let Some(el) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(id))
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = el.focus();
        }
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = id;
}

/// Focus the element with this DOM id once the DOM has caught up with a
/// signal change (the element may only render after it): a short timer,
/// browser only.
pub fn focus_id_later(id: &str) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        let id = id.to_string();
        let cb = Closure::once(move || focus_id(&id));
        if let Some(win) = web_sys::window() {
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                30,
            );
        }
        cb.forget();
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = id;
}

/// Click the element with this DOM id (browser only).
pub fn click_id(id: &str) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        if let Some(el) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(id))
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        {
            el.click();
        }
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = id;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_lists_wrap_and_jump() {
        assert_eq!(roving_index("ArrowDown", None, 3, true), Some(0));
        assert_eq!(roving_index("ArrowUp", None, 3, true), Some(2));
        assert_eq!(roving_index("ArrowDown", Some(2), 3, true), Some(0));
        assert_eq!(roving_index("ArrowUp", Some(0), 3, true), Some(2));
        assert_eq!(roving_index("Home", Some(1), 3, true), Some(0));
        assert_eq!(roving_index("End", Some(1), 3, true), Some(2));
        assert_eq!(roving_index("ArrowRight", Some(1), 3, true), None);
        assert_eq!(roving_index("ArrowDown", None, 0, true), None);
    }

    #[test]
    fn horizontal_lists_use_left_and_right() {
        assert_eq!(roving_index("ArrowRight", Some(0), 2, false), Some(1));
        assert_eq!(roving_index("ArrowLeft", Some(0), 2, false), Some(1));
        assert_eq!(roving_index("ArrowDown", Some(0), 2, false), None);
    }

    #[test]
    fn typeahead_finds_the_next_match_and_wraps() {
        let labels: Vec<String> = ["Profile", "Pool", "Invites", "Sign out"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(typeahead(&labels, None, "p"), Some(0));
        assert_eq!(typeahead(&labels, Some(0), "P"), Some(1));
        assert_eq!(typeahead(&labels, Some(1), "p"), Some(0));
        assert_eq!(typeahead(&labels, None, "s"), Some(3));
        assert_eq!(typeahead(&labels, None, "x"), None);
        assert_eq!(typeahead(&labels, None, "Enter"), None);
        assert_eq!(typeahead(&labels, None, " "), None);
    }
}
