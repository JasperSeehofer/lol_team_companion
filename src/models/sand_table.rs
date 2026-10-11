//! Sand Table storage models (M6: storage only, no UI yet).
//!
//! A replay is a team's imported copy of one match timeline: per-minute
//! frames with every participant's position and economy, plus positioned
//! events. A board is a team's tactics canvas, saved as numbered versions of
//! a scene document whose shape the Sand Table UI will define.

use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::types::SurrealValue;

/// Where a replay came from. Only Riot Match-V5 timelines so far.
pub const SOURCE_RIOT_TIMELINE: &str = "riot_timeline";

/// A participant of a replayed match.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ssr", derive(surrealdb_types_derive::SurrealValue))]
pub struct ReplayParticipant {
    /// 1-10 as in Riot's timeline (1-5 blue side, 6-10 red side).
    pub participant_id: i64,
    pub puuid: Option<String>,
    /// 100 (blue) or 200 (red).
    pub team_id: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Replay {
    pub id: Option<String>,
    pub team_id: String,
    /// Riot match ID, such as `EUW1_7100000001`.
    pub match_id: String,
    pub patch: Option<String>,
    pub imported_at: Option<String>,
    pub source: String,
    pub frame_interval_ms: i64,
    pub participants: Vec<ReplayParticipant>,
}

/// One participant in one frame.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ssr", derive(surrealdb_types_derive::SurrealValue))]
pub struct ParticipantFrame {
    pub participant_id: i64,
    /// Map coordinates (Summoner's Rift is roughly 0..15000 on both axes).
    pub x: i64,
    pub y: i64,
    /// Total gold earned so far.
    pub gold: i64,
    /// Unspent gold.
    pub current_gold: i64,
    pub xp: i64,
    /// Lane minions plus jungle monsters.
    pub cs: i64,
    pub level: i64,
    /// Estimated from kill events and base respawn times; the timeline does
    /// not report it.
    pub alive: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(surrealdb_types_derive::SurrealValue))]
pub struct ReplayFrame {
    pub t_ms: i64,
    pub participants: Vec<ParticipantFrame>,
}

/// Event kinds the importer keeps.
pub mod event_kind {
    pub const KILL: &str = "kill";
    pub const SPECIAL_KILL: &str = "special_kill";
    pub const BUILDING: &str = "building";
    pub const TURRET_PLATE: &str = "turret_plate";
    pub const OBJECTIVE: &str = "objective";
    pub const DRAGON_SOUL: &str = "dragon_soul";
    pub const WARD_PLACED: &str = "ward_placed";
    pub const WARD_KILL: &str = "ward_kill";
    pub const LEVEL_UP: &str = "level_up";
    pub const GAME_END: &str = "game_end";
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(surrealdb_types_derive::SurrealValue))]
pub struct ReplayEvent {
    /// Position in the replay's event order (events can share a timestamp).
    pub seq: i64,
    pub t_ms: i64,
    /// One of [`event_kind`].
    pub kind: String,
    /// Position, when the timeline gives one (wards and level-ups have none).
    pub x: Option<i64>,
    pub y: Option<i64>,
    /// Participants involved. For kills: killer, victim, then assists. For
    /// objectives and buildings: killer, then assists. Otherwise the actor.
    /// A killer of 0 (minions, turrets) is left out.
    pub participants: Vec<i64>,
    /// The team the event belongs to: the killing team for objectives, the
    /// team that lost the structure for buildings and plates, the winner for
    /// game end.
    pub team_id: Option<i64>,
    /// Remaining type-specific fields (monster type, lane, ward type, ...).
    pub meta: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Board {
    pub id: Option<String>,
    pub team_id: String,
    pub name: String,
    pub created_by: String,
    pub updated_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BoardVersion {
    pub board_id: String,
    pub version: i64,
    pub scene: serde_json::Value,
    pub author: String,
    pub created_at: Option<String>,
}
