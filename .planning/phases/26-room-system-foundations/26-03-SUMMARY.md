---
phase: 26-room-system-foundations
plan: 03
subsystem: ui
tags: [leptos, wasm, web-sys, parallax, accessibility, prefers-reduced-motion, room-system]

# Dependency graph
requires:
  - phase: solo-journey (pre-26)
    provides: AtmosphereRoot lib/armory/banner axes + mousemove parallax bridge
provides:
  - AtmosphereRoot generalized to all twelve room axes (Solo/War/Library/Throne) with data-*-tier emissions
  - Runtime prefers-reduced-motion MediaQueryList change-listener (freeze on ON, restore on OFF, no reload)
affects: [war-room, 27, library, throne, 28, 29]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Optional Signal<u8> axis props default to a 0-yielding signal; one data-{axis}-tier per axis"
    - "Runtime MQL change-listener built/stored up-front so OFF can re-attach mousemove even if started reduced; torn down in on_cleanup"

key-files:
  created: []
  modified:
    - src/components/room/parallax.rs

key-decisions:
  - "Change closure typed FnMut() reading MediaQueryList.matches() — avoids adding the MediaQueryListEvent web-sys feature (no Cargo.toml change / full rebuild)"
  - "Mousemove closure created + stored before the reduced-motion gate so the started-reduced → toggled-off restore path works"

patterns-established:
  - "AtmosphereRoot is the shared 12-axis atmosphere root every room consumes; growth tiers ride as data-* attributes (no structural branching → hydration-safe)"

requirements-completed: [ROOM-02, A11Y-01]

# Metrics
duration: 35min
completed: 2026-06-05
---

# Phase 26 / Plan 03: AtmosphereRoot Generalization + Runtime Reduce-Motion Listener Summary

**AtmosphereRoot now carries all twelve room axes (Solo/War/Library/Throne) as `data-*-tier` emissions, and a runtime prefers-reduced-motion `MediaQueryList` change-listener freezes parallax on a mid-session Reduce-Motion ON and restores it on OFF — no reload — torn down on cleanup.**

## Performance

- **Duration:** ~35 min including live browser verification
- **Completed:** 2026-06-05
- **Tasks:** 3 (2 code + 1 human-verify checkpoint)
- **Files modified:** 1

## Accomplishments
- Task 1 (D-10): added the 9 War/Library/Throne axis `Signal<u8>` props and `data-*-tier` emissions; Solo `lib/armory/banner` unchanged; GrowthOverlay axis doc lists all twelve. `solo_journey.rs` untouched.
- Task 2 (A11Y-01, D-11): runtime MQL `change` listener — ON freezes (`--px`/`--py` → 0) and detaches the mousemove handler; OFF re-attaches it; the pair is removed in `on_cleanup` (T-26-03). No WASM `.unwrap()`.
- Task 3: human-verify checkpoint — verified live and **approved by the user**.

## Task Commits

1. **Task 1 + Task 2 (same file): AtmosphereRoot axes + reduce-motion listener** - `076b04d` (feat)

_Tasks 1 and 2 edit the same file (`parallax.rs`) — combined in one commit, as the plan itself combined them for that reason._

## Files Created/Modified
- `src/components/room/parallax.rs` - 9 new axis props + 9 data-*-tier attributes + GrowthOverlay doc update; restructured hydrate Effect with the runtime MQL change-listener and extended `on_cleanup`

## Decisions Made
- Typed the change closure `FnMut()` and read state from a captured `MediaQueryList.matches()` instead of a `MediaQueryListEvent` parameter — functionally identical and avoids enabling a new web-sys feature (which would force a full dependency rebuild and lies outside this plan's declared `files_modified`).
- Built and stored the mousemove closure *before* the reduced-motion gate so the started-reduced → toggled-off restore works (the gate now only controls the *initial* mousemove attach).

## Deviations from Plan
None functional. Approach note: change closure reads `MediaQueryList.matches()` rather than `MediaQueryListEvent.matches()` (see Decisions) — satisfies every acceptance criterion (`add_event_listener_with_callback("change", …)`, ON-freeze + mousemove detach, OFF re-attach, cleanup teardown, no unwraps, both targets compile).

## Issues Encountered

**Checkpoint route discrepancy (verification-doc bug, not a code defect):**
The plan's human-verify `how-to-verify` (and the relayed checkpoint) said to test at **`/solo`**. `/solo` routes to `SoloDashboardPage` (the plain dashboard, no `AtmosphereRoot`) — so it shows no parallax background. The Solo Journey **room** is at **`/journey`** (`app.rs:156` → `SoloJourneyRoom`). Verifying at `/journey` confirmed the room renders correctly. The deliverable is correct; only the verification instruction was wrong. Future room-verification docs should reference `/journey`.

## Verification (automated, live browser at /journey)
- `room-atmosphere-root` present; **all 12** `data-*-tier` attributes emit (`data-tactics-tier` … `data-crown-tier`).
- 3 parallax planes load (demacia, 1600px, `complete=true`); 3 growth overlays present.
- Parallax responds to mousemove: `--px,--py` `0,0` → `1.34,1.95`.
- **Reduce Motion ON** (CDP `set media reduced-motion` → real `change` event): `--px/--py` forced to `0,0` and a seeded `0.9,0.9` left unchanged by a subsequent mousemove → mousemove **detached** (freeze).
- **Reduce Motion OFF**: mousemove → `--px/--py` back to `1.34,1.95` → handler **re-attached** (restore).
- **No tachys `Option::unwrap` hydration panic** (only pre-existing resource-read-outside-Suspense warnings).
- `cargo check --features ssr` and `cargo check --features hydrate --target wasm32-unknown-unknown` both exit 0; `solo_journey.rs` unmodified.

## User Setup Required
None.

## Next Phase Readiness
- War Room (Phase 27) can pass `tactics_tier`/`intel_tier`/`command_tier` into `AtmosphereRoot` and rely on the runtime reduce-motion gate.

## Self-Check: PASSED
- 12-axis generalization renders live; runtime reduce-motion freeze/restore confirmed; no hydration panic; both targets compile; Solo Journey regression clean. Checkpoint approved by user.

---
*Phase: 26-room-system-foundations*
*Completed: 2026-06-05*
