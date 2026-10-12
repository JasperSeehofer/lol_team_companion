//! Share links (M5): a read-only link to one game plan, review or player
//! card. The link carries a random token; it opens without signing in,
//! expires (a day, a week or a month) and can be revoked. The public page
//! shows only the shared artefact: no navigation into the team, no ids of
//! other records.

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::local_time::{LocalTime, TimeStyle};
use crate::components::ui::{
    AvatarSize, Badge, BadgeTone, Button, ButtonSize, ButtonVariant, Callout, CalloutTone,
    ChampionAvatar, Dialog, EmptyState, Field, GameResult, GlyphSize, HeadingLevel, PageHeader,
    PageShell, PageWidth, ResultGlyph, Role, RoleLabel, SectionHeader, Select, SelectOption,
    Skeleton, SkeletonGroup, Toast, ToastContext,
};
use crate::models::share::{
    share_path, CardChampion, PlayerCard, ShareKind, ShareLink, Shared, SharedPlan, SharedReview,
    SharedView, SHARE_DAYS,
};

/// Create a link to one of the team's plans, reviews or members.
#[server]
pub async fn create_share(
    kind: String,
    target_id: String,
    days: u32,
) -> Result<ShareLink, ServerFnError> {
    use crate::server::{authz, db, rate_limit::Bucket};
    let kind = ShareKind::parse(&kind).ok_or_else(|| ServerFnError::new("Unknown link kind"))?;
    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    authz::rate_limit(Bucket::Share).await?;
    db::create_share_link(&ctx.db, &team_id, &ctx.user.id, kind, &target_id, days)
        .await
        .map_err(authz::server_err)
}

/// The team's live links to one target.
#[server]
pub async fn list_shares(kind: String, target_id: String) -> Result<Vec<ShareLink>, ServerFnError> {
    use crate::server::{authz, db};
    let kind = ShareKind::parse(&kind).ok_or_else(|| ServerFnError::new("Unknown link kind"))?;
    let ctx = authz::authed().await?;
    let Some(team_id) = ctx.team().await? else {
        return Ok(Vec::new());
    };
    db::list_share_links(&ctx.db, &team_id, kind, &target_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn revoke_share(link_id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};
    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::revoke_share_link(&ctx.db, &team_id, &link_id)
        .await
        .map_err(authz::server_err)
}

/// What a token opens (public, rate limited per IP). Unknown, revoked and
/// expired tokens, and targets deleted since, are all `None`.
#[server]
pub async fn get_shared(token: String) -> Result<Option<Shared>, ServerFnError> {
    use crate::server::{authz, db, rate_limit::Bucket};
    authz::rate_limit(Bucket::Share).await?;
    let db = authz::db()?;
    let Some(link) = db::resolve_share_token(&db, &token)
        .await
        .map_err(authz::server_err)?
    else {
        return Ok(None);
    };
    let view = match link.kind {
        ShareKind::GamePlan => {
            let Some(p) = db::get_game_plan(&db, &link.team_id, &link.target_id)
                .await
                .map_err(authz::server_err)?
            else {
                return Ok(None);
            };
            let briefs = [
                (Role::Top, &p.top_strategy),
                (Role::Jungle, &p.jungle_strategy),
                (Role::Mid, &p.mid_strategy),
                (Role::Bot, &p.bot_strategy),
                (Role::Support, &p.support_strategy),
            ]
            .into_iter()
            .filter_map(|(r, b)| {
                b.as_ref()
                    .filter(|t| !t.trim().is_empty())
                    .map(|t| (r.key().to_string(), t.clone()))
            })
            .collect();
            SharedView::GamePlan(SharedPlan {
                name: if p.name.trim().is_empty() {
                    "Game plan".into()
                } else {
                    p.name
                },
                our_champions: p
                    .our_champions
                    .into_iter()
                    .filter(|c| !c.is_empty())
                    .collect(),
                enemy_champions: p
                    .enemy_champions
                    .into_iter()
                    .filter(|c| !c.is_empty())
                    .collect(),
                win_conditions: p
                    .win_conditions
                    .into_iter()
                    .filter(|c| !c.trim().is_empty())
                    .collect(),
                objective_priority: p
                    .objective_priority
                    .into_iter()
                    .filter(|c| !c.trim().is_empty())
                    .collect(),
                teamfight_strategy: p.teamfight_strategy,
                early_game: p.early_game.filter(|t| !t.trim().is_empty()),
                role_briefs: briefs,
                notes: p.notes.filter(|t| !t.trim().is_empty()),
            })
        }
        ShareKind::Review => {
            let Some(r) = db::get_post_game_learning(&db, &link.team_id, &link.target_id)
                .await
                .map_err(authz::server_err)?
            else {
                return Ok(None);
            };
            SharedView::Review(SharedReview {
                won: r.win_loss.as_deref().and_then(|w| match w {
                    "win" => Some(true),
                    "loss" => Some(false),
                    _ => None,
                }),
                rating: r.rating,
                what_went_well: r
                    .what_went_well
                    .into_iter()
                    .filter(|l| !l.trim().is_empty())
                    .collect(),
                improvements: r
                    .improvements
                    .into_iter()
                    .filter(|l| !l.trim().is_empty())
                    .collect(),
                action_items: r
                    .action_items
                    .into_iter()
                    .filter(|l| !l.trim().is_empty())
                    .collect(),
                notes: r.open_notes.filter(|t| !t.trim().is_empty()),
                created_at: r.created_at,
            })
        }
        ShareKind::Player => match db::player_card(&db, &link.team_id, &link.target_id).await {
            // A player who hid their card: older links open nothing.
            Ok(card) if card.share_hidden => return Ok(None),
            Ok(card) => SharedView::Player(card),
            Err(db::DbError::NotFound) => return Ok(None),
            Err(e) => return Err(authz::server_err(e)),
        },
    };
    Ok(Some(Shared {
        view,
        expires_at: link.expires_at,
    }))
}

fn days_label(d: u32) -> &'static str {
    match d {
        1 => "One day",
        7 => "One week",
        _ => "One month",
    }
}

/// The absolute URL of a share path, in the browser (the path alone on the
/// server, which does not know the public host).
fn absolute(path: &str) -> String {
    #[cfg(feature = "hydrate")]
    if let Some(origin) = web_sys::window().and_then(|w| w.location().origin().ok()) {
        return format!("{origin}{path}");
    }
    path.to_string()
}

/// Make, copy and revoke read-only links to one artefact.
///
/// A11y: a `Dialog` (focus trapped, Escape closes); the live links are a
/// list, each URL in a read-only text field with a "Copy link" button whose
/// result is announced in a toast; "Revoke" names the link it ends.
#[component]
pub fn ShareDialog(
    open: RwSignal<bool>,
    kind: ShareKind,
    /// The plan, review or member id; `None` while the record is unsaved.
    #[prop(into)]
    target: Signal<Option<String>>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let days = RwSignal::new("7".to_string());
    let links = Resource::new(
        move || (open.get(), target.get()),
        move |(o, t)| async move {
            match (o, t) {
                (true, Some(t)) => list_shares(kind.key().to_string(), t)
                    .await
                    .unwrap_or_default(),
                _ => Vec::new(),
            }
        },
    );
    let creating = RwSignal::new(false);
    let create = Callback::new(move |_| {
        let Some(t) = target.get_untracked() else {
            return;
        };
        let d: u32 = days.get_untracked().parse().unwrap_or(7);
        creating.set(true);
        leptos::task::spawn_local(async move {
            match create_share(kind.key().to_string(), t, d).await {
                Ok(_) => {
                    toast.push.run(Toast::success("Link created"));
                    links.refetch();
                }
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
            creating.set(false);
        });
    });
    let revoke = Callback::new(move |id: String| {
        leptos::task::spawn_local(async move {
            match revoke_share(id).await {
                Ok(()) => {
                    toast
                        .push
                        .run(Toast::success("Link revoked: it opens nothing now"));
                    links.refetch();
                }
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
        });
    });
    let copy = Callback::new(move |url: String| {
        #[cfg(feature = "hydrate")]
        if let Some(w) = web_sys::window() {
            let _ = w.navigator().clipboard().write_text(&url);
            toast.push.run(Toast::success("Link copied"));
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = url;
    });
    let options: Vec<SelectOption> = SHARE_DAYS
        .iter()
        .map(|d| SelectOption::new(d.to_string(), days_label(*d)))
        .collect();
    let options = StoredValue::new(options);
    view! {
        <Dialog
            id="share-dialog"
            open=open
            title=format!("Share this {}", kind.noun())
            description="Anyone with the link can read it without signing in, until it expires or you revoke it. It shows nothing else of the team."
            footer=move || view! { <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| open.set(false))>"Done"</Button> }
        >
            <div class="flex flex-col gap-5">
                <div class="flex flex-wrap items-end gap-3">
                    <Field id="share-days" label="Link lasts">
                        <Select value=days options=options.get_value() />
                    </Field>
                    <Button on_click=create loading=creating attr:data-testid="create-share-link">"Create link"</Button>
                </div>
                <Suspense fallback=|| view! { <SkeletonGroup label="Loading links"><Skeleton lines=2 /></SkeletonGroup> }>
                    {move || links.get().map(|list| {
                        if list.is_empty() {
                            view! { <p class="text-body text-secondary">"No live links. Links you create appear here with their expiry."</p> }.into_any()
                        } else {
                            view! {
                                <ul class="border-b border-rule" aria-label="Live links">
                                    {list.into_iter().map(|l| {
                                        let url = absolute(&share_path(&l.token));
                                        let url_copy = url.clone();
                                        let id = l.id.clone();
                                        view! {
                                            <li class="flex flex-col gap-2 border-t border-rule py-3" data-share-link=l.token.clone()>
                                                <p class="select-all break-all font-mono text-caption text-primary" data-share-url>{url}</p>
                                                <div class="flex flex-wrap items-center justify-between gap-2 text-caption text-muted">
                                                    <span>
                                                        {format!("By {} \u{00b7} expires ", l.created_by_name)}
                                                        <LocalTime iso=l.expires_at.clone() />
                                                    </span>
                                                    <span class="flex gap-1">
                                                        <Button size=ButtonSize::Sm on_click=Callback::new(move |_| copy.run(url_copy.clone()))>"Copy link"</Button>
                                                        <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| revoke.run(id.clone()))>"Revoke"</Button>
                                                    </span>
                                                </div>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </div>
        </Dialog>
    }
}

/// The public page behind `/s/{token}`.
#[component]
pub fn SharePage() -> impl IntoView {
    let params = use_params_map();
    let token = move || params.read().get("token").unwrap_or_default();
    let shared = Resource::new(token, get_shared);
    view! {
        // A shared link is for the people it was sent to: keep it out of
        // search engines.
        <leptos_meta::Meta name="robots" content="noindex, nofollow" />
        <PageShell width=PageWidth::Reading attr:data-shared-page>
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading the shared page"><Skeleton lines=6 /></SkeletonGroup> }>
                {move || shared.get().map(|r| match r {
                    Ok(Some(s)) => view! { <SharedBody shared=s /> }.into_any(),
                    Ok(None) => view! {
                        <PageHeader title="This link does not open anything" eyebrow="Shared link" />
                        <EmptyState message="It may have expired or been revoked, or it was copied incompletely. Ask the team that sent it for a new link." />
                    }.into_any(),
                    Err(e) => view! {
                        <PageHeader title="This link could not be opened" eyebrow="Shared link" />
                        <Callout tone=CalloutTone::Warning>{crate::models::utils::error_text(&e)}</Callout>
                    }.into_any(),
                })}
            </Suspense>
        </PageShell>
    }
}

#[component]
fn SharedBody(shared: Shared) -> impl IntoView {
    let expiry = shared.expires_at.clone();
    let note = move || {
        view! {
            <p class="mb-8 mt-10 text-caption text-muted" data-shared-note>
                "Read-only. Shared with a link that expires " <LocalTime iso=expiry.clone() style=TimeStyle::When /> "."
            </p>
        }
    };
    match shared.view {
        SharedView::GamePlan(p) => view! { <SharedPlanView plan=p /> {note()} }.into_any(),
        SharedView::Review(r) => view! { <SharedReviewView review=r /> {note()} }.into_any(),
        SharedView::Player(c) => view! { <PageHeader title=c.username.clone() eyebrow="Player card" /> <PlayerCardView card=c /> {note()} }.into_any(),
    }
}

fn champ_row(list: Vec<String>, empty: &'static str) -> AnyView {
    if list.is_empty() {
        return view! { <p class="text-body text-muted">{empty}</p> }.into_any();
    }
    view! {
        <ul class="flex flex-wrap gap-x-4 gap-y-2">
            {list.into_iter().map(|c| view! {
                <li class="inline-flex items-center gap-1.5 text-body text-primary">
                    <ChampionAvatar name=c.clone() size=AvatarSize::Sm decorative=true />
                    {c}
                </li>
            }).collect_view()}
        </ul>
    }
    .into_any()
}

fn line_list(list: Vec<String>) -> AnyView {
    view! {
        <ul class="flex flex-col gap-1.5">
            {list.into_iter().map(|l| view! { <li class="text-reading text-primary">{l}</li> }).collect_view()}
        </ul>
    }
    .into_any()
}

#[component]
fn SharedPlanView(plan: SharedPlan) -> impl IntoView {
    view! {
        <PageHeader title=plan.name.clone() eyebrow="Game plan" />
        <div class="flex flex-col gap-10">
            <section aria-labelledby="sp-comp">
                <SectionHeader title="Champions" id="sp-comp" />
                <h3 class="mb-2 eyebrow">"Ours"</h3>
                {champ_row(plan.our_champions, "Not set.")}
                <h3 class="mb-2 mt-4 eyebrow">"Theirs"</h3>
                {champ_row(plan.enemy_champions, "Not set.")}
            </section>
            {(!plan.win_conditions.is_empty()).then(|| view! {
                <section aria-labelledby="sp-win"><SectionHeader title="How we win" id="sp-win" />{line_list(plan.win_conditions.clone())}</section>
            })}
            {(!plan.objective_priority.is_empty()).then(|| view! {
                <section aria-labelledby="sp-obj"><SectionHeader title="Objectives" id="sp-obj" />{line_list(plan.objective_priority.clone())}</section>
            })}
            {(!plan.teamfight_strategy.trim().is_empty()).then(|| view! {
                <section aria-labelledby="sp-tf"><SectionHeader title="Teamfights" id="sp-tf" /><p class="whitespace-pre-line text-reading text-primary">{plan.teamfight_strategy.clone()}</p></section>
            })}
            {plan.early_game.clone().map(|e| view! {
                <section aria-labelledby="sp-early"><SectionHeader title="Early game" id="sp-early" /><p class="whitespace-pre-line text-reading text-primary">{e}</p></section>
            })}
            {(!plan.role_briefs.is_empty()).then(|| view! {
                <section aria-labelledby="sp-roles">
                    <SectionHeader title="Role briefs" id="sp-roles" />
                    <ul class="border-b border-rule">
                        {plan.role_briefs.clone().into_iter().map(|(r, b)| view! {
                            <li class="grid grid-cols-[3rem_minmax(0,1fr)] gap-3 border-t border-rule py-2.5">
                                <span>{Role::parse(&r).map(|r| view! { <RoleLabel role=r /> })}</span>
                                <span class="whitespace-pre-line text-reading text-primary">{b}</span>
                            </li>
                        }).collect_view()}
                    </ul>
                </section>
            })}
            {plan.notes.clone().map(|n| view! {
                <section aria-labelledby="sp-notes"><SectionHeader title="Notes" id="sp-notes" /><p class="whitespace-pre-line text-reading text-primary">{n}</p></section>
            })}
        </div>
    }
}

#[component]
fn SharedReviewView(review: SharedReview) -> impl IntoView {
    let created = review.created_at.clone();
    view! {
        <PageHeader
            title="Game review"
            eyebrow="Review"
            meta=move || view! {
                {review.won.map(|w| view! { <ResultGlyph result=GameResult::from_win(w) word=true /> })}
                {review.rating.map(|r| view! { <span class="figures">{format!("Plan rated {r} of 5")}</span> })}
                {created.clone().map(|c| view! { <LocalTime iso=c style=TimeStyle::When /> })}
            }
        />
        <div class="flex flex-col gap-10">
            <section aria-labelledby="sr-well"><SectionHeader title="What went well" id="sr-well" />
                {if review.what_went_well.is_empty() { view! { <p class="text-body text-muted">"Nothing written."</p> }.into_any() } else { line_list(review.what_went_well.clone()) }}
            </section>
            <section aria-labelledby="sr-improve"><SectionHeader title="To improve" id="sr-improve" />
                {if review.improvements.is_empty() { view! { <p class="text-body text-muted">"Nothing written."</p> }.into_any() } else { line_list(review.improvements.clone()) }}
            </section>
            {(!review.action_items.is_empty()).then(|| view! {
                <section aria-labelledby="sr-items"><SectionHeader title="Action items" id="sr-items" />{line_list(review.action_items.clone())}</section>
            })}
            {review.notes.clone().map(|n| view! {
                <section aria-labelledby="sr-notes"><SectionHeader title="Notes" id="sr-notes" /><p class="whitespace-pre-line text-reading text-primary">{n}</p></section>
            })}
        </div>
    }
}

/// "Comfort", "Match ready" ...
pub fn tier_words(tier: &str) -> &'static str {
    match tier {
        "comfort" => "Comfort",
        "match_ready" => "Match ready",
        "scrim_ready" => "Scrim ready",
        "practicing" => "Practising",
        "to_practice" | "to_practise" => "To practise",
        _ => "In the pool",
    }
}

fn place_words(place: &str) -> &'static str {
    match place {
        "starter" => "Starting five",
        "substitute" => "Substitute",
        "coach" => "Coach",
        _ => "Not placed yet",
    }
}

/// A player card: position and place, best pool champions, recent form,
/// most played champions in stored games.
///
/// A11y: plain sections with headings; the form row is one image with the
/// results in words ("Last 5 games, oldest to newest: win, loss, ...").
#[component]
pub fn PlayerCardView(card: PlayerCard) -> impl IntoView {
    let role = Role::parse(&card.role);
    let words: Vec<&str> = card
        .form
        .iter()
        .map(|w| if *w { "win" } else { "loss" })
        .collect();
    let form_label = format!(
        "Last {} games, oldest to newest: {}",
        card.form.len(),
        words.join(", ")
    );
    view! {
        <div class="flex flex-col gap-10" data-player-card>
            <p class="flex flex-wrap items-center gap-3 text-body text-secondary">
                {role.map(|r| view! { <RoleLabel role=r /> })}
                <span>{place_words(&card.place)}</span>
                <span aria-hidden="true" class="text-muted">"/"</span>
                <span>{card.team_name.clone()}</span>
            </p>
            <section aria-labelledby="pc-pool">
                <SectionHeader title="Champion pool" id="pc-pool" meta="Best tiers first" />
                {if card.champions.is_empty() {
                    view! { <p class="text-body text-muted">"No champions in the pool yet."</p> }.into_any()
                } else {
                    view! {
                        <ul class="border-b border-rule">
                            {card.champions.clone().into_iter().map(|c: CardChampion| view! {
                                <li class="flex items-center justify-between gap-3 border-t border-rule py-2">
                                    <span class="inline-flex items-center gap-2 text-body text-primary">
                                        <ChampionAvatar name=c.champion.clone() size=AvatarSize::Sm decorative=true />
                                        {c.champion.clone()}
                                    </span>
                                    <Badge tone=BadgeTone::Neutral>{tier_words(&c.tier)}</Badge>
                                </li>
                            }).collect_view()}
                        </ul>
                    }.into_any()
                }}
            </section>
            <section aria-labelledby="pc-form">
                <SectionHeader title="Form" id="pc-form" meta="Stored games" level=HeadingLevel::H2 />
                {if card.form.is_empty() {
                    view! { <p class="text-body text-muted">"No games stored yet. Games appear after the player links a Riot account and syncs."</p> }.into_any()
                } else {
                    view! {
                        <div class="flex items-center gap-1.5" role="img" aria-label=form_label.clone()>
                            {card.form.iter().map(|w| view! { <span aria-hidden="true"><ResultGlyph result=GameResult::from_win(*w) size=GlyphSize::Sm /></span> }).collect_view()}
                        </div>
                        {(!card.most_played.is_empty()).then(|| view! {
                            <ul class="mt-4 border-b border-rule">
                                {card.most_played.clone().into_iter().map(|(c, g, w)| view! {
                                    <li class="flex items-center justify-between gap-3 border-t border-rule py-2 text-body">
                                        <span class="inline-flex items-center gap-2 text-primary">
                                            <ChampionAvatar name=c.clone() size=AvatarSize::Xs decorative=true />
                                            {c.clone()}
                                        </span>
                                        <span class="figures text-label text-muted">{format!("{g} games, {w} won")}</span>
                                    </li>
                                }).collect_view()}
                            </ul>
                        })}
                    }.into_any()
                }}
            </section>
        </div>
    }
}

/// The team's own view of a member's card, with sharing.
#[server]
pub async fn get_player_card(user_id: String) -> Result<PlayerCard, ServerFnError> {
    use crate::server::{authz, db};
    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::player_card(&ctx.db, &team_id, &user_id)
        .await
        .map_err(authz::server_err)
}

/// `/team/player/{user id}`: a teammate's card, shareable read-only.
#[component]
pub fn PlayerCardPage() -> impl IntoView {
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });
    let params = use_params_map();
    let user_id = move || params.read().get("id").unwrap_or_default();
    let card = Resource::new(user_id, get_player_card);
    let share_open = RwSignal::new(false);
    let target = Signal::derive(move || Some(user_id()).filter(|u| !u.is_empty()));
    view! {
        <PageShell width=PageWidth::Reading attr:data-feedback-label="Player card">
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading the player card"><Skeleton lines=6 /></SkeletonGroup> }>
                {move || card.get().map(|r| match r {
                    Ok(c) => {
                        let hidden = c.share_hidden;
                        view! {
                        <PageHeader
                            title=c.username.clone()
                            eyebrow="Player card"
                            meta=move || hidden.then(|| view! { <span data-testid="card-not-shareable">"Not shareable: this player keeps their card out of share links."</span> })
                            actions=move || view! {
                                <Button href="/team/dashboard" variant=ButtonVariant::Quiet>"Team page"</Button>
                                {(!hidden).then(|| view! {
                                    <Button on_click=Callback::new(move |_| share_open.set(true)) attr:data-testid="share-player-card">"Share"</Button>
                                })}
                            }
                        />
                        <PlayerCardView card=c />
                    }.into_any()
                    }
                    Err(_) => view! {
                        <PageHeader title="Player card" />
                        <EmptyState message="This player is not on your team, or the card did not load. Cards exist for every member of your team; open one from the team page." cta_label="Team page" cta_href="/team/dashboard" />
                    }.into_any(),
                })}
            </Suspense>
            <ShareDialog open=share_open kind=ShareKind::Player target=target />
        </PageShell>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiers_and_places_read_as_words() {
        assert_eq!(tier_words("match_ready"), "Match ready");
        assert_eq!(tier_words("x"), "In the pool");
        assert_eq!(place_words("substitute"), "Substitute");
        assert_eq!(days_label(30), "One month");
    }
}
