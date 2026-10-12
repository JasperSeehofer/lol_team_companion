//! Account export and deletion (M8, done item 8).
//!
//! **Export** ([`export_user_data`]): everything stored about one user as a
//! JSON document: the account, the Riot link, the team membership and join
//! requests, the personal pages (pool, champion notes, learnings, goals),
//! synced games and ranked snapshots, notes and pair notes the user wrote,
//! bug reports, votes and survey answers, and a list of the team records the
//! user created (kind, id, name, date; their content is the team's). Nothing
//! about other users: no teammates, no admin notes, no password hash.
//!
//! **Deletion** ([`delete_account`]), one transaction:
//! - Personal rows are deleted: the user, the membership, join requests,
//!   pool, champion notes, learnings, goals, ranked snapshots, synced games,
//!   votes, survey prompts and answers, share links to the user's player card.
//! - Records that belong to the team (drafts, trees, series, plans, reviews,
//!   boards, notes, pair notes, the next match, share links) and the global
//!   records admins wrote (surveys, proposals, ledger months, site texts,
//!   champion traits, invites) stay, but every link to the user points at
//!   the tombstone account [`FORMER_MEMBER_ID`] ("Former member"), stored
//!   copies of the username are replaced, action items assigned to the
//!   username are unassigned and the user leaves every "brief read" list.
//! - Bug reports stay (their status, the public title and note and the
//!   triage history keep the bug board and "fixed in" links consistent), but
//!   they move to the tombstone and lose what the reporter wrote (text,
//!   pointed element, window size).
//! - A team leader whose team has other members must hand leadership over
//!   first ([`DbError::Invalid`]). A leader who is the only member deletes
//!   the team with everything it owns.
//! - Sessions are deleted by the caller (`session_store::delete_for_user`)
//!   right after the transaction.
//!
//! [`USER_LINKS`] lists every `record<user>` field of the schema with what
//! deletion does to it; a unit test fails when the schema gains a user link
//! that is not in the list, and the integration tests check that a deleted
//! user leaves no row behind in any of them.

use serde::Deserialize;
use surrealdb::{
    engine::local::Db,
    types::{RecordId, SurrealValue, Value},
    Surreal,
};

use crate::server::db::{check_tx, invariant_lock, record_key, DbError, DbResult};

/// The tombstone account that inherits authorship of team records.
pub const FORMER_MEMBER_ID: &str = "user:former_member";
/// Shown wherever a deleted user's name was.
pub const FORMER_MEMBER_NAME: &str = "Former member";

/// What deletion does with one `record<user>` field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Treatment {
    /// The row is the user's own and is deleted.
    Delete,
    /// The row stays; the link moves to the tombstone account.
    Tombstone,
    /// An array of users; the user is removed from it.
    RemoveFromArray,
    /// `team.created_by` (the leader): refuse while others are in the team,
    /// else delete the team with everything it owns.
    Leader,
}

/// Every `record<user>` field in `schema.surql`: (table, field, treatment).
pub const USER_LINKS: &[(&str, &str, Treatment)] = &[
    ("team", "created_by", Treatment::Leader),
    ("team_member", "user", Treatment::Delete),
    ("join_request", "user", Treatment::Delete),
    ("match", "synced_by", Treatment::Tombstone),
    ("player_match", "user", Treatment::Delete),
    ("draft", "created_by", Treatment::Tombstone),
    ("game_plan", "created_by", Treatment::Tombstone),
    ("post_game_learning", "created_by", Treatment::Tombstone),
    ("champion_pool", "user", Treatment::Delete),
    ("champion_note", "user", Treatment::Delete),
    ("draft_tree", "created_by", Treatment::Tombstone),
    ("series", "created_by", Treatment::Tombstone),
    ("team_note", "author", Treatment::Tombstone),
    ("invite", "created_by", Treatment::Tombstone),
    ("ranked_snapshot", "user", Treatment::Delete),
    ("personal_learning", "user", Treatment::Delete),
    ("personal_goal", "user", Treatment::Delete),
    ("bug_report", "user", Treatment::Tombstone),
    ("board", "created_by", Treatment::Tombstone),
    ("board_version", "author", Treatment::Tombstone),
    ("champion_trait", "edited_by", Treatment::Tombstone),
    ("team_relation", "author", Treatment::Tombstone),
    (
        "scheduled_match",
        "brief_read_by",
        Treatment::RemoveFromArray,
    ),
    ("scheduled_match", "created_by", Treatment::Tombstone),
    ("share_link", "created_by", Treatment::Tombstone),
    ("share_link", "target", Treatment::Delete),
    ("survey", "approved_by", Treatment::Tombstone),
    ("survey", "created_by", Treatment::Tombstone),
    ("survey", "updated_by", Treatment::Tombstone),
    ("survey_prompt", "user", Treatment::Delete),
    ("proposal", "published_by", Treatment::Tombstone),
    ("proposal", "created_by", Treatment::Tombstone),
    ("proposal", "updated_by", Treatment::Tombstone),
    ("proposal_vote", "user", Treatment::Delete),
    ("bug_report", "triaged_by", Treatment::Tombstone),
    ("ledger_month", "updated_by", Treatment::Tombstone),
    ("site_text", "approved_by", Treatment::Tombstone),
    ("site_text", "updated_by", Treatment::Tombstone),
];

/// The tombstone row. Its email has no `@` and its username key is longer
/// than registration allows, so no registration can collide with it; its
/// password hash is not a valid hash, so no sign-in can succeed.
const ENSURE_TOMBSTONE: &str = "UPSERT user:former_member SET \
     username = 'Former member', \
     username_lc = 'former member (account deleted, tombstone)', \
     email = 'former-member.invalid', \
     password_hash = '!', \
     is_admin = false;";

/// Delete one team and everything it owns (inside a transaction; `$t` is
/// the team record).
const DELETE_TEAM: &str = "\
    LET $drafts = (SELECT VALUE id FROM draft WHERE team = $t); \
    LET $trees = (SELECT VALUE id FROM draft_tree WHERE team = $t); \
    LET $nodes = (SELECT VALUE id FROM draft_tree_node WHERE tree IN $trees); \
    LET $opponents = (SELECT VALUE id FROM opponent WHERE team = $t); \
    LET $replays = (SELECT VALUE id FROM replay WHERE team = $t); \
    LET $boards = (SELECT VALUE id FROM board WHERE team = $t); \
    DELETE draft_action WHERE draft IN $drafts; \
    DELETE draft WHERE id IN $drafts; \
    DELETE tree_node_action WHERE node IN $nodes; \
    DELETE draft_tree_node WHERE id IN $nodes; \
    DELETE draft_tree WHERE id IN $trees; \
    DELETE opponent_player WHERE opponent IN $opponents; \
    DELETE opponent WHERE id IN $opponents; \
    DELETE replay_frame WHERE replay IN $replays; \
    DELETE replay_event WHERE replay IN $replays; \
    DELETE replay WHERE id IN $replays; \
    DELETE board_version WHERE board IN $boards; \
    DELETE board WHERE id IN $boards; \
    DELETE game_plan WHERE team = $t; \
    DELETE post_game_learning WHERE team = $t; \
    DELETE action_item WHERE team = $t; \
    DELETE team_note WHERE team = $t; \
    DELETE team_relation WHERE team = $t; \
    DELETE series WHERE team = $t; \
    DELETE ban_priority WHERE team = $t; \
    DELETE checklist_template WHERE team = $t; \
    DELETE checklist_instance WHERE team = $t; \
    DELETE scheduled_match WHERE team = $t; \
    DELETE share_link WHERE team = $t; \
    UPDATE match SET team_id = NONE WHERE team_id = $t; \
    DELETE join_request WHERE team = $t; \
    DELETE team_member WHERE team = $t; \
    DELETE $t;";

/// Everything the account deletion removes or rewrites for user `$u`
/// (username `$name`), after the team question was settled. The two
/// username copies (`team_note.edited_by_name`, `action_item.assigned_to`)
/// are plain strings, so they are rewritten only inside the user's own team:
/// another team may use the same free-text name for someone else.
const DELETE_USER: &str = "\
    LET $tomb = user:former_member; \
    LET $my_teams = (SELECT VALUE team FROM team_member WHERE user = $u); \
    DELETE team_member WHERE user = $u; \
    DELETE join_request WHERE user = $u; \
    DELETE player_match WHERE user = $u; \
    DELETE champion_pool WHERE user = $u; \
    DELETE champion_note WHERE user = $u; \
    DELETE ranked_snapshot WHERE user = $u; \
    DELETE personal_learning WHERE user = $u; \
    DELETE personal_goal WHERE user = $u; \
    DELETE survey_prompt WHERE user = $u; \
    DELETE proposal_vote WHERE user = $u; \
    DELETE share_link WHERE target = $u; \
    UPDATE match SET synced_by = $tomb WHERE synced_by = $u; \
    UPDATE draft SET created_by = $tomb WHERE created_by = $u; \
    UPDATE game_plan SET created_by = $tomb WHERE created_by = $u; \
    UPDATE post_game_learning SET created_by = $tomb WHERE created_by = $u; \
    UPDATE draft_tree SET created_by = $tomb WHERE created_by = $u; \
    UPDATE series SET created_by = $tomb WHERE created_by = $u; \
    UPDATE team_note SET author = $tomb, author_name = $former WHERE author = $u; \
    UPDATE team_note SET edited_by_name = $former WHERE team IN $my_teams AND edited_by_name = $name; \
    UPDATE invite SET created_by = $tomb WHERE created_by = $u; \
    UPDATE board SET created_by = $tomb WHERE created_by = $u; \
    UPDATE board_version SET author = $tomb WHERE author = $u; \
    UPDATE champion_trait SET edited_by = $tomb, edited_by_name = $former WHERE edited_by = $u; \
    UPDATE team_relation SET author = $tomb, author_name = $former WHERE author = $u; \
    UPDATE scheduled_match SET brief_read_by = array::complement(brief_read_by, [$u]) WHERE $u IN brief_read_by; \
    UPDATE scheduled_match SET created_by = $tomb WHERE created_by = $u; \
    UPDATE share_link SET created_by = $tomb WHERE created_by = $u; \
    UPDATE survey SET approved_by = $tomb WHERE approved_by = $u; \
    UPDATE survey SET created_by = $tomb WHERE created_by = $u; \
    UPDATE survey SET updated_by = $tomb WHERE updated_by = $u; \
    UPDATE proposal SET published_by = $tomb WHERE published_by = $u; \
    UPDATE proposal SET created_by = $tomb WHERE created_by = $u; \
    UPDATE proposal SET updated_by = $tomb WHERE updated_by = $u; \
    UPDATE bug_report SET triaged_by = $tomb WHERE triaged_by = $u; \
    UPDATE bug_report SET user = $tomb, description = $removed, element_label = '', viewport_w = NONE, viewport_h = NONE WHERE user = $u; \
    UPDATE ledger_month SET updated_by = $tomb WHERE updated_by = $u; \
    UPDATE site_text SET approved_by = $tomb WHERE approved_by = $u; \
    UPDATE site_text SET updated_by = $tomb WHERE updated_by = $u; \
    UPDATE action_item SET assigned_to = NONE WHERE team IN $my_teams AND assigned_to = $name; \
    DELETE $u;";

/// What a deletion did, for the log and the confirmation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deleted {
    /// The team was deleted with the account (its only member led it).
    pub team_deleted: bool,
}

/// Delete the account `user_id` (see the module docs for what that means).
pub async fn delete_account(db: &Surreal<Db>, user_id: &str) -> DbResult<Deleted> {
    let user_key = record_key(user_id, "user");
    if format!("user:{user_key}") == FORMER_MEMBER_ID {
        return Err(DbError::NotFound);
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct Me {
        username: String,
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Led {
        id: RecordId,
        /// Members other than the user.
        others: i64,
    }

    // The team question and the deletion must not interleave with a join
    // being accepted or a leadership hand-over.
    let _guard = invariant_lock().await;
    let mut r = db
        .query(
            "LET $u = type::record('user', $user_key); \
             SELECT username FROM ONLY $u; \
             SELECT id, count((SELECT id FROM team_member WHERE team = $parent.id AND user != $u)) AS others FROM team WHERE created_by = $u;",
        )
        .bind(("user_key", user_key.clone()))
        .await?;
    let me: Option<Me> = r.take(1)?;
    let Some(me) = me else {
        return Err(DbError::NotFound);
    };
    // An error here must not skip the leader check (the team would keep a
    // deleted leader), so it is propagated.
    let led: Vec<Led> = r.take(2)?;
    if led.iter().any(|t| t.others > 0) {
        return Err(DbError::Invalid(
            "You lead a team with other members. Hand leadership to one of them on the team \
             page first, then delete your account."
                .into(),
        ));
    }
    let team_ids: Vec<RecordId> = led.into_iter().map(|t| t.id).collect();
    let team_deleted = !team_ids.is_empty();

    let query = format!(
        "BEGIN TRANSACTION; \
         LET $u = type::record('user', $user_key); \
         {ENSURE_TOMBSTONE} \
         FOR $t IN $teams {{ {DELETE_TEAM} }}; \
         {DELETE_USER} \
         COMMIT TRANSACTION;"
    );
    let response = db
        .query(query)
        .bind(("user_key", user_key))
        .bind(("teams", team_ids))
        .bind(("name", me.username))
        .bind(("former", FORMER_MEMBER_NAME))
        .bind((
            "removed",
            "(The reporter deleted their account; their text was removed.)",
        ))
        .await?;
    check_tx(response)?;
    Ok(Deleted { team_deleted })
}

/// Make `new_leader_id`, a member of the team, its leader (`team.created_by`
/// is the leader). The caller checked that the signed-in user leads it.
pub async fn hand_over_leadership(
    db: &Surreal<Db>,
    team_id: &str,
    new_leader_id: &str,
) -> DbResult<()> {
    let _guard = invariant_lock().await;
    // One transaction: the THROW aborts the update (outside a transaction
    // the next statement would still run).
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             LET $t = type::record('team', $team_key); \
             LET $n = type::record('user', $user_key); \
             IF (SELECT VALUE id FROM team_member WHERE team = $t AND user = $n) = [] { THROW 'NOT_MEMBER' }; \
             UPDATE $t SET created_by = $n; \
             COMMIT TRANSACTION;",
        )
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("user_key", record_key(new_leader_id, "user")))
        .await?;
    match check_tx(response) {
        Ok(_) => {}
        Err(e) if crate::server::db::thrown_tag(&e).as_deref() == Some("NOT_MEMBER") => {
            return Err(DbError::NotFound)
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

/// Whether the user keeps their player card out of share links.
pub async fn share_card_hidden(db: &Surreal<Db>, user_id: &str) -> DbResult<bool> {
    let mut r = db
        .query("SELECT VALUE share_card_hidden FROM ONLY type::record('user', $user_key)")
        .bind(("user_key", record_key(user_id, "user")))
        .await?;
    let v: Option<Option<bool>> = r.take(0)?;
    Ok(v.flatten().unwrap_or(false))
}

/// Keep the user's player card out of share links, or allow it again.
/// Hiding revokes the live links to the card in the same transaction.
pub async fn set_share_card_hidden(db: &Surreal<Db>, user_id: &str, hidden: bool) -> DbResult<()> {
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             LET $u = type::record('user', $user_key); \
             UPDATE $u SET share_card_hidden = $hidden; \
             IF $hidden { UPDATE share_link SET revoked = true WHERE target = $u AND revoked = false }; \
             COMMIT TRANSACTION;",
        )
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("hidden", hidden))
        .await?;
    check_tx(response)?;
    Ok(())
}

/// One statement per export section: (section name, SurrealQL over `$u`).
/// Only the user's own rows, and only fields that are about the user.
const EXPORT_SECTIONS: &[(&str, &str)] = &[
    (
        "account",
        "SELECT id, username, email, created_at, riot_summoner_name, riot_puuid, riot_region, \
         last_solo_sync, is_admin, survey_opt_out, share_card_hidden FROM ONLY $u",
    ),
    (
        "team_membership",
        "SELECT team.name AS team, role AS position, roster_type AS place, \
         (team.created_by = $u) AS leader FROM team_member WHERE user = $u",
    ),
    (
        "join_requests",
        "SELECT team.name AS team, status, created_at FROM join_request WHERE user = $u",
    ),
    (
        "champion_pool",
        "SELECT champion, role, tier, notes, comfort_level, meta_tag, created_at \
         FROM champion_pool WHERE user = $u",
    ),
    (
        "champion_notes",
        "SELECT champion, role, note_type, title, content, difficulty, created_at \
         FROM champion_note WHERE user = $u",
    ),
    (
        "personal_learnings",
        "SELECT title, learning_type, champion, opponent, what_happened, what_i_learned, \
         next_time, tags, win_loss, match_riot_id, game_timestamp_ms, event_name, created_at \
         FROM personal_learning WHERE user = $u",
    ),
    (
        "personal_goals",
        "SELECT goal_type, target_value, created_at, updated_at FROM personal_goal WHERE user = $u",
    ),
    (
        "ranked_snapshots",
        "SELECT queue_type, tier, division, lp, wins, losses, snapshotted_at \
         FROM ranked_snapshot WHERE user = $u",
    ),
    (
        "synced_games",
        "SELECT match.match_id AS match_id, match.queue_id AS queue_id, \
         match.game_end AS game_end, champion, kills, deaths, assists, cs, vision_score, \
         damage, win FROM player_match WHERE user = $u",
    ),
    (
        "team_notes_written",
        "SELECT team.name AS team, content, pinned, created_at, edited_at, deleted_at \
         FROM team_note WHERE author = $u",
    ),
    (
        "pair_notes_written",
        "SELECT team.name AS team, a, b, kind, weight, reason, created_at \
         FROM team_relation WHERE author = $u",
    ),
    (
        "bug_reports",
        "SELECT page_url, element_label, description, category, viewport_w, viewport_h, \
         created_at, status, on_board, public_title, public_note, fixed_in, reporter_seen_at \
         FROM bug_report WHERE user = $u",
    ),
    (
        "votes",
        "SELECT proposal.title AS idea, created_at FROM proposal_vote WHERE user = $u",
    ),
    (
        "survey_questions",
        "SELECT survey.question AS question, state, created_at, shown_at, closed_at, \
         answer_choice, answer_scale, answer_text FROM survey_prompt WHERE user = $u",
    ),
    (
        "team_records_created",
        "array::flatten([ \
           (SELECT 'draft' AS kind, id, name, created_at FROM draft WHERE created_by = $u), \
           (SELECT 'draft_tree' AS kind, id, name, created_at FROM draft_tree WHERE created_by = $u), \
           (SELECT 'series' AS kind, id, name, created_at FROM series WHERE created_by = $u), \
           (SELECT 'game_plan' AS kind, id, name, created_at FROM game_plan WHERE created_by = $u), \
           (SELECT 'review' AS kind, id, created_at FROM post_game_learning WHERE created_by = $u), \
           (SELECT 'board' AS kind, id, name, created_at FROM board WHERE created_by = $u), \
           (SELECT 'board_version' AS kind, id, version, created_at FROM board_version WHERE author = $u), \
           (SELECT 'next_match' AS kind, id, starts_at, created_at FROM scheduled_match WHERE created_by = $u), \
           (SELECT 'share_link' AS kind, id, kind AS link_kind, created_at, expires_at, revoked FROM share_link WHERE created_by = $u) \
         ])",
    ),
    (
        "briefs_marked_read",
        "SELECT id, starts_at FROM scheduled_match WHERE $u IN brief_read_by",
    ),
];

/// Section names of the export, in order (tests check completeness).
pub fn export_section_names() -> Vec<&'static str> {
    EXPORT_SECTIONS.iter().map(|(n, _)| *n).collect()
}

/// Everything stored about `user_id`, as one JSON document.
pub async fn export_user_data(db: &Surreal<Db>, user_id: &str) -> DbResult<serde_json::Value> {
    let mut query = String::from("LET $u = type::record('user', $user_key); ");
    for (_, q) in EXPORT_SECTIONS {
        query.push_str(q);
        query.push_str("; ");
    }
    let mut r = db
        .query(query)
        .bind(("user_key", record_key(user_id, "user")))
        .await?;
    let mut sections = serde_json::Map::new();
    for (i, (name, _)) in EXPORT_SECTIONS.iter().enumerate() {
        let v: Value = r.take(i + 1)?;
        sections.insert((*name).to_string(), v.into_json_value());
    }
    if sections
        .get("account")
        .is_none_or(|a| a.is_null() || a.as_array().is_some_and(|a| a.is_empty()))
    {
        return Err(DbError::NotFound);
    }
    Ok(serde_json::json!({
        "format": "companion-export-1",
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "about": "Everything Companion stores about your account. Team records you created are listed by kind, id, name and date; their content belongs to the team and is not part of this file.",
        "not_included": [
            "Your password: only an Argon2 hash is stored, and it is never exported.",
            "Other people's data: teammates, their pages, admin notes on your bug reports.",
            "Server logs: the reverse proxy's access log (client addresses shortened) is rotated and not tied to accounts.",
            "Backups: encrypted copies of the whole database, kept for 30 days."
        ],
        "data": serde_json::Value::Object(sections),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every `record<user>` field of the schema: (table, field).
    fn schema_user_links() -> Vec<(String, String)> {
        include_str!("../../schema.surql")
            .lines()
            .filter(|l| l.contains("record<user>") || l.contains("| user>"))
            .filter_map(|l| {
                let rest = l.trim().strip_prefix("DEFINE FIELD IF NOT EXISTS ")?;
                let mut words = rest.split_whitespace();
                let field = words.next()?.to_string();
                let _on = words.next()?;
                let table = words.next()?.to_string();
                Some((table, field))
            })
            .collect()
    }

    #[test]
    fn every_user_link_has_a_treatment() {
        let schema = schema_user_links();
        assert!(schema.len() > 20, "{schema:?}");
        for (t, f) in &schema {
            assert!(
                USER_LINKS.iter().any(|(lt, lf, _)| lt == t && lf == f),
                "{t}.{f} links a user but account deletion does not handle it (USER_LINKS)"
            );
        }
        for (t, f, _) in USER_LINKS {
            assert!(
                schema.iter().any(|(st, sf)| st == t && sf == f),
                "USER_LINKS names {t}.{f}, which is not a user link in the schema"
            );
        }
    }

    /// The deletion query touches every listed field.
    #[test]
    fn the_deletion_query_covers_every_link() {
        let all = format!("{DELETE_TEAM} {DELETE_USER}");
        for (t, f, treatment) in USER_LINKS {
            let needle = match treatment {
                Treatment::Delete => format!("DELETE {t} WHERE {f} = $u"),
                Treatment::Tombstone => format!("UPDATE {t} SET {f} = $tomb"),
                Treatment::RemoveFromArray => format!("WHERE $u IN {f}"),
                Treatment::Leader => continue,
            };
            assert!(all.contains(&needle), "{t}.{f}: expected `{needle}`");
        }
    }
}
