#![cfg(feature = "ssr")]

mod common;

use lol_team_companion::server::db;

async fn make_user(d: &surrealdb::Surreal<surrealdb::engine::local::Db>) -> String {
    db::create_user(
        d,
        "lp_user".into(),
        "lp@example.com".into(),
        "hash".into(),
    )
    .await
    .unwrap()
}

async fn insert_snapshot(
    d: &surrealdb::Surreal<surrealdb::engine::local::Db>,
    user_id: &str,
    tier: &str,
    division: &str,
    lp: i32,
) {
    let user_key = user_id
        .strip_prefix("user:")
        .unwrap_or(user_id)
        .to_string();
    d.query(
        "CREATE ranked_snapshot SET \
         user = type::record('user', $u), \
         queue_type = 'RANKED_SOLO_5x5', \
         tier = $tier, division = $div, lp = $lp, \
         wins = 0, losses = 0",
    )
    .bind(("u", user_key))
    .bind(("tier", tier.to_string()))
    .bind(("div", division.to_string()))
    .bind(("lp", lp))
    .await
    .unwrap()
    .check()
    .unwrap();
}

#[tokio::test]
async fn get_lp_history_empty_returns_empty_vec() {
    let d = common::test_db().await;
    let u = make_user(&d).await;
    let history = db::get_lp_history(&d, &u, None).await.unwrap();
    assert!(history.is_empty());
}

#[tokio::test]
async fn get_lp_history_includes_rank_score() {
    let d = common::test_db().await;
    let u = make_user(&d).await;
    insert_snapshot(&d, &u, "GOLD", "II", 47).await;
    let history = db::get_lp_history(&d, &u, None).await.unwrap();
    assert_eq!(history.len(), 1);
    // Gold II 47LP: tier_idx=3, div_idx=2 → 3*400 + 2*100 + 47 = 1447
    assert_eq!(
        history[0].rank_score, 1447,
        "rank_score must be 1447 for Gold II 47LP"
    );
    assert_eq!(history[0].tier, "GOLD");
}

#[tokio::test]
async fn get_lp_history_sorted_ascending() {
    let d = common::test_db().await;
    let u = make_user(&d).await;
    // Insert with small delays so snapshotted_at DEFAULT time::now() differs.
    insert_snapshot(&d, &u, "GOLD", "IV", 0).await;
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    insert_snapshot(&d, &u, "GOLD", "II", 47).await;
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    insert_snapshot(&d, &u, "PLATINUM", "IV", 12).await;
    let history = db::get_lp_history(&d, &u, None).await.unwrap();
    assert_eq!(history.len(), 3);
    // ASC: oldest (Gold IV 0LP = 1200) → Gold II 47LP (1447) → Plat IV 12LP (1612)
    assert!(
        history[0].rank_score < history[1].rank_score,
        "history must be sorted ascending by rank_score"
    );
    assert!(history[1].rank_score < history[2].rank_score);
}
