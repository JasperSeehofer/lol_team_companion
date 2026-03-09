use leptos::prelude::*;
use crate::models::draft::{Draft, DraftAction};
use crate::components::draft_board::DraftBoard;

#[server]
pub async fn save_draft(
    name: String,
    opponent: Option<String>,
    notes: Option<String>,
    actions_json: String,
) -> Result<String, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let actions: Vec<DraftAction> = serde_json::from_str(&actions_json)
        .map_err(|e| ServerFnError::new(format!("Invalid actions JSON: {e}")))?;

    let team_id = db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("You must be in a team to create a draft"))?;

    db::save_draft(&db, &team_id, &user.id, name, opponent, notes, actions)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_drafts() -> Result<Vec<Draft>, ServerFnError> {
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
        .ok_or_else(|| ServerFnError::new("No team"))?;

    db::list_drafts(&db, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn DraftPage() -> impl IntoView {
    let (draft_name, set_draft_name) = signal(String::new());
    let (opponent, set_opponent) = signal(String::new());
    let (notes, set_notes) = signal(String::new());
    let (actions, set_actions) = signal(Vec::<DraftAction>::new());
    let (save_result, set_save_result) = signal(Option::<String>::None);

    let drafts = Resource::new(|| (), |_| list_drafts());

    let add_action = move |side: &'static str, phase: &'static str, champion: &'static str| {
        let order = actions.get_untracked().len() as i32;
        set_actions.update(|a| a.push(DraftAction {
            id: None,
            draft_id: String::new(),
            phase: phase.to_string(),
            side: side.to_string(),
            champion: champion.to_string(),
            order,
        }));
    };

    view! {
        <div class="max-w-5xl mx-auto py-8 px-6 flex flex-col gap-8">
            <h1 class="text-3xl font-bold text-white">"Draft Planner"</h1>

            <div class="bg-gray-800 border border-gray-700 rounded-lg p-6 flex flex-col gap-4">
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-gray-300 text-sm mb-1">"Draft Name"</label>
                        <input
                            type="text"
                            class="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white focus:outline-none focus:border-yellow-400"
                            on:input=move |ev| set_draft_name.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-gray-300 text-sm mb-1">"Opponent (optional)"</label>
                        <input
                            type="text"
                            class="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white focus:outline-none focus:border-yellow-400"
                            on:input=move |ev| set_opponent.set(event_target_value(&ev))
                        />
                    </div>
                </div>
                <div>
                    <label class="block text-gray-300 text-sm mb-1">"Notes"</label>
                    <textarea
                        rows="3"
                        class="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white focus:outline-none focus:border-yellow-400"
                        on:input=move |ev| set_notes.set(event_target_value(&ev))
                    />
                </div>

                <div class="flex gap-2 flex-wrap">
                    <button
                        class="bg-blue-700 hover:bg-blue-600 text-white text-sm rounded px-3 py-1"
                        on:click=move |_| add_action("blue", "ban1", "Zed")
                    >
                        "+ Blue Ban (Zed)"
                    </button>
                    <button
                        class="bg-red-700 hover:bg-red-600 text-white text-sm rounded px-3 py-1"
                        on:click=move |_| add_action("red", "ban1", "Yasuo")
                    >
                        "+ Red Ban (Yasuo)"
                    </button>
                    <button
                        class="bg-blue-500 hover:bg-blue-400 text-white text-sm rounded px-3 py-1"
                        on:click=move |_| add_action("blue", "pick1", "Jinx")
                    >
                        "+ Blue Pick (Jinx)"
                    </button>
                    <button
                        class="bg-red-500 hover:bg-red-400 text-white text-sm rounded px-3 py-1"
                        on:click=move |_| add_action("red", "pick1", "Caitlyn")
                    >
                        "+ Red Pick (Caitlyn)"
                    </button>
                    <button
                        class="bg-gray-600 hover:bg-gray-500 text-white text-sm rounded px-3 py-1"
                        on:click=move |_| set_actions.set(Vec::new())
                    >
                        "Clear"
                    </button>
                </div>

                {move || {
                    let acts = actions.get();
                    if acts.is_empty() {
                        view! { <p class="text-gray-500 text-sm">"Add picks and bans above to preview."</p> }.into_any()
                    } else {
                        view! { <DraftBoard actions=acts /> }.into_any()
                    }
                }}

                <button
                    class="bg-yellow-400 hover:bg-yellow-300 text-gray-900 font-bold rounded px-4 py-2 transition-colors"
                    on:click=move |_| {
                        let name = draft_name.get_untracked();
                        let opp = opponent.get_untracked();
                        let n = notes.get_untracked();
                        let acts = serde_json::to_string(&actions.get_untracked()).unwrap_or_default();
                        leptos::task::spawn_local(async move {
                            match save_draft(
                                name,
                                if opp.is_empty() { None } else { Some(opp) },
                                if n.is_empty() { None } else { Some(n) },
                                acts,
                            ).await {
                                Ok(id) => set_save_result.set(Some(format!("Saved! ID: {id}"))),
                                Err(e) => set_save_result.set(Some(format!("Error: {e}"))),
                            }
                        });
                    }
                >
                    "Save Draft"
                </button>

                {move || save_result.get().map(|msg| view! {
                    <div class="text-green-300 text-sm">{msg}</div>
                })}
            </div>

            <div>
                <h2 class="text-xl font-bold text-white mb-3">"Saved Drafts"</h2>
                <Suspense fallback=|| view! { <div class="text-gray-400">"Loading..."</div> }>
                    {move || drafts.get().map(|result| match result {
                        Ok(list) if list.is_empty() => view! {
                            <p class="text-gray-500">"No drafts yet."</p>
                        }.into_any(),
                        Ok(list) => view! {
                            <div class="flex flex-col gap-2">
                                {list.into_iter().map(|d| view! {
                                    <div class="bg-gray-800 border border-gray-700 rounded px-4 py-3">
                                        <span class="text-white font-medium">{d.name}</span>
                                        {d.opponent.map(|o| view! {
                                            <span class="text-gray-400 text-sm ml-2">"vs " {o}</span>
                                        })}
                                    </div>
                                }).collect_view()}
                            </div>
                        }.into_any(),
                        Err(e) => view! {
                            <div class="text-red-400">"Error: " {e.to_string()}</div>
                        }.into_any(),
                    })}
                </Suspense>
            </div>
        </div>
    }
}
