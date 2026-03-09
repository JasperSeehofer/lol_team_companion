use leptos::prelude::*;
use crate::models::team::Team;
use crate::models::user::TeamMember;

#[server]
pub async fn get_team_dashboard() -> Result<Option<(Team, Vec<TeamMember>)>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn TeamDashboard() -> impl IntoView {
    let dashboard = Resource::new(|| (), |_| get_team_dashboard());

    view! {
        <div class="max-w-4xl mx-auto py-8 px-6">
            <h1 class="text-3xl font-bold text-white mb-6">"Team Dashboard"</h1>
            <Suspense fallback=|| view! { <div class="text-gray-400">"Loading..."</div> }>
                {move || dashboard.get().map(|result| match result {
                    Ok(Some((team, members))) => view! {
                        <div class="flex flex-col gap-6">
                            <div class="bg-gray-800 border border-gray-700 rounded-lg p-6">
                                <h2 class="text-xl font-bold text-yellow-400 mb-1">{team.name}</h2>
                                <p class="text-gray-400 text-sm">"Region: " {team.region}</p>
                            </div>
                            <div>
                                <h3 class="text-lg font-semibold text-white mb-3">"Roster"</h3>
                                <div class="flex flex-col gap-2">
                                    {members.into_iter().map(|m| view! {
                                        <div class="bg-gray-800 border border-gray-700 rounded px-4 py-3 flex items-center justify-between">
                                            <span class="text-white">{m.username}</span>
                                            <span class="text-gray-400 text-sm capitalize">{m.role}</span>
                                        </div>
                                    }).collect_view()}
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    Ok(None) => view! {
                        <div class="text-center py-16">
                            <p class="text-gray-400 mb-4">"You are not part of a team yet."</p>
                            <a href="/team/roster" class="bg-yellow-400 hover:bg-yellow-300 text-gray-900 font-bold rounded px-4 py-2">
                                "Create a Team"
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
