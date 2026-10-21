//! M9 guidance: rows are per user (another user's seen or muted state never
//! leaks or applies), writes are idempotent, the switch and the resets touch
//! only the user's own rows, and the checklist, hints and nudges read the
//! user's own team.

use crate::common;

use chrono::{Duration, Utc};
use lol_team_companion::models::guidance::{
    self, pick_nudge, GuidanceRow, Hint, NudgeKind, CHECKLIST, WELCOME,
};
use lol_team_companion::server::db;
use lol_team_companion::server::guidance::{self as g, Mark, Reset};
use surrealdb::{engine::local::Db, Surreal};

async fn user(db: &Surreal<Db>, name: &str) -> String {
    db::create_user(db, name.into(), format!("{name}@example.com"), "h".into())
        .await
        .unwrap()
}

async fn sql(db: &Surreal<Db>, q: &str, u: &str, t: &str) {
    db.query(format!(
        "LET $u = type::record('user', $uk); LET $t = type::record('team', $tk); {q}"
    ))
    .bind(("uk", db::record_key(u, "user")))
    .bind(("tk", db::record_key(t, "team")))
    .await
    .unwrap()
    .check()
    .unwrap();
}

fn row<'a>(rows: &'a [GuidanceRow], key: &str) -> Option<&'a GuidanceRow> {
    rows.iter().find(|r| r.key == key)
}

#[tokio::test]
async fn rows_belong_to_one_user_and_writes_are_idempotent() {
    let db = common::test_db().await;
    let a = user(&db, "ann").await;
    let b = user(&db, "ben").await;

    g::mark(&db, &a, "hint.draft_keys", Mark::Seen)
        .await
        .unwrap();
    g::mark(&db, &a, "hint.draft_keys", Mark::Seen)
        .await
        .unwrap();
    g::mark(&db, &a, "hint.draft_keys", Mark::Muted)
        .await
        .unwrap();
    g::mark(&db, &a, WELCOME, Mark::Muted).await.unwrap();

    let ra = g::rows(&db, &a).await.unwrap();
    assert_eq!(ra.len(), 2, "{ra:?}");
    let r = row(&ra, "hint.draft_keys").unwrap();
    assert!(r.seen && r.muted);
    assert!(
        g::rows(&db, &b).await.unwrap().is_empty(),
        "nothing of ann's applies to ben"
    );

    // The unique index holds even for a hand-written duplicate.
    let dup = db
        .query("CREATE user_guidance SET user = type::record('user', $uk), key = 'welcome'")
        .bind(("uk", db::record_key(&a, "user")))
        .await
        .unwrap()
        .check();
    assert!(dup.is_err(), "a second row for the same user and key");

    // Ben's state still has every hint; ann's lost the dismissed one.
    let now = Utc::now();
    let sa = g::state(&db, &a, now).await.unwrap();
    let sb = g::state(&db, &b, now).await.unwrap();
    assert!(!sa.hints.contains(&Hint::DraftKeys));
    assert!(sb.hints.contains(&Hint::DraftKeys));
}

#[tokio::test]
async fn the_switch_and_the_resets_touch_only_the_users_own_rows() {
    let db = common::test_db().await;
    let a = user(&db, "ann").await;
    let b = user(&db, "ben").await;
    for u in [&a, &b] {
        g::mark(&db, u, "hint.pool_tiers", Mark::Muted)
            .await
            .unwrap();
        g::mark(&db, u, CHECKLIST, Mark::Muted).await.unwrap();
    }
    assert!(g::enabled(&db, &a).await.unwrap(), "on by default");

    g::reset(&db, &a, Reset::Hints).await.unwrap();
    let ra = g::rows(&db, &a).await.unwrap();
    assert!(row(&ra, "hint.pool_tiers").is_none());
    assert!(row(&ra, CHECKLIST).is_some(), "the checklist stays hidden");
    assert!(row(&g::rows(&db, &b).await.unwrap(), "hint.pool_tiers").is_some());

    g::reset(&db, &a, Reset::Checklist).await.unwrap();
    assert!(g::rows(&db, &a).await.unwrap().is_empty());
    assert_eq!(g::rows(&db, &b).await.unwrap().len(), 2);

    // "Show nudges again": every muted nudge kind comes back, nothing else
    // of ann's changes and nothing of ben's.
    for u in [&a, &b] {
        g::mark(&db, u, "nudge.returning", Mark::Muted)
            .await
            .unwrap();
        g::mark(&db, u, "nudge.old_action_items", Mark::Muted)
            .await
            .unwrap();
    }
    g::mark(&db, &a, "hint.draft_keys", Mark::Muted)
        .await
        .unwrap();
    g::reset(&db, &a, Reset::Nudges).await.unwrap();
    let ra = g::rows(&db, &a).await.unwrap();
    assert!(ra.iter().all(|r| !r.key.starts_with("nudge.")), "{ra:?}");
    assert!(
        row(&ra, "hint.draft_keys").is_some(),
        "hints stay dismissed"
    );
    let rb = g::rows(&db, &b).await.unwrap();
    assert!(row(&rb, "nudge.returning").is_some_and(|r| r.muted));
    assert!(row(&rb, "nudge.old_action_items").is_some_and(|r| r.muted));
    g::reset(&db, &a, Reset::Hints).await.unwrap();

    g::set_enabled(&db, &a, false).await.unwrap();
    assert!(!g::enabled(&db, &a).await.unwrap());
    assert!(g::enabled(&db, &b).await.unwrap());
    let now = Utc::now();
    let off = g::state(&db, &a, now).await.unwrap();
    assert!(!off.enabled && off.hints.is_empty() && off.intros.is_empty());
    let home = g::home_guide(&db, &a, "ann", false, false, None, now)
        .await
        .unwrap();
    assert!(home.welcome.is_none() && home.checklist.is_none() && home.nudge.is_none());
    // "Guide me" still opens the checklist while guidance is off.
    let forced = g::home_guide(&db, &a, "ann", false, true, None, now)
        .await
        .unwrap();
    assert!(forced.checklist.is_some() && forced.forced);
}

#[tokio::test]
async fn the_welcome_shows_once_and_the_checklist_follows_the_users_team() {
    let db = common::test_db().await;
    let a = user(&db, "ann").await;
    let b = user(&db, "ben").await;
    let now = Utc::now();

    let home = g::home_guide(&db, &a, "ann", false, false, None, now)
        .await
        .unwrap();
    let w = home.welcome.expect("a new account is welcomed");
    assert_eq!(w.start_href, "/team/dashboard");
    assert_eq!(w.team_name, None);
    let items = home.checklist.unwrap();
    assert!(items.iter().all(|i| !i.done));

    g::mark(&db, &a, WELCOME, Mark::Seen).await.unwrap();
    let home = g::home_guide(&db, &a, "ann", false, false, None, now)
        .await
        .unwrap();
    assert!(home.welcome.is_none());
    assert!(home.checklist.is_some(), "Start keeps the checklist");

    // Ann builds a team with a draft; ben joins it and finds those steps done.
    let team = db::create_team(&db, &a, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    sql(
        &db,
        "CREATE draft SET name = 'D1', team = $t, created_by = $u;",
        &a,
        &team,
    )
    .await;
    db::join_team(&db, &b, &team).await.unwrap();
    let fb = g::checklist_facts(&db, &b).await.unwrap();
    assert_eq!(fb.team_name.as_deref(), Some("Northgate"));
    assert_eq!(fb.drafts, 1);
    assert_eq!(fb.members, 2);
    let items = guidance::checklist(&fb);
    assert!(items[0].done && items[3].done && items[6].done);

    // An outsider's checklist sees nothing of that team.
    let c = user(&db, "cid").await;
    let fc = g::checklist_facts(&db, &c).await.unwrap();
    assert_eq!(fc.team_name, None);
    assert_eq!(fc.drafts, 0);
}

#[tokio::test]
async fn hints_read_the_users_own_actions() {
    let db = common::test_db().await;
    let a = user(&db, "ann").await;
    let b = user(&db, "ben").await;
    let team = db::create_team(&db, &a, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    db::join_team(&db, &b, &team).await.unwrap();
    sql(&db, "CREATE draft SET name = 'D1', team = $t, created_by = $u; CREATE champion_pool SET user = $u, champion = 'Ahri', role = 'mid', tier = 'match_ready';", &a, &team).await;
    let ua = g::hint_use(&db, &a).await.unwrap();
    let ub = g::hint_use(&db, &b).await.unwrap();
    assert!(ua.drafted && ua.tiered);
    assert!(!ub.drafted && !ub.tiered, "ann's draft is not ben's");

    // Branches and note edits are the team's.
    sql(&db, "LET $tree = (CREATE ONLY draft_tree SET name = 'T', team = $t, created_by = $u); \
              LET $root = (CREATE ONLY draft_tree_node SET tree = $tree.id, label = 'Root'); \
              CREATE draft_tree_node SET tree = $tree.id, parent = $root.id, label = 'If they ban Ahri'; \
              CREATE team_note SET team = $t, author = $u, content = 'x', previous_content = 'y';", &a, &team).await;
    let ub = g::hint_use(&db, &b).await.unwrap();
    assert!(ub.branched && ub.note_changed);
    let c = user(&db, "cid").await;
    let uc = g::hint_use(&db, &c).await.unwrap();
    assert!(!uc.branched && !uc.note_changed && !uc.drafted);
}

#[tokio::test]
async fn nudges_read_the_users_team_and_writes() {
    let db = common::test_db().await;
    let a = user(&db, "ann").await;
    let b = user(&db, "ben").await;
    let team = db::create_team(&db, &a, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    let team_b = db::create_team(&db, &b, "Rooks".into(), "EUW".into())
        .await
        .unwrap();
    let now = Utc::now();

    // A recent draft without a plan, and an old open action item.
    sql(&db, "CREATE draft SET name = 'Rooks bo3', team = $t, created_by = $u; \
              CREATE action_item SET team = $t, text = 'ward river', created_at = time::now() - 10d;", &a, &team).await;
    let fa = g::nudge_facts(&db, &a, &team, now).await.unwrap();
    assert_eq!(fa.old_open_items, 1);
    assert_eq!(
        fa.draft_without_plan.as_ref().map(|d| d.1.as_str()),
        Some("Rooks bo3")
    );
    let n = pick_nudge(true, &fa, &[], None).unwrap();
    assert_eq!(n.kind, NudgeKind::OldActionItems);

    // Ben's team has none of it.
    let fb = g::nudge_facts(&db, &b, &team_b, now).await.unwrap();
    assert_eq!(fb.old_open_items, 0);
    assert!(fb.draft_without_plan.is_none());
    assert!(pick_nudge(true, &fb, &[], None).is_none());

    // Ann has been away: everything she wrote is a month old.
    sql(
        &db,
        "UPDATE draft SET created_at = time::now() - 30d WHERE created_by = $u; \
              UPDATE $u SET created_at = time::now() - 40d;",
        &a,
        &team,
    )
    .await;
    let fa = g::nudge_facts(&db, &a, &team, now).await.unwrap();
    assert!(fa.last_write.is_some_and(|t| now - t >= Duration::days(29)));
    let n = pick_nudge(true, &fa, &[], None).unwrap();
    assert_eq!(n.kind, NudgeKind::Returning);
    // Muted: the next rule applies.
    g::mark(&db, &a, "nudge.returning", Mark::Muted)
        .await
        .unwrap();
    let rows = g::rows(&db, &a).await.unwrap();
    assert_eq!(
        pick_nudge(true, &fa, &rows, None).unwrap().kind,
        NudgeKind::OldActionItems
    );
    // Ben's mute state is his own.
    assert!(g::rows(&db, &b).await.unwrap().is_empty());
}

/// Reading counts for "Picking up again?" (polish pass): a visit is
/// recorded at most once an hour, the previous visit moves to
/// `prev_seen_at`, and a member who was here yesterday is not "returning"
/// however old their last write is.
#[tokio::test]
async fn visits_count_for_the_returning_rule() {
    let db = common::test_db().await;
    let a = user(&db, "ann").await;
    let team = db::create_team(&db, &a, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    // Everything Ann wrote is a month old; she has never been seen.
    sql(
        &db,
        "CREATE draft SET name = 'Old', team = $t, created_by = $u, created_at = time::now() - 30d; \
         UPDATE $u SET created_at = time::now() - 40d;",
        &a,
        &team,
    )
    .await;
    let now = Utc::now();
    assert!(
        db::touch_last_seen(&db, &a).await.unwrap(),
        "first visit is written"
    );
    assert!(
        !db::touch_last_seen(&db, &a).await.unwrap(),
        "not again within the hour"
    );
    // First recorded visit: no earlier one, so the old write decides.
    let f = g::nudge_facts(&db, &a, &team, now).await.unwrap();
    assert!(f.prev_seen.is_none());
    assert_eq!(
        pick_nudge(true, &f, &[], None).unwrap().kind,
        NudgeKind::Returning
    );

    // She was here yesterday (last_seen a day old): the next visit moves it
    // to prev_seen and the line is gone.
    sql(
        &db,
        "UPDATE $u SET last_seen_at = time::now() - 1d;",
        &a,
        &team,
    )
    .await;
    assert!(db::touch_last_seen(&db, &a).await.unwrap());
    let f = g::nudge_facts(&db, &a, &team, now).await.unwrap();
    assert!(f.prev_seen.is_some_and(|t| now - t < Duration::days(2)));
    assert!(pick_nudge(true, &f, &[], None).is_none_or(|n| n.kind != NudgeKind::Returning));

    // Away for three weeks: returning again.
    sql(
        &db,
        "UPDATE $u SET last_seen_at = time::now() - 21d;",
        &a,
        &team,
    )
    .await;
    assert!(db::touch_last_seen(&db, &a).await.unwrap());
    let f = g::nudge_facts(&db, &a, &team, now).await.unwrap();
    assert_eq!(
        pick_nudge(true, &f, &[], None).unwrap().kind,
        NudgeKind::Returning
    );

    // M10 review: a request of the same page load reads before the visit is
    // written (another request is writing it). last_seen_at (yesterday) is
    // then the previous visit, not the older prev_seen_at.
    sql(
        &db,
        "UPDATE $u SET last_seen_at = time::now() - 1d, prev_seen_at = time::now() - 21d;",
        &a,
        &team,
    )
    .await;
    let f = g::nudge_facts(&db, &a, &team, Utc::now()).await.unwrap();
    assert!(f
        .prev_seen
        .is_some_and(|t| Utc::now() - t < Duration::days(2)));
    assert!(pick_nudge(true, &f, &[], None).is_none_or(|n| n.kind != NudgeKind::Returning));
}
