//! Data Dragon image URLs built from the live patch version.
//!
//! The server keeps the current version in its Data Dragon cache
//! (`server::data_dragon`, refreshed every 6 hours) and writes it into the
//! page head as `<meta name="ddragon-version">`. In the browser the hydrated
//! components read it back from that tag, so server and client build the
//! same URLs.

/// Used until a live (or cached) version is known.
pub const FALLBACK_VERSION: &str = "15.6.1";

/// `name` of the `<meta>` tag that carries the version to the browser.
pub const META_NAME: &str = "ddragon-version";

const CDN: &str = "https://ddragon.leagueoflegends.com/cdn";

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

/// Square champion icon for a Data Dragon champion ID ("KSante").
pub fn champion_icon_url(champion_id: &str) -> String {
    format!("{CDN}/{}/img/champion/{champion_id}.png", version())
}

/// Item icon for an item ID.
pub fn item_icon_url(item_id: i32) -> String {
    format!("{CDN}/{}/img/item/{item_id}.png", version())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_use_the_current_version() {
        let v = version();
        assert_eq!(
            champion_icon_url("Ahri"),
            format!("https://ddragon.leagueoflegends.com/cdn/{v}/img/champion/Ahri.png")
        );
        assert!(item_icon_url(3157).ends_with(&format!("/{v}/img/item/3157.png")));
    }
}
