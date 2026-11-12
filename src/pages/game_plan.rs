//! Game plans: the plan for one game (picks, how we win, role briefs,
//! notes, a pre-game checklist), linked to a draft and, after the game, to
//! a review.

use crate::components::champion_autocomplete::ChampionAutocomplete;
use crate::components::draft_board::slot_meta;
use crate::components::ui::a11y::focus_id;
use crate::components::ui::{
    save_failure_reason, use_save_state, use_unsaved_guard, LoadFailed, SaveState, SaveStatus,
    SAVE_FAILED_TOAST,
};
use crate::components::ui::{
    AvatarSize, Button, ButtonSize, ButtonVariant, ChampionAvatar, Checkbox, ConfirmDialog,
    ConfirmTone, Density, Dialog, DialogSize, EmptySize, EmptyState, Field, PageHeader, PageShell,
    PrepEmphasis, PrepSequence, PrepState, PrepStep, Role, RoleLabel, SectionHeader, Segment,
    SegmentSize, SegmentedControl, Select, SelectOption, Skeleton, SkeletonGroup, SkeletonShape,
    StrengthDots, SuggestionChip, TextField, Toast, ToastContext,
};
use crate::models::champion::Champion;
use crate::models::draft::{most_common_tag, Draft};
use crate::models::game_plan::{ChecklistInstance, ChecklistTemplate, GamePlan};
use crate::models::suggestion::{Suggestion, SuggestionTarget};
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
        // M159: a plan from a draft made for a match carries its stamp
        // onward when the link did not name the match.
        if plan.scheduled_match_id.is_none() {
            if let Some(d) = db::get_draft_for_prefill(&surreal, &team_id, did)
                .await
                .map_err(authz::server_err)?
            {
                if d.scheduled_match_id.is_some() {
                    plan.scheduled_match_id = d.scheduled_match_id;
                    plan.game_number = d.game_number;
                }
            }
        }
    }
    // The match stamp is re-checked in the DB layer as the team's match.
    plan.scheduled_match_deleted = false;
    plan.dismissed_suggestions =
        crate::models::suggestion::sanitize_dismissed(plan.dismissed_suggestions);
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
    let mut plan: GamePlan = serde_json::from_str(&plan_json).map_err(authz::server_err)?;
    if let Some(did) = plan.draft_id.as_deref().filter(|d| !d.is_empty()) {
        db::ensure_team_record(&surreal, "draft", did, &team_id)
            .await
            .map_err(authz::server_err)?;
    }
    plan.dismissed_suggestions =
        crate::models::suggestion::sanitize_dismissed(plan.dismissed_suggestions);
    db::update_game_plan(&surreal, &team_id, plan)
        .await
        .map_err(authz::server_err)
}

/// The most picks a side may send to [`plan_suggestions`].
pub const SUGGEST_MAX_PICKS: usize = 5;
/// The longest pick string [`plan_suggestions`] accepts.
pub const SUGGEST_MAX_PICK_CHARS: usize = 64;

/// M150: suggestions for the plan's matchup, from the trait rules only
/// (`server::graph::suggest`). It takes the picks in the editor, not a plan
/// id, so a plan that is not saved yet gets them too; nothing stored is
/// read (no plan, no games, no Riot call). Team members only. Empty unless
/// both sides have five known champions, and empty when the trait table is
/// not loaded.
#[server]
pub async fn plan_suggestions(
    ours: Vec<String>,
    theirs: Vec<String>,
) -> Result<Vec<Suggestion>, ServerFnError> {
    use crate::server::db::DbError;
    use crate::server::graph::{suggest, traits};
    use crate::server::{authz, rate_limit::Bucket};

    let ctx = authz::authed().await?;
    ctx.require_team().await?;
    authz::rate_limit(Bucket::Graph).await?;
    let too_big = |side: &[String]| {
        side.len() > SUGGEST_MAX_PICKS
            || side
                .iter()
                .any(|p| p.chars().count() > SUGGEST_MAX_PICK_CHARS)
    };
    if too_big(&ours) || too_big(&theirs) {
        return Err(authz::server_err(DbError::Invalid(
            "A side has at most five picks.".into(),
        )));
    }
    let Some(table) = traits::table() else {
        return Ok(Vec::new());
    };
    Ok(suggest::suggest(&table, &ours, &theirs))
}

/// M150: a suggestion was added to a plan ("accept") or dismissed. Only
/// the rule id and the action are logged, one line each, to learn which
/// rules coaches trust; no user, team or plan id, and nothing is stored.
#[server]
pub async fn suggestion_feedback(rule_id: String, action: String) -> Result<(), ServerFnError> {
    use crate::server::db::DbError;
    use crate::server::graph::suggest;
    use crate::server::{authz, rate_limit::Bucket};

    let ctx = authz::authed().await?;
    ctx.require_team().await?;
    // Per user, like votes and dismissals elsewhere (30, then one every 2 s).
    authz::rate_limit_user(Bucket::Community, &ctx.user)?;
    if !matches!(action.as_str(), "accept" | "dismiss") || !suggest::is_known_rule_id(&rule_id) {
        return Err(authz::server_err(DbError::Invalid(
            "Unknown suggestion.".into(),
        )));
    }
    tracing::info!(rule_id = %rule_id, action = %action, "plan suggestion");
    Ok(())
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

    // M159: the review carries the plan's match stamp (and game) onward.
    let stamp = db::get_game_plan(&surreal, &team_id, &plan_id)
        .await
        .map_err(authz::server_err)?
        .filter(|p| p.scheduled_match_id.is_some())
        .map(|p| (p.scheduled_match_id, p.game_number));
    let (scheduled_match_id, game_number) = stamp.unwrap_or((None, None));
    let review = PostGameLearning {
        scheduled_match_id,
        series_id: None,
        game_number,
        scheduled_match_deleted: false,
        draft_deleted: false,
        created_at: None,
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
    /// The draft this plan was linked to was deleted.
    draft_deleted: RwSignal<bool>,
    /// M159: the scheduled match this plan was made for was deleted.
    match_deleted: RwSignal<bool>,
    /// M150: suggestion keys dismissed on this plan ("Not for us").
    dismissed: RwSignal<Vec<String>>,
    /// M150: bumped whenever another plan (or a blank one) is loaded, so
    /// the suggestions collapse again; a first save does not bump it.
    loads: RwSignal<u32>,
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
            draft_deleted: RwSignal::new(false),
            match_deleted: RwSignal::new(false),
            dismissed: RwSignal::new(Vec::new()),
            loads: RwSignal::new(0),
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
        self.draft_deleted.set(false);
        self.match_deleted.set(false);
        self.dismissed.set(Vec::new());
        self.loads.update(|g| *g = g.wrapping_add(1));
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
        self.draft_deleted.set(p.draft_deleted);
        self.match_deleted.set(p.scheduled_match_deleted);
        self.dismissed.set(p.dismissed_suggestions.clone());
        self.loads.update(|g| *g = g.wrapping_add(1));
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
            scheduled_match_id: None,
            game_number: None,
            scheduled_match_deleted: false,
            draft_deleted: false,
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
            dismissed_suggestions: if tracked {
                self.dismissed.get()
            } else {
                self.dismissed.get_untracked()
            },
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
            crate::pages::navigate::redirect_to_login();
        }
    });

    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let query = leptos_router::hooks::use_query_map();
    let plans = Resource::new(|| (), |_| list_plans());
    let drafts = Resource::new(|| (), |_| list_team_drafts());
    let champions = Resource::new(|| (), |_| get_champions_for_game_plan());
    let win_rates = Resource::new(|| (), |_| get_strategy_win_rates());
    let ed = PlanEd::new();
    let echo = Echo::new();
    // The header's save state (`ui::status`, DESIGN 6.6).
    let save = use_save_state();
    save.show(SaveState::NotSaved);
    // An empty editor, to tell a new plan with content from a blank one.
    let blank_plan = StoredValue::new(ed.build(false));

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
    // A plan opened from `?draft_id=` can be saved once that draft has
    // loaded, so a quick save never drops the link to it.
    let prefill_pending = RwSignal::new(
        query
            .read_untracked()
            .get("draft_id")
            .is_some_and(|d| !d.is_empty()),
    );
    Effect::new(move |_| {
        let result = prefill.get();
        if result.is_some() {
            prefill_pending.set(false);
        }
        if prefilled.get_value() {
            return;
        }
        if let Some(Ok(Some(d))) = result {
            // Start from a clean editor unless this is already a new plan
            // (then keep what the user typed while the draft loaded).
            if ed.id.get_untracked().is_some() {
                ed.clear();
            }
            ed.apply_draft(&d, true);
            prefilled.set_value(true);
        }
    });
    // M159: `?match_id=&game=` (from the match thread) stamps a new plan
    // with that match on its first save. The match is resolved for the
    // team on the server; an unknown or foreign id is dropped with a line.
    use crate::models::match_day::ThreadLookup;
    let thread_res = Resource::new(
        move || {
            query
                .read()
                .get("match_id")
                .filter(|m| !m.trim().is_empty())
        },
        |id| async move {
            match id {
                Some(id) => match crate::pages::home::get_thread_match(id).await {
                    Ok(t) => ThreadLookup::Found(t),
                    Err(_) => ThreadLookup::Unknown,
                },
                None => ThreadLookup::NoLink,
            }
        },
    );
    // (match id, game) for a new plan.
    let thread = RwSignal::new(None::<(String, Option<i32>)>);
    let thread_unknown = RwSignal::new(false);
    Effect::new(move |_| {
        let game = query.read().get("game");
        match thread_res.get() {
            Some(ThreadLookup::Found(t)) => {
                let g = crate::models::match_day::parse_game(game.as_deref(), &t.format);
                thread.set(Some((t.id, g)));
                thread_unknown.set(false);
            }
            Some(ThreadLookup::Unknown) => {
                thread.set(None);
                thread_unknown.set(true);
            }
            _ => {}
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
                save.saved_steady();
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
        // While saves fail, "Could not save" stays until one works (the
        // next autosave below is the retry an edit gives).
        if !save.is_failing() {
            save.show(SaveState::Unsaved);
        }
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
                    let current = edit_gen.get_value() == gen;
                    if current {
                        save.saving();
                    }
                    match update_plan(json).await {
                        Ok(_) => {
                            // Only the plan still open takes the new snapshot.
                            if edit_gen.get_value() == gen {
                                let unchanged = ed.build(false) == plan;
                                loaded_snapshot.set_value(Some(plan));
                                save.saved_steady();
                                if !unchanged {
                                    save.show(SaveState::Unsaved);
                                }
                            }
                            plans.refetch();
                            win_rates.refetch();
                        }
                        Err(e) => {
                            let reason = save_failure_reason(&e);
                            if edit_gen.get_value() == gen {
                                if save.failed(reason) {
                                    toast.push.run(Toast::error(SAVE_FAILED_TOAST));
                                }
                            } else {
                                // The editor moved on: say so once, in words.
                                toast.push.run(Toast::error(format!(
                                    "Your last changes to the plan you left are not saved: {reason}"
                                )));
                            }
                        }
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
        save.saved_steady();
        edit_gen.update_value(|g| *g += 1);
        opened.set_value(true);
        if let Some(id) = p.id.as_deref() {
            replace_url(nav, &format!("/game-plan?plan_id={id}"));
        }
    });

    let name_error = RwSignal::new(None::<String>);
    // "Save plan" and the header's "Retry" after a failed save.
    let save_now = Callback::new(move |_: ()| {
        if saving.get_untracked() || prefill_pending.get_untracked() {
            return;
        }
        let plan = ed.build(false);
        if plan.name.trim().is_empty() {
            name_error.set(Some("Name the plan so the team can find it.".into()));
            focus_id("plan-name");
            return;
        }
        name_error.set(None);
        let is_update = plan.id.is_some();
        // M159: a new plan takes the match thread's stamp (the editor's own
        // snapshot stays without it, so it compares as before).
        let mut to_send = plan.clone();
        if !is_update {
            if let Some((m, g)) = thread.get_untracked() {
                to_send.scheduled_match_id = Some(m);
                to_send.game_number = g;
            }
        }
        let json = serde_json::to_string(&to_send).unwrap_or_default();
        let gen = edit_gen.get_value();
        saving.set(true);
        save.saving();
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
                    save.saved_steady();
                    if !unchanged {
                        save.show(SaveState::Unsaved);
                    }
                }
                Err(e) => {
                    let reason = save_failure_reason(&e);
                    if edit_gen.get_value() != gen {
                        toast.push.run(Toast::error(format!(
                            "Your last changes to the plan you left are not saved: {reason}"
                        )));
                    } else if save.failed(reason) {
                        toast.push.run(Toast::error(SAVE_FAILED_TOAST));
                    }
                }
            }
        });
    });
    let do_save = Callback::new(move |_: leptos::ev::MouseEvent| save_now.run(()));
    // Leaving with work that is not saved asks first: an edit waiting for
    // its autosave, a save on its way or failed, or a new plan with content.
    use_unsaved_guard(Signal::derive(move || {
        let state = save.state.get();
        state.guards_leave()
            || (state == SaveState::NotSaved && blank_plan.with_value(|b| ed.build(true) != *b))
    }));
    let new_plan = Callback::new(move |_: leptos::ev::MouseEvent| {
        opened.set_value(true);
        replace_url(nav, "/game-plan");
        thread.set(None);
        thread_unknown.set(false);
        ed.clear();
        edit_gen.update_value(|g| *g += 1);
        loaded_snapshot.set_value(None);
        save.show(SaveState::NotSaved);
        name_error.set(None);
    });
    let confirm_delete = RwSignal::new(false);
    let share_open = RwSignal::new(false);
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
                    save.show(SaveState::NotSaved);
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
                    <SaveStatus state=save.state on_retry=save_now attr:data-testid="plan-save-state" />
                }
                actions=move || view! {
                    <Button variant=ButtonVariant::Quiet on_click=new_plan>"New plan"</Button>
                    {move || ed.id.get().map(|_| view! {
                        <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| share_open.set(true)) attr:data-testid="plan-share">"Share"</Button>
                        <Button variant=ButtonVariant::Danger on_click=Callback::new(move |_| confirm_delete.set(true))>"Delete"</Button>
                    })}
                    <Button variant=ButtonVariant::Primary size=ButtonSize::Touch on_click=do_save loading=Signal::derive(move || saving.get() || prefill_pending.get()) attr:data-testid="plan-save">"Save plan"</Button>
                }
            />
            <crate::pages::share::ShareDialog open=share_open kind=crate::models::share::ShareKind::GamePlan target=Signal::derive(move || ed.id.get()) />
            // M159: a new plan from the match thread says which match it is
            // for; a link whose match is not the team's says so.
            {move || (ed.id.get().is_none() && thread.get().is_some()).then(|| view! {
                <p class="-mt-3 mb-6 text-caption text-muted" data-testid="plan-thread">"Saving links this plan to the match it was opened from."</p>
            })}
            {move || (ed.id.get().is_none() && thread_unknown.get()).then(|| view! {
                <p class="-mt-3 mb-6 text-caption text-muted" data-testid="plan-thread-unknown">"The link named a match that is not on your team's schedule. The plan is saved without it."</p>
            })}
            {move || ed.id.get().map(|_| view! {
                <crate::components::guidance::FirstUseHint hint=crate::models::guidance::Hint::ShareLinks title="Share a read-only link" class="mb-6">
                    "Share makes a link to this plan that opens without signing in, for a coach or a sub outside the team. Choose how long it lives; revoke it any time."
                </crate::components::guidance::FirstUseHint>
            })}
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
                            Err(_) => view! { <LoadFailed what="The plans" on_retry=Callback::new(move |_| plans.refetch()) attr:data-testid="plans-load-failed" /> }.into_any(),
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
                    <div class="flex flex-col gap-4">
                        <MatchupSection ed=ed champions=champions />
                        <SuggestionsSection ed=ed echo=echo />
                    </div>
                    <StrategySection ed=ed win_rates=win_rates echo=echo />
                    <RolesSection ed=ed champions=champions />
                    <section aria-labelledby="plan-notes-heading" class="flex flex-col gap-3">
                        <SectionHeader title="Notes" id="plan-notes-heading" />
                        <Field id="plan-notes" label="Anything else the team should know">
                            <div class="relative">
                                <TextField value=ed.notes rows=3 placeholder="Swap lanes if they pick Kalista." />
                                <EchoLayer echo=echo target=SuggestionTarget::Notes />
                            </div>
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
                view! { <PrepSequence label="Plan steps" steps=items emphasis=PrepEmphasis::Quiet /> }
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
                        if d.is_empty() {
                            ed.draft_deleted.get().then(|| view! {
                                <p class="text-caption text-muted" data-testid="plan-draft-deleted">"Draft deleted: the draft this plan was linked to no longer exists. Choose another to link it again."</p>
                            }.into_any())
                        } else {
                            Some(view! {
                                <div><Button variant=ButtonVariant::Quiet size=ButtonSize::Sm href=format!("/draft?draft_id={d}")>"Open the draft"</Button></div>
                            }.into_any())
                        }
                    }}
                    {move || ed.match_deleted.get().then(|| view! {
                        <p class="text-caption text-muted" data-testid="plan-match-deleted">"The match this was made for was deleted."</p>
                    })}
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
                                                // Same geometry as the editable row (icon slot at
                                                // the left, the name where the input's text starts),
                                                // so "Edit picks" does not shift a line.
                                                if v.is_empty() {
                                                    view! {
                                                        <span class="flex h-9 min-w-0 items-center gap-2 pl-2">
                                                            <span class="block size-6 shrink-0 rounded-ctl border border-dashed border-rule" aria-hidden="true"></span>
                                                            <span class="text-label text-muted">"\u{2014}"</span>
                                                        </span>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="flex h-9 min-w-0 items-center gap-2 pl-2">
                                                            <ChampionAvatar name=name.clone() size=AvatarSize::Sm decorative=true />
                                                            <span class="truncate text-label font-medium text-primary">{name}</span>
                                                        </span>
                                                    }.into_any()
                                                }
                                            } else {
                                                view! {
                                                    <ChampionAutocomplete champions=champs.get_value() value=sig placeholder="Champion" id=id.clone() aria_label=aria.clone() icon=true />
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

// ---------------------------------------------------------------------------
// Suggestions (M150)
// ---------------------------------------------------------------------------

/// What the echo overlay draws over a field after "Add to plan".
#[derive(Clone, PartialEq)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct EchoData {
    gen: u32,
    target: SuggestionTarget,
    /// The field's text before the claim (drawn transparent, for layout).
    prefix: String,
    claim: String,
    /// The textarea's client box, in px.
    width: i32,
    height: i32,
}

/// The accept moment (card 4): the claim written in chalk over the new
/// line, dried to ink, then removed. Phase 0 hidden, 1 written (chalk),
/// 2 dried (ink).
#[derive(Clone, Copy)]
struct Echo {
    data: RwSignal<Option<EchoData>>,
    phase: RwSignal<u8>,
}

impl Echo {
    fn new() -> Self {
        Self {
            data: RwSignal::new(None),
            phase: RwSignal::new(0),
        }
    }
}

fn echo_dom_id(target: SuggestionTarget) -> &'static str {
    match target {
        SuggestionTarget::WinConditions => "plan-win-echo",
        SuggestionTarget::Notes => "plan-notes-echo",
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn field_dom_id(target: SuggestionTarget) -> &'static str {
    match target {
        SuggestionTarget::WinConditions => "plan-win",
        SuggestionTarget::Notes => "plan-notes",
    }
}

/// The overlay over one textarea (the page wraps the field in `relative`).
/// Decorative: the line itself is plain text in the field underneath.
#[component]
fn EchoLayer(echo: Echo, target: SuggestionTarget) -> impl IntoView {
    let phase_class = move || {
        match echo.phase.get() {
        0 => "block bg-ink-2 text-accent [clip-path:inset(0_100%_0_0)] transition-[clip-path,color] duration-[180ms] ease-out motion-reduce:transition-none",
        1 => "block bg-ink-2 text-accent [clip-path:inset(0_0_0_0)] transition-[clip-path,color] duration-[180ms] ease-out motion-reduce:transition-none",
        _ => "block bg-ink-2 text-primary [clip-path:inset(0_0_0_0)] transition-[clip-path,color] duration-[120ms] ease-out motion-reduce:transition-none",
    }
    };
    let shown = Memo::new(move |_| echo.data.get().filter(|d| d.target == target));
    view! {
        {move || shown.get().map(|d| {
            let style = format!("width:{}px;height:{}px", d.width, d.height);
            view! {
                <div
                    id=echo_dom_id(target)
                    aria-hidden="true"
                    data-testid="suggestion-echo"
                    class="pointer-events-none absolute left-px top-px overflow-hidden whitespace-pre-wrap break-words px-3 py-2 text-body leading-relaxed"
                    style=style
                >
                    <span class="text-transparent">{d.prefix}</span>
                    <span class="inline-block max-w-full bg-ink-2 align-top">
                        <span class=phase_class>{d.claim}</span>
                    </span>
                </div>
            }
        })}
    }
}

/// Whether the viewport is at least `md` wide (inline card) rather than a
/// phone (bottom sheet). The server and a missing window say desktop.
fn viewport_is_md() -> bool {
    #[cfg(feature = "hydrate")]
    {
        web_sys::window()
            .and_then(|w| w.match_media("(min-width: 48rem)").ok().flatten())
            .is_none_or(|m| m.matches())
    }
    #[cfg(not(feature = "hydrate"))]
    true
}

#[cfg(feature = "hydrate")]
fn reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .is_some_and(|m| m.matches())
}

/// Starts the echo over the field `target` once the new text is in the
/// DOM. Skipped when the field's bottom edge is off screen (the line is
/// written all the same and the live region speaks).
#[allow(unused_variables)]
fn run_echo(echo: Echo, target: SuggestionTarget, prefix: String, claim: String) {
    #[cfg(feature = "hydrate")]
    request_animation_frame(move || {
        use std::time::Duration;
        let Some(field) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(field_dom_id(target)))
        else {
            return;
        };
        // The textarea's own scroll only; the page never scrolls.
        field.set_scroll_top(field.scroll_height());
        let rect = field.get_bounding_client_rect();
        let inner_h = web_sys::window()
            .and_then(|w| w.inner_height().ok())
            .and_then(|h| h.as_f64())
            .unwrap_or(0.0);
        if rect.bottom() > inner_h || rect.bottom() < 0.0 {
            return;
        }
        let gen = echo
            .data
            .get_untracked()
            .map(|d| d.gen.wrapping_add(1))
            .unwrap_or(1);
        let reduced = reduced_motion();
        echo.phase.set(if reduced { 1 } else { 0 });
        echo.data.set(Some(EchoData {
            gen,
            target,
            prefix,
            claim,
            width: field.client_width(),
            height: field.client_height(),
        }));
        let scroll = field.scroll_top();
        let alive = move || echo.data.get_untracked().is_some_and(|d| d.gen == gen);
        request_animation_frame(move || {
            if let Some(el) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id(echo_dom_id(target)))
            {
                el.set_scroll_top(scroll);
            }
            if !reduced && alive() {
                // Frame 0 is painted hidden; now the write (180 ms).
                request_animation_frame(move || {
                    if alive() {
                        echo.phase.set(1);
                    }
                });
                set_timeout(
                    move || {
                        if alive() {
                            echo.phase.set(2);
                        }
                    },
                    Duration::from_millis(180),
                );
            }
            set_timeout(
                move || {
                    if alive() {
                        echo.data.set(None);
                    }
                },
                Duration::from_millis(300),
            );
        });
    });
}

/// How long a retiring chip fades before it leaves the row.
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
const RETIRE_MS: u64 = 170;

/// Up to three suggestions about the matchup, under the picks (M150).
/// Collapsed by default on every load; nothing opens or writes on its own.
#[component]
fn SuggestionsSection(ed: PlanEd, echo: Echo) -> impl IntoView {
    // The ten picks, settled 400 ms after the last change, so typing a
    // champion's name does not ask the server on every key.
    let picks = Memo::new(move |_| {
        let side = |sigs: &[RwSignal<String>; 5]| -> Vec<String> {
            sigs.iter().map(|s| s.get().trim().to_string()).collect()
        };
        (side(&ed.ours), side(&ed.theirs))
    });
    let settled = RwSignal::new(picks.get_untracked());
    #[allow(unused_variables)]
    let settle_timer: StoredValue<Option<leptos::prelude::TimeoutHandle>> = StoredValue::new(None);
    Effect::new(move |prev: Option<()>| {
        let now = picks.get();
        if prev.is_none() {
            settled.set(now);
            return;
        }
        #[cfg(feature = "hydrate")]
        {
            if let Some(h) = settle_timer.get_value() {
                h.clear();
            }
            let h = set_timeout_with_handle(
                move || settled.set(now),
                std::time::Duration::from_millis(400),
            )
            .ok();
            settle_timer.set_value(h);
        }
        #[cfg(not(feature = "hydrate"))]
        settled.set(now);
    });
    let full = Memo::new(move |_| {
        settled.with(|(o, t)| {
            o.iter().filter(|p| !p.is_empty()).count() == 5
                && t.iter().filter(|p| !p.is_empty()).count() == 5
        })
    });
    // Browser-only: keyed on the picks alone (never on the plan's text).
    let all = LocalResource::new(move || {
        let (ours, theirs) = settled.get();
        let ask = full.get();
        async move {
            if ask {
                plan_suggestions(ours, theirs).await
            } else {
                Ok(Vec::new())
            }
        }
    });

    let row_open = RwSignal::new(false);
    let open_key = RwSignal::new(None::<String>);
    // The card's content stays while it collapses.
    let card = RwSignal::new(None::<Suggestion>);
    // Inline card (md and up) or bottom sheet (phones), fixed when opened.
    let inline_mode = RwSignal::new(true);
    let sheet_open = RwSignal::new(false);
    // (place in the row, suggestion) fading out after Add or Dismiss.
    let retiring = RwSignal::new(None::<(usize, Suggestion)>);
    let live = RwSignal::new(String::new());

    // Another plan, or a new one: everything closed again.
    Effect::new(move |_| {
        let _ = ed.loads.get();
        row_open.set(false);
        open_key.set(None);
        sheet_open.set(false);
    });
    // Escape or the scrim closed the sheet.
    Effect::new(move |_| {
        if !sheet_open.get() && !inline_mode.get_untracked() {
            open_key.set(None);
        }
    });

    let visible_now = Memo::new(move |_| {
        let list = match all.get() {
            Some(Ok(l)) => l,
            _ => Vec::new(),
        };
        let dismissed = ed.dismissed.get();
        crate::models::suggestion::visible(&list, &dismissed, &ed.win.get(), &ed.notes.get())
    });
    // The row: the visible chips, with a retiring one kept in its place
    // until it has faded.
    let shown = Memo::new(move |_| {
        let mut v = visible_now.get();
        if let Some((i, s)) = retiring.get() {
            if !v.iter().any(|x| x.key == s.key) {
                v.insert(i.min(v.len()), s);
                v.truncate(crate::models::suggestion::MAX_SHOWN);
            }
        }
        v
    });
    #[derive(Clone, Copy, PartialEq)]
    enum Kind {
        Nothing,
        Wait,
        None,
        Chips,
    }
    let kind = Memo::new(move |_| {
        if !full.get() {
            return Kind::Wait;
        }
        match all.get() {
            None => Kind::Nothing,
            Some(Err(_)) => Kind::Nothing,
            Some(Ok(_)) if visible_now.get().is_empty() && retiring.get().is_none() => Kind::None,
            Some(Ok(_)) => Kind::Chips,
        }
    });
    let count_text = move || match visible_now.with(|v| v.len()) {
        1 => "1 suggestion".to_string(),
        n => format!("{n} suggestions"),
    };

    // After Add or Dismiss: the next chip, else the line, else the quiet
    // line (never the textarea, which would raise a phone's keyboard).
    let focus_next = move || {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
                return;
            };
            let el = doc
                .query_selector("[data-testid=suggestion-chip]:not([data-retiring])")
                .ok()
                .flatten()
                .or_else(|| doc.get_element_by_id("plan-suggestions-toggle"))
                .or_else(|| doc.get_element_by_id("plan-suggestions-none"));
            if let Some(el) = el.and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok()) {
                let _ = el.focus();
            }
        }
    };
    let retire = move |s: Suggestion| {
        let idx = shown.with_untracked(|v| v.iter().position(|x| x.key == s.key).unwrap_or(0));
        let key = s.key.clone();
        retiring.set(Some((idx, s)));
        open_key.set(None);
        sheet_open.set(false);
        #[cfg(feature = "hydrate")]
        set_timeout(
            move || {
                if retiring.with_untracked(|r| r.as_ref().is_some_and(|(_, x)| x.key == key)) {
                    retiring.set(None);
                }
                // After the row has re-rendered (effects run first).
                set_timeout(focus_next, std::time::Duration::ZERO);
            },
            std::time::Duration::from_millis(RETIRE_MS),
        );
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = key;
            retiring.set(None);
            focus_next();
        }
    };
    let feedback = |rule_id: String, action: &'static str| {
        leptos::task::spawn_local(async move {
            let _ = suggestion_feedback(rule_id, action.to_string()).await;
        });
    };
    let accept = Callback::new(move |s: Suggestion| {
        let (sig, label) = match s.target {
            SuggestionTarget::WinConditions => (ed.win, "Added to Win conditions."),
            SuggestionTarget::Notes => (ed.notes, "Added to Notes."),
        };
        let before = sig.get_untracked();
        let after = crate::models::suggestion::append_claim(&before, &s.claim);
        let prefix = after
            .strip_suffix(s.claim.trim())
            .unwrap_or_default()
            .to_string();
        sig.set(after);
        live.set(label.to_string());
        feedback(s.rule_id.clone(), "accept");
        run_echo(echo, s.target, prefix, s.claim.trim().to_string());
        retire(s);
    });
    let dismiss = Callback::new(move |s: Suggestion| {
        ed.dismissed.update(|d| {
            if !d.contains(&s.key) {
                d.push(s.key.clone());
            }
        });
        live.set("Dismissed for this plan.".to_string());
        feedback(s.rule_id.clone(), "dismiss");
        retire(s);
    });
    let open_chip = move |s: Suggestion| {
        if open_key.get_untracked().as_deref() == Some(s.key.as_str()) {
            open_key.set(None);
            sheet_open.set(false);
            return;
        }
        let md = viewport_is_md();
        inline_mode.set(md);
        open_key.set(Some(s.key.clone()));
        card.set(Some(s));
        if md {
            crate::components::ui::a11y::focus_id_later("plan-suggestion-claim");
        } else {
            sheet_open.set(true);
        }
    };
    let close_inline = move || {
        if let Some(k) = open_key.get_untracked() {
            open_key.set(None);
            crate::components::ui::a11y::focus_id(&chip_dom_id(&k));
        }
    };

    let rows = move || shown.get().into_iter().enumerate().collect::<Vec<_>>();
    let inline_open = move || open_key.with(|k| k.is_some()) && inline_mode.get();
    let inline_has = move || inline_mode.get() && card.with(|c| c.is_some());

    view! {
        <section aria-label="Suggestions" data-testid="plan-suggestions" class="flex flex-col border-t border-rule">
            <p class="sr-only" aria-live="polite">{move || live.get()}</p>
            {move || match kind.get() {
                Kind::Nothing => ().into_any(),
                Kind::Wait => view! {
                    <p class="py-2 text-caption text-muted" data-testid="suggestions-wait">"Suggestions appear when both drafts are in."</p>
                }.into_any(),
                Kind::None => view! {
                    <p id="plan-suggestions-none" tabindex="-1" class="py-2 text-caption text-muted" data-testid="suggestions-none">"No suggestions for these picks."</p>
                }.into_any(),
                Kind::Chips => view! {
                    <button
                        type="button"
                        id="plan-suggestions-toggle"
                        data-testid="suggestions-toggle"
                        aria-expanded=move || if row_open.get() { "true" } else { "false" }
                        aria-controls="plan-suggestion-row"
                        class="flex min-h-11 w-full cursor-pointer items-center gap-2 text-left text-label font-medium text-secondary transition-colors duration-[160ms] ease-out hover:text-primary motion-reduce:transition-none md:min-h-8"
                        on:click=move |_| {
                            let next = !row_open.get_untracked();
                            row_open.set(next);
                            if !next {
                                open_key.set(None);
                            }
                        }
                    >
                        // A drawn caret, so the button's text is only the count.
                        <svg
                            aria-hidden="true"
                            viewBox="0 0 8 8"
                            class=move || if row_open.get() { "size-2 shrink-0 rotate-90 fill-current text-muted" } else { "size-2 shrink-0 fill-current text-muted" }
                        >
                            <path d="M2 1l4 3-4 3z" />
                        </svg>
                        <span>{count_text}</span>
                    </button>
                    // Closed: zero rows, inert, and hidden once collapsed (visibility
                    // flips at the end of the transition, at its start on open).
                    <div
                        id="plan-suggestion-row"
                        class="invisible grid grid-rows-[0fr] transition-[grid-template-rows,visibility] duration-[180ms] ease-out motion-reduce:transition-none data-[open=true]:visible data-[open=true]:grid-rows-[1fr]"
                        data-open=move || if row_open.get() { "true" } else { "false" }
                        inert=move || !row_open.get()
                    >
                        <div class="min-h-0 overflow-hidden">
                            <ul role="list" class="-mx-4 flex snap-x snap-mandatory gap-2 overflow-x-auto overscroll-x-contain px-4 py-2 md:mx-0 md:flex-wrap md:snap-none md:overflow-visible md:px-0">
                                <For
                                    each=rows
                                    key=|(_, s)| s.key.clone()
                                    let:item
                                >
                                    {
                                        let (i, s) = item;
                                        let key = s.key.clone();
                                        let key_exp = key.clone();
                                        let key_ret = key.clone();
                                        let source = crate::models::graph::ReasonSource::from_label(&s.source);
                                        let claim = s.claim.clone();
                                        view! {
                                            <li class="shrink-0 snap-start">
                                                <SuggestionChip
                                                    claim=claim
                                                    source=source
                                                    index=i
                                                    controls="plan-suggestion-card".to_string()
                                                    expanded=Signal::derive(move || open_key.with(|k| k.as_deref() == Some(key_exp.as_str())))
                                                    retiring=Signal::derive(move || retiring.with(|r| r.as_ref().is_some_and(|(_, x)| x.key == key_ret)))
                                                    on_open=Callback::new(move |_| open_chip(s.clone()))
                                                    attr:id=chip_dom_id(&key)
                                                    attr:data-testid="suggestion-chip"
                                                    attr:data-key=key.clone()
                                                />
                                            </li>
                                        }
                                    }
                                </For>
                            </ul>
                            // Card 2: the margin note, in page flow (md and up).
                            <div
                                id="plan-suggestion-card"
                                role="region"
                                aria-labelledby="plan-suggestion-claim"
                                data-testid=move || inline_has().then_some("suggestion-card")
                                data-feedback-label=move || inline_has().then_some("Suggestion card")
                                class="invisible grid grid-rows-[0fr] transition-[grid-template-rows,visibility] duration-[180ms] ease-out motion-reduce:transition-none data-[open=true]:visible data-[open=true]:grid-rows-[1fr]"
                                data-open=move || if inline_open() { "true" } else { "false" }
                                inert=move || !inline_open()
                                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                    if ev.key() == "Escape" {
                                        ev.prevent_default();
                                        close_inline();
                                    }
                                }
                            >
                                <div class="min-h-0 overflow-hidden">
                                    {move || inline_has().then(|| card.get()).flatten().map(|s| view! {
                                        <div class="mt-3 flex max-w-[36rem] flex-col gap-2 border-t border-rule pb-1 pt-3">
                                            <p id="plan-suggestion-claim" tabindex="-1" class="text-body font-medium text-primary">{s.claim.clone()}</p>
                                            <SuggestionCardBody s=s accept=accept dismiss=dismiss full_width=false />
                                        </div>
                                    })}
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any(),
            }}
            // Card 5: on phones the same note in a bottom sheet. Outside
            // any Suspense, and fed from a signal (rule 59).
            {move || (!inline_mode.get()).then(|| card.get()).flatten().map(|s| {
                let title = s.claim.clone();
                view! {
                    <Dialog id="plan-suggestion-sheet" open=sheet_open title=title size=DialogSize::BottomSheet>
                        <div data-testid="suggestion-card" data-feedback-label="Suggestion card" class="flex flex-col gap-2">
                            <SuggestionCardBody s=s.clone() accept=accept dismiss=dismiss full_width=true />
                        </div>
                    </Dialog>
                }
            })}
        </section>
    }
}

fn chip_dom_id(key: &str) -> String {
    format!("plan-chip-{key}")
}

/// The card's reason, facts and two actions (inline and in the sheet).
#[component]
fn SuggestionCardBody(
    s: Suggestion,
    accept: Callback<Suggestion>,
    dismiss: Callback<Suggestion>,
    full_width: bool,
) -> impl IntoView {
    let source = crate::models::graph::ReasonSource::from_label(&s.source).label();
    let pair = match s.pair.as_slice() {
        [a, b] => format!("{a} and {b}"),
        [a] => a.clone(),
        _ if s.key.ends_with(":theirs") => "Their picks".to_string(),
        _ => "Our picks".to_string(),
    };
    let strength = s.strength;
    let reason = s.reason.clone();
    let s_add = s.clone();
    let s_dis = s;
    let buttons = if full_width {
        "mt-1 flex flex-col gap-2"
    } else {
        "mt-1 flex flex-wrap gap-2"
    };
    view! {
        <p class="note text-reading text-secondary">{reason}</p>
        <p class="flex flex-wrap items-center gap-x-2 gap-y-1 text-caption text-muted">
            <span>{source}</span>
            <span aria-hidden="true">"\u{b7}"</span>
            <span>{pair}</span>
            <span aria-hidden="true">"\u{b7}"</span>
            <StrengthDots value=strength />
        </p>
        <div class=buttons>
            <Button variant=ButtonVariant::Secondary size=ButtonSize::Touch full_width=full_width on_click=Callback::new(move |_| accept.run(s_add.clone()))>"Add to plan"</Button>
            <Button variant=ButtonVariant::Quiet size=ButtonSize::Touch full_width=full_width on_click=Callback::new(move |_| dismiss.run(s_dis.clone()))>"Not for us"</Button>
        </div>
    }
}

/// Win rates per strategy tag: (tag, games, wins).
type WinRates = Resource<Result<Vec<(String, i32, i32)>, ServerFnError>>;

#[component]
fn StrategySection(ed: PlanEd, win_rates: WinRates, echo: Echo) -> impl IntoView {
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
                    <div class="relative">
                        <TextField value=ed.win rows=4 placeholder="Force fights at the second dragon" />
                        <EchoLayer echo=echo target=SuggestionTarget::WinConditions />
                    </div>
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
    // The source reads only signals; the drafts resource is awaited inside
    // the fetcher, so nothing reads a resource outside a Suspense.
    let vs = Resource::new(
        move || (scope.get(), ed.draft_id.get()),
        move |(s, id)| async move {
            if s != "vs" {
                return Ok(Vec::new());
            }
            let opp = drafts
                .await
                .ok()
                .unwrap_or_default()
                .into_iter()
                .find(|d| d.id.as_deref() == Some(id.as_str()))
                .and_then(|d| d.opponent)
                .filter(|o| !o.is_empty());
            match opp {
                Some(o) => get_strategy_win_rates_vs_opponent(o).await,
                None => Ok(Vec::new()),
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
                            <crate::components::record_table::RecordTable caption="Win rate by strategy" caption_hidden=true first="Strategy" rows=rows current=current />
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
