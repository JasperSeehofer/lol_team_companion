//! Seed binary: populates the app with realistic test data.
//!
//! Run with: cargo run --features ssr --bin seed
//! Requires: RIOT_API_KEY in .env (optional — falls back to static data)
//! IMPORTANT: Stop the dev server first (SurrealKV exclusive lock)

use std::{env, time::Duration};
use surrealdb::{engine::local::Db, Surreal};

use lol_team_companion::{
    models::{
        champion::{ChampionNote, ChampionPoolEntry},
        draft::DraftAction,
        game_plan::{GamePlan, PostGameLearning},
    },
    server::{auth::hash_password, db},
};

// ---------------------------------------------------------------------------
// Static champion data per role
// ---------------------------------------------------------------------------

const TOP_CHAMPS: &[&str] = &["Darius", "Camille", "Garen", "Fiora", "Aatrox"];
const JG_CHAMPS: &[&str] = &["Hecarim", "Vi", "Amumu", "Khazix", "Gragas"];
const MID_CHAMPS: &[&str] = &["Azir", "Syndra", "Lux", "Zed", "Orianna"];
const BOT_CHAMPS: &[&str] = &["Jinx", "Caitlyn", "Jhin", "Aphelios", "Tristana"];
const SUP_CHAMPS: &[&str] = &["Thresh", "Leona", "Nautilus", "Lulu", "Soraka"];

const TIERS: &[&str] = &["S", "A", "B"];

// Ban phases in draft order: ban1 is 6 bans each side (3+3), pick phases follow
const DRAFT_ACTIONS: &[(&str, &str, &str, i32)] = &[
    // Phase 1 bans (3 blue, 3 red)
    ("ban1", "blue", "Zed", 1),
    ("ban1", "red", "Fiora", 2),
    ("ban1", "blue", "Khazix", 3),
    ("ban1", "red", "Lux", 4),
    ("ban1", "blue", "Tristana", 5),
    ("ban1", "red", "Lulu", 6),
    // Phase 1 picks (blue 1, red 2, blue 2, red 2, blue 1)
    ("pick1", "blue", "Darius", 7),
    ("pick1", "red", "Camille", 8),
    ("pick1", "red", "Syndra", 9),
    ("pick1", "blue", "Azir", 10),
    ("pick1", "blue", "Jinx", 11),
    ("pick1", "red", "Caitlyn", 12),
    ("pick1", "red", "Leona", 13),
    ("pick1", "blue", "Thresh", 14),
    ("pick1", "blue", "Hecarim", 15),
    ("pick1", "red", "Vi", 16),
    // Phase 2 bans (2 red, 2 blue)
    ("ban2", "red", "Orianna", 17),
    ("ban2", "blue", "Amumu", 18),
    ("ban2", "red", "Jhin", 19),
    ("ban2", "blue", "Gragas", 20),
];

// Second draft variant
const DRAFT2_ACTIONS: &[(&str, &str, &str, i32)] = &[
    ("ban1", "blue", "Aatrox", 1),
    ("ban1", "red", "Hecarim", 2),
    ("ban1", "blue", "Aphelios", 3),
    ("ban1", "red", "Jinx", 4),
    ("ban1", "blue", "Soraka", 5),
    ("ban1", "red", "Thresh", 6),
    ("pick1", "blue", "Camille", 7),
    ("pick1", "red", "Darius", 8),
    ("pick1", "red", "Azir", 9),
    ("pick1", "blue", "Orianna", 10),
    ("pick1", "blue", "Caitlyn", 11),
    ("pick1", "red", "Jhin", 12),
    ("pick1", "red", "Nautilus", 13),
    ("pick1", "blue", "Lulu", 14),
    ("pick1", "blue", "Vi", 15),
    ("pick1", "red", "Gragas", 16),
    ("ban2", "red", "Zed", 17),
    ("ban2", "blue", "Syndra", 18),
    ("ban2", "red", "Fiora", 19),
    ("ban2", "blue", "Tristana", 20),
];

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Initialize tracing so DB startup messages are visible
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("warn".parse().unwrap()))
        .init();

    let dry_run = env::args().any(|a| a == "--dry-run");
    let data_dir = env::var("SURREAL_DATA_DIR").unwrap_or_else(|_| "./data".to_string());

    println!("Seed: initializing database from {data_dir}...");
    let db = db::init_db(&data_dir)
        .await
        .expect("DB init failed — is the dev server still running? Stop it first.");

    if dry_run {
        println!("[DRY RUN] Would clean up seed data and create 2 teams with 5 users each.");
        return;
    }

    // Phase 1: Clean up existing seed data
    cleanup_seed_data(&db).await;

    // Phase 2: Create Team A (5 users)
    let team_a_id = create_seed_team(
        &db,
        "Alpha Wolves",
        &[
            ("seed-alpha-top", "SeedTop1", "EUW"),
            ("seed-alpha-jg", "SeedJungle1", "EUW"),
            ("seed-alpha-mid", "SeedMid1", "EUW"),
            ("seed-alpha-bot", "SeedBot1", "EUW"),
            ("seed-alpha-sup", "SeedSup1", "EUW"),
        ],
    )
    .await;

    // Phase 3: Create Team B (5 users)
    let team_b_id = create_seed_team(
        &db,
        "Beta Dragons",
        &[
            ("seed-beta-top", "SeedTop2", "EUW"),
            ("seed-beta-jg", "SeedJungle2", "EUW"),
            ("seed-beta-mid", "SeedMid2", "EUW"),
            ("seed-beta-bot", "SeedBot2", "EUW"),
            ("seed-beta-sup", "SeedSup2", "EUW"),
        ],
    )
    .await;

    // Phase 4: Populate data for each team
    populate_team_data(&db, &team_a_id, "Alpha Wolves").await;
    populate_team_data(&db, &team_b_id, "Beta Dragons").await;

    // Phase 5: Create opponent scouting profiles
    create_opponent_profiles(&db, &team_a_id).await;

    println!("\nSeed complete! Created 2 teams with full demo data.");
    println!("Login with any seed account: seed-alpha-top@example.com / seedpass123");
}

// ---------------------------------------------------------------------------
// Cleanup
// ---------------------------------------------------------------------------

async fn cleanup_seed_data(db: &Surreal<Db>) {
    println!("Cleaning up existing seed data...");
    let queries = [
        "DELETE post_game_learning WHERE created_by IN (SELECT VALUE id FROM user WHERE string::starts_with(email, 'seed-'))",
        "DELETE game_plan WHERE team IN (SELECT VALUE id FROM team WHERE string::starts_with(name, 'Alpha') OR string::starts_with(name, 'Beta'))",
        "DELETE draft_action WHERE draft IN (SELECT VALUE id FROM draft WHERE team IN (SELECT VALUE id FROM team WHERE string::starts_with(name, 'Alpha') OR string::starts_with(name, 'Beta')))",
        "DELETE draft WHERE team IN (SELECT VALUE id FROM team WHERE string::starts_with(name, 'Alpha') OR string::starts_with(name, 'Beta'))",
        "DELETE champion_note WHERE user IN (SELECT VALUE id FROM user WHERE string::starts_with(email, 'seed-'))",
        "DELETE champion_pool WHERE user IN (SELECT VALUE id FROM user WHERE string::starts_with(email, 'seed-'))",
        "DELETE team_member WHERE team IN (SELECT VALUE id FROM team WHERE string::starts_with(name, 'Alpha') OR string::starts_with(name, 'Beta'))",
        "DELETE opponent_player WHERE opponent IN (SELECT VALUE id FROM opponent WHERE team IN (SELECT VALUE id FROM team WHERE string::starts_with(name, 'Alpha') OR string::starts_with(name, 'Beta')))",
        "DELETE opponent WHERE team IN (SELECT VALUE id FROM team WHERE string::starts_with(name, 'Alpha') OR string::starts_with(name, 'Beta'))",
        "DELETE team WHERE string::starts_with(name, 'Alpha') OR string::starts_with(name, 'Beta')",
        "DELETE user WHERE string::starts_with(email, 'seed-')",
    ];
    for q in queries {
        if let Err(e) = db.query(q).await {
            eprintln!("  Warning during cleanup: {e}");
        }
    }
    println!("  Cleanup done.");
}

// ---------------------------------------------------------------------------
// Team + User creation
// ---------------------------------------------------------------------------

/// Creates a team with the given users. The first user becomes the team owner.
/// Returns the team ID.
async fn create_seed_team(
    db: &Surreal<Db>,
    team_name: &str,
    users: &[(&str, &str, &str)], // (username, riot_name, tag)
) -> String {
    let password_hash = hash_password("seedpass123").expect("hash_password failed");

    // Create all users first
    let mut user_ids: Vec<String> = Vec::new();
    for (username, riot_name, _tag) in users {
        let email = format!("{username}@example.com");
        match db::create_user(
            db,
            username.to_string(),
            email.clone(),
            password_hash.clone(),
        )
        .await
        {
            Ok(id) => {
                println!("  Created user {email} → {id}");
                // Optionally try to link Riot account (best-effort, no API key needed)
                if lol_team_companion::server::riot::has_api_key() {
                    // Rate-limit: 150ms between calls
                    tokio::time::sleep(Duration::from_millis(150)).await;
                    match lol_team_companion::server::riot::get_puuid(riot_name, "EUW", lol_team_companion::server::riot::platform_route_from_str("EUW")).await {
                        Ok(puuid) => {
                            if let Err(e) =
                                db::update_user_riot(db, id.clone(), puuid, riot_name.to_string())
                                    .await
                            {
                                eprintln!("    Warning: could not link Riot account for {email}: {e}");
                            }
                        }
                        Err(e) => {
                            eprintln!("    Warning: Riot API error for {riot_name}#EUW: {e}");
                        }
                    }
                }
                user_ids.push(id);
            }
            Err(e) => {
                eprintln!("  Error creating user {email}: {e}");
            }
        }
    }

    if user_ids.is_empty() {
        panic!("Failed to create any users for team {team_name}");
    }

    // First user creates the team (becomes owner + member)
    let owner_id = &user_ids[0];
    let team_id = match db::create_team(db, owner_id, team_name.to_string(), "EUW".to_string())
        .await
    {
        Ok(id) => {
            println!("  Created team {team_name} → {id}");
            id
        }
        Err(e) => {
            panic!("Failed to create team {team_name}: {e}");
        }
    };

    // Add remaining users as team members
    for user_id in user_ids.iter().skip(1) {
        if let Err(e) = db::join_team(db, user_id, &team_id).await {
            eprintln!("  Warning: could not add user {user_id} to team {team_id}: {e}");
        }
    }

    team_id
}

// ---------------------------------------------------------------------------
// Data population
// ---------------------------------------------------------------------------

/// Gets the user IDs for all members of a team.
async fn get_team_member_ids(db: &Surreal<Db>, team_id: &str) -> Vec<String> {
    use serde::Deserialize;
    use surrealdb::types::{RecordId, SurrealValue, ToSql};

    #[derive(Debug, Deserialize, SurrealValue)]
    struct MemberRow {
        user: RecordId,
    }

    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query("SELECT user FROM team_member WHERE team = type::record('team', $team_key)")
        .bind(("team_key", team_key))
        .await
        .unwrap_or_else(|e| panic!("get_team_member_ids query failed: {e}"));

    let rows: Vec<MemberRow> = r.take(0).unwrap_or_default();
    rows.into_iter().map(|r| r.user.to_sql()).collect()
}

async fn populate_team_data(db: &Surreal<Db>, team_id: &str, team_name: &str) {
    println!("  Populating data for team {team_name}...");

    let member_ids = get_team_member_ids(db, team_id).await;
    if member_ids.is_empty() {
        eprintln!("  Warning: no members found for team {team_name}, skipping data population");
        return;
    }

    let role_champs: &[&[&str]] = &[TOP_CHAMPS, JG_CHAMPS, MID_CHAMPS, BOT_CHAMPS, SUP_CHAMPS];
    let roles = ["top", "jungle", "mid", "bot", "support"];

    // --- Champion pool entries ---
    for (i, user_id) in member_ids.iter().enumerate() {
        let champs = role_champs[i % role_champs.len()];
        let role = roles[i % roles.len()];
        for (j, champ) in champs.iter().enumerate() {
            let tier = TIERS[j % TIERS.len()].to_string();
            let entry = ChampionPoolEntry {
                id: None,
                user_id: user_id.clone(),
                champion: champ.to_string(),
                role: role.to_string(),
                tier,
                notes: Some(format!("Strong in current meta")),
                comfort_level: Some((5 - (j % 5)) as u8),
                meta_tag: Some("strong".to_string()),
            };
            if let Err(e) = save_champion_pool_entry(db, &entry).await {
                eprintln!("    Warning: champion pool entry for {champ}: {e}");
            }
        }

        // --- Champion notes ---
        let champ0 = champs[0];
        let champ1 = if champs.len() > 1 { champs[1] } else { champs[0] };
        let notes = [
            ChampionNote {
                id: None,
                user_id: user_id.clone(),
                champion: champ0.to_string(),
                role: role.to_string(),
                note_type: "power_spike".to_string(),
                title: "Level 6 spike".to_string(),
                content: format!("{champ0} becomes very strong at level 6. Look for all-in after hitting 6."),
                difficulty: Some(3),
                created_at: None,
            },
            ChampionNote {
                id: None,
                user_id: user_id.clone(),
                champion: champ1.to_string(),
                role: role.to_string(),
                note_type: "matchup".to_string(),
                title: "Weak vs poke".to_string(),
                content: format!("{champ1} struggles against long-range poke. Play safe early and wait for jungler."),
                difficulty: Some(4),
                created_at: None,
            },
        ];
        for note in notes {
            if let Err(e) = db::add_champion_note(db, user_id, note).await {
                eprintln!("    Warning: champion note: {e}");
            }
        }
    }

    // --- Drafts (2-3 per team) ---
    let owner_id = member_ids.first().cloned().unwrap_or_default();

    let draft1_id = save_draft_from_actions(
        db,
        team_id,
        &owner_id,
        &format!("{team_name} vs Beta — Game 1"),
        DRAFT_ACTIONS,
    )
    .await;

    let draft2_id = save_draft_from_actions(
        db,
        team_id,
        &owner_id,
        &format!("{team_name} vs Alpha — Game 2"),
        DRAFT2_ACTIONS,
    )
    .await;

    // --- Game plans (linked to drafts) ---

    let plan1_id = if let Some(ref d_id) = draft1_id {
        let plan = GamePlan {
            id: None,
            team_id: team_id.to_string(),
            draft_id: Some(d_id.clone()),
            name: format!("{team_name} vs Beta — Teamfight Plan"),
            our_champions: vec!["Darius".to_string(), "Hecarim".to_string(), "Azir".to_string(), "Jinx".to_string(), "Thresh".to_string()],
            enemy_champions: vec!["Camille".to_string(), "Vi".to_string(), "Syndra".to_string(), "Caitlyn".to_string(), "Leona".to_string()],
            win_conditions: vec!["Control teamfights".to_string(), "Peel for Jinx in fights".to_string()],
            objective_priority: vec!["Dragon".to_string(), "Baron".to_string()],
            teamfight_strategy: "Thresh hook initiates, Hecarim follow-up engage, Darius pulls stragglers".to_string(),
            early_game: Some("Level 1 invade potential with Hecarim. Control early river vision.".to_string()),
            top_strategy: Some("Darius — trade when E is on cooldown. Win lane and join teamfights.".to_string()),
            jungle_strategy: Some("Hecarim — clear fast, path toward mid to enable Azir.".to_string()),
            mid_strategy: Some("Azir — farm to 3 items. Do not die. Control vision around mid.".to_string()),
            bot_strategy: Some("Jinx — stay safe early, farm up to powerspike at 2 items.".to_string()),
            support_strategy: Some("Thresh — catch priority with hooks. Save lantern for carries.".to_string()),
            notes: Some("Watch out for Syndra burst — CC chain before she can 100-0.".to_string()),
            win_condition_tag: Some("teamfight".to_string()),
        };
        match db::save_game_plan(db, plan, &owner_id).await {
            Ok(id) => {
                println!("    Created game plan → {id}");
                Some(id)
            }
            Err(e) => {
                eprintln!("    Warning: game plan 1: {e}");
                None
            }
        }
    } else {
        None
    };

    let plan2_id = if let Some(ref d_id) = draft2_id {
        let plan = GamePlan {
            id: None,
            team_id: team_id.to_string(),
            draft_id: Some(d_id.clone()),
            name: format!("{team_name} vs Alpha — Split Push Plan"),
            our_champions: vec!["Camille".to_string(), "Vi".to_string(), "Orianna".to_string(), "Caitlyn".to_string(), "Lulu".to_string()],
            enemy_champions: vec!["Darius".to_string(), "Hecarim".to_string(), "Azir".to_string(), "Jinx".to_string(), "Thresh".to_string()],
            win_conditions: vec!["Camille split push top".to_string(), "Orianna ball control in teamfights".to_string()],
            objective_priority: vec!["Rift Herald".to_string(), "Towers".to_string(), "Baron".to_string()],
            teamfight_strategy: "Orianna ultimate into Vi W for engage. Lulu shield carries.".to_string(),
            early_game: Some("Vi invade red buff. Camille look for level 2 trade with E.".to_string()),
            top_strategy: Some("Camille — split push, draw pressure, teleport to teamfights.".to_string()),
            jungle_strategy: Some("Vi — pressure mid-jungle corridor. Herald setup.".to_string()),
            mid_strategy: Some("Orianna — poke and follow flanks. Hold ball on Vi for engage.".to_string()),
            bot_strategy: Some("Caitlyn — trap placement under tower. Farm safely.".to_string()),
            support_strategy: Some("Lulu — keep carries alive, polymorph engage threats.".to_string()),
            notes: Some("If Darius ults Camille she needs to disengage. Do not 5v5 early.".to_string()),
            win_condition_tag: Some("split-push".to_string()),
        };
        match db::save_game_plan(db, plan, &owner_id).await {
            Ok(id) => {
                println!("    Created game plan → {id}");
                Some(id)
            }
            Err(e) => {
                eprintln!("    Warning: game plan 2: {e}");
                None
            }
        }
    } else {
        None
    };

    // --- Post-game reviews (linked to game plans) ---
    if let Some(ref gp_id) = plan1_id {
        let review = PostGameLearning {
            id: None,
            team_id: team_id.to_string(),
            match_riot_id: None,
            game_plan_id: Some(gp_id.clone()),
            draft_id: draft1_id.clone(),
            what_went_well: vec![
                "Hecarim engage was clean at Dragon fight".to_string(),
                "Jinx stayed safe and converted teamfight damage".to_string(),
            ],
            improvements: vec![
                "Need better vision control before Baron spawns".to_string(),
                "Mid lane lost too much tempo after roaming".to_string(),
            ],
            action_items: vec![
                "Practice Thresh hook timing".to_string(),
                "Review Baron vision setup".to_string(),
            ],
            open_notes: Some("Overall solid execution. Need to work on late-game decision-making.".to_string()),
            created_by: owner_id.clone(),
            win_loss: Some("win".to_string()),
            rating: Some(4),
        };
        match db::save_post_game_learning(db, review).await {
            Ok(id) => println!("    Created post-game review (win) → {id}"),
            Err(e) => eprintln!("    Warning: post-game review 1: {e}"),
        }
    }

    if let Some(ref gp_id) = plan2_id {
        let review = PostGameLearning {
            id: None,
            team_id: team_id.to_string(),
            match_riot_id: None,
            game_plan_id: Some(gp_id.clone()),
            draft_id: draft2_id.clone(),
            what_went_well: vec![
                "Camille split push drew consistent pressure".to_string(),
                "Vi made good engages with Orianna ball".to_string(),
            ],
            improvements: vec![
                "Lulu positioning was too aggressive in lane".to_string(),
                "Teleport timing on Camille was late for one key fight".to_string(),
            ],
            action_items: vec![
                "Practice Orianna ball placement for Vi W".to_string(),
                "Review Camille TP decision points".to_string(),
            ],
            open_notes: Some("Good macro execution but micro was shaky. Keep practicing the TP timing.".to_string()),
            created_by: owner_id.clone(),
            win_loss: Some("loss".to_string()),
            rating: Some(3),
        };
        match db::save_post_game_learning(db, review).await {
            Ok(id) => println!("    Created post-game review (loss) → {id}"),
            Err(e) => eprintln!("    Warning: post-game review 2: {e}"),
        }
    }

    // A third post-game review without a game plan (standalone)
    let review3 = PostGameLearning {
        id: None,
        team_id: team_id.to_string(),
        match_riot_id: None,
        game_plan_id: None,
        draft_id: None,
        what_went_well: vec![
            "Strong early game coordination".to_string(),
            "Objective control was excellent".to_string(),
        ],
        improvements: vec![
            "Need to close games faster when ahead".to_string(),
            "Overstayed in fights when outnumbered".to_string(),
        ],
        action_items: vec![
            "Practice early surrender of bad fights".to_string(),
        ],
        open_notes: Some("Scrim session — solid effort, keep the momentum.".to_string()),
        created_by: owner_id.clone(),
        win_loss: Some("win".to_string()),
        rating: Some(5),
    };
    match db::save_post_game_learning(db, review3).await {
        Ok(id) => println!("    Created post-game review (win, no plan) → {id}"),
        Err(e) => eprintln!("    Warning: post-game review 3: {e}"),
    }

    println!("  Team {team_name} population complete.");
}

/// Save a draft using pre-defined action tuples.
async fn save_draft_from_actions(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    name: &str,
    action_tuples: &[(&str, &str, &str, i32)],
) -> Option<String> {
    let actions: Vec<DraftAction> = action_tuples
        .iter()
        .map(|(phase, side, champion, order)| DraftAction {
            id: None,
            draft_id: String::new(), // will be set by DB
            phase: phase.to_string(),
            side: side.to_string(),
            champion: champion.to_string(),
            order: *order,
            comment: None,
            role: None,
        })
        .collect();

    match db::save_draft(
        db,
        team_id,
        user_id,
        name.to_string(),
        None,
        None,
        vec![],
        actions,
        None,
        "blue".to_string(),
        vec!["teamfight".to_string()],
        Some("Control teamfights and peel for carries".to_string()),
        None,
        None,
        None,
    )
    .await
    {
        Ok(id) => {
            println!("    Created draft '{name}' → {id}");
            Some(id)
        }
        Err(e) => {
            eprintln!("    Warning: draft '{name}': {e}");
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Champion pool save helper (inline since db.rs doesn't have save_champion_pool_entry)
// ---------------------------------------------------------------------------

async fn save_champion_pool_entry(
    db: &Surreal<Db>,
    entry: &ChampionPoolEntry,
) -> Result<(), String> {
    let user_key = entry
        .user_id
        .strip_prefix("user:")
        .unwrap_or(&entry.user_id)
        .to_string();

    db.query(
        "CREATE champion_pool SET user = type::record('user', $user_key), champion = $champion, role = $role, tier = $tier, notes = $notes, comfort_level = $comfort_level, meta_tag = $meta_tag",
    )
    .bind(("user_key", user_key))
    .bind(("champion", entry.champion.clone()))
    .bind(("role", entry.role.clone()))
    .bind(("tier", entry.tier.clone()))
    .bind(("notes", entry.notes.clone()))
    .bind(("comfort_level", entry.comfort_level.map(|v| v as i64)))
    .bind(("meta_tag", entry.meta_tag.clone()))
    .await
    .map_err(|e| e.to_string())?
    .check()
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Opponent profiles
// ---------------------------------------------------------------------------

async fn create_opponent_profiles(db: &Surreal<Db>, team_id: &str) {
    println!("  Creating opponent scouting profiles...");

    let players: Vec<(String, Option<String>)> = vec![
        ("top".to_string(), Some("EnemyTop1".to_string())),
        ("jungle".to_string(), Some("EnemyJungle1".to_string())),
        ("mid".to_string(), Some("EnemyMid1".to_string())),
        ("bot".to_string(), Some("EnemyBot1".to_string())),
        ("support".to_string(), Some("EnemySup1".to_string())),
    ];

    match db::create_opponent_with_players(
        db,
        team_id,
        "Beta Dragons".to_string(),
        Some("Main rivals — strong teamfighting and dragon control".to_string()),
        players,
    )
    .await
    {
        Ok((opp_id, player_ids)) => {
            println!("    Created opponent 'Beta Dragons' → {opp_id} ({} players)", player_ids.len());
        }
        Err(e) => {
            eprintln!("    Warning: opponent creation: {e}");
        }
    }
}
