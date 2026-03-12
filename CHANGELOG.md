# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Context7 MCP server for live library documentation lookup (Leptos, SurrealDB, Axum, etc.)
- GitHub MCP server for rich GitHub integration (issues, PRs, code search) via `GITHUB_TOKEN`
- GSD (Get Shit Done) planning system with `/gsd:*` slash commands for spec-driven development
- Skill-Creator meta-skill for building custom Claude Code slash commands
- Pre-commit docs check skill (`/pre-commit-docs`) — reviews CHANGELOG, TODO, CLAUDE.md, INSTRUCTIONS.md before every commit
- `GITHUB_TOKEN` in `.env.example` for GitHub MCP server
- "Plugins & MCP Servers" section in CLAUDE.md
- Team dashboard: pencil-icon modal for renaming team (replaces inline form), conditional Riot account notice with link to profile
- Tree graph: reactive SVG node highlighting with accent color glow filter; two-way graph/editor sync (click graph node selects in editor and vice versa); new branches auto-select after creation
- Stats page: queue type selector (Solo/Duo, Flex, All Queues) for match sync; OP.GG-style match history layout with champion icons, KDA, CS, vision, damage, win/loss row tinting; expandable match detail panel showing team members' stats per match
- `verify-implementation` skill for post-implementation testing (compile, browser, e2e, regression sweep)
- Stats page: animated spinner on Sync button + blue progress banner during match sync; "Team Games / All Matches" segmented toggle to switch between team-only (min_players≥2) and all matches (including solo queue)

### Fixed

- Stats: matches not appearing after sync — `player_match` writes silently discarded by `.ok()` (now `.check()?`), and `get_team_match_stats` deserialization failed on `datetime` field (added `<string>` cast) and `match` record ref returned as `NONE` by `SELECT *` traversal (now `Option<RecordId>`)
- 23 clippy warnings: unnecessary casts in riot.rs, clone-on-copy in draft_board.rs, redundant closure in register.rs, type complexity in post_game.rs, manual_memcpy in tree_drafter.rs, too_many_arguments in db.rs
- E2e test fixture: updated for registration auto-login (no longer does register+login two-step)
- E2e WASM Effect race condition: added 500ms settle delay after registration redirect to prevent `window.location.set_href` from interrupting subsequent navigations
- Playwright MCP config: changed browser from `chrome` to `chromium` in `.mcp.json`

## [0.16.0] - 2026-03-11

### Fixed

- Tree drafter UI freeze after branching: auto-save Effect now captures all signal values eagerly instead of reading lazily inside timer callback, preventing stale data saves
- Tree drafter node switch bug: `select_node` now suppresses auto-save during signal batch updates via `suppress_autosave` flag + microtask re-enable, with `cancel_autosave_timer` and `clear_editor` helpers
- Tree drafter cannot switch between trees: tree click handler now calls `clear_editor()` to fully reset editor state; removed redundant `nodes_resource.refetch()` (resource already keyed on `selected_tree_id`)
- Tree graph too many icons on edge: reduced `MAX_ICONS` from 5 to 3; `diff_actions` now filters empty champion names

## [0.15.0] - 2026-03-11

### Added

- Auth-aware nav: Team, Draft, Tree Drafter, Stats, Game Plan, and Post Game links only visible when logged in
- Registration auto-login: `register_action()` now authenticates the user immediately after account creation, redirecting to `/team/dashboard`
- Auth redirect on all protected pages: unauthenticated users are redirected to `/auth/login` instead of seeing broken/empty pages
- Empty-state CTAs on team-dependent pages (tree drafter, stats, game plan, post-game): show "Create or join a team" message with link to `/team/roster` when no team exists
- CLAUDE.md gotchas 51–52: registration auto-login, auth-aware nav links

### Fixed

- Logout now hard-navigates to `/` (both nav and profile page), fully clearing session state (CLAUDE.md rule 8/49)
- 9 rule-44 violations: server functions now return empty lists instead of errors when user has no team (`list_trees`, `get_team_stats`, `list_plans`, `list_team_drafts`, `list_reviews`, `list_plans_for_postgame`, `list_drafts_for_postgame`, `get_recent_match_ids`); `sync_team_stats` returns user-friendly error
- Missing `.check()` on profile UPDATE query — username changes no longer fail silently

### Changed

- CLAUDE.md: updated gotchas 49 (logout now fixed) and 50 (protected pages now redirect)
- `e2e/tests/auth.spec.ts`: un-skipped "logout clears session" test; improved "register → auto-login" test to verify redirect and username presence

## [0.14.1] - 2026-03-11

### Added

- Full e2e audit of all 13 routes — documented bugs, UX issues, and rule-44 violations in TODO.md
- CLAUDE.md gotchas 45–51: WebFetch localhost limitation, Leptos HTML text extraction, tailwind 404, e2e auth fixture, logout hard-nav, protected page behavior, `just` availability

### Changed

- `e2e/tests/fixtures.ts`: auth fixture now always registers then logs in (registration does not auto-login)
- `e2e/tests/smoke.spec.ts`, `pages.spec.ts`: filter Tailwind v4 `@import "tailwindcss"` 404 from error assertions
- `e2e/tests/auth.spec.ts`: logout test marked `test.fixme` (logout is broken — session not cleared); fixed test to match actual app behavior (inline "not logged in" message, not redirect)
- CLAUDE.md auth workflow updated to reflect two-step register→login requirement

### Fixed

- `e2e/tests/auth.spec.ts`: "logout clears session" test was incorrectly asserting a redirect to `/auth/login` — protected pages show inline messages, not redirects

## [0.14.0] - 2026-03-11

### Added

- Playwright MCP server config (`.mcp.json`) for browser interaction during development — navigate pages, snapshot accessibility trees, click elements, fill forms
- `scripts/wait_for_server.sh` — polls `/healthz` until the dev server is ready (configurable timeout)
- Justfile recipes: `verify` (full validation), `smoke` (runtime tests), `wait-for-server`, `full-check` (verify + e2e)
- E2e test fixtures (`e2e/tests/fixtures.ts`) — shared auth helper that registers + logs in a test user
- Authenticated page smoke tests (`e2e/tests/pages.spec.ts`) — smoke tests for all 10 protected routes checking for JS errors, correct rendering, and no login redirect
- Claude Code dev workflow section in CLAUDE.md documenting browser verification patterns with Playwright MCP

### Changed

- Expanded `.claude/settings.local.json` permissions: `cargo:*`, `just:*`, `curl:*`, `./scripts/*`, MCP tool access
- "Adding a New Feature > New page" checklist now includes e2e smoke test and MCP verification steps

### Fixed

- `Cargo.toml`: `end2end-dir` corrected from `"end2end"` to `"e2e"` to match actual directory
- `CLAUDE.md`: dev server ports corrected from `:3000`/`:3001` to `:3002`/`:3003`

## [0.13.0] - 2026-03-11

### Added

- Stats: "Minimum players" dropdown (2 → roster size) replaces the binary "full roster only" checkbox — shows all matches where ≥ N linked team members played together

### Fixed

- Draft list: `list_drafts` now returns an empty list (not `Err`) when the user has no team, preventing the saved drafts panel from erroring out entirely
- Draft save: saving with an empty name now shows an inline error instead of silently creating an unnamed draft record
- Tree graph edges: champion icons now centered around the edge midpoint (previously left-aligned from midpoint); capped at 5 with a `+N` overflow text badge when more icons are present
- Team dashboard: starter role slots no longer show both the SVG role icon and a duplicate text label; icon is now 24 px with the role name as a `title` tooltip

## [0.12.0] - 2026-03-11

### Added

- Draft planner: 2 s debounced auto-save for existing named drafts; "✓ Saved" / "● Unsaved changes" status indicator in header
- Tree drafter: 2 s debounced auto-save per node with save status indicator next to Save Node button
- Tree drafter: Enter key on tree name / opponent inputs triggers Create Tree
- Draft board: highlight-first slot deletion — click a filled slot to highlight it (red border + × badge); click × to remove; click elsewhere to deselect
- Champion autocomplete: optional `on_select` callback prop fired on dropdown item click
- Champion pool: clicking an autocomplete suggestion now immediately adds the champion (no separate button press)
- Champion pool: all tiers always rendered from the start; empty tiers show "No champions yet" placeholder

### Fixed

- Join request accept silently failing: errors now surfaced with per-request inline error message; dashboard and request list refetch on success
- Team owner no longer missing from roster: `create_team()` in `db.rs` now immediately inserts the owner as a `team_member` (`role = 'unassigned', roster_type = 'sub'`)
- Tree drafter root node now labelled with the tree name instead of hardcoded "Root"

## [0.11.0] - 2026-03-11

### Added

- Full integration test suite: 44 tests (19 unit + 25 integration) across `db_users`, `db_teams`, `db_drafts`, `db_tree`, `db_champion_pool`
- In-memory SurrealDB test helper (`tests/common/mod.rs`) for fast, isolated integration tests
- GitHub Actions CI pipeline: SSR + WASM type-checks, tests, clippy, rustfmt
- `justfile` with `check`, `test`, `watch`, `build` recipes
- `.env.example` with all required and optional environment variables
- `delete_draft()` server function and DB query

### Fixed

- `list_pending_join_requests`: removed `ORDER BY created_at` from partial SELECT (SurrealDB 3.x rejects fields not in partial SELECT)
- `get_tree_nodes` tree assembly: replaced bottom-up heuristic with recursive DFS using `children_of: HashMap` — fixes node ordering when siblings share the same `sort_order`

## [0.10.0] - 2026-03-11

### Fixed

- WASM runtime crash caused by `.unwrap()` in event handlers (nav Escape listener, drag-and-drop) that froze all subsequent user interactions

## [0.9.0] - 2026-03-11

### Added

- SVG-based tree graph visualization (`src/components/tree_graph.rs`) with top-down layout algorithm
- Champion icons on tree graph edges showing picks/bans diff between parent and child nodes
- Ban indicators (red border + cross overlay) vs pick indicators (green border) on edge icons
- List/graph view toggle in tree structure panel header

## [0.8.0] - 2026-03-11

### Added

- CSS custom property-based theming system with semantic color tokens (`input.css`)
- Dark/light mode toggle (moon/sun icon) with localStorage persistence
- Anti-FOUC inline script in HTML head to prevent flash on page load
- Accent color picker with 5 palettes (yellow, blue, purple, emerald, rose)
- Theme toggle component (`src/components/theme_toggle.rs`)

### Changed

- Replaced ~285 hardcoded Tailwind color classes with semantic tokens across all pages and components
- Body class changed from `bg-gray-950` to `bg-base text-primary` for theme support

## [0.7.0] - 2026-03-11

### Added

- Nav dropdowns close on outside click (transparent backdrop), Escape key, and link click
- Notification dropdown with inline accept/decline for join requests
- Team dashboard: coach role slots (2 slots below starting 5)
- Team dashboard: "Leave Team" button for non-leaders with confirmation
- Team dashboard: leader badge (star icon) next to team leader
- Drafts: blue/red side toggle on draft form
- Drafts: auto-populate game plan champion inputs from selected draft
- Champion autocomplete component with Data Dragon icons (`src/components/champion_autocomplete.rs`)
- Champion pool standalone page (`/champion-pool`) with tier system (comfort, match ready, scrim ready, practicing, should be practiced) and notes
- Profile: champion pool summary with link to full pool page
- Tree drafter: "Branch from here" button to create branch from a selected draft position
- Tree drafter: enlarged node +/x buttons with improved hover states

### Fixed

- Removed duplicate "Team Settings" from profile dropdown menu
- Fixed game plan save "Connection uninitialised" error (missing `.check()` on CREATE query)
- Fixed Tree drafter Live Game button not activating immediately

## [0.6.0] - 2026-03-10

### Added

- Dashboard page: team summary, draft/plan/review counts, recent game stats, win rate
- Landing page for unauthenticated users with call-to-action
- Alert banners: pending join requests, no team, missing API key
- Consistent sticky header with backdrop blur, notifications dropdown, user avatar menu
- Mobile-responsive navigation with hamburger menu
- Reusable `ErrorBanner` and `StatusMessage` UI components
- Consistent error display across all pages (standardized to ErrorBanner)

## [0.5.0] - 2026-03-10

### Added

- Post-game review page with structured feedback fields and open-ended notes
- Link post-game reviews to actual matches from stats
- Link post-game reviews to original game plan and draft
- Pattern analysis for recurring insights

## [0.4.0] - 2026-03-10

### Added

- Game plan system for pre-game strategy tied to specific matchups (your 5 champs vs enemy 5)
- Macro strategy section (team-wide) and 5 role-specific sections
- Link game plans to drafts
- Template-based auto-generation of strategy suggestions

## [0.3.0] - 2026-03-09

### Added

- Match history sync from Riot API (manual refresh)
- Filter for games with all 5 roster players
- Stats dashboard with date range and opponent filters
- Clear messaging when RIOT_API_KEY is missing

## [0.2.0] - 2026-03-09

### Added

- Draft tree system at `/tree-drafter` (separate from existing `/draft`)
- Tree data model: `DraftTree` + `DraftTreeNode` with parent/child relationships
- Tree visualization: indented list with expand/collapse
- Node editor: full draft board per node with notes
- Live game navigator: step-by-step branch selection
- Improvisation mode: create branches mid-game, tagged as improvised
- Join request system: players request to join, leader accepts/declines
- Nav badge: red dot on Team link shows pending request count (leaders only)
- Substitute roster: new members land on the bench after joining
- Starter slots: 5 role positions (Top/Jungle/Mid/Bot/Support) with drag-and-drop assignment
- Leader can kick members and edit team name/region
- Role dropdown per bench member

## [0.1.0] - 2026-03-09

### Added

- Project scaffold with Leptos 0.8, Axum 0.8, and SurrealDB 2.x (SurrealKV)
- User authentication (register, login, sessions) with axum-login and argon2 password hashing
- User profile page with Riot account linking via PUUID
- Team creation, dashboard, and roster management
- Team builder page
- Draft planner with champion picker component and draft board UI
- Match stats page with player match history from Riot API (riven crate)
- Game plan page for pre-game strategy (win conditions, objective priority, teamfight strategy)
- Post-game review page (what went well, improvements, action items)
- Data Dragon integration for champion metadata and images
- Leaguepedia module for pro play data (not yet wired to UI)
- Tailwind CSS v4 with dark theme (standalone binary, no npm)
- Environment config via dotenvy (.env file support)
- Full SurrealDB schema with tables: user, team, team_member, match, player_match, draft, draft_action, game_plan, post_game_learning, sessions

### Fixed

- SurrealDB 2.x compatibility: migrated from `type::thing()` to `type::record()` throughout all queries
- SurrealDB 2.x compatibility: removed `string()` casts, use `RecordId::to_sql()` in Rust instead
- SurrealDB RecordId deserialization: introduced `Db*` intermediate structs to avoid `serde_json::Value` failures
- Linker configuration: added `.cargo/config.toml` with bfd linker to work around LLVM 22 lld crash on nightly
- Tailwind binary: switched to standalone `tailwindcss-linux-x64` to eliminate npm dependency
- Auth state refresh: use hard navigation (`window.location().set_href()`) after login/register instead of server redirect
- ActionForm styling: wrap in `<div>` since Leptos 0.8 `ActionForm` does not support `class` prop
- Session store: implement `SessionStore::create` to properly handle ID collisions instead of relying on default `save`-based fallback
