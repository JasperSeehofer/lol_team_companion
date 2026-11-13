# M151 references: Survey, sharpened (atelier, 2026-10-16)

Mode: sheet + recipe. Sources: `link-check.md` (fresh re-open, trusted over memory) plus my own captures in `refs/` (headless Chromium 1280x900, 16 Oct 2026; I looked at every one). **V** = opened and seen, counts toward the minimum of 3. **U** = unverified, cited for context only. Tokens are the names in `input.css`; no new colour, no new token except where a card names a **token gap**.

Ink, defined once: `--color-primary` at 70 % (`stroke="var(--color-primary)" stroke-opacity="0.7"`, never a `text-primary/70` class, which the `faded-text` lint rejects). Measured composite: 8.3:1 on `page`, 8.2 on `canvas`, 7.9 on `terrain-jungle`, 6.6 on `terrain-lane`. Ink at 40 % is 3.5:1 (used only where a card says so).

## Corrections from link-check.md, resolved

1. **Lucide is 24 px / 2 px / 1 px safe zone / round joins and caps** (seen: `refs/lucide.png`, the 14-rule list). Our grid is **16 px master, 1.5 px stroke, 1 px safe zone; at 24 px the same drawing scaled 1.5x with the stroke held at 2 px** (non-scaling). Why: the marks sit beside 13 px `text-label` (Public Sans 500, stems about 1.4 px), so 1.5 px matches the label's weight; 1 px goes grey at DPR 1, and 2 px at 16 fills the 14 px live area and closes counters. At 24 the 2 px stroke equals Lucide. Ratio 0.094 at 16 against Lucide's 0.083: slightly heavier on purpose, stated here so no one cites Lucide for it.
2. **FiveThirtyEight is gone** (redirects to ABC News). Replaced by Our World in Data (home + topic page) and Stripe Press book pages for the lead figure.
3. **Kowalski:** cited only for what his page says: under 300 ms, "180 ms beats 400 ms", frequent and keyboard-repeated actions should not animate. The curve `cubic-bezier(0.23, 1, 0.32, 1)` is **our token** (`--ease-out`), not attributed to him; MDN (V) confirms the `ease-out` keyword is `cubic-bezier(0, 0, 0.58, 1)`, so ours is a deliberately stronger ease-out. **Why lock-in may animate despite his frequency warning:** it is the one signature moment of the draft surface, it runs at most 10 times per draft (picks only, never bans), a team drafts a few times a week, not hundreds of times a day; and it is suppressed exactly where he says (keyboard bulk entry, see the lock-in card).
4. swisstopo key is the **Zeichenerklaerung** PDF (V, rendered: `refs/key-a.png`, `key-b.png`).
5. The Pudding, Strava/FATMAP, Linear behaviour, Apple HIG body text: **dropped** (U). Reduced motion is cited from WCAG 2.3.3 and MDN instead (both V).
6. Lucide guide lives at `lucide.dev/contribute/icons/design-principles/`.

## 1. Field-sketch ink (T1)

| Ref | V/U | Take exactly this | Ignore |
|---|---|---|---|
| https://map.geo.admin.ch (colour 1:25k, `refs/map.png`) | V | Line hierarchy without weight noise: roads double-line, streams single, contours thinner; black ink for everything built, labels set *along* features; open line ends where features meet | Colour fills, forest green, relief shading, the UI chrome |
| https://map.geo.admin.ch grey layer (`ch.swisstopo.pixelkarte-grau`, `refs/map-grey.png`) | V | One ink only: every feature is line work in one tone, density makes the darks (rock hachure is just many strokes) | The rock hachure itself (figurative, busy at 390), the halo labels |
| swisstopo Zeichenerklaerung PDF https://www.swisstopo.admin.ch/dam/en/sd-web/WxsMJ4yE7xeV/Zeichenerklaerung_2008_e.pdf | V | Strict economy: white ground, black ink, one spot red; dashed vs solid carries meaning (railway, contour types) | Its colour swatches; Frutiger (we keep our faces) |
| Stripe Press social image (`refs/pressog.png`), https://images.stripeassets.com/fzn2n1nzq965/13zaWB3aiz08hqNr69CMW4/772586e910faef5d26d15e01edb9c010/social.png | V | A drawing made of parallel single-weight lines in one colour on near-black: proof that one stroke weight plus spacing reads as crafted, not clip-art | The yellow, the monogram |
| USGS "Field Notes" https://www.usgs.gov/media/images/field-notes | V (link-check opened; my headless fetch got 403) | Only the idea: a field sheet is annotated line work, not an illustration | Using it as a visual source (content not seen by me) |

## 2. Mark grid (nine marks)

| Ref | V/U | Take exactly this | Ignore |
|---|---|---|---|
| https://lucide.dev/contribute/icons/design-principles/ (`refs/lucide.png`) | V | Safe zone, round joins, round caps on open paths, corner radius even with round joins, 2 px gap between elements (scaled to 1.5 at 16), "keep density low" | Its 24/2 numbers as our master (see correction 1) |
| Phosphor README https://github.com/phosphor-icons/homepage (+ `refs/phosphor.png`) | V | "Designed at 16 x 16px": a family can be drawn for 16 first | Its six weights and duotone; we ship one weight |
| swisstopo Zeichenerklaerung (key-a, "Individual symbols") | V | Symbols are small, one weight, always in a row with their word; the same symbol at three scales stays the same construction | Its pictorial ones (castle, church) |
| https://en.wikipedia.org/wiki/Otl_Aicher | V | Method: one family "using a series of grid systems" | His figurative bodies; any claim of 45/90 construction (U) |
| In-game position icons | not fetched, by rule | **Must NOT resemble closely** (see grid spec) | Everything |

## 3. Lock-in stroke motion

| Ref | V/U | Take exactly this | Ignore |
|---|---|---|---|
| https://emilkowal.ski/ui/you-dont-need-animations (`refs/kowal.png`) | V | Under 300 ms; shorter feels faster; no animation on frequent or keyboard-repeated actions | Treating any curve as his |
| https://developer.mozilla.org/en-US/docs/Web/CSS/easing-function | V | `ease-out` = (0, 0, 0.58, 1); our `--ease-out` is the stronger variant: fast start, long settle | n/a |
| https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html | V | "Motion animation triggered by interaction can be disabled": reduced motion keeps the state change, drops the movement | Parallax examples |
| https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion | V | The media query is the switch; our global rule in `input.css` already cuts to 0.01 ms | n/a |
| https://sonner.emilkowal.ski | V (opened) | One transient element enters, nothing else on screen moves | Stacking claims (U) |
| animations.dev, Apple HIG, Linear | U | none | dropped |

## 4. Lead-figure typography (one figure, one sentence, one action)

| Ref | V/U | Take exactly this | Ignore |
|---|---|---|---|
| https://ourworldindata.org (`refs/owid.png`) | V | The number lives inside a sentence-headline ("A 73% average decline in wildlife populations"); a small caps overline above each entry; counters as figure + word | The navy, the yellow serif, pill counters, cards |
| https://ourworldindata.org/life-expectancy (`refs/owid-le.png`) | V | The lead paragraph is one claim with two figures ("32 years ... 71 years"), then one question; a reading measure of about 60 ch | Serif display (ours is Familjen), the tab bar |
| https://press.stripe.com/the-dream-machine (`refs/press-book.png`) | V | Title, byline, a short rule, a synopsis, then the one action (Purchase); nothing competes | The pastel gradient ground (DESIGN bans gradients), the 3D book stack on the home page |
| https://fivethirtyeight.com | U (dead) | none | replaced |

## 5. Open Graph card 1200x630

| Ref | V/U | Take exactly this | Ignore |
|---|---|---|---|
| https://linear.app/api/og/main?title=Linear&v=4 (`refs/linear-og.png`) | V, 1200x630 | One mark plus one word, centred, huge margin; per-title generated route | Its grey gradient |
| https://ourworldindata.org/default-thumbnail.png (`refs/owid-og.png`) | V, 1200x630 | Title + one sentence, set big; the map as a faint ground (about 1.2:1) behind, never competing | Navy, serif |
| Stripe Press social.png (`refs/pressog.png`) | V, 1200x675 | Line-drawing mark in one colour on near-black | 16:9 size: we ship 1.91:1 |
| https://vercel.com/docs/og-image-generation | V | 1200x630; server renderers want flex not grid and a small bundle; ttf/otf fonts | Vercel stack |
| GitHub social preview docs (link-check 5) | V | PNG under 1 MB | 2:1 |
| https://animations.dev (meta) | V | Declare `og:image:width` 1200 / `height` 630 and `twitter:card summary_large_image` | n/a |

## 6. Engraved contour (T2)

| Ref | V/U | Take exactly this | Ignore |
|---|---|---|---|
| grey 1:25k layer (`refs/map-grey.png`) | V | Contours are hairlines; index contours carry a figure set *along* the line ("3400"), the line broken under the figure | Rock hachure, glacier stipple |
| Zeichenerklaerung, "Topography" (key-a) | V | Three contour grades: ordinary, index (heavier, labelled), intermediate (dashed); spacing constant | Brown/blue per surface: we have one tint |
| colour 1:25k (`refs/map.png`) | V | Contours sit *under* everything and never compete with labels | Relief shading |
| https://en.wikipedia.org/wiki/Eduard_Imhof | V | Lineage only (Swiss school cartography) | Relief shading: gradients are banned (DESIGN 13) |

## 7. Paper still (C)

| Ref | V/U | Take exactly this | Ignore |
|---|---|---|---|
| Zeichenerklaerung (key-a, key-b) | V | White ground, black type, section heads with a full-width rule, three tidy columns, one spot colour | Its Frutiger |
| Stripe Press book page (`refs/press-book.png`) | V | Reading layout: title, rule, measure, one action | Pastel gradient |
| colour 1:25k (`refs/map.png`) | V | Paper white, near-black ink, one red for the important line | Fills |

---

# Recipe cards

Common to every card: radius 0 on artwork and frames; artwork is `aria-hidden` with the words beside it; nothing loops; the landing, share, Match Day, post-game and draft surfaces each get **one** signature and no other effect. Verification for every card: stills at 390x852 and 1280x800, contrast from pixels, side-by-side with the refs above.

## T1 field-sketch ink (landing 1280 + 390, share 390, Match Day 390)

- **Technique:** a schematic Rift built from geometry on a 100x100 unit square, never traced from a minimap screenshot or any Riot image. Elements: frame (four strokes stopping 1.5 units short of each corner, open corners = sketch), three lanes (top: (10,90) to (10,10) to (90,10); bot mirrored; mid (12,88) to (88,12)), river (two parallel lines from (8,8) towards (92,92), 3 units apart, broken for 4 units where mid crosses), jungle as sparse 45-degree hatching in each of the four quadrants (spacing 4 units, each hatch ending at least 2 units before a lane or the river, covering about 60 % of the quadrant: the tone fades by lines ending, never by opacity), bases as a small circle r3 at (12,88) and a 6-unit square at (85,9) (the side shapes, drawn in ink).
- **Tokens:** ground `page` (landing, share) or `ink-1` (inside the Match Day card); ink as defined above; `accent` only for the surveyed team label and its 1 px leader; edge ticks and their labels `muted`.
- **Stroke:** one weight for every line: 1 px at 390, 1.25 px at 1280 (`vector-effect: non-scaling-stroke`); round caps, round joins.
- **Type:** chalk label `eyebrow` role but in `accent` ("YOUR TEAM" on landing; the opponent's real name on a share page); coordinate labels A to H / 1 to 5 in `figures` 12 px `muted`, 1280 only.
- **390:** landing: `text-headline` title, one sentence (`text-reading`, `secondary`), one primary button, then the sketch full width (358 px square) under it. Share: a 358x96 crop of the river crossing above the `PageHeader`, no labels, no chalk unless it names the team the plan is about. Match Day: see field card.
- **1280:** two columns, text 5/12 left (top-aligned with the sketch), sketch 7/12 right at 560 px square with tick labels on two edges.
- **Signature:** the sketch itself, still. No draw-on animation.
- **Do not:** jitter or wobble filters to fake a hand; brush textures; more than one stroke weight; side colours or terrain fills (that is T3); a compass rose; parchment; chalk on anything but the team label; a second chalk item in the same viewport as the primary button on Match Day.

## T2 engraved contour (same surfaces)

- **Technique:** no figurative Rift. A contour field: 6 to 8 nested closed curves (slightly offset ellipses, like the Draft Graph's fit rings, 12 units apart), grades per the swisstopo key: ordinary `rule` 1 px, every fourth an index contour in ink at 40 % (3.5:1) carrying a `figures` 12 px label set along the line with the line broken 4 px either side of it, intermediate dashed `rule` 1 px (4 on, 4 off) only between the two innermost rings. The nine marks stand as spot points on the field, each with its word.
- **Tokens:** ground `canvas`; `rule`, ink 40 %, `muted` for labels; `accent` for the one surveyed team label.
- **Stroke:** 1 px contours at every width; marks per the grid spec.
- **390:** 358x240 field under the text, at most 4 marks visible. **1280:** 560 px square right column, all nine marks.
- **Signature:** the labelled index contour.
- **Do not:** hill shading or gradients; filled tint bands; hachure; any Rift outline; more than one labelled index contour per 200 px.

## T3 schematic (same surfaces)

- **Technique:** the T1 geometry as flat shapes: jungle quadrants filled `terrain-jungle`, river a band 8 units wide filled `terrain-river`, lanes 4-unit bands in `terrain-lane`, bases filled `side-blue-deep` (circle) and `side-red-deep` (square), on `canvas`, framed by a 1 px `rule` square. Straight polygon edges, sharp corners: "sharpened".
- **Contrast:** this is a ground, not ink (lane on canvas 1.36:1, measured); text never sits on it except the chalk label with a `canvas` halo.
- **Composition:** as T1. **Signature:** the river band.
- **Do not:** outlines around shapes; rounded blobs; textures; any second fill per element; shadows; terrain tokens behind running text.

## Lock-in (draft board pick slot; `DraftBoard`)

- **Trigger:** a champion fills a **pick** slot by pointer (click, drag) or a single Enter. Not bans, not swaps or moves, not loading or undoing a saved draft. If a lock-in follows the previous one within 1 s (keyboard bulk entry), it is instant (DESIGN 3.7: no animation on keyboard-repeated actions).
- **Motion:** a 2 px bar on the slot's bottom edge, `bg-accent`, `transform: scaleX(0 to 1)`, `transform-origin: left` (`right` on the red column), `220ms var(--ease-out)` (`--dur-3`). Same 220 ms: the slot's `RoleLabel` mark settles, `translateY(2px to 0)` and opacity 0 to 1. Transform and opacity only; nothing else on the board moves.
- **After:** the bar holds 400 ms, then its colour hands to `outline` over `--dur-2` (160 ms): chalk marks the moment, not a state, so ten finished picks never show ten chalk bars. The next step's dashed chalk edge appears at once, as today.
- **Reduced motion:** the bar appears at full width already in `outline`; the mark is in place; the step counter's live region still announces.
- **Verify:** 6-frame strip at 0, 40, 80, 120, 180, 220 ms (Playwright, animations paused and `currentTime` stepped), plus one frame at 800 ms; a reduced-motion still; a bulk-entry run proving no animation.
- **Do not:** springs or overshoot; glow; scaling the champion avatar; confetti or flashes; sound; animating the echo in the Draft Graph (one moment); durations over 220 ms.

## Match Day field card (390 first)

- **Content:** eyebrow "Next match" · lead figure: the start time via `LocalTime` ("19:00") in `figures` 500, `text-headline` (32 px) at 390, `text-display` (44 px) at 1280, `primary`; the day word beside it in `text-label` `muted` ("Today") · one sentence in `text-reading` `secondary`: "Scrim vs Riverside Rooks. 3 of 5 prep steps done." · one action: the prep sequence's next step as the page's single `Button` `Primary` (the `PrepSequence` below switches to `PrepEmphasis::Quiet`, so chalk appears once).
- **Art:** an 88x88 T1 sketch inset at the card's top right, ink only, no chalk, no labels.
- **Surface:** `Surface` `Ink1`, `Edge::RuleY`, `Pad::Md`, full width at 390 (358 px), max 40rem at 1280 in the left column.
- **Empty:** no match = the existing `EmptyState` ("Add the next match"), no sketch, no invented time.
- **Signature:** the lead figure. **Do not:** a countdown that ticks; more than one figure; a box around the figure; side colours in the inset; the opponent's name in chalk here.

## Post-game result stamp

- **Shape:** a square, radius 0, 64 px at 390 / 80 px at 1280. Win: `bg-win` with a W in `win-ink`, Familjen 600 at 40 / 48 px. Loss: 1.5 px `loss` outline, L in `loss`, no fill. A new `GlyphSize::Lg` on `ResultGlyph` (builder work), named "Win"/"Loss".
- **Beside it:** one sentence in `text-reading` ("Won in 31:20 on blue side.", the side with its `SideMarker`), then the page's one action.
- **Motion:** none; the stamp is present at first paint.
- **Do not:** rotation, ink-bleed or distressed texture (sepia rubber-stamp cliché), red for loss, side colours, a second stamp per page.

## OG card 1200x630

- **Layout:** `page` ground. Left 640 px: "Team Companion" Familjen 600 at 72 px `primary`, below it one sentence at 40 px `secondary` (minimum 40 px: Discord shows the card about 400 px wide on a phone, so 40 becomes about 13 px). Right 560 px: the T1 sketch, cropped to bleed off the right edge, ink at 70 %, the "YOUR TEAM" chalk label at 32 px. 64 px margin; nothing essential outside the centre 1100x530.
- **Files:** PNG under 1 MB (aim 200 KB), `og:image:width` 1200, `height` 630, `twitter:card summary_large_image`. Per-share titles may come later; the static card first.
- **Do not:** gradients, screenshots of the app, champion art, any Riot name or mark, more than one sentence.

## C paper still (draft only, `/s/{token}` at 390)

- **Token gap:** there is no paper token set. A builder proposes one in DESIGN 3.2 form (same names re-mapped under a wrapper class) before the still is rendered; I will not invent values. Constraints for that pass: paper near-white with chroma at most 0.01 in the green-slate hue family (no warm hue: no parchment, no sepia); ink near-black; `rule` hairlines; sides and results keep their meaning at 3:1 on paper; one chalk equivalent that clears 4.5:1 as text or 3:1 as a line (pale chalk fails on white).
- **Composition:** the share page as a printed sheet: `PageHeader` with a full-width rule under it (swisstopo key), reading measure, T1 strip in ink, no chrome.
- **Do not:** cream or yellowed paper, paper grain, deckled edges, serif for anything a person did not write.

## Mark grid spec

- **Canvas:** 16x16 master, 1 px safe zone (live area 1 to 15). 24 px = the master scaled 1.5x, stroke held at 2 px.
- **Keylines:** circle d 12 (centre 8,8); square 11x11 (2.5 to 13.5); diamond fits the 12 circle; triangle base 12, height 10.5.
- **Stroke:** 1.5 px centred, one width in all nine; round joins, round caps; rectangles get a 1 px corner radius; a dot is a circle r 0.75 stroked 1.5 (reads as a 3 px dot); at least 1.5 px between parallel strokes.
- **Colour:** `currentColor`, set to `secondary` beside labels (8.5:1+); never chalk, never side colours. Always beside the word (`RoleLabel` keeps TOP JGL MID BOT SUP); the mark is `aria-hidden`.
- **The motif is surveying instruments and map symbols, never lane geometry.** The in-game icons are built from map-corner L-shapes, a diagonal bar and crest/branch silhouettes; none of ours uses a corner L, a diagonal lane bar, wings, a helmet, antlers or a claw.

| Mark | Construction (16 grid) | Avoids the in-game icon by |
|---|---|---|
| Top | Triangulation point: triangle (2,13.5)-(8,3)-(14,13.5) with a dot at (8,10) | a summit, not a top-left corner L |
| Jungle | Three tree-crown circles r 2 at (4,6), (11,5), (8,11), each with a 2 px trunk tick (map "isolated tree" logic) | dots of trees, no branching or claw |
| Mid | Survey station: circle d 8 at centre, four 2 px ticks outside it on the axes | concentric and axis-aligned, no diagonal bar |
| Bot | Range: dot at (4,12), quarter arc r 9 from (4,3) to (13,12) | a reach arc, no bottom-right corner L |
| Support | Hold: dot at (8,6), a cup (semicircle r 5) below it from (3,9) to (13,9) | an open cup, no V crest or wings |
| Dragon | Objective diamond with one sine-like wave stroke across its middle | no head, wing or scale |
| Baron | Diamond with a smaller inner diamond (inset 3) | no horns, eye or purple |
| Herald / grubs | Diamond with three dots in a row inside | no eye, shell or creature |
| Tower | Lookout symbol: square 8x8 on a 12 px base line, dot in the centre | no battlements; stroke-only and based, so not the filled red-side square |

Risk: Top's triangle shares a shape family with the match timeline's filled "kill" triangle; the stroke-only rule and the dot keep them apart, judge it at 16 px in the gallery.

## Staleness flags

- CONTRADICTION: docs/DESIGN.md 6.3 and 7 ("no role icons ... nothing else draws a role") vs the owner's M151 ruling (nine marks beside labels): DESIGN needs an amendment in the same change that ships the marks.
- CONTRADICTION: docs/DESIGN.md 2 ("everything else is plain type ... no textures") and principle 14 (visionary in two places) vs landing/share/Match Day artwork: amend with the T-card scope.
- STALE: .planning/missions/M137-team-companion-first-patch/identity-proposal.md section 1: Lucide "16px grid, 1.5px stroke", FiveThirtyEight, the Kowalski curve and "Zeichenschluessel" are wrong or unverified (link-check corrections 1 to 4).
- STALE: docs/DESIGN.md 2 cites Strava/FATMAP as a closeness reference; no link was verified for it.
- Legal: whether the original marks are far enough from Riot's icons is a librarian question if anyone doubts the table above: ask the librarian.
