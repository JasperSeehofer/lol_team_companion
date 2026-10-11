//! M6: Data Dragon cache persistence and patch-change detection, offline
//! (snapshots are built from a fixture document, not fetched).

use crate::common;
use lol_team_companion::server::data_dragon::{self, RefreshOutcome};
use lol_team_companion::server::db;

fn doc(version: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "champion",
        "version": version,
        "data": {
            "Aatrox": {"id": "Aatrox", "key": "266", "name": "Aatrox", "title": "the Darkin Blade",
                       "tags": ["Fighter"], "image": {"full": "Aatrox.png"}},
            "Ahri": {"id": "Ahri", "key": "103", "name": "Ahri", "title": "the Nine-Tailed Fox",
                     "tags": ["Mage"], "image": {"full": "Ahri.png"}}
        }
    })
}

// One test: the cache is process-wide state.
#[tokio::test]
async fn snapshots_persist_and_patch_changes_are_recorded() {
    let d = common::test_db().await;

    let first = data_dragon::parse_champion_json("90.1.1", &doc("90.1.1")).unwrap();
    assert_eq!(
        data_dragon::apply_snapshot(&d, first).await.unwrap(),
        RefreshOutcome::FirstSeen("90.1.1".into())
    );
    assert!(db::list_patch_changes(&d, 10).await.unwrap().is_empty());
    assert_eq!(data_dragon::version(), "90.1.1");
    assert_eq!(
        data_dragon::champion_id_for_key(103).as_deref(),
        Some("Ahri")
    );
    assert_eq!(data_dragon::champions().await.unwrap().len(), 2);

    let same = data_dragon::parse_champion_json("90.1.1", &doc("90.1.1")).unwrap();
    assert_eq!(
        data_dragon::apply_snapshot(&d, same).await.unwrap(),
        RefreshOutcome::Unchanged("90.1.1".into())
    );

    let next = data_dragon::parse_champion_json("90.2.1", &doc("90.2.1")).unwrap();
    assert_eq!(
        data_dragon::apply_snapshot(&d, next).await.unwrap(),
        RefreshOutcome::PatchChanged("90.1.1".into(), "90.2.1".into())
    );
    let changes = db::list_patch_changes(&d, 10).await.unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].old_version.as_deref(), Some("90.1.1"));
    assert_eq!(changes[0].new_version, "90.2.1");
    assert!(chrono::DateTime::parse_from_rfc3339(&changes[0].detected_at).is_ok());

    // The stored copy is the last good one and loads on a cold start.
    let (v, json) = db::load_ddragon_cache(&d).await.unwrap().unwrap();
    assert_eq!(v, "90.2.1");
    assert!(json.contains("\"key\":103"));
    assert_eq!(
        data_dragon::load_cached(&d).await.unwrap().as_deref(),
        Some("90.2.1")
    );
    let champs = data_dragon::champions().await.unwrap();
    assert!(champs[0].image_full.starts_with("/img/champion/"));
    assert!(champs[0].image_full.ends_with("/icon?v=90.2.1"));
    assert_eq!(
        lol_team_companion::models::ddragon::champion_icon_url("Ahri"),
        "/img/champion/Ahri/icon?v=90.2.1"
    );
}
