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

/// Aggregation summary for the dashboard widget.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DashboardSummary {
    pub open_action_item_count: usize,
    pub recent_action_items: Vec<ActionItemPreview>,
    pub recent_post_games: Vec<PostGamePreview>,
    pub pool_gap_warnings: Vec<PoolGapWarning>,
    pub drafts_without_game_plan: usize,
    pub game_plans_without_post_game: usize,
}

/// Lightweight preview of a single action item for the dashboard.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ActionItemPreview {
    pub id: String,
    pub text: String,
}

/// Lightweight preview of a post-game learning record for the dashboard.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PostGamePreview {
    pub id: String,
    pub improvements: Vec<String>,
    pub created_at: Option<String>,
}

/// Warning that a player's champion pool lacks coverage for a given role/class.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PoolGapWarning {
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub dominant_class: Option<String>,
    pub missing_classes: Vec<String>,
    pub opponent_escalated: bool,
}

/// Cross-feature performance summary for a single champion.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct ChampionPerformanceSummary {
    pub champion: String,
    pub games_in_draft: usize,
    pub games_in_match: usize,
    pub wins_in_match: usize,
    pub games_in_plan: usize,
    pub post_game_wins: usize,
    pub post_game_losses: usize,
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
    fn dashboard_summary_default_round_trips() {
        let summary = DashboardSummary::default();
        let json = serde_json::to_string(&summary).unwrap();
        let back: DashboardSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(summary, back);
        assert_eq!(back.open_action_item_count, 0);
        assert_eq!(back.drafts_without_game_plan, 0);
    }

    #[test]
    fn dashboard_summary_populated_round_trips() {
        let summary = DashboardSummary {
            open_action_item_count: 3,
            recent_action_items: vec![
                ActionItemPreview { id: "ai:1".into(), text: "Review baron fight".into() },
            ],
            recent_post_games: vec![
                PostGamePreview {
                    id: "pg:1".into(),
                    improvements: vec!["Better vision control".into()],
                    created_at: Some("2026-03-14T20:00:00Z".into()),
                },
            ],
            pool_gap_warnings: vec![
                PoolGapWarning {
                    user_id: "user:u1".into(),
                    username: "Player1".into(),
                    role: "top".into(),
                    dominant_class: Some("Fighter".into()),
                    missing_classes: vec!["Tank".into()],
                    opponent_escalated: true,
                },
            ],
            drafts_without_game_plan: 2,
            game_plans_without_post_game: 1,
        };
        let json = serde_json::to_string(&summary).unwrap();
        let back: DashboardSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(summary, back);
    }

    #[test]
    fn champion_performance_summary_round_trips() {
        let summary = ChampionPerformanceSummary {
            champion: "Jinx".into(),
            games_in_draft: 10,
            games_in_match: 8,
            wins_in_match: 5,
            games_in_plan: 7,
            post_game_wins: 4,
            post_game_losses: 3,
        };
        let json = serde_json::to_string(&summary).unwrap();
        let back: ChampionPerformanceSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(summary, back);
    }

    #[test]
    fn pool_gap_warning_round_trips() {
        let w = PoolGapWarning {
            user_id: "user:u1".into(),
            username: "Player1".into(),
            role: "jungle".into(),
            dominant_class: None,
            missing_classes: vec!["Assassin".into(), "Tank".into()],
            opponent_escalated: false,
        };
        let json = serde_json::to_string(&w).unwrap();
        let back: PoolGapWarning = serde_json::from_str(&json).unwrap();
        assert_eq!(w, back);
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
