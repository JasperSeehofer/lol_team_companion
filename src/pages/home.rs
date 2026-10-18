//! Home. Signed out: the landing (what Team Companion is, the closed beta, sign
//! in). Signed in with a team: Match Day, the guidance flow for the next
//! scrim or match: the preparation sequence (brief read, draft plans, game
//! plan, open action items, review after the game), the opponent brief, open
//! action items, the last review's lesson and recent form. Everything comes
//! from the team's own records; empty parts say how they fill.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::guidance::{GettingStarted, NudgeLine, WelcomePanel};
use crate::components::local_time::{
    local_input_to_utc, parse_utc, utc_to_local_inputs, LocalTime, TimeStyle,
};
use crate::components::ui::PrepEmphasis;
use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, ChampionAvatar, ConfirmDialog, ConfirmTone, Dialog, Edge,
    EmptySize, EmptyState, Field, GameResult, GlyphSize, HeadingLevel, InputKind, Layer, Pad,
    PageHeader, PageShell, PageWidth, PrepSequence, PrepState, PrepStep, ResultGlyph, RiftCrop,
    RiftGround, RiftSketch, Role, RoleLabel, SectionHeader, Segment, SegmentedControl, Select,
    SelectOption, Skeleton, SkeletonGroup, SkeletonShape, Surface, SurfaceTag, TextField, Toast,
    ToastContext,
};
use crate::models::guidance::HomeGuide;
use crate::models::match_day::{
    format_label, kind_label, BriefPlayer, LinkRef, MatchDay, OpponentBrief, ScheduledMatchInput,
    FORMATS, KINDS,
};
use crate::pages::closed_beta::Landing;

/// What the home page shows for the visitor.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum Home {
    SignedOut,
    NoTeam { username: String, guide: HomeGuide },
    Team(Box<MatchDay>, HomeGuide),
}

/// Match Day's guidance for the user (M9). A failure here never breaks
/// Match Day: it is logged and the page shows no guidance.
#[cfg(feature = "ssr")]
async fn home_guide_for(
    db: &crate::server::authz::DbHandle,
    user: &crate::server::auth::AppUser,
    guide: bool,
    skip_href: Option<&str>,
) -> HomeGuide {
    crate::server::guidance::home_guide(
        db,
        &user.id,
        &user.username,
        user.riot_summoner_name.is_some(),
        guide,
        skip_href,
        chrono::Utc::now(),
    )
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(user = %user.id, error = %e, "match day guidance failed");
        HomeGuide::default()
    })
}

/// Home. `guide` is "Guide me" (`/?guide=1`): the checklist shows even when
/// it is hidden or complete.
#[server]
pub async fn get_home(guide: bool) -> Result<Home, ServerFnError> {
    use crate::models::match_day::{shorten, BriefPlayer, OpponentBrief, PatchLine, ReviewLine};
    use crate::models::opponent::is_stale;
    use crate::models::utils::format_timestamp;
    use crate::server::{authz, db};

    let Some(user) = authz::optional_user().await? else {
        return Ok(Home::SignedOut);
    };
    let db = authz::db()?;
    let Some((team, members)) = db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(authz::server_err)?
    else {
        let guide = home_guide_for(&db, &user, guide, None).await;
        return Ok(Home::NoTeam {
            username: user.username,
            guide,
        });
    };
    let team_id = team.id.clone().unwrap_or_default();

    let mut next = db::next_scheduled_match(&db, &team_id)
        .await
        .map_err(authz::server_err)?;
    // "Read by N of M" counts current members only (people who left keep
    // their mark in the row).
    if let Some(m) = next.as_mut() {
        m.brief_read_by
            .retain(|u| members.iter().any(|mem| &mem.user_id == u));
    }

    let mut brief = None;
    let mut facts = db::MatchPrepFacts::default();
    if let Some(m) = &next {
        if let Some(opp_id) = &m.opponent_id {
            if let Some((opp, players)) = db::get_opponent(&db, &team_id, opp_id)
                .await
                .map_err(authz::server_err)?
            {
                let latest = players
                    .iter()
                    .filter_map(|p| p.last_fetched.clone())
                    .filter(|s| !s.is_empty())
                    .max();
                let mut ps: Vec<BriefPlayer> = players
                    .iter()
                    .filter(|p| !p.name.trim().is_empty() || p.riot_summoner_name.is_some())
                    .map(|p| BriefPlayer {
                        name: if p.name.trim().is_empty() {
                            p.riot_summoner_name.clone().unwrap_or_default()
                        } else {
                            p.name.clone()
                        },
                        role: p.role.clone(),
                        champions: p.recent_champions.iter().take(3).cloned().collect(),
                    })
                    .collect();
                ps.sort_by_key(|p| Role::parse(&p.role).map(|r| r as u8).unwrap_or(9));
                facts = db::match_prep_facts(&db, &team_id, opp_id, &opp.name, &m.starts_at)
                    .await
                    .map_err(authz::server_err)?;
                brief = Some(OpponentBrief {
                    id: opp_id.clone(),
                    name: opp.name.clone(),
                    notes: opp.notes.clone().filter(|n| !n.trim().is_empty()),
                    champions_seen: players.iter().map(|p| p.recent_champions.len()).sum(),
                    players: ps,
                    surveyed: latest.as_deref().map(format_timestamp),
                    stale: latest.as_deref().map(is_stale).unwrap_or(false),
                });
            }
        }
    }

    let open = db::list_open_action_items(&db, &team_id)
        .await
        .map_err(authz::server_err)?;
    let reviews = db::recent_reviews(&db, &team_id, 20)
        .await
        .map_err(authz::server_err)?;
    let last_review = reviews.first().map(|r| ReviewLine {
        id: r.id.clone().unwrap_or_default(),
        won: r.win_loss.as_deref().map(|w| w == "win"),
        lesson: r
            .improvements
            .iter()
            .chain(r.action_items.iter())
            .find(|l| !l.trim().is_empty())
            .cloned()
            .or_else(|| r.open_notes.clone().filter(|n| !n.trim().is_empty()))
            .map(|l| shorten(&l, 140)),
        created_at: r.created_at.clone(),
    });
    let mut form: Vec<bool> = reviews
        .iter()
        .filter_map(|r| r.win_loss.as_deref())
        .filter(|w| *w == "win" || *w == "loss")
        .take(5)
        .map(|w| w == "win")
        .collect();
    form.reverse();
    let patch = db::list_patch_changes(&db, 1)
        .await
        .unwrap_or_default()
        .into_iter()
        .next()
        .map(|p| PatchLine {
            version: p.new_version,
            since: p.detected_at,
        })
        .filter(|p| p.is_news(chrono::Utc::now()));
    let opponents = db::list_opponents(&db, &team_id)
        .await
        .map_err(authz::server_err)?
        .into_iter()
        .filter_map(|o| {
            Some(LinkRef {
                id: o.id?,
                name: o.name,
            })
        })
        .collect();

    let md = MatchDay {
        team_name: team.name,
        member_count: members.len(),
        viewer_id: user.id.clone(),
        next,
        brief,
        drafts: facts.drafts,
        plans: facts.plans,
        review_after: facts.review_after,
        open_item_count: open.len(),
        open_items: open.into_iter().take(4).collect(),
        last_review,
        form,
        patch,
        opponents,
        now: chrono::Utc::now().to_rfc3339(),
    };
    // A nudge never repeats the prep sequence's next step.
    let skip = prep_steps(&md)
        .into_iter()
        .find(|s| s.state == PrepState::Next)
        .and_then(|s| s.href);
    let guide = home_guide_for(&db, &user, guide, skip.as_deref()).await;
    Ok(Home::Team(Box::new(md), guide))
}

/// Create or update the next match (any member).
#[server]
pub async fn save_next_match(input_json: String) -> Result<String, ServerFnError> {
    use crate::server::{authz, db};
    let input: ScheduledMatchInput = serde_json::from_str(&input_json)
        .map_err(|_| ServerFnError::new("The match details could not be read."))?;
    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::save_scheduled_match(&ctx.db, &team_id, &ctx.user.id, input)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn remove_next_match(match_id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};
    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::delete_scheduled_match(&ctx.db, &team_id, &match_id)
        .await
        .map_err(authz::server_err)
}

/// Mark the next match's opponent brief read (or unread) for yourself.
#[server]
pub async fn mark_brief_read(match_id: String, read: bool) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};
    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::set_brief_read(&ctx.db, &team_id, &match_id, &ctx.user.id, read)
        .await
        .map_err(authz::server_err)
}

fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// The preparation sequence for the next match: what is done, what needs
/// attention, and the one step to do now (the first open step that has a
/// place to go; the review only once the match has started).
pub fn prep_steps(md: &MatchDay) -> Vec<PrepStep> {
    let Some(m) = &md.next else {
        return Vec::new();
    };
    let started = match (parse_utc(&m.starts_at), parse_utc(&md.now)) {
        (Some(s), Some(n)) => n >= s,
        _ => false,
    };
    let read_by = m.brief_read_by.len();
    let viewer_read = m.brief_read_by.contains(&md.viewer_id);
    let read_status = format!("Read by {read_by} of {}", md.member_count);

    let brief = match &md.brief {
        None => PrepStep::new("Read the opponent brief", PrepState::Todo)
            .detail(
                "No opponent set for this match yet. Choose their dossier in the match details.",
            )
            .status("No opponent")
            .link("/opponents", "Open the dossiers"),
        Some(b) => {
            let survey = match (&b.surveyed, b.stale) {
                (Some(s), true) => format!("Surveyed {s}: refresh the dossier before the match."),
                (Some(s), false) => format!(
                    "{} scouted, surveyed {s}.",
                    plural(b.players.len(), "player", "players")
                ),
                (None, _) => format!(
                    "{} scouted, not refreshed from Riot yet.",
                    plural(b.players.len(), "player", "players")
                ),
            };
            let state = if viewer_read {
                PrepState::Done
            } else if b.stale {
                PrepState::Attention
            } else {
                PrepState::Todo
            };
            PrepStep::new(format!("Read the brief on {}", b.name), state)
                .detail(survey)
                .status(read_status)
                .link("#brief", "Read the brief")
        }
    };

    let opp_name = md
        .brief
        .as_ref()
        .map(|b| b.name.clone())
        .unwrap_or_else(|| "them".to_string());
    let drafts = if md.drafts.is_empty() {
        let mut s = PrepStep::new("Draft plans", PrepState::Todo)
            .detail(if md.brief.is_some() {
                format!("No draft against {opp_name} yet.")
            } else {
                "Drafts against the opponent count here once it is set.".to_string()
            })
            .status("None yet");
        if md.brief.is_some() {
            s = s.link("/draft", "Start a draft");
        }
        s
    } else {
        PrepStep::new("Draft plans", PrepState::Done)
            .detail(
                md.drafts
                    .iter()
                    .map(|d| d.name.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
            .status(plural(md.drafts.len(), "draft", "drafts"))
            .link(
                format!("/draft?draft_id={}", md.drafts[0].id),
                "Open the draft",
            )
    };

    let plan = if md.plans.is_empty() {
        let mut s = PrepStep::new("Game plan", PrepState::Todo).status("None yet");
        match md.drafts.first() {
            Some(d) => {
                s = s
                    .detail(format!("Plan the game from \u{201c}{}\u{201d}.", d.name))
                    .link(
                        format!("/game-plan?draft_id={}", d.id),
                        "Write the game plan",
                    );
            }
            None => s = s.detail("A game plan starts from a draft."),
        }
        s
    } else {
        PrepStep::new("Game plan", PrepState::Done)
            .detail(
                md.plans
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
            .status(plural(md.plans.len(), "plan", "plans"))
            .link(
                format!("/game-plan?plan_id={}", md.plans[0].id),
                "Open the plan",
            )
    };

    let items = if md.open_item_count == 0 {
        PrepStep::new("Open action items", PrepState::Done)
            .detail("Nothing open from earlier reviews.")
            .status("None open")
            .link("/action-items", "Open action items")
    } else {
        PrepStep::new("Open action items", PrepState::Attention)
            .detail("Lessons from earlier reviews still waiting.")
            .status(format!("{} open", md.open_item_count))
            .link("/action-items", "Go through them")
    };

    let review = match (&md.review_after, started) {
        (Some(r), _) => PrepStep::new("After the game: review", PrepState::Done)
            .detail("The review of this match is written.")
            .status("Written")
            .link(format!("/post-game?review_id={}", r.id), "Open the review"),
        (None, false) => PrepStep::new("After the game: review", PrepState::Todo)
            .detail("Write it right after the match, while it is fresh.")
            .status("After the game"),
        (None, true) => {
            let href = match (md.plans.first(), md.drafts.first()) {
                (Some(p), _) => format!("/post-game?plan_id={}", p.id),
                (None, Some(d)) => format!("/post-game?draft_id={}", d.id),
                _ => "/post-game".to_string(),
            };
            PrepStep::new("After the game: review", PrepState::Todo)
                .detail("The match has started. Review it while it is fresh.")
                .status("To write")
                .link(href, "Write the review")
        }
    };

    let mut steps = vec![brief, drafts, plan, items, review];
    if let Some(next) = steps
        .iter_mut()
        .find(|s| matches!(s.state, PrepState::Todo | PrepState::Attention) && s.href.is_some())
    {
        next.state = PrepState::Next;
    }
    steps
}

#[component]
pub fn HomePage() -> impl IntoView {
    let query = leptos_router::hooks::use_query_map();
    let home = Resource::new(
        move || query.read().get("guide").as_deref() == Some("1"),
        get_home,
    );
    view! {
        <Suspense fallback=|| view! {
            <PageShell>
                <SkeletonGroup label="Loading Match Day">
                    <Skeleton width="w-72 max-w-full" size="h-10" />
                    <Skeleton width="w-96 max-w-full" size="h-5" />
                    <Skeleton shape=SkeletonShape::Block size="h-64" />
                </SkeletonGroup>
            </PageShell>
        }>
            {move || Suspend::new(async move {
                match home.await {
                    Ok(Home::SignedOut) => view! { <AccountDeletedNote /> <Landing /> }.into_any(),
                    Ok(Home::NoTeam { username, guide }) => view! { <NoTeamHome username=username guide=guide on_change=Callback::new(move |_| home.refetch()) /> }.into_any(),
                    Ok(Home::Team(md, guide)) => view! { <MatchDayView md=*md guide=guide on_change=Callback::new(move |_| home.refetch()) /> }.into_any(),
                    Err(_) => view! {
                        <PageShell>
                            <PageHeader title="Match Day" />
                            <EmptyState title="Match Day could not load" message="The server did not answer. Reload the page in a moment." />
                        </PageShell>
                    }.into_any(),
                }
            })}
        </Suspense>
    }
}

/// After "Delete my account" the profile page sends the browser to
/// `/?account=deleted`; say plainly that it happened.
#[component]
fn AccountDeletedNote() -> impl IntoView {
    let query = leptos_router::hooks::use_query_map();
    let deleted = move || query.read().get("account").as_deref() == Some("deleted");
    view! {
        <Show when=deleted>
            <div class="mx-auto max-w-5xl px-4 pt-8 sm:px-8" data-testid="account-deleted">
                <crate::components::ui::Callout tone=crate::components::ui::CalloutTone::Success title="Your account was deleted">
                    "Your personal data is gone and you are signed out everywhere. Backups drop it within 30 days."
                </crate::components::ui::Callout>
            </div>
        </Show>
    }
}

/// Signed in without a team: where Match Day comes from, and the personal
/// pages that work already.
#[component]
fn NoTeamHome(username: String, guide: HomeGuide, on_change: Callback<()>) -> impl IntoView {
    let has_welcome = guide.welcome.is_some();
    let has_list = guide.checklist.is_some();
    view! {
        <PageShell width=PageWidth::Standard>
            <PageHeader title="Match Day" eyebrow=format!("Signed in as {username}") />
            {guide.welcome.clone().map(|w| view! { <WelcomePanel welcome=w primary=true on_change=on_change /> })}
            {guide.checklist.clone().map(|items| view! {
                <GettingStarted
                    items=items
                    forced=guide.forced
                    emphasis=if has_welcome { PrepEmphasis::Quiet } else { PrepEmphasis::Primary }
                    on_change=on_change
                />
            })}
            {(!has_list).then(|| view! {
                <EmptyState
                    title="Match Day needs a team"
                    level=HeadingLevel::H2
                    message="Match Day walks your team through the next scrim or match: the opponent brief, draft plans, the game plan, open action items and the review. Create a team, use your team's invite link, or ask to join one on the roster page."
                    cta_label="Create or join your team"
                    cta_href="/team/roster"
                />
            })}
            <section class="mt-12" aria-labelledby="home-yours">
                <SectionHeader title="Your own pages" id="home-yours" />
                <ul class="max-w-2xl">
                    {[
                        ("/me", "My games", "Your synced games, ranked standing, LP and champion trends."),
                        ("/personal-learnings", "Learnings", "What you learned, game by game."),
                        ("/champion-pool", "Champion pool", "Your champions by role and tier."),
                        ("/profile", "Profile", "Your name, region and Riot account."),
                    ].into_iter().map(|(href, label, text)| view! {
                        <li class="border-t border-rule py-3 last:border-b">
                            <A href=href attr:class="font-medium text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">{label}</A>
                            <p class="text-label text-muted">{text}</p>
                        </li>
                    }).collect_view()}
                </ul>
            </section>
        </PageShell>
    }
}

#[component]
fn MatchDayView(md: MatchDay, guide: HomeGuide, on_change: Callback<()>) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let editor_open = RwSignal::new(false);
    let steps = prep_steps(&md);
    let next = md.next.clone();
    let title = match &next {
        Some(m) => match &m.opponent_name {
            Some(o) => format!("{} vs {o}", kind_label(&m.kind)),
            None => kind_label(&m.kind).to_string(),
        },
        None => "Match Day".to_string(),
    };
    let viewer_read = next
        .as_ref()
        .is_some_and(|m| m.brief_read_by.contains(&md.viewer_id));
    let match_id = next.as_ref().map(|m| m.id.clone());
    let editor_match = StoredValue::new(next.clone());
    let opponents = StoredValue::new(md.opponents.clone());
    let patch = md.patch.clone();
    let has_match = next.is_some();
    let team_name = md.team_name.clone();
    let nudge = guide.nudge.clone();

    let set_read = Callback::new(move |read: bool| {
        let Some(id) = match_id.clone() else {
            return;
        };
        leptos::task::spawn_local(async move {
            match mark_brief_read(id, read).await {
                Ok(()) => on_change.run(()),
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
        });
    });

    let meta = {
        let patch = patch.clone();
        move || {
            // The start time and format moved into the field card (M152);
            // without a match the meta is the team and the patch.
            let patch = patch.clone().map(|p| {
                view! {
                    <span aria-hidden="true" class="text-muted">"/"</span>
                    <span>
                        "Patch " <span class="figures">{p.version.clone()}</span> " since "
                        <LocalTime iso=p.since.clone() style=TimeStyle::DayMonth />
                    </span>
                }
            });
            view! {
                <span>{team_name.clone()}</span>
                {patch}
            }
        }
    };

    view! {
        <PageShell attr:data-feedback-label="Match Day">
            <PageHeader
                title=title
                eyebrow="Match Day"
                meta=meta
                actions=move || has_match.then(|| view! {
                    <div class="hidden md:block">
                        <Button on_click=Callback::new(move |_| editor_open.set(true)) attr:data-testid="edit-next-match">
                            "Edit match"
                        </Button>
                    </div>
                })
            />
            // Phones may add the first match (the empty state's action) but
            // editing an existing one stays a larger-screen task (DESIGN 6.9).
            {has_match.then(|| view! {
                <p class="-mt-3 mb-6 text-caption text-muted md:hidden" data-testid="phone-read-only">
                    "Read-only on the phone. Edit the match on a larger screen."
                </p>
            })}
            {next.clone().map(|m| view! {
                <FieldCard
                    starts_at=m.starts_at.clone()
                    format=m.format.clone()
                    steps=steps.clone()
                />
            })}
            {guide.welcome.clone().map(|w| view! { <WelcomePanel welcome=w primary=false on_change=on_change /> })}
            // Match Day has its own primary action (the prep sequence's
            // next step or "Add the next match"): the checklist stays quiet.
            {guide.checklist.clone().map(|items| view! {
                <GettingStarted items=items forced=guide.forced emphasis=PrepEmphasis::Quiet on_change=on_change />
            })}

            {match next.clone() {
                None => view! {
                    <EmptyState
                        title="No upcoming match"
                        level=HeadingLevel::H2
                        message="Add one to start the preparation sequence: the opponent brief, draft plans, the game plan, open action items, and the review after the game."
                        action=move || view! {
                            <Button variant=ButtonVariant::Primary size=ButtonSize::Touch on_click=Callback::new(move |_| editor_open.set(true)) attr:data-testid="add-next-match">
                                "Add the next match"
                            </Button>
                        }
                    />
                    {nudge.clone().map(|n| view! { <div class="mt-6"><NudgeLine nudge=n on_change=on_change /></div> })}
                    <div class="mt-10 grid gap-10 lg:grid-cols-3">
                        <OpenItems md=md.clone() />
                        <LastReview md=md.clone() />
                        <Form md=md.clone() />
                    </div>
                }.into_any(),
                Some(m) => view! {
                    <div class="grid gap-x-10 gap-y-10 lg:grid-cols-[minmax(0,1fr)_20rem]">
                        <div class="flex min-w-0 flex-col gap-10">
                            <section aria-labelledby="prep-h" data-prep>
                                <SectionHeader title="Preparation" id="prep-h" meta=format!("{} of 5 done", steps.iter().filter(|s| s.state == PrepState::Done).count()) />
                                // The field card holds the next step as the page's one
                                // primary action: the sequence stays quiet (M152).
                                <PrepSequence label="Preparation for the next match" steps=steps.clone() emphasis=PrepEmphasis::Quiet />
                                {m.notes.clone().map(|n| view! {
                                    <p class="mt-4 max-w-prose text-body text-secondary"><span class="eyebrow mr-2">"Notes"</span>{n}</p>
                                })}
                                {nudge.clone().map(|n| view! { <div class="mt-4"><NudgeLine nudge=n on_change=on_change /></div> })}
                            </section>
                            <Brief brief=md.brief.clone() viewer_read=viewer_read read_by=m.brief_read_by.len() members=md.member_count set_read=set_read />
                        </div>
                        <aside class="flex flex-col gap-8 border-t border-rule pt-8 lg:border-l lg:border-t-0 lg:pl-8 lg:pt-0" aria-label="Team status">
                            <OpenItems md=md.clone() />
                            <LastReview md=md.clone() />
                            <Form md=md.clone() />
                        </aside>
                    </div>
                }.into_any(),
            }}
            <MatchEditor open=editor_open existing=editor_match.get_value() opponents=opponents.get_value() on_saved=on_change />
        </PageShell>
    }
}

/// "Best of 3, with 2 of 5 prep steps done." (R0 T1-7: the title already
/// names the opponent.)
pub fn field_sentence(format: &str, steps: &[PrepStep]) -> String {
    let done = steps.iter().filter(|s| s.state == PrepState::Done).count();
    format!(
        "{}, with {done} of {} prep steps done.",
        format_label(format),
        steps.len()
    )
}

/// Match Day's field card (M152, M151 references.md field card, R0 fixes
/// T1-4 and T1-7): eyebrow, one figure (the start time), one sentence (the
/// format and the prep count, from the team's records), one action (the
/// prep sequence's next step, the page's only primary button) and the 88 px
/// Rift inset, ink only, floated top right with the sentence wrapping
/// beside it.
///
/// No team names beside the inset (recipe card E, critique r1 surfaces
/// ruling 1): a name by a map pins a team to a base the data does not hold
/// (a best of 3 has no sides), and the page title already faces the teams
/// ("Scrim vs Riverside Rooks").
///
/// A11y: one `<h2>` ("Next match"); the figure is a `<time>`; the inset is
/// `aria-hidden` (the page title already says who plays); the action is a
/// real link.
#[component]
fn FieldCard(starts_at: String, format: String, steps: Vec<PrepStep>) -> impl IntoView {
    let sentence = field_sentence(&format, &steps);
    let action = steps
        .iter()
        .find(|s| s.state == PrepState::Next)
        .and_then(|s| Some((s.href.clone()?, s.action.clone()?)));
    view! {
        <Surface layer=Layer::Ink1 edge=Edge::RuleY pad=Pad::Md tag=SurfaceTag::Section labelled_by="field-h" class="mb-10 max-w-[40rem] flow-root" attr:data-field-card="">
            <div class="float-right mb-2 ml-4 w-[88px]" aria-hidden="true" data-field-inset>
                <RiftSketch crop=RiftCrop::Inset ground=RiftGround::Ink1 class="w-[88px]" />
            </div>
            <h2 id="field-h" class="eyebrow">"Next match"</h2>
            <p class="mt-2 flex flex-wrap items-baseline gap-x-3">
                <LocalTime iso=starts_at.clone() style=TimeStyle::Clock class="figures font-medium text-headline text-primary lg:text-display" attr:data-field-figure="" />
                <LocalTime iso=starts_at style=TimeStyle::Day class="text-label text-muted" />
            </p>
            <p class="mt-3 max-w-[60ch] text-reading text-secondary" data-field-sentence>{sentence}</p>
            {action.map(|(href, label)| view! {
                <div class="clear-both mt-5 grid sm:block">
                    <Button href=href variant=ButtonVariant::Primary size=ButtonSize::Touch attr:data-testid="field-next-step">{label}</Button>
                </div>
            })}
        </Surface>
    }
}

/// The opponent brief: the dossier summary with its survey age.
#[component]
fn Brief(
    brief: Option<OpponentBrief>,
    viewer_read: bool,
    read_by: usize,
    members: usize,
    set_read: Callback<bool>,
) -> impl IntoView {
    let Some(b) = brief else {
        return view! {
            <section id="brief" aria-labelledby="brief-h" class="scroll-mt-20">
                <SectionHeader title="Opponent brief" id="brief-h" />
                <EmptyState
                    size=EmptySize::Compact
                    message="No opponent set for this match. Choose their dossier in the match details and the brief appears here: players, what they play, your notes and when they were last surveyed."
                    cta_label="Open the dossiers"
                    cta_href="/opponents"
                />
            </section>
        }
        .into_any();
    };
    let survey = match (&b.surveyed, b.stale) {
        (Some(s), stale) => view! {
            <span class=if stale { "text-warning" } else { "text-muted" }>
                {stale.then(|| view! { <span aria-hidden="true" class="mr-1">"!"</span> })}
                {format!("Last surveyed {s}")}
            </span>
        }
        .into_any(),
        (None, _) => {
            view! { <span class="text-muted">"Not refreshed from Riot yet"</span> }.into_any()
        }
    };
    let dossier_href = format!("/opponents?opponent_id={}", b.id);
    view! {
        <section id="brief" aria-labelledby="brief-h" class="scroll-mt-20" data-brief>
            <SectionHeader
                title=format!("Brief: {}", b.name)
                id="brief-h"
                actions=move || view! {
                    {if viewer_read {
                        view! {
                            <span class="figures text-label text-success" data-testid="brief-read">"Read"</span>
                            <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| set_read.run(false))>"Mark unread"</Button>
                        }.into_any()
                    } else {
                        view! {
                            <Button size=ButtonSize::Sm on_click=Callback::new(move |_| set_read.run(true)) attr:data-testid="mark-brief-read">"Mark as read"</Button>
                        }.into_any()
                    }}
                }
            />
            <p class="-mt-2 mb-4 flex flex-wrap gap-x-3 text-label">
                {survey}
                <span class="text-muted">{format!("Read by {read_by} of {members}")}</span>
                <span class="text-muted">{format!("{} champions seen", b.champions_seen)}</span>
            </p>
            {b.notes.clone().map(|n| view! {
                <p class="mb-5 max-w-[65ch] whitespace-pre-line text-reading text-primary">{n}</p>
            })}
            {if b.players.is_empty() {
                view! {
                    <EmptyState size=EmptySize::Compact message="No players in this dossier yet. Add them on the dossier; their champions appear here after a refresh from Riot." />
                }.into_any()
            } else {
                view! {
                    <ul class="border-b border-rule" aria-label="Their players">
                        {b.players.clone().into_iter().map(|p: BriefPlayer| {
                            let role = Role::parse(&p.role);
                            view! {
                                <li class="grid grid-cols-[3rem_minmax(0,1fr)] items-center gap-3 border-t border-rule py-2.5 sm:grid-cols-[3rem_10rem_minmax(0,1fr)]">
                                    <span>{role.map(|r| view! { <RoleLabel role=r /> })}</span>
                                    <span class="truncate font-medium text-primary">{p.name.clone()}</span>
                                    <span class="col-span-2 flex flex-wrap items-center gap-x-3 gap-y-1 sm:col-span-1">
                                        {if p.champions.is_empty() {
                                            view! { <span class="text-label text-muted">"No champions seen yet"</span> }.into_any()
                                        } else {
                                            p.champions.into_iter().map(|c| view! {
                                                <span class="inline-flex items-center gap-1.5 text-label text-secondary">
                                                    <ChampionAvatar name=c.clone() size=crate::components::ui::AvatarSize::Xs decorative=true />
                                                    {c.clone()}
                                                </span>
                                            }).collect_view().into_any()
                                        }}
                                    </span>
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                }.into_any()
            }}
            <p class="mt-3 text-label">
                <A href=dossier_href attr:class="text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">"Open the full dossier"</A>
            </p>
        </section>
    }
    .into_any()
}

#[component]
fn OpenItems(md: MatchDay) -> impl IntoView {
    let count = md.open_item_count;
    view! {
        <section aria-labelledby="items-h" data-open-items>
            <h2 id="items-h" class="mb-3 flex items-baseline justify-between gap-3 font-display text-heading font-semibold text-primary">
                "Open action items"
                <span class="figures text-label font-normal text-muted">{count}</span>
            </h2>
            {if md.open_items.is_empty() {
                view! { <p class="text-body text-secondary">"Nothing open. Action items from your reviews appear here until they are done."</p> }.into_any()
            } else {
                view! {
                    <ul>
                        {md.open_items.iter().map(|i| view! {
                            <li class="border-t border-rule py-2 last:border-b">
                                <p class="text-body text-primary">{i.text.clone()}</p>
                                <p class="text-caption text-muted">
                                    {i.assigned_to.clone().filter(|a| !a.is_empty()).unwrap_or_else(|| "Whole team".to_string())}
                                    {(i.status == "in_progress").then_some(" \u{00b7} in progress")}
                                </p>
                            </li>
                        }).collect_view()}
                    </ul>
                    <p class="mt-2 text-label">
                        <A href="/action-items" attr:class="text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">
                            {if count > md.open_items.len() { format!("All {count} open items") } else { "Open action items".to_string() }}
                        </A>
                    </p>
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn LastReview(md: MatchDay) -> impl IntoView {
    view! {
        <section aria-labelledby="review-h" data-last-review>
            <h2 id="review-h" class="mb-3 font-display text-heading font-semibold text-primary">"Last review"</h2>
            {match md.last_review {
                None => view! { <p class="text-body text-secondary">"Reviews you write after a game appear here with their main lesson."</p> }.into_any(),
                Some(r) => {
                    let href = format!("/post-game?review_id={}", r.id);
                    view! {
                        <div class="flex items-center gap-3">
                            {match r.won {
                                Some(w) => view! { <ResultGlyph result=GameResult::from_win(w) size=GlyphSize::Sm /> }.into_any(),
                                None => view! { <span class="text-label text-muted">"No result"</span> }.into_any(),
                            }}
                            {r.created_at.clone().map(|c| view! { <LocalTime iso=c style=TimeStyle::DayMonth class="figures text-label text-muted" /> })}
                        </div>
                        <p class="mt-2 text-body text-primary">
                            {r.lesson.clone().map(|l| format!("\u{201c}{l}\u{201d}")).unwrap_or_else(|| "No lesson written down.".to_string())}
                        </p>
                        <p class="mt-2 text-label">
                            <A href=href attr:class="text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">"Open the review"</A>
                        </p>
                    }.into_any()
                }
            }}
        </section>
    }
}

#[component]
fn Form(md: MatchDay) -> impl IntoView {
    let words: Vec<&str> = md
        .form
        .iter()
        .map(|w| if *w { "win" } else { "loss" })
        .collect();
    let label = format!(
        "Last {} reviewed games, oldest to newest: {}",
        md.form.len(),
        words.join(", ")
    );
    view! {
        <section aria-labelledby="form-h" data-form>
            <h2 id="form-h" class="mb-3 font-display text-heading font-semibold text-primary">"Form"</h2>
            {if md.form.is_empty() {
                view! { <p class="text-body text-secondary">"Results of your last five reviewed games appear here. Set win or loss on a review to count it."</p> }.into_any()
            } else {
                view! {
                    <div class="flex items-center gap-1.5" role="img" aria-label=label>
                        <span class="figures mr-1 text-caption text-muted" aria-hidden="true">"old"</span>
                        {md.form.iter().map(|w| view! { <span aria-hidden="true"><ResultGlyph result=GameResult::from_win(*w) size=GlyphSize::Sm /></span> }).collect_view()}
                        <span class="figures ml-1 text-caption text-muted" aria-hidden="true">"new"</span>
                    </div>
                    <p class="mt-2 text-caption text-muted">"From the result set on each review."</p>
                }.into_any()
            }}
        </section>
    }
}

/// The small "next match" editor: kind, date and time (the viewer's time
/// zone), opponent from the dossiers, format, notes.
#[component]
fn MatchEditor(
    open: RwSignal<bool>,
    existing: Option<crate::models::match_day::ScheduledMatch>,
    opponents: Vec<LinkRef>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let id = existing.as_ref().map(|m| m.id.clone());
    let kind = RwSignal::new(
        existing
            .as_ref()
            .map(|m| m.kind.clone())
            .unwrap_or_else(|| "scrim".into()),
    );
    let format = RwSignal::new(
        existing
            .as_ref()
            .map(|m| m.format.clone())
            .unwrap_or_else(|| "bo3".into()),
    );
    let opponent = RwSignal::new(
        existing
            .as_ref()
            .and_then(|m| m.opponent_id.clone())
            .unwrap_or_default(),
    );
    let notes = RwSignal::new(
        existing
            .as_ref()
            .and_then(|m| m.notes.clone())
            .unwrap_or_default(),
    );
    let date = RwSignal::new(String::new());
    let time = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let saving = RwSignal::new(false);
    let confirm_remove = RwSignal::new(false);
    let starts_at = existing.as_ref().map(|m| m.starts_at.clone());
    // Prefill the date and time in the viewer's zone once the editor opens
    // (the server cannot know the zone).
    Effect::new(move |_| {
        if open.get() && date.get_untracked().is_empty() {
            if let Some((d, t)) = starts_at.as_deref().and_then(utc_to_local_inputs) {
                date.set(d);
                time.set(t);
            } else {
                time.set("19:00".into());
            }
        }
    });

    let mut opponent_options = vec![SelectOption::new("", "No opponent yet")];
    opponent_options.extend(
        opponents
            .iter()
            .map(|o| SelectOption::new(o.id.clone(), o.name.clone())),
    );
    let no_dossiers = opponents.is_empty();
    let opponent_options = StoredValue::new(opponent_options);
    let id_for_save = id.clone();
    let save = Callback::new(move |_| {
        let Some(starts) = local_input_to_utc(&date.get_untracked(), &time.get_untracked()) else {
            error.set(Some("Enter a date and a time.".into()));
            return;
        };
        let input = ScheduledMatchInput {
            id: id_for_save.clone(),
            starts_at: starts,
            kind: kind.get_untracked(),
            format: format.get_untracked(),
            opponent_id: Some(opponent.get_untracked()).filter(|o| !o.is_empty()),
            notes: Some(notes.get_untracked()).filter(|n| !n.trim().is_empty()),
        };
        let Ok(json) = serde_json::to_string(&input) else {
            return;
        };
        saving.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match save_next_match(json).await {
                Ok(_) => {
                    open.set(false);
                    toast.push.run(Toast::success("Match saved"));
                    on_saved.run(());
                }
                Err(e) => error.set(Some(crate::models::utils::error_text(&e))),
            }
            saving.set(false);
        });
    });
    let id_for_remove = id.clone();
    let remove = Callback::new(move |_| {
        let Some(mid) = id_for_remove.clone() else {
            return;
        };
        leptos::task::spawn_local(async move {
            match remove_next_match(mid).await {
                Ok(()) => {
                    open.set(false);
                    toast.push.run(Toast::success("Match removed"));
                    on_saved.run(());
                }
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
        });
    });
    let editing = id.is_some();
    view! {
        <Dialog
            id="match-editor"
            open=open
            title=if editing { "Edit the next match" } else { "Add the next match" }
            description="Times are in your own time zone."
            dismiss_on_scrim=false
            footer=move || view! {
                {editing.then(|| view! {
                    <div class="mr-auto">
                        <Button variant=ButtonVariant::Danger size=ButtonSize::Sm on_click=Callback::new(move |_| confirm_remove.set(true))>"Remove match"</Button>
                    </div>
                })}
                <Button variant=ButtonVariant::Quiet size=ButtonSize::Touch on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                <Button variant=ButtonVariant::Primary size=ButtonSize::Touch loading=saving on_click=save attr:data-testid="save-next-match">"Save match"</Button>
            }
        >
            <div class="flex flex-col gap-4">
                <SegmentedControl id="match-kind" label="Kind of match" options=KINDS.iter().map(|(k, l)| Segment::new(*k, *l)).collect() value=kind />
                <div class="grid grid-cols-2 gap-3">
                    <Field id="match-date" label="Date" required=true>
                        <TextField kind=InputKind::Date value=date required=true />
                    </Field>
                    <Field id="match-time" label="Time" required=true>
                        <TextField kind=InputKind::Time value=time required=true />
                    </Field>
                </div>
                <Field id="match-opponent" label="Opponent" hint=if no_dossiers { "No dossiers yet: add the opponent on the Opponents page first.".to_string() } else { "Their dossier becomes the brief.".to_string() }>
                    <Select value=opponent options=opponent_options.get_value() />
                </Field>
                // Without a dossier the hint alone was a dead end: offer the
                // way there (a match without an opponent can still be saved).
                {no_dossiers.then(|| view! {
                    <div>
                        <Button href="/opponents" variant=ButtonVariant::Secondary size=ButtonSize::Touch attr:data-testid="add-opponent-link">
                            "Add an opponent"
                        </Button>
                    </div>
                })}
                <SegmentedControl id="match-format" label="Format" options=FORMATS.iter().map(|(k, l)| Segment::new(*k, *l)).collect() value=format />
                <Field id="match-notes" label="Notes" hint="Where, who hosts, anything the team should know (500 characters).">
                    <TextField value=notes rows=3 maxlength=500 />
                </Field>
                {move || error.get().map(|e| view! { <p role="alert" class="text-label text-danger">{e}</p> })}
            </div>
        </Dialog>
        <ConfirmDialog
            id="remove-match"
            open=confirm_remove
            title="Remove this match?"
            message="Match Day goes back to empty until the next match is added. Drafts, plans and reviews stay."
            confirm_label="Remove match"
            tone=ConfirmTone::Destructive
            on_confirm=remove
        />
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::match_day::ScheduledMatch;

    fn md(starts_in_hours: i64) -> MatchDay {
        let now = chrono::Utc::now();
        MatchDay {
            team_name: "Northgate".into(),
            member_count: 5,
            viewer_id: "user:me".into(),
            next: Some(ScheduledMatch {
                id: "scheduled_match:1".into(),
                starts_at: (now + chrono::Duration::hours(starts_in_hours)).to_rfc3339(),
                kind: "scrim".into(),
                format: "bo3".into(),
                opponent_id: Some("opponent:o".into()),
                opponent_name: Some("Rooks".into()),
                notes: None,
                brief_read_by: vec![],
            }),
            brief: Some(OpponentBrief {
                id: "opponent:o".into(),
                name: "Rooks".into(),
                notes: None,
                players: vec![],
                champions_seen: 0,
                surveyed: None,
                stale: false,
            }),
            drafts: vec![],
            plans: vec![],
            review_after: None,
            open_items: vec![],
            open_item_count: 0,
            last_review: None,
            form: vec![],
            patch: None,
            opponents: vec![],
            now: now.to_rfc3339(),
        }
    }

    fn states(steps: &[PrepStep]) -> Vec<PrepState> {
        steps.iter().map(|s| s.state).collect()
    }

    #[test]
    fn the_field_sentence_counts_done_steps_from_the_records() {
        let mut m = md(5);
        // Format bo3, only "Open action items" done.
        assert_eq!(
            field_sentence("bo3", &prep_steps(&m)),
            "Best of 3, with 1 of 5 prep steps done."
        );
        m.next
            .as_mut()
            .unwrap()
            .brief_read_by
            .push("user:me".into());
        assert_eq!(
            field_sentence("bo3", &prep_steps(&m)),
            "Best of 3, with 2 of 5 prep steps done."
        );
    }

    #[test]
    fn no_match_no_steps() {
        let mut m = md(5);
        m.next = None;
        assert!(prep_steps(&m).is_empty());
    }

    #[test]
    fn the_sequence_advances_as_artefacts_exist() {
        use PrepState::*;
        let mut m = md(5);
        assert_eq!(states(&prep_steps(&m)), vec![Next, Todo, Todo, Done, Todo]);
        m.next
            .as_mut()
            .unwrap()
            .brief_read_by
            .push("user:me".into());
        assert_eq!(states(&prep_steps(&m)), vec![Done, Next, Todo, Done, Todo]);
        m.drafts.push(LinkRef {
            id: "draft:d".into(),
            name: "D".into(),
        });
        let s = prep_steps(&m);
        assert_eq!(states(&s), vec![Done, Done, Next, Done, Todo]);
        assert_eq!(s[2].href.as_deref(), Some("/game-plan?draft_id=draft:d"));
        m.plans.push(LinkRef {
            id: "game_plan:p".into(),
            name: "P".into(),
        });
        m.open_item_count = 2;
        let s = prep_steps(&m);
        assert_eq!(states(&s), vec![Done, Done, Done, Next, Todo]);
        assert_eq!(s[3].status.as_deref(), Some("2 open"));
        m.open_item_count = 0;
        // Before the match nothing is left to do now; the review waits.
        assert_eq!(states(&prep_steps(&m)), vec![Done, Done, Done, Done, Todo]);
    }

    #[test]
    fn the_review_is_next_once_the_match_started() {
        use PrepState::*;
        let mut m = md(-1);
        m.next
            .as_mut()
            .unwrap()
            .brief_read_by
            .push("user:me".into());
        m.drafts.push(LinkRef {
            id: "draft:d".into(),
            name: "D".into(),
        });
        m.plans.push(LinkRef {
            id: "game_plan:p".into(),
            name: "P".into(),
        });
        let s = prep_steps(&m);
        assert_eq!(states(&s), vec![Done, Done, Done, Done, Next]);
        assert_eq!(s[4].href.as_deref(), Some("/post-game?plan_id=game_plan:p"));
        m.review_after = Some(LinkRef {
            id: "post_game_learning:r".into(),
            name: "R".into(),
        });
        assert_eq!(states(&prep_steps(&m)), vec![Done, Done, Done, Done, Done]);
    }

    #[test]
    fn a_stale_dossier_needs_attention_and_no_opponent_points_at_dossiers() {
        let mut m = md(5);
        m.brief.as_mut().unwrap().stale = true;
        m.brief.as_mut().unwrap().surveyed = Some("9 days ago".into());
        let s = prep_steps(&m);
        assert_eq!(s[0].state, PrepState::Next);
        assert!(s[0].detail.as_deref().unwrap().contains("refresh"));
        m.brief = None;
        let s = prep_steps(&m);
        assert_eq!(s[0].href.as_deref(), Some("/opponents"));
        // Without an opponent, drafts cannot be counted, so there is no link.
        assert!(s[1].href.is_none());
    }
}
