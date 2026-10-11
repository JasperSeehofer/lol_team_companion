//! Game plans: the plan for one game (picks, how we win, role briefs,
//! notes, a pre-game checklist), linked to a draft and, after the game, to
//! a review.

use crate::components::champion_autocomplete::ChampionAutocomplete;
use crate::components::draft_board::slot_meta;
use crate::components::ui::a11y::focus_id;
use crate::components::ui::{
    AvatarSize, Button, ButtonSize, ButtonVariant, Callout, CalloutTone, ChampionAvatar, Checkbox,
    ConfirmDialog, ConfirmTone, Density, EmptySize, EmptyState, Field, PageHeader, PageShell,
    PrepSequence, PrepState, PrepStep, Role, RoleLabel, SectionHeader, Segment, SegmentSize,
    SegmentedControl, Select, SelectOption, Skeleton, SkeletonGroup, SkeletonShape, Table, Td,
    TextField, Th, Toast, ToastContext, Tr,
};
use crate::models::champion::Champion;
use crate::models::draft::{most_common_tag, Draft};
use crate::models::game_plan::{ChecklistInstance, ChecklistTemplate, GamePlan};
use crate::pages::navigate::{navigator, replace_url};
use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn list_plans() -> Result<Vec<GamePlan>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_game_plans(&surreal, &team_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn list_team_drafts() -> Result<Vec<Draft>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_drafts(&surreal, &team_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn create_plan(plan_json: String) -> Result<String, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
        .ok_or_else(|| ServerFnError::new(authz::NO_TEAM))?;

    let mut plan: GamePlan = serde_json::from_str(&plan_json).map_err(authz::server_err)?;
    if let Some(did) = plan.draft_id.as_deref().filter(|d| !d.is_empty()) {
        db::ensure_team_record(&surreal, "draft", did, &team_id)
            .await
            .map_err(authz::server_err)?;
    }
    plan.team_id = team_id;
    db::save_game_plan(&surreal, plan, &user.id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn update_plan(plan_json: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;
    let team_id = authz::require_team(&surreal, &user).await?;
    let plan: GamePlan = serde_json::from_str(&plan_json).map_err(authz::server_err)?;
    if let Some(did) = plan.draft_id.as_deref().filter(|d| !d.is_empty()) {
        db::ensure_team_record(&surreal, "draft", did, &team_id)
            .await
            .map_err(authz::server_err)?;
    }
    db::update_game_plan(&surreal, &team_id, plan)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn delete_plan(plan_id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;
    let team_id = authz::require_team(&surreal, &user).await?;
    db::delete_game_plan(&surreal, &team_id, &plan_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_champions_for_game_plan() -> Result<Vec<Champion>, ServerFnError> {
    use crate::server::authz;
    use crate::server::data_dragon;
    data_dragon::champions().await.map_err(authz::server_err)
}

/// Fetch a draft with its actions by ID, for prefilling a new game plan.
/// Returns None if the draft does not exist.
#[server]
pub async fn get_draft_for_prefill(
    draft_id: String,
) -> Result<Option<crate::models::draft::Draft>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;
    let Some(team_id) = authz::current_team_for(&surreal, &user).await? else {
        return Ok(None);
    };
    db::get_draft_for_prefill(&surreal, &team_id, &draft_id)
        .await
        .map_err(authz::server_err)
}

/// Check whether any game plan already references the given draft ID.
/// Returns Some(plan_id) if at least one plan exists, or None if none do.
/// Used for duplicate detection in the "Prep for This Draft" CTA.
#[server]
pub async fn check_draft_has_game_plan(draft_id: String) -> Result<Option<String>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;
    let Some(team_id) = authz::current_team_for(&surreal, &user).await? else {
        return Ok(None);
    };
    let plans = db::get_game_plans_for_draft(&surreal, &team_id, &draft_id)
        .await
        .map_err(authz::server_err)?;

    Ok(plans.into_iter().next().and_then(|p| p.id))
}

#[server]
pub async fn get_checklist_templates() -> Result<Vec<ChecklistTemplate>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_checklist_templates(&surreal, &team_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn save_checklist_template(
    name: String,
    items_json: String,
) -> Result<String, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
        .ok_or_else(|| ServerFnError::new(authz::NO_TEAM))?;

    let items: Vec<String> = serde_json::from_str(&items_json).map_err(authz::server_err)?;

    db::create_checklist_template(&surreal, &team_id, name, items)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn delete_checklist_template_fn(id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;
    let team_id = authz::require_team(&surreal, &user).await?;
    db::delete_checklist_template(&surreal, &team_id, &id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_plan_checklist(
    plan_id: String,
) -> Result<Option<ChecklistInstance>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;
    let Some(team_id) = authz::current_team_for(&surreal, &user).await? else {
        return Ok(None);
    };
    db::get_checklist_for_plan(&surreal, &team_id, &plan_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn create_plan_checklist(
    plan_id: String,
    template_id: Option<String>,
    items_json: String,
) -> Result<String, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
        .ok_or_else(|| ServerFnError::new(authz::NO_TEAM))?;

    let items: Vec<String> = serde_json::from_str(&items_json).map_err(authz::server_err)?;
    db::ensure_team_record(&surreal, "game_plan", &plan_id, &team_id)
        .await
        .map_err(authz::server_err)?;
    let template_id = template_id.filter(|t| !t.is_empty());
    if let Some(tid) = &template_id {
        db::ensure_team_record(&surreal, "checklist_template", tid, &team_id)
            .await
            .map_err(authz::server_err)?;
    }
    db::create_checklist_instance(&surreal, &team_id, Some(plan_id), template_id, items)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn update_checklist(
    instance_id: String,
    checked_json: String,
) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;
    let team_id = authz::require_team(&surreal, &user).await?;
    let checked: Vec<bool> = serde_json::from_str(&checked_json).map_err(authz::server_err)?;
    db::update_checklist_checked(&surreal, &team_id, &instance_id, checked)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn start_post_game_review(
    plan_id: String,
    draft_id: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::models::game_plan::PostGameLearning;
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
        .ok_or_else(|| ServerFnError::new(authz::NO_TEAM))?;

    db::ensure_team_record(&surreal, "game_plan", &plan_id, &team_id)
        .await
        .map_err(authz::server_err)?;
    let draft_id = draft_id.filter(|d| !d.is_empty());
    if let Some(did) = &draft_id {
        db::ensure_team_record(&surreal, "draft", did, &team_id)
            .await
            .map_err(authz::server_err)?;
    }

    let review = PostGameLearning {
        id: None,
        team_id,
        match_riot_id: None,
        game_plan_id: Some(plan_id),
        draft_id,
        what_went_well: Vec::new(),
        improvements: Vec::new(),
        action_items: Vec::new(),
        open_notes: None,
        created_by: user.id,
        win_loss: None,
        rating: None,
    };

    db::save_post_game_learning(&surreal, review)
        .await
        .map_err(authz::server_err)
}

// ---------------------------------------------------------------------------
// Template generation
// ---------------------------------------------------------------------------

fn generate_template(
    our: &[String],
    enemy: &[String],
) -> (Vec<String>, Vec<String>, String, String) {
    let mut win_conditions = Vec::new();
    let objectives = vec![
        "Dragon".to_string(),
        "Rift Herald".to_string(),
        "Baron".to_string(),
    ];

    // Simple heuristic based on champion count (placeholder for real analysis)
    if !our.is_empty() {
        win_conditions.push(format!(
            "Play around {} in teamfights",
            our.first().unwrap_or(&"carry".to_string())
        ));
    }
    if our.len() >= 3 {
        win_conditions.push("Control vision around neutral objectives before fights".to_string());
    }
    if !enemy.is_empty() {
        win_conditions.push(format!(
            "Deny {} from scaling",
            enemy.last().unwrap_or(&"enemy carry".to_string())
        ));
    }
    if win_conditions.is_empty() {
        win_conditions.push("Secure early game advantages through proactive plays".to_string());
        win_conditions.push("Transition leads into objective control".to_string());
    }

    let teamfight = if our.len() >= 5 {
        format!(
            "Front-to-back: protect {} while {} zones the enemy",
            our[3], our[0]
        )
    } else {
        "Focus priority targets and peel for carries".to_string()
    };

    let early = "Contest level 1 vision. Jungle path based on matchup. Look for early ganks on volatile lanes.".to_string();

    (win_conditions, objectives, teamfight, early)
}

#[server]
pub async fn get_strategy_win_rates() -> Result<Vec<(String, i32, i32)>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::get_win_condition_stats(&surreal, &team_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_strategy_win_rates_vs_opponent(
    opponent_name: String,
) -> Result<Vec<(String, i32, i32)>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::get_win_condition_stats_vs_opponent(&surreal, &team_id, &opponent_name)
        .await
        .map_err(authz::server_err)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const STRATEGY_TAGS: &[&str] = &[
    "teamfight",
    "split-push",
    "poke",
    "pick",
    "scaling",
    "early-game",
    "protect-the-carry",
];

fn opt(s: String) -> Option<String> {
    (!s.trim().is_empty()).then_some(s)
}

fn lines(s: &str) -> Vec<String> {
    s.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Picks of one side of a draft in pick order (champion per pick).
fn side_picks(d: &Draft, side: &str) -> Vec<String> {
    let mut picks: Vec<(i32, String)> = d
        .actions
        .iter()
        .filter(|a| a.side == side && slot_meta(a.order as usize).1 == "pick")
        .map(|a| (a.order, a.champion.clone()))
        .collect();
    picks.sort_by_key(|(o, _)| *o);
    let mut out: Vec<String> = picks.into_iter().map(|(_, c)| c).collect();
    out.resize(5, String::new());
    out
}

/// The plan being edited. Every field is a signal (Copy).
#[derive(Clone, Copy)]
struct PlanEd {
    id: RwSignal<Option<String>>,
    name: RwSignal<String>,
    draft_id: RwSignal<String>,
    ours: [RwSignal<String>; 5],
    theirs: [RwSignal<String>; 5],
    win: RwSignal<String>,
    objectives: RwSignal<String>,
    teamfight: RwSignal<String>,
    early: RwSignal<String>,
    roles: [RwSignal<String>; 5],
    notes: RwSignal<String>,
    tag: RwSignal<String>,
    /// Picks came from the linked draft (shown read-only until edited).
    from_draft: RwSignal<bool>,
}

impl PlanEd {
    fn new() -> Self {
        let s = || RwSignal::new(String::new());
        Self {
            id: RwSignal::new(None),
            name: s(),
            draft_id: s(),
            ours: [s(), s(), s(), s(), s()],
            theirs: [s(), s(), s(), s(), s()],
            win: s(),
            objectives: s(),
            teamfight: s(),
            early: s(),
            roles: [s(), s(), s(), s(), s()],
            notes: s(),
            tag: s(),
            from_draft: RwSignal::new(false),
        }
    }

    fn clear(&self) {
        self.id.set(None);
        for sig in [
            self.name,
            self.draft_id,
            self.win,
            self.objectives,
            self.teamfight,
            self.early,
            self.notes,
            self.tag,
        ] {
            sig.set(String::new());
        }
        for sig in self
            .ours
            .iter()
            .chain(self.theirs.iter())
            .chain(self.roles.iter())
        {
            sig.set(String::new());
        }
        self.from_draft.set(false);
    }

    fn load(&self, p: &GamePlan) {
        self.id.set(p.id.clone());
        self.name.set(p.name.clone());
        self.draft_id.set(p.draft_id.clone().unwrap_or_default());
        for (i, sig) in self.ours.iter().enumerate() {
            sig.set(p.our_champions.get(i).cloned().unwrap_or_default());
        }
        for (i, sig) in self.theirs.iter().enumerate() {
            sig.set(p.enemy_champions.get(i).cloned().unwrap_or_default());
        }
        self.win.set(p.win_conditions.join("\n"));
        self.objectives.set(p.objective_priority.join("\n"));
        self.teamfight.set(p.teamfight_strategy.clone());
        self.early.set(p.early_game.clone().unwrap_or_default());
        let r = [
            &p.top_strategy,
            &p.jungle_strategy,
            &p.mid_strategy,
            &p.bot_strategy,
            &p.support_strategy,
        ];
        for (i, sig) in self.roles.iter().enumerate() {
            sig.set(r[i].clone().unwrap_or_default());
        }
        self.notes.set(p.notes.clone().unwrap_or_default());
        self.tag
            .set(p.win_condition_tag.clone().unwrap_or_default());
        self.from_draft.set(false);
    }

    /// Seed picks (and, for a new plan, name, tag, win conditions and role
    /// notes) from a draft.
    fn apply_draft(&self, d: &Draft, new_plan: bool) {
        let our = if d.our_side == "red" { "red" } else { "blue" };
        let enemy = if our == "blue" { "red" } else { "blue" };
        for (i, c) in side_picks(d, our).into_iter().enumerate() {
            self.ours[i].set(c);
        }
        for (i, c) in side_picks(d, enemy).into_iter().enumerate() {
            self.theirs[i].set(c);
        }
        self.draft_id.set(d.id.clone().unwrap_or_default());
        self.from_draft.set(true);
        if !new_plan {
            return;
        }
        // Suggestions fill only what is still empty.
        if !d.name.is_empty() && self.name.get_untracked().trim().is_empty() {
            self.name.set(format!("{} plan", d.name));
        }
        if let Some(t) = most_common_tag(&d.tags) {
            if self.tag.get_untracked().is_empty() {
                self.tag.set(t);
            }
        }
        if let Some(wc) = d.win_conditions.clone().filter(|w| !w.is_empty()) {
            if self.win.get_untracked().trim().is_empty() {
                self.win.set(wc);
            }
        }
        for a in d
            .actions
            .iter()
            .filter(|a| a.side == our && slot_meta(a.order as usize).1 == "pick")
        {
            if let Some(i) = a
                .role
                .as_deref()
                .and_then(Role::parse)
                .and_then(|r| Role::ALL.iter().position(|x| *x == r))
            {
                if self.roles[i].get_untracked().is_empty() {
                    self.roles[i].set(format!("{}: ", a.champion));
                }
            }
        }
    }

    fn build(&self, tracked: bool) -> GamePlan {
        let g = |s: RwSignal<String>| if tracked { s.get() } else { s.get_untracked() };
        GamePlan {
            id: if tracked {
                self.id.get()
            } else {
                self.id.get_untracked()
            },
            team_id: String::new(),
            draft_id: opt(g(self.draft_id)),
            name: g(self.name),
            our_champions: self
                .ours
                .iter()
                .map(|s| g(*s))
                .filter(|s| !s.is_empty())
                .collect(),
            enemy_champions: self
                .theirs
                .iter()
                .map(|s| g(*s))
                .filter(|s| !s.is_empty())
                .collect(),
            win_conditions: lines(&g(self.win)),
            objective_priority: lines(&g(self.objectives)),
            teamfight_strategy: g(self.teamfight),
            early_game: opt(g(self.early)),
            top_strategy: opt(g(self.roles[0])),
            jungle_strategy: opt(g(self.roles[1])),
            mid_strategy: opt(g(self.roles[2])),
            bot_strategy: opt(g(self.roles[3])),
            support_strategy: opt(g(self.roles[4])),
            notes: opt(g(self.notes)),
            win_condition_tag: opt(g(self.tag)),
        }
    }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn GamePlanPage() -> impl IntoView {
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let query = leptos_router::hooks::use_query_map();
    let plans = Resource::new(|| (), |_| list_plans());
    let drafts = Resource::new(|| (), |_| list_team_drafts());
    let champions = Resource::new(|| (), |_| get_champions_for_game_plan());
    let win_rates = Resource::new(|| (), |_| get_strategy_win_rates());
    let ed = PlanEd::new();
    let save_state = RwSignal::new("");

    // `?draft_id=` prefills a new plan from that draft, once.
    let prefill = Resource::new(
        move || query.read().get("draft_id"),
        |id| async move {
            match id {
                Some(id) if !id.is_empty() => get_draft_for_prefill(id).await,
                _ => Ok(None),
            }
        },
    );
    let prefilled = StoredValue::new(false);
    Effect::new(move |_| {
        if prefilled.get_value() {
            return;
        }
        if let Some(Ok(Some(d))) = prefill.get() {
            // Start from a clean editor unless this is already a new plan
            // (then keep what the user typed while the draft loaded).
            if ed.id.get_untracked().is_some() {
                ed.clear();
            }
            ed.apply_draft(&d, true);
            prefilled.set_value(true);
        }
    });
    // What the open plan looked like when it was loaded or last saved, as
    // the editor builds it (so opening a plan does not count as a change).
    let loaded_snapshot: StoredValue<Option<GamePlan>> = StoredValue::new(None);
    // Bumped whenever the editor switches to another plan, so a save that
    // returns late does not write its id into a different plan.
    let edit_gen = StoredValue::new(0_u64);
    let saving = RwSignal::new(false);
    // `?plan_id=` opens that plan, once.
    let opened = StoredValue::new(false);
    Effect::new(move |_| {
        if opened.get_value() {
            return;
        }
        let Some(target) = query.read().get("plan_id").filter(|t| !t.is_empty()) else {
            return;
        };
        if let Some(Ok(list)) = plans.get() {
            if let Some(p) = list
                .into_iter()
                .find(|p| p.id.as_deref() == Some(target.as_str()))
            {
                ed.load(&p);
                loaded_snapshot.set_value(Some(ed.build(false)));
                save_state.set("saved");
                edit_gen.update_value(|g| *g += 1);
                opened.set_value(true);
            }
        }
    });

    // Auto-save existing plans 2 s after the last change (rule 54).
    #[allow(unused_variables)]
    let timer: RwSignal<Option<i32>> = RwSignal::new(None);
    Effect::new(move |_| {
        let plan = ed.build(true);
        if plan.id.is_none() || plan.name.trim().is_empty() {
            return;
        }
        if loaded_snapshot.with_value(|l| l.as_ref() == Some(&plan)) {
            return;
        }
        save_state.set("unsaved");
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;
            if let Some(t) = timer.get_untracked() {
                if let Some(win) = web_sys::window() {
                    win.clear_timeout_with_handle(t);
                }
            }
            let json = serde_json::to_string(&plan).unwrap_or_default();
            let gen = edit_gen.get_value();
            let cb = Closure::once(move || {
                leptos::task::spawn_local(async move {
                    if update_plan(json).await.is_ok() {
                        // Only the plan still open takes the new snapshot.
                        if edit_gen.get_value() == gen {
                            let unchanged = ed.build(false) == plan;
                            loaded_snapshot.set_value(Some(plan));
                            save_state.set(if unchanged { "saved" } else { "unsaved" });
                        }
                        plans.refetch();
                        win_rates.refetch();
                    }
                });
            });
            if let Some(win) = web_sys::window() {
                if let Ok(t) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    2000,
                ) {
                    timer.set(Some(t));
                }
            }
            cb.forget();
        }
    });
    // The open plan's id goes in the address so a reload or link reopens it.
    let nav = navigator();
    let open_plan = Callback::new(move |p: GamePlan| {
        ed.load(&p);
        loaded_snapshot.set_value(Some(ed.build(false)));
        save_state.set("saved");
        edit_gen.update_value(|g| *g += 1);
        opened.set_value(true);
        if let Some(id) = p.id.as_deref() {
            replace_url(nav, &format!("/game-plan?plan_id={id}"));
        }
    });

    let name_error = RwSignal::new(None::<String>);
    let do_save = Callback::new(move |_: leptos::ev::MouseEvent| {
        if saving.get_untracked() {
            return;
        }
        let plan = ed.build(false);
        if plan.name.trim().is_empty() {
            name_error.set(Some("Name the plan so the team can find it.".into()));
            focus_id("plan-name");
            return;
        }
        name_error.set(None);
        let json = serde_json::to_string(&plan).unwrap_or_default();
        let is_update = plan.id.is_some();
        let gen = edit_gen.get_value();
        saving.set(true);
        leptos::task::spawn_local(async move {
            let res = if is_update {
                update_plan(json).await.map(|_| String::new())
            } else {
                create_plan(json).await
            };
            saving.set(false);
            match res {
                Ok(id) => {
                    plans.refetch();
                    toast.push.run(Toast::success("Game plan saved"));
                    // The editor moved on to another plan meanwhile.
                    if edit_gen.get_value() != gen {
                        return;
                    }
                    // The snapshot is what was sent: anything typed while
                    // the request ran is still a change (and auto-saves).
                    let mut sent = plan;
                    if !is_update {
                        opened.set_value(true);
                        replace_url(nav, &format!("/game-plan?plan_id={id}"));
                        sent.id = Some(id.clone());
                        ed.id.set(Some(id));
                    }
                    let unchanged = ed.build(false) == sent;
                    loaded_snapshot.set_value(Some(sent));
                    save_state.set(if unchanged { "saved" } else { "unsaved" });
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not save the plan: {e}"))),
            }
        });
    });
    let new_plan = Callback::new(move |_: leptos::ev::MouseEvent| {
        opened.set_value(true);
        replace_url(nav, "/game-plan");
        ed.clear();
        edit_gen.update_value(|g| *g += 1);
        loaded_snapshot.set_value(None);
        save_state.set("");
        name_error.set(None);
    });
    let confirm_delete = RwSignal::new(false);
    let do_delete = Callback::new(move |_| {
        let Some(id) = ed.id.get_untracked() else {
            return;
        };
        leptos::task::spawn_local(async move {
            match delete_plan(id).await {
                Ok(()) => {
                    replace_url(nav, "/game-plan");
                    ed.clear();
                    edit_gen.update_value(|g| *g += 1);
                    loaded_snapshot.set_value(None);
                    save_state.set("");
                    plans.refetch();
                    toast.push.run(Toast::success("Game plan deleted"));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not delete: {e}"))),
            }
        });
    });

    let title = move || {
        let n = ed.name.get();
        if n.trim().is_empty() {
            "New game plan".to_string()
        } else {
            n
        }
    };

    view! {
        <PageShell density=Density::Compact attr:data-feedback-label="Game plan">
            <PageHeader
                title=title
                eyebrow="Game plan"
                meta=move || view! {
                    <span data-testid="plan-save-state" class="text-muted">
                        {move || match (ed.id.get().is_some(), save_state.get()) {
                            (false, _) => "Not saved yet",
                            (true, "unsaved") => "Unsaved changes",
                            (true, _) => "Saved",
                        }}
                    </span>
                }
                actions=move || view! {
                    <Button variant=ButtonVariant::Quiet on_click=new_plan>"New plan"</Button>
                    {move || ed.id.get().map(|_| view! {
                        <Button variant=ButtonVariant::Danger on_click=Callback::new(move |_| confirm_delete.set(true))>"Delete"</Button>
                    })}
                    <Button variant=ButtonVariant::Primary on_click=do_save loading=saving attr:data-testid="plan-save">"Save plan"</Button>
                }
            />
            <ConfirmDialog
                id="delete-plan"
                open=confirm_delete
                title="Delete this game plan?"
                message="The plan and its checklist are deleted. Reviews linked to it keep their notes."
                confirm_label="Delete plan"
                tone=ConfirmTone::Destructive
                on_confirm=do_delete
            />
            <div class="grid grid-cols-1 gap-x-10 gap-y-10 lg:grid-cols-[16rem_minmax(0,1fr)]">
                <section aria-labelledby="plans-heading" class="flex min-w-0 flex-col gap-3">
                    <SectionHeader title="Plans" id="plans-heading" />
                    <Suspense fallback=|| view! { <SkeletonGroup label="Loading plans"><Skeleton shape=SkeletonShape::Line lines=3 /></SkeletonGroup> }>
                        {move || plans.get().map(|r| match r {
                            Err(_) => view! { <Callout tone=CalloutTone::Danger>"The plans did not load."</Callout> }.into_any(),
                            Ok(list) if list.is_empty() => view! {
                                <EmptyState size=EmptySize::Compact message="Plans your team saves are listed here. Write one on the right, or start one from a draft with Prep a game plan." />
                            }.into_any(),
                            Ok(list) => view! {
                                <ul class="flex flex-col border-t border-rule" data-testid="plan-list">
                                    {list.into_iter().map(|p| {
                                        let id = p.id.clone().unwrap_or_default();
                                        let name = if p.name.trim().is_empty() { "Untitled plan".to_string() } else { p.name.clone() };
                                        let tag = p.win_condition_tag.clone();
                                        let p2 = p.clone();
                                        view! {
                                            <li class="border-b border-rule">
                                                <Button
                                                    variant=ButtonVariant::Quiet
                                                    full_width=true
                                                    pressed=Signal::derive(move || ed.id.get().as_deref() == Some(id.as_str()))
                                                    on_click=Callback::new(move |_| open_plan.run(p2.clone()))
                                                >
                                                    <span class="flex w-full min-w-0 items-baseline justify-between gap-2 text-left">
                                                        <span class="truncate">{name}</span>
                                                        {tag.map(|t| view! { <span class="text-caption font-normal text-muted">{t}</span> })}
                                                    </span>
                                                </Button>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            }.into_any(),
                        })}
                    </Suspense>
                </section>
                <div class="flex min-w-0 flex-col gap-12">
                    <PlanSteps ed=ed />
                    <BasicsSection ed=ed drafts=drafts name_error=name_error />
                    <MatchupSection ed=ed champions=champions />
                    <StrategySection ed=ed win_rates=win_rates />
                    <RolesSection ed=ed champions=champions />
                    <section aria-labelledby="plan-notes-heading" class="flex flex-col gap-3">
                        <SectionHeader title="Notes" id="plan-notes-heading" />
                        <Field id="plan-notes" label="Anything else the team should know">
                            <TextField value=ed.notes rows=3 placeholder="Swap lanes if they pick Kalista." />
                        </Field>
                    </section>
                    <ChecklistSection ed=ed />
                    <HistorySection ed=ed drafts=drafts win_rates=win_rates />
                    <ReviewSection ed=ed />
                </div>
            </div>
        </PageShell>
    }
}

/// The plan's own steps: draft, strategy, roles, checklist, review.
#[component]
fn PlanSteps(ed: PlanEd) -> impl IntoView {
    view! {
        <section aria-label="Plan steps">
            {move || {
                let has_draft = !ed.draft_id.get().is_empty();
                let has_picks = ed.ours.iter().any(|s| !s.get().is_empty());
                let has_strategy = !ed.win.get().trim().is_empty() || !ed.teamfight.get().trim().is_empty();
                let has_roles = ed.roles.iter().filter(|s| !s.get().trim().is_empty()).count();
                let saved = ed.id.get().is_some();
                let steps = [
                    ("Link the draft", has_draft || has_picks, if has_draft { "Linked; the picks come from it" } else if has_picks { "Picks entered by hand" } else { "Or enter the picks yourself" }.to_string(), "#plan-basics-heading"),
                    ("Write how you win", has_strategy, if has_strategy { "Win conditions written".to_string() } else { "Win conditions, objectives, fights".to_string() }, "#plan-strategy-heading"),
                    ("Brief each role", has_roles == 5, format!("{has_roles} of 5 roles briefed"), "#plan-roles-heading"),
                    ("Save the plan", saved, if saved { "Saved; the checklist below is for game day".to_string() } else { "Saving keeps it for the team and allows a checklist".to_string() }, "#plan-checklist-heading"),
                    ("Review the game", false, "After the game, start a review linked to this plan".to_string(), "#plan-review-heading"),
                ];
                let next = steps.iter().position(|s| !s.1);
                let items: Vec<PrepStep> = steps
                    .iter()
                    .enumerate()
                    .map(|(i, (t, done, detail, href))| {
                        let state = if *done { PrepState::Done } else if Some(i) == next { PrepState::Next } else { PrepState::Todo };
                        let step = PrepStep::new(*t, state).detail(detail.clone());
                        if state == PrepState::Next { step.link(*href, "Go there") } else { step }
                    })
                    .collect();
                view! { <PrepSequence label="Plan steps" steps=items /> }
            }}
        </section>
    }
}

#[component]
fn BasicsSection(
    ed: PlanEd,
    drafts: Resource<Result<Vec<Draft>, ServerFnError>>,
    name_error: RwSignal<Option<String>>,
) -> impl IntoView {
    // Choosing a draft fills the picks from it.
    let chosen = RwSignal::new(String::new());
    Effect::new(move |_| {
        let v = ed.draft_id.get();
        if chosen.get_untracked() != v {
            chosen.set(v);
        }
    });
    Effect::new(move |prev: Option<()>| {
        let v = chosen.get();
        if prev.is_none() || v == ed.draft_id.get_untracked() {
            return;
        }
        if v.is_empty() {
            ed.draft_id.set(String::new());
            ed.from_draft.set(false);
            return;
        }
        if let Some(Ok(list)) = drafts.get_untracked() {
            if let Some(d) = list.iter().find(|d| d.id.as_deref() == Some(v.as_str())) {
                ed.apply_draft(d, false);
            }
        }
    });
    view! {
        <section aria-labelledby="plan-basics-heading" class="flex flex-col gap-4">
            <SectionHeader title="Plan" id="plan-basics-heading" />
            <div class="grid grid-cols-1 gap-x-8 gap-y-4 md:grid-cols-2">
                <Field id="plan-name" label="Plan name" required=true error=Signal::derive(move || name_error.get())>
                    <TextField value=ed.name placeholder="Front-to-back vs Riverside" maxlength=120 />
                </Field>
                <div class="flex flex-col gap-1.5">
                    <Suspense fallback=|| view! { <Skeleton shape=SkeletonShape::Block size="h-9 w-full" /> }>
                        {move || drafts.get().map(|r| {
                            let mut opts = vec![SelectOption::new("", "No draft")];
                            opts.extend(r.unwrap_or_default().into_iter().map(|d| SelectOption::new(d.id.unwrap_or_default(), d.name)));
                            view! {
                                <Field id="plan-draft" label="Draft" hint="Choosing a draft fills both teams' picks.">
                                    <Select value=chosen options=opts />
                                </Field>
                            }
                        })}
                    </Suspense>
                    {move || {
                        let d = ed.draft_id.get();
                        (!d.is_empty()).then(|| view! {
                            <div><Button variant=ButtonVariant::Quiet size=ButtonSize::Sm href=format!("/draft?draft_id={d}")>"Open the draft"</Button></div>
                        })
                    }}
                </div>
            </div>
        </section>
    }
}

/// Five picks per side, read-only when they came from the draft.
#[component]
fn MatchupSection(
    ed: PlanEd,
    champions: Resource<Result<Vec<Champion>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <section aria-labelledby="plan-matchup-heading" class="flex flex-col gap-4">
            <SectionHeader
                title="Matchup"
                id="plan-matchup-heading"
                actions=move || view! {
                    {move || ed.from_draft.get().then(|| view! {
                        <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| ed.from_draft.set(false))>"Edit picks"</Button>
                    })}
                }
            />
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading champions"><Skeleton shape=SkeletonShape::Line lines=5 /></SkeletonGroup> }>
                {move || champions.get().map(|r| {
                    let champs = StoredValue::new(r.unwrap_or_default());
                    let side = move |label: &'static str, sigs: [RwSignal<String>; 5], key: &'static str| view! {
                        <div class="flex min-w-0 flex-col gap-2">
                            <h3 class="font-display text-heading font-semibold text-primary">{label}</h3>
                            <ul class="flex flex-col border-t border-rule">
                                {Role::ALL.iter().enumerate().map(|(i, role)| {
                                    let sig = sigs[i];
                                    let id = format!("plan-{key}-{}", role.label().to_lowercase());
                                    let aria = format!("{label}: {}", role.label());
                                    view! {
                                        <li class="grid grid-cols-[3rem_minmax(0,1fr)] items-center gap-2 border-b border-rule py-1.5">
                                            <RoleLabel role=*role />
                                            {move || if ed.from_draft.get() {
                                                let v = sig.get();
                                                let name = champs.with_value(|l| l.iter().find(|c| c.id == v || c.name == v).map(|c| c.name.clone())).unwrap_or_else(|| v.clone());
                                                if v.is_empty() {
                                                    view! { <span class="text-label text-muted">"\u{2014}"</span> }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="flex min-w-0 items-center gap-2">
                                                            <ChampionAvatar name=name.clone() size=AvatarSize::Sm decorative=true />
                                                            <span class="truncate text-label font-medium text-primary">{name}</span>
                                                        </span>
                                                    }.into_any()
                                                }
                                            } else {
                                                view! {
                                                    <ChampionAutocomplete champions=champs.get_value() value=sig placeholder="Champion" id=id.clone() aria_label=aria.clone() />
                                                }.into_any()
                                            }}
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        </div>
                    };
                    view! {
                        <div class="grid grid-cols-1 gap-x-10 gap-y-6 md:grid-cols-2">
                            {side("Our team", ed.ours, "ours")}
                            {side("Their team", ed.theirs, "theirs")}
                        </div>
                    }
                })}
            </Suspense>
        </section>
    }
}

/// Win rates per strategy tag: (tag, games, wins).
type WinRates = Resource<Result<Vec<(String, i32, i32)>, ServerFnError>>;

#[component]
fn StrategySection(ed: PlanEd, win_rates: WinRates) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let outline = Callback::new(move |_: leptos::ev::MouseEvent| {
        let ours: Vec<String> = ed.ours.iter().map(|s| s.get_untracked()).collect();
        let theirs: Vec<String> = ed.theirs.iter().map(|s| s.get_untracked()).collect();
        let (wc, obj, tf, eg) = generate_template(&ours, &theirs);
        let fill = |sig: RwSignal<String>, v: String| {
            if sig.get_untracked().trim().is_empty() {
                sig.set(v);
            }
        };
        fill(ed.win, wc.join("\n"));
        fill(ed.objectives, obj.join("\n"));
        fill(ed.teamfight, tf);
        fill(ed.early, eg);
        toast.push.run(Toast::info(
            "Starter outline added to the empty fields; rewrite it for your team.",
        ));
    });
    view! {
        <section aria-labelledby="plan-strategy-heading" class="flex flex-col gap-4">
            <SectionHeader
                title="How we win"
                id="plan-strategy-heading"
                actions=move || view! { <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=outline>"Starter outline"</Button> }
            />
            <div class="flex flex-col gap-1.5">
                <span class="text-label font-medium text-secondary" id="plan-tag-label">"Strategy"</span>
                <div role="group" aria-labelledby="plan-tag-label" class="flex flex-wrap gap-1">
                    {STRATEGY_TAGS.iter().map(|&t| view! {
                        <Button
                            size=ButtonSize::Sm
                            pressed=Signal::derive(move || ed.tag.get() == t)
                            on_click=Callback::new(move |_| ed.tag.update(|cur| { *cur = if cur == t { String::new() } else { t.to_string() }; }))
                        >{t}</Button>
                    }).collect_view()}
                </div>
                <Suspense fallback=|| ()>
                    {move || {
                        let tag = ed.tag.get();
                        (!tag.is_empty()).then(|| win_rates.get().map(|r| {
                            let found = r.unwrap_or_default().into_iter().find(|(t, _, _)| *t == tag);
                            let text = match found {
                                Some((_, g, w)) if g > 0 => format!("Your team is {w}-{} with this strategy in reviewed games.", g - w),
                                _ => "No reviewed games with this strategy yet.".to_string(),
                            };
                            view! { <p class="text-caption text-muted">{text}</p> }
                        }))
                    }}
                </Suspense>
            </div>
            <div class="grid grid-cols-1 gap-x-8 gap-y-4 md:grid-cols-2">
                <Field id="plan-win" label="Win conditions" hint="One per line.">
                    <TextField value=ed.win rows=4 placeholder="Force fights at the second dragon" />
                </Field>
                <Field id="plan-objectives" label="Objective priority" hint="One per line, most important first.">
                    <TextField value=ed.objectives rows=4 placeholder="Dragon" />
                </Field>
                <Field id="plan-teamfight" label="Teamfights">
                    <TextField value=ed.teamfight rows=3 placeholder="Front-to-back: protect Jinx, Ornn zones." />
                </Field>
                <Field id="plan-early" label="Early game">
                    <TextField value=ed.early rows=3 placeholder="Contest level 1 vision; path towards bot." />
                </Field>
            </div>
        </section>
    }
}

#[component]
fn RolesSection(
    ed: PlanEd,
    champions: Resource<Result<Vec<Champion>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <section aria-labelledby="plan-roles-heading" class="flex flex-col gap-4">
            <SectionHeader title="Roles" id="plan-roles-heading" />
            <Suspense fallback=|| ()>
                {move || {
                    let champs = champions.get().and_then(|r| r.ok()).unwrap_or_default();
                    let champs = StoredValue::new(champs);
                    view! {
                        <ul class="flex flex-col border-t border-rule">
                            {Role::ALL.iter().enumerate().map(move |(i, role)| {
                                let (ours, theirs, note) = (ed.ours[i], ed.theirs[i], ed.roles[i]);
                                let id = format!("plan-role-{}", role.label().to_lowercase());
                                let name = move |v: String| champs.with_value(|l| l.iter().find(|c| c.id == v || c.name == v).map(|c| c.name.clone())).unwrap_or(v);
                                view! {
                                    <li class="grid grid-cols-1 gap-2 border-b border-rule py-3 sm:grid-cols-[10rem_minmax(0,1fr)]">
                                        <div class="flex flex-col gap-0.5">
                                            <RoleLabel role=*role />
                                            <span class="text-caption text-muted">
                                                {move || {
                                                    let (o, t) = (ours.get(), theirs.get());
                                                    match (o.is_empty(), t.is_empty()) {
                                                        (false, false) => format!("{} vs {}", name(o), name(t)),
                                                        (false, true) => name(o),
                                                        _ => String::new(),
                                                    }
                                                }}
                                            </span>
                                        </div>
                                        <TextField id=id value=note rows=2 aria_label=format!("{} brief", role.label()) placeholder="What this player plays for, and when." />
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    }
                }}
            </Suspense>
        </section>
    }
}

/// The pre-game checklist: ticks save at once.
#[component]
fn ChecklistSection(ed: PlanEd) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let items = RwSignal::new(Vec::<String>::new());
    let checked = RwSignal::new(Vec::<bool>::new());
    let instance = RwSignal::new(None::<String>);
    let new_item = RwSignal::new(String::new());
    let templates = Resource::new(|| (), |_| get_checklist_templates());

    Effect::new(move |_| {
        let id = ed.id.get();
        match id {
            Some(pid) => leptos::task::spawn_local(async move {
                let found = get_plan_checklist(pid).await.ok().flatten();
                instance.set(found.as_ref().and_then(|i| i.id.clone()));
                items.set(found.as_ref().map(|i| i.items.clone()).unwrap_or_default());
                checked.set(found.map(|i| i.checked).unwrap_or_default());
            }),
            None => {
                instance.set(None);
                items.set(Vec::new());
                checked.set(Vec::new());
            }
        }
    });

    let persist_checks = move || {
        if let Some(iid) = instance.get_untracked() {
            let json = serde_json::to_string(&checked.get_untracked()).unwrap_or_default();
            leptos::task::spawn_local(async move {
                if let Err(e) = update_checklist(iid, json).await {
                    toast
                        .push
                        .run(Toast::error(format!("Could not save the tick: {e}")));
                }
            });
        }
    };
    let add = Callback::new(move |_: ()| {
        let t = new_item.get_untracked().trim().to_string();
        if t.is_empty() {
            return;
        }
        items.update(|v| v.push(t));
        checked.update(|c| c.push(false));
        new_item.set(String::new());
    });
    let save = Callback::new(move |_: leptos::ev::MouseEvent| {
        let Some(pid) = ed.id.get_untracked() else {
            toast.push.run(Toast::error(
                "Save the plan first; the checklist belongs to it.",
            ));
            return;
        };
        let list = items.get_untracked();
        if list.is_empty() {
            toast.push.run(Toast::error("Add at least one item."));
            return;
        }
        let json = serde_json::to_string(&list).unwrap_or_default();
        leptos::task::spawn_local(async move {
            // A changed list is a new instance; ticks of an unchanged one update.
            match create_plan_checklist(pid, None, json).await {
                Ok(id) => {
                    instance.set(Some(id));
                    toast.push.run(Toast::success("Checklist saved"));
                }
                Err(e) => toast.push.run(Toast::error(format!("Could not save: {e}"))),
            }
        });
    });
    let save_template = Callback::new(move |_: leptos::ev::MouseEvent| {
        let list = items.get_untracked();
        if list.is_empty() {
            return;
        }
        let json = serde_json::to_string(&list).unwrap_or_default();
        leptos::task::spawn_local(async move {
            match save_checklist_template("Pre-game checklist".to_string(), json).await {
                Ok(_) => {
                    templates.refetch();
                    toast.push.run(Toast::success("Saved as a template"));
                }
                Err(e) => toast.push.run(Toast::error(format!("Could not save: {e}"))),
            }
        });
    });

    view! {
        <section aria-labelledby="plan-checklist-heading" class="flex flex-col gap-4">
            <SectionHeader
                title="Pre-game checklist"
                id="plan-checklist-heading"
            />
            <p class="text-label text-secondary" aria-live="polite" data-testid="checklist-progress">
                {move || {
                    let total = items.get().len();
                    let done = checked.get().iter().filter(|c| **c).count();
                    if total == 0 { "No items yet. Add what the team checks before the game, or start from a template.".to_string() } else { format!("{done} of {total} done") }
                }}
            </p>
            <ul class="flex flex-col">
                {move || items.get().into_iter().enumerate().map(|(i, text)| {
                    let state = RwSignal::new(checked.with_untracked(|c| c.get(i).copied().unwrap_or(false)));
                    view! {
                        <li class="flex items-center justify-between gap-3 border-b border-rule py-1.5 last:border-b-0">
                            <Checkbox
                                label=text
                                checked=state
                                strike=true
                                on_change=Callback::new(move |v: bool| {
                                    checked.update(|c| {
                                        if c.len() <= i { c.resize(i + 1, false); }
                                        c[i] = v;
                                    });
                                    persist_checks();
                                })
                            />
                            <Button variant=ButtonVariant::Quiet size=ButtonSize::Sm on_click=Callback::new(move |_| {
                                items.update(|v| { if i < v.len() { v.remove(i); } });
                                checked.update(|c| { if i < c.len() { c.remove(i); } });
                            })>"Remove"</Button>
                        </li>
                    }
                }).collect_view()}
            </ul>
            <div class="flex flex-col gap-2 sm:flex-row sm:items-end">
                <div class="min-w-0 flex-1">
                    <Field id="checklist-new" label="New item">
                        <TextField
                            value=new_item
                            placeholder="Check summoner spells"
                            on_keydown=Callback::new(move |ev: leptos::ev::KeyboardEvent| {
                                if ev.key() == "Enter" { ev.prevent_default(); add.run(()); }
                            })
                        />
                    </Field>
                </div>
                <Button on_click=Callback::new(move |_| add.run(()))>"Add item"</Button>
            </div>
            <Suspense fallback=|| ()>
                {move || templates.get().map(|r| {
                    let list = r.unwrap_or_default();
                    (!list.is_empty()).then(|| view! {
                        <div class="flex flex-wrap items-center gap-2">
                            <span class="text-label text-muted">"Templates:"</span>
                            {list.into_iter().map(|t| {
                                let t2 = t.clone();
                                let tid = t.id.clone().unwrap_or_default();
                                let n = t.items.len();
                                view! {
                                    <span class="inline-flex items-center">
                                        <Button size=ButtonSize::Sm on_click=Callback::new(move |_| {
                                            items.set(t2.items.clone());
                                            checked.set(vec![false; t2.items.len()]);
                                            instance.set(None);
                                            toast.push.run(Toast::info(format!("Loaded \u{201c}{}\u{201d}; save the checklist to keep it.", t2.name)));
                                        })>{format!("{} ({n})", t.name)}</Button>
                                        <Button variant=ButtonVariant::Quiet size=ButtonSize::Sm aria_label=format!("Delete template {}", t.name) on_click=Callback::new(move |_| {
                                            let id = tid.clone();
                                            leptos::task::spawn_local(async move {
                                                let _ = delete_checklist_template_fn(id).await;
                                                templates.refetch();
                                            });
                                        })>"Delete"</Button>
                                    </span>
                                }
                            }).collect_view()}
                        </div>
                    })
                })}
            </Suspense>
            <div class="flex flex-wrap gap-2">
                <Button on_click=save>"Save checklist"</Button>
                <Button variant=ButtonVariant::Quiet on_click=save_template>"Save as template"</Button>
            </div>
        </section>
    }
}

/// Win rate per strategy tag, all time or against the linked opponent.
#[component]
fn HistorySection(
    ed: PlanEd,
    drafts: Resource<Result<Vec<Draft>, ServerFnError>>,
    win_rates: WinRates,
) -> impl IntoView {
    let scope = RwSignal::new("all".to_string());
    let opponent = Memo::new(move |_| {
        let id = ed.draft_id.get();
        drafts
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
            .into_iter()
            .find(|d| d.id.as_deref() == Some(id.as_str()))
            .and_then(|d| d.opponent)
            .filter(|o| !o.is_empty())
    });
    let vs = Resource::new(
        move || (scope.get(), opponent.get()),
        |(s, o)| async move {
            match (s.as_str(), o) {
                ("vs", Some(o)) => get_strategy_win_rates_vs_opponent(o).await,
                _ => Ok(Vec::new()),
            }
        },
    );
    view! {
        <section aria-labelledby="plan-history-heading" class="flex flex-col gap-4">
            <SectionHeader
                title="Strategy record"
                id="plan-history-heading"
                actions=move || view! {
                    <SegmentedControl
                        id="plan-history-scope"
                        label="Games counted"
                        options=vec![Segment::new("all", "All games"), Segment::new("vs", "Against this opponent")]
                        value=scope
                        size=SegmentSize::Sm
                    />
                }
            />
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading the record"><Skeleton shape=SkeletonShape::Line lines=3 /></SkeletonGroup> }>
                {move || {
                    let rows = if scope.get() == "vs" {
                        if opponent.get().is_none() {
                            return Some(view! {
                                <EmptyState size=EmptySize::Compact message="Link a draft with an opponent to see your record against them." />
                            }.into_any());
                        }
                        vs.get()
                    } else {
                        win_rates.get()
                    };
                    let current = ed.tag.get();
                    rows.map(|r| {
                        let rows = r.unwrap_or_default();
                        if rows.is_empty() {
                            return view! {
                                <EmptyState size=EmptySize::Compact message="Your record per strategy appears once post-game reviews are linked to tagged plans." />
                            }.into_any();
                        }
                        view! {
                            <Table caption="Win rate by strategy" caption_hidden=true>
                                <thead><tr><Th>"Strategy"</Th><Th numeric=true>"Games"</Th><Th numeric=true>"Won"</Th><Th numeric=true>"Win rate"</Th></tr></thead>
                                <tbody>
                                    {rows.into_iter().map(|(tag, games, wins)| {
                                        let pct = if games > 0 { wins * 100 / games } else { 0 };
                                        view! {
                                            <Tr selected=tag == current>
                                                <Td header=true>{tag}</Td>
                                                <Td numeric=true>{games}</Td>
                                                <Td numeric=true>{wins}</Td>
                                                <Td numeric=true>{format!("{pct}%")}</Td>
                                            </Tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </Table>
                        }.into_any()
                    })
                }}
            </Suspense>
        </section>
    }
}

/// After the game: a review linked to this plan and its draft.
#[component]
fn ReviewSection(ed: PlanEd) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let busy = RwSignal::new(false);
    let start = Callback::new(move |_: leptos::ev::MouseEvent| {
        let Some(pid) = ed.id.get_untracked() else {
            return;
        };
        let did = opt(ed.draft_id.get_untracked());
        busy.set(true);
        leptos::task::spawn_local(async move {
            match start_post_game_review(pid, did).await {
                #[allow(unused_variables)]
                Ok(review_id) =>
                {
                    #[cfg(feature = "hydrate")]
                    if let Some(window) = web_sys::window() {
                        let _ = window
                            .location()
                            .set_href(&format!("/post-game?review_id={review_id}"));
                    }
                }
                Err(e) => {
                    busy.set(false);
                    toast
                        .push
                        .run(Toast::error(format!("Could not start the review: {e}")));
                }
            }
        });
    });
    view! {
        <section aria-labelledby="plan-review-heading" class="flex flex-col gap-3">
            <SectionHeader title="After the game" id="plan-review-heading" />
            {move || if ed.id.get().is_some() {
                view! {
                    <p class="text-body text-secondary">"Start a review that is already linked to this plan and its draft."</p>
                    <div><Button on_click=start loading=Signal::derive(move || busy.get())>"Start the review"</Button></div>
                }.into_any()
            } else {
                view! { <p class="text-body text-secondary">"Save the plan to start a review linked to it after the game."</p> }.into_any()
            }}
        </section>
    }
}
