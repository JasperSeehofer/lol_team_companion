//! M6: multi-write paths run in one transaction. Each test makes the last
//! write of a path fail (a test-only ASSERT on a child field) and checks that
//! the earlier writes were rolled back.

use crate::common;
use lol_team_companion::models::draft::DraftAction;
use lol_team_companion::models::game_plan::PostGameLearning;
use lol_team_companion::server::db;
use surrealdb::engine::local::Db;
use surrealdb::types::SurrealValue;
use surrealdb::Surreal;

async fn setup(d: &Surreal<Db>) -> (String, String) {
    let u = db::create_user(d, "tx".into(), "tx@example.com".into(), "h".into())
        .await
        .unwrap();
    let t = db::create_team(d, &u, "Tx".into(), "EUW".into())
        .await
        .unwrap();
    (u, t)
}

async fn count(d: &Surreal<Db>, table: &str) -> usize {
    let mut r = d
        .query(format!("SELECT count() AS n FROM {table} GROUP ALL"))
        .await
        .unwrap();
    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct N {
        n: i64,
    }
    let row: Option<N> = r.take(0).unwrap();
    row.map(|n| n.n as usize).unwrap_or(0)
}

async fn forbid(d: &Surreal<Db>, table: &str, field: &str, value: &str) {
    d.query(format!(
        "DEFINE FIELD OVERWRITE {field} ON {table} TYPE string ASSERT $value != '{value}'"
    ))
    .await
    .unwrap()
    .check()
    .unwrap();
}

fn action(champion: &str, order: i32) -> DraftAction {
    DraftAction {
        id: None,
        draft_id: String::new(),
        phase: "pick1".into(),
        side: "blue".into(),
        champion: champion.into(),
        order,
        comment: None,
        role: None,
    }
}

async fn draft(
    d: &Surreal<Db>,
    t: &str,
    u: &str,
    actions: Vec<DraftAction>,
) -> db::DbResult<String> {
    db::save_draft(
        d,
        t,
        u,
        "Game 1".into(),
        None,
        None,
        vec![],
        actions,
        None,
        "blue".into(),
        vec![],
        None,
        None,
        None,
        None,
    )
    .await
}

#[tokio::test]
async fn save_draft_writes_draft_and_actions_together() {
    let d = common::test_db().await;
    let (u, t) = setup(&d).await;

    let id = draft(&d, &t, &u, vec![action("Ahri", 1), action("Jinx", 2)])
        .await
        .unwrap();
    let drafts = db::list_drafts(&d, &t).await.unwrap();
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].id.as_deref(), Some(id.as_str()));
    assert_eq!(drafts[0].actions.len(), 2);
    assert_eq!(drafts[0].actions[1].champion, "Jinx");

    forbid(&d, "draft_action", "champion", "BOOM").await;
    assert!(
        draft(&d, &t, &u, vec![action("Ahri", 1), action("BOOM", 2)])
            .await
            .is_err()
    );
    assert_eq!(
        count(&d, "draft").await,
        1,
        "the failed draft was rolled back"
    );
    assert_eq!(count(&d, "draft_action").await, 2);
}

#[tokio::test]
async fn update_draft_keeps_old_actions_when_the_rewrite_fails() {
    let d = common::test_db().await;
    let (u, t) = setup(&d).await;
    let id = draft(&d, &t, &u, vec![action("Ahri", 1)]).await.unwrap();

    forbid(&d, "draft_action", "champion", "BOOM").await;
    let r = db::update_draft(
        &d,
        &t,
        &id,
        "Renamed".into(),
        None,
        None,
        vec![],
        vec![action("BOOM", 1)],
        None,
        "red".into(),
        vec![],
        None,
        None,
        None,
        None,
    )
    .await;
    assert!(r.is_err());
    let drafts = db::list_drafts(&d, &t).await.unwrap();
    assert_eq!(drafts[0].name, "Game 1", "the field update was rolled back");
    assert_eq!(drafts[0].our_side, "blue");
    assert_eq!(drafts[0].actions.len(), 1);
    assert_eq!(drafts[0].actions[0].champion, "Ahri");
}

#[tokio::test]
async fn opponent_and_players_are_created_together() {
    let d = common::test_db().await;
    let (_, t) = setup(&d).await;
    forbid(&d, "opponent_player", "role", "BOOM").await;

    let r = db::create_opponent_with_players(
        &d,
        &t,
        "Rivals".into(),
        None,
        vec![("top".into(), None), ("BOOM".into(), None)],
    )
    .await;
    assert!(r.is_err());
    assert_eq!(count(&d, "opponent").await, 0);
    assert_eq!(count(&d, "opponent_player").await, 0);

    let (opp, players) = db::create_opponent_with_players(
        &d,
        &t,
        "Rivals".into(),
        Some("notes".into()),
        vec![("top".into(), None), ("mid".into(), Some("Mid#EUW".into()))],
    )
    .await
    .unwrap();
    assert_eq!(players.len(), 2);
    let (o, loaded) = db::get_opponent(&d, &t, &opp).await.unwrap().unwrap();
    assert_eq!(o.notes.as_deref(), Some("notes"));
    assert_eq!(loaded.len(), 2);
}

fn review(t: &str, u: &str, improvements: &[&str]) -> PostGameLearning {
    PostGameLearning {
        id: None,
        team_id: t.into(),
        match_riot_id: None,
        game_plan_id: None,
        draft_id: None,
        what_went_well: vec![],
        improvements: improvements.iter().map(|s| s.to_string()).collect(),
        action_items: vec![],
        open_notes: None,
        created_by: u.into(),
        win_loss: Some("loss".into()),
        rating: Some(2),
    }
}

#[tokio::test]
async fn review_and_its_action_items_are_written_together() {
    let d = common::test_db().await;
    let (u, t) = setup(&d).await;

    let (id, n) = db::create_review_with_action_items(
        &d,
        review(&t, &u, &["Ward river", " ward RIVER ", "", "Track jungler"]),
    )
    .await
    .unwrap();
    assert_eq!(n, 2, "blank and duplicate improvements are skipped");
    let items = db::list_action_items(&d, &t).await.unwrap();
    assert_eq!(items.len(), 2);
    assert!(items
        .iter()
        .all(|i| i.source_review.as_deref() == Some(id.as_str())));

    // Updating adds only improvements that are not open items yet.
    let mut edit = review(&t, &u, &["Ward river", "Respect timers"]);
    edit.id = Some(id.clone());
    assert_eq!(
        db::update_review_with_action_items(&d, &t, edit)
            .await
            .unwrap(),
        1
    );
    assert_eq!(db::list_action_items(&d, &t).await.unwrap().len(), 3);

    // Action item lines become items too (an open duplicate is skipped).
    let mut edit = review(&t, &u, &["Ward river", "Respect timers"]);
    edit.id = Some(id.clone());
    edit.action_items = vec!["Drill Baron setups".into(), "respect timers".into()];
    assert_eq!(
        db::update_review_with_action_items(&d, &t, edit)
            .await
            .unwrap(),
        1
    );
    assert_eq!(db::list_action_items(&d, &t).await.unwrap().len(), 4);

    // Saving again does not bring back an item of this review that is done.
    let drill = db::list_action_items(&d, &t)
        .await
        .unwrap()
        .into_iter()
        .find(|i| i.text == "Drill Baron setups")
        .and_then(|i| i.id)
        .unwrap();
    db::update_action_item_status(&d, &t, &drill, "done".into())
        .await
        .unwrap();
    let mut again = review(&t, &u, &["Ward river", "Respect timers"]);
    again.id = Some(id.clone());
    again.action_items = vec!["Drill Baron setups".into()];
    assert_eq!(
        db::update_review_with_action_items(&d, &t, again)
            .await
            .unwrap(),
        0,
        "a done item of the same review is not recreated"
    );
    assert_eq!(db::list_action_items(&d, &t).await.unwrap().len(), 4);

    // A failing action item rolls back the review.
    forbid(&d, "action_item", "text", "BOOM").await;
    assert!(
        db::create_review_with_action_items(&d, review(&t, &u, &["fine", "BOOM"]))
            .await
            .is_err()
    );
    assert_eq!(count(&d, "post_game_learning").await, 1);
    assert_eq!(count(&d, "action_item").await, 4);
}

#[tokio::test]
async fn tree_with_root_is_created_and_deleted_as_a_unit() {
    let d = common::test_db().await;
    let (u, t) = setup(&d).await;
    let tree = db::create_draft_tree(&d, &t, &u, "Tree".into(), None)
        .await
        .unwrap();
    let roots = db::get_tree_nodes(&d, &t, &tree).await.unwrap();
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].label, "Tree");
    let root = roots[0].id.clone().unwrap();
    let child = db::create_tree_node(&d, &t, &tree, Some(root.clone()), "A".into())
        .await
        .unwrap();
    db::create_tree_node(&d, &t, &tree, Some(child.clone()), "A1".into())
        .await
        .unwrap();
    db::update_tree_node(
        &d,
        &t,
        &child,
        "A".into(),
        None,
        false,
        vec![action("Ahri", 1)],
    )
    .await
    .unwrap();

    // Deleting a node removes its subtree and their actions.
    db::delete_tree_node(&d, &t, &child).await.unwrap();
    assert_eq!(count(&d, "draft_tree_node").await, 1);
    assert_eq!(count(&d, "tree_node_action").await, 0);

    db::delete_draft_tree(&d, &t, &tree).await.unwrap();
    assert_eq!(count(&d, "draft_tree").await, 0);
    assert_eq!(count(&d, "draft_tree_node").await, 0);
}

#[tokio::test]
async fn deleting_a_series_unlinks_its_drafts() {
    let d = common::test_db().await;
    let (u, t) = setup(&d).await;
    let series = db::create_series(
        &d,
        &t,
        &u,
        "Bo3".into(),
        None,
        Some("Rivals".into()),
        "bo3".into(),
        true,
        None,
    )
    .await
    .unwrap();
    let id = db::save_draft(
        &d,
        &t,
        &u,
        "G1".into(),
        None,
        None,
        vec![],
        vec![action("Ahri", 1)],
        None,
        "blue".into(),
        vec![],
        None,
        None,
        Some(series.clone()),
        Some(1),
    )
    .await
    .unwrap();
    assert_eq!(
        db::get_fearless_used_champions(&d, &t, &series, None)
            .await
            .unwrap(),
        vec!["Ahri".to_string()]
    );
    assert_eq!(
        db::list_series_drafts(&d, &t, &series).await.unwrap().len(),
        1
    );

    db::delete_series(&d, &t, &series).await.unwrap();
    let drafts = db::list_drafts(&d, &t).await.unwrap();
    let d0 = drafts
        .iter()
        .find(|x| x.id.as_deref() == Some(id.as_str()))
        .unwrap();
    assert_eq!(d0.series_id, None);
    assert_eq!(d0.game_number, None);
}

#[tokio::test]
async fn match_detail_cache_is_written_whole() {
    use lol_team_companion::models::match_data::{EventCategory, TimelineEvent};
    let d = common::test_db().await;
    let ev = |et: &str| TimelineEvent {
        timestamp_ms: 60_000,
        event_type: et.into(),
        category: EventCategory::Kill,
        team_id: Some(100),
        killer_participant_id: Some(1),
        victim_participant_id: Some(6),
        monster_type: None,
        monster_sub_type: None,
        building_type: None,
        is_first_blood: true,
        multi_kill_length: None,
        is_teamfight: false,
        involved_participants: vec![1, 6],
    };
    forbid(&d, "match_timeline_event", "event_type", "BOOM").await;
    assert!(db::store_match_detail(
        &d,
        "EUW1_9",
        &[],
        &[ev("CHAMPION_KILL"), ev("BOOM")],
        1800,
        "CLASSIC"
    )
    .await
    .is_err());
    assert!(db::get_cached_match_detail(&d, "EUW1_9")
        .await
        .unwrap()
        .is_none());

    db::store_match_detail(&d, "EUW1_9", &[], &[ev("CHAMPION_KILL")], 1800, "CLASSIC")
        .await
        .unwrap();
    let (_, events, dur, _) = db::get_cached_match_detail(&d, "EUW1_9")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(dur, 1800);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].involved_participants, vec![1, 6]);
}
