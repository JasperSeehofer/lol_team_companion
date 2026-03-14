# Domain Pitfalls

**Domain:** Cross-feature intelligence in an existing Leptos 0.8 / SurrealDB 3.x team management app
**Researched:** 2026-03-14
**Scope:** Connecting previously-siloed features (drafts, stats, game plans, post-game reviews) into a
coherent intelligence layer without rewriting or breaking existing functionality.

---

## Critical Pitfalls

Mistakes that cause rewrites, persistent data corruption, or runtime freezes.

---

### Pitfall 1: Schema Cross-Links Stored as Loose Strings Instead of Record References

**What goes wrong:** The `game_plan.draft` field is already typed as `option<string>` (not `record<draft>`)
per `schema.surql` line 83. The `post_game_learning.game_plan_id` and `post_game_learning.draft_id` fields
follow the same pattern. If new cross-feature links (e.g. a suggestion linking a champion stat back to a
draft) are also stored as plain strings, SurrealDB cannot enforce referential integrity and queries that
join across tables will silently return nothing when IDs drift or are incorrectly formatted.

**Why it happens:** In Leptos server functions the ID is serialized to JSON as a String and round-tripped
back; developers copy the existing `option<string>` pattern rather than auditing whether a proper
`record<table>` reference is feasible.

**Consequences:** Cross-feature queries using `type::record('table', $id)` fail silently if the stored
string includes the table prefix (`draft:abc`) because the prefix must be stripped before passing to
`type::record`. A stats-to-draft suggestion that reads `draft_id = "draft:abc123"` and tries to JOIN will
return zero rows.

**Prevention:**
- Decide at schema design time: new linking fields that are SurrealDB-first (never user-supplied strings)
  should use `record<table>` type. Only use `option<string>` for IDs that arrive from external sources
  (Riot match IDs, opponent names).
- When querying by a stored string ID, always apply `type::record('draft', $stripped_id)` and strip the
  prefix in Rust before binding: `id.strip_prefix("draft:").unwrap_or(&id)`.
- Add an integration test for every new cross-table query before merging.

**Detection:** If a cross-feature query returns an empty `Vec` when rows clearly exist, check whether the
stored ID includes the table prefix. Add a `tracing::debug!` log in the DB function printing the raw
bound value.

**Phase:** Schema/DB layer design phase — must be resolved before any intelligence queries are written.

---

### Pitfall 2: N+1 DB Queries When Building the Smart Dashboard

**What goes wrong:** The dashboard needs to surface multiple domains simultaneously: recent drafts, open
action items, unreviewed games, champion pool gaps. The naive approach — one `Resource` per domain, each
calling its own server function, each doing its own `get_user_team_id` lookup — results in 5–8 sequential
round-trips per page load. With SurrealDB embedded this is fast locally but becomes noticeable under any
load, and the existing `db.rs` already does individual queries where batch queries would be more efficient
(see CONCERNS.md performance section).

**Why it happens:** Each existing page was built as an island. The dashboard reuses existing server
functions for expedience but never combines them.

**Consequences:** 5+ network round-trips before the dashboard renders useful content. Each server function
independently calls `get_user_team_id`, which is a separate DB query. On slow connections (or when the
server is under load) this creates a visually broken "loading in pieces" experience even with Suspense.

**Prevention:**
- Write a single `get_dashboard_summary` server function that batches all dashboard queries in one SurrealDB
  `.query()` call using statement chaining (pattern from CLAUDE.md rule 29). Return a typed struct
  `DashboardSummary` containing all sub-resources.
- Call `get_user_team_id` once at the top of the function and reuse the result across all sub-queries.
- Use `Resource::new` once for the summary rather than one resource per section.

**Detection:** Count the number of server function calls logged on dashboard load. More than two (one for
user, one for dashboard) is a smell.

**Phase:** Dashboard implementation phase.

---

### Pitfall 3: Stale Resource Caches After Cross-Feature Mutations

**What goes wrong:** If creating a post-game review auto-generates action items, the action items page
resource will be stale until a hard reload. If linking a draft to a game plan updates the draft's
`game_plan_id` field, the draft list resource on the draft page won't reflect the change. Leptos resources
do not auto-invalidate across page boundaries — `resource.refetch()` only works for resources in the
current component tree.

**Why it happens:** Each page owns its own resources. Cross-feature mutations (post-game → action items,
draft → game plan) happen in server functions but there is no cross-page signal or shared store to trigger
refetch on other pages.

**Consequences:** User creates a post-game review, navigates to action items, sees an empty list. The data
is in the DB but the resource fetched before the mutation finished. User thinks the save failed.

**Prevention:**
- For cross-page side effects, the only reliable mechanism in this architecture (no WebSocket, no shared
  global store) is a hard navigation or a user-triggered refetch. Design UX around this: after a
  post-game review creates action items, show a link "View X new action items" that navigates to the
  action items page (which fetches fresh data on mount).
- Never rely on resource caching across page boundaries for intelligence features.
- Add explicit `resource.refetch()` calls inside `spawn_local` success branches (rule 23) for all
  resources that a mutation might affect within the current page.

**Detection:** After any mutation that has cross-feature side effects, manually navigate to the affected
page and verify data appeared. If it only appears after a hard reload, the refetch is missing.

**Phase:** Any phase that introduces cross-feature writes.

---

### Pitfall 4: Reactive Signal Lifecycle Bugs When Adding Intelligence Panels to Existing Pages

**What goes wrong:** The draft page (`draft.rs`, 2,614 lines) and tree drafter (`tree_drafter.rs`,
1,608 lines) already have fragile signal lifecycle patterns (see CONCERNS.md). Adding an "Intel sidebar"
or "stats context panel" to these pages introduces new signals and resources. If these new signals are
read inside closures that also track existing draft signals, they can inadvertently register reactive
dependencies that cause the panel to re-render on every draft pick, or cause the auto-save Effect to
fire when the intel panel updates.

**Why it happens:** In Leptos 0.8 every `.get()` inside a reactive context (Effect, closure in `view!`,
`Resource` fetcher) registers a dependency. Adding a new `Resource::new(|| selected_champion.get(), ...)`
inside `draft.rs` will cause the intel resource to refetch every time a champion is selected, which can
create a rapid fetch loop if the fetch itself updates state.

**Consequences:** Auto-save fires unexpectedly. Intel panel re-fetches on every user interaction.
Debounced timer from CLAUDE.md rule 42 fires during champion selection and saves a partially-filled draft.

**Prevention:**
- Use `get_untracked()` (not `.get()`) when reading draft state inside intel panel closures that should
  not register as reactive dependencies of the draft (rule 20).
- Keep intelligence resources in separate `Resource::new(|| intel_trigger.get(), ...)` signals that are
  only updated on explicit user actions (e.g. clicking "Analyze this draft"), not on every draft state
  change.
- When adding a new resource to an existing complex page, add a comment explicitly documenting which
  signals it tracks and why.
- Run `cargo check --features hydrate --target wasm32-unknown-unknown` after every change to a large
  existing page — some reactive bugs only surface in the WASM target (rule 37-38).

**Detection:** After adding an intel panel, watch the browser network tab. If server function calls fire
on every champion pick, a `.get()` is in the wrong place.

**Phase:** Any phase adding intelligence panels to draft.rs or tree_drafter.rs.

---

### Pitfall 5: Stats Aggregation Queries That Blow Up db.rs Further

**What goes wrong:** Cross-feature intelligence requires new aggregation queries: "which champions has
this team picked most in ranked games", "which game plans have positive post-game outcomes", "what is the
win rate for drafts tagged with X". Each of these requires joining 2–3 tables. Adding them all to the
already 3,243-line `db.rs` monolith makes the file nearly unmaintainable and increases the chance of
accidentally breaking an unrelated query during a merge.

**Why it happens:** The project constraint is "DB monolith stays" (PROJECT.md). New intelligence queries
will be appended to `db.rs` by default.

**Consequences:** Name collisions on DB struct types (`DbDraft` used by two different queries with
different selected fields). SurrealDB 3.x rejects `ORDER BY` on fields not in a partial `SELECT` (rule
40), so adding aggregation queries with `GROUP BY` and partial selects will fail at runtime if the field
list is wrong. Bugs are hard to debug in a 4,000-line file.

**Prevention:**
- Group new intelligence query functions in a clearly marked section at the bottom of `db.rs` (e.g.
  `// === INTELLIGENCE / AGGREGATION ===`). Do not interleave them with CRUD functions.
- Define new `Db*` structs with unique names for aggregation results (e.g. `DbChampionWinRate`, not
  reusing `DbDraft`). Name collisions cause compile errors only after both features are merged.
- Test every new aggregation query against the real DB in an integration test before wiring it to a
  server function. SurrealDB aggregation behavior (GROUP BY, COUNT, array functions) differs from SQL.
- Always include the ORDER BY field in partial SELECTs (rule 40).

**Detection:** Compile error about conflicting `Db*` struct definitions. Runtime `SurrealDB error`
on a query that seemed correct locally but fails with a partial SELECT ORDER BY mismatch.

**Phase:** Any phase adding stats-to-draft or post-game analytics queries.

---

## Moderate Pitfalls

---

### Pitfall 6: Linking Draft to Game Plan Via Loose String Causes Type Mismatch at Deserialization

**What goes wrong:** `game_plan.draft` is already stored as `option<string>` (using `DEFINE FIELD OVERWRITE`
per schema line 83). This means the draft ID is a plain string in the DB. When the "import from draft"
feature reads `game_plan.draft`, deserializes the draft ID, then calls `get_draft_by_id`, the function
must strip the table prefix before using `type::record('draft', $id)`. If the stored value is `"draft:abc"`
and the code passes `"draft:abc"` directly to `type::record`, it returns no results.

**Prevention:**
- Centralize the strip-and-lookup pattern in a shared `db::get_draft_by_id(id: &str)` function that
  always normalizes the input ID. Never duplicate the stripping logic across server functions.
- Consider migrating `game_plan.draft` to `record<draft>` type if no external callers rely on the string
  format. The schema re-applies on startup but field type changes require manual data migration.

**Phase:** Draft-to-game-plan pipeline phase.

---

### Pitfall 7: Riot API Rate Limit Exceeded by Cross-Feature Smart Suggestions

**What goes wrong:** If the smart dashboard or draft suggestion feature triggers Riot API calls (e.g.
refreshing stats for all 5 team members to compute champion win rates), and 5 teammates all load the
dashboard at the same time, the per-key rate limit is hit. The `riven` crate handles rate limits
internally but blocks until the window resets, potentially causing server function timeouts visible to
users as error banners.

**Why it happens:** The existing stats fetch is user-triggered. Cross-feature intelligence that surfaces
stats automatically (on dashboard load, on draft open) makes API calls implicit and potentially concurrent.

**Consequences:** Dashboard loads stall for 10–30 seconds. Users see "Failed to load suggestions"
errors. Riot API key gets temporarily suspended if the limit is exceeded repeatedly.

**Prevention:**
- Never trigger Riot API calls implicitly on page load for cross-feature features. Only use cached DB
  data (already-imported match records) for dashboard summaries and draft suggestions.
- Riot API calls should remain user-triggered and explicit ("Refresh stats" button).
- If on-load enrichment is needed, implement it as a background job that runs after the page renders
  rather than blocking the initial server function response.

**Phase:** Dashboard and stats-to-draft phases.

---

### Pitfall 8: Empty States vs. Error States Conflated for New Cross-Feature Queries

**What goes wrong:** A cross-feature query like "drafts with win rates" returns an empty Vec when the
team has no linked post-game reviews, zero played games, or no stats imported. If the server function
returns `Err(...)` for any of these conditions (instead of `Ok(Vec::new())`), the UI renders an error
banner rather than a helpful "No data yet — link post-game reviews to see this" empty state.

**Why it happens:** Cross-feature queries have more ways to be "empty" than single-feature queries. A
developer adds a short-circuit `?` on a team ID lookup instead of the `match ... None => return Ok(Vec::new())`
pattern (rule 44).

**Prevention:**
- Apply rule 44 universally for all new intelligence server functions: an absent team, an empty data set,
  or zero linked records are all valid empty states, not errors.
- Each new cross-feature page section should have a designed empty state message before implementation
  begins, so developers know the correct path.

**Detection:** If an intelligence panel shows a red error banner for a new team with no history, it's
returning `Err` where it should return `Ok(empty)`.

**Phase:** All phases, but especially dashboard and "smart suggestions" phase.

---

### Pitfall 9: `recursion_limit` Exceeded When Adding Deep View Nesting to Large Pages

**What goes wrong:** The recursion limit is already at 512 (rule 38), required by `post_game.rs`. Adding
complex intelligence panels — nested `<Suspense>` inside `<Show>` inside conditional `move ||` closures
inside a `<For>` — to already-large pages can push the view type nesting past 512 in the SSR target,
causing a compile error that looks unrelated to the actual change.

**Prevention:**
- Extract intelligence panels into their own `#[component]` functions rather than inlining them in
  existing page components. A new component resets the view nesting depth.
- If a compile error appears mentioning recursion limit exhausted, split the offending view block into
  a sub-component immediately.

**Detection:** Compile error: "reached the recursion limit while instantiating...". Appears in SSR target
only (`cargo check --features ssr`).

**Phase:** Any phase adding panels to draft.rs, dashboard.rs, or post_game.rs.

---

### Pitfall 10: Champion Name Normalization Mismatch Between Data Sources

**What goes wrong:** Champion names flow from three different sources: Riot API match history (uses
`champion` string from `player_match`), champion pool tier lists (user-entered strings), and draft picks
(strings from the champion picker component). A suggestion that says "you played Aurelion Sol 3 times —
add to draft" will fail to match if Riot returns `"AurelionSol"` (no space) while the draft stores
`"Aurelion Sol"` (with space).

**Why it happens:** The champion picker populates from Data Dragon, which uses display names with spaces.
Riot match history uses internal IDs without spaces. The existing stats page notes in the champion
aggregation that this is already a latent mismatch.

**Consequences:** Cross-feature suggestions that join on champion name produce zero matches. A user's
most-played champion does not appear in draft suggestions even though the data is there.

**Prevention:**
- Define a single normalization function (lowercase + strip spaces) and apply it at every ingestion point:
  when storing Riot match data, when storing draft picks, and when querying across tables.
- Do not assume champion name strings are consistent across tables. Any new cross-feature join on champion
  name must go through normalization.

**Detection:** A champion-based cross-feature suggestion returns 0 results when match history clearly
shows that champion was played. Log the raw champion strings from both sides of the join.

**Phase:** Stats-to-draft intelligence phase.

---

## Minor Pitfalls

---

### Pitfall 11: Dashboard Resource Waterfall Due to `<Suspense>` Nesting

**What goes wrong:** If each dashboard section is wrapped in its own `<Suspense>`, sections load
independently but also stall independently. If the action items section inside a nested `<Suspense>`
resolves slowly, the entire dashboard layout shifts after content appears. Conversely, wrapping
everything in one outer `<Suspense>` with a single server function prevents showing partial content
early.

**Prevention:** Use a single `get_dashboard_summary` resource (see Pitfall 2) with one outer `<Suspense>`.
For sections that may be expensive (Riot API stats), break them into a second explicit resource with its
own `<Suspense>` so the fast content renders first.

**Phase:** Dashboard phase.

---

### Pitfall 12: `#[allow(unused_variables)]` Required on WASM-Only Signals in New Intel Pages

**What goes wrong:** Any `RwSignal` used only inside `#[cfg(feature = "hydrate")]` blocks will produce
an unused-variable warning in SSR builds. In a large page this warning can be missed, and if `deny(warnings)`
is ever added to the build, it becomes a compile error.

**Prevention:** Apply `#[allow(unused_variables)]` to every signal that is only consumed in a hydrate
block. This is already documented as rule 43 in CLAUDE.md. Apply it proactively for all new debounce
timers and UI-only state signals in intelligence pages.

**Phase:** All phases.

---

### Pitfall 13: `DEFINE FIELD OVERWRITE` Needed for Schema Changes to Existing Tables

**What goes wrong:** Adding a new cross-feature link field to an existing table (e.g. adding
`opponent_id` as a proper `record<opponent>` field to `draft`) will silently fail to update the
schema on restart if `IF NOT EXISTS` is present and the field already exists with a different type.
The `IF NOT EXISTS` guard is correct for idempotent re-runs, but when changing an existing field's
type or adding a field that conflicts with existing data, `DEFINE FIELD OVERWRITE` is needed (as
already used for `game_plan.draft` on line 83).

**Prevention:** When modifying an existing field definition (not adding a new one), use
`DEFINE FIELD OVERWRITE`. When adding a truly new field, use `IF NOT EXISTS`. Document which lines
in `schema.surql` used `OVERWRITE` and why — this is a migration audit trail given the lack of a
formal migration system.

**Phase:** Any phase that modifies existing table schemas.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Smart dashboard | N+1 queries (Pitfall 2), Riot API rate limit (Pitfall 7) | Single batch server fn; use cached DB stats only |
| Stats-to-draft suggestions | Champion name normalization (Pitfall 10), db.rs growth (Pitfall 5) | Normalize at ingestion; use dedicated aggregation section in db.rs |
| Draft-to-game-plan pipeline | Loose string ID mismatch (Pitfall 1, 6), stale resource on nav (Pitfall 3) | Record reference fields; post-link navigation triggers fresh fetch |
| Post-game → action items loop | Empty state vs error (Pitfall 8), stale action items resource (Pitfall 3) | `Ok(Vec::new())` everywhere; navigate-to-action-items with fresh load |
| Intel panels in draft.rs | Reactive dependency pollution (Pitfall 4), recursion limit (Pitfall 9) | `get_untracked()` in panel closures; extract panel as sub-component |
| Any schema change | `OVERWRITE` vs `IF NOT EXISTS` confusion (Pitfall 13) | Use `OVERWRITE` for field type changes, `IF NOT EXISTS` for new fields only |

---

## Sources

- Codebase analysis: `/home/jasper/Repositories/lol_team_companion/schema.surql` (direct inspection)
- Codebase analysis: `/home/jasper/Repositories/lol_team_companion/src/server/db.rs` (pattern review)
- Codebase analysis: `/home/jasper/Repositories/lol_team_companion/src/pages/game_plan.rs`, `post_game.rs`, `stats.rs`, `action_items.rs` (existing cross-feature patterns)
- `.planning/codebase/CONCERNS.md` (fragile areas, technical debt, performance concerns)
- `.planning/codebase/ARCHITECTURE.md` (data flow, resource/signal patterns)
- `CLAUDE.md` rules 20, 23, 29, 37–44 (Leptos/SurrealDB/WASM patterns specific to this codebase)
- Confidence: HIGH for Leptos/SurrealDB-specific pitfalls (verified against codebase); HIGH for cross-feature data model pitfalls (verified against schema.surql); MEDIUM for N+1 and rate-limit pitfalls (inferred from existing code patterns, not from observed failures)
