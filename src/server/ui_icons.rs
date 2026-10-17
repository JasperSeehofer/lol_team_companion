//! Interface icons (lane positions, objectives, structures), served by the
//! app so the browser never calls a third party (M152).
//!
//! `GET /img/ui/{kind}/{name}` answers only for the pairs in [`ICONS`], a
//! hard-coded allow-list; anything else is 404. The request text is only
//! compared with those constants: the upstream URL and the committed copy
//! are both taken from the matching entry, never built from the URL.
//!
//! - **Position icons (SVG)** are always the committed, sanitised copy in
//!   `assets/ui/position/` (made with `scripts/ui-icons/sanitise.py`: only
//!   `svg/g/path/circle/rect`, every fill `currentColor`). They are never
//!   proxied, so no upstream markup reaches a browser. They carry
//!   `Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'; sandbox`.
//! - **Objective and structure icons (PNG)** are fetched once per process
//!   from CommunityDragon's pinned patch directory ([`CDRAGON_PATCH`]) and
//!   kept in memory. When the fetch fails or times out, the committed copy
//!   under `assets/ui/` is served instead, and upstream is tried again after
//!   [`RETRY_AFTER`]. An upstream body must look like a PNG and stay under
//!   [`MAX_BYTES`], or the committed copy is used.
//!
//! Every response has an ETag (a hash of the bytes), honours
//! `If-None-Match` with 304, and is sent with
//! `Cache-Control: public, max-age=604800` and `X-Content-Type-Options: nosniff`.
//! The allow-list holds 16 PNGs, so a client can cause at most 16 upstream
//! requests per [`RETRY_AFTER`]; no rate limit is needed. Paths, patch and
//! hashes are recorded in `docs/ASSETS.md`.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::{
    body::Bytes,
    extract::Path,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use sha2::{Digest, Sha256};

/// The CommunityDragon patch directory the icons are pinned to (its
/// `content-metadata.json` read `16.20.8248524` on 2026-10-17). Change it
/// together with the committed copies and `docs/ASSETS.md`.
pub const CDRAGON_PATCH: &str = "16.20";
const CDRAGON: &str = "https://raw.communitydragon.org";
const MINIMAP: &str = "game/assets/ux/minimap/icons";

const CACHE_CONTROL: &str = "public, max-age=604800";
const SVG_CSP: &str = "default-src 'none'; style-src 'unsafe-inline'; sandbox";
/// How long a fallback copy is served before upstream is asked again.
pub const RETRY_AFTER: Duration = Duration::from_secs(15 * 60);
/// Upstream bodies larger than this are refused (the icons are 1 to 10 KB).
const MAX_BYTES: usize = 256 * 1024;
const FETCH_TIMEOUT: Duration = Duration::from_secs(5);

/// One allowed icon.
pub struct UiIcon {
    pub kind: &'static str,
    pub name: &'static str,
    /// File in CommunityDragon's pinned patch directory; `None` = served
    /// only from the committed copy (the sanitised SVGs).
    upstream: Option<&'static str>,
    /// The committed copy under `assets/ui/`.
    fallback: &'static [u8],
    content_type: &'static str,
}

macro_rules! svg {
    ($name:literal) => {
        UiIcon {
            kind: "position",
            name: $name,
            upstream: None,
            fallback: include_bytes!(concat!("../../assets/ui/position/", $name, ".svg")),
            content_type: "image/svg+xml",
        }
    };
}

macro_rules! png {
    ($kind:literal, $name:literal, $file:literal) => {
        UiIcon {
            kind: $kind,
            name: $name,
            upstream: Some($file),
            fallback: include_bytes!(concat!("../../assets/ui/", $kind, "/", $name, ".png")),
            content_type: "image/png",
        }
    };
}

/// The allow-list: every icon the route serves.
pub static ICONS: &[UiIcon] = &[
    svg!("top"),
    svg!("jungle"),
    svg!("middle"),
    svg!("bottom"),
    svg!("utility"),
    png!("objective", "baron", "baron.png"),
    png!("objective", "riftherald", "riftherald.png"),
    png!("objective", "grub", "grub.png"),
    png!("objective", "dragon", "dragon.png"),
    png!("objective", "dragon_cloud", "dragon_cloud.png"),
    png!("objective", "dragon_infernal", "dragon_infernal.png"),
    png!("objective", "dragon_mountain", "dragon_mountain.png"),
    png!("objective", "dragon_ocean", "dragon_ocean.png"),
    png!("objective", "dragon_hextech", "dragon_hextech.png"),
    png!("objective", "dragon_chemtech", "dragon_chemtech.png"),
    png!("objective", "dragon_elder", "dragon_elder.png"),
    png!("objective", "atakhan_r", "atakhan_r.png"),
    png!("objective", "atakhan_v", "atakhan_v.png"),
    png!("structure", "tower", "icon_ui_tower_minimap.png"),
    png!("structure", "inhibitor", "icon_ui_inhibitor_minimap_v2.png"),
    png!("structure", "nexus", "icon_ui_nexus_minimap_v2.png"),
];

/// The allowed icon for `kind`/`name`, compared byte for byte.
pub fn lookup(kind: &str, name: &str) -> Option<&'static UiIcon> {
    ICONS.iter().find(|i| i.kind == kind && i.name == name)
}

/// The pinned upstream URL, built from allow-list constants only.
fn upstream_url(icon: &UiIcon) -> Option<String> {
    icon.upstream
        .map(|file| format!("{CDRAGON}/{CDRAGON_PATCH}/{MINIMAP}/{file}"))
}

struct Cached {
    bytes: Bytes,
    etag: String,
    /// The committed copy, served because upstream failed.
    fallback_since: Option<Instant>,
}

type Cache = Mutex<HashMap<&'static str, Cached>>;

fn cache() -> &'static Cache {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn etag_for(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let hex: String = digest.iter().take(8).map(|b| format!("{b:02x}")).collect();
    format!("\"ui-{CDRAGON_PATCH}-{hex}\"")
}

fn looks_like_png(b: &[u8]) -> bool {
    b.len() <= MAX_BYTES && b.starts_with(b"\x89PNG\r\n\x1a\n")
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "Not found").into_response()
}

/// The bytes and ETag to send for `icon`, fetching upstream when needed.
async fn resolve<F, Fut>(icon: &'static UiIcon, cache: &Cache, fetch: F) -> (Bytes, String)
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Option<Bytes>>,
{
    // One entry per allow-list item (names are unique, a test checks it);
    // the patch is a constant.
    let key = icon.name;
    let Some(url) = upstream_url(icon) else {
        let bytes = Bytes::from_static(icon.fallback);
        let etag = etag_for(&bytes);
        return (bytes, etag);
    };
    {
        let map = cache.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(c) = map.get(key) {
            let fresh = c
                .fallback_since
                .is_none_or(|since| since.elapsed() < RETRY_AFTER);
            if fresh {
                return (c.bytes.clone(), c.etag.clone());
            }
        }
    }
    let entry = match fetch(url).await {
        Some(b) if looks_like_png(&b) => Cached {
            etag: etag_for(&b),
            bytes: b,
            fallback_since: None,
        },
        _ => {
            let bytes = Bytes::from_static(icon.fallback);
            Cached {
                etag: etag_for(&bytes),
                bytes,
                fallback_since: Some(Instant::now()),
            }
        }
    };
    let out = (entry.bytes.clone(), entry.etag.clone());
    cache
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(key, entry);
    out
}

async fn serve_with<F, Fut>(
    kind: &str,
    name: &str,
    headers: &HeaderMap,
    cache: &Cache,
    fetch: F,
) -> Response
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Option<Bytes>>,
{
    let Some(icon) = lookup(kind, name) else {
        return not_found();
    };
    let (bytes, etag) = resolve(icon, cache, fetch).await;
    let matches = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.split(',').any(|t| t.trim() == etag || t.trim() == "*"));
    let mut resp = if matches {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        let mut r = bytes.into_response();
        r.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(icon.content_type),
        );
        r
    };
    let h = resp.headers_mut();
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(CACHE_CONTROL),
    );
    if let Ok(v) = HeaderValue::from_str(&etag) {
        h.insert(header::ETAG, v);
    }
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    if icon.upstream.is_none() {
        h.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(SVG_CSP),
        );
    }
    resp
}

/// Fetch one PNG from CommunityDragon (failures are logged, never cached
/// as missing: the committed copy covers them).
async fn fetch_upstream(url: String) -> Option<Bytes> {
    let resp = match crate::server::data_dragon::http()
        .get(&url)
        .timeout(FETCH_TIMEOUT)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(%url, "ui icon fetch failed, serving the committed copy: {e}");
            return None;
        }
    };
    if !resp.status().is_success() {
        tracing::warn!(%url, status = %resp.status(), "ui icon fetch failed, serving the committed copy");
        return None;
    }
    match resp.bytes().await {
        Ok(b) => Some(b),
        Err(e) => {
            tracing::warn!(%url, "ui icon body failed, serving the committed copy: {e}");
            None
        }
    }
}

/// `GET /img/ui/{kind}/{name}`
pub async fn ui_icon(Path((kind, name)): Path<(String, String)>, headers: HeaderMap) -> Response {
    serve_with(&kind, &name, &headers, cache(), fetch_upstream).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn fresh_cache() -> Cache {
        Mutex::new(HashMap::new())
    }

    async fn body(r: Response) -> Vec<u8> {
        axum::body::to_bytes(r.into_body(), 1 << 20)
            .await
            .expect("body")
            .to_vec()
    }

    async fn failing(_: String) -> Option<Bytes> {
        None
    }

    #[test]
    fn allow_list_is_complete_and_unique() {
        assert_eq!(ICONS.len(), 21);
        for (i, a) in ICONS.iter().enumerate() {
            for b in &ICONS[i + 1..] {
                assert!(!(a.kind == b.kind && a.name == b.name), "{}", a.name);
                assert_ne!(a.name, b.name, "cache keys are names");
            }
            assert!(!a.fallback.is_empty(), "{}", a.name);
            if a.content_type == "image/png" {
                assert!(looks_like_png(a.fallback), "{}", a.name);
            }
        }
    }

    #[test]
    fn unknown_and_traversal_shaped_names_are_refused() {
        assert!(lookup("position", "top").is_some());
        assert!(lookup("structure", "nexus").is_some());
        for (k, n) in [
            ("position", "Top"),
            ("position", "top.svg"),
            ("position", "../top"),
            ("position", "..%2ftop"),
            ("position", "..%2Ftop"),
            ("position", "%2e%2e%2ftop"),
            ("position", "..\\top"),
            ("position", "top/../../etc/passwd"),
            ("position", ""),
            ("objective", "top"),
            ("champion", "baron"),
            ("..", "top"),
            ("objective", "baron "),
            ("objective", "baron\0"),
        ] {
            assert!(lookup(k, n).is_none(), "{k}/{n}");
        }
    }

    #[test]
    fn upstream_urls_are_pinned_constants() {
        assert_eq!(
            upstream_url(lookup("structure", "tower").unwrap()).as_deref(),
            Some("https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/icon_ui_tower_minimap.png")
        );
        assert!(upstream_url(lookup("position", "jungle").unwrap()).is_none());
        for i in ICONS {
            if let Some(u) = upstream_url(i) {
                assert!(
                    u.starts_with("https://raw.communitydragon.org/16.20/"),
                    "{u}"
                );
                assert!(!u.contains("latest"));
            }
        }
    }

    /// `public/` is the static site root, served without the icon route's
    /// sandbox CSP and `nosniff`. The committed copies must stay outside it,
    /// so `/ui/...` is a 404 and `/img/ui/...` is the only way to them.
    #[test]
    fn committed_copies_are_not_in_the_static_site_root() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(
            !root.join("public/ui").exists(),
            "public/ui/ would be served as plain static files; keep the icons in assets/ui/"
        );
        for i in ICONS {
            let ext = if i.content_type == "image/svg+xml" {
                "svg"
            } else {
                "png"
            };
            let path = root.join(format!("assets/ui/{}/{}.{ext}", i.kind, i.name));
            assert!(path.is_file(), "{} missing", path.display());
        }
    }

    #[test]
    fn committed_svgs_are_sanitised() {
        for i in ICONS.iter().filter(|i| i.content_type == "image/svg+xml") {
            let s = std::str::from_utf8(i.fallback)
                .expect("utf8")
                .to_lowercase();
            assert!(s.starts_with("<svg"), "{}", i.name);
            for bad in [
                "<script",
                "href",
                "style",
                "foreignobject",
                "<?xml-stylesheet",
                " on",
                " id=",
                " class=",
                "url(",
                "#c8aa6e",
            ] {
                assert!(!s.contains(bad), "{} contains {bad}", i.name);
            }
            assert!(s.contains("currentcolor"), "{}", i.name);
        }
    }

    #[tokio::test]
    async fn unknown_pairs_are_404_without_a_fetch() {
        let calls = AtomicUsize::new(0);
        let r = serve_with(
            "position",
            "../top",
            &HeaderMap::new(),
            &fresh_cache(),
            |_| async {
                calls.fetch_add(1, Ordering::SeqCst);
                None
            },
        )
        .await;
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
        let r = ui_icon(
            Path(("position".to_string(), "..%2ftop".to_string())),
            HeaderMap::new(),
        )
        .await;
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn failed_upstream_serves_the_committed_copy() {
        let cache = fresh_cache();
        let r = serve_with("objective", "baron", &HeaderMap::new(), &cache, failing).await;
        assert_eq!(r.status(), StatusCode::OK);
        assert_eq!(r.headers()[header::CONTENT_TYPE], "image/png");
        assert_eq!(r.headers()[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
        assert_eq!(r.headers()[header::CACHE_CONTROL], CACHE_CONTROL);
        let expected = lookup("objective", "baron").unwrap().fallback;
        assert_eq!(body(r).await, expected);

        // A body that is not a PNG is refused in favour of the copy too.
        let cache = fresh_cache();
        let r = serve_with("objective", "grub", &HeaderMap::new(), &cache, |_| async {
            Some(Bytes::from_static(b"<html>error</html>"))
        })
        .await;
        assert_eq!(body(r).await, lookup("objective", "grub").unwrap().fallback);
    }

    #[tokio::test]
    async fn a_good_upstream_body_is_fetched_once_and_cached() {
        let cache = fresh_cache();
        let calls = AtomicUsize::new(0);
        let png = Bytes::from_static(b"\x89PNG\r\n\x1a\nupstream");
        for _ in 0..3 {
            let served = png.clone();
            let r = serve_with("structure", "tower", &HeaderMap::new(), &cache, |url| {
                calls.fetch_add(1, Ordering::SeqCst);
                assert!(url.ends_with("/icon_ui_tower_minimap.png"));
                async move { Some(served) }
            })
            .await;
            assert_eq!(body(r).await, png.to_vec());
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn etag_answers_304() {
        let cache = fresh_cache();
        let r = serve_with("position", "top", &HeaderMap::new(), &cache, failing).await;
        assert_eq!(r.status(), StatusCode::OK);
        assert_eq!(r.headers()[header::CONTENT_TYPE], "image/svg+xml");
        assert_eq!(r.headers()[header::CONTENT_SECURITY_POLICY], SVG_CSP);
        assert_eq!(r.headers()[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
        let etag = r.headers()[header::ETAG].clone();
        assert!(etag.to_str().unwrap().starts_with("\"ui-16.20-"));

        let mut h = HeaderMap::new();
        h.insert(header::IF_NONE_MATCH, etag.clone());
        let r = serve_with("position", "top", &h, &cache, failing).await;
        assert_eq!(r.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(r.headers()[header::ETAG], etag);
        assert!(body(r).await.is_empty());

        let mut h = HeaderMap::new();
        h.insert(header::IF_NONE_MATCH, HeaderValue::from_static("\"other\""));
        let r = serve_with("position", "top", &h, &cache, failing).await;
        assert_eq!(r.status(), StatusCode::OK);
    }
}
