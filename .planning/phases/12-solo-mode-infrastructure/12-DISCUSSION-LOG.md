# Phase 12: Solo Mode Infrastructure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-25
**Phase:** 12-solo-mode-infrastructure
**Areas discussed:** Mode toggle & navigation, Region support scope, Solo dashboard content, Queue filter UX

---

## Mode Toggle & Navigation

### Toggle placement

| Option | Description | Selected |
|--------|-------------|----------|
| Nav bar (Recommended) | Persistent toggle in top nav, one click to switch | ✓ |
| Profile page only | Toggle on profile/settings page, less visible | |
| Both nav + profile | Quick toggle in nav, detailed section on profile | |

**User's choice:** Nav bar
**Notes:** None

### Team features in solo mode

| Option | Description | Selected |
|--------|-------------|----------|
| Hide them (Recommended) | Nav only shows solo-relevant links | |
| Show but mark disabled | Grey out team-only links with tooltip | |
| Show all, gate on click | All links visible, team pages show switch message | ✓ |

**User's choice:** Show all, gate on click
**Notes:** None

### Default mode for new users

| Option | Description | Selected |
|--------|-------------|----------|
| Solo by default (Recommended) | New users start in solo mode, land on solo dashboard | ✓ |
| Team by default | Current behavior, land on /team/dashboard | |
| Prompt on first login | Ask user on first login which mode they prefer | |

**User's choice:** Solo by default
**Notes:** None

---

## Region Support Scope

### Number of regions

| Option | Description | Selected |
|--------|-------------|----------|
| Multi-region from day one (Recommended) | Support all major regions, parameterize riot.rs | ✓ |
| EUW only, parameterize later | Keep hardcoded EUW, add region field but don't wire | |
| Top 4 regions (EUW, NA, KR, EUNE) | Support most popular, still requires parameterizing | |

**User's choice:** Multi-region from day one
**Notes:** None

### Region selection UX

| Option | Description | Selected |
|--------|-------------|----------|
| Profile page, during Riot linking (Recommended) | Dropdown alongside game_name/tag_line inputs | ✓ |
| Registration flow | Ask for region during account creation | |
| Auto-detect from Riot account | Try each region route until puuid resolves | |

**User's choice:** Profile page, during Riot linking
**Notes:** None

---

## Solo Dashboard Content

### Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Single column, stacked sections (Recommended) | Top: rank, Middle: matches, Bottom: goals | ✓ |
| Two-column grid | Left: rank + goals, Right: matches | |
| Card-based widgets | Discrete cards that can be rearranged | |

**User's choice:** Single column, stacked sections
**Notes:** Preview mockup confirmed

### Ranked badge display

| Option | Description | Selected |
|--------|-------------|----------|
| Tier icon + text + LP (Recommended) | Official rank emblem + "Gold II - 47 LP" | ✓ |
| Text only, styled by tier | No image, text colored by tier | |
| You decide | Claude picks | |

**User's choice:** Tier icon + text + LP
**Notes:** None

### Sync trigger

| Option | Description | Selected |
|--------|-------------|----------|
| Manual button on dashboard (Recommended) | Click to sync, no auto behavior | |
| Manual + auto on page load | Auto-sync if last sync > 10 min, plus manual button | ✓ |
| You decide | Claude picks | |

**User's choice:** Manual + auto on page load
**Notes:** None

### Queue rank display

| Option | Description | Selected |
|--------|-------------|----------|
| Solo/Duo only (Recommended) | Show only solo/duo rank badge | ✓ |
| Both queues side by side | Show solo/duo and flex rank | |

**User's choice:** Solo/Duo only
**Notes:** None

### Goal placeholders

| Option | Description | Selected |
|--------|-------------|----------|
| Greyed-out cards with 'Coming in Phase 15' | 2-3 placeholder cards teasing specific goals | ✓ |
| Simple text banner | Single line saying 'coming soon' | |
| You decide | Claude picks | |

**User's choice:** Greyed-out cards with 'Coming in Phase 15'
**Notes:** None

---

## Queue Filter UX

### Filter style

| Option | Description | Selected |
|--------|-------------|----------|
| Segmented toggle buttons (Recommended) | Three toggle buttons: All / Solo/Duo / Flex | |
| Dropdown select | Dropdown menu, compact, two clicks | ✓ |
| Tabs | Tab bar above match list | |

**User's choice:** Dropdown select
**Notes:** None

### Filter scope

| Option | Description | Selected |
|--------|-------------|----------|
| Stats page only (Recommended) | Add filter to stats page only, dashboard is pre-scoped | |
| Both stats page and solo dashboard | Filter on both pages | ✓ |
| You decide | Claude picks | |

**User's choice:** Both stats page and solo dashboard
**Notes:** None

---

## Claude's Discretion

- Auto-sync staleness threshold (10 min suggested)
- Region-to-route mapping implementation details
- "Switch to team mode" gate message styling
- Ranked emblem image source

## Deferred Ideas

None — discussion stayed within phase scope
