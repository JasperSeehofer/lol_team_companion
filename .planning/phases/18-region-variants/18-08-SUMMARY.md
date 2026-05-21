---
phase: 18
plan: "08"
subsystem: mode-toggle-persistence
tags: [region, mode-toggle, persistence, schema, db, resolver, auth, leptos]
dependency-graph:
  requires: [18-01, 18-02, 18-04, 18-05, 18-06, 18-07]
  provides:
    - "ModeToggle UI primitive in src/components/region/controls.rs"
    - "3 DB getter/setter pairs for per-route mode preferences"
    - "AppUser + PublicUser carry draft_mode/team_dashboard_mode/solo_mode"
    - "resolve_mode helpers with region-coupled defaults (D-04) + explicit-override (D-05)"
    - "End-to-end mode toggle flow on /draft, /solo, /team/dashboard"
  affects:
    - "schema.surql — 3 new user fields"
    - "src/server/db.rs — 6 new public functions"
    - "src/server/auth.rs — DbUser + AppUser extended"
    - "src/models/user.rs — PublicUser extended"
    - "src/pages/profile.rs — get_current_user maps new fields"
    - "src/components/region/controls.rs — ModeToggle added"
    - "src/pages/draft.rs — stub replaced"
    - "src/pages/solo_dashboard.rs — stub replaced"
    - "src/pages/team/dashboard.rs — stub replaced"
tech-stack:
  added: []
  patterns:
    - "DEFINE FIELD IF NOT EXISTS ... DEFAULT 'auto' with no ASSERT (app-layer allowlist instead)"
    - "Option<String> -> String unwrap_or_else('auto') in From<DbUser> for AppUser"
    - "ReadSignal<String> + Callback<String> ModeToggle prop signature"
    - "StoredValue wrapping options Vec for FnOnce-safe iteration in reactive closures"
    - "auth_user.refetch() after set_mode_action success for session sync"
    - "resolve_mode() inline per-page fn: explicit stored value wins, 'auto' routes to region default"
key-files:
  created: []
  modified:
    - schema.surql
    - src/server/db.rs
    - src/server/auth.rs
    - src/models/user.rs
    - src/pages/profile.rs
    - src/components/region/controls.rs
    - src/pages/draft.rs
    - src/pages/solo_dashboard.rs
    - src/pages/team/dashboard.rs
decisions:
  - "resolve_mode() implemented inline per page (not a shared module) — simplest approach, avoids cross-page coupling for a 3-line function"
  - "StoredValue::new(options) in ModeToggle — options Vec is moved into move|| closure; StoredValue allows the inner iteration to call get_value() without ownership issues"
  - "Team dashboard uses resolve_team_dashboard_mode() (separate function name) to avoid name collision with draft.rs + solo_dashboard.rs which each define their own resolve_mode()"
  - "PublicUser in models/user.rs extended with 3 new fields — get_current_user() is the bridge from AppUser to client-side signal, so PublicUser must carry these"
  - "WASM target: no #[allow(unused_variables)] needed for mode_current because it IS used in both the Effect (set) and view! (read)"
metrics:
  duration: "~90min"
  completed: "2026-05-21"
  tasks-completed: 3
  files-modified: 9
---

# Phase 18 Plan 08: Mode Toggle UI + DB Persistence + Resolver Summary

End-to-end mode toggle flow — schema → DB → auth layer → resolver → ModeToggle UI primitive → server fn validation → signal-driven page dispatch. All three stub `let mode: String = "..."` placeholders from 18-04/05/06 replaced with a DB-backed, signal-driven flow.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Schema + DB 6-function layer + AppUser/PublicUser extension | 3addb84 |
| 2 | ModeToggle region-aware UI primitive | 25aa571 |
| 3a | Draft page — server fn + resolver + toggle wiring | 703577f |
| 3b | Solo dashboard — server fn + resolver + toggle wiring | 76792e7 |
| 3c | Team dashboard — server fn + resolver + toggle wiring | dcebfc0 |

## Task 1: Schema + DB + Auth

### Schema additions (`schema.surql`)

Three new fields appended after the `theme` ASSERT line:

```sql
DEFINE FIELD IF NOT EXISTS draft_mode ON user TYPE string DEFAULT 'auto';
DEFINE FIELD IF NOT EXISTS team_dashboard_mode ON user TYPE string DEFAULT 'auto';
DEFINE FIELD IF NOT EXISTS solo_mode ON user TYPE string DEFAULT 'auto';
```

No ASSERT constraint per Research Pitfall 4 — value set is wider than the two-value `theme` field (4 draft values, 3 solo values, 3 team values) and may grow. App-layer allowlist in each server fn is the sole validation gate.

### DB layer (`src/server/db.rs`)

6 functions added following the existing `get_user_mode`/`set_user_mode` pattern (lines 4464-4488):

- `get_user_draft_mode` / `set_user_draft_mode`
- `get_user_team_dashboard_mode` / `set_user_team_dashboard_mode`
- `get_user_solo_mode` / `set_user_solo_mode`

Getters use `Option<String>` + `.unwrap_or_else(|| "auto".to_string())` to handle NONE for pre-Phase records (Research Pitfall 2). Setters call `.check()?` per surreal-patterns rule 27.

### Auth layer (`src/server/auth.rs` + `src/models/user.rs` + `src/pages/profile.rs`)

- `DbUser`: 3 new `Option<String>` fields
- `AppUser`: 3 new `String` fields
- `From<DbUser>`: 3 `.unwrap_or_else(|| "auto".to_string())` conversions
- `PublicUser` (models/user.rs): 3 new `String` fields
- `get_current_user()` (profile.rs): maps all 3 fields from `AppUser` → `PublicUser`

## Task 2: ModeToggle UI Primitive

`src/components/region/controls.rs` — `ModeToggle` component added after `Btn` and `Badge`.

**Prop signature (final):**
```rust
pub fn ModeToggle(
    region: String,
    current: ReadSignal<String>,
    #[prop(into)] options: Vec<(String, String, String)>,
    on_select: Callback<String>,
) -> impl IntoView
```

**Demacia variant:** rounded-pill segmented control with gilt gradient border, `font-imperial` uppercase labels.

**Pandemonium variant:** flat tab-pull with `border-accent/30` bracket border, `font-mono` uppercase labels.

Both variants: `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none` on every button (G-12 compliant). `StoredValue::new(options)` used to avoid FnOnce move issues when iterating in a reactive `move || {}` block.

## Task 3: Page Wiring (all 3 stubs replaced)

### Common pattern applied to all 3 pages

Each page received:
1. A `#[server]` fn with allowlist validation (T-18-08-01 mitigation)
2. A `resolve_mode()` / `resolve_team_dashboard_mode()` pure fn
3. Mode signal setup: `(mode_current, set_mode_current) = signal(resolve_mode("auto", &region, ...))`
4. `Effect` syncing `mode_current` from `auth_user` resource when it loads
5. `Action` dispatching the server fn + `auth_user.refetch()` on success
6. `Callback` for optimistic UI update + dispatch
7. `<ModeToggle ...>` in the page view

### Per-route allowlists (T-18-08-01 mitigations)

| Route | Server fn | Allowlist |
|-------|-----------|-----------|
| `/draft` | `set_draft_mode_pref` | `["auto", "carousel", "war-table", "ledger"]` |
| `/solo` | `set_solo_mode_pref` | `["auto", "constellation", "forge", "journal"]` |
| `/team/dashboard` | `set_team_dashboard_mode_pref` | `["auto", "dashboard", "brief"]` |

### Mode dispatch changes

| Page | Before | After |
|------|--------|-------|
| draft.rs | `match mode.as_str()` (static `let mode = "carousel"`) | `move \|\| match mode_current.get().as_str()` |
| solo_dashboard.rs | `match mode.as_str()` (static `let mode = "constellation"`) | `match mode_current.get().as_str()` (inside existing reactive closure) |
| team/dashboard.rs | `if mode == "brief"` (static `let mode = "dashboard"`) | `if mode_current.get() == "brief"` |

## resolve_mode() Resolver — Final Table

Per D-04 (region-coupled defaults) + D-05 (explicit override wins):

| Route | Stored | Region | Resolved |
|-------|--------|--------|---------|
| draft | "auto" | demacia | "carousel" |
| draft | "auto" | pandemonium | "ledger" |
| draft | "war-table" | any | "war-table" (explicit wins) |
| draft | "ledger" | demacia | "ledger" (explicit wins) |
| solo | "auto" | demacia | "constellation" |
| solo | "auto" | pandemonium | "forge" |
| solo | "journal" | any | "journal" (explicit wins) |
| team-dashboard | "auto" | demacia | "dashboard" |
| team-dashboard | "auto" | pandemonium | "brief" |
| team-dashboard | "brief" | demacia | "brief" (explicit wins) |

Note: Pandemonium /solo default is **forge** (not journal). Journal is an explicit pick only.

## Explicit-Override Persistence (D-05)

D-05 states: an explicit user pick persists across region switches. This is implemented because:
- The stored value ("forge", "war-table", etc.) is written to the DB by the server fn
- `resolve_mode()` returns `stored.to_string()` immediately if `stored != "auto"`
- Region is only consulted when stored == "auto"

Therefore: if a user picks "ledger" on Demacia and then switches to Pandemonium, `stored = "ledger"` → `resolve_mode("ledger", "pandemonium", "draft")` → `"ledger"`. The explicit pick survives.

Manual verification: not run in this agent session (no running server). The compile + test suite (111 passing) + grep-based acceptance criteria provide confidence. End-to-end browser test deferred to 18-09 or a post-phase smoke run.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `v_for_class` moved into class closure, unavailable for `aria-pressed`**
- **Found during:** Task 2 (cargo check on controls.rs)
- **Issue:** `let v_for_class = value.clone()` was used in both `class=move || ...` and `aria-pressed=move || ...` closures. The first `move ||` consumed it, leaving `aria-pressed` with no binding.
- **Fix:** Added `let v_for_aria = value.clone()` as a separate clone for the aria-pressed closure. Same fix applied in both Pandemonium and Demacia arms.
- **Files modified:** src/components/region/controls.rs
- **Commit:** 25aa571

**2. [Deviation - Scope extension] PublicUser + get_current_user() required updating**
- **Found during:** Task 3 — the plan referenced `user.draft_mode` in page components, but the pages read from `auth_user` resource which returns `PublicUser`, not `AppUser`.
- **Fix:** Added 3 fields to `PublicUser` (models/user.rs) and updated `get_current_user()` (profile.rs) to map them. This is within scope of the plan's intent (AppUser carries the fields) and required for the end-to-end flow.
- **Files modified:** src/models/user.rs, src/pages/profile.rs
- **Commit:** 3addb84

**3. [Deviation - Naming] `resolve_team_dashboard_mode()` vs `resolve_mode()`**
- **Found during:** Task 3 — draft.rs and solo_dashboard.rs both define `fn resolve_mode()`. team/dashboard.rs also needed one. Since all 3 are in separate modules (separate files), each has its own private `resolve_mode()`. No name collision within a file, but for clarity the team dashboard version was named `resolve_team_dashboard_mode()` to match its specific context.
- **Commit:** dcebfc0

## Known Stubs

None introduced by this plan. The pre-existing stat placeholders (KDA "3.42", CS/min "7.1" etc.) in solo_dashboard.rs were documented in 18-05-SUMMARY.md and remain unchanged.

## Threat Surface Scan

No new network endpoints or auth paths beyond the 3 documented server fns. All 3 server fns:
- Extract `auth.user.ok_or_else(...)` (T-18-08-02 mitigation: no unauthenticated writes)
- Validate mode against allowlist before DB write (T-18-08-01 mitigation)
- Bind `user_id` from session, never from client param (T-18-08-04 mitigation)

## Self-Check: PASSED

| Item | Status |
|------|--------|
| schema.surql: draft_mode field | FOUND |
| schema.surql: team_dashboard_mode field | FOUND |
| schema.surql: solo_mode field | FOUND |
| No ASSERT on mode fields | CONFIRMED |
| src/server/db.rs: get_user_draft_mode | FOUND |
| src/server/db.rs: set_user_draft_mode | FOUND |
| src/server/db.rs: get_user_team_dashboard_mode | FOUND |
| src/server/db.rs: set_user_team_dashboard_mode | FOUND |
| src/server/db.rs: get_user_solo_mode | FOUND |
| src/server/db.rs: set_user_solo_mode | FOUND |
| src/server/auth.rs: draft_mode: String | FOUND |
| src/server/auth.rs: team_dashboard_mode: String | FOUND |
| src/server/auth.rs: solo_mode: String | FOUND |
| src/server/auth.rs: unwrap_or_else "auto" | FOUND |
| src/components/region/controls.rs: ModeToggle | FOUND |
| src/components/region/controls.rs: focus-visible:ring-2 | FOUND |
| src/pages/draft.rs: set_draft_mode_pref | FOUND |
| src/pages/solo_dashboard.rs: set_solo_mode_pref | FOUND |
| src/pages/team/dashboard.rs: set_team_dashboard_mode_pref | FOUND |
| All 3 stubs removed | CONFIRMED |
| All 3 ModeToggle instances present | CONFIRMED |
| cargo check --features ssr | PASS |
| cargo check --features hydrate --target wasm32-unknown-unknown | PASS |
| cargo test --features ssr --lib (111 tests) | PASS |
| commit 3addb84 | FOUND |
| commit 25aa571 | FOUND |
| commit 703577f | FOUND |
| commit 76792e7 | FOUND |
| commit dcebfc0 | FOUND |
