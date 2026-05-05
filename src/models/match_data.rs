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

// ---------------------------------------------------------------------------
// Phase 15: Goals & LP History — shared model structs
// ---------------------------------------------------------------------------

/// A ranked snapshot data point for the LP history graph (D-03).
/// `rank_score` is computed server-side from `tier`/`division`/`lp`; not stored in DB.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RankedSnapshot {
    pub id: Option<String>,
    pub tier: String,
    pub division: String,
    pub lp: i32,
    pub snapshotted_at: String, // ISO datetime string
    pub rank_score: i32,        // computed via rank_score(); not stored in DB
}

/// A user's personal improvement goal (D-08).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PersonalGoal {
    pub id: Option<String>,
    pub goal_type: String,    // "rank_target" | "cs_per_min" | "deaths_per_game"
    pub target_value: String, // "DIAMOND:IV" | "7.5" | "4" (D-09 encoding)
}

/// Progress for a single goal type (D-13).
/// `current_value` is `None` when fewer than 5 solo/duo games are available (D-15).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GoalProgress {
    pub goal: PersonalGoal,
    pub current_value: Option<f32>, // None = insufficient data (< 5 games)
    pub game_count: i32,            // games used for average (max 20, D-12)
    pub achieved: bool,
}

/// Combined payload from `compute_goal_progress` server function (D-13).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GoalProgressPayload {
    pub rank: Option<GoalProgress>,
    pub cs: Option<GoalProgress>,
    pub deaths: Option<GoalProgress>,
    pub current_rank: Option<crate::models::user::RankedInfo>,
}

/// Per-champion aggregated stats for the champion trends table (D-16).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChampionTrend {
    pub champion: String,
    pub games: i32,
    pub wins: i32,
    pub avg_kda: f32,    // (kills + assists) / max(deaths, 1) per game, averaged
    pub cs_per_min: f32, // total_cs / (total_game_duration_sec / 60.0) — seconds-based
    pub avg_damage: i32, // total_damage / games
}

/// Convert tier/division/lp to a single continuous integer scale (D-03).
///
/// Iron 4 LP 0 = 0; each division adds 100; each tier adds 400.
/// Master, Grandmaster, and Challenger share a single continuation scale: 2800 + raw lp.
/// Diamond 1 99 LP = 2799; Master 0 LP = 2800 (smooth boundary, no discontinuity).
pub fn rank_score(tier: &str, division: &str, lp: i32) -> i32 {
    let tier_idx = match tier.to_uppercase().as_str() {
        "IRON" => 0,
        "BRONZE" => 1,
        "SILVER" => 2,
        "GOLD" => 3,
        "PLATINUM" => 4,
        "EMERALD" => 5,
        "DIAMOND" => 6,
        "MASTER" | "GRANDMASTER" | "CHALLENGER" => 7,
        _ => 0,
    };
    if tier_idx == 7 {
        return 2800 + lp;
    }
    let div_idx = match division.to_uppercase().as_str() {
        "IV" => 0,
        "III" => 1,
        "II" => 2,
        "I" => 3,
        _ => 0,
    };
    tier_idx * 400 + div_idx * 100 + lp
}

#[cfg(test)]
mod phase15_tests {
    use super::*;

    #[test]
    fn rank_score_gold2_47lp() {
        // tier_idx(GOLD)=3, div_idx(II)=2: 3*400 + 2*100 + 47 = 1200 + 200 + 47 = 1447
        // Note: plan said 1547 but that's an arithmetic error in the plan spec.
        assert_eq!(rank_score("GOLD", "II", 47), 1447);
    }

    #[test]
    fn rank_score_master_300lp() {
        // Master+ continuation: 2800 + 300 = 3100
        assert_eq!(rank_score("MASTER", "", 300), 3100);
    }

    #[test]
    fn rank_score_grandmaster_250_eq_challenger_250() {
        // GM and Challenger share the same scale — no extra tier offset
        assert_eq!(rank_score("GRANDMASTER", "", 250), 3050);
        assert_eq!(rank_score("CHALLENGER", "", 250), 3050);
    }

    #[test]
    fn rank_score_iron4_0lp_is_origin() {
        assert_eq!(rank_score("IRON", "IV", 0), 0);
    }

    #[test]
    fn rank_score_diamond_one_99lp() {
        // 6*400 + 3*100 + 99 = 2400 + 300 + 99 = 2799
        assert_eq!(rank_score("DIAMOND", "I", 99), 2799);
    }

    #[test]
    fn rank_score_continuity_diamond_to_master() {
        // Master 0 LP must be 2800 (one above Diamond I 99 LP = 2799)
        assert_eq!(rank_score("MASTER", "", 0), 2800);
    }

    #[test]
    fn rank_score_platinum_iii_0lp() {
        // 4*400 + 1*100 + 0 = 1600 + 100 = 1700
        assert_eq!(rank_score("PLATINUM", "III", 0), 1700);
    }

    #[test]
    fn rank_score_lowercase_input_normalises() {
        // Input normalised to uppercase internally — same result as uppercase form
        assert_eq!(rank_score("gold", "ii", 47), 1447);
    }
}
