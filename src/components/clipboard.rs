//! Clipboard and system share sheet (M148, atelier cards C1 and C9).
//!
//! The rule these helpers exist for: a page says "Copied" only after the
//! browser's promise resolved. `navigator.clipboard.writeText` is
//! asynchronous and can be refused (no clipboard API on plain HTTP, a denied
//! permission, a document without focus); firing it and toasting at once
//! told people a code was copied when it was not.
//!
//! Both helpers are browser-only. On the server (the `ssr` build renders
//! the page but never runs a click handler) they return an error without
//! doing anything.

/// Copy `text` to the clipboard and wait for the browser's answer.
///
/// `Ok(())` only once the `writeText` promise resolved. `Err` when there is
/// no clipboard API (insecure context, old browser) or the promise was
/// rejected; the message is for logs, the page shows its own words.
pub async fn copy_text(text: &str) -> Result<(), String> {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::JsFuture;

        let navigator = web_sys::window()
            .ok_or_else(|| "no window".to_string())?
            .navigator();
        // `navigator.clipboard` is undefined outside a secure context;
        // calling `writeText` on it would throw.
        let has_clipboard = js_sys::Reflect::get(&navigator, &JsValue::from_str("clipboard"))
            .map(|c| !c.is_undefined() && !c.is_null())
            .unwrap_or(false);
        if !has_clipboard {
            return Err("clipboard API unavailable".into());
        }
        let promise = navigator.clipboard().write_text(text);
        JsFuture::from(promise)
            .await
            .map(|_| ())
            .map_err(|e| format!("clipboard write refused: {e:?}"))
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = text;
        Err("copying needs the browser".into())
    }
}

/// Whether the browser offers the system share sheet (`navigator.share`).
/// Phones mostly do, desktop browsers mostly not: show a Share button only
/// when this is true (call it after hydration, it is `false` on the server).
pub fn can_share() -> bool {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsValue;
        web_sys::window()
            .and_then(|w| js_sys::Reflect::get(&w.navigator(), &JsValue::from_str("share")).ok())
            .map(|f| f.is_function())
            .unwrap_or(false)
    }
    #[cfg(not(feature = "hydrate"))]
    {
        false
    }
}

/// Open the system share sheet for `url` with `title`.
///
/// `Ok(true)` when the person shared, `Ok(false)` when they closed the sheet
/// (`AbortError`: say nothing), `Err` when there is no share sheet or it
/// failed (fall back to the copy failure message).
pub async fn share_url(title: &str, url: &str) -> Result<bool, String> {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::{JsCast, JsValue};
        use wasm_bindgen_futures::JsFuture;

        let navigator = web_sys::window()
            .ok_or_else(|| "no window".to_string())?
            .navigator();
        let share = js_sys::Reflect::get(&navigator, &JsValue::from_str("share"))
            .ok()
            .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
            .ok_or_else(|| "share sheet unavailable".to_string())?;
        let data = js_sys::Object::new();
        let set = |k: &str, v: &str| {
            js_sys::Reflect::set(&data, &JsValue::from_str(k), &JsValue::from_str(v))
                .map_err(|e| format!("share data: {e:?}"))
        };
        set("title", title)?;
        set("url", url)?;
        let promise = share
            .call1(&navigator, &data)
            .map_err(|e| format!("share refused: {e:?}"))?
            .dyn_into::<js_sys::Promise>()
            .map_err(|_| "share did not return a promise".to_string())?;
        match JsFuture::from(promise).await {
            Ok(_) => Ok(true),
            Err(e) => {
                let name = js_sys::Reflect::get(&e, &JsValue::from_str("name"))
                    .ok()
                    .and_then(|n| n.as_string());
                if name.as_deref() == Some("AbortError") {
                    Ok(false)
                } else {
                    Err(format!("share failed: {e:?}"))
                }
            }
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (title, url);
        Err("sharing needs the browser".into())
    }
}

/// Select everything inside `node` (the fallback after a refused copy: the
/// text is ready for the person to copy by hand). Browser-only.
#[cfg(feature = "hydrate")]
pub fn select_contents(node: &web_sys::Node) {
    if let Some(sel) = web_sys::window().and_then(|w| w.get_selection().ok().flatten()) {
        let _ = sel.select_all_children(node);
    }
}

#[cfg(all(test, not(feature = "hydrate")))]
mod tests {
    use super::*;

    // The server never claims a copy or a share happened.
    #[tokio::test]
    async fn the_server_build_never_reports_success() {
        assert!(copy_text("ABCD-EFGH").await.is_err());
        assert!(share_url("Join", "/auth/register?invite=X").await.is_err());
        assert!(!can_share());
    }
}
