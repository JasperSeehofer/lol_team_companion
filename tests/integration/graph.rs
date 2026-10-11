//! Draft Graph storage: the trait table (sync from the shipped defaults,
//! vocabulary enforced by the schema, maintainer edits kept) and the
//! match-evidence reads. Cross-tenant cases for team relations and team
//! games live in `security.rs`.

use crate::common;

use lol_team_companion::models::graph::{Engage, TraitSource};
use lol_team_companion::server::db::{self, DbError};
use lol_team_companion::server::graph::traits;

#[tokio::test]
async fn sync_writes_a_valid_row_for_every_default() {
    let db = common::test_db().await;
    let report = traits::sync(&db).await.unwrap();
    let defaults = traits::defaults();
    assert_eq!(report.created, defaults.len());
    let rows = db::list_champion_traits(&db).await.unwrap();
    // Audit: every default row came back through the vocabulary (a row
    // outside it would be skipped and the counts would differ).
    assert_eq!(rows.len(), defaults.len());
    for r in &rows {
        assert_eq!(r.source, TraitSource::Default);
        r.traits.validate().unwrap();
        assert_eq!(
            Some(&r.traits),
            traits::default_for(&r.traits.champion),
            "{} matches the file",
            r.traits.champion
        );
    }
    // A second sync has nothing to do.
    let again = traits::sync(&db).await.unwrap();
    assert_eq!((again.created, again.refreshed), (0, 0));
}

#[tokio::test]
async fn schema_rejects_values_outside_the_vocabulary() {
    let db = common::test_db().await;
    traits::sync(&db).await.unwrap();
    for (field, value) in [
        ("damage_type", "arcane"),
        ("engage", "maybe"),
        ("cc", "lots"),
        ("source", "community"),
    ] {
        let q = format!("UPDATE champion_trait:Ahri SET {field} = $v");
        let r = db
            .query(q)
            .bind(("v", value.to_string()))
            .await
            .unwrap()
            .check();
        assert!(r.is_err(), "{field} = {value} must be rejected");
    }
    let r = db
        .query("UPDATE champion_trait:Ahri SET roles = ['middle']")
        .await
        .unwrap()
        .check();
    assert!(r.is_err(), "unknown role");
    let r = db
        .query("UPDATE champion_trait:Ahri SET roles = []")
        .await
        .unwrap()
        .check();
    assert!(r.is_err(), "no role");
    let r = db
        .query("UPDATE champion_trait:Ahri SET styles = ['flank']")
        .await
        .unwrap()
        .check();
    assert!(r.is_err(), "unknown playstyle");
}

#[tokio::test]
async fn maintainer_edits_survive_sync_and_can_be_reset() {
    let db = common::test_db().await;
    traits::sync(&db).await.unwrap();
    let admin = db::create_user(&db, "maint".into(), "maint@example.com".into(), "h".into())
        .await
        .unwrap();
    let mut ahri = traits::default_for("Ahri").unwrap().clone();
    ahri.engage = Engage::Hard;
    db::save_champion_traits(&db, &ahri, &admin, "maint", Some("16.20.1".into()))
        .await
        .unwrap();

    // Sync (and a direct default write) leave the edit alone.
    traits::sync(&db).await.unwrap();
    db::write_default_traits(&db, vec![traits::default_for("Ahri").unwrap().clone()])
        .await
        .unwrap();
    let rows = db::list_champion_traits(&db).await.unwrap();
    let row = rows.iter().find(|r| r.traits.champion == "Ahri").unwrap();
    assert_eq!(row.source, TraitSource::Maintainer);
    assert_eq!(row.traits.engage, Engage::Hard);
    assert_eq!(row.edited_by.as_deref(), Some("maint"));
    assert!(row.edited_at.is_some());
    assert_eq!(row.reviewed_for_patch.as_deref(), Some("16.20.1"));
    // The in-memory table follows a reload.
    let table = traits::reload(&db).await.unwrap();
    assert_eq!(table.get("Ahri").unwrap().traits.engage, Engage::Hard);

    db::reset_champion_traits(&db, traits::default_for("Ahri").unwrap())
        .await
        .unwrap();
    let rows = db::list_champion_traits(&db).await.unwrap();
    let row = rows.iter().find(|r| r.traits.champion == "Ahri").unwrap();
    assert_eq!(row.source, TraitSource::Default);
    assert_eq!(row.edited_by, None);
    assert_eq!(&row.traits, traits::default_for("Ahri").unwrap());
}

#[tokio::test]
async fn editing_an_unknown_champion_is_not_found() {
    let db = common::test_db().await;
    traits::sync(&db).await.unwrap();
    let admin = db::create_user(&db, "maint".into(), "maint@example.com".into(), "h".into())
        .await
        .unwrap();
    let mut ghost = traits::default_for("Ahri").unwrap().clone();
    ghost.champion = "Nobody".into();
    match db::save_champion_traits(&db, &ghost, &admin, "maint", None).await {
        Err(DbError::NotFound) => {}
        other => panic!("expected NotFound, got {other:?}"),
    }
    let rows = db::list_champion_traits(&db).await.unwrap();
    assert!(!rows.iter().any(|r| r.traits.champion == "Nobody"));
    // A malformed id is refused before any query.
    ghost.champion = "Ahri'); DELETE champion_trait; --".into();
    assert!(matches!(
        db::save_champion_traits(&db, &ghost, &admin, "maint", None).await,
        Err(DbError::Invalid(_))
    ));
    assert_eq!(
        db::list_champion_traits(&db).await.unwrap().len(),
        traits::defaults().len()
    );
}

#[tokio::test]
async fn team_games_group_by_match_for_pair_evidence() {
    use lol_team_companion::server::riot::MatchData;
    let db = common::test_db().await;
    let a = db::create_user(&db, "a".into(), "a@example.com".into(), "h".into())
        .await
        .unwrap();
    let b = db::create_user(&db, "b".into(), "b@example.com".into(), "h".into())
        .await
        .unwrap();
    let team = db::create_team(&db, &a, "T".into(), "EUW".into())
        .await
        .unwrap();
    db::join_team(&db, &b, &team).await.unwrap();
    let m = |id: &str, champ: &str, win: bool| MatchData {
        match_id: id.into(),
        queue_id: 420,
        game_duration: 1800,
        game_end_epoch_ms: Some(1_700_000_000_000),
        champion: champ.into(),
        kills: 0,
        deaths: 0,
        assists: 0,
        cs: 0,
        vision_score: 0,
        damage: 0,
        win,
    };
    db::store_matches(
        &db,
        &a,
        vec![m("EUW1_10", "Jinx", true), m("EUW1_11", "Jinx", false)],
    )
    .await
    .unwrap();
    db::store_matches(&db, &b, vec![m("EUW1_10", "Thresh", true)])
        .await
        .unwrap();
    let rows = db::team_game_rows(&db, &team).await.unwrap();
    assert_eq!(rows.len(), 3);
    let shared: Vec<_> = rows
        .iter()
        .filter(|r| r.riot_match_id == "EUW1_10")
        .collect();
    assert_eq!(shared.len(), 2);
    assert_eq!(
        shared[0].match_key, shared[1].match_key,
        "same match record"
    );
    let ev = lol_team_companion::server::graph::evidence::compute(
        &rows,
        &[],
        &Default::default(),
        |n| Some(n.to_string()),
    );
    assert_eq!(ev.pair("Jinx", "Thresh").unwrap().games, 1);
    assert!(db::match_participants(&db, vec!["EUW1_10".into()])
        .await
        .unwrap()
        .is_empty());
}
