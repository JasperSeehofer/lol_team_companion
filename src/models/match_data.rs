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
    pub items: [i32; 6], // item0-item5 (NOT item6/trinket)
    pub win: bool,
}

/// Prefix of the seed tool's fictional demo games (M141). No Riot platform
/// is called "SEED", so a demo id can never name a real game, and the match
/// page never sends one to Riot.
pub const DEMO_MATCH_PREFIX: &str = "SEED1_";

/// The seed tool's demo game for `seed-alpha-top`.
pub const DEMO_MATCH_ID: &str = "SEED1_1000000001";

/// Whether an id names a seed demo game (never looked up on Riot).
pub fn is_demo_match_id(match_id: &str) -> bool {
    match_id.starts_with(DEMO_MATCH_PREFIX)
}

/// Timeline event categories for UI filtering
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EventCategory {
    Objective, // dragon, baron, herald
    Tower,     // tower, inhibitor
    Kill,      // champion kills, first blood, multikills
    Ward,      // ward placements (user's own only)
    Teamfight, // 4+ participants within 10s window
    // Riot's Match-V5 timeline has no recall event. Since M149 a recall is
    // inferred from item purchases (`infer_recalls`, event type
    // `RECALL_INFERRED`) and always labelled as inferred; it is never stored.
    // (An ITEM_UNDO is the shop's undo, never a recall: M141.)
    Recall,
    // An item purchase (`ITEM_PURCHASED`, M149): stored for recall
    // inference, never drawn and never sent to the page (`viewer_timeline`).
    Purchase,
}

/// A point on Summoner's Rift in Riot's map units (about 0 to 15,000 on
/// both axes, blue fountain bottom left).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapPosition {
    pub x: i32,
    pub y: i32,
}

/// The importer version a cached game's timeline was written with
/// (`match_detail.timeline_version`; rows without one are version 1).
/// Version 2 (M149) keeps event positions, item purchases and ward
/// creators. Bump it when the importer keeps something new; the admin
/// re-import (`/admin/reimport`) then picks up every older game.
pub const TIMELINE_VERSION: i32 = 2;

/// Event type of an inferred recall (computed on read, never stored).
pub const RECALL_EVENT_TYPE: &str = "RECALL_INFERRED";

/// A single processed timeline event
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimelineEvent {
    pub timestamp_ms: i64,
    pub event_type: String, // "CHAMPION_KILL", "BUILDING_KILL", "ELITE_MONSTER_KILL", "WARD_PLACED", "TEAMFIGHT"
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
    /// The acting participant where Riot names one that is not a killer:
    /// the buyer of an `ITEM_PURCHASED`, the creator of a `WARD_PLACED`,
    /// the player of an inferred recall (M149). `None` on older imports.
    #[serde(default)]
    pub participant_id: Option<i32>,
    /// The item of an `ITEM_PURCHASED` (M149).
    #[serde(default)]
    pub item_id: Option<i32>,
    /// Where it happened, for the events Riot gives a position (champion
    /// kills, buildings, elite monsters; Match-V5 gives wards and purchases
    /// none). `None` on older imports (M149).
    #[serde(default)]
    pub position: Option<MapPosition>,
}

/// Item purchases of one player closer together than this belong to one
/// shop visit.
pub const RECALL_BURST_GAP_MS: i64 = 25_000;

/// Purchases in the first 90 seconds are the opening shop, not a recall.
pub const RECALL_OPENING_SHOP_MS: i64 = 90_000;

/// How long after a death a shop visit counts as the respawn's, not a
/// recall: 20 s plus 2 s per game minute, at most 90 s. That covers the
/// death timer (about 8 s at level 1 to about 79 s late in a long game)
/// plus the time to shop after respawning.
pub fn recall_death_window_ms(at_ms: i64) -> i64 {
    let minutes = at_ms.max(0) / 60_000;
    (20 + 2 * minutes).min(90) * 1000
}

/// Recalls inferred from item purchases (M149). Riot's Match-V5 timeline
/// has no recall event, so this is a guess, and everything built on it says
/// "inferred" (the event type [`RECALL_EVENT_TYPE`], the page's "Inferred
/// recall" text, "Recall (inferred)", a dashed outline).
///
/// The heuristic, per participant:
/// 1. Their `ITEM_PURCHASED` events, in time order, are grouped into shop
///    visits: a purchase within [`RECALL_BURST_GAP_MS`] (25 s) of the
///    previous one belongs to the same visit.
/// 2. A visit is a recall when it starts after the opening shop
///    ([`RECALL_OPENING_SHOP_MS`], 90 s) and the participant did not die
///    (a `CHAMPION_KILL` naming them as victim) in the
///    [`recall_death_window_ms`] before it starts: buying after a death is
///    the respawn, not a recall.
/// 3. The recall is placed at the visit's first purchase.
///
/// Not used: the participant frames' positions. They come once a minute,
/// and a champion walks from the fountain back to a lane in less than that,
/// so a "near the fountain at the nearest frame" test would drop real
/// recalls. Known misses: a recall without a purchase, and a visit that
/// follows a death closely. Purchases away from the fountain (rare
/// champion abilities) can show as a recall.
pub fn infer_recalls(events: &[TimelineEvent]) -> Vec<TimelineEvent> {
    use std::collections::BTreeMap;
    let mut buys: BTreeMap<i32, Vec<&TimelineEvent>> = BTreeMap::new();
    let mut deaths: BTreeMap<i32, Vec<i64>> = BTreeMap::new();
    for e in events {
        match (
            e.event_type.as_str(),
            e.participant_id,
            e.victim_participant_id,
        ) {
            ("ITEM_PURCHASED", Some(pid), _) => buys.entry(pid).or_default().push(e),
            ("CHAMPION_KILL", _, Some(victim)) => {
                deaths.entry(victim).or_default().push(e.timestamp_ms)
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    for (pid, mut list) in buys {
        list.sort_by_key(|e| e.timestamp_ms);
        let died = deaths.get(&pid).map(Vec::as_slice).unwrap_or(&[]);
        let mut last: Option<i64> = None;
        for e in list {
            let t = e.timestamp_ms;
            let new_visit = last.is_none_or(|l| t - l > RECALL_BURST_GAP_MS);
            last = Some(t);
            if !new_visit || t <= RECALL_OPENING_SHOP_MS {
                continue;
            }
            let window = recall_death_window_ms(t);
            if died.iter().any(|&d| d <= t && t - d <= window) {
                continue;
            }
            out.push(TimelineEvent {
                timestamp_ms: t,
                event_type: RECALL_EVENT_TYPE.to_string(),
                category: EventCategory::Recall,
                team_id: e.team_id,
                killer_participant_id: None,
                victim_participant_id: None,
                monster_type: None,
                monster_sub_type: None,
                building_type: None,
                is_first_blood: false,
                multi_kill_length: None,
                is_teamfight: false,
                involved_participants: vec![pid],
                participant_id: Some(pid),
                item_id: None,
                position: None,
            });
        }
    }
    out.sort_by_key(|e| (e.timestamp_ms, e.participant_id));
    out
}

/// The timeline the match page shows to the player `user_pid`, and whether
/// the stored timeline holds item purchases (imported since M149; without
/// them no recall can be inferred and the page says to re-import):
/// - inferred recalls of the viewer's team are added ([`infer_recalls`]);
/// - purchases are removed (they only feed the inference);
/// - wards are the viewer's own; a ward from an older import, which does
///   not name its creator, is kept.
pub fn viewer_timeline(
    events: Vec<TimelineEvent>,
    participants: &[MatchParticipant],
    user_pid: i32,
) -> (Vec<TimelineEvent>, bool) {
    let has_purchases = events.iter().any(|e| e.category == EventCategory::Purchase);
    let team = participants
        .iter()
        .find(|p| p.participant_id == user_pid)
        .map(|p| p.team_id);
    let recalls: Vec<TimelineEvent> = infer_recalls(&events)
        .into_iter()
        .filter(|r| match team {
            Some(t) => r.team_id == Some(t),
            None => r.participant_id == Some(user_pid),
        })
        .collect();
    let mut out: Vec<TimelineEvent> = events
        .into_iter()
        .filter(|e| match e.category {
            EventCategory::Purchase => false,
            EventCategory::Ward => e.participant_id.is_none_or(|p| p == user_pid),
            _ => true,
        })
        .collect();
    out.extend(recalls);
    out.sort_by_key(|e| e.timestamp_ms);
    (out, has_purchases)
}

/// What one admin re-import batch did (`/admin/reimport`, M149).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ReimportReport {
    /// Games whose timeline was replaced in this batch.
    pub done: u32,
    /// Games Riot has no data for any more (left as they were, not tried again).
    pub failed: u32,
    /// Games still waiting for a re-import.
    pub remaining: u32,
    /// Games marked as not re-importable (all batches so far).
    pub unavailable: u32,
    /// Why the batch stopped early (Riot busy, Riot down), if it did.
    pub stopped: Option<String>,
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
    pub game_duration: i32, // seconds
    pub game_mode: String,
    pub participants: Vec<MatchParticipant>,
    pub timeline_events: Vec<TimelineEvent>,
    pub user_participant_id: i32, // which participant is the current user
    pub user_puuid: String,
    pub performance: PerformanceStats,
    /// Whether the stored timeline holds item purchases, so recalls could
    /// be inferred (M149). `false` for games imported before.
    #[serde(default)]
    pub has_purchase_data: bool,
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
    pub goal_type: String, // "rank_target" | "cs_per_min" | "deaths_per_game"
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

#[cfg(test)]
mod recall_tests {
    use super::*;

    fn ev(ts: i64, kind: &str, cat: EventCategory) -> TimelineEvent {
        TimelineEvent {
            timestamp_ms: ts,
            event_type: kind.to_string(),
            category: cat,
            team_id: None,
            killer_participant_id: None,
            victim_participant_id: None,
            monster_type: None,
            monster_sub_type: None,
            building_type: None,
            is_first_blood: false,
            multi_kill_length: None,
            is_teamfight: false,
            involved_participants: Vec::new(),
            participant_id: None,
            item_id: None,
            position: None,
        }
    }

    fn buy(ts: i64, pid: i32) -> TimelineEvent {
        let mut e = ev(ts, "ITEM_PURCHASED", EventCategory::Purchase);
        e.participant_id = Some(pid);
        e.item_id = Some(1001);
        e.team_id = Some(if pid <= 5 { 100 } else { 200 });
        e
    }

    fn death(ts: i64, victim: i32) -> TimelineEvent {
        let mut e = ev(ts, "CHAMPION_KILL", EventCategory::Kill);
        e.victim_participant_id = Some(victim);
        e.killer_participant_id = Some(if victim <= 5 { 6 } else { 1 });
        e
    }

    fn participant(pid: i32) -> MatchParticipant {
        MatchParticipant {
            participant_id: pid,
            puuid: format!("p{pid}"),
            summoner_name: format!("P{pid}"),
            champion_name: "Ahri".into(),
            team_id: if pid <= 5 { 100 } else { 200 },
            team_position: String::new(),
            kills: 0,
            deaths: 0,
            assists: 0,
            cs: 0,
            vision_score: 0,
            damage: 0,
            gold_earned: 0,
            items: [0; 6],
            win: true,
        }
    }

    const MIN: i64 = 60_000;

    #[test]
    fn the_opening_shop_is_not_a_recall() {
        let r = infer_recalls(&[buy(0, 1), buy(5_000, 1), buy(80_000, 1)]);
        assert!(r.is_empty(), "{r:?}");
    }

    #[test]
    fn one_shop_visit_is_one_recall_at_its_first_purchase() {
        let events = [
            buy(7 * MIN, 1),
            buy(7 * MIN + 10_000, 1),
            buy(7 * MIN + 30_000, 1), // 20 s after the previous: same visit
            buy(12 * MIN, 1),         // a second visit
        ];
        let r = infer_recalls(&events);
        assert_eq!(r.len(), 2, "{r:?}");
        assert_eq!(r[0].timestamp_ms, 7 * MIN);
        assert_eq!(r[1].timestamp_ms, 12 * MIN);
        assert!(r.iter().all(|e| e.event_type == RECALL_EVENT_TYPE
            && e.category == EventCategory::Recall
            && e.participant_id == Some(1)
            && e.involved_participants == vec![1]
            && e.team_id == Some(100)));
    }

    #[test]
    fn a_purchase_more_than_25_seconds_later_is_a_new_visit() {
        let r = infer_recalls(&[buy(5 * MIN, 2), buy(5 * MIN + 26_000, 2)]);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn buying_after_a_death_is_the_respawn_not_a_recall() {
        // Died at 10:00; the window at 10:30 is 20 + 2 * 10 = 40 s.
        let events = [death(10 * MIN, 3), buy(10 * MIN + 30_000, 3)];
        assert!(infer_recalls(&events).is_empty());
        // 45 s after the death is outside the window: a recall.
        let events = [death(10 * MIN, 3), buy(10 * MIN + 45_000, 3)];
        assert_eq!(infer_recalls(&events).len(), 1);
        // Someone else's death does not matter.
        let events = [death(10 * MIN, 4), buy(10 * MIN + 30_000, 3)];
        assert_eq!(infer_recalls(&events).len(), 1);
    }

    #[test]
    fn the_death_window_grows_with_game_time_and_is_capped() {
        assert_eq!(recall_death_window_ms(0), 20_000);
        assert_eq!(recall_death_window_ms(10 * MIN), 40_000);
        assert_eq!(recall_death_window_ms(30 * MIN), 80_000);
        assert_eq!(recall_death_window_ms(60 * MIN), 90_000);
    }

    #[test]
    fn the_viewer_sees_their_teams_recalls_and_their_own_wards() {
        let mut my_ward = ev(4 * MIN, "WARD_PLACED", EventCategory::Ward);
        my_ward.participant_id = Some(1);
        let mut mate_ward = ev(4 * MIN, "WARD_PLACED", EventCategory::Ward);
        mate_ward.participant_id = Some(2);
        let old_ward = ev(5 * MIN, "WARD_PLACED", EventCategory::Ward);
        let events = vec![
            buy(0, 1),
            buy(6 * MIN, 1),
            buy(7 * MIN, 2),
            buy(8 * MIN, 7),
            my_ward,
            mate_ward,
            old_ward,
            death(9 * MIN, 7),
        ];
        let parts: Vec<MatchParticipant> = (1..=10).map(participant).collect();
        let (shown, has_purchases) = viewer_timeline(events, &parts, 1);
        assert!(has_purchases);
        assert!(shown.iter().all(|e| e.category != EventCategory::Purchase));
        let recalls: Vec<i32> = shown
            .iter()
            .filter(|e| e.category == EventCategory::Recall)
            .filter_map(|e| e.participant_id)
            .collect();
        assert_eq!(recalls, vec![1, 2], "own team only");
        let wards: Vec<Option<i32>> = shown
            .iter()
            .filter(|e| e.category == EventCategory::Ward)
            .map(|e| e.participant_id)
            .collect();
        assert_eq!(
            wards,
            vec![Some(1), None],
            "own ward and the old import's ward"
        );
        assert!(shown
            .windows(2)
            .all(|w| w[0].timestamp_ms <= w[1].timestamp_ms));
    }

    #[test]
    fn an_old_import_has_no_purchases_and_no_recalls() {
        let events = vec![death(9 * MIN, 7)];
        let parts: Vec<MatchParticipant> = (1..=10).map(participant).collect();
        let (shown, has_purchases) = viewer_timeline(events, &parts, 1);
        assert!(!has_purchases);
        assert_eq!(shown.len(), 1);
    }

    #[test]
    fn old_serialized_events_still_read() {
        // A TimelineEvent serialized before M149 has none of the new fields.
        let json = r#"{"timestamp_ms":1,"event_type":"CHAMPION_KILL","category":"Kill","team_id":100,
            "killer_participant_id":1,"victim_participant_id":6,"monster_type":null,
            "monster_sub_type":null,"building_type":null,"is_first_blood":true,
            "multi_kill_length":null,"is_teamfight":false,"involved_participants":[1,6]}"#;
        let e: TimelineEvent = serde_json::from_str(json).unwrap();
        assert_eq!(e.position, None);
        assert_eq!(e.participant_id, None);
    }
}
