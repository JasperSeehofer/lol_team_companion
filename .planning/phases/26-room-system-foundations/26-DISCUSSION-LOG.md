# Phase 26: Room System Foundations - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-04
**Phase:** 26-room-system-foundations
**Areas discussed:** RoomSurface boundary, Team growth mapping, player_match in team growth, Pause-atmosphere toggle

---

## RoomSurface boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Build contract in 26, prove in 27 | Phase 26 ships RoomSurface as a standalone reusable component (signal + `<Show>`, ARIA tablist, deep-link `?tab=`) with a minimal render test using dummy surfaces; War Room is first real consumer. | ✓ |
| Defer fully to War Room (27) | Build RoomSurface inside War Room where it has a real consumer, then extract; ROOM-02 spans two phases. | |
| Build skeleton + ?tab= in 26 | Structural skeleton + deep-link only in 26; styling/polish in War Room. | |

**User's choice:** Build contract in 26, prove in 27.
**Notes:** Satisfies ROOM-02 in full within Phase 26. Accepts the risk of designing the API without a concrete consumer in exchange for a locked contract from day one.

---

## Team growth mapping (shape)

| Option | Description | Selected |
|--------|-------------|----------|
| Raw counts + War Room derivation | `TeamGrowthCounts` + `get_team_growth_counts` + War Room axis tiers (tactics/intel/command) as the proven template; Library/Throne derivations in their own phases. | ✓ |
| Raw counts only | Only the raw count round-trip; every room derives its own axis tiers. | |
| Full 9-axis TeamGrowthState now | Derive all nine axis tiers up front. | |

**User's choice:** Raw counts + War Room derivation.
**Notes:** Keeps Phase 26 testable and avoids committing thresholds/mappings for rooms not yet built (which research says should calibrate against real data).

### War Room axis → data mapping (follow-up)

| Option | Description | Selected |
|--------|-------------|----------|
| tactics=draft, intel=opponent, command=ban_priority | Direct 1:1 with the three most War-Room-relevant tables. | |
| tactics=draft+tree, intel=opponent, command=ban_priority+learnings | Broaden tactics to include tree-drafter; command folds in post-game learnings. | ✓ |

**User's choice:** tactics = `draft`+`draft_tree`, intel = `opponent`, command = `ban_priority`+`post_game_learning`.
**Notes:** All five tables confirmed team-scoped (`team: record<team>`) against `schema.surql`.

---

## player_match in team growth

| Option | Description | Selected |
|--------|-------------|----------|
| Drop it from Phase 26 | No Phase 26 derivation consumes it; it needs a `team_member→user→player_match` join. Add later if a Library/Throne axis needs match volume. | ✓ |
| Include via member join now | Sum player_match across team members so the field exists for future axes. | |

**User's choice:** Drop it from Phase 26.
**Notes:** Established the rule — raw team counts cover directly-team-scoped tables; join-requiring tables are deferred until an axis consumes them.

---

## Pause-atmosphere toggle

| Option | Description | Selected |
|--------|-------------|----------|
| Defer the toggle | Phase 26 ships only the required runtime MQL change-listener (A11Y-01); in-app toggle is a deferred idea. | ✓ |
| Include the toggle now | Add an in-app "Pause atmosphere" control alongside the listener. | |

**User's choice:** Defer the toggle.
**Notes:** Keeps Phase 26 to its "no new visible UI" boundary; the toggle belongs in a later a11y/settings phase.

---

## Claude's Discretion

- Exact tier-threshold integers for War Room axes (sane starting values; flagged for beta calibration).
- Internal naming of the team-scoped growth structs / War Room derivation module location (keep parallel to existing `growth.rs`).
- `?tab=` deep-link fallback for unknown/invalid tab values (default to first surface).

## Deferred Ideas

- In-app "Pause atmosphere" toggle → later a11y/settings phase.
- `player_match` team aggregation (member join) → when a Library/Throne axis needs it.
- Library (`records`/`tomes`/`analysis`) + Throne (`roster`/`progression`/`crown`) axis derivations + thresholds → Phases 28/29.
- Mousemove listener accumulation guard (`AtomicBool`, Leptos #3665) → Phase 27 (already in STATE.md deferred items).
