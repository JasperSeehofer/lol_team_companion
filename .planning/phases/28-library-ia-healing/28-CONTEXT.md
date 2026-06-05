# Phase 28: Library & IA Healing - Context

**Gathered:** 2026-06-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 28 delivers the **Library** (Hall of Records) — a new `/library` route
(`LibraryRoom` page + `LibrarySurfaces` component) that hosts five existing
features as ARIA-tabbed surfaces over a single parallax `AtmosphereRoot`, whose
growth overlays advance by tier from real team data — plus **IA healing** of the
features that previously lived in flat silos. Phase 28 is the **direct sibling of
Phase 27 (War Room)** and reuses its established pattern; it is the second real
consumer of the Phase 26 `RoomSurface` tab component. Scope (ROOM-05, ROOM-07):

- `/library` route + `LibraryRoom` page composing `AtmosphereRoot` over `LibrarySurfaces`.
- **Five surfaces** in a `RoomSurface` tab strip: **Stats / Match Detail / Analytics / Post-Game / Personal Learnings**.
- `get_library_growth_state` server fn (mirrors Phase 27's `get_team_growth_state`).
- Atmosphere axes `records` / `tomes` / `analysis` drive `data-{axis}-tier` overlay opacity.
- Placeholder gradient WebP assets at the room's asset paths (real FLUX art is Phase 30).
- **IA-healing redirects:** the 5 Library feature routes + `/opponents` 302-redirect into their intent room (see D-08..D-12).
- e2e smoke coverage for `/library` + hydration-no-panic canary in **both** demacia and pandemonium themes.

**Out of scope:** War Room flat-route cleanup (`/draft`, `/tree-drafter`,
`/game-plan` stay as Phase 27 thin shells — NOT redirected this phase); QuickNav
(ROOM-08, Phase 29); Throne / team-dashboard / Team Notebook room (ROOM-06, Phase 29);
real art generation (Phase 30); any new feature capability inside the five surfaces.

</domain>

<decisions>
## Implementation Decisions

### Surface roster (the five Library tabs)
- **D-01:** The Library hosts **exactly five surfaces**: **Stats, Match Detail,
  Analytics, Post-Game, Personal Learnings.** ROADMAP SC1's "team learnings"
  resolves to the **Post-Game** surface (team `PostGameLearning` reviews) — it is
  NOT a separate tab. Personal Learnings (the Phase 14 journal) is the surface that
  `/personal-learnings` heals into (SC2).
- **D-02:** **Post-Game and Personal Learnings stay two distinct tabs** backed by
  their existing, separate data models and CRUD flows (`post_game.rs` →
  `PostGameLearning`/`list_reviews`; `personal_learnings.rs` → personal journal).
  Do NOT merge them into one toggled surface — that would fuse two data models for
  no scope reason.
- **D-03:** **Team Notebook is NOT a Library surface.** It stays embedded in the
  team dashboard (`team/dashboard.rs:1526`) and belongs to the Throne room (Phase 29).
  It only contributes to Library *growth* (see D-14), not as a tab.

### Surface extraction strategy (carried from Phase 27 D-01..D-03)
- **D-04:** **Full shared extraction.** Extract each feature's chrome-free content
  into a reusable `*Surface` component (`StatsSurface`, `MatchDetailSurface`,
  `AnalyticsSurface`, `PostGameSurface`, `PersonalLearningsSurface`).
  `LibrarySurfaces` composes all five. "Chrome-free" = no page-level `<h1>`/title
  block, no outer `min-h-screen`/page padding, no viewport-owning wrapper
  (Phase 27 D-03 definition). Note: `OpponentsSurface` already exists from Phase 27
  — Phase 28 does not re-extract it, only redirects `/opponents` (D-09).
- **D-05:** **Keep all panels mounted** (CSS-hidden when inactive, `hidden` attr +
  `role="tabpanel"`), NOT `<Show>` unmount/remount — the `RoomSurface` keep-mounted
  contract from Phase 27 D-06/D-07/D-09 is already built; Library reuses it.
  Accepted consequence: all five surfaces' `Resource`s fetch eagerly on room entry.

### Match Detail surface (the parameterized one)
- **D-06:** The Match Detail tab shows a **recent-match list in-tab**; selecting a
  match expands its full scoreboard inline. The tab is self-contained — it works
  with no match pre-selected (no dead empty-state tab on first open).
- **D-07:** **Stats rows are the canonical "open a match" path.** Clicking a match
  row in the Stats surface switches to the Match Detail tab with that match
  expanded. The Match Detail tab's own list is the secondary/fallback entry point.
  This requires the two keep-mounted surfaces to share match-selection state
  (planner decides the mechanism — a shared `RwSignal<Option<MatchId>>` provided at
  the `LibrarySurfaces` level is the obvious approach).

### IA-healing route strategy (ROOM-07)
- **D-08:** **302 redirect to the intent room** for all healed legacy routes. SC2's
  "resolves rather than 404ing" is satisfied *by* the redirect resolving. One
  canonical home per feature; the standalone flat page is retired.
- **D-09:** **Redirect set (this phase only):** the five Library feature routes +
  `/opponents`:
  - `/stats` → `/library?tab=stats`
  - `/analytics` → `/library?tab=analytics`
  - `/post-game` → `/library?tab=post-game`
  - `/personal-learnings` → `/library?tab=personal`
  - `/match/:id` → Library Match Detail tab **with the `:id` carried through** (D-11)
  - `/opponents` → `/war-room?tab=opponents` (OpponentsSurface already lives there)
- **D-10:** **Do NOT touch** `/draft`, `/tree-drafter`, `/game-plan` — those remain
  Phase 27 thin shells. Redirecting them is War Room IA cleanup, out of Phase 28's
  "opponents + learnings" scope. (If desired later, it's a small follow-up.)
- **D-11:** **Preserve params, heal index.** Param-carrying / sub-routes must not
  break:
  - `/match/:id` redirects into the Library with that specific match selected/
    expanded (id preserved — not dropped to the bare tab).
  - `/personal-learnings/new` (the `NewLearningPage` create form) **must keep
    working** — either left standalone or healed into an in-room "new learning"
    state. The dedicated create flow must not 404 or lose its entry point.
- **D-12:** Tab-deep-link query params (`?tab=...`) reuse `RoomSurface`'s existing
  `?tab=` resolver (Phase 26/27); unknown/missing tab → first surface (Stats).

### Library growth model (new — mirrors Phase 26/27)
- **D-13:** **Three axes mapped to data:**
  - `records` ← **match syncs** (the archive grows as matches accumulate)
  - `tomes` ← **written learnings** (post-game reviews + personal journal entries = the books)
  - `analysis` ← **reviews / analytical depth** (post-game reviews + plan-effectiveness/analytics signal)
  The exact counts/thresholds are defined for the planner here as intent; the
  planner finalizes precise count sources after reading `db.rs`, mirroring how
  Phase 26 derived `TeamGrowthCounts`/`TeamGrowthState`.
- **D-14:** **Team-scoped growth**, mirroring War Room. `get_library_growth_state`
  wraps team counts like `get_team_growth_state`; **no-team → all tier 0, never
  `Err`** (Phase 27 D-pattern, rule #44). Team Notebook entries (D-03) count toward
  growth. Personal-learnings counts fold in within the user's team context.
  (Blended team+personal growth for solo users was considered and deferred — see Deferred.)
- **D-15:** **Environmental art only** (Phase 27 D-10/D-11): growth is communicated
  purely through `AtmosphereRoot` overlays fading in by tier — no tier badges,
  counts, or growth panel. With placeholder gradients this phase, SC4 ("a richer
  team sees more atmospheric detail") is verified via `data-records-tier` /
  `data-tomes-tier` / `data-analysis-tier` attributes + overlay opacity (seed a
  Tier-2 team vs a fresh team and compare), NOT via visible text.

### Claude's Discretion
- Exact deep-link routing mechanism for `/match/:id` → Library (nested route vs
  query param `?tab=match&id=:id`) — planner's call; hard constraint: resolves, no 404, id preserved (D-11).
- Shared match-selection state mechanism between Stats and Match Detail surfaces (D-07).
- Asset path convention: mirror `/img/rooms/war-room/{region}/...` → `/img/rooms/library/{region}/...`; placeholder gradient WebPs.
- Plane/overlay count + parallax depths for the Library atmosphere (mirror `war_room.rs` / `solo_journey.rs` planes + three axis overlays).
- Internal module layout for the five `*Surface` components (keep parallel to the War Room `*Surface` precedent).
- `get_library_growth_state` placement (beside the Library page) and empty/no-team handling (return default → all tier 0, never `Err`).
- Exact `records`/`tomes`/`analysis` count sources + tier thresholds (D-13), reusing the Phase 26 `TeamGrowthCounts` approach.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 27 (War Room) — the template to copy (read FIRST)
- `.planning/phases/27-war-room/27-CONTEXT.md` — the sibling phase; extract-then-compose (D-01..D-03), keep-mounted panels (D-06..D-09), environmental-only growth (D-10/D-11), `get_team_growth_state` wrapper, asset path + atmosphere conventions. Library mirrors all of this.
- `src/pages/rooms/war_room.rs` — `WarRoomRoom` + `WarSurfaces`: region-read-once, `Resource` growth fetch, per-axis `Signal::derive`, plane/overlay composition, ≤200-line room target. **Direct structural template for `LibraryRoom`/`LibrarySurfaces`.**

### Phase 26 foundations contract
- `.planning/phases/26-room-system-foundations/26-CONTEXT.md` — locked room-system decisions; RoomSurface contract, region-read-once rule, axis→data mapping approach.
- `.planning/research/SUMMARY.md` — extract-then-compose pattern, build sequence, axis table.
- `.planning/research/ARCHITECTURE.md` — component contracts, `<Room>Surfaces` composition.
- `.planning/research/PITFALLS.md` — mousemove accumulation, hydration panic, asset budget.

### Worked examples
- `src/pages/rooms/solo_journey.rs` — region-read-once (`:18`), plane/overlay composition, asset path convention, contrast scrim, `z-10` surface layer.
- `src/pages/solo_dashboard.rs` — the `SoloSurfaces` extraction precedent.

### Components to reuse (already built — wire only)
- `src/components/room/room_surface.rs` — `RoomSurface`; keep-mounted CSS-hidden panels + `?tab=` resolver already shipped (Phase 27 D-06/D-09). Reuse as-is.
- `src/components/room/parallax.rs` — `AtmosphereRoot` (axis props, `data-*-tier` emission, mousemove guard) already shipped (Phase 27). Pass `records`/`tomes`/`analysis` tier signals.
- `src/components/room/mod.rs` — re-exports; add `LibraryRoom`/`LibrarySurfaces`.

### Surface sources to extract (D-04)
- `src/pages/stats.rs` — extract `StatsSurface`; Stats rows must drive the Match Detail tab (D-07).
- `src/pages/match_detail.rs` — `MatchDetailPage` (`:543`, `use_params_map` `:563`); extract `MatchDetailSurface` with in-tab recent-match list (D-06).
- `src/pages/analytics.rs` — extract `AnalyticsSurface`.
- `src/pages/post_game.rs` — extract `PostGameSurface` (`PostGamePage` `:356`; `list_reviews` `:14`; note `#![recursion_limit="512"]` dependency, rule #38).
- `src/pages/personal_learnings.rs` — extract `PersonalLearningsSurface` (`PersonalLearningsPage` `:190`); keep `/personal-learnings/new` (`NewLearningPage`) flow working (D-11).

### Growth data (Phase 26/27 — pattern to mirror)
- `src/models/growth.rs` — `TeamGrowthCounts` / `TeamGrowthState` (`from_counts`, tier derivation, `Default`). Add Library axes (`records`/`tomes`/`analysis`) following the War Room tier pattern (D-13).
- `src/server/db.rs` — `get_team_growth_counts` (Phase 26 batched query) as the model for a Library counts query; Team Notebook + match-sync + post-game/personal-learning count sources live here.

### Routing & redirects (D-08..D-12)
- `src/app.rs:140-167` — `<Route>` block; add `/library` → `LibraryRoom`; convert `/stats`, `/analytics`, `/post-game`, `/match/:id`, `/personal-learnings`, `/opponents` to redirects (preserve `/personal-learnings/new` and `/match/:id` param). `/war-room` (`:158`), `/journey` (`:157`) are room precedents.

### Hydration safety (MUST read before writing the room)
- `CLAUDE.md` gotcha #58 — read region/tier ONCE at room top level, never inside reactive primitives.
- `.claude/rules/leptos-patterns.md` #58 — region-read-once rule + correct/wrong examples; #44 (return `Ok(default)` not `Err` for no-team); #38 (`recursion_limit=512` for post_game views).
- `.claude/rules/wasm-patterns.md` #35/#37/#42/#43 — WASM safety, hydrate-guarded timers/listeners, `#[allow(unused_variables)]`.
- `.planning/phases/18.2-pandemonium-hydration-reconciliation-gap-closure-for-ui-18-1/18.2-CONTEXT.md` — root cause of the structural-branching hydration panic the canary guards against.

### e2e
- `e2e/tests/pages.spec.ts` — `AUTHED_PAGES`: add `/library`; **update** entries for routes now redirecting (`/stats`, `/analytics`, `/post-game`, `/personal-learnings`, `/opponents`) so the suite expects the redirect, not a standalone render.
- `e2e/tests/hydration-no-panic.spec.ts` — add `/library` to the canary (both regions, SC5).
- `e2e/tests/fixtures.ts` — auth fixture; reduced-motion CDP automation for a11y.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `AtmosphereRoot` / `Plane` / `GrowthOverlay` (`parallax.rs`) — generalized axis props + mousemove guard already shipped (Phase 27); Library passes `records`/`tomes`/`analysis` tier signals.
- `RoomSurface` (`room_surface.rs`) — ARIA tablist + `?tab=` deep-link + keep-mounted panels already built (Phase 27 D-06/D-09). Library is its second consumer; **no RoomSurface change needed** (unlike Phase 27, which had to add keep-mounted).
- `TeamGrowthState` derivation (`growth.rs`) — Library adds three new axes following the same `from_counts` pattern.
- `OpponentsSurface` — already extracted in Phase 27; Phase 28 only redirects `/opponents` to it (no re-extraction).
- `war_room.rs` — a complete, working sibling room to copy structurally.

### Established Patterns
- Extract-then-compose: `solo_dashboard.rs`→`SoloSurfaces`, then Phase 27's five `*Surface` extractions. Apply identically to the five Library features.
- Region read ONCE at room top → plain `region: String` prop (rule #58); used only in `src`/`href`/`data-*`/child props — never in a reactive closure/branch.
- Server fns: DB via `use_context::<Arc<Surreal<Db>>>()`; return `Ok(default/empty)` not `Err` for no-team (rules #11/#44).
- WASM: hydrate-only signals `#[allow(unused_variables)]`; MQL/mousemove listeners `#[cfg(feature="hydrate")]`-guarded (rules #37/#42/#43).

### Integration Points
- `LibraryRoom` (new, `src/pages/rooms/library.rs`) reads `InitialTheme` once → `region`, fetches `get_library_growth_state` Resource, derives three tier signals (`records`/`tomes`/`analysis`), composes `AtmosphereRoot` + `LibrarySurfaces`. Target ≤200 lines.
- `LibrarySurfaces` (new) holds the `RoomSurface` tab strip + five extracted `*Surface` components, all mounted; provides shared match-selection state for Stats↔Match Detail (D-07).
- Flat routes in `app.rs` become 302 redirects (D-08/D-09), preserving `/match/:id` id and `/personal-learnings/new` (D-11).
- `get_library_growth_state` (new, beside the Library page) wraps a team counts query mirroring `get_team_growth_counts`.

</code_context>

<specifics>
## Specific Ideas

- The user consistently chose the **clean canonical-home** path: 302-redirect legacy
  routes (one home per feature, retire the standalone page) rather than dual-URL thin
  shells — a deliberate step beyond Phase 27's D-02 (which preserved standalone routes).
  Phase 27's War Room routes are explicitly left untouched (D-10); only Library + opponents heal.
- Match Detail is reachable two ways but with **one canonical open-a-match path**:
  Stats rows drive the Match Detail tab (D-07); the tab's own recent-match list is the fallback.
- Growth metaphor is intuitive and locked at the intent level: `records`=the archive
  (match syncs), `tomes`=written knowledge (learnings), `analysis`=analytical depth
  (reviews) — diegetic, environmental-only, no badges (D-15).

</specifics>

<deferred>
## Deferred Ideas

- **Redirect `/draft`, `/tree-drafter`, `/game-plan` into War Room** — completes the
  rooms IA but is War Room cleanup, out of Phase 28 scope (D-10). Small follow-up if wanted.
- **Blended team + personal Library growth** (solo users without a team see the Library
  grow from personal matches/journal) — considered (D-14); deferred as a new growth-scoping
  model. Phase 28 stays team-scoped like War Room.
- **QuickNav command palette + sub-nav suppression on room paths** — ROOM-08, Phase 29.
- **Throne / team-dashboard / Team Notebook room** — ROOM-06, Phase 29 (Team Notebook only
  contributes to Library growth this phase, D-03).
- **Real FLUX environment art** (replaces placeholder gradients at same paths, no code change) — Phase 30.
- **Eager-fetch optimization** for the five mounted surfaces (D-05) — only if it proves a problem.
- **Unified Post-Game + Personal learnings surface** — explicitly rejected for Phase 28 (D-02); could revisit if two tabs feel redundant to beta users.

### Reviewed Todos (not folded)
None — no pending todos matched this phase.

</deferred>

---

*Phase: 28-library-ia-healing*
*Context gathered: 2026-06-06*
