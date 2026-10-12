// The #[server] macro generates a client stub that does not inherit item-level
// attributes, so clippy::too_many_arguments is allowed for this file: save/update
// server fns take one argument per form field.
#![allow(clippy::too_many_arguments)]
//! The draft page: the tournament draft board with the champion picker
//! and the intel rail beside it, then the plan (name, side, opponent,
//! series, rating, composition, notes), comments, next steps, saved drafts
//! and insights.

use crate::components::champion_picker::ChampionPicker;
use crate::components::draft_board::slot_meta;
use crate::components::ui::a11y::focus_id;
use crate::components::ui::{
    slot_dom_id, AvatarSize, AvatarState, Button, ButtonSize, ButtonVariant, Callout, CalloutTone,
    ChampionAvatar, ConfirmDialog, ConfirmTone, Density, Dialog, DraftBoard, DraftSide, EmptySize,
    EmptyState, Field, FieldSize, PageHeader, PageShell, Role, RoleLabel, RowLink, SectionHeader,
    Segment, SegmentSize, SegmentedControl, Select, SelectOption, SideMarker, SideText, Skeleton,
    SkeletonGroup, SkeletonShape, StatReadout, Switch, TabItem, TabPanel, Table, Tabs, Td,
    TextField, Th, Toast, ToastContext, Tr, TOURNAMENT_ORDER,
};
use crate::models::champion::{
    note_type_label, Champion, ChampionNote, ChampionPoolEntry, ChampionStatSummary,
};
use crate::models::draft::{guess_role_from_tags, BanPriority, Draft, DraftAction};
use crate::models::opponent::Opponent;
use crate::models::series::Series;
use crate::models::team::Team;
use crate::pages::draft_graph::{DraftGraphPanel, GraphInputs, GraphPick};
use crate::pages::game_plan::check_draft_has_game_plan;
use crate::pages::navigate::{go, navigator, replace_url};
use leptos::prelude::*;
use std::collections::HashMap;

#[server]
pub async fn get_champions() -> Result<Vec<Champion>, ServerFnError> {
    use crate::server::authz;
    use crate::server::data_dragon;
    data_dragon::champions().await.map_err(authz::server_err)
}

#[server]
pub async fn save_draft(
    name: String,
    opponent: Option<String>,
    team_id: Option<String>,
    actions_json: String,
    comments_json: String,
    rating: Option<String>,
    our_side: Option<String>,
    tags_json: String,
    win_conditions: Option<String>,
    watch_out: Option<String>,
    series_id: Option<String>,
    game_number: Option<i32>,
) -> Result<String, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    let actions: Vec<DraftAction> =
        serde_json::from_str(&actions_json).map_err(authz::server_err)?;
    let comments: Vec<String> = serde_json::from_str(&comments_json).map_err(authz::server_err)?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(authz::server_err)?;

    // The team is always the caller's own; a client-sent team_id is only
    // accepted when it matches.
    let resolved_team_id = authz::current_team_for(&db, &user)
        .await?
        .ok_or_else(|| ServerFnError::new("You must be in a team to create a draft"))?;
    if let Some(tid) = team_id.filter(|s| !s.is_empty()) {
        if db::record_key(&tid, "team") != db::record_key(&resolved_team_id, "team") {
            return Err(ServerFnError::new(authz::NOT_FOUND));
        }
    }
    let series_id = series_id.filter(|s| !s.is_empty());
    if let Some(sid) = &series_id {
        db::ensure_team_record(&db, "series", sid, &resolved_team_id)
            .await
            .map_err(authz::server_err)?;
    }

    let draft_id = db::save_draft(
        &db,
        &resolved_team_id,
        &user.id,
        name,
        opponent,
        None,
        comments,
        actions,
        rating,
        our_side.unwrap_or_else(|| "blue".to_string()),
        tags,
        win_conditions,
        watch_out,
        series_id,
        game_number,
    )
    .await
    .map_err(authz::server_err)?;
    // A live survey may ask about drafting after the 3rd draft (M7).
    crate::server::community::store::note_draft_saved(&db, &user.id).await;
    Ok(draft_id)
}

#[server]
pub async fn update_draft(
    draft_id: String,
    name: String,
    opponent: Option<String>,
    actions_json: String,
    comments_json: String,
    rating: Option<String>,
    our_side: Option<String>,
    tags_json: String,
    win_conditions: Option<String>,
    watch_out: Option<String>,
    series_id: Option<String>,
    game_number: Option<i32>,
) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    let team_id = authz::require_team(&db, &user).await?;

    let actions: Vec<DraftAction> =
        serde_json::from_str(&actions_json).map_err(authz::server_err)?;
    let comments: Vec<String> = serde_json::from_str(&comments_json).map_err(authz::server_err)?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(authz::server_err)?;
    let series_id = series_id.filter(|s| !s.is_empty());
    if let Some(sid) = &series_id {
        db::ensure_team_record(&db, "series", sid, &team_id)
            .await
            .map_err(authz::server_err)?;
    }

    db::update_draft(
        &db,
        &team_id,
        &draft_id,
        name,
        opponent,
        None,
        comments,
        actions,
        rating,
        our_side.unwrap_or_else(|| "blue".to_string()),
        tags,
        win_conditions,
        watch_out,
        series_id,
        game_number,
    )
    .await
    .map_err(authz::server_err)
}

/// Delete one of the team's drafts (and its actions).
#[server]
pub async fn delete_draft_fn(draft_id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    let team_id = authz::require_team(&db, &user).await?;
    db::delete_draft(&db, &team_id, &draft_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn list_drafts() -> Result<Vec<Draft>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_drafts(&db, &team_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn list_user_teams() -> Result<Vec<Team>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    db::get_user_teams(&db, &user.id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_ban_priorities() -> Result<Vec<BanPriority>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::get_ban_priorities(&db, &team_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn save_ban_priorities(priorities_json: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    let team_id = db::get_user_team_id(&db, &user.id)
        .await
        .map_err(authz::server_err)?
        .ok_or_else(|| ServerFnError::new(authz::NO_TEAM))?;

    let priorities: Vec<BanPriority> =
        serde_json::from_str(&priorities_json).map_err(authz::server_err)?;

    db::set_ban_priorities(&db, &team_id, priorities)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn list_series_fn() -> Result<Vec<crate::models::series::Series>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_series(&db, &team_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn create_series_fn(
    name: String,
    opponent_name: Option<String>,
    format: String,
    is_fearless: bool,
) -> Result<String, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    let team_id = db::get_user_team_id(&db, &user.id)
        .await
        .map_err(authz::server_err)?
        .ok_or_else(|| ServerFnError::new(authz::NO_TEAM))?;

    db::create_series(
        &db,
        &team_id,
        &user.id,
        name,
        None,
        opponent_name,
        format,
        is_fearless,
        None,
    )
    .await
    .map_err(authz::server_err)
}

#[server]
pub async fn get_fearless_champions(
    series_id: String,
    exclude_draft: Option<String>,
) -> Result<Vec<String>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    let Some(team_id) = authz::current_team_for(&db, &user).await? else {
        return Ok(Vec::new());
    };

    db::get_fearless_used_champions(&db, &team_id, &series_id, exclude_draft.as_deref())
        .await
        .map_err(authz::server_err)
}

/// Get champion pool entries for all starters on the user's team.
/// Returns Vec<(username, role, pool_entries)>.
#[server]
pub async fn get_team_pools() -> Result<
    Vec<(
        String,
        String,
        Vec<crate::models::champion::ChampionPoolEntry>,
    )>,
    ServerFnError,
> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    let _team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    let (_, members) = match db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    let mut result = Vec::new();
    for member in members.iter().filter(|m| m.roster_type == "starter") {
        let pool = db::get_champion_pool(&db, &member.user_id)
            .await
            .unwrap_or_default();
        result.push((member.username.clone(), member.role.clone(), pool));
    }

    Ok(result)
}

/// Get per-champion match stats for all team starters.
/// Returns Vec<(username, Vec<ChampionStatSummary>)>.
#[server]
pub async fn get_team_champion_stats(
) -> Result<Vec<(String, Vec<ChampionStatSummary>)>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    let (_, members) = match db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    let mut result = Vec::new();
    for member in members.iter().filter(|m| m.roster_type == "starter") {
        let stats = db::get_champion_stats_for_user(&db, &member.user_id)
            .await
            .unwrap_or_default();
        result.push((member.username.clone(), stats));
    }

    Ok(result)
}

/// Get opponent players for a given opponent ID.
#[server]
pub async fn get_opponent_intel(
    opponent_id: String,
) -> Result<Vec<crate::models::opponent::OpponentPlayer>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    let Some(team_id) = authz::current_team_for(&db, &user).await? else {
        return Ok(Vec::new());
    };

    match db::get_opponent(&db, &team_id, &opponent_id)
        .await
        .map_err(authz::server_err)?
    {
        Some((_, players)) => Ok(players),
        None => Ok(Vec::new()),
    }
}

/// Get all matchup notes from team members that mention a specific champion.
#[server]
pub async fn get_matchup_notes_for_champion(
    champion: String,
) -> Result<Vec<(String, crate::models::champion::ChampionNote)>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::get_team_matchup_notes(&db, &team_id, &champion)
        .await
        .map_err(authz::server_err)
}

// ---------------------------------------------------------------------------
// Analytics data types + server functions (Phase 7a-c)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DraftTendency {
    pub champion: String,
    pub phase: String,
    pub order: i32,
    pub count: i32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct DraftAnalytics {
    pub blue_games: i32,
    pub blue_wins: i32,
    pub red_games: i32,
    pub red_wins: i32,
    pub tag_stats: Vec<(String, i32, i32)>,
    pub first_pick_stats: Vec<(String, i32, i32)>,
}

#[server]
pub async fn get_draft_tendency_data() -> Result<Vec<DraftTendency>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    let raw = db::get_draft_tendencies(&surreal, &team_id)
        .await
        .map_err(authz::server_err)?;

    Ok(raw
        .into_iter()
        .map(|(champion, phase, order, count)| DraftTendency {
            champion,
            phase,
            order,
            count,
        })
        .collect())
}

#[server]
pub async fn get_draft_analytics() -> Result<DraftAnalytics, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(DraftAnalytics::default()),
    };

    let data = db::get_draft_outcome_stats(&surreal, &team_id)
        .await
        .map_err(authz::server_err)?;

    Ok(DraftAnalytics {
        blue_games: data.blue_games,
        blue_wins: data.blue_wins,
        red_games: data.red_games,
        red_wins: data.red_wins,
        tag_stats: data.tag_stats,
        first_pick_stats: data.first_pick_stats,
    })
}

/// Batch-fetch the number of game plans referencing each draft for the user's team.
/// Returns a list of (draft_id, count) pairs for drafts that have at least one game plan.
#[server]
pub async fn get_draft_game_plan_counts() -> Result<Vec<(String, usize)>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    let plans = db::list_game_plans(&surreal, &team_id)
        .await
        .map_err(authz::server_err)?;

    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for plan in &plans {
        if let Some(ref draft_id) = plan.draft_id {
            *counts.entry(draft_id.clone()).or_default() += 1;
        }
    }

    Ok(counts.into_iter().collect())
}

#[server]
pub async fn get_pool_notes_for_champions(
    champions_json: String,
) -> Result<Vec<(String, Vec<ChampionNote>)>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let champions: Vec<String> =
        serde_json::from_str(&champions_json).map_err(authz::server_err)?;

    let notes = db::get_pool_notes_for_champions(&surreal, &user.id, &champions)
        .await
        .map_err(authz::server_err)?;

    // Group by champion
    let mut map: std::collections::HashMap<String, Vec<ChampionNote>> =
        std::collections::HashMap::new();
    for note in notes {
        map.entry(note.champion.clone()).or_default().push(note);
    }
    // Return in same order as input champions list
    Ok(champions
        .into_iter()
        .map(|c| {
            let ns = map.remove(&c).unwrap_or_default();
            (c, ns)
        })
        .collect())
}

fn build_actions(
    slots: Vec<Option<String>>,
    slot_comments: &[Option<String>],
    roles: &[Option<String>],
    guessed: &[bool],
) -> Vec<DraftAction> {
    slots
        .into_iter()
        .enumerate()
        .filter_map(|(i, opt)| {
            opt.map(|champ| {
                let (side, kind, label) = slot_meta(i);
                DraftAction {
                    id: None,
                    draft_id: String::new(),
                    phase: format!("{}_{}", kind, label),
                    side: side.to_string(),
                    champion: champ,
                    order: i as i32,
                    comment: slot_comments.get(i).cloned().flatten(),
                    role: roles.get(i).cloned().flatten(),
                    role_guessed: guessed.get(i).copied().unwrap_or(false),
                }
            })
        })
        .collect()
}

const COMPOSITION_TAGS: &[&str] = &[
    "teamfight",
    "split-push",
    "poke",
    "pick",
    "scaling",
    "early-game",
    "protect-the-carry",
];

/// Compute pool warning annotations for each of the 20 draft slots.
///
/// Returns a `Vec<Option<(player_name, class_gap_detail)>>` of length 20.
/// `Some((player, detail))` means the slot's champion is not in that player's pool AND
/// exposes a class gap (i.e. coaching-quality insight, not just a binary yes/no).
/// `None` means no warning for that slot.
///
/// # Arguments
/// * `slots` - 20-slot draft array, values are champion display names
/// * `pools` - Team pools: `(username, role, pool_entries)` per player
/// * `champions` - Champion lookup keyed by **display name** (`Champion.name`)
/// * `our_side` - "blue" or "red"
/// * `role_overrides` - Manual slot→role assignment overrides (keyed by slot index)
pub fn compute_slot_warnings(
    slots: &[Option<String>],
    pools: &[(
        String,
        String,
        Vec<crate::models::champion::ChampionPoolEntry>,
    )],
    champions: &HashMap<String, crate::models::champion::Champion>,
    our_side: &str,
    role_overrides: &HashMap<usize, String>,
) -> Vec<Option<(String, String)>> {
    use crate::models::champion::Champion;

    // Build a reverse lookup: canonical Data Dragon ID -> Champion
    // This is needed because pool entries store canonical IDs, but champions map uses display names.
    let id_to_champion: HashMap<&str, &Champion> =
        champions.values().map(|c| (c.id.as_str(), c)).collect();

    // Our-side pick slot indices and their conventional role mapping
    // Draft order: pick1=top, pick2=jungle, pick3=mid, pick4=bot, pick5=support
    let (our_pick_slots, _opp_pick_slots): (&[usize], &[usize]) = if our_side == "blue" {
        (&[6, 9, 10, 17, 18], &[7, 8, 11, 16, 19])
    } else {
        (&[7, 8, 11, 16, 19], &[6, 9, 10, 17, 18])
    };

    // Map pick slot index -> conventional role
    let slot_to_default_role: HashMap<usize, &str> = our_pick_slots
        .iter()
        .zip(["top", "jungle", "mid", "bot", "support"].iter())
        .map(|(&slot, &role)| (slot, role))
        .collect();

    let mut result: Vec<Option<(String, String)>> = vec![None; 20];

    for &slot_idx in our_pick_slots {
        let champ_name = match slots.get(slot_idx).and_then(|s| s.as_ref()) {
            Some(name) => name,
            None => continue, // empty slot, no warning
        };

        // Determine role for this slot
        let role: &str = if let Some(override_role) = role_overrides.get(&slot_idx) {
            override_role.as_str()
        } else {
            match slot_to_default_role.get(&slot_idx) {
                Some(r) => r,
                None => continue,
            }
        };

        // Find the player whose role matches
        let player = pools
            .iter()
            .find(|(_, player_role, _)| normalize_role(player_role) == normalize_role(role));
        let (username, _, pool_entries) = match player {
            Some(p) => p,
            None => continue, // no player for this role
        };

        // Check if champion is in the player's pool
        // Pool entries may use canonical IDs or display names - check both
        let in_pool = pool_entries.iter().any(|entry| {
            entry.champion == *champ_name
                || id_to_champion
                    .get(entry.champion.as_str())
                    .map(|c| c.name == *champ_name)
                    .unwrap_or(false)
        });

        if in_pool {
            // Champion is in pool - no warning
            continue;
        }

        // Champion not in pool - check for class gap
        // Get the picked champion's class tags
        let picked_champ = champions.get(champ_name);
        let picked_tags = match picked_champ {
            Some(c) => &c.tags,
            None => continue, // unknown champion, skip
        };

        if picked_tags.is_empty() {
            continue; // no class info, can't determine gap
        }

        // Collect the classes already covered by the player's pool entries
        let covered_classes: std::collections::HashSet<&str> = pool_entries
            .iter()
            .flat_map(|entry| {
                // Look up by display name first, then by canonical ID
                let champ = champions
                    .get(entry.champion.as_str())
                    .or_else(|| id_to_champion.get(entry.champion.as_str()).copied());
                champ
                    .map(|c| c.tags.iter().map(|t| t.as_str()).collect::<Vec<_>>())
                    .unwrap_or_default()
            })
            .collect();

        // Find first class tag of the picked champion that is NOT covered
        let missing_class = picked_tags
            .iter()
            .find(|tag| !covered_classes.contains(tag.as_str()));

        if let Some(class) = missing_class {
            result[slot_idx] = Some((username.clone(), format!("No {} coverage", class)));
        }
        // If all classes are covered (no gap), no warning despite champion not being in pool
    }

    result
}

/// Normalize role strings for matching (handles aliases like "jng" -> "jungle")
fn normalize_role(role: &str) -> &str {
    match role {
        "jng" | "jungle" => "jungle",
        "middle" | "mid" => "mid",
        "bot" | "adc" => "bot",
        "sup" | "support" => "support",
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Editor state
// ---------------------------------------------------------------------------

/// Rating ladder for a draft. Neutral chips (no accent): a rating is an
/// opinion, not the next action.
const RATINGS: &[&str] = &["S+", "S", "A", "B", "C", "D"];

/// Board and picker DOM id prefixes (the e2e suite uses them too).
const BOARD_ID: &str = "draft-board";
const PICKER_ID: &str = "draft-picker";

/// Everything the editor holds, for undo and for "has this changed".
#[derive(Clone, Debug, PartialEq)]
struct Snapshot {
    slots: Vec<Option<String>>,
    roles: Vec<Option<String>>,
    /// Per slot: the role was guessed, not set by a person.
    guessed: Vec<bool>,
    notes: Vec<Option<String>>,
    name: String,
    opponent: String,
    side: String,
    rating: Option<String>,
    tags: Vec<String>,
    win: String,
    watch: String,
    comments: Vec<String>,
    loaded_id: Option<String>,
    series_id: String,
    game: String,
}

/// The draft being edited. Every field is a signal, so the struct is Copy
/// and every part of the page shares it.
#[derive(Clone, Copy)]
struct Editor {
    slots: RwSignal<Vec<Option<String>>>,
    roles: RwSignal<Vec<Option<String>>>,
    /// Roles guessed from the champion's class (not confirmed by a person).
    guessed: RwSignal<Vec<bool>>,
    notes: RwSignal<Vec<Option<String>>>,
    /// The slot being filled: the one lit thing on the board.
    active: RwSignal<Option<usize>>,
    name: RwSignal<String>,
    /// Opponent id, or the free text of a draft saved before opponents
    /// were records.
    opponent: RwSignal<String>,
    side: RwSignal<String>,
    rating: RwSignal<Option<String>>,
    tags: RwSignal<Vec<String>>,
    win: RwSignal<String>,
    watch: RwSignal<String>,
    comments: RwSignal<Vec<String>>,
    loaded_id: RwSignal<Option<String>>,
    series_id: RwSignal<String>,
    game: RwSignal<String>,
}

impl Editor {
    fn new() -> Self {
        Self {
            slots: RwSignal::new(vec![None; 20]),
            roles: RwSignal::new(vec![None; 20]),
            guessed: RwSignal::new(vec![false; 20]),
            notes: RwSignal::new(vec![None; 20]),
            active: RwSignal::new(Some(0)),
            name: RwSignal::new(String::new()),
            opponent: RwSignal::new(String::new()),
            side: RwSignal::new("blue".to_string()),
            rating: RwSignal::new(None),
            tags: RwSignal::new(Vec::new()),
            win: RwSignal::new(String::new()),
            watch: RwSignal::new(String::new()),
            comments: RwSignal::new(Vec::new()),
            loaded_id: RwSignal::new(None),
            series_id: RwSignal::new(String::new()),
            game: RwSignal::new("1".to_string()),
        }
    }

    /// The current state, read untracked (for event handlers).
    fn snapshot(&self) -> Snapshot {
        Snapshot {
            slots: self.slots.get_untracked(),
            roles: self.roles.get_untracked(),
            guessed: self.guessed.get_untracked(),
            notes: self.notes.get_untracked(),
            name: self.name.get_untracked(),
            opponent: self.opponent.get_untracked(),
            side: self.side.get_untracked(),
            rating: self.rating.get_untracked(),
            tags: self.tags.get_untracked(),
            win: self.win.get_untracked(),
            watch: self.watch.get_untracked(),
            comments: self.comments.get_untracked(),
            loaded_id: self.loaded_id.get_untracked(),
            series_id: self.series_id.get_untracked(),
            game: self.game.get_untracked(),
        }
    }

    /// The current state, tracked (for effects).
    fn snapshot_tracked(&self) -> Snapshot {
        Snapshot {
            slots: self.slots.get(),
            roles: self.roles.get(),
            guessed: self.guessed.get(),
            notes: self.notes.get(),
            name: self.name.get(),
            opponent: self.opponent.get(),
            side: self.side.get(),
            rating: self.rating.get(),
            tags: self.tags.get(),
            win: self.win.get(),
            watch: self.watch.get(),
            comments: self.comments.get(),
            loaded_id: self.loaded_id.get(),
            series_id: self.series_id.get(),
            game: self.game.get(),
        }
    }

    fn restore(&self, s: Snapshot) {
        let next = s.slots.iter().position(|c| c.is_none());
        self.slots.set(s.slots);
        self.roles.set(s.roles);
        self.guessed.set(s.guessed);
        self.notes.set(s.notes);
        self.active.set(next);
        self.name.set(s.name);
        self.opponent.set(s.opponent);
        self.side.set(s.side);
        self.rating.set(s.rating);
        self.tags.set(s.tags);
        self.win.set(s.win);
        self.watch.set(s.watch);
        self.comments.set(s.comments);
        self.loaded_id.set(s.loaded_id);
        self.series_id.set(s.series_id);
        self.game.set(s.game);
    }

    fn blank() -> Snapshot {
        Snapshot {
            slots: vec![None; 20],
            roles: vec![None; 20],
            guessed: vec![false; 20],
            notes: vec![None; 20],
            name: String::new(),
            opponent: String::new(),
            side: "blue".to_string(),
            rating: None,
            tags: Vec::new(),
            win: String::new(),
            watch: String::new(),
            comments: Vec::new(),
            loaded_id: None,
            series_id: String::new(),
            game: "1".to_string(),
        }
    }

    /// The editor state of a saved draft. An opponent stored by name (older
    /// drafts, the seed data) becomes the id of the dossier with that name.
    fn from_draft(d: &Draft, opponents: &[Opponent]) -> Snapshot {
        let mut s = Self::blank();
        for a in &d.actions {
            let o = a.order as usize;
            if o < 20 {
                s.slots[o] = Some(a.champion.clone());
                s.roles[o] = a.role.clone();
                s.guessed[o] = a.role.is_some() && a.role_guessed;
                s.notes[o] = a.comment.clone();
            }
        }
        s.name = d.name.clone();
        s.opponent = opponent_ref(d.opponent.as_deref().unwrap_or_default(), opponents);
        s.side = if d.our_side == "red" { "red" } else { "blue" }.to_string();
        s.rating = d.rating.clone();
        s.tags = d.tags.clone();
        s.win = d.win_conditions.clone().unwrap_or_default();
        s.watch = d.watch_out.clone().unwrap_or_default();
        s.comments = d.comments.clone();
        s.loaded_id = d.id.clone();
        s.series_id = d.series_id.clone().unwrap_or_default();
        s.game = d.game_number.unwrap_or(1).to_string();
        s
    }

    /// Put a champion in a slot (by display name). A champion already on
    /// the board is refused. Picks get a role guessed from the class unless
    /// one is set. The lit slot moves to the next empty one.
    fn fill(
        &self,
        slot: usize,
        champion: String,
        champs: &[Champion],
        trait_roles: &HashMap<String, Vec<Role>>,
    ) -> bool {
        if slot >= 20 {
            return false;
        }
        let taken = self
            .slots
            .with_untracked(|s| s.iter().any(|c| c.as_deref() == Some(champion.as_str())));
        if taken {
            return false;
        }
        self.slots.update(|s| s[slot] = Some(champion.clone()));
        let unset = self.roles.with_untracked(|r| r[slot].is_none())
            || self.guessed.with_untracked(|g| g[slot]);
        if slot_meta(slot).1 == "pick" && unset {
            let guess = self
                .roles
                .with_untracked(|roles| guess_role(slot, &champion, champs, trait_roles, roles));
            if let Some(g) = guess {
                self.roles.update(|r| r[slot] = Some(g));
                self.guessed.update(|g| g[slot] = true);
            }
        }
        let next = self
            .slots
            .with_untracked(|s| s.iter().position(|c| c.is_none()));
        self.active.set(next);
        true
    }

    /// Empty a slot; returns what was there for undo. A guessed role goes
    /// with the champion; a role a person set stays on the slot.
    fn clear(&self, slot: usize) -> Option<Cleared> {
        let champion = self
            .slots
            .with_untracked(|s| s.get(slot).cloned().flatten())?;
        let role = self.roles.with_untracked(|r| r[slot].clone());
        let guessed = self.guessed.with_untracked(|g| g[slot]);
        let note = self.notes.with_untracked(|n| n[slot].clone());
        self.slots.update(|s| s[slot] = None);
        if guessed {
            self.roles.update(|r| r[slot] = None);
            self.guessed.update(|g| g[slot] = false);
        }
        self.notes.update(|n| n[slot] = None);
        self.active.set(Some(slot));
        Some(Cleared {
            champion,
            role,
            guessed,
            note,
        })
    }

    /// Exchange two slots (a drag onto a filled slot swaps; onto an empty
    /// one it moves). Champion and note travel together; between two picks
    /// the role goes along, and a guessed role dropped into a ban is
    /// cleared.
    fn swap(&self, a: usize, b: usize) {
        if a == b || a >= 20 || b >= 20 {
            return;
        }
        self.slots.update(|s| s.swap(a, b));
        self.notes.update(|n| n.swap(a, b));
        let same_kind = slot_meta(a).1 == slot_meta(b).1;
        if same_kind {
            self.roles.update(|r| r.swap(a, b));
            self.guessed.update(|g| g.swap(a, b));
        } else {
            for slot in [a, b] {
                if self.guessed.with_untracked(|g| g[slot]) {
                    self.roles.update(|r| r[slot] = None);
                    self.guessed.update(|g| g[slot] = false);
                }
            }
        }
        let next = self
            .slots
            .with_untracked(|s| s.iter().position(|c| c.is_none()));
        self.active.set(next);
    }

    /// Put back what [`Editor::clear`] took out.
    fn restore_slot(&self, slot: usize, c: Cleared) {
        self.slots.update(|s| s[slot] = Some(c.champion));
        self.roles.update(|r| r[slot] = c.role);
        self.guessed.update(|g| g[slot] = c.guessed);
        self.notes.update(|n| n[slot] = c.note);
        let next = self
            .slots
            .with_untracked(|s| s.iter().position(|c| c.is_none()));
        self.active.set(next);
    }
}

/// What a cleared slot held (for Undo).
#[derive(Clone, Debug)]
struct Cleared {
    champion: String,
    role: Option<String>,
    guessed: bool,
    note: Option<String>,
}

/// A role for a pick nobody has set: the first of the champion's roles in
/// the trait table that its side has not filled yet (else its main role),
/// falling back to a guess from the champion's class. Always a guess.
fn guess_role(
    slot: usize,
    champion: &str,
    champs: &[Champion],
    trait_roles: &HashMap<String, Vec<Role>>,
    roles: &[Option<String>],
) -> Option<String> {
    let side = TOURNAMENT_ORDER.get(slot)?.side;
    let taken: Vec<Role> = crate::components::ui::pick_slots(side)
        .into_iter()
        .filter(|s| *s != slot)
        .filter_map(|s| roles.get(s).cloned().flatten())
        .filter_map(|r| Role::parse(&r))
        .collect();
    let champ = champs
        .iter()
        .find(|c| c.name == champion || c.id == champion);
    let listed = trait_roles
        .get(champion)
        .or_else(|| champ.and_then(|c| trait_roles.get(&c.id)));
    if let Some(list) = listed.filter(|l| !l.is_empty()) {
        let r = list.iter().find(|r| !taken.contains(r)).unwrap_or(&list[0]);
        return Some(r.key().to_string());
    }
    champ.map(|c| guess_role_from_tags(&c.tags).to_string())
}

/// The arguments `save_draft`/`update_draft` take, built from a snapshot.
struct SaveArgs {
    name: String,
    opponent: Option<String>,
    actions_json: String,
    comments_json: String,
    rating: Option<String>,
    side: String,
    tags_json: String,
    win: Option<String>,
    watch: Option<String>,
    series_id: Option<String>,
    game: Option<i32>,
}

fn non_empty(s: String) -> Option<String> {
    (!s.trim().is_empty()).then_some(s)
}

impl SaveArgs {
    fn from(s: &Snapshot) -> Self {
        let actions = build_actions(s.slots.clone(), &s.notes, &s.roles, &s.guessed);
        let series_id = non_empty(s.series_id.clone());
        let game = series_id.as_ref().and_then(|_| s.game.parse::<i32>().ok());
        Self {
            name: s.name.trim().to_string(),
            opponent: non_empty(s.opponent.clone()),
            actions_json: serde_json::to_string(&actions).unwrap_or_default(),
            comments_json: serde_json::to_string(&s.comments).unwrap_or_default(),
            rating: s.rating.clone(),
            side: s.side.clone(),
            tags_json: serde_json::to_string(&s.tags).unwrap_or_default(),
            win: non_empty(s.win.clone()),
            watch: non_empty(s.watch.clone()),
            series_id,
            game,
        }
    }
}

/// Save a snapshot: update when it has an id, else create. Returns the id.
async fn persist(s: Snapshot) -> Result<String, ServerFnError> {
    let a = SaveArgs::from(&s);
    match s.loaded_id {
        Some(id) => update_draft(
            id.clone(),
            a.name,
            a.opponent,
            a.actions_json,
            a.comments_json,
            a.rating,
            Some(a.side),
            a.tags_json,
            a.win,
            a.watch,
            a.series_id,
            a.game,
        )
        .await
        .map(|_| id),
        None => {
            save_draft(
                a.name,
                a.opponent,
                None,
                a.actions_json,
                a.comments_json,
                a.rating,
                Some(a.side),
                a.tags_json,
                a.win,
                a.watch,
                a.series_id,
                a.game,
            )
            .await
        }
    }
}

/// Saved state of the editor, shown in the header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SaveState {
    /// Never saved.
    New,
    /// Saved, then changed.
    Dirty,
    Saving,
    Saved,
}

/// Warnings per slot in words ("Not in Kim's pool, no mage coverage").
fn warning_texts(raw: Vec<Option<(String, String)>>) -> Vec<Option<String>> {
    raw.into_iter()
        .map(|w| {
            w.map(|(player, detail)| format!("Not in {player}'s pool, {}", detail.to_lowercase()))
        })
        .collect()
}

/// Display name of the opponent value (an id from the list, or legacy text).
fn opponent_name(value: &str, list: &[Opponent]) -> Option<String> {
    if value.is_empty() {
        return None;
    }
    Some(
        list.iter()
            .find(|o| o.id.as_deref() == Some(value))
            .map(|o| o.name.clone())
            .unwrap_or_else(|| value.to_string()),
    )
}

/// A stored opponent reference as an opponent id: an id stays, a name that
/// matches exactly one dossier (ignoring case) becomes its id, anything
/// else stays as written (shown as "not in Opponents").
fn opponent_ref(stored: &str, opponents: &[Opponent]) -> String {
    if stored.is_empty() || opponents.iter().any(|o| o.id.as_deref() == Some(stored)) {
        return stored.to_string();
    }
    let mut by_name = opponents
        .iter()
        .filter(|o| o.name.trim().eq_ignore_ascii_case(stored.trim()));
    match (by_name.next(), by_name.next()) {
        (Some(o), None) => o.id.clone().unwrap_or_else(|| stored.to_string()),
        _ => stored.to_string(),
    }
}

/// A scouted player's name: the name, else the Riot ID, else a placeholder.
fn player_name(p: &crate::models::opponent::OpponentPlayer) -> String {
    if !p.name.trim().is_empty() {
        return p.name.clone();
    }
    p.riot_summoner_name
        .clone()
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| "Unnamed player".to_string())
}

/// Shared page plumbing the editor's parts need.
#[derive(Clone, Copy)]
struct Ctx {
    ed: Editor,
    drafts: Resource<Result<Vec<Draft>, ServerFnError>>,
    series: Resource<Result<Vec<Series>, ServerFnError>>,
    plan_counts: Resource<Result<Vec<(String, usize)>, ServerFnError>>,
    save_state: RwSignal<SaveState>,
    last_saved: StoredValue<Option<Snapshot>>,
    url_loaded: StoredValue<Option<String>>,
    fearless_used: RwSignal<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

#[component]
pub fn DraftPage() -> impl IntoView {
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    let ed = Editor::new();
    let champions_resource = Resource::new(|| (), |_| get_champions());
    let teams_resource = Resource::new(|| (), |_| list_user_teams());
    let opponents_list = Resource::new(|| (), |_| crate::pages::opponents::get_opponents());
    let team_pools = Resource::new(|| (), |_| get_team_pools());
    let cx = Ctx {
        ed,
        drafts: Resource::new(|| (), |_| list_drafts()),
        series: Resource::new(|| (), |_| list_series_fn()),
        plan_counts: Resource::new(|| (), |_| get_draft_game_plan_counts()),
        save_state: RwSignal::new(SaveState::New),
        last_saved: StoredValue::new(None),
        url_loaded: StoredValue::new(None),
        fearless_used: RwSignal::new(Vec::new()),
    };

    // Fearless: champions used in the series' other drafts.
    Effect::new(move |_| {
        let id = ed.series_id.get();
        let series = cx
            .series
            .get()
            .and_then(|r| r.ok())
            .and_then(|l| l.into_iter().find(|s| s.id.as_deref() == Some(id.as_str())));
        let exclude = ed.loaded_id.get_untracked();
        match series {
            Some(s) if s.is_fearless => {
                if let Some(sid) = s.id {
                    leptos::task::spawn_local(async move {
                        if let Ok(champs) = get_fearless_champions(sid, exclude).await {
                            cx.fearless_used.set(champs);
                        }
                    });
                }
            }
            _ => cx.fearless_used.set(Vec::new()),
        }
    });

    // Open a draft from the URL (`?draft_id=`), once per value; the saved
    // drafts list links here, so opening one is a normal navigation.
    let query = leptos_router::hooks::use_query_map();
    Effect::new(move |_| {
        let param = query.read().get("draft_id");
        let Some(target) = param else {
            cx.url_loaded.set_value(None);
            return;
        };
        if cx.url_loaded.get_value().as_deref() == Some(target.as_str()) {
            return;
        }
        // Wait for the opponents too: older drafts store the opponent's
        // name, which is matched to its dossier.
        let opps = match opponents_list.get() {
            Some(Ok(o)) => o,
            Some(Err(_)) => Vec::new(),
            None => return,
        };
        if let Some(Ok(list)) = cx.drafts.get() {
            if let Some(d) = list
                .iter()
                .find(|d| d.id.as_deref() == Some(target.as_str()))
            {
                let snap = Editor::from_draft(d, &opps);
                cx.last_saved.set_value(Some(snap.clone()));
                ed.restore(snap);
                cx.url_loaded.set_value(Some(target));
            }
        }
    });

    // Save state and auto-save (existing drafts only, 2 s after the last
    // change). Values are captured eagerly (leptos rule 54).
    #[allow(unused_variables)]
    let auto_save_timer: RwSignal<Option<i32>> = RwSignal::new(None);
    Effect::new(move |_| {
        let snap = ed.snapshot_tracked();
        if snap.loaded_id.is_none() {
            cx.save_state.set(SaveState::New);
            return;
        }
        if cx.last_saved.with_value(|l| l.as_ref() == Some(&snap)) {
            cx.save_state.set(SaveState::Saved);
            return;
        }
        cx.save_state.set(SaveState::Dirty);
        // An unnamed draft is not saved automatically.
        #[cfg(feature = "hydrate")]
        if !snap.name.trim().is_empty() {
            use wasm_bindgen::prelude::*;
            if let Some(id) = auto_save_timer.get_untracked() {
                if let Some(win) = web_sys::window() {
                    win.clear_timeout_with_handle(id);
                }
            }
            let cb = Closure::once(move || {
                leptos::task::spawn_local(async move {
                    if persist(snap.clone()).await.is_ok() {
                        cx.last_saved.set_value(Some(snap.clone()));
                        if ed.snapshot() == snap {
                            cx.save_state.set(SaveState::Saved);
                        }
                        cx.drafts.refetch();
                    }
                });
            });
            if let Some(win) = web_sys::window() {
                if let Ok(id) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    2000,
                ) {
                    auto_save_timer.set(Some(id));
                }
            }
            cb.forget();
        }
    });

    view! {
        <PageShell density=Density::Compact attr:data-feedback-label="Draft">
            <Suspense fallback=|| view! {
                <PageHeader title="Draft" eyebrow="Draft" />
                <SkeletonGroup label="Loading the draft board">
                    <Skeleton shape=SkeletonShape::Block size="h-96 w-full" />
                </SkeletonGroup>
            }>
                {move || Suspend::new(async move {
                    let champs = champions_resource.await.unwrap_or_default();
                    let opponents = opponents_list.await.unwrap_or_default();
                    let team_name = teams_resource
                        .await
                        .ok()
                        .and_then(|t| t.into_iter().next())
                        .map(|t| t.name);
                    let pools = team_pools.await.unwrap_or_default();
                    view! {
                        <DraftEditor
                            cx=cx
                            champs=champs
                            opponents=opponents
                            team_name=team_name
                            pools=pools
                        />
                    }
                })}
            </Suspense>
        </PageShell>
    }
}

/// The editor, once the champion list, opponents, team and pools are in.
#[component]
fn DraftEditor(
    cx: Ctx,
    champs: Vec<Champion>,
    opponents: Vec<Opponent>,
    team_name: Option<String>,
    pools: Vec<(String, String, Vec<ChampionPoolEntry>)>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let nav = navigator();
    let ed = cx.ed;
    let champ_list = champs.clone();
    let champs = StoredValue::new(champs);
    let opponents = StoredValue::new(opponents);
    let pools = StoredValue::new(pools);
    let opp_name =
        Memo::new(move |_| opponents.with_value(|o| opponent_name(&ed.opponent.get(), o)));

    let name_error = RwSignal::new(None::<String>);
    let do_save = Callback::new(move |_: leptos::ev::MouseEvent| {
        let snap = ed.snapshot();
        if snap.name.trim().is_empty() {
            name_error.set(Some(
                "Give the draft a name so you can find it again.".into(),
            ));
            focus_id("draft-name");
            return;
        }
        name_error.set(None);
        cx.save_state.set(SaveState::Saving);
        leptos::task::spawn_local(async move {
            match persist(snap.clone()).await {
                Ok(id) => {
                    let mut saved = snap.clone();
                    saved.loaded_id = Some(id.clone());
                    cx.last_saved.set_value(Some(saved));
                    cx.url_loaded.set_value(Some(id.clone()));
                    ed.loaded_id.set(Some(id.clone()));
                    toast.push.run(Toast::success("Draft saved"));
                    cx.drafts.refetch();
                    replace_url(nav, &format!("/draft?draft_id={id}"));
                }
                Err(e) => {
                    cx.save_state.set(SaveState::Dirty);
                    toast
                        .push
                        .run(Toast::error(format!("Could not save the draft: {e}")));
                }
            }
        });
    });

    let new_draft = Callback::new(move |_: leptos::ev::MouseEvent| {
        let before = ed.snapshot();
        ed.restore(Editor::blank());
        cx.last_saved.set_value(None);
        go(nav, "/draft");
        if before != Editor::blank() {
            let undo = Callback::new(move |_| {
                if let Some(id) = before.loaded_id.clone() {
                    cx.url_loaded.set_value(Some(id));
                }
                ed.restore(before.clone());
            });
            toast
                .push
                .run(Toast::info("Started a new draft").with_action("Undo", undo));
        }
    });

    let confirm_delete = RwSignal::new(false);
    let on_delete = Callback::new(move |_| {
        let Some(id) = ed.loaded_id.get_untracked() else {
            return;
        };
        let name = ed.name.get_untracked();
        leptos::task::spawn_local(async move {
            match delete_draft_fn(id).await {
                Ok(()) => {
                    ed.restore(Editor::blank());
                    cx.last_saved.set_value(None);
                    cx.drafts.refetch();
                    cx.plan_counts.refetch();
                    go(nav, "/draft");
                    toast
                        .push
                        .run(Toast::success(format!("Deleted \u{201c}{name}\u{201d}")));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not delete: {e}"))),
            }
        });
    });

    // Roles per champion from the trait table (for guessing a pick's role).
    let trait_roles: RwSignal<HashMap<String, Vec<Role>>> = RwSignal::new(HashMap::new());
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            if let Ok(rows) = crate::pages::draft_graph::trait_roles().await {
                let mut map = HashMap::with_capacity(rows.len() * 2);
                for (id, name, roles) in rows {
                    map.insert(name, roles.clone());
                    map.insert(id, roles);
                }
                trait_roles.set(map);
            }
        });
    });
    // Picker or Draft Graph beside the board.
    let view_mode = RwSignal::new("picker".to_string());

    // Board callbacks.
    let unavailable = Signal::derive(move || {
        let mut used: Vec<String> = ed.slots.get().into_iter().flatten().collect();
        used.extend(cx.fearless_used.get());
        used
    });
    let fill = Callback::new(move |(slot, name): (usize, String)| {
        let display = champs.with_value(|list| {
            list.iter()
                .find(|c| c.id == name || c.name == name)
                .map(|c| c.name.clone())
                .unwrap_or(name)
        });
        if cx.fearless_used.with_untracked(|f| f.contains(&display)) {
            toast.push.run(Toast::info(format!(
                "{display} was played earlier in this fearless series."
            )));
            return;
        }
        trait_roles.with_untracked(|tr| champs.with_value(|list| ed.fill(slot, display, list, tr)));
    });
    let on_pick = Callback::new(move |c: Champion| {
        if let Some(slot) = ed.active.get_untracked() {
            fill.run((slot, c.name));
        }
    });
    let on_select = Callback::new(move |slot: usize| ed.active.set(Some(slot)));
    let on_open = Callback::new(move |_slot: usize| {
        if view_mode.get_untracked() == "graph" {
            focus_id(&crate::pages::draft_graph::candidate_dom_id(0));
        } else {
            focus_id(&crate::components::champion_picker::picker_input_id(
                PICKER_ID,
            ));
        }
    });
    let on_clear = Callback::new(move |slot: usize| {
        if let Some(cleared) = ed.clear(slot) {
            let label = TOURNAMENT_ORDER[slot].describe();
            let champ = cleared.champion.clone();
            let undo = Callback::new(move |_| ed.restore_slot(slot, cleared.clone()));
            toast.push.run(
                Toast::info(format!("Cleared {champ} from {label}")).with_action("Undo", undo),
            );
        }
    });
    let on_escape = Callback::new(move |_| {
        let slot = ed.active.get_untracked().unwrap_or(19);
        focus_id(&slot_dom_id(BOARD_ID, slot));
    });
    let on_drop = Callback::new(move |(slot, name): (usize, String)| fill.run((slot, name)));
    let on_move = Callback::new(move |(from, to): (usize, usize)| ed.swap(from, to));
    // A pick from the Draft Graph: the champion, and the role the graph
    // ranked it for (guessed unless the user chose the role there). A role
    // a person set on the slot stays.
    let on_graph_pick = Callback::new(move |p: GraphPick| {
        let before = ed.snapshot();
        fill.run((p.slot, p.name.clone()));
        let placed = ed
            .slots
            .with_untracked(|s| s[p.slot].as_deref() == Some(p.name.as_str()));
        if !placed {
            return;
        }
        let person_set = before.roles[p.slot].is_some() && !before.guessed[p.slot];
        if let (Some(r), false) = (p.role, person_set) {
            ed.roles.update(|x| x[p.slot] = Some(r.key().to_string()));
            ed.guessed.update(|g| g[p.slot] = p.role_guessed);
        }
        let label = TOURNAMENT_ORDER[p.slot].describe();
        let undo = Callback::new(move |_| {
            let mut back = ed.snapshot();
            back.slots[p.slot] = before.slots[p.slot].clone();
            back.roles[p.slot] = before.roles[p.slot].clone();
            back.guessed[p.slot] = before.guessed[p.slot];
            back.notes[p.slot] = before.notes[p.slot].clone();
            let active = Some(p.slot);
            ed.restore(back);
            ed.active.set(active);
        });
        toast.push.run(
            Toast::success(format!("Picked {} for {label}", p.name)).with_action("Undo", undo),
        );
    });
    let graph_inputs = GraphInputs {
        slots: ed.slots,
        roles: ed.roles,
        guessed: ed.guessed,
        active: ed.active,
        side: ed.side,
        tags: ed.tags,
        opponent: Signal::derive(move || {
            let v = ed.opponent.get();
            opponents
                .with_value(|l| l.iter().any(|o| o.id.as_deref() == Some(v.as_str())))
                .then_some(v)
        }),
        unavailable: Signal::derive(move || cx.fearless_used.get()),
    };

    // Pool warnings for our picks.
    let warnings = Memo::new(move |_| {
        let slots = ed.slots.get();
        let side = ed.side.get();
        let roles = ed.roles.get();
        let by_name: HashMap<String, Champion> =
            champs.with_value(|l| l.iter().map(|c| (c.name.clone(), c.clone())).collect());
        let overrides: HashMap<usize, String> = roles
            .iter()
            .enumerate()
            .filter_map(|(i, r)| r.clone().map(|r| (i, r)))
            .collect();
        pools.with_value(|p| {
            warning_texts(compute_slot_warnings(
                &slots, p, &by_name, &side, &overrides,
            ))
        })
    });

    let our_side =
        Signal::derive(move || DraftSide::parse(&ed.side.get()).unwrap_or(DraftSide::Blue));
    let team_name = StoredValue::new(team_name);
    let blue_team = Signal::derive(move || match our_side.get() {
        DraftSide::Blue => team_name.get_value(),
        DraftSide::Red => opp_name.get(),
    });
    let red_team = Signal::derive(move || match our_side.get() {
        DraftSide::Red => team_name.get_value(),
        DraftSide::Blue => opp_name.get(),
    });
    let picker_label = Signal::derive(move || match ed.active.get() {
        Some(i) => format!("Champion for {}", TOURNAMENT_ORDER[i].describe()),
        None => "Draft complete: choose a slot on the board to change it".to_string(),
    });
    let title = move || {
        let n = ed.name.get();
        if n.trim().is_empty() {
            "New draft".to_string()
        } else {
            n
        }
    };
    let save_state = cx.save_state;
    let has_champs = !champ_list.is_empty();
    let champ_list_sv = StoredValue::new(champ_list);

    view! {
        <PageHeader
            title=title
            eyebrow="Draft"
            meta=move || view! {
                {move || opp_name.get().map(|o| view! { <span>"vs " {o}</span> })}
                {move || view! { <SideMarker side=our_side.get() text=SideText::Long /> }}
                <span data-testid="draft-save-state" class="text-muted">
                    {move || match save_state.get() {
                        SaveState::New => "Not saved yet",
                        SaveState::Dirty => "Unsaved changes",
                        SaveState::Saving => "Saving\u{2026}",
                        SaveState::Saved => "Saved",
                    }}
                </span>
            }
            actions=move || view! {
                <Button variant=ButtonVariant::Quiet on_click=new_draft>"New draft"</Button>
                {move || ed.loaded_id.get().map(|_| view! {
                    <Button
                        variant=ButtonVariant::Danger
                        on_click=Callback::new(move |_| confirm_delete.set(true))
                    >"Delete"</Button>
                })}
                <Button
                    variant=ButtonVariant::Primary
                    on_click=do_save
                    loading=Signal::derive(move || save_state.get() == SaveState::Saving)
                    attr:data-testid="draft-save"
                >"Save draft"</Button>
            }
        />
        <ConfirmDialog
            id="delete-draft"
            open=confirm_delete
            title="Delete this draft?"
            message="The draft, its picks and its comments are deleted. Game plans and reviews made from it stay and say the draft was deleted."
            confirm_label="Delete draft"
            tone=ConfirmTone::Destructive
            on_confirm=on_delete
        />

        // Board, picker and the side rail.
        <section
            aria-labelledby="draft-board-heading"
            class=move || if view_mode.get() == "graph" {
                "grid grid-cols-1 gap-x-8 gap-y-8 lg:grid-cols-[minmax(0,30rem)_minmax(0,1fr)]"
            } else {
                "grid grid-cols-1 gap-x-8 gap-y-8 lg:grid-cols-[minmax(0,30rem)_minmax(0,1fr)] xl:grid-cols-[minmax(0,30rem)_minmax(0,1fr)_17rem]"
            }
        >
            <h2 id="draft-board-heading" class="sr-only">"Board"</h2>
            <div class="flex min-w-0 flex-col gap-4">
                <crate::components::guidance::FirstUseHint hint=crate::models::guidance::Hint::DraftKeys title="The board works from the keyboard">
                    "Click the board once, then use the arrow keys to move between slots, Enter to choose a champion, Backspace to clear and Esc to come back. The keys are listed under the board."
                </crate::components::guidance::FirstUseHint>
                <DraftBoard
                    id=BOARD_ID
                    label="Draft board"
                    champions=ed.slots
                    roles=ed.roles
                    guessed=ed.guessed
                    notes=ed.notes
                    warnings=Signal::derive(move || warnings.get())
                    current=ed.active
                    blue_team=blue_team
                    red_team=red_team
                    on_select=on_select
                    on_open=on_open
                    on_clear=on_clear
                    on_drop=on_drop
                    on_move=on_move
                    hints=true
                />
                <SlotInspector ed=ed on_clear=on_clear />
            </div>
            <div class="flex min-w-0 flex-col gap-4">
                <div class="overflow-x-auto">
                    <SegmentedControl
                        id="draft-view"
                        label="Beside the board"
                        options=vec![
                            Segment::new("picker", "Picker"),
                            Segment::new("graph", "Draft Graph"),
                        ]
                        value=view_mode
                        size=SegmentSize::Sm
                    />
                </div>
                {move || {
                    if !has_champs {
                        return view! {
                            <Callout tone=CalloutTone::Danger title="Champions did not load">
                                "The champion list comes from the server's cache. Reload the page; if it stays empty, the server could not reach Data Dragon."
                            </Callout>
                        }.into_any();
                    }
                    if view_mode.get() == "graph" {
                        view! {
                            <DraftGraphPanel
                                inputs=graph_inputs
                                champions=champs
                                on_pick=on_graph_pick
                                on_escape=Callback::new(move |_| on_escape.run(()))
                            />
                        }.into_any()
                    } else {
                        view! {
                            <ChampionPicker
                                id=PICKER_ID
                                champions=champ_list_sv.get_value()
                                unavailable=unavailable
                                on_select=on_pick
                                on_escape=on_escape
                                label=picker_label
                                enabled=Signal::derive(move || ed.active.get().is_some())
                            />
                        }.into_any()
                    }
                }}
            </div>
            <div class=move || if view_mode.get() == "graph" { "min-w-0 lg:col-span-2" } else { "min-w-0 lg:col-span-2 xl:col-span-1" }>
                <DraftRail ed=ed opponents=opponents pools=pools champs=champs fill=fill />
            </div>
        </section>

        <PlanSection cx=cx opponents=opponents name_error=name_error />
        <CommentsSection ed=ed />
        <NextSteps cx=cx />
        <SavedDrafts cx=cx opponents=opponents />
        <Insights />
    }
}

/// Role, note and clear for the slot that is lit, when it holds a champion.
#[component]
fn SlotInspector(ed: Editor, on_clear: Callback<usize>) -> impl IntoView {
    let role_edit = RwSignal::new(String::new());
    let note_edit = RwSignal::new(String::new());
    // Follow the lit slot.
    Effect::new(move |_| {
        let slot = ed.active.get();
        let role = slot.and_then(|i| ed.roles.with(|r| r[i].clone()));
        role_edit.set(
            role.as_deref()
                .and_then(Role::parse)
                .map(|r| r.label().to_lowercase())
                .unwrap_or_default(),
        );
    });
    Effect::new(move |_| {
        let slot = ed.active.get();
        let note = slot.and_then(|i| ed.notes.with_untracked(|n| n[i].clone()));
        note_edit.set(note.unwrap_or_default());
    });
    // Write the note back as it is typed.
    Effect::new(move |_| {
        let text = note_edit.get();
        let Some(slot) = ed.active.get_untracked() else {
            return;
        };
        let value = non_empty(text);
        if ed.notes.with_untracked(|n| n[slot] != value)
            && ed.slots.with_untracked(|s| s[slot].is_some())
        {
            ed.notes.update(|n| n[slot] = value);
        }
    });
    let on_role = Callback::new(move |r: String| {
        if let Some(slot) = ed.active.get_untracked() {
            ed.roles.update(|roles| roles[slot] = Some(r));
            ed.guessed.update(|g| g[slot] = false);
        }
    });
    view! {
        {move || {
            let slot = ed.active.get()?;
            let champ = ed.slots.with(|s| s[slot].clone())?;
            let step = TOURNAMENT_ORDER[slot];
            let is_pick = slot_meta(slot).1 == "pick";
            let guessed = ed.guessed.with(|g| g[slot]);
            Some(view! {
                <div class="flex flex-col gap-3 border-t border-rule pt-4" data-testid="slot-inspector">
                    <p class="flex flex-wrap items-baseline gap-x-2">
                        <span class="eyebrow">{step.describe()}</span>
                        <span class="font-display text-heading font-semibold text-primary">{champ}</span>
                    </p>
                    {is_pick.then(|| view! {
                        <div class="flex flex-col gap-1.5">
                            <span class="text-label font-medium text-secondary" id="slot-role-label">
                                "Role"
                                {guessed.then(|| view! { <span class="font-normal text-muted" data-testid="slot-role-guessed">" (guessed; choose one to confirm it)"</span> })}
                            </span>
                            <div class="overflow-x-auto">
                                <SegmentedControl
                                    id="slot-role"
                                    label="Role"
                                    options={Role::ALL.iter().map(|r| Segment::new(r.label().to_lowercase(), r.label())).collect::<Vec<_>>()}
                                    value=role_edit
                                    size=SegmentSize::Sm
                                    on_change=on_role
                                />
                            </div>
                        </div>
                        <Field id="slot-note" label="Why this pick" hint="Shown under the champion on the board.">
                            <TextField value=note_edit placeholder="Engage for Jinx, safe blind" maxlength=140 />
                        </Field>
                    })}
                    <div>
                        <Button
                            variant=ButtonVariant::Quiet
                            size=ButtonSize::Sm
                            shortcut="Backspace"
                            on_click=Callback::new(move |_| on_clear.run(slot))
                        >"Clear slot"</Button>
                    </div>
                </div>
            })
        }}
    }
}

/// The side rail: opponent brief, our pools, notes on the lit champion.
#[component]
fn DraftRail(
    ed: Editor,
    opponents: StoredValue<Vec<Opponent>>,
    pools: StoredValue<Vec<(String, String, Vec<ChampionPoolEntry>)>>,
    champs: StoredValue<Vec<Champion>>,
    fill: Callback<(usize, String)>,
) -> impl IntoView {
    let tab = RwSignal::new("opponent".to_string());
    view! {
        <aside aria-label="Draft intel" class="flex min-w-0 flex-col gap-4">
            <Tabs
                group="draft-rail"
                label="Draft intel"
                tabs=vec![
                    TabItem::new("opponent", "Opponent"),
                    TabItem::new("pools", "Our pools"),
                    TabItem::new("notes", "Notes"),
                ]
                selected=tab
            />
            <TabPanel group="draft-rail" tab="opponent" selected=tab>
                <OpponentBrief ed=ed opponents=opponents />
            </TabPanel>
            <TabPanel group="draft-rail" tab="pools" selected=tab>
                <PoolsPanel ed=ed pools=pools champs=champs fill=fill />
            </TabPanel>
            <TabPanel group="draft-rail" tab="notes" selected=tab>
                <NotesPanel ed=ed />
            </TabPanel>
        </aside>
    }
}

/// Their players and the champions they played most in scouted games.
#[component]
fn OpponentBrief(ed: Editor, opponents: StoredValue<Vec<Opponent>>) -> impl IntoView {
    let opp_id = Memo::new(move |_| {
        let v = ed.opponent.get();
        opponents
            .with_value(|l| l.iter().any(|o| o.id.as_deref() == Some(v.as_str())))
            .then_some(v)
    });
    let players = Resource::new(
        move || opp_id.get(),
        |id| async move {
            match id {
                Some(id) => get_opponent_intel(id).await,
                None => Ok(Vec::new()),
            }
        },
    );
    view! {
        <div class="flex flex-col gap-3" data-testid="opponent-brief">
            {move || match opp_id.get() {
                None => view! {
                    <EmptyState
                        size=EmptySize::Compact
                        message="Choose an opponent under Plan to see their players and the champions seen in their scouted games."
                    />
                }.into_any(),
                Some(_) => view! {
                    <Suspense fallback=|| view! {
                        <SkeletonGroup label="Loading the opponent">
                            <Skeleton shape=SkeletonShape::Line lines=4 />
                        </SkeletonGroup>
                    }>
                        {move || players.get().map(|r| match r {
                            Err(_) => view! {
                                <Callout tone=CalloutTone::Danger>"The opponent's players did not load."</Callout>
                            }.into_any(),
                            Ok(list) if list.is_empty() => view! {
                                <EmptyState
                                    size=EmptySize::Compact
                                    message="No players scouted for this opponent yet. Add them on the Opponents page and their recent champions appear here."
                                    cta_label="Open Opponents"
                                    cta_href="/opponents"
                                />
                            }.into_any(),
                            Ok(mut list) => {
                                list.sort_by_key(|p| Role::parse(&p.role).map(|r| Role::ALL.iter().position(|x| *x == r).unwrap_or(9)).unwrap_or(9));
                                let drafted = ed.slots.get();
                                view! {
                                    <ul class="flex flex-col">
                                        {list.into_iter().map(|p| {
                                            let intel = p.compute_intel();
                                            let role = Role::parse(&p.role);
                                            let games = p.recent_champions.len();
                                            let top: Vec<(String, u32)> = intel.champion_frequencies.into_iter().take(5).collect();
                                            let drafted = drafted.clone();
                                            view! {
                                                <li class="flex flex-col gap-1.5 border-b border-rule py-2.5 last:border-b-0">
                                                    <div class="flex items-center gap-2">
                                                        {role.map(|r| view! { <RoleLabel role=r /> })}
                                                        <span class="truncate text-label font-medium text-primary">{player_name(&p)}</span>
                                                        <span class="figures ml-auto text-caption text-muted">
                                                            {if games == 1 { "1 champion seen".to_string() } else { format!("{games} champions seen") }}
                                                        </span>
                                                    </div>
                                                    {if top.is_empty() {
                                                        view! { <p class="text-caption text-muted">"No recent games scouted."</p> }.into_any()
                                                    } else {
                                                        view! {
                                                            <ul class="flex flex-wrap gap-x-2 gap-y-1">
                                                                {top.into_iter().map(|(c, n)| {
                                                                    let taken = drafted.iter().any(|d| d.as_deref() == Some(c.as_str()));
                                                                    view! {
                                                                        <li class="inline-flex items-center gap-1">
                                                                            <ChampionAvatar
                                                                                name=c.clone()
                                                                                size=AvatarSize::Sm
                                                                                state=if taken { AvatarState::Dimmed } else { AvatarState::Normal }
                                                                            />
                                                                            <span class="figures text-caption text-secondary">{format!("\u{00d7}{n}")}</span>
                                                                            {taken.then(|| view! { <span class="sr-only">" (on the board)"</span> })}
                                                                        </li>
                                                                    }
                                                                }).collect_view()}
                                                            </ul>
                                                        }.into_any()
                                                    }}
                                                    {intel.otp_champion.map(|c| view! {
                                                        <p class="text-caption text-warning">{format!("One-trick: {c}")}</p>
                                                    })}
                                                </li>
                                            }
                                        }).collect_view()}
                                    </ul>
                                    <p class="text-caption text-muted">
                                        "From scouted match history; bans are not part of it. Dimmed champions are already on the board."
                                    </p>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                }.into_any(),
            }}
        </div>
    }
}

/// Our starters' pools; a champion fills the lit slot.
#[component]
fn PoolsPanel(
    ed: Editor,
    pools: StoredValue<Vec<(String, String, Vec<ChampionPoolEntry>)>>,
    champs: StoredValue<Vec<Champion>>,
    fill: Callback<(usize, String)>,
) -> impl IntoView {
    let tier_rank = |t: &str| match t {
        "comfort" => 0,
        "match_ready" => 1,
        "scrim_ready" => 2,
        "practicing" => 3,
        _ => 4,
    };
    view! {
            {match pools.get_value() {
                pools if pools.iter().all(|(_, _, e)| e.is_empty()) => view! {
                    <EmptyState
                        size=EmptySize::Compact
                        message="Your starters' champion pools appear here once they add champions on the Champion pool page."
                        cta_label="Open Champion pool"
                        cta_href="/champion-pool"
                    />
                }.into_any(),
                mut pools => {
                    pools.sort_by_key(|(_, role, _)| Role::parse(role).map(|r| Role::ALL.iter().position(|x| *x == r).unwrap_or(9)).unwrap_or(9));
                    let names: HashMap<String, String> = champs.with_value(|l| l.iter().map(|c| (c.id.clone(), c.name.clone())).collect());
                    view! {
                        <ul class="flex flex-col">
                            {pools.into_iter().map(|(user, role, mut entries)| {
                                entries.sort_by_key(|e| tier_rank(&e.tier));
                                let role = Role::parse(&role);
                                let names = names.clone();
                                view! {
                                    <li class="flex flex-col gap-1.5 border-b border-rule py-2.5 last:border-b-0">
                                        <div class="flex items-center gap-2">
                                            {role.map(|r| view! { <RoleLabel role=r /> })}
                                            <span class="truncate text-label font-medium text-primary">{user}</span>
                                        </div>
                                        {if entries.is_empty() {
                                            view! { <p class="text-caption text-muted">"No champions in the pool yet."</p> }.into_any()
                                        } else {
                                            view! {
                                                <div class="flex flex-wrap gap-1">
                                                    {entries.into_iter().map(|e| {
                                                        let display = names.get(&e.champion).cloned().unwrap_or_else(|| e.champion.clone());
                                                        let for_click = display.clone();
                                                        let icon_name = display.clone();
                                                        view! {
                                                            <Button
                                                                variant=ButtonVariant::Quiet
                                                                size=ButtonSize::Sm
                                                                icon=move || view! { <ChampionAvatar name=icon_name.clone() size=AvatarSize::Xs decorative=true /> }
                                                                on_click=Callback::new(move |_| {
                                                                    if let Some(slot) = ed.active.get_untracked() {
                                                                        fill.run((slot, for_click.clone()));
                                                                    }
                                                                })
                                                            >{display}</Button>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }}
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                        <p class="text-caption text-muted">"Choosing a champion here fills the lit slot."</p>
                    }.into_any()
                }
            }}
    }
}

/// Our pool notes and the team's matchup notes on the lit champion.
#[component]
fn NotesPanel(ed: Editor) -> impl IntoView {
    let champ = Memo::new(move |_| {
        ed.active
            .get()
            .and_then(|i| ed.slots.with(|s| s[i].clone()))
    });
    let notes = Resource::new(
        move || champ.get(),
        |c| async move {
            let Some(c) = c else {
                return Ok((Vec::new(), Vec::new()));
            };
            let json = serde_json::to_string(&vec![c.clone()]).unwrap_or_default();
            let pool = get_pool_notes_for_champions(json)
                .await?
                .into_iter()
                .flat_map(|(_, n)| n)
                .collect::<Vec<ChampionNote>>();
            let matchup = get_matchup_notes_for_champion(c).await?;
            Ok::<_, ServerFnError>((pool, matchup))
        },
    );
    view! {
        {move || match champ.get() {
            None => view! {
                <EmptyState
                    size=EmptySize::Compact
                    message="Select a champion on the board to see your pool notes and your team's matchup notes on it."
                />
            }.into_any(),
            Some(c) => view! {
                <div class="flex flex-col gap-3">
                    <p class="font-display text-heading font-semibold text-primary">{c.clone()}</p>
                    <Suspense fallback=|| view! { <SkeletonGroup label="Loading notes"><Skeleton shape=SkeletonShape::Line lines=3 /></SkeletonGroup> }>
                        {move || notes.get().map(|r| match r {
                            Err(_) => view! { <Callout tone=CalloutTone::Danger>"The notes did not load."</Callout> }.into_any(),
                            Ok((pool, matchup)) if pool.is_empty() && matchup.is_empty() => view! {
                                <EmptyState
                                    size=EmptySize::Compact
                                    message="No notes on this champion yet. Notes written on the Champion pool page appear here."
                                />
                            }.into_any(),
                            Ok((pool, matchup)) => view! {
                                <ul class="flex flex-col">
                                    {pool.into_iter().map(|n| view! {
                                        <li class="flex flex-col gap-0.5 border-b border-rule py-2 last:border-b-0">
                                            <span class="eyebrow">{note_type_label(&n.note_type)}</span>
                                            {(!n.title.is_empty()).then(|| view! { <span class="text-label font-medium text-primary">{n.title.clone()}</span> })}
                                            <span class="text-label text-secondary">{n.content.clone()}</span>
                                        </li>
                                    }).collect_view()}
                                    {matchup.into_iter().map(|(author, n)| view! {
                                        <li class="flex flex-col gap-0.5 border-b border-rule py-2 last:border-b-0">
                                            <span class="eyebrow">{format!("Matchup \u{00b7} {author}")}</span>
                                            {(!n.title.is_empty()).then(|| view! { <span class="text-label font-medium text-primary">{n.title.clone()}</span> })}
                                            <span class="text-label text-secondary">{n.content.clone()}</span>
                                        </li>
                                    }).collect_view()}
                                </ul>
                            }.into_any(),
                        })}
                    </Suspense>
                </div>
            }.into_any(),
        }}
    }
}

/// Name, side, opponent, series, rating, tags and the two plan notes.
#[component]
fn PlanSection(
    cx: Ctx,
    opponents: StoredValue<Vec<Opponent>>,
    name_error: RwSignal<Option<String>>,
) -> impl IntoView {
    let ed = cx.ed;
    let series_resource = cx.series;
    let fearless_used = cx.fearless_used;
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let nav = navigator();
    // A stored opponent that is not in the list (free text from before
    // opponents were records) stays selectable.
    let opponent_options = Memo::new(move |_| {
        // Recomputed when a draft is opened (a legacy opponent appears);
        // not on every selection, so the select keeps focus.
        let _ = ed.loaded_id.get();
        let list = opponents.get_value();
        let current = ed.opponent.get_untracked();
        let mut opts = vec![SelectOption::new("", "No opponent")];
        let mut known = false;
        for o in &list {
            let id = o.id.clone().unwrap_or_default();
            known |= id == current;
            opts.push(SelectOption::new(id, o.name.clone()));
        }
        if !current.is_empty() && !known {
            opts.push(SelectOption::new(
                current.clone(),
                format!("{current} (not in Opponents)"),
            ));
        }
        opts
    });
    let series_options = move |list: Vec<Series>| {
        let mut opts = vec![SelectOption::new("", "Not part of a series")];
        for s in list {
            let label = format!(
                "{} ({}{})",
                s.name,
                s.format.to_uppercase(),
                if s.is_fearless { ", fearless" } else { "" }
            );
            opts.push(SelectOption::new(s.id.unwrap_or_default(), label));
        }
        opts
    };

    // New series dialog.
    let series_open = RwSignal::new(false);
    let s_name = RwSignal::new(String::new());
    let s_opp = RwSignal::new(String::new());
    let s_format = RwSignal::new("bo3".to_string());
    let s_fearless = RwSignal::new(false);
    let s_error = RwSignal::new(None::<String>);
    let create_series = Callback::new(move |_: leptos::ev::MouseEvent| {
        let name = s_name.get_untracked();
        if name.trim().is_empty() {
            s_error.set(Some("Give the series a name.".into()));
            return;
        }
        s_error.set(None);
        let opp = non_empty(s_opp.get_untracked());
        let fmt = s_format.get_untracked();
        let fearless = s_fearless.get_untracked();
        leptos::task::spawn_local(async move {
            match create_series_fn(name.trim().to_string(), opp, fmt, fearless).await {
                Ok(id) => {
                    series_resource.refetch();
                    ed.series_id.set(id);
                    ed.game.set("1".to_string());
                    s_name.set(String::new());
                    s_opp.set(String::new());
                    series_open.set(false);
                    toast.push.run(Toast::success("Series created"));
                }
                Err(e) => s_error.set(Some(format!("Could not create the series: {e}"))),
            }
        });
    });

    // Save the draft, then go and scout a new opponent (comes back here).
    let new_opponent = Callback::new(move |_: leptos::ev::MouseEvent| {
        let snap = ed.snapshot();
        if snap.name.trim().is_empty() {
            name_error.set(Some(
                "Name the draft first; it is saved before you leave.".into(),
            ));
            focus_id("draft-name");
            return;
        }
        leptos::task::spawn_local(async move {
            match persist(snap).await {
                Ok(id) => go(nav, &format!("/opponents?return_to=draft&draft_id={id}")),
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not save the draft: {e}"))),
            }
        });
    });

    let side_value = ed.side;
    view! {
        <section aria-labelledby="draft-plan-heading" class="mt-12">
            <SectionHeader title="Plan" id="draft-plan-heading" />
            <div class="grid grid-cols-1 gap-x-8 gap-y-5 md:grid-cols-2">
                <Field id="draft-name" label="Draft name" required=true error=Signal::derive(move || name_error.get())>
                    <TextField value=ed.name placeholder="vs Riverside, game 1" maxlength=120 />
                </Field>
                <div class="flex flex-col gap-1.5">
                    <span class="text-label font-medium text-secondary">"Our side"</span>
                    <div>
                        <SegmentedControl
                            id="our-side"
                            label="Our side"
                            options=vec![Segment::new("blue", "Blue"), Segment::new("red", "Red")]
                            value=side_value
                        />
                    </div>
                </div>
                <div class="flex flex-col gap-1.5">
                    {move || {
                        let opts = opponent_options.get();
                        view! {
                            <Field id="draft-opponent" label="Opponent">
                                <Select value=ed.opponent options=opts />
                            </Field>
                        }
                    }}
                    <div>
                        <Button variant=ButtonVariant::Quiet size=ButtonSize::Sm on_click=new_opponent>
                            "Scout a new opponent"
                        </Button>
                    </div>
                </div>
                <div class="flex flex-col gap-1.5">
                    <Suspense fallback=|| view! { <Skeleton shape=SkeletonShape::Block size="h-9 w-full" /> }>
                        {move || series_resource.get().map(|r| {
                            let list = r.unwrap_or_default();
                            let opts = series_options(list.clone());
                            let list = StoredValue::new(list);
                            view! {
                                <Field id="draft-series" label="Series" hint="Fearless series rule out champions played in the series' other games.">
                                    <Select value=ed.series_id options=opts />
                                </Field>
                                {move || {
                                    let id = ed.series_id.get();
                                    list.with_value(|l| l.iter().find(|s| s.id.as_deref() == Some(id.as_str())).cloned())
                                }.map(|s| {
                                    let n = match s.format.as_str() { "bo5" => 5, "bo3" => 3, _ => 1 };
                                    let opts = (1..=n).map(|g| Segment::new(g.to_string(), format!("Game {g}"))).collect::<Vec<_>>();
                                    view! {
                                        <div class="pt-1">
                                            <SegmentedControl id="series-game" label="Game in the series" options=opts value=ed.game size=SegmentSize::Sm />
                                        </div>
                                    }
                                })}
                            }
                        })}
                    </Suspense>
                    <div class="flex flex-wrap items-center gap-3">
                        <Button variant=ButtonVariant::Quiet size=ButtonSize::Sm on_click=Callback::new(move |_| series_open.set(true))>
                            "New series"
                        </Button>
                    </div>
                    {move || {
                        let used = fearless_used.get();
                        (!used.is_empty()).then(|| view! {
                            <p class="text-caption text-secondary">
                                <span class="font-medium">"Played earlier in the series: "</span>
                                {used.join(", ")}
                            </p>
                        })
                    }}
                </div>
                <div class="flex flex-col gap-1.5">
                    <span class="text-label font-medium text-secondary" id="draft-rating-label">"Rating"</span>
                    <div role="group" aria-labelledby="draft-rating-label" class="flex flex-wrap gap-1">
                        {RATINGS.iter().map(|&r| view! {
                            <Button
                                size=ButtonSize::Sm
                                pressed=Signal::derive(move || ed.rating.get().as_deref() == Some(r))
                                on_click=Callback::new(move |_| ed.rating.update(|cur| {
                                    *cur = if cur.as_deref() == Some(r) { None } else { Some(r.to_string()) };
                                }))
                            >{r}</Button>
                        }).collect_view()}
                    </div>
                </div>
                <div class="flex flex-col gap-1.5">
                    <span class="text-label font-medium text-secondary" id="draft-tags-label">"Composition"</span>
                    <div role="group" aria-labelledby="draft-tags-label" class="flex flex-wrap gap-1">
                        {COMPOSITION_TAGS.iter().map(|&t| view! {
                            <Button
                                size=ButtonSize::Sm
                                pressed=Signal::derive(move || ed.tags.get().iter().any(|x| x == t))
                                on_click=Callback::new(move |_| ed.tags.update(|tags| {
                                    if let Some(p) = tags.iter().position(|x| x == t) {
                                        tags.remove(p);
                                    } else {
                                        tags.push(t.to_string());
                                    }
                                }))
                            >{t}</Button>
                        }).collect_view()}
                    </div>
                </div>
                <Field id="draft-win" label="How we win">
                    <TextField value=ed.win rows=3 placeholder="Front-to-back around Jinx; play for dragon soul." />
                </Field>
                <Field id="draft-watch" label="Watch out for">
                    <TextField value=ed.watch rows=3 placeholder="Their engage on our carry before 15 minutes." />
                </Field>
            </div>
            <Dialog
                id="new-series"
                open=series_open
                title="New series"
                description="Drafts in a series share a game count; a fearless series rules out champions already played."
                dismiss_on_scrim=false
                footer=move || view! {
                    <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| series_open.set(false))>"Cancel"</Button>
                    <Button variant=ButtonVariant::Primary on_click=create_series>"Create series"</Button>
                }
            >
                <div class="flex flex-col gap-4 px-5 py-4">
                    <Field id="series-name" label="Name" required=true error=Signal::derive(move || s_error.get())>
                        <TextField value=s_name placeholder="Spring split week 3" />
                    </Field>
                    <Field id="series-opponent" label="Opponent">
                        <TextField value=s_opp placeholder="Riverside Rooks" />
                    </Field>
                    <Field id="series-format" label="Format">
                        <Select value=s_format options=vec![
                            SelectOption::new("bo1", "Best of 1"),
                            SelectOption::new("bo3", "Best of 3"),
                            SelectOption::new("bo5", "Best of 5"),
                        ] />
                    </Field>
                    <Switch label="Fearless" checked=s_fearless show_state=true />
                </div>
            </Dialog>
        </section>
    }
}

/// Team comments on the draft (saved with it).
#[component]
fn CommentsSection(ed: Editor) -> impl IntoView {
    let input = RwSignal::new(String::new());
    let add = Callback::new(move |_: leptos::ev::MouseEvent| {
        let text = input.get_untracked().trim().to_string();
        if !text.is_empty() {
            ed.comments.update(|c| c.push(text));
            input.set(String::new());
        }
    });
    view! {
        <section aria-labelledby="draft-comments-heading" class="mt-12">
            <SectionHeader title="Comments" id="draft-comments-heading" />
            <div class="flex max-w-3xl flex-col gap-4">
                {move || {
                    let list = ed.comments.get();
                    if list.is_empty() {
                        view! {
                            <p class="text-body text-secondary">"No comments yet. Comments are saved with the draft."</p>
                        }.into_any()
                    } else {
                        view! {
                            <ul class="flex flex-col">
                                {list.into_iter().map(|c| view! {
                                    <li class="border-b border-rule py-2 text-reading text-primary last:border-b-0">{c}</li>
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }
                }}
                <Field id="draft-comment" label="Add a comment">
                    <TextField value=input rows=2 placeholder="Ask the jungler about level 3 timings." />
                </Field>
                <div><Button on_click=add>"Add comment"</Button></div>
            </div>
        </section>
    }
}

/// From a saved draft to its game plan and review.
#[component]
fn NextSteps(cx: Ctx) -> impl IntoView {
    let ed = cx.ed;
    let game_plan_counts = cx.plan_counts;
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let nav = navigator();
    let existing = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    Effect::new(move |_| {
        let _ = ed.loaded_id.get();
        existing.set(None);
    });
    let prep = Callback::new(move |_: leptos::ev::MouseEvent| {
        let Some(id) = ed.loaded_id.get_untracked() else {
            return;
        };
        busy.set(true);
        leptos::task::spawn_local(async move {
            match check_draft_has_game_plan(id.clone()).await {
                Ok(None) => go(nav, &format!("/game-plan?draft_id={id}")),
                Ok(Some(plan)) => existing.set(Some(plan)),
                Err(e) => toast.push.run(Toast::error(format!("{e}"))),
            }
            busy.set(false);
        });
    });
    view! {
        <Suspense fallback=|| ()>
        {move || ed.loaded_id.get().map(|id| {
            let plans = game_plan_counts
                .get()
                .and_then(|r| r.ok())
                .and_then(|l| l.into_iter().find(|(d, _)| *d == id).map(|(_, n)| n))
                .unwrap_or(0);
            let id_review = id.clone();
            let id_new = id.clone();
            view! {
                <section aria-labelledby="draft-next-heading" class="mt-12">
                    <SectionHeader title="Next steps" id="draft-next-heading" />
                    <div class="flex flex-col gap-3">
                        <div class="flex flex-wrap items-center gap-2">
                            <Button on_click=prep loading=Signal::derive(move || busy.get())>"Prep a game plan"</Button>
                            {(plans > 0).then(|| view! {
                                <Button href=format!("/post-game?draft_id={id_review}")>"Review this game"</Button>
                            })}
                            <span class="text-label text-muted">
                                {match plans { 0 => "No game plan yet".to_string(), 1 => "1 game plan".to_string(), n => format!("{n} game plans") }}
                            </span>
                        </div>
                        {move || existing.get().map(|plan| {
                            let new_href = format!("/game-plan?draft_id={id_new}");
                            view! {
                                <Callout
                                    tone=CalloutTone::Info
                                    title="This draft already has a game plan"
                                    action=move || view! {
                                        <div class="flex flex-wrap gap-2">
                                            <Button size=ButtonSize::Sm href=format!("/game-plan?plan_id={plan}")>"Open it"</Button>
                                            <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet href=new_href.clone()>"Make another"</Button>
                                        </div>
                                    }
                                >
                                    "Open the existing plan or start a second one for this draft."
                                </Callout>
                            }
                        })}
                    </div>
                </section>
            }
        })}
        </Suspense>
    }
}

/// The team's drafts, newest first, with a tag filter.
#[component]
fn SavedDrafts(cx: Ctx, opponents: StoredValue<Vec<Opponent>>) -> impl IntoView {
    let drafts = cx.drafts;
    let game_plan_counts = cx.plan_counts;
    let tag = RwSignal::new(String::new());
    let tag_options = {
        let mut o = vec![SelectOption::new("", "All drafts")];
        o.extend(COMPOSITION_TAGS.iter().map(|t| SelectOption::new(*t, *t)));
        o
    };
    view! {
        <section aria-labelledby="saved-drafts-heading" class="mt-12">
            <SectionHeader
                title="Saved drafts"
                id="saved-drafts-heading"
                actions=move || {
                    let opts = tag_options.clone();
                    view! {
                        <div class="w-44">
                            <Select value=tag options=opts size=FieldSize::Sm aria_label="Filter by composition" />
                        </div>
                    }
                }
            />
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading drafts"><Skeleton shape=SkeletonShape::Line lines=4 /></SkeletonGroup> }>
                {move || drafts.get().map(|r| match r {
                    Err(_) => view! { <Callout tone=CalloutTone::Danger>"The drafts did not load."</Callout> }.into_any(),
                    Ok(list) if list.is_empty() => view! {
                        <EmptyState size=EmptySize::Compact message="Drafts you save appear here, with their picks, rating and game plans." />
                    }.into_any(),
                    Ok(list) => {
                        let t = tag.get();
                        let list: Vec<Draft> = list.into_iter().filter(|d| t.is_empty() || d.tags.contains(&t)).collect();
                        if list.is_empty() {
                            return view! {
                                <EmptyState size=EmptySize::Compact message="No saved draft has this composition tag." />
                            }.into_any();
                        }
                        let counts: HashMap<String, usize> = game_plan_counts.get().and_then(|r| r.ok()).unwrap_or_default().into_iter().collect();
                        let opps = opponents.get_value();
                        view! {
                            <Table caption="Saved drafts" caption_hidden=true>
                                <thead>
                                    <tr>
                                        <Th>"Draft"</Th>
                                        <Th>"Opponent"</Th>
                                        <Th>"Side"</Th>
                                        <Th>"Our picks"</Th>
                                        <Th>"Rating"</Th>
                                        <Th>"Tags"</Th>
                                        <Th numeric=true>"Plans"</Th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {list.into_iter().map(|d| {
                                        let id = d.id.clone().unwrap_or_default();
                                        let side = DraftSide::parse(&d.our_side).unwrap_or(DraftSide::Blue);
                                        let mut picks: Vec<(usize, String)> = d.actions.iter()
                                            .filter(|a| a.side == side.as_str() && slot_meta(a.order as usize).1 == "pick")
                                            .map(|a| (a.order as usize, a.champion.clone()))
                                            .collect();
                                        picks.sort_by_key(|(o, _)| *o);
                                        let opp = d.opponent.as_deref().and_then(|o| opponent_name(o, &opps));
                                        let no_opp = opp.is_none();
                                        let plans = counts.get(&id).copied().unwrap_or(0);
                                        let no_rating = d.rating.is_none();
                                        view! {
                                            <Tr>
                                                <Td header=true><RowLink href=format!("/draft?draft_id={id}")>{d.name.clone()}</RowLink></Td>
                                                <Td muted=no_opp>{opp.unwrap_or_else(|| "\u{2014}".to_string())}</Td>
                                                <Td><SideMarker side=side text=SideText::Short /></Td>
                                                <Td>
                                                    {if picks.is_empty() {
                                                        view! { <span class="text-muted">"\u{2014}"</span> }.into_any()
                                                    } else {
                                                        view! {
                                                            <span class="flex gap-0.5">
                                                                {picks.into_iter().map(|(_, c)| view! { <ChampionAvatar name=c size=AvatarSize::Xs /> }).collect_view()}
                                                            </span>
                                                        }.into_any()
                                                    }}
                                                </Td>
                                                <Td muted=no_rating>{d.rating.clone().unwrap_or_else(|| "\u{2014}".to_string())}</Td>
                                                <Td muted=true>{if d.tags.is_empty() { "\u{2014}".to_string() } else { d.tags.join(", ") }}</Td>
                                                <Td numeric=true muted=plans == 0>{plans}</Td>
                                            </Tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </Table>
                        }.into_any()
                    }
                })}
            </Suspense>
        </section>
    }
}

/// Ban priorities, tendencies and outcomes, one tab each.
#[component]
fn Insights() -> impl IntoView {
    let tab = RwSignal::new("bans".to_string());
    view! {
        <section aria-labelledby="draft-insights-heading" class="mt-12">
            <SectionHeader title="Insights" id="draft-insights-heading" />
            <div class="flex flex-col gap-4">
                <Tabs
                    group="draft-insights"
                    label="Draft insights"
                    tabs=vec![
                        TabItem::new("bans", "Ban priorities"),
                        TabItem::new("tendencies", "Tendencies"),
                        TabItem::new("outcomes", "Outcomes"),
                    ]
                    selected=tab
                />
                <TabPanel group="draft-insights" tab="bans" selected=tab><BanPriorities /></TabPanel>
                <TabPanel group="draft-insights" tab="tendencies" selected=tab><Tendencies /></TabPanel>
                <TabPanel group="draft-insights" tab="outcomes" selected=tab><Outcomes /></TabPanel>
            </div>
        </section>
    }
}

#[component]
fn BanPriorities() -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let list = Resource::new(|| (), |_| get_ban_priorities());
    let editing = RwSignal::new(false);
    let edit = RwSignal::new(Vec::<BanPriority>::new());
    let champ = RwSignal::new(String::new());
    let reason = RwSignal::new(String::new());
    let start = Callback::new(move |_: leptos::ev::MouseEvent| {
        edit.set(
            list.get_untracked()
                .and_then(|r| r.ok())
                .unwrap_or_default(),
        );
        editing.set(true);
    });
    let add = Callback::new(move |_: leptos::ev::MouseEvent| {
        let c = champ.get_untracked().trim().to_string();
        if c.is_empty() {
            return;
        }
        let r = non_empty(reason.get_untracked());
        edit.update(|l| {
            let rank = l.len() as i32;
            l.push(BanPriority {
                id: None,
                team_id: String::new(),
                champion: c,
                rank,
                reason: r,
            });
        });
        champ.set(String::new());
        reason.set(String::new());
        focus_id("ban-champion");
    });
    let save = Callback::new(move |_: leptos::ev::MouseEvent| {
        let json = serde_json::to_string(&edit.get_untracked()).unwrap_or_default();
        leptos::task::spawn_local(async move {
            match save_ban_priorities(json).await {
                Ok(()) => {
                    list.refetch();
                    editing.set(false);
                    toast.push.run(Toast::success("Ban priorities saved"));
                }
                Err(e) => toast.push.run(Toast::error(format!("Could not save: {e}"))),
            }
        });
    });
    view! {
        <div class="flex max-w-3xl flex-col gap-4">
            {move || if editing.get() {
                view! {
                    <div class="flex flex-col gap-4">
                        {move || {
                            let items = edit.get();
                            if items.is_empty() {
                                view! { <p class="text-body text-secondary">"No priorities yet. Add the champions you want banned first."</p> }.into_any()
                            } else {
                                view! {
                                    <ol class="flex flex-col">
                                        {items.into_iter().enumerate().map(|(i, b)| view! {
                                            <li class="flex items-center gap-3 border-b border-rule py-2 last:border-b-0">
                                                <span class="figures w-6 text-label text-muted">{i + 1}</span>
                                                <ChampionAvatar name=b.champion.clone() size=AvatarSize::Sm decorative=true />
                                                <span class="text-label font-medium text-primary">{b.champion.clone()}</span>
                                                <span class="min-w-0 flex-1 truncate text-label text-secondary">{b.reason.clone().unwrap_or_default()}</span>
                                                <Button
                                                    variant=ButtonVariant::Quiet
                                                    size=ButtonSize::Sm
                                                    aria_label=format!("Remove {}", b.champion)
                                                    on_click=Callback::new(move |_| edit.update(|l| {
                                                        if i < l.len() {
                                                            l.remove(i);
                                                        }
                                                        for (j, x) in l.iter_mut().enumerate() {
                                                            x.rank = j as i32;
                                                        }
                                                    }))
                                                >"Remove"</Button>
                                            </li>
                                        }).collect_view()}
                                    </ol>
                                }.into_any()
                            }
                        }}
                        <div class="grid grid-cols-1 items-end gap-3 sm:grid-cols-[1fr_2fr_auto]">
                            <Field id="ban-champion" label="Champion">
                                <TextField value=champ placeholder="Kalista" />
                            </Field>
                            <Field id="ban-reason" label="Reason">
                                <TextField value=reason placeholder="Their bot lane's comfort pick" />
                            </Field>
                            <Button on_click=add>"Add"</Button>
                        </div>
                        <div class="flex gap-2">
                            <Button variant=ButtonVariant::Primary on_click=save>"Save priorities"</Button>
                            <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| editing.set(false))>"Cancel"</Button>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="flex flex-col gap-4">
                        <Suspense fallback=|| view! { <SkeletonGroup label="Loading ban priorities"><Skeleton shape=SkeletonShape::Line lines=3 /></SkeletonGroup> }>
                            {move || list.get().map(|r| match r {
                                Err(_) => view! { <Callout tone=CalloutTone::Danger>"Ban priorities did not load."</Callout> }.into_any(),
                                Ok(l) if l.is_empty() => view! {
                                    <EmptyState size=EmptySize::Compact message="No ban priorities yet. List the champions your team bans first, with a reason, and they appear here for every draft." />
                                }.into_any(),
                                Ok(l) => view! {
                                    <ol class="flex flex-col">
                                        {l.into_iter().map(|b| view! {
                                            <li class="flex items-center gap-3 border-b border-rule py-2 last:border-b-0">
                                                <span class="figures w-6 text-label text-muted">{b.rank + 1}</span>
                                                <ChampionAvatar name=b.champion.clone() size=AvatarSize::Sm decorative=true />
                                                <span class="text-label font-medium text-primary">{b.champion.clone()}</span>
                                                <span class="min-w-0 flex-1 truncate text-label text-secondary">{b.reason.unwrap_or_default()}</span>
                                            </li>
                                        }).collect_view()}
                                    </ol>
                                }.into_any(),
                            })}
                        </Suspense>
                        <div><Button on_click=start>"Edit priorities"</Button></div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

/// Champions that keep appearing in the same step of the team's drafts.
#[component]
fn Tendencies() -> impl IntoView {
    let data = Resource::new(|| (), |_| get_draft_tendency_data());
    view! {
        <Suspense fallback=|| view! { <SkeletonGroup label="Loading tendencies"><Skeleton shape=SkeletonShape::Line lines=3 /></SkeletonGroup> }>
            {move || data.get().map(|r| match r {
                Err(_) => view! { <Callout tone=CalloutTone::Danger>"Tendencies did not load."</Callout> }.into_any(),
                Ok(rows) => {
                    // Per champion: total appearances and the most common step.
                    let mut by_champ: HashMap<String, (i32, i32, i32)> = HashMap::new();
                    for t in &rows {
                        let e = by_champ.entry(t.champion.clone()).or_insert((0, -1, 0));
                        e.0 += t.count;
                        if t.count > e.2 {
                            e.1 = t.order;
                            e.2 = t.count;
                        }
                    }
                    let mut list: Vec<(String, i32, i32, i32)> = by_champ
                        .into_iter()
                        .filter(|(_, (total, _, _))| *total >= 2)
                        .map(|(c, (total, order, count))| (c, total, order, count))
                        .collect();
                    list.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
                    if list.is_empty() {
                        return view! {
                            <EmptyState size=EmptySize::Compact message="Tendencies appear once a champion shows up in two or more saved drafts: which step it usually takes, and whether that is predictable." />
                        }.into_any();
                    }
                    let predictable: Vec<String> = list
                        .iter()
                        .filter(|(_, total, _, count)| *total >= 3 && (*count as f64 / *total as f64) >= 0.7)
                        .map(|(c, total, order, count)| {
                            let step = TOURNAMENT_ORDER.get(*order as usize).map(|s| s.describe()).unwrap_or_default();
                            format!("{c}: {count} of {total} drafts as {step}")
                        })
                        .collect();
                    view! {
                        <div class="flex flex-col gap-4">
                            {(!predictable.is_empty()).then(|| view! {
                                <Callout tone=CalloutTone::Warning title="Predictable patterns">
                                    <ul class="flex flex-col gap-0.5">
                                        {predictable.into_iter().map(|p| view! { <li>{p}</li> }).collect_view()}
                                    </ul>
                                </Callout>
                            })}
                            <Table caption="Champions by appearances in saved drafts">
                                <thead>
                                    <tr>
                                        <Th>"Champion"</Th>
                                        <Th numeric=true>"Drafts"</Th>
                                        <Th>"Most common step"</Th>
                                        <Th numeric=true>"Share"</Th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {list.into_iter().map(|(c, total, order, count)| {
                                        let step = TOURNAMENT_ORDER.get(order as usize).map(|s| s.describe()).unwrap_or_default();
                                        let share = (count as f64 / total.max(1) as f64 * 100.0).round() as i32;
                                        view! {
                                            <Tr>
                                                <Td header=true>
                                                    <span class="inline-flex items-center gap-2">
                                                        <ChampionAvatar name=c.clone() size=AvatarSize::Xs decorative=true />{c}
                                                    </span>
                                                </Td>
                                                <Td numeric=true>{total}</Td>
                                                <Td>{step}</Td>
                                                <Td numeric=true>{format!("{share}%")}</Td>
                                            </Tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </Table>
                        </div>
                    }.into_any()
                }
            })}
        </Suspense>
    }
}

/// Win rates by side, composition tag and first pick (from reviewed games).
#[component]
fn Outcomes() -> impl IntoView {
    let data = Resource::new(|| (), |_| get_draft_analytics());
    let pct = |w: i32, g: i32| {
        if g > 0 {
            format!("{:.0}", w as f64 / g as f64 * 100.0)
        } else {
            "0".to_string()
        }
    };
    view! {
        <Suspense fallback=|| view! { <SkeletonGroup label="Loading outcomes"><Skeleton shape=SkeletonShape::Line lines=3 /></SkeletonGroup> }>
            {move || data.get().map(|r| match r {
                Err(_) => view! { <Callout tone=CalloutTone::Danger>"Outcomes did not load."</Callout> }.into_any(),
                Ok(d) if d.blue_games + d.red_games == 0 && d.tag_stats.is_empty() && d.first_pick_stats.is_empty() => view! {
                    <EmptyState size=EmptySize::Compact message="Outcomes appear once post-game reviews are linked to drafts: win rates by side, composition and first pick." />
                }.into_any(),
                Ok(d) => {
                    let blue = if d.blue_games > 0 {
                        view! { <StatReadout label="Blue side" value=pct(d.blue_wins, d.blue_games) unit=format!("% won, {}-{}", d.blue_wins, d.blue_games - d.blue_wins) /> }.into_any()
                    } else {
                        view! { <StatReadout label="Blue side" value="" empty_reason="No reviewed games on blue yet" /> }.into_any()
                    };
                    let red = if d.red_games > 0 {
                        view! { <StatReadout label="Red side" value=pct(d.red_wins, d.red_games) unit=format!("% won, {}-{}", d.red_wins, d.red_games - d.red_wins) /> }.into_any()
                    } else {
                        view! { <StatReadout label="Red side" value="" empty_reason="No reviewed games on red yet" /> }.into_any()
                    };
                    let table = |caption: &'static str, first: &'static str, rows: Vec<(String, i32, i32)>| {
                        (!rows.is_empty()).then(|| view! {
                            <Table caption=caption>
                                <thead><tr><Th>{first}</Th><Th numeric=true>"Games"</Th><Th numeric=true>"Won"</Th><Th numeric=true>"Win rate"</Th></tr></thead>
                                <tbody>
                                    {rows.into_iter().map(|(k, g, w)| view! {
                                        <Tr>
                                            <Td header=true>{k}</Td>
                                            <Td numeric=true>{g}</Td>
                                            <Td numeric=true>{w}</Td>
                                            <Td numeric=true>{format!("{}%", if g > 0 { (w as f64 / g as f64 * 100.0).round() as i32 } else { 0 })}</Td>
                                        </Tr>
                                    }).collect_view()}
                                </tbody>
                            </Table>
                        })
                    };
                    view! {
                        <div class="flex flex-col gap-6">
                            <div class="flex flex-wrap gap-x-12 gap-y-4">{blue}{red}</div>
                            {table("Win rate by composition tag", "Composition", d.tag_stats)}
                            {table("Win rate by first pick", "First pick", d.first_pick_stats)}
                        </div>
                    }.into_any()
                }
            })}
        </Suspense>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::champion::{Champion, ChampionPoolEntry};
    use std::collections::HashMap;

    #[test]
    fn stored_opponent_names_map_to_their_dossier() {
        let opp = |id: &str, name: &str| Opponent {
            id: Some(id.into()),
            name: name.into(),
            team_id: "team:t".into(),
            notes: None,
        };
        let list = vec![
            opp("opponent:a", "Riverside Rooks"),
            opp("opponent:b", "Twins"),
            opp("opponent:c", "twins"),
        ];
        assert_eq!(opponent_ref("opponent:a", &list), "opponent:a");
        assert_eq!(opponent_ref("riverside rooks", &list), "opponent:a");
        // Ambiguous or unknown names stay as written.
        assert_eq!(opponent_ref("Twins", &list), "Twins");
        assert_eq!(opponent_ref("Nobody", &list), "Nobody");
        assert_eq!(opponent_ref("", &list), "");
    }

    fn make_champion(id: &str, tags: Vec<&str>) -> Champion {
        Champion {
            id: id.to_string(),
            name: id.to_string(),
            title: String::new(),
            tags: tags.into_iter().map(String::from).collect(),
            image_full: String::new(),
        }
    }

    fn make_pool_entry(user_id: &str, champion: &str, role: &str) -> ChampionPoolEntry {
        ChampionPoolEntry {
            id: None,
            user_id: user_id.to_string(),
            champion: champion.to_string(),
            role: role.to_string(),
            tier: "S".to_string(),
            notes: None,
            comfort_level: None,
            meta_tag: None,
        }
    }

    #[test]
    fn champion_in_pool_no_warning() {
        // Player has Darius in pool -> picking Darius = no warning
        let mut champs = HashMap::new();
        champs.insert(
            "Darius".to_string(),
            make_champion("Darius", vec!["Fighter"]),
        );
        let pools = vec![(
            "TopPlayer".to_string(),
            "top".to_string(),
            vec![make_pool_entry("user:u1", "Darius", "top")],
        )];
        let mut slots: Vec<Option<String>> = vec![None; 20];
        slots[6] = Some("Darius".to_string()); // Blue pick 1 = top
        let result = compute_slot_warnings(&slots, &pools, &champs, "blue", &HashMap::new());
        assert!(
            result[6].is_none(),
            "Champion in pool should not trigger warning"
        );
    }

    #[test]
    fn champion_not_in_pool_with_class_gap_warns() {
        // Player has only Fighters, picks a Mage (not in pool + class gap)
        let mut champs = HashMap::new();
        champs.insert(
            "Darius".to_string(),
            make_champion("Darius", vec!["Fighter"]),
        );
        champs.insert("Syndra".to_string(), make_champion("Syndra", vec!["Mage"]));
        let pools = vec![(
            "TopPlayer".to_string(),
            "top".to_string(),
            vec![make_pool_entry("user:u1", "Darius", "top")],
        )];
        let mut slots: Vec<Option<String>> = vec![None; 20];
        slots[6] = Some("Syndra".to_string()); // Blue pick 1 = top
        let result = compute_slot_warnings(&slots, &pools, &champs, "blue", &HashMap::new());
        assert!(
            result[6].is_some(),
            "Not-in-pool champion with class gap should warn"
        );
        let (name, detail) = result[6].as_ref().unwrap();
        assert_eq!(name, "TopPlayer");
        assert!(detail.contains("Mage"), "Should mention the missing class");
    }

    #[test]
    fn champion_not_in_pool_without_class_gap_no_warning() {
        // Player has Darius (Fighter) in pool, picks Garen (also Fighter, not in pool)
        // No class gap because Fighter is already covered
        let mut champs = HashMap::new();
        champs.insert(
            "Darius".to_string(),
            make_champion("Darius", vec!["Fighter"]),
        );
        champs.insert("Garen".to_string(), make_champion("Garen", vec!["Fighter"]));
        let pools = vec![(
            "TopPlayer".to_string(),
            "top".to_string(),
            vec![make_pool_entry("user:u1", "Darius", "top")],
        )];
        let mut slots: Vec<Option<String>> = vec![None; 20];
        slots[6] = Some("Garen".to_string());
        let result = compute_slot_warnings(&slots, &pools, &champs, "blue", &HashMap::new());
        assert!(
            result[6].is_none(),
            "No class gap = no warning even if champion not in pool"
        );
    }

    #[test]
    fn wrong_side_slot_no_warning() {
        // Blue side: red pick slots should never have warnings
        let mut champs = HashMap::new();
        champs.insert("Syndra".to_string(), make_champion("Syndra", vec!["Mage"]));
        let pools = vec![("TopPlayer".to_string(), "top".to_string(), vec![])];
        let mut slots: Vec<Option<String>> = vec![None; 20];
        slots[7] = Some("Syndra".to_string()); // Red pick 1 (not our side when blue)
        let result = compute_slot_warnings(&slots, &pools, &champs, "blue", &HashMap::new());
        assert!(
            result[7].is_none(),
            "Opponent-side slots should not have warnings"
        );
    }

    #[test]
    fn side_switch_changes_checked_slots() {
        let mut champs = HashMap::new();
        champs.insert("Syndra".to_string(), make_champion("Syndra", vec!["Mage"]));
        champs.insert(
            "Darius".to_string(),
            make_champion("Darius", vec!["Fighter"]),
        );
        let pools = vec![(
            "TopPlayer".to_string(),
            "top".to_string(),
            vec![make_pool_entry("user:u1", "Darius", "top")],
        )];
        let mut slots: Vec<Option<String>> = vec![None; 20];
        slots[6] = Some("Syndra".to_string()); // Blue pick 1
        slots[7] = Some("Syndra".to_string()); // Red pick 1
        let blue_result = compute_slot_warnings(&slots, &pools, &champs, "blue", &HashMap::new());
        let red_result = compute_slot_warnings(&slots, &pools, &champs, "red", &HashMap::new());
        // Blue side: slot 6 is ours (may warn), slot 7 is theirs (no warn)
        // Red side: slot 7 is ours (may warn), slot 6 is theirs (no warn)
        assert!(blue_result[7].is_none(), "Slot 7 is opponent on blue side");
        assert!(red_result[6].is_none(), "Slot 6 is opponent on red side");
    }

    #[test]
    fn empty_inputs_no_panic() {
        let result = compute_slot_warnings(
            &vec![None; 20],
            &[],
            &HashMap::new(),
            "blue",
            &HashMap::new(),
        );
        assert_eq!(result.len(), 20);
        assert!(result.iter().all(|w| w.is_none()));
    }
}
