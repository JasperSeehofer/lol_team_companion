#![cfg(feature = "ssr")]

mod common;

use lol_team_companion::server::db;

async fn setup(db: &surrealdb::Surreal<surrealdb::engine::local::Db>) -> (String, String) {
    let user_id = db::create_user(db, "treeuser".into(), "tree@example.com".into(), "h".into())
        .await
        .unwrap();
    let team_id = db::create_team(db, &user_id, "TreeTeam".into(), "EUW".into())
        .await
        .unwrap();
    (user_id, team_id)
}

#[tokio::test]
async fn test_create_tree_creates_root_node() {
    let db = common::test_db().await;
    let (user_id, team_id) = setup(&db).await;

    let tree_id = db::create_draft_tree(&db, &team_id, &user_id, "My Tree".into(), None)
        .await
        .unwrap();

    assert!(tree_id.starts_with("draft_tree:"));

    let nodes = db::get_tree_nodes(&db, &tree_id).await.unwrap();
    assert_eq!(nodes.len(), 1, "should have auto-created root node");
    assert_eq!(nodes[0].label, "Root");
    assert!(nodes[0].parent_id.is_none());
}

#[tokio::test]
async fn test_add_child_node() {
    let db = common::test_db().await;
    let (user_id, team_id) = setup(&db).await;

    let tree_id = db::create_draft_tree(&db, &team_id, &user_id, "Tree".into(), None)
        .await
        .unwrap();

    let nodes = db::get_tree_nodes(&db, &tree_id).await.unwrap();
    let root_id = nodes[0].id.clone().unwrap();

    let child_id =
        db::create_tree_node(&db, &tree_id, Some(root_id.clone()), "Child A".into())
            .await
            .unwrap();

    assert!(child_id.starts_with("draft_tree_node:"));

    let nodes = db::get_tree_nodes(&db, &tree_id).await.unwrap();
    assert_eq!(nodes.len(), 1, "root-level should have 1 root");
    assert_eq!(nodes[0].children.len(), 1, "root should have 1 child");
    assert_eq!(nodes[0].children[0].label, "Child A");
}

#[tokio::test]
async fn test_get_tree_nodes_reconstructs_hierarchy() {
    let db = common::test_db().await;
    let (user_id, team_id) = setup(&db).await;

    let tree_id = db::create_draft_tree(&db, &team_id, &user_id, "Hierarchy".into(), None)
        .await
        .unwrap();

    let nodes = db::get_tree_nodes(&db, &tree_id).await.unwrap();
    let root_id = nodes[0].id.clone().unwrap();

    let child1 = db::create_tree_node(&db, &tree_id, Some(root_id.clone()), "Child 1".into())
        .await
        .unwrap();
    db::create_tree_node(&db, &tree_id, Some(child1.clone()), "Grandchild".into())
        .await
        .unwrap();
    db::create_tree_node(&db, &tree_id, Some(root_id.clone()), "Child 2".into())
        .await
        .unwrap();

    let tree = db::get_tree_nodes(&db, &tree_id).await.unwrap();
    assert_eq!(tree.len(), 1);
    let root = &tree[0];
    assert_eq!(root.children.len(), 2);
    let c1 = root.children.iter().find(|c| c.label == "Child 1").unwrap();
    assert_eq!(c1.children.len(), 1);
    assert_eq!(c1.children[0].label, "Grandchild");
}

#[tokio::test]
async fn test_delete_node_cascades_to_children() {
    let db = common::test_db().await;
    let (user_id, team_id) = setup(&db).await;

    let tree_id = db::create_draft_tree(&db, &team_id, &user_id, "Cascade".into(), None)
        .await
        .unwrap();

    let nodes = db::get_tree_nodes(&db, &tree_id).await.unwrap();
    let root_id = nodes[0].id.clone().unwrap();

    let child = db::create_tree_node(&db, &tree_id, Some(root_id.clone()), "Child".into())
        .await
        .unwrap();
    db::create_tree_node(&db, &tree_id, Some(child.clone()), "Grandchild".into())
        .await
        .unwrap();

    // Delete child — grandchild should also be gone
    db::delete_tree_node(&db, &child).await.unwrap();

    let tree = db::get_tree_nodes(&db, &tree_id).await.unwrap();
    assert_eq!(tree.len(), 1, "only root should remain");
    assert!(tree[0].children.is_empty(), "child and grandchild should be deleted");
}

#[tokio::test]
async fn test_tree_sort_order_is_maintained() {
    let db = common::test_db().await;
    let (user_id, team_id) = setup(&db).await;

    let tree_id = db::create_draft_tree(&db, &team_id, &user_id, "Sorted".into(), None)
        .await
        .unwrap();
    let nodes = db::get_tree_nodes(&db, &tree_id).await.unwrap();
    let root_id = nodes[0].id.clone().unwrap();

    db::create_tree_node(&db, &tree_id, Some(root_id.clone()), "First".into())
        .await
        .unwrap();
    db::create_tree_node(&db, &tree_id, Some(root_id.clone()), "Second".into())
        .await
        .unwrap();
    db::create_tree_node(&db, &tree_id, Some(root_id.clone()), "Third".into())
        .await
        .unwrap();

    let tree = db::get_tree_nodes(&db, &tree_id).await.unwrap();
    let children = &tree[0].children;
    assert_eq!(children.len(), 3);
    // sort_order should be 0, 1, 2 — verify they are sorted ascending
    let orders: Vec<i32> = children.iter().map(|c| c.sort_order).collect();
    let mut sorted = orders.clone();
    sorted.sort();
    assert_eq!(orders, sorted, "children should be in sort_order order");
}
