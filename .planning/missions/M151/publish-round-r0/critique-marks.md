# M151 critique: the nine marks (atelier, 2026-10-16)

Mode: critique. Reference: `references.md` "Mark grid spec" + section 2 (Lucide principles, Phosphor 16-first, swisstopo key). Builder of record: the M151 builder (I wrote the spec, not the drawings, so no self-approval conflict).

## What I ran

- Read `public/marks/*.svg`, `build-marks.py`, `src/pages/design_marks.rs`, both contact sheets (1280, 390).
- Rasterised every 16 px master with `rsvg-convert` at 16 px (DPR 1) and 32 px (DPR 2) on a page-tone ground; looked at nearest-neighbour 10x enlargements and at 3x soft enlargements (what an eye at arm's length sees). Work files are in `/tmp/m151crit/` (not committed).
- Ink coverage per mark (alpha sum at 160 px) and stroke bounding box, to check optical weight and the live area (1 to 15).
- Contrast from pixels in `marks-contact-1280.png`: the brightest 16 px stroke pixel in the "page, secondary" panel is 10.8:1 against the ground (rgb 8,20,21). Passes.
- Not run: the mobile capture, motion and reduced-motion steps (the marks are static, nothing to animate) and `rubric-check.mjs` (it needs a running app render; this slice is a contact sheet). Re-run them on the `/design` gallery once fixes land.
- Lint: `./scripts/lint_design.sh` gives OK, every rule at zero. Tokens: `stroke-rule` / `stroke-outline` come from `--color-rule` / `--color-outline` in `input.css` `@theme` (lines 104 and 105), so Tailwind v4 generates both. `text-label`, `text-reading`, `text-caption`, `text-secondary`, `text-primary`, `text-muted` all exist. No raw palette, no radius, no sub-12 text.

## Weight (ink coverage at the 16 master; set median 23.1 %)

| top | jungle | mid | bot | support | dragon | baron | herald | tower |
|---|---|---|---|---|---|---|---|---|
| 23.5 | 25.4 | 19.1 | **11.7** | **17.0** | 25.3 | 26.2 | 21.8 | 23.1 |

Target band: median plus or minus 20 % (18.5 to 27.7). Bot is at half the median and support is under the band, which matches what you see in the inline strip: those two look like they belong to a lighter set.

## Craft questions first (per mark, at 16 px beside the word)

| Mark | Can you name the shape? | Edge/stroke character | Riot distance | Verdict |
|---|---|---|---|---|
| Top | trig point (triangle + dot), as briefed | clean, apex round join | summit, no corner L: far | **PASS** |
| Jungle | three map trees | crowns read; the bottom trunk ends at y 15.25, **outside the live area**; right trunk 0.8 px from the bottom crown (rule: 1.5) | no branches or claw: far | **FIX** |
| Mid | survey station / map "locate" | strongest mark at 16 | concentric, no diagonal: far | **PASS** (watch: never place it where it could pass for a "locate me" button) |
| Bot | **yes: the RSS feed glyph** (dot + quarter arc) | thin; lightest by far | no bottom-right L: far | **FIX** |
| Support | cup holding a point; the redraw fixed the smiley | fine, but the drawing is 10.6 px wide while the set spans 13.6 | U, not V crest: far | **FIX** (size) |
| Dragon | diamond with a wave | the wave ends 0.7 px from the inner diamond edge, so at DPR 1 it fuses with the side vertices and reads as a diamond cut in half | no head or wing: far | **FIX** |
| Baron | at 32: nested diamond. At 16: **a muddy ring with a grey hole** (the 1.33 px counter fills with antialiasing) | the inner shape is neither open nor solid | no horns or eye: far | **FIX** |
| Herald | diamond with a dotted row | outer points at x 5.5 / 10.5 leave 1 px gaps (rule: 1.5) and sit on pixel centres while the middle one sits on a pixel edge, so the middle point renders dimmer | no eye or shell: far | **FIX** (small) |
| Tower | lookout on a base line | crisp, open-based | no battlements, not the filled side square: far | **PASS** |

Family coherence: one hand. One stroke width, round caps and joins, `currentColor`, stroke-only, diamond family for objectives, open forms for roles. Distinguishable as a set at 16 px (all nine read differently in the inline strip). DESIGN 1.13: no gamer ornament, no emoji, no confetti (nine marks, each with a word, is a vocabulary, not confetti). Anti-AI-illustration: none of the convergent defaults; nothing figurative.

Labels: every mark in the gallery stands beside its word and is `aria-hidden`; the e2e test in `gallery.spec.ts` checks the word. The standalone SVG files carry `<title>`; wherever a file is used through `<img>`, it needs `alt=""` beside the word (otherwise the word is read twice).

## FIX list (buildable; edit `build-marks.py` MARKS and `design_marks.rs` MARKS together)

1. **Jungle**: three trees with 1-unit trunks, all inside 1 to 15, every gap at least 1.5:
   `M2 5a2 2 0 1 0 4 0a2 2 0 1 0-4 0`, `M4 7v1`, `M10 4.5a2 2 0 1 0 4 0a2 2 0 1 0-4 0`, `M12 6.5v1`, `M6 11a2 2 0 1 0 4 0a2 2 0 1 0-4 0`, `M8 13v1`. (Bbox 1.25 to 14.75; trunk tip to bottom crown 1.5 on both sides; coverage 23.6 %.)
2. **Bot**: keep the dot and arc, add three graduation ticks crossing the arc (radius 8 to 10 from the origin, at 22.5, 45, 67.5 degrees from vertical), so it reads as a sextant scale, not RSS:
   add `M7.06 4.61L7.83 2.76`, `M9.66 6.34L11.07 4.93`, `M11.39 8.94L13.24 8.17`. Coverage 11.7 to 14.7 %. Still the lightest; acceptable, because the ticks give it a texture the others lack. Do **not** add a second concentric arc (that completes the RSS glyph) or radii (a corner L, banned).
3. **Support**: widen and deepen the cup to the keyline width: cup `M3 3V8.5a5 5 0 0 0 10 0V3`, dot `M7.25 8.5a.75.75 0 1 0 1.5 0a.75.75 0 1 0-1.5 0`. Bbox 2.25 to 13.75 by 2.25 to 14.25; coverage 19.1 % (in band).
4. **Dragon**: shorten the wave so it floats inside: `M5 8q1.5-2 3 0t3 0`.
5. **Baron**: make the inner diamond a deliberate solid core rather than a failed counter: `M8 6.25L9.75 8L8 9.75L6.25 8Z` (its counter closes, the gap to the outer diamond is exactly 1.5 px). There is no room for a readable open inner diamond with 1.5 px gaps at 16 (2 strokes plus 2 gaps need 4.24 px of the 4.24 available).
6. **Herald**: points at `M5 8h0M8 8h0M11 8h0` (gaps 1.5 px, all three points render alike). The 1.5 px point departure is accepted: three 3 px dots do not fit inside the diamond.
7. **Gallery specimen** (`design_marks.rs` `mark_specimen`): add the triangle keyline `<path d="M2 13.25L8 2.75L14 13.25Z" />` to the dashed `stroke-outline` group so it matches the contact sheet and the spec (Top is drawn on it).
8. **Contact sheet only**: the chalk panels are fine as a test, but the spec says marks are never chalk; make sure no shipped surface uses `text-accent` on a mark.

## Verdict

**Overall: FIX.** The family is right (one hand, one grid, far from the in-game icons, no bans hit), Top, Mid and Tower pass as drawn. Six marks need coordinate changes, two of them for craft rather than numbers: Bot reads as RSS, and Baron's centre muddies at 16 px. After the fixes, re-capture the `/design` marks section at 1280 and 390 (DPR 1 and 2) and send it to a fresh critique run; I checked these coordinates in throwaway renders, so the next pass should be a different context.

## Staleness flags

- CONTRADICTION: docs/DESIGN.md 6.3 and 7 ("no role icons") vs the shipped marks: still open from `references.md`; the amendment must land with the marks.
