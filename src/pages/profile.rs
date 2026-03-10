use leptos::prelude::*;
use crate::models::user::PublicUser;
use crate::models::champion::{Champion, ChampionPoolEntry};

#[server]
pub async fn get_current_user() -> Result<Option<PublicUser>, ServerFnError> {
    use crate::server::auth::AuthSession;

    let auth: AuthSession = leptos_axum::extract().await?;
    Ok(auth.user.map(|u| PublicUser {
        id: u.id,
        username: u.username,
        riot_summoner_name: u.riot_summoner_name,
    }))
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use leptos_axum::redirect;

    let mut auth: AuthSession = leptos_axum::extract().await?;
    auth.logout().await.map_err(|e| ServerFnError::new(e.to_string()))?;
    redirect("/");
    Ok(())
}

#[server]
pub async fn update_profile(username: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let user_key = user.id.strip_prefix("user:").unwrap_or(&user.id).to_string();
    db.query("UPDATE type::record('user', $user_key) SET username = $username")
        .bind(("user_key", user_key))
        .bind(("username", username))
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

#[server]
pub async fn get_champion_pool() -> Result<Vec<ChampionPoolEntry>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::get_champion_pool(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn add_champion_to_pool(champion: String, role: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    if champion.trim().is_empty() {
        return Err(ServerFnError::new("Champion name cannot be empty"));
    }

    db::add_to_champion_pool(&db, &user.id, champion.trim().to_string(), role)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn remove_champion_from_pool(champion: String, role: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::remove_from_champion_pool(&db, &user.id, champion, role)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_champions_for_pool() -> Result<Vec<Champion>, ServerFnError> {
    use crate::server::data_dragon;
    data_dragon::fetch_champions()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

const POOL_ROLES: &[&str] = &["Top", "Jungle", "Mid", "ADC", "Support"];

#[component]
pub fn ProfilePage() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());
    let update_profile_action = ServerAction::<UpdateProfile>::new();
    let link_riot = ServerAction::<crate::pages::team::roster::LinkRiotAccount>::new();
    let logout_action = ServerAction::<Logout>::new();

    let pool_resource = Resource::new(|| (), |_| get_champion_pool());
    let champions_resource = Resource::new(|| (), |_| get_champions_for_pool());

    let (active_role, set_active_role) = signal("Top");
    let (add_input, set_add_input) = signal(String::new());
    let (pool_error, set_pool_error) = signal(Option::<String>::None);

    view! {
        <div class="max-w-2xl mx-auto py-8 px-6">
            <h1 class="text-3xl font-bold text-white mb-8">"Profile"</h1>
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading..."</p> }>
                {move || Suspend::new(async move {
                    match user.await {
                        Ok(Some(u)) => {
                            let username = u.username.clone();
                            let riot_name = u.riot_summoner_name.clone();
                            view! {
                                <div class="flex flex-col gap-6">
                                    // Account info
                                    <section class="bg-gray-900 border border-gray-700 rounded-lg p-6">
                                        <h2 class="text-xl font-semibold text-white mb-4">"Account"</h2>

                                        {move || update_profile_action.value().get().map(|r| match r {
                                            Ok(()) => view! {
                                                <div class="bg-green-900 border border-green-700 text-green-200 rounded px-4 py-3 text-sm mb-4">
                                                    "Profile updated!"
                                                </div>
                                            }.into_any(),
                                            Err(e) => view! {
                                                <div class="bg-red-900 border border-red-700 text-red-200 rounded px-4 py-3 text-sm mb-4">
                                                    {e.to_string()}
                                                </div>
                                            }.into_any(),
                                        })}

                                        <ActionForm action=update_profile_action>
                                            <div class="flex flex-col gap-4">
                                                <div>
                                                    <label class="block text-gray-300 text-sm mb-1">"Username"</label>
                                                    <input
                                                        type="text"
                                                        name="username"
                                                        value=username
                                                        required
                                                        class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white focus:outline-none focus:border-yellow-400"
                                                    />
                                                </div>
                                                <button
                                                    type="submit"
                                                    class="bg-yellow-400 hover:bg-yellow-300 text-gray-900 font-bold rounded px-4 py-2 transition-colors"
                                                >
                                                    "Save"
                                                </button>
                                            </div>
                                        </ActionForm>
                                    </section>

                                    // Riot account linking
                                    <section class="bg-gray-900 border border-gray-700 rounded-lg p-6">
                                        <h2 class="text-xl font-semibold text-white mb-4">"Riot Account"</h2>

                                        {match riot_name {
                                            Some(name) => view! {
                                                <p class="text-green-400 text-sm mb-4">
                                                    "Linked: "
                                                    <span class="font-semibold">{name}</span>
                                                </p>
                                            }.into_any(),
                                            None => view! {
                                                <p class="text-gray-400 text-sm mb-4">
                                                    "No Riot account linked."
                                                </p>
                                            }.into_any(),
                                        }}

                                        {move || link_riot.value().get().map(|r| match r {
                                            Ok(()) => view! {
                                                <div class="bg-green-900 border border-green-700 text-green-200 rounded px-4 py-3 text-sm mb-4">
                                                    "Riot account linked!"
                                                </div>
                                            }.into_any(),
                                            Err(e) => view! {
                                                <div class="bg-red-900 border border-red-700 text-red-200 rounded px-4 py-3 text-sm mb-4">
                                                    {e.to_string()}
                                                </div>
                                            }.into_any(),
                                        })}

                                        <ActionForm action=link_riot>
                                            <div class="flex flex-col gap-4">
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

                                    // Champion Pool
                                    <section class="bg-gray-900 border border-gray-700 rounded-lg p-6">
                                        <h2 class="text-xl font-semibold text-white mb-1">"Champion Pool"</h2>
                                        <p class="text-gray-400 text-sm mb-4">"Champions you play, organized by role."</p>

                                        // Role tabs
                                        <div class="flex gap-1 mb-4">
                                            {POOL_ROLES.iter().map(|&role| {
                                                view! {
                                                    <button
                                                        class=move || if active_role.get() == role {
                                                            "px-3 py-1.5 rounded text-sm font-medium bg-yellow-400 text-gray-900 transition-colors"
                                                        } else {
                                                            "px-3 py-1.5 rounded text-sm font-medium bg-gray-800 text-gray-300 hover:bg-gray-700 transition-colors"
                                                        }
                                                        on:click=move |_| set_active_role.set(role)
                                                    >
                                                        {role}
                                                    </button>
                                                }
                                            }).collect_view()}
                                        </div>

                                        // Champions for active role
                                        <div class="mb-4 min-h-16">
                                            <Suspense fallback=|| view! { <div class="text-gray-500 text-sm">"Loading pool..."</div> }>
                                                {move || pool_resource.get().map(|result| match result {
                                                    Ok(pool) => {
                                                        let role = active_role.get();
                                                        let role_entries: Vec<ChampionPoolEntry> = pool.into_iter()
                                                            .filter(|e| e.role == role)
                                                            .collect();

                                                        if role_entries.is_empty() {
                                                            view! {
                                                                <p class="text-gray-500 text-sm">"No champions in pool for this role yet."</p>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="flex flex-wrap gap-2">
                                                                    {role_entries.into_iter().map(|entry| {
                                                                        let champ = entry.champion.clone();
                                                                        let role_for_remove = entry.role.clone();
                                                                        let champ_for_remove = champ.clone();

                                                                        // Get image URL from champions resource
                                                                        let img_url = champions_resource.get()
                                                                            .and_then(|r| r.ok())
                                                                            .and_then(|champs| champs.into_iter().find(|c| c.name == champ))
                                                                            .map(|c| c.image_full)
                                                                            .unwrap_or_default();

                                                                        view! {
                                                                            <div class="flex items-center gap-1 bg-gray-800 border border-gray-700 rounded px-2 py-1">
                                                                                {if !img_url.is_empty() {
                                                                                    view! {
                                                                                        <img src=img_url alt=champ.clone() class="w-6 h-6 rounded object-cover" />
                                                                                    }.into_any()
                                                                                } else {
                                                                                    view! { <span></span> }.into_any()
                                                                                }}
                                                                                <span class="text-white text-sm">{champ}</span>
                                                                                <button
                                                                                    class="text-gray-500 hover:text-red-400 ml-1 leading-none transition-colors"
                                                                                    title="Remove from pool"
                                                                                    on:click=move |_| {
                                                                                        let c = champ_for_remove.clone();
                                                                                        let r = role_for_remove.clone();
                                                                                        leptos::task::spawn_local(async move {
                                                                                            match remove_champion_from_pool(c, r).await {
                                                                                                Ok(_) => pool_resource.refetch(),
                                                                                                Err(e) => set_pool_error.set(Some(e.to_string())),
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                >
                                                                                    "×"
                                                                                </button>
                                                                            </div>
                                                                        }
                                                                    }).collect_view()}
                                                                </div>
                                                            }.into_any()
                                                        }
                                                    }
                                                    Err(e) => view! {
                                                        <p class="text-red-400 text-sm">{e.to_string()}</p>
                                                    }.into_any(),
                                                })}
                                            </Suspense>
                                        </div>

                                        // Add champion form
                                        {move || pool_error.get().map(|e| view! {
                                            <p class="text-red-400 text-sm mb-2">{e}</p>
                                        })}
                                        <div class="flex gap-2">
                                            <input
                                                type="text"
                                                placeholder="Champion name..."
                                                prop:value=move || add_input.get()
                                                on:input=move |ev| set_add_input.set(event_target_value(&ev))
                                                class="flex-1 bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white placeholder-gray-400 focus:outline-none focus:border-yellow-400"
                                            />
                                            <button
                                                class="bg-yellow-400 hover:bg-yellow-300 text-gray-900 font-bold rounded px-4 py-2 transition-colors"
                                                on:click=move |_| {
                                                    let champion = add_input.get_untracked();
                                                    let role = active_role.get_untracked().to_string();
                                                    if champion.trim().is_empty() { return; }
                                                    leptos::task::spawn_local(async move {
                                                        match add_champion_to_pool(champion, role).await {
                                                            Ok(_) => {
                                                                set_add_input.set(String::new());
                                                                set_pool_error.set(None);
                                                                pool_resource.refetch();
                                                            }
                                                            Err(e) => set_pool_error.set(Some(e.to_string())),
                                                        }
                                                    });
                                                }
                                            >
                                                "Add"
                                            </button>
                                        </div>
                                    </section>

                                    // Logout
                                    <section>
                                        <ActionForm action=logout_action>
                                            <button
                                                type="submit"
                                                class="bg-red-700 hover:bg-red-600 text-white font-bold rounded px-4 py-2 transition-colors w-full"
                                            >
                                                "Log Out"
                                            </button>
                                        </ActionForm>
                                    </section>
                                </div>
                            }.into_any()
                        }
                        Ok(None) => view! {
                            <p class="text-gray-400">"You are not logged in."</p>
                        }.into_any(),
                        Err(e) => view! {
                            <p class="text-red-400">{e.to_string()}</p>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
