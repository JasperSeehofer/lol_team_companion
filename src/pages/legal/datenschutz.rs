//! Privacy (Datenschutz). Lists exactly what Team Companion stores (checked
//! against `schema.surql` by the tests below), who else is involved, the one
//! cookie, the logs, how long things are kept and how to use your rights.
//! The operator's facts (controller, providers, log retention, supervisory
//! authority) are the owner's own (launch checklist, 12 October 2026); the
//! legal bases are still an `[OWNER: ...]` placeholder. Nothing here is
//! invented.

use crate::components::footer::RIOT_DISCLAIMER;
use crate::components::ui::{
    Callout, CalloutTone, PageHeader, PageShell, PageWidth, SectionHeader,
};
use crate::pages::legal::impressum::{
    OwnerPlaceholder, OPERATOR_ADDRESS, OPERATOR_EMAIL, OPERATOR_NAME,
};
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
            "Whether you are an admin (and whether that was set with the promote command), and when the account was created.",
            "Your settings: whether you get survey questions, whether guidance (the welcome, checklist, hints and nudges) is on, and whether teammates may share your player card by link.",
            "Guidance you have seen or turned off: which item (for example \u{201c}hint.draft_keys\u{201d}) and when, so it is not shown again.",
        ],
    ),
    (
        "Signing in",
        &[
            "A session cookie (HttpOnly, SameSite=Strict) and the session it points to on the server. A session ends after 30 days without use; ended sessions are deleted.",
            "Invite codes with how often they were used and, for a team invite, the team and position it leads to.",
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
        "Bug reports",
        &[
            "What you send with the bug-report button: the page, the element you pointed at, your text, the category and the window size. Open reports are also written to a file on the server for the maintainer to read.",
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
                    "One item is still open: the legal basis for each purpose, marked OWNER below. The operator confirms it with advice before Team Companion opens beyond the closed beta."
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
                <ul class=ul>
                    <li>"Your account, your team's workspace and your own pages: to provide the service you signed up for."</li>
                    <li>"Game data from Riot: to show your games and your team's games and, when your team scouts an opponent, the public match history of the players it names."</li>
                    <li>"Logs, rate limits and backups: to keep the service secure, available and recoverable."</li>
                    <li>"Bug reports, votes and survey answers: to decide what to fix and build next, with you."</li>
                </ul>
                <p class="mt-3"><OwnerPlaceholder what="the legal basis for each purpose above (for example Art. 6(1)(b) and 6(1)(f) GDPR), confirmed with advice" /></p>
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
                    <dd class="text-secondary">"A nightly copy of the database, encrypted on the server before it leaves, is kept in object storage at Hetzner Online GmbH, Falkenstein (Germany)."</dd>
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
                    <li>"Your account, your own pages, synced games and ranked snapshots: until you delete your account."</li>
                    <li>"Your team's records: until the team deletes them, or the team is deleted. When you delete your account, what you wrote for the team stays with the team, shown as written by a former member."</li>
                    <li>"Deleted team notes: one day, so they can be restored. Sessions: 30 days after the last visit."</li>
                    <li>"Bug reports: until an admin deletes them; when you delete your account, your text is removed from them."</li>
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
        "proposal",
        "proposal_vote",
        "ledger_month",
        "site_text",
        "champion_trait",
    ];
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
