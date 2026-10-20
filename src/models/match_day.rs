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
    /// M159: the series a Bo3 or Bo5 is played as (if any).
    #[serde(default)]
    pub series_id: Option<String>,
}

/// M159: one row of Match Day's upcoming-match list.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UpcomingMatch {
    pub id: String,
    /// Start time, RFC 3339 in UTC.
    pub starts_at: String,
    /// "scrim" | "match"
    pub kind: String,
    /// "bo1" | "bo3" | "bo5"
    pub format: String,
    pub opponent_name: Option<String>,
    /// How many members confirmed (RSVP, M159 item 7). Not shown while
    /// `None`.
    #[serde(default)]
    pub confirmed: Option<usize>,
}

impl From<&ScheduledMatch> for UpcomingMatch {
    fn from(m: &ScheduledMatch) -> Self {
        UpcomingMatch {
            id: m.id.clone(),
            starts_at: m.starts_at.clone(),
            kind: m.kind.clone(),
            format: m.format.clone(),
            opponent_name: m.opponent_name.clone(),
            confirmed: None,
        }
    }
}

/// M159: whether two scheduled-match ids name the same record (with or
/// without the `scheduled_match:` prefix).
pub fn same_match_id(a: &str, b: &str) -> bool {
    let key = |s: &str| {
        let s = s.trim();
        s.strip_prefix("scheduled_match:").unwrap_or(s).to_string()
    };
    !a.trim().is_empty() && key(a) == key(b)
}

/// The current match of Match Day and the rest of the upcoming list (M159).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct MatchChoice {
    /// The match the page is about: the chosen one, else the next one.
    pub current: Option<ScheduledMatch>,
    /// The other upcoming matches, earliest first (the current one left out).
    pub others: Vec<ScheduledMatch>,
    /// A `match_id` was given and is the current match, not the next one.
    pub chosen: bool,
    /// A `match_id` was given but is not one of the team's upcoming
    /// matches: the page shows the next match and says so.
    pub unknown: bool,
}

/// Pick Match Day's current match from the team's upcoming matches
/// (earliest first) and an optional `match_id` from the address. An id that
/// is not in `upcoming` (another team's, an old or a made-up one) falls
/// back to `next`, the same for every such id.
pub fn choose_match(
    upcoming: Vec<ScheduledMatch>,
    next: Option<ScheduledMatch>,
    match_id: Option<&str>,
) -> MatchChoice {
    let wanted = match_id.map(str::trim).filter(|s| !s.is_empty());
    let found = wanted.and_then(|w| upcoming.iter().position(|m| same_match_id(w, &m.id)));
    let current = match found {
        Some(i) => Some(upcoming[i].clone()),
        None => next,
    };
    let chosen = found.is_some_and(|i| i != 0);
    let others = upcoming
        .into_iter()
        .filter(|m| current.as_ref().is_none_or(|c| c.id != m.id))
        .collect();
    MatchChoice {
        current,
        others,
        chosen,
        unknown: wanted.is_some() && found.is_none(),
    }
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

/// How long a patch change stays news on Match Day (M5 review: the line
/// "Patch 16.20 since 14 Sep" is true for months but stops being useful).
pub const PATCH_NEWS_DAYS: i64 = 14;

impl PatchLine {
    /// Whether the change happened less than [`PATCH_NEWS_DAYS`] before
    /// `now`. An unreadable date is not news.
    pub fn is_news(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        chrono::DateTime::parse_from_rfc3339(&self.since)
            .map(|t| now.signed_duration_since(t) < chrono::Duration::days(PATCH_NEWS_DAYS))
            .unwrap_or(false)
    }
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
    /// M159: the team's other upcoming matches, earliest first (the
    /// current one left out; at most 19).
    #[serde(default)]
    pub upcoming: Vec<UpcomingMatch>,
    /// M159: the current match was chosen from the list (`/?match_id=`)
    /// and is not the next one.
    #[serde(default)]
    pub chosen: bool,
    /// M159: the address named a match that is not on the team's schedule;
    /// the page shows the next match instead and says so.
    #[serde(default)]
    pub unknown_match: bool,
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
    #[test]
    fn a_patch_is_news_for_two_weeks() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-12T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let line = |since: &str| PatchLine {
            version: "16.20".into(),
            since: since.into(),
        };
        assert!(line("2026-10-01T09:00:00Z").is_news(now));
        assert!(line("2026-09-28T12:00:01Z").is_news(now));
        assert!(!line("2026-09-28T12:00:00Z").is_news(now));
        assert!(!line("2026-08-01T09:00:00Z").is_news(now));
        assert!(!line("not a date").is_news(now));
    }

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

    fn sm(id: &str, starts_at: &str) -> super::ScheduledMatch {
        super::ScheduledMatch {
            id: id.into(),
            starts_at: starts_at.into(),
            kind: "scrim".into(),
            format: "bo3".into(),
            opponent_id: None,
            opponent_name: Some("Rooks".into()),
            notes: None,
            brief_read_by: vec![],
            series_id: None,
        }
    }

    #[test]
    fn the_current_match_comes_from_the_list_or_falls_back_to_the_next() {
        use super::choose_match;
        let list = vec![
            sm("scheduled_match:a", "2026-10-20T18:00:00Z"),
            sm("scheduled_match:b", "2026-10-21T18:00:00Z"),
            sm("scheduled_match:c", "2026-10-22T18:00:00Z"),
        ];
        let next = Some(list[0].clone());

        // No id: the next match, the others listed.
        let c = choose_match(list.clone(), next.clone(), None);
        assert_eq!(
            c.current.as_ref().map(|m| m.id.as_str()),
            Some("scheduled_match:a")
        );
        assert_eq!(c.others.len(), 2);
        assert!(!c.chosen && !c.unknown);

        // A later match, with or without the prefix: it is current and the
        // next one moves into the list.
        for id in ["scheduled_match:b", "b", " b "] {
            let c = choose_match(list.clone(), next.clone(), Some(id));
            assert_eq!(
                c.current.as_ref().map(|m| m.id.as_str()),
                Some("scheduled_match:b")
            );
            let ids: Vec<&str> = c.others.iter().map(|m| m.id.as_str()).collect();
            assert_eq!(ids, ["scheduled_match:a", "scheduled_match:c"]);
            assert!(c.chosen && !c.unknown);
        }

        // The next match by id is not "chosen".
        let c = choose_match(list.clone(), next.clone(), Some("scheduled_match:a"));
        assert!(!c.chosen && !c.unknown);

        // Unknown (or another team's) id: the next match and a flag.
        let c = choose_match(list.clone(), next.clone(), Some("scheduled_match:zzz"));
        assert_eq!(
            c.current.as_ref().map(|m| m.id.as_str()),
            Some("scheduled_match:a")
        );
        assert!(c.unknown && !c.chosen);
        assert_eq!(c.others.len(), 2);

        // A blank id is no id.
        let c = choose_match(list.clone(), next.clone(), Some("  "));
        assert!(!c.unknown);

        // Nothing scheduled.
        let c = choose_match(vec![], None, Some("scheduled_match:a"));
        assert!(c.current.is_none() && c.others.is_empty() && c.unknown);
    }
}
