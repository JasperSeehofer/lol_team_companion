//! Phase 17 plan 17-06 — Bug-report widget (visual stub).
//!
//! Per UI-SPEC §"Bug-Report Widget Placement" lines 580-606. This
//! phase ships visual anatomy only: floating button, modal stub,
//! bug/wishlist toggle, free-text textarea, submit/cancel buttons.
//! Phase 18 wires the actual element-picker, DB write, and
//! sanitisation.
//!
//! Mounted in `app.rs` — the widget gates its own visibility on the
//! authenticated state via `get_current_user()`, and additionally on
//! the current pathname (hidden on `/`, `/auth/*`, `/closed-beta`,
//! `/legal/*`). This keeps the auth-gating logic local to the widget
//! rather than threading conditional mounting through the router.

use leptos::ev;
use leptos::prelude::*;

use crate::components::icon::Icon;
use crate::pages::profile::get_current_user;

/// Pathname prefixes on which the widget should NOT render. UI-SPEC
/// line 590 — public/auth/legal pages must not show the widget.
const HIDDEN_PREFIXES: &[&str] = &["/auth", "/closed-beta", "/legal"];

#[component]
pub fn BugReportWidget() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());
    let modal_open = RwSignal::new(false);
    let report_kind = RwSignal::new("bug".to_string());
    let report_text = RwSignal::new(String::new());

    // Phase 18 will populate this with the user-selected element label
    // (via element-picker mode). Phase 17 stub: a fixed placeholder
    // string so the modal anatomy is testable.
    let element_label = RwSignal::new("(no element selected)".to_string());

    // Pathname guard — re-evaluated on each Effect tick. We can't
    // use `use_location()` here without pulling in extra wiring; the
    // simpler approach is to read window.location().pathname() inside
    // a Show predicate during hydration. SSR side: the widget is
    // wrapped behind `auth.user.is_some()` so it never SSRs for
    // anonymous visitors.
    let pathname = RwSignal::new(String::new());
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        if let Some(window) = web_sys::window() {
            if let Ok(path) = window.location().pathname() {
                pathname.set(path);
            }
        }
    });

    let on_pathname_excluded = move || {
        let p = pathname.get();
        HIDDEN_PREFIXES.iter().any(|prefix| p.starts_with(prefix))
    };

    let is_authed = move || matches!(user.get(), Some(Ok(Some(_))));

    let widget_visible = move || is_authed() && !on_pathname_excluded();

    view! {
        <Show when=widget_visible fallback=|| ()>
            <BugReportWidgetInner
                modal_open=modal_open
                report_kind=report_kind
                report_text=report_text
                element_label=element_label
            />
        </Show>
    }
}

#[component]
fn BugReportWidgetInner(
    modal_open: RwSignal<bool>,
    report_kind: RwSignal<String>,
    report_text: RwSignal<String>,
    element_label: RwSignal<String>,
) -> impl IntoView {
    // Cloned signal access for separate closures (rule 18: clones
    // before multiple closures).
    let report_kind_cls_bug = report_kind;
    let report_kind_cls_wish = report_kind;
    let report_kind_set_bug = report_kind;
    let report_kind_set_wish = report_kind;
    // Wasm rule 43: these are only read inside #[cfg(feature = "hydrate")]
    // (the Submit click handler logs to console.log). SSR doesn't need
    // them, so suppress the unused-variable warning on the SSR build.
    #[allow(unused_variables)]
    let report_kind_submit = report_kind;
    let report_text_set = report_text;
    let report_text_get = report_text;
    #[allow(unused_variables)]
    let report_text_submit = report_text;

    view! {
        // Floating button (UI-SPEC line 583)
        <button
            type="button"
            class="fixed bottom-6 right-6 z-50 w-11 h-11 rounded-full bg-elevated border border-divider shadow-lg hover:bg-surface hover:border-outline transition-all flex items-center justify-center focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
            title="Report a bug or wishlist item"
            aria-label="Report a bug or wishlist item"
            on:click=move |_| modal_open.set(true)
        >
            <Icon name="feather" size=18 class="text-muted" />
        </button>

        // Modal stub (Phase 18 wires submit-to-DB).
        <Show when=move || modal_open.get() fallback=|| ()>
            <div
                class="fixed inset-0 z-50 flex items-center justify-center p-6"
                style="background-color: var(--color-overlay-strong);"
                role="dialog"
                aria-modal="true"
                aria-label="Report a bug or wishlist item"
                on:click=move |_| modal_open.set(false)
            >
                // Inner card — stop propagation so clicking inside
                // doesn't close the modal.
                <div
                    class="bg-surface border border-divider rounded-xl p-6 shadow-xl max-w-md w-full"
                    on:click=|ev: ev::MouseEvent| ev.stop_propagation()
                >
                    <h2 class="font-display italic text-xl text-primary mb-4">
                        "Report"
                    </h2>

                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted mb-1">
                        "Element"
                    </div>
                    <div class="bg-elevated rounded-lg p-2 text-sm text-secondary mb-4">
                        {move || element_label.get()}
                    </div>

                    // Bug / Wishlist toggle (UI-SPEC line 600)
                    <div class="flex gap-2 mb-4">
                        <button
                            type="button"
                            class=move || if report_kind_cls_bug.get() == "bug" {
                                "bg-accent text-accent-contrast px-4 py-2 rounded-lg text-sm font-semibold focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            } else {
                                "text-muted hover:text-secondary px-4 py-2 rounded-lg text-sm border border-divider focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            }
                            on:click=move |_| report_kind_set_bug.set("bug".to_string())
                        >
                            "Bug"
                        </button>
                        <button
                            type="button"
                            class=move || if report_kind_cls_wish.get() == "wishlist" {
                                "bg-accent text-accent-contrast px-4 py-2 rounded-lg text-sm font-semibold focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            } else {
                                "text-muted hover:text-secondary px-4 py-2 rounded-lg text-sm border border-divider focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            }
                            on:click=move |_| report_kind_set_wish.set("wishlist".to_string())
                        >
                            "Wishlist"
                        </button>
                    </div>

                    <textarea
                        class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-3 text-primary text-sm resize-none focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                        rows="4"
                        placeholder="What went wrong, or what would you like?"
                        prop:value=move || report_text_get.get()
                        on:input=move |ev| report_text_set.set(event_target_value(&ev))
                    />

                    <div class="flex gap-3 justify-end mt-4">
                        <button
                            type="button"
                            class="text-muted hover:text-secondary text-sm px-3 py-2 rounded-lg focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            on:click=move |_| modal_open.set(false)
                        >
                            "Cancel"
                        </button>
                        <button
                            type="button"
                            class="bg-accent text-accent-contrast font-semibold rounded-lg px-4 py-2 hover:bg-accent-hover focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                            on:click=move |_| {
                                // Phase 18 wires the submit-to-DB
                                // behaviour. Phase 17: log to console
                                // so the visual stub is observable.
                                #[cfg(feature = "hydrate")]
                                {
                                    let kind = report_kind_submit.get_untracked();
                                    let text = report_text_submit.get_untracked();
                                    let msg = format!("[Phase 18 stub] {}: {}", kind, text);
                                    web_sys::console::log_1(&msg.into());
                                }
                                modal_open.set(false);
                            }
                        >
                            "Submit"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
