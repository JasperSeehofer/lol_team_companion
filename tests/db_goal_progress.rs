#![cfg(feature = "ssr")]

mod common;

use lol_team_companion::server::db;

async fn make_user(d: &surrealdb::Surreal<surrealdb::engine::local::Db>, name: &str) -> String {
    db::create_user(
        d,
        name.into(),
        format!("{name}@example.com"),
        "hash".into(),
    )
    .await
    .unwrap()
}

/// Insert a match and a corresponding player_match row for the given user.
/// Uses `(SELECT VALUE id FROM match WHERE match_id = $mid)[0]` to get the record ID.
async fn insert_match_with_player(
    d: &surrealdb::Surreal<surrealdb::engine::local::Db>,
    match_id: &str,
    user_id: &str,
    queue_id: i32,
    duration_sec: i32,
    kills: i32,
    deaths: i32,
    assists: i32,
    cs: i32,
) {
    let user_key = user_id
        .strip_prefix("user:")
        .unwrap_or(user_id)
        .to_string();
    d.query(
        "CREATE match SET match_id = $mid, queue_id = $qid, game_duration = $dur, \
         game_end = time::now()",
    )
    .bind(("mid", match_id.to_string()))
    .bind(("qid", queue_id))
    .bind(("dur", duration_sec))
    .await
    .unwrap()
    .check()
    .unwrap();
    d.query(
        "CREATE player_match SET \
         match = (SELECT VALUE id FROM match WHERE match_id = $mid)[0], \
         user = type::record('user', $u), \
         champion = 'TestChamp', \
         kills = $k, deaths = $de, assists = $a, cs = $cs, \
         vision_score = 20, damage = 15000, win = true",
    )
    .bind(("mid", match_id.to_string()))
    .bind(("u", user_key))
    .bind(("k", kills))
    .bind(("de", deaths))
    .bind(("a", assists))
    .bind(("cs", cs))
    .await
    .unwrap()
    .check()
    .unwrap();
}

#[tokio::test]
async fn goal_progress_zero_games_insufficient() {
    let d = common::test_db().await;
    let u = make_user(&d, "zero").await;
    db::upsert_personal_goal(&d, &u, "cs_per_min", "7.0")
        .await
        .unwrap();
    let p = db::compute_goal_progress(&d, &u).await.unwrap();
    let cs = p.cs.expect("cs goal should be present");
    assert!(cs.current_value.is_none(), "zero games must be insufficient");
    assert_eq!(cs.game_count, 0);
}

#[tokio::test]
async fn goal_progress_three_games_below_threshold() {
    let d = common::test_db().await;
    let u = make_user(&d, "three").await;
    db::upsert_personal_goal(&d, &u, "cs_per_min", "7.0")
        .await
        .unwrap();
    for i in 0..3 {
        insert_match_with_player(&d, &format!("M{i}"), &u, 420, 1800, 5, 2, 3, 180).await;
    }
    let p = db::compute_goal_progress(&d, &u).await.unwrap();
    let cs = p.cs.expect("cs goal should be present");
    assert!(
        cs.current_value.is_none(),
        "< 5 games must be insufficient (D-15)"
    );
    assert_eq!(cs.game_count, 3);
}

#[tokio::test]
async fn goal_progress_seven_games_aggregates() {
    let d = common::test_db().await;
    let u = make_user(&d, "seven").await;
    db::upsert_personal_goal(&d, &u, "cs_per_min", "7.0")
        .await
        .unwrap();
    db::upsert_personal_goal(&d, &u, "deaths_per_game", "4")
        .await
        .unwrap();
    // cs=180, duration=1800s → cs/min = 180 / (1800/60) = 180/30 = 6.0
    for i in 0..7 {
        insert_match_with_player(&d, &format!("S{i}"), &u, 420, 1800, 5, 3, 4, 180).await;
    }
    let p = db::compute_goal_progress(&d, &u).await.unwrap();
    let cs = p.cs.expect("cs goal");
    let deaths = p.deaths.expect("deaths goal");
    assert!(
        cs.current_value.is_some(),
        "≥ 5 games must produce a value"
    );
    assert_eq!(cs.game_count, 7);
    assert!(deaths.current_value.is_some());
    assert_eq!(deaths.game_count, 7);
    // cs/min = 180 / 30 = 6.0; game_duration is in SECONDS (Pitfall 1)
    assert!(
        (cs.current_value.unwrap() - 6.0).abs() < 0.1,
        "expected cs/min ≈ 6.0 (seconds-based), got {:?}",
        cs.current_value
    );
}

#[tokio::test]
async fn goal_progress_caps_at_twenty_games() {
    let d = common::test_db().await;
    let u = make_user(&d, "many").await;
    db::upsert_personal_goal(&d, &u, "cs_per_min", "7.0")
        .await
        .unwrap();
    for i in 0..35 {
        insert_match_with_player(&d, &format!("X{i}"), &u, 420, 1800, 5, 2, 3, 180).await;
    }
    let p = db::compute_goal_progress(&d, &u).await.unwrap();
    let cs = p.cs.expect("cs goal");
    assert_eq!(cs.game_count, 20, "window must cap at 20 (D-12)");
}

#[tokio::test]
async fn goal_progress_queue_isolation_solo_only() {
    let d = common::test_db().await;
    let u = make_user(&d, "iso").await;
    db::upsert_personal_goal(&d, &u, "cs_per_min", "7.0")
        .await
        .unwrap();
    // 4 solo + 4 flex — together they would cross 5-game threshold, but solo alone does not
    for i in 0..4 {
        insert_match_with_player(&d, &format!("SOLO{i}"), &u, 420, 1800, 5, 2, 3, 180).await;
    }
    for i in 0..4 {
        insert_match_with_player(&d, &format!("FLEX{i}"), &u, 440, 1800, 5, 2, 3, 180).await;
    }
    let p = db::compute_goal_progress(&d, &u).await.unwrap();
    let cs = p.cs.expect("cs goal");
    // Solo-only count = 4 → still below threshold; flex must not be counted
    assert_eq!(
        cs.game_count, 4,
        "must use solo/duo only (queue_id=420), got {}",
        cs.game_count
    );
    assert!(cs.current_value.is_none());
}
