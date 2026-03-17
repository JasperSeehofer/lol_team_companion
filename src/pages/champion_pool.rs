use crate::components::champion_autocomplete::ChampionAutocomplete;
use crate::components::ui::{EmptyState, SkeletonCard, ToastContext, ToastKind};
use crate::models::champion::{
    note_type_label, Champion, ChampionNote, ChampionPoolEntry, ChampionStatSummary,
};
use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn get_pool() -> Result<Vec<ChampionPoolEntry>, ServerFnError> {
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
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;
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
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;
    db::remove_from_champion_pool(&db, &user.id, champion, role)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn set_champion_tier(
    champion: String,
    role: String,
    tier: String,
) -> Result<(), ServerFnError> {
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
    db::update_champion_tier(&db, &user.id, &champion, &role, tier)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn set_champion_notes(
    champion: String,
    role: String,
    notes: String,
) -> Result<(), ServerFnError> {
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
    let notes_opt = if notes.is_empty() { None } else { Some(notes) };
    db::update_champion_notes(&db, &user.id, &champion, &role, notes_opt)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn set_champion_comfort(
    champion: String,
    role: String,
    level: Option<u8>,
) -> Result<(), ServerFnError> {
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
    db::update_champion_comfort(&db, &user.id, &champion, &role, level.map(|v| v as i64))
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn set_champion_meta_tag(
    champion: String,
    role: String,
    tag: Option<String>,
) -> Result<(), ServerFnError> {
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
    db::update_champion_meta_tag(&db, &user.id, &champion, &role, tag)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_notes(
    champion: String,
    role: String,
) -> Result<Vec<ChampionNote>, ServerFnError> {
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
    db::get_champion_notes(&db, &user.id, &champion, &role)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn save_note(note_json: String) -> Result<String, ServerFnError> {
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

    let note: ChampionNote = serde_json::from_str(&note_json)
        .map_err(|e| ServerFnError::new(format!("Invalid note JSON: {e}")))?;

    if note.id.is_some() {
        db::update_champion_note(&db, note)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(String::new())
    } else {
        db::add_champion_note(&db, &user.id, note)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
}

#[server]
pub async fn delete_note(note_id: String) -> Result<(), ServerFnError> {
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
    db::delete_champion_note(&db, &note_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ---------------------------------------------------------------------------
// Constants & helpers
// ---------------------------------------------------------------------------

const POOL_ROLES: &[&str] = &["Top", "Jungle", "Mid", "ADC", "Support"];
const TIERS: &[&str] = &[
    "comfort",
    "match_ready",
    "scrim_ready",
    "practicing",
    "to_practice",
];

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

fn meta_tag_class(tag: &str) -> &'static str {
    match tag {
        "strong" => "bg-green-500/20 text-green-400 border-green-500/30",
        "neutral" => "bg-blue-500/20 text-blue-400 border-blue-500/30",
        "weak" => "bg-red-500/20 text-red-400 border-red-500/30",
        _ => "bg-overlay text-muted border-divider",
    }
}

const META_TAGS: &[&str] = &["strong", "neutral", "weak"];
const NOTE_TYPE_LIST: &[&str] = &[
    "matchup",
    "power_spike",
    "combo",
    "lesson",
    "synergy",
    "positioning",
];

#[server]
pub async fn get_my_champion_stats() -> Result<Vec<ChampionStatSummary>, ServerFnError> {
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

    db::get_champion_stats_for_user(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn ChampionPoolPage() -> impl IntoView {
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

    let pool = Resource::new(|| (), |_| get_pool());
    let champions_resource = Resource::new(|| (), |_| get_pool_champions());
    let stats_resource = Resource::new(|| (), |_| get_my_champion_stats());

    let toast = use_context::<ToastContext>().expect("ToastProvider");

    let (active_role, set_active_role) = signal("Top");
    // Selected champion: (champion_name, role)
    let (selected_entry, set_selected_entry) = signal(Option::<(String, String)>::None);
    // Detail panel tab: "overview", "matchups", "notes", "journal"
    let (detail_tab, set_detail_tab) = signal("overview");

    // Notes resource keyed on selected champion
    let notes_resource = Resource::new(
        move || selected_entry.get(),
        move |sel| async move {
            match sel {
                Some((champ, role)) => get_notes(champ, role).await.unwrap_or_default(),
                None => Vec::new(),
            }
        },
    );

    // Add champion form
    let add_input_signal = RwSignal::new(String::new());

    let do_add = move || {
        let champion = add_input_signal.get_untracked();
        let role = active_role.get_untracked().to_string();
        if champion.trim().is_empty() {
            return;
        }
        leptos::task::spawn_local(async move {
            match add_to_pool(champion, role).await {
                Ok(_) => {
                    add_input_signal.set(String::new());
                    pool.refetch();
                    toast.show.run((ToastKind::Success, "Champion added to pool".into()));
                }
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        });
    };

    let on_autocomplete_select = Callback::new(move |_name: String| {
        do_add();
    });

    // Note form state
    let (note_form_open, set_note_form_open) = signal(false);
    let (note_form_type, set_note_form_type) = signal("matchup".to_string());
    let (note_form_title, set_note_form_title) = signal(String::new());
    let (note_form_content, set_note_form_content) = signal(String::new());
    let (note_form_difficulty, set_note_form_difficulty) = signal(Option::<u8>::None);
    let (note_form_editing_id, set_note_form_editing_id) = signal(Option::<String>::None);

    let clear_note_form = move || {
        set_note_form_open.set(false);
        set_note_form_type.set("matchup".to_string());
        set_note_form_title.set(String::new());
        set_note_form_content.set(String::new());
        set_note_form_difficulty.set(None);
        set_note_form_editing_id.set(None);
    };

    let do_save_note = Callback::new(move |_: ()| {
        let sel = selected_entry.get_untracked();
        let Some((champ, role)) = sel else { return };
        let note = ChampionNote {
            id: note_form_editing_id.get_untracked(),
            user_id: String::new(), // filled server-side
            champion: champ,
            role,
            note_type: note_form_type.get_untracked(),
            title: note_form_title.get_untracked(),
            content: note_form_content.get_untracked(),
            difficulty: note_form_difficulty.get_untracked(),
            created_at: None,
        };
        let json = serde_json::to_string(&note).unwrap_or_default();
        leptos::task::spawn_local(async move {
            match save_note(json).await {
                Ok(_) => {
                    notes_resource.refetch();
                    toast.show.run((ToastKind::Success, "Note saved".into()));
                }
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        });
        clear_note_form();
    });

    let do_delete_note = Callback::new(move |note_id: String| {
        leptos::task::spawn_local(async move {
            match delete_note(note_id).await {
                Ok(_) => {
                    notes_resource.refetch();
                    toast.show.run((ToastKind::Success, "Note deleted".into()));
                }
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        });
    });

    let edit_note = Callback::new(move |note: ChampionNote| {
        set_note_form_editing_id.set(note.id);
        set_note_form_type.set(note.note_type);
        set_note_form_title.set(note.title);
        set_note_form_content.set(note.content);
        set_note_form_difficulty.set(note.difficulty);
        set_note_form_open.set(true);
    });

    // Notes input for the legacy overview notes field
    let (notes_input, set_notes_input) = signal(String::new());

    view! {
        <div class="max-w-7xl mx-auto py-8 px-4 sm:px-6 flex flex-col gap-6">
            <div>
                <h1 class="text-3xl font-bold text-primary">"Champion Pool"</h1>
                <p class="text-muted text-sm mt-1">"Organize your champion pool by role and readiness tier"</p>
            </div>

            // Role tabs — scrollable on mobile
            <div class="flex gap-1 overflow-x-auto pb-1">
                {POOL_ROLES.iter().map(|&role| {
                    view! {
                        <button
                            class=move || if active_role.get() == role {
                                "px-4 py-2 rounded-lg text-sm font-medium bg-accent text-accent-contrast transition-colors cursor-pointer whitespace-nowrap"
                            } else {
                                "px-4 py-2 rounded-lg text-sm font-medium bg-elevated text-secondary hover:bg-overlay transition-colors cursor-pointer whitespace-nowrap"
                            }
                            on:click=move |_| {
                                set_active_role.set(role);
                                set_selected_entry.set(None);
                                set_detail_tab.set("overview");
                            }
                        >
                            {role}
                        </button>
                    }
                }).collect_view()}
            </div>

            <div class="flex flex-col lg:flex-row gap-6 min-h-[30rem]">
                // Main content: tiers
                <div class="flex-1 flex flex-col gap-4 min-w-0">
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
                                            on_select=on_autocomplete_select
                                        />
                                    }
                                })}
                            </Suspense>
                        </div>
                        <button
                            class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm transition-colors cursor-pointer"
                            on:click=move |_| do_add()
                        >"Add"</button>
                    </div>

                    // Tier columns
                    <Suspense fallback=|| view! { <div class="flex flex-col gap-2"><SkeletonCard height="h-20" /><SkeletonCard height="h-20" /></div> }>
                        {move || {
                            let role = active_role.get();
                            pool.get().map(move |result| {
                                let entries = result.unwrap_or_default();
                                let role_entries: Vec<ChampionPoolEntry> = entries.into_iter()
                                    .filter(|e| e.role == role)
                                    .collect();

                                if role_entries.is_empty() {
                                    return view! {
                                        <EmptyState
                                            icon="🎯"
                                            message="Your champion pool is empty — add champions to track your picks and matchups"
                                            cta_label="Add a Champion"
                                            cta_href="#add-champion"
                                        />
                                    }.into_any();
                                }

                                view! {
                                    <div class="flex flex-col gap-3">
                                        {TIERS.iter().map(|&tier| {
                                            let tier_entries: Vec<&ChampionPoolEntry> = role_entries.iter()
                                                .filter(|e| e.tier == tier)
                                                .collect();
                                            view! {
                                                <div class=format!("border rounded-xl p-4 {}", tier_color(tier))>
                                                    <h3 class=format!("text-xs font-semibold uppercase tracking-wider mb-3 {}", tier_label_color(tier))>
                                                        {tier_label(tier)}
                                                        {(!tier_entries.is_empty()).then(|| view! {
                                                            <span class="text-dimmed ml-1">{format!("({})", tier_entries.len())}</span>
                                                        })}
                                                    </h3>
                                                    {if tier_entries.is_empty() {
                                                        view! {
                                                            <p class="text-dimmed text-xs italic">"No champions yet"</p>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <div class="flex flex-wrap gap-2">
                                                                {tier_entries.into_iter().map(|entry| {
                                                                    let champ = entry.champion.clone();
                                                                    let role_val = entry.role.clone();
                                                                    let champ_for_class = champ.clone();
                                                                    let role_for_class = role_val.clone();
                                                                    let champ_for_select = champ.clone();
                                                                    let role_for_select = role_val.clone();
                                                                    let champ_for_remove = champ.clone();
                                                                    let role_for_remove = role_val.clone();
                                                                    let comfort = entry.comfort_level;
                                                                    let meta = entry.meta_tag.clone();

                                                                    let (img_url, display_name) = champions_resource.get()
                                                                        .and_then(|r| r.ok())
                                                                        .and_then(|champs| champs.into_iter().find(|c| c.id == champ))
                                                                        .map(|c| (c.image_full, c.name))
                                                                        .unwrap_or_else(|| (String::new(), champ.clone()));

                                                                    view! {
                                                                        <div
                                                                            class=move || {
                                                                                let is_selected = selected_entry.get()
                                                                                    .map(|(c, r)| c == champ_for_class && r == role_for_class)
                                                                                    .unwrap_or(false);
                                                                                if is_selected {
                                                                                    "flex items-center gap-1.5 bg-accent/10 border border-accent/50 rounded-lg px-2.5 py-1.5 transition-colors cursor-pointer group"
                                                                                } else {
                                                                                    "flex items-center gap-1.5 bg-elevated border border-divider rounded-lg px-2.5 py-1.5 hover:border-accent/50 transition-colors cursor-pointer group"
                                                                                }
                                                                            }
                                                                            on:click=move |_| {
                                                                                set_selected_entry.set(Some((champ_for_select.clone(), role_for_select.clone())));
                                                                                set_detail_tab.set("overview");
                                                                            }
                                                                        >
                                                                            {if !img_url.is_empty() {
                                                                                view! {
                                                                                    <img src=img_url alt=display_name.clone() class="w-7 h-7 rounded object-cover" />
                                                                                }.into_any()
                                                                            } else {
                                                                                view! { <span></span> }.into_any()
                                                                            }}
                                                                            <div class="flex flex-col min-w-0">
                                                                                <span class="text-primary text-sm truncate">{display_name.clone()}</span>
                                                                                <div class="flex items-center gap-1">
                                                                                    // Comfort stars (compact)
                                                                                    {comfort.map(|lvl| {
                                                                                        let stars: String = (0..5).map(|i| if i < lvl { '\u{2605}' } else { '\u{2606}' }).collect();
                                                                                        view! { <span class="text-accent text-[10px] leading-none">{stars}</span> }
                                                                                    })}
                                                                                    // Meta tag badge
                                                                                    {meta.as_ref().map(|t| {
                                                                                        let cls = meta_tag_class(t);
                                                                                        let label = match t.as_str() {
                                                                                            "strong" => "S",
                                                                                            "neutral" => "~",
                                                                                            "weak" => "W",
                                                                                            _ => "?",
                                                                                        };
                                                                                        view! {
                                                                                            <span class=format!("text-[9px] px-1 rounded border font-bold {cls}")>{label}</span>
                                                                                        }
                                                                                    })}
                                                                                    // Match stats badge
                                                                                    {move || {
                                                                                        let champ_name = champ.clone();
                                                                                        stats_resource.get().and_then(|r| r.ok()).and_then(|stats| {
                                                                                            stats.into_iter().find(|s| s.champion == champ_name)
                                                                                        }).map(|s| {
                                                                                            let wr = if s.games > 0 { (s.wins as f64 / s.games as f64 * 100.0).round() as i32 } else { 0 };
                                                                                            view! {
                                                                                                <span class="text-[9px] px-1 rounded bg-overlay text-muted whitespace-nowrap" title="Games / Win% / KDA from match history">
                                                                                                    {format!("{}G {}%W {:.1}", s.games, wr, s.avg_kda)}
                                                                                                </span>
                                                                                            }
                                                                                        })
                                                                                    }}
                                                                                </div>
                                                                            </div>
                                                                            <button
                                                                                class="text-overlay-strong hover:text-red-400 ml-auto transition-colors opacity-0 group-hover:opacity-100 cursor-pointer flex-shrink-0"
                                                                                title="Remove"
                                                                                on:click=move |ev| {
                                                                                    ev.stop_propagation();
                                                                                    let c = champ_for_remove.clone();
                                                                                    let r = role_for_remove.clone();
                                                                                    leptos::task::spawn_local(async move {
                                                                                        match remove_from_pool(c, r).await {
                                                                                            Ok(_) => pool.refetch(),
                                                                                            Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                                                                                        }
                                                                                    });
                                                                                }
                                                                            >"\u{00d7}"</button>
                                                                        </div>
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                        }.into_any()
                                                    }}
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            })
                        }}
                    </Suspense>
                </div>

                // Right panel: champion details (tabbed)
                <div class="w-full lg:w-96 flex-shrink-0">
                    {move || {
                        let sel = selected_entry.get();
                        match sel {
                            None => view! {
                                <div class="bg-elevated/30 border border-divider/30 rounded-xl p-6 flex items-center justify-center min-h-[200px]">
                                    <p class="text-dimmed text-sm text-center">"Click a champion to view details, manage notes, and track matchups"</p>
                                </div>
                            }.into_any(),
                            Some((ref champ, ref role)) => {
                                // Look up current entry data
                                let current_entry = pool.get()
                                    .and_then(|r| r.ok())
                                    .and_then(|entries| entries.into_iter().find(|e| e.champion == *champ && e.role == *role));

                                let current_tier = current_entry.as_ref().map(|e| e.tier.clone()).unwrap_or("comfort".to_string());
                                let current_notes_text = current_entry.as_ref().and_then(|e| e.notes.clone()).unwrap_or_default();
                                let current_comfort = current_entry.as_ref().and_then(|e| e.comfort_level);
                                let current_meta = current_entry.as_ref().and_then(|e| e.meta_tag.clone());

                                let (img_url, champ_display_name) = champions_resource.get()
                                    .and_then(|r| r.ok())
                                    .and_then(|champs| champs.into_iter().find(|c| c.id == *champ))
                                    .map(|c| (c.image_full, c.name))
                                    .unwrap_or_else(|| (String::new(), champ.clone()));

                                set_notes_input.set(current_notes_text.clone());

                                // Clones for closures
                                let champ_for_tier = champ.clone();
                                let role_for_tier = role.clone();
                                let champ_for_notes = champ.clone();
                                let role_for_notes = role.clone();
                                let champ_for_comfort = champ.clone();
                                let role_for_comfort = role.clone();
                                let champ_for_meta = champ.clone();
                                let role_for_meta = role.clone();

                                view! {
                                    <div class="bg-elevated/50 border border-divider/50 rounded-xl flex flex-col">
                                        // Header
                                        <div class="flex items-center gap-3 p-4 border-b border-divider/30">
                                            {if !img_url.is_empty() {
                                                view! { <img src=img_url alt=champ_display_name.clone() class="w-12 h-12 rounded-lg object-cover" /> }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }}
                                            <div class="min-w-0 flex-1">
                                                <h3 class="text-primary font-semibold truncate">{champ_display_name.clone()}</h3>
                                                <div class="flex items-center gap-2">
                                                    <p class="text-muted text-xs capitalize">{role.clone()}</p>
                                                    {current_meta.as_ref().map(|t| {
                                                        let cls = meta_tag_class(t);
                                                        let label = match t.as_str() {
                                                            "strong" => "Meta Strong",
                                                            "neutral" => "Neutral",
                                                            "weak" => "Off-Meta",
                                                            _ => "Unknown",
                                                        };
                                                        view! {
                                                            <span class=format!("text-[10px] px-1.5 py-0.5 rounded border font-medium {cls}")>{label}</span>
                                                        }
                                                    })}
                                                </div>
                                            </div>
                                        </div>

                                        // Tab bar
                                        <div class="flex border-b border-divider/30 overflow-x-auto">
                                            {["overview", "matchups", "notes", "journal"].into_iter().map(|tab| {
                                                let label = match tab {
                                                    "overview" => "Overview",
                                                    "matchups" => "Matchups",
                                                    "notes" => "Notes",
                                                    "journal" => "Journal",
                                                    _ => tab,
                                                };
                                                view! {
                                                    <button
                                                        class=move || if detail_tab.get() == tab {
                                                            "px-3 py-2 text-xs font-medium text-accent border-b-2 border-accent whitespace-nowrap cursor-pointer"
                                                        } else {
                                                            "px-3 py-2 text-xs font-medium text-muted hover:text-secondary border-b-2 border-transparent whitespace-nowrap cursor-pointer"
                                                        }
                                                        on:click=move |_| set_detail_tab.set(tab)
                                                    >
                                                        {label}
                                                    </button>
                                                }
                                            }).collect_view()}
                                        </div>

                                        // Tab content
                                        <div class="p-4 flex flex-col gap-4 overflow-y-auto max-h-[60vh]">
                                            {move || {
                                                let tab = detail_tab.get();
                                                match tab {
                                                    // ===== OVERVIEW TAB =====
                                                    "overview" => {
                                                        let champ_t = champ_for_tier.clone();
                                                        let role_t = role_for_tier.clone();
                                                        let champ_c = champ_for_comfort.clone();
                                                        let role_c = role_for_comfort.clone();
                                                        let champ_m = champ_for_meta.clone();
                                                        let role_m = role_for_meta.clone();
                                                        let champ_n = champ_for_notes.clone();
                                                        let role_n = role_for_notes.clone();
                                                        view! {
                                                            <div class="flex flex-col gap-4">
                                                                // Tier selector
                                                                <div>
                                                                    <label class="block text-muted text-xs font-medium mb-2">"Tier"</label>
                                                                    <div class="flex flex-col gap-1">
                                                                        {TIERS.iter().map(|&tier| {
                                                                            let is_current = current_tier == tier;
                                                                            let ct = champ_t.clone();
                                                                            let rt = role_t.clone();
                                                                            view! {
                                                                                <button
                                                                                    class=move || if is_current {
                                                                                        format!("w-full text-left px-3 py-1.5 rounded text-sm font-medium {} cursor-pointer", tier_label_color(tier))
                                                                                    } else {
                                                                                        "w-full text-left px-3 py-1.5 rounded text-sm text-muted hover:bg-overlay/50 transition-colors cursor-pointer".to_string()
                                                                                    }
                                                                                    on:click=move |_| {
                                                                                        let c = ct.clone();
                                                                                        let r = rt.clone();
                                                                                        let t = tier.to_string();
                                                                                        leptos::task::spawn_local(async move {
                                                                                            match set_champion_tier(c, r, t).await {
                                                                                                Ok(_) => pool.refetch(),
                                                                                                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                >
                                                                                    {if is_current { "\u{25cf} " } else { "\u{25cb} " }}
                                                                                    {tier_label(tier)}
                                                                                </button>
                                                                            }
                                                                        }).collect_view()}
                                                                    </div>
                                                                </div>

                                                                // Comfort level (1-5 stars)
                                                                <div>
                                                                    <label class="block text-muted text-xs font-medium mb-2">"Comfort Level"</label>
                                                                    <div class="flex gap-1">
                                                                        {(1u8..=5).map(|level| {
                                                                            let is_filled = current_comfort.unwrap_or(0) >= level;
                                                                            let cc = champ_c.clone();
                                                                            let rc = role_c.clone();
                                                                            view! {
                                                                                <button
                                                                                    class="text-xl cursor-pointer transition-colors hover:scale-110"
                                                                                    style=move || if is_filled { "color: var(--color-accent)" } else { "color: var(--color-muted)" }
                                                                                    on:click=move |_| {
                                                                                        let c = cc.clone();
                                                                                        let r = rc.clone();
                                                                                        let new_level = if current_comfort == Some(level) { None } else { Some(level) };
                                                                                        leptos::task::spawn_local(async move {
                                                                                            match set_champion_comfort(c, r, new_level).await {
                                                                                                Ok(_) => pool.refetch(),
                                                                                                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                >
                                                                                    {if is_filled { "\u{2605}" } else { "\u{2606}" }}
                                                                                </button>
                                                                            }
                                                                        }).collect_view()}
                                                                    </div>
                                                                </div>

                                                                // Meta fitness tag
                                                                <div>
                                                                    <label class="block text-muted text-xs font-medium mb-2">"Meta Fitness"</label>
                                                                    <div class="flex gap-2">
                                                                        {META_TAGS.iter().map(|&tag| {
                                                                            let is_active = current_meta.as_deref() == Some(tag);
                                                                            let cls = meta_tag_class(tag);
                                                                            let label = match tag {
                                                                                "strong" => "Strong",
                                                                                "neutral" => "Neutral",
                                                                                "weak" => "Weak",
                                                                                _ => tag,
                                                                            };
                                                                            let cm = champ_m.clone();
                                                                            let rm = role_m.clone();
                                                                            view! {
                                                                                <button
                                                                                    class=move || {
                                                                                        if is_active {
                                                                                            format!("px-3 py-1 rounded-lg text-xs font-medium border cursor-pointer {cls}")
                                                                                        } else {
                                                                                            "px-3 py-1 rounded-lg text-xs font-medium border border-divider text-muted hover:bg-overlay cursor-pointer".to_string()
                                                                                        }
                                                                                    }
                                                                                    on:click=move |_| {
                                                                                        let c = cm.clone();
                                                                                        let r = rm.clone();
                                                                                        let new_tag = if is_active { None } else { Some(tag.to_string()) };
                                                                                        leptos::task::spawn_local(async move {
                                                                                            match set_champion_meta_tag(c, r, new_tag).await {
                                                                                                Ok(_) => pool.refetch(),
                                                                                                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                >
                                                                                    {label}
                                                                                </button>
                                                                            }
                                                                        }).collect_view()}
                                                                    </div>
                                                                </div>

                                                                // General notes
                                                                <div>
                                                                    <label class="block text-muted text-xs font-medium mb-1">"General Notes"</label>
                                                                    <textarea
                                                                        rows="3"
                                                                        class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus:outline-none focus:border-accent/50 resize-none transition-colors"
                                                                        placeholder="General notes about this champion..."
                                                                        prop:value=move || notes_input.get()
                                                                        on:input=move |ev| set_notes_input.set(event_target_value(&ev))
                                                                    />
                                                                    <button
                                                                        class="mt-2 bg-overlay hover:bg-overlay-strong text-secondary text-xs font-medium rounded px-3 py-1.5 transition-colors cursor-pointer"
                                                                        on:click=move |_| {
                                                                            let c = champ_n.clone();
                                                                            let r = role_n.clone();
                                                                            let n = notes_input.get_untracked();
                                                                            leptos::task::spawn_local(async move {
                                                                                match set_champion_notes(c, r, n).await {
                                                                                    Ok(_) => {
                                                                                        toast.show.run((ToastKind::Success, "Notes saved".into()));
                                                                                        pool.refetch();
                                                                                    }
                                                                                    Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                                                                                }
                                                                            });
                                                                        }
                                                                    >"Save Notes"</button>
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    },

                                                    // ===== MATCHUPS TAB =====
                                                    "matchups" => {
                                                        view! {
                                                            <div class="flex flex-col gap-3">
                                                                <div class="flex items-center justify-between">
                                                                    <label class="text-muted text-xs font-medium">"Matchup Notes"</label>
                                                                    <button
                                                                        class="text-accent hover:text-accent-hover text-xs font-medium cursor-pointer"
                                                                        on:click=move |_| {
                                                                            set_note_form_type.set("matchup".to_string());
                                                                            set_note_form_open.set(true);
                                                                        }
                                                                    >"+ Add Matchup"</button>
                                                                </div>

                                                                <Suspense fallback=|| view! { <SkeletonCard height="h-20" /> }>
                                                                    {move || notes_resource.get().map(|notes| {
                                                                        let matchups: Vec<&ChampionNote> = notes.iter()
                                                                            .filter(|n| n.note_type == "matchup")
                                                                            .collect();
                                                                        if matchups.is_empty() {
                                                                            view! {
                                                                                <p class="text-dimmed text-xs italic py-2">"No matchup notes yet. Add one to track how you play against specific champions."</p>
                                                                            }.into_any()
                                                                        } else {
                                                                            view! {
                                                                                <div class="flex flex-col gap-2">
                                                                                    {matchups.into_iter().map(|note| {
                                                                                        let note_clone = note.clone();
                                                                                        let note_for_edit = note.clone();
                                                                                        let note_id = note.id.clone().unwrap_or_default();
                                                                                        let difficulty_stars: String = note.difficulty
                                                                                            .map(|d| (0..5).map(|i| if i < d { '\u{2605}' } else { '\u{2606}' }).collect())
                                                                                            .unwrap_or_default();
                                                                                        view! {
                                                                                            <div class="bg-surface/30 border border-outline/30 rounded-lg p-3">
                                                                                                <div class="flex items-start justify-between gap-2">
                                                                                                    <div class="min-w-0 flex-1">
                                                                                                        <div class="flex items-center gap-2">
                                                                                                            <span class="text-primary text-sm font-medium">{note_clone.title.clone()}</span>
                                                                                                            {(!difficulty_stars.is_empty()).then(|| view! {
                                                                                                                <span class="text-red-400 text-[10px]" title="Difficulty">{difficulty_stars}</span>
                                                                                                            })}
                                                                                                        </div>
                                                                                                        {(!note_clone.content.is_empty()).then(|| view! {
                                                                                                            <p class="text-secondary text-xs mt-1 whitespace-pre-wrap">{note_clone.content.clone()}</p>
                                                                                                        })}
                                                                                                    </div>
                                                                                                    <div class="flex gap-1 flex-shrink-0">
                                                                                                        <button
                                                                                                            class="text-muted hover:text-secondary text-xs cursor-pointer"
                                                                                                            on:click=move |_| edit_note.run(note_for_edit.clone())
                                                                                                        >"Edit"</button>
                                                                                                        <button
                                                                                                            class="text-muted hover:text-red-400 text-xs cursor-pointer"
                                                                                                            on:click=move |_| do_delete_note.run(note_id.clone())
                                                                                                        >"Del"</button>
                                                                                                    </div>
                                                                                                </div>
                                                                                            </div>
                                                                                        }
                                                                                    }).collect_view()}
                                                                                </div>
                                                                            }.into_any()
                                                                        }
                                                                    })}
                                                                </Suspense>
                                                            </div>
                                                        }.into_any()
                                                    },

                                                    // ===== NOTES TAB (power spikes, combos, synergies, positioning) =====
                                                    "notes" => {
                                                        view! {
                                                            <div class="flex flex-col gap-4">
                                                                {["power_spike", "combo", "synergy", "positioning"].into_iter().map(|ntype| {
                                                                    let ntype_string = ntype.to_string();
                                                                    let label = note_type_label(ntype);
                                                                    view! {
                                                                        <div>
                                                                            <div class="flex items-center justify-between mb-2">
                                                                                <label class="text-muted text-xs font-medium">{label}</label>
                                                                                <button
                                                                                    class="text-accent hover:text-accent-hover text-xs font-medium cursor-pointer"
                                                                                    on:click=move |_| {
                                                                                        set_note_form_type.set(ntype_string.clone());
                                                                                        set_note_form_open.set(true);
                                                                                    }
                                                                                >"+ Add"</button>
                                                                            </div>
                                                                            <Suspense fallback=|| view! { <span></span> }>
                                                                                {move || notes_resource.get().map(|notes| {
                                                                                    let typed_notes: Vec<&ChampionNote> = notes.iter()
                                                                                        .filter(|n| n.note_type == ntype)
                                                                                        .collect();
                                                                                    if typed_notes.is_empty() {
                                                                                        view! {
                                                                                            <p class="text-dimmed text-xs italic">"None yet"</p>
                                                                                        }.into_any()
                                                                                    } else {
                                                                                        view! {
                                                                                            <div class="flex flex-col gap-1.5">
                                                                                                {typed_notes.into_iter().map(|note| {
                                                                                                    let note_for_edit = note.clone();
                                                                                                    let note_id = note.id.clone().unwrap_or_default();
                                                                                                    view! {
                                                                                                        <div class="bg-surface/30 border border-outline/30 rounded-lg p-2.5 group">
                                                                                                            <div class="flex items-start justify-between gap-2">
                                                                                                                <div class="min-w-0 flex-1">
                                                                                                                    <span class="text-primary text-xs font-medium">{note.title.clone()}</span>
                                                                                                                    {(!note.content.is_empty()).then(|| view! {
                                                                                                                        <p class="text-secondary text-xs mt-0.5 whitespace-pre-wrap">{note.content.clone()}</p>
                                                                                                                    })}
                                                                                                                </div>
                                                                                                                <div class="flex gap-1 flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                                                                                                                    <button
                                                                                                                        class="text-muted hover:text-secondary text-xs cursor-pointer"
                                                                                                                        on:click=move |_| edit_note.run(note_for_edit.clone())
                                                                                                                    >"Edit"</button>
                                                                                                                    <button
                                                                                                                        class="text-muted hover:text-red-400 text-xs cursor-pointer"
                                                                                                                        on:click=move |_| do_delete_note.run(note_id.clone())
                                                                                                                    >"Del"</button>
                                                                                                                </div>
                                                                                                            </div>
                                                                                                        </div>
                                                                                                    }
                                                                                                }).collect_view()}
                                                                                            </div>
                                                                                        }.into_any()
                                                                                    }
                                                                                })}
                                                                            </Suspense>
                                                                        </div>
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                        }.into_any()
                                                    },

                                                    // ===== JOURNAL TAB (lessons learned) =====
                                                    "journal" => {
                                                        view! {
                                                            <div class="flex flex-col gap-3">
                                                                <div class="flex items-center justify-between">
                                                                    <label class="text-muted text-xs font-medium">"Lessons Learned"</label>
                                                                    <button
                                                                        class="text-accent hover:text-accent-hover text-xs font-medium cursor-pointer"
                                                                        on:click=move |_| {
                                                                            set_note_form_type.set("lesson".to_string());
                                                                            set_note_form_open.set(true);
                                                                        }
                                                                    >"+ Add Entry"</button>
                                                                </div>

                                                                <Suspense fallback=|| view! { <SkeletonCard height="h-20" /> }>
                                                                    {move || notes_resource.get().map(|notes| {
                                                                        let lessons: Vec<&ChampionNote> = notes.iter()
                                                                            .filter(|n| n.note_type == "lesson")
                                                                            .collect();
                                                                        if lessons.is_empty() {
                                                                            view! {
                                                                                <p class="text-dimmed text-xs italic py-2">"No lessons recorded yet. Add entries after games to track what you learn."</p>
                                                                            }.into_any()
                                                                        } else {
                                                                            view! {
                                                                                <div class="flex flex-col gap-2">
                                                                                    {lessons.into_iter().map(|note| {
                                                                                        let note_for_edit = note.clone();
                                                                                        let note_id = note.id.clone().unwrap_or_default();
                                                                                        let date = note.created_at.as_deref().unwrap_or("").chars().take(10).collect::<String>();
                                                                                        view! {
                                                                                            <div class="bg-surface/30 border border-outline/30 rounded-lg p-3 group">
                                                                                                <div class="flex items-start justify-between gap-2">
                                                                                                    <div class="min-w-0 flex-1">
                                                                                                        <div class="flex items-center gap-2">
                                                                                                            <span class="text-primary text-sm font-medium">{note.title.clone()}</span>
                                                                                                            <span class="text-dimmed text-[10px]">{date}</span>
                                                                                                        </div>
                                                                                                        {(!note.content.is_empty()).then(|| view! {
                                                                                                            <p class="text-secondary text-xs mt-1 whitespace-pre-wrap">{note.content.clone()}</p>
                                                                                                        })}
                                                                                                    </div>
                                                                                                    <div class="flex gap-1 flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                                                                                                        <button
                                                                                                            class="text-muted hover:text-secondary text-xs cursor-pointer"
                                                                                                            on:click=move |_| edit_note.run(note_for_edit.clone())
                                                                                                        >"Edit"</button>
                                                                                                        <button
                                                                                                            class="text-muted hover:text-red-400 text-xs cursor-pointer"
                                                                                                            on:click=move |_| do_delete_note.run(note_id.clone())
                                                                                                        >"Del"</button>
                                                                                                    </div>
                                                                                                </div>
                                                                                            </div>
                                                                                        }
                                                                                    }).collect_view()}
                                                                                </div>
                                                                            }.into_any()
                                                                        }
                                                                    })}
                                                                </Suspense>
                                                            </div>
                                                        }.into_any()
                                                    },

                                                    _ => view! { <div></div> }.into_any(),
                                                }
                                            }}
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }
                    }}

                    // Note form modal (overlay at bottom of detail panel)
                    {move || note_form_open.get().then(|| {
                        let form_type = note_form_type.get();
                        let is_matchup = form_type == "matchup";
                        let type_label = note_type_label(&form_type);
                        let is_editing = note_form_editing_id.get().is_some();
                        view! {
                            <div class="mt-3 bg-elevated border border-accent/30 rounded-xl p-4 flex flex-col gap-3">
                                <div class="flex items-center justify-between">
                                    <h4 class="text-primary text-sm font-semibold">
                                        {if is_editing { "Edit " } else { "New " }}
                                        {type_label}
                                    </h4>
                                    <button
                                        class="text-muted hover:text-secondary text-xs cursor-pointer"
                                        on:click=move |_| clear_note_form()
                                    >"\u{2715} Cancel"</button>
                                </div>

                                // Note type selector (only when creating new)
                                {(!is_editing).then(|| view! {
                                    <div>
                                        <label class="block text-muted text-xs font-medium mb-1">"Type"</label>
                                        <select
                                            class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus:outline-none focus:border-accent/50 cursor-pointer"
                                            prop:value=move || note_form_type.get()
                                            on:change=move |ev| set_note_form_type.set(event_target_value(&ev))
                                        >
                                            {NOTE_TYPE_LIST.iter().map(|&t| {
                                                view! { <option value=t>{note_type_label(t)}</option> }
                                            }).collect_view()}
                                        </select>
                                    </div>
                                })}

                                <div>
                                    <label class="block text-muted text-xs font-medium mb-1">
                                        {if is_matchup { "Opponent Champion" } else { "Title" }}
                                    </label>
                                    <input type="text"
                                        class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus:outline-none focus:border-accent/50 transition-colors"
                                        placeholder=move || if note_form_type.get() == "matchup" { "e.g. Syndra" } else { "e.g. Level 6 all-in" }
                                        prop:value=move || note_form_title.get()
                                        on:input=move |ev| set_note_form_title.set(event_target_value(&ev))
                                    />
                                </div>

                                // Difficulty (matchups only)
                                {is_matchup.then(|| view! {
                                    <div>
                                        <label class="block text-muted text-xs font-medium mb-1">"Difficulty"</label>
                                        <div class="flex gap-1">
                                            {(1u8..=5).map(|level| {
                                                let is_filled = note_form_difficulty.get().unwrap_or(0) >= level;
                                                view! {
                                                    <button
                                                        class="text-lg cursor-pointer transition-colors hover:scale-110"
                                                        style=move || { if note_form_difficulty.get().unwrap_or(0) >= level { "color: #f87171" } else { "color: var(--color-muted)" } }
                                                        on:click=move |_| {
                                                            if note_form_difficulty.get_untracked() == Some(level) {
                                                                set_note_form_difficulty.set(None);
                                                            } else {
                                                                set_note_form_difficulty.set(Some(level));
                                                            }
                                                        }
                                                    >
                                                        {if is_filled { "\u{2605}" } else { "\u{2606}" }}
                                                    </button>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                })}

                                <div>
                                    <label class="block text-muted text-xs font-medium mb-1">"Content"</label>
                                    <textarea
                                        rows="3"
                                        class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus:outline-none focus:border-accent/50 resize-none transition-colors"
                                        placeholder="Notes, strategy, details..."
                                        prop:value=move || note_form_content.get()
                                        on:input=move |ev| set_note_form_content.set(event_target_value(&ev))
                                    />
                                </div>

                                <button
                                    class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm transition-colors cursor-pointer"
                                    on:click=move |_| do_save_note.run(())
                                >
                                    {if is_editing { "Update Note" } else { "Save Note" }}
                                </button>
                            </div>
                        }
                    })}
                </div>
            </div>
        </div>
    }
}
