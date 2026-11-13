# M151 critique R0: T1 stills, marks re-check, prototypes (atelier, 2026-10-16)

Mode: critique. References: `references.md` (T1 card, Match Day field card, Lock-in, Post-game stamp, Mark grid spec, refs `map.png` / `map-grey.png` / `pressog.png`), `critique-marks.md`, `docs/DESIGN.md` (2.x accent rule line 202, Match Day line 421, chalk rule line 463, section 13 bans). I wrote the cards, not these builds: no self-approval conflict. This critique **amends the T1 card** (river, hatch, share crop, inset); the re-render of those amendments must go to a fresh-context critique, not to me.

## What I ran

- Opened every PNG named in the task (T1 stills + `-full`, marks contact 1280/390 at DPR 1/2, lock-in and stamp strips, rm and 800 ms frames), plus 4x to 10x crops of the sketch, the inset, the share strip and every 16 px mark at DPR 1 (`/tmp/m151r0/`, not committed).
- Contrast from rendered pixels (WCAG relative luminance, ground `page` rgb 8,20,21 or `ink-1` rgb 12,26,27): sketch ink 8.3:1 at 390 and up to 10.8:1 at 1280 (crossing pixels), chalk 14.5:1, tick labels `muted` 7.0:1, Match Day inset 8.0:1, loss L + outline 6.7:1. All pass A2.
- Visual mass at 1280 (luminance above ground, summed): headline 6,692 vs sketch 3,347. The headline holds on mass; the hatch still competes as *texture* (see concern 2).
- `rubric-check.mjs --app-dir gallery/surfaces` on all three surfaces (phone + desktop), served locally, `NODE_PATH=e2e/node_modules`:

| Row | landing | share | matchday | Note |
|---|---|---|---|---|
| V1 | PASS | PASS | PASS | tokens only |
| V2 | FIX* | FIX* | FIX* | *`.sr-only` 1 px, `.btn` 1 px transparent border, 1 px SVG strokes: hairlines, not spacing. Accepted as not-a-finding. |
| V3 | FIX* | PASS | FIX* | *heuristic counts source chars; rendered measure ~52 CPL. Accepted. |
| V4, A1, A2, A3, I7 | PASS | PASS | PASS | |
| I1, I2, I6, I8 | FIX | FIX | FIX | GUESS rows: app features (settings, undo) absent from a still. N/A here. |

- Not run: a 50 ms first-frame capture (the gallery injects the art after `fetch`; production must inline the SVG server-side, as the builder notes). Lock-in bulk-entry capture: **missing from the deliverables** (see C).

## Craft first (one sentence per question)

- **Could you name the shape?** At 390 and in the share strip: yes, "a square with an X" / "slashes". That is the core failure: the hatch runs at 45 degrees **parallel to the mid lane**, so mid dissolves into the hatch, and the straight double-line river reads as two more hatch lines. Only the bases and the open corners say "map".
- **Edge/stroke character vs reference?** One weight, one ink, open line ends: matches `map-grey.png` and `pressog.png`. But the builder's quadratic bows (<= 0.45 units) on frame, lanes and mid make the lines look mis-ruled (top lane drifts 3 px over 410 px at 1280), not hand-drawn: the worst of both. The card forbade faking a hand.
- **Occludes content?** No. Chalk label has a clearing; hatch never touches text.
- **Would a design director ship it?** Not yet: the skeleton is right, the hatch and river are wrong.

## Rulings on the orchestrator's concerns

1. **Mechanical vs one-stroke hand.** T1 promised a ruled survey schematic ("built from geometry ... never traced", "Do not: jitter or wobble to fake a hand", link-check calls it "one-stroke schematic"), not freehand. **Do not** move to a jittered, variable-width (1 to 1.5 px) hand generator: that breaks the one-weight rule and lands on the Excalidraw / "hand-drawn AI illustration" default (V7). The fix is cartographic, per swisstopo: **built things ruled, water free**. Straighten every lane and frame line (remove the bows); make the river the one curved feature. That gives the drawing its life and makes it read as the Rift, not an X.
2. **Hatch too heavy.** Yes, as texture: ~60 % cover of long 45-degree lines over 560 px. Fix with direction (stop paralleling mid) and length (below), which cuts line length about 2.5x.
3. **Match Day "both sides' marks facing".** My card asked for an 88 px thumbnail; the builder followed it. The phrase is not in `references.md` or on disk (if it is in the owner's brief, send me the line). Either way the render fails craft: at 88 px the 4-unit hatch is 3.5 px apart and goes to grey mush; the bases are 2.6 px dots. Fix below gives the two side shapes facing across the river.
4. **V5 at 390 ("League team" in one second).** FIX. "Team" is in the headline; "League" rests on recognising lanes-river-bases, which the parallel hatch currently hides. Re-judge after fixes T1-1 to T1-3. No copy change from me (naming the game in copy is a legal question: ask the librarian).
5. **Chalk count.** Landing 1280 and 390: **2** (Sign in, YOUR TEAM label + leader). Share 390: **1** (opponent label; no primary action on the page). Match Day 390: **1** (Read the brief). Landing's second chalk is card-sanctioned as "selection" (your team, surveyed) and sits on a diagonal away from the button at 1280: accept, but DESIGN line 202 ("not for decoration") must be amended to say so (flag below).

## Builder's calls

- **Chalk "RIVERSIDE ROOKS" on share:** allowed by the card, but in this crop it labels nothing (no base, no leader) and implies a side the data may not have. FIX (T1-5).
- **Match Day header title "Northgate Academy":** REJECT. Your own team name carries no information on your own Match Day. Revert to DESIGN 421: title "Scrim vs Riverside Rooks"; change the field-card sentence to "Best of 3, with 2 of 5 prep steps done." (no opponent, no format repeated in the header meta: meta becomes "Northgate Academy · Patch 16.20 since 11 Oct", the time lives only in the lead figure).
- **Quiet PrepSequence without chalk:** PASS. Builder work if T1 ships: `PrepEmphasis::Quiet` draws the next step's number box `border-outline` + `text-primary`, no `accent` (DESIGN 463).
- **Baron/Dragon spot marks at 1280 only:** PASS (muted, with words, read as survey annotations). Move them per T1-2 so they sit in the *curved* river's pits, 2 units off the bank.
- **Tower unused:** PASS. No tower on the sketch (would invite eleven more).

## A. T1 stills

| Still | V1 | V2 | V5 | V7 | V8 | A2 | A3 | A4 | I7 | Verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| landing 1280 | PASS | PASS* | PASS (headline leads) | PASS | FIX (bows; hatch parallel to mid) | PASS | PASS | PASS | PASS | **FIX** |
| landing 390 | PASS | PASS* | FIX | PASS | FIX | PASS | PASS | PASS | PASS | **FIX** |
| share 390 | PASS | PASS* | n/a | PASS | FIX (label labels nothing) | PASS | PASS | PASS | PASS | **FIX** |
| matchday 390 | PASS | PASS* | PASS (lead figure) | PASS | FIX (inset mush; header title) | PASS | PASS | PASS | PASS | **FIX** |

V7: no gradient, blur, emoji, glow, parchment, sepia, brush, compass rose, Inter or rounded-card kit; Familjen display. Clean.

### FIX list, `tools/build-t1-rift.py` (all in 100x100 units)

**T1-1 Straight built features.** Set every `wobbly(...)` bow to 0 (frame, both lanes, mid); replace the lane `Q` paths with straight `L` polylines: top `M10,84.5 L10,10 L83.5,10`, bot `M15.5,90 L90,90 L90,16.5`, mid `M15.2,84.8 L83.5,16.5`. Keep the open corners (frame stops 1.5 short) and the open ends at the bases. Remove the river's `j(0.1)` jitter.

**T1-2 Curved river (the one free line).** Centreline, point-symmetric about (50,50): `M14,14 C26,30 36,40 50,50 C64,60 74,70 86,86`. Banks: sample 48 points per cubic, offset +/-1.5 units along the normal, emit each bank as a polyline. Break both banks where they come within 2 units of the mid line (x + y = 100). Baron mark centre at (31,27), Dragon at (69,73) (in the bends, 2 units clear of the near bank); recompute `MARK_CLEAR` boxes around mark + word.

**T1-3 Hatch: perpendicular to its lane, shorter, sparser.** Left and right triangles: horizontal hatches; top and bottom: vertical. Spacing 6 units (was 4), positions 20, 26, ... 80 along the lane. For the left triangle at height y: `avail = 40 - |y - 50|`; hatch from x = 13 to x = 13 + 0.55 * (avail - 5), then clip to end >= 2 units before the curved bank and the mid line; drop any hatch shorter than 3 units; seeded jitter +/-0.6 units on the inner end only. Other triangles by rotation (right = point reflection through (50,50); top/bottom = swap x and y). Keep `LABEL_CLEAR`. Same stroke and ink as everything else. Expected: lit pixels from ~6.4 % to ~3 % at 1280.

**T1-4 Match Day inset (new `t1-inset.svg`, same generator, flag `--inset`).** 88 px, `ink-1` ground. No outer frame, no hatch, no labels. Lanes square 10..90 as T1-1; mid; river as a **single** centreline (T1-2 path, no banks: two banks 1.3 px apart merge). Bases enlarged: circle r 7 centre (14,86); square side 12 from (82,6) (centre 88,12); lanes and mid stop 3 units short of each base outline. These are the two side shapes facing across the river (ink only; no side colour, per card).

**T1-5 Share crop names a base.** Crop by the plan's draft side (`draft.our_side`; opponent is the other side):
- opponent red: viewBox `54 4.6 40 10.73`; chalk "RIVERSIDE ROOKS" (`eyebrow`, `accent`, page halo) right-aligned to x 82, baseline y 8; 1 px chalk leader (82.5,7.6) to (85,9).
- opponent blue: viewBox `6 84.67 40 10.73`; label left-aligned at x 17, baseline y 93.2; leader (16.5,92.4) to (14.1,90.1).
- no draft side: keep today's crossing crop with **no** label (card: no chalk unless it names the team the plan is about, and it may not invent a side).
Card amended accordingly.

**T1-6 Share title wrap.** "vs Riverside Rooks / - Game 1 plan" breaks before the hyphen. Display the seed separator as no-break space + en dash: `"vs Riverside Rooks – Game 1 plan"` (presentation only, at render).

**T1-7 Match Day header:** title "Scrim vs Riverside Rooks"; meta "Northgate Academy · Patch 16.20 since 11 Oct"; field-card sentence "Best of 3, with 2 of 5 prep steps done."

Re-capture all four stills (+ full) at DPR 1 **and** 2 and send to a fresh critique.

## B. Marks after the FIX pass

All eight FIX items from `critique-marks.md` landed as specified (paths verified in `public/marks/*.svg`; triangle keyline in `design_marks.rs` line 136; 24 px files hold 2 px via `stroke-width 1.33333`). At 16 px DPR 1: Bot now reads as a graduated scale, not RSS; Baron's solid core is crisp; Dragon's wave floats; Jungle inside the live area; Support in band. **PASS.** Watch items: Support's U + dot can read as a letter U beside "SUP"; Herald's 1.5 px points are the faintest element in the set (accepted earlier). Ship conditions unchanged: DESIGN 6.3/7 amendment in the same change; `alt=""` on any `<img>` use; never `text-accent` on a mark.

## C. Prototypes

**Lock-in** (strips 390/1280, 800 ms, rm). Motion matches the card: 220 ms scaleX from the slot edge, strong ease-out (bar ~70 % at 44 ms), mark settles 2 px, nothing else moves; 800 ms frame shows the bar in `outline`; reduced motion shows the end state at once; the `animation: none` reasoning in NOTES is correct. **FIX** two items:
1. **Truncation "Ori..." at 390** (present even at 0 ms, because the invisible mark already takes width). Below `sm` (640 px), the pick slot stacks: line 1 champion name `text-label` (truncate only as a last resort), line 2 `RoleLabel` at `text-caption` with the 16 px mark. Both lines fit the existing slot height (about 18 + 16 px inside the 44 px slot); verify "Orianna" and "Aurelion Sol" in full. From `sm` up, keep one line as in the 1280 strip.
2. **Missing verification:** capture the bulk-entry case (`data-locked="instant"`, a lock-in < 1 s after the last) as one frame at 0 ms showing the full-width `outline` bar and no chalk.

**Stamp** (strips, rm). Craft PASS: square, radius 0, Win `win` fill / `win-ink` W, Loss 1.5 px outline (6.7:1), no rotation or texture, side with its shape in the sentence. **Ruling on motion: static at first paint. Delete `stamp-in` / `stamp-letter` and `window.stamp()`.** "Drawn once" in the brief means one stamp per page, present once, never replayed; a 160 ms opacity entrance on page load is decorative load motion (the fade-in default) and gives the server-rendered first frame an invisible stamp. Lock-in remains the product's only signature motion. **PASS** once the entrance is removed.

## Verdict

| Item | Verdict |
|---|---|
| T1 landing 1280 | FIX (T1-1, T1-2, T1-3) |
| T1 landing 390 | FIX (same; V5 re-judge) |
| T1 share 390 | FIX (T1-5, T1-6) |
| T1 Match Day 390 | FIX (T1-4, T1-7) |
| Marks (nine) | PASS |
| Lock-in | FIX (stack below sm; bulk-entry frame) |
| Stamp | PASS, static; remove the entrance |

No REJECT: the T1 primitive (one-ink line schematic on geometry) is right; the hatch direction and the straight river are what make it an X.

## Staleness flags

- CONTRADICTION: docs/DESIGN.md line 202 ("chalk ... not for decoration") vs `references.md` T1 card (chalk team label on landing/share art): amend DESIGN to allow one chalk team label per artwork view, never on Match Day.
- STALE: `references.md` T1 card: river "two parallel lines" straight, hatch "45-degree", share crop "river crossing": superseded by T1-2, T1-3, T1-5 of this critique (card to be updated by the calling session; I do not rewrite it here).
- CONTRADICTION: `references.md` Match Day field card sentence ("Scrim vs Riverside Rooks. 3 of 5 ...") vs DESIGN 421 header title: resolved by T1-7 (header keeps the opponent; sentence drops it).
- CONTRADICTION (open): docs/DESIGN.md 6.3 and 7 ("no role icons") vs the shipped marks.
