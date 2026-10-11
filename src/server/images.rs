//! Champion and item images, served by the app so the browser never calls
//! a third party.
//!
//! `GET /img/champion/{key}` (loading-screen portrait),
//! `GET /img/champion/{key}/icon` (square icon) and `GET /img/item/{id}`.
//! A champion key must resolve through the Data Dragon cache (id or name);
//! an item id must be digits. Anything else is 404, and nothing from the
//! URL reaches the filesystem or the upstream URL unchecked. Each image is
//! fetched once per patch version from Data Dragon and kept in memory
//! (bounded), then sent with long-lived cache headers and an ETag.
//!
//! The routes are public, so a client cannot turn them into a stream of
//! upstream requests or flush the cache: cache misses are rate limited per
//! client IP (`Bucket::ImageFetch`), keys Data Dragon does not have (a 403
//! or 404 upstream) live in their own bounded set, and a transient upstream
//! failure (timeout, 5xx) is answered with 502 and retried next time
//! instead of being remembered as missing for the whole patch.

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    body::Bytes,
    extract::{ConnectInfo, Path},
    http::{header, Extensions, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

use crate::server::data_dragon;
use crate::server::rate_limit::{Bucket, Limiter};

/// Images kept before the cache is cleared (about 170 champions with two
/// images each, plus the items of a patch, fit comfortably). Only images
/// Data Dragon really has land here, so a key scan cannot fill it.
const MAX_ENTRIES: usize = 1200;
/// Keys Data Dragon answered 403/404 for, kept so repeats are not refetched.
const MAX_MISSING: usize = 2000;
/// Browser cache lifetime. URLs carry `?v=<patch>`, so a new patch is a new URL.
const CACHE_CONTROL: &str = "public, max-age=604800";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Icon,
    Portrait,
    Item,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::Icon => "icon",
            Kind::Portrait => "portrait",
            Kind::Item => "item",
        }
    }
}

struct Entry {
    bytes: Bytes,
    content_type: &'static str,
    etag: String,
}

/// What one upstream request gave.
enum Fetched {
    Image(Bytes, &'static str),
    /// Data Dragon has no such image (403 or 404): remembered.
    Missing,
    /// Network error, timeout, 5xx or an empty body: not remembered.
    Failed,
}

fn cache() -> &'static Mutex<HashMap<String, Arc<Entry>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<Entry>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn missing() -> &'static Mutex<HashSet<String>> {
    static MISSING: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    MISSING.get_or_init(|| Mutex::new(HashSet::new()))
}

fn client_key(headers: &HeaderMap, ext: &Extensions) -> String {
    let peer = ext
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(a)| *a);
    crate::server::authz::client_ip_from(headers, peer)
}

/// Item ids are short digit strings.
fn valid_item_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 6 && id.bytes().all(|b| b.is_ascii_digit())
}

/// Champion ids from the cache are ASCII letters and digits; anything else
/// is refused before it can reach a URL.
fn valid_champion_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 40 && id.bytes().all(|b| b.is_ascii_alphanumeric())
}

fn upstream_url(kind: Kind, id: &str, version: &str) -> String {
    let cdn = data_dragon::CDN;
    match kind {
        Kind::Icon => format!("{cdn}/{version}/img/champion/{id}.png"),
        Kind::Portrait => format!("{cdn}/img/champion/loading/{id}_0.jpg"),
        Kind::Item => format!("{cdn}/{version}/img/item/{id}.png"),
    }
}

fn etag_for(kind: Kind, id: &str, version: &str, len: usize) -> String {
    format!("\"{version}-{}-{id}-{len}\"", kind.as_str())
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "Not found").into_response()
}

async fn serve(kind: Kind, id: String, headers: HeaderMap, ext: Extensions) -> Response {
    let version = data_dragon::version();
    let cache_key = format!("{version}/{}/{id}", kind.as_str());

    let cached = cache()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(&cache_key)
        .cloned();
    let entry = match cached {
        Some(e) => e,
        None => {
            if missing()
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .contains(&cache_key)
            {
                return not_found();
            }
            if let Err(wait) =
                Limiter::global().hit(Bucket::ImageFetch, &client_key(&headers, &ext))
            {
                let mut r = (StatusCode::TOO_MANY_REQUESTS, "Slow down").into_response();
                if let Ok(v) = HeaderValue::from_str(&wait.as_secs().max(1).to_string()) {
                    r.headers_mut().insert(header::RETRY_AFTER, v);
                }
                return r;
            }
            match fetch(kind, &id, &version).await {
                Fetched::Image(bytes, content_type) => {
                    let entry = Arc::new(Entry {
                        etag: etag_for(kind, &id, &version, bytes.len()),
                        bytes,
                        content_type,
                    });
                    let mut map = cache().lock().unwrap_or_else(|p| p.into_inner());
                    if map.len() >= MAX_ENTRIES {
                        map.clear();
                    }
                    map.insert(cache_key, Arc::clone(&entry));
                    entry
                }
                Fetched::Missing => {
                    let mut set = missing().lock().unwrap_or_else(|p| p.into_inner());
                    if set.len() >= MAX_MISSING {
                        set.clear();
                    }
                    set.insert(cache_key);
                    return not_found();
                }
                Fetched::Failed => {
                    return (StatusCode::BAD_GATEWAY, "Image temporarily unavailable")
                        .into_response();
                }
            }
        }
    };

    let Entry {
        bytes,
        content_type,
        etag,
    } = entry.as_ref();
    let matches = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.split(',').any(|t| t.trim() == etag || t.trim() == "*"));
    let mut resp = if matches {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        let mut r = bytes.clone().into_response();
        r.headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
        r
    };
    let h = resp.headers_mut();
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(CACHE_CONTROL),
    );
    if let Ok(v) = HeaderValue::from_str(etag) {
        h.insert(header::ETAG, v);
    }
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    resp
}

/// Fetch one image from Data Dragon (failures are logged).
async fn fetch(kind: Kind, id: &str, version: &str) -> Fetched {
    let url = upstream_url(kind, id, version);
    let resp = match data_dragon::http().get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(%url, "image fetch failed: {e}");
            return Fetched::Failed;
        }
    };
    let status = resp.status();
    if status == reqwest::StatusCode::NOT_FOUND || status == reqwest::StatusCode::FORBIDDEN {
        tracing::debug!(%url, %status, "image not available upstream");
        return Fetched::Missing;
    }
    if !status.is_success() {
        tracing::warn!(%url, %status, "image fetch failed upstream");
        return Fetched::Failed;
    }
    let content_type = match kind {
        Kind::Portrait => "image/jpeg",
        Kind::Icon | Kind::Item => "image/png",
    };
    match resp.bytes().await {
        Ok(b) if !b.is_empty() => Fetched::Image(b, content_type),
        Ok(_) => Fetched::Failed,
        Err(e) => {
            tracing::warn!(%url, "image body failed: {e}");
            Fetched::Failed
        }
    }
}

fn champion_id(key: &str) -> Option<String> {
    data_dragon::resolve_champion_id(key).filter(|id| valid_champion_id(id))
}

/// `GET /img/champion/{key}/icon`
pub async fn champion_icon(
    Path(key): Path<String>,
    headers: HeaderMap,
    ext: Extensions,
) -> Response {
    match champion_id(&key) {
        Some(id) => serve(Kind::Icon, id, headers, ext).await,
        None => not_found(),
    }
}

/// `GET /img/champion/{key}`
pub async fn champion_portrait(
    Path(key): Path<String>,
    headers: HeaderMap,
    ext: Extensions,
) -> Response {
    match champion_id(&key) {
        Some(id) => serve(Kind::Portrait, id, headers, ext).await,
        None => not_found(),
    }
}

/// `GET /img/item/{id}`
pub async fn item_icon(Path(id): Path<String>, headers: HeaderMap, ext: Extensions) -> Response {
    if valid_item_id(&id) {
        serve(Kind::Item, id, headers, ext).await
    } else {
        not_found()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_ids_are_short_digit_strings() {
        assert!(valid_item_id("3157"));
        assert!(!valid_item_id(""));
        assert!(!valid_item_id("1234567"));
        assert!(!valid_item_id("../etc"));
        assert!(!valid_item_id("31a7"));
    }

    #[test]
    fn champion_ids_are_alphanumeric() {
        assert!(valid_champion_id("KSante"));
        assert!(valid_champion_id("MonkeyKing"));
        assert!(!valid_champion_id("K'Sante"));
        assert!(!valid_champion_id("../x"));
        assert!(!valid_champion_id(""));
    }

    #[test]
    fn upstream_urls_are_data_dragon_only() {
        assert_eq!(
            upstream_url(Kind::Icon, "Ahri", "15.6.1"),
            "https://ddragon.leagueoflegends.com/cdn/15.6.1/img/champion/Ahri.png"
        );
        assert_eq!(
            upstream_url(Kind::Portrait, "Ahri", "15.6.1"),
            "https://ddragon.leagueoflegends.com/cdn/img/champion/loading/Ahri_0.jpg"
        );
        assert_eq!(
            upstream_url(Kind::Item, "3157", "15.6.1"),
            "https://ddragon.leagueoflegends.com/cdn/15.6.1/img/item/3157.png"
        );
    }

    #[test]
    fn etags_change_with_the_patch() {
        assert_ne!(
            etag_for(Kind::Icon, "Ahri", "15.6.1", 10),
            etag_for(Kind::Icon, "Ahri", "15.7.1", 10)
        );
        assert!(etag_for(Kind::Icon, "Ahri", "15.6.1", 10).starts_with('"'));
    }

    #[tokio::test]
    async fn unknown_keys_are_not_found_without_a_fetch() {
        let r = champion_icon(
            Path("../../etc/passwd".to_string()),
            HeaderMap::new(),
            Extensions::new(),
        )
        .await;
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
        let r = item_icon(Path("abc".to_string()), HeaderMap::new(), Extensions::new()).await;
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
    }
}
