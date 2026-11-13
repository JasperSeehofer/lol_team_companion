//! Unit tests of the review session (in-memory database): sealing, the
//! read-time reveal, anonymity of the off_plan group, caps, decide and
//! close, scope, the quiet nudge and the links other deletions clear.

use super::*;
use crate::models::review_together::Prompt;
use crate::server::test_helpers::test_db;

struct World {
    db: std::sync::Arc<Surreal<Db>>,
    team: String,
    /// Leader (lead), members (ann, bob, cat), coach (coa).
    lead: String,
    ann: String,
    bob: String,
    cat: String,
    coa: String,
    /// Another team (out of scope).
    other_team: String,
    otto: String,
    /// A Bo1 match of the team that started an hour ago.
    match_id: String,
}

async fn user(db: &Surreal<Db>, name: &str) -> String {
    db::create_user(db, name.into(), format!("{name}@example.com"), "h".into())
        .await
        .unwrap()
}

async fn add_match(db: &Surreal<Db>, team: &str, by: &str, format: &str, hours_ago: i64) -> String {
    let mut r = db
        .query("CREATE scheduled_match SET team = type::record('team', $t), starts_at = time::now() - <duration>$ago, kind = 'scrim', format = $f, created_by = type::record('user', $u) RETURN id")
        .bind(("t", record_key(team, "team")))
        .bind(("u", record_key(by, "user")))
        .bind(("f", format.to_string()))
        .bind(("ago", format!("{hours_ago}h")))
        .await
        .unwrap();
    let row: Option<IdRow> = r.take(0).unwrap();
    row.unwrap().id.to_sql()
}

async fn world() -> World {
    let db = test_db().await;
    let lead = user(&db, "lead").await;
    let ann = user(&db, "ann").await;
    let bob = user(&db, "bob").await;
    let cat = user(&db, "cat").await;
    let coa = user(&db, "coa").await;
    let otto = user(&db, "otto").await;
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    for m in [&ann, &bob, &cat, &coa] {
        db::join_team(&db, m, &team).await.unwrap();
    }
    db.query("UPDATE team_member SET roster_type = 'coach' WHERE user = type::record('user', $u)")
        .bind(("u", record_key(&coa, "user")))
        .await
        .unwrap()
        .check()
        .unwrap();
    let other_team = db::create_team(&db, &otto, "Riverside".into(), "EUW".into())
        .await
        .unwrap();
    let match_id = add_match(&db, &team, &lead, "bo1", 1).await;
    World {
        db,
        team,
        lead,
        ann,
        bob,
        cat,
        coa,
        other_team,
        otto,
        match_id,
    }
}

impl World {
    async fn open(&self, by: &str) -> String {
        open_session(&self.db, &self.team, by, &self.match_id, None, None)
            .await
            .unwrap()
            .session_id
    }

    async fn state(&self, viewer: &str, session: &str) -> SessionState {
        let name = self.name(viewer);
        match get_session_state(&self.db, &self.team, viewer, name, false, session, None)
            .await
            .unwrap()
        {
            SessionPoll::Full(s) => *s,
            SessionPoll::Unchanged { .. } => panic!("expected a full state"),
        }
    }

    fn name(&self, id: &str) -> &'static str {
        [
            (&self.lead, "lead"),
            (&self.ann, "ann"),
            (&self.bob, "bob"),
            (&self.cat, "cat"),
            (&self.coa, "coa"),
            (&self.otto, "otto"),
        ]
        .into_iter()
        .find(|(u, _)| u.as_str() == id)
        .map(|(_, n)| n)
        .unwrap()
    }

    async fn write(&self, who: &str, session: &str, prompt: Prompt, text: &str) -> String {
        upsert_entry(&self.db, &self.team, who, session, prompt, None, text)
            .await
            .unwrap()
    }

    /// Move the deadline into the past (as if 24 h went by).
    async fn expire(&self, session: &str) {
        self.db
            .query("UPDATE type::record('review_session', $k) SET writing_closes_at = time::now() - 1s")
            .bind(("k", record_key(session, "review_session")))
            .await
            .unwrap()
            .check()
            .unwrap();
    }

    fn everyone(&self) -> Vec<(&String, &'static str)> {
        [&self.lead, &self.ann, &self.bob, &self.cat, &self.coa]
            .into_iter()
            .map(|u| (u, self.name(u)))
            .collect()
    }
}

fn locked(e: DbError) -> bool {
    matches!(e, DbError::Invalid(ref m) if m == LOCKED)
}

#[tokio::test]
async fn opening_is_idempotent_and_checks_the_game() {
    let w = world().await;
    let a = open_session(&w.db, &w.team, &w.ann, &w.match_id, None, None)
        .await
        .unwrap();
    assert!(a.created);
    let b = open_session(&w.db, &w.team, &w.bob, &w.match_id, Some(1), None)
        .await
        .unwrap();
    assert!(!b.created);
    assert_eq!(a.session_id, b.session_id);
    // The opener facilitates; the second caller did not take over.
    let s = w.state(&w.ann, &a.session_id).await;
    assert!(s.i_am_facilitator);
    assert_eq!(s.facilitator_name, "ann");
    assert_eq!(s.game_number, 1);
    // A Bo1 has no game 2; another team's match is not found.
    assert!(matches!(
        open_session(&w.db, &w.team, &w.ann, &w.match_id, Some(2), None).await,
        Err(DbError::Invalid(_))
    ));
    let foreign = add_match(&w.db, &w.other_team, &w.otto, "bo3", 1).await;
    assert!(matches!(
        open_session(&w.db, &w.team, &w.ann, &foreign, Some(1), None).await,
        Err(DbError::NotFound)
    ));
    // Writing closes 24 h after the start (an hour ago).
    let closes = chrono::DateTime::parse_from_rfc3339(&s.writing_closes_at).unwrap();
    let left = closes.with_timezone(&chrono::Utc) - chrono::Utc::now();
    assert!(left > chrono::Duration::hours(22) && left < chrono::Duration::hours(24));
    // A match that ended long ago gets its own 24 h from now.
    let old = add_match(&w.db, &w.team, &w.lead, "bo3", 72).await;
    let o = open_session(&w.db, &w.team, &w.ann, &old, Some(2), None)
        .await
        .unwrap();
    let s = w.state(&w.ann, &o.session_id).await;
    let closes = chrono::DateTime::parse_from_rfc3339(&s.writing_closes_at).unwrap();
    assert!(closes.with_timezone(&chrono::Utc) - chrono::Utc::now() > chrono::Duration::hours(23));
    assert_eq!(s.phase, Phase::Writing);
}

#[tokio::test]
async fn before_the_reveal_each_member_sees_only_their_own_cards() {
    let w = world().await;
    let s = w.open(&w.lead).await;
    w.write(&w.ann, &s, Prompt::Decided, "ANN-decided-text")
        .await;
    w.write(&w.ann, &s, Prompt::OffPlan, "ANN-offplan-secret")
        .await;
    w.write(&w.bob, &s, Prompt::ToPlan, "BOB-toplan-text").await;
    pass_prompt(&w.db, &w.team, &w.cat, &s, Prompt::NextTime, true)
        .await
        .unwrap();
    let texts = ["ANN-decided-text", "ANN-offplan-secret", "BOB-toplan-text"];
    for (id, _) in w.everyone() {
        let st = w.state(id, &s).await;
        assert_eq!(st.phase, Phase::Writing);
        assert!(st.cards.is_none() && st.proposals.is_empty());
        // ann, bob and cat (a pass) have written; across all prompts.
        assert_eq!(st.written_count, 3);
        assert_eq!(st.member_count, 5);
        let json = serde_json::to_string(&st).unwrap();
        for t in texts {
            let own = st.mine.iter().any(|e| e.text == t);
            assert_eq!(json.contains(t), own, "{t} in {}'s state", w.name(id));
        }
        // No other member's id appears anywhere in the payload.
        for (other, _) in w.everyone() {
            if other != id {
                assert!(
                    !json.contains(other.as_str()),
                    "{other} in {}'s state",
                    w.name(id)
                );
            }
        }
    }
    // The facilitator (lead) sees no text at all.
    let lead = w.state(&w.lead, &s).await;
    assert!(lead.mine.is_empty());
    let cat = w.state(&w.cat, &s).await;
    assert_eq!(cat.mine.len(), 1);
    assert!(cat.mine[0].passed && cat.mine[0].text.is_empty());
}

#[tokio::test]
async fn caps_passes_and_own_cards_only() {
    let w = world().await;
    let s = w.open(&w.lead).await;
    for i in 0..3 {
        w.write(&w.ann, &s, Prompt::Decided, &format!("card {i}"))
            .await;
    }
    assert!(matches!(
        upsert_entry(&w.db, &w.team, &w.ann, &s, Prompt::Decided, None, "fourth").await,
        Err(DbError::Invalid(_))
    ));
    // Empty or too long text is refused.
    assert!(
        upsert_entry(&w.db, &w.team, &w.ann, &s, Prompt::ToPlan, None, "  ")
            .await
            .is_err()
    );
    assert!(upsert_entry(
        &w.db,
        &w.team,
        &w.ann,
        &s,
        Prompt::ToPlan,
        None,
        &"x".repeat(281)
    )
    .await
    .is_err());
    // A prompt with cards cannot be passed; passing then writing withdraws the pass.
    assert!(
        pass_prompt(&w.db, &w.team, &w.ann, &s, Prompt::Decided, true)
            .await
            .is_err()
    );
    pass_prompt(&w.db, &w.team, &w.ann, &s, Prompt::NextTime, true)
        .await
        .unwrap();
    w.write(&w.ann, &s, Prompt::NextTime, "after all").await;
    let st = w.state(&w.ann, &s).await;
    assert!(!st.mine.iter().any(|e| e.passed));
    assert_eq!(st.mine.len(), 4);
    // Bob cannot edit or delete Ann's card.
    let anns = st.mine[0].id.clone();
    assert!(matches!(
        upsert_entry(
            &w.db,
            &w.team,
            &w.bob,
            &s,
            Prompt::Decided,
            Some(anns.clone()),
            "mine now"
        )
        .await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        delete_entry(&w.db, &w.team, &w.bob, &s, &anns).await,
        Err(DbError::NotFound)
    ));
    // Ann can.
    upsert_entry(
        &w.db,
        &w.team,
        &w.ann,
        &s,
        Prompt::Decided,
        Some(anns.clone()),
        "edited",
    )
    .await
    .unwrap();
    delete_entry(&w.db, &w.team, &w.ann, &s, &anns)
        .await
        .unwrap();
}

#[tokio::test]
async fn the_reveal_happens_at_read_time_and_locks_the_cards() {
    let w = world().await;
    let s = w.open(&w.lead).await;
    let e = w.write(&w.ann, &s, Prompt::OffPlan, "lost vision").await;
    w.write(&w.bob, &s, Prompt::OffPlan, "late to drake").await;
    w.write(&w.cat, &s, Prompt::ToPlan, "lane went fine").await;
    let before = w.state(&w.ann, &s).await.rev;
    w.expire(&s).await;
    // Any write after the deadline is refused (and performs the reveal).
    assert!(locked(
        upsert_entry(
            &w.db,
            &w.team,
            &w.ann,
            &s,
            Prompt::OffPlan,
            Some(e.clone()),
            "edit"
        )
        .await
        .unwrap_err()
    ));
    assert!(locked(
        delete_entry(&w.db, &w.team, &w.ann, &s, &e)
            .await
            .unwrap_err()
    ));
    assert!(locked(
        pass_prompt(&w.db, &w.team, &w.dan(), &s, Prompt::Decided, true)
            .await
            .unwrap_err()
    ));
    let st = w.state(&w.ann, &s).await;
    assert_eq!(st.phase, Phase::Revealed);
    assert!(st.rev > before);
    assert!(st.mine.is_empty(), "no own list after the reveal");
    let cards = st.cards.unwrap();
    assert_eq!(cards.off_plan.len(), 2);
    assert_eq!(cards.to_plan.len(), 1);
    assert_eq!(cards.to_plan[0].author_name, "cat");
    // Every entry got a random order key, and the reveal is idempotent.
    let keys: Vec<Option<i64>> = {
        let mut r =
            w.db.query("SELECT VALUE order_key FROM review_entry")
                .await
                .unwrap();
        r.take(0).unwrap()
    };
    assert!(keys.iter().all(|k| k.is_some()), "{keys:?}");
    let again = w.state(&w.bob, &s).await;
    assert_eq!(again.rev, st.rev);
    let keys2: Vec<Option<i64>> = {
        let mut r =
            w.db.query("SELECT VALUE order_key FROM review_entry")
                .await
                .unwrap();
        r.take(0).unwrap()
    };
    let (mut a, mut b) = (keys.clone(), keys2);
    a.sort();
    b.sort();
    assert_eq!(a, b);
    // An unchanged poll answers Unchanged with presence.
    match get_session_state(&w.db, &w.team, &w.bob, "bob", false, &s, Some(st.rev))
        .await
        .unwrap()
    {
        SessionPoll::Unchanged { rev, present } => {
            assert_eq!(rev, st.rev);
            assert!(present.contains(&"ann".to_string()) && present.contains(&"bob".to_string()));
        }
        other => panic!("expected Unchanged, got {other:?}"),
    }
}

impl World {
    /// A member with nothing written (the coach).
    fn dan(&self) -> String {
        self.coa.clone()
    }
}

#[tokio::test]
async fn anonymous_cards_carry_no_author_for_anyone_the_author_included() {
    let w = world().await;
    let s = w.open(&w.lead).await;
    let anon = w
        .write(&w.ann, &s, Prompt::OffPlan, "we panicked at baron")
        .await;
    w.write(&w.bob, &s, Prompt::Decided, "the bot dive").await;
    reveal_now(&w.db, &w.team, &w.lead, &s).await.unwrap();
    // A proposal from the anonymous card.
    create_proposal(&w.db, &w.team, &s, Some(anon.clone()), "Call baron earlier")
        .await
        .unwrap();
    plus_one(&w.db, &w.team, &w.ann, &s, &anon).await.unwrap();
    for (id, _) in w.everyone() {
        let st = w.state(id, &s).await;
        let cards = st.cards.clone().unwrap();
        let off = serde_json::to_string(&cards.off_plan).unwrap();
        let props = serde_json::to_string(&st.proposals).unwrap();
        for (member, name) in w.everyone() {
            assert!(!off.contains(member.as_str()) && !props.contains(member.as_str()));
            assert!(
                !off.contains(&format!("\"{name}\"")) && !props.contains(&format!("\"{name}\""))
            );
        }
        for field in ["author", "mine", "created", "updated", "order_key", "user"] {
            assert!(!off.contains(field), "{field} in off_plan: {off}");
            assert!(!props.contains(field), "{field} in proposals: {props}");
        }
        // The attributed card keeps its author.
        assert_eq!(cards.decided[0].author_name, "bob");
        // +1 is per viewer only.
        assert_eq!(cards.off_plan[0].plus_ones, 1);
        assert_eq!(cards.off_plan[0].i_plus_oned, id == &w.ann);
    }
}

#[tokio::test]
async fn votes_accepts_owners_and_close_write_a_review_that_resaves_clean() {
    let w = world().await;
    let s = w.open(&w.ann).await;
    let e1 = w
        .write(&w.ann, &s, Prompt::NextTime, "ward the river")
        .await;
    w.write(&w.bob, &s, Prompt::OffPlan, "late rotations").await;
    w.write(&w.cat, &s, Prompt::ToPlan, "early dragon").await;
    pass_prompt(&w.db, &w.team, &w.lead, &s, Prompt::Decided, true)
        .await
        .unwrap();
    // Only the facilitator reveals.
    assert!(reveal_now(&w.db, &w.team, &w.bob, &s).await.is_err());
    reveal_now(&w.db, &w.team, &w.ann, &s).await.unwrap();
    // +1 toggles, own card included.
    assert!(plus_one(&w.db, &w.team, &w.ann, &s, &e1).await.unwrap());
    assert!(!plus_one(&w.db, &w.team, &w.ann, &s, &e1).await.unwrap());
    let mut props = Vec::new();
    for t in ["A", "B", "C", "D"] {
        props.push(
            create_proposal(&w.db, &w.team, &s, Some(e1.clone()), &format!("item {t}"))
                .await
                .unwrap(),
        );
    }
    // Three decide votes per member.
    for p in &props[..3] {
        assert!(decide_vote(&w.db, &w.team, &w.bob, &s, p).await.unwrap());
    }
    assert!(matches!(
        decide_vote(&w.db, &w.team, &w.bob, &s, &props[3]).await,
        Err(DbError::Invalid(_))
    ));
    assert_eq!(w.state(&w.bob, &s).await.decide_votes_left, 0);
    // Taking one back frees it.
    assert!(!decide_vote(&w.db, &w.team, &w.bob, &s, &props[2])
        .await
        .unwrap());
    assert!(decide_vote(&w.db, &w.team, &w.bob, &s, &props[3])
        .await
        .unwrap());
    // Three accepted at most, facilitator only.
    assert!(set_accepted(&w.db, &w.team, &w.bob, &s, &props[0], true)
        .await
        .is_err());
    for p in &props[..3] {
        set_accepted(&w.db, &w.team, &w.ann, &s, p, true)
            .await
            .unwrap();
    }
    assert!(matches!(
        set_accepted(&w.db, &w.team, &w.ann, &s, &props[3], true).await,
        Err(DbError::Invalid(_))
    ));
    // Owners: current members only; blank is nobody.
    assert!(matches!(
        set_owner(&w.db, &w.team, &w.ann, &s, &props[0], Some("otto".into())).await,
        Err(DbError::Invalid(_))
    ));
    set_owner(&w.db, &w.team, &w.ann, &s, &props[0], Some("BOB".into()))
        .await
        .unwrap();
    set_owner(&w.db, &w.team, &w.ann, &s, &props[1], Some("cat".into()))
        .await
        .unwrap();
    set_owner(&w.db, &w.team, &w.ann, &s, &props[2], Some(" ".into()))
        .await
        .unwrap();
    let st = w.state(&w.ann, &s).await;
    assert_eq!(st.proposals.iter().filter(|p| p.accepted).count(), 3);
    assert!(st
        .proposals
        .iter()
        .any(|p| p.owner.as_deref() == Some("bob")));

    let review_id = close_session(&w.db, &w.team, &w.ann, &s).await.unwrap();
    let review = db::get_post_game_learning(&w.db, &w.team, &review_id)
        .await
        .unwrap()
        .unwrap();
    assert!(review.what_went_well.is_empty() && review.improvements.is_empty());
    assert_eq!(review.action_items.len(), 3);
    assert_eq!(review.created_by, w.ann);
    assert_eq!(
        review.scheduled_match_id.as_deref(),
        Some(w.match_id.as_str())
    );
    assert_eq!(review.series_id, None);
    assert_eq!(review.game_number, None, "a Bo1 review has no game number");
    let items: Vec<_> = db::list_action_items(&w.db, &w.team)
        .await
        .unwrap()
        .into_iter()
        .filter(|i| i.source_review.as_deref() == Some(review_id.as_str()))
        .collect();
    assert_eq!(items.len(), 3);
    assert!(items
        .iter()
        .any(|i| i.assigned_to.as_deref() == Some("bob")));
    assert!(items.iter().any(|i| i.assigned_to.is_none()));
    // Saving the review again in the editor creates no item.
    let created = db::update_review_with_action_items(&w.db, &w.team, review.clone())
        .await
        .unwrap();
    assert_eq!(created, 0);
    assert_eq!(
        db::list_action_items(&w.db, &w.team).await.unwrap().len(),
        3
    );
    // Closed: voices (lead only passed), the review link, nothing writable.
    let st = w.state(&w.bob, &s).await;
    assert_eq!(st.phase, Phase::Closed);
    assert_eq!(st.review_id.as_deref(), Some(review_id.as_str()));
    assert_eq!(st.voices.unwrap().label(), "3 of 5 voices");
    assert!(plus_one(&w.db, &w.team, &w.bob, &s, &e1).await.is_err());
    assert!(close_session(&w.db, &w.team, &w.ann, &s).await.is_err());
    // The review page's view.
    let view = session_for_review(&w.db, &w.team, &w.cat, &review_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(view.accepted.len(), 3);
    assert_eq!(view.cards.off_plan.len(), 1);
    assert_eq!(view.voices.unwrap().label(), "3 of 5 voices");
    // Patterns read the closed session's texts.
    let texts = session_pattern_texts(&w.db, &w.team).await.unwrap();
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].to_plan, vec!["early dragon"]);
    assert_eq!(texts[0].off_plan, vec!["late rotations"]);
    assert_eq!(texts[0].review_id.as_deref(), Some(review_id.as_str()));
    // Deleting the review clears the session's link.
    db::delete_post_game_learning(&w.db, &w.team, &review_id)
        .await
        .unwrap();
    assert_eq!(w.state(&w.bob, &s).await.review_id, None);
}

#[tokio::test]
async fn a_series_game_gets_its_number_and_the_series_on_the_review() {
    let w = world().await;
    let m = add_match(&w.db, &w.team, &w.lead, "bo3", 2).await;
    let series = {
        let mut r = w
            .db
            .query("CREATE series SET name = 'Bo3 vs Rooks', team = type::record('team', $t), format = 'bo3', created_by = type::record('user', $u) RETURN id")
            .bind(("t", record_key(&w.team, "team")))
            .bind(("u", record_key(&w.lead, "user")))
            .await
            .unwrap();
        let row: Option<IdRow> = r.take(0).unwrap();
        row.unwrap().id.to_sql()
    };
    w.db.query(
        "UPDATE type::record('scheduled_match', $m) SET series = type::record('series', $s)",
    )
    .bind(("m", record_key(&m, "scheduled_match")))
    .bind(("s", record_key(&series, "series")))
    .await
    .unwrap()
    .check()
    .unwrap();
    let s = open_session(&w.db, &w.team, &w.ann, &m, Some(2), Some("EUW1_123".into()))
        .await
        .unwrap()
        .session_id;
    w.write(&w.bob, &s, Prompt::Decided, "level 1 invade").await;
    reveal_now(&w.db, &w.team, &w.ann, &s).await.unwrap();
    let review_id = close_session(&w.db, &w.team, &w.ann, &s).await.unwrap();
    let review = db::get_post_game_learning(&w.db, &w.team, &review_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(review.game_number, Some(2));
    assert_eq!(review.match_riot_id.as_deref(), Some("EUW1_123"));
    assert_eq!(review.series_id.as_deref(), Some(series.as_str()));
    assert!(review.action_items.is_empty());
}

#[tokio::test]
async fn other_teams_cannot_reach_a_session() {
    let w = world().await;
    let s = w.open(&w.lead).await;
    let e = w.write(&w.ann, &s, Prompt::Decided, "x marks").await;
    let t = &w.other_team;
    let o = &w.otto;
    assert!(matches!(
        get_session_state(&w.db, t, o, "otto", true, &s, None).await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        upsert_entry(&w.db, t, o, &s, Prompt::Decided, None, "hi").await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        upsert_entry(&w.db, t, o, &s, Prompt::Decided, Some(e.clone()), "hi").await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        reveal_now(&w.db, t, o, &s).await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        close_session(&w.db, t, o, &s).await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        sessions_of_match(&w.db, t, &w.match_id).await,
        Err(DbError::NotFound)
    ));
    // A card id of another session is not found in this one.
    let m2 = add_match(&w.db, &w.team, &w.lead, "bo1", 3).await;
    let s2 = open_session(&w.db, &w.team, &w.ann, &m2, None, None)
        .await
        .unwrap()
        .session_id;
    reveal_now(&w.db, &w.team, &w.ann, &s2).await.unwrap();
    assert!(matches!(
        plus_one(&w.db, &w.team, &w.ann, &s2, &e).await,
        Err(DbError::NotFound)
    ));
}

#[tokio::test]
async fn facilitation_timed_round_and_take_over() {
    let w = world().await;
    let s = w.open(&w.ann).await;
    let before = w.state(&w.ann, &s).await.writing_closes_at;
    assert!(start_timed_round(&w.db, &w.team, &w.bob, &s, None)
        .await
        .is_err());
    start_timed_round(&w.db, &w.team, &w.ann, &s, None)
        .await
        .unwrap();
    let st = w.state(&w.ann, &s).await;
    assert!(st.timed_round);
    let at = |x: &str| chrono::DateTime::parse_from_rfc3339(x).unwrap();
    assert!(at(&st.writing_closes_at) < at(&before));
    let left = at(&st.writing_closes_at).with_timezone(&chrono::Utc) - chrono::Utc::now();
    assert!(left <= chrono::Duration::minutes(3));
    add_minute(&w.db, &w.team, &w.ann, &s).await.unwrap();
    let later = w.state(&w.ann, &s).await.writing_closes_at;
    assert_eq!(
        at(&later) - at(&st.writing_closes_at),
        chrono::Duration::minutes(1)
    );
    // A plain member cannot take over; the coach can.
    assert!(take_over(&w.db, &w.team, &w.bob, false, &s).await.is_err());
    take_over(&w.db, &w.team, &w.coa, true, &s).await.unwrap();
    let st = w.state(&w.coa, &s).await;
    assert!(st.i_am_facilitator);
    assert_eq!(st.facilitator_name, "coa");
    let ann = match get_session_state(&w.db, &w.team, &w.ann, "ann", false, &s, None)
        .await
        .unwrap()
    {
        SessionPoll::Full(s) => s,
        _ => unreachable!(),
    };
    assert!(!ann.i_am_facilitator && !ann.can_take_over);
}

#[tokio::test]
async fn merge_keeps_every_card_and_unmerge_restores() {
    let w = world().await;
    let s = w.open(&w.lead).await;
    let a = w
        .write(&w.ann, &s, Prompt::OffPlan, "lost dragon control")
        .await;
    let b = w
        .write(&w.bob, &s, Prompt::OffPlan, "dragon control lost")
        .await;
    let c = w.write(&w.cat, &s, Prompt::ToPlan, "dragon control").await;
    reveal_now(&w.db, &w.team, &w.lead, &s).await.unwrap();
    let st = w.state(&w.lead, &s).await;
    let sug = st.cards.unwrap().suggestions;
    assert_eq!(sug.len(), 1);
    assert_eq!(sug[0].prompt, Prompt::OffPlan);
    // Members get no suggestions; across prompts nothing merges.
    assert!(w
        .state(&w.ann, &s)
        .await
        .cards
        .unwrap()
        .suggestions
        .is_empty());
    assert!(
        merge_entries(&w.db, &w.team, &w.lead, &s, &a, std::slice::from_ref(&c))
            .await
            .is_err()
    );
    assert!(
        merge_entries(&w.db, &w.team, &w.ann, &s, &a, std::slice::from_ref(&b))
            .await
            .is_err()
    );
    merge_entries(&w.db, &w.team, &w.lead, &s, &a, std::slice::from_ref(&b))
        .await
        .unwrap();
    let off = w.state(&w.bob, &s).await.cards.unwrap().off_plan;
    assert_eq!(off.len(), 2, "both cards stay");
    let root = off.iter().find(|x| x.id == a).unwrap();
    let child = off.iter().find(|x| x.id == b).unwrap();
    assert_eq!(root.cluster, None);
    assert_eq!(child.cluster.as_deref(), Some(a.as_str()));
    unmerge(&w.db, &w.team, &w.lead, &s, &a).await.unwrap();
    let off = w.state(&w.bob, &s).await.cards.unwrap().off_plan;
    assert!(off.iter().all(|x| x.cluster.is_none()));
}

#[tokio::test]
async fn the_quiet_nudge_needs_three_closed_sessions_and_is_a_count() {
    let w = world().await;
    let mut last = String::new();
    for i in 0..3 {
        // Everyone writes except cat (who passes) and coa.
        let m = add_match(&w.db, &w.team, &w.lead, "bo1", 3 + i).await;
        let s = open_session(&w.db, &w.team, &w.lead, &m, None, None)
            .await
            .unwrap()
            .session_id;
        for who in [&w.lead, &w.ann, &w.bob] {
            w.write(who, &s, Prompt::Decided, "spoke").await;
        }
        pass_prompt(&w.db, &w.team, &w.cat, &s, Prompt::Decided, true)
            .await
            .unwrap();
        if i < 2 {
            assert_eq!(quiet_nudge(&w.db, &w.team).await.unwrap(), None);
        }
        reveal_now(&w.db, &w.team, &w.lead, &s).await.unwrap();
        close_session(&w.db, &w.team, &w.lead, &s).await.unwrap();
        last = s;
    }
    let _ = last;
    // cat (passes only) and coa (nothing) were quiet in all three.
    assert_eq!(quiet_nudge(&w.db, &w.team).await.unwrap(), Some(2));
    // Shown to the facilitator of a writing session only, as a count.
    let s = w.open(&w.lead).await;
    let st = w.state(&w.lead, &s).await;
    assert_eq!(st.quiet, Some(2));
    assert_eq!(w.state(&w.ann, &s).await.quiet, None);
    let json = serde_json::to_string(&st).unwrap();
    assert!(json.contains("\"quiet\":2"));
    assert!(!json.contains(w.cat.as_str()) && !json.contains("\"cat\""));
}

#[tokio::test]
async fn deleting_the_match_keeps_sessions_without_a_link() {
    let w = world().await;
    // Two matches, each with a closed game-1 session: after both matches are
    // deleted both sessions have (NONE, 1), which the unique index allows.
    for i in 0..2 {
        let m = add_match(&w.db, &w.team, &w.lead, "bo1", 2 + i).await;
        let s = open_session(&w.db, &w.team, &w.ann, &m, None, None)
            .await
            .unwrap()
            .session_id;
        w.write(&w.bob, &s, Prompt::Decided, "a call").await;
        db::delete_scheduled_match(&w.db, &w.team, &m)
            .await
            .unwrap();
        let st = w.state(&w.ann, &s).await;
        assert_eq!(st.match_id, None);
        assert!(st.match_deleted);
        // The session still works: reveal and close without a match stamp.
        reveal_now(&w.db, &w.team, &w.ann, &s).await.unwrap();
        let r = close_session(&w.db, &w.team, &w.ann, &s).await.unwrap();
        let review = db::get_post_game_learning(&w.db, &w.team, &r)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(review.scheduled_match_id, None);
    }
}

#[tokio::test]
async fn owner_check_takes_current_members_only() {
    let w = world().await;
    assert_eq!(
        db::check_owner(&w.db, &w.team, Some("Ann".into()))
            .await
            .unwrap(),
        Some("ann".into())
    );
    assert_eq!(
        db::check_owner(&w.db, &w.team, Some("".into()))
            .await
            .unwrap(),
        None
    );
    assert_eq!(db::check_owner(&w.db, &w.team, None).await.unwrap(), None);
    assert!(matches!(
        db::check_owner(&w.db, &w.team, Some("otto".into())).await,
        Err(DbError::Invalid(_))
    ));
    db::remove_team_member(&w.db, &w.team, &w.cat)
        .await
        .unwrap();
    assert!(db::check_owner(&w.db, &w.team, Some("cat".into()))
        .await
        .is_err());
}

#[tokio::test]
async fn account_deletion_export_and_team_deletion_cover_sessions() {
    use crate::server::account;
    let w = world().await;
    let s = w.open(&w.ann).await;
    let anon = w.write(&w.ann, &s, Prompt::OffPlan, "ANN-anon-card").await;
    w.write(&w.ann, &s, Prompt::Decided, "ANN-named-card").await;
    pass_prompt(&w.db, &w.team, &w.ann, &s, Prompt::NextTime, true)
        .await
        .unwrap();
    w.write(&w.bob, &s, Prompt::Decided, "BOB-card").await;
    reveal_now(&w.db, &w.team, &w.ann, &s).await.unwrap();
    plus_one(&w.db, &w.team, &w.ann, &s, &anon).await.unwrap();
    let p = create_proposal(&w.db, &w.team, &s, Some(anon.clone()), "fix it")
        .await
        .unwrap();
    decide_vote(&w.db, &w.team, &w.ann, &s, &p).await.unwrap();

    // The export holds her cards (the anonymous one included) and votes.
    let doc = account::export_user_data(&w.db, &w.ann).await.unwrap();
    let cards = &doc["data"]["review_cards_written"];
    let text = cards.to_string();
    assert!(text.contains("ANN-anon-card") && text.contains("ANN-named-card"));
    assert!(!text.contains("BOB-card"));
    assert_eq!(cards.as_array().unwrap().len(), 3);
    assert!(cards
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["anonymous"] == true && c["text"] == "ANN-anon-card"));
    assert_eq!(doc["data"]["review_votes"].as_array().unwrap().len(), 2);

    // Deletion: cards stay under the tombstone, votes go, facilitation too.
    account::delete_account(&w.db, &w.ann).await.unwrap();
    let st = w.state(&w.bob, &s).await;
    let cards = st.cards.unwrap();
    assert_eq!(
        cards
            .decided
            .iter()
            .filter(|c| c.author_name == "Former member")
            .count(),
        1
    );
    assert_eq!(cards.off_plan[0].plus_ones, 0);
    assert_eq!(st.facilitator_name, "Former member");
    assert_eq!(st.proposals[0].votes, 0);

    // Deleting the team removes every session row.
    db::remove_team_member(&w.db, &w.team, &w.bob)
        .await
        .unwrap();
    db::remove_team_member(&w.db, &w.team, &w.cat)
        .await
        .unwrap();
    db::remove_team_member(&w.db, &w.team, &w.coa)
        .await
        .unwrap();
    account::delete_account(&w.db, &w.lead).await.unwrap();
    for table in [
        "review_session",
        "review_entry",
        "review_vote",
        "review_proposal",
    ] {
        let mut r =
            w.db.query(format!("SELECT count() AS n FROM {table} GROUP ALL"))
                .await
                .unwrap();
        #[derive(Debug, Deserialize, SurrealValue)]
        struct N {
            n: i64,
        }
        let n: Option<N> = r.take(0).unwrap();
        assert_eq!(n.map(|n| n.n).unwrap_or(0), 0, "{table} rows left");
    }
}

#[test]
fn presence_expires_after_ten_seconds_and_is_purged() {
    let t0 = Instant::now();
    touch_presence_at("review_session:presence_test", "user:a", "ann", t0);
    touch_presence_at(
        "review_session:presence_test",
        "user:b",
        "Bob",
        t0 + Duration::from_secs(8),
    );
    assert_eq!(
        present_names_at("review_session:presence_test", t0 + Duration::from_secs(9)),
        vec!["ann", "Bob"]
    );
    assert_eq!(
        present_names_at("review_session:presence_test", t0 + Duration::from_secs(11)),
        vec!["Bob"]
    );
    assert!(purge_presence_at(t0 + Duration::from_secs(11)) >= 1);
    assert!(purge_presence_at(t0 + Duration::from_secs(30)) >= 1);
    assert!(
        present_names_at("review_session:presence_test", t0 + Duration::from_secs(30)).is_empty()
    );
}

/// A fixture state, serialized: the off_plan group holds no member id,
/// username or avatar path, for the author and the facilitator too.
#[test]
fn a_revealed_state_serializes_the_off_plan_group_without_authors() {
    let session = Session {
        id: "review_session:s1".into(),
        team_id: "team:t1".into(),
        match_id: Some("scheduled_match:m1".into()),
        match_format: Some("bo1".into()),
        match_deleted: false,
        series_id: None,
        game_number: 1,
        match_riot_id: None,
        facilitator_id: "user:fac".into(),
        facilitator_name: "Facil".into(),
        stored_phase: Phase::Revealed,
        writing_closes_at: "2026-11-13T18:00:00Z".into(),
        async_closes_at: "2026-11-13T18:00:00Z".into(),
        timed_round: false,
        review_id: None,
        voices: None,
        rev: 7,
    };
    let entry = |id: &str, author: &str, name: &str, prompt: Prompt, text: &str, key: i64| Entry {
        id: id.into(),
        author_id: author.into(),
        author_name: name.into(),
        prompt,
        text: Some(text.into()),
        passed: false,
        cluster: None,
        order_key: Some(key),
    };
    let contents = Contents {
        entries: vec![
            entry(
                "review_entry:e1",
                "user:auth",
                "Authorname",
                Prompt::OffPlan,
                "missed the wave",
                9,
            ),
            entry(
                "review_entry:e2",
                "user:fac",
                "Facil",
                Prompt::OffPlan,
                "bad call",
                2,
            ),
            entry(
                "review_entry:e3",
                "user:auth",
                "Authorname",
                Prompt::Decided,
                "level one",
                1,
            ),
        ],
        votes: vec![Vote {
            target: "review_entry:e1".into(),
            user_id: "user:auth".into(),
            decide: false,
        }],
        proposals: vec![Proposal {
            id: "review_proposal:p1".into(),
            text: "farm waves".into(),
            source: Some("review_entry:e1".into()),
            accepted: false,
            owner: None,
        }],
        members: vec!["user:auth".into(), "user:fac".into()],
    };
    for viewer in ["user:auth", "user:fac"] {
        let st = assemble_state(
            &session,
            &contents,
            viewer,
            false,
            "2026-11-13T19:00:00Z",
            Some(1),
            vec![],
        );
        assert!(st.mine.is_empty());
        assert_eq!(st.quiet, None, "no nudge after writing");
        let cards = st.cards.as_ref().unwrap();
        // order_key order, not insertion order.
        assert_eq!(cards.off_plan[0].id, "review_entry:e2");
        let off = serde_json::to_string(&cards.off_plan).unwrap();
        let props = serde_json::to_string(&st.proposals).unwrap();
        for needle in [
            "user:",
            "auth",
            "fac",
            "Authorname",
            "Facil",
            "/img/",
            "avatar",
            "author",
        ] {
            assert!(!off.contains(needle), "{needle} in {off}");
            assert!(!props.contains(needle), "{needle} in {props}");
        }
        assert_eq!(cards.decided[0].author_name, "Authorname");
    }
    // While writing the same contents give the viewer only their own cards.
    let mut writing = session.clone();
    writing.stored_phase = Phase::Writing;
    writing.writing_closes_at = "2026-11-13T20:00:00Z".into();
    let st = assemble_state(
        &writing,
        &contents,
        "user:fac",
        false,
        "2026-11-13T19:00:00Z",
        Some(1),
        vec![],
    );
    assert_eq!(st.phase, Phase::Writing);
    assert_eq!(st.mine.len(), 1);
    assert_eq!(st.quiet, Some(1));
    assert_eq!(st.written_count, 2);
    let json = serde_json::to_string(&st).unwrap();
    assert!(!json.contains("missed the wave") && !json.contains("level one"));
    assert!(!json.contains("Authorname"));
}
