# T1 fix log: critique-r0-t1.md T1-1 to T1-7 (R0b, 2026-10-17)

Paths relative to `.planning/missions/M151/gallery/`. Line numbers as committed.

| Item | Change | Where |
|---|---|---|
| T1-1 Straight built features | `wobbly()` bows and river jitter removed; frame = four straight strokes, open corners 1.5; lanes `M10,84.5 L10,10 L83.5,10`, `M15.5,90 L90,90 L90,16.5`, mid `M15.2,84.8 L83.5,16.5` | `tools/build-t1-rift.py:84-90` |
| T1-2 Curved river | centreline `M14,14 C26,30 36,40 50,50 C64,60 74,70 86,86`; banks = 48 samples per cubic offset +-1.5 along the normal, polylines, broken within 2 units of mid | `tools/build-t1-rift.py:23-26, 92-107` |
| T1-2 Baron/Dragon in the bends | centres (31,27) and (69,73), 3.35 units clear of the near bank (printed by the build); Dragon's word on the left so it does not cross the river; `MARK_CLEAR` recomputed | `tools/build-t1-rift.py:79-81`; `surfaces/art/t1.json:7-10`; `surfaces/surface.js:45`; `surfaces/surfaces.css:76` |
| T1-3 Hatch | perpendicular to its lane (left/right horizontal, top/bottom vertical), positions 20..80 step 6, from 13 to 13 + 0.55 (avail - 5), clipped 2 units short of bank and mid, < 3 dropped, +-0.6 seeded jitter on the inner end only; right = point reflection, top/bottom = x/y swap; `LABEL_CLEAR` kept. Hatch length 637.6 -> 388.2 units (-39 %) | `tools/build-t1-rift.py:121-162` |
| T1-4 Match Day inset | new `art/t1-inset.svg` (`--inset`): no frame/hatch/labels, lanes + mid stop 3 short of bases, single-line river, circle r7 at (14,86), square 12 from (82,6). Card shows it as a facing row: "Northgate Academy" beside the circle (bottom left), 88 px inset, "Riverside Rooks" beside the square (top right); legible at 88 px, so not enlarged. T2/T3 keep their floated inset (`facing` only in t1.json) | `tools/build-t1-rift.py:50-74`; `surfaces/art/t1.json:4-5`; `surfaces/surface.js:11-17, 28`; `surfaces/matchday.html:17-25, 68-73` |
| T1-5 Share crop names a base | `shareSides` in t1.json: opponent red -> viewBox `54 4.6 40 10.73`, label right-aligned to x 82, leader (82.5,7.6)-(85,9); opponent blue -> `6 84.67 40 10.73`, label x 17, leader (16.5,92.4)-(14.1,90.1); no side -> old crossing crop, no label. Side from `data-our-side` (still: blue) or `?side=`. Leader drawn as `.chalk-lead`, 1 px accent | `surfaces/art/t1.json:13-16`; `surfaces/surface.js:29-34, 53-59`; `surfaces/surfaces.css:68, 73`; `surfaces/share.html:11, 30` |
| T1-6 Share title wrap | "vs Riverside Rooks&nbsp;– Game 1 plan" (no-break space + en dash); same rendering for the plan/draft names in Match Day's prep details | `surfaces/share.html:33`; `surfaces/matchday.html:84, 87` |
| T1-7 Match Day header | title "Scrim vs Riverside Rooks"; meta "Northgate Academy · Patch 16.20 since 11 Oct"; sentence "Best of 3, with 2 of 5 prep steps done." | `surfaces/matchday.html:58-59, 67` |
| OG share label | `&v=share` labels the team ("NORTHGATE ACADEMY") instead of "YOUR TEAM", at 26 px so it stays inside the centre 1100 px | `og/og.html:30-31, 66` |

Shared-skeleton change (surface.js, surfaces.css, matchday.html, share.html), so T2 and T3 stills were re-captured under the same filenames; their art is unchanged (they keep `shareLabel` and the floated inset), only the Match Day header/sentence and the en dash differ.

Open: the critique expected hatch lit pixels to roughly halve (~6.4 % -> ~3 %). With its exact parameters the hatch length falls 39 %; lit pixels in the 1280 sketch box go 5.75 % -> 5.13 % because the curved banks are longer than the old straight pair. Not tuned beyond the spec; the fresh critique decides.
