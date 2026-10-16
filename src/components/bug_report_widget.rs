//! The feedback widget (M7 rebuild on `components::ui`; "Give Feedback"
//! since M141).
//!
//! A docked "Give Feedback" button in the page flow above the footer (it
//! never covers content, at any width) starts select mode: a hint bar at
//! the top asks the reporter to click the part of the page the report is
//! about (any element with `data-feedback-label`), with "Skip" for the whole
//! page and Escape to cancel. The form is a `Dialog`: what the report is
//! about, the kind (Bug, Idea or Praise: labels only, stored as `bug` or
//! `wishlist`, praise with a `[praise] ` body prefix, see
//! `models::bug_report::FeedbackKind`), the text, and "Send feedback". On
//! success a toast offers "Your reports" (`/bugs?tab=mine`).
//!
//! Mounted in `app.rs`; it renders only for signed-in users and never on
//! `/auth`, `/closed-beta` or `/legal`. Plain, neutral copy: no exclamation
//! marks, no emoji, no guilt on Cancel.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::ui::{
    ctrl_enter_submits, Button, ButtonSize, ButtonVariant, Callout, CalloutTone, Dialog,
    DialogSize, Field, Keycap, Segment, SegmentSize, SegmentedControl, TextField, Toast,
    ToastContext,
};
use crate::models::bug_report::FeedbackKind;
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
) -> Result<String, ServerFnError> {
    use crate::models::community::{clean_text, MAX_BUG_DESCRIPTION};
    use crate::server::authz;
    use crate::server::db;
    use crate::server::rate_limit::Bucket;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let description = clean_text(&description);
    if description.is_empty() {
        return Err(ServerFnError::new("Please write a few words first."));
    }
    if description.chars().count() > MAX_BUG_DESCRIPTION {
        return Err(ServerFnError::new("Please keep it under 4,000 characters."));
    }
    if category != "bug" && category != "wishlist" {
        return Err(ServerFnError::new("Choose Bug, Idea or Praise."));
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

    let id = db::create_bug_report(
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
    .map_err(authz::server_err)?;
    crate::server::bug_report_export::request_export();
    // The id lets the dialog attach its screenshot (M141 H2).
    Ok(id)
}

/// Image types the dialog accepts (the server checks the bytes again).
pub const ATTACH_TYPES: &[&str] = &["image/png", "image/jpeg", "image/webp"];
/// Largest image the dialog sends (the server's cap too).
pub const ATTACH_MAX_BYTES: f64 = 5.0 * 1024.0 * 1024.0;

/// Why an attached file is refused in the browser, if it is.
pub fn attach_problem(mime: &str, bytes: f64) -> Option<&'static str> {
    if !ATTACH_TYPES.contains(&mime) {
        Some("Attach a PNG, JPEG or WebP image.")
    } else if bytes > ATTACH_MAX_BYTES {
        Some("The image is too large (up to 5 MB).")
    } else {
        None
    }
}

/// "340 KB", "1.2 MB".
pub fn size_label(bytes: f64) -> String {
    if bytes >= 1024.0 * 1024.0 {
        format!("{:.1} MB", bytes / (1024.0 * 1024.0))
    } else {
        format!("{} KB", (bytes / 1024.0).ceil().max(1.0) as i64)
    }
}

/// The image attached in the dialog, as shown (the file itself is kept in
/// the browser until sending).
#[derive(Clone, Debug, PartialEq)]
struct AttachedImage {
    name: String,
    bytes: f64,
    preview: String,
}

/// Send a report's screenshot after the report: `POST /feedback/attachment/{key}`.
#[cfg(feature = "hydrate")]
async fn upload_attachment(report_id: String, blob: web_sys::Blob) -> Result<(), String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    let key = report_id
        .strip_prefix("bug_report:")
        .unwrap_or(&report_id)
        .to_string();
    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    init.set_body(&blob);
    let win = web_sys::window().ok_or_else(|| "No window".to_string())?;
    let resp =
        JsFuture::from(win.fetch_with_str_and_init(&format!("/feedback/attachment/{key}"), &init))
            .await
            .map_err(|_| "The connection failed.".to_string())?;
    let resp: web_sys::Response = resp
        .dyn_into()
        .map_err(|_| "The connection failed.".to_string())?;
    if resp.ok() {
        return Ok(());
    }
    let text = match resp.text() {
        Ok(p) => JsFuture::from(p).await.ok().and_then(|v| v.as_string()),
        Err(_) => None,
    };
    Err(text
        .filter(|t| !t.trim().is_empty() && t.len() < 200)
        .unwrap_or_else(|| "The server refused it.".into()))
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
    // The dialog's kind: "bug" | "idea" | "praise" (FeedbackKind).
    let kind = RwSignal::new("bug".to_string());
    let text = RwSignal::new(String::new());
    let label = RwSignal::new(WHOLE_PAGE.to_string());
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let sending = RwSignal::new(false);
    // Screenshot (M141 H2): attached by file or paste, sent after the report.
    let attached = RwSignal::new(None::<AttachedImage>);
    let attach_error = RwSignal::new(None::<String>);
    let file_input = NodeRef::<leptos::html::Input>::new();
    #[cfg(feature = "hydrate")]
    let attached_blob: StoredValue<Option<web_sys::Blob>, leptos::prelude::LocalStorage> =
        StoredValue::new_local(None);
    let clear_attachment = move || {
        #[cfg(feature = "hydrate")]
        {
            if let Some(a) = attached.get_untracked() {
                let _ = web_sys::Url::revoke_object_url(&a.preview);
            }
            attached_blob.set_value(None);
        }
        attached.set(None);
        attach_error.set(None);
    };
    #[cfg(feature = "hydrate")]
    let take_file = move |file: web_sys::File| {
        let mime = file.type_();
        let bytes = file.size();
        if let Some(problem) = attach_problem(&mime, bytes) {
            attach_error.set(Some(problem.into()));
            return;
        }
        clear_attachment();
        let blob: web_sys::Blob = file.clone().into();
        let preview = web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default();
        let name = Some(file.name())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "Pasted image".into());
        attached_blob.set_value(Some(blob));
        attached.set(Some(AttachedImage {
            name,
            bytes,
            preview,
        }));
    };
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

    // Focus goes back to the docked trigger when a report ends (sent,
    // cancelled, or select mode left with Escape). The dialog's own focus
    // return points at the element focused before it opened, which is
    // often the hint bar's Skip button that no longer exists.
    let focus_trigger = move || {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::{closure::Closure, JsCast};
            if let Some(win) = web_sys::window() {
                let cb = Closure::once(move || {
                    let el = web_sys::window()
                        .and_then(|w| w.document())
                        .and_then(|d| {
                            d.query_selector("[data-bug-report-dock] button")
                                .ok()
                                .flatten()
                        })
                        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
                    if let Some(el) = el {
                        let _ = el.focus();
                    }
                });
                let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    0,
                );
                cb.forget();
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
        // A second click on the trigger while selecting must not register a
        // second pair of listeners (replacing the stored closures would drop
        // ones the browser still calls).
        if state.get_untracked() == WidgetState::Selecting {
            return;
        }
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
                        focus_trigger();
                    }
                },
            );
            let _ =
                win.add_event_listener_with_callback("keydown", esc_cb.as_ref().unchecked_ref());
            esc_handle.set_value(Some(esc_cb));
        }
    };

    // Following a link that is not a report target (the nav, the footer)
    // ends select mode instead of carrying it to the next page.
    let location = leptos_router::hooks::use_location();
    Effect::new(move |prev: Option<String>| {
        let path = location.pathname.get();
        if prev.as_deref().is_some_and(|p| p != path)
            && state.get_untracked() == WidgetState::Selecting
        {
            state.set(WidgetState::Idle);
            exit_select();
        }
        path
    });

    // Closing the dialog (Cancel, Escape, the close button) ends editing.
    Effect::new(move |_| {
        if !dialog_open.get() && state.get_untracked() == WidgetState::Editing {
            state.set(WidgetState::Idle);
            error.set(None);
            focus_trigger();
        }
    });

    let send = Callback::new(move |_: ()| {
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
            error.set(Some("Please write a few words first.".into()));
            return;
        }
        let (k, t) = {
            let fk = FeedbackKind::parse(&k).unwrap_or(FeedbackKind::Bug);
            let (cat, body) = fk.stored(&t);
            (cat.to_string(), body)
        };
        sending.set(true);
        #[cfg(feature = "hydrate")]
        let blob = attached_blob.get_value();
        spawn_local(async move {
            let result = submit_bug_report(page_url, l, t, k, vw, vh).await;
            #[allow(unused_mut)]
            let mut shot_failed: Option<String> = None;
            #[cfg(feature = "hydrate")]
            if let (Ok(id), Some(blob)) = (&result, blob) {
                if let Err(msg) = upload_attachment(id.clone(), blob).await {
                    shot_failed = Some(msg);
                }
            }
            sending.set(false);
            match result {
                Ok(_) => {
                    dialog_open.set(false);
                    state.set(WidgetState::Idle);
                    text.set(String::new());
                    clear_attachment();
                    focus_trigger();
                    if let (Some(toast), Some(msg)) = (toast, shot_failed) {
                        toast.push.run(Toast::error(format!(
                            "Your feedback is in, but the screenshot was not saved: {msg}"
                        )));
                    } else if let Some(toast) = toast {
                        toast
                            .push
                            .run(Toast::success("Thanks. Your feedback is in.").with_action(
                                "Your reports",
                                Callback::new(move |_| {
                                    navigate.with_value(|n| n("/bugs?tab=mine", Default::default()))
                                }),
                            ));
                    }
                }
                Err(e) => error.set(Some(crate::models::utils::error_text(&e))),
            }
        });
    });

    let placeholder = Signal::derive(move || {
        FeedbackKind::parse(&kind.get())
            .unwrap_or(FeedbackKind::Bug)
            .placeholder()
    });

    view! {
        <div class="mx-auto flex max-w-7xl justify-end px-4 pt-10 sm:px-8" data-bug-report-dock>
            <Button
                variant=ButtonVariant::Secondary
                size=ButtonSize::Touch
                aria_label="Give Feedback"
                on_click=Callback::new(move |_| start_select())
            >
                "Give Feedback"
            </Button>
        </div>

        <Show when=move || state.get() == WidgetState::Selecting fallback=|| ()>
            <div class="fixed inset-x-0 top-14 z-(--z-popover) mx-auto w-[min(40rem,calc(100vw-2rem))] bg-ink-1 shadow-overlay" data-feedback-hint>
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
                            <span class="hidden items-center gap-1 text-caption text-secondary sm:flex">
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
            title="Give Feedback"
            size=DialogSize::Md
            dismiss_on_scrim=false
            footer=move || view! {
                <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| dialog_open.set(false))>"Cancel"</Button>
                <Button variant=ButtonVariant::Primary loading=sending on_click=Callback::new(move |_| send.run(()))>"Send feedback"</Button>
            }
        >
            <div
                class="flex flex-col gap-5"
                on:paste=move |ev: web_sys::ClipboardEvent| {
                    #[cfg(feature = "hydrate")]
                    {
                        let Some(items) = ev.clipboard_data().map(|d| d.items()) else { return };
                        for i in 0..items.length() {
                            let Some(item) = items.get(i) else { continue };
                            if item.kind() == "file" && item.type_().starts_with("image/") {
                                if let Ok(Some(file)) = item.get_as_file() {
                                    ev.prevent_default();
                                    take_file(file);
                                    return;
                                }
                            }
                        }
                    }
                    #[cfg(not(feature = "hydrate"))]
                    let _ = ev;
                }
            >
                <div>
                    <p class="eyebrow">"About"</p>
                    <p class="mt-1 text-body text-primary" data-testid="report-about">{move || label.get()}</p>
                </div>
                <SegmentedControl
                    id="bug-report-type"
                    label="Kind"
                    size=SegmentSize::Sm
                    options={FeedbackKind::ALL.iter().map(|k| Segment::new(k.as_str(), k.label())).collect::<Vec<_>>()}
                    value=kind
                />
                <Field
                    id="bug-report-text"
                    label="Your feedback"
                    hint="Please leave out passwords and other people's personal details."
                    error=Signal::derive(move || error.get())
                >
                    {move || view! {
                        <TextField value=text rows=5 maxlength=4000 placeholder=placeholder.get() on_keydown=ctrl_enter_submits(send) />
                    }}
                </Field>
                <Field
                    id="bug-report-file"
                    label="Screenshot (optional)"
                    hint="PNG, JPEG or WebP, up to 5 MB. We convert it to WebP. It can show other people's names: leave out what you do not want to share."
                    error=Signal::derive(move || attach_error.get())
                >
                    <div class="flex flex-col gap-2" data-testid="report-attachment">
                        <input
                            node_ref=file_input
                            id="bug-report-file"
                            type="file"
                            accept="image/png,image/jpeg,image/webp"
                            class="sr-only"
                            tabindex="-1"
                            on:change=move |_ev| {
                                #[cfg(feature = "hydrate")]
                                if let Some(input) = file_input.get_untracked() {
                                    if let Some(file) = input.files().and_then(|l| l.get(0)) {
                                        take_file(file);
                                    }
                                    input.set_value("");
                                }
                            }
                        />
                        {move || match attached.get() {
                            None => view! {
                                <div class="flex flex-wrap items-center gap-3">
                                    <Button
                                        variant=ButtonVariant::Secondary
                                        on_click=Callback::new(move |_| {
                                            if let Some(input) = file_input.get_untracked() {
                                                input.click();
                                            }
                                        })
                                    >"Attach an image"</Button>
                                    <span class="hidden text-caption text-muted md:inline">"or paste an image (Ctrl+V)"</span>
                                </div>
                            }.into_any(),
                            Some(a) => view! {
                                <div class="flex items-center gap-3 rounded-ctl border border-rule p-2" data-testid="report-attachment-attached">
                                    <img src=a.preview.clone() alt="Screenshot preview" class="h-10 w-16 shrink-0 rounded-ctl object-cover" />
                                    <span class="flex min-w-0 flex-1 flex-col">
                                        <span class="truncate text-label text-primary">{a.name.clone()}</span>
                                        <span class="figures text-caption text-muted">{size_label(a.bytes)}</span>
                                    </span>
                                    <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| clear_attachment())>"Remove"</Button>
                                </div>
                            }.into_any(),
                        }}
                    </div>
                </Field>
            </div>
        </Dialog>
    }
}
