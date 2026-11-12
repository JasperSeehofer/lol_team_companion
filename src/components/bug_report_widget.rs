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
//! On a desktop browser with screen capture (M149 B), the screenshot floor
//! starts with "Select an area of the page": the dialog hides, the browser's
//! own share prompt asks for the tab, one frame is drawn and every track is
//! stopped at once (nothing is recorded), and [`AreaPicker`] lets the
//! reporter crop that still; the crop goes through the same checks and
//! preview as an attached file. Phones and browsers without
//! `getDisplayMedia` never see the button (decided after hydration).
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
use crate::models::area_select::{size_readout, Rect};
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
    /// The dialog is hidden while the browser's share prompt is open and
    /// one frame is grabbed (M149 B).
    Capturing,
    /// The area picker is over the captured still.
    Picking,
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

/// What the screenshot floor says when "Select an area of the page" ends
/// without a picture (M149 B, card B wording).
pub const CAPTURE_CANCELLED: &str = "Screen capture was cancelled. Nothing was attached.";
pub const CAPTURE_DENIED: &str =
    "The browser did not allow screen capture. You can still attach an image or paste one.";
pub const CAPTURE_FAILED: &str = "Could not capture this page. Attach an image instead.";
/// Where focus lands when the dialog comes back from the area picker.
pub const FOCUS_PICK_BUTTON: &str = "[data-area-pick] button";
pub const FOCUS_REMOVE: &str = "[data-testid=report-attachment-attached] button";

/// Focus the element matching `selector` once the dialog has reopened.
#[cfg(feature = "hydrate")]
fn focus_later(selector: &'static str) {
    use wasm_bindgen::{closure::Closure, JsCast};
    let Some(win) = web_sys::window() else {
        return;
    };
    let cb = Closure::once(move || {
        let el = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector(selector).ok().flatten())
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(el) = el {
            let _ = el.focus();
        }
    });
    let _ =
        win.set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), 50);
    cb.forget();
}

/// Screen capture for "Select an area of the page" (M149 B), browser only.
/// One frame is drawn from the stream and every track is stopped right
/// after, on every path; nothing is recorded or sent anywhere.
#[cfg(feature = "hydrate")]
mod capture {
    use crate::models::area_select::Crop;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CaptureError {
        Cancelled,
        NotAllowed,
        Failed,
    }

    impl CaptureError {
        pub fn message(self) -> &'static str {
            match self {
                CaptureError::Cancelled => super::CAPTURE_CANCELLED,
                CaptureError::NotAllowed => super::CAPTURE_DENIED,
                CaptureError::Failed => super::CAPTURE_FAILED,
            }
        }
    }

    /// Dismissing the share prompt is a `NotAllowedError` too; a refusal
    /// by the system or a permissions policy says so in its message.
    fn classify(e: &JsValue) -> CaptureError {
        let field = |k: &str| {
            js_sys::Reflect::get(e, &JsValue::from_str(k))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default()
        };
        let name = field("name");
        let message = field("message").to_lowercase();
        match name.as_str() {
            "NotAllowedError" if message.contains("system") || message.contains("policy") => {
                CaptureError::NotAllowed
            }
            "NotAllowedError" => CaptureError::Cancelled,
            "SecurityError" | "NotSupportedError" => CaptureError::NotAllowed,
            _ => CaptureError::Failed,
        }
    }

    /// `getDisplayMedia` exists, the window is at least `md` wide and the
    /// main pointer is fine (a mouse or trackpad).
    pub fn supported() -> bool {
        let Some(win) = web_sys::window() else {
            return false;
        };
        let api = win.navigator().media_devices().ok().is_some_and(|d| {
            js_sys::Reflect::get(&d, &JsValue::from_str("getDisplayMedia"))
                .is_ok_and(|f| f.is_function())
        });
        let fits = win
            .match_media("(min-width: 768px) and (pointer: fine)")
            .ok()
            .flatten()
            .is_some_and(|m| m.matches());
        api && fits
    }

    /// Ask for the stream. Must run inside the click (a user gesture).
    /// `preferCurrentTab`, `selfBrowserSurface` and `displaySurface` are
    /// not in web-sys's dictionary, so they are set as plain properties;
    /// browsers that do not know them ignore them.
    pub fn request() -> Result<js_sys::Promise, CaptureError> {
        let win = web_sys::window().ok_or(CaptureError::Failed)?;
        let devices = win
            .navigator()
            .media_devices()
            .map_err(|_| CaptureError::NotAllowed)?;
        let constraints = web_sys::DisplayMediaStreamConstraints::new();
        let video = js_sys::Object::new();
        let _ = js_sys::Reflect::set(
            &video,
            &JsValue::from_str("displaySurface"),
            &JsValue::from_str("browser"),
        );
        constraints.set_video(&video);
        constraints.set_audio_bool(false);
        let _ = js_sys::Reflect::set(
            &constraints,
            &JsValue::from_str("preferCurrentTab"),
            &JsValue::TRUE,
        );
        let _ = js_sys::Reflect::set(
            &constraints,
            &JsValue::from_str("selfBrowserSurface"),
            &JsValue::from_str("include"),
        );
        devices
            .get_display_media_with_constraints(&constraints)
            .map_err(|e| classify(&e))
    }

    /// Wait for the stream, draw one frame to a canvas, stop every track.
    pub async fn grab(
        promise: js_sys::Promise,
    ) -> Result<web_sys::HtmlCanvasElement, CaptureError> {
        let value = JsFuture::from(promise).await.map_err(|e| classify(&e))?;
        let Ok(stream) = value.dyn_into::<web_sys::MediaStream>() else {
            return Err(CaptureError::Failed);
        };
        let frame = first_frame(&stream).await;
        // At once, whether the frame was drawn or not.
        stop_tracks(&stream);
        frame
    }

    pub fn stop_tracks(stream: &web_sys::MediaStream) {
        for t in stream.get_tracks().iter() {
            if let Ok(track) = t.dyn_into::<web_sys::MediaStreamTrack>() {
                track.stop();
            }
        }
    }

    async fn first_frame(
        stream: &web_sys::MediaStream,
    ) -> Result<web_sys::HtmlCanvasElement, CaptureError> {
        let doc = web_sys::window()
            .and_then(|w| w.document())
            .ok_or(CaptureError::Failed)?;
        let video: web_sys::HtmlVideoElement = doc
            .create_element("video")
            .ok()
            .and_then(|e| e.dyn_into().ok())
            .ok_or(CaptureError::Failed)?;
        video.set_muted(true);
        video.set_src_object(Some(stream));
        let drawn = draw_video(&doc, &video).await;
        video.set_src_object(None);
        drawn
    }

    async fn draw_video(
        doc: &web_sys::Document,
        video: &web_sys::HtmlVideoElement,
    ) -> Result<web_sys::HtmlCanvasElement, CaptureError> {
        if let Ok(p) = video.play() {
            let _ = JsFuture::from(p).await;
        }
        // HAVE_CURRENT_DATA (2) with a size, for up to three seconds.
        for _ in 0..60 {
            if video.ready_state() >= 2 && video.video_width() > 0 && video.video_height() > 0 {
                break;
            }
            sleep(50).await;
        }
        let (w, h) = (video.video_width(), video.video_height());
        if w == 0 || h == 0 {
            return Err(CaptureError::Failed);
        }
        let canvas = new_canvas(doc, w, h)?;
        context(&canvas)?
            .draw_image_with_html_video_element(video, 0.0, 0.0)
            .map_err(|_| CaptureError::Failed)?;
        Ok(canvas)
    }

    fn new_canvas(
        doc: &web_sys::Document,
        w: u32,
        h: u32,
    ) -> Result<web_sys::HtmlCanvasElement, CaptureError> {
        let canvas: web_sys::HtmlCanvasElement = doc
            .create_element("canvas")
            .ok()
            .and_then(|e| e.dyn_into().ok())
            .ok_or(CaptureError::Failed)?;
        canvas.set_width(w);
        canvas.set_height(h);
        Ok(canvas)
    }

    fn context(
        canvas: &web_sys::HtmlCanvasElement,
    ) -> Result<web_sys::CanvasRenderingContext2d, CaptureError> {
        canvas
            .get_context("2d")
            .ok()
            .flatten()
            .and_then(|c| c.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
            .ok_or(CaptureError::Failed)
    }

    /// The crop of the still as a PNG blob.
    pub async fn crop_png(source: &web_sys::HtmlCanvasElement, c: Crop) -> Option<web_sys::Blob> {
        let doc = web_sys::window().and_then(|w| w.document())?;
        let out = new_canvas(&doc, c.w, c.h).ok()?;
        let (x, y, w, h) = (c.x as f64, c.y as f64, c.w as f64, c.h as f64);
        context(&out)
            .ok()?
            .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                source, x, y, w, h, 0.0, 0.0, w, h,
            )
            .ok()?;
        let promise =
            js_sys::Promise::new(&mut |resolve: js_sys::Function, reject: js_sys::Function| {
                if out.to_blob_with_type(&resolve, "image/png").is_err() {
                    let _ = reject.call0(&JsValue::NULL);
                }
            });
        JsFuture::from(promise)
            .await
            .ok()?
            .dyn_into::<web_sys::Blob>()
            .ok()
    }

    /// Resolves once the open `<dialog data-dialog="{id}">` has fired its
    /// `close` event (at once when it is not open; after a second at the
    /// latest). `HTMLDialogElement::close()` queues that event as a task, so
    /// a capture refused at once (a microtask) would otherwise reopen the
    /// dialog first and the late `close` would shut it again.
    /// Call it before the dialog is asked to close.
    pub fn dialog_closed(id: &str) -> js_sys::Promise {
        let dialog = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| {
                d.query_selector(&format!("dialog[data-dialog=\"{id}\"]"))
                    .ok()
                    .flatten()
            })
            .and_then(|e| e.dyn_into::<web_sys::HtmlDialogElement>().ok())
            .filter(|d| d.open());
        js_sys::Promise::new(
            &mut |resolve: js_sys::Function, _reject: js_sys::Function| {
                let Some(dialog) = dialog.as_ref() else {
                    let _ = resolve.call0(&JsValue::NULL);
                    return;
                };
                let opts = web_sys::AddEventListenerOptions::new();
                opts.set_once(true);
                let listened = dialog
                    .add_event_listener_with_callback_and_add_event_listener_options(
                        "close", &resolve, &opts,
                    )
                    .is_ok();
                let timed = web_sys::window().is_some_and(|w| {
                    w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 1000)
                        .is_ok()
                });
                if !listened && !timed {
                    let _ = resolve.call0(&JsValue::NULL);
                }
            },
        )
    }

    async fn sleep(ms: i32) {
        let promise = js_sys::Promise::new(
            &mut |resolve: js_sys::Function, _reject: js_sys::Function| {
                let scheduled = web_sys::window().is_some_and(|w| {
                    w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                        .is_ok()
                });
                if !scheduled {
                    let _ = resolve.call0(&JsValue::NULL);
                }
            },
        );
        let _ = JsFuture::from(promise).await;
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
    // One path for a chosen file, a pasted image and a selected area
    // (M149 B): the same type and size checks, the same preview.
    #[cfg(feature = "hydrate")]
    let take_blob = move |blob: web_sys::Blob, name: String| {
        let mime = blob.type_();
        let bytes = blob.size();
        if let Some(problem) = attach_problem(&mime, bytes) {
            attach_error.set(Some(problem.into()));
            return;
        }
        clear_attachment();
        let preview = web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default();
        attached_blob.set_value(Some(blob));
        attached.set(Some(AttachedImage {
            name,
            bytes,
            preview,
        }));
    };
    #[cfg(feature = "hydrate")]
    let take_file = move |file: web_sys::File| {
        let name = Some(file.name())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "Pasted image".into());
        let blob: web_sys::Blob = file.into();
        take_blob(blob, name);
    };

    // "Select an area of the page" (M149 B): shown only on a desktop
    // browser with screen capture, decided after hydration (the server
    // renders nothing, so there is no mismatch and no dead control).
    let area_supported = RwSignal::new(false);
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        area_supported.set(capture::supported());
    });
    let area = RwSignal::new(Rect::new(0.0, 0.0, 0.0, 0.0));
    let area_view = RwSignal::new((0.0_f64, 0.0_f64));
    let still_box = NodeRef::<leptos::html::Div>::new();
    #[cfg(feature = "hydrate")]
    let still: StoredValue<Option<web_sys::HtmlCanvasElement>, leptos::prelude::LocalStorage> =
        StoredValue::new_local(None);
    // The captured still goes into the picker once it is mounted.
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let (Some(el), Some(canvas)) = (still_box.get(), still.get_value()) {
            canvas.set_class_name("block h-full w-full object-contain");
            let _ = el.append_child(&canvas);
        }
    });
    // Back to the dialog, with the floor's message and focus where it belongs.
    let reopen = move |message: Option<&'static str>, focus: &'static str| {
        state.set(WidgetState::Editing);
        dialog_open.set(true);
        if let Some(m) = message {
            attach_error.set(Some(m.into()));
        }
        #[cfg(feature = "hydrate")]
        focus_later(focus);
        #[cfg(not(feature = "hydrate"))]
        let _ = focus;
    };
    let start_area = move || {
        if state.get_untracked() != WidgetState::Editing {
            return;
        }
        attach_error.set(None);
        // Before closing: resolves once the dialog's queued `close` event
        // has run, so a quick refusal cannot be undone by it.
        #[cfg(feature = "hydrate")]
        let closed = capture::dialog_closed("bug-report");
        // Capturing first, so closing the dialog does not end the report.
        state.set(WidgetState::Capturing);
        dialog_open.set(false);
        #[cfg(feature = "hydrate")]
        {
            // Called here, inside the click: the browser only asks from a
            // user gesture.
            let request = capture::request();
            spawn_local(async move {
                let grabbed = match request {
                    Ok(p) => capture::grab(p).await,
                    Err(e) => Err(e),
                };
                // One finish path: every outcome waits for the close, then
                // either shows the picker or reopens with its message.
                let _ = wasm_bindgen_futures::JsFuture::from(closed).await;
                match grabbed {
                    Ok(canvas) => {
                        still.set_value(Some(canvas));
                        state.set(WidgetState::Picking);
                    }
                    Err(e) => reopen(Some(e.message()), FOCUS_PICK_BUTTON),
                }
            });
        }
    };
    let confirm_area = Callback::new(move |_: ()| {
        #[cfg(feature = "hydrate")]
        {
            let Some(source) = still.get_value() else {
                return;
            };
            let (vw, vh) = area_view.get_untracked();
            let Some(crop) = crate::models::area_select::crop_to_source(
                area.get_untracked(),
                vw,
                vh,
                f64::from(source.width()),
                f64::from(source.height()),
            ) else {
                return;
            };
            // Taken out first: a second Enter does nothing.
            still.set_value(None);
            spawn_local(async move {
                match capture::crop_png(&source, crop).await {
                    Some(blob) => {
                        take_blob(blob, "Selected area".into());
                        let focus = if attached.get_untracked().is_some() {
                            FOCUS_REMOVE
                        } else {
                            FOCUS_PICK_BUTTON
                        };
                        reopen(None, focus);
                    }
                    None => reopen(Some(CAPTURE_FAILED), FOCUS_PICK_BUTTON),
                }
            });
        }
    });
    let cancel_area = Callback::new(move |_: ()| {
        #[cfg(feature = "hydrate")]
        still.set_value(None);
        reopen(Some(CAPTURE_CANCELLED), FOCUS_PICK_BUTTON);
    });
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

        <Show when=move || state.get() == WidgetState::Picking fallback=|| ()>
            <AreaPicker
                sel=area
                view=area_view
                still_box=still_box
                on_confirm=confirm_area
                on_cancel=cancel_area
            />
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
                                    <Show when=move || area_supported.get() fallback=|| ()>
                                        <span class="contents" data-area-pick>
                                            <Button
                                                variant=ButtonVariant::Secondary
                                                on_click=Callback::new(move |_| start_area())
                                            >"Select an area of the page"</Button>
                                        </span>
                                    </Show>
                                    <Button
                                        variant=ButtonVariant::Secondary
                                        on_click=Callback::new(move |_| {
                                            if let Some(input) = file_input.get_untracked() {
                                                input.click();
                                            }
                                        })
                                    >"Attach an image"</Button>
                                    <span class="hidden text-caption text-muted md:inline">"or paste an image (Ctrl+V)"</span>
                                    <Show when=move || area_supported.get() fallback=|| ()>
                                        <p class="w-full text-caption text-muted">"A captured area stays in your browser until you send."</p>
                                    </Show>
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

/// "Select an area of the page" (M149 B): a full-viewport overlay on the
/// captured still. Outside the selection a flat `bg-page` layer at 60 %
/// (four edge rectangles), the selection a `border-accent` rectangle with
/// square handles and a size readout; selections under 24 x 24 px are
/// ignored. Starts with a centred rectangle half the viewport each way.
///
/// A11y: a modal dialog (`role="dialog"`, `aria-modal`) labelled "Select an
/// area of the page", focused on open; arrow keys move the area 8 px (Shift:
/// 1 px), Alt + arrows resize it, Enter confirms, Esc cancels, Tab cycles the
/// bar's native "Use area" and "Cancel" buttons; a polite live region reads
/// the size and position (debounced); focus returns to the feedback dialog.
/// The still stays in the browser until the reporter sends.
#[component]
fn AreaPicker(
    /// The selection in CSS pixels over the still box.
    sel: RwSignal<Rect>,
    /// The still box's size in CSS pixels, measured on mount.
    view: RwSignal<(f64, f64)>,
    /// Where the captured still (a canvas) is mounted.
    still_box: NodeRef<leptos::html::Div>,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let overlay = NodeRef::<leptos::html::Div>::new();
    // Drag start (box coordinates) and the selection before the drag.
    #[allow(unused_variables)]
    let drag: RwSignal<Option<(f64, f64, Rect)>> = RwSignal::new(None);
    let spoken = RwSignal::new(String::new());
    #[allow(unused_variables)]
    let speak_timer: RwSignal<Option<i32>> = RwSignal::new(None);

    // Measure, start centred, take focus.
    Effect::new(move |_| {
        if let Some(el) = overlay.get() {
            #[cfg(feature = "hydrate")]
            {
                let (w, h) = still_box
                    .get_untracked()
                    .map(|b| {
                        let r = b.get_bounding_client_rect();
                        (r.width(), r.height())
                    })
                    .unwrap_or((0.0, 0.0));
                view.set((w, h));
                sel.set(Rect::centred(w, h));
                let _ = el.focus();
            }
            #[cfg(not(feature = "hydrate"))]
            let _ = el;
        }
    });

    // The live region, debounced so a held arrow key is read once.
    Effect::new(move |_| {
        let text = crate::models::area_select::announcement(sel.get());
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::{closure::Closure, JsCast};
            let Some(win) = web_sys::window() else {
                return;
            };
            if let Some(id) = speak_timer.get_untracked() {
                win.clear_timeout_with_handle(id);
            }
            let cb = Closure::once(move || spoken.set(text));
            if let Ok(id) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                500,
            ) {
                speak_timer.set(Some(id));
            }
            cb.forget();
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = text;
    });

    // A point of a pointer event in still-box coordinates.
    #[cfg(feature = "hydrate")]
    let local = move |ev: &web_sys::PointerEvent| -> Option<(f64, f64)> {
        let r = still_box.get_untracked()?.get_bounding_client_rect();
        Some((
            f64::from(ev.client_x()) - r.left(),
            f64::from(ev.client_y()) - r.top(),
        ))
    };

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        // Enter on a bar button is that button's own click.
        #[cfg(feature = "hydrate")]
        let on_button = {
            use wasm_bindgen::JsCast;
            ev.target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                .is_some_and(|el| el.tag_name().eq_ignore_ascii_case("button"))
        };
        #[cfg(not(feature = "hydrate"))]
        let on_button = false;
        let (vw, vh) = view.get_untracked();
        let step = if ev.shift_key() { 1.0 } else { 8.0 };
        let (dx, dy) = match key.as_str() {
            "ArrowLeft" => (-step, 0.0),
            "ArrowRight" => (step, 0.0),
            "ArrowUp" => (0.0, -step),
            "ArrowDown" => (0.0, step),
            "Escape" => {
                ev.prevent_default();
                ev.stop_propagation();
                on_cancel.run(());
                return;
            }
            "Enter" if !on_button => {
                ev.prevent_default();
                on_confirm.run(());
                return;
            }
            "Tab" => {
                // Keep focus inside the picker: cycle the bar's buttons.
                ev.prevent_default();
                #[cfg(feature = "hydrate")]
                if let Some(root) = overlay.get_untracked() {
                    use wasm_bindgen::JsCast;
                    let buttons: Vec<web_sys::HtmlElement> = root
                        .query_selector_all("[data-area-bar] button")
                        .ok()
                        .map(|l| {
                            (0..l.length())
                                .filter_map(|i| l.get(i))
                                .filter_map(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
                                .collect()
                        })
                        .unwrap_or_default();
                    if buttons.is_empty() {
                        return;
                    }
                    let active = web_sys::window()
                        .and_then(|w| w.document())
                        .and_then(|d| d.active_element());
                    let at = buttons.iter().position(|b| {
                        active
                            .as_ref()
                            .is_some_and(|a| a == b.unchecked_ref::<web_sys::Element>())
                    });
                    let n = buttons.len();
                    let next = match (at, ev.shift_key()) {
                        (Some(i), false) => (i + 1) % n,
                        (Some(i), true) => (i + n - 1) % n,
                        (None, false) => 0,
                        (None, true) => n - 1,
                    };
                    let _ = buttons[next].focus();
                }
                return;
            }
            _ => return,
        };
        ev.prevent_default();
        let cur = sel.get_untracked();
        sel.set(if ev.alt_key() {
            cur.resized(dx, dy, vw, vh)
        } else {
            cur.moved(dx, dy, vw, vh)
        });
    };

    let rect_style = move || {
        let r = sel.get().normalised();
        format!(
            "left:{}px;top:{}px;width:{}px;height:{}px",
            r.x, r.y, r.w, r.h
        )
    };
    // The four dim layers around the selection.
    let dim_top = move || {
        format!(
            "left:0;top:0;right:0;height:{}px",
            sel.get().normalised().y.max(0.0)
        )
    };
    let dim_bottom = move || {
        let r = sel.get().normalised();
        format!("left:0;right:0;top:{}px;bottom:0", r.y + r.h)
    };
    let dim_left = move || {
        let r = sel.get().normalised();
        format!(
            "left:0;top:{}px;width:{}px;height:{}px",
            r.y,
            r.x.max(0.0),
            r.h
        )
    };
    let dim_right = move || {
        let r = sel.get().normalised();
        format!(
            "right:0;left:{}px;top:{}px;height:{}px",
            r.x + r.w,
            r.y,
            r.h
        )
    };
    // The readout sits under the selection, or inside it near the bottom.
    let readout_style = move || {
        let r = sel.get().normalised();
        let (_, vh) = view.get();
        let top = if r.y + r.h + 32.0 < vh {
            r.y + r.h + 6.0
        } else {
            (r.y + r.h - 26.0).max(0.0)
        };
        format!("left:{}px;top:{}px", r.x, top)
    };

    view! {
        <div
            node_ref=overlay
            // The selection rectangle is the focus indicator here. lint:allow(g12)
            class="fixed inset-0 z-(--z-dialog) cursor-crosshair select-none bg-page outline-none" // lint:allow(g12)
            role="dialog"
            aria-modal="true"
            aria-label="Select an area of the page"
            aria-describedby="area-picker-help"
            tabindex="-1"
            data-testid="area-picker"
            on:keydown=on_keydown
            on:pointerdown=move |ev: web_sys::PointerEvent| {
                #[cfg(feature = "hydrate")]
                {
                    if ev.button() != 0 {
                        return;
                    }
                    let Some((x, y)) = local(&ev) else { return };
                    ev.prevent_default();
                    if let Some(el) = overlay.get_untracked() {
                        let _ = el.set_pointer_capture(ev.pointer_id());
                        let _ = el.focus();
                    }
                    drag.set(Some((x, y, sel.get_untracked())));
                    sel.set(Rect::new(x, y, 0.0, 0.0));
                }
                #[cfg(not(feature = "hydrate"))]
                let _ = ev;
            }
            on:pointermove=move |ev: web_sys::PointerEvent| {
                #[cfg(feature = "hydrate")]
                {
                    let Some((ax, ay, _)) = drag.get_untracked() else { return };
                    let Some((x, y)) = local(&ev) else { return };
                    let (vw, vh) = view.get_untracked();
                    sel.set(Rect::from_corners(ax, ay, x, y).clamped(vw, vh));
                }
                #[cfg(not(feature = "hydrate"))]
                let _ = ev;
            }
            on:pointerup=move |ev: web_sys::PointerEvent| {
                #[cfg(feature = "hydrate")]
                {
                    let Some((_, _, before)) = drag.get_untracked() else { return };
                    drag.set(None);
                    if let Some(el) = overlay.get_untracked() {
                        let _ = el.release_pointer_capture(ev.pointer_id());
                    }
                    // A click or a tiny drag keeps the previous selection.
                    if !sel.get_untracked().is_usable() {
                        sel.set(before);
                    }
                }
                #[cfg(not(feature = "hydrate"))]
                let _ = ev;
            }
        >
            <div node_ref=still_box class="absolute inset-0" aria-hidden="true"></div>
            <div class="absolute bg-page/60" style=dim_top aria-hidden="true"></div>
            <div class="absolute bg-page/60" style=dim_bottom aria-hidden="true"></div>
            <div class="absolute bg-page/60" style=dim_left aria-hidden="true"></div>
            <div class="absolute bg-page/60" style=dim_right aria-hidden="true"></div>
            <div class="absolute border-2 border-accent" style=rect_style data-testid="area-selection" aria-hidden="true">
                <span class="absolute -left-1.5 -top-1.5 size-3 bg-accent"></span>
                <span class="absolute -right-1.5 -top-1.5 size-3 bg-accent"></span>
                <span class="absolute -bottom-1.5 -left-1.5 size-3 bg-accent"></span>
                <span class="absolute -bottom-1.5 -right-1.5 size-3 bg-accent"></span>
            </div>
            <span
                class="figures absolute bg-ink-2 px-2 text-caption text-primary"
                style=readout_style
                data-testid="area-size"
                aria-hidden="true"
            >
                {move || size_readout(sel.get())}
            </span>
            <p id="area-picker-help" class="sr-only">"Drag to select, or use the arrow keys."</p>
            <p class="sr-only" aria-live="polite">{move || spoken.get()}</p>
            <div
                class="absolute inset-x-0 bottom-6 mx-auto flex w-fit cursor-default items-center gap-3 border border-rule bg-ink-2 px-4 py-3"
                data-area-bar
                on:pointerdown=|ev: web_sys::PointerEvent| ev.stop_propagation()
            >
                <span class="hidden items-center gap-1 text-caption text-secondary lg:flex">
                    <Keycap keys="Enter" />
                    " use, "
                    <Keycap keys="Esc" />
                    " cancel, arrows move, Alt + arrows resize"
                </span>
                <Button
                    variant=ButtonVariant::Primary
                    size=ButtonSize::Touch
                    on_click=Callback::new(move |_| on_confirm.run(()))
                >
                    "Use area"
                </Button>
                <Button
                    variant=ButtonVariant::Quiet
                    on_click=Callback::new(move |_| on_cancel.run(()))
                >
                    "Cancel"
                </Button>
            </div>
        </div>
    }
}
