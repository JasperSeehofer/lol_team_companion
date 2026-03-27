use serde::{Deserialize, Serialize};

/// Predefined category tags for personal learnings (per D-06).
pub const LEARNING_TAGS: &[&str] = &[
    "Laning",
    "Teamfighting",
    "Macro / Rotations",
    "Vision",
    "Trading",
    "Wave Management",
    "Objective Control",
    "Mental / Tilt",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PersonalLearning {
    pub id: Option<String>,
    pub user_id: String,
    pub title: String,
    pub learning_type: String, // "general" | "champion" | "matchup"
    pub champion: Option<String>,
    pub opponent: Option<String>,
    pub what_happened: String,
    pub what_i_learned: String,
    pub next_time: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub win_loss: Option<String>, // "win" | "loss" | None
    #[serde(default)]
    pub match_riot_id: Option<String>,
    #[serde(default)]
    pub game_timestamp_ms: Option<i64>,
    #[serde(default)]
    pub event_name: Option<String>,
    pub created_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn personal_learning_round_trip() {
        let pl = PersonalLearning {
            id: Some("personal_learning:abc".into()),
            user_id: "user:123".into(),
            title: "Zed vs Ahri".into(),
            learning_type: "matchup".into(),
            champion: Some("Zed".into()),
            opponent: Some("Ahri".into()),
            what_happened: "Got solo killed at 6".into(),
            what_i_learned: "Respect Ahri charm range".into(),
            next_time: "Play safe at 6 until I have Serrated Dirk".into(),
            tags: vec!["Laning".into(), "Trading".into()],
            win_loss: Some("loss".into()),
            match_riot_id: Some("EUW1_12345".into()),
            game_timestamp_ms: Some(360000),
            event_name: Some("Champion Kill at 6:00".into()),
            created_at: Some("2026-03-27T10:00:00Z".into()),
        };
        let json = serde_json::to_string(&pl).unwrap();
        let deserialized: PersonalLearning = serde_json::from_str(&json).unwrap();
        assert_eq!(pl, deserialized);
    }

    #[test]
    fn personal_learning_missing_optional_fields() {
        let json = r#"{"user_id":"user:1","title":"Test","learning_type":"general","what_happened":"x","what_i_learned":"y","next_time":"z","created_at":null}"#;
        let pl: PersonalLearning = serde_json::from_str(json).unwrap();
        assert_eq!(pl.tags, Vec::<String>::new());
        assert_eq!(pl.win_loss, None);
        assert_eq!(pl.champion, None);
    }
}
