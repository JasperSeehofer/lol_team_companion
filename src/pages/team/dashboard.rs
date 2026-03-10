use leptos::prelude::*;
use crate::models::team::Team;
use crate::models::user::{JoinRequest, TeamMember};

/// Returns (team, members, current_user_id) so the client can check leadership.
#[server]
pub async fn get_team_dashboard() -> Result<Option<(Team, Vec<TeamMember>, String)>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let user_id = user.id.clone();
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    match db::get_user_team_with_members(&db, &user.id).await.map_err(|e| ServerFnError::new(e.to_string()))? {
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
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let (team, _) = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    if team.created_by != user.id {
        return Err(ServerFnError::new("Only the team leader can edit team details"));
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
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

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
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let (team, _) = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    if team.created_by != user.id {
        return Err(ServerFnError::new("Only the team leader can remove members"));
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
pub async fn get_pending_requests() -> Result<Vec<JoinRequest>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

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
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let (team, _) = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    if team.created_by != user.id {
        return Err(ServerFnError::new("Only the team leader can respond to requests"));
    }

    let team_id = team.id.ok_or_else(|| ServerFnError::new("No team id"))?;
    db::respond_to_join_request(&db, &request_id, accept, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn assign_member_to_slot(member_user_id: String, role: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

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
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let (team, _) = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    if team.created_by != user.id {
        return Err(ServerFnError::new("Only the team leader can unassign slots"));
    }

    let team_id = team.id.ok_or_else(|| ServerFnError::new("No team id"))?;
    db::remove_from_slot(&db, &team_id, &member_user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

const MEMBER_ROLES: &[&str] = &["top", "jungle", "mid", "bot", "support", "unassigned"];
const STARTER_ROLES: &[&str] = &["top", "jungle", "mid", "bot", "support"];

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
    let dashboard = Resource::new(|| (), |_| get_team_dashboard());
    let requests = Resource::new(|| (), |_| get_pending_requests());

    view! {
        <div class="max-w-4xl mx-auto py-8 px-6">
            <h1 class="text-3xl font-bold text-white mb-6">"Team Dashboard"</h1>
            <Suspense fallback=|| view! { <div class="text-gray-400">"Loading..."</div> }>
                {move || dashboard.get().map(|result| match result {
                    Ok(Some((team, members, current_user_id))) => {
                        let is_leader = team.created_by == current_user_id;
                        let (edit_name, set_edit_name) = signal(team.name.clone());
                        let (edit_region, set_edit_region) = signal(team.region.clone());
                        let (edit_msg, set_edit_msg) = signal(Option::<String>::None);

                        // Partition members
                        let starters: Vec<TeamMember> = members.iter()
                            .filter(|m| m.roster_type == "starter")
                            .cloned()
                            .collect();
                        let subs: Vec<TeamMember> = members.iter()
                            .filter(|m| m.roster_type != "starter")
                            .cloned()
                            .collect();

                        view! {
                            <div class="flex flex-col gap-6">
                                // Team info card
                                <div class="bg-gray-800 border border-gray-700 rounded-lg p-6">
                                    <div class="flex items-start justify-between gap-4">
                                        <div>
                                            <h2 class="text-xl font-bold text-yellow-400 mb-1">{team.name.clone()}</h2>
                                            <p class="text-gray-400 text-sm">"Region: " {team.region.clone()}</p>
                                            {if is_leader {
                                                view! { <span class="inline-block mt-1 text-xs text-yellow-400 font-medium bg-yellow-400/10 rounded px-1.5 py-0.5">"Team Leader"</span> }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }}
                                        </div>
                                    </div>

                                    // Leader: edit team details
                                    {if is_leader {
                                        view! {
                                            <div class="mt-4 pt-4 border-t border-gray-700">
                                                <h3 class="text-gray-300 text-sm font-medium mb-3">"Edit Team"</h3>
                                                {move || edit_msg.get().map(|m| {
                                                    let is_err = m.starts_with("Error");
                                                    let cls = if is_err {
                                                        "bg-red-900 border border-red-700 text-red-200 rounded px-3 py-2 text-sm mb-3"
                                                    } else {
                                                        "bg-green-900 border border-green-700 text-green-200 rounded px-3 py-2 text-sm mb-3"
                                                    };
                                                    view! { <div class=cls>{m}</div> }
                                                })}
                                                <div class="flex gap-3 items-end">
                                                    <div class="flex-1">
                                                        <label class="block text-gray-400 text-xs mb-1">"Team Name"</label>
                                                        <input
                                                            type="text"
                                                            prop:value=move || edit_name.get()
                                                            on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                                            class="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white text-sm focus:outline-none focus:border-yellow-400"
                                                        />
                                                    </div>
                                                    <div>
                                                        <label class="block text-gray-400 text-xs mb-1">"Region"</label>
                                                        <select
                                                            prop:value=move || edit_region.get()
                                                            on:change=move |ev| set_edit_region.set(event_target_value(&ev))
                                                            class="bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white text-sm focus:outline-none focus:border-yellow-400"
                                                        >
                                                            {["EUW","EUNE","NA","KR","BR"].iter().map(|&r| view! {
                                                                <option value=r>{r}</option>
                                                            }).collect_view()}
                                                        </select>
                                                    </div>
                                                    <button
                                                        class="bg-yellow-400 hover:bg-yellow-300 text-gray-900 font-bold rounded px-4 py-2 text-sm transition-colors"
                                                        on:click=move |_| {
                                                            let name = edit_name.get_untracked();
                                                            let region = edit_region.get_untracked();
                                                            leptos::task::spawn_local(async move {
                                                                match update_team_info(name, region).await {
                                                                    Ok(_) => {
                                                                        set_edit_msg.set(Some("Saved!".into()));
                                                                        dashboard.refetch();
                                                                    }
                                                                    Err(e) => set_edit_msg.set(Some(format!("Error: {e}"))),
                                                                }
                                                            });
                                                        }
                                                    >
                                                        "Save"
                                                    </button>
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>

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
                                                            <div class="bg-gray-800 border border-yellow-400/30 rounded-lg p-5">
                                                                <h3 class="text-yellow-400 font-semibold mb-3 flex items-center gap-2">
                                                                    "Join Requests"
                                                                    <span class="bg-yellow-400 text-gray-900 text-xs font-bold rounded-full w-5 h-5 flex items-center justify-center">
                                                                        {reqs.len()}
                                                                    </span>
                                                                </h3>
                                                                <div class="flex flex-col gap-2">
                                                                    {reqs.into_iter().map(|req| {
                                                                        let req_id_accept = req.id.clone();
                                                                        let req_id_decline = req.id.clone();
                                                                        view! {
                                                                            <div class="flex items-center justify-between bg-gray-700 rounded px-4 py-3">
                                                                                <div>
                                                                                    <span class="text-white font-medium">{req.username}</span>
                                                                                    {req.riot_summoner_name.map(|n| view! {
                                                                                        <span class="text-gray-400 text-sm ml-2">{n}</span>
                                                                                    })}
                                                                                </div>
                                                                                <div class="flex gap-2">
                                                                                    <button
                                                                                        class="bg-green-700 hover:bg-green-600 text-white text-sm font-medium rounded px-3 py-1.5 transition-colors"
                                                                                        on:click=move |_| {
                                                                                            let id = req_id_accept.clone();
                                                                                            leptos::task::spawn_local(async move {
                                                                                                let _ = handle_join_request(id, true).await;
                                                                                                dashboard.refetch();
                                                                                                requests.refetch();
                                                                                            });
                                                                                        }
                                                                                    >"Accept"</button>
                                                                                    <button
                                                                                        class="bg-gray-600 hover:bg-red-700 text-gray-300 hover:text-white text-sm font-medium rounded px-3 py-1.5 transition-colors"
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
                                    <h3 class="text-lg font-semibold text-white mb-3">"Starting Roster"</h3>
                                    <div class="grid grid-cols-5 gap-3">
                                        {STARTER_ROLES.iter().map(|&role| {
                                            let assigned = starters.iter().find(|m| m.role == role).cloned();
                                            let role_label = role.to_string();
                                            let role_label2 = role_label.clone();
                                            let (drag_over, set_drag_over) = signal(false);

                                            view! {
                                                <div
                                                    class=move || format!(
                                                        "bg-gray-800 border rounded-lg p-3 flex flex-col items-center gap-2 min-h-[120px] transition-colors {}",
                                                        if drag_over.get() { "border-yellow-400 bg-gray-700" } else { "border-gray-700" }
                                                    )
                                                    on:dragover=move |ev| {
                                                        ev.prevent_default();
                                                        set_drag_over.set(true);
                                                    }
                                                    on:dragleave=move |_| set_drag_over.set(false)
                                                    on:drop=move |ev| {
                                                        ev.prevent_default();
                                                        set_drag_over.set(false);
                                                        let dt = ev.data_transfer().unwrap();
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
                                                    // Role icon + label
                                                    <div class="flex items-center gap-1.5">
                                                        {if !role_icon_url(role).is_empty() {
                                                            view! {
                                                                <img src=role_icon_url(role) alt=role class="w-4 h-4 invert opacity-60" />
                                                            }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        <span class="text-gray-400 text-xs capitalize">{role_label2.clone()}</span>
                                                    </div>

                                                    // Assigned player or empty slot
                                                    {if let Some(m) = assigned {
                                                        let uid_for_unassign = m.user_id.clone();
                                                        view! {
                                                            <div class="flex-1 flex flex-col items-center justify-center gap-1 w-full">
                                                                <span class="text-white text-sm font-medium text-center truncate w-full text-center">{m.username.clone()}</span>
                                                                {if is_leader {
                                                                    view! {
                                                                        <button
                                                                            class="text-gray-600 hover:text-red-400 text-xs transition-colors"
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
                                                                <span class="text-gray-600 text-xs">"Empty"</span>
                                                            </div>
                                                        }.into_any()
                                                    }}
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                    {if is_leader {
                                        view! { <p class="text-gray-500 text-xs mt-2">"Drag players from the bench below to assign them to role slots."</p> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>

                                // Substitute bench
                                <div>
                                    <h3 class="text-lg font-semibold text-white mb-3">"Bench / Substitutes"</h3>
                                    {if subs.is_empty() {
                                        view! { <p class="text-gray-500 text-sm">"No players on the bench."</p> }.into_any()
                                    } else {
                                        view! {
                                            <div class="flex flex-col gap-2">
                                                {subs.into_iter().map(|m| {
                                                    let uid_drag = m.user_id.clone();
                                                    let uid_kick = m.user_id.clone();
                                                    let is_self = m.user_id == current_user_id;
                                                    let current_role = m.role.clone();
                                                    let display_name = m.username.clone();
                                                    let (role_msg, set_role_msg) = signal(Option::<String>::None);

                                                    view! {
                                                        <div
                                                            class="bg-gray-800 border border-gray-700 rounded px-4 py-3 flex items-center justify-between gap-3 cursor-grab active:cursor-grabbing"
                                                            draggable="true"
                                                            on:dragstart=move |ev| {
                                                                let dt = ev.data_transfer().unwrap();
                                                                dt.set_data("text/plain", &uid_drag).unwrap();
                                                            }
                                                        >
                                                            <div class="flex items-center gap-2 min-w-0">
                                                                <span class="text-gray-400 text-xs select-none" title="Drag to assign to a role slot">"⠿"</span>
                                                                <span class="text-white font-medium truncate">{display_name}</span>
                                                                {m.riot_summoner_name.map(|n| view! {
                                                                    <span class="text-gray-500 text-sm truncate">{n}</span>
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
                                                                            class="bg-gray-700 border border-gray-600 rounded px-2 py-1 text-gray-200 text-sm focus:outline-none focus:border-yellow-400"
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
                                                                        <span class="text-gray-400 text-sm capitalize">{current_role}</span>
                                                                    }.into_any()
                                                                }}

                                                                {if is_leader && !is_self {
                                                                    view! {
                                                                        <button
                                                                            class="text-gray-600 hover:text-red-400 text-sm transition-colors"
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
                            </div>
                        }.into_any()
                    },
                    Ok(None) => view! {
                        <div class="text-center py-16">
                            <p class="text-gray-400 mb-4">"You are not part of a team yet."</p>
                            <a href="/team/roster" class="bg-yellow-400 hover:bg-yellow-300 text-gray-900 font-bold rounded px-4 py-2">
                                "Create or Join a Team"
                            </a>
                        </div>
                    }.into_any(),
                    Err(e) => view! {
                        <div class="text-red-400">"Error: " {e.to_string()}</div>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
