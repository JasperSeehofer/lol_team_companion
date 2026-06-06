---
phase: 28-library-ia-healing
verified: 2026-06-06T00:00:00Z
status: gaps_found
score: 4/5 must-haves verified
overrides_applied: 0
gaps:
  - truth: "A team with post-game reviews, team notebook entries, and match syncs sees more atmospheric detail than a fresh team — Library growth axes: records, tomes, analysis."
    status: failed
    reason: "SC4 names three specific axes (records, tomes, analysis). The records axis is permanently stuck at tier 0 because get_library_growth_counts counts match rows by team_id, but store_matches_with_synced_by (the only match-creation path) never writes team_id. Every match row has team_id = NONE, so match_syncs is always 0. The e2e test in library.spec.ts explicitly excludes records: it seeds only tomes+analysis and comments 'records_tier stays 0'. Additionally, the CR-02 UTF-8 panic (personal_learnings.rs:551-553 slices by byte index, not char boundary) can crash the WASM runtime on any multibyte input in the Personal Learnings surface, violating wasm-patterns rule #35."
    artifacts:
      - path: "src/server/db.rs"
        issue: "get_library_growth_counts (~4226) queries 'SELECT count() as n FROM match WHERE team_id = type::record(...)' — match.team_id is never written by store_matches_with_synced_by (~1466-1522), so match_syncs always returns 0, records_tier is permanently 0."
      - path: "src/pages/personal_learnings.rs"
        issue: "Line 551-553 slices &preview[..80] by byte index. If a multibyte UTF-8 character straddles byte 80, this panics and crashes the WASM runtime. PersonalLearningsSurface is now wired into LibrarySurfaces (library.rs:325), making the panic reachable from /library."
    missing:
      - "Fix store_matches_with_synced_by to set team_id on the match row at creation time (or rewrite get_library_growth_counts to count via player_match → team_member join, matching the approach used by get_team_match_stats)."
      - "Fix the UTF-8 slice: use preview.chars().count() > 80 / preview.chars().take(80).collect() instead of &preview[..80] (wasm-patterns rule #35)."
      - "Add an e2e assertion or unit test that exercises a non-zero records_tier so it cannot silently regress again."
human_verification:
  - test: "Verify /library tab switching does not remount the atmosphere (no flash/reload of background art between tabs)"
    expected: "Switching between Stats, Match Detail, Analytics, Post-Game, and Personal Learnings tabs keeps the parallax art stable — the AtmosphereRoot div remains in the DOM with the same structure."
    why_human: "Tab-switch atmosphere-remount behavior requires visual observation in a real browser; Playwright does not check for DOM node identity across interactions."
  - test: "Verify /personal-learnings/new still loads the create-learning form (not a redirect)"
    expected: "Navigating to /personal-learnings/new renders the learning creation form, not a redirect to /library."
    why_human: "Route ordering correctness in Leptos router (specificity-based) is compiler-confirmed, but the rendered form content and absence of unexpected redirects should be spot-checked in a browser after the phase lands."
---

# Phase 28: Library & IA Healing Verification Report

**Phase Goal:** Users can reach the Library at /library and access stats, match detail, analytics, post-game review, and team learnings as tabbed surfaces under a hall-of-records atmosphere; previously siloed features (personal learnings, opponents) are now reachable inside their intent rooms while their legacy flat routes remain valid deep-link targets.
**Verified:** 2026-06-06
**Status:** gaps_found
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can navigate to /library and browse stats, match detail, analytics, post-game, and team learnings as surfaces WITHOUT the atmosphere remounting between tabs. | VERIFIED (human spot-check recommended) | `app.rs:158` — `<Route path=path!("/library") view=LibraryRoom />` is present. `library.rs:295-329` — `LibrarySurfaces` composes all 5 surfaces (`StatsSurface`, `MatchDetailSurface`, `AnalyticsSurface`, `PostGameSurface`, `PersonalLearningsSurface`) inside a single `RoomSurface` tab strip. `pages.spec.ts:25` — `/library` is in `AUTHED_PAGES` with content match `/stats\|match\|analytics\|post.game\|learnings/i`. |
| 2 | A link to /personal-learnings (old flat route) arrives at the learnings surface within the Library — resolves rather than 404ing. | VERIFIED | `app.rs:160` routes `/personal-learnings` to `PersonalLearningsRedirect`. `library.rs:113-123` implements `PersonalLearningsRedirect` — dispatches `redirect_to_library_personal()` (SSR 302 → `/library?tab=personal`) and `window.location().set_href("/library?tab=personal")` (client-side). Route `/personal-learnings/new` declared at `app.rs:161` resolves to `NewLearningPage` directly (Leptos specificity-based routing). |
| 3 | A link to /opponents (old flat route) arrives at the Opponents surface within the War Room — IA-healed flat routes redirect into their intent room. | VERIFIED | `app.rs:152` routes `/opponents` to `OpponentsRedirect`. `library.rs:125-137` implements `OpponentsRedirect` — dispatches `redirect_to_war_room_opponents()` (SSR 302 → `/war-room?tab=opponents`) and `window.location().set_href("/war-room?tab=opponents")` (client-side). `/stats`, `/analytics`, `/post-game` redirects also present (app.rs:147, 151, 154). `/match/:id` carries `id` param through redirect (library.rs:145-168). |
| 4 | A team with post-game reviews, team notebook entries, and match syncs sees more atmospheric detail than a fresh team (Library growth axes: records, tomes, analysis). | FAILED (BLOCKER) | **tomes+analysis only work.** The tomes axis (post_game_learning + team_note) and analysis axis (post_game_learning) both derive from tables that are correctly populated — `get_library_growth_counts` counts them via `WHERE team = type::record('team', $team_key)` which is correct. The CSS selectors at `input.css:456-464` are present and map `data-records-tier`/`data-tomes-tier`/`data-analysis-tier` to overlay opacity. The `AtmosphereRoot` at `parallax.rs:343-345` renders all three data-attributes reactively. The e2e seed helper `seedLibraryTier1Team` creates a post-game review and asserts `max(tomes, analysis) ≥ 1`. **BUT:** The records axis named in SC4 is permanently broken — `get_library_growth_counts:4226` queries `match WHERE team_id = ...`, but `store_matches_with_synced_by:1503-1505` never writes `team_id`, so `match_syncs = 0` always, `records_tier = 0` always. SC4 explicitly names all three axes. Additionally, `personal_learnings.rs:551-553` slices user text by byte index (not char boundary), a WASM-crashing bug per wasm-patterns rule #35, now reachable via `PersonalLearningsSurface` in the Library. |
| 5 | /library appears in the e2e smoke suite and passes the hydration-no-panic canary in BOTH regions. | VERIFIED | `pages.spec.ts:25` — `/library` added to `AUTHED_PAGES` with content regex. `hydration-no-panic.spec.ts:88` — `/library` added to `SCOPED_ROUTES` (covers both demacia and pandemonium via the `for (const region of ...)` loop at line 145). Rule #58 is followed: `LibraryRoom` reads `InitialTheme` once at top level (library.rs:224), passes `region` as plain prop, never reads context inside reactive primitives. |

**Score:** 4/5 truths verified

---

### CR-01 Assessment: Records Growth Axis Is Dead

The code review finding is confirmed. Tracing the data path:

1. `get_library_growth_counts` (db.rs:4227): `SELECT count() as n FROM match WHERE team_id = type::record('team', $team_key) GROUP ALL`
2. `store_matches_with_synced_by` (db.rs:1501-1505): `CREATE match SET match_id = $match_id, queue_id = $queue_id, game_duration = $game_duration, game_end = ...` — no `team_id` field, ever.
3. Schema: `schema.surql` defines `team_id ON match TYPE option<record<team>>` — the field exists but is never written.

Result: every match row has `team_id = NONE`, the count query returns 0 rows, `match_syncs = 0`, `records_tier = tier_for(0, [1, 10, 30]) = Tier::Seed (0)` — permanently.

**SC4 judgment:** SC4 is FAILED as a success criterion. SC4's exact wording — "match syncs sees more atmospheric detail … Library growth axes: records, tomes, analysis" — names all three axes. The records axis cannot light up. A partial pass (tomes+analysis work) does not satisfy the stated criterion that match syncs contribute. A team that syncs 30 matches sees the same empty records overlay as a fresh team.

The e2e test (`library.spec.ts` "growth tiers") explicitly acknowledges this in its comment: `"(records_tier stays 0 — that requires Riot API match syncs)"` — but this comment conflates "hard to test" with "broken by design". The records axis doesn't stay 0 because of a testing limitation; it stays 0 because `team_id` is never written. The test masks the defect by only asserting `max(tomes, analysis) ≥ 1`.

### CR-02 Assessment: WASM Panic on Multibyte Preview Text

Confirmed at `src/pages/personal_learnings.rs:550-556`:

```rust
let preview = &l.what_happened;
if preview.len() > 80 {
    format!("{}\u{2026}", &preview[..80])  // panics if byte 80 is not a char boundary
} else {
    preview.clone()
}
```

`preview.len()` is a byte count. `&preview[..80]` panics with `byte index 80 is not a char boundary` on any input where a multibyte UTF-8 sequence straddles byte 80. `what_happened` is free-form user text. `PersonalLearningsSurface` is now wired into `LibrarySurfaces` (library.rs:325), making this panic directly reachable from `/library`. Per wasm-patterns rule #35, this crashes the entire WASM runtime and freezes all interactions.

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/pages/rooms/library.rs` | LibraryRoom + LibrarySurfaces + redirect shells | VERIFIED | All 5 surface imports, 6 redirect server fns, 6 redirect components, growth state server fn present |
| `src/models/growth.rs` | LibraryGrowthCounts, LibraryGrowthState, unit tests | VERIFIED | `LibraryGrowthCounts`, `LibraryGrowthState`, `from_counts`, 4 unit tests (empty/records/tomes/analysis) present |
| `src/server/db.rs` | get_library_growth_counts | STUB (defective) | Function exists and queries, but match_syncs query is broken (team_id never written) |
| `src/components/room/parallax.rs` | AtmosphereRoot with data-records/tomes/analysis-tier attributes | VERIFIED | Lines 343-345 render all three data-attributes reactively |
| `input.css` | Library tier→opacity selectors (records/tomes/analysis × tiers 1/2/3) | VERIFIED | Lines 456-464: 9 selectors present and correct |
| `e2e/tests/library.spec.ts` | Growth tiers seed-and-assert test | VERIFIED (partial) | Test exists, seeds tomes+analysis, checks max ≥ 1, asserts overlay opacity. Does NOT assert records_tier because the axis is broken — comment acknowledges this but frames it as a testing limitation rather than a code defect. |
| `e2e/tests/fixtures.ts` | seedLibraryTier1Team helper | VERIFIED | Lines 198-245: seeds via /library?tab=post-game, fills post-game review, clicks Save Review |
| `e2e/tests/pages.spec.ts` | /library in AUTHED_PAGES | VERIFIED | Line 25 present |
| `e2e/tests/hydration-no-panic.spec.ts` | /library in SCOPED_ROUTES | VERIFIED | Line 88 present |
| `src/app.rs` | All redirect routes + /library route | VERIFIED | Lines 147, 151, 152, 154, 158, 159, 160, 161 all present |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `/library` route | `LibraryRoom` | `app.rs:158` | WIRED | `<Route path=path!("/library") view=LibraryRoom />` |
| `/stats` route | `StatsRedirect` | `app.rs:147` | WIRED | Routes to redirect shell |
| `/analytics` route | `AnalyticsRedirect` | `app.rs:154` | WIRED | Routes to redirect shell |
| `/post-game` route | `PostGameRedirect` | `app.rs:151` | WIRED | Routes to redirect shell |
| `/personal-learnings` route | `PersonalLearningsRedirect` | `app.rs:160` | WIRED | Routes to redirect shell |
| `/opponents` route | `OpponentsRedirect` | `app.rs:152` | WIRED | Routes to redirect shell |
| `/match/:id` route | `MatchRedirect` | `app.rs:159` | WIRED | Routes to redirect shell, id param preserved |
| `/personal-learnings/new` route | `NewLearningPage` | `app.rs:161` | WIRED | Exact match, not redirected |
| `LibraryRoom` → `LibrarySurfaces` | 5 surfaces | `library.rs:281` | WIRED | All 5 `*Surface` components composed |
| `LibraryRoom` → growth state | `get_library_growth_state` | `library.rs:227` | WIRED | Resource created, tiers derived, passed to AtmosphereRoot |
| `get_library_growth_state` → `get_library_growth_counts` | `db.rs:4219` | `library.rs:209` | WIRED (defective) | Call is wired; records axis broken at DB query level |
| `AtmosphereRoot` data-*-tier | CSS overlays | `input.css:456-464` | WIRED | 9 Library selectors match `data-{records,tomes,analysis}-tier` |
| `StatsRedirect` → `/library?tab=stats` | SSR+client | `library.rs:68-78` | WIRED | ServerAction + window.location |
| `PersonalLearningsRedirect` → `/library?tab=personal` | SSR+client | `library.rs:113-123` | WIRED | ServerAction + window.location |
| `OpponentsRedirect` → `/war-room?tab=opponents` | SSR+client | `library.rs:125-137` | WIRED | ServerAction + window.location |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| `AtmosphereRoot` (records overlay) | `records_tier` | `get_library_growth_counts` → `match WHERE team_id = ...` | NO — match.team_id never written | HOLLOW — wired but records data disconnected |
| `AtmosphereRoot` (tomes overlay) | `tomes_tier` | `get_library_growth_counts` → `post_game_learning WHERE team = ...` | YES — team field is written by `store_post_game_review` (db.rs:1868) | FLOWING |
| `AtmosphereRoot` (analysis overlay) | `analysis_tier` | `get_library_growth_counts` → `post_game_learning WHERE team = ...` | YES — same query as tomes | FLOWING |

---

### Requirements Coverage

| Requirement | Description | Status | Evidence |
|-------------|-------------|--------|----------|
| ROOM-05 | Hall of Records / Library hosts stats, match detail, analytics, post-game, and team learnings as surfaces | PARTIAL | Route, room composition, 5 surfaces, redirects, CSS overlays all present. Records growth axis broken (cannot fulfill "match syncs" part of SC4). |
| ROOM-07 | IA healing — previously-siloed features are reachable in their intent room; legacy flat routes still resolve as deep links | VERIFIED | All 6 flat routes (`/stats`, `/analytics`, `/post-game`, `/personal-learnings`, `/opponents`, `/match/:id`) redirect to their intent room. `/personal-learnings/new` is preserved. |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/pages/personal_learnings.rs` | 551-553 | `&preview[..80]` — byte-index string slice on free-form user text | BLOCKER | Panics WASM runtime on multibyte UTF-8 at boundary; crashes all interactions on /library Personal Learnings tab |
| `src/server/db.rs` | 4227 | `WHERE team_id = type::record(...)` on match table — field never written | BLOCKER | records_tier permanently 0; SC4 records axis dead |
| `src/pages/stats.rs` | 526 | `pub fn StatsPage` dead code (no longer routed) | WARNING | ~1000 lines of unreachable UI across 5 files; no compiler warning due to `pub` |
| `src/pages/stats.rs` | 289-290 | `.expect("LibrarySurfaces provides selected_match_id")` | WARNING | Latent WASM panic if surface mounted outside LibrarySurfaces |
| `src/pages/match_detail.rs` | 560-561 | `.expect("LibrarySurfaces provides selected_match_id")` | WARNING | Same as above |

---

### Behavioral Spot-Checks

The dev server is running at http://127.0.0.1:3020. HTTP spot-checks are not performed (WASM fetch cannot reach localhost per wasm-patterns rule #45, and this verification uses code analysis as the primary signal).

| Behavior | Evidence | Status |
|----------|----------|--------|
| `/library` route resolves | `app.rs:158` route present; `pages.spec.ts:25` in AUTHED_PAGES | PASS (code) |
| `/opponents` redirects to `/war-room?tab=opponents` | `app.rs:152` → `OpponentsRedirect` → `library.rs:125-137` | PASS (code) |
| `/personal-learnings` redirects to `/library?tab=personal` | `app.rs:160` → `PersonalLearningsRedirect` → `library.rs:113-123` | PASS (code) |
| Rule #58 compliance (region read once) | `library.rs:224` — `let region = use_context::<InitialTheme>()...` at top level, never inside reactive primitives | PASS (code) |
| records_tier can exceed 0 | `db.rs:1501-1505` — CREATE match never sets team_id | FAIL (code) |

---

### Human Verification Required

#### 1. Tab-switch atmosphere stability

**Test:** Navigate to `/library`. Switch through all five tabs (Stats, Match Detail, Analytics, Post-Game, Personal Learnings). Watch the background art layer.
**Expected:** The parallax planes and growth overlays do not flash, reload, or remount when switching tabs — only the surface panel content changes.
**Why human:** Tab-switch DOM node identity cannot be reliably asserted by Playwright without explicit DOM observation hooks not present in the current e2e suite.

#### 2. /personal-learnings/new form preservation

**Test:** Navigate to `/personal-learnings/new` while authenticated.
**Expected:** The learning creation form renders (title input, what happened textarea, what I learned textarea, etc.) — NOT a redirect to `/library`.
**Why human:** Leptos specificity-based routing should guarantee this, but the rendered form content and absence of a redirect should be verified by navigating in an actual browser after the phase lands.

---

### Gaps Summary

**Two blockers, one WASM safety gap:**

1. **records_tier is permanently 0 (CR-01).** SC4 explicitly names "records" as one of three Library growth axes. The query that derives it (`match WHERE team_id = ...`) is broken because `team_id` is never written to any match row. A team that syncs 100 matches sees the identical empty records overlay as a team with 0 matches. The e2e test deliberately skips asserting `records_tier ≥ 1` (comment: "records_tier stays 0 — requires Riot API match syncs") but the issue is not a testing limitation — it is a broken field. Fix: populate `team_id` in `store_matches_with_synced_by`, or rewrite `get_library_growth_counts` to count via `player_match → team_member` join (mirroring `get_team_match_stats`). Then add a test or unit test that catches this axis at non-zero.

2. **WASM panic on multibyte preview text (CR-02).** `personal_learnings.rs:551-553` slices `&preview[..80]` by byte index on free-form user text. Any input with a multibyte UTF-8 character (emoji, accented letter, CJK) at or before byte position 80 panics with `byte index 80 is not a char boundary`, crashing the WASM runtime and freezing all interactions. This surface is now rendered inside the Library room as the fifth tab. Fix: `preview.chars().count() > 80 / preview.chars().take(80).collect()`.

Both gaps must be resolved before phase 28 can be considered complete against its stated success criteria.

---

_Verified: 2026-06-06_
_Verifier: Claude (gsd-verifier)_
