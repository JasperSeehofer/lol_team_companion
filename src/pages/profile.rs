use crate::components::ui::{EmptyState, SkeletonCard, SkeletonGrid, ToastContext, ToastKind};
use crate::models::champion::ChampionPoolEntry;
use crate::models::user::PublicUser;
use leptos::prelude::*;
use leptos::web_sys;

#[server]
pub async fn get_current_user() -> Result<Option<PublicUser>, ServerFnError> {
    use crate::server::auth::AuthSession;

    let auth: AuthSession = leptos_axum::extract().await?;
    Ok(auth.user.map(|u| PublicUser {
        id: u.id,
        username: u.username,
        riot_summoner_name: u.riot_summoner_name,
        mode: u.mode,
        riot_region: u.riot_region,
    }))
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use leptos_axum::redirect;

    let mut auth: AuthSession = leptos_axum::extract().await?;
    auth.logout()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    redirect("/");
    Ok(())
}

#[server]
pub async fn update_profile(username: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let user_key = user
        .id
        .strip_prefix("user:")
        .unwrap_or(&user.id)
        .to_string();
    db.query("UPDATE type::record('user', $user_key) SET username = $username")
        .bind(("user_key", user_key))
        .bind(("username", username))
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .check()
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
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::get_champion_pool(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn save_region(region: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;
    db::set_user_region(&db, &user.id, &region)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

const REGIONS: &[(&str, &str)] = &[
    ("EUW", "EUW (Europe West)"),
    ("EUNE", "EUNE (Europe Nordic & East)"),
    ("NA", "NA (North America)"),
    ("KR", "KR (Korea)"),
    ("BR", "BR (Brazil)"),
    ("LAN", "LAN (Latin America North)"),
    ("LAS", "LAS (Latin America South)"),
    ("OCE", "OCE (Oceania)"),
    ("TR", "TR (Turkey)"),
    ("RU", "RU (Russia)"),
    ("JP", "JP (Japan)"),
    ("SG", "SG (Singapore)"),
    ("TW", "TW (Taiwan)"),
    ("VN", "VN (Vietnam)"),
    ("ME", "ME (Middle East)"),
];

const POOL_ROLES: &[&str] = &["Top", "Jungle", "Mid", "ADC", "Support"];

#[component]
pub fn ProfilePage() -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let user = Resource::new(|| (), |_| get_current_user());
    let update_profile_action = ServerAction::<UpdateProfile>::new();
    let link_riot = ServerAction::<crate::pages::team::roster::LinkRiotAccount>::new();
    let logout_action = ServerAction::<Logout>::new();

    let pool_resource = Resource::new(|| (), |_| get_champion_pool());

    // Toast feedback for update_profile
    Effect::new(move || {
        if let Some(result) = update_profile_action.value().get() {
            match result {
                Ok(()) => toast.show.run((ToastKind::Success, "Profile updated".into())),
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        }
    });

    // Toast feedback for link_riot
    Effect::new(move || {
        if let Some(result) = link_riot.value().get() {
            match result {
                Ok(()) => toast.show.run((ToastKind::Success, "Riot account linked".into())),
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        }
    });

    // Hard-navigate on logout so session state is fully cleared
    Effect::new(move || {
        if let Some(Ok(())) = logout_action.value().get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/");
            }
        }
    });

    // Redirect to login if not authenticated
    Effect::new(move || {
        if let Some(Ok(None)) = user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    view! {
        <div class="max-w-2xl mx-auto py-8 px-6">
            <h1 class="text-3xl font-bold text-primary mb-8">"Profile"</h1>
            <Suspense fallback=move || view! { <SkeletonCard height="h-20" /> }>
                {move || Suspend::new(async move {
                    match user.await {
                        Ok(Some(u)) => {
                            let username = u.username.clone();
                            let riot_name = u.riot_summoner_name.clone();
                            let initial_region = u.riot_region.clone().unwrap_or_default();
                            let (editing_username, set_editing_username) = signal(false);
                            let username_for_edit = username.clone();
                            view! {
                                <div class="flex flex-col gap-6">
                                    // Account info
                                    <section class="bg-surface border border-divider rounded-lg p-6">
                                        <h2 class="text-xl font-semibold text-primary mb-4">"Account"</h2>


                                        <div>
                                            <label class="block text-secondary text-sm mb-1">"Username"</label>
                                            {move || {
                                                if editing_username.get() {
                                                    let username_val = username_for_edit.clone();
                                                    view! {
                                                        <div class="flex gap-2 items-center">
                                                            <ActionForm action=update_profile_action>
                                                                <div class="flex gap-2 items-center">
                                                                    <input
                                                                        type="text"
                                                                        name="username"
                                                                        value=username_val
                                                                        required
                                                                        class="bg-elevated border border-outline rounded px-3 py-2 text-primary focus:outline-none focus:border-accent"
                                                                    />
                                                                    <button
                                                                        type="submit"
                                                                        class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-2 py-2 text-sm transition-colors"
                                                                    >
                                                                        "Save"
                                                                    </button>
                                                                </div>
                                                            </ActionForm>
                                                            <button
                                                                class="text-muted hover:text-secondary transition-colors cursor-pointer"
                                                                on:click=move |_| set_editing_username.set(false)
                                                            >
                                                                "Cancel"
                                                            </button>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    let username_display = username_for_edit.clone();
                                                    view! {
                                                        <div class="flex items-center gap-2">
                                                            <span class="text-primary text-lg">{username_display}</span>
                                                            <button
                                                                class="text-muted hover:text-accent transition-colors cursor-pointer"
                                                                title="Edit username"
                                                                on:click=move |_| set_editing_username.set(true)
                                                            >
                                                                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                                                                    <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z"/>
                                                                </svg>
                                                            </button>
                                                        </div>
                                                    }.into_any()
                                                }
                                            }}
                                        </div>
                                    </section>

                                    // Riot account linking
                                    <section class="bg-surface border border-divider rounded-lg p-6">
                                        <h2 class="text-xl font-semibold text-primary mb-4">"Riot Account"</h2>

                                        {match riot_name {
                                            Some(name) => view! {
                                                <p class="text-green-400 text-sm mb-4">
                                                    "Linked: "
                                                    <span class="font-semibold">{name}</span>
                                                </p>
                                            }.into_any(),
                                            None => view! {
                                                <EmptyState
                                                    icon="🔗"
                                                    message="Link your Riot account to track match stats and see champion performance across the app"
                                                />
                                            }.into_any(),
                                        }}

                                        // Region dropdown
                                        <div class="mb-4">
                                            <label class="block text-secondary text-sm mb-1">"Region"</label>
                                            <select
                                                class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-sm text-primary w-full max-w-xs cursor-pointer"
                                                on:change=move |ev| {
                                                    let region = event_target_value(&ev);
                                                    leptos::task::spawn_local(async move {
                                                        match save_region(region).await {
                                                            Ok(()) => toast.show.run((ToastKind::Success, "Region saved".into())),
                                                            Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                                                        }
                                                    });
                                                }
                                            >
                                                <option value="" selected={initial_region.is_empty()}>
                                                    "Select your region"
                                                </option>
                                                {REGIONS.iter().map(|&(value, label)| {
                                                    let sel = initial_region.clone();
                                                    view! {
                                                        <option value=value selected=move || sel == value>
                                                            {label}
                                                        </option>
                                                    }
                                                }).collect_view()}
                                            </select>
                                        </div>

                                        <ActionForm action=link_riot>
                                            <div class="flex flex-col gap-4">
                                                <div>
                                                    <label class="block text-secondary text-sm mb-1">
                                                        "Riot ID (e.g. PlayerName#EUW)"
                                                    </label>
                                                    <input
                                                        type="text"
                                                        name="riot_id"
                                                        placeholder="GameName#TAG"
                                                        required
                                                        class="w-full bg-elevated border border-outline rounded px-3 py-2 text-primary focus:outline-none focus:border-accent"
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

                                    // Champion Pool summary + link
                                    <section class="bg-surface border border-divider rounded-lg p-6">
                                        <div class="flex items-center justify-between mb-3">
                                            <div>
                                                <h2 class="text-xl font-semibold text-primary">"Champion Pool"</h2>
                                                <p class="text-muted text-sm">"Champions you play, organized by role and tier."</p>
                                            </div>
                                            <a href="/champion-pool" class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-4 py-2 text-sm transition-colors">
                                                "Manage Pool"
                                            </a>
                                        </div>
                                        <Suspense fallback=|| view! { <SkeletonGrid cols=4 rows=2 card_height="h-16" /> }>
                                            {move || pool_resource.get().map(|result| match result {
                                                Ok(pool) => {
                                                    if pool.is_empty() {
                                                        view! { <p class="text-dimmed text-sm">"No champions in pool yet."</p> }.into_any()
                                                    } else {
                                                        let counts: Vec<(&str, usize)> = POOL_ROLES.iter().map(|&role| {
                                                            let count = pool.iter().filter(|e| e.role == role).count();
                                                            (role, count)
                                                        }).filter(|(_, c)| *c > 0).collect();
                                                        view! {
                                                            <div class="flex gap-3">
                                                                {counts.into_iter().map(|(role, count)| view! {
                                                                    <div class="bg-elevated border border-divider rounded px-3 py-1.5">
                                                                        <span class="text-muted text-xs">{role}</span>
                                                                        <span class="text-primary text-sm font-medium ml-1">{count}</span>
                                                                    </div>
                                                                }).collect_view()}
                                                            </div>
                                                        }.into_any()
                                                    }
                                                }
                                                Err(e) => view! { <p class="text-red-400 text-sm">{e.to_string()}</p> }.into_any(),
                                            })}
                                        </Suspense>
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
                            <p class="text-muted">"You are not logged in."</p>
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
