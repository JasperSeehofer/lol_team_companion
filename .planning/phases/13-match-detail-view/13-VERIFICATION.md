---
phase: 13-match-detail-view
verified: 2026-03-27T00:00:00Z
status: human_needed
score: 13/13 must-haves verified
re_verification: false
human_verification:
  - test: "Visual inspection of full match detail page with real match data"
    expected: "Blue and Red team scoreboard tables each show 5 players with champion icon, KDA, items, damage, gold, vision. User row has accent left border. Item icons load from Data Dragon CDN."
    why_human: "Requires a linked Riot account, synced match history, and a running server. Item icon rendering and team table layout cannot be confirmed from static analysis alone."
  - test: "Timeline bar event marker positioning and interactivity"
    expected: "Event markers appear at proportional positions along the timeline bar, clicking a marker expands the detail panel below, user's own events show accent ring highlight."
    why_human: "Timeline marker rendering depends on real TimelineEvent data from the Riot API. Filter toggle reactivity and click-to-expand behavior requires browser interaction."
  - test: "Performance breakdown bars with comparison toggle"
    expected: "Four bars (Damage Share, Vision Score, CS per Minute, Gold Earned) render with fill proportional to value. 'vs Lane Opponent' toggle button is disabled when lane role data is unavailable."
    why_human: "Bar fill calculations and the disabled state for lane opponent mode depend on real PerformanceStats data. Cannot verify correct visual normalization statically."
  - test: "Click-to-navigate from stats page match rows and solo dashboard"
    expected: "Clicking a match row on the Stats page navigates to /match/{riot_match_id}. Clicking a recent match on the Solo Dashboard navigates to /match/{match_id}."
    why_human: "The stats page uses a 'Details' link inside an expandable row (not a full-row wrap). Confirming the UX is correct — that the Details link is visible and the expand behavior is preserved — requires visual verification."
  - test: "Error state on API failure"
    expected: "When no Riot API key is configured, the match detail page shows an ErrorBanner with a Retry button. Clicking Retry calls detail.refetch()."
    why_human: "Requires triggering a real API error condition, which needs server manipulation."
---

# Phase 13: Match Detail View Verification Report

**Phase Goal:** Users can open any match from their history and see the full 10-player scoreboard with items, damage, gold, and vision — plus timeline events — without that data being fetched during the initial sync

**Verified:** 2026-03-27
**Status:** human_needed (all automated checks passed — 5 items require visual confirmation)
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Riot API match data for all 10 participants can be fetched and converted to MatchParticipant structs | VERIFIED | `fetch_full_match_detail` at riot.rs:324; `MatchParticipant` struct at match_data.rs:32 |
| 2 | Timeline events can be fetched, classified by type, and converted to TimelineEvent structs | VERIFIED | `classify_event` at riot.rs:518; `TimelineEvent` struct at match_data.rs:63; `EventCategory` enum at match_data.rs:52 |
| 3 | Performance stats (damage share, CS/min, vision, gold) can be computed from participant data | VERIFIED | `compute_performance` at riot.rs:533; `PerformanceStats` struct at match_data.rs:81; unit test `test_compute_performance_stats` passes |
| 4 | Match detail data is cached in SurrealDB and returned on cache hit without API call | VERIFIED | `get_cached_match_detail`/`store_match_detail` at db.rs:4673/4738; cache-check-then-fetch pattern in match_detail.rs:22-45; `fetch_full_match_detail` not called from sync path |
| 5 | User can navigate to /match/:id from the stats page match list | VERIFIED | stats.rs:792 `href=format!("/match/{}", riot_id)` with `stop_propagation`; cursor-pointer at stats.rs:699 |
| 6 | User can navigate to /match/:id from the solo dashboard recent matches | VERIFIED | solo_dashboard.rs:386-389 wraps rows in `<a href=format!("/match/{}", m.match_id)>` with cursor-pointer |
| 7 | User can see all 10 participants in two team tables with champion icon, KDA, items, damage, gold, vision | VERIFIED (structure) | match_detail.rs:189 item icons from DDragon CDN; border-l-4 border-accent bg-accent/10 at line 207; TeamScoreboard component present; NEEDS HUMAN for visual confirmation |
| 8 | User's own row is highlighted with accent left border | VERIFIED | match_detail.rs:207 `"h-12 flex items-center gap-3 px-3 border-l-4 border-accent bg-accent/10"` |
| 9 | User can see their performance breakdown with bar charts comparing to game average | VERIFIED (structure) | PerformanceSection component at match_detail.rs:392; "My Performance" heading at line 403; 4 PerformanceBar invocations; NEEDS HUMAN for visual bar rendering |
| 10 | User can toggle between game average and lane opponent comparison | VERIFIED | `ComparisonMode` signal at match_detail.rs:542; "vs Game Average" at line 415; "vs Lane Opponent" at line 437; disabled state when `lane_opponent_damage.is_none()` at line 419 |
| 11 | Skeleton placeholders show while data loads | VERIFIED | `SkeletonCard` called 5 times in Suspense fallback at match_detail.rs:562-566 |
| 12 | Error banner with retry button shows on API failure | VERIFIED | `ErrorBanner` at match_detail.rs:576; `detail.refetch()` wired to retry button |
| 13 | User can see timeline events placed on a horizontal bar with filter toggles and click-to-expand | VERIFIED (structure) | `fn timeline_pct` at line 123; `fn event_tooltip` at line 131; 7 filter signals (lines 546-552); "ring-2 ring-accent" at line 804; "No events match the current filters." present; NEEDS HUMAN for interactive confirmation |

**Score:** 13/13 truths verified (5 require human confirmation for visual/interactive correctness)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `schema.surql` | match_detail and match_timeline_event table definitions | VERIFIED | Lines 261, 270: both SCHEMAFULL tables defined with IF NOT EXISTS |
| `src/models/match_data.rs` | MatchParticipant, MatchDetail, TimelineEvent, PerformanceStats, ComparisonMode | VERIFIED | 114 lines; all 6 types present at lines 32, 52, 63, 81, 98, 111 |
| `src/server/riot.rs` | fetch_full_match_detail, classify_event, compute_performance | VERIFIED | 755 lines; all 3 functions present at lines 324, 518, 533 |
| `src/server/db.rs` | get_cached_match_detail, store_match_detail | VERIFIED | 5147 lines; both functions at lines 4673, 4738 |
| `src/pages/match_detail.rs` | fetch_match_detail server fn + full MatchDetailPage component | VERIFIED | 893 lines; server fn at line 5; full UI with scoreboard, timeline, performance sections |
| `src/app.rs` | Route for /match/:id | VERIFIED | Line 79: `<Route path=path!("/match/:id") view=MatchDetailPage />`; import at line 19 |
| `e2e/tests/match-detail.spec.ts` | E2e smoke test for match detail page | VERIFIED | 16-line smoke test navigating to /match/test-match-id, checking nav and body visibility |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/pages/match_detail.rs` | `src/server/db.rs` | `db::get_cached_match_detail` + `db::store_match_detail` | WIRED | Both calls present at match_detail.rs:24, 68 |
| `src/pages/match_detail.rs` | `src/server/riot.rs` | `riot::fetch_full_match_detail` | WIRED | Call at match_detail.rs:55, result consumed and returned |
| `src/server/db.rs` | `schema.surql` tables | `match_detail`, `match_timeline_event` | WIRED | db.rs:4686, 4699, 4750, 4762 query these exact tables |
| `src/pages/match_detail.rs` (component) | `fetch_match_detail` server fn | `Resource::new(match_id, fetch_match_detail)` | WIRED | match_detail.rs:539 |
| `src/pages/stats.rs` | `/match/{riot_match_id}` | href link in match row Details | WIRED | stats.rs:792 |
| `src/pages/solo_dashboard.rs` | `/match/{match_id}` | `<a href=format!(...)>` wrapper | WIRED | solo_dashboard.rs:386-389 |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `src/pages/match_detail.rs` (MatchDetailPage) | `detail` (Resource) | `fetch_match_detail` server fn → DB cache or Riot API | Yes — cache-check-then-fetch; Riot API returns real 10-player data; empty match_id returns early error | FLOWING |
| `src/server/riot.rs::fetch_full_match_detail` | `participants`, `timeline_events` | `riven` crate Riot API calls | Yes — real API call; no static fallback | FLOWING |
| `src/server/db.rs::get_cached_match_detail` | `DbMatchDetail`, `Vec<DbTimelineEvent>` | `SELECT * FROM match_detail WHERE match_id = $match_id` | Yes — real DB query | FLOWING |

---

### Behavioral Spot-Checks

| Behavior | Check | Result | Status |
|----------|-------|--------|--------|
| SSR compilation succeeds | `cargo check --features ssr` | Finished with 3 warnings (unused imports, not errors) | PASS |
| WASM compilation succeeds | `cargo check --features hydrate --target wasm32-unknown-unknown` | Finished successfully | PASS |
| All unit tests pass | `cargo test --features ssr --lib` | 92 passed, 0 failed | PASS |
| /match/:id route registered | grep in app.rs | `path!("/match/:id")` found at line 79 | PASS |
| fetch_match_detail NOT called during sync | grep in stats.rs sync path | `fetch_full_match_detail` absent from sync path | PASS |
| All 5 commits from summaries exist | git log | 808b9d3, b73b275, a36234b, e331e41, 314c8fe all verified | PASS |

---

### Requirements Coverage

| Requirement | Source Plan(s) | Description | Status | Evidence |
|-------------|---------------|-------------|--------|----------|
| MATCH-01 | 13-01, 13-02 | User can view all 10 participants in a match (KDA, items, damage, gold, vision) | SATISFIED | MatchParticipant struct with all fields; TeamScoreboard + ParticipantRow with item icons from DDragon CDN; match_detail.rs 893 lines |
| MATCH-02 | 13-01, 13-03 | User can view match timeline events (dragons, barons, towers, kills) | SATISFIED | TimelineEvent + EventCategory in models; timeline section with 6 filter toggles; event markers positioned by timestamp_ms; classify_event maps all types |
| MATCH-03 | 13-01, 13-02 | User can see personal performance breakdown (damage share, vision score, CS vs game average) | SATISFIED | PerformanceStats struct; compute_performance in riot.rs; PerformanceSection with 4 bar charts and GameAverage/LaneOpponent toggle |
| MATCH-04 | 13-01 | Match detail loads on-demand (not during sync) and caches in DB after first fetch | SATISFIED | fetch_full_match_detail only called from match_detail.rs server fn; DB cache check-then-fetch pattern at lines 22-90; store_match_detail called non-fatally after API fetch |

No orphaned requirements — all 4 IDs declared in plan frontmatter are accounted for and satisfied.

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `src/pages/match_detail.rs` | "Add Learning ->" CTA links to `/personal-learnings/new` (will 404) | Info | Intentional forward pointer for Phase 14; documented in Summary 02 as known stub. Not blocking. |

No TODO/FIXME comments, no empty implementations, no static return stubs found in match-related code.

---

### Human Verification Required

The automated verification is fully passing (13/13 truths, all artifacts exist and are substantive, all key links wired, data flows through real API/DB). The following items require a running server and real match data to confirm visual and interactive correctness:

#### 1. 10-Player Scoreboard Layout

**Test:** Register, link a Riot account on the profile page, sync matches on the Stats page, click any match row's "Details" link, and navigate to /match/{id}.
**Expected:** Blue Team and Red Team cards each show 5 rows. Each row has champion icon (28px), summoner name, KDA formatted as "K/D/A", 6 item slots (real icons or empty placeholders), damage, gold, and vision score columns. Your own row has a visible accent left border.
**Why human:** Item icon rendering from Data Dragon CDN, column alignment, and the user row highlight cannot be confirmed from static analysis.

#### 2. Timeline Bar Interaction

**Test:** On the match detail page, observe the Timeline section. Click a filter toggle pill (e.g., "Kills") to hide kill events. Click a timeline marker.
**Expected:** Markers disappear when their category is filtered out. Clicking a marker expands a detail panel below the bar with event description and involved participants. Your own events (as killer or involved participant) show an accent ring. A second click on the same marker collapses the detail panel.
**Why human:** Timeline positioning correctness and the filter/expand interaction require browser rendering.

#### 3. Performance Bars and Comparison Toggle

**Test:** Scroll to "My Performance" section. Click "vs Lane Opponent" button.
**Expected:** Four bar charts render with non-zero fill proportional to your stats. "vs Lane Opponent" either switches the comparison bars or shows as disabled (opacity-40 cursor-not-allowed) if lane role data is missing for this match.
**Why human:** Bar fill normalization and the disabled-state appearance depend on live PerformanceStats data.

#### 4. Stats Page and Solo Dashboard Click Navigation

**Test:** On Stats page, expand a match row and click the "Details" link. On Solo Dashboard (/solo), click a recent match row.
**Expected:** Both navigate to /match/{match_id}. The expand/collapse behavior on the stats page row is not broken by the Details link (stop_propagation is applied).
**Why human:** The stats page uses a "Details" link inside an expandable accordion row (not a full-row wrap). Confirming the UX — that expand still works and the Details link is visible — requires browser interaction.

#### 5. Error State

**Test:** Remove the RIOT_API_KEY from .env and reload a match detail page for an uncached match.
**Expected:** ErrorBanner appears with message "Riot API key not configured..." and a Retry button. Clicking Retry re-attempts the fetch.
**Why human:** Requires environment manipulation and server restart.

---

### Summary

Phase 13 goal is structurally achieved. All three waves delivered:

- **Wave 1 (Plan 01):** Complete data layer — schema tables, model types, Riot API extraction with teamfight detection, DB cache (cache-check-then-fetch), and server function. Unit tests for classify_event and compute_performance pass.
- **Wave 2 (Plan 02):** Full MatchDetailPage component — route at /match/:id, 10-player scoreboard tables, user row highlight, item icons, performance breakdown with comparison toggle, skeleton loading, error + retry. Click navigation added to Stats and Solo Dashboard.
- **Wave 3 (Plan 03):** Timeline section with 6 filter toggles, absolute-positioned event markers colored by team, accent ring for user's own events, click-to-expand detail panel, e2e smoke test.

The on-demand fetch constraint (MATCH-04) is verified: `fetch_full_match_detail` is only invoked from the `fetch_match_detail` server function (triggered when the user opens /match/:id), never from the sync path in stats.rs.

Five human verification items remain for visual layout, CDN image rendering, and interactive behavior confirmation. These are quality-level checks, not functional gaps — the code paths are wired and substantive.

---

_Verified: 2026-03-27_
_Verifier: Claude (gsd-verifier)_
