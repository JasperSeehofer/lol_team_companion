#![cfg(feature = "ssr")]

mod common;

use lol_team_companion::server::db;

async fn make_user(db: &surrealdb::Surreal<surrealdb::engine::local::Db>, name: &str) -> String {
    db::create_user(
        db,
        name.into(),
        format!("{name}@example.com"),
        "hash".into(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn test_create_team_and_get_it() {
    let db = common::test_db().await;
    let user_id = make_user(&db, "captain").await;
    let team_id = db::create_team(&db, &user_id, "Alpha".into(), "EUW".into())
        .await
        .unwrap();
    assert!(team_id.starts_with("team:"));

    let team_id_lookup = db::get_user_team_id(&db, &user_id).await.unwrap();
    assert_eq!(team_id_lookup, Some(team_id));
}

#[tokio::test]
async fn test_join_team_success() {
    let db = common::test_db().await;
    let u1 = make_user(&db, "u1").await;
    let u2 = make_user(&db, "u2").await;
    let team_id = db::create_team(&db, &u1, "Beta".into(), "KR".into())
        .await
        .unwrap();
    db::join_team(&db, &u2, &team_id).await.unwrap();
    let team_id_for_u2 = db::get_user_team_id(&db, &u2).await.unwrap();
    assert_eq!(team_id_for_u2, Some(team_id));
}

#[tokio::test]
async fn test_join_team_already_member_fails() {
    let db = common::test_db().await;
    let u1 = make_user(&db, "dup_member").await;
    let team_id = db::create_team(&db, &u1, "Gamma".into(), "NA".into())
        .await
        .unwrap();
    // u1 is already a member from create_team
    let result = db::join_team(&db, &u1, &team_id).await;
    assert!(
        result.is_err(),
        "joining a team you're already in should fail"
    );
}

#[tokio::test]
async fn test_join_request_flow_accept() {
    let db = common::test_db().await;
    let owner = make_user(&db, "owner_accept").await;
    let joiner = make_user(&db, "joiner_accept").await;
    let team_id = db::create_team(&db, &owner, "Delta".into(), "EUW".into())
        .await
        .unwrap();

    db::create_join_request(&db, &joiner, &team_id)
        .await
        .unwrap();
    let requests = db::list_pending_join_requests(&db, &team_id).await.unwrap();
    assert_eq!(requests.len(), 1);
    let req_id = &requests[0].id;

    db::respond_to_join_request(&db, req_id, true, &team_id)
        .await
        .unwrap();

    // joiner should now be on the team
    let team_id_for_joiner = db::get_user_team_id(&db, &joiner).await.unwrap();
    assert_eq!(team_id_for_joiner, Some(team_id.clone()));

    // no more pending requests
    let remaining = db::list_pending_join_requests(&db, &team_id).await.unwrap();
    assert!(remaining.is_empty());
}

#[tokio::test]
async fn test_join_request_flow_decline() {
    let db = common::test_db().await;
    let owner = make_user(&db, "owner_decline").await;
    let joiner = make_user(&db, "joiner_decline").await;
    let team_id = db::create_team(&db, &owner, "Epsilon".into(), "EUW".into())
        .await
        .unwrap();

    db::create_join_request(&db, &joiner, &team_id)
        .await
        .unwrap();
    let requests = db::list_pending_join_requests(&db, &team_id).await.unwrap();
    let req_id = &requests[0].id;

    db::respond_to_join_request(&db, req_id, false, &team_id)
        .await
        .unwrap();

    // joiner should NOT be on the team
    let team_id_for_joiner = db::get_user_team_id(&db, &joiner).await.unwrap();
    assert!(team_id_for_joiner.is_none());
}

#[tokio::test]
async fn test_join_request_duplicate_fails() {
    let db = common::test_db().await;
    let owner = make_user(&db, "owner_dup_req").await;
    let joiner = make_user(&db, "joiner_dup_req").await;
    let team_id = db::create_team(&db, &owner, "Zeta".into(), "EUW".into())
        .await
        .unwrap();

    db::create_join_request(&db, &joiner, &team_id)
        .await
        .unwrap();
    let result = db::create_join_request(&db, &joiner, &team_id).await;
    assert!(result.is_err(), "duplicate join request should fail");
}

#[tokio::test]
async fn test_assign_to_slot_bumps_existing_starter() {
    let db = common::test_db().await;
    let owner = make_user(&db, "slot_owner").await;
    let u2 = make_user(&db, "slot_u2").await;
    let u3 = make_user(&db, "slot_u3").await;
    let team_id = db::create_team(&db, &owner, "Eta".into(), "EUW".into())
        .await
        .unwrap();
    db::join_team(&db, &u2, &team_id).await.unwrap();
    db::join_team(&db, &u3, &team_id).await.unwrap();

    // Assign u2 as mid starter
    db::assign_to_slot(&db, &team_id, &u2, "mid").await.unwrap();
    // Assign u3 as mid starter — should bump u2 to sub
    db::assign_to_slot(&db, &team_id, &u3, "mid").await.unwrap();

    let (_, members) = db::get_user_team_with_members(&db, &owner)
        .await
        .unwrap()
        .unwrap();

    let u2_member = members.iter().find(|m| {
        m.user_id.contains("slot_u2") || {
            // user_id is a RecordId SQL string like "user:xxx"
            m.username == "slot_u2"
        }
    });
    let u3_member = members.iter().find(|m| m.username == "slot_u3");

    if let (Some(u2m), Some(u3m)) = (u2_member, u3_member) {
        assert_eq!(u2m.roster_type, "sub", "u2 should be bumped to sub");
        assert_eq!(u3m.roster_type, "starter", "u3 should be starter");
        assert_eq!(u3m.role, "mid");
    } else {
        panic!("could not find expected members in team");
    }
}

#[tokio::test]
async fn test_remove_from_slot() {
    let db = common::test_db().await;
    let owner = make_user(&db, "remove_slot_owner").await;
    let player = make_user(&db, "remove_slot_player").await;
    let team_id = db::create_team(&db, &owner, "Theta".into(), "EUW".into())
        .await
        .unwrap();
    db::join_team(&db, &player, &team_id).await.unwrap();
    db::assign_to_slot(&db, &team_id, &player, "top")
        .await
        .unwrap();
    db::remove_from_slot(&db, &team_id, &player).await.unwrap();

    let (_, members) = db::get_user_team_with_members(&db, &owner)
        .await
        .unwrap()
        .unwrap();
    let player_member = members.iter().find(|m| m.username == "remove_slot_player");
    assert_eq!(
        player_member.map(|m| m.roster_type.as_str()),
        Some("sub"),
        "player should be sub after removal"
    );
}
