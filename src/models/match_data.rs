use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::types::SurrealValue;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MatchSummary {
    pub id: Option<String>,
    pub match_id: String,
    pub queue_id: i32,
    pub game_duration: i32,
    pub team_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(surrealdb_types_derive::SurrealValue))]
pub struct PlayerMatchStats {
    pub id: Option<String>,
    pub match_id: String,
    pub user_id: String,
    pub champion: String,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub cs: i32,
    pub vision_score: i32,
    pub damage: i32,
    pub win: bool,
}

/// Single participant in a match (all 10 players)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MatchParticipant {
    pub participant_id: i32,
    pub puuid: String,
    pub summoner_name: String,
    pub champion_name: String,
    pub team_id: i32,          // 100 = Blue, 200 = Red
    pub team_position: String, // "TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY", or ""
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub cs: i32,
    pub vision_score: i32,
    pub damage: i32,
    pub gold_earned: i32,
    pub items: [i32; 6],      // item0-item5 (NOT item6/trinket)
    pub win: bool,
}

/// Timeline event categories for UI filtering
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EventCategory {
    Objective,  // dragon, baron, herald
    Tower,      // tower, inhibitor
    Kill,       // champion kills, first blood, multikills
    Ward,       // ward placements (user's own only)
    Teamfight,  // 4+ participants within 10s window
    Recall,     // champion recalls (per D-07)
}

/// A single processed timeline event
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimelineEvent {
    pub timestamp_ms: i64,
    pub event_type: String,     // "CHAMPION_KILL", "BUILDING_KILL", "ELITE_MONSTER_KILL", "WARD_PLACED", "ITEM_UNDO", "TEAMFIGHT"
    pub category: EventCategory,
    pub team_id: Option<i32>,
    pub killer_participant_id: Option<i32>,
    pub victim_participant_id: Option<i32>,
    pub monster_type: Option<String>,
    pub monster_sub_type: Option<String>,
    pub building_type: Option<String>,
    pub is_first_blood: bool,
    pub multi_kill_length: Option<i32>,
    pub is_teamfight: bool,
    pub involved_participants: Vec<i32>,
}

/// User's personal performance compared to game averages
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PerformanceStats {
    pub damage_share_pct: f32,
    pub vision_score: i32,
    pub vision_score_avg: f32,
    pub cs_per_min: f32,
    pub cs_per_min_avg: f32,
    pub gold_earned: i32,
    pub gold_earned_avg: f32,
    // Lane opponent values (None if role detection failed)
    pub lane_opponent_damage: Option<i32>,
    pub lane_opponent_vision: Option<i32>,
    pub lane_opponent_cs_per_min: Option<f32>,
    pub lane_opponent_gold: Option<i32>,
}

/// Complete match detail returned by the server function
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MatchDetail {
    pub match_id: String,
    pub game_duration: i32,     // seconds
    pub game_mode: String,
    pub participants: Vec<MatchParticipant>,
    pub timeline_events: Vec<TimelineEvent>,
    pub user_participant_id: i32,  // which participant is the current user
    pub user_puuid: String,
    pub performance: PerformanceStats,
}

/// Comparison mode for performance section
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ComparisonMode {
    GameAverage,
    LaneOpponent,
}
