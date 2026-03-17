# Phase 03: Smart Dashboard - Research

**Researched:** 2026-03-15
**Domain:** Leptos 0.8 dashboard UI, Rust server functions, SurrealDB (already-built aggregation layer)
**Confidence:** HIGH

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INTL-01 | Smart dashboard surfaces prep priorities (upcoming game context, incomplete workflows, recent action items) | `get_dashboard_summary` in db.rs fully implemented in Phase 2; dashboard page already has partial panel scaffolding; this phase wires them together into a complete, independently-loading, empty-state-aware UI |
</phase_requirements>

---

## Summary

Phase 3 is primarily a **UI wiring phase**. The backend is already complete. Phase 2 delivered `db::get_dashboard_summary` (5-statement batched query returning `DashboardSummary`), `compute_pool_gaps`, and all supporting model types. Phase 1 delivered `db::get_open_action_items_summary` and the `get_recent_team_matches` server function. The current dashboard page (`src/pages/team/dashboard.rs`) already has:

- An "Open Action Items" widget using `get_open_action_items_summary` with its own `Resource` and `Suspense`
- A "Recent Matches" widget with its own `Resource` and `Suspense`
- A `get_team_dashboard` server function that loads team/roster data

What Phase 3 must add: a new `get_smart_dashboard_panels` server function that calls `db::get_dashboard_summary`, and three new dashboard panels wired to independent `Resource`/`Suspense` pairs: (1) Open Action Items (count + links, using `DashboardSummary`), (2) Recent Post-Game Summaries with patterns, (3) Champion Pool Gap Warnings. The existing action items widget may be refactored to use `DashboardSummary` instead of a separate query, or left in place — either approach is fine. Empty states with contextual CTAs are required for all panels for new teams.

The key Leptos constraint for "independent loading" is to ensure each panel creates its own `Resource` and wraps in its own `Suspense`. If all three panels share one `Resource`, a slow pool gap query blocks all of them. Since `get_dashboard_summary` is a single batched call, the alternative is to split into separate server functions — one per panel — so each `Suspense` boundary is truly independent.

**Primary recommendation:** Split the three dashboard panel queries into two separate server functions (`get_action_item_panel` and `get_post_game_panel` can share the existing patterns, `get_pool_gap_panel` is its own function). Each panel in the UI has its own `Resource` and `Suspense`. This gives true independent loading and matches the success criterion exactly.

---

## Standard Stack

### Core (all already in project — no new dependencies)

| Library | Version | Purpose | Role in Phase |
|---------|---------|---------|--------------|
| `leptos` | 0.8 | `Resource`, `Suspense`, `view!` | Dashboard panels, independent loading |
| `surrealdb` | 3.x | Queries | Already done in Phase 2 (`get_dashboard_summary`) |
| `leptos_router` | 0.8 | `<A>` links | CTA links to `/action-items`, `/post-game`, `/champion-pool` |

### No New Dependencies

This phase adds no new crate dependencies.

---

## Architecture Patterns

### Pattern 1: Independent Panel Loading with Separate Resources (HIGH confidence)

Each dashboard panel must load independently. The correct Leptos 0.8 pattern is to give each panel its own `Resource` and wrap it in its own `Suspense`.

```rust
// Source: existing dashboard.rs — action items and recent matches already do this

// Panel A: independent resource
let action_item_panel = Resource::new(|| (), |_| get_action_item_panel());

// Panel B: independent resource
let post_game_panel = Resource::new(|| (), |_| get_post_game_panel());

// Panel C: independent resource
let pool_gap_panel = Resource::new(|| (), |_| get_pool_gap_panel());

// In view!:
<Suspense fallback=|| view! { <PanelSkeleton /> }>
    {move || post_game_panel.get().map(|res| ...)}
</Suspense>
```

This means either three separate server functions (one per panel), or one function returning all data where each panel reads a subset — but the `Resource` must be separate per panel for true Suspense isolation.

**Recommended approach:** Three separate server functions. The pool gap computation (`compute_pool_gaps_for_team`) is the slowest because it calls Data Dragon. Keeping it isolated means slow Data Dragon responses don't delay the action items panel.

### Pattern 2: Empty State with CTA (HIGH confidence)

The success criteria require contextual CTAs for empty states. Pattern from existing codebase:

```rust
// Source: dashboard.rs Ok(None) branch, roster.rs no-team state
if items.is_empty() {
    view! {
        <div class="text-center py-6">
            <p class="text-muted text-sm mb-3">"No post-game reviews yet."</p>
            <A href="/post-game" attr:class="text-accent text-sm hover:underline">
                "Start your first review"
            </A>
        </div>
    }.into_any()
} else {
    view! { /* panel content */ }.into_any()
}
```

### Pattern 3: Server Function — Auth + Team ID Guard (HIGH confidence)

All new server functions follow the established pattern from `get_open_action_items_summary`:

```rust
#[server]
pub async fn get_post_game_panel() -> Result<Vec<PostGamePreview>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&db, &user.id).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),  // Rule 44: empty list, not Err
    };

    // Call the Phase 2 function
    let summary = db::get_dashboard_summary(&db, &team_id).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(summary.recent_post_games)
}
```

### Pattern 4: `into_any()` for Divergent Branches (HIGH confidence)

Per CLAUDE.md rule 19, empty vs. content branches inside `{move || ...}` closures must each call `.into_any()`.

### Pattern 5: `<A>` with `attr:class` (HIGH confidence)

Per CLAUDE.md rule 10, Leptos router `<A>` uses `attr:class` not `class`:

```rust
<A href="/action-items" attr:class="text-accent text-sm hover:underline">"View all"</A>
```

### Recommended New Server Functions

Three new server functions in `dashboard.rs`:

1. **`get_action_item_panel`** — returns `(usize, Vec<ActionItemPreview>)` (total count + top 3). Can reuse `list_open_action_items` or delegate to `get_dashboard_summary`. May replace the existing `get_open_action_items_summary` function.

2. **`get_post_game_panel`** — returns `Vec<PostGamePreview>` (last 5 reviews). Calls `get_dashboard_summary` and plucks `recent_post_games`.

3. **`get_pool_gap_panel`** — returns `Vec<PoolGapWarning>`. Calls `get_dashboard_summary` and plucks `pool_gap_warnings`.

**Alternative:** Call `get_dashboard_summary` from a single new server function and return the entire `DashboardSummary`. Then split into three resources on the client by deriving signals or using three separate `Suspense` that all wait on the same resource but render different parts. This is simpler to implement but means the pool gap query (slowest) blocks all panels from showing their `Suspense` fallback together. The planner should choose: three separate functions (true independent loading) vs. one function with three `Suspense` around the same resource.

**Recommendation for planner:** Three separate server functions for true independent loading per success criterion 4.

### Anti-Patterns to Avoid

- **Nesting `Resource::new` inside a `Suspense` closure:** The existing dashboard has one instance of `Resource::new` created inside a `{move || ...}` closure within `<Suspense>` (the action items panel at line 1081). This is an anti-pattern — a new `Resource` is created on every render cycle. New panels should create Resources at the component top level.
- **Using `.unwrap()` in WASM closures:** CLAUDE.md rule 35. Use `unwrap_or_default()` or `if let`.
- **`class=` on `<A>`:** Must be `attr:class=` per rule 10.
- **Returning `Err` for no-team state:** Rule 44. Return `Ok(empty)`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Action item count + preview | Custom query | `db::list_open_action_items` (already exists) or `db::get_dashboard_summary` (Phase 2) | Already correct, tested |
| Post-game summaries | Custom query | `db::get_dashboard_summary(...).recent_post_games` | Phase 2 already has 5-statement batched query |
| Pool gap warnings | Custom gap detection | `db::get_dashboard_summary(...).pool_gap_warnings` | Phase 2 `compute_pool_gaps` handles all class diversity logic |
| Champion name display | ID-to-name in panels | `champions.iter().find(\|c\| c.id == champ_id).map(\|c\| c.name.clone())` | Established Phase 2/3 pattern |

**Key insight:** Phase 2 did the hard work. Phase 3 is about wiring existing functions into the UI correctly.

---

## Common Pitfalls

### Pitfall 1: Resource Created Inside Suspense Closure (Bug in Existing Code)
**What goes wrong:** The existing action items panel creates `Resource::new` inside the `{move || ...}` closure at line 1081 of `dashboard.rs`. This creates a fresh Resource on every reactive re-render, causing spurious refetches.
**Why it happens:** The panel was written inline inside a deeply nested `Suspense` closure.
**How to avoid:** All new panel Resources are created at component top-level. The existing bug should be fixed as part of Phase 3 (move `action_items_res` to top of `TeamDashboard`).
**Warning signs:** Network DevTools shows repeated requests for the same endpoint on page interaction.

### Pitfall 2: Single Suspense Blocking All Panels
**What goes wrong:** If all three panels share one `Resource` and one `Suspense`, and the pool gap query takes 500ms (Data Dragon network call), none of the panels show any content during that time.
**Why it happens:** Simplifying by using one server function for all panels.
**How to avoid:** Three separate Resources, three separate Suspense boundaries.

### Pitfall 3: Pool Gap Panel Shows Errors for No-Team State
**What goes wrong:** If `get_pool_gap_panel` returns `Err` when user has no team, the Suspense renders the error state instead of an empty CTA.
**Why it happens:** Forgetting rule 44.
**How to avoid:** `None => return Ok(Vec::new())` in the team_id guard.

### Pitfall 4: `DashboardSummary.open_action_item_count` vs. `Vec.len()`
**What goes wrong:** `DashboardSummary.recent_action_items` contains only the top 3, but `open_action_item_count` is the true total (may be > 3). Displaying `recent_action_items.len()` as the count shows "3 open items" when there are really 15.
**Why it happens:** Conflating the preview list with the total count.
**How to avoid:** Always use `DashboardSummary.open_action_item_count` for the count badge, not `recent_action_items.len()`.

### Pitfall 5: `PostGamePreview.improvements` Rendering
**What goes wrong:** `improvements: Vec<String>` is a list of improvement notes. Rendering all of them verbatim makes the panel verbose.
**Why it happens:** Not applying a display limit.
**How to avoid:** Show the first 2–3 improvements with a truncation indicator if more exist.

---

## Code Examples

### Example 1: Panel Server Function Template

```rust
// Source: pattern from get_open_action_items_summary (dashboard.rs ~line 469)
#[server]
pub async fn get_post_game_panel() -> Result<Vec<PostGamePreview>, ServerFnError> {
    use crate::models::game_plan::PostGamePreview;
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&db, &user.id).await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    let summary = db::get_dashboard_summary(&db, &team_id).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(summary.recent_post_games)
}
```

### Example 2: Independent Panel Resources at Component Top Level

```rust
// Source: pattern from dashboard.rs Resources at component top-level (lines 525-527)
// Three resources, each independently suspendable
let action_item_panel = Resource::new(|| (), |_| get_action_item_panel());
let post_game_panel   = Resource::new(|| (), |_| get_post_game_panel());
let pool_gap_panel    = Resource::new(|| (), |_| get_pool_gap_panel());
```

### Example 3: Post-Game Panel with Empty State CTA

```rust
// Pattern: into_any() on each branch (CLAUDE.md rule 19)
<Suspense fallback=|| view! { <p class="text-dimmed text-sm">"Loading..."</p> }>
    {move || post_game_panel.get().map(|res| match res {
        Ok(previews) if previews.is_empty() => view! {
            <div class="text-center py-6">
                <p class="text-muted text-sm mb-3">"No post-game reviews yet."</p>
                <A href="/post-game" attr:class="text-accent text-sm hover:underline">
                    "Start your first review"
                </A>
            </div>
        }.into_any(),
        Ok(previews) => view! {
            <div class="space-y-3">
                {previews.into_iter().map(|p| view! {
                    <div class="bg-elevated border border-divider rounded-lg p-3">
                        <div class="text-xs text-muted mb-1">{p.created_at.unwrap_or_default()}</div>
                        {p.improvements.into_iter().take(2).map(|imp| view! {
                            <p class="text-sm text-secondary truncate">{imp}</p>
                        }).collect_view()}
                    </div>
                }).collect_view()}
                <A href="/post-game" attr:class="text-accent text-sm hover:underline">
                    "View all reviews"
                </A>
            </div>
        }.into_any(),
        Err(_) => view! {
            <p class="text-dimmed text-sm">"Could not load post-game reviews."</p>
        }.into_any(),
    })}
</Suspense>
```

### Example 4: Pool Gap Panel with Warning Display

```rust
// PoolGapWarning has: user_id, username, role, dominant_class, missing_classes, opponent_escalated
{move || pool_gap_panel.get().map(|res| match res {
    Ok(warnings) if warnings.is_empty() => view! {
        <div class="text-center py-6">
            <p class="text-muted text-sm mb-3">"No pool gaps detected."</p>
            <A href="/champion-pool" attr:class="text-accent text-sm hover:underline">
                "Manage champion pools"
            </A>
        </div>
    }.into_any(),
    Ok(warnings) => view! {
        <div class="space-y-2">
            {warnings.into_iter().map(|w| {
                let label = if w.opponent_escalated {
                    format!("{} ({} — opponent threat)", w.username, w.role)
                } else {
                    format!("{} ({})", w.username, w.role)
                };
                view! {
                    <div class="flex items-start gap-2 bg-elevated border border-divider rounded px-3 py-2">
                        <span class="text-yellow-500 shrink-0">"!"</span>
                        <div>
                            <span class="text-sm text-primary">{label}</span>
                            {if !w.missing_classes.is_empty() {
                                view! {
                                    <p class="text-xs text-muted">
                                        "Missing: "{w.missing_classes.join(", ")}
                                    </p>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                        </div>
                    </div>
                }
            }).collect_view()}
            <A href="/champion-pool" attr:class="text-accent text-sm hover:underline">
                "Update pools"
            </A>
        </div>
    }.into_any(),
    Err(_) => view! {
        <p class="text-dimmed text-sm">"Could not load pool gaps."</p>
    }.into_any(),
})}
```

---

## What Already Exists (Do Not Re-Implement)

| Already Done | Where | Notes |
|-------------|-------|-------|
| `db::get_dashboard_summary` | `src/server/db.rs` line 3490 | Returns `DashboardSummary` with all panel data |
| `db::list_open_action_items` | `src/server/db.rs` line 2737 | Returns full `ActionItem` list (open + in_progress) |
| `DashboardSummary`, `ActionItemPreview`, `PostGamePreview`, `PoolGapWarning` | `src/models/game_plan.rs` | All model types ready |
| `get_open_action_items_summary` server fn | `dashboard.rs` line 469 | Returns `(usize, Vec<ActionItem>)` — may be refactored or replaced |
| Action Items widget (partial) | `dashboard.rs` lines 1073–1115 | Has `Suspense` + `Resource` but Resource created inside closure (bug) |
| Recent Matches widget | `dashboard.rs` lines 1021–1067 | Independent Resource + Suspense, working |

---

## State of the Art

| Old Approach | Current Approach | Phase 3 Change |
|--------------|-----------------|----------------|
| No dashboard intelligence | Partial: action items widget + recent matches | Full: 3 independent panels (action items, post-game, pool gaps) with CTAs |
| Action items Resource created inside Suspense (bug) | Still in place | Fix: move Resource to component top level |
| `get_open_action_items_summary` separate query | Exists but duplicates `get_dashboard_summary` | Either delegate to `get_dashboard_summary` or keep separate (planner decides) |

---

## Open Questions

1. **Refactor existing action items widget or add new one?**
   - What we know: `get_open_action_items_summary` returns full `ActionItem` structs; `DashboardSummary.recent_action_items` returns `ActionItemPreview` (id + text only, no `status` or `assigned_to`). The existing widget displays `status` dots and `assigned_to` labels.
   - What's unclear: Whether `ActionItemPreview` has enough data for the desired display.
   - Recommendation: Keep `get_open_action_items_summary` for the action items widget (it has richer data). Add `get_post_game_panel` and `get_pool_gap_panel` as new server functions calling `get_dashboard_summary`. Fix the Resource-inside-closure bug in the existing widget.

2. **`DashboardSummary` single call vs. three separate queries?**
   - What we know: `get_dashboard_summary` batches all 5 sub-queries in one round-trip. Three separate server functions would require 3 separate round-trips, each with its own auth + team_id lookup overhead.
   - What's unclear: Whether the perf difference matters for a desktop app with a local SurrealDB.
   - Recommendation: Use `get_dashboard_summary` for post-game and pool gap panels (the data is already there). Use `list_open_action_items` for action items panel to preserve the `status` + `assigned_to` display. This is 2 DB round-trips total for 3 panels, which is acceptable.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`#[test]`) + Playwright (e2e) |
| Config file | `Cargo.toml` (no separate test config) |
| Quick run command | `cargo test --features ssr --lib` |
| Full suite command | `cargo test --features ssr --lib && npx playwright test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INTL-01 SC-1 | Dashboard shows open action items panel with count and link | e2e | `npx playwright test audit-team.spec.ts` | Partial (audit-team.spec.ts covers dashboard render) |
| INTL-01 SC-2 | Dashboard shows recent post-game summaries | e2e | `npx playwright test audit-team.spec.ts` | ❌ Wave 0 |
| INTL-01 SC-3 | Dashboard shows champion pool gap warnings | e2e | `npx playwright test audit-team.spec.ts` | ❌ Wave 0 |
| INTL-01 SC-4 | Each panel loads independently (Suspense isolation) | e2e (visual) | `npx playwright test audit-team.spec.ts` | ❌ Wave 0 |
| INTL-01 SC-5 | Empty states show CTAs, not blank panels | e2e | `npx playwright test audit-team.spec.ts` | ❌ Wave 0 (new team has no data) |

Note: SC-4 (independent loading) is verifiable structurally via code review (separate Resources/Suspense) and is not easily tested via Playwright without slow/timeout injection. Code review is sufficient for the plan.

Note: SC-5 (empty states) is verified by the e2e auth fixture which creates a brand-new team with no data, making every dashboard load a "new team" state.

### Sampling Rate

- **Per task commit:** `cargo test --features ssr --lib`
- **Per wave merge:** `cargo test --features ssr --lib`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `e2e/tests/audit-team.spec.ts` — add test: "dashboard shows post-game panel (empty CTA)" using `teamPage` fixture
- [ ] `e2e/tests/audit-team.spec.ts` — add test: "dashboard shows pool gap panel (empty CTA)" using `teamPage` fixture
- [ ] `e2e/tests/audit-team.spec.ts` — add test: "dashboard shows action items count panel" using `teamPage` fixture

*(The `teamPage` fixture always starts with a new team with no data — perfect for empty-state testing.)*

---

## Sources

### Primary (HIGH confidence)

- Direct code reading of `src/pages/team/dashboard.rs` — existing panel patterns, `Resource`/`Suspense` structure, existing server functions
- Direct code reading of `src/server/db.rs` lines 3490–3585 — `get_dashboard_summary` implementation
- Direct code reading of `src/models/game_plan.rs` — `DashboardSummary`, `PostGamePreview`, `PoolGapWarning`, `ActionItemPreview` struct definitions
- `CLAUDE.md` rules 10, 19, 20, 25, 35, 44 — critical Leptos and server fn patterns
- Phase 2 plan summaries (02-01, 02-02, 02-03) confirming what was built

### Secondary (MEDIUM confidence)

- Phase 2 RESEARCH.md — architecture decisions that constrain Phase 3 (champion IDs, gap analysis approach, batching)

### Tertiary (LOW confidence)

None — all findings are from direct code inspection.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, all patterns from direct code reading
- Architecture: HIGH — existing patterns in dashboard.rs and db.rs are the template
- Pitfalls: HIGH — pitfall 1 (Resource inside closure) found by direct code inspection at line 1081
- Empty state CTAs: HIGH — established pattern from existing dashboard `Ok(None)` branch

**Research date:** 2026-03-15
**Valid until:** 2026-06-15 (stable stack)
