//! Guidance queries (M9). Every function is scoped to one user (`user_id`)
//! and, where team records count, to that user's own team, read here from
//! the membership and never taken from the browser. The rules themselves
//! are pure functions in `models::guidance`.

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use surrealdb::{
    engine::local::Db,
    types::{Datetime, RecordId, SurrealValue, ToSql},
    Surreal,
};

use crate::models::guidance::{
    self, active_hints, checklist, checklist_due, eligible_intros, next_step, pick_nudge,
    welcome_due, ChecklistFacts, GuidanceRow, GuidanceState, HintUse, HomeGuide, NudgeFacts,
    PastMatch, Welcome, OPEN_ITEM_DAYS, RECENT_DAYS,
};
use crate::server::db::{self, record_key, DbError, DbResult};

/// What a write records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// Seen or used (a hint's feature was used, an introduction was seen,
    /// the welcome was started).
    Seen,
    /// Turned off ("Got it", "Skip", "Hide this checklist", "Don't show
    /// this again").
    Muted,
}

/// What "show again" resets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reset {
    /// Every first-use hint (seen and dismissed).
    Hints,
    /// The checklist's "Hide this checklist".
    Checklist,
    /// Every nudge kind turned off with "Don't show this again".
    Nudges,
}

fn parse_dt(dt: &Datetime) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&db::dt_to_string(dt))
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

/// The row id: one row per user and key, so an UPSERT never races.
fn row_key(user_id: &str, key: &str) -> String {
    format!("{}~{key}", record_key(user_id, "user"))
}

/// The user's guidance rows.
pub async fn rows(db: &Surreal<Db>, user_id: &str) -> DbResult<Vec<GuidanceRow>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        key: String,
        seen_at: Option<Datetime>,
        muted_at: Option<Datetime>,
    }
    let mut r = db
        .query("SELECT key, seen_at, muted_at FROM user_guidance WHERE user = type::record('user', $uk)")
        .bind(("uk", record_key(user_id, "user")))
        .await?;
    // An error here must not read as "nothing seen, nothing muted": that
    // would bring back everything the user turned off.
    let rows: Vec<Row> = r.take(0)?;
    Ok(rows
        .into_iter()
        .map(|r| GuidanceRow {
            key: r.key,
            seen: r.seen_at.is_some(),
            muted: r.muted_at.is_some(),
        })
        .collect())
}

/// Record that the user saw or muted `key`. The caller checks that the key
/// is one guidance knows (`models::guidance::is_known_key`).
pub async fn mark(db: &Surreal<Db>, user_id: &str, key: &str, what: Mark) -> DbResult<()> {
    if key.is_empty() || key.len() > guidance::MAX_KEY_LEN {
        return Err(DbError::Invalid("Unknown guidance item.".into()));
    }
    let q = match what {
        Mark::Seen => {
            "UPSERT type::record('user_guidance', $id) SET user = type::record('user', $uk), key = $key, seen_at = seen_at ?? time::now()"
        }
        Mark::Muted => {
            "UPSERT type::record('user_guidance', $id) SET user = type::record('user', $uk), key = $key, muted_at = time::now()"
        }
    };
    db.query(q)
        .bind(("id", row_key(user_id, key)))
        .bind(("uk", record_key(user_id, "user")))
        .bind(("key", key.to_string()))
        .await?
        .check()?;
    Ok(())
}

/// "Show hints again" / "Show the checklist again".
pub async fn reset(db: &Surreal<Db>, user_id: &str, what: Reset) -> DbResult<()> {
    let q = match what {
        Reset::Hints => {
            "DELETE user_guidance WHERE user = type::record('user', $uk) AND string::starts_with(key, 'hint.')"
        }
        Reset::Checklist => {
            "DELETE user_guidance WHERE user = type::record('user', $uk) AND key = 'checklist'"
        }
        Reset::Nudges => {
            "DELETE user_guidance WHERE user = type::record('user', $uk) AND string::starts_with(key, 'nudge.')"
        }
    };
    db.query(q)
        .bind(("uk", record_key(user_id, "user")))
        .await?
        .check()?;
    Ok(())
}

/// The visit before this one. Normally `user.prev_seen_at` (this visit's
/// first request moved the old `last_seen_at` there; see
/// `db::touch_last_seen`). While that write has not landed yet (another
/// request of the same page load is doing it), `last_seen_at` is still an
/// hour old or more and is itself the previous visit (M10 review).
async fn prev_seen(
    db: &Surreal<Db>,
    user_id: &str,
    now: DateTime<Utc>,
) -> DbResult<Option<DateTime<Utc>>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Seen {
        last_seen_at: Option<Datetime>,
        prev_seen_at: Option<Datetime>,
    }
    let mut r = db
        .query("SELECT last_seen_at, prev_seen_at FROM ONLY type::record('user', $uk)")
        .bind(("uk", record_key(user_id, "user")))
        .await?;
    let v: Option<Seen> = r.take(0)?;
    let Some(v) = v else {
        return Ok(None);
    };
    let last = v.last_seen_at.as_ref().and_then(parse_dt);
    let prev = v.prev_seen_at.as_ref().and_then(parse_dt);
    Ok(match last {
        Some(l) if now - l >= Duration::hours(1) => Some(l),
        _ => prev,
    })
}

/// The account's guidance switch and creation time.
async fn account(db: &Surreal<Db>, user_id: &str) -> DbResult<(bool, Option<DateTime<Utc>>)> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        guidance_enabled: Option<bool>,
        created_at: Option<Datetime>,
    }
    let mut r = db
        .query("SELECT guidance_enabled, created_at FROM ONLY type::record('user', $uk)")
        .bind(("uk", record_key(user_id, "user")))
        .await?;
    let row: Option<Row> = r.take(0)?;
    let row = row.ok_or(DbError::NotFound)?;
    Ok((
        row.guidance_enabled.unwrap_or(true),
        row.created_at.as_ref().and_then(parse_dt),
    ))
}

/// Whether guidance is on for the user (on unless they turned it off).
pub async fn enabled(db: &Surreal<Db>, user_id: &str) -> DbResult<bool> {
    Ok(account(db, user_id).await?.0)
}

pub async fn set_enabled(db: &Surreal<Db>, user_id: &str, on: bool) -> DbResult<()> {
    db.query("UPDATE type::record('user', $uk) SET guidance_enabled = $on")
        .bind(("uk", record_key(user_id, "user")))
        .bind(("on", on))
        .await?
        .check()?;
    Ok(())
}

/// The team the user belongs to (from the membership; never from the client).
async fn team_of(db: &Surreal<Db>, user_id: &str) -> DbResult<Option<String>> {
    db::get_user_team_id(db, user_id).await
}

/// What the checklist is computed from.
pub async fn checklist_facts(db: &Surreal<Db>, user_id: &str) -> DbResult<ChecklistFacts> {
    let Some(team_id) = team_of(db, user_id).await? else {
        return Ok(ChecklistFacts::default());
    };
    let counts = db::team_counts(db, &team_id).await?;
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Count {
        n: i64,
    }
    let mut r = db
        .query(
            "LET $t = type::record('team', $tk); \
             SELECT VALUE role FROM team_member WHERE team = $t AND roster_type = 'starter'; \
             SELECT count() AS n FROM join_request WHERE team = $t AND status = 'pending' GROUP ALL;",
        )
        .bind(("tk", record_key(&team_id, "team")))
        .await?;
    let roles: Vec<String> = r.take(1).unwrap_or_default();
    let pending: Option<Count> = r.take(2).unwrap_or_default();
    let mut positions: Vec<&str> = roles
        .iter()
        .map(String::as_str)
        .filter(|r| ["top", "jungle", "mid", "bot", "support"].contains(r))
        .collect();
    positions.sort_unstable();
    positions.dedup();
    Ok(ChecklistFacts {
        team_name: Some(counts.name).filter(|n| !n.is_empty()),
        starting_positions: positions.len(),
        opponents: counts.opponents,
        drafts: counts.drafts,
        plans: counts.plans,
        reviews: counts.reviews,
        members: counts.members,
        pending_requests: pending.map(|c| c.n.max(0) as usize).unwrap_or(0),
    })
}

/// Which hinted features the records show as used.
pub async fn hint_use(db: &Surreal<Db>, user_id: &str) -> DbResult<HintUse> {
    let team = team_of(db, user_id).await?;
    let mut r = db
        .query(
            "LET $u = type::record('user', $uk); \
             LET $t = IF $tk = NONE { NONE } ELSE { type::record('team', $tk) }; \
             SELECT VALUE id FROM draft WHERE team = $t AND created_by = $u LIMIT 1; \
             SELECT VALUE id FROM draft_tree_node WHERE parent != NONE AND tree IN (SELECT VALUE id FROM draft_tree WHERE team = $t) LIMIT 1; \
             SELECT VALUE id FROM share_link WHERE team = $t AND created_by = $u LIMIT 1; \
             SELECT VALUE id FROM team_note WHERE team = $t AND (previous_content != NONE OR deleted_at != NONE) LIMIT 1; \
             SELECT VALUE id FROM champion_pool WHERE user = $u AND tier != 'comfort' LIMIT 1;",
        )
        .bind(("uk", record_key(user_id, "user")))
        .bind(("tk", team.map(|t| record_key(&t, "team"))))
        .await?;
    let mut any = |i: usize| -> bool {
        let ids: Vec<RecordId> = r.take(i).unwrap_or_default();
        !ids.is_empty()
    };
    Ok(HintUse {
        drafted: any(2),
        branched: any(3),
        shared: any(4),
        note_changed: any(5),
        tiered: any(6),
    })
}

/// The newest record the user wrote. Edits that keep no timestamp of their
/// own (a draft's picks changed later) do not count; creating anything does.
async fn last_write(
    db: &Surreal<Db>,
    user_id: &str,
    team_id: Option<&str>,
) -> DbResult<Option<DateTime<Utc>>> {
    let mut r = db
        .query(
            "LET $u = type::record('user', $uk); \
             LET $t = IF $tk = NONE { NONE } ELSE { type::record('team', $tk) }; \
             SELECT VALUE created_at FROM draft WHERE team = $t AND created_by = $u ORDER BY created_at DESC LIMIT 1; \
             SELECT VALUE created_at FROM game_plan WHERE team = $t AND created_by = $u ORDER BY created_at DESC LIMIT 1; \
             SELECT VALUE created_at FROM post_game_learning WHERE team = $t AND created_by = $u ORDER BY created_at DESC LIMIT 1; \
             SELECT VALUE created_at FROM draft_tree WHERE team = $t AND created_by = $u ORDER BY created_at DESC LIMIT 1; \
             SELECT VALUE created_at FROM team_note WHERE team = $t AND author = $u ORDER BY created_at DESC LIMIT 1; \
             SELECT VALUE updated_at FROM scheduled_match WHERE team = $t AND created_by = $u ORDER BY updated_at DESC LIMIT 1; \
             SELECT VALUE created_at FROM series WHERE team = $t AND created_by = $u ORDER BY created_at DESC LIMIT 1; \
             SELECT VALUE created_at FROM team_relation WHERE team = $t AND author = $u ORDER BY created_at DESC LIMIT 1; \
             SELECT VALUE created_at FROM personal_learning WHERE user = $u ORDER BY created_at DESC LIMIT 1; \
             SELECT VALUE created_at FROM champion_pool WHERE user = $u ORDER BY created_at DESC LIMIT 1; \
             SELECT VALUE created_at FROM champion_note WHERE user = $u ORDER BY created_at DESC LIMIT 1;",
        )
        .bind(("uk", record_key(user_id, "user")))
        .bind(("tk", team_id.map(|t| record_key(t, "team"))))
        .await?;
    let mut newest: Option<DateTime<Utc>> = None;
    for i in 2..=12 {
        // A failed read must not look like "no write" (that would show
        // "Picking up again?" to someone who was here yesterday).
        let v: Vec<Option<Datetime>> = r.take(i)?;
        if let Some(t) = v.into_iter().flatten().next().as_ref().and_then(parse_dt) {
            newest = Some(newest.map_or(t, |n| n.max(t)));
        }
    }
    Ok(newest)
}

/// What the nudge rules read for the user's team. `team_id` must be the
/// user's own team (checked: anything else is `NotFound`).
pub async fn nudge_facts(
    db: &Surreal<Db>,
    user_id: &str,
    team_id: &str,
    now: DateTime<Utc>,
) -> DbResult<NudgeFacts> {
    if team_of(db, user_id)
        .await?
        .as_deref()
        .map(|t| record_key(t, "team"))
        != Some(record_key(team_id, "team"))
    {
        return Err(DbError::NotFound);
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Opp {
        id: RecordId,
        name: String,
        created_at: Option<Datetime>,
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Refreshed {
        opponent: RecordId,
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct DraftRow {
        id: RecordId,
        name: String,
        opponent: Option<String>,
        created_at: Option<Datetime>,
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct PastRow {
        id: RecordId,
        #[allow(dead_code)]
        starts_at: Datetime,
        kind: String,
        opponent: Option<RecordId>,
        opponent_name: Option<String>,
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Count {
        n: i64,
    }
    let recent = (now - Duration::days(RECENT_DAYS)).to_rfc3339();
    let old_items = (now - Duration::days(OPEN_ITEM_DAYS)).to_rfc3339();
    let mut r = db
        .query(
            "LET $t = type::record('team', $tk); \
             SELECT id, name, created_at FROM opponent WHERE team = $t; \
             SELECT opponent FROM opponent_player WHERE opponent IN (SELECT VALUE id FROM opponent WHERE team = $t) AND last_fetched >= <datetime>$recent; \
             SELECT id, name, opponent, created_at FROM draft WHERE team = $t AND created_at >= <datetime>$recent ORDER BY created_at DESC; \
             SELECT VALUE draft FROM game_plan WHERE team = $t AND draft != NONE; \
             SELECT id, starts_at, kind, opponent, opponent.name AS opponent_name FROM scheduled_match WHERE team = $t AND starts_at < <datetime>$now AND starts_at >= <datetime>$recent ORDER BY starts_at DESC LIMIT 1; \
             SELECT count() AS n FROM action_item WHERE team = $t AND status IN ['open', 'in_progress'] AND created_at < <datetime>$old GROUP ALL;",
        )
        .bind(("tk", record_key(team_id, "team")))
        .bind(("recent", recent.clone()))
        .bind(("old", old_items))
        .bind(("now", now.to_rfc3339()))
        .await?;
    let opponents: Vec<Opp> = r.take(1).unwrap_or_default();
    let refreshed: Vec<Refreshed> = r.take(2).unwrap_or_default();
    let drafts: Vec<DraftRow> = r.take(3).unwrap_or_default();
    let planned: Vec<RecordId> = r.take(4).unwrap_or_default();
    let past: Option<PastRow> = r.take(5).unwrap_or_default();
    let old: Option<Count> = r.take(6).unwrap_or_default();

    let cutoff = now - Duration::days(RECENT_DAYS);
    let planned: Vec<String> = planned.iter().map(|p| p.to_sql()).collect();
    let draft_without_plan = drafts
        .iter()
        .find(|d| !planned.contains(&d.id.to_sql()))
        .map(|d| {
            let name = if d.name.trim().is_empty() {
                "Untitled draft".to_string()
            } else {
                d.name.clone()
            };
            (d.id.to_sql(), name)
        });
    let drafted_against = |o: &Opp| {
        let id = o.id.to_sql();
        let name = o.name.trim().to_lowercase();
        drafts.iter().any(|d| {
            d.opponent
                .as_deref()
                .is_some_and(|x| x == id || x.trim().to_lowercase() == name)
        })
    };
    let dossier_without_draft = opponents
        .iter()
        .filter(|o| {
            let fresh = o
                .created_at
                .as_ref()
                .and_then(parse_dt)
                .is_some_and(|c| c >= cutoff);
            let refreshed = refreshed.iter().any(|p| p.opponent == o.id);
            (fresh || refreshed) && !drafted_against(o)
        })
        .max_by_key(|o| o.created_at.as_ref().and_then(parse_dt))
        .map(|o| (o.id.to_sql(), o.name.clone()));

    let mut match_without_review = None;
    if let Some(p) = past {
        if let Some(opp) = &p.opponent {
            let name = p.opponent_name.clone().unwrap_or_default();
            // M159: only the plan and review stamped to that match count.
            let facts = db::match_prep_facts(
                db,
                team_id,
                &p.id.to_sql(),
                Some(&opp.to_sql()),
                Some(&name),
            )
            .await?;
            if let (Some(plan), None) = (facts.plans.first(), facts.review_after) {
                match_without_review = Some(PastMatch {
                    opponent: name,
                    kind_label: crate::models::match_day::kind_label(&p.kind).to_string(),
                    plan_id: plan.id.clone(),
                });
            }
        }
    }

    let (_, created) = account(db, user_id).await?;
    Ok(NudgeFacts {
        now,
        last_write: last_write(db, user_id, Some(team_id)).await?,
        prev_seen: prev_seen(db, user_id, now).await?,
        account_created: created,
        match_without_review,
        old_open_items: old.map(|c| c.n.max(0) as usize).unwrap_or(0),
        draft_without_plan,
        dossier_without_draft,
    })
}

/// The shell's guidance state for the user.
pub async fn state(db: &Surreal<Db>, user_id: &str, now: DateTime<Utc>) -> DbResult<GuidanceState> {
    let (enabled, created) = account(db, user_id).await?;
    if !enabled {
        return Ok(GuidanceState::default());
    }
    let rows = rows(db, user_id).await?;
    let used = hint_use(db, user_id).await?;
    let intros = crate::server::community::changelog::intros();
    Ok(GuidanceState {
        enabled,
        hints: active_hints(enabled, &rows, &used),
        intros: eligible_intros(enabled, intros, &rows, created, now),
    })
}

/// Match Day's guidance: the welcome, the checklist and one nudge.
/// `forced` is "Guide me" (`/?guide=1`); `skip_href` is the prep sequence's
/// next link, which a nudge must not repeat.
pub async fn home_guide(
    db: &Surreal<Db>,
    user_id: &str,
    username: &str,
    riot_linked: bool,
    forced: bool,
    skip_href: Option<&str>,
    now: DateTime<Utc>,
) -> DbResult<HomeGuide> {
    let (enabled, created) = account(db, user_id).await?;
    if !enabled && !forced {
        return Ok(HomeGuide::default());
    }
    let rows = rows(db, user_id).await?;
    let facts = checklist_facts(db, user_id).await?;
    let items = checklist(&facts);
    let start_href = next_step(&items)
        .and_then(|i| items[i].href.clone())
        .unwrap_or_else(|| "/team/roster".to_string());
    let welcome = welcome_due(enabled, &rows, created, now).then(|| Welcome {
        username: username.to_string(),
        team_name: facts.team_name.clone(),
        riot_linked,
        start_href,
    });
    let show_list = checklist_due(enabled, &rows, &items, forced);
    let forced_open = forced
        && (guidance::is_muted(&rows, guidance::CHECKLIST)
            || guidance::checklist_complete(&items)
            || !enabled);
    // A nudge is optional: a failure there leaves the welcome and the
    // checklist in place.
    let nudge = match team_of(db, user_id).await? {
        Some(team) if enabled => match nudge_facts(db, user_id, &team, now).await {
            Ok(f) => pick_nudge(enabled, &f, &rows, skip_href),
            Err(e) => {
                tracing::warn!(user = %user_id, error = %e, "nudge facts failed");
                None
            }
        },
        _ => None,
    };
    Ok(HomeGuide {
        welcome,
        checklist: show_list.then_some(items),
        forced: forced_open,
        nudge,
    })
}
