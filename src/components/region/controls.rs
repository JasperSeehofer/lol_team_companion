//! Shim: `ModeToggle`, a small segmented control for page modes.
//! Batch 2's `SegmentedControl` replaces it.

use leptos::prelude::*;

/// Segmented mode switch. `options` are `(value, label)` pairs.
///
/// Buttons expose their state with `aria-pressed`; the group is labelled.
#[component]
pub fn ModeToggle(
    /// Current mode value.
    current: ReadSignal<String>,
    #[prop(into)] options: Vec<(String, String)>,
    on_select: Callback<String>,
    #[prop(optional, into)] label: Option<String>,
) -> impl IntoView {
    let label = label.unwrap_or_else(|| "View".to_string());
    view! {
        <div role="group" aria-label=label class="inline-flex items-stretch gap-0.5 rounded-ctl border border-outline p-0.5">
            {options
                .into_iter()
                .map(|(value, text)| {
                    let v_class = value.clone();
                    let v_aria = value.clone();
                    let v_click = value.clone();
                    view! {
                        <button
                            type="button"
                            class=move || {
                                if current.get() == v_class {
                                    "h-7 cursor-pointer rounded-[2px] bg-ink-3 px-3 text-label font-semibold text-primary"
                                } else {
                                    "h-7 cursor-pointer rounded-[2px] px-3 text-label text-muted transition-colors hover:text-primary"
                                }
                            }
                            aria-pressed=move || (current.get() == v_aria).to_string()
                            on:click=move |_| on_select.run(v_click.clone())
                        >
                            {text}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}
