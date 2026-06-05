//! Parallax atmosphere primitives for the immersive "rooms" UI.
//!
//! The performance contract: a single `mousemove` listener, RAF-batched,
//! writes exactly two CSS custom properties (`--px`, `--py`) onto the room
//! root. Every depth plane then consumes them in pure CSS
//! (`transform: translate3d(calc(var(--px) * var(--depth) * 1px), …)`), so the
//! mouse position never enters the Leptos reactive graph and the compositor
//! does all per-frame work on the GPU.
//!
//! Accessibility: parallax is gated behind `prefers-reduced-motion` in **two**
//! places — the JS listener bails before attaching (here) and `input.css`
//! zeroes the transforms via `@media` (belt-and-suspenders, since the OS
//! setting can flip at runtime).

use leptos::prelude::*;

/// Monotonic instance counter: assigns each `AtmosphereRoot` a unique `my_id`
/// so the single-listener slot can track which instance currently owns it.
/// Only used in the hydrate target — gated to suppress the SSR dead-code
/// warning (mirrors the ACTIVE_MOUSEMOVE thread_local below).
#[cfg(feature = "hydrate")]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "hydrate")]
static NEXT_PARALLAX_ID: AtomicUsize = AtomicUsize::new(1);

/// Single active mousemove listener slot (hydrate target only — WASM is
/// single-threaded so `thread_local!` + `RefCell` is safe and avoids the
/// `Send` bound that a raw `static` would require for `Closure`).
///
/// Invariant: at most ONE `mousemove` listener is registered on `window` at
/// any time while any `AtmosphereRoot` is mounted; ZERO after the last
/// unmounts. The Closure lives here from mount to cleanup — the slot is the
/// single source of truth for both the function pointer and the owner id.
///
/// Verified trace (WR-01 fix):
///   A mounts  → slot=None   → add cb_A, slot=(A,cb_A)            [1 listener]
///   B mounts  → slot=(A,*) → remove cb_A, add cb_B, slot=(B,cb_B) [1 listener]
///   A cleans  → stored_id=B ≠ my_id=A → no-op                   [1 listener]
///   B cleans  → stored_id=B == my_id=B → remove cb_B, slot=None  [0 listeners]
#[cfg(feature = "hydrate")]
thread_local! {
    static ACTIVE_MOUSEMOVE: std::cell::RefCell<
        Option<(usize, wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MouseEvent)>)>
    > = const { std::cell::RefCell::new(None) };
}

/// DOM id of the atmosphere root — the RAF callback looks it up to write
/// `--px`/`--py`. Only one room mounts at a time, so a fixed id is safe.
const ATMOSPHERE_ROOT_ID: &str = "room-atmosphere-root";

/// Root wrapper that owns the pointer→CSS-var bridge. Drop the planes and the
/// surface content in as `children`. Mirrors the layered structure of
/// `closed_beta.rs` (fixed background + vignette + `relative z-10` content).
#[component]
pub fn AtmosphereRoot(
    /// Extra classes for the content-flow root (e.g. `min-h-screen`).
    #[prop(optional, into)]
    class: String,
    /// Growth tiers (0..=3) written as `data-{axis}-tier` on the root so the
    /// growth overlays (children) can fade in via pure CSS. Reactive: pass a
    /// signal derived from the growth resource.
    #[prop(optional, into)]
    lib_tier: Signal<u8>,
    #[prop(optional, into)] armory_tier: Signal<u8>,
    #[prop(optional, into)] banner_tier: Signal<u8>,
    // War Room axes (D-10). Optional → default signal yields 0 (Seed).
    #[prop(optional, into)] tactics_tier: Signal<u8>,
    #[prop(optional, into)] intel_tier: Signal<u8>,
    #[prop(optional, into)] command_tier: Signal<u8>,
    // Library axes (D-10).
    #[prop(optional, into)] records_tier: Signal<u8>,
    #[prop(optional, into)] tomes_tier: Signal<u8>,
    #[prop(optional, into)] analysis_tier: Signal<u8>,
    // Throne axes (D-10).
    #[prop(optional, into)] roster_tier: Signal<u8>,
    #[prop(optional, into)] progression_tier: Signal<u8>,
    #[prop(optional, into)] crown_tier: Signal<u8>,
    children: Children,
) -> impl IntoView {
    // Pointer → CSS var bridge. Hydrate-only; SSR renders a static painting.
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;

        // Assign a unique id to this AtmosphereRoot instance (WR-01 fix).
        // Used to guard cleanup: only the current ACTIVE_MOUSEMOVE owner removes
        // the listener, so a stale older root's cleanup never tears down the
        // newer root's listener.
        let my_id = NEXT_PARALLAX_ID.fetch_add(1, Ordering::Relaxed);

        // Non-reactive shared state (rule 22): latest pointer position and a
        // re-entrancy guard for the rAF. Both are per-instance (not global) —
        // the single active listener always captures its own copies of these.
        use leptos::prelude::LocalStorage;
        let latest = StoredValue::new((0.0_f64, 0.0_f64));
        let raf_pending = StoredValue::new(false);

        // A11Y-01 / D-11: the runtime prefers-reduced-motion `change` listener
        // pair — (MediaQueryList, Closure) — kept alive for teardown. The
        // closure reads state from the captured MediaQueryList (`.matches()`),
        // so no `MediaQueryListEvent` web-sys feature is required.
        let mql_listener: StoredValue<
            Option<(web_sys::MediaQueryList, Closure<dyn FnMut()>)>,
            LocalStorage,
        > = StoredValue::new_local(None);

        Effect::new(move |_| {
            let Some(win) = web_sys::window() else {
                return;
            };

            // The reduced-motion media query. We both read its initial state
            // (gate #1, below) and listen for runtime toggles (A11Y-01) off this
            // single object.
            let mql = win
                .match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten();

            // Build the mousemove → CSS-var handler. The Closure will be moved
            // into ACTIVE_MOUSEMOVE below (if motion is currently allowed) or
            // discarded if reduced-motion is active (the change-listener will
            // handle the ON→OFF transition via ACTIVE_MOUSEMOVE at that point).
            let cb =
                Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
                    // Normalise pointer to -1..=1 relative to viewport centre.
                    if let Some(w) = web_sys::window() {
                        let iw = w
                            .inner_width()
                            .ok()
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.0)
                            .max(1.0);
                        let ih = w
                            .inner_height()
                            .ok()
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.0)
                            .max(1.0);
                        latest.set_value((
                            (ev.client_x() as f64 / iw) * 2.0 - 1.0,
                            (ev.client_y() as f64 / ih) * 2.0 - 1.0,
                        ));
                    }

                    // Coalesce bursts of mousemove into one write per frame.
                    if !raf_pending.get_value() {
                        raf_pending.set_value(true);
                        let raf_cb = Closure::once(move || {
                            raf_pending.set_value(false);
                            let (x, y) = latest.get_value();
                            // Look the root up by id and write the two CSS vars.
                            // (id lookup → definite web_sys::Element; avoids the
                            // Leptos `.style(value)` builder shadowing the
                            // web_sys `.style()` getter on NodeRef output.)
                            if let Some(el) = web_sys::window()
                                .and_then(|w| w.document())
                                .and_then(|d| d.get_element_by_id(ATMOSPHERE_ROOT_ID))
                            {
                                if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                                    let style = web_sys::HtmlElement::style(&html_el);
                                    let _ = style.set_property("--px", &format!("{x:.4}"));
                                    let _ = style.set_property("--py", &format!("{y:.4}"));
                                }
                            }
                        });
                        if let Some(w) = web_sys::window() {
                            let _ = w.request_animation_frame(raf_cb.as_ref().unchecked_ref());
                        }
                        raf_cb.forget();
                    }
                });

            // A11Y-01 / D-11: attach a runtime `change` listener on the media
            // query so a mid-session Reduce Motion flip freezes (ON) or restores
            // (OFF) parallax without a reload. Attached regardless of the initial
            // state so the started-reduced → toggled-off path also restores.
            //
            // The change-listener routes all mousemove attach/detach through
            // ACTIVE_MOUSEMOVE so the single-listener invariant holds (WR-01 fix,
            // WR-03 scope: change-listener branches now use the same slot).
            if let Some(media) = mql.as_ref() {
                let media_state = media.clone();
                let change_cb = Closure::<dyn FnMut()>::new(move || {
                    let Some(w) = web_sys::window() else {
                        return;
                    };
                    if media_state.matches() {
                        // Reduce Motion turned ON: freeze the planes (--px/--py
                        // → 0) and detach the mousemove listener if this instance
                        // currently owns it. The Closure stays in the slot so the
                        // OFF branch can re-attach it.
                        if let Some(el) = w
                            .document()
                            .and_then(|d| d.get_element_by_id(ATMOSPHERE_ROOT_ID))
                        {
                            if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                                let style = web_sys::HtmlElement::style(&html_el);
                                let _ = style.set_property("--px", "0");
                                let _ = style.set_property("--py", "0");
                            }
                        }
                        ACTIVE_MOUSEMOVE.with(|slot| {
                            let owner = slot.borrow().as_ref().map(|(id, _)| *id);
                            if owner == Some(my_id) {
                                // Remove from window.
                                if let Some((_, ref stored_cb)) = *slot.borrow() {
                                    let _ = w.remove_event_listener_with_callback(
                                        "mousemove",
                                        stored_cb.as_ref().unchecked_ref(),
                                    );
                                }
                                // Leave slot occupied (Closure stays alive for the
                                // OFF branch). Mark with sentinel id=0: "stored but
                                // not on window". Counter starts at 1 so 0 is safe.
                                let mut guard = slot.borrow_mut();
                                if let Some(pair) = guard.take() {
                                    *guard = Some((0, pair.1));
                                }
                            }
                        });
                    } else {
                        // Reduce Motion turned OFF: re-attach the stored Closure
                        // via ACTIVE_MOUSEMOVE. Only re-attach if the slot is in
                        // the detached-sentinel state (owner == 0) or belongs to
                        // this instance. If another root has taken ownership since
                        // the ON event, leave it alone — it already has its own
                        // listener registered.
                        ACTIVE_MOUSEMOVE.with(|slot| {
                            let owner = slot.borrow().as_ref().map(|(id, _)| *id);
                            // owner == 0: this instance's cb is stored but detached
                            // (set by the ON branch above). Re-attach it.
                            if owner == Some(0) {
                                if let Some((_, ref stored_cb)) = *slot.borrow() {
                                    let _ = w.add_event_listener_with_callback(
                                        "mousemove",
                                        stored_cb.as_ref().unchecked_ref(),
                                    );
                                }
                                // Restore real owner id.
                                let mut guard = slot.borrow_mut();
                                if let Some(pair) = guard.take() {
                                    *guard = Some((my_id, pair.1));
                                }
                            }
                        });
                    }
                });
                let _ = media
                    .add_event_listener_with_callback("change", change_cb.as_ref().unchecked_ref());
                mql_listener.set_value(Some((media.clone(), change_cb)));
            }

            // Reduced-motion gate #1: only attach the mousemove listener now
            // when motion is currently allowed. The change-listener above
            // handles later toggles either way.
            let reduced = mql.as_ref().map(|m| m.matches()).unwrap_or(false);
            if reduced {
                // Under reduced motion: drop cb (never used). The change-listener
                // is already registered; when motion turns OFF it will find the
                // slot in an appropriate state.
                drop(cb);
                return;
            }

            // Single-listener slot (WR-01 fix): evict the previous owner's
            // listener (if any), attach this instance's new Closure, and take
            // ownership of the slot. WASM is single-threaded so no locking needed.
            ACTIVE_MOUSEMOVE.with(|slot| {
                // Remove old owner's listener from window (if any).
                if let Some((_, ref old_cb)) = *slot.borrow() {
                    let _ = win.remove_event_listener_with_callback(
                        "mousemove",
                        old_cb.as_ref().unchecked_ref(),
                    );
                }
                // Attach new listener.
                let _ = win
                    .add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref());
                // Store (my_id, cb) — moves cb into the slot (Closure is now
                // owned by ACTIVE_MOUSEMOVE, not a local variable).
                *slot.borrow_mut() = Some((my_id, cb));
            });
        });

        on_cleanup(move || {
            // Owner-guarded cleanup (WR-01 fix): only remove the global listener
            // if this instance currently owns the slot. A stale older root's
            // cleanup must not tear down the newer root's listener.
            //
            // Also handles the detached-sentinel case (owner == 0): that means
            // our cb is stored but not on window — clear the slot to free the
            // Closure, but no removeEventListener call needed.
            ACTIVE_MOUSEMOVE.with(|slot| {
                let owner = slot.borrow().as_ref().map(|(id, _)| *id);
                match owner {
                    Some(id) if id == my_id => {
                        // This instance owns the slot and the listener is live.
                        if let Some((_, ref cb)) = *slot.borrow() {
                            if let Some(w) = web_sys::window() {
                                let _ = w.remove_event_listener_with_callback(
                                    "mousemove",
                                    cb.as_ref().unchecked_ref(),
                                );
                            }
                        }
                        *slot.borrow_mut() = None;
                    }
                    Some(0) => {
                        // Detached-sentinel: our cb is stored but not on window
                        // (reduced-motion ON was active). Just free the Closure.
                        *slot.borrow_mut() = None;
                    }
                    _ => {
                        // Another instance owns the slot — leave it untouched.
                    }
                }
            });
            // Tear down the runtime reduced-motion change-listener too, so it
            // does not accumulate across room navigations (T-26-03).
            mql_listener.update_value(|slot| {
                if let Some((mql, cb)) = slot.take() {
                    let _ = mql
                        .remove_event_listener_with_callback("change", cb.as_ref().unchecked_ref());
                    // cb dropped here, freeing the JS closure.
                }
            });
        });
    }

    view! {
        <div
            id=ATMOSPHERE_ROOT_ID
            class=format!("room-atmosphere bg-base relative overflow-hidden {class}")
            style="--px:0; --py:0;"
            data-lib-tier=move || lib_tier.get().to_string()
            data-armory-tier=move || armory_tier.get().to_string()
            data-banner-tier=move || banner_tier.get().to_string()
            data-tactics-tier=move || tactics_tier.get().to_string()
            data-intel-tier=move || intel_tier.get().to_string()
            data-command-tier=move || command_tier.get().to_string()
            data-records-tier=move || records_tier.get().to_string()
            data-tomes-tier=move || tomes_tier.get().to_string()
            data-analysis-tier=move || analysis_tier.get().to_string()
            data-roster-tier=move || roster_tier.get().to_string()
            data-progression-tier=move || progression_tier.get().to_string()
            data-crown-tier=move || crown_tier.get().to_string()
        >
            {children()}
        </div>
    }
}

/// A single fixed, full-bleed parallax plane. `depth` is the pixel offset per
/// unit of pointer travel (bigger = closer/more movement); `z` is the stacking
/// order (more negative = further back). `src` should already be the active
/// region's asset — only one region's planes are ever rendered (per the
/// 400 KB budget; CSS-hiding the other region still triggers a fetch).
#[component]
pub fn Plane(#[prop(into)] src: String, depth: i32, z: i32) -> impl IntoView {
    view! {
        <img
            src=src
            alt=""
            aria-hidden="true"
            loading="lazy"
            class="room-plane w-full h-full object-cover"
            style=format!("--depth:{depth}; z-index:{z};")
        />
    }
}

/// A growth-driven atmosphere overlay for one axis (e.g. `"lib"`, `"armory"`,
/// `"banner"`). Always present in the DOM; its visibility/opacity is driven
/// purely by the `data-{axis}-tier` attribute on the [`AtmosphereRoot`] via
/// CSS in `input.css` — no reactive structural branching, so hydration stays
/// symmetric (the Phase 18.1/18.2 lesson).
#[component]
pub fn GrowthOverlay(
    /// Short axis key matching the CSS rules. One of the twelve room axes:
    /// Solo `lib` | `armory` | `banner`; War `tactics` | `intel` | `command`;
    /// Library `records` | `tomes` | `analysis`; Throne `roster` |
    /// `progression` | `crown`.
    #[prop(into)]
    axis: String,
    #[prop(into)] src: String,
    z: i32,
) -> impl IntoView {
    view! {
        <img
            src=src
            alt=""
            aria-hidden="true"
            loading="lazy"
            class=format!("room-overlay room-overlay--{axis} w-full h-full object-cover")
            style=format!("z-index:{z};")
        />
    }
}
