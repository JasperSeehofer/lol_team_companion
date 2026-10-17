//! Terms of use (M8). The structure and the rules of the closed beta are
//! written here. The pricing statement is the owner's own (launch checklist,
//! 12 October 2026); the liability and governing-law paragraphs were drafted
//! for the owner and are listed for the owner's adviser to confirm. Riot's
//! production key review asks for terms next to the privacy page.

use crate::components::footer::RIOT_DISCLAIMER;
use crate::components::ui::{PageHeader, PageShell, PageWidth, SectionHeader};
use leptos::prelude::*;

/// The pricing statement, exactly as the owner gave it.
pub const PRICING: &str = "Using Team Companion costs nothing; support is voluntary.";

/// The sections of the terms: (id, title, paragraphs).
pub const SECTIONS: &[(&str, &str, &[&str])] = &[
    (
        "service-h",
        "What Team Companion is",
        &[
            "Team Companion is a private coaching workspace for one team: scouting, drafting, game plans, reviews and lessons. It is in a closed beta: you can register only with an invite code, features change, and things can break.",
            PRICING,
            "The operator is named in the Impressum. These terms apply between you and the operator.",
        ],
    ),
    (
        "account-h",
        "Your account",
        &[
            "You must be 16 or older to register; at sign-up you confirm it and accept these terms.",
            "One account per person. Keep your password to yourself; you are responsible for what happens with your account. Tell the operator at once if you think someone else used it.",
            "Invite codes are meant for the people they were given to. Do not publish them.",
            "You can download your data or delete your account at any time on your profile.",
        ],
    ),
    (
        "team-h",
        "Your team's workspace",
        &[
            "What a team writes (drafts, plans, reviews, notes, scouting dossiers) belongs to the team. Every member can read it and most of it can be edited by every member; the leader places players, answers join requests and can remove members.",
            "When you leave a team or delete your account, what you wrote for the team stays with the team. Share links make single plans, reviews or player cards readable to anyone who has the link, until they expire or are revoked.",
        ],
    ),
    (
        "use-h",
        "Fair use",
        &[
            "Use Team Companion to prepare and review your own team's games. Do not try to read another team's data, get around rate limits or the invite gate, automate requests, or copy data out of Team Companion in bulk.",
            "Scouting covers what is public: the match history and champions of players your team names, looked up through Riot's official API. Do not use Team Companion to follow or expose individual players beyond preparing for a match.",
            "Do not put content in notes, reports or names that is unlawful, harassing or that reveals other people's private information.",
            "Team Companion gives no help during a live game or champion select beyond what your team prepared in advance.",
        ],
    ),
    (
        "riot-h",
        "Riot Games",
        &[
            "Game data comes from the Riot Games API and is shown as Riot provides it. Riot can change or withdraw access, which can switch Riot-based features off.",
        ],
    ),
    (
        "availability-h",
        "Availability and changes",
        &[
            "Team Companion is offered as it is, without a promise of availability. It restarts for a few seconds every night for the backup and for updates.",
            "Data is not reset without notice. Changes to features are announced in the changelog; changes to these terms are announced there before they apply.",
            "The operator may suspend an account that breaks these terms, and will say why.",
        ],
    ),
    (
        "liability-h",
        "Liability and law",
        &[
            "Team Companion is a free, private, non-commercial service. The operator is liable only for intent and gross negligence, and for injury to life, body or health. Otherwise liability is excluded as far as the law allows.",
            "There is no guarantee that Team Companion is available at any given time, or that game data from Riot is correct or complete.",
            "These terms are governed by the law of the Federal Republic of Germany. If you use Team Companion as a consumer, you keep the protection of the mandatory law of the country where you live. The place of jurisdiction follows the statutory rules.",
        ],
    ),
    (
        "contact-h",
        "Contact",
        &[
            "Questions about these terms go to the operator (see the Impressum), or through the bug-report button on any page after signing in.",
        ],
    ),
];

#[component]
pub fn TermsPage() -> impl IntoView {
    view! {
        <PageShell width=PageWidth::Reading>
            <PageHeader title="Terms of use" eyebrow="Legal" meta=move || view! { <span>"The rules for using Team Companion during the closed beta."</span> } />
            {SECTIONS.iter().enumerate().map(|(i, (id, title, paragraphs))| view! {
                <section class=if i == 0 { "" } else { "mt-10" } aria-labelledby=*id>
                    <SectionHeader title=*title id=*id />
                    {paragraphs.iter().map(|text| view! { <p class="mt-2 max-w-prose text-body text-secondary">{*text}</p> }).collect_view()}
                </section>
            }).collect_view()}
            <section class="mt-10" aria-labelledby="disclaimer-h">
                <SectionHeader title="Disclaimer" id="disclaimer-h" />
                <p class="max-w-prose text-body text-secondary">{RIOT_DISCLAIMER}</p>
            </section>
        </PageShell>
    }
}

#[cfg(test)]
mod tests {
    use super::{PRICING, SECTIONS};

    fn paragraphs() -> impl Iterator<Item = &'static str> {
        SECTIONS.iter().flat_map(|(_, _, p)| p.iter().copied())
    }

    /// No placeholder is left, and the owner's pricing statement is there
    /// word for word.
    #[test]
    fn the_owners_facts_are_filled_in() {
        assert!(paragraphs().all(|t| !t.contains("OWNER")));
        assert_eq!(
            PRICING,
            "Using Team Companion costs nothing; support is voluntary."
        );
        assert!(paragraphs().any(|t| t == PRICING));
    }

    /// The drafted clauses cover what the owner asked for.
    #[test]
    fn liability_and_law_are_stated() {
        let all: String = paragraphs().collect::<Vec<_>>().join(" ");
        for needle in [
            "intent and gross negligence",
            "life, body or health",
            "Federal Republic of Germany",
            "mandatory law of the country where you live",
        ] {
            assert!(all.contains(needle), "{needle} missing from the terms");
        }
    }
}
