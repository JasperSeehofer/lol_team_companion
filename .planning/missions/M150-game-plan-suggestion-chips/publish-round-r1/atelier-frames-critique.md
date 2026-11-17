# M150 atelier critique, publish round r1 (fresh context)

Atelier, critique mode, 2026-11-17. Fresh context: this run did not write `atelier-recipes-r1.md` (playbook §5.7 satisfied). Spec: that file's cards 1 to 5. Contract: garden `M150-game-plan-suggestion-chips.md`. Tokens: `input.css` / `docs/DESIGN.md`.

Orchestration note: the caller asked for edits and test runs to go to subagents. This run had no subagent tool, so the reads were done in this context. Nothing was edited or run except pixel sampling (PIL) and writing this file.

## Capture protocol: what was used

- No new captures. The judgement rests on the 16 CI frames (run 43298644926 on 080a1eb, debug build, `e2e/tests/suggestion-chips.spec.ts`) and the 3 BEFORE frames from M156 r1.
- Phone frames are 390×844, not 852. Light/dark: there is one theme only.
- Motion is judged from the 3-frame accept strip (chip, ~120 ms, ~400 ms) and the reduced-motion strip. There is no 6-frame/10 s strip because nothing on the surface idles in motion.
- Contrast was recomputed from frame pixels (WCAG relative luminance).
- `rubric-check.mjs` was not re-run: the harness reaches public pages only (`rubric-check.md`). A1–A4, I1–I8 and V1–V3 are judged below from frames plus `src/components/ui/suggestion.rs`, `src/components/ui/dialog.rs` and `src/pages/game_plan.rs`.
- `lint_design.sh`: not run (read-only critique).

## Pixel measurements (1280 unless noted)

| What | Pixels | Ratio | Need |
|---|---|---|---|
| Chip claim on `bg-ink-3` | (233,240,239) on (28,43,43) | 12.7:1 | 4.5 |
| Chip edge (`border-outline`) vs page | (98,115,116) on (8,20,21) | 3.77:1 | 3.0 |
| Count line `text-secondary` on page | (188,199,198) | 10.8:1 | 4.5 |
| Reason (serif italic), dots, "Not for us" | (188,199,198) | 10.8:1 | 4.5 / 3.0 |
| Facts caption, wait line `text-muted` 12 px | (149,161,162) | 7.0:1 | 4.5 |
| Card hairline `border-rule` | (46,59,59) | 1.6:1 | decorative |
| Mid-write chalk on `bg-ink-2` | (231,231,152) on (18,33,34) | 12.9:1 | 4.5 |
| Chalk-hue pixels in the Win field | mid 191, **settled 7**, at rest after accept 7; reduced mid 212, reduced settled 15 (two lines) | n/a | settled = rest baseline |
| Chip row and card at rest | 11 / 48 warm pixels; all are sub-pixel AA fringes on white glyphs, not chalk | n/a | 0 chalk |
| Phone sheet (390) | top hairline y=562, sheet 282 px tall (33% of the viewport); "Add to plan" 732→775 = 44 px; close X `size-8` = **32 px** | n/a | ≤85%, 44 px |
| Chip height | 32 px at 1280, 44 px at 390 | n/a | 32 / 44 |

## Craft first (step 7: same size, next to the reference rows)

- **Chip.** You cannot name a shape here: it is a paper tag with a 3 px corner, a pin-dot and a "§". The edge matches the reference: an outline hairline, no fill change, no chalk. It covers nothing. A design director would ship it. One flaw: the rotation makes the text baseline step by 1 px mid-claim (zoomed 1280: "Malphite bursts" sits higher than "Amumu engages,").
- **Card.** It reads as a margin note, not a panel: a 576 px hairline, the claim in sans, the reason in Source Serif italic, then facts and dots. There is no box, fill or shadow, and it pushes "How we win" down by 152 px. Ship.
- **Collapsed line.** "▸ 3 suggestions" sits in the plan's own sub-label voice under Matchup. Compared with the BEFORE frames, it is the only addition to the reading flow. Ship.
- **Written line.** The mid frame genuinely shows the reveal. "Amumu engages, Malphite bur" is chalk and cut left-to-right, sitting exactly on the settled line's baseline: offset 20 px from the field top in both frames, so 0 px drift. The settled frame is ink. This is the signature moment and it is the only one on the page. Ship.
- **Phone row and sheet.** The sheet is right: on the bottom edge, hairline top, no shadow, no radius, no grabber, full-width 44 px buttons. The row is not right: the first chip's left edge is cut off and the bleed stops at the gutter (see FIX B1).

## Verdict per surface

| # | Surface | Shape | Edge | Contrast | Speed | Layering | Tint | Type | Verdict |
|---|---|---|---|---|---|---|---|---|---|
| 1 | Chip | tag, 3 px, not a capsule ✓ | outline 3.77:1 ✓; expanded `border-primary` ✓ | 12.7:1 ✓ | no motion at rest; retire fade ✓ | none ✓ | no chalk at rest ✓ | 13 px medium; rotation steps the baseline ✗ (minor) | **PASS** (non-blocking N1) |
| 2 | Card (desktop inline) | margin note ✓ | 1 px rule ✓ | 10.8 / 7.0:1 ✓ | 180 ms grid rows ✓ | in flow, covers nothing ✓ | Secondary + Quiet buttons, no chalk ✓ | one serif (the reason) ✓; claim, source label "Trait rules" and pair verbatim ✓; dots, no number ✓ | **PASS** |
| 3 | Collapsed, wait and none lines | sub-label, not an h2 ✓ | section `border-t border-rule` ✓ | 10.8 / 7.0:1 ✓ | none on load ✓ | covers nothing at 390 or 1280 ✓ (Law 11) | no badge ✓ | 13 px / 12 px ✓ | **PASS** (the none line was not captured; verified in code at `game_plan.rs:1814`, wording verbatim) |
| 4 | Written line / accept strip | text reveal, not a moving object ✓ | n/a | chalk 12.9:1 while writing ✓ | reveal visible at ~120 ms ✓; ≤300 ms ✓ | echo exactly on the line ✓ | chalk only mid-moment; settled = baseline ✓ | field metrics match ✓ | **PASS** (watch item W1) |
| 5 | Phone row + bottom sheet | sheet ✓; row clipped ✗ | hairline top, no shadow ✓ | ✓ | 220 ms rise (not judged; no mid-rise frame) | the scrim does the separation ✓ | no chalk ✓ | sheet h2 = claim ✓ | **FIX** (B1, B2) |

Reduced strip: there is no partial-reveal frame; the chalk line is static, then ink at 400 ms ✓.

## Checks the caller named

- Plan content never covered, 390 and 1280: ✓ (`m150-reading-flow-*`, `m150-chips-open-*`, `m150-card-1280`).
- No sparkle or AI glyph, percentages, gradients, shadows, toast or new colours: ✓. The glyph is "§" and `aria-hidden`; there is no shadow on BottomSheet (`dialog.rs:35`); `ToastContext` is not used on accept.
- One chalk primary per view: ✓. Only the header's "Save plan" is chalk; "Add to plan" is Secondary.
- 44 px on phone: chips ✓, line ✓ (`min-h-11`), sheet buttons ✓, **sheet close X 32 px ✗**.
- Legibility: ✓ everywhere. The mild exception is the rotated chip text (N1).

## Rubric (judged from frames + code)

| Id | Result | Rests on |
|---|---|---|
| A1 target size | **FIX**: the sheet close X is 32×32 (`dialog.rs:160` `size-8`). Everything else passes, with 8 px between chips | sheet-390, chips-open-390 |
| A2 contrast | PASS (table above) | card-1280, wait-line-1280, strip-2-mid |
| A3 reduced motion | PASS: no reveal, static chalk ≤300 ms, then ink; the state is also told by the live region "Added to Win conditions." | strip-reduced-1..3 |
| A4 pointer + screen reader | PASS (provisional; human-run): the row scrolls, but every chip is a Tab stop; strength uses shape, not hue; the glyph is `aria-hidden` and the source is spoken in words | code `suggestion.rs` |
| I1 / I2 / I6 | n/a: no agent feature that can fail navigation, no wait over 10 s, no notifier | n/a |
| I3 disclosure ≤3 | PASS: line → chip → card; planning itself needs no open | reading-flow-* |
| I4 correction path | PASS: the accepted line is plain editable text in the field | accepted-win-* |
| I5 confirms | PASS: none | n/a |
| I7 secondary collapsed | PASS: collapsed on every load, no input in the row | reading-flow-*, accepted-win-390 (after reload) |
| I8 undo / consent | PASS: nothing writes without the tap (the consent); the line can be deleted in the field | strip |
| V1 tokens | PASS: no colour literal; every colour comes from the app's tokens | code |
| V2 4/8 scale | PASS against the app's `--spacing` token. Note: `py-1.5` (6 px) and `pl-2.5` (10 px) on the chip are off the 4/8 list if the harness measures raw px | `suggestion.rs:32` |
| V3 measure 45–75 | **FIX (non-blocking)**: in the 390 sheet the reason wraps at about 41 characters (16 px serif in 350 px) | sheet-390 |

## UX1–UX10 (critique-checklist.json)

| Id | Answer | Frames |
|---|---|---|
| UX1 | n/a: Add writes locally and the autosave is not stubbed, so there is no in-flight frame | n/a |
| UX2 | n/a: the surface has no input | n/a |
| UX3 | PASS: blurred, the plan fields and Matchup still lead; the line is quiet | reading-flow-1280/390 |
| UX4 | PASS: chalk means only "being written / focus" | strip-2-mid, card-1280 |
| UX5 | PASS: the same `SuggestionCardBody` is used inline and in the sheet | card-1280, sheet-390 |
| UX6 | partial: expanded and rest are captured; hover and pressed were not (`force` hook exists, gallery) | card-1280 |
| UX7 | **FIX**: sheet close X 32 px | sheet-390 |
| UX8 | PASS: everything above the acted-on control keeps its place (the card below collapses) | strip-1 → strip-3 |
| UX9 | PASS: the claim is identical on chip, card title, sheet h2 and written line | card-1280, sheet-390, strip-3 |
| UX10 | PASS: the content comes first; chalk lasts ≤300 ms | strip |

## FIX list, ranked

### Blocking

**B1. The phone row's first chip is clipped and the bleed stops at the gutter** (`m150-chips-open-390`, `m150-sheet-390`).
- What happens:
  - `snap-start` without scroll padding snaps the first chip to the scroller's own edge, x=0 after the `-mx-4` bleed.
  - The grid child `min-h-0 overflow-hidden` then clips at the 16 px gutter, so the left border and half the pin-dot vanish.
  - The right-hand peek is also cut at x=374 instead of running off the screen edge.
- Fix in `src/pages/game_plan.rs`:
  - Line 1851 `<ul>`: add `scroll-px-4`. Snapping then respects the 16 px gutter.
  - Line 1850 wrapper: change `min-h-0 overflow-hidden` to `-mx-4 min-h-0 overflow-hidden px-4 md:mx-0 md:px-0`. The clip box then reaches the screen edges and the bleed and peek work.
- Re-capture `chips-open-390` and check two things: the first chip's left edge is at x=16 with its border visible, and `scrollWidth <= 390`.

**B2. The sheet's close X is 32×32 on the phone** (A1 / UX7).
- Fix in `src/components/ui/dialog.rs:160`: change `-mr-2 -mt-1 grid size-8` to `-mr-3 -mt-2 grid size-11 md:-mr-2 md:-mt-1 md:size-8`.
- This change applies to every Dialog on phones, which is correct anyway.
- Alternatively, scope it to `BottomSheet` with a match arm. Do not remove the X: Escape and the scrim are not discoverable on touch.

### Non-blocking

**N1. Chip text baseline steps from the rotation.**
- The recipe's own escape clause applies ("if it smears at 1280, reduce to ±0.3°").
- Fix in `src/components/ui/suggestion.rs` `tilt()`: use `rotate-[-0.3deg]`, `rotate-[0.2deg]` and `rotate-[-0.15deg]`.

**N2. Sheet reason measure under 45 characters at 390** (V3).
- Fix in `game_plan.rs:1958`: change `note text-reading text-secondary` to `note text-body text-secondary md:text-reading`. That gives about 48 characters per line in the sheet, with the desktop unchanged.

**N3. Chip padding off the 4/8 list** (V2, only if the harness measures raw px).
- Fix in `suggestion.rs:32`: change `py-1.5 pl-2.5 pr-2` to `py-1 pl-2 pr-2`. The `min-h-11` / `md:min-h-8` heights keep the size.

**W1 (watch item, no change asked).**
- The inline card collapses (180 ms) during the 180 ms write, so the Win field rises about 150 px while the line is being written.
- The echo travels with the field (0 px drift), so the moment holds. But the Strategy tags end up about 8 px from where the pointer clicked "Add to plan", so a double-click could toggle a tag.
- Put it on the react card for the human. Do not fix it now.

## Overall

**FIX: two small blocking changes (B1, B2), then PASS.**
- The craft is right where it matters. The written line is a real reveal that dries to ink with no residue, which is a genuine signature moment. The tag and the margin note avoid every default (no pill, no panel, no chalk at rest, no sparkle).
- The round's signature moment worth showing the human first is the 3-frame accept strip (`m150-accept-strip-1/2/3-chip|mid|settled-1280`).
- After B1/B2, re-capture `chips-open-390` and `sheet-390` only. A fresh-context recheck of those two frames is enough.

## Staleness flags

- STALE: `.planning/missions/M150-game-plan-suggestion-chips/atelier-recipes-r1.md` card 2/5. It says "no close X (the chip toggles it)" for the card. The sheet inherits the Dialog's X, and `dialog.rs`'s A11y line says "the close button" where the card said "the chip". The implementation is the better choice on touch; the card text should follow it.

## Re-check after 945aa08 (CI run 44135227504)

Atelier, critique mode, fresh context, 2026-11-17. This run did not write the recipes or the fix (playbook §5.7 satisfied). Orchestration note: the caller asked for edits and test runs to go to subagents. This run had no subagent tool, so the reads were done in this context. Nothing was edited or run except PIL pixel sampling and appending this section.

Inputs:
- Fix diff: `git show 945aa08`.
  - `dialog.rs`: close button `size-11` below `md`, `md:size-8`.
  - `suggestion.rs`: tilt changed to -0.3/0.2/-0.15°, chip padding `py-1 pl-2 pr-2`.
  - `game_plan.rs`: the wrapper is now `-mx-4 px-4 md:mx-0 md:px-0`, the `<ul>` gets `scroll-px-4`, the sheet reason is `text-body md:text-reading`.
- Frames: `frames/after/m150-*.png` from CI run 44135227504 on eeec922. eeec922 contains 945aa08 (`merge-base --is-ancestor`). The e2e suite was 200/200.
- No new captures and no 6-frame idle strip, as in r1: nothing on the surface idles in motion.

### Measurements

| Check | Frame | Measured | Need | Result |
|---|---|---|---|---|
| B1 first chip left edge | chips-open-390, sheet-390 | outline column at **x=16** (page at x=15: 8,20,21; at x=16: 109,126,127). Left border and pin-dot are intact | x=16, border visible | ✓ |
| B1 first chip box | chips-open-390 | x 16→268, y 134→177 (**44 px** tall) | 44 px | ✓ |
| B1 bleed and peek | chips-open-390 | the 2nd chip starts at x=277 (8 px gap). Its text "Amumu's engag" runs to **x=389, the screen edge**. In r1 it was cut at x=374 | runs off the edge | ✓ |
| B1 page overflow | spec `shot()` | `documentElement.scrollWidth <= 390` is asserted before every frame is written, and the run is green | ≤ 390 | ✓ |
| B2 close X | sheet-390 + `dialog.rs:160` | `size-11` below `md` = **44×44**. The glyph's centre (359.5, 618.5) matches the box the code implies: px-5 sheet + `-mr-3` gives 338→382, centred at 360 | ≥ 44 px | ✓ |
| Sheet geometry | sheet-390 | top hairline at y=584, sheet 260 px (31% of the viewport); "Add to plan" 733→774 | ≤ 85%, 44 px | ✓ |
| N1 tilt | card-1280 | the chip-1 baseline goes from y=141 ("Amumu") to 140 ("bursts") across 200 px, about 1 px. That is the -0.3° floor the recipe's escape clause allows; r1 was about 2 px at -0.6° | ≤ ±0.3° | ✓ (residual 1 px AA step accepted) |
| N2 sheet reason | sheet-390 | serif italic at text-body: x 20→320 for the 50-character reason on **one line**. The 350 px measure holds about 55–58 characters | 45–75 | ✓ |
| N3 chip padding | `suggestion.rs` CHIP | `py-1 pl-2 pr-2` = 4/8 px. Heights are kept: 32 px at 1280 (y 122→153), 44 px at 390 | 4/8 scale | ✓ |
| Chip contrast | card-1280 | claim 12.7:1; outline (107,124,125) on page 4.4:1 | 4.5 / 3.0 | ✓ |
| Accept echo drift | strip-2-mid vs strip-3-settled | the chalk text sits 14 px below the field top in both frames (field top 375 vs 368: W1 collapse travel). **0 px drift** | 0 | ✓ |
| Settled residue | strip-3-settled vs accepted-win-1280 | the chalk-hue count in the Win field is 40 vs 40 (AA fringe baseline, same as at rest). The brightest text is (233,240,239), which is ink | = rest | ✓ |
| Reduced strip | reduced-2-mid, reduced-3-settled | mid: the whole second line is static chalk, with no partial reveal. Settled: ink (233,240,239) | static, then ink | ✓ |

### Craft (step 7)

- **Row.** The phone row now reads as a deliberate shelf. The first tag sits on the page's gutter, aligned with "3 suggestions" and "How we win". The second tag runs off the glass, which says "scroll" without any affordance chrome.
- **Sheet.** It is unchanged in character: bottom edge, hairline, no shadow, no grabber. The X still reads as a quiet glyph. Its hit box grew, and so did its weight.
- **Tags at 1280.** The tilt is now felt rather than seen, which is right for a paper tag.

A director would ship all of it.

### Verdict per surface

| # | Surface | Verdict | Rests on |
|---|---|---|---|
| 1 | Chip | **PASS**. N1 and N3 are applied, with no regression at 1280 (32 px, 8 px gaps, wrap row) | card-1280, chips-open-1280 |
| 2 | Card (desktop inline) | **PASS**. It is unchanged at 1280 (`md:text-reading` keeps the desktop reason size), with Secondary + Quiet buttons | card-1280 |
| 3 | Collapsed / wait lines | **PASS**. Nothing is covered at 390 or 1280 | reading-flow-390/1280, chips-open-* |
| 4 | Written line / accept strip | **PASS**. The reveal, 0 px drift and an ink settle are intact. W1 is still a watch item (7 px field travel in this capture) | accept-strip-1..3, reduced-1..3 |
| 5 | Phone row + bottom sheet | **PASS**. B1 and B2 are fixed, N2 is applied | chips-open-390, sheet-390 |

Rubric deltas from r1:
- A1 / UX7: FIX → **PASS**.
- V2: → **PASS**.
- V3: FIX → **PASS**.
- Everything else stands.

### Overall

**PASS.** Both blocking findings are fixed in the pixels, not only in the code, and all three polish items are visible with no regression at 1280. The signature moment for the human is still the 3-frame accept strip. W1 (the card collapsing during the write) remains a react-card item, with no change asked.

The r1 staleness flag (the recipe card's "no close X" wording) still stands. 945aa08 touched `atelier-recipes-r1.md`, but this run did not verify whether that edit resolves the flag.
