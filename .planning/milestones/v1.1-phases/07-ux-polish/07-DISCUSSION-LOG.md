# Phase 7: UX Polish - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-22
**Phase:** 07-ux-polish
**Areas discussed:** Timestamp format, Team search UX, Role icon treatment, Toast & profile fixes

---

## Timestamp Format

| Option | Description | Selected |
|--------|-------------|----------|
| Absolute only | "Mar 19, 2026 14:30" everywhere | |
| Relative for recent, absolute for old | "5 min ago" for <24h, then absolute | ✓ |
| Relative with absolute tooltip | Relative text, full date on hover | |

**User's choice:** Relative for recent, absolute for old
**Notes:** None

### Follow-up: Cutoff threshold

| Option | Description | Selected |
|--------|-------------|----------|
| 24 hours | <24h relative, >=24h absolute | ✓ |
| 7 days | <7d relative, >=7d absolute | |
| 48 hours | <48h relative, >=48h absolute | |

**User's choice:** 24 hours

### Follow-up: Year display

| Option | Description | Selected |
|--------|-------------|----------|
| Year only when different | "Mar 19, 14:30" this year, "Mar 19, 2025 14:30" for older | ✓ |
| Always include year | "Mar 19, 2026 14:30" always | |

**User's choice:** Year only when different

### Follow-up: Clock format

| Option | Description | Selected |
|--------|-------------|----------|
| 24-hour | "14:30" — compact, common in gaming | ✓ |
| 12-hour with AM/PM | "2:30 PM" — US-familiar | |

**User's choice:** 24-hour

---

## Team Search UX

| Option | Description | Selected |
|--------|-------------|----------|
| Client-side filter | Fetch all, filter as user types | ✓ |
| Server-side search | Debounced server query | |
| Combo: client + server fallback | Load 50 client-side, server for no matches | |

**User's choice:** Client-side filter

### Follow-up: Empty search state

| Option | Description | Selected |
|--------|-------------|----------|
| All teams | Show full list when empty | |
| Prompt to search | "Type to search" with no list | ✓ |
| Popular/recent teams | Curated subset | |

**User's choice:** Prompt to search

### Follow-up: Team card content

| Option | Description | Selected |
|--------|-------------|----------|
| Name + member count | "TSM Academy — 4 members" | |
| Name only (current) | Minimal change | |
| Name + region + member count | All metadata visible | ✓ |

**User's choice:** Name + region + member count

### Follow-up: Search field scope

| Option | Description | Selected |
|--------|-------------|----------|
| Name only | Filter matches team name | ✓ |
| Name + region | Filter matches both fields | |

**User's choice:** Name only

---

## Role Icon Treatment

| Option | Description | Selected |
|--------|-------------|----------|
| Watermark background | Large, faded icon (~10% opacity) as card background | ✓ |
| Inline icon (current style) | Small 24px icon next to name on all cards | |
| Color-coded border + icon | Left border color per role plus small icon | |

**User's choice:** Watermark background

### Follow-up: Watermark position

| Option | Description | Selected |
|--------|-------------|----------|
| Bottom-right | Partially cropped in bottom-right corner | ✓ |
| Center | Centered behind all content | |
| Top-right | Top-right corner | |

**User's choice:** Bottom-right

### Follow-up: Unassigned roles

| Option | Description | Selected |
|--------|-------------|----------|
| No watermark for unassigned | Blank = visual cue to assign | ✓ |
| Generic placeholder watermark | Question mark or generic icon | |

**User's choice:** No watermark for unassigned

### Follow-up: Card type scope

| Option | Description | Selected |
|--------|-------------|----------|
| Starters + bench only | Coaches excluded | |
| All card types | Coaches get generic coach icon | ✓ |

**User's choice:** All card types (coaches get coach icon)

---

## Toast & Profile Fixes

### Toast positioning

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed below nav height | top-16 (~64px) to clear sticky nav | ✓ |
| Slide down from nav edge | Animate from below nav | |
| Top-right corner below nav | Less intrusive corner position | |

**User's choice:** Fixed below nav height

### Profile button

| Option | Description | Selected |
|--------|-------------|----------|
| Verify in browser first | Check what actually renders | |
| I've seen two buttons | Definitely two in some state | |
| Just ensure single button | Ensure one button in all states | ✓ |

**User's choice:** Just ensure single button in all states

---

## Claude's Discretion

- Exact watermark opacity value (8-12%)
- Toast top offset pixel value
- Timestamp helper implementation approach
- Search input placeholder and styling
- Coach watermark icon choice

## Deferred Ideas

None — discussion stayed within phase scope
