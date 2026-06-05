---
phase: 27-war-room
plan: "05"
subsystem: tree-drafter-surface
tags: [leptos, refactor, extract-then-compose, region-threading]
dependency_graph:
  requires: [27-01, 27-02, 27-03]
  provides: [TreeDrafterSurface-region-prop, TreeDrafterPage-thin-shell]
  affects: [src/pages/tree_drafter.rs]
tech_stack:
  added: []
  patterns:
    - "extract-then-compose: TreeDrafterPage→TreeDrafterSurface (mirror of SoloDashboardPage→SoloSurfaces)"
    - "region-read-once at shell top (rule #58): TreeDrafterPage reads InitialTheme once, TreeDrafterSurface receives region as plain String prop"
key_files:
  created: []
  modified:
    - src/pages/tree_drafter.rs
decisions:
  - "TreeDrafterSurface view root is the max-w-[90rem] inner div; canvas-grain chrome belongs to the thin TreeDrafterPage shell"
  - "auth-redirect Effect moved into TreeDrafterSurface (mirrors SoloSurfaces and DraftSurface patterns)"
  - "TreeDrafterSurface placed before TreeDrafterPage in tree_drafter.rs (Leptos server fn ordering rule #34)"
metrics:
  duration_minutes: 12
  completed_date: "2026-06-05"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 1
---

# Phase 27 Plan 05: TreeDrafterSurface Extraction Summary

Extracted `tree_drafter.rs` (~1,679 lines) into a chrome-free `TreeDrafterSurface(region: String)` and reduced `TreeDrafterPage` to a thin page shell, following the exact same extract-then-compose boundary as D-01/D-02/D-03 with the Draft extraction (plan 27-04).

## Tasks

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 | Extract TreeDrafterSurface(region: String) and reduce TreeDrafterPage to a thin shell | da8f342 | tree_drafter.rs |
| 2 | Checkpoint: Verify /tree-drafter behavior preserved (auto-verified) | — | — |

## Checkpoint Auto-Verification Evidence

Pre-authorized auto-verification per checkpoint_preauthorization. Ran all three compile/test checks:

- `cargo check --features ssr`: exit 0, 4 pre-existing warnings (unrelated to this plan)
- `cargo check --features hydrate --target wasm32-unknown-unknown`: exit 0, 3 pre-existing warnings
- `cargo test --features ssr --lib`: **148 passed, 0 failed**, 5 ignored

The definitive browser regression for `/tree-drafter` (tree graph, node add/select/switch, champion edge icons, auto-save on node switch) runs in Wave 4 (plan 27-08 e2e suite).

## Structural Assertions

- `pub fn TreeDrafterSurface(region: String)`: present in tree_drafter.rs (line 251)
- `pub fn TreeDrafterPage()`: thin shell present (line 1114)
- `use_context::<InitialTheme>`: exactly 1 read in file — in `TreeDrafterPage` shell (rule #58 enforced)
- `TreeDrafterSurface` view root: `<div class="max-w-[90rem] mx-auto flex flex-col gap-6">` (chrome-free)
- `TreeDrafterPage` shell: `<div class="canvas-grain bg-base min-h-screen px-8 py-6" data-feedback-label="Tree drafter">` wraps `<TreeDrafterSurface region=region />`
- `children_of` map tree-assembly logic: untouched (gotcha #41)
- auto-save timer + suppress_autosave guard: untouched (rules #54/#55)
- Auth/team scoping in all server fns: unchanged

## Extraction Boundary

| What moved | From | To |
|---|---|---|
| Auth-redirect Resource + Effect | TreeDrafterPage | TreeDrafterSurface |
| All signals and RwSignals | TreeDrafterPage | TreeDrafterSurface |
| All resources (champions, trees, nodes) | TreeDrafterPage | TreeDrafterSurface |
| All callbacks (do_create_tree, select_node, do_save_node, etc.) | TreeDrafterPage | TreeDrafterSurface |
| auto-save Effect | TreeDrafterPage | TreeDrafterSurface |
| `region_for_section` clone and view content | TreeDrafterPage | TreeDrafterSurface |
| `canvas-grain bg-base min-h-screen px-8 py-6` outer div | TreeDrafterPage view | TreeDrafterPage thin shell |
| `use_context::<InitialTheme>()` read | TreeDrafterPage (kept) | TreeDrafterPage shell only |

## Deviations from Plan

None — plan executed exactly as written. Single-task extraction, no logic changes, no server fn contract changes, no Cargo.toml changes.

## Known Stubs

None — pure refactor/extraction. All data paths and server fns are preserved verbatim.

## Threat Flags

None — no new attack surface. All existing tree-drafter server fns retain their auth + team-scoping unchanged (T-27-05-NA confirmed).

## Self-Check: PASSED

- [x] src/pages/tree_drafter.rs exists and modified
- [x] Commit da8f342 exists (`feat(27-05): extract TreeDrafterSurface...`)
- [x] `pub fn TreeDrafterSurface(region: String)` present
- [x] `pub fn TreeDrafterPage()` present as thin shell
- [x] `use_context::<InitialTheme>` count = 1 (TreeDrafterPage shell only)
- [x] SSR: exit 0; hydrate/wasm: exit 0; 148 unit tests pass
