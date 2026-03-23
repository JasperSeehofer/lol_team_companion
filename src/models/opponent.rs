use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Opponent {
    pub id: Option<String>,
    pub name: String,
    pub team_id: String,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpponentPlayer {
    pub id: Option<String>,
    pub opponent_id: String,
    pub name: String,
    pub role: String,
    pub riot_puuid: Option<String>,
    pub riot_summoner_name: Option<String>,
    pub recent_champions: Vec<String>,
    pub notes: Option<String>,
    /// ISO datetime string, set when Riot API data was last fetched for this player.
    #[serde(default)]
    pub last_fetched: Option<String>,
    /// JSON-encoded Vec<(String, i32, i32)>: (champion_name, mastery_level, mastery_points)
    #[serde(default)]
    pub mastery_data_json: Option<String>,
    /// JSON-encoded Vec<(String, u32)>: (role_name, count)
    #[serde(default)]
    pub role_distribution_json: Option<String>,
}

/// Enriched opponent player data with champion frequency counts, OTP detection,
/// and Riot API champion mastery data.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpponentPlayerIntel {
    pub player: OpponentPlayer,
    /// Champion pick frequencies from recent_champions, sorted descending by count.
    pub champion_frequencies: Vec<(String, u32)>,
    /// Top mastery champions from Riot API: (champion_name, mastery_level, mastery_points).
    /// Empty when API key is missing or player has no linked Riot account.
    pub mastery_data: Vec<(String, i32, i32)>,
    /// Champion name if the player is a one-trick-pony (one champion >60% of scouted games).
    pub otp_champion: Option<String>,
}

impl OpponentPlayer {
    /// Compute enriched intel from the player's stored data.
    ///
    /// OTP detection threshold: strictly > 60% (not >= 60%).
    pub fn compute_intel(&self) -> OpponentPlayerIntel {
        let total = self.recent_champions.len() as u32;

        // Count champion occurrences
        let mut counts: HashMap<String, u32> = HashMap::new();
        for champ in &self.recent_champions {
            *counts.entry(champ.clone()).or_insert(0) += 1;
        }

        // Sort descending by count
        let mut champion_frequencies: Vec<(String, u32)> = counts.into_iter().collect();
        champion_frequencies.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        // OTP: top champion strictly > 60% of games
        let otp_champion = if total > 0 {
            champion_frequencies
                .first()
                .and_then(|(name, count)| {
                    if *count as f64 / total as f64 > 0.6 {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
        } else {
            None
        };

        let mastery_data = self.mastery_data();

        OpponentPlayerIntel {
            player: self.clone(),
            champion_frequencies,
            mastery_data,
            otp_champion,
        }
    }

    /// Deserialize role_distribution_json into Vec<(role, count)>.
    /// Returns empty vec on None or parse failure.
    pub fn role_distribution(&self) -> Vec<(String, u32)> {
        self.role_distribution_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    /// Deserialize mastery_data_json into Vec<(champion_name, mastery_level, mastery_points)>.
    /// Returns empty vec on None or parse failure.
    pub fn mastery_data(&self) -> Vec<(String, i32, i32)> {
        self.mastery_data_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    /// Top 3 champions by frequency with percentage of total games.
    /// Returns Vec<(champion_name, percentage)> where percentage = count/total*100.
    pub fn comfort_picks(&self) -> Vec<(String, f32)> {
        let total = self.recent_champions.len() as f32;
        if total == 0.0 {
            return Vec::new();
        }

        let intel = self.compute_intel();
        intel
            .champion_frequencies
            .into_iter()
            .take(3)
            .map(|(name, count)| (name, count as f32 / total * 100.0))
            .collect()
    }

    /// Count of unique champions in recent_champions.
    pub fn pool_size(&self) -> usize {
        self.recent_champions
            .iter()
            .collect::<HashSet<_>>()
            .len()
    }
}

/// Returns true if the given ISO datetime string is 7 or more days old compared to now.
/// Returns false on empty string or parse failure.
pub fn is_stale(last_fetched: &str) -> bool {
    is_stale_with_now(last_fetched, Utc::now())
}

/// Deterministic version of is_stale for testing with a fixed "now" datetime.
pub fn is_stale_with_now(last_fetched: &str, now: DateTime<Utc>) -> bool {
    if last_fetched.is_empty() {
        return false;
    }
    let dt = match last_fetched.parse::<DateTime<Utc>>() {
        Ok(d) => d,
        Err(_) => return false,
    };
    let age = now.signed_duration_since(dt);
    age.num_days() >= 7
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_player(recent_champions: Vec<&str>) -> OpponentPlayer {
        OpponentPlayer {
            id: None,
            opponent_id: "opponent:test".into(),
            name: "TestPlayer".into(),
            role: "mid".into(),
            riot_puuid: None,
            riot_summoner_name: None,
            recent_champions: recent_champions.into_iter().map(String::from).collect(),
            notes: None,
            last_fetched: None,
            mastery_data_json: None,
            role_distribution_json: None,
        }
    }

    fn make_now(s: &str) -> DateTime<Utc> {
        s.parse::<DateTime<Utc>>().unwrap()
    }

    // --- compute_intel tests ---

    #[test]
    fn test_otp_7_of_10_is_otp() {
        // 7/10 = 70% > 60% -> OTP
        let champs = vec!["Ahri"; 7]
            .into_iter()
            .chain(vec!["Lux", "Syndra", "Zed"])
            .collect::<Vec<_>>();
        let player = make_player(champs);
        let intel = player.compute_intel();
        assert_eq!(intel.otp_champion, Some("Ahri".into()));
    }

    #[test]
    fn test_otp_6_of_10_is_not_otp() {
        // 6/10 = 60% -> exactly 60%, NOT strictly > 60%, so None
        let champs = vec!["Ahri"; 6]
            .into_iter()
            .chain(vec!["Lux", "Syndra", "Zed", "Orianna"])
            .collect::<Vec<_>>();
        let player = make_player(champs);
        let intel = player.compute_intel();
        assert_eq!(intel.otp_champion, None);
    }

    #[test]
    fn test_otp_5_of_10_is_not_otp() {
        // 5/10 = 50% < 60% -> None
        let champs = vec!["Ahri"; 5]
            .into_iter()
            .chain(vec!["Lux", "Syndra", "Zed", "Orianna", "Azir"])
            .collect::<Vec<_>>();
        let player = make_player(champs);
        let intel = player.compute_intel();
        assert_eq!(intel.otp_champion, None);
    }

    #[test]
    fn test_otp_zero_games_is_none() {
        let player = make_player(vec![]);
        let intel = player.compute_intel();
        assert_eq!(intel.otp_champion, None);
        assert!(intel.champion_frequencies.is_empty());
    }

    #[test]
    fn test_champion_frequencies_sorted_descending() {
        // Ahri x3, Lux x2, Zed x1
        let champs = vec!["Ahri", "Lux", "Ahri", "Zed", "Lux", "Ahri"];
        let player = make_player(champs);
        let intel = player.compute_intel();
        assert_eq!(intel.champion_frequencies[0], ("Ahri".into(), 3));
        assert_eq!(intel.champion_frequencies[1], ("Lux".into(), 2));
        assert_eq!(intel.champion_frequencies[2], ("Zed".into(), 1));
    }

    #[test]
    fn test_comfort_picks_top_3_with_percentages() {
        // Ahri x4, Lux x3, Zed x2, Syndra x1 (total=10)
        let champs = vec!["Ahri", "Ahri", "Ahri", "Ahri", "Lux", "Lux", "Lux", "Zed", "Zed", "Syndra"];
        let player = make_player(champs);
        let picks = player.comfort_picks();
        assert_eq!(picks.len(), 3);
        assert_eq!(picks[0].0, "Ahri");
        assert!((picks[0].1 - 40.0).abs() < 0.01, "Ahri should be 40%");
        assert_eq!(picks[1].0, "Lux");
        assert!((picks[1].1 - 30.0).abs() < 0.01, "Lux should be 30%");
        assert_eq!(picks[2].0, "Zed");
        assert!((picks[2].1 - 20.0).abs() < 0.01, "Zed should be 20%");
    }

    #[test]
    fn test_role_distribution_percentages() {
        let dist = vec![("mid".to_string(), 8u32), ("support".to_string(), 2u32)];
        let json = serde_json::to_string(&dist).unwrap();
        let mut player = make_player(vec![]);
        player.role_distribution_json = Some(json);
        let result = player.role_distribution();
        assert_eq!(result.len(), 2);
        let total: u32 = result.iter().map(|(_, c)| c).sum();
        assert_eq!(total, 10);
    }

    // --- is_stale tests ---

    #[test]
    fn test_is_stale_7_days_ago_is_stale() {
        // now = 2026-03-23T00:00:00Z, 7 days ago = 2026-03-16T00:00:00Z
        let now = make_now("2026-03-23T00:00:00Z");
        assert!(
            is_stale_with_now("2026-03-16T00:00:00Z", now),
            "7 days old should be stale"
        );
    }

    #[test]
    fn test_is_stale_less_than_7_days_not_stale() {
        // now = 2026-03-23T00:00:00Z, last_fetched = 2026-03-17T00:00:01Z (< 7 days)
        let now = make_now("2026-03-23T00:00:00Z");
        assert!(
            !is_stale_with_now("2026-03-17T00:00:01Z", now),
            "Less than 7 days old should not be stale"
        );
    }

    #[test]
    fn test_is_stale_empty_string_is_false() {
        let now = make_now("2026-03-23T00:00:00Z");
        assert!(!is_stale_with_now("", now), "Empty string should not be stale");
    }
}
