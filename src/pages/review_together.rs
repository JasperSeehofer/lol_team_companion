//! M165 "Review together": the server functions of a review session (the
//! page component is built on top of them). Every function starts with
//! `authz::authed()` and takes the team from the session
//! (`ctx.require_team()`); session, match, card and proposal ids from the
//! client are checked as records of that team in `server::review_together`
//! ("Not found" otherwise). Writes take `Bucket::ReviewWrite` per user; the
//! poll is not rate limited (a sync round polls every 2 s).
//! Signatures, payloads and phase rules: `.planning/missions/M165-*/server-api.md`.

use crate::models::review_together::{
    MatchSessionLink, OpenedSession, ReviewSessionView, SessionPage, SessionPatternTexts,
    SessionPoll,
};
use leptos::prelude::*;

/// The quiet nudge follows the viewer's guidance (M165 item 8): off with
/// the profile's Guidance switch or once muted ("Don't show this again").
#[cfg(feature = "ssr")]
async fn respect_guidance(
    db: &surrealdb::Surreal<surrealdb::engine::local::Db>,
    user_id: &str,
    state: &mut crate::models::review_together::SessionState,
) -> Result<(), ServerFnError> {
    use crate::models::guidance::{is_muted, NudgeKind};
    use crate::server::{authz, guidance};
    if state.quiet.is_none() {
        return Ok(());
    }
    let on = guidance::enabled(db, user_id)
        .await
        .map_err(authz::server_err)?;
    let rows = guidance::rows(db, user_id)
        .await
        .map_err(authz::server_err)?;
    if !on || is_muted(&rows, NudgeKind::QuietPlayer.key()) {
        state.quiet = None;
    }
    Ok(())
}

/// Parse a prompt key from the client ("Not found" when unknown).
#[cfg(feature = "ssr")]
fn prompt_of(key: &str) -> Result<crate::models::review_together::Prompt, ServerFnError> {
    crate::models::review_together::Prompt::from_key(key.trim())
        .ok_or_else(|| crate::server::authz::server_err(crate::server::db::DbError::NotFound))
}

/// The signed-in member, their team and the write budget.
#[cfg(feature = "ssr")]
async fn writer() -> Result<(crate::server::authz::Ctx, String), ServerFnError> {
    use crate::server::authz;
    use crate::server::rate_limit::Bucket;
    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    authz::rate_limit_user(Bucket::ReviewWrite, &ctx.user)?;
    Ok((ctx, team_id))
}

/// The poll: `Unchanged` (with who is present) when `rev` is current, else
/// the viewer's full state. Before the reveal it holds only the viewer's own
/// cards and team-level counts.
#[server]
pub async fn get_session_state(
    session_id: String,
    rev: Option<i64>,
) -> Result<SessionPoll, ServerFnError> {
    use crate::server::{authz, db, review_together};
    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    let leads = db::is_team_leader_or_coach(&ctx.db, &ctx.user.id, &team_id)
        .await
        .map_err(authz::server_err)?;
    let mut poll = review_together::get_session_state(
        &ctx.db,
        &team_id,
        &ctx.user.id,
        &ctx.user.username,
        leads,
        &session_id,
        rev,
    )
    .await
    .map_err(authz::server_err)?;
    if let SessionPoll::Full(state) = &mut poll {
        respect_guidance(&ctx.db, &ctx.user.id, state).await?;
    }
    Ok(poll)
}

/// The match a session reviews, as its page title reads: "Scrim vs
/// Riverside Rooks", plus " · G2" for a series (`None` when the match is
/// gone or not the team's).
#[cfg(feature = "ssr")]
async fn match_title(
    db: &surrealdb::Surreal<surrealdb::engine::local::Db>,
    team_id: &str,
    match_id: &str,
    game_number: i32,
) -> Option<String> {
    use crate::models::match_day::{game_count, kind_label};
    let m = crate::server::db::get_scheduled_match(db, team_id, match_id)
        .await
        .ok()?;
    let base = match &m.opponent_name {
        Some(o) => format!("{} vs {o}", kind_label(&m.kind)),
        None => kind_label(&m.kind).to_string(),
    };
    Some(if game_count(&m.format) > 1 {
        format!("{base} \u{b7} G{game_number}")
    } else {
        base
    })
}

/// The session page's first load: the session named by `session`, or the
/// one of `match_id` and `game` (opened when the game has none yet, the
/// opener facilitating: the match thread's "Review together"). Returns the
/// page title, the team's usernames for the owner chips and the viewer's
/// full state.
#[server]
pub async fn get_session_page(
    session: Option<String>,
    match_id: Option<String>,
    game: Option<i32>,
) -> Result<SessionPage, ServerFnError> {
    use crate::server::{authz, db, review_together};
    let session = session.filter(|s| !s.trim().is_empty());
    let session_id = match (session, match_id.filter(|m| !m.trim().is_empty())) {
        (Some(id), _) => id,
        (None, Some(m)) => {
            let (ctx, team_id) = writer().await?;
            review_together::open_session(&ctx.db, &team_id, &ctx.user.id, &m, game, None)
                .await
                .map_err(authz::server_err)?
                .session_id
        }
        (None, None) => return Err(authz::server_err(db::DbError::NotFound)),
    };
    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    let leads = db::is_team_leader_or_coach(&ctx.db, &ctx.user.id, &team_id)
        .await
        .map_err(authz::server_err)?;
    let poll = review_together::get_session_state(
        &ctx.db,
        &team_id,
        &ctx.user.id,
        &ctx.user.username,
        leads,
        &session_id,
        None,
    )
    .await
    .map_err(authz::server_err)?;
    let SessionPoll::Full(mut state) = poll else {
        return Err(authz::server_err(db::DbError::Other(
            "a first load answered Unchanged".into(),
        )));
    };
    respect_guidance(&ctx.db, &ctx.user.id, &mut state).await?;
    let title = match &state.match_id {
        Some(m) => match_title(&ctx.db, &team_id, m, state.game_number).await,
        None => None,
    }
    .unwrap_or_else(|| "Review together".to_string());
    let members = db::get_user_team_with_members(&ctx.db, &ctx.user.id)
        .await
        .map_err(authz::server_err)?
        .map(|(_, m)| m.into_iter().map(|m| m.username).collect())
        .unwrap_or_default();
    Ok(SessionPage {
        session_id: state.id.clone(),
        title,
        members,
        state: *state,
    })
}

/// Open (or return) the session of one game of the team's scheduled match.
/// Any member; the opener facilitates. `game_number` is 1 to the match's
/// game count (`None` = 1).
#[server]
pub async fn open_session(
    match_id: String,
    game_number: Option<i32>,
    match_riot_id: Option<String>,
) -> Result<OpenedSession, ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    review_together::open_session(
        &ctx.db,
        &team_id,
        &ctx.user.id,
        &match_id,
        game_number,
        match_riot_id,
    )
    .await
    .map_err(authz::server_err)
}

/// The sessions of one of the team's scheduled matches, per game.
#[server]
pub async fn list_match_sessions(match_id: String) -> Result<Vec<MatchSessionLink>, ServerFnError> {
    use crate::server::{authz, review_together};
    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    review_together::sessions_of_match(&ctx.db, &team_id, &match_id)
        .await
        .map_err(authz::server_err)
}

/// The leader or a coach takes over facilitation.
#[server]
pub async fn take_over_facilitation(session_id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, db, review_together};
    let (ctx, team_id) = writer().await?;
    let leads = db::is_team_leader_or_coach(&ctx.db, &ctx.user.id, &team_id)
        .await
        .map_err(authz::server_err)?;
    review_together::take_over(&ctx.db, &team_id, &ctx.user.id, leads, &session_id)
        .await
        .map_err(authz::server_err)
}

/// Write a card (or change one of your own, `entry_id`) while writing.
/// Returns the card's id.
#[server]
pub async fn upsert_entry(
    session_id: String,
    prompt: String,
    entry_id: Option<String>,
    text: String,
) -> Result<String, ServerFnError> {
    use crate::server::{authz, review_together};
    let prompt = prompt_of(&prompt)?;
    let (ctx, team_id) = writer().await?;
    review_together::upsert_entry(
        &ctx.db,
        &team_id,
        &ctx.user.id,
        &session_id,
        prompt,
        entry_id,
        &text,
    )
    .await
    .map_err(authz::server_err)
}

/// Delete one of your own cards (or your pass) while writing.
#[server]
pub async fn delete_entry(session_id: String, entry_id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    review_together::delete_entry(&ctx.db, &team_id, &ctx.user.id, &session_id, &entry_id)
        .await
        .map_err(authz::server_err)
}

/// Pass on a prompt (`passed = true`) or take the pass back.
#[server]
pub async fn pass_prompt(
    session_id: String,
    prompt: String,
    passed: bool,
) -> Result<(), ServerFnError> {
    use crate::server::{authz, review_together};
    let prompt = prompt_of(&prompt)?;
    let (ctx, team_id) = writer().await?;
    review_together::pass_prompt(&ctx.db, &team_id, &ctx.user.id, &session_id, prompt, passed)
        .await
        .map_err(authz::server_err)
}

/// Facilitator: start a timed silent round (default 3 minutes).
#[server]
pub async fn start_timed_round(
    session_id: String,
    minutes: Option<u32>,
) -> Result<(), ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    review_together::start_timed_round(&ctx.db, &team_id, &ctx.user.id, &session_id, minutes)
        .await
        .map_err(authz::server_err)
}

/// Facilitator: one more minute for the timed round.
#[server]
pub async fn add_minute(session_id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    review_together::add_minute(&ctx.db, &team_id, &ctx.user.id, &session_id)
        .await
        .map_err(authz::server_err)
}

/// Facilitator: reveal now.
#[server]
pub async fn reveal_now(session_id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    review_together::reveal_now(&ctx.db, &team_id, &ctx.user.id, &session_id)
        .await
        .map_err(authz::server_err)
}

/// Facilitator: from the revealed cards to deciding.
#[server]
pub async fn start_deciding(session_id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    review_together::start_deciding(&ctx.db, &team_id, &ctx.user.id, &session_id)
        .await
        .map_err(authz::server_err)
}

/// Toggle your +1 on a revealed card. Returns whether it is on now.
#[server]
pub async fn plus_one(session_id: String, entry_id: String) -> Result<bool, ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    review_together::plus_one(&ctx.db, &team_id, &ctx.user.id, &session_id, &entry_id)
        .await
        .map_err(authz::server_err)
}

/// Facilitator: merge cards under `target_id`. `entry_ids_json` is a JSON
/// array of card ids (same prompt).
#[server]
pub async fn merge_entries(
    session_id: String,
    target_id: String,
    entry_ids_json: String,
) -> Result<(), ServerFnError> {
    use crate::server::{authz, review_together};
    let ids: Vec<String> = serde_json::from_str(&entry_ids_json)
        .map_err(|_| ServerFnError::new("The cards could not be read."))?;
    if ids.is_empty() || ids.len() > 50 {
        return Err(ServerFnError::new("Choose the cards to merge."));
    }
    let (ctx, team_id) = writer().await?;
    review_together::merge_entries(
        &ctx.db,
        &team_id,
        &ctx.user.id,
        &session_id,
        &target_id,
        &ids,
    )
    .await
    .map_err(authz::server_err)
}

/// Facilitator: take a card out of its cluster (a cluster head dissolves it).
#[server]
pub async fn unmerge(session_id: String, entry_id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    review_together::unmerge(&ctx.db, &team_id, &ctx.user.id, &session_id, &entry_id)
        .await
        .map_err(authz::server_err)
}

/// Any member: a proposal from a revealed card (`source_entry`), with the
/// text as edited. No proposer is stored. Returns the proposal's id.
#[server]
pub async fn create_proposal(
    session_id: String,
    source_entry: Option<String>,
    text: String,
) -> Result<String, ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    review_together::create_proposal(&ctx.db, &team_id, &session_id, source_entry, &text)
        .await
        .map_err(authz::server_err)
}

/// Toggle your decide vote on a proposal (three per member and session).
/// Returns whether it is on now.
#[server]
pub async fn decide_vote(session_id: String, proposal_id: String) -> Result<bool, ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    review_together::decide_vote(&ctx.db, &team_id, &ctx.user.id, &session_id, &proposal_id)
        .await
        .map_err(authz::server_err)
}

/// Facilitator: accept a proposal (at most three) or take it back.
#[server]
pub async fn set_accepted(
    session_id: String,
    proposal_id: String,
    accepted: bool,
) -> Result<(), ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    review_together::set_accepted(
        &ctx.db,
        &team_id,
        &ctx.user.id,
        &session_id,
        &proposal_id,
        accepted,
    )
    .await
    .map_err(authz::server_err)
}

/// Facilitator: the owner of a proposal, a current member's username (blank
/// for nobody); anyone else is refused.
#[server]
pub async fn set_owner(
    session_id: String,
    proposal_id: String,
    owner: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    review_together::set_owner(
        &ctx.db,
        &team_id,
        &ctx.user.id,
        &session_id,
        &proposal_id,
        owner,
    )
    .await
    .map_err(authz::server_err)
}

/// Facilitator: delete a proposal.
#[server]
pub async fn delete_proposal(session_id: String, proposal_id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    review_together::delete_proposal(&ctx.db, &team_id, &ctx.user.id, &session_id, &proposal_id)
        .await
        .map_err(authz::server_err)
}

/// Facilitator: close the session into the generated review with its action
/// items. Returns the review's id (`/post-game?review_id=`).
#[server]
pub async fn close_session(session_id: String) -> Result<String, ServerFnError> {
    use crate::server::{authz, review_together};
    let (ctx, team_id) = writer().await?;
    let review = review_together::close_session(&ctx.db, &team_id, &ctx.user.id, &session_id)
        .await
        .map_err(authz::server_err)?;
    // A live survey may ask about the review (M7), as for a written one.
    crate::server::community::store::note_review_saved(&ctx.db, &ctx.user.id).await;
    Ok(review)
}

/// For the review page: the session that generated `review_id`, read-only
/// (`None` for a review written alone).
#[server]
pub async fn get_review_session_for_review(
    review_id: String,
) -> Result<Option<ReviewSessionView>, ServerFnError> {
    use crate::server::{authz, review_together};
    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    let mut view = review_together::session_for_review(&ctx.db, &team_id, &ctx.user.id, &review_id)
        .await
        .map_err(authz::server_err)?;
    if let Some(v) = view.as_mut() {
        if let Some(m) = v.match_id.clone() {
            v.match_title = match_title(&ctx.db, &team_id, &m, v.game_number).await;
        }
    }
    Ok(view)
}

/// For "Patterns" on the review page: the closed sessions' "went to plan"
/// and "didn't go to plan" texts (texts only; empty without a team).
#[server]
pub async fn get_session_pattern_texts() -> Result<Vec<SessionPatternTexts>, ServerFnError> {
    use crate::server::{authz, review_together};
    let ctx = authz::authed().await?;
    let Some(team_id) = ctx.team().await? else {
        return Ok(Vec::new());
    };
    review_together::session_pattern_texts(&ctx.db, &team_id)
        .await
        .map_err(authz::server_err)
}

// ---------------------------------------------------------------------------
// The session page: /review/together?session=<id>
// (or ?match_id=<id>&game=<n> from the match thread, which opens the
// session and then puts its id in the address)
// ---------------------------------------------------------------------------

use crate::components::review_together::{
    Disclosure, NoteGroup, OwnerChips, PlusOne, PresenceStrip, QuietNudge, ReviewNote, VotesLeft,
    WrittenCount,
};
use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, Callout, CalloutTone, Checkbox, ConfirmDialog, EmptyState,
    PageHeader, PageShell, SaveState, SaveStatus, SaveTracker, Skeleton, SkeletonGroup,
    SkeletonShape, Text, TextField, TextTag, Toast, ToastContext, Tone,
};
use crate::models::review_together::{
    minutes_left, note_rows, row_root, Phase, Prompt, SessionState, ACCEPT_MAX, DECIDE_VOTES,
    ENTRIES_PER_PROMPT, TEXT_MAX,
};
use crate::pages::navigate::{go, navigator, replace_url, Nav};

/// The sealed sentence (reference a1): said once, in words.
const SEALED: &str = "Sealed. Only you see what you write until the reveal.";
/// The toast for an edit the server refused because the cards were revealed.
const LATE_EDIT: &str = "Your last edit arrived after the reveal and was not kept.";
/// The character count shows from here.
const COUNT_FROM: usize = 240;

fn err_text(e: &ServerFnError) -> String {
    crate::models::utils::error_text(e)
}

/// One of the viewer's own notes while writing. Keyed by `key` (local,
/// never a user id); `id` is the card's record id once saved.
#[derive(Clone, Copy)]
struct LocalEntry {
    key: u64,
    prompt: Prompt,
    id: RwSignal<Option<String>>,
    text: RwSignal<String>,
    /// The text the server has (to skip saves that change nothing).
    saved_text: StoredValue<String>,
    in_flight: StoredValue<bool>,
    again: StoredValue<bool>,
    #[allow(dead_code)]
    timer: StoredValue<Option<i32>>,
}

/// What every handler on the page needs. `Copy`.
#[derive(Clone, Copy)]
struct Ctx {
    sid: StoredValue<String>,
    state: RwSignal<SessionState>,
    tracker: SaveTracker,
    toast: ToastContext,
    refresh: Callback<()>,
    nav: Nav,
}

impl Ctx {
    fn sid(&self) -> String {
        self.sid.get_value()
    }
    fn error(&self, e: &ServerFnError) {
        self.toast.push.run(Toast::error(err_text(e)));
    }
    /// Run a write, then poll at once (other members see it on their next
    /// poll).
    fn act<F>(&self, f: F)
    where
        F: std::future::Future<Output = Result<(), ServerFnError>> + 'static,
    {
        let c = *self;
        leptos::task::spawn_local(async move {
            match f.await {
                Ok(()) => c.refresh.run(()),
                Err(e) => c.error(&e),
            }
        });
    }
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

/// Milliseconds since the epoch on this device (0 on the server).
fn now_ms() -> f64 {
    #[cfg(feature = "hydrate")]
    {
        js_sys::Date::now()
    }
    #[cfg(not(feature = "hydrate"))]
    {
        0.0
    }
}

fn parse_ms(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|d| d.timestamp_millis())
}

/// The poll loop: a self-re-arming `setTimeout`, every 2 s while the tab is
/// visible and 10 s while it is hidden, stopped once the session is closed
/// and when the page unmounts. Browser only.
#[cfg(feature = "hydrate")]
#[derive(Clone)]
struct Poller {
    alive: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: std::sync::Arc<std::sync::atomic::AtomicI32>,
    sid: String,
    state: RwSignal<SessionState>,
    present: RwSignal<Vec<String>>,
    reconnecting: RwSignal<bool>,
    apply: Callback<SessionState>,
}

#[cfg(feature = "hydrate")]
fn poll_delay() -> i32 {
    let hidden = web_sys::window()
        .and_then(|w| w.document())
        .is_some_and(|d| d.hidden());
    if hidden {
        10_000
    } else {
        2_000
    }
}

#[cfg(feature = "hydrate")]
fn arm(p: Poller, ms: i32) {
    use std::sync::atomic::Ordering;
    use wasm_bindgen::prelude::*;
    if !p.alive.load(Ordering::SeqCst) {
        return;
    }
    let next = p.clone();
    let cb = Closure::once(move || {
        leptos::task::spawn_local(async move {
            if !next.alive.load(Ordering::SeqCst) {
                return;
            }
            let Some(rev) = next.state.try_with_untracked(|s| s.rev) else {
                return;
            };
            let res = get_session_state(next.sid.clone(), Some(rev)).await;
            if !next.alive.load(Ordering::SeqCst) {
                return;
            }
            match res {
                Ok(SessionPoll::Unchanged { present, .. }) => {
                    let _ = next.present.try_set(present);
                    let _ = next.reconnecting.try_set(false);
                }
                Ok(SessionPoll::Full(s)) => {
                    let _ = next.apply.try_run(*s);
                }
                Err(_) => {
                    let _ = next.reconnecting.try_set(true);
                }
            }
            let closed = next
                .state
                .try_with_untracked(|s| s.phase == Phase::Closed)
                .unwrap_or(true);
            if !closed {
                arm(next, poll_delay());
            }
        });
    });
    if let Some(w) = web_sys::window() {
        if let Ok(h) =
            w.set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), ms)
        {
            p.handle.store(h, Ordering::SeqCst);
        }
    }
    cb.forget();
}

/// `/review/together`: one game's review session, written by the whole
/// team: a silent round of four prompts, one reveal, then deciding at most
/// three action items. Phases are page states, never dialogs.
#[component]
pub fn ReviewTogetherPage() -> impl IntoView {
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            crate::pages::navigate::redirect_to_login();
        }
    });
    let query = leptos_router::hooks::use_query_map();
    let page = Resource::new(
        move || {
            let q = query.read();
            let session = q.get("session").filter(|s| !s.trim().is_empty());
            let match_id = q.get("match_id").filter(|s| !s.trim().is_empty());
            let game = q.get("game").and_then(|g| g.trim().parse::<i32>().ok());
            (session, match_id, game)
        },
        |(session, match_id, game)| async move {
            if session.is_none() && match_id.is_none() {
                return Ok(None);
            }
            get_session_page(session, match_id, game).await.map(Some)
        },
    );
    // The session the page shows: once a page has loaded, a change of the
    // address to the same session (`?match_id=` -> `?session=`) keeps it.
    view! {
        <PageShell attr:data-feedback-label="Review together">
            <Suspense fallback=|| view! {
                <PageHeader title="Review together" eyebrow="Review together" />
                <SkeletonGroup label="Loading the session"><Skeleton shape=SkeletonShape::Line lines=4 /></SkeletonGroup>
            }>
                {move || page.get().map(|r| match r {
                    Ok(Some(p)) => view! { <SessionView page=p /> }.into_any(),
                    Ok(None) => view! {
                        <PageHeader title="Review together" eyebrow="Review together" />
                        <EmptyState
                            title="No session open"
                            message="A review session belongs to one game of a scheduled match. Open it from the review step on Match Day once the match has started."
                            cta_label="Go to Match Day"
                            cta_href="/"
                        />
                    }.into_any(),
                    Err(e) => view! {
                        <PageHeader title="Review together" eyebrow="Review together" />
                        <Callout tone=CalloutTone::Danger title="This session did not open">
                            {err_text(&e)}
                        </Callout>
                    }.into_any(),
                })}
            </Suspense>
        </PageShell>
    }
}

/// The loaded session: holds the state, runs the poll and renders the phase.
#[component]
fn SessionView(page: crate::models::review_together::SessionPage) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let nav = navigator();
    let crate::models::review_together::SessionPage {
        session_id,
        title,
        members,
        state: initial,
    } = page;
    let sid = StoredValue::new(session_id.clone());
    let present = RwSignal::new(initial.present.clone());
    let reconnecting = RwSignal::new(false);
    // When this device received the state, to run the calm timer on the
    // server's clock (`server_now`), never the device's alone.
    let received = RwSignal::new(now_ms());
    let state = RwSignal::new(initial.clone());
    // The reveal (card 8): one wrapper, opacity 0 -> 1 in --dur-3, only
    // when this tab saw writing turn into revealed through a poll.
    let reveal_class = RwSignal::new("opacity-100");
    let announce = RwSignal::new(String::new());
    let tracker = crate::components::ui::use_save_state();

    let apply = Callback::new(move |next: SessionState| {
        let was_writing = state.with_untracked(|s| s.phase == Phase::Writing);
        let now_revealed = next.phase != Phase::Writing;
        if was_writing && now_revealed {
            let notes = next
                .cards
                .as_ref()
                .map(|c| c.decided.len() + c.to_plan.len() + c.off_plan.len() + c.next_time.len())
                .unwrap_or(0);
            let notes = match notes {
                1 => "1 note".to_string(),
                n => format!("{n} notes"),
            };
            announce.set(match next.voices {
                Some(v) => format!("Revealed: {notes}, {}.", v.label()),
                None => format!("Revealed: {notes}."),
            });
            #[cfg(feature = "hydrate")]
            if !reduced_motion() {
                use wasm_bindgen::prelude::*;
                reveal_class.set("opacity-0");
                // Two frames: the wrapper is painted at 0 before it moves.
                let outer = Closure::once(move || {
                    let inner = Closure::once(move || {
                        let _ = reveal_class.try_set("opacity-100");
                    });
                    if let Some(w) = web_sys::window() {
                        let _ = w.request_animation_frame(inner.as_ref().unchecked_ref());
                    }
                    inner.forget();
                });
                if let Some(w) = web_sys::window() {
                    let _ = w.request_animation_frame(outer.as_ref().unchecked_ref());
                }
                outer.forget();
            }
        }
        present.set(next.present.clone());
        reconnecting.set(false);
        received.set(now_ms());
        state.set(next);
    });

    let refresh = Callback::new(move |_| {
        let rev = state.get_untracked().rev;
        let id = sid.get_value();
        leptos::task::spawn_local(async move {
            match get_session_state(id, Some(rev)).await {
                Ok(SessionPoll::Full(s)) => {
                    let _ = apply.try_run(*s);
                }
                Ok(SessionPoll::Unchanged { present: p, .. }) => {
                    let _ = present.try_set(p);
                }
                Err(_) => {
                    let _ = reconnecting.try_set(true);
                }
            }
        });
    });

    // The address names the session (a match thread link opened it).
    let query = leptos_router::hooks::use_query_map();
    Effect::new(move |_| {
        let shown = query.with_untracked(|q| q.get("session"));
        let id = sid.get_value();
        if shown.as_deref() != Some(id.as_str()) {
            replace_url(nav, &format!("/review/together?session={id}"));
        }
    });

    #[cfg(feature = "hydrate")]
    {
        use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
        use std::sync::Arc;
        let alive = Arc::new(AtomicBool::new(true));
        let handle = Arc::new(AtomicI32::new(-1));
        {
            let alive = alive.clone();
            let handle = handle.clone();
            on_cleanup(move || {
                alive.store(false, Ordering::SeqCst);
                let h = handle.load(Ordering::SeqCst);
                if h >= 0 {
                    if let Some(w) = web_sys::window() {
                        w.clear_timeout_with_handle(h);
                    }
                }
            });
        }
        let poller = Poller {
            alive,
            handle,
            sid: session_id.clone(),
            state,
            present,
            reconnecting,
            apply,
        };
        // Once, after hydration: the server never polls.
        Effect::new(move |_| {
            if state.with_untracked(|s| s.phase != Phase::Closed) {
                arm(poller.clone(), poll_delay());
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = &session_id;

    let ctx = Ctx {
        sid,
        state,
        tracker,
        toast,
        refresh,
        nav,
    };
    let members = StoredValue::new(members);
    let phase = Memo::new(move |_| state.with(|s| s.phase));
    // Revealed and deciding are one page state: moving between them keeps
    // the notes (and an open proposal form) mounted.
    let stage = Memo::new(move |_| match phase.get() {
        Phase::Deciding => Phase::Revealed,
        p => p,
    });
    let open_after = Memo::new(move |_| phase.get().is_open_after_reveal());
    let writing = move || phase.get() == Phase::Writing;
    let facilitator = Memo::new(move |_| state.with(|s| s.i_am_facilitator));

    view! {
        // Present from the first paint, empty until this tab sees the reveal.
        <p role="status" class="sr-only" data-reveal-status="">{move || announce.get()}</p>
        <PageHeader
            title=title
            eyebrow="Review together"
            meta=move || view! {
                // Idle says nothing: the region stays mounted (announced on
                // its first change) but out of the flow, so it takes no gap.
                <span class=move || if tracker.state.with(|s| matches!(s, SaveState::Idle)) { "sr-only" } else { "contents" }>
                    <SaveStatus state=tracker.state />
                </span>
                <DeadlineLine state=state received=received />
                {move || (phase.get().is_open_after_reveal()).then(|| {
                    let n = state.with(|s| s.proposals.len());
                    view! { <a href="#decide" class="text-label text-secondary underline decoration-outline underline-offset-4 hover:text-primary lg:hidden">{format!("Decide ({n})")}</a> }
                })}
            }
        />
        {move || writing().then(|| view! {
            <Text tag=TextTag::P tone=Tone::Secondary class="mb-6">{SEALED}</Text>
        })}
        <div class="flex flex-col gap-8 lg:grid lg:grid-cols-[minmax(0,42rem)_20rem] lg:grid-rows-[auto_1fr] lg:items-start lg:gap-x-12 lg:gap-y-8">
            <aside class="flex flex-col gap-4 lg:col-start-2 lg:row-start-1 lg:border-l lg:border-rule lg:pl-8" aria-label="The session" data-session-side="">
                <PresenceStrip names=present reconnecting=reconnecting />
                {move || (facilitator.get() && phase.get() != Phase::Closed).then(|| view! {
                    <p class="text-caption text-muted md:hidden">"Round controls are on a wider screen."</p>
                })}
                <FacilitatorTools ctx=ctx />
            </aside>
            <div class="flex min-w-0 max-w-2xl flex-col gap-8 lg:col-start-1 lg:row-span-2 lg:row-start-1">
                {move || match stage.get() {
                    Phase::Writing => view! { <WritingView ctx=ctx /> }.into_any(),
                    Phase::Revealed | Phase::Deciding => view! {
                        <div
                            class=move || format!("flex flex-col gap-12 transition-opacity duration-(--dur-3) ease-out motion-reduce:transition-none {}", reveal_class.get())
                            data-reveal=""
                        >
                            <RevealedView ctx=ctx />
                        </div>
                    }.into_any(),
                    Phase::Closed => view! { <ClosedView ctx=ctx /> }.into_any(),
                }}
            </div>
            {move || open_after.get().then(|| view! {
                <div class="lg:col-start-2 lg:row-start-2">
                    <DecideSection ctx=ctx members=members.get_value() />
                </div>
            })}
        </div>
    }
}

/// The deadline line (card 2, a3): "Open until Sat 21:40" for the async
/// window, "Closes 21:40 · 3 min left" in a timed round, re-rendered on the
/// minute; no seconds, no colour change, no bar.
#[component]
fn DeadlineLine(state: RwSignal<SessionState>, received: RwSignal<f64>) -> impl IntoView {
    use crate::components::local_time::{LocalTime, TimeStyle};
    let tick = RwSignal::new(0u32);
    let remaining = move || {
        let _ = tick.get();
        let (closes, server_now) =
            state.with(|s| (parse_ms(&s.writing_closes_at), parse_ms(&s.server_now)));
        let elapsed = (now_ms() - received.get()).max(0.0) as i64;
        match (closes, server_now) {
            (Some(c), Some(n)) => c - n - elapsed,
            _ => 0,
        }
    };
    // Wake on the next whole minute of the remaining time.
    #[cfg(feature = "hydrate")]
    {
        use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
        use std::sync::Arc;
        let alive = Arc::new(AtomicBool::new(true));
        let handle = Arc::new(AtomicI32::new(-1));
        {
            let alive = alive.clone();
            let handle = handle.clone();
            on_cleanup(move || {
                alive.store(false, Ordering::SeqCst);
                let h = handle.load(Ordering::SeqCst);
                if h >= 0 {
                    if let Some(w) = web_sys::window() {
                        w.clear_timeout_with_handle(h);
                    }
                }
            });
        }
        fn wake(
            alive: Arc<AtomicBool>,
            handle: Arc<AtomicI32>,
            tick: RwSignal<u32>,
            ms: i64,
            next_ms: std::rc::Rc<dyn Fn() -> i64>,
        ) {
            use wasm_bindgen::prelude::*;
            if !alive.load(Ordering::SeqCst) {
                return;
            }
            let (a, h) = (alive.clone(), handle.clone());
            let cb = Closure::once(move || {
                if !a.load(Ordering::SeqCst) {
                    return;
                }
                let _ = tick.try_update(|t| *t = t.wrapping_add(1));
                let ms = next_ms();
                wake(a, h, tick, ms, next_ms);
            });
            if let Some(w) = web_sys::window() {
                if let Ok(id) = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    ms.clamp(1_000, 60_000) as i32,
                ) {
                    handle.store(id, Ordering::SeqCst);
                }
            }
            cb.forget();
        }
        let next_ms: std::rc::Rc<dyn Fn() -> i64> = std::rc::Rc::new(move || {
            let r = remaining();
            let into = r.rem_euclid(60_000);
            if into < 500 {
                into + 60_050
            } else {
                into + 50
            }
        });
        Effect::new(move |_| {
            wake(
                alive.clone(),
                handle.clone(),
                tick,
                next_ms(),
                next_ms.clone(),
            );
        });
    }
    view! {
        {move || {
            let (phase, timed, closes) =
                state.with(|s| (s.phase, s.timed_round, s.writing_closes_at.clone()));
            match (phase, timed) {
                (Phase::Writing, false) => view! {
                    <span class="figures text-label text-secondary" data-deadline="">
                        "Open until " <LocalTime iso=closes style=TimeStyle::Short />
                    </span>
                }.into_any(),
                (Phase::Writing, true) => view! {
                    <span class="figures text-label text-secondary" data-deadline="">
                        "Closes " <LocalTime iso=closes style=TimeStyle::Clock /> " \u{b7} "
                        {move || match minutes_left(remaining()) {
                            Some(m) => format!("{m} min left"),
                            None => "under a minute left".to_string(),
                        }}
                    </span>
                }.into_any(),
                (Phase::Revealed | Phase::Deciding, _) => view! {
                    <span class="text-label text-secondary">"Revealed"</span>
                }.into_any(),
                (Phase::Closed, _) => view! {
                    <span class="text-label text-secondary">"Closed"</span>
                }.into_any(),
            }
        }}
    }
}

/// Facilitator tools at md and wider (card 3): the one written count, the
/// quiet nudge while writing, and the round controls, collapsed; "Take over
/// facilitating" for the leader or a coach. Never on a phone.
#[component]
fn FacilitatorTools(ctx: Ctx) -> impl IntoView {
    let state = ctx.state;
    let open = RwSignal::new(false);
    let confirm_reveal = RwSignal::new(false);
    let reveal_msg = RwSignal::new(String::new());
    let phase = Memo::new(move |_| state.with(|s| s.phase));
    let facilitator = Memo::new(move |_| state.with(|s| s.i_am_facilitator));
    let can_take_over = Memo::new(move |_| state.with(|s| s.can_take_over));
    let timed = Memo::new(move |_| state.with(|s| s.timed_round));
    let written = Signal::derive(move || state.with(|s| s.written_count));
    let members = Signal::derive(move || state.with(|s| s.member_count));
    let quiet = Memo::new(move |_| state.with(|s| s.quiet));

    let start_round = Callback::new(move |_| {
        let id = ctx.sid();
        ctx.act(async move { start_timed_round(id, Some(3)).await });
    });
    let add_one = Callback::new(move |_| {
        let id = ctx.sid();
        ctx.act(async move { add_minute(id).await });
    });
    let ask_reveal = Callback::new(move |_| {
        let (w, m) = state.with_untracked(|s| (s.written_count, s.member_count));
        reveal_msg.set(format!(
            "Writing ends for everyone. {w} of {m} have written."
        ));
        confirm_reveal.set(true);
    });
    let do_reveal = Callback::new(move |_| {
        let id = ctx.sid();
        ctx.act(async move { reveal_now(id).await });
    });
    let take_over = Callback::new(move |_| {
        let id = ctx.sid();
        ctx.act(async move { take_over_facilitation(id).await });
    });
    let start_decide = Callback::new(move |_| {
        let id = ctx.sid();
        // Decide opens with the top three by votes accepted (the
        // facilitator can still change them).
        let suggested: Vec<String> = state.with_untracked(|s| {
            s.proposals
                .iter()
                .filter(|p| p.suggested && !p.accepted)
                .map(|p| p.id.clone())
                .collect()
        });
        let already = state.with_untracked(|s| s.proposals.iter().filter(|p| p.accepted).count());
        ctx.act(async move {
            start_deciding(id.clone()).await?;
            for p in suggested
                .into_iter()
                .take(ACCEPT_MAX.saturating_sub(already))
            {
                set_accepted(id.clone(), p, true).await?;
            }
            Ok(())
        });
    });

    view! {
        <div class="hidden flex-col gap-4 md:flex md:flex-row md:flex-wrap md:items-start md:gap-x-8 md:border-y md:border-rule md:py-3 lg:flex-col lg:gap-4 lg:border-y-0 lg:py-0" data-facilitator-tools="">
            {move || (facilitator.get() && phase.get() == Phase::Writing).then(|| view! {
                <WrittenCount written=written members=members />
            })}
            {move || (facilitator.get() && phase.get() == Phase::Writing)
                .then(|| quiet.get())
                .flatten()
                .map(|n| view! { <div class="md:basis-full lg:basis-auto"><QuietNudge count=n /></div> })}
            {move || (facilitator.get() && phase.get() == Phase::Writing).then(|| view! {
                <div class="flex flex-col items-start gap-3">
                    <Disclosure label=Signal::derive(|| "Round controls".to_string()) open=open controls="round-controls" />
                    <Show when=move || open.get()>
                        <div id="round-controls" class="flex flex-wrap gap-3" data-round-controls="">
                            {move || if timed.get() {
                                view! {
                                    <Button size=ButtonSize::Touch on_click=add_one attr:data-testid="add-minute">"Add a minute"</Button>
                                }.into_any()
                            } else {
                                view! {
                                    <Button size=ButtonSize::Touch on_click=start_round attr:data-testid="start-round">"Start a 3-minute round"</Button>
                                }.into_any()
                            }}
                            <Button variant=ButtonVariant::Primary size=ButtonSize::Touch on_click=ask_reveal attr:data-testid="reveal-now">"Reveal now"</Button>
                        </div>
                    </Show>
                </div>
            })}
            {move || (facilitator.get() && phase.get() == Phase::Revealed).then(|| view! {
                <Button size=ButtonSize::Touch on_click=start_decide attr:data-testid="start-deciding">"Start deciding"</Button>
            })}
            {move || can_take_over.get().then(|| view! {
                <Button variant=ButtonVariant::Quiet size=ButtonSize::Touch on_click=take_over attr:data-testid="take-over">"Take over facilitating"</Button>
            })}
            {move || (!facilitator.get() && phase.get() != Phase::Closed).then(|| view! {
                <p class="text-caption text-muted">{move || format!("Facilitated by {}", state.with(|s| s.facilitator_name.clone()))}</p>
            })}
        </div>
        {move || {
            let msg = reveal_msg.get();
            view! {
                <ConfirmDialog
                    id="confirm-reveal"
                    open=confirm_reveal
                    title="Reveal now?"
                    message=msg
                    confirm_label="Reveal now"
                    on_confirm=do_reveal
                />
            }
        }}
    }
}

/// Make a local entry (under the page's owner, so it lives with the page).
fn local_entry(
    owner: &Owner,
    key: u64,
    prompt: Prompt,
    id: Option<String>,
    text: String,
) -> LocalEntry {
    owner.with(|| LocalEntry {
        key,
        prompt,
        id: RwSignal::new(id),
        text: RwSignal::new(text.clone()),
        saved_text: StoredValue::new(text),
        in_flight: StoredValue::new(false),
        again: StoredValue::new(false),
        timer: StoredValue::new(None),
    })
}

/// Save one entry now: create, change or (emptied) delete the card.
/// One request at a time per entry; a change during it saves again after.
fn save_entry(ctx: Ctx, e: LocalEntry) {
    if e.in_flight.try_get_value().unwrap_or(true) {
        e.again.set_value(true);
        return;
    }
    let text = e.text.get_untracked();
    let trimmed = text.trim().to_string();
    let id = e.id.get_untracked();
    if trimmed == e.saved_text.get_value().trim() && (id.is_some() || trimmed.is_empty()) {
        return;
    }
    e.in_flight.set_value(true);
    ctx.tracker.saving();
    let sid = ctx.sid();
    leptos::task::spawn_local(async move {
        let res: Result<Option<String>, ServerFnError> = if trimmed.is_empty() {
            match id {
                Some(id) => delete_entry(sid, id).await.map(|_| None),
                None => Ok(None),
            }
        } else {
            upsert_entry(sid, e.prompt.key().to_string(), id, trimmed.clone())
                .await
                .map(Some)
        };
        // `try_set_value` hands the value back when the row is gone.
        if e.in_flight.try_set_value(false).is_some() {
            return;
        }
        match res {
            Ok(new_id) => {
                let _ = e.id.try_set(new_id);
                e.saved_text.set_value(trimmed);
                if e.again.get_value() {
                    e.again.set_value(false);
                    save_entry(ctx, e);
                } else {
                    ctx.tracker.saved();
                }
            }
            Err(err) => {
                e.again.set_value(false);
                let msg = err_text(&err);
                if msg.contains("revealed") {
                    ctx.tracker.show(SaveState::Idle);
                    ctx.toast.push.run(Toast::info(LATE_EDIT));
                    ctx.refresh.run(());
                } else if ctx.tracker.failed(msg.clone()) {
                    ctx.toast
                        .push
                        .run(Toast::error(format!("Your note was not saved: {msg}")));
                }
            }
        }
    });
}

/// Save after 600 ms without typing (autosave); blur saves at once.
fn schedule_save(ctx: Ctx, e: LocalEntry) {
    ctx.tracker.show(SaveState::Unsaved);
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        if let Some(Some(old)) = e.timer.try_get_value() {
            if let Some(w) = web_sys::window() {
                w.clear_timeout_with_handle(old);
            }
        }
        let cb = Closure::once(move || {
            if e.timer.try_set_value(None).is_none() {
                save_entry(ctx, e);
            }
        });
        if let Some(w) = web_sys::window() {
            if let Ok(id) = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                600,
            ) {
                e.timer.set_value(Some(id));
            }
        }
        cb.forget();
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (ctx, e);
}

/// The silent round (card 2): your own notes under the four prompts, with
/// autosave, Pass and Remove. Nothing of anyone else is on the page.
#[component]
fn WritingView(ctx: Ctx) -> impl IntoView {
    let owner = Owner::current().unwrap_or_default();
    let owner = StoredValue::new_local(owner);
    let next_key = StoredValue::new(0u64);
    let mine = ctx.state.with_untracked(|s| s.mine.clone());
    let make = move |prompt: Prompt, id: Option<String>, text: String| -> LocalEntry {
        let k = next_key.get_value();
        next_key.set_value(k + 1);
        owner.with_value(|o| local_entry(o, k, prompt, id, text))
    };
    let lists: Vec<(Prompt, RwSignal<Vec<LocalEntry>>, RwSignal<bool>)> = Prompt::ALL
        .into_iter()
        .map(|p| {
            let passed = mine.iter().any(|m| m.prompt == p && m.passed);
            let mut entries: Vec<LocalEntry> = mine
                .iter()
                .filter(|m| m.prompt == p && !m.passed)
                .map(|m| make(p, Some(m.id.clone()), m.text.clone()))
                .collect();
            if entries.is_empty() {
                entries.push(make(p, None, String::new()));
            }
            (p, RwSignal::new(entries), RwSignal::new(passed))
        })
        .collect();
    let answered = {
        let lists = lists.clone();
        move || {
            lists
                .iter()
                .filter(|(_, l, passed)| {
                    passed.get() || l.with(|l| l.iter().any(|e| e.id.get().is_some()))
                })
                .count()
        }
    };
    view! {
        <p class="figures text-caption text-muted" data-answered="">{move || format!("{} of 4 answered", answered())}</p>
        {lists
            .into_iter()
            .map(|(prompt, entries, passed)| view! {
                <PromptSection ctx=ctx prompt=prompt entries=entries passed=passed make=Callback::new(move |(p, id, t): (Prompt, Option<String>, String)| make(p, id, t)) />
            })
            .collect_view()}
    }
}

#[component]
fn PromptSection(
    ctx: Ctx,
    prompt: Prompt,
    entries: RwSignal<Vec<LocalEntry>>,
    passed: RwSignal<bool>,
    make: Callback<(Prompt, Option<String>, String), LocalEntry>,
) -> impl IntoView {
    let heading = format!("write-{}", prompt.key());
    let has_saved =
        Signal::derive(move || entries.with(|l| l.iter().any(|e| e.id.get().is_some())));
    let add = Callback::new(move |_| {
        entries.update(|l| {
            if l.len() < ENTRIES_PER_PROMPT {
                l.push(make.run((prompt, None, String::new())));
            }
        });
    });
    let on_pass = Callback::new(move |v: bool| {
        let id = ctx.sid();
        leptos::task::spawn_local(async move {
            match pass_prompt(id, prompt.key().to_string(), v).await {
                Ok(()) => {
                    // Passing collapses the empty field; taking it back
                    // brings one empty field again.
                    entries.update(|l| {
                        l.retain(|e| e.id.get_untracked().is_some());
                        if l.is_empty() {
                            l.push(make.run((prompt, None, String::new())));
                        }
                    });
                    ctx.tracker.saved();
                }
                Err(e) => {
                    let _ = passed.try_set(!v);
                    ctx.error(&e);
                }
            }
        });
    });
    let remove = Callback::new(move |e: LocalEntry| {
        let text = e.text.get_untracked();
        let id = e.id.get_untracked();
        entries.update(|l| {
            l.retain(|x| x.key != e.key);
            if l.is_empty() {
                l.push(make.run((prompt, None, String::new())));
            }
        });
        if let Some(id) = id {
            let sid = ctx.sid();
            leptos::task::spawn_local(async move {
                if let Err(err) = delete_entry(sid, id).await {
                    ctx.error(&err);
                }
            });
        }
        if !text.trim().is_empty() {
            // Undo: the words were kept here; they are written again.
            let undo = Callback::new(move |_| {
                let back = make.run((prompt, None, text.clone()));
                entries.update(|l| {
                    l.retain(|x| {
                        x.id.get_untracked().is_some() || !x.text.get_untracked().trim().is_empty()
                    });
                    if l.len() < ENTRIES_PER_PROMPT {
                        l.push(back);
                    }
                });
                save_entry(ctx, back);
            });
            ctx.toast
                .push
                .run(Toast::success("Note removed.").with_action("Undo", undo));
        }
    });
    view! {
        <section class="flex flex-col gap-3 border-t border-rule pt-6" aria-labelledby=heading.clone() data-prompt=prompt.key()>
            <h2 id=heading.clone() class="font-display text-heading font-semibold text-primary">{prompt.label()}</h2>
            {prompt.is_anonymous().then(|| view! {
                <p class="-mt-1 text-label text-muted" data-anonymous-caption="">{crate::models::review_together::ANONYMOUS_CAPTION}</p>
            })}
            <Show when=move || !passed.get()>
                <div class="flex flex-col gap-4">
                    <For
                        each=move || entries.get()
                        key=|e| e.key
                        children=move |e| {
                            let n = entries.with_untracked(|l| l.iter().position(|x| x.key == e.key).unwrap_or(0) + 1);
                            view! { <EntryRow ctx=ctx entry=e label=format!("{}, note {n}", prompt.label()) on_remove=remove /> }
                        }
                    />
                    {move || (entries.with(|l| l.len()) < ENTRIES_PER_PROMPT).then(|| view! {
                        <div>
                            <Button variant=ButtonVariant::Quiet size=ButtonSize::Touch on_click=add attr:data-testid="add-note">"Add another"</Button>
                        </div>
                    })}
                </div>
            </Show>
            <Checkbox label="Nothing to add here" checked=passed touch=true disabled=has_saved on_change=on_pass />
        </section>
    }
}

#[component]
fn EntryRow(
    ctx: Ctx,
    entry: LocalEntry,
    label: String,
    on_remove: Callback<LocalEntry>,
) -> impl IntoView {
    let e = entry;
    // Autosave: a change of the words (not the first run) saves after 600 ms.
    Effect::new(move |prev: Option<String>| {
        let t = e.text.get();
        if prev.is_some_and(|p| p != t) {
            schedule_save(ctx, e);
        }
        t
    });
    on_cleanup(move || {
        #[cfg(feature = "hydrate")]
        if let Some(Some(id)) = e.timer.try_get_value() {
            if let Some(w) = web_sys::window() {
                w.clear_timeout_with_handle(id);
            }
        }
    });
    let chars = move || e.text.with(|t| t.chars().count());
    let blur = Callback::new(move |_| {
        #[cfg(feature = "hydrate")]
        if let Some(Some(id)) = e.timer.try_get_value() {
            if let Some(w) = web_sys::window() {
                w.clear_timeout_with_handle(id);
            }
            e.timer.set_value(None);
        }
        save_entry(ctx, e);
    });
    view! {
        <div class="flex flex-col gap-2" data-entry="">
            <TextField value=e.text rows=3 maxlength=TEXT_MAX as u32 aria_label=label on_blur=blur />
            <div class="flex min-h-11 items-center justify-between gap-2 md:min-h-0">
                <span class="figures text-caption text-muted">
                    {move || (chars() >= COUNT_FROM).then(|| format!("{}/{TEXT_MAX}", chars()))}
                </span>
                {move || (e.id.get().is_some() || !e.text.with(|t| t.trim().is_empty())).then(|| view! {
                    <Button variant=ButtonVariant::Quiet size=ButtonSize::Touch on_click=Callback::new(move |_| on_remove.run(e)) attr:data-testid="remove-note">"Remove"</Button>
                })}
            </div>
        </div>
    }
}

/// One row of a revealed group: a note, or a merged cluster of notes.
#[derive(Clone, PartialEq)]
struct Row {
    /// Stable key: the row's card ids joined.
    key: String,
    /// The card its one +1 and its proposal count on.
    root: String,
    /// (id, words, author) per note; no author for the anonymous prompt.
    notes: Vec<(String, String, Option<String>)>,
}

/// (id, cluster) or (words, author): two strings, the second optional.
type IdCluster = (String, Option<String>);

fn group_rows(s: &SessionState, prompt: Prompt) -> Vec<Row> {
    let Some(c) = &s.cards else {
        return Vec::new();
    };
    // (id, cluster) and (words, author) per card, in the server's order.
    let cards: Vec<(IdCluster, IdCluster)> = match prompt {
        Prompt::OffPlan => c
            .off_plan
            .iter()
            .map(|a| ((a.id.clone(), a.cluster.clone()), (a.text.clone(), None)))
            .collect(),
        Prompt::Decided | Prompt::ToPlan | Prompt::NextTime => {
            let list = match prompt {
                Prompt::Decided => &c.decided,
                Prompt::ToPlan => &c.to_plan,
                _ => &c.next_time,
            };
            list.iter()
                .map(|a| {
                    (
                        (a.id.clone(), a.cluster.clone()),
                        (a.text.clone(), Some(a.author_name.clone())),
                    )
                })
                .collect()
        }
    };
    let ids: Vec<(String, Option<String>)> = cards.iter().map(|(i, _)| i.clone()).collect();
    note_rows(&ids)
        .into_iter()
        .map(|row| {
            let root = ids[row_root(&ids, &row)].0.clone();
            let notes: Vec<(String, String, Option<String>)> = row
                .iter()
                .map(|&i| {
                    (
                        cards[i].0 .0.clone(),
                        cards[i].1 .0.clone(),
                        cards[i].1 .1.clone(),
                    )
                })
                .collect();
            Row {
                key: notes
                    .iter()
                    .map(|n| n.0.as_str())
                    .collect::<Vec<_>>()
                    .join("|"),
                root,
                notes,
            }
        })
        .collect()
}

/// A card's +1 count and whether the viewer gave one.
fn plus_of(s: &SessionState, id: &str) -> (u32, bool) {
    let Some(c) = &s.cards else {
        return (0, false);
    };
    c.decided
        .iter()
        .chain(c.to_plan.iter())
        .chain(c.next_time.iter())
        .find(|a| a.id == id)
        .map(|a| (a.plus_ones, a.i_plus_oned))
        .or_else(|| {
            c.off_plan
                .iter()
                .find(|a| a.id == id)
                .map(|a| (a.plus_ones, a.i_plus_oned))
        })
        .unwrap_or((0, false))
}

fn card_text(s: &SessionState, id: &str) -> String {
    let Some(c) = &s.cards else {
        return String::new();
    };
    c.decided
        .iter()
        .chain(c.to_plan.iter())
        .chain(c.next_time.iter())
        .find(|a| a.id == id)
        .map(|a| a.text.clone())
        .or_else(|| {
            c.off_plan
                .iter()
                .find(|a| a.id == id)
                .map(|a| a.text.clone())
        })
        .unwrap_or_default()
}

fn short(text: &str) -> String {
    if text.chars().count() <= 80 {
        text.to_string()
    } else {
        format!(
            "{}\u{2026}",
            text.chars().take(79).collect::<String>().trim_end()
        )
    }
}

/// The reveal (card 4): the four prompts as ruled lists of notes.
#[component]
fn RevealedView(ctx: Ctx) -> impl IntoView {
    let proposing: RwSignal<Option<String>> = RwSignal::new(None);
    let draft = RwSignal::new(String::new());
    let kept_apart: RwSignal<Vec<String>> = RwSignal::new(Vec::new());
    Prompt::ALL
        .into_iter()
        .map(|prompt| {
            let rows = Memo::new(move |_| ctx.state.with(|s| group_rows(s, prompt)));
            let count = Signal::derive(move || rows.with(|r| r.iter().map(|r| r.notes.len()).sum::<usize>()));
            view! {
                <NoteGroup
                    prompt=prompt
                    count=count
                    tools=move || view! { <Suggestions ctx=ctx prompt=prompt kept_apart=kept_apart /> }
                >
                    <For
                        each=move || rows.get()
                        key=|r| r.key.clone()
                        children=move |r| view! { <NoteRow ctx=ctx row=r proposing=proposing draft=draft /> }
                    />
                </NoteGroup>
            }
        })
        .collect_view()
}

/// The facilitator's near-duplicate line for one group, with Merge and
/// Keep apart per suggestion (texts only, never who wrote them).
#[component]
fn Suggestions(ctx: Ctx, prompt: Prompt, kept_apart: RwSignal<Vec<String>>) -> impl IntoView {
    let open = RwSignal::new(false);
    let list = Memo::new(move |_| {
        let hidden = kept_apart.get();
        ctx.state.with(|s| {
            if !s.i_am_facilitator {
                return Vec::new();
            }
            s.cards
                .as_ref()
                .map(|c| {
                    c.suggestions
                        .iter()
                        .filter(|g| g.prompt == prompt)
                        .map(|g| g.entry_ids.clone())
                        .filter(|ids| !hidden.contains(&ids.join("|")))
                        .map(|ids| {
                            let texts = ids
                                .iter()
                                .map(|i| short(&card_text(s, i)))
                                .collect::<Vec<_>>();
                            (ids, texts)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
    });
    let controls = format!("alike-{}", prompt.key());
    let label = Signal::derive(move || match list.with(|l| l.len()) {
        1 => "1 group of notes reads alike".to_string(),
        n => format!("{n} groups of notes read alike"),
    });
    view! {
        {move || (!list.with(|l| l.is_empty())).then(|| {
            let controls = controls.clone();
            view! {
                <div class="flex flex-col gap-2" data-suggestions="">
                    <Disclosure label=label open=open controls=controls.clone() />
                    <Show when=move || open.get()>
                        <ul id=controls.clone() class="flex flex-col gap-3">
                            {move || list.get().into_iter().map(|(ids, texts)| {
                                let key = ids.join("|");
                                let merge_ids = ids.clone();
                                let merge = Callback::new(move |_| {
                                    let sid = ctx.sid();
                                    let ids = merge_ids.clone();
                                    let target = ids[0].clone();
                                    let json = serde_json::to_string(&ids).unwrap_or_default();
                                    leptos::task::spawn_local(async move {
                                        match merge_entries(sid.clone(), target.clone(), json).await {
                                            Ok(()) => {
                                                ctx.refresh.run(());
                                                let undo = Callback::new(move |_| {
                                                    let sid = sid.clone();
                                                    let target = target.clone();
                                                    ctx.act(async move { unmerge(sid, target).await });
                                                });
                                                ctx.toast.push.run(Toast::success("Merged.").with_action("Undo", undo));
                                            }
                                            Err(e) => ctx.error(&e),
                                        }
                                    });
                                });
                                let apart = Callback::new(move |_| kept_apart.update(|k| k.push(key.clone())));
                                view! {
                                    <li class="flex flex-col gap-2 border-t border-rule pt-3">
                                        {texts.into_iter().map(|t| view! { <p class="text-label text-secondary [overflow-wrap:anywhere]">{t}</p> }).collect_view()}
                                        <div class="flex flex-wrap gap-2">
                                            <Button size=ButtonSize::Touch on_click=merge>"Merge"</Button>
                                            <Button variant=ButtonVariant::Quiet size=ButtonSize::Touch on_click=apart>"Keep apart"</Button>
                                        </div>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    </Show>
                </div>
            }
        })}
    }
}

/// One revealed row: its note (or merged notes), one +1 and "Propose".
#[component]
fn NoteRow(
    ctx: Ctx,
    row: Row,
    proposing: RwSignal<Option<String>>,
    draft: RwSignal<String>,
) -> impl IntoView {
    let root = StoredValue::new(row.root.clone());
    let merged = row.notes.len() > 1;
    let n = row.notes.len();
    let first_text = row.notes.first().map(|n| n.1.clone()).unwrap_or_default();
    let count = Signal::derive(move || ctx.state.with(|s| plus_of(s, &root.get_value()).0));
    let pressed = Signal::derive(move || ctx.state.with(|s| plus_of(s, &root.get_value()).1));
    let facilitator = Memo::new(move |_| ctx.state.with(|s| s.i_am_facilitator));
    let toggle = Callback::new(move |_| {
        let sid = ctx.sid();
        let id = root.get_value();
        ctx.act(async move { plus_one(sid, id).await.map(|_| ()) });
    });
    let open_propose = Callback::new(move |_| {
        draft.set(first_text.clone());
        proposing.set(Some(root.get_value()));
    });
    let separate = Callback::new(move |_| {
        let sid = ctx.sid();
        let id = root.get_value();
        ctx.act(async move { unmerge(sid, id).await });
    });
    let add_proposal = Callback::new(move |_| {
        let sid = ctx.sid();
        let id = root.get_value();
        let text = draft.get_untracked();
        leptos::task::spawn_local(async move {
            match create_proposal(sid, Some(id), text).await {
                Ok(_) => {
                    proposing.set(None);
                    ctx.refresh.run(());
                }
                Err(e) => ctx.error(&e),
            }
        });
    });
    let is_open = move || proposing.get().as_deref() == Some(root.get_value().as_str());
    view! {
        <li class="flex flex-col gap-2 py-4">
            {merged.then(|| view! { <p class="eyebrow">{format!("{n} said much the same")}</p> })}
            <div class="flex flex-col gap-3">
                {row.notes.into_iter().map(|(id, text, author)| view! {
                    <div class="flex flex-col gap-2" data-note-id=id>
                        <ReviewNote text=text author=author />
                    </div>
                }).collect_view()}
            </div>
            <div class="flex flex-wrap items-center justify-end gap-2">
                {move || (merged && facilitator.get()).then(|| view! {
                    <Button variant=ButtonVariant::Quiet size=ButtonSize::Touch on_click=separate>"Separate"</Button>
                })}
                <PlusOne count=count pressed=pressed on_toggle=toggle />
                <Button variant=ButtonVariant::Quiet size=ButtonSize::Touch on_click=open_propose attr:data-testid="propose">"Propose"</Button>
            </div>
            <Show when=is_open>
                <div class="flex flex-col gap-2" data-propose-form="">
                    <TextField value=draft rows=2 maxlength=TEXT_MAX as u32 aria_label="Proposal for an action item" />
                    <div class="flex flex-wrap gap-2">
                        <Button size=ButtonSize::Touch on_click=add_proposal attr:data-testid="add-proposal">"Add proposal"</Button>
                        <Button variant=ButtonVariant::Quiet size=ButtonSize::Touch on_click=Callback::new(move |_| proposing.set(None))>"Cancel"</Button>
                    </div>
                </div>
            </Show>
        </li>
    }
}

/// Decide (card 5): proposals, three votes each, the facilitator accepts at
/// most three and gives each an owner, then closes into the review.
#[component]
fn DecideSection(ctx: Ctx, members: Vec<String>) -> impl IntoView {
    let state = ctx.state;
    let members = StoredValue::new(members);
    let left = Signal::derive(move || state.with(|s| s.decide_votes_left));
    let facilitator = Memo::new(move |_| state.with(|s| s.i_am_facilitator));
    let accepted_n =
        Memo::new(move |_| state.with(|s| s.proposals.iter().filter(|p| p.accepted).count()));
    let ids = Memo::new(move |_| {
        state.with(|s| {
            s.proposals
                .iter()
                .map(|p| (p.id.clone(), p.text.clone()))
                .collect::<Vec<_>>()
        })
    });
    let can_close = Memo::new(move |_| {
        state.with(|s| {
            let acc: Vec<_> = s.proposals.iter().filter(|p| p.accepted).collect();
            (1..=ACCEPT_MAX).contains(&acc.len()) && acc.iter().all(|p| p.owner.is_some())
        })
    });
    let confirm_close = RwSignal::new(false);
    let close_msg = RwSignal::new(String::new());
    let ask_close = Callback::new(move |_| {
        let owners: Vec<String> = state.with_untracked(|s| {
            s.proposals
                .iter()
                .filter(|p| p.accepted)
                .filter_map(|p| p.owner.clone())
                .collect()
        });
        let n = owners.len();
        let names = match owners.as_slice() {
            [] => String::new(),
            [a] => a.clone(),
            [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
        };
        close_msg.set(format!(
            "This writes the review with {n} action item{}, owned by {names}. Notes and votes are locked. The items stay editable on the action items page.",
            if n == 1 { "" } else { "s" }
        ));
        confirm_close.set(true);
    });
    let do_close = Callback::new(move |_| {
        let sid = ctx.sid();
        leptos::task::spawn_local(async move {
            match close_session(sid).await {
                Ok(review) => go(ctx.nav, &format!("/post-game?review_id={review}")),
                Err(e) => ctx.error(&e),
            }
        });
    });
    view! {
        <section id="decide" class="flex scroll-mt-20 flex-col gap-3" aria-labelledby="decide-h" data-decide="">
            <div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
                <h2 id="decide-h" class="font-display text-title font-semibold tracking-[-0.01em] text-primary">"Decide"</h2>
                <VotesLeft left=left total=DECIDE_VOTES as u32 />
            </div>
            {move || (left.get() == 0).then(|| view! {
                <p class="text-caption text-muted">"Take a vote back to vote for something else."</p>
            })}
            {move || (facilitator.get() && accepted_n.get() >= ACCEPT_MAX).then(|| view! {
                <p class="text-caption text-muted">"Three accepted. Untick one to accept another."</p>
            })}
            {move || ids.with(|l| l.is_empty()).then(|| view! {
                <p class="text-label text-muted">"No proposals yet. Choose Propose under a note to suggest an action item."</p>
            })}
            <Show when=move || !ids.with(|l| l.is_empty())>
                <ul class="divide-y divide-rule border-y border-rule">
                    <For
                        each=move || ids.get()
                        key=|(id, _)| id.clone()
                        children=move |(id, text)| view! {
                            <ProposalRow ctx=ctx id=id text=text members=members.get_value() />
                        }
                    />
                </ul>
            </Show>
            {move || facilitator.get().then(|| view! {
                <div>
                    <Button variant=ButtonVariant::Primary size=ButtonSize::Touch disabled=Signal::derive(move || !can_close.get()) on_click=ask_close attr:data-testid="close-session">
                        "Close and write the review"
                    </Button>
                </div>
            })}
        </section>
        {move || {
            let msg = close_msg.get();
            view! {
                <ConfirmDialog
                    id="confirm-close"
                    open=confirm_close
                    title="Close and write the review?"
                    message=msg
                    confirm_label="Close and write the review"
                    on_confirm=do_close
                />
            }
        }}
    }
}

#[component]
fn ProposalRow(ctx: Ctx, id: String, text: String, members: Vec<String>) -> impl IntoView {
    let state = ctx.state;
    let id = StoredValue::new(id);
    let view_of = move || {
        state.with(|s| {
            s.proposals
                .iter()
                .find(|p| p.id == id.get_value())
                .map(|p| (p.votes, p.i_voted, p.accepted, p.owner.clone()))
                .unwrap_or((0, false, false, None))
        })
    };
    let votes = move || view_of().0;
    let voted = Signal::derive(move || view_of().1);
    let accepted = Memo::new(move |_| view_of().2);
    let owner = Signal::derive(move || view_of().3);
    let left = Signal::derive(move || state.with(|s| s.decide_votes_left));
    let facilitator = Memo::new(move |_| state.with(|s| s.i_am_facilitator));
    let accepted_n =
        Memo::new(move |_| state.with(|s| s.proposals.iter().filter(|p| p.accepted).count()));
    let acc = RwSignal::new(accepted.get_untracked());
    Effect::new(move |_| acc.set(accepted.get()));
    let vote = Callback::new(move |_| {
        let sid = ctx.sid();
        let pid = id.get_value();
        ctx.act(async move { decide_vote(sid, pid).await.map(|_| ()) });
    });
    let on_accept = Callback::new(move |v: bool| {
        let sid = ctx.sid();
        let pid = id.get_value();
        ctx.act(async move { set_accepted(sid, pid, v).await });
    });
    let pick = Callback::new(move |o: Option<String>| {
        let sid = ctx.sid();
        let pid = id.get_value();
        ctx.act(async move { set_owner(sid, pid, o).await });
    });
    let text_for_chips = text.clone();
    view! {
        <li class="flex flex-col gap-3 py-4" data-proposal-id=id.get_value()>
            <p class="text-body text-primary [overflow-wrap:anywhere]">{text}</p>
            <div class="flex flex-wrap items-center gap-x-4 gap-y-2">
                <span class="inline-flex items-center gap-2">
                    <Button
                        size=ButtonSize::Touch
                        pressed=voted
                        disabled=Signal::derive(move || !voted.get() && left.get() == 0)
                        on_click=vote
                        attr:data-testid="decide-vote"
                    >
                        {move || if voted.get() { "Voted" } else { "Vote" }}
                    </Button>
                    <span class="figures text-label text-secondary">
                        {move || match votes() { 1 => "1 vote".to_string(), n => format!("{n} votes") }}
                    </span>
                </span>
                {move || facilitator.get().then(|| view! {
                    <Checkbox
                        label="Accept"
                        checked=acc
                        touch=true
                        disabled=Signal::derive(move || !accepted.get() && accepted_n.get() >= ACCEPT_MAX)
                        on_change=on_accept
                    />
                })}
            </div>
            {move || accepted.get().then(|| {
                let line = view! {
                    <p class="text-label text-secondary" data-owner-line="">
                        {move || match owner.get() {
                            Some(o) => format!("Accepted \u{b7} owner {o}"),
                            None => "Accepted \u{b7} no owner yet".to_string(),
                        }}
                    </p>
                };
                if facilitator.get() {
                    view! {
                        <OwnerChips text=text_for_chips.clone() members=members.clone() owner=owner on_pick=pick />
                        {line}
                    }.into_any()
                } else {
                    line.into_any()
                }
            })}
        </li>
    }
}

/// After close: where the review is.
#[component]
fn ClosedView(ctx: Ctx) -> impl IntoView {
    let review = move || ctx.state.with(|s| s.review_id.clone());
    view! {
        {move || match review() {
            Some(r) => view! {
                <EmptyState
                    title="This session is closed"
                    message="The notes, the decided action items and the voices line are in the review it wrote."
                    action=move || {
                        let href = format!("/post-game?review_id={r}");
                        view! { <Button href=href size=ButtonSize::Touch>"Open the review"</Button> }
                    }
                />
            }.into_any(),
            None => view! {
                <EmptyState
                    title="This session is closed"
                    message="The review it wrote was deleted."
                />
            }.into_any(),
        }}
    }
}

/// The read-only section on a generated review (`/post-game?review_id=`,
/// card 6): the four groups as revealed, "N agreed" in place of +1, the
/// decided items, and exactly one team-level voices line.
#[component]
pub fn ReviewedTogether(view: ReviewSessionView) -> impl IntoView {
    use crate::components::local_time::LocalTime;
    let ReviewSessionView {
        cards,
        accepted,
        voices,
        match_title,
        closed_at,
        ..
    } = view;
    let state_like = SessionState {
        id: String::new(),
        rev: 0,
        phase: Phase::Closed,
        match_id: None,
        match_deleted: false,
        game_number: 1,
        series_id: None,
        writing_closes_at: String::new(),
        server_now: String::new(),
        timed_round: false,
        facilitator_name: String::new(),
        i_am_facilitator: false,
        can_take_over: false,
        review_id: None,
        written_count: 0,
        member_count: 0,
        mine: Vec::new(),
        cards: Some(cards),
        proposals: Vec::new(),
        decide_votes_left: 0,
        quiet: None,
        voices,
        present: Vec::new(),
    };
    let meta = voices.map(|v| v.label());
    view! {
        <section class="flex flex-col gap-8 border-y border-rule bg-ink-1 py-6" aria-labelledby="together-h" data-reviewed-together="">
            <div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 px-4">
                <h2 id="together-h" class="font-display text-title font-semibold tracking-[-0.01em] text-primary">"Reviewed together"</h2>
                <p class="flex flex-wrap items-baseline gap-x-2 text-label text-secondary" data-together-meta="">
                    {match_title.map(|t| view! { <span data-together-match="">{t}</span><span aria-hidden="true">"\u{b7}"</span> })}
                    {meta.map(|m| view! { <span class="figures" data-voices="">{m}</span> })}
                    {closed_at.map(|c| view! {
                        <span aria-hidden="true">"\u{b7}"</span>
                        <span>"closed "<LocalTime iso=c class="figures" /></span>
                    })}
                </p>
            </div>
            <div class="flex max-w-2xl flex-col gap-8 px-4">
                {Prompt::ALL.into_iter().map(|prompt| {
                    let rows = group_rows(&state_like, prompt);
                    let n: usize = rows.iter().map(|r| r.notes.len()).sum();
                    let plus: Vec<u32> = rows.iter().map(|r| plus_of(&state_like, &r.root).0).collect();
                    view! {
                        <NoteGroup prompt=prompt count=Signal::derive(move || n) nested=true>
                            {rows.into_iter().zip(plus).map(|(r, p)| {
                                let merged = r.notes.len() > 1;
                                let k = r.notes.len();
                                view! {
                                    <li class="flex flex-col gap-2 py-4">
                                        {merged.then(|| view! { <p class="eyebrow">{format!("{k} said much the same")}</p> })}
                                        <div class="flex flex-col gap-3">
                                            {r.notes.into_iter().map(|(id, text, author)| view! {
                                                <div class="flex flex-col gap-2" data-note-id=id>
                                                    <ReviewNote text=text author=author />
                                                </div>
                                            }).collect_view()}
                                        </div>
                                        <p class="figures text-caption text-muted">{format!("{p} agreed")}</p>
                                    </li>
                                }
                            }).collect_view()}
                        </NoteGroup>
                    }
                }).collect_view()}
                <section class="flex flex-col gap-3" aria-labelledby="decided-h">
                    <h3 id="decided-h" class="font-display text-heading font-semibold text-primary">"Decided"</h3>
                    {if accepted.is_empty() {
                        view! { <p class="text-label text-muted">"No action item was decided."</p> }.into_any()
                    } else {
                        view! {
                            <ul class="divide-y divide-rule border-y border-rule" data-decided="">
                                {accepted.into_iter().map(|a| view! {
                                    <li class="flex flex-col gap-1 py-3">
                                        <p class="text-body text-primary [overflow-wrap:anywhere]">{a.text}</p>
                                        <p class="text-label text-secondary">
                                            {match a.owner { Some(o) => format!("Owner: {o}"), None => "No owner".to_string() }}
                                        </p>
                                        <a href="/action-items" class="w-fit text-label text-secondary underline decoration-outline underline-offset-4 hover:text-primary">"Open in action items"</a>
                                    </li>
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }}
                </section>
            </div>
        </section>
    }
}
