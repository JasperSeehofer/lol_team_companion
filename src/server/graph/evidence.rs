//! Match evidence from the team's own stored games.
//!
//! - **Pairs**: two of our champions in the same game on the same side (two
//!   members' rows on the same `match` record with the same result): games
//!   and wins.
//! - **Matchups**: our champion against the enemy champion in the same
//!   position, from cached match details (all ten participants) where a
//!   member's PUUID identifies our side. Only games whose detail was opened
//!   once (and so cached) count.
//!
//! Evidence is shown with its sample size and hidden below
//! [`MIN_SAMPLE`](crate::models::graph::MIN_SAMPLE) games. It corroborates a
//! rule or a team note and never moves fit on its own (see `fit.rs`).
//!
//! Computed per team and cached for ten minutes, keyed by the patch
//! version, so a new patch (or new synced games, after the TTL) recomputes.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use surrealdb::{engine::local::Db, Surreal};

use crate::models::graph::Role;
use crate::models::match_data::MatchParticipant;
use crate::server::db::{self, DbResult, TeamGameRow};

/// Games and wins.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub games: u32,
    pub wins: u32,
}

impl Tally {
    fn add(&mut self, win: bool) {
        self.games += 1;
        self.wins += u32::from(win);
    }

    /// Win share, 0 to 1.
    pub fn rate(&self) -> f32 {
        if self.games == 0 {
            0.0
        } else {
            self.wins as f32 / self.games as f32
        }
    }
}

/// The team's evidence, keyed by Data Dragon champion ids.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MatchEvidence {
    /// Distinct games considered.
    pub games: u32,
    /// Unordered pairs (smaller id first).
    pub pairs: HashMap<(String, String), Tally>,
    /// (position, ours, theirs).
    pub matchups: HashMap<(Role, String, String), Tally>,
}

impl MatchEvidence {
    pub fn pair(&self, a: &str, b: &str) -> Option<Tally> {
        let key = if a <= b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        };
        self.pairs.get(&key).copied()
    }

    /// Our `ours` against their `theirs`, in any position (summed).
    pub fn matchup(&self, ours: &str, theirs: &str) -> Option<Tally> {
        let mut t = Tally::default();
        for ((_, o, e), v) in &self.matchups {
            if o == ours && e == theirs {
                t.games += v.games;
                t.wins += v.wins;
            }
        }
        (t.games > 0).then_some(t)
    }
}

/// Build the evidence. `resolve` maps a Riot champion name to a Data
/// Dragon id (`None`: unknown, skipped); `roster` holds the members' PUUIDs.
pub fn compute(
    rows: &[TeamGameRow],
    details: &[(String, Vec<MatchParticipant>)],
    roster: &HashSet<String>,
    resolve: impl Fn(&str) -> Option<String>,
) -> MatchEvidence {
    let mut ev = MatchEvidence::default();
    // Group our rows by game.
    let mut by_game: HashMap<&str, Vec<(&str, String, bool)>> = HashMap::new();
    for r in rows {
        if let Some(id) = resolve(&r.champion) {
            by_game
                .entry(r.match_key.as_str())
                .or_default()
                .push((r.user_id.as_str(), id, r.win));
        }
    }
    ev.games = by_game.len() as u32;
    let mut keys: Vec<&&str> = by_game.keys().collect();
    keys.sort();
    for k in keys {
        let mut members = by_game[*k].clone();
        members.sort();
        members.dedup_by(|a, b| a.0 == b.0);
        for i in 0..members.len() {
            for j in i + 1..members.len() {
                let (a, b) = (&members[i].1, &members[j].1);
                // Teammates share the result; two members on opposite
                // sides of the same game (solo queue) are not a pair.
                if a == b || members[i].2 != members[j].2 {
                    continue;
                }
                let key = if a < b {
                    (a.clone(), b.clone())
                } else {
                    (b.clone(), a.clone())
                };
                ev.pairs.entry(key).or_default().add(members[i].2);
            }
        }
    }
    for (_, parts) in details {
        for ours in parts.iter().filter(|p| roster.contains(&p.puuid)) {
            let Some(role) = Role::parse(&ours.team_position) else {
                continue;
            };
            let Some(enemy) = parts
                .iter()
                .find(|p| p.team_id != ours.team_id && Role::parse(&p.team_position) == Some(role))
            else {
                continue;
            };
            if let (Some(o), Some(e)) =
                (resolve(&ours.champion_name), resolve(&enemy.champion_name))
            {
                ev.matchups.entry((role, o, e)).or_default().add(ours.win);
            }
        }
    }
    ev
}

const TTL: Duration = Duration::from_secs(600);

type CacheMap = HashMap<String, (String, Instant, Arc<MatchEvidence>)>;

fn cache() -> &'static Mutex<CacheMap> {
    static CACHE: OnceLock<Mutex<CacheMap>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The team's evidence, from the cache when it is fresh for this patch.
pub async fn for_team(db: &Surreal<Db>, team_id: &str) -> DbResult<Arc<MatchEvidence>> {
    let version = crate::server::data_dragon::version();
    if let Some((v, at, ev)) = cache()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(team_id)
        .cloned()
    {
        if v == version && at.elapsed() < TTL {
            return Ok(ev);
        }
    }
    let rows = db::team_game_rows(db, team_id).await?;
    let mut ids: Vec<String> = rows
        .iter()
        .map(|r| r.riot_match_id.clone())
        .filter(|s| !s.is_empty())
        .collect();
    ids.sort();
    ids.dedup();
    let details = db::match_participants(db, ids).await?;
    let roster: HashSet<String> = db::get_roster_puuids(db, team_id)
        .await?
        .into_iter()
        .filter_map(|m| m.riot_puuid)
        .collect();
    let table = crate::server::graph::traits::ensure_loaded(db).await?;
    let ev = Arc::new(compute(&rows, &details, &roster, |name| {
        table.get(name).map(|r| r.traits.champion.clone())
    }));
    cache()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(team_id.to_string(), (version, Instant::now(), ev.clone()));
    Ok(ev)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(game: &str, user: &str, champ: &str, win: bool) -> TeamGameRow {
        TeamGameRow {
            match_key: format!("match:{game}"),
            riot_match_id: format!("EUW1_{game}"),
            user_id: format!("user:{user}"),
            champion: champ.into(),
            win,
        }
    }

    fn part(puuid: &str, champ: &str, side: i32, pos: &str, win: bool) -> MatchParticipant {
        MatchParticipant {
            participant_id: 0,
            puuid: puuid.into(),
            summoner_name: String::new(),
            champion_name: champ.into(),
            team_id: side,
            team_position: pos.into(),
            kills: 0,
            deaths: 0,
            assists: 0,
            cs: 0,
            vision_score: 0,
            damage: 0,
            gold_earned: 0,
            items: [0; 6],
            win,
        }
    }

    fn resolve(name: &str) -> Option<String> {
        match name {
            "FiddleSticks" => Some("Fiddlesticks".into()),
            "Unknown" => None,
            other => Some(other.to_string()),
        }
    }

    #[test]
    fn pairs_count_games_played_together() {
        let rows = vec![
            row("1", "a", "Jinx", true),
            row("1", "b", "Thresh", true),
            row("2", "a", "Jinx", false),
            row("2", "b", "Thresh", false),
            row("3", "a", "Jinx", true),
            row("3", "c", "FiddleSticks", true),
            // A solo game: no pair.
            row("4", "a", "Jinx", true),
            // Two members on opposite sides of one game: not a pair.
            row("6", "a", "Jinx", true),
            row("6", "b", "Thresh", false),
            row("5", "a", "Unknown", true),
            row("5", "b", "Thresh", true),
        ];
        let ev = compute(&rows, &[], &HashSet::new(), resolve);
        assert_eq!(ev.games, 6);
        assert_eq!(ev.pair("Thresh", "Jinx"), Some(Tally { games: 2, wins: 1 }));
        assert_eq!(
            ev.pair("Jinx", "Fiddlesticks"),
            Some(Tally { games: 1, wins: 1 })
        );
        assert_eq!(ev.pair("Jinx", "Ahri"), None);
    }

    #[test]
    fn matchups_pair_same_positions_across_sides() {
        let roster: HashSet<String> = ["p-mid".to_string()].into();
        let details = vec![
            (
                "EUW1_1".to_string(),
                vec![
                    part("p-mid", "Ahri", 100, "MIDDLE", true),
                    part("x", "Syndra", 200, "MIDDLE", false),
                    part("y", "Jinx", 100, "BOTTOM", true),
                ],
            ),
            (
                "EUW1_2".to_string(),
                vec![
                    part("p-mid", "Ahri", 200, "MIDDLE", false),
                    part("x", "Syndra", 100, "MIDDLE", true),
                ],
            ),
        ];
        let ev = compute(&[], &details, &roster, resolve);
        assert_eq!(
            ev.matchup("Ahri", "Syndra"),
            Some(Tally { games: 2, wins: 1 })
        );
        assert_eq!(ev.matchup("Jinx", "Syndra"), None, "not a roster player");
    }
}
