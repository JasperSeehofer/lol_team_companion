# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions will follow semantic versioning once releases start.

## [Unreleased]

### Milestone M1: Security hardening

#### Added
- Invite codes are checked on the server. Registering needs a valid code; each code has a use limit, an optional expiry and can be disabled.
- Invite management page for admins (`/admin/invites`): list codes with their uses and expiry, mint a code, disable a code. Admins are set with `ADMIN_EMAILS`; everyone else gets "Page not found".
- The seed tool prints an invite code for local registrations.
- Rate limits on sign-in, registration and everything that calls the Riot API, and a one-minute pause between match syncs. Hitting a limit shows "Too many requests. Please slow down..." with the wait time.
- `/healthz` reports HTTP 503 when the database does not answer.

#### Changed
- Passwords must have at least 10 characters.
- Email addresses are case-insensitive for sign-in and registration; usernames must be unique regardless of case.
- Registration failures for an invalid invite or an email that already has an account show one generic message.
- Error messages no longer contain internal database or Riot API details.
- Sessions expire on the server and expired ones are removed every hour; the session cookie is marked `Secure` outside development. Existing sessions end once with this update (sign in again).
- A player can belong to one team at a time.
- The team list on the roster page needs a signed-in user.

#### Fixed
- Security: a signed-in user could read, change or delete other teams' drafts, draft trees, game plans, checklists, reviews, action items, team notes, opponents and private personal learnings by guessing IDs, and opponent scouting worked without signing in. Every request is now checked against the user's own team or account.
- Security: tree branches could be used to run arbitrary database queries (query injection). All values are now passed as parameters.
- Security: a team leader could accept join requests sent to other teams, and accept the same request twice.
- Personal learnings were never listed (a date field failed to load).

#### Removed
- The unused Leaguepedia pro-play scraper.

### Milestone M0: Reset and safety net

#### Added
- Site footer on every page with Riot Games' legal disclaimer and links to the legal pages.
- A real "Page not found" page for unknown addresses (HTTP 404).
- `MISSION.md`, `docs/ARCHITECTURE.md`, `docs/DESIGN.md` (skeleton for M2) and this changelog.
- SessionStart hook for cloud sessions (cargo-leptos, clippy/rustfmt, e2e dependencies, Playwright browsers, `.env`).
- CI now runs clippy for the WASM target and the Playwright smoke and hydration suites against a built server.

#### Changed
- Toolchain moved from nightly to stable Rust; the BFD linker workaround is gone.
- Debug builds carry line tables for this crate only (no dependency debug info), which shrinks test and server binaries enough to link in a small container.
- The browser tab title is now "Companion".
- Tree drafter: the empty tree list now tells team members how to create a tree instead of asking them to join a team.
- DB integration tests are one test binary (`tests/integration/`), so `cargo test --features ssr` links once.
- Bug-report export moved from `.planning/INBOX/` to `docs/inbox/` (git-ignored).
- Code formatted with `cargo fmt`; clippy is clean with `-D warnings` on both targets.

#### Removed
- Invented data and dead controls: solo dashboard pool-gap suggestions, placeholder KDA/CS stats, the Forge tab's sample champions and "Queue" buttons, the no-op sort buttons; the team dashboard's Game Day Brief (fake bans, threats and results) and the Pandemonium dashboard panels; the War Table's fake composition scores and the draft carousel's fake confidence line. Each now shows an honest empty state or is gone.
- The disabled "Live" hub and the dead "Match" link in the navigation.
- The Pandemonium theme option and theme toggle; every request now renders the single dark theme.
- Old planning tree (archived under the git tag `planning-archive-2026-10`), dangling GSD agent links and stray test output.

#### Fixed
- Login and closed-beta pages: background images no longer push the card off-screen.
- Every page requested `/pkg/tailwindcss` and got a 404; the stylesheet is now built only by the Tailwind pipeline.
- The tree drafter showed the literal text `&#x1f333;` instead of an icon.
- Warning, success and info badges rendered grey instead of their status colours.
- The draft ledger view assigned phase-two bans to the wrong side.
