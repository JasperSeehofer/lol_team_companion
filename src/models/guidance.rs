//! Guidance (M9): the welcome moment, the getting-started checklist,
//! first-use hints, next-step nudges and new-feature introductions.
//!
//! Everything here is a pure function of facts the server read from the
//! user's and the team's own records, plus the user's guidance rows (what
//! they have seen or muted). A step counts as done only when the data says
//! so; every piece of guidance has a key the user can mute, and one switch
//! (`user.guidance_enabled`) turns all of it off. The rules and their mute
//! keys are listed in `docs/DESIGN.md` section 6.11.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// The welcome panel on Match Day.
pub const WELCOME: &str = "welcome";
/// The getting-started checklist ("Hide this checklist" mutes it).
pub const CHECKLIST: &str = "checklist";
/// Prefix of new-feature introduction keys: `intro:<entry slug>:<route>`.
pub const INTRO_PREFIX: &str = "intro:";
/// Prefix of first-use hint keys.
pub const HINT_PREFIX: &str = "hint.";
/// Prefix of nudge keys.
pub const NUDGE_PREFIX: &str = "nudge.";

/// The welcome shows to accounts younger than this (so nobody who used
/// the app before guidance existed gets a welcome).
pub const WELCOME_WINDOW_DAYS: i64 = 14;
/// "Picking up again?" after this many days without a write.
pub const RETURNING_AFTER_DAYS: i64 = 14;
/// "Recent" for dossiers, drafts and past matches.
pub const RECENT_DAYS: i64 = 21;
/// Open action items count as old after this many days.
pub const OPEN_ITEM_DAYS: i64 = 7;
/// Introductions older than this are not shown any more.
pub const INTRO_WINDOW_DAYS: i64 = 180;
/// Longest key accepted from the browser.
pub const MAX_KEY_LEN: usize = 160;

// ---------------------------------------------------------------------------
// Guidance rows
// ---------------------------------------------------------------------------

/// One `user_guidance` row: the user saw (or used) it, or muted it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuidanceRow {
    pub key: String,
    pub seen: bool,
    pub muted: bool,
}

/// Whether `key` is seen or muted in `rows`.
pub fn is_done(rows: &[GuidanceRow], key: &str) -> bool {
    rows.iter().any(|r| r.key == key && (r.seen || r.muted))
}

/// Whether `key` is muted in `rows`.
pub fn is_muted(rows: &[GuidanceRow], key: &str) -> bool {
    rows.iter().any(|r| r.key == key && r.muted)
}

// ---------------------------------------------------------------------------
// First-use hints
// ---------------------------------------------------------------------------

/// A first-use hint: one per feature, shown inline where the feature is on
/// screen while the user has not used it and has not said "Got it".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hint {
    /// The draft board's keyboard keys.
    DraftKeys,
    /// The Draft Graph: click a candidate to see why.
    DraftGraph,
    /// The draft tree: branch from any node.
    TreeBranching,
    /// Share links on plans, reviews and player cards.
    ShareLinks,
    /// Team notes: edits and deletes can be undone.
    NoteUndo,
    /// Champion pool tiers.
    PoolTiers,
    /// The command bar (Ctrl K), a quiet line in the navigation.
    CommandBar,
}

impl Hint {
    /// Page hints first (in their pages' priority order), the navigation
    /// hint last: it shows only where no page hint does.
    pub const ALL: [Hint; 7] = [
        Hint::DraftKeys,
        Hint::DraftGraph,
        Hint::TreeBranching,
        Hint::ShareLinks,
        Hint::NoteUndo,
        Hint::PoolTiers,
        Hint::CommandBar,
    ];

    /// The mute key.
    pub fn key(self) -> &'static str {
        match self {
            Hint::DraftKeys => "hint.draft_keys",
            Hint::DraftGraph => "hint.draft_graph",
            Hint::TreeBranching => "hint.tree_branching",
            Hint::ShareLinks => "hint.share_links",
            Hint::NoteUndo => "hint.note_undo",
            Hint::PoolTiers => "hint.pool_tiers",
            Hint::CommandBar => "hint.command_bar",
        }
    }

    pub fn from_key(key: &str) -> Option<Hint> {
        Hint::ALL.into_iter().find(|h| h.key() == key)
    }

    /// The pages the hint appears on (prefix match for `/team/player/`).
    /// Empty for the navigation hint, which belongs to no page.
    pub fn routes(self) -> &'static [&'static str] {
        match self {
            Hint::DraftKeys | Hint::DraftGraph => &["/draft"],
            Hint::TreeBranching => &["/tree-drafter"],
            Hint::ShareLinks => &["/game-plan", "/post-game", "/team/player/"],
            Hint::NoteUndo => &["/team/dashboard"],
            Hint::PoolTiers => &["/champion-pool"],
            Hint::CommandBar => &[],
        }
    }

    pub fn on_route(self, path: &str) -> bool {
        self.routes().iter().any(|r| {
            if r.ends_with('/') {
                path.starts_with(r)
            } else {
                path == *r
            }
        })
    }
}

/// Which hinted features the data shows as used already (server-computed;
/// the Draft Graph and the command bar have no record of their own, so
/// using them writes the hint's row from the browser instead).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HintUse {
    /// The user created a draft.
    pub drafted: bool,
    /// The team's draft tree has a branch (a node with a parent).
    pub branched: bool,
    /// The user made a share link.
    pub shared: bool,
    /// A team note was edited or deleted (the team knows about Undo).
    pub note_changed: bool,
    /// The user moved a champion out of the default tier.
    pub tiered: bool,
}

impl HintUse {
    fn used(&self, h: Hint) -> bool {
        match h {
            Hint::DraftKeys => self.drafted,
            Hint::TreeBranching => self.branched,
            Hint::ShareLinks => self.shared,
            Hint::NoteUndo => self.note_changed,
            Hint::PoolTiers => self.tiered,
            Hint::DraftGraph | Hint::CommandBar => false,
        }
    }
}

/// The hints still to show, in priority order.
pub fn active_hints(enabled: bool, rows: &[GuidanceRow], used: &HintUse) -> Vec<Hint> {
    if !enabled {
        return Vec::new();
    }
    Hint::ALL
        .into_iter()
        .filter(|h| !used.used(*h) && !is_done(rows, h.key()))
        .collect()
}

/// Pages that never show guidance: admin pages, the gallery, sign-in and
/// registration, legal pages and public share links.
pub fn is_quiet_path(path: &str) -> bool {
    path == "/admin"
        || path.starts_with("/admin/")
        || path == "/design"
        || path.starts_with("/auth/")
        || path.starts_with("/legal/")
        || path.starts_with("/s/")
        || path == "/closed-beta"
}

/// Whether a signed-in load of `path` counts as a visit for "Picking up
/// again?" (`user.last_seen_at`). Public pages do not: share links, legal
/// and sign-in pages, the closed-beta page, changelog and transparency
/// (M10 review).
pub fn counts_as_visit(path: &str) -> bool {
    !(path.starts_with("/s/")
        || path.starts_with("/legal/")
        || path.starts_with("/auth/")
        || path == "/closed-beta"
        || path == "/changelog"
        || path == "/transparency")
}

/// The one page hint for `path`: the first active hint that lives there.
/// An introduction on the page wins over hints (never more than one).
pub fn page_hint(state: &GuidanceState, path: &str) -> Option<Hint> {
    if !state.enabled || is_quiet_path(path) || intro_for(state, path).is_some() {
        return None;
    }
    state.hints.iter().copied().find(|h| h.on_route(path))
}

/// Whether the navigation's command-bar line shows on `path`: only when
/// nothing else guides on the page (and not on Match Day, which has its own
/// welcome and checklist).
pub fn nav_hint(state: &GuidanceState, path: &str) -> bool {
    state.enabled
        && path != "/"
        && !is_quiet_path(path)
        && intro_for(state, path).is_none()
        && !state
            .hints
            .iter()
            .any(|h| *h != Hint::CommandBar && h.on_route(path))
        && state.hints.contains(&Hint::CommandBar)
}

// ---------------------------------------------------------------------------
// New-feature introductions
// ---------------------------------------------------------------------------

/// A new-feature introduction from a changelog entry's `Introduce:` line.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intro {
    /// `intro:<slug>:<route>`.
    pub key: String,
    pub route: String,
    /// "Draft Graph".
    pub feature: String,
    /// How to try it, one sentence.
    pub text: String,
    /// The changelog entry (`/changelog#<slug>`).
    pub slug: String,
    /// The entry's date, `YYYY-MM-DD`.
    pub date: String,
}

pub fn intro_key(slug: &str, route: &str) -> String {
    format!("{INTRO_PREFIX}{slug}:{route}")
}

/// Introductions this user should still see: the entry is dated after the
/// account was created (people who joined later meet the feature through
/// the welcome and the hints instead), is at most `INTRO_WINDOW_DAYS` old,
/// and the user has not seen or muted it. Newest first.
pub fn eligible_intros(
    enabled: bool,
    all: &[Intro],
    rows: &[GuidanceRow],
    account_created: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Vec<Intro> {
    if !enabled {
        return Vec::new();
    }
    let Some(created) = account_created else {
        return Vec::new();
    };
    let mut out: Vec<Intro> = all
        .iter()
        .filter(|i| {
            let Some(day) = NaiveDate::parse_from_str(&i.date, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|d| d.and_utc())
            else {
                return false;
            };
            created < day
                && day <= now
                && now - day <= Duration::days(INTRO_WINDOW_DAYS)
                && !is_done(rows, &i.key)
        })
        .cloned()
        .collect();
    out.sort_by(|a, b| b.date.cmp(&a.date));
    out
}

/// The introduction to show on `path` (the newest unseen one).
pub fn intro_for<'a>(state: &'a GuidanceState, path: &str) -> Option<&'a Intro> {
    if !state.enabled || is_quiet_path(path) {
        return None;
    }
    state.intros.iter().find(|i| i.route == path)
}

/// Routes that carry a "New" mark in the navigation.
pub fn new_routes(state: &GuidanceState) -> Vec<String> {
    if !state.enabled {
        return Vec::new();
    }
    let mut out: Vec<String> = state.intros.iter().map(|i| i.route.clone()).collect();
    out.dedup();
    out
}

/// What the app shell needs on every page: the switch, the hints still to
/// show and the introductions still to see.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuidanceState {
    pub enabled: bool,
    pub hints: Vec<Hint>,
    pub intros: Vec<Intro>,
}

// ---------------------------------------------------------------------------
// Getting-started checklist
// ---------------------------------------------------------------------------

/// What the checklist is computed from (team and user records).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChecklistFacts {
    pub team_name: Option<String>,
    /// Distinct positions (top, jungle, mid, bot, support) with a starter.
    pub starting_positions: usize,
    pub opponents: usize,
    pub drafts: usize,
    pub plans: usize,
    pub reviews: usize,
    pub members: usize,
    /// Join requests waiting for the leader (a teammate is on the way).
    pub pending_requests: usize,
}

/// One checklist step, ready for `PrepSequence`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub title: String,
    pub detail: String,
    pub status: String,
    pub done: bool,
    /// Where it is done; `None` while an earlier step blocks it.
    pub href: Option<String>,
    pub action: String,
}

fn item(
    title: &str,
    detail: impl Into<String>,
    done: bool,
    status_done: String,
    href: Option<&str>,
    action: &str,
) -> ChecklistItem {
    ChecklistItem {
        title: title.to_string(),
        detail: detail.into(),
        status: if done {
            status_done
        } else if href.is_some() {
            "To do".to_string()
        } else {
            "Needs a team".to_string()
        },
        done,
        href: href.map(str::to_string),
        action: action.to_string(),
    }
}

fn count(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// The seven getting-started steps. Each is done only when the records
/// show it; a user who joins an established team finds the team's steps
/// ticked already.
pub fn checklist(f: &ChecklistFacts) -> Vec<ChecklistItem> {
    let team = f.team_name.is_some();
    let t = |href: &'static str| if team { Some(href) } else { None };
    vec![
        item(
            "Create or join your team",
            match &f.team_name {
                Some(_) => "The steps below count what your team has done.".to_string(),
                None => "Create a team, or use your team's invite link, or find your team and ask to join.".to_string(),
            },
            team,
            "Done".to_string(),
            Some("/team/dashboard"),
            "Create or join your team",
        ),
        item(
            "Invite a teammate",
            if f.members > 1 {
                format!("{} on the team.", count(f.members, "member", "members"))
            } else if f.pending_requests > 0 {
                "A join request is waiting on the team page.".to_string()
            } else {
                "The team leader or a coach sends an invite link from the team page. Teammates who already have an account find your team on the roster page, then ask to join."
                    .to_string()
            },
            f.members > 1 || f.pending_requests > 0,
            if f.members > 1 {
                count(f.members, "member", "members")
            } else {
                "Request waiting".to_string()
            },
            t("/team/dashboard#joining-h"),
            "How teammates join",
        ),
        item(
            "Fill the starting five",
            if f.starting_positions >= 5 {
                "Top, jungle, mid, bot and support each have a starter.".to_string()
            } else {
                format!(
                    "{} of 5 positions have a starter. The team leader places players on the team page.",
                    f.starting_positions
                )
            },
            f.starting_positions >= 5,
            "5 of 5".to_string(),
            t("/team/dashboard"),
            "Place the starters",
        ),
        item(
            "Add a first opponent dossier",
            "Scout who you play next: players, champions, notes.",
            f.opponents > 0,
            count(f.opponents, "dossier", "dossiers"),
            t("/opponents"),
            "Add a dossier",
        ),
        item(
            "Make a draft",
            "Plan picks and bans; the Draft Graph explains each pick.",
            f.drafts > 0,
            count(f.drafts, "draft", "drafts"),
            t("/draft"),
            "Start a draft",
        ),
        item(
            "Write a game plan",
            "Turn a draft into win conditions and a plan per lane.",
            f.plans > 0,
            count(f.plans, "plan", "plans"),
            t("/game-plan"),
            "Write a game plan",
        ),
        item(
            "Review a game",
            "Right after the game: what went well, what to fix, action items.",
            f.reviews > 0,
            count(f.reviews, "review", "reviews"),
            t("/post-game"),
            "Write a review",
        ),
    ]
}

/// Index of the step to do now: the first open step with somewhere to go.
pub fn next_step(items: &[ChecklistItem]) -> Option<usize> {
    items.iter().position(|i| !i.done && i.href.is_some())
}

/// Whether every step is done.
pub fn checklist_complete(items: &[ChecklistItem]) -> bool {
    items.iter().all(|i| i.done)
}

/// Whether the welcome shows: guidance on, a young account, not yet started
/// or skipped.
pub fn welcome_due(
    enabled: bool,
    rows: &[GuidanceRow],
    account_created: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> bool {
    enabled
        && !is_done(rows, WELCOME)
        && account_created.is_some_and(|c| now - c <= Duration::days(WELCOME_WINDOW_DAYS))
}

/// Whether the checklist shows: guidance on, not hidden, not complete, or
/// asked for with "Guide me" (which shows it even when hidden or complete).
pub fn checklist_due(
    enabled: bool,
    rows: &[GuidanceRow],
    items: &[ChecklistItem],
    forced: bool,
) -> bool {
    forced || (enabled && !is_muted(rows, CHECKLIST) && !checklist_complete(items))
}

// ---------------------------------------------------------------------------
// Nudges
// ---------------------------------------------------------------------------

/// The kinds of next-step nudge, in priority order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NudgeKind {
    /// No write by the user for `RETURNING_AFTER_DAYS`.
    Returning,
    /// A match in the last `RECENT_DAYS` had a game plan and has no review.
    MatchWithoutReview,
    /// Open action items older than `OPEN_ITEM_DAYS`.
    OldActionItems,
    /// A recent draft has no game plan.
    DraftWithoutPlan,
    /// A recent dossier has no draft against that opponent.
    DossierWithoutDraft,
}

impl NudgeKind {
    pub const ALL: [NudgeKind; 5] = [
        NudgeKind::Returning,
        NudgeKind::MatchWithoutReview,
        NudgeKind::OldActionItems,
        NudgeKind::DraftWithoutPlan,
        NudgeKind::DossierWithoutDraft,
    ];

    /// The mute key ("Don't show this again" stores it).
    pub fn key(self) -> &'static str {
        match self {
            NudgeKind::Returning => "nudge.returning",
            NudgeKind::MatchWithoutReview => "nudge.match_without_review",
            NudgeKind::OldActionItems => "nudge.old_action_items",
            NudgeKind::DraftWithoutPlan => "nudge.draft_without_plan",
            NudgeKind::DossierWithoutDraft => "nudge.dossier_without_draft",
        }
    }

    pub fn from_key(key: &str) -> Option<NudgeKind> {
        NudgeKind::ALL.into_iter().find(|k| k.key() == key)
    }
}

/// One quiet line under the prep sequence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nudge {
    pub kind: NudgeKind,
    pub text: String,
    pub action: String,
    pub href: String,
}

/// A past match with a game plan and no review.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PastMatch {
    pub opponent: String,
    pub kind_label: String,
    pub plan_id: String,
}

/// What the nudge rules read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NudgeFacts {
    pub now: DateTime<Utc>,
    /// The newest record the user wrote (drafts, plans, reviews, trees,
    /// notes, learnings, pool, the next match ...).
    pub last_write: Option<DateTime<Utc>>,
    /// The visit before this one (`user.prev_seen_at`): reading counts too.
    pub prev_seen: Option<DateTime<Utc>>,
    pub account_created: Option<DateTime<Utc>>,
    pub match_without_review: Option<PastMatch>,
    /// Open action items older than `OPEN_ITEM_DAYS`.
    pub old_open_items: usize,
    /// (draft id, name) of the newest recent draft without a plan.
    pub draft_without_plan: Option<(String, String)>,
    /// (opponent id, name) of a recent dossier without a recent draft.
    pub dossier_without_draft: Option<(String, String)>,
    /// Whether the viewer is the team leader or a coach (can manage drafts/plans).
    pub viewer_manages: bool,
}

/// Whether the user has been away: neither a write nor a visit for
/// `RETURNING_AFTER_DAYS` before this visit (polish pass: reading counts).
/// Without either, the account's age decides.
pub fn is_returning(f: &NudgeFacts) -> bool {
    let since = match (f.last_write, f.prev_seen) {
        (Some(w), Some(s)) => Some(w.max(s)),
        (w, s) => w.or(s).or(f.account_created),
    };
    since.is_some_and(|t| f.now - t >= Duration::days(RETURNING_AFTER_DAYS))
}

fn candidate(kind: NudgeKind, f: &NudgeFacts) -> Option<Nudge> {
    let n = |text: String, action: &str, href: String| Nudge {
        kind,
        text,
        action: action.to_string(),
        href,
    };
    match kind {
        NudgeKind::Returning => is_returning(f).then(|| {
            n(
                "Picking up again? Here is the loop: scout, draft, plan, play, review.".into(),
                "Open the checklist",
                "/?guide=1#getting-started".into(),
            )
        }),
        NudgeKind::MatchWithoutReview => f.match_without_review.as_ref().map(|m| {
            n(
                format!(
                    "The {} against {} had a game plan and has no review yet.",
                    m.kind_label.to_lowercase(),
                    m.opponent
                ),
                "Write the review",
                format!("/post-game?plan_id={}", m.plan_id),
            )
        }),
        NudgeKind::OldActionItems => (f.old_open_items > 0).then(|| {
            n(
                if f.old_open_items == 1 {
                    "One action item from a review has been open for more than a week.".into()
                } else {
                    format!(
                        "{} action items from reviews have been open for more than a week.",
                        f.old_open_items
                    )
                },
                "Go through them",
                "/action-items".into(),
            )
        }),
        NudgeKind::DraftWithoutPlan => {
            if !f.viewer_manages {
                return None;
            }
            f.draft_without_plan.as_ref().map(|(id, name)| {
                n(
                    format!("\u{201c}{name}\u{201d} has no game plan yet."),
                    "Write the game plan",
                    format!("/game-plan?draft_id={id}"),
                )
            })
        }
        NudgeKind::DossierWithoutDraft => {
            if !f.viewer_manages {
                return None;
            }
            f.dossier_without_draft.as_ref().map(|(_, name)| {
                n(
                    format!(
                        "You scouted {name}, but there is no draft against them in the last {} weeks.",
                        RECENT_DAYS / 7
                    ),
                    "Start a draft",
                    "/draft".into(),
                )
            })
        }
    }
}

/// The one nudge to show: the first rule that applies, is not muted and
/// does not repeat what the prep sequence already asks for (`skip_href`,
/// its next step's link).
pub fn pick_nudge(
    enabled: bool,
    f: &NudgeFacts,
    rows: &[GuidanceRow],
    skip_href: Option<&str>,
) -> Option<Nudge> {
    if !enabled {
        return None;
    }
    NudgeKind::ALL
        .into_iter()
        .filter(|k| !is_muted(rows, k.key()))
        .filter_map(|k| candidate(k, f))
        .find(|n| Some(n.href.as_str()) != skip_href)
}

// ---------------------------------------------------------------------------
// Match Day
// ---------------------------------------------------------------------------

/// The welcome panel's facts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Welcome {
    pub username: String,
    pub team_name: Option<String>,
    pub riot_linked: bool,
    /// Where "Start" goes: the first open checklist step.
    pub start_href: String,
}

/// Everything Match Day shows for guidance.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomeGuide {
    pub welcome: Option<Welcome>,
    /// The checklist while it is open (or asked for with "Guide me").
    pub checklist: Option<Vec<ChecklistItem>>,
    /// The checklist was hidden or complete and "Guide me" asked for it.
    pub forced: bool,
    pub nudge: Option<Nudge>,
}

/// Whether the browser may write `key` (anything else is refused).
pub fn is_known_key(key: &str, intros: &[Intro]) -> bool {
    key.len() <= MAX_KEY_LEN
        && (key == WELCOME
            || key == CHECKLIST
            || Hint::from_key(key).is_some()
            || NudgeKind::from_key(key).is_some()
            || intros.iter().any(|i| i.key == key))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    fn row(key: &str, seen: bool, muted: bool) -> GuidanceRow {
        GuidanceRow {
            key: key.into(),
            seen,
            muted,
        }
    }

    fn states(items: &[ChecklistItem]) -> Vec<bool> {
        items.iter().map(|i| i.done).collect()
    }

    #[test]
    fn a_new_user_starts_at_the_team_step_and_later_steps_wait() {
        let items = checklist(&ChecklistFacts::default());
        assert_eq!(items.len(), 7);
        assert!(items.iter().all(|i| !i.done));
        assert_eq!(next_step(&items), Some(0));
        assert_eq!(items[0].href.as_deref(), Some("/team/dashboard"));
        assert!(items[1..].iter().all(|i| i.href.is_none()));
        assert_eq!(items[3].status, "Needs a team");
    }

    #[test]
    fn steps_tick_themselves_from_the_records() {
        let mut f = ChecklistFacts {
            team_name: Some("Northgate".into()),
            members: 1,
            ..Default::default()
        };
        let items = checklist(&f);
        assert_eq!(
            states(&items),
            [true, false, false, false, false, false, false]
        );
        assert_eq!(next_step(&items), Some(1));
        assert_eq!(
            items[2].detail,
            "0 of 5 positions have a starter. The team leader places players on the team page."
        );
        f.drafts = 2;
        f.starting_positions = 4;
        let items = checklist(&f);
        assert_eq!(
            states(&items),
            [true, false, false, false, true, false, false]
        );
        assert_eq!(items[4].status, "2 drafts");
        assert_eq!(next_step(&items), Some(1));
        f.starting_positions = 5;
        f.opponents = 1;
        assert_eq!(next_step(&checklist(&f)), Some(1));
    }

    #[test]
    fn joining_an_established_team_ticks_the_teams_steps() {
        let f = ChecklistFacts {
            team_name: Some("Rooks".into()),
            starting_positions: 5,
            opponents: 3,
            drafts: 4,
            plans: 2,
            reviews: 6,
            members: 6,
            pending_requests: 0,
        };
        let items = checklist(&f);
        assert!(checklist_complete(&items));
        assert_eq!(next_step(&items), None);
        assert_eq!(items[1].status, "6 members");
        // A waiting join request counts as an invited teammate.
        let solo = ChecklistFacts {
            team_name: Some("Rooks".into()),
            members: 1,
            pending_requests: 1,
            ..Default::default()
        };
        assert!(checklist(&solo)[1].done);
    }

    #[test]
    fn the_checklist_hides_when_muted_or_complete_unless_asked_for() {
        let open = checklist(&ChecklistFacts::default());
        assert!(checklist_due(true, &[], &open, false));
        assert!(!checklist_due(false, &[], &open, false));
        let hidden = [row(CHECKLIST, false, true)];
        assert!(!checklist_due(true, &hidden, &open, false));
        assert!(checklist_due(true, &hidden, &open, true));
        let done = checklist(&ChecklistFacts {
            team_name: Some("T".into()),
            starting_positions: 5,
            opponents: 1,
            drafts: 1,
            plans: 1,
            reviews: 1,
            members: 2,
            pending_requests: 0,
        });
        assert!(!checklist_due(true, &[], &done, false));
        assert!(checklist_due(true, &[], &done, true));
    }

    #[test]
    fn invite_comes_before_the_starting_five() {
        let items = checklist(&ChecklistFacts::default());
        assert_eq!(items.len(), 7);
        assert!(items[1].title.contains("Invite"));
        assert!(items[2].title.contains("starting five"));
        // For a new team with 1 member and 0 starters, next_step should be Some(1) (Invite a teammate)
        let f = ChecklistFacts {
            team_name: Some("NewTeam".into()),
            members: 1,
            starting_positions: 0,
            ..Default::default()
        };
        let items = checklist(&f);
        assert_eq!(next_step(&items), Some(1));
    }

    #[test]
    fn draft_and_dossier_nudges_are_for_leaders_only() {
        let mut f = facts("2026-10-12T12:00:00Z");
        f.draft_without_plan = Some(("draft:d".into(), "Rooks bo3".into()));
        f.dossier_without_draft = Some(("opponent:o".into(), "Rooks".into()));
        f.match_without_review = None;
        f.old_open_items = 0;

        // Leader/coach sees DraftWithoutPlan first
        f.viewer_manages = true;
        let n = pick_nudge(true, &f, &[], None).unwrap();
        assert_eq!(n.kind, NudgeKind::DraftWithoutPlan);
        assert_eq!(n.href, "/game-plan?draft_id=draft:d");
        
        // With DraftWithoutPlan muted, they see DossierWithoutDraft
        let muted = [row(NudgeKind::DraftWithoutPlan.key(), false, true)];
        let n = pick_nudge(true, &f, &muted, None).unwrap();
        assert_eq!(n.kind, NudgeKind::DossierWithoutDraft);
        
        // Regular member sees nothing
        f.viewer_manages = false;
        assert!(pick_nudge(true, &f, &[], None).is_none());
    }

    #[test]
    fn match_nudge_is_for_everyone() {
        let mut f = facts("2026-10-12T12:00:00Z");
        f.match_without_review = Some(PastMatch {
            opponent: "Rooks".into(),
            kind_label: "Scrim".into(),
            plan_id: "game_plan:p".into(),
        });
        f.draft_without_plan = None;
        f.dossier_without_draft = None;
        f.old_open_items = 0;

        // Regular member sees MatchWithoutReview
        f.viewer_manages = false;
        let n = pick_nudge(true, &f, &[], None).unwrap();
        assert_eq!(n.kind, NudgeKind::MatchWithoutReview);
        assert_eq!(n.href, "/post-game?plan_id=game_plan:p");
    }

    #[test]
    fn the_welcome_is_for_young_accounts_once() {
        let now = at("2026-10-12T12:00:00Z");
        let young = Some(at("2026-10-12T11:00:00Z"));
        let old = Some(at("2026-09-01T00:00:00Z"));
        assert!(welcome_due(true, &[], young, now));
        assert!(!welcome_due(false, &[], young, now));
        assert!(!welcome_due(true, &[], old, now));
        assert!(!welcome_due(true, &[row(WELCOME, true, false)], young, now));
        assert!(!welcome_due(true, &[row(WELCOME, false, true)], young, now));
        assert!(!welcome_due(true, &[], None, now));
    }

    fn facts(now: &str) -> NudgeFacts {
        NudgeFacts {
            now: at(now),
            last_write: Some(at(now)),
            prev_seen: None,
            account_created: Some(at("2026-01-01T00:00:00Z")),
            match_without_review: None,
            old_open_items: 0,
            draft_without_plan: None,
            dossier_without_draft: None,
            viewer_manages: true,
        }
    }

    #[test]
    fn returning_after_fourteen_days_without_a_write() {
        let mut f = facts("2026-10-12T12:00:00Z");
        assert!(pick_nudge(true, &f, &[], None).is_none());
        f.last_write = Some(at("2026-09-28T12:00:00Z"));
        let n = pick_nudge(true, &f, &[], None).unwrap();
        assert_eq!(n.kind, NudgeKind::Returning);
        assert_eq!(n.href, "/?guide=1#getting-started");
        f.last_write = Some(at("2026-09-28T12:00:01Z"));
        assert!(pick_nudge(true, &f, &[], None).is_none());
        // Never wrote anything: counted from the account's creation.
        f.last_write = None;
        assert!(is_returning(&f));
        f.account_created = Some(at("2026-10-10T00:00:00Z"));
        assert!(!is_returning(&f));
    }

    #[test]
    fn public_pages_do_not_count_as_visits() {
        for p in [
            "/",
            "/draft",
            "/team/dashboard",
            "/profile",
            "/admin/invites",
        ] {
            assert!(counts_as_visit(p), "{p}");
        }
        for p in [
            "/s/abc",
            "/legal/terms",
            "/auth/register",
            "/closed-beta",
            "/changelog",
            "/transparency",
        ] {
            assert!(!counts_as_visit(p), "{p}");
        }
    }

    /// Reading counts (polish pass): a member who opened the app a day ago
    /// is not "picking up again", however long ago they last wrote.
    #[test]
    fn a_recent_visit_counts_like_a_write() {
        let mut f = facts("2026-10-12T12:00:00Z");
        f.last_write = Some(at("2026-08-01T00:00:00Z"));
        f.prev_seen = Some(at("2026-10-11T09:00:00Z"));
        assert!(!is_returning(&f));
        // Away for two weeks: both the last write and the last visit are old.
        f.prev_seen = Some(at("2026-09-28T12:00:00Z"));
        assert!(is_returning(&f));
        // A visit but never a write: the visit decides, not the account age.
        f.last_write = None;
        f.account_created = Some(at("2026-01-01T00:00:00Z"));
        f.prev_seen = Some(at("2026-10-05T00:00:00Z"));
        assert!(!is_returning(&f));
    }

    #[test]
    fn one_nudge_in_priority_order_skipping_mutes_and_the_prep_step() {
        let mut f = facts("2026-10-12T12:00:00Z");
        f.old_open_items = 3;
        f.draft_without_plan = Some(("draft:d".into(), "Rooks bo3".into()));
        f.dossier_without_draft = Some(("opponent:o".into(), "Rooks".into()));
        f.match_without_review = Some(PastMatch {
            opponent: "Rooks".into(),
            kind_label: "Scrim".into(),
            plan_id: "game_plan:p".into(),
        });
        let n = pick_nudge(true, &f, &[], None).unwrap();
        assert_eq!(n.kind, NudgeKind::MatchWithoutReview);
        assert_eq!(
            n.text,
            "The scrim against Rooks had a game plan and has no review yet."
        );
        // The prep sequence already asks for this review: the next rule.
        let n = pick_nudge(true, &f, &[], Some("/post-game?plan_id=game_plan:p")).unwrap();
        assert_eq!(n.kind, NudgeKind::OldActionItems);
        assert_eq!(
            n.text,
            "3 action items from reviews have been open for more than a week."
        );
        let muted = [
            row("nudge.match_without_review", false, true),
            row("nudge.old_action_items", false, true),
        ];
        let n = pick_nudge(true, &f, &muted, None).unwrap();
        assert_eq!(n.kind, NudgeKind::DraftWithoutPlan);
        assert_eq!(n.href, "/game-plan?draft_id=draft:d");
        // Seen is not muted: a nudge comes back until its rule stops applying.
        let seen = [row("nudge.match_without_review", true, false)];
        assert_eq!(
            pick_nudge(true, &f, &seen, None).unwrap().kind,
            NudgeKind::MatchWithoutReview
        );
        f.match_without_review = None;
        f.old_open_items = 0;
        f.draft_without_plan = None;
        let n = pick_nudge(true, &f, &[], None).unwrap();
        assert_eq!(n.kind, NudgeKind::DossierWithoutDraft);
        assert!(n.text.contains("last 3 weeks"));
        assert!(pick_nudge(false, &f, &[], None).is_none());
        let all_muted: Vec<GuidanceRow> = NudgeKind::ALL
            .iter()
            .map(|k| row(k.key(), false, true))
            .collect();
        assert!(pick_nudge(true, &f, &all_muted, None).is_none());
    }

    #[test]
    fn hints_hide_when_used_or_dismissed_and_one_per_page() {
        let used = HintUse {
            drafted: true,
            ..Default::default()
        };
        let rows = [row("hint.pool_tiers", false, true)];
        let hints = active_hints(true, &rows, &used);
        assert!(!hints.contains(&Hint::DraftKeys));
        assert!(!hints.contains(&Hint::PoolTiers));
        assert!(hints.contains(&Hint::DraftGraph));
        assert!(active_hints(false, &[], &HintUse::default()).is_empty());

        let state = GuidanceState {
            enabled: true,
            hints: active_hints(true, &[], &HintUse::default()),
            intros: vec![],
        };
        assert_eq!(page_hint(&state, "/draft"), Some(Hint::DraftKeys));
        assert_eq!(
            page_hint(&state, "/team/player/user:x"),
            Some(Hint::ShareLinks)
        );
        assert_eq!(page_hint(&state, "/admin/bugs"), None);
        assert_eq!(page_hint(&state, "/stats"), None);
        // The navigation line only where no page hint is.
        assert!(nav_hint(&state, "/stats"));
        assert!(!nav_hint(&state, "/draft"));
        assert!(!nav_hint(&state, "/"));
        assert!(!nav_hint(&state, "/admin"));
        for h in Hint::ALL {
            assert_eq!(Hint::from_key(h.key()), Some(h));
            assert!(h.key().starts_with(HINT_PREFIX));
        }
    }

    fn intro(slug: &str, route: &str, date: &str) -> Intro {
        Intro {
            key: intro_key(slug, route),
            route: route.into(),
            feature: "Feature".into(),
            text: "Try it.".into(),
            slug: slug.into(),
            date: date.into(),
        }
    }

    #[test]
    fn introductions_are_for_people_who_knew_the_app_before() {
        let now = at("2026-10-20T12:00:00Z");
        let all = [
            intro("m4", "/draft", "2026-10-11"),
            intro("m9", "/profile", "2026-10-12"),
            intro("old", "/stats", "2026-01-01"),
            intro("future", "/me", "2026-11-01"),
        ];
        let before = Some(at("2026-10-01T00:00:00Z"));
        let got = eligible_intros(true, &all, &[], before, now);
        let keys: Vec<&str> = got.iter().map(|i| i.slug.as_str()).collect();
        // Newest first; too old and not yet dated are left out.
        assert_eq!(keys, ["m9", "m4"]);
        // An account made on or after the entry's day meets it as new anyway.
        let later = Some(at("2026-10-12T08:00:00Z"));
        let got = eligible_intros(true, &all, &[], later, now);
        assert!(got.is_empty(), "{got:?}");
        // Seen once: gone, and its New mark with it.
        let rows = [row(&intro_key("m9", "/profile"), true, false)];
        let got = eligible_intros(true, &all, &rows, before, now);
        assert_eq!(got.len(), 1);
        let state = GuidanceState {
            enabled: true,
            hints: vec![Hint::DraftKeys],
            intros: got,
        };
        assert_eq!(new_routes(&state), vec!["/draft".to_string()]);
        // The introduction wins over the page's hint.
        assert!(intro_for(&state, "/draft").is_some());
        assert_eq!(page_hint(&state, "/draft"), None);
        assert!(eligible_intros(false, &all, &[], before, now).is_empty());
    }

    #[test]
    fn only_known_keys_are_written() {
        let intros = [intro("m9", "/profile", "2026-10-12")];
        assert!(is_known_key("welcome", &intros));
        assert!(is_known_key("hint.draft_keys", &intros));
        assert!(is_known_key("nudge.returning", &intros));
        assert!(is_known_key("intro:m9:/profile", &intros));
        assert!(!is_known_key("intro:m9:/elsewhere", &intros));
        assert!(!is_known_key("hint.nope", &intros));
        assert!(!is_known_key(&"x".repeat(500), &intros));
    }
}
