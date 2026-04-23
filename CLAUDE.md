# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# LoL Team Companion

A League of Legends team management app for coordinating drafts, tracking stats, and planning games.

**Stack:** Rust nightly · Leptos 0.8 · Axum 0.8 · SurrealDB 3.x (SurrealKV) · Tailwind CSS v4

## Commands

```bash
# Dev server with live-reload (runs on :3020, reload on :3021)
cargo leptos watch

# Production build
cargo leptos build --release

# Fast type-check (SSR target, no WASM compile)
cargo check --features ssr

# Fast type-check (WASM/hydrate target)
cargo check --features hydrate --target wasm32-unknown-unknown

# Run server tests (integration tests OOM with BFD linker — use --lib)
cargo test --features ssr --lib

# Pipeline flow test: draft → game-plan linking (requires running server)
just flow-test

# Adjust log level (default: info, app crate: debug)
RUST_LOG=debug cargo leptos watch
```

## Setup

```bash
# Prerequisites: rust nightly, cargo-leptos, tailwindcss standalone binary (no npm)
cargo install cargo-leptos

# Environment
cp .env.example .env   # Add RIOT_API_KEY, optionally SURREAL_DATA_DIR
cargo leptos watch
```

The app runs at `http://127.0.0.1:3020` with live-reload on port 3021.

## Architecture

Single crate, dual compile target:
- **`ssr` feature** → Server binary (Axum + SurrealDB + Riot API)
- **`hydrate` feature** → WASM client (browser hydration)

`cargo-leptos` builds both targets and serves the WASM bundle.

### Directory Layout

```
src/
├── main.rs              # Axum server setup, DB init, session/auth layers
├── app.rs               # Router, shell, all <Route> definitions
├── lib.rs               # Crate root, re-exports app module
├── error_template.rs    # Fallback error page
├── server/              # SSR-only (behind #[cfg(feature = "ssr")])
│   ├── db.rs            # All SurrealDB queries
│   ├── auth.rs          # AuthBackend (axum-login), Credentials, password hashing
│   ├── session_store.rs # SurrealSessionStore for tower-sessions
│   ├── riot.rs          # Riot API client (riven crate)
│   ├── data_dragon.rs   # Champion data from Data Dragon CDN
│   └── leaguepedia.rs   # Pro play data scraping
├── models/              # Shared types (compiled for both SSR and WASM)
│   ├── user.rs          # AppUser, TeamMember
│   ├── team.rs          # Team
│   ├── draft.rs         # Draft, DraftAction
│   ├── match_data.rs    # PlayerMatchStats
│   ├── game_plan.rs     # GamePlan, PostGameLearning
│   └── champion.rs      # Champion metadata
├── pages/               # Route components (each may contain #[server] fns)
│   ├── home.rs          # Landing page (auth-aware with CTA)
│   ├── profile.rs       # Riot account linking, champion pool summary
│   ├── auth/login.rs    # Login form
│   ├── auth/register.rs # Registration form
│   ├── team/dashboard.rs # Roster slots, coach slots, join requests
│   ├── team/roster.rs   # Team creation/joining
│   ├── team_builder.rs
│   ├── draft.rs         # Draft planner with blue/red side toggle
│   ├── tree_drafter.rs  # Tree-based draft planning with graph view
│   ├── stats.rs         # Match history + stats
│   ├── champion_pool.rs # Tier-based champion pool management
│   ├── game_plan.rs     # Pre-game strategy with champion autocomplete
│   └── post_game.rs     # Post-game review with pattern analysis
├── components/          # Reusable UI
│   ├── nav.rs           # Top nav bar with notifications, theme toggle
│   ├── champion_picker.rs # Grid-based champion selection
│   ├── champion_autocomplete.rs # Text input with champion dropdown
│   ├── draft_board.rs   # 20-slot draft board (picks + bans)
│   ├── tree_graph.rs    # SVG tree visualization with champion edge icons
│   ├── theme_toggle.rs  # Dark/light mode + accent color picker
│   ├── stat_card.rs
│   └── ui.rs            # ErrorBanner, StatusMessage
schema.surql             # DB schema (loaded on startup via include_str!)
```

## Routes

| Path               | Component        | Auth Required |
|--------------------|------------------|---------------|
| `/`                | HomePage         | No            |
| `/auth/login`      | LoginPage        | No            |
| `/auth/register`   | RegisterPage     | No            |
| `/profile`         | ProfilePage      | Yes           |
| `/team/dashboard`  | TeamDashboard    | Yes           |
| `/team/roster`     | RosterPage       | Yes           |
| `/team-builder`    | TeamBuilderPage  | Yes           |
| `/draft`           | DraftPage        | Yes           |
| `/tree-drafter`    | TreeDrafterPage  | Yes           |
| `/stats`           | StatsPage        | Yes           |
| `/champion-pool`   | ChampionPoolPage | Yes           |
| `/game-plan`       | GamePlanPage     | Yes           |
| `/post-game`       | PostGamePage     | Yes           |
| `/opponents`       | OpponentsPage    | Yes           |
| `/action-items`    | ActionItemsPage  | Yes           |

## Critical Patterns and Gotchas

Path-specific patterns live in `.claude/rules/` and load automatically:
- **SurrealDB** (db.rs, *.surql): `.claude/rules/surreal-patterns.md`
- **Leptos/Server Functions/Reactivity/Auth** (pages/**, components/**): `.claude/rules/leptos-patterns.md`
- **WASM/browser testing** (src/**/*.rs, e2e/**): `.claude/rules/wasm-patterns.md`

### Toolchain

13. **BFD linker** — `.cargo/config.toml` forces `bfd` linker to work around an LLVM 22 lld crash on nightly.

14. **Standalone Tailwind** — Uses the `tailwindcss-linux-x64` binary directly, no npm/node required. Referenced in `[package.metadata.leptos]`.

15. **tower-sessions version** — Must match the version used by axum-login (currently 0.14). A mismatch causes silent session failures.

### External APIs

16. **riven crate** — `Queue::from(u16)` not `i16`; `puuid` field is `String` not `Option<String>`.

17. **Riot API key** — Set `RIOT_API_KEY` in `.env`. Loaded via `dotenvy::dotenv()` on startup.

## Adding a New Feature

### New page

1. Create `src/pages/my_page.rs` with a `#[component] pub fn MyPage()` and any `#[server]` functions
2. Add `pub mod my_page;` to `src/pages/mod.rs`
3. Add a `<Route>` in `src/app.rs`
4. Add a nav link in `src/components/nav.rs` if needed
5. Add a smoke test entry to `e2e/tests/pages.spec.ts` (`AUTHED_PAGES` array if auth-required, or `PUBLIC_PAGES` in `smoke.spec.ts` if public)
6. **Verify via agent-browser or e2e test**: navigate to the new route, snapshot, confirm it renders without errors

### New DB table

1. Add `DEFINE TABLE` / `DEFINE FIELD` statements to `schema.surql`
2. Add query functions to `src/server/db.rs` (with `Db*` struct → model conversion)
3. Add shared model struct to `src/models/`

### New model struct

- Shared structs go in `src/models/` — must compile for both SSR and WASM
- DB-facing structs with `RecordId` go in `src/server/db.rs` (SSR only)
- Derive `Serialize, Deserialize, Clone` on shared structs

### Testing

39. **Run tests with `--features ssr --lib`** — `cargo test --features ssr --lib` runs unit tests only. Integration tests in `tests/` OOM during BFD linking and are skipped. Unit tests live in `#[cfg(test)]` blocks in model and server files.

41. **Tree assembly: use `children_of` map, not reversal heuristic** — Building a `HashMap<String, Vec<String>>` of parent→child IDs and doing a recursive DFS is correct regardless of DB return order. The old reverse-child_ids heuristic failed when sibling nodes shared the same `sort_order`.

## Claude Code Dev Workflow

> These guidelines are for interactive Claude Code sessions. GSD agents follow plan-specific verification.

### Starting a dev session
1. Start dev server in background: `cargo leptos watch` (use `run_in_background`)
2. Wait for ready: `./scripts/wait_for_server.sh 120`
3. Register + log in via the e2e auth fixture or agent-browser so subsequent page visits are authenticated

### Browser verification

Use the Playwright e2e test suite and agent-browser skill for browser verification. **Run tests liberally** — they are the primary way to catch UI/runtime bugs that the compiler can't.

**Recommended after UI changes (interactive sessions):**
1. Run a targeted Playwright test: `cd e2e && npx playwright test <spec> -g "<test name>"`
2. Or take a quick screenshot: `npx agent-browser screenshot http://127.0.0.1:3020/<route>`
3. Check for missing elements, broken text, wrong state
4. If the change involves interaction (click, form submit, drag), run the relevant e2e test or use agent-browser to click through and snapshot again

**When to use agent-browser vs Playwright e2e tests:**
- **agent-browser (interactive):** Quick one-off checks during development via agent-browser skill. Navigate, screenshot, interact.
- **e2e tests (`just e2e`):** Regression suite. Run before committing or when validating multiple pages at once.

**Auth for agent-browser sessions:**
1. Use the `authenticatePage` helper from `e2e/tests/fixtures.ts` as a reference for the auth flow.
2. Register via `/auth/register` — registration auto-logs in and redirects to `/team/dashboard`.
3. All subsequent navigations in the same browser session are authenticated.

**Common verification patterns:**
- New page: navigate → screenshot/snapshot → confirm page-specific content visible, no error banners
- Form submission: fill fields → submit → snapshot → confirm success message or redirect
- List mutation (add/delete): perform action → snapshot → confirm list updated
- Error handling: trigger an error condition → snapshot → confirm error banner appears with useful message
- "No team" state: verify pages show empty state (not error banners) for users without a team

### After making code changes
- `cargo leptos watch` auto-recompiles on file save
- rust-analyzer LSP provides real-time type errors
- `just check-ssr` for a quick compile check when LSP is insufficient
- **Verify the affected page in the browser when feasible** — use agent-browser or `cd e2e && npx playwright test` — compiler checks alone miss rendering bugs, wrong CSS, missing data, and broken interactions

### Debugging reactive bugs without a browser
Many WASM/Leptos runtime bugs (UI freezes, stale data, broken interactions) are **signal lifecycle issues** that the Rust compiler cannot catch. When diagnosing these:
1. **Trace the signal flow** — read the Effect, identify which signals are tracked (`.get()`) vs read untracked (`.get_untracked()`), and check if any are read lazily inside delayed callbacks (`Closure::once`, `setTimeout`)
2. **Check for stale captures** — if a closure captures a signal value lazily, another part of the code may have updated that signal by the time the closure runs (e.g. node switch updates `node_label` but the old timer callback reads `node_label.get_untracked()` → saves new node's data to old node's ID)
3. **Look for missing teardown** — switching contexts (node, tree, tab) should cancel pending timers and suppress auto-save Effects before updating signals
4. **Verify with both `cargo check --features ssr` and `cargo check --features hydrate --target wasm32-unknown-unknown`** — some bugs only surface in one target (e.g. unused variable warnings from `#[cfg(feature = "hydrate")]` guards)

### Periodic / before committing (optional, recommended for interactive sessions)
- `just verify` — check both targets + test + lint + fmt
- `just smoke` — health check + API smoke tests + pipeline flow test (requires running server)
- `just flow-test` — pipeline flow test: draft → game-plan linking (requires running server)
- `just e2e` — full Playwright e2e suite (requires running server)

## Plugins & MCP Servers

- **agent-browser** — Vercel's agent-browser skill for interactive browser verification during development. Installed as a Claude Code skill in `.claude/skills/agent-browser/`.
- **Context7** — up-to-date library docs; add "use context7" to prompts for live documentation lookup
- **GitHub MCP** — rich GitHub integration (PRs, issues, code search) via `GITHUB_TOKEN` env var

## Design System

Design tokens, color, typography, spacing, and accessibility standards are inherited from the Manyfold wiki:
- Token catalog + Tailwind `@theme` block: `../professional-vault/wiki/concepts/design-system.md`
- Component rules: `../professional-vault/wiki/concepts/ui-guidelines.md`
- Accessibility: `../professional-vault/wiki/concepts/accessibility-standards.md`

This project uses **standard** density. Semantic tokens (`bg-base`, `bg-surface`, `text-primary`, etc.) are defined in `input.css` `@theme` block — use these, not hardcoded colors.

**UI-SPEC.md scope (`/gsd-ui-phase`):** Cover project-specific decisions only — page/route inventory, draft board layout, tree graph interactions, auth flows, champion picker UX. Do not re-specify tokens, colors, typography, or accessibility rules already in the wiki.

## Code Style

- **Errors:** `thiserror` for custom error types, map to `ServerFnError` via `.map_err(|e| ServerFnError::new(e.to_string()))`
- **Theming:** Uses CSS custom properties via `@theme` in `input.css`. Dark theme is default. Use semantic tokens (`bg-base`, `bg-surface`, `bg-elevated`, `text-primary`, `text-secondary`, `text-muted`, `text-dimmed`, `border-divider`, `border-outline`, `bg-accent`, `text-accent-contrast`) instead of hardcoded colors.
- **Exceptions to semantic tokens:** Colored buttons with white text (e.g. `bg-red-700 text-white`, `bg-blue-500 text-white`) keep `text-white` literally, not `text-primary`.
- **Forms:** Tailwind utility classes inline, `bg-surface/50` inputs with `border-outline/50`
- **Naming:** snake_case files and functions, PascalCase components
