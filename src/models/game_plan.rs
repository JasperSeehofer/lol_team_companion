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
    // Strategy tag (e.g. "teamfight", "split-push", "poke")
    pub win_condition_tag: Option<String>,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChecklistTemplate {
    pub id: Option<String>,
    pub team_id: String,
    pub name: String,
    pub items: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChecklistInstance {
    pub id: Option<String>,
    pub team_id: String,
    pub game_plan_id: Option<String>,
    pub template_id: Option<String>,
    pub items: Vec<String>,
    pub checked: Vec<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_plan_all_options_none_round_trips() {
        let plan = GamePlan {
            id: None,
            team_id: "team:t1".into(),
            draft_id: None,
            name: "Plan A".into(),
            our_champions: vec!["Jinx".into()],
            enemy_champions: vec!["Caitlyn".into()],
            win_conditions: vec!["Teamfight".into()],
            objective_priority: vec!["Dragon".into()],
            teamfight_strategy: "Fight 5v5".into(),
            early_game: None,
            top_strategy: None,
            jungle_strategy: None,
            mid_strategy: None,
            bot_strategy: None,
            support_strategy: None,
            notes: None,
            win_condition_tag: None,
        };
        let json = serde_json::to_string(&plan).unwrap();
        let back: GamePlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, back);
    }

    #[test]
    fn game_plan_all_options_some_round_trips() {
        let plan = GamePlan {
            id: Some("game_plan:1".into()),
            team_id: "team:t1".into(),
            draft_id: Some("draft:1".into()),
            name: "Plan B".into(),
            our_champions: vec![],
            enemy_champions: vec![],
            win_conditions: vec![],
            objective_priority: vec![],
            teamfight_strategy: "Poke".into(),
            early_game: Some("Invade".into()),
            top_strategy: Some("Split push".into()),
            jungle_strategy: Some("Farm".into()),
            mid_strategy: Some("Roam".into()),
            bot_strategy: Some("Farm".into()),
            support_strategy: Some("Engage".into()),
            notes: Some("Important notes".into()),
            win_condition_tag: Some("teamfight".into()),
        };
        let json = serde_json::to_string(&plan).unwrap();
        let back: GamePlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, back);
    }

    #[test]
    fn post_game_learning_round_trips_json() {
        let pgl = PostGameLearning {
            id: Some("post_game_learning:1".into()),
            team_id: "team:t1".into(),
            match_riot_id: Some("EUW1_1234".into()),
            game_plan_id: None,
            draft_id: None,
            what_went_well: vec!["Dragon control".into()],
            improvements: vec!["Baron timing".into()],
            action_items: vec!["Review baron fight vod".into()],
            open_notes: None,
            created_by: "user:u1".into(),
        };
        let json = serde_json::to_string(&pgl).unwrap();
        let back: PostGameLearning = serde_json::from_str(&json).unwrap();
        assert_eq!(pgl, back);
    }
}
