use leptos::prelude::*;
use crate::models::team::Team;

#[server]
pub async fn create_team(name: String, region: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use leptos_axum::redirect;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

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
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let parts: Vec<&str> = riot_id.splitn(2, '#').collect();
    if parts.len() != 2 {
        return Err(ServerFnError::new("Invalid Riot ID format (use GameName#TAG)"));
    }
    let (game_name, tag_line) = (parts[0], parts[1]);

    let puuid = riot::get_puuid(game_name, tag_line)
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

    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::list_all_teams(&db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn join_team(team_id: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use leptos_axum::redirect;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::join_team(&db, &user.id, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    redirect("/team/dashboard");
    Ok(())
}

#[component]
pub fn RosterPage() -> impl IntoView {
    let create_team_action = ServerAction::<CreateTeam>::new();
    let link_riot = ServerAction::<LinkRiotAccount>::new();
    let teams_resource = Resource::new(|| (), |_| list_teams());
    let (join_error, set_join_error) = signal(Option::<String>::None);

    view! {
        <div class="max-w-2xl mx-auto py-8 px-6 flex flex-col gap-8">
            // Create Team
            <section>
                <h2 class="text-2xl font-bold text-white mb-4">"Create a New Team"</h2>
                <ActionForm action=create_team_action>
                    <div class="flex flex-col gap-4">
                        {move || create_team_action.value().get().and_then(|r| r.err()).map(|e| view! {
                            <div class="bg-red-900 border border-red-700 text-red-200 rounded px-4 py-3 text-sm">
                                {e.to_string()}
                            </div>
                        })}
                        <div>
                            <label class="block text-gray-300 text-sm mb-1">"Team Name"</label>
                            <input
                                type="text"
                                name="name"
                                required
                                class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white focus:outline-none focus:border-yellow-400"
                            />
                        </div>
                        <div>
                            <label class="block text-gray-300 text-sm mb-1">"Region"</label>
                            <select
                                name="region"
                                class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white focus:outline-none focus:border-yellow-400"
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
                            class="bg-yellow-400 hover:bg-yellow-300 text-gray-900 font-bold rounded px-4 py-2 transition-colors"
                        >
                            "Create Team"
                        </button>
                    </div>
                </ActionForm>
            </section>

            // Join Existing Team
            <section>
                <h2 class="text-2xl font-bold text-white mb-1">"Join an Existing Team"</h2>
                <p class="text-gray-400 text-sm mb-4">"Find a team below and click Join to become a member."</p>

                {move || join_error.get().map(|e| view! {
                    <div class="bg-red-900 border border-red-700 text-red-200 rounded px-4 py-3 text-sm mb-4">
                        {e}
                    </div>
                })}

                <Suspense fallback=|| view! { <div class="text-gray-400 text-sm">"Loading teams..."</div> }>
                    {move || teams_resource.get().map(|result| match result {
                        Ok(teams) if teams.is_empty() => view! {
                            <p class="text-gray-500 text-sm">"No teams yet. Be the first to create one!"</p>
                        }.into_any(),
                        Ok(teams) => view! {
                            <div class="flex flex-col gap-2">
                                {teams.into_iter().map(|team| {
                                    let team_id = team.id.clone().unwrap_or_default();
                                    let team_name = team.name.clone();
                                    let region = team.region.clone();
                                    view! {
                                        <div class="bg-gray-800 border border-gray-700 rounded px-4 py-3 flex items-center justify-between">
                                            <div>
                                                <span class="text-white font-medium">{team_name}</span>
                                                <span class="text-gray-400 text-sm ml-2">{region}</span>
                                            </div>
                                            <button
                                                class="bg-gray-700 hover:bg-yellow-400 hover:text-gray-900 text-gray-300 text-sm font-medium rounded px-3 py-1.5 transition-colors"
                                                on:click=move |_| {
                                                    let id = team_id.clone();
                                                    leptos::task::spawn_local(async move {
                                                        match join_team(id).await {
                                                            Ok(_) => {
                                                                // redirect handled server-side
                                                            }
                                                            Err(e) => set_join_error.set(Some(e.to_string())),
                                                        }
                                                    });
                                                }
                                            >
                                                "Join"
                                            </button>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any(),
                        Err(e) => view! {
                            <p class="text-red-400 text-sm">{e.to_string()}</p>
                        }.into_any(),
                    })}
                </Suspense>
            </section>

            // Link Riot Account
            <section>
                <h2 class="text-2xl font-bold text-white mb-4">"Link Riot Account"</h2>
                <ActionForm action=link_riot>
                    <div class="flex flex-col gap-4">
                        {move || link_riot.value().get().and_then(|r| r.err()).map(|e| view! {
                            <div class="bg-red-900 border border-red-700 text-red-200 rounded px-4 py-3 text-sm">
                                {e.to_string()}
                            </div>
                        })}
                        {move || link_riot.value().get().and_then(|r| r.ok()).map(|_| view! {
                            <div class="bg-green-900 border border-green-700 text-green-200 rounded px-4 py-3 text-sm">
                                "Riot account linked successfully!"
                            </div>
                        })}
                        <div>
                            <label class="block text-gray-300 text-sm mb-1">
                                "Riot ID (e.g. PlayerName#EUW)"
                            </label>
                            <input
                                type="text"
                                name="riot_id"
                                placeholder="GameName#TAG"
                                required
                                class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white focus:outline-none focus:border-yellow-400"
                            />
                        </div>
                        <button
                            type="submit"
                            class="bg-blue-500 hover:bg-blue-400 text-white font-bold rounded px-4 py-2 transition-colors"
                        >
                            "Link Account"
                        </button>
                    </div>
                </ActionForm>
            </section>
        </div>
    }
}
