---
phase: 27-war-room
plan: "02"
subsystem: ui
tags: [leptos, wasm, parallax, atomicbool, rust]

# Dependency graph
requires:
  - phase: 26-room-system-foundations
    provides: AtmosphereRoot component with parallax mousemove listener lifecycle
provides:
  - AtomicBool MOUSEMOVE_ACTIVE accumulation guard in AtmosphereRoot preventing doubled mousemove listeners on room round-trips
affects: [27-war-room, 28-library-room, 29-throne-room]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "std::sync::atomic::AtomicBool as unconditional module-level static, guarding hydrate-only listener registration via compare_exchange"
    - "compare_exchange(false, true) gate placed after reduced-motion early-return so reduced-motion mounts never claim the flag"
    - "store(false) reset in on_cleanup fires before listener drop — flag clears even if window() is unavailable"

key-files:
  created: []
  modified:
    - src/components/room/parallax.rs

key-decisions:
  - "Declare static MOUSEMOVE_ACTIVE unconditionally (not cfg-gated) — SSR reads but never fires the hydrate-only compare_exchange path; no special wasm32 cfg needed (Open Question 2 resolved)"
  - "compare_exchange guard sits AFTER reduced-motion early-return so prefers-reduced-motion mounts never claim the flag and cannot block a later full-motion mount from registering"
  - "store(false) placed as first statement in on_cleanup, before listener.update_value, so flag clears even if the web_sys::window() call inside the removal block returns None"

patterns-established:
  - "AtomicBool accumulation guard pattern: static flag + compare_exchange before registration + store(false) in on_cleanup — reusable for any singleton hydrate-only browser listener"

requirements-completed: [ROOM-04]

# Metrics
duration: 8min
completed: 2026-06-05
---

# Phase 27 Plan 02: AtmosphereRoot AtomicBool Mousemove Accumulation Guard Summary

**`static MOUSEMOVE_ACTIVE: AtomicBool` gate in `AtmosphereRoot` caps the window `mousemove` listener count at one across War Room / Solo Journey room round-trips (Leptos #3665)**

## Performance

- **Duration:** 8 min
- **Started:** 2026-06-05T17:25:23Z
- **Completed:** 2026-06-05T17:33:43Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Added `static MOUSEMOVE_ACTIVE: AtomicBool = AtomicBool::new(false)` at module level — unconditional, SSR-harmless
- Inserted `compare_exchange(false, true, SeqCst, SeqCst)` guard inside the `#[cfg(feature = "hydrate")]` block, positioned after the reduced-motion early-return so reduced-motion mounts never claim the flag
- Added `MOUSEMOVE_ACTIVE.store(false, SeqCst)` as first statement in `on_cleanup`, clearing the flag before the listener removal so the next `AtmosphereRoot` mount can register successfully even if `web_sys::window()` is unavailable
- Both SSR and WASM compile targets pass with zero new errors

## Task Commits

Each task was committed atomically:

1. **Task 1: Add AtomicBool mousemove accumulation guard to AtmosphereRoot** - `b5b8921` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src/components/room/parallax.rs` - Added AtomicBool guard: import, static declaration, compare_exchange gate, store(false) reset in on_cleanup

## Decisions Made
- Declared the static unconditionally (not under `#[cfg(feature = "hydrate")]`): SSR never reaches the `compare_exchange` path (it is inside the hydrate block), so no `wasm32` cfg is needed. This matches the research conclusion in Open Question 2.
- Positioned `compare_exchange` after the reduced-motion `if reduced { return; }` block: a reduced-motion mount must not claim the flag, or it would permanently block a full-motion mount from registering.
- `MOUSEMOVE_ACTIVE.store(false)` is the first line in `on_cleanup` (before `listener.update_value`): the flag clears whether or not the subsequent listener-removal succeeds, ensuring no permanent flag lock if `window()` is unavailable.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. Compile passed first time on both targets with no new warnings related to this change.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Change is client-side WASM only. T-27-02-D (listener accumulation DoS) is now mitigated by the AtomicBool cap. T-27-02-SC not triggered (zero Cargo.toml changes; `std::sync::atomic` is std).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The accumulation guard is in place before Plan 03 introduces the War Room route where two `AtmosphereRoot` instances can coexist. The guard is now load-bearing.
- Plans 03-08 can proceed. The SC4 e2e round-trip test (`getEventListeners(window).mousemove.length === 1`) is deferred to Plan 03/08 per the plan's done-criteria note.

## Self-Check

- [x] `src/components/room/parallax.rs` — file modified and confirmed present
- [x] Commit `b5b8921` — verified via `git rev-parse --short HEAD`
- [x] `cargo check --features hydrate --target wasm32-unknown-unknown` — exited 0
- [x] `cargo check --features ssr` — exited 0
- [x] `grep -q 'static MOUSEMOVE_ACTIVE'` — PASS
- [x] `grep -q 'compare_exchange'` — PASS
- [x] `grep -q 'MOUSEMOVE_ACTIVE.store(false'` — PASS
- [x] `compare_exchange` at line 215 > reduced-motion return at line 205 — PASS

## Self-Check: PASSED

---
*Phase: 27-war-room*
*Completed: 2026-06-05*
