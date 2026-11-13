# Critique R1 Rift (atelier, critique mode, 2026-10-17)

Rubric CLI: not runnable. It needs playwright-core in the app dir (absent: no e2e/node_modules) and takes a live page, not PNGs. Gap for the builder, not a verdict.
Contrast from pixels (bg 8,20,21): ink core 8.3:1 (share, inset), 14.5:1 on chalk and labels; hatch strokes antialias to 2.7 to 5:1 mean, within R0's ruling.

| Still | Verdict | Reason |
|---|---|---|
| rift-sketch-1280 / 390 / dpr2 | FIX | Could you name the shape? No: the hatch reads as scattered barcode and text lines, not rock. Edge character lost: stacks of 4 to 16 u runs at pitch 1.1 with ragged ends form blocks (left quadrant like a paragraph). Hatch field is 40x denser per run than R0 and has no mass shape. Lanes, river, bases, labels, chalk label fine. |
| rift-survey-1280 | PASS | Outline plus 45-degree hatch vs stipple bushes: two vocabularies, matches map-grey. |
| rift-survey-4x-1280 | PASS | Same, stipple legible. Clip at viewport edges is the harness. |
| rift-share-390 / dpr2 / 1280 | FIX (minor) | Four bare strokes read as a cancel glyph, not a map. Bridge gap fine. |
| rift-inset-390 / dpr2 | FIX (minor) | Card says no frame; inset draws an octagon frame. Bases, lanes, gap read. |

Reading-flow at 390: eye lands on YOUR TEAM, then hatch noise competes with Baron and Dragon labels; hatch must recede.

## Fix list
1. build.py hatch pitch 1.1 -> 5.0 u (card range 5 to 7; never stroke or opacity).
2. Inset 0.4 -> 1.5 u.
3. Min run length 3 -> 6 u; max 16 stays; drop jitter on both ends (inner end only, 0.6).
4. Drop mask rings below 20 u2 or narrower than 6 u (card 8 and 4); merge rings under 2 u apart; at most 60 vertices total (now 367).
5. Simplification 0.5 -> 2.0 u (Douglas-Peucker), at most 14 vertices per ring.
6. Re-measure lit pixels: target 4.5 to 6.0 percent at 1280 and 390 by adjusting pitch only (5 to 7). 9.7 percent at 390 is out of range.
7. River refit 2.2 u off exceeds 1.5: keep R1-a cubics unless measured deviation >1.5, then refit with the same two cubics; document the deviation in docs/rift-schematic.md.
8. rift.rs inset variant: remove the frame path; keep lanes, river gap (mid_dist < 5), bases.
9. Share crop (rift.rs): add the Baron diamond? No (card forbids). Instead centre on (50,50) with width 40 u as specified and keep bank ends unclipped inside the crop by extending crop 3 u so every stroke ends in-frame or at the edge on purpose; recheck at 390 side by side with the card.

Re-run capture after 1 to 6; I judge the new stills fresh.
