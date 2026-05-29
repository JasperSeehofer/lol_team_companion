// Bug-report widget — Phase 17 visual anatomy + Phase 19 wired flow.
//
// Phase 17 (plan 17-06) shipped the floating button, modal anatomy, the
// bug/wishlist toggle, the textarea, and the auth+pathname visibility
// gate. Phase 19 (plan 19-02) wires:
//   - WidgetState machine (Idle, Selecting, Editing)
//   - Global click-capture listener that resolves the clicked element
//     via Element::closest with the data-feedback-label attribute
//   - Escape-key cancel listener
//   - Real submit_bug_report server-fn call (replaces the Phase 17
//     console.log stub) wired to db::create_bug_report from plan 19-01
//   - Toast dispatch via the existing ToastContext on success
//   - Symmetric listener teardown on every exit path (Esc, success,
//     Cancel, click-outside, click-completion)
//
// The widget is mounted in app.rs. The visibility gate is local to
// the component (auth + pathname), not threaded through the router.
//
// Dark-pattern audit (G-10): no exclamation marks, no emoji, no NPS
// prompt, no star widget, no confirmshaming on Cancel. The single
// toast string is "Thanks. Your report is in." (period, neutral).

use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::icon::Icon;
use crate::components::ui::{ToastContext, ToastKind};
use crate::pages::profile::get_current_user;

/// Pathname prefixes on which the widget should NOT render. UI-SPEC
/// line 590 — public/auth/legal pages must not show the widget.
const HIDDEN_PREFIXES: &[&str] = &["/auth", "/closed-beta", "/legal"];

/// Three-mode widget state.
///
/// - `Idle`        — only the floating button is visible
/// - `Selecting`   — global click-capture listener is attached; user
///                   is targeting a tagged element
/// - `Editing`     — modal is open with the captured element label
///
/// `#[allow(dead_code)]` because `Selecting` and `Editing` are only
/// constructed inside `#[cfg(feature = "hydrate")]` blocks (the click
/// closure transitions Selecting → Editing). SSR sees only `Idle`.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WidgetState {
    Idle,
    Selecting,
    Editing,
}

// ---------------------------------------------------------------------------
// Server functions (Phase 19 plan 02)
//
// Per leptos-patterns rule 34 these MUST be defined before the calling
// component, and per rule 9 every SSR-only `use` statement lives INSIDE
// the function body.
// ---------------------------------------------------------------------------

/// Persist a bug report. Defense-in-depth: the DB layer already trims +
/// rejects empty descriptions and the schema `ASSERT` clause rejects bad
/// categories; we duplicate both checks here so the server-fn boundary
/// fails fast and never reaches the DB with bad input.
///
/// T-19-01 mitigation: category whitelist.
/// T-19-02 mitigation: description trim + length cap (4000 chars).
#[server]
pub async fn submit_bug_report(
    page_url: String,
    element_label: String,
    description: String,
    category: String,
    viewport_w: Option<i32>,
    viewport_h: Option<i32>,
) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    // T-19-02 server-fn guard (description hygiene).
    let description = description.trim().to_string();
    if description.is_empty() {
        return Err(ServerFnError::new("Description is required"));
    }
    if description.len() > 4000 {
        return Err(ServerFnError::new("Description exceeds 4000 characters"));
    }
    // T-19-01 server-fn guard (category whitelist).
    if category != "bug" && category != "wishlist" {
        return Err(ServerFnError::new("Invalid category"));
    }

    db::create_bug_report(
        &surreal,
        &user.id,
        page_url,
        element_label,
        description,
        category,
        viewport_w,
        viewport_h,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

/// List bug reports. v1 returns Forbidden for ALL callers — no admin role
/// field exists on the user model yet. Plan 19-03 (inbox export) writes to
/// disk via `db::list_open_bug_reports` directly, bypassing this gate.
// TODO Phase 22: replace with admin gate once role field exists on user
#[server]
pub async fn list_bug_reports(
    _status: Option<String>,
) -> Result<Vec<crate::models::bug_report::BugReport>, ServerFnError> {
    // TODO Phase 22: replace with admin gate once role field exists on user
    Err(ServerFnError::new("Forbidden"))
}

#[component]
pub fn BugReportWidget() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());
    let widget_state = RwSignal::new(WidgetState::Idle);
    let report_kind = RwSignal::new("bug".to_string());
    let report_text = RwSignal::new(String::new());
    let submit_error: RwSignal<Option<String>> = RwSignal::new(None);

    // Phase 19 wires this from the click-capture closure — pre-fill is
    // "(no element selected)" until the user clicks a tagged element.
    let element_label = RwSignal::new("(no element selected)".to_string());

    // Pathname guard — re-evaluated on each Effect tick. SSR side: the
    // widget is wrapped behind `auth.user.is_some()` so it never SSRs
    // for anonymous visitors.
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
        <Suspense fallback=|| ()>
            <Show when=widget_visible fallback=|| ()>
                <BugReportWidgetInner
                    widget_state=widget_state
                    report_kind=report_kind
                    report_text=report_text
                    element_label=element_label
                    submit_error=submit_error
                />
            </Show>
        </Suspense>
    }
}

#[component]
fn BugReportWidgetInner(
    widget_state: RwSignal<WidgetState>,
    report_kind: RwSignal<String>,
    report_text: RwSignal<String>,
    element_label: RwSignal<String>,
    submit_error: RwSignal<Option<String>>,
) -> impl IntoView {
    // Cloned signal access for separate closures (rule 18: clones
    // before multiple closures). `RwSignal` is Copy, so these are just
    // aliases for readability rather than real clones.
    let report_kind_cls_bug = report_kind;
    let report_kind_cls_wish = report_kind;
    let report_kind_set_bug = report_kind;
    let report_kind_set_wish = report_kind;
    let report_text_set = report_text;
    let report_text_get = report_text;

    // Closure carriers for the global listeners. `Closure` is `!Send`,
    // so the default `SyncStorage` parameter on `StoredValue<T>` fails
    // the `Send + Sync` bound. Use `StoredValue::new_local()` (leptos
    // 0.2.13 reactive_graph API) which returns
    // `StoredValue<T, LocalStorage>` and works with `!Send` types — the
    // canonical carrier for `wasm_bindgen::closure::Closure` (leptos
    // rule 22 + non-Send extension).
    //
    // Wasm rule 43: these are only read inside #[cfg(feature = "hydrate")]
    // blocks; suppress the SSR-build unused-variable warnings on the
    // `let` bindings.
    #[allow(unused_variables)]
    #[cfg(feature = "hydrate")]
    let click_capture_handle: StoredValue<
        Option<wasm_bindgen::closure::Closure<dyn Fn(web_sys::MouseEvent)>>,
        leptos::prelude::LocalStorage,
    > = StoredValue::new_local(None);
    #[allow(unused_variables)]
    #[cfg(feature = "hydrate")]
    let esc_handle: StoredValue<
        Option<wasm_bindgen::closure::Closure<dyn Fn(web_sys::KeyboardEvent)>>,
        leptos::prelude::LocalStorage,
    > = StoredValue::new_local(None);

    // --- exit_select_mode (declared first so start_select_mode can call it) ---
    //
    // Removes both global listeners, clears the body cursor, and removes
    // the `data-feedback-selecting` attribute from <html>. Runs on every
    // exit path (Esc, click-completion, Submit success, Cancel,
    // click-outside).
    let exit_select_mode = move || {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            if let Some(win) = web_sys::window() {
                // Pull the click-capture closure out of the StoredValue
                // slot and detach it. Closure's Drop frees the JS shim
                // when the inner `cb` falls out of scope at end of the
                // `update_value` block.
                click_capture_handle.update_value(|slot| {
                    if let Some(cb) = slot.take() {
                        // Must match the add-side `capture: true` flag —
                        // removeEventListener only detaches when capture
                        // matches. Without this the listener stays
                        // attached and subsequent clicks reopen the
                        // modal even after Esc-cancel.
                        let _ = win.remove_event_listener_with_callback_and_bool(
                            "click",
                            cb.as_ref().unchecked_ref(),
                            true,
                        );
                        cb.forget();
                    }
                });
                esc_handle.update_value(|slot| {
                    if let Some(cb) = slot.take() {
                        let _ = win.remove_event_listener_with_callback(
                            "keydown",
                            cb.as_ref().unchecked_ref(),
                        );
                        cb.forget();
                    }
                });
                if let Some(doc) = win.document() {
                    if let Some(body) = doc.body() {
                        let _ = body.style().remove_property("cursor");
                    }
                    if let Some(el) = doc.document_element() {
                        let _ = el.remove_attribute("data-feedback-selecting");
                    }
                }
            }
        }
    };

    // --- start_select_mode ---
    //
    // Sets `widget_state` to Selecting, paints the body cursor, sets
    // the `data-feedback-selecting` attribute on <html> (driving the
    // CSS outline rule in input.css), and attaches a CAPTURE-phase
    // click listener plus a keydown listener for Esc-cancel.
    let start_select_mode = move || {
        widget_state.set(WidgetState::Selecting);
        submit_error.set(None);

        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::closure::Closure;
            use wasm_bindgen::JsCast;

            let Some(win) = web_sys::window() else { return; };
            let Some(doc) = win.document() else { return; };

            if let Some(body) = doc.body() {
                let _ = body.style().set_property("cursor", "crosshair");
            }
            if let Some(el) = doc.document_element() {
                let _ = el.set_attribute("data-feedback-selecting", "");
            }

            // Click-capture closure. We attach in CAPTURE phase so we
            // intercept before any bubble-phase `on:click` handlers
            // attached to tagged elements.
            let click_cb = Closure::<dyn Fn(web_sys::MouseEvent)>::new(
                move |ev: web_sys::MouseEvent| {
                    let target = match ev.target() {
                        Some(t) => t,
                        None => return,
                    };
                    let el = match target.dyn_into::<web_sys::Element>() {
                        Ok(e) => e,
                        Err(_) => return,
                    };
                    // `Element::closest` returns `Result<Option<Element>, JsValue>`
                    // — two unwrap levels. Forgiving UX: if `Ok(None)`
                    // (user clicked an UNTAGGED area), just return so
                    // select-mode stays active and the user can try
                    // again. Only commit on a tagged hit.
                    let Ok(Some(tagged)) = el.closest("[data-feedback-label]") else {
                        return;
                    };
                    // Tagged hit — swallow the event so the underlying
                    // app does not also receive the click.
                    ev.prevent_default();
                    ev.stop_propagation();
                    ev.stop_immediate_propagation();

                    let label = tagged
                        .get_attribute("data-feedback-label")
                        .unwrap_or_else(|| "(unlabeled)".to_string());
                    element_label.set(label);
                    widget_state.set(WidgetState::Editing);
                    exit_select_mode();
                },
            );

            let opts = web_sys::AddEventListenerOptions::new();
            opts.set_capture(true);
            let _ = win.add_event_listener_with_callback_and_add_event_listener_options(
                "click",
                click_cb.as_ref().unchecked_ref(),
                &opts,
            );
            click_capture_handle.set_value(Some(click_cb));

            // Esc-cancel closure. Bubble-phase is fine for keydown.
            let esc_cb = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |ev: web_sys::KeyboardEvent| {
                    if ev.key() == "Escape" {
                        widget_state.set(WidgetState::Idle);
                        exit_select_mode();
                    }
                },
            );
            let _ = win.add_event_listener_with_callback(
                "keydown",
                esc_cb.as_ref().unchecked_ref(),
            );
            esc_handle.set_value(Some(esc_cb));
        }
    };

    // Cancel-flow shared by the Cancel button, the click-outside
    // overlay handler, and any other "abort editing" path. Symmetric
    // teardown: clear listeners, clear error, reset signals.
    let cancel_editing = move || {
        exit_select_mode();
        submit_error.set(None);
        widget_state.set(WidgetState::Idle);
        report_text.set(String::new());
        element_label.set("(no element selected)".to_string());
    };

    view! {
        // Floating button (UI-SPEC line 583). Single click enters
        // select-mode directly — no intermediate menu, per D-08 (no
        // multi-step nags).
        <button
            type="button"
            class="fixed bottom-6 right-6 z-50 w-11 h-11 rounded-full bg-elevated border border-divider shadow-lg hover:bg-surface hover:border-outline transition-all flex items-center justify-center focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
            title="Report a bug or wishlist item"
            aria-label="Report a bug or wishlist item"
            on:click=move |_| start_select_mode()
        >
            <Icon name="feather" size=18 class="text-muted" />
        </button>

        // Editing modal — only rendered when WidgetState::Editing.
        <Show when=move || matches!(widget_state.get(), WidgetState::Editing) fallback=|| ()>
            <div
                class="fixed inset-0 z-50 flex items-center justify-center p-6"
                style="background-color: var(--color-overlay-strong);"
                role="dialog"
                aria-modal="true"
                aria-label="Report a bug or wishlist item"
                on:click=move |_| cancel_editing()
            >
                // Inner card — stop propagation so clicking inside
                // does not close the modal.
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

                    // Inline submit error (only rendered on server-fn failure).
                    // Uses text-red-400 per CLAUDE.md "Exceptions to semantic
                    // tokens" — error literals are explicitly allowed.
                    <Show when=move || submit_error.get().is_some() fallback=|| ()>
                        <p class="text-sm text-red-400 mt-2">
                            {move || submit_error.get().unwrap_or_default()}
                        </p>
                    </Show>

                    <div class="flex gap-3 justify-end mt-4">
                        <button
                            type="button"
                            class="text-muted hover:text-secondary text-sm px-3 py-2 rounded-lg focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            on:click=move |_| cancel_editing()
                        >
                            "Cancel"
                        </button>
                        <button
                            type="button"
                            class="bg-accent text-accent-contrast font-semibold rounded-lg px-4 py-2 hover:bg-accent-hover focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                            on:click=move |_| {
                                submit_error.set(None);

                                // Read viewport + pathname before
                                // spawning the async task so we capture
                                // values at click-time, not at
                                // settle-time.
                                #[cfg(feature = "hydrate")]
                                let (page_url, vw, vh) = web_sys::window()
                                    .map(|w| {
                                        let url = w
                                            .location()
                                            .pathname()
                                            .ok()
                                            .unwrap_or_default();
                                        let vw = w
                                            .inner_width()
                                            .ok()
                                            .and_then(|v| v.as_f64())
                                            .map(|f| f as i32);
                                        let vh = w
                                            .inner_height()
                                            .ok()
                                            .and_then(|v| v.as_f64())
                                            .map(|f| f as i32);
                                        (url, vw, vh)
                                    })
                                    .unwrap_or_default();
                                #[cfg(not(feature = "hydrate"))]
                                let (page_url, vw, vh): (String, Option<i32>, Option<i32>) =
                                    (String::new(), None, None);

                                let kind = report_kind.get_untracked();
                                let text = report_text.get_untracked();
                                let label = element_label.get_untracked();

                                // Capture the toast context outside the
                                // async block so the `Copy` `ToastContext`
                                // value flows into the move closure.
                                let toast = use_context::<ToastContext>();

                                spawn_local(async move {
                                    match submit_bug_report(
                                        page_url, label, text, kind, vw, vh,
                                    )
                                    .await
                                    {
                                        Ok(()) => {
                                            // Shadow `toast: Option<ToastContext>`
                                            // with the unwrapped value so the
                                            // canonical dispatch shape
                                            // `toast.show.run((kind, msg))`
                                            // appears literally (matches plan
                                            // acceptance grep + src/pages/
                                            // action_items.rs:220).
                                            if let Some(toast) = toast {
                                                toast.show.run((ToastKind::Success, "Thanks. Your report is in.".into()));
                                            }
                                            widget_state.set(WidgetState::Idle);
                                            report_text.set(String::new());
                                            element_label
                                                .set("(no element selected)".to_string());
                                        }
                                        Err(e) => {
                                            submit_error.set(Some(e.to_string()));
                                        }
                                    }
                                });
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
