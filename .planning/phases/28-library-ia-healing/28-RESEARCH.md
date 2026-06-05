# Phase 28: Library & IA Healing - Research

**Researched:** 2026-06-06
**Domain:** Leptos 0.8 room system — second RoomSurface consumer, five surface extractions, IA-healing redirects, Library growth model
**Confidence:** HIGH (all findings verified directly from codebase)

---

## Summary

Phase 28 is the closest thing to a "copy + adapt" phase the roadmap has. Every
mechanical pattern it needs — `AtmosphereRoot` with named axis tiers, `RoomSurface`
tab strip, keep-mounted CSS-hidden panels, region-read-once, `TeamGrowthState`
derivation, `get_team_growth_state` server fn shape — was proved and committed in
Phase 27. The Library is the **second consumer** of all those components; no
component changes are required.

The five source files (`stats.rs`, `match_detail.rs`, `analytics.rs`,
`post_game.rs`, `personal_learnings.rs`) are all standard Leptos page components.
Each carries a page-level `auth_user` Resource + Effect redirect, an outer
`min-h-screen` wrapper, a `SectionHead` title block, and a `region` read from
`InitialTheme`. All of that is "chrome" that gets stripped during extraction; the
data Resources and CRUD forms are the content.

The one genuinely new design problem is the **match-selection bridge** (D-07):
Stats currently links to `/match/:id` via a plain `<a href>`. Inside the Library
that link becomes obsolete — clicking a Stats row should switch the Match Detail
tab with the match pre-selected. This requires a shared `RwSignal<Option<String>>`
(match id is `String` — the Riot `riot_match_id`, e.g. `EUW1_1234`) provided at
the `LibrarySurfaces` level and consumed by both `StatsSurface` (to emit) and
`MatchDetailSurface` (to react).

**Primary recommendation:** Copy `war_room.rs` as the structural template for
`library.rs`; copy the Phase 27 `*Surface` extraction pattern for all five
surfaces; wire `AtmosphereRoot` with the three new Library axis props
(`records_tier`, `tomes_tier`, `analysis_tier`) — all already declared in
`parallax.rs`; add `LibraryGrowthCounts` + `LibraryGrowthState` to `growth.rs`
following the `TeamGrowthCounts`/`TeamGrowthState` template.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Surface roster (D-01..D-03):** Exactly five surfaces — Stats, Match Detail,
Analytics, Post-Game, Personal Learnings. Post-Game = team `PostGameLearning`
reviews. Personal Learnings = Phase 14 journal. Team Notebook is NOT a surface.

**Extraction strategy (D-04):** Full shared extraction into `StatsSurface`,
`MatchDetailSurface`, `AnalyticsSurface`, `PostGameSurface`,
`PersonalLearningsSurface`. Chrome-free = no page-level `<h1>`/title block, no
outer `min-h-screen`, no viewport-owning wrapper.

**Keep-mounted (D-05):** All five panels always mounted, CSS-hidden when inactive.
No `<Show>` unmount. All five Resources fetch eagerly on room entry.

**Match Detail (D-06/D-07):** Tab shows recent-match list in-tab; self-contained
(no dead empty-state). Stats rows are the canonical "open a match" path — they
switch to the Match Detail tab with that match expanded. Shared state mechanism
is planner's call.

**IA-healing redirects (D-08..D-12):** 302 redirects only (one canonical home per
feature). Exact redirect set:
- `/stats` → `/library?tab=stats`
- `/analytics` → `/library?tab=analytics`
- `/post-game` → `/library?tab=post-game`
- `/personal-learnings` → `/library?tab=personal`
- `/match/:id` → Library Match Detail tab with `:id` carried through
- `/opponents` → `/war-room?tab=opponents`

Do NOT touch `/draft`, `/tree-drafter`, `/game-plan` (D-10). `/personal-learnings/new`
MUST keep working (D-11). Param-carrying routes must not drop params.

**Growth model (D-13..D-15):** Three axes: `records`=match syncs, `tomes`=written
learnings, `analysis`=reviews/analytical depth. Team-scoped; no-team → all tier 0,
never `Err`. Environmental art only — no tier badges. SC4 verified via
`data-records-tier` / `data-tomes-tier` / `data-analysis-tier` attribute values.

### Claude's Discretion

- Exact deep-link routing mechanism for `/match/:id` → Library (nested route vs
  query param `?tab=match&id=:id`).
- Shared match-selection state mechanism between Stats and Match Detail surfaces.
- Asset path convention: `/img/rooms/library/{region}/...`; placeholder gradient WebPs.
- Plane/overlay count + parallax depths for Library atmosphere.
- Internal module layout for the five `*Surface` components.
- `get_library_growth_state` placement and empty/no-team handling.
- Exact `records`/`tomes`/`analysis` count sources and tier thresholds.

### Deferred Ideas (OUT OF SCOPE)

- Redirect `/draft`, `/tree-drafter`, `/game-plan` into War Room (D-10).
- Blended team + personal Library growth for solo users (D-14).
- QuickNav command palette (ROOM-08, Phase 29).
- Throne / team-dashboard / Team Notebook room (ROOM-06, Phase 29).
- Real FLUX environment art (Phase 30).
- Eager-fetch optimization (D-05) — only if it proves a problem.
- Unified Post-Game + Personal Learnings surface — rejected (D-02).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ROOM-05 | Hall of Records / Library hosts stats, match detail, analytics, post-game, and team learnings as surfaces | Five extraction targets identified; `RoomSurface` API confirmed; `AtmosphereRoot` Library axis props (`records_tier`, `tomes_tier`, `analysis_tier`) already declared and wired in `parallax.rs:71-73` |
| ROOM-07 | IA healing — previously-siloed features are reachable in their intent room; legacy flat routes still resolve as deep links | `leptos_axum::redirect` mechanism confirmed; six redirect targets identified with exact tab slugs; `/personal-learnings/new` standalone route preservation documented |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Library room shell + parallax | Frontend Server (SSR) | Browser (mousemove RAF) | `AtmosphereRoot` renders static HTML on SSR; WASM attaches `mousemove` listener at hydration |
| Growth axis tiers | API / Backend (`#[server]`) | Frontend (Signal::derive) | `get_library_growth_state` fetches counts; tier signals are client-side `Signal::derive` wraps |
| Tab routing (`?tab=`) | Browser / Client | — | `RoomSurface` uses `use_query_map` + `use_navigate`; no SSR routing involved |
| Surface data (stats, reviews, etc.) | API / Backend (`#[server]`) | — | Each surface's existing server fn unchanged; fetched eagerly on mount |
| Match-selection bridge | Browser / Client | — | `RwSignal<Option<String>>` provided at `LibrarySurfaces` level; pure client state |
| IA-healing 302 redirects | API / Backend (server fn) | — | `leptos_axum::redirect()` inside a `#[server]` fn called from a page component; fires on SSR render |
| `/match/:id` param preservation | Browser / Client | — | `use_params_map` reads `:id`; deep-link carries it as `?tab=match&id=:id` or similar |

---

## Standard Stack

No new external packages in this phase. All required components are already in the codebase.

### Core (already shipped — wire only)

| Component | Location | Purpose | Confirmed |
|-----------|----------|---------|-----------|
| `AtmosphereRoot` | `src/components/room/parallax.rs:54` | Parallax root with axis tier `data-*` attrs | Axis props `records_tier`/`tomes_tier`/`analysis_tier` declared at lines 71-73 [VERIFIED: codebase] |
| `RoomSurface` | `src/components/room/room_surface.rs` | ARIA tablist + `?tab=` deep-link + keep-mounted panels | API: `panels: Vec<(SurfaceTab, PanelFn)>`; no changes needed [VERIFIED: codebase] |
| `GrowthOverlay` | `src/components/room/parallax.rs` | Axis-named overlay that fades in by tier via CSS | Used with `axis="records"` / `axis="tomes"` / `axis="analysis"` [VERIFIED: codebase] |
| `Plane` | `src/components/room/parallax.rs` | Single parallax depth plane | Same API as War Room [VERIFIED: codebase] |
| `TeamGrowthCounts` / `TeamGrowthState` | `src/models/growth.rs:109-155` | Proven team-scoped growth pattern | Template for Library's parallel types [VERIFIED: codebase] |
| `tier_for` | `src/models/growth.rs:66` | Pure count→tier mapper, reusable | Private fn — Library uses the same `from_counts` pattern [VERIFIED: codebase] |
| `leptos_axum::redirect` | `src/pages/profile.rs:28`, `src/pages/team/roster.rs:9` | Server-side 302 redirect | Existing usage confirms the mechanism [VERIFIED: codebase] |

### New Types Required

| Type | Location | Pattern |
|------|----------|---------|
| `LibraryGrowthCounts` | `src/models/growth.rs` | Mirrors `TeamGrowthCounts`; fields: `match_syncs: u32`, `post_game_learning: u32`, `personal_learning: u32`, `team_note: u32` |
| `LibraryGrowthState` | `src/models/growth.rs` | Mirrors `TeamGrowthState`; fields: `records_tier: u8`, `tomes_tier: u8`, `analysis_tier: u8` |
| `LibraryRoom` | `src/pages/rooms/library.rs` | Mirrors `WarRoomRoom` |
| `LibrarySurfaces` | `src/pages/rooms/library.rs` | Mirrors `WarSurfaces` + shared match-selection signal |

---

## Package Legitimacy Audit

No external packages are installed in this phase. All dependencies are already in `Cargo.toml`.

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
Browser request: GET /library?tab=stats
        │
        ▼
  LibraryRoom (SSR renders static HTML)
        │  reads InitialTheme ONCE → region: String
        │  fetches get_library_growth_state → LibraryGrowthState
        │
        ├─► AtmosphereRoot (data-records-tier, data-tomes-tier, data-analysis-tier)
        │       ├─ Plane (sky, mid, near)
        │       ├─ GrowthOverlay (records, tomes, analysis)
        │       └─ contrast scrim
        │
        └─► LibrarySurfaces (z-10, region prop)
                │  provides: RwSignal<Option<String>> (selected_match_id)
                │
                └─► RoomSurface (panels=[Stats, MatchDetail, Analytics, PostGame, PersonalLearnings])
                        │  reads ?tab= → active_idx
                        │  provides: Signal<&'static str> (active_id) via context
                        │
                        ├─ StatsSurface      (hidden when tab != "stats")
                        │      │ on "Open match detail" click →
                        │      │   1. set selected_match_id.set(Some(riot_match_id))
                        │      │   2. navigate("?tab=match")
                        │
                        ├─ MatchDetailSurface (hidden when tab != "match")
                        │      reads selected_match_id to auto-expand
                        │      shows recent-match list when None
                        │
                        ├─ AnalyticsSurface  (hidden when tab != "analytics")
                        ├─ PostGameSurface   (hidden when tab != "post-game")
                        └─ PersonalLearningsSurface (hidden when tab != "personal")

Legacy routes in app.rs:
  /stats              → server fn wrapper → leptos_axum::redirect("/library?tab=stats")
  /analytics          → server fn wrapper → leptos_axum::redirect("/library?tab=analytics")
  /post-game          → server fn wrapper → leptos_axum::redirect("/library?tab=post-game")
  /personal-learnings → server fn wrapper → leptos_axum::redirect("/library?tab=personal")
  /match/:id          → server fn wrapper → leptos_axum::redirect("/library?tab=match&id={id}")
  /opponents          → server fn wrapper → leptos_axum::redirect("/war-room?tab=opponents")
  /personal-learnings/new  → UNCHANGED (must stay functional, D-11)
```

### Recommended Project Structure

```
src/
├── pages/rooms/
│   ├── mod.rs          # add: pub mod library;
│   ├── library.rs      # LibraryRoom + LibrarySurfaces + get_library_growth_state server fn
│   ├── war_room.rs     # unchanged
│   └── solo_journey.rs # unchanged
├── models/
│   └── growth.rs       # add LibraryGrowthCounts + LibraryGrowthState
└── app.rs              # add /library route; convert 6 flat routes to redirect shells
```

Surface extraction: each `*Surface` lives in its originating page file (same pattern as War Room — e.g., `DraftSurface` lives in `src/pages/draft.rs`). Do NOT move them to a new file.

### Pattern 1: Library Room Structure (copy of War Room)

```rust
// Source: src/pages/rooms/war_room.rs (direct template)
#[server]
pub async fn get_library_growth_state(
) -> Result<crate::models::growth::LibraryGrowthState, ServerFnError> {
    // ... same shape as get_team_growth_state ...
    // No team → LibraryGrowthState::default() (rule #44)
    let counts = db::get_library_growth_counts(&surreal, &team_id)
        .await
        .unwrap_or_default();
    Ok(LibraryGrowthState::from_counts(counts))
}

#[component]
pub fn LibraryRoom() -> impl IntoView {
    // Region read ONCE — never inside primitives (rule #58)
    let region = use_context::<InitialTheme>().unwrap_or_default().0;
    let growth = Resource::new(|| (), |_| get_library_growth_state());

    let records_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.records_tier).unwrap_or(0)
    });
    let tomes_tier = Signal::derive(move || { /* ... */ });
    let analysis_tier = Signal::derive(move || { /* ... */ });

    let dir = format!("/img/rooms/library/{region}");
    let surfaces_region = region.clone();

    view! {
        <AtmosphereRoot
            class="min-h-screen".to_string()
            records_tier=records_tier
            tomes_tier=tomes_tier
            analysis_tier=analysis_tier
        >
            <Plane src=format!("{dir}/plane-sky.webp") depth=6 z=1 />
            <Plane src=format!("{dir}/plane-mid.webp") depth=16 z=2 />
            <Plane src=format!("{dir}/plane-near.webp") depth=32 z=3 />
            <GrowthOverlay axis="records" src=format!("{dir}/overlay-records.webp") z=4 />
            <GrowthOverlay axis="tomes" src=format!("{dir}/overlay-tomes.webp") z=5 />
            <GrowthOverlay axis="analysis" src=format!("{dir}/overlay-analysis.webp") z=6 />
            <div class="fixed inset-0 pointer-events-none"
                style="z-index:8; background:radial-gradient(ellipse at center,transparent 0%,var(--color-base) 95%);"
                aria-hidden="true" />
            <div class="relative z-10 px-8 py-6" data-feedback-label="Library">
                <LibrarySurfaces region=surfaces_region />
            </div>
        </AtmosphereRoot>
    }
}
```

[VERIFIED: codebase — direct structural copy of `src/pages/rooms/war_room.rs:57-125`]

### Pattern 2: IA-Healing Redirect Shell

The existing codebase uses `leptos_axum::redirect()` inside `#[server]` functions for
programmatic redirects (confirmed: `profile.rs:28-34`, `roster.rs:9-23`). For stateless
page-level redirects (no user input required), the pattern is a thin page component that
immediately triggers a server fn:

```rust
// Source: src/pages/profile.rs:28-34 and src/pages/team/roster.rs:9-23 (redirect pattern)
// Redirect shell for /stats → /library?tab=stats
#[server]
pub async fn redirect_to_library_stats() -> Result<(), ServerFnError> {
    leptos_axum::redirect("/library?tab=stats");
    Ok(())
}

#[component]
pub fn StatsRedirect() -> impl IntoView {
    // Fire the redirect action immediately on mount (SSR renders 302).
    let redirect_action = Action::new(|_: &()| redirect_to_library_stats());
    redirect_action.dispatch(());
    view! { <></> }
}
```

**Important caveat:** `leptos_axum::redirect` fires on the SSR render pass, setting a
302 response header. For the client-side WASM path (when navigating client-side after
initial load), the redirect also fires but the response headers are not observed —
`use_navigate` or `window.location().set_href()` must complement it. The existing
codebase handles this via client-side `window.location().set_href()` in `Effect::new`
(see `home.rs:196`, `auth` redirects). The planner should use the same dual approach:
server fn `redirect()` + `#[cfg(feature = "hydrate")]` `window.location().set_href()`.

[VERIFIED: codebase — `src/pages/profile.rs:28-34`, `src/pages/team/roster.rs:9-23`]

### Pattern 3: `/match/:id` Param Preservation (D-11)

The match id in the current route is `params.read().get("id")` returning a `String`
(the Riot match id, e.g. `"EUW1_6940123456"`). The `fetch_match_detail` server fn
takes `match_id: String`.

For the redirect shell at `/match/:id`, the id must be read and appended to the
Library URL. The recommended approach (Claude's Discretion, D-11):

```
/match/{id} → /library?tab=match&id={id}
```

`MatchDetailSurface` reads `?id=` from `use_query_map()` to pre-select a match. The
tab deep-link `?tab=match` is the RoomSurface resolver; `?id=` is a second param
`MatchDetailSurface` reads independently.

The redirect shell reads `use_params_map()` for `:id` then navigates:

```rust
// In the /match/:id redirect component (client-side path):
let params = use_params_map();
let id = params.read().get("id").unwrap_or_default();
// Server-side: leptos_axum::redirect(&format!("/library?tab=match&id={id}"))
// Client-side: window.location().set_href(...)
```

[VERIFIED: codebase — `src/pages/match_detail.rs:563-565`]

### Pattern 4: Shared Match-Selection Signal

`LibrarySurfaces` provides a `RwSignal<Option<String>>` to descendants. Stats surface
uses it instead of the current `<a href="/match/{id}">` link. Match Detail surface
derives its active match id from this signal (falling back to `use_query_map` `?id=`
for the initial deep-link case).

```rust
// In LibrarySurfaces
let selected_match_id: RwSignal<Option<String>> = RwSignal::new(None);
provide_context(selected_match_id);

// In StatsSurface — replace the <a href> CTA:
let selected_match_id = use_context::<RwSignal<Option<String>>>().expect("...");
let navigate = use_navigate();
// on "Open match detail" click:
on:click=move |_| {
    selected_match_id.set(Some(riot_id.clone()));
    navigate("?tab=match", Default::default());
}

// In MatchDetailSurface — derive active id:
let selected_match_id = use_context::<RwSignal<Option<String>>>().expect("...");
let query = use_query_map();
let match_id = Signal::derive(move || {
    selected_match_id.get()
        .or_else(|| query.read().get("id"))
        .unwrap_or_default()
});
let detail = Resource::new(match_id, |id| fetch_match_detail(id));
```

Note: `selected_match_id` is WASM-only state — on SSR it will always be `None`,
which is correct (the `?id=` query param handles the SSR deep-link path).

[VERIFIED: codebase — pattern follows `use_context` / `provide_context` used throughout the codebase]

### Pattern 5: Chrome Extraction Recipe

Strip exactly these elements from each source page to create `*Surface`:

| Strip | Keep |
|-------|------|
| `let auth_user = Resource::new(...); Effect::new(...)` auth guard | All data Resources and CRUD signals |
| `let theme = use_context::<InitialTheme>()...; let region = theme.0.clone();` | Pass `region` as prop if needed downstream (note: none of the five sources use region in their content, only in `SectionHead`/`Card` wrappers) |
| `view! { <div class="canvas-grain bg-base min-h-screen">` outer wrapper | Inner content `<div>` children |
| `<SectionHead ... title=... />` page title block | All data display, forms, tables |
| `<div class="max-w-Xxxxxl mx-auto px-Y py-Z">` page padding wrapper | Leave inner padding if needed for readability |

Post-Game-specific: the file-level `#![recursion_limit = "512"]` in `lib.rs` and
`main.rs` is ALREADY present (rule #38). The `PostGameSurface` extraction does not
require any new recursion_limit change — it's already set project-wide.

[VERIFIED: codebase — confirmed `#![recursion_limit = "512"]` pattern in CLAUDE.md rule #38 and leptos-patterns.md rule #38]

### Anti-Patterns to Avoid

- **Reading context inside reactive primitives (the Phase 18.2 panic):** `use_context::<InitialTheme>()` must only appear at the top of `LibraryRoom`. Never inside `Signal::derive`, `Effect`, `move || {}`, or a child component.
- **Using `<Show>` for surface visibility:** `RoomSurface` uses `attr:hidden` — panels never unmount. Do not add `<Show>` wrappers around `*Surface` components inside `WarSurfaces`/`LibrarySurfaces`.
- **Forgetting `#[cfg(feature = "hydrate")]` on client-side redirect:** The `window.location().set_href()` part of redirect shells MUST be gated. Calling it on SSR causes a compile error / linker warning.
- **Providing context below `LibrarySurfaces` but above `RoomSurface`:** The `selected_match_id` signal must be provided BEFORE `RoomSurface` so both `StatsSurface` and `MatchDetailSurface` panels (rendered inside `RoomSurface`) can consume it.
- **Dropping `/personal-learnings/new` route:** D-11 mandates it stays. Only `/personal-learnings` (the browse page) redirects; `/personal-learnings/new` (`NewLearningPage`) keeps its existing route in `app.rs`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tab strip + `?tab=` deep-link | Custom tab component | `RoomSurface` (already built Phase 26/27) | Keep-mounted panels, ARIA, unknown→first fallback all implemented |
| Growth tier derivation | Custom tier logic | `tier_for()` in `growth.rs:66` + `LibraryGrowthState::from_counts` | Pure function, already unit-tested |
| Parallax mouse bridge | Custom mousemove handler | `AtmosphereRoot` (already built) | Single-listener slot, RAF batching, reduced-motion guard — correct implementation is non-trivial |
| Server-side 302 redirect | Axum middleware or custom handler | `leptos_axum::redirect()` inside `#[server]` fn | Already the project pattern; compatible with axum-login session layer |
| Count queries (batch) | Separate DB round-trips per table | Batch `count() GROUP ALL` in one `.query()` call | Proven pattern in `get_team_growth_counts` — 6 counts, 1 round-trip |

**Key insight:** The Library's growth model is a pure data transformation (counts → tiers). The tier thresholds are hypotheses calibrated for beta; they live in `LibraryGrowthState::from_counts` and can be tuned without touching the room UI.

---

## Growth Count Sources (D-13 Detail)

### Table Availability

The `match` table has `team_id: option<record<team>>` (schema.surql:49). This is the canonical team-scoped match record. [VERIFIED: codebase — `schema.surql:44-52`]

```
records axis  ← count(*) FROM match WHERE team_id = type::record('team', $team_key)
tomes axis    ← count(*) FROM post_game_learning WHERE team = type::record('team', $team_key)
              + count(*) FROM personal_learning WHERE user IN [team member user ids]  [ASSUMED — see notes]
              + count(*) FROM team_note WHERE team = type::record('team', $team_key)
analysis axis ← count(*) FROM post_game_learning WHERE team = type::record('team', $team_key)
              (same table as tomes — post-game reviews are both "written knowledge" AND "analytical depth")
```

**Simplification recommendation (Claude's Discretion):** For Phase 28, `tomes` can
be `post_game_learning + team_note` (team-scoped only, matching D-14's team-scoped
decision). Personal learnings require a sub-query joining through `team_member` to
get user IDs — that adds complexity and the tomes axis still advances meaningfully
without it. Defer personal-learning counting into tomes to a follow-up (the deferred
"blended growth" idea in CONTEXT.md).

`analysis` can map solely to `post_game_learning` count — it's the direct proxy for
analytical depth.

### Proposed LibraryGrowthCounts

```rust
// src/models/growth.rs — add after TeamGrowthCounts
pub struct LibraryGrowthCounts {
    pub match_syncs: u32,       // match table, team_id-scoped
    pub post_game_learning: u32, // post_game_learning table, team-scoped
    pub team_note: u32,          // team_note table, team-scoped
}
```

### Proposed Batch Query (mirrors get_team_growth_counts)

```rust
// src/server/db.rs — add get_library_growth_counts
db.query(
    "SELECT count() as n FROM match WHERE team_id = type::record('team', $team_key) GROUP ALL; \
     SELECT count() as n FROM post_game_learning WHERE team = type::record('team', $team_key) GROUP ALL; \
     SELECT count() as n FROM team_note WHERE team = type::record('team', $team_key) GROUP ALL",
)
.bind(("team_key", team_key))
.await?
```

[VERIFIED: codebase — `src/server/db.rs:4164-4207` for pattern; `schema.surql:44-52` for `match.team_id` field]

### Proposed Tier Derivation + Thresholds

Following the War Room calibration philosophy ("thresholds are hypotheses for beta tuning"):

```rust
impl LibraryGrowthState {
    pub fn from_counts(counts: LibraryGrowthCounts) -> Self {
        // records: grows as the team syncs matches
        // 1 match = first session, 10 = regular practice, 30 = established archive
        let records = tier_for(counts.match_syncs, [1, 10, 30]);

        // tomes: post-game reviews + team notes (written knowledge)
        // 1 = started writing, 5 = consistent reviewer, 15 = rich knowledge base
        let tomes = tier_for(counts.post_game_learning + counts.team_note, [1, 5, 15]);

        // analysis: post-game review depth specifically
        // 1 = first review, 5 = analytical habit, 15 = deep review culture
        let analysis = tier_for(counts.post_game_learning, [1, 5, 15]);

        LibraryGrowthState {
            counts,
            records_tier: records.level(),
            tomes_tier: tomes.level(),
            analysis_tier: analysis.level(),
        }
    }
}
```

[ASSUMED — threshold values; follow War Room D-09 calibration hypothesis pattern]

---

## Runtime State Inventory

No rename/refactor involved. Skipped.

---

## Common Pitfalls

### Pitfall 1: Region Read Inside LibrarySurfaces (the Phase 18.2 panic)
**What goes wrong:** Passing `region` as a prop to `LibraryRoom` children is correct, but calling `use_context::<InitialTheme>()` inside `LibrarySurfaces` or any `*Surface` causes the tachys reconciler to see different SSR/hydrate DOM structure when the user's region is non-default ("pandemonium"). It panics at `html/mod.rs:217` on `Option::unwrap()` and silently kills all WASM event handlers.
**Why it happens:** SSR renders the room once with the resolved region; WASM hydration attempts to reconcile the same DOM but the reactive context read introduces a structural branch that doesn't exist in the SSR pass.
**How to avoid:** Read `InitialTheme` exactly once at `LibraryRoom` top level. Pass `region: String` prop. Children use the prop — never call `use_context::<InitialTheme>()` again anywhere in the module.
**Warning signs:** e2e hydration-no-panic spec fails for "pandemonium" but passes for "demacia". Console shows `tachys` or `Option::unwrap() on None` pageerror.

### Pitfall 2: Stats Surface CTA Still Navigates to `/match/:id`
**What goes wrong:** After extraction, `StatsSurface` still has `<a href="/match/{riot_id}">` at line ~1004. This now points to the redirect shell, causing a full page navigation instead of an in-tab tab switch.
**Why it happens:** The extract step removes page chrome but leaves content logic untouched. The `detail_href` link at `stats.rs:933` must be replaced.
**How to avoid:** During extraction of `StatsSurface`, replace the `<a href=detail_href>` CTA with the shared match-selection signal + `navigate("?tab=match")` dispatch.
**Warning signs:** Clicking "Open match detail" from the Stats tab triggers a full navigation away from the Library room.

### Pitfall 3: `/personal-learnings` Redirects but `/personal-learnings/new` Breaks
**What goes wrong:** Converting `PersonalLearningsPage` route to a redirect shell in `app.rs` without verifying the `/personal-learnings/new` route is separate. If the router matches `/personal-learnings` as a prefix of `/personal-learnings/new` (unlikely with Leptos exact matching but possible with ordering), the new form redirects instead of loading.
**Why it happens:** Route ordering in `app.rs:139-167` — longer/more-specific routes should be declared before shorter ones, or at minimum verified to not conflict.
**How to avoid:** Keep `<Route path=path!("/personal-learnings/new") view=NewLearningPage />` as-is at its current position. Only add the redirect for the exact path `/personal-learnings` (not a prefix match). Verify with e2e test: `/personal-learnings/new` content should match `new|learning|happened|learned`.
**Warning signs:** `/personal-learnings/new` redirects to `/library?tab=personal` instead of showing the form.

### Pitfall 4: `selected_match_id` Context Not Available to Panel Contents
**What goes wrong:** `provide_context(selected_match_id)` is called inside `LibrarySurfaces` AFTER `view!` constructs `RoomSurface`. Panel closures (`PanelFn`) are called inside `RoomSurface`'s `view!`, which runs during the reactive tree construction phase — if the context hasn't been provided before `view!` is called, `use_context` inside `StatsSurface`/`MatchDetailSurface` returns `None`.
**Why it happens:** Leptos context follows the reactive owner tree; `provide_context` must happen before the children that consume it are constructed.
**How to avoid:** Call `provide_context(selected_match_id)` at the top of `LibrarySurfaces`, before the `view!` block. The panels are constructed lazily (their `PanelFn` closures capture `selected_match_id` by value at construction time, not call time).
**Warning signs:** `use_context::<RwSignal<Option<String>>>()` returns `None` inside `StatsSurface`; `expect("...")` panics at WASM runtime.

### Pitfall 5: Growth DB Query Uses `post_game_learning.team` But `match.team_id`
**What goes wrong:** The `post_game_learning` table uses `team record<team>` (field name: `team`). The `match` table uses `team_id option<record<team>>` (field name: `team_id`). Using the wrong field name in the count query silently returns 0.
**Why it happens:** Different schema field names for the team foreign key across tables.
**How to avoid:** `SELECT count() as n FROM match WHERE team_id = type::record(...)` vs `SELECT count() as n FROM post_game_learning WHERE team = type::record(...)`. Verify against `schema.surql:49` (match) and `schema.surql` post_game_learning table definition.
**Warning signs:** `records_tier` stays 0 even after syncing matches; `tomes_tier`/`analysis_tier` return correct values.

---

## Code Examples

### Existing War Room Template (direct copy target)

```rust
// Source: src/pages/rooms/war_room.rs:61-125
// LibraryRoom mirrors this exactly, substituting axis names and asset paths.
#[component]
pub fn WarRoomRoom() -> impl IntoView {
    let region = use_context::<InitialTheme>().unwrap_or_default().0; // ONCE
    let growth = Resource::new(|| (), |_| get_team_growth_state());
    let tactics_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.tactics_tier).unwrap_or(0)
    });
    // ... intel_tier, command_tier ...
    let dir = format!("/img/rooms/war-room/{region}");
    view! {
        <AtmosphereRoot class="min-h-screen" tactics_tier=tactics_tier ...>
            <Plane ... /> <GrowthOverlay ... />
            <div class="relative z-10 px-8 py-6">
                <WarSurfaces region=surfaces_region />
            </div>
        </AtmosphereRoot>
    }
}
```

### `AtmosphereRoot` Library Axis Props (already wired)

```rust
// Source: src/components/room/parallax.rs:71-73
// These props already exist — just pass the signals:
#[prop(optional, into)] records_tier: Signal<u8>,
#[prop(optional, into)] tomes_tier: Signal<u8>,
#[prop(optional, into)] analysis_tier: Signal<u8>,
```

### `RoomSurface` API (confirmed, no change needed)

```rust
// Source: src/components/room/room_surface.rs:74-78
pub fn RoomSurface(
    panels: Vec<(SurfaceTab, PanelFn)>,
) -> impl IntoView { ... }
// PanelFn = Box<dyn Fn() -> AnyView>
// SurfaceTab { id: &'static str, label: &'static str }
// Unknown/missing ?tab= → index 0 (first tab)
```

### Growth Count Query Shape

```rust
// Source: src/server/db.rs:4164-4207 (get_team_growth_counts — exact template)
pub async fn get_library_growth_counts(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<LibraryGrowthCounts> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut result = db.query(
        "SELECT count() as n FROM match WHERE team_id = type::record('team', $team_key) GROUP ALL; \
         SELECT count() as n FROM post_game_learning WHERE team = type::record('team', $team_key) GROUP ALL; \
         SELECT count() as n FROM team_note WHERE team = type::record('team', $team_key) GROUP ALL",
    ).bind(("team_key", team_key)).await?;
    // take(0) → match_syncs, take(1) → post_game_learning, take(2) → team_note
}
```

### Match ID Type

```
type: String
field: riot_match_id (in TeamMatchRow / MatchGroup)
source: match.match_id (the Riot API match id, e.g. "EUW1_6940123456")
used in: fetch_match_detail(match_id: String) — the server fn takes this String directly
```

[VERIFIED: codebase — `src/pages/stats.rs:13`, `src/pages/match_detail.rs:10`]

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|-----------------|--------|
| Stats `<a href="/match/:id">` link | Switch tab + set shared RwSignal | Match Detail stays in-room; no full page reload |
| Flat `/stats`, `/analytics` etc. routes | 302-redirect to Library tab | One canonical home per feature; deep links preserved |
| `match.team_id` excluded from TeamGrowthCounts | Included in LibraryGrowthCounts as `match_syncs` | Library "records" axis now reflects actual match archive size |

**Deprecated/outdated in this phase:**
- `StatsPage` standalone component: replace with `StatsSurface` + redirect shell
- `AnalyticsPage` standalone: replace with `AnalyticsSurface` + redirect shell
- `PostGamePage` standalone: replace with `PostGameSurface` + redirect shell
- `PersonalLearningsPage` standalone: replace with `PersonalLearningsSurface` + redirect shell (keep `NewLearningPage`)
- `MatchDetailPage` standalone: replace with `MatchDetailSurface` + redirect shell

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Tier thresholds for `records` [1,10,30], `tomes` [1,5,15], `analysis` [1,5,15] are reasonable for beta calibration | Growth Count Sources | Low — thresholds are explicitly labelled hypotheses in Phase 26/27; adjustable without code changes to surfaces |
| A2 | `personal_learning` count excluded from `tomes` axis (team-only scope for Phase 28) simplifies the query and is acceptable | Growth Count Sources | Low — D-14 explicitly says team-scoped growth is the Phase 28 decision; blended growth is deferred |
| A3 | Redirect shells use `#[server] fn + leptos_axum::redirect()` + `#[cfg(hydrate)] window.location().set_href()` dual pattern | Architecture Patterns (Pattern 2) | Medium — if Leptos 0.8 has a dedicated `<Redirect>` component that handles both SSR and WASM correctly, that would be cleaner; current codebase shows no `<Redirect>` usage anywhere (grepped across all `.rs` files) |

---

## Open Questions

1. **`/match/:id` redirect — query param vs nested route**
   - What we know: `MatchDetailSurface` needs to receive the match id; `RoomSurface` uses `?tab=` for surface switching; there is no sub-routing inside `RoomSurface` (by design, D-02)
   - What's unclear: Whether Leptos's `use_query_map` cleanly handles two params simultaneously (`?tab=match&id=EUW1_123`) or if the `id` param conflicts with anything
   - Recommendation: Use `?tab=match&id={riot_match_id}` — two independent `use_query_map` reads, one for `tab` (in `RoomSurface`) and one for `id` (in `MatchDetailSurface`). This is the simplest approach and follows the existing `?tab=` pattern.

2. **`StatsPage` in-page match selection (lines ~640-660)**
   - What we know: `stats.rs` already has a `selected_match: RwSignal<Option<String>>` internal signal at ~line 656 for in-page match selection/expansion
   - What's unclear: Whether `StatsSurface` should remove this internal selection signal entirely (since match detail moves to its own tab) or keep it for in-stats-surface row highlighting
   - Recommendation: Keep the internal row-highlight selection but replace the `<a href>` CTA with the cross-surface tab switch. Both signals can coexist.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is code/config-only changes. No new external tools, services, or CLIs required beyond the existing project stack.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Playwright (e2e) + Rust `#[cfg(test)]` unit tests |
| Config file | `e2e/playwright.config.ts` |
| Quick run command | `cd e2e && npx playwright test pages.spec.ts` |
| Full suite command | `cd e2e && npx playwright test` |

### SC1–SC5 → Test Map

| SC | Behavior | Test Type | Automated Command | File Exists? |
|----|----------|-----------|-------------------|-------------|
| SC1 | `/library` route loads with tab strip visible, no JS errors | smoke | `cd e2e && npx playwright test pages.spec.ts -g "/library loads"` | ❌ Wave 0 — add `/library` entry to `AUTHED_PAGES` |
| SC2 | `/personal-learnings`, `/stats`, `/analytics`, `/post-game`, `/opponents` resolve (redirect, not 404) | smoke | `cd e2e && npx playwright test pages.spec.ts` | ❌ Wave 0 — update those entries to expect redirect destination |
| SC3 | `/personal-learnings/new` still loads its create form | smoke | `cd e2e && npx playwright test pages.spec.ts -g "/personal-learnings/new"` | ✅ existing entry |
| SC4 | Tier-2 team sees `data-records-tier="2"` on the atmosphere root | unit | `cargo test --features ssr --lib -- growth` | ❌ Wave 0 — add `library_growth_tier_derivation` test to `growth.rs` |
| SC5 | `/library` hydrates without tachys panic in both demacia and pandemonium regions | e2e canary | `cd e2e && npx playwright test hydration-no-panic.spec.ts -g "/library"` | ❌ Wave 0 — add `/library` to `SCOPED_ROUTES` in `hydration-no-panic.spec.ts` |

### e2e Suite Changes Required (detailed)

**`e2e/tests/pages.spec.ts` AUTHED_PAGES:**

```typescript
// ADD:
{ path: "/library", content: /stats|match|analytics|post.game|learnings/i },

// UPDATE (routes now redirect — test must expect the redirect destination, not the old standalone page):
// Option A: remove these entries (the redirects resolve to /library which is tested above)
// Option B: update `content` regex to match /library page content
// Recommended: remove /stats, /analytics, /post-game, /personal-learnings, /opponents entries
// and keep /library covering SC1/SC2 together.

// KEEP UNCHANGED:
{ path: "/personal-learnings/new", content: /learning|new|happened|learned/i }, // SC3
```

**`e2e/tests/hydration-no-panic.spec.ts` SCOPED_ROUTES:**

```typescript
// ADD to SCOPED_ROUTES:
"/library",
// Note: /stats, /post-game, /match/test-match-id are in SCOPED_ROUTES;
// after redirect they resolve to /library — the existing tests now exercise
// the redirect-then-load path, which is a valid canary continuation.
// Do NOT remove /match/test-match-id — it tests the param-carrying redirect.
```

### Sampling Rate
- **Per task commit:** `cargo check --features ssr` + `cargo check --features hydrate --target wasm32-unknown-unknown`
- **Per wave merge:** `cd e2e && npx playwright test pages.spec.ts hydration-no-panic.spec.ts`
- **Phase gate:** Full e2e suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `e2e/tests/pages.spec.ts` — add `/library` entry to `AUTHED_PAGES`; update/remove redirected route entries
- [ ] `e2e/tests/hydration-no-panic.spec.ts` — add `/library` to `SCOPED_ROUTES` array
- [ ] `src/models/growth.rs` unit tests — add `library_records_tier`, `library_tomes_tier`, `library_analysis_tier` tests following existing pattern (lines 229-282)

---

## Security Domain

`security_enforcement` is not set to `false` in config. However, this phase adds no authentication changes, no new user input surface beyond redirects, and no new data access paths beyond mirroring the existing `get_team_growth_counts` pattern which already enforces team membership authorization.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | Redirects preserve existing auth guards |
| V3 Session Management | No | No session changes |
| V4 Access Control | Yes | `get_library_growth_state` must validate team membership before querying — same `get_user_team_id` gate as `get_team_growth_state` |
| V5 Input Validation | Yes (low) | `match_id` in redirect shell is read from URL params; passed to `fetch_match_detail` which already validates it |
| V6 Cryptography | No | No new crypto |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via `region` in asset URLs | Tampering | Region is server-set from cookie (war_room.rs T-27-07-02 comment); already mitigated — Library inherits this |
| Team id spoofing in growth count query | Elevation of Privilege | `get_library_growth_counts` must use pre-authorized `team_id` from `get_user_team_id` — never accept raw client input (same contract as `get_team_growth_counts:4163`) |

---

## Sources

### Primary (HIGH confidence)
- `src/pages/rooms/war_room.rs` — direct structural template; all War Room patterns verified by reading
- `src/components/room/room_surface.rs` — `RoomSurface` API contract, `?tab=` resolver behavior
- `src/components/room/parallax.rs:54-79` — `AtmosphereRoot` props including Library axis declarations
- `src/models/growth.rs` — `TeamGrowthCounts`/`TeamGrowthState` pattern; `tier_for` function
- `src/server/db.rs:4109-4207` — `get_growth_counts` and `get_team_growth_counts` batch query shape
- `schema.surql:44-52` — `match.team_id` field type confirming team-scoped match counting
- `e2e/tests/pages.spec.ts` — current `AUTHED_PAGES` entries
- `e2e/tests/hydration-no-panic.spec.ts` — `SCOPED_ROUTES` and `PANIC_REGEX`
- `src/app.rs:139-167` — current route definitions

### Secondary (MEDIUM confidence)
- `src/pages/profile.rs:28-34`, `src/pages/team/roster.rs:9-23` — `leptos_axum::redirect` usage pattern
- `src/pages/match_detail.rs:543-565` — `MatchDetailPage` component structure and `use_params_map` usage

### Tertiary (LOW confidence — flagged as ASSUMED above)
- Tier threshold values — calibration hypotheses following Phase 26/27 precedent

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all components verified in codebase; no new packages
- Architecture: HIGH — verified by reading `war_room.rs`, `parallax.rs`, `room_surface.rs`
- Growth count sources: HIGH (table fields) / ASSUMED (thresholds)
- Redirect mechanism: HIGH — verified by reading existing usage in `profile.rs` and `roster.rs`
- Pitfalls: HIGH — Phase 27/18.2 evidence trail fully documented

**Research date:** 2026-06-06
**Valid until:** 2026-07-06 (stable stack; 30-day window)
