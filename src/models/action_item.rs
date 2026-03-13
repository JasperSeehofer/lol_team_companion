use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ActionItem {
    pub id: Option<String>,
    pub team_id: String,
    pub source_review: Option<String>,
    pub text: String,
    /// "open", "in_progress", "done"
    pub status: String,
    pub assigned_to: Option<String>,
    pub created_at: Option<String>,
    pub resolved_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_item_round_trips_json() {
        let item = ActionItem {
            id: Some("action_item:1".into()),
            team_id: "team:t1".into(),
            source_review: Some("post_game_learning:1".into()),
            text: "Review baron fight".into(),
            status: "open".into(),
            assigned_to: Some("user:u1".into()),
            created_at: Some("2024-01-01T00:00:00Z".into()),
            resolved_at: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        let back: ActionItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, back);
    }
}
