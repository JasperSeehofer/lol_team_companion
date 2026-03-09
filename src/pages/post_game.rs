use leptos::prelude::*;

#[server]
pub async fn save_post_game(
    what_went_well: String,
    improvements: String,
    action_items: String,
) -> Result<(), ServerFnError> {
    use crate::models::game_plan::PostGameLearning;
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

    let learning = PostGameLearning {
        id: None,
        match_id: None,
        team_id,
        what_went_well: what_went_well
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        improvements: improvements
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        action_items: action_items
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        created_by: user.id,
    };

    db::save_post_game_learning(&db, learning)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn PostGamePage() -> impl IntoView {
    let save = ServerAction::<SavePostGame>::new();

    view! {
        <div class="max-w-3xl mx-auto py-8 px-6 flex flex-col gap-6">
            <h1 class="text-3xl font-bold text-white">"Post-Game Learnings"</h1>
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
                                "Post-game learning saved!"
                            </div>
                        })}

                        <div>
                            <label class="block text-gray-300 text-sm mb-1">
                                "What Went Well (one per line)"
                            </label>
                            <textarea
                                name="what_went_well"
                                rows="4"
                                placeholder="Good dragon control\nSupport roaming paid off"
                                class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-yellow-400"
                            />
                        </div>

                        <div>
                            <label class="block text-gray-300 text-sm mb-1">
                                "Improvements (one per line)"
                            </label>
                            <textarea
                                name="improvements"
                                rows="4"
                                placeholder="Ward coverage around Baron\nBetter grouping after first tower"
                                class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-yellow-400"
                            />
                        </div>

                        <div>
                            <label class="block text-gray-300 text-sm mb-1">
                                "Action Items (one per line)"
                            </label>
                            <textarea
                                name="action_items"
                                rows="3"
                                placeholder="Review VOD of mid-game fights\nPractice 2v2 bot lane"
                                class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-yellow-400"
                            />
                        </div>

                        <button
                            type="submit"
                            class="bg-yellow-400 hover:bg-yellow-300 text-gray-900 font-bold rounded px-4 py-2 transition-colors"
                        >
                            "Save Learnings"
                        </button>
                    </div>
                </ActionForm>
            </div>
        </div>
    }
}
