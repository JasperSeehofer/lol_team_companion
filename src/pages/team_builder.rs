use crate::components::ui::{EmptyState, ToastContext, ToastKind};
use crate::models::champion::{Champion, ChampionPoolEntry, ChampionStatSummary};
use crate::models::opponent::{Opponent, OpponentPlayer};
use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

/// Returns Vec<(user_id, username, role, pool_entries)> for starters
#[server]
pub async fn get_team_roster_with_pools(
) -> Result<Vec<(String, String, String, Vec<ChampionPoolEntry>)>, ServerFnError> {
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

    let (_, members) = match db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    let mut result = Vec::new();
    for member in members.iter().filter(|m| m.roster_type == "starter") {
        let pool = db::get_champion_pool(&db, &member.user_id)
            .await
            .unwrap_or_default();
        result.push((
            member.user_id.clone(),
            member.username.clone(),
            member.role.clone(),
            pool,
        ));
    }

    Ok(result)
}

/// Get per-champion match stats for all team starters.
#[server]
pub async fn get_team_stats_for_builder(
) -> Result<Vec<(String, Vec<ChampionStatSummary>)>, ServerFnError> {
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

    let (_, members) = match db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    let mut result = Vec::new();
    for member in members.iter().filter(|m| m.roster_type == "starter") {
        let stats = db::get_champion_stats_for_user(&db, &member.user_id)
            .await
            .unwrap_or_default();
        result.push((member.username.clone(), stats));
    }

    Ok(result)
}

/// List team's opponents
#[server]
pub async fn get_opponents_for_builder() -> Result<Vec<Opponent>, ServerFnError> {
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

    db::list_opponents(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Get opponent's players with recent champions
#[server]
pub async fn get_opponent_players_for_builder(
    opponent_id: String,
) -> Result<Vec<OpponentPlayer>, ServerFnError> {
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    match db::get_opponent(&db, &opponent_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some((_, players)) => Ok(players),
        None => Ok(Vec::new()),
    }
}

/// Get champion data from Data Dragon
#[server]
pub async fn get_champions_for_builder() -> Result<Vec<Champion>, ServerFnError> {
    use crate::server::data_dragon;
    data_dragon::fetch_champions()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Creates a new draft from the composition (5 picks, no bans)
#[server]
pub async fn save_comp_as_draft(
    name: String,
    champions_json: String,
    opponent: Option<String>,
    our_side: Option<String>,
    tags_json: String,
) -> Result<String, ServerFnError> {
    use crate::models::draft::DraftAction;
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db_ctx =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let champions: Vec<(String, String)> = serde_json::from_str(&champions_json)
        .map_err(|e| ServerFnError::new(format!("Invalid champions JSON: {e}")))?;
    let tags: Vec<String> = serde_json::from_str(&tags_json)
        .map_err(|e| ServerFnError::new(format!("Invalid tags JSON: {e}")))?;

    let team_id = db::get_user_team_id(&db_ctx, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("You must be in a team to save a draft"))?;

    // Build pick actions for our side (blue by default)
    let side = our_side.unwrap_or_else(|| "blue".to_string());
    let actions: Vec<DraftAction> = champions
        .iter()
        .enumerate()
        .filter(|(_, (_, champ))| !champ.is_empty())
        .map(|(i, (_, champ))| DraftAction {
            id: None,
            draft_id: String::new(),
            phase: "pick1".to_string(),
            side: side.clone(),
            champion: champ.clone(),
            order: i as i32,
            comment: None,
        })
        .collect();

    let comments: Vec<String> = vec![String::new(); 20];

    db::save_draft(
        &db_ctx,
        &team_id,
        &user.id,
        name,
        opponent,
        None,
        comments,
        actions,
        None,
        side,
        tags,
        None,
        None,
        None,
        None,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

const ROLES: [&str; 5] = ["top", "jungle", "mid", "bot", "support"];

fn role_display(role: &str) -> &'static str {
    match role.to_lowercase().as_str() {
        "top" => "Top",
        "jungle" => "Jungle",
        "mid" | "middle" => "Mid",
        "bot" | "adc" | "bottom" => "Bot",
        "support" | "sup" => "Support",
        _ => "Fill",
    }
}

fn role_icon_url(role: &str) -> &'static str {
    match role.to_lowercase().as_str() {
        "top" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-top.svg",
        "jungle" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-jungle.svg",
        "mid" | "middle" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-middle.svg",
        "bot" | "adc" | "bottom" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-bottom.svg",
        "support" | "sup" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-utility.svg",
        _ => "",
    }
}

fn champion_image_url(champion_name: &str) -> String {
    format!(
        "https://ddragon.leagueoflegends.com/cdn/15.6.1/img/champion/{champion_name}.png"
    )
}

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

fn tier_badge_class(tier: &str) -> &'static str {
    match tier {
        "comfort" => "bg-accent/20 text-accent border-accent/30",
        "match_ready" => "bg-green-400/20 text-green-400 border-green-400/30",
        "scrim_ready" => "bg-blue-400/20 text-blue-400 border-blue-400/30",
        "practicing" => "bg-purple-400/20 text-purple-400 border-purple-400/30",
        "to_practice" => "bg-gray-400/20 text-muted border-gray-400/30",
        _ => "bg-gray-400/20 text-muted border-gray-400/30",
    }
}

fn comfort_stars(level: u8) -> String {
    let filled = level.min(5) as usize;
    let empty = 5 - filled;
    format!(
        "{}{}",
        "\u{2605}".repeat(filled),
        "\u{2606}".repeat(empty)
    )
}

/// Compute composition tags from selected champions + Data Dragon champion data.
fn compute_comp_tags(
    selected: &[(String, String)], // (role, champion_name)
    all_champions: &[Champion],
) -> Vec<String> {
    use std::collections::HashMap;

    let champ_map: HashMap<&str, &Champion> = all_champions
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();

    let picked: Vec<(&str, Option<&&Champion>)> = selected
        .iter()
        .filter(|(_, name)| !name.is_empty())
        .map(|(role, name)| (role.as_str(), champ_map.get(name.as_str())))
        .collect();

    if picked.is_empty() {
        return Vec::new();
    }

    let mut tags = Vec::new();

    // Count champion tags
    let mut fighter_count = 0u32;
    let mut tank_count = 0u32;
    let mut mage_count = 0u32;
    let mut assassin_count = 0u32;
    let mut marksman_count = 0u32;

    for (_, champ_opt) in &picked {
        if let Some(champ) = champ_opt {
            for tag in &champ.tags {
                match tag.as_str() {
                    "Fighter" => fighter_count += 1,
                    "Tank" => tank_count += 1,
                    "Mage" => mage_count += 1,
                    "Assassin" => assassin_count += 1,
                    "Marksman" => marksman_count += 1,
                    _ => {}
                }
            }
        }
    }

    // "teamfight" - 3+ fighters/tanks/mages
    if fighter_count + tank_count + mage_count >= 3 {
        tags.push("teamfight".to_string());
    }

    // "split-push" - contains known split-pushers
    let split_pushers = [
        "Fiora", "Tryndamere", "Jax", "Camille", "Gwen", "Yorick", "Nasus", "Shen",
    ];
    let has_split = picked
        .iter()
        .any(|(_, c)| c.map_or(false, |ch| split_pushers.contains(&ch.name.as_str())));
    if has_split {
        tags.push("split-push".to_string());
    }

    // "poke" - 2+ mages or marksmen (ranged poke)
    if mage_count + marksman_count >= 3 {
        tags.push("poke".to_string());
    }

    // "pick" - 2+ assassins
    if assassin_count >= 2 {
        tags.push("pick".to_string());
    }

    // "scaling" - 3+ late-game scalers
    let scalers = [
        "Kayle", "Kassadin", "Vayne", "Jinx", "Kog'Maw", "Veigar", "Senna", "Aphelios",
        "Zeri", "Smolder", "Viktor", "Azir", "Ryze", "Cassiopeia", "Vladimir",
    ];
    let scaler_count = picked
        .iter()
        .filter(|(_, c)| c.map_or(false, |ch| scalers.contains(&ch.name.as_str())))
        .count();
    if scaler_count >= 3 {
        tags.push("scaling".to_string());
    }

    // "early-game" - 3+ strong early champions
    let early_game = [
        "Draven", "Renekton", "Lee Sin", "Elise", "Pantheon", "Lucian", "Jayce", "Nidalee",
        "Rek'Sai", "Olaf", "Rumble", "Caitlyn",
    ];
    let early_count = picked
        .iter()
        .filter(|(_, c)| c.map_or(false, |ch| early_game.contains(&ch.name.as_str())))
        .count();
    if early_count >= 3 {
        tags.push("early-game".to_string());
    }

    // "protect-the-carry" - enchanter support + hypercarry ADC
    let enchanters = [
        "Lulu", "Janna", "Soraka", "Nami", "Yuumi", "Karma", "Sona", "Milio",
    ];
    let hypercarries = [
        "Kog'Maw", "Jinx", "Vayne", "Twitch", "Aphelios", "Zeri", "Smolder",
    ];
    let has_enchanter = picked.iter().any(|(role, c)| {
        role.to_lowercase() == "support"
            && c.map_or(false, |ch| enchanters.contains(&ch.name.as_str()))
    });
    let has_hypercarry = picked.iter().any(|(role, c)| {
        (role.to_lowercase() == "bot" || role.to_lowercase() == "adc")
            && c.map_or(false, |ch| hypercarries.contains(&ch.name.as_str()))
    });
    if has_enchanter && has_hypercarry {
        tags.push("protect-the-carry".to_string());
    }

    tags
}

fn comp_tag_class(tag: &str) -> &'static str {
    match tag {
        "teamfight" => "bg-red-500/20 text-red-400 border-red-500/30",
        "split-push" => "bg-orange-500/20 text-orange-400 border-orange-500/30",
        "poke" => "bg-blue-500/20 text-blue-400 border-blue-500/30",
        "pick" => "bg-purple-500/20 text-purple-400 border-purple-500/30",
        "scaling" => "bg-cyan-500/20 text-cyan-400 border-cyan-500/30",
        "early-game" => "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
        "protect-the-carry" => "bg-pink-500/20 text-pink-400 border-pink-500/30",
        _ => "bg-gray-500/20 text-gray-400 border-gray-500/30",
    }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn TeamBuilderPage() -> impl IntoView {
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

    // Resources
    let roster = Resource::new(|| (), |_| get_team_roster_with_pools());
    let team_stats = Resource::new(|| (), |_| get_team_stats_for_builder());
    let champions_data = Resource::new(|| (), |_| get_champions_for_builder());
    let opponents = Resource::new(|| (), |_| get_opponents_for_builder());

    // Selected champion per role (index by ROLES order)
    let selected_top: RwSignal<String> = RwSignal::new(String::new());
    let selected_jungle: RwSignal<String> = RwSignal::new(String::new());
    let selected_mid: RwSignal<String> = RwSignal::new(String::new());
    let selected_bot: RwSignal<String> = RwSignal::new(String::new());
    let selected_support: RwSignal<String> = RwSignal::new(String::new());

    let selected_signals: [RwSignal<String>; 5] = [
        selected_top,
        selected_jungle,
        selected_mid,
        selected_bot,
        selected_support,
    ];

    // Opponent selection
    let selected_opponent_id: RwSignal<Option<String>> = RwSignal::new(None);
    let opponent_players = Resource::new(
        move || selected_opponent_id.get(),
        move |id| async move {
            match id {
                Some(id) => get_opponent_players_for_builder(id).await,
                None => Ok(Vec::new()),
            }
        },
    );

    let toast = use_context::<ToastContext>().expect("ToastProvider");

    // Opponent section collapsed state
    let opponent_expanded: RwSignal<bool> = RwSignal::new(false);

    // Comp name for saving
    let comp_name: RwSignal<String> = RwSignal::new(String::new());

    view! {
        <div class="max-w-6xl mx-auto py-8 px-6">
            // Header
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-primary mb-2">"Team Builder"</h1>
                <p class="text-muted">"Explore team compositions from your champion pools"</p>
            </div>

            <Suspense fallback=move || view! { <p class="text-muted">"Loading roster..."</p> }>
                {move || {
                    let roster_data = roster.get().unwrap_or(Ok(Vec::new())).unwrap_or_default();
                    let stats_data = team_stats.get().unwrap_or(Ok(Vec::new())).unwrap_or_default();
                    let champs = champions_data.get().unwrap_or(Ok(Vec::new())).unwrap_or_default();
                    let opps = opponents.get().unwrap_or(Ok(Vec::new())).unwrap_or_default();
                    let opp_players_data = opponent_players.get().unwrap_or(Ok(Vec::new())).unwrap_or_default();

                    if roster_data.is_empty() {
                        return view! {
                            <EmptyState
                                icon="⚗️"
                                message="No team compositions saved yet — use the builder below to try role combinations"
                            />
                        }.into_any();
                    }

                    // Build a map of role -> (user_id, username, pool)
                    let mut role_map: Vec<(String, String, String, Vec<ChampionPoolEntry>)> = Vec::new();
                    for role in ROLES {
                        let found = roster_data.iter().find(|(_, _, r, _)| {
                            r.to_lowercase() == role
                        });
                        match found {
                            Some((uid, uname, _r, pool)) => {
                                role_map.push((role.to_string(), uid.clone(), uname.clone(), pool.clone()));
                            }
                            None => {
                                role_map.push((role.to_string(), String::new(), String::new(), Vec::new()));
                            }
                        }
                    }

                    // Stats map: username -> Vec<ChampionStatSummary>
                    let stats_map: std::collections::HashMap<String, Vec<ChampionStatSummary>> =
                        stats_data.into_iter().collect();

                    let champs_for_tags = champs.clone();
                    let champs_for_save = champs;
                    let opps_for_save = opps.clone();

                    view! {
                        // Role slots
                        <div class="grid grid-cols-1 md:grid-cols-5 gap-4 mb-8">
                            {role_map.into_iter().enumerate().map(|(idx, (role, _user_id, username, pool))| {
                                let signal = selected_signals[idx];
                                let role_display_name = role_display(&role);
                                let icon_url = role_icon_url(&role);
                                let username_for_stats = username.clone();
                                let stats_for_role = stats_map.get(&username_for_stats).cloned().unwrap_or_default();

                                // Group pool by tier
                                let tiers_order = ["comfort", "match_ready", "scrim_ready", "practicing", "to_practice"];
                                let pool_for_select = pool.clone();
                                let stats_for_badge = stats_for_role.clone();

                                view! {
                                    <div class="bg-elevated rounded-lg border border-divider p-4 flex flex-col gap-3">
                                        // Role header
                                        <div class="flex items-center gap-2">
                                            {if !icon_url.is_empty() {
                                                view! {
                                                    <img
                                                        src=icon_url
                                                        alt=role_display_name
                                                        class="w-5 h-5 brightness-75"
                                                    />
                                                }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }}
                                            <span class="text-sm font-semibold text-primary">{role_display_name}</span>
                                        </div>

                                        // Player name
                                        {if !username.is_empty() {
                                            view! {
                                                <p class="text-xs text-secondary truncate">{username}</p>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <p class="text-xs text-dimmed italic">"No starter assigned"</p>
                                            }.into_any()
                                        }}

                                        // Champion select dropdown grouped by tier
                                        <select
                                            class="w-full bg-surface/50 border border-outline/50 rounded px-2 py-1.5 text-sm text-primary focus:outline-none focus:border-accent"
                                            prop:value=move || signal.get()
                                            on:change=move |ev| {
                                                signal.set(event_target_value(&ev));
                                            }
                                        >
                                            <option value="">"-- Select --"</option>
                                            {tiers_order.iter().map(|&tier| {
                                                let tier_entries: Vec<&ChampionPoolEntry> = pool_for_select.iter()
                                                    .filter(|e| e.tier == tier && e.role.to_lowercase() == role.to_lowercase())
                                                    .collect();
                                                if tier_entries.is_empty() {
                                                    view! { <optgroup label=""></optgroup> }.into_any()
                                                } else {
                                                    let label = tier_label(tier);
                                                    view! {
                                                        <optgroup label=label>
                                                            {tier_entries.into_iter().map(|entry| {
                                                                let champ_name = entry.champion.clone();
                                                                let comfort = entry.comfort_level.unwrap_or(0);
                                                                let display = if comfort > 0 {
                                                                    format!("{} ({})", champ_name, comfort_stars(comfort as u8))
                                                                } else {
                                                                    champ_name.clone()
                                                                };
                                                                view! {
                                                                    <option value=champ_name.clone()>{display}</option>
                                                                }
                                                            }).collect_view()}
                                                        </optgroup>
                                                    }.into_any()
                                                }
                                            }).collect_view()}
                                        </select>

                                        // Selected champion display
                                        {move || {
                                            let champ = signal.get();
                                            if champ.is_empty() {
                                                view! {
                                                    <div class="h-24 flex items-center justify-center border border-dashed border-outline/30 rounded-lg">
                                                        <span class="text-dimmed text-xs">"Pick a champion"</span>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                let img_url = champion_image_url(&champ);
                                                let pool_entry = pool.iter().find(|e| e.champion == champ);
                                                let tier = pool_entry.map(|e| e.tier.clone()).unwrap_or_default();
                                                let comfort = pool_entry.and_then(|e| e.comfort_level).unwrap_or(0);
                                                let tier_cls = tier_badge_class(&tier);
                                                let tier_lbl = tier_label(&tier);

                                                let stat = stats_for_badge.iter().find(|s| s.champion == champ);
                                                let stat_view = stat.map(|s| {
                                                    let wr = if s.games > 0 {
                                                        (s.wins as f64 / s.games as f64 * 100.0).round() as i32
                                                    } else { 0 };
                                                    let wr_color = if wr >= 60 { "text-green-400" }
                                                        else if wr >= 50 { "text-secondary" }
                                                        else { "text-red-400" };
                                                    view! {
                                                        <div class="flex items-center gap-2 text-xs">
                                                            <span class=format!("font-semibold {wr_color}")>
                                                                {format!("{}%", wr)}
                                                            </span>
                                                            <span class="text-dimmed">
                                                                {format!("({} games)", s.games)}
                                                            </span>
                                                            <span class="text-muted">
                                                                {format!("{:.1} KDA", s.avg_kda)}
                                                            </span>
                                                        </div>
                                                    }
                                                });

                                                view! {
                                                    <div class="flex flex-col items-center gap-2 border border-accent/30 rounded-lg p-3 bg-surface/30">
                                                        <img
                                                            src=img_url
                                                            alt=champ.clone()
                                                            class="w-14 h-14 rounded-full border-2 border-accent/50"
                                                        />
                                                        <span class="text-sm font-semibold text-primary">{champ.clone()}</span>
                                                        {if !tier.is_empty() {
                                                            view! {
                                                                <span class=format!("text-xs px-2 py-0.5 rounded-full border {tier_cls}")>
                                                                    {tier_lbl}
                                                                </span>
                                                            }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        {if comfort > 0 {
                                                            view! {
                                                                <span class="text-accent text-xs">{comfort_stars(comfort as u8)}</span>
                                                            }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        {stat_view}
                                                    </div>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
                                }
                            }).collect_view()}
                        </div>

                        // Composition tags
                        <div class="mb-8">
                            <h2 class="text-lg font-semibold text-primary mb-3">"Composition Identity"</h2>
                            {move || {
                                let selected: Vec<(String, String)> = ROLES.iter().enumerate().map(|(i, &role)| {
                                    (role.to_string(), selected_signals[i].get())
                                }).collect();
                                let filled_count = selected.iter().filter(|(_, c)| !c.is_empty()).count();
                                let tags = compute_comp_tags(&selected, &champs_for_tags);

                                if filled_count == 0 {
                                    view! {
                                        <p class="text-dimmed text-sm">"Select champions to see composition tags"</p>
                                    }.into_any()
                                } else if tags.is_empty() {
                                    view! {
                                        <p class="text-muted text-sm">"No strong composition identity detected"</p>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="flex flex-wrap gap-2">
                                            {tags.iter().map(|tag| {
                                                let cls = comp_tag_class(tag);
                                                let tag_display = tag.clone();
                                                view! {
                                                    <span class=format!("px-3 py-1 rounded-full text-sm font-medium border {cls}")>
                                                        {tag_display}
                                                    </span>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>

                        // Opponent section (collapsible)
                        <div class="mb-8 bg-elevated rounded-lg border border-divider">
                            <button
                                class="w-full flex items-center justify-between p-4 text-left hover:bg-surface/30 transition-colors rounded-lg"
                                on:click=move |_| opponent_expanded.set(!opponent_expanded.get_untracked())
                            >
                                <div class="flex items-center gap-2">
                                    <span class="text-lg font-semibold text-primary">"Opponent Intel"</span>
                                    {move || {
                                        let id = selected_opponent_id.get();
                                        if id.is_some() {
                                            view! {
                                                <span class="text-xs px-2 py-0.5 rounded-full bg-accent/20 text-accent border border-accent/30">"Selected"</span>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }
                                    }}
                                </div>
                                <span class="text-muted">
                                    {move || if opponent_expanded.get() { "\u{25B2}" } else { "\u{25BC}" }}
                                </span>
                            </button>

                            {move || {
                                if !opponent_expanded.get() {
                                    return view! { <div></div> }.into_any();
                                }

                                let opp_list = opps.clone();
                                let opp_players_list = opp_players_data.clone();

                                view! {
                                    <div class="p-4 pt-0 space-y-4">
                                        // Opponent selector
                                        <select
                                            class="w-full md:w-64 bg-surface/50 border border-outline/50 rounded px-3 py-2 text-sm text-primary focus:outline-none focus:border-accent"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                if val.is_empty() {
                                                    selected_opponent_id.set(None);
                                                } else {
                                                    selected_opponent_id.set(Some(val));
                                                }
                                            }
                                        >
                                            <option value="">"-- Select Opponent --"</option>
                                            {opp_list.iter().map(|o| {
                                                let id = o.id.clone().unwrap_or_default();
                                                let name = o.name.clone();
                                                view! {
                                                    <option value=id>{name}</option>
                                                }
                                            }).collect_view()}
                                        </select>

                                        // Opponent players
                                        {if opp_players_list.is_empty() {
                                            if selected_opponent_id.get_untracked().is_some() {
                                                view! {
                                                    <p class="text-dimmed text-sm">"No player data for this opponent. Add players in the Opponents page."</p>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <p class="text-dimmed text-sm">"Select an opponent to see their players"</p>
                                                }.into_any()
                                            }
                                        } else {
                                            view! {
                                                <div class="grid grid-cols-1 md:grid-cols-5 gap-3">
                                                    {opp_players_list.iter().map(|player| {
                                                        let role_icon = role_icon_url(&player.role);
                                                        let player_name = player.name.clone();
                                                        let recent = player.recent_champions.clone();
                                                        view! {
                                                            <div class="bg-surface/30 rounded-lg border border-outline/30 p-3">
                                                                <div class="flex items-center gap-2 mb-2">
                                                                    {if !role_icon.is_empty() {
                                                                        view! {
                                                                            <img
                                                                                src=role_icon
                                                                                alt=player.role.clone()
                                                                                class="w-4 h-4 brightness-75"
                                                                            />
                                                                        }.into_any()
                                                                    } else {
                                                                        view! { <span></span> }.into_any()
                                                                    }}
                                                                    <span class="text-sm font-medium text-primary">{player_name}</span>
                                                                </div>
                                                                {if !recent.is_empty() {
                                                                    view! {
                                                                        <div class="flex flex-wrap gap-1">
                                                                            {recent.iter().map(|c| {
                                                                                let img = champion_image_url(c);
                                                                                let name = c.clone();
                                                                                view! {
                                                                                    <img
                                                                                        src=img
                                                                                        alt=name
                                                                                        class="w-7 h-7 rounded-full border border-outline/30"
                                                                                        title=c.clone()
                                                                                    />
                                                                                }
                                                                            }).collect_view()}
                                                                        </div>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <p class="text-dimmed text-xs">"No recent champions"</p>
                                                                    }.into_any()
                                                                }}
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }}
                                    </div>
                                }.into_any()
                            }}
                        </div>

                        // Save as Draft section
                        <div class="bg-elevated rounded-lg border border-divider p-6">
                            <h2 class="text-lg font-semibold text-primary mb-4">"Save as Draft"</h2>
                            <div class="flex flex-col md:flex-row gap-4 items-end">
                                <div class="flex-1">
                                    <label class="block text-sm text-secondary mb-1">"Composition Name"</label>
                                    <input
                                        type="text"
                                        placeholder="e.g. Teamfight Comp v1"
                                        class="w-full bg-surface/50 border border-outline/50 rounded px-3 py-2 text-sm text-primary focus:outline-none focus:border-accent"
                                        prop:value=move || comp_name.get()
                                        on:input=move |ev| comp_name.set(event_target_value(&ev))
                                    />
                                </div>
                                <button
                                    class="px-6 py-2 bg-accent text-accent-contrast rounded font-semibold text-sm hover:opacity-90 transition-opacity disabled:opacity-50"
                                    on:click=move |_| {
                                        let name = comp_name.get_untracked();
                                        if name.trim().is_empty() {
                                            toast.show.run((ToastKind::Error, "Please enter a name for the composition".into()));
                                            return;
                                        }

                                        let selected: Vec<(String, String)> = ROLES.iter().enumerate().map(|(i, &role)| {
                                            (role.to_string(), selected_signals[i].get_untracked())
                                        }).collect();

                                        let filled: Vec<(String, String)> = selected.iter()
                                            .filter(|(_, c)| !c.is_empty())
                                            .cloned()
                                            .collect();

                                        if filled.is_empty() {
                                            toast.show.run((ToastKind::Error, "Select at least one champion".into()));
                                            return;
                                        }

                                        let champs_json = serde_json::to_string(&filled).unwrap_or_default();
                                        let tags = compute_comp_tags(&selected, &champs_for_save);
                                        let tags_json = serde_json::to_string(&tags).unwrap_or_default();

                                        let opp_name = selected_opponent_id.get_untracked().and_then(|id| {
                                            opps_for_save.iter().find(|o| o.id.as_deref() == Some(&id)).map(|o| o.name.clone())
                                        });

                                        leptos::task::spawn_local(async move {
                                            match save_comp_as_draft(name, champs_json, opp_name, Some("blue".to_string()), tags_json).await {
                                                Ok(_id) => {
                                                    toast.show.run((ToastKind::Success, "Composition saved".into()));
                                                }
                                                Err(e) => {
                                                    toast.show.run((ToastKind::Error, format!("{e}")));
                                                }
                                            }
                                        });
                                    }
                                >
                                    "Save as Draft"
                                </button>
                            </div>

                        </div>
                    }.into_any()
                }}
            </Suspense>
        </div>
    }
}
