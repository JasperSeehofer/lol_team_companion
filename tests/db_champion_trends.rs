#![cfg(feature = "ssr")]

mod common;

use lol_team_companion::server::db;

async fn make_user(d: &surrealdb::Surreal<surrealdb::engine::local::Db>) -> String {
    db::create_user(
        d,
        "trend_user".into(),
        "trend@example.com".into(),
        "hash".into(),
    )
    .await
    .unwrap()
}

/// Insert a match + player_match row with configurable fields.
#[allow(clippy::too_many_arguments)]
async fn insert_match_pm(
    d: &surrealdb::Surreal<surrealdb::engine::local::Db>,
    match_id: &str,
    user_id: &str,
    queue_id: i32,
    duration_sec: i32,
    champion: &str,
    kills: i32,
    deaths: i32,
    assists: i32,
    cs: i32,
    damage: i32,
    win: bool,
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
         champion = $champ, \
         kills = $k, deaths = $de, assists = $a, cs = $cs, \
         vision_score = 20, damage = $dmg, win = $win",
    )
    .bind(("mid", match_id.to_string()))
    .bind(("u", user_key))
    .bind(("champ", champion.to_string()))
    .bind(("k", kills))
    .bind(("de", deaths))
    .bind(("a", assists))
    .bind(("cs", cs))
    .bind(("dmg", damage))
    .bind(("win", win))
    .await
    .unwrap()
    .check()
    .unwrap();
}

#[tokio::test]
async fn get_champion_trends_empty_returns_empty_vec() {
    let d = common::test_db().await;
    let u = make_user(&d).await;
    let trends = db::get_champion_trends(&d, &u, None).await.unwrap();
    assert!(trends.is_empty());
}

#[tokio::test]
async fn get_champion_trends_aggregates_correctly() {
    let d = common::test_db().await;
    let u = make_user(&d).await;
    insert_match_pm(&d, "T1", &u, 420, 1800, "Jinx", 5, 2, 3, 180, 20000, true).await;
    insert_match_pm(&d, "T2", &u, 420, 2000, "Jinx", 3, 4, 5, 200, 22000, false).await;
    let trends = db::get_champion_trends(&d, &u, None).await.unwrap();
    assert_eq!(trends.len(), 1);
    let jinx = &trends[0];
    assert_eq!(jinx.champion, "Jinx");
    assert_eq!(jinx.games, 2);
    assert_eq!(jinx.wins, 1);
}

#[tokio::test]
async fn get_champion_trends_cs_per_min_uses_seconds() {
    let d = common::test_db().await;
    let u = make_user(&d).await;
    // cs=180 over 1800 seconds = 180 / (1800/60) = 180/30 = 6.0 cs/min
    // If bug: cs/game_duration = 180/1800 = 0.1 — clearly wrong
    insert_match_pm(&d, "C1", &u, 420, 1800, "Vayne", 5, 2, 3, 180, 18000, true).await;
    let trends = db::get_champion_trends(&d, &u, None).await.unwrap();
    assert_eq!(trends.len(), 1);
    assert!(
        (trends[0].cs_per_min - 6.0).abs() < 0.1,
        "expected cs/min = 6.0 (seconds-based), got {} (would be 0.1 with minutes bug)",
        trends[0].cs_per_min
    );
}

#[tokio::test]
async fn get_champion_trends_excludes_aram() {
    let d = common::test_db().await;
    let u = make_user(&d).await;
    // ARAM = queue_id 450 — must be excluded from champion trends (D-19)
    insert_match_pm(&d, "ARAM1", &u, 450, 1200, "Ezreal", 8, 3, 6, 100, 25000, true).await;
    let trends = db::get_champion_trends(&d, &u, None).await.unwrap();
    assert!(
        trends.iter().all(|t| t.champion != "Ezreal"),
        "ARAM (queue_id=450) must be excluded from champion trends"
    );
}

#[tokio::test]
async fn get_champion_trends_includes_solo_and_flex() {
    let d = common::test_db().await;
    let u = make_user(&d).await;
    insert_match_pm(&d, "SOLO1", &u, 420, 1800, "Yasuo", 5, 4, 7, 180, 19000, true).await;
    insert_match_pm(&d, "FLEX1", &u, 440, 1800, "Yasuo", 6, 3, 8, 200, 21000, true).await;
    let trends = db::get_champion_trends(&d, &u, None).await.unwrap();
    let yasuo = trends
        .iter()
        .find(|t| t.champion == "Yasuo")
        .expect("Yasuo trend must be present");
    assert_eq!(yasuo.games, 2, "both solo and flex games must be aggregated");
}
