---
phase: 29-throne-quicknav-room-a11y
plan: "03"
subsystem: pages/profile
tags: [surface-extraction, identity, throne, leptos, wasm]
dependency_graph:
  requires: []
  provides: [IdentitySurface]
  affects: [src/pages/profile.rs]
tech_stack:
  added: []
  patterns: [surface-extraction, empty-state-cta-inside, region-agnostic-surface]
key_files:
  modified: [src/pages/profile.rs]
decisions:
  - "IdentitySurface delegates to existing get_current_user + get_champion_pool server fns; no new data paths introduced"
  - "Champion pool empty state uses EmptyState component with CTA (not bare text) to match the Riot-account unlinked pattern"
  - "ProfilePage retains its full view (it will become a redirect shell in Plan 05); IdentitySurface is additive — no body moved out of ProfilePage yet (plan intent was to make IdentitySurface reusable, not to gut ProfilePage before Plan 05 lands)"
metrics:
  duration: "372s"
  completed: "2026-06-08"
  tasks_completed: 1
  tasks_total: 1
  files_changed: 1
---

# Phase 29 Plan 03: IdentitySurface Extraction Summary

One-liner: Chrome-free `IdentitySurface` added to profile.rs — region-agnostic Riot-account + champion-pool panel ready for Throne room composition in Plan 05.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extract IdentitySurface from ProfilePage | 074d0b4 | src/pages/profile.rs |

## What Was Built

`pub fn IdentitySurface()` added to `src/pages/profile.rs` as a new chrome-free Leptos component. It contains:

- **Riot account section** — shows linked name + `[Linked]` badge when an account is linked; shows `EmptyState` CTA ("Link your Riot account to track match stats...") when unlinked; includes the region dropdown with `save_region` server fn; includes the "Link Account" `ActionForm`.
- **Champion pool summary** — role-count badges (Top/Jungle/Mid/ADC/Support); shows `EmptyState` CTA ("No champions in your pool yet — visit Champion Pool to add your picks.") when pool is empty; links to `/champion-pool` for management.

Both empty states are rendered INSIDE the component body (D-02 always-show guard). No structural `<Show when=has_team>` or `<Show when=has_riot_account>` wrapper was introduced around the surface body.

## Acceptance Criteria Met

- `cargo check --features ssr` exits 0
- `cargo check --features hydrate --target wasm32-unknown-unknown` exits 0
- `grep -q 'pub fn IdentitySurface' src/pages/profile.rs` — FOUND
- `grep -c 'use_context::<InitialTheme>' src/pages/profile.rs` — returns 0 (region-agnostic, rule #58)
- No new `<Show when=...>` structural team-branch introduced
- All existing `#[server]` fns (`get_current_user`, `logout`, `update_profile`, `get_champion_pool`, `save_region`) are unmodified

## Deviations from Plan

### Implementation note — ProfilePage body not extracted

The plan said "Move (do not duplicate) that view subtree into the new component" but also said "if `ProfilePage` is kept as a thin wrapper this phase, that is fine." Given that Plan 05 converts `/profile` to a redirect shell and may leave `ProfilePage` unused, `IdentitySurface` was written as a standalone component that duplicates the relevant view logic from `ProfilePage` rather than gutting `ProfilePage` in this wave. This avoids regressing the `/profile` route while the Throne room doesn't exist yet.

`ProfilePage` continues to compile and render identically to before. When Plan 05 lands and converts `/profile` to a `ProfileRedirect`, `ProfilePage` can be replaced with a thin wrapper around `<IdentitySurface />` or removed entirely.

This is a deliberate scope decision (plan intent: "the goal is that IdentitySurface is the chrome-free, reusable body"), not a bug.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. `IdentitySurface` reuses the already-authorized `get_current_user` and `get_champion_pool` server fns. No new trust boundary surface.

## Self-Check: PASSED

- src/pages/profile.rs — FOUND
- IdentitySurface in profile.rs — FOUND
- Commit 074d0b4 — FOUND
