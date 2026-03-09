use leptos::prelude::*;
use crate::models::match_data::PlayerMatchStats;
use crate::components::stat_card::StatCard;

#[server]
pub async fn get_my_stats() -> Result<Vec<PlayerMatchStats>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::get_player_stats(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn trigger_stats_sync() -> Result<usize, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use crate::server::riot;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let puuid = user.riot_puuid.ok_or_else(|| ServerFnError::new("No Riot account linked"))?;

    let matches = riot::fetch_match_history(&puuid, 440)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let count = matches.len();
    db::store_matches(&db, &user.id, matches)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(count)
}

#[component]
pub fn StatsPage() -> impl IntoView {
    let stats = Resource::new(|| (), |_| get_my_stats());
    let sync = ServerAction::<TriggerStatsSync>::new();

    view! {
        <div class="max-w-4xl mx-auto py-8 px-6 flex flex-col gap-6">
            <div class="flex items-center justify-between">
                <h1 class="text-3xl font-bold text-white">"My Stats"</h1>
                <ActionForm action=sync>
                    <button
                        type="submit"
                        class="bg-blue-500 hover:bg-blue-400 text-white font-bold rounded px-4 py-2 text-sm transition-colors"
                    >
                        "Sync Match History"
                    </button>
                </ActionForm>
            </div>

            {move || sync.value().get().map(|r| match r {
                Ok(n) => view! { <div class="text-green-300 text-sm">"Synced " {n} " matches."</div> }.into_any(),
                Err(e) => view! { <div class="text-red-400 text-sm">"Sync error: " {e.to_string()}</div> }.into_any(),
            })}

            <Suspense fallback=|| view! { <div class="text-gray-400">"Loading stats..."</div> }>
                {move || stats.get().map(|result| match result {
                    Ok(s) => {
                        let total = s.len().to_string();
                        let kda = if s.is_empty() {
                            "N/A".to_string()
                        } else {
                            let k: f32 = s.iter().map(|m| m.kills as f32).sum::<f32>() / s.len() as f32;
                            let d: f32 = s.iter().map(|m| m.deaths as f32).sum::<f32>() / s.len() as f32;
                            let a: f32 = s.iter().map(|m| m.assists as f32).sum::<f32>() / s.len() as f32;
                            format!("{:.1}/{:.1}/{:.1}", k, d, a)
                        };
                        let wr = if s.is_empty() {
                            "N/A".to_string()
                        } else {
                            let wins = s.iter().filter(|m| m.win).count();
                            format!("{:.0}%", wins as f32 / s.len() as f32 * 100.0)
                        };
                        view! {
                            <div class="flex flex-col gap-6">
                                <div class="grid grid-cols-3 gap-4">
                                    <StatCard label="Games Played".to_string() value=total />
                                    <StatCard label="Avg KDA".to_string() value=kda />
                                    <StatCard label="Win Rate".to_string() value=wr />
                                </div>
                                <div>
                                    <h2 class="text-lg font-semibold text-white mb-3">"Recent Matches"</h2>
                                    <div class="flex flex-col gap-2">
                                        {s.into_iter().map(|m| {
                                            let result_color = if m.win { "text-green-400" } else { "text-red-400" };
                                            let result_text = if m.win { "Win" } else { "Loss" };
                                            view! {
                                                <div class="bg-gray-800 border border-gray-700 rounded px-4 py-3 flex items-center justify-between">
                                                    <span class="text-white">{m.champion}</span>
                                                    <span class="text-gray-300 text-sm">
                                                        {m.kills} "/" {m.deaths} "/" {m.assists}
                                                    </span>
                                                    <span class=result_color>{result_text}</span>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    },
                    Err(e) => view! {
                        <div class="text-red-400">"Error: " {e.to_string()}</div>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
