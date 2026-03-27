---
status: partial
phase: 14-personal-learnings-journal
source: [14-VERIFICATION.md]
started: "2026-03-27T16:45:00Z"
updated: "2026-03-27T16:45:00Z"
---

## Current Test

[awaiting human testing]

## Tests

### 1. Browse page empty state
expected: Navigate to /personal-learnings while authenticated — filter bar with type/champion/tag/sort dropdowns visible; "No learnings yet" heading and "Write your first learning" CTA link
result: [pending]

### 2. Form page initial state
expected: Navigate to /personal-learnings/new — type selector shows 3 buttons (General/Champion/Matchup); three labeled text areas; 8 tag chips from LEARNING_TAGS
result: [pending]

### 3. Create flow
expected: Fill all three required sections, save — "Learning saved." toast appears; redirect to /personal-learnings; card appears in grid
result: [pending]

### 4. Card inline expansion and edit/delete
expected: Click card to expand showing all three sections with headings; edit link goes to ?edit={id} with fields pre-populated; delete shows "Learning deleted." toast and removes card
result: [pending]

### 5. Filter bar
expected: Change type/champion/tag dropdowns — card grid filters client-side without page reload; "No learnings match these filters." shown when no match
result: [pending]

### 6. Sort by champion
expected: Switch sort dropdown to "By champion" — cards regrouped into champion-labeled sections with section headers
result: [pending]

### 7. Scoreboard "Add Learning" button
expected: On match detail page, button navigates to /personal-learnings/new with champion, opponent, match_id, and result pre-filled
result: [pending]

### 8. Timeline event "Add Learning from this event" link
expected: In match detail event panel, link navigates to /personal-learnings/new with all 7 params (champion, opponent, match_id, result, event_ts, event_name, tag_hint) pre-filled
result: [pending]

### 9. Nav link visibility
expected: Authenticated: "Learnings" nav link visible, navigates to /personal-learnings. Unauthenticated: link absent.
result: [pending]

## Summary

total: 9
passed: 0
issues: 0
pending: 9
skipped: 0
blocked: 0

## Gaps
