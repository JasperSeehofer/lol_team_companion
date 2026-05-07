use crate::components::ui::{EmptyState, SkeletonCard, ToastContext, ToastKind};
use crate::models::team::Team;
use leptos::prelude::*;

#[server]
pub async fn create_team(name: String, region: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use leptos_axum::redirect;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::create_team(&db, &user.id, name, region)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    redirect("/team/dashboard");
    Ok(())
}

#[server]
pub async fn link_riot_account(riot_id: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use crate::server::riot;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let parts: Vec<&str> = riot_id.splitn(2, '#').collect();
    if parts.len() != 2 {
        return Err(ServerFnError::new(
            "Invalid Riot ID format (use GameName#TAG)",
        ));
    }
    let (game_name, tag_line) = (parts[0], parts[1]);

    let platform = riot::platform_route_from_str(user.riot_region.as_deref().unwrap_or("EUW"));
    let puuid = riot::get_puuid(game_name, tag_line, platform)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    db::update_user_riot(&db, user.id, puuid, riot_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

#[server]
pub async fn list_teams() -> Result<Vec<Team>, ServerFnError> {
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::list_all_teams(&db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn request_to_join(team_id: String) -> Result<(), ServerFnError> {
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

    db::create_join_request(&db, &user.id, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn RosterPage() -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    // Auth redirect + mode detection
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
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
            }
            _ => {}
        }
    });

    // Toast for create_team errors (success redirects to dashboard)
    Effect::new(move || {
        if let Some(Err(e)) = ServerAction::<CreateTeam>::new().value().get() {
            toast.show.run((ToastKind::Error, format!("{e}")));
        }
    });

    let (search_query, set_search_query) = signal(String::new());
    let create_team_action = ServerAction::<CreateTeam>::new();
    let link_riot = ServerAction::<LinkRiotAccount>::new();
    let teams_resource = Resource::new(|| (), |_| list_teams());

    // Toast for link_riot
    Effect::new(move || {
        if let Some(result) = link_riot.value().get() {
            match result {
                Ok(()) => toast.show.run((ToastKind::Success, "Riot account linked".into())),
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        }
    });

    view! {
        <div class="canvas-grain bg-base min-h-screen px-8 py-6">
            <div class="max-w-3xl mx-auto">
                <header class="mb-6">
                    <div class="font-imperial uppercase tracking-[0.18em] text-[11px] text-accent">
                        "Founding the banner"
                    </div>
                    <h1 class="font-display italic text-[40px] leading-tight text-primary mt-1">
                        "Team Roster"
                    </h1>
                </header>
                // Mode gate: show CTA for solo-mode users instead of team content
                {move || if is_solo_mode.get() {
                    view! {
                        <div class="py-8 text-center">
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
                    }.into_any()
                } else {
                    view! {
            <div class="flex flex-col gap-8">
                // No-team context message
                <EmptyState
                    icon="👥"
                    message="You're not part of a team yet — create a new team or join an existing one"
                />

                // Create Team — Card.plain
                <section class="bg-elevated border border-divider rounded-xl p-6">
                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Found"</div>
                    <h2 class="font-display italic text-2xl text-primary mb-4">"Create a New Team"</h2>
                    <ActionForm action=create_team_action>
                        <div class="flex flex-col gap-4">
                            {move || create_team_action.value().get().and_then(|r| r.err()).map(|e| view! {
                                <div class="bg-danger/10 border border-danger/30 text-danger rounded-lg px-4 py-3 text-sm" role="alert">
                                    {e.to_string()}
                                </div>
                            })}
                            <div>
                                <label class="block font-imperial uppercase tracking-[0.18em] text-[10px] text-muted mb-1">"Team Name"</label>
                                <input
                                    type="text"
                                    name="name"
                                    required
                                    class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                />
                            </div>
                            <div>
                                <label class="block font-imperial uppercase tracking-[0.18em] text-[10px] text-muted mb-1">"Region"</label>
                                <select
                                    name="region"
                                    class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                >
                                    <option value="EUW">"EUW"</option>
                                    <option value="EUNE">"EUNE"</option>
                                    <option value="NA">"NA"</option>
                                    <option value="KR">"KR"</option>
                                    <option value="BR">"BR"</option>
                                </select>
                            </div>
                            <button
                                type="submit"
                                class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded-lg px-4 py-2 transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            >
                                "Create Team"
                            </button>
                        </div>
                    </ActionForm>
                </section>

                // Join Existing Team — Card.plain
                <section class="bg-elevated border border-divider rounded-xl p-6">
                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Enlist"</div>
                    <h2 class="font-display italic text-2xl text-primary mb-1">"Join an Existing Team"</h2>
                    <p class="text-muted text-sm mb-4">"Search for a team by name and request to join."</p>

                    <input
                        type="text"
                        placeholder="Search teams by name..."
                        prop:value=move || search_query.get()
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                        class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none mb-4"
                    />

                    <Suspense fallback=|| view! { <div class="flex flex-col gap-2"><SkeletonCard height="h-12" /><SkeletonCard height="h-12" /><SkeletonCard height="h-12" /></div> }>
                        {move || teams_resource.get().map(|result| match result {
                            Ok(teams) => {
                                let search_val = search_query.get();
                                if search_val.is_empty() {
                                    view! {
                                        <p class="text-muted text-sm">"Type to search for teams..."</p>
                                    }.into_any()
                                } else {
                                    let filtered: Vec<_> = teams.into_iter()
                                        .filter(|t| t.name.to_lowercase().contains(&search_val.to_lowercase()))
                                        .collect();
                                    if filtered.is_empty() {
                                        view! {
                                            <p class="text-dimmed text-sm">"No teams match your search."</p>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="flex flex-col gap-2">
                                                {filtered.into_iter().map(|team| {
                                                    let team_id = team.id.clone().unwrap_or_default();
                                                    let team_name = team.name.clone();
                                                    let region = team.region.clone();
                                                    let member_count = team.member_count.unwrap_or(0);
                                                    view! {
                                                        <div class="bg-surface border border-outline/50 rounded-lg px-4 py-3 flex items-center justify-between">
                                                            <div class="flex items-center gap-3">
                                                                <span class="font-display italic text-primary text-base">{team_name.clone()}</span>
                                                                <span class="font-mono text-muted text-sm">{region}</span>
                                                                <span class="text-muted text-sm tabular-nums">{member_count} " members"</span>
                                                            </div>
                                                            <button
                                                                class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded-lg px-3 py-1.5 text-sm transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                                on:click=move |_| {
                                                                    let id = team_id.clone();
                                                                    let tname = team_name.clone();
                                                                    leptos::task::spawn_local(async move {
                                                                        match request_to_join(id).await {
                                                                            Ok(_) => toast.show.run((ToastKind::Success, format!("Join request sent to {}!", tname))),
                                                                            Err(e) => toast.show.run((ToastKind::Error, e.to_string())),
                                                                        }
                                                                    });
                                                                }
                                                            >
                                                                "Request to Join"
                                                            </button>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_any()
                                    }
                                }
                            },
                            Err(e) => view! {
                                <p class="text-danger text-sm" role="alert">{e.to_string()}</p>
                            }.into_any(),
                        })}
                    </Suspense>
                </section>

                // Link Riot Account — Card.plain
                <section class="bg-elevated border border-divider rounded-xl p-6">
                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Linked seal"</div>
                    <h2 class="font-display italic text-2xl text-primary mb-4">"Link Riot Account"</h2>
                    <ActionForm action=link_riot>
                        <div class="flex flex-col gap-4">
                            <div>
                                <label class="block font-imperial uppercase tracking-[0.18em] text-[10px] text-muted mb-1">
                                    "Riot ID (e.g. PlayerName#EUW)"
                                </label>
                                <input
                                    type="text"
                                    name="riot_id"
                                    placeholder="GameName#TAG"
                                    required
                                    class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                />
                            </div>
                            <button
                                type="submit"
                                class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded-lg px-4 py-2 transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            >
                                "Link Account"
                            </button>
                        </div>
                    </ActionForm>
                </section>
            </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
