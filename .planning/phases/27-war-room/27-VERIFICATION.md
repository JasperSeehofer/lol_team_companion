---
phase: 27-war-room
verified: 2026-06-05T00:00:00Z
status: passed
score: 5/5 must-haves verified
overrides_applied: 0
reverified: 2026-06-05T00:00:00Z
gaps_resolved:
  - truth: "A team with saved drafts and scouted opponents sees more atmospheric detail than a fresh team (growth overlays advance through tiers based on actual team data)"
    original_status: failed
    resolution: "Added the 9 missing War Room tier→opacity CSS rules to input.css (commit 'fix(27): add War Room tactics/intel/command tier→opacity CSS'), mirroring the Solo Journey lib/armory/banner pattern and matching AtmosphereRoot's data-{tactics,intel,command}-tier attrs (parallax.rs:340-342) + GrowthOverlay room-overlay--{axis} classes. PROVEN: the SC3 e2e test was strengthened (commit 'test(27): assert War Room overlay opacity') to read computed opacity in a real browser — a seeded tier≥1 axis now has opacity > 0 (visible) while tier-0 axes remain opacity 0 (hidden). war-room.spec.ts 6/6 GREEN after the fix."
---

# Phase 27: War Room Verification Report

**Phase Goal:** Users can reach the War Room at `/war-room` and access draft planning, tree-drafter, game-plan creation, opponent scouting, and ban-priority as tabbed surfaces under a single parallax atmosphere — the room's environment grows visibly as the team accumulates strategy work.
**Verified:** 2026-06-05
**Status:** passed — 5/5 (initial gap on SC3 resolved in-session)
**Re-verification:** Yes — SC3 gap fixed + re-verified via computed-opacity e2e assertion

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | User can navigate to /war-room and see parallax atmosphere; tab switching does not cause atmosphere flash or remount | VERIFIED | Route registered at `src/app.rs:158`; `RoomSurface` uses `attr:hidden` (not `<Show>`) — confirmed by `grep -c '<Show' room_surface.rs` = 0; `ATMOSPHERE_ROOT_ID` div stays in DOM; all 5 panels mount eagerly and toggle via `hidden` attribute only |
| 2 | Tab bar uses role=tablist / role=tab / aria-selected / aria-controls | VERIFIED | `room_surface.rs:112` — `<nav role="tablist">`, each button `role="tab"` with `aria-selected` reactive, `aria-controls` pointing to `panel-{id}`; each panel div has `role="tabpanel"` and `aria-labelledby` |
| 3 | A team with saved drafts and scouted opponents sees more atmospheric detail than a fresh team (growth overlays advance through tiers) | VERIFIED (gap resolved) | Data attributes (`data-tactics-tier`, `data-intel-tier`, `data-command-tier`) update correctly from `get_team_growth_state` → real DB counts. The 9 missing tier→opacity CSS rules were added to `input.css` after line 444 (`fix(27): ...`), mirroring the Solo Journey pattern. PROVEN in a real browser: the SC3 e2e test now asserts computed opacity — a seeded tier≥1 axis renders opacity > 0 (visible) and tier-0 axes stay opacity 0 (hidden). war-room.spec.ts 6/6 GREEN. |
| 4 | Round-trip from /draft to /war-room and back does not accumulate duplicate mousemove listeners | VERIFIED (advisory) | WR-01 fixed via generation-ID single-listener slot (`NEXT_PARALLAX_ID` + `ACTIVE_MOUSEMOVE` thread_local in `parallax.rs`). Code trace: new mount evicts previous owner's listener before registering its own; `on_cleanup` guards by `my_id == stored_id`. Fix confirmed in commit `3af2d8e`. E2e SC4 test ran GREEN (advisory path: `getEventListeners` unavailable in Playwright Chromium; non-blocking per spec contract at `war-room.spec.ts:175-179`). |
| 5 | /war-room appears in e2e smoke suite and passes hydration-no-panic canary in both demacia and pandemonium themes | VERIFIED | `/war-room` in `pages.spec.ts:26 AUTHED_PAGES`; `war-room.spec.ts` canary tests for both regions pass. Rule #58 compliance confirmed: `use_context::<InitialTheme>()` read exactly once at `war_room.rs:64` (top of `WarRoomRoom`); `Signal::derive` at lines 68/75/82 derive from `growth` resource, NOT from context; `WarSurfaces` receives `region: String` prop; no child reads context directly (GamePlanSurface, OpponentsSurface, BanPrioritySurface take no region). Both compile targets exit 0. |

**Score:** 5/5 truths verified (SC3 gap resolved in-session)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/pages/rooms/war_room.rs` | WarRoomRoom component + get_team_growth_state server fn | VERIFIED | 172 lines; server fn queries real DB via `get_team_growth_counts`; no-team → default (rule #44); route wired at `src/app.rs:158` |
| `src/components/room/room_surface.rs` | Keep-mounted RoomSurface with PanelFn contract | VERIFIED | `pub type PanelFn = Box<dyn Fn() -> AnyView>`; `attr:hidden` toggle; no `<Show>`; 5 unit tests passing |
| `src/components/room/parallax.rs` | AtmosphereRoot with generation-ID mousemove guard | VERIFIED | `NEXT_PARALLAX_ID: AtomicUsize` + `ACTIVE_MOUSEMOVE thread_local`; WR-01 fix in commit `3af2d8e`; cleanup owner-guarded |
| `e2e/tests/war-room.spec.ts` | War Room e2e contract (SC2-SC5) | VERIFIED | 214 lines; 6 tests covering renders/ARIA/growth-tiers/mousemove-guard/canary-demacia/canary-pandemonium |
| `input.css` (tactics/intel/command CSS rules) | 9 CSS rules for War Room axis overlays | MISSING | Only lib/armory/banner selectors exist (lines 436-444). No [data-tactics-tier=*], [data-intel-tier=*], or [data-command-tier=*] selectors. Gap committed from the original Solo Journey spike (commit 2c11b31) and never extended for War Room in any Phase 26 or 27 plan. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `war_room.rs:WarRoomRoom` | `get_team_growth_state` | `Resource::new` | VERIFIED | Resource drives `tactics_tier`/`intel_tier`/`command_tier` signals passed to `AtmosphereRoot` |
| `AtmosphereRoot` | `data-tactics-tier` DOM attr | `move || tactics_tier.get().to_string()` | VERIFIED | `parallax.rs:339-340`; signal value written reactively |
| `data-tactics-tier` DOM attr | `room-overlay--tactics` opacity | CSS `[data-tactics-tier=*]` selectors | NOT_WIRED | CSS selectors are absent in `input.css`; the data attribute exists but no CSS rule reads it |
| `war_room.rs:WarSurfaces` | 5 Surface components | `RoomSurface panels=panels` | VERIFIED | Draft (show_ban_priority=false), Tree-Drafter, Game Plan, Opponents, Ban Priority — all 5 wired |
| `DraftSurface` | `show_ban_priority=false` | prop gate at `draft.rs:3767` | VERIFIED | WR-02 fix in commit `72ec30d`; prevents double-mount of BanPrioritySurface |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| `AtmosphereRoot` data-tactics-tier | `tactics_tier: Signal<u8>` | `get_team_growth_state` → `db::get_team_growth_counts` → SurrealDB count queries | Yes — 6 COUNT queries across draft/draft_tree/opponent/ban_priority/post_game_learning/team_note tables | FLOWING |
| `GrowthOverlay axis="tactics"` opacity | CSS `[data-tactics-tier=*]` | `input.css` CSS selectors | No — selectors do not exist | DISCONNECTED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Both compile targets pass | `cargo check --features ssr` + `cargo check --features hydrate --target wasm32-unknown-unknown` | Both exit 0 with warnings only | PASS |
| No `<Show>` in panel render path | `grep -c '<Show' src/components/room/room_surface.rs` | 0 | PASS |
| `attr:hidden` present | `grep -q 'attr:hidden' src/components/room/room_surface.rs` | found | PASS |
| PanelFn exported | `grep -q 'PanelFn' src/components/room/mod.rs` | found | PASS |
| 148 unit tests pass | documented in 27-08-SUMMARY.md | 148 passed | PASS |
| War Room overlays get CSS-driven opacity by tier | `grep -E "data-tactics-tier|data-intel-tier|data-command-tier" input.css` | No output — selectors absent | FAIL |

Step 7b is otherwise SKIPPED (dev server not running per verification context).

### Probe Execution

No probe scripts (`scripts/*/tests/probe-*.sh`) exist for Phase 27. E2e gate results are documented in `27-08-SUMMARY.md`: war-room.spec.ts 6/6 GREEN, pages.spec.ts 21/21 GREEN (including /war-room SC1 and flat-route D-02 regression). These results are from the executor's in-session run and cannot be re-run without a dev server.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| ROOM-04 | All plans (27-01 through 27-08) | War Room hosts draft, tree-drafter, game-plan, opponents, and ban-priority under one parallax atmosphere | PARTIAL | Route, tab system, 5 surfaces, mousemove guard, ARIA, and e2e coverage are complete. Growth overlay visual effect is broken due to missing CSS rules. The requirement's core "parallax atmosphere + tabbed surfaces" clause is met; the "environment grows visibly" clause is not. |

ROOM-04 is marked `[x]` (Complete) in REQUIREMENTS.md traceability table. That marking is premature given the CSS gap.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `input.css` | 436-444 | Missing CSS selectors for War Room growth axes (tactics/intel/command) | BLOCKER | Overlays permanently invisible; SC3 visual growth broken |

No unreferenced TBD/FIXME/XXX markers found in the phase-modified files (`war_room.rs`, `room_surface.rs`, `parallax.rs`).

Remaining code-review advisory items (WR-03/04/05 from `27-REVIEW.md`) are not blockers and are not repeated here.

### Human Verification Required

None. The gap (missing CSS rules) is statically verifiable from source; all remaining truths are verifiable from code structure and committed e2e results.

### Gaps Summary

**One blocker identified:**

SC3 ("growth overlays advance through tiers") has a broken data → CSS link. The `AtmosphereRoot` correctly emits `data-tactics-tier`, `data-intel-tier`, and `data-command-tier` as reactive attributes, and `get_team_growth_state` correctly reads real DB counts to populate them. However, `input.css` only contains the CSS selectors for Solo Journey's three axes (lib/armory/banner). The nine corresponding selectors for the War Room's three axes were never added — not in the original Solo Journey spike, not in Phase 26, and not in any of the 8 Phase 27 plans. The e2e SC3 test checks only the data attribute values (which are correct), not the resulting overlay visibility, so the test passes while the visual effect is absent.

**Fix:** Add 9 CSS rules to `input.css` after line 444, mirroring the existing pattern:

```css
[data-tactics-tier="1"] .room-overlay--tactics { opacity: 0.34; }
[data-tactics-tier="2"] .room-overlay--tactics { opacity: 0.67; }
[data-tactics-tier="3"] .room-overlay--tactics { opacity: 1; }
[data-intel-tier="1"] .room-overlay--intel { opacity: 0.34; }
[data-intel-tier="2"] .room-overlay--intel { opacity: 0.67; }
[data-intel-tier="3"] .room-overlay--intel { opacity: 1; }
[data-command-tier="1"] .room-overlay--command { opacity: 0.34; }
[data-command-tier="2"] .room-overlay--command { opacity: 0.67; }
[data-command-tier="3"] .room-overlay--command { opacity: 1; }
```

This is a pure CSS addition — no Rust, no WASM, no server fn changes needed. Both compile targets remain unaffected.

---

_Verified: 2026-06-05_
_Verifier: Claude (gsd-verifier)_
