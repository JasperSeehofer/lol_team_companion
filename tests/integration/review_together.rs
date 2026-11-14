//! M165 Review together: sealed until the reveal across members, locked
//! after it, anonymous cards without an author, the generated review with
//! its stamps and empty lists, a re-save creating no items, the owner
//! check, cross-team ids, and team and account deletion with the export.
//! (The unit tests in `server::review_together::tests` cover the same rules
//! in more detail; these run through the public API only.)

use lol_team_companion::models::match_day::ScheduledMatchInput;
use lol_team_companion::models::review_together::{Phase, Prompt, SessionPoll, SessionState};
use lol_team_companion::server::account;
use lol_team_companion::server::db::{self, DbError};
use lol_team_companion::server::review_together as rt;
use surrealdb::{engine::local::Db, Surreal};

struct World {
    db: std::sync::Arc<Surreal<Db>>,
    team: String,
    lead: String,
    ann: String,
    bob: String,
    cat: String,
    other_team: String,
    otto: String,
    match_id: String,
}

async fn world() -> World {
    let db = crate::common::test_db().await;
    let mk = |n: &'static str| {
        let db = db.clone();
        async move {
            db::create_user(&db, n.into(), format!("{n}@example.com"), "h".into())
                .await
                .unwrap()
        }
    };
    let lead = mk("rt_lead").await;
    let ann = mk("rt_ann").await;
    let bob = mk("rt_bob").await;
    let cat = mk("rt_cat").await;
    let otto = mk("rt_otto").await;
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    for m in [&ann, &bob, &cat] {
        db::join_team(&db, m, &team).await.unwrap();
    }
    let other_team = db::create_team(&db, &otto, "Riverside".into(), "EUW".into())
        .await
        .unwrap();
    let match_id = add_match(&db, &team, &lead, "bo1").await;
    World {
        db,
        team,
        lead,
        ann,
        bob,
        cat,
        other_team,
        otto,
        match_id,
    }
}

async fn add_match(db: &Surreal<Db>, team: &str, by: &str, format: &str) -> String {
    let starts = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
    db::save_scheduled_match(
        db,
        team,
        by,
        ScheduledMatchInput {
            id: None,
            starts_at: starts,
            kind: "scrim".into(),
            format: format.into(),
            opponent_id: None,
            notes: None,
        },
    )
    .await
    .unwrap()
}

impl World {
    async fn state(&self, viewer: &str, session: &str) -> SessionState {
        match rt::get_session_state(&self.db, &self.team, viewer, "viewer", false, session, None)
            .await
            .unwrap()
        {
            SessionPoll::Full(s) => *s,
            SessionPoll::Unchanged { .. } => panic!("expected a full state"),
        }
    }

    async fn write(&self, who: &str, session: &str, prompt: Prompt, text: &str) -> String {
        rt::upsert_entry(&self.db, &self.team, who, session, prompt, None, text)
            .await
            .unwrap()
    }
}

#[tokio::test]
async fn cards_are_sealed_until_the_reveal_and_locked_after_it() {
    let w = world().await;
    let s = rt::open_session(&w.db, &w.team, &w.lead, &w.match_id, None, None)
        .await
        .unwrap()
        .session_id;
    let ann_card = w.write(&w.ann, &s, Prompt::OffPlan, "ANN-sealed-one").await;
    w.write(&w.bob, &s, Prompt::Decided, "BOB-sealed-two").await;
    rt::pass_prompt(&w.db, &w.team, &w.cat, &s, Prompt::ToPlan, true)
        .await
        .unwrap();
    for (viewer, own) in [
        (&w.lead, None),
        (&w.ann, Some("ANN-sealed-one")),
        (&w.bob, Some("BOB-sealed-two")),
        (&w.cat, None),
    ] {
        let st = w.state(viewer, &s).await;
        let json = serde_json::to_string(&st).unwrap();
        for t in ["ANN-sealed-one", "BOB-sealed-two"] {
            assert_eq!(json.contains(t), own == Some(t), "{t} for {viewer}");
        }
        assert_eq!(st.written_count, 3);
        assert_eq!(st.member_count, 4);
        assert!(st.cards.is_none());
    }
    // After the deadline: revealed at read time, and nothing can change.
    w.db.query(
        "UPDATE type::record('review_session', $k) SET writing_closes_at = time::now() - 1s",
    )
    .bind(("k", db::record_key(&s, "review_session")))
    .await
    .unwrap()
    .check()
    .unwrap();
    let st = w.state(&w.bob, &s).await;
    assert_eq!(st.phase, Phase::Revealed);
    assert!(st.mine.is_empty());
    let err = rt::upsert_entry(
        &w.db,
        &w.team,
        &w.ann,
        &s,
        Prompt::OffPlan,
        Some(ann_card.clone()),
        "edited after the reveal",
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, DbError::Invalid(ref m) if m == rt::LOCKED),
        "{err:?}"
    );
    assert!(rt::delete_entry(&w.db, &w.team, &w.ann, &s, &ann_card)
        .await
        .is_err());
}

#[tokio::test]
async fn anonymous_cards_and_their_proposals_name_nobody() {
    let w = world().await;
    let s = rt::open_session(&w.db, &w.team, &w.lead, &w.match_id, None, None)
        .await
        .unwrap()
        .session_id;
    let anon = w
        .write(&w.ann, &s, Prompt::OffPlan, "we lost the river")
        .await;
    w.write(&w.bob, &s, Prompt::Decided, "the second drake")
        .await;
    rt::reveal_now(&w.db, &w.team, &w.lead, &s).await.unwrap();
    rt::create_proposal(&w.db, &w.team, &s, Some(anon.clone()), "Ward the river")
        .await
        .unwrap();
    for viewer in [&w.lead, &w.ann, &w.bob, &w.cat] {
        let st = w.state(viewer, &s).await;
        let cards = st.cards.clone().unwrap();
        let off = serde_json::to_string(&cards.off_plan).unwrap();
        let props = serde_json::to_string(&st.proposals).unwrap();
        for who in [&w.lead, &w.ann, &w.bob, &w.cat] {
            assert!(!off.contains(who.as_str()) && !props.contains(who.as_str()));
        }
        for name in ["rt_lead", "rt_ann", "rt_bob", "rt_cat"] {
            assert!(!off.contains(name) && !props.contains(name), "{name}");
        }
        for field in ["author", "created", "updated", "order_key", "mine"] {
            assert!(!off.contains(field) && !props.contains(field), "{field}");
        }
        assert_eq!(cards.decided[0].author_name, "rt_bob");
    }
}

#[tokio::test]
async fn closing_writes_a_stamped_review_that_resaves_without_new_items() {
    let w = world().await;
    let s = rt::open_session(
        &w.db,
        &w.team,
        &w.ann,
        &w.match_id,
        None,
        Some("EUW1_42".into()),
    )
    .await
    .unwrap()
    .session_id;
    let e = w
        .write(&w.ann, &s, Prompt::NextTime, "ward the river")
        .await;
    w.write(&w.bob, &s, Prompt::OffPlan, "late rotations").await;
    w.write(&w.cat, &s, Prompt::ToPlan, "early drake").await;
    rt::reveal_now(&w.db, &w.team, &w.ann, &s).await.unwrap();
    let p1 = rt::create_proposal(&w.db, &w.team, &s, Some(e.clone()), "Ward river at 3:00")
        .await
        .unwrap();
    let p2 = rt::create_proposal(&w.db, &w.team, &s, None, "Rotate on the timer")
        .await
        .unwrap();
    rt::decide_vote(&w.db, &w.team, &w.bob, &s, &p1)
        .await
        .unwrap();
    rt::set_accepted(&w.db, &w.team, &w.ann, &s, &p1, true)
        .await
        .unwrap();
    rt::set_accepted(&w.db, &w.team, &w.ann, &s, &p2, true)
        .await
        .unwrap();
    // The owner must be a current member.
    assert!(matches!(
        rt::set_owner(&w.db, &w.team, &w.ann, &s, &p1, Some("rt_otto".into())).await,
        Err(DbError::Invalid(_))
    ));
    rt::set_owner(&w.db, &w.team, &w.ann, &s, &p1, Some("rt_bob".into()))
        .await
        .unwrap();
    let review_id = rt::close_session(&w.db, &w.team, &w.ann, &s).await.unwrap();
    let review = db::get_post_game_learning(&w.db, &w.team, &review_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        review.scheduled_match_id.as_deref(),
        Some(w.match_id.as_str())
    );
    assert_eq!(review.game_number, None);
    assert_eq!(review.series_id, None);
    assert_eq!(review.match_riot_id.as_deref(), Some("EUW1_42"));
    assert_eq!(review.created_by, w.ann);
    assert!(review.what_went_well.is_empty());
    assert!(review.improvements.is_empty());
    assert_eq!(review.action_items.len(), 2);
    let items = db::list_action_items(&w.db, &w.team).await.unwrap();
    assert_eq!(items.len(), 2);
    assert!(items
        .iter()
        .all(|i| i.source_review.as_deref() == Some(review_id.as_str())));
    assert!(items
        .iter()
        .any(|i| i.text == "Ward river at 3:00" && i.assigned_to.as_deref() == Some("rt_bob")));
    // Re-saving in the editor creates nothing.
    let n = db::update_review_with_action_items(&w.db, &w.team, review)
        .await
        .unwrap();
    assert_eq!(n, 0);
    assert_eq!(
        db::list_action_items(&w.db, &w.team).await.unwrap().len(),
        2
    );
    let st = w.state(&w.cat, &s).await;
    assert_eq!(st.phase, Phase::Closed);
    assert_eq!(st.voices.unwrap().label(), "2 of 4 voices");
    let view = rt::session_for_review(&w.db, &w.team, &w.cat, &review_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(view.accepted.len(), 2);
}

#[tokio::test]
async fn the_owner_check_refuses_non_members() {
    let w = world().await;
    assert!(matches!(
        db::check_owner(&w.db, &w.team, Some("rt_otto".into())).await,
        Err(DbError::Invalid(_))
    ));
    assert_eq!(
        db::check_owner(&w.db, &w.team, Some("RT_ANN".into()))
            .await
            .unwrap()
            .as_deref(),
        Some("rt_ann")
    );
    assert_eq!(db::check_owner(&w.db, &w.team, None).await.unwrap(), None);
}

#[tokio::test]
async fn another_teams_session_and_match_ids_are_not_found() {
    let w = world().await;
    let s = rt::open_session(&w.db, &w.team, &w.lead, &w.match_id, None, None)
        .await
        .unwrap()
        .session_id;
    let card = w.write(&w.ann, &s, Prompt::Decided, "x").await;
    let (t, o) = (&w.other_team, &w.otto);
    assert!(matches!(
        rt::get_session_state(&w.db, t, o, "o", true, &s, None).await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        rt::open_session(&w.db, t, o, &w.match_id, None, None).await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        rt::upsert_entry(&w.db, t, o, &s, Prompt::Decided, Some(card.clone()), "y").await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        rt::pass_prompt(&w.db, t, o, &s, Prompt::Decided, true).await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        rt::take_over(&w.db, t, o, true, &s).await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        rt::close_session(&w.db, t, o, &s).await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        rt::sessions_of_match(&w.db, t, &w.match_id).await,
        Err(DbError::NotFound)
    ));
    assert!(rt::session_pattern_texts(&w.db, t)
        .await
        .unwrap()
        .is_empty());
    // A member of this team cannot edit a teammate's card either.
    assert!(matches!(
        rt::upsert_entry(&w.db, &w.team, &w.bob, &s, Prompt::Decided, Some(card), "y").await,
        Err(DbError::NotFound)
    ));
}

#[tokio::test]
async fn deletion_and_export_cover_cards_and_votes() {
    let w = world().await;
    let s = rt::open_session(&w.db, &w.team, &w.lead, &w.match_id, None, None)
        .await
        .unwrap()
        .session_id;
    let anon = w
        .write(&w.ann, &s, Prompt::OffPlan, "ANN-export-anon")
        .await;
    w.write(&w.ann, &s, Prompt::Decided, "ANN-export-named")
        .await;
    w.write(&w.bob, &s, Prompt::Decided, "BOB-not-exported")
        .await;
    rt::reveal_now(&w.db, &w.team, &w.lead, &s).await.unwrap();
    rt::plus_one(&w.db, &w.team, &w.ann, &s, &anon)
        .await
        .unwrap();

    let doc = account::export_user_data(&w.db, &w.ann).await.unwrap();
    let cards = doc["data"]["review_cards_written"].to_string();
    assert!(cards.contains("ANN-export-anon") && cards.contains("ANN-export-named"));
    assert!(!cards.contains("BOB-not-exported"));
    assert_eq!(
        doc["data"]["review_votes"].as_array().map(|a| a.len()),
        Some(1)
    );

    account::delete_account(&w.db, &w.ann).await.unwrap();
    let st = w.state(&w.bob, &s).await;
    let cards = st.cards.unwrap();
    assert!(cards
        .decided
        .iter()
        .any(|c| c.author_name == account::FORMER_MEMBER_NAME));
    assert_eq!(cards.off_plan.len(), 1);
    assert_eq!(cards.off_plan[0].plus_ones, 0);

    // The leader deletes the team (once alone in it): every session row goes.
    db::remove_team_member(&w.db, &w.team, &w.bob)
        .await
        .unwrap();
    db::remove_team_member(&w.db, &w.team, &w.cat)
        .await
        .unwrap();
    let out = account::delete_account(&w.db, &w.lead).await.unwrap();
    assert!(out.team_deleted);
    for table in [
        "review_session",
        "review_entry",
        "review_vote",
        "review_proposal",
    ] {
        let mut r =
            w.db.query(format!("SELECT VALUE id FROM {table}"))
                .await
                .unwrap();
        let ids: Vec<surrealdb::types::RecordId> = r.take(0).unwrap();
        assert!(ids.is_empty(), "{table} rows left");
    }
}
