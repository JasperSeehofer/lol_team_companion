use crate::components::ui::{EmptyState, NoTeamState, SkeletonCard, ToastContext, ToastKind};
use crate::models::opponent::{is_stale, Opponent, OpponentPlayer};
use crate::models::utils::format_timestamp;
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

    let platform = riot::platform_route_from_str("EUW");
    let puuid = riot::get_puuid(parts[0], parts[1], platform)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let champions = riot::fetch_player_champions(&puuid, 20, platform)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Update the player record with champions
    db::update_opponent_player_champions(&surreal, &player_id, champions.clone())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(champions)
}

/// Create an opponent with an initial set of players in a single server call.
///
/// Returns (opponent_id, player_ids) so the UI can immediately trigger per-player intel fetches
/// without an extra round-trip to get the newly created player IDs.
///
/// `players_json`: JSON-encoded `Vec<(String, Option<String>)>` where each tuple is
/// `(role, riot_summoner_name_or_none)`.
#[server]
pub async fn create_opponent_with_players_fn(
    name: String,
    players_json: String,
) -> Result<(String, Vec<String>), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Err(ServerFnError::new("No team")),
    };

    let players: Vec<(String, Option<String>)> = serde_json::from_str(&players_json)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let (opponent_id, player_ids) =
        db::create_opponent_with_players(&surreal, &team_id, name, None, players)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok((opponent_id, player_ids))
}

/// Fetch combined Riot API intel for a player (champions, per-match roles, mastery) and persist it.
///
/// Computes role distribution from per-match `team_position` data and stores as JSON.
/// Returns `Ok(())` immediately if `riot_summoner_name` is empty — no error for optional field.
#[server]
pub async fn fetch_player_intel_fn(
    player_id: String,
    riot_summoner_name: String,
) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use crate::server::riot;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    // Empty summoner name is not an error — skip gracefully
    if riot_summoner_name.trim().is_empty() {
        return Ok(());
    }

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    if !riot::has_api_key() {
        return Err(ServerFnError::new("No Riot API key configured"));
    }

    // Parse "GameName#TagLine" format
    let parts: Vec<&str> = riot_summoner_name.split('#').collect();
    if parts.len() != 2 {
        return Err(ServerFnError::new(
            "Summoner name must be in Name#Tag format",
        ));
    }

    let platform = riot::platform_route_from_str("EUW");
    let puuid = riot::get_puuid(parts[0], parts[1], platform)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let intel = riot::fetch_player_intel(&puuid, 20, platform)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Compute role distribution: count occurrences of each team_position
    let mut role_counts: std::collections::HashMap<String, u32> =
        std::collections::HashMap::new();
    for (_champion, position) in &intel.champion_with_role {
        if !position.is_empty() {
            *role_counts.entry(position.clone()).or_insert(0) += 1;
        }
    }
    let role_dist: Vec<(String, u32)> = role_counts.into_iter().collect();

    let mastery_json = serde_json::to_string(&intel.mastery_data).unwrap_or_default();
    let role_dist_json = serde_json::to_string(&role_dist).unwrap_or_default();

    db::update_opponent_player_intel(
        &surreal,
        &player_id,
        Some(puuid),
        intel.recent_champions,
        Some(mastery_json),
        Some(role_dist_json),
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
enum FetchState {
    Idle,
    Fetching,
    Success,
    Error(String),
}

const ROLES: [(&str, &str); 5] = [
    ("top", "Top"),
    ("jungle", "Jungle"),
    ("mid", "Mid"),
    ("bot", "Bot"),
    ("support", "Support"),
];

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[component]
pub fn OpponentsPage() -> impl IntoView {
    let user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    let is_solo_mode: RwSignal<bool> = RwSignal::new(false);

    // Auth redirect + mode detection
    Effect::new(move || {
        match user.get() {
            Some(Ok(None)) => {
                #[cfg(feature = "hydrate")]
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/auth/login");
                }
            }
            Some(Ok(Some(u))) => {
                is_solo_mode.set(u.mode == "solo");
            }
            _ => {}
        }
    });

    let toast = use_context::<ToastContext>().expect("ToastProvider");

    // return_to query param: when navigated from draft page
    let params = leptos_router::hooks::use_query_map();
    let return_to = move || params.read().get("return_to").unwrap_or_default();
    let return_draft_id = move || params.read().get("draft_id").unwrap_or_default();

    // Team check for NoTeamState
    let has_team = Resource::new(
        || (),
        |_| async { crate::pages::team::dashboard::get_team_dashboard().await.ok().flatten().is_some() },
    );

    let opponents = Resource::new(|| (), |_| get_opponents());
    let selected_id: RwSignal<Option<String>> = RwSignal::new(None);
    let creating: RwSignal<bool> = RwSignal::new(false);

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

    let on_delete = Callback::new(move |id: String| {
        leptos::task::spawn_local(async move {
            match delete_opponent_action(id).await {
                Ok(()) => {
                    selected_id.set(None);
                    opponents.refetch();
                    toast.show.run((ToastKind::Success, "Opponent deleted".into()));
                }
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        });
    });

    let gate_view = move || {
        if is_solo_mode.get() {
            Some(view! {
                <div class="max-w-7xl mx-auto px-4 sm:px-6 py-8">
                    <h1 class="text-2xl font-semibold text-primary mb-6">"Opponents"</h1>
                    <div class="max-w-2xl py-8 text-center">
                        <div class="bg-surface border border-outline rounded-xl p-6">
                            <h2 class="text-xl font-semibold text-primary mb-2">"Team feature"</h2>
                            <p class="text-secondary text-sm mb-4">"Switch to team mode to use this feature."</p>
                            <button
                                class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm cursor-pointer"
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
                </div>
            })
        } else {
            None
        }
    };

    view! {
        {gate_view}
        <div class="max-w-7xl mx-auto px-4 sm:px-6 py-8" style:display=move || if is_solo_mode.get() { "none" } else { "" }>

            // Back to Draft link (when navigated from draft page via Add New Opponent)
            {move || {
                if return_to() == "draft" {
                    let draft_id = return_draft_id();
                    let href = if draft_id.is_empty() {
                        "/draft".to_string()
                    } else {
                        format!("/draft?draft_id={}", draft_id)
                    };
                    view! {
                        <a href=href class="flex items-center gap-1 text-sm text-accent hover:underline mb-4">
                            <span>"\u{2190} Back to Draft"</span>
                        </a>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            <div class="flex items-center justify-between mb-6">
                <h1 class="text-2xl font-semibold text-primary">"Opponents"</h1>
                <button
                    on:click=move |_| {
                        selected_id.set(None);
                        creating.set(true);
                    }
                    class="bg-accent text-accent-contrast hover:bg-accent-hover rounded-lg px-4 py-2 text-sm font-medium cursor-pointer"
                >
                    "+ New Opponent"
                </button>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Left column: opponent list
                <div class="lg:col-span-1">
                    <div class="bg-surface rounded-xl border border-divider overflow-hidden">
                        <div class="px-4 py-3 border-b border-divider">
                            <h2 class="text-sm font-semibold text-secondary">"All Opponents"</h2>
                        </div>
                        <Suspense fallback=move || view! { <div class="p-4 flex flex-col gap-2"><SkeletonCard height="h-10" /><SkeletonCard height="h-10" /><SkeletonCard height="h-10" /></div> }>
                            {move || Suspend::new(async move {
                                match opponents.await {
                                    Ok(list) => {
                                        if list.is_empty() {
                                            let user_has_team = has_team.get().unwrap_or(false);
                                            if user_has_team {
                                                view! {
                                                    <EmptyState
                                                        icon="🎭"
                                                        message="No opponents scouted yet — add an opponent team to start tracking their picks and bans"
                                                        cta_label="Add Opponent"
                                                        cta_href="#add-opponent"
                                                    />
                                                }.into_any()
                                            } else {
                                                view! { <NoTeamState /> }.into_any()
                                            }
                                        } else {
                                            let items = list.into_iter().map(|opp| {
                                                let opp_id = opp.id.clone().unwrap_or_default();
                                                let opp_id_click = opp_id.clone();
                                                let opp_name = opp.name.clone();
                                                view! {
                                                    <button
                                                        on:click=move |_| {
                                                            creating.set(false);
                                                            selected_id.set(Some(opp_id_click.clone()));
                                                        }
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

                // Right column: creation form, detail panel, or placeholder
                <div class="lg:col-span-2">
                    {move || {
                        if creating.get() {
                            view! {
                                <CreationForm
                                    on_done=move |new_id: String| {
                                        creating.set(false);
                                        selected_id.set(Some(new_id));
                                        opponents.refetch();
                                    }
                                    on_discard=move || creating.set(false)
                                />
                            }.into_any()
                        } else {
                            view! {
                                <Suspense fallback=move || view! { <SkeletonCard height="h-64" /> }>
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
                                                            toast.show.run((ToastKind::Success, "Opponent updated".into()));
                                                        }
                                                        on_delete=on_delete
                                                        on_player_change=move || detail.refetch()
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
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// CreationForm component
// ---------------------------------------------------------------------------

#[component]
fn CreationForm(
    on_done: impl Fn(String) + Copy + Send + Sync + 'static,
    on_discard: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");

    let team_name: RwSignal<String> = RwSignal::new(String::new());
    let name_error: RwSignal<Option<String>> = RwSignal::new(None);
    let saving: RwSignal<bool> = RwSignal::new(false);

    // Per-role input signals: [top, jungle, mid, bot, support]
    let role_inputs: [RwSignal<String>; 5] = [
        RwSignal::new(String::new()),
        RwSignal::new(String::new()),
        RwSignal::new(String::new()),
        RwSignal::new(String::new()),
        RwSignal::new(String::new()),
    ];
    let validation_errors: [RwSignal<Option<String>>; 5] = [
        RwSignal::new(None),
        RwSignal::new(None),
        RwSignal::new(None),
        RwSignal::new(None),
        RwSignal::new(None),
    ];

    let on_save_fetch = move |_| {
        // Validate team name
        let name = team_name.get_untracked();
        if name.trim().is_empty() {
            name_error.set(Some("Team name is required".into()));
            return;
        }
        name_error.set(None);

        // Validate Riot ID fields
        let mut has_error = false;
        let mut inputs_vals: Vec<String> = Vec::new();
        for i in 0..5 {
            let val = role_inputs[i].get_untracked();
            if !val.trim().is_empty() && !val.contains('#') {
                validation_errors[i].set(Some("Use Name#Tag format (e.g. Faker#KR1)".into()));
                has_error = true;
            } else {
                validation_errors[i].set(None);
            }
            inputs_vals.push(val);
        }
        if has_error {
            return;
        }

        // Build players_json: Vec<(role, Option<riot_id>)>
        let players: Vec<(String, Option<String>)> = ROLES.iter().enumerate().map(|(i, (role_key, _))| {
            let v = inputs_vals[i].trim().to_string();
            let opt = if v.is_empty() { None } else { Some(v) };
            (role_key.to_string(), opt)
        }).collect();

        let players_json = serde_json::to_string(&players).unwrap_or_default();
        let inputs_vals_clone = inputs_vals.clone();

        saving.set(true);
        leptos::task::spawn_local(async move {
            match create_opponent_with_players_fn(name, players_json).await {
                Ok((opponent_id, player_ids)) => {
                    saving.set(false);
                    toast.show.run((ToastKind::Success, "Opponent saved and players fetched".into()));
                    let done_id = opponent_id.clone();

                    // Sequential fetch for players that have a riot ID
                    for (i, player_id) in player_ids.into_iter().enumerate() {
                        let riot_id = inputs_vals_clone.get(i).cloned().unwrap_or_default();
                        let riot_id = riot_id.trim().to_string();
                        if !riot_id.is_empty() {
                            let _ = fetch_player_intel_fn(player_id, riot_id).await;
                        }
                    }

                    on_done(done_id);
                }
                Err(e) => {
                    saving.set(false);
                    toast.show.run((ToastKind::Error, format!("{e}")));
                }
            }
        });
    };

    view! {
        <div class="bg-surface rounded-xl border border-divider p-6">
            <h2 class="text-base font-semibold text-primary mb-4">"New Opponent Team"</h2>

            // Team name input
            <div class="mb-4">
                <input
                    type="text"
                    placeholder="Opponent team name..."
                    prop:value=move || team_name.get()
                    on:input=move |ev| team_name.set(event_target_value(&ev))
                    class="w-full text-sm bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary placeholder:text-muted focus:outline-none focus:border-accent"
                />
                {move || {
                    if let Some(err) = name_error.get() {
                        view! { <p class="text-xs text-red-400 mt-1">{err}</p> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                }}
            </div>

            // 5 role rows
            <div class="flex flex-col gap-2 mb-4">
                {ROLES.iter().enumerate().map(|(i, (_, role_display))| {
                    let role_sig = role_inputs[i];
                    let err_sig = validation_errors[i];
                    let role_label = *role_display;
                    view! {
                        <div>
                            <div class="flex items-center gap-2 py-1">
                                <span class="w-20 text-sm text-secondary font-medium">{role_label}</span>
                                <input
                                    type="text"
                                    placeholder="Name#Tag (e.g. Faker#KR1)"
                                    prop:value=move || role_sig.get()
                                    on:input=move |ev| role_sig.set(event_target_value(&ev))
                                    class="flex-1 text-sm bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary placeholder:text-muted focus:outline-none focus:border-accent"
                                />
                            </div>
                            {move || {
                                if let Some(err) = err_sig.get() {
                                    view! { <p class="text-xs text-red-400 ml-22 pl-2">{err}</p> }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }
                            }}
                        </div>
                    }
                }).collect_view()}
            </div>

            // Footer buttons
            <div class="flex justify-end gap-3 mt-4">
                <button
                    on:click=move |_| on_discard()
                    class="bg-elevated border border-divider text-secondary text-sm px-4 py-2 rounded-lg hover:bg-overlay cursor-pointer"
                >
                    "Discard Form"
                </button>
                <button
                    on:click=on_save_fetch
                    disabled=move || saving.get()
                    class=move || {
                        let base = "bg-accent text-accent-contrast text-sm px-4 py-2 rounded-lg font-medium hover:bg-accent-hover cursor-pointer";
                        if saving.get() {
                            format!("{base} opacity-50 cursor-not-allowed")
                        } else {
                            base.to_string()
                        }
                    }
                >
                    {move || if saving.get() { "Saving..." } else { "Save & Fetch" }}
                </button>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// OpponentDetail component
// ---------------------------------------------------------------------------

#[component]
fn OpponentDetail(
    opponent: Opponent,
    players: Vec<OpponentPlayer>,
    on_save_done: impl Fn() + Copy + Send + Sync + 'static,
    on_delete: Callback<String>,
    on_player_change: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let opp_id = opponent.id.clone().unwrap_or_default();
    let opp_id_save = opp_id.clone();
    let opp_id_delete = opp_id.clone();

    let name = RwSignal::new(opponent.name.clone());
    let notes = RwSignal::new(opponent.notes.clone().unwrap_or_default());
    let confirm_delete = RwSignal::new(false);
    let refreshing_all: RwSignal<bool> = RwSignal::new(false);

    // Per-role fetch states (5 slots matching ROLES order)
    let fetch_states: [RwSignal<FetchState>; 5] = [
        RwSignal::new(FetchState::Idle),
        RwSignal::new(FetchState::Idle),
        RwSignal::new(FetchState::Idle),
        RwSignal::new(FetchState::Idle),
        RwSignal::new(FetchState::Idle),
    ];

    // Build ordered players vec matching ROLES order
    let players_by_role: [Option<OpponentPlayer>; 5] = {
        let find_player = |role_key: &str| {
            players.iter().find(|p| p.role == role_key).cloned()
        };
        [
            find_player("top"),
            find_player("jungle"),
            find_player("mid"),
            find_player("bot"),
            find_player("support"),
        ]
    };

    let players_stored = StoredValue::new(players_by_role);

    let on_save = move |_| {
        let id = opp_id_save.clone();
        let n = name.get_untracked();
        let nt = notes.get_untracked();
        leptos::task::spawn_local(async move {
            match save_opponent(id, n, nt).await {
                Ok(()) => on_save_done(),
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        });
    };

    let on_refresh_all = move |_| {
        let players_snap = players_stored.get_value();
        refreshing_all.set(true);
        for i in 0..5 {
            fetch_states[i].set(FetchState::Idle);
        }
        let fetch_states_clone = fetch_states;
        leptos::task::spawn_local(async move {
            let mut success_count = 0u32;
            for (i, maybe_player) in players_snap.iter().enumerate() {
                if let Some(player) = maybe_player {
                    if let Some(riot_id) = &player.riot_summoner_name {
                        if !riot_id.trim().is_empty() {
                            let pid = player.id.clone().unwrap_or_default();
                            let rid = riot_id.clone();
                            fetch_states_clone[i].set(FetchState::Fetching);
                            match fetch_player_intel_fn(pid, rid).await {
                                Ok(()) => {
                                    fetch_states_clone[i].set(FetchState::Success);
                                    success_count += 1;
                                }
                                Err(e) => {
                                    fetch_states_clone[i].set(FetchState::Error(e.to_string()));
                                }
                            }
                        }
                    }
                }
            }
            refreshing_all.set(false);
            on_player_change();
            if success_count == 5 {
                toast.show.run((ToastKind::Success, "All players refreshed".into()));
            } else if success_count > 0 {
                toast.show.run((ToastKind::Success, format!("{success_count}/5 players refreshed")));
            }
        });
    };

    view! {
        <div class="bg-surface rounded-xl border border-divider">
            // Header
            <div class="px-6 py-4 border-b border-divider flex items-center justify-between gap-2">
                <input
                    type="text"
                    prop:value=move || name.get()
                    on:input=move |ev| name.set(event_target_value(&ev))
                    class="flex-1 text-sm bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary placeholder:text-muted focus:outline-none focus:border-accent"
                    placeholder="Team name..."
                />
                <div class="flex gap-2 shrink-0">
                    <button
                        on:click=on_save
                        class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold px-4 py-1.5 rounded-lg text-sm transition-colors cursor-pointer"
                    >
                        "Save"
                    </button>
                    <button
                        on:click=on_refresh_all
                        disabled=move || refreshing_all.get()
                        class="bg-elevated border border-divider text-secondary text-sm px-3 py-2 rounded-lg hover:bg-overlay disabled:opacity-50 cursor-pointer"
                    >
                        {move || if refreshing_all.get() { "Refreshing..." } else { "Refresh All" }}
                    </button>
                    {move || {
                        let id = opp_id_delete.clone();
                        if confirm_delete.get() {
                            view! {
                                <div class="flex gap-1">
                                    <span class="text-sm text-secondary self-center">"Confirm delete?"</span>
                                    <button
                                        on:click=move |_| {
                                            on_delete.run(id.clone());
                                            confirm_delete.set(false);
                                        }
                                        class="bg-red-700 hover:bg-red-600 text-white text-sm px-3 py-2 rounded-lg transition-colors cursor-pointer"
                                    >
                                        "Delete"
                                    </button>
                                    <button
                                        on:click=move |_| confirm_delete.set(false)
                                        class="bg-elevated border border-divider text-secondary text-sm px-3 py-2 rounded-lg hover:bg-overlay cursor-pointer"
                                    >
                                        "Keep Opponent"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <button
                                    on:click=move |_| confirm_delete.set(true)
                                    class="text-red-400 text-sm hover:text-red-300 cursor-pointer"
                                >
                                    "Delete"
                                </button>
                            }.into_any()
                        }
                    }}
                </div>
            </div>

            // Notes
            <div class="px-6 py-4 border-b border-divider">
                <label class="block text-xs font-medium text-muted mb-1">"Notes"</label>
                <textarea
                    prop:value=move || notes.get()
                    on:input=move |ev| notes.set(event_target_value(&ev))
                    rows="2"
                    class="w-full text-sm bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary placeholder:text-muted focus:outline-none focus:border-accent resize-y"
                    placeholder="General scouting notes..."
                />
            </div>

            // 5 PlayerCard rows
            <div class="px-6 py-4 flex flex-col gap-3">
                {ROLES.iter().enumerate().map(|(i, (role_key, role_display))| {
                    let maybe_player = players_stored.with_value(|p| p[i].clone());
                    let fs = fetch_states[i];
                    let opp_id_for_slot = opp_id.clone();
                    let role_key_str = role_key.to_string();
                    let role_display_str = role_display.to_string();
                    match maybe_player {
                        Some(player) => view! {
                            <PlayerCard
                                player=player
                                fetch_state=fs
                                on_change=on_player_change
                            />
                        }.into_any(),
                        None => view! {
                            <EmptyRoleSlot
                                role_key=role_key_str
                                role_display=role_display_str
                                opponent_id=opp_id_for_slot
                                on_added=on_player_change
                            />
                        }.into_any(),
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// EmptyRoleSlot — shown when a role has no player yet
// ---------------------------------------------------------------------------

#[component]
fn EmptyRoleSlot(
    role_key: String,
    role_display: String,
    opponent_id: String,
    on_added: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let riot_id_input: RwSignal<String> = RwSignal::new(String::new());

    let role_key_clone = role_key.clone();
    let opponent_id_clone = opponent_id.clone();

    let on_add = move |_| {
        let riot_id = riot_id_input.get_untracked();
        let oid = opponent_id_clone.clone();
        let rk = role_key_clone.clone();
        leptos::task::spawn_local(async move {
            let name = riot_id.trim().to_string();
            match add_player(oid, name, rk).await {
                Ok(_) => on_added(),
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        });
    };

    view! {
        <div class="bg-elevated rounded-lg border border-divider/50 p-4">
            <div class="flex items-center gap-2">
                <span class="w-20 text-sm font-semibold text-secondary">{role_display}</span>
                <input
                    type="text"
                    placeholder="Name#Tag (e.g. Faker#KR1)"
                    prop:value=move || riot_id_input.get()
                    on:input=move |ev| riot_id_input.set(event_target_value(&ev))
                    class="flex-1 text-sm bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary placeholder:text-muted focus:outline-none focus:border-accent"
                />
                <button
                    on:click=on_add
                    class="text-xs text-secondary hover:text-primary bg-surface hover:bg-overlay border border-divider px-3 py-2 rounded-lg cursor-pointer"
                >
                    "+ Add"
                </button>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// PlayerCard component
// ---------------------------------------------------------------------------

#[component]
fn PlayerCard(
    player: OpponentPlayer,
    fetch_state: RwSignal<FetchState>,
    on_change: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let player_id = player.id.clone().unwrap_or_default();
    let player_id_refresh = player_id.clone();

    let role_display = match player.role.as_str() {
        "top" => "Top",
        "jungle" => "Jungle",
        "mid" => "Mid",
        "bot" => "Bot",
        "support" => "Support",
        _ => &player.role,
    }
    .to_string();

    let p_summoner: RwSignal<String> = RwSignal::new(
        player.riot_summoner_name.clone().unwrap_or_default(),
    );
    let pool_expanded: RwSignal<bool> = RwSignal::new(false);

    // Compute intel once from player data
    let intel = player.compute_intel();
    let otp_champion = intel.otp_champion.clone();
    let mastery_data = player.mastery_data();
    let last_fetched = player.last_fetched.clone();
    let recent_champions = player.recent_champions.clone();
    let comfort_picks = player.comfort_picks();
    let role_distribution = player.role_distribution();
    let pool_sz = player.pool_size();

    // Build mastery lookup: champion_name -> (level, points)
    let mastery_map: std::collections::HashMap<String, (i32, i32)> = mastery_data
        .iter()
        .map(|(name, level, pts)| (name.clone(), (*level, *pts)))
        .collect();

    // Sort champions by mastery points descending
    let mut sorted_champions = recent_champions.clone();
    sorted_champions.sort_by(|a, b| {
        let pa = mastery_map.get(a).map(|(_, pts)| *pts).unwrap_or(0);
        let pb = mastery_map.get(b).map(|(_, pts)| *pts).unwrap_or(0);
        pb.cmp(&pa)
    });
    // Deduplicate while preserving mastery-sorted order
    let mut seen = std::collections::HashSet::new();
    sorted_champions.retain(|c| seen.insert(c.clone()));

    let mastery_map_stored = StoredValue::new(mastery_map);

    let on_refresh = move |_| {
        let pid = player_id_refresh.clone();
        let summoner = p_summoner.get_untracked();
        if summoner.trim().is_empty() {
            toast.show.run((ToastKind::Error, "Enter a summoner name (Name#Tag) first".into()));
            return;
        }
        fetch_state.set(FetchState::Fetching);
        let summoner_clone = summoner.clone();
        leptos::task::spawn_local(async move {
            match fetch_player_intel_fn(pid, summoner_clone).await {
                Ok(()) => {
                    fetch_state.set(FetchState::Success);
                    on_change();
                    // Auto-clear success after 3s
                    #[cfg(feature = "hydrate")]
                    {
                        use wasm_bindgen::prelude::*;
                        let cb = Closure::once(move || {
                            fetch_state.set(FetchState::Idle);
                        });
                        if let Some(win) = web_sys::window() {
                            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                                cb.as_ref().unchecked_ref(),
                                3000,
                            );
                        }
                        cb.forget();
                    }
                }
                Err(e) => {
                    fetch_state.set(FetchState::Error(e.to_string()));
                }
            }
        });
    };

    // Recency badge text and class
    let recency_display = match &last_fetched {
        None => None,
        Some(lf) if lf.is_empty() => None,
        Some(lf) => Some((format_timestamp(lf), is_stale(lf))),
    };

    view! {
        <div class="bg-elevated rounded-lg border border-divider/50 p-4">
            // Row 1: header
            <div class="flex items-center gap-2 mb-3 flex-wrap">
                <span class="text-sm font-semibold text-secondary">{role_display}</span>

                // OTP badge
                {otp_champion.as_ref().map(|champ| {
                    let champ = champ.clone();
                    view! {
                        <span class="text-xs bg-orange-500/20 text-orange-400 border border-orange-500/30 rounded px-2 py-0.5">
                            "\u{26a0} OTP: "{champ}
                        </span>
                    }
                })}

                // Recency badge
                {match recency_display {
                    None => view! {
                        <span class="text-xs text-dimmed">"Never fetched"</span>
                    }.into_any(),
                    Some((text, stale)) => view! {
                        <span class=if stale { "text-xs text-orange-400" } else { "text-xs text-muted" }>
                            "Last fetched: "{text}
                        </span>
                    }.into_any(),
                }}

                // Refresh icon button
                <button
                    on:click=on_refresh
                    title="Refresh player data"
                    class="ml-auto w-8 h-8 flex items-center justify-center rounded text-muted hover:text-secondary hover:bg-elevated transition-colors cursor-pointer"
                >
                    {move || {
                        if fetch_state.get() == FetchState::Fetching {
                            view! {
                                <svg class="animate-spin w-4 h-4 text-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                </svg>
                            }.into_any()
                        } else {
                            // Refresh/reload icon (two arrows in circle)
                            view! {
                                <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                                </svg>
                            }.into_any()
                        }
                    }}
                </button>
            </div>

            // Row 2: Riot ID input + fetch status icon
            <div class="mb-3">
                <div class="flex items-center gap-2">
                    <input
                        type="text"
                        placeholder="Name#Tag (e.g. Faker#KR1)"
                        prop:value=move || p_summoner.get()
                        on:input=move |ev| p_summoner.set(event_target_value(&ev))
                        class="flex-1 text-sm bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary placeholder:text-muted focus:outline-none focus:border-accent"
                    />
                    // Fetch status icon
                    {move || match fetch_state.get() {
                        FetchState::Idle => view! { <span></span> }.into_any(),
                        FetchState::Fetching => view! {
                            <svg class="animate-spin w-4 h-4 text-muted shrink-0" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                        }.into_any(),
                        FetchState::Success => view! {
                            <span class="text-green-400 w-4 h-4 text-sm shrink-0">"✓"</span>
                        }.into_any(),
                        FetchState::Error(_) => view! {
                            <span class="text-red-400 w-4 h-4 text-sm shrink-0">"✗"</span>
                        }.into_any(),
                    }}
                </div>
                // Error message below input
                {move || match fetch_state.get() {
                    FetchState::Error(msg) => view! {
                        <p class="text-xs text-red-400 mt-1">{msg}</p>
                    }.into_any(),
                    _ => view! { <span></span> }.into_any(),
                }}
            </div>

            // Row 3: Champion pills (sorted by mastery points)
            {if sorted_champions.is_empty() {
                view! { <span></span> }.into_any()
            } else {
                let pills = sorted_champions.iter().map(|champ_name| {
                    let name = champ_name.clone();
                    let level_display = mastery_map_stored.with_value(|m| {
                        m.get(&name).map(|(lvl, _)| format!(" M{}", lvl))
                    });
                    let display = match level_display {
                        Some(suffix) => format!("{}{}", name, suffix),
                        None => name,
                    };
                    view! {
                        <span class="text-xs bg-surface border border-divider/50 text-secondary rounded px-2 py-1">
                            {display}
                        </span>
                    }
                }).collect_view();
                view! {
                    <div class="flex flex-wrap gap-1 mb-3">
                        {pills}
                    </div>
                }.into_any()
            }}

            // Row 4: Pool Analysis (collapsible)
            <div>
                <button
                    on:click=move |_| pool_expanded.update(|v| *v = !*v)
                    class="text-xs text-muted hover:text-secondary cursor-pointer flex items-center gap-1"
                >
                    {move || if pool_expanded.get() { "Pool Analysis \u{25be}" } else { "Pool Analysis \u{25b8}" }}
                </button>
                {move || {
                    if pool_expanded.get() {
                        let has_data = pool_sz > 0 || !comfort_picks.is_empty();
                        if has_data {
                            // Role distribution display
                            let role_dist_text = if role_distribution.is_empty() {
                                String::new()
                            } else {
                                let total_matches: u32 = role_distribution.iter().map(|(_, c)| c).sum();
                                let mut parts: Vec<String> = role_distribution.iter().map(|(role, count)| {
                                    let display = match role.as_str() {
                                        "TOP" => "Top",
                                        "JUNGLE" => "Jungle",
                                        "MIDDLE" => "Mid",
                                        "BOTTOM" => "Bot",
                                        "UTILITY" => "Support",
                                        _ => role.as_str(),
                                    };
                                    let pct = if total_matches > 0 {
                                        (*count as f32 / total_matches as f32 * 100.0).round() as u32
                                    } else {
                                        0
                                    };
                                    format!("{} {}%", display, pct)
                                }).collect();
                                parts.sort();
                                parts.join(" / ")
                            };

                            // Comfort picks display
                            let comfort_text = if comfort_picks.is_empty() {
                                String::new()
                            } else {
                                comfort_picks.iter()
                                    .map(|(name, pct)| format!("{} {:.0}%", name, pct))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            };

                            let plural = if pool_sz != 1 { "s" } else { "" };

                            view! {
                                <div class="mt-2 text-xs text-muted space-y-0.5">
                                    <p>"Pool: "{pool_sz}" champion"{plural}</p>
                                    {if !role_dist_text.is_empty() {
                                        view! { <p>"Roles: "{role_dist_text}</p> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                    {if !comfort_text.is_empty() {
                                        view! { <p>"Comfort: "{comfort_text}</p> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <p class="text-xs text-muted mt-2">"Fetch player data to see pool analysis."</p>
                            }.into_any()
                        }
                    } else {
                        view! { <span></span> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
