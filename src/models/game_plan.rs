use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GamePlan {
    pub id: Option<String>,
    pub team_id: String,
    pub draft_id: Option<String>,
    pub name: String,
    // Matchup champions (5 each)
    pub our_champions: Vec<String>,
    pub enemy_champions: Vec<String>,
    // Macro strategy
    pub win_conditions: Vec<String>,
    pub objective_priority: Vec<String>,
    pub teamfight_strategy: String,
    pub early_game: Option<String>,
    // Role-specific strategy
    pub top_strategy: Option<String>,
    pub jungle_strategy: Option<String>,
    pub mid_strategy: Option<String>,
    pub bot_strategy: Option<String>,
    pub support_strategy: Option<String>,
    // Notes
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PostGameLearning {
    pub id: Option<String>,
    pub team_id: String,
    pub match_riot_id: Option<String>,
    pub game_plan_id: Option<String>,
    pub draft_id: Option<String>,
    pub what_went_well: Vec<String>,
    pub improvements: Vec<String>,
    pub action_items: Vec<String>,
    pub open_notes: Option<String>,
    pub created_by: String,
}
