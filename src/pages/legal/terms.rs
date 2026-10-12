//! Terms of use (M8). The structure and the rules of the closed beta are
//! written here; facts only the operator can state (identity, pricing,
//! liability, governing law) are `[OWNER: ...]` placeholders. Riot's
//! production key review asks for terms next to the privacy page.

use crate::components::footer::RIOT_DISCLAIMER;
use crate::components::ui::{
    Callout, CalloutTone, PageHeader, PageShell, PageWidth, SectionHeader,
};
use crate::pages::legal::impressum::OwnerPlaceholder;
use leptos::prelude::*;

/// The sections of the terms: (id, title, paragraphs). A paragraph that
/// starts with `OWNER:` renders as a placeholder.
pub const SECTIONS: &[(&str, &str, &[&str])] = &[
    (
        "service-h",
        "What Companion is",
        &[
            "Companion is a private coaching workspace for one team: scouting, drafting, game plans, reviews and lessons. It is in a closed beta: you can register only with an invite code, features change, and things can break.",
            "OWNER: pricing statement (for example that using Companion costs nothing)",
            "The operator is named in the Impressum. These terms apply between you and the operator.",
        ],
    ),
    (
        "account-h",
        "Your account",
        &[
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
            "Use Companion to prepare and review your own team's games. Do not try to read another team's data, get around rate limits or the invite gate, automate requests, or copy data out of Companion in bulk.",
            "Scouting covers what is public: the match history and champions of players your team names, looked up through Riot's official API. Do not use Companion to follow or expose individual players beyond preparing for a match.",
            "Do not put content in notes, reports or names that is unlawful, harassing or that reveals other people's private information.",
            "Companion gives no help during a live game or champion select beyond what your team prepared in advance.",
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
            "Companion is offered as it is, without a promise of availability. It restarts for a few seconds every night for the backup and for updates.",
            "Data is not reset without notice. Changes to features are announced in the changelog; changes to these terms are announced there before they apply.",
            "The operator may suspend an account that breaks these terms, and will say why.",
        ],
    ),
    (
        "liability-h",
        "Liability and law",
        &[
            "OWNER: liability clause suitable for a free private service under the operator's law",
            "OWNER: governing law and place of jurisdiction",
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
            <PageHeader title="Terms of use" eyebrow="Legal" meta=move || view! { <span>"The rules for using Companion during the closed beta."</span> } />
            <div class="mb-8">
                <Callout tone=CalloutTone::Warning title="Partly a draft">
                    "Items marked OWNER are completed by the operator before Companion opens beyond the closed beta."
                </Callout>
            </div>
            {SECTIONS.iter().enumerate().map(|(i, (id, title, paragraphs))| view! {
                <section class=if i == 0 { "" } else { "mt-10" } aria-labelledby=*id>
                    <SectionHeader title=*title id=*id />
                    {paragraphs.iter().map(|text| match text.strip_prefix("OWNER: ") {
                        Some(what) => view! { <p class="mt-2"><OwnerPlaceholder what=what /></p> }.into_any(),
                        None => view! { <p class="mt-2 max-w-prose text-body text-secondary">{*text}</p> }.into_any(),
                    }).collect_view()}
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
    use super::SECTIONS;

    /// Facts that are the operator's stay placeholders until filled in.
    #[test]
    fn owner_facts_are_placeholders() {
        let owner: Vec<&str> = SECTIONS
            .iter()
            .flat_map(|(_, _, p)| p.iter())
            .filter_map(|t| t.strip_prefix("OWNER: "))
            .collect();
        for needle in ["pricing", "liability", "governing law"] {
            assert!(
                owner.iter().any(|o| o.contains(needle)),
                "{needle} must be an OWNER placeholder"
            );
        }
    }
}
