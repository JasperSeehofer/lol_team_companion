# Phase 6: Bug Fixes - Context

**Gathered:** 2026-03-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Eliminate 5 known regressions (BUG-01 through BUG-05, PLAN-02) and configure test infrastructure (TEST-02) so the app is stable and verifiable before adding new surface in subsequent phases. This phase also replaces Playwright MCP with Vercel's agent-browser skill and refactors existing e2e specs to use shared helpers.

</domain>

<decisions>
## Implementation Decisions

### Regression Test Coverage
- Every bug fix (BUG-01 through BUG-05) gets a dedicated e2e regression test
- All regression tests live in a single file: `e2e/tests/regression.spec.ts`
- Tests verify the fix works (correct behavior), not reproduce the original bug condition
- BUG-04 (hydration) gets a console assertion test checking for no hydration mismatch warnings
- BUG-05 (drag-and-drop) uses real Playwright `dragAndDrop()` events, not click-based proxies
- BUG-01 (tree drafter freeze) test branches from a node, then verifies editor interactivity (type label, click save, confirm response)

### Playwright CLI Infrastructure (TEST-02)
- Add page navigation helpers: `navigateTo('draft')` with proper waits (networkidle + hydration settle)
- Add common interaction helpers: `selectChampion()`, `fillDraftSlot()`, `createDraft()`, `saveGamePlan()`
- Centralize console error capture utility (currently copy-pasted across audit specs)
- Add visual regression snapshot support using Playwright's built-in `toMatchSnapshot()` / `toHaveScreenshot()`
- Helpers live in `e2e/tests/` as Playwright test utilities (not shared with external tools)
- Refactor ALL existing audit specs to use new helpers (not just new tests)

### Agent Browser Setup (TEST-02)
- Install Vercel's `agent-browser` as a Claude Code skill (not just global CLI)
- Replace Playwright MCP: remove from `.mcp.json`, use agent-browser skill instead
- Update CLAUDE.md browser verification instructions to reference agent-browser instead of MCP commands
- Agent-browser becomes the primary tool for interactive browser verification during development

### Bug Fix Verification Flow
- Each bug: fix code + write regression test + browser check via agent-browser
- One atomic commit per bug fix (fix + regression test together)
- Full e2e suite (`just e2e`) must pass green at end of Phase 6, including all refactored audit specs

### Plan Grouping & Execution Order
- 3 plans, executed sequentially:
  - **Plan 1:** Test infrastructure — Playwright CLI helpers, agent-browser skill setup, CLAUDE.md update, error capture utility, snapshot support
  - **Plan 2:** Bug fixes — All 5 bugs (BUG-01–05, PLAN-02) with regression tests in `regression.spec.ts`. Fix order: easiest first (BUG-03 roster, BUG-04 hydration, BUG-05 drag-drop) then harder (BUG-01 tree drafter, BUG-02/PLAN-02 game plan nav). One commit per bug.
  - **Plan 3:** Audit spec refactor — Refactor all existing audit specs to use new helpers + verify full suite green

### Claude's Discretion
- Exact helper function signatures and API design
- How to structure the error capture utility (class, function, fixture)
- Which existing audit specs need the most refactoring vs light touch
- Agent-browser skill configuration details
- Snapshot baseline management strategy

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Bug Specifications
- `.planning/REQUIREMENTS.md` — BUG-01 through BUG-05, PLAN-02, TEST-02 acceptance criteria
- `.planning/ROADMAP.md` — Phase 6 success criteria (6 items)

### Existing Test Infrastructure
- `e2e/tests/fixtures.ts` — Current auth fixture (`authenticatePage`, `createTeam`, `authedPage`, `teamPage`)
- `e2e/playwright.config.ts` — Test runner configuration
- `e2e/tests/pipeline.spec.ts` — Draft-to-game-plan roundtrip test (partially covers BUG-02)
- `e2e/tests/audit-tree-drafter.spec.ts` — Tree drafter audit (partially covers BUG-01)

### Bug Source Files
- `src/pages/tree_drafter.rs` — BUG-01: branch logic (lines 599-655), auto-save Effect (lines 443-510), suppress_autosave (line 352)
- `src/pages/draft.rs` — BUG-02/PLAN-02: game plan CTA (lines 2345-2381), BUG-05: draft slots + drag-drop
- `src/pages/game_plan.rs` — BUG-02/PLAN-02: `check_draft_has_game_plan` (lines 170-191), `get_draft_for_prefill` (lines 146-165)
- `src/pages/team/dashboard.rs` — BUG-03: roster display
- `src/server/db.rs` — BUG-03: `get_user_team_with_members` query (lines 680-683)
- `src/components/nav.rs` — BUG-04: user Resource read outside Suspense (line 118), Effect (lines 84-92)
- `src/components/draft_board.rs` — BUG-05: drag-and-drop handlers (lines 67-79, 144-156)

### Agent Browser
- Vercel agent-browser: install as Claude Code skill, replaces Playwright MCP in `.mcp.json`

### CLAUDE.md Sections to Update
- `CLAUDE.md` — "Browser verification with Playwright MCP" section → rewrite for agent-browser
- `CLAUDE.md` — "Plugins & MCP Servers" section → remove Playwright MCP, add agent-browser skill

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `e2e/tests/fixtures.ts`: Auth fixture pattern (`authenticatePage`, `createTeam`) — extend with new helpers
- Console error capture pattern in `audit-tree-drafter.spec.ts` (lines 37-50) — centralize into shared utility
- `pipeline.spec.ts`: Draft creation and game plan navigation flow — reference for regression tests

### Established Patterns
- Auto-save with debounce: `RwSignal<Option<i32>>` timer + `suppress_autosave` guard (CLAUDE.md rules 42, 54, 55)
- Hard navigation via `window.location().set_href()` for auth state changes (CLAUDE.md rule 8)
- `Callback::new()` for Copy closures shared across reactive contexts (CLAUDE.md rule 36)
- DB query → `Db*` struct → model conversion with `RecordId.to_sql()` (CLAUDE.md rule 3)
- Server fn URL discovery via WASM binary strings (CLAUDE.md rule 57)

### Integration Points
- `e2e/tests/` — all test files will import shared helpers
- `.mcp.json` — remove Playwright MCP entry
- `CLAUDE.md` — update browser verification and plugins sections
- `justfile` — `e2e` target already configured, may need updates for agent-browser

</code_context>

<specifics>
## Specific Ideas

- Bug fix order within Plan 2: easiest first (BUG-03, BUG-04, BUG-05) then harder (BUG-01, BUG-02/PLAN-02) — builds momentum and catches quick wins early
- BUG-01 freeze test: branch from node, then type in label + click save to prove editor is responsive (timeout = freeze)
- BUG-05 drag test: use real Playwright `dragAndDrop()` not click-based proxy — tests the actual broken code path
- BUG-04 hydration test: capture console messages and assert no "hydration mismatch" warnings

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-bug-fixes*
*Context gathered: 2026-03-19*
