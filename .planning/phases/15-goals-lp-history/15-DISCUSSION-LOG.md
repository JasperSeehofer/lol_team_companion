# Phase 15: Goals & LP History - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-05
**Phase:** 15-goals-lp-history
**Mode:** `--auto` (recommended option auto-selected for every gray area)
**Areas discussed:** LP History Graph, Goal System, Goal Progress Tracking, Champion Performance Trends

---

## LP History Graph

| Option | Description | Selected |
|--------|-------------|----------|
| Inline SVG line chart | Hand-rolled SVG, consistent with `tree_graph.rs`, no new deps | ✓ |
| Canvas-based | Better for thousands of points, overkill at this scale | |
| Charts library (Plotters / similar) | Adds dependency + WASM bundle weight | |

**Auto-selected:** Inline SVG line chart (recommended default).

### LP graph time window

| Option | Description | Selected |
|--------|-------------|----------|
| Selectable 7d/30d/90d/all-time, default 30d | Matches selectable-window pattern required for trends | ✓ |
| All-time only | Simpler but less useful on long histories | |
| Last N games | Consistent with goal-progress window but harder to read on a calendar axis | |

**Auto-selected:** Selectable 7d/30d/90d/all-time, default 30d.

### LP graph y-axis scale (resolves STATE.md blocker)

| Option | Description | Selected |
|--------|-------------|----------|
| Cumulative rank score (tier × 400 + division × 100 + LP; Master+ continues past 2800) | Smooth across promos/relegation; handles Master+ no-division correctly | ✓ |
| Raw LP only (0–100) with discontinuities at division boundaries | Easy to compute but visually misleading | |
| Color-banded tier strips | Visual but hard to read precise LP | |

**Auto-selected:** Cumulative rank score.

### LP graph placement

| Option | Description | Selected |
|--------|-------------|----------|
| Solo dashboard, between ranked badge and recent matches | Preserves Phase 12 single-column stack; LP graph anchors to the ranked badge | ✓ |
| Dedicated `/rank-history` page | More room but breaks the dashboard-first pattern | |
| `/stats` page | Stats is for match analytics, not rank history | |

**Auto-selected:** Solo dashboard (between ranked badge and recent matches).

---

## Goal System

### Goal cardinality

| Option | Description | Selected |
|--------|-------------|----------|
| One active goal per type per user (rank / CS-min / deaths) | Simple v1; matches the 3 placeholder cards | ✓ |
| Multiple concurrent goals per type | Adds complexity for unclear benefit | |
| One active + history of past goals | Useful but defers cleanly to a future phase | |

**Auto-selected:** One active per type. History deferred.

### Goal storage

| Option | Description | Selected |
|--------|-------------|----------|
| New `personal_goal` table with `(user, goal_type)` unique index | Mirrors `personal_learning` pattern; room to grow | ✓ |
| Three new fields on the `user` record | Simpler now, harder to evolve (no created_at, no history) | |
| Single JSON blob field on `user` | Fights schemafull SurrealDB conventions | |

**Auto-selected:** New `personal_goal` table.

### Goal UI placement & edit affordance

| Option | Description | Selected |
|--------|-------------|----------|
| Replace `GoalPlaceholders` cards on solo dashboard; inline edit form on each card | Direct continuation of the Phase 12 placeholders; no extra navigation | ✓ |
| Dedicated `/goals` page | Adds a route + nav link for very few decisions | |
| Modal-based edit | Inconsistent with Phase 14 inline-expand precedent | |

**Auto-selected:** Inline edit on solo dashboard cards.

---

## Goal Progress Tracking

### Progress window

| Option | Description | Selected |
|--------|-------------|----------|
| Last 20 ranked solo/duo matches | Stable recent-form readout; matches LoL community norm | ✓ |
| Since goal was set | Resets on every edit; bad for users who tweak goals | |
| Last 7 days (calendar window) | Inconsistent for users who play sporadically | |
| User-selectable per goal | Adds setting complexity for v1 | |

**Auto-selected:** Last 20 ranked solo/duo matches.

### Match queue source for goals

| Option | Description | Selected |
|--------|-------------|----------|
| Solo/duo only (queue_id = 420) | Goals are personal solo improvement; flex/normals dilute the signal | ✓ |
| All ranked queues (solo + flex) | Bigger sample but mixed-mode signal | |
| All queues | Worst signal-to-noise | |

**Auto-selected:** Solo/duo only.

### Recompute timing

| Option | Description | Selected |
|--------|-------------|----------|
| Recompute on dashboard load (server fn returns derived view) | Always fresh; no stored state to drift | ✓ |
| Recompute on each match sync, store result | Faster reads but adds a write path that can desync | |
| Recompute on a cron / background job | Overkill for per-user data | |

**Auto-selected:** Recompute on dashboard load.

### Rank goal progress display

| Option | Description | Selected |
|--------|-------------|----------|
| LP delta to target on cumulative scale + progress bar | Concrete and matches the LP graph's scale | ✓ |
| ETA-based ("≈ 12 wins to target") | Speculative; relies on win-rate assumptions | |
| Streak-based (W3 / L2) | Different kind of signal, deferred | |

**Auto-selected:** LP delta + progress bar.

### Insufficient-data threshold

| Option | Description | Selected |
|--------|-------------|----------|
| < 5 solo/duo games → show "need more games" message | Avoids skewed averages on tiny samples | ✓ |
| Always show whatever average is available | Misleads on day-1 users | |
| < 10 game threshold | Stricter; users wait too long to see their goals working | |

**Auto-selected:** < 5 game threshold.

---

## Champion Performance Trends

### Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Sortable table (champion / games / win% / KDA / CS-min / damage) | Compact; reuses stats page table styling; sortable by any column | ✓ |
| Card grid (like Phase 14 learnings) | Burns vertical space when there are many champions | |
| Combined chart + table | More work; adds value only at the high end | |

**Auto-selected:** Sortable table.

### Time window

| Option | Description | Selected |
|--------|-------------|----------|
| Selectable 7d/30d/90d/all-time, default 30d | Same toggle pattern as LP graph (independent state) | ✓ |
| Fixed window | Required text says "selectable" — must offer choice | |

**Auto-selected:** Selectable 7d/30d/90d/all-time.

### Min-games filter

| Option | Description | Selected |
|--------|-------------|----------|
| < 3 games hidden by default; "show all" toggle reveals them | Removes one-off-pick noise but keeps option to inspect | ✓ |
| No filter | Trends table dominated by single-game outliers | |
| < 5 game threshold | Too aggressive on shorter windows | |

**Auto-selected:** < 3 game threshold with toggle.

### Queue source for trends

| Option | Description | Selected |
|--------|-------------|----------|
| Solo/duo + flex (both ranked queues) | Trends inform draft/pool decisions; flex is meaningful skill signal | ✓ |
| Solo/duo only (matches goal-tracking source) | Smaller sample for users who play flex more | |
| All queues including normals/ARAM | Skews CS/min and KDA wildly | |

**Auto-selected:** Solo/duo + flex.

### Placement

| Option | Description | Selected |
|--------|-------------|----------|
| New section on `/stats` page below match history | `/stats` is already the analytics home; keeps solo dashboard from sprawling | ✓ |
| New section on solo dashboard | Dashboard already has 4 sections (badge, LP graph, goals, recent matches) | |
| Dedicated `/champion-trends` page | New nav item for a single section; not worth it | |
| Add to `/champion-pool` page | Pool is about tier/comfort, not performance | |

**Auto-selected:** `/stats` page, below match history.

---

## Claude's Discretion

User did not weigh in on any of the following — Claude has flexibility:

- LP graph line color (recommend `--color-accent`).
- Tier emblem image source for y-axis labels (Data Dragon CDN vs. bundled — same as Phase 12 ranked-badge decision).
- SVG dimensions and responsive breakpoints.
- Hover tooltip styling.
- Whether to keep the existing icons on goal cards (recommend yes).
- Progress bar colors (recommend green for on-track, dimmed accent otherwise; no red).
- CS/min and deaths input bounds for goal edit form.
- Whether time-window toggles persist per-user or reset (recommend reset).
- KDA formula for trends ((K+A)/max(D,1) recommended).
- Prefetch vs lazy fetch for champion trends (recommend prefetch under `<Suspense>`).

## Deferred Ideas

- Goal history / archived goals — wait until users ask.
- Goal achievement notifications (toast / nav badge).
- Configurable goal window (10/20/50 games).
- Per-champion goals.
- Peer average comparisons (privacy + cost concerns).
- LP graph annotations for sync sessions.
- Streak tracking (W3 / L2).

---

*Auto-mode discussion completed in a single pass. CONTEXT.md is the canonical record.*
