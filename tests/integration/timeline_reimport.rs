//! M149 item A: the timeline keeps positions and purchases, and an admin
//! re-imports stored games. Covers storing and reading the new fields, the
//! re-import's replace (one transaction, idempotent), which games the
//! backfill selects (older importer versions only, never seed demo games,
//! not again once Riot had no data), and that the retention purge still
//! removes the new rows. No Riot API: events are built here.

use lol_team_companion::models::match_data::{
    EventCategory, MapPosition, TimelineEvent, TIMELINE_VERSION,
};
use lol_team_companion::server::db;
use surrealdb::{engine::local::Db, types::SurrealValue, Surreal};

async fn count(db: &Surreal<Db>, q: &str) -> i64 {
    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct N {
        n: i64,
    }
    let mut r = db.query(q).await.unwrap();
    let rows: Vec<N> = r.take(0).unwrap();
    rows.first().map(|n| n.n).unwrap_or(0)
}

fn event(ts: i64, kind: &str, cat: EventCategory) -> TimelineEvent {
    TimelineEvent {
        timestamp_ms: ts,
        event_type: kind.into(),
        category: cat,
        team_id: Some(100),
        killer_participant_id: None,
        victim_participant_id: None,
        monster_type: None,
        monster_sub_type: None,
        building_type: None,
        is_first_blood: false,
        multi_kill_length: None,
        is_teamfight: false,
        involved_participants: Vec::new(),
        participant_id: None,
        item_id: None,
        position: None,
    }
}

fn new_events() -> Vec<TimelineEvent> {
    let mut kill = event(300_000, "CHAMPION_KILL", EventCategory::Kill);
    kill.killer_participant_id = Some(1);
    kill.victim_participant_id = Some(6);
    kill.involved_participants = vec![1, 6];
    kill.position = Some(MapPosition { x: 5000, y: 9000 });
    let mut buy = event(400_000, "ITEM_PURCHASED", EventCategory::Purchase);
    buy.participant_id = Some(1);
    buy.item_id = Some(3802);
    let mut ward = event(95_000, "WARD_PLACED", EventCategory::Ward);
    ward.participant_id = Some(1);
    vec![ward, kill, buy]
}

/// A game cached by the importer before M149: no version, one old event.
async fn old_cached_game(db: &Surreal<Db>, id: &str) {
    db.query(
        "CREATE match_detail SET match_id = $id, participants_json = '[]', game_duration = 1800; \
         CREATE match_timeline_event SET match_id = $id, timestamp_ms = 1000, event_type = 'CHAMPION_KILL';",
    )
    .bind(("id", id.to_string()))
    .await
    .unwrap()
    .check()
    .unwrap();
}

#[tokio::test]
async fn new_fields_are_stored_and_read_back() {
    let db = crate::common::test_db().await;
    db::store_match_detail(&db, "EUW1_1", &[], &new_events(), 1800, "CLASSIC")
        .await
        .unwrap();
    let (_, events, _, _) = db::get_cached_match_detail(&db, "EUW1_1")
        .await
        .unwrap()
        .unwrap();
    let kill = events
        .iter()
        .find(|e| e.event_type == "CHAMPION_KILL")
        .unwrap();
    assert_eq!(kill.position, Some(MapPosition { x: 5000, y: 9000 }));
    let buy = events
        .iter()
        .find(|e| e.event_type == "ITEM_PURCHASED")
        .unwrap();
    assert_eq!(buy.category, EventCategory::Purchase);
    assert_eq!(buy.participant_id, Some(1));
    assert_eq!(buy.item_id, Some(3802));
    let ward = events
        .iter()
        .find(|e| e.event_type == "WARD_PLACED")
        .unwrap();
    assert_eq!(ward.participant_id, Some(1));
    assert_eq!(ward.position, None);
    // A game cached now is written by the current importer: nothing to re-import.
    assert!(db::matches_needing_reimport(&db, 10)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn the_backfill_selects_only_older_games() {
    let db = crate::common::test_db().await;
    old_cached_game(&db, "EUW1_OLD_A").await;
    old_cached_game(&db, "EUW1_OLD_B").await;
    // A seed demo game is never sent to Riot, whatever its version.
    old_cached_game(&db, "SEED1_1000000009").await;
    db::store_match_detail(&db, "EUW1_NEW", &[], &new_events(), 1800, "CLASSIC")
        .await
        .unwrap();
    // An explicit older version counts as old too.
    db.query("UPDATE match_detail SET timeline_version = 1 WHERE match_id = 'EUW1_OLD_B'")
        .await
        .unwrap()
        .check()
        .unwrap();

    let ids = db::matches_needing_reimport(&db, 10).await.unwrap();
    assert_eq!(
        ids,
        vec!["EUW1_OLD_A".to_string(), "EUW1_OLD_B".to_string()]
    );
    assert_eq!(
        db::matches_needing_reimport(&db, 1).await.unwrap(),
        vec!["EUW1_OLD_A".to_string()]
    );
    assert_eq!(db::reimport_counts(&db).await.unwrap(), (2, 0));

    // Riot has no data for B: marked, not selected again, counted apart,
    // and its stored timeline stays.
    db::mark_reimport_failed(&db, "EUW1_OLD_B").await.unwrap();
    assert_eq!(
        db::matches_needing_reimport(&db, 10).await.unwrap(),
        vec!["EUW1_OLD_A".to_string()]
    );
    assert_eq!(db::reimport_counts(&db).await.unwrap(), (1, 1));
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM match_timeline_event WHERE match_id = 'EUW1_OLD_B' GROUP ALL"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn the_replace_is_whole_and_idempotent() {
    let db = crate::common::test_db().await;
    old_cached_game(&db, "EUW1_OLD").await;
    let events = new_events();
    for _ in 0..2 {
        db::replace_match_timeline(&db, "EUW1_OLD", &[], &events, 1900, "CLASSIC")
            .await
            .unwrap();
        // The old event is gone and the new ones are there exactly once.
        assert_eq!(
            count(
                &db,
                "SELECT count() AS n FROM match_timeline_event WHERE match_id = 'EUW1_OLD' GROUP ALL"
            )
            .await,
            3
        );
        assert_eq!(
            count(
                &db,
                "SELECT count() AS n FROM match_detail WHERE match_id = 'EUW1_OLD' GROUP ALL"
            )
            .await,
            1
        );
    }
    let (_, stored, dur, _) = db::get_cached_match_detail(&db, "EUW1_OLD")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(dur, 1900);
    assert!(stored.iter().all(|e| e.timestamp_ms != 1000));
    assert_eq!(
        count(
            &db,
            &format!(
                "SELECT count() AS n FROM match_detail WHERE match_id = 'EUW1_OLD' \
                 AND timeline_version = {TIMELINE_VERSION} GROUP ALL"
            )
        )
        .await,
        1
    );
    assert!(db::matches_needing_reimport(&db, 10)
        .await
        .unwrap()
        .is_empty());
    assert_eq!(db::reimport_counts(&db).await.unwrap(), (0, 0));

    // A game no longer cached (purged meanwhile) is not recreated.
    let gone = db::replace_match_timeline(&db, "EUW1_GONE", &[], &events, 1900, "CLASSIC").await;
    assert!(matches!(gone, Err(db::DbError::NotFound)), "{gone:?}");
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM match_timeline_event WHERE match_id = 'EUW1_GONE' GROUP ALL"
        )
        .await,
        0
    );
}

#[tokio::test]
async fn a_failed_replace_keeps_the_old_timeline() {
    let db = crate::common::test_db().await;
    old_cached_game(&db, "EUW1_OLD").await;
    // Make the second new event fail its write: the whole replace rolls back.
    db.query(
        "DEFINE FIELD OVERWRITE event_type ON match_timeline_event TYPE string \
         ASSERT $value != 'ITEM_PURCHASED';",
    )
    .await
    .unwrap()
    .check()
    .unwrap();
    assert!(
        db::replace_match_timeline(&db, "EUW1_OLD", &[], &new_events(), 1900, "CLASSIC")
            .await
            .is_err()
    );
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM match_timeline_event WHERE match_id = 'EUW1_OLD' GROUP ALL"
        )
        .await,
        1
    );
    assert_eq!(
        db::matches_needing_reimport(&db, 10).await.unwrap(),
        vec!["EUW1_OLD".to_string()]
    );
}

#[tokio::test]
async fn the_retention_purge_removes_reimported_rows() {
    let db = crate::common::test_db().await;
    old_cached_game(&db, "EUW1_OLD").await;
    db::replace_match_timeline(&db, "EUW1_OLD", &[], &new_events(), 1900, "CLASSIC")
        .await
        .unwrap();
    // The purge deletes a re-imported game with all its new rows.
    let purged = db::purge_match_cache(
        &db,
        chrono::Utc::now() + chrono::Duration::minutes(1),
        db::MATCH_CACHE_BATCH,
    )
    .await
    .unwrap();
    assert_eq!(purged.details, 1);
    assert_eq!(purged.timeline_events, 3);
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM match_timeline_event GROUP ALL"
        )
        .await,
        0
    );
}
