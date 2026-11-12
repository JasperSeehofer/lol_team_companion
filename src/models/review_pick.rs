//! M159 item 5: which synced games a team review can be of. A game is a
//! candidate when at least three current members played it together (a
//! solo queue game of one member is not a team game to review); games near
//! the scheduled match the review was opened from come first. Pure: the
//! server builds the groups from the team's synced rows.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Current members who must have played a game for it to be reviewed as a
/// team (Team games' "Played together" readout stays at two).
pub const MIN_MEMBERS_TOGETHER: usize = 3;

/// A game ending within this many hours of the scheduled match's start is
/// "near the scheduled match".
pub const NEAR_HOURS: i64 = 6;

/// One synced game as the server groups it: the members (user id,
/// champion) who played it.
#[derive(Clone, Debug, PartialEq)]
pub struct GameGroup {
    pub riot_match_id: String,
    /// RFC 3339, if Riot said.
    pub game_end: Option<String>,
    pub win: bool,
    pub players: Vec<(String, String)>,
}

/// One row of the review's game picker. Never shows the Riot id; it is the
/// value the review stores.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReviewCandidate {
    pub riot_match_id: String,
    pub game_end: Option<String>,
    pub win: bool,
    /// Our members' champions in that game.
    pub champions: Vec<String>,
    /// Ended within [`NEAR_HOURS`] of the scheduled match's start.
    pub near: bool,
    /// The team already has a review of this game.
    pub reviewed: bool,
}

fn parse(t: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(t.trim())
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

/// Distinct members in a game.
fn member_count(g: &GameGroup) -> usize {
    let mut users: Vec<&str> = g.players.iter().map(|(u, _)| u.as_str()).collect();
    users.sort_unstable();
    users.dedup();
    users.len()
}

/// Games at least [`MIN_MEMBERS_TOGETHER`] current members played
/// together, newest first (a game without an end time last).
pub fn played_together(groups: Vec<GameGroup>) -> Vec<GameGroup> {
    let mut out: Vec<GameGroup> = groups
        .into_iter()
        .filter(|g| member_count(g) >= MIN_MEMBERS_TOGETHER)
        .collect();
    out.sort_by(|a, b| {
        let (ta, tb) = (
            a.game_end.as_deref().and_then(parse),
            b.game_end.as_deref().and_then(parse),
        );
        tb.cmp(&ta)
    });
    out
}

/// Marks games ending within [`NEAR_HOURS`] of `start` and lifts them to
/// the top, keeping the order inside each part. Without a start (or an
/// unreadable one) nothing is near.
pub fn order_near(list: Vec<ReviewCandidate>, start: Option<&str>) -> Vec<ReviewCandidate> {
    let start = start.and_then(parse);
    let (mut near, other): (Vec<_>, Vec<_>) = list
        .into_iter()
        .map(|mut c| {
            c.near = match (start, c.game_end.as_deref().and_then(parse)) {
                (Some(s), Some(e)) => (e - s).num_minutes().abs() <= NEAR_HOURS * 60,
                _ => false,
            };
            c
        })
        .partition(|c| c.near);
    near.extend(other);
    near
}

/// The picker's rows: games played together, newest first, near the
/// scheduled match first, each marked when the team already reviewed it.
pub fn review_candidates(
    groups: Vec<GameGroup>,
    start: Option<&str>,
    reviewed: &[String],
) -> Vec<ReviewCandidate> {
    let list = played_together(groups)
        .into_iter()
        .map(|g| ReviewCandidate {
            reviewed: reviewed.iter().any(|r| r == &g.riot_match_id),
            riot_match_id: g.riot_match_id,
            game_end: g.game_end,
            win: g.win,
            champions: g.players.into_iter().map(|(_, c)| c).collect(),
            near: false,
        })
        .collect();
    order_near(list, start)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(id: &str, end: &str, users: &[&str]) -> GameGroup {
        GameGroup {
            riot_match_id: id.into(),
            game_end: Some(end.into()),
            win: true,
            players: users
                .iter()
                .map(|u| (format!("user:{u}"), "Jinx".to_string()))
                .collect(),
        }
    }

    #[test]
    fn only_games_three_members_played_together_newest_first() {
        let groups = vec![
            game("OLD", "2026-10-01T18:00:00Z", &["a", "b", "c"]),
            game("PAIR", "2026-10-05T18:00:00Z", &["a", "b"]),
            game("NEW", "2026-10-04T18:00:00Z", &["a", "b", "c", "d", "e"]),
            // One member's two rows are one member.
            game("DUP", "2026-10-06T18:00:00Z", &["a", "a", "b"]),
            game("SOLO", "2026-10-07T18:00:00Z", &["a"]),
        ];
        let ids: Vec<String> = played_together(groups)
            .into_iter()
            .map(|g| g.riot_match_id)
            .collect();
        assert_eq!(ids, vec!["NEW".to_string(), "OLD".to_string()]);
    }

    #[test]
    fn games_within_six_hours_of_the_match_come_first() {
        let groups = vec![
            game("LATEST", "2026-10-18T09:00:00Z", &["a", "b", "c"]),
            game("AFTER", "2026-10-17T23:30:00Z", &["a", "b", "c"]),
            game("BEFORE", "2026-10-17T13:00:00Z", &["a", "b", "c"]),
            game("DAY_BEFORE", "2026-10-16T19:00:00Z", &["a", "b", "c"]),
        ];
        // The match starts on 17 Oct at 19:00: AFTER (+4.5 h) and BEFORE
        // (-6 h, the edge) are near; LATEST (+14 h) and DAY_BEFORE (-24 h)
        // are not, though 24 h would have taken them.
        let c = review_candidates(groups, Some("2026-10-17T19:00:00Z"), &[]);
        let ids: Vec<(&str, bool)> = c
            .iter()
            .map(|c| (c.riot_match_id.as_str(), c.near))
            .collect();
        assert_eq!(
            ids,
            vec![
                ("AFTER", true),
                ("BEFORE", true),
                ("LATEST", false),
                ("DAY_BEFORE", false)
            ]
        );
    }

    #[test]
    fn without_a_match_nothing_is_near_and_reviews_are_marked() {
        let groups = vec![
            game("A", "2026-10-18T09:00:00Z", &["a", "b", "c"]),
            game("B", "2026-10-17T09:00:00Z", &["a", "b", "c"]),
        ];
        let c = review_candidates(groups.clone(), None, &["B".to_string()]);
        assert!(c.iter().all(|c| !c.near));
        assert_eq!(c[0].riot_match_id, "A");
        assert!(!c[0].reviewed);
        assert!(c[1].reviewed);
        // An unreadable start is no start.
        let c = review_candidates(groups, Some("soon"), &[]);
        assert!(c.iter().all(|c| !c.near));
    }
}
