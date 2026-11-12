//! M159 item 5: the synced games a team review can be of, team-scoped. The
//! rows come from the team's synced games (`db::get_team_match_stats`:
//! current members' `player_match` rows) grouped per game by
//! `pages::stats::group_matches`; the rules (three members together, near
//! the scheduled match first) are `models::review_pick`.

use crate::models::review_pick::{self, GameGroup, ReviewCandidate};
use crate::server::db::{self, DbError, DbResult};
use surrealdb::{engine::local::Db, Surreal};

/// The team's review candidates. `match_id` is a scheduled match from a
/// link: it orders the games near its start first when it is the team's,
/// and is ignored otherwise (another team's match orders nothing).
pub async fn review_candidates(
    db: &Surreal<Db>,
    team_id: &str,
    match_id: Option<&str>,
) -> DbResult<Vec<ReviewCandidate>> {
    let start = match match_id.map(str::trim).filter(|m| !m.is_empty()) {
        Some(m) => match db::get_scheduled_match(db, team_id, m).await {
            Ok(m) => Some(m.starts_at),
            Err(DbError::NotFound) => None,
            Err(e) => return Err(e),
        },
        None => None,
    };
    let rows: Vec<crate::pages::stats::TeamMatchRow> = db::get_team_match_stats(db, team_id)
        .await?
        .into_iter()
        .map(|r| crate::pages::stats::TeamMatchRow {
            match_db_id: r.match_db_id,
            user_id: r.user_id,
            username: r.username,
            riot_match_id: r.riot_match_id,
            queue_id: r.queue_id,
            game_duration: r.game_duration,
            game_end: r.game_end,
            champion: r.champion,
            kills: r.kills,
            deaths: r.deaths,
            assists: r.assists,
            cs: r.cs,
            vision_score: r.vision_score,
            damage: r.damage,
            win: r.win,
        })
        .collect();
    let groups: Vec<GameGroup> = crate::pages::stats::group_matches(&rows)
        .into_iter()
        .map(|g| GameGroup {
            riot_match_id: g.riot_match_id,
            game_end: g.game_end,
            win: g.win,
            players: g
                .players
                .into_iter()
                .map(|p| (p.user_id, p.champion))
                .collect(),
        })
        .collect();
    let reviewed: Vec<String> = db::list_post_game_learnings(db, team_id)
        .await?
        .into_iter()
        .filter_map(|r| r.match_riot_id)
        .collect();
    Ok(review_pick::review_candidates(
        groups,
        start.as_deref(),
        &reviewed,
    ))
}

/// For "Review as a team" on one game: `None` when the game is not one of
/// the team's candidates (the action is not shown), else whether the team
/// already reviewed it.
pub async fn team_review_state(
    db: &Surreal<Db>,
    team_id: &str,
    riot_match_id: &str,
) -> DbResult<Option<bool>> {
    Ok(review_candidates(db, team_id, None)
        .await?
        .into_iter()
        .find(|c| c.riot_match_id == riot_match_id)
        .map(|c| c.reviewed))
}
