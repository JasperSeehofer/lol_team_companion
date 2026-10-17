//! Link previews (M152 item 5): Open Graph and Twitter card tags.
//!
//! One static card per surface class: the landing (`/`, `/closed-beta`),
//! share links (`/s/{token}`) and everything else. The surface is chosen from
//! the path alone by `App` (outside every `Suspense`), so the tags never wait
//! for data and never depend on it. `OgMeta` takes no data at all: on a share
//! link the preview says what kind of page it is and nothing more; no token,
//! team or artefact name, the same for a valid, unknown, expired or revoked
//! link (R0 `gallery/og/NOTES.md`: chat apps cache previews beyond revocation).
//!
//! `leptos_meta` does not dedupe tags, so exactly one `OgMeta` is rendered per
//! page, by `App`; pages never add their own `og:*` tags.

use leptos::prelude::*;
use leptos_meta::Meta;

/// The public address of production, used when the request's host cannot be
/// read or is not a plain host name.
pub const FALLBACK_ORIGIN: &str = "https://app.team-companion.eu"; // lint:allow(external-url) our own public origin, not a third party

/// Which card a page gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OgSurface {
    /// The landing page and the closed-beta page.
    Landing,
    /// A share link: static, never anything about the shared item.
    Share,
    /// Every other page.
    Default,
}

impl OgSurface {
    /// The surface for a path, plus the canonical path written as `og:url`
    /// (only for the landing pages; share pages never get one).
    pub fn for_path(path: &str) -> (OgSurface, Option<&'static str>) {
        match path {
            "/" | "" => (OgSurface::Landing, Some("/")),
            "/closed-beta" | "/closed-beta/" => (OgSurface::Landing, Some("/closed-beta")),
            p if p.starts_with("/s/") => (OgSurface::Share, None),
            _ => (OgSurface::Default, None),
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            OgSurface::Landing => "Team Companion",
            OgSurface::Share => "Shared link · Team Companion",
            OgSurface::Default => "Team Companion",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            OgSurface::Landing => {
                "A private coaching workspace for one team: scouting, drafts, game plans, reviews and lessons in one loop."
            }
            OgSurface::Share => "A read-only page a team shared from Team Companion.",
            OgSurface::Default => "Scouting, drafts, game plans and reviews in one loop.",
        }
    }

    /// Path of the 1200x630 PNG under `public/`.
    pub const fn image_path(self) -> &'static str {
        match self {
            OgSurface::Landing => "/og/landing.png",
            OgSurface::Share => "/og/share.png",
            OgSurface::Default => "/og/default.png",
        }
    }

    pub const fn image_alt(self) -> &'static str {
        match self {
            // The card's own words (scripts/og/build.py writes the PNGs).
            OgSurface::Landing => {
                "Team Companion: scouting, drafts, game plans and reviews in one loop."
            }
            OgSurface::Default => "Team Companion: the coaching workspace for one team.",
            OgSurface::Share => "Team Companion: a read-only page shared by a team.",
        }
    }
}

/// The origin (`scheme://host[:port]`) for absolute `og:image` URLs, from the
/// request's `Host` header. There is no configured public URL: in production
/// Caddy only answers for `COMPANION_DOMAIN` and passes that host through, so
/// the header is the public host. Anything that is not a plain host name
/// (with an optional port) falls back to production. Loopback hosts (dev, CI)
/// are plain HTTP; everything else is HTTPS (Caddy terminates TLS, so the
/// app itself never sees the scheme).
pub fn origin_for_host(host: Option<&str>) -> String {
    let Some(host) = host.map(str::trim).filter(|h| valid_host(h)) else {
        return FALLBACK_ORIGIN.to_string();
    };
    let host = host.to_ascii_lowercase();
    let name = host.rsplit_once(':').map_or(host.as_str(), |(n, _)| n);
    let scheme = if matches!(name, "localhost" | "127.0.0.1" | "[::1]") {
        "http"
    } else {
        "https"
    };
    format!("{scheme}://{host}")
}

fn valid_host(host: &str) -> bool {
    if host.is_empty() || host.len() > 260 {
        return false;
    }
    let (name, port) = match host.rsplit_once(':') {
        Some((n, p)) if !n.ends_with(':') && n != "[" => (n, Some(p)),
        _ => (host, None),
    };
    if let Some(p) = port {
        if p.is_empty() || p.len() > 5 || !p.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
    }
    name == "[::1]"
        || (!name.is_empty()
            && !name.starts_with(['.', '-'])
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-'))
}

/// The origin of the current request (server) or page (browser).
fn current_origin() -> String {
    #[cfg(feature = "ssr")]
    {
        let host = use_context::<axum::http::request::Parts>().and_then(|p| {
            p.headers
                .get(axum::http::header::HOST)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
                .or_else(|| p.uri.authority().map(|a| a.as_str().to_owned()))
        });
        origin_for_host(host.as_deref())
    }
    #[cfg(all(feature = "hydrate", not(feature = "ssr")))]
    {
        web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_else(|| FALLBACK_ORIGIN.to_string())
    }
    #[cfg(not(any(feature = "ssr", feature = "hydrate")))]
    {
        FALLBACK_ORIGIN.to_string()
    }
}

/// Open Graph and Twitter card tags for one surface. Takes no page data on
/// purpose: what a preview shows can never depend on what a page loaded.
///
/// A11y: renders only `<meta>` tags in the head; nothing on screen.
#[component]
pub fn OgMeta(surface: OgSurface, canonical: Option<&'static str>) -> impl IntoView {
    let origin = current_origin();
    let image = format!("{origin}{}", surface.image_path());
    let url = canonical.map(|p| format!("{origin}{p}"));
    view! {
        <Meta property="og:site_name" content="Team Companion" />
        <Meta property="og:type" content="website" />
        <Meta property="og:title" content=surface.title() />
        <Meta property="og:description" content=surface.description() />
        <Meta property="og:image" content=image />
        <Meta property="og:image:type" content="image/png" />
        <Meta property="og:image:width" content="1200" />
        <Meta property="og:image:height" content="630" />
        <Meta property="og:image:alt" content=surface.image_alt() />
        {url.map(|u| view! { <Meta property="og:url" content=u /> })}
        <Meta name="twitter:card" content="summary_large_image" />
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn share_strings_are_static_and_name_nothing() {
        let s = OgSurface::Share;
        assert_eq!(s.title(), "Shared link · Team Companion");
        assert_eq!(
            s.description(),
            "A read-only page a team shared from Team Companion."
        );
        assert_eq!(s.image_path(), "/og/share.png");
        for text in [s.title(), s.description(), s.image_path(), s.image_alt()] {
            assert!(!text.contains('{') && !text.contains('?') && !text.contains("/s/"));
            assert!(!text.to_lowercase().contains("token"));
        }
    }

    #[test]
    fn every_share_path_maps_to_the_same_card_without_url() {
        for p in ["/s/abcdefghijkmnpqrstuvwxyz23456789", "/s/x", "/s/"] {
            assert_eq!(OgSurface::for_path(p), (OgSurface::Share, None));
        }
        assert_eq!(OgSurface::for_path("/"), (OgSurface::Landing, Some("/")));
        assert_eq!(
            OgSurface::for_path("/closed-beta"),
            (OgSurface::Landing, Some("/closed-beta"))
        );
        assert_eq!(OgSurface::for_path("/draft"), (OgSurface::Default, None));
    }

    #[test]
    fn origin_from_host() {
        assert_eq!(
            origin_for_host(Some("app.team-companion.eu")),
            FALLBACK_ORIGIN
        );
        assert_eq!(
            origin_for_host(Some("127.0.0.1:3020")),
            "http://127.0.0.1:3020" // lint:allow(external-url) loopback in a test
        );
        assert_eq!(
            origin_for_host(Some("localhost:3020")),
            "http://localhost:3020"
        );
        assert_eq!(origin_for_host(Some("[::1]:3020")), "http://[::1]:3020");
        assert_eq!(origin_for_host(None), FALLBACK_ORIGIN);
        for bad in [
            "",
            "evil.com/x",
            "a b",
            "x:99999999",
            "\"><script>",
            "user@host",
            "-x",
        ] {
            assert_eq!(origin_for_host(Some(bad)), FALLBACK_ORIGIN, "{bad}");
        }
    }

    #[test]
    fn images_are_pngs_in_public_og() {
        for s in [OgSurface::Landing, OgSurface::Share, OgSurface::Default] {
            let p = s.image_path();
            assert!(p.starts_with("/og/") && p.ends_with(".png"));
            let file = format!("{}/public{p}", env!("CARGO_MANIFEST_DIR"));
            let bytes = std::fs::read(&file).expect(&file);
            assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
            let w = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
            let h = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
            assert_eq!((w, h), (1200, 630), "{file}");
        }
    }
}
