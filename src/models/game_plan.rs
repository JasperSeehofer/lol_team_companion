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
    #[serde(default)]
    pub win_loss: Option<String>,   // "win" | "loss" | None
    #[serde(default)]
    pub rating: Option<u8>,         // 1-5 stars | None
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

/// Strategy tag aggregation for analytics cards (per D-05, D-08)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StrategyTagSummary {
    pub tag: String,
    pub games_played: usize,
    pub wins: usize,
    pub losses: usize,
    pub avg_rating: Option<f32>,
}

/// Per-game-plan effectiveness row (per D-05, D-09)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GamePlanEffectiveness {
    pub plan_id: String,
    pub plan_name: String,
    pub tag: Option<String>,
    pub wins: usize,
    pub losses: usize,
    pub avg_rating: Option<f32>,
    pub reviews: Vec<PostGameLearning>,
}

/// Full analytics payload returned by the server function (per D-07)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AnalyticsPayload {
    pub tag_summaries: Vec<StrategyTagSummary>,
    pub plan_effectiveness: Vec<GamePlanEffectiveness>,
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
            win_loss: None,
            rating: None,
        };
        let json = serde_json::to_string(&pgl).unwrap();
        let back: PostGameLearning = serde_json::from_str(&json).unwrap();
        assert_eq!(pgl, back);
    }

    #[test]
    fn post_game_learning_with_win_loss_rating_round_trips() {
        let pgl = PostGameLearning {
            id: Some("post_game_learning:2".into()),
            team_id: "team:t1".into(),
            match_riot_id: None,
            game_plan_id: Some("game_plan:gp1".into()),
            draft_id: None,
            what_went_well: vec!["Great teamfight".into()],
            improvements: vec!["Vision control".into()],
            action_items: vec![],
            open_notes: None,
            created_by: "user:u1".into(),
            win_loss: Some("win".into()),
            rating: Some(4),
        };
        let json = serde_json::to_string(&pgl).unwrap();
        let back: PostGameLearning = serde_json::from_str(&json).unwrap();
        assert_eq!(pgl, back);
        assert_eq!(back.win_loss, Some("win".into()));
        assert_eq!(back.rating, Some(4));
    }

    #[test]
    fn post_game_learning_without_new_fields_deserializes() {
        // Simulate an old DB record that lacks win_loss and rating
        let old_json = r#"{
            "id": "post_game_learning:old",
            "team_id": "team:t1",
            "match_riot_id": null,
            "game_plan_id": null,
            "draft_id": null,
            "what_went_well": [],
            "improvements": [],
            "action_items": [],
            "open_notes": null,
            "created_by": "user:u1"
        }"#;
        let pgl: PostGameLearning = serde_json::from_str(old_json).unwrap();
        assert_eq!(pgl.win_loss, None);
        assert_eq!(pgl.rating, None);
    }

    #[test]
    fn strategy_tag_summary_round_trips() {
        let summary = StrategyTagSummary {
            tag: "teamfight".into(),
            games_played: 10,
            wins: 7,
            losses: 3,
            avg_rating: Some(4.2),
        };
        let json = serde_json::to_string(&summary).unwrap();
        let back: StrategyTagSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(summary, back);
    }

    #[test]
    fn analytics_payload_empty_round_trips() {
        let payload = AnalyticsPayload {
            tag_summaries: Vec::new(),
            plan_effectiveness: Vec::new(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let back: AnalyticsPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload, back);
    }
}
