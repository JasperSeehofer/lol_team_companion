//! Team-bound invites (polish pass): a code that registers an account into
//! a team, scoped to that team for listing and disabling, unusable by an
//! account that already belongs to a team, and gone with the team.

use crate::common;

use lol_team_companion::server::db::{self, DbError, InviteGrant};

fn soon() -> Option<chrono::DateTime<chrono::Utc>> {
    Some(chrono::Utc::now() + chrono::Duration::days(7))
}

#[tokio::test]
async fn a_team_code_registers_the_account_into_the_team() {
    let db = common::test_db().await;
    let lead = db::create_user(&db, "lead".into(), "lead@example.com".into(), "h".into())
        .await
        .unwrap();
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
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
    assert_eq!(
        db::team_invite_name(&db, "team-code")
            .await
            .unwrap()
            .as_deref(),
        Some("Northgate")
    );

    // Registration: one use, then the new account joins the team.
    let grant = db::consume_invite_grant(&db, " team-code ")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(grant.team_id.as_deref(), Some(team.as_str()));
    assert_eq!(grant.position.as_deref(), Some("mid"));
    let carl = db::create_user(&db, "carl".into(), "carl@example.com".into(), "h".into())
        .await
        .unwrap();
    assert_eq!(
        db::join_team_from_invite(&db, &carl, &grant)
            .await
            .unwrap()
            .as_deref(),
        Some("mid")
    );
    assert_eq!(
        db::get_user_team_id(&db, &carl).await.unwrap().as_deref(),
        Some(team.as_str())
    );
    let (_, members) = db::get_user_team_with_members(&db, &carl)
        .await
        .unwrap()
        .unwrap();
    let carl_row = members.iter().find(|m| m.user_id == carl).unwrap();
    assert_eq!(carl_row.role, "mid", "the code's position");
    // M160: the mid starter slot was open, so the code placed carl there
    // (the leader or a coach can undo it; see `auto_place_*` below).
    assert_eq!(
        carl_row.roster_type, "starter",
        "the open slot at the code's position"
    );

    // A plain code grants nothing beyond the account.
    db::create_invite(&db, None, "PLAIN".into(), 1, None, None)
        .await
        .unwrap();
    assert_eq!(
        db::consume_invite_grant(&db, "plain").await.unwrap(),
        Some(InviteGrant::default())
    );
    assert_eq!(db::team_invite_name(&db, "plain").await.unwrap(), None);

    let invites = db::list_team_invites(&db, &team).await.unwrap();
    assert_eq!(invites.len(), 1);
    assert_eq!(invites[0].uses, 1);
    assert_eq!(invites[0].team_name.as_deref(), Some("Northgate"));
}

#[tokio::test]
async fn a_team_code_cannot_be_used_by_someone_already_in_a_team() {
    let db = common::test_db().await;
    let lead = db::create_user(&db, "lead".into(), "lead@example.com".into(), "h".into())
        .await
        .unwrap();
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    let other = db::create_user(&db, "other".into(), "other@example.com".into(), "h".into())
        .await
        .unwrap();
    db::create_team(&db, &other, "Riverside".into(), "EUW".into())
        .await
        .unwrap();
    db::create_team_invite(&db, &lead, &team, "JOIN-ME".into(), 3, soon(), None, None)
        .await
        .unwrap();

    // Already on a team: refused, and no use is taken.
    for who in [&other, &lead] {
        match db::redeem_team_invite(&db, who, "JOIN-ME").await {
            Err(DbError::Invalid(m)) => assert!(m.contains("already belong"), "{m}"),
            r => panic!("expected a refusal, got {r:?}"),
        }
    }
    assert_eq!(db::list_team_invites(&db, &team).await.unwrap()[0].uses, 0);

    // A signed-in account without a team joins with it.
    let dana = db::create_user(&db, "dana".into(), "dana@example.com".into(), "h".into())
        .await
        .unwrap();
    let joined = db::redeem_team_invite(&db, &dana, "join-me").await.unwrap();
    assert_eq!(joined.team_id, team);
    assert_eq!(joined.auto_placed, None, "a code without a position");
    assert_eq!(
        db::get_user_team_id(&db, &dana).await.unwrap().as_deref(),
        Some(team.as_str())
    );
    assert_eq!(db::list_team_invites(&db, &team).await.unwrap()[0].uses, 1);
    // ...but only once: now dana is on a team.
    assert!(matches!(
        db::redeem_team_invite(&db, &dana, "join-me").await,
        Err(DbError::Invalid(_))
    ));

    // A plain code, an unknown one or a disabled team code puts nobody in a team.
    db::create_invite(&db, None, "PLAIN-ONE".into(), 5, None, None)
        .await
        .unwrap();
    let eve = db::create_user(&db, "eve".into(), "eve@example.com".into(), "h".into())
        .await
        .unwrap();
    for code in ["PLAIN-ONE", "NOPE", ""] {
        match db::redeem_team_invite(&db, &eve, code).await {
            Err(DbError::Invalid(m)) => assert_eq!(m, db::INVALID_TEAM_INVITE),
            r => panic!("{code}: expected a refusal, got {r:?}"),
        }
    }
    let plain = db::list_invites(&db).await.unwrap();
    assert_eq!(
        plain.iter().find(|i| i.code == "PLAIN-ONE").unwrap().uses,
        0
    );
    let id = db::list_team_invites(&db, &team).await.unwrap()[0]
        .id
        .clone();
    db::disable_team_invite(&db, &team, &id).await.unwrap();
    assert!(db::redeem_team_invite(&db, &eve, "JOIN-ME").await.is_err());
    assert_eq!(db::get_user_team_id(&db, &eve).await.unwrap(), None);
}

#[tokio::test]
async fn deleting_the_team_removes_its_codes() {
    let db = common::test_db().await;
    let lead = db::create_user(&db, "lead".into(), "lead@example.com".into(), "h".into())
        .await
        .unwrap();
    let team = db::create_team(&db, &lead, "Solo".into(), "EUW".into())
        .await
        .unwrap();
    db::create_team_invite(&db, &lead, &team, "GONE-SOON".into(), 1, soon(), None, None)
        .await
        .unwrap();
    db::create_invite(&db, Some(&lead), "GLOBAL".into(), 1, None, None)
        .await
        .unwrap();
    lol_team_companion::server::account::delete_account(&db, &lead)
        .await
        .unwrap();
    let left = db::list_invites(&db).await.unwrap();
    assert_eq!(left.len(), 1, "{left:?}");
    assert_eq!(left[0].code, "GLOBAL");
    assert!(db::consume_invite_grant(&db, "GONE-SOON")
        .await
        .unwrap()
        .is_none());
}

/// M10 review: a use taken just before the team is deleted must not put
/// the new account into a team that no longer exists.
#[tokio::test]
async fn joining_a_team_deleted_after_the_use_was_taken_is_refused() {
    let db = common::test_db().await;
    let lead = db::create_user(&db, "lead".into(), "lead@example.com".into(), "h".into())
        .await
        .unwrap();
    let team = db::create_team(&db, &lead, "Brief".into(), "EUW".into())
        .await
        .unwrap();
    db::create_team_invite(&db, &lead, &team, "LAST-ONE".into(), 1, soon(), None, None)
        .await
        .unwrap();
    let grant = db::consume_invite_grant(&db, "LAST-ONE")
        .await
        .unwrap()
        .unwrap();
    lol_team_companion::server::account::delete_account(&db, &lead)
        .await
        .unwrap();
    let dana = db::create_user(&db, "dana".into(), "dana@example.com".into(), "h".into())
        .await
        .unwrap();
    assert!(matches!(
        db::join_team_from_invite(&db, &dana, &grant).await,
        Err(DbError::NotFound)
    ));
    assert_eq!(db::get_user_team_id(&db, &dana).await.unwrap(), None);
    // A grant without a team is a no-op.
    assert_eq!(
        db::join_team_from_invite(&db, &dana, &InviteGrant::default())
            .await
            .unwrap(),
        None
    );
}

/// M148 item 2: who may manage a team's invite links. The server functions
/// (`mint_team_invite`, `list_team_invites`, `disable_team_invite`) take the
/// team from the session (`db::get_user_team_id`) and allow it only when
/// `db::is_team_leader_or_coach` holds; disabling stays scoped to that team.
#[tokio::test]
async fn the_leader_and_coaches_manage_invites_members_do_not() {
    let db = common::test_db().await;
    let user = |name: &str| (name.to_string(), format!("{name}@example.com"));
    let mut ids = Vec::new();
    for name in ["lead", "coach", "player", "sub", "lead_b", "coach_b"] {
        let (u, e) = user(name);
        ids.push(db::create_user(&db, u, e, "h".into()).await.unwrap());
    }
    let [lead, coach, player, sub, lead_b, coach_b]: [String; 6] = ids.try_into().unwrap();
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    let team_b = db::create_team(&db, &lead_b, "Riverside".into(), "EUW".into())
        .await
        .unwrap();
    for u in [&coach, &player, &sub] {
        db::join_team(&db, u, &team).await.unwrap();
    }
    db::join_team(&db, &coach_b, &team_b).await.unwrap();
    db::set_member_place(&db, &team, &coach, db::RosterPlace::Coach)
        .await
        .unwrap();
    db::set_member_place(&db, &team, &sub, db::RosterPlace::Substitute)
        .await
        .unwrap();
    db::set_member_place(&db, &team_b, &coach_b, db::RosterPlace::Coach)
        .await
        .unwrap();

    // The rights check, as the server functions run it.
    assert!(db::is_team_leader_or_coach(&db, &lead, &team)
        .await
        .unwrap());
    assert!(db::is_team_leader_or_coach(&db, &coach, &team)
        .await
        .unwrap());
    assert!(!db::is_team_leader_or_coach(&db, &player, &team)
        .await
        .unwrap());
    assert!(!db::is_team_leader_or_coach(&db, &sub, &team).await.unwrap());
    // Another team's coach or leader has no rights on this team.
    assert!(!db::is_team_leader_or_coach(&db, &coach_b, &team)
        .await
        .unwrap());
    assert!(!db::is_team_leader_or_coach(&db, &lead_b, &team)
        .await
        .unwrap());
    assert!(db::is_team_coach(&db, &coach, &team).await.unwrap());
    assert!(!db::is_team_coach(&db, &coach, &team_b).await.unwrap());

    // A coach mints (the team from their own membership), lists and disables.
    let coach_team = db::get_user_team_id(&db, &coach).await.unwrap().unwrap();
    assert_eq!(coach_team, team);
    db::create_team_invite(
        &db,
        &coach,
        &coach_team,
        "COACH-CODE".into(),
        3,
        soon(),
        None,
        None,
    )
    .await
    .unwrap();
    let listed = db::list_team_invites(&db, &coach_team).await.unwrap();
    assert_eq!(listed.len(), 1);
    let id = listed[0].id.clone();
    assert_eq!(listed[0].code, "COACH-CODE");
    assert_eq!(listed[0].team_id.as_deref(), Some(team.as_str()));

    // Team B's coach cannot disable it through their own team (the scope a
    // server function uses), nor list it.
    let b_team = db::get_user_team_id(&db, &coach_b).await.unwrap().unwrap();
    assert!(matches!(
        db::disable_team_invite(&db, &b_team, &id).await,
        Err(DbError::NotFound)
    ));
    assert!(db::list_team_invites(&db, &b_team)
        .await
        .unwrap()
        .is_empty());
    assert!(db::consume_invite_grant(&db, "COACH-CODE")
        .await
        .unwrap()
        .is_some());

    // The coach disables it; the leader still mints and disables their own.
    db::disable_team_invite(&db, &coach_team, &id)
        .await
        .unwrap();
    assert!(db::list_team_invites(&db, &team).await.unwrap()[0].disabled);
    db::create_team_invite(&db, &lead, &team, "LEAD-CODE".into(), 1, soon(), None, None)
        .await
        .unwrap();
    let lead_id = db::list_team_invites(&db, &team)
        .await
        .unwrap()
        .into_iter()
        .find(|i| i.code == "LEAD-CODE")
        .unwrap()
        .id;
    db::disable_team_invite(&db, &team, &lead_id).await.unwrap();

    // A coach moved back to a plain member loses the right at once.
    db::set_member_place(&db, &team, &coach, db::RosterPlace::Member)
        .await
        .unwrap();
    assert!(!db::is_team_leader_or_coach(&db, &coach, &team)
        .await
        .unwrap());
}

// ---------------------------------------------------------------------------
// M160 item 2b: an invited joiner takes their open starter slot
// ---------------------------------------------------------------------------

/// The place and the auto-placed mark of `user` in the team.
async fn place_of(
    db: &surrealdb::Surreal<surrealdb::engine::local::Db>,
    user: &str,
) -> (String, bool) {
    let (_, members) = db::get_user_team_with_members(db, user)
        .await
        .unwrap()
        .unwrap();
    let place = members
        .iter()
        .find(|m| m.user_id == user)
        .unwrap()
        .roster_type
        .clone();
    let key = user.strip_prefix("user:").unwrap_or(user).to_string();
    let mut r = db
        .query(
            "SELECT VALUE auto_placed_at != NONE FROM team_member \
             WHERE user = type::record('user', $k)",
        )
        .bind(("k", key))
        .await
        .unwrap();
    let marked: Vec<bool> = r.take(0).unwrap();
    (place, marked.first().copied().unwrap_or(false))
}

async fn new_user(db: &surrealdb::Surreal<surrealdb::engine::local::Db>, name: &str) -> String {
    db::create_user(db, name.into(), format!("{name}@example.com"), "h".into())
        .await
        .unwrap()
}

async fn mid_code(
    db: &surrealdb::Surreal<surrealdb::engine::local::Db>,
    lead: &str,
    team: &str,
    code: &str,
) {
    db::create_team_invite(
        db,
        lead,
        team,
        code.into(),
        5,
        soon(),
        Some("mid".into()),
        None,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn auto_place_in_open_starter_slot() {
    let db = common::test_db().await;
    let lead = new_user(&db, "lead").await;
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    mid_code(&db, &lead, &team, "MID-OPEN").await;
    let grant = db::consume_invite_grant(&db, "MID-OPEN")
        .await
        .unwrap()
        .unwrap();
    let carl = new_user(&db, "carl").await;
    let placed = db::join_team_from_invite(&db, &carl, &grant).await.unwrap();
    assert_eq!(placed.as_deref(), Some("mid"));
    assert_eq!(place_of(&db, &carl).await, ("starter".to_string(), true));
    let listed = db::list_auto_placed(&db, &team).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].user_id, carl);
    assert_eq!(listed[0].username, "carl");
    assert_eq!(listed[0].role, "mid");
    // The leader is never marked.
    assert!(!place_of(&db, &lead).await.1);
}

#[tokio::test]
async fn auto_place_when_slot_taken() {
    let db = common::test_db().await;
    let lead = new_user(&db, "lead").await;
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    let mia = new_user(&db, "mia").await;
    db::join_team(&db, &mia, &team).await.unwrap();
    db::assign_to_slot(&db, &team, &mia, "mid").await.unwrap();
    mid_code(&db, &lead, &team, "MID-TAKEN").await;
    let grant = db::consume_invite_grant(&db, "MID-TAKEN")
        .await
        .unwrap()
        .unwrap();
    let carl = new_user(&db, "carl").await;
    assert_eq!(
        db::join_team_from_invite(&db, &carl, &grant).await.unwrap(),
        None
    );
    assert_eq!(place_of(&db, &carl).await, ("member".to_string(), false));
    // The starter stays where they were.
    assert_eq!(place_of(&db, &mia).await, ("starter".to_string(), false));
    assert!(db::list_auto_placed(&db, &team).await.unwrap().is_empty());

    // A code without a position never places anyone.
    db::create_team_invite(&db, &lead, &team, "NO-POS".into(), 2, soon(), None, None)
        .await
        .unwrap();
    let dana = new_user(&db, "dana").await;
    let joined = db::redeem_team_invite(&db, &dana, "NO-POS").await.unwrap();
    assert_eq!(joined.auto_placed, None);
    assert_eq!(place_of(&db, &dana).await, ("member".to_string(), false));
}

#[tokio::test]
async fn auto_place_via_redeem_team_invite() {
    let db = common::test_db().await;
    let lead = new_user(&db, "lead").await;
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    mid_code(&db, &lead, &team, "MID-REDEEM").await;
    let dana = new_user(&db, "dana").await;
    let joined = db::redeem_team_invite(&db, &dana, "mid-redeem")
        .await
        .unwrap();
    assert_eq!(joined.team_id, team);
    assert_eq!(joined.auto_placed.as_deref(), Some("mid"));
    assert_eq!(place_of(&db, &dana).await, ("starter".to_string(), true));
    let listed = db::list_auto_placed(&db, &team).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].user_id, dana);

    // The slot is now taken: the next joiner with the same code waits.
    let eve = new_user(&db, "eve").await;
    let joined = db::redeem_team_invite(&db, &eve, "MID-REDEEM")
        .await
        .unwrap();
    assert_eq!(joined.auto_placed, None);
    assert_eq!(place_of(&db, &eve).await, ("member".to_string(), false));
    assert_eq!(db::list_auto_placed(&db, &team).await.unwrap().len(), 1);
    // The existing refusals are unchanged.
    match db::redeem_team_invite(&db, &eve, "MID-REDEEM").await {
        Err(DbError::Invalid(m)) => assert!(m.contains("already belong"), "{m}"),
        r => panic!("expected a refusal, got {r:?}"),
    }
}

#[tokio::test]
async fn clear_auto_placed_undo_and_keep() {
    let db = common::test_db().await;
    let lead = new_user(&db, "lead").await;
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    db::create_team_invite(
        &db,
        &lead,
        &team,
        "TOP-CODE".into(),
        2,
        soon(),
        Some("top".into()),
        None,
    )
    .await
    .unwrap();
    mid_code(&db, &lead, &team, "MID-CODE").await;
    let tom = new_user(&db, "tom").await;
    let mo = new_user(&db, "mo").await;
    assert_eq!(
        db::redeem_team_invite(&db, &tom, "TOP-CODE")
            .await
            .unwrap()
            .auto_placed
            .as_deref(),
        Some("top")
    );
    assert_eq!(
        db::redeem_team_invite(&db, &mo, "MID-CODE")
            .await
            .unwrap()
            .auto_placed
            .as_deref(),
        Some("mid")
    );
    assert_eq!(db::list_auto_placed(&db, &team).await.unwrap().len(), 2);

    // Undo: tom moves to "not placed yet" and the mark is gone.
    db::clear_auto_placed(&db, &team, &tom, true).await.unwrap();
    assert_eq!(place_of(&db, &tom).await, ("member".to_string(), false));
    // Keep: mo stays the mid starter, the mark is gone.
    db::clear_auto_placed(&db, &team, &mo, false).await.unwrap();
    assert_eq!(place_of(&db, &mo).await, ("starter".to_string(), false));
    assert!(db::list_auto_placed(&db, &team).await.unwrap().is_empty());

    // Answered once: a second answer is NotFound and changes nothing.
    assert!(matches!(
        db::clear_auto_placed(&db, &team, &mo, true).await,
        Err(DbError::NotFound)
    ));
    assert_eq!(place_of(&db, &mo).await, ("starter".to_string(), false));
    // Nobody who was not placed automatically can be moved with it: the
    // leader, or a starter placed by hand.
    assert!(matches!(
        db::clear_auto_placed(&db, &team, &lead, true).await,
        Err(DbError::NotFound)
    ));
    let sam = new_user(&db, "sam").await;
    db::join_team(&db, &sam, &team).await.unwrap();
    db::assign_to_slot(&db, &team, &sam, "support")
        .await
        .unwrap();
    assert!(matches!(
        db::clear_auto_placed(&db, &team, &sam, true).await,
        Err(DbError::NotFound)
    ));
    assert_eq!(place_of(&db, &sam).await, ("starter".to_string(), false));
}

#[tokio::test]
async fn auto_place_cross_tenant() {
    let db = common::test_db().await;
    let lead = new_user(&db, "lead").await;
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    let lead_b = new_user(&db, "lead_b").await;
    let team_b = db::create_team(&db, &lead_b, "Riverside".into(), "EUW".into())
        .await
        .unwrap();
    mid_code(&db, &lead, &team, "MID-A").await;
    let carl = new_user(&db, "carl").await;
    db::redeem_team_invite(&db, &carl, "MID-A").await.unwrap();

    // Team B sees nothing of team A's placements and cannot answer them.
    assert!(db::list_auto_placed(&db, &team_b).await.unwrap().is_empty());
    assert!(matches!(
        db::clear_auto_placed(&db, &team_b, &carl, true).await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        db::clear_auto_placed(&db, &team_b, &carl, false).await,
        Err(DbError::NotFound)
    ));
    // Carl is untouched in team A.
    assert_eq!(place_of(&db, &carl).await, ("starter".to_string(), true));
    assert_eq!(db::list_auto_placed(&db, &team).await.unwrap().len(), 1);
}
