//! M11 retention of the shared Riot match caches: cached games (detail and
//! timeline) older than the cutoff go, younger ones stay; `match` rows go
//! only when no player's synced games use them any more; nothing else is
//! touched. Batches smaller than the work prove the loop finishes.

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

/// `age` in days for the cache time (details) or the game end (matches).
async fn cached_game(db: &Surreal<Db>, id: &str, age: i64, events: i64) {
    db.query(
        "CREATE match_detail SET match_id = $id, participants_json = '[]', game_duration = 1800, \
             fetched_at = time::now() - <duration>$age; \
         FOR $i IN 0..$events { CREATE match_timeline_event SET match_id = $id, timestamp_ms = $i * 1000, event_type = 'CHAMPION_KILL'; };",
    )
    .bind(("id", id.to_string()))
    .bind(("age", format!("{age}d")))
    .bind(("events", events))
    .await
    .unwrap()
    .check()
    .unwrap();
}

async fn match_row(db: &Surreal<Db>, id: &str, age: Option<i64>, synced_by: Option<&str>) {
    db.query(
        "LET $m = (CREATE ONLY match SET match_id = $id, queue_id = 420, game_duration = 1800, \
             game_end = IF $age = NONE { NONE } ELSE { time::now() - <duration>$age }); \
         IF $user != NONE { CREATE player_match SET match = $m.id, user = type::record('user', $user), \
             champion = 'Ahri', kills = 1, deaths = 2, assists = 3, cs = 100, vision_score = 10, damage = 1000, win = true };",
    )
    .bind(("id", id.to_string()))
    .bind(("age", age.map(|a| format!("{a}d"))))
    .bind((
        "user",
        synced_by.map(|u| u.strip_prefix("user:").unwrap_or(u).to_string()),
    ))
    .await
    .unwrap()
    .check()
    .unwrap();
}

#[tokio::test]
async fn old_match_caches_are_deleted_in_batches() {
    let db = crate::common::test_db().await;
    let ana = db::create_user(&db, "ana".into(), "ana@example.com".into(), "h".into())
        .await
        .unwrap();

    // Five games cached 400 days ago, two yesterday; each with a timeline.
    for i in 0..5 {
        cached_game(&db, &format!("OLD_{i}"), 400, 3).await;
    }
    cached_game(&db, "NEW_0", 1, 4).await;
    cached_game(&db, "NEW_1", 364, 2).await;
    // Match rows: old and unused (go), old but in ana's synced games (stay),
    // recent and unused (stay), unused without an end time (stay).
    match_row(&db, "M_OLD_A", Some(500), None).await;
    match_row(&db, "M_OLD_B", Some(380), None).await;
    match_row(&db, "M_OLD_C", Some(370), None).await;
    match_row(&db, "M_OLD_SYNCED", Some(500), Some(&ana)).await;
    match_row(&db, "M_NEW", Some(10), None).await;
    match_row(&db, "M_NO_END", None, None).await;

    let cutoff = chrono::Utc::now() - chrono::Duration::days(365);
    let purged = db::purge_match_cache(&db, cutoff, 2).await.unwrap();
    assert_eq!(
        purged,
        db::MatchCachePurge {
            details: 5,
            timeline_events: 15,
            matches: 3
        }
    );

    assert_eq!(
        count(&db, "SELECT count() AS n FROM match_detail GROUP ALL").await,
        2
    );
    assert_eq!(
        count(&db, "SELECT count() AS n FROM match_detail WHERE string::starts_with(match_id, 'NEW_') GROUP ALL").await,
        2
    );
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM match_timeline_event GROUP ALL"
        )
        .await,
        6
    );
    assert_eq!(
        count(&db, "SELECT count() AS n FROM match_timeline_event WHERE string::starts_with(match_id, 'OLD_') GROUP ALL").await,
        0
    );
    let mut r = db
        .query("SELECT VALUE match_id FROM match ORDER BY match_id")
        .await
        .unwrap();
    let left: Vec<String> = r.take(0).unwrap();
    assert_eq!(left, vec!["M_NEW", "M_NO_END", "M_OLD_SYNCED"]);
    // Ana's synced game is untouched and still resolves its match.
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM player_match WHERE match.match_id = 'M_OLD_SYNCED' GROUP ALL"
        )
        .await,
        1
    );
    // The cache still serves the young games.
    assert!(db::get_cached_match_detail(&db, "NEW_0")
        .await
        .unwrap()
        .is_some_and(|(_, events, _, _)| events.len() == 4));
    assert!(db::get_cached_match_detail(&db, "OLD_0")
        .await
        .unwrap()
        .is_none());

    // A second run finds nothing.
    let again = db::purge_match_cache(&db, cutoff, 2).await.unwrap();
    assert_eq!(again, db::MatchCachePurge::default());
}

/// A game cached through the app (`store_match_detail`) is kept until the
/// retention window has passed.
#[tokio::test]
async fn a_freshly_cached_game_is_kept() {
    let db = crate::common::test_db().await;
    db::store_match_detail(&db, "EUW1_FRESH", &[], &[], 1500, "CLASSIC")
        .await
        .unwrap();
    let cutoff = chrono::Utc::now() - chrono::Duration::days(365);
    let purged = db::purge_match_cache(&db, cutoff, db::MATCH_CACHE_BATCH)
        .await
        .unwrap();
    assert_eq!(purged, db::MatchCachePurge::default());
    assert!(db::get_cached_match_detail(&db, "EUW1_FRESH")
        .await
        .unwrap()
        .is_some());
    // With a cutoff in the future (retention of zero days) it would go.
    let purged = db::purge_match_cache(&db, chrono::Utc::now() + chrono::Duration::minutes(1), 10)
        .await
        .unwrap();
    assert_eq!(purged.details, 1);
}

/// Review (M11): syncs and the retention task share the write lock. An old
/// game synced now keeps its `match` row (its `player_match` comes in the
/// same locked step), and a sync that finishes after the account was
/// deleted stores nothing (no `player_match` pointing at a missing user).
#[tokio::test]
async fn syncs_and_retention_do_not_race() {
    use lol_team_companion::server::{account, riot::MatchData};
    let db = crate::common::test_db().await;
    let ana = db::create_user(&db, "ana".into(), "ana@example.com".into(), "h".into())
        .await
        .unwrap();
    let two_years_ago = (chrono::Utc::now() - chrono::Duration::days(730)).timestamp_millis();
    let game = |id: &str| MatchData {
        match_id: id.into(),
        queue_id: 420,
        game_duration: 1800,
        game_end_epoch_ms: Some(two_years_ago),
        champion: "Ahri".into(),
        kills: 1,
        deaths: 2,
        assists: 3,
        cs: 200,
        vision_score: 20,
        damage: 15000,
        win: true,
    };
    let cutoff = chrono::Utc::now() - chrono::Duration::days(365);
    // A long-idle player syncs games older than the retention window while
    // the task runs: the rows stay with her synced games.
    let (stored, purged) = tokio::join!(
        db::store_matches(
            &db,
            &ana,
            (0..20).map(|i| game(&format!("EUW1_OLD{i}"))).collect()
        ),
        db::purge_match_cache(&db, cutoff, 2),
    );
    stored.unwrap();
    purged.unwrap();
    db::purge_match_cache(&db, cutoff, 2).await.unwrap();
    assert_eq!(
        count(&db, "SELECT count() AS n FROM match GROUP ALL").await,
        20
    );
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM player_match WHERE match.match_id != NONE GROUP ALL"
        )
        .await,
        20
    );

    // Riot answered after the account was deleted: nothing is stored.
    account::delete_account(&db, &ana).await.unwrap();
    let late = db::store_matches(&db, &ana, vec![game("EUW1_LATE")]).await;
    assert!(matches!(late, Err(db::DbError::NotFound)), "{late:?}");
    assert_eq!(
        count(&db, "SELECT count() AS n FROM player_match GROUP ALL").await,
        0
    );
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM match WHERE match_id = 'EUW1_LATE' GROUP ALL"
        )
        .await,
        0
    );
    // With her games gone, the task now removes the old rows.
    let purged = db::purge_match_cache(&db, cutoff, 7).await.unwrap();
    assert_eq!(purged.matches, 20);
}

/// M141: ranked snapshots (the LP history, one per sync) follow the match
/// cache's retention: older ones go, recent ones stay, a second run finds
/// nothing.
#[tokio::test]
async fn old_ranked_snapshots_are_purged() {
    let db = crate::common::test_db().await;
    let ana = db::create_user(&db, "ana".into(), "ana@example.com".into(), "h".into())
        .await
        .unwrap();
    for _ in 0..3 {
        db::store_ranked_snapshot(&db, &ana, "RANKED_SOLO_5x5", "GOLD", "II", 40, 10, 9)
            .await
            .unwrap();
    }
    // Two of them are from more than a year ago.
    db.query("LET $two = (SELECT VALUE id FROM ranked_snapshot LIMIT 2); UPDATE ranked_snapshot SET snapshotted_at = time::now() - 400d WHERE id IN $two;")
        .await
        .unwrap()
        .check()
        .unwrap();
    let cutoff = chrono::Utc::now() - chrono::Duration::days(365);
    assert_eq!(db::purge_ranked_snapshots(&db, cutoff, 1).await.unwrap(), 2);
    assert_eq!(
        count(&db, "SELECT count() AS n FROM ranked_snapshot GROUP ALL").await,
        1
    );
    assert_eq!(db::purge_ranked_snapshots(&db, cutoff, 1).await.unwrap(), 0);
}
