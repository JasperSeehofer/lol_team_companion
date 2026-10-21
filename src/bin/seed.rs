//! Seed tool: fills a database with demo data for local development.
//!
//!     cargo run --features ssr --bin seed            # seed ./data (or SURREAL_DATA_DIR)
//!     cargo run --features ssr --bin seed -- --dry-run
//!     cargo run --features ssr --bin seed -- --link-riot   # try to link Riot IDs (fictional names)
//!
//! Stop the server first: SurrealKV allows one process per data directory.
//!
//! Idempotent: every run first removes what an earlier run created (the
//! accounts whose email starts with `seed-` and ends with `@example.com`,
//! the teams they created and everything those teams own, and invites with
//! the note `seed`), then creates it again. Nothing else is touched.
//!
//! What it creates: two teams of five players each (a leader plus four
//! members, all assigned to starting roles), each with two drafts, two game
//! plans linked to them, three post-game reviews (whose improvements become
//! action items), a team note, champion pools and notes, and a scouting
//! profile of the other team. Plus one invite code (25 uses, 30 days).
//! Names are ordinary player handles; champion picks are real champions.

use std::{env, time::Duration};
use surrealdb::types::SurrealValue;
use surrealdb::{engine::local::Db, Surreal};

use lol_team_companion::{
    models::{
        champion::ChampionNote,
        draft::DraftAction,
        game_plan::{GamePlan, PostGameLearning},
        graph::{NewRelation, RelationKind},
        match_data::{
            EventCategory, MatchParticipant, TimelineEvent, DEMO_MATCH_ID, DEMO_MATCH_PREFIX,
        },
    },
    server::{
        auth::hash_password,
        db,
        riot::{self, RiotClient},
    },
};

const PASSWORD: &str = "seedpass123";
const ROLES: [&str; 5] = ["top", "jungle", "mid", "bot", "support"];

/// A seeded player: email local part, username, Riot ID game name.
struct Player {
    login: &'static str,
    username: &'static str,
    riot_name: &'static str,
}

struct TeamSpec {
    name: &'static str,
    region: &'static str,
    players: [Player; 5],
    /// Champion pool per role, in ROLES order.
    pools: [&'static [&'static str]; 5],
}

const TEAMS: [TeamSpec; 2] = [
    TeamSpec {
        name: "Northgate Academy",
        region: "EUW",
        players: [
            Player {
                login: "seed-alpha-top",
                username: "Linnea Voss",
                riot_name: "LinVoss",
            },
            Player {
                login: "seed-alpha-jg",
                username: "Tomas Kral",
                riot_name: "TKral",
            },
            Player {
                login: "seed-alpha-mid",
                username: "Yusuf Demir",
                riot_name: "Ydemir",
            },
            Player {
                login: "seed-alpha-bot",
                username: "Ines Moreau",
                riot_name: "InesM",
            },
            Player {
                login: "seed-alpha-sup",
                username: "Oskar Lind",
                riot_name: "OLind",
            },
        ],
        pools: [
            &["Darius", "Camille", "Garen", "Fiora", "Aatrox"],
            &["Hecarim", "Vi", "Amumu", "Khazix", "Gragas"],
            &["Azir", "Syndra", "Lux", "Zed", "Orianna"],
            &["Jinx", "Caitlyn", "Jhin", "Aphelios", "Tristana"],
            &["Thresh", "Leona", "Nautilus", "Lulu", "Soraka"],
        ],
    },
    TeamSpec {
        name: "Riverside Rooks",
        region: "EUW",
        players: [
            Player {
                login: "seed-beta-top",
                username: "Mara Ilic",
                riot_name: "MaraI",
            },
            Player {
                login: "seed-beta-jg",
                username: "Jonas Berg",
                riot_name: "JBerg",
            },
            Player {
                login: "seed-beta-mid",
                username: "Sofia Reyes",
                riot_name: "SReyes",
            },
            Player {
                login: "seed-beta-bot",
                username: "Daniel Okafor",
                riot_name: "DOkafor",
            },
            Player {
                login: "seed-beta-sup",
                username: "Hanna Wiik",
                riot_name: "HWiik",
            },
        ],
        pools: [
            &["Renekton", "Gnar", "Jax", "Ornn", "KSante"],
            &["LeeSin", "Sejuani", "Viego", "Maokai", "Elise"],
            &["Ahri", "Viktor", "Sylas", "Taliyah", "Akali"],
            &["Ezreal", "Kaisa", "Varus", "Xayah", "Ashe"],
            &["Rakan", "Alistar", "Braum", "Nami", "Rell"],
        ],
    },
];

/// (phase, side, champion, order) for a full 20-step draft.
type Step = (&'static str, &'static str, &'static str, i32);

const DRAFT_1: &[Step] = &[
    ("ban1", "blue", "Zed", 1),
    ("ban1", "red", "Fiora", 2),
    ("ban1", "blue", "Khazix", 3),
    ("ban1", "red", "Lux", 4),
    ("ban1", "blue", "Tristana", 5),
    ("ban1", "red", "Lulu", 6),
    ("pick1", "blue", "Darius", 7),
    ("pick1", "red", "Gnar", 8),
    ("pick1", "red", "Viktor", 9),
    ("pick1", "blue", "Azir", 10),
    ("pick1", "blue", "Jinx", 11),
    ("pick1", "red", "Ezreal", 12),
    ("ban2", "red", "Orianna", 13),
    ("ban2", "blue", "Sejuani", 14),
    ("ban2", "red", "Jhin", 15),
    ("ban2", "blue", "Rakan", 16),
    ("pick2", "red", "Braum", 17),
    ("pick2", "blue", "Thresh", 18),
    ("pick2", "blue", "Hecarim", 19),
    ("pick2", "red", "LeeSin", 20),
];

const DRAFT_2: &[Step] = &[
    ("ban1", "blue", "Aatrox", 1),
    ("ban1", "red", "Hecarim", 2),
    ("ban1", "blue", "Aphelios", 3),
    ("ban1", "red", "Jinx", 4),
    ("ban1", "blue", "Soraka", 5),
    ("ban1", "red", "Thresh", 6),
    ("pick1", "blue", "Camille", 7),
    ("pick1", "red", "Renekton", 8),
    ("pick1", "red", "Ahri", 9),
    ("pick1", "blue", "Orianna", 10),
    ("pick1", "blue", "Caitlyn", 11),
    ("pick1", "red", "Kaisa", 12),
    ("ban2", "red", "Zed", 13),
    ("ban2", "blue", "Viego", 14),
    ("ban2", "red", "Leona", 15),
    ("ban2", "blue", "Nami", 16),
    ("pick2", "red", "Alistar", 17),
    ("pick2", "blue", "Lulu", 18),
    ("pick2", "blue", "Vi", 19),
    ("pick2", "red", "Elise", 20),
];

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,lol_team_companion=info".parse().unwrap()),
        )
        .init();

    let dry_run = env::args().any(|a| a == "--dry-run");
    let link_riot = env::args().any(|a| a == "--link-riot");
    let data_dir = env::var("SURREAL_DATA_DIR").unwrap_or_else(|_| "./data".to_string());

    if dry_run {
        println!("[dry run] Would open {data_dir}, remove earlier seed data and create:");
        for t in &TEAMS {
            let logins: Vec<String> = t
                .players
                .iter()
                .map(|p| format!("{}@example.com", p.login))
                .collect();
            println!(
                "  team {} ({}) with {}",
                t.name,
                t.region,
                logins.join(", ")
            );
        }
        println!(
            "  per team: 2 drafts, 2 game plans, 3 reviews with action items, a team note, \
             a scouting profile of the other team"
        );
        println!(
            "  2 bug reports (one triaged and on the bug board, one new) and votes on 3 seeded \
             What next ideas (ideas stay unpublished until an admin publishes them; no ledger figures)"
        );
        println!(
            "  a fictional demo game {DEMO_MATCH_ID} (scoreboards and timeline, never looked up on Riot), \
             in seed-alpha-top's games"
        );
        println!("  guidance demo: seed-beta-sup is a returning user (account 40 days old, writes 30 days old)");
        println!("  1 invite code (25 uses, 30 days)");
        return;
    }

    println!("Seed: opening {data_dir} ...");
    let db = match db::init_db(&data_dir).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Could not open the database at {data_dir}: {e}");
            eprintln!(
                "If the server (cargo leptos watch) is running, stop it and wait for it to exit: \
                 SurrealKV allows one process per data directory."
            );
            std::process::exit(1);
        }
    };

    if let Err(e) = cleanup(&db).await {
        eprintln!("Cleanup of earlier seed data failed: {e}");
        std::process::exit(1);
    }

    let riot = RiotClient::from_env();
    let password_hash = hash_password(PASSWORD).expect("hash_password failed");

    let mut teams = Vec::new();
    for spec in &TEAMS {
        match create_team(&db, spec, &password_hash, &riot, link_riot).await {
            Ok(seeded) => teams.push(seeded),
            Err(e) => {
                eprintln!("Seeding {} failed: {e}", spec.name);
                std::process::exit(1);
            }
        }
    }

    for (i, team) in teams.iter().enumerate() {
        if let Err(e) = populate(&db, team, &TEAMS[i], &TEAMS[1 - i]).await {
            eprintln!("Populating {} failed: {e}", team.name);
            std::process::exit(1);
        }
    }

    if let Err(e) = community(&db, &teams).await {
        eprintln!("Community demo data failed: {e}");
        std::process::exit(1);
    }

    if let Err(e) = demo_match(&db, &teams).await {
        eprintln!("The demo match failed: {e}");
        std::process::exit(1);
    }

    if let Err(e) = returning_user(&db, &teams).await {
        eprintln!("Backdating the returning demo user failed: {e}");
        std::process::exit(1);
    }

    let invite_code = db::generate_invite_code();
    let invite = db::create_invite(
        &db,
        None,
        invite_code.clone(),
        25,
        Some(chrono::Utc::now() + chrono::Duration::days(30)),
        Some("seed".to_string()),
    )
    .await;

    println!("Demo match page (as seed-alpha-top): /match/{DEMO_MATCH_ID}");
    println!("\nSeed complete. Logins (password for all: {PASSWORD}):");
    for (spec, team) in TEAMS.iter().zip(&teams) {
        println!("  {}:", team.name);
        for (i, p) in spec.players.iter().enumerate() {
            let lead = match (i, spec.name == TEAMS[1].name) {
                (0, _) => ", team leader",
                (4, true) => ", returning user: 40 days old, last write 30 days ago",
                _ => "",
            };
            println!(
                "    {}@example.com  ({}, {}{lead})",
                p.login, p.username, ROLES[i]
            );
        }
    }
    match invite {
        Ok(_) => println!(
            "Invite code (25 uses, 30 days): {invite_code}\n  Register at /auth/register?invite={invite_code}"
        ),
        Err(e) => eprintln!("Warning: could not create the seed invite code: {e}"),
    }
}

// ---------------------------------------------------------------------------
// Community loop (M7)
// ---------------------------------------------------------------------------

/// Two bug reports by seed players (one triaged and on the board, one new)
/// and votes on three seeded proposals. Proposals and surveys come from the
/// migration and are never touched; no ledger figure is ever seeded.
/// Guidance demo (M9): the second team's support player is a "returning"
/// user. The account is 40 days old and everything it wrote is 30 days
/// old, so Match Day shows "Picking up again?" and the account sees the
/// new-feature introductions dated after it was created. The other seeded
/// accounts are new and get the welcome.
async fn returning_user(db: &Surreal<Db>, teams: &[SeededTeam]) -> Result<(), String> {
    let Some(user) = teams.get(1).and_then(|t| t.members.get(4)) else {
        return Ok(());
    };
    let q = "LET $u = type::record('user', $uk); \
        UPDATE $u SET created_at = time::now() - 40d; \
        UPDATE draft SET created_at = time::now() - 30d WHERE created_by = $u; \
        UPDATE game_plan SET created_at = time::now() - 30d WHERE created_by = $u; \
        UPDATE post_game_learning SET created_at = time::now() - 30d WHERE created_by = $u; \
        UPDATE draft_tree SET created_at = time::now() - 30d WHERE created_by = $u; \
        UPDATE team_note SET created_at = time::now() - 30d WHERE author = $u; \
        UPDATE scheduled_match SET updated_at = time::now() - 30d WHERE created_by = $u; \
        UPDATE series SET created_at = time::now() - 30d WHERE created_by = $u; \
        UPDATE team_relation SET created_at = time::now() - 30d WHERE author = $u; \
        UPDATE personal_learning SET created_at = time::now() - 30d WHERE user = $u; \
        UPDATE champion_pool SET created_at = time::now() - 30d WHERE user = $u; \
        UPDATE champion_note SET created_at = time::now() - 30d WHERE user = $u;";
    db.query(q)
        .bind(("uk", db::record_key(user, "user")))
        .await
        .map_err(|e| e.to_string())?
        .check()
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn community(db: &Surreal<Db>, teams: &[SeededTeam]) -> Result<(), String> {
    use lol_team_companion::models::bug_report::BugTriage;
    use lol_team_companion::server::community::store;

    let reporter = &teams[0].members[2];
    let other = &teams[1].members[4];
    db::create_bug_report(
        db,
        reporter,
        "/draft".into(),
        "Draft".into(),
        "The pick order resets when I switch tabs during a draft.".into(),
        "bug".into(),
        Some(1280),
        Some(800),
    )
    .await
    .map_err(|e| e.to_string())?;
    db::create_bug_report(
        db,
        other,
        "/game-plan".into(),
        "The whole page".into(),
        "A printable version of the game plan for the coach.".into(),
        "wishlist".into(),
        Some(390),
        Some(844),
    )
    .await
    .map_err(|e| e.to_string())?;
    let mine = store::list_my_bug_reports(db, reporter)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(r) = mine.first() {
        store::admin_update_bug_report(
            db,
            &teams[0].members[0],
            &BugTriage {
                id: r.id.clone(),
                status: "planned".into(),
                on_board: true,
                public_title: "Draft pick order resets after switching tabs".into(),
                public_note: "We can reproduce it; a fix is planned.".into(),
                admin_note: "Seed demo report.".into(),
                fixed_in: String::new(),
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    }
    #[derive(serde::Deserialize, SurrealValue)]
    struct Row {
        id: surrealdb::types::RecordId,
    }
    let mut r = db
        .query("SELECT id FROM proposal WHERE seed_key IN ['series_fearless', 'opponent_memory', 'discord']")
        .await
        .map_err(|e| e.to_string())?;
    let ids: Vec<Row> = r.take(0).map_err(|e| e.to_string())?;
    let voters: Vec<&String> = teams.iter().flat_map(|t| t.members.iter()).collect();
    for (i, p) in ids.iter().enumerate() {
        for u in voters.iter().take(6 - i * 2) {
            db.query("CREATE proposal_vote SET proposal = $p, user = type::record('user', $u)")
                .bind(("p", p.id.clone()))
                .bind(("u", db::record_key(u, "user")))
                .await
                .and_then(|r| r.check())
                .map_err(|e| e.to_string())?;
        }
    }
    println!(
        "Created 2 bug reports and votes on {} seeded ideas.",
        ids.len()
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Demo match (M141)
// ---------------------------------------------------------------------------

/// One fictional game between the two seed teams, so the match page has a
/// scoreboard and a timeline without Riot: Northgate Academy on blue (the
/// first draft's picks, seed-alpha-top on Darius) beats Riverside Rooks.
/// Stored like a cached Riot game (`match_detail` with its timeline rows)
/// plus every Northgate member's synced row, under an id no Riot platform uses
/// (`SEED1_...`; the match page never sends it to Riot). Cleanup removes it.
async fn demo_match(db: &Surreal<Db>, teams: &[SeededTeam]) -> Result<(), String> {
    let err = |e: db::DbError| e.to_string();
    let Some(alpha) = teams.first() else {
        return Ok(());
    };
    if alpha.members.is_empty() {
        return Ok(());
    }
    const DURATION: i32 = 29 * 60 + 14;
    // (name, champion, position, k, d, a, cs, vision, damage, gold, items)
    type Line = (
        &'static str,
        &'static str,
        &'static str,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        [i32; 6],
    );
    // (k, d, a, cs, vision, damage, gold, items)
    type Stats = (i32, i32, i32, i32, i32, i32, i32, [i32; 6]);
    let positions = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];
    let blue_champs = ["Darius", "Hecarim", "Azir", "Jinx", "Thresh"];
    let red_champs = ["Gnar", "LeeSin", "Viktor", "Ezreal", "Braum"];
    let blue_stats: [Stats; 5] = [
        (
            7,
            3,
            6,
            214,
            21,
            24_310,
            13_420,
            [3071, 3053, 3047, 6333, 0, 0],
        ),
        (
            5,
            4,
            12,
            168,
            34,
            15_880,
            11_250,
            [3742, 3065, 3111, 3076, 0, 0],
        ),
        (
            6,
            2,
            9,
            256,
            26,
            28_940,
            13_980,
            [3115, 3020, 4645, 3089, 0, 0],
        ),
        (
            9,
            3,
            7,
            271,
            19,
            26_105,
            14_320,
            [3031, 3006, 3085, 3094, 3036, 0],
        ),
        (0, 4, 18, 41, 72, 6_420, 8_130, [3190, 3117, 3109, 0, 0, 0]),
    ];
    let red_stats: [Stats; 5] = [
        (
            3,
            5,
            4,
            198,
            18,
            17_240,
            10_870,
            [3078, 3047, 3748, 0, 0, 0],
        ),
        (4, 6, 5, 151, 29, 13_120, 9_640, [6692, 3111, 3071, 0, 0, 0]),
        (
            2,
            6,
            6,
            239,
            22,
            21_480,
            11_210,
            [3165, 3020, 3157, 0, 0, 0],
        ),
        (
            5,
            5,
            3,
            248,
            17,
            19_930,
            11_660,
            [3078, 3158, 3042, 0, 0, 0],
        ),
        (0, 5, 9, 37, 61, 4_980, 7_020, [3190, 3047, 3109, 0, 0, 0]),
    ];
    let names = |t: usize| {
        TEAMS[t]
            .players
            .iter()
            .map(|p| p.username)
            .collect::<Vec<_>>()
    };
    let mut lines: Vec<(i32, Line)> = Vec::new();
    for (team, champs, stats) in [
        (0usize, blue_champs, blue_stats),
        (1, red_champs, red_stats),
    ] {
        let names = names(team);
        for i in 0..5 {
            let (k, d, a, cs, vis, dmg, gold, items) = stats[i];
            lines.push((
                if team == 0 { 100 } else { 200 },
                (
                    names[i],
                    champs[i],
                    positions[i],
                    k,
                    d,
                    a,
                    cs,
                    vis,
                    dmg,
                    gold,
                    items,
                ),
            ));
        }
    }
    let participants: Vec<MatchParticipant> = lines
        .iter()
        .enumerate()
        .map(|(i, (team_id, l))| MatchParticipant {
            participant_id: i as i32 + 1,
            puuid: format!("seed-demo-{}", i + 1),
            summoner_name: l.0.to_string(),
            champion_name: l.1.to_string(),
            team_id: *team_id,
            team_position: l.2.to_string(),
            kills: l.3,
            deaths: l.4,
            assists: l.5,
            cs: l.6,
            vision_score: l.7,
            damage: l.8,
            gold_earned: l.9,
            items: l.10,
            win: *team_id == 100,
        })
        .collect();

    let ms = |m: i64, s: i64| (m * 60 + s) * 1000;
    let ev = |ts: i64, kind: &str, cat: EventCategory, team: Option<i32>| TimelineEvent {
        timestamp_ms: ts,
        event_type: kind.to_string(),
        category: cat,
        team_id: team,
        killer_participant_id: None,
        victim_participant_id: None,
        monster_type: None,
        monster_sub_type: None,
        building_type: None,
        is_first_blood: false,
        multi_kill_length: None,
        is_teamfight: false,
        involved_participants: Vec::new(),
    };
    let kill = |ts: i64, killer: i32, victim: i32, assists: &[i32], fight: bool| {
        let mut e = ev(
            ts,
            "CHAMPION_KILL",
            EventCategory::Kill,
            Some(if killer <= 5 { 100 } else { 200 }),
        );
        e.killer_participant_id = Some(killer);
        e.victim_participant_id = Some(victim);
        e.involved_participants = [&[killer, victim][..], assists].concat();
        e.is_teamfight = fight;
        if fight {
            e.category = EventCategory::Teamfight;
        }
        e
    };
    let monster = |ts: i64, team: i32, killer: i32, kind: &str, sub: Option<&str>| {
        let mut e = ev(
            ts,
            "ELITE_MONSTER_KILL",
            EventCategory::Objective,
            Some(team),
        );
        e.killer_participant_id = Some(killer);
        e.monster_type = Some(kind.to_string());
        e.monster_sub_type = sub.map(str::to_string);
        e.involved_participants = vec![killer];
        e
    };
    let building = |ts: i64, team: i32, killer: i32, kind: &str| {
        let mut e = ev(ts, "BUILDING_KILL", EventCategory::Tower, Some(team));
        e.killer_participant_id = Some(killer);
        e.building_type = Some(kind.to_string());
        e.involved_participants = vec![killer];
        e
    };
    let mut first = kill(ms(3, 12), 2, 6, &[1], false);
    first.is_first_blood = true;
    let mut fight = ev(ms(18, 24), "TEAMFIGHT", EventCategory::Teamfight, None);
    fight.is_teamfight = true;
    fight.involved_participants = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut events = vec![
        // A stored shop undo, as matches cached before M141 hold them: the
        // read path skips it, so the page draws no recall from it.
        ev(ms(0, 48), "ITEM_UNDO", EventCategory::Recall, None),
        ev(ms(1, 30), "WARD_PLACED", EventCategory::Ward, None),
        first,
        ev(ms(4, 55), "WARD_PLACED", EventCategory::Ward, None),
        monster(ms(6, 2), 200, 7, "HORDE", None),
        kill(ms(6, 40), 9, 4, &[10], false),
        monster(ms(7, 5), 100, 2, "DRAGON", Some("FIRE_DRAGON")),
        ev(ms(9, 40), "WARD_PLACED", EventCategory::Ward, None),
        building(ms(11, 30), 100, 1, "TOWER_BUILDING"),
        kill(ms(12, 5), 1, 6, &[2], false),
        monster(ms(13, 32), 200, 7, "DRAGON", Some("WATER_DRAGON")),
        kill(ms(14, 10), 3, 8, &[5], false),
        ev(ms(14, 20), "WARD_PLACED", EventCategory::Ward, None),
        kill(ms(16, 2), 7, 1, &[6], false),
        kill(ms(18, 20), 4, 9, &[5, 3], true),
        kill(ms(18, 23), 1, 6, &[2], true),
        kill(ms(18, 26), 8, 2, &[7], true),
        kill(ms(18, 29), 3, 7, &[4, 5], true),
        fight,
        monster(ms(19, 10), 100, 2, "DRAGON", Some("AIR_DRAGON")),
        building(ms(20, 4), 100, 4, "TOWER_BUILDING"),
        ev(ms(21, 0), "WARD_PLACED", EventCategory::Ward, None),
        kill(ms(22, 45), 1, 10, &[], false),
        monster(ms(24, 30), 100, 2, "BARON_NASHOR", None),
        building(ms(25, 40), 100, 3, "TOWER_BUILDING"),
        building(ms(27, 10), 100, 1, "INHIBITOR_BUILDING"),
        kill(ms(28, 30), 4, 8, &[1, 3], false),
    ];
    events.sort_by_key(|e| e.timestamp_ms);

    db::store_match_detail(
        db,
        DEMO_MATCH_ID,
        &participants,
        &events,
        DURATION,
        "CLASSIC",
    )
    .await
    .map_err(err)?;
    // Every Northgate member synced the game (M159: a team review lists
    // games three or more current members played together).
    let game_end = Some((chrono::Utc::now() - chrono::Duration::days(1)).timestamp_millis());
    for (member, me) in alpha.members.iter().zip(participants.iter()) {
        db::store_matches(
            db,
            member,
            vec![riot::MatchData {
                match_id: DEMO_MATCH_ID.to_string(),
                queue_id: 420,
                game_duration: DURATION,
                game_end_epoch_ms: game_end,
                champion: me.champion_name.clone(),
                kills: me.kills,
                deaths: me.deaths,
                assists: me.assists,
                cs: me.cs,
                vision_score: me.vision_score,
                damage: me.damage,
                win: me.win,
            }],
        )
        .await
        .map_err(err)?;
    }
    // The game is the team's too (Team games, the post-game review's match list).
    let response = db
        .query(
            "UPDATE match SET team_id = type::record('team', $team_key) WHERE match_id = $match_id",
        )
        .bind(("team_key", db::record_key(&alpha.id, "team")))
        .bind(("match_id", DEMO_MATCH_ID.to_string()))
        .await
        .map_err(|e| e.to_string())?;
    response.check().map_err(|e| e.to_string())?;
    println!("Stored the demo match {DEMO_MATCH_ID}.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Cleanup
// ---------------------------------------------------------------------------

/// Remove everything an earlier run created, in one transaction: seed
/// accounts, the teams they created and every record those teams or
/// accounts own, and seed invites.
async fn cleanup(db: &Surreal<Db>) -> Result<(), String> {
    let q = "BEGIN TRANSACTION; \
        LET $users = (SELECT VALUE id FROM user WHERE string::starts_with(email, 'seed-') AND string::ends_with(email, '@example.com')); \
        LET $teams = (SELECT VALUE id FROM team WHERE created_by IN $users); \
        LET $drafts = (SELECT VALUE id FROM draft WHERE team IN $teams); \
        LET $trees = (SELECT VALUE id FROM draft_tree WHERE team IN $teams); \
        LET $nodes = (SELECT VALUE id FROM draft_tree_node WHERE tree IN $trees); \
        LET $opponents = (SELECT VALUE id FROM opponent WHERE team IN $teams); \
        LET $replays = (SELECT VALUE id FROM replay WHERE team IN $teams); \
        LET $boards = (SELECT VALUE id FROM board WHERE team IN $teams); \
        DELETE draft_action WHERE draft IN $drafts; \
        DELETE draft WHERE id IN $drafts; \
        DELETE tree_node_action WHERE node IN $nodes; \
        DELETE draft_tree_node WHERE id IN $nodes; \
        DELETE draft_tree WHERE id IN $trees; \
        DELETE opponent_player WHERE opponent IN $opponents; \
        DELETE opponent WHERE id IN $opponents; \
        DELETE replay_frame WHERE replay IN $replays; \
        DELETE replay_event WHERE replay IN $replays; \
        DELETE replay WHERE id IN $replays; \
        DELETE board_version WHERE board IN $boards; \
        DELETE board WHERE id IN $boards; \
        DELETE game_plan WHERE team IN $teams; \
        DELETE post_game_learning WHERE team IN $teams; \
        DELETE action_item WHERE team IN $teams; \
        DELETE team_note WHERE team IN $teams; \
        DELETE team_relation WHERE team IN $teams; \
        DELETE series WHERE team IN $teams; \
        DELETE ban_priority WHERE team IN $teams; \
        DELETE checklist_template WHERE team IN $teams; \
        DELETE checklist_instance WHERE team IN $teams; \
        DELETE scheduled_match WHERE team IN $teams; \
        DELETE share_link WHERE team IN $teams; \
        UPDATE match SET team_id = NONE WHERE team_id IN $teams; \
        DELETE join_request WHERE team IN $teams OR user IN $users; \
        DELETE team_member WHERE team IN $teams OR user IN $users; \
        DELETE team WHERE id IN $teams; \
        DELETE champion_pool WHERE user IN $users; \
        DELETE champion_note WHERE user IN $users; \
        DELETE personal_learning WHERE user IN $users; \
        DELETE personal_goal WHERE user IN $users; \
        DELETE ranked_snapshot WHERE user IN $users; \
        DELETE player_match WHERE user IN $users; \
        DELETE proposal_vote WHERE user IN $users; \
        DELETE survey_prompt WHERE user IN $users; \
        DELETE user_guidance WHERE user IN $users; \
        DELETE bug_report WHERE user IN $users; \
        DELETE user WHERE id IN $users; \
        DELETE invite WHERE note = 'seed'; \
        DELETE match_timeline_event WHERE string::starts_with(match_id, $demo_prefix); \
        DELETE match_detail WHERE string::starts_with(match_id, $demo_prefix); \
        DELETE match WHERE string::starts_with(match_id, $demo_prefix); \
        COMMIT TRANSACTION;";
    let response = db
        .query(q)
        .bind(("demo_prefix", DEMO_MATCH_PREFIX.to_string()))
        .await
        .map_err(|e| e.to_string())?;
    db::check_tx(response).map_err(|e| e.to_string())?;
    println!("Removed earlier seed data (if any).");
    Ok(())
}

// ---------------------------------------------------------------------------
// Teams and players
// ---------------------------------------------------------------------------

struct SeededTeam {
    name: &'static str,
    id: String,
    /// User IDs in ROLES order; the first is the leader.
    members: Vec<String>,
}

async fn create_user(db: &Surreal<Db>, p: &Player, password_hash: &str) -> Result<String, String> {
    let email = format!("{}@example.com", p.login);
    match db::create_user(db, p.username.into(), email.clone(), password_hash.into()).await {
        Ok(id) => Ok(id),
        // A real account may already use this name; keep the seed going.
        Err(db::DbError::Invalid(msg)) if msg.contains("username is taken") => {
            let fallback = format!("{} (demo)", p.username);
            db::create_user(db, fallback, email, password_hash.into())
                .await
                .map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}

async fn create_team(
    db: &Surreal<Db>,
    spec: &TeamSpec,
    password_hash: &str,
    riot: &RiotClient,
    link_riot: bool,
) -> Result<SeededTeam, String> {
    let mut members = Vec::new();
    for p in &spec.players {
        let id = create_user(db, p, password_hash).await?;
        db::set_user_region(db, &id, spec.region)
            .await
            .map_err(|e| e.to_string())?;
        // Seed players are fictional; only link Riot IDs when asked to.
        if link_riot && riot.is_configured() {
            tokio::time::sleep(Duration::from_millis(150)).await;
            let platform = riot::platform_for(&[Some(spec.region)]);
            match riot.get_puuid(p.riot_name, spec.region, platform).await {
                Ok(puuid) => {
                    let riot_id = format!("{}#{}", p.riot_name, spec.region);
                    if let Err(e) = db::update_user_riot(db, id.clone(), puuid, riot_id).await {
                        eprintln!("  Warning: could not link {}: {e}", p.riot_name);
                    }
                }
                Err(e) => eprintln!("  Warning: Riot lookup for {} failed: {e}", p.riot_name),
            }
        }
        members.push(id);
    }

    let team_id = db::create_team(db, &members[0], spec.name.into(), spec.region.into())
        .await
        .map_err(|e| e.to_string())?;
    for user_id in members.iter().skip(1) {
        db::join_team(db, user_id, &team_id)
            .await
            .map_err(|e| e.to_string())?;
    }
    for (user_id, role) in members.iter().zip(ROLES) {
        db::assign_to_slot(db, &team_id, user_id, role)
            .await
            .map_err(|e| e.to_string())?;
    }
    println!("Created team {} with {} players.", spec.name, members.len());
    Ok(SeededTeam {
        name: spec.name,
        id: team_id,
        members,
    })
}

// ---------------------------------------------------------------------------
// Team data
// ---------------------------------------------------------------------------

fn actions(steps: &[Step]) -> Vec<DraftAction> {
    steps
        .iter()
        .map(|(phase, side, champion, order)| DraftAction {
            id: None,
            draft_id: String::new(),
            phase: phase.to_string(),
            side: side.to_string(),
            champion: champion.to_string(),
            order: *order,
            comment: None,
            role: None,
            role_guessed: false,
        })
        .collect()
}

fn picks(steps: &[Step], side: &str) -> Vec<String> {
    steps
        .iter()
        .filter(|(phase, s, _, _)| phase.starts_with("pick") && *s == side)
        .map(|(_, _, c, _)| c.to_string())
        .collect()
}

/// (review linked to (plan, draft), result, rating, went well, improvements)
type ReviewSpec = (
    Option<(String, String)>,
    &'static str,
    u8,
    Vec<String>,
    Vec<String>,
);

async fn populate(
    db: &Surreal<Db>,
    team: &SeededTeam,
    spec: &TeamSpec,
    rival: &TeamSpec,
) -> Result<(), String> {
    let err = |e: db::DbError| e.to_string();
    let leader = &team.members[0];

    // Champion pools and two notes per player.
    for (i, user_id) in team.members.iter().enumerate() {
        let role = ROLES[i];
        let pool = spec.pools[i];
        for champ in pool {
            db::add_to_champion_pool(db, user_id, champ.to_string(), role.to_string())
                .await
                .map_err(err)?;
        }
        for (j, champ) in pool.iter().take(3).enumerate() {
            // The pool page's tiers (the first three; the rest stay at
            // the default "comfort" until moved).
            let tier = ["comfort", "match_ready", "scrim_ready"][j];
            db::update_champion_tier(db, user_id, champ, role, tier.to_string())
                .await
                .map_err(err)?;
            db::update_champion_comfort(db, user_id, champ, role, Some(5 - j as i64))
                .await
                .map_err(err)?;
        }
        let notes = [
            (
                "power_spike",
                pool[0],
                "Level 6 spike",
                format!("{} wants the first all-in right after level 6.", pool[0]),
            ),
            (
                "matchup",
                pool[1],
                "Into poke lanes",
                format!(
                    "{} struggles against long-range poke. Play for jungle help early.",
                    pool[1]
                ),
            ),
        ];
        for (note_type, champ, title, content) in notes {
            db::add_champion_note(
                db,
                user_id,
                ChampionNote {
                    id: None,
                    user_id: user_id.clone(),
                    champion: champ.to_string(),
                    role: role.to_string(),
                    note_type: note_type.to_string(),
                    title: title.to_string(),
                    content,
                    difficulty: Some(3),
                    created_at: None,
                },
            )
            .await
            .map_err(err)?;
        }
    }

    // Two drafts against the rival, each with a linked game plan.
    let mut plans = Vec::new();
    for (n, steps) in [(1, DRAFT_1), (2, DRAFT_2)] {
        let tag = if n == 1 { "teamfight" } else { "split-push" };
        let draft_id = db::save_draft(
            db,
            &team.id,
            leader,
            format!("vs {} - Game {n}", rival.name),
            Some(rival.name.to_string()),
            Some("Scrim block, best of two.".to_string()),
            vec![],
            actions(steps),
            None,
            "blue".to_string(),
            vec![tag.to_string()],
            Some(if n == 1 {
                "Group for dragons and fight on our power spikes".to_string()
            } else {
                "Side-lane pressure, then collapse on Baron".to_string()
            }),
            None,
            None,
            None,
            None,
        )
        .await
        .map_err(err)?;

        let plan = GamePlan {
            scheduled_match_id: None,
            game_number: None,
            scheduled_match_deleted: false,
            draft_deleted: false,
            id: None,
            team_id: team.id.clone(),
            draft_id: Some(draft_id.clone()),
            name: format!("vs {} - Game {n} plan", rival.name),
            our_champions: picks(steps, "blue"),
            enemy_champions: picks(steps, "red"),
            win_conditions: if n == 1 {
                vec![
                    "Win the dragon fights at 5 and 11 minutes".into(),
                    "Keep the carry alive in teamfights".into(),
                ]
            } else {
                vec![
                    "Top lane pulls two players to the side".into(),
                    "Trade objectives instead of contesting".into(),
                ]
            },
            objective_priority: vec!["Dragon".into(), "Rift Herald".into(), "Baron".into()],
            teamfight_strategy: if n == 1 {
                "Front to back; engage only with the jungler in range".into()
            } else {
                "Pick fights in side-lane vision, avoid 5v5 in mid".into()
            },
            early_game: Some("Ward the enemy raptors by 1:20; first recall on cannon wave.".into()),
            top_strategy: Some("Hold the wave near our tower until jungle arrives.".into()),
            jungle_strategy: Some("Full clear, then path toward bot for the dragon.".into()),
            mid_strategy: Some("Shove and move to river on enemy jungler sightings.".into()),
            bot_strategy: Some("Play for level 2 and freeze when ahead.".into()),
            support_strategy: Some("Roam mid after first back if bot is safe.".into()),
            notes: Some("Review the first 10 minutes together after the game.".into()),
            win_condition_tag: Some(tag.into()),
        };
        let plan_id = db::save_game_plan(db, plan, leader).await.map_err(err)?;
        plans.push((plan_id, draft_id));
    }

    // Three reviews: two linked to the plans, one standalone scrim review.
    // Improvements become action items (a repeated one only once).
    let reviews: [ReviewSpec; 3] = [
        (
            Some(plans[0].clone()),
            "win",
            4,
            vec![
                "Dragon setup was on time".into(),
                "Clean engage at 22 minutes".into(),
            ],
            vec![
                "Ward Baron pit before it spawns".into(),
                "Mid tempo after the first roam".into(),
            ],
        ),
        (
            Some(plans[1].clone()),
            "loss",
            3,
            vec!["Top drew two players repeatedly".into()],
            vec![
                "Teleport timing for side-lane fights".into(),
                "Support positioning in lane".into(),
            ],
        ),
        (
            None,
            "win",
            5,
            vec!["Strong early coordination".into()],
            vec![
                "Close the game when ahead".into(),
                "Ward Baron pit before it spawns".into(),
            ],
        ),
    ];
    for (link, result, rating, went_well, improvements) in reviews {
        db::create_review_with_action_items(
            db,
            PostGameLearning {
                scheduled_match_id: None,
                series_id: None,
                game_number: None,
                scheduled_match_deleted: false,
                draft_deleted: false,
                created_at: None,
                id: None,
                team_id: team.id.clone(),
                match_riot_id: None,
                game_plan_id: link.as_ref().map(|(p, _)| p.clone()),
                draft_id: link.as_ref().map(|(_, d)| d.clone()),
                what_went_well: went_well,
                improvements,
                action_items: vec![],
                open_notes: Some("Discussed in the post-scrim call.".to_string()),
                created_by: leader.clone(),
                win_loss: Some(result.to_string()),
                rating: Some(rating),
            },
        )
        .await
        .map_err(err)?;
    }

    db::create_team_note(
        db,
        &team.id,
        leader,
        spec.players[0].username,
        "Scrims Tuesday and Thursday 19:00. VOD review Sunday.".to_string(),
    )
    .await
    .map_err(err)?;

    // Two team notes on pairs for the Draft Graph.
    let relations: [(&str, &str, RelationKind, u8, &str); 2] = if spec.name == TEAMS[0].name {
        [
            (
                "Jinx",
                "Thresh",
                RelationKind::Synergy,
                3,
                "Lantern gets Jinx out after she resets into the fight",
            ),
            (
                "Akali",
                "Syndra",
                RelationKind::Counter,
                2,
                "Shroud dodges the stun; our mid needs a ward on the flank",
            ),
        ]
    } else {
        [
            (
                "Kaisa",
                "Rakan",
                RelationKind::Synergy,
                3,
                "Rakan's knock-up lets Kai'Sa ult in safely",
            ),
            (
                "Braum",
                "Jinx",
                RelationKind::Counter,
                2,
                "The shield eats Jinx's rockets in every siege",
            ),
        ]
    };
    for (a, b, kind, weight, reason) in relations {
        db::create_team_relation(
            db,
            &team.id,
            leader,
            spec.players[0].username,
            NewRelation {
                a: a.to_string(),
                b: b.to_string(),
                kind,
                weight,
                reason: reason.to_string(),
            },
        )
        .await
        .map_err(err)?;
    }

    // Scouting profile of the rival team.
    let players: Vec<(String, Option<String>)> = ROLES
        .iter()
        .zip(&rival.players)
        .map(|(role, p)| {
            (
                role.to_string(),
                Some(format!("{}#{}", p.riot_name, rival.region)),
            )
        })
        .collect();
    let (opponent_id, _) = db::create_opponent_with_players(
        db,
        &team.id,
        rival.name.to_string(),
        Some("Regular scrim partner. Strong early skirmishes around the jungle.".to_string()),
        players,
    )
    .await
    .map_err(err)?;

    // The next match, for Match Day: a scrim against the rival tomorrow evening.
    let tomorrow = (chrono::Utc::now() + chrono::Duration::days(1))
        .date_naive()
        .and_hms_opt(18, 0, 0)
        .map(|t| t.and_utc().to_rfc3339())
        .unwrap_or_default();
    db::save_scheduled_match(
        db,
        &team.id,
        &team.members[0],
        lol_team_companion::models::match_day::ScheduledMatchInput {
            id: None,
            starts_at: tomorrow,
            kind: "scrim".into(),
            format: "bo3".into(),
            opponent_id: Some(opponent_id),
            notes: Some("Scrim block on their server; lobby opens 15 minutes before.".into()),
        },
    )
    .await
    .map_err(err)?;

    println!("Populated {}.", team.name);
    Ok(())
}
