use crate::models::champion::Champion;
use thiserror::Error;

/// Normalize a champion name input to its canonical Data Dragon ID.
///
/// Lookup order:
/// 1. Exact match on `c.id == input`
/// 2. Case-insensitive display name match: `c.name.to_lowercase() == input.to_lowercase()`
/// 3. Fuzzy match by stripping non-alphanumeric chars and lowercasing both sides
///
/// Returns `Some(canonical_id)` or `None` if no champion matches.
#[cfg(feature = "ssr")]
pub fn normalize_champion_name(input: &str, champions: &[Champion]) -> Option<String> {
    // Pass 1: exact ID match
    if let Some(c) = champions.iter().find(|c| c.id == input) {
        return Some(c.id.clone());
    }

    // Pass 2: case-insensitive display name match
    let input_lower = input.to_lowercase();
    if let Some(c) = champions
        .iter()
        .find(|c| c.name.to_lowercase() == input_lower)
    {
        return Some(c.id.clone());
    }

    // Pass 3: fuzzy match — strip non-alphanumeric chars and lowercase both
    let input_stripped: String = input
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase();
    if input_stripped.is_empty() {
        return None;
    }
    let stripped = |s: &str| -> String {
        s.chars()
            .filter(|ch| ch.is_alphanumeric())
            .collect::<String>()
            .to_lowercase()
    };
    // Ids first, then display names ("NunuWillump" -> "Nunu & Willump",
    // whose id is "Nunu").
    champions
        .iter()
        .find(|c| stripped(&c.id) == input_stripped)
        .or_else(|| {
            champions
                .iter()
                .find(|c| stripped(&c.name) == input_stripped)
        })
        .map(|c| c.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_champions() -> Vec<Champion> {
        vec![
            Champion {
                id: "KSante".into(),
                name: "K'Sante".into(),
                title: "The Pride of Nazumah".into(),
                tags: vec!["Tank".into()],
                image_full: "KSante.png".into(),
            },
            Champion {
                id: "AurelionSol".into(),
                name: "Aurelion Sol".into(),
                title: "The Star Forger".into(),
                tags: vec!["Mage".into()],
                image_full: "AurelionSol.png".into(),
            },
            Champion {
                id: "Jinx".into(),
                name: "Jinx".into(),
                title: "The Loose Cannon".into(),
                tags: vec!["Marksman".into()],
                image_full: "Jinx.png".into(),
            },
            Champion {
                id: "Belveth".into(),
                name: "Bel'Veth".into(),
                title: "The Empress of the Void".into(),
                tags: vec!["Fighter".into()],
                image_full: "Belveth.png".into(),
            },
            Champion {
                id: "Nunu".into(),
                name: "Nunu & Willump".into(),
                title: "The Boy and His Yeti".into(),
                tags: vec!["Tank".into()],
                image_full: "Nunu.png".into(),
            },
        ]
    }

    #[test]
    fn stripped_display_names_resolve() {
        // What the legacy ChampPortrait shim sends (spaces and "&" removed).
        let champs = sample_champions();
        assert_eq!(
            normalize_champion_name("NunuWillump", &champs),
            Some("Nunu".into())
        );
        assert_eq!(normalize_champion_name("NotAChampion", &champs), None);
    }

    #[test]
    fn test_normalize_champion_name() {
        let champs = sample_champions();

        // exact ID match
        assert_eq!(
            normalize_champion_name("KSante", &champs),
            Some("KSante".into())
        );
        assert_eq!(
            normalize_champion_name("Jinx", &champs),
            Some("Jinx".into())
        );

        // case-insensitive display name match
        assert_eq!(
            normalize_champion_name("K'Sante", &champs),
            Some("KSante".into())
        );
        assert_eq!(
            normalize_champion_name("aurelion sol", &champs),
            Some("AurelionSol".into())
        );
        assert_eq!(
            normalize_champion_name("JINX", &champs),
            Some("Jinx".into())
        );

        // fuzzy stripped lowercase match
        assert_eq!(
            normalize_champion_name("ksante", &champs),
            Some("KSante".into())
        );
        assert_eq!(
            normalize_champion_name("bel'veth", &champs),
            Some("Belveth".into())
        );

        // no match
        assert_eq!(normalize_champion_name("NotAChampion", &champs), None);
        assert_eq!(normalize_champion_name("", &champs), None);
    }
}

#[derive(Debug, Error)]
pub enum DataDragonError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

// ---------------------------------------------------------------------------
// Live version and champion cache
// ---------------------------------------------------------------------------
//
// The champion list, the key -> champion map and the current patch version
// live in memory (`STATE`). `init` loads the last good copy from the DB (so a
// cold start without network still has champion data), then refreshes from
// Data Dragon; `spawn_refresh_task` refreshes every 6 hours. Each successful
// refresh is persisted, and a new patch version writes a `patch_change` row.
// The public champion-list server functions read this cache instead of
// calling Data Dragon per request.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use surrealdb::{engine::local::Db, Surreal};

const VERSIONS_URL: &str = "https://ddragon.leagueoflegends.com/api/versions.json";
pub(crate) const CDN: &str = "https://ddragon.leagueoflegends.com/cdn";

/// How often the background task refreshes the cache.
pub const REFRESH_EVERY: Duration = Duration::from_secs(6 * 3600);

/// A loaded copy of Data Dragon.
#[derive(Debug, Clone)]
pub struct DdSnapshot {
    pub version: String,
    pub champions: Vec<Champion>,
    /// Numeric champion key (as Riot APIs report champion IDs) -> Data
    /// Dragon champion ID ("266" -> "Aatrox").
    pub id_by_key: HashMap<i64, String>,
}

/// One champion as stored in the DB copy (the shared model plus its key).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CachedChampion {
    key: i64,
    #[serde(flatten)]
    champion: Champion,
}

fn state() -> &'static RwLock<Option<Arc<DdSnapshot>>> {
    static STATE: std::sync::OnceLock<RwLock<Option<Arc<DdSnapshot>>>> = std::sync::OnceLock::new();
    STATE.get_or_init(|| RwLock::new(None))
}

fn set_snapshot(mut s: DdSnapshot) {
    // Image URLs always point at the app's own endpoint (older DB copies
    // stored CDN URLs).
    for c in &mut s.champions {
        c.image_full = app_icon_url(&c.id, &s.version);
    }
    *state().write().unwrap_or_else(|p| p.into_inner()) = Some(Arc::new(s));
}

/// The app's square-icon URL for a champion id at a version.
fn app_icon_url(id: &str, version: &str) -> String {
    format!("/img/champion/{id}/icon?v={version}")
}

/// Resolve a champion id ("KSante") or display name ("K'Sante", "ksante")
/// to its Data Dragon id, using the cache. `None` when unknown or when no
/// copy is loaded.
pub fn resolve_champion_id(key_or_name: &str) -> Option<String> {
    let snap = snapshot()?;
    normalize_champion_name(key_or_name, &snap.champions)
}

/// The cached snapshot, if any has been loaded.
pub fn snapshot() -> Option<Arc<DdSnapshot>> {
    state().read().unwrap_or_else(|p| p.into_inner()).clone()
}

/// The current patch version, or the built-in fallback before any copy is
/// loaded.
pub fn version() -> String {
    snapshot()
        .map(|s| s.version.clone())
        .unwrap_or_else(|| crate::models::ddragon::FALLBACK_VERSION.to_string())
}

/// The Data Dragon champion ID for a numeric champion key, when known.
pub fn champion_id_for_key(key: i64) -> Option<String> {
    snapshot().and_then(|s| s.id_by_key.get(&key).cloned())
}

/// The champion list. Served from the cache; when nothing is cached yet
/// (no DB copy and Data Dragon was unreachable at startup) it tries one
/// fetch, without persisting it.
pub async fn champions() -> Result<Vec<Champion>, DataDragonError> {
    if let Some(s) = snapshot() {
        return Ok(s.champions.clone());
    }
    let fresh = fetch_snapshot().await?;
    let champions = fresh.champions.clone();
    set_snapshot(fresh);
    Ok(champions)
}

pub(crate) fn http() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default()
    })
}

async fn fetch_latest_version() -> Result<String, DataDragonError> {
    let versions: Vec<String> = http()
        .get(VERSIONS_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    versions
        .into_iter()
        .next()
        .ok_or_else(|| DataDragonError::Parse("Empty versions list".into()))
}

/// Parse a `champion.json` document for `version` into a snapshot.
pub fn parse_champion_json(
    version: &str,
    doc: &serde_json::Value,
) -> Result<DdSnapshot, DataDragonError> {
    let data = doc
        .get("data")
        .and_then(|d| d.as_object())
        .ok_or_else(|| DataDragonError::Parse("Missing 'data' field".into()))?;

    let mut champions = Vec::with_capacity(data.len());
    let mut id_by_key = HashMap::with_capacity(data.len());
    for v in data.values() {
        let parsed = (|| {
            let id = v.get("id")?.as_str()?.to_string();
            let key = v.get("key")?.as_str()?.parse::<i64>().ok()?;
            let name = v.get("name")?.as_str()?.to_string();
            let title = v.get("title")?.as_str()?.to_string();
            let tags = v
                .get("tags")?
                .as_array()?
                .iter()
                .filter_map(|t| t.as_str().map(|s| s.to_string()))
                .collect();
            // Every champion has an image entry; the app serves it itself.
            v.get("image")?.get("full")?.as_str()?;
            Some((
                key,
                Champion {
                    image_full: app_icon_url(&id, version),
                    id,
                    name,
                    title,
                    tags,
                },
            ))
        })();
        if let Some((key, champion)) = parsed {
            id_by_key.insert(key, champion.id.clone());
            champions.push(champion);
        }
    }
    if champions.is_empty() {
        return Err(DataDragonError::Parse(
            "No champions in champion.json".into(),
        ));
    }
    champions.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(DdSnapshot {
        version: version.to_string(),
        champions,
        id_by_key,
    })
}

async fn fetch_snapshot() -> Result<DdSnapshot, DataDragonError> {
    let version = fetch_latest_version().await?;
    let url = format!("{CDN}/{version}/data/en_US/champion.json");
    let doc: serde_json::Value = http()
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    parse_champion_json(&version, &doc)
}

fn to_cache_json(s: &DdSnapshot) -> String {
    let key_of: HashMap<&str, i64> = s
        .id_by_key
        .iter()
        .map(|(k, id)| (id.as_str(), *k))
        .collect();
    let rows: Vec<CachedChampion> = s
        .champions
        .iter()
        .map(|c| CachedChampion {
            key: key_of.get(c.id.as_str()).copied().unwrap_or_default(),
            champion: c.clone(),
        })
        .collect();
    serde_json::to_string(&rows).unwrap_or_else(|_| "[]".into())
}

fn from_cache_json(version: &str, json: &str) -> Option<DdSnapshot> {
    let rows: Vec<CachedChampion> = serde_json::from_str(json).ok()?;
    if rows.is_empty() {
        return None;
    }
    let id_by_key = rows
        .iter()
        .filter(|r| r.key != 0)
        .map(|r| (r.key, r.champion.id.clone()))
        .collect();
    Some(DdSnapshot {
        version: version.to_string(),
        champions: rows.into_iter().map(|r| r.champion).collect(),
        id_by_key,
    })
}

/// What a refresh did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefreshOutcome {
    /// First version this database has seen.
    FirstSeen(String),
    Unchanged(String),
    /// The patch changed from the first to the second version (a
    /// `patch_change` row was written).
    PatchChanged(String, String),
}

/// Apply a freshly fetched snapshot: keep it in memory, persist it as the
/// last good copy and record a patch change. Split from the network fetch
/// so it can be tested offline.
pub async fn apply_snapshot(
    db: &Surreal<Db>,
    fresh: DdSnapshot,
) -> crate::server::db::DbResult<RefreshOutcome> {
    use crate::server::db;
    let previous = db::load_ddragon_cache(db).await?.map(|(v, _)| v);
    let version = fresh.version.clone();
    db::save_ddragon_cache(db, &version, &to_cache_json(&fresh)).await?;
    set_snapshot(fresh);
    Ok(match previous {
        None => {
            tracing::info!(%version, "Data Dragon: first patch version seen");
            RefreshOutcome::FirstSeen(version)
        }
        Some(old) if old == version => RefreshOutcome::Unchanged(version),
        Some(old) => {
            db::record_patch_change(db, &old, &version).await?;
            tracing::info!(old = %old, new = %version, "Data Dragon: new patch detected");
            RefreshOutcome::PatchChanged(old, version)
        }
    })
}

/// Fetch the latest version and champions from Data Dragon and apply them.
pub async fn refresh(db: &Surreal<Db>) -> Result<RefreshOutcome, String> {
    let fresh = fetch_snapshot().await.map_err(|e| e.to_string())?;
    apply_snapshot(db, fresh).await.map_err(|e| e.to_string())
}

/// Load the last good copy from the DB into memory (when nothing newer is
/// loaded). Returns the cached version, if any.
pub async fn load_cached(db: &Surreal<Db>) -> crate::server::db::DbResult<Option<String>> {
    let Some((version, json)) = crate::server::db::load_ddragon_cache(db).await? else {
        return Ok(None);
    };
    match from_cache_json(&version, &json) {
        Some(s) => {
            if snapshot().is_none() {
                set_snapshot(s);
            }
            Ok(Some(version))
        }
        None => {
            tracing::warn!("Data Dragon: stored copy is unreadable; ignoring it");
            Ok(None)
        }
    }
}

/// Startup: load the DB copy, then try a live refresh. Never fails; problems
/// are logged and the app keeps running on whatever copy it has.
pub async fn init(db: &Surreal<Db>) {
    match load_cached(db).await {
        Ok(Some(v)) => tracing::info!(version = %v, "Data Dragon: loaded cached copy"),
        Ok(None) => tracing::info!("Data Dragon: no cached copy yet"),
        Err(e) => tracing::warn!("Data Dragon: could not read cached copy: {e}"),
    }
    if let Err(e) = refresh(db).await {
        tracing::warn!(
            "Data Dragon refresh failed ({e}); using {}",
            if snapshot().is_some() {
                "the cached copy"
            } else {
                "no champion data until the next refresh"
            }
        );
    }
}

/// Refresh every [`REFRESH_EVERY`] in the background.
pub fn spawn_refresh_task(db: Arc<Surreal<Db>>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(REFRESH_EVERY);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await; // the first tick fires immediately; init already refreshed
        loop {
            interval.tick().await;
            if let Err(e) = refresh(&db).await {
                tracing::warn!("Data Dragon refresh failed: {e}");
            }
        }
    });
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    fn doc() -> serde_json::Value {
        serde_json::json!({
            "type": "champion",
            "version": "99.1.1",
            "data": {
                "Aatrox": {"id": "Aatrox", "key": "266", "name": "Aatrox", "title": "the Darkin Blade",
                           "tags": ["Fighter"], "image": {"full": "Aatrox.png"}},
                "KSante": {"id": "KSante", "key": "897", "name": "K'Sante", "title": "the Pride of Nazumah",
                           "tags": ["Tank", "Fighter"], "image": {"full": "KSante.png"}},
                "Broken": {"id": "Broken", "name": "No key"}
            }
        })
    }

    #[test]
    fn champion_json_parses_into_a_snapshot() {
        let s = parse_champion_json("99.1.1", &doc()).unwrap();
        assert_eq!(s.version, "99.1.1");
        assert_eq!(s.champions.len(), 2, "entries without a key are skipped");
        assert_eq!(s.id_by_key.get(&897).map(String::as_str), Some("KSante"));
        let aatrox = s.champions.iter().find(|c| c.id == "Aatrox").unwrap();
        // Images are served by the app, never the CDN, from the browser.
        assert_eq!(aatrox.image_full, "/img/champion/Aatrox/icon?v=99.1.1");
    }

    #[test]
    fn cache_json_round_trips() {
        let s = parse_champion_json("99.1.1", &doc()).unwrap();
        let back = from_cache_json("99.1.1", &to_cache_json(&s)).unwrap();
        assert_eq!(back.champions, s.champions);
        assert_eq!(back.id_by_key, s.id_by_key);
        assert!(from_cache_json("1", "not json").is_none());
        assert!(from_cache_json("1", "[]").is_none());
    }

    #[test]
    fn empty_champion_json_is_an_error() {
        assert!(parse_champion_json("1", &serde_json::json!({"data": {}})).is_err());
        assert!(parse_champion_json("1", &serde_json::json!({})).is_err());
    }
}
