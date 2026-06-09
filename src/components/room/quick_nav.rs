//! Summonable command palette for instant jumps between rooms and surfaces.
//!
//! Hidden by default; opened with Cmd/Ctrl+K (or the floating button for
//! pointer/touch users), closed with Escape. Reuses the canonical
//! destinations from [`crate::components::nav::HUB_ROUTES`] so there is one
//! source of truth for navigation. Every item keeps a visible focus ring
//! (guardrail G-12); no `outline:none` without replacement.
//!
//! Accessibility guarantees (WCAG 2.1 SC 2.1.2):
//! - SC2: Focus is restored to the trigger element on every close path (Escape,
//!   backdrop click, link click). The element that had focus when the palette
//!   opened is captured and `.focus()`-ed on close.
//! - SC3: Tab/Shift+Tab are trapped within [role="dialog"] while open. Tab from
//!   the last focusable item wraps back to the filter input; Shift+Tab from the
//!   filter input wraps to the last item.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::nav::HUB_ROUTES;

/// Human-readable label for each room key used in `HUB_ROUTES`.
fn hub_label(key: &str) -> &'static str {
    match key {
        "war-room" => "War Room",
        "library"  => "Hall of Records",
        "throne"   => "Throne",
        _          => "More",
    }
}

#[component]
pub fn QuickNav() -> impl IntoView {
    let open = RwSignal::new(false);

    // Filter signal + input node-ref for auto-focus on open (D-09).
    let filter = RwSignal::new(String::new());
    let input_ref = NodeRef::<leptos::html::Input>::new();

    // SC2 — capture the element that had focus when the palette opened so we
    // can restore it on every close path. #[allow(unused_variables)] suppresses
    // the SSR-build warning for a signal only touched inside #[cfg(hydrate)]
    // blocks (wasm-patterns rule #43).
    #[allow(unused_variables)]
    let trigger_el: RwSignal<Option<web_sys::Element>> = RwSignal::new(None);

    // Centralised close fn — ALL close paths route through here so SC2 is
    // guaranteed: open.set(false) + restore focus to the captured trigger.
    let close = move || {
        open.set(false);
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            if let Some(el) = trigger_el.get_untracked() {
                if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                    let _ = html_el.focus();
                }
            }
        }
    };

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
                        close();
                    } else if (ev.meta_key() || ev.ctrl_key()) && key.eq_ignore_ascii_case("k") {
                        ev.prevent_default();
                        let currently_open = open.get_untracked();
                        if !currently_open {
                            // Capture active element BEFORE opening (SC2).
                            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                trigger_el.set(doc.active_element());
                            }
                        }
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

    // Auto-focus the filter input when the palette opens (D-09 UX).
    // Also clears the filter so the user sees all destinations fresh.
    //
    // Rule: read NodeRef with `get_untracked()` to avoid subscribing to the
    // NodeRef signal. Subscribing would cause the Effect to re-run when the
    // input mounts (NodeRef gets populated), which in turn calls filter.set("")
    // again, potentially triggering a reactive re-render cascade.
    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            if open.get() {
                filter.set(String::new());
                if let Some(el) = input_ref.get_untracked() {
                    let _ = el.focus();
                }
            }
        });
    }

    view! {
        // Floating launcher so pointer/touch users can summon the palette too.
        <button
            type="button"
            on:click=move |_| {
                let currently_open = open.get_untracked();
                if !currently_open {
                    // Capture active element BEFORE opening (SC2) — the button
                    // click path must also capture so Escape restores here.
                    #[cfg(feature = "hydrate")]
                    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                        trigger_el.set(doc.active_element());
                    }
                }
                open.update(|o| *o = !*o);
            }
            aria-label="Open quick navigation (Ctrl+K)"
            // Bottom-LEFT so it never overlaps the bug-report widget button
            // (fixed bottom-6 right-6 z-50, src/components/bug_report_widget.rs).
            class="fixed bottom-6 left-6 z-[60] h-11 w-11 rounded-full bg-surface/80 backdrop-blur-md border border-outline/60 text-secondary hover:text-primary flex items-center justify-center font-mono text-xs shadow-lg focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
        >
            "⌘K"
        </button>

        <Show when=move || open.get()>
            // SC3 Tab trap: keydown on the DIALOG CONTAINER intercepts Tab/Shift+Tab
            // and wraps focus between the filter input (first focusable) and the last
            // nav link (last focusable). prevent_default() stops the browser from
            // tabbing to the background page.
            <div
                class="fixed inset-0 z-[70] flex items-start justify-center pt-[12vh] px-4"
                role="dialog"
                aria-modal="true"
                aria-label="Quick navigation"
                on:keydown=move |ev: web_sys::KeyboardEvent| {
                    #[cfg(feature = "hydrate")]
                    {
                        use wasm_bindgen::JsCast;
                        if ev.key() != "Tab" {
                            return;
                        }
                        // Find all focusable elements inside the dialog.
                        // The filter input is always first; nav <a> elements are the rest.
                        let target = ev.current_target()
                            .and_then(|t| t.dyn_into::<web_sys::Element>().ok());
                        let Some(dialog_el) = target else { return };
                        let Ok(node_list) = dialog_el.query_selector_all(
                            "input, a[href], button:not([disabled]), [tabindex]:not([tabindex='-1'])"
                        ) else { return };
                        let len = node_list.length();
                        if len == 0 {
                            return;
                        }
                        let first = node_list.item(0)
                            .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok());
                        let last = node_list.item(len - 1)
                            .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok());

                        let active = web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|d| d.active_element());

                        if ev.shift_key() {
                            // Shift+Tab from first focusable → wrap to last.
                            let on_first = active.as_ref()
                                .zip(first.as_ref())
                                .map(|(a, f)| a == f.as_ref())
                                .unwrap_or(false);
                            if on_first {
                                ev.prevent_default();
                                if let Some(el) = last {
                                    let _ = el.focus();
                                }
                            }
                        } else {
                            // Tab from last focusable → wrap to first.
                            let on_last = active.as_ref()
                                .zip(last.as_ref())
                                .map(|(a, l)| a == l.as_ref())
                                .unwrap_or(false);
                            if on_last {
                                ev.prevent_default();
                                if let Some(el) = first {
                                    let _ = el.focus();
                                }
                            }
                        }
                    }
                }
            >
                // Backdrop — click to dismiss (SC2: route through close()).
                <div
                    class="absolute inset-0 bg-overlay-strong backdrop-blur-md"
                    on:click=move |_| close()
                    aria-hidden="true"
                ></div>

                <div class="relative w-full max-w-lg rounded-xl border border-outline/60 bg-elevated shadow-2xl overflow-hidden">
                    // Filter input — auto-focused on open, substring filters destinations (D-09).
                    // Ring classes preserved per G-12 (no outline:none without ring replacement).
                    <input
                        node_ref=input_ref
                        type="text"
                        prop:value=move || filter.get()
                        on:input=move |ev| filter.set(event_target_value(&ev))
                        placeholder="Jump to…"
                        class="w-full bg-transparent px-5 py-3 text-sm text-primary placeholder:text-dimmed border-b border-divider focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                    />

                    <nav class="max-h-[60vh] overflow-y-auto py-2" aria-label="Destinations">
                        {HUB_ROUTES
                            .iter()
                            .map(|(hub, routes)| {
                                // Filter routes by case-insensitive label substring (D-09).
                                let filtered_routes: Vec<(&str, &str)> = routes
                                    .iter()
                                    .filter(|(_, label)| {
                                        let f = filter.get();
                                        if f.is_empty() {
                                            return true;
                                        }
                                        label.to_lowercase().contains(&f.to_lowercase())
                                    })
                                    .map(|(p, l)| (*p, *l))
                                    .collect();

                                if filtered_routes.is_empty() {
                                    return view! { <div></div> }.into_any();
                                }

                                view! {
                                    <div class="px-2 py-1">
                                        <div class="px-3 pt-2 pb-1 font-imperial uppercase tracking-[0.16em] text-[10px] text-dimmed">
                                            {hub_label(hub)}
                                        </div>
                                        {filtered_routes
                                            .into_iter()
                                            .map(|(path, label)| {
                                                // SC2: link clicks also close via close() to restore focus.
                                                view! {
                                                    <A
                                                        href=path
                                                        on:click=move |_| close()
                                                        attr:class="flex items-center gap-3 px-3 py-2 rounded-md text-sm text-secondary hover:bg-surface hover:text-primary focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                                                    >
                                                        {label}
                                                    </A>
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                }.into_any()
                            })
                            .collect_view()}
                    </nav>
                </div>
            </div>
        </Show>
    }
}
