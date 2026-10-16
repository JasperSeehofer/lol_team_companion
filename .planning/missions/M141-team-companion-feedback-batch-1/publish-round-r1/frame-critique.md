# M141 frame critique (after frames, CI run 38881767825)

Method: every after frame (390, 1280, flow, hover) viewed; before frames used for comparison; strips in `strips/`.
Harness: `rubric-check.mjs` requires `--app-dir` and drives a live page; not run on stills (skipped, no server). Contrast
from pixels was therefore not recomputed (A2 judged by eye plus token use only). Button height at 390 measured from pixels.
Not frame-provable (hover, drag, Enter, status change, failure) are listed per surface as "not frame-provable".

| Surface | Verdict | Cards |
|---|---|---|
| match-detail | PASS (one note) | R-A1 A2 A3 A4 A5 |
| game-plan | PASS (state not shown) | R-B1 R-E1? |
| champion-pool | PASS | R-C1 R-C2 R-D? |
| tree-drafter | PASS (thin) | R-D1 |
| post-game | FIX | R-E1 |
| me | FIX (minor) | R-F1 R-F2 R-F3 |
| admin-invites | FIX | R-G1 |
| bugs-mine | PASS (empty only) | R-H3 |
| admin-bugs | PASS | R-H3 |
| admin-proposals | PASS (dialog not shown) | R-H4 |
| feedback-widget | PASS | R-H1 R-H2 |

## match-detail  (match-detail-{390,1280,1280-flow}.png, match-detail-hover-*)
- R-A1 PASS: 1280 shows blue then red stacked, identical column grid and row heights, no scrollbar, role rows align
  (TOP..SUP both). 390: tables fit at 390 with no horizontal scroll, but Items/Damage/Gold/Vision are dropped at that
  width (only Pos, Player, K/D/A). Acceptable as a no-scroll answer; note it as a decision, not a defect.
- R-A2 PASS: hover frame (match-detail-hover-1280) tints MID in blue and MID in red together; default frame has BOT
  tinted in both (leftover pointer). Tap/focus at 390 not frame-provable (e2e match-detail spec).
- R-A3 PASS: "you" is a chalk outline on the Da avatar, no "(you)" text; K/D/A single line incl. "5 / 4 / 12" at 1280 and
  390.
- R-A4 PASS at 1280: three rails (Objectives, Fights, Vision) with own toggles, no overlapping markers except the
  deliberate teamfight hex cluster at ~18:00 (a stacked second lane keeps it legible). 390: rails fit; objective and fight
  markers cluster near the right end (about 12 px apart), so tap targets there are likely under A1's 24 px; the
  per-event Previous/Next buttons are the A4 equivalent. Toggle-off behaviour not frame-provable.
- R-A5 PASS: no Recall marker; caption "Wards: yours only. Riot's game data reports no recalls, so none are drawn."
- Item slots render as empty dashed boxes because the seeded match has no items; the "Items" column is therefore
  unverified for icons. Not a defect of the code.
- Before frame is the stats fallback only (no match detail existed), so no before/after comparison is possible.

## game-plan  (game-plan-1280.png, -390, -flow)
- R-B1 PASS: every blank Matchup row shows the dashed icon slot beside the "Champion" input (all ten), per the card's
  blank-row check. The "ten typed picks show ten icons" check is not frame-provable here (no picks typed): covered only
  if the e2e spec types them; ask the builder to confirm.

## champion-pool  (champion-pool-*.png)
- R-C1/R-C2: tier rows, chips with avatars, key legend, "Enter adds the highlighted champion" all present at 390 and
  1280. Scroll-keeps-on-drop, Saving.../Saved and failure state: not frame-provable (needs drag + action failure);
  covered by the champion pool e2e spec. Layout PASS; the 390 stack is clean, no clipping.

## tree-drafter  (tree-drafter-*.png)
- R-D1: Enter submits "New branch": not frame-provable (needs an open tree; frames show the empty state with "No tree
  open" and "Create tree"). Empty state is good copy; nothing to fault. Covered by e2e only.

## post-game  (post-game-1280.png)
- R-E1: the "Filled from match" badge and note are not visible (Match = "No match"); not frame-provable from this frame.
- FIX (V2): left review list: the first row "Review without a plan" has its W badge indented ~33 px (x=59) while the
  other rows start at x=26; rows with no plan indent a missing glyph. Make the leading result glyph a fixed-width
  slot (`w-5 shrink-0`) in every row, and the row text starts at one x.
- FIX (minor, V2): "Outcome" control and "How well did the plan work?" Select share a row but the 8 px label gap
  differs from the Match row (label y 536 vs control 553/551): align the control tops (both 551), the Select is 2 px
  higher than the segmented control.

## me  (me-1280.png, me-390.png)
- R-F2 PASS (frame): sortable headers carry a diamond glyph, Games shows the descending glyph; keyboard Tab+Enter not
  frame-provable (e2e).
- R-F3 PASS: LP history empty state states the source: "Each sync stores your Solo/Duo standing as one point; choose
  All to see every sync so far." Only the 0-snapshot frame exists; the 1 and many states are not shown.
- R-F1: "Sync my games" is disabled (no Riot link), muted olive on olive; idle vs loading size comparison not
  provable from one frame. FIX (A2, low): disabled state reads at roughly 3:1 or less; fine if exempt as disabled, but a
  link-first CTA ("Link it on your profile") already exists. Keep; confirm contrast >=3:1 by pixel script.
- 390: Recent games and Champion trends tables are clipped on the right (Damage, CS/Min cut at the edge) with no scroll
  cue. FIX (A4, med): at 390 hide CS and Damage columns in Recent games and Damage/KDA in Champion trends, or add the
  `Table` fade-edge affordance. Same issue seen on admin-invites 390.

## admin-invites  (admin-invites-1280.png, -390)
- R-G1 PASS for the Mint form: Max uses, Expires, Team, Note and Mint code sit on one baseline, hints sit under their
  own fields only ("Empty: never", "Joins the account to this team").
- FIX (high, 1280): the "Disable" buttons are cut off at the table's right edge (button border clipped at x=1120);
  table is wider than its container. Give the last column `w-24`, put the Note column on `truncate max-w-[12rem]`, and set
  Expires and Created to `whitespace-nowrap` (Expires currently wraps "2026-/10-19").
- FIX (med, 1280/390): invite Code wraps to three lines ("R2RL-/KHF9-/RGKU"). Add `whitespace-nowrap` and
  `font-mono text-label` to the code cell, or widen Code to `w-44`. At 390 the table clips at Team with no cue.

## bugs-mine  (bugs-mine-1280.png)
- Only the empty state is captured (0 reports); the status timeline (R-H3) is not frame-provable. Empty-state copy,
  tab counts and legend line are clear. No fault.

## admin-bugs  (admin-bugs-1280.png)
- List with type, status and date reads cleanly; segmented filter fits. R-H3's reporter-side timeline not applicable
  here. PASS.

## admin-proposals  (admin-proposals-1280.png)
- R-H4: "Import proposals (JSON)" button is the secondary and "New proposal" beside it; the import dialog, result callout
  and the two draft rows are not in the frame; not frame-provable here (ask for the import dialog frame in the next
  capture or rely on the e2e spec). The list shows "Not published" badges with a diamond glyph plus text (not colour
  only): good. Badge density is high (two badges and a vote count per row); acceptable for an admin list.

## feedback-widget  (feedback-widget-1280.png, -390)
- R-H1 PASS: dialog titled "Give Feedback" with a Bug / Idea / Praise segmented control and the footer button
  measured at 44 px high at 390 (pixels 2616 to 2659).
- R-H2 PASS (floor): "Attach an image" button plus "or paste an image (Ctrl+V)", file-type and size line, privacy line
  and "Send feedback" primary. Attachment-with-thumbnail, region capture overlay and the failure callout are not
  frame-provable from these frames.
- Frame artefact (not a defect): the full-page screenshot puts the scrim over only the top 987 px at 1280 and the dialog
  floats above the page at 390; the dialog itself is correct.

## Signature moment
The match-detail page now has one: the stacked mirrored scoreboards plus the three-rail timeline with the shared chalk
playhead. Show that first.

## FIX list
1. admin-invites 1280: Disable button clipped; wrapping Expires and Code (table widths). High.
2. post-game 1280: review list glyph slot not fixed width (first row indented); Select vs segmented control tops.
3. me 390 (and admin-invites 390): tables clipped with no scroll cue or column reduction.
4. match-detail 390: confirm marker tap size in the clustered right end (A1); measure with the harness live.
5. me 1280: confirm disabled "Sync my games" contrast by pixel script.

## Re-check (run 38892154457)

Capture: CI after frames only (no live browser); strips rebuilt as before/after pairs in strips/ (admin-invites, post-game, me; 1280 and 390). Disabled-Sync contrast computed from rendered pixels of me-1280.png.

| Surface | Verdict | Evidence |
|---|---|---|
| admin-invites 1280 | PASS | Table fits the container; Disable buttons whole (x 1024-1091, inside the 1120 edge); Code on one line in mono; Expires on one line; Team, Created-by and Note truncate with an ellipsis. |
| admin-invites 390 | PASS (note) | Columns reduced to Code, Status, Uses and Disable; no clipped cell. Disable sits flush on the 16 px gutter edge, acceptable. |
| post-game 1280 | PASS | Review list glyph slot is now fixed width, so all three rows align. Outcome control and "How well did the plan work?" Select share one top (y 551). |
| post-game 390 | PASS | Stacked cleanly; list, Outcome and Select aligned; full-width Save/New buttons fine. |
| me 1280 | PASS | All columns present and aligned; no clipping. |
| me 390 | PASS | Tables drop to Result/Champion/KDA/Game and Champion/Games/Win rate; nothing clipped, no horizontal scroll. |
| Regression sweep (Button full-width label span) | PASS | Mint code at 390 centred on a full-width button; game-plan, tree-drafter, champion-pool, bugs, feedback-widget, admin-bugs and admin-proposals frames show no stretched or misaligned labels. |
| Item 2d, match-detail markers at 390 | Agree: not a bug | Clustering comes from seed data; Previous/Next buttons reach every marker. A live tap-size measurement stays an e2e job. |
| Item 5, disabled "Sync my games" | PASS (note) | Rendered bg rgb(120,126,87), text rgb(20,26,17): 4.15:1. Below 4.5:1 but it is a disabled control (WCAG exempts it), and the page callout states why. Optional polish: nudge the label darker to reach 4.5:1. |

Overall: PASS. All three FIX items from round 1 are closed and no regression was found in the frames.
