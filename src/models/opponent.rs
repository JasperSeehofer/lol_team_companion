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
