use crate::app::InitialTheme;
use crate::components::region::*;
use crate::components::ui::{ErrorBanner, NoTeamState, SkeletonCard, ToastContext, ToastKind};
use crate::models::team::Team;
use crate::models::user::{JoinRequest, TeamMember};
use crate::models::utils::format_timestamp;
use leptos::prelude::*;
use leptos_router::components::A;

/// Returns (team, members, current_user_id) so the client can check leadership.
#[server]
pub async fn get_team_dashboard() -> Result<Option<(Team, Vec<TeamMember>, String)>, ServerFnError>
{
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let user_id = user.id.clone();
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    match db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some((team, members)) => Ok(Some((team, members, user_id))),
        None => Ok(None),
    }
}

#[server]
pub async fn update_team_info(name: String, region: String) -> Result<(), ServerFnError> {
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

    let (team, _) = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    if team.created_by != user.id {
        return Err(ServerFnError::new(
            "Only the team leader can edit team details",
        ));
    }

    let team_id = team.id.ok_or_else(|| ServerFnError::new("No team id"))?;
    db::update_team(&db, &team_id, name, region)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn set_member_role(member_user_id: String, role: String) -> Result<(), ServerFnError> {
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

    let (team, _) = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    if team.created_by != user.id {
        return Err(ServerFnError::new("Only the team leader can assign roles"));
    }

    let team_id = team.id.ok_or_else(|| ServerFnError::new("No team id"))?;
    db::update_member_role(&db, &team_id, &member_user_id, role)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn kick_member(member_user_id: String) -> Result<(), ServerFnError> {
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

    let (team, _) = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    if team.created_by != user.id {
        return Err(ServerFnError::new(
            "Only the team leader can remove members",
        ));
    }

    if member_user_id == user.id {
        return Err(ServerFnError::new("Team leader cannot remove themselves"));
    }

    let team_id = team.id.ok_or_else(|| ServerFnError::new("No team id"))?;
    db::remove_team_member(&db, &team_id, &member_user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn leave_team() -> Result<(), ServerFnError> {
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

    let (team, _) = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    if team.created_by == user.id {
        return Err(ServerFnError::new(
            "Team leader cannot leave the team. Transfer leadership or delete the team first.",
        ));
    }

    let team_id = team.id.ok_or_else(|| ServerFnError::new("No team id"))?;
    db::remove_team_member(&db, &team_id, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_pending_requests() -> Result<Vec<JoinRequest>, ServerFnError> {
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

    let (team, _) = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    if team.created_by != user.id {
        return Ok(Vec::new()); // non-leaders see empty list
    }

    let team_id = team.id.ok_or_else(|| ServerFnError::new("No team id"))?;
    db::list_pending_join_requests(&db, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn handle_join_request(request_id: String, accept: bool) -> Result<(), ServerFnError> {
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

    let (team, _) = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    if team.created_by != user.id {
        return Err(ServerFnError::new(
            "Only the team leader can respond to requests",
        ));
    }

    let team_id = team.id.ok_or_else(|| ServerFnError::new("No team id"))?;
    db::respond_to_join_request(&db, &request_id, accept, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn assign_member_to_slot(
    member_user_id: String,
    role: String,
) -> Result<(), ServerFnError> {
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

    let (team, _) = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    if team.created_by != user.id {
        return Err(ServerFnError::new("Only the team leader can assign slots"));
    }

    let team_id = team.id.ok_or_else(|| ServerFnError::new("No team id"))?;
    db::assign_to_slot(&db, &team_id, &member_user_id, &role)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn unassign_member_from_slot(member_user_id: String) -> Result<(), ServerFnError> {
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

    let (team, _) = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    if team.created_by != user.id {
        return Err(ServerFnError::new(
            "Only the team leader can unassign slots",
        ));
    }

    let team_id = team.id.ok_or_else(|| ServerFnError::new("No team id"))?;
    db::remove_from_slot(&db, &team_id, &member_user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

const MEMBER_ROLES: &[&str] = &[
    "top",
    "jungle",
    "mid",
    "bot",
    "support",
    "coach",
    "unassigned",
];
const STARTER_ROLES: &[&str] = &["top", "jungle", "mid", "bot", "support"];

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RecentMatch {
    pub riot_match_id: String,
    pub champion: String,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub win: bool,
    pub game_end: Option<String>,
    pub username: String,
}

#[server]
pub async fn get_recent_team_matches() -> Result<Vec<RecentMatch>, ServerFnError> {
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

    let rows = db::get_team_match_stats(&db, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Group by riot_match_id, take most recent 3 unique matches
    let mut seen = std::collections::HashSet::new();
    let mut recent: Vec<RecentMatch> = Vec::new();
    for r in rows {
        if seen.contains(&r.riot_match_id) {
            continue;
        }
        seen.insert(r.riot_match_id.clone());
        recent.push(RecentMatch {
            riot_match_id: r.riot_match_id,
            champion: r.champion,
            kills: r.kills,
            deaths: r.deaths,
            assists: r.assists,
            win: r.win,
            game_end: r.game_end,
            username: r.username,
        });
        if recent.len() >= 3 {
            break;
        }
    }
    Ok(recent)
}

// ---------------------------------------------------------------------------
// Team Notebook server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn get_all_team_notes(
) -> Result<Vec<crate::models::team_note::TeamNote>, ServerFnError> {
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

    db::list_team_notes(&db, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn add_team_note(content: String) -> Result<String, ServerFnError> {
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

    db::create_team_note(&db, &team_id, &user.id, &user.username, content)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn toggle_note_pin(note_id: String, pinned: bool) -> Result<(), ServerFnError> {
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

    db::toggle_pin_team_note(&db, &note_id, pinned)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn edit_team_note(note_id: String, content: String) -> Result<(), ServerFnError> {
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

    db::update_team_note(&db, &note_id, content)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn remove_team_note(note_id: String) -> Result<(), ServerFnError> {
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

    db::delete_team_note(&db, &note_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_open_action_items_summary(
) -> Result<(usize, Vec<crate::models::action_item::ActionItem>), ServerFnError> {
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
        None => return Ok((0, Vec::new())),
    };

    let items = db::list_open_action_items(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let total = items.len();
    let top3: Vec<_> = items.into_iter().take(3).collect();
    Ok((total, top3))
}

#[server]
pub async fn get_post_game_panel() -> Result<Vec<crate::models::game_plan::PostGamePreview>, ServerFnError> {
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

    let summary = db::get_dashboard_summary(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(summary.recent_post_games)
}

#[server]
pub async fn get_pool_gap_panel() -> Result<Vec<crate::models::game_plan::PoolGapWarning>, ServerFnError> {
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

    let summary = db::get_dashboard_summary(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(summary.pool_gap_warnings)
}

fn role_icon_url(role: &str) -> &'static str {
    match role {
        "top" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-top.svg",
        "jungle" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-jungle.svg",
        "mid" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-middle.svg",
        "bot" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-bottom.svg",
        "support" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-utility.svg",
        _ => "",
    }
}

// ---------------------------------------------------------------------------
// Phase 18-08: Mode toggle persistence
// ---------------------------------------------------------------------------

/// Persist the user's team dashboard mode preference. Validates against allowlist.
/// Mitigates T-18-08-01 (tampering via arbitrary mode string injection).
#[server]
pub async fn set_team_dashboard_mode_pref(mode: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    // App-layer validation — DB has no ASSERT per Research Pitfall 4
    const VALID: &[&str] = &["auto", "dashboard", "brief"];
    if !VALID.contains(&mode.as_str()) {
        return Err(ServerFnError::new("Invalid team dashboard mode"));
    }

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;
    db::set_user_team_dashboard_mode(&db, &user.id, &mode)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

/// Resolve the effective team dashboard mode from a stored preference and region context.
/// Returns region-coupled defaults when stored == "auto" (D-04).
/// An explicit user pick (stored != "auto") always wins over the default (D-05).
fn resolve_team_dashboard_mode(stored: &str, region: &str) -> String {
    if stored != "auto" {
        return stored.to_string();
    }
    match region {
        "pandemonium" => "brief".to_string(),
        _ => "dashboard".to_string(),
    }
}

/// Top-level team dashboard page. Reads region once, dispatches to region-specific view.
#[component]
pub fn TeamDashboard() -> impl IntoView {
    // Read region once at page entry — passed as String to all subcomponents.
    let theme = use_context::<InitialTheme>().unwrap_or_default();
    let region = theme.0.clone();
    let is_pandemonium = region == "pandemonium";

    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());

    // Mode preference — signal-driven, persisted via set_team_dashboard_mode_pref server fn (18-08)
    let (mode_current, set_mode_current) = signal(
        resolve_team_dashboard_mode("auto", &region)
    );
    let region_for_mode = region.clone();
    let is_solo_mode: RwSignal<bool> = RwSignal::new(false);
    Effect::new(move || {
        match auth_user.get() {
            Some(Ok(None)) => {
                #[cfg(feature = "hydrate")]
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/auth/login");
                }
            }
            Some(Ok(Some(u))) => {
                is_solo_mode.set(u.mode == "solo");
                let resolved = resolve_team_dashboard_mode(&u.team_dashboard_mode, &region_for_mode);
                set_mode_current.set(resolved);
            }
            _ => {}
        }
    });

    let set_mode_action = Action::new(move |new_mode: &String| {
        let m = new_mode.clone();
        async move { set_team_dashboard_mode_pref(m).await }
    });
    Effect::new(move |_| {
        if let Some(Ok(())) = set_mode_action.value().get() {
            auth_user.refetch();
        }
    });
    let on_mode_select = Callback::new(move |new_mode: String| {
        set_mode_current.set(new_mode.clone());
        set_mode_action.dispatch(new_mode);
    });

    let dashboard = Resource::new(|| (), |_| get_team_dashboard());
    let requests = Resource::new(|| (), |_| get_pending_requests());
    let recent_matches = Resource::new(|| (), |_| get_recent_team_matches());
    let action_items_res = Resource::new(|| (), |_| get_open_action_items_summary());
    let post_game_panel = Resource::new(|| (), |_| get_post_game_panel());
    let pool_gap_panel = Resource::new(|| (), |_| get_pool_gap_panel());

    let region_for_toggle = region.clone();

    view! {
        <div class="canvas-grain bg-base min-h-screen px-8 py-6">
            <div class="max-w-5xl mx-auto">
            // Mode toggle — Dashboard / Game Day Brief (18-08)
            <div class="mb-4 flex items-center">
                <ModeToggle
                    region=region_for_toggle
                    current=mode_current
                    options=vec![
                        ("dashboard".to_string(), "Dashboard".to_string(), "DASHBOARD".to_string()),
                        ("brief".to_string(), "Game Day Brief".to_string(), "GAME_DAY".to_string()),
                    ]
                    on_select=on_mode_select
                />
            </div>
            <Suspense fallback=|| view! { <SkeletonCard height="h-32" /> }>
                {move || {
                    if is_solo_mode.get() {
                        return Some(view! {
                            <div class="max-w-2xl py-8 text-center">
                                <div class="bg-elevated border border-outline rounded-xl p-6">
                                    <h2 class="font-display italic text-2xl text-primary mb-2">"Team feature"</h2>
                                    <p class="text-secondary text-sm mb-4">"Switch to team mode to use this feature."</p>
                                    <button
                                        class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                        on:click=move |_| {
                                            leptos::task::spawn_local(async move {
                                                let _ = crate::components::nav::set_user_mode("team".to_string()).await;
                                                #[cfg(feature = "hydrate")]
                                                if let Some(window) = web_sys::window() {
                                                    let _ = window.location().reload();
                                                }
                                            });
                                        }
                                    >
                                        "Switch to Team Mode"
                                    </button>
                                </div>
                            </div>
                        }.into_any());
                    }
                    dashboard.get().map(|result| match result {
                    Ok(Some((team, members, current_user_id))) => {
                        // Brief mode — signal-driven (18-08: DB persistence + resolve_team_dashboard_mode)
                        if mode_current.get() == "brief" {
                            let region_brief = region.clone();
                            return view! {
                                <TeamGameDayBriefView region=region_brief team=team members=members />
                            }.into_any();
                        }
                        if is_pandemonium {
                            view! { <PandemoniumTeamDashboard team=team members=members /> }.into_any()
                        } else {
                            // Demacia: original team management UI with gilt header
                            let is_leader = team.created_by == current_user_id;
                            let created_by = team.created_by.clone();
                            let (edit_name, set_edit_name) = signal(team.name.clone());
                        let (edit_region, set_edit_region) = signal(team.region.clone());
                        let (show_edit_modal, set_show_edit_modal) = signal(false);
                        let (leave_confirm, set_leave_confirm) = signal(false);

                        // Check if current user has riot account linked
                        let current_user_riot_linked = members.iter()
                            .find(|m| m.user_id == current_user_id)
                            .map(|m| m.riot_summoner_name.is_some())
                            .unwrap_or(false);

                        // Partition members
                        let starters: Vec<TeamMember> = members.iter()
                            .filter(|m| m.roster_type == "starter")
                            .cloned()
                            .collect();
                        let coaches: Vec<TeamMember> = members.iter()
                            .filter(|m| m.role == "coach" && m.roster_type != "starter")
                            .cloned()
                            .collect();
                        let mut subs: Vec<TeamMember> = members.iter()
                            .filter(|m| m.roster_type != "starter" && m.role != "coach")
                            .cloned()
                            .collect();

                        // BUG-03: Ensure the team leader is always visible in the bench section.
                        // The leader may be absent if their team_member record has an unexpected state.
                        // If not found in any partition, insert them at the top of the bench.
                        let leader_in_any = starters.iter().any(|m| m.user_id == created_by)
                            || coaches.iter().any(|m| m.user_id == created_by)
                            || subs.iter().any(|m| m.user_id == created_by);
                        if !leader_in_any {
                            if let Some(leader_member) = members.iter().find(|m| m.user_id == created_by) {
                                subs.insert(0, leader_member.clone());
                            }
                        }

                        let created_by_for_starters = created_by.clone();
                        let created_by_for_coaches = created_by.clone();
                        let created_by_for_subs = created_by.clone();

                        view! {
                            <div class="flex flex-col gap-6">
                                // Team info card — gilt Card (region primitive)
                                <Card region="demacia".to_string() variant="gilt".to_string()>
                                <div>
                                    <div class="flex items-start justify-between gap-4">
                                        <div>
                                            <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted mb-1">"STRATEGY ROOM"</div>
                                            <div class="flex items-center gap-2">
                                                <h2 class="font-display italic text-[28px] text-accent">{team.name.clone()}</h2>
                                                {if is_leader {
                                                    view! {
                                                        <button
                                                            class="text-muted hover:text-accent transition-colors cursor-pointer p-1 rounded-md hover:bg-overlay focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                            title="Edit team details"
                                                            aria-label="Edit team details"
                                                            on:click=move |_| set_show_edit_modal.set(true)
                                                        >
                                                            // Pencil SVG icon
                                                            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                                                <path stroke-linecap="round" stroke-linejoin="round" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                                                            </svg>
                                                        </button>
                                                    }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }}
                                            </div>
                                            <p class="text-muted text-sm mt-1">"Region: " <span class="font-mono text-secondary">{team.region.clone()}</span></p>
                                            {if is_leader {
                                                view! { <span class="inline-block mt-1 font-imperial uppercase tracking-[0.18em] text-[10px] text-accent bg-accent/10 rounded-md px-2 py-0.5">"Team Leader"</span> }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }}
                                        </div>
                                    </div>

                                    // Riot account status notice
                                    {if !current_user_riot_linked {
                                        view! {
                                            <div class="mt-4 pt-4 border-t border-divider">
                                                <p class="text-muted text-sm">
                                                    "Riot account not linked \u{2014} "
                                                    <A href="/profile" attr:class="text-accent hover:underline focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md">"link it in your profile"</A>
                                                </p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                    <div class="mt-4 pt-3 border-t border-divider/50 flex items-center gap-3">
                                        <HeraldicDivider width=200 />
                                        <Btn region="demacia".to_string() variant="primary".to_string()>
                                            <A href="/draft" attr:class="text-inherit no-underline">"Open Draft"</A>
                                        </Btn>
                                    </div>
                                </div>
                                </Card>

                                // Edit team modal (leader only)
                                {move || if show_edit_modal.get() {
                                    view! {
                                        // Backdrop
                                        <div
                                            class="fixed inset-0 bg-overlay-strong z-50 flex items-center justify-center"
                                            on:click=move |_| set_show_edit_modal.set(false)
                                        >
                                            // Modal content — stop click propagation
                                            <div
                                                class="bg-elevated border border-outline rounded-xl shadow-2xl p-6 w-full max-w-md mx-4"
                                                on:click=move |ev| ev.stop_propagation()
                                            >
                                                <h3 class="font-display italic text-primary text-xl mb-4">"Edit Team"</h3>
                                                <div class="flex flex-col gap-4">
                                                    <div>
                                                        <label class="block font-imperial uppercase tracking-[0.18em] text-[10px] text-muted mb-1">"Team Name"</label>
                                                        <input
                                                            type="text"
                                                            prop:value=move || edit_name.get()
                                                            on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                                            class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                        />
                                                    </div>
                                                    <div>
                                                        <label class="block font-imperial uppercase tracking-[0.18em] text-[10px] text-muted mb-1">"Region"</label>
                                                        <select
                                                            prop:value=move || edit_region.get()
                                                            on:change=move |ev| set_edit_region.set(event_target_value(&ev))
                                                            class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                        >
                                                            {["EUW","EUNE","NA","KR","BR"].iter().map(|&r| view! {
                                                                <option value=r>{r}</option>
                                                            }).collect_view()}
                                                        </select>
                                                    </div>
                                                    <div class="flex justify-end gap-3 pt-2">
                                                        <button
                                                            class="text-secondary hover:text-primary text-sm px-4 py-2 rounded-lg transition-colors cursor-pointer hover:bg-overlay focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                            on:click=move |_| {
                                                                set_show_edit_modal.set(false);
                                                            }
                                                        >
                                                            "Cancel"
                                                        </button>
                                                        <button
                                                            class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded-lg px-4 py-2 text-sm transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                            on:click=move |_| {
                                                                let name = edit_name.get_untracked();
                                                                let region = edit_region.get_untracked();
                                                                leptos::task::spawn_local(async move {
                                                                    match update_team_info(name, region).await {
                                                                        Ok(_) => {
                                                                            toast.show.run((ToastKind::Success, "Team info updated".into()));
                                                                            dashboard.refetch();
                                                                            // Close modal after a brief delay to show success
                                                                            #[cfg(feature = "hydrate")]
                                                                            {
                                                                                use wasm_bindgen::prelude::*;
                                                                                let cb = Closure::once(move || {
                                                                                    set_show_edit_modal.set(false);
                                                                                });
                                                                                if let Some(win) = web_sys::window() {
                                                                                    let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                                                                                        cb.as_ref().unchecked_ref(), 800,
                                                                                    );
                                                                                }
                                                                                cb.forget();
                                                                            }
                                                                        }
                                                                        Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            "Save"
                                                        </button>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}

                                // Join requests (leader only)
                                {if is_leader {
                                    view! {
                                        <div>
                                            <Suspense fallback=|| view! { <span></span> }>
                                                {move || requests.get().map(|res| {
                                                    let reqs = res.unwrap_or_default();
                                                    if reqs.is_empty() {
                                                        view! { <span></span> }.into_any()
                                                    } else {
                                                        view! {
                                                            <div class="bg-elevated border border-accent/30 rounded-xl p-5">
                                                                <h3 class="font-display italic text-accent text-xl mb-3 flex items-center gap-2">
                                                                    "Join Requests"
                                                                    <span class="bg-accent text-accent-contrast text-xs font-bold rounded-full w-5 h-5 flex items-center justify-center tabular-nums">
                                                                        {reqs.len()}
                                                                    </span>
                                                                </h3>
                                                                <div class="flex flex-col gap-2">
                                                                    {reqs.into_iter().map(|req| {
                                                                        let req_id_accept = req.id.clone();
                                                                        let req_id_decline = req.id.clone();
                                                                        let (req_error, set_req_error) = signal(Option::<String>::None);
                                                                        view! {
                                                                            <div class="flex flex-col gap-1">
                                                                                <div class="flex items-center justify-between bg-surface border border-outline/50 rounded-lg px-4 py-3">
                                                                                <div>
                                                                                    <span class="text-primary font-medium">{req.username}</span>
                                                                                    {req.riot_summoner_name.map(|n| view! {
                                                                                        <span class="text-muted text-sm ml-2 font-mono">{n}</span>
                                                                                    })}
                                                                                </div>
                                                                                <div class="flex gap-2">
                                                                                    <button
                                                                                        class="bg-success/15 text-success border border-success/30 hover:bg-success/25 text-sm font-medium rounded-lg px-3 py-1.5 transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-success/50 focus-visible:outline-none"
                                                                                        on:click=move |_| {
                                                                                            let id = req_id_accept.clone();
                                                                                            leptos::task::spawn_local(async move {
                                                                                                match handle_join_request(id, true).await {
                                                                                                    Ok(_) => {
                                                                                                        set_req_error.set(None);
                                                                                                        dashboard.refetch();
                                                                                                        requests.refetch();
                                                                                                    }
                                                                                                    Err(e) => set_req_error.set(Some(format!("Error: {e}"))),
                                                                                                }
                                                                                            });
                                                                                        }
                                                                                    >"Accept"</button>
                                                                                    <button
                                                                                        class="bg-danger/10 text-danger border border-danger/30 hover:bg-danger/20 text-sm font-medium rounded-lg px-3 py-1.5 transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-danger/50 focus-visible:outline-none"
                                                                                        on:click=move |_| {
                                                                                            let id = req_id_decline.clone();
                                                                                            leptos::task::spawn_local(async move {
                                                                                                let _ = handle_join_request(id, false).await;
                                                                                                requests.refetch();
                                                                                            });
                                                                                        }
                                                                                    >"Decline"</button>
                                                                                </div>
                                                                            </div>
                                                                            {move || req_error.get().map(|e| view! {
                                                                                <p class="text-danger text-xs px-1" role="alert">{e}</p>
                                                                            })}
                                                                            </div>
                                                                        }
                                                                    }).collect_view()}
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    }
                                                })}
                                            </Suspense>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}

                                // Starting roster — 5 role slots
                                <div>
                                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Order of battle"</div>
                                    <h3 class="font-display italic text-2xl text-primary mb-3">"Starting Roster"</h3>
                                    <div class="grid grid-cols-5 gap-3">
                                        {STARTER_ROLES.iter().map(|&role| {
                                            let assigned = starters.iter().find(|m| m.role == role).cloned();
                                            let role_label = role.to_string();
                                            let role_label2 = role_label.clone();
                                            let (drag_over, set_drag_over) = signal(false);
                                            let leader_id = created_by_for_starters.clone();

                                            view! {
                                                <div
                                                    class=move || format!(
                                                        "relative overflow-hidden bg-elevated border rounded-xl p-3 flex flex-col items-center gap-2 min-h-[120px] transition-colors {}",
                                                        if drag_over.get() { "border-accent bg-overlay" } else { "border-outline/50" }
                                                    )
                                                    on:dragover=move |ev| {
                                                        ev.prevent_default();
                                                        set_drag_over.set(true);
                                                    }
                                                    on:dragleave=move |_| set_drag_over.set(false)
                                                    on:drop=move |ev| {
                                                        ev.prevent_default();
                                                        set_drag_over.set(false);
                                                        let Some(dt) = ev.data_transfer() else { return };
                                                        let uid = dt.get_data("text/plain").unwrap_or_default();
                                                        if !uid.is_empty() {
                                                            let r = role_label.clone();
                                                            leptos::task::spawn_local(async move {
                                                                let _ = assign_member_to_slot(uid, r).await;
                                                                dashboard.refetch();
                                                            });
                                                        }
                                                    }
                                                >
                                                    // Watermark role icon
                                                    {if !role_icon_url(role).is_empty() {
                                                        view! {
                                                            <img
                                                                src=role_icon_url(role)
                                                                alt=""
                                                                aria-hidden="true"
                                                                class="absolute bottom-0 right-0 w-14 h-14 opacity-10 invert pointer-events-none select-none translate-x-2 translate-y-2"
                                                            />
                                                        }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }}

                                                    // Role icon
                                                    {if !role_icon_url(role).is_empty() {
                                                        view! {
                                                            <img src=role_icon_url(role) alt=role_label2.clone() title=role_label2 class="w-6 h-6 invert opacity-70" />
                                                        }.into_any()
                                                    } else {
                                                        view! { <span class="text-muted text-xs capitalize">{role_label2}</span> }.into_any()
                                                    }}

                                                    // Assigned player or empty slot
                                                    {if let Some(m) = assigned {
                                                        let uid_for_unassign = m.user_id.clone();
                                                        let is_member_leader = m.user_id == leader_id;
                                                        view! {
                                                            <div class="flex-1 flex flex-col items-center justify-center gap-1 w-full">
                                                                <div class="flex items-center gap-1">
                                                                    <span class="text-primary text-sm font-medium text-center truncate">{m.username.clone()}</span>
                                                                    {is_member_leader.then(|| view! {
                                                                        <span class="text-accent text-xs" title="Team Leader">"★"</span>
                                                                    })}
                                                                </div>
                                                                {if is_leader {
                                                                    view! {
                                                                        <button
                                                                            class="text-muted hover:text-danger text-xs transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-danger/50 focus-visible:outline-none rounded"
                                                                            title="Remove from slot"
                                                                            aria-label="Remove from slot"
                                                                            on:click=move |_| {
                                                                                let uid = uid_for_unassign.clone();
                                                                                leptos::task::spawn_local(async move {
                                                                                    let _ = unassign_member_from_slot(uid).await;
                                                                                    dashboard.refetch();
                                                                                });
                                                                            }
                                                                        >"✕"</button>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <span></span> }.into_any()
                                                                }}
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <div class="flex-1 flex items-center justify-center">
                                                                <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-dimmed">"Empty"</span>
                                                            </div>
                                                        }.into_any()
                                                    }}
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                    {if is_leader {
                                        view! { <p class="text-dimmed text-xs mt-2">"Drag players from the bench below to assign them to role slots."</p> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>

                                // Coaches section
                                <div>
                                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Council"</div>
                                    <h3 class="font-display italic text-2xl text-primary mb-3">"Coaches"</h3>
                                    {if coaches.is_empty() {
                                        view! { <p class="text-dimmed text-sm">"No coaches assigned. Set a member's role to \"coach\" from the bench."</p> }.into_any()
                                    } else {
                                        view! {
                                            <div class="grid grid-cols-2 gap-3">
                                                {coaches.into_iter().map(|m| {
                                                    let is_member_leader = m.user_id == created_by_for_coaches;
                                                    view! {
                                                        <div class="relative overflow-hidden bg-elevated border border-outline/50 rounded-xl p-3 flex items-center gap-3">
                                                            // Watermark clipboard icon for coach
                                                            <div class="absolute inset-0 overflow-hidden pointer-events-none">
                                                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5"
                                                                     class="absolute bottom-0 right-0 w-14 h-14 opacity-10 text-muted pointer-events-none select-none translate-x-2 translate-y-2"
                                                                     aria-hidden="true">
                                                                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h3.75M9 15h3.75M9 18h3.75m3 .75H18a2.25 2.25 0 002.25-2.25V6.108c0-1.135-.845-2.098-1.976-2.192a48.424 48.424 0 00-1.123-.08m-5.801 0c-.065.21-.1.433-.1.664 0 .414.336.75.75.75h4.5a.75.75 0 00.75-.75 2.25 2.25 0 00-.1-.664m-5.8 0A2.251 2.251 0 0113.5 2.25H15c1.012 0 1.867.668 2.15 1.586m-5.8 0c-.376.023-.75.05-1.124.08C9.095 4.01 8.25 4.973 8.25 6.108V8.25m0 0H4.875c-.621 0-1.125.504-1.125 1.125v11.25c0 .621.504 1.125 1.125 1.125h9.75c.621 0 1.125-.504 1.125-1.125V9.375c0-.621-.504-1.125-1.125-1.125H8.25z" />
                                                                </svg>
                                                            </div>
                                                            <span class="bg-info/20 text-info rounded-full w-8 h-8 flex items-center justify-center text-xs font-bold uppercase">
                                                                {m.username.chars().next().unwrap_or('?').to_string()}
                                                            </span>
                                                            <div>
                                                                <div class="flex items-center gap-1">
                                                                    <span class="text-primary text-sm font-medium">{m.username}</span>
                                                                    {is_member_leader.then(|| view! {
                                                                        <span class="text-accent text-xs" title="Team Leader" aria-hidden="true">"★"</span>
                                                                    })}
                                                                </div>
                                                                <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-info">"Coach"</span>
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_any()
                                    }}
                                </div>

                                // Substitute bench
                                <div>
                                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Reserves"</div>
                                    <h3 class="font-display italic text-2xl text-primary mb-3">"Bench / Substitutes"</h3>
                                    {if subs.is_empty() {
                                        view! { <p class="text-dimmed text-sm">"No players on the bench."</p> }.into_any()
                                    } else {
                                        view! {
                                            <div class="flex flex-col gap-2">
                                                {subs.into_iter().map(|m| {
                                                    let uid_drag = m.user_id.clone();
                                                    let uid_kick = m.user_id.clone();
                                                    let is_self = m.user_id == current_user_id;
                                                    let is_member_leader = m.user_id == created_by_for_subs;
                                                    let current_role = m.role.clone();
                                                    let role_for_wm = m.role.clone();
                                                    let display_name = m.username.clone();
                                                    let (role_msg, set_role_msg) = signal(Option::<String>::None);

                                                    view! {
                                                        <div
                                                            class="relative bg-elevated border border-outline/50 rounded-xl px-4 py-3 flex items-center justify-between gap-3 cursor-grab active:cursor-grabbing"
                                                            draggable="true"
                                                            on:dragstart=move |ev| {
                                                                if let Some(dt) = ev.data_transfer() {
                                                                    let _ = dt.set_data("text/plain", &uid_drag);
                                                                }
                                                            }
                                                        >
                                                            // Watermark role icon (clipped to card)
                                                            <div class="absolute inset-0 overflow-hidden pointer-events-none">
                                                                {if !role_icon_url(&role_for_wm).is_empty() {
                                                                    view! {
                                                                        <img
                                                                            src=role_icon_url(&role_for_wm)
                                                                            alt=""
                                                                            aria-hidden="true"
                                                                            class="absolute bottom-0 right-0 w-14 h-14 opacity-10 invert pointer-events-none select-none translate-x-2 translate-y-2"
                                                                        />
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <span></span> }.into_any()
                                                                }}
                                                            </div>
                                                            <div class="flex items-center gap-2 min-w-0">
                                                                <span class="text-muted text-xs select-none" title="Drag to assign to a role slot">"⠿"</span>
                                                                <span class="text-primary font-medium truncate">{display_name}</span>
                                                                {is_member_leader.then(|| view! {
                                                                    <span class="text-accent text-xs" title="Team Leader">"★"</span>
                                                                })}
                                                                {m.riot_summoner_name.map(|n| view! {
                                                                    <span class="text-dimmed text-sm truncate">{n}</span>
                                                                })}
                                                                {move || role_msg.get().map(|msg| view! {
                                                                    <span class="text-xs text-success">{msg}</span>
                                                                })}
                                                            </div>
                                                            <div class="flex items-center gap-2 flex-shrink-0">
                                                                {if is_leader {
                                                                    let mid = m.user_id.clone();
                                                                    view! {
                                                                        <select
                                                                            class="bg-surface/50 border border-outline/50 rounded-lg px-2 py-1 text-secondary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                                            on:change=move |ev| {
                                                                                let role = event_target_value(&ev);
                                                                                let uid = mid.clone();
                                                                                leptos::task::spawn_local(async move {
                                                                                    match set_member_role(uid, role).await {
                                                                                        Ok(_) => {
                                                                                            set_role_msg.set(Some("✓".into()));
                                                                                            dashboard.refetch();
                                                                                        }
                                                                                        Err(e) => set_role_msg.set(Some(e.to_string())),
                                                                                    }
                                                                                });
                                                                            }
                                                                        >
                                                                            {MEMBER_ROLES.iter().map(|&r| {
                                                                                let selected = r == current_role.as_str();
                                                                                view! {
                                                                                    <option value=r selected=selected>{r}</option>
                                                                                }
                                                                            }).collect_view()}
                                                                        </select>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <span class="text-muted text-sm capitalize">{current_role}</span>
                                                                    }.into_any()
                                                                }}

                                                                {if is_leader && !is_self {
                                                                    view! {
                                                                        <button
                                                                            class="text-muted hover:text-danger text-sm transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-danger/50 focus-visible:outline-none rounded"
                                                                            title="Remove from team"
                                                                            aria-label="Remove from team"
                                                                            on:click=move |_| {
                                                                                let uid = uid_kick.clone();
                                                                                leptos::task::spawn_local(async move {
                                                                                    let _ = kick_member(uid).await;
                                                                                    dashboard.refetch();
                                                                                });
                                                                            }
                                                                        >
                                                                            "✕"
                                                                        </button>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <span></span> }.into_any()
                                                                }}
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_any()
                                    }}
                                </div>

                                // Recent games
                                <div>
                                    <div class="flex items-center justify-between mb-3">
                                        <div>
                                            <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Battle log"</div>
                                            <h3 class="font-display italic text-2xl text-primary">"Recent Games"</h3>
                                        </div>
                                        <A href="/stats" attr:class="text-accent text-sm hover:underline focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md">"View all stats \u{2192}"</A>
                                    </div>
                                    <Suspense fallback=|| view! { <SkeletonCard height="h-24" /> }>
                                        {move || recent_matches.get().map(|res| match res {
                                            Ok(matches) if matches.is_empty() => {
                                                view! { <p class="text-dimmed text-sm">"No matches synced yet. Go to Stats to sync match history."</p> }.into_any()
                                            }
                                            Ok(matches) => {
                                                view! {
                                                    <div class="flex flex-col gap-2">
                                                        {matches.into_iter().map(|m| {
                                                            let champ_img = format!(
                                                                "https://ddragon.leagueoflegends.com/cdn/15.5.1/img/champion/{}.png",
                                                                m.champion
                                                            );
                                                            let kda = format!("{}/{}/{}", m.kills, m.deaths, m.assists);
                                                            let win = m.win;
                                                            let date_str = m.game_end.unwrap_or_else(|| "Unknown".into());
                                                            view! {
                                                                <div class=if win {
                                                                    "flex items-center gap-3 bg-info/10 border border-info/30 rounded-xl px-4 py-2.5"
                                                                } else {
                                                                    "flex items-center gap-3 bg-danger/10 border border-danger/30 rounded-xl px-4 py-2.5"
                                                                }>
                                                                    <img src=champ_img alt=m.champion.clone() class="w-8 h-8 rounded-full" />
                                                                    <div class="flex-1 min-w-0">
                                                                        <div class="flex items-center gap-2">
                                                                            <span class="text-primary text-sm font-medium">{m.champion}</span>
                                                                            <span class="text-muted text-xs">{m.username}</span>
                                                                        </div>
                                                                        <span class="text-dimmed text-xs font-mono">{date_str}</span>
                                                                    </div>
                                                                    <span class="text-secondary text-sm font-mono tabular-nums">{kda}</span>
                                                                    <span class=if win {
                                                                        "font-imperial uppercase tracking-[0.18em] text-[10px] text-info bg-info/20 rounded-md px-2 py-0.5"
                                                                    } else {
                                                                        "font-imperial uppercase tracking-[0.18em] text-[10px] text-danger bg-danger/20 rounded-md px-2 py-0.5"
                                                                    }>
                                                                        {if win { "W" } else { "L" }}
                                                                    </span>
                                                                </div>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                }.into_any()
                                            }
                                            Err(_) => view! { <p class="text-dimmed text-sm">"Could not load recent matches."</p> }.into_any(),
                                        })}
                                    </Suspense>
                                </div>

                                // Team Notebook
                                <TeamNotebook current_user_id=current_user_id.clone() is_leader=is_leader />

                                // Action Items widget
                                <div>
                                    <div class="flex items-center justify-between mb-3">
                                        <div>
                                            <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Tasking"</div>
                                            <h3 class="font-display italic text-2xl text-primary">"Open Action Items"</h3>
                                        </div>
                                        <A href="/action-items" attr:class="text-accent text-sm hover:underline focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md">"View all \u{2192}"</A>
                                    </div>
                                    <Suspense fallback=|| view! { <SkeletonCard height="h-24" /> }>
                                        {move || action_items_res.get().map(|result| match result {
                                            Ok((total, top_items)) => {
                                                if total == 0 {
                                                    view! {
                                                        <p class="text-dimmed text-sm">"No open action items. "</p>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="space-y-2">
                                                            <p class="text-secondary text-sm mb-2">{total}" open item(s)"</p>
                                                            {top_items.into_iter().map(|item| {
                                                                let status_dot = match item.status.as_str() {
                                                                    "in_progress" => "w-2.5 h-2.5 rounded-full bg-warning shrink-0",
                                                                    _ => "w-2.5 h-2.5 rounded-full bg-success shrink-0",
                                                                };
                                                                view! {
                                                                    <div class="flex items-center gap-2 bg-elevated border border-outline/50 rounded-lg px-3 py-2">
                                                                        <span class=status_dot aria-hidden="true"></span>
                                                                        <span class="text-primary text-sm truncate">{item.text}</span>
                                                                        {item.assigned_to.map(|a| view! {
                                                                            <span class="text-xs bg-surface text-muted rounded-md px-2 py-0.5 shrink-0">{a}</span>
                                                                        })}
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_any()
                                                }
                                            }
                                            Err(_) => view! { <p class="text-dimmed text-sm">"Could not load action items."</p> }.into_any(),
                                        })}
                                    </Suspense>
                                </div>

                                // Post-Game Summaries Panel
                                <div>
                                    <div class="flex items-center justify-between mb-3">
                                        <div>
                                            <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Debrief"</div>
                                            <h3 class="font-display italic text-2xl text-primary">"Recent Reviews"</h3>
                                        </div>
                                        <A href="/post-game" attr:class="text-accent text-sm hover:underline focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md">"View all reviews"</A>
                                    </div>
                                    <Suspense fallback=|| view! { <SkeletonCard height="h-24" /> }>
                                        {move || post_game_panel.get().map(|result| match result {
                                            Ok(previews) if previews.is_empty() => view! {
                                                <p class="text-dimmed text-sm">
                                                    "No post-game reviews yet. "
                                                    <A href="/post-game" attr:class="text-accent hover:underline">"Start your first review"</A>
                                                </p>
                                            }.into_any(),
                                            Ok(previews) => view! {
                                                <div class="space-y-2">
                                                    {previews.into_iter().map(|preview| {
                                                        let extra = if preview.improvements.len() > 2 {
                                                            Some(preview.improvements.len() - 2)
                                                        } else {
                                                            None
                                                        };
                                                        let top_improvements: Vec<String> = preview.improvements.into_iter().take(2).collect();
                                                        view! {
                                                            <div class="bg-elevated border border-outline/50 rounded-xl p-3">
                                                                {preview.created_at.map(|d| view! {
                                                                    <p class="text-xs text-muted mb-1 font-mono">{format_timestamp(&d)}</p>
                                                                })}
                                                                {top_improvements.into_iter().map(|imp| view! {
                                                                    <p class="text-sm text-secondary truncate">{imp}</p>
                                                                }).collect_view()}
                                                                {extra.map(|n| view! {
                                                                    <p class="text-xs text-muted mt-1">{"+"}{n}{" more"}</p>
                                                                })}
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any(),
                                            Err(_) => view! {
                                                <p class="text-dimmed text-sm">"Could not load post-game reviews."</p>
                                            }.into_any(),
                                        })}
                                    </Suspense>
                                </div>

                                // Pool Gap Warnings Panel
                                <div>
                                    <div class="flex items-center justify-between mb-3">
                                        <div>
                                            <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Reconnaissance"</div>
                                            <h3 class="font-display italic text-2xl text-primary">"Pool Gap Warnings"</h3>
                                        </div>
                                        <A href="/champion-pool" attr:class="text-accent text-sm hover:underline focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md">"Manage pools"</A>
                                    </div>
                                    <Suspense fallback=|| view! { <SkeletonCard height="h-24" /> }>
                                        {move || pool_gap_panel.get().map(|result| match result {
                                            Ok(warnings) if warnings.is_empty() => view! {
                                                <p class="text-dimmed text-sm">
                                                    "No pool gaps detected. "
                                                    <A href="/champion-pool" attr:class="text-accent hover:underline">"Manage champion pools"</A>
                                                </p>
                                            }.into_any(),
                                            Ok(warnings) => view! {
                                                <div class="space-y-2">
                                                    {warnings.into_iter().map(|warning| {
                                                        let label = if warning.opponent_escalated {
                                                            format!("{} ({}) — opponent threat", warning.username, warning.role)
                                                        } else {
                                                            format!("{} ({})", warning.username, warning.role)
                                                        };
                                                        let missing = warning.missing_classes.join(", ");
                                                        view! {
                                                            <div class="flex items-start gap-2 bg-elevated border border-warning/30 rounded-lg px-3 py-2">
                                                                <span class="text-warning shrink-0 font-bold" aria-hidden="true">"!"</span>
                                                                <div>
                                                                    <p class="text-primary text-sm">{label}</p>
                                                                    <p class="text-xs text-muted">{"Missing: "}{missing}</p>
                                                                    {warning.dominant_class.map(|dc| view! {
                                                                        <span class="text-xs bg-surface text-muted rounded-md px-2 py-0.5">{"Dominant: "}{dc}</span>
                                                                    })}
                                                                </div>
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any(),
                                            Err(_) => view! {
                                                <p class="text-dimmed text-sm">"Could not load pool gap warnings."</p>
                                            }.into_any(),
                                        })}
                                    </Suspense>
                                </div>

                                // Leave team (non-leaders only)
                                {if !is_leader {
                                    view! {
                                        <div class="border-t border-divider pt-4">
                                            {move || if leave_confirm.get() {
                                                view! {
                                                    <div class="flex items-center gap-3">
                                                        <span class="text-secondary text-sm">"Are you sure you want to leave this team?"</span>
                                                        <button
                                                            class="bg-danger/10 text-danger border border-danger/30 hover:bg-danger/20 text-sm font-medium rounded-lg px-3 py-1.5 transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-danger/50 focus-visible:outline-none"
                                                            on:click=move |_| {
                                                                leptos::task::spawn_local(async move {
                                                                    match leave_team().await {
                                                                        Ok(_) => {
                                                                            toast.show.run((ToastKind::Success, "You have left the team".into()));
                                                                            dashboard.refetch();
                                                                        }
                                                                        Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                                                                    }
                                                                });
                                                                set_leave_confirm.set(false);
                                                            }
                                                        >"Yes, leave"</button>
                                                        <button
                                                            class="bg-surface hover:bg-overlay text-secondary text-sm rounded-lg px-3 py-1.5 transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                            on:click=move |_| set_leave_confirm.set(false)
                                                        >"Cancel"</button>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <button
                                                        class="text-danger hover:opacity-80 text-sm transition-colors border border-danger/30 hover:border-danger/50 rounded-lg px-3 py-1.5 cursor-pointer focus-visible:ring-2 focus-visible:ring-danger/50 focus-visible:outline-none"
                                                        on:click=move |_| set_leave_confirm.set(true)
                                                    >"Leave Team"</button>
                                                }.into_any()
                                            }}
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}
                            </div>
                        }.into_any()
                        } // end else (Demacia) branch
                    },
                    Ok(None) => view! {
                        <NoTeamState />
                    }.into_any(),
                    Err(e) => view! {
                        <ErrorBanner message=format!("Failed to load team data: {e}") />
                    }.into_any(),
                })}}
            </Suspense>
            </div>
        </div>
    }
}

#[component]
fn TeamNotebook(current_user_id: String, is_leader: bool) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let notes_resource = Resource::new(|| (), |_| get_all_team_notes());
    let (expanded, set_expanded) = signal(false);
    let (new_note, set_new_note) = signal(String::new());
    let (editing_id, set_editing_id) = signal(Option::<String>::None);
    let (edit_content, set_edit_content) = signal(String::new());

    view! {
        <div>
            <div class="flex items-center justify-between mb-3">
                <div>
                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Folio"</div>
                    <h3 class="font-display italic text-2xl text-primary">"Team Notebook"</h3>
                </div>
                <button
                    class="text-muted hover:text-accent text-sm transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md px-2 py-1"
                    on:click=move |_| set_expanded.update(|v| *v = !*v)
                >
                    {move || if expanded.get() { "Collapse" } else { "Show all" }}
                </button>
            </div>

            // Add note form
            <div class="flex gap-2 mb-4">
                <textarea
                    prop:value=move || new_note.get()
                    on:input=move |ev| set_new_note.set(event_target_value(&ev))
                    placeholder="Write a note for the team..."
                    rows="2"
                    class="flex-1 bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none resize-none"
                />
                <button
                    class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded-lg px-4 py-2 text-sm transition-colors cursor-pointer self-end focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                    on:click=move |_| {
                        let content = new_note.get_untracked();
                        if content.trim().is_empty() {
                            return;
                        }
                        leptos::task::spawn_local(async move {
                            match add_team_note(content).await {
                                Ok(_) => {
                                    set_new_note.set(String::new());
                                    notes_resource.refetch();
                                }
                                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                            }
                        });
                    }
                >"Add"</button>
            </div>

            <Suspense fallback=|| view! { <div class="flex flex-col gap-1"><SkeletonCard height="h-10" /><SkeletonCard height="h-10" /></div> }>
                {move || {
                    let uid = current_user_id.clone();
                    let leader = is_leader;
                    notes_resource.get().map(move |res| match res {
                        Ok(notes) if notes.is_empty() => {
                            view! { <p class="text-dimmed text-sm">"No notes yet. Be the first to write one!"</p> }.into_any()
                        }
                        Ok(notes) => {
                            let show_expanded = expanded.get();
                            let display_notes: Vec<_> = if show_expanded {
                                notes
                            } else {
                                // Collapsed: show only pinned notes
                                notes.into_iter().filter(|n| n.pinned).collect()
                            };

                            if display_notes.is_empty() && !show_expanded {
                                view! {
                                    <p class="text-dimmed text-sm">"No pinned notes. Click \"Show all\" to see all notes."</p>
                                }.into_any()
                            } else {
                                let uid2 = uid.clone();
                                view! {
                                    <div class="flex flex-col gap-2">
                                        {display_notes.into_iter().map(|note| {
                                            let note_id = note.id.clone().unwrap_or_default();
                                            let note_id_pin = note_id.clone();
                                            let note_id_del = note_id.clone();
                                            let note_id_edit = note_id.clone();
                                            let note_id_save = note_id.clone();
                                            let is_author = note.author_id == uid2;
                                            let can_delete = is_author || leader;
                                            let pinned = note.pinned;
                                            let initial = note.author_name.chars().next().unwrap_or('?').to_uppercase().to_string();
                                            let content_for_edit = note.content.clone();

                                            view! {
                                                <div class=if pinned {
                                                    "bg-accent/10 border border-accent/30 rounded-xl p-4"
                                                } else {
                                                    "bg-elevated border border-outline/50 rounded-xl p-4"
                                                }>
                                                    <div class="flex items-start gap-3">
                                                        // Author avatar
                                                        <span class="bg-accent/20 text-accent rounded-full w-8 h-8 flex items-center justify-center text-xs font-bold flex-shrink-0">
                                                            {initial}
                                                        </span>
                                                        <div class="flex-1 min-w-0">
                                                            <div class="flex items-center gap-2 mb-1">
                                                                <span class="text-primary text-sm font-medium">{note.author_name.clone()}</span>
                                                                {if pinned {
                                                                    view! {
                                                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 text-accent" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                                                            <path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z"/>
                                                                        </svg>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <span></span> }.into_any()
                                                                }}
                                                                <span class="text-dimmed text-xs ml-auto font-mono">
                                                                    {note.created_at.as_deref().map(format_timestamp).unwrap_or_default()}
                                                                </span>
                                                            </div>

                                                            // Content: show edit form or text
                                                            {
                                                                let note_id_for_display = note_id_edit.clone();
                                                                let note_id_for_save = note_id_save.clone();
                                                                let content_text = content_for_edit.clone();
                                                                move || {
                                                                    let nid = note_id_for_display.clone();
                                                                    let nid_save = note_id_for_save.clone();
                                                                    let ct = content_text.clone();
                                                                    if editing_id.get() == Some(nid) {
                                                                        view! {
                                                                            <div class="flex flex-col gap-2">
                                                                                <textarea
                                                                                    prop:value=move || edit_content.get()
                                                                                    on:input=move |ev| set_edit_content.set(event_target_value(&ev))
                                                                                    rows="3"
                                                                                    class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none resize-none"
                                                                                />
                                                                                <div class="flex gap-2">
                                                                                    <button
                                                                                        class="bg-accent hover:bg-accent-hover text-accent-contrast text-xs font-bold rounded-lg px-3 py-1 transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                                                        on:click=move |_| {
                                                                                            let id = nid_save.clone();
                                                                                            let content = edit_content.get_untracked();
                                                                                            leptos::task::spawn_local(async move {
                                                                                                match edit_team_note(id, content).await {
                                                                                                    Ok(_) => {
                                                                                                        set_editing_id.set(None);
                                                                                                        notes_resource.refetch();
                                                                                                    }
                                                                                                    Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                                                                                                }
                                                                                            });
                                                                                        }
                                                                                    >"Save"</button>
                                                                                    <button
                                                                                        class="text-secondary hover:text-primary text-xs rounded-lg px-3 py-1 transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                                                        on:click=move |_| set_editing_id.set(None)
                                                                                    >"Cancel"</button>
                                                                                </div>
                                                                            </div>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <p class="text-secondary text-sm whitespace-pre-wrap">{ct}</p>
                                                                        }.into_any()
                                                                    }
                                                                }
                                                            }

                                                            // Action buttons
                                                            <div class="flex items-center gap-2 mt-2">
                                                                // Pin/Unpin
                                                                <button
                                                                    class="text-muted hover:text-accent text-xs transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md px-1"
                                                                    title=if pinned { "Unpin" } else { "Pin" }
                                                                    on:click=move |_| {
                                                                        let id = note_id_pin.clone();
                                                                        let new_pinned = !pinned;
                                                                        leptos::task::spawn_local(async move {
                                                                            match toggle_note_pin(id, new_pinned).await {
                                                                                Ok(_) => notes_resource.refetch(),
                                                                                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                                                                            }
                                                                        });
                                                                    }
                                                                >
                                                                    {if pinned { "Unpin" } else { "Pin" }}
                                                                </button>

                                                                // Edit (author only)
                                                                {if is_author {
                                                                    let content_for_btn = note.content.clone();
                                                                    let nid = note_id.clone();
                                                                    view! {
                                                                        <button
                                                                            class="text-muted hover:text-accent text-xs transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md px-1"
                                                                            on:click=move |_| {
                                                                                set_edit_content.set(content_for_btn.clone());
                                                                                set_editing_id.set(Some(nid.clone()));
                                                                            }
                                                                        >"Edit"</button>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <span></span> }.into_any()
                                                                }}

                                                                // Delete (author or leader)
                                                                {if can_delete {
                                                                    view! {
                                                                        <button
                                                                            class="text-muted hover:text-danger text-xs transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-danger/50 focus-visible:outline-none rounded-md px-1"
                                                                            on:click=move |_| {
                                                                                let id = note_id_del.clone();
                                                                                leptos::task::spawn_local(async move {
                                                                                    match remove_team_note(id).await {
                                                                                        Ok(_) => notes_resource.refetch(),
                                                                                        Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                                                                                    }
                                                                                });
                                                                            }
                                                                        >"Delete"</button>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <span></span> }.into_any()
                                                                }}
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }
                        }
                        Err(e) => view! {
                            <p class="text-danger text-sm" role="alert">{format!("Could not load notes: {e}")}</p>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Pandemonium Team Dashboard — 7-section full data-surface rebuild
// Per CONTENT-CONTRACT-AUDIT.md: all 7 sections must be visually present.
// Placeholder values have TODO comments for future data-source wiring.
// ---------------------------------------------------------------------------

/// Pandemonium variant of the team dashboard. Receives team + members from the
/// parent Suspense so it renders without an additional network round-trip.
#[component]
fn PandemoniumTeamDashboard(
    team: crate::models::team::Team,
    members: Vec<TeamMember>,
) -> impl IntoView {
    let team_name = team.name.clone();

    // Section 2: build roster slots from real member data where available.
    // Pad to 5 slots with empty placeholders.
    let starters: Vec<Option<TeamMember>> = {
        let mut slots: Vec<Option<TeamMember>> = vec![None; 5];
        let role_order = ["top", "jungle", "mid", "bot", "support"];
        for (i, role) in role_order.iter().enumerate() {
            if let Some(m) = members.iter().find(|m| m.role == *role && m.roster_type == "starter") {
                slots[i] = Some(m.clone());
            }
        }
        slots
    };
    let role_labels = ["TOP", "JGL", "MID", "BOT", "SUP"];

    view! {
        <div class="space-y-3 bg-base bg-scanline p-4">

            // Section 1: RiotTape header strip
            <RiotTape width=1200 label="TEAM_BRIEF GAME_DAY" />

            // Section 2: 5-player roster row with MoodMeter
            <div>
                <div class="font-mono text-[10px] text-muted uppercase tracking-[0.16em] mb-2">"// ROSTER"</div>
                <div class="grid grid-cols-5 gap-2">
                    {starters.into_iter().enumerate().map(|(i, slot)| {
                        let role_label = role_labels[i];
                        // TODO(future phase): wire mood from team-vibe-check feature
                        let mood_value = 0.7_f64;
                        let player_name = slot.map(|m| m.username).unwrap_or_else(|| "// EMPTY".to_string());
                        let name_clone = player_name.clone();
                        view! {
                            <div class="bg-surface border border-outline/30 p-3 flex flex-col items-center gap-2 relative">
                                // bracket corners (zine aesthetic)
                                <div class="absolute top-0 left-0 w-2 h-2 border-l-2 border-t-2 border-accent"></div>
                                <div class="absolute top-0 right-0 w-2 h-2 border-r-2 border-t-2 border-accent"></div>
                                <div class="absolute bottom-0 left-0 w-2 h-2 border-l-2 border-b-2 border-accent"></div>
                                <div class="absolute bottom-0 right-0 w-2 h-2 border-r-2 border-b-2 border-accent"></div>
                                <span class="font-mono text-[10px] text-muted uppercase">{role_label}</span>
                                <span class="font-glitch text-[11px] uppercase tracking-[0.14em] text-accent text-center truncate w-full"
                                      style="text-shadow: -1px -1px 0 var(--accent-2), 1px 1px 0 var(--t-accent);">
                                    {name_clone}
                                </span>
                                <MoodMeter value=mood_value />
                                <span class="font-mono text-[9px] text-muted">"MOOD"</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>

            // Section 3: Captain's note
            // TODO(future phase): add captain_note field to Team model and wire here.
            <div class="bg-surface border border-outline/30 p-4 relative">
                <div class="absolute top-0 left-0 w-3 h-3 border-l-2 border-t-2 border-accent"></div>
                <div class="absolute top-0 right-0 w-3 h-3 border-r-2 border-t-2 border-accent"></div>
                <div class="absolute bottom-0 left-0 w-3 h-3 border-l-2 border-b-2 border-accent"></div>
                <div class="absolute bottom-0 right-0 w-3 h-3 border-r-2 border-b-2 border-accent"></div>
                <div class="font-mono text-[10px] text-accent uppercase tracking-[0.16em] mb-2">"// FROM_THE_CAPTAIN"</div>
                <p class="font-mono text-[13px] leading-relaxed text-secondary whitespace-pre-wrap">
                    "Watch for their level-6 baron-area invade — we ate it last time.\nMid roams hard at 3:45. Stay on wards, keep tempo."
                </p>
            </div>

            // Section 4: Reasoned Bans
            // TODO(future phase): wire from ban-reasoning feature when built.
            <PandemoniumBansPanel />

            // Section 5: Our Pool Ready
            // TODO(future phase): compute from champion_pool resource per-player.
            <div class="bg-surface border border-outline/30 p-4 relative">
                <div class="absolute top-0 left-0 w-3 h-3 border-l-2 border-t-2 border-accent"></div>
                <div class="absolute top-0 right-0 w-3 h-3 border-r-2 border-t-2 border-accent"></div>
                <div class="absolute bottom-0 left-0 w-3 h-3 border-l-2 border-b-2 border-accent"></div>
                <div class="absolute bottom-0 right-0 w-3 h-3 border-r-2 border-b-2 border-accent"></div>
                <div class="font-mono text-[10px] text-accent uppercase tracking-[0.16em] mb-2">"// READINESS"</div>
                <div class="font-mono text-[11px] text-muted uppercase mb-3">"OUR POOL READY"</div>
                <div class="flex items-baseline gap-2">
                    <span class="font-mono text-[28px] tabular-nums text-accent">"4"</span>
                    <span class="font-mono text-muted text-[12px]">"/ 5 PLAYERS · POOL FILLED"</span>
                </div>
                <div class="mt-2 h-2 bg-elevated">
                    <div class="h-2 bg-accent" style="width: 80%"></div>
                </div>
            </div>

            // Section 6: Their Pattern (opponent intel)
            // TODO(future phase): wire from get_opponents_for_team or opponent-intel resource.
            <div class="bg-surface border border-outline/30 p-4 relative">
                <div class="absolute top-0 left-0 w-3 h-3 border-l-2 border-t-2 border-accent"></div>
                <div class="absolute top-0 right-0 w-3 h-3 border-r-2 border-t-2 border-accent"></div>
                <div class="absolute bottom-0 left-0 w-3 h-3 border-l-2 border-b-2 border-accent"></div>
                <div class="absolute bottom-0 right-0 w-3 h-3 border-r-2 border-b-2 border-accent"></div>
                <div class="font-mono text-[10px] text-accent uppercase tracking-[0.16em] mb-2">"// SCOUT"</div>
                <div class="font-mono text-[11px] text-muted uppercase mb-3">"THEIR PATTERN"</div>
                <div class="space-y-2 font-mono text-[12px]">
                    <div>
                        <span class="text-accent">"// LAST_5_BANS"</span>
                        <span class="ml-2 text-secondary">"Yasuo, Yone, Akali, Zed, Sylas"</span>
                    </div>
                    <div>
                        <span class="text-accent">"// PICK_HABIT"</span>
                        <span class="ml-2 text-secondary">"Mid-priority drafts; engage support always blue side"</span>
                    </div>
                    <div>
                        <span class="text-accent">"// EARLY_GAME"</span>
                        <span class="ml-2 text-secondary">"Jungle invades 2:30–3:30; rotates bot at 5:00"</span>
                    </div>
                </div>
            </div>

            // Section 7: Threat Ranking + "If you let it through" warnings
            <PandemoniumThreatsPanel />

            // Team name footer
            <div class="pt-2 border-t border-outline/20">
                <span class="font-mono text-[10px] text-dimmed">"// SQUAD: "</span>
                <span class="font-mono text-[10px] text-muted">{team_name}</span>
            </div>
        </div>
    }
}

/// Pandemonium Section 4: Reasoned bans panel.
/// Hardcoded placeholders — TODO(future phase): wire from ban-reasoning feature.
#[component]
fn PandemoniumBansPanel() -> impl IntoView {
    view! {
        <div class="bg-surface border border-outline/30 p-4 relative">
            <div class="absolute top-0 left-0 w-3 h-3 border-l-2 border-t-2 border-accent"></div>
            <div class="absolute top-0 right-0 w-3 h-3 border-r-2 border-t-2 border-accent"></div>
            <div class="absolute bottom-0 left-0 w-3 h-3 border-l-2 border-b-2 border-accent"></div>
            <div class="absolute bottom-0 right-0 w-3 h-3 border-r-2 border-b-2 border-accent"></div>
            <div class="font-mono text-[10px] text-accent uppercase tracking-[0.16em] mb-2">"// PRE-PICK"</div>
            <div class="font-mono text-[11px] text-muted uppercase mb-3">"REASONED BANS"</div>
            <div class="grid grid-cols-2 gap-3">
                <div class="flex items-start gap-2">
                    <ChampTile name="Yasuo".to_string() size=40 banned=true />
                    <div class="flex-1">
                        <span class="inline-flex items-center px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.12em] rounded-sm bg-danger/15 text-danger">"BAN"</span>
                        <p class="font-mono text-[11px] text-muted mt-1">"OTP on mid; team-fight uptime kills us"</p>
                    </div>
                </div>
                <div class="flex items-start gap-2">
                    <ChampTile name="Yone".to_string() size=40 banned=true />
                    <div class="flex-1">
                        <span class="inline-flex items-center px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.12em] rounded-sm bg-danger/15 text-danger">"BAN"</span>
                        <p class="font-mono text-[11px] text-muted mt-1">"Their mid/adc both proficient; pick rate 80%"</p>
                    </div>
                </div>
                <div class="flex items-start gap-2">
                    <ChampTile name="Zed".to_string() size=40 banned=true />
                    <div class="flex-1">
                        <span class="inline-flex items-center px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.12em] rounded-sm bg-danger/15 text-danger">"BAN"</span>
                        <p class="font-mono text-[11px] text-muted mt-1">"Snowballs hard; our support can't disengage"</p>
                    </div>
                </div>
                <div class="flex items-start gap-2">
                    <ChampTile name="Akali".to_string() size=40 banned=true />
                    <div class="flex-1">
                        <span class="inline-flex items-center px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.12em] rounded-sm bg-elevated text-muted">"FLEX-BAN"</span>
                        <p class="font-mono text-[11px] text-muted mt-1">"Counter to our top's current champion pool"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Pandemonium Section 7: Threat ranking + "If you let it through" warnings.
/// Hardcoded placeholders — TODO(future phase): wire from opponent-intel threat scoring.
#[component]
fn PandemoniumThreatsPanel() -> impl IntoView {
    view! {
        <div class="bg-surface border border-outline/30 p-4 relative">
            <div class="absolute top-0 left-0 w-3 h-3 border-l-2 border-t-2 border-accent"></div>
            <div class="absolute top-0 right-0 w-3 h-3 border-r-2 border-t-2 border-accent"></div>
            <div class="absolute bottom-0 left-0 w-3 h-3 border-l-2 border-b-2 border-accent"></div>
            <div class="absolute bottom-0 right-0 w-3 h-3 border-r-2 border-b-2 border-accent"></div>
            <div class="font-mono text-[10px] text-accent uppercase tracking-[0.16em] mb-2">"// PRIORITY"</div>
            <div class="font-mono text-[11px] text-muted uppercase mb-3">"THREATS"</div>
            <ol class="space-y-3">
                <li class="flex items-start gap-3">
                    <span class="font-mono text-[18px] text-accent tabular-nums">"1."</span>
                    <ChampTile name="Azir".to_string() size=40 />
                    <div class="flex-1">
                        <div class="flex items-center gap-2">
                            <span class="inline-flex items-center px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.12em] rounded-sm bg-danger/15 text-danger">"CRITICAL"</span>
                            <span class="font-mono text-[12px] text-primary">"Azir"</span>
                        </div>
                        <p class="font-mono text-[11px] text-danger mt-1">"If you let it through: 65% chance of team-fight loss"</p>
                    </div>
                </li>
                <li class="flex items-start gap-3">
                    <span class="font-mono text-[18px] text-accent tabular-nums">"2."</span>
                    <ChampTile name="Orianna".to_string() size=40 />
                    <div class="flex-1">
                        <div class="flex items-center gap-2">
                            <span class="inline-flex items-center px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.12em] rounded-sm bg-elevated text-muted">"HIGH"</span>
                            <span class="font-mono text-[12px] text-primary">"Orianna"</span>
                        </div>
                        <p class="font-mono text-[11px] text-danger mt-1">"If you let it through: peel composition counters our dive"</p>
                    </div>
                </li>
                <li class="flex items-start gap-3">
                    <span class="font-mono text-[18px] text-accent tabular-nums">"3."</span>
                    <ChampTile name="Leona".to_string() size=40 />
                    <div class="flex-1">
                        <div class="flex items-center gap-2">
                            <span class="inline-flex items-center px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.12em] rounded-sm bg-elevated text-muted">"MED"</span>
                            <span class="font-mono text-[12px] text-primary">"Leona"</span>
                        </div>
                        <p class="font-mono text-[11px] text-muted mt-1">"If you let it through: stun-chain disrupts our rotations"</p>
                    </div>
                </li>
            </ol>
        </div>
    }
}

// ---------------------------------------------------------------------------
// TeamGameDayBriefView — 18-07
// Game-day brief sub-view for /team/dashboard route mode="brief".
// Demacia: "THE COMPANION GAZETTE" newspaper (3-col, gilt, Cormorant)
// Pandemonium: "GAME_DAY · ZINE_v0.3" collage (RiotTape, rotated zine cards)
// Mode toggle wired in 18-08; reachable now via mode="brief" stub.
//
// ChildrenFn rule: all String props inside Card/Glitch children must use
// .clone() so the generated closure remains Fn. Data needing multiple
// closure accesses uses StoredValue<T> (Copy, so closures stay Fn).
// ---------------------------------------------------------------------------

#[component]
fn TeamGameDayBriefView(
    region: String,
    team: crate::models::team::Team,
    members: Vec<crate::models::user::TeamMember>,
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";

    let team_name_sv = StoredValue::new(team.name.clone());
    let members_sv = StoredValue::new(members);

    if is_pandemonium {
        // Pandemonium: xeroxed match-day zine
        let r1 = region.clone();
        let r2 = region.clone();
        let r3 = region.clone();
        let r4 = region.clone();
        let r5 = region.clone();
        let r6 = region.clone();
        let r7 = region.clone();
        let r8 = region.clone();
        let r9 = region.clone();
        let r10 = region.clone();
        view! {
            <div class="canvas-grain bg-base flex flex-col gap-3">
                <RiotTape label="GAME_DAY · ZINE_v0.3" />

                // Section 1: Roster card
                <div class="transform -rotate-1">
                    <Card region=r1 variant="zine">
                        <Glitch region=r2.clone()>"// ROSTER_CARD"</Glitch>
                        <div class="mt-2 grid grid-cols-5 gap-2">
                            {move || members_sv.get_value().into_iter().take(5).map(|m| {
                                let role_sv = StoredValue::new(m.role.to_uppercase());
                                let name_sv = StoredValue::new(
                                    m.riot_summoner_name.unwrap_or_else(|| m.username.clone())
                                );
                                view! {
                                    <div class="flex flex-col items-center gap-1">
                                        <ChampTile name=name_sv.get_value() size=36 />
                                        <span class="font-mono text-[9px] text-muted">
                                            {move || format!("// {}", role_sv.get_value())}
                                        </span>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </Card>
                </div>

                // Section 2: Strat note
                <div class="transform rotate-1">
                    <Card region=r3 variant="zine">
                        <Glitch region=r4.clone()>"// STRAT_NOTE"</Glitch>
                        // TODO: wire real strategy from game_plan resource once linked
                        <pre class="font-mono text-[11px] text-secondary mt-2 whitespace-pre-wrap leading-relaxed">
                            "WIN_CON: early tower pressure\nBANS: focus engage supports\nDRAFT: flex top/mid picks preferred"
                        </pre>
                    </Card>
                </div>

                // Section 3: Opponent intel
                <div class="transform -rotate-1">
                    <Card region=r5 variant="zine">
                        <Glitch region=r6.clone()>"// OPPONENT_INTEL"</Glitch>
                        // TODO: wire real opponent data from match history when available
                        <div class="mt-2 flex flex-col gap-1 font-mono text-[11px]">
                            <div class="text-secondary">"LAST_5_BANS: Yasuo, Yone, Zed, Katarina, Akali"</div>
                            <div class="text-secondary">"LAST_5_FORM: W · L · W · W · L"</div>
                            <div class="text-secondary">"PLAYSTYLE: aggressive early, poke-heavy mid"</div>
                        </div>
                    </Card>
                </div>

                // Section 4: Threat rank
                <div class="transform rotate-1">
                    <Card region=r7 variant="zine">
                        <Glitch region=r8.clone()>"// THREAT_RANK"</Glitch>
                        <div class="mt-2 flex flex-col gap-2">
                            <div class="flex items-center gap-3 border-b border-outline/20 pb-2">
                                <ChampTile name="Azir".to_string() size=32 />
                                <div class="flex-1">
                                    <div class="flex items-center gap-2">
                                        <Badge tone="danger">"CRITICAL"</Badge>
                                        <span class="font-mono text-[11px] text-primary">"Azir"</span>
                                    </div>
                                    <p class="font-mono text-[10px] text-muted mt-1">"// if let through: 65% team-fight loss"</p>
                                </div>
                            </div>
                            <div class="flex items-center gap-3 border-b border-outline/20 pb-2">
                                <ChampTile name="Orianna".to_string() size=32 />
                                <div class="flex-1">
                                    <div class="flex items-center gap-2">
                                        <Badge tone="warning">"HIGH"</Badge>
                                        <span class="font-mono text-[11px] text-primary">"Orianna"</span>
                                    </div>
                                    <p class="font-mono text-[10px] text-muted mt-1">"// if let through: peel counters our dive"</p>
                                </div>
                            </div>
                            <div class="flex items-center gap-3">
                                <ChampTile name="Leona".to_string() size=32 />
                                <div class="flex-1">
                                    <div class="flex items-center gap-2">
                                        <Badge tone="neutral">"MED"</Badge>
                                        <span class="font-mono text-[11px] text-primary">"Leona"</span>
                                    </div>
                                    <p class="font-mono text-[10px] text-muted mt-1">"// if let through: stun-chain disrupts rotations"</p>
                                </div>
                            </div>
                        </div>
                    </Card>
                </div>

                // Footer: squad tag
                <Card region=r9 variant="zine">
                    <Glitch region=r10.clone()>
                        {move || format!("// SQUAD: {}", team_name_sv.get_value())}
                    </Glitch>
                    <div class="mt-1 font-mono text-[10px] text-muted">
                        {move || format!("{} MEMBERS CONFIRMED", members_sv.get_value().len())}
                    </div>
                </Card>
            </div>
        }.into_any()
    } else {
        // Demacia: "THE COMPANION GAZETTE" newspaper layout
        let r1 = region.clone();
        let r2 = region.clone();
        let r3 = region.clone();
        let r4 = region.clone();
        let r5 = region.clone();
        view! {
            <div class="flex flex-col gap-6">
                // Masthead
                <Card region=r1.clone() variant="gilt">
                    <div class="text-center py-2">
                        <div class="font-imperial text-[9px] uppercase tracking-[0.25em] text-muted mb-1">
                            "GAME DAY EDITION"
                        </div>
                        <h1 class="font-display font-bold text-[28px] tracking-[0.06em] text-primary uppercase">
                            "THE COMPANION GAZETTE"
                        </h1>
                        <div class="font-imperial text-[10px] text-muted mt-1 tracking-[0.15em]">
                            {move || format!("TEAM: {}", team_name_sv.get_value())}
                        </div>
                        <div class="mt-3 flex justify-center">
                            <HeraldicDivider width=640 />
                        </div>
                    </div>
                </Card>

                // Three-column newspaper body
                <div class="grid grid-cols-3 gap-4">
                    // Column 1: Roster
                    <Card region=r2.clone() variant="gilt">
                        <SectionHead
                            region=r2.clone()
                            eyebrow="TODAY'S LINEUP"
                            title="Roster".to_string()
                        />
                        <div class="mt-2 flex justify-center">
                            <HeraldicDivider width=220 />
                        </div>
                        // Real roster from team members
                        <div class="mt-3 flex flex-col gap-3">
                            {move || members_sv.get_value().into_iter().take(5).map(|m| {
                                let role_sv = StoredValue::new(m.role.clone());
                                let name_sv = StoredValue::new(
                                    m.riot_summoner_name.clone().unwrap_or_else(|| m.username.clone())
                                );
                                view! {
                                    <div class="flex items-center gap-2 border-b border-outline/20 pb-2">
                                        <ChampTile name=name_sv.get_value() size=32 />
                                        <div class="flex-1">
                                            <div class="font-imperial text-[8px] uppercase tracking-[0.18em] text-muted">
                                                {move || role_sv.get_value()}
                                            </div>
                                            <div class="font-display text-[13px] text-primary italic">
                                                {move || name_sv.get_value()}
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </Card>

                    // Column 2: Strategy
                    <Card region=r3.clone() variant="gilt">
                        <SectionHead
                            region=r3.clone()
                            eyebrow="TODAY'S ORDERS"
                            title="Strategy".to_string()
                        />
                        <div class="mt-2 flex justify-center">
                            <HeraldicDivider width=220 />
                        </div>
                        // TODO: wire real strategy from game_plan resource once linked
                        <p class="font-display italic text-secondary text-[13px] leading-relaxed mt-3
                            first-letter:float-left first-letter:font-display first-letter:text-5xl first-letter:text-accent first-letter:mr-1 first-letter:leading-none">
                            "Early pressure along the top side. Secure Rift Herald first objective. Ban engage supports. Flex picks preferred for blue side."
                        </p>
                        <div class="mt-4">
                            <div class="font-imperial text-[8px] uppercase tracking-[0.18em] text-muted mb-2">"BAN INTENTIONS"</div>
                            <div class="font-display italic text-[12px] text-secondary">
                                "Priority: Yasuo · Yone · Zed"
                            </div>
                        </div>
                    </Card>

                    // Column 3: Opponent intel + threats
                    <Card region=r4.clone() variant="gilt">
                        <SectionHead
                            region=r4.clone()
                            eyebrow="INTELLIGENCE"
                            title="Opponent Intel".to_string()
                        />
                        <div class="mt-2 flex justify-center">
                            <HeraldicDivider width=220 />
                        </div>
                        // TODO: wire real opponent data from match history when available
                        <div class="mt-3 flex flex-col gap-3">
                            <div>
                                <div class="font-imperial text-[8px] uppercase tracking-[0.18em] text-muted mb-1">"RECENT FORM"</div>
                                <p class="font-display italic text-[12px] text-secondary leading-relaxed
                                    first-letter:float-left first-letter:font-display first-letter:text-5xl first-letter:text-accent first-letter:mr-1 first-letter:leading-none">
                                    "Victory in 3 of last 5. Aggressive early, poke-heavy midgame."
                                </p>
                            </div>
                            <div class="flex justify-center">
                                <HeraldicDivider width=180 />
                            </div>
                            <div>
                                <div class="font-imperial text-[8px] uppercase tracking-[0.18em] text-muted mb-2">"KEY THREATS"</div>
                                <div class="flex flex-col gap-2">
                                    <div class="flex items-center gap-2">
                                        <ChampTile name="Azir".to_string() size=28 />
                                        <span class="font-display italic text-[12px] text-secondary">"Azir — must ban"</span>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        <ChampTile name="Orianna".to_string() size=28 />
                                        <span class="font-display italic text-[12px] text-secondary">"Orianna — peel threat"</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Card>
                </div>

                // Editor's Note sidebar
                <Card region=r5.clone() variant="gilt">
                    <SectionHead
                        region=r5.clone()
                        eyebrow="FROM THE CAPTAIN"
                        title="Editor's Note".to_string()
                    />
                    <div class="mt-2 flex justify-center">
                        <HeraldicDivider width=480 />
                    </div>
                    // TODO: wire real captain's note from team notes once notes
                    // resource exposes a "captain_note" field
                    <p class="font-display italic text-secondary mt-3 text-[14px] leading-relaxed">
                        "Today we face a disciplined opponent. Trust the preparation, execute the early game plan, and maintain map awareness above all else. Steady and methodical — that is how Demacia wins."
                    </p>
                    <div class="mt-3 font-imperial text-[9px] tracking-[0.18em] text-muted uppercase text-right">
                        {move || format!("— {}", team_name_sv.get_value())}
                    </div>
                </Card>
            </div>
        }.into_any()
    }
}
