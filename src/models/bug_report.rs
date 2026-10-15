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

/// Label for a stored category ("bug" | "wishlist"). A wishlist report is
/// an "Idea" in the UI (M141); use [`feedback_label`] where the description
/// is at hand, so praise shows as praise.
pub fn category_label(value: &str) -> &'static str {
    match value {
        "wishlist" => "Idea",
        _ => "Bug",
    }
}

/// The body prefix that marks praise. Praise is stored as `wishlist` (the
/// schema asserts `bug` | `wishlist`; M141 adds no migration).
pub const PRAISE_PREFIX: &str = "[praise] ";

/// What the "Give Feedback" dialog offers. Labels only: [`FeedbackKind::stored`]
/// maps each to the stored category and description.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackKind {
    Bug,
    Idea,
    Praise,
}

impl FeedbackKind {
    pub const ALL: [FeedbackKind; 3] =
        [FeedbackKind::Bug, FeedbackKind::Idea, FeedbackKind::Praise];

    /// The dialog's segment value.
    pub fn as_str(self) -> &'static str {
        match self {
            FeedbackKind::Bug => "bug",
            FeedbackKind::Idea => "idea",
            FeedbackKind::Praise => "praise",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == value)
    }

    pub fn label(self) -> &'static str {
        match self {
            FeedbackKind::Bug => "Bug",
            FeedbackKind::Idea => "Idea",
            FeedbackKind::Praise => "Praise",
        }
    }

    pub fn placeholder(self) -> &'static str {
        match self {
            FeedbackKind::Bug => "What did you do, what did you expect, what happened instead?",
            FeedbackKind::Idea => "What would help your team, and when would you use it?",
            FeedbackKind::Praise => "What worked well for you?",
        }
    }

    /// The stored category and description for this kind and text.
    pub fn stored(self, text: &str) -> (&'static str, String) {
        match self {
            FeedbackKind::Bug => ("bug", text.to_string()),
            FeedbackKind::Idea => ("wishlist", text.to_string()),
            FeedbackKind::Praise => ("wishlist", format!("{PRAISE_PREFIX}{}", text.trim_start())),
        }
    }

    /// Read a stored report back as a kind.
    pub fn of_stored(category: &str, description: &str) -> Self {
        match category {
            "wishlist" if description.starts_with(PRAISE_PREFIX.trim_end()) => FeedbackKind::Praise,
            "wishlist" => FeedbackKind::Idea,
            _ => FeedbackKind::Bug,
        }
    }
}

/// "Bug", "Idea" or "Praise" for a stored report.
pub fn feedback_label(category: &str, description: &str) -> &'static str {
    FeedbackKind::of_stored(category, description).label()
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
        assert_eq!(category_label("wishlist"), "Idea");
        assert_eq!(category_label("bug"), "Bug");
    }

    #[test]
    fn feedback_kinds_map_to_the_two_stored_categories() {
        assert_eq!(FeedbackKind::Bug.stored("x"), ("bug", "x".to_string()));
        assert_eq!(
            FeedbackKind::Idea.stored("x"),
            ("wishlist", "x".to_string())
        );
        assert_eq!(
            FeedbackKind::Praise.stored("  Love it"),
            ("wishlist", "[praise] Love it".to_string())
        );
        for k in FeedbackKind::ALL {
            assert_eq!(FeedbackKind::parse(k.as_str()), Some(k));
            let (cat, body) = k.stored("text");
            assert!(cat == "bug" || cat == "wishlist");
            assert_eq!(FeedbackKind::of_stored(cat, &body), k);
            assert_eq!(feedback_label(cat, &body), k.label());
        }
        assert_eq!(FeedbackKind::parse("wishlist"), None);
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
