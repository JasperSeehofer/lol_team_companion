# Before frames (proxies)

Review together is a new surface; these are the closest existing frames of the
surfaces it touches, copied unchanged from M159's published round
(`.planning/missions/M159-match-thread-schedule-and-scoping/publish-round-r1/frames/after/`,
last changed in commit `6840b13`). They were captured by M159's CI run, not re-shot on
this branch's base.

| File | Source | Stands in for |
|------|--------|---------------|
| `match-day-prep-bo3-1280.png`, `match-day-prep-bo3-390.png` | M159 `match-thread.spec.ts`, "earlier work is offered" | The match thread's review step before "Review together" (one link, "Write the review"; after: `match-day-review-together-*`) |
| `match-detail-review-1280.png`, `match-detail-review-390.png` | M159 `match-thread.spec.ts`, "a team game is chosen from the match page" | The match page's review button (unchanged by M165; kept as the review entry point's before) |
| `post-game-picker-1280.png`, `post-game-picker-1280-flow.png`, `post-game-picker-390.png` | M159 `match-thread.spec.ts`, review picker | The review page before the "Reviewed together" section (after: `review-together-review-*`) |

## Gaps (no before exists)

- The session page `/review/together`: silent round (phone and desktop), facilitator side panel
  (presence, "N of 5", round controls, quiet nudge), reveal and its frame sequence, decide.
- "Reviewed together" on a generated review, with its voices line.
- The bo3 review-step frame shows a series readout with a review written alone; the after frame
  shows a Bo5 thread whose game 1 has no session yet. Same component (`PrepSequence`), different state.
