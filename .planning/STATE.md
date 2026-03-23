---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Polish, Draft & Opponents Rework
status: Ready to execute
stopped_at: Completed 10-01-PLAN.md
last_updated: "2026-03-23T12:34:43.416Z"
progress:
  total_phases: 6
  completed_phases: 4
  total_plans: 13
  completed_plans: 11
---

## Current Position

Phase: 10 (opponents-redesign) — EXECUTING
Plan: 2 of 3

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-19)

**Core value:** Features talk to each other — stats inform drafts, drafts flow into game plans, post-game lessons surface when preparing the next game.
**Current focus:** Phase 10 — opponents-redesign

## Decisions

- v1.1 scoped to bugs, UX polish, draft/opponents rework, and testing infrastructure
- Solo player mode, match detail expansion, and remaining INTL items deferred to v2.0
- PLAN-02 assigned to Phase 6 (same underlying fix as BUG-02: game plan navigation)
- TEST-02 (Playwright CLI workflow) moved to Phase 6 so browser verification is available for all subsequent phases
- TEST-01 (seed real test data) stays in Phase 11 — needs features built first before seeding
- Phase 8 isolated for champion pool overhaul + draft role icons (substantial UI rework)
- Phase 9 (Draft & Game Plan) depends on Phase 8 so role assignment is available during draft work
- Opponents redesign (Phase 10) depends on Phase 9 so opponent dropdown in draft is built first
- Testing (Phase 11) runs last — needs built features to seed data against
- [Phase 06]: Used agent-browser npm package (Vercel Labs) for browser verification skill instead of Playwright MCP
- [Phase 06-bug-fixes]: Slot-to-slot drag: clear source before fill_slot to bypass already_used guard
- [Phase 06-bug-fixes]: Use plan_id query param for View Game Plan link (draft_id already serves prefill)
- [Phase 06-bug-fixes]: BUG-05 regression test uses soft assertion + WASM-panic check due to Playwright headless DragEvent limitation
- [Phase 07-ux-polish]: format_timestamp_with_now takes explicit now for deterministic unit tests; public format_timestamp calls it with Utc::now()
- [Phase 07-02]: Bench card watermark uses inner overflow-hidden div to avoid clipping the role select dropdown
- [Phase 07-02]: Coach watermark uses inline clipboard SVG (no Community Dragon icon available for coach)
- [Phase 08-02]: serde(default) on DraftAction.role ensures backward compatibility with saved drafts missing the role key
- [Phase 08-02]: Role badge uses dashed border + 50% opacity for auto-guessed, solid accent border for user-confirmed
- [Phase 08-champion-pool-role-ui]: Champion pool card grid: 3-column grid with aspect-square portraits, drag-and-drop tiers via DataTransfer, matchup autocomplete with 'vs {name}' auto-fill
- [Phase 09]: Opponent autocomplete stores ID in Draft.opponent with legacy free-text fallback (D-05)
- [Phase 09]: Intel sidebar opponent select removed (D-06) — header dropdown is single source of truth
- [Phase 09]: Pool notes query uses champion name (not ID) matching since champion_note stores by name; collapsed_note_types defaults to all except matchup
- [Phase 09-draft-game-plan]: most_common_tag tiebreak: first occurrence in list (D-17), not alphabetical or random
- [Phase 09-draft-game-plan]: Role strategy prefill: only our side pick actions; first pick per role wins
- [Phase 10-opponents-redesign]: Two-step create_opponent_with_players chosen over single-transaction subquery to avoid name-based lookup ambiguity
- [Phase 10-opponents-redesign]: get_opponent query uses <string>last_fetched cast for datetime-to-String deserialization (consistent with created_at pattern)

## Blockers/Concerns

- BUG-02 and PLAN-02 describe the same underlying fix (game plan navigation); both assigned to Phase 6 to consolidate the work

## Session Continuity

Last session: 2026-03-23T12:34:43.414Z
Stopped at: Completed 10-01-PLAN.md
Resume file: None
