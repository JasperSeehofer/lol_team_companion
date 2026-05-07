---
phase: 17-ui-consolidation
plan: 03d
subsystem: ui
tags: [open-design, semantic-tokens, utility-tier, opponents, action-items, ui-components]

# Dependency graph
requires:
  - phase: 17-ui-consolidation
    provides: "Plan 01 — input.css demacia/pandemonium tokens (--danger / --success / --warning aliased through @theme as bg-danger etc.); Plan 02 — Open-Design lol-companion seed (DESIGN.md component patterns: Card.plain, Card.elevated, Button.primary/ghost/destructive, Input.text, ErrorBanner) + 17-OD-MAP.md surface table"
provides:
  - "/opponents page restyled to utility tier (Open-Design HTML port from .od/projects/7e1c0a92-1b3d-47fe-b5c8-2f3a4b5c6d7e/opponents.html)"
  - "/action-items page restyled to utility tier (Open-Design HTML port from .od/projects/7e1c0a92-1b3d-47fe-b5c8-2f3a4b5c6d7e/action-items.html)"
  - "src/components/ui.rs — ErrorBanner, StatusMessage, ToastOverlay, EmptyState restyled with semantic bg-danger / bg-success tokens (legacy bg-red-500 / bg-emerald-500 retired)"
  - "src/components/stat_card.rs — Card.elevated pattern (bg-surface border border-outline/50 rounded-xl p-4) with font-display value + font-imperial label"
  - "17-OD-MAP.md — opponents + action-items rows updated to status=ported with OD UUID 7e1c0a92-1b3d-47fe-b5c8-2f3a4b5c6d7e"
affects: [17-04, 17-05, 17-06, all-pages-using-ErrorBanner-StatusMessage-StatCard]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Utility-tier visual recipe: canvas-grain bg-base min-h-screen wrapper + font-display italic h1 + font-imperial uppercase tracking-wider eyebrow + Card.plain (bg-elevated border border-divider rounded-xl p-6) for primary panels + Card.elevated (bg-surface border border-outline/50 rounded-xl p-4) for nested rows"
    - "Semantic-token migration: raw bg-red-500 / bg-emerald-500 / text-red-400 / text-green-400 → bg-danger / bg-success / text-danger / text-success via @theme aliasing in input.css (--color-danger: var(--danger))"
    - "Status-dot semantic token recipe: w-2.5 h-2.5 rounded-full bg-success | bg-warning | bg-muted (theme-aware vs raw bg-green-500 / bg-yellow-500 / bg-gray-500)"
    - "Button.destructive pattern: bg-danger/10 text-danger border border-danger/30 hover:bg-danger/20 + focus-visible:ring-2 focus-visible:ring-danger/50 — replaces bg-red-700 text-white CTA"
    - "G-12 focus-ring application: every interactive element (button, a, input, select) gets focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none (or ring-danger/50 on destructive variants)"

key-files:
  created:
    - ".planning/phases/17-ui-consolidation/17-03d-SUMMARY.md (this file)"
  modified:
    - "src/pages/opponents.rs (canvas-grain wrap; Card.plain list panel + creation form + detail panel; Card.elevated player slots; semantic danger/warning/success dots; G-12 focus rings throughout)"
    - "src/pages/action_items.rs (canvas-grain wrap; Card.plain Add Action Item form; Card.plain item rows; semantic dots; status pills; G-12 focus rings)"
    - "src/components/ui.rs (ErrorBanner: bg-danger + role=alert; StatusMessage: bg-danger/bg-success; ToastOverlay: bg-danger/20 + bg-success/20 + role=status + aria-live; EmptyState: focus-visible on CTA, aria-hidden on icon)"
    - "src/components/stat_card.rs (Card.elevated pattern + font-display value + font-imperial label + tabular-nums)"
    - ".planning/phases/17-ui-consolidation/17-OD-MAP.md (opponents + action-items rows: status=pending → status=ported, OD UUID + path filled in)"

key-decisions:
  - "OD UUID reuse: Phase 17 utility-tier OD project 7e1c0a92-1b3d-47fe-b5c8-2f3a4b5c6d7e was already seeded by an earlier session with both opponents.html and action-items.html artifacts. Reused that UUID across both surfaces rather than creating a new one — consistent with OD-MAP §How-to-generate step 2 (\"A single OD project can host many .html artifacts (one per surface)\")."
  - "Card.plain for the right-side detail panel of /opponents (was bg-surface) and Card.elevated for nested player-slot rows. Demarcates the visual hierarchy: outer panel = utility-tier surface, inner row = sub-panel."
  - "Replaced raw red-700 destructive confirm button (bg-red-700 hover:bg-red-600 text-white) with the Button.destructive pattern from DESIGN.md §5.2 (bg-danger/10 text-danger border border-danger/30) — softer, more editorial, theme-aware. The CLAUDE.md exception for 'colored buttons with white text' explicitly allows the old style; we chose the newer pattern because it's the documented design-system primitive for irreversible actions."
  - "OTP badge (warning-OTP marker) migrated from bg-orange-500/20 text-orange-400 border-orange-500/30 to bg-warning/15 text-warning border-warning/30. Maps closely; pandemonium's --warning is bright yellow so the marker reads correctly on both themes."
  - "Stale recency badge: was text-orange-400 → text-warning. \"Never fetched\" stays text-dimmed (low-priority metadata)."
  - "stat_card.rs: applied tabular-nums (Tailwind utility for font-variant-numeric: tabular-nums). DESIGN.md §3.3 mandates tabular numerics for stats; the previous component lacked this rule."

patterns-established:
  - "Worktree absolute-path requirement: when an executor runs in a parallel worktree (.claude/worktrees/agent-*) and the user's main checkout (/home/jasper/Repositories/lol_team_companion) is open simultaneously, Edit/Write tools may resolve relative paths to the OUTER repo. Always use absolute paths to the worktree root for source mutations. Detection: cargo check finishing in <0.2s after edits indicates no rebuild needed = no source change actually made."
  - "Token-migration hygiene: Tailwind v4 @theme aliases (--color-danger: var(--danger);) make raw → semantic class swaps a 1:1 string substitution. The migration order: bg-X → bg-{semantic}, text-X → text-{semantic}, border-X → border-{semantic}, hover:bg-X → hover:bg-{semantic}/{intensity}."

requirements-completed: [SC-2-claude-design-implementation, SC-3-open-design-seeding, SC-4-ui-review-pass]

# Metrics
duration: 13min
completed: 2026-05-07
---

# Phase 17 Plan 03d: Strategy Hub Utility Pages + Shared Components Restyle Summary

**Restyled the utility-tier `/opponents` and `/action-items` pages by porting the Open-Design HTML artifacts (`.od/projects/7e1c0a92-…/{opponents,action-items}.html`) to Leptos, plus migrated `src/components/ui.rs` (ErrorBanner, StatusMessage, ToastOverlay, EmptyState) and `src/components/stat_card.rs` to fully semantic tokens — no raw red/green/yellow/orange/gray classes remain in any modified file, every interactive element carries the G-12 `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none` rule, and all server fns + reactive logic are preserved verbatim.**

## Performance

- **Duration:** ~13 min (1 abandoned outer-repo edit cycle + 1 successful worktree edit cycle)
- **Started:** 2026-05-07T14:37:53Z
- **Completed:** 2026-05-07T14:50:58Z
- **Tasks:** 4 of 4 (Task 4 = checkpoint:human-verify, auto-approved per auto-mode)
- **Files modified:** 5 source + 1 doc = 6 files
- **Commits:** 3 atomic per-task commits (Task 4 is a checkpoint, no commit) + this metadata commit

## Accomplishments

- **/opponents** restyled — page now wraps in `canvas-grain bg-base min-h-screen`, header h1 uses `font-display italic text-3xl`, list panel + creation form + detail panel use Card.plain, player slots use Card.elevated, all status colors semantic. Selected-row indicator switched from `bg-elevated` to `bg-surface border-l-4 border-l-accent` (matches OD HTML row.selected).
- **/action-items** restyled — page wraps in canvas-grain, h1 uses font-display italic, "Add Action Item" Card.plain with font-imperial eyebrow, status-dots semantic (bg-success/warning/muted), action pills use bg-accent/20 text-accent + bg-success/20 text-success for reopen, delete button uses text-danger, all focus rings G-12 compliant.
- **ui.rs ErrorBanner** — `bg-red-500/10 → bg-danger/10`, `border-red-500/30 → border-danger/30`, `text-red-400 → text-danger`. Added `role="alert"` + `aria-hidden="true"` on the decorative SVG icon. Rounded radius adjusted from `rounded-xl` to `rounded-lg` per DESIGN.md §5.5.
- **ui.rs StatusMessage** — `bg-emerald-500/10 → bg-success/10`, `text-emerald-400 → text-success` (and danger variant). `rounded-xl → rounded-lg`.
- **ui.rs ToastOverlay** — preserved 4-second auto-dismiss, 3-toast cap, fixed-position semantics. Color migration: `bg-red-500/20 → bg-danger/20`, `bg-emerald-500/20 → bg-success/20`. Added `role="status"` + `aria-live="assertive|polite"` to satisfy ARIA-live patterns. Copy / dismiss buttons gain G-12 focus rings.
- **ui.rs EmptyState** — CTA link gains G-12 focus ring; icon span gains `aria-hidden="true"`.
- **stat_card.rs** — adopted Card.elevated pattern (`bg-surface border border-outline/50 rounded-xl p-4`), value uses `font-display text-primary text-2xl tabular-nums` (Cormorant Garamond + tabular-nums per DESIGN.md §3.3), label uses `font-imperial uppercase tracking-wider text-xs text-muted` (Cinzel imperial eyebrow).
- **17-OD-MAP.md** — opponents + action-items rows updated: `Plan: 03 → 03d`, `OD UUID: TBD → 7e1c0a92-1b3d-47fe-b5c8-2f3a4b5c6d7e`, `OD HTML path: TBD → .od/projects/7e1c0a92-…/{opponents,action-items}.html`, `Status: pending → ported`.

## Task Commits

1. **Task 1 — restyle /opponents and /action-items via OD HTML port** — `dbb7b6e`
   - `feat(17-03d): restyle /opponents + /action-items via Open-Design HTML`
   - 3 files changed, +305 / -299
   - Includes 17-OD-MAP.md updates for both surfaces

2. **Task 2 — restyle src/components/ui.rs** — `59faf8b`
   - `refactor(17-03d): restyle ui.rs with semantic tokens (ErrorBanner, StatusMessage, ToastOverlay, EmptyState)`
   - 1 file, +17 / -12

3. **Task 3 — restyle src/components/stat_card.rs (Card.elevated)** — `e805e32`
   - `refactor(17-03d): restyle stat_card.rs with Card.elevated pattern`
   - 1 file, +11 / -3

4. **Task 4 — checkpoint:human-verify** — auto-approved (auto-mode active)
   - All automated guards passed: `cargo check --features ssr` green, `cargo check --features hydrate --target wasm32-unknown-unknown` green, no raw hex, no legacy color classes, no Google Fonts CDN, every modified file's interactive elements carry the G-12 focus ring class string.

## Files Created/Modified

### Created
- `.planning/phases/17-ui-consolidation/17-03d-SUMMARY.md` (this file)

### Modified
- `src/pages/opponents.rs` (1336 lines, +/- visual layer only — server fns 11–340 untouched, reactive signal lifecycle unchanged)
  - Page wrap: `<div class="max-w-7xl mx-auto px-4 sm:px-6 py-8">` → `<div class="canvas-grain bg-base min-h-screen"><div class="max-w-7xl mx-auto px-4 sm:px-6 py-8">…</div></div>`
  - h1: `text-2xl font-semibold text-primary` → `font-display italic text-primary text-3xl`
  - List panel header: `text-sm font-semibold text-secondary` → `font-imperial uppercase tracking-wider text-xs text-muted`
  - Selected row: extra `border-l-4 border-l-accent` indicator
  - Form / detail card: `bg-surface rounded-xl border border-divider` → `bg-elevated border border-divider rounded-xl`
  - Player slot rows: `bg-elevated rounded-lg border border-divider/50` → `bg-surface border border-outline/50 rounded-xl`
  - OTP badge: `bg-orange-500/20 text-orange-400 border-orange-500/30` → `bg-warning/15 text-warning border-warning/30`
  - Stale-recency: `text-orange-400` → `text-warning`
  - Fetch success/error glyphs: `text-green-400 / text-red-400` → `text-success / text-danger`
  - Champion pills: `bg-surface border border-divider/50 text-secondary` → `bg-elevated border border-divider text-secondary`
  - Confirm-delete button: `bg-red-700 hover:bg-red-600 text-white` → Button.destructive (`bg-danger/10 text-danger border border-danger/30 hover:bg-danger/20`)
  - Inline-delete link: `text-red-400 hover:text-red-300` → `text-danger hover:opacity-80`
  - Validation errors: `text-red-400` → `text-danger`
  - All buttons / links / inputs gained `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none` (or ring-danger/50 on destructive)
- `src/pages/action_items.rs` (513 lines, +/- visual layer only — server fns 10–158 untouched)
  - Page wrap: `<div class="max-w-4xl mx-auto p-6">` → `<div class="canvas-grain bg-base min-h-screen"><div class="max-w-4xl mx-auto p-6">…</div></div>`
  - h1: `text-2xl font-bold text-primary` → `font-display italic text-primary text-3xl`
  - Add Action Item card: `bg-surface border border-divider rounded-lg p-4` → `bg-elevated border border-divider rounded-xl p-6` (Card.plain), heading switched from `text-primary font-semibold` to `font-imperial uppercase tracking-wider text-xs text-muted`
  - Item row card: `bg-surface border border-divider rounded-lg` → `bg-elevated border border-divider rounded-lg`
  - Status dots: `bg-green-500 / bg-yellow-500 / bg-gray-500` → `bg-success / bg-warning / bg-muted`
  - Reopen pill: `bg-green-500/20 text-green-400` → `bg-success/20 text-success`
  - Delete pill: `text-red-400 hover:text-red-300` → `text-danger hover:opacity-80` + focus-visible:ring-danger/50
  - Stat-row chips: `bg-surface border border-divider` → `bg-surface border border-outline/50`
  - Inputs/selects: `focus:outline-none focus:border-accent` → `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none`, placeholder `text-muted` → `text-dimmed` (matches DESIGN.md §5.3)
  - Assigned-pill: `bg-elevated text-secondary rounded` → `bg-surface text-secondary rounded-md`
  - "Back to Dashboard" link gains G-12 focus ring on `<A>`
- `src/components/ui.rs` (244 lines after edit) — full semantic-token migration (covered above)
- `src/components/stat_card.rs` (27 lines after edit) — Card.elevated pattern (covered above)
- `.planning/phases/17-ui-consolidation/17-OD-MAP.md` (137 lines) — 2 row updates

## Decisions Made

See frontmatter `key-decisions` for canonical list. Highlights:

1. **OD project reuse.** Phase 17 utility-tier project at `7e1c0a92-1b3d-47fe-b5c8-2f3a4b5c6d7e` was already seeded with both `opponents.html` and `action-items.html` artifacts. Reused that UUID rather than creating a new project — matches OD-MAP guidance ("a single OD project can host many `.html` artifacts").
2. **Button.destructive over CLAUDE.md exception.** Confirm-delete button could have kept `bg-red-700 text-white` per the project's "colored buttons with white text" exception. We chose the OD Button.destructive pattern because §5.2 calls it the documented primitive for irreversible actions and it's theme-aware.
3. **Warning palette for OTP badges.** OTP badges and stale-recency markers were originally orange. Mapped to `--warning` (Demacia: gilt amber `#d4974a`; Pandemonium: bright yellow `#fff157`). Both render as a clear "caution" tier on both themes.
4. **Tabular-nums on StatCard.** Added `tabular-nums` Tailwind utility per DESIGN.md §3.3 (tabular numerics for stats). Pre-existing component lacked this.

## Deviations from Plan

### Rule 3 — auto-fixed blocking issue (executor / tooling environment)

**1. [Rule 3 — Tooling] Initial Edit operations applied to outer repo, not worktree**
- **Found during:** Task 1 (post-edit verification of opponents.rs)
- **Issue:** When this executor operates inside a parallel worktree at `/home/jasper/Repositories/lol_team_companion/.claude/worktrees/agent-a0751db3930383dd0/`, the user's main checkout (`/home/jasper/Repositories/lol_team_companion`) was simultaneously open in another tool surface. Edit calls that took *relative* paths (or paths I had passed earlier without the worktree prefix) silently routed to the OUTER repo. Symptom: `cargo check` finished in 0.17s (cache hit, no source rebuilt), `git diff` empty in the worktree, `md5sum` unchanged on the worktree's file, but Edit tool reported "successfully updated".
- **Fix:** (a) captured the outer-repo diff via `git diff src/pages/opponents.rs src/pages/action_items.rs > /tmp/opponents_action_items.patch`; (b) `git checkout -- <files>` reverted only the two source files in the outer repo (preserving the user's other unrelated working-tree changes — the `.claude/skills` deletions remain untouched); (c) `git apply /tmp/opponents_action_items.patch` applied the patch cleanly to the worktree; (d) all subsequent edits used absolute paths to the worktree root.
- **Verification:** `git diff --stat` in worktree = 3 files modified; `md5sum` shows different hash; `git status` in outer repo = my pages back to baseline (only the user's pre-existing unrelated changes remain).
- **Lesson recorded in patterns-established frontmatter** for future executors to recognize the symptom and use absolute worktree paths from the start.
- **No commit deviation required** — the eventual commit landed in the worktree.

### Rule 2 — auto-added missing critical functionality (accessibility)

**2. [Rule 2 — Accessibility] Missing ARIA on ToastOverlay + ErrorBanner**
- **Found during:** Task 2 (ui.rs restyle — review against DESIGN.md §5.5 + §9.4)
- **Issue:** Original `ErrorBanner` lacked `role="alert"` and the decorative SVG icon lacked `aria-hidden`. Original `ToastOverlay` lacked `role="status"` + `aria-live`. DESIGN.md §9.4 mandates these for ARIA-live regions and role-indicating components.
- **Fix:** Added `role="alert"` to ErrorBanner div + `aria-hidden="true"` to its decorative SVG; added `role="status"` + `aria-live="assertive"` (error toasts) / `aria-live="polite"` (success toasts) to ToastOverlay items; added `aria-hidden="true"` on EmptyState's emoji icon span.
- **Files modified:** `src/components/ui.rs`
- **Committed in:** `59faf8b` (Task 2)

**3. [Rule 2 — Accessibility] Missing G-12 focus rings on inputs in modified pages**
- **Found during:** Task 1 (opponents.rs port; pre-existing inputs used `focus:outline-none focus:border-accent`, no ring)
- **Issue:** Per Phase 17 plan-01 deferred-items.md, the codebase has 62 pre-existing G-12 violations using the legacy `focus:outline-none focus:border-accent` pattern. Plan 03d's UI restyle is the natural migration point for the inputs in opponents.rs and action_items.rs.
- **Fix:** Migrated every input/select/textarea in the two pages and ui.rs's toast buttons to `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none` (or `ring-danger/50` on destructive variants).
- **Scope:** Only inputs in files touched by this plan — `opponents.rs`, `action_items.rs`, `ui.rs`, `stat_card.rs`. Other pages remain on the legacy pattern per the deferred-items plan (their hub plans 04 / 05 / 06 own the migration).

## Authentication Gates

None encountered.

## Threat Flags

None — surface introduced is purely visual (Tailwind class strings on existing DOM elements). No new network endpoints, no new schema fields, no auth-flow changes. T-17-14 (OD HTML import tampering) was mitigated by manually transcribing class strings (no `<script>` or external `<link>` tags imported); T-17-12d (canvas-grain inline SVG noise) is unchanged from plan 01 (already accepted).

## Known Stubs

None. All visual changes wire to existing data flows. The pages were already wired to live server fns (Riot API intel fetch, action-item CRUD, opponent CRUD); no placeholder data introduced.

## Verification Checklist

- [x] `cargo check --features ssr` exits 0 (worktree)
- [x] `cargo check --features hydrate --target wasm32-unknown-unknown` exits 0 (worktree)
- [x] `grep -c canvas-grain src/pages/opponents.rs` → 2 (page wrap + solo-mode gate)
- [x] `grep -c canvas-grain src/pages/action_items.rs` → 1 (page wrap)
- [x] `grep -c bg-danger src/components/ui.rs` → 7
- [x] No `bg-red-500` in `src/components/ui.rs`
- [x] No raw hex `#[0-9a-f]{6}` in any modified file
- [x] No legacy `(text|bg|border)-(red|green|yellow|orange|gray|emerald)-[0-9]` classes in any modified file
- [x] No Google Fonts CDN references (`fonts.googleapis|fonts.gstatic`) in any modified file
- [x] All interactive elements in modified files carry `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none` (or `ring-danger/50` on destructive variants)
- [x] OD-MAP.md status for opponents + action-items = `ported`
- [x] All 3 task commits made atomically; no STATE.md or ROADMAP.md modifications (per parallel-execution rule)
- [x] Public component APIs (StatCard, ErrorBanner, StatusMessage, ToastProvider, ToastContext, ToastKind, EmptyState, NoTeamState, SkeletonLine/Card/Grid) unchanged

## Self-Check: PASSED

All claimed files exist on disk and all 3 commits are reachable in `git log`:

- `FOUND: src/pages/opponents.rs`
- `FOUND: src/pages/action_items.rs`
- `FOUND: src/components/ui.rs`
- `FOUND: src/components/stat_card.rs`
- `FOUND: .planning/phases/17-ui-consolidation/17-OD-MAP.md`
- `FOUND: .planning/phases/17-ui-consolidation/17-03d-SUMMARY.md` (this file)
- `FOUND: dbb7b6e — feat(17-03d): restyle /opponents + /action-items via Open-Design HTML`
- `FOUND: 59faf8b — refactor(17-03d): restyle ui.rs with semantic tokens`
- `FOUND: e805e32 — refactor(17-03d): restyle stat_card.rs with Card.elevated pattern`
