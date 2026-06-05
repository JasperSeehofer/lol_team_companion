---
phase: 27-war-room
plan: "04"
subsystem: draft-surface
tags: [leptos, refactor, extract-then-compose, region-threading, ban-priority]
dependency_graph:
  requires: [27-01, 27-02, 27-03]
  provides: [DraftSurface-region-prop, BanPrioritySurface, DraftPage-thin-shell]
  affects: [src/pages/draft.rs]
tech_stack:
  added: []
  patterns:
    - "extract-then-compose: DraftPage→DraftSurface + BanPrioritySurface (mirror of SoloDashboardPage→SoloSurfaces)"
    - "region-read-once at shell top (rule #58): DraftPage reads InitialTheme once, DraftSurface receives region as plain String prop"
    - "D-05: banned_champions live-draft closure stays in DraftSurface (distinct from persistent ban_priority table)"
    - "D-04: full ban-priority editor (signals + view) extracted to BanPrioritySurface with its own auth-redirect"
key_files:
  created: []
  modified:
    - src/pages/draft.rs
decisions:
  - "BanPrioritySurface placed before DraftPage/DraftSurface in draft.rs (Leptos server fn ordering rule #34)"
  - "DraftSurface view root is the max-w-[1600px] inner div; canvas-grain chrome belongs to the thin DraftPage shell"
  - "BanPrioritySurface rendered via <BanPrioritySurface /> inside DraftSurface for the /draft flat route (D-02 behavior preserved)"
metrics:
  duration_minutes: 18
  completed_date: "2026-06-05"
  tasks_completed: 3
  tasks_total: 3
  files_changed: 1
---

# Phase 27 Plan 04: DraftSurface + BanPrioritySurface Extraction Summary

Extracted `draft.rs` (~4,525 lines, project's largest file) into two chrome-free reusable surface components — `DraftSurface(region: String)` and `BanPrioritySurface` — and reduced `DraftPage` to a thin page shell. This is the highest-risk extraction in Phase 27 (D-01, D-02, D-03, D-04, D-05).

## Tasks

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 | Extract BanPrioritySurface (server fns + signals + editor view) out of DraftPage | c3418e5 | draft.rs |
| 2 | Extract DraftSurface(region: String) and reduce DraftPage to a thin shell | 5234a63 | draft.rs |
| 3 | Checkpoint: Verify /draft behavior preserved after extraction (auto-verified) | — | — |

## Checkpoint Auto-Verification Evidence

Pre-authorized auto-verification per checkpoint_preauthorization. Ran all three compile/test checks with clean results:

- `cargo check --features ssr`: exit 0, 4 pre-existing warnings (unrelated to this plan)
- `cargo check --features hydrate --target wasm32-unknown-unknown`: exit 0, 3 pre-existing warnings
- `cargo test --features ssr --lib`: **148 passed, 0 failed**, 5 ignored

The definitive browser regression for `/draft` (champion picker, banned_champions behavior, blue/red toggle) runs in Wave 4 (plan 27-08 e2e suite).

## Structural Assertions

- `pub fn BanPrioritySurface()`: present in draft.rs, no `region` parameter
- `pub fn DraftSurface(region: String)`: present, receives region as plain prop
- `banned_champions` closure: inside DraftSurface (D-05 confirmed via source)
- `use_context::<InitialTheme>`: exactly 1 read in file — in `DraftPage` shell (rule #58 enforced)
- Ban signals (`ban_priorities`, `ban_panel_open`, `editing_bans`, `ban_edit_list`, `ban_new_champ`, `ban_new_reason`, `ban_status`): inside `BanPrioritySurface`, not in DraftSurface
- Auth/team scoping in `get_ban_priorities`/`save_ban_priorities`: unchanged (T-27-04-01, T-27-04-02)

## Extraction Boundary

| What moved | From | To |
|---|---|---|
| Ban signals (7 signals + Resource) | DraftPage (lines 1012–1019) | BanPrioritySurface |
| Ban-priority editor view | DraftPage inline view | BanPrioritySurface view! |
| Auth-redirect Effect | (new, mirrored from SoloSurfaces) | BanPrioritySurface |
| All draft planning content | DraftPage | DraftSurface |
| `banned_champions` closure | DraftPage | DraftSurface (stays, D-05) |
| `canvas-grain bg-base min-h-screen` outer div | DraftPage view | DraftPage shell |
| `use_context::<InitialTheme>()` read | DraftPage (kept) | DraftPage shell only |

## Deviations from Plan

None — plan executed exactly as written. The extraction followed the documented boundary precisely: BanPrioritySurface extracted first (Task 1), then DraftSurface (Task 2). No logic changes, no server fn contract changes, no Cargo.toml changes.

## Known Stubs

None — this is a pure refactor/extraction. All data paths and server fns are preserved verbatim.

## Threat Flags

None — no new attack surface. The `get_ban_priorities`/`save_ban_priorities` server fns are unchanged; auth + team-scoping preserved as documented in T-27-04-01 and T-27-04-02.

## Self-Check: PASSED

- [x] src/pages/draft.rs exists and modified
- [x] Commit c3418e5 exists (`feat(27-04): extract BanPrioritySurface`)
- [x] Commit 5234a63 exists (`feat(27-04): extract DraftSurface + thin DraftPage shell`)
- [x] `pub fn BanPrioritySurface()` present — no region param, no InitialTheme read
- [x] `pub fn DraftSurface(region: String)` present
- [x] `banned_champions` present in file (in DraftSurface)
- [x] `use_context::<InitialTheme>` count = 1 (DraftPage shell only)
- [x] SSR: exit 0; hydrate/wasm: exit 0; 148 unit tests pass
