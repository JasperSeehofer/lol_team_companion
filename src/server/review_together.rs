//! M165 "Review together": the database side of a review session, team-scoped.
//! The shared types and pure rules are `models::review_together`; the server
//! functions (`authz::authed()` + `require_team()`, the team never from the
//! client) are `pages::review_together`.
//!
//! - **One session per game**: (scheduled match, game number), a unique index;
//!   opening again returns the existing one. The opener is the facilitator;
//!   the leader or a coach may take over.
//! - **Phases at read time**: a stored `writing` past `writing_closes_at` is
//!   revealed. The reveal's side effects (a random `order_key` per card, the
//!   phase written, `rev` bumped) run lazily and idempotently on the first
//!   read or write after the deadline, in one transaction under the
//!   invariant lock, so they survive restarts and need no background task.
//! - **Sealed**: before the reveal a member's state holds only their own
//!   cards and two team-level counts ([`assemble_state`]). After it, the
//!   anonymous prompt's cards travel as `AnonymousCard`, which has no author
//!   field. Cards are ordered by `order_key`, never by time; entry ids are
//!   random. No log line here names an entry together with a user.
//! - **`rev`**: bumped by every write that changes what any member sees; a
//!   poll with the current `rev` gets `Unchanged` (one record read).
//! - **Presence**: in memory only ([`touch_presence`]), 10 s after the last
//!   poll a member is gone; purged with the rate limiter's map. Never stored,
//!   never logged, not part of `rev` (it travels in `Unchanged` too).
//!
//! Every write takes `db::invariant_lock` for its check and write (per-prompt
//! card caps, one vote per card, three decide votes, three accepted, the
//! reveal and the close), as the rest of the code base does for rules that
//! span a read and a write.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Deserialize;
use surrealdb::{
    engine::local::Db,
    types::{Datetime, RecordId, SurrealValue, ToSql},
    Surreal,
};

use crate::models::match_day::game_count;
use crate::models::review_together::{
    clean_text, effective_phase, quiet_count, suggest_clusters, suggested_accepts, AcceptedItem,
    AnonymousCard, AttributedCard, ClusterInput, MatchSessionLink, OpenedSession, OwnEntry, Phase,
    Prompt, ProposalView, RevealedCards, ReviewSessionView, SessionPatternTexts, SessionPoll,
    SessionState, Voices, ACCEPT_MAX, ASYNC_HOURS, DECIDE_VOTES, ENTRIES_PER_PROMPT,
    QUIET_LOOKBACK, ROUND_MINUTES_DEFAULT, ROUND_MINUTES_MAX,
};
use crate::server::db::{
    self, check_tx, dt_to_string, invariant_lock, record_key, DbError, DbResult,
};

/// Shown when a card is written, changed or passed after the reveal.
pub const LOCKED: &str = "The cards are revealed; they can no longer change.";
/// Shown when someone else tries a facilitator's step.
pub const FACILITATOR_ONLY: &str = "Only the facilitator can do that.";
/// Shown when a step does not fit the session's phase.
pub const NOT_NOW: &str = "That step is not open in this phase of the review.";
/// Proposals one session may hold.
pub const PROPOSALS_MAX: usize = 30;

// ---------------------------------------------------------------------------
// Presence (in memory only)
// ---------------------------------------------------------------------------

/// A member counts as present this long after their last poll.
pub const PRESENCE_TTL: Duration = Duration::from_secs(10);

type PresenceMap = HashMap<String, HashMap<String, (String, Instant)>>;

/// session id -> user id -> (username, last poll).
fn presence() -> &'static Mutex<PresenceMap> {
    static PRESENCE: std::sync::OnceLock<Mutex<PresenceMap>> = std::sync::OnceLock::new();
    PRESENCE.get_or_init(Default::default)
}

/// Note a poll of `user_id` on `session_id`.
pub fn touch_presence(session_id: &str, user_id: &str, username: &str) {
    touch_presence_at(session_id, user_id, username, Instant::now());
}

fn touch_presence_at(session_id: &str, user_id: &str, username: &str, now: Instant) {
    let mut map = presence().lock().unwrap_or_else(|p| p.into_inner());
    map.entry(session_id.to_string())
        .or_default()
        .insert(user_id.to_string(), (username.to_string(), now));
}

/// Who polled `session_id` in the last [`PRESENCE_TTL`]: usernames,
/// alphabetical (never in arrival order).
pub fn present_names(session_id: &str) -> Vec<String> {
    present_names_at(session_id, Instant::now())
}

fn present_names_at(session_id: &str, now: Instant) -> Vec<String> {
    let map = presence().lock().unwrap_or_else(|p| p.into_inner());
    let mut names: Vec<String> = map
        .get(session_id)
        .map(|users| {
            users
                .values()
                .filter(|(_, at)| now.saturating_duration_since(*at) <= PRESENCE_TTL)
                .map(|(name, _)| name.clone())
                .collect()
        })
        .unwrap_or_default();
    names.sort_by_key(|n| n.to_lowercase());
    names.dedup();
    names
}

/// Drop expired presence entries and empty sessions; returns how many
/// entries went. Run by the rate limiter's purge task, off the request path.
pub fn purge_presence() -> usize {
    purge_presence_at(Instant::now())
}

fn purge_presence_at(now: Instant) -> usize {
    let mut map = presence().lock().unwrap_or_else(|p| p.into_inner());
    let mut removed = 0;
    map.retain(|_, users| {
        let before = users.len();
        users.retain(|_, (_, at)| now.saturating_duration_since(*at) <= PRESENCE_TTL);
        removed += before - users.len();
        !users.is_empty()
    });
    removed
}

// ---------------------------------------------------------------------------
// Rows
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct IdRow {
    id: RecordId,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbSession {
    id: RecordId,
    team: RecordId,
    #[serde(default)]
    scheduled_match: Option<RecordId>,
    #[serde(default)]
    match_format: Option<String>,
    #[serde(default)]
    series: Option<RecordId>,
    game_number: i64,
    #[serde(default)]
    match_riot_id: Option<String>,
    facilitator: RecordId,
    #[serde(default)]
    facilitator_name: Option<String>,
    phase: String,
    writing_closes_at: Datetime,
    async_closes_at: Datetime,
    #[serde(default)]
    round_started_at: Option<Datetime>,
    #[serde(default)]
    review: Option<RecordId>,
    #[serde(default)]
    voices: Option<i64>,
    #[serde(default)]
    members: Option<i64>,
    rev: i64,
    #[serde(default)]
    scheduled_match_deleted: Option<bool>,
}

/// One session as the assembly reads it (stored phase; see [`Session::phase_at`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub id: String,
    pub team_id: String,
    pub match_id: Option<String>,
    /// The match's format (`bo1`, `bo3`, `bo5`), None once it was deleted.
    pub match_format: Option<String>,
    pub match_deleted: bool,
    pub series_id: Option<String>,
    pub game_number: i32,
    pub match_riot_id: Option<String>,
    pub facilitator_id: String,
    pub facilitator_name: String,
    pub stored_phase: Phase,
    pub writing_closes_at: String,
    pub async_closes_at: String,
    pub timed_round: bool,
    pub review_id: Option<String>,
    pub voices: Option<Voices>,
    pub rev: i64,
}

impl Session {
    pub fn phase_at(&self, now: &str) -> Phase {
        effective_phase(self.stored_phase, &self.writing_closes_at, now)
    }

    /// The stored phase is `writing` but the deadline has passed: the
    /// reveal's side effects are due.
    fn reveal_due(&self, now: &str) -> bool {
        self.stored_phase == Phase::Writing && self.phase_at(now) == Phase::Revealed
    }
}

impl From<DbSession> for Session {
    fn from(s: DbSession) -> Self {
        let voices = match (s.voices, s.members) {
            (Some(v), Some(m)) => Some(Voices {
                voices: v.max(0) as u32,
                members: m.max(0) as u32,
            }),
            _ => None,
        };
        Session {
            id: s.id.to_sql(),
            team_id: s.team.to_sql(),
            match_id: s.scheduled_match.map(|m| m.to_sql()),
            match_format: s.match_format,
            match_deleted: s.scheduled_match_deleted.unwrap_or(false),
            series_id: s.series.map(|m| m.to_sql()),
            game_number: s.game_number as i32,
            match_riot_id: s.match_riot_id,
            facilitator_id: s.facilitator.to_sql(),
            facilitator_name: s
                .facilitator_name
                .unwrap_or_else(|| crate::server::account::FORMER_MEMBER_NAME.to_string()),
            // An unknown stored phase is treated as closed (nothing writes).
            stored_phase: Phase::from_key(&s.phase).unwrap_or(Phase::Closed),
            writing_closes_at: dt_to_string(&s.writing_closes_at),
            async_closes_at: dt_to_string(&s.async_closes_at),
            timed_round: s.round_started_at.is_some(),
            review_id: s.review.map(|r| r.to_sql()),
            voices,
            rev: s.rev,
        }
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbEntry {
    id: RecordId,
    author: RecordId,
    #[serde(default)]
    author_name: Option<String>,
    prompt: String,
    #[serde(default)]
    text: Option<String>,
    passed: bool,
    #[serde(default)]
    cluster: Option<RecordId>,
    #[serde(default)]
    order_key: Option<i64>,
}

/// One card or pass, with its author (server side only: the author never
/// leaves the server for an anonymous card).
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub prompt: Prompt,
    pub text: Option<String>,
    pub passed: bool,
    pub cluster: Option<String>,
    pub order_key: Option<i64>,
}

impl Entry {
    fn from_db(e: DbEntry) -> Option<Entry> {
        Some(Entry {
            id: e.id.to_sql(),
            author_id: e.author.to_sql(),
            author_name: e
                .author_name
                .unwrap_or_else(|| crate::server::account::FORMER_MEMBER_NAME.to_string()),
            prompt: Prompt::from_key(&e.prompt)?,
            text: e.text,
            passed: e.passed,
            cluster: e.cluster.map(|c| c.to_sql()),
            order_key: e.order_key,
        })
    }

    /// A card (not a pass) with text.
    fn is_card(&self) -> bool {
        !self.passed && self.text.as_deref().is_some_and(|t| !t.is_empty())
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbVote {
    target: RecordId,
    user: RecordId,
    kind: String,
}

/// A +1 or decide vote.
#[derive(Clone, Debug, PartialEq)]
pub struct Vote {
    pub target: String,
    pub user_id: String,
    pub decide: bool,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbProposal {
    id: RecordId,
    text: String,
    #[serde(default)]
    source: Option<RecordId>,
    accepted: bool,
    #[serde(default)]
    owner: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Proposal {
    pub id: String,
    pub text: String,
    pub source: Option<String>,
    pub accepted: bool,
    pub owner: Option<String>,
}

impl From<DbProposal> for Proposal {
    fn from(p: DbProposal) -> Self {
        Proposal {
            id: p.id.to_sql(),
            text: p.text,
            source: p.source.map(|s| s.to_sql()),
            accepted: p.accepted,
            owner: p.owner,
        }
    }
}

/// Everything one session's state is assembled from.
#[derive(Clone, Debug, PartialEq)]
pub struct Contents {
    pub entries: Vec<Entry>,
    pub votes: Vec<Vote>,
    pub proposals: Vec<Proposal>,
    /// The team's current members (user ids).
    pub members: Vec<String>,
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

// ---------------------------------------------------------------------------
// Reads
// ---------------------------------------------------------------------------

const SESSION_FIELDS: &str = "id, team, scheduled_match, scheduled_match.format AS match_format, series, game_number, match_riot_id, facilitator, facilitator.username AS facilitator_name, phase, writing_closes_at, async_closes_at, round_started_at, review, voices, members, rev, scheduled_match_deleted";

/// One session of `team_id`; another team's or an unknown id is `NotFound`.
pub async fn load_session(db: &Surreal<Db>, team_id: &str, session_id: &str) -> DbResult<Session> {
    let key = record_key(session_id.trim(), "review_session");
    if key.is_empty() {
        return Err(DbError::NotFound);
    }
    let mut r = db
        .query(format!(
            "SELECT {SESSION_FIELDS} FROM type::record('review_session', $key) WHERE team = type::record('team', $team_key)"
        ))
        .bind(("key", key))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let rows: Vec<DbSession> = r.take(0)?;
    rows.into_iter()
        .next()
        .map(Session::from)
        .ok_or(DbError::NotFound)
}

/// The entries, votes, proposals and current members behind a session.
pub async fn load_contents(db: &Surreal<Db>, session: &Session) -> DbResult<Contents> {
    let mut r = db
        .query(
            "LET $s = type::record('review_session', $key); \
             SELECT id, author, author.username AS author_name, prompt, text, passed, cluster, order_key FROM review_entry WHERE session = $s; \
             SELECT target, user, kind FROM review_vote WHERE session = $s; \
             SELECT id, text, source, accepted, owner FROM review_proposal WHERE session = $s; \
             SELECT VALUE user FROM team_member WHERE team = type::record('team', $team_key);",
        )
        .bind(("key", record_key(&session.id, "review_session")))
        .bind(("team_key", record_key(&session.team_id, "team")))
        .await?;
    let entries: Vec<DbEntry> = r.take(1)?;
    let votes: Vec<DbVote> = r.take(2)?;
    let proposals: Vec<DbProposal> = r.take(3)?;
    let members: Vec<RecordId> = r.take(4)?;
    Ok(Contents {
        entries: entries.into_iter().filter_map(Entry::from_db).collect(),
        votes: votes
            .into_iter()
            .map(|v| Vote {
                target: v.target.to_sql(),
                user_id: v.user.to_sql(),
                decide: v.kind == "decide",
            })
            .collect(),
        proposals: proposals.into_iter().map(Proposal::from).collect(),
        members: members.iter().map(|m| m.to_sql()).collect(),
    })
}

/// Write the reveal's side effects if the session is still stored as
/// `writing` (idempotent): a random `order_key` on every entry, the phase,
/// `revealed_at`, a deadline no later than now, and `rev`. The caller holds
/// the invariant lock and has decided the reveal is due.
async fn reveal_locked(db: &Surreal<Db>, session_id: &str) -> DbResult<()> {
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             LET $s = type::record('review_session', $key); \
             LET $row = (SELECT phase FROM ONLY $s); \
             IF $row.phase = 'writing' { \
                 UPDATE review_entry SET order_key = rand::int(0, 2147483647) WHERE session = $s; \
                 UPDATE $s SET phase = 'revealed', revealed_at = time::now(), writing_closes_at = IF writing_closes_at > time::now() { time::now() } ELSE { writing_closes_at }, rev += 1; \
             }; \
             COMMIT TRANSACTION;",
        )
        .bind(("key", record_key(session_id, "review_session")))
        .await?;
    check_tx(response)?;
    Ok(())
}

/// Load a session and apply a due reveal (taking the lock only then).
async fn load_settled(db: &Surreal<Db>, team_id: &str, session_id: &str) -> DbResult<Session> {
    let s = load_session(db, team_id, session_id).await?;
    if !s.reveal_due(&now_rfc3339()) {
        return Ok(s);
    }
    {
        let _guard = invariant_lock().await;
        let again = load_session(db, team_id, session_id).await?;
        if again.reveal_due(&now_rfc3339()) {
            reveal_locked(db, &again.id).await?;
        }
    }
    load_session(db, team_id, session_id).await
}

/// `load_settled` for a caller that already holds the invariant lock.
async fn load_settled_locked(
    db: &Surreal<Db>,
    team_id: &str,
    session_id: &str,
) -> DbResult<Session> {
    let s = load_session(db, team_id, session_id).await?;
    if s.reveal_due(&now_rfc3339()) {
        reveal_locked(db, &s.id).await?;
        return load_session(db, team_id, session_id).await;
    }
    Ok(s)
}

/// The poll: `Unchanged` (with presence) when `rev` is the session's, else
/// the full state for `viewer_id`. Notes the viewer's presence. A due reveal
/// is written first, so it changes `rev`.
#[allow(clippy::too_many_arguments)]
pub async fn get_session_state(
    db: &Surreal<Db>,
    team_id: &str,
    viewer_id: &str,
    viewer_name: &str,
    viewer_leads: bool,
    session_id: &str,
    rev: Option<i64>,
) -> DbResult<SessionPoll> {
    let s = load_settled(db, team_id, session_id).await?;
    touch_presence(&s.id, viewer_id, viewer_name);
    let present = present_names(&s.id);
    if rev == Some(s.rev) {
        return Ok(SessionPoll::Unchanged {
            rev: s.rev,
            present,
        });
    }
    let contents = load_contents(db, &s).await?;
    let now = now_rfc3339();
    let quiet = if s.facilitator_id == viewer_id && s.phase_at(&now) == Phase::Writing {
        quiet_nudge(db, team_id).await?
    } else {
        None
    };
    Ok(SessionPoll::Full(Box::new(assemble_state(
        &s,
        &contents,
        viewer_id,
        viewer_leads,
        &now,
        quiet,
        present,
    ))))
}

/// The revealed cards with counts for `viewer_id`; suggestions only when
/// `with_suggestions` (the facilitator).
pub fn revealed_cards(
    contents: &Contents,
    viewer_id: &str,
    with_suggestions: bool,
) -> RevealedCards {
    let mut cards: Vec<&Entry> = contents.entries.iter().filter(|e| e.is_card()).collect();
    // The random key written at the reveal, then the random id: never time.
    cards.sort_by(|a, b| {
        (a.order_key.is_none(), a.order_key, &a.id).cmp(&(
            b.order_key.is_none(),
            b.order_key,
            &b.id,
        ))
    });
    let plus = |id: &str| -> (u32, bool) {
        let votes = contents
            .votes
            .iter()
            .filter(|v| !v.decide && v.target == id);
        let mut n = 0;
        let mut mine = false;
        for v in votes {
            n += 1;
            mine |= v.user_id == viewer_id;
        }
        (n, mine)
    };
    let attributed = |prompt: Prompt| -> Vec<AttributedCard> {
        cards
            .iter()
            .filter(|e| e.prompt == prompt)
            .map(|e| {
                let (plus_ones, i_plus_oned) = plus(&e.id);
                AttributedCard {
                    id: e.id.clone(),
                    text: e.text.clone().unwrap_or_default(),
                    author_name: e.author_name.clone(),
                    plus_ones,
                    i_plus_oned,
                    cluster: e.cluster.clone(),
                }
            })
            .collect()
    };
    let off_plan = cards
        .iter()
        .filter(|e| e.prompt == Prompt::OffPlan)
        .map(|e| {
            let (plus_ones, i_plus_oned) = plus(&e.id);
            AnonymousCard {
                id: e.id.clone(),
                text: e.text.clone().unwrap_or_default(),
                plus_ones,
                i_plus_oned,
                cluster: e.cluster.clone(),
            }
        })
        .collect();
    let suggestions = if with_suggestions {
        let inputs: Vec<ClusterInput> = cards
            .iter()
            .map(|e| ClusterInput {
                id: e.id.clone(),
                prompt: e.prompt,
                text: e.text.clone().unwrap_or_default(),
                cluster: e.cluster.clone(),
            })
            .collect();
        suggest_clusters(&inputs)
    } else {
        Vec::new()
    };
    RevealedCards {
        decided: attributed(Prompt::Decided),
        to_plan: attributed(Prompt::ToPlan),
        off_plan,
        next_time: attributed(Prompt::NextTime),
        suggestions,
    }
}

/// Members with at least one card or pass (any prompt), of current members.
fn written_count(contents: &Contents) -> u32 {
    let authors: HashSet<&str> = contents
        .entries
        .iter()
        .map(|e| e.author_id.as_str())
        .collect();
    contents
        .members
        .iter()
        .filter(|m| authors.contains(m.as_str()))
        .count() as u32
}

/// Current members with at least one card (a pass is not a voice).
fn voice_count(contents: &Contents) -> u32 {
    let authors: HashSet<&str> = contents
        .entries
        .iter()
        .filter(|e| e.is_card())
        .map(|e| e.author_id.as_str())
        .collect();
    contents
        .members
        .iter()
        .filter(|m| authors.contains(m.as_str()))
        .count() as u32
}

/// The state one member sees (the sealing rule): while writing, only the
/// viewer's own entries and the team-level counts; after the reveal, every
/// card (anonymous ones without author), proposals with counts, and no
/// "mine" of any kind. `quiet` is passed only to the facilitator while
/// writing.
pub fn assemble_state(
    s: &Session,
    contents: &Contents,
    viewer_id: &str,
    viewer_leads: bool,
    now: &str,
    quiet: Option<u8>,
    present: Vec<String>,
) -> SessionState {
    let phase = s.phase_at(now);
    let is_facilitator = s.facilitator_id == viewer_id;
    let writing = phase == Phase::Writing;
    let mine = if writing {
        contents
            .entries
            .iter()
            .filter(|e| e.author_id == viewer_id)
            .map(|e| OwnEntry {
                id: e.id.clone(),
                prompt: e.prompt,
                text: e.text.clone().unwrap_or_default(),
                passed: e.passed,
            })
            .collect()
    } else {
        Vec::new()
    };
    let (cards, proposals) = if writing {
        (None, Vec::new())
    } else {
        let votes_of = |id: &str| -> (u32, bool) {
            let mut n = 0;
            let mut mine = false;
            for v in contents.votes.iter().filter(|v| v.decide && v.target == id) {
                n += 1;
                mine |= v.user_id == viewer_id;
            }
            (n, mine)
        };
        let counted: Vec<(String, u32)> = contents
            .proposals
            .iter()
            .map(|p| (p.id.clone(), votes_of(&p.id).0))
            .collect();
        let suggested = suggested_accepts(&counted);
        let mut proposals: Vec<ProposalView> = contents
            .proposals
            .iter()
            .map(|p| {
                let (votes, i_voted) = votes_of(&p.id);
                ProposalView {
                    id: p.id.clone(),
                    text: p.text.clone(),
                    source: p.source.clone(),
                    votes,
                    i_voted,
                    accepted: p.accepted,
                    suggested: suggested.contains(&p.id),
                    owner: p.owner.clone(),
                }
            })
            .collect();
        proposals.sort_by(|a, b| b.votes.cmp(&a.votes).then_with(|| a.id.cmp(&b.id)));
        (
            Some(revealed_cards(contents, viewer_id, is_facilitator)),
            proposals,
        )
    };
    let my_decides = contents
        .votes
        .iter()
        .filter(|v| v.decide && v.user_id == viewer_id)
        .count();
    SessionState {
        id: s.id.clone(),
        rev: s.rev,
        phase,
        match_id: s.match_id.clone(),
        match_deleted: s.match_deleted,
        game_number: s.game_number,
        series_id: s.series_id.clone(),
        writing_closes_at: s.writing_closes_at.clone(),
        server_now: now.to_string(),
        timed_round: s.timed_round,
        facilitator_name: s.facilitator_name.clone(),
        i_am_facilitator: is_facilitator,
        can_take_over: viewer_leads && !is_facilitator && phase != Phase::Closed,
        review_id: s.review_id.clone(),
        written_count: written_count(contents),
        member_count: contents.members.len() as u32,
        mine,
        cards,
        proposals,
        decide_votes_left: (DECIDE_VOTES.saturating_sub(my_decides)) as u32,
        quiet: if is_facilitator && writing {
            quiet
        } else {
            None
        },
        voices: s.voices,
        present,
    }
}

/// The quiet nudge for the team: computed on request, never stored; a count
/// only. Members whose account is newer than the oldest of the sessions
/// looked at cannot have been quiet in it and are left out.
pub async fn quiet_nudge(db: &Surreal<Db>, team_id: &str) -> DbResult<Option<u8>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    #[allow(dead_code)] // closed_at is selected for ORDER BY only
    struct Closed {
        id: RecordId,
        opened_at: Datetime,
        closed_at: Option<Datetime>,
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Wrote {
        session: RecordId,
        author: RecordId,
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Member {
        user: RecordId,
        created: Option<Datetime>,
    }
    let mut r = db
        .query(
            "LET $t = type::record('team', $team_key); \
             SELECT id, opened_at, closed_at FROM review_session WHERE team = $t AND phase = 'closed' ORDER BY closed_at DESC LIMIT $n; \
             SELECT user, user.created_at AS created FROM team_member WHERE team = $t;",
        )
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("n", QUIET_LOOKBACK as i64))
        .await?;
    let closed: Vec<Closed> = r.take(1)?;
    let members: Vec<Member> = r.take(2)?;
    if closed.len() < QUIET_LOOKBACK {
        return Ok(None);
    }
    let ids: Vec<RecordId> = closed.iter().map(|c| c.id.clone()).collect();
    let mut r = db
        .query("SELECT session, author FROM review_entry WHERE session IN $ids AND passed = false")
        .bind(("ids", ids))
        .await?;
    let wrote: Vec<Wrote> = r.take(0)?;
    let at = |d: &Datetime| chrono::DateTime::parse_from_rfc3339(&dt_to_string(d)).ok();
    let oldest = closed.iter().filter_map(|c| at(&c.opened_at)).min();
    let members: Vec<String> = members
        .into_iter()
        .filter(|m| match (m.created.as_ref().and_then(at), oldest) {
            (Some(created), Some(oldest)) => created <= oldest,
            _ => true,
        })
        .map(|m| m.user.to_sql())
        .collect();
    let sessions: Vec<Vec<String>> = closed
        .iter()
        .map(|c| {
            wrote
                .iter()
                .filter(|w| w.session == c.id)
                .map(|w| w.author.to_sql())
                .collect()
        })
        .collect();
    Ok(quiet_count(&members, &sessions))
}

/// What the review page shows of the session that generated `review_id`
/// (team-scoped; `None` for a review not made by a session).
pub async fn session_for_review(
    db: &Surreal<Db>,
    team_id: &str,
    viewer_id: &str,
    review_id: &str,
) -> DbResult<Option<ReviewSessionView>> {
    db::ensure_team_record(db, "post_game_learning", review_id, team_id).await?;
    let mut r = db
        .query("SELECT VALUE id FROM review_session WHERE team = type::record('team', $team_key) AND review = type::record('post_game_learning', $review_key) LIMIT 1")
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("review_key", record_key(review_id, "post_game_learning")))
        .await?;
    let ids: Vec<RecordId> = r.take(0)?;
    let Some(id) = ids.into_iter().next() else {
        return Ok(None);
    };
    let s = load_session(db, team_id, &id.to_sql()).await?;
    let contents = load_contents(db, &s).await?;
    let mut accepted: Vec<&Proposal> = contents.proposals.iter().filter(|p| p.accepted).collect();
    let votes = |id: &str| {
        contents
            .votes
            .iter()
            .filter(|v| v.decide && v.target == id)
            .count()
    };
    accepted.sort_by(|a, b| {
        votes(&b.id)
            .cmp(&votes(&a.id))
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(Some(ReviewSessionView {
        session_id: s.id.clone(),
        cards: revealed_cards(&contents, viewer_id, false),
        accepted: accepted
            .into_iter()
            .map(|p| AcceptedItem {
                text: p.text.clone(),
                owner: p.owner.clone(),
            })
            .collect(),
        voices: s.voices,
    }))
}

/// The closed sessions' "went to plan" and "didn't go to plan" texts, for
/// the review page's patterns (texts only).
pub async fn session_pattern_texts(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<SessionPatternTexts>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        session: RecordId,
        review: Option<RecordId>,
        prompt: String,
        text: Option<String>,
    }
    let mut r = db
        .query("SELECT session, session.review AS review, prompt, text FROM review_entry WHERE team = type::record('team', $team_key) AND passed = false AND prompt IN ['to_plan', 'off_plan'] AND session.phase = 'closed'")
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let rows: Vec<Row> = r.take(0)?;
    let mut out: Vec<(String, SessionPatternTexts)> = Vec::new();
    for row in rows {
        let Some(text) = row.text.filter(|t| !t.trim().is_empty()) else {
            continue;
        };
        let key = row.session.to_sql();
        let idx = match out.iter().position(|(k, _)| *k == key) {
            Some(i) => i,
            None => {
                out.push((
                    key,
                    SessionPatternTexts {
                        review_id: row.review.map(|r| r.to_sql()),
                        ..Default::default()
                    },
                ));
                out.len() - 1
            }
        };
        if row.prompt == "to_plan" {
            out[idx].1.to_plan.push(text);
        } else {
            out[idx].1.off_plan.push(text);
        }
    }
    Ok(out.into_iter().map(|(_, t)| t).collect())
}

// ---------------------------------------------------------------------------
// Writes
// ---------------------------------------------------------------------------

/// Bump `rev` (a write that changes what members see).
const BUMP: &str = "UPDATE type::record('review_session', $skey) SET rev += 1;";

fn require_facilitator(s: &Session, user_id: &str) -> DbResult<()> {
    if s.facilitator_id == user_id {
        Ok(())
    } else {
        Err(DbError::Invalid(FACILITATOR_ONLY.into()))
    }
}

/// The session is writing (deadline not passed): else [`LOCKED`].
fn require_writing(s: &Session) -> DbResult<()> {
    if s.phase_at(&now_rfc3339()) == Phase::Writing {
        Ok(())
    } else {
        Err(DbError::Invalid(LOCKED.into()))
    }
}

/// The session is revealed or deciding: else [`NOT_NOW`].
fn require_open(s: &Session) -> DbResult<()> {
    if s.phase_at(&now_rfc3339()).is_open_after_reveal() {
        Ok(())
    } else {
        Err(DbError::Invalid(NOT_NOW.into()))
    }
}

/// A Riot match id as carried: letters, digits and `_`, at most 40.
fn clean_riot_id(raw: Option<String>) -> DbResult<Option<String>> {
    match raw.map(|r| r.trim().to_string()).filter(|r| !r.is_empty()) {
        None => Ok(None),
        Some(r) if r.len() <= 40 && r.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') => {
            Ok(Some(r))
        }
        Some(_) => Err(DbError::Invalid("That game id is not valid.".into())),
    }
}

/// Open the session for game `game_number` (1 for a Bo1; `None` means 1) of
/// the team's scheduled match, or return the one that exists. The opener is
/// the facilitator. Writing closes 24 h after the match start, or 24 h from
/// now when that has passed. The match's series is copied when it has one.
pub async fn open_session(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    match_id: &str,
    game_number: Option<i32>,
    match_riot_id: Option<String>,
) -> DbResult<OpenedSession> {
    let m = db::get_scheduled_match(db, team_id, match_id).await?;
    let game = game_number.unwrap_or(1);
    if !(1..=game_count(&m.format)).contains(&game) {
        return Err(DbError::Invalid(
            "That game is not part of this match.".into(),
        ));
    }
    let riot_id = clean_riot_id(match_riot_id)?;
    let starts = chrono::DateTime::parse_from_rfc3339(&m.starts_at)
        .map_err(|e| DbError::Other(format!("match start: {e}")))?
        .with_timezone(&chrono::Utc);
    let now = chrono::Utc::now();
    let mut closes = starts + chrono::Duration::hours(ASYNC_HOURS);
    if closes <= now {
        closes = now + chrono::Duration::hours(ASYNC_HOURS);
    }
    let _guard = invariant_lock().await;
    let mut r = db
        .query("SELECT VALUE id FROM review_session WHERE team = type::record('team', $team_key) AND scheduled_match = type::record('scheduled_match', $match_key) AND game_number = $game LIMIT 1")
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("match_key", record_key(&m.id, "scheduled_match")))
        .bind(("game", game as i64))
        .await?;
    let existing: Vec<RecordId> = r.take(0)?;
    if let Some(id) = existing.into_iter().next() {
        return Ok(OpenedSession {
            session_id: id.to_sql(),
            created: false,
        });
    }
    let closes = closes.to_rfc3339();
    let mut r = db
        .query("CREATE review_session SET team = type::record('team', $team_key), scheduled_match = type::record('scheduled_match', $match_key), series = IF $series_key = NONE { NONE } ELSE { type::record('series', $series_key) }, game_number = $game, match_riot_id = $riot_id, facilitator = type::record('user', $user_key), phase = 'writing', writing_closes_at = <datetime>$closes, async_closes_at = <datetime>$closes, rev = 1 RETURN id")
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("match_key", record_key(&m.id, "scheduled_match")))
        .bind(("series_key", db::opt_key(m.series_id.clone(), "series")))
        .bind(("game", game as i64))
        .bind(("riot_id", riot_id))
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("closes", closes))
        .await?;
    let row: Option<IdRow> = r.take(0)?;
    let id = row.ok_or_else(|| DbError::Other("Failed to open the review session".into()))?;
    Ok(OpenedSession {
        session_id: id.id.to_sql(),
        created: true,
    })
}

/// The leader or a coach becomes the facilitator (`viewer_leads` comes from
/// `db::is_team_leader_or_coach`, checked by the caller).
pub async fn take_over(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    viewer_leads: bool,
    session_id: &str,
) -> DbResult<()> {
    if !viewer_leads {
        return Err(DbError::Invalid(
            "Only the team leader or a coach can take over.".into(),
        ));
    }
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    if s.stored_phase == Phase::Closed {
        return Err(DbError::Invalid(NOT_NOW.into()));
    }
    db.query("UPDATE type::record('review_session', $skey) SET facilitator = type::record('user', $user_key), rev += 1")
        .bind(("skey", record_key(&s.id, "review_session")))
        .bind(("user_key", record_key(user_id, "user")))
        .await?
        .check()?;
    Ok(())
}

/// Write a card for `prompt`, or change one of the caller's own (`entry_id`)
/// while writing. At most [`ENTRIES_PER_PROMPT`] cards per prompt; writing
/// a card withdraws a pass on that prompt. Returns the card's id.
pub async fn upsert_entry(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
    prompt: Prompt,
    entry_id: Option<String>,
    text: &str,
) -> DbResult<String> {
    let text = clean_text(text).map_err(DbError::Invalid)?;
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_writing(&s)?;
    let skey = record_key(&s.id, "review_session");
    let ukey = record_key(user_id, "user");
    if let Some(entry_id) = entry_id.filter(|e| !e.trim().is_empty()) {
        let ekey = record_key(entry_id.trim(), "review_entry");
        let mut r = db
            .query(format!(
                "BEGIN TRANSACTION; \
                 UPDATE type::record('review_entry', $ekey) SET text = $text, updated_at = time::now() WHERE session = type::record('review_session', $skey) AND author = type::record('user', $ukey) AND passed = false RETURN id; \
                 {BUMP} \
                 COMMIT TRANSACTION;"
            ))
            .bind(("ekey", ekey))
            .bind(("skey", skey))
            .bind(("ukey", ukey))
            .bind(("text", text))
            .await
            .map_err(DbError::from)
            .and_then(check_tx)?;
        let rows: Vec<IdRow> = r.take(1)?;
        return rows
            .into_iter()
            .next()
            .map(|r| r.id.to_sql())
            .ok_or(DbError::NotFound);
    }
    let mut r = db
        .query("SELECT count() AS n FROM review_entry WHERE session = type::record('review_session', $skey) AND author = type::record('user', $ukey) AND prompt = $prompt AND passed = false GROUP ALL")
        .bind(("skey", skey.clone()))
        .bind(("ukey", ukey.clone()))
        .bind(("prompt", prompt.key()))
        .await?;
    #[derive(Debug, Deserialize, SurrealValue)]
    struct N {
        n: i64,
    }
    let n: Option<N> = r.take(0)?;
    if n.map(|n| n.n).unwrap_or(0) as usize >= ENTRIES_PER_PROMPT {
        return Err(DbError::Invalid(format!(
            "At most {ENTRIES_PER_PROMPT} cards per prompt."
        )));
    }
    let mut r = db
        .query(format!(
            "BEGIN TRANSACTION; \
             DELETE review_entry WHERE session = type::record('review_session', $skey) AND author = type::record('user', $ukey) AND prompt = $prompt AND passed = true; \
             CREATE review_entry SET session = type::record('review_session', $skey), team = type::record('team', $team_key), author = type::record('user', $ukey), prompt = $prompt, text = $text, passed = false RETURN id; \
             {BUMP} \
             COMMIT TRANSACTION;"
        ))
        .bind(("skey", skey))
        .bind(("ukey", ukey))
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("prompt", prompt.key()))
        .bind(("text", text))
        .await
        .map_err(DbError::from)
        .and_then(check_tx)?;
    let rows: Vec<IdRow> = r.take(2)?;
    rows.into_iter()
        .next()
        .map(|r| r.id.to_sql())
        .ok_or_else(|| DbError::Other("Failed to save the card".into()))
}

/// Delete one of the caller's own cards (or a pass) while writing.
pub async fn delete_entry(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
    entry_id: &str,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_writing(&s)?;
    let ekey = record_key(entry_id.trim(), "review_entry");
    let skey = record_key(&s.id, "review_session");
    let ukey = record_key(user_id, "user");
    let mut r = db
        .query("SELECT id FROM type::record('review_entry', $ekey) WHERE session = type::record('review_session', $skey) AND author = type::record('user', $ukey)")
        .bind(("ekey", ekey.clone()))
        .bind(("skey", skey.clone()))
        .bind(("ukey", ukey))
        .await?;
    let own: Vec<IdRow> = r.take(0)?;
    if own.is_empty() {
        return Err(DbError::NotFound);
    }
    let r = db
        .query(format!(
            "BEGIN TRANSACTION; \
             DELETE type::record('review_entry', $ekey); \
             {BUMP} \
             COMMIT TRANSACTION;"
        ))
        .bind(("ekey", ekey))
        .bind(("skey", skey))
        .await?;
    check_tx(r)?;
    Ok(())
}

/// Pass on a prompt (`passed`), or take the pass back, while writing. A
/// prompt with cards cannot be passed: delete them first.
pub async fn pass_prompt(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
    prompt: Prompt,
    passed: bool,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_writing(&s)?;
    let skey = record_key(&s.id, "review_session");
    let ukey = record_key(user_id, "user");
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Own {
        passed: bool,
    }
    let mut r = db
        .query("SELECT passed FROM review_entry WHERE session = type::record('review_session', $skey) AND author = type::record('user', $ukey) AND prompt = $prompt")
        .bind(("skey", skey.clone()))
        .bind(("ukey", ukey.clone()))
        .bind(("prompt", prompt.key()))
        .await?;
    let own: Vec<Own> = r.take(0)?;
    let has_pass = own.iter().any(|o| o.passed);
    let has_cards = own.iter().any(|o| !o.passed);
    let q = match (passed, has_pass, has_cards) {
        (true, _, true) => {
            return Err(DbError::Invalid(
                "Delete your cards for this prompt first, then pass.".into(),
            ))
        }
        (true, true, _) | (false, false, _) => return Ok(()),
        (true, false, false) => format!(
            "BEGIN TRANSACTION; \
             CREATE review_entry SET session = type::record('review_session', $skey), team = type::record('team', $team_key), author = type::record('user', $ukey), prompt = $prompt, text = NONE, passed = true; \
             {BUMP} \
             COMMIT TRANSACTION;"
        ),
        (false, true, _) => format!(
            "BEGIN TRANSACTION; \
             DELETE review_entry WHERE session = type::record('review_session', $skey) AND author = type::record('user', $ukey) AND prompt = $prompt AND passed = true; \
             {BUMP} \
             COMMIT TRANSACTION;"
        ),
    };
    let r = db
        .query(q)
        .bind(("skey", skey))
        .bind(("ukey", ukey))
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("prompt", prompt.key()))
        .await?;
    check_tx(r)?;
    Ok(())
}

/// Facilitator: start a timed silent round of `minutes` (default 3, at most
/// 30). It only moves the deadline earlier: members writing on their own
/// are unaffected until it passes.
pub async fn start_timed_round(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
    minutes: Option<u32>,
) -> DbResult<()> {
    let minutes = minutes
        .unwrap_or(ROUND_MINUTES_DEFAULT)
        .clamp(1, ROUND_MINUTES_MAX);
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_facilitator(&s, user_id)?;
    require_writing(&s)?;
    let ends = (chrono::Utc::now() + chrono::Duration::minutes(minutes as i64)).to_rfc3339();
    db.query("UPDATE type::record('review_session', $skey) SET writing_closes_at = IF <datetime>$ends < writing_closes_at { <datetime>$ends } ELSE { writing_closes_at }, round_started_at = time::now(), rev += 1")
        .bind(("skey", record_key(&s.id, "review_session")))
        .bind(("ends", ends))
        .await?
        .check()?;
    Ok(())
}

/// Facilitator: one more minute for a running timed round (never past the
/// async deadline).
pub async fn add_minute(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_facilitator(&s, user_id)?;
    require_writing(&s)?;
    if !s.timed_round {
        return Err(DbError::Invalid("Start a timed round first.".into()));
    }
    db.query("UPDATE type::record('review_session', $skey) SET writing_closes_at = IF writing_closes_at + 1m > async_closes_at { async_closes_at } ELSE { writing_closes_at + 1m }, rev += 1")
        .bind(("skey", record_key(&s.id, "review_session")))
        .await?
        .check()?;
    Ok(())
}

/// Facilitator: reveal now (close writing early).
pub async fn reveal_now(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_facilitator(&s, user_id)?;
    require_writing(&s)?;
    reveal_locked(db, &s.id).await
}

/// Facilitator: move the team from the revealed cards to deciding.
pub async fn start_deciding(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_facilitator(&s, user_id)?;
    if s.phase_at(&now_rfc3339()) != Phase::Revealed {
        return Err(DbError::Invalid(NOT_NOW.into()));
    }
    db.query("UPDATE type::record('review_session', $skey) SET phase = 'deciding', rev += 1 WHERE phase = 'revealed'")
        .bind(("skey", record_key(&s.id, "review_session")))
        .await?
        .check()?;
    Ok(())
}

/// The ids of `ids` that are cards (not passes) of the session; `NotFound`
/// if any is not.
async fn session_cards(db: &Surreal<Db>, session_id: &str, ids: &[String]) -> DbResult<Vec<Entry>> {
    let keys: Vec<String> = ids
        .iter()
        .map(|i| record_key(i.trim(), "review_entry"))
        .collect();
    let mut r = db
        .query("SELECT id, author, author.username AS author_name, prompt, text, passed, cluster, order_key FROM review_entry WHERE session = type::record('review_session', $skey) AND passed = false AND record::id(id) IN $keys")
        .bind(("skey", record_key(session_id, "review_session")))
        .bind(("keys", keys.clone()))
        .await?;
    let rows: Vec<DbEntry> = r.take(0)?;
    let entries: Vec<Entry> = rows.into_iter().filter_map(Entry::from_db).collect();
    let unique: HashSet<&String> = keys.iter().collect();
    if entries.len() != unique.len() {
        return Err(DbError::NotFound);
    }
    Ok(entries)
}

/// Toggle the caller's +1 on a revealed card (own cards too, the same for
/// everyone). Returns whether the caller now +1s it.
pub async fn plus_one(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
    entry_id: &str,
) -> DbResult<bool> {
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_open(&s)?;
    let entry = session_cards(db, &s.id, &[entry_id.to_string()])
        .await?
        .remove(0);
    toggle_vote(
        db,
        team_id,
        &s.id,
        user_id,
        "review_entry",
        &entry.id,
        "plus_one",
    )
    .await
}

/// Toggle a vote row (the caller holds the lock and checked the target).
async fn toggle_vote(
    db: &Surreal<Db>,
    team_id: &str,
    session_id: &str,
    user_id: &str,
    target_table: &'static str,
    target_id: &str,
    kind: &'static str,
) -> DbResult<bool> {
    let mut r = db
        .query(format!(
            "BEGIN TRANSACTION; \
             LET $target = type::record($ttb, $tkey); \
             LET $mine = (SELECT VALUE id FROM review_vote WHERE target = $target AND user = type::record('user', $ukey) AND kind = $kind); \
             IF array::len($mine) > 0 {{ DELETE review_vote WHERE id IN $mine; }} ELSE {{ CREATE review_vote SET session = type::record('review_session', $skey), team = type::record('team', $team_key), target = $target, user = type::record('user', $ukey), kind = $kind; }}; \
             {BUMP} \
             RETURN array::len($mine) = 0; \
             COMMIT TRANSACTION;"
        ))
        .bind(("ttb", target_table))
        .bind(("tkey", record_key(target_id, target_table)))
        .bind(("ukey", record_key(user_id, "user")))
        .bind(("kind", kind))
        .bind(("skey", record_key(session_id, "review_session")))
        .bind(("team_key", record_key(team_id, "team")))
        .await
        .map_err(DbError::from)
        .and_then(check_tx)
        .map_err(|e| {
            if e.to_string().contains("review_vote_unique") {
                DbError::Invalid("That vote is already counted.".into())
            } else {
                e
            }
        })?;
    let on: Option<bool> = r.take(5)?;
    Ok(on.unwrap_or(false))
}

/// Facilitator: merge `entry_ids` under `target_id` (cards of one prompt;
/// each keeps its words). A card that heads a cluster brings its cluster.
pub async fn merge_entries(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
    target_id: &str,
    entry_ids: &[String],
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_facilitator(&s, user_id)?;
    require_open(&s)?;
    let mut all: Vec<String> = vec![target_id.to_string()];
    all.extend(entry_ids.iter().cloned());
    let cards = session_cards(db, &s.id, &all).await?;
    let target_key = record_key(target_id.trim(), "review_entry");
    let target = cards
        .iter()
        .find(|c| record_key(&c.id, "review_entry") == target_key)
        .ok_or(DbError::NotFound)?;
    if cards.iter().any(|c| c.prompt != target.prompt) {
        return Err(DbError::Invalid(
            "Only cards of the same prompt can be merged.".into(),
        ));
    }
    // The root: the target's own cluster head, else the target.
    let root = target.cluster.clone().unwrap_or_else(|| target.id.clone());
    let root_key = record_key(&root, "review_entry");
    let members: Vec<String> = cards
        .iter()
        .map(|c| record_key(&c.id, "review_entry"))
        .filter(|k| *k != root_key)
        .collect();
    let r = db
        .query(format!(
            "BEGIN TRANSACTION; \
             LET $root = type::record('review_entry', $root_key); \
             LET $s = type::record('review_session', $skey); \
             LET $moved = (SELECT VALUE id FROM review_entry WHERE session = $s AND record::id(id) IN $members); \
             UPDATE review_entry SET cluster = $root WHERE session = $s AND (id IN $moved OR cluster IN $moved); \
             UPDATE $root SET cluster = NONE; \
             {BUMP} \
             COMMIT TRANSACTION;"
        ))
        .bind(("root_key", root_key))
        .bind(("skey", record_key(&s.id, "review_session")))
        .bind(("members", members))
        .await?;
    check_tx(r)?;
    Ok(())
}

/// Facilitator: take a card out of its cluster; a cluster head dissolves
/// its whole cluster.
pub async fn unmerge(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
    entry_id: &str,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_facilitator(&s, user_id)?;
    require_open(&s)?;
    let card = session_cards(db, &s.id, &[entry_id.to_string()])
        .await?
        .remove(0);
    let r = db
        .query(format!(
            "BEGIN TRANSACTION; \
             LET $e = type::record('review_entry', $ekey); \
             UPDATE review_entry SET cluster = NONE WHERE session = type::record('review_session', $skey) AND (id = $e OR cluster = $e); \
             {BUMP} \
             COMMIT TRANSACTION;"
        ))
        .bind(("ekey", record_key(&card.id, "review_entry")))
        .bind(("skey", record_key(&s.id, "review_session")))
        .await?;
    check_tx(r)?;
    Ok(())
}

/// Turn a revealed card into an action-item proposal (any member; the text
/// is the member's edit of the card). No proposer is stored.
pub async fn create_proposal(
    db: &Surreal<Db>,
    team_id: &str,
    session_id: &str,
    source_entry: Option<String>,
    text: &str,
) -> DbResult<String> {
    let text = clean_text(text).map_err(DbError::Invalid)?;
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_open(&s)?;
    let source = match source_entry.filter(|e| !e.trim().is_empty()) {
        Some(e) => Some(session_cards(db, &s.id, &[e]).await?.remove(0).id),
        None => None,
    };
    let skey = record_key(&s.id, "review_session");
    let mut r = db
        .query("SELECT count() AS n FROM review_proposal WHERE session = type::record('review_session', $skey) GROUP ALL")
        .bind(("skey", skey.clone()))
        .await?;
    #[derive(Debug, Deserialize, SurrealValue)]
    struct N {
        n: i64,
    }
    let n: Option<N> = r.take(0)?;
    if n.map(|n| n.n).unwrap_or(0) as usize >= PROPOSALS_MAX {
        return Err(DbError::Invalid(format!(
            "A review holds at most {PROPOSALS_MAX} proposals."
        )));
    }
    let mut r = db
        .query(format!(
            "BEGIN TRANSACTION; \
             CREATE review_proposal SET session = type::record('review_session', $skey), team = type::record('team', $team_key), text = $text, source = IF $source_key = NONE {{ NONE }} ELSE {{ type::record('review_entry', $source_key) }}, accepted = false RETURN id; \
             {BUMP} \
             COMMIT TRANSACTION;"
        ))
        .bind(("skey", skey))
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("text", text))
        .bind(("source_key", source.map(|s| record_key(&s, "review_entry"))))
        .await
        .map_err(DbError::from)
        .and_then(check_tx)?;
    let rows: Vec<IdRow> = r.take(1)?;
    rows.into_iter()
        .next()
        .map(|r| r.id.to_sql())
        .ok_or_else(|| DbError::Other("Failed to save the proposal".into()))
}

/// One proposal of the session (`NotFound` otherwise).
async fn session_proposal(
    db: &Surreal<Db>,
    session_id: &str,
    proposal_id: &str,
) -> DbResult<Proposal> {
    let mut r = db
        .query("SELECT id, text, source, accepted, owner FROM type::record('review_proposal', $pkey) WHERE session = type::record('review_session', $skey)")
        .bind(("pkey", record_key(proposal_id.trim(), "review_proposal")))
        .bind(("skey", record_key(session_id, "review_session")))
        .await?;
    let rows: Vec<DbProposal> = r.take(0)?;
    rows.into_iter()
        .next()
        .map(Proposal::from)
        .ok_or(DbError::NotFound)
}

/// Toggle the caller's decide vote on a proposal; at most
/// [`DECIDE_VOTES`] per member and session. Returns whether it is on now.
pub async fn decide_vote(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
    proposal_id: &str,
) -> DbResult<bool> {
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_open(&s)?;
    let p = session_proposal(db, &s.id, proposal_id).await?;
    let contents = load_contents(db, &s).await?;
    let mine: Vec<&Vote> = contents
        .votes
        .iter()
        .filter(|v| v.decide && v.user_id == user_id)
        .collect();
    let already = mine.iter().any(|v| v.target == p.id);
    if !already && mine.len() >= DECIDE_VOTES {
        return Err(DbError::Invalid(format!(
            "You have used your {DECIDE_VOTES} votes. Take one back to vote here."
        )));
    }
    toggle_vote(
        db,
        team_id,
        &s.id,
        user_id,
        "review_proposal",
        &p.id,
        "decide",
    )
    .await
}

/// Facilitator: accept a proposal (at most [`ACCEPT_MAX`]) or take it back.
pub async fn set_accepted(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
    proposal_id: &str,
    accepted: bool,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_facilitator(&s, user_id)?;
    require_open(&s)?;
    let p = session_proposal(db, &s.id, proposal_id).await?;
    if accepted && !p.accepted {
        let contents = load_contents(db, &s).await?;
        if contents.proposals.iter().filter(|p| p.accepted).count() >= ACCEPT_MAX {
            return Err(DbError::Invalid(format!(
                "At most {ACCEPT_MAX} items are accepted. Take one back first."
            )));
        }
    }
    let r = db
        .query(format!(
            "BEGIN TRANSACTION; \
             UPDATE type::record('review_proposal', $pkey) SET accepted = $accepted; \
             {BUMP} \
             COMMIT TRANSACTION;"
        ))
        .bind(("pkey", record_key(&p.id, "review_proposal")))
        .bind(("skey", record_key(&s.id, "review_session")))
        .bind(("accepted", accepted))
        .await?;
    check_tx(r)?;
    Ok(())
}

/// Facilitator: the owner chip of a proposal: a current member's username,
/// or blank for nobody (checked with `db::check_owner`).
pub async fn set_owner(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
    proposal_id: &str,
    owner: Option<String>,
) -> DbResult<()> {
    let owner = db::check_owner(db, team_id, owner).await?;
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_facilitator(&s, user_id)?;
    require_open(&s)?;
    let p = session_proposal(db, &s.id, proposal_id).await?;
    let r = db
        .query(format!(
            "BEGIN TRANSACTION; \
             UPDATE type::record('review_proposal', $pkey) SET owner = $owner; \
             {BUMP} \
             COMMIT TRANSACTION;"
        ))
        .bind(("pkey", record_key(&p.id, "review_proposal")))
        .bind(("skey", record_key(&s.id, "review_session")))
        .bind(("owner", owner))
        .await?;
    check_tx(r)?;
    Ok(())
}

/// Facilitator: delete a proposal (with its votes).
pub async fn delete_proposal(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
    proposal_id: &str,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_facilitator(&s, user_id)?;
    require_open(&s)?;
    let p = session_proposal(db, &s.id, proposal_id).await?;
    let r = db
        .query(format!(
            "BEGIN TRANSACTION; \
             LET $p = type::record('review_proposal', $pkey); \
             DELETE review_vote WHERE target = $p; \
             DELETE $p; \
             {BUMP} \
             COMMIT TRANSACTION;"
        ))
        .bind(("pkey", record_key(&p.id, "review_proposal")))
        .bind(("skey", record_key(&s.id, "review_session")))
        .await?;
    check_tx(r)?;
    Ok(())
}

/// One action item written at close.
#[derive(Debug, Clone, serde::Serialize, Deserialize, SurrealValue)]
struct CloseItem {
    text: String,
    owner: Option<String>,
}

/// Facilitator: close the session. One transaction creates the generated
/// review (`created_by` = the facilitator; M159's stamps: the match, its
/// series when it has one, the game for a series, the Riot match id when
/// known; `what_went_well` and `improvements` empty; `action_items` = the
/// accepted texts), an action item per accepted proposal with
/// `source_review` = the review and its owner (re-checked: an owner who left
/// is dropped), and closes the session with its voices. The items are
/// written here, not through `review_item_lines`, and because they name the
/// review as their source, saving the review again creates none. Returns
/// the review's id.
pub async fn close_session(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    session_id: &str,
) -> DbResult<String> {
    let names = db::team_member_usernames(db, team_id).await?;
    let _guard = invariant_lock().await;
    let s = load_settled_locked(db, team_id, session_id).await?;
    require_facilitator(&s, user_id)?;
    require_open(&s)?;
    let contents = load_contents(db, &s).await?;
    let votes = |id: &str| {
        contents
            .votes
            .iter()
            .filter(|v| v.decide && v.target == id)
            .count()
    };
    let mut accepted: Vec<&Proposal> = contents.proposals.iter().filter(|p| p.accepted).collect();
    if accepted.len() > ACCEPT_MAX {
        return Err(DbError::Invalid(format!(
            "At most {ACCEPT_MAX} items are accepted."
        )));
    }
    accepted.sort_by(|a, b| {
        votes(&b.id)
            .cmp(&votes(&a.id))
            .then_with(|| a.id.cmp(&b.id))
    });
    let items: Vec<CloseItem> = accepted
        .iter()
        .map(|p| CloseItem {
            text: p.text.clone(),
            owner: p
                .owner
                .as_ref()
                .filter(|o| names.iter().any(|n| n == *o))
                .cloned(),
        })
        .collect();
    let texts: Vec<String> = items.iter().map(|i| i.text.clone()).collect();
    let voices = voice_count(&contents) as i64;
    let members = contents.members.len() as i64;
    // A Bo1 has no game number on its review (M159); a deleted match none.
    let game_stamp = match (&s.match_id, &s.match_format) {
        (Some(_), Some(format)) if game_count(format) > 1 => Some(s.game_number),
        _ => None,
    };
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             LET $s = type::record('review_session', $skey); \
             LET $t = type::record('team', $team_key); \
             LET $r = (CREATE ONLY post_game_learning SET team = $t, match_riot_id = $riot_id, what_went_well = [], improvements = [], action_items = $texts, created_by = type::record('user', $fac_key), scheduled_match = IF $match_key = NONE { NONE } ELSE { type::record('scheduled_match', $match_key) }, series = IF $series_key = NONE { NONE } ELSE { type::record('series', $series_key) }, game_number = $game); \
             FOR $i IN $items { CREATE action_item SET team = $t, text = $i.text, source_review = $r.id, assigned_to = $i.owner; }; \
             UPDATE $s SET review = $r.id, phase = 'closed', closed_at = time::now(), voices = $voices, members = $members, rev += 1; \
             RETURN { id: $r.id }; \
             COMMIT TRANSACTION;",
        )
        .bind(("skey", record_key(&s.id, "review_session")))
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("riot_id", s.match_riot_id.clone()))
        .bind(("texts", texts))
        .bind(("fac_key", record_key(&s.facilitator_id, "user")))
        .bind((
            "match_key",
            s.match_id.as_ref().map(|m| record_key(m, "scheduled_match")),
        ))
        .bind((
            "series_key",
            s.series_id.as_ref().map(|m| record_key(m, "series")),
        ))
        .bind(("game", game_stamp.map(|g| g as i64)))
        .bind(("items", items))
        .bind(("voices", voices))
        .bind(("members", members))
        .await?;
    let mut response = check_tx(response)?;
    let row: Option<IdRow> = response.take(6)?;
    row.map(|r| r.id.to_sql())
        .ok_or_else(|| DbError::Other("Failed to close the review session".into()))
}

/// The team's sessions of one scheduled match, for the match thread's links. `NotFound` for another team's
/// match.
pub async fn sessions_of_match(
    db: &Surreal<Db>,
    team_id: &str,
    match_id: &str,
) -> DbResult<Vec<MatchSessionLink>> {
    let m = db::get_scheduled_match(db, team_id, match_id).await?;
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        id: RecordId,
        game_number: i64,
        phase: String,
        writing_closes_at: Datetime,
    }
    let mut r = db
        .query("SELECT id, game_number, phase, writing_closes_at FROM review_session WHERE team = type::record('team', $team_key) AND scheduled_match = type::record('scheduled_match', $match_key) ORDER BY game_number ASC")
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("match_key", record_key(&m.id, "scheduled_match")))
        .await?;
    let rows: Vec<Row> = r.take(0)?;
    let now = now_rfc3339();
    Ok(rows
        .into_iter()
        .map(|row| {
            let stored = Phase::from_key(&row.phase).unwrap_or(Phase::Closed);
            MatchSessionLink {
                game_number: row.game_number as i32,
                session_id: row.id.to_sql(),
                phase: effective_phase(stored, &dt_to_string(&row.writing_closes_at), &now),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests;
