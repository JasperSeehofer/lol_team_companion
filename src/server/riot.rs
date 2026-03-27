use riven::consts::{PlatformRoute, RegionalRoute};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RiotError {
    #[error("Riot API error: {0}")]
    Api(String),
    #[error("riven error: {0}")]
    Riven(#[from] riven::RiotApiError),
}

pub struct MatchData {
    pub match_id: String,
    pub queue_id: i32,
    pub game_duration: i32,
    pub game_end_epoch_ms: Option<i64>,
    pub champion: String,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub cs: i32,
    pub vision_score: i32,
    pub damage: i32,
    pub win: bool,
}

pub fn has_api_key() -> bool {
    let key = std::env::var("RIOT_API_KEY").unwrap_or_default();
    !key.is_empty()
}

fn api() -> riven::RiotApi {
    let key = std::env::var("RIOT_API_KEY").unwrap_or_default();
    riven::RiotApi::new(key)
}

pub fn platform_route_from_str(region: &str) -> PlatformRoute {
    match region {
        "EUW" => PlatformRoute::EUW1,
        "EUNE" => PlatformRoute::EUN1,
        "NA" => PlatformRoute::NA1,
        "KR" => PlatformRoute::KR,
        "BR" => PlatformRoute::BR1,
        "LAN" => PlatformRoute::LA1,
        "LAS" => PlatformRoute::LA2,
        "OCE" => PlatformRoute::OC1,
        "TR" => PlatformRoute::TR1,
        "RU" => PlatformRoute::RU,
        "JP" => PlatformRoute::JP1,
        "SG" => PlatformRoute::SG2,
        "TW" => PlatformRoute::TW2,
        "VN" => PlatformRoute::VN2,
        "ME" => PlatformRoute::ME1,
        _ => PlatformRoute::EUW1, // safe fallback
    }
}

/// account-v1 uses different regional grouping than match-v5
pub fn account_region_for(platform: PlatformRoute) -> RegionalRoute {
    match platform {
        PlatformRoute::OC1
        | PlatformRoute::SG2
        | PlatformRoute::TW2
        | PlatformRoute::VN2
        | PlatformRoute::ME1 => RegionalRoute::SEA,
        PlatformRoute::NA1 | PlatformRoute::BR1 | PlatformRoute::LA1 | PlatformRoute::LA2 => {
            RegionalRoute::AMERICAS
        }
        PlatformRoute::KR | PlatformRoute::JP1 => RegionalRoute::ASIA,
        _ => RegionalRoute::EUROPE,
    }
}

pub async fn get_puuid(
    game_name: &str,
    tag_line: &str,
    platform: PlatformRoute,
) -> Result<String, RiotError> {
    let api = api();
    let account = api
        .account_v1()
        .get_by_riot_id(account_region_for(platform), game_name, tag_line)
        .await?
        .ok_or_else(|| RiotError::Api(format!("Account {game_name}#{tag_line} not found")))?;
    Ok(account.puuid)
}

pub async fn fetch_match_history(
    puuid: &str,
    queue_id: Option<i32>,
    platform: PlatformRoute,
) -> Result<Vec<MatchData>, RiotError> {
    let api = api();

    let queue_filter = queue_id.map(|q| riven::consts::Queue::from(q as u16));

    let match_ids = api
        .match_v5()
        .get_match_ids_by_puuid(
            platform.to_regional(),
            puuid,
            Some(20),
            None,
            queue_filter,
            None,
            None,
            None,
        )
        .await?;

    let mut results = Vec::new();

    for mid in match_ids {
        let Some(m) = api
            .match_v5()
            .get_match(platform.to_regional(), &mid)
            .await?
        else {
            continue;
        };

        let participant = m.info.participants.iter().find(|p| p.puuid == puuid);

        let Some(p) = participant else { continue };

        results.push(MatchData {
            match_id: mid,
            queue_id: u16::from(m.info.queue_id) as i32,
            game_duration: m.info.game_duration as i32,
            game_end_epoch_ms: m.info.game_end_timestamp,
            champion: p.champion_name.clone(),
            kills: p.kills,
            deaths: p.deaths,
            assists: p.assists,
            cs: p.total_minions_killed + p.neutral_minions_killed,
            vision_score: p.vision_score,
            damage: p.total_damage_dealt_to_champions,
            win: p.win,
        });
    }

    Ok(results)
}

/// Fetch champion mastery data for a player by PUUID.
///
/// Returns a list of (champion_name, mastery_level, mastery_points) tuples,
/// ordered by mastery points descending (highest mastery first).
///
/// Returns an empty Vec on API error — callers should degrade gracefully.
pub async fn fetch_champion_masteries(
    puuid: &str,
    platform: PlatformRoute,
) -> Result<Vec<(String, i32, i32)>, RiotError> {
    let api = api();
    let masteries = api
        .champion_mastery_v4()
        .get_all_champion_masteries_by_puuid(platform, puuid)
        .await?;
    let result = masteries
        .into_iter()
        .filter_map(|m| {
            // Use the identifier (Data Dragon canonical name) for consistency.
            // Fall back to the display name if identifier is unavailable.
            let name = m
                .champion_id
                .identifier()
                .or_else(|| m.champion_id.name())
                .map(|s| s.to_string())?;
            Some((name, m.champion_level, m.champion_points))
        })
        .collect();
    Ok(result)
}

pub async fn fetch_player_champions(
    puuid: &str,
    count: usize,
    platform: PlatformRoute,
) -> Result<Vec<String>, RiotError> {
    let api = api();
    let match_ids = api
        .match_v5()
        .get_match_ids_by_puuid(
            platform.to_regional(),
            puuid,
            Some(count as i32),
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

    let mut champions = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for mid in match_ids {
        let Some(m) = api
            .match_v5()
            .get_match(platform.to_regional(), &mid)
            .await?
        else {
            continue;
        };
        if let Some(p) = m.info.participants.iter().find(|p| p.puuid == puuid) {
            if seen.insert(p.champion_name.clone()) {
                champions.push(p.champion_name.clone());
            }
        }
    }

    Ok(champions)
}

/// Combined intel fetch: champion names, per-match role data, and mastery data in a single call.
///
/// - `recent_champions`: unique champion names played (same dedup as `fetch_player_champions`)
/// - `champion_with_role`: (champion_name, team_position) per match — NOT deduplicated, used for role distribution
/// - `mastery_data`: (champion_name, mastery_level, mastery_points) from champion mastery endpoint
pub struct PlayerIntelData {
    pub recent_champions: Vec<String>,
    pub champion_with_role: Vec<(String, String)>,
    pub mastery_data: Vec<(String, i32, i32)>,
}

pub async fn fetch_player_intel(
    puuid: &str,
    match_count: usize,
    platform: PlatformRoute,
) -> Result<PlayerIntelData, RiotError> {
    let api = api();
    let match_ids = api
        .match_v5()
        .get_match_ids_by_puuid(
            platform.to_regional(),
            puuid,
            Some(match_count as i32),
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

    let mut recent_champions = Vec::new();
    let mut champion_with_role: Vec<(String, String)> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for mid in match_ids {
        let Some(m) = api
            .match_v5()
            .get_match(platform.to_regional(), &mid)
            .await?
        else {
            continue;
        };
        if let Some(p) = m.info.participants.iter().find(|p| p.puuid == puuid) {
            let champion_name = p.champion_name.clone();
            let team_position = p.team_position.clone();
            // Track per-match role data (not deduplicated)
            champion_with_role.push((champion_name.clone(), team_position));
            // Track unique champions for recent_champions
            if seen.insert(champion_name.clone()) {
                recent_champions.push(champion_name);
            }
        }
    }

    let mastery_data = fetch_champion_masteries(puuid, platform).await?;

    Ok(PlayerIntelData {
        recent_champions,
        champion_with_role,
        mastery_data,
    })
}

pub struct RankedEntry {
    pub queue_type: String,
    pub tier: String,
    pub division: String,
    pub lp: i32,
    pub wins: i32,
    pub losses: i32,
}

pub async fn fetch_ranked_data(
    puuid: &str,
    platform: PlatformRoute,
) -> Result<Vec<RankedEntry>, RiotError> {
    let api = api();
    let entries = api
        .league_v4()
        .get_league_entries_by_puuid(platform, puuid)
        .await?;
    Ok(entries
        .into_iter()
        .map(|e| RankedEntry {
            queue_type: format!("{:?}", e.queue_type),
            tier: e.tier.map(|t| format!("{:?}", t)).unwrap_or_default(),
            division: e.rank.map(|d| format!("{:?}", d)).unwrap_or_default(),
            lp: e.league_points,
            wins: e.wins,
            losses: e.losses,
        })
        .collect())
}

use crate::models::match_data::{
    EventCategory, MatchDetail, MatchParticipant, PerformanceStats, TimelineEvent,
};

pub struct FullMatchData {
    pub match_id: String,
    pub game_duration: i32,
    pub game_mode: String,
    pub participants: Vec<MatchParticipant>,
    pub raw_timeline_events: Vec<TimelineEvent>,
}

pub async fn fetch_full_match_detail(
    match_id: &str,
    platform: PlatformRoute,
) -> Result<FullMatchData, RiotError> {
    let api = api();
    let regional = platform.to_regional();

    // Fetch match data (all 10 participants)
    let m = api
        .match_v5()
        .get_match(regional, match_id)
        .await?
        .ok_or_else(|| RiotError::Api(format!("Match {match_id} not found")))?;

    let game_duration = m.info.game_duration as i32;
    let game_mode = format!("{:?}", m.info.game_mode);

    let participants: Vec<MatchParticipant> = m
        .info
        .participants
        .iter()
        .map(|p| MatchParticipant {
            participant_id: p.participant_id,
            puuid: p.puuid.clone(),
            summoner_name: p.summoner_name.clone(),
            champion_name: p.champion_name.clone(),
            team_id: u16::from(p.team_id) as i32,
            team_position: p.team_position.clone(),
            kills: p.kills,
            deaths: p.deaths,
            assists: p.assists,
            cs: p.total_minions_killed + p.neutral_minions_killed,
            vision_score: p.vision_score,
            damage: p.total_damage_dealt_to_champions,
            gold_earned: p.gold_earned,
            items: [p.item0, p.item1, p.item2, p.item3, p.item4, p.item5],
            win: p.win,
        })
        .collect();

    // Fetch timeline
    let timeline_opt = api.match_v5().get_timeline(regional, match_id).await?;
    let mut raw_events = Vec::new();

    if let Some(timeline) = timeline_opt {
        for frame in &timeline.info.frames {
            for event in &frame.events {
                let event_type = event.r#type.clone();
                let category = classify_event(
                    &event_type,
                    event.monster_type.as_deref(),
                    event.building_type.as_deref(),
                );
                let Some(category) = category else { continue };

                let mut involved = Vec::new();
                if let Some(kid) = event.killer_id {
                    involved.push(kid);
                }
                if let Some(vid) = event.victim_id {
                    involved.push(vid);
                }
                if let Some(ref assists) = event.assisting_participant_ids {
                    involved.extend(assists.iter().copied());
                }

                raw_events.push(TimelineEvent {
                    timestamp_ms: event.timestamp,
                    event_type,
                    category,
                    team_id: event
                        .team_id
                        .map(|t| u16::from(t) as i32)
                        .or_else(|| event.killer_team_id.map(|t| u16::from(t) as i32)),
                    killer_participant_id: event.killer_id,
                    victim_participant_id: event.victim_id,
                    monster_type: event.monster_type.clone(),
                    monster_sub_type: event.monster_sub_type.clone(),
                    building_type: event.building_type.clone(),
                    is_first_blood: false, // set below
                    multi_kill_length: event.multi_kill_length,
                    is_teamfight: false, // set below
                    involved_participants: involved,
                });
            }
        }
    }

    // Mark first blood
    if let Some(first_kill) = raw_events
        .iter_mut()
        .find(|e| e.event_type == "CHAMPION_KILL")
    {
        first_kill.is_first_blood = true;
    }

    // Detect teamfights: sliding 10s window, 4+ unique participant IDs
    let kill_events: Vec<(usize, i64, Vec<i32>)> = raw_events
        .iter()
        .enumerate()
        .filter(|(_, e)| e.event_type == "CHAMPION_KILL")
        .map(|(i, e)| (i, e.timestamp_ms, e.involved_participants.clone()))
        .collect();

    let mut teamfight_indices: std::collections::HashSet<usize> =
        std::collections::HashSet::new();
    for (i, (idx, ts, _)) in kill_events.iter().enumerate() {
        let mut unique_ids: std::collections::HashSet<i32> = std::collections::HashSet::new();
        let mut window_indices = vec![*idx];
        for (jdx, jts, jparticipants) in &kill_events[i..] {
            if *jts - *ts > 10_000 {
                break;
            }
            for pid in jparticipants {
                unique_ids.insert(*pid);
            }
            window_indices.push(*jdx);
        }
        if unique_ids.len() >= 4 {
            for idx in &window_indices {
                teamfight_indices.insert(*idx);
            }
        }
    }
    for idx in &teamfight_indices {
        if let Some(e) = raw_events.get_mut(*idx) {
            e.is_teamfight = true;
        }
    }

    // Group adjacent teamfight kills into clusters and create a single TEAMFIGHT event per cluster
    let mut teamfight_clusters: Vec<Vec<usize>> = Vec::new();
    let mut sorted_tf_indices: Vec<usize> = teamfight_indices.into_iter().collect();
    sorted_tf_indices.sort();
    let mut current_cluster: Vec<usize> = Vec::new();
    for idx in sorted_tf_indices {
        if let Some(last) = current_cluster.last() {
            let last_ts = raw_events[*last].timestamp_ms;
            let cur_ts = raw_events[idx].timestamp_ms;
            if cur_ts - last_ts > 10_000 {
                if !current_cluster.is_empty() {
                    teamfight_clusters.push(current_cluster.clone());
                }
                current_cluster = vec![idx];
            } else {
                current_cluster.push(idx);
            }
        } else {
            current_cluster.push(idx);
        }
    }
    if !current_cluster.is_empty() {
        teamfight_clusters.push(current_cluster);
    }

    for cluster in teamfight_clusters {
        if cluster.is_empty() {
            continue;
        }
        let center_ts = raw_events[cluster[cluster.len() / 2]].timestamp_ms;
        let mut all_involved: Vec<i32> = cluster
            .iter()
            .flat_map(|i| raw_events[*i].involved_participants.clone())
            .collect();
        all_involved.sort();
        all_involved.dedup();
        raw_events.push(TimelineEvent {
            timestamp_ms: center_ts,
            event_type: "TEAMFIGHT".to_string(),
            category: EventCategory::Teamfight,
            team_id: None,
            killer_participant_id: None,
            victim_participant_id: None,
            monster_type: None,
            monster_sub_type: None,
            building_type: None,
            is_first_blood: false,
            multi_kill_length: None,
            is_teamfight: true,
            involved_participants: all_involved,
        });
    }

    raw_events.sort_by_key(|e| e.timestamp_ms);

    Ok(FullMatchData {
        match_id: match_id.to_string(),
        game_duration,
        game_mode,
        participants,
        raw_timeline_events: raw_events,
    })
}

fn classify_event(
    event_type: &str,
    _monster_type: Option<&str>,
    _building_type: Option<&str>,
) -> Option<EventCategory> {
    match event_type {
        "ELITE_MONSTER_KILL" => Some(EventCategory::Objective),
        "BUILDING_KILL" => Some(EventCategory::Tower),
        "CHAMPION_KILL" => Some(EventCategory::Kill),
        "WARD_PLACED" => Some(EventCategory::Ward),
        "ITEM_UNDO" => Some(EventCategory::Recall), // Riot API uses ITEM_UNDO for recalls (per D-07)
        _ => None, // skip GAME_END, ITEM_PURCHASED, etc.
    }
}

pub fn compute_performance(
    participants: &[MatchParticipant],
    user_participant_id: i32,
    game_duration_secs: i32,
) -> PerformanceStats {
    let user = participants
        .iter()
        .find(|p| p.participant_id == user_participant_id)
        .expect("user participant must exist");
    let total_damage: i32 = participants.iter().map(|p| p.damage).sum();
    let damage_share_pct = if total_damage > 0 {
        (user.damage as f32 / total_damage as f32) * 100.0
    } else {
        0.0
    };
    let game_min = game_duration_secs as f32 / 60.0;
    let cs_per_min = if game_min > 0.0 {
        user.cs as f32 / game_min
    } else {
        0.0
    };
    let avg_cs_per_min = if game_min > 0.0 {
        participants.iter().map(|p| p.cs as f32).sum::<f32>() / 10.0 / game_min
    } else {
        0.0
    };
    let avg_vision = participants
        .iter()
        .map(|p| p.vision_score as f32)
        .sum::<f32>()
        / 10.0;
    let avg_gold = participants
        .iter()
        .map(|p| p.gold_earned as f32)
        .sum::<f32>()
        / 10.0;

    // Lane opponent: same position, opposite team
    let lane_opponent = if !user.team_position.is_empty() {
        participants.iter().find(|p| {
            p.team_id != user.team_id && p.team_position == user.team_position
        })
    } else {
        None
    };

    PerformanceStats {
        damage_share_pct,
        vision_score: user.vision_score,
        vision_score_avg: avg_vision,
        cs_per_min,
        cs_per_min_avg: avg_cs_per_min,
        gold_earned: user.gold_earned,
        gold_earned_avg: avg_gold,
        lane_opponent_damage: lane_opponent.map(|o| o.damage),
        lane_opponent_vision: lane_opponent.map(|o| o.vision_score),
        lane_opponent_cs_per_min: lane_opponent.map(|o| {
            if game_min > 0.0 {
                o.cs as f32 / game_min
            } else {
                0.0
            }
        }),
        lane_opponent_gold: lane_opponent.map(|o| o.gold_earned),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riven::consts::{PlatformRoute, RegionalRoute};

    #[test]
    fn platform_route_from_str_all_regions() {
        assert_eq!(platform_route_from_str("EUW"), PlatformRoute::EUW1);
        assert_eq!(platform_route_from_str("EUNE"), PlatformRoute::EUN1);
        assert_eq!(platform_route_from_str("NA"), PlatformRoute::NA1);
        assert_eq!(platform_route_from_str("KR"), PlatformRoute::KR);
        assert_eq!(platform_route_from_str("BR"), PlatformRoute::BR1);
        assert_eq!(platform_route_from_str("LAN"), PlatformRoute::LA1);
        assert_eq!(platform_route_from_str("LAS"), PlatformRoute::LA2);
        assert_eq!(platform_route_from_str("OCE"), PlatformRoute::OC1);
        assert_eq!(platform_route_from_str("TR"), PlatformRoute::TR1);
        assert_eq!(platform_route_from_str("RU"), PlatformRoute::RU);
        assert_eq!(platform_route_from_str("JP"), PlatformRoute::JP1);
        assert_eq!(platform_route_from_str("SG"), PlatformRoute::SG2);
        assert_eq!(platform_route_from_str("TW"), PlatformRoute::TW2);
        assert_eq!(platform_route_from_str("VN"), PlatformRoute::VN2);
        assert_eq!(platform_route_from_str("ME"), PlatformRoute::ME1);
        assert_eq!(platform_route_from_str("unknown"), PlatformRoute::EUW1);
        assert_eq!(platform_route_from_str(""), PlatformRoute::EUW1);
    }

    #[test]
    fn account_region_mapping() {
        // SEA group
        assert_eq!(account_region_for(PlatformRoute::OC1), RegionalRoute::SEA);
        assert_eq!(account_region_for(PlatformRoute::SG2), RegionalRoute::SEA);
        assert_eq!(account_region_for(PlatformRoute::TW2), RegionalRoute::SEA);
        assert_eq!(account_region_for(PlatformRoute::VN2), RegionalRoute::SEA);
        assert_eq!(account_region_for(PlatformRoute::ME1), RegionalRoute::SEA);
        // AMERICAS group
        assert_eq!(
            account_region_for(PlatformRoute::NA1),
            RegionalRoute::AMERICAS
        );
        assert_eq!(
            account_region_for(PlatformRoute::BR1),
            RegionalRoute::AMERICAS
        );
        assert_eq!(
            account_region_for(PlatformRoute::LA1),
            RegionalRoute::AMERICAS
        );
        assert_eq!(
            account_region_for(PlatformRoute::LA2),
            RegionalRoute::AMERICAS
        );
        // ASIA group
        assert_eq!(account_region_for(PlatformRoute::KR), RegionalRoute::ASIA);
        assert_eq!(account_region_for(PlatformRoute::JP1), RegionalRoute::ASIA);
        // EUROPE group
        assert_eq!(
            account_region_for(PlatformRoute::EUW1),
            RegionalRoute::EUROPE
        );
        assert_eq!(
            account_region_for(PlatformRoute::EUN1),
            RegionalRoute::EUROPE
        );
        assert_eq!(
            account_region_for(PlatformRoute::TR1),
            RegionalRoute::EUROPE
        );
        assert_eq!(account_region_for(PlatformRoute::RU), RegionalRoute::EUROPE);
    }
}
