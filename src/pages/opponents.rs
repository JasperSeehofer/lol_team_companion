use crate::models::opponent::{Opponent, OpponentPlayer};
use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn get_opponents() -> Result<Vec<Opponent>, ServerFnError> {
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

    db::list_opponents(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_opponent(name: String) -> Result<String, ServerFnError> {
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

    db::create_opponent(&surreal, &team_id, name, None)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_opponent_detail(
    id: String,
) -> Result<Option<(Opponent, Vec<OpponentPlayer>)>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let _auth: AuthSession = leptos_axum::extract().await?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::get_opponent(&surreal, &id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn save_opponent(id: String, name: String, notes: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let _auth: AuthSession = leptos_axum::extract().await?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let notes_opt = if notes.is_empty() { None } else { Some(notes) };
    db::update_opponent(&surreal, &id, name, notes_opt)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_opponent_action(id: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let _auth: AuthSession = leptos_axum::extract().await?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::delete_opponent(&surreal, &id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn add_player(opponent_id: String, name: String, role: String) -> Result<String, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let _auth: AuthSession = leptos_axum::extract().await?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::add_opponent_player(&surreal, &opponent_id, name, role)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn save_player(
    id: String,
    name: String,
    role: String,
    riot_summoner_name: String,
    notes: String,
) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let _auth: AuthSession = leptos_axum::extract().await?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let summoner_opt = if riot_summoner_name.is_empty() {
        None
    } else {
        Some(riot_summoner_name)
    };
    let notes_opt = if notes.is_empty() { None } else { Some(notes) };

    db::save_opponent_player_info(
        &surreal,
        &id,
        name,
        role,
        summoner_opt,
        notes_opt,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn remove_player(id: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let _auth: AuthSession = leptos_axum::extract().await?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::delete_opponent_player(&surreal, &id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn fetch_champions(
    player_id: String,
    summoner_name: String,
) -> Result<Vec<String>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use crate::server::riot;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let _auth: AuthSession = leptos_axum::extract().await?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    if !riot::has_api_key() {
        return Err(ServerFnError::new("No Riot API key configured"));
    }

    // Parse summoner name: "Name#Tag" format
    let parts: Vec<&str> = summoner_name.split('#').collect();
    if parts.len() != 2 {
        return Err(ServerFnError::new(
            "Summoner name must be in Name#Tag format",
        ));
    }

    let puuid = riot::get_puuid(parts[0], parts[1])
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let champions = riot::fetch_player_champions(&puuid, 20)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Update the player record with champions
    db::update_opponent_player_champions(&surreal, &player_id, champions.clone())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(champions)
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn OpponentsPage() -> impl IntoView {
    #[allow(unused_variables)]
    let user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());

    // Auth redirect
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let Some(Ok(None)) = user.get() {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    let opponents = Resource::new(|| (), |_| get_opponents());
    let selected_id: RwSignal<Option<String>> = RwSignal::new(None);
    let error_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let status_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let new_name: RwSignal<String> = RwSignal::new(String::new());

    // Detail resource: refetch when selected_id changes
    let detail = Resource::new(
        move || selected_id.get(),
        move |id| async move {
            match id {
                Some(id) => get_opponent_detail(id).await,
                None => Ok(None),
            }
        },
    );

    let do_create = move || {
        let name = new_name.get_untracked();
        if name.trim().is_empty() {
            return;
        }
        new_name.set(String::new());
        leptos::task::spawn_local(async move {
            match create_opponent(name).await {
                Ok(id) => {
                    selected_id.set(Some(id));
                    opponents.refetch();
                }
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        });
    };

    let on_delete = Callback::new(move |id: String| {
        leptos::task::spawn_local(async move {
            match delete_opponent_action(id).await {
                Ok(()) => {
                    selected_id.set(None);
                    opponents.refetch();
                    status_msg.set(Some("Opponent deleted".into()));
                }
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        });
    });

    view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 py-8">
            {move || error_msg.get().map(|msg| view! {
                <div class="bg-red-500/10 border border-red-500/30 rounded-xl p-4 mb-4 flex items-start gap-3">
                    <p class="text-red-400 text-sm">{msg}</p>
                    <button
                        on:click=move |_| error_msg.set(None)
                        class="text-red-400 hover:text-red-300 ml-auto cursor-pointer"
                    >"x"</button>
                </div>
            })}
            {move || status_msg.get().map(|msg| view! {
                <div class="bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 rounded-xl px-4 py-3 text-sm mb-4 flex items-center justify-between">
                    <span>{msg}</span>
                    <button
                        on:click=move |_| status_msg.set(None)
                        class="text-emerald-400 hover:text-emerald-300 ml-2 cursor-pointer"
                    >"x"</button>
                </div>
            })}

            <div class="flex items-center justify-between mb-6">
                <h1 class="text-2xl font-bold text-primary">"Opponents"</h1>
                <div class="flex gap-2">
                    <input
                        type="text"
                        placeholder="New opponent name..."
                        prop:value=move || new_name.get()
                        on:input=move |ev| new_name.set(event_target_value(&ev))
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" {
                                do_create();
                            }
                        }
                        class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-1.5 text-sm text-primary placeholder:text-muted focus:outline-none focus:border-accent"
                    />
                    <button
                        on:click=move |_| do_create()
                        class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold px-4 py-1.5 rounded-lg text-sm transition-colors cursor-pointer"
                    >
                        "+ New Opponent"
                    </button>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Left column: opponent list
                <div class="lg:col-span-1">
                    <div class="bg-surface rounded-xl border border-divider overflow-hidden">
                        <div class="px-4 py-3 border-b border-divider">
                            <h2 class="text-sm font-semibold text-secondary">"All Opponents"</h2>
                        </div>
                        <Suspense fallback=move || view! { <div class="p-4 text-muted text-sm">"Loading..."</div> }>
                            {move || Suspend::new(async move {
                                match opponents.await {
                                    Ok(list) => {
                                        if list.is_empty() {
                                            view! {
                                                <div class="p-4 text-muted text-sm">"No opponents yet. Create one to get started."</div>
                                            }.into_any()
                                        } else {
                                            let items = list.into_iter().map(|opp| {
                                                let opp_id = opp.id.clone().unwrap_or_default();
                                                let opp_id_click = opp_id.clone();
                                                let opp_name = opp.name.clone();
                                                view! {
                                                    <button
                                                        on:click=move |_| selected_id.set(Some(opp_id_click.clone()))
                                                        class=move || {
                                                            let base = "w-full text-left px-4 py-3 border-b border-divider/50 hover:bg-elevated transition-colors cursor-pointer";
                                                            let is_sel = selected_id.get().as_deref() == Some(opp_id.as_str());
                                                            if is_sel {
                                                                format!("{base} bg-elevated")
                                                            } else {
                                                                base.to_string()
                                                            }
                                                        }
                                                    >
                                                        <span class="text-primary text-sm font-medium">{opp_name}</span>
                                                    </button>
                                                }
                                            }).collect_view();
                                            view! { <div>{items}</div> }.into_any()
                                        }
                                    }
                                    Err(e) => view! {
                                        <div class="p-4 text-red-400 text-sm">{e.to_string()}</div>
                                    }.into_any(),
                                }
                            })}
                        </Suspense>
                    </div>
                </div>

                // Right column: detail panel
                <div class="lg:col-span-2">
                    <Suspense fallback=move || view! { <div class="text-muted text-sm">"Loading..."</div> }>
                        {move || Suspend::new(async move {
                            match detail.await {
                                Ok(Some((opp, players))) => {
                                    view! {
                                        <OpponentDetail
                                            opponent=opp
                                            players=players
                                            on_save_done=move || {
                                                opponents.refetch();
                                                detail.refetch();
                                                status_msg.set(Some("Saved".into()));
                                            }
                                            on_delete=on_delete
                                            on_player_change=move || detail.refetch()
                                            error_msg=error_msg
                                        />
                                    }.into_any()
                                }
                                Ok(None) => {
                                    view! {
                                        <div class="bg-surface rounded-xl border border-divider p-8 text-center text-muted">
                                            "Select an opponent from the list or create a new one."
                                        </div>
                                    }.into_any()
                                }
                                Err(e) => view! {
                                    <div class="text-red-400 text-sm">{e.to_string()}</div>
                                }.into_any(),
                            }
                        })}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}

#[component]
fn OpponentDetail(
    opponent: Opponent,
    players: Vec<OpponentPlayer>,
    on_save_done: impl Fn() + Copy + Send + Sync + 'static,
    on_delete: Callback<String>,
    on_player_change: impl Fn() + Copy + Send + Sync + 'static,
    error_msg: RwSignal<Option<String>>,
) -> impl IntoView {
    let opp_id = opponent.id.clone().unwrap_or_default();
    let opp_id_save = opp_id.clone();
    let opp_id_delete = opp_id.clone();
    let opp_id_add = opp_id.clone();

    let name = RwSignal::new(opponent.name.clone());
    let notes = RwSignal::new(opponent.notes.clone().unwrap_or_default());
    let confirm_delete = RwSignal::new(false);
    let add_player_name = RwSignal::new(String::new());
    let add_player_role = RwSignal::new("top".to_string());

    let on_save = move |_| {
        let id = opp_id_save.clone();
        let n = name.get_untracked();
        let nt = notes.get_untracked();
        leptos::task::spawn_local(async move {
            match save_opponent(id, n, nt).await {
                Ok(()) => on_save_done(),
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        });
    };

    let on_add_player = move |_| {
        let p_name = add_player_name.get_untracked();
        let p_role = add_player_role.get_untracked();
        if p_name.trim().is_empty() {
            return;
        }
        let oid = opp_id_add.clone();
        add_player_name.set(String::new());
        leptos::task::spawn_local(async move {
            match add_player(oid, p_name, p_role).await {
                Ok(_) => on_player_change(),
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        });
    };

    let roles = vec!["top", "jungle", "mid", "bot", "support"];

    view! {
        <div class="bg-surface rounded-xl border border-divider">
            // Header
            <div class="px-6 py-4 border-b border-divider flex items-center justify-between">
                <h2 class="text-lg font-semibold text-primary">"Opponent Details"</h2>
                <div class="flex gap-2">
                    <button
                        on:click=on_save
                        class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold px-4 py-1.5 rounded-lg text-sm transition-colors cursor-pointer"
                    >
                        "Save"
                    </button>
                    {move || {
                        let id = opp_id_delete.clone();
                        if confirm_delete.get() {
                            view! {
                                <div class="flex gap-1">
                                    <button
                                        on:click=move |_| {
                                            on_delete.run(id.clone());
                                            confirm_delete.set(false);
                                        }
                                        class="bg-red-700 hover:bg-red-600 text-white font-semibold px-3 py-1.5 rounded-lg text-sm transition-colors cursor-pointer"
                                    >
                                        "Confirm"
                                    </button>
                                    <button
                                        on:click=move |_| confirm_delete.set(false)
                                        class="bg-elevated hover:bg-overlay-strong text-secondary px-3 py-1.5 rounded-lg text-sm transition-colors cursor-pointer"
                                    >
                                        "Cancel"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <button
                                    on:click=move |_| confirm_delete.set(true)
                                    class="bg-red-700/20 hover:bg-red-700 text-red-400 hover:text-white font-semibold px-4 py-1.5 rounded-lg text-sm transition-colors cursor-pointer"
                                >
                                    "Delete"
                                </button>
                            }.into_any()
                        }
                    }}
                </div>
            </div>

            // Name and notes
            <div class="px-6 py-4 space-y-4 border-b border-divider">
                <div>
                    <label class="block text-xs font-medium text-muted mb-1">"Name"</label>
                    <input
                        type="text"
                        prop:value=move || name.get()
                        on:input=move |ev| name.set(event_target_value(&ev))
                        class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-sm text-primary placeholder:text-muted focus:outline-none focus:border-accent"
                    />
                </div>
                <div>
                    <label class="block text-xs font-medium text-muted mb-1">"Notes"</label>
                    <textarea
                        prop:value=move || notes.get()
                        on:input=move |ev| notes.set(event_target_value(&ev))
                        rows="3"
                        class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-sm text-primary placeholder:text-muted focus:outline-none focus:border-accent resize-y"
                        placeholder="General scouting notes..."
                    />
                </div>
            </div>

            // Players section
            <div class="px-6 py-4">
                <div class="flex items-center justify-between mb-4">
                    <h3 class="text-sm font-semibold text-secondary">"Players"</h3>
                </div>

                // Add player form
                <div class="flex gap-2 mb-4">
                    <input
                        type="text"
                        placeholder="Player name..."
                        prop:value=move || add_player_name.get()
                        on:input=move |ev| add_player_name.set(event_target_value(&ev))
                        class="flex-1 bg-surface/50 border border-outline/50 rounded-lg px-3 py-1.5 text-sm text-primary placeholder:text-muted focus:outline-none focus:border-accent"
                    />
                    <select
                        prop:value=move || add_player_role.get()
                        on:change=move |ev| add_player_role.set(event_target_value(&ev))
                        class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-1.5 text-sm text-primary focus:outline-none focus:border-accent"
                    >
                        {roles.iter().map(|r| {
                            let r_val = r.to_string();
                            let r_label = match *r {
                                "top" => "Top",
                                "jungle" => "Jungle",
                                "mid" => "Mid",
                                "bot" => "Bot",
                                "support" => "Support",
                                _ => r,
                            };
                            view! { <option value=r_val>{r_label}</option> }
                        }).collect_view()}
                    </select>
                    <button
                        on:click=on_add_player
                        class="bg-elevated hover:bg-overlay-strong text-secondary hover:text-primary font-medium px-3 py-1.5 rounded-lg text-sm transition-colors cursor-pointer"
                    >
                        "+ Add"
                    </button>
                </div>

                // Player list
                <div class="space-y-3">
                    {players.into_iter().map(|player| {
                        view! {
                            <PlayerCard
                                player=player
                                on_change=on_player_change
                                error_msg=error_msg
                            />
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn PlayerCard(
    player: OpponentPlayer,
    on_change: impl Fn() + Copy + Send + Sync + 'static,
    error_msg: RwSignal<Option<String>>,
) -> impl IntoView {
    let player_id = player.id.clone().unwrap_or_default();
    let player_id_save = player_id.clone();
    let player_id_remove = player_id.clone();
    let player_id_fetch = player_id.clone();

    let p_name = RwSignal::new(player.name.clone());
    let p_role = RwSignal::new(player.role.clone());
    let p_summoner = RwSignal::new(player.riot_summoner_name.clone().unwrap_or_default());
    let p_notes = RwSignal::new(player.notes.clone().unwrap_or_default());
    let p_champions = RwSignal::new(player.recent_champions.clone());
    let fetching = RwSignal::new(false);

    let role_label = match player.role.as_str() {
        "top" => "Top",
        "jungle" => "Jungle",
        "mid" => "Mid",
        "bot" => "Bot",
        "support" => "Support",
        _ => &player.role,
    };
    let role_label = role_label.to_string();

    let on_save_player = move |_| {
        let id = player_id_save.clone();
        let n = p_name.get_untracked();
        let r = p_role.get_untracked();
        let s = p_summoner.get_untracked();
        let nt = p_notes.get_untracked();
        leptos::task::spawn_local(async move {
            match save_player(id, n, r, s, nt).await {
                Ok(()) => on_change(),
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        });
    };

    let on_remove = move |_| {
        let id = player_id_remove.clone();
        leptos::task::spawn_local(async move {
            match remove_player(id).await {
                Ok(()) => on_change(),
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        });
    };

    let on_fetch = move |_| {
        let id = player_id_fetch.clone();
        let summoner = p_summoner.get_untracked();
        if summoner.trim().is_empty() {
            error_msg.set(Some("Enter a summoner name (Name#Tag) first".into()));
            return;
        }
        fetching.set(true);
        leptos::task::spawn_local(async move {
            match fetch_champions(id, summoner).await {
                Ok(champs) => {
                    p_champions.set(champs);
                    fetching.set(false);
                    on_change();
                }
                Err(e) => {
                    fetching.set(false);
                    error_msg.set(Some(e.to_string()));
                }
            }
        });
    };

    let roles = vec!["top", "jungle", "mid", "bot", "support"];

    view! {
        <div class="bg-elevated rounded-lg border border-divider/50 p-4">
            <div class="flex items-center justify-between mb-3">
                <div class="flex items-center gap-2">
                    <span class="text-xs font-bold uppercase text-accent bg-accent/10 px-2 py-0.5 rounded">
                        {role_label}
                    </span>
                </div>
                <div class="flex gap-1.5">
                    <button
                        on:click=on_save_player
                        class="text-xs text-secondary hover:text-primary bg-surface hover:bg-elevated px-2 py-1 rounded transition-colors cursor-pointer"
                    >
                        "Save"
                    </button>
                    <button
                        on:click=on_remove
                        class="text-xs text-red-400 hover:text-red-300 bg-surface hover:bg-red-700/20 px-2 py-1 rounded transition-colors cursor-pointer"
                    >
                        "Remove"
                    </button>
                </div>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 mb-3">
                <div>
                    <label class="block text-xs text-muted mb-1">"Name"</label>
                    <input
                        type="text"
                        prop:value=move || p_name.get()
                        on:input=move |ev| p_name.set(event_target_value(&ev))
                        class="w-full bg-surface/50 border border-outline/50 rounded px-2 py-1 text-sm text-primary focus:outline-none focus:border-accent"
                    />
                </div>
                <div>
                    <label class="block text-xs text-muted mb-1">"Role"</label>
                    <select
                        prop:value=move || p_role.get()
                        on:change=move |ev| p_role.set(event_target_value(&ev))
                        class="w-full bg-surface/50 border border-outline/50 rounded px-2 py-1 text-sm text-primary focus:outline-none focus:border-accent"
                    >
                        {roles.iter().map(|r| {
                            let r_val = r.to_string();
                            let r_label = match *r {
                                "top" => "Top",
                                "jungle" => "Jungle",
                                "mid" => "Mid",
                                "bot" => "Bot",
                                "support" => "Support",
                                _ => r,
                            };
                            view! { <option value=r_val>{r_label}</option> }
                        }).collect_view()}
                    </select>
                </div>
            </div>

            <div class="mb-3">
                <label class="block text-xs text-muted mb-1">"Summoner Name (Name#Tag)"</label>
                <div class="flex gap-2">
                    <input
                        type="text"
                        placeholder="Faker#KR1"
                        prop:value=move || p_summoner.get()
                        on:input=move |ev| p_summoner.set(event_target_value(&ev))
                        class="flex-1 bg-surface/50 border border-outline/50 rounded px-2 py-1 text-sm text-primary placeholder:text-muted focus:outline-none focus:border-accent"
                    />
                    <button
                        on:click=on_fetch
                        disabled=move || fetching.get()
                        class="bg-blue-700 hover:bg-blue-600 disabled:opacity-50 text-white text-xs font-medium px-3 py-1 rounded transition-colors cursor-pointer"
                    >
                        {move || if fetching.get() { "Fetching..." } else { "Fetch Champs" }}
                    </button>
                </div>
            </div>

            // Recent champions
            {move || {
                let champs = p_champions.get();
                if champs.is_empty() {
                    view! {
                        <div class="text-xs text-muted italic">"No champions fetched yet"</div>
                    }.into_any()
                } else {
                    view! {
                        <div class="flex flex-wrap gap-1.5 mb-3">
                            {champs.into_iter().map(|c| {
                                view! {
                                    <span class="text-xs bg-surface border border-divider/50 text-secondary rounded px-2 py-0.5">
                                        {c}
                                    </span>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }
            }}

            <div>
                <label class="block text-xs text-muted mb-1">"Notes"</label>
                <textarea
                    prop:value=move || p_notes.get()
                    on:input=move |ev| p_notes.set(event_target_value(&ev))
                    rows="2"
                    class="w-full bg-surface/50 border border-outline/50 rounded px-2 py-1 text-sm text-primary placeholder:text-muted focus:outline-none focus:border-accent resize-y"
                    placeholder="Player tendencies, comfort picks..."
                />
            </div>
        </div>
    }
}
