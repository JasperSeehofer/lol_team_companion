# M137 item 0b: identity proposal (PROPOSAL ONLY, 2026-10-14)

No code, no DESIGN.md change. Frozen reference for "what exists" is `input.css` and `docs/DESIGN.md` ("Survey"). Judged from the public-page stills and the spec; signed-in surfaces not seen live. Reference quotes are from my own knowledge of these products, not fetched today: before any mockup, a researcher pass must re-open each link and confirm the detail. Where a legal question appears (Riot assets), the answer is "ask the librarian".

Audience: semi-pro and new Prime League teams. They live in Discord and the game client (both dark, both evening), read op.gg / u.gg / Mobalytics / Leaguepedia / Oracle's Elixir all week, and are coached by one person who keeps a spreadsheet.

## 1. Reference sheet (first)

| Effect | 3 to 5 references | Take exactly this | Ignore |
|---|---|---|---|
| Dark ground and one accent | Linear (linear.app) | Tonal layers 2 to 4 steps apart, one accent spent on selection and the primary action; hairlines over boxes | Its Inter voice (DESIGN bans it), the purple |
| | Discord's own dark theme (the client the team already has open) | Familiarity of the darkness: they will not be dazzled switching tabs mid-scrim; same "never pure black" rule | Blurple, rounded 8px everything |
| | Bloomberg Terminal | One saturated hue on black used only for what must be read first; figures in a single mono face | Amber as brand; the density of the entire screen |
| | swisstopo maps (map.geo.admin.ch) and its printed sheets | The "Survey" source: thin contour tints, one red for the thing that matters, calm paper-and-ink order | Literal contour decoration on non-spatial pages (DESIGN already forbids it) |
| Type | FiveThirtyEight (Atlas Grotesk with Decima Mono) | A characterful grotesk for headlines and a mono for every number, as Survey does | Its all-caps kickers everywhere |
| | Stripe docs / Stripe Press (Sohne; Press titles) | Large display size, small tight UI size, generous measure for reading, nothing between | Marketing gradients |
| | Observable (Plot docs) | Tabular figures in running text so tables align | Notebook chrome |
| | Riot's own client (Beaufort for the Rift, Spiegel for UI) | What the audience already reads as "League"; proof that a serif capitals voice is the fantasy register DESIGN rejects | Do not use; also an asset question: ask the librarian |
| Density and tables | Oracle's Elixir (oracleselixir.com) tables | Analysts accept very dense, sortable, 24 to 28 px rows, stat columns right-aligned | Its visual untidiness |
| | Linear issue list | 32px rows, secondary text on the same line, a row is a link; status is a glyph plus word | Its grey-on-grey meta |
| | GitHub Primer "density" guidance | Two named densities (normal, condensed) switched per screen, not per theme | n/a |
| | op.gg / u.gg match history | The audience's mental model for a games list: result glyph, champion, KDA, time; one row per game | Ads, 5 different greens |
| Iconography | Otl Aicher's Munich 1972 pictograms | A tiny family drawn on one grid, one stroke, no outline fills: the right method for 5 role marks and 4 objective marks | Their figurative bodies |
| | Lucide / Phosphor icon sets (open licences) | 16px grid, 1.5px stroke, rounded joins; only what you can label | Using a generic set for domain concepts |
| | swisstopo Zeichenschluessel (symbol key) | Domain symbols as a legend you can read: filled shape plus text, never colour alone (Survey's blue circle / red square already does this) | n/a |
| | The in-game position icons (Top, Jungle, Mid, Bot, Support) | Players recognise them faster than TOP JGL MID BOT SUP; check whether a fan tool may redraw them: ask the librarian | Copying Riot artwork without that answer |
| Motion | Emil Kowalski, "Animations on the Web" and the Sonner toast | Custom ease-out (Survey's `cubic-bezier(0.23,1,0.32,1)` is already this), 150 to 250ms, toasts that stack and stay for an action | Springs for their own sake |
| | Linear | Sub-200ms transitions that explain state change (selection, row insert); no scroll-triggered reveal | n/a |
| | Rauno Freiberg (rauno.me) | One interaction crafted per surface (the single signature moment) | Cursor effects |
| | Apple HIG "Motion" | Reduced-motion substitutes (cross-fade or instant), not removal of the state change | n/a |
| Esports / Rift cues | lolesports.com broadcast overlays and the LEC/LCK stat panels | Tabular match-up header: side colours at the left/right edges, objective counts in small shapes | Hextech gold, chamfers, glow |
| | Leaguepedia | Plain, linkable evidence tables for every claim (matches DESIGN principle 9) | Wiki chrome |
| | Mobalytics | The counter-example: purple gradient, glass cards, avatar-heavy hero. Do not look like this | Everything visual |
| | The Summoner's Rift minimap itself | The only legitimate spatial frame; schematic (river diagonal, lanes, three jungle quadrants) in `terrain-*` tokens as the Draft Graph / Sand Table ground, already in `input.css` | Using it as wallpaper |
| Voice | GOV.UK content design | Plain verbs, say what happened and what to do, no humour on error pages | Bureaucratic tone |
| | Mailchimp Content Style Guide | A voice with permission to be warm on success and quiet on failure | Jokes |
| | Discord microcopy | The register the audience speaks: short, second person, no corporate "we are pleased" | Mascot |

## 2. Keep (with reason)

- **Dark-only for the working app.** The audience plays and reviews in the evening beside Discord and the client; AA contrast is machine-checked (DESIGN 3.2); one theme removed a whole class of bugs (the old hydration panic). Reason to keep: it fits where they are and costs nothing.
- **Tonal layers, no shadows, hairlines over boxes** (principles 1 and 4). It is the cheapest way to look expensive, and it is already enforced by lint.
- **Chalk as the single accent.** A pale yellow-green on blue-black reads like a surveyor's highlighter: distinct from blue/red side, from Mobalytics purple and from Discord blurple. It means "the one next action" and nothing else.
- **Sides as a blue circle and a red square, win as a teal W, loss as a hollow L.** Domain-correct and never colour-alone.
- **Serif italic only for what a person on the team wrote.** The best single idea in the identity: it is voice-as-typography.
- **Honest empties, "fit 78 of 100, from a neutral 50", the text twin of the Draft Graph.** This is the differentiator; no ranking site does it.
- **12px floor, keyboard-first, plain product language.**

## 3. Question (with reason)

1. **The identity lives in type alone.** The landing is four numbered columns and no picture (audit P3). Nothing says "League team" or "Prime League" within one second, and an invited player arriving from Discord on a phone gets a docs-site feel. Question: where is the one signature moment?
2. **Public Sans as the body voice.** A competent neutral (US government family) that adds no character at 13 to 14px; and four families plus italics cost 12 font files while the 8.6 MB WASM (audit F19) is the real weight problem. Question: is the body face earning its slot, or could Familjen Grotesk at 400 to 500 plus the mono carry everything (3 families)? Needs a rendering test at 13px, not an opinion.
3. **Density named "compact" but reading comfortable** (audit: 14px body, `mt-12` sections). The audience's daily tools (op.gg, Oracle's Elixir) are far denser; a coach reviewing 10 drafts will want more per screen. Question: a real `Density::Compact` default on tables and list pages.
4. **No role icons.** "TOP JGL MID BOT SUP" is clear, but the five position icons are the audience's native shorthand. Typographic marks are a defensible choice (no Riot art), yet a drawn five-mark set on one grid (Aicher method) is a free upgrade if the legal check passes.
5. **"Survey" metaphor vs audience.** The map idea is only used on canvases; players say "draft, scrim, review". The metaphor is invisible outside the Draft Graph and the Sand Table, which is fine by DESIGN 14, but the landing copy, "coaching workspace for one team", is generic. Question whether the product has a name-able look at all.
6. **Chalk on the sign-in button is the loudest thing a visitor sees,** for the one action most of them cannot take without an invite (audit F07). Identity choice and flow problem together.
7. **Dark-only and sharing.** A shared plan opened by a teammate on a phone in daylight, or printed (audit G10), is the one place where dark is a cost.

## 4. Three candidate directions (react, do not pick)

### A. "Survey, sharpened" (evolution, lowest risk)
References: swisstopo sheets, Linear, Aicher pictograms, Emil Kowalski.
Recipe:
1. Keep every token in `input.css` and every primitive.
2. Draw the five role marks and four objective marks on one 16px grid (1.5px stroke, no fills except side colour), shown beside the text label, not instead of it.
3. One signature moment: pick lock-in on the draft board and its echo in the Draft Graph, 220ms ease-out, nothing else moves; reduced motion = instant.
4. Landing: one real still of the board plus graph under the four steps (audit P3), plus an Open Graph card so a link pasted in Discord looks designed.
Smallest first step: the role-mark set in the gallery (`/design`), 5 SVGs, no page uses them yet; Jasper judges the five shapes at 16 and 24px.

### B. "Analyst sheet" (density-first, esports-native)
References: Oracle's Elixir, Linear lists, GitHub Primer density, Bloomberg Terminal restraint.
Recipe:
1. A second named density, Dense: 13px/1.35 body in tables, 28px rows, mono figures right-aligned with units in the header, sortable columns, sticky header.
2. Result glyph, role mark and champion avatar on one row grammar for every game and draft list.
3. Match Day becomes a table-first screen: next match, five confirmations, prep status in one row each.
4. Voice shortens: labels as nouns, no sentences in rows.
Smallest first step: switch `/stats` (Team games) to the Dense grammar behind no flag, desktop only, and compare against the current page at the same 1280px width.

### C. "Locker-room paper" (light companion theme for shared and printed things)
References: swisstopo printed sheets, field-manual layout, Stripe Press, a coach's printed match sheet.
Recipe:
1. Define a light "paper" token set that re-maps the same names (page, ink-1 to ink-3, rule, outline, accent) at AA, scoped by a class on the wrapper so it cannot leak into the app.
2. Use it only for `/s/{token}` pages, the print stylesheet (G10) and an optional phone "daylight" toggle later.
3. The shared plan opened from Discord reads as a printed sheet: the team name set in Familjen, side colours kept, no chrome.
Smallest first step: the public `/s/{token}` page only, plus the contrast pair table for it in DESIGN 3.2 form (so the existing token test can cover it); nothing else changes.

## 5. Open question (react, don't pick)

When an invited player opens a link from Discord on their phone, the first screen of Team Companion should feel most like: (A) a surveyor's field sheet you already see, tightened and given its own role marks; (B) an analyst's table, as dense as the sites they use every day; or (C) a printed team sheet that is dark in the app and paper when shared?

Suggested reactions (any freeform answer is welcome):
1. "A, and show me the five role marks first." (cheapest, nothing else moves)
2. "A for the app, plus C on the shared page only; skip B for now."
3. "B on the stats and list pages only; keep the rest as it is."
4. "None: the identity is fine, spend the effort on features (G01, G02, G03)." That is a legitimate answer; the identity audit found no blocker for the ~10-team beta.

## Staleness flags

- STALE: wiki references to role icons: none found in what I read; the librarian should confirm whether Riot's position icons may be redrawn in a fan tool before direction A step 2 (no wiki page I was given answers it).
