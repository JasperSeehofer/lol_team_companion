---
phase: 28-library-ia-healing
reviewed: 2026-06-06T00:00:00Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - src/models/growth.rs
  - src/server/db.rs
  - src/pages/stats.rs
  - src/pages/match_detail.rs
  - src/pages/analytics.rs
  - src/pages/post_game.rs
  - src/pages/personal_learnings.rs
  - src/pages/rooms/library.rs
  - src/pages/rooms/mod.rs
  - src/app.rs
  - input.css
  - e2e/tests/fixtures.ts
  - e2e/tests/library.spec.ts
  - e2e/tests/pages.spec.ts
  - e2e/tests/hydration-no-panic.spec.ts
findings:
  critical: 2
  warning: 6
  info: 3
  total: 11
status: issues_found
---

# Phase 28: Library & IA Healing — Code Review Report

**Reviewed:** 2026-06-06
**Depth:** standard
**Files Reviewed:** 15
**Status:** issues_found

## Summary

The IA-healing refactor (flat routes → Library room tabs via redirect shells, five chrome-free
surfaces, and the `LibraryGrowthState` derivation) is structurally sound and follows the project's
room-system conventions (region read once at room top level per rule #58, growth tiers wired as
`data-*-tier` attributes with matching `input.css` opacity selectors, dual SSR/client redirect
pattern). Unit tests for the growth model are thorough.

However, the review surfaced two blockers and several quality defects:

1. **The Library `records` growth axis can never light up.** `get_library_growth_counts` counts
   `match` rows by `team_id`, but `team_id` is never written to any `match` row during sync. SC4's
   records axis is dead on arrival, and the e2e test masks this by only seeding the tomes/analysis
   axes.
2. **A multibyte-UTF-8 string slice in the now-Library-surfaced learning card can panic the WASM
   runtime** on free-text input.
3. **Five large standalone page components became dead code** (no longer routed) but stay `pub`, so
   the compiler will not flag them — ~1000 lines of unreachable UI.

Plus latent `.expect()` panics in two surfaces, a broken back-button on deep-linked matches, and a
mislabeled empty-state in Analytics.

No structural-findings substrate was supplied with this review.

## Critical Issues

### CR-01: Library `records_tier` is permanently stuck at 0 — `match.team_id` is never written

**File:** `src/server/db.rs:4219-4255` (consumer) / `src/server/db.rs:1466-1522` (root cause)
**Issue:** `get_library_growth_counts` derives the `records` axis from:
```
SELECT count() as n FROM match WHERE team_id = type::record('team', $team_key) GROUP ALL
```
But `store_matches_with_synced_by` (the only path that `CREATE`s `match` rows) sets only
`match_id`, `queue_id`, `game_duration`, `game_end`, and optionally `synced_by` — it **never sets
`team_id`** (schema field `team_id ON match TYPE option<record<team>>`, `schema.surql:49`). Every
`match` row therefore has `team_id = NONE`, so the count is always 0 and `records_tier` can never
exceed Seed (tier 0) regardless of how many matches a team syncs. This silently breaks the SC4
records axis (and, separately, the pre-existing analytics match-join queries at db.rs:3383, 3519,
3626 that use the same `WHERE team_id = …` predicate). The `library.spec.ts` test does not catch
this because it explicitly only seeds tomes/analysis and comments that "records_tier stays 0".
**Fix:** Either populate `team_id` on match creation, or count records consistently with how
`get_team_match_stats` already works (join through `player_match` → team members). Preferred — count
via `player_match` so it matches the data that actually drives the Stats tab:
```rust
// records: matches that have at least one player_match by a team member
let mut result = db.query(
    "SELECT count() as n FROM (
        SELECT match FROM player_match
        WHERE user IN (SELECT VALUE user FROM team_member WHERE team = type::record('team', $team_key))
        GROUP BY match
     ) GROUP ALL; \
     SELECT count() as n FROM post_game_learning WHERE team = type::record('team', $team_key) GROUP ALL; \
     SELECT count() as n FROM team_note WHERE team = type::record('team', $team_key) GROUP ALL"
).bind(("team_key", team_key)).await?;
```
Add an e2e assertion (or unit test) that exercises a non-zero `records_tier` so this cannot silently
regress again.

### CR-02: WASM-runtime panic on multibyte preview text in the Library-surfaced learning card

**File:** `src/pages/personal_learnings.rs:548-556`
**Issue:** The card preview slices `what_happened` by byte index:
```rust
if preview.len() > 80 {
    format!("{}\u{2026}", &preview[..80])
} else { preview.clone() }
```
`preview.len()` is a byte count and `&preview[..80]` slices at byte 80. If a multibyte UTF-8
character (emoji, accented letter, CJK) straddles byte boundary 80, this panics with
`byte index 80 is not a char boundary`. `what_happened` is free-form user text, so this is reachable
input. Per wasm-patterns rule #35 a panic crashes the entire WASM runtime and freezes all
interactions. This path is now rendered inside the Library room's Personal Learnings tab.
**Fix:** Slice on a char boundary:
```rust
let preview = &l.what_happened;
if preview.chars().count() > 80 {
    let truncated: String = preview.chars().take(80).collect();
    format!("{truncated}\u{2026}")
} else {
    preview.clone()
}
```

## Warnings

### WR-01: Five standalone page components are now dead code but stay `pub` (no compiler warning)

**File:** `src/pages/stats.rs:526` (`StatsPage`), `src/pages/match_detail.rs:1029` (`MatchDetailPage`), `src/pages/analytics.rs:444` (`AnalyticsPage`), `src/pages/post_game.rs:998` (`PostGamePage`), `src/pages/personal_learnings.rs:408` (`PersonalLearningsPage`)
**Issue:** `app.rs` now routes `/stats`, `/analytics`, `/post-game`, `/opponents`,
`/personal-learnings`, and `/match/:id` to thin redirect shells (`StatsRedirect`, etc.). The original
full-page components (`StatsPage`, `MatchDetailPage`, `AnalyticsPage`, `PostGamePage`,
`PersonalLearningsPage`) are no longer referenced anywhere (grep confirms zero non-definition
references). Because they are `pub fn`, Rust's dead-code lint does not fire, so ~1000 lines of
unreachable UI (including duplicated sync/filter logic in `StatsPage` and the entire
`MatchDetailPage` timeline view) linger and will rot. Their comments even say "retained … until Plan
06 redirect" — Plan 06 has landed.
**Fix:** Delete the five unused page components (and their now-unused helpers/imports), or, if
intentionally kept for a future flat-route fallback, add a brief module comment stating why and add
`#[allow(dead_code)]` so intent is explicit. Keeping the chrome-free `*Surface` components is correct;
the page wrappers are the dead part.

### WR-02: `.expect()` on context in two surfaces — latent WASM panic if ever mounted outside `LibrarySurfaces`

**File:** `src/pages/stats.rs:289-290`, `src/pages/match_detail.rs:560-561`
**Issue:**
```rust
let selected_match_id = use_context::<RwSignal<Option<String>>>()
    .expect("LibrarySurfaces provides selected_match_id");
```
This is safe today because both `StatsSurface` and `MatchDetailSurface` are only instantiated by
`LibrarySurfaces`, which provides the context before the view. But these are `pub` components; any
future caller (or a refactor that mounts a surface outside the room) turns this into a WASM-runtime
panic, contradicting wasm-patterns rule #35 ("never `.unwrap()`/`.expect()` in WASM code"). The
recent-match-list fallback in `MatchDetailSurface` already proves these surfaces want to degrade
gracefully.
**Fix:** Degrade instead of panicking:
```rust
let selected_match_id = use_context::<RwSignal<Option<String>>>()
    .unwrap_or_else(|| RwSignal::new(None));
```
(A locally-created signal means the cross-tab handoff is inert outside the room, but the surface still
renders rather than crashing the runtime.)

### WR-03: Back button is broken for deep-linked matches in `MatchDetailSurface`

**File:** `src/pages/match_detail.rs:567-572, 698`
**Issue:** The active match id is `selected_match_id.get().or_else(|| query.read().get("id"))`. For a
deep link `/library?tab=match&id=XYZ`, `selected_match_id` starts `None`, so `match_id` resolves to
the query param `XYZ` and the detail view renders. The "← All matches" button does
`selected_match_id.set(None)` — but because the `?id=XYZ` query param is still in the URL, `match_id`
immediately falls back to `XYZ` again, so the detail view never collapses to the recent-match list.
The back button is a no-op for any user who arrived via a deep link (the redirect from `/match/:id`
produces exactly this URL).
**Fix:** Either clear the query param when going back (`navigate("?tab=match", Default::default())`),
or track an explicit "cleared" state, e.g. use an `RwSignal<Option<Option<String>>>` /
`RwSignal<bool>` "show_list" flag so `None` means "show list" and wins over the query fallback after
the user clicks back.

### WR-04: Analytics empty-state mislabels a team-with-no-data user as "no team"

**File:** `src/pages/analytics.rs:142-161`
**Issue:** `let no_team = payload.tag_summaries.is_empty() && payload.plan_effectiveness.is_empty();`
But `get_analytics_data` returns empty vecs both when the user has no team *and* when the user has a
team but no tagged reviews/plans yet (analytics.rs:29-35 vs 37-44). A team that simply hasn't logged
analytics data is shown "No analytics yet … Create or join a team to get started" with a "Go to team
roster" CTA — wrong and confusing for an existing team. (Carried verbatim from the old `AnalyticsPage`
into the extracted surface, but it now ships inside the Library room.)
**Fix:** Distinguish the two states server-side — e.g. add a `has_team: bool` to `AnalyticsPayload`,
or return a typed empty variant — so the surface can show "No analytics yet, log a tagged review"
for a team vs the join-a-team CTA for the teamless case.

### WR-05: `match.team_id`-predicated analytics joins also silently return zero (same root cause as CR-01)

**File:** `src/server/db.rs:3383, 3519, 3626`
**Issue:** Three analytics queries filter `match` by `WHERE team_id = type::record('team', $team_key)`
and then join `player_match`. Since `match.team_id` is never written (see CR-01), the
`match`-statement of each batch returns zero rows, so any analytics feature that depends on resolving
a team's matches by `team_id` is silently empty. Pre-existing, but the Library phase newly depends on
the same broken column for `records_tier`, so fixing CR-01 should fix these consumers together (or
confirm they intentionally rely on a different join).
**Fix:** Audit these three queries alongside CR-01; switch the `match` lookup to the
`player_match`-via-`team_member` join used by `get_team_match_stats`, or populate `match.team_id` at
sync time and backfill existing rows.

### WR-06: `#[allow(dead_code)]` on `trends_window_to_cutoff`, which is actually used

**File:** `src/pages/stats.rs:184-194`
**Issue:** `trends_window_to_cutoff` carries `#[allow(dead_code)]` but is called from
`get_champion_trends` at stats.rs:178. A stale `#[allow]` suppresses a real future warning if the
function ever *does* become unused, masking dead code. Misleading annotation.
**Fix:** Remove the `#[allow(dead_code)]` attribute.

## Info

### IN-01: `LibrarySurfaces` accepts a `region` prop it discards

**File:** `src/pages/rooms/library.rs:295, 303-304`
**Issue:** `LibrarySurfaces(region: String)` binds `let _region = region;` and never uses it. The
comment explains it's "for prop parity with WarSurfaces". This is harmless but the dead prop adds a
clone at the call site (library.rs:253, 281) and invites confusion.
**Fix:** Drop the `region` prop from `LibrarySurfaces` unless a near-term surface needs it; remove the
`surfaces_region` clone in `LibraryRoom`.

### IN-02: Duplicated `champion_icon_url` / `format_damage` / `format_duration` helpers across surfaces

**File:** `src/pages/stats.rs:268-278`, `src/pages/match_detail.rs:101-125`, `src/pages/personal_learnings.rs:181-183`
**Issue:** `champion_icon_url`, `format_damage`, and `format_duration` (with the hardcoded Data Dragon
version `15.6.1`) are re-implemented in three+ files. The magic version string is duplicated, so a CDN
version bump requires edits in several places and risks divergence.
**Fix:** Hoist these helpers (and a single `DDRAGON_VERSION` const) into a shared module (e.g.
`src/models/` or a `ui` util) and import them.

### IN-03: Five Library panels each fire their own data fetch eagerly at room mount

**File:** `src/pages/rooms/library.rs:306-327`
**Issue:** `RoomSurface` mounts all panels eagerly (keep-mounted design, D-08). `StatsSurface`,
`MatchDetailSurface`, and `get_recent_match_ids`/`get_team_stats` consumers each create their own
`Resource::new(|| (), |_| get_team_stats())` — so opening `/library` issues `get_team_stats` (and
several other server fns) multiple times in parallel on first load even though only one tab is
visible. Correctness is unaffected (performance is out of v1 scope), but flagged because the
eager-mount design multiplies redundant round-trips; consider a shared resource provided once by
`LibrarySurfaces` if this becomes a load concern.

---

_Reviewed: 2026-06-06_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
