use crate::components::champion_picker::ChampionPicker;
use crate::components::draft_board::{slot_meta, DraftBoard};
use crate::components::ui::ErrorBanner;
use crate::models::champion::Champion;
use crate::models::draft::{BanPriority, Draft, DraftAction};
use crate::models::team::Team;
use leptos::prelude::*;
use std::collections::HashMap;

#[server]
pub async fn get_champions() -> Result<Vec<Champion>, ServerFnError> {
    use crate::server::data_dragon;
    data_dragon::fetch_champions()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn save_draft(
    name: String,
    opponent: Option<String>,
    team_id: Option<String>,
    actions_json: String,
    comments_json: String,
    rating: Option<String>,
    our_side: Option<String>,
    tags_json: String,
    win_conditions: Option<String>,
    watch_out: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let actions: Vec<DraftAction> = serde_json::from_str(&actions_json)
        .map_err(|e| ServerFnError::new(format!("Invalid actions JSON: {e}")))?;
    let comments: Vec<String> = serde_json::from_str(&comments_json)
        .map_err(|e| ServerFnError::new(format!("Invalid comments JSON: {e}")))?;
    let tags: Vec<String> = serde_json::from_str(&tags_json)
        .map_err(|e| ServerFnError::new(format!("Invalid tags JSON: {e}")))?;

    let resolved_team_id = match team_id.filter(|s| !s.is_empty()) {
        Some(tid) => tid,
        None => db::get_user_team_id(&db, &user.id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .ok_or_else(|| ServerFnError::new("You must be in a team to create a draft"))?,
    };

    db::save_draft(
        &db,
        &resolved_team_id,
        &user.id,
        name,
        opponent,
        None,
        comments,
        actions,
        rating,
        our_side.unwrap_or_else(|| "blue".to_string()),
        tags,
        win_conditions,
        watch_out,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_draft(
    draft_id: String,
    name: String,
    opponent: Option<String>,
    actions_json: String,
    comments_json: String,
    rating: Option<String>,
    our_side: Option<String>,
    tags_json: String,
    win_conditions: Option<String>,
    watch_out: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let actions: Vec<DraftAction> = serde_json::from_str(&actions_json)
        .map_err(|e| ServerFnError::new(format!("Invalid actions JSON: {e}")))?;
    let comments: Vec<String> = serde_json::from_str(&comments_json)
        .map_err(|e| ServerFnError::new(format!("Invalid comments JSON: {e}")))?;
    let tags: Vec<String> = serde_json::from_str(&tags_json)
        .map_err(|e| ServerFnError::new(format!("Invalid tags JSON: {e}")))?;

    db::update_draft(
        &db,
        &draft_id,
        name,
        opponent,
        None,
        comments,
        actions,
        rating,
        our_side.unwrap_or_else(|| "blue".to_string()),
        tags,
        win_conditions,
        watch_out,
    )
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
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_drafts(&db, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_user_teams() -> Result<Vec<Team>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::get_user_teams(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_ban_priorities() -> Result<Vec<BanPriority>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::get_ban_priorities(&db, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn save_ban_priorities(priorities_json: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    let priorities: Vec<BanPriority> = serde_json::from_str(&priorities_json)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;

    db::set_ban_priorities(&db, &team_id, priorities)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

fn build_actions(slots: Vec<Option<String>>, slot_comments: &[Option<String>]) -> Vec<DraftAction> {
    slots
        .into_iter()
        .enumerate()
        .filter_map(|(i, opt)| {
            opt.map(|champ| {
                let (side, kind, label) = slot_meta(i);
                DraftAction {
                    id: None,
                    draft_id: String::new(),
                    phase: format!("{}_{}", kind, label),
                    side: side.to_string(),
                    champion: champ,
                    order: i as i32,
                    comment: slot_comments.get(i).cloned().flatten(),
                }
            })
        })
        .collect()
}

const COMPOSITION_TAGS: &[&str] = &[
    "teamfight",
    "split-push",
    "poke",
    "pick",
    "scaling",
    "early-game",
    "protect-the-carry",
];

fn tier_badge_class(tier: &str) -> &'static str {
    match tier {
        "S+" => "bg-purple-500 text-white",
        "S" => "bg-accent text-accent-contrast",
        "A" => "bg-green-500 text-primary",
        "B" => "bg-blue-500 text-white",
        "C" => "bg-orange-500 text-primary",
        "D" => "bg-red-600 text-white",
        _ => "bg-overlay-strong text-secondary",
    }
}

const TIERS: &[&str] = &["S+", "S", "A", "B", "C", "D"];

#[component]
pub fn DraftPage() -> impl IntoView {
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

    let (draft_name, set_draft_name) = signal(String::new());
    let (opponent, set_opponent) = signal(String::new());
    let (selected_team_id, set_selected_team_id) = signal(String::new());
    let (rating, set_rating) = signal(Option::<String>::None);
    let (our_side, set_our_side) = signal("blue".to_string());
    let (draft_slots, set_draft_slots) = signal(vec![None::<String>; 20]);
    let (active_slot, set_active_slot) = signal(Some(0_usize));
    let (highlighted_slot, set_highlighted_slot) = signal(Option::<usize>::None);
    let (comments, set_comments) = signal(Vec::<String>::new());
    let (comment_input, set_comment_input) = signal(String::new());
    let (save_result, set_save_result) = signal(Option::<String>::None);
    let (loaded_draft_id, set_loaded_draft_id) = signal(Option::<String>::None);
    // Per-slot rationale comments (Phase 1)
    let (slot_comments, set_slot_comments) = signal(vec![None::<String>; 20]);
    let (slot_comment_input, set_slot_comment_input) = signal(String::new());
    // Composition tags + win conditions (Phase 2)
    let (tags, set_tags) = signal(Vec::<String>::new());
    let (win_conditions, set_win_conditions) = signal(String::new());
    let (watch_out, set_watch_out) = signal(String::new());
    // Tag filter for saved drafts list
    let (filter_tag, set_filter_tag) = signal(String::new());
    // Ban priorities (Phase 4)
    let ban_priorities = Resource::new(|| (), |_| get_ban_priorities());
    let (ban_panel_open, set_ban_panel_open) = signal(false);
    let (editing_bans, set_editing_bans) = signal(false);
    let (ban_edit_list, set_ban_edit_list) = signal(Vec::<BanPriority>::new());
    let (ban_new_champ, set_ban_new_champ) = signal(String::new());
    let (ban_new_reason, set_ban_new_reason) = signal(String::new());
    let (ban_status, set_ban_status) = signal(Option::<String>::None);

    let champions_resource = Resource::new(|| (), |_| get_champions());
    let drafts = Resource::new(|| (), |_| list_drafts());
    let teams_resource = Resource::new(|| (), |_| list_user_teams());

    // Auto-select first team when resource loads
    Effect::new(move |_| {
        if let Some(Ok(teams)) = teams_resource.get() {
            if selected_team_id.get_untracked().is_empty() {
                if let Some(first) = teams.first() {
                    set_selected_team_id.set(first.id.clone().unwrap_or_default());
                }
            }
        }
    });

    let used_champions = move || {
        draft_slots
            .get()
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
    };

    let fill_slot = move |slot_idx: usize, champion_name: String| {
        let already_used = draft_slots
            .get_untracked()
            .iter()
            .any(|s| s.as_deref() == Some(&champion_name));
        if already_used {
            return;
        }
        set_draft_slots.update(|s| s[slot_idx] = Some(champion_name));
        set_highlighted_slot.set(None);
        let updated = draft_slots.get_untracked();
        let next = (0..20).find(|&i| updated[i].is_none());
        set_active_slot.set(next);
    };

    let on_champion_select = Callback::new(move |champ: Champion| {
        if let Some(slot) = active_slot.get_untracked() {
            fill_slot(slot, champ.name);
        }
    });

    let on_slot_drop = Callback::new(move |(slot_idx, name): (usize, String)| {
        fill_slot(slot_idx, name);
    });

    let on_slot_click = Callback::new(move |slot_idx: usize| {
        let slots = draft_slots.get_untracked();
        if slots.get(slot_idx).and_then(|s| s.as_ref()).is_some() {
            let currently_highlighted = highlighted_slot.get_untracked();
            if currently_highlighted == Some(slot_idx) {
                // Second click: set as active_slot for champion replacement
                set_active_slot.set(Some(slot_idx));
            } else {
                // First click: just highlight, set as active
                set_highlighted_slot.set(Some(slot_idx));
                set_active_slot.set(Some(slot_idx));
            }
        } else {
            set_highlighted_slot.set(None);
            set_active_slot.update(|a| {
                *a = if *a == Some(slot_idx) {
                    None
                } else {
                    Some(slot_idx)
                };
            });
        }
    });

    let on_slot_clear = Callback::new(move |slot_idx: usize| {
        set_draft_slots.update(|s| s[slot_idx] = None);
        set_highlighted_slot.set(None);
        set_active_slot.set(Some(slot_idx));
    });

    let phase_label = move || match active_slot.get() {
        Some(0..=5) => "Phase 1 — Bans",
        Some(6..=11) => "Phase 1 — Picks",
        Some(12..=15) => "Phase 2 — Bans",
        Some(16..=19) => "Phase 2 — Picks",
        None => "Draft Complete",
        _ => "",
    };

    let active_slot_label = move || {
        active_slot.get().map(|i| {
            let (side, _, label) = slot_meta(i);
            let side_cap = if side == "blue" { "Blue" } else { "Red" };
            format!("Selecting for: {side_cap} {label}")
        })
    };

    let do_save = move |_| {
        let name = draft_name.get_untracked();
        if name.trim().is_empty() {
            set_save_result.set(Some("Give this draft a name before saving.".into()));
            return;
        }
        let opp = opponent.get_untracked();
        let tid = selected_team_id.get_untracked();
        let rate = rating.get_untracked();
        let side = our_side.get_untracked();
        let sc = slot_comments.get_untracked();
        let actions = build_actions(draft_slots.get_untracked(), &sc);
        let acts_json = serde_json::to_string(&actions).unwrap_or_default();
        let cmts_json = serde_json::to_string(&comments.get_untracked()).unwrap_or_default();
        let tags_json = serde_json::to_string(&tags.get_untracked()).unwrap_or_default();
        let wc = { let s = win_conditions.get_untracked(); if s.is_empty() { None } else { Some(s) } };
        let wo = { let s = watch_out.get_untracked(); if s.is_empty() { None } else { Some(s) } };
        let existing_id = loaded_draft_id.get_untracked();

        leptos::task::spawn_local(async move {
            let opp_opt = if opp.is_empty() { None } else { Some(opp) };
            let team_opt = if tid.is_empty() { None } else { Some(tid) };

            if let Some(draft_id) = existing_id {
                match update_draft(
                    draft_id, name, opp_opt, acts_json, cmts_json, rate, Some(side),
                    tags_json, wc, wo,
                )
                .await
                {
                    Ok(_) => {
                        set_save_result.set(Some("Updated!".into()));
                        drafts.refetch();
                    }
                    Err(e) => set_save_result.set(Some(format!("Error: {e}"))),
                }
            } else {
                match save_draft(
                    name, opp_opt, team_opt, acts_json, cmts_json, rate, Some(side),
                    tags_json, wc, wo,
                )
                .await
                {
                    Ok(id) => {
                        set_save_result.set(Some("Saved!".into()));
                        set_loaded_draft_id.set(Some(id));
                        drafts.refetch();
                    }
                    Err(e) => set_save_result.set(Some(format!("Error: {e}"))),
                }
            }
        });
    };

    // Sync slot_comment_input when highlighted_slot changes
    Effect::new(move |_| {
        let hl = highlighted_slot.get();
        if let Some(idx) = hl {
            let sc = slot_comments.get_untracked();
            set_slot_comment_input.set(sc.get(idx).cloned().flatten().unwrap_or_default());
        } else {
            set_slot_comment_input.set(String::new());
        }
    });

    // Auto-save timer handle (only used in hydrate/WASM builds)
    #[allow(unused_variables)]
    let auto_save_timer: RwSignal<Option<i32>> = RwSignal::new(None);
    let (auto_save_status, set_auto_save_status) = signal(""); // "", "unsaved", "saved"

    #[allow(unused_variables)]
    Effect::new(move |_| {
        // Eagerly track + capture ALL content signals (CLAUDE.md rule 54)
        let slots_val = draft_slots.get();
        let comments_val = comments.get();
        let name = draft_name.get();
        let existing_id = loaded_draft_id.get();
        let opp = opponent.get();
        let rate = rating.get();
        let side = our_side.get();
        let sc = slot_comments.get();
        let tags_val = tags.get();
        let wc_val = win_conditions.get();
        let wo_val = watch_out.get();

        // Only auto-save if we have a name AND it's an existing draft
        if name.trim().is_empty() || existing_id.is_none() {
            return;
        }

        // Cancel pending timer
        #[cfg(feature = "hydrate")]
        if let Some(timer_id) = auto_save_timer.get_untracked() {
            if let Some(win) = web_sys::window() {
                win.clear_timeout_with_handle(timer_id);
            }
        }

        set_auto_save_status.set("unsaved");

        // Schedule new 2s auto-save
        #[cfg(feature = "hydrate")]
        {
            // Pre-compute all values before the closure
            let actions = build_actions(slots_val, &sc);
            let acts_json = serde_json::to_string(&actions).unwrap_or_default();
            let cmts_json = serde_json::to_string(&comments_val).unwrap_or_default();
            let tags_json = serde_json::to_string(&tags_val).unwrap_or_default();
            let wc = if wc_val.is_empty() { None } else { Some(wc_val) };
            let wo = if wo_val.is_empty() { None } else { Some(wo_val) };
            let opp_opt = if opp.is_empty() { None } else { Some(opp) };

            use wasm_bindgen::prelude::*;
            let cb = Closure::once(move || {
                if let Some(draft_id) = existing_id {
                    leptos::task::spawn_local(async move {
                        let _ = update_draft(
                            draft_id, name, opp_opt, acts_json, cmts_json, rate, Some(side),
                            tags_json, wc, wo,
                        )
                        .await;
                        set_auto_save_status.set("saved");
                        drafts.refetch();
                    });
                }
            });
            if let Some(win) = web_sys::window() {
                if let Ok(timer_id) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    2000,
                ) {
                    auto_save_timer.set(Some(timer_id));
                }
            }
            cb.forget();
        }
    });

    view! {
        <div class="max-w-6xl mx-auto py-8 px-6 flex flex-col gap-6">
            <div>
                <h1 class="text-3xl font-bold text-primary">"Draft Planner"</h1>
                <p class="text-accent font-medium mt-1">{phase_label}</p>
                {move || loaded_draft_id.get().map(|_| {
                    let status = auto_save_status.get();
                    let (cls, text) = match status {
                        "saved" => ("text-green-400 text-sm mt-0.5", "✓ Saved"),
                        "unsaved" => ("text-amber-400 text-sm mt-0.5", "● Unsaved changes"),
                        _ => ("text-muted text-sm mt-0.5", "Editing saved draft"),
                    };
                    view! { <p class=cls>{text}</p> }
                })}
            </div>

            // Header form
            <div class="bg-elevated border border-divider rounded-lg p-4 flex flex-col gap-4">
                <div class="grid grid-cols-3 gap-4">
                    // Draft Name
                    <div>
                        <label class="block text-secondary text-sm mb-1">"Draft Name"</label>
                        <input
                            type="text"
                            prop:value=move || draft_name.get()
                            class="w-full bg-overlay border border-outline rounded px-3 py-2 text-primary focus:outline-none focus:border-accent"
                            on:input=move |ev| set_draft_name.set(event_target_value(&ev))
                        />
                    </div>
                    // Team selection
                    <div>
                        <label class="block text-secondary text-sm mb-1">"Team"</label>
                        <Suspense fallback=|| view! { <div class="h-9 bg-overlay rounded animate-pulse"></div> }>
                            {move || teams_resource.get().map(|result| match result {
                                Ok(teams) if teams.is_empty() => view! {
                                    <p class="text-dimmed text-sm py-2">"Not part of a team yet."</p>
                                }.into_any(),
                                Ok(teams) => view! {
                                    <select
                                        prop:value=move || selected_team_id.get()
                                        on:change=move |ev| set_selected_team_id.set(event_target_value(&ev))
                                        class="w-full bg-overlay border border-outline rounded px-3 py-2 text-primary focus:outline-none focus:border-accent"
                                    >
                                        {teams.into_iter().map(|t| {
                                            let id = t.id.clone().unwrap_or_default();
                                            let name = t.name.clone();
                                            view! { <option value=id>{name}</option> }
                                        }).collect_view()}
                                    </select>
                                }.into_any(),
                                Err(_) => view! {
                                    <p class="text-red-400 text-sm py-2">"Failed to load teams."</p>
                                }.into_any(),
                            })}
                        </Suspense>
                    </div>
                    // Opponent
                    <div>
                        <label class="block text-secondary text-sm mb-1">"Opponent (optional)"</label>
                        <input
                            type="text"
                            prop:value=move || opponent.get()
                            class="w-full bg-overlay border border-outline rounded px-3 py-2 text-primary focus:outline-none focus:border-accent"
                            on:input=move |ev| set_opponent.set(event_target_value(&ev))
                        />
                    </div>
                </div>
                // Our Side toggle
                <div class="flex items-center gap-4">
                    <label class="text-secondary text-sm">"Our Side"</label>
                    <div class="flex gap-1">
                        <button
                            class=move || if our_side.get() == "blue" {
                                "px-3 py-1 rounded text-sm font-medium bg-blue-500 text-white cursor-pointer"
                            } else {
                                "px-3 py-1 rounded text-sm font-medium bg-overlay text-muted hover:bg-overlay-strong transition-colors cursor-pointer"
                            }
                            on:click=move |_| set_our_side.set("blue".to_string())
                        >"Blue"</button>
                        <button
                            class=move || if our_side.get() == "red" {
                                "px-3 py-1 rounded text-sm font-medium bg-red-500 text-white cursor-pointer"
                            } else {
                                "px-3 py-1 rounded text-sm font-medium bg-overlay text-muted hover:bg-overlay-strong transition-colors cursor-pointer"
                            }
                            on:click=move |_| set_our_side.set("red".to_string())
                        >"Red"</button>
                    </div>
                </div>

                // Rating picker
                <div>
                    <label class="block text-secondary text-sm mb-2">"Rating"</label>
                    <div class="flex gap-1.5">
                        {TIERS.iter().map(|&tier| {
                            view! {
                                <button
                                    class=move || {
                                        let selected = rating.get().as_deref() == Some(tier);
                                        if selected {
                                            format!("rounded px-3 py-1 text-sm font-bold transition-colors {}", tier_badge_class(tier))
                                        } else {
                                            "rounded px-3 py-1 text-sm font-bold transition-colors bg-overlay hover:bg-overlay-strong text-muted".to_string()
                                        }
                                    }
                                    on:click=move |_| {
                                        let current = rating.get_untracked();
                                        set_rating.set(
                                            if current.as_deref() == Some(tier) { None }
                                            else { Some(tier.to_string()) }
                                        );
                                    }
                                >
                                    {tier}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>

                // Composition tags
                <div>
                    <label class="block text-secondary text-sm mb-2">"Composition Tags"</label>
                    <div class="flex flex-wrap gap-1.5">
                        {COMPOSITION_TAGS.iter().map(|&tag| {
                            let tag_str = tag.to_string();
                            let tag_for_class = tag_str.clone();
                            let tag_for_click = tag_str.clone();
                            view! {
                                <button
                                    class=move || {
                                        let selected = tags.get().contains(&tag_for_class);
                                        if selected {
                                            "rounded px-3 py-1 text-sm font-medium bg-accent text-accent-contrast transition-colors cursor-pointer"
                                        } else {
                                            "rounded px-3 py-1 text-sm font-medium bg-overlay text-muted hover:bg-overlay-strong transition-colors cursor-pointer"
                                        }
                                    }
                                    on:click=move |_| {
                                        let tag_val = tag_for_click.clone();
                                        set_tags.update(|t| {
                                            if let Some(pos) = t.iter().position(|x| x == &tag_val) {
                                                t.remove(pos);
                                            } else {
                                                t.push(tag_val);
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

                // Win condition + watch out textareas
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-secondary text-sm mb-1">"How We Win"</label>
                        <textarea
                            rows="3"
                            placeholder="Win condition notes..."
                            class="w-full bg-overlay border border-outline rounded px-3 py-2 text-primary text-sm placeholder-gray-400 focus:outline-none focus:border-accent resize-none"
                            prop:value=move || win_conditions.get()
                            on:input=move |ev| set_win_conditions.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-secondary text-sm mb-1">"Watch Out For"</label>
                        <textarea
                            rows="3"
                            placeholder="Threats to be aware of..."
                            class="w-full bg-overlay border border-outline rounded px-3 py-2 text-primary text-sm placeholder-gray-400 focus:outline-none focus:border-accent resize-none"
                            prop:value=move || watch_out.get()
                            on:input=move |ev| set_watch_out.set(event_target_value(&ev))
                        />
                    </div>
                </div>
            </div>

            // Board + Comments
            <div class="flex gap-4">
                <div class="flex-1 bg-elevated border border-divider rounded-lg p-4">
                    <Suspense fallback=|| view! { <div class="text-muted text-center py-8">"Loading champions..."</div> }>
                        {move || champions_resource.get().map(|result| match result {
                            Err(e) => view! {
                                <div class="text-red-400">"Failed to load champions: " {e.to_string()}</div>
                            }.into_any(),
                            Ok(champs) => {
                                let champion_map: HashMap<String, Champion> = champs
                                    .into_iter()
                                    .map(|c| (c.name.clone(), c))
                                    .collect();
                                view! {
                                    <DraftBoard
                                        draft_slots=draft_slots
                                        champion_map=champion_map
                                        active_slot=active_slot
                                        on_slot_click=on_slot_click
                                        on_slot_drop=on_slot_drop
                                        highlighted_slot=highlighted_slot
                                        on_slot_clear=on_slot_clear
                                        slot_comments=slot_comments
                                    />
                                }.into_any()
                            }
                        })}
                    </Suspense>
                    // Per-slot comment editor (visible when a pick slot is highlighted)
                    {move || {
                        let hl = highlighted_slot.get();
                        hl.and_then(|idx| {
                            let (_, kind, _) = slot_meta(idx);
                            if kind != "pick" { return None; }
                            let slots = draft_slots.get();
                            let filled = slots.get(idx).and_then(|s| s.as_ref()).is_some();
                            if !filled { return None; }
                            let champ = slots[idx].clone().unwrap_or_default();
                            Some(view! {
                                <div class="mt-2 flex items-center gap-2">
                                    <span class="text-secondary text-sm flex-shrink-0">{format!("{} comment:", champ)}</span>
                                    <input
                                        type="text"
                                        placeholder="Pick rationale..."
                                        class="flex-1 bg-overlay border border-outline rounded px-2 py-1 text-primary text-sm focus:outline-none focus:border-accent"
                                        prop:value=move || slot_comment_input.get()
                                        on:input=move |ev| set_slot_comment_input.set(event_target_value(&ev))
                                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                                            if ev.key() == "Enter" {
                                                if let Some(slot_idx) = highlighted_slot.get_untracked() {
                                                    let val = slot_comment_input.get_untracked();
                                                    let comment = if val.trim().is_empty() { None } else { Some(val) };
                                                    set_slot_comments.update(|sc| {
                                                        if slot_idx < sc.len() { sc[slot_idx] = comment; }
                                                    });
                                                }
                                            }
                                        }
                                        on:blur=move |_| {
                                            if let Some(slot_idx) = highlighted_slot.get_untracked() {
                                                let val = slot_comment_input.get_untracked();
                                                let comment = if val.trim().is_empty() { None } else { Some(val) };
                                                set_slot_comments.update(|sc| {
                                                    if slot_idx < sc.len() { sc[slot_idx] = comment; }
                                                });
                                            }
                                        }
                                    />
                                </div>
                            })
                        })
                    }}
                </div>

                // Comments sidebar
                <div class="w-72 bg-elevated border border-divider rounded-lg p-4 flex flex-col gap-3">
                    <h3 class="text-primary font-bold">"Comments"</h3>
                    <div class="flex flex-col gap-1 max-h-64 overflow-y-auto flex-1">
                        {move || {
                            let list = comments.get();
                            if list.is_empty() {
                                view! { <p class="text-dimmed text-sm">"No comments yet."</p> }.into_any()
                            } else {
                                view! {
                                    <div class="flex flex-col gap-1">
                                        {list.into_iter().map(|c| view! {
                                            <div class="bg-surface rounded p-2 text-sm text-gray-200">{c}</div>
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                    <textarea
                        rows="3"
                        placeholder="Add a comment..."
                        class="w-full bg-overlay border border-outline rounded px-3 py-2 text-primary text-sm placeholder-gray-400 focus:outline-none focus:border-accent resize-none"
                        on:input=move |ev| set_comment_input.set(event_target_value(&ev))
                        prop:value=move || comment_input.get()
                    />
                    <button
                        class="bg-overlay-strong hover:bg-overlay-strong text-primary text-sm rounded px-3 py-1 transition-colors"
                        on:click=move |_| {
                            let text = comment_input.get_untracked();
                            let trimmed = text.trim().to_string();
                            if !trimmed.is_empty() {
                                set_comments.update(|c| c.push(trimmed));
                                set_comment_input.set(String::new());
                            }
                        }
                    >
                        "+ Add Comment"
                    </button>
                </div>
            </div>

            // Champion Picker
            <div class="bg-elevated border border-divider rounded-lg p-4">
                {move || active_slot_label().map(|label| view! {
                    <p class="text-accent-hover text-sm font-medium mb-2">{label}</p>
                })}
                <Suspense fallback=|| view! { <div class="text-muted">"Loading champions..."</div> }>
                    {move || champions_resource.get().map(|result| match result {
                        Err(e) => view! {
                            <ErrorBanner message=format!("Failed to load champions: {e}") />
                        }.into_any(),
                        Ok(champs) => view! {
                            <ChampionPicker
                                champions=champs
                                used_champions=used_champions()
                                on_select=on_champion_select
                            />
                        }.into_any(),
                    })}
                </Suspense>
            </div>

            // Action buttons
            <div class="flex gap-3 items-center">
                <button
                    class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-6 py-2 transition-colors"
                    on:click=do_save
                >
                    {move || if loaded_draft_id.get().is_some() { "Update Draft" } else { "Save Draft" }}
                </button>
                <button
                    class="bg-overlay-strong hover:bg-overlay-strong text-primary rounded px-4 py-2 transition-colors"
                    on:click=move |_| {
                        set_draft_slots.set(vec![None; 20]);
                        set_active_slot.set(Some(0));
                        set_highlighted_slot.set(None);
                        set_comments.set(Vec::new());
                        set_save_result.set(None);
                        set_loaded_draft_id.set(None);
                        set_draft_name.set(String::new());
                        set_opponent.set(String::new());
                        set_rating.set(None);
                        set_tags.set(Vec::new());
                        set_win_conditions.set(String::new());
                        set_watch_out.set(String::new());
                        set_slot_comments.set(vec![None; 20]);
                        set_slot_comment_input.set(String::new());
                        // Keep selected_team_id — the user probably wants the same team
                    }
                >
                    {move || if loaded_draft_id.get().is_some() { "New Draft" } else { "Clear" }}
                </button>
                {move || save_result.get().map(|msg| view! {
                    <div class="text-green-300 text-sm">{msg}</div>
                })}
            </div>

            // Saved Drafts
            <div>
                <h2 class="text-xl font-bold text-primary mb-3">"Saved Drafts"</h2>
                // Tag filter buttons
                <div class="flex flex-wrap gap-1.5 mb-3">
                    <button
                        class=move || if filter_tag.get().is_empty() {
                            "rounded px-2.5 py-0.5 text-xs font-medium bg-accent text-accent-contrast transition-colors cursor-pointer"
                        } else {
                            "rounded px-2.5 py-0.5 text-xs font-medium bg-overlay text-muted hover:bg-overlay-strong transition-colors cursor-pointer"
                        }
                        on:click=move |_| set_filter_tag.set(String::new())
                    >"All"</button>
                    {COMPOSITION_TAGS.iter().map(|&tag| {
                        let tag_str = tag.to_string();
                        let tag_for_class = tag_str.clone();
                        let tag_for_click = tag_str.clone();
                        view! {
                            <button
                                class=move || if filter_tag.get() == tag_for_class {
                                    "rounded px-2.5 py-0.5 text-xs font-medium bg-accent text-accent-contrast transition-colors cursor-pointer"
                                } else {
                                    "rounded px-2.5 py-0.5 text-xs font-medium bg-overlay text-muted hover:bg-overlay-strong transition-colors cursor-pointer"
                                }
                                on:click=move |_| set_filter_tag.set(tag_for_click.clone())
                            >{tag}</button>
                        }
                    }).collect_view()}
                </div>
                <Suspense fallback=|| view! { <div class="text-muted">"Loading..."</div> }>
                    {move || {
                        let champ_url_map: HashMap<String, String> = champions_resource.get()
                            .and_then(|r| r.ok())
                            .map(|champs| champs.into_iter().map(|c| (c.name, c.image_full)).collect())
                            .unwrap_or_default();
                        let champ_map_sv = StoredValue::new(champ_url_map);

                        drafts.get().map(move |result| match result {
                            Ok(list) if list.is_empty() => view! {
                                <p class="text-dimmed">"No drafts yet."</p>
                            }.into_any(),
                            Ok(list) => {
                                let ft = filter_tag.get();
                                let filtered: Vec<Draft> = if ft.is_empty() {
                                    list
                                } else {
                                    list.into_iter().filter(|d| d.tags.contains(&ft)).collect()
                                };
                                if filtered.is_empty() {
                                    return view! {
                                        <p class="text-dimmed">"No drafts match this filter."</p>
                                    }.into_any();
                                }
                                view! {
                                <div class="flex flex-col gap-2">
                                    {filtered.into_iter().map(|d| {
                                        let d_id = d.id.clone();
                                        let d_name = d.name.clone();
                                        let d_opp = d.opponent.clone().unwrap_or_default();
                                        let d_comments = d.comments.clone();
                                        let d_actions = d.actions.clone();
                                        let d_team_id = d.team_id.clone();
                                        let d_rating = d.rating.clone();
                                        let d_our_side = d.our_side.clone();
                                        let d_tags = d.tags.clone();
                                        let d_win_conditions = d.win_conditions.clone().unwrap_or_default();
                                        let d_watch_out = d.watch_out.clone().unwrap_or_default();

                                        let icon_url = |a: &DraftAction| champ_map_sv.with_value(|m| m.get(&a.champion).cloned().unwrap_or_default());

                                        // Blue bans: phase1 = orders 0,2,4  phase2 = orders 13,15
                                        let blue_ban_p1: Vec<(String, String)> = d.actions.iter()
                                            .filter(|a| a.side == "blue" && matches!(a.order, 0 | 2 | 4))
                                            .map(|a| (a.champion.clone(), icon_url(a)))
                                            .collect();
                                        let blue_ban_p2: Vec<(String, String)> = d.actions.iter()
                                            .filter(|a| a.side == "blue" && matches!(a.order, 13 | 15))
                                            .map(|a| (a.champion.clone(), icon_url(a)))
                                            .collect();
                                        // Red bans: phase1 = orders 1,3,5  phase2 = orders 12,14
                                        let red_ban_p1: Vec<(String, String)> = d.actions.iter()
                                            .filter(|a| a.side == "red" && matches!(a.order, 1 | 3 | 5))
                                            .map(|a| (a.champion.clone(), icon_url(a)))
                                            .collect();
                                        let red_ban_p2: Vec<(String, String)> = d.actions.iter()
                                            .filter(|a| a.side == "red" && matches!(a.order, 12 | 14))
                                            .map(|a| (a.champion.clone(), icon_url(a)))
                                            .collect();

                                        let blue_pick_icons: Vec<(String, String)> = d.actions.iter()
                                            .filter(|a| { let o = a.order as usize; !(o < 6 || (12..16).contains(&o)) && a.side == "blue" })
                                            .map(|a| (a.champion.clone(), icon_url(a)))
                                            .collect();
                                        let red_pick_icons: Vec<(String, String)> = d.actions.iter()
                                            .filter(|a| { let o = a.order as usize; !(o < 6 || (12..16).contains(&o)) && a.side == "red" })
                                            .map(|a| (a.champion.clone(), icon_url(a)))
                                            .collect();

                                        let has_picks = !blue_pick_icons.is_empty() || !red_pick_icons.is_empty();
                                        let has_both_sides = !blue_pick_icons.is_empty() && !red_pick_icons.is_empty();
                                        let has_left_bans = !blue_ban_p1.is_empty() || !blue_ban_p2.is_empty();
                                        let has_right_bans = !red_ban_p1.is_empty() || !red_ban_p2.is_empty();

                                        let display_name = d.name.clone();
                                        let display_opp = d.opponent.clone();
                                        let display_rating = d_rating.clone();
                                        let display_tags = d.tags.clone();

                                        view! {
                                            <div class="bg-elevated border border-divider rounded px-4 py-3 flex items-center gap-4">
                                                <div class="flex-1 min-w-0">
                                                    // Name + opponent + rating badge
                                                    <div class="flex items-center gap-2 mb-1.5">
                                                        <span class="text-primary font-medium">{display_name}</span>
                                                        {display_opp.map(|o| view! {
                                                            <span class="text-muted text-sm">"vs " {o}</span>
                                                        })}
                                                        {display_rating.map(|r| {
                                                            let cls = format!("rounded px-1.5 py-0.5 text-xs font-bold {}", tier_badge_class(&r));
                                                            view! { <span class=cls>{r}</span> }
                                                        })}
                                                        {display_tags.into_iter().map(|tag| {
                                                            view! { <span class="rounded px-1.5 py-0.5 text-xs font-medium bg-overlay-strong text-secondary">{tag}</span> }
                                                        }).collect_view()}
                                                    </div>
                                                    // Icon summary: [blue bans] | [picks] | [red bans]
                                                    <div class="flex items-center gap-0.5 flex-wrap">
                                                        // Blue bans (left)
                                                        {blue_ban_p1.into_iter().map(|(name, url)| view! {
                                                            <div class="relative w-6 h-6 flex-shrink-0" title=name.clone()>
                                                                <div class="w-6 h-6 rounded overflow-hidden border border-outline grayscale opacity-50">
                                                                    <img src=url alt=name.clone() class="w-full h-full object-cover" />
                                                                </div>
                                                                <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                                                                    <div class="w-4 h-px bg-red-500 rotate-45"></div>
                                                                </div>
                                                            </div>
                                                        }).collect_view()}
                                                        // Small gap between phase-1 and phase-2 blue bans
                                                        {if !blue_ban_p2.is_empty() {
                                                            view! { <span class="w-1.5 flex-shrink-0 inline-block"></span> }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        {blue_ban_p2.into_iter().map(|(name, url)| view! {
                                                            <div class="relative w-6 h-6 flex-shrink-0" title=name.clone()>
                                                                <div class="w-6 h-6 rounded overflow-hidden border border-outline grayscale opacity-50">
                                                                    <img src=url alt=name.clone() class="w-full h-full object-cover" />
                                                                </div>
                                                                <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                                                                    <div class="w-4 h-px bg-red-500 rotate-45"></div>
                                                                </div>
                                                            </div>
                                                        }).collect_view()}
                                                        // Separator between bans and picks
                                                        {if has_left_bans && has_picks {
                                                            view! { <span class="text-overlay-strong text-xs mx-0.5 flex-shrink-0">"|"</span> }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        // Blue picks
                                                        {blue_pick_icons.into_iter().map(|(name, url)| view! {
                                                            <div class="w-6 h-6 rounded overflow-hidden border border-blue-700 flex-shrink-0" title=name.clone()>
                                                                <img src=url alt=name.clone() class="w-full h-full object-cover" />
                                                            </div>
                                                        }).collect_view()}
                                                        // VS separator
                                                        {if has_both_sides {
                                                            view! { <span class="text-dimmed text-xs mx-0.5 flex-shrink-0">"vs"</span> }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        // Red picks
                                                        {red_pick_icons.into_iter().map(|(name, url)| view! {
                                                            <div class="w-6 h-6 rounded overflow-hidden border border-red-700 flex-shrink-0" title=name.clone()>
                                                                <img src=url alt=name.clone() class="w-full h-full object-cover" />
                                                            </div>
                                                        }).collect_view()}
                                                        // Separator between picks and red bans
                                                        {if has_right_bans && has_picks {
                                                            view! { <span class="text-overlay-strong text-xs mx-0.5 flex-shrink-0">"|"</span> }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        // Red bans (right)
                                                        {red_ban_p1.into_iter().map(|(name, url)| view! {
                                                            <div class="relative w-6 h-6 flex-shrink-0" title=name.clone()>
                                                                <div class="w-6 h-6 rounded overflow-hidden border border-outline grayscale opacity-50">
                                                                    <img src=url alt=name.clone() class="w-full h-full object-cover" />
                                                                </div>
                                                                <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                                                                    <div class="w-4 h-px bg-red-500 rotate-45"></div>
                                                                </div>
                                                            </div>
                                                        }).collect_view()}
                                                        // Small gap between phase-1 and phase-2 red bans
                                                        {if !red_ban_p2.is_empty() {
                                                            view! { <span class="w-1.5 flex-shrink-0 inline-block"></span> }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        {red_ban_p2.into_iter().map(|(name, url)| view! {
                                                            <div class="relative w-6 h-6 flex-shrink-0" title=name.clone()>
                                                                <div class="w-6 h-6 rounded overflow-hidden border border-outline grayscale opacity-50">
                                                                    <img src=url alt=name.clone() class="w-full h-full object-cover" />
                                                                </div>
                                                                <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                                                                    <div class="w-4 h-px bg-red-500 rotate-45"></div>
                                                                </div>
                                                            </div>
                                                        }).collect_view()}
                                                    </div>
                                                </div>
                                                <button
                                                    class="flex-shrink-0 bg-overlay hover:bg-accent hover:text-accent-contrast text-secondary text-sm font-medium rounded px-3 py-1.5 transition-colors"
                                                    on:click=move |_| {
                                                        set_loaded_draft_id.set(d_id.clone());
                                                        set_draft_name.set(d_name.clone());
                                                        set_opponent.set(d_opp.clone());
                                                        set_selected_team_id.set(d_team_id.clone());
                                                        set_rating.set(d_rating.clone());
                                                        set_our_side.set(d_our_side.clone());
                                                        set_comments.set(d_comments.clone());
                                                        set_tags.set(d_tags.clone());
                                                        set_win_conditions.set(d_win_conditions.clone());
                                                        set_watch_out.set(d_watch_out.clone());
                                                        set_save_result.set(None);
                                                        set_highlighted_slot.set(None);
                                                        let mut slots = vec![None::<String>; 20];
                                                        let mut sc = vec![None::<String>; 20];
                                                        for action in &d_actions {
                                                            let o = action.order as usize;
                                                            if o < 20 {
                                                                slots[o] = Some(action.champion.clone());
                                                                sc[o] = action.comment.clone();
                                                            }
                                                        }
                                                        let next = (0..20).find(|&i| slots[i].is_none());
                                                        set_draft_slots.set(slots);
                                                        set_slot_comments.set(sc);
                                                        set_slot_comment_input.set(String::new());
                                                        set_active_slot.set(next);
                                                    }
                                                >
                                                    "Open"
                                                </button>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                                }.into_any()
                            },
                            Err(e) => view! {
                                <ErrorBanner message=format!("Failed to load drafts: {e}") />
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </div>

            // Ban Priority Panel
            <div class="bg-elevated border border-divider rounded-lg">
                <button
                    class="w-full flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-surface/50 transition-colors"
                    on:click=move |_| set_ban_panel_open.update(|v| *v = !*v)
                >
                    <h2 class="text-xl font-bold text-primary">"Ban Priorities"</h2>
                    <span class="text-muted">{move || if ban_panel_open.get() { "\u{25B2}" } else { "\u{25BC}" }}</span>
                </button>
                {move || ban_panel_open.get().then(|| {
                    let is_editing = editing_bans.get();
                    view! {
                        <div class="px-4 pb-4 flex flex-col gap-3">
                            {if is_editing {
                                view! {
                                    <div class="flex flex-col gap-2">
                                        // Existing items
                                        {move || {
                                            let items = ban_edit_list.get();
                                            if items.is_empty() {
                                                view! { <p class="text-dimmed text-sm">"No ban priorities yet."</p> }.into_any()
                                            } else {
                                                view! {
                                                    <div class="flex flex-col gap-1">
                                                        {items.into_iter().enumerate().map(|(i, bp)| {
                                                            let champ = bp.champion.clone();
                                                            let reason = bp.reason.clone().unwrap_or_default();
                                                            view! {
                                                                <div class="flex items-center gap-2 bg-surface rounded px-3 py-2">
                                                                    <span class="text-accent font-bold text-sm w-6">{format!("#{}", i + 1)}</span>
                                                                    <span class="text-primary font-medium flex-1">{champ}</span>
                                                                    <span class="text-muted text-sm flex-1 truncate">{reason}</span>
                                                                    <button
                                                                        class="text-red-400 hover:text-red-300 text-sm cursor-pointer"
                                                                        on:click=move |_| {
                                                                            set_ban_edit_list.update(|list| {
                                                                                if i < list.len() { list.remove(i); }
                                                                                // Re-rank
                                                                                for (j, item) in list.iter_mut().enumerate() {
                                                                                    item.rank = j as i32;
                                                                                }
                                                                            });
                                                                        }
                                                                    >"Remove"</button>
                                                                </div>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                }.into_any()
                                            }
                                        }}
                                        // Add new entry
                                        <div class="flex gap-2 items-end">
                                            <div class="flex-1">
                                                <label class="block text-secondary text-xs mb-1">"Champion"</label>
                                                <input
                                                    type="text"
                                                    placeholder="Champion name..."
                                                    class="w-full bg-overlay border border-outline rounded px-2 py-1 text-primary text-sm focus:outline-none focus:border-accent"
                                                    prop:value=move || ban_new_champ.get()
                                                    on:input=move |ev| set_ban_new_champ.set(event_target_value(&ev))
                                                />
                                            </div>
                                            <div class="flex-1">
                                                <label class="block text-secondary text-xs mb-1">"Reason (optional)"</label>
                                                <input
                                                    type="text"
                                                    placeholder="Why ban?"
                                                    class="w-full bg-overlay border border-outline rounded px-2 py-1 text-primary text-sm focus:outline-none focus:border-accent"
                                                    prop:value=move || ban_new_reason.get()
                                                    on:input=move |ev| set_ban_new_reason.set(event_target_value(&ev))
                                                />
                                            </div>
                                            <button
                                                class="bg-overlay-strong hover:bg-overlay text-primary text-sm rounded px-3 py-1 transition-colors cursor-pointer"
                                                on:click=move |_| {
                                                    let champ = ban_new_champ.get_untracked();
                                                    if champ.trim().is_empty() { return; }
                                                    let reason_val = ban_new_reason.get_untracked();
                                                    let reason = if reason_val.trim().is_empty() { None } else { Some(reason_val) };
                                                    set_ban_edit_list.update(|list| {
                                                        let rank = list.len() as i32;
                                                        list.push(BanPriority {
                                                            id: None,
                                                            team_id: String::new(),
                                                            champion: champ.trim().to_string(),
                                                            rank,
                                                            reason,
                                                        });
                                                    });
                                                    set_ban_new_champ.set(String::new());
                                                    set_ban_new_reason.set(String::new());
                                                }
                                            >"+ Add"</button>
                                        </div>
                                        // Save / Cancel
                                        <div class="flex gap-2 mt-1">
                                            <button
                                                class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-4 py-1.5 text-sm transition-colors cursor-pointer"
                                                on:click=move |_| {
                                                    let list = ban_edit_list.get_untracked();
                                                    let json = serde_json::to_string(&list).unwrap_or_default();
                                                    leptos::task::spawn_local(async move {
                                                        match save_ban_priorities(json).await {
                                                            Ok(_) => {
                                                                set_ban_status.set(Some("Saved!".into()));
                                                                ban_priorities.refetch();
                                                                set_editing_bans.set(false);
                                                            }
                                                            Err(e) => set_ban_status.set(Some(format!("Error: {e}"))),
                                                        }
                                                    });
                                                }
                                            >"Save"</button>
                                            <button
                                                class="bg-overlay hover:bg-overlay-strong text-secondary rounded px-4 py-1.5 text-sm transition-colors cursor-pointer"
                                                on:click=move |_| set_editing_bans.set(false)
                                            >"Cancel"</button>
                                            {move || ban_status.get().map(|msg| view! {
                                                <span class="text-green-300 text-sm self-center">{msg}</span>
                                            })}
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                // View mode
                                view! {
                                    <div class="flex flex-col gap-2">
                                        <Suspense fallback=|| view! { <div class="text-muted text-sm">"Loading..."</div> }>
                                            {move || ban_priorities.get().map(|result| match result {
                                                Ok(list) if list.is_empty() => view! {
                                                    <p class="text-dimmed text-sm">"No ban priorities set."</p>
                                                }.into_any(),
                                                Ok(list) => view! {
                                                    <div class="flex flex-col gap-1">
                                                        {list.iter().map(|bp| {
                                                            let champ = bp.champion.clone();
                                                            let reason = bp.reason.clone().unwrap_or_default();
                                                            let rank = bp.rank + 1;
                                                            view! {
                                                                <div class="flex items-center gap-2 bg-surface rounded px-3 py-2">
                                                                    <span class="text-accent font-bold text-sm w-6">{format!("#{rank}")}</span>
                                                                    <span class="text-primary font-medium flex-1">{champ}</span>
                                                                    <span class="text-muted text-sm flex-1 truncate">{reason}</span>
                                                                </div>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                }.into_any(),
                                                Err(e) => view! {
                                                    <ErrorBanner message=format!("Failed to load ban priorities: {e}") />
                                                }.into_any(),
                                            })}
                                        </Suspense>
                                        <button
                                            class="bg-overlay-strong hover:bg-overlay text-primary text-sm rounded px-3 py-1.5 transition-colors self-start cursor-pointer"
                                            on:click=move |_| {
                                                // Copy current priorities into edit list
                                                if let Some(Ok(list)) = ban_priorities.get_untracked() {
                                                    set_ban_edit_list.set(list);
                                                } else {
                                                    set_ban_edit_list.set(Vec::new());
                                                }
                                                set_ban_status.set(None);
                                                set_editing_bans.set(true);
                                            }
                                        >"Edit"</button>
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
