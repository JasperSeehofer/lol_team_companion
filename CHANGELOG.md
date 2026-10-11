# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions will follow semantic versioning once releases start.

## [Unreleased]

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
