# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions will follow semantic versioning once releases start.

## [Unreleased]

### Milestone M2: Design system foundation (stage B)

#### Added
- A command bar: press Ctrl+K (Cmd+K on a Mac), or use "Go to" in the navigation, type a few letters and press Enter to jump to any page. Commands for drafting and Match Day arrive with those pages.
- The account menu, dialogs, menus, notifications, tooltips, switches and segmented controls are now one consistent, fully keyboard-operable set. Notifications appear bottom left, can carry an action such as "Undo", and pause while you point at them.
- New gallery sections for every interaction and domain component, including the coordinate frame, legend, surveyor's notes and replay timeline reserved for the Sand Table and the Draft Graph.

#### Changed
- Roles are shown as short labels (TOP, JGL, MID, BOT, SUP) instead of icons loaded from a third-party site; your browser now loads nothing from outside Companion.
- Champion pictures show a two-letter monogram while they load or when an image is missing.
- Section headings say what the section is ("Starting roster", "Coaches", "Bench", "Recent games", "Team notes", "Open action items", "Recent reviews", "Pool gaps") instead of fantasy labels such as "Order of battle" or "Battle log".
- Every page uses the design tokens: side colours only for sides (match timeline events now use the blue and red side colours instead of status colours), results in their own colours, statuses with words, one rounded shape only for avatars and switches, and loading placeholders that stop moving when you ask your system for reduced motion.
- The destructive button turns dark crimson on hover instead of a pale pink.
- The first-pick choice on the draft board and the Improvised flag in the tree drafter are now a segmented control and a switch.

#### Removed
- The per-user theme setting stored in the database (there is one theme); a migration removes the stored values.
- The "resource read outside Suspense" warning from the bug-report button on every page load.

### Milestone M2: Design system foundation (stage A)

#### Added
- A new visual language, "Survey": a dark green-slate ground in tonal layers, one chalk accent for focus, selection and the next action, matched blue and red side colours, win and loss on their own colours, and plain, precise type. Every text and control colour meets WCAG AA contrast.
- New typefaces, served by the app itself: Familjen Grotesk for titles, Public Sans for text, JetBrains Mono for numbers, Source Serif 4 italic for annotations. Numbers line up in every table and readout, and no text is smaller than 12px.
- A design-system gallery at `/design` for admins (and developers): every component in every state, the colour tokens with their measured contrast, the type scale, a worked tournament draft and the Match Day preparation checklist.
- A tournament draft layout that shows the real order (five bans and five picks per side; blue starts, red opens the second ban phase) with the current step highlighted, and a "what to do next" checklist for Match Day. Pages adopt them in M3 and M5.
- A "Skip to content" link for keyboard users and a visible focus outline everywhere.

#### Changed
- Champion and item pictures load from Companion's own server (cached per patch) instead of from Riot's image servers, so your browser no longer contacts a third party for them.
- Sign-in, registration, the closed-beta page, the 404 page, the navigation, the footer and the headers of the solo and team dashboards use the new components; every other page already wears the new colours and type and is rebuilt in M3 and M5.
- The solo dashboard's views are called Overview, Rank and Journal; the last ten results show W and L marks.

#### Removed
- The heraldic ornaments, the painted background images, gradients, glows and blur, emoji icons and the leftover second theme.
- The theme card on the profile page, which pointed at a switch that no longer exists.

### Milestone M6: Data layer and integrations

#### Added
- Riot routing follows each player's region from their profile (and the team's region for team syncs and opponent scouting) instead of always using EUW. The profile only accepts the listed regions.
- When the app's Riot API budget is used up, Riot features answer "Riot is busy, try again in N s." instead of stalling.
- Champion images and data follow the current patch automatically (checked every 6 hours), and champion lists still work after a restart without internet access. New patches are recorded for later display.
- Storage for the upcoming Sand Table: imported match timelines (per-minute positions and economy, positioned events) and versioned tactics boards. No screens yet.
- The seed tool creates fuller demo data (two complete teams with drafts, plans, reviews, action items, notes and scouting) and can be run repeatedly.

#### Changed
- Usernames are unique regardless of case at the database level; a few accounts from before M1 whose names differed only by case get a numeric suffix (for example `bob-2`).
- A player can only ever be in one team, now also enforced by the database; extra memberships from before M1 are removed (the team the player leads, or else their first team, is kept).
- Saving a draft, a review, an opponent with players, a draft tree, a team or a deletion with its children either completes fully or not at all.
- The team match sync can run once a minute per team (it was per member).

#### Fixed
- The team dashboard's "drafts without a game plan" and "game plans without a review" counts counted everything.
- The 7/30/90-day windows for LP history and champion trends did not filter by date.
- Several lists could come back empty when a stored date failed to load (the same cause as the personal learnings bug); dates now load the same way everywhere.
- Invite codes could show the wrong status within the second they expired.
- Two people registering the same username (or email) at the same moment could both be told it worked while only one account was created; a tactics board saved twice at once could lose one version.
- Linking a Riot account or scouting an opponent failed for the OCE, SG, TW, VN and ME regions.
- When Riot was busy, a match sync also started the one-minute pause, so the suggested retry was refused.
- Accounts from before M1 whose email addresses differed only by case could no longer sign in when none of them was stored in lowercase; the oldest of them can sign in again.

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
