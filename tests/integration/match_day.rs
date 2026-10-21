//! M5: Match Day (next match, prep facts), share links, player cards, and
//! the "draft deleted" mark. Cross-tenant cases follow the security suite's
//! shape: team two's user gets NotFound (or nothing) for team one's data.

use crate::common;

use lol_team_companion::models::game_plan::{GamePlan, PostGameLearning};
use lol_team_companion::models::match_day::ScheduledMatchInput;
use lol_team_companion::models::share::ShareKind;
use lol_team_companion::server::db::{self, DbError};
use surrealdb::{engine::local::Db, Surreal};

struct T {
    a: String,
    team1: String,
    b: String,
    team2: String,
}

async fn tenants(db: &Surreal<Db>) -> T {
    let a = db::create_user(db, "alice".into(), "alice@example.com".into(), "h".into())
        .await
        .unwrap();
    let b = db::create_user(db, "bob".into(), "bob@example.com".into(), "h".into())
        .await
        .unwrap();
    let team1 = db::create_team(db, &a, "Team One".into(), "EUW".into())
        .await
        .unwrap();
    let team2 = db::create_team(db, &b, "Team Two".into(), "EUW".into())
        .await
        .unwrap();
    T { a, team1, b, team2 }
}

fn not_found<T: std::fmt::Debug>(r: Result<T, DbError>, what: &str) {
    match r {
        Err(DbError::NotFound) => {}
        other => panic!("{what}: expected NotFound, got {other:?}"),
    }
}

fn in_hours(h: i64) -> String {
    (chrono::Utc::now() + chrono::Duration::hours(h)).to_rfc3339()
}

fn input(starts_at: String, opponent: Option<String>) -> ScheduledMatchInput {
    ScheduledMatchInput {
        id: None,
        starts_at,
        kind: "scrim".into(),
        format: "bo3".into(),
        opponent_id: opponent,
        notes: Some("  Prime League week 3  ".into()),
    }
}

async fn draft_vs(db: &Surreal<Db>, team: &str, user: &str, opponent: Option<String>) -> String {
    db::save_draft(
        db,
        team,
        user,
        "Draft vs Rooks".into(),
        opponent,
        None,
        vec![],
        vec![],
        None,
        "blue".into(),
        vec![],
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap()
}

fn plan(team: &str, draft: Option<String>) -> GamePlan {
    GamePlan {
        scheduled_match_id: None,
        game_number: None,
        scheduled_match_deleted: false,
        draft_deleted: false,
        id: None,
        team_id: team.into(),
        draft_id: draft,
        name: "Plan vs Rooks".into(),
        our_champions: vec![],
        enemy_champions: vec![],
        win_conditions: vec![],
        objective_priority: vec![],
        teamfight_strategy: String::new(),
        early_game: None,
        top_strategy: None,
        jungle_strategy: None,
        mid_strategy: None,
        bot_strategy: None,
        support_strategy: None,
        notes: None,
        win_condition_tag: None,
    }
}

fn review(team: &str, user: &str, plan: Option<String>, draft: Option<String>) -> PostGameLearning {
    PostGameLearning {
        scheduled_match_id: None,
        series_id: None,
        game_number: None,
        scheduled_match_deleted: false,
        draft_deleted: false,
        created_at: None,
        id: None,
        team_id: team.into(),
        match_riot_id: None,
        game_plan_id: plan,
        draft_id: draft,
        what_went_well: vec![],
        improvements: vec!["Lost tempo after 20".into()],
        action_items: vec![],
        open_notes: None,
        created_by: user.into(),
        win_loss: Some("loss".into()),
        rating: None,
    }
}

#[tokio::test]
async fn the_next_match_is_team_scoped_and_validated() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let opp = db::create_opponent(&db, &t.team1, "Riverside Rooks".into(), None)
        .await
        .unwrap();
    let other_opp = db::create_opponent(&db, &t.team2, "Other".into(), None)
        .await
        .unwrap();

    assert!(db::next_scheduled_match(&db, &t.team1)
        .await
        .unwrap()
        .is_none());
    // Team one cannot point its match at team two's dossier.
    not_found(
        db::save_scheduled_match(&db, &t.team1, &t.a, input(in_hours(5), Some(other_opp))).await,
        "foreign opponent",
    );
    // Bad kind, format and time are refused.
    let mut bad = input(in_hours(5), None);
    bad.format = "bo7".into();
    assert!(matches!(
        db::save_scheduled_match(&db, &t.team1, &t.a, bad).await,
        Err(DbError::Invalid(_))
    ));
    assert!(matches!(
        db::save_scheduled_match(&db, &t.team1, &t.a, input("tomorrow".into(), None)).await,
        Err(DbError::Invalid(_))
    ));
    assert!(matches!(
        db::save_scheduled_match(&db, &t.team1, &t.a, input(in_hours(-48), None)).await,
        Err(DbError::Invalid(_))
    ));

    // A later match and an earlier one: the earlier is next.
    db::save_scheduled_match(&db, &t.team1, &t.a, input(in_hours(72), None))
        .await
        .unwrap();
    let id = db::save_scheduled_match(&db, &t.team1, &t.a, input(in_hours(5), Some(opp.clone())))
        .await
        .unwrap();
    let next = db::next_scheduled_match(&db, &t.team1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(next.id, id);
    assert_eq!(next.opponent_name.as_deref(), Some("Riverside Rooks"));
    assert_eq!(next.notes.as_deref(), Some("Prime League week 3"));
    assert_eq!(next.format, "bo3");
    assert!(db::next_scheduled_match(&db, &t.team2)
        .await
        .unwrap()
        .is_none());

    // Team two can neither edit, delete nor mark it.
    let mut edit = input(in_hours(6), None);
    edit.id = Some(id.clone());
    not_found(
        db::save_scheduled_match(&db, &t.team2, &t.b, edit.clone()).await,
        "B edits A's match",
    );
    not_found(
        db::delete_scheduled_match(&db, &t.team2, &id).await,
        "B deletes A's match",
    );
    not_found(
        db::set_brief_read(&db, &t.team2, &id, &t.b, true).await,
        "B marks A's brief",
    );

    // Reading the brief is per member and idempotent.
    db::set_brief_read(&db, &t.team1, &id, &t.a, true)
        .await
        .unwrap();
    db::set_brief_read(&db, &t.team1, &id, &t.a, true)
        .await
        .unwrap();
    let next = db::next_scheduled_match(&db, &t.team1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(next.brief_read_by, vec![t.a.clone()]);
    db::set_brief_read(&db, &t.team1, &id, &t.a, false)
        .await
        .unwrap();
    let next = db::next_scheduled_match(&db, &t.team1)
        .await
        .unwrap()
        .unwrap();
    assert!(next.brief_read_by.is_empty());

    db::save_scheduled_match(&db, &t.team1, &t.a, edit)
        .await
        .unwrap();
    let next = db::next_scheduled_match(&db, &t.team1)
        .await
        .unwrap()
        .unwrap();
    assert!(next.opponent_id.is_none());
    db::delete_scheduled_match(&db, &t.team1, &id)
        .await
        .unwrap();
}

#[tokio::test]
async fn prep_facts_follow_the_artefacts() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let opp = db::create_opponent(&db, &t.team1, "Riverside Rooks".into(), None)
        .await
        .unwrap();
    let starts = in_hours(-1);
    let m = db::save_scheduled_match(&db, &t.team1, &t.a, input(starts, Some(opp.clone())))
        .await
        .unwrap();
    let f = db::match_prep_facts(&db, &t.team1, &m, Some(&opp), Some("Riverside Rooks"))
        .await
        .unwrap();
    assert!(f.drafts.is_empty() && f.plans.is_empty() && f.review_after.is_none());
    assert!(f.earlier.is_empty());

    // One draft by id, one older one by name, one against someone else.
    let d1 = draft_vs(&db, &t.team1, &t.a, Some(opp.clone())).await;
    draft_vs(&db, &t.team1, &t.a, Some("riverside rooks".into())).await;
    draft_vs(&db, &t.team1, &t.a, Some("Someone else".into())).await;
    // Team two's draft against a same-named team does not count.
    draft_vs(&db, &t.team2, &t.b, Some("Riverside Rooks".into())).await;
    let p1 = db::save_game_plan(&db, plan(&t.team1, Some(d1.clone())), &t.a)
        .await
        .unwrap();
    db::save_game_plan(&db, plan(&t.team1, None), &t.a)
        .await
        .unwrap();
    db::save_post_game_learning(&db, review(&t.team1, &t.a, Some(p1.clone()), None))
        .await
        .unwrap();
    // M159: unstamped work is earlier work, never counted for the match.
    let f = db::match_prep_facts(&db, &t.team1, &m, Some(&opp), Some("Riverside Rooks"))
        .await
        .unwrap();
    assert!(f.drafts.is_empty() && f.plans.is_empty() && f.review_after.is_none());
    assert_eq!(f.earlier.len(), 4, "two drafts, the plan and the review");
    assert!(f.earlier.iter().any(|e| e.id == p1));
    assert!(f.earlier.iter().all(|e| !e.id.is_empty()));

    let counts = db::team_counts(&db, &t.team1).await.unwrap();
    assert_eq!(
        (
            counts.opponents,
            counts.drafts,
            counts.plans,
            counts.reviews,
            counts.members
        ),
        (1, 3, 2, 1, 1)
    );
    let reviews = db::recent_reviews(&db, &t.team1, 5).await.unwrap();
    assert_eq!(reviews.len(), 1);
    assert!(reviews[0].created_at.is_some());
    assert!(db::recent_reviews(&db, &t.team2, 5)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn deleting_a_draft_marks_its_plans_and_reviews() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let d = draft_vs(&db, &t.team1, &t.a, None).await;
    let p = db::save_game_plan(&db, plan(&t.team1, Some(d.clone())), &t.a)
        .await
        .unwrap();
    let r = db::save_post_game_learning(&db, review(&t.team1, &t.a, None, Some(d.clone())))
        .await
        .unwrap();
    db::delete_draft(&db, &t.team1, &d).await.unwrap();
    let plan = db::get_game_plan(&db, &t.team1, &p).await.unwrap().unwrap();
    assert!(plan.draft_id.is_none() && plan.draft_deleted);
    let rev = db::get_post_game_learning(&db, &t.team1, &r)
        .await
        .unwrap()
        .unwrap();
    assert!(rev.draft_id.is_none() && rev.draft_deleted);
    // Linking a new draft clears the mark.
    let d2 = draft_vs(&db, &t.team1, &t.a, None).await;
    let mut plan = plan;
    plan.draft_id = Some(d2);
    db::update_game_plan(&db, &t.team1, plan).await.unwrap();
    let plan = db::get_game_plan(&db, &t.team1, &p).await.unwrap().unwrap();
    assert!(plan.draft_id.is_some() && !plan.draft_deleted);
    // Reviews are team-scoped.
    assert!(db::get_post_game_learning(&db, &t.team2, &r)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn share_links_open_one_artefact_and_stay_in_their_team() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let p = db::save_game_plan(&db, plan(&t.team1, None), &t.a)
        .await
        .unwrap();
    let r = db::save_post_game_learning(&db, review(&t.team1, &t.a, None, None))
        .await
        .unwrap();

    // Team two cannot share team one's plan, review or member.
    not_found(
        db::create_share_link(&db, &t.team2, &t.b, ShareKind::GamePlan, &p, 7).await,
        "B shares A's plan",
    );
    not_found(
        db::create_share_link(&db, &t.team2, &t.b, ShareKind::Review, &r, 7).await,
        "B shares A's review",
    );
    not_found(
        db::create_share_link(&db, &t.team2, &t.b, ShareKind::Player, &t.a, 7).await,
        "B shares A's player card",
    );
    // A plan id passed as a review is refused too.
    not_found(
        db::create_share_link(&db, &t.team1, &t.a, ShareKind::Review, &p, 7).await,
        "plan id as a review",
    );
    assert!(matches!(
        db::create_share_link(&db, &t.team1, &t.a, ShareKind::GamePlan, &p, 365).await,
        Err(DbError::Invalid(_))
    ));

    let link = db::create_share_link(&db, &t.team1, &t.a, ShareKind::GamePlan, &p, 7)
        .await
        .unwrap();
    assert!(db::plausible_share_token(&link.token));
    assert_eq!(link.created_by_name, "alice");
    let resolved = db::resolve_share_token(&db, &link.token)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(resolved.team_id, t.team1);
    assert_eq!(resolved.kind, ShareKind::GamePlan);
    assert_eq!(resolved.target_id, p);

    // Unknown and malformed tokens open nothing.
    assert!(db::resolve_share_token(&db, &db::generate_share_token())
        .await
        .unwrap()
        .is_none());
    assert!(db::resolve_share_token(&db, "' OR 1=1 --")
        .await
        .unwrap()
        .is_none());

    // Listing and revoking are team-scoped.
    assert_eq!(
        db::list_share_links(&db, &t.team1, ShareKind::GamePlan, &p)
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(db::list_share_links(&db, &t.team2, ShareKind::GamePlan, &p)
        .await
        .unwrap()
        .is_empty());
    not_found(
        db::revoke_share_link(&db, &t.team2, &link.id).await,
        "B revokes A's link",
    );
    assert!(db::resolve_share_token(&db, &link.token)
        .await
        .unwrap()
        .is_some());
    db::revoke_share_link(&db, &t.team1, &link.id)
        .await
        .unwrap();
    assert!(db::resolve_share_token(&db, &link.token)
        .await
        .unwrap()
        .is_none());
    assert!(db::list_share_links(&db, &t.team1, ShareKind::GamePlan, &p)
        .await
        .unwrap()
        .is_empty());

    // An expired link opens nothing.
    let old = db::create_share_link(&db, &t.team1, &t.a, ShareKind::Review, &r, 1)
        .await
        .unwrap();
    db.query("UPDATE share_link SET expires_at = time::now() - 1h WHERE token = $t")
        .bind(("t", old.token.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();
    assert!(db::resolve_share_token(&db, &old.token)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn player_cards_show_members_only() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    db::add_to_champion_pool(&db, &t.a, "Jinx".into(), "ADC".into())
        .await
        .unwrap();
    db::add_to_champion_pool(&db, &t.a, "Ahri".into(), "mid".into())
        .await
        .unwrap();
    db::assign_to_slot(&db, &t.team1, &t.a, "bot")
        .await
        .unwrap();
    let card = db::player_card(&db, &t.team1, &t.a).await.unwrap();
    assert_eq!(card.username, "alice");
    assert_eq!(card.team_name, "Team One");
    assert_eq!(
        (card.role.as_str(), card.place.as_str()),
        ("bot", "starter")
    );
    assert_eq!(card.champions.len(), 2);
    assert!(card.form.is_empty());
    // The pool role was stored in one spelling.
    let pool = db::get_champion_pool(&db, &t.a).await.unwrap();
    assert!(pool.iter().any(|e| e.champion == "Jinx" && e.role == "bot"));

    not_found(
        db::player_card(&db, &t.team2, &t.a).await,
        "B reads A's card",
    );
    let link = db::create_share_link(&db, &t.team1, &t.a, ShareKind::Player, &t.a, 30)
        .await
        .unwrap();
    let resolved = db::resolve_share_token(&db, &link.token)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(resolved.target_id, t.a);
}

#[tokio::test]
async fn team_match_rows_carry_their_match_id() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    db.query(
        "CREATE match:m1 SET match_id = 'EUW1_1', queue_id = 420, game_duration = 1800, game_end = time::now(); \
         CREATE player_match SET match = match:m1, user = type::record('user', $u), champion = 'Jinx', kills = 1, deaths = 2, assists = 3, cs = 100, vision_score = 10, damage = 1000, win = true;",
    )
    .bind(("u", t.a.strip_prefix("user:").unwrap().to_string()))
    .await
    .unwrap()
    .check()
    .unwrap();
    let rows = db::get_team_match_stats(&db, &t.team1).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].match_db_id, "match:m1");
    assert_eq!(rows[0].riot_match_id, "EUW1_1");
    assert!(db::get_team_match_stats(&db, &t.team2)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn a_new_opponent_or_a_deleted_dossier_resets_the_brief() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let x = db::create_opponent(&db, &t.team1, "Rooks".into(), None)
        .await
        .unwrap();
    let y = db::create_opponent(&db, &t.team1, "Owls".into(), None)
        .await
        .unwrap();
    let id = db::save_scheduled_match(&db, &t.team1, &t.a, input(in_hours(5), Some(x.clone())))
        .await
        .unwrap();
    db::set_brief_read(&db, &t.team1, &id, &t.a, true)
        .await
        .unwrap();

    // Same opponent, new time: the read mark stays.
    let mut same = input(in_hours(6), Some(x.clone()));
    same.id = Some(id.clone());
    db::save_scheduled_match(&db, &t.team1, &t.a, same)
        .await
        .unwrap();
    let next = db::next_scheduled_match(&db, &t.team1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(next.brief_read_by, vec![t.a.clone()]);

    // Another opponent: nobody has read that brief yet.
    let mut other = input(in_hours(6), Some(y.clone()));
    other.id = Some(id.clone());
    db::save_scheduled_match(&db, &t.team1, &t.a, other)
        .await
        .unwrap();
    let next = db::next_scheduled_match(&db, &t.team1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(next.opponent_id.as_deref(), Some(y.as_str()));
    assert!(next.brief_read_by.is_empty());

    // Deleting the dossier clears the match's opponent instead of leaving
    // a link to a deleted record (the match stays editable).
    db::set_brief_read(&db, &t.team1, &id, &t.a, true)
        .await
        .unwrap();
    db::delete_opponent(&db, &t.team1, &y).await.unwrap();
    let next = db::next_scheduled_match(&db, &t.team1)
        .await
        .unwrap()
        .unwrap();
    assert!(next.opponent_id.is_none());
    assert!(next.brief_read_by.is_empty());
    let mut edit = input(in_hours(7), None);
    edit.id = Some(id.clone());
    db::save_scheduled_match(&db, &t.team1, &t.a, edit)
        .await
        .unwrap();
}
