# Design system

The single source of truth for how Companion looks and behaves. `input.css` implements the tokens, `src/components/ui/` the primitives, `/design` shows both (the gallery), and `scripts/lint_design.sh` plus two Rust tests keep the code honest. When this file and the code disagree, fix one of them in the same change.

Status: M2 complete (stage A: tokens, type, fonts, lint gates, gallery, batch 1; stage B: batches 2 and 3, hard-zero lint, the page sweep).

## 1. Principles

Condensed from the M2 research (14 principles, binding) and the owner's decisions of 11 October 2026 (`MISSION.md` section 8).

1. **Type does the structuring.** Weight, size, case and space make the hierarchy before any rule or surface does. Draw a box only around something you can act on (a draft slot, a field, a token).
2. **Three faces plus one for notes, self-hosted, tabular figures on every number.** Nothing under 12px. Body 14px, reading 16px.
3. **One accent.** Survey chalk marks focus, selection, the primary action and the playhead. Saturation is spent on signals only. No gradients unless they encode data, no glows, no translucency over text.
4. **Tonal layers, not shadows.** Four green-slate layers a few OKLCH lightness steps apart; never pure black. Hairlines for structure, a 3:1 outline for anything interactive.
5. **Radius means something:** 0 for structure and tables, 3px for controls, round only for avatars, toggles and map tokens.
6. **Domain colours are a system.** Blue and red side are matched in lightness and chroma. Win and loss never reuse them. Every colour is paired with text, a glyph, a shape or a line style.
7. **Chrome recedes; content and canvases lead.** Quiet navigation, few and small icons, one header grammar on every page (eyebrow, title, meta, actions).
8. **Density is chosen per screen.** Compact for boards and tables, comfortable for briefs and reviews (60 to 75 characters, 1.5 to 1.6 leading).
9. **Every aggregate drills down to evidence, every suggestion carries a reason.** The Draft Graph always has a ranked-list twin.
10. **Keyboard first.** Shortcuts are shown as keycaps; a visible 2px focus outline everywhere.
11. **Motion explains, never decorates.** 100 to 300ms, ease-out, nothing loops; reduced motion swaps movement for an instant change.
12. **Honesty is visible.** Real empty states that say what will appear, uncertainty drawn as uncertainty (dashed estimates, solid anchors), plain tables for money.
13. **Avoid on sight:** card soup, uniform rounding, purple or blue gradients, glass, neon, chamfers, icon confetti, emoji, hero-plus-three-cards, coloured left stripes, Inter or Geist as the voice, fantasy capitals (Cinzel), fleur-de-lis, anything "gamer".
14. **Visionary in two places, familiar everywhere else.** The invention goes into the Draft Graph and the Sand Table; the rest is quietly excellent.

## 2. Direction: "Survey"

Companion is the team's field survey of the Rift and of every opponent. The map is the organising idea, but only where something is spatial:

- **Canvases** (Sand Table map, Draft Graph, map insets) carry a coordinate frame, a legend and, at most, a faint contour or grid texture.
- **Everything else** is plain type on hairlines: Match Day's prep list, action items, tables, settings, the community strip. No textures, no compass roses, no parchment.
- **Surveyor's notes** (the coach's annotations, with a leader line on a canvas) are set in serif italic. The serif appears nowhere else, so italic serif always means "a person wrote this here". The one exception off a canvas (owner's decision, M4): team notes quoted in the Draft Graph's text twin (the Why panel and the team's notes on pairs), which are also a person's words.
- **Dossiers** for opponents carry a "last surveyed" date and the evidence behind each finding.
- A compass rose or scale bar appears at most once per page, and only where distance means something (the Sand Table).

Not this: a fantasy map, sepia, Tolkien ornament, esports HUD, a developer tool. Closer to Swiss topographic sheets, field manuals and Strava/FATMAP than to a game launcher.

## 3. Tokens

All colours are OKLCH, defined once in `input.css` `@theme` (Tailwind turns `--color-x` into `bg-x`, `text-x`, `border-x` ...). Dark only in Release 1: `:root` sets `color-scheme: dark`; a light theme would redefine the same names later. There is no `data-theme` attribute any more.

### 3.1 Colour catalogue

| Token (Tailwind name) | Value | Role | Checked against |
|---|---|---|---|
| `page` | `oklch(0.180 0.018 200)` | Workspace behind everything | text 16.2 / 10.8 / 7.1 |
| `ink-1` | `oklch(0.205 0.020 200)` | App chrome, side rails, raised layer | text 15.5 / 10.3 / 6.7 |
| `ink-2` | `oklch(0.235 0.020 200)` | Fields, selected rows, second layer | text 14.4 / 9.5 / 6.2 |
| `ink-3` | `oklch(0.275 0.020 200)` | Hover and pressed | text 12.8 / 8.5 / 5.5 |
| `canvas` | `oklch(0.190 0.022 195)` | Map and graph ground | text 15.9 / 10.6 / 6.9 |
| `rule` | `oklch(0.340 0.016 200)` | Hairlines (structure only, 1.4 to 1.6:1, decorative) | n/a |
| `outline` | `oklch(0.580 0.020 200)` | Edge of every control | 3.5 to 4.4 on layers |
| `primary` | `oklch(0.950 0.008 190)` | Primary text | 12.8+ |
| `secondary` | `oklch(0.820 0.012 195)` | Secondary text | 8.5+ |
| `muted` | `oklch(0.700 0.014 200)` | Labels, overlines, metadata | 5.5+ |
| `accent` | `oklch(0.910 0.100 108)` | Survey chalk: focus, selection, primary action | 11.1+ on layers |
| `accent-hover` | `oklch(0.950 0.070 108)` | Primary button hover | |
| `accent-ink` | `oklch(0.240 0.030 110)` | Text on chalk fills | 12.3 on accent |
| `accent-soft` | accent at 10% | Selected or "selecting" wash | |
| `side-blue` | `oklch(0.700 0.120 245)` | Blue side, circle marker | 5.6+ |
| `side-red` | `oklch(0.700 0.120 28)` | Red side, square marker | 5.2+ |
| `side-blue-deep` / `side-red-deep` | `oklch(0.300 0.050 245 / 28)` | Token and badge fills | primary 11.8 / 12.1 |
| `win` / `win-ink` | `oklch(0.800 0.110 175)` / `oklch(0.220 0.030 175)` | Win: filled teal W | 9.6 |
| `loss` | `oklch(0.700 0.014 200)` | Loss: hollow neutral L | 6.7 |
| `success` | `oklch(0.790 0.120 155)` | Done, saved, ready | 9.0+ |
| `warning` | `oklch(0.800 0.140 78)` | Needs attention | 8.7+ |
| `danger` | `oklch(0.720 0.160 12)` | Errors, destructive actions | 6.2+ |
| `danger-deep` | `oklch(0.420 0.130 14)` | Danger button hover fill, with primary text | primary 7.86 |
| `info` | `oklch(0.790 0.080 215)` | Neutral information | 8.8+ |
| `terrain-jungle`, `terrain-river`, `terrain-lane`, `terrain-texture` | see `input.css` | Canvases only; texture is 1.1:1 on canvas | n/a |
| `scrim` | `oklch(0.140 0.016 200 / 0.88)` | Behind dialogs (opaque enough, no blur) | |

Legacy names stay as aliases so unmigrated pages restyle without edits: `base` = page, `surface` = ink-1, `elevated` = ink-2, `divider` = rule, `dimmed` = muted, `accent-contrast` = accent-ink, `overlay` / `overlay-strong` = scrim. New code uses the canonical names.

### 3.2 Contrast pairs (enforced)

The unit test `components::ui::tokens::tests::design_md_contrast_pairs_meet_their_minimum` parses this table and fails if any pair falls below its minimum; `rust_table_matches_input_css` keeps the Rust copy of the tokens (used by the gallery) equal to `input.css`. Minimums: 4.5 for text, 3.0 for UI boundaries, focus indicators and markers (WCAG 2.2 1.4.3 and 1.4.11).

<!-- contrast-pairs:start -->
| Foreground | Background | Minimum | Measured | Use |
|---|---|---|---|---|
| `primary` | `page` | 4.5 | 16.24 | text |
| `primary` | `ink-1` | 4.5 | 15.45 | text |
| `primary` | `ink-2` | 4.5 | 14.36 | text |
| `primary` | `ink-3` | 4.5 | 12.76 | text |
| `primary` | `canvas` | 4.5 | 15.93 | text |
| `secondary` | `page` | 4.5 | 10.78 | text |
| `secondary` | `ink-1` | 4.5 | 10.26 | text |
| `secondary` | `ink-2` | 4.5 | 9.54 | text |
| `secondary` | `ink-3` | 4.5 | 8.47 | text |
| `secondary` | `canvas` | 4.5 | 10.57 | text |
| `muted` | `page` | 4.5 | 7.06 | text |
| `muted` | `ink-1` | 4.5 | 6.71 | text |
| `muted` | `ink-2` | 4.5 | 6.24 | text |
| `muted` | `ink-3` | 4.5 | 5.54 | text |
| `muted` | `canvas` | 4.5 | 6.92 | text |
| `accent-ink` | `accent` | 4.5 | 12.70 | text on the primary button |
| `accent-ink` | `accent-hover` | 4.5 | 14.30 | text on the primary button |
| `accent` | `page` | 4.5 | 14.52 | selected text, focus ring |
| `accent` | `ink-1` | 4.5 | 13.81 | selected text, focus ring |
| `accent` | `ink-2` | 4.5 | 12.84 | selected text, focus ring |
| `accent` | `ink-3` | 4.5 | 11.41 | selected text, focus ring |
| `outline` | `page` | 3.0 | 4.42 | edge of controls (1.4.11) |
| `outline` | `ink-1` | 3.0 | 4.20 | edge of controls (1.4.11) |
| `outline` | `ink-2` | 3.0 | 3.91 | edge of controls (1.4.11) |
| `outline` | `ink-3` | 3.0 | 3.47 | edge of controls (1.4.11) |
| `side-blue` | `page` | 3.0 | 7.09 | side marker |
| `side-blue` | `ink-1` | 3.0 | 6.74 | side marker |
| `side-blue` | `ink-2` | 3.0 | 6.27 | side marker |
| `side-blue` | `canvas` | 3.0 | 6.95 | side marker |
| `side-red` | `page` | 3.0 | 6.67 | side marker |
| `side-red` | `ink-1` | 3.0 | 6.34 | side marker |
| `side-red` | `ink-2` | 3.0 | 5.90 | side marker |
| `side-red` | `canvas` | 3.0 | 6.54 | side marker |
| `primary` | `side-blue-deep` | 4.5 | 11.77 | label on a blue token |
| `primary` | `side-red-deep` | 4.5 | 12.05 | label on a red token |
| `win-ink` | `win` | 4.5 | 9.63 | W on the win fill |
| `win` | `page` | 3.0 | 10.51 | win glyph |
| `win` | `ink-1` | 3.0 | 10.00 | win glyph |
| `loss` | `page` | 3.0 | 7.06 | loss glyph edge |
| `loss` | `ink-1` | 3.0 | 6.71 | loss glyph edge |
| `success` | `page` | 4.5 | 10.17 | status text |
| `success` | `ink-1` | 4.5 | 9.67 | status text |
| `success` | `ink-2` | 4.5 | 8.99 | status text |
| `warning` | `page` | 4.5 | 9.88 | status text |
| `warning` | `ink-1` | 4.5 | 9.39 | status text |
| `warning` | `ink-2` | 4.5 | 8.73 | status text |
| `danger` | `page` | 4.5 | 7.01 | status text |
| `danger` | `ink-1` | 4.5 | 6.67 | status text |
| `danger` | `ink-2` | 4.5 | 6.20 | status text |
| `info` | `page` | 4.5 | 9.94 | status text |
| `info` | `ink-1` | 4.5 | 9.46 | status text |
| `info` | `ink-2` | 4.5 | 8.79 | status text |
| `primary` | `danger-deep` | 4.5 | 7.86 | text on the danger button (hover) |
<!-- contrast-pairs:end -->

### 3.3 Type

| Face | Role | Weights used | Notes |
|---|---|---|---|
| **Familjen Grotesk** | Display: page titles, section titles, tab labels, the next prep step | 600 (500 to 700 available) | `font-display`. Never below 16px. |
| **Public Sans** | UI and text: everything else | 400, 500, 600 | `font-sans`, set on `body`. Italic exists for emphasis in prose only. |
| **JetBrains Mono** | Figures, coordinates, overlines, keycaps, timers | 400, 500, 600 | `font-mono`, `figures` utility. |
| **Source Serif 4 italic** | Surveyor's notes (annotations on canvases, the coach's note on a moment) and team notes quoted in the Draft Graph (its Why panel and the team's notes on pairs: the graph's text twin) | 400 | `note` utility. Nowhere else: italic serif always means "a person on the team wrote this". |

Self-hosted woff2 in `public/fonts/` (latin and latin-ext files per style, split with `unicode-range`), `font-display: swap`, the two critical files (Public Sans and Familjen Grotesk latin) preloaded in the shell. Licences: `public/fonts/LICENSES.md` (all OFL 1.1). Fallback stacks: system UI sans, `ui-monospace`, Georgia.

**Scale** (ratio about 1.2; utilities generated from `--text-*`):

| Utility | Size / leading | Use |
|---|---|---|
| `text-caption` | 12 / 1.4 | Help text, captions, metadata. The floor: nothing smaller, anywhere. |
| `text-label` | 13 / 1.4 | Labels, buttons, dense rows, table text |
| `text-body` | 14 / 1.5 | UI default (set on `body`) |
| `text-reading` | 16 / 1.6 | Briefs, reviews, long notes (measure 60 to 75 ch) |
| `text-heading` | 20 / 1.3 | Sub-section and panel titles (display face) |
| `text-title` | 24 / 1.2 | Section titles, `<h2>` (display face) |
| `text-headline` | 32 / 1.1 | Page titles, `<h1>` (display face) |
| `text-display` | 44 / 1.05 | One per page at most (landing, closed beta) |

Role utilities: `eyebrow` (the overline role: mono 12px, 500, +0.08em tracking, caps, muted), `figures` (mono, tabular), `note` (serif italic). The `Text` primitive maps roles to these. Tailwind's own `text-xs` and up still exist (all at or above 12px); arbitrary `text-[Npx]` below 12px fails the lint.

**Figures:** `body` sets `font-variant-numeric: tabular-nums`, so every number lines up (all three sans and serif faces have `tnum`; the mono is monospaced). Use `figures` for numbers that are the point (scores, fit, clock, gold).

**Weights on dark:** 400 and up; light weights bloom.

### 3.4 Spacing and density

Tailwind's 4px scale. The common steps are 4, 8, 12, 16, 24, 32, 48, 64. Boards and tables are compact (8 to 12px row padding, `Density::Compact`); briefs and forms are comfortable (24 to 32px between groups). `PageShell` sets 16px side gutters on phones and 32px from `sm`, so no page scrolls sideways at 400px.

### 3.5 Radius

| Token | Value | Where |
|---|---|---|
| none | 0 | Structure: page bands, panels, tables, rules, canvases |
| `rounded-ctl` (and every named Tailwind radius) | 3px | Controls: buttons, fields, selects, badges, keycaps, draft slots |
| `rounded-full` | round | Avatars (`UserAvatar`, round `ChampionAvatar`), toggles (`Switch`), map tokens, side markers. Only inside primitives (lint). |

`rounded-sm` to `rounded-4xl` all resolve to 3px, so legacy `rounded-xl` boxes no longer read as "everything rounded"; M3 removes the boxes themselves.

### 3.6 Elevation and layers

Elevation is tone: page, then ink-1, ink-2, ink-3, each about 0.03 L lighter. There is one shadow, `shadow-overlay`, for floating things (toasts, menus, dialogs). All other Tailwind shadows are switched off in `@theme`.

Layers (CSS variables, use as `z-(--z-nav)`): `--z-nav` 30, `--z-popover` 40, `--z-dialog` 50, `--z-toast` 60.

### 3.7 Motion

| Token | Value | Use |
|---|---|---|
| `--dur-1` | 100ms | Press, colour change |
| `--dur-2` | 160ms | Hover reveal, tooltip, toggle (Tailwind's default transition duration) |
| `--dur-3` | 220ms | Popover, tray |
| `--dur-4` | 300ms | Panel, route content |
| `ease-out` | `cubic-bezier(0.23, 1, 0.32, 1)` | Entering and leaving (Tailwind default timing) |
| `ease-in-out` | `cubic-bezier(0.77, 0, 0.175, 1)` | Movement on screen |

No ease-in for UI, nothing loops (the only animations left are the skeleton pulse and the loading spinner), no animation on keyboard-repeated actions. **Reduced motion:** `@media (prefers-reduced-motion: reduce)` in `input.css` cuts every animation and transition to an instant change; primitives also add `motion-reduce:animate-none`.

## 4. Colour rules

- **Accent:** chalk is for focus, the selected tab, the step being decided ("selecting"), the playhead and the one primary action per view. Not for decoration, headings or icons.
- **Sides:** blue is a circle, red is a square (markers, tokens, badges). Always with a label or lane. Never use the side colours for anything but sides.
- **Results:** win is a filled teal "W", loss a hollow neutral "L". They never use blue or red, so "red side" and "loss" cannot be confused. Accessible names are "Win" and "Loss".
- **Status:** success, warning, danger and info always come with words (and a shape in `Badge`). Danger is a crimson distinct from the red side, and appears only for errors and destructive actions. The danger button is an outline in danger; on hover it fills with the dark `danger-deep` and primary text, never a pastel fill. Mapping old colours: map by meaning (an error is danger, a side label is a side token, a result is win/loss, a state is a status), never "red means red side" or "red means danger".
- **Never colour alone** (WCAG 1.4.1): every encoding has a second channel (text, glyph, shape, line style, position).
- **Raw Tailwind palette classes** (`text-red-400`, `bg-blue-500`, `text-white`) are banned in `src/` (lint, hard zero).

## 5. Canvas rules

Applies to the Sand Table map, the Draft Graph, map insets and charts (M4, Release 2; components reserved in batch 3).

- **Coordinate frame:** tick-marked edges with real numbers (Sand Table 0 to 1000; Draft Graph grid A to H by 1 to 5) so a coach can say "Rakan, D4" on voice. Charts outside canvases get plain axes, not frames.
- **Legend:** every canvas that encodes colour, shape, line style or width has a legend block next to it, with direct labels where possible. On the Sand Table the overlays are the legend (switches, at most three active).
- **Texture:** contours or grid at about 1.1:1 against `canvas`, only inside the frame, never behind running text. Labels on canvases get a canvas-coloured halo.
- **Uncertainty:** estimated paths dashed, exact events as solid anchors, approximate areas as soft dashed blobs.
- **Accessibility:** a canvas is one tab stop with a text twin (the ranked reasons list for the graph, the readouts and moments for the map).

## 6. Primitives

Location: `src/components/ui/`, re-exported from `components::ui`. Every component has typed variants (Rust enums), class names written out in full (Tailwind only sees literal strings; enums map to `&'static str` in `match` arms), identical markup on server and browser, a forced-state hook for the gallery (`force: Option<Force>` on interactive primitives, rendered as `data-force`), and a doc comment whose `A11y:` line states its accessibility contract (`components::ui::tests::every_component_documents_its_a11y_contract` fails otherwise). Focus is the global rule in `input.css` (`:focus-visible`, 2px chalk outline, 2px offset); primitives do not repeat focus classes.

### 6.1 Batch 1 (in the gallery)

| Primitive | Props and variants | A11y contract |
|---|---|---|
| `PageShell` | `width: PageWidth {Narrow 28rem, Reading 48rem, Standard 64rem, Wide 80rem (default), Full}`, `density: Density {Compact, Comfortable}`, `class` | A `div` inside the shell's single `<main id="main">` (the shell has the skip link). 16px phone gutters. |
| `PageHeader` | `title` (text or a closure, so an editor's document name stays live), `eyebrow?`, `meta?: ViewFn`, `actions?: ViewFn` | The page's only `<h1>`; actions follow it in tab order. |
| `SectionHeader` | `title`, `level: HeadingLevel {H2, H3}`, `eyebrow?`, `meta?`, `actions?`, `id?` | A real `h2`/`h3`; `id` for `aria-labelledby`. |
| `Surface` | `layer: Layer {Inherit (default), Page, Ink1, Ink2, Ink3, Canvas}`, `edge: Edge {None (default), RuleTop, RuleY, Rule, Object}`, `pad: Pad {None, Sm, Md, Lg}`, `tag: SurfaceTag {Div, Section, Article, Aside}`, `label?`, `labelled_by?`, `class` | Landmarks get a name; `attr:` attributes (for example `data-feedback-label`) land on the root. Undecorated by default. |
| `Text` | `role: TextRole {Display, Page, Title, Heading, Reading, Body (default), Label, Caption, Overline, Figure, FigureLarge, Note}`, `tone: Tone {Inherit, Primary, Secondary, Muted, Accent, Success, Warning, Danger, Info}`, `tag: TextTag {Span, P, Div}` | Never a heading; every tone 4.5:1 on every layer; nothing below 12px. |
| `Button` | `variant: ButtonVariant {Primary, Secondary (default), Quiet, Danger}`, `size: ButtonSize {Sm 28px, Md 32px, Lg 40px}`, `kind: ButtonType {Button, Submit, Reset}`, `disabled`, `loading`, `on_click`, `href` (renders a link), `icon` slot, `shortcut` (keycap), `aria_label`, `full_width`, `pressed: Option<Signal<bool>>`, `force` | Native `<button type="button">`; loading disables and sets `aria-busy`; `pressed` makes it a toggle button (`aria-pressed`, filled while pressed: the selected row of a list); keycap hidden from screen readers, `aria-keyshortcuts` set; icon-only buttons need `aria_label`. Targets at least 24px. |
| `Keycap` | `keys` | A `<kbd>`. |
| `Field` | `id`, `label`, `hint?`, `error: Signal<Option<String>>`, `required` | Visible label bound with `for`; hint and error via `aria-describedby`; `aria-invalid` on the control; error announced (`role="alert"`); "(required)" written out. Provides `FieldContext` to the control inside. |
| `TextField` | `value?: RwSignal<String>`, `kind: InputKind {Text, Email, Password, Search, Number, Date, Url}`, `size: FieldSize {Sm 32px, Md 36px}`, `name`, `placeholder`, `autocomplete`, `minlength`, `maxlength`, `required`, `disabled`, `rows` (textarea), `id`, `aria_label`, `on_keydown`, `on_blur`, `combobox: Option<Combobox {controls, active}>`, `force` | Takes id, description and invalid state from `Field`; 3:1 outline edge; `prop:value`. With `combobox` the input is `role="combobox"` with `aria-controls`, `aria-expanded` and `aria-activedescendant` (the champion search fields). |
| `Select` | `value?`, `options: Vec<SelectOption>`, `size`, `name`, `disabled`, `id`, `aria_label`, `force` | Native `<select>` (type-ahead, arrows); chevron decorative. |
| `Badge` | `tone: BadgeTone {Neutral, Accent, Success, Warning, Danger, Info, SideBlue, SideRed, Win, Loss}`, `label?` | Meaning in words; marker shape repeats it (hidden from screen readers); `label` gives W/L glyphs the name "Win"/"Loss". |
| `Table` (+ `Th`, `Tr`, `Td`, `RowLink`) | `caption`, `caption_hidden`, `density`, `sticky_header`; `Th {align, numeric, sort}`, `Td {numeric, muted, header, align}`, `Tr {selected}`, `RowLink {href}` | Real `<table>` with `<caption>`; scroll wrapper is a named focusable region; `th scope`, `aria-sort`; numbers right-aligned in tabular figures; row link is one tab stop. |
| `Tabs` (+ `TabPanel`) | `group`, `label`, `tabs: Vec<TabItem {id, label, count?}>`, `selected: RwSignal<String>` | WAI-ARIA tabs: roving tabindex, Left/Right (wrapping), Home/End, automatic activation, `aria-selected`, `aria-controls`; panels hidden, not removed (SSR-safe). |

Domain primitives shipped early (owner's decisions):

| Primitive | Props | A11y contract |
|---|---|---|
| `DraftBoard` | `id`, `label`, `champions: Signal<Vec<Option<String>>>` (index = order - 1), `roles`, `guessed` (per slot: the role is a guess), `notes`, `warnings`, `current: Signal<Option<usize>>`, `blue_team`, `red_team`, `on_select`, `on_open`, `on_clear`, `on_drop`, `hints` (keycap legend), `format: DraftFormat` | `role="grid"` named by `label`: a header row, the ban row and five pick rows of `role="gridcell"` slots, one Tab stop (roving tabindex). Each slot's name says its step, rotation, champion, role, note and state ("Red pick 2, rotation 2: Syndra, Mid"); the current step has `aria-current="step"`; the step counter is a polite live region. Read-only without `on_select`. Full specification in section 6.5. |
| `PrepSequence` | `label`, `steps: Vec<PrepStep {title, detail?, status?, state: PrepState {Done, Next, Todo, Attention}, href?, action?}>` | Ordered list; status in words; the next step has `aria-current="step"`, a larger display title and a real link to where it is done. The Match Day "what to do next" pattern from direction B, translated into quiet type. |

The older feedback components became batch 2 primitives or shims for them (section 6.4).

### 6.2 Batch 2: interaction (stage B, in the gallery)

| Primitive | Props and variants | A11y contract | Use it for / not for |
|---|---|---|---|
| `Dialog` | `id`, `open: RwSignal<bool>`, `title`, `description?`, `size: DialogSize {Sm 24rem, Md 32rem (default), Lg 44rem}`, `dismiss_on_scrim` (default true), `footer?: ViewFn`, children (optional) | Native `<dialog>` opened with `showModal()`: focus moves in (first `autofocus` element, else the first focusable), Tab stays inside, the page behind is inert, Escape closes. `aria-modal`, `aria-labelledby` (the `<h2>` title), `aria-describedby` (the description). Focus returns to the element focused before it opened. Opaque `scrim` backdrop, no blur; page scroll locked while open. | Short forms and decisions that block the page. Not for information that can sit inline (`Callout`) or for navigation. |
| `ConfirmDialog` | `id`, `open`, `title`, `message` (the consequence), `confirm_label` (names the action: "Delete plan"), `cancel_label` ("Cancel"), `tone: ConfirmTone {Default, Destructive}`, `on_confirm: Callback<()>` | A `Dialog` (Sm). Destructive: the confirm button is `Danger` and Cancel has initial focus, so Enter by habit destroys nothing; Default: the primary confirm has focus. | Every destructive action that cannot be undone. Prefer undo (a `Toast` with "Undo") where the data allows it. |
| `Menu` | `id`, `label`, `items: Vec<MenuItem {id, label, href?, shortcut?, danger, disabled}>`, `on_select: Callback<String>`, `align: MenuAlign {Start, End}`, `variant`/`size` (trigger, as `Button`), `trigger?: ViewFn`, `force_open` (gallery) | WAI-ARIA menu button: trigger `aria-haspopup="menu"`, `aria-expanded`, `aria-controls`; list `role="menu"` labelled by the trigger; items `role="menuitem"` (real buttons or links, `tabindex=-1`). Enter, Space or Down open and focus the first item, Up the last; Up/Down move (wrapping, skipping disabled), Home/End, a letter jumps (typeahead), Escape closes and returns focus, Tab closes, a click outside closes. A custom trigger gets `label` as its name. | A short list of actions on one object (plan actions, the account menu). Destructive items sit last under a rule. Not for choosing a value (`Select`, `SegmentedControl`). |
| `CommandBar` | `id`, `open: RwSignal<bool>`, `commands: Vec<Command {id, label, group, keywords, href?, hint?}>`, `on_run: Callback<String>` (commands without a link), `global_shortcut` (default true) | A modal `<dialog>` with an input `role="combobox"` (`aria-expanded`, `aria-controls`, `aria-activedescendant`, `aria-autocomplete="list"`) and a `role="listbox"` of `role="option"` links in named `role="group"`s. Up/Down move the active option (wrapping), Enter follows it, typing filters (`filter_commands`: every word must match; prefix matches first), the result count is announced politely, Escape closes and focus returns. Ctrl+K and Cmd+K toggle it; the trigger carries `aria-keyshortcuts="Control+K Meta+K"`. | The nav mounts one for signed-in users with every hub route (`route_commands`); M3/M5 add real commands. One per page with the global shortcut. |
| `Toast` (`ToastProvider`, `ToastContext {show, push, dismiss}`, `Toast {kind, message, action?}`) | `ToastKind {Success, Info, Warning, Error}`; `Toast::success/info/error/new(kind, msg).with_action("Undo", cb)`; `toast.show.run((kind, msg))` stays source-compatible | One polite live region (`aria-live="polite"`) that exists from the first paint, in a landmark named "Notifications", bottom left. Toasts never take focus. Success and info leave after 5 s, with an action or a warning after 8 s, errors stay until dismissed; hover or focus pauses the timer (it restarts on leave). At most three; the oldest non-error goes first. The action and dismiss are real buttons of 28px. The kind is said in words ("Done:", "Error:"). | Something that just happened, with at most one action. "Undo" lives here (team notes, M5). Not for lasting state (`Callout`) or for errors the user must fix in a form (`Field` error). |
| `Callout` | `tone: CalloutTone {Info, Success, Warning, Danger}`, `title?`, `action?: ViewFn`, children | Danger and warning are `role="alert"`, info and success `role="status"`. The tone is in words (visually hidden prefix "Warning:") and in a shape (circle "i", circle check, triangle "!", square "!"), never colour alone; warnings always show the "!" glyph next to the text. `title` is a bold first line, not a heading. One action at most. | Inline, lasting messages that belong to a place. `ErrorBanner` and `StatusMessage` are compatibility names for it. |
| `EmptyState` | `message` (what will appear and how), `title?`, `level: Option<HeadingLevel>` (default h3), `size: EmptySize {Section, Compact}`, `cta_label` + `cta_href` (shorthand link) or `action: ViewFn` | Plain text, no icon or emoji. A title is a real heading at `level`, so the empty section stays in the outline. At most one action, a real link or button. | Every place with no data yet (honesty rule): say what will appear and how it gets there. Never fake rows. `NoTeamState` is the team-page instance. |
| `Skeleton` (+ `SkeletonGroup`) | `shape: SkeletonShape {Line (3px), Block (square, rule edge), Circle}`, `width` and `size` (literal Tailwind classes from the call site), `lines` (Line only; last line shorter); `SkeletonGroup {label, children}` | Shapes are `aria-hidden`. `SkeletonGroup` is `role="status"` with `aria-busy` and a visually hidden label ("Loading drafts"), so screen readers hear one status. Pulses only while loading; under reduced motion the shapes stay still (`motion-reduce:animate-none`). `ui/skeleton.rs` is the only file where `animate-pulse` is allowed (lint). | Suspense fallbacks shaped like the content. `SkeletonLine/Card/Grid` are compatibility names. |
| `Tooltip` | `id`, `text`, `placement: TooltipPlacement {Top, Bottom}`, `force_open` (gallery), one child | Bubble `role="tooltip"`; after hydration the wrapped control gets `aria-describedby` pointing at it. Shows on hover and on keyboard focus after 300ms, Escape hides it until the pointer or focus leaves. Never focusable, never interactive content. | Hints that repeat or explain what is on screen (an icon button's purpose, a shortcut). Never the only place information lives: phones have no hover. |
| `SegmentedControl` | `id`, `label` (group name), `options: Vec<Segment {value, label}>`, `value: RwSignal<String>`, `size: SegmentSize {Sm 24px, Md 28px}`, `on_change`, `disabled`, `manual`, `force` | `role="radiogroup"` with a name; segments `role="radio"` with `aria-checked`. Roving tabindex (one Tab stop, the checked segment); Left/Right and Up/Down move and select (wrapping), Home/End. With `manual` the arrows only move focus and Enter, Space or a click selects: for switches with a side effect (the nav's Solo/Team workspace reloads data). The checked segment is marked by fill and weight. | Two to five short options that switch a view or mode at once (Solo/Team, first pick Blue/Red). Not tabs with panels (`Tabs`), not long lists (`Select`). |
| `Checkbox` | `label`, `checked: RwSignal<bool>`, `strike` (cross the label out while checked), `disabled`, `on_change`, `id` | A native `<input type="checkbox">` inside its `<label>`: the text is the name, Space toggles, the state is the box (mark and fill); the strike is decorative. Square (round is for switches). | Checklist items (game plan, action items) and yes/no inside a form. Not for settings that apply at once (`Switch`). |
| `Switch` | `label`, `checked: RwSignal<bool>`, `label_hidden`, `show_state` ("On"/"Off"), `disabled`, `on_change`, `force` | `<button role="switch">` with `aria-checked`; Space and Enter toggle. The visible label is the name (`aria-label` when hidden). Track and thumb are round (a toggle); the state is position plus fill, and in words with `show_state`. | Settings that apply immediately. Inside a form that is submitted, use a checkbox. |
| `Keycap` | `keys` | A `<kbd>` (mono). Buttons draw theirs through `shortcut` and set `aria-keyshortcuts`; a keycap is never the only way to learn a shortcut a control exposes. | Shortcut legends in text, buttons and the command bar. |

### 6.3 Batch 3: domain (stage B, in the gallery)

| Primitive | Props and variants | A11y contract | Use it for / not for |
|---|---|---|---|
| `ChampionAvatar` | `name` (display name or Data Dragon id), `size: AvatarSize {Xs 20, Sm 24, Md 32 (default), Lg 40, Xl 56}`, `shape: AvatarShape {Square (3px, default), Round}`, `side: Option<DraftSide>` (overrides shape: blue round, red square, with the side colour), `state: AvatarState {Normal, Banned, Dimmed}`, `decorative` | An `<img>` from `/img/champion/{name}/icon` only (never a third party), alt text = the name ("Azir", or "Azir, banned"); `decorative` empties the alt when the name is written next to it. A two-letter monogram (`monogram()`: "Lee Sin" LS, "Azir" Az) sits underneath while the image loads and stays when it is missing (404 or failed before hydration), hidden from screen readers. Banned adds a strike and the word, not only grey. Fixed size, `loading="lazy"`. | Champion art everywhere (lists square, tokens round). Pages still using raw `<img>` move onto it in M3/M5. |
| `UserAvatar` | `name`, `size`, `label?` | The initial in a round token. Decorative unless `label` is set (then `role="img"` with that name). | The nav account menu, roster, comments. |
| `RoleLabel` | `role: Role {Top, Jungle, Mid, Bot, Support}` (`Role::parse` takes every stored spelling: jgl, middle, adc, utility...; `Role::key` is the stored spelling), `style: RoleLabelStyle {Plain, Boxed, Active}`, `guessed` | Mono caps TOP, JGL, MID, BOT, SUP; the letters are `aria-hidden` and the word ("Jungle") is read. Text, so it scales and never breaks. A `guessed` role (inferred, not set by a person) is drawn as uncertain, a dashed underline and a "?" (`data-role-guessed`), and read as "Jungle, role guessed". | Every role mark. Role icons from CommunityDragon are gone; nothing else draws a role. |
| `SideMarker` | `side: DraftSide`, `text: SideText {None, Short "Blue", Long "Blue side"}` | Blue circle, red square in the matched side colours; the shape is decorative, the side is always in text (visible or visually hidden). | Side headers and labels. Side colours appear nowhere else. |
| `ResultGlyph` | `result: GameResult {Win, Loss}` (`GameResult::from_win`), `size: GlyphSize {Sm 20, Md 24}`, `word` | `role="img"` named "Win"/"Loss"; filled teal W, hollow neutral L, never the side colours. `word` writes the word instead of the letter. | Results in history, stats, dashboards (`Badge` W/L tones remain for inline pills). |
| `StatReadout` | `label`, `value`, `unit?`, `delta: Option<Delta {value, trend: Trend {Up, Down, Flat}, sentiment: Sentiment {Good, Bad, Neutral}, against}>`, `size: ReadoutSize {Sm 14, Md 24, Lg 32}`, `slots` (characters reserved), `empty_reason?` | A `<dl>`: label `<dt>` (eyebrow), figure `<dd>` (mono, tabular). The change is text with its direction in words for screen readers ("up 4.2 vs last 10"); the arrow is decorative; colour follows good/bad, never direction or side. No data shows an em dash and the reason. | Headline numbers (win rate, KDA, gold at 15). Not for tables (`Table` numeric cells). |
| `LegendBlock` | `id`, `title` ("Legend"), `entries: Vec<LegendEntry {swatch, label}>`, `columns`; `Swatch {Line {style: LineStyle {Solid, Dashed, Dotted}, weight: LineWeight {Thin, Medium, Thick}, ink: SwatchInk, arrow}, Widths(ink), Area {soft, ink}, Anchor(ink), Sides, Side(side), Contours}` | A list labelled by its visible title; every entry is words, the swatch drawing is `aria-hidden`. Swatches differ by shape, line style and width, not only hue; all inks meet 3:1 on the canvas. | Next to every canvas that encodes colour, shape, line style or width (canvas rules, section 5). |
| `CanvasFrame` (+ `ScaleBar`) | `caption`, `grid: FrameGrid {Numeric {max, step} (vertical axis counts up), Reference {cols, rows} (A to H by 1 to 5)}`, `texture` (faint grid inside), `scale?: ViewFn`, `aspect` (class, default 16/10), children; `ScaleBar {label, segments}` | A `<figure>` named by its `<figcaption>`. Edge numbers and ticks are decorative (they hang off the canvas box, so they always line up). The canvas inside must be one tab stop with a text twin, which the caption names. Texture about 1.1:1, only inside the frame. `ScaleBar` is `role="img"` "Scale: 200 units", at most once per page. | Reserved for the Sand Table and the Draft Graph (M4). No behaviour yet. Never around plain charts. |
| `Annotation` | `x`, `y` (anchor in percent of the canvas), `placement: NotePlacement {NorthEast (default), NorthWest, SouthEast, SouthWest}`, `title?` (mono, "17:00 Pick-off"), `at?` (grid reference read to screen readers), children (the note) | `role="note"` with real text in reading order; `title` and `at` are written out so the note stands without the drawing; anchor dot and leader line decorative. Serif italic (the `note` utility): "a person wrote this here". | Canvases only. Never elsewhere: the serif means a coach's annotation. |
| `TimelineScrub` | `label`, `duration` (s), `value: RwSignal<u32>`, `moments: Vec<Moment {at, label}>` | The ruler is one tab stop, `role="slider"` with `aria-valuemin/max/now` (seconds) and `aria-valuetext` ("17:05, moment: Pick-off"). Left/Right 1 s (Shift 10 s), PageUp/PageDown 60 s, Home/End, J/L previous/next moment (K is reserved for pause once playback exists); a click seeks. Moments are also a visually hidden list. On phones flags show the time only and every other scale label drops. No play button until there is playback. | Reserved for the Sand Table (M4). Static data today. |

Shared keyboard helpers live in `ui/a11y.rs` (`roving_index`, `typeahead`, `focus_id`, `click_id`), unit-tested so the contracts above hold without a browser.

### 6.4 Compatibility shims

`src/components/region/` keeps the names the solo dashboard and match detail still call (`Card`, `SectionHead`, `Eyebrow`, `ModeToggle`, `RankBadge`, `LPProgress`) without the `region` prop; `ModeToggle` renders a `SegmentedControl`. `ChampTile` and `ChampPortrait` were deleted in M3 with their last callers. In `components::ui`, `ErrorBanner` and `StatusMessage` render `Callout`, and `SkeletonLine`, `SkeletonCard`, `SkeletonGrid` render `Skeleton`; `components::skeleton::PageLoading` and `PageEmpty` are built from primitives. The strategy pages no longer use any of them; their remaining callers are M5 pages (home, team dashboard, roster, profile, stats, match detail, personal learnings, solo dashboard, sign-in and registration, the parked team builder). Each is deleted when its last caller is gone.

### 6.5 Draft board

There is one draft view, the tournament board, used by the draft page, the tree drafter (editor and live view) and the gallery example (the same component). `DraftFormat` is an enum whose only member is `Tournament`, so another format (Flex Queue's simultaneous bans) can be added later without a rewrite; there is no format toggle.

- **Layout.** Two columns, blue left and red right; each column is that team's final draft. The column header names the side (circle or square plus the word) and the team. At the top, five bans per side grouped 3 + 2 by ban rotation (B R B R B R, then R B R B); below, five picks per side in pick order, bracketed by pick rotation: B1 · R1 R2 · B2 B3 · R3 · R4 · B4 B5 · R5. Blue pick 1 carries a quiet "First pick" caption while empty.
- **State.** Only the step being decided is lit (chalk dashed edge on `accent-soft`); filled slots sit on `ink-1` with an outline, empty ones are dashed. A step counter above the board says "Pick 7 of 10 · Red to pick" (or "Draft complete", or "12 of 20 steps decided" for a partly filled saved draft).
- **Slots.** A pick slot shows the champion avatar, the name, the role (`RoleLabel`; a guessed role is drawn as uncertain, "MID?" with a dashed underline, and the slot reads "Mid (role guessed)") and an optional note or warning (with the "!" glyph) under the name. Ban slots are small squares with the avatar struck through. Roles are guessed only for picks nobody has set a role on: the champion's roles from the trait table, preferring roles its side has not filled, else its class; choosing a role in the slot details confirms it, and the guess flag is stored (`draft_action.role_guessed`).
- **Size.** The compact board fits beside the picker at 1280 px without scrolling; on phones the columns stay side by side with smaller ban squares, and the picker stacks under the board.
- **Keys** (legend under the board when `hints` is set):

| Key | On the board |
|---|---|
| Arrow keys | Move between slots: Left/Right walk the ban row and cross between the columns, Up/Down walk a column (bans, then picks 1 to 5) |
| Home / End | First / last step of the draft |
| Enter or Space | Choose the slot and move focus to the champion search |
| Backspace or Delete | Clear a filled slot (a toast offers Undo) |
| Escape (in the search) | Back to the board, on the current step |
| Enter (in the search) | Take the highlighted champion (the first match) into the current step; the board moves to the next empty step and focus stays in the search |

Pointer: click a slot to choose it, click a champion to fill the current step, drag a champion onto a slot or a filled slot onto another.

### 6.6 Page patterns (learned in M3)

- **List and editor.** Strategy pages put the team's records in a narrow list on the left (rows are full-width `Quiet` buttons with `pressed` for the open one, or `RowLink`s in a table) and the editor on the right; on phones the list comes first. The list says what each record holds (counts, result, survey age), not only its name.
- **Save state in the header.** Editors show "Not saved yet", "Unsaved changes", "Saving…" or "Saved" in the `PageHeader` meta, with the primary save action in the header actions; saving puts the record id in the URL (`?draft_id=`, `?plan_id=`, `?review_id=`) so a reload or a link reopens it.
- **Undo before confirmation.** Deletes that can be reversed from what the page holds (a cleared slot, a pool entry, an action item) happen at once with a toast that offers Undo; deletes that cannot (a draft, a plan, a tree, an opponent) go through `ConfirmDialog`.
- **Keyboard legends.** A page that has its own keys shows them once in a `Keycap` legend next to the thing they act on (the draft board, the pool tiers, the action items), and the same keys appear in the component's `A11y:` line.
- **Next step links.** Records link forward along the loop: a draft offers "Prep a game plan" and "Review this game", a plan "Start the review" (already linked), a review with action item lines creates the items and links to them.
- **Honest emptiness.** Every list, panel and sub-section without data shows an `EmptyState` saying what will appear and how; nothing renders invented numbers. A guess is labelled as one.
- **Focus after a change.** When a row moves (completed, restored, saved), focus follows it to its new place.

### 6.7 Draft Graph, stage A: the ranked list, Why panel and comp profile (M4)

The Draft Graph lives in the draft page as a switchable view next to the board, which stays the primary thing. A `SegmentedControl` above the column beside the board switches between **Picker** and **Draft Graph**. In the graph view the column widens over the intel rail's place at 1280 (the rail moves below the board row), and inside it a container query puts the ranked list and the Why panel side by side from 42rem; on phones they stack (list, Why, comp profile). The view is mounted only while selected, so a hidden graph does no work. Stage B put the force-directed canvas at the top of the same panel (section 6.8); this section is its text twin, which DESIGN principle 9 requires and which works fully without the canvas.

**Fit.** Every candidate gets a fit from 0 to 100. It starts at a neutral 50; four parts move it, in the owner's priority order, by at most half their weight: synergy (weight 40, ±20 points), counters (30, ±15), playstyle fit (20, ±10), power spikes (10, ±5); then the slot player's pool adds −8 to +6 (comfort +6, match ready +4, scrim ready +2, practicing 0, to practise −2, not in the pool −8). The word is always "fit", never a percentage or a chance to win, and the panel says "fit 78 of 100, from a neutral 50".

**Ranked list** (`data-graph-list`): a `role="listbox"` named after the slot ("Candidates for Blue pick 3"), top 10 with "Show all 30". Each option is one ruled row: rank (mono, muted), the champion's square avatar, name, `RoleLabel` (guessed unless the user chose the role above the list), the pool standing in words ("Comfort · Oskar Lind", or "Not in Oskar Lind's pool" in the warning tone), the fit as a mono figure, and the two strongest reasons, each with a short line in the canvas language: **solid** for synergy, playstyle and timing, **dashed** for counters, **dotted** for anything that works against the pick. A `LegendBlock` ("Reason lines") under the list explains the lines. The focused row is `ink-2` with a chalk outline (selection). Keys (Keycap legend under the list): Up/Down move the focus (roving tabindex, one Tab stop; Home/End), the Why panel follows the focus, Enter or a double click picks into the current slot, Escape returns to the board on the current step; Enter on the board in graph mode moves focus into the list. A click focuses (so a pointer user reads the Why panel before deciding), and the Why panel's "Pick" button picks. A pick shows a toast with Undo.

Above the list: the status line (a polite live region: "30 candidates for Blue pick 3 · Top, Mid or Support (open roles)") and a role `SegmentedControl` ("Open roles", or one role: then the role is chosen, not guessed). Ban steps and a complete draft show an `EmptyState` that says what the graph ranks; an enemy step ranks for their side and says so in an info `Callout` (pools apply to our picks only).

**Why panel** (`data-graph-why`, an `h3` "Why Leona"): avatar, fit, a secondary "Pick Leona" button with its keycap and a quiet "Argue with it: add a team note" (prefilled with the candidate and the champion of its strongest pair reason, and the relation that reason states). Then the fit by part as a two-column list ("Synergy (weight 40) +17 of ±20"), then every reason grouped by source under eyebrow headings, in this order: **Trait rules** (the rule's sentence, its id and weight in mono: "syn.engage_burst · weight 3"), **Team notes** (the note quoted in the serif `note` style, "team note by Linnea Voss on Leona and Jinx · weight 3", Remove), **Your games** ("Thresh and Jinx together in 7 of your games, 5 won", "n=7, corroborates +1"), **Scouting** ("Answers Malphite, which their support Nova has played (opponent dossier)") and **Player pool**. A last caption says when the candidate's trait row is still a shipped default or low confidence.

**Comp profile** (`data-graph-profile`, `h3`): two columns, the ranked side and the other side (`SideMarker` headers), as small `StatReadout`s: AD / AP (physical and magic shares, mixed counts half to each), Engage and Front line and Wave clear (of 10: each champion adds 0 to 2), Crowd control (of 15: 0 to 3 each), and Strongest (the curve most picks share: Early, Mid, Late or Even). The ranked side's readouts carry the change the focused candidate makes ("↑ +2 if Leona"), good-toned when it fills a gap (no engage yet, little front line, slow wave clear, little crowd control), neutral otherwise.

**Team notes on pairs** (`data-graph-team-notes`): below, the team's notes newest first ("Jinx works with Thresh", the reason in the serif note style, strength, author, date, Remove with Undo), or an `EmptyState` that says what a note does. "Add what you know" (quiet, in the panel header; team members only) opens a `Dialog`: champion, relation (`SegmentedControl` works with / beats / loses to), other champion, strength (Slight / Clear / Strong), and a required one-sentence reason (200 characters).

**What the ranking stands on** (`data-graph-notes`), one caption under the panel: "Built from the trait rules (173 rows still shipped defaults, 6 low confidence), 2 team notes, your stored games (0 so far; evidence shows from 5 games), the opponent dossier. Fit is explained, not a chance to win."

**Trait vocabulary** (the model behind the rules; `models::graph`, stored as these keys, edited on `/admin/traits`):

| Field | Values |
|---|---|
| roles | top, jungle, mid, bot, support (one or more; the first is the main role) |
| damage type | physical, magic, mixed, true-heavy |
| damage profile | burst, sustained, poke, hybrid |
| range | melee, short (ranged under 525 or a mixed kit), long (525 and up, or artillery) |
| engage | none, conditional, hard |
| disengage, peel, front line | none, some, strong |
| crowd control | none, light, medium, heavy |
| wave clear | weak, OK, strong |
| scaling | early, mid, late, flat |
| mobility | low, medium, high |
| playstyles | poke, dive, pick, split-push, teamfight, protect the carry, skirmish, siege, front to back, wombo (zero or more) |
| confidence | normal, low (neutral values where the kit was unclear) |

`/admin/traits` follows the page patterns: `PageHeader` with counts (champions, shipped defaults, edited, low confidence), a search field and a `SegmentedControl` filter (All, Defaults only, Edited, Low confidence), a sticky-header `Table` that scrolls inside its own region on phones, and an edit `Dialog` (Lg) with `Checkbox` roles and playstyles, `Select` for damage type and profile, a `SegmentedControl` per scale, "Reviewed for patch 16.20.1", and "Reset to default" for edited rows.

### 6.8 Draft Graph, stage B: the canvas (M4)

The force-directed picture of the same `GraphResult`, at the top of the Draft Graph panel, above the ranked list and the Why panel. Component: `DraftGraphCanvas` (`src/components/graph_canvas.rs`); layout: `src/components/graph_layout.rs` (plain Rust, no JS library, unit-tested). Both twins and the Why panel share the panel's `focused` signal (a champion id), so a focus anywhere is a focus everywhere.

**Frame.** A `CanvasFrame` with grid references A to H by 1 to 5 and the faint grid texture; 16:10 from `sm`, square on phones. The layout space is 800 by 500 units, one grid cell per 100 units, so every position reads as a grid reference ("Thresh, D3"). The caption names the slot and says the ranked list below is the same ranking in words. A `LegendBlock` ("Graph legend", columns) sits under the canvas.

**Nodes.** The ranked side's locked picks are pinned in column A, the other side's in column H, one row per role (Top row 1 to Support row 5; picks without a role fill the free rows in pick order): blue circles and red squares in the side colours with the champion's monogram, the name beside them when the canvas is at least 480 px wide. Open roles of the ranked side are dashed "?" ghosts in their row ("Support open"). Candidates (the top 24, the top 12 under 520 px) are round tokens with a monogram, free to move in columns B to G.

**Fit as elevation.** Each candidate carries a spot height ("▲84") and one contour ring for every 10 fit from 50 (fit 84: rings at 50, 60, 70, 80; under 50: none). The rings are slightly offset ellipses, so the picture reads as terrain rather than targets. Higher fit also means a stronger pull to the middle, so the summits sit near the centre and weak candidates drift outward. The spot height is text, so elevation is never shape alone.

**Edges** join a candidate to the locked picks it relates to (`GraphResult.edges`). Line style is the relation: **solid** synergy, **dashed** counter with an arrowhead at the champion that loses, **dotted** for a weak relation (under 1.5) or a synergy that works against the candidate. Width is strength (1.5 to 4.5 px). Colour is the side of the locked pick, nothing else.

**Forces** (what the positions mean, said in the legend and here): a candidate is pulled toward the picks it has synergy with (rest 190 units, stiffer when stronger) and, more weakly, toward an enemy pick it beats (rest 230); it is pushed away from a pick that beats it or works against it (inside 520 units); nodes repel at short range and never overlap (collision radius = the outer contour ring); a gentle pull to the middle grows with fit. The layout is deterministic: start positions come from the rank (a golden-angle spiral) or from the champion's place in the previous layout, so the same draft draws the same picture.

**Focus.** The focused candidate has a chalk ring (the accent: focus and selection only), its name and "▲ 84" in chalk under it, drawn on top of everything. Its one-hop neighbourhood lights: its edges in side colours at full width and the picks they reach; every other node, edge and ring fades to the rule tone (no opacity over text, faded labels stay in `muted`, 6.9:1). Faded edges show only the strong relations (1.5 and up) as hairlines. Clicking a locked pick lights the candidates related to it instead ("Jinx (your pick, at A4): 23 of the drawn candidates relate to it").

**Interaction.**

| Input | On the canvas |
|---|---|
| Tab | The canvas is one tab stop (`role="group"`, `aria-roledescription="graph canvas"`, described by the status line and the key legend) |
| Arrow keys | Next / previous candidate by rank (Right and Down forward, Left and Up back, wrapping); the list and the Why panel follow |
| Home / End | First / last drawn candidate |
| Enter | Pick the focused candidate into the current slot (the picker's path; toast with Undo) |
| Escape | Back to the board, on the current step |
| Click | Focus a candidate (or light a locked pick's candidates; click the empty ground to clear) |
| Double-click | Pick |
| Drag | Move a candidate; the rest re-settles around it, and on release it settles back to where its relations put it |

The status line under the canvas (`data-graph-canvas-status`) says what the focus means in words: "Thresh, fit 87, at D3 (1 of 24): synergy with Jinx, Vi and Orianna; beats Sejuani, Kai'Sa and Renekton." It is a polite live region while the canvas has focus and silent otherwise (the list announces its own options). Every node also has an SVG `<title>`; nothing lives only on hover. A focus on the canvas below the list's top 10 opens the whole list, so the text twin always shows the focused row.

**Motion.** The layout animates once per draft change, a few ticks per frame, then freezes: no perpetual motion, nothing loops. Under `prefers-reduced-motion: reduce` the simulation runs to the end synchronously and the settled layout is drawn directly; a drag then jumps instead of gliding.

**Performance** (budget from the M4 brief, measured 11 October 2026):

| Measure | Budget | Measured |
|---|---|---|
| Nodes | at most 60 | 31 at 1280 px (7 picks + 24 candidates), 19 at 400 px |
| Tick (layout step) | under 8 ms | natively 0.05 ms per tick for 32 nodes and 41 springs (debug build, `tick_cost_is_far_inside_the_budget`); in headless Chromium with the debug WASM build: mean 0.04 ms, worst 0.5 to 0.8 ms |
| Settle | under 1.5 s on a laptop | 150 ticks, 6 per frame: 0.4 s at 60 Hz; measured 0.53 to 0.84 s in the container's headless Chromium (its frames come every 30 to 45 ms); reduced motion 7 to 9 ms in one go |
| DOM | nodes reused, no re-creation per tick | the SVG is built once per result (and again only when a resize changes how many candidates fit); each frame writes one position signal that updates `transform` and the edges' `d` in place |
| Hidden view | no layout work | the panel (and so the canvas) is mounted only in graph mode; unmounting disposes the signals and the frame loop stops; the browser pauses frames in hidden tabs |
| Bundle | WASM grows less than 150 KB gzipped | +66 KB gzipped (2,273,468 to 2,341,037 bytes; +284 KB uncompressed): the release WASM (`wasm-release` profile, opt-level z, LTO) with custom sections stripped, gzip -9, before (4e85c76) and after the canvas, both without wasm-bindgen and wasm-opt |
| Server | recompute only on draft changes | the panel asks again 250 ms after the draft stops changing (stage A); the canvas recomputes the layout only when the result changes |

The page exposes the last run as attributes on `[data-graph-canvas]`: `data-sim-state` (running, settled), `data-sim-mode` (animated, instant), `data-sim-ticks`, `data-sim-nodes`, `data-sim-settle-ms`, `data-sim-tick-max-ms`, `data-sim-tick-mean-ms`, `data-sim-frame-max-ms`; `draft-graph.spec.ts` asserts the budget from them.

**DOM hooks.** `[data-graph-canvas-wrap]`, `[data-graph-canvas]`, `[data-node=candidate|pin][data-champion][data-name][data-fit][data-light=focused|lit|normal|faded]`, `[data-edge="{candidate}-{pick}"]` (lit layer), `[data-graph-canvas-status]`, `#graph-canvas-keys`.

**Not yet.** The tree drafter does not show the graph: its nodes store no roles and no side, so a pick from the graph would lose its role and the ranking would not know whose turn it is; that needs a role and side column on tree nodes first. Edge labels (a short rule name on the focused edges, as direction C sketched) are left out: the Why panel says the full sentence next to the canvas.

## 7. Champion images

Portraits are real (owner's decision) and never load from a third party in the browser. `GET /img/champion/{key}` (loading-screen portrait) and `GET /img/champion/{key}/icon` (square icon) are served by the app: the key must be a champion id or name known to the Data Dragon cache (unknown keys are 404; nothing from the URL touches the filesystem), the image is fetched once per patch from Data Dragon, kept in memory, and sent with `Cache-Control: public, max-age=604800`, an `ETag` and `If-None-Match` support. Cache misses are rate limited per client IP, keys Data Dragon does not have are remembered apart from the images, and a transient upstream failure is a 502 that is retried next time. `models::ddragon::champion_icon_url` points at the endpoint and `ChampionAvatar` uses it, with a monogram fallback. Role icons are typographic (`RoleLabel`); the browser loads no image from a third party (the gallery spec checks it).

## 8. Gallery

`/design`, for developers and admins only (dev builds, or `ADMIN_EMAILS`; everyone else gets the 404 page). One section per primitive with every variant and state (default, hover, focus-visible, pressed, disabled, loading, error, empty), forced states for screenshots, the token board with the measured contrast next to each pair, the type scale, the draft board (the page's own `DraftBoard`, mid-draft and complete) and the prep sequence example. All content is fixture text labelled as such.

`e2e/tests/gallery.spec.ts` screenshots every section at 1280 and 400px (attached to the Playwright report, and written to `$GALLERY_SHOTS_DIR` as `design-<section>-<width>.png` when set), fails if the page scrolls sideways at either width, and checks keyboard operation: Tabs (arrows, Home and End); Button (Enter, Space, the focus outline); TextField; Select; the error wiring of `Field`; Dialog (focus moves in, Tab stays inside, Escape closes, focus returns); ConfirmDialog (Cancel first for destructive actions); Menu (arrows skipping disabled items, typeahead, Home, Escape, selection); CommandBar (Ctrl+K, filtering, `aria-activedescendant`, Enter on a command and on a link); SegmentedControl and Switch; Tooltip (focus, `aria-describedby`, Escape); Toast with Undo in the polite region; TimelineScrub (slider keys and J/L); avatars loading only from `/img/`; the manual SegmentedControl (arrows move focus, Enter selects) and Checkbox; plus the draft board example (every step once, five bans and five picks per side, the one current step and its counter, arrow keys and Enter). Sections: tokens, contrast, type, space, layout, text, button, fields, badge, table, tabs, toggles, keycap, callout, toast, empty, skeleton, tooltip, dialog, menu, command, avatar, markers, readout, draft, prep, legend, canvas, timeline. Screenshots are evidence for review, not pixel baselines, so font rasterisation differences between machines do not fail CI. The route-level pixel baselines live in `visual-regression.spec.ts` (re-captured with `--update-snapshots`).

## 9. Lint gates

`scripts/lint_design.sh` (bash and grep, no Node), run in CI's `style_guardrails` job and by `just lint-design`. Rules over `src/**/*.rs` (server, models and bin excluded):

| Rule | Fails on | Allowed |
|---|---|---|
| `palette` | raw Tailwind palette colours (`text-red-400`, `bg-blue-500`, `text-white` ...) | none |
| `hex` | hex colour literals outside comments | none |
| `px-text` | `text-[Npx]` below 12px | none |
| `rounded-full` | `rounded-full` | inside `src/components/ui/` |
| `blur` | `backdrop-blur`, `backdrop-filter` | none |
| `gradient` | `bg-gradient`, `linear-gradient`, `radial-gradient` | none |
| `drop-shadow` | `drop-shadow`, `text-shadow` | none |
| `pulse` | `animate-pulse` | `src/components/ui/skeleton.rs` only |
| `ornament` | Cinzel, `font-imperial`, `font-glitch`, VT323, `canvas-grain`, fleur-de-lis and the other retired ornaments, Pandemonium | none |
| `emoji` | pictographs (emoji, and `\u{...}` escapes of them) in UI strings | typographic marks: stars, check, cross |
| `region` | `region ==`, `region=` props, `InitialTheme`, `data-theme` | none |
| `external-url` | an absolute URL to another host in client code, including `src/models` (SVG namespaces aside): the browser loads nothing from a third party | none |
| `faded-text` | a text tone with an opacity modifier (`text-muted/60`): not in the contrast table, and below 4.5:1 | none |
| `faint-outline` | an `outline` border, divide or ring with an opacity modifier (`border-outline/50`): control edges must keep 3:1 | none |
| G-12 | `outline-none` without a `focus-visible:` replacement on the same line | none |

Every rule is hard zero: any hit fails, and so does a grep error (a broken pattern or a grep without PCRE), so the gate cannot pass by matching nothing. (During M2 a ratcheting baseline allowed the 170 pre-existing hits; stage B mapped them to tokens, the radius policy and `Skeleton`, and deleted the baseline.)

Escape hatch for a legitimate exception: `lint:allow(<rule>)` in a comment on the same line, with the reason. It is used on six lines, all in `match_detail.rs`: the match timeline's event tokens stay round (map tokens may be round) until M4 moves them into `TimelineScrub`.

Two Rust tests complete the gates: the contrast test (section 3.2) and the a11y doc-comment test (section 6).

## 10. Migration plan

- **M2 stage A (done):** tokens, fonts, one theme, shims. Every page wears the Survey palette and type; nav, footer, sign-in, registration, closed beta, 404, solo dashboard header and team dashboard header use batch 1 primitives.
- **M2 stage B (done):** batches 2 and 3; the lint is hard zero (palette classes by meaning, the radius policy, `Skeleton`); role icons are `RoleLabel`s; lore eyebrows are gone (headings say what the section is); the nav has the command bar, the account `Menu` and the workspace `SegmentedControl`; `user.theme` is dropped (migration 7).
- **M3 (strategy pages, done):** draft (the compact tournament `DraftBoard` with a keyboard grid and the champion picker, replacing the carousel, table and ledger views and `DraftSequence`/`DraftSlot`), tree drafter, champion pool (tier ledger with keyboard moves), game plan (`PrepSequence`, `Checkbox` checklist), post-game (`ResultGlyph`, linked plan and draft), opponents (dossiers with evidence and survey age) and action items (status groups, inline edit, Undo, keys) are built from primitives with one `PageHeader` each; `ChampTile`/`ChampPortrait` are deleted; `border-outline/N` is gone everywhere (lint `faint-outline`).
- **M5 (remaining pages):** home, team dashboard, roster, profile, stats, match detail, personal layer, legal pages, Match Day on `PrepSequence`, phone menu and read-only views; with them the remaining shims (`region/*`, `ErrorBanner`, `StatusMessage`, `SkeletonLine/Card/Grid`, `PageLoading`, `PageEmpty`) and the last raw `<h1>`s (stats, roster, profile, admin invites, team builder, sign-in and registration, legal pages) move onto `PageHeader`.
- Page migration rule: no raw `<button>`/`<input>`/`<select>`/`<textarea>` outside `components/ui/`, no palette classes, no boxes that you cannot act on.
