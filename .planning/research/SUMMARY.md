# Project Research Summary

**Project:** LoL Team Companion v1.2 — Solo Mode & Match Intelligence
**Domain:** Brownfield solo player mode addition to an existing Leptos/SurrealDB team companion app
**Researched:** 2026-03-25
**Confidence:** HIGH

## Executive Summary

This milestone adds solo queue tracking, match intelligence, and personal improvement tooling to an existing team-scoped Leptos 0.8 / SurrealDB 3.x app. The core stack is frozen — the only new dependency is `leptos-chartistry 0.2.3`, the only Leptos 0.8-compatible pure-Rust SVG charting library, needed for LP trend and champion performance charts. All Riot API data (ranked tier, match timeline, full 10-player match detail) is accessible via the already-pinned `riven ^2` crate at its resolved 2.78.0 version. No version bumps are required.

The competitive landscape analysis shows that personal journaling, matchup notes, and the feedback loop from match history back into draft preparation are genuine gaps that no existing tool (OP.GG, Mobalytics, U.GG, Porofessor, tracker.gg) fills. The unique value proposition is private, self-hosted personal improvement tied directly to the draft tool — not a public stats tracker. The recommended build order is: solo mode infrastructure and data model first (to establish correct context discriminators), then match detail, then the learnings journal, then goals/LP tracking, and finally the cross-feature draft integration.

The most critical risk is data model design: the shared `match` table's `team_id` field creates ambiguous nullability (is `NULL` solo or unassigned?), the existing `post_game_learning` table cannot hold solo entries without a schema change, and per-user solo context must live on `player_match` not on the shared `match` record. These discriminators must be established before any solo sync code is written, or recovering from mixed data requires a costly migration. Rate limit management is the second critical risk: match timeline fetches must be on-demand and cached, never eager, to avoid exhausting the development API key (100 req/2-min) during bulk sync.

## Key Findings

### Recommended Stack

The stack requires exactly one new crate. `leptos-chartistry 0.2.3` is the only charting library that targets Leptos 0.8 directly with an `ssr` feature, renders pure SVG (no canvas, no JS), and has no npm dependency — matching all project constraints. Add it as a shared dependency (not SSR-gated). All Riot API work uses the existing `riven = "^2"` at its resolved 2.78.0 version, which already exposes `league_v4().get_league_entries_by_puuid()` for ranked data and `match_v5().get_timeline()` for timeline events.

**Core technologies:**
- `leptos-chartistry 0.2.3`: multi-series time-series SVG charts — only Leptos 0.8 compatible charting option without JS/npm
- `riven ^2` (existing): gains `get_league_entries_by_puuid` (ranked LP/tier) and `get_timeline` (match events/frames) — no version bump
- `leptos 0.8 / axum 0.8 / SurrealDB 3.x` (existing): unchanged; all new features follow established patterns

**Critical version note:** `leptos-chartistry` must have the `ssr` feature enabled; interactive tooltips should be gated with `#[cfg(feature = "hydrate")]` since SSR renders static SVG.

**What NOT to use:** D3/Chart.js WASM bindings (bloat + npm), eager timeline fetching, raw timeline JSON stored in SurrealDB, `get_league_entries_for_summoner` (requires extra summoner_v4 round-trip vs direct PUUID method).

### Expected Features

**Must have (table stakes):**
- Match history list with W/L, champion, KDA, CS/min per game — data already partially stored, needs queue filter
- Per-match detail view showing all 10 participants (items, damage, gold, vision) — users expect this from any tracking tool
- Current ranked tier, division, and LP display — every tracker surfaces this as the entry point
- Champion KDA and win rate aggregated over time — derivable from existing `player_match` records
- Solo vs flex queue filter (queue_id 420 = solo) — already in `MatchData`, needs surfacing
- Solo/team context toggle with solo dashboard shell — without this, solo users have no entry point

**Should have (competitive differentiators):**
- Personal learnings journal (champion/matchup-tagged) — fills the biggest gap; community builds spreadsheets to fill this void
- LP history graph (snapshot-based, not per-game — Riot API limitation) — present in Mobalytics behind paywall; absent elsewhere
- Match-to-learning one-click flow — directly closes the improvement loop
- Goal setting (rank target, CS/min threshold, death limit) — auto-tracks against match history
- Matchup notes surfaced in draft Intel sidebar — no competitor connects personal game experience back into draft

**Defer (v1.3+):**
- Practice list auto-progress tracking — requires stable match history aggregation first
- Champion pool gap detection from solo history — high value cross-join; complex
- Match timeline objectives/events — available in Riot API; adds depth but high effort
- Auto LP sync polling — rate limit management needed; manual entry sufficient for v1.2

### Architecture Approach

The app adds user-scoped solo features alongside the existing team-scoped system. The mode toggle is a persistent `mode: string` field on the `user` table (not a WASM context signal, which would reset on hard navigation). Five new DB tables are required (`solo_goal`, `ranked_snapshot`, `personal_learning`, `match_participant`, `practice_item`), plus an additive `synced_by` field on `match` for solo ownership and a `mode` field on `user`. Three new pages (`SoloDashboard`, `LearningsPage`, `GoalsPage`), four new components (`MatchDetailDrawer`, `RankedBadge`, `LpHistoryGraph`, `PerformanceBar`), and modifications to `StatsPage`, `DraftPage`, `ProfilePage`, and `Nav`.

**Major components:**
1. `SoloDashboard` (`src/pages/solo_dashboard.rs`) — personal progress hub with independent Resources per panel (ranked, recent matches, recent learnings, goal status)
2. `MatchDetailDrawer` (`src/components/match_detail.rs`) — inline drawer (not a new route) showing full 10-player scoreboard + timeline events; lazy-fetched and DB-cached
3. `LearningsPage` (`src/pages/learnings.rs`) — personal journal with champion/matchup tagging; URL-param pre-fill from match detail; data surfaced in `DraftPage` Intel sidebar
4. `GoalsPage` (`src/pages/goals.rs`) — goal setting + `LpHistoryGraph` + ranked snapshot sync
5. `db.rs` solo section — all new solo DB functions prefixed `DbSolo*`; shared match ingestion extracted to `upsert_match_and_player_stat()` to avoid duplicating sync logic

**Key patterns to follow:**
- Lazy match detail fetch with DB cache (check `match_participant` before calling Riot API)
- JSON strings for heterogeneous sub-objects (items array, goal targets) — matches existing `mastery_data_json` pattern
- Mode-aware server-side branching — server fns check `user.mode` and return appropriate data; WASM sends no mode logic
- Independent Resources per dashboard panel — failure isolation

### Critical Pitfalls

1. **`match` table context ambiguity** — `team_id IS NULL` conflates solo matches with unassigned team matches. Add an explicit `synced_by: option<record<user>>` field to `match` before any solo sync code is written. Team stat queries must explicitly filter `team_id != NONE`; solo queries filter `synced_by = $user`. Must be resolved in Phase 1 — recovery requires a data migration.

2. **Per-user solo context on the shared `match` record** — the `match_id_unique` index means two users who played the same game share one `match` record. Setting `solo_owner` on the shared record means the second user's sync silently inherits the first user's owner. All per-user data (`lp_before`, `lp_after`, solo context) belongs on `player_match`, not `match`.

3. **Eager timeline fetches in bulk sync** — a 20-match sync with timelines = 61 API calls = 61% of the 2-minute rate limit window. Timeline must be fetched on-demand only (when user opens a specific match detail), never in `sync_solo_stats()`. Cache in DB after first fetch.

4. **`sync_solo_stats` copied from `sync_team_stats`** — the team version returns `Err` if the user has no team. Solo mode is precisely the case where no team exists. Write solo sync from scratch; apply rule 44 (return `Ok(Vec::new())` for absent optional entities).

5. **Solo toggle as WASM context signal** — WASM signals reset on hard navigation (login/logout use `window.location().set_href()`). Persist `mode` in the user DB record; read via the existing `get_current_user()` call that already runs on every nav render.

## Implications for Roadmap

Based on research, the architecture's phase dependency graph is clear and should drive the roadmap structure directly.

### Phase 1: Solo Mode Infrastructure & Data Model
**Rationale:** All subsequent phases depend on correct context discriminators and the mode toggle. Schema mistakes here require costly migration. This phase has no external feature dependencies.
**Delivers:** Mode toggle persists across page reload; solo dashboard skeleton; stats page shows solo queue filter; match sync works without a team; ranked snapshot captured at first sync.
**Addresses:** Solo/team context toggle, current rank display, solo stats sync (table stakes)
**Avoids:** Pitfall 1 (match context ambiguity), Pitfall 4 (toggle as WASM signal), Pitfall 6 (per-user data on shared match record), Pitfall 8 (team guard copied to solo sync)
**New schema:** `DEFINE FIELD IF NOT EXISTS mode ON user`; `DEFINE FIELD IF NOT EXISTS synced_by ON match`; `DEFINE TABLE IF NOT EXISTS ranked_snapshot`

### Phase 2: Match Detail View
**Rationale:** The most-requested single feature; requires solo sync (Phase 1) to have match records to detail. Establishes the lazy-fetch + DB-cache pattern needed by all subsequent Riot API calls. Must resolve timeline storage format before first fetch.
**Delivers:** Click any match in history → full 10-player scoreboard with items, damage, gold, vision; participant self-highlight.
**Addresses:** Per-match detail view (P1 feature — HIGH value, MEDIUM cost)
**Avoids:** Pitfall 2 (rate limit from eager timeline fetch), Pitfall 3 (raw timeline JSON bloat ~300-500 KB/match), Pitfall 10 (re-fetching on every detail view)
**New schema:** `match_participant`, `match_timeline_event` tables
**Adds crate:** `leptos-chartistry 0.2.3` to `Cargo.toml` (used in Phase 4 charts)

### Phase 3: Personal Learnings Journal
**Rationale:** Highest-ROI differentiator; low implementation cost. Depends on match detail (Phase 2) for the one-click flow entry point. Establishes the `personal_learning` table that Phase 5 (draft integration) requires.
**Delivers:** Add learnings from match detail with one click; browse/filter by champion/matchup/tag; data ready for draft sidebar.
**Addresses:** Personal learnings journal (core differentiator), match-to-learning one-click flow (P1 features)
**Avoids:** Pitfall 11 (solo learnings stored in `post_game_learning` team table — must be a separate `personal_learning` table with `record<user>` scope)
**New schema:** `personal_learning` table

### Phase 4: Goals & LP History
**Rationale:** LP graph requires ranked snapshot data accumulating since Phase 1. Goal progress requires champion trends derivable from `player_match`. Standalone phase that does not block Phase 5.
**Delivers:** Set rank/metric goals; LP history graph (snapshot-based); goal progress on solo dashboard; champion performance trends (win rate, KDA, CS/min per configurable window).
**Addresses:** LP history graph, goal setting, champion performance trends (P1/P2 features)
**Avoids:** Pitfall 9 (LP history from per-game match data — league-v4 returns current snapshot only; derive history from successive snapshots)
**New schema:** `solo_goal`, `practice_item` tables
**Uses:** `leptos-chartistry` for LP trend line chart and champion trend bars

### Phase 5: Draft Integration & Cross-Feature Connections
**Rationale:** Depends on Phase 3 (`personal_learning` data) and Phase 1 (solo stats from `player_match`). Delivers the unique value proposition: personal game experience feeds back into draft decisions.
**Delivers:** Personal matchup notes surface in draft Intel sidebar when a champion is picked; champion performance trends visible as win rate badge on champion pool cards; the full match-to-learning-to-draft loop is complete.
**Addresses:** Matchup notes surfaced in draft (P2), champion performance in pool (P2)
**Avoids:** No new pitfalls — additive queries to existing pages with no new tables

### Phase Ordering Rationale

- Phase 1 must be first: the `match` context discriminator and mode toggle are foundational; every other phase builds on correct data scoping. Doing Phase 2 before Phase 1 means match detail records have no `synced_by` owner and team/solo data is ambiguously mixed.
- Phase 2 before Phase 3: the one-click match-to-learning flow requires match detail to exist; without it, learnings are entered manually (workable but misses the core UX that is the differentiator).
- Phase 3 before Phase 5: the draft Intel sidebar queries `personal_learning` — no data to surface if Phase 3 has not been built.
- Phase 4 is independent of Phases 3 and 5 and could be built in parallel, but is sequenced after Phase 2 to keep focus on one feature area at a time.
- Phase 5 is last because it is additive (no new tables) and depends on all prior phases having data to surface.

### Research Flags

Phases with well-documented patterns (skip research-phase):
- **Phase 1:** User mode field + server-side branching follows `champion_pool` user-scoped table pattern exactly
- **Phase 2:** Lazy fetch + DB-cache pattern is documented with working code examples in ARCHITECTURE.md
- **Phase 3:** URL param pre-fill follows existing game plan `draft_id` pattern; `personal_learning` schema is simple
- **Phase 5:** Additive queries to existing pages; no new patterns or crates

Phases that may benefit from a focused research pass before implementation starts:
- **Phase 4 (LP delta computation):** LP gain/loss across tier/division boundaries has edge cases (relegation, promotion series, skipping divisions in Master+). Worth a targeted implementation research pass — not a full research-phase invocation.
- **Phase 1 (region support decision):** `riot.rs` hardcodes `EUROPE`/`EUW1` (Pitfall 7). Decide scope before writing solo sync: EUW-only is acceptable for v1.2 if documented. Multi-region requires adding a region selector to account linking in Phase 1 — retrofitting later touches every Riot API call.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | One new crate (`leptos-chartistry`); all other tech existing and proven. Crate version verified against leptos 0.8 dependency; no open SSR issues found. |
| Features | MEDIUM | Competitive landscape well-documented. Gap analysis (personal journaling, matchup notes) verified via community spreadsheet download counts. Priority matrix is opinionated but grounded in stated differentiators. |
| Architecture | HIGH | Based on direct codebase inspection of all affected files + riven 2.78.0 API verification. Build order derived from actual data dependencies in schema. Code patterns verified against working examples in codebase. |
| Pitfalls | HIGH | All 11 pitfalls verified against actual code in `riot.rs`, `schema.surql`, `db.rs`, `stats.rs`. Rate limit numbers from official Riot developer documentation. Timeline size estimate from peer-reviewed paper. |

**Overall confidence:** HIGH

### Gaps to Address

- **Region support scope decision:** `riot.rs` hardcodes EUW. The team must decide before Phase 1 whether v1.2 is EUW-only (acceptable to defer) or multi-region (requires adding region selector to account linking). If deferred, document it so non-EUW users get a clear error, not a silent failure.
- **LP delta boundary logic:** No research source clearly documents how to handle LP delta computation across tier/division changes (e.g. from promo series results, or Master/Grandmaster/Challenger which have no divisions). Validate the snapshot-diff approach handles promotions, relegations, and the Challenger boundary before finalising the `GoalsPage` LP graph.
- **`db.rs` size management:** At ~4,000 lines, `db.rs` risks becoming unmaintainable. The PITFALLS research recommends a `// === SOLO MODE ===` section with `DbSolo*` struct prefixes. Consider whether this milestone is the trigger for splitting into `db/team.rs` and `db/solo.rs` modules — not required for v1.2, but worth flagging to the roadmapper.

## Sources

### Primary (HIGH confidence)
- Codebase direct inspection: `src/server/riot.rs`, `schema.surql`, `src/server/db.rs`, `src/pages/stats.rs`, `src/models/` — ground truth for existing patterns and constraints
- `riven 2.78.0` source (`~/.cargo/registry`): verified `get_timeline`, `get_league_entries_by_puuid`, `Participant` struct fields
- `https://docs.rs/leptos-chartistry/latest/leptos_chartistry/` — chart types, SSR feature, Leptos 0.8 dependency confirmed
- `https://docs.rs/riven/latest/riven/` — version 2.78.0; all new API endpoints confirmed
- `https://hextechdocs.dev/rate-limiting/` — development key limits: 20 req/s, 100 req/2-min

### Secondary (MEDIUM confidence)
- `https://mobalytics.gg/gpi/` — GPI feature description (official); used for competitor feature table
- `https://mobalytics.gg/blog/how-to-track-lp-by-game` — LP tracking approach (official blog)
- `https://tracker.gg/lol/` — LP climb tracking features
- `https://github.com/ikhebgeenaccount/Excel-Match-History-Analyzer` (5k+ downloads) — community evidence of journaling gap
- `https://github.com/ForeverTalone/Game-Review-Spreadsheet` — community evidence of post-game review gap
- `https://www.mdpi.com/2076-3417/15/13/7190` — timeline data size estimate (~300-500 KB raw per match)

### Tertiary (LOW confidence)
- `https://happysmurf.com/blog/what-is-opgg/` — OP.GG feature overview (third-party review)
- `https://www.itero.gg/articles/what-is-the-best-league-of-legends-companion-app-in-2025` — competitor comparison (self-published)

---
*Research completed: 2026-03-25*
*Ready for roadmap: yes*
