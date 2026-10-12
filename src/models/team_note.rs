use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TeamNote {
    pub id: Option<String>,
    pub team_id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub pinned: bool,
    pub created_at: Option<String>,
    /// When the text was last changed, and by whom.
    #[serde(default)]
    pub edited_at: Option<String>,
    #[serde(default)]
    pub edited_by_name: Option<String>,
    /// The last edit can be undone (the replaced text is kept).
    #[serde(default)]
    pub can_undo_edit: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_note_round_trips_json() {
        let note = TeamNote {
            id: Some("team_note:1".into()),
            team_id: "team:t1".into(),
            author_id: "user:u1".into(),
            author_name: "JohnDoe".into(),
            content: "Practice session takeaways: focus on dragon control".into(),
            pinned: true,
            created_at: Some("2024-01-01T00:00:00Z".into()),
            edited_at: None,
            edited_by_name: None,
            can_undo_edit: false,
        };
        let json = serde_json::to_string(&note).unwrap();
        let back: TeamNote = serde_json::from_str(&json).unwrap();
        assert_eq!(note, back);
    }
}
