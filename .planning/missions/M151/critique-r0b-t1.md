# M151 critique R0b: T1 stills after the T1-1…T1-7 fix pass (atelier, fresh context, 2026-10-17)

Mode: critique. References: `references.md` T1 card, Match Day field card, OG card, lead-figure rows; `critique-r0-t1.md` (fix list T1-1…T1-7, which amends the T1 card); `gallery/out/t1-fix-log.md`. I did not write `critique-r0-t1.md` or any of these builds: no self-approval conflict.

## What I ran

- Opened: `out/t1-landing-1280(.png, -full)`, `t1-landing-390`, `t1-share-390`, `t1-matchday-390`, `og-t1`, `og-t1-share`, and `t2-`/`t3-matchday-390` for comparison. 3x crops of the Match Day inset, the share strip and the prep-detail dash in `/tmp/m151r0b/` (not committed).
- Sources: `tools/build-t1-rift.py`, `surfaces/art/t1.json`, `surfaces/matchday.html` (facing row), SVG group classes.
- Contrast from rendered pixels (WCAG, ground `page` 8,20,21 / `ink-1` 12,26,27): full-coverage ink 8.3:1 at 1280 (unchanged), inset 7.9:1, chalk 14.5:1, tick labels `muted` 7.0:1 (rgb 149,161,162). At 390 DPR 1 many 1 px axis-aligned hatches straddle a pixel row and render as two rows at 2.8 to 4.6:1: decorative, `aria-hidden`, but it makes the hatch tone uneven line to line (R1-c).
- Lit pixels, sketch box 550x540 at 1280: 5.47 % (frame, lanes, banks, hatch, ticks); 390: 6.9 %; share strip 4.7 %; OG right half 5.1 %.
- Only DPR 1 captures exist; the DPR 2 set asked for in R0 was not delivered (not blocking: line art only gets cleaner at 2x).
- `rubric-check.mjs --app-dir gallery/surfaces` on `landing|share|matchday.html?t=t1`, phone + desktop (served on :8173, `NODE_PATH=e2e/node_modules`):

| Row | landing | share | matchday | Note |
|---|---|---|---|---|
| V1 | PASS | PASS | PASS | tokens only |
| V2 | FIX* | FIX* | FIX* | *`.sr-only` 1 px and hairlines, as accepted in R0 |
| V3 | FIX* | PASS | FIX* | *heuristic on short label paragraphs, as in R0 |
| V4, A1, A2, A3, I7 | PASS | PASS | PASS | |

## Were T1-1…T1-7 fixed?

| Item | Landed? | Note |
|---|---|---|
| T1-1 straight built features | Yes | Frame, lanes, mid ruled; open corners and base ends kept |
| T1-2 curved river, bridge gap, Baron/Dragon in bends | Yes, as specified | But the spec was timid: the S deviates only 2.1 units from the diagonal (12 px at 560). It reads as a gentle bow, not water. R1-a |
| T1-3 hatch perpendicular, 6-unit step | Yes | Hatch length −39 %. Lit pixels only 5.75 → 5.13 % because the curved banks are longer. **The R0 target of about 3 % was a miscalculation by the R0 critique (it left out bank length), not a builder miss** |
| T1-4 inset | Geometry yes; layout no | The builder added a "facing row" with team names either side of the inset, mid-card. See Match Day FIX |
| T1-5 share crop names a base | Yes | Red crop, label right-aligned, leader to the square |
| T1-6 title wrap | Yes | "vs Riverside Rooks –" / "Game 1 plan" |
| T1-7 Match Day header | Yes | Title, meta and sentence as specified |

## Craft first

- **Landing 1280/390.** Could you name it? "A square with an X", still, because mid and the river really do cross corner to corner on the real map. That is the true layout, not a sticker. What was wrong in R0 (hatch parallel to mid, river reading as two more hatch lines) is gone: mid is now the only long straight diagonal, the river is a double line with a bridge gap, and the hatch combs off the lanes. Same stroke, one ink, no hand faking. A director would show this to the owner.
- **Share 390.** A lane ending at a square, labelled in chalk: it reads as a legend entry, quiet and correct. The four hatch stubs under the lane (hatch cut off at y 15.3) read as ruler ticks and say nothing. R1-b.
- **Match Day 390.** At 88 px the inset is a **box with an X**: lanes square plus two crossing diagonals. That is the wireframe "image placeholder" glyph, a nameable silhouette (V7 tell). The facing row adds two names the header already gives, puts a fourth block between the figure and the action (the card allows one figure, one sentence, one action), and **invents sides**: in a Best of 3 neither team owns blue, but the row pins Northgate to the circle. R0's T1-5 rule ("it may not invent a side") applies here too.
- **OG.** `og-t1`: PASS, matches the OG card (72/40 px, sketch bleeds right, chalk "YOUR TEAM"). `og-t1-share`: the chalk labels *our* team at the blue base, but the share page labels the *opponent*. It also repeats the display headline and invents a side. R1-d.

## Hatch density: ruling

**Acceptable as is. Do not tune it further for R0b.** At 5.1 to 5.5 % lit pixels it is below the headline's visual mass, it never touches text, and the tone comes from lines ending, as the card asks. What remains is rhythm, not density: at 1280 the top quadrant's verticals sit under the A to H ruler and echo it, and at 390 DPR 1 the half-pixel straddle makes neighbouring hatches differ in brightness. R1-c fixes the second point; judge the first again at R1 only if the owner picks T1.

## Verdicts

| Still | V1 | V2 | V5 | V7 | V8 | A2 | A3 | A4 | I7 | Chalk | Verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|
| landing 1280 | PASS | PASS* | PASS | PASS | PASS | PASS | PASS | PASS | PASS | 2 (Sign in; YOUR TEAM + leader, on the diagonal away from the button) | **PASS** |
| landing 390 | PASS | PASS* | PASS (see below) | PASS | PASS | PASS | PASS | PASS | PASS | 2 (same) | **PASS** (R1-a, R1-c) |
| share 390 | PASS | PASS* | n/a | PASS | PASS | PASS | PASS | PASS | PASS | 1 (opponent label + leader) | **PASS** (R1-b) |
| matchday 390 | PASS | PASS* | PASS (lead figure) | **FIX** (placeholder X) | **FIX** (layout off-card; invented sides) | PASS | PASS | PASS | PASS | 1 (Read the brief) | **FIX: blocks showing this still** |
| og-t1 | n/a | n/a | n/a | PASS | PASS | PASS | n/a | n/a | n/a | 1 | **PASS** |
| og-t1-share | n/a | n/a | n/a | PASS | FIX | PASS | n/a | n/a | n/a | 1 | **FIX, R1** (not in the owner set) |

V5 at 390: does the first screen say "League team" in one second? "Team" is in the headline. "League" now comes from the sketch, which is fully inside the first 844 px: two base shapes in opposite corners, three lanes, a river crossing mid at a bridge. For the audience (League players) that is the Rift's silhouette at a glance. It would not reach a non-player, who is not the audience. Naming the game in copy is a legal question: ask the librarian. Caveat as in R0: the gallery injects the art after `fetch`, so a true 50 ms first frame is only provable once the app inlines the SVG.

V7: no gradient, blur, glow, emoji, parchment, sepia, brush, compass rose, Inter or rounded-card kit. Familjen display. Geometric and generated, not AI-illustration. The only tell is the Match Day inset (above).

## FIX: blocks the Match Day still (do this before the owner sees T1)

**MD-1 Remove the facing row.** In `surfaces/art/t1.json` delete `"facing": true`. The inset falls back to `.inset.float` (88 px, floated top right, as in T2 and T3, which also makes the three treatments directly comparable). No team names beside the inset. The header ("Scrim vs Riverside Rooks", meta "Northgate Academy") already says who plays.

**MD-2 Break the placeholder X: a bridge gap in the inset river.** In `tools/build-t1-rift.py`, `--inset` branch, replace `RIVER_D` in `ink` with two polylines sampled from `CUBICS` (48 samples per cubic, as for the landing banks), dropping every sample with `mid_dist(x, y) < 4`. That gives an 8-unit gap (about 7 px at 88 px) where the river meets mid, the map convention for a road over water, the same idea as the landing's broken banks. Mid stays continuous. Leave the lanes, the base gaps and the bases as built. Rebuild `t1-inset.svg`.

**MD-3 Re-capture** `t1-matchday-390.png` and `-full` (DPR 1 and 2). Check that the sentence wraps beside the floated inset as it does on T2/T3, that nothing chalk sits in the card except the button, and that the inset no longer reads as box-plus-X at 1:1.

## R1 refinements (do not block the owner showing)

- **R1-a River curvature about 2.3x.** Replace both cubics (and `RIVER_D`) with `M14,14 C24,32 34,44 50,50 C66,56 76,68 86,86`: deviation from the diagonal 4.8 units, still point-symmetric. Baron (31,27) and Dragon (69,73) stay put: their clearance to the near bank rises from 3.35 to 5.7 units. The hatch clip adapts by itself. Re-check the label clearings.
- **R1-b Share strip: no hatch.** Add to `surfaces/surfaces.css`: `[data-art="share"] .hatch { display: none; }`. The crop then shows lane, base and label only.
- **R1-c Even hatch tone at 1x.** Add `[data-art] .hatch { shape-rendering: crispEdges; }`. Hatches are axis-aligned only, so this cannot jag them, and every hatch then renders at the full 8.3:1 ink instead of a 2.8 to 4.6:1 smear.
- **R1-d OG share variant: no chalk label.** For `v=share` in `og/og.html`, draw no team label and no leader. The headline names the team already, an OG card cannot know the plan's side, and the share page's chalk belongs to the opponent. Also ask the owner whether a link preview may show the sharing team's name at all: the share page itself does not (`/s/` shows "only the artefact").

## Staleness flags

- STALE: `critique-r0-t1.md` T1-3 "Expected: lit pixels from ~6.4 % to ~3 %": wrong arithmetic (it left out the longer curved banks). Superseded by the density ruling above.
- STALE: `references.md` T1 card still describes a straight two-line river, 45-degree hatch and a crossing share crop, and the field card says the inset sits "at the card's top right". Bring the card in line with R0's T1-1…T1-5, keeping the top-right float (MD-1).
- CONTRADICTION (open, carried from R0): `docs/DESIGN.md` line 202 ("chalk ... not for decoration") vs the T1 chalk team label.

## R0b iteration 2

Atelier, fresh context, 2026-10-17. Checked MD-1, MD-2 and R1-c (`shape-rendering: crispEdges` on `.hatch`) against `gallery/out/t1-fix-log.md` ("Re-critique fixes"). Opened `t1-matchday-390`, `-dpr2`, `-full`, `t1-landing-1280`, `t1-landing-390`, `t1-share-390` and `og-t1`. I also looked at a 4x nearest-neighbour crop of the 1x inset (`/tmp/m151r0b2/inset4x.png`, not committed). I wrote neither the fixes nor the R0b list they answer, so there is no self-approval conflict.

| Still | V5 | V7 | V8 | A2 | Chalk | Verdict |
|---|---|---|---|---|---|---|
| matchday 390 (1x, 2x, full) | PASS: the time is the lead figure | PASS, just: the river is now a curve with a gap at mid, and the lanes stop short of the bases. It no longer reads as the straight two-diagonal placeholder glyph. At 1x the gap is only about 2 px on each side of mid | PASS: no facing row, the inset is floated top right like T2/T3, the sentence wraps beside it, and no side is invented | PASS | 1 (Read the brief) | **PASS** |
| landing 1280 | PASS | PASS | PASS | PASS | 2 (unchanged) | **PASS**: R1-c makes the hatch tone even, with no jaggies |
| landing 390 | PASS | PASS | PASS | PASS | 2 (unchanged) | **PASS**: the hatch smear at 1x is gone |
| share 390 | n/a | PASS | PASS | PASS | 1 | **PASS** (R1-b still open: the hatch stubs) |
| og-t1 | n/a | PASS | PASS | PASS | 1 | **PASS** |

Nothing blocks showing T1 to the owner. Carried to R1, in priority order:
- **R1-a first.** The curvature change (`M14,14 C24,32 34,44 50,50 C66,56 76,68 86,86`) also applies to the `--inset` build. At 88 px it is what finally separates the river from the mid diagonal at 1x.
- **Optional, only if the owner picks T1:** widen the inset's gap to `mid_dist < 5`.
- R1-b and R1-d stand as written.
