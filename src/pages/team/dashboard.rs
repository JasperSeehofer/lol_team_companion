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

#[component]
pub fn TeamDashboard() -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
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

    let dashboard = Resource::new(|| (), |_| get_team_dashboard());
    let requests = Resource::new(|| (), |_| get_pending_requests());
    let recent_matches = Resource::new(|| (), |_| get_recent_team_matches());
    let action_items_res = Resource::new(|| (), |_| get_open_action_items_summary());
    let post_game_panel = Resource::new(|| (), |_| get_post_game_panel());
    let pool_gap_panel = Resource::new(|| (), |_| get_pool_gap_panel());

    view! {
        <div class="max-w-4xl mx-auto py-8 px-6">
            <h1 class="text-3xl font-bold text-primary mb-6">"Team Dashboard"</h1>
            <Suspense fallback=|| view! { <SkeletonCard height="h-32" /> }>
                {move || dashboard.get().map(|result| match result {
                    Ok(Some((team, members, current_user_id))) => {
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
                                // Team info card
                                <div class="bg-elevated border border-divider rounded-lg p-6">
                                    <div class="flex items-start justify-between gap-4">
                                        <div>
                                            <div class="flex items-center gap-2">
                                                <h2 class="text-xl font-bold text-accent">{team.name.clone()}</h2>
                                                {if is_leader {
                                                    view! {
                                                        <button
                                                            class="text-muted hover:text-accent transition-colors cursor-pointer p-1 rounded hover:bg-overlay"
                                                            title="Edit team details"
                                                            on:click=move |_| set_show_edit_modal.set(true)
                                                        >
                                                            // Pencil SVG icon
                                                            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                                <path stroke-linecap="round" stroke-linejoin="round" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                                                            </svg>
                                                        </button>
                                                    }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }}
                                            </div>
                                            <p class="text-muted text-sm mt-1">"Region: " {team.region.clone()}</p>
                                            {if is_leader {
                                                view! { <span class="inline-block mt-1 text-xs text-accent font-medium bg-accent/10 rounded px-1.5 py-0.5">"Team Leader"</span> }.into_any()
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
                                                    <A href="/profile" attr:class="text-accent hover:underline">"link it in your profile"</A>
                                                </p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>

                                // Edit team modal (leader only)
                                {move || if show_edit_modal.get() {
                                    view! {
                                        // Backdrop
                                        <div
                                            class="fixed inset-0 bg-black/60 z-50 flex items-center justify-center"
                                            on:click=move |_| set_show_edit_modal.set(false)
                                        >
                                            // Modal content — stop click propagation
                                            <div
                                                class="bg-elevated border border-divider rounded-xl shadow-2xl p-6 w-full max-w-md mx-4"
                                                on:click=move |ev| ev.stop_propagation()
                                            >
                                                <h3 class="text-primary text-lg font-semibold mb-4">"Edit Team"</h3>
                                                <div class="flex flex-col gap-4">
                                                    <div>
                                                        <label class="block text-muted text-xs mb-1">"Team Name"</label>
                                                        <input
                                                            type="text"
                                                            prop:value=move || edit_name.get()
                                                            on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                                            class="w-full bg-surface/50 border border-outline/50 rounded px-3 py-2 text-primary text-sm focus:outline-none focus:border-accent"
                                                        />
                                                    </div>
                                                    <div>
                                                        <label class="block text-muted text-xs mb-1">"Region"</label>
                                                        <select
                                                            prop:value=move || edit_region.get()
                                                            on:change=move |ev| set_edit_region.set(event_target_value(&ev))
                                                            class="w-full bg-surface/50 border border-outline/50 rounded px-3 py-2 text-primary text-sm focus:outline-none focus:border-accent"
                                                        >
                                                            {["EUW","EUNE","NA","KR","BR"].iter().map(|&r| view! {
                                                                <option value=r>{r}</option>
                                                            }).collect_view()}
                                                        </select>
                                                    </div>
                                                    <div class="flex justify-end gap-3 pt-2">
                                                        <button
                                                            class="text-secondary hover:text-primary text-sm px-4 py-2 rounded transition-colors cursor-pointer hover:bg-overlay"
                                                            on:click=move |_| {
                                                                set_show_edit_modal.set(false);
                                                            }
                                                        >
                                                            "Cancel"
                                                        </button>
                                                        <button
                                                            class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-4 py-2 text-sm transition-colors cursor-pointer"
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
                                                            <div class="bg-elevated border border-accent/30 rounded-lg p-5">
                                                                <h3 class="text-accent font-semibold mb-3 flex items-center gap-2">
                                                                    "Join Requests"
                                                                    <span class="bg-accent text-accent-contrast text-xs font-bold rounded-full w-5 h-5 flex items-center justify-center">
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
                                                                                <div class="flex items-center justify-between bg-overlay rounded px-4 py-3">
                                                                                <div>
                                                                                    <span class="text-primary font-medium">{req.username}</span>
                                                                                    {req.riot_summoner_name.map(|n| view! {
                                                                                        <span class="text-muted text-sm ml-2">{n}</span>
                                                                                    })}
                                                                                </div>
                                                                                <div class="flex gap-2">
                                                                                    <button
                                                                                        class="bg-green-700 hover:bg-green-600 text-white text-sm font-medium rounded px-3 py-1.5 transition-colors cursor-pointer"
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
                                                                                        class="bg-overlay-strong hover:bg-red-700 text-secondary hover:text-primary text-sm font-medium rounded px-3 py-1.5 transition-colors cursor-pointer"
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
                                                                                <p class="text-red-400 text-xs px-1">{e}</p>
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
                                    <h3 class="text-lg font-semibold text-primary mb-3">"Starting Roster"</h3>
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
                                                        "bg-elevated border rounded-lg p-3 flex flex-col items-center gap-2 min-h-[120px] transition-colors {}",
                                                        if drag_over.get() { "border-accent bg-overlay" } else { "border-divider" }
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
                                                                            class="text-overlay-strong hover:text-red-400 text-xs transition-colors cursor-pointer"
                                                                            title="Remove from slot"
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
                                                                <span class="text-overlay-strong text-xs">"Empty"</span>
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
                                    <h3 class="text-lg font-semibold text-primary mb-3">"Coaches"</h3>
                                    {if coaches.is_empty() {
                                        view! { <p class="text-dimmed text-sm">"No coaches assigned. Set a member's role to \"coach\" from the bench."</p> }.into_any()
                                    } else {
                                        view! {
                                            <div class="grid grid-cols-2 gap-3">
                                                {coaches.into_iter().map(|m| {
                                                    let is_member_leader = m.user_id == created_by_for_coaches;
                                                    view! {
                                                        <div class="bg-elevated border border-divider rounded-lg p-3 flex items-center gap-3">
                                                            <span class="bg-blue-500/20 text-blue-400 rounded-full w-8 h-8 flex items-center justify-center text-xs font-bold uppercase">
                                                                {m.username.chars().next().unwrap_or('?').to_string()}
                                                            </span>
                                                            <div>
                                                                <div class="flex items-center gap-1">
                                                                    <span class="text-primary text-sm font-medium">{m.username}</span>
                                                                    {is_member_leader.then(|| view! {
                                                                        <span class="text-accent text-xs" title="Team Leader">"★"</span>
                                                                    })}
                                                                </div>
                                                                <span class="text-blue-400 text-xs">"Coach"</span>
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
                                    <h3 class="text-lg font-semibold text-primary mb-3">"Bench / Substitutes"</h3>
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
                                                    let display_name = m.username.clone();
                                                    let (role_msg, set_role_msg) = signal(Option::<String>::None);

                                                    view! {
                                                        <div
                                                            class="bg-elevated border border-divider rounded px-4 py-3 flex items-center justify-between gap-3 cursor-grab active:cursor-grabbing"
                                                            draggable="true"
                                                            on:dragstart=move |ev| {
                                                                if let Some(dt) = ev.data_transfer() {
                                                                    let _ = dt.set_data("text/plain", &uid_drag);
                                                                }
                                                            }
                                                        >
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
                                                                    <span class="text-xs text-green-400">{msg}</span>
                                                                })}
                                                            </div>
                                                            <div class="flex items-center gap-2 flex-shrink-0">
                                                                {if is_leader {
                                                                    let mid = m.user_id.clone();
                                                                    view! {
                                                                        <select
                                                                            class="bg-overlay border border-outline rounded px-2 py-1 text-gray-200 text-sm focus:outline-none focus:border-accent"
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
                                                                            class="text-overlay-strong hover:text-red-400 text-sm transition-colors cursor-pointer"
                                                                            title="Remove from team"
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
                                        <h3 class="text-lg font-semibold text-primary">"Recent Games"</h3>
                                        <A href="/stats" attr:class="text-accent text-sm hover:underline">"View all stats \u{2192}"</A>
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
                                                                    "flex items-center gap-3 bg-blue-950/40 border border-blue-800/30 rounded-lg px-4 py-2.5"
                                                                } else {
                                                                    "flex items-center gap-3 bg-red-950/40 border border-red-800/30 rounded-lg px-4 py-2.5"
                                                                }>
                                                                    <img src=champ_img alt=m.champion.clone() class="w-8 h-8 rounded-full" />
                                                                    <div class="flex-1 min-w-0">
                                                                        <div class="flex items-center gap-2">
                                                                            <span class="text-primary text-sm font-medium">{m.champion}</span>
                                                                            <span class="text-muted text-xs">{m.username}</span>
                                                                        </div>
                                                                        <span class="text-dimmed text-xs">{date_str}</span>
                                                                    </div>
                                                                    <span class="text-secondary text-sm font-medium">{kda}</span>
                                                                    <span class=if win {
                                                                        "text-xs font-bold text-blue-400 bg-blue-500/20 rounded px-1.5 py-0.5"
                                                                    } else {
                                                                        "text-xs font-bold text-red-400 bg-red-500/20 rounded px-1.5 py-0.5"
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
                                        <h3 class="text-lg font-semibold text-primary">"Open Action Items"</h3>
                                        <A href="/action-items" attr:class="text-accent text-sm hover:underline">"View all \u{2192}"</A>
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
                                                                    "in_progress" => "w-2 h-2 rounded-full bg-yellow-500 shrink-0",
                                                                    _ => "w-2 h-2 rounded-full bg-green-500 shrink-0",
                                                                };
                                                                view! {
                                                                    <div class="flex items-center gap-2 bg-elevated border border-divider rounded px-3 py-2">
                                                                        <span class=status_dot></span>
                                                                        <span class="text-primary text-sm truncate">{item.text}</span>
                                                                        {item.assigned_to.map(|a| view! {
                                                                            <span class="text-xs bg-surface text-muted rounded px-1.5 py-0.5 shrink-0">{a}</span>
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
                                        <h3 class="text-lg font-semibold text-primary">"Recent Reviews"</h3>
                                        <A href="/post-game" attr:class="text-accent text-sm hover:underline">"View all reviews"</A>
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
                                                            <div class="bg-elevated border border-divider rounded-lg p-3">
                                                                {preview.created_at.map(|d| view! {
                                                                    <p class="text-xs text-muted mb-1">{format_timestamp(&d)}</p>
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
                                        <h3 class="text-lg font-semibold text-primary">"Pool Gap Warnings"</h3>
                                        <A href="/champion-pool" attr:class="text-accent text-sm hover:underline">"Manage pools"</A>
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
                                                            <div class="flex items-start gap-2 bg-elevated border border-divider rounded px-3 py-2">
                                                                <span class="text-yellow-500 shrink-0 font-bold">"!"</span>
                                                                <div>
                                                                    <p class="text-primary text-sm">{label}</p>
                                                                    <p class="text-xs text-muted">{"Missing: "}{missing}</p>
                                                                    {warning.dominant_class.map(|dc| view! {
                                                                        <span class="text-xs bg-surface text-muted rounded px-1.5 py-0.5">{"Dominant: "}{dc}</span>
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
                                                            class="bg-red-700 hover:bg-red-600 text-white text-sm font-medium rounded px-3 py-1.5 transition-colors cursor-pointer"
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
                                                            class="bg-overlay hover:bg-overlay-strong text-secondary text-sm rounded px-3 py-1.5 transition-colors cursor-pointer"
                                                            on:click=move |_| set_leave_confirm.set(false)
                                                        >"Cancel"</button>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <button
                                                        class="text-red-400 hover:text-red-300 text-sm transition-colors border border-red-400/20 hover:border-red-400/40 rounded-lg px-3 py-1.5 cursor-pointer"
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
                    },
                    Ok(None) => view! {
                        <NoTeamState />
                    }.into_any(),
                    Err(e) => view! {
                        <ErrorBanner message=format!("Failed to load team data: {e}") />
                    }.into_any(),
                })}
            </Suspense>
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
                <h3 class="text-lg font-semibold text-primary">"Team Notebook"</h3>
                <button
                    class="text-muted hover:text-accent text-sm transition-colors cursor-pointer"
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
                    class="flex-1 bg-surface/50 border border-outline/50 rounded px-3 py-2 text-primary text-sm focus:outline-none focus:border-accent resize-none"
                />
                <button
                    class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-4 py-2 text-sm transition-colors cursor-pointer self-end"
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
                                                    "bg-accent/10 border border-accent/30 rounded-lg p-4"
                                                } else {
                                                    "bg-surface border border-divider rounded-lg p-4"
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
                                                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 text-accent" fill="currentColor" viewBox="0 0 24 24">
                                                                            <path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z"/>
                                                                        </svg>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <span></span> }.into_any()
                                                                }}
                                                                <span class="text-dimmed text-xs ml-auto">
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
                                                                                    class="w-full bg-surface/50 border border-outline/50 rounded px-3 py-2 text-primary text-sm focus:outline-none focus:border-accent resize-none"
                                                                                />
                                                                                <div class="flex gap-2">
                                                                                    <button
                                                                                        class="bg-accent hover:bg-accent-hover text-accent-contrast text-xs font-bold rounded px-3 py-1 transition-colors cursor-pointer"
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
                                                                                        class="text-secondary hover:text-primary text-xs rounded px-3 py-1 transition-colors cursor-pointer"
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
                                                                    class="text-muted hover:text-accent text-xs transition-colors cursor-pointer"
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
                                                                            class="text-muted hover:text-accent text-xs transition-colors cursor-pointer"
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
                                                                            class="text-muted hover:text-red-400 text-xs transition-colors cursor-pointer"
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
                            <p class="text-red-400 text-sm">{format!("Could not load notes: {e}")}</p>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
