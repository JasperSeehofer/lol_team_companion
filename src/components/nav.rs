use leptos::prelude::*;
use leptos_router::components::A;

use crate::pages::profile::{get_current_user, Logout};

#[server]
pub async fn get_pending_request_count() -> Result<usize, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = match auth.user {
        Some(u) => u,
        None => return Ok(0),
    };
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let result = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let (team, _) = match result {
        Some(t) => t,
        None => return Ok(0),
    };

    if team.created_by != user.id {
        return Ok(0); // only leaders see badge
    }

    let team_id = match team.id {
        Some(id) => id,
        None => return Ok(0),
    };

    db::count_pending_join_requests(&db, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn Nav() -> impl IntoView {
    let logout_action = ServerAction::<Logout>::new();
    let menu_open = RwSignal::new(false);

    // Refetch current user whenever logout action completes
    let logout_version = logout_action.version();
    let user = Resource::new(move || logout_version.get(), |_| get_current_user());

    let request_count = Resource::new(|| (), |_| get_pending_request_count());

    // Close menu when logout finishes
    Effect::new(move || {
        if logout_version.get() > 0 {
            menu_open.set(false);
        }
    });

    view! {
        <nav class="bg-gray-900 border-b border-gray-700 px-6 py-3 flex items-center justify-between">
            <div class="flex items-center gap-6">
                <span class="text-yellow-400 font-bold text-lg tracking-wide">
                    "LoL Team Companion"
                </span>
                <div class="flex gap-4 text-sm">
                    <A href="/" attr:class="text-gray-300 hover:text-white transition-colors">
                        "Home"
                    </A>
                    <span class="relative inline-flex items-center">
                        <A href="/team/dashboard" attr:class="text-gray-300 hover:text-white transition-colors">
                            "Team"
                        </A>
                        <Suspense fallback=|| view! { <span></span> }>
                            {move || request_count.get().map(|res| {
                                let n = res.unwrap_or(0);
                                if n > 0 {
                                    view! {
                                        <span class="absolute -top-2 -right-3 bg-red-500 text-white text-[10px] font-bold rounded-full w-4 h-4 flex items-center justify-center leading-none">
                                            {n}
                                        </span>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }
                            })}
                        </Suspense>
                    </span>
                    <A href="/draft" attr:class="text-gray-300 hover:text-white transition-colors">
                        "Draft"
                    </A>
                    <A href="/tree-drafter" attr:class="text-gray-300 hover:text-white transition-colors">
                        "Tree Drafter"
                    </A>
                    <A href="/stats" attr:class="text-gray-300 hover:text-white transition-colors">
                        "Stats"
                    </A>
                    <A href="/game-plan" attr:class="text-gray-300 hover:text-white transition-colors">
                        "Game Plan"
                    </A>
                    <A href="/post-game" attr:class="text-gray-300 hover:text-white transition-colors">
                        "Post Game"
                    </A>
                </div>
            </div>
            <div class="relative text-sm">
                <Suspense fallback=move || view! {
                    <span class="text-gray-500">"..."</span>
                }>
                    {move || Suspend::new(async move {
                        match user.await {
                            Ok(Some(u)) => view! {
                                <button
                                    on:click=move |_| menu_open.update(|v| *v = !*v)
                                    class="text-yellow-400 hover:text-yellow-300 font-semibold transition-colors cursor-pointer"
                                >
                                    {u.username}
                                </button>
                                <div
                                    class="absolute right-0 mt-2 w-40 bg-gray-800 border border-gray-700 rounded-lg shadow-lg py-1 z-50"
                                    style:display=move || if menu_open.get() { "block" } else { "none" }
                                >
                                    <A
                                        href="/profile"
                                        attr:class="block px-4 py-2 text-gray-300 hover:bg-gray-700 hover:text-white transition-colors"
                                    >
                                        "Profile Details"
                                    </A>
                                    <ActionForm action=logout_action>
                                        <button
                                            type="submit"
                                            class="block w-full text-left px-4 py-2 text-red-400 hover:bg-gray-700 hover:text-red-300 transition-colors cursor-pointer"
                                        >
                                            "Logout"
                                        </button>
                                    </ActionForm>
                                </div>
                            }.into_any(),
                            _ => view! {
                                <A href="/auth/login" attr:class="text-gray-300 hover:text-white transition-colors">
                                    "Login"
                                </A>
                                <span class="text-gray-600">"/"</span>
                                <A href="/auth/register" attr:class="text-gray-300 hover:text-white transition-colors">
                                    "Register"
                                </A>
                            }.into_any(),
                        }
                    })}
                </Suspense>
            </div>
        </nav>
    }
}
