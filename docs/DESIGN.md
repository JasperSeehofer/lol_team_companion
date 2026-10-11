# Design system

> **Status: skeleton. To be completed in M2** ("Design system foundation", a STRONG gate in `MISSION.md`).
> Until then, follow the code style in `CLAUDE.md`: semantic tokens from `input.css`, no raw palette colours, the existing `EmptyState` / `ErrorBanner` / `ToastProvider` components.

This document becomes the single source of truth for how Companion looks and behaves. It replaces the external wiki references and the planning-era UI specs (archived under the git tag `planning-archive-2026-10`).

## Decisions already taken

From `MISSION.md` section 4 (agreed 2026-10-10):

- **Dark only** in Release 1. Tokens are structured so a light theme can be added later.
- **One theme.** The structural "Pandemonium" theme is removed; theming is a token swap, never different markup.
- **Mixed density.** Compact for boards, tables and the draft surface; comfortable for reading pages, forms and settings.
- **Purposeful motion.** Animation explains a change of state; nothing decorative loops. `prefers-reduced-motion` is respected everywhere.
- **WCAG AA** across the app: contrast, keyboard operation (draft board, dialogs, menus), visible focus, labelled form fields.
- **Not gamer-styled, not generic.** Avoid classic gamer styling and generic AI-made UI (box soup, everything rounded, purple gradients). No Riot IP in the brand.
- **Phones:** a menu plus read-only views of plans, drafts, reviews and briefs.
- **Process:** three visual directions are shown as mockups; the owner picks one. Components are approved in three batches (controls, layout, data).

## 1. Principles

_To be written in M2 from the chosen direction._ Expected topics: data over ornament, one face for UI and one for numbers, every empty state teaches, keyboard first on the draft surface, honesty (no invented data).

## 2. Tokens

_To be specified in M2._ The token pass covers:

| Group | Scope |
|-------|-------|
| Colour | neutral ramp, one accent, status (success, warning, danger, info), domain tokens (blue/red side, win/loss, tiers, rank tiers, chart palette); all text pairs at AA or better |
| Typography | type scale (no sub-12px body text), UI font applied to `body`, numeric font, line heights, tracking |
| Space and size | spacing scale, density modifiers (compact / comfortable) |
| Radius, shadow, layers | a small radius set, two elevations, a z-index scale (nav, popover, dialog, toast) |
| Motion | durations and easings, reduced-motion behaviour |

Current state for reference: `input.css` defines semantic colour tokens and five font families only.

## 3. Primitives

_To be specified in M2._ About 25 primitives with typed variants (Rust enums, no stringly-typed props), delivered in three approved batches:

1. **Controls:** Button, IconButton, TextField, Select, Textarea, Checkbox, Switch, SegmentedControl, Tabs, Tooltip, Popover/Menu.
2. **Layout:** PageShell, PageHeader, SectionHeader, Card, Dialog (focus trap, Esc, restore focus), Toast (four kinds), EmptyState, Skeleton.
3. **Data:** DataTable, KpiTile, Badge, Progress, Avatar, ChampIcon, SideChip, WinLossPill.

Reserved for the Sand Table (Release 2): map, token, timeline and overlay components.

## 4. Gallery

_To be built in M2._ A gallery page for developers and admins that renders every primitive in every variant and state. It doubles as the visual-regression surface.

## 5. Lint gates

_To be implemented in M2._ Planned CI checks:

- no raw palette colour classes or hex values in `src/pages/` and `src/components/` (tokens only),
- no arbitrary pixel font sizes (`text-[Npx]`) outside the token file,
- no raw `<button>`, `<input>`, `<select>` or `<textarea>` outside the primitives module,
- AA contrast unit test over token pairs,
- the existing guardrails stay: no Google Fonts CDN, no `outline-none` without a focus-visible replacement.
