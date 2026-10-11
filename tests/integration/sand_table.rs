//! M6 Sand Table storage: an imported timeline round-trips through the DB,
//! re-imports replace, boards keep numbered versions. (Cross-tenant cases are
//! in security.rs.)

use crate::common;
use lol_team_companion::server::db::{self, DbError};
use lol_team_companion::server::sand_table::import_timeline_str;
use surrealdb::types::SurrealValue;

const FIXTURE: &str = include_str!("../fixtures/riot_match_v5_timeline.json");

async fn team(d: &surrealdb::Surreal<surrealdb::engine::local::Db>) -> (String, String) {
    let u = db::create_user(d, "coach".into(), "coach@example.com".into(), "h".into())
        .await
        .unwrap();
    let t = db::create_team(d, &u, "Sand".into(), "EUW".into())
        .await
        .unwrap();
    (u, t)
}

#[tokio::test]
async fn imported_timeline_round_trips() {
    let d = common::test_db().await;
    let (_, t) = team(&d).await;
    let imported = import_timeline_str(FIXTURE).unwrap();

    let id = db::store_replay(&d, &t, &imported, Some("15.6".into()))
        .await
        .unwrap();
    let replay = db::get_replay(&d, &t, &id).await.unwrap().unwrap();
    assert_eq!(replay.match_id, "EUW1_7100000001");
    assert_eq!(replay.source, "riot_timeline");
    assert_eq!(replay.patch.as_deref(), Some("15.6"));
    assert_eq!(replay.frame_interval_ms, 60_000);
    assert_eq!(replay.participants, imported.participants);
    assert!(replay.imported_at.is_some());

    let frames = db::get_replay_frames(&d, &t, &id).await.unwrap();
    assert_eq!(
        frames, imported.frames,
        "frames come back exactly as imported"
    );
    let events = db::get_replay_events(&d, &t, &id).await.unwrap();
    assert_eq!(events.len(), imported.events.len());
    assert_eq!(
        events, imported.events,
        "events (including meta) round-trip"
    );

    // Importing the same match again replaces the team's copy.
    let again = db::store_replay(&d, &t, &imported, None).await.unwrap();
    assert_ne!(again, id);
    let list = db::list_replays(&d, &t).await.unwrap();
    assert_eq!(list.len(), 1);
    assert!(db::get_replay(&d, &t, &id).await.unwrap().is_none());
    assert_eq!(
        db::get_replay_frames(&d, &t, &again).await.unwrap().len(),
        4
    );

    db::delete_replay(&d, &t, &again).await.unwrap();
    assert!(db::list_replays(&d, &t).await.unwrap().is_empty());
    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct Id {
        #[allow(dead_code)]
        id: surrealdb::types::RecordId,
    }
    let mut r = d
        .query("SELECT id FROM replay_frame; SELECT id FROM replay_event")
        .await
        .unwrap();
    let frames_left: Vec<Id> = r.take(0).unwrap();
    let events_left: Vec<Id> = r.take(1).unwrap();
    assert!(
        frames_left.is_empty() && events_left.is_empty(),
        "children deleted"
    );
}

#[tokio::test]
async fn boards_keep_numbered_versions() {
    let d = common::test_db().await;
    let (u, t) = team(&d).await;
    assert!(matches!(
        db::create_board(&d, &t, &u, "  ".into()).await,
        Err(DbError::Invalid(_))
    ));
    let b = db::create_board(&d, &t, &u, "Baron setup".into())
        .await
        .unwrap();
    assert!(db::get_board_version(&d, &t, &b, None)
        .await
        .unwrap()
        .is_none());

    let v1 = db::save_board_version(
        &d,
        &t,
        &b,
        &u,
        serde_json::json!({"tokens": [{"id": 1, "x": 10}]}),
    )
    .await
    .unwrap();
    let v2 = db::save_board_version(
        &d,
        &t,
        &b,
        &u,
        serde_json::json!({"tokens": [{"id": 1, "x": 20}], "arrows": []}),
    )
    .await
    .unwrap();
    assert_eq!((v1, v2), (1, 2));

    let latest = db::get_board_version(&d, &t, &b, None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest.version, 2);
    assert_eq!(latest.scene["tokens"][0]["x"], 20);
    assert_eq!(latest.author, u);
    let first = db::get_board_version(&d, &t, &b, Some(1))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first.scene["tokens"][0]["x"], 10);
    assert!(db::get_board_version(&d, &t, &b, Some(9))
        .await
        .unwrap()
        .is_none());

    assert!(matches!(
        db::save_board_version(&d, &t, &b, &u, serde_json::json!([1, 2])).await,
        Err(DbError::Invalid(_))
    ));

    let boards = db::list_boards(&d, &t).await.unwrap();
    assert_eq!(boards.len(), 1);
    assert_eq!(boards[0].name, "Baron setup");

    db::delete_board(&d, &t, &b).await.unwrap();
    assert!(db::list_boards(&d, &t).await.unwrap().is_empty());
}
