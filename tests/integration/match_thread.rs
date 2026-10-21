//! M159: the match thread. Drafts, plans and reviews carry a stamp of the
//! scheduled match (and series and game) they were made for; each stamp is
//! checked as the caller's team record. Deleting a match clears the stamps
//! and marks the work, never deletes it.

use crate::common;

use lol_team_companion::models::game_plan::{GamePlan, PostGameLearning};
use lol_team_companion::models::match_day::ScheduledMatchInput;
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

fn invalid<T: std::fmt::Debug>(r: Result<T, DbError>, what: &str) {
    match r {
        Err(DbError::Invalid(_)) => {}
        other => panic!("{what}: expected Invalid, got {other:?}"),
    }
}

fn in_hours(h: i64) -> String {
    (chrono::Utc::now() + chrono::Duration::hours(h)).to_rfc3339()
}

async fn new_match(db: &Surreal<Db>, team: &str, user: &str, hours: i64) -> String {
    db::save_scheduled_match(
        db,
        team,
        user,
        ScheduledMatchInput {
            id: None,
            starts_at: in_hours(hours),
            kind: "scrim".into(),
            format: "bo3".into(),
            opponent_id: None,
            notes: None,
        },
    )
    .await
    .unwrap()
}

async fn draft(
    db: &Surreal<Db>,
    team: &str,
    user: &str,
    series: Option<String>,
    game: Option<i32>,
    scheduled_match: Option<String>,
) -> Result<String, DbError> {
    db::save_draft(
        db,
        team,
        user,
        "Draft for the match".into(),
        None,
        None,
        vec![],
        vec![],
        None,
        "blue".into(),
        vec![],
        None,
        None,
        series,
        game,
        scheduled_match,
    )
    .await
}

async fn update_draft(
    db: &Surreal<Db>,
    team: &str,
    id: &str,
    series: Option<String>,
    game: Option<i32>,
    scheduled_match: Option<String>,
) -> Result<(), DbError> {
    db::update_draft(
        db,
        team,
        id,
        "Draft for the match".into(),
        None,
        None,
        vec![],
        vec![],
        None,
        "red".into(),
        vec![],
        None,
        None,
        series,
        game,
        scheduled_match,
    )
    .await
}

fn plan(team: &str, scheduled_match: Option<String>, game: Option<i32>) -> GamePlan {
    GamePlan {
        scheduled_match_id: scheduled_match,
        game_number: game,
        scheduled_match_deleted: false,
        draft_deleted: false,
        id: None,
        team_id: team.into(),
        draft_id: None,
        name: "Plan for the match".into(),
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

fn review(
    team: &str,
    user: &str,
    scheduled_match: Option<String>,
    series: Option<String>,
    game: Option<i32>,
) -> PostGameLearning {
    PostGameLearning {
        scheduled_match_id: scheduled_match,
        series_id: series,
        game_number: game,
        scheduled_match_deleted: false,
        draft_deleted: false,
        created_at: None,
        id: None,
        team_id: team.into(),
        match_riot_id: None,
        game_plan_id: None,
        draft_id: None,
        what_went_well: vec![],
        improvements: vec![],
        action_items: vec![],
        open_notes: Some("Scrim, no synced game".into()),
        created_by: user.into(),
        win_loss: None,
        rating: None,
    }
}

async fn series(db: &Surreal<Db>, team: &str, user: &str) -> String {
    db::create_series(
        db,
        team,
        user,
        "vs Rooks Bo3".into(),
        None,
        Some("Rooks".into()),
        "bo3".into(),
        false,
        None,
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn stamps_take_the_teams_own_match_series_and_game() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let m = new_match(&db, &t.team1, &t.a, 24).await;
    let s = series(&db, &t.team1, &t.a).await;

    let d = draft(
        &db,
        &t.team1,
        &t.a,
        Some(s.clone()),
        Some(2),
        Some(m.clone()),
    )
    .await
    .unwrap();
    let got = db::get_draft_for_prefill(&db, &t.team1, &d)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.scheduled_match_id.as_deref(), Some(m.as_str()));
    assert_eq!(got.series_id.as_deref(), Some(s.as_str()));
    assert_eq!(got.game_number, Some(2));
    // An update without a match keeps the stamp (the page does not carry it).
    update_draft(&db, &t.team1, &d, Some(s.clone()), Some(2), None)
        .await
        .unwrap();
    let got = db::get_draft_for_prefill(&db, &t.team1, &d)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.scheduled_match_id.as_deref(), Some(m.as_str()));

    let p = db::save_game_plan(&db, plan(&t.team1, Some(m.clone()), Some(1)), &t.a)
        .await
        .unwrap();
    let got = db::get_game_plan(&db, &t.team1, &p).await.unwrap().unwrap();
    assert_eq!(got.scheduled_match_id.as_deref(), Some(m.as_str()));
    assert_eq!(got.game_number, Some(1));
    // Saving the plan from the page (no stamp in the form) keeps it.
    let mut edited = got.clone();
    edited.scheduled_match_id = None;
    edited.game_number = None;
    edited.name = "Renamed".into();
    db::update_game_plan(&db, &t.team1, edited).await.unwrap();
    let got = db::get_game_plan(&db, &t.team1, &p).await.unwrap().unwrap();
    assert_eq!(got.name, "Renamed");
    assert_eq!(got.scheduled_match_id.as_deref(), Some(m.as_str()));
    assert_eq!(got.game_number, Some(1));

    // A scrim review without a Riot id, stamped to the match.
    let (r, _) = db::create_review_with_action_items(
        &db,
        review(&t.team1, &t.a, Some(m.clone()), Some(s.clone()), Some(3)),
    )
    .await
    .unwrap();
    let got = db::get_post_game_learning(&db, &t.team1, &r)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.scheduled_match_id.as_deref(), Some(m.as_str()));
    assert_eq!(got.series_id.as_deref(), Some(s.as_str()));
    assert_eq!(got.game_number, Some(3));
    assert!(got.match_riot_id.is_none());

    // Game numbers are 1 to 5.
    invalid(
        draft(&db, &t.team1, &t.a, Some(s.clone()), Some(6), None).await,
        "draft game 6",
    );
    invalid(
        db::save_game_plan(&db, plan(&t.team1, Some(m.clone()), Some(0)), &t.a).await,
        "plan game 0",
    );
    invalid(
        db::create_review_with_action_items(
            &db,
            review(&t.team1, &t.a, Some(m.clone()), None, Some(-1)),
        )
        .await,
        "review game -1",
    );
}

#[tokio::test]
async fn another_teams_match_or_series_is_not_found() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let theirs = new_match(&db, &t.team2, &t.b, 24).await;
    let their_series = series(&db, &t.team2, &t.b).await;

    not_found(
        draft(&db, &t.team1, &t.a, None, None, Some(theirs.clone())).await,
        "draft stamped with a foreign match",
    );
    not_found(
        draft(
            &db,
            &t.team1,
            &t.a,
            Some(their_series.clone()),
            Some(1),
            None,
        )
        .await,
        "draft in a foreign series",
    );
    not_found(
        draft(
            &db,
            &t.team1,
            &t.a,
            None,
            None,
            Some("scheduled_match:nope".into()),
        )
        .await,
        "draft stamped with an unknown match",
    );
    not_found(
        db::save_game_plan(&db, plan(&t.team1, Some(theirs.clone()), None), &t.a).await,
        "plan stamped with a foreign match",
    );
    not_found(
        db::create_review_with_action_items(
            &db,
            review(&t.team1, &t.a, Some(theirs.clone()), None, None),
        )
        .await,
        "review stamped with a foreign match",
    );
    not_found(
        db::save_post_game_learning(
            &db,
            review(&t.team1, &t.a, None, Some(their_series.clone()), Some(1)),
        )
        .await,
        "review in a foreign series",
    );

    // Updates are checked too, and nothing changed.
    let d = draft(&db, &t.team1, &t.a, None, None, None).await.unwrap();
    not_found(
        update_draft(&db, &t.team1, &d, None, None, Some(theirs.clone())).await,
        "draft update with a foreign match",
    );
    let p = db::save_game_plan(&db, plan(&t.team1, None, None), &t.a)
        .await
        .unwrap();
    let mut edited = db::get_game_plan(&db, &t.team1, &p).await.unwrap().unwrap();
    edited.scheduled_match_id = Some(theirs.clone());
    not_found(
        db::update_game_plan(&db, &t.team1, edited).await,
        "plan update with a foreign match",
    );
    let (r, _) = db::create_review_with_action_items(&db, review(&t.team1, &t.a, None, None, None))
        .await
        .unwrap();
    let mut edited = db::get_post_game_learning(&db, &t.team1, &r)
        .await
        .unwrap()
        .unwrap();
    edited.scheduled_match_id = Some(theirs.clone());
    not_found(
        db::update_review_with_action_items(&db, &t.team1, edited.clone()).await,
        "review update with a foreign match",
    );
    not_found(
        db::update_post_game_learning(&db, &t.team1, edited).await,
        "review update (plain) with a foreign match",
    );
    let got = db::get_post_game_learning(&db, &t.team1, &r)
        .await
        .unwrap()
        .unwrap();
    assert!(got.scheduled_match_id.is_none());
    // Team two cannot delete team one's match either.
    let ours = new_match(&db, &t.team1, &t.a, 24).await;
    not_found(
        db::delete_scheduled_match(&db, &t.team2, &ours).await,
        "foreign match deletion",
    );
}

#[tokio::test]
async fn deleting_a_match_clears_its_stamps_and_keeps_the_work() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let m = new_match(&db, &t.team1, &t.a, 24).await;
    let other = new_match(&db, &t.team1, &t.a, 48).await;
    let d = draft(&db, &t.team1, &t.a, None, None, Some(m.clone()))
        .await
        .unwrap();
    let d_other = draft(&db, &t.team1, &t.a, None, None, Some(other.clone()))
        .await
        .unwrap();
    let p = db::save_game_plan(&db, plan(&t.team1, Some(m.clone()), Some(1)), &t.a)
        .await
        .unwrap();
    let (r, _) = db::create_review_with_action_items(
        &db,
        review(&t.team1, &t.a, Some(m.clone()), None, Some(1)),
    )
    .await
    .unwrap();

    db::delete_scheduled_match(&db, &t.team1, &m).await.unwrap();

    let got = db::get_draft_for_prefill(&db, &t.team1, &d)
        .await
        .unwrap()
        .expect("the draft survives");
    assert!(got.scheduled_match_id.is_none() && got.scheduled_match_deleted);
    let got = db::get_game_plan(&db, &t.team1, &p)
        .await
        .unwrap()
        .expect("the plan survives");
    assert!(got.scheduled_match_id.is_none() && got.scheduled_match_deleted);
    assert_eq!(got.game_number, Some(1));
    let got = db::get_post_game_learning(&db, &t.team1, &r)
        .await
        .unwrap()
        .expect("the review survives");
    assert!(got.scheduled_match_id.is_none() && got.scheduled_match_deleted);
    // Work for another match is untouched.
    let got = db::get_draft_for_prefill(&db, &t.team1, &d_other)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.scheduled_match_id.as_deref(), Some(other.as_str()));
    assert!(!got.scheduled_match_deleted);

    // Stamping the plan to another match clears the mark.
    let mut edited = db::get_game_plan(&db, &t.team1, &p).await.unwrap().unwrap();
    edited.scheduled_match_id = Some(other.clone());
    db::update_game_plan(&db, &t.team1, edited).await.unwrap();
    let got = db::get_game_plan(&db, &t.team1, &p).await.unwrap().unwrap();
    assert_eq!(got.scheduled_match_id.as_deref(), Some(other.as_str()));
    assert!(!got.scheduled_match_deleted);
}

#[tokio::test]
async fn deleting_a_series_unlinks_reviews_and_matches() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let s = series(&db, &t.team1, &t.a).await;
    let r =
        db::save_post_game_learning(&db, review(&t.team1, &t.a, None, Some(s.clone()), Some(2)))
            .await
            .unwrap();
    // Item 6 links a match to its series; set the link directly here.
    let m = new_match(&db, &t.team1, &t.a, 24).await;
    db.query("UPDATE type::record('scheduled_match', $m) SET series = type::record('series', $s)")
        .bind(("m", db::record_key(&m, "scheduled_match")))
        .bind(("s", db::record_key(&s, "series")))
        .await
        .unwrap()
        .check()
        .unwrap();
    let next = db::next_scheduled_match(&db, &t.team1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(next.series_id.as_deref(), Some(s.as_str()));
    db::delete_series(&db, &t.team1, &s).await.unwrap();
    let next = db::next_scheduled_match(&db, &t.team1)
        .await
        .unwrap()
        .unwrap();
    assert!(next.series_id.is_none());
    let got = db::get_post_game_learning(&db, &t.team1, &r)
        .await
        .unwrap()
        .expect("the review survives");
    assert!(got.series_id.is_none());
}

// ---------------------------------------------------------------------------
// Item 2: the upcoming-match list
// ---------------------------------------------------------------------------

/// Move a match's start (past the editor's checks) for the grace cases.
async fn set_start(db: &Surreal<Db>, id: &str, hours: i64) {
    db.query("UPDATE type::record('scheduled_match', $k) SET starts_at = <datetime>$s")
        .bind(("k", db::record_key(id, "scheduled_match")))
        .bind(("s", in_hours(hours)))
        .await
        .unwrap()
        .check()
        .unwrap();
}

#[tokio::test]
async fn upcoming_matches_are_ordered_bounded_and_team_scoped() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    // Created out of order; one long over, one in its grace period.
    let later = new_match(&db, &t.team1, &t.a, 72).await;
    let soon = new_match(&db, &t.team1, &t.a, 24).await;
    let in_grace = new_match(&db, &t.team1, &t.a, 1).await;
    set_start(&db, &in_grace, -6).await;
    let over = new_match(&db, &t.team1, &t.a, 2).await;
    set_start(&db, &over, -20).await;
    let theirs = new_match(&db, &t.team2, &t.b, 12).await;

    let list = db::upcoming_scheduled_matches(&db, &t.team1).await.unwrap();
    let ids: Vec<&str> = list.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, [in_grace.as_str(), soon.as_str(), later.as_str()]);
    // The first row is the next match.
    let next = db::next_scheduled_match(&db, &t.team1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(next.id, in_grace);
    // The other team sees only its own.
    let other = db::upcoming_scheduled_matches(&db, &t.team2).await.unwrap();
    assert_eq!(other.len(), 1);
    assert_eq!(other[0].id, theirs);

    // At most 20 rows, the earliest ones.
    for h in 0..20 {
        new_match(&db, &t.team1, &t.a, 100 + h).await;
    }
    let list = db::upcoming_scheduled_matches(&db, &t.team1).await.unwrap();
    assert_eq!(list.len(), db::UPCOMING_MATCHES_MAX);
    assert_eq!(list[0].id, in_grace);
    assert!(list.windows(2).all(|w| w[0].starts_at <= w[1].starts_at));
    // Nothing more than 366 days ahead.
    let far = new_match(&db, &t.team2, &t.b, 48).await;
    set_start(&db, &far, 24 * 400).await;
    let other = db::upcoming_scheduled_matches(&db, &t.team2).await.unwrap();
    assert!(other.iter().all(|m| m.id != far));
}

#[tokio::test]
async fn a_match_id_from_the_address_falls_back_to_the_next_match() {
    use lol_team_companion::models::match_day::choose_match;
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let first = new_match(&db, &t.team1, &t.a, 24).await;
    let second = new_match(&db, &t.team1, &t.a, 48).await;
    let theirs = new_match(&db, &t.team2, &t.b, 36).await;
    let over = new_match(&db, &t.team1, &t.a, 2).await;
    set_start(&db, &over, -30).await;

    let pick = |id: Option<String>| {
        let db = db.clone();
        let team = t.team1.clone();
        async move {
            let upcoming = db::upcoming_scheduled_matches(&db, &team).await.unwrap();
            let next = db::next_scheduled_match(&db, &team).await.unwrap();
            choose_match(upcoming, next, id.as_deref())
        }
    };

    // The team's own later match becomes the current view.
    let c = pick(Some(second.clone())).await;
    assert_eq!(
        c.current.as_ref().map(|m| m.id.as_str()),
        Some(second.as_str())
    );
    assert!(c.chosen && !c.unknown);
    assert_eq!(c.others.len(), 1);
    assert_eq!(c.others[0].id, first);

    // Another team's match, a match long over and a made-up id all fall
    // back to the next match, the same way.
    for id in [
        theirs.clone(),
        over.clone(),
        "scheduled_match:nope".to_string(),
    ] {
        let c = pick(Some(id.clone())).await;
        assert_eq!(
            c.current.as_ref().map(|m| m.id.as_str()),
            Some(first.as_str()),
            "{id}"
        );
        assert!(c.unknown && !c.chosen, "{id}");
        assert!(c.others.iter().all(|m| m.id != theirs));
    }
}

/// Item 3: a carried link's match resolves only for the caller's team; the
/// draft the link starts is stamped with it on the first save, and an
/// update without a match keeps the stamp.
#[tokio::test]
async fn a_carried_match_resolves_for_the_team_only() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let ours = new_match(&db, &t.team1, &t.a, 24).await;
    let theirs = new_match(&db, &t.team2, &t.b, 24).await;
    // A match long over still resolves (a review link after the match).
    let over = new_match(&db, &t.team1, &t.a, 2).await;
    set_start(&db, &over, -72).await;

    let m = db::get_scheduled_match(&db, &t.team1, &ours).await.unwrap();
    assert_eq!(m.id, ours);
    assert_eq!(m.format, "bo3");
    let key = ours.trim_start_matches("scheduled_match:").to_string();
    assert_eq!(
        db::get_scheduled_match(&db, &t.team1, &key)
            .await
            .unwrap()
            .id,
        ours
    );
    assert!(db::get_scheduled_match(&db, &t.team1, &over).await.is_ok());
    not_found(
        db::get_scheduled_match(&db, &t.team1, &theirs).await,
        "another team's match",
    );
    not_found(
        db::get_scheduled_match(&db, &t.team1, "scheduled_match:nope").await,
        "an unknown match",
    );
    not_found(
        db::get_scheduled_match(&db, &t.team1, "  ").await,
        "a blank id",
    );

    // The first save stamps the draft with the match and its game.
    let d = draft(&db, &t.team1, &t.a, None, Some(2), Some(ours.clone()))
        .await
        .unwrap();
    let saved = db::get_draft_for_prefill(&db, &t.team1, &d)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.scheduled_match_id.as_deref(), Some(ours.as_str()));
    assert_eq!(saved.game_number, Some(2));
    // Later saves from the draft page send no match: the stamp stays.
    update_draft(&db, &t.team1, &d, None, Some(2), None)
        .await
        .unwrap();
    let saved = db::get_draft_for_prefill(&db, &t.team1, &d)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.scheduled_match_id.as_deref(), Some(ours.as_str()));
    // A link naming another team's match never stamps.
    not_found(
        draft(&db, &t.team1, &t.a, None, Some(1), Some(theirs.clone())).await,
        "a draft stamped with another team's match",
    );
}

async fn match_vs(db: &Surreal<Db>, team: &str, user: &str, opponent: &str, hours: i64) -> String {
    db::save_scheduled_match(
        db,
        team,
        user,
        ScheduledMatchInput {
            id: None,
            starts_at: in_hours(hours),
            kind: "scrim".into(),
            format: "bo3".into(),
            opponent_id: Some(opponent.into()),
            notes: None,
        },
    )
    .await
    .unwrap()
}

async fn unstamped_draft_vs(db: &Surreal<Db>, team: &str, user: &str, opponent: &str) -> String {
    db::save_draft(
        db,
        team,
        user,
        "Old draft vs Rooks".into(),
        Some(opponent.into()),
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

/// Item 4: Match Day's prep facts count only the work stamped to this
/// match, per game; a rematch against the same opponent starts empty, and
/// unstamped work against the opponent is earlier work, never counted.
#[tokio::test]
async fn prep_facts_count_only_work_stamped_to_the_match() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let opp = db::create_opponent(&db, &t.team1, "Riverside Rooks".into(), None)
        .await
        .unwrap();
    let first = match_vs(&db, &t.team1, &t.a, &opp, 24).await;
    let rematch = match_vs(&db, &t.team1, &t.a, &opp, 72).await;

    // The first match: games 1 and 2 drafted, a plan, game 1 reviewed.
    draft(&db, &t.team1, &t.a, None, Some(1), Some(first.clone()))
        .await
        .unwrap();
    draft(&db, &t.team1, &t.a, None, Some(2), Some(first.clone()))
        .await
        .unwrap();
    db::save_game_plan(&db, plan(&t.team1, Some(first.clone()), Some(1)), &t.a)
        .await
        .unwrap();
    db::save_post_game_learning(
        &db,
        review(&t.team1, &t.a, Some(first.clone()), None, Some(1)),
    )
    .await
    .unwrap();
    // Earlier, unstamped work against the same opponent.
    let old = unstamped_draft_vs(&db, &t.team1, &t.a, &opp).await;
    // Team two's work never shows.
    let opp2 = db::create_opponent(&db, &t.team2, "Riverside Rooks".into(), None)
        .await
        .unwrap();
    unstamped_draft_vs(&db, &t.team2, &t.b, &opp2).await;

    let f = db::match_prep_facts(&db, &t.team1, &first, Some(&opp), Some("Riverside Rooks"))
        .await
        .unwrap();
    assert_eq!(f.drafts.len(), 2);
    assert_eq!(f.plans.len(), 1);
    assert!(f.review_after.is_some());
    assert_eq!(f.played_games, vec![1, 2]);
    assert_eq!(f.reviewed_games, vec![1]);
    assert_eq!(f.earlier.len(), 1);
    assert_eq!(f.earlier[0].id, old);

    // The rematch counts none of the first match's work.
    let f = db::match_prep_facts(&db, &t.team1, &rematch, Some(&opp), Some("Riverside Rooks"))
        .await
        .unwrap();
    assert!(f.drafts.is_empty() && f.plans.is_empty() && f.review_after.is_none());
    assert!(f.played_games.is_empty() && f.reviewed_games.is_empty());
    assert_eq!(f.earlier.len(), 1, "only the unstamped draft is offered");

    // "Use for this match" stamps it: it now counts for the rematch and is
    // no longer offered for either match.
    db::use_for_match(&db, &t.team1, &rematch, "draft", &old)
        .await
        .unwrap();
    let f = db::match_prep_facts(&db, &t.team1, &rematch, Some(&opp), Some("Riverside Rooks"))
        .await
        .unwrap();
    assert_eq!(f.drafts.len(), 1);
    assert_eq!(f.drafts[0].id, old);
    assert!(f.earlier.is_empty());
    let f = db::match_prep_facts(&db, &t.team1, &first, Some(&opp), Some("Riverside Rooks"))
        .await
        .unwrap();
    assert_eq!(f.drafts.len(), 2);
    assert!(f.earlier.is_empty());
    // Work already stamped stays with its match.
    invalid(
        db::use_for_match(&db, &t.team1, &first, "draft", &old).await,
        "a draft already stamped",
    );
}

/// Item 4: "Use for this match" checks both records as the caller's team's:
/// another team's match or work is NotFound.
#[tokio::test]
async fn use_for_match_is_team_scoped() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let ours = new_match(&db, &t.team1, &t.a, 24).await;
    let theirs = new_match(&db, &t.team2, &t.b, 24).await;
    let our_draft = draft(&db, &t.team1, &t.a, None, None, None).await.unwrap();
    let their_draft = draft(&db, &t.team2, &t.b, None, None, None).await.unwrap();
    let their_plan = db::save_game_plan(&db, plan(&t.team2, None, None), &t.b)
        .await
        .unwrap();

    not_found(
        db::use_for_match(&db, &t.team2, &ours, "draft", &their_draft).await,
        "team two stamping with team one's match",
    );
    not_found(
        db::use_for_match(&db, &t.team1, &ours, "draft", &their_draft).await,
        "team one stamping team two's draft",
    );
    not_found(
        db::use_for_match(&db, &t.team1, &ours, "game_plan", &their_plan).await,
        "team one stamping team two's plan",
    );
    not_found(
        db::use_for_match(&db, &t.team1, &theirs, "draft", &our_draft).await,
        "team one using team two's match",
    );
    not_found(
        db::use_for_match(&db, &t.team1, &ours, "user", &t.a).await,
        "a table that is not prep work",
    );
    // Nothing was stamped by the refused calls.
    let d = db::get_draft_for_prefill(&db, &t.team2, &their_draft)
        .await
        .unwrap()
        .unwrap();
    assert!(d.scheduled_match_id.is_none());
    let d = db::get_draft_for_prefill(&db, &t.team1, &our_draft)
        .await
        .unwrap()
        .unwrap();
    assert!(d.scheduled_match_id.is_none());
    // The team's own pair works.
    db::use_for_match(&db, &t.team1, &ours, "draft", &our_draft)
        .await
        .unwrap();
}
