use leptos::prelude::*;
use crate::models::champion::{Champion, ChampionPoolEntry};
use crate::components::champion_autocomplete::ChampionAutocomplete;
use crate::components::ui::StatusMessage;

#[server]
pub async fn get_pool() -> Result<Vec<ChampionPoolEntry>, ServerFnError> {
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
pub async fn get_pool_champions() -> Result<Vec<Champion>, ServerFnError> {
    use crate::server::data_dragon;
    data_dragon::fetch_champions()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn add_to_pool(champion: String, role: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;
    db::add_to_champion_pool(&db, &user.id, champion, role)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn remove_from_pool(champion: String, role: String) -> Result<(), ServerFnError> {
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
pub async fn set_champion_tier(champion: String, role: String, tier: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;
    db::update_champion_tier(&db, &user.id, &champion, &role, tier)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn set_champion_notes(champion: String, role: String, notes: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;
    let notes_opt = if notes.is_empty() { None } else { Some(notes) };
    db::update_champion_notes(&db, &user.id, &champion, &role, notes_opt)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

const POOL_ROLES: &[&str] = &["Top", "Jungle", "Mid", "ADC", "Support"];
const TIERS: &[&str] = &["comfort", "match_ready", "scrim_ready", "practicing", "to_practice"];

fn tier_label(tier: &str) -> &'static str {
    match tier {
        "comfort" => "Comfort",
        "match_ready" => "Match Ready",
        "scrim_ready" => "Scrim Ready",
        "practicing" => "Practicing",
        "to_practice" => "Should Practice",
        _ => "Other",
    }
}

fn tier_color(tier: &str) -> &'static str {
    match tier {
        "comfort" => "border-accent/30 bg-accent/5",
        "match_ready" => "border-green-400/30 bg-green-400/5",
        "scrim_ready" => "border-blue-400/30 bg-blue-400/5",
        "practicing" => "border-purple-400/30 bg-purple-400/5",
        "to_practice" => "border-gray-400/30 bg-gray-400/5",
        _ => "border-divider bg-elevated/50",
    }
}

fn tier_label_color(tier: &str) -> &'static str {
    match tier {
        "comfort" => "text-accent",
        "match_ready" => "text-green-400",
        "scrim_ready" => "text-blue-400",
        "practicing" => "text-purple-400",
        "to_practice" => "text-muted",
        _ => "text-muted",
    }
}

#[component]
pub fn ChampionPoolPage() -> impl IntoView {
    let pool = Resource::new(|| (), |_| get_pool());
    let champions_resource = Resource::new(|| (), |_| get_pool_champions());

    let (active_role, set_active_role) = signal("Top");
    let (add_input, set_add_input) = signal(String::new());
    let (status_msg, set_status_msg) = signal(Option::<String>::None);
    let (selected_entry, set_selected_entry) = signal(Option::<(String, String)>::None); // (champion, role)
    let (notes_input, set_notes_input) = signal(String::new());

    let add_input_signal = RwSignal::new(String::new());

    // Sync add_input and add_input_signal
    Effect::new(move |_| {
        let v = add_input_signal.get();
        if v != add_input.get_untracked() {
            set_add_input.set(v);
        }
    });

    view! {
        <div class="max-w-6xl mx-auto py-8 px-6 flex flex-col gap-6">
            <div>
                <h1 class="text-3xl font-bold text-primary">"Champion Pool"</h1>
                <p class="text-muted text-sm mt-1">"Organize your champion pool by role and readiness tier"</p>
            </div>

            {move || status_msg.get().map(|msg| view! { <StatusMessage message=msg /> })}

            // Role tabs
            <div class="flex gap-1">
                {POOL_ROLES.iter().map(|&role| {
                    view! {
                        <button
                            class=move || if active_role.get() == role {
                                "px-4 py-2 rounded-lg text-sm font-medium bg-accent text-accent-contrast transition-colors cursor-pointer"
                            } else {
                                "px-4 py-2 rounded-lg text-sm font-medium bg-elevated text-secondary hover:bg-overlay transition-colors cursor-pointer"
                            }
                            on:click=move |_| {
                                set_active_role.set(role);
                                set_selected_entry.set(None);
                            }
                        >
                            {role}
                        </button>
                    }
                }).collect_view()}
            </div>

            <div class="flex gap-6 min-h-[30rem]">
                // Main content: tiers
                <div class="flex-1 flex flex-col gap-4">
                    // Add champion form
                    <div class="flex gap-2 items-end">
                        <div class="flex-1">
                            <Suspense fallback=|| view! { <div class="h-10 bg-overlay rounded animate-pulse"></div> }>
                                {move || champions_resource.get().map(|result| {
                                    let champs = result.unwrap_or_default();
                                    view! {
                                        <ChampionAutocomplete
                                            champions=champs
                                            value=add_input_signal
                                            placeholder="Add a champion..."
                                        />
                                    }
                                })}
                            </Suspense>
                        </div>
                        <button
                            class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm transition-colors cursor-pointer"
                            on:click=move |_| {
                                let champion = add_input_signal.get_untracked();
                                let role = active_role.get_untracked().to_string();
                                if champion.trim().is_empty() { return; }
                                leptos::task::spawn_local(async move {
                                    match add_to_pool(champion, role).await {
                                        Ok(_) => {
                                            add_input_signal.set(String::new());
                                            pool.refetch();
                                        }
                                        Err(e) => set_status_msg.set(Some(format!("Error: {e}"))),
                                    }
                                });
                            }
                        >"Add"</button>
                    </div>

                    // Tier columns
                    <Suspense fallback=|| view! { <div class="text-dimmed text-sm">"Loading pool..."</div> }>
                        {move || {
                            let role = active_role.get();
                            pool.get().map(move |result| {
                                let entries = result.unwrap_or_default();
                                let role_entries: Vec<ChampionPoolEntry> = entries.into_iter()
                                    .filter(|e| e.role == role)
                                    .collect();

                                if role_entries.is_empty() {
                                    return view! {
                                        <div class="flex-1 flex items-center justify-center">
                                            <p class="text-dimmed text-sm">"No champions in pool for this role. Add one above."</p>
                                        </div>
                                    }.into_any();
                                }

                                view! {
                                    <div class="flex flex-col gap-3">
                                        {TIERS.iter().map(|&tier| {
                                            let tier_entries: Vec<&ChampionPoolEntry> = role_entries.iter()
                                                .filter(|e| e.tier == tier)
                                                .collect();
                                            if tier_entries.is_empty() {
                                                return view! { <div></div> }.into_any();
                                            }
                                            view! {
                                                <div class=format!("border rounded-xl p-4 {}", tier_color(tier))>
                                                    <h3 class=format!("text-xs font-semibold uppercase tracking-wider mb-3 {}", tier_label_color(tier))>
                                                        {tier_label(tier)}
                                                        <span class="text-dimmed ml-1">{format!("({})", tier_entries.len())}</span>
                                                    </h3>
                                                    <div class="flex flex-wrap gap-2">
                                                        {tier_entries.into_iter().map(|entry| {
                                                            let champ = entry.champion.clone();
                                                            let role_val = entry.role.clone();
                                                            let champ_for_select = champ.clone();
                                                            let role_for_select = role_val.clone();
                                                            let champ_for_remove = champ.clone();
                                                            let role_for_remove = role_val.clone();

                                                            let img_url = champions_resource.get()
                                                                .and_then(|r| r.ok())
                                                                .and_then(|champs| champs.into_iter().find(|c| c.name == champ))
                                                                .map(|c| c.image_full)
                                                                .unwrap_or_default();

                                                            view! {
                                                                <div
                                                                    class="flex items-center gap-1.5 bg-elevated border border-divider rounded-lg px-2.5 py-1.5 hover:border-accent/50 transition-colors cursor-pointer group"
                                                                    on:click=move |_| {
                                                                        set_selected_entry.set(Some((champ_for_select.clone(), role_for_select.clone())));
                                                                    }
                                                                >
                                                                    {if !img_url.is_empty() {
                                                                        view! {
                                                                            <img src=img_url alt=champ.clone() class="w-7 h-7 rounded object-cover" />
                                                                        }.into_any()
                                                                    } else {
                                                                        view! { <span></span> }.into_any()
                                                                    }}
                                                                    <span class="text-primary text-sm">{champ}</span>
                                                                    <button
                                                                        class="text-overlay-strong hover:text-red-400 ml-0.5 transition-colors opacity-0 group-hover:opacity-100 cursor-pointer"
                                                                        title="Remove"
                                                                        on:click=move |ev| {
                                                                            ev.stop_propagation();
                                                                            let c = champ_for_remove.clone();
                                                                            let r = role_for_remove.clone();
                                                                            leptos::task::spawn_local(async move {
                                                                                match remove_from_pool(c, r).await {
                                                                                    Ok(_) => pool.refetch(),
                                                                                    Err(e) => set_status_msg.set(Some(e.to_string())),
                                                                                }
                                                                            });
                                                                        }
                                                                    >"×"</button>
                                                                </div>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            })
                        }}
                    </Suspense>
                </div>

                // Right panel: champion details
                <div class="w-72 flex-shrink-0">
                    {move || {
                        let sel = selected_entry.get();
                        match sel {
                            None => view! {
                                <div class="bg-elevated/30 border border-divider/30 rounded-xl p-6 flex items-center justify-center min-h-[200px]">
                                    <p class="text-dimmed text-sm text-center">"Click a champion to view details and change tier"</p>
                                </div>
                            }.into_any(),
                            Some((ref champ, ref role)) => {
                                let champ_clone = champ.clone();
                                let role_clone = role.clone();
                                let champ_for_notes = champ.clone();
                                let role_for_notes = role.clone();

                                // Look up current entry data
                                let current_tier = pool.get()
                                    .and_then(|r| r.ok())
                                    .and_then(|entries| entries.into_iter().find(|e| e.champion == *champ && e.role == *role))
                                    .map(|e| (e.tier.clone(), e.notes.clone().unwrap_or_default()))
                                    .unwrap_or(("comfort".to_string(), String::new()));

                                let img_url = champions_resource.get()
                                    .and_then(|r| r.ok())
                                    .and_then(|champs| champs.into_iter().find(|c| c.name == *champ))
                                    .map(|c| c.image_full)
                                    .unwrap_or_default();

                                // Set notes input when selecting
                                set_notes_input.set(current_tier.1.clone());

                                view! {
                                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4 flex flex-col gap-4">
                                        <div class="flex items-center gap-3">
                                            {if !img_url.is_empty() {
                                                view! { <img src=img_url alt=champ.clone() class="w-12 h-12 rounded-lg object-cover" /> }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }}
                                            <div>
                                                <h3 class="text-primary font-semibold">{champ.clone()}</h3>
                                                <p class="text-muted text-xs capitalize">{role.clone()}</p>
                                            </div>
                                        </div>

                                        // Tier selector
                                        <div>
                                            <label class="block text-muted text-xs font-medium mb-2">"Tier"</label>
                                            <div class="flex flex-col gap-1">
                                                {TIERS.iter().map(|&tier| {
                                                    let is_current = current_tier.0 == tier;
                                                    let champ_t = champ_clone.clone();
                                                    let role_t = role_clone.clone();
                                                    view! {
                                                        <button
                                                            class=move || if is_current {
                                                                format!("w-full text-left px-3 py-1.5 rounded text-sm font-medium {} cursor-pointer", tier_label_color(tier))
                                                            } else {
                                                                "w-full text-left px-3 py-1.5 rounded text-sm text-muted hover:bg-overlay/50 transition-colors cursor-pointer".to_string()
                                                            }
                                                            on:click=move |_| {
                                                                let c = champ_t.clone();
                                                                let r = role_t.clone();
                                                                let t = tier.to_string();
                                                                leptos::task::spawn_local(async move {
                                                                    match set_champion_tier(c, r, t).await {
                                                                        Ok(_) => pool.refetch(),
                                                                        Err(e) => set_status_msg.set(Some(e.to_string())),
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            {if is_current { "● " } else { "○ " }}
                                                            {tier_label(tier)}
                                                        </button>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>

                                        // Notes
                                        <div>
                                            <label class="block text-muted text-xs font-medium mb-1">"Notes"</label>
                                            <textarea
                                                rows="4"
                                                class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus:outline-none focus:border-accent/50 resize-none transition-colors"
                                                placeholder="Matchups, combos, notes..."
                                                prop:value=move || notes_input.get()
                                                on:input=move |ev| set_notes_input.set(event_target_value(&ev))
                                            />
                                            <button
                                                class="mt-2 bg-overlay hover:bg-overlay-strong text-secondary text-xs font-medium rounded px-3 py-1.5 transition-colors cursor-pointer"
                                                on:click=move |_| {
                                                    let c = champ_for_notes.clone();
                                                    let r = role_for_notes.clone();
                                                    let n = notes_input.get_untracked();
                                                    leptos::task::spawn_local(async move {
                                                        match set_champion_notes(c, r, n).await {
                                                            Ok(_) => {
                                                                set_status_msg.set(Some("Notes saved!".into()));
                                                                pool.refetch();
                                                            }
                                                            Err(e) => set_status_msg.set(Some(e.to_string())),
                                                        }
                                                    });
                                                }
                                            >"Save Notes"</button>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
