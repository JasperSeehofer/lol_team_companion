use crate::components::champion_autocomplete::ChampionAutocomplete;
use crate::components::draft_board::slot_meta;
use crate::components::ui::{ErrorBanner, SkeletonCard, SkeletonGrid, SkeletonLine, ToastContext, ToastKind};
use crate::models::champion::Champion;
use crate::models::draft::Draft;
use crate::models::game_plan::{ChecklistInstance, ChecklistTemplate, GamePlan};
use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn list_plans() -> Result<Vec<GamePlan>, ServerFnError> {
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

    db::list_game_plans(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_team_drafts() -> Result<Vec<Draft>, ServerFnError> {
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

    db::list_drafts(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_plan(plan_json: String) -> Result<String, ServerFnError> {
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

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    let mut plan: GamePlan = serde_json::from_str(&plan_json)
        .map_err(|e| ServerFnError::new(format!("Invalid plan JSON: {e}")))?;
    plan.team_id = team_id;

    db::save_game_plan(&surreal, plan, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_plan(plan_json: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let plan: GamePlan = serde_json::from_str(&plan_json)
        .map_err(|e| ServerFnError::new(format!("Invalid plan JSON: {e}")))?;

    db::update_game_plan(&surreal, plan)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_plan(plan_id: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::delete_game_plan(&surreal, &plan_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_champions_for_game_plan() -> Result<Vec<Champion>, ServerFnError> {
    use crate::server::data_dragon;
    data_dragon::fetch_champions()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Fetch a draft with its actions by ID, for prefilling a new game plan.
/// Returns None if the draft does not exist.
#[server]
pub async fn get_draft_for_prefill(
    draft_id: String,
) -> Result<Option<crate::models::draft::Draft>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::get_draft_for_prefill(&surreal, &draft_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Check whether any game plan already references the given draft ID.
/// Returns Some(plan_id) if at least one plan exists, or None if none do.
/// Used for duplicate detection in the "Prep for This Draft" CTA.
#[server]
pub async fn check_draft_has_game_plan(
    draft_id: String,
) -> Result<Option<String>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let plans = db::get_game_plans_for_draft(&surreal, &draft_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(plans.into_iter().next().and_then(|p| p.id))
}

#[server]
pub async fn get_checklist_templates() -> Result<Vec<ChecklistTemplate>, ServerFnError> {
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

    db::list_checklist_templates(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn save_checklist_template(
    name: String,
    items_json: String,
) -> Result<String, ServerFnError> {
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

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    let items: Vec<String> = serde_json::from_str(&items_json)
        .map_err(|e| ServerFnError::new(format!("Invalid items JSON: {e}")))?;

    db::create_checklist_template(&surreal, &team_id, name, items)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_checklist_template_fn(id: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::delete_checklist_template(&surreal, &id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_plan_checklist(plan_id: String) -> Result<Option<ChecklistInstance>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::get_checklist_for_plan(&surreal, &plan_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_plan_checklist(
    plan_id: String,
    template_id: Option<String>,
    items_json: String,
) -> Result<String, ServerFnError> {
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

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    let items: Vec<String> = serde_json::from_str(&items_json)
        .map_err(|e| ServerFnError::new(format!("Invalid items JSON: {e}")))?;

    db::create_checklist_instance(&surreal, &team_id, Some(plan_id), template_id, items)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_checklist(
    instance_id: String,
    checked_json: String,
) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let checked: Vec<bool> = serde_json::from_str(&checked_json)
        .map_err(|e| ServerFnError::new(format!("Invalid checked JSON: {e}")))?;

    db::update_checklist_checked(&surreal, &instance_id, checked)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn start_post_game_review(
    plan_id: String,
    draft_id: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use crate::models::game_plan::PostGameLearning;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    let review = PostGameLearning {
        id: None,
        team_id,
        match_riot_id: None,
        game_plan_id: Some(plan_id),
        draft_id,
        what_went_well: Vec::new(),
        improvements: Vec::new(),
        action_items: Vec::new(),
        open_notes: None,
        created_by: user.id,
    };

    db::save_post_game_learning(&surreal, review)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ---------------------------------------------------------------------------
// Template generation
// ---------------------------------------------------------------------------

fn generate_template(
    our: &[String],
    enemy: &[String],
) -> (Vec<String>, Vec<String>, String, String) {
    let mut win_conditions = Vec::new();
    let objectives = vec![
        "Dragon".to_string(),
        "Rift Herald".to_string(),
        "Baron".to_string(),
    ];

    // Simple heuristic based on champion count (placeholder for real analysis)
    if !our.is_empty() {
        win_conditions.push(format!(
            "Play around {} in teamfights",
            our.first().unwrap_or(&"carry".to_string())
        ));
    }
    if our.len() >= 3 {
        win_conditions.push("Control vision around neutral objectives before fights".to_string());
    }
    if !enemy.is_empty() {
        win_conditions.push(format!(
            "Deny {} from scaling",
            enemy.last().unwrap_or(&"enemy carry".to_string())
        ));
    }
    if win_conditions.is_empty() {
        win_conditions.push("Secure early game advantages through proactive plays".to_string());
        win_conditions.push("Transition leads into objective control".to_string());
    }

    let teamfight = if our.len() >= 5 {
        format!(
            "Front-to-back: protect {} while {} zones the enemy",
            our[3], our[0]
        )
    } else {
        "Focus priority targets and peel for carries".to_string()
    };

    let early = "Contest level 1 vision. Jungle path based on matchup. Look for early ganks on volatile lanes.".to_string();

    (win_conditions, objectives, teamfight, early)
}

#[server]
pub async fn get_strategy_win_rates() -> Result<Vec<(String, i32, i32)>, ServerFnError> {
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

    db::get_win_condition_stats(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_strategy_win_rates_vs_opponent(
    opponent_name: String,
) -> Result<Vec<(String, i32, i32)>, ServerFnError> {
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

    db::get_win_condition_stats_vs_opponent(&surreal, &team_id, &opponent_name)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const STRATEGY_TAGS: &[&str] = &[
    "teamfight",
    "split-push",
    "poke",
    "pick",
    "scaling",
    "early-game",
    "protect-the-carry",
];

const ROLES: [&str; 5] = ["Top", "Jungle", "Mid", "Bot", "Support"];

fn textarea_class() -> &'static str {
    "w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus:outline-none focus:border-accent/50 resize-none transition-colors"
}

fn input_class() -> &'static str {
    "w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus:outline-none focus:border-accent/50 transition-colors"
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn GamePlanPage() -> impl IntoView {
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

    // URL query param: ?draft_id=X for prefill
    use leptos_router::hooks::use_query_map;
    let query = use_query_map();

    // Prefill Resource keyed on draft_id query param
    let prefill_data = Resource::new(
        move || query.read().get("draft_id"),
        |draft_id_opt| async move {
            match draft_id_opt {
                Some(id) if !id.is_empty() => get_draft_for_prefill(id).await,
                _ => Ok(None),
            }
        },
    );

    let prefill_applied = RwSignal::new(false);
    let (champs_locked, set_champs_locked) = signal(false);

    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let plans = Resource::new(|| (), |_| list_plans());
    let drafts = Resource::new(|| (), |_| list_team_drafts());
    let champions = Resource::new(|| (), |_| get_champions_for_game_plan());

    // Editor state
    let (editing_id, set_editing_id) = signal(Option::<String>::None);
    let (plan_name, set_plan_name) = signal(String::new());
    let (draft_id, set_draft_id) = signal(String::new());
    let our_champ_signals: Vec<RwSignal<String>> =
        (0..5).map(|_| RwSignal::new(String::new())).collect();
    let enemy_champ_signals: Vec<RwSignal<String>> =
        (0..5).map(|_| RwSignal::new(String::new())).collect();
    let (win_conditions, set_win_conditions) = signal(String::new());
    let (obj_priority, set_obj_priority) = signal(String::new());
    let (teamfight, set_teamfight) = signal(String::new());
    let (early_game, set_early_game) = signal(String::new());
    let (role_strats, set_role_strats) = signal(vec![String::new(); 5]);
    let (notes, set_notes) = signal(String::new());
    let (win_condition_tag, set_win_condition_tag) = signal(String::new());
    let strategy_win_rates = Resource::new(|| (), |_| get_strategy_win_rates());

    // Win condition tracker state
    let (tracker_open, set_tracker_open) = signal(true);
    let (tracker_vs_opponent, set_tracker_vs_opponent) = signal(false);
    // opponent_win_rates loads when vs-opponent tab is active and an opponent is linked to the draft
    let opponent_win_rates = Resource::new(
        move || {
            let show = tracker_vs_opponent.get();
            let opp = drafts
                .get()
                .and_then(|r| r.ok())
                .unwrap_or_default()
                .into_iter()
                .find(|d| d.id.as_deref() == Some(&draft_id.get()))
                .and_then(|d| d.opponent.clone())
                .unwrap_or_default();
            (show, opp)
        },
        move |(show, opp)| async move {
            if !show || opp.is_empty() {
                Ok(Vec::<(String, i32, i32)>::new())
            } else {
                get_strategy_win_rates_vs_opponent(opp).await
            }
        },
    );

    // Prefill Effect: seed editor from draft when ?draft_id=X is set
    let our_sigs_for_prefill = our_champ_signals.clone();
    let enemy_sigs_for_prefill = enemy_champ_signals.clone();
    Effect::new(move |_| {
        if prefill_applied.get() {
            return;
        }
        if let Some(Ok(Some(draft))) = prefill_data.get() {
            let our_side = draft.our_side.clone();
            let enemy_side = if our_side == "blue" { "red" } else { "blue" };
            let mut our_picks: Vec<String> = draft
                .actions
                .iter()
                .filter(|a| a.side == our_side && a.phase.contains("pick"))
                .map(|a| a.champion.clone())
                .collect();
            let mut enemy_picks: Vec<String> = draft
                .actions
                .iter()
                .filter(|a| a.side == enemy_side && a.phase.contains("pick"))
                .map(|a| a.champion.clone())
                .collect();
            our_picks.resize(5, String::new());
            enemy_picks.resize(5, String::new());
            for (i, s) in our_sigs_for_prefill.iter().enumerate() {
                s.set(our_picks[i].clone());
            }
            for (i, s) in enemy_sigs_for_prefill.iter().enumerate() {
                s.set(enemy_picks[i].clone());
            }
            // Seed other fields from draft
            if let Some(wc) = &draft.win_conditions {
                if !wc.is_empty() {
                    set_win_conditions.set(wc.clone());
                }
            }
            if let Some(n) = &draft.notes {
                if !n.is_empty() {
                    set_notes.set(n.clone());
                }
            }
            // Set draft FK so the plan saves with the link
            if let Some(did) = &draft.id {
                set_draft_id.set(did.clone());
            }
            prefill_applied.set(true);
            set_champs_locked.set(true);
        }
    });

    // Clear editor (Callback is Copy, safe in multiple closures)
    let our_champ_signals_clone = our_champ_signals.clone();
    let enemy_champ_signals_clone = enemy_champ_signals.clone();
    let clear_editor = Callback::new(move |_: ()| {
        set_editing_id.set(None);
        set_plan_name.set(String::new());
        set_draft_id.set(String::new());
        for s in &our_champ_signals_clone {
            s.set(String::new());
        }
        for s in &enemy_champ_signals_clone {
            s.set(String::new());
        }
        set_win_conditions.set(String::new());
        set_obj_priority.set(String::new());
        set_teamfight.set(String::new());
        set_early_game.set(String::new());
        set_role_strats.set(vec![String::new(); 5]);
        set_notes.set(String::new());
        set_win_condition_tag.set(String::new());
    });

    // Load a plan into editor (Callback is Copy, safe in reactive closures)
    let our_sigs_for_load = our_champ_signals.clone();
    let enemy_sigs_for_load = enemy_champ_signals.clone();
    let load_plan = Callback::new(move |p: GamePlan| {
        set_editing_id.set(p.id.clone());
        set_plan_name.set(p.name.clone());
        set_draft_id.set(p.draft_id.clone().unwrap_or_default());
        let mut ours = p.our_champions.clone();
        ours.resize(5, String::new());
        for (i, s) in our_sigs_for_load.iter().enumerate() {
            s.set(ours[i].clone());
        }
        let mut theirs = p.enemy_champions.clone();
        theirs.resize(5, String::new());
        for (i, s) in enemy_sigs_for_load.iter().enumerate() {
            s.set(theirs[i].clone());
        }
        set_win_conditions.set(p.win_conditions.join("\n"));
        set_obj_priority.set(p.objective_priority.join("\n"));
        set_teamfight.set(p.teamfight_strategy.clone());
        set_early_game.set(p.early_game.clone().unwrap_or_default());
        set_role_strats.set(vec![
            p.top_strategy.clone().unwrap_or_default(),
            p.jungle_strategy.clone().unwrap_or_default(),
            p.mid_strategy.clone().unwrap_or_default(),
            p.bot_strategy.clone().unwrap_or_default(),
            p.support_strategy.clone().unwrap_or_default(),
        ]);
        set_notes.set(p.notes.clone().unwrap_or_default());
        set_win_condition_tag.set(p.win_condition_tag.clone().unwrap_or_default());
    });

    // BUG-02/PLAN-02: Auto-load a specific plan when ?plan_id=X is in the URL.
    // When navigating from "View Game Plan" on the draft page, the URL includes
    // plan_id=<id>. This Effect finds the matching plan in the loaded list and
    // calls load_plan to populate the editor, so the correct plan opens immediately.
    let plan_id_from_url = query.with(|q| q.get("plan_id").map(|s| s.clone()));
    let plan_id_applied = RwSignal::new(false);
    {
        let plan_id_from_url = plan_id_from_url.clone();
        Effect::new(move |_| {
            if plan_id_applied.get() {
                return;
            }
            let Some(ref target_id) = plan_id_from_url else { return };
            if target_id.is_empty() {
                return;
            }
            if let Some(Ok(list)) = plans.get() {
                if let Some(plan) = list.into_iter().find(|p| p.id.as_deref() == Some(target_id.as_str())) {
                    load_plan.run(plan);
                    plan_id_applied.set(true);
                }
            }
        });
    }

    let our_sigs_for_build = our_champ_signals.clone();
    let enemy_sigs_for_build = enemy_champ_signals.clone();
    let build_plan = move || -> GamePlan {
        let strats = role_strats.get_untracked();
        GamePlan {
            id: editing_id.get_untracked(),
            team_id: String::new(), // filled by server
            draft_id: {
                let d = draft_id.get_untracked();
                if d.is_empty() {
                    None
                } else {
                    Some(d)
                }
            },
            name: plan_name.get_untracked(),
            our_champions: our_sigs_for_build
                .iter()
                .map(|s| s.get_untracked())
                .filter(|s| !s.is_empty())
                .collect(),
            enemy_champions: enemy_sigs_for_build
                .iter()
                .map(|s| s.get_untracked())
                .filter(|s| !s.is_empty())
                .collect(),
            win_conditions: win_conditions
                .get_untracked()
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            objective_priority: obj_priority
                .get_untracked()
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            teamfight_strategy: teamfight.get_untracked(),
            early_game: {
                let s = early_game.get_untracked();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            top_strategy: {
                let s = strats[0].clone();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            jungle_strategy: {
                let s = strats[1].clone();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            mid_strategy: {
                let s = strats[2].clone();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            bot_strategy: {
                let s = strats[3].clone();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            support_strategy: {
                let s = strats[4].clone();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            notes: {
                let s = notes.get_untracked();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            win_condition_tag: {
                let s = win_condition_tag.get_untracked();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
        }
    };

    // Auto-save timer + status (same pattern as draft.rs / tree_drafter.rs)
    #[allow(unused_variables)]
    let auto_save_timer: RwSignal<Option<i32>> = RwSignal::new(None);
    let (auto_save_status, set_auto_save_status) = signal("");

    let our_sigs_for_autosave = our_champ_signals.clone();
    let enemy_sigs_for_autosave = enemy_champ_signals.clone();

    #[allow(unused_variables)]
    Effect::new(move |_| {
        // === RULE 54: Eagerly track + capture ALL signals ===
        let name_val = plan_name.get();
        let existing_id = editing_id.get();
        let draft_val = draft_id.get();
        let wc_val = win_conditions.get();
        let obj_val = obj_priority.get();
        let tf_val = teamfight.get();
        let eg_val = early_game.get();
        let strats_val = role_strats.get();
        let notes_val = notes.get();
        let wct_val = win_condition_tag.get();
        let our_vals: Vec<String> = our_sigs_for_autosave.iter().map(|s| s.get()).collect();
        let enemy_vals: Vec<String> = enemy_sigs_for_autosave.iter().map(|s| s.get()).collect();

        // Only auto-save if we have a name AND it's an existing plan
        if name_val.trim().is_empty() || existing_id.is_none() {
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

        #[cfg(feature = "hydrate")]
        {
            // Build plan from eagerly captured values
            let plan = GamePlan {
                id: existing_id,
                team_id: String::new(),
                draft_id: if draft_val.is_empty() { None } else { Some(draft_val) },
                name: name_val,
                our_champions: our_vals.into_iter().filter(|s| !s.is_empty()).collect(),
                enemy_champions: enemy_vals.into_iter().filter(|s| !s.is_empty()).collect(),
                win_conditions: wc_val.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                objective_priority: obj_val.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                teamfight_strategy: tf_val,
                early_game: if eg_val.is_empty() { None } else { Some(eg_val) },
                top_strategy: if strats_val[0].is_empty() { None } else { Some(strats_val[0].clone()) },
                jungle_strategy: if strats_val[1].is_empty() { None } else { Some(strats_val[1].clone()) },
                mid_strategy: if strats_val[2].is_empty() { None } else { Some(strats_val[2].clone()) },
                bot_strategy: if strats_val[3].is_empty() { None } else { Some(strats_val[3].clone()) },
                support_strategy: if strats_val[4].is_empty() { None } else { Some(strats_val[4].clone()) },
                notes: if notes_val.is_empty() { None } else { Some(notes_val) },
                win_condition_tag: if wct_val.is_empty() { None } else { Some(wct_val) },
            };
            let plan_json = serde_json::to_string(&plan).unwrap_or_default();

            use wasm_bindgen::prelude::*;
            let cb = Closure::once(move || {
                leptos::task::spawn_local(async move {
                    let _ = update_plan(plan_json).await;
                    set_auto_save_status.set("saved");
                    plans.refetch();
                    strategy_win_rates.refetch();
                });
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

    let do_save = move |_| {
        let plan = build_plan();
        let plan_json = serde_json::to_string(&plan).unwrap_or_default();
        let is_update = editing_id.get_untracked().is_some();

        leptos::task::spawn_local(async move {
            let result = if is_update {
                update_plan(plan_json).await.map(|_| String::new())
            } else {
                create_plan(plan_json).await
            };
            match result {
                Ok(id) => {
                    if !is_update && !id.is_empty() {
                        set_editing_id.set(Some(id));
                    }
                    toast.show.run((ToastKind::Success, "Game plan saved".into()));
                    plans.refetch();
                }
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        });
    };

    let do_delete = Callback::new(move |plan_id: String| {
        leptos::task::spawn_local(async move {
            match delete_plan(plan_id).await {
                Ok(_) => {
                    clear_editor.run(());
                    toast.show.run((ToastKind::Success, "Game plan deleted".into()));
                    plans.refetch();
                }
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        });
    });

    let our_sigs_for_template = our_champ_signals.clone();
    let enemy_sigs_for_template = enemy_champ_signals.clone();
    let do_generate_template = move |_| {
        let ours: Vec<String> = our_sigs_for_template
            .iter()
            .map(|s| s.get_untracked())
            .collect();
        let theirs: Vec<String> = enemy_sigs_for_template
            .iter()
            .map(|s| s.get_untracked())
            .collect();
        let (wc, obj, tf, eg) = generate_template(&ours, &theirs);
        set_win_conditions.set(wc.join("\n"));
        set_obj_priority.set(obj.join("\n"));
        set_teamfight.set(tf);
        set_early_game.set(eg);
        toast.show.run((ToastKind::Success, "Template generated! Customize to your strategy.".into()));
    };

    // Pre-clone for multiple move closures in view!
    let our_champs_for_draft = our_champ_signals.clone();
    let enemy_champs_for_draft = enemy_champ_signals.clone();
    let our_champs_for_matchup = our_champ_signals.clone();
    let enemy_champs_for_matchup = enemy_champ_signals.clone();
    // our_champ_signals/enemy_champ_signals used last for role-specific section

    view! {
        <div class="max-w-[80rem] mx-auto py-8 px-6 flex flex-col gap-6">
            <div>
                <h1 class="text-3xl font-bold text-primary">"Game Plans"</h1>
                <p class="text-muted text-sm mt-1">"Strategic plans for specific champion matchups"</p>
            </div>

            <div class="flex gap-6 min-h-[36rem]">
                // Left: plan list
                <div class="w-72 flex-shrink-0 flex flex-col gap-3">
                    <button
                        class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm transition-colors"
                        on:click=move |_| clear_editor.run(())
                    >"+ New Plan"</button>

                    <Suspense fallback=|| view! { <div class="flex flex-col gap-2"><SkeletonCard height="h-10" /><SkeletonCard height="h-10" /><SkeletonCard height="h-10" /></div> }>
                        {move || plans.get().map(|result| match result {
                            Ok(list) if list.is_empty() => view! {
                                <div class="text-center py-6">
                                    <p class="text-dimmed text-sm mb-3">"No game plans yet"</p>
                                    <p class="text-dimmed text-xs mb-4">"Create or join a team to get started."</p>
                                    <a href="/team/roster" class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-3 py-1.5 text-xs transition-colors">
                                        "Go to Team"
                                    </a>
                                </div>
                            }.into_any(),
                            Ok(list) => view! {
                                <div class="flex flex-col gap-1.5">
                                    {list.into_iter().map(|p| {
                                        let plan_for_load = p.clone();
                                        let plan_id = p.id.clone().unwrap_or_default();
                                        let plan_id_for_cls = plan_id.clone();
                                        let plan_id_for_delete = plan_id.clone();
                                        let name = if p.name.is_empty() { "Untitled".to_string() } else { p.name.clone() };
                                        let champ_summary = if !p.our_champions.is_empty() && !p.enemy_champions.is_empty() {
                                            format!("{} vs {}", p.our_champions.len(), p.enemy_champions.len())
                                        } else {
                                            String::new()
                                        };

                                        view! {
                                            <div class=move || {
                                                let sel = editing_id.get();
                                                if sel.as_deref() == Some(&plan_id_for_cls) {
                                                    "bg-accent/10 border border-accent/30 rounded-lg p-3 transition-all"
                                                } else {
                                                    "bg-elevated/30 border border-divider/30 rounded-lg p-3 hover:bg-overlay/30 transition-all"
                                                }
                                            }>
                                                <button
                                                    class="w-full text-left"
                                                    on:click=move |_| load_plan.run(plan_for_load.clone())
                                                >
                                                    <div class="text-primary text-sm font-medium truncate">{name}</div>
                                                    {(!champ_summary.is_empty()).then(|| view! {
                                                        <div class="text-dimmed text-xs mt-0.5">{champ_summary} " champs"</div>
                                                    })}
                                                </button>
                                                <button
                                                    class="text-red-400/50 hover:text-red-400 text-xs mt-1 transition-colors"
                                                    on:click=move |_| do_delete.run(plan_id_for_delete.clone())
                                                >"Delete"</button>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any(),
                            Err(e) => view! {
                                <ErrorBanner message=format!("Failed to load plans: {e}") />
                            }.into_any(),
                        })}
                    </Suspense>
                </div>

                // Right: editor
                <div class="flex-1 flex flex-col gap-5">
                    // Plan name + draft link
                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4 flex flex-col gap-4">
                        // Back-reference badge: shown when a draft is linked
                        {move || {
                            let did = draft_id.get();
                            if did.is_empty() {
                                view! { <span></span> }.into_any()
                            } else {
                                view! {
                                    <div class="flex items-center gap-2">
                                        <a
                                            href=format!("/draft?draft_id={did}")
                                            class="inline-flex items-center gap-1 bg-surface border border-outline/50 text-muted text-xs rounded px-2 py-1 hover:text-primary hover:border-accent/50 transition-colors"
                                        >
                                            <span class="text-accent">"Source Draft"</span>
                                            <span class="text-dimmed">"- click to open"</span>
                                        </a>
                                        {move || if champs_locked.get() {
                                            view! {
                                                <span class="text-xs text-muted bg-surface border border-outline/30 rounded px-2 py-0.5">
                                                    "Champions pre-filled from draft"
                                                </span>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>
                                }.into_any()
                            }
                        }}
                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label class="block text-muted text-xs font-medium mb-1">"Plan Name"</label>
                                <input type="text" class=input_class()
                                    placeholder="e.g. Comp A vs Scaling"
                                    prop:value=move || plan_name.get()
                                    on:input=move |ev| set_plan_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-muted text-xs font-medium mb-1">"Linked Draft (optional)"</label>
                                <Suspense fallback=|| view! { <SkeletonLine width="w-full" height="h-9" /> }>
                                    {move || {
                                        let our_sigs = our_champs_for_draft.clone();
                                        let enemy_sigs = enemy_champs_for_draft.clone();
                                        drafts.get().map(move |result| match result {
                                            Ok(list) => {
                                                let list_for_handler = list.clone();
                                                let our_sigs = our_sigs.clone();
                                                let enemy_sigs = enemy_sigs.clone();
                                                view! {
                                                    <select class=input_class()
                                                        prop:value=move || draft_id.get()
                                                        on:change=move |ev| {
                                                            let selected_id = event_target_value(&ev);
                                                            set_draft_id.set(selected_id.clone());
                                                            // Auto-populate champions from the selected draft
                                                            if let Some(draft) = list_for_handler.iter().find(|d| d.id.as_deref() == Some(&selected_id)) {
                                                                let our_side = &draft.our_side;
                                                                let enemy_side = if our_side == "blue" { "red" } else { "blue" };
                                                                let mut our_picks: Vec<String> = Vec::new();
                                                                let mut enemy_picks: Vec<String> = Vec::new();
                                                                for action in &draft.actions {
                                                                    let (_side, kind, _) = slot_meta(action.order as usize);
                                                                    if !kind.contains("pick") { continue; }
                                                                    if action.side == *our_side {
                                                                        our_picks.push(action.champion.clone());
                                                                    } else if action.side == *enemy_side {
                                                                        enemy_picks.push(action.champion.clone());
                                                                    }
                                                                }
                                                                our_picks.resize(5, String::new());
                                                                enemy_picks.resize(5, String::new());
                                                                for (i, s) in our_sigs.iter().enumerate() { s.set(our_picks[i].clone()); }
                                                                for (i, s) in enemy_sigs.iter().enumerate() { s.set(enemy_picks[i].clone()); }
                                                            }
                                                        }
                                                    >
                                                        <option value="">"None"</option>
                                                        {list.into_iter().map(|d| {
                                                            let id = d.id.clone().unwrap_or_default();
                                                            let label = d.name.clone();
                                                            view! { <option value=id>{label}</option> }
                                                        }).collect_view()}
                                                    </select>
                                                }.into_any()
                                            },
                                            Err(_) => view! { <p class="text-dimmed text-sm">"No drafts"</p> }.into_any(),
                                        })
                                    }}
                                </Suspense>
                            </div>
                        </div>
                    </div>

                    // Matchup: Your 5 vs Enemy 5
                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4">
                        <div class="flex items-center justify-between mb-3">
                            <h3 class="text-primary font-semibold text-sm">"Champion Matchup"</h3>
                            <button
                                class="bg-overlay hover:bg-overlay-strong text-secondary text-xs font-medium px-3 py-1.5 rounded-lg transition-colors cursor-pointer"
                                on:click=do_generate_template
                            >"Generate Template"</button>
                        </div>
                        <Suspense fallback=|| view! { <SkeletonGrid cols=3 rows=1 card_height="h-8" /> }>
                            {move || {
                                let our_sigs = our_champs_for_matchup.clone();
                                let enemy_sigs = enemy_champs_for_matchup.clone();
                                champions.get().map(move |result| {
                                    let champ_list = result.unwrap_or_default();
                                    let champ_list2 = champ_list.clone();
                                    view! {
                                        <div class="grid grid-cols-[1fr_auto_1fr] gap-4 items-start">
                                            // Our team
                                            <div class="flex flex-col gap-2">
                                                <div class="flex items-center justify-between">
                                                    <span class="text-blue-400 text-xs font-semibold uppercase">"Your Team"</span>
                                                    {move || if champs_locked.get() {
                                                        view! {
                                                            <button
                                                                class="text-muted hover:text-accent text-xs transition-colors"
                                                                on:click=move |_| set_champs_locked.set(false)
                                                            >"Edit"</button>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }}
                                                </div>
                                                {our_sigs.iter().enumerate().map(|(i, sig)| {
                                                    let role = ROLES[i];
                                                    let champs_locked_branch = champ_list.clone();
                                                    let sig_for_lock = *sig;
                                                    view! {
                                                        <div class="flex items-center gap-2">
                                                            <span class="text-dimmed text-xs w-14">{role}</span>
                                                            {move || {
                                                                let champs_for_input = champs_locked_branch.clone();
                                                                if champs_locked.get() {
                                                                    let val = sig_for_lock.get();
                                                                    let display = if val.is_empty() {
                                                                        "-".to_string()
                                                                    } else {
                                                                        champs_for_input.iter()
                                                                            .find(|c| c.id == val)
                                                                            .map(|c| c.name.clone())
                                                                            .unwrap_or(val)
                                                                    };
                                                                    view! {
                                                                        <div class="flex-1 bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm">
                                                                            {display}
                                                                        </div>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <ChampionAutocomplete
                                                                            champions=champs_for_input
                                                                            value=sig_for_lock
                                                                            placeholder=role
                                                                        />
                                                                    }.into_any()
                                                                }
                                                            }}
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                            <div class="flex items-center justify-center self-center text-dimmed font-bold text-sm pt-6">"VS"</div>
                                            // Enemy team
                                            <div class="flex flex-col gap-2">
                                                <span class="text-red-400 text-xs font-semibold uppercase">"Enemy Team"</span>
                                                {enemy_sigs.iter().enumerate().map(|(i, sig)| {
                                                    let role = ROLES[i];
                                                    let champs = champ_list2.clone();
                                                    view! {
                                                        <div class="flex items-center gap-2">
                                                            <span class="text-dimmed text-xs w-14">{role}</span>
                                                            <ChampionAutocomplete
                                                                champions=champs
                                                                value=*sig
                                                                placeholder=role
                                                            />
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    }
                                })
                            }}
                        </Suspense>
                    </div>

                    // Strategy Tag
                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4 flex flex-col gap-3">
                        <h3 class="text-primary font-semibold text-sm">"Strategy Tag"</h3>
                        <div class="flex flex-wrap gap-2">
                            {STRATEGY_TAGS.iter().map(|&tag| {
                                let tag_str = tag.to_string();
                                let tag_for_cls = tag_str.clone();
                                view! {
                                    <button
                                        class=move || {
                                            let current = win_condition_tag.get();
                                            if current == tag_for_cls {
                                                "px-3 py-1.5 rounded-lg text-sm font-medium bg-accent text-accent-contrast transition-colors cursor-pointer"
                                            } else {
                                                "px-3 py-1.5 rounded-lg text-sm font-medium bg-overlay hover:bg-overlay-strong text-secondary transition-colors cursor-pointer"
                                            }
                                        }
                                        on:click=move |_| {
                                            let current = win_condition_tag.get_untracked();
                                            if current == tag_str {
                                                set_win_condition_tag.set(String::new());
                                            } else {
                                                set_win_condition_tag.set(tag_str.clone());
                                            }
                                        }
                                    >{tag}</button>
                                }
                            }).collect_view()}
                        </div>
                        // Show historical win rate for selected tag
                        <Suspense fallback=|| ()>
                            {move || {
                                let tag = win_condition_tag.get();
                                if tag.is_empty() {
                                    return None;
                                }
                                strategy_win_rates.get().map(|result| {
                                    match result {
                                        Ok(rates) => {
                                            if let Some((_, games, wins)) = rates.iter().find(|(t, _, _)| *t == tag) {
                                                if *games > 0 {
                                                    view! {
                                                        <div class="text-sm text-muted">
                                                            {format!("Your team is {wins}-{losses} with this strategy", losses = games - wins)}
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="text-sm text-dimmed">"No game results yet for this strategy"</div>
                                                    }.into_any()
                                                }
                                            } else {
                                                view! {
                                                    <div class="text-sm text-dimmed">"No game results yet for this strategy"</div>
                                                }.into_any()
                                            }
                                        },
                                        Err(_) => view! { <div></div> }.into_any(),
                                    }
                                })
                            }}
                        </Suspense>
                    </div>

                    // Win Condition Tracker panel
                    <div class="bg-surface border border-divider rounded-lg">
                        // Panel header with collapse toggle and tab switcher
                        <div class="flex items-center justify-between px-4 py-3 border-b border-divider/50">
                            <button
                                class="flex items-center gap-2 text-primary font-medium text-sm hover:text-accent transition-colors"
                                on:click=move |_| set_tracker_open.update(|v| *v = !*v)
                            >
                                {move || if tracker_open.get() { "\u{25bc}" } else { "\u{25b6}" }}
                                " Win Condition History"
                            </button>
                            {move || if tracker_open.get() {
                                // Determine if an opponent is linked for enabling vs tab
                                let linked_opponent = drafts
                                    .get()
                                    .and_then(|r| r.ok())
                                    .unwrap_or_default()
                                    .into_iter()
                                    .find(|d| d.id.as_deref() == Some(&draft_id.get()))
                                    .and_then(|d| d.opponent.clone())
                                    .unwrap_or_default();
                                let has_opponent = !linked_opponent.is_empty();
                                let vs_label = if has_opponent {
                                    format!("vs {linked_opponent}")
                                } else {
                                    "vs Opponent".to_string()
                                };
                                view! {
                                    <div class="flex items-center gap-1">
                                        <button
                                            class=move || {
                                                if !tracker_vs_opponent.get() {
                                                    "px-2.5 py-1 rounded text-xs font-medium bg-accent text-accent-contrast transition-colors"
                                                } else {
                                                    "px-2.5 py-1 rounded text-xs font-medium bg-overlay hover:bg-overlay-strong text-secondary transition-colors"
                                                }
                                            }
                                            on:click=move |_| set_tracker_vs_opponent.set(false)
                                        >"All-Time"</button>
                                        <button
                                            class=move || {
                                                let active = tracker_vs_opponent.get();
                                                if !has_opponent {
                                                    "px-2.5 py-1 rounded text-xs font-medium bg-overlay text-dimmed cursor-not-allowed opacity-50"
                                                } else if active {
                                                    "px-2.5 py-1 rounded text-xs font-medium bg-accent text-accent-contrast transition-colors"
                                                } else {
                                                    "px-2.5 py-1 rounded text-xs font-medium bg-overlay hover:bg-overlay-strong text-secondary transition-colors"
                                                }
                                            }
                                            disabled=!has_opponent
                                            on:click=move |_| {
                                                if has_opponent {
                                                    set_tracker_vs_opponent.set(true);
                                                }
                                            }
                                        >{vs_label}</button>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                        </div>
                        // Panel body
                        {move || if tracker_open.get() {
                            let current_tag = win_condition_tag.get();
                            let show_vs = tracker_vs_opponent.get();
                            view! {
                                <div class="p-4">
                                    <Suspense fallback=|| view! { <SkeletonCard height="h-32" /> }>
                                        {move || {
                                            let rates_result = if show_vs {
                                                opponent_win_rates.get()
                                            } else {
                                                strategy_win_rates.get()
                                            };
                                            let current_tag_inner = current_tag.clone();
                                            rates_result.map(|result| match result {
                                                Ok(rates) if rates.is_empty() => view! {
                                                    <div class="text-center py-4">
                                                        <p class="text-muted text-sm">"No win condition data yet."</p>
                                                        <p class="text-dimmed text-xs mt-1">
                                                            "Complete post-game reviews with tagged win conditions to see trends here."
                                                        </p>
                                                    </div>
                                                }.into_any(),
                                                Ok(rates) => {
                                                    let max_games = rates.iter().map(|(_, t, _)| *t).max().unwrap_or(1).max(1);
                                                    view! {
                                                        <div class="flex flex-col divide-y divide-divider/30">
                                                            {rates.into_iter().map(|(tag, total, wins)| {
                                                                let win_pct = if total > 0 { (wins * 100) / total } else { 0 };
                                                                let bar_pct = if max_games > 0 { (total * 100) / max_games } else { 0 };
                                                                let is_current = tag == current_tag_inner;
                                                                let bar_color = if win_pct > 60 {
                                                                    "bg-emerald-600"
                                                                } else if win_pct >= 40 {
                                                                    "bg-amber-500"
                                                                } else {
                                                                    "bg-red-600"
                                                                };
                                                                let row_class = if is_current {
                                                                    "py-2 px-2 rounded bg-accent/10 border-l-2 border-accent flex items-center gap-3"
                                                                } else {
                                                                    "py-2 flex items-center gap-3"
                                                                };
                                                                view! {
                                                                    <div class=row_class>
                                                                        <span class="text-primary text-xs font-medium w-36 shrink-0 truncate">{tag}</span>
                                                                        <div class="flex-1 bg-overlay/50 rounded-full h-2 overflow-hidden">
                                                                            <div
                                                                                class=format!("{bar_color} h-2 rounded-full transition-all")
                                                                                style=format!("width: {bar_pct}%")
                                                                            ></div>
                                                                        </div>
                                                                        <span class="text-muted text-xs w-12 text-right shrink-0">
                                                                            {format!("{wins}/{total}")}
                                                                        </span>
                                                                        <span class="text-secondary text-xs w-10 text-right shrink-0">
                                                                            {format!("{win_pct}%")}
                                                                        </span>
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_any()
                                                },
                                                Err(_) => view! {
                                                    <div class="text-center py-4">
                                                        <p class="text-muted text-sm">"No win condition data yet."</p>
                                                        <p class="text-dimmed text-xs mt-1">
                                                            "Complete post-game reviews with tagged win conditions to see trends here."
                                                        </p>
                                                    </div>
                                                }.into_any(),
                                            })
                                        }}
                                    </Suspense>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}
                    </div>

                    // Macro strategy
                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4 flex flex-col gap-4">
                        <h3 class="text-primary font-semibold text-sm">"Macro Strategy"</h3>
                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label class="block text-muted text-xs font-medium mb-1">"Win Conditions (one per line)"</label>
                                <textarea rows="4" class=textarea_class()
                                    placeholder="Force early teamfights\nControl Dragon side"
                                    prop:value=move || win_conditions.get()
                                    on:input=move |ev| set_win_conditions.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-muted text-xs font-medium mb-1">"Objective Priority (one per line)"</label>
                                <textarea rows="4" class=textarea_class()
                                    placeholder="Dragon\nRift Herald\nBaron"
                                    prop:value=move || obj_priority.get()
                                    on:input=move |ev| set_obj_priority.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                        <div>
                            <label class="block text-muted text-xs font-medium mb-1">"Teamfight Strategy"</label>
                            <textarea rows="2" class=textarea_class()
                                placeholder="Front-to-back: protect ADC while top zones..."
                                prop:value=move || teamfight.get()
                                on:input=move |ev| set_teamfight.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class="block text-muted text-xs font-medium mb-1">"Early Game Plan"</label>
                            <textarea rows="2" class=textarea_class()
                                placeholder="Contest level 1 vision, jungle path..."
                                prop:value=move || early_game.get()
                                on:input=move |ev| set_early_game.set(event_target_value(&ev))
                            />
                        </div>
                    </div>

                    // Role-specific strategy
                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4 flex flex-col gap-3">
                        <h3 class="text-primary font-semibold text-sm">"Role-Specific Strategy"</h3>
                        <div class="grid grid-cols-1 gap-3">
                            {ROLES.iter().enumerate().map(|(i, &role)| {
                                let our_sig = our_champ_signals[i];
                                let enemy_sig = enemy_champ_signals[i];
                                view! {
                                    <div class="flex gap-3 items-start">
                                        <div class="w-16 flex-shrink-0 pt-2">
                                            <span class="text-accent text-xs font-semibold">{role}</span>
                                            {move || {
                                                let o = our_sig.get();
                                                let t = enemy_sig.get();
                                                if !o.is_empty() && !t.is_empty() {
                                                    let champ_list = champions.get()
                                                        .and_then(|r| r.ok())
                                                        .unwrap_or_default();
                                                    let o_name = champ_list.iter()
                                                        .find(|c| c.id == o)
                                                        .map(|c| c.name.clone())
                                                        .unwrap_or(o);
                                                    let t_name = champ_list.iter()
                                                        .find(|c| c.id == t)
                                                        .map(|c| c.name.clone())
                                                        .unwrap_or(t);
                                                    view! {
                                                        <div class="text-dimmed text-xs mt-0.5">{format!("{o_name} vs {t_name}")}</div>
                                                    }.into_any()
                                                } else {
                                                    view! { <div></div> }.into_any()
                                                }
                                            }}
                                        </div>
                                        <textarea rows="2" class=format!("{} flex-1", textarea_class())
                                            placeholder=format!("Strategy for {role}...")
                                            prop:value=move || role_strats.get().get(i).cloned().unwrap_or_default()
                                            on:input=move |ev| {
                                                let val = event_target_value(&ev);
                                                set_role_strats.update(|v| {
                                                    if i < v.len() { v[i] = val; }
                                                });
                                            }
                                        />
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    // Notes
                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4">
                        <label class="block text-muted text-xs font-medium mb-1">"Additional Notes"</label>
                        <textarea rows="3" class=textarea_class()
                            placeholder="Anything else the team should know..."
                            prop:value=move || notes.get()
                            on:input=move |ev| set_notes.set(event_target_value(&ev))
                        />
                    </div>

                    // Pre-Game Checklist section
                    <ChecklistSection editing_id=editing_id toast=toast />

                    // Start Post-Game Review button
                    {move || {
                        let eid = editing_id.get();
                        let did = draft_id.get();
                        eid.map(|plan_id| {
                            let plan_id_clone = plan_id.clone();
                            let draft_for_review = if did.is_empty() { None } else { Some(did.clone()) };
                            view! {
                                <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4 flex items-center justify-between">
                                    <div>
                                        <h3 class="text-primary font-semibold text-sm">"Game Complete?"</h3>
                                        <p class="text-muted text-xs mt-0.5">"Start a post-game review with this plan pre-linked"</p>
                                    </div>
                                    <button
                                        class="bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg px-5 py-2 text-sm transition-colors"
                                        on:click=move |_| {
                                            let pid = plan_id_clone.clone();
                                            let did = draft_for_review.clone();
                                            leptos::task::spawn_local(async move {
                                                match start_post_game_review(pid, did).await {
                                                    #[allow(unused_variables)]
                                                    Ok(review_id) => {
                                                        #[cfg(feature = "hydrate")]
                                                        if let Some(window) = web_sys::window() {
                                                            let _ = window.location().set_href(
                                                                &format!("/post-game?review_id={review_id}")
                                                            );
                                                        }
                                                    }
                                                    Err(e) => {
                                                        toast.show.run((ToastKind::Error, format!("{e}")));
                                                    }
                                                }
                                            });
                                        }
                                    >"Start Post-Game Review"</button>
                                </div>
                            }
                        })
                    }}

                    // Save buttons + auto-save status
                    <div class="flex gap-3 items-center">
                        <button
                            class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-6 py-2 text-sm transition-colors"
                            on:click=do_save
                        >
                            {move || if editing_id.get().is_some() { "Update Plan" } else { "Save Plan" }}
                        </button>
                        <button
                            class="bg-overlay hover:bg-overlay-strong text-secondary rounded-lg px-4 py-2 text-sm transition-colors"
                            on:click=move |_| clear_editor.run(())
                        >"Clear"</button>
                        {move || {
                            let status = auto_save_status.get();
                            match status {
                                "saved" => view! {
                                    <span class="text-green-400 text-xs font-medium">"\u{2713} Saved"</span>
                                }.into_any(),
                                "unsaved" => view! {
                                    <span class="text-accent text-xs font-medium">"\u{25cf} Unsaved changes"</span>
                                }.into_any(),
                                _ => view! { <span></span> }.into_any(),
                            }
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Checklist section component
// ---------------------------------------------------------------------------

#[component]
fn ChecklistSection(
    editing_id: ReadSignal<Option<String>>,
    toast: ToastContext,
) -> impl IntoView {
    let (checklist_open, set_checklist_open) = signal(false);
    let (checklist_items, set_checklist_items) = signal(Vec::<String>::new());
    let (checklist_checked, set_checklist_checked) = signal(Vec::<bool>::new());
    let (checklist_instance_id, set_checklist_instance_id) = signal(Option::<String>::None);
    let (new_item_text, set_new_item_text) = signal(String::new());

    let templates = Resource::new(|| (), |_| get_checklist_templates());

    // Load checklist when plan changes
    Effect::new(move || {
        let plan_id = editing_id.get();
        if let Some(pid) = plan_id {
            let pid = pid.clone();
            leptos::task::spawn_local(async move {
                match get_plan_checklist(pid).await {
                    Ok(Some(inst)) => {
                        set_checklist_instance_id.set(inst.id.clone());
                        set_checklist_items.set(inst.items.clone());
                        set_checklist_checked.set(inst.checked.clone());
                    }
                    Ok(None) => {
                        set_checklist_instance_id.set(None);
                        set_checklist_items.set(Vec::new());
                        set_checklist_checked.set(Vec::new());
                    }
                    Err(_) => {
                        set_checklist_instance_id.set(None);
                        set_checklist_items.set(Vec::new());
                        set_checklist_checked.set(Vec::new());
                    }
                }
            });
        } else {
            set_checklist_instance_id.set(None);
            set_checklist_items.set(Vec::new());
            set_checklist_checked.set(Vec::new());
        }
    });

    let do_add_item = move |_| {
        let text = new_item_text.get_untracked();
        if text.trim().is_empty() {
            return;
        }
        set_checklist_items.update(|items| items.push(text.trim().to_string()));
        set_checklist_checked.update(|c| c.push(false));
        set_new_item_text.set(String::new());
    };

    let do_use_template = Callback::new(move |tmpl: ChecklistTemplate| {
        set_checklist_items.set(tmpl.items.clone());
        set_checklist_checked.set(vec![false; tmpl.items.len()]);
        set_checklist_instance_id.set(None); // Will create new on save
        toast.show.run((ToastKind::Success, format!("Loaded template: {}", tmpl.name)));
    });

    let do_save_checklist = move |_| {
        let plan_id = editing_id.get_untracked();
        let items = checklist_items.get_untracked();
        let checked = checklist_checked.get_untracked();
        let instance_id = checklist_instance_id.get_untracked();

        if items.is_empty() {
            toast.show.run((ToastKind::Error, "No checklist items to save.".into()));
            return;
        }

        let Some(pid) = plan_id else {
            toast.show.run((ToastKind::Error, "Save the plan first before adding a checklist.".into()));
            return;
        };

        leptos::task::spawn_local(async move {
            if let Some(iid) = instance_id {
                // Update existing
                let checked_json = serde_json::to_string(&checked).unwrap_or_default();
                match update_checklist(iid, checked_json).await {
                    Ok(_) => toast.show.run((ToastKind::Success, "Checklist updated!".into())),
                    Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                }
            } else {
                // Create new
                let items_json = serde_json::to_string(&items).unwrap_or_default();
                match create_plan_checklist(pid, None, items_json).await {
                    Ok(id) => {
                        set_checklist_instance_id.set(Some(id));
                        toast.show.run((ToastKind::Success, "Checklist created!".into()));
                    }
                    Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                }
            }
        });
    };

    let do_save_as_template = move |_| {
        let items = checklist_items.get_untracked();
        if items.is_empty() {
            toast.show.run((ToastKind::Error, "No items to save as template.".into()));
            return;
        }
        let items_json = serde_json::to_string(&items).unwrap_or_default();
        leptos::task::spawn_local(async move {
            match save_checklist_template("Pre-Game Checklist".to_string(), items_json).await {
                Ok(_) => {
                    toast.show.run((ToastKind::Success, "Saved as template!".into()));
                    templates.refetch();
                }
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        });
    };

    view! {
        <div class="bg-elevated/50 border border-divider/50 rounded-xl">
            <button
                class="w-full text-left p-4 flex items-center justify-between"
                on:click=move |_| set_checklist_open.update(|v| *v = !*v)
            >
                <div class="flex items-center gap-2">
                    <h3 class="text-primary font-semibold text-sm">"Pre-Game Checklist"</h3>
                    {move || {
                        let items = checklist_items.get();
                        let checked = checklist_checked.get();
                        if items.is_empty() {
                            view! { <span class="text-dimmed text-xs">"No items"</span> }.into_any()
                        } else {
                            let done = checked.iter().filter(|c| **c).count();
                            let total = items.len();
                            view! {
                                <span class="text-muted text-xs">{format!("{done}/{total} complete")}</span>
                            }.into_any()
                        }
                    }}
                </div>
                <span class="text-dimmed text-sm">
                    {move || if checklist_open.get() { "\u{25BC}" } else { "\u{25B6}" }}
                </span>
            </button>

            {move || checklist_open.get().then(|| {
                let items = checklist_items.get();
                let checked = checklist_checked.get();
                let total = items.len();
                let done = checked.iter().filter(|c| **c).count();

                view! {
                    <div class="px-4 pb-4 flex flex-col gap-3">
                        // Progress bar
                        {(total > 0).then(|| {
                            let pct = if total > 0 { (done * 100) / total } else { 0 };
                            view! {
                                <div class="w-full bg-overlay rounded-full h-2">
                                    <div
                                        class="bg-accent rounded-full h-2 transition-all"
                                        style=format!("width: {}%", pct)
                                    />
                                </div>
                            }
                        })}

                        // Checklist items
                        <div class="flex flex-col gap-1">
                            {items.iter().enumerate().map(|(i, item)| {
                                let item_text = item.clone();
                                let is_checked = checked.get(i).copied().unwrap_or(false);
                                view! {
                                    <label class="flex items-center gap-2 py-1 px-2 rounded hover:bg-overlay/30 cursor-pointer transition-colors">
                                        <input
                                            type="checkbox"
                                            class="accent-accent"
                                            prop:checked=is_checked
                                            on:change=move |ev| {
                                                let val = event_target_checked(&ev);
                                                set_checklist_checked.update(|c| {
                                                    if i < c.len() { c[i] = val; }
                                                });
                                                // Auto-save check state
                                                let instance_id = checklist_instance_id.get_untracked();
                                                if let Some(iid) = instance_id {
                                                    let checked_now = checklist_checked.get_untracked();
                                                    let checked_json = serde_json::to_string(&checked_now).unwrap_or_default();
                                                    leptos::task::spawn_local(async move {
                                                        let _ = update_checklist(iid, checked_json).await;
                                                    });
                                                }
                                            }
                                        />
                                        <span class=move || {
                                            let c = checklist_checked.get();
                                            if c.get(i).copied().unwrap_or(false) {
                                                "text-muted text-sm line-through"
                                            } else {
                                                "text-primary text-sm"
                                            }
                                        }>{item_text}</span>
                                        <button
                                            class="ml-auto text-red-400/50 hover:text-red-400 text-xs transition-colors"
                                            on:click=move |_| {
                                                set_checklist_items.update(|items| { if i < items.len() { items.remove(i); } });
                                                set_checklist_checked.update(|c| { if i < c.len() { c.remove(i); } });
                                            }
                                        >"x"</button>
                                    </label>
                                }
                            }).collect_view()}
                        </div>

                        // Add new item
                        <div class="flex gap-2">
                            <input
                                type="text"
                                class=input_class()
                                placeholder="Add checklist item..."
                                prop:value=move || new_item_text.get()
                                on:input=move |ev| set_new_item_text.set(event_target_value(&ev))
                                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                    if ev.key() == "Enter" {
                                        ev.prevent_default();
                                        let text = new_item_text.get_untracked();
                                        if !text.trim().is_empty() {
                                            set_checklist_items.update(|items| items.push(text.trim().to_string()));
                                            set_checklist_checked.update(|c| c.push(false));
                                            set_new_item_text.set(String::new());
                                        }
                                    }
                                }
                            />
                            <button
                                class="bg-overlay hover:bg-overlay-strong text-secondary text-sm font-medium px-3 py-2 rounded-lg transition-colors flex-shrink-0"
                                on:click=do_add_item
                            >"Add"</button>
                        </div>

                        // Template selector
                        <Suspense fallback=|| ()>
                            {move || templates.get().map(|result| match result {
                                Ok(tmpls) if !tmpls.is_empty() => {
                                    view! {
                                        <div class="flex flex-wrap gap-2 items-center">
                                            <span class="text-muted text-xs">"Templates:"</span>
                                            {tmpls.into_iter().map(|t| {
                                                let t_for_use = t.clone();
                                                let t_id = t.id.clone().unwrap_or_default();
                                                let t_name = t.name.clone();
                                                view! {
                                                    <button
                                                        class="bg-overlay hover:bg-overlay-strong text-secondary text-xs px-2 py-1 rounded transition-colors"
                                                        on:click=move |_| do_use_template.run(t_for_use.clone())
                                                    >{t_name}</button>
                                                    <button
                                                        class="text-red-400/50 hover:text-red-400 text-xs transition-colors"
                                                        on:click=move |_| {
                                                            let id = t_id.clone();
                                                            leptos::task::spawn_local(async move {
                                                                let _ = delete_checklist_template_fn(id).await;
                                                                templates.refetch();
                                                            });
                                                        }
                                                    >"x"</button>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                },
                                _ => view! { <span></span> }.into_any(),
                            })}
                        </Suspense>

                        // Action buttons
                        <div class="flex gap-2">
                            <button
                                class="bg-accent hover:bg-accent-hover text-accent-contrast text-xs font-medium px-3 py-1.5 rounded-lg transition-colors"
                                on:click=do_save_checklist
                            >"Save Checklist"</button>
                            <button
                                class="bg-overlay hover:bg-overlay-strong text-secondary text-xs font-medium px-3 py-1.5 rounded-lg transition-colors"
                                on:click=do_save_as_template
                            >"Save as Template"</button>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
