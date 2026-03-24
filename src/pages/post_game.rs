use crate::components::ui::{ErrorBanner, SkeletonCard, ToastContext, ToastKind};
use crate::models::draft::Draft;
use crate::models::game_plan::{GamePlan, PostGameLearning};
use leptos::prelude::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn list_reviews() -> Result<Vec<PostGameLearning>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_post_game_learnings(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_plans_for_postgame() -> Result<Vec<GamePlan>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_game_plans(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_drafts_for_postgame() -> Result<Vec<Draft>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_drafts(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_recent_match_ids() -> Result<Vec<String>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    let rows = db::get_team_match_stats(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut seen = std::collections::HashSet::new();
    let ids: Vec<String> = rows
        .into_iter()
        .filter(|r| seen.insert(r.riot_match_id.clone()))
        .map(|r| r.riot_match_id)
        .collect();
    Ok(ids)
}

#[server]
pub async fn get_linked_plan(plan_id: String) -> Result<Option<GamePlan>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::get_game_plan(&surreal, &plan_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_review(review_json: String) -> Result<(String, usize), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    let mut review: PostGameLearning = serde_json::from_str(&review_json)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;
    review.team_id = team_id.clone();
    review.created_by = user.id;

    let improvements = review.improvements.clone();
    let review_id = db::save_post_game_learning(&surreal, review)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let items_created = db::batch_create_action_items_from_review(
        &surreal,
        &team_id,
        &review_id,
        &improvements,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok((review_id, items_created))
}

#[server]
pub async fn update_review(review_json: String) -> Result<usize, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let review: PostGameLearning = serde_json::from_str(&review_json)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;

    let review_id = review.id.clone().unwrap_or_default();
    let team_id = review.team_id.clone();
    let improvements = review.improvements.clone();
    db::update_post_game_learning(&surreal, review)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let items_created = db::batch_create_action_items_from_review(
        &surreal,
        &team_id,
        &review_id,
        &improvements,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(items_created)
}

#[server]
pub async fn delete_review(review_id: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::delete_post_game_learning(&surreal, &review_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Auto-detect win/loss outcome from Riot API by matching the linked draft's picks
/// against the user's recent match history.
#[server]
pub async fn auto_detect_outcome(draft_id: String) -> Result<Option<String>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use crate::server::riot;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;

    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    // No Riot account linked — cannot auto-detect
    let puuid = match user.riot_puuid {
        Some(ref p) if !p.is_empty() => p.clone(),
        _ => return Ok(None),
    };

    // Load the draft to find our side's picked champions
    let draft = match db::get_draft_for_prefill(&surreal, &draft_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(d) => d,
        None => return Ok(None),
    };

    let our_side = draft.our_side.as_str();

    // Collect champions that were picked (not banned) on our side
    let our_picks: Vec<String> = draft
        .actions
        .iter()
        .filter(|a| a.side == our_side && !a.phase.starts_with("ban") && !a.champion.is_empty())
        .map(|a| a.champion.to_lowercase())
        .collect();

    if our_picks.is_empty() {
        return Ok(None);
    }

    if !riot::has_api_key() {
        return Ok(None);
    }

    // Fetch recent match history (up to 20 matches; check first 5 for a match)
    let matches = riot::fetch_match_history(&puuid, None)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    for m in matches.into_iter().take(5) {
        let played_champ = m.champion.to_lowercase();
        if our_picks.contains(&played_champ) {
            let outcome = if m.win { "win" } else { "loss" };
            return Ok(Some(outcome.to_string()));
        }
    }

    Ok(None) // No matching game found in recent history
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn textarea_class() -> &'static str {
    "w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus:outline-none focus:border-accent/50 resize-none transition-colors"
}

fn input_class() -> &'static str {
    "w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus:outline-none focus:border-accent/50 transition-colors"
}

/// Basic pattern analysis: find recurring themes across reviews
#[allow(clippy::type_complexity)]
fn analyze_patterns(reviews: &[PostGameLearning]) -> (Vec<(String, usize)>, Vec<(String, usize)>) {
    let mut good_counts: HashMap<String, usize> = HashMap::new();
    let mut improve_counts: HashMap<String, usize> = HashMap::new();

    for r in reviews {
        for item in &r.what_went_well {
            let key = item.to_lowercase();
            *good_counts.entry(key).or_default() += 1;
        }
        for item in &r.improvements {
            let key = item.to_lowercase();
            *improve_counts.entry(key).or_default() += 1;
        }
    }

    let mut good: Vec<(String, usize)> = good_counts.into_iter().filter(|(_, c)| *c >= 2).collect();
    good.sort_by_key(|a| std::cmp::Reverse(a.1));
    good.truncate(5);

    let mut bad: Vec<(String, usize)> = improve_counts
        .into_iter()
        .filter(|(_, c)| *c >= 2)
        .collect();
    bad.sort_by_key(|a| std::cmp::Reverse(a.1));
    bad.truncate(5);

    (good, bad)
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn PostGamePage() -> impl IntoView {
    // Auth redirect
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    // URL query params
    use leptos_router::hooks::use_query_map;
    let query = use_query_map();
    let url_review_id = move || query.read().get("review_id");
    let url_plan_id = move || query.read().get("plan_id");
    let url_draft_id = move || query.read().get("draft_id");
    let (url_review_loaded, set_url_review_loaded) = signal(false);

    let reviews = Resource::new(|| (), |_| list_reviews());
    let plans = Resource::new(|| (), |_| list_plans_for_postgame());
    let drafts = Resource::new(|| (), |_| list_drafts_for_postgame());
    let match_ids = Resource::new(|| (), |_| get_recent_match_ids());

    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let action_item_count: RwSignal<Option<usize>> = RwSignal::new(None);
    let (editing_id, set_editing_id) = signal(Option::<String>::None);
    let (match_riot_id, set_match_riot_id) = signal(String::new());
    let (game_plan_id, set_game_plan_id) = signal(String::new());
    let (draft_id, set_draft_id) = signal(String::new());
    let (went_well, set_went_well) = signal(String::new());
    let (improvements, set_improvements) = signal(String::new());
    let (action_items, set_action_items) = signal(String::new());
    let (open_notes, set_open_notes) = signal(String::new());
    let (win_loss, set_win_loss) = signal::<Option<String>>(None);
    let (rating, set_rating) = signal::<Option<u8>>(None);
    let (fetching, set_fetching) = signal(false);
    let (fetch_status, set_fetch_status) = signal::<Option<String>>(None);

    let clear_editor = move || {
        set_editing_id.set(None);
        set_match_riot_id.set(String::new());
        set_game_plan_id.set(String::new());
        set_draft_id.set(String::new());
        set_went_well.set(String::new());
        set_improvements.set(String::new());
        set_action_items.set(String::new());
        set_open_notes.set(String::new());
        set_win_loss.set(None);
        set_rating.set(None);
        set_fetching.set(false);
        set_fetch_status.set(None);
    };

    let load_review = move |r: &PostGameLearning| {
        set_editing_id.set(r.id.clone());
        set_match_riot_id.set(r.match_riot_id.clone().unwrap_or_default());
        set_game_plan_id.set(r.game_plan_id.clone().unwrap_or_default());
        set_draft_id.set(r.draft_id.clone().unwrap_or_default());
        set_went_well.set(r.what_went_well.join("\n"));
        set_improvements.set(r.improvements.join("\n"));
        set_action_items.set(r.action_items.join("\n"));
        set_open_notes.set(r.open_notes.clone().unwrap_or_default());
        set_win_loss.set(r.win_loss.clone());
        set_rating.set(r.rating);
    };

    // URL param: auto-load review when ?review_id=X is present
    Effect::new(move |_| {
        if url_review_loaded.get_untracked() {
            return;
        }
        let Some(target_id) = url_review_id() else { return };
        if let Some(Ok(list)) = reviews.get() {
            if let Some(r) = list.iter().find(|r| r.id.as_deref() == Some(&target_id)) {
                load_review(r);
                set_url_review_loaded.set(true);
            }
        }
    });

    // URL param: seed plan_id and draft_id signals when present (for new review flows)
    Effect::new(move |_| {
        if url_review_loaded.get_untracked() {
            return;
        }
        if let Some(pid) = url_plan_id() {
            if game_plan_id.get_untracked().is_empty() {
                set_game_plan_id.set(pid);
            }
        }
        if let Some(did) = url_draft_id() {
            if draft_id.get_untracked().is_empty() {
                set_draft_id.set(did);
            }
        }
    });

    let build_review = move || -> PostGameLearning {
        PostGameLearning {
            id: editing_id.get_untracked(),
            team_id: String::new(),
            match_riot_id: {
                let s = match_riot_id.get_untracked();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            game_plan_id: {
                let s = game_plan_id.get_untracked();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            draft_id: {
                let s = draft_id.get_untracked();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            what_went_well: went_well
                .get_untracked()
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            improvements: improvements
                .get_untracked()
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            action_items: action_items
                .get_untracked()
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            open_notes: {
                let s = open_notes.get_untracked();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            created_by: String::new(),
            win_loss: win_loss.get_untracked(),
            rating: rating.get_untracked(),
        }
    };

    let do_save = move |_| {
        let review = build_review();
        let json = serde_json::to_string(&review).unwrap_or_default();
        let is_update = editing_id.get_untracked().is_some();

        leptos::task::spawn_local(async move {
            if is_update {
                match update_review(json).await {
                    Ok(n_items) => {
                        action_item_count.set(if n_items > 0 { Some(n_items) } else { None });
                        toast.show.run((ToastKind::Success, "Review updated".into()));
                        reviews.refetch();
                    }
                    Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                }
            } else {
                match create_review(json).await {
                    Ok((id, n_items)) => {
                        if !id.is_empty() {
                            set_editing_id.set(Some(id));
                        }
                        action_item_count.set(if n_items > 0 { Some(n_items) } else { None });
                        toast.show.run((ToastKind::Success, "Review saved".into()));
                        reviews.refetch();
                    }
                    Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                }
            }
        });
    };

    let do_delete = move |review_id: String| {
        leptos::task::spawn_local(async move {
            match delete_review(review_id).await {
                Ok(_) => {
                    clear_editor();
                    toast.show.run((ToastKind::Success, "Review deleted".into()));
                    reviews.refetch();
                }
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        });
    };

    view! {
        <div class="max-w-[80rem] mx-auto py-8 px-6 flex flex-col gap-6">
            <div>
                <h1 class="text-3xl font-bold text-primary">"Post-Game Review"</h1>
                <p class="text-muted text-sm mt-1">"Analyze games, track patterns, and improve together"</p>
            </div>

            {move || action_item_count.get().map(|n| {
                let label = if n == 1 {
                    "1 action item created".to_string()
                } else {
                    format!("{n} action items created")
                };
                view! {
                    <div class="bg-accent/10 border border-accent/30 text-secondary text-sm rounded-xl px-4 py-3">
                        {label}" — "<a href="/action-items" class="text-accent underline">"View"</a>
                    </div>
                }
            })}

            <div class="flex gap-6 min-h-[36rem]">
                // Left: review list + patterns
                <div class="w-72 flex-shrink-0 flex flex-col gap-4">
                    <button
                        class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm transition-colors"
                        on:click=move |_| clear_editor()
                    >"+ New Review"</button>

                    // Saved reviews
                    <Suspense fallback=|| view! { <div class="flex flex-col gap-2"><SkeletonCard height="h-12" /><SkeletonCard height="h-12" /><SkeletonCard height="h-12" /></div> }>
                        {move || reviews.get().map(|result| match result {
                            Ok(list) if list.is_empty() => view! {
                                <div class="text-center py-6">
                                    <p class="text-dimmed text-sm mb-3">"No post-game reviews yet"</p>
                                    <p class="text-dimmed text-xs mb-4">"Create or join a team to get started."</p>
                                    <a href="/team/roster" class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-3 py-1.5 text-xs transition-colors">
                                        "Go to Team"
                                    </a>
                                </div>
                            }.into_any(),
                            Ok(list) => {
                                let list_for_patterns = list.clone();
                                let (good_patterns, bad_patterns) = analyze_patterns(&list_for_patterns);
                                let has_patterns = !good_patterns.is_empty() || !bad_patterns.is_empty();
                                view! {
                                    <div class="flex flex-col gap-1.5">
                                        {list.into_iter().map(|r| {
                                            let review_for_load = r.clone();
                                            let rid = r.id.clone().unwrap_or_default();
                                            let rid_for_cls = rid.clone();
                                            let rid_for_delete = rid.clone();
                                            let match_label = r.match_riot_id.clone().unwrap_or_else(|| "No match linked".to_string());
                                            let item_count = r.what_went_well.len() + r.improvements.len() + r.action_items.len();
                                            view! {
                                                <div class=move || {
                                                    if editing_id.get().as_deref() == Some(&rid_for_cls) {
                                                        "bg-accent/10 border border-accent/30 rounded-lg p-3 transition-all"
                                                    } else {
                                                        "bg-elevated/30 border border-divider/30 rounded-lg p-3 hover:bg-overlay/30 transition-all"
                                                    }
                                                }>
                                                    <button
                                                        class="w-full text-left"
                                                        on:click=move |_| load_review(&review_for_load)
                                                    >
                                                        <div class="text-primary text-sm font-medium truncate">{match_label}</div>
                                                        <div class="text-dimmed text-xs mt-0.5">{format!("{item_count} points")}</div>
                                                    </button>
                                                    <button
                                                        class="text-red-400/50 hover:text-red-400 text-xs mt-1 transition-colors"
                                                        on:click=move |_| do_delete(rid_for_delete.clone())
                                                    >"Delete"</button>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>

                                    // Pattern analysis
                                    {has_patterns.then(|| view! {
                                        <div class="mt-4 bg-elevated/50 border border-divider/50 rounded-xl p-4 flex flex-col gap-3">
                                            <h4 class="text-primary font-semibold text-xs uppercase tracking-wider">"Recurring Patterns"</h4>
                                            {(!good_patterns.is_empty()).then(|| {
                                                let patterns = good_patterns.clone();
                                                view! {
                                                    <div>
                                                        <span class="text-emerald-400 text-xs font-medium">"Strengths"</span>
                                                        {patterns.into_iter().map(|(item, count)| view! {
                                                            <div class="flex items-center gap-2 mt-1">
                                                                <span class="text-secondary text-xs truncate flex-1">{item}</span>
                                                                <span class="text-dimmed text-xs flex-shrink-0">{format!("{count}x")}</span>
                                                            </div>
                                                        }).collect_view()}
                                                    </div>
                                                }
                                            })}
                                            {(!bad_patterns.is_empty()).then(|| {
                                                let patterns = bad_patterns.clone();
                                                view! {
                                                    <div>
                                                        <span class="text-red-400 text-xs font-medium">"Recurring Issues"</span>
                                                        {patterns.into_iter().map(|(item, count)| view! {
                                                            <div class="flex items-center gap-2 mt-1">
                                                                <span class="text-secondary text-xs truncate flex-1">{item}</span>
                                                                <span class="text-dimmed text-xs flex-shrink-0">{format!("{count}x")}</span>
                                                            </div>
                                                        }).collect_view()}
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    })}
                                }.into_any()
                            },
                            Err(e) => view! {
                                <ErrorBanner message=format!("Failed to load reviews: {e}") />
                            }.into_any(),
                        })}
                    </Suspense>
                </div>

                // Right: editor
                <div class="flex-1 flex flex-col gap-5">
                    // Back-reference badges (when a review is loaded)
                    {move || {
                        let gp_id = game_plan_id.get();
                        let dr_id = draft_id.get();
                        let has_refs = !gp_id.is_empty() || !dr_id.is_empty();
                        if has_refs {
                            view! {
                                <div class="flex items-center gap-2 flex-wrap">
                                    {if !gp_id.is_empty() {
                                        view! {
                                            <a href="/game-plan"
                                               class="inline-flex items-center gap-1 bg-surface border border-outline/50 text-muted text-xs rounded px-2 py-1 hover:text-primary hover:border-accent/50 transition-colors">
                                                <span class="text-accent">"Game Plan"</span>
                                            </a>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                    {move || {
                                        let did = draft_id.get();
                                        if did.is_empty() {
                                            view! { <span></span> }.into_any()
                                        } else {
                                            view! {
                                                <a
                                                    href=format!("/draft?draft_id={did}")
                                                    class="inline-flex items-center gap-1 bg-surface border border-outline/50 text-muted text-xs rounded px-2 py-1 hover:text-primary hover:border-accent/50 transition-colors"
                                                >
                                                    <span class="text-accent">"Draft"</span>
                                                </a>
                                            }.into_any()
                                        }
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }
                    }}
                    // Linking: match, game plan, draft
                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4">
                        <h3 class="text-primary font-semibold text-sm mb-3">"Link to..."</h3>
                        <div class="grid grid-cols-3 gap-4">
                            // Match
                            <div>
                                <label class="block text-muted text-xs font-medium mb-1">"Match"</label>
                                <Suspense fallback=|| view! { <div class="h-9 bg-overlay/50 rounded-lg animate-pulse"></div> }>
                                    {move || match_ids.get().map(|result| match result {
                                        Ok(ids) => view! {
                                            <select class=input_class()
                                                prop:value=move || match_riot_id.get()
                                                on:change=move |ev| set_match_riot_id.set(event_target_value(&ev))
                                            >
                                                <option value="">"None"</option>
                                                {ids.into_iter().map(|id| {
                                                    let label = if id.len() > 20 {
                                                        format!("...{}", &id[id.len()-12..])
                                                    } else {
                                                        id.clone()
                                                    };
                                                    view! { <option value=id>{label}</option> }
                                                }).collect_view()}
                                            </select>
                                        }.into_any(),
                                        Err(_) => view! { <p class="text-dimmed text-xs">"No matches"</p> }.into_any(),
                                    })}
                                </Suspense>
                            </div>
                            // Game Plan
                            <div>
                                <label class="block text-muted text-xs font-medium mb-1">"Game Plan"</label>
                                <Suspense fallback=|| view! { <div class="h-9 bg-overlay/50 rounded-lg animate-pulse"></div> }>
                                    {move || plans.get().map(|result| match result {
                                        Ok(list) => view! {
                                            <select class=input_class()
                                                prop:value=move || game_plan_id.get()
                                                on:change=move |ev| set_game_plan_id.set(event_target_value(&ev))
                                            >
                                                <option value="">"None"</option>
                                                {list.into_iter().map(|p| {
                                                    let id = p.id.clone().unwrap_or_default();
                                                    let name = if p.name.is_empty() { "Untitled".to_string() } else { p.name };
                                                    view! { <option value=id>{name}</option> }
                                                }).collect_view()}
                                            </select>
                                        }.into_any(),
                                        Err(_) => view! { <p class="text-dimmed text-xs">"No plans"</p> }.into_any(),
                                    })}
                                </Suspense>
                            </div>
                            // Draft
                            <div>
                                <label class="block text-muted text-xs font-medium mb-1">"Draft"</label>
                                <Suspense fallback=|| view! { <div class="h-9 bg-overlay/50 rounded-lg animate-pulse"></div> }>
                                    {move || drafts.get().map(|result| match result {
                                        Ok(list) => view! {
                                            <select class=input_class()
                                                prop:value=move || draft_id.get()
                                                on:change=move |ev| set_draft_id.set(event_target_value(&ev))
                                            >
                                                <option value="">"None"</option>
                                                {list.into_iter().map(|d| {
                                                    let id = d.id.clone().unwrap_or_default();
                                                    let name = d.name.clone();
                                                    view! { <option value=id>{name}</option> }
                                                }).collect_view()}
                                            </select>
                                        }.into_any(),
                                        Err(_) => view! { <p class="text-dimmed text-xs">"No drafts"</p> }.into_any(),
                                    })}
                                </Suspense>
                            </div>
                        </div>
                    </div>

                    // Linked game plan summary
                    {move || {
                        let gp_id = game_plan_id.get();
                        if gp_id.is_empty() {
                            None
                        } else {
                            Some(view! { <LinkedPlanCard plan_id=gp_id /> })
                        }
                    }}

                    // Game Outcome and Plan Rating
                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4 flex flex-col gap-4">
                        // Win/Loss selector
                        <div class="flex flex-col gap-2">
                            <span class="text-sm font-semibold text-primary">"Game Outcome"</span>
                            <div class="flex gap-2">
                                // None / unset button
                                <button
                                    type="button"
                                    class=move || {
                                        let base = "px-3 py-2 rounded-lg text-sm font-semibold transition-colors";
                                        if win_loss.get().is_none() {
                                            format!("{base} bg-overlay text-muted")
                                        } else {
                                            format!("{base} bg-overlay/50 text-dimmed hover:text-primary")
                                        }
                                    }
                                    on:click=move |_| set_win_loss.set(None)
                                >"---"</button>
                                // Win button
                                <button
                                    type="button"
                                    class=move || {
                                        let base = "px-3 py-2 rounded-lg text-sm font-semibold transition-colors";
                                        if win_loss.get().as_deref() == Some("win") {
                                            format!("{base} bg-emerald-500/20 text-emerald-400 border border-emerald-500/30")
                                        } else {
                                            format!("{base} bg-overlay/50 text-dimmed hover:text-primary")
                                        }
                                    }
                                    on:click=move |_| set_win_loss.set(Some("win".into()))
                                >"Win"</button>
                                // Loss button
                                <button
                                    type="button"
                                    class=move || {
                                        let base = "px-3 py-2 rounded-lg text-sm font-semibold transition-colors";
                                        if win_loss.get().as_deref() == Some("loss") {
                                            format!("{base} bg-red-500/20 text-red-400 border border-red-500/30")
                                        } else {
                                            format!("{base} bg-overlay/50 text-dimmed hover:text-primary")
                                        }
                                    }
                                    on:click=move |_| set_win_loss.set(Some("loss".into()))
                                >"Loss"</button>
                            </div>

                            // Fetch Result button (shown when no outcome set and a draft is linked)
                            {move || {
                                let wl = win_loss.get();
                                let did = draft_id.get();
                                if wl.is_none() && !did.is_empty() {
                                    view! {
                                        <div class="flex items-center gap-2 flex-wrap">
                                            <button
                                                type="button"
                                                class="inline-flex items-center gap-1 bg-surface border border-outline/50 text-muted text-xs rounded px-2 py-1 hover:text-primary hover:border-accent/50 transition-colors disabled:opacity-50"
                                                on:click=move |_| {
                                                    let did = draft_id.get_untracked();
                                                    if did.is_empty() { return; }
                                                    set_fetching.set(true);
                                                    set_fetch_status.set(None);
                                                    leptos::task::spawn_local(async move {
                                                        match auto_detect_outcome(did).await {
                                                            Ok(Some(result)) => {
                                                                set_win_loss.set(Some(result));
                                                                set_fetch_status.set(Some("detected".into()));
                                                            }
                                                            Ok(None) => {
                                                                set_fetch_status.set(Some("Result not found \u{2014} select manually".into()));
                                                            }
                                                            Err(_) => {
                                                                set_fetch_status.set(Some("Result not found \u{2014} select manually".into()));
                                                            }
                                                        }
                                                        set_fetching.set(false);
                                                    });
                                                }
                                                prop:disabled=move || fetching.get()
                                            >
                                                {move || if fetching.get() { "Fetching..." } else { "Fetch result" }}
                                            </button>
                                            {move || {
                                                let status = fetch_status.get();
                                                let wl = win_loss.get();
                                                if let Some(s) = status {
                                                    if s == "detected" {
                                                        let cls = if wl.as_deref() == Some("win") {
                                                            "text-emerald-400 text-xs"
                                                        } else {
                                                            "text-red-400 text-xs"
                                                        };
                                                        let label = if wl.as_deref() == Some("win") { "Win detected" } else { "Loss detected" };
                                                        view! { <span class=cls>{label}</span> }.into_any()
                                                    } else {
                                                        view! { <span class="text-muted text-xs">{s}</span> }.into_any()
                                                    }
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }
                                            }}
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }
                            }}
                        </div>

                        // Star Rating input
                        <div class="flex flex-col gap-1">
                            <span class="text-sm font-semibold text-primary">"Plan Rating"</span>
                            <span class="text-xs text-muted">"How well did the plan work?"</span>
                            <div class="flex gap-1">
                                {(1u8..=5).map(|n| {
                                    view! {
                                        <button
                                            type="button"
                                            class=move || {
                                                if rating.get().map_or(false, |r| r >= n) {
                                                    "text-accent text-xl hover:text-accent transition-colors"
                                                } else {
                                                    "text-dimmed text-xl hover:text-accent/70 transition-colors"
                                                }
                                            }
                                            on:click=move |_| set_rating.set(Some(n))
                                            aria-label=format!("Rate how well the plan worked: {} of 5 stars", n)
                                        >
                                            {move || if rating.get().map_or(false, |r| r >= n) { "\u{2605}" } else { "\u{2606}" }}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    </div>

                    // Structured feedback
                    <div class="grid grid-cols-3 gap-4">
                        // What went well
                        <div class="bg-emerald-500/5 border border-emerald-500/20 rounded-xl p-4 flex flex-col gap-2">
                            <label class="text-emerald-400 text-xs font-semibold uppercase tracking-wider">"What Went Well"</label>
                            <textarea rows="8" class=textarea_class()
                                placeholder="Good dragon control\nSupport roaming paid off\nStrong level 1 invade"
                                prop:value=move || went_well.get()
                                on:input=move |ev| set_went_well.set(event_target_value(&ev))
                            />
                        </div>

                        // Improvements
                        <div class="bg-amber-500/5 border border-amber-500/20 rounded-xl p-4 flex flex-col gap-2">
                            <label class="text-amber-400 text-xs font-semibold uppercase tracking-wider">"Improvements Needed"</label>
                            <textarea rows="8" class=textarea_class()
                                placeholder="Ward coverage around Baron\nBetter grouping after towers\nMid-game macro decisions"
                                prop:value=move || improvements.get()
                                on:input=move |ev| set_improvements.set(event_target_value(&ev))
                            />
                        </div>

                        // Action items
                        <div class="bg-blue-500/5 border border-blue-500/20 rounded-xl p-4 flex flex-col gap-2">
                            <label class="text-blue-400 text-xs font-semibold uppercase tracking-wider">"Action Items"</label>
                            <textarea rows="8" class=textarea_class()
                                placeholder="Review VOD of teamfights\nPractice 2v2 bot lane\nDrill baron timings"
                                prop:value=move || action_items.get()
                                on:input=move |ev| set_action_items.set(event_target_value(&ev))
                            />
                        </div>
                    </div>

                    // Open notes
                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4">
                        <label class="block text-muted text-xs font-medium mb-1">"Open Notes"</label>
                        <textarea rows="4" class=textarea_class()
                            placeholder="Any additional thoughts, observations, or context..."
                            prop:value=move || open_notes.get()
                            on:input=move |ev| set_open_notes.set(event_target_value(&ev))
                        />
                    </div>

                    // Save buttons
                    <div class="flex gap-3 items-center">
                        <button
                            class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-6 py-2 text-sm transition-colors"
                            on:click=do_save
                        >
                            {move || if editing_id.get().is_some() { "Update Review" } else { "Save Review" }}
                        </button>
                        <button
                            class="bg-overlay hover:bg-overlay-strong text-secondary rounded-lg px-4 py-2 text-sm transition-colors"
                            on:click=move |_| clear_editor()
                        >"Clear"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Linked game plan summary card
// ---------------------------------------------------------------------------

#[component]
fn LinkedPlanCard(plan_id: String) -> impl IntoView {
    let pid = plan_id.clone();
    let linked_plan = Resource::new(move || pid.clone(), |id| get_linked_plan(id));

    view! {
        <Suspense fallback=|| view! { <SkeletonCard height="h-16" /> }>
            {move || linked_plan.get().map(|result| match result {
                Ok(Some(plan)) => {
                    let name = if plan.name.is_empty() { "Untitled Plan".to_string() } else { plan.name.clone() };
                    let our_champs = plan.our_champions.clone();
                    let enemy_champs = plan.enemy_champions.clone();
                    let win_conds = plan.win_conditions.clone();
                    let teamfight = plan.teamfight_strategy.clone();
                    let top = plan.top_strategy.clone();
                    let jg = plan.jungle_strategy.clone();
                    let mid = plan.mid_strategy.clone();
                    let bot = plan.bot_strategy.clone();
                    let sup = plan.support_strategy.clone();

                    view! {
                        <div class="bg-blue-500/5 border border-blue-500/20 rounded-xl p-4 flex flex-col gap-3">
                            <div class="flex items-center gap-2">
                                <span class="text-blue-400 text-xs font-semibold uppercase tracking-wider">"Game Plan Summary"</span>
                                <span class="text-primary text-sm font-medium">{name}</span>
                            </div>

                            // Champions
                            {(!our_champs.is_empty() || !enemy_champs.is_empty()).then(|| {
                                let ours = our_champs.join(", ");
                                let theirs = enemy_champs.join(", ");
                                view! {
                                    <div class="grid grid-cols-[1fr_auto_1fr] gap-2 text-xs">
                                        <div>
                                            <span class="text-blue-400 font-medium">"Our Team: "</span>
                                            <span class="text-secondary">{ours}</span>
                                        </div>
                                        <span class="text-dimmed">"vs"</span>
                                        <div>
                                            <span class="text-red-400 font-medium">"Enemy: "</span>
                                            <span class="text-secondary">{theirs}</span>
                                        </div>
                                    </div>
                                }
                            })}

                            // Win conditions
                            {(!win_conds.is_empty()).then(|| {
                                view! {
                                    <div class="text-xs">
                                        <span class="text-muted font-medium">"Win Conditions: "</span>
                                        <span class="text-secondary">{win_conds.join(" | ")}</span>
                                    </div>
                                }
                            })}

                            // Teamfight
                            {(!teamfight.is_empty()).then(|| {
                                view! {
                                    <div class="text-xs">
                                        <span class="text-muted font-medium">"Teamfight: "</span>
                                        <span class="text-secondary">{teamfight}</span>
                                    </div>
                                }
                            })}

                            // Role strategies (compact)
                            {
                                let roles = vec![
                                    ("Top", top),
                                    ("Jg", jg),
                                    ("Mid", mid),
                                    ("Bot", bot),
                                    ("Sup", sup),
                                ];
                                let has_any = roles.iter().any(|(_, s)| s.is_some());
                                has_any.then(|| {
                                    view! {
                                        <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs">
                                            {roles.into_iter().filter_map(|(role, strat)| {
                                                strat.map(|s| view! {
                                                    <div>
                                                        <span class="text-accent font-medium">{role}": "</span>
                                                        <span class="text-secondary">{s}</span>
                                                    </div>
                                                })
                                            }).collect_view()}
                                        </div>
                                    }
                                })
                            }
                        </div>
                    }.into_any()
                },
                Ok(None) => view! {
                    <div class="text-dimmed text-xs py-1">"Linked plan not found."</div>
                }.into_any(),
                Err(e) => view! {
                    <div class="text-red-400 text-xs py-1">{format!("Error loading plan: {e}")}</div>
                }.into_any(),
            })}
        </Suspense>
    }
}
