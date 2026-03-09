use leptos::prelude::*;

#[server]
pub async fn save_game_plan(
    win_conditions: String,
    objective_priority: String,
    teamfight_strategy: String,
    early_game: String,
    notes: String,
) -> Result<(), ServerFnError> {
    use crate::models::game_plan::GamePlan;
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not in a team"))?;

    let plan = GamePlan {
        id: None,
        draft_id: None,
        team_id,
        win_conditions: win_conditions
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        objective_priority: objective_priority
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        teamfight_strategy,
        early_game: if early_game.is_empty() { None } else { Some(early_game) },
        notes: if notes.is_empty() { None } else { Some(notes) },
    };

    db::save_game_plan(&db, plan)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn GamePlanPage() -> impl IntoView {
    let save = ServerAction::<SaveGamePlan>::new();

    view! {
        <div class="max-w-3xl mx-auto py-8 px-6 flex flex-col gap-6">
            <h1 class="text-3xl font-bold text-white">"Game Plan"</h1>
            <div>
                <ActionForm action=save>
                    <div class="flex flex-col gap-5">
                        {move || save.value().get().and_then(|r| r.err()).map(|e| view! {
                            <div class="bg-red-900 border border-red-700 text-red-200 rounded px-4 py-3 text-sm">
                                {e.to_string()}
                            </div>
                        })}
                        {move || save.value().get().and_then(|r| r.ok()).map(|_| view! {
                            <div class="bg-green-900 border border-green-700 text-green-200 rounded px-4 py-3 text-sm">
                                "Game plan saved!"
                            </div>
                        })}

                        <div>
                            <label class="block text-gray-300 text-sm mb-1">
                                "Win Conditions (one per line)"
                            </label>
                            <textarea
                                name="win_conditions"
                                rows="4"
                                placeholder="Force early teamfights\nControl Dragon side"
                                class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-yellow-400"
                            />
                        </div>

                        <div>
                            <label class="block text-gray-300 text-sm mb-1">
                                "Objective Priority (one per line)"
                            </label>
                            <textarea
                                name="objective_priority"
                                rows="3"
                                placeholder="Dragon\nRift Herald\nBaron"
                                class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-yellow-400"
                            />
                        </div>

                        <div>
                            <label class="block text-gray-300 text-sm mb-1">
                                "Teamfight Strategy"
                            </label>
                            <textarea
                                name="teamfight_strategy"
                                rows="3"
                                placeholder="Peel for ADC and win extended trades..."
                                class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-yellow-400"
                            />
                        </div>

                        <div>
                            <label class="block text-gray-300 text-sm mb-1">
                                "Early Game Plan (optional)"
                            </label>
                            <textarea
                                name="early_game"
                                rows="2"
                                class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white focus:outline-none focus:border-yellow-400"
                            />
                        </div>

                        <div>
                            <label class="block text-gray-300 text-sm mb-1">"Notes (optional)"</label>
                            <textarea
                                name="notes"
                                rows="2"
                                class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white focus:outline-none focus:border-yellow-400"
                            />
                        </div>

                        <button
                            type="submit"
                            class="bg-yellow-400 hover:bg-yellow-300 text-gray-900 font-bold rounded px-4 py-2 transition-colors"
                        >
                            "Save Game Plan"
                        </button>
                    </div>
                </ActionForm>
            </div>
        </div>
    }
}
