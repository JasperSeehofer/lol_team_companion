use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GamePlan {
    pub id: Option<String>,
    pub draft_id: Option<String>,
    pub team_id: String,
    pub win_conditions: Vec<String>,
    pub objective_priority: Vec<String>,
    pub teamfight_strategy: String,
    pub early_game: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PostGameLearning {
    pub id: Option<String>,
    pub match_id: Option<String>,
    pub team_id: String,
    pub what_went_well: Vec<String>,
    pub improvements: Vec<String>,
    pub action_items: Vec<String>,
    pub created_by: String,
}
