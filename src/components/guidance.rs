//! Guidance in the page (M9): the shell's guidance state, first-use hints,
//! new-feature introductions with their "New" marks, the command-bar line,
//! and Match Day's welcome, getting-started checklist and nudge line.
//!
//! Rules (`docs/DESIGN.md` 6.11): guidance is inline, never a modal or an
//! overlay, never blocks an action; at most one hint or introduction per
//! page and at most one nudge on Match Day; everything can be muted, and
//! the profile's "Guidance" switch turns all of it off. Admin pages, the
//! gallery, sign-in, legal pages and share links get none
//! (`models::guidance::is_quiet_path`).
//!
//! Hints, introductions and "New" marks render after hydration from a
//! browser-side resource (they depend on the page the user is on and on
//! what they just did); Match Day's guidance is server-rendered with the
//! page (`pages::home::get_home`).

use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, Callout, CalloutTone, PrepEmphasis, PrepSequence, PrepState,
    PrepStep, SectionHeader, Switch, Toast, ToastContext,
};
use crate::models::guidance::{self, ChecklistItem, GuidanceState, Hint, Intro, Nudge, Welcome};

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

/// The signed-in user's hints and introductions (empty when signed out or
/// with guidance off).
#[server]
pub async fn get_guidance() -> Result<GuidanceState, ServerFnError> {
    use crate::server::rate_limit::Bucket;
    use crate::server::{authz, guidance};
    let Some(user) = authz::optional_user().await? else {
        return Ok(GuidanceState::default());
    };
    if authz::rate_limit_user(Bucket::GuidanceCheck, &user).is_err() {
        return Ok(GuidanceState::default());
    }
    let db = authz::db()?;
    guidance::state(&db, &user.id, chrono::Utc::now())
        .await
        .map_err(authz::server_err)
}

/// Record that the user saw (`mute` false) or turned off (`mute` true) one
/// guidance item. Only keys guidance knows are accepted.
#[server]
pub async fn mark_guidance(key: String, mute: bool) -> Result<(), ServerFnError> {
    use crate::server::rate_limit::Bucket;
    use crate::server::{authz, guidance};
    let ctx = authz::authed().await?;
    authz::rate_limit_user(Bucket::Guidance, &ctx.user)?;
    let intros = crate::server::community::changelog::intros();
    if !crate::models::guidance::is_known_key(&key, intros) {
        return Err(ServerFnError::new("Unknown guidance item."));
    }
    let what = if mute {
        guidance::Mark::Muted
    } else {
        guidance::Mark::Seen
    };
    guidance::mark(&ctx.db, &ctx.user.id, &key, what)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_guidance_enabled() -> Result<bool, ServerFnError> {
    use crate::server::{authz, guidance};
    let ctx = authz::authed().await?;
    guidance::enabled(&ctx.db, &ctx.user.id)
        .await
        .map_err(authz::server_err)
}

/// The profile's "Guidance" switch.
#[server]
pub async fn set_guidance_enabled(on: bool) -> Result<(), ServerFnError> {
    use crate::server::rate_limit::Bucket;
    use crate::server::{authz, guidance};
    let ctx = authz::authed().await?;
    authz::rate_limit_user(Bucket::Guidance, &ctx.user)?;
    guidance::set_enabled(&ctx.db, &ctx.user.id, on)
        .await
        .map_err(authz::server_err)
}

/// "Show the checklist again" (`checklist`), "Show hints again" (`hints`) or
/// "Show nudges again" (`nudges`: every muted nudge kind).
#[server]
pub async fn reset_guidance(what: String) -> Result<(), ServerFnError> {
    use crate::server::rate_limit::Bucket;
    use crate::server::{authz, guidance};
    let ctx = authz::authed().await?;
    authz::rate_limit_user(Bucket::Guidance, &ctx.user)?;
    let what = match what.as_str() {
        "hints" => guidance::Reset::Hints,
        "checklist" => guidance::Reset::Checklist,
        "nudges" => guidance::Reset::Nudges,
        _ => return Err(ServerFnError::new("Unknown guidance item.")),
    };
    guidance::reset(&ctx.db, &ctx.user.id, what)
        .await
        .map_err(authz::server_err)
}

// ---------------------------------------------------------------------------
// The shell's guidance state
// ---------------------------------------------------------------------------

/// Guidance state shared by the shell and the pages.
#[derive(Clone, Copy)]
pub struct GuidanceCtx {
    /// The state for the signed-in user, once the browser has it.
    pub state: RwSignal<GuidanceState>,
    /// The introduction shown on the current page (kept for the visit after
    /// it was marked seen, so it does not vanish while being read).
    pub current_intro: RwSignal<Option<Intro>>,
    /// Bumped to fetch the state again.
    pub version: RwSignal<u32>,
}

impl GuidanceCtx {
    /// Hide `key` at once and record it ("Got it", or the feature was used).
    pub fn dismiss(&self, key: &'static str, mute: bool) {
        self.state.update(|s| {
            if let Some(h) = Hint::from_key(key) {
                s.hints.retain(|x| *x != h);
            }
        });
        let key = key.to_string();
        leptos::task::spawn_local(async move {
            let _ = mark_guidance(key, mute).await;
        });
    }

    /// The user used a hinted feature that leaves no record of its own (the
    /// Draft Graph's Why panel, the command bar): the hint is done.
    pub fn used(&self, hint: Hint) {
        if self.state.with_untracked(|s| s.hints.contains(&hint)) {
            self.dismiss(hint.key(), false);
        }
    }
}

/// Create the guidance state (call once in `App`, inside the router).
pub fn provide_guidance() -> GuidanceCtx {
    let location = use_location();
    let ctx = GuidanceCtx {
        state: RwSignal::new(GuidanceState::default()),
        current_intro: RwSignal::new(None),
        version: RwSignal::new(0),
    };
    // Browser-only: hints depend on the page and on what was just done, so
    // the state is fetched again on every route change.
    let res = LocalResource::new(move || {
        let _ = location.pathname.get();
        let _ = ctx.version.get();
        get_guidance()
    });
    Effect::new(move |_| {
        if let Some(Ok(s)) = res.get() {
            ctx.state.set(s);
        }
    });
    // The introduction for this page: shown for the visit, recorded as seen
    // at once (so its "New" mark goes), dropped on leaving the page.
    Effect::new(move |_| {
        let path = location.pathname.get();
        let found = ctx.state.with(|s| guidance::intro_for(s, &path).cloned());
        let keep = ctx
            .current_intro
            .with_untracked(|c| c.as_ref().is_some_and(|i| i.route == path));
        if let Some(intro) = found {
            ctx.state
                .update(|s| s.intros.retain(|i| i.key != intro.key));
            let key = intro.key.clone();
            ctx.current_intro.set(Some(intro));
            leptos::task::spawn_local(async move {
                let _ = mark_guidance(key, false).await;
            });
        } else if !keep {
            ctx.current_intro.set(None);
        }
    });
    provide_context(ctx);
    ctx
}

/// Provides [`GuidanceCtx`] to everything inside (the app shell).
#[component]
pub fn GuidanceProvider(children: Children) -> impl IntoView {
    provide_guidance();
    children()
}

fn ctx() -> Option<GuidanceCtx> {
    use_context::<GuidanceCtx>()
}

// ---------------------------------------------------------------------------
// Hints, introductions, "New" marks
// ---------------------------------------------------------------------------

/// A first-use hint where its feature is on screen.
///
/// A11y: an info `Callout` (`role="status"`, polite) with a bold first
/// line and one "Got it" button; it renders in the page flow, never over
/// content, and takes no focus. At most one hint or introduction shows per
/// page (`models::guidance::page_hint`).
#[component]
pub fn FirstUseHint(
    hint: Hint,
    #[prop(into)] title: String,
    #[prop(optional, into)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let location = use_location();
    let Some(g) = ctx() else {
        return ().into_any();
    };
    let visible = move || {
        let path = location.pathname.get();
        let intro_here = g
            .current_intro
            .with(|c| c.as_ref().is_some_and(|i| i.route == path));
        !intro_here
            && g.state
                .with(|s| guidance::page_hint(s, &path) == Some(hint))
    };
    let title = StoredValue::new(title);
    let children = StoredValue::new(children);
    view! {
        <Show when=visible>
            <div class=class.clone() data-guidance-hint=hint.key()>
                <Callout
                    title=title.get_value()
                    action=move || view! {
                        <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| g.dismiss(hint.key(), true)) attr:data-testid="hint-got-it">
                            "Got it"
                        </Button>
                    }
                >
                    {children.with_value(|c| c())}
                </Callout>
            </div>
        </Show>
    }
    .into_any()
}

/// The page's new-feature introduction, at the top of the page.
///
/// A11y: an info `Callout` (`role="status"`) starting "New:", with a link
/// to the changelog entry and one "Got it" button. Shown once: the visit
/// records it as seen.
#[component]
pub fn IntroNote() -> impl IntoView {
    let location = use_location();
    let Some(g) = ctx() else {
        return ().into_any();
    };
    let current = move || {
        g.current_intro
            .get()
            .filter(|i| i.route == location.pathname.get())
    };
    view! {
        {move || current().map(|i| {
            let key = i.key.clone();
            let href = format!("/changelog#{}", i.slug);
            view! {
                <div class="mx-auto max-w-7xl px-4 pt-6 sm:px-8" data-guidance-intro=i.route.clone()>
                    <Callout
                        title=format!("New: {}", i.feature)
                        action=move || {
                            let key = key.clone();
                            view! {
                                <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet attr:data-testid="intro-got-it" on_click=Callback::new(move |_| {
                                    g.current_intro.set(None);
                                    let key = key.clone();
                                    leptos::task::spawn_local(async move {
                                        let _ = mark_guidance(key, true).await;
                                    });
                                })>
                                    "Got it"
                                </Button>
                            }
                        }
                    >
                        {i.text.clone()} " "
                        <a href=href class="text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">"What changed"</a>
                    </Callout>
                </div>
            }
        })}
    }
    .into_any()
}

/// "New" after a page's name in the navigation while its introduction is
/// unseen.
///
/// A11y: visible text "New" (no icon); screen readers hear ", new".
#[component]
pub fn NewMark(route: &'static str) -> impl IntoView {
    let Some(g) = ctx() else {
        return ().into_any();
    };
    let is_new = move || {
        g.state
            .with(|s| guidance::new_routes(s).iter().any(|r| r == route))
    };
    view! {
        <Show when=is_new>
            <span class="ml-1.5 text-caption font-semibold text-info" data-new-mark=route>
                <span class="sr-only">", "</span>"New"
            </span>
        </Show>
    }
    .into_any()
}

/// Whether any page of `paths` carries a "New" mark (the hub link).
pub fn any_new(paths: &[&str]) -> bool {
    ctx().is_some_and(|g| {
        g.state.with(|s| {
            guidance::new_routes(s)
                .iter()
                .any(|r| paths.contains(&r.as_str()))
        })
    })
}

/// The command bar's quiet line in the navigation (desktop only: phones
/// have no keyboard shortcut).
///
/// A11y: plain text with a keycap and a "Got it" button; it never takes
/// focus. Opening the command bar also ends it.
#[component]
pub fn CommandBarHint() -> impl IntoView {
    let location = use_location();
    let Some(g) = ctx() else {
        return ().into_any();
    };
    // The page's introduction stays on screen after it was recorded as
    // seen (it left the state then): no second piece of guidance beside it.
    let visible = move || {
        let path = location.pathname.get();
        let intro_here = g
            .current_intro
            .with(|c| c.as_ref().is_some_and(|i| i.route == path));
        !intro_here && g.state.with(|s| guidance::nav_hint(s, &path))
    };
    view! {
        <Show when=visible>
            <span class="hidden items-center gap-2 text-caption text-muted lg:flex" data-guidance-hint="hint.command_bar">
                <span>"Jump to any page with " <kbd class="rounded-ctl border border-outline px-1 text-caption text-secondary">"Ctrl K"</kbd></span>
                <button
                    type="button"
                    class="cursor-pointer rounded-ctl px-1.5 py-0.5 text-caption text-secondary underline decoration-outline underline-offset-4 hover:text-primary"
                    on:click=move |_| g.dismiss(Hint::CommandBar.key(), true)
                >
                    "Got it"
                </button>
            </span>
        </Show>
    }
    .into_any()
}

// ---------------------------------------------------------------------------
// Match Day: welcome, checklist, nudge
// ---------------------------------------------------------------------------

/// Mark `key` from Match Day, then run `after`.
fn mark_then(key: &'static str, mute: bool, after: impl FnOnce() + 'static) {
    leptos::task::spawn_local(async move {
        let _ = mark_guidance(key.to_string(), mute).await;
        after();
    });
}

/// The welcome: one calm panel at the top of Match Day for a new account.
///
/// A11y: a `section` labelled by its `h2`; the loop is a list; "Start" is a
/// real link (the page's one primary action) and "Skip, I'll find my way"
/// a button. Nothing moves focus.
#[component]
pub fn WelcomePanel(
    welcome: Welcome,
    /// Start is the page's primary action (no other chalk action on the
    /// page); otherwise it is a secondary button.
    primary: bool,
    on_change: Callback<()>,
) -> impl IntoView {
    let start_href = welcome.start_href.clone();
    let start = Callback::new(move |_| {
        // Record the start first, then follow the link (a click that leaves
        // before the write would show the welcome again).
        mark_then(guidance::WELCOME, false, || ());
    });
    let skip =
        Callback::new(move |_| mark_then(guidance::WELCOME, true, move || on_change.run(())));
    let first = match &welcome.team_name {
        Some(t) => format!("You are on {t}. The checklist below shows what your team has done and what comes next."),
        None => "Start with your team: create it, or find it on the roster page and ask to join. The checklist below takes you through the rest.".to_string(),
    };
    view! {
        <section class="mb-10 border-y border-rule bg-ink-1 px-5 py-6 sm:px-8" aria-labelledby="welcome-h" data-welcome>
            <p class="eyebrow">"Welcome"</p>
            <h2 id="welcome-h" class="mt-1 break-words font-display text-title font-semibold text-primary">
                {format!("Welcome to Team Companion, {}", welcome.username)}
            </h2>
            <p class="mt-3 max-w-prose text-reading text-secondary">
                "Team Companion keeps your team's week in one loop:"
            </p>
            <ol class="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 font-mono text-label text-primary" aria-label="The loop">
                {["Scout", "Draft", "Plan", "Play", "Review"].into_iter().enumerate().map(|(i, s)| view! {
                    <li class="flex items-center gap-2">
                        {(i > 0).then(|| view! { <span aria-hidden="true" class="text-muted">"\u{2192}"</span> })}
                        <span>{s}</span>
                    </li>
                }).collect_view()}
            </ol>
            <p class="mt-4 max-w-prose text-body text-secondary">{first}</p>
            {(!welcome.riot_linked).then(|| view! {
                <p class="mt-2 max-w-prose text-body text-secondary">
                    "Optional: link your Riot account on your "
                    <a href="/profile#riot-h" class="text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">"profile"</a>
                    " to sync your games."
                </p>
            })}
            <div class="mt-5 flex flex-wrap items-center gap-3">
                <Button href=start_href variant=if primary { ButtonVariant::Primary } else { ButtonVariant::Secondary } on_click=start attr:data-testid="welcome-start">"Start"</Button>
                <Button variant=ButtonVariant::Quiet on_click=skip attr:data-testid="welcome-skip">"Skip, I\u{2019}ll find my way"</Button>
            </div>
        </section>
    }
}

/// Checklist items as prep steps (the next open one emphasised).
pub fn checklist_steps(items: &[ChecklistItem]) -> Vec<PrepStep> {
    let next = guidance::next_step(items);
    items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            let state = if it.done {
                PrepState::Done
            } else if Some(i) == next {
                PrepState::Next
            } else {
                PrepState::Todo
            };
            let status = if state == PrepState::Next {
                "Next".to_string()
            } else {
                it.status.clone()
            };
            let mut s = PrepStep::new(it.title.clone(), state)
                .detail(it.detail.clone())
                .status(status);
            if let Some(h) = &it.href {
                s = s.link(h.clone(), it.action.clone());
            }
            s
        })
        .collect()
}

/// The getting-started checklist on Match Day.
///
/// A11y: a `section` labelled by its `h2` with the `PrepSequence` contract
/// (ordered list, status in words, the next step `aria-current="step"`);
/// "Hide this checklist" is a button, and the toast that follows says where
/// to bring it back.
#[component]
pub fn GettingStarted(
    items: Vec<ChecklistItem>,
    forced: bool,
    emphasis: PrepEmphasis,
    on_change: Callback<()>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>();
    let query = leptos_router::hooks::use_query_map();
    let navigate = leptos_router::hooks::use_navigate();
    let done = items.iter().filter(|i| i.done).count();
    let total = items.len();
    let steps = checklist_steps(&items);
    let hide = Callback::new(move |_| {
        let navigate = navigate.clone();
        mark_then(guidance::CHECKLIST, true, move || {
            if let Some(t) = toast {
                t.push.run(Toast::info(
                    "Checklist hidden. Bring it back on your profile, under Guidance.",
                ));
            }
            // "Guide me" (`/?guide=1`) forces the checklist open: a refetch
            // of the same address would draw it again, so leave the query.
            if query.with_untracked(|q| q.get("guide").is_some()) {
                navigate("/", Default::default());
            } else {
                on_change.run(());
            }
        })
    });
    view! {
        <section id="getting-started" class="mb-10 scroll-mt-20" aria-labelledby="start-h" data-checklist>
            <SectionHeader
                title="Getting started"
                id="start-h"
                meta=format!("{done} of {total} done")
                actions=move || view! {
                    <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=hide attr:data-testid="hide-checklist">"Hide this checklist"</Button>
                }
            />
            {forced.then(|| view! {
                <p class="-mt-2 mb-3 text-label text-muted">"Shown because you asked for it. It stays hidden on Match Day otherwise."</p>
            })}
            <PrepSequence label="Getting started with Team Companion" steps=steps emphasis=emphasis />
        </section>
    }
}

/// One quiet nudge line under the prep sequence.
///
/// A11y: a paragraph with a real link and a "Don't show this again"
/// button; no live region (it renders with the page).
#[component]
pub fn NudgeLine(nudge: Nudge, on_change: Callback<()>) -> impl IntoView {
    let key = nudge.kind.key();
    let mute = Callback::new(move |_| mark_then(key, true, move || on_change.run(())));
    view! {
        <p class="flex flex-wrap items-baseline gap-x-3 gap-y-1 border-t border-rule pt-3 text-body text-secondary" data-nudge=key>
            <span>{nudge.text.clone()}</span>
            <a href=nudge.href.clone() class="font-medium text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">{nudge.action.clone()}</a>
            <button
                type="button"
                class="cursor-pointer text-label text-muted underline decoration-rule underline-offset-4 hover:text-primary"
                on:click=move |ev| mute.run(ev)
                data-testid="mute-nudge"
            >
                "Don\u{2019}t show this again"
            </button>
        </p>
    }
}

// ---------------------------------------------------------------------------
// Profile
// ---------------------------------------------------------------------------

/// The profile's "Guidance" section: the switch and the three resets.
///
/// A11y: a `Switch` (`role="switch"`) named "Show guidance"; the resets are
/// buttons; results are announced by toasts (polite).
#[component]
pub fn GuidanceSettings(enabled: bool) -> impl IntoView {
    let toast = use_context::<ToastContext>();
    let on = RwSignal::new(enabled);
    let say = move |t: Toast| {
        if let Some(c) = toast {
            c.push.run(t);
        }
    };
    let bump = move || {
        if let Some(g) = ctx() {
            g.version.update(|v| *v += 1);
        }
    };
    let on_change = Callback::new(move |v: bool| {
        leptos::task::spawn_local(async move {
            match set_guidance_enabled(v).await {
                Ok(()) => {
                    bump();
                    say(Toast::success(if v {
                        "Guidance is on."
                    } else {
                        "Guidance is off: no welcome, checklist, hints, nudges or New marks."
                    }));
                }
                Err(e) => {
                    on.set(!v);
                    say(Toast::error(crate::models::utils::error_text(&e)));
                }
            }
        });
    });
    let reset = move |what: &'static str, done: &'static str| {
        Callback::new(move |_| {
            leptos::task::spawn_local(async move {
                match reset_guidance(what.to_string()).await {
                    Ok(()) => {
                        bump();
                        say(Toast::success(done));
                    }
                    Err(e) => say(Toast::error(crate::models::utils::error_text(&e))),
                }
            });
        })
    };
    view! {
        <section class="mt-12 scroll-mt-20 flex flex-col gap-4" id="guidance" aria-labelledby="guidance-h" data-guidance-settings>
            <SectionHeader title="Guidance" id="guidance-h" />
            <div>
                <Switch label="Show guidance" checked=on on_change=on_change />
                <p class="mt-2 max-w-prose text-caption text-muted">"The welcome, the getting-started checklist on Match Day, one-time hints where a feature is, next-step lines on Match Day and New marks for new features. Every piece can also be turned off where it appears."</p>
            </div>
            <div class="flex flex-wrap gap-3">
                <Button on_click=reset("checklist", "The checklist is back on Match Day until it is complete.") attr:data-testid="reset-checklist">"Show the checklist again"</Button>
                <Button on_click=reset("hints", "Hints you dismissed will show again where their feature is.") attr:data-testid="reset-hints">"Show hints again"</Button>
                <Button on_click=reset("nudges", "Next-step lines you turned off can show on Match Day again.") attr:data-testid="reset-nudges">"Show nudges again"</Button>
            </div>
            {move || (!on.get()).then(|| view! {
                <Callout tone=CalloutTone::Info>"Guidance is off. These buttons take effect when you turn it on again."</Callout>
            })}
        </section>
    }
}
