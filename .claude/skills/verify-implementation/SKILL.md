---
name: verify-implementation
description: Post-implementation verification and interactive testing of the web app. Run after completing code changes to validate compilation, tests, browser rendering, interactions, console errors, and regressions. Trigger when: user says "verify", "test this", "check the app", "does it work", "verify implementation", or after completing a significant implementation phase. Also trigger on "/verify-implementation".
---

# Post-Implementation Verification

Comprehensive verification skill for the LoL Team Companion app. Runs after code changes to catch compile errors, test failures, rendering bugs, interaction regressions, and console noise that the Rust compiler cannot detect.

## Why This Skill Exists

The Rust compiler catches type errors and borrow issues, but it **cannot** catch:
- Rendering bugs (wrong CSS, missing elements, broken layout)
- Reactive signal bugs (stale data, UI freezes, missing refetch)
- Server function failures that only surface at runtime
- Console errors from broken API calls or missing resources
- Regressions on pages unrelated to the change
- Auth flow breakage (session loss, redirect failures)

This skill systematically covers all those gaps.

---

## Verification Phases

Run phases in order. Each phase gates the next — if Phase 1 fails, fix before proceeding.

### Phase 1: Compile & Test (automated, no browser)

Run these checks in parallel:

```bash
# 1a. SSR compile check
cargo check --features ssr

# 1b. WASM compile check
cargo check --features hydrate --target wasm32-unknown-unknown

# 1c. Run all tests (19 unit + 25 integration)
cargo test --features ssr

# 1d. Clippy lint check
cargo clippy --features ssr -- -D warnings

# 1e. Format check
cargo fmt --check
```

**Pass criteria:** All 5 must exit 0. If any fail, fix the issue before continuing.

**Report format:**
```
## Phase 1: Compile & Test
- [x] SSR compile — passed
- [x] WASM compile — passed
- [x] Tests — 44/44 passed
- [x] Clippy — no warnings
- [x] Fmt — formatted
```

---

### Phase 2: Server Health (requires running server)

Start the dev server if not already running:

```bash
# Start in background
cargo leptos watch
# Wait for ready
./scripts/wait_for_server.sh 120
```

Then verify health:

```bash
curl -s http://127.0.0.1:3002/healthz | python3 -c "import sys,json; d=json.load(sys.stdin); assert d['status']=='ok' and d['db']=='ok', f'Health check failed: {d}'"
```

**Pass criteria:** Health endpoint returns `{"status":"ok","db":"ok"}`.

---

### Phase 3: Auth Flow (Playwright MCP — interactive)

This phase establishes an authenticated session for all subsequent browser checks. Every verification run must start fresh to avoid stale session state.

**Steps:**

1. **Navigate to registration:**
   - `browser_navigate` to `http://127.0.0.1:3002/auth/register`
   - `browser_snapshot` — confirm registration form is visible (email, username, password fields + submit button)

2. **Register a test account:**
   - Generate unique credentials: `verify_<timestamp>@test.invalid` / `verifyuser_<timestamp>` / `Test1234!`
   - `browser_fill_form` — fill email, username, password fields
   - `browser_click` — click submit button
   - `browser_wait_for` — wait for text "Dashboard" or URL change to `/team/dashboard`
   - `browser_snapshot` — confirm redirected to team dashboard, username visible in nav

3. **Verify auth state persists:**
   - `browser_navigate` to `/profile`
   - `browser_snapshot` — confirm profile page renders (NOT redirected to login)
   - Confirm username appears on page

4. **Check console for auth errors:**
   - `browser_console_messages` with level "error"
   - Filter out known harmless errors (Tailwind 404, favicon 404)
   - Any remaining errors = FAIL

**Pass criteria:** Registration succeeds, auto-login works, profile page accessible, no auth console errors.

---

### Phase 4: Changed Pages — Deep Verification (Playwright MCP)

This is the core verification phase. For **each page that was modified** in the implementation:

#### 4a. Render Check

1. `browser_navigate` to the page
2. `browser_snapshot` — read full accessibility tree
3. Verify:
   - [ ] Page-specific heading or title is present
   - [ ] No `ErrorBanner` or unexpected error messages visible
   - [ ] Expected sections/components are present (forms, lists, cards)
   - [ ] Nav bar is visible with correct links for auth state
   - [ ] No raw HTML or unstyled content (indicates CSS failure)

#### 4b. Console & Network Check

1. `browser_console_messages` level "error" — capture all JS errors
2. `browser_network_requests` — check for:
   - [ ] No failed API calls (4xx/5xx on server function endpoints)
   - [ ] No unexpected 404s (beyond known Tailwind 404)
   - [ ] All Data Dragon CDN requests succeed (champion images)
3. Filter known harmless noise:
   - `/pkg/tailwindcss` 404 (Tailwind v4 build-time import)
   - `/favicon.ico` 404

#### 4c. Interaction Testing

For each interactive element on the changed page, test the user flow:

**Forms:**
1. Fill all fields with valid data
2. Submit
3. `browser_snapshot` — confirm success message or expected state change
4. Try submitting with empty/invalid data — confirm error handling (no crash, useful message)

**Buttons / Click Handlers:**
1. `browser_click` each actionable button
2. `browser_snapshot` after click — confirm expected state change
3. If button triggers async operation (spawn_local), `browser_wait_for` the result

**Modals / Toggles:**
1. Click trigger element to open
2. `browser_snapshot` — confirm modal content visible
3. Click cancel/close/backdrop — confirm modal dismisses
4. `browser_snapshot` — confirm modal gone, page restored

**Lists / Dynamic Content:**
1. If the page has a list that can be mutated (add/delete), perform the mutation
2. `browser_snapshot` — confirm list updated (new item appears / old item removed)
3. Refresh the page (`browser_navigate` to same URL) — confirm persistence

**Selectors / Dropdowns:**
1. `browser_select_option` or `browser_click` to change selection
2. `browser_snapshot` — confirm dependent UI updated

#### 4d. Empty State Check

If the page shows different content based on data presence:
1. Test with no data (e.g., new user with no team, no drafts, no stats)
2. Confirm empty state message shown (not an error banner)
3. Confirm CTA links work (e.g., "Create a team" links to `/team/roster`)

---

### Phase 5: Regression Sweep (Playwright MCP)

Visit ALL protected pages (not just changed ones) to catch regressions. This is a lighter check than Phase 4 — just render + console.

**Pages to sweep:**

| Route | Key Content Check |
|-------|------------------|
| `/` | "LoL Team Companion" or landing CTA |
| `/auth/login` | Login form visible |
| `/auth/register` | Register form visible |
| `/profile` | Username, "Riot Account" section |
| `/team/dashboard` | Team name or "Create a Team" |
| `/team/roster` | "Create" or "Join" or roster list |
| `/draft` | "Draft" heading, champion picker or empty state |
| `/tree-drafter` | "Tree" heading, tree list or empty state |
| `/stats` | "Stats" heading, sync button or empty state |
| `/champion-pool` | "Champion Pool" heading, tier sections |
| `/game-plan` | "Game Plan" heading, form or empty state |
| `/post-game` | "Post-Game" heading, review list or empty state |

**Per page:**
1. `browser_navigate` to route
2. `browser_snapshot` — confirm key content present
3. `browser_console_messages` level "error" — no new errors
4. Confirm NOT redirected to `/auth/login` (auth still valid)

**Efficiency:** Run this as a fast sequential sweep — navigate + snapshot only, no interaction. If a page fails the render check, flag it for deeper investigation but continue the sweep.

---

### Phase 6: E2E Test Suite (automated)

Run the full Playwright e2e test suite as a final regression gate:

```bash
cd e2e && npx playwright test 2>&1
```

**Pass criteria:** All tests pass (currently 21/21).

If tests fail:
1. Read the test output to identify which test failed
2. Check if the failure is caused by the implementation changes
3. If yes — fix the code and re-run from Phase 1
4. If no (flaky/infra) — note it and continue

---

### Phase 7: Smoke Tests (automated)

```bash
# API smoke tests (curl-based, covers server functions)
./scripts/api_test.sh
```

**Pass criteria:** All API endpoints return expected status codes.

---

## Reporting

After all phases complete, output a summary table:

```
## Verification Report

| Phase | Status | Details |
|-------|--------|---------|
| 1. Compile & Test | PASS | SSR, WASM, 44 tests, clippy, fmt |
| 2. Server Health | PASS | healthz ok |
| 3. Auth Flow | PASS | Register + auto-login + profile access |
| 4. Changed Pages | PASS | [list pages checked with notes] |
| 5. Regression Sweep | PASS | 12/12 pages render correctly |
| 6. E2E Suite | PASS | 21/21 tests |
| 7. Smoke Tests | PASS | All API endpoints ok |

### Issues Found
- [list any issues, or "None"]

### Console Errors (filtered)
- [list non-harmless errors, or "None beyond known Tailwind 404"]
```

---

## Scope Control

**Full verification** (default): All 7 phases. Use after major implementations, merges, or before releases.

**Quick verification** (`/verify-implementation quick`): Phases 1 + 2 + 4 only. Use after small, targeted changes where you're confident in the blast radius.

**Browser-only** (`/verify-implementation browser`): Phases 3 + 4 + 5 only. Use when compilation already passed and you just need to check the UI.

---

## What This Skill Does NOT Cover

- **Performance testing** — no load testing or timing benchmarks
- **Cross-browser testing** — Chromium only (matches e2e config)
- **Mobile responsive testing** — not tested via MCP (would need `browser_resize`)
- **Visual regression** — no screenshot diffing (accessibility tree is structural, not visual)
- **Security testing** — no XSS/CSRF/injection testing
- **Production deployment** — this is dev-mode verification only

---

## Interaction Testing Patterns by Component Type

Reference guide for Phase 4c — what to test for each component type found in this app:

### Draft Board (draft.rs, tree_drafter.rs)
- Click an empty slot → champion picker opens
- Select a champion → slot fills with icon + name
- Click filled slot → highlight appears (red border + x badge)
- Click x badge → champion removed, slot empty
- Verify pick/ban slot assignments match blue/red side toggle

### Champion Picker (champion_picker.rs)
- Role filter tabs filter the grid
- Search input filters by name
- Clicking a champion fires the selection callback
- All champion images load (Data Dragon CDN)

### Champion Autocomplete (champion_autocomplete.rs)
- Typing shows dropdown with matching champions
- Clicking a dropdown item selects it
- Dropdown closes after selection

### Tree Graph (tree_graph.rs)
- Nodes render with labels
- Selected node has visual highlight (accent color, glow)
- Clicking a node selects it in the editor (two-way sync)
- Edge icons show champion picks/bans diff
- "+" button on hover adds a branch

### Modal Dialogs
- Backdrop click closes modal
- Cancel button closes modal
- Form submission in modal triggers action + closes
- Modal content is accessible (form fields, labels)

### Auto-Save Indicators
- After editing, "Unsaved changes" indicator appears
- After 2s debounce, save fires and "Saved" indicator appears
- Switching away cancels pending save (no stale data)

### Nav Bar (nav.rs)
- Auth-aware: correct links shown for logged-in vs logged-out
- Notification badge shows pending count (if leader)
- Dropdown menus open/close correctly
- Theme toggle works (dark/light)
- Mobile hamburger menu works (if testing responsive)

---

## Tips for Efficient Verification

1. **Parallel Phase 1 commands** — run all 5 compile/test checks in parallel via separate Bash calls
2. **Reuse the MCP session** — once authenticated in Phase 3, all subsequent navigations share the session
3. **Snapshot before and after** — when testing interactions, snapshot before the action and after to diff the accessibility trees
4. **Filter console noise early** — check `browser_console_messages` once per page, filter known 404s, only flag genuinely new errors
5. **Skip unchanged pages in Phase 4** — only deep-test pages that were actually modified; the regression sweep (Phase 5) covers the rest at a lighter level
6. **Trust the compiler for type safety** — if Phase 1 passes, focus browser testing on rendering and interactions, not logic correctness
