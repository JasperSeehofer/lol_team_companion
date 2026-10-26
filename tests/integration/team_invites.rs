//! Tests for auto-placing members in starter slots via invite links.

#[tokio::test]
async fn auto_place_in_open_starter_slot() {
    let db = common::test_db().await;
    let lead = db::create_user(&db, "lead".into(), "lead@example.com".into(), "h".into())
        .await
        .unwrap();
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    
    // Create an invite for mid position
    db::create_team_invite(
        &db,
        &lead,
        &team,
        "TEAM-CODE".into(),
        2,
        soon(),
        Some("mid".into()),
        None,
    )
    .await
    .unwrap();
    
    // Register a new user with the invite
    let grant = db::consume_invite_grant(&db, "team-code")
        .await
        .unwrap()
        .unwrap();
    let carl = db::create_user(&db, "carl".into(), "carl@example.com".into(), "h".into())
        .await
        .unwrap();
    
    // Join the team - should be auto-placed as starter since mid slot is open
    let auto_placed_role = db::join_team_from_invite(&db, &carl, &grant).await.unwrap();
    assert_eq!(auto_placed_role, Some("mid".to_string()));
    
    // Check that the member is now a starter with auto_placed_at set
    let (_, members) = db::get_user_team_with_members(&db, &carl)
        .await
        .unwrap()
        .unwrap();
    let carl_row = members.iter().find(|m| m.user_id == carl).unwrap();
    assert_eq!(carl_row.role, "mid");
    assert_eq!(carl_row.roster_type, "starter");
    
    // Check that auto_placed_at is set
    let auto_placed = db::list_auto_placed(&db, &team).await.unwrap();
    assert_eq!(auto_placed.len(), 1);
    assert_eq!(auto_placed[0].0, carl);
    assert_eq!(auto_placed[0].1, "carl");
    assert_eq!(auto_placed[0].2, "mid");
}

#[tokio::test]
async fn auto_place_when_slot_taken() {
    let db = common::test_db().await;
    let lead = db::create_user(&db, "lead".into(), "lead@example.com".into(), "h".into())
        .await
        .unwrap();
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    
    // Create an invite for mid position
    db::create_team_invite(
        &db,
        &lead,
        &team,
        "TEAM-CODE".into(),
        2,
        soon(),
        Some("mid".into()),
        None,
    )
    .await
    .unwrap();
    
    // First, place someone in the mid slot
    let existing = db::create_user(&db, "existing".into(), "existing@example.com".into(), "h".into())
        .await
        .unwrap();
    db::join_team(&db, &existing, &team).await.unwrap();
    db::set_member_place(&db, &team, &existing, db::RosterPlace::Starter("mid".into()))
        .await
        .unwrap();
    
    // Register a new user with the invite
    let grant = db::consume_invite_grant(&db, "team-code")
        .await
        .unwrap()
        .unwrap();
    let carl = db::create_user(&db, "carl".into(), "carl@example.com".into(), "h".into())
        .await
        .unwrap();
    
    // Join the team - should NOT be auto-placed since mid slot is taken
    let auto_placed_role = db::join_team_from_invite(&db, &carl, &grant).await.unwrap();
    assert_eq!(auto_placed_role, None);
    
    // Check that the member is a regular member, not a starter
    let (_, members) = db::get_user_team_with_members(&db, &carl)
        .await
        .unwrap()
        .unwrap();
    let carl_row = members.iter().find(|m| m.user_id == carl).unwrap();
    assert_eq!(carl_row.role, "mid");
    assert_eq!(carl_row.roster_type, "member");
    
    // Check that auto_placed list is empty
    let auto_placed = db::list_auto_placed(&db, &team).await.unwrap();
    assert_eq!(auto_placed.len(), 0);
}

#[tokio::test]
async fn auto_place_via_redeem_team_invite() {
    let db = common::test_db().await;
    let lead = db::create_user(&db, "lead".into(), "lead@example.com".into(), "h".into())
        .await
        .unwrap();
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    
    // Create an invite for mid position
    db::create_team_invite(
        &db,
        &lead,
        &team,
        "TEAM-CODE".into(),
        2,
        soon(),
        Some("mid".into()),
        None,
    )
    .await
    .unwrap();
    
    // Create a user and redeem the invite
    let carl = db::create_user(&db, "carl".into(), "carl@example.com".into(), "h".into())
        .await
        .unwrap();
    
    // Redeem the invite - should be auto-placed as starter since mid slot is open
    let (team_id, auto_placed_role) = db::redeem_team_invite(&db, &carl, "team-code").await.unwrap();
    assert_eq!(team_id, team);
    assert_eq!(auto_placed_role, Some("mid".to_string()));
    
    // Check that the member is now a starter
    let (_, members) = db::get_user_team_with_members(&db, &carl)
        .await
        .unwrap()
        .unwrap();
    let carl_row = members.iter().find(|m| m.user_id == carl).unwrap();
    assert_eq!(carl_row.role, "mid");
    assert_eq!(carl_row.roster_type, "starter");
    
    // Check that auto_placed_at is set
    let auto_placed = db::list_auto_placed(&db, &team).await.unwrap();
    assert_eq!(auto_placed.len(), 1);
    assert_eq!(auto_placed[0].0, carl);
    assert_eq!(auto_placed[0].1, "carl");
    assert_eq!(auto_placed[0].2, "mid");
}

#[tokio::test]
async fn clear_auto_placed() {
    let db = common::test_db().await;
    let lead = db::create_user(&db, "lead".into(), "lead@example.com".into(), "h".into())
        .await
        .unwrap();
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    
    // Create an invite for mid position and auto-place a member
    db::create_team_invite(
        &db,
        &lead,
        &team,
        "TEAM-CODE".into(),
        2,
        soon(),
        Some("mid".into()),
        None,
    )
    .await
    .unwrap();
    
    let grant = db::consume_invite_grant(&db, "team-code")
        .await
        .unwrap()
        .unwrap();
    let carl = db::create_user(&db, "carl".into(), "carl@example.com".into(), "h".into())
        .await
        .unwrap();
    db::join_team_from_invite(&db, &carl, &grant).await.unwrap();
    
    // Verify auto-placed
    let auto_placed = db::list_auto_placed(&db, &team).await.unwrap();
    assert_eq!(auto_placed.len(), 1);
    
    // Clear with undo=true - should move to member and clear auto_placed_at
    db::clear_auto_placed(&db, &team, &carl, true).await.unwrap();
    
    // Check that the member is now a regular member
    let (_, members) = db::get_user_team_with_members(&db, &carl)
        .await
        .unwrap()
        .unwrap();
    let carl_row = members.iter().find(|m| m.user_id == carl).unwrap();
    assert_eq!(carl_row.role, "mid");
    assert_eq!(carl_row.roster_type, "member");
    
    // Check that auto_placed list is empty
    let auto_placed = db::list_auto_placed(&db, &team).await.unwrap();
    assert_eq!(auto_placed.len(), 0);
    
    // Auto-place again
    db::set_member_place(&db, &team, &carl, db::RosterPlace::Starter("mid".into()))
        .await
        .unwrap();
    let mut r = db.query("UPDATE team_member SET auto_placed_at = time::now() WHERE user = type::record('user', $user_key) AND team = type::record('team', $team_key)")
        .bind(("user_key", db::record_key(&carl, "user")))
        .bind(("team_key", db::record_key(&team, "team")))
        .await
        .unwrap();
    r.check().unwrap();
    
    // Verify auto-placed again
    let auto_placed = db::list_auto_placed(&db, &team).await.unwrap();
    assert_eq!(auto_placed.len(), 1);
    
    // Clear with undo=false - should keep as starter but clear auto_placed_at
    db::clear_auto_placed(&db, &team, &carl, false).await.unwrap();
    
    // Check that the member is still a starter but not auto-placed
    let (_, members) = db::get_user_team_with_members(&db, &carl)
        .await
        .unwrap()
        .unwrap();
    let carl_row = members.iter().find(|m| m.user_id == carl).unwrap();
    assert_eq!(carl_row.role, "mid");
    assert_eq!(carl_row.roster_type, "starter");
    
    // Check that auto_placed list is empty
    let auto_placed = db::list_auto_placed(&db, &team).await.unwrap();
    assert_eq!(auto_placed.len(), 0);
}

#[tokio::test]
async fn auto_place_cross_tenant() {
    let db = common::test_db().await;
    let lead = db::create_user(&db, "lead".into(), "lead@example.com".into(), "h".into())
        .await
        .unwrap();
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    
    let lead_b = db::create_user(&db, "lead_b".into(), "lead_b@example.com".into(), "h".into())
        .await
        .unwrap();
    let team_b = db::create_team(&db, &lead_b, "Riverside".into(), "EUW".into())
        .await
        .unwrap();
    
    // Create an invite for mid position in team A
    db::create_team_invite(
        &db,
        &lead,
        &team,
        "TEAM-CODE".into(),
        2,
        soon(),
        Some("mid".into()),
        None,
    )
    .await
    .unwrap();
    
    // Try to list auto-placed members for team B - should be empty
    let auto_placed = db::list_auto_placed(&db, &team_b).await.unwrap();
    assert_eq!(auto_placed.len(), 0);
    
    // Try to clear auto-placed for a user not in team B - should return NotFound
    assert!(matches!(
        db::clear_auto_placed(&db, &team_b, &lead, true).await,
        Err(DbError::NotFound)
    ));
}

fn soon() -> Option<chrono::DateTime<chrono::Utc>> {
    Some(chrono::Utc::now() + chrono::Duration::days(7))
}