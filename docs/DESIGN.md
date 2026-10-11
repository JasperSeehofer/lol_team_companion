# Design system

The single source of truth for how Companion looks and behaves. `input.css` implements the tokens, `src/components/ui/` the primitives, `/design` shows both (the gallery), and `scripts/lint_design.sh` plus two Rust tests keep the code honest. When this file and the code disagree, fix one of them in the same change.

Status: M2 stage A (tokens, type, fonts, lint gates, gallery, batch 1 primitives). Batches 2 and 3 are listed as planned and land in stage B.

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
- **Surveyor's notes** (the coach's annotations, with a leader line on a canvas) are set in serif italic. The serif appears nowhere else, so italic serif always means "a person wrote this here".
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
| `page` | `danger` | 4.5 | 7.01 | text on the danger button (hover) |
<!-- contrast-pairs:end -->

### 3.3 Type

| Face | Role | Weights used | Notes |
|---|---|---|---|
| **Familjen Grotesk** | Display: page titles, section titles, tab labels, the next prep step | 600 (500 to 700 available) | `font-display`. Never below 16px. |
| **Public Sans** | UI and text: everything else | 400, 500, 600 | `font-sans`, set on `body`. Italic exists for emphasis in prose only. |
| **JetBrains Mono** | Figures, coordinates, overlines, keycaps, timers | 400, 500, 600 | `font-mono`, `figures` utility. |
| **Source Serif 4 italic** | Surveyor's notes (annotations on canvases, the coach's note on a moment) | 400 | `note` utility. Nowhere else. |

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
| `text-page` | 32 / 1.1 | Page titles, `<h1>` (display face) |
| `text-display` | 44 / 1.05 | One per page at most (landing, closed beta) |

Role utilities: `overline` (mono 12px, 500, +0.08em tracking, caps, muted), `figures` (mono, tabular), `note` (serif italic). The `Text` primitive maps roles to these. Tailwind's own `text-xs` and up still exist (all at or above 12px); arbitrary `text-[Npx]` below 12px fails the lint.

**Figures:** `body` sets `font-variant-numeric: tabular-nums`, so every number lines up (all three sans and serif faces have `tnum`; the mono is monospaced). Use `figures` for numbers that are the point (scores, fit, clock, gold).

**Weights on dark:** 400 and up; light weights bloom.

### 3.4 Spacing and density

Tailwind's 4px scale. The common steps are 4, 8, 12, 16, 24, 32, 48, 64. Boards and tables are compact (8 to 12px row padding, `Density::Compact`); briefs and forms are comfortable (24 to 32px between groups). `PageShell` sets 16px side gutters on phones and 32px from `sm`, so no page scrolls sideways at 400px.

### 3.5 Radius

| Token | Value | Where |
|---|---|---|
| none | 0 | Structure: page bands, panels, tables, rules, canvases |
| `rounded-ctl` (and every named Tailwind radius) | 3px | Controls: buttons, fields, selects, badges, keycaps, draft slots |
| `rounded-full` | round | Avatars, toggles, map tokens, side markers. Only inside primitives (lint). |

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
- **Status:** success, warning, danger and info always come with words (and a shape in `Badge`). Danger is a crimson distinct from the red side, and appears only for errors and destructive actions.
- **Never colour alone** (WCAG 1.4.1): every encoding has a second channel (text, glyph, shape, line style, position).
- **Raw Tailwind palette classes** (`text-red-400`, `bg-blue-500`) are banned in `src/` (lint). `text-white` survives only on the few coloured buttons listed in the lint allow list until M3.

## 5. Canvas rules

Applies to the Sand Table map, the Draft Graph, map insets and charts (M4, Release 2; components reserved in batch 3).

- **Coordinate frame:** tick-marked edges with real numbers (Sand Table 0 to 1000; Draft Graph grid A to H by 1 to 5) so a coach can say "Rakan, D4" on voice. Charts outside canvases get plain axes, not frames.
- **Legend:** every canvas that encodes colour, shape, line style or width has a legend block next to it, with direct labels where possible. On the Sand Table the overlays are the legend (switches, at most three active).
- **Texture:** contours or grid at about 1.1:1 against `canvas`, only inside the frame, never behind running text. Labels on canvases get a canvas-coloured halo.
- **Uncertainty:** estimated paths dashed, exact events as solid anchors, approximate areas as soft dashed blobs.
- **Accessibility:** a canvas is one tab stop with a text twin (the ranked reasons list for the graph, the readouts and moments for the map).

## 6. Primitives

Location: `src/components/ui/`, re-exported from `components::ui`. Every component has typed variants (Rust enums), class names written out in full (Tailwind only sees literal strings; enums map to `&'static str` in `match` arms), identical markup on server and browser, a forced-state hook for the gallery (`force: Option<Force>` on interactive primitives, rendered as `data-force`), and a doc comment whose `A11y:` line states its accessibility contract (`components::ui::tests::every_component_documents_its_a11y_contract` fails otherwise). Focus is the global rule in `input.css` (`:focus-visible`, 2px chalk outline, 2px offset); primitives do not repeat focus classes.

### 6.1 Batch 1 (stage A, in the gallery)

| Primitive | Props and variants | A11y contract |
|---|---|---|
| `PageShell` | `width: PageWidth {Narrow 28rem, Reading 48rem, Standard 64rem, Wide 80rem (default), Full}`, `density: Density {Compact, Comfortable}`, `class` | A `div` inside the shell's single `<main id="main">` (the shell has the skip link). 16px phone gutters. |
| `PageHeader` | `title`, `eyebrow?`, `meta?: ViewFn`, `actions?: ViewFn` | The page's only `<h1>`; actions follow it in tab order. |
| `SectionHeader` | `title`, `level: HeadingLevel {H2, H3}`, `eyebrow?`, `meta?`, `actions?`, `id?` | A real `h2`/`h3`; `id` for `aria-labelledby`. |
| `Surface` | `layer: Layer {Inherit (default), Page, Ink1, Ink2, Ink3, Canvas}`, `edge: Edge {None (default), RuleTop, RuleY, Rule, Object}`, `pad: Pad {None, Sm, Md, Lg}`, `tag: SurfaceTag {Div, Section, Article, Aside}`, `label?`, `labelled_by?`, `class` | Landmarks get a name; `attr:` attributes (for example `data-feedback-label`) land on the root. Undecorated by default. |
| `Text` | `role: TextRole {Display, Page, Title, Heading, Reading, Body (default), Label, Caption, Overline, Figure, FigureLarge, Note}`, `tone: Tone {Inherit, Primary, Secondary, Muted, Accent, Success, Warning, Danger, Info}`, `tag: TextTag {Span, P, Div}` | Never a heading; every tone 4.5:1 on every layer; nothing below 12px. |
| `Button` | `variant: ButtonVariant {Primary, Secondary (default), Quiet, Danger}`, `size: ButtonSize {Sm 28px, Md 32px, Lg 40px}`, `kind: ButtonType {Button, Submit, Reset}`, `disabled`, `loading`, `on_click`, `href` (renders a link), `icon` slot, `shortcut` (keycap), `aria_label`, `full_width`, `force` | Native `<button type="button">`; loading disables and sets `aria-busy`; keycap hidden from screen readers, `aria-keyshortcuts` set; icon-only buttons need `aria_label`. Targets at least 24px. |
| `Keycap` | `keys` | A `<kbd>`. |
| `Field` | `id`, `label`, `hint?`, `error: Signal<Option<String>>`, `required` | Visible label bound with `for`; hint and error via `aria-describedby`; `aria-invalid` on the control; error announced (`role="alert"`); "(required)" written out. Provides `FieldContext` to the control inside. |
| `TextField` | `value?: RwSignal<String>`, `kind: InputKind {Text, Email, Password, Search, Number, Date, Url}`, `size: FieldSize {Sm 32px, Md 36px}`, `name`, `placeholder`, `autocomplete`, `minlength`, `maxlength`, `required`, `disabled`, `rows` (textarea), `id`, `aria_label`, `force` | Takes id, description and invalid state from `Field`; 3:1 outline edge; `prop:value`. |
| `Select` | `value?`, `options: Vec<SelectOption>`, `size`, `name`, `disabled`, `id`, `aria_label`, `force` | Native `<select>` (type-ahead, arrows); chevron decorative. |
| `Badge` | `tone: BadgeTone {Neutral, Accent, Success, Warning, Danger, Info, SideBlue, SideRed, Win, Loss}`, `label?` | Meaning in words; marker shape repeats it (hidden from screen readers); `label` gives W/L glyphs the name "Win"/"Loss". |
| `Table` (+ `Th`, `Tr`, `Td`, `RowLink`) | `caption`, `caption_hidden`, `density`, `sticky_header`; `Th {align, numeric, sort}`, `Td {numeric, muted, header, align}`, `Tr {selected}`, `RowLink {href}` | Real `<table>` with `<caption>`; scroll wrapper is a named focusable region; `th scope`, `aria-sort`; numbers right-aligned in tabular figures; row link is one tab stop. |
| `Tabs` (+ `TabPanel`) | `group`, `label`, `tabs: Vec<TabItem {id, label, count?}>`, `selected: RwSignal<String>` | WAI-ARIA tabs: roving tabindex, Left/Right (wrapping), Home/End, automatic activation, `aria-selected`, `aria-controls`; panels hidden, not removed (SSR-safe). |

Domain primitives shipped early (owner's decisions):

| Primitive | Props | A11y contract |
|---|---|---|
| `DraftSlot` | `step: DraftStep {order 1..=20, phase, side, action, nth}`, `state: SlotState {Empty, Selecting, Locked}`, `champion?`, `role?`, `on_select?` | Accessible name says it all ("Step 19, Blue pick 5, Support: selecting now"); side by shape and lane; `aria-current="step"` while selecting; a real button when `on_select` is set. |
| `DraftSequence` | `champions: Vec<Option<String>>` (index = order - 1), `roles`, `current: Option<u8>`, `on_select?`, `heading_h4` | Four named phase groups, each an ordered list in draft order with a blue and a red lane, so B R B R B R · B R R B B R · R B R B · R B B R reads top to bottom. `TOURNAMENT_ORDER` is the single definition (tested against `draft_board::slot_meta`). |
| `PrepSequence` | `label`, `steps: Vec<PrepStep {title, detail?, status?, state: PrepState {Done, Next, Todo, Attention}, href?, action?}>` | Ordered list; status in words; the next step has `aria-current="step"`, a larger display title and a real link to where it is done. The Match Day "what to do next" pattern from direction B, translated into quiet type. |

Older components restyled onto the tokens until batch 2 replaces them: `ErrorBanner`, `StatusMessage`, `ToastProvider`, `SkeletonLine/Card/Grid`, `EmptyState` (no icon or emoji any more), `NoTeamState`.

### 6.2 Batch 2 (stage B, planned)

Dialog (native `<dialog>`, focus in and back, Esc, solid scrim), Menu (menu button pattern), CommandBar (Ctrl/Cmd+K, combobox + listbox), Toast (four kinds, rework of `ToastProvider`), Callout (replaces `ErrorBanner`/`StatusMessage`), EmptyState (title, body, action), Skeleton, Tooltip, SegmentedControl and Switch, Checkbox.

### 6.3 Batch 3 (stage B, planned)

ChampionAvatar (portrait from `/img/champion/{key}/icon`, monogram fallback, banned/locked/dimmed states), SideMarker, ResultGlyph, StatReadout, LegendBlock, CanvasFrame (coordinate frame; reserved for map and graph), Annotation (serif italic callout with leader line), TimelineScrub (reserved, Sand Table).

### 6.4 Compatibility shims

`src/components/region/` keeps the names pages still call (`Card`, `SectionHead`, `Btn`, `Badge`, `ModeToggle`, `Eyebrow`, `ChampTile`, `ChampPortrait`, `RankBadge`, `LPProgress`) without the `region` prop, rendering batch 1 primitives or token-only markup. They exist so every page restyles now and M3 only migrates structure; they are deleted when the last caller is gone.

## 7. Champion images

Portraits are real (owner's decision) and never load from a third party in the browser. `GET /img/champion/{key}` (loading-screen portrait) and `GET /img/champion/{key}/icon` (square icon) are served by the app: the key must be a champion id or name known to the Data Dragon cache (unknown keys are 404; nothing from the URL touches the filesystem), the image is fetched once per patch from Data Dragon, kept in memory, and sent with `Cache-Control: public, max-age=604800`, an `ETag` and `If-None-Match` support. `models::ddragon::champion_icon_url` points at the endpoint.

## 8. Gallery

`/design`, for developers and admins only (dev builds, or `ADMIN_EMAILS`; everyone else gets the 404 page). One section per primitive with every variant and state (default, hover, focus-visible, pressed, disabled, loading, error, empty), forced states for screenshots, the token board with the measured contrast next to each pair, the type scale, the worked tournament draft example and the prep sequence example. All content is fixture text labelled as such.

`e2e/tests/gallery.spec.ts` screenshots every section at 1280 and 400px (attached to the Playwright report, and written to `$GALLERY_SHOTS_DIR` as `design-<section>-<width>.png` when set), fails if the page scrolls sideways at either width, and checks keyboard operation (Tabs arrows, Home and End; Button with Enter and Space and its focus outline; TextField; Select; the error wiring of `Field`) plus the tournament order of the draft example. Screenshots are evidence for review, not pixel baselines, so font rasterisation differences between machines do not fail CI. The route-level pixel baselines live in `visual-regression.spec.ts` (re-captured with `--update-snapshots`).

## 9. Lint gates

`scripts/lint_design.sh` (bash and grep, no Node), run in CI's `style_guardrails` job and by `just lint-design`. Rules over `src/**/*.rs` (server, models and bin excluded):

| Rule | Fails on | Allowed |
|---|---|---|
| `palette` | raw Tailwind palette colours (`text-red-400`, `bg-blue-500`, `text-white` ...) | none (the remaining `text-white` on coloured buttons is in the baseline) |
| `hex` | hex colour literals outside comments | none |
| `px-text` | `text-[Npx]` below 12px | none |
| `rounded-full` | `rounded-full` | inside `src/components/ui/` |
| `blur` | `backdrop-blur`, `backdrop-filter` | none |
| `gradient` | `bg-gradient`, `linear-gradient`, `radial-gradient` | none |
| `drop-shadow` | `drop-shadow`, `text-shadow` | none |
| `pulse` | `animate-pulse` | the skeleton components (`src/components/ui/feedback.rs`, `src/components/skeleton.rs`) |
| `ornament` | Cinzel, `font-imperial`, `font-glitch`, VT323, `canvas-grain`, fleur-de-lis and the other retired ornaments, Pandemonium | none |
| `emoji` | pictographs (emoji, and `\u{...}` escapes of them) in UI strings | typographic marks: stars, check, cross |
| `region` | `region ==`, `region=` props, `InitialTheme`, `data-theme` | none |
| G-12 | `outline-none` without a `focus-visible:` replacement on the same line | none |

A baseline (`scripts/design-lint-baseline.txt`, `rule<TAB>file<TAB>count`) lets the count ratchet during M2: a rule may not gain hits in any file, and new files start at zero. At the end of stage B the baseline is deleted and every rule is hard zero. `scripts/lint_design.sh --update-baseline` rewrites it (only ever downwards in review). At the end of stage A the baseline holds 170 hits: palette classes (88, mostly `draft.rs` and `post_game.rs`), `rounded-full` outside primitives (75) and skeleton-style `animate-pulse` in five pages (7). Every other rule is already at zero.

Escape hatch for a legitimate exception: `lint:allow(<rule>)` in a comment on the same line (used once, for the nav avatar until batch 3 ships an avatar primitive).

Two Rust tests complete the gates: the contrast test (section 3.2) and the a11y doc-comment test (section 6).

## 10. Migration plan

- **Now (M2 stage A):** tokens, fonts, one theme, shims. Every page wears the Survey palette and type; nav, footer, sign-in, registration, closed beta, 404, solo dashboard header and team dashboard header use batch 1 primitives.
- **Stage B:** batches 2 and 3; lint baseline to zero (palette classes, sub-scale sizes, `rounded-full`, raw controls).
- **M3 (strategy pages):** draft (rebuilt shell on `DraftSequence`/`DraftSlot`, keyboard grid), tree drafter, champion pool, game plan, post-game, opponents (dossier), action items move onto primitives; `region/` shims and legacy `feedback` components are deleted when the last caller is gone. Every page gets exactly one `PageHeader` (`<h1>`).
- **M5 (remaining pages):** home, team dashboard, roster, profile, stats, match detail, personal layer, legal pages, Match Day on `PrepSequence`, phone menu and read-only views.
- Page migration rule: no raw `<button>`/`<input>`/`<select>`/`<textarea>` outside `components/ui/`, no palette classes, no boxes that you cannot act on.
