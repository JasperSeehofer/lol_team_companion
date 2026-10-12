//! M7 community loop: bug reports v2, the what-next board, surveys, the
//! ledger and site texts. Cross-user cases follow the security suite's
//! shape: user B acting on user A's rows gets "not found" (or nothing), and
//! A's rows are unchanged.

use std::sync::Arc;

use lol_team_companion::models::bug_report::{BugStatus, BugTriage};
use lol_team_companion::models::community::{
    LedgerInput, ProposalInput, SurveyAnswer, SurveyInput, SurveyStatus, SurveyTrigger,
    TEXT_MONEY_POLICY,
};
use lol_team_companion::server::community::store;
use lol_team_companion::server::db::{self, DbError};
use surrealdb::types::SurrealValue;
use surrealdb::{engine::local::Db, Surreal};

use crate::common;

async fn user(db: &Surreal<Db>, name: &str) -> String {
    db::create_user(db, name.into(), format!("{name}@example.com"), "h".into())
        .await
        .unwrap()
}

async fn report(db: &Surreal<Db>, user_id: &str, category: &str, text: &str) -> String {
    db::create_bug_report(
        db,
        user_id,
        "/draft".into(),
        "Draft board".into(),
        text.into(),
        category.into(),
        Some(1280),
        Some(800),
    )
    .await
    .unwrap();
    store::list_my_bug_reports(db, user_id)
        .await
        .unwrap()
        .into_iter()
        .find(|r| r.description == text)
        .unwrap()
        .id
}

fn triage(id: &str, status: &str, on_board: bool, title: &str) -> BugTriage {
    BugTriage {
        id: id.into(),
        status: status.into(),
        on_board,
        public_title: title.into(),
        public_note: String::new(),
        admin_note: String::new(),
        fixed_in: String::new(),
    }
}

fn assert_not_found<T: std::fmt::Debug>(r: Result<T, DbError>, what: &str) {
    match r {
        Err(DbError::NotFound) => {}
        other => panic!("{what}: expected NotFound, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Bug reports
// ---------------------------------------------------------------------------

#[tokio::test]
async fn reporters_see_only_their_own_reports() {
    let db = common::test_db().await;
    let a = user(&db, "alice").await;
    let b = user(&db, "bob").await;
    let ra = report(&db, &a, "bug", "Alice's private words").await;
    report(&db, &b, "wishlist", "Bob's wish").await;

    let mine_a = store::list_my_bug_reports(&db, &a).await.unwrap();
    assert_eq!(mine_a.len(), 1);
    assert_eq!(mine_a[0].status, "new");
    assert!(!mine_a[0].updated);
    let mine_b = store::list_my_bug_reports(&db, &b).await.unwrap();
    assert_eq!(mine_b.len(), 1);
    assert!(mine_b.iter().all(|r| r.id != ra));

    // B marking "seen" touches only B's rows.
    let admin = user(&db, "admin").await;
    let mut t = triage(&ra, "triaged", false, "");
    t.public_note = "We can reproduce it.".into();
    store::admin_update_bug_report(&db, &admin, &t)
        .await
        .unwrap();
    store::mark_my_reports_seen(&db, &b).await.unwrap();
    assert_eq!(store::count_my_report_updates(&db, &a).await.unwrap(), 1);
    let mine_a = store::list_my_bug_reports(&db, &a).await.unwrap();
    assert!(mine_a[0].updated);
    assert_eq!(
        mine_a[0].public_note.as_deref(),
        Some("We can reproduce it.")
    );
    store::mark_my_reports_seen(&db, &a).await.unwrap();
    assert_eq!(store::count_my_report_updates(&db, &a).await.unwrap(), 0);
}

#[tokio::test]
async fn the_board_shows_only_admin_written_text_of_public_reports() {
    let db = common::test_db().await;
    let a = user(&db, "alice").await;
    let admin = user(&db, "admin").await;
    let public = report(&db, &a, "bug", "SECRET description with team details").await;
    let private = report(&db, &a, "bug", "Another secret").await;
    let mut t = triage(&public, "planned", true, "Pick order resets");
    t.public_note = "A fix is planned.".into();
    t.admin_note = "internal only".into();
    store::admin_update_bug_report(&db, &admin, &t)
        .await
        .unwrap();
    store::admin_update_bug_report(&db, &admin, &triage(&private, "triaged", false, "Hidden"))
        .await
        .unwrap();

    let board = store::list_bug_board(&db).await.unwrap();
    assert_eq!(board.len(), 1);
    assert_eq!(board[0].public_title, "Pick order resets");
    assert_eq!(board[0].status, "planned");
    let json = serde_json::to_string(&board).unwrap();
    assert!(!json.contains("SECRET"));
    assert!(!json.contains("internal only"));
    assert!(!json.contains("/draft"));
    assert!(!json.contains(&a));
}

#[tokio::test]
async fn triage_rules_are_enforced() {
    let fake_exists = |s: &str| s == "milestone-m7";
    let mut t = triage("bug_report:x", "triaged", true, "");
    assert!(matches!(
        store::validate_triage(&t, fake_exists),
        Err(DbError::Invalid(m)) if m.contains("title")
    ));
    t.public_title = "Title".into();
    t.status = "new".into();
    assert!(
        store::validate_triage(&t, fake_exists).is_err(),
        "New is not on the board"
    );
    t.status = "fixed".into();
    assert!(matches!(
        store::validate_triage(&t, fake_exists),
        Err(DbError::Invalid(m)) if m.contains("changelog entry that announced")
    ));
    t.fixed_in = "unknown".into();
    assert!(matches!(
        store::validate_triage(&t, fake_exists),
        Err(DbError::Invalid(m)) if m.contains("not in this version")
    ));
    t.fixed_in = "milestone-m7".into();
    assert_eq!(
        store::validate_triage(&t, fake_exists).unwrap().fixed_in,
        "milestone-m7"
    );
    t.status = "planned".into();
    assert_eq!(
        store::validate_triage(&t, fake_exists).unwrap().fixed_in,
        "",
        "cleared"
    );
    t.status = "open".into();
    assert!(store::validate_triage(&t, fake_exists).is_err());
}

#[tokio::test]
async fn admin_list_filters_by_status_and_names_the_reporter() {
    let db = common::test_db().await;
    let a = user(&db, "alice").await;
    let admin = user(&db, "admin").await;
    let r1 = report(&db, &a, "bug", "one").await;
    report(&db, &a, "bug", "two").await;
    store::admin_update_bug_report(&db, &admin, &triage(&r1, "triaged", false, ""))
        .await
        .unwrap();
    let new = store::admin_list_bug_reports(&db, Some(BugStatus::New))
        .await
        .unwrap();
    assert_eq!(new.len(), 1);
    assert_eq!(new[0].reporter, "alice");
    let all = store::admin_list_bug_reports(&db, None).await.unwrap();
    assert_eq!(all.len(), 2);
    let triaged = all.iter().find(|r| r.id == r1).unwrap();
    assert_eq!(triaged.triaged_by.as_deref(), Some("admin"));
    assert!(triaged.updated_at.is_some());
    assert_not_found(
        store::admin_update_bug_report(
            &db,
            &admin,
            &triage("bug_report:nope", "triaged", false, ""),
        )
        .await,
        "unknown report",
    );
}

#[tokio::test]
async fn only_wishlist_reports_become_proposals() {
    let db = common::test_db().await;
    let a = user(&db, "alice").await;
    let admin = user(&db, "admin").await;
    let bug = report(&db, &a, "bug", "a bug").await;
    let wish = report(&db, &a, "wishlist", "A dark   mode\nfor the draft board").await;
    assert!(matches!(
        store::admin_bug_to_proposal(&db, &admin, &bug).await,
        Err(DbError::Invalid(_))
    ));
    let p = store::admin_bug_to_proposal(&db, &admin, &wish)
        .await
        .unwrap();
    let again = store::admin_bug_to_proposal(&db, &admin, &wish)
        .await
        .unwrap();
    assert_eq!(p, again, "linked once");
    let list = store::admin_list_proposals(&db).await.unwrap();
    let created = list.iter().find(|x| x.id == p).unwrap();
    assert_eq!(created.title, "A dark mode for the draft board");
    assert_eq!(created.status, "draft");
    assert!(!created.published);
    assert_eq!(created.source, "wishlist");
    assert_eq!(created.wishlist_report.as_deref(), Some(wish.as_str()));
    // Users never see it.
    assert!(store::list_proposals(&db, &a)
        .await
        .unwrap()
        .iter()
        .all(|x| x.id != p));
}

// ---------------------------------------------------------------------------
// What-next board
// ---------------------------------------------------------------------------

async fn proposal(
    db: &Surreal<Db>,
    admin: &str,
    title: &str,
    status: &str,
    published: bool,
) -> String {
    store::admin_save_proposal(
        db,
        admin,
        &ProposalInput {
            id: None,
            title: title.into(),
            summary: String::new(),
            status: status.into(),
            status_note: String::new(),
            shipped_in: String::new(),
            published,
        },
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn one_vote_per_user_and_withdrawal_is_own_only() {
    let db = common::test_db().await;
    let admin = user(&db, "admin").await;
    let a = user(&db, "alice").await;
    let b = user(&db, "bob").await;
    let p = proposal(&db, &admin, "Idea", "proposed", true).await;

    assert_eq!(store::set_vote(&db, &a, &p, true).await.unwrap(), 1);
    assert_eq!(
        store::set_vote(&db, &a, &p, true).await.unwrap(),
        1,
        "idempotent"
    );
    // B withdrawing does not touch A's vote.
    assert_eq!(store::set_vote(&db, &b, &p, false).await.unwrap(), 1);
    assert_eq!(store::set_vote(&db, &b, &p, true).await.unwrap(), 2);
    let board = store::list_proposals(&db, &a).await.unwrap();
    let row = board.iter().find(|x| x.id == p).unwrap();
    assert!(row.voted_by_me);
    assert_eq!(row.votes, 2);
    assert_eq!(store::set_vote(&db, &a, &p, false).await.unwrap(), 1);
    let board = store::list_proposals(&db, &a).await.unwrap();
    assert!(!board.iter().find(|x| x.id == p).unwrap().voted_by_me);
}

#[tokio::test]
async fn concurrent_votes_leave_one_row() {
    let db = common::test_db().await;
    let admin = user(&db, "admin").await;
    let a = user(&db, "alice").await;
    let p = proposal(&db, &admin, "Idea", "planned", true).await;
    let mut handles = Vec::new();
    for _ in 0..10 {
        let db = Arc::clone(&db);
        let (a, p) = (a.clone(), p.clone());
        handles.push(tokio::spawn(async move {
            store::set_vote(&db, &a, &p, true).await
        }));
    }
    for h in handles {
        h.await.unwrap().unwrap();
    }
    assert_eq!(store::set_vote(&db, &a, &p, true).await.unwrap(), 1);
}

#[tokio::test]
async fn hidden_and_closed_proposals_refuse_votes() {
    let db = common::test_db().await;
    let admin = user(&db, "admin").await;
    let a = user(&db, "alice").await;
    let unpublished = proposal(&db, &admin, "Hidden", "proposed", false).await;
    let draft = proposal(&db, &admin, "Draft", "draft", false).await;
    let shipped = store::admin_save_proposal(
        &db,
        &admin,
        &ProposalInput {
            id: None,
            title: "Shipped".into(),
            status: "shipped".into(),
            shipped_in: "milestone-m5-remaining-pages-navigation-mobile".into(),
            published: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_not_found(
        store::set_vote(&db, &a, &unpublished, true).await,
        "unpublished",
    );
    assert_not_found(store::set_vote(&db, &a, &draft, true).await, "draft");
    assert_not_found(
        store::set_vote(&db, &a, "proposal:nope", true).await,
        "unknown",
    );
    assert!(matches!(
        store::set_vote(&db, &a, &shipped, true).await,
        Err(DbError::Invalid(m)) if m == "Voting is closed for this idea."
    ));
    let board = store::list_proposals(&db, &a).await.unwrap();
    assert!(board.iter().all(|p| p.id != unpublished && p.id != draft));
    assert!(board.iter().any(|p| p.id == shipped));
    // Seeded proposals stay off the board until published.
    assert!(board.iter().all(|p| p.title != "Light theme"));
}

#[tokio::test]
async fn proposals_with_votes_cannot_be_deleted() {
    let db = common::test_db().await;
    let admin = user(&db, "admin").await;
    let a = user(&db, "alice").await;
    let p = proposal(&db, &admin, "Idea", "proposed", true).await;
    store::set_vote(&db, &a, &p, true).await.unwrap();
    assert!(matches!(
        store::admin_delete_proposal(&db, &p).await,
        Err(DbError::Invalid(_))
    ));
    store::set_vote(&db, &a, &p, false).await.unwrap();
    store::admin_delete_proposal(&db, &p).await.unwrap();
    assert_not_found(store::admin_delete_proposal(&db, &p).await, "deleted");
}

#[tokio::test]
async fn proposal_rules() {
    let exists = |s: &str| s == "ok";
    let base = ProposalInput {
        title: "T".into(),
        status: "declined".into(),
        ..Default::default()
    };
    assert!(
        store::validate_proposal(&base, exists).is_err(),
        "declined needs a note"
    );
    let mut p = base.clone();
    p.status = "shipped".into();
    assert!(store::validate_proposal(&p, exists).is_err());
    p.shipped_in = "ok".into();
    assert!(store::validate_proposal(&p, exists).is_ok());
    let mut p = base;
    p.status = "draft".into();
    p.published = true;
    assert!(
        store::validate_proposal(&p, exists).is_err(),
        "drafts are not published"
    );
    p.title = " ".into();
    assert!(store::validate_proposal(&p, exists).is_err());
}

// ---------------------------------------------------------------------------
// Surveys
// ---------------------------------------------------------------------------

async fn live_survey(db: &Surreal<Db>, admin: &str, trigger: &str, kind: &str) -> String {
    let id = store::admin_save_survey(
        db,
        admin,
        &SurveyInput {
            id: None,
            question: format!("Question for {trigger}?"),
            kind: kind.into(),
            choices_text: "Yes\nNo".into(),
            scale_low_label: "Hard".into(),
            scale_high_label: "Easy".into(),
            trigger: trigger.into(),
            note: String::new(),
        },
    )
    .await
    .unwrap();
    let hash = hash_of(db, &id).await;
    store::admin_approve_survey(db, admin, &id, &hash)
        .await
        .unwrap();
    store::admin_set_survey_status(db, admin, &id, SurveyStatus::Live)
        .await
        .unwrap();
    id
}

async fn hash_of(db: &Surreal<Db>, id: &str) -> String {
    store::admin_list_surveys(db)
        .await
        .unwrap()
        .into_iter()
        .find(|s| s.id == id)
        .unwrap()
        .wording_hash
}

#[tokio::test]
async fn unapproved_surveys_never_reach_users() {
    let db = common::test_db().await;
    let a = user(&db, "alice").await;
    // The seeded drafts exist but are not approved.
    for t in SurveyTrigger::ALL {
        store::note_trigger(&db, &a, t, "").await.unwrap();
    }
    assert_eq!(
        store::get_pending_survey(&db, &a, "/what-next")
            .await
            .unwrap(),
        None
    );
    let admin = user(&db, "admin").await;
    let id = store::admin_save_survey(
        &db,
        &admin,
        &SurveyInput {
            question: "Q?".into(),
            kind: "short_text".into(),
            trigger: "review_saved".into(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        store::admin_set_survey_status(&db, &admin, &id, SurveyStatus::Live).await,
        Err(DbError::Invalid(m)) if m == "Only an approved survey can go live."
    ));
    assert!(matches!(
        store::admin_approve_survey(&db, &admin, &id, "stale-hash").await,
        Err(DbError::Invalid(_))
    ));
}

#[tokio::test]
async fn a_wording_edit_withdraws_the_approval() {
    let db = common::test_db().await;
    let admin = user(&db, "admin").await;
    let id = live_survey(&db, &admin, "review_saved", "single_choice").await;
    let changed = SurveyInput {
        id: Some(id.clone()),
        question: "A different question?".into(),
        kind: "single_choice".into(),
        choices_text: "Yes\nNo".into(),
        trigger: "review_saved".into(),
        ..Default::default()
    };
    store::admin_save_survey(&db, &admin, &changed)
        .await
        .unwrap();
    let s = store::admin_list_surveys(&db)
        .await
        .unwrap()
        .into_iter()
        .find(|s| s.id == id)
        .unwrap();
    assert_eq!(s.status, "draft");
    assert!(s.approval_withdrawn());
    assert!(
        store::admin_set_survey_status(&db, &admin, &id, SurveyStatus::Live)
            .await
            .is_err()
    );
    // A note-only edit keeps it.
    let id2 = live_survey(&db, &admin, "draft_saved_3", "scale").await;
    let note_only = SurveyInput {
        id: Some(id2.clone()),
        question: "Question for draft_saved_3?".into(),
        kind: "scale".into(),
        scale_low_label: "Hard".into(),
        scale_high_label: "Easy".into(),
        trigger: "draft_saved_3".into(),
        note: "Why we ask".into(),
        ..Default::default()
    };
    store::admin_save_survey(&db, &admin, &note_only)
        .await
        .unwrap();
    let s2 = store::admin_list_surveys(&db)
        .await
        .unwrap()
        .into_iter()
        .find(|s| s.id == id2)
        .unwrap();
    assert_eq!(s2.status, "live");
    assert!(s2.approval_valid());
}

#[tokio::test]
async fn delivery_answer_and_weekly_cap() {
    let db = common::test_db().await;
    let admin = user(&db, "admin").await;
    let a = user(&db, "alice").await;
    let s1 = live_survey(&db, &admin, "review_saved", "single_choice").await;
    live_survey(&db, &admin, "draft_saved_3", "scale").await;

    store::note_trigger(&db, &a, SurveyTrigger::ReviewSaved, "")
        .await
        .unwrap();
    store::note_trigger(&db, &a, SurveyTrigger::DraftSaved3, "")
        .await
        .unwrap();
    let shown = store::get_pending_survey(&db, &a, "/post-game")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(shown.kind, "single_choice");
    // Reloading shows the same prompt again until it is answered.
    let again = store::get_pending_survey(&db, &a, "/")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(again.prompt_id, shown.prompt_id);

    assert!(
        store::answer_survey(&db, &a, &shown.prompt_id, &SurveyAnswer::Choice(5))
            .await
            .is_err()
    );
    store::answer_survey(&db, &a, &shown.prompt_id, &SurveyAnswer::Choice(1))
        .await
        .unwrap();
    assert!(matches!(
        store::answer_survey(&db, &a, &shown.prompt_id, &SurveyAnswer::Choice(0)).await,
        Err(DbError::Invalid(m)) if m == "You already answered this question."
    ));
    // The second survey waits for next week.
    assert_eq!(
        store::get_pending_survey(&db, &a, "/draft").await.unwrap(),
        None
    );

    let results = store::admin_survey_results(&db, &s1).await.unwrap();
    assert_eq!(results.counts.answered, 1);
    assert_eq!(results.versions[0].choice_counts[1], ("No".to_string(), 1));

    // A week later the next one is delivered.
    db.query("UPDATE survey_prompt SET shown_at = time::now() - 8d WHERE shown_at != NONE")
        .await
        .unwrap()
        .check()
        .unwrap();
    let next = store::get_pending_survey(&db, &a, "/draft")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(next.kind, "scale");
}

#[tokio::test]
async fn prompts_are_private_to_their_user() {
    let db = common::test_db().await;
    let admin = user(&db, "admin").await;
    let a = user(&db, "alice").await;
    let b = user(&db, "bob").await;
    live_survey(&db, &admin, "review_saved", "short_text").await;
    store::note_trigger(&db, &a, SurveyTrigger::ReviewSaved, "")
        .await
        .unwrap();
    let p = store::get_pending_survey(&db, &a, "/")
        .await
        .unwrap()
        .unwrap();
    assert_not_found(
        store::answer_survey(&db, &b, &p.prompt_id, &SurveyAnswer::Text("hijack".into())).await,
        "answer another user's prompt",
    );
    assert_not_found(
        store::dismiss_survey(&db, &b, &p.prompt_id).await,
        "dismiss another user's prompt",
    );
    // Still answerable by A.
    store::answer_survey(&db, &a, &p.prompt_id, &SurveyAnswer::Text("mine".into()))
        .await
        .unwrap();
}

#[tokio::test]
async fn dismissed_prompts_never_return_and_opt_out_expires_pending() {
    let db = common::test_db().await;
    let admin = user(&db, "admin").await;
    let a = user(&db, "alice").await;
    let b = user(&db, "bob").await;
    live_survey(&db, &admin, "review_saved", "single_choice").await;
    store::note_trigger(&db, &a, SurveyTrigger::ReviewSaved, "")
        .await
        .unwrap();
    let p = store::get_pending_survey(&db, &a, "/")
        .await
        .unwrap()
        .unwrap();
    store::dismiss_survey(&db, &a, &p.prompt_id).await.unwrap();
    store::note_trigger(&db, &a, SurveyTrigger::ReviewSaved, "")
        .await
        .unwrap();
    db.query("UPDATE survey_prompt SET shown_at = time::now() - 8d")
        .await
        .unwrap()
        .check()
        .unwrap();
    assert_eq!(store::get_pending_survey(&db, &a, "/").await.unwrap(), None);

    store::note_trigger(&db, &b, SurveyTrigger::ReviewSaved, "")
        .await
        .unwrap();
    store::set_survey_opt_out(&db, &b, true).await.unwrap();
    assert!(store::survey_opt_out(&db, &b).await.unwrap());
    assert_eq!(store::get_pending_survey(&db, &b, "/").await.unwrap(), None);
    store::set_survey_opt_out(&db, &b, false).await.unwrap();
    assert_eq!(
        store::get_pending_survey(&db, &b, "/").await.unwrap(),
        None,
        "the pending prompt expired when questions were turned off"
    );
}

#[tokio::test]
async fn stale_pending_prompts_expire_and_paused_surveys_stay_silent() {
    let db = common::test_db().await;
    let admin = user(&db, "admin").await;
    let a = user(&db, "alice").await;
    let b = user(&db, "bob").await;
    let s = live_survey(&db, &admin, "review_saved", "single_choice").await;
    store::note_trigger(&db, &a, SurveyTrigger::ReviewSaved, "")
        .await
        .unwrap();
    db.query("UPDATE survey_prompt SET created_at = time::now() - 15d")
        .await
        .unwrap()
        .check()
        .unwrap();
    assert_eq!(store::get_pending_survey(&db, &a, "/").await.unwrap(), None);

    store::note_trigger(&db, &b, SurveyTrigger::ReviewSaved, "")
        .await
        .unwrap();
    store::admin_set_survey_status(&db, &admin, &s, SurveyStatus::Paused)
        .await
        .unwrap();
    assert_eq!(store::get_pending_survey(&db, &b, "/").await.unwrap(), None);
    store::admin_set_survey_status(&db, &admin, &s, SurveyStatus::Live)
        .await
        .unwrap();
    assert!(store::get_pending_survey(&db, &b, "/")
        .await
        .unwrap()
        .is_some());
}

#[tokio::test]
async fn concurrent_triggers_leave_one_prompt_and_patches_count_once_each() {
    let db = common::test_db().await;
    let admin = user(&db, "admin").await;
    let a = user(&db, "alice").await;
    live_survey(&db, &admin, "review_saved", "single_choice").await;
    let mut handles = Vec::new();
    for _ in 0..2 {
        let db = Arc::clone(&db);
        let a = a.clone();
        handles.push(tokio::spawn(async move {
            store::note_trigger(&db, &a, SurveyTrigger::ReviewSaved, "").await
        }));
    }
    for h in handles {
        h.await.unwrap().unwrap();
    }
    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct N {
        n: i64,
    }
    let mut r = db
        .query("SELECT count() AS n FROM survey_prompt GROUP ALL")
        .await
        .unwrap();
    let n: Option<N> = r.take(0).unwrap();
    assert_eq!(n.unwrap().n, 1);

    // Patch surveys: one prompt per detected version, with the version in
    // the question.
    let patch = store::admin_save_survey(
        &db,
        &admin,
        &SurveyInput {
            question: "Has patch {patch} changed how you draft?".into(),
            kind: "single_choice".into(),
            choices_text: "Yes\nNo".into(),
            trigger: "patch_changed".into(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let h = hash_of(&db, &patch).await;
    store::admin_approve_survey(&db, &admin, &patch, &h)
        .await
        .unwrap();
    store::admin_set_survey_status(&db, &admin, &patch, SurveyStatus::Live)
        .await
        .unwrap();
    let b = user(&db, "bob").await;
    db.query("CREATE patch_change SET old_version = '16.20.1', new_version = '16.21.1', detected_at = time::now()")
        .await
        .unwrap()
        .check()
        .unwrap();
    let p = store::get_pending_survey(&db, &b, "/")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(p.question, "Has patch 16.21.1 changed how you draft?");
}

// ---------------------------------------------------------------------------
// Ledger and site texts
// ---------------------------------------------------------------------------

fn month(
    m: &str,
    support: &str,
    hosting: &str,
) -> lol_team_companion::models::community::LedgerMonth {
    lol_team_companion::models::community::validate_ledger(
        &LedgerInput {
            month: m.into(),
            support: support.into(),
            hosting: hosting.into(),
            ..Default::default()
        },
        "2099-12",
    )
    .unwrap()
}

#[tokio::test]
async fn ledger_drafts_stay_private_until_published() {
    let db = common::test_db().await;
    let admin = user(&db, "admin").await;
    assert!(
        !store::admin_save_ledger_month(&db, &admin, &month("2026-09", "20", "5"))
            .await
            .unwrap()
    );
    assert!(store::list_published_ledger(&db).await.unwrap().is_empty());
    store::admin_publish_ledger_month(&db, &admin, "2026-09", true)
        .await
        .unwrap();
    let public = store::list_published_ledger(&db).await.unwrap();
    assert_eq!(public.len(), 1);
    assert_eq!(public[0].to_maintainer_cents(), 1500);
    assert!(public[0].revised_at.is_none());

    // An edit after publishing is a revision.
    assert!(
        store::admin_save_ledger_month(&db, &admin, &month("2026-09", "20", "6"))
            .await
            .unwrap()
    );
    let public = store::list_published_ledger(&db).await.unwrap();
    assert!(public[0].revised_at.is_some());
    assert_eq!(public[0].hosting_cents, 600);
    assert!(store::admin_delete_ledger_month(&db, "2026-09")
        .await
        .is_err());

    // Unpublishing is a typo fix within 24 hours only.
    db.query("UPDATE ledger_month SET published_at = time::now() - 25h")
        .await
        .unwrap()
        .check()
        .unwrap();
    assert!(matches!(
        store::admin_publish_ledger_month(&db, &admin, "2026-09", false).await,
        Err(DbError::Invalid(_))
    ));

    // One row per month; drafts can be deleted.
    store::admin_save_ledger_month(&db, &admin, &month("2026-10", "1", "5"))
        .await
        .unwrap();
    store::admin_save_ledger_month(&db, &admin, &month("2026-10", "2", "5"))
        .await
        .unwrap();
    assert_eq!(store::admin_list_ledger(&db).await.unwrap().len(), 2);
    store::admin_delete_ledger_month(&db, "2026-10")
        .await
        .unwrap();
    assert_not_found(
        store::admin_publish_ledger_month(&db, &admin, "2026-10", true).await,
        "deleted month",
    );
}

#[tokio::test]
async fn the_schema_rejects_bad_ledger_values() {
    let db = common::test_db().await;
    assert!(db
        .query("CREATE ledger_month SET month = '2026-13', support_cents = 1")
        .await
        .unwrap()
        .check()
        .is_err());
    assert!(db
        .query("CREATE ledger_month SET month = '2026-10', support_cents = -1")
        .await
        .unwrap()
        .check()
        .is_err());
}

#[tokio::test]
async fn site_texts_need_approval_of_the_exact_body() {
    let db = common::test_db().await;
    let admin = user(&db, "admin").await;
    assert!(store::approved_texts(&db).await.unwrap().is_empty());
    let texts = store::list_site_texts(&db).await.unwrap();
    assert_eq!(texts.len(), 5);
    let policy = texts.iter().find(|t| t.slug == TEXT_MONEY_POLICY).unwrap();
    assert!(!policy.approved);
    assert!(matches!(
        store::admin_approve_site_text(&db, &admin, TEXT_MONEY_POLICY, "something else").await,
        Err(DbError::Invalid(_))
    ));
    store::admin_approve_site_text(&db, &admin, TEXT_MONEY_POLICY, &policy.body)
        .await
        .unwrap();
    assert!(store::approved_texts(&db)
        .await
        .unwrap()
        .contains_key(TEXT_MONEY_POLICY));
    // Saving the same body keeps the approval; a change withdraws it.
    store::admin_save_site_text(&db, &admin, TEXT_MONEY_POLICY, &policy.body)
        .await
        .unwrap();
    assert!(store::approved_texts(&db)
        .await
        .unwrap()
        .contains_key(TEXT_MONEY_POLICY));
    store::admin_save_site_text(&db, &admin, TEXT_MONEY_POLICY, "Changed.")
        .await
        .unwrap();
    assert!(store::approved_texts(&db).await.unwrap().is_empty());
    assert_not_found(
        store::admin_save_site_text(&db, &admin, "nope", "x").await,
        "unknown text",
    );
}

#[tokio::test]
async fn the_admin_overview_counts() {
    let db = common::test_db().await;
    let a = user(&db, "alice").await;
    report(&db, &a, "bug", "one").await;
    let o = store::admin_overview(&db).await.unwrap();
    assert_eq!(o.bugs_new, 1);
    assert_eq!(o.surveys_waiting, 4, "the seeded drafts wait for approval");
    assert_eq!(o.surveys_live, 0);
    assert_eq!(o.proposals_unpublished, 17);
    assert_eq!(o.proposals_open, 0);
    assert_eq!(o.ledger_last_published, None);
    assert_eq!(o.texts_waiting, 5);
    assert_eq!(o.texts_total, 5);
}
