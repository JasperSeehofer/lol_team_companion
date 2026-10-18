# M149 R2 atelier critique: recipe cards (intake round)

Mode: critique, from code and existing frames (no browser run, so every verdict is code-read, not render-read).
Tokens: `input.css` `@theme` only (direction "Survey"). Contrast computed from the OKLCH values
(OKLCH to linear sRGB, WCAG ratio); the verifier rechecks on rendered pixels.
This run had no subagent tool; I wrote this one file myself and touched no code.

## Verdicts on current surfaces

| Surface | Verdict | Why |
|---|---|---|
| Match detail markers at 390 | FIX | Hit box is `p-2` around a 12-16 px glyph = 28-32 px, and neighbours 12 px apart overlap. Below the 44 px phone rule (`ButtonSize::Touch`). |
| Match detail at 1280 | PASS | `md:p-1.5` mouse targets; lanes (`assign_lanes`, `LANE_GAP_PCT`) keep shapes apart. Leave unchanged. |
| Disabled "Sync my games" | FIX | `disabled:opacity-50` on Primary blends chalk and accent-ink with `page`: 4.11:1 computed (4.15 reported). |
| Feedback attachment floor | PASS as is; area picker is new (card B) | File and paste path is plain and honest. |
| Vision rail caption | FIX | Says "none are drawn"; A wants the inference and the map pointer. |

## Card C: late-game markers, 44 px targets (phone only)

1. Technique: a "neighbourhood row", not bigger hit boxes. On a 12 px pitch no padding or merged/Voronoi area can reach
   44 px (a merged area is at most the pitch). The row is the 44 px path; the track stays a picture.
2. Where: below `md` only (`md:hidden`), between the last rail and `TimelineScrub`. From `md` up nothing is added and
   the track markup is unchanged, so 1280 density is identical.
3. Content: up to 5 shown events nearest the playhead, in time order, same filter as `step()` (only kinds whose
   toggle is on). Each is `<button type="button">` 44 x 44 px (`size-11`), `rounded-ctl border border-outline`,
   with the marker's own `<svg>` at `size-4`, side colour `text-side-blue|text-side-red`, its clock beneath in
   `figures text-caption text-secondary` (row `h-14`; the button stays 44). `gap-2`: 5 x 44 + 4 x 8 = 252 px,
   fits 390 minus page padding. Container `role="group"`, `aria-label="Events near the playhead"`.
4. States: default transparent; hover `bg-ink-3`; the `selected` event is `aria-pressed="true"` (`border-secondary
   bg-ink-3`, same as Button `pressed`); "you were involved" keeps the chalk stroke (`stroke-accent`), the
   track's own grammar. Focus: the global 2 px chalk `--focus-ring`.
5. Behaviour: click runs the track marker's handler (`selected` toggle, `playhead.set(at)`); the row re-centres on the new
   playhead instantly (no animation). Previous / Next event (`ButtonSize::Touch`) are untouched and step the full ordered
   list, so every marker stays reachable even when the row shows five. Events sharing a second all appear, ordered
   `(timestamp_ms, idx)`, never merged.
6. Tab stops: below `md` the track markers get `tabindex="-1"` (taps still work for coarse picks) so a keyboard or
   switch user meets five row stops instead of dozens; from `md` up the track markers stay the tab stops.
7. A11y: each name is the existing `event_tooltip` text (+ ", you were involved"); the `data-near-playhead` live
   sentence is unchanged; targets 44 px (above WCAG 2.5.8's 24).
8. Verify: Playwright at 390 on a seeded late-game match: every row button box >= 44 x 44; Next from the last
   shown reaches the final event; 1280 matches the `visual-regression.spec.ts` baseline.

## Card D: disabled "Sync my games" at >= 4.5:1

1. Cause: `BASE` has `disabled:opacity-50`; Primary then blends `accent` and `accent-ink` with `page` (4.11:1).
   Opacity-faded text is the pattern CLAUDE.md bans.
2. Fix in `ButtonVariant::Primary` (benefits every disabled Primary): add `disabled:bg-ink-3 disabled:border-outline
   disabled:text-muted disabled:opacity-100` (opacity override for Primary only; keep `disabled:cursor-not-allowed`;
   hover/active must not apply when disabled: `enabled:hover:` or the existing disabled pointer behaviour).
3. Tokens and ratios: `text-muted` oklch(0.700 0.014 200) on `bg-ink-3` oklch(0.275 0.020 200) = 5.54:1.
   On `bg-ink-2` 6.24:1; `text-secondary` on `ink-3` 8.47:1 (too loud, reads enabled). Choose muted on ink-3.
   `border-outline` (3:1 token) keeps the shape visible; loss of chalk fill is what signals "off".
4. The reason it is off (no Riot link / Riot not configured) must be visible text near the button or
   `aria-describedby`; builder checks /me already has one, else adds a `text-caption text-muted` line.
5. Out of scope: Secondary, Quiet, Danger disabled still use 50% opacity; add them to the `/design` contrast board
   and measure before the next round.

## Card B: desktop "Select an area of the page"

1. Gate: render only when `navigator.mediaDevices.getDisplayMedia` exists AND `(min-width: 768px)` (`md`) AND
   `(pointer: fine)`. Decided in an Effect after hydration; SSR renders nothing (no flash, no dead control).
2. Placement: in the attachment floor, first: `Button variant=Secondary` "Select an area of the page", then
   "Attach an image", then the "or paste" caption. Floor hint gains: "A captured area stays in your browser until you send."
3. Flow: click closes the dialog (state kept), the browser's own share prompt appears (`preferCurrentTab: true`);
   one frame is drawn to a canvas and the stream is stopped at once (no recording); then the overlay mounts.
4. Overlay: `fixed inset-0`, above the dialog scrim, `cursor-crosshair`, the captured still drawn full-viewport on a
   `<canvas>` so the page cannot move under the drag. Outside the selection a flat `bg-page` layer at 60% alpha
   (four edge rectangles; no blur, no gradient: lint). Selection `border-2 border-accent` (chalk = the viewer's
   mark), no fill, 12 px square handles `bg-accent`. Readout `figures text-caption bg-ink-2 text-primary px-2`:
   "640 x 360". Selections under 24 x 24 px are ignored.
5. Keyboard (no pointer needed): overlay is `role="dialog" aria-modal`, label "Select an area of the page",
   description "Drag to select, or use the arrow keys." Start: centred rectangle, 50% of the viewport. Arrows move
   8 px (Shift 1 px); Alt+Arrows resize 8 px (Alt+Shift 1 px); Tab cycles "Use this area" (Primary,
   `ButtonSize::Touch`) and "Cancel" (Quiet) in a bar, bottom centre, `bg-ink-2 border border-rule`. Enter confirms,
   Esc cancels. A polite live region reads size and position, debounced.
6. Confirm: crop to a PNG blob, hand to the existing `take_file` path (same type/size checks, same preview row:
   64 x 40 thumb, size, Remove); overlay unmounts, dialog reopens (`showModal`), focus lands on Remove.
7. Cancel / deny / failure: overlay unmounts, dialog reopens, focus on "Select an area of the page", and
   `attach_error` (the Field error slot) says: "Screen capture was cancelled. Nothing was attached." /
   "The browser did not allow screen capture. You can still attach an image or paste one." /
   "Could not capture this page. Attach an image instead."
8. Motion: none, so no reduced-motion case. 
9. A11y line for the doc comment: native buttons; the overlay is a modal dialog with Esc and Enter and a full
   keyboard alternative to dragging; size is announced politely; focus returns where it came from; the capture
   stays in the browser until the reporter sends.
10. Verify: Playwright stubs `getDisplayMedia` with a canvas stream: hidden at 390 and when absent; drag crop and
    keyboard crop give the expected size; Esc; denied message. No frame strip needed (no motion).

## Card A: map deferred, notes on the timeline

1. Style: the `note` style (serif italic) means "a person on the team wrote this" (DESIGN section 4; `Annotation`,
   canvases). These are system sentences, so `text-caption text-muted`, the existing `rail.caption` grammar.
2. Vision rail caption (replaces "Riot's game data reports no recalls, so none are drawn" once recalls are drawn
   from purchases): "Recalls are inferred from item purchases at the fountain; Riot reports no recall event."
   Inferred markers take a dashed outline (same rule as `RoleLabel guessed`; add "inferred = dashed outline" to
   DESIGN 6.3) and their tooltip starts "Inferred recall". Until inference ships, keep today's caption.
3. Map pointer: one line after the legend list, not inside a rail: "Map view comes with the Rift Board." No link
   (no destination; honesty rule). Remove when the board ships. Not shown on `/s/` shares.
4. At 390 both lines wrap, never truncate. A11y: plain text in reading order, no live region.

## Staleness flags

- STALE: `src/pages/match_detail.rs` RAILS comment and the Vision caption once recall inference lands.
- STALE: `src/components/ui/timeline.rs` header ("reserved for M4", "no playback") vs its use on match detail.
- STALE: `docs/DESIGN.md` Button row "Targets at least 24px" vs the 44 px phone rule; state the phone rule for timeline markers in 6.3.
