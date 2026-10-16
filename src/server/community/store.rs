//! Queries for the community loop (M7). Tables are global (no team scope);
//! user-scoped rows (`survey_prompt`, `proposal_vote`, a reporter's
//! `bug_report` rows) are always filtered by `user = type::record('user', $u)`
//! so another user's rows read as "not found". Every value is bound.
//!
//! Admin functions take the admin's user id and store it with the time of
//! the write (`triaged_by`, `approved_by`, `updated_by`, `published_by`).

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};
use serde::Deserialize;
use surrealdb::{
    engine::local::Db,
    types::{Datetime, RecordId, SurrealValue, ToSql},
    Surreal,
};

use crate::models::bug_report::{
    BugReportAdmin, BugStatus, BugTriage, MyBugReport, PublicBugReport,
};
use crate::models::community::{
    clean_limited, AdminOverview, EntryLinks, ImportedProposal, LedgerMonth, ProposalInput,
    ProposalStatus, ProposalView, SiteText, Survey, SurveyAnswer, SurveyCounts, SurveyInput,
    SurveyKind, SurveyPromptView, SurveyResults, SurveyStatus, SurveyTrigger, SurveyVersionResult,
    SurveyWording, AGENT_KEY_PREFIX, MAX_ADMIN_NOTE, MAX_IMPORT_PROPOSALS, MAX_PROPOSAL_SUMMARY,
    MAX_PROPOSAL_TITLE, MAX_PUBLIC_NOTE, MAX_PUBLIC_TITLE, MAX_STATUS_NOTE, SITE_TEXTS,
};
use crate::server::community::{changelog, surveys, wording_hash};
use crate::server::db::{dt_to_string, invariant_lock, opt_dt, record_key, DbError, DbResult};

fn to_utc(dt: &Datetime) -> DateTime<Utc> {
    dt_to_string(dt)
        .parse::<DateTime<Utc>>()
        .unwrap_or_else(|_| Utc::now())
}

fn user_key(user_id: &str) -> String {
    record_key(user_id, "user")
}

#[derive(Debug, Deserialize, SurrealValue)]
struct IdRow {
    id: RecordId,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct CountRow {
    n: i64,
}

// ---------------------------------------------------------------------------
// Bug reports v2
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbMyReport {
    id: RecordId,
    created_at: Datetime,
    page_url: String,
    element_label: String,
    description: String,
    category: String,
    status: String,
    public_note: Option<String>,
    fixed_in: Option<String>,
    updated: bool,
    status_changed_at: Option<Datetime>,
}

/// The caller's own reports, newest first.
pub async fn list_my_bug_reports(db: &Surreal<Db>, user_id: &str) -> DbResult<Vec<MyBugReport>> {
    let mut r = db
        .query(
            "SELECT id, created_at, page_url, element_label, description, category, status, \
                 public_note, fixed_in, status_changed_at, \
                 (status_changed_at != NONE AND (reporter_seen_at = NONE OR status_changed_at > reporter_seen_at)) AS updated \
             FROM bug_report WHERE user = type::record('user', $u) ORDER BY created_at DESC",
        )
        .bind(("u", user_key(user_id)))
        .await?;
    let rows: Vec<DbMyReport> = r.take(0)?;
    Ok(rows
        .into_iter()
        .map(|b| MyBugReport {
            id: b.id.to_sql(),
            created_at: dt_to_string(&b.created_at),
            page_url: b.page_url,
            element_label: b.element_label,
            description: b.description,
            category: b.category,
            status: b.status,
            public_note: b.public_note,
            fixed_in_title: b.fixed_in.as_deref().and_then(changelog::entry_title),
            fixed_in: b.fixed_in,
            updated: b.updated,
            status_changed_at: opt_dt(b.status_changed_at),
        })
        .collect())
}

/// Mark every report of the caller as seen ("Your reports" was opened).
pub async fn mark_my_reports_seen(db: &Surreal<Db>, user_id: &str) -> DbResult<()> {
    db.query("UPDATE bug_report SET reporter_seen_at = time::now() WHERE user = type::record('user', $u)")
        .bind(("u", user_key(user_id)))
        .await?
        .check()?;
    Ok(())
}

/// How many of the caller's reports changed since they last looked.
pub async fn count_my_report_updates(db: &Surreal<Db>, user_id: &str) -> DbResult<usize> {
    let mut r = db
        .query(
            "SELECT count() AS n FROM bug_report WHERE user = type::record('user', $u) \
               AND status_changed_at != NONE \
               AND (reporter_seen_at = NONE OR status_changed_at > reporter_seen_at) GROUP ALL",
        )
        .bind(("u", user_key(user_id)))
        .await?;
    let row: Option<CountRow> = r.take(0)?;
    Ok(row.map(|c| c.n as usize).unwrap_or(0))
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbBoardRow {
    id: RecordId,
    public_title: String,
    public_note: Option<String>,
    category: String,
    status: String,
    status_changed_at: Option<Datetime>,
    fixed_in: Option<String>,
}

/// The bug board: reports an admin put on it, with only admin-written text.
pub async fn list_bug_board(db: &Surreal<Db>) -> DbResult<Vec<PublicBugReport>> {
    let mut r = db
        .query(
            "SELECT id, public_title, public_note, category, status, status_changed_at, fixed_in, created_at \
             FROM bug_report WHERE on_board = true AND public_title != NONE AND status != 'new' \
             ORDER BY status_changed_at DESC, created_at DESC",
        )
        .await?;
    let rows: Vec<DbBoardRow> = r.take(0)?;
    Ok(rows
        .into_iter()
        .map(|b| PublicBugReport {
            id: b.id.to_sql(),
            public_title: b.public_title,
            public_note: b.public_note,
            category: b.category,
            status: b.status,
            status_changed_at: opt_dt(b.status_changed_at),
            fixed_in_title: b.fixed_in.as_deref().and_then(changelog::entry_title),
            fixed_in: b.fixed_in,
        })
        .collect())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbAdminReport {
    id: RecordId,
    reporter: Option<String>,
    created_at: Datetime,
    page_url: String,
    element_label: String,
    description: String,
    category: String,
    viewport_w: Option<i64>,
    viewport_h: Option<i64>,
    status: String,
    on_board: Option<bool>,
    public_title: Option<String>,
    public_note: Option<String>,
    admin_note: Option<String>,
    fixed_in: Option<String>,
    status_changed_at: Option<Datetime>,
    triaged_by_name: Option<String>,
    updated_at: Option<Datetime>,
    proposal: Option<RecordId>,
}

/// Every report (or those with `status`), newest first, for triage.
pub async fn admin_list_bug_reports(
    db: &Surreal<Db>,
    status: Option<BugStatus>,
) -> DbResult<Vec<BugReportAdmin>> {
    let mut r = db
        .query(
            "SELECT id, user.username AS reporter, created_at, page_url, element_label, description, \
                 category, viewport_w, viewport_h, status, on_board, public_title, public_note, \
                 admin_note, fixed_in, status_changed_at, triaged_by.username AS triaged_by_name, \
                 updated_at, proposal \
             FROM bug_report WHERE $status = NONE OR status = $status ORDER BY created_at DESC",
        )
        .bind(("status", status.map(|s| s.as_str().to_string())))
        .await?;
    let rows: Vec<DbAdminReport> = r.take(0)?;
    Ok(rows
        .into_iter()
        .map(|b| BugReportAdmin {
            id: b.id.to_sql(),
            reporter: b.reporter.unwrap_or_else(|| "(deleted account)".into()),
            created_at: dt_to_string(&b.created_at),
            page_url: b.page_url,
            element_label: b.element_label,
            description: b.description,
            category: b.category,
            viewport_w: b.viewport_w.map(|v| v as i32),
            viewport_h: b.viewport_h.map(|v| v as i32),
            status: b.status,
            on_board: b.on_board.unwrap_or(false),
            public_title: b.public_title,
            public_note: b.public_note,
            admin_note: b.admin_note,
            fixed_in: b.fixed_in,
            status_changed_at: opt_dt(b.status_changed_at),
            triaged_by: b.triaged_by_name,
            updated_at: opt_dt(b.updated_at),
            proposal_id: b.proposal.map(|p| p.to_sql()),
        })
        .collect())
}

/// A triage decision checked against the rules; the changelog lookup is
/// passed in so tests can supply their own.
pub fn validate_triage(
    t: &BugTriage,
    entry_exists: impl Fn(&str) -> bool,
) -> Result<BugTriage, DbError> {
    let status =
        BugStatus::parse(&t.status).ok_or_else(|| DbError::Invalid("Choose a status.".into()))?;
    let public_title =
        clean_limited(&t.public_title, MAX_PUBLIC_TITLE).map_err(DbError::Invalid)?;
    let public_note = clean_limited(&t.public_note, MAX_PUBLIC_NOTE).map_err(DbError::Invalid)?;
    let admin_note = clean_limited(&t.admin_note, MAX_ADMIN_NOTE).map_err(DbError::Invalid)?;
    if t.on_board && public_title.is_empty() {
        return Err(DbError::Invalid(
            "Write a title before showing a report on the board.".into(),
        ));
    }
    if t.on_board && status == BugStatus::New {
        return Err(DbError::Invalid(
            "Set a status other than New before showing a report on the board.".into(),
        ));
    }
    let fixed_in = if status == BugStatus::Fixed {
        let slug = t.fixed_in.trim().to_string();
        if slug.is_empty() {
            return Err(DbError::Invalid(
                "Choose the changelog entry that announced the fix.".into(),
            ));
        }
        if !entry_exists(&slug) {
            return Err(DbError::Invalid(
                "That changelog entry is not in this version.".into(),
            ));
        }
        slug
    } else {
        String::new()
    };
    Ok(BugTriage {
        id: t.id.clone(),
        status: status.as_str().to_string(),
        on_board: t.on_board,
        public_title,
        public_note,
        admin_note,
        fixed_in,
    })
}

/// Save a (validated) triage decision. `status_changed_at` moves when the
/// status or the note to the reporter changes (both are what the reporter
/// is told about).
pub async fn admin_update_bug_report(
    db: &Surreal<Db>,
    admin_id: &str,
    t: &BugTriage,
) -> DbResult<()> {
    let key = record_key(&t.id, "bug_report");
    let opt = |s: &str| Some(s.to_string()).filter(|v| !v.is_empty());
    let mut r = db
        .query(
            "UPDATE type::record('bug_report', $key) SET \
                 status_changed_at = IF status != $status OR (public_note ?? '') != ($public_note ?? '') \
                     { time::now() } ELSE { status_changed_at }, \
                 status = $status, on_board = $on_board, public_title = $public_title, \
                 public_note = $public_note, admin_note = $admin_note, fixed_in = $fixed_in, \
                 triaged_by = type::record('user', $admin), updated_at = time::now() \
             RETURN id",
        )
        .bind(("key", key))
        .bind(("status", t.status.clone()))
        .bind(("on_board", t.on_board))
        .bind(("public_title", opt(&t.public_title)))
        .bind(("public_note", opt(&t.public_note)))
        .bind(("admin_note", opt(&t.admin_note)))
        .bind(("fixed_in", opt(&t.fixed_in)))
        .bind(("admin", user_key(admin_id)))
        .await?;
    let rows: Vec<IdRow> = r.take(0)?;
    if rows.is_empty() {
        return Err(DbError::NotFound);
    }
    Ok(())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct WishRow {
    category: String,
    description: String,
    public_title: Option<String>,
    proposal: Option<RecordId>,
}

/// Turn a wishlist report into a draft proposal (unpublished), linked both
/// ways. Returns the proposal id; an already linked report returns its
/// proposal.
pub async fn admin_bug_to_proposal(
    db: &Surreal<Db>,
    admin_id: &str,
    report_id: &str,
) -> DbResult<String> {
    let _guard = invariant_lock().await;
    let key = record_key(report_id, "bug_report");
    let mut r = db
        .query("SELECT category, description, public_title, proposal FROM type::record('bug_report', $key)")
        .bind(("key", key.clone()))
        .await?;
    let row: Option<WishRow> = r.take(0)?;
    let row = row.ok_or(DbError::NotFound)?;
    if let Some(p) = row.proposal {
        return Ok(p.to_sql());
    }
    if row.category != "wishlist" {
        return Err(DbError::Invalid(
            "Only wishlist reports can become proposals.".into(),
        ));
    }
    let title: String = row
        .public_title
        .filter(|t| !t.trim().is_empty())
        .unwrap_or(row.description)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_PROPOSAL_TITLE)
        .collect();
    let title = if title.is_empty() {
        "Wishlist idea".to_string()
    } else {
        title
    };
    let mut r = db
        .query(
            "BEGIN TRANSACTION; \
             LET $p = (CREATE ONLY proposal SET title = $title, summary = '', status = 'draft', \
                 source = 'wishlist', published = false, created_by = type::record('user', $admin), \
                 created_at = time::now(), status_changed_at = time::now()); \
             UPDATE type::record('bug_report', $key) SET proposal = $p.id, \
                 triaged_by = type::record('user', $admin), updated_at = time::now(); \
             RETURN { id: $p.id }; \
             COMMIT TRANSACTION;",
        )
        .bind(("title", title))
        .bind(("admin", user_key(admin_id)))
        .bind(("key", key))
        .await
        .map(crate::server::db::check_tx)??;
    let row: Option<IdRow> = r.take(3)?;
    row.map(|r| r.id.to_sql())
        .ok_or_else(|| DbError::Other("proposal not created".into()))
}

// ---------------------------------------------------------------------------
// What-next board
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbProposal {
    id: RecordId,
    title: String,
    summary: String,
    status: String,
    status_note: Option<String>,
    shipped_in: Option<String>,
    created_at: Datetime,
    status_changed_at: Option<Datetime>,
    published: bool,
    source: String,
    seed_key: Option<String>,
    votes: i64,
    mine: i64,
    report: Option<RecordId>,
}

const PROPOSAL_FIELDS: &str = "id, title, summary, status, status_note, shipped_in, created_at, \
     status_changed_at, published, source, seed_key, \
     (count(SELECT id FROM proposal_vote WHERE proposal = $parent.id)) AS votes, \
     (count(SELECT id FROM proposal_vote WHERE proposal = $parent.id AND user = type::record('user', $u))) AS mine, \
     (SELECT VALUE id FROM bug_report WHERE proposal = $parent.id LIMIT 1)[0] AS report";

fn proposal_view(p: DbProposal, admin: bool) -> ProposalView {
    let agent_proposed_on = p
        .seed_key
        .as_deref()
        .is_some_and(|k| k.starts_with(AGENT_KEY_PREFIX))
        .then(|| dt_to_string(&p.created_at).chars().take(10).collect());
    ProposalView {
        agent_proposed_on,
        id: p.id.to_sql(),
        title: p.title,
        summary: p.summary,
        status: p.status,
        status_note: p.status_note,
        shipped_in_title: p.shipped_in.as_deref().and_then(changelog::entry_title),
        shipped_in: p.shipped_in,
        votes: p.votes.max(0) as u32,
        voted_by_me: p.mine > 0,
        created_at: dt_to_string(&p.created_at),
        status_changed_at: opt_dt(p.status_changed_at),
        published: admin && p.published,
        source: if admin { p.source } else { String::new() },
        wishlist_report: if admin {
            p.report.map(|r| r.to_sql())
        } else {
            None
        },
    }
}

/// Published proposals (never drafts) with votes, sorted for the board.
pub async fn list_proposals(db: &Surreal<Db>, user_id: &str) -> DbResult<Vec<ProposalView>> {
    let query = format!(
        "SELECT {PROPOSAL_FIELDS} FROM proposal WHERE published = true AND status != 'draft'"
    );
    let mut r = db.query(query).bind(("u", user_key(user_id))).await?;
    let rows: Vec<DbProposal> = r.take(0)?;
    let mut list: Vec<ProposalView> = rows.into_iter().map(|p| proposal_view(p, false)).collect();
    crate::models::community::sort_proposals(&mut list);
    Ok(list)
}

#[derive(Debug, serde::Serialize, SurrealValue)]
struct ImportRow {
    seed_key: String,
    title: String,
    summary: String,
}

/// Import agent proposals (M141 H4): every one a draft, unpublished, keyed
/// `agent:<key>` and created by the importing admin. All or nothing: a key
/// that was imported before refuses the whole import. Input comes from
/// [`crate::models::community::parse_proposal_import`].
pub async fn admin_import_proposals(
    db: &Surreal<Db>,
    admin_id: &str,
    items: &[ImportedProposal],
) -> DbResult<usize> {
    if items.is_empty() || items.len() > MAX_IMPORT_PROPOSALS {
        return Err(DbError::Invalid("Nothing to import.".into()));
    }
    let rows: Vec<ImportRow> = items
        .iter()
        .map(|p| ImportRow {
            seed_key: format!("{AGENT_KEY_PREFIX}{}", p.key),
            title: p.title.clone(),
            summary: p.summary.clone(),
        })
        .collect();
    let keys: Vec<String> = rows.iter().map(|r| r.seed_key.clone()).collect();
    let _guard = invariant_lock().await;
    let mut r = db
        .query("SELECT VALUE seed_key FROM proposal WHERE seed_key IN $keys")
        .bind(("keys", keys))
        .await?;
    let existing: Vec<Option<String>> = r.take(0)?;
    let existing: Vec<String> = existing
        .into_iter()
        .flatten()
        .map(|k| k.trim_start_matches(AGENT_KEY_PREFIX).to_string())
        .collect();
    if !existing.is_empty() {
        return Err(DbError::Invalid(format!(
            "Already imported, nothing was saved: {}.",
            existing.join(", ")
        )));
    }
    let n = rows.len();
    let r = db
        .query(
            "BEGIN TRANSACTION; \
             FOR $p IN $rows { \
                 CREATE proposal SET seed_key = $p.seed_key, title = $p.title, summary = $p.summary, \
                     status = 'draft', source = 'mission', published = false, \
                     created_by = type::record('user', $admin), created_at = time::now(), \
                     status_changed_at = time::now(); \
             }; \
             COMMIT TRANSACTION;",
        )
        .bind(("rows", rows))
        .bind(("admin", user_key(admin_id)))
        .await?;
    crate::server::db::check_tx(r)?;
    Ok(n)
}

/// Every proposal, for admins (votes counted, own vote irrelevant).
pub async fn admin_list_proposals(db: &Surreal<Db>) -> DbResult<Vec<ProposalView>> {
    let query = format!("SELECT {PROPOSAL_FIELDS} FROM proposal");
    // No caller: a key no user has, so `mine` counts nothing.
    let mut r = db.query(query).bind(("u", "-".to_string())).await?;
    let rows: Vec<DbProposal> = r.take(0)?;
    let mut list: Vec<ProposalView> = rows.into_iter().map(|p| proposal_view(p, true)).collect();
    list.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(a.id.cmp(&b.id)));
    Ok(list)
}

#[derive(Debug, Deserialize, SurrealValue)]
struct VoteTarget {
    status: String,
    published: bool,
}

/// Vote (`true`) or withdraw the vote (`false`). Idempotent; one vote per
/// user per proposal. Returns the new count.
pub async fn set_vote(
    db: &Surreal<Db>,
    user_id: &str,
    proposal_id: &str,
    vote: bool,
) -> DbResult<u32> {
    let _guard = invariant_lock().await;
    let key = record_key(proposal_id, "proposal");
    let mut r = db
        .query("SELECT status, published FROM type::record('proposal', $key)")
        .bind(("key", key.clone()))
        .await?;
    let target: Option<VoteTarget> = r.take(0)?;
    let target = target.ok_or(DbError::NotFound)?;
    let status = ProposalStatus::parse(&target.status);
    if !target.published || status == Some(ProposalStatus::Draft) {
        return Err(DbError::NotFound);
    }
    if !status.is_some_and(|s| s.is_open()) {
        return Err(DbError::Invalid("Voting is closed for this idea.".into()));
    }
    let u = user_key(user_id);
    if vote {
        let mut r = db
            .query(
                "SELECT id FROM proposal_vote WHERE proposal = type::record('proposal', $key) \
                   AND user = type::record('user', $u)",
            )
            .bind(("key", key.clone()))
            .bind(("u", u.clone()))
            .await?;
        let existing: Vec<IdRow> = r.take(0)?;
        if existing.is_empty() {
            let res = db
                .query(
                    "CREATE proposal_vote SET proposal = type::record('proposal', $key), \
                         user = type::record('user', $u), created_at = time::now()",
                )
                .bind(("key", key.clone()))
                .bind(("u", u.clone()))
                .await?
                .check();
            if let Err(e) = res {
                // A duplicate is absorbed: the vote exists either way.
                if !e.to_string().contains("proposal_vote_unique") {
                    return Err(e.into());
                }
            }
        }
    } else {
        db.query(
            "DELETE proposal_vote WHERE proposal = type::record('proposal', $key) \
               AND user = type::record('user', $u)",
        )
        .bind(("key", key.clone()))
        .bind(("u", u))
        .await?
        .check()?;
    }
    let mut r = db
        .query("SELECT count() AS n FROM proposal_vote WHERE proposal = type::record('proposal', $key) GROUP ALL")
        .bind(("key", key))
        .await?;
    let row: Option<CountRow> = r.take(0)?;
    Ok(row.map(|c| c.n.max(0) as u32).unwrap_or(0))
}

/// Check an admin's proposal form; returns it cleaned.
pub fn validate_proposal(
    input: &ProposalInput,
    entry_exists: impl Fn(&str) -> bool,
) -> Result<ProposalInput, DbError> {
    let title = clean_limited(&input.title, MAX_PROPOSAL_TITLE).map_err(DbError::Invalid)?;
    if title.is_empty() {
        return Err(DbError::Invalid("Write a title.".into()));
    }
    let summary = clean_limited(&input.summary, MAX_PROPOSAL_SUMMARY).map_err(DbError::Invalid)?;
    let status = ProposalStatus::parse(&input.status)
        .ok_or_else(|| DbError::Invalid("Choose a status.".into()))?;
    let status_note =
        clean_limited(&input.status_note, MAX_STATUS_NOTE).map_err(DbError::Invalid)?;
    if status == ProposalStatus::Declined && status_note.is_empty() {
        return Err(DbError::Invalid("Say why it was declined.".into()));
    }
    let shipped_in = if status == ProposalStatus::Shipped {
        let s = input.shipped_in.trim().to_string();
        if s.is_empty() {
            return Err(DbError::Invalid(
                "Choose the changelog entry that shipped it.".into(),
            ));
        }
        if !entry_exists(&s) {
            return Err(DbError::Invalid(
                "That changelog entry is not in this version.".into(),
            ));
        }
        s
    } else {
        String::new()
    };
    if input.published && status == ProposalStatus::Draft {
        return Err(DbError::Invalid(
            "Choose a status other than Draft before publishing.".into(),
        ));
    }
    Ok(ProposalInput {
        id: input.id.clone().filter(|s| !s.trim().is_empty()),
        title,
        summary,
        status: status.as_str().to_string(),
        status_note,
        shipped_in,
        published: input.published,
    })
}

/// Create or update a (validated) proposal. Votes are kept on every change.
pub async fn admin_save_proposal(
    db: &Surreal<Db>,
    admin_id: &str,
    p: &ProposalInput,
) -> DbResult<String> {
    let opt = |s: &str| Some(s.to_string()).filter(|v| !v.is_empty());
    let admin = user_key(admin_id);
    match &p.id {
        None => {
            let mut r = db
                .query(
                    "CREATE ONLY proposal SET title = $title, summary = $summary, status = $status, \
                         status_note = $note, shipped_in = $shipped, source = 'admin', \
                         published = $published, \
                         published_by = IF $published { type::record('user', $admin) } ELSE { NONE }, \
                         published_at = IF $published { time::now() } ELSE { NONE }, \
                         created_by = type::record('user', $admin), created_at = time::now(), \
                         status_changed_at = time::now() RETURN id",
                )
                .bind(("title", p.title.clone()))
                .bind(("summary", p.summary.clone()))
                .bind(("status", p.status.clone()))
                .bind(("note", opt(&p.status_note)))
                .bind(("shipped", opt(&p.shipped_in)))
                .bind(("published", p.published))
                .bind(("admin", admin))
                .await?;
            let row: Option<IdRow> = r.take(0)?;
            row.map(|r| r.id.to_sql())
                .ok_or_else(|| DbError::Other("proposal not created".into()))
        }
        Some(id) => {
            let key = record_key(id, "proposal");
            let mut r = db
                .query(
                    "UPDATE type::record('proposal', $key) SET \
                         status_changed_at = IF status != $status { time::now() } ELSE { status_changed_at }, \
                         published_by = IF $published AND !published { type::record('user', $admin) } ELSE { published_by }, \
                         published_at = IF $published AND !published { time::now() } ELSE { published_at }, \
                         title = $title, summary = $summary, status = $status, status_note = $note, \
                         shipped_in = $shipped, published = $published, \
                         updated_by = type::record('user', $admin), updated_at = time::now() \
                     RETURN id",
                )
                .bind(("key", key))
                .bind(("title", p.title.clone()))
                .bind(("summary", p.summary.clone()))
                .bind(("status", p.status.clone()))
                .bind(("note", opt(&p.status_note)))
                .bind(("shipped", opt(&p.shipped_in)))
                .bind(("published", p.published))
                .bind(("admin", admin))
                .await?;
            let rows: Vec<IdRow> = r.take(0)?;
            rows.first().map(|r| r.id.to_sql()).ok_or(DbError::NotFound)
        }
    }
}

/// Delete a proposal without votes (one with votes is declined instead).
pub async fn admin_delete_proposal(db: &Surreal<Db>, id: &str) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let key = record_key(id, "proposal");
    let mut r = db
        .query(
            "SELECT id FROM type::record('proposal', $key); \
             SELECT count() AS n FROM proposal_vote WHERE proposal = type::record('proposal', $key) GROUP ALL;",
        )
        .bind(("key", key.clone()))
        .await?;
    let found: Vec<IdRow> = r.take(0)?;
    if found.is_empty() {
        return Err(DbError::NotFound);
    }
    let votes: Option<CountRow> = r.take(1)?;
    if votes.is_some_and(|v| v.n > 0) {
        return Err(DbError::Invalid(
            "Ideas with votes cannot be deleted. Decline it instead, with a reason.".into(),
        ));
    }
    db.query(
        "UPDATE bug_report SET proposal = NONE WHERE proposal = type::record('proposal', $key); \
         DELETE type::record('proposal', $key);",
    )
    .bind(("key", key))
    .await?
    .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Surveys
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct OptOutRow {
    survey_opt_out: Option<bool>,
}

pub async fn survey_opt_out(db: &Surreal<Db>, user_id: &str) -> DbResult<bool> {
    let mut r = db
        .query("SELECT survey_opt_out FROM type::record('user', $u)")
        .bind(("u", user_key(user_id)))
        .await?;
    let row: Option<OptOutRow> = r.take(0)?;
    Ok(row.and_then(|r| r.survey_opt_out).unwrap_or(false))
}

/// Turn survey questions off (`true`) or on; off also expires every
/// pending prompt of the user.
pub async fn set_survey_opt_out(db: &Surreal<Db>, user_id: &str, off: bool) -> DbResult<()> {
    db.query(
        "UPDATE type::record('user', $u) SET survey_opt_out = $off; \
         IF $off { UPDATE survey_prompt SET state = 'expired', closed_at = time::now() \
             WHERE user = type::record('user', $u) AND state = 'pending' };",
    )
    .bind(("u", user_key(user_id)))
    .bind(("off", off))
    .await?
    .check()?;
    Ok(())
}

/// Note that the user did something a live survey asks about. Best effort
/// for callers (they log and continue); once per user, survey and occurrence.
pub async fn note_trigger(
    db: &Surreal<Db>,
    user_id: &str,
    trigger: SurveyTrigger,
    occurrence: &str,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    note_trigger_locked(db, user_id, trigger, occurrence).await
}

async fn note_trigger_locked(
    db: &Surreal<Db>,
    user_id: &str,
    trigger: SurveyTrigger,
    occurrence: &str,
) -> DbResult<()> {
    if survey_opt_out(db, user_id).await? {
        return Ok(());
    }
    let u = user_key(user_id);
    let mut r = db
        .query(
            "SELECT id FROM survey WHERE trigger = $trigger AND status = 'live' \
               AND approved_hash = wording_hash",
        )
        .bind(("trigger", trigger.as_str().to_string()))
        .await?;
    let live: Vec<IdRow> = r.take(0)?;
    for s in live {
        let skey = record_key(&s.id.to_sql(), "survey");
        let mut r = db
            .query(
                "SELECT id FROM survey_prompt WHERE user = type::record('user', $u) \
                   AND survey = type::record('survey', $s) AND occurrence = $occ",
            )
            .bind(("u", u.clone()))
            .bind(("s", skey.clone()))
            .bind(("occ", occurrence.to_string()))
            .await?;
        let existing: Vec<IdRow> = r.take(0)?;
        if !existing.is_empty() {
            continue;
        }
        let res = db
            .query(
                "CREATE survey_prompt SET user = type::record('user', $u), \
                     survey = type::record('survey', $s), occurrence = $occ, state = 'pending', \
                     created_at = time::now()",
            )
            .bind(("u", u.clone()))
            .bind(("s", skey))
            .bind(("occ", occurrence.to_string()))
            .await?
            .check();
        if let Err(e) = res {
            if !e.to_string().contains("survey_prompt_unique") {
                return Err(e.into());
            }
        }
    }
    Ok(())
}

/// After a draft is created: note the "3rd draft" trigger once the user
/// has created three or more drafts. Best effort: errors are logged.
pub async fn note_draft_saved(db: &Surreal<Db>, user_id: &str) {
    let result = async {
        let mut r = db
            .query("SELECT count() AS n FROM draft WHERE created_by = type::record('user', $u) GROUP ALL")
            .bind(("u", user_key(user_id)))
            .await?;
        let n: Option<CountRow> = r.take(0)?;
        if n.is_some_and(|n| n.n >= 3) {
            note_trigger(db, user_id, SurveyTrigger::DraftSaved3, "").await?;
        }
        Ok::<(), DbError>(())
    }
    .await;
    if let Err(e) = result {
        tracing::warn!(user = %user_id, error = %e, "survey trigger (draft) failed");
    }
}

/// After a review is created. Best effort: errors are logged.
pub async fn note_review_saved(db: &Surreal<Db>, user_id: &str) {
    if let Err(e) = note_trigger(db, user_id, SurveyTrigger::ReviewSaved, "").await {
        tracing::warn!(user = %user_id, error = %e, "survey trigger (review) failed");
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbPromptRow {
    id: RecordId,
    state: String,
    created_at: Datetime,
    shown_at: Option<Datetime>,
    trigger: Option<String>,
    deliverable: Option<bool>,
}

async fn prompt_rows(db: &Surreal<Db>, user_id: &str) -> DbResult<Vec<surveys::PromptRow>> {
    let mut r = db
        .query(
            "SELECT id, state, created_at, shown_at, survey.trigger AS trigger, \
                 (survey.status = 'live' AND survey.approved_hash = survey.wording_hash) AS deliverable \
             FROM survey_prompt WHERE user = type::record('user', $u)",
        )
        .bind(("u", user_key(user_id)))
        .await?;
    let rows: Vec<DbPromptRow> = r.take(0)?;
    Ok(rows
        .into_iter()
        .filter_map(|p| {
            Some(surveys::PromptRow {
                id: p.id.to_sql(),
                trigger: SurveyTrigger::parse(p.trigger.as_deref()?)?,
                state: p.state,
                created_at: to_utc(&p.created_at),
                shown_at: p.shown_at.as_ref().map(to_utc),
                deliverable: p.deliverable.unwrap_or(false),
            })
        })
        .collect())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbPromptView {
    id: RecordId,
    occurrence: String,
    question: String,
    kind: String,
    choices: Vec<String>,
    low: Option<String>,
    high: Option<String>,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct PatchRow {
    new_version: String,
    detected_at: Datetime,
}

/// The survey to show on `path` now, if any (spec 3.3). Marks a newly
/// chosen prompt as shown. Callers run this only from the browser.
pub async fn get_pending_survey(
    db: &Surreal<Db>,
    user_id: &str,
    path: &str,
) -> DbResult<Option<SurveyPromptView>> {
    let _guard = invariant_lock().await;
    if survey_opt_out(db, user_id).await? {
        return Ok(None);
    }
    let now = Utc::now();
    let u = user_key(user_id);
    db.query(
        "UPDATE survey_prompt SET state = 'expired', closed_at = time::now() \
         WHERE user = type::record('user', $u) AND state = 'pending' AND created_at < time::now() - 14d",
    )
    .bind(("u", u.clone()))
    .await?
    .check()?;

    let mut rows = prompt_rows(db, user_id).await?;
    let blocked = rows
        .iter()
        .any(|r| r.shown_at.is_some_and(|t| now - t < surveys::WEEKLY_CAP));
    if !blocked {
        // Lazy triggers: a recent patch, and the what-next board itself.
        let mut r = db
            .query("SELECT new_version, detected_at FROM patch_change ORDER BY detected_at DESC LIMIT 1")
            .await?;
        let patch: Option<PatchRow> = r.take(0)?;
        if let Some(occ) = surveys::patch_occurrence(
            patch
                .as_ref()
                .map(|p| (p.new_version.as_str(), to_utc(&p.detected_at))),
            now,
        ) {
            note_trigger_locked(db, user_id, SurveyTrigger::PatchChanged, &occ).await?;
        }
        if path == surveys::WHAT_NEXT_PATH {
            note_trigger_locked(db, user_id, SurveyTrigger::WhatNextVisit, "").await?;
        }
        rows = prompt_rows(db, user_id).await?;
    }

    let id = match surveys::select(&rows, now, path) {
        surveys::Delivery::Nothing => return Ok(None),
        surveys::Delivery::Again(id) => {
            // Shown again with the survey's current wording (it may have
            // been edited and re-approved since): record that wording.
            db.query(
                "UPDATE type::record('survey_prompt', $k) SET answer_hash = survey.wording_hash \
                 WHERE state = 'shown'",
            )
            .bind(("k", record_key(&id, "survey_prompt")))
            .await?
            .check()?;
            id
        }
        surveys::Delivery::Show(id) => {
            db.query(
                "UPDATE type::record('survey_prompt', $k) SET state = 'shown', shown_at = time::now(), \
                     answer_hash = survey.wording_hash WHERE state = 'pending'",
            )
            .bind(("k", record_key(&id, "survey_prompt")))
            .await?
            .check()?;
            id
        }
    };
    let mut r = db
        .query(
            "SELECT id, occurrence, survey.question AS question, survey.kind AS kind, \
                 survey.choices AS choices, survey.scale_low_label AS low, survey.scale_high_label AS high \
             FROM type::record('survey_prompt', $k) WHERE user = type::record('user', $u)",
        )
        .bind(("k", record_key(&id, "survey_prompt")))
        .bind(("u", u))
        .await?;
    let row: Option<DbPromptView> = r.take(0)?;
    Ok(row.map(|p| SurveyPromptView {
        prompt_id: p.id.to_sql(),
        question: surveys::render_question(&p.question, &p.occurrence),
        kind: p.kind,
        choices: p.choices,
        scale_low_label: p.low,
        scale_high_label: p.high,
    }))
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbOwnPrompt {
    state: String,
    answer_hash: Option<String>,
    wording_hash: Option<String>,
    survey_status: Option<String>,
    kind: Option<String>,
    choices: Option<Vec<String>>,
}

async fn own_prompt(db: &Surreal<Db>, user_id: &str, prompt_id: &str) -> DbResult<DbOwnPrompt> {
    let mut r = db
        .query(
            "SELECT state, answer_hash, survey.wording_hash AS wording_hash, survey.status AS survey_status, \
                 survey.kind AS kind, survey.choices AS choices \
             FROM type::record('survey_prompt', $k) WHERE user = type::record('user', $u)",
        )
        .bind(("k", record_key(prompt_id, "survey_prompt")))
        .bind(("u", user_key(user_id)))
        .await?;
    let row: Option<DbOwnPrompt> = r.take(0)?;
    row.ok_or(DbError::NotFound)
}

/// Answer one's own shown prompt. Final.
pub async fn answer_survey(
    db: &Surreal<Db>,
    user_id: &str,
    prompt_id: &str,
    answer: &SurveyAnswer,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let p = own_prompt(db, user_id, prompt_id).await?;
    if p.state != "shown" {
        return Err(DbError::Invalid(
            "You already answered this question.".into(),
        ));
    }
    // Closed, paused, or reworded since this person was shown it (their
    // answer would be counted against wording they never saw).
    if p.survey_status.as_deref() != Some("live") || p.answer_hash != p.wording_hash {
        return Err(DbError::Invalid(
            "This question has closed. Thanks anyway.".into(),
        ));
    }
    let kind = p
        .kind
        .as_deref()
        .and_then(SurveyKind::parse)
        .ok_or(DbError::NotFound)?;
    let answer = answer
        .validate(kind, p.choices.map(|c| c.len()).unwrap_or(0))
        .map_err(DbError::Invalid)?;
    let (choice, scale, text) = match answer {
        SurveyAnswer::Choice(i) => (Some(i as i64), None, None),
        SurveyAnswer::Scale(v) => (None, Some(v as i64), None),
        SurveyAnswer::Text(t) => (None, None, Some(t)),
    };
    db.query(
        "UPDATE type::record('survey_prompt', $k) SET state = 'answered', closed_at = time::now(), \
             answer_choice = $choice, answer_scale = $scale, answer_text = $text \
         WHERE user = type::record('user', $u) AND state = 'shown'",
    )
    .bind(("k", record_key(prompt_id, "survey_prompt")))
    .bind(("u", user_key(user_id)))
    .bind(("choice", choice))
    .bind(("scale", scale))
    .bind(("text", text))
    .await?
    .check()?;
    Ok(())
}

/// Dismiss one's own prompt; it never comes back (the weekly cap counts it).
pub async fn dismiss_survey(db: &Surreal<Db>, user_id: &str, prompt_id: &str) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let p = own_prompt(db, user_id, prompt_id).await?;
    if p.state != "shown" && p.state != "pending" {
        return Ok(());
    }
    db.query(
        "UPDATE type::record('survey_prompt', $k) SET state = 'dismissed', closed_at = time::now(), \
             shown_at = shown_at ?? time::now() \
         WHERE user = type::record('user', $u) AND state IN ['shown', 'pending']",
    )
    .bind(("k", record_key(prompt_id, "survey_prompt")))
    .bind(("u", user_key(user_id)))
    .await?
    .check()?;
    Ok(())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbSurvey {
    id: RecordId,
    seed_key: Option<String>,
    question: String,
    kind: String,
    choices: Vec<String>,
    scale_low_label: Option<String>,
    scale_high_label: Option<String>,
    trigger: String,
    status: String,
    wording_hash: String,
    approved_hash: Option<String>,
    approved_by_name: Option<String>,
    approved_at: Option<Datetime>,
    note: Option<String>,
    updated_at: Option<Datetime>,
    created_at: Datetime,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct StateCount {
    survey: RecordId,
    state: String,
    n: i64,
}

/// Every survey with its prompt counts, newest first.
pub async fn admin_list_surveys(db: &Surreal<Db>) -> DbResult<Vec<Survey>> {
    let mut r = db
        .query(
            "SELECT id, seed_key, question, kind, choices, scale_low_label, scale_high_label, trigger, \
                 status, wording_hash, approved_hash, approved_by.username AS approved_by_name, \
                 approved_at, note, updated_at, created_at \
             FROM survey ORDER BY created_at DESC; \
             SELECT survey, state, count() AS n FROM survey_prompt GROUP BY survey, state;",
        )
        .await?;
    let rows: Vec<DbSurvey> = r.take(0)?;
    let counts: Vec<StateCount> = r.take(1)?;
    let mut by_survey: HashMap<String, SurveyCounts> = HashMap::new();
    for c in counts {
        let e = by_survey.entry(c.survey.to_sql()).or_default();
        let n = c.n.max(0) as u32;
        match c.state.as_str() {
            "pending" => e.pending = n,
            "shown" => e.shown = n,
            "answered" => e.answered = n,
            "dismissed" => e.dismissed = n,
            _ => e.expired = n,
        }
    }
    let mut list: Vec<(String, Survey)> = rows
        .into_iter()
        .map(|s| {
            let id = s.id.to_sql();
            (
                dt_to_string(&s.created_at),
                Survey {
                    counts: by_survey.remove(&id).unwrap_or_default(),
                    id,
                    seed_key: s.seed_key,
                    wording: SurveyWording {
                        question: s.question,
                        kind: s.kind,
                        choices: s.choices,
                        scale_low_label: s.scale_low_label,
                        scale_high_label: s.scale_high_label,
                    },
                    trigger: s.trigger,
                    status: s.status,
                    wording_hash: s.wording_hash,
                    approved_hash: s.approved_hash,
                    approved_by: s.approved_by_name,
                    approved_at: opt_dt(s.approved_at),
                    note: s.note,
                    updated_at: opt_dt(s.updated_at),
                },
            )
        })
        .collect();
    list.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.id.cmp(&b.1.id)));
    Ok(list.into_iter().map(|(_, s)| s).collect())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct HashStatus {
    wording_hash: String,
    status: String,
}

/// Create or update a survey. A changed wording withdraws any approval: an
/// approved, live or paused survey returns to draft. Returns the id.
pub async fn admin_save_survey(
    db: &Surreal<Db>,
    admin_id: &str,
    input: &SurveyInput,
) -> DbResult<String> {
    let (wording, trigger, note) =
        crate::models::community::validate_survey(input).map_err(DbError::Invalid)?;
    let hash = wording_hash(&wording);
    let admin = user_key(admin_id);
    let _guard = invariant_lock().await;
    match input.id.as_deref().filter(|s| !s.trim().is_empty()) {
        None => {
            let mut r = db
                .query(
                    "CREATE ONLY survey SET question = $q, kind = $kind, choices = $choices, \
                         scale_low_label = $low, scale_high_label = $high, trigger = $trigger, \
                         status = 'draft', wording_hash = $hash, note = $note, \
                         created_by = type::record('user', $admin), created_at = time::now() RETURN id",
                )
                .bind(("q", wording.question))
                .bind(("kind", wording.kind))
                .bind(("choices", wording.choices))
                .bind(("low", wording.scale_low_label))
                .bind(("high", wording.scale_high_label))
                .bind(("trigger", trigger.as_str().to_string()))
                .bind(("hash", hash))
                .bind(("note", note))
                .bind(("admin", admin))
                .await?;
            let row: Option<IdRow> = r.take(0)?;
            row.map(|r| r.id.to_sql())
                .ok_or_else(|| DbError::Other("survey not created".into()))
        }
        Some(id) => {
            let key = record_key(id, "survey");
            let mut r = db
                .query("SELECT wording_hash, status FROM type::record('survey', $key)")
                .bind(("key", key.clone()))
                .await?;
            let current: Option<HashStatus> = r.take(0)?;
            let current = current.ok_or(DbError::NotFound)?;
            if current.status == "closed" {
                return Err(DbError::Invalid(
                    "A closed survey cannot be changed. Write a new one.".into(),
                ));
            }
            let wording_changed = current.wording_hash != hash;
            let status = if wording_changed
                && matches!(current.status.as_str(), "approved" | "live" | "paused")
            {
                "draft".to_string()
            } else {
                current.status
            };
            db.query(
                "UPDATE type::record('survey', $key) SET question = $q, kind = $kind, choices = $choices, \
                     scale_low_label = $low, scale_high_label = $high, trigger = $trigger, \
                     status = $status, wording_hash = $hash, note = $note, \
                     updated_by = type::record('user', $admin), updated_at = time::now()",
            )
            .bind(("key", key))
            .bind(("q", wording.question))
            .bind(("kind", wording.kind))
            .bind(("choices", wording.choices))
            .bind(("low", wording.scale_low_label))
            .bind(("high", wording.scale_high_label))
            .bind(("trigger", trigger.as_str().to_string()))
            .bind(("status", status))
            .bind(("hash", hash))
            .bind(("note", note))
            .bind(("admin", admin))
            .await?
            .check()?;
            Ok(id.to_string())
        }
    }
}

/// Approve exactly the wording the admin saw (`seen_hash`).
pub async fn admin_approve_survey(
    db: &Surreal<Db>,
    admin_id: &str,
    id: &str,
    seen_hash: &str,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let key = record_key(id, "survey");
    let mut r = db
        .query("SELECT wording_hash, status FROM type::record('survey', $key)")
        .bind(("key", key.clone()))
        .await?;
    let current: Option<HashStatus> = r.take(0)?;
    let current = current.ok_or(DbError::NotFound)?;
    if current.wording_hash != seen_hash {
        return Err(DbError::Invalid(
            "This wording changed since you opened it. Reload and check it before approving."
                .into(),
        ));
    }
    if current.status == "closed" {
        return Err(DbError::Invalid("This survey is closed.".into()));
    }
    let status = if current.status == "draft" {
        "approved".to_string()
    } else {
        current.status
    };
    db.query(
        "UPDATE type::record('survey', $key) SET approved_hash = wording_hash, \
             approved_by = type::record('user', $admin), approved_at = time::now(), status = $status",
    )
    .bind(("key", key))
    .bind(("admin", user_key(admin_id)))
    .bind(("status", status))
    .await?
    .check()?;
    Ok(())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct StatusApproval {
    status: String,
    wording_hash: String,
    approved_hash: Option<String>,
}

/// Move a survey: approved -> live, live <-> paused, any -> closed. Going
/// live needs a valid approval.
pub async fn admin_set_survey_status(
    db: &Surreal<Db>,
    admin_id: &str,
    id: &str,
    to: SurveyStatus,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let key = record_key(id, "survey");
    let mut r = db
        .query("SELECT status, wording_hash, approved_hash FROM type::record('survey', $key)")
        .bind(("key", key.clone()))
        .await?;
    let cur: Option<StatusApproval> = r.take(0)?;
    let cur = cur.ok_or(DbError::NotFound)?;
    let from = SurveyStatus::parse(&cur.status).unwrap_or(SurveyStatus::Draft);
    if to == SurveyStatus::Live && cur.approved_hash.as_deref() != Some(cur.wording_hash.as_str()) {
        return Err(DbError::Invalid(
            "Only an approved survey can go live.".into(),
        ));
    }
    if !from.can_move_to(to) {
        return Err(DbError::Invalid(format!(
            "A {} survey cannot be set to {}.",
            from.label().to_lowercase(),
            to.label().to_lowercase()
        )));
    }
    db.query(
        "UPDATE type::record('survey', $key) SET status = $status, \
             updated_by = type::record('user', $admin), updated_at = time::now()",
    )
    .bind(("key", key))
    .bind(("status", to.as_str().to_string()))
    .bind(("admin", user_key(admin_id)))
    .await?
    .check()?;
    Ok(())
}

/// Delete a draft survey nobody was asked yet (otherwise close it).
pub async fn admin_delete_survey(db: &Surreal<Db>, id: &str) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let key = record_key(id, "survey");
    let mut r = db
        .query(
            "SELECT wording_hash, status FROM type::record('survey', $key); \
             SELECT count() AS n FROM survey_prompt WHERE survey = type::record('survey', $key) GROUP ALL;",
        )
        .bind(("key", key.clone()))
        .await?;
    let cur: Option<HashStatus> = r.take(0)?;
    let cur = cur.ok_or(DbError::NotFound)?;
    let prompts: Option<CountRow> = r.take(1)?;
    if cur.status != "draft" || prompts.is_some_and(|c| c.n > 0) {
        return Err(DbError::Invalid(
            "Only drafts without answers can be deleted. Close the survey instead.".into(),
        ));
    }
    db.query("DELETE type::record('survey', $key)")
        .bind(("key", key))
        .await?
        .check()?;
    Ok(())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbAnswer {
    state: String,
    answer_hash: Option<String>,
    answer_choice: Option<i64>,
    answer_scale: Option<i64>,
    answer_text: Option<String>,
    closed_at: Option<Datetime>,
}

/// Counts and answers of one survey, grouped by the wording users saw.
/// Text answers carry a date only, never a user or team.
pub async fn admin_survey_results(db: &Surreal<Db>, id: &str) -> DbResult<SurveyResults> {
    let surveys = admin_list_surveys(db).await?;
    let wanted = format!("survey:{}", record_key(id, "survey"));
    let s = surveys
        .into_iter()
        .find(|s| s.id == wanted || s.id == id)
        .ok_or(DbError::NotFound)?;
    let mut r = db
        .query(
            "SELECT state, answer_hash, answer_choice, answer_scale, answer_text, closed_at \
             FROM survey_prompt WHERE survey = type::record('survey', $key) ORDER BY closed_at DESC",
        )
        .bind(("key", record_key(id, "survey")))
        .await?;
    let rows: Vec<DbAnswer> = r.take(0)?;
    let mut versions: BTreeMap<String, SurveyVersionResult> = BTreeMap::new();
    for a in rows.into_iter().filter(|a| a.state == "answered") {
        let hash = a.answer_hash.unwrap_or_default();
        let current = hash == s.wording_hash;
        let v = versions.entry(hash.clone()).or_insert_with(|| {
            let choice_counts = if current {
                s.wording.choices.iter().map(|c| (c.clone(), 0)).collect()
            } else {
                Vec::new()
            };
            SurveyVersionResult {
                hash: hash.clone(),
                question: if current {
                    s.wording.question.clone()
                } else {
                    String::new()
                },
                choice_counts,
                scale_counts: vec![0; 5],
                ..Default::default()
            }
        });
        v.answers += 1;
        if let Some(i) = a.answer_choice {
            let i = i.max(0) as usize;
            while v.choice_counts.len() <= i {
                let n = v.choice_counts.len() + 1;
                v.choice_counts.push((format!("Option {n}"), 0));
            }
            v.choice_counts[i].1 += 1;
        }
        if let Some(sc) = a.answer_scale {
            if (1..=5).contains(&sc) {
                v.scale_counts[(sc - 1) as usize] += 1;
            }
        }
        if let Some(t) = a.answer_text {
            v.texts.push((opt_dt(a.closed_at).unwrap_or_default(), t));
        }
    }
    let mut versions: Vec<SurveyVersionResult> = versions.into_values().collect();
    versions.sort_by_key(|v| v.hash != s.wording_hash);
    Ok(SurveyResults {
        kind: s.wording.kind,
        counts: s.counts,
        versions,
    })
}

// ---------------------------------------------------------------------------
// Transparency ledger
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbLedger {
    month: String,
    support_cents: i64,
    payment_fees_cents: Option<i64>,
    hosting_cents: i64,
    domain_cents: i64,
    ai_tokens_cents: i64,
    other_cents: i64,
    other_label: Option<String>,
    note: Option<String>,
    published: bool,
    published_at: Option<Datetime>,
    revised_at: Option<Datetime>,
    updated_at: Option<Datetime>,
    updated_by_name: Option<String>,
}

impl From<DbLedger> for LedgerMonth {
    fn from(m: DbLedger) -> Self {
        LedgerMonth {
            month: m.month,
            support_cents: m.support_cents,
            payment_fees_cents: m.payment_fees_cents.unwrap_or(0),
            hosting_cents: m.hosting_cents,
            domain_cents: m.domain_cents,
            ai_tokens_cents: m.ai_tokens_cents,
            other_cents: m.other_cents,
            other_label: m.other_label,
            note: m.note,
            published: m.published,
            published_at: opt_dt(m.published_at),
            revised_at: opt_dt(m.revised_at),
            updated_at: opt_dt(m.updated_at),
            updated_by: m.updated_by_name,
        }
    }
}

const LEDGER_FIELDS: &str =
    "month, support_cents, payment_fees_cents, hosting_cents, domain_cents, ai_tokens_cents, \
     other_cents, other_label, note, published, published_at, revised_at, updated_at, \
     updated_by.username AS updated_by_name";

/// Published months, newest first.
pub async fn list_published_ledger(db: &Surreal<Db>) -> DbResult<Vec<LedgerMonth>> {
    let q = format!(
        "SELECT {LEDGER_FIELDS} FROM ledger_month WHERE published = true ORDER BY month DESC"
    );
    let mut r = db.query(q).await?;
    let rows: Vec<DbLedger> = r.take(0)?;
    Ok(rows.into_iter().map(LedgerMonth::from).collect())
}

/// Every month (drafts included), newest first.
pub async fn admin_list_ledger(db: &Surreal<Db>) -> DbResult<Vec<LedgerMonth>> {
    let q = format!("SELECT {LEDGER_FIELDS} FROM ledger_month ORDER BY month DESC");
    let mut r = db.query(q).await?;
    let rows: Vec<DbLedger> = r.take(0)?;
    Ok(rows.into_iter().map(LedgerMonth::from).collect())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct PubState {
    published: bool,
    published_at: Option<Datetime>,
}

/// Insert or update a (validated) month. A month that was ever public is
/// marked revised. With `create_only` ("Add month") an existing month is
/// refused rather than overwritten. Returns whether it was a revision.
pub async fn admin_save_ledger_month(
    db: &Surreal<Db>,
    admin_id: &str,
    m: &LedgerMonth,
    create_only: bool,
) -> DbResult<bool> {
    let _guard = invariant_lock().await;
    let mut r = db
        .query("SELECT published, published_at FROM type::record('ledger_month', $month)")
        .bind(("month", m.month.clone()))
        .await?;
    let existing: Option<PubState> = r.take(0)?;
    if create_only && existing.is_some() {
        return Err(DbError::Invalid(
            "That month is already in the ledger. Use Edit on its row to change it.".into(),
        ));
    }
    let revision = existing
        .as_ref()
        .is_some_and(|e| e.published || e.published_at.is_some());
    db.query(
        "UPSERT type::record('ledger_month', $month) SET month = $month, \
             support_cents = $support, payment_fees_cents = $fees, hosting_cents = $hosting, domain_cents = $domain, \
             ai_tokens_cents = $ai, other_cents = $other, other_label = $other_label, note = $note, \
             published = $published, published_at = <option<datetime>> $published_at, \
             revised_at = IF $revision { time::now() } ELSE { NONE }, \
             updated_by = type::record('user', $admin), updated_at = time::now()",
    )
    .bind(("month", m.month.clone()))
    .bind(("support", m.support_cents))
    .bind(("fees", m.payment_fees_cents))
    .bind(("hosting", m.hosting_cents))
    .bind(("domain", m.domain_cents))
    .bind(("ai", m.ai_tokens_cents))
    .bind(("other", m.other_cents))
    .bind(("other_label", m.other_label.clone()))
    .bind(("note", m.note.clone()))
    .bind(("revision", revision))
    .bind(("published", existing.as_ref().is_some_and(|e| e.published)))
    .bind((
        "published_at",
        existing.as_ref().and_then(|e| e.published_at),
    ))
    .bind(("admin", user_key(admin_id)))
    .await?
    .check()?;
    Ok(revision)
}

/// Publish a month, or unpublish it within 24 hours of first publishing.
pub async fn admin_publish_ledger_month(
    db: &Surreal<Db>,
    admin_id: &str,
    month: &str,
    publish: bool,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let mut r = db
        .query("SELECT published, published_at FROM type::record('ledger_month', $month)")
        .bind(("month", month.to_string()))
        .await?;
    let cur: Option<PubState> = r.take(0)?;
    let cur = cur.ok_or(DbError::NotFound)?;
    if !publish && cur.published {
        let within = cur
            .published_at
            .as_ref()
            .is_some_and(|t| Utc::now() - to_utc(t) < chrono::Duration::hours(24));
        if !within {
            return Err(DbError::Invalid(
                "Published months can only be unpublished within 24 hours. Save a revision instead."
                    .into(),
            ));
        }
    }
    db.query(
        "UPDATE type::record('ledger_month', $month) SET published = $publish, \
             published_at = IF $publish { published_at ?? time::now() } ELSE { published_at }, \
             updated_by = type::record('user', $admin), updated_at = time::now()",
    )
    .bind(("month", month.to_string()))
    .bind(("publish", publish))
    .bind(("admin", user_key(admin_id)))
    .await?
    .check()?;
    Ok(())
}

/// Delete a month that was never published.
pub async fn admin_delete_ledger_month(db: &Surreal<Db>, month: &str) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let mut r = db
        .query("SELECT published, published_at FROM type::record('ledger_month', $month)")
        .bind(("month", month.to_string()))
        .await?;
    let cur: Option<PubState> = r.take(0)?;
    let cur = cur.ok_or(DbError::NotFound)?;
    if cur.published || cur.published_at.is_some() {
        return Err(DbError::Invalid("Only drafts can be deleted.".into()));
    }
    db.query("DELETE type::record('ledger_month', $month)")
        .bind(("month", month.to_string()))
        .await?
        .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Site texts
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbText {
    slug: String,
    body: String,
    approved: bool,
    approved_by_name: Option<String>,
    approved_at: Option<Datetime>,
    updated_by_name: Option<String>,
    updated_at: Option<Datetime>,
}

/// Every site text, in admin order.
pub async fn list_site_texts(db: &Surreal<Db>) -> DbResult<Vec<SiteText>> {
    let mut r = db
        .query(
            "SELECT slug, body, approved, approved_by.username AS approved_by_name, approved_at, \
                 updated_by.username AS updated_by_name, updated_at FROM site_text",
        )
        .await?;
    let rows: Vec<DbText> = r.take(0)?;
    let mut by_slug: HashMap<String, DbText> =
        rows.into_iter().map(|t| (t.slug.clone(), t)).collect();
    Ok(SITE_TEXTS
        .iter()
        .filter_map(|(slug, purpose)| {
            by_slug.remove(*slug).map(|t| SiteText {
                slug: t.slug,
                purpose: purpose.to_string(),
                body: t.body,
                approved: t.approved,
                approved_by: t.approved_by_name,
                approved_at: opt_dt(t.approved_at),
                updated_by: t.updated_by_name,
                updated_at: opt_dt(t.updated_at),
            })
        })
        .collect())
}

/// slug -> body of every approved text.
pub async fn approved_texts(db: &Surreal<Db>) -> DbResult<HashMap<String, String>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        slug: String,
        body: String,
    }
    let mut r = db
        .query("SELECT slug, body FROM site_text WHERE approved = true")
        .await?;
    let rows: Vec<Row> = r.take(0)?;
    Ok(rows.into_iter().map(|t| (t.slug, t.body)).collect())
}

/// Save a (validated) text. A changed body withdraws the approval.
pub async fn admin_save_site_text(
    db: &Surreal<Db>,
    admin_id: &str,
    slug: &str,
    body: &str,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let mut r = db
        .query(
            "UPDATE site_text SET \
                 approved = IF body = $body { approved } ELSE { false }, \
                 approved_by = IF body = $body { approved_by } ELSE { NONE }, \
                 approved_at = IF body = $body { approved_at } ELSE { NONE }, \
                 body = $body, updated_by = type::record('user', $admin), updated_at = time::now() \
             WHERE slug = $slug RETURN id",
        )
        .bind(("slug", slug.to_string()))
        .bind(("body", body.to_string()))
        .bind(("admin", user_key(admin_id)))
        .await?;
    let rows: Vec<IdRow> = r.take(0)?;
    if rows.is_empty() {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Approve exactly the body the admin saw.
pub async fn admin_approve_site_text(
    db: &Surreal<Db>,
    admin_id: &str,
    slug: &str,
    seen_body: &str,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    let mut r = db
        .query(
            "UPDATE site_text SET approved = true, approved_by = type::record('user', $admin), \
                 approved_at = time::now() WHERE slug = $slug AND body = $body RETURN id; \
             SELECT id FROM site_text WHERE slug = $slug;",
        )
        .bind(("slug", slug.to_string()))
        .bind(("body", seen_body.to_string()))
        .bind(("admin", user_key(admin_id)))
        .await?;
    let updated: Vec<IdRow> = r.take(0)?;
    let exists: Vec<IdRow> = r.take(1)?;
    if exists.is_empty() {
        return Err(DbError::NotFound);
    }
    if updated.is_empty() {
        return Err(DbError::Invalid(
            "This text changed since you opened it. Reload and check it before approving.".into(),
        ));
    }
    Ok(())
}

/// Withdraw an approval without changing the text.
pub async fn admin_withdraw_site_text(
    db: &Surreal<Db>,
    admin_id: &str,
    slug: &str,
) -> DbResult<()> {
    db.query(
        "UPDATE site_text SET approved = false, approved_by = NONE, approved_at = NONE, \
             updated_by = type::record('user', $admin), updated_at = time::now() WHERE slug = $slug",
    )
    .bind(("slug", slug.to_string()))
    .bind(("admin", user_key(admin_id)))
    .await?
    .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Changelog links and the admin overview
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct LinkRow {
    id: RecordId,
    title: String,
    slug: String,
}

/// Public bug-board reports fixed in, and published proposals shipped in,
/// each changelog entry.
pub async fn changelog_links(db: &Surreal<Db>) -> DbResult<Vec<EntryLinks>> {
    let mut r = db
        .query(
            "SELECT id, public_title AS title, fixed_in AS slug FROM bug_report \
               WHERE on_board = true AND public_title != NONE AND status = 'fixed' AND fixed_in != NONE; \
             SELECT id, title, shipped_in AS slug FROM proposal \
               WHERE published = true AND status = 'shipped' AND shipped_in != NONE;",
        )
        .await?;
    let fixed: Vec<LinkRow> = r.take(0)?;
    let shipped: Vec<LinkRow> = r.take(1)?;
    let mut by_slug: BTreeMap<String, EntryLinks> = BTreeMap::new();
    for f in fixed {
        by_slug
            .entry(f.slug.clone())
            .or_insert_with(|| EntryLinks {
                slug: f.slug.clone(),
                ..Default::default()
            })
            .fixed_reports
            .push((f.id.to_sql(), f.title));
    }
    for s in shipped {
        by_slug
            .entry(s.slug.clone())
            .or_insert_with(|| EntryLinks {
                slug: s.slug.clone(),
                ..Default::default()
            })
            .shipped_proposals
            .push((s.id.to_sql(), s.title));
    }
    Ok(by_slug.into_values().collect())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct GroupCount {
    k: Option<String>,
    n: i64,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct LastMonth {
    month: String,
}

/// Counts for the admin hub (invites are counted by the caller).
pub async fn admin_overview(db: &Surreal<Db>) -> DbResult<AdminOverview> {
    let mut r = db
        .query(
            "SELECT status AS k, count() AS n FROM bug_report GROUP BY k; \
             SELECT status AS k, count() AS n FROM survey WHERE approved_hash = NONE OR approved_hash != wording_hash GROUP BY k; \
             SELECT count() AS n FROM survey WHERE status = 'live' GROUP ALL; \
             SELECT count() AS n FROM proposal WHERE published = true AND status IN ['proposed', 'planned', 'building'] GROUP ALL; \
             SELECT count() AS n FROM proposal WHERE published = false GROUP ALL; \
             SELECT count() AS n FROM proposal_vote GROUP ALL; \
             SELECT month FROM ledger_month WHERE published = true ORDER BY month DESC LIMIT 1; \
             SELECT count() AS n FROM ledger_month WHERE published = false GROUP ALL; \
             SELECT approved AS k, count() AS n FROM (SELECT <string> approved AS approved FROM site_text) GROUP BY k;",
        )
        .await?;
    let bugs: Vec<GroupCount> = r.take(0)?;
    let waiting: Vec<GroupCount> = r.take(1)?;
    let live: Option<CountRow> = r.take(2)?;
    let open: Option<CountRow> = r.take(3)?;
    let unpublished: Option<CountRow> = r.take(4)?;
    let votes: Option<CountRow> = r.take(5)?;
    let last: Option<LastMonth> = r.take(6)?;
    let drafts: Option<CountRow> = r.take(7)?;
    let texts: Vec<GroupCount> = r.take(8)?;
    let n = |c: Option<CountRow>| c.map(|c| c.n.max(0) as u32).unwrap_or(0);
    let bug = |s: &str| {
        bugs.iter()
            .filter(|b| b.k.as_deref() == Some(s))
            .map(|b| b.n.max(0) as u32)
            .sum::<u32>()
    };
    Ok(AdminOverview {
        bugs_new: bug("new"),
        bugs_triaged: bug("triaged"),
        surveys_waiting: waiting
            .iter()
            .filter(|w| w.k.as_deref() != Some("closed"))
            .map(|w| w.n.max(0) as u32)
            .sum(),
        surveys_live: n(live),
        proposals_open: n(open),
        proposals_unpublished: n(unpublished),
        votes: n(votes),
        ledger_last_published: last.map(|l| l.month),
        ledger_drafts: n(drafts),
        invites_open: 0,
        texts_waiting: texts
            .iter()
            .filter(|t| t.k.as_deref() != Some("true"))
            .map(|t| t.n.max(0) as u32)
            .sum(),
        texts_total: texts.iter().map(|t| t.n.max(0) as u32).sum(),
    })
}

// ---------------------------------------------------------------------------
// Feedback export (M137): every bug report and proposal for the inbox files
// ---------------------------------------------------------------------------

/// One bug report as the feedback export reads it. The reporter's admin
/// fields come from their `user` row (never from the report's text); the
/// email is used only to apply the `ADMIN_EMAILS` rule and is never written
/// out.
#[derive(Debug, Clone, Deserialize, SurrealValue)]
pub struct FeedbackBugRow {
    pub id: RecordId,
    pub user: RecordId,
    pub reporter_email: Option<String>,
    pub reporter_flag: Option<bool>,
    pub reporter_promoted: Option<Datetime>,
    pub page_url: String,
    pub element_label: String,
    pub description: String,
    pub category: String,
    pub viewport_w: Option<i64>,
    pub viewport_h: Option<i64>,
    pub created_at: Datetime,
    pub updated_at: Option<Datetime>,
    pub status: String,
    pub admin_note: Option<String>,
    pub public_title: Option<String>,
    pub proposal: Option<RecordId>,
}

/// One proposal as the feedback export reads it, with its vote counts.
#[derive(Debug, Clone, Deserialize, SurrealValue)]
pub struct FeedbackProposalRow {
    pub id: RecordId,
    pub title: String,
    pub summary: String,
    pub status: String,
    pub status_note: Option<String>,
    pub published: bool,
    pub source: String,
    pub created_at: Datetime,
    pub updated_at: Option<Datetime>,
    pub status_changed_at: Option<Datetime>,
    pub created_by: Option<RecordId>,
    pub creator_email: Option<String>,
    pub creator_flag: Option<bool>,
    pub creator_promoted: Option<Datetime>,
    pub votes: i64,
    pub voters: i64,
}

/// Every bug report (all statuses) and every proposal, oldest first, for
/// the feedback export (`server::bug_report_export`). Global, admin-facing
/// data: only the export task calls it, never a request on a user's behalf.
pub async fn export_feedback_rows(
    db: &Surreal<Db>,
) -> DbResult<(Vec<FeedbackBugRow>, Vec<FeedbackProposalRow>)> {
    let mut r = db
        .query(
            "SELECT id, user, user.email AS reporter_email, user.is_admin AS reporter_flag, \
                 user.admin_promoted_at AS reporter_promoted, page_url, element_label, \
                 description, category, viewport_w, viewport_h, created_at, updated_at, \
                 status, admin_note, public_title, proposal \
             FROM bug_report ORDER BY created_at ASC; \
             SELECT id, title, summary, status, status_note, published, source, created_at, \
                 updated_at, status_changed_at, created_by, created_by.email AS creator_email, \
                 created_by.is_admin AS creator_flag, \
                 created_by.admin_promoted_at AS creator_promoted, \
                 (count(SELECT id FROM proposal_vote WHERE proposal = $parent.id)) AS votes, \
                 (array::len(array::distinct((SELECT VALUE user FROM proposal_vote \
                     WHERE proposal = $parent.id)))) AS voters \
             FROM proposal ORDER BY created_at ASC;",
        )
        .await?;
    let bugs: Vec<FeedbackBugRow> = r.take(0)?;
    let proposals: Vec<FeedbackProposalRow> = r.take(1)?;
    Ok((bugs, proposals))
}
