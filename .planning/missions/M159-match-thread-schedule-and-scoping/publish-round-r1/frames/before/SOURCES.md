# Before frames: sources

"Before" is main at f86e1dc (the M152 merge). No frames were rendered from
f86e1dc itself. Each frame below is a copy of the closest existing frame of
the same surface from an earlier mission's strip; all source commits are
ancestors of f86e1dc. They show the surface as it was, not the exact M159
state (team, seed data and match setup differ).

| before frame | source | notes |
|---|---|---|
| match-day-upcoming-390.png | M152-identity-survey-sharpened-r1/publish-round-r1/frames/after/match-day-390.png | Match Day viewport at 390; no "Upcoming matches" list existed |
| match-day-upcoming-1280.png | M152-.../frames/after/match-day-1280.png | viewport frame (1280x800) |
| match-day-upcoming-1280-flow.png | M152-.../frames/after/match-day-1280.png | same file; M152 had no separate flow frame |
| match-day-prep-bo3-390.png | M152-.../frames/after/match-day-390-full.png | full page at 390; seeded Bo3 scrim, prep sequence |
| match-day-prep-bo3-1280.png | M152-.../frames/after/match-day-1280.png | viewport only; no full-page 1280 Match Day frame exists before M159 |
| draft-from-thread-{390,1280,1280-flow}.png | M152-.../frames/after/draft-{390,1280,1280-flow}.png | /draft with a saved draft open; opening a draft from a match thread did not exist |
| post-game-picker-{390,1280,1280-flow}.png | M141-team-companion-feedback-batch-1/publish-round-r1/frames/after/post-game-{390,1280,1280-flow}.png | review page before the M159 match picker; newest post-game frame on main |
| match-detail-review-{390,1280}.png | M141-team-companion-feedback-batch-1/publish-round-r1/frames/after/match-detail-{390,1280}.png | match detail before the review link; newest match-detail frame on main |

## Surfaces without a before frame

- The M159-specific states have no true before: the "Upcoming matches" list,
  a draft opened from a match thread, the post-game match picker and the
  review entry on match detail did not exist on main. The frames above show
  the same page, not the same feature.
- No 1280 full-page Match Day frame and no match-detail 1280-flow frame
  exist before M159; none was fabricated.

## Rendering true before frames (described, not built)

M148 did this: a helper branch `mission/M148-team-companion-first-patch-r2-before`
cut from main (9b7da51) carried only a frames spec; CI's e2e job ran it,
and the "chore(ci): e2e summary and frames" step copied
`$MISSION_FRAMES_DIR/before/*.png` to `frames/before/` (ci.yml, `copy_pngs ...
before`). Commit 363613a then copied them to the mission branch, renamed to
match the after names. For M159: branch `mission/M159-...-before` from
f86e1dc, add a frames spec that writes the five surfaces (with the M159
names, falling back to the nearest pre-M159 state) into
`$MISSION_FRAMES_DIR/before`, push, let CI run once, copy the PNGs here.
Cost: one CI run; no local build.
