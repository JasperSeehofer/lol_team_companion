//! M165 "Review together": the shared types of a review session and the pure
//! rules behind them (phases, text limits, lexical clustering, vote caps,
//! the quiet nudge). The database side is `server::review_together`; the
//! server functions are in `pages::review_together`.
//!
//! The anonymity rule lives in the types: cards of the anonymous prompt
//! ("What didn't go to plan", [`Prompt::OffPlan`]) travel as
//! [`AnonymousCard`], which has no author field at all, so no code path can
//! serialize one with its author. No card carries a timestamp, and cards are
//! ordered by a random key written at the reveal, never by time.

use serde::{Deserialize, Serialize};

/// The four prompts of the silent round.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Prompt {
    /// "The moment that decided the game" (attributed).
    Decided,
    /// "What went to plan" (attributed).
    ToPlan,
    /// "What didn't go to plan" (anonymous to the team).
    OffPlan,
    /// "One thing for next time" (attributed).
    NextTime,
}

impl Prompt {
    pub const ALL: [Prompt; 4] = [
        Prompt::Decided,
        Prompt::ToPlan,
        Prompt::OffPlan,
        Prompt::NextTime,
    ];

    /// The stored value (`review_entry.prompt`).
    pub fn key(self) -> &'static str {
        match self {
            Prompt::Decided => "decided",
            Prompt::ToPlan => "to_plan",
            Prompt::OffPlan => "off_plan",
            Prompt::NextTime => "next_time",
        }
    }

    pub fn from_key(key: &str) -> Option<Prompt> {
        Prompt::ALL.into_iter().find(|p| p.key() == key)
    }

    /// The question as members read it.
    pub fn label(self) -> &'static str {
        match self {
            Prompt::Decided => "The moment that decided the game",
            Prompt::ToPlan => "What went to plan",
            Prompt::OffPlan => "What didn't go to plan",
            Prompt::NextTime => "One thing for next time",
        }
    }

    /// Cards of this prompt never show who wrote them.
    pub fn is_anonymous(self) -> bool {
        self == Prompt::OffPlan
    }
}

/// The caption under the anonymous prompt's heading after the reveal.
pub const ANONYMOUS_CAPTION: &str = "Anonymous: nobody on the team sees who wrote these.";

/// Where a session is. `Writing` turns into `Revealed` by itself when
/// `writing_closes_at` passes (computed at read time, [`effective_phase`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Writing,
    Revealed,
    Deciding,
    Closed,
}

impl Phase {
    pub fn key(self) -> &'static str {
        match self {
            Phase::Writing => "writing",
            Phase::Revealed => "revealed",
            Phase::Deciding => "deciding",
            Phase::Closed => "closed",
        }
    }

    pub fn from_key(key: &str) -> Option<Phase> {
        [
            Phase::Writing,
            Phase::Revealed,
            Phase::Deciding,
            Phase::Closed,
        ]
        .into_iter()
        .find(|p| p.key() == key)
    }

    /// Cards are shown, +1s, merges, proposals and decide votes are open.
    pub fn is_open_after_reveal(self) -> bool {
        matches!(self, Phase::Revealed | Phase::Deciding)
    }
}

/// The phase a session is in at `now`: a stored `writing` whose deadline has
/// passed is revealed (`now > writing_closes_at`). Both times are RFC 3339;
/// an unreadable deadline counts as passed (a session never stays sealed
/// forever by accident).
pub fn effective_phase(stored: Phase, writing_closes_at: &str, now: &str) -> Phase {
    if stored != Phase::Writing {
        return stored;
    }
    let parse = |s: &str| chrono::DateTime::parse_from_rfc3339(s).ok();
    match (parse(writing_closes_at), parse(now)) {
        (Some(closes), Some(now)) if now <= closes => Phase::Writing,
        (_, None) => Phase::Writing,
        _ => Phase::Revealed,
    }
}

/// Longest card or proposal, in characters.
pub const TEXT_MAX: usize = 280;
/// Cards one member may write per prompt (or one pass instead).
pub const ENTRIES_PER_PROMPT: usize = 3;
/// Decide votes one member has per session.
pub const DECIDE_VOTES: usize = 3;
/// Proposals the facilitator may accept.
pub const ACCEPT_MAX: usize = 3;
/// Default length of a timed silent round, in minutes.
pub const ROUND_MINUTES_DEFAULT: u32 = 3;
/// Longest timed round, in minutes.
pub const ROUND_MINUTES_MAX: u32 = 30;
/// The async window after the match start, in hours.
pub const ASYNC_HOURS: i64 = 24;
/// Closed sessions the quiet nudge looks back on.
pub const QUIET_LOOKBACK: usize = 3;
/// Jaccard similarity of two cards' word sets from which they are suggested
/// as near-duplicates.
pub const CLUSTER_THRESHOLD: f64 = 0.5;

/// A card or proposal text as stored: control characters (line breaks
/// included) become spaces, runs of spaces collapse, the ends are trimmed;
/// 1 to [`TEXT_MAX`] characters. The error is safe to show.
pub fn clean_text(raw: &str) -> Result<String, String> {
    let spaced: String = raw
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let text = spaced.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() {
        return Err("Write something first, or pass.".into());
    }
    if text.chars().count() > TEXT_MAX {
        return Err(format!("A card is at most {TEXT_MAX} characters."));
    }
    Ok(text)
}

/// Short words that say nothing about what a card is about.
const STOP_WORDS: &[&str] = &[
    "the", "and", "for", "but", "not", "you", "our", "was", "were", "are", "with", "too", "had",
    "has", "have", "that", "this", "then", "than", "when", "they", "them", "their", "into", "from",
    "out", "all", "very", "just", "did", "didn", "dont", "don", "its", "it's", "get", "got",
];

/// The words a card is compared by: lowercase alphanumeric runs of three or
/// more characters, without stop words, as a sorted set.
pub fn word_set(text: &str) -> Vec<String> {
    let mut words: Vec<String> = text
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 3 && !STOP_WORDS.contains(w))
        .map(str::to_string)
        .collect();
    words.sort();
    words.dedup();
    words
}

/// Jaccard similarity of two word sets (sorted, deduplicated): shared words
/// over all words; 0 when either is empty.
pub fn jaccard(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let shared = a.iter().filter(|w| b.binary_search(w).is_ok()).count();
    let all = a.len() + b.len() - shared;
    shared as f64 / all as f64
}

/// One card for [`suggest_clusters`]: id, prompt, text, and the cluster it is
/// already in (its root's id, or its own id when it is a root of others).
#[derive(Clone, Debug)]
pub struct ClusterInput {
    pub id: String,
    pub prompt: Prompt,
    pub text: String,
    pub cluster: Option<String>,
}

/// Near-duplicate groups the facilitator may merge: cards of one prompt
/// (never across prompts) linked by a [`jaccard`] similarity of at least
/// [`CLUSTER_THRESHOLD`], joined transitively. Cards already merged
/// together are one card here, so a suggestion always adds something.
/// Groups are listed by prompt order, then by their first id; ids inside a
/// group are in input order. Pure lexical comparison, no model.
pub fn suggest_clusters(cards: &[ClusterInput]) -> Vec<ClusterSuggestion> {
    let mut out = Vec::new();
    for prompt in Prompt::ALL {
        let cards: Vec<&ClusterInput> = cards.iter().filter(|c| c.prompt == prompt).collect();
        let n = cards.len();
        if n < 2 {
            continue;
        }
        let words: Vec<Vec<String>> = cards.iter().map(|c| word_set(&c.text)).collect();
        let group_of = |c: &ClusterInput| c.cluster.clone().unwrap_or_else(|| c.id.clone());
        // Union-find over indexes.
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(parent: &mut [usize], i: usize) -> usize {
            let mut r = i;
            while parent[r] != r {
                r = parent[r];
            }
            let mut j = i;
            while parent[j] != r {
                let next = parent[j];
                parent[j] = r;
                j = next;
            }
            r
        }
        let mut similar = vec![false; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let same_group = group_of(cards[i]) == group_of(cards[j]);
                if same_group || jaccard(&words[i], &words[j]) >= CLUSTER_THRESHOLD {
                    let (a, b) = (find(&mut parent, i), find(&mut parent, j));
                    if a != b {
                        parent[b] = a;
                    }
                    if !same_group {
                        similar[i] = true;
                        similar[j] = true;
                    }
                }
            }
        }
        let mut groups: Vec<(usize, Vec<usize>)> = Vec::new();
        for i in 0..n {
            let r = find(&mut parent, i);
            match groups.iter_mut().find(|(root, _)| *root == r) {
                Some((_, members)) => members.push(i),
                None => groups.push((r, vec![i])),
            }
        }
        for (_, members) in groups {
            // Only groups with a new similarity (not just an existing merge).
            if members.len() < 2 || !members.iter().any(|&i| similar[i]) {
                continue;
            }
            out.push(ClusterSuggestion {
                prompt,
                entry_ids: members.iter().map(|&i| cards[i].id.clone()).collect(),
            });
        }
    }
    out
}

/// The quiet nudge: how many of `members` (user ids) wrote no card (a pass
/// does not count) in each of the last sessions. `sessions` holds, per
/// closed session (newest first), the ids of the users with at least one
/// non-pass card. `None` with fewer than [`QUIET_LOOKBACK`] sessions, and
/// when nobody is quiet. Only the count leaves this function.
pub fn quiet_count(members: &[String], sessions: &[Vec<String>]) -> Option<u8> {
    if sessions.len() < QUIET_LOOKBACK {
        return None;
    }
    let last = &sessions[..QUIET_LOOKBACK];
    let quiet = members
        .iter()
        .filter(|m| last.iter().all(|wrote| !wrote.contains(m)))
        .count();
    (quiet > 0).then(|| quiet.min(u8::MAX as usize) as u8)
}

/// The nudge line for a count (the page renders it, muteable).
pub fn quiet_text(count: u8) -> String {
    match count {
        1 => "One player has been quiet in the last 3 reviews.".to_string(),
        2 => "Two players have been quiet in the last 3 reviews.".to_string(),
        n => format!("{n} players have been quiet in the last 3 reviews."),
    }
}

/// Which proposals are suggested for acceptance: the top [`ACCEPT_MAX`] by
/// decide votes (at least one vote), ties by id. `proposals` is (id, votes).
pub fn suggested_accepts(proposals: &[(String, u32)]) -> Vec<String> {
    let mut sorted: Vec<&(String, u32)> = proposals.iter().filter(|(_, v)| *v > 0).collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sorted
        .into_iter()
        .take(ACCEPT_MAX)
        .map(|(id, _)| id.clone())
        .collect()
}

// ---------------------------------------------------------------------------
// The state payload
// ---------------------------------------------------------------------------

/// What `get_session_state` answers. `Unchanged` when the caller's `rev` is
/// the session's: only the presence list (presence is not part of `rev`, so
/// people coming and going never force a full payload).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionPoll {
    Unchanged { rev: i64, present: Vec<String> },
    Full(Box<SessionState>),
}

/// One session as one member sees it. Before the reveal it holds the
/// member's own cards only, and two team-level counts; after it, every
/// card grouped by prompt, the anonymous ones without an author.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionState {
    pub id: String,
    /// Compare with the next poll's answer.
    pub rev: i64,
    pub phase: Phase,
    /// The scheduled match (None when it was deleted).
    pub match_id: Option<String>,
    pub match_deleted: bool,
    /// The game of the match (1 for a Bo1).
    pub game_number: i32,
    pub series_id: Option<String>,
    /// When writing ends (RFC 3339, UTC), and the server's clock now, so a
    /// timer can count down without trusting the browser's clock.
    pub writing_closes_at: String,
    pub server_now: String,
    /// A timed (sync) round is running: the facilitator started it.
    pub timed_round: bool,
    pub facilitator_name: String,
    pub i_am_facilitator: bool,
    /// The viewer leads or coaches the team and is not the facilitator.
    pub can_take_over: bool,
    /// The generated review, once closed.
    pub review_id: Option<String>,
    /// Members with at least one card or pass, over all four prompts (never
    /// per prompt), and the team's current members. The facilitator is a
    /// member like everyone else.
    pub written_count: u32,
    pub member_count: u32,
    /// The viewer's own cards and passes, while writing only (empty after
    /// the reveal, so no "yours" can mark an anonymous card).
    pub mine: Vec<OwnEntry>,
    /// Every card, after the reveal.
    pub cards: Option<RevealedCards>,
    pub proposals: Vec<ProposalView>,
    /// Decide votes the viewer has left.
    pub decide_votes_left: u32,
    /// For the facilitator while writing: how many players were quiet in
    /// the last 3 closed reviews. A count only, never who.
    pub quiet: Option<u8>,
    /// After close: members with a card, of members at close.
    pub voices: Option<Voices>,
    /// Who has the session open now (usernames, alphabetical). Nothing
    /// about who has written.
    pub present: Vec<String>,
}

/// One of the viewer's own cards before the reveal (a pass has no text).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnEntry {
    pub id: String,
    pub prompt: Prompt,
    pub text: String,
    pub passed: bool,
}

/// The revealed cards, grouped by prompt, each group in `order_key` order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RevealedCards {
    pub decided: Vec<AttributedCard>,
    pub to_plan: Vec<AttributedCard>,
    /// "What didn't go to plan": no author, ever.
    pub off_plan: Vec<AnonymousCard>,
    pub next_time: Vec<AttributedCard>,
    /// Near-duplicates the facilitator may merge (facilitator only).
    pub suggestions: Vec<ClusterSuggestion>,
}

/// A card with its author's name (resolved at read time; "Former member"
/// after an account deletion).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttributedCard {
    pub id: String,
    pub text: String,
    pub author_name: String,
    pub plus_ones: u32,
    /// The viewer +1'd it (per viewer; says nothing about anyone else).
    pub i_plus_oned: bool,
    /// The card it is merged under (that card's id), if any.
    pub cluster: Option<String>,
}

/// A card of the anonymous prompt. There is deliberately no author field,
/// no "mine" flag and no time: nothing to serialize them from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnonymousCard {
    pub id: String,
    pub text: String,
    pub plus_ones: u32,
    pub i_plus_oned: bool,
    pub cluster: Option<String>,
}

/// A near-duplicate group within one prompt.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClusterSuggestion {
    pub prompt: Prompt,
    pub entry_ids: Vec<String>,
}

/// An action-item proposal. No proposer is stored or shown.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProposalView {
    pub id: String,
    pub text: String,
    /// The card it was made from (its id is already on screen).
    pub source: Option<String>,
    pub votes: u32,
    pub i_voted: bool,
    pub accepted: bool,
    /// Among the top three by votes (the default to accept).
    pub suggested: bool,
    /// A current member's username.
    pub owner: Option<String>,
}

/// "5 of 5 voices": members with at least one card, of members at close.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Voices {
    pub voices: u32,
    pub members: u32,
}

impl Voices {
    pub fn label(self) -> String {
        format!("{} of {} voices", self.voices, self.members)
    }
}

/// What the review page shows of the session that generated it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReviewSessionView {
    pub session_id: String,
    pub cards: RevealedCards,
    /// The accepted items with their owners, as decided.
    pub accepted: Vec<AcceptedItem>,
    pub voices: Option<Voices>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AcceptedItem {
    pub text: String,
    pub owner: Option<String>,
}

/// The closed sessions' texts "Recurring issues" counts with the reviews
/// (texts only, no authors).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SessionPatternTexts {
    /// The generated review, so its lines are not counted twice.
    pub review_id: Option<String>,
    pub to_plan: Vec<String>,
    pub off_plan: Vec<String>,
}

/// One game's session on the match thread.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatchSessionLink {
    pub game_number: i32,
    pub session_id: String,
    pub phase: Phase,
}

/// What `open_session` answers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpenedSession {
    pub session_id: String,
    /// False when the game already had a session (it is returned).
    pub created: bool,
}

/// What the session page loads first (`get_session_page`): the session's
/// id (opened from a match thread link when the address named a match),
/// the page title, the team's current members' usernames (owner chips),
/// and the viewer's full state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionPage {
    pub session_id: String,
    /// "Scrim vs Riverside Rooks" (+ " \u{b7} G2" for a series game).
    pub title: String,
    pub members: Vec<String>,
    pub state: SessionState,
}

/// The rows a revealed group renders, in the group's order: each row holds
/// the indexes of its cards. A merged cluster is one row, placed where its
/// first card stands, with its cards in the group's order; every other card
/// is a row of its own. `cards` is (id, cluster) per card.
pub fn note_rows(cards: &[(String, Option<String>)]) -> Vec<Vec<usize>> {
    let mut rows: Vec<(String, Vec<usize>)> = Vec::new();
    for (i, (id, cluster)) in cards.iter().enumerate() {
        let root = cluster.clone().unwrap_or_else(|| id.clone());
        match rows.iter_mut().find(|(r, _)| *r == root) {
            Some((_, members)) => members.push(i),
            None => rows.push((root, vec![i])),
        }
    }
    rows.into_iter().map(|(_, m)| m).collect()
}

/// The card a merged row's one +1 counts on: the cluster's root when it is
/// in the row, else the row's first card.
pub fn row_root(cards: &[(String, Option<String>)], row: &[usize]) -> usize {
    let first = row.first().copied().unwrap_or(0);
    let Some(root) = cards
        .get(first)
        .map(|(id, c)| c.clone().unwrap_or_else(|| id.clone()))
    else {
        return first;
    };
    row.iter()
        .copied()
        .find(|&i| cards.get(i).is_some_and(|(id, _)| *id == root))
        .unwrap_or(first)
}

/// Minutes left on a timed round, for the calm timer: `None` once it is
/// under a minute ("under a minute left"), else whole minutes rounded up.
pub fn minutes_left(remaining_ms: i64) -> Option<i64> {
    (remaining_ms > 60_000).then(|| (remaining_ms + 59_999) / 60_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompts_round_trip_and_only_off_plan_is_anonymous() {
        for p in Prompt::ALL {
            assert_eq!(Prompt::from_key(p.key()), Some(p));
            assert_eq!(p.is_anonymous(), p == Prompt::OffPlan);
        }
        assert_eq!(Prompt::from_key("nope"), None);
        for p in [
            Phase::Writing,
            Phase::Revealed,
            Phase::Deciding,
            Phase::Closed,
        ] {
            assert_eq!(Phase::from_key(p.key()), Some(p));
        }
    }

    #[test]
    fn writing_turns_revealed_after_the_deadline_at_read_time() {
        let closes = "2026-11-13T18:00:00Z";
        assert_eq!(
            effective_phase(Phase::Writing, closes, "2026-11-13T17:59:59Z"),
            Phase::Writing
        );
        assert_eq!(
            effective_phase(Phase::Writing, closes, "2026-11-13T18:00:00Z"),
            Phase::Writing
        );
        assert_eq!(
            effective_phase(Phase::Writing, closes, "2026-11-13T18:00:01Z"),
            Phase::Revealed
        );
        // Other phases never move by time.
        assert_eq!(
            effective_phase(Phase::Deciding, closes, "2030-01-01T00:00:00Z"),
            Phase::Deciding
        );
        assert_eq!(
            effective_phase(Phase::Closed, closes, "2020-01-01T00:00:00Z"),
            Phase::Closed
        );
        // An unreadable deadline is passed.
        assert_eq!(
            effective_phase(Phase::Writing, "garbage", "2026-11-13T18:00:00Z"),
            Phase::Revealed
        );
    }

    #[test]
    fn card_text_is_trimmed_flattened_and_capped() {
        assert_eq!(clean_text("  we  won\nlane  ").unwrap(), "we won lane");
        assert!(clean_text("   \n\t ").is_err());
        assert!(clean_text(&"a".repeat(TEXT_MAX)).is_ok());
        assert!(clean_text(&"a".repeat(TEXT_MAX + 1)).is_err());
        // Characters, not bytes.
        assert!(clean_text(&"\u{e9}".repeat(TEXT_MAX)).is_ok());
    }

    fn card(id: &str, prompt: Prompt, text: &str) -> ClusterInput {
        ClusterInput {
            id: id.into(),
            prompt,
            text: text.into(),
            cluster: None,
        }
    }

    #[test]
    fn near_duplicates_cluster_within_a_prompt_only() {
        let cards = vec![
            card("a", Prompt::OffPlan, "We lost dragon control at 20 minutes"),
            card("b", Prompt::OffPlan, "lost dragon control"),
            card("c", Prompt::OffPlan, "Bot lane got dived twice"),
            card("d", Prompt::NextTime, "dragon control"),
            card("e", Prompt::OffPlan, "Dragon control lost, again"),
        ];
        let s = suggest_clusters(&cards);
        assert_eq!(s.len(), 1, "{s:?}");
        assert_eq!(s[0].prompt, Prompt::OffPlan);
        assert_eq!(s[0].entry_ids, vec!["a", "b", "e"]);
    }

    #[test]
    fn jaccard_threshold_is_what_the_rule_says() {
        let a = word_set("Ward the river before dragon");
        let b = word_set("ward river before baron");
        // {ward, river, before, dragon} vs {ward, river, before, baron}: 3/5.
        assert!((jaccard(&a, &b) - 0.6).abs() < 1e-9);
        assert!(jaccard(&a, &b) >= CLUSTER_THRESHOLD);
        let c = word_set("ward baron");
        // {ward, baron} vs {ward, river, before, dragon}: 1/5.
        assert!(jaccard(&a, &c) < CLUSTER_THRESHOLD);
        assert_eq!(jaccard(&[], &a), 0.0);
        // Stop words and short words do not count.
        assert_eq!(word_set("the and of it we"), Vec::<String>::new());
    }

    #[test]
    fn merged_cards_are_not_suggested_again() {
        let mut cards = vec![
            card("a", Prompt::ToPlan, "early dragon control"),
            card("b", Prompt::ToPlan, "dragon control early"),
        ];
        cards[1].cluster = Some("a".into());
        assert!(suggest_clusters(&cards).is_empty());
        // A third similar card joins the merged pair.
        cards.push(card("c", Prompt::ToPlan, "early dragon control again"));
        let s = suggest_clusters(&cards);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].entry_ids, vec!["a", "b", "c"]);
    }

    #[test]
    fn the_quiet_nudge_is_a_count_and_needs_three_sessions() {
        let members: Vec<String> = ["u1", "u2", "u3"].iter().map(|s| s.to_string()).collect();
        let s = |ids: &[&str]| ids.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        // Fewer than three closed sessions: nothing.
        assert_eq!(quiet_count(&members, &[s(&["u1"]), s(&["u1"])]), None);
        // u3 wrote in none of the last three; u2 in one.
        let sessions = vec![s(&["u1"]), s(&["u1", "u2"]), s(&["u1"]), s(&[])];
        assert_eq!(quiet_count(&members, &sessions), Some(1));
        // Only the last three count: u2 wrote in the fourth-newest only.
        let sessions = vec![s(&["u1"]), s(&["u1"]), s(&["u1"]), s(&["u2", "u3"])];
        assert_eq!(quiet_count(&members, &sessions), Some(2));
        // Everyone spoke: no nudge.
        let all = s(&["u1", "u2", "u3"]);
        assert_eq!(
            quiet_count(&members, &[all.clone(), s(&["u2", "u3", "u1"]), all]),
            None
        );
        assert_eq!(
            quiet_text(1),
            "One player has been quiet in the last 3 reviews."
        );
        assert_eq!(
            quiet_text(2),
            "Two players have been quiet in the last 3 reviews."
        );
    }

    #[test]
    fn suggested_accepts_follow_vote_order_up_to_three() {
        let p = |id: &str, v: u32| (id.to_string(), v);
        let got = suggested_accepts(&[p("d", 1), p("a", 3), p("c", 0), p("b", 3), p("e", 2)]);
        assert_eq!(got, vec!["a", "b", "e"]);
        assert!(suggested_accepts(&[p("a", 0)]).is_empty());
    }

    #[test]
    fn merged_cards_become_one_row_where_the_first_stands() {
        let c = |id: &str, cl: Option<&str>| (id.to_string(), cl.map(str::to_string));
        let cards = vec![
            c("x", None),
            c("b", Some("a")),
            c("y", None),
            c("a", None),
            c("z", Some("y")),
        ];
        assert_eq!(note_rows(&cards), vec![vec![0], vec![1, 3], vec![2, 4]]);
        assert_eq!(row_root(&cards, &[1, 3]), 3);
        assert_eq!(row_root(&cards, &[2, 4]), 2);
        assert!(note_rows(&[]).is_empty());
    }

    #[test]
    fn the_timer_counts_whole_minutes() {
        assert_eq!(minutes_left(180_000), Some(3));
        assert_eq!(minutes_left(150_000), Some(3));
        assert_eq!(minutes_left(61_000), Some(2));
        assert_eq!(minutes_left(60_000), None);
        assert_eq!(minutes_left(0), None);
        assert_eq!(minutes_left(-5), None);
    }

    #[test]
    fn voices_read_as_one_team_line() {
        assert_eq!(
            Voices {
                voices: 4,
                members: 5
            }
            .label(),
            "4 of 5 voices"
        );
    }
}
