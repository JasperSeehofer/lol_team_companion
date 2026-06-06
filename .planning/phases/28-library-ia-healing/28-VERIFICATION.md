---
phase: 28-library-ia-healing
verified: 2026-06-06T13:35:00Z
status: passed
score: 5/5 must-haves verified
overrides_applied: 0
human_uat: passed (28-HUMAN-UAT.md — all 3 items pass; 2 UAT-surfaced room-feel defects fixed in ffc06f2)
re_verification:
  previous_status: gaps_found
  previous_score: 4/5
  gaps_closed:
    - "CR-01: records growth axis permanently 0 — get_library_growth_counts now counts via player_match→team_member join; two unit tests cover distinct-dedup and zero-member short-circuit."
    - "CR-02: WASM multibyte byte-slice panic in personal_learnings.rs:551 — now uses chars().count()/chars().take(80).collect() instead of &preview[..80]."
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Verify /library tab switching does not remount the atmosphere (no flash/reload of background art between tabs)"
    expected: "Switching between Stats, Match Detail, Analytics, Post-Game, and Personal Learnings tabs keeps the parallax art stable — the AtmosphereRoot div remains in the DOM with the same structure."
    why_human: "Tab-switch DOM node identity cannot be reliably asserted by Playwright without explicit DOM observation hooks not present in the current e2e suite."
  - test: "Verify /personal-learnings/new still loads the create-learning form (not a redirect)"
    expected: "Navigating to /personal-learnings/new renders the learning creation form (title input, what happened textarea, what I learned textarea, etc.) — NOT a redirect to /library."
    why_human: "Leptos specificity-based routing is compiler-confirmed, but the rendered form content and absence of an unexpected redirect should be spot-checked in a browser after the phase lands."
  - test: "Verify growth overlays fade in on a seeded team at /library"
    expected: "A team with at least one post-game review shows a visibly non-zero opacity on the tomes or analysis overlay (.room-overlay--tomes / .room-overlay--analysis). A fresh team with no data shows all overlays at opacity 0. The records overlay stays at opacity 0 (no Riot API key in dev)."
    why_human: "The e2e library.spec.ts 'growth tiers' test covers computed opacity assertion in CI; this is a defense-in-depth human spot-check for the end-of-phase UAT pass."
---

# Phase 28: Library & IA Healing Verification Report

**Phase Goal:** Users can reach the Library at /library and access stats, match detail, analytics, post-game review, and team learnings as tabbed surfaces under a hall-of-records atmosphere; previously siloed features (personal learnings, opponents) are now reachable inside their intent rooms while their legacy flat routes remain valid deep-link targets.
**Verified:** 2026-06-06
**Status:** human_needed (all 5 SCs verified; 3 browser-only confirmations remain for end-of-phase UAT)
**Re-verification:** Yes — after gap closure (previous: gaps_found 4/5)

---

## Gap Closure Confirmation

### CR-01: Records Growth Axis (Previously BLOCKER)

**Status: CLOSED**

`get_library_growth_counts` (db.rs:4219-4281) no longer queries `match WHERE team_id = ...`. The rewrite:

1. Step 1 — resolves team member user IDs via `SELECT user FROM team_member WHERE team = type::record('team', $team_key)` (same join strategy as `get_team_match_stats`).
2. Step 2 — short-circuits to 0 if the team has no members (zero-member guard).
3. Step 3 — counts distinct matches with `RETURN array::len(array::distinct(SELECT VALUE match FROM player_match WHERE user IN [...]))`. This correctly deduplicates matches played by multiple team members and excludes matches belonging to non-team users.

The old `WHERE team_id = type::record(...)` pattern is entirely absent from this function. The two other occurrences in db.rs (lines 3383, 3519, 3626) are in analytics functions (`get_team_match_stats` and related) that are unrelated to the Library growth counts.

**Unit test coverage (both pass):**

- `server::db::tests::library_records_counts_distinct_matches_via_player_match` — seeds team + 2 members + 2 matches (one shared between both members), verifies distinct count = 2, and verifies a non-team user's match is excluded.
- `server::db::tests::library_records_zero_when_no_team_members` — verifies the zero-member short-circuit returns `match_syncs = 0`.

**E2e seeding limitation (non-blocking note):** The e2e test `library.spec.ts` "growth tiers" cannot seed the records axis to tier > 0 because `store_matches_with_synced_by` requires a real Riot API response. The test explicitly documents this, references both unit tests by name, and guards the CSS wiring with a `records-tier-0 → opacity 0` assertion (line 101). This is an adequate coverage posture: the COUNT QUERY correctness is proven at unit level; the CSS rendering path for tier > 0 is structurally identical to tomes/analysis (same `input.css` selector pattern, same `AtmosphereRoot` data-attribute wiring), which ARE asserted at non-zero by the e2e seed. The test was NOT weakened.

### CR-02: WASM Multibyte Byte-Slice Panic (Previously BLOCKER)

**Status: CLOSED**

`src/pages/personal_learnings.rs:550-556` now reads:

```rust
if preview.chars().count() > 80 {
    format!("{}\u{2026}", preview.chars().take(80).collect::<String>())
} else {
    preview.clone()
}
```

No residual `&preview[..80]` byte-slice pattern exists anywhere in this file. The fix correctly uses `chars().count()` (character count) for the length check and `chars().take(80).collect::<String>()` (character-boundary-safe) for truncation — per wasm-patterns rule #35.

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can navigate to /library and browse stats, match detail, analytics, post-game, and team learnings as surfaces WITHOUT the atmosphere remounting between tabs. | VERIFIED (human spot-check recommended) | `app.rs:158` — `/library` route present. `library.rs:295-329` — `LibrarySurfaces` composes all 5 surfaces inside a single `RoomSurface` tab strip. `pages.spec.ts:25` — `/library` in `AUTHED_PAGES`. |
| 2 | A link to /personal-learnings (old flat route) arrives at the learnings surface within the Library — resolves rather than 404ing. | VERIFIED | `app.rs:160` → `PersonalLearningsRedirect` → SSR 302 + client `window.location` to `/library?tab=personal`. `/personal-learnings/new` at `app.rs:161` → `NewLearningPage` directly (Leptos specificity routing). |
| 3 | A link to /opponents (old flat route) arrives at the Opponents surface within the War Room — IA-healed flat routes redirect into their intent room. | VERIFIED | `app.rs:152` → `OpponentsRedirect` → SSR 302 + client redirect to `/war-room?tab=opponents`. `/stats`, `/analytics`, `/post-game` redirects also present. `/match/:id` carries id param through redirect. |
| 4 | A team with post-game reviews, team notebook entries, and match syncs sees more atmospheric detail than a fresh team (Library growth axes: records, tomes, analysis). | VERIFIED | **Records axis:** `get_library_growth_counts` now counts via `player_match→team_member` join (CR-01 closed). Two unit tests pass (distinct-dedup + zero-member short-circuit). **Tomes axis:** counts `post_game_learning + team_note WHERE team = ...` — unchanged and correct. **Analysis axis:** counts `post_game_learning WHERE team = ...` — unchanged and correct. **CSS wiring:** `input.css:456-464` — 9 Library tier→opacity selectors (`data-records/tomes/analysis-tier` × tiers 1/2/3) present. **DOM wiring:** `parallax.rs:343-345` renders all three data-attributes reactively. **WASM safety:** CR-02 byte-slice panic eliminated (personal_learnings.rs:551-552). **E2e:** `library.spec.ts` "growth tiers" seeds tomes/analysis to tier ≥ 1, asserts overlay opacity > 0 for active tiers and opacity = 0 for tier-0 axes (including records). |
| 5 | /library appears in the e2e smoke suite and passes the hydration-no-panic canary in BOTH regions. | VERIFIED | `pages.spec.ts:25` — `/library` in `AUTHED_PAGES`. `hydration-no-panic.spec.ts:88` — `/library` in `SCOPED_ROUTES`. Rule #58 compliance: `LibraryRoom` reads `InitialTheme` once at top level (library.rs:224), passes `region` as plain prop, never reads context inside reactive primitives. |

**Score:** 5/5 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/pages/rooms/library.rs` | LibraryRoom + LibrarySurfaces + redirect shells | VERIFIED | All 5 surface imports, 6 redirect server fns, 6 redirect components, growth state server fn present |
| `src/models/growth.rs` | LibraryGrowthCounts, LibraryGrowthState, unit tests | VERIFIED | `LibraryGrowthCounts`, `LibraryGrowthState`, `from_counts`, 4 unit tests (empty/records/tomes/analysis) present |
| `src/server/db.rs` | get_library_growth_counts via player_match join | VERIFIED | CR-01 closed: counts via team_member→player_match join; 2 new unit tests pass |
| `src/components/room/parallax.rs` | AtmosphereRoot with data-records/tomes/analysis-tier attributes | VERIFIED | Lines 343-345 render all three data-attributes reactively |
| `input.css` | Library tier→opacity selectors (records/tomes/analysis × tiers 1/2/3) | VERIFIED | Lines 456-464: 9 selectors present and correct |
| `e2e/tests/library.spec.ts` | Growth tiers seed-and-assert test | VERIFIED | Seeds tomes+analysis to tier ≥ 1; asserts overlay opacity > 0 for active tiers and = 0 for tier-0 (including records). Records seeding limitation documented with unit test references. |
| `e2e/tests/fixtures.ts` | seedLibraryTier1Team helper | VERIFIED | Lines 198-245: seeds via /library?tab=post-game, fills post-game review, clicks Save Review |
| `e2e/tests/pages.spec.ts` | /library in AUTHED_PAGES | VERIFIED | Line 25 present |
| `e2e/tests/hydration-no-panic.spec.ts` | /library in SCOPED_ROUTES | VERIFIED | Line 88 present |
| `src/app.rs` | All redirect routes + /library route | VERIFIED | Lines 147, 151, 152, 154, 158, 159, 160, 161 all present |
| `src/pages/personal_learnings.rs` | Safe UTF-8 preview truncation | VERIFIED | Line 551-552: `chars().count() > 80` / `chars().take(80).collect::<String>()` — CR-02 closed |

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
| `get_library_growth_state` → `get_library_growth_counts` | `db.rs:4219` | `library.rs:209` | WIRED | Call wired; records axis now counts correctly via player_match join |
| `AtmosphereRoot` data-*-tier | CSS overlays | `input.css:456-464` | WIRED | 9 Library selectors match `data-{records,tomes,analysis}-tier` |
| `StatsRedirect` → `/library?tab=stats` | SSR+client | `library.rs:68-78` | WIRED | ServerAction + window.location |
| `PersonalLearningsRedirect` → `/library?tab=personal` | SSR+client | `library.rs:113-123` | WIRED | ServerAction + window.location |
| `OpponentsRedirect` → `/war-room?tab=opponents` | SSR+client | `library.rs:125-137` | WIRED | ServerAction + window.location |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| `AtmosphereRoot` (records overlay) | `records_tier` | `get_library_growth_counts` → `player_match WHERE user IN [team members]` | YES — CR-01 closed; query counts distinct matches via team_member join | FLOWING (unit-verified; e2e seeding requires Riot API key) |
| `AtmosphereRoot` (tomes overlay) | `tomes_tier` | `get_library_growth_counts` → `post_game_learning + team_note WHERE team = ...` | YES — team field written by `store_post_game_review` | FLOWING |
| `AtmosphereRoot` (analysis overlay) | `analysis_tier` | `get_library_growth_counts` → `post_game_learning WHERE team = ...` | YES — same query path as tomes | FLOWING |

---

### Unit Test Results

```
running 7 tests
test models::growth::tests::library_analysis_tracks_post_game_reviews ... ok
test models::growth::tests::library_empty_counts_are_all_seed ... ok
test models::growth::tests::library_records_tracks_match_syncs ... ok
test models::growth::tests::library_tomes_combines_reviews_and_notes ... ok
test models::growth::tests::library_combines_learnings_and_notes ... ok
test server::db::tests::library_records_zero_when_no_team_members ... ok
test server::db::tests::library_records_counts_distinct_matches_via_player_match ... ok

test result: ok. 7 passed; 0 failed
```

Full suite: `cargo test --features ssr --lib` = 154+ passed / 0 failed.

---

### Requirements Coverage

| Requirement | Description | Status | Evidence |
|-------------|-------------|--------|----------|
| ROOM-05 | Hall of Records / Library hosts stats, match detail, analytics, post-game, and team learnings as surfaces | VERIFIED | Route, room composition, 5 surfaces, redirects, CSS overlays all present and functional. All three growth axes (records/tomes/analysis) now produce real data. |
| ROOM-07 | IA healing — previously-siloed features are reachable in their intent room; legacy flat routes still resolve as deep links | VERIFIED | All 6 flat routes redirect to their intent room. `/personal-learnings/new` is preserved. |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/pages/stats.rs` | 526 | `pub fn StatsPage` dead code (no longer routed) | WARNING | ~1000 lines of unreachable UI across 5 files; no compiler warning due to `pub` |
| `src/pages/stats.rs` | 289-290 | `.expect("LibrarySurfaces provides selected_match_id")` | WARNING | Latent WASM panic if surface mounted outside LibrarySurfaces |
| `src/pages/match_detail.rs` | 560-561 | `.expect("LibrarySurfaces provides selected_match_id")` | WARNING | Same as above |

The two BLOCKER anti-patterns from the previous verification (byte-slice panic, broken DB query) are resolved.

---

### Behavioral Spot-Checks

| Behavior | Evidence | Status |
|----------|----------|--------|
| `/library` route resolves | `app.rs:158` route present; `pages.spec.ts:25` in AUTHED_PAGES; `npx playwright test library.spec.ts` = 1 passed | PASS |
| records axis produces non-zero counts | `db.rs:4219-4281` player_match join; 2 unit tests pass | PASS (unit) |
| preview truncation is char-safe | `personal_learnings.rs:551-552` uses `chars().count()`/`chars().take(80)` | PASS |
| Rule #58 compliance (region read once) | `library.rs:224` — `let region = use_context::<InitialTheme>()...` at top level | PASS |
| Both compile targets clean | `cargo check --features ssr` + `cargo check --features hydrate --target wasm32-unknown-unknown` | PASS |

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

#### 3. Growth overlays fade in on a seeded team

**Test:** Create a team, add one post-game review via `/library?tab=post-game`, then reload `/library`.
**Expected:** The tomes or analysis overlay (`.room-overlay--tomes` / `.room-overlay--analysis`) is visibly non-zero opacity. A fresh team (no reviews) shows all overlays at opacity 0. The records overlay stays at opacity 0 (no Riot API key in dev).
**Why human:** The e2e test covers computed opacity assertion in CI; this is a defense-in-depth visual confirmation for the end-of-phase UAT pass, and also lets you confirm the visual appearance is actually perceptible (not just technically > 0).

---

### Records Axis E2e Seeding Note

The records axis (`data-records-tier`) cannot be seeded to tier > 0 in the e2e suite because `store_matches_with_synced_by` requires a real Riot API response — there is no UI path to seed `player_match` rows without a live API key. This is an infrastructure constraint, not a code defect. The records COUNT QUERY correctness is proven at the unit level by two dedicated tests. The CSS rendering path for records tier > 0 is structurally identical to tomes/analysis (same `input.css` selector pattern, same `AtmosphereRoot` data-attribute wiring), which are asserted at non-zero by the e2e seed. The `library.spec.ts` test explicitly documents this limitation and references the unit tests, and it still guards the tier-0 overlay opacity = 0 assertion for the records axis.

---

_Verified: 2026-06-06_
_Re-verification: gap closure confirmed_
_Verifier: Claude (gsd-verifier)_
