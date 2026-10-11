//! Sand Table: importer from a Riot Match-V5 timeline into replay frames and
//! events (storage lives in `db.rs`; there is no UI yet).
//!
//! Input is the JSON document `GET /lol/match/v5/matches/{matchId}/timeline`
//! returns (`MatchTimelineDto`). The importer keeps:
//!
//! - every frame (one per `info.frameInterval`, normally 60 s) with each
//!   participant's position, total and current gold, xp, level and CS;
//! - the events a replay needs (kills, objectives, structures, plates,
//!   wards, level-ups, game end), each with its own timestamp, position
//!   when the timeline has one, the participants involved and the rest of
//!   its fields as `meta`.
//!
//! Item, skill and pause events are skipped; unknown event types are counted
//! in `skipped_event_types` so a new patch's events show up in logs instead
//! of breaking the import. `alive` is not in the timeline: it is estimated
//! from kill events and base respawn times (see [`BASE_RESPAWN_SECS`]).

use std::collections::BTreeMap;

use serde_json::{Map, Value};
use thiserror::Error;

use crate::models::sand_table::{
    event_kind, ParticipantFrame, ReplayEvent, ReplayFrame, ReplayParticipant,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ImportError {
    #[error("not a Match-V5 timeline: {0}")]
    Shape(String),
}

/// A timeline converted for storage.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportedTimeline {
    pub match_id: String,
    pub frame_interval_ms: i64,
    pub participants: Vec<ReplayParticipant>,
    pub frames: Vec<ReplayFrame>,
    pub events: Vec<ReplayEvent>,
    /// Event types that were not imported, with how often they occurred.
    pub skipped_event_types: BTreeMap<String, usize>,
}

/// Base respawn time in seconds by champion level 1-18 (the League wiki's
/// death timer table as of Season 14). Unverified against the current patch,
/// and it ignores the late-game time multiplier, so `alive` is an estimate
/// that errs towards "alive" late in a game.
pub const BASE_RESPAWN_SECS: [f64; 18] = [
    6.0, 6.0, 8.0, 8.0, 10.0, 12.0, 16.0, 21.0, 26.0, 32.5, 35.0, 37.5, 40.0, 42.5, 45.0, 47.5,
    50.0, 52.5,
];

fn respawn_ms(level: i64) -> i64 {
    let idx = (level.clamp(1, 18) - 1) as usize;
    (BASE_RESPAWN_SECS[idx] * 1000.0) as i64
}

fn int(v: &Value, key: &str) -> Option<i64> {
    v.get(key).and_then(Value::as_i64)
}

fn position(v: &Value) -> (Option<i64>, Option<i64>) {
    match v.get("position") {
        Some(p) => (int(p, "x"), int(p, "y")),
        None => (None, None),
    }
}

/// Fields lifted out of an event into `ReplayEvent` columns, or too large to
/// keep (per-hit damage breakdowns on kills).
const LIFTED: &[&str] = &[
    "type",
    "timestamp",
    "position",
    "killerId",
    "victimId",
    "assistingParticipantIds",
    "participantId",
    "creatorId",
    "teamId",
    "killerTeamId",
    "winningTeam",
    "victimDamageDealt",
    "victimDamageReceived",
];

fn meta_of(ev: &Value) -> Value {
    let mut m = Map::new();
    if let Some(obj) = ev.as_object() {
        for (k, v) in obj {
            if !LIFTED.contains(&k.as_str()) {
                m.insert(k.clone(), v.clone());
            }
        }
    }
    Value::Object(m)
}

/// Killer (when it is a champion: 0 means minions or a turret), then the
/// optional extra participant (victim), then assists.
fn involved(ev: &Value, actor_key: &str, extra_key: Option<&str>) -> Vec<i64> {
    let mut out = Vec::new();
    if let Some(a) = int(ev, actor_key).filter(|a| *a > 0) {
        out.push(a);
    }
    if let Some(k) = extra_key {
        if let Some(v) = int(ev, k).filter(|v| *v > 0) {
            out.push(v);
        }
    }
    if let Some(assists) = ev.get("assistingParticipantIds").and_then(Value::as_array) {
        out.extend(assists.iter().filter_map(Value::as_i64).filter(|a| *a > 0));
    }
    out
}

fn convert_event(ev: &Value) -> Option<ReplayEvent> {
    let kind_str = ev.get("type")?.as_str()?;
    let t_ms = int(ev, "timestamp")?;
    let (x, y) = position(ev);
    let (kind, participants, team_id) = match kind_str {
        "CHAMPION_KILL" => (
            event_kind::KILL,
            involved(ev, "killerId", Some("victimId")),
            None,
        ),
        "CHAMPION_SPECIAL_KILL" => (
            event_kind::SPECIAL_KILL,
            involved(ev, "killerId", None),
            None,
        ),
        "BUILDING_KILL" => (
            event_kind::BUILDING,
            involved(ev, "killerId", None),
            int(ev, "teamId"),
        ),
        "TURRET_PLATE_DESTROYED" => (
            event_kind::TURRET_PLATE,
            involved(ev, "killerId", None),
            int(ev, "teamId"),
        ),
        "ELITE_MONSTER_KILL" => (
            event_kind::OBJECTIVE,
            involved(ev, "killerId", None),
            int(ev, "killerTeamId"),
        ),
        "DRAGON_SOUL_GIVEN" => (event_kind::DRAGON_SOUL, Vec::new(), int(ev, "teamId")),
        "WARD_PLACED" => (
            event_kind::WARD_PLACED,
            involved(ev, "creatorId", None),
            None,
        ),
        "WARD_KILL" => (event_kind::WARD_KILL, involved(ev, "killerId", None), None),
        "LEVEL_UP" => (
            event_kind::LEVEL_UP,
            involved(ev, "participantId", None),
            None,
        ),
        "GAME_END" => (event_kind::GAME_END, Vec::new(), int(ev, "winningTeam")),
        _ => return None,
    };
    Some(ReplayEvent {
        seq: 0,
        t_ms,
        kind: kind.to_string(),
        x,
        y,
        participants,
        team_id,
        meta: meta_of(ev),
    })
}

/// Event types that are expected and deliberately not imported.
const IGNORED_TYPES: &[&str] = &[
    "ITEM_PURCHASED",
    "ITEM_SOLD",
    "ITEM_DESTROYED",
    "ITEM_UNDO",
    "SKILL_LEVEL_UP",
    "PAUSE_START",
    "PAUSE_END",
    "OBJECTIVE_BOUNTY_PRESTART",
    "OBJECTIVE_BOUNTY_FINISH",
    "CHAMPION_TRANSFORM",
];

/// Convert a parsed Match-V5 timeline document.
pub fn import_timeline(doc: &Value) -> Result<ImportedTimeline, ImportError> {
    let match_id = doc
        .pointer("/metadata/matchId")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ImportError::Shape("metadata.matchId is missing".into()))?
        .to_string();
    let info = doc
        .get("info")
        .ok_or_else(|| ImportError::Shape("info is missing".into()))?;
    let raw_frames = info
        .get("frames")
        .and_then(Value::as_array)
        .ok_or_else(|| ImportError::Shape("info.frames is missing".into()))?;
    let frame_interval_ms = int(info, "frameInterval").unwrap_or(60_000);

    // Participants: from info.participants, else from the first frame's keys.
    let mut participants: Vec<ReplayParticipant> = info
        .get("participants")
        .and_then(Value::as_array)
        .map(|ps| {
            ps.iter()
                .filter_map(|p| {
                    let id = int(p, "participantId")?;
                    Some(ReplayParticipant {
                        participant_id: id,
                        puuid: p.get("puuid").and_then(Value::as_str).map(str::to_string),
                        team_id: if id <= 5 { 100 } else { 200 },
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    if participants.is_empty() {
        if let Some(pf) = raw_frames
            .first()
            .and_then(|f| f.get("participantFrames"))
            .and_then(Value::as_object)
        {
            participants = pf
                .values()
                .filter_map(|p| int(p, "participantId"))
                .map(|id| ReplayParticipant {
                    participant_id: id,
                    puuid: None,
                    team_id: if id <= 5 { 100 } else { 200 },
                })
                .collect();
        }
    }
    participants.sort_by_key(|p| p.participant_id);

    // Events, in time order.
    let mut events = Vec::new();
    let mut skipped_event_types: BTreeMap<String, usize> = BTreeMap::new();
    for f in raw_frames {
        for ev in f
            .get("events")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            match convert_event(ev) {
                Some(e) => events.push(e),
                None => {
                    let t = ev
                        .get("type")
                        .and_then(Value::as_str)
                        .unwrap_or("(no type)")
                        .to_string();
                    if !IGNORED_TYPES.contains(&t.as_str()) {
                        *skipped_event_types.entry(t).or_default() += 1;
                    }
                }
            }
        }
    }
    events.sort_by_key(|e| e.t_ms);
    for (i, e) in events.iter_mut().enumerate() {
        e.seq = i as i64;
    }

    // Level of each participant at each frame, for respawn estimates.
    let level_at = |pid: i64, t: i64| -> i64 {
        raw_frames
            .iter()
            .filter(|f| int(f, "timestamp").is_some_and(|ft| ft <= t))
            .filter_map(|f| f.pointer(&format!("/participantFrames/{pid}/level")))
            .filter_map(Value::as_i64)
            .next_back()
            .unwrap_or(1)
    };
    // (victim, died at, respawns at) from the raw kill events.
    let deaths: Vec<(i64, i64, i64)> = raw_frames
        .iter()
        .flat_map(|f| {
            f.get("events")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter(|ev| ev.get("type").and_then(Value::as_str) == Some("CHAMPION_KILL"))
        .filter_map(|ev| {
            let victim = int(ev, "victimId").filter(|v| *v > 0)?;
            let t = int(ev, "timestamp")?;
            Some((victim, t, t + respawn_ms(level_at(victim, t))))
        })
        .collect();

    let mut frames = Vec::with_capacity(raw_frames.len());
    for f in raw_frames {
        let Some(t_ms) = int(f, "timestamp") else {
            continue;
        };
        let mut ps: Vec<ParticipantFrame> = f
            .get("participantFrames")
            .and_then(Value::as_object)
            .map(|pf| {
                pf.values()
                    .filter_map(|p| {
                        let pid = int(p, "participantId")?;
                        let (x, y) = position(p);
                        let dead = deaths
                            .iter()
                            .any(|(v, from, until)| *v == pid && t_ms >= *from && t_ms < *until);
                        Some(ParticipantFrame {
                            participant_id: pid,
                            x: x.unwrap_or(0),
                            y: y.unwrap_or(0),
                            gold: int(p, "totalGold").unwrap_or(0),
                            current_gold: int(p, "currentGold").unwrap_or(0),
                            xp: int(p, "xp").unwrap_or(0),
                            cs: int(p, "minionsKilled").unwrap_or(0)
                                + int(p, "jungleMinionsKilled").unwrap_or(0),
                            level: int(p, "level").unwrap_or(1),
                            alive: !dead,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        ps.sort_by_key(|p| p.participant_id);
        frames.push(ReplayFrame {
            t_ms,
            participants: ps,
        });
    }
    frames.sort_by_key(|f| f.t_ms);

    Ok(ImportedTimeline {
        match_id,
        frame_interval_ms,
        participants,
        frames,
        events,
        skipped_event_types,
    })
}

/// [`import_timeline`] from JSON text.
pub fn import_timeline_str(json: &str) -> Result<ImportedTimeline, ImportError> {
    let doc: Value =
        serde_json::from_str(json).map_err(|e| ImportError::Shape(format!("invalid JSON: {e}")))?;
    import_timeline(&doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../tests/fixtures/riot_match_v5_timeline.json");

    fn imported() -> ImportedTimeline {
        import_timeline_str(FIXTURE).unwrap()
    }

    #[test]
    fn reads_match_and_participants() {
        let t = imported();
        assert_eq!(t.match_id, "EUW1_7100000001");
        assert_eq!(t.frame_interval_ms, 60_000);
        assert_eq!(t.participants.len(), 10);
        assert_eq!(t.participants[0].participant_id, 1);
        assert_eq!(t.participants[0].team_id, 100);
        assert_eq!(t.participants[9].team_id, 200);
        assert_eq!(t.participants[3].puuid.as_deref(), Some("fixture-puuid-04"));
    }

    #[test]
    fn frames_carry_position_economy_and_level() {
        let t = imported();
        assert_eq!(
            t.frames.iter().map(|f| f.t_ms).collect::<Vec<_>>(),
            vec![0, 60_000, 120_000, 180_000]
        );
        let f1 = &t.frames[1];
        assert_eq!(f1.participants.len(), 10);
        let p2 = &f1.participants[1];
        assert_eq!(p2.participant_id, 2);
        assert_eq!((p2.x, p2.y), (3600, 7800));
        assert_eq!(p2.gold, 870);
        assert_eq!(p2.current_gold, 290);
        assert_eq!(p2.xp, 280);
        assert_eq!(p2.level, 2);
        assert_eq!(p2.cs, 4, "lane minions plus jungle monsters");
        assert!(t.frames[0].participants.iter().all(|p| p.alive));
    }

    #[test]
    fn alive_is_estimated_from_kills() {
        let t = imported();
        // Participant 6 dies at 178 s at level 3 (8 s base timer): dead in
        // the 180 s frame.
        let last = &t.frames[3];
        assert!(!last.participants[5].alive);
        assert!(
            last.participants[1].alive,
            "participant 2 respawned after 125 s"
        );
        assert!(
            t.frames[2].participants[1].alive,
            "the 120 s frame is before the kill"
        );
    }

    #[test]
    fn events_are_converted_in_time_order() {
        let t = imported();
        let kinds: Vec<&str> = t.events.iter().map(|e| e.kind.as_str()).collect();
        assert_eq!(
            kinds,
            vec![
                "ward_placed",
                "level_up",
                "kill",
                "special_kill",
                "ward_kill",
                "objective",
                "turret_plate",
                "kill",
                "building",
                "game_end"
            ]
        );
        let kill = t.events.iter().find(|e| e.kind == "kill").unwrap();
        assert_eq!(kill.t_ms, 125_000);
        assert_eq!((kill.x, kill.y), (Some(9800), Some(4400)));
        assert_eq!(kill.participants, vec![7, 2, 8], "killer, victim, assists");
        assert_eq!(kill.meta["bounty"], 400);

        let dragon = t.events.iter().find(|e| e.kind == "objective").unwrap();
        assert_eq!(dragon.team_id, Some(100));
        assert_eq!(dragon.participants, vec![3, 1, 4]);
        assert_eq!(dragon.meta["monsterType"], "DRAGON");
        assert_eq!(dragon.meta["monsterSubType"], "FIRE_DRAGON");

        let ward = t.events.iter().find(|e| e.kind == "ward_placed").unwrap();
        assert_eq!((ward.x, ward.y), (None, None), "wards have no position");
        assert_eq!(ward.participants, vec![3]);

        let tower = t.events.iter().find(|e| e.kind == "building").unwrap();
        assert_eq!(tower.team_id, Some(100), "the team that lost the tower");
        assert_eq!(tower.meta["towerType"], "OUTER_TURRET");

        let end = t.events.last().unwrap();
        assert_eq!(end.team_id, Some(100), "winning team");
    }

    #[test]
    fn unknown_events_are_counted_not_fatal() {
        let t = imported();
        assert_eq!(
            t.skipped_event_types
                .get("MYSTERY_EVENT_FROM_A_FUTURE_PATCH"),
            Some(&1)
        );
        assert!(
            !t.skipped_event_types.contains_key("ITEM_PURCHASED"),
            "expected types are ignored silently"
        );
    }

    #[test]
    fn malformed_documents_are_rejected() {
        assert!(import_timeline_str("not json").is_err());
        assert!(matches!(
            import_timeline(&serde_json::json!({"info": {"frames": []}})),
            Err(ImportError::Shape(_))
        ));
        assert!(import_timeline(&serde_json::json!({"metadata": {"matchId": "X_1"}})).is_err());
        let empty = import_timeline(&serde_json::json!({
            "metadata": {"matchId": "X_1"},
            "info": {"frames": []}
        }))
        .unwrap();
        assert!(empty.frames.is_empty() && empty.participants.is_empty());
    }

    #[test]
    fn respawn_table_is_clamped() {
        assert_eq!(respawn_ms(0), 6000);
        assert_eq!(respawn_ms(3), 8000);
        assert_eq!(respawn_ms(30), 52_500);
    }
}
