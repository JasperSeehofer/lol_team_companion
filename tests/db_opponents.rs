#![cfg(feature = "ssr")]

mod common;

use lol_team_companion::server::db;

async fn setup(
    db: &surrealdb::Surreal<surrealdb::engine::local::Db>,
) -> (String, String) {
    let user_id = db::create_user(
        db,
        "scout".into(),
        "scout@example.com".into(),
        "hashed".into(),
    )
    .await
    .unwrap();
    let team_id = db::create_team(db, &user_id, "ScoutTeam".into(), "EUW".into())
        .await
        .unwrap();
    (user_id, team_id)
}

#[tokio::test]
async fn test_create_opponent_with_players() {
    let db = common::test_db().await;
    let (_user_id, team_id) = setup(&db).await;

    let players = vec![
        ("top".into(), Some("TopPlayer".into())),
        ("jungle".into(), Some("JunglePlayer".into())),
        ("mid".into(), Some("MidPlayer".into())),
        ("bottom".into(), Some("BotPlayer".into())),
        ("support".into(), Some("SupportPlayer".into())),
    ];

    let (opponent_id, player_ids) = db::create_opponent_with_players(
        &db,
        &team_id,
        "TestOpponent".into(),
        Some("Test notes".into()),
        players,
    )
    .await
    .unwrap();

    // Returns correct IDs
    assert!(opponent_id.starts_with("opponent:"), "opponent_id should start with 'opponent:'");
    assert_eq!(player_ids.len(), 5, "should create exactly 5 player slots");
    for pid in &player_ids {
        assert!(pid.starts_with("opponent_player:"), "player_id should start with 'opponent_player:'");
    }

    // Verify opponent + 5 players exist in the DB
    let detail = db::get_opponent(&db, &opponent_id).await.unwrap();
    assert!(detail.is_some(), "opponent should exist");
    let (opp, players_out) = detail.unwrap();
    assert_eq!(opp.name, "TestOpponent");
    assert_eq!(opp.notes, Some("Test notes".into()));
    assert_eq!(players_out.len(), 5, "should retrieve 5 players");

    // Verify roles and summoner names
    let roles: Vec<&str> = players_out.iter().map(|p| p.role.as_str()).collect();
    assert!(roles.contains(&"top"));
    assert!(roles.contains(&"jungle"));
    assert!(roles.contains(&"mid"));
    assert!(roles.contains(&"bottom"));
    assert!(roles.contains(&"support"));

    let summoners: Vec<Option<&String>> = players_out
        .iter()
        .map(|p| p.riot_summoner_name.as_ref())
        .collect();
    assert!(summoners.contains(&Some(&"TopPlayer".to_string())));
    assert!(summoners.contains(&Some(&"MidPlayer".to_string())));
}

#[tokio::test]
async fn test_update_opponent_player_intel() {
    let db = common::test_db().await;
    let (_user_id, team_id) = setup(&db).await;

    let players = vec![
        ("top".into(), None),
        ("jungle".into(), None),
        ("mid".into(), None),
        ("bottom".into(), None),
        ("support".into(), None),
    ];

    let (opponent_id, player_ids) = db::create_opponent_with_players(
        &db,
        &team_id,
        "IntelOpponent".into(),
        None,
        players,
    )
    .await
    .unwrap();

    let target_player_id = &player_ids[2]; // mid player

    let mastery_json = serde_json::to_string(&vec![
        ("Ahri".to_string(), 7i32, 500_000i32),
        ("Syndra".to_string(), 6i32, 200_000i32),
    ])
    .unwrap();

    let role_dist_json = serde_json::to_string(&vec![
        ("mid".to_string(), 8u32),
        ("support".to_string(), 2u32),
    ])
    .unwrap();

    db::update_opponent_player_intel(
        &db,
        target_player_id,
        Some("test-puuid-abc123".into()),
        vec!["Ahri".into(), "Ahri".into(), "Syndra".into()],
        Some(mastery_json),
        Some(role_dist_json),
    )
    .await
    .unwrap();

    // Retrieve and verify
    let detail = db::get_opponent(&db, &opponent_id).await.unwrap().unwrap();
    let updated_player = detail
        .1
        .into_iter()
        .find(|p| p.id.as_deref() == Some(target_player_id.as_str()))
        .expect("target player should exist");

    assert_eq!(
        updated_player.riot_puuid,
        Some("test-puuid-abc123".into())
    );
    assert_eq!(updated_player.recent_champions.len(), 3);
    assert!(
        updated_player.mastery_data_json.is_some(),
        "mastery_data_json should be set"
    );
    assert!(
        updated_player.role_distribution_json.is_some(),
        "role_distribution_json should be set"
    );
    assert!(
        updated_player.last_fetched.is_some(),
        "last_fetched should be set after intel update"
    );
}

#[tokio::test]
async fn test_create_opponent_with_empty_summoner_names() {
    let db = common::test_db().await;
    let (_user_id, team_id) = setup(&db).await;

    let players = vec![
        ("top".into(), None),
        ("jungle".into(), Some("JungleSummoner".into())),
        ("mid".into(), None),
        ("bottom".into(), None),
        ("support".into(), Some("SupportSummoner".into())),
    ];

    let (opponent_id, player_ids) = db::create_opponent_with_players(
        &db,
        &team_id,
        "PartialOpponent".into(),
        None,
        players,
    )
    .await
    .unwrap();

    assert_eq!(player_ids.len(), 5);

    let detail = db::get_opponent(&db, &opponent_id).await.unwrap().unwrap();
    let players_out = detail.1;
    assert_eq!(players_out.len(), 5);

    // Players with None summoner should have riot_summoner_name == None
    let none_count = players_out
        .iter()
        .filter(|p| p.riot_summoner_name.is_none())
        .count();
    let some_count = players_out
        .iter()
        .filter(|p| p.riot_summoner_name.is_some())
        .count();
    assert_eq!(none_count, 3, "3 players should have no summoner name");
    assert_eq!(some_count, 2, "2 players should have summoner names");
}
