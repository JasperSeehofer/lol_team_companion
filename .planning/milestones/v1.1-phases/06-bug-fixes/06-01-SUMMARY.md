---
phase: 06-bug-fixes
plan: 01
subsystem: e2e-testing
tags: [playwright, testing, infrastructure, agent-browser, helpers]
dependency_graph:
  requires: []
  provides: [e2e/tests/helpers.ts, e2e/tests/regression.spec.ts, .claude/skills/agent-browser/SKILL.md]
  affects: [CLAUDE.md, .mcp.json]
tech_stack:
  added: [agent-browser]
  patterns: [shared-test-helpers, visual-regression-snapshots, playwright-skip-scaffold]
key_files:
  created:
    - e2e/tests/helpers.ts
    - e2e/tests/regression.spec.ts
    - .claude/skills/agent-browser/SKILL.md
  modified:
    - .mcp.json
    - CLAUDE.md
decisions:
  - "Used agent-browser npm package (not @anthropic/agent-browser) — correct package is 'agent-browser' by Vercel Labs"
  - "Playwright MCP removed from .mcp.json; .mcp.json is gitignored so used git add -f"
  - "test.skip(true, msg) pattern used for BUG placeholders to satisfy Playwright's test list requirement"
metrics:
  duration: "~8 minutes"
  completed: "2026-03-22"
  tasks_completed: 2
  files_created: 3
  files_modified: 2
---

# Phase 06 Plan 01: Shared E2E Infrastructure Summary

Shared Playwright test helpers module (9 exports), regression test scaffold for BUG-01 through BUG-05, agent-browser skill installation, and CLAUDE.md updated to replace Playwright MCP with agent-browser + e2e workflow.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create shared helpers module and regression test scaffold | ac4c6a8 | e2e/tests/helpers.ts, e2e/tests/regression.spec.ts |
| 2 | Install agent-browser skill, remove Playwright MCP, update CLAUDE.md | d0175ad | .claude/skills/agent-browser/SKILL.md, .mcp.json, CLAUDE.md |

## What Was Built

### e2e/tests/helpers.ts (9 exports)

- `captureErrors(page)` — registers pageerror + console.error listeners, extracted from audit-tree-drafter.spec.ts
- `filterRealErrors(errors)` — filters favicon + 404 noise
- `navigateTo(page, route)` — goto + networkidle + 500ms WASM hydration settle (CLAUDE.md rule 56)
- `captureHydrationWarnings(page)` — console listener for "hydration|mismatch" messages (BUG-04)
- `selectChampion(page, name)` — fills search input + clicks matching champion button in grid
- `fillDraftSlot(page, slot, champion)` — delegates to selectChampion (board auto-advances slots)
- `createDraft(page, name)` — navigateTo /draft + fill name + Save Draft
- `saveGamePlan(page, name)` — fill plan name + Save Plan
- `takeSnapshot(page, name)` — page.screenshot() + toMatchSnapshot() for visual regression

### e2e/tests/regression.spec.ts

- 5 `test.describe` blocks for BUG-01 through BUG-05 (all skipped with `test.skip(true, msg)`)
- 1 `test.describe("Visual Regression")` with active home-page baseline snapshot
- Snapshot update workflow documented in file header comment
- `npx playwright test regression.spec.ts --list` shows 6 tests, exits 0

### .claude/skills/agent-browser/SKILL.md

- Documents `agent-browser` (Vercel Labs, Apache-2.0, npm: `agent-browser`)
- Install: `npm install -g agent-browser` or `npx agent-browser`
- Common commands: screenshot, navigate, click, type, snapshot
- Auth pattern for dev server (127.0.0.1:3002)
- When to use agent-browser vs e2e tests comparison table

### .mcp.json

- Playwright MCP entry removed
- context7 and github entries preserved
- Exactly 2 entries in mcpServers

### CLAUDE.md

- `### Browser verification with Playwright MCP` → `### Browser verification`
- First paragraph updated to reference e2e tests + agent-browser
- Recommended steps updated: `browser_navigate`/`browser_snapshot` → targeted playwright test + agent-browser screenshot
- "MCP vs e2e" subsection → "agent-browser vs e2e" subsection
- Auth section: removed MCP browser session steps, references `authenticatePage` from fixtures.ts
- "After making code changes" section: removed MCP reference
- Gotcha 45: "Playwright MCP" → "agent-browser"
- "Adding a New Feature" step 6: "Verify via Playwright MCP" → "Verify via agent-browser or e2e test"
- Plugins section: removed Playwright MCP bullet, added agent-browser bullet

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] .mcp.json is gitignored**
- **Found during:** Task 2 commit
- **Issue:** `.mcp.json` is listed in `.gitignore` so `git add` refused without `-f`
- **Fix:** Used `git add -f .mcp.json` to force-track the file
- **Files modified:** .mcp.json
- **Commit:** d0175ad

None other — plan executed as written.

## Known Stubs

None — all artifacts are complete. The BUG placeholder tests use `test.skip(true, msg)` which is intentional scaffolding (not a stub), to be filled in Plan 2.

## Self-Check: PASSED

All files exist. All commits found.
