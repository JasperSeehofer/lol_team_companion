//! Survey delivery rules (M7, spec 3.3) as pure functions over rows, so the
//! weekly cap, expiry and context preference are unit-tested without a DB.

use chrono::{DateTime, Duration, Utc};

use crate::models::community::SurveyTrigger;

/// At most one survey shown per user in this window (answered or not).
pub const WEEKLY_CAP: Duration = Duration::days(7);
/// A pending prompt that was never shown expires after this.
pub const PENDING_TTL: Duration = Duration::days(14);
/// A patch survey is asked within this window after the patch was detected.
pub const PATCH_WINDOW: Duration = Duration::days(7);
/// A trigger noted this recently counts as "what the user just did".
pub const RECENT: Duration = Duration::minutes(10);

/// The path where the what-next trigger fires.
pub const WHAT_NEXT_PATH: &str = "/what-next";

/// One of the user's prompts, with what delivery needs to know about it.
#[derive(Clone, Debug, PartialEq)]
pub struct PromptRow {
    pub id: String,
    pub trigger: SurveyTrigger,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub shown_at: Option<DateTime<Utc>>,
    /// The survey is live and its approval covers the current wording.
    pub deliverable: bool,
}

/// What to show.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Delivery {
    /// Nothing this time.
    Nothing,
    /// The prompt shown earlier in the window, still unanswered.
    Again(String),
    /// A pending prompt to mark shown now.
    Show(String),
}

/// Pick what to deliver. `rows` are the user's prompts (any state).
pub fn select(rows: &[PromptRow], now: DateTime<Utc>, path: &str) -> Delivery {
    // Weekly cap: anything shown in the window blocks a new survey.
    if let Some(recent) = rows
        .iter()
        .filter(|r| r.shown_at.is_some_and(|t| now - t < WEEKLY_CAP))
        .max_by_key(|r| r.shown_at)
    {
        return if recent.state == "shown" && recent.deliverable {
            Delivery::Again(recent.id.clone())
        } else {
            Delivery::Nothing
        };
    }
    let rank = |r: &PromptRow| -> u8 {
        if r.trigger == SurveyTrigger::WhatNextVisit && path == WHAT_NEXT_PATH {
            0
        } else if now - r.created_at < RECENT {
            1
        } else {
            2
        }
    };
    rows.iter()
        .filter(|r| r.state == "pending" && r.deliverable && now - r.created_at < PENDING_TTL)
        .min_by(|a, b| {
            rank(a)
                .cmp(&rank(b))
                .then(a.created_at.cmp(&b.created_at))
                .then(a.id.cmp(&b.id))
        })
        .map(|r| Delivery::Show(r.id.clone()))
        .unwrap_or(Delivery::Nothing)
}

/// The patch occurrence to note now, if the newest patch is recent enough.
pub fn patch_occurrence(
    newest: Option<(&str, DateTime<Utc>)>,
    now: DateTime<Utc>,
) -> Option<String> {
    newest
        .filter(|(_, at)| now - *at < PATCH_WINDOW)
        .map(|(v, _)| v.to_string())
}

/// The question with `{patch}` replaced by the occurrence (patch surveys).
pub fn render_question(question: &str, occurrence: &str) -> String {
    if occurrence.is_empty() {
        question.replace("{patch}", "this patch")
    } else {
        question.replace("{patch}", occurrence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        "2026-10-12T12:00:00Z".parse().unwrap()
    }

    fn row(
        id: &str,
        trigger: SurveyTrigger,
        state: &str,
        age_min: i64,
        shown_min: Option<i64>,
    ) -> PromptRow {
        PromptRow {
            id: id.into(),
            trigger,
            state: state.into(),
            created_at: now() - Duration::minutes(age_min),
            shown_at: shown_min.map(|m| now() - Duration::minutes(m)),
            deliverable: true,
        }
    }

    #[test]
    fn oldest_pending_without_context() {
        let rows = vec![
            row("a", SurveyTrigger::ReviewSaved, "pending", 600, None),
            row("b", SurveyTrigger::DraftSaved3, "pending", 900, None),
        ];
        assert_eq!(select(&rows, now(), "/draft"), Delivery::Show("b".into()));
    }

    #[test]
    fn context_wins_over_age() {
        let rows = vec![
            row("old", SurveyTrigger::DraftSaved3, "pending", 900, None),
            row("recent", SurveyTrigger::ReviewSaved, "pending", 2, None),
            row("board", SurveyTrigger::WhatNextVisit, "pending", 1000, None),
        ];
        assert_eq!(
            select(&rows, now(), "/post-game"),
            Delivery::Show("recent".into())
        );
        assert_eq!(
            select(&rows, now(), "/what-next"),
            Delivery::Show("board".into())
        );
    }

    #[test]
    fn weekly_cap_counts_answered_and_dismissed() {
        for state in ["answered", "dismissed"] {
            let rows = vec![
                row(
                    "done",
                    SurveyTrigger::ReviewSaved,
                    state,
                    5000,
                    Some(6 * 24 * 60),
                ),
                row("next", SurveyTrigger::DraftSaved3, "pending", 5, None),
            ];
            assert_eq!(select(&rows, now(), "/"), Delivery::Nothing, "{state}");
        }
        let rows = vec![
            row(
                "done",
                SurveyTrigger::ReviewSaved,
                "answered",
                20000,
                Some(8 * 24 * 60),
            ),
            row("next", SurveyTrigger::DraftSaved3, "pending", 5, None),
        ];
        assert_eq!(select(&rows, now(), "/"), Delivery::Show("next".into()));
    }

    #[test]
    fn an_unanswered_prompt_comes_back_within_the_week() {
        let rows = vec![row("s", SurveyTrigger::ReviewSaved, "shown", 60, Some(30))];
        assert_eq!(select(&rows, now(), "/"), Delivery::Again("s".into()));
        let mut paused = rows.clone();
        paused[0].deliverable = false;
        assert_eq!(select(&paused, now(), "/"), Delivery::Nothing);
    }

    #[test]
    fn expired_undeliverable_and_closed_prompts_are_skipped() {
        let mut rows = vec![
            row(
                "stale",
                SurveyTrigger::ReviewSaved,
                "pending",
                15 * 24 * 60,
                None,
            ),
            row("paused", SurveyTrigger::DraftSaved3, "pending", 5, None),
            row("gone", SurveyTrigger::DraftSaved3, "expired", 5, None),
        ];
        rows[1].deliverable = false;
        assert_eq!(select(&rows, now(), "/"), Delivery::Nothing);
    }

    #[test]
    fn patch_window_and_question() {
        let at = now() - Duration::days(3);
        assert_eq!(
            patch_occurrence(Some(("16.21", at)), now()),
            Some("16.21".into())
        );
        let old = now() - Duration::days(8);
        assert_eq!(patch_occurrence(Some(("16.20", old)), now()), None);
        assert_eq!(patch_occurrence(None, now()), None);
        assert_eq!(
            render_question("Has patch {patch} changed it?", "16.21"),
            "Has patch 16.21 changed it?"
        );
    }
}
