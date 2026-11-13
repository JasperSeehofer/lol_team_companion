//! M165 "Review together": the server functions of a review session (the
//! page component is built on top of them). Every function starts with
//! `authz::authed()` and takes the team from the session
//! (`ctx.require_team()`); session, match, card and proposal ids from the
//! client are checked as records of that team in `server::review_together`
//! ("Not found" otherwise). Writes take `Bucket::ReviewWrite` per user; the
//! poll is not rate limited (a sync round polls every 2 s).
//! Signatures, payloads and phase rules: `.planning/missions/M165-*/server-api.md`.

use crate::models::review_together::{
    MatchSessionLink, OpenedSession, ReviewSessionView, SessionPatternTexts, SessionPoll,
};
use leptos::prelude::*;

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
    review_together::get_session_state(
        &ctx.db,
        &team_id,
        &ctx.user.id,
        &ctx.user.username,
        leads,
        &session_id,
        rev,
    )
    .await
    .map_err(authz::server_err)
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
    review_together::session_for_review(&ctx.db, &team_id, &ctx.user.id, &review_id)
        .await
        .map_err(authz::server_err)
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
