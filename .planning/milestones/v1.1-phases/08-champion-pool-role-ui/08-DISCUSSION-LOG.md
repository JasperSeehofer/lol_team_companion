# Phase 8: Champion Pool & Role UI - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-22
**Phase:** 08-champion-pool-role-ui
**Areas discussed:** Champion pool layout, Drag-and-drop between tiers, Matchup entry rework, Draft role assignment

---

## Champion Pool Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Square tile grid | 48-64px portrait tiles in a grid, name below/on hover | |
| Enlarged pills | Keep pill layout but make icons 48px with more breathing room | |
| Card grid | Full cards with portrait, name, comfort stars, meta tag, stats | ✓ |

**User's choice:** Card grid
**Notes:** User wants maximum info density — portrait, name, comfort stars, meta tag badge, and match stats all visible per card.

### Grid Columns

| Option | Description | Selected |
|--------|-------------|----------|
| 3 columns | Fits 3 cards per row (~140px wide) | ✓ |
| 4 columns | Tighter grid (~110px), may need to drop stats | |
| Auto-fill responsive | CSS grid auto-fill min 120px | |

**User's choice:** 3 columns

### Selection Visual

| Option | Description | Selected |
|--------|-------------|----------|
| Accent border + subtle glow | Border highlight + box-shadow glow | ✓ |
| Accent border only | Same as current behavior on larger card | |
| Raised/elevated card | Scale transform + shadow lift | |

**User's choice:** Accent border + subtle glow

### Remove Button

| Option | Description | Selected |
|--------|-------------|----------|
| Corner X on hover | Small × badge in top-right on hover | ✓ |
| Always visible X | × always shown | |
| Context menu | Right-click/long-press | |

**User's choice:** Corner X on hover

---

## Drag-and-Drop Between Tiers

| Option | Description | Selected |
|--------|-------------|----------|
| Drop on tier header | Drag to tier label area | |
| Drop anywhere in tier bucket | Entire tier container is drop zone | ✓ |
| Drop between cards | Full reorder with insertion line | |

**User's choice:** Drop anywhere in tier bucket

### Drag Visual Feedback

| Option | Description | Selected |
|--------|-------------|----------|
| Ghost card + highlight tier | Source opacity 50%, target tier accent border | ✓ |
| Placeholder gap + highlight | Dashed placeholder at source, highlight at target | |
| Minimal highlight | Just tier border highlight | |

**User's choice:** Ghost card + highlight tier

---

## Matchup Entry Rework

| Option | Description | Selected |
|--------|-------------|----------|
| Champion search + notes | Matchup type: replace title with ChampionAutocomplete, auto-fill "vs {opponent}" | ✓ |
| Dedicated matchup tab | Split matchups into own section | |
| Unified simplified form | Remove note_type, all free-form | |

**User's choice:** Champion search + notes

### Matchup Note Display

| Option | Description | Selected |
|--------|-------------|----------|
| Icon + name badge | 24px opponent icon + name badge next to Matchup label | ✓ |
| Name text only | "vs Syndra" text without icon | |
| You decide | Claude picks | |

**User's choice:** Icon + name badge

---

## Draft Role Assignment

### Role Icon Position

| Option | Description | Selected |
|--------|-------------|----------|
| Bottom-right corner badge | 16-20px role icon overlaid in bottom-right of pick slot | ✓ |
| Below the slot | Role icon below pick slot as separate element | |
| Left side strip | Thin vertical strip on left edge | |

**User's choice:** Bottom-right corner badge

### Role Selector

| Option | Description | Selected |
|--------|-------------|----------|
| Popover with 5 role icons | Click badge → popover with 5 role icon buttons | ✓ |
| Inline icon row | Click badge → slot expands with icon row below | |
| Dropdown menu | Click badge → text dropdown | |

**User's choice:** Popover with 5 role icons

### Auto-Guess Visual Distinction

| Option | Description | Selected |
|--------|-------------|----------|
| Dimmed auto-guess, solid when set | 50% opacity + dashed border for auto, full + solid for confirmed | ✓ |
| Same appearance | No distinction | |
| Question mark overlay | Small ? on auto-guessed | |

**User's choice:** Dimmed auto-guess, solid when set

### Role Persistence

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, add role field to DraftAction | Optional role field in model + schema, persists with draft | ✓ |
| Client-side only | Local state, lost on refresh | |

**User's choice:** Yes, add role field to DraftAction

---

## Claude's Discretion

- Exact card dimensions and spacing
- Champion class → role mapping heuristics
- Popover positioning and dismiss behavior
- Drag ghost appearance
- Card tier transition animations

## Deferred Ideas

None — discussion stayed within phase scope
