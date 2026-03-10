use leptos::prelude::*;
use crate::models::team::Team;
use crate::models::user::TeamMember;

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

const MEMBER_ROLES: &[&str] = &["top", "jungle", "mid", "bot", "support", "sub"];

#[component]
pub fn TeamDashboard() -> impl IntoView {
    let dashboard = Resource::new(|| (), |_| get_team_dashboard());

    view! {
        <div class="max-w-4xl mx-auto py-8 px-6">
            <h1 class="text-3xl font-bold text-white mb-6">"Team Dashboard"</h1>
            <Suspense fallback=|| view! { <div class="text-gray-400">"Loading..."</div> }>
                {move || dashboard.get().map(|result| match result {
                    Ok(Some((team, members, current_user_id))) => {
                        let is_leader = team.created_by == current_user_id;
                        // Edit-team state (only used by leader)
                        let (edit_name, set_edit_name) = signal(team.name.clone());
                        let (edit_region, set_edit_region) = signal(team.region.clone());
                        let (edit_msg, set_edit_msg) = signal(Option::<String>::None);

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

                                // Roster
                                <div>
                                    <h3 class="text-lg font-semibold text-white mb-3">"Roster"</h3>
                                    <div class="flex flex-col gap-2">
                                        {members.into_iter().map(|m| {
                                            let member_id = m.user_id.clone();
                                            let member_id_for_kick = member_id.clone();
                                            let is_self = member_id == current_user_id;
                                            let display_name = m.username.clone();
                                            let current_role = m.role.clone();
                                            let (role_msg, set_role_msg) = signal(Option::<String>::None);

                                            view! {
                                                <div class="bg-gray-800 border border-gray-700 rounded px-4 py-3 flex items-center justify-between gap-3">
                                                    <div class="flex items-center gap-2 min-w-0">
                                                        <span class="text-white font-medium truncate">{display_name}</span>
                                                        {m.riot_summoner_name.map(|n| view! {
                                                            <span class="text-gray-500 text-sm truncate">{n}</span>
                                                        })}
                                                        {move || role_msg.get().map(|msg| view! {
                                                            <span class="text-xs text-green-400">{msg}</span>
                                                        })}
                                                    </div>
                                                    <div class="flex items-center gap-2 flex-shrink-0">
                                                        // Leader: role dropdown per member
                                                        {if is_leader {
                                                            let mid = member_id.clone();
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

                                                        // Leader can kick non-self members
                                                        {if is_leader && !is_self {
                                                            view! {
                                                                <button
                                                                    class="text-gray-600 hover:text-red-400 text-sm transition-colors"
                                                                    title="Remove from team"
                                                                    on:click=move |_| {
                                                                        let uid = member_id_for_kick.clone();
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
