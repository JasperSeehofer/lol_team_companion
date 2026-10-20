# M160 item 1 — Solo mode (spec + acceptance)
Repo worktree: /home/ops/Repositories/lol_team_companion-wt/M160-team-companion-honesty-and-first-ten-minutes (branch mission/M160-...). Follow its CLAUDE.md and .claude/rules/*.md.
Acceptance (all must hold):
1. One pure function in src/components/nav.rs decides visible hubs/pages from a has_team flag; desktop hub bar, phone menu and command bar all use it. With has_team=false only the You hub (plus home via the wordmark) is shown, plus one quiet link "Find or create a team" -> /team/dashboard. Prep, Review, Team hubs are absent (not greyed).
2. Unit tests in nav.rs cover: has_team=false hides Prep/Review/Team in all three surfaces; has_team=true unchanged; existing invariants (every page has a hub; each page appears once in the phone menu; command bar lists all) still pass for the with-team model.
3. /draft, /game-plan, /post-game render NoTeamState (src/components/ui/empty.rs) instead of the editor when the user has no team, using the same no-team signal as /opponents or /stats. Edits to draft.rs/game_plan.rs/post_game.rs stay a narrow wrapper (≤ ~15 lines each).
4. NoTeamState copy names what is missing per page and its single CTA links /team/dashboard; existing callers still compile.
5. No server fn behaviour change (direct calls still error "You must be in a team ...").
6. e2e/tests/solo-mode.spec.ts: fresh authedPage account sees no Prep/Review/Team hubs (desktop 1280 and phone menu ~400px), sees "Find or create a team", and /draft, /game-plan, /post-game show the no-team state. Listed in .github/workflows/ci.yml Playwright step. Edited existing specs (bug-report, hydration-no-panic, match-day) must still make sense for accounts with/without team.
7. Gates on host (CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0, one cargo at a time): cargo fmt --check; cargo check --features ssr; cargo check --features hydrate --target wasm32-unknown-unknown; cargo test --features ssr --lib; ./scripts/lint_design.sh — all pass.
