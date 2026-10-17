//! Privacy (Datenschutz). Lists exactly what Team Companion stores (checked
//! against `schema.surql` by the tests below), who else is involved, the one
//! cookie, the logs, how long things are kept and how to use your rights.
//! The operator's facts (controller, providers, log retention, supervisory
//! authority) are the owner's own (launch checklist, 12 October 2026). The
//! legal bases ([`LEGAL_BASES`]) are drafted defaults pending the operator's
//! adviser (M10 review); the callout says so. Nothing here is invented.

use crate::components::footer::RIOT_DISCLAIMER;
use crate::components::ui::{
    Callout, CalloutTone, PageHeader, PageShell, PageWidth, SectionHeader,
};
use crate::pages::legal::impressum::{OPERATOR_ADDRESS, OPERATOR_EMAIL, OPERATOR_NAME};
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
            "When you confirmed at sign-up that you are 16 or older and accepted the terms.",
            "Whether you are an admin (and whether that was set with the promote command), when the account was created, and when you were last here and the time before (to the hour), so Match Day can tell when you come back after a break.",
            "Your settings: whether you get survey questions, whether guidance (the welcome, checklist, hints and nudges) is on, and whether teammates may share your player card by link.",
            "Guidance you have seen or turned off: which item (for example \u{201c}hint.draft_keys\u{201d}) and when, so it is not shown again.",
        ],
    ),
    (
        "Signing in",
        &[
            "A session cookie (HttpOnly, SameSite=Strict) and the session it points to on the server. A session ends after 30 days without use; ended sessions are deleted.",
            "Invite codes with how often they were used and, for a team invite, the team and position it leads to.",
            "Password reset codes an admin creates for you when you forgot your password: only a hash of the code, which admin created it, when it expires (15 minutes) and when it was used. They are deleted with your account.",
        ],
    ),
    (
        "Your team",
        &[
            "The team's name and region, its members with their place on the roster (starting five, substitute, coach) and position, and join requests with their status.",
            "What the team writes: drafts with picks and bans, series of drafts (best-of sets), draft trees, ban priority lists, game plans with checklists and checklist templates, reviews, action items (text, owner, due day), Sand Table boards with their saved versions, team notes (with the text before the last edit, and deleted notes for one day, so they can be restored), notes on champion pairs, and the next match (time, opponent, notes, who marked the brief read).",
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
            "Game replays a team imports for the Sand Table: the per-minute positions and the events of all ten players, with their Riot account ids.",
        ],
    ),
    (
        "Feedback",
        &[
            "What you send with Give Feedback: the page, the element you pointed at, your text, the kind (bug, idea or praise) and the window size. Open reports are also written to a file on the server for the maintainer to read.",
            "A screenshot, if you attach one: stored on the server as a picture file next to the report (converted to WebP, which removes what the original file carried about where, when and on what device it was taken), seen only by admins. It can show other people, for example scouted players' Riot IDs; admins delete such a picture when it should not be kept, and it is deleted with your account.",
            "What the team does with a report: its status, a title and a note the team writes (shown on the bug board only when the team puts it there), an internal note, which admin changed it and when, and when you last looked at your reports.",
        ],
    ),
    (
        "Community",
        &[
            "Your votes on What next ideas (which idea, when). Others see only how many votes an idea has, never who voted.",
            "Survey questions you were asked: which question, when it was shown, and your answer or that you dismissed it; and whether you turned survey questions off. Admins see answers without names.",
            "What admins write: survey questions, What next ideas, the monthly transparency figures, the texts of the transparency page and the champion traits behind the Draft Graph, each with who changed it and when.",
        ],
    ),
];

/// How long the ranked standing snapshots behind the LP history are kept
/// (M141: the same nightly task and `MATCH_CACHE_RETENTION_DAYS` as the
/// match cache).
pub const RANKED_SNAPSHOT_RETENTION: &str = "Ranked snapshots (your standing read from Riot at each sync, the points of your LP history): deleted 12 months after the sync by the same nightly task, or earlier when you delete your account.";

/// How long the shared Riot match caches are kept: the default of
/// `MATCH_CACHE_RETENTION_DAYS` (365). Change the page and the variable
/// together; a test checks the default.
pub const MATCH_CACHE_RETENTION: &str = "Games opened on Team Companion (the public match data of all ten players and the timeline) are cached for everyone and deleted 12 months after they were fetched; a nightly task removes them. The basic record of a game (its id, queue and length) stays as long as someone's synced games use it, and is deleted 12 months after the game once nobody's do.";

/// Why each kind of data is processed, and on what legal basis: (purpose,
/// basis). Drafted defaults (orchestrator's decision in the M10 review),
/// pending the operator's adviser; the page's callout says so.
pub const LEGAL_BASES: &[(&str, &str)] = &[
    (
        "Your account, your team's workspace and your own pages: to provide the service you signed up for.",
        "Legal basis: performance of the contract with you (Art. 6(1)(b) GDPR).",
    ),
    (
        "Linking your Riot account, syncing your games and your team's games, and share links: done when you or your team choose to.",
        "Legal basis: your own action under the contract with you (Art. 6(1)(b) GDPR).",
    ),
    (
        "Scouting: when your team scouts an opponent, the public match history of the players it names.",
        "Legal basis: the team's legitimate interest in preparing for its matches (Art. 6(1)(f) GDPR).",
    ),
    (
        "Logs, rate limits and backups: to keep the service secure, available and recoverable.",
        "Legal basis: legitimate interest in the security and operation of the service (Art. 6(1)(f) GDPR).",
    ),
    (
        "Bug reports, votes and survey answers: to decide what to fix and build next, with you.",
        "Legal basis: your own action under the contract with you (Art. 6(1)(b) GDPR); you choose whether to send them.",
    ),
];

/// Who else touches the data: (who, role, what reaches them).
pub const THIRD_PARTIES: &[(&str, &str)] = &[
    (
        "Riot Games",
        "The source of game data. Team Companion's server asks the Riot API for match history, match details and timelines, ranked standing and champion mastery when you or your team sync, link a Riot ID or scout an opponent. The request carries the Riot ID or account id being looked up and Team Companion's API key; your browser never contacts Riot.",
    ),
    (
        "Riot Games Data Dragon",
        "Champion and item pictures and the champion list. Team Companion's server downloads them once per game patch and serves them itself; nothing about you is sent.",
    ),
];

#[component]
pub fn DatenschutzPage() -> impl IntoView {
    let h3 = "mt-6 font-display text-heading font-semibold text-primary";
    let p = "mt-2 max-w-prose text-body text-secondary";
    let ul = "mt-2 flex max-w-prose list-disc flex-col gap-1.5 pl-5 text-body text-secondary";
    view! {
        <PageShell width=PageWidth::Reading>
            <PageHeader title="Privacy" eyebrow="Legal \u{00b7} Datenschutz" meta=move || view! { <span>"What Team Companion stores, who else is involved, and your rights."</span> } />
            <div class="mb-8">
                <Callout tone=CalloutTone::Warning title="Partly a draft">
                    "The legal bases below and two clauses of the terms (liability, governing law) are drafts pending the operator's adviser."
                </Callout>
            </div>

            <section aria-labelledby="controller-h">
                <SectionHeader title="Who is responsible" id="controller-h" />
                <p class=p>"The controller for the data on this site is:"</p>
                <p class="mt-2 text-body text-primary" data-testid="controller">
                    <span class="block">{OPERATOR_NAME}</span>
                    {OPERATOR_ADDRESS.iter().map(|line| view! { <span class="block">{*line}</span> }).collect_view()}
                </p>
                <p class=p>
                    "Data protection questions: "
                    <a href=format!("mailto:{OPERATOR_EMAIL}") class="text-primary underline decoration-rule underline-offset-4 hover:decoration-accent">{OPERATOR_EMAIL}</a>
                </p>
            </section>

            <section class="mt-10" aria-labelledby="stored-h">
                <SectionHeader title="What Team Companion stores" id="stored-h" />
                {STORED.iter().map(|(area, items)| view! {
                    <h3 class=h3>{*area}</h3>
                    <ul class=ul>
                        {items.iter().map(|i| view! { <li>{*i}</li> }).collect_view()}
                    </ul>
                }).collect_view()}
                <h3 class=h3>"What it does not do"</h3>
                <p class=p>
                    "No analytics, no advertising and no third-party trackers. The pages load no scripts, fonts or pictures from other sites: fonts are served by Team Companion itself, and champion and item pictures come through Team Companion's own server. Team Companion stores nothing in your browser besides the session cookie."
                </p>
            </section>

            <section class="mt-10" aria-labelledby="basis-h">
                <SectionHeader title="Why, and on what legal basis" id="basis-h" />
                <ul class=ul data-testid="legal-bases">
                    {LEGAL_BASES.iter().map(|(purpose, basis)| view! { <li>{*purpose} " " <span class="text-primary">{*basis}</span></li> }).collect_view()}
                </ul>
            </section>

            <section class="mt-10" aria-labelledby="third-h">
                <SectionHeader title="Who else is involved" id="third-h" />
                <dl class="mt-2 grid max-w-prose gap-x-6 gap-y-3 text-body sm:grid-cols-[12rem_minmax(0,1fr)]">
                    {THIRD_PARTIES.iter().map(|(who, what)| view! {
                        <dt class="font-medium text-primary">{*who}</dt>
                        <dd class="text-secondary">{*what}</dd>
                    }).collect_view()}
                    <dt class="font-medium text-primary">"Hosting"</dt>
                    <dd class="text-secondary">"The server and its database run on a rented virtual machine at Hetzner Online GmbH, Helsinki (Finland)."</dd>
                    <dt class="font-medium text-primary">"Backups"</dt>
                    <dd class="text-secondary">"A nightly copy of the database, encrypted on the server before it leaves, is kept on a storage box at Hetzner Online GmbH, Falkenstein (Germany)."</dd>
                </dl>
                <p class=p>"Nothing else: no analytics service, no email service, no payment provider."</p>
            </section>

            <section class="mt-10" aria-labelledby="cookies-h">
                <SectionHeader title="Cookies" id="cookies-h" />
                <p class=p>
                    "One cookie, named " <code class="font-mono text-label text-primary">"id"</code> ", set when you sign in. It holds a random session number, is sent only to Team Companion (HttpOnly, SameSite=Strict, and only over HTTPS) and ends 30 days after your last visit or when you sign out. It is needed to stay signed in; there are no other cookies and no tracking."
                </p>
            </section>

            <section class="mt-10" aria-labelledby="logs-h">
                <SectionHeader title="Server logs" id="logs-h" />
                <ul class=ul>
                    <li>"The web server's access log records each request: time, address requested, status, response size, the request headers except cookies (such as the browser name and language) and your IP address shortened to its first three parts (IPv4) or first three groups (IPv6). Cookies and passwords are never logged. The access log is kept for 14 days, then deleted."</li>
                    <li>"The application log records errors and notable events, such as an admin change or an account deletion, with the account's internal id. It never records passwords or session numbers. It is rotated by size (five files of 10 MB), so older entries are overwritten as new ones arrive."</li>
                    <li>"Rate limits keep IP addresses and account ids in memory only; an entry is dropped a minute after it is no longer needed and nothing is written to disk."</li>
                </ul>
            </section>

            <section class="mt-10" aria-labelledby="retention-h">
                <SectionHeader title="How long data is kept" id="retention-h" />
                <ul class=ul>
                    <li>"Your account, your own pages and synced games: until you delete your account."</li>
                    <li>{RANKED_SNAPSHOT_RETENTION}</li>
                    <li>"Your team's records: until the team deletes them, or the team is deleted. When you delete your account, what you wrote for the team stays with the team, shown as written by a former member."</li>
                    <li>"Deleted team notes: one day, so they can be restored. Sessions: 30 days after the last visit."</li>
                    <li>{MATCH_CACHE_RETENTION}</li>
                    <li>"Feedback: until an admin deletes it. When you delete your account, your reports stay without your name, shown as from a former member, and your text, the element you pointed at, your window size and any screenshot are removed. An admin can delete a screenshot at any time, for example when it shows someone who asks for that."</li>
                    <li>"Backups: 30 days. Data you delete disappears from the backups within 30 days. If nightly backups stop working for a while, the last good copies are kept until new ones succeed, so that a working copy always exists."</li>
                </ul>
            </section>

            <section class="mt-10" aria-labelledby="rights-h">
                <SectionHeader title="Your rights" id="rights-h" />
                <ul class=ul>
                    <li>"Access and portability: " <a href="/profile#your-data" class="text-primary underline decoration-rule underline-offset-4 hover:decoration-accent">"Download my data"</a> " on your profile gives you everything stored about you as a JSON file, at any time."</li>
                    <li>"Correction: change your username, region and Riot link on your profile; ask your team to correct team records."</li>
                    <li>"Erasure: " <a href="/profile#your-data" class="text-primary underline decoration-rule underline-offset-4 hover:decoration-accent">"Delete my account"</a> " on your profile, without asking anyone."</li>
                    <li>"Restriction and objection, and anything the buttons do not cover: write to the contact above."</li>
                    <li>"You can complain to a data protection supervisory authority. The one responsible for the operator is "
                        <span data-testid="authority">"Der Landesbeauftragte f\u{00fc}r den Datenschutz und die Informationsfreiheit Baden-W\u{00fc}rttemberg (LfDI), Lautenschlagerstra\u{00df}e 20, 70173 Stuttgart, "
                        <span class="text-primary">"www.baden-wuerttemberg.datenschutz.de"</span></span>
                        "."
                    </li>
                </ul>
            </section>

            <section class="mt-10" aria-labelledby="riot-h">
                <SectionHeader title="Riot Games" id="riot-h" />
                <p class="max-w-prose text-body text-secondary">{RIOT_DISCLAIMER}</p>
            </section>
        </PageShell>
    }
}

#[cfg(test)]
mod tests {
    /// Tables whose rows the page above describes.
    const LISTED: &[&str] = &[
        "user",
        "sessions",
        "invite",
        "team",
        "team_member",
        "join_request",
        "draft",
        "draft_action",
        "series",
        "draft_tree",
        "draft_tree_node",
        "tree_node_action",
        "ban_priority",
        "game_plan",
        "checklist_template",
        "checklist_instance",
        "post_game_learning",
        "action_item",
        "board",
        "board_version",
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
        "match",
        "player_match",
        "match_detail",
        "match_timeline_event",
        "replay",
        "replay_frame",
        "replay_event",
        "ranked_snapshot",
        "bug_report",
        "survey",
        "survey_prompt",
        "user_guidance",
        "password_reset",
        "proposal",
        "proposal_vote",
        "ledger_month",
        "site_text",
        "champion_trait",
    ];
    /// Personal data kept outside the database, by the variable that names
    /// where (M141 H2): each must be described in `STORED` and covered by
    /// the account deletion.
    const STORED_FILES: &[(&str, &str)] = &[("ATTACHMENTS_DIR", "screenshot")];

    #[test]
    fn files_outside_the_database_are_described() {
        let text: String = super::STORED
            .iter()
            .flat_map(|(_, items)| items.iter())
            .copied()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        for (var, word) in STORED_FILES {
            assert!(
                text.contains(word),
                "the privacy page does not describe what {var} holds ({word})"
            );
        }
        #[cfg(feature = "ssr")]
        for (var, _) in STORED_FILES {
            assert!(
                crate::server::config::VARIABLES
                    .iter()
                    .any(|v| v.name == *var),
                "{var} is not a configured variable"
            );
        }
        let account = include_str!("../../server/account.rs");
        assert!(
            account.contains("attachments::remove_all"),
            "account deletion does not delete feedback screenshots"
        );
    }

    /// Tables that hold nothing about a person or a team: migration
    /// bookkeeping, the champion data cache and patch changes. (Champion
    /// traits are listed above: they record the admin who edited them.)
    const NOT_PERSONAL: &[&str] = &["schema_migration", "ddragon_cache", "patch_change"];

    fn schema_tables() -> Vec<String> {
        include_str!("../../../schema.surql")
            .lines()
            .filter_map(|l| l.trim().strip_prefix("DEFINE TABLE IF NOT EXISTS "))
            .filter_map(|rest| rest.split_whitespace().next())
            .map(|t| t.trim_end_matches(';').to_string())
            .collect()
    }

    /// The page states the default retention of the match caches.
    #[cfg(feature = "ssr")]
    #[test]
    fn match_cache_retention_matches_the_default() {
        assert_eq!(
            crate::server::config::DEFAULT_MATCH_CACHE_RETENTION_DAYS,
            365,
            "the privacy page says 12 months"
        );
        assert!(super::MATCH_CACHE_RETENTION.contains("12 months after they were fetched"));
        assert!(super::RANKED_SNAPSHOT_RETENTION.contains("12 months after the sync"));
    }

    /// Every purpose has a stated legal basis; no placeholder is left.
    #[test]
    fn every_purpose_has_a_legal_basis() {
        assert_eq!(super::LEGAL_BASES.len(), 5);
        for (purpose, basis) in super::LEGAL_BASES {
            assert!(!purpose.contains("OWNER") && !basis.contains("OWNER"));
            assert!(basis.starts_with("Legal basis: "), "{basis}");
            assert!(basis.contains("Art. 6(1)("), "{basis}");
        }
    }

    #[test]
    fn the_listed_tables_exist() {
        let tables = schema_tables();
        for t in LISTED.iter().chain(NOT_PERSONAL) {
            assert!(
                tables.iter().any(|s| s == t),
                "{t} is named for the privacy page but is not in the schema"
            );
        }
    }

    /// A new table must be described on the privacy page (and added to
    /// `LISTED`) or be marked as holding no personal or team data.
    #[test]
    fn every_table_is_accounted_for() {
        for t in schema_tables() {
            assert!(
                LISTED.contains(&t.as_str()) || NOT_PERSONAL.contains(&t.as_str()),
                "table {t} is not described on the privacy page"
            );
        }
    }
}
