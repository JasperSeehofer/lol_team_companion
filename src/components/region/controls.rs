//! Shim: `ModeToggle` keeps its old props and renders the batch 2
//! `SegmentedControl` (radiogroup with roving tabindex). M3/M5 move the
//! callers to `SegmentedControl` directly and delete this file.

use leptos::prelude::*;

use crate::components::ui::{Segment, SegmentSize, SegmentedControl};

/// Segmented mode switch. `options` are `(value, label)` pairs.
///
/// A11y: a `SegmentedControl` (`role="radiogroup"` named by `label`,
/// `role="radio"` segments, arrow keys move and select).
#[component]
pub fn ModeToggle(
    /// Current mode value.
    current: ReadSignal<String>,
    #[prop(into)] options: Vec<(String, String)>,
    on_select: Callback<String>,
    #[prop(optional, into)] label: Option<String>,
) -> impl IntoView {
    let label = label.unwrap_or_else(|| "View".to_string());
    let value = RwSignal::new(current.get_untracked());
    Effect::new(move |_| {
        let c = current.get();
        if value.get_untracked() != c {
            value.set(c);
        }
    });
    let segments = options
        .into_iter()
        .map(|(v, l)| Segment::new(v, l))
        .collect::<Vec<_>>();
    view! {
        <SegmentedControl
            id="page-mode"
            label=label
            options=segments
            value=value
            size=SegmentSize::Md
            on_change=on_select
        />
    }
}
