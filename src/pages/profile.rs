use leptos::prelude::*;
use crate::models::user::PublicUser;

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

#[component]
pub fn ProfilePage() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());
    let update_profile_action = ServerAction::<UpdateProfile>::new();
    let link_riot = ServerAction::<crate::pages::team::roster::LinkRiotAccount>::new();
    let logout_action = ServerAction::<Logout>::new();

    view! {
        <div class="max-w-lg mx-auto py-8 px-6">
            <h1 class="text-3xl font-bold text-white mb-8">"Profile"</h1>
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading..."</p> }>
                {move || Suspend::new(async move {
                    match user.await {
                        Ok(Some(u)) => {
                            let username = u.username.clone();
                            let riot_name = u.riot_summoner_name.clone();
                            view! {
                                <div class="flex flex-col gap-6">
                                    // Profile info
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
