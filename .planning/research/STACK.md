# Technology Stack

**Project:** LoL Team Companion — Cross-Feature Intelligence Milestone
**Researched:** 2026-03-14
**Overall Confidence:** HIGH (stack is fixed; research is about patterns and specific additions within it)

---

## Context

The stack is not being chosen — it is locked: Rust nightly, Leptos 0.8, Axum 0.8, SurrealDB 3.x
(SurrealKV), Tailwind CSS v4. This research answers "what patterns and minor additions does the
cross-feature intelligence milestone require within that stack?"

---

## Core Framework (Existing — No Changes)

| Technology | Version (Cargo.toml) | Purpose | Notes |
|------------|---------------------|---------|-------|
| Leptos | 0.8 | Full-stack reactive UI | SSR + WASM hydration |
| Axum | 0.8 | HTTP server, routing | Serves WASM bundle + API |
| SurrealDB | 3.x (SurrealKV) | Persistent storage | All domain data |
| Tailwind CSS | v4 | Styling | Standalone binary, no npm |
| Rust nightly | current | Compiler | Required by cargo-leptos |

---

## Pattern Decisions for Cross-Feature Intelligence

### 1. Data Aggregation Layer: Dedicated `#[server]` Functions in `db.rs`

**Recommendation:** Add dedicated server functions for cross-feature queries, keeping all
SurrealDB logic in `db.rs`. Do NOT create a separate service layer or module.

**Why:** The codebase already has one large `db.rs` (~3,243 lines) with established patterns
(DbStruct → model conversion, batch multi-statement queries). Introducing a new layer (e.g.,
`src/server/intelligence.rs`) would split the pattern without proportional benefit. The
milestone's scope is intelligence read-queries, not write-path complexity.

**Concretely:** Add `pub async fn get_dashboard_summary(db, team_id) -> DbResult<DashboardSummary>`
and similar aggregation functions. The server function in the page calls db::*, db.rs holds
the SurQL.

**Confidence:** HIGH — matches the existing codebase pattern exactly.

### 2. Cross-Table Aggregation: Multi-Statement Batch Queries in SurrealDB

**Recommendation:** Use chained `.query()` with multiple semicolon-separated statements,
indexed by position. Do NOT use SurrealDB pre-computed table views (aggregate views) for this
milestone.

**Why batch queries:** SurrealDB 3.x supports batching multiple SELECT statements in a single
`.query()` call, returning results indexed by statement order. This is already the codebase
pattern (see `db.rs` batch patterns, CLAUDE.md rule 29). For cross-feature intelligence
(e.g., "fetch recent drafts + their linked post-game learnings + open action items"), batching
3-5 statements in one round-trip is the right tradeoff.

**Why NOT aggregate views:** SurrealDB pre-computed views auto-update when the FROM table
changes, but not when related tables referenced elsewhere in the query change (their trigger
scope is limited to the table in the FROM clause). A dashboard surface combining drafts,
champion pool, stats, and action items spans multiple tables — a pre-computed view cannot
reliably maintain cross-table consistency without per-table views for each aggregate, which
adds schema complexity. Additionally, the SurrealDB performance docs note the database "is not
optimised for large ad-hoc analytical queries in the same way as specialised columnar data
warehouses." Our data volumes (team of 5, hundreds of records, not millions) make batch
SELECT queries fast enough without views.

**Confidence:** HIGH (official docs, existing codebase pattern, verified SurrealDB 3 trigger scope limitation).

**Example pattern (extend existing):**
```rust
let mut r = db
    .query("SELECT * FROM post_game_learning WHERE team = $team ORDER BY created_at DESC LIMIT 5;
            SELECT * FROM action_item WHERE team = $team AND status = 'open' ORDER BY created_at DESC LIMIT 10;
            SELECT champion, count() AS games, math::sum(win) AS wins FROM player_match
              WHERE user IN (SELECT VALUE user FROM team_member WHERE team = $team)
              GROUP BY champion ORDER BY games DESC LIMIT 20;")
    .bind(("team", team_record_id))
    .await?;
let recent_reviews: Vec<DbPostGameLearning> = r.take(0).unwrap_or_default();
let open_items: Vec<DbActionItem> = r.take(1).unwrap_or_default();
let champ_stats: Vec<DbChampionAggregate> = r.take(2).unwrap_or_default();
```

### 3. Reactive UI Layer: `Memo` + `Suspend` for Dashboard Components

**Recommendation:** Use `Resource` (one per logical data group) wrapped in `<Suspense>` with
`animate-pulse` skeleton fallbacks. Use `Memo` for derived computations that combine resolved
resource data client-side (e.g., computing suggestion strings from champion win-rate data).

**Why separate resources per logical group (not one mega-resource):** The dashboard will show
independent panels (recent action items, upcoming prep, stat highlights). One mega-server-fn
that fetches everything ties all panels to the slowest query. Separate resources allow
progressive loading: action items can render while champion stats are still fetching.

**Why `Suspend` for cleaner async:** Leptos 0.8 `Suspend::new(async move { ... })` lets you
await multiple resources in a single block without nested Option-matching. The Leptos book
documents this as providing "massive performance improvements during SSR." Use `Suspend` inside
`<Suspense>` for any view block that needs multiple resolved values.

**Why `Memo` for derived data:** `Memo` caches the result and only re-runs when dependencies
change (unlike a raw derived signal/closure which re-runs on every access). For suggestion
logic that combines 3+ signals into a `Vec<String>` of recommendations, `Memo` prevents
redundant computation on each render cycle.

**Confidence:** HIGH (Leptos 0.8 book verified, existing codebase uses Resource + Suspense pattern).

**Pattern example:**
```rust
// Server fn returns focused data
let dashboard_data = Resource::new(|| (), move |_| get_dashboard_summary());
let action_items = Resource::new(|| (), move |_| get_action_items());

// Derived computation from resolved data
let suggestion_text = Memo::new(move |_| {
    match dashboard_data.get() {
        Some(Ok(data)) => compute_suggestions(&data), // pure fn, no async
        _ => vec![],
    }
});

view! {
    <Suspense fallback=move || view! { <DashboardSkeleton /> }>
        {move || Suspend::new(async move {
            let data = dashboard_data.await;
            view! { <DashboardContent data /> }
        })}
    </Suspense>
}
```

### 4. Suggestion Engine: Rule-Based Computation in Rust (No External Library)

**Recommendation:** Implement suggestions as pure Rust functions that take aggregated data
structs as input and return `Vec<Suggestion>`. No ML library, no external crate.

**Why pure Rust:** The suggestion requirements are rule-based, not learned:
- "Ban X — team has 0% win rate against it" (threshold on a `f64`)
- "Champion pool gap: no AP mid in pool" (set difference between pool tiers and roles)
- "Last 3 post-game reviews all flagged baron timing" (frequency count on a `Vec<String>`)

These are straightforward aggregations over data already fetched for the dashboard. Introducing
a recommendation crate (sbr-rs, etc.) would add ML infrastructure for problems that are
literally just `if rate < 0.4 { suggest_ban(champ) }`. Rust's ownership model and its iterator
combinators (`filter_map`, `fold`, `group_by` via itertools) handle these well without
additional dependencies.

**Confidence:** HIGH (no external library needed; rule-based logic is the correct approach for
these requirements).

**Pattern:**
```rust
// src/server/suggestions.rs  (SSR-only, called from server fns)
pub struct Suggestion {
    pub category: &'static str,  // "ban", "pool_gap", "pattern"
    pub text: String,
    pub priority: u8,            // 1 (high) to 3 (low)
    pub source_ids: Vec<String>, // IDs of records that triggered it
}

pub fn compute_ban_suggestions(
    champ_stats: &[ChampionStatSummary],
    pool: &[ChampionPoolEntry],
) -> Vec<Suggestion> { ... }

pub fn compute_pool_gap_warnings(pool: &[ChampionPoolEntry]) -> Vec<Suggestion> { ... }

pub fn surface_recurring_patterns(reviews: &[PostGameLearning]) -> Vec<Suggestion> { ... }
```

### 5. Loading States: Tailwind `animate-pulse` Skeleton Components

**Recommendation:** Build reusable skeleton components in `src/components/` using Tailwind's
`animate-pulse` utility. No external skeleton crate needed.

**Why:** Tailwind v4's `animate-pulse` is built-in (confirmed available). Skeleton screens in
this project are HTML div structures with `bg-surface/50 animate-pulse rounded` classes. A
handful of skeleton components (`<CardSkeleton>`, `<ListSkeleton>`, `<StatRowSkeleton>`) cover
all dashboard panels. This is pure Leptos + Tailwind — zero new dependencies.

**Confidence:** HIGH (Tailwind docs, existing theme token patterns in codebase).

**Pattern:**
```rust
#[component]
pub fn CardSkeleton() -> impl IntoView {
    view! {
        <div class="bg-surface rounded-lg p-4 animate-pulse space-y-3">
            <div class="h-4 bg-bg-elevated rounded w-3/4"></div>
            <div class="h-3 bg-bg-elevated rounded w-1/2"></div>
        </div>
    }
}
```

### 6. Draft → Game Plan Pipeline: FK Link Already Exists, No New Schema Required

**Recommendation:** `GamePlan` already has `draft_id: Option<String>`. Implement the pipeline
as a UX flow: when saving a draft, offer "Create game plan from this draft" that pre-populates
`our_champions` and `enemy_champions` from the draft's `DraftAction` records.

**Why no schema change:** The link already exists. The gap is UI (no button) and a server fn
that resolves the draft actions and maps them to champion lists. One new server function in
`game_plan.rs` handles this: `create_game_plan_from_draft(draft_id)`.

**Confidence:** HIGH (direct code inspection).

### 7. Post-Game → Action Items Loop: Source Reference Pattern (Existing)

**Recommendation:** `ActionItem` already has `source_review: Option<String>` linking to
`post_game_learning`. The intelligence gap is surfacing these in the dashboard and in the draft
planner with context. No schema change needed.

**Concrete work:** A server fn that fetches open action items grouped by their source (linked
post-game review → linked draft → opponent name chain), returned as a structured type the
dashboard can render with context. This is a batch multi-statement join query in SurrealDB.

**Confidence:** HIGH (direct code inspection).

---

## Supporting Libraries (Existing — Sufficient)

| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| serde / serde_json | 1 | Serialization for server fn args/returns | Sufficient |
| chrono | 0.4 | Date/time for recency scoring in suggestions | Sufficient |
| thiserror | 2 | Error types in db.rs and suggestions module | Sufficient |

**No new dependencies are needed for this milestone.** All cross-feature intelligence is
implementable with the existing crate set. The suggestion engine is rule-based Rust, skeleton
screens are Tailwind utilities, and cross-table data is fetched via SurrealDB batch queries.

---

## What NOT to Use

| Rejected Approach | Why Not |
|-------------------|---------|
| SurrealDB aggregate/pre-computed views | Trigger scope limited to FROM table; can't reliably maintain cross-table aggregates automatically |
| External recommendation library (sbr-rs, etc.) | Overkill for rule-based thresholds; adds ML infrastructure for `if rate < 0.4` logic |
| Separate intelligence service layer / module | Adds architectural complexity; existing db.rs pattern handles it directly |
| Real-time WebSockets / LIVE SELECT | Out of scope per PROJECT.md; async prep workflow doesn't need it |
| DataFusion / query engine | Extreme overkill; data volumes are small (hundreds of records per team) |
| tokio::join! for parallel server fn calls | Server fns are Axum handlers; parallel DB queries should use SurrealDB multi-statement batching, not separate async tasks per query |
| One mega-dashboard server function | Ties all panels to the slowest query; separate focused resources allow progressive rendering |

---

## Additions to Cargo.toml

None required. This milestone adds no new crate dependencies.

---

## Sources

- [SurrealDB SELECT statement docs](https://surrealdb.com/docs/surrealql/statements/select) — GROUP BY, aggregation functions (HIGH confidence)
- [SurrealDB Define Table / Aggregate Views](https://surrealdb.com/learn/fundamentals/schemafull/define-table) — trigger scope limitation confirmed (HIGH confidence)
- [SurrealDB Performance Best Practices](https://surrealdb.com/docs/surrealdb/reference-guide/performance-best-practices) — "not optimised for large ad-hoc analytical queries" (HIGH confidence)
- [Leptos book: Loading Data with Resources](https://book.leptos.dev/async/10_resources.html) — Resource patterns (HIGH confidence)
- [Leptos book: Suspense](https://book.leptos.dev/async/11_suspense.html) — Suspend::new async block pattern (HIGH confidence)
- [Leptos docs.rs](https://docs.rs/leptos/latest/leptos/) — Memo, Resource, Signal confirmed exports (HIGH confidence)
- [Tailwind CSS animation docs](https://tailwindcss.com/docs/animation) — animate-pulse availability (HIGH confidence)
- Codebase inspection: `src/server/db.rs`, `schema.surql`, `src/models/`, `Cargo.toml` — direct source of truth for existing patterns (HIGH confidence)
