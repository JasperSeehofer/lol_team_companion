# Phase 10: Opponents Redesign - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 10-opponents-redesign
**Areas discussed:** New opponent form, Player card design, Pool analysis section, Auto-fetch behavior

---

## New Opponent Form

### Form Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Inline 5-role form | Replace current flow with single form: team name + 5 pre-labeled role rows with Riot ID fields | ✓ |
| Modal/dialog form | '+ New Opponent' opens a modal with the 5-role form | |
| Two-step flow | Step 1: enter team name. Step 2: fill 5 role slots in detail panel | |

**User's choice:** Inline 5-role form
**Notes:** None

### Partial Entry

| Option | Description | Selected |
|--------|-------------|----------|
| Allow partial | Any combination of filled/empty Riot ID fields is valid | ✓ |
| Require all 5 | All 5 Riot ID fields must be filled before saving | |

**User's choice:** Allow partial
**Notes:** None

### Riot ID Input Format

| Option | Description | Selected |
|--------|-------------|----------|
| Single field: Name#Tag | One input per role, validates on '#' separator | ✓ |
| Split fields: Name + Tag | Two inputs per role | |

**User's choice:** Single field: Name#Tag
**Notes:** None

---

## Player Card Design

### OTP Badge

| Option | Description | Selected |
|--------|-------------|----------|
| Warning badge on card header | Orange/yellow 'OTP: {champion}' badge next to role label | ✓ |
| Inline text warning | Red text line below champion list | |
| Champion highlight | OTP champion pill gets special border | |

**User's choice:** Warning badge on card header
**Notes:** None

### Champion Mastery Display

| Option | Description | Selected |
|--------|-------------|----------|
| Mastery level badges on pills | Each pill shows mastery level: 'Ahri M7', sorted by mastery points | ✓ |
| Champion icons with mastery overlay | Portrait icons with mastery as colored border/overlay | |
| Separate mastery table | Compact table: champion / level / points | |

**User's choice:** Mastery level badges on champion pills
**Notes:** None

### Data Recency

| Option | Description | Selected |
|--------|-------------|----------|
| Relative time badge | 'Last fetched: 2d ago' next to summoner name, stale-orange after 7+ days | ✓ |
| Absolute date | 'Fetched: Mar 21, 2026' | |
| Traffic light dot | Green/yellow/red dot based on freshness thresholds | |

**User's choice:** Relative time badge
**Notes:** None

---

## Pool Analysis Section

### Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Collapsible section below champions | Toggleable 'Pool Analysis' section, collapsed by default | ✓ |
| Always-visible stats row | Compact single-row summary always visible | |
| Separate analysis tab | Tab system on player card (Overview / Analysis) | |

**User's choice:** Collapsible section below champions
**Notes:** None

### Role Flexibility Calculation

| Option | Description | Selected |
|--------|-------------|----------|
| Count distinct roles | Simple count of distinct roles played | |
| Weighted role distribution | Percentage per role from match history (e.g., 'Mid 60% / Top 25%') | ✓ |
| You decide | Claude picks based on available data | |

**User's choice:** Weighted role distribution
**Notes:** Requires storing per-match role data from Riot API teamPosition field

---

## Auto-Fetch Behavior

### Fetch Trigger

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-fetch on save | 'Save & Fetch' saves and immediately kicks off API fetches | ✓ |
| Save first, fetch separately | Save decoupled from fetch via separate button | |
| Auto-fetch per player on ID change | Debounced fetch when Riot ID field changes | |

**User's choice:** Auto-fetch on save
**Notes:** None

### Partial Failure Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Per-player status indicators | Spinner/check/X per player row, retry individual failures | ✓ |
| All-or-nothing with retry | Summary error + 'Retry Failed' button | |
| Silent failures | Empty champion list implies failure | |

**User's choice:** Per-player status indicators
**Notes:** None

### Re-fetch/Refresh

| Option | Description | Selected |
|--------|-------------|----------|
| Per-player refresh button | Small refresh icon on each player card | |
| Team-level 'Refresh All' button | Single button to re-fetch all 5 players | |
| Both per-player and team-level | Per-player refresh buttons plus 'Refresh All' at top | ✓ |

**User's choice:** Both per-player and team-level
**Notes:** None

---

## Claude's Discretion

- Exact card layout dimensions and spacing
- Collapsible section animation behavior
- Number of recent matches for role distribution
- Sequential vs parallel player fetches
- Exact stale threshold for recency coloring
- Whether last_fetched updates on failed attempts

## Deferred Ideas

None — discussion stayed within phase scope
