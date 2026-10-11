//! Post-game reviews: what went well, what to improve and the action
//! items that come out of a game, linked to its match, plan and draft, with
//! the patterns that recur across reviews.

use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, Callout, CalloutTone, ConfirmDialog, ConfirmTone, Density,
    EmptySize, EmptyState, Field, GameResult, GlyphSize, PageHeader, PageShell, ResultGlyph, Role,
    RoleLabel, SectionHeader, Segment, SegmentedControl, Select, SelectOption, Skeleton,
    SkeletonGroup, SkeletonShape, TextField, Toast, ToastContext,
};
use crate::models::draft::Draft;
use crate::models::game_plan::{GamePlan, PostGameLearning};
use leptos::prelude::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn list_reviews() -> Result<Vec<PostGameLearning>, ServerFnError> {
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

    db::list_post_game_learnings(&surreal, &team_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn list_plans_for_postgame() -> Result<Vec<GamePlan>, ServerFnError> {
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
pub async fn list_drafts_for_postgame() -> Result<Vec<Draft>, ServerFnError> {
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
pub async fn get_recent_match_ids() -> Result<Vec<String>, ServerFnError> {
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

    let rows = db::get_team_match_stats(&surreal, &team_id)
        .await
        .map_err(authz::server_err)?;

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
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;
    let Some(team_id) = authz::current_team_for(&surreal, &user).await? else {
        return Ok(None);
    };
    db::get_game_plan(&surreal, &team_id, &plan_id)
        .await
        .map_err(authz::server_err)
}

/// A review may only link to a game plan and draft of the same team.
#[cfg(feature = "ssr")]
async fn check_review_links(
    db: &surrealdb::Surreal<surrealdb::engine::local::Db>,
    team_id: &str,
    review: &PostGameLearning,
) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};
    if let Some(pid) = review.game_plan_id.as_deref().filter(|s| !s.is_empty()) {
        db::ensure_team_record(db, "game_plan", pid, team_id)
            .await
            .map_err(authz::server_err)?;
    }
    if let Some(did) = review.draft_id.as_deref().filter(|s| !s.is_empty()) {
        db::ensure_team_record(db, "draft", did, team_id)
            .await
            .map_err(authz::server_err)?;
    }
    Ok(())
}

#[server]
pub async fn create_review(review_json: String) -> Result<(String, usize), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
        .ok_or_else(|| ServerFnError::new(authz::NO_TEAM))?;

    let mut review: PostGameLearning =
        serde_json::from_str(&review_json).map_err(authz::server_err)?;
    review.team_id = team_id.clone();
    review.created_by = user.id;
    check_review_links(&surreal, &team_id, &review).await?;

    // The review and its new action items are written in one transaction.
    db::create_review_with_action_items(&surreal, review)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn update_review(review_json: String) -> Result<usize, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;
    // The team comes from the session, never from the submitted review.
    let team_id = authz::require_team(&surreal, &user).await?;
    let review: PostGameLearning = serde_json::from_str(&review_json).map_err(authz::server_err)?;
    check_review_links(&surreal, &team_id, &review).await?;
    db::update_review_with_action_items(&surreal, &team_id, review)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn delete_review(review_id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;
    let team_id = authz::require_team(&surreal, &user).await?;
    db::delete_post_game_learning(&surreal, &team_id, &review_id)
        .await
        .map_err(authz::server_err)
}

/// Auto-detect win/loss outcome from Riot API by matching the linked draft's picks
/// against the user's recent match history.
#[server]
pub async fn auto_detect_outcome(draft_id: String) -> Result<Option<String>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;
    use crate::server::rate_limit::Bucket;
    use crate::server::riot;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;
    let Some(team_id) = authz::current_team_for(&surreal, &user).await? else {
        return Ok(None);
    };

    // No Riot account linked — cannot auto-detect
    let puuid = match user.riot_puuid {
        Some(ref p) if !p.is_empty() => p.clone(),
        _ => return Ok(None),
    };

    // Load the draft to find our side's picked champions
    let draft = match db::get_draft_for_prefill(&surreal, &team_id, &draft_id)
        .await
        .map_err(authz::server_err)?
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

    let riot_client = authz::riot()?;
    if !riot_client.is_configured() {
        return Ok(None);
    }
    authz::rate_limit(Bucket::Riot).await?;
    riot_client
        .ensure_headroom(riot::MATCH_HISTORY_REQUESTS)
        .map_err(authz::server_err)?;

    // Fetch recent match history (up to 20 matches; check first 5 for a match)
    let team_region = db::get_team_region(&surreal, &team_id)
        .await
        .map_err(authz::server_err)?;
    let platform = riot::platform_for(&[user.riot_region.as_deref(), team_region.as_deref()]);
    let matches = riot_client
        .fetch_match_history(&puuid, None, platform)
        .await
        .map_err(authz::server_err)?;

    for m in matches.into_iter().take(5) {
        let played_champ = m.champion.to_lowercase();
        if our_picks.contains(&played_champ) {
            let outcome = if m.win { "win" } else { "loss" };
            return Ok(Some(outcome.to_string()));
        }
    }

    Ok(None) // No matching game found in recent history
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
// Editor state
// ---------------------------------------------------------------------------

fn opt(s: String) -> Option<String> {
    (!s.trim().is_empty()).then_some(s)
}

fn lines(s: &str) -> Vec<String> {
    s.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn result_of(win_loss: Option<&str>) -> Option<GameResult> {
    match win_loss {
        Some("win") => Some(GameResult::Win),
        Some("loss") => Some(GameResult::Loss),
        _ => None,
    }
}

/// Short label for a match id ("EUW1_7012345678" stays as it is; very long
/// ids keep their tail).
fn match_label(id: &str) -> String {
    if id.chars().count() > 20 {
        let tail: String = id
            .chars()
            .rev()
            .take(12)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("\u{2026}{tail}")
    } else {
        id.to_string()
    }
}

#[derive(Clone, Copy)]
struct ReviewEd {
    id: RwSignal<Option<String>>,
    match_id: RwSignal<String>,
    plan_id: RwSignal<String>,
    draft_id: RwSignal<String>,
    went_well: RwSignal<String>,
    improve: RwSignal<String>,
    actions: RwSignal<String>,
    notes: RwSignal<String>,
    /// "", "win" or "loss".
    outcome: RwSignal<String>,
    /// "" or "1".."5".
    rating: RwSignal<String>,
}

impl ReviewEd {
    fn new() -> Self {
        let s = || RwSignal::new(String::new());
        Self {
            id: RwSignal::new(None),
            match_id: s(),
            plan_id: s(),
            draft_id: s(),
            went_well: s(),
            improve: s(),
            actions: s(),
            notes: s(),
            outcome: s(),
            rating: s(),
        }
    }
    fn clear(&self) {
        self.id.set(None);
        for sig in [
            self.match_id,
            self.plan_id,
            self.draft_id,
            self.went_well,
            self.improve,
            self.actions,
            self.notes,
            self.outcome,
            self.rating,
        ] {
            sig.set(String::new());
        }
    }
    fn load(&self, r: &PostGameLearning) {
        self.id.set(r.id.clone());
        self.match_id
            .set(r.match_riot_id.clone().unwrap_or_default());
        self.plan_id.set(r.game_plan_id.clone().unwrap_or_default());
        self.draft_id.set(r.draft_id.clone().unwrap_or_default());
        self.went_well.set(r.what_went_well.join("\n"));
        self.improve.set(r.improvements.join("\n"));
        self.actions.set(r.action_items.join("\n"));
        self.notes.set(r.open_notes.clone().unwrap_or_default());
        self.outcome.set(r.win_loss.clone().unwrap_or_default());
        self.rating
            .set(r.rating.map(|n| n.to_string()).unwrap_or_default());
    }
    fn build(&self) -> PostGameLearning {
        PostGameLearning {
            id: self.id.get_untracked(),
            team_id: String::new(),
            match_riot_id: opt(self.match_id.get_untracked()),
            game_plan_id: opt(self.plan_id.get_untracked()),
            draft_id: opt(self.draft_id.get_untracked()),
            what_went_well: lines(&self.went_well.get_untracked()),
            improvements: lines(&self.improve.get_untracked()),
            action_items: lines(&self.actions.get_untracked()),
            open_notes: opt(self.notes.get_untracked()),
            created_by: String::new(),
            win_loss: opt(self.outcome.get_untracked()),
            rating: self.rating.get_untracked().parse::<u8>().ok(),
        }
    }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn PostGamePage() -> impl IntoView {
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
    let reviews = Resource::new(|| (), |_| list_reviews());
    let plans = Resource::new(|| (), |_| list_plans_for_postgame());
    let drafts = Resource::new(|| (), |_| list_drafts_for_postgame());
    let match_ids = Resource::new(|| (), |_| get_recent_match_ids());
    let ed = ReviewEd::new();
    let created_items = RwSignal::new(None::<usize>);

    // `?review_id=` opens a review; `?plan_id=`/`?draft_id=` seed a new one.
    let opened = StoredValue::new(false);
    Effect::new(move |_| {
        if opened.get_value() {
            return;
        }
        let q = query.read();
        if let Some(target) = q.get("review_id") {
            if let Some(Ok(list)) = reviews.get() {
                if let Some(r) = list
                    .iter()
                    .find(|r| r.id.as_deref() == Some(target.as_str()))
                {
                    ed.load(r);
                    opened.set_value(true);
                }
            }
            return;
        }
        if let Some(pid) = q.get("plan_id") {
            ed.plan_id.set(pid);
        }
        if let Some(did) = q.get("draft_id") {
            ed.draft_id.set(did);
        }
        opened.set_value(true);
    });

    let do_save = Callback::new(move |_: leptos::ev::MouseEvent| {
        let review = ed.build();
        let json = serde_json::to_string(&review).unwrap_or_default();
        let is_update = review.id.is_some();
        leptos::task::spawn_local(async move {
            let res = if is_update {
                update_review(json).await.map(|n| (String::new(), n))
            } else {
                create_review(json).await
            };
            match res {
                Ok((id, n)) => {
                    if !is_update && !id.is_empty() {
                        ed.id.set(Some(id));
                    }
                    created_items.set((n > 0).then_some(n));
                    reviews.refetch();
                    toast.push.run(Toast::success("Review saved"));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not save the review: {e}"))),
            }
        });
    });
    let new_review = Callback::new(move |_: leptos::ev::MouseEvent| {
        ed.clear();
        created_items.set(None);
    });
    let confirm_delete = RwSignal::new(false);
    let do_delete = Callback::new(move |_| {
        let Some(id) = ed.id.get_untracked() else {
            return;
        };
        leptos::task::spawn_local(async move {
            match delete_review(id).await {
                Ok(()) => {
                    ed.clear();
                    created_items.set(None);
                    reviews.refetch();
                    toast.push.run(Toast::success("Review deleted"));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not delete: {e}"))),
            }
        });
    });

    let plan_name = move |id: &str| -> Option<String> {
        plans
            .get()
            .and_then(|r| r.ok())
            .and_then(|l| l.into_iter().find(|p| p.id.as_deref() == Some(id)))
            .map(|p| {
                if p.name.trim().is_empty() {
                    "Untitled plan".to_string()
                } else {
                    p.name
                }
            })
    };

    view! {
        <PageShell density=Density::Compact attr:data-feedback-label="Post game">
            <PageHeader
                title=move || if ed.id.get().is_some() { "Review".to_string() } else { "New review".to_string() }
                eyebrow="Post-game"
                meta=move || view! {
                    {move || result_of(Some(ed.outcome.get().as_str())).map(|r| view! { <ResultGlyph result=r word=true /> })}
                    <Suspense fallback=|| ()>
                        {move || {
                            let pid = ed.plan_id.get();
                            (!pid.is_empty()).then(|| plan_name(&pid).map(|n| view! { <span>"Plan: " {n}</span> }))
                        }}
                    </Suspense>
                    <span class="text-muted" data-testid="review-save-state">
                        {move || if ed.id.get().is_some() { "Saved" } else { "Not saved yet" }}
                    </span>
                }
                actions=move || view! {
                    <Button variant=ButtonVariant::Quiet on_click=new_review>"New review"</Button>
                    {move || ed.id.get().map(|_| view! {
                        <Button variant=ButtonVariant::Danger on_click=Callback::new(move |_| confirm_delete.set(true))>"Delete"</Button>
                    })}
                    <Button variant=ButtonVariant::Primary on_click=do_save attr:data-testid="review-save">"Save review"</Button>
                }
            />
            <ConfirmDialog
                id="delete-review"
                open=confirm_delete
                title="Delete this review?"
                message="The review is deleted. Action items created from it stay on the action items page."
                confirm_label="Delete review"
                tone=ConfirmTone::Destructive
                on_confirm=do_delete
            />
            {move || created_items.get().map(|n| view! {
                <div class="mb-6">
                    <Callout
                        tone=CalloutTone::Success
                        title=if n == 1 { "1 action item created".to_string() } else { format!("{n} action items created") }
                        action=move || view! { <Button size=ButtonSize::Sm href="/action-items">"Open action items"</Button> }
                    >
                        "Each line under Action items became an item the team can track."
                    </Callout>
                </div>
            })}
            <div class="grid grid-cols-1 gap-x-10 gap-y-10 lg:grid-cols-[16rem_minmax(0,1fr)]">
                <div class="flex min-w-0 flex-col gap-10">
                    <section aria-labelledby="reviews-heading" class="flex flex-col gap-3">
                        <SectionHeader title="Reviews" id="reviews-heading" />
                        <Suspense fallback=|| view! { <SkeletonGroup label="Loading reviews"><Skeleton shape=SkeletonShape::Line lines=3 /></SkeletonGroup> }>
                            {move || reviews.get().map(|r| match r {
                                Err(_) => view! { <Callout tone=CalloutTone::Danger>"The reviews did not load."</Callout> }.into_any(),
                                Ok(list) if list.is_empty() => view! {
                                    <EmptyState size=EmptySize::Compact message="Reviews your team writes after games are listed here. Write one on the right, or start one from a game plan." />
                                }.into_any(),
                                Ok(list) => view! {
                                    <ul class="flex flex-col border-t border-rule" data-testid="review-list">
                                        {list.into_iter().map(|r| {
                                            let id = r.id.clone().unwrap_or_default();
                                            let label = r.game_plan_id.as_deref().and_then(plan_name)
                                                .or_else(|| r.match_riot_id.as_deref().map(match_label))
                                                .unwrap_or_else(|| "Review without a plan".to_string());
                                            let points = r.what_went_well.len() + r.improvements.len() + r.action_items.len();
                                            let result = result_of(r.win_loss.as_deref());
                                            let r2 = r.clone();
                                            view! {
                                                <li class="border-b border-rule">
                                                    <Button
                                                        variant=ButtonVariant::Quiet
                                                        full_width=true
                                                        pressed=Signal::derive(move || ed.id.get().as_deref() == Some(id.as_str()))
                                                        on_click=Callback::new(move |_| { created_items.set(None); ed.load(&r2); })
                                                    >
                                                        <span class="flex w-full min-w-0 items-center gap-2 text-left">
                                                            {match result {
                                                                Some(res) => view! { <ResultGlyph result=res size=GlyphSize::Sm /> }.into_any(),
                                                                None => view! { <span class="figures inline-grid size-5 shrink-0 place-items-center text-caption text-muted" aria-label="No result">"\u{2014}"</span> }.into_any(),
                                                            }}
                                                            <span class="min-w-0 flex-1 truncate">{label}</span>
                                                            <span class="figures shrink-0 text-caption font-normal text-muted">{format!("{points} pts")}</span>
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
                    <PatternsSection reviews=reviews />
                </div>
                <div class="flex min-w-0 flex-col gap-12">
                    <LinksSection ed=ed plans=plans drafts=drafts match_ids=match_ids />
                    <OutcomeSection ed=ed />
                    <section aria-labelledby="review-points-heading" class="flex flex-col gap-4">
                        <SectionHeader title="What we learned" id="review-points-heading" />
                        <div class="grid grid-cols-1 gap-x-6 gap-y-4 md:grid-cols-3">
                            <Field id="review-well" label="What went well" hint="One point per line.">
                                <TextField value=ed.went_well rows=7 placeholder="Dragon control around 14 minutes" />
                            </Field>
                            <Field id="review-improve" label="To improve" hint="One point per line.">
                                <TextField value=ed.improve rows=7 placeholder="Vision before Baron" />
                            </Field>
                            <Field id="review-actions" label="Action items" hint="Each line becomes an action item when you save.">
                                <TextField value=ed.actions rows=7 placeholder="Drill Baron setups in Thursday scrims" />
                            </Field>
                        </div>
                        <Field id="review-notes" label="Notes">
                            <TextField value=ed.notes rows=4 placeholder="Context the points above do not cover." />
                        </Field>
                    </section>
                </div>
            </div>
        </PageShell>
    }
}

/// Links to the match, the game plan and the draft, plus the plan summary.
#[component]
fn LinksSection(
    ed: ReviewEd,
    plans: Resource<Result<Vec<GamePlan>, ServerFnError>>,
    drafts: Resource<Result<Vec<Draft>, ServerFnError>>,
    match_ids: Resource<Result<Vec<String>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <section aria-labelledby="review-links-heading" class="flex flex-col gap-4">
            <SectionHeader title="Game" id="review-links-heading" />
            <div class="grid grid-cols-1 gap-x-6 gap-y-4 md:grid-cols-3">
                <Suspense fallback=|| view! { <Skeleton shape=SkeletonShape::Block size="h-9 w-full" /> }>
                    {move || match_ids.get().map(|r| {
                        let ids = r.unwrap_or_default();
                        let hint = if ids.is_empty() { "Synced team matches appear here." } else { "From your team's synced matches." };
                        let mut opts = vec![SelectOption::new("", "No match")];
                        opts.extend(ids.into_iter().map(|id| { let l = match_label(&id); SelectOption::new(id, l) }));
                        view! { <Field id="review-match" label="Match" hint=hint><Select value=ed.match_id options=opts /></Field> }
                    })}
                </Suspense>
                <Suspense fallback=|| view! { <Skeleton shape=SkeletonShape::Block size="h-9 w-full" /> }>
                    {move || plans.get().map(|r| {
                        let mut opts = vec![SelectOption::new("", "No game plan")];
                        opts.extend(r.unwrap_or_default().into_iter().map(|p| {
                            let n = if p.name.trim().is_empty() { "Untitled plan".to_string() } else { p.name };
                            SelectOption::new(p.id.unwrap_or_default(), n)
                        }));
                        view! { <Field id="review-plan" label="Game plan"><Select value=ed.plan_id options=opts /></Field> }
                    })}
                </Suspense>
                <Suspense fallback=|| view! { <Skeleton shape=SkeletonShape::Block size="h-9 w-full" /> }>
                    {move || drafts.get().map(|r| {
                        let mut opts = vec![SelectOption::new("", "No draft")];
                        opts.extend(r.unwrap_or_default().into_iter().map(|d| SelectOption::new(d.id.unwrap_or_default(), d.name)));
                        view! { <Field id="review-draft" label="Draft"><Select value=ed.draft_id options=opts /></Field> }
                    })}
                </Suspense>
            </div>
            <div class="flex flex-wrap gap-2">
                {move || { let p = ed.plan_id.get(); (!p.is_empty()).then(|| view! { <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet href=format!("/game-plan?plan_id={p}")>"Open the game plan"</Button> }) }}
                {move || { let d = ed.draft_id.get(); (!d.is_empty()).then(|| view! { <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet href=format!("/draft?draft_id={d}")>"Open the draft"</Button> }) }}
            </div>
            {move || {
                let pid = ed.plan_id.get();
                (!pid.is_empty()).then(|| view! { <LinkedPlan plan_id=pid /> })
            }}
        </section>
    }
}

/// Win or loss, the Riot lookup, and how well the plan worked.
#[component]
fn OutcomeSection(ed: ReviewEd) -> impl IntoView {
    let fetching = RwSignal::new(false);
    let status = RwSignal::new(None::<String>);
    let fetch = Callback::new(move |_: leptos::ev::MouseEvent| {
        let did = ed.draft_id.get_untracked();
        if did.is_empty() {
            return;
        }
        fetching.set(true);
        status.set(None);
        leptos::task::spawn_local(async move {
            match auto_detect_outcome(did).await {
                Ok(Some(r)) => {
                    let word = if r == "win" { "a win" } else { "a loss" };
                    ed.outcome.set(r);
                    status.set(Some(format!("Found {word} with one of the draft's picks in your recent games.")));
                }
                Ok(None) => status.set(Some("No recent game of yours matches the draft's picks (or no Riot account is linked). Set the result yourself.".into())),
                Err(e) => status.set(Some(format!("The lookup failed: {e}. Set the result yourself."))),
            }
            fetching.set(false);
        });
    });
    view! {
        <section aria-labelledby="review-outcome-heading" class="flex flex-col gap-4">
            <SectionHeader title="Result" id="review-outcome-heading" />
            <div class="flex flex-wrap items-end gap-x-10 gap-y-4">
                <div class="flex flex-col gap-1.5">
                    <span class="text-label font-medium text-secondary">"Outcome"</span>
                    <div>
                        <SegmentedControl
                            id="review-outcome"
                            label="Outcome"
                            options=vec![Segment::new("", "Not set"), Segment::new("win", "Win"), Segment::new("loss", "Loss")]
                            value=ed.outcome
                        />
                    </div>
                </div>
                <div class="w-48">
                    <Field id="review-rating" label="How well did the plan work?">
                        <Select value=ed.rating options={
                            let mut o = vec![SelectOption::new("", "Not rated")];
                            o.extend((1..=5).map(|i| SelectOption::new(i.to_string(), format!("{i} of 5"))));
                            o
                        } />
                    </Field>
                </div>
            </div>
            {move || (ed.outcome.get().is_empty() && !ed.draft_id.get().is_empty()).then(|| view! {
                <div class="flex flex-wrap items-center gap-3">
                    <Button size=ButtonSize::Sm on_click=fetch loading=Signal::derive(move || fetching.get())>"Look up the result"</Button>
                    <span class="text-caption text-muted">"Checks your recent ranked games for one of the draft's picks (uses the Riot request budget)."</span>
                </div>
            })}
            {move || status.get().map(|s| view! { <p class="text-label text-secondary" role="status">{s}</p> })}
        </section>
    }
}

/// Recurring points across reviews (two or more mentions).
#[component]
fn PatternsSection(
    reviews: Resource<Result<Vec<PostGameLearning>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <section aria-labelledby="patterns-heading" class="flex flex-col gap-3">
            <SectionHeader title="Patterns" id="patterns-heading" />
            <Suspense fallback=|| ()>
                {move || reviews.get().map(|r| {
                    let list = r.unwrap_or_default();
                    let (good, bad) = analyze_patterns(&list);
                    if good.is_empty() && bad.is_empty() {
                        return view! {
                            <EmptyState size=EmptySize::Compact message="A point that shows up in two or more reviews appears here, as a strength or a recurring issue." />
                        }.into_any();
                    }
                    let block = |title: &'static str, rows: Vec<(String, usize)>| (!rows.is_empty()).then(|| view! {
                        <div class="flex flex-col gap-1">
                            <h3 class="eyebrow">{title}</h3>
                            <ul class="flex flex-col">
                                {rows.into_iter().map(|(t, n)| view! {
                                    <li class="flex items-baseline justify-between gap-2 border-b border-rule py-1.5 last:border-b-0">
                                        <span class="min-w-0 truncate text-label text-primary">{t}</span>
                                        <span class="figures shrink-0 text-caption text-muted">{format!("{n} reviews")}</span>
                                    </li>
                                }).collect_view()}
                            </ul>
                        </div>
                    });
                    view! {
                        <div class="flex flex-col gap-4">
                            {block("Strengths", good)}
                            {block("Recurring issues", bad)}
                        </div>
                    }.into_any()
                })}
            </Suspense>
        </section>
    }
}

/// What the linked plan said, to review against.
#[component]
fn LinkedPlan(plan_id: String) -> impl IntoView {
    let plan = Resource::new(move || plan_id.clone(), get_linked_plan);
    view! {
        <Suspense fallback=|| view! { <Skeleton shape=SkeletonShape::Line lines=3 /> }>
            {move || plan.get().map(|r| match r {
                Ok(Some(p)) => {
                    let roles = [
                        (Role::Top, p.top_strategy.clone()),
                        (Role::Jungle, p.jungle_strategy.clone()),
                        (Role::Mid, p.mid_strategy.clone()),
                        (Role::Bot, p.bot_strategy.clone()),
                        (Role::Support, p.support_strategy.clone()),
                    ];
                    view! {
                        <div class="flex flex-col gap-2 border-t border-rule pt-3" data-testid="linked-plan">
                            <p class="eyebrow">"What the plan said"</p>
                            {(!p.win_conditions.is_empty()).then(|| view! {
                                <p class="text-label text-secondary"><span class="font-medium text-primary">"Win conditions: "</span>{p.win_conditions.join("; ")}</p>
                            })}
                            {(!p.teamfight_strategy.is_empty()).then(|| view! {
                                <p class="text-label text-secondary"><span class="font-medium text-primary">"Teamfights: "</span>{p.teamfight_strategy.clone()}</p>
                            })}
                            <ul class="flex flex-col gap-0.5">
                                {roles.into_iter().filter_map(|(role, s)| s.map(|s| view! {
                                    <li class="flex gap-2 text-label text-secondary"><RoleLabel role=role /><span>{s}</span></li>
                                })).collect_view()}
                            </ul>
                        </div>
                    }.into_any()
                }
                Ok(None) => view! { <p class="text-label text-muted">"The linked plan no longer exists."</p> }.into_any(),
                Err(_) => view! { <p class="text-label text-muted">"The linked plan did not load."</p> }.into_any(),
            })}
        </Suspense>
    }
}
