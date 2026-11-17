# M160 item 11 — CHANGELOG entries (mechanical)
Read CHANGELOG.md (Keep a Changelog) and CLAUDE.md "Adding things → Changelog": entries go under `## [Unreleased]` (or however the file names the unreleased section — read it), in the style and section headings (Added / Changed / Fixed / Removed) the file uses for recent missions (look at the M148, M152, M159 or M165 entries and copy their shape exactly, including whether they group under a `### Milestone ...` heading; if they do, add `### Milestone M160: Honesty and the first ten minutes (2026-11-17)` and NEVER rename existing headings). A unit test parses the file: run `cargo test --features ssr --lib changelog` (memory permitting) and make it pass.
Write one plain-language entry per shipped change (user words, no internal names, no "Riot" in product copy except where the file already does):
- Added: Without a team, the navigation shows your own pages and a "Find or create a team" link; the draft, game plan and review pages say a team is needed instead of failing.
- Changed: The getting-started checklist asks a new team to invite a teammate before filling the starting five.
- Added: A player who joins with an invite link for a position is placed in that starting slot when it is free; the leader and coaches see a notice with Undo.
- Changed: The roster page is part of the team page; old /team/roster links still work.
- Added: Coaches can place players (not the leader or other coaches) and answer join requests; join-request notices reach coaches too.
- Changed: The reminders about drafts without a plan and dossiers without a draft go to the leader and coaches only.
- Fixed: Error messages no longer show technical text such as "error running server function" or server setup instructions; Riot-off messages say the feature is off on this server.
- Fixed: When a list or panel cannot load, the page says so with "Try again" instead of looking empty.
- Added: Notes per opponent player on the dossier (collapsed until opened, saved as you leave the field).
- Added: Draft trees can be renamed, and their opponent is picked from the team's dossiers.
- Removed: nothing user-visible (do not list the deleted dead server functions).
Optionally one `Introduce: /team/dashboard | Coach rights | ...` line ONLY if the file's parser documents that format (read the unit test) — skip if unsure.
