//! Champion and item image URLs, and the live Data Dragon version.
//!
//! Images are served by the app itself (`server::images`: `/img/champion/..`
//! and `/img/item/..`), which fetches each one once per patch from Data
//! Dragon and caches it, so the browser never calls a third party. The
//! server keeps the current version in its Data Dragon cache
//! (`server::data_dragon`, refreshed every 6 hours) and writes it into the
//! page head as `<meta name="ddragon-version">`; the browser reads it back
//! from that tag and adds it to image URLs (`?v=`), so a new patch busts the
//! browser cache.

/// Used until a live (or cached) version is known.
pub const FALLBACK_VERSION: &str = "15.6.1";

/// `name` of the `<meta>` tag that carries the version to the browser.
pub const META_NAME: &str = "ddragon-version";

/// The current Data Dragon version.
pub fn version() -> String {
    #[cfg(feature = "ssr")]
    {
        crate::server::data_dragon::version()
    }
    #[cfg(not(feature = "ssr"))]
    {
        client_version()
    }
}

#[cfg(not(feature = "ssr"))]
fn client_version() -> String {
    thread_local! {
        static VERSION: std::cell::OnceCell<String> = const { std::cell::OnceCell::new() };
    }
    VERSION.with(|v| {
        v.get_or_init(|| {
            web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| {
                    d.query_selector(&format!("meta[name=\"{META_NAME}\"]"))
                        .ok()
                        .flatten()
                })
                .and_then(|m| m.get_attribute("content"))
                .filter(|c| !c.is_empty())
                .unwrap_or_else(|| FALLBACK_VERSION.to_string())
        })
        .clone()
    })
}

/// Percent-encode a champion id or name for a URL path segment.
fn path_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Square champion icon for a Data Dragon champion id ("KSante") or a
/// display name ("K'Sante"); the server resolves either.
pub fn champion_icon_url(champion: &str) -> String {
    format!(
        "/img/champion/{}/icon?v={}",
        path_segment(champion),
        version()
    )
}

/// Loading-screen portrait (tall) for a champion id or name.
pub fn champion_portrait_url(champion: &str) -> String {
    format!("/img/champion/{}?v={}", path_segment(champion), version())
}

/// Item icon for an item ID.
pub fn item_icon_url(item_id: i32) -> String {
    format!("/img/item/{item_id}?v={}", version())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_point_at_the_app_and_carry_the_version() {
        let v = version();
        assert_eq!(
            champion_icon_url("Ahri"),
            format!("/img/champion/Ahri/icon?v={v}")
        );
        assert_eq!(
            champion_portrait_url("Kai'Sa"),
            format!("/img/champion/Kai%27Sa?v={v}")
        );
        assert_eq!(item_icon_url(3157), format!("/img/item/3157?v={v}"));
        assert!(!champion_icon_url("Ahri").contains("://"));
    }
}
