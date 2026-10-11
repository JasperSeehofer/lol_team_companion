//! Champion pool: each player's champions per role in a tier ledger
//! (comfort to "to practise"), with comfort, meta fit and notes. Champions
//! move between tiers by drag or by keyboard.

use crate::components::champion_autocomplete::ChampionAutocomplete;
use crate::components::ui::{
    AvatarSize, Badge, BadgeTone, Button, ButtonSize, ButtonVariant, ChampionAvatar, ConfirmDialog,
    ConfirmTone, Density, Dialog, EmptySize, EmptyState, Field, Keycap, PageHeader, PageShell,
    Role, RoleLabel, Select, SelectOption, Skeleton, SkeletonGroup, SkeletonShape, TabItem,
    TabPanel, Tabs, TextField, Toast, ToastContext,
};
use crate::models::champion::{
    note_type_label, Champion, ChampionNote, ChampionPoolEntry, ChampionStatSummary,
};
use crate::models::utils::format_timestamp;
use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn get_pool() -> Result<Vec<ChampionPoolEntry>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    db::get_champion_pool(&db, &user.id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_pool_champions() -> Result<Vec<Champion>, ServerFnError> {
    use crate::server::authz;
    use crate::server::data_dragon;
    data_dragon::champions().await.map_err(authz::server_err)
}

#[server]
pub async fn add_to_pool(champion: String, role: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    db::add_to_champion_pool(&db, &user.id, champion, role)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn remove_from_pool(champion: String, role: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    db::remove_from_champion_pool(&db, &user.id, champion, role)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn set_champion_tier(
    champion: String,
    role: String,
    tier: String,
) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    db::update_champion_tier(&db, &user.id, &champion, &role, tier)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn set_champion_notes(
    champion: String,
    role: String,
    notes: String,
) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    let notes_opt = if notes.is_empty() { None } else { Some(notes) };
    db::update_champion_notes(&db, &user.id, &champion, &role, notes_opt)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn set_champion_comfort(
    champion: String,
    role: String,
    level: Option<u8>,
) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    db::update_champion_comfort(&db, &user.id, &champion, &role, level.map(|v| v as i64))
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn set_champion_meta_tag(
    champion: String,
    role: String,
    tag: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    db::update_champion_meta_tag(&db, &user.id, &champion, &role, tag)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_notes(champion: String, role: String) -> Result<Vec<ChampionNote>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    db::get_champion_notes(&db, &user.id, &champion, &role)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn save_note(note_json: String) -> Result<String, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    let note: ChampionNote = serde_json::from_str(&note_json).map_err(authz::server_err)?;

    if note.id.is_some() {
        db::update_champion_note(&db, &user.id, note)
            .await
            .map_err(authz::server_err)?;
        Ok(String::new())
    } else {
        db::add_champion_note(&db, &user.id, note)
            .await
            .map_err(authz::server_err)
    }
}

#[server]
pub async fn delete_note(note_id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    db::delete_champion_note(&db, &user.id, &note_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_my_champion_stats() -> Result<Vec<ChampionStatSummary>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    db::get_champion_stats_for_user(&db, &user.id)
        .await
        .map_err(authz::server_err)
}

// ---------------------------------------------------------------------------
// Constants & helpers
// ---------------------------------------------------------------------------

/// Roles as stored (the pool predates `Role`; "ADC" is the bot lane).
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
        "match_ready" => "Match ready",
        "scrim_ready" => "Scrim ready",
        "practicing" => "Practising",
        "to_practice" => "To practise",
        _ => "Other",
    }
}

fn tier_hint(tier: &str) -> &'static str {
    match tier {
        "comfort" => "Play it in any game",
        "match_ready" => "Ready for official games",
        "scrim_ready" => "Ready for scrims",
        "practicing" => "In practice now",
        "to_practice" => "Next to learn",
        _ => "",
    }
}

fn meta_label(tag: &str) -> &'static str {
    match tag {
        "strong" => "Meta strong",
        "neutral" => "Meta neutral",
        "weak" => "Off meta",
        _ => "",
    }
}

fn meta_tone(tag: &str) -> BadgeTone {
    match tag {
        "strong" => BadgeTone::Success,
        "weak" => BadgeTone::Warning,
        _ => BadgeTone::Neutral,
    }
}

const NOTE_TYPE_LIST: &[&str] = &[
    "matchup",
    "power_spike",
    "combo",
    "lesson",
    "synergy",
    "positioning",
];

/// Whether a stored role ("ADC", "bot", "Top") is the tab's role.
fn same_role(stored: &str, tab: &str) -> bool {
    Role::parse(stored).is_some() && Role::parse(stored) == Role::parse(tab)
}

/// The spelling new entries are stored with (the roster's: "top", "bot").
fn stored_role(tab: &str) -> String {
    Role::parse(tab)
        .map(|r| r.label().to_lowercase())
        .unwrap_or_else(|| tab.to_lowercase())
}

fn tier_index(tier: &str) -> usize {
    TIERS.iter().position(|t| *t == tier).unwrap_or(0)
}

/// Where a keyboard move lands: one tier up or down, clamped.
pub fn tier_after_move(tier: &str, key: &str) -> Option<&'static str> {
    let i = tier_index(tier);
    match key {
        "ArrowUp" if i > 0 => Some(TIERS[i - 1]),
        "ArrowDown" if i + 1 < TIERS.len() => Some(TIERS[i + 1]),
        _ => None,
    }
}

fn dom_key(champ: &str) -> String {
    champ
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn ChampionPoolPage() -> impl IntoView {
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
    let role_tab = RwSignal::new("Top".to_string());
    let selected = RwSignal::new(None::<(String, String)>);
    let announce = RwSignal::new(String::new());

    view! {
        <PageShell density=Density::Compact attr:data-feedback-label="Champion pool">
            <PageHeader
                title="Champion pool"
                eyebrow="Strategy"
                meta=move || view! {
                    <span>"Your champions per role, from comfort picks to the ones you are learning. The draft page shows them to your team."</span>
                }
            />
            <p class="sr-only" aria-live="polite" data-testid="pool-announce">{move || announce.get()}</p>
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading your pool"><Skeleton shape=SkeletonShape::Line lines=6 /></SkeletonGroup> }>
                {move || Suspend::new(async move {
                    let champs = champions_resource.await.unwrap_or_default();
                    let champs = StoredValue::new(champs);
                    view! {
                        <div class="grid grid-cols-1 gap-x-10 gap-y-10 lg:grid-cols-[minmax(0,1fr)_22rem]">
                            <div class="flex min-w-0 flex-col gap-6">
                                <Tabs
                                    group="pool-roles"
                                    label="Role"
                                    tabs={POOL_ROLES.iter().map(|r| TabItem::new(*r, Role::parse(r).map(|x| x.label()).unwrap_or(r))).collect::<Vec<_>>()}
                                    selected=role_tab
                                />
                                {POOL_ROLES.iter().map(|&role| view! {
                                    <TabPanel group="pool-roles" tab=role selected=role_tab>
                                        <RoleLedger
                                            role=role
                                            pool=pool
                                            stats=stats_resource
                                            champs=champs
                                            selected=selected
                                            announce=announce
                                        />
                                    </TabPanel>
                                }).collect_view()}
                            </div>
                            <ChampionDetail pool=pool champs=champs selected=selected />
                        </div>
                    }
                })}
            </Suspense>
        </PageShell>
    }
}

/// One role: the add field and the tier ledger.
#[component]
fn RoleLedger(
    role: &'static str,
    pool: Resource<Result<Vec<ChampionPoolEntry>, ServerFnError>>,
    stats: Resource<Result<Vec<ChampionStatSummary>, ServerFnError>>,
    champs: StoredValue<Vec<Champion>>,
    selected: RwSignal<Option<(String, String)>>,
    announce: RwSignal<String>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let add_value = RwSignal::new(String::new());
    let role_word = Role::parse(role).map(|r| r.label()).unwrap_or(role);
    let field_id = format!("pool-add-{}", role.to_lowercase());
    let add = Callback::new(move |champion: String| {
        if champion.trim().is_empty() {
            return;
        }
        let known = champs.with_value(|l| {
            l.iter()
                .find(|c| c.id == champion || c.name.eq_ignore_ascii_case(&champion))
                .map(|c| (c.id.clone(), c.name.clone()))
        });
        let Some((id, name)) = known else {
            toast
                .push
                .run(Toast::error("Choose a champion from the list."));
            return;
        };
        leptos::task::spawn_local(async move {
            match add_to_pool(id, stored_role(role).to_string()).await {
                Ok(()) => {
                    add_value.set(String::new());
                    pool.refetch();
                    toast.push.run(Toast::success(format!(
                        "{name} added to your {} pool",
                        role_word.to_lowercase()
                    )));
                }
                Err(e) => toast.push.run(Toast::error(format!("Could not add: {e}"))),
            }
        });
    });
    // Keyboard move: Space picks a champion up, Up/Down choose the tier,
    // Space or Enter drops it there, Escape puts it back.
    // (champion, stored role, name, target tier)
    let grabbed = RwSignal::new(None::<(String, String, String, &'static str)>);
    let drag_over = RwSignal::new(None::<&'static str>);
    let move_to = Callback::new(
        move |(champion, stored, name, tier): (String, String, String, &'static str)| {
            leptos::task::spawn_local(async move {
                match set_champion_tier(champion, stored, tier.to_string()).await {
                    Ok(()) => {
                        pool.refetch();
                        announce.set(format!("{name} moved to {}.", tier_label(tier)));
                    }
                    Err(e) => toast.push.run(Toast::error(format!("Could not move: {e}"))),
                }
            });
        },
    );

    view! {
        <div class="flex flex-col gap-5">
            <div class="flex flex-col gap-2 sm:flex-row sm:items-end">
                <div class="min-w-0 flex-1">
                    <Field id=field_id.clone() label=format!("Add a {} champion", role_word.to_lowercase()) hint="Type a name; Enter adds the highlighted champion.">
                        <ChampionAutocomplete
                            champions=champs.get_value()
                            value=add_value
                            placeholder="Ahri"
                            on_select=add
                        />
                    </Field>
                </div>
                <div class="sm:pb-6">
                    <Button on_click=Callback::new(move |_| add.run(add_value.get_untracked()))>"Add"</Button>
                </div>
            </div>
            <p class="flex flex-wrap items-center gap-x-3 gap-y-1 text-caption text-muted">
                <span class="inline-flex items-center gap-1"><Keycap keys="Space" />" pick up"</span>
                <span class="inline-flex items-center gap-1"><Keycap keys="\u{2191}" /><Keycap keys="\u{2193}" />" choose a tier"</span>
                <span class="inline-flex items-center gap-1"><Keycap keys="Space" />" drop"</span>
                <span class="inline-flex items-center gap-1"><Keycap keys="Esc" />" cancel"</span>
                <span>"Or drag with the pointer."</span>
            </p>
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading tiers"><Skeleton shape=SkeletonShape::Line lines=5 /></SkeletonGroup> }>
                {move || pool.get().map(|r| {
                    let entries: Vec<ChampionPoolEntry> = r.unwrap_or_default().into_iter().filter(|e| same_role(&e.role, role)).collect();
                    if entries.is_empty() {
                        return view! {
                            <EmptyState
                                size=EmptySize::Compact
                                message=format!("No {} champions yet. Add the ones you play; they start in Comfort and you can move them between tiers.", role_word.to_lowercase())
                            />
                        }.into_any();
                    }
                    let stats_list = stats.get().and_then(|r| r.ok()).unwrap_or_default();
                    view! {
                        <ol class="flex flex-col border-t border-rule" aria-label=format!("{role_word} tiers")>
                            {TIERS.iter().map(|&tier| {
                                let in_tier: Vec<ChampionPoolEntry> = entries.iter().filter(|e| e.tier == tier).cloned().collect();
                                let count = in_tier.len();
                                let stats_list = stats_list.clone();
                                view! {
                                    <li
                                        class=move || {
                                            let target = grabbed.with(|g| g.as_ref().map(|(_, _, _, t)| *t == tier).unwrap_or(false)) || drag_over.get() == Some(tier);
                                            if target {
                                                "grid grid-cols-1 gap-x-6 gap-y-2 border-b border-rule bg-accent-soft py-3 sm:grid-cols-[10rem_minmax(0,1fr)]"
                                            } else {
                                                "grid grid-cols-1 gap-x-6 gap-y-2 border-b border-rule py-3 sm:grid-cols-[10rem_minmax(0,1fr)]"
                                            }
                                        }
                                        data-tier=tier
                                        on:dragover=move |ev: web_sys::DragEvent| { ev.prevent_default(); drag_over.set(Some(tier)); }
                                        on:dragleave=move |_| drag_over.set(None)
                                        on:drop=move |ev: web_sys::DragEvent| {
                                            ev.prevent_default();
                                            drag_over.set(None);
                                            if let Some(dt) = ev.data_transfer() {
                                                let champ = dt.get_data("text/x-champion").unwrap_or_default();
                                                let from = dt.get_data("text/x-source-tier").unwrap_or_default();
                                                let name = dt.get_data("text/x-name").unwrap_or_default();
                                                let stored = dt.get_data("text/x-role").unwrap_or_default();
                                                if !champ.is_empty() && from != tier {
                                                    move_to.run((champ, stored, name, tier));
                                                }
                                            }
                                        }
                                    >
                                        <div class="flex items-baseline gap-2 sm:flex-col sm:gap-0.5">
                                            <h3 class="font-display text-heading font-semibold text-primary">{tier_label(tier)}</h3>
                                            <span class="text-caption text-muted">{tier_hint(tier)}</span>
                                            <span class="figures text-caption text-muted">{if count == 1 { "1 champion".to_string() } else { format!("{count} champions") }}</span>
                                        </div>
                                        {if in_tier.is_empty() {
                                            view! { <p class="self-center text-label text-muted">"None yet."</p> }.into_any()
                                        } else {
                                            view! {
                                                <ul class="flex flex-wrap gap-1.5">
                                                    {in_tier.into_iter().map(|e| {
                                                        let name = champs.with_value(|l| l.iter().find(|c| c.id == e.champion).map(|c| c.name.clone())).unwrap_or_else(|| e.champion.clone());
                                                        let stat = stats_list.iter().find(|s| s.champion == e.champion || s.champion == name).cloned();
                                                        let champ = e.champion.clone();
                                                        let stored = e.role.clone();
                                                        let stored_sel = stored.clone();
                                                        let stored_key = stored.clone();
                                                        let stored_drag = stored.clone();
                                                        let champ_sel = champ.clone();
                                                        let champ_key = champ.clone();
                                                        let champ_drag = champ.clone();
                                                        let name_key = name.clone();
                                                        let name_drag = name.clone();
                                                        let item_id = format!("pool-{}-{}", role.to_lowercase(), dom_key(&champ));
                                                        let item_id_focus = item_id.clone();
                                                        let is_sel = Signal::derive({
                                                            let champ = champ.clone();
                                                            let stored = stored.clone();
                                                            move || selected.get().map(|(c, r)| c == champ && r == stored).unwrap_or(false)
                                                        });
                                                        let is_grabbed = Signal::derive({
                                                            let champ = champ.clone();
                                                            move || grabbed.with(|g| g.as_ref().map(|(c, _, _, _)| *c == champ).unwrap_or(false))
                                                        });
                                                        let comfort = e.comfort_level.unwrap_or(0);
                                                        let meta = e.meta_tag.clone().filter(|m| !m.is_empty());
                                                        let avatar_name = name.clone();
                                                        let name_attr = name.clone();
                                                        view! {
                                                            <li
                                                                draggable="true"
                                                                data-champion=name_attr
                                                                on:dragstart=move |ev: web_sys::DragEvent| {
                                                                    if let Some(dt) = ev.data_transfer() {
                                                                        let _ = dt.set_data("text/x-champion", &champ_drag);
                                                                        let _ = dt.set_data("text/x-name", &name_drag);
                                                                        let _ = dt.set_data("text/x-source-tier", tier);
                                                                        let _ = dt.set_data("text/x-role", &stored_drag);
                                                                    }
                                                                }
                                                                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                                                    let key = ev.key();
                                                                    let held = grabbed.get_untracked();
                                                                    match (key.as_str(), held) {
                                                                        (" ", None) => {
                                                                            ev.prevent_default();
                                                                            grabbed.set(Some((champ_key.clone(), stored_key.clone(), name_key.clone(), tier)));
                                                                            announce.set(format!("{name_key} picked up from {}. Up and Down choose a tier, Space drops, Escape cancels.", tier_label(tier)));
                                                                        }
                                                                        (" " | "Enter", Some((c, st, n, target))) if c == champ_key => {
                                                                            ev.prevent_default();
                                                                            grabbed.set(None);
                                                                            if target != tier {
                                                                                move_to.run((c, st, n, target));
                                                                            } else {
                                                                                announce.set(format!("{n} stays in {}.", tier_label(tier)));
                                                                            }
                                                                            let id = item_id_focus.clone();
                                                                            focus_id_later(id);
                                                                        }
                                                                        ("ArrowUp" | "ArrowDown", Some((c, st, n, target))) if c == champ_key => {
                                                                            ev.prevent_default();
                                                                            if let Some(next) = tier_after_move(target, &key) {
                                                                                grabbed.set(Some((c, st, n, next)));
                                                                                announce.set(format!("Over {}.", tier_label(next)));
                                                                            }
                                                                        }
                                                                        ("Escape", Some((_, _, n, _))) => {
                                                                            ev.prevent_default();
                                                                            grabbed.set(None);
                                                                            announce.set(format!("{n} stays in {}.", tier_label(tier)));
                                                                        }
                                                                        _ => {}
                                                                    }
                                                                }
                                                            >
                                                                <Button
                                                                    variant=ButtonVariant::Secondary
                                                                    size=ButtonSize::Md
                                                                    pressed=Signal::derive(move || is_sel.get() || is_grabbed.get())
                                                                    icon=move || view! { <ChampionAvatar name=avatar_name.clone() size=AvatarSize::Sm decorative=true /> }
                                                                    attr:id=item_id.clone()
                                                                    attr:aria-roledescription="draggable champion"
                                                                    on_click=Callback::new(move |_| selected.set(Some((champ_sel.clone(), stored_sel.clone()))))
                                                                >
                                                                    <span>{name.clone()}</span>
                                                                    {(comfort > 0).then(|| view! {
                                                                        <span class="figures text-caption font-normal text-muted" aria-label=format!("comfort {comfort} of 5")>
                                                                            {(0..5).map(|i| if i < comfort { '\u{2605}' } else { '\u{2606}' }).collect::<String>()}
                                                                        </span>
                                                                    })}
                                                                    {meta.map(|m| view! { <Badge tone=meta_tone(&m)>{meta_label(&m)}</Badge> })}
                                                                    {stat.map(|s| {
                                                                        let wr = if s.games > 0 { (s.wins as f64 / s.games as f64 * 100.0).round() as i32 } else { 0 };
                                                                        view! { <span class="figures text-caption font-normal text-muted">{format!("{}G {}%", s.games, wr)}</span> }
                                                                    })}
                                                                </Button>
                                                            </li>
                                                        }
                                                    }).collect_view()}
                                                </ul>
                                            }.into_any()
                                        }}
                                    </li>
                                }
                            }).collect_view()}
                        </ol>
                    }.into_any()
                })}
            </Suspense>
        </div>
    }
}

/// Focus an element after the list re-renders (the moved champion's button).
fn focus_id_later(id: String) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        let cb = Closure::once(move || crate::components::ui::a11y::focus_id(&id));
        if let Some(win) = web_sys::window() {
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                400,
            );
        }
        cb.forget();
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = id;
}

/// The selected champion: tier, comfort, meta, notes, removal.
#[component]
fn ChampionDetail(
    pool: Resource<Result<Vec<ChampionPoolEntry>, ServerFnError>>,
    champs: StoredValue<Vec<Champion>>,
    selected: RwSignal<Option<(String, String)>>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let entry = Memo::new(move |_| {
        let (c, r) = selected.get()?;
        pool.get()
            .and_then(|p| p.ok())
            .and_then(|l| l.into_iter().find(|e| e.champion == c && e.role == r))
    });
    let tier = RwSignal::new(String::new());
    let comfort = RwSignal::new(String::new());
    let meta = RwSignal::new(String::new());
    let general = RwSignal::new(String::new());
    // Follow the selection (not every refetch of the same entry).
    let key = Memo::new(move |_| entry.get().map(|e| (e.champion, e.role)));
    Effect::new(move |_| {
        let _ = key.get();
        if let Some(e) = entry.get_untracked() {
            tier.set(e.tier.clone());
            comfort.set(e.comfort_level.map(|c| c.to_string()).unwrap_or_default());
            meta.set(e.meta_tag.clone().unwrap_or_default());
            general.set(e.notes.clone().unwrap_or_default());
        }
    });
    let save_field = move |what: &'static str| {
        let Some(e) = entry.get_untracked() else {
            return;
        };
        let (c, r) = (e.champion.clone(), e.role.clone());
        leptos::task::spawn_local(async move {
            let res = match what {
                "tier" => set_champion_tier(c, r, tier.get_untracked()).await,
                "comfort" => {
                    set_champion_comfort(c, r, comfort.get_untracked().parse::<u8>().ok()).await
                }
                "meta" => {
                    set_champion_meta_tag(
                        c,
                        r,
                        Some(meta.get_untracked()).filter(|m| !m.is_empty()),
                    )
                    .await
                }
                _ => set_champion_notes(c, r, general.get_untracked()).await,
            };
            match res {
                Ok(()) => {
                    pool.refetch();
                    if what == "notes" {
                        toast.push.run(Toast::success("Notes saved"));
                    }
                }
                Err(e) => toast.push.run(Toast::error(format!("Could not save: {e}"))),
            }
        });
    };
    let remove = Callback::new(move |_: leptos::ev::MouseEvent| {
        let Some(e) = entry.get_untracked() else {
            return;
        };
        let name = champs
            .with_value(|l| {
                l.iter()
                    .find(|c| c.id == e.champion)
                    .map(|c| c.name.clone())
            })
            .unwrap_or_else(|| e.champion.clone());
        leptos::task::spawn_local(async move {
            match remove_from_pool(e.champion.clone(), e.role.clone()).await {
                Ok(()) => {
                    selected.set(None);
                    pool.refetch();
                    let undo = Callback::new(move |_| {
                        let e = e.clone();
                        leptos::task::spawn_local(async move {
                            let (c, r) = (e.champion.clone(), e.role.clone());
                            if add_to_pool(c.clone(), r.clone()).await.is_ok() {
                                let _ =
                                    set_champion_tier(c.clone(), r.clone(), e.tier.clone()).await;
                                let _ = set_champion_comfort(c.clone(), r.clone(), e.comfort_level)
                                    .await;
                                let _ =
                                    set_champion_meta_tag(c.clone(), r.clone(), e.meta_tag.clone())
                                        .await;
                                let _ =
                                    set_champion_notes(c, r, e.notes.clone().unwrap_or_default())
                                        .await;
                                pool.refetch();
                            }
                        });
                    });
                    toast.push.run(
                        Toast::info(format!("{name} removed from the pool"))
                            .with_action("Undo", undo),
                    );
                }
                Err(err) => toast
                    .push
                    .run(Toast::error(format!("Could not remove: {err}"))),
            }
        });
    });

    view! {
        <aside aria-labelledby="pool-detail-heading" class="flex min-w-0 flex-col gap-5 lg:sticky lg:top-24 lg:self-start">
            {move || match entry.get() {
                None => view! {
                    <h2 id="pool-detail-heading" class="sr-only">"Champion"</h2>
                    <EmptyState
                        size=EmptySize::Compact
                        message="Choose a champion in your pool to set its tier, comfort and meta fit, and to keep matchup notes, combos and lessons."
                    />
                }.into_any(),
                Some(e) => {
                    let name = champs.with_value(|l| l.iter().find(|c| c.id == e.champion).map(|c| c.name.clone())).unwrap_or_else(|| e.champion.clone());
                    let role = Role::parse(&e.role);
                    view! {
                        <div class="flex items-center gap-3 border-b border-rule pb-4">
                            <ChampionAvatar name=name.clone() size=AvatarSize::Xl decorative=true />
                            <div class="min-w-0">
                                <h2 id="pool-detail-heading" class="font-display text-title font-semibold text-primary">{name.clone()}</h2>
                                <p class="flex items-center gap-2">
                                    {role.map(|r| view! { <RoleLabel role=r /> })}
                                    <span class="text-label text-secondary">{tier_label(&e.tier)}</span>
                                </p>
                            </div>
                        </div>
                        <Field id="pool-tier" label="Tier">
                            <Select value=tier options={TIERS.iter().map(|t| SelectOption::new(*t, tier_label(t))).collect::<Vec<_>>()} />
                        </Field>
                        <div class="grid grid-cols-2 gap-3">
                            <Field id="pool-comfort" label="Comfort">
                                <Select value=comfort options={
                                    let mut o = vec![SelectOption::new("", "Not set")];
                                    o.extend((1..=5).map(|i| SelectOption::new(i.to_string(), format!("{i} of 5"))));
                                    o
                                } />
                            </Field>
                            <Field id="pool-meta" label="Meta fit">
                                <Select value=meta options=vec![
                                    SelectOption::new("", "Not set"),
                                    SelectOption::new("strong", "Strong"),
                                    SelectOption::new("neutral", "Neutral"),
                                    SelectOption::new("weak", "Weak"),
                                ] />
                            </Field>
                        </div>
                        <Field id="pool-general" label="General notes">
                            <TextField value=general rows=3 placeholder="When you pick it, what it needs from the team." />
                        </Field>
                        <div class="flex flex-wrap gap-2">
                            <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| { save_field("tier"); save_field("comfort"); save_field("meta"); save_field("notes"); })>
                                "Save"
                            </Button>
                            <Button variant=ButtonVariant::Danger on_click=remove>"Remove from pool"</Button>
                        </div>
                        <ChampionNotes champion=e.champion.clone() role=e.role.clone() champs=champs />
                    }.into_any()
                }
            }}
        </aside>
    }
}

/// Matchups, notes and lessons for one champion in one role.
#[component]
fn ChampionNotes(
    champion: String,
    role: String,
    champs: StoredValue<Vec<Champion>>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let key = (champion.clone(), role.clone());
    let notes = Resource::new(
        move || key.clone(),
        |(c, r)| async move { get_notes(c, r).await },
    );
    let tab = RwSignal::new("matchup".to_string());
    let open = RwSignal::new(false);
    let editing = RwSignal::new(None::<String>);
    let f_type = RwSignal::new("matchup".to_string());
    let f_title = RwSignal::new(String::new());
    let f_opponent = RwSignal::new(String::new());
    let f_content = RwSignal::new(String::new());
    let f_difficulty = RwSignal::new(String::new());
    let f_error = RwSignal::new(None::<String>);
    let champion = StoredValue::new(champion);
    let role = StoredValue::new(role);

    let start = move |kind: &str, note: Option<ChampionNote>| {
        f_error.set(None);
        match note {
            Some(n) => {
                editing.set(n.id.clone());
                f_type.set(n.note_type.clone());
                if n.note_type == "matchup" {
                    f_opponent.set(n.title.strip_prefix("vs ").unwrap_or(&n.title).to_string());
                    f_title.set(String::new());
                } else {
                    f_title.set(n.title.clone());
                    f_opponent.set(String::new());
                }
                f_content.set(n.content.clone());
                f_difficulty.set(n.difficulty.map(|d| d.to_string()).unwrap_or_default());
            }
            None => {
                editing.set(None);
                f_type.set(kind.to_string());
                f_title.set(String::new());
                f_opponent.set(String::new());
                f_content.set(String::new());
                f_difficulty.set(String::new());
            }
        }
        open.set(true);
    };
    let save = Callback::new(move |_: leptos::ev::MouseEvent| {
        let kind = f_type.get_untracked();
        let title = if kind == "matchup" {
            let opp = f_opponent.get_untracked();
            let name = champs
                .with_value(|l| {
                    l.iter()
                        .find(|c| c.id == opp || c.name == opp)
                        .map(|c| c.name.clone())
                })
                .unwrap_or(opp);
            if name.trim().is_empty() {
                f_error.set(Some("Choose the champion this matchup is against.".into()));
                return;
            }
            format!("vs {name}")
        } else {
            f_title.get_untracked()
        };
        if title.trim().is_empty() {
            f_error.set(Some("Give the note a title.".into()));
            return;
        }
        let note = ChampionNote {
            id: editing.get_untracked(),
            user_id: String::new(),
            champion: champion.get_value(),
            role: role.get_value(),
            note_type: kind,
            title,
            content: f_content.get_untracked(),
            difficulty: f_difficulty.get_untracked().parse::<u8>().ok(),
            created_at: None,
        };
        let json = serde_json::to_string(&note).unwrap_or_default();
        leptos::task::spawn_local(async move {
            match save_note(json).await {
                Ok(_) => {
                    open.set(false);
                    notes.refetch();
                    toast.push.run(Toast::success("Note saved"));
                }
                Err(e) => f_error.set(Some(format!("Could not save: {e}"))),
            }
        });
    });
    let confirm = RwSignal::new(false);
    let to_delete = RwSignal::new(None::<String>);
    let do_delete = Callback::new(move |_| {
        let Some(id) = to_delete.get_untracked() else {
            return;
        };
        leptos::task::spawn_local(async move {
            match delete_note(id).await {
                Ok(()) => {
                    notes.refetch();
                    toast.push.run(Toast::success("Note deleted"));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not delete: {e}"))),
            }
        });
    });

    let group = |kind: &str| -> Vec<&'static str> {
        match kind {
            "matchup" => vec!["matchup"],
            "lesson" => vec!["lesson"],
            _ => vec!["power_spike", "combo", "synergy", "positioning"],
        }
    };

    view! {
        <section aria-labelledby="pool-notes-heading" class="flex flex-col gap-3 border-t border-rule pt-4">
            <h3 id="pool-notes-heading" class="font-display text-heading font-semibold text-primary">"Notes"</h3>
            <Tabs
                group="pool-notes"
                label="Note kind"
                tabs=vec![TabItem::new("matchup", "Matchups"), TabItem::new("play", "Play"), TabItem::new("lesson", "Lessons")]
                selected=tab
            />
            {["matchup", "play", "lesson"].into_iter().map(|kind| view! {
                <TabPanel group="pool-notes" tab=kind selected=tab>
                    <div class="flex flex-col gap-2">
                        <Suspense fallback=|| view! { <Skeleton shape=SkeletonShape::Line lines=2 /> }>
                            {move || notes.get().map(|r| {
                                let kinds = group(kind);
                                let list: Vec<ChampionNote> = r.unwrap_or_default().into_iter().filter(|n| kinds.contains(&n.note_type.as_str())).collect();
                                if list.is_empty() {
                                    let msg = match kind {
                                        "matchup" => "No matchup notes yet: how you play against a champion, and how hard it is.",
                                        "lesson" => "No lessons yet: what you learned in a game, written after it.",
                                        _ => "No play notes yet: power spikes, combos, synergies and positioning.",
                                    };
                                    return view! { <p class="text-label text-secondary">{msg}</p> }.into_any();
                                }
                                view! {
                                    <ul class="flex flex-col">
                                        {list.into_iter().map(|n| {
                                            let n_edit = n.clone();
                                            let id = n.id.clone().unwrap_or_default();
                                            let date = n.created_at.as_deref().map(format_timestamp).unwrap_or_default();
                                            view! {
                                                <li class="flex flex-col gap-1 border-b border-rule py-2 last:border-b-0">
                                                    <div class="flex flex-wrap items-baseline gap-x-2">
                                                        {(kind == "play").then(|| view! { <span class="eyebrow">{note_type_label(&n.note_type)}</span> })}
                                                        <span class="text-label font-medium text-primary">{n.title.clone()}</span>
                                                        {n.difficulty.map(|d| view! { <span class="figures text-caption text-muted">{format!("difficulty {d} of 5")}</span> })}
                                                        {(kind == "lesson" && !date.is_empty()).then(|| view! { <span class="figures text-caption text-muted">{date}</span> })}
                                                    </div>
                                                    {(!n.content.is_empty()).then(|| view! { <p class="whitespace-pre-wrap text-label text-secondary">{n.content.clone()}</p> })}
                                                    <div class="flex gap-1">
                                                        <Button variant=ButtonVariant::Quiet size=ButtonSize::Sm on_click=Callback::new(move |_| start("", Some(n_edit.clone())))>"Edit"</Button>
                                                        <Button variant=ButtonVariant::Quiet size=ButtonSize::Sm on_click=Callback::new(move |_| { to_delete.set(Some(id.clone())); confirm.set(true); })>"Delete"</Button>
                                                    </div>
                                                </li>
                                            }
                                        }).collect_view()}
                                    </ul>
                                }.into_any()
                            })}
                        </Suspense>
                        <div>
                            <Button size=ButtonSize::Sm on_click=Callback::new(move |_| start(match kind { "play" => "power_spike", k => k }, None))>
                                {match kind { "matchup" => "Add a matchup", "lesson" => "Add a lesson", _ => "Add a play note" }}
                            </Button>
                        </div>
                    </div>
                </TabPanel>
            }).collect_view()}
            <Dialog
                id="pool-note"
                open=open
                title="Champion note"
                dismiss_on_scrim=false
                footer=move || view! {
                    <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                    <Button variant=ButtonVariant::Primary on_click=save>"Save note"</Button>
                }
            >
                <div class="flex flex-col gap-4 px-5 py-4">
                    <Field id="note-type" label="Kind">
                        <Select value=f_type options={NOTE_TYPE_LIST.iter().map(|t| SelectOption::new(*t, note_type_label(t))).collect::<Vec<_>>()} />
                    </Field>
                    {move || if f_type.get() == "matchup" {
                        view! {
                            <Field id="note-opponent" label="Against" error=Signal::derive(move || f_error.get())>
                                <ChampionAutocomplete champions=champs.get_value() value=f_opponent placeholder="Zed" />
                            </Field>
                            <Field id="note-difficulty" label="Difficulty">
                                <Select value=f_difficulty options={
                                    let mut o = vec![SelectOption::new("", "Not rated")];
                                    o.extend((1..=5).map(|i| SelectOption::new(i.to_string(), format!("{i} of 5"))));
                                    o
                                } />
                            </Field>
                        }.into_any()
                    } else {
                        view! {
                            <Field id="note-title" label="Title" error=Signal::derive(move || f_error.get())>
                                <TextField value=f_title placeholder="Level 6 all-in" />
                            </Field>
                        }.into_any()
                    }}
                    <Field id="note-content" label="Note">
                        <TextField value=f_content rows=4 placeholder="What to do, and why." />
                    </Field>
                </div>
            </Dialog>
            <ConfirmDialog
                id="pool-note-delete"
                open=confirm
                title="Delete this note?"
                message="The note is deleted for good."
                confirm_label="Delete note"
                tone=ConfirmTone::Destructive
                on_confirm=do_delete
            />
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_moves_step_one_tier_and_stop_at_the_ends() {
        assert_eq!(tier_after_move("comfort", "ArrowDown"), Some("match_ready"));
        assert_eq!(tier_after_move("match_ready", "ArrowUp"), Some("comfort"));
        assert_eq!(tier_after_move("comfort", "ArrowUp"), None);
        assert_eq!(tier_after_move("to_practice", "ArrowDown"), None);
        assert_eq!(tier_after_move("scrim_ready", "Enter"), None);
    }
}
