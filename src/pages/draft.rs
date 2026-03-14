use crate::components::champion_picker::ChampionPicker;
use crate::components::draft_board::{slot_meta, DraftBoard};
use crate::components::ui::ErrorBanner;
use crate::models::champion::{Champion, ChampionNote, ChampionStatSummary};
use crate::models::draft::{BanPriority, Draft, DraftAction};
use crate::models::opponent::OpponentPlayer;
use crate::models::series::Series;
use crate::models::team::Team;
use crate::pages::game_plan::check_draft_has_game_plan;
use leptos::prelude::*;
use std::collections::HashMap;

#[server]
pub async fn get_champions() -> Result<Vec<Champion>, ServerFnError> {
    use crate::server::data_dragon;
    data_dragon::fetch_champions()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
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
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let actions: Vec<DraftAction> = serde_json::from_str(&actions_json)
        .map_err(|e| ServerFnError::new(format!("Invalid actions JSON: {e}")))?;
    let comments: Vec<String> = serde_json::from_str(&comments_json)
        .map_err(|e| ServerFnError::new(format!("Invalid comments JSON: {e}")))?;
    let tags: Vec<String> = serde_json::from_str(&tags_json)
        .map_err(|e| ServerFnError::new(format!("Invalid tags JSON: {e}")))?;

    let resolved_team_id = match team_id.filter(|s| !s.is_empty()) {
        Some(tid) => tid,
        None => db::get_user_team_id(&db, &user.id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .ok_or_else(|| ServerFnError::new("You must be in a team to create a draft"))?,
    };

    db::save_draft(
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
    .map_err(|e| ServerFnError::new(e.to_string()))
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
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let actions: Vec<DraftAction> = serde_json::from_str(&actions_json)
        .map_err(|e| ServerFnError::new(format!("Invalid actions JSON: {e}")))?;
    let comments: Vec<String> = serde_json::from_str(&comments_json)
        .map_err(|e| ServerFnError::new(format!("Invalid comments JSON: {e}")))?;
    let tags: Vec<String> = serde_json::from_str(&tags_json)
        .map_err(|e| ServerFnError::new(format!("Invalid tags JSON: {e}")))?;

    db::update_draft(
        &db,
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
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_drafts() -> Result<Vec<Draft>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_drafts(&db, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_user_teams() -> Result<Vec<Team>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::get_user_teams(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_ban_priorities() -> Result<Vec<BanPriority>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::get_ban_priorities(&db, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn save_ban_priorities(priorities_json: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    let priorities: Vec<BanPriority> = serde_json::from_str(&priorities_json)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;

    db::set_ban_priorities(&db, &team_id, priorities)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_series_fn() -> Result<Vec<crate::models::series::Series>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_series(&db, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_series_fn(
    name: String,
    opponent_name: Option<String>,
    format: String,
    is_fearless: bool,
) -> Result<String, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

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
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_fearless_champions(
    series_id: String,
    exclude_draft: Option<String>,
) -> Result<Vec<String>, ServerFnError> {
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::get_fearless_used_champions(&db, &series_id, exclude_draft.as_deref())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Get champion pool entries for all starters on the user's team.
/// Returns Vec<(username, role, pool_entries)>.
#[server]
pub async fn get_team_pools(
) -> Result<Vec<(String, String, Vec<crate::models::champion::ChampionPoolEntry>)>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let _team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    let (_, members) = match db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
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
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let (_, members) = match db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
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
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    match db::get_opponent(&db, &opponent_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
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
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::get_team_matchup_notes(&db, &team_id, &champion)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
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
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    let raw = db::get_draft_tendencies(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

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
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(DraftAnalytics::default()),
    };

    let data = db::get_draft_outcome_stats(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

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
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    let plans = db::list_game_plans(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for plan in &plans {
        if let Some(ref draft_id) = plan.draft_id {
            *counts.entry(draft_id.clone()).or_default() += 1;
        }
    }

    Ok(counts.into_iter().collect())
}

fn build_actions(slots: Vec<Option<String>>, slot_comments: &[Option<String>]) -> Vec<DraftAction> {
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

fn tier_badge_class(tier: &str) -> &'static str {
    match tier {
        "S+" => "bg-purple-500 text-white",
        "S" => "bg-accent text-accent-contrast",
        "A" => "bg-green-500 text-primary",
        "B" => "bg-blue-500 text-white",
        "C" => "bg-orange-500 text-primary",
        "D" => "bg-red-600 text-white",
        _ => "bg-overlay-strong text-secondary",
    }
}

const TIERS: &[&str] = &["S+", "S", "A", "B", "C", "D"];

#[component]
pub fn DraftPage() -> impl IntoView {
    // Auth redirect
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    let (draft_name, set_draft_name) = signal(String::new());
    let (opponent, set_opponent) = signal(String::new());
    let (selected_team_id, set_selected_team_id) = signal(String::new());
    let (rating, set_rating) = signal(Option::<String>::None);
    let (our_side, set_our_side) = signal("blue".to_string());
    let (draft_slots, set_draft_slots) = signal(vec![None::<String>; 20]);
    let (active_slot, set_active_slot) = signal(Some(0_usize));
    let (highlighted_slot, set_highlighted_slot) = signal(Option::<usize>::None);
    let (comments, set_comments) = signal(Vec::<String>::new());
    let (comment_input, set_comment_input) = signal(String::new());
    let (save_result, set_save_result) = signal(Option::<String>::None);
    let (loaded_draft_id, set_loaded_draft_id) = signal(Option::<String>::None);
    // Per-slot rationale comments (Phase 1)
    let (slot_comments, set_slot_comments) = signal(vec![None::<String>; 20]);
    let (slot_comment_input, set_slot_comment_input) = signal(String::new());
    // Composition tags + win conditions (Phase 2)
    let (tags, set_tags) = signal(Vec::<String>::new());
    let (win_conditions, set_win_conditions) = signal(String::new());
    let (watch_out, set_watch_out) = signal(String::new());
    // Tag filter for saved drafts list
    let (filter_tag, set_filter_tag) = signal(String::new());
    // Series mode (Phase 3 - Fearless Draft)
    let (series_panel_open, set_series_panel_open) = signal(false);
    let (active_series, set_active_series) = signal(Option::<Series>::None);
    let (active_game_number, set_active_game_number) = signal(1_i32);
    let (fearless_used, set_fearless_used) = signal(Vec::<String>::new());
    let (series_name_input, set_series_name_input) = signal(String::new());
    let (series_opponent_input, set_series_opponent_input) = signal(String::new());
    let (series_format_input, set_series_format_input) = signal("bo3".to_string());
    let (series_fearless_input, set_series_fearless_input) = signal(false);
    let (series_status, set_series_status) = signal(Option::<String>::None);
    let series_resource = Resource::new(|| (), |_| list_series_fn());

    // Ban priorities (Phase 4)
    let ban_priorities = Resource::new(|| (), |_| get_ban_priorities());
    let (ban_panel_open, set_ban_panel_open) = signal(false);
    let (editing_bans, set_editing_bans) = signal(false);
    let (ban_edit_list, set_ban_edit_list) = signal(Vec::<BanPriority>::new());
    let (ban_new_champ, set_ban_new_champ) = signal(String::new());
    let (ban_new_reason, set_ban_new_reason) = signal(String::new());
    let (ban_status, set_ban_status) = signal(Option::<String>::None);

    // Intel sidebar (Phase 5 — Pool Awareness & Matchup Surfacing)
    let (intel_open, set_intel_open) = signal(false);
    let (intel_tab, set_intel_tab) = signal("pools".to_string());
    let (selected_opponent_id, set_selected_opponent_id) = signal(String::new());
    let (matchup_champion, set_matchup_champion) = signal(Option::<String>::None);

    let team_pools = Resource::new(|| (), |_| get_team_pools());
    let team_stats = Resource::new(|| (), |_| get_team_champion_stats());
    let opponents_list = Resource::new(|| (), |_| crate::pages::opponents::get_opponents());
    let opponent_players = Resource::new(
        move || selected_opponent_id.get(),
        move |opp_id| async move {
            if opp_id.is_empty() {
                Ok(Vec::<OpponentPlayer>::new())
            } else {
                get_opponent_intel(opp_id).await
            }
        },
    );
    let matchup_notes = Resource::new(
        move || matchup_champion.get(),
        move |champ_opt| async move {
            match champ_opt {
                Some(champ) if !champ.is_empty() => get_matchup_notes_for_champion(champ).await,
                _ => Ok(Vec::<(String, ChampionNote)>::new()),
            }
        },
    );

    let champions_resource = Resource::new(|| (), |_| get_champions());
    let drafts = Resource::new(|| (), |_| list_drafts());
    let teams_resource = Resource::new(|| (), |_| list_user_teams());

    // Analytics resources (Phase 7a + 7c)
    let (tendencies_open, set_tendencies_open) = signal(false);
    let (analytics_open, set_analytics_open) = signal(false);
    let tendency_data = Resource::new(|| (), |_| get_draft_tendency_data());
    let analytics_data = Resource::new(|| (), |_| get_draft_analytics());

    // Pipeline CTAs: game plan count badges per draft
    let game_plan_counts = Resource::new(|| (), |_| get_draft_game_plan_counts());

    // Pipeline CTAs: duplicate prompt (draft_id, existing_plan_id) when "Prep for This Draft"
    // detects an existing game plan
    let (duplicate_prompt, set_duplicate_prompt) = signal(Option::<(String, String)>::None);
    let (cta_loading, set_cta_loading) = signal(false);
    let (cta_status, set_cta_status) = signal(Option::<String>::None);

    // URL param auto-load: ?draft_id=X deep-links to a specific draft
    use leptos_router::hooks::use_query_map;
    let query = use_query_map();
    let (url_draft_loaded, set_url_draft_loaded) = signal(false);

    // Auto-select first team when resource loads
    Effect::new(move |_| {
        if let Some(Ok(teams)) = teams_resource.get() {
            if selected_team_id.get_untracked().is_empty() {
                if let Some(first) = teams.first() {
                    set_selected_team_id.set(first.id.clone().unwrap_or_default());
                }
            }
        }
    });

    // URL param auto-load: when ?draft_id=X is present, load that draft once the draft list resolves
    Effect::new(move |_| {
        let param_id = query.read().get("draft_id");
        let Some(target_id) = param_id else { return };
        if url_draft_loaded.get_untracked() {
            return;
        }
        if let Some(Ok(list)) = drafts.get() {
            if let Some(d) = list.iter().find(|d| d.id.as_deref() == Some(&target_id)) {
                let d_id = d.id.clone();
                let d_name = d.name.clone();
                let d_opp = d.opponent.clone().unwrap_or_default();
                let d_comments = d.comments.clone();
                let d_actions = d.actions.clone();
                let d_team_id = d.team_id.clone();
                let d_rating = d.rating.clone();
                let d_our_side = d.our_side.clone();
                let d_tags = d.tags.clone();
                let d_win_conditions = d.win_conditions.clone().unwrap_or_default();
                let d_watch_out = d.watch_out.clone().unwrap_or_default();
                let d_game_number = d.game_number;

                set_loaded_draft_id.set(d_id);
                set_draft_name.set(d_name);
                set_opponent.set(d_opp);
                set_selected_team_id.set(d_team_id);
                set_rating.set(d_rating);
                set_our_side.set(d_our_side);
                set_comments.set(d_comments);
                set_tags.set(d_tags);
                set_win_conditions.set(d_win_conditions);
                set_watch_out.set(d_watch_out);
                set_save_result.set(None);
                set_highlighted_slot.set(None);
                let mut slots = vec![None::<String>; 20];
                let mut sc = vec![None::<String>; 20];
                for action in &d_actions {
                    let o = action.order as usize;
                    if o < 20 {
                        slots[o] = Some(action.champion.clone());
                        sc[o] = action.comment.clone();
                    }
                }
                let next = (0..20).find(|&i| slots[i].is_none());
                set_draft_slots.set(slots);
                set_slot_comments.set(sc);
                set_slot_comment_input.set(String::new());
                set_active_slot.set(next);
                if let Some(gn) = d_game_number {
                    set_active_game_number.set(gn);
                }
                set_url_draft_loaded.set(true);
            }
        }
    });

    let used_champions = move || {
        let mut used: Vec<String> = draft_slots
            .get()
            .into_iter()
            .flatten()
            .collect();
        // In fearless mode, also include champions used in prior series games
        for champ in fearless_used.get() {
            if !used.contains(&champ) {
                used.push(champ);
            }
        }
        used
    };

    let fill_slot = move |slot_idx: usize, champion_name: String| {
        let already_used = draft_slots
            .get_untracked()
            .iter()
            .any(|s| s.as_deref() == Some(&champion_name));
        if already_used {
            return;
        }
        set_draft_slots.update(|s| s[slot_idx] = Some(champion_name));
        set_highlighted_slot.set(None);
        let updated = draft_slots.get_untracked();
        let next = (0..20).find(|&i| updated[i].is_none());
        set_active_slot.set(next);
    };

    let on_champion_select = Callback::new(move |champ: Champion| {
        if let Some(slot) = active_slot.get_untracked() {
            fill_slot(slot, champ.name);
        }
    });

    let on_slot_drop = Callback::new(move |(slot_idx, name): (usize, String)| {
        fill_slot(slot_idx, name);
    });

    let on_slot_click = Callback::new(move |slot_idx: usize| {
        let slots = draft_slots.get_untracked();
        if slots.get(slot_idx).and_then(|s| s.as_ref()).is_some() {
            let currently_highlighted = highlighted_slot.get_untracked();
            if currently_highlighted == Some(slot_idx) {
                // Second click: set as active_slot for champion replacement
                set_active_slot.set(Some(slot_idx));
            } else {
                // First click: just highlight, set as active
                set_highlighted_slot.set(Some(slot_idx));
                set_active_slot.set(Some(slot_idx));
            }
        } else {
            set_highlighted_slot.set(None);
            set_active_slot.update(|a| {
                *a = if *a == Some(slot_idx) {
                    None
                } else {
                    Some(slot_idx)
                };
            });
        }
    });

    let on_slot_clear = Callback::new(move |slot_idx: usize| {
        set_draft_slots.update(|s| s[slot_idx] = None);
        set_highlighted_slot.set(None);
        set_active_slot.set(Some(slot_idx));
    });

    let phase_label = move || match active_slot.get() {
        Some(0..=5) => "Phase 1 — Bans",
        Some(6..=11) => "Phase 1 — Picks",
        Some(12..=15) => "Phase 2 — Bans",
        Some(16..=19) => "Phase 2 — Picks",
        None => "Draft Complete",
        _ => "",
    };

    let active_slot_label = move || {
        active_slot.get().map(|i| {
            let (side, _, label) = slot_meta(i);
            let side_cap = if side == "blue" { "Blue" } else { "Red" };
            format!("Selecting for: {side_cap} {label}")
        })
    };

    let do_save = move |_| {
        let name = draft_name.get_untracked();
        if name.trim().is_empty() {
            set_save_result.set(Some("Give this draft a name before saving.".into()));
            return;
        }
        let opp = opponent.get_untracked();
        let tid = selected_team_id.get_untracked();
        let rate = rating.get_untracked();
        let side = our_side.get_untracked();
        let sc = slot_comments.get_untracked();
        let actions = build_actions(draft_slots.get_untracked(), &sc);
        let acts_json = serde_json::to_string(&actions).unwrap_or_default();
        let cmts_json = serde_json::to_string(&comments.get_untracked()).unwrap_or_default();
        let tags_json = serde_json::to_string(&tags.get_untracked()).unwrap_or_default();
        let wc = { let s = win_conditions.get_untracked(); if s.is_empty() { None } else { Some(s) } };
        let wo = { let s = watch_out.get_untracked(); if s.is_empty() { None } else { Some(s) } };
        let existing_id = loaded_draft_id.get_untracked();
        let s_id = active_series.get_untracked().and_then(|s| s.id.clone());
        let g_num = if s_id.is_some() { Some(active_game_number.get_untracked()) } else { None };

        leptos::task::spawn_local(async move {
            let opp_opt = if opp.is_empty() { None } else { Some(opp) };
            let team_opt = if tid.is_empty() { None } else { Some(tid) };

            if let Some(draft_id) = existing_id {
                match update_draft(
                    draft_id, name, opp_opt, acts_json, cmts_json, rate, Some(side),
                    tags_json, wc, wo, s_id, g_num,
                )
                .await
                {
                    Ok(_) => {
                        set_save_result.set(Some("Updated!".into()));
                        drafts.refetch();
                    }
                    Err(e) => set_save_result.set(Some(format!("Error: {e}"))),
                }
            } else {
                match save_draft(
                    name, opp_opt, team_opt, acts_json, cmts_json, rate, Some(side),
                    tags_json, wc, wo, s_id, g_num,
                )
                .await
                {
                    Ok(id) => {
                        set_save_result.set(Some("Saved!".into()));
                        set_loaded_draft_id.set(Some(id));
                        drafts.refetch();
                    }
                    Err(e) => set_save_result.set(Some(format!("Error: {e}"))),
                }
            }
        });
    };

    // Sync slot_comment_input and matchup champion when highlighted_slot changes
    Effect::new(move |_| {
        let hl = highlighted_slot.get();
        if let Some(idx) = hl {
            let sc = slot_comments.get_untracked();
            set_slot_comment_input.set(sc.get(idx).cloned().flatten().unwrap_or_default());
            // Update matchup champion for Intel sidebar
            let slots = draft_slots.get_untracked();
            let champ = slots.get(idx).and_then(|s| s.clone());
            set_matchup_champion.set(champ);
        } else {
            set_slot_comment_input.set(String::new());
            set_matchup_champion.set(None);
        }
    });

    // Auto-save timer handle (only used in hydrate/WASM builds)
    #[allow(unused_variables)]
    let auto_save_timer: RwSignal<Option<i32>> = RwSignal::new(None);
    let (auto_save_status, set_auto_save_status) = signal(""); // "", "unsaved", "saved"

    #[allow(unused_variables)]
    Effect::new(move |_| {
        // Eagerly track + capture ALL content signals (CLAUDE.md rule 54)
        let slots_val = draft_slots.get();
        let comments_val = comments.get();
        let name = draft_name.get();
        let existing_id = loaded_draft_id.get();
        let opp = opponent.get();
        let rate = rating.get();
        let side = our_side.get();
        let sc = slot_comments.get();
        let tags_val = tags.get();
        let wc_val = win_conditions.get();
        let wo_val = watch_out.get();

        // Only auto-save if we have a name AND it's an existing draft
        if name.trim().is_empty() || existing_id.is_none() {
            return;
        }

        // Cancel pending timer
        #[cfg(feature = "hydrate")]
        if let Some(timer_id) = auto_save_timer.get_untracked() {
            if let Some(win) = web_sys::window() {
                win.clear_timeout_with_handle(timer_id);
            }
        }

        set_auto_save_status.set("unsaved");

        // Schedule new 2s auto-save
        #[cfg(feature = "hydrate")]
        {
            // Pre-compute all values before the closure
            let actions = build_actions(slots_val, &sc);
            let acts_json = serde_json::to_string(&actions).unwrap_or_default();
            let cmts_json = serde_json::to_string(&comments_val).unwrap_or_default();
            let tags_json = serde_json::to_string(&tags_val).unwrap_or_default();
            let wc = if wc_val.is_empty() { None } else { Some(wc_val) };
            let wo = if wo_val.is_empty() { None } else { Some(wo_val) };
            let opp_opt = if opp.is_empty() { None } else { Some(opp) };
            let s_id = active_series.get_untracked().and_then(|s| s.id.clone());
            let g_num = if s_id.is_some() { Some(active_game_number.get_untracked()) } else { None };

            use wasm_bindgen::prelude::*;
            let cb = Closure::once(move || {
                if let Some(draft_id) = existing_id {
                    leptos::task::spawn_local(async move {
                        let _ = update_draft(
                            draft_id, name, opp_opt, acts_json, cmts_json, rate, Some(side),
                            tags_json, wc, wo, s_id, g_num,
                        )
                        .await;
                        set_auto_save_status.set("saved");
                        drafts.refetch();
                    });
                }
            });
            if let Some(win) = web_sys::window() {
                if let Ok(timer_id) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    2000,
                ) {
                    auto_save_timer.set(Some(timer_id));
                }
            }
            cb.forget();
        }
    });

    // Role ordering for pool display
    let role_order = |r: &str| -> u8 {
        match r {
            "top" => 0,
            "jungle" | "jng" => 1,
            "mid" | "middle" => 2,
            "bot" | "adc" => 3,
            "support" | "sup" => 4,
            _ => 5,
        }
    };

    view! {
        <div class="max-w-[1600px] mx-auto py-8 px-6 flex flex-col gap-6">
            <div class="flex items-start justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-primary">"Draft Planner"</h1>
                    <p class="text-accent font-medium mt-1">{phase_label}</p>
                    {move || loaded_draft_id.get().map(|_| {
                        let status = auto_save_status.get();
                        let (cls, text) = match status {
                            "saved" => ("text-green-400 text-sm mt-0.5", "✓ Saved"),
                            "unsaved" => ("text-amber-400 text-sm mt-0.5", "● Unsaved changes"),
                            _ => ("text-muted text-sm mt-0.5", "Editing saved draft"),
                        };
                        view! { <p class=cls>{text}</p> }
                    })}
                </div>
                <button
                    class=move || if intel_open.get() {
                        "flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-accent text-accent-contrast transition-colors cursor-pointer"
                    } else {
                        "flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-elevated border border-divider text-secondary hover:text-primary hover:border-accent transition-colors cursor-pointer"
                    }
                    on:click=move |_| set_intel_open.update(|v| *v = !*v)
                >
                    <span>"Intel"</span>
                    <span>{move || if intel_open.get() { "▼" } else { "▶" }}</span>
                </button>
            </div>

            // Header form
            <div class="bg-elevated border border-divider rounded-lg p-4 flex flex-col gap-4">
                <div class="grid grid-cols-3 gap-4">
                    // Draft Name
                    <div>
                        <label class="block text-secondary text-sm mb-1">"Draft Name"</label>
                        <input
                            type="text"
                            prop:value=move || draft_name.get()
                            class="w-full bg-overlay border border-outline rounded px-3 py-2 text-primary focus:outline-none focus:border-accent"
                            on:input=move |ev| set_draft_name.set(event_target_value(&ev))
                        />
                    </div>
                    // Team selection
                    <div>
                        <label class="block text-secondary text-sm mb-1">"Team"</label>
                        <Suspense fallback=|| view! { <div class="h-9 bg-overlay rounded animate-pulse"></div> }>
                            {move || teams_resource.get().map(|result| match result {
                                Ok(teams) if teams.is_empty() => view! {
                                    <p class="text-dimmed text-sm py-2">"Not part of a team yet."</p>
                                }.into_any(),
                                Ok(teams) => view! {
                                    <select
                                        prop:value=move || selected_team_id.get()
                                        on:change=move |ev| set_selected_team_id.set(event_target_value(&ev))
                                        class="w-full bg-overlay border border-outline rounded px-3 py-2 text-primary focus:outline-none focus:border-accent"
                                    >
                                        {teams.into_iter().map(|t| {
                                            let id = t.id.clone().unwrap_or_default();
                                            let name = t.name.clone();
                                            view! { <option value=id>{name}</option> }
                                        }).collect_view()}
                                    </select>
                                }.into_any(),
                                Err(_) => view! {
                                    <p class="text-red-400 text-sm py-2">"Failed to load teams."</p>
                                }.into_any(),
                            })}
                        </Suspense>
                    </div>
                    // Opponent
                    <div>
                        <label class="block text-secondary text-sm mb-1">"Opponent (optional)"</label>
                        <input
                            type="text"
                            prop:value=move || opponent.get()
                            class="w-full bg-overlay border border-outline rounded px-3 py-2 text-primary focus:outline-none focus:border-accent"
                            on:input=move |ev| set_opponent.set(event_target_value(&ev))
                        />
                    </div>
                </div>
                // Our Side toggle
                <div class="flex items-center gap-4">
                    <label class="text-secondary text-sm">"Our Side"</label>
                    <div class="flex gap-1">
                        <button
                            class=move || if our_side.get() == "blue" {
                                "px-3 py-1 rounded text-sm font-medium bg-blue-500 text-white cursor-pointer"
                            } else {
                                "px-3 py-1 rounded text-sm font-medium bg-overlay text-muted hover:bg-overlay-strong transition-colors cursor-pointer"
                            }
                            on:click=move |_| set_our_side.set("blue".to_string())
                        >"Blue"</button>
                        <button
                            class=move || if our_side.get() == "red" {
                                "px-3 py-1 rounded text-sm font-medium bg-red-500 text-white cursor-pointer"
                            } else {
                                "px-3 py-1 rounded text-sm font-medium bg-overlay text-muted hover:bg-overlay-strong transition-colors cursor-pointer"
                            }
                            on:click=move |_| set_our_side.set("red".to_string())
                        >"Red"</button>
                    </div>
                </div>

                // Rating picker
                <div>
                    <label class="block text-secondary text-sm mb-2">"Rating"</label>
                    <div class="flex gap-1.5">
                        {TIERS.iter().map(|&tier| {
                            view! {
                                <button
                                    class=move || {
                                        let selected = rating.get().as_deref() == Some(tier);
                                        if selected {
                                            format!("rounded px-3 py-1 text-sm font-bold transition-colors {}", tier_badge_class(tier))
                                        } else {
                                            "rounded px-3 py-1 text-sm font-bold transition-colors bg-overlay hover:bg-overlay-strong text-muted".to_string()
                                        }
                                    }
                                    on:click=move |_| {
                                        let current = rating.get_untracked();
                                        set_rating.set(
                                            if current.as_deref() == Some(tier) { None }
                                            else { Some(tier.to_string()) }
                                        );
                                    }
                                >
                                    {tier}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>

                // Composition tags
                <div>
                    <label class="block text-secondary text-sm mb-2">"Composition Tags"</label>
                    <div class="flex flex-wrap gap-1.5">
                        {COMPOSITION_TAGS.iter().map(|&tag| {
                            let tag_str = tag.to_string();
                            let tag_for_class = tag_str.clone();
                            let tag_for_click = tag_str.clone();
                            view! {
                                <button
                                    class=move || {
                                        let selected = tags.get().contains(&tag_for_class);
                                        if selected {
                                            "rounded px-3 py-1 text-sm font-medium bg-accent text-accent-contrast transition-colors cursor-pointer"
                                        } else {
                                            "rounded px-3 py-1 text-sm font-medium bg-overlay text-muted hover:bg-overlay-strong transition-colors cursor-pointer"
                                        }
                                    }
                                    on:click=move |_| {
                                        let tag_val = tag_for_click.clone();
                                        set_tags.update(|t| {
                                            if let Some(pos) = t.iter().position(|x| x == &tag_val) {
                                                t.remove(pos);
                                            } else {
                                                t.push(tag_val);
                                            }
                                        });
                                    }
                                >
                                    {tag}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>

                // Win condition + watch out textareas
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-secondary text-sm mb-1">"How We Win"</label>
                        <textarea
                            rows="3"
                            placeholder="Win condition notes..."
                            class="w-full bg-overlay border border-outline rounded px-3 py-2 text-primary text-sm placeholder-gray-400 focus:outline-none focus:border-accent resize-none"
                            prop:value=move || win_conditions.get()
                            on:input=move |ev| set_win_conditions.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-secondary text-sm mb-1">"Watch Out For"</label>
                        <textarea
                            rows="3"
                            placeholder="Threats to be aware of..."
                            class="w-full bg-overlay border border-outline rounded px-3 py-2 text-primary text-sm placeholder-gray-400 focus:outline-none focus:border-accent resize-none"
                            prop:value=move || watch_out.get()
                            on:input=move |ev| set_watch_out.set(event_target_value(&ev))
                        />
                    </div>
                </div>
            </div>

            // Series Mode (Fearless Draft)
            <div class="bg-elevated border border-divider rounded-lg">
                <button
                    class="w-full flex items-center justify-between px-4 py-3 text-secondary text-sm font-medium hover:text-primary transition-colors"
                    on:click=move |_| set_series_panel_open.update(|v| *v = !*v)
                >
                    <span class="flex items-center gap-2">
                        {move || if active_series.get().is_some() {
                            view! { <span class="w-2 h-2 rounded-full bg-green-400"></span> }.into_any()
                        } else {
                            view! { <span class="w-2 h-2 rounded-full bg-overlay-strong"></span> }.into_any()
                        }}
                        "Series Mode"
                        {move || active_series.get().map(|s| {
                            let label = if s.is_fearless {
                                format!(" - {} (Fearless {})", s.name, s.format.to_uppercase())
                            } else {
                                format!(" - {} ({})", s.name, s.format.to_uppercase())
                            };
                            view! { <span class="text-accent">{label}</span> }
                        })}
                    </span>
                    <span class="text-muted">{move || if series_panel_open.get() { "▲" } else { "▼" }}</span>
                </button>

                {move || if series_panel_open.get() {
                    view! {
                        <div class="border-t border-divider px-4 py-4 flex flex-col gap-4">
                            // Active series header with game tabs
                            {move || if let Some(series) = active_series.get() {
                                let max_games = match series.format.as_str() {
                                    "bo3" => 3,
                                    "bo5" => 5,
                                    _ => 1,
                                };
                                let is_fearless = series.is_fearless;
                                view! {
                                    <div class="flex flex-col gap-3">
                                        <div class="flex items-center justify-between">
                                            <div class="flex items-center gap-3">
                                                <span class="text-primary font-medium">{series.name.clone()}</span>
                                                {if is_fearless {
                                                    view! { <span class="px-2 py-0.5 rounded text-xs font-bold bg-purple-500 text-white">"FEARLESS"</span> }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }}
                                                {series.opponent_name.clone().map(|opp| view! {
                                                    <span class="text-secondary text-sm">"vs " {opp}</span>
                                                })}
                                            </div>
                                            <button
                                                class="text-xs text-muted hover:text-red-400 transition-colors"
                                                on:click=move |_| {
                                                    set_active_series.set(None);
                                                    set_fearless_used.set(Vec::new());
                                                    set_active_game_number.set(1);
                                                }
                                            >"Exit Series"</button>
                                        </div>
                                        // Game tabs
                                        <div class="flex gap-1">
                                            {(1..=max_games).map(|g| {
                                                view! {
                                                    <button
                                                        class=move || if active_game_number.get() == g {
                                                            "px-4 py-1.5 rounded text-sm font-medium bg-accent text-accent-contrast cursor-pointer"
                                                        } else {
                                                            "px-4 py-1.5 rounded text-sm font-medium bg-overlay text-muted hover:bg-overlay-strong transition-colors cursor-pointer"
                                                        }
                                                        on:click=move |_| {
                                                            set_active_game_number.set(g);
                                                            // Clear current draft and load if a draft exists for this game
                                                            set_draft_slots.set(vec![None::<String>; 20]);
                                                            set_slot_comments.set(vec![None::<String>; 20]);
                                                            set_loaded_draft_id.set(None);
                                                            set_draft_name.set(String::new());
                                                            set_comments.set(Vec::new());
                                                            set_tags.set(Vec::new());
                                                            set_win_conditions.set(String::new());
                                                            set_watch_out.set(String::new());
                                                            set_rating.set(None);
                                                            set_active_slot.set(Some(0));

                                                            // Fetch fearless used champions for this game
                                                            let s = active_series.get_untracked();
                                                            if let Some(ref series) = s {
                                                                if series.is_fearless {
                                                                    if let Some(sid) = series.id.clone() {
                                                                        leptos::task::spawn_local(async move {
                                                                            if let Ok(champs) = get_fearless_champions(sid, None).await {
                                                                                set_fearless_used.set(champs);
                                                                            }
                                                                        });
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    >
                                                        {format!("Game {g}")}
                                                    </button>
                                                }
                                            }).collect_view()}
                                        </div>
                                        {move || if is_fearless && !fearless_used.get().is_empty() {
                                            view! {
                                                <div class="text-xs text-muted">
                                                    <span class="font-medium text-purple-400">"Fearless bans: "</span>
                                                    {fearless_used.get().join(", ")}
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>
                                }.into_any()
                            } else {
                                // No active series: show create form + existing series list
                                view! {
                                    <div class="flex flex-col gap-4">
                                        // Create new series
                                        <div class="flex flex-col gap-2">
                                            <span class="text-secondary text-sm font-medium">"Start New Series"</span>
                                            <div class="grid grid-cols-4 gap-3">
                                                <input
                                                    type="text"
                                                    placeholder="Series name..."
                                                    class="bg-overlay border border-outline rounded px-3 py-2 text-primary text-sm focus:outline-none focus:border-accent"
                                                    prop:value=move || series_name_input.get()
                                                    on:input=move |ev| set_series_name_input.set(event_target_value(&ev))
                                                />
                                                <input
                                                    type="text"
                                                    placeholder="Opponent name..."
                                                    class="bg-overlay border border-outline rounded px-3 py-2 text-primary text-sm focus:outline-none focus:border-accent"
                                                    prop:value=move || series_opponent_input.get()
                                                    on:input=move |ev| set_series_opponent_input.set(event_target_value(&ev))
                                                />
                                                <select
                                                    class="bg-overlay border border-outline rounded px-3 py-2 text-primary text-sm focus:outline-none focus:border-accent"
                                                    prop:value=move || series_format_input.get()
                                                    on:change=move |ev| set_series_format_input.set(event_target_value(&ev))
                                                >
                                                    <option value="bo1">"Bo1"</option>
                                                    <option value="bo3" selected>"Bo3"</option>
                                                    <option value="bo5">"Bo5"</option>
                                                </select>
                                                <div class="flex items-center gap-3">
                                                    <label class="flex items-center gap-2 text-sm text-secondary cursor-pointer">
                                                        <input
                                                            type="checkbox"
                                                            class="accent-purple-500"
                                                            prop:checked=move || series_fearless_input.get()
                                                            on:change=move |ev| {
                                                                let checked = event_target_checked(&ev);
                                                                set_series_fearless_input.set(checked);
                                                            }
                                                        />
                                                        "Fearless"
                                                    </label>
                                                    <button
                                                        class="px-3 py-1.5 rounded text-sm font-medium bg-accent text-accent-contrast hover:opacity-90 transition-opacity cursor-pointer"
                                                        on:click=move |_| {
                                                            let name = series_name_input.get_untracked();
                                                            if name.trim().is_empty() {
                                                                set_series_status.set(Some("Give the series a name.".into()));
                                                                return;
                                                            }
                                                            let opp = series_opponent_input.get_untracked();
                                                            let opp_opt = if opp.is_empty() { None } else { Some(opp) };
                                                            let fmt = series_format_input.get_untracked();
                                                            let fearless = series_fearless_input.get_untracked();
                                                            leptos::task::spawn_local(async move {
                                                                match create_series_fn(name.clone(), opp_opt.clone(), fmt.clone(), fearless).await {
                                                                    Ok(id) => {
                                                                        let new_series = Series {
                                                                            id: Some(id),
                                                                            name,
                                                                            team_id: String::new(),
                                                                            opponent_id: None,
                                                                            opponent_name: opp_opt,
                                                                            format: fmt,
                                                                            is_fearless: fearless,
                                                                            notes: None,
                                                                            created_by: String::new(),
                                                                        };
                                                                        set_active_series.set(Some(new_series));
                                                                        set_active_game_number.set(1);
                                                                        set_series_name_input.set(String::new());
                                                                        set_series_opponent_input.set(String::new());
                                                                        set_series_status.set(Some("Series created!".into()));
                                                                        series_resource.refetch();
                                                                    }
                                                                    Err(e) => set_series_status.set(Some(format!("Error: {e}"))),
                                                                }
                                                            });
                                                        }
                                                    >"Create"</button>
                                                </div>
                                            </div>
                                        </div>

                                        // Existing series list
                                        <Suspense fallback=|| view! { <div class="text-muted text-sm">"Loading series..."</div> }>
                                            {move || series_resource.get().map(|result| match result {
                                                Ok(list) if list.is_empty() => view! {
                                                    <p class="text-dimmed text-sm">"No series yet."</p>
                                                }.into_any(),
                                                Ok(list) => view! {
                                                    <div class="flex flex-col gap-1">
                                                        <span class="text-secondary text-sm font-medium">"Existing Series"</span>
                                                        {list.into_iter().map(|s| {
                                                            let series_clone = s.clone();
                                                            let label = format!(
                                                                "{} {}{}",
                                                                s.name,
                                                                s.format.to_uppercase(),
                                                                if s.is_fearless { " (Fearless)" } else { "" }
                                                            );
                                                            let opp_label = s.opponent_name.clone().unwrap_or_default();
                                                            view! {
                                                                <button
                                                                    class="flex items-center justify-between w-full px-3 py-2 rounded bg-overlay hover:bg-overlay-strong text-sm text-primary transition-colors cursor-pointer"
                                                                    on:click=move |_| {
                                                                        let sc = series_clone.clone();
                                                                        let is_fearless = sc.is_fearless;
                                                                        let sid = sc.id.clone();
                                                                        set_active_series.set(Some(sc));
                                                                        set_active_game_number.set(1);
                                                                        // Load fearless bans if applicable
                                                                        if is_fearless {
                                                                            if let Some(sid) = sid {
                                                                                leptos::task::spawn_local(async move {
                                                                                    if let Ok(champs) = get_fearless_champions(sid, None).await {
                                                                                        set_fearless_used.set(champs);
                                                                                    }
                                                                                });
                                                                            }
                                                                        }
                                                                    }
                                                                >
                                                                    <span>{label}</span>
                                                                    {if !opp_label.is_empty() {
                                                                        view! { <span class="text-muted text-xs">"vs " {opp_label}</span> }.into_any()
                                                                    } else {
                                                                        view! { <span></span> }.into_any()
                                                                    }}
                                                                </button>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                }.into_any(),
                                                Err(e) => view! {
                                                    <p class="text-red-400 text-sm">"Failed to load series: " {e.to_string()}</p>
                                                }.into_any(),
                                            })}
                                        </Suspense>

                                        {move || series_status.get().map(|msg| view! {
                                            <p class="text-sm text-accent">{msg}</p>
                                        })}
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
            </div>

            // Board + Comments
            <div class="flex gap-4">
                <div class="flex-1 bg-elevated border border-divider rounded-lg p-4">
                    <Suspense fallback=|| view! { <div class="text-muted text-center py-8">"Loading champions..."</div> }>
                        {move || champions_resource.get().map(|result| match result {
                            Err(e) => view! {
                                <div class="text-red-400">"Failed to load champions: " {e.to_string()}</div>
                            }.into_any(),
                            Ok(champs) => {
                                let champion_map: HashMap<String, Champion> = champs
                                    .into_iter()
                                    .map(|c| (c.name.clone(), c))
                                    .collect();
                                view! {
                                    <DraftBoard
                                        draft_slots=draft_slots
                                        champion_map=champion_map
                                        active_slot=active_slot
                                        on_slot_click=on_slot_click
                                        on_slot_drop=on_slot_drop
                                        highlighted_slot=highlighted_slot
                                        on_slot_clear=on_slot_clear
                                        slot_comments=slot_comments
                                    />
                                }.into_any()
                            }
                        })}
                    </Suspense>
                    // Per-slot comment editor (visible when a pick slot is highlighted)
                    {move || {
                        let hl = highlighted_slot.get();
                        hl.and_then(|idx| {
                            let (_, kind, _) = slot_meta(idx);
                            if kind != "pick" { return None; }
                            let slots = draft_slots.get();
                            let filled = slots.get(idx).and_then(|s| s.as_ref()).is_some();
                            if !filled { return None; }
                            let champ = slots[idx].clone().unwrap_or_default();
                            Some(view! {
                                <div class="mt-2 flex items-center gap-2">
                                    <span class="text-secondary text-sm flex-shrink-0">{format!("{} comment:", champ)}</span>
                                    <input
                                        type="text"
                                        placeholder="Pick rationale..."
                                        class="flex-1 bg-overlay border border-outline rounded px-2 py-1 text-primary text-sm focus:outline-none focus:border-accent"
                                        prop:value=move || slot_comment_input.get()
                                        on:input=move |ev| set_slot_comment_input.set(event_target_value(&ev))
                                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                                            if ev.key() == "Enter" {
                                                if let Some(slot_idx) = highlighted_slot.get_untracked() {
                                                    let val = slot_comment_input.get_untracked();
                                                    let comment = if val.trim().is_empty() { None } else { Some(val) };
                                                    set_slot_comments.update(|sc| {
                                                        if slot_idx < sc.len() { sc[slot_idx] = comment; }
                                                    });
                                                }
                                            }
                                        }
                                        on:blur=move |_| {
                                            if let Some(slot_idx) = highlighted_slot.get_untracked() {
                                                let val = slot_comment_input.get_untracked();
                                                let comment = if val.trim().is_empty() { None } else { Some(val) };
                                                set_slot_comments.update(|sc| {
                                                    if slot_idx < sc.len() { sc[slot_idx] = comment; }
                                                });
                                            }
                                        }
                                    />
                                </div>
                            })
                        })
                    }}
                </div>

                // Comments sidebar
                <div class="w-72 bg-elevated border border-divider rounded-lg p-4 flex flex-col gap-3">
                    <h3 class="text-primary font-bold">"Comments"</h3>
                    <div class="flex flex-col gap-1 max-h-64 overflow-y-auto flex-1">
                        {move || {
                            let list = comments.get();
                            if list.is_empty() {
                                view! { <p class="text-dimmed text-sm">"No comments yet."</p> }.into_any()
                            } else {
                                view! {
                                    <div class="flex flex-col gap-1">
                                        {list.into_iter().map(|c| view! {
                                            <div class="bg-surface rounded p-2 text-sm text-gray-200">{c}</div>
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                    <textarea
                        rows="3"
                        placeholder="Add a comment..."
                        class="w-full bg-overlay border border-outline rounded px-3 py-2 text-primary text-sm placeholder-gray-400 focus:outline-none focus:border-accent resize-none"
                        on:input=move |ev| set_comment_input.set(event_target_value(&ev))
                        prop:value=move || comment_input.get()
                    />
                    <button
                        class="bg-overlay-strong hover:bg-overlay-strong text-primary text-sm rounded px-3 py-1 transition-colors"
                        on:click=move |_| {
                            let text = comment_input.get_untracked();
                            let trimmed = text.trim().to_string();
                            if !trimmed.is_empty() {
                                set_comments.update(|c| c.push(trimmed));
                                set_comment_input.set(String::new());
                            }
                        }
                    >
                        "+ Add Comment"
                    </button>
                </div>

                // Intel Sidebar
                {move || if intel_open.get() {
                    let current_tab = intel_tab.get();
                    let draft_champs = draft_slots.get();
                    let all_draft_champs: Vec<String> = draft_champs.iter().filter_map(|s| s.clone()).collect();
                    let all_draft_champs_for_matchup = all_draft_champs.clone();
                    view! {
                        <div class="w-[350px] flex-shrink-0 bg-elevated border border-divider rounded-lg p-4 flex flex-col gap-3 max-h-[600px] overflow-y-auto">
                            // Tab buttons
                            <div class="flex gap-1">
                                {["pools", "their_picks", "matchups"].iter().map(|&tab| {
                                    let tab_str = tab.to_string();
                                    let tab_for_class = tab_str.clone();
                                    let tab_for_click = tab_str.clone();
                                    let label = match tab {
                                        "pools" => "Our Pools",
                                        "their_picks" => "Their Picks",
                                        "matchups" => "Matchups",
                                        _ => "",
                                    };
                                    view! {
                                        <button
                                            class=move || {
                                                if intel_tab.get() == tab_for_class {
                                                    "flex-1 px-2 py-1.5 rounded text-xs font-medium bg-accent text-accent-contrast transition-colors cursor-pointer"
                                                } else {
                                                    "flex-1 px-2 py-1.5 rounded text-xs font-medium bg-overlay text-muted hover:bg-overlay-strong transition-colors cursor-pointer"
                                                }
                                            }
                                            on:click=move |_| set_intel_tab.set(tab_for_click.clone())
                                        >
                                            {label}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>

                            // Tab content
                            {if current_tab == "pools" {
                                view! {
                                    <div class="flex flex-col gap-3">
                                        <Suspense fallback=|| view! { <div class="text-muted text-sm">"Loading pools..."</div> }>
                                            {move || team_pools.get().map(|result| match result {
                                                Ok(pools) if pools.is_empty() => view! {
                                                    <p class="text-dimmed text-sm">"No starters with champion pools yet."</p>
                                                }.into_any(),
                                                Ok(mut pools) => {
                                                    pools.sort_by_key(|(_, role, _)| role_order(role));
                                                    view! {
                                                        <div class="flex flex-col gap-3">
                                                            // Collect team stats into a lookup map: username -> Vec<ChampionStatSummary>
                                                            {let all_stats: std::collections::HashMap<String, Vec<ChampionStatSummary>> = team_stats.get()
                                                                .and_then(|r| r.ok())
                                                                .map(|v| v.into_iter().collect())
                                                                .unwrap_or_default();
                                                            pools.into_iter().map(|(username, role, entries)| {
                                                                let user_stats = all_stats.get(&username).cloned().unwrap_or_default();
                                                                let role_label = match role.as_str() {
                                                                    "top" => "TOP",
                                                                    "jungle" | "jng" => "JNG",
                                                                    "mid" | "middle" => "MID",
                                                                    "bot" | "adc" => "BOT",
                                                                    "support" | "sup" => "SUP",
                                                                    other => other,
                                                                };
                                                                view! {
                                                                    <div class="bg-surface rounded p-2">
                                                                        <div class="flex items-center gap-2 mb-1.5">
                                                                            <span class="text-xs font-bold text-accent uppercase">{role_label.to_string()}</span>
                                                                            <span class="text-xs text-secondary">{username}</span>
                                                                        </div>
                                                                        {if entries.is_empty() {
                                                                            view! { <p class="text-dimmed text-xs">"No champions"</p> }.into_any()
                                                                        } else {
                                                                            // Group by tier
                                                                            let tiers_order = ["comfort", "match_ready", "scrim_ready", "practicing", "to_practice"];
                                                                            fn tier_label_fn(t: &str) -> &'static str {
                                                                                match t {
                                                                                    "comfort" => "Comfort",
                                                                                    "match_ready" => "Match Ready",
                                                                                    "scrim_ready" => "Scrim Ready",
                                                                                    "practicing" => "Practicing",
                                                                                    "to_practice" => "To Practice",
                                                                                    _ => "Other",
                                                                                }
                                                                            }
                                                                            view! {
                                                                                <div class="flex flex-col gap-1">
                                                                                    {tiers_order.iter().filter_map(|&tier| {
                                                                                        let tier_entries: Vec<_> = entries.iter().filter(|e| e.tier == tier).collect();
                                                                                        if tier_entries.is_empty() {
                                                                                            return None;
                                                                                        }
                                                                                        let tier_cls = match tier {
                                                                                            "comfort" => "text-green-400",
                                                                                            "match_ready" => "text-blue-400",
                                                                                            "scrim_ready" => "text-yellow-400",
                                                                                            "practicing" => "text-orange-400",
                                                                                            "to_practice" => "text-muted",
                                                                                            _ => "text-muted",
                                                                                        };
                                                                                        let label = tier_label_fn(tier);
                                                                                        Some(view! {
                                                                                            <div>
                                                                                                <span class=format!("text-[10px] font-bold uppercase {tier_cls}")>{label.to_string()}</span>
                                                                                                <div class="flex flex-wrap gap-1 mt-0.5">
                                                                                                    {tier_entries.into_iter().map(|entry| {
                                                                                                        let champ_name = entry.champion.clone();
                                                                                                        let champ_for_click = champ_name.clone();
                                                                                                        let champ_for_title = champ_name.clone();
                                                                                                        let champ_for_display = champ_name.clone();
                                                                                                        let comfort = entry.comfort_level.unwrap_or(0);
                                                                                                        // Look up match stats for this champion
                                                                                                        let stat = user_stats.iter().find(|s| s.champion == champ_name).cloned();
                                                                                                        let title_text = if let Some(ref s) = stat {
                                                                                                            let wr = if s.games > 0 { (s.wins as f64 / s.games as f64 * 100.0).round() as i32 } else { 0 };
                                                                                                            format!("{} - {}G {}%W {:.1} KDA", champ_for_title, s.games, wr, s.avg_kda)
                                                                                                        } else {
                                                                                                            format!("Click to add {} to active slot", champ_for_title)
                                                                                                        };
                                                                                                        let comfort_dots = (0..5).map(|i| {
                                                                                                            if i < comfort {
                                                                                                                view! { <span class="w-1 h-1 rounded-full bg-accent inline-block"></span> }
                                                                                                            } else {
                                                                                                                view! { <span class="w-1 h-1 rounded-full bg-overlay-strong inline-block"></span> }
                                                                                                            }
                                                                                                        }).collect_view();
                                                                                                        let stat_badge = stat.map(|s| {
                                                                                                            let wr = if s.games > 0 { (s.wins as f64 / s.games as f64 * 100.0).round() as i32 } else { 0 };
                                                                                                            format!("{}G/{}%", s.games, wr)
                                                                                                        });
                                                                                                        view! {
                                                                                                            <button
                                                                                                                class="bg-overlay rounded px-1.5 py-0.5 text-xs text-primary hover:bg-accent hover:text-accent-contrast transition-colors cursor-pointer flex items-center gap-1"
                                                                                                                title=title_text
                                                                                                                on:click=move |_| {
                                                                                                                    if let Some(slot) = active_slot.get_untracked() {
                                                                                                                        fill_slot(slot, champ_for_click.clone());
                                                                                                                    }
                                                                                                                }
                                                                                                            >
                                                                                                                <span>{champ_for_display}</span>
                                                                                                                <span class="flex gap-px">{comfort_dots}</span>
                                                                                                                {stat_badge.map(|badge| view! {
                                                                                                                    <span class="text-[9px] text-muted ml-0.5">{badge}</span>
                                                                                                                })}
                                                                                                            </button>
                                                                                                        }
                                                                                                    }).collect_view()}
                                                                                                </div>
                                                                                            </div>
                                                                                        })
                                                                                    }).collect_view()}
                                                                                </div>
                                                                            }.into_any()
                                                                        }}
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_any()
                                                },
                                                Err(e) => view! {
                                                    <p class="text-red-400 text-sm">{format!("Error: {e}")}</p>
                                                }.into_any(),
                                            })}
                                        </Suspense>
                                    </div>
                                }.into_any()
                            } else if current_tab == "their_picks" {
                                view! {
                                    <div class="flex flex-col gap-3">
                                        // Opponent selector
                                        <Suspense fallback=|| view! { <div class="text-muted text-sm">"Loading opponents..."</div> }>
                                            {move || opponents_list.get().map(|result| match result {
                                                Ok(opps) if opps.is_empty() => view! {
                                                    <p class="text-dimmed text-sm">"No opponents scouted yet. Add opponents from the Opponents page."</p>
                                                }.into_any(),
                                                Ok(opps) => view! {
                                                    <select
                                                        class="w-full bg-overlay border border-outline rounded px-2 py-1.5 text-primary text-sm focus:outline-none focus:border-accent"
                                                        prop:value=move || selected_opponent_id.get()
                                                        on:change=move |ev| set_selected_opponent_id.set(event_target_value(&ev))
                                                    >
                                                        <option value="">"-- Select Opponent --"</option>
                                                        {opps.into_iter().map(|opp| {
                                                            let id = opp.id.clone().unwrap_or_default();
                                                            let name = opp.name.clone();
                                                            view! { <option value=id>{name}</option> }
                                                        }).collect_view()}
                                                    </select>
                                                }.into_any(),
                                                Err(e) => view! {
                                                    <p class="text-red-400 text-sm">{format!("Error: {e}")}</p>
                                                }.into_any(),
                                            })}
                                        </Suspense>
                                        // Opponent players
                                        <Suspense fallback=|| view! { <div class="text-muted text-sm">"Loading..."</div> }>
                                            {move || {
                                                let draft_champs = all_draft_champs.clone();
                                                opponent_players.get().map(move |result| match result {
                                                    Ok(players) if players.is_empty() && !selected_opponent_id.get_untracked().is_empty() => view! {
                                                        <p class="text-dimmed text-sm">"No players scouted for this opponent."</p>
                                                    }.into_any(),
                                                    Ok(players) if players.is_empty() => view! {
                                                        <span></span>
                                                    }.into_any(),
                                                    Ok(players) => {
                                                        let draft_set = draft_champs.clone();
                                                        view! {
                                                            <div class="flex flex-col gap-2">
                                                                {players.into_iter().map(|player| {
                                                                    let draft_set_inner = draft_set.clone();
                                                                    let role_label = match player.role.as_str() {
                                                                        "top" => "TOP",
                                                                        "jungle" | "jng" => "JNG",
                                                                        "mid" | "middle" => "MID",
                                                                        "bot" | "adc" => "BOT",
                                                                        "support" | "sup" => "SUP",
                                                                        other => other,
                                                                    };
                                                                    view! {
                                                                        <div class="bg-surface rounded p-2">
                                                                            <div class="flex items-center gap-2 mb-1">
                                                                                <span class="text-xs font-bold text-red-400 uppercase">{role_label.to_string()}</span>
                                                                                <span class="text-xs text-secondary">{player.name.clone()}</span>
                                                                            </div>
                                                                            {if player.recent_champions.is_empty() {
                                                                                view! { <p class="text-dimmed text-xs">"No recent champions"</p> }.into_any()
                                                                            } else {
                                                                                view! {
                                                                                    <div class="flex flex-wrap gap-1">
                                                                                        {player.recent_champions.iter().map(|champ| {
                                                                                            let is_drafted = draft_set_inner.contains(champ);
                                                                                            let cls = if is_drafted {
                                                                                                "bg-overlay rounded px-1.5 py-0.5 text-xs text-dimmed line-through opacity-50"
                                                                                            } else {
                                                                                                "bg-overlay rounded px-1.5 py-0.5 text-xs text-primary"
                                                                                            };
                                                                                            view! {
                                                                                                <span class=cls>{champ.clone()}</span>
                                                                                            }
                                                                                        }).collect_view()}
                                                                                    </div>
                                                                                }.into_any()
                                                                            }}
                                                                            {player.notes.as_ref().map(|notes| view! {
                                                                                <p class="text-dimmed text-xs mt-1 italic">{notes.clone()}</p>
                                                                            })}
                                                                        </div>
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                        }.into_any()
                                                    },
                                                    Err(e) => view! {
                                                        <p class="text-red-400 text-sm">{format!("Error: {e}")}</p>
                                                    }.into_any(),
                                                })
                                            }}
                                        </Suspense>
                                    </div>
                                }.into_any()
                            } else {
                                // Matchups tab
                                view! {
                                    <div class="flex flex-col gap-3">
                                        {move || {
                                            let champ = matchup_champion.get();
                                            let draft_champs_inner = all_draft_champs_for_matchup.clone();
                                            if let Some(ref c) = champ {
                                                view! {
                                                    <div>
                                                        <p class="text-sm text-secondary mb-2">
                                                            "Matchup notes for "
                                                            <span class="text-primary font-medium">{c.clone()}</span>
                                                        </p>
                                                        <Suspense fallback=|| view! { <div class="text-muted text-sm">"Loading notes..."</div> }>
                                                            {move || matchup_notes.get().map(|result| match result {
                                                                Ok(notes) if notes.is_empty() => view! {
                                                                    <p class="text-dimmed text-sm">"No matchup notes found for this champion."</p>
                                                                }.into_any(),
                                                                Ok(notes) => view! {
                                                                    <div class="flex flex-col gap-2">
                                                                        {notes.into_iter().map(|(author, note)| {
                                                                            view! {
                                                                                <div class="bg-surface rounded p-2">
                                                                                    <div class="flex items-center gap-2 mb-1">
                                                                                        <span class="text-xs font-medium text-accent">{author}</span>
                                                                                        <span class="text-xs text-muted">{note.champion.clone()}" "{note.role.clone()}</span>
                                                                                        {note.difficulty.map(|d| {
                                                                                            let diff_cls = match d {
                                                                                                1..=2 => "text-green-400",
                                                                                                3 => "text-yellow-400",
                                                                                                4..=5 => "text-red-400",
                                                                                                _ => "text-muted",
                                                                                            };
                                                                                            view! {
                                                                                                <span class=format!("text-xs {diff_cls}")>{format!("Diff: {d}/5")}</span>
                                                                                            }
                                                                                        })}
                                                                                    </div>
                                                                                    <p class="text-xs text-primary font-medium">{note.title.clone()}</p>
                                                                                    <p class="text-xs text-secondary mt-0.5">{note.content.clone()}</p>
                                                                                </div>
                                                                            }
                                                                        }).collect_view()}
                                                                    </div>
                                                                }.into_any(),
                                                                Err(e) => view! {
                                                                    <p class="text-red-400 text-sm">{format!("Error: {e}")}</p>
                                                                }.into_any(),
                                                            })}
                                                        </Suspense>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                // Show clickable list of drafted champions
                                                view! {
                                                    <div>
                                                        <p class="text-dimmed text-sm mb-2">"Select a drafted champion to view matchup notes."</p>
                                                        {if draft_champs_inner.is_empty() {
                                                            view! { <p class="text-dimmed text-xs">"No champions drafted yet."</p> }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="flex flex-wrap gap-1">
                                                                    {draft_champs_inner.into_iter().map(|champ| {
                                                                        let champ_for_click = champ.clone();
                                                                        view! {
                                                                            <button
                                                                                class="bg-overlay rounded px-2 py-1 text-xs text-primary hover:bg-accent hover:text-accent-contrast transition-colors cursor-pointer"
                                                                                on:click=move |_| set_matchup_champion.set(Some(champ_for_click.clone()))
                                                                            >
                                                                                {champ.clone()}
                                                                            </button>
                                                                        }
                                                                    }).collect_view()}
                                                                </div>
                                                            }.into_any()
                                                        }}
                                                    </div>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
            </div>

            // Champion Picker
            <div class="bg-elevated border border-divider rounded-lg p-4">
                {move || active_slot_label().map(|label| view! {
                    <p class="text-accent-hover text-sm font-medium mb-2">{label}</p>
                })}
                <Suspense fallback=|| view! { <div class="text-muted">"Loading champions..."</div> }>
                    {move || champions_resource.get().map(|result| match result {
                        Err(e) => view! {
                            <ErrorBanner message=format!("Failed to load champions: {e}") />
                        }.into_any(),
                        Ok(champs) => view! {
                            <ChampionPicker
                                champions=champs
                                used_champions=used_champions()
                                on_select=on_champion_select
                            />
                        }.into_any(),
                    })}
                </Suspense>
            </div>

            // Action buttons
            <div class="flex gap-3 items-center">
                <button
                    class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-6 py-2 transition-colors"
                    on:click=do_save
                >
                    {move || if loaded_draft_id.get().is_some() { "Update Draft" } else { "Save Draft" }}
                </button>
                <button
                    class="bg-overlay-strong hover:bg-overlay-strong text-primary rounded px-4 py-2 transition-colors"
                    on:click=move |_| {
                        set_draft_slots.set(vec![None; 20]);
                        set_active_slot.set(Some(0));
                        set_highlighted_slot.set(None);
                        set_comments.set(Vec::new());
                        set_save_result.set(None);
                        set_loaded_draft_id.set(None);
                        set_draft_name.set(String::new());
                        set_opponent.set(String::new());
                        set_rating.set(None);
                        set_tags.set(Vec::new());
                        set_win_conditions.set(String::new());
                        set_watch_out.set(String::new());
                        set_slot_comments.set(vec![None; 20]);
                        set_slot_comment_input.set(String::new());
                        // Keep selected_team_id — the user probably wants the same team
                    }
                >
                    {move || if loaded_draft_id.get().is_some() { "New Draft" } else { "Clear" }}
                </button>
                {move || save_result.get().map(|msg| view! {
                    <div class="text-green-300 text-sm">{msg}</div>
                })}
                // Pipeline CTAs for loaded draft
                {move || {
                    let did = loaded_draft_id.get();
                    if let Some(current_draft_id) = did {
                        let draft_id_for_prep = current_draft_id.clone();
                        let draft_id_for_review = current_draft_id.clone();
                        let draft_id_for_review2 = current_draft_id.clone();
                        view! {
                            <div class="flex items-center gap-2 flex-wrap">
                                // Prep for This Draft CTA
                                <button
                                    class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm transition-colors disabled:opacity-50"
                                    disabled=move || cta_loading.get()
                                    on:click=move |_| {
                                        let did2 = draft_id_for_prep.clone();
                                        set_cta_loading.set(true);
                                        set_cta_status.set(None);
                                        leptos::task::spawn_local(async move {
                                            match check_draft_has_game_plan(did2.clone()).await {
                                                Ok(None) => {
                                                    set_cta_loading.set(false);
                                                    #[cfg(feature = "hydrate")]
                                                    if let Some(window) = web_sys::window() {
                                                        let _ = window.location().set_href(&format!("/game-plan?draft_id={did2}"));
                                                    }
                                                }
                                                Ok(Some(plan_id)) => {
                                                    set_cta_loading.set(false);
                                                    set_duplicate_prompt.set(Some((did2, plan_id)));
                                                }
                                                Err(e) => {
                                                    set_cta_loading.set(false);
                                                    set_cta_status.set(Some(format!("Error: {e}")));
                                                }
                                            }
                                        });
                                    }
                                >"Prep for This Draft"</button>
                                // Review This Game CTA (only when draft has linked game plans)
                                {move || {
                                    let bid = draft_id_for_review.clone();
                                    let has_plan = game_plan_counts.get()
                                        .and_then(|r| r.ok())
                                        .and_then(|pairs| pairs.into_iter().find(|(id, _)| id == &bid).map(|(_, c)| c))
                                        .unwrap_or(0) > 0;
                                    if has_plan {
                                        let dr2 = draft_id_for_review2.clone();
                                        view! {
                                            <button
                                                class="bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg px-4 py-2 text-sm transition-colors"
                                                on:click=move |_| {
                                                    let did3 = dr2.clone();
                                                    #[cfg(feature = "hydrate")]
                                                    if let Some(window) = web_sys::window() {
                                                        let _ = window.location().set_href(&format!("/post-game?draft_id={did3}"));
                                                    }
                                                }
                                            >"Review This Game"</button>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }
                                }}
                                {move || cta_status.get().map(|msg| view! {
                                    <span class="text-red-400 text-sm">{msg}</span>
                                })}
                            </div>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                }}
            </div>

            // Saved Drafts
            <div>
                <h2 class="text-xl font-bold text-primary mb-3">"Saved Drafts"</h2>
                // Tag filter buttons
                <div class="flex flex-wrap gap-1.5 mb-3">
                    <button
                        class=move || if filter_tag.get().is_empty() {
                            "rounded px-2.5 py-0.5 text-xs font-medium bg-accent text-accent-contrast transition-colors cursor-pointer"
                        } else {
                            "rounded px-2.5 py-0.5 text-xs font-medium bg-overlay text-muted hover:bg-overlay-strong transition-colors cursor-pointer"
                        }
                        on:click=move |_| set_filter_tag.set(String::new())
                    >"All"</button>
                    {COMPOSITION_TAGS.iter().map(|&tag| {
                        let tag_str = tag.to_string();
                        let tag_for_class = tag_str.clone();
                        let tag_for_click = tag_str.clone();
                        view! {
                            <button
                                class=move || if filter_tag.get() == tag_for_class {
                                    "rounded px-2.5 py-0.5 text-xs font-medium bg-accent text-accent-contrast transition-colors cursor-pointer"
                                } else {
                                    "rounded px-2.5 py-0.5 text-xs font-medium bg-overlay text-muted hover:bg-overlay-strong transition-colors cursor-pointer"
                                }
                                on:click=move |_| set_filter_tag.set(tag_for_click.clone())
                            >{tag}</button>
                        }
                    }).collect_view()}
                </div>
                <Suspense fallback=|| view! { <div class="text-muted">"Loading..."</div> }>
                    {move || {
                        let champ_url_map: HashMap<String, String> = champions_resource.get()
                            .and_then(|r| r.ok())
                            .map(|champs| champs.into_iter().map(|c| (c.name, c.image_full)).collect())
                            .unwrap_or_default();
                        let champ_map_sv = StoredValue::new(champ_url_map);

                        drafts.get().map(move |result| match result {
                            Ok(list) if list.is_empty() => view! {
                                <p class="text-dimmed">"No drafts yet."</p>
                            }.into_any(),
                            Ok(list) => {
                                let ft = filter_tag.get();
                                let filtered: Vec<Draft> = if ft.is_empty() {
                                    list
                                } else {
                                    list.into_iter().filter(|d| d.tags.contains(&ft)).collect()
                                };
                                if filtered.is_empty() {
                                    return view! {
                                        <p class="text-dimmed">"No drafts match this filter."</p>
                                    }.into_any();
                                }
                                view! {
                                <div class="flex flex-col gap-2">
                                    {filtered.into_iter().map(|d| {
                                        let d_id = d.id.clone();
                                        let d_id_for_cta = d.id.clone().unwrap_or_default();
                                        let d_id_for_badge = d.id.clone().unwrap_or_default();
                                        let d_name = d.name.clone();
                                        let d_opp = d.opponent.clone().unwrap_or_default();
                                        let d_comments = d.comments.clone();
                                        let d_actions = d.actions.clone();
                                        let d_team_id = d.team_id.clone();
                                        let d_rating = d.rating.clone();
                                        let d_our_side = d.our_side.clone();
                                        let d_tags = d.tags.clone();
                                        let d_win_conditions = d.win_conditions.clone().unwrap_or_default();
                                        let d_watch_out = d.watch_out.clone().unwrap_or_default();
                                        let _d_series_id = d.series_id.clone();
                                        let d_game_number = d.game_number;

                                        let icon_url = |a: &DraftAction| champ_map_sv.with_value(|m| m.get(&a.champion).cloned().unwrap_or_default());

                                        // Blue bans: phase1 = orders 0,2,4  phase2 = orders 13,15
                                        let blue_ban_p1: Vec<(String, String)> = d.actions.iter()
                                            .filter(|a| a.side == "blue" && matches!(a.order, 0 | 2 | 4))
                                            .map(|a| (a.champion.clone(), icon_url(a)))
                                            .collect();
                                        let blue_ban_p2: Vec<(String, String)> = d.actions.iter()
                                            .filter(|a| a.side == "blue" && matches!(a.order, 13 | 15))
                                            .map(|a| (a.champion.clone(), icon_url(a)))
                                            .collect();
                                        // Red bans: phase1 = orders 1,3,5  phase2 = orders 12,14
                                        let red_ban_p1: Vec<(String, String)> = d.actions.iter()
                                            .filter(|a| a.side == "red" && matches!(a.order, 1 | 3 | 5))
                                            .map(|a| (a.champion.clone(), icon_url(a)))
                                            .collect();
                                        let red_ban_p2: Vec<(String, String)> = d.actions.iter()
                                            .filter(|a| a.side == "red" && matches!(a.order, 12 | 14))
                                            .map(|a| (a.champion.clone(), icon_url(a)))
                                            .collect();

                                        let blue_pick_icons: Vec<(String, String)> = d.actions.iter()
                                            .filter(|a| { let o = a.order as usize; !(o < 6 || (12..16).contains(&o)) && a.side == "blue" })
                                            .map(|a| (a.champion.clone(), icon_url(a)))
                                            .collect();
                                        let red_pick_icons: Vec<(String, String)> = d.actions.iter()
                                            .filter(|a| { let o = a.order as usize; !(o < 6 || (12..16).contains(&o)) && a.side == "red" })
                                            .map(|a| (a.champion.clone(), icon_url(a)))
                                            .collect();

                                        let has_picks = !blue_pick_icons.is_empty() || !red_pick_icons.is_empty();
                                        let has_both_sides = !blue_pick_icons.is_empty() && !red_pick_icons.is_empty();
                                        let has_left_bans = !blue_ban_p1.is_empty() || !blue_ban_p2.is_empty();
                                        let has_right_bans = !red_ban_p1.is_empty() || !red_ban_p2.is_empty();

                                        let display_name = d.name.clone();
                                        let display_opp = d.opponent.clone();
                                        let display_rating = d_rating.clone();
                                        let display_tags = d.tags.clone();

                                        view! {
                                            <div class="bg-elevated border border-divider rounded px-4 py-3 flex items-center gap-4">
                                                <div class="flex-1 min-w-0">
                                                    // Name + opponent + rating badge
                                                    <div class="flex items-center gap-2 mb-1.5">
                                                        <span class="text-primary font-medium">{display_name}</span>
                                                        {display_opp.map(|o| view! {
                                                            <span class="text-muted text-sm">"vs " {o}</span>
                                                        })}
                                                        {display_rating.map(|r| {
                                                            let cls = format!("rounded px-1.5 py-0.5 text-xs font-bold {}", tier_badge_class(&r));
                                                            view! { <span class=cls>{r}</span> }
                                                        })}
                                                        {display_tags.into_iter().map(|tag| {
                                                            view! { <span class="rounded px-1.5 py-0.5 text-xs font-medium bg-overlay-strong text-secondary">{tag}</span> }
                                                        }).collect_view()}
                                                        // Game plan count badge
                                                        {move || {
                                                            let bid = d_id_for_badge.clone();
                                                            let count = game_plan_counts.get()
                                                                .and_then(|r| r.ok())
                                                                .and_then(|pairs| pairs.into_iter().find(|(id, _)| id == &bid).map(|(_, c)| c))
                                                                .unwrap_or(0);
                                                            if count > 0 {
                                                                let label = if count == 1 { "1 game plan".to_string() } else { format!("{count} game plans") };
                                                                view! {
                                                                    <a href="/game-plan"
                                                                       class="bg-surface border border-outline/50 text-muted text-xs rounded px-2 py-0.5 hover:text-primary hover:border-accent/50 transition-colors">
                                                                        {label}
                                                                    </a>
                                                                }.into_any()
                                                            } else {
                                                                view! { <span></span> }.into_any()
                                                            }
                                                        }}
                                                    </div>
                                                    // Icon summary: [blue bans] | [picks] | [red bans]
                                                    <div class="flex items-center gap-0.5 flex-wrap">
                                                        // Blue bans (left)
                                                        {blue_ban_p1.into_iter().map(|(name, url)| view! {
                                                            <div class="relative w-6 h-6 flex-shrink-0" title=name.clone()>
                                                                <div class="w-6 h-6 rounded overflow-hidden border border-outline grayscale opacity-50">
                                                                    <img src=url alt=name.clone() class="w-full h-full object-cover" />
                                                                </div>
                                                                <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                                                                    <div class="w-4 h-px bg-red-500 rotate-45"></div>
                                                                </div>
                                                            </div>
                                                        }).collect_view()}
                                                        // Small gap between phase-1 and phase-2 blue bans
                                                        {if !blue_ban_p2.is_empty() {
                                                            view! { <span class="w-1.5 flex-shrink-0 inline-block"></span> }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        {blue_ban_p2.into_iter().map(|(name, url)| view! {
                                                            <div class="relative w-6 h-6 flex-shrink-0" title=name.clone()>
                                                                <div class="w-6 h-6 rounded overflow-hidden border border-outline grayscale opacity-50">
                                                                    <img src=url alt=name.clone() class="w-full h-full object-cover" />
                                                                </div>
                                                                <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                                                                    <div class="w-4 h-px bg-red-500 rotate-45"></div>
                                                                </div>
                                                            </div>
                                                        }).collect_view()}
                                                        // Separator between bans and picks
                                                        {if has_left_bans && has_picks {
                                                            view! { <span class="text-overlay-strong text-xs mx-0.5 flex-shrink-0">"|"</span> }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        // Blue picks
                                                        {blue_pick_icons.into_iter().map(|(name, url)| view! {
                                                            <div class="w-6 h-6 rounded overflow-hidden border border-blue-700 flex-shrink-0" title=name.clone()>
                                                                <img src=url alt=name.clone() class="w-full h-full object-cover" />
                                                            </div>
                                                        }).collect_view()}
                                                        // VS separator
                                                        {if has_both_sides {
                                                            view! { <span class="text-dimmed text-xs mx-0.5 flex-shrink-0">"vs"</span> }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        // Red picks
                                                        {red_pick_icons.into_iter().map(|(name, url)| view! {
                                                            <div class="w-6 h-6 rounded overflow-hidden border border-red-700 flex-shrink-0" title=name.clone()>
                                                                <img src=url alt=name.clone() class="w-full h-full object-cover" />
                                                            </div>
                                                        }).collect_view()}
                                                        // Separator between picks and red bans
                                                        {if has_right_bans && has_picks {
                                                            view! { <span class="text-overlay-strong text-xs mx-0.5 flex-shrink-0">"|"</span> }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        // Red bans (right)
                                                        {red_ban_p1.into_iter().map(|(name, url)| view! {
                                                            <div class="relative w-6 h-6 flex-shrink-0" title=name.clone()>
                                                                <div class="w-6 h-6 rounded overflow-hidden border border-outline grayscale opacity-50">
                                                                    <img src=url alt=name.clone() class="w-full h-full object-cover" />
                                                                </div>
                                                                <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                                                                    <div class="w-4 h-px bg-red-500 rotate-45"></div>
                                                                </div>
                                                            </div>
                                                        }).collect_view()}
                                                        // Small gap between phase-1 and phase-2 red bans
                                                        {if !red_ban_p2.is_empty() {
                                                            view! { <span class="w-1.5 flex-shrink-0 inline-block"></span> }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        {red_ban_p2.into_iter().map(|(name, url)| view! {
                                                            <div class="relative w-6 h-6 flex-shrink-0" title=name.clone()>
                                                                <div class="w-6 h-6 rounded overflow-hidden border border-outline grayscale opacity-50">
                                                                    <img src=url alt=name.clone() class="w-full h-full object-cover" />
                                                                </div>
                                                                <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                                                                    <div class="w-4 h-px bg-red-500 rotate-45"></div>
                                                                </div>
                                                            </div>
                                                        }).collect_view()}
                                                    </div>
                                                </div>
                                                <button
                                                    class="flex-shrink-0 bg-overlay hover:bg-accent hover:text-accent-contrast text-secondary text-sm font-medium rounded px-3 py-1.5 transition-colors"
                                                    on:click=move |_| {
                                                        set_loaded_draft_id.set(d_id.clone());
                                                        set_draft_name.set(d_name.clone());
                                                        set_opponent.set(d_opp.clone());
                                                        set_selected_team_id.set(d_team_id.clone());
                                                        set_rating.set(d_rating.clone());
                                                        set_our_side.set(d_our_side.clone());
                                                        set_comments.set(d_comments.clone());
                                                        set_tags.set(d_tags.clone());
                                                        set_win_conditions.set(d_win_conditions.clone());
                                                        set_watch_out.set(d_watch_out.clone());
                                                        set_save_result.set(None);
                                                        set_highlighted_slot.set(None);
                                                        let mut slots = vec![None::<String>; 20];
                                                        let mut sc = vec![None::<String>; 20];
                                                        for action in &d_actions {
                                                            let o = action.order as usize;
                                                            if o < 20 {
                                                                slots[o] = Some(action.champion.clone());
                                                                sc[o] = action.comment.clone();
                                                            }
                                                        }
                                                        let next = (0..20).find(|&i| slots[i].is_none());
                                                        set_draft_slots.set(slots);
                                                        set_slot_comments.set(sc);
                                                        set_slot_comment_input.set(String::new());
                                                        set_active_slot.set(next);

                                                        // If this draft belongs to a series, set game number
                                                        if let Some(gn) = d_game_number {
                                                            set_active_game_number.set(gn);
                                                        }
                                                    }
                                                >
                                                    "Open"
                                                </button>
                                                // "Prep for This Draft" CTA or duplicate prompt
                                                {move || {
                                                    let cta_draft_id = d_id_for_cta.clone();
                                                    let prompt = duplicate_prompt.get();
                                                    if prompt.as_ref().map(|(id, _)| id == &cta_draft_id).unwrap_or(false) {
                                                        // Show duplicate prompt for this draft
                                                        let (_, existing_plan_id) = prompt.unwrap();
                                                        let cta_draft_id2 = cta_draft_id.clone();
                                                        view! {
                                                            <div class="flex-shrink-0 flex flex-col gap-1 bg-surface border border-outline/50 rounded-lg px-3 py-2 text-xs">
                                                                <span class="text-muted">"A game plan already exists."</span>
                                                                <div class="flex gap-1.5">
                                                                    <a href="/game-plan"
                                                                       attr:class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded px-2 py-0.5 transition-colors">
                                                                        "View Game Plan"
                                                                    </a>
                                                                    <button
                                                                        class="bg-overlay hover:bg-overlay-strong text-secondary border border-outline/50 font-semibold rounded px-2 py-0.5 transition-colors"
                                                                        on:click=move |_| {
                                                                            let did = cta_draft_id2.clone();
                                                                            #[cfg(feature = "hydrate")]
                                                                            if let Some(window) = web_sys::window() {
                                                                                let _ = window.location().set_href(&format!("/game-plan?draft_id={did}"));
                                                                            }
                                                                            let _ = existing_plan_id.clone(); // consumed
                                                                        }
                                                                    >"Create New"</button>
                                                                    <button
                                                                        class="text-muted hover:text-primary text-xs px-1 transition-colors"
                                                                        on:click=move |_| set_duplicate_prompt.set(None)
                                                                    >"Cancel"</button>
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        let is_loading = cta_loading.get();
                                                        let cta_draft_id3 = cta_draft_id.clone();
                                                        view! {
                                                            <button
                                                                class="flex-shrink-0 bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-3 py-1.5 text-xs transition-colors disabled:opacity-50"
                                                                disabled=is_loading
                                                                on:click=move |_| {
                                                                    let did = cta_draft_id3.clone();
                                                                    set_cta_loading.set(true);
                                                                    set_cta_status.set(None);
                                                                    leptos::task::spawn_local(async move {
                                                                        match check_draft_has_game_plan(did.clone()).await {
                                                                            Ok(None) => {
                                                                                set_cta_loading.set(false);
                                                                                #[cfg(feature = "hydrate")]
                                                                                if let Some(window) = web_sys::window() {
                                                                                    let _ = window.location().set_href(&format!("/game-plan?draft_id={did}"));
                                                                                }
                                                                            }
                                                                            Ok(Some(plan_id)) => {
                                                                                set_cta_loading.set(false);
                                                                                set_duplicate_prompt.set(Some((did, plan_id)));
                                                                            }
                                                                            Err(e) => {
                                                                                set_cta_loading.set(false);
                                                                                set_cta_status.set(Some(format!("Error: {e}")));
                                                                            }
                                                                        }
                                                                    });
                                                                }
                                                            >"Prep for This Draft"</button>
                                                        }.into_any()
                                                    }
                                                }}
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                                }.into_any()
                            },
                            Err(e) => view! {
                                <ErrorBanner message=format!("Failed to load drafts: {e}") />
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </div>

            // Draft Tendencies Panel (Phase 7a)
            <div class="bg-elevated border border-divider rounded-lg">
                <button
                    class="w-full flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-surface/50 transition-colors"
                    on:click=move |_| set_tendencies_open.update(|v| *v = !*v)
                >
                    <h2 class="text-xl font-bold text-primary">"Draft Tendencies"</h2>
                    <span class="text-muted">{move || if tendencies_open.get() { "\u{25B2}" } else { "\u{25BC}" }}</span>
                </button>
                {move || tendencies_open.get().then(|| {
                    view! {
                        <div class="px-4 pb-4">
                            <Suspense fallback=|| view! { <div class="text-dimmed text-sm">"Loading tendencies..."</div> }>
                                {move || tendency_data.get().map(|result| match result {
                                    Ok(tendencies) => {
                                        // Filter to champions with 2+ appearances
                                        let mut champ_totals: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
                                        for t in &tendencies {
                                            *champ_totals.entry(t.champion.clone()).or_insert(0) += t.count;
                                        }
                                        let mut champs: Vec<(String, i32)> = champ_totals.into_iter().filter(|(_, c)| *c >= 2).collect();
                                        champs.sort_by(|a, b| b.1.cmp(&a.1));

                                        if champs.is_empty() {
                                            return view! {
                                                <p class="text-dimmed text-sm">"Not enough draft data yet (need 2+ appearances for a champion)."</p>
                                            }.into_any();
                                        }

                                        // Build position counts per champion
                                        let mut champ_position_counts: std::collections::HashMap<String, std::collections::HashMap<i32, i32>> = std::collections::HashMap::new();
                                        for t in &tendencies {
                                            champ_position_counts
                                                .entry(t.champion.clone())
                                                .or_default()
                                                .insert(t.order, t.count);
                                        }

                                        // Detect predictable patterns (70%+ in same position, 3+ games)
                                        let mut warnings: Vec<String> = Vec::new();
                                        for (champ, total) in &champs {
                                            if let Some(positions) = champ_position_counts.get(champ) {
                                                for (&order, &count) in positions {
                                                    if *total >= 3 && (count as f64 / *total as f64) >= 0.7 {
                                                        let (side, kind, _) = crate::components::draft_board::slot_meta(order as usize);
                                                        warnings.push(format!("{champ}: {:.0}% in {side} {kind} (slot {})", count as f64 / *total as f64 * 100.0, order + 1));
                                                    }
                                                }
                                            }
                                        }

                                        let warnings_clone = warnings.clone();

                                        view! {
                                            <div class="flex flex-col gap-3">
                                                // Warnings
                                                {if !warnings_clone.is_empty() {
                                                    view! {
                                                        <div class="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-3">
                                                            <p class="text-yellow-400 text-xs font-semibold mb-1">"Predictable Patterns Detected"</p>
                                                            {warnings_clone.into_iter().map(|w| view! {
                                                                <p class="text-yellow-300/80 text-xs">{w}</p>
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <div></div> }.into_any()
                                                }}

                                                // Heatmap table
                                                <div class="overflow-x-auto">
                                                    <table class="text-xs w-full">
                                                        <thead>
                                                            <tr>
                                                                <th class="text-left text-muted px-2 py-1 sticky left-0 bg-elevated">"Champion"</th>
                                                                <th class="text-muted px-1 py-1">"Total"</th>
                                                                {(0..20).map(|i| {
                                                                    let (side, kind, _) = crate::components::draft_board::slot_meta(i);
                                                                    let label = format!("{}{}", &side[..1].to_uppercase(), if kind.contains("ban") { "B" } else { "P" });
                                                                    view! { <th class="text-muted px-0.5 py-1 text-center" title=format!("{side} {kind}")>{label}</th> }
                                                                }).collect_view()}
                                                            </tr>
                                                        </thead>
                                                        <tbody>
                                                            {champs.into_iter().map(|(champ, total)| {
                                                                let positions = champ_position_counts.get(&champ).cloned().unwrap_or_default();
                                                                view! {
                                                                    <tr class="border-t border-divider/30">
                                                                        <td class="text-primary px-2 py-1 font-medium sticky left-0 bg-elevated whitespace-nowrap">{champ.clone()}</td>
                                                                        <td class="text-secondary px-1 py-1 text-center">{total}</td>
                                                                        {(0..20).map(|i| {
                                                                            let count = positions.get(&i).copied().unwrap_or(0);
                                                                            let opacity = if count == 0 {
                                                                                "bg-transparent"
                                                                            } else if count == 1 {
                                                                                "bg-accent/20"
                                                                            } else if count == 2 {
                                                                                "bg-accent/40"
                                                                            } else if count == 3 {
                                                                                "bg-accent/60"
                                                                            } else {
                                                                                "bg-accent/80"
                                                                            };
                                                                            view! {
                                                                                <td class=format!("px-0.5 py-1 text-center {opacity} rounded-sm")>
                                                                                    {if count > 0 { format!("{count}") } else { String::new() }}
                                                                                </td>
                                                                            }
                                                                        }).collect_view()}
                                                                    </tr>
                                                                }
                                                            }).collect_view()}
                                                        </tbody>
                                                    </table>
                                                </div>
                                            </div>
                                        }.into_any()
                                    },
                                    Err(e) => view! {
                                        <ErrorBanner message=format!("Failed to load tendencies: {e}") />
                                    }.into_any(),
                                })}
                            </Suspense>
                        </div>
                    }
                })}
            </div>

            // Draft Analytics Panel (Phase 7c)
            <div class="bg-elevated border border-divider rounded-lg">
                <button
                    class="w-full flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-surface/50 transition-colors"
                    on:click=move |_| set_analytics_open.update(|v| *v = !*v)
                >
                    <h2 class="text-xl font-bold text-primary">"Draft Analytics"</h2>
                    <span class="text-muted">{move || if analytics_open.get() { "\u{25B2}" } else { "\u{25BC}" }}</span>
                </button>
                {move || analytics_open.get().then(|| {
                    view! {
                        <div class="px-4 pb-4">
                            <Suspense fallback=|| view! { <div class="text-dimmed text-sm">"Loading analytics..."</div> }>
                                {move || analytics_data.get().map(|result| match result {
                                    Ok(data) => {
                                        let has_data = data.blue_games + data.red_games > 0
                                            || !data.tag_stats.is_empty()
                                            || !data.first_pick_stats.is_empty();

                                        if !has_data {
                                            return view! {
                                                <p class="text-dimmed text-sm">"No draft outcome data yet. Link drafts to post-game reviews to see analytics."</p>
                                            }.into_any();
                                        }

                                        let blue_wr = if data.blue_games > 0 { data.blue_wins as f64 / data.blue_games as f64 * 100.0 } else { 0.0 };
                                        let red_wr = if data.red_games > 0 { data.red_wins as f64 / data.red_games as f64 * 100.0 } else { 0.0 };

                                        let tag_stats = data.tag_stats.clone();
                                        let fp_stats = data.first_pick_stats.clone();

                                        view! {
                                            <div class="flex flex-col gap-4">
                                                // Side win rates
                                                {if data.blue_games + data.red_games > 0 {
                                                    view! {
                                                        <div class="grid grid-cols-2 gap-3">
                                                            <div class="bg-blue-500/10 border border-blue-500/30 rounded-lg p-3 text-center">
                                                                <p class="text-blue-400 text-xs font-semibold uppercase mb-1">"Blue Side"</p>
                                                                <p class="text-primary text-lg font-bold">{format!("{blue_wr:.0}% WR")}</p>
                                                                <p class="text-muted text-xs">{format!("{}-{}", data.blue_wins, data.blue_games - data.blue_wins)}</p>
                                                            </div>
                                                            <div class="bg-red-500/10 border border-red-500/30 rounded-lg p-3 text-center">
                                                                <p class="text-red-400 text-xs font-semibold uppercase mb-1">"Red Side"</p>
                                                                <p class="text-primary text-lg font-bold">{format!("{red_wr:.0}% WR")}</p>
                                                                <p class="text-muted text-xs">{format!("{}-{}", data.red_wins, data.red_games - data.red_wins)}</p>
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <div></div> }.into_any()
                                                }}

                                                // Tag stats
                                                {if !tag_stats.is_empty() {
                                                    view! {
                                                        <div>
                                                            <h3 class="text-secondary text-sm font-semibold mb-2">"Composition Tag Win Rates"</h3>
                                                            <div class="flex flex-col gap-1">
                                                                {tag_stats.into_iter().map(|(tag, games, wins)| {
                                                                    let wr = if games > 0 { wins as f64 / games as f64 * 100.0 } else { 0.0 };
                                                                    view! {
                                                                        <div class="flex items-center gap-3 bg-surface/30 rounded px-3 py-1.5">
                                                                            <span class="text-primary text-sm font-medium w-32">{tag}</span>
                                                                            <span class="text-secondary text-sm w-16 text-right">{format!("{wr:.0}%")}</span>
                                                                            <span class="text-muted text-xs">{format!("({wins}-{})", games - wins)}</span>
                                                                        </div>
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <div></div> }.into_any()
                                                }}

                                                // First pick stats
                                                {if !fp_stats.is_empty() {
                                                    view! {
                                                        <div>
                                                            <h3 class="text-secondary text-sm font-semibold mb-2">"First Pick Win Rates"</h3>
                                                            <div class="flex flex-col gap-1">
                                                                {fp_stats.into_iter().map(|(champ, games, wins)| {
                                                                    let wr = if games > 0 { wins as f64 / games as f64 * 100.0 } else { 0.0 };
                                                                    view! {
                                                                        <div class="flex items-center gap-3 bg-surface/30 rounded px-3 py-1.5">
                                                                            <span class="text-primary text-sm font-medium w-32">{champ}</span>
                                                                            <span class="text-secondary text-sm w-16 text-right">{format!("{wr:.0}%")}</span>
                                                                            <span class="text-muted text-xs">{format!("({wins}-{})", games - wins)}</span>
                                                                        </div>
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <div></div> }.into_any()
                                                }}
                                            </div>
                                        }.into_any()
                                    },
                                    Err(e) => view! {
                                        <ErrorBanner message=format!("Failed to load analytics: {e}") />
                                    }.into_any(),
                                })}
                            </Suspense>
                        </div>
                    }
                })}
            </div>

            // Ban Priority Panel
            <div class="bg-elevated border border-divider rounded-lg">
                <button
                    class="w-full flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-surface/50 transition-colors"
                    on:click=move |_| set_ban_panel_open.update(|v| *v = !*v)
                >
                    <h2 class="text-xl font-bold text-primary">"Ban Priorities"</h2>
                    <span class="text-muted">{move || if ban_panel_open.get() { "\u{25B2}" } else { "\u{25BC}" }}</span>
                </button>
                {move || ban_panel_open.get().then(|| {
                    let is_editing = editing_bans.get();
                    view! {
                        <div class="px-4 pb-4 flex flex-col gap-3">
                            {if is_editing {
                                view! {
                                    <div class="flex flex-col gap-2">
                                        // Existing items
                                        {move || {
                                            let items = ban_edit_list.get();
                                            if items.is_empty() {
                                                view! { <p class="text-dimmed text-sm">"No ban priorities yet."</p> }.into_any()
                                            } else {
                                                view! {
                                                    <div class="flex flex-col gap-1">
                                                        {items.into_iter().enumerate().map(|(i, bp)| {
                                                            let champ = bp.champion.clone();
                                                            let reason = bp.reason.clone().unwrap_or_default();
                                                            view! {
                                                                <div class="flex items-center gap-2 bg-surface rounded px-3 py-2">
                                                                    <span class="text-accent font-bold text-sm w-6">{format!("#{}", i + 1)}</span>
                                                                    <span class="text-primary font-medium flex-1">{champ}</span>
                                                                    <span class="text-muted text-sm flex-1 truncate">{reason}</span>
                                                                    <button
                                                                        class="text-red-400 hover:text-red-300 text-sm cursor-pointer"
                                                                        on:click=move |_| {
                                                                            set_ban_edit_list.update(|list| {
                                                                                if i < list.len() { list.remove(i); }
                                                                                // Re-rank
                                                                                for (j, item) in list.iter_mut().enumerate() {
                                                                                    item.rank = j as i32;
                                                                                }
                                                                            });
                                                                        }
                                                                    >"Remove"</button>
                                                                </div>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                }.into_any()
                                            }
                                        }}
                                        // Add new entry
                                        <div class="flex gap-2 items-end">
                                            <div class="flex-1">
                                                <label class="block text-secondary text-xs mb-1">"Champion"</label>
                                                <input
                                                    type="text"
                                                    placeholder="Champion name..."
                                                    class="w-full bg-overlay border border-outline rounded px-2 py-1 text-primary text-sm focus:outline-none focus:border-accent"
                                                    prop:value=move || ban_new_champ.get()
                                                    on:input=move |ev| set_ban_new_champ.set(event_target_value(&ev))
                                                />
                                            </div>
                                            <div class="flex-1">
                                                <label class="block text-secondary text-xs mb-1">"Reason (optional)"</label>
                                                <input
                                                    type="text"
                                                    placeholder="Why ban?"
                                                    class="w-full bg-overlay border border-outline rounded px-2 py-1 text-primary text-sm focus:outline-none focus:border-accent"
                                                    prop:value=move || ban_new_reason.get()
                                                    on:input=move |ev| set_ban_new_reason.set(event_target_value(&ev))
                                                />
                                            </div>
                                            <button
                                                class="bg-overlay-strong hover:bg-overlay text-primary text-sm rounded px-3 py-1 transition-colors cursor-pointer"
                                                on:click=move |_| {
                                                    let champ = ban_new_champ.get_untracked();
                                                    if champ.trim().is_empty() { return; }
                                                    let reason_val = ban_new_reason.get_untracked();
                                                    let reason = if reason_val.trim().is_empty() { None } else { Some(reason_val) };
                                                    set_ban_edit_list.update(|list| {
                                                        let rank = list.len() as i32;
                                                        list.push(BanPriority {
                                                            id: None,
                                                            team_id: String::new(),
                                                            champion: champ.trim().to_string(),
                                                            rank,
                                                            reason,
                                                        });
                                                    });
                                                    set_ban_new_champ.set(String::new());
                                                    set_ban_new_reason.set(String::new());
                                                }
                                            >"+ Add"</button>
                                        </div>
                                        // Save / Cancel
                                        <div class="flex gap-2 mt-1">
                                            <button
                                                class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-4 py-1.5 text-sm transition-colors cursor-pointer"
                                                on:click=move |_| {
                                                    let list = ban_edit_list.get_untracked();
                                                    let json = serde_json::to_string(&list).unwrap_or_default();
                                                    leptos::task::spawn_local(async move {
                                                        match save_ban_priorities(json).await {
                                                            Ok(_) => {
                                                                set_ban_status.set(Some("Saved!".into()));
                                                                ban_priorities.refetch();
                                                                set_editing_bans.set(false);
                                                            }
                                                            Err(e) => set_ban_status.set(Some(format!("Error: {e}"))),
                                                        }
                                                    });
                                                }
                                            >"Save"</button>
                                            <button
                                                class="bg-overlay hover:bg-overlay-strong text-secondary rounded px-4 py-1.5 text-sm transition-colors cursor-pointer"
                                                on:click=move |_| set_editing_bans.set(false)
                                            >"Cancel"</button>
                                            {move || ban_status.get().map(|msg| view! {
                                                <span class="text-green-300 text-sm self-center">{msg}</span>
                                            })}
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                // View mode
                                view! {
                                    <div class="flex flex-col gap-2">
                                        <Suspense fallback=|| view! { <div class="text-muted text-sm">"Loading..."</div> }>
                                            {move || ban_priorities.get().map(|result| match result {
                                                Ok(list) if list.is_empty() => view! {
                                                    <p class="text-dimmed text-sm">"No ban priorities set."</p>
                                                }.into_any(),
                                                Ok(list) => view! {
                                                    <div class="flex flex-col gap-1">
                                                        {list.iter().map(|bp| {
                                                            let champ = bp.champion.clone();
                                                            let reason = bp.reason.clone().unwrap_or_default();
                                                            let rank = bp.rank + 1;
                                                            view! {
                                                                <div class="flex items-center gap-2 bg-surface rounded px-3 py-2">
                                                                    <span class="text-accent font-bold text-sm w-6">{format!("#{rank}")}</span>
                                                                    <span class="text-primary font-medium flex-1">{champ}</span>
                                                                    <span class="text-muted text-sm flex-1 truncate">{reason}</span>
                                                                </div>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                }.into_any(),
                                                Err(e) => view! {
                                                    <ErrorBanner message=format!("Failed to load ban priorities: {e}") />
                                                }.into_any(),
                                            })}
                                        </Suspense>
                                        <button
                                            class="bg-overlay-strong hover:bg-overlay text-primary text-sm rounded px-3 py-1.5 transition-colors self-start cursor-pointer"
                                            on:click=move |_| {
                                                // Copy current priorities into edit list
                                                if let Some(Ok(list)) = ban_priorities.get_untracked() {
                                                    set_ban_edit_list.set(list);
                                                } else {
                                                    set_ban_edit_list.set(Vec::new());
                                                }
                                                set_ban_status.set(None);
                                                set_editing_bans.set(true);
                                            }
                                        >"Edit"</button>
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
