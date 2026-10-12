//! Privacy (Datenschutz). Lists exactly what Companion stores, checked
//! against `schema.surql`, and the Riot disclaimer. The legal parts
//! (controller, legal basis, hosting, retention, rights) are the owner's to
//! write with advice; until then they are marked placeholders.

use crate::components::footer::RIOT_DISCLAIMER;
use crate::components::ui::{
    Callout, CalloutTone, PageHeader, PageShell, PageWidth, SectionHeader,
};
use crate::pages::legal::impressum::OwnerPlaceholder;
use leptos::prelude::*;

/// What the app stores, by area: (area, items). Kept in step with
/// `schema.surql`; a test checks the tables it names exist there.
pub const STORED: &[(&str, &[&str])] = &[
    (
        "Your account",
        &[
            "Username and email address (the email in lowercase).",
            "Your password, only as an Argon2 hash; the password itself is never stored.",
            "Your region, and once you link it, your Riot ID and the Riot account id (PUUID) Riot returns for it.",
            "Whether you are an admin, and when the account was created.",
        ],
    ),
    (
        "Signing in",
        &[
            "A session cookie (HttpOnly, SameSite=Strict) and the session it points to on the server. A session ends after 30 days without use; ended sessions are deleted.",
            "Invite codes with how often they were used.",
        ],
    ),
    (
        "Your team",
        &[
            "The team's name and region, its members with their place on the roster (starting five, substitute, coach) and position, and join requests with their status.",
            "What the team writes: drafts with picks and bans, draft trees, game plans and checklists, reviews, action items (text, owner, due day), team notes (with the text before the last edit, and deleted notes for one day, so they can be restored), notes on champion pairs, and the next match (time, opponent, notes, who marked the brief read).",
            "Opponent dossiers: player names, Riot IDs and Riot account ids, champions seen, mastery and role data read from Riot, and your notes.",
            "Share links: a random token, what it opens, who made it, when it expires, and whether it was revoked.",
        ],
    ),
    (
        "Your own pages",
        &[
            "Your champion pool and champion notes, personal learnings and personal goals.",
        ],
    ),
    (
        "From Riot, when you or your team sync",
        &[
            "Your recent games: champion, kills, deaths, assists, CS, vision score, damage and the result.",
            "For a game you open: the public match data of all ten players and its timeline.",
            "Snapshots of your ranked standing (tier, division, LP, wins, losses).",
        ],
    ),
    (
        "Bug reports",
        &[
            "What you send with the bug-report button: the page, the element you pointed at, your text, the category and the window size. Open reports are also written to a file on the server for the maintainer to read.",
        ],
    ),
];

#[component]
pub fn DatenschutzPage() -> impl IntoView {
    let legal: [(&str, &str, &str); 5] = [
        (
            "controller-h",
            "Controller",
            "name, address and email of the controller",
        ),
        (
            "basis-h",
            "Legal basis",
            "the legal basis for each kind of data above",
        ),
        (
            "hosting-h",
            "Hosting and server logs",
            "hosting provider, location, what the server logs record and for how long",
        ),
        (
            "retention-h",
            "Retention and deletion",
            "how long each kind of data is kept and how to have it deleted",
        ),
        (
            "rights-h",
            "Your rights",
            "the rights of data subjects and the supervisory authority",
        ),
    ];
    view! {
        <PageShell width=PageWidth::Reading>
            <PageHeader title="Privacy" eyebrow="Legal \u{00b7} Datenschutz" meta=move || view! { <span>"What Companion stores, and why it is there."</span> } />
            <div class="mb-8">
                <Callout tone=CalloutTone::Warning title="Partly a draft">
                    "The list of stored data is complete for this version. The legal sections further down are placeholders until the operator completes them."
                </Callout>
            </div>
            <section aria-labelledby="stored-h">
                <SectionHeader title="What Companion stores" id="stored-h" />
                {STORED.iter().map(|(area, items)| view! {
                    <h3 class="mt-6 font-display text-heading font-semibold text-primary">{*area}</h3>
                    <ul class="mt-2 flex list-disc flex-col gap-1.5 pl-5 text-body text-secondary">
                        {items.iter().map(|i| view! { <li>{*i}</li> }).collect_view()}
                    </ul>
                }).collect_view()}
                <h3 class="mt-6 font-display text-heading font-semibold text-primary">"What it does not do"</h3>
                <p class="mt-2 max-w-prose text-body text-secondary">
                    "No analytics and no third-party trackers. Fonts are served by Companion itself, and champion and item pictures come through Companion's own server, so your browser does not contact other sites while you use it."
                </p>
            </section>
            {legal.into_iter().map(|(id, title, what)| view! {
                <section class="mt-10" aria-labelledby=id>
                    <SectionHeader title=title id=id />
                    <p><OwnerPlaceholder what=what /></p>
                </section>
            }).collect_view()}
            <section class="mt-10" aria-labelledby="riot-h">
                <SectionHeader title="Riot Games" id="riot-h" />
                <p class="max-w-prose text-body text-secondary">{RIOT_DISCLAIMER}</p>
            </section>
        </PageShell>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_listed_tables_exist() {
        let schema = include_str!("../../../schema.surql");
        for table in [
            "user",
            "sessions",
            "invite",
            "team",
            "team_member",
            "join_request",
            "draft",
            "draft_tree",
            "game_plan",
            "checklist_instance",
            "post_game_learning",
            "action_item",
            "team_note",
            "team_relation",
            "scheduled_match",
            "opponent",
            "opponent_player",
            "share_link",
            "champion_pool",
            "champion_note",
            "personal_learning",
            "personal_goal",
            "player_match",
            "match_detail",
            "match_timeline_event",
            "ranked_snapshot",
            "bug_report",
        ] {
            assert!(
                schema.contains(&format!("DEFINE TABLE IF NOT EXISTS {table} ")),
                "{table} is listed on the privacy page but not in the schema"
            );
        }
    }
}
