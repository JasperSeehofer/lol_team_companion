use serde::{Deserialize, Serialize};

/// Where a bug report or wish stands (M7). Stored as the `as_str` value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BugStatus {
    New,
    Triaged,
    Planned,
    Fixed,
    WontFix,
}

impl BugStatus {
    pub const ALL: [BugStatus; 5] = [
        BugStatus::New,
        BugStatus::Triaged,
        BugStatus::Planned,
        BugStatus::Fixed,
        BugStatus::WontFix,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            BugStatus::New => "new",
            BugStatus::Triaged => "triaged",
            BugStatus::Planned => "planned",
            BugStatus::Fixed => "fixed",
            BugStatus::WontFix => "wont_fix",
        }
    }

    pub fn parse(value: &str) -> Option<BugStatus> {
        BugStatus::ALL.into_iter().find(|s| s.as_str() == value)
    }

    pub fn label(self) -> &'static str {
        match self {
            BugStatus::New => "New",
            BugStatus::Triaged => "Triaged",
            BugStatus::Planned => "Planned",
            BugStatus::Fixed => "Fixed",
            BugStatus::WontFix => "Won't fix",
        }
    }

    /// Open on the board: read, and either understood or planned.
    pub fn is_open_on_board(self) -> bool {
        matches!(self, BugStatus::Triaged | BugStatus::Planned)
    }
}

/// Label for a stored status string; unknown values are shown as written.
pub fn status_label(value: &str) -> String {
    BugStatus::parse(value)
        .map(|s| s.label().to_string())
        .unwrap_or_else(|| value.to_string())
}

/// Label for a stored category ("bug" | "wishlist").
pub fn category_label(value: &str) -> &'static str {
    match value {
        "wishlist" => "Wishlist",
        _ => "Bug",
    }
}

/// A report as the startup inbox export sees it (the reporter's words
/// included: the export is for the maintainer only).
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
    /// "new" | "triaged" | "planned" | "fixed" | "wont_fix" (see [`BugStatus`])
    pub status: String,
}

/// One of the caller's own reports ("Your reports").
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MyBugReport {
    pub id: String,
    pub created_at: String,
    pub page_url: String,
    pub element_label: String,
    pub description: String,
    pub category: String,
    pub status: String,
    pub public_note: Option<String>,
    pub fixed_in: Option<String>,
    pub fixed_in_title: Option<String>,
    /// The status changed since the reporter last opened "Your reports".
    pub updated: bool,
}

/// A report on the bug board: only what an admin wrote for everyone.
/// Never the reporter's description, page, element, screen size or name.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PublicBugReport {
    pub id: String,
    pub public_title: String,
    pub public_note: Option<String>,
    pub category: String,
    pub status: String,
    pub status_changed_at: Option<String>,
    pub fixed_in: Option<String>,
    pub fixed_in_title: Option<String>,
}

/// Everything about a report, for triage (admins only).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BugReportAdmin {
    pub id: String,
    pub reporter: String,
    pub created_at: String,
    pub page_url: String,
    pub element_label: String,
    pub description: String,
    pub category: String,
    pub viewport_w: Option<i32>,
    pub viewport_h: Option<i32>,
    pub status: String,
    pub on_board: bool,
    pub public_title: Option<String>,
    pub public_note: Option<String>,
    pub admin_note: Option<String>,
    pub fixed_in: Option<String>,
    pub status_changed_at: Option<String>,
    pub triaged_by: Option<String>,
    pub updated_at: Option<String>,
    pub proposal_id: Option<String>,
}

/// An admin's triage decision, sent as JSON (leptos rule 33).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BugTriage {
    pub id: String,
    pub status: String,
    pub on_board: bool,
    pub public_title: String,
    pub public_note: String,
    pub admin_note: String,
    pub fixed_in: String,
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
    fn statuses_round_trip_and_have_labels() {
        for s in BugStatus::ALL {
            assert_eq!(BugStatus::parse(s.as_str()), Some(s));
            assert!(!s.label().is_empty());
        }
        assert_eq!(BugStatus::parse("open"), None);
        assert_eq!(BugStatus::parse("closed"), None);
        assert_eq!(BugStatus::WontFix.label(), "Won't fix");
        assert_eq!(status_label("wont_fix"), "Won't fix");
        assert!(BugStatus::Planned.is_open_on_board());
        assert!(!BugStatus::New.is_open_on_board());
        assert_eq!(category_label("wishlist"), "Wishlist");
        assert_eq!(category_label("bug"), "Bug");
    }

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
            status: "new".into(),
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
