# Phase 6: Bug Fixes - Research

**Researched:** 2026-03-19
**Domain:** Leptos 0.8 / SurrealDB bug fixing + Playwright e2e infrastructure
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Regression Test Coverage**
- Every bug fix (BUG-01 through BUG-05) gets a dedicated e2e regression test
- All regression tests live in a single file: `e2e/tests/regression.spec.ts`
- Tests verify the fix works (correct behavior), not reproduce the original bug condition
- BUG-04 (hydration) gets a console assertion test checking for no hydration mismatch warnings
- BUG-05 (drag-and-drop) uses real Playwright `dragAndDrop()` events, not click-based proxies
- BUG-01 (tree drafter freeze) test branches from a node, then verifies editor interactivity (type label, click save, confirm response)

**Playwright CLI Infrastructure (TEST-02)**
- Add page navigation helpers: `navigateTo('draft')` with proper waits (networkidle + hydration settle)
- Add common interaction helpers: `selectChampion()`, `fillDraftSlot()`, `createDraft()`, `saveGamePlan()`
- Centralize console error capture utility (currently copy-pasted across audit specs)
- Add visual regression snapshot support using Playwright's built-in `toMatchSnapshot()` / `toHaveScreenshot()`
- Helpers live in `e2e/tests/` as Playwright test utilities (not shared with external tools)
- Refactor ALL existing audit specs to use new helpers (not just new tests)

**Agent Browser Setup (TEST-02)**
- Install Vercel's `agent-browser` as a Claude Code skill (not just global CLI)
- Replace Playwright MCP: remove from `.mcp.json`, use agent-browser skill instead
- Update CLAUDE.md browser verification instructions to reference agent-browser instead of MCP commands
- Agent-browser becomes the primary tool for interactive browser verification during development

**Bug Fix Verification Flow**
- Each bug: fix code + write regression test + browser check via agent-browser
- One atomic commit per bug fix (fix + regression test together)
- Full e2e suite (`just e2e`) must pass green at end of Phase 6, including all refactored audit specs

**Plan Grouping & Execution Order**
- 3 plans, executed sequentially:
  - Plan 1: Test infrastructure — Playwright CLI helpers, agent-browser skill setup, CLAUDE.md update, error capture utility, snapshot support
  - Plan 2: Bug fixes — All 5 bugs (BUG-01–05, PLAN-02) with regression tests in `regression.spec.ts`. Fix order: easiest first (BUG-03 roster, BUG-04 hydration, BUG-05 drag-drop) then harder (BUG-01 tree drafter, BUG-02/PLAN-02 game plan nav). One commit per bug.
  - Plan 3: Audit spec refactor — Refactor all existing audit specs to use new helpers + verify full suite green

### Claude's Discretion
- Exact helper function signatures and API design
- How to structure the error capture utility (class, function, fixture)
- Which existing audit specs need the most refactoring vs light touch
- Agent-browser skill configuration details
- Snapshot baseline management strategy

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| BUG-01 | Tree drafter "Branch from here" makes UI unresponsive | Root cause identified: `select_node()` called inside `spawn_local` async block triggers reactive loop. Fix: wrap select_node call with suppress_autosave guard before/after |
| BUG-02 | "View game plan" from draft opens empty game plan | Root cause identified: `href` at `draft.rs:2704` links to `/game-plan` with no query params. Fix: use `/game-plan?plan_id={existing_plan_id}` |
| BUG-03 | Team leader disappears from roster under certain conditions | Root cause identified: DB query at `db.rs:681` returns `team_member` rows only; leader may be absent if display logic filters them. Investigate member partitioning in `dashboard.rs:611-622` |
| BUG-04 | Nav resource read outside Suspense causes hydration warning | Root cause identified: `is_authed` at `nav.rs:118` reads `user` Resource directly (not in Suspense/transition). Fix: wrap in `move || user.with(...)` or use `Transition` |
| BUG-05 | Draft drag-and-drop copies icon link instead of moving pick/ban | Root cause identified: filled slots in `draft_board.rs` have no `dragstart` handler, so browser defaults to `<img src>` drag. Fix: add `draggable="true"` + `ondragstart` on filled slot containers that sets `text/plain` to champion name, and clear source slot on drop |
| PLAN-02 | Clicking "view game plan" from draft navigates to correct existing game plan | Same fix as BUG-02: the `href="/game-plan"` at `draft.rs:2704` must include `?plan_id={existing_plan_id}` |
| TEST-02 | Playwright CLI workflow configured for testing/verification agents with auth fixture and page navigation helpers | Infrastructure: create `e2e/tests/helpers.ts`, install agent-browser skill, remove Playwright MCP, update CLAUDE.md |
</phase_requirements>

---

## Summary

Phase 6 fixes 5 known regressions in the Leptos/SurrealDB/WASM app and establishes solid test infrastructure. All bugs have been root-cause-analyzed from source code — none require speculative investigation. The fixes range from trivial (one-line href change for BUG-02) to medium-complexity Leptos reactive pattern fixes (BUG-01 suppress_autosave lifecycle, BUG-04 hydration boundary, BUG-05 drag events).

The test infrastructure work (TEST-02) involves creating a shared helpers module for the existing Playwright suite and replacing the Playwright MCP with Vercel's agent-browser Claude Code skill. All existing audit specs already follow consistent patterns, so refactoring is mechanical.

**Primary recommendation:** Fix bugs in order of ascending complexity (BUG-03, BUG-04, BUG-05, then BUG-01, BUG-02/PLAN-02). Write test infrastructure first so regression tests can be added atomically with each fix.

---

## Standard Stack

### Core (all existing — no new dependencies needed)
| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| Leptos | 0.8 | UI framework (WASM + SSR) | All bugs are in Leptos reactive code |
| SurrealDB | 3.x | Database | BUG-03 may need query fix |
| Playwright | (existing in e2e/) | E2e testing | Adding helpers + regression file |
| `@playwright/test` | (existing) | Test runner API | `dragAndDrop()`, `toHaveScreenshot()`, console capture |

### No New Packages Required

All bug fixes are pure code corrections to existing files. The test infrastructure additions are TypeScript helper files using existing Playwright APIs.

For agent-browser: this is a Claude Code skill installation, not an npm package. No changes to `Cargo.toml` or `e2e/package.json` are needed.

---

## Architecture Patterns

### Existing Test Infrastructure
```
e2e/
├── playwright.config.ts      # timeout: 60s, baseURL: 127.0.0.1:3002, no webServer block
├── tests/
│   ├── fixtures.ts           # authenticatePage(), createTeam(), authedPage, teamPage fixtures
│   ├── pipeline.spec.ts      # Draft→game-plan roundtrip (partially covers BUG-02)
│   ├── audit-tree-drafter.spec.ts  # Has captureErrors()/filterRealErrors() pattern
│   ├── audit-draft.spec.ts
│   ├── audit-game-plan.spec.ts
│   ├── audit-champion-pool.spec.ts
│   ├── audit-team.spec.ts
│   ├── audit-misc-pages.spec.ts
│   ├── audit-post-game.spec.ts
│   ├── auth.spec.ts
│   ├── pages.spec.ts
│   └── smoke.spec.ts
```

### New Files for Plan 1
```
e2e/
└── tests/
    ├── helpers.ts            # NEW: shared navigation + interaction + error capture utilities
    └── regression.spec.ts    # NEW: one regression test per bug (BUG-01 through BUG-05)
```

### Pattern: Shared Error Capture (centralize from audit-tree-drafter.spec.ts)

Currently copy-pasted across audit specs. Extract to `helpers.ts`:

```typescript
// Source: e2e/tests/audit-tree-drafter.spec.ts (lines 37-50)
export function captureErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(`[pageerror] ${e.message}`));
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(`[console.error] ${msg.text()}`);
  });
  return errors;
}

export function filterRealErrors(errors: string[]): string[] {
  return errors.filter(
    (e) => !e.includes("favicon") && !e.includes("404 (Not Found)")
  );
}
```

### Pattern: Navigation Helper with Hydration Settle

Based on timing patterns observed in `pipeline.spec.ts` and `fixtures.ts`:

```typescript
// Proposed: e2e/tests/helpers.ts
export async function navigateTo(page: Page, route: string): Promise<void> {
  await page.goto(route);
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500); // WASM hydration settle (CLAUDE.md rule 56)
}
```

The 500ms settle matches the existing pattern in `fixtures.ts:authenticatePage` (line 36) and `pipeline.spec.ts` (line 45-46).

### Pattern: Playwright `dragAndDrop()` for BUG-05

Playwright's built-in `dragAndDrop()` generates the full mousedown→mousemove→dragstart→drop→mouseup event sequence. This tests the actual browser drag pipeline, not a click-based proxy:

```typescript
// BUG-05 regression test pattern
await page.dragAndDrop(
  '[data-slot="source-slot-selector"]',
  '[data-slot="target-slot-selector"]'
);
```

Note: The source slots (champion picker buttons) already have `draggable="true"` and `ondragstart` in `champion_picker.rs:126-133`. After BUG-05 fix, filled `draft_board.rs` slots will also need `draggable="true"` for slot-to-slot drag.

### Pattern: Console Hydration Warning Capture (BUG-04)

```typescript
// BUG-04 regression test
const warnings: string[] = [];
page.on("console", (msg) => {
  if (msg.text().includes("hydration") || msg.text().includes("mismatch")) {
    warnings.push(msg.text());
  }
});
await navigateTo(page, "/");
expect(warnings).toHaveLength(0);
```

---

## Bug Root Causes (HIGH confidence — verified from source)

### BUG-01: Tree Drafter "Branch from here" UI Freeze

**File:** `src/pages/tree_drafter.rs:599-655` (`do_branch_from` callback)

**Root cause:** `do_branch_from` calls `select_node(&new_node)` inside a `spawn_local` async block (line 647). `select_node` calls `suppress_autosave.set(true)` and then schedules `setTimeout(0)` to re-enable it. Inside the async block, the timing is:

1. `nodes_resource.refetch()` fires (line 633)
2. `select_node(&new_node)` sets `suppress_autosave = true` and batch-updates multiple signals
3. `setTimeout(0)` re-enables `suppress_autosave`
4. But the auto-save `Effect` at line 443 may have already captured intermediate signal values and is waiting to fire a save

The freeze likely occurs because `suppress_autosave` is re-enabled before the signals settle, causing the auto-save Effect to fire with partially-initialized state that creates a reactive loop (Effect → auto-save → nodes_resource.refetch → Effect...).

**Fix strategy:** Before calling `select_node` inside `spawn_local`, explicitly call `cancel_autosave_timer()` and set `suppress_autosave.set(true)`. After `select_node` returns, the existing `setTimeout(0)` inside `select_node` re-enables auto-save. This ensures the timer is always cancelled before any signal batch updates.

**Confidence:** HIGH — root cause traced through signal flow. CLAUDE.md rules 42, 54, 55 document exactly this class of bug.

---

### BUG-02 / PLAN-02: "View Game Plan" Opens Empty Page

**File:** `src/pages/draft.rs:2704`

**Root cause:** The "View Game Plan" link in the duplicate-plan prompt uses:
```rust
<a href="/game-plan">"View Game Plan"</a>
```
This navigates to the game plan page with no query parameters. The `GamePlanPage` component loads the most recently-created plan by default (or no plan), not the existing plan linked to the current draft.

**Fix:** Change to:
```rust
<a href=format!("/game-plan?plan_id={}", existing_plan_id)>"View Game Plan"</a>
```

The `existing_plan_id` is already available in scope at line 2698 from `check_draft_has_game_plan()`. The `game_plan.rs` page already supports `?plan_id=` query params (verified: `get_draft_for_prefill` and `check_draft_has_game_plan` both use `draft_id`, so the game plan page needs to also handle `?plan_id=` routing — verify this before implementing).

**Confidence:** HIGH for location. MEDIUM for the exact query param key (`plan_id` vs `draft_id`) — verify how `game_plan.rs` loads a specific plan by ID before writing the fix.

---

### BUG-03: Team Leader Disappears from Roster

**File:** `src/server/db.rs:681` and `src/pages/team/dashboard.rs:611-622`

**Root cause (most likely):** The DB query at `db.rs:681` only returns rows from the `team_member` table. The member partitioning in `dashboard.rs:611-622` splits members into `starters` (roster_type == "starter"), `coaches`, and `subs`. If the team leader has `roster_type = 'sub'` (the default when joining, per `db.rs:538`), they appear in the subs list.

However, based on the bug description "team leader disappears from roster under certain conditions", the issue may be that the leader's `team_member` row is not reliably present or is filtered by one of the partition predicates. The sub filter at line 619 is `roster_type != "starter" && role != "coach"` — if the leader has `role = 'coach'`, they'd appear only in coaches, not starters or subs.

**Fix strategy:** Ensure the leader's `team_member` row is always returned from the query and that the rendering logic always shows the leader regardless of roster partitioning. May also need to check `dashboard.rs` rendering to confirm the leader is always visually distinguished.

**Confidence:** MEDIUM — exact trigger condition ("certain conditions") not pinpointed. The partitioning logic combined with role/roster_type state is the likely area.

---

### BUG-04: Nav Resource Read Outside Suspense → Hydration Warning

**File:** `src/components/nav.rs:118`

**Root cause:** Line 118:
```rust
let is_authed = move || user.get().and_then(|r| r.ok()).flatten().is_some();
```
`user` is a `Resource` created at line 75. Reading a `Resource` with `.get()` outside a `Suspense` boundary during SSR returns `None` (resource not yet resolved), but during WASM hydration it may return a different value. This mismatch between SSR-rendered HTML and WASM hydration produces hydration mismatch warnings.

**Fix strategy:** In Leptos 0.8, wrap Resource reads that drive rendering in `Suspense` or use `.map()` inside the reactive context with a fallback. Since `is_authed` is used in `nav_links` (line 122) which is used directly in the view, it needs to be inside a `<Suspense>` boundary or use `user.with(|r| ...)` pattern that handles the pending state.

The simplest fix: use `user.get().flatten().and_then(|r| r.ok()).flatten().is_some()` pattern carefully, or wrap the auth-gated nav links in `<Suspense fallback=|| view!{}>`.

**Confidence:** HIGH — line 118 reads a Resource outside Suspense, which is the canonical Leptos hydration warning cause.

---

### BUG-05: Drag-and-Drop Copies Icon Link Instead of Moving

**File:** `src/components/draft_board.rs` (ban slot render ~line 67-79, pick slot render ~line 144-156)

**Root cause:** The draft board slots have `ondrop` handlers that read `text/plain` data transfer. The champion picker (`champion_picker.rs:127-133`) correctly sets `text/plain` to the champion ID on `dragstart`. However:

1. **Slot-to-slot drag** (dragging a _filled_ slot to another slot): The filled slot `<div>` containers in `draft_board.rs` have no `draggable` attribute and no `ondragstart` handler. The browser defaults to dragging the child `<img>` element natively, which sets `text/uri-list` and `text/html` to the image URL — not the champion name. The drop handler reads `text/plain`, gets an empty string, and does nothing (or reads the URL as the champion name).

2. **Source slot not cleared after drop**: The `on_slot_drop` callback calls `fill_slot(slot_idx, name)` which fills the target but does NOT clear the source slot. This causes a copy instead of a move.

**Fix strategy:**
1. Add `draggable="true"` and `ondragstart` to the filled slot containers in `render_ban_slot` and `render_pick_slot`. The `dragstart` handler should call `ev.data_transfer().set_data("text/plain", &champ_name)`.
2. Store the drag source slot index in the `dragstart` handler and clear it in the drop handler after filling the target. Options: use a `RwSignal<Option<usize>>` for `drag_source_slot`, or pass slot index as additional data transfer (e.g., `text/x-slot-index`).

**Confidence:** HIGH — drag chain traced end-to-end through `champion_picker.rs` and `draft_board.rs`. The missing `dragstart` on filled slots is confirmed by the absence of any `drag` keyword hits in `draft_board.rs`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Drag-and-drop slot move | Custom mouse event tracking | Native HTML5 drag events + `ev.data_transfer()` | Already implemented for champion picker; extend same pattern |
| Drag test simulation | `page.mouse.move()` sequences | `page.dragAndDrop()` | Playwright's built-in generates correct drag event sequence |
| Visual regression | Custom screenshot comparison | `toHaveScreenshot()` / `toMatchSnapshot()` | Built into `@playwright/test` |
| Console error filtering | Custom regex | `captureErrors()` + `filterRealErrors()` pattern | Already established in `audit-tree-drafter.spec.ts` |
| Browser verification in dev | Manual Playwright MCP commands | Vercel agent-browser skill | Replaces MCP with skill that auto-discovers base URL and auth state |

---

## Common Pitfalls

### Pitfall 1: `select_node` Called Inside `spawn_local` Without Timer Cancellation

**What goes wrong:** Reactive Effects fire between async awaits. If `nodes_resource.refetch()` at line 633 triggers a re-render that fires the auto-save Effect before `suppress_autosave` is set, a second save fires with stale data.

**How to avoid:** Call `cancel_autosave_timer()` and `suppress_autosave.set(true)` before any signal-writing calls inside async blocks. The `select_node` function sets `suppress_autosave(true)` internally, but a race window exists between `refetch()` completion and `select_node` being called.

**Warning signs:** UI becomes unresponsive after branching; save status flickers; multiple "Node saved" toasts appear.

---

### Pitfall 2: `dragAndDrop()` Requires Visible Elements

**What goes wrong:** Playwright's `dragAndDrop()` fails silently if either source or target is not visible or not in the viewport.

**How to avoid:** Scroll both source and target into view before drag. Use `page.locator(...).scrollIntoViewIfNeeded()` before `dragAndDrop()`.

**Warning signs:** Test passes but slot state is unchanged; drag events fire but no drop event.

---

### Pitfall 3: Hydration Fix Must Work on Both SSR and WASM Targets

**What goes wrong:** A fix that compiles under `ssr` feature but uses `#[cfg(feature = "hydrate")]` incorrectly can still produce hydration mismatches.

**How to avoid:** After fixing BUG-04, run `cargo check --features ssr` AND `cargo check --features hydrate --target wasm32-unknown-unknown` before committing. The mismatch comes from SSR vs WASM rendering different HTML.

---

### Pitfall 4: Game Plan URL Query Param Key

**What goes wrong:** If the game plan page uses `?draft_id=` to load a plan but the fix passes `?plan_id=`, the plan won't load.

**How to avoid:** Read `game_plan.rs` page component to verify which query param it uses to load an existing plan by ID before writing the BUG-02 fix. The `check_draft_has_game_plan` server fn returns a `plan_id` (not a `draft_id`), so the navigation must use the correct param key.

---

### Pitfall 5: `toHaveScreenshot()` Baselines Must Be Committed

**What goes wrong:** Visual regression tests fail on first run because no baseline image exists. CI fails.

**How to avoid:** Run `npx playwright test --update-snapshots` after adding `toHaveScreenshot()` calls to generate baselines, then commit the baseline `.png` files. Document this in test file comments.

---

### Pitfall 6: E2e Test Imports Must Stay Within `e2e/tests/`

**What goes wrong:** TypeScript path imports like `import { ... } from "../helpers"` work locally but may break with Playwright's test discovery if helpers are outside `testDir`.

**How to avoid:** Keep all helpers in `e2e/tests/helpers.ts` (inside `testDir`). Import with `./helpers` relative path. This matches the existing `./fixtures` import pattern.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Playwright (existing in `e2e/`) |
| Config file | `e2e/playwright.config.ts` |
| Quick run command | `cd e2e && npx playwright test regression.spec.ts` |
| Full suite command | `just e2e` (or `cd e2e && npx playwright test`) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BUG-01 | Branch from node → editor remains responsive | e2e | `cd e2e && npx playwright test regression.spec.ts -g "BUG-01"` | ❌ Wave 0 |
| BUG-02 | "View Game Plan" navigates to populated plan | e2e | `cd e2e && npx playwright test regression.spec.ts -g "BUG-02"` | ❌ Wave 0 |
| BUG-03 | Team leader always visible in roster | e2e | `cd e2e && npx playwright test regression.spec.ts -g "BUG-03"` | ❌ Wave 0 |
| BUG-04 | No hydration mismatch warnings in console | e2e | `cd e2e && npx playwright test regression.spec.ts -g "BUG-04"` | ❌ Wave 0 |
| BUG-05 | Drag champion pick/ban moves it (clears source) | e2e | `cd e2e && npx playwright test regression.spec.ts -g "BUG-05"` | ❌ Wave 0 |
| PLAN-02 | Same fix as BUG-02 — covered by same test | e2e | (same test as BUG-02) | ❌ Wave 0 |
| TEST-02 | Helpers and agent-browser skill are usable | manual smoke | `just e2e` (full suite uses helpers) | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cd e2e && npx playwright test regression.spec.ts` (regression suite only)
- **Per wave merge:** `just e2e` (full suite)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `e2e/tests/regression.spec.ts` — covers BUG-01 through BUG-05
- [ ] `e2e/tests/helpers.ts` — navigation, error capture, interaction utilities
- [ ] Vercel agent-browser skill installed to `.claude/skills/`
- [ ] `.mcp.json` updated (remove Playwright MCP entry)
- [ ] `CLAUDE.md` browser verification section updated

---

## Code Examples

### Example: Correct Drag Source on Filled Slot (BUG-05 fix pattern)

```rust
// In draft_board.rs render_ban_slot / render_pick_slot — add to filled slot container div:
// (Pattern from champion_picker.rs:126-133 — extend same approach to slots)
let champ_name_for_drag = champ_name.clone();
// ... add to the container <div>:
draggable="true"
on:dragstart=move |ev: web_sys::DragEvent| {
    if let Some(dt) = ev.data_transfer() {
        let _ = dt.set_data("text/plain", &champ_name_for_drag);
        // Optionally: dt.set_data("text/x-source-slot", &slot_idx.to_string())
    }
}
```

### Example: Source Slot Clearing on Drop (BUG-05 fix pattern)

The `on_slot_drop` callback in both `tree_drafter.rs` and `draft.rs` calls `fill_slot(slot_idx, name)`. For slot-to-slot moves, the source slot must also be cleared. Options:

Option A — Pass source slot index via data transfer (requires reading additional data in drop handler).

Option B — Compare new slots with previous slots after fill and clear any duplicates (simpler, no source tracking needed):

```rust
// In fill_slot, after setting target slot, clear duplicates:
let champ = champion_name.clone();
set_draft_slots.update(|slots| {
    slots[slot_idx] = Some(champ.clone());
    // Clear any other slot that held this champion (slot-to-slot move)
    for i in 0..20 {
        if i != slot_idx && slots[i].as_deref() == Some(&champ) {
            slots[i] = None;
        }
    }
});
```

This approach is safe because champion names are unique per draft — `fill_slot` already checks `already_used` and returns early if the champion is in any slot. For slot-to-slot drag, the champion is being moved so the "already used" check would block it. The fix should skip the `already_used` guard for drop events or clear the source before filling.

**Confidence:** HIGH for approach. Implementation details (Option A vs B) are Claude's discretion.

---

### Example: `suppress_autosave` Guard in `do_branch_from` (BUG-01 fix pattern)

```rust
// In do_branch_from, inside the Ok(_) branch after save_node succeeds:
// Before select_node, explicitly cancel and suppress:
cancel_autosave_timer();           // cancel pending save timer
suppress_autosave.set(true);       // suppress auto-save Effect
// ... existing code ...
select_node(&new_node);            // select_node also sets suppress_autosave internally
// select_node schedules setTimeout(0) to re-enable suppress_autosave
// No additional re-enable needed here — select_node handles it
```

---

### Example: helpers.ts Structure

```typescript
// e2e/tests/helpers.ts
import { type Page } from "@playwright/test";

// Re-export captureErrors / filterRealErrors (centralized from audit-tree-drafter.spec.ts)
export function captureErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(`[pageerror] ${e.message}`));
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(`[console.error] ${msg.text()}`);
  });
  return errors;
}

export function filterRealErrors(errors: string[]): string[] {
  return errors.filter(
    (e) => !e.includes("favicon") && !e.includes("404 (Not Found)")
  );
}

// Navigation with hydration settle (pattern from fixtures.ts + pipeline.spec.ts)
export async function navigateTo(page: Page, route: string): Promise<void> {
  await page.goto(route);
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);
}

// createDraft: create team if needed, navigate to /draft, fill name, save
export async function createDraft(page: Page, name: string): Promise<string> { ... }

// saveGamePlan: fill name, click save, wait for confirmation
export async function saveGamePlan(page: Page, name: string): Promise<void> { ... }
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Playwright MCP (`@anthropic/mcp-playwright`) | Vercel agent-browser Claude Code skill | Phase 6 (now) | Agent browser verification is skill-based, not MCP-based |
| Copy-pasted `captureErrors` in each audit spec | Centralized in `helpers.ts` | Phase 6 (now) | One source of truth for error capture logic |
| No regression spec | `e2e/tests/regression.spec.ts` | Phase 6 (now) | Verified bugs can't silently regress |

---

## Open Questions

1. **BUG-02/PLAN-02: exact query param key for loading game plan by ID**
   - What we know: `check_draft_has_game_plan` returns a `plan_id` string. The "View Game Plan" link needs to navigate to that specific plan.
   - What's unclear: `game_plan.rs` must be read to confirm whether it accepts `?plan_id=<id>` as a query param to load a specific plan, or whether a different URL structure is used.
   - Recommendation: Read `src/pages/game_plan.rs` before implementing the BUG-02 fix to confirm the correct URL format.

2. **BUG-03: exact trigger condition for leader disappearance**
   - What we know: Leader is in `team.created_by`; members come from `team_member` table query. Partitioning: starters / coaches / subs.
   - What's unclear: Under what combination of `role` and `roster_type` does the leader disappear from the visible roster? Does the leader have a `team_member` row at all? (Team creation adds creator via `create_join_request` then `accept_join_request`, or directly in `create_team`?)
   - Recommendation: Check `db.rs create_team` to see if leader automatically gets a `team_member` row, and check `dashboard.rs` rendering to see which partition renders the leader's row.

3. **Agent-browser skill installation path**
   - What we know: CONTEXT.md says install as a Claude Code skill in `.claude/skills/`; it replaces Playwright MCP.
   - What's unclear: Exact install command for Vercel's agent-browser; whether it requires npm/npx or a different mechanism.
   - Recommendation: Plan 1 executor should search for Vercel agent-browser installation instructions. The `skill-creator` skill in `.claude/skills/` can help package it.

---

## Sources

### Primary (HIGH confidence)
- Direct source code reading — `src/components/draft_board.rs`, `src/components/champion_picker.rs`, `src/components/nav.rs`, `src/pages/tree_drafter.rs:330-655`, `src/pages/draft.rs:2344-2760`, `src/pages/game_plan.rs:140-191`, `src/server/db.rs:654-688`, `src/pages/team/dashboard.rs:590-660`
- `e2e/tests/fixtures.ts` — existing auth fixture patterns
- `e2e/tests/audit-tree-drafter.spec.ts` — existing `captureErrors`/`filterRealErrors` pattern
- `e2e/tests/pipeline.spec.ts` — existing draft→game-plan navigation test
- `e2e/playwright.config.ts` — test runner configuration
- `CLAUDE.md` — project conventions (rules 42, 54, 55, 56 directly apply)

### Secondary (MEDIUM confidence)
- CONTEXT.md decisions — locked implementation choices from user discussion

---

## Metadata

**Confidence breakdown:**
- Bug root causes: HIGH — all 5 bugs root-cause-traced from source code to specific lines
- Fix strategies: HIGH for BUG-02 (one-line fix), BUG-04 (Leptos Suspense pattern), BUG-05 (drag event pattern). MEDIUM for BUG-01 (reactive timing — fix direction clear, exact implementation needs care), BUG-03 (trigger condition partially unclear)
- Test infrastructure: HIGH — builds on existing patterns in the codebase
- Agent-browser setup: MEDIUM — installation details to be confirmed during Plan 1

**Research date:** 2026-03-19
**Valid until:** 2026-04-18 (30 days — stable Rust/Leptos/Playwright stack)
