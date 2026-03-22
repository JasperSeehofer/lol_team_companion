---
phase: 07-ux-polish
plan: 01
subsystem: ui-components, models, pages
tags: [ux-polish, toast, timestamp, profile]
dependency_graph:
  requires: []
  provides: [format_timestamp utility, toast-top-16 positioning, profile single CTA]
  affects: [src/components/ui.rs, src/models/utils.rs, src/pages/profile.rs, src/pages/team/dashboard.rs, src/pages/champion_pool.rs]
tech_stack:
  added: []
  patterns: [format_timestamp_with_now testability pattern, relative-then-absolute timestamp display]
key_files:
  created:
    - src/models/utils.rs
  modified:
    - src/models/mod.rs
    - src/components/ui.rs
    - src/pages/profile.rs
    - src/pages/team/dashboard.rs
    - src/pages/champion_pool.rs
decisions:
  - "format_timestamp_with_now(s, now) takes explicit now for deterministic unit tests; public format_timestamp(s) calls it with Utc::now()"
  - "Used %-d (unpadded day) and %b (abbreviated month) for locale-neutral absolute format"
  - "Future timestamps (negative age) fall through to absolute format rather than showing 'in X minutes'"
metrics:
  duration: 15m
  completed_date: "2026-03-22T13:33:04Z"
  tasks_completed: 2
  files_changed: 5
requirements_addressed: [UX-04, UX-05, UX-06]
---

# Phase 07 Plan 01: Toast Polish, Timestamp Formatter, and Profile Button Dedup Summary

Toast container moved below nav, shared timestamp formatter added with 10 unit tests, and the profile page deduplicated to a single "Link Account" button.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Toast positioning fix, profile button dedup, timestamp formatter with unit tests | dac3e01 | src/models/utils.rs, src/models/mod.rs, src/components/ui.rs, src/pages/profile.rs |
| 2 | Apply format_timestamp at all call sites | 34ce328 | src/pages/team/dashboard.rs, src/pages/champion_pool.rs |

## What Was Built

### format_timestamp() — src/models/utils.rs

New shared utility with `pub fn format_timestamp(s: &str) -> String` and an internal `format_timestamp_with_now(s: &str, now: DateTime<Utc>) -> String` for deterministic unit testing.

Rules:
- Parse failure or empty input → return input unchanged
- age < 0 (future clock skew) → absolute format
- age < 60s → "just now"
- age < 60m → "{N} minute(s) ago"
- age < 24h → "{N} hour(s) ago"
- same year → "19 Mar, 14:30"
- different year → "5 Dec 2025, 14:30"

10 unit tests cover all branches.

### Toast positioning — src/components/ui.rs

ToastOverlay outer div changed from `top-4` to `top-16` so toasts appear below the `h-14` nav bar and no longer overlap navigation elements.

### Profile single CTA — src/pages/profile.rs

Removed `cta_label="Link Account"` and `cta_href="#link-account"` from the `EmptyState` call in the unlinked Riot account state. The `ActionForm` submit button labeled "Link Account" remains as the single canonical interaction point.

### Call sites updated — dashboard.rs and champion_pool.rs

- dashboard.rs: post-game review preview dates and team note dates now use `format_timestamp()`
- champion_pool.rs: champion note dates replaced `.chars().take(10)` truncation with `format_timestamp()`

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. All timestamp displays are now wired to format_timestamp().

## Verification

- `cargo test --features ssr --lib models::utils` — 10/10 tests pass
- `cargo check --features ssr` — clean
- `cargo check --features hydrate --target wasm32-unknown-unknown` — clean
- ui.rs contains `top-16` (not `top-4`)
- profile.rs does not contain `cta_label="Link Account"` or `cta_href="#link-account"`
- profile.rs still contains `"Link Account"` button text in the ActionForm
- dashboard.rs and champion_pool.rs import and use `format_timestamp`

## Self-Check: PASSED
