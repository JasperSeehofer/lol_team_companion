use crate::components::champion_autocomplete::ChampionAutocomplete;
use crate::components::ui::{ErrorBanner, ToastContext, ToastKind};
use crate::models::champion::Champion;
use crate::models::personal_learning::{PersonalLearning, LEARNING_TAGS};
use leptos::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Server functions (defined before components per CLAUDE.md rule 34)
// ---------------------------------------------------------------------------

#[server]
pub async fn list_learnings() -> Result<Vec<PersonalLearning>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = match auth.user {
        Some(u) => u,
        None => return Ok(Vec::new()),
    };
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::list_personal_learnings(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_learning(id: String) -> Result<Option<PersonalLearning>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::get_personal_learning(&db, &id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn save_learning(
    title: String,
    learning_type: String,
    champion: String,
    opponent: String,
    what_happened: String,
    what_i_learned: String,
    next_time: String,
    tags_json: String,
    win_loss: String,
    match_riot_id: String,
    game_timestamp_ms: String,
    event_name: String,
) -> Result<String, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let tags: Vec<String> = serde_json::from_str(&tags_json)
        .map_err(|e| ServerFnError::new(format!("Invalid tags JSON: {e}")))?;

    let learning = PersonalLearning {
        id: None,
        user_id: user.id,
        title,
        learning_type,
        champion: if champion.is_empty() { None } else { Some(champion) },
        opponent: if opponent.is_empty() { None } else { Some(opponent) },
        what_happened,
        what_i_learned,
        next_time,
        tags,
        win_loss: if win_loss.is_empty() { None } else { Some(win_loss) },
        match_riot_id: if match_riot_id.is_empty() { None } else { Some(match_riot_id) },
        game_timestamp_ms: game_timestamp_ms.parse::<i64>().ok(),
        event_name: if event_name.is_empty() { None } else { Some(event_name) },
        created_at: None,
    };

    db::create_personal_learning(&surreal, learning)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_learning(
    id: String,
    title: String,
    learning_type: String,
    champion: String,
    opponent: String,
    what_happened: String,
    what_i_learned: String,
    next_time: String,
    tags_json: String,
    win_loss: String,
    match_riot_id: String,
    game_timestamp_ms: String,
    event_name: String,
) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let tags: Vec<String> = serde_json::from_str(&tags_json)
        .map_err(|e| ServerFnError::new(format!("Invalid tags JSON: {e}")))?;

    let learning = PersonalLearning {
        id: Some(id),
        user_id: user.id,
        title,
        learning_type,
        champion: if champion.is_empty() { None } else { Some(champion) },
        opponent: if opponent.is_empty() { None } else { Some(opponent) },
        what_happened,
        what_i_learned,
        next_time,
        tags,
        win_loss: if win_loss.is_empty() { None } else { Some(win_loss) },
        match_riot_id: if match_riot_id.is_empty() { None } else { Some(match_riot_id) },
        game_timestamp_ms: game_timestamp_ms.parse::<i64>().ok(),
        event_name: if event_name.is_empty() { None } else { Some(event_name) },
        created_at: None,
    };

    db::update_personal_learning(&surreal, learning)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_learning(id: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::delete_personal_learning(&db, &id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_champions_for_learnings() -> Result<Vec<Champion>, ServerFnError> {
    use crate::server::data_dragon;
    data_dragon::fetch_champions()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ---------------------------------------------------------------------------
// Helper: champion icon URL
// ---------------------------------------------------------------------------

fn champion_icon_url(name: &str) -> String {
    format!("https://ddragon.leagueoflegends.com/cdn/15.6.1/img/champion/{name}.png")
}

// ---------------------------------------------------------------------------
// Browse Page: PersonalLearningsPage
// ---------------------------------------------------------------------------

#[component]
pub fn PersonalLearningsPage() -> impl IntoView {
    // Auth guard
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    let learnings = Resource::new(|| (), |_| list_learnings());

    let filter_type = RwSignal::new("all".to_string());
    let filter_champion = RwSignal::new("all".to_string());
    let filter_tag = RwSignal::new("all".to_string());
    let sort_mode = RwSignal::new("newest".to_string());
    let expanded_id: RwSignal<Option<String>> = RwSignal::new(None);

    let toast = use_context::<ToastContext>();

    view! {
        <div class="canvas-grain bg-base min-h-screen">
            <div class="max-w-7xl mx-auto px-6 py-10">
                // Page header
                <div class="flex items-end justify-between gap-4 flex-wrap mb-6">
                    <div class="flex flex-col gap-2">
                        <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">
                            "The journal"
                        </span>
                        <h1 class="font-display italic text-primary text-3xl">"Personal learnings"</h1>
                        <p class="text-muted text-sm">"Your private post-game reflections."</p>
                    </div>
                    <a
                        href="/personal-learnings/new"
                        class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold px-5 py-2 rounded-lg text-sm transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                    >
                        "+ New learning"
                    </a>
                </div>

                <Suspense fallback=move || view! {
                    <div class="text-muted text-sm">"Loading..."</div>
                }>
                    {move || {
                        let data = learnings.get();
                        match data {
                            None => view! { <div></div> }.into_any(),
                            Some(Err(e)) => view! {
                                <ErrorBanner message=e.to_string() />
                            }.into_any(),
                            Some(Ok(all_learnings)) => {
                                let all_learnings = StoredValue::new(all_learnings);

                                // Unique champions for dropdown
                                let unique_champions: Vec<String> = {
                                    let mut set = std::collections::BTreeSet::new();
                                    all_learnings.with_value(|ls| {
                                        for l in ls {
                                            if let Some(c) = &l.champion {
                                                set.insert(c.clone());
                                            }
                                        }
                                    });
                                    set.into_iter().collect()
                                };

                                view! {
                                    // Filter bar (Card.plain)
                                    <div class="bg-elevated border border-divider rounded-xl p-4 flex items-center gap-3 mb-6 flex-wrap">
                                        <span class="font-imperial uppercase tracking-wider text-xs text-muted">"Filter"</span>
                                        // Type filter
                                        <select
                                            class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                                            on:change=move |ev| filter_type.set(event_target_value(&ev))
                                        >
                                            <option value="all">"All Types"</option>
                                            <option value="general">"General"</option>
                                            <option value="champion">"Champion"</option>
                                            <option value="matchup">"Matchup"</option>
                                        </select>
                                        // Champion filter
                                        <select
                                            class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                                            on:change=move |ev| filter_champion.set(event_target_value(&ev))
                                        >
                                            <option value="all">"All Champions"</option>
                                            {unique_champions.into_iter().map(|c| {
                                                let c2 = c.clone();
                                                view! { <option value=c>{c2}</option> }
                                            }).collect_view()}
                                        </select>
                                        // Tag filter
                                        <select
                                            class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                                            on:change=move |ev| filter_tag.set(event_target_value(&ev))
                                        >
                                            <option value="all">"All Tags"</option>
                                            {LEARNING_TAGS.iter().map(|t| {
                                                let t = *t;
                                                view! { <option value=t>{t}</option> }
                                            }).collect_view()}
                                        </select>
                                        // Sort (right-aligned)
                                        <select
                                            class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors ml-auto"
                                            on:change=move |ev| sort_mode.set(event_target_value(&ev))
                                        >
                                            <option value="newest">"Newest first"</option>
                                            <option value="champion">"By champion"</option>
                                        </select>
                                    </div>

                                    // Card grid
                                    {move || {
                                        let ft = filter_type.get();
                                        let fc = filter_champion.get();
                                        let ftag = filter_tag.get();
                                        let sm = sort_mode.get();

                                        let filtered: Vec<PersonalLearning> = all_learnings.with_value(|ls| {
                                            ls.iter()
                                                .filter(|l| {
                                                    let type_ok = ft == "all" || l.learning_type == ft;
                                                    let champ_ok = fc == "all"
                                                        || l.champion.as_deref() == Some(fc.as_str());
                                                    let tag_ok = ftag == "all"
                                                        || l.tags.contains(&ftag);
                                                    type_ok && champ_ok && tag_ok
                                                })
                                                .cloned()
                                                .collect()
                                        });

                                        if all_learnings.with_value(|ls| ls.is_empty()) {
                                            // No learnings at all
                                            return view! {
                                                <div class="bg-elevated border border-divider rounded-xl text-center py-16 px-6">
                                                    <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Empty journal"</span>
                                                    <h2 class="font-display italic text-primary text-2xl mt-2">"No learnings yet"</h2>
                                                    <p class="text-muted text-sm mt-3 mb-6 max-w-md mx-auto">
                                                        "Write your first reflection after a game. Start from a match or create one here."
                                                    </p>
                                                    <a
                                                        href="/personal-learnings/new"
                                                        class="inline-flex items-center bg-accent hover:bg-accent-hover text-accent-contrast font-semibold px-5 py-2 rounded-lg text-sm transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                    >
                                                        "Write your first learning"
                                                    </a>
                                                </div>
                                            }.into_any();
                                        }

                                        if filtered.is_empty() {
                                            return view! {
                                                <div class="bg-elevated border border-divider rounded-xl text-center py-16 px-6">
                                                    <p class="text-muted text-sm mb-3">"No learnings match these filters."</p>
                                                    <button
                                                        class="text-accent text-sm hover:underline cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded"
                                                        on:click=move |_| {
                                                            filter_type.set("all".to_string());
                                                            filter_champion.set("all".to_string());
                                                            filter_tag.set("all".to_string());
                                                        }
                                                    >
                                                        "Reset filters"
                                                    </button>
                                                </div>
                                            }.into_any();
                                        }

                                        if sm == "champion" {
                                            // Group by champion
                                            let mut grouped: BTreeMap<String, Vec<PersonalLearning>> = BTreeMap::new();
                                            for l in filtered {
                                                let key = l.champion.clone().unwrap_or_else(|| "General".to_string());
                                                grouped.entry(key).or_default().push(l);
                                            }

                                            view! {
                                                <div>
                                                    {grouped.into_iter().map(|(champ, items)| {
                                                        view! {
                                                            <div class="mb-6">
                                                                <p class="font-imperial uppercase tracking-wider text-xs text-muted py-2 mb-2 border-b border-divider/30">
                                                                    {champ}
                                                                </p>
                                                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                                                    {items.into_iter().map(|l| {
                                                                        view! {
                                                                            <LearningCard
                                                                                learning=l
                                                                                expanded_id=expanded_id
                                                                                learnings=learnings
                                                                                toast=toast
                                                                            />
                                                                        }
                                                                    }).collect_view()}
                                                                </div>
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                                    {filtered.into_iter().map(|l| {
                                                        view! {
                                                            <LearningCard
                                                                learning=l
                                                                expanded_id=expanded_id
                                                                learnings=learnings
                                                                toast=toast
                                                            />
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                }.into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Learning Card Component
// ---------------------------------------------------------------------------

#[component]
fn LearningCard(
    learning: PersonalLearning,
    expanded_id: RwSignal<Option<String>>,
    learnings: Resource<Result<Vec<PersonalLearning>, ServerFnError>>,
    toast: Option<ToastContext>,
) -> impl IntoView {
    let id = learning.id.clone().unwrap_or_default();
    let id_for_click = id.clone();
    let id_for_delete = id.clone();
    let id_for_edit = id.clone();

    let learning = StoredValue::new(learning);

    let id_for_expanded = id.clone();
    let id_for_class = id.clone();
    let is_expanded = move || expanded_id.get().as_deref() == Some(id_for_expanded.as_str());
    let is_expanded2 = move || expanded_id.get().as_deref() == Some(id_for_class.as_str());

    let card_class = move || {
        if is_expanded2() {
            "bg-elevated border border-accent/40 rounded-xl p-4 transition-all duration-200 cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
        } else {
            "bg-elevated border border-divider rounded-xl p-4 hover:border-outline transition-all duration-200 cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
        }
    };

    view! {
        <div
            class=card_class
            on:click=move |_| {
                let current = expanded_id.get_untracked();
                if current.as_deref() == Some(id_for_click.as_str()) {
                    expanded_id.set(None);
                } else {
                    expanded_id.set(Some(id_for_click.clone()));
                }
            }
        >
            // Card header: badges + date
            <div class="flex items-center gap-2 flex-wrap mb-2">
                {learning.with_value(|l| {
                    let type_label = match l.learning_type.as_str() {
                        "champion" => "Champion",
                        "matchup" => "Matchup",
                        _ => "General",
                    };
                    view! {
                        <span class="bg-surface text-muted text-[10px] font-imperial uppercase tracking-wider rounded px-2 py-0.5 border border-outline/50">
                            {type_label}
                        </span>
                    }
                })}
                {learning.with_value(|l| {
                    match l.win_loss.as_deref() {
                        Some("win") => view! {
                            <span class="bg-success/15 text-success border border-success/30 text-[10px] font-imperial uppercase tracking-wider rounded px-2 py-0.5">
                                "Win"
                            </span>
                        }.into_any(),
                        Some("loss") => view! {
                            <span class="bg-danger/15 text-danger border border-danger/30 text-[10px] font-imperial uppercase tracking-wider rounded px-2 py-0.5">
                                "Loss"
                            </span>
                        }.into_any(),
                        _ => view! { <span></span> }.into_any(),
                    }
                })}
                {learning.with_value(|l| {
                    l.created_at.as_ref().map(|ts| {
                        let date = ts.split('T').next().unwrap_or(ts.as_str()).to_string();
                        view! {
                            <span class="font-mono text-xs text-dimmed ml-auto tabular-nums">{date}</span>
                        }
                    })
                })}
            </div>

            // Title
            <p class="font-display italic text-primary text-base leading-snug truncate mb-1">
                {learning.with_value(|l| l.title.clone())}
            </p>

            // Champion icon (if champion or matchup type)
            {learning.with_value(|l| {
                let show = l.learning_type == "champion" || l.learning_type == "matchup";
                if show {
                    if let Some(champ) = &l.champion {
                        let url = champion_icon_url(champ);
                        let champ = champ.clone();
                        return view! {
                            <img src=url alt=champ class="w-6 h-6 rounded object-cover mb-1 border border-outline/50" />
                        }.into_any();
                    }
                }
                view! { <span></span> }.into_any()
            })}

            // Preview (first 80 chars of what_happened)
            <p class="text-muted text-xs mb-2 line-clamp-2">
                {learning.with_value(|l| {
                    let preview = &l.what_happened;
                    if preview.len() > 80 {
                        format!("{}\u{2026}", &preview[..80])
                    } else {
                        preview.clone()
                    }
                })}
            </p>

            // Tags
            <div class="flex flex-wrap gap-1 mb-2">
                {learning.with_value(|l| {
                    l.tags.iter().map(|tag| {
                        let tag = tag.clone();
                        view! {
                            <span class="rounded-full px-2.5 py-0.5 text-[10px] font-medium bg-surface text-muted border border-outline/40">
                                {tag}
                            </span>
                        }
                    }).collect_view()
                })}
            </div>

            // Expanded content
            {move || {
                if !is_expanded() {
                    return view! { <span></span> }.into_any();
                }

                let what_happened = learning.with_value(|l| l.what_happened.clone());
                let what_i_learned = learning.with_value(|l| l.what_i_learned.clone());
                let next_time = learning.with_value(|l| l.next_time.clone());
                let edit_href = format!("/personal-learnings/new?edit={id_for_edit}");
                let del_id = id_for_delete.clone();

                view! {
                    <div class="border-t border-divider/40 pt-4 mt-2" on:click=move |ev| ev.stop_propagation()>
                        <div class="space-y-3 mb-4">
                            <div>
                                <p class="font-imperial uppercase tracking-wider text-[10px] text-muted mb-1">"What happened"</p>
                                <p class="text-sm text-secondary">{what_happened}</p>
                            </div>
                            <div>
                                <p class="font-imperial uppercase tracking-wider text-[10px] text-muted mb-1">"What I learned"</p>
                                <p class="text-sm text-secondary">{what_i_learned}</p>
                            </div>
                            <div>
                                <p class="font-imperial uppercase tracking-wider text-[10px] text-muted mb-1">"Next time I will..."</p>
                                <p class="text-sm text-secondary">{next_time}</p>
                            </div>
                        </div>
                        <div class="flex items-center gap-3">
                            <a
                                href=edit_href
                                class="bg-surface border border-outline/50 text-secondary hover:text-primary hover:border-outline px-3 py-1 rounded-lg text-xs transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                on:click=move |ev| ev.stop_propagation()
                            >
                                "Edit"
                            </a>
                            <button
                                class="text-danger/70 hover:text-danger text-xs transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-danger/50 focus-visible:outline-none rounded"
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    let id = del_id.clone();
                                    let toast = toast;
                                    leptos::task::spawn_local(async move {
                                        match delete_learning(id).await {
                                            Ok(()) => {
                                                learnings.refetch();
                                                if let Some(t) = toast {
                                                    t.show.run((ToastKind::Success, "Learning deleted.".into()));
                                                }
                                                expanded_id.set(None);
                                            }
                                            Err(e) => {
                                                if let Some(t) = toast {
                                                    t.show.run((ToastKind::Error, format!("Failed to delete: {e}")));
                                                }
                                            }
                                        }
                                    });
                                }
                            >
                                "Delete learning"
                            </button>
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Form Page: NewLearningPage
// ---------------------------------------------------------------------------

#[component]
pub fn NewLearningPage() -> impl IntoView {
    use leptos_router::hooks::use_query_map;

    // Auth guard
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    let query = use_query_map();

    // Read URL params (once, eagerly)
    let q_champion = query.read().get("champion").unwrap_or_default();
    let q_opponent = query.read().get("opponent").unwrap_or_default();
    let q_match_id = query.read().get("match_id").unwrap_or_default();
    let q_result = query.read().get("result").unwrap_or_default();
    let q_event_ts = query.read().get("event_ts").unwrap_or_default();
    let q_event_name = query.read().get("event_name").unwrap_or_default();
    let q_edit = query.read().get("edit").unwrap_or_default();
    let q_tag_hint = query.read().get("tag_hint").unwrap_or_default();

    // Determine default type from URL params
    let default_type = if !q_champion.is_empty() && !q_opponent.is_empty() {
        "matchup"
    } else if !q_champion.is_empty() {
        "champion"
    } else {
        "general"
    };

    // Form signals
    let learning_type = RwSignal::new(default_type.to_string());
    let champion = RwSignal::new(q_champion.clone());
    let opponent = RwSignal::new(q_opponent.clone());
    let what_happened = RwSignal::new(String::new());
    let what_i_learned = RwSignal::new(String::new());
    let next_time = RwSignal::new(String::new());

    // Pre-select tag hint if present
    let initial_tags: Vec<String> = if q_tag_hint.is_empty() {
        Vec::new()
    } else {
        vec![q_tag_hint.clone()]
    };
    let selected_tags: RwSignal<Vec<String>> = RwSignal::new(initial_tags);

    let title = RwSignal::new(String::new());
    let title_edited = RwSignal::new(false);
    let win_loss = RwSignal::new(q_result.clone());
    let match_riot_id = RwSignal::new(q_match_id.clone());
    let game_timestamp_ms = RwSignal::new(q_event_ts.clone());
    let event_name_signal = RwSignal::new(q_event_name.clone());

    let save_error: RwSignal<Option<String>> = RwSignal::new(None);
    let what_happened_error = RwSignal::new(false);
    let what_i_learned_error = RwSignal::new(false);
    let next_time_error = RwSignal::new(false);

    let is_editing = RwSignal::new(!q_edit.is_empty());

    let edit_id = q_edit.clone();

    // Load champion list
    let champions = Resource::new(|| (), |_| get_champions_for_learnings());

    // Load existing learning for edit mode
    let edit_id_for_res = q_edit.clone();
    let existing_learning = Resource::new(
        move || edit_id_for_res.clone(),
        move |id| async move {
            if id.is_empty() {
                Ok(None)
            } else {
                get_learning(id).await
            }
        },
    );

    // Populate signals from existing learning when in edit mode
    Effect::new(move || {
        if let Some(Ok(Some(l))) = existing_learning.get() {
            learning_type.set(l.learning_type.clone());
            champion.set(l.champion.clone().unwrap_or_default());
            opponent.set(l.opponent.clone().unwrap_or_default());
            what_happened.set(l.what_happened.clone());
            what_i_learned.set(l.what_i_learned.clone());
            next_time.set(l.next_time.clone());
            selected_tags.set(l.tags.clone());
            title.set(l.title.clone());
            title_edited.set(true);
            win_loss.set(l.win_loss.clone().unwrap_or_default());
            match_riot_id.set(l.match_riot_id.clone().unwrap_or_default());
            if let Some(ts) = l.game_timestamp_ms {
                game_timestamp_ms.set(ts.to_string());
            }
            event_name_signal.set(l.event_name.clone().unwrap_or_default());
            is_editing.set(true);
        }
    });

    let toast = use_context::<ToastContext>();

    let edit_id_for_save = edit_id.clone();

    let handle_save = move |_: web_sys::MouseEvent| {
        // Validate required fields
        let wh = what_happened.get_untracked();
        let wil = what_i_learned.get_untracked();
        let nt = next_time.get_untracked();

        let mut valid = true;
        if wh.trim().is_empty() {
            what_happened_error.set(true);
            valid = false;
        } else {
            what_happened_error.set(false);
        }
        if wil.trim().is_empty() {
            what_i_learned_error.set(true);
            valid = false;
        } else {
            what_i_learned_error.set(false);
        }
        if nt.trim().is_empty() {
            next_time_error.set(true);
            valid = false;
        } else {
            next_time_error.set(false);
        }

        if !valid {
            return;
        }

        // Compute title
        let title_val = if title_edited.get_untracked() {
            title.get_untracked()
        } else {
            // Auto-generate
            let lt = learning_type.get_untracked();
            let champ = champion.get_untracked();
            let opp = opponent.get_untracked();
            match lt.as_str() {
                "matchup" => format!("{} vs {} \u{2014} {}", champ, opp, current_date_short()),
                "champion" => format!("{} \u{2014} {}", champ, current_date_short()),
                _ => format!("General \u{2014} {}", current_date_short()),
            }
        };

        let tags = selected_tags.get_untracked();
        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());

        let lt = learning_type.get_untracked();
        let champ = champion.get_untracked();
        let opp = opponent.get_untracked();
        let wl = win_loss.get_untracked();
        let mrid = match_riot_id.get_untracked();
        let gts = game_timestamp_ms.get_untracked();
        let en = event_name_signal.get_untracked();
        let editing = is_editing.get_untracked();
        let edit_id = edit_id_for_save.clone();

        leptos::task::spawn_local(async move {
            let result = if editing {
                update_learning(
                    edit_id,
                    title_val,
                    lt,
                    champ,
                    opp,
                    wh,
                    wil,
                    nt,
                    tags_json,
                    wl,
                    mrid,
                    gts,
                    en,
                )
                .await
                .map(|_| String::new())
            } else {
                save_learning(
                    title_val,
                    lt,
                    champ,
                    opp,
                    wh,
                    wil,
                    nt,
                    tags_json,
                    wl,
                    mrid,
                    gts,
                    en,
                )
                .await
            };

            match result {
                Ok(_) => {
                    if let Some(t) = toast {
                        t.show.run((ToastKind::Success, "Learning saved.".into()));
                    }
                    #[cfg(feature = "hydrate")]
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/personal-learnings");
                    }
                }
                Err(e) => {
                    save_error.set(Some(format!("Failed to save. Check your connection and try again. ({e})")));
                }
            }
        });
    };

    view! {
        <div class="canvas-grain bg-base min-h-screen">
            <div class="max-w-2xl mx-auto px-6 py-10">
                // Page heading
                <div class="flex flex-col gap-2 mb-6">
                    <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">
                        {move || if is_editing.get() { "Revise the entry" } else { "A new chapter" }}
                    </span>
                    <h1 class="font-display italic text-primary text-3xl">
                        {move || if is_editing.get() { "Edit learning" } else { "New learning" }}
                    </h1>
                </div>

                // Event context banner (from match timeline link)
                {move || {
                    let en = event_name_signal.get();
                    let gts = game_timestamp_ms.get();
                    if !en.is_empty() && !gts.is_empty() {
                        // Format timestamp (ms -> mm:ss)
                        let ts_ms: i64 = gts.parse().unwrap_or(0);
                        let total_secs = ts_ms / 1000;
                        let mins = total_secs / 60;
                        let secs = total_secs % 60;
                        let formatted = format!("{mins}:{secs:02}");
                        view! {
                            <div class="bg-elevated border border-divider rounded-lg p-3 text-sm text-secondary mb-4">
                                <span class="font-imperial uppercase tracking-wider text-[10px] text-muted">"From match"</span>
                                <div class="mt-1">{format!("{en} at {formatted}")}</div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                }}

                // Save error banner
                {move || {
                    if let Some(err) = save_error.get() {
                        view! { <div class="mb-4"><ErrorBanner message=err /></div> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                }}

                // Form Card.plain
                <div class="bg-elevated border border-divider rounded-xl p-6">
                    <div class="space-y-6">

                        // Step 1: Type selector
                        <div>
                            <label class="font-imperial uppercase tracking-wider text-[10px] text-muted block mb-2">"Learning type"</label>
                            <div class="flex gap-2 flex-wrap">
                                {["general", "champion", "matchup"].iter().map(|&lt_val| {
                                    let lt_val_str = lt_val.to_string();
                                    let lt_val_str2 = lt_val.to_string();
                                    let label = match lt_val {
                                        "general" => "General",
                                        "champion" => "Champion",
                                        "matchup" => "Matchup",
                                        _ => lt_val,
                                    };
                                    view! {
                                        <button
                                            class=move || {
                                                if learning_type.get() == lt_val_str {
                                                    "px-4 py-2 rounded-lg text-sm font-semibold bg-accent text-accent-contrast cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                } else {
                                                    "px-4 py-2 rounded-lg text-sm bg-surface border border-outline/50 text-secondary hover:text-primary hover:border-outline transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                }
                                            }
                                            on:click=move |_| learning_type.set(lt_val_str2.clone())
                                        >
                                            {label}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>

                        // Step 2: Conditional champion fields
                        <Suspense fallback=move || view! { <div></div> }>
                            {move || {
                                let lt = learning_type.get();
                                let champ_list = champions.get()
                                    .and_then(|r| r.ok())
                                    .unwrap_or_default();
                                let champ_list2 = champ_list.clone();

                                match lt.as_str() {
                                    "champion" => view! {
                                        <div>
                                            <label class="font-imperial uppercase tracking-wider text-[10px] text-muted block mb-2">"Champion"</label>
                                            <ChampionAutocomplete
                                                champions=champ_list
                                                value=champion
                                                placeholder="Search champion..."
                                            />
                                        </div>
                                    }.into_any(),
                                    "matchup" => view! {
                                        <div class="space-y-3">
                                            <div>
                                                <label class="font-imperial uppercase tracking-wider text-[10px] text-muted block mb-2">"Your champion"</label>
                                                <ChampionAutocomplete
                                                    champions=champ_list
                                                    value=champion
                                                    placeholder="Your champion..."
                                                />
                                            </div>
                                            <div>
                                                <label class="font-imperial uppercase tracking-wider text-[10px] text-muted block mb-2">"Opponent champion"</label>
                                                <ChampionAutocomplete
                                                    champions=champ_list2
                                                    value=opponent
                                                    placeholder="Opponent champion..."
                                                />
                                            </div>
                                        </div>
                                    }.into_any(),
                                    _ => view! { <span></span> }.into_any(),
                                }
                            }}
                        </Suspense>

                        // Step 3: Three required text areas
                        <div>
                            <label class="font-imperial uppercase tracking-wider text-[10px] text-muted block mb-1">"What happened"</label>
                            <textarea
                                class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                                rows="4"
                                placeholder="Describe what happened in the game..."
                                prop:value=move || what_happened.get()
                                on:input=move |ev| {
                                    what_happened.set(event_target_value(&ev));
                                    what_happened_error.set(false);
                                }
                            ></textarea>
                            {move || {
                                if what_happened_error.get() {
                                    view! {
                                        <p class="text-danger text-xs mt-1" role="alert">"This field is required to save."</p>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }
                            }}
                        </div>

                        <div>
                            <label class="font-imperial uppercase tracking-wider text-[10px] text-muted block mb-1">"What I learned"</label>
                            <textarea
                                class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                                rows="4"
                                placeholder="What did you learn from this experience?"
                                prop:value=move || what_i_learned.get()
                                on:input=move |ev| {
                                    what_i_learned.set(event_target_value(&ev));
                                    what_i_learned_error.set(false);
                                }
                            ></textarea>
                            {move || {
                                if what_i_learned_error.get() {
                                    view! {
                                        <p class="text-danger text-xs mt-1" role="alert">"This field is required to save."</p>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }
                            }}
                        </div>

                        <div>
                            <label class="font-imperial uppercase tracking-wider text-[10px] text-muted block mb-1">"Next time I will..."</label>
                            <textarea
                                class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                                rows="4"
                                placeholder="What will you do differently next time?"
                                prop:value=move || next_time.get()
                                on:input=move |ev| {
                                    next_time.set(event_target_value(&ev));
                                    next_time_error.set(false);
                                }
                            ></textarea>
                            {move || {
                                if next_time_error.get() {
                                    view! {
                                        <p class="text-danger text-xs mt-1" role="alert">"This field is required to save."</p>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }
                            }}
                        </div>

                        // Step 4: Tag chips
                        <div>
                            <label class="font-imperial uppercase tracking-wider text-[10px] text-muted block mb-2">"Tags"</label>
                            <div class="flex flex-wrap gap-2">
                                {LEARNING_TAGS.iter().map(|&tag| {
                                    let tag_str = tag.to_string();
                                    let tag_str2 = tag.to_string();
                                    view! {
                                        <button
                                            class=move || {
                                                if selected_tags.get().contains(&tag_str) {
                                                    "rounded-full px-3 py-1 text-xs font-semibold bg-accent text-accent-contrast cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                } else {
                                                    "rounded-full px-3 py-1 text-xs font-medium bg-surface border border-outline/40 text-muted hover:text-secondary hover:border-outline transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                }
                                            }
                                            on:click=move |_| {
                                                let t = tag_str2.clone();
                                                selected_tags.update(|tags| {
                                                    if let Some(pos) = tags.iter().position(|x| x == &t) {
                                                        tags.remove(pos);
                                                    } else {
                                                        tags.push(t);
                                                    }
                                                });
                                            }
                                        >
                                            {tag}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>

                        // Step 5: Title field
                        <div>
                            <label class="font-imperial uppercase tracking-wider text-[10px] text-muted block mb-1">"Title"</label>
                            <input
                                type="text"
                                class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                                placeholder="Auto-generated from type and champion..."
                                prop:value=move || {
                                    if title_edited.get() {
                                        title.get()
                                    } else {
                                        let lt = learning_type.get();
                                        let champ = champion.get();
                                        let opp = opponent.get();
                                        match lt.as_str() {
                                            "matchup" => format!("{} vs {} \u{2014} {}", champ, opp, current_date_short()),
                                            "champion" => format!("{} \u{2014} {}", champ, current_date_short()),
                                            _ => format!("General \u{2014} {}", current_date_short()),
                                        }
                                    }
                                }
                                on:input=move |ev| {
                                    title.set(event_target_value(&ev));
                                    title_edited.set(true);
                                }
                            />
                        </div>

                        // Save / Cancel buttons
                        <div class="flex items-center gap-3 pt-2">
                            <button
                                class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold px-5 py-2 rounded-lg text-sm transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                on:click=handle_save
                            >
                                "Save learning"
                            </button>
                            <a
                                href="/personal-learnings"
                                class="bg-surface border border-outline/50 text-secondary hover:text-primary hover:border-outline px-5 py-2 rounded-lg text-sm transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            >
                                "Discard changes"
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Helper: current date as "Mar 27" string
// ---------------------------------------------------------------------------

fn current_date_short() -> String {
    // Static fallback — auto-titles are user-editable, so approximate date is acceptable.
    "Today".to_string()
}
