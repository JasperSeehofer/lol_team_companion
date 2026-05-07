---
phase: 17-ui-consolidation
plan: 03b
subsystem: ui
tags: [tree-drafter, tree-graph, champion-autocomplete, restyle, demacia, pandemonium, g-12, semantic-tokens, svg-theming]

# Dependency graph
requires:
  - phase: 17-ui-consolidation
    provides: "demacia/pandemonium tokens + canvas-grain (17-01); 17-UI-SPEC §Tree Graph Interactions; 17-PATTERNS tree_graph.rs analog (lines 491-531)"
provides:
  - "tree_graph.rs with 5 logical node states (Locked / Alternate / Ghost / Leaf + reactive Selected) painted via CSS-variable strokes"
  - "Animated dash-flow on selected edges via scoped @keyframes dashFlow inline in the SVG"
  - "Side-tinted edges: blue picks/bans -> info (lapis), red -> danger; ghost lineage -> muted dashed"
  - "Leaf nodes render circular aureole overlay (NODE_H=56 enables centred circle)"
  - "tree_drafter page wrapped in canvas-grain bg-base min-h-screen + restyled headers/forms/buttons"
  - "Inline Delete Tree confirmation (per UI-SPEC: never modal)"
  - "champion_autocomplete restyled: bg-surface dropdown, 24x24 thumbnails, focus-visible parity with hover for keyboard nav"
  - "Page-wide G-12 focus-visible:ring application across all interactive controls"
affects: [17-03c, 17-03d]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "SVG stroke theming via style=\"stroke: var(--color-…)\" (Pitfall 9) — Tailwind utilities don't reach SVG stroke; CSS variables re-evaluate on theme switch automatically"
    - "Logical node-state derivation: NodeState enum from layout-time properties (children/actions/improvised) + reactive Selected layered on top"
    - "Two-layer edge rendering: base stroke + selected overlay path with separate animated dashFlow class — base dims to opacity 0.18 when selected so animation reads cleanly"
    - "Inline destructive confirmation: RwSignal<bool> toggles a confirmation card in place, no modal/dialog component required"

key-files:
  created: []
  modified:
    - "src/components/tree_graph.rs (PRESERVED LayoutNode/compute_widths/assign_positions/to_layout_nodes; ADDED NodeState + EdgeSide enums, default_state(), diff_side(), edge_base_style(); refactored render to use CSS var styles + dashFlow keyframe; 9 new unit tests)"
    - "src/components/champion_autocomplete.rs (PRESERVED signature, on_select callback, filter_text sync Effect, blur-delay close logic; RESTYLED input + dropdown per UI-SPEC §Champion Picker UX > Autocomplete dropdown)"
    - "src/pages/tree_drafter.rs (PRESERVED all #[server] fns, debounced auto-save Effect with eager signal capture, suppress_autosave guard, branch-from-position logic, live navigator; RESTYLED page wrapper to canvas-grain, header, all form inputs, view toggle, tree-select buttons, NodeEditor controls, LiveNavigator child cards/breadcrumbs; ADDED inline delete-tree confirmation)"

key-decisions:
  - "NodeState derived once at layout time (default_state), Selected painted reactively on top — keeps the layout pass deterministic and avoids re-deriving state per render frame."
  - "Two-path edge rendering (base + selected overlay) instead of one reactive style — simplifies the @keyframes dashFlow animation lifecycle (the overlay path always exists; only its stroke goes transparent when not selected)."
  - "Side-tint mapping uses action.side string field: \"blue\"/\"us\"/\"ally\" -> Us, everything else -> Them. \"blue\"=Us preserves the project's pre-existing tint convention (info colour = lapis = our team)."
  - "Inline confirmation pattern (RwSignal<bool>) chosen over a Confirmation modal component because UI-SPEC explicitly states \"NOT a modal\" and the worktree has no modal system yet."
  - "NODE_H raised from 42 -> 56 to enable the leaf circular aureole inside the existing rect bounds without breaking the layout algo (the LEVEL_H=120 spacing absorbs the difference)."

requirements-completed: [SC-2-claude-design-implementation-tree-cluster, SC-4-ui-review-pass-tree-cluster]

# Metrics
duration: 16min
completed: 2026-05-07
---

# Phase 17 Plan 03b: Tree Drafter Cluster Restyle Summary

**Restyled the tree-drafter feature (page + tree_graph + champion_autocomplete) to demacia/pandemonium tokens with 5 visual node states, animated selected edges, side-tinted lineage, and a fully G-12-compliant focus ring pass — all behaviour preserved (CLAUDE.md rule 41 children_of HashMap DFS, debounced auto-save eager signal capture, suppress_autosave guard, on_select callback signatures).**

## Performance

- **Duration:** ~16 min
- **Started:** 2026-05-07T14:39:50Z
- **Completed:** 2026-05-07T14:55:37Z
- **Tasks:** 3 of 3 (Task 3 is a checkpoint:human-verify, auto-approved per auto-mode)
- **Files modified:** 3 source files (1 page, 2 components)

## Accomplishments

- **5 node states implemented.** `NodeState` enum (Locked / Alternate / Ghost / Leaf) derived from layout-time properties; `Selected` painted reactively over the default state. Each state ships its own stroke / fill / opacity / dasharray combination per UI-SPEC §"Tree Graph Interactions".
- **Animated edge dash-flow.** Scoped inline `<style>` with `@keyframes dashFlow` on `.tree-edge-selected`; base edge dims to opacity 0.18 while the accent overlay animates `stroke-dashoffset` 0 → -20 over 1.2s linear infinite.
- **Side-tinted edges.** `EdgeSide::Us` -> info (lapis blue), `EdgeSide::Them` -> danger (oxblood red); ghost lineage overrides with muted dashed regardless of side.
- **Theme switch survives.** All SVG strokes use `style="stroke: var(--color-…)"` (Pitfall 9) — CSS variables are re-evaluated by the browser when `<html data-theme>` flips, so demacia ↔ pandemonium swap works without a re-render.
- **canvas-grain page wrapper** + imperial wordmark + ghost-pair Edit/Live toggle restyled per UI-SPEC.
- **Inline destructive confirmation** for Delete Tree (RwSignal<bool>) replaces the immediate-action button — UI-SPEC explicitly forbids modals here.
- **G-12 sweep.** Added `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none` to every interactive control in the three modified files: Edit/Live toggle, Create Tree button, tree-select buttons, view toggle (list/graph), inline branch form inputs, NodeEditor inputs, Save Node, Clear Board, Branch from here, Improvise, breadcrumb buttons, choose-path child cards, Back. Inputs additionally get `focus-visible:border-accent/50`.
- **Semantic-token migration.** Replaced hardcoded utility colours (`text-red-400`, `text-amber-400`, `text-emerald-400`, `text-green-400`, `bg-purple-600`, `bg-amber-500/*`, `bg-gray-300`) with `text-danger`, `text-warning`, `text-success`, `bg-accent-soft` / `bg-warning` etc. Toggle thumb now uses `bg-secondary` / `bg-base` so it tracks the active theme.

## Task Commits

1. **Task 1: tree_graph.rs restyle** — `b6dcd7d` (feat)
2. **Task 2: tree_drafter.rs + champion_autocomplete.rs restyle** — `aea2791` (feat)
3. **Task 3: User review checkpoint** — auto-approved (auto mode + parallel-executor; visual review deferred to orchestrator merge gate per `<auto_mode>` directive)

2 atomic git commits total.

## Files Created/Modified

### Created
None.

### Modified
- `src/components/tree_graph.rs` (+426 / -83):
  - **Preserved:** `LayoutNode`, `to_layout_nodes`, `compute_widths`, `assign_positions`, `collect_edges`, `collect_nodes`, `canvas_size`, `diff_actions`. The recursive layout pass is byte-for-byte unchanged.
  - **Added:** `NodeState` (Locked / Alternate / Ghost / Leaf) + `LayoutNode::default_state()`, `EdgeSide` (Us / Them), `diff_side()`, `edge_base_style()`. Render path now emits a base `<path>` (state-driven style) + a selected overlay `<path>` (animated dashFlow class). Inline `<style>` block defines `@keyframes dashFlow` and `.tree-edge-selected`. Node rectangles get state-conditional fill/stroke/opacity/dasharray; leaf nodes overlay a circular aureole; ghost nodes hide the "+" add-branch button per UI-SPEC. SVG `<defs>` retained (`selected-glow`, `grayscale-ban`); `selected-glow` flood-color now uses `var(--color-accent)` without a raw-hex fallback.
  - **Tests:** 9 new unit tests cover `default_state` for all four base states, `diff_side` for blue/red/mixed/no-diff cases, and a `edge_base_style_uses_only_var_tokens` invariant that guards against future raw-hex regressions.
- `src/components/champion_autocomplete.rs` (+15 / -8):
  - **Preserved:** signature (`champions`, `value: RwSignal<String>`, optional `placeholder`, optional `on_select: Callback<String>`); `select_champion` closure firing `on_select.run(champ.id)`; `filter_text` sync Effect; blur-delay close via `Closure::once` + `set_timeout`; `on:mousedown + prevent_default` for dropdown items so click-before-blur works.
  - **Restyled:** input gets `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none focus-visible:border-accent/50`; dropdown changes `bg-elevated` -> `bg-surface` to match UI-SPEC §"Autocomplete dropdown" and `shadow-xl` -> `shadow-lg`; `max-h-56` -> `max-h-60`; items default to `text-secondary` and switch to `bg-elevated text-primary` on hover OR focus-visible (keyboard parity); thumbnails get `flex-shrink-0`; name `<span>` gets `truncate` for long champion names.
- `src/pages/tree_drafter.rs` (+101 / -44):
  - **Preserved:** all 8 `#[server]` fns (`get_champions_for_tree`, `list_trees`, `create_tree`, `delete_tree`, `update_tree_meta`, `get_tree_nodes`, `add_branch`, `save_node`, `remove_node`); helpers (`build_actions_from_slots`, `actions_to_slots`); the debounced auto-save `Effect::new` with eager signal capture (rule 54) + `auto_save_node_timer` cancellation; `suppress_autosave` guard around node/tree switches (rule 55); `select_node`, `clear_editor`, `cancel_autosave_timer`; `do_branch_from` race-window fix (BUG-01 in source). All sub-components (`TreeNodeView`, `NodeEditor`, `LiveNavigator`) keep identical signatures; only their class strings changed.
  - **Added:** `confirm_delete_tree: RwSignal<bool>` toggling an inline confirmation card. Tree-select handler resets the confirmation when switching trees so it doesn't bleed across selections.
  - **Restyled:** outer wrapper `canvas-grain bg-base min-h-screen px-8 py-6` + inner max-width container; header `font-display italic`; Edit/Live toggle now a connected button-pair inside `border border-divider/50` with focus rings; New Tree form inputs and button get focus rings; inline Delete confirmation replaces the previous immediate-action button (semantic colours: `bg-danger/20 hover:bg-danger/30 border-danger/40`); tree-select buttons add `type="button"` + focus rings; tree-view toggle gets focus rings; inline branch form inputs/buttons restyled; NodeEditor inputs/textarea/save/clear get focus rings; Branch-from-here switched from `bg-purple-600 text-white` to `bg-accent-soft border-accent/40 text-accent`; LiveNavigator breadcrumb Root + step buttons get focus rings; Improvise badge uses `bg-warning/20 text-warning`; Improvise button gets focus ring; choose-path child cards add focus rings; Back button focus ring; save-status badges use `text-success`/`text-warning` semantic tokens.
  - **G-12 deviation tracking:** This file was NOT in the Phase 17 G-12 enforcement scope (deferred-items.md tracks 62 pre-existing form-input violations). The plan-touched controls in this file are now G-12 compliant; pre-existing violations elsewhere in the worktree are out of scope and remain in the deferred-items roll-up.

## Decisions Made

See frontmatter `key-decisions`.

## Deviations from Plan

### Auto-fixed issues

**1. [Rule 2 - Critical functionality] G-12 focus rings extended beyond the stated minimum**
- **Found during:** Task 2.
- **Issue:** The plan's `<action>` for Task 2 only mandates focus rings on the autocomplete and the Add-branch buttons. However, CLAUDE.md "Critical Patterns" + accessibility-standards inheritance require focus-visible rings on every interactive control in plan-touched files (G-12). Several pre-existing controls in `tree_drafter.rs` (view toggle, tree-select, save buttons, breadcrumb buttons, child path cards, Back) lacked them.
- **Fix:** Added `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none` to every interactive control in the modified file. Each affected control also gained `type="button"` where missing (avoids accidental form submissions).
- **Rationale:** Phase 17 plan 03b's must_haves include "every visible gate ... passes per-page user review per D-10" — a partial G-12 pass would fail review and reopen the work in a follow-up plan.
- **Files modified:** `src/pages/tree_drafter.rs`.
- **Committed in:** `aea2791`.

**2. [Rule 1 - Bug] Hardcoded color utilities ignore theme switch**
- **Found during:** Task 2 (semantic-token sweep).
- **Issue:** `text-red-400` / `text-amber-400` / `text-emerald-400` / `text-green-400` / `bg-purple-600` / `bg-amber-500` / `bg-gray-300` are static Tailwind palette utilities; they do NOT swap when `data-theme="pandemonium"` flips. Save-status badges and toggle thumb would silently stay on demacia colours even under pandemonium.
- **Fix:** Migrated to semantic tokens (`text-danger`, `text-warning`, `text-success`, `bg-accent-soft`, `bg-warning`, `bg-secondary`, `bg-base`). Branch-from-here button (originally `bg-purple-600 text-white`) re-coloured to `bg-accent-soft text-accent border-accent/40` for theme parity.
- **Files modified:** `src/pages/tree_drafter.rs`.
- **Committed in:** `aea2791`.

**3. [Rule 1 - Bug] selected-glow filter raw-hex fallback in tree_graph.rs**
- **Found during:** Task 1 read of existing source.
- **Issue:** The pre-existing `<filter id="selected-glow">` used `flood-color="var(--color-accent, #6366f1)"` — the raw `#6366f1` fallback would trigger a future raw-hex CI sweep on this file and is also a stale color reference (not a current token).
- **Fix:** Stripped fallback; `var(--color-accent)` is always defined in both demacia and pandemonium token blocks (input.css plan 17-01).
- **Files modified:** `src/components/tree_graph.rs`.
- **Committed in:** `b6dcd7d`.

### Out-of-scope items not fixed

**4. Pre-existing dev server / fixture mismatch (not caused by this plan)**
- **Observed during:** Task 2 verify (`cd e2e && npx playwright test audit-tree-drafter.spec.ts`).
- **Issue:** The running dev server (PID 173292/359182) is the *main repo's* binary, not the worktree's. The fixture `authenticatePage` waits for `**/team/dashboard` after registration, but the running main-repo server redirects to `/solo`. All 4 audit-tree-drafter spec tests fail at the fixture step before any tree-drafter UI is exercised.
- **Why out of scope:** `git diff main HEAD` shows no changes to `e2e/tests/fixtures.ts` or `src/server/auth.rs` from this plan. The mismatch is between the main repo's running binary state and the fixture's documented behaviour ("registration auto-logs in and redirects to /team/dashboard"). The 17-PATTERNS.md rules forbid restarting servers from a parallel executor; the orchestrator's merge stage will run the spec against a fresh build and validate.
- **Action:** Documented here so the orchestrator's verifier knows the spec failure is pre-existing infrastructure drift, not a regression. No code change required from this plan.

## Authentication Gates

None encountered.

## Threat Flags

None — both threat-register entries (`T-17-11b` debounced auto-save preservation, `T-17-15b` children_of DFS preservation) are mitigated as planned:
- **T-17-11b:** `Effect::new` at `tree_drafter.rs:455-522` preserved verbatim, including eager signal capture (`current_label`, `current_notes`, `current_slots`, `current_improvised` read OUTSIDE the `Closure::once` body — rule 54). `auto_save_node_timer` cancellation (rule 42) and `suppress_autosave` guard (rule 55) untouched.
- **T-17-15b:** `tree_graph.rs` does not call `db::get_tree_nodes` directly; tree assembly happens upstream in `src/server/db.rs` via the `children_of: HashMap<String, Vec<String>>` DFS, which this plan never touched. Verify gate `grep -q "children_of" src/server/db.rs` returns OK.

## Known Stubs

None. All UI states render real data; the inline delete confirmation, all form inputs, and the autocomplete dropdown are wired to existing signals/server fns.

## Verification Checklist

- [x] `cargo check --features ssr` exits 0
- [x] `cargo check --features hydrate --target wasm32-unknown-unknown` exits 0
- [x] `cargo test --features ssr --lib` — 111 tests pass (102 pre-existing + 9 new tree_graph tests)
- [x] `grep -q "children_of" src/server/db.rs` (CLAUDE.md rule 41 invariant)
- [x] `grep -q "LayoutNode\|compute_widths" src/components/tree_graph.rs` (layout algo preserved)
- [x] `grep -q "stroke: var" src/components/tree_graph.rs` (Pitfall 9 SVG var-token discipline)
- [x] `grep -q "canvas-grain" src/pages/tree_drafter.rs` (page wrapper)
- [x] `grep -q "on_select" src/components/champion_autocomplete.rs` (callback preserved)
- [x] `grep -q "focus-visible:ring" src/components/champion_autocomplete.rs` (G-12)
- [x] `! grep -qE "#[0-9a-fA-F]{6}" src/components/tree_graph.rs src/pages/tree_drafter.rs src/components/champion_autocomplete.rs` (no raw hex)
- [ ] `cd e2e && npx playwright test audit-tree-drafter.spec.ts` — pre-existing fixture drift documented as Deviation #4; not caused by this plan.

## Self-Check: PASSED

All claimed source files modified; both task commits present in `git log`; both compile targets exit 0; new tree_graph unit tests all pass; no raw hex; CLAUDE.md rule 41 invariant intact.
