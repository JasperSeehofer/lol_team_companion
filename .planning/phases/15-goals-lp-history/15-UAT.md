---
status: complete
phase: 15-goals-lp-history
source: [15-01-SUMMARY.md, 15-02-SUMMARY.md, 15-03-SUMMARY.md]
started: 2026-05-06T00:00:00Z
updated: 2026-05-06T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Cold Start Smoke Test
expected: Kill any running cargo leptos watch process. Start fresh with `cargo leptos watch`. Server boots without errors. Navigate to http://127.0.0.1:3020 — homepage loads. Log in and go to /solo and /stats — both pages load without error banners.
result: pass
note: snapshotted_at datetime→string cast fix required before /solo loaded cleanly

### 2. LP History Graph Renders
expected: On /solo, an LP history graph appears below the ranked badge section. It shows a line/area chart with LP data points. If no ranked snapshot data exists yet, an empty state is shown (not an error banner). The graph has 7d / 30d / 90d / All-time selector pills visible.
result: pass
note: single data point (one day), start and end date both show 2026-05-06 — correct behavior

### 3. LP Time Window Selector
expected: On /solo, click each time-window pill (7d, 30d, 90d, All-time). The chart updates to reflect the selected window (different number of data points, or same if data is sparse). Active pill is visually highlighted. No page reload required.
result: pass
note: pill highlights correctly; single snapshot means same date across all windows — expected

### 4. LP Graph Hover Tooltip
expected: On /solo, hover over the LP chart line/area. A tooltip appears near the cursor showing the LP value (and ideally date) for the nearest data point. Tooltip disappears when mouse leaves chart area.
result: pass

### 5. Rank Target Goal Card
expected: On /solo, a "Rank Target" goal card is visible below the chart. It shows a tier+division selector (e.g. Gold I, Platinum IV). Selecting a rank and saving persists the goal. A progress bar shows current rank vs target. If no goal is set, card shows a prompt to set one.
result: pass

### 6. CS/min Goal Card
expected: On /solo, a CS/min goal card is visible. Entering a target CS/min value and saving persists it. A progress bar (or indicator) shows average CS/min vs target. The displayed average reflects actual match data.
result: pass

### 7. Deaths/game Goal Card
expected: On /solo, a Deaths/game goal card is visible. Entering a target deaths/game value and saving persists it. A progress bar (or indicator) shows average deaths/game vs target. The displayed average reflects actual match data.
result: pass

### 8. Goal Persistence
expected: After setting any goal (rank/CS/deaths) and refreshing the page, the goal value is still shown — it survived the page reload. Goals are stored per user, not just in-memory.
result: pass

### 9. Champion Trends Section on /stats
expected: On /stats page, a Champion Trends section appears after the match history. It shows a table with columns: Champion, Games, Win%, KDA, CS/min, Avg Damage. If no ranked match data exists, an empty state is shown (not an error). Time-window pills (7d/30d/90d/All-time) appear above the table.
result: pass
note: data looks identical across windows because all match data is from today — correct behavior with sparse data

### 10. Champion Trends Sorting
expected: On the Champion Trends table, clicking a column header (e.g. Games, Win%, KDA) sorts the table by that column. Clicking again reverses the sort order. The active sort column is visually indicated.
result: pass

### 11. Champion Trends Time Window
expected: On /stats, clicking a different Champion Trends time-window pill (7d/30d/90d/All-time) refetches and updates the trend data without a full page reload. The table updates to show stats only from that window.
result: pass
note: covered in test 9 — pills work, data identical across windows due to today-only match data

## Summary

total: 11
passed: 11
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

- truth: "/solo and /stats both load without error banners after fresh start"
  status: failed
  reason: "User reported: /solo throws 'Failed to deserialize field snapshotted_at on type DbRankedSnapshot: Expected string, got datetime'"
  severity: blocker
  test: 1
  root_cause: "get_latest_ranked_snapshot and compute_goal_progress SELECT snapshotted_at without <string> cast. SurrealDB stores it as datetime; DbRankedSnapshot/DbCurrentRankRow expect Option<String>. get_lp_history correctly uses <string>snapshotted_at AS snapshotted_at — same fix needed in both other queries."
  artifacts:
    - path: "src/server/db.rs:4582"
      issue: "snapshotted_at not cast to string in get_latest_ranked_snapshot SELECT"
    - path: "src/server/db.rs:4819"
      issue: "snapshotted_at not cast to string in compute_goal_progress third SELECT"
  missing:
    - "Add <string>snapshotted_at AS snapshotted_at to both queries (matching get_lp_history pattern at line 4658)"
  debug_session: ""
