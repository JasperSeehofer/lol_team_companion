//! Match Day (M5): the team's next scrim or match and what the page shows
//! around it. Shared by the server functions and the page.

use serde::{Deserialize, Serialize};

/// Formats a match can be played in.
pub const FORMATS: [(&str, &str); 3] = [
    ("bo1", "Best of 1"),
    ("bo3", "Best of 3"),
    ("bo5", "Best of 5"),
];
/// Kinds of match.
pub const KINDS: [(&str, &str); 2] = [("scrim", "Scrim"), ("match", "Official match")];

/// "Best of 3" for a stored format.
pub fn format_label(format: &str) -> &'static str {
    FORMATS
        .iter()
        .find(|(k, _)| *k == format)
        .map(|(_, l)| *l)
        .unwrap_or("Best of 1")
}

/// "Scrim" or "Official match".
pub fn kind_label(kind: &str) -> &'static str {
    KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, l)| *l)
        .unwrap_or("Scrim")
}

/// The team's next scrim or match as entered by a member.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ScheduledMatch {
    pub id: String,
    /// Start time, RFC 3339 in UTC.
    pub starts_at: String,
    /// "scrim" | "match"
    pub kind: String,
    /// "bo1" | "bo3" | "bo5"
    pub format: String,
    pub opponent_id: Option<String>,
    pub opponent_name: Option<String>,
    pub notes: Option<String>,
    /// Members (user ids) who marked the opponent brief read.
    pub brief_read_by: Vec<String>,
}

/// What the "next match" editor sends.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct ScheduledMatchInput {
    /// Set to edit an existing match.
    pub id: Option<String>,
    /// Start time, RFC 3339 (the browser converts local time to UTC).
    pub starts_at: String,
    pub kind: String,
    pub format: String,
    pub opponent_id: Option<String>,
    pub notes: Option<String>,
}

/// A record the page links to.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LinkRef {
    pub id: String,
    pub name: String,
}

/// One scouted player in the opponent brief.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BriefPlayer {
    pub name: String,
    pub role: String,
    /// Champions seen, at most three.
    pub champions: Vec<String>,
}

/// The dossier summary on Match Day.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpponentBrief {
    pub id: String,
    pub name: String,
    pub notes: Option<String>,
    pub players: Vec<BriefPlayer>,
    pub champions_seen: usize,
    /// "3 days ago", `None` when no player was ever refreshed from Riot.
    pub surveyed: Option<String>,
    /// Surveyed a week or more ago.
    pub stale: bool,
}

/// The last review, as Match Day quotes it.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReviewLine {
    pub id: String,
    /// Some(true) won, Some(false) lost, None not recorded.
    pub won: Option<bool>,
    /// The first "to improve" line, else the first action item line, else
    /// the start of the notes.
    pub lesson: Option<String>,
    pub created_at: Option<String>,
}

/// The patch line ("Patch 16.20 since Tuesday").
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PatchLine {
    pub version: String,
    /// When the change was detected, RFC 3339.
    pub since: String,
}

/// Everything Match Day shows for a signed-in team member.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MatchDay {
    pub team_name: String,
    pub member_count: usize,
    pub viewer_id: String,
    pub next: Option<ScheduledMatch>,
    pub brief: Option<OpponentBrief>,
    /// Drafts against the next opponent.
    pub drafts: Vec<LinkRef>,
    /// Game plans linked to those drafts.
    pub plans: Vec<LinkRef>,
    /// A review of this match (linked to its drafts or plans, written after
    /// it started).
    pub review_after: Option<LinkRef>,
    pub open_items: Vec<crate::models::action_item::ActionItem>,
    pub open_item_count: usize,
    pub last_review: Option<ReviewLine>,
    /// Results of the last five reviewed games, oldest first.
    pub form: Vec<bool>,
    pub patch: Option<PatchLine>,
    /// The team's dossiers, for the editor's opponent list.
    pub opponents: Vec<LinkRef>,
    /// Server time when this was built, RFC 3339.
    pub now: String,
}

/// Shorten a line to `max` characters on a word boundary, with an ellipsis.
pub fn shorten(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max).collect();
    let cut = match cut.rfind(' ') {
        Some(i) if i > max / 2 => cut[..i].to_string(),
        _ => cut,
    };
    format!("{}\u{2026}", cut.trim_end_matches([',', '.', ';', ':']))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_cover_every_stored_value() {
        assert_eq!(format_label("bo3"), "Best of 3");
        assert_eq!(format_label("nonsense"), "Best of 1");
        assert_eq!(kind_label("match"), "Official match");
    }

    #[test]
    fn shorten_cuts_on_words() {
        assert_eq!(shorten("short", 10), "short");
        assert_eq!(
            shorten("Lost tempo after twenty minutes, again", 20),
            "Lost tempo after\u{2026}"
        );
    }
}
