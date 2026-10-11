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
    /// Due day, `YYYY-MM-DD`.
    #[serde(default)]
    pub due: Option<String>,
    pub created_at: Option<String>,
    pub resolved_at: Option<String>,
}

impl ActionItem {
    /// Checks a due date typed by a user: empty means none; otherwise a
    /// calendar date written `YYYY-MM-DD` (what a date input sends).
    pub fn parse_due(s: &str) -> Result<Option<String>, &'static str> {
        let s = s.trim();
        if s.is_empty() {
            return Ok(None);
        }
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map(|d| Some(d.format("%Y-%m-%d").to_string()))
            .map_err(|_| "Use a date like 2026-10-31.")
    }

    /// Open or in progress with a due day before `today` (`YYYY-MM-DD`).
    pub fn is_overdue(&self, today: &str) -> bool {
        self.status != "done" && self.due.as_deref().is_some_and(|d| d < today)
    }
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
            due: Some("2024-01-08".into()),
            created_at: Some("2024-01-01T00:00:00Z".into()),
            resolved_at: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        let back: ActionItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, back);
    }

    #[test]
    fn parse_due_accepts_dates_and_empty() {
        assert_eq!(ActionItem::parse_due(""), Ok(None));
        assert_eq!(
            ActionItem::parse_due(" 2026-10-31 "),
            Ok(Some("2026-10-31".into()))
        );
        assert!(ActionItem::parse_due("2026-02-30").is_err());
        assert!(ActionItem::parse_due("31.10.2026").is_err());
        assert!(ActionItem::parse_due("2026-10-31' OR 1=1").is_err());
    }

    #[test]
    fn overdue_needs_an_open_item_past_its_day() {
        let mut item: ActionItem = serde_json::from_str(
            r#"{"id":null,"team_id":"team:t","source_review":null,"text":"x","status":"open","assigned_to":null,"created_at":null,"resolved_at":null}"#,
        )
        .unwrap();
        assert!(!item.is_overdue("2026-10-11"));
        item.due = Some("2026-10-10".into());
        assert!(item.is_overdue("2026-10-11"));
        assert!(!item.is_overdue("2026-10-10"));
        item.status = "done".into();
        assert!(!item.is_overdue("2026-10-11"));
    }
}
