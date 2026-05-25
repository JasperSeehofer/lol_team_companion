//! Per-request theme injection middleware (Phase 18.1).
//!
//! Closes UI-18-RUNTIME-01 — the architectural gap where `src/main.rs`
//! provided `InitialTheme::default()` unconditionally for every SSR
//! request, so the Phase 18 `region` branching never activated at
//! runtime even though it compiled and unit-tested correctly.
//!
//! Design (locked in `.planning/phases/18.1-.../18.1-CONTEXT.md`):
//! - D-01: cookie `lol_companion_theme=<demacia|pandemonium>; Path=/;
//!   Max-Age=31536000; SameSite=Lax` (NOT `HttpOnly` — the client
//!   reads it for the optimistic update after a toggle click).
//! - D-02: precedence is cookie > `AuthSession.user.theme` > `"demacia"`.
//! - D-03: layer runs AFTER `auth_layer` (so `AuthSession` is in
//!   request extensions) and BEFORE the leptos routes context closure.
//! - D-05: the cookie value MUST validate against the
//!   `{"demacia", "pandemonium"}` allowlist before injection — invalid
//!   values are treated as absent.
//!
//! The middleware sets a `tokio::task_local!` `REQUEST_THEME` to the
//! resolved `InitialTheme` for the duration of `next.run(req)`; the
//! leptos context closure reads it synchronously via
//! `REQUEST_THEME.try_with(|t| t.clone()).unwrap_or_default()`.

use crate::app::InitialTheme;
use crate::server::auth::AuthSession;
use axum::{extract::Request, http::header::COOKIE, middleware::Next, response::Response};

tokio::task_local! {
    /// Per-request theme value set by `theme_injection_middleware` so
    /// the synchronous `leptos_routes_with_context` closure can read
    /// it without an async extractor.
    pub static REQUEST_THEME: InitialTheme;
}

/// The two locked allowlist values (D-05). Anything else is treated
/// as a missing cookie (defense against trivial tampering / XSS via
/// the `<html data-theme="...">` attribute).
const VALID_THEMES: [&str; 2] = ["demacia", "pandemonium"];

/// Parse a `Cookie:` header value and return the validated
/// `lol_companion_theme` value if one is present and is in the
/// allowlist.
///
/// Returns `None` for:
/// - missing key
/// - empty header
/// - value not in `{"demacia", "pandemonium"}` (D-05)
pub fn parse_theme_cookie(_cookie_header: &str) -> Option<String> {
    // RED stub — to be implemented in GREEN
    None
}

/// Pure resolver for theme precedence (D-02): cookie wins over
/// authenticated user's account theme, which wins over the default
/// `"demacia"`.
///
/// The cookie value is assumed to be pre-validated by
/// `parse_theme_cookie`. The `auth_theme` value comes from
/// `AppUser.theme` which is constrained at the DB schema layer
/// (`ASSERT $value IN ['demacia','pandemonium']`).
pub fn resolve_theme(_cookie: Option<String>, _auth_theme: Option<String>) -> String {
    // RED stub — to be implemented in GREEN
    String::new()
}

/// Axum middleware function: reads the cookie + `AuthSession` from
/// the request, resolves the theme, sets the `REQUEST_THEME`
/// task-local for the duration of the inner service call, and
/// forwards the request unchanged.
///
/// Use via `axum::middleware::from_fn(theme_injection_middleware)`
/// in the router build. Placement: AFTER `.layer(auth_layer)` in
/// builder order so `auth_layer` runs first on each request and
/// `AuthSession` is populated in extensions when this layer reads it.
pub async fn theme_injection_middleware(req: Request, next: Next) -> Response {
    // 1. Pull the Cookie header (case-insensitive header name lookup).
    let cookie_value = req
        .headers()
        .get(COOKIE)
        .and_then(|hv| hv.to_str().ok())
        .and_then(parse_theme_cookie);

    // 2. Pull AuthSession from request extensions (populated by
    //    axum_login's auth_layer running before us — D-03).
    let auth_theme = req
        .extensions()
        .get::<AuthSession>()
        .and_then(|s| s.user.as_ref().map(|u| u.theme.clone()));

    // 3. Resolve precedence (D-02).
    let resolved = resolve_theme(cookie_value, auth_theme);

    // 4. Set the per-request task-local for the duration of the
    //    downstream service call so the leptos context closure can
    //    read it synchronously.
    REQUEST_THEME
        .scope(InitialTheme(resolved), next.run(req))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1 — single cookie, valid value
    #[test]
    fn parse_theme_cookie_single_demacia() {
        assert_eq!(
            parse_theme_cookie("lol_companion_theme=demacia"),
            Some("demacia".to_string())
        );
    }

    // Test 2 — multi-cookie header, middle position
    #[test]
    fn parse_theme_cookie_multi_pandemonium() {
        assert_eq!(
            parse_theme_cookie("a=1; lol_companion_theme=pandemonium; b=2"),
            Some("pandemonium".to_string())
        );
    }

    // Test 3 — invalid value rejected per D-05
    #[test]
    fn parse_theme_cookie_rejects_garbage() {
        assert_eq!(parse_theme_cookie("lol_companion_theme=garbage"), None);
    }

    // Test 4 — missing key
    #[test]
    fn parse_theme_cookie_missing_key() {
        assert_eq!(parse_theme_cookie("other=stuff"), None);
    }

    // Test 5 — empty header
    #[test]
    fn parse_theme_cookie_empty() {
        assert_eq!(parse_theme_cookie(""), None);
    }

    // Test 6 — cookie wins over auth (D-02)
    #[test]
    fn resolve_theme_cookie_wins_over_auth() {
        assert_eq!(
            resolve_theme(Some("pandemonium".to_string()), Some("demacia".to_string())),
            "pandemonium"
        );
    }

    // Test 7 — auth fallback when no cookie
    #[test]
    fn resolve_theme_auth_fallback() {
        assert_eq!(
            resolve_theme(None, Some("pandemonium".to_string())),
            "pandemonium"
        );
    }

    // Test 8 — final default when neither cookie nor auth (D-06)
    #[test]
    fn resolve_theme_default() {
        assert_eq!(resolve_theme(None, None), "demacia");
    }

    // Test 9 — cookie overrides authenticated user's account theme
    #[test]
    fn resolve_theme_cookie_overrides_auth_demacia_over_pandemonium() {
        assert_eq!(
            resolve_theme(Some("demacia".to_string()), Some("pandemonium".to_string())),
            "demacia"
        );
    }

    // Extra: XSS-style payload is rejected (defensive; D-05).
    #[test]
    fn parse_theme_cookie_rejects_xss_string() {
        assert_eq!(
            parse_theme_cookie("lol_companion_theme=<script>alert(1)</script>"),
            None
        );
    }
}
