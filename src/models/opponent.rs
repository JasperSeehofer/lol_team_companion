use serde::{Deserialize, Serialize};

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
