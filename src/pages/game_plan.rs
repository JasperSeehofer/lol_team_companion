use leptos::prelude::*;
use crate::models::game_plan::GamePlan;
use crate::models::draft::Draft;
use crate::components::ui::{ErrorBanner, StatusMessage};

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn list_plans() -> Result<Vec<GamePlan>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    db::list_game_plans(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_team_drafts() -> Result<Vec<Draft>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    db::list_drafts(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_plan(plan_json: String) -> Result<String, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    let mut plan: GamePlan = serde_json::from_str(&plan_json)
        .map_err(|e| ServerFnError::new(format!("Invalid plan JSON: {e}")))?;
    plan.team_id = team_id;

    db::save_game_plan(&surreal, plan, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_plan(plan_json: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let plan: GamePlan = serde_json::from_str(&plan_json)
        .map_err(|e| ServerFnError::new(format!("Invalid plan JSON: {e}")))?;

    db::update_game_plan(&surreal, plan)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_plan(plan_id: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::delete_game_plan(&surreal, &plan_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ---------------------------------------------------------------------------
// Template generation
// ---------------------------------------------------------------------------

fn generate_template(our: &[String], enemy: &[String]) -> (Vec<String>, Vec<String>, String, String) {
    let mut win_conditions = Vec::new();
    let objectives = vec!["Dragon".to_string(), "Rift Herald".to_string(), "Baron".to_string()];

    // Simple heuristic based on champion count (placeholder for real analysis)
    if !our.is_empty() {
        win_conditions.push(format!("Play around {} in teamfights", our.first().unwrap_or(&"carry".to_string())));
    }
    if our.len() >= 3 {
        win_conditions.push("Control vision around neutral objectives before fights".to_string());
    }
    if !enemy.is_empty() {
        win_conditions.push(format!("Deny {} from scaling", enemy.last().unwrap_or(&"enemy carry".to_string())));
    }
    if win_conditions.is_empty() {
        win_conditions.push("Secure early game advantages through proactive plays".to_string());
        win_conditions.push("Transition leads into objective control".to_string());
    }

    let teamfight = if our.len() >= 5 {
        format!("Front-to-back: protect {} while {} zones the enemy", our[3], our[0])
    } else {
        "Focus priority targets and peel for carries".to_string()
    };

    let early = "Contest level 1 vision. Jungle path based on matchup. Look for early ganks on volatile lanes.".to_string();

    (win_conditions, objectives, teamfight, early)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const ROLES: [&str; 5] = ["Top", "Jungle", "Mid", "Bot", "Support"];

fn textarea_class() -> &'static str {
    "w-full bg-gray-900/50 border border-gray-600/50 rounded-lg px-3 py-2 text-white text-sm placeholder-gray-500 focus:outline-none focus:border-yellow-400/50 resize-none transition-colors"
}

fn input_class() -> &'static str {
    "w-full bg-gray-900/50 border border-gray-600/50 rounded-lg px-3 py-2 text-white text-sm placeholder-gray-500 focus:outline-none focus:border-yellow-400/50 transition-colors"
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn GamePlanPage() -> impl IntoView {
    let plans = Resource::new(|| (), |_| list_plans());
    let drafts = Resource::new(|| (), |_| list_team_drafts());
    let (status_msg, set_status_msg) = signal(Option::<String>::None);

    // Editor state
    let (editing_id, set_editing_id) = signal(Option::<String>::None);
    let (plan_name, set_plan_name) = signal(String::new());
    let (draft_id, set_draft_id) = signal(String::new());
    let (our_champs, set_our_champs) = signal(vec![String::new(); 5]);
    let (enemy_champs, set_enemy_champs) = signal(vec![String::new(); 5]);
    let (win_conditions, set_win_conditions) = signal(String::new());
    let (obj_priority, set_obj_priority) = signal(String::new());
    let (teamfight, set_teamfight) = signal(String::new());
    let (early_game, set_early_game) = signal(String::new());
    let (role_strats, set_role_strats) = signal(vec![String::new(); 5]);
    let (notes, set_notes) = signal(String::new());

    // Clear editor
    let clear_editor = move || {
        set_editing_id.set(None);
        set_plan_name.set(String::new());
        set_draft_id.set(String::new());
        set_our_champs.set(vec![String::new(); 5]);
        set_enemy_champs.set(vec![String::new(); 5]);
        set_win_conditions.set(String::new());
        set_obj_priority.set(String::new());
        set_teamfight.set(String::new());
        set_early_game.set(String::new());
        set_role_strats.set(vec![String::new(); 5]);
        set_notes.set(String::new());
    };

    // Load a plan into editor
    let load_plan = move |p: &GamePlan| {
        set_editing_id.set(p.id.clone());
        set_plan_name.set(p.name.clone());
        set_draft_id.set(p.draft_id.clone().unwrap_or_default());
        let mut ours = p.our_champions.clone();
        ours.resize(5, String::new());
        set_our_champs.set(ours);
        let mut theirs = p.enemy_champions.clone();
        theirs.resize(5, String::new());
        set_enemy_champs.set(theirs);
        set_win_conditions.set(p.win_conditions.join("\n"));
        set_obj_priority.set(p.objective_priority.join("\n"));
        set_teamfight.set(p.teamfight_strategy.clone());
        set_early_game.set(p.early_game.clone().unwrap_or_default());
        set_role_strats.set(vec![
            p.top_strategy.clone().unwrap_or_default(),
            p.jungle_strategy.clone().unwrap_or_default(),
            p.mid_strategy.clone().unwrap_or_default(),
            p.bot_strategy.clone().unwrap_or_default(),
            p.support_strategy.clone().unwrap_or_default(),
        ]);
        set_notes.set(p.notes.clone().unwrap_or_default());
    };

    let build_plan = move || -> GamePlan {
        let strats = role_strats.get_untracked();
        GamePlan {
            id: editing_id.get_untracked(),
            team_id: String::new(), // filled by server
            draft_id: {
                let d = draft_id.get_untracked();
                if d.is_empty() { None } else { Some(d) }
            },
            name: plan_name.get_untracked(),
            our_champions: our_champs.get_untracked().into_iter().filter(|s| !s.is_empty()).collect(),
            enemy_champions: enemy_champs.get_untracked().into_iter().filter(|s| !s.is_empty()).collect(),
            win_conditions: win_conditions.get_untracked().lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
            objective_priority: obj_priority.get_untracked().lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
            teamfight_strategy: teamfight.get_untracked(),
            early_game: { let s = early_game.get_untracked(); if s.is_empty() { None } else { Some(s) } },
            top_strategy: { let s = strats[0].clone(); if s.is_empty() { None } else { Some(s) } },
            jungle_strategy: { let s = strats[1].clone(); if s.is_empty() { None } else { Some(s) } },
            mid_strategy: { let s = strats[2].clone(); if s.is_empty() { None } else { Some(s) } },
            bot_strategy: { let s = strats[3].clone(); if s.is_empty() { None } else { Some(s) } },
            support_strategy: { let s = strats[4].clone(); if s.is_empty() { None } else { Some(s) } },
            notes: { let s = notes.get_untracked(); if s.is_empty() { None } else { Some(s) } },
        }
    };

    let do_save = move |_| {
        let plan = build_plan();
        let plan_json = serde_json::to_string(&plan).unwrap_or_default();
        let is_update = editing_id.get_untracked().is_some();

        leptos::task::spawn_local(async move {
            let result = if is_update {
                update_plan(plan_json).await.map(|_| String::new())
            } else {
                create_plan(plan_json).await
            };
            match result {
                Ok(id) => {
                    if !is_update && !id.is_empty() {
                        set_editing_id.set(Some(id));
                    }
                    set_status_msg.set(Some(if is_update { "Plan updated!".into() } else { "Plan created!".into() }));
                    plans.refetch();
                }
                Err(e) => set_status_msg.set(Some(format!("Error: {e}"))),
            }
        });
    };

    let do_delete = move |plan_id: String| {
        leptos::task::spawn_local(async move {
            match delete_plan(plan_id).await {
                Ok(_) => {
                    clear_editor();
                    set_status_msg.set(Some("Plan deleted.".into()));
                    plans.refetch();
                }
                Err(e) => set_status_msg.set(Some(format!("Error: {e}"))),
            }
        });
    };

    let do_generate_template = move |_| {
        let ours = our_champs.get_untracked();
        let theirs = enemy_champs.get_untracked();
        let (wc, obj, tf, eg) = generate_template(&ours, &theirs);
        set_win_conditions.set(wc.join("\n"));
        set_obj_priority.set(obj.join("\n"));
        set_teamfight.set(tf);
        set_early_game.set(eg);
        set_status_msg.set(Some("Template generated! Customize to your strategy.".into()));
    };

    view! {
        <div class="max-w-[80rem] mx-auto py-8 px-6 flex flex-col gap-6">
            <div>
                <h1 class="text-3xl font-bold text-white">"Game Plans"</h1>
                <p class="text-gray-400 text-sm mt-1">"Strategic plans for specific champion matchups"</p>
            </div>

            {move || status_msg.get().map(|msg| {
                view! { <StatusMessage message=msg /> }
            })}

            <div class="flex gap-6 min-h-[36rem]">
                // Left: plan list
                <div class="w-72 flex-shrink-0 flex flex-col gap-3">
                    <button
                        class="bg-yellow-400 hover:bg-yellow-300 text-gray-900 font-semibold rounded-lg px-4 py-2 text-sm transition-colors"
                        on:click=move |_| clear_editor()
                    >"+ New Plan"</button>

                    <Suspense fallback=|| view! { <div class="text-gray-500 text-sm">"Loading..."</div> }>
                        {move || plans.get().map(|result| match result {
                            Ok(list) if list.is_empty() => view! {
                                <p class="text-gray-500 text-sm">"No plans yet."</p>
                            }.into_any(),
                            Ok(list) => view! {
                                <div class="flex flex-col gap-1.5">
                                    {list.into_iter().map(|p| {
                                        let plan_for_load = p.clone();
                                        let plan_id = p.id.clone().unwrap_or_default();
                                        let plan_id_for_cls = plan_id.clone();
                                        let plan_id_for_delete = plan_id.clone();
                                        let name = if p.name.is_empty() { "Untitled".to_string() } else { p.name.clone() };
                                        let champ_summary = if !p.our_champions.is_empty() && !p.enemy_champions.is_empty() {
                                            format!("{} vs {}", p.our_champions.len(), p.enemy_champions.len())
                                        } else {
                                            String::new()
                                        };

                                        view! {
                                            <div class=move || {
                                                let sel = editing_id.get();
                                                if sel.as_deref() == Some(&plan_id_for_cls) {
                                                    "bg-yellow-400/10 border border-yellow-400/30 rounded-lg p-3 transition-all"
                                                } else {
                                                    "bg-gray-800/30 border border-gray-700/30 rounded-lg p-3 hover:bg-gray-700/30 transition-all"
                                                }
                                            }>
                                                <button
                                                    class="w-full text-left"
                                                    on:click=move |_| load_plan(&plan_for_load)
                                                >
                                                    <div class="text-white text-sm font-medium truncate">{name}</div>
                                                    {(!champ_summary.is_empty()).then(|| view! {
                                                        <div class="text-gray-500 text-xs mt-0.5">{champ_summary} " champs"</div>
                                                    })}
                                                </button>
                                                <button
                                                    class="text-red-400/50 hover:text-red-400 text-xs mt-1 transition-colors"
                                                    on:click=move |_| do_delete(plan_id_for_delete.clone())
                                                >"Delete"</button>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any(),
                            Err(e) => view! {
                                <ErrorBanner message=format!("Failed to load plans: {e}") />
                            }.into_any(),
                        })}
                    </Suspense>
                </div>

                // Right: editor
                <div class="flex-1 flex flex-col gap-5">
                    // Plan name + draft link
                    <div class="bg-gray-800/50 border border-gray-700/50 rounded-xl p-4 flex flex-col gap-4">
                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label class="block text-gray-400 text-xs font-medium mb-1">"Plan Name"</label>
                                <input type="text" class=input_class()
                                    placeholder="e.g. Comp A vs Scaling"
                                    prop:value=move || plan_name.get()
                                    on:input=move |ev| set_plan_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-gray-400 text-xs font-medium mb-1">"Linked Draft (optional)"</label>
                                <Suspense fallback=|| view! { <div class="h-9 bg-gray-700/50 rounded-lg animate-pulse"></div> }>
                                    {move || drafts.get().map(|result| match result {
                                        Ok(list) => view! {
                                            <select class=input_class()
                                                prop:value=move || draft_id.get()
                                                on:change=move |ev| set_draft_id.set(event_target_value(&ev))
                                            >
                                                <option value="">"None"</option>
                                                {list.into_iter().map(|d| {
                                                    let id = d.id.clone().unwrap_or_default();
                                                    let label = d.name.clone();
                                                    view! { <option value=id>{label}</option> }
                                                }).collect_view()}
                                            </select>
                                        }.into_any(),
                                        Err(_) => view! { <p class="text-gray-500 text-sm">"No drafts"</p> }.into_any(),
                                    })}
                                </Suspense>
                            </div>
                        </div>
                    </div>

                    // Matchup: Your 5 vs Enemy 5
                    <div class="bg-gray-800/50 border border-gray-700/50 rounded-xl p-4">
                        <div class="flex items-center justify-between mb-3">
                            <h3 class="text-white font-semibold text-sm">"Champion Matchup"</h3>
                            <button
                                class="bg-gray-700 hover:bg-gray-600 text-gray-300 text-xs font-medium px-3 py-1.5 rounded-lg transition-colors"
                                on:click=do_generate_template
                            >"Generate Template"</button>
                        </div>
                        <div class="grid grid-cols-[1fr_auto_1fr] gap-4 items-start">
                            // Our team
                            <div class="flex flex-col gap-2">
                                <span class="text-blue-400 text-xs font-semibold uppercase">"Your Team"</span>
                                {(0..5).map(|i| {
                                    let role = ROLES[i];
                                    view! {
                                        <div class="flex items-center gap-2">
                                            <span class="text-gray-500 text-xs w-14">{role}</span>
                                            <input type="text" class=input_class()
                                                placeholder=format!("{role} champion")
                                                prop:value=move || our_champs.get().get(i).cloned().unwrap_or_default()
                                                on:input=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    set_our_champs.update(|v| {
                                                        if i < v.len() { v[i] = val; }
                                                    });
                                                }
                                            />
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                            <div class="flex items-center justify-center self-center text-gray-500 font-bold text-sm pt-6">"VS"</div>
                            // Enemy team
                            <div class="flex flex-col gap-2">
                                <span class="text-red-400 text-xs font-semibold uppercase">"Enemy Team"</span>
                                {(0..5).map(|i| {
                                    let role = ROLES[i];
                                    view! {
                                        <div class="flex items-center gap-2">
                                            <span class="text-gray-500 text-xs w-14">{role}</span>
                                            <input type="text" class=input_class()
                                                placeholder=format!("{role} champion")
                                                prop:value=move || enemy_champs.get().get(i).cloned().unwrap_or_default()
                                                on:input=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    set_enemy_champs.update(|v| {
                                                        if i < v.len() { v[i] = val; }
                                                    });
                                                }
                                            />
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    </div>

                    // Macro strategy
                    <div class="bg-gray-800/50 border border-gray-700/50 rounded-xl p-4 flex flex-col gap-4">
                        <h3 class="text-white font-semibold text-sm">"Macro Strategy"</h3>
                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label class="block text-gray-400 text-xs font-medium mb-1">"Win Conditions (one per line)"</label>
                                <textarea rows="4" class=textarea_class()
                                    placeholder="Force early teamfights\nControl Dragon side"
                                    prop:value=move || win_conditions.get()
                                    on:input=move |ev| set_win_conditions.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-gray-400 text-xs font-medium mb-1">"Objective Priority (one per line)"</label>
                                <textarea rows="4" class=textarea_class()
                                    placeholder="Dragon\nRift Herald\nBaron"
                                    prop:value=move || obj_priority.get()
                                    on:input=move |ev| set_obj_priority.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                        <div>
                            <label class="block text-gray-400 text-xs font-medium mb-1">"Teamfight Strategy"</label>
                            <textarea rows="2" class=textarea_class()
                                placeholder="Front-to-back: protect ADC while top zones..."
                                prop:value=move || teamfight.get()
                                on:input=move |ev| set_teamfight.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class="block text-gray-400 text-xs font-medium mb-1">"Early Game Plan"</label>
                            <textarea rows="2" class=textarea_class()
                                placeholder="Contest level 1 vision, jungle path..."
                                prop:value=move || early_game.get()
                                on:input=move |ev| set_early_game.set(event_target_value(&ev))
                            />
                        </div>
                    </div>

                    // Role-specific strategy
                    <div class="bg-gray-800/50 border border-gray-700/50 rounded-xl p-4 flex flex-col gap-3">
                        <h3 class="text-white font-semibold text-sm">"Role-Specific Strategy"</h3>
                        <div class="grid grid-cols-1 gap-3">
                            {ROLES.iter().enumerate().map(|(i, &role)| {
                                let our_idx = i;
                                let enemy_idx = i;
                                view! {
                                    <div class="flex gap-3 items-start">
                                        <div class="w-16 flex-shrink-0 pt-2">
                                            <span class="text-yellow-400 text-xs font-semibold">{role}</span>
                                            {move || {
                                                let ours = our_champs.get();
                                                let theirs = enemy_champs.get();
                                                let our = ours.get(our_idx).filter(|s| !s.is_empty());
                                                let their = theirs.get(enemy_idx).filter(|s| !s.is_empty());
                                                match (our, their) {
                                                    (Some(o), Some(t)) => view! {
                                                        <div class="text-gray-500 text-xs mt-0.5">{format!("{o} vs {t}")}</div>
                                                    }.into_any(),
                                                    _ => view! { <div></div> }.into_any(),
                                                }
                                            }}
                                        </div>
                                        <textarea rows="2" class=format!("{} flex-1", textarea_class())
                                            placeholder=format!("Strategy for {role}...")
                                            prop:value=move || role_strats.get().get(i).cloned().unwrap_or_default()
                                            on:input=move |ev| {
                                                let val = event_target_value(&ev);
                                                set_role_strats.update(|v| {
                                                    if i < v.len() { v[i] = val; }
                                                });
                                            }
                                        />
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    // Notes
                    <div class="bg-gray-800/50 border border-gray-700/50 rounded-xl p-4">
                        <label class="block text-gray-400 text-xs font-medium mb-1">"Additional Notes"</label>
                        <textarea rows="3" class=textarea_class()
                            placeholder="Anything else the team should know..."
                            prop:value=move || notes.get()
                            on:input=move |ev| set_notes.set(event_target_value(&ev))
                        />
                    </div>

                    // Save buttons
                    <div class="flex gap-3 items-center">
                        <button
                            class="bg-yellow-400 hover:bg-yellow-300 text-gray-900 font-semibold rounded-lg px-6 py-2 text-sm transition-colors"
                            on:click=do_save
                        >
                            {move || if editing_id.get().is_some() { "Update Plan" } else { "Save Plan" }}
                        </button>
                        <button
                            class="bg-gray-700 hover:bg-gray-600 text-gray-300 rounded-lg px-4 py-2 text-sm transition-colors"
                            on:click=move |_| clear_editor()
                        >"Clear"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}
