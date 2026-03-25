# Phase 13: Match Detail View - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-25
**Phase:** 13-match-detail-view
**Areas discussed:** Navigation & layout, Timeline presentation, Performance breakdown, Loading & caching UX

---

## Navigation & Layout

### How should users access a match detail view?

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated route | Click match row → /match/:id full page. Back button returns. Follows page-per-feature pattern. | ✓ |
| Inline expand | Click row → accordion expands below. No navigation. | |
| Modal/overlay | Click row → large modal with scoreboard + timeline. | |

**User's choice:** Dedicated route
**Notes:** Consistent with existing page-per-feature pattern (draft, game-plan, etc.)

### How should the 10-player scoreboard be laid out?

| Option | Description | Selected |
|--------|-------------|----------|
| Classic two-team table | Blue team top, red team below. Columns: champion, name, KDA, items, damage, gold, vision. | ✓ |
| Interleaved by role | Matchups: Top vs Top, Jungle vs Jungle. Emphasizes lane matchups. | |
| You decide | Claude picks best layout. | |

**User's choice:** Classic two-team table
**Notes:** Mirrors post-game screen in League client

### Should the current user's row be highlighted?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, highlight row | Subtle accent background on user's own row | ✓ |
| No highlighting | All rows uniform | |
| You decide | Claude picks | |

**User's choice:** Yes, highlight row

### Should item icons use Data Dragon images or text?

| Option | Description | Selected |
|--------|-------------|----------|
| Item images | 6 small square icons per player from Data Dragon CDN, tooltip on hover | ✓ |
| Text item names | List names as text | |
| You decide | Claude picks | |

**User's choice:** Item images

---

## Timeline Presentation

### How should match timeline events be displayed?

| Option | Description | Selected |
|--------|-------------|----------|
| Chronological event list | Vertical list ordered by game time | |
| Grouped by type | Events grouped into sections (objectives, towers, kills) | |
| Visual timeline bar | Horizontal bar with event markers at timestamps | ✓ |
| You decide | Claude picks | |

**User's choice:** Visual timeline bar

### Event marker interaction?

| Option | Description | Selected |
|--------|-------------|----------|
| Tooltip on hover | Small popover with timestamp, event type, participants | |
| Expand section below | Clicking expands detail row below timeline | |
| Both | Hover for tooltip, click for expanded details | ✓ |

**User's choice:** Both — tooltip on hover + click to expand
**Notes:** User selected both options

### Which event types on the timeline?

| Option | Description | Selected |
|--------|-------------|----------|
| Objectives (dragons, baron, herald) | Major neutral objectives | ✓ |
| Tower kills | Tower destructions | ✓ |
| Champion kills (first blood, multikills, aces) | Notable kill events only | ✓ |
| Inhibitors | Inhibitor kills | ✓ |

**User's choice:** All four, plus wards placed, recalls, and teamfight indicators (4+ participants)
**Notes:** User added ward placements, back calls, and teamfight detection beyond the offered options

### Filter toggles for event categories?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, toggle filters | Icon buttons above timeline to show/hide categories | ✓ |
| Show all, no filtering | All events always visible | |
| You decide | Claude determines | |

**User's choice:** Yes, toggle filters

### Color coding?

| Option | Description | Selected |
|--------|-------------|----------|
| Team colors (blue/red) | Events tinted by team | |
| Color by event type | Color represents what happened | |
| Both — team + type | Team color as background, event type as icon | ✓ |

**User's choice:** Both — team color + event type differentiation

### Highlight user's own events?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, highlight user's events | Special indicator (glow, ring, star) on user's events | ✓ |
| No personal highlighting | All team events look the same | |
| You decide | Claude picks | |

**User's choice:** Yes, highlight user's events

---

## Performance Breakdown

### Visualization style?

| Option | Description | Selected |
|--------|-------------|----------|
| Bar charts with game average | Horizontal bars, value filled, average as marker | ✓ |
| Radial/spider chart | Spider chart with multiple axes | |
| Plain number cards | Grid of stat cards with value + average + delta | |

**User's choice:** Bar charts with game average

### Metrics included?

| Option | Description | Selected |
|--------|-------------|----------|
| Damage share % | % of team's total damage | ✓ |
| Vision score | Vision score vs game average | ✓ |
| CS/min | Creep score per minute vs average | ✓ |
| Gold earned | Total gold vs average | ✓ |

**User's choice:** All four metrics

### Comparison baseline?

| Option | Description | Selected |
|--------|-------------|----------|
| Game average | Compare against all 10 players | |
| Role-based comparison | Compare against lane opponent | |
| Both | Game average default, toggle to role comparison | ✓ |

**User's choice:** Both — game average default with toggle to role-based

### Include KDA in performance section?

| Option | Description | Selected |
|--------|-------------|----------|
| Include KDA bar | Add KDA ratio bar to performance section | |
| Skip — scoreboard has it | KDA already prominent in scoreboard | ✓ |
| You decide | Claude decides | |

**User's choice:** Skip — scoreboard has it

### Text verdicts?

| Option | Description | Selected |
|--------|-------------|----------|
| Numbers only | Let bars and numbers speak | |
| Add short verdicts | One-liner labels like "Above average" | |
| You decide | Claude picks | ✓ |

**User's choice:** You decide (Claude's discretion)

---

## Loading & Caching UX

### Loading state?

| Option | Description | Selected |
|--------|-------------|----------|
| Skeleton placeholders | Page layout with grey shimmer placeholders | ✓ |
| Spinner with message | Centered spinner with loading text | |
| You decide | Claude picks | |

**User's choice:** Skeleton placeholders

### Cache indicator?

| Option | Description | Selected |
|--------|-------------|----------|
| No indicator | Instant load is self-evident | ✓ |
| Subtle timestamp | "Fetched 2h ago" text | |
| You decide | Claude picks | |

**User's choice:** No indicator

### Error state?

| Option | Description | Selected |
|--------|-------------|----------|
| Error banner + retry button | Existing ErrorBanner with retry action | ✓ |
| Inline error in page | Error text in page layout | |
| You decide | Claude picks | |

**User's choice:** Error banner + retry button

---

## Claude's Discretion

- Text verdicts on performance bars (numbers only vs short labels)
- Exact icon shapes for timeline event types
- User event highlight style (glow, ring, star)
- Teamfight detection heuristic
- Role detection approach for role-based comparison toggle

## Deferred Ideas

None — discussion stayed within phase scope
