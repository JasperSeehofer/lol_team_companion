# Project Research Summary

**Project:** LoL Team Companion — Cross-Feature Intelligence Milestone
**Domain:** Competitive LoL team preparation — connecting siloed features into a coherent intelligence layer
**Researched:** 2026-03-14
**Confidence:** HIGH

## Executive Summary

This is a brownfield intelligence milestone on a complete Rust/Leptos/SurrealDB team management app. Auth, drafts, champion pools, stats, game plans, post-game reviews, opponent scouting, and action items all exist but operate as isolated islands. The research conclusion is clear: the data model already has the foreign key linkages needed (`draft_id`, `plan_id`, `source_review`), the stack requires no new dependencies, and the architecture solution is a three-layer build order — navigation wiring first, cross-table aggregation queries second, dashboard intelligence surface third. Build in this order because each layer depends on the previous one being in place.

The recommended approach is strictly rule-based: pure Rust suggestion functions operating on data already in SurrealDB, fetched via batched multi-statement queries, surfaced through independent per-panel Leptos Resources with Suspense skeleton fallbacks. No ML libraries, no WebSockets, no new crate dependencies. The market benchmark (Mobalytics, ProComps, iTero) confirms that ban recommendations from own stats, draft-to-game-plan pipeline, and contextual pool warnings are table stakes — their absence signals an unfinished product. The smart dashboard that surfaces "what's next" is the genuine differentiator: no existing LoL team tool has a prioritised prep surface.

The primary risks are not technical but architectural: SurrealDB aggregate views have documented GROUP BY bugs in 3.x and must be avoided in favour of Rust-level aggregation, stale Leptos resources do not auto-invalidate across page boundaries after cross-feature mutations, and champion name normalization (Riot API returns `"AurelionSol"` while Data Dragon returns `"Aurelion Sol"`) will cause silent zero-result joins if not standardised at ingestion. All three risks are preventable with patterns that already exist in the codebase.

---

## Key Findings

### Recommended Stack

The stack is locked and requires no additions for this milestone. Zero new Cargo dependencies. All cross-feature intelligence is implementable with the existing crate set: batched SurrealDB queries, pure Rust aggregation functions, and Leptos `Resource` + `Suspend` for progressive loading.

**Core technologies:**
- **SurrealDB 3.x (SurrealKV):** Multi-statement batch queries (semicolon-chained, indexed by position) are the correct aggregation mechanism. Pre-computed aggregate views must be avoided — multiple open bugs in 3.x with GROUP BY (issues #4881, #2988, #2825). Existing `get_draft_outcome_stats()` is the proven pattern.
- **Leptos 0.8 Resource + Suspend:** Independent `Resource::new()` per dashboard panel, each wrapped in its own `<Suspense>` with `animate-pulse` skeleton fallback. `Suspend::new(async move { ... })` enables clean multi-resource await inside a single `<Suspense>` block with SSR performance benefits.
- **Leptos `use_query_map()`:** URL query params (`?draft_id=`, `?plan_id=`) are the correct mechanism for cross-feature navigation context — no global signal needed, works with SSR, bookmarkable within the team.
- **Pure Rust suggestion functions:** A dedicated `src/server/suggestions.rs` with functions that take aggregated data structs and return `Vec<Suggestion>`. Rule-based thresholds (`if rate < 0.4 { suggest_ban(champ) }`), no external ML crate.
- **Tailwind `animate-pulse`:** Built-in skeleton screen components using `bg-surface/50 animate-pulse rounded` — zero new dependencies.

See `.planning/research/STACK.md` for pattern code examples.

### Expected Features

The feature bar is set by Mobalytics, ProComps, and iTero. The research identifies a clear priority ordering from the competitive landscape survey.

**Must have (table stakes) — absence signals an unfinished product:**
- Ban recommendations informed by own match history win rates
- Draft pulls from team champion pools — pool warnings when a player has no coverage for their assigned lane
- Draft → game plan pipeline with CTA that pre-fills picks, bans, and side
- Post-game review → action item auto-creation
- Empty states with contextual CTAs on all panels (not blank pages)
- Loading skeletons on all Suspense fallbacks
- Consistent mutation feedback (save/delete confirmations) across all pages

**Should have (differentiators) — genuine advantage over market tools:**
- Smart dashboard surfacing "what's next" (upcoming opponent context, incomplete workflows, open action items)
- Opponent tendency sidebar inside the draft planner
- Contextual "last time we faced this team" recall in opponent profiles and draft
- Post-game lesson recall when creating a game plan for a repeat opponent
- Win condition tracker (declared win conditions vs. outcomes over time)

**Defer (v2+):**
- Draft outcome correlation (needs 10+ tagged outcomes to be meaningful)
- Full game day wizard flow (existing checklist is sufficient)
- Real-time collaborative editing (WebSockets)
- AI/LLM draft recommendations

**Feature dependency ordering (from research):**
Champion pool data is required for pool warnings and ban recommendations. Draft → game plan pipeline is the lowest-hanging fruit (FK exists, low new code). Smart dashboard is highest leverage but depends on all lower-level links being in place first.

See `.planning/research/FEATURES.md` for full feature dependency graph.

### Architecture Approach

The architecture solution is three ordered layers. The foreign key linkages already exist in the schema — the missing piece is surfacing them in the UI. New intelligence queries belong in a clearly-marked `// === INTELLIGENCE / AGGREGATION ===` section at the bottom of `db.rs`, returning purpose-built structs (e.g. `DbChampionWinRate`, `DashboardSummary`) rather than reusing existing model types.

**Major components:**
1. **URL query params layer** — `use_query_map()` on GamePlanPage, PostGamePage, and DraftPage accepts `?draft_id=`, `?plan_id=`, `?series_id=` context. Zero new server code; uses existing FK fields.
2. **Aggregation query layer in `db.rs`** — New batch multi-statement queries: `get_champion_performance_summary`, `get_recent_post_game_patterns`, `get_dashboard_summary`. Aggregate in Rust, not SurrealQL. No schema changes required.
3. **Dashboard intelligence surface** — Upgraded `/team/dashboard` with independent per-panel Resources. Each panel has its own `<Suspense>` so a failing query doesn't block others.
4. **Suggestion engine in `src/server/suggestions.rs`** — Pure Rust functions over aggregated data structs, called from server functions before returning to the client. `Suggestion { category, text, priority, source_ids }`.

**Critical anti-patterns to avoid (from research):**
- SurrealDB pre-computed views for analytics (GROUP BY bugs in 3.x)
- Global Leptos signal for cross-feature state (stale signal lifecycle bugs)
- Mega server function for the entire dashboard (ties all panels to the slowest query)
- Duplicating multi-table JOIN logic across page files (double maintenance surface)

See `.planning/research/ARCHITECTURE.md` for full data flow diagram and component boundary table.

### Critical Pitfalls

1. **Schema cross-links stored as loose strings cause silent zero-result joins** — `game_plan.draft` is already `option<string>`. Any query that joins on this must strip the table prefix before `type::record('draft', $stripped_id)`. New internal-only linking fields should use `record<table>` type. Centralize stripping in `db::get_draft_by_id(id: &str)` — never duplicate it.

2. **SurrealDB aggregate views are unreliable in 3.x — always use Rust-level aggregation** — Issues #4881, #2988, #2825 are open against GROUP BY in aggregate views. Fetch raw records via batch SELECT, aggregate with Rust iterators. This is already the pattern in `get_draft_outcome_stats()`.

3. **Stale Leptos resources across page boundaries after cross-feature mutations** — After post-game review auto-creates action items, the action items page resource was fetched before the mutation. Fix: design UX to navigate to the affected page (which fetches fresh on mount) with a contextual link ("View N new action items").

4. **Reactive dependency pollution when adding intel panels to large pages** — Any `Resource::new(|| selected_champion.get(), ...)` inside `draft.rs` (2,614 lines) refetches on every champion pick. Use `get_untracked()` in panel closures. Keep intel resources triggered by explicit user actions, not implicit draft state changes.

5. **Champion name normalization mismatch** — Riot API returns `"AurelionSol"` (no space), Data Dragon returns `"Aurelion Sol"`. Cross-feature champion joins produce zero results without a single normalization function (lowercase + strip spaces) applied at every ingestion point.

See `.planning/research/PITFALLS.md` for 8 additional moderate and minor pitfalls with phase-specific warnings.

---

## Implications for Roadmap

The research establishes a clear dependency chain. Navigation wiring must come before aggregation queries (to populate FK links with real data). Aggregation queries must come before the dashboard intelligence surface (which depends on them). The suggestion engine and inline contextual features come last (depend on both being proven).

### Phase 1: Navigation Wiring and Pipeline CTAs

**Rationale:** The FK fields exist. The first step is making them reachable via UX. This phase requires zero new server functions beyond one pre-fill helper — only `use_query_map()` param reading and CTA button placement. It delivers the navigational skeleton of the entire prep loop before any query complexity is added.

**Delivers:** Users navigate draft → game plan → post-game without copy-pasting IDs. "Create game plan" CTA on draft page. "Write post-game review" CTA on game plan page. `/game-plan?draft_id=` pre-fills picks/bans/side from draft via `get_plan_init(draft_id: Option<String>)`.

**Addresses:** Draft → game plan pipeline (table stakes), empty states with contextual CTAs (table stakes).

**Avoids:** Pitfall 6 (loose string ID mismatch) — centralize `db::get_draft_by_id` normalization here before other phases depend on it.

**Research flag:** Standard patterns. Skip research-phase.

---

### Phase 2: Cross-Table Aggregation Queries

**Rationale:** With FK links being populated by real navigation (Phase 1), aggregation queries have meaningful data. This phase adds all multi-table JOIN queries to `db.rs` in a dedicated intelligence section. No UI changes — only server-side query logic and new typed structs validated with integration tests.

**Delivers:** `get_champion_performance_summary(team_id)`, `get_recent_post_game_patterns(team_id)`, `get_open_action_items(team_id)`, `get_dashboard_summary(team_id)`. Each is a batch multi-statement query aggregated in Rust. New `src/models/analytics.rs` with purpose-built structs (`ChampionPerformanceSummary`, `DashboardSummary`).

**Addresses:** Ban recommendations from own stats, champion pool gap detection, post-game pattern surfacing.

**Avoids:** Pitfall 2 (no aggregate views), Pitfall 5 (db.rs growth managed with section header + unique struct names), Pitfall 10 (champion name normalization applied in Rust aggregation).

**Research flag:** Standard patterns. Skip research-phase.

---

### Phase 3: Dashboard Intelligence Surface

**Rationale:** Phase 2 produces the aggregation functions. Phase 3 wires them into `/team/dashboard` as independent per-panel Resources with Suspense skeleton fallbacks. This is the highest-value user-visible deliverable.

**Delivers:** Smart dashboard with open action items panel, recent post-game summaries panel, upcoming opponent context, champion pool gap warnings. Each panel loads independently. Skeleton fallbacks during load. Empty states with guidance CTAs for new teams with no history.

**Addresses:** Smart dashboard (differentiator), loading skeletons (table stakes), empty states (table stakes), consistent mutation feedback.

**Avoids:** Pitfall 7 (Riot API never called implicitly on dashboard load — DB-cached data only), Pitfall 8 (`Ok(Vec::new())` for all empty states, never `Err`), Pitfall 11 (one focused Resource per panel group, not one mega-resource).

**Research flag:** Standard patterns. Skip research-phase.

---

### Phase 4: Inline Contextual Intelligence

**Rationale:** With aggregation queries proven and dashboard flow established, Phase 4 surfaces intelligence inline on feature pages. Requires the most care due to reactive signal lifecycle risks in large existing pages.

**Delivers:** Opponent tendency sidebar in DraftPage, ban suggestions from champion stats, post-game lesson recall in game plan creation for repeat opponents, win condition tracker, loading skeletons on all Suspense fallbacks app-wide.

**Addresses:** Opponent tendency sidebar (differentiator), ban recommendations (table stakes), post-game lesson recall (differentiator), win condition tracker (differentiator).

**Avoids:** Pitfall 4 (intel panels as sub-components with `get_untracked()`, explicit trigger signals), Pitfall 9 (recursion limit — extract panels as separate `#[component]` functions to reset view nesting depth).

**Research flag:** Targeted review of `draft.rs` auto-save timer and signal lifecycle before planning Phase 4 tasks. The interaction between a new intel Resource and the existing debounced auto-save Effect is the highest-risk change in the milestone.

---

### Phase 5: Post-Game Loop Automation and Polish

**Rationale:** The post-game → action item connection involves cross-feature write mutations, which require careful resource invalidation design. Placing it last gives the team proven patterns from Phases 1–4 to draw from. Polish (consistent toasts, final empty states) follows complete features.

**Delivers:** Post-game review auto-creates action items from tagged patterns. "View N new action items" navigation link after review save. Consistent confirmation feedback (toast pattern) applied across all mutations app-wide.

**Addresses:** Post-game → action item auto-creation (table stakes), consistent mutation feedback (table stakes).

**Avoids:** Pitfall 3 (navigate-to-action-items with fresh load rather than cross-page signal refetch), Pitfall 8 (designed empty states before implementation begins).

**Research flag:** Standard patterns. Skip research-phase.

---

### Phase Ordering Rationale

- **Layer dependency is strict:** Navigation wiring (Phase 1) populates FK links that aggregation queries (Phase 2) need real data for. Dashboard (Phase 3) depends on Phase 2 server functions. Inline intelligence (Phase 4) depends on Phase 2 queries being proven reliable. Post-game automation (Phase 5) uses patterns established by Phases 1–4.
- **Risk is front-loaded:** Champion name normalization (Pitfall 10) and ID stripping centralization (Pitfall 1, 6) happen in Phase 2 before any intelligence query depends on them.
- **User-visible value ordering:** Phase 1 and 3 deliver the most immediately visible improvements. Phases 2, 4, and 5 are infrastructure or polish — they enable and refine the visible features.

---

### Research Flags

**Needs deeper research before planning:**
- **Phase 4 (Draft intel sidebar):** Adding intelligence panels to `draft.rs` (2,614 lines) with fragile existing signal lifecycle is the highest-risk change in the milestone. Review the auto-save timer pattern (CLAUDE.md rules 54–55) before writing tasks. Confirm `get_untracked()` placement strategy for the intel Resource trigger.

**Standard patterns — skip research-phase:**
- Phase 1: URL params + FK pre-fill — fully documented in Leptos book and CLAUDE.md
- Phase 2: Batch SurrealDB queries + Rust aggregation — established codebase pattern in `get_draft_outcome_stats()`
- Phase 3: Dashboard Resources + Suspense skeletons — established codebase pattern
- Phase 5: Cross-feature mutations + refetch — established codebase pattern (CLAUDE.md rule 23)

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Stack is locked; all patterns verified against codebase and official Leptos/SurrealDB docs. Zero new dependencies confirmed. |
| Features | MEDIUM-HIGH | Market research from official sources (ProComps, Mobalytics, iTero); some competitive feature claims are self-published. Priority order is well-grounded. |
| Architecture | HIGH | Based on direct codebase inspection (db.rs, schema.surql, existing patterns) and verified Leptos 0.8 docs. FK links confirmed in schema. |
| Pitfalls | HIGH | Critical pitfalls verified against schema.surql and open SurrealDB GitHub issues. Moderate pitfalls inferred from code patterns, not observed failures. |

**Overall confidence:** HIGH

### Gaps to Address

- **Champion name normalization scope:** The extent of any existing mismatch in `player_match` records is unknown until queried. May require a one-time data migration script in addition to fixing ingestion. Flag this for the start of Phase 2 investigation.
- **Draft intel sidebar signal interaction:** The exact reactive interaction between a new intel `Resource` and the existing auto-save Effect in `draft.rs` is not fully modelled without running code. This is the primary unknown — address with a brief spike before planning Phase 4 tasks.
- **Suggestion scoring heuristics:** "Ban X because 0% win rate" is clear. "Surface recurring improvement theme from last 3 post-game reviews" requires deciding what constitutes a recurring theme (exact string match, keyword frequency, minimum occurrence count). Plan a brief heuristic design step at the start of Phase 3 before committing to the UI.

---

## Sources

### Primary (HIGH confidence)
- SurrealDB SELECT / aggregation docs — batch query patterns, GROUP BY behaviour
- SurrealDB GitHub issues #4881, #2988, #2825 — aggregate view GROUP BY bugs in 3.x (open)
- Leptos 0.8 book: Resources, Suspense, Params and Queries — `use_query_map()`, `Suspend::new()`, `Resource::new()`, `Memo`
- Codebase: `src/server/db.rs`, `schema.surql`, `src/models/`, `src/pages/` — ground truth for existing patterns and FK links
- Tailwind CSS animation docs — `animate-pulse` availability confirmed
- DraftGap GitHub README — matchup-aware pick recommendation patterns

### Secondary (MEDIUM confidence)
- Mobalytics feature overview (official blog) — pre/post game loop benchmarks
- ProComps.gg (official site + 2024 changelog) — team draft + pool integration patterns
- iTero.gg (official docs) — stats-connected draft tool feature set
- Esportsheaven editorial — analyst workflow for draft tools

### Tertiary (LOW confidence)
- Statup.gg top 10 coaching tools 2025 — third-party review; used for competitive landscape survey only
- iTero self-published comparison article — used only to identify feature categories, not specific claims

---
*Research completed: 2026-03-14*
*Ready for roadmap: yes*
