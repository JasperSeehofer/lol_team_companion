---
status: partial
phase: 28-library-ia-healing
source: [28-VERIFICATION.md]
started: 2026-06-06T12:14:10Z
updated: 2026-06-06T12:14:10Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Tab-switch atmosphere stability
expected: Navigating between the five Library tabs (Stats, Match Detail, Analytics, Post-Game, Personal Learnings) at `/library` does NOT remount or flash the parallax background atmosphere — the background stays continuously mounted while only the foreground surface content swaps (D-05 keep-mounted panels).
result: [pending]

### 2. /personal-learnings/new preserved (not redirected)
expected: Visiting `/personal-learnings/new` renders the "new personal learning" form directly and does NOT redirect to `/library` (D-10/D-11 — only the bare `/personal-learnings` flat route redirects into the Library learnings surface). Automated coverage: present in `pages.spec.ts` smoke (SC3, passing).
result: [pending]

### 3. Growth overlays visibly fade in on a seeded team
expected: On a team that has at least one post-game review and/or team notebook entry saved, the corresponding Library growth overlay(s) (`.room-overlay--tomes` / `.room-overlay--analysis`) are faintly visible (computed opacity > 0) at `/library`; a fresh teamless user shows all overlays fully transparent. Automated coverage: `library.spec.ts` seeds tomes+analysis and asserts opacity > 0 (passing). Records axis is unit-tested in `db.rs` (no Riot-API-free e2e seed path for real match syncs).
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
