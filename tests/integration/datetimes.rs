//! M6: datetime fields load through `Datetime` (no `<string>` casts), so every
//! list that shows a timestamp returns its rows, and time-window filters
//! compare datetimes with datetimes.

use crate::common;
use lol_team_companion::models::champion::ChampionNote;
use lol_team_companion::server::db;
use lol_team_companion::server::riot::MatchData;

async fn user_and_team(d: &surrealdb::Surreal<surrealdb::engine::local::Db>) -> (String, String) {
    let u = db::create_user(d, "dana".into(), "dana@example.com".into(), "h".into())
        .await
        .unwrap();
    let t = db::create_team(d, &u, "Dates".into(), "EUW".into())
        .await
        .unwrap();
    (u, t)
}

fn is_rfc3339(s: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(s).is_ok() && s.ends_with('Z')
}

#[tokio::test]
async fn lists_with_timestamps_return_their_rows() {
    let d = common::test_db().await;
    let (u, t) = user_and_team(&d).await;

    db::create_action_item(&d, &t, "Ward river".into(), None, None, None)
        .await
        .unwrap();
    let items = db::list_action_items(&d, &t).await.unwrap();
    assert_eq!(items.len(), 1);
    assert!(is_rfc3339(items[0].created_at.as_deref().unwrap()));
    db::update_action_item_status(&d, &t, items[0].id.as_deref().unwrap(), "done".into())
        .await
        .unwrap();
    let items = db::list_action_items(&d, &t).await.unwrap();
    assert!(is_rfc3339(items[0].resolved_at.as_deref().unwrap()));
    assert_eq!(db::list_open_action_items(&d, &t).await.unwrap().len(), 0);

    db::create_team_note(&d, &t, &u, "dana", "Scrim at 8".into())
        .await
        .unwrap();
    let notes = db::list_team_notes(&d, &t).await.unwrap();
    assert_eq!(notes.len(), 1);
    assert!(is_rfc3339(notes[0].created_at.as_deref().unwrap()));

    db::add_champion_note(
        &d,
        &u,
        ChampionNote {
            id: None,
            user_id: u.clone(),
            champion: "Ahri".into(),
            role: "mid".into(),
            note_type: "matchup".into(),
            title: "vs Zed".into(),
            content: "Hold charm".into(),
            difficulty: Some(3),
            created_at: None,
        },
    )
    .await
    .unwrap();
    let cnotes = db::get_champion_notes(&d, &u, "Ahri", "mid").await.unwrap();
    assert_eq!(cnotes.len(), 1);
    assert!(is_rfc3339(cnotes[0].created_at.as_deref().unwrap()));

    let (opp, players) = db::create_opponent_with_players(
        &d,
        &t,
        "Rivals".into(),
        None,
        vec![("top".into(), Some("Someone#EUW".into()))],
    )
    .await
    .unwrap();
    db::update_opponent_player_intel(
        &d,
        &t,
        &players[0],
        Some("puuid".into()),
        vec![],
        None,
        None,
    )
    .await
    .unwrap();
    let (_, loaded) = db::get_opponent(&d, &t, &opp).await.unwrap().unwrap();
    assert_eq!(loaded.len(), 1);
    assert!(is_rfc3339(loaded[0].last_fetched.as_deref().unwrap()));

    db::create_bug_report(
        &d,
        &u,
        "/draft".into(),
        "Save".into(),
        "It broke".into(),
        "bug".into(),
        None,
        None,
    )
    .await
    .unwrap();
    let reports = db::list_bug_reports(&d, None).await.unwrap();
    assert_eq!(reports.len(), 1);
    assert!(is_rfc3339(reports[0].created_at.as_deref().unwrap()));

    let expiry = chrono::Utc::now() + chrono::Duration::days(3);
    db::create_invite(&d, None, "DATE-TEST-CODE".into(), 2, Some(expiry), None)
        .await
        .unwrap();
    let invites = db::list_invites(&d).await.unwrap();
    assert_eq!(invites.len(), 1);
    assert!(is_rfc3339(&invites[0].created_at));
    assert!(is_rfc3339(invites[0].expires_at.as_deref().unwrap()));
    assert_eq!(invites[0].status(&chrono::Utc::now().to_rfc3339()), "open");
}

#[tokio::test]
async fn lp_history_window_filters_by_time() {
    let d = common::test_db().await;
    let (u, _) = user_and_team(&d).await;
    db::store_ranked_snapshot(&d, &u, "RANKED_SOLO_5x5", "GOLD", "II", 40, 10, 8)
        .await
        .unwrap();

    let all = db::get_lp_history(&d, &u, None).await.unwrap();
    assert_eq!(all.len(), 1);
    assert!(is_rfc3339(&all[0].snapshotted_at));

    let past = (chrono::Utc::now() - chrono::Duration::days(7)).to_rfc3339();
    assert_eq!(
        db::get_lp_history(&d, &u, Some(past)).await.unwrap().len(),
        1
    );

    let future = (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc3339();
    assert!(db::get_lp_history(&d, &u, Some(future))
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn champion_trend_window_filters_by_game_end() {
    let d = common::test_db().await;
    let (u, _) = user_and_team(&d).await;
    let game = |id: &str, days_ago: i64| MatchData {
        match_id: id.into(),
        queue_id: 420,
        game_duration: 1800,
        game_end_epoch_ms: Some(
            (chrono::Utc::now() - chrono::Duration::days(days_ago)).timestamp_millis(),
        ),
        champion: "Ahri".into(),
        kills: 5,
        deaths: 2,
        assists: 7,
        cs: 200,
        vision_score: 20,
        damage: 15000,
        win: true,
    };
    db::store_matches(&d, &u, vec![game("EUW1_1", 2), game("EUW1_2", 40)])
        .await
        .unwrap();

    let all = db::get_champion_trends(&d, &u, None).await.unwrap();
    assert_eq!(all[0].games, 2);

    let cutoff = (chrono::Utc::now() - chrono::Duration::days(7)).to_rfc3339();
    let recent = db::get_champion_trends(&d, &u, Some(cutoff)).await.unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].games, 1, "only the game inside the window counts");
}
