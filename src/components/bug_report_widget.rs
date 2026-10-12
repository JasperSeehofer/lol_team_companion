//! The bug-report widget (M7 rebuild on `components::ui`).
//!
//! A docked "Report a bug" button in the page flow above the footer (it
//! never covers content, at any width) starts select mode: a hint bar at
//! the top asks the reporter to click the part of the page the report is
//! about (any element with `data-feedback-label`), with "Skip" for the whole
//! page and Escape to cancel. The form is a `Dialog`: what the report is
//! about, the type (Bug or Wishlist), the text, and "Send report". On
//! success a toast offers "Your reports" (`/bugs?tab=mine`).
//!
//! Mounted in `app.rs`; it renders only for signed-in users and never on
//! `/auth`, `/closed-beta` or `/legal`. Plain, neutral copy: no exclamation
//! marks, no emoji, no guilt on Cancel.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, Callout, CalloutTone, Dialog, DialogSize, Field, Keycap,
    Segment, SegmentSize, SegmentedControl, TextField, Toast, ToastContext,
};
use crate::pages::profile::get_current_user;

/// Pathname prefixes on which the widget does not render.
pub const HIDDEN_PREFIXES: &[&str] = &["/auth", "/closed-beta", "/legal", "/s/"];

/// The element label stored when no part of the page was chosen.
pub const WHOLE_PAGE: &str = "The whole page";

/// A registered global event listener, kept so it can be removed later
/// (hydrate only; closures are not `Send`, hence `LocalStorage`).
#[cfg(feature = "hydrate")]
type ListenerHandle<E> =
    StoredValue<Option<wasm_bindgen::closure::Closure<dyn Fn(E)>>, leptos::prelude::LocalStorage>;

/// `Selecting` and `Editing` are only entered from browser events.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WidgetState {
    Idle,
    Selecting,
    Editing,
}

/// Persist a bug report or wish from the signed-in user. Rate limited per
/// user (`Feedback`: 5, then one every two minutes).
#[server]
pub async fn submit_bug_report(
    page_url: String,
    element_label: String,
    description: String,
    category: String,
    viewport_w: Option<i32>,
    viewport_h: Option<i32>,
) -> Result<(), ServerFnError> {
    use crate::models::community::{clean_text, MAX_BUG_DESCRIPTION};
    use crate::server::authz;
    use crate::server::db;
    use crate::server::rate_limit::Bucket;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let description = clean_text(&description);
    if description.is_empty() {
        return Err(ServerFnError::new(
            "Please describe the problem or the wish.",
        ));
    }
    if description.chars().count() > MAX_BUG_DESCRIPTION {
        return Err(ServerFnError::new("Please keep it under 4,000 characters."));
    }
    if category != "bug" && category != "wishlist" {
        return Err(ServerFnError::new("Choose Bug or Wishlist."));
    }
    let page_url: String = clean_text(&page_url).chars().take(300).collect();
    let page_url = if page_url.starts_with('/') {
        page_url
    } else {
        "/".to_string()
    };
    let element_label: String = clean_text(&element_label).chars().take(200).collect();
    let element_label = if element_label.is_empty() {
        WHOLE_PAGE.to_string()
    } else {
        element_label
    };
    let clamp = |v: Option<i32>| v.filter(|v| (1..=20_000).contains(v));
    authz::rate_limit_user(Bucket::Feedback, &user).map_err(|_| {
        ServerFnError::new(
            "You sent several reports in a short time. Please wait a few minutes before sending another.",
        )
    })?;

    db::create_bug_report(
        &surreal,
        &user.id,
        page_url,
        element_label,
        description,
        category,
        clamp(viewport_w),
        clamp(viewport_h),
    )
    .await
    .map_err(authz::server_err)
}

/// The widget: a docked trigger, the select-mode hint bar and the form.
///
/// Signed-out visitors and public paths render nothing.
#[component]
pub fn BugReportWidget() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());
    let location = leptos_router::hooks::use_location();
    let excluded = move || {
        let p = location.pathname.get();
        HIDDEN_PREFIXES.iter().any(|prefix| p.starts_with(prefix))
    };
    // `Suspend` resolves from the value serialized by the server, so the
    // server and the hydrating browser render the same tree.
    view! {
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move {
                let authed = matches!(user.await, Ok(Some(_)));
                view! {
                    <Show when=move || authed && !excluded() fallback=|| ()>
                        <WidgetInner />
                    </Show>
                }
            })}
        </Suspense>
    }
}

#[component]
fn WidgetInner() -> impl IntoView {
    let state = RwSignal::new(WidgetState::Idle);
    let dialog_open = RwSignal::new(false);
    let kind = RwSignal::new("bug".to_string());
    let text = RwSignal::new(String::new());
    let label = RwSignal::new(WHOLE_PAGE.to_string());
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let sending = RwSignal::new(false);
    let toast = use_context::<ToastContext>();
    let navigate = leptos_router::hooks::use_navigate();
    let navigate = StoredValue::new(navigate);

    #[allow(unused_variables)]
    #[cfg(feature = "hydrate")]
    let click_handle: ListenerHandle<web_sys::MouseEvent> = StoredValue::new_local(None);
    #[allow(unused_variables)]
    #[cfg(feature = "hydrate")]
    let esc_handle: ListenerHandle<web_sys::KeyboardEvent> = StoredValue::new_local(None);

    // Remove both global listeners and the select-mode marks. Runs on every
    // exit path (Escape, a captured click, Skip).
    let exit_select = move || {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            if let Some(win) = web_sys::window() {
                click_handle.update_value(|slot| {
                    if let Some(cb) = slot.take() {
                        // `capture` must match the add side, or the
                        // listener stays attached.
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

    let open_form = move |about: String| {
        exit_select();
        label.set(about);
        error.set(None);
        state.set(WidgetState::Editing);
        dialog_open.set(true);
    };

    let start_select = move || {
        state.set(WidgetState::Selecting);
        error.set(None);
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::closure::Closure;
            use wasm_bindgen::JsCast;
            let Some(win) = web_sys::window() else {
                return;
            };
            let Some(doc) = win.document() else {
                return;
            };
            if let Some(body) = doc.body() {
                let _ = body.style().set_property("cursor", "crosshair");
            }
            if let Some(el) = doc.document_element() {
                let _ = el.set_attribute("data-feedback-selecting", "");
            }
            // Capture phase, so the page's own handlers do not run.
            let click_cb =
                Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
                    let Some(el) = ev
                        .target()
                        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                    else {
                        return;
                    };
                    // The hint bar's own buttons are not part of the page.
                    if let Ok(Some(_)) = el.closest("[data-feedback-hint]") {
                        return;
                    }
                    let Ok(Some(tagged)) = el.closest("[data-feedback-label]") else {
                        return;
                    };
                    ev.prevent_default();
                    ev.stop_propagation();
                    ev.stop_immediate_propagation();
                    let about = tagged
                        .get_attribute("data-feedback-label")
                        .filter(|l| !l.trim().is_empty())
                        .unwrap_or_else(|| WHOLE_PAGE.to_string());
                    open_form(about);
                });
            let opts = web_sys::AddEventListenerOptions::new();
            opts.set_capture(true);
            let _ = win.add_event_listener_with_callback_and_add_event_listener_options(
                "click",
                click_cb.as_ref().unchecked_ref(),
                &opts,
            );
            click_handle.set_value(Some(click_cb));
            let esc_cb = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |ev: web_sys::KeyboardEvent| {
                    if ev.key() == "Escape" {
                        state.set(WidgetState::Idle);
                        exit_select();
                    }
                },
            );
            let _ =
                win.add_event_listener_with_callback("keydown", esc_cb.as_ref().unchecked_ref());
            esc_handle.set_value(Some(esc_cb));
        }
    };

    // Closing the dialog (Cancel, Escape, the close button) ends editing.
    Effect::new(move |_| {
        if !dialog_open.get() && state.get_untracked() == WidgetState::Editing {
            state.set(WidgetState::Idle);
            error.set(None);
        }
    });

    let send = move |_| {
        if sending.get_untracked() {
            return;
        }
        error.set(None);
        #[cfg(feature = "hydrate")]
        let (page_url, vw, vh) = web_sys::window()
            .map(|w| {
                let url = w.location().pathname().ok().unwrap_or_default();
                let size = |v: Result<wasm_bindgen::JsValue, _>| {
                    v.ok().and_then(|v| v.as_f64()).map(|f| f as i32)
                };
                (url, size(w.inner_width()), size(w.inner_height()))
            })
            .unwrap_or_default();
        #[cfg(not(feature = "hydrate"))]
        let (page_url, vw, vh): (String, Option<i32>, Option<i32>) = (String::new(), None, None);
        let (k, t, l) = (
            kind.get_untracked(),
            text.get_untracked(),
            label.get_untracked(),
        );
        if t.trim().is_empty() {
            error.set(Some("Please describe the problem or the wish.".into()));
            return;
        }
        sending.set(true);
        spawn_local(async move {
            let result = submit_bug_report(page_url, l, t, k, vw, vh).await;
            sending.set(false);
            match result {
                Ok(()) => {
                    dialog_open.set(false);
                    state.set(WidgetState::Idle);
                    text.set(String::new());
                    if let Some(toast) = toast {
                        toast
                            .push
                            .run(Toast::success("Thanks. Your report is in.").with_action(
                                "Your reports",
                                Callback::new(move |_| {
                                    navigate.with_value(|n| n("/bugs?tab=mine", Default::default()))
                                }),
                            ));
                    }
                }
                Err(e) => error.set(Some(
                    e.to_string()
                        .trim_start_matches("error running server function: ")
                        .to_string(),
                )),
            }
        });
    };

    let placeholder = Signal::derive(move || {
        if kind.get() == "wishlist" {
            "What would help your team, and when would you use it?"
        } else {
            "What did you do, what did you expect, what happened instead?"
        }
    });

    view! {
        <div class="mx-auto flex max-w-7xl justify-end px-4 pt-10 sm:px-8" data-bug-report-dock>
            <Button
                variant=ButtonVariant::Secondary
                size=ButtonSize::Sm
                aria_label="Report a bug or a wish"
                on_click=Callback::new(move |_| start_select())
            >
                "Report a bug"
            </Button>
        </div>

        <Show when=move || state.get() == WidgetState::Selecting fallback=|| ()>
            <div class="fixed inset-x-0 top-14 z-(--z-popover) mx-auto w-[min(40rem,calc(100vw-2rem))]" data-feedback-hint>
                <Callout
                    tone=CalloutTone::Info
                    action=move || view! {
                        <span class="flex items-center gap-3">
                            <Button
                                size=ButtonSize::Sm
                                variant=ButtonVariant::Secondary
                                on_click=Callback::new(move |_| open_form(WHOLE_PAGE.to_string()))
                            >
                                "Skip"
                            </Button>
                            <span class="flex items-center gap-1 text-caption text-secondary">
                                <Keycap keys="Esc" />
                                " to cancel"
                            </span>
                        </span>
                    }
                >
                    "Click the part of the page your report is about."
                </Callout>
            </div>
        </Show>

        <Dialog
            id="bug-report"
            open=dialog_open
            title="Report"
            size=DialogSize::Md
            dismiss_on_scrim=false
            footer=move || view! {
                <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| dialog_open.set(false))>"Cancel"</Button>
                <Button variant=ButtonVariant::Primary loading=sending on_click=Callback::new(send)>"Send report"</Button>
            }
        >
            <div class="flex flex-col gap-5">
                <div>
                    <p class="eyebrow">"About"</p>
                    <p class="mt-1 text-body text-primary" data-testid="report-about">{move || label.get()}</p>
                </div>
                <SegmentedControl
                    id="bug-report-type"
                    label="Type"
                    size=SegmentSize::Sm
                    options=vec![Segment::new("bug", "Bug"), Segment::new("wishlist", "Wishlist")]
                    value=kind
                />
                <Field
                    id="bug-report-text"
                    label="What happened, or what would you like?"
                    hint="Please leave out passwords and other people's personal details."
                    error=Signal::derive(move || error.get())
                >
                    {move || view! {
                        <TextField value=text rows=5 maxlength=4000 placeholder=placeholder.get() />
                    }}
                </Field>
            </div>
        </Dialog>
    }
}
