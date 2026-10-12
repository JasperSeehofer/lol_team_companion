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
    db::join_team_from_invite(&db, &carl, &grant).await.unwrap();
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
    assert_eq!(
        carl_row.roster_type, "member",
        "not placed until the leader does"
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
    assert_eq!(joined, team);
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
