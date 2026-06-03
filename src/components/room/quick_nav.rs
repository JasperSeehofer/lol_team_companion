//! Summonable command palette for instant jumps between rooms and surfaces.
//!
//! Hidden by default; opened with Cmd/Ctrl+K (or the floating button for
//! pointer/touch users), closed with Escape. Reuses the canonical
//! destinations from [`crate::components::nav::HUB_ROUTES`] so there is one
//! source of truth for navigation. Every item keeps a visible focus ring
//! (guardrail G-12); no `outline:none` without replacement.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::nav::HUB_ROUTES;

/// Human-readable label for each hub key used in `HUB_ROUTES`.
fn hub_label(key: &str) -> &'static str {
    match key {
        "strategy" => "War Room",
        "history" => "Hall of Records",
        "profile" => "Profile & Journey",
        _ => "More",
    }
}

#[component]
pub fn QuickNav() -> impl IntoView {
    let open = RwSignal::new(false);

    // Global keyboard shortcuts: Cmd/Ctrl+K toggles, Escape closes.
    // Same listener idiom as nav.rs (Escape handler); hydrate-only, guarded,
    // no panicking unwraps (wasm-patterns rule 35).
    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            use wasm_bindgen::closure::Closure;
            use wasm_bindgen::JsCast;
            let cb = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |ev: web_sys::KeyboardEvent| {
                    let key = ev.key();
                    if key == "Escape" {
                        open.set(false);
                    } else if (ev.meta_key() || ev.ctrl_key()) && key.eq_ignore_ascii_case("k") {
                        ev.prevent_default();
                        open.update(|o| *o = !*o);
                    }
                },
            );
            if let Some(window) = web_sys::window() {
                let _ =
                    window.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
            }
            // Mounted once for the app lifetime (sibling of Nav), so a forget()
            // here is acceptable — it never stacks across navigations.
            cb.forget();
        });
    }

    view! {
        // Floating launcher so pointer/touch users can summon the palette too.
        <button
            type="button"
            on:click=move |_| open.update(|o| *o = !*o)
            aria-label="Open quick navigation (Ctrl+K)"
            // Bottom-LEFT so it never overlaps the bug-report widget button
            // (fixed bottom-6 right-6 z-50, src/components/bug_report_widget.rs).
            class="fixed bottom-6 left-6 z-[60] h-11 w-11 rounded-full bg-surface/80 backdrop-blur-md border border-outline/60 text-secondary hover:text-primary flex items-center justify-center font-mono text-xs shadow-lg focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
        >
            "⌘K"
        </button>

        <Show when=move || open.get()>
            <div
                class="fixed inset-0 z-[70] flex items-start justify-center pt-[12vh] px-4"
                role="dialog"
                aria-modal="true"
                aria-label="Quick navigation"
            >
                // Backdrop — click to dismiss.
                <div
                    class="absolute inset-0 bg-overlay-strong backdrop-blur-md"
                    on:click=move |_| open.set(false)
                    aria-hidden="true"
                ></div>

                <div class="relative w-full max-w-lg rounded-xl border border-outline/60 bg-elevated shadow-2xl overflow-hidden">
                    <div class="px-5 py-3 border-b border-divider flex items-center justify-between">
                        <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">
                            "Travel"
                        </span>
                        <span class="font-mono text-[10px] text-dimmed">"Esc to close"</span>
                    </div>

                    <nav class="max-h-[60vh] overflow-y-auto py-2" aria-label="Destinations">
                        {HUB_ROUTES
                            .iter()
                            .map(|(hub, routes)| {
                                view! {
                                    <div class="px-2 py-1">
                                        <div class="px-3 pt-2 pb-1 font-imperial uppercase tracking-[0.16em] text-[10px] text-dimmed">
                                            {hub_label(hub)}
                                        </div>
                                        {routes
                                            .iter()
                                            .map(|(path, label)| {
                                                view! {
                                                    <A
                                                        href=*path
                                                        on:click=move |_| open.set(false)
                                                        attr:class="flex items-center gap-3 px-3 py-2 rounded-md text-sm text-secondary hover:bg-surface hover:text-primary focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                                                    >
                                                        {*label}
                                                    </A>
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                }
                            })
                            .collect_view()}
                    </nav>
                </div>
            </div>
        </Show>
    }
}
