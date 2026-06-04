---
phase: 26-room-system-foundations
plan: 02
subsystem: testing
tags: [docs, leptos, hydration, ssr, conventions]

# Dependency graph
requires:
  - phase: 18.2-pandemonium-hydration
    provides: the html/mod.rs:217 tachys hydration-panic incident this rule codifies
provides:
  - CLAUDE.md gotcha 58 — anti-structural-branching hydration rule
  - leptos-patterns.md rule 58 — same rule with 18.2-CONTEXT.md incident path + CORRECT/WRONG snippets
affects: [war-room, library, throne, 27, 28, 29]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Read InitialTheme/region once at room top level → pass as plain region: String prop; never read context inside reactive primitives"

key-files:
  created: []
  modified:
    - CLAUDE.md
    - .claude/rules/leptos-patterns.md

key-decisions:
  - "Numbered the rule 58 in both files for cross-reference consistency (per plan)"

patterns-established:
  - "Room-component context-read discipline documented before any room is built (D-12)"

requirements-completed: [ROOM-03]

# Metrics
duration: 4min
completed: 2026-06-04
---

# Phase 26 / Plan 02: Anti-Structural-Branching Hydration Rule Summary

**The anti-structural-branching hydration rule is now in both developer-facing docs (CLAUDE.md gotcha 58 + leptos-patterns.md rule 58), each referencing the Phase 18.2 html/mod.rs:217 tachys panic — so any agent building War Room/Library/Throne finds it first.**

## Performance

- **Duration:** ~4 min (inline execution)
- **Completed:** 2026-06-04
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- CLAUDE.md: numbered gotcha 58 in `## Critical Patterns and Gotchas` (after rule 41), with `18.2` and `html/mod.rs:217` references and a CORRECT/WRONG rust snippet, citing `solo_journey.rs:18`
- leptos-patterns.md: rule 58 under `## Reactivity` (after rule 55), with the explicit `18.2-CONTEXT.md` incident path and CORRECT/WRONG snippets

## Task Commits

1. **Task 1: Add rule to CLAUDE.md** - `931acb5` (docs)
2. **Task 2: Add rule to leptos-patterns.md** - `b845c8b` (docs)

## Files Created/Modified
- `CLAUDE.md` - Gotcha 58 (read region once at room top level, never inside reactive primitives)
- `.claude/rules/leptos-patterns.md` - Rule 58 with 18.2-CONTEXT.md path and worked CORRECT/WRONG examples

## Decisions Made
None beyond plan — rule text taken verbatim from 26-PATTERNS.md.

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None.

## Next Phase Readiness
- The hydration discipline is documented before any room component is built (D-12 satisfied).
- Verification: `grep "html/mod.rs:217" CLAUDE.md` → 2; `grep "18.2" CLAUDE.md` → 1; `grep "html/mod.rs:217" .claude/rules/leptos-patterns.md` → 2; `grep "18.2-CONTEXT.md" .claude/rules/leptos-patterns.md` → 1.

## Self-Check: PASSED
- Both files contain the rule with the required `18.2` / `html/mod.rs:217` references and remain valid markdown.

---
*Phase: 26-room-system-foundations*
*Completed: 2026-06-04*
