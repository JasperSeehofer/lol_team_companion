use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BugReport {
    pub id: Option<String>,
    pub user_id: String,
    pub page_url: String,
    pub element_label: String,
    pub description: String,
    /// "bug" | "wishlist"
    pub category: String,
    pub viewport_w: Option<i32>,
    pub viewport_h: Option<i32>,
    pub created_at: Option<String>,
    /// "open" | "triaged" | "closed"
    pub status: String,
}

/// Payload sent by the widget on submit. Server fn receives positional args
/// (see leptos-patterns rule 32), but a Default struct keeps client-side state
/// ergonomic.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct NewBugReport {
    pub page_url: String,
    pub element_label: String,
    pub description: String,
    pub category: String,
    pub viewport_w: Option<i32>,
    pub viewport_h: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bug_report_round_trips_json() {
        let report = BugReport {
            id: Some("bug_report:1".into()),
            user_id: "user:u1".into(),
            page_url: "/draft".into(),
            element_label: "Draft → Blue side → Pick 3".into(),
            description: "Hover broke".into(),
            category: "bug".into(),
            viewport_w: Some(1920),
            viewport_h: Some(1080),
            created_at: Some("2026-05-26T00:00:00Z".into()),
            status: "open".into(),
        };
        let json = serde_json::to_string(&report).unwrap();
        let back: BugReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report, back);
    }

    #[test]
    fn new_bug_report_round_trips_json() {
        let req = NewBugReport {
            page_url: "/draft".into(),
            element_label: "Draft → Blue side → Pick 3".into(),
            description: "Hover broke".into(),
            category: "bug".into(),
            viewport_w: Some(1920),
            viewport_h: Some(1080),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: NewBugReport = serde_json::from_str(&json).unwrap();
        assert_eq!(req, back);
    }
}
