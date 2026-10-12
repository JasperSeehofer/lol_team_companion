//! Seed content inserted once by migration `community_seed_content` (M7):
//! the first what-next proposals (unpublished until an admin publishes
//! them), the proposed surveys (drafts, unapproved) and the money texts
//! (unapproved). Data only; wording from the M7 copy deck.

use serde::Serialize;
use surrealdb::types::SurrealValue;

use crate::models::community::{
    SurveyWording, TEXT_LEDGER_LABELS, TEXT_MONEY_POLICY, TEXT_NOT_COVERED, TEXT_SPLIT_LEGEND,
    TEXT_SUPPORT_SENTENCE,
};

#[derive(Clone, Debug, Serialize, SurrealValue)]
pub struct SeedProposal {
    pub seed_key: String,
    pub title: String,
    pub summary: String,
    pub status: String,
    pub source: String,
    pub shipped_in: Option<String>,
}

#[derive(Clone, Debug, Serialize, SurrealValue)]
pub struct SeedSurvey {
    pub seed_key: String,
    pub question: String,
    pub kind: String,
    pub choices: Vec<String>,
    pub scale_low_label: Option<String>,
    pub scale_high_label: Option<String>,
    pub trigger: String,
    pub note: String,
    pub wording_hash: String,
}

#[derive(Clone, Debug, Serialize, SurrealValue)]
pub struct SeedText {
    pub slug: String,
    pub body: String,
}

/// The changelog entry that shipped shareable player cards (M5).
pub const PLAYER_CARDS_SHIPPED_IN: &str = "milestone-m5-remaining-pages-navigation-mobile";

pub fn proposals() -> Vec<SeedProposal> {
    let rows: [(&str, &str, &str, &str, &str); 17] = [
        ("graph_ban_suggestions", "Ban suggestions in the Draft Graph", "The Draft Graph suggests bans as well as picks, each with its reasons: what the ban takes away from the opponent and what it protects in your draft.", "planned", "mission"),
        ("sand_table_board", "Sand Table: a shared tactics board", "Draw a play on the map together, with everyone's cursor visible, and save it as part of the game plan.", "planned", "mission"),
        ("sand_table_replay", "Sand Table: replay a game's macro", "Replay a finished game on the map from its timeline: where everyone was, objective setups, fights and timers, with exact events marked and estimated paths shown as estimates.", "planned", "mission"),
        ("plan_vs_reality", "Plan against reality", "Lay the game plan over the replay to see where the game followed the plan and where it left it.", "planned", "mission"),
        ("series_fearless", "Series and Fearless draft", "Group drafts into a best-of series; in Fearless mode, champions used earlier in the series are marked unavailable.", "planned", "mission"),
        ("opponent_memory", "Opponent memory", "When you meet a team again, see your earlier drafts, reviews and what worked against them.", "planned", "mission"),
        ("practice_loop", "Practice targets from pool gaps", "Turn gaps in the champion pool into practice targets per player and follow progress from their ranked games.", "planned", "mission"),
        ("patch_awareness", "Patch notes for your pools", "When a patch lands, flag the champions in your team's pools that changed.", "planned", "mission"),
        ("vod_bookmarks", "Video bookmarks on reviews", "Attach a video link with timestamped notes to a review. Notes only: Team Companion does not analyse video.", "planned", "mission"),
        ("prediction_check", "Prediction check", "Write what you expect in the game plan, mark what happened in the review, and see how often the plan read the game right.", "planned", "mission"),
        ("multiple_teams", "More than one team per coach", "Let a coach or analyst work with several teams from one account, with each team's data kept apart.", "proposed", "mission"),
        ("flex_draft", "Flex queue draft format", "Plan drafts for Flex queue, where bans happen at the same time, next to the tournament format.", "proposed", "backlog"),
        ("player_cards", "Shareable player cards", "A read-only card per player with role, pool and form, to share with a link.", "shipped", "mission"),
        ("discord", "Post plans to Discord", "Send the opponent brief or the game plan to your team's Discord channel.", "proposed", "mission"),
        ("community_traits", "Suggest champion traits", "Propose corrections to the Draft Graph's champion traits; accepted proposals update everyone's graph.", "proposed", "mission"),
        ("german_ui", "German interface", "Use Team Companion in German.", "proposed", "mission"),
        ("light_theme", "Light theme", "A light colour theme for bright rooms.", "proposed", "mission"),
    ];
    rows.into_iter()
        .map(|(key, title, summary, status, source)| SeedProposal {
            seed_key: key.into(),
            title: title.into(),
            summary: summary.into(),
            status: status.into(),
            source: source.into(),
            shipped_in: (status == "shipped").then(|| PLAYER_CARDS_SHIPPED_IN.to_string()),
        })
        .collect()
}

/// The four proposed surveys with their wording hashes.
#[allow(clippy::type_complexity)]
pub fn surveys(hash: impl Fn(&SurveyWording) -> String) -> Vec<SeedSurvey> {
    let rows: [(
        &str,
        &str,
        &str,
        Vec<&str>,
        Option<(&str, &str)>,
        &str,
        &str,
    ); 4] = [
        (
            "draft_ease",
            "How easy was it to plan this draft?",
            "scale",
            vec![],
            Some(("Hard", "Easy")),
            "draft_saved_3",
            "Whether the draft board and the Draft Graph help or get in the way.",
        ),
        (
            "review_value",
            "Did this review give your team something concrete to work on?",
            "single_choice",
            vec!["Yes, clearly", "Somewhat", "Not really"],
            None,
            "review_saved",
            "Whether reviews turn into action, or feel like paperwork.",
        ),
        (
            "patch_impact",
            "Has patch {patch} changed how your team drafts?",
            "single_choice",
            vec!["Yes, a lot", "A little", "Not yet", "Not sure"],
            None,
            "patch_changed",
            "Whether patch awareness (R2) is worth building, and how fast teams adapt.",
        ),
        (
            "what_next_missing",
            "What should Team Companion do for your team that is not on this board?",
            "short_text",
            vec![],
            None,
            "what_next_visit",
            "Ideas missing from the board, in users' own words.",
        ),
    ];
    rows.into_iter()
        .map(|(key, question, kind, choices, scale, trigger, note)| {
            let wording = SurveyWording {
                question: question.into(),
                kind: kind.into(),
                choices: choices.iter().map(|c| c.to_string()).collect(),
                scale_low_label: scale.map(|s| s.0.to_string()),
                scale_high_label: scale.map(|s| s.1.to_string()),
            };
            SeedSurvey {
                seed_key: key.into(),
                wording_hash: hash(&wording),
                question: wording.question,
                kind: wording.kind,
                choices: wording.choices,
                scale_low_label: wording.scale_low_label,
                scale_high_label: wording.scale_high_label,
                trigger: trigger.into(),
                note: note.into(),
            }
        })
        .collect()
}

/// The money texts, as proposed (unapproved until an admin approves them).
pub fn texts() -> Vec<SeedText> {
    [
        (
            TEXT_MONEY_POLICY,
            "Team Companion is free to use, and every feature stays free. Support is a voluntary contribution to the running costs. It goes to costs first (hosting, domain and AI tokens), and whatever remains goes to the maintainer.\n\nSupport buys nothing: there are no extras and no advantage in the app. Team Companion is run by a private person, not a charity, so no donation receipt is issued.\n\nEach month below shows what came in, what running Team Companion cost, and what went to the maintainer.",
        ),
        (
            TEXT_LEDGER_LABELS,
            "Support received\nHosting\nDomain\nAI tokens\nOther\nRunning costs\nTo the maintainer\nNot covered by support",
        ),
        (
            TEXT_SPLIT_LEGEND,
            "Split: costs covered, then to the maintainer, then any costs not covered.",
        ),
        (
            TEXT_NOT_COVERED,
            "Costs above support were paid by the maintainer.",
        ),
        (
            TEXT_SUPPORT_SENTENCE,
            "If Team Companion is useful to your team, you can support it. Support goes to running costs first, then to the maintainer. It buys nothing in the app, and no donation receipt is issued.",
        ),
    ]
    .into_iter()
    .map(|(slug, body)| SeedText {
        slug: slug.into(),
        body: body.into(),
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::community::{
        forbidden_money_words, validate_site_text, validate_survey, ProposalStatus, SurveyInput,
        MAX_PROPOSAL_SUMMARY, MAX_PROPOSAL_TITLE,
    };

    #[test]
    fn seeded_proposals_fit_the_limits() {
        let p = proposals();
        assert_eq!(p.len(), 17);
        let mut keys = std::collections::BTreeSet::new();
        for row in &p {
            assert!(keys.insert(row.seed_key.clone()));
            assert!(row.title.chars().count() <= MAX_PROPOSAL_TITLE);
            assert!(row.summary.chars().count() <= MAX_PROPOSAL_SUMMARY);
            assert!(ProposalStatus::parse(&row.status).is_some());
            assert_eq!(row.status == "shipped", row.shipped_in.is_some());
        }
    }

    #[test]
    fn seeded_surveys_pass_validation() {
        let s = surveys(|w| w.canonical_json());
        assert_eq!(s.len(), 4);
        for row in s {
            let input = SurveyInput {
                id: None,
                question: row.question.clone(),
                kind: row.kind.clone(),
                choices_text: row.choices.join("\n"),
                scale_low_label: row.scale_low_label.clone().unwrap_or_default(),
                scale_high_label: row.scale_high_label.clone().unwrap_or_default(),
                trigger: row.trigger.clone(),
                note: row.note.clone(),
            };
            let (w, _, _) =
                validate_survey(&input).unwrap_or_else(|e| panic!("{}: {e}", row.seed_key));
            assert_eq!(w.canonical_json(), row.wording_hash);
        }
    }

    #[test]
    fn seeded_texts_are_valid_and_use_the_right_words() {
        for t in texts() {
            assert!(validate_site_text(&t.slug, &t.body).is_ok(), "{}", t.slug);
            assert!(forbidden_money_words(&t.body).is_empty(), "{}", t.slug);
        }
    }
}
