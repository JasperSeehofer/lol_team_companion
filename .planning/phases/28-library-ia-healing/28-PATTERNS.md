# Phase 28: Library & IA Healing - Pattern Map

**Mapped:** 2026-06-06
**Files analyzed:** 10 new/modified files
**Analogs found:** 10 / 10

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/pages/rooms/library.rs` | page + service | request-response | `src/pages/rooms/war_room.rs` | exact |
| `src/models/growth.rs` (modify) | model | transform | `src/models/growth.rs` (`TeamGrowthCounts`/`TeamGrowthState`) | exact |
| `src/server/db.rs` (modify) | service | CRUD (batch query) | `src/server/db.rs:4164-4207` (`get_team_growth_counts`) | exact |
| `src/pages/stats.rs` (extract `StatsSurface`) | component | request-response | `src/pages/draft.rs` (`DraftSurface`) via War Room Phase 27 pattern | role-match |
| `src/pages/match_detail.rs` (extract `MatchDetailSurface`) | component | request-response | `src/pages/match_detail.rs` (`MatchDetailPage`) | exact-source |
| `src/pages/analytics.rs` (extract `AnalyticsSurface`) | component | request-response | `src/pages/analytics.rs` (`AnalyticsPage`) | exact-source |
| `src/pages/post_game.rs` (extract `PostGameSurface`) | component | CRUD | `src/pages/post_game.rs` (`PostGamePage`) | exact-source |
| `src/pages/personal_learnings.rs` (extract `PersonalLearningsSurface`) | component | CRUD | `src/pages/personal_learnings.rs` (`PersonalLearningsPage`) | exact-source |
| `src/app.rs` (modify routes) | config | request-response | `src/app.rs:156-161` (existing room routes + redirect shells) | exact |
| `src/components/room/mod.rs` (modify) | config | — | `src/components/room/mod.rs:9-11` (current re-exports) | exact |
| `src/pages/rooms/mod.rs` (modify) | config | — | `src/pages/rooms/mod.rs:5-6` (current module decls) | exact |
| `e2e/tests/pages.spec.ts` (modify) | test | — | `e2e/tests/pages.spec.ts:8-27` (AUTHED_PAGES array) | exact |
| `e2e/tests/hydration-no-panic.spec.ts` (modify) | test | — | `e2e/tests/hydration-no-panic.spec.ts:72-85` (SCOPED_ROUTES) | exact |

---

## Pattern Assignments

### `src/pages/rooms/library.rs` — `LibraryRoom` + `LibrarySurfaces` + `get_library_growth_state`

**Primary analog:** `src/pages/rooms/war_room.rs` (direct structural template — line-for-line copy with axis name substitution)

**Imports pattern** (`war_room.rs` lines 11-18):
```rust
use leptos::prelude::*;

use crate::app::InitialTheme;
use crate::components::room::{AtmosphereRoot, GrowthOverlay, Plane, PanelFn, RoomSurface, SurfaceTab};
use crate::pages::stats::StatsSurface;
use crate::pages::match_detail::MatchDetailSurface;
use crate::pages::analytics::AnalyticsSurface;
use crate::pages::post_game::PostGameSurface;
use crate::pages::personal_learnings::PersonalLearningsSurface;
```

**Server function pattern** (`war_room.rs` lines 27-53) — copy verbatim, substituting `LibraryGrowthState` and `get_library_growth_counts`:
```rust
#[server]
pub async fn get_library_growth_state(
) -> Result<crate::models::growth::LibraryGrowthState, ServerFnError> {
    use crate::models::growth::LibraryGrowthState;
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    // No team → all-zero default (rule #44: never Err on absent optional data).
    let team_id = match db::get_user_team_id(&surreal, &user.id).await? {
        Some(id) => id,
        None => return Ok(LibraryGrowthState::default()),
    };

    let counts = db::get_library_growth_counts(&surreal, &team_id)
        .await
        .unwrap_or_default();
    Ok(LibraryGrowthState::from_counts(counts))
}
```

**Region-read-once + Resource + tier Signal::derive pattern** (`war_room.rs` lines 61-93) — THE CRITICAL HYDRATION SAFETY PATTERN (rule #58):
```rust
#[component]
pub fn LibraryRoom() -> impl IntoView {
    // Region read ONCE at room entry (never inside primitives) — rule #58.
    let region = use_context::<InitialTheme>().unwrap_or_default().0;

    let growth = Resource::new(|| (), |_| get_library_growth_state());
    let records_tier = Signal::derive(move || {
        growth
            .get()
            .and_then(|r| r.ok())
            .map(|g| g.records_tier)
            .unwrap_or(0)
    });
    let tomes_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.tomes_tier).unwrap_or(0)
    });
    let analysis_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.analysis_tier).unwrap_or(0)
    });

    // Asset dir — region is server-set from cookie (T-28-*: no path traversal).
    let dir = format!("/img/rooms/library/{region}");
    let surfaces_region = region.clone();
    // ...
}
```

**AtmosphereRoot + Plane + GrowthOverlay + surface layer composition** (`war_room.rs` lines 95-125):
```rust
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

            <div
                class="fixed inset-0 pointer-events-none"
                style="z-index: 8; background: radial-gradient(ellipse at center, transparent 0%, var(--color-base) 95%);"
                aria-hidden="true"
            ></div>

            <div class="relative z-10 px-8 py-6" data-feedback-label="Library">
                <LibrarySurfaces region=surfaces_region />
            </div>
        </AtmosphereRoot>
    }
```

**WarSurfaces → LibrarySurfaces pattern** (`war_room.rs` lines 134-171):
```rust
#[component]
fn LibrarySurfaces(region: String) -> impl IntoView {
    // Provide shared match-selection signal BEFORE view! (Pitfall 4 prevention).
    let selected_match_id: RwSignal<Option<String>> = RwSignal::new(None);
    provide_context(selected_match_id);

    // Clone region only for surfaces that need it (e.g. none of the five need it
    // in content, but keep the clone pattern for future-proofing if needed).
    let panels: Vec<(SurfaceTab, PanelFn)> = vec![
        (
            SurfaceTab { id: "stats", label: "Stats" },
            Box::new(move || view! { <StatsSurface /> }.into_any()) as PanelFn,
        ),
        (
            SurfaceTab { id: "match", label: "Match Detail" },
            Box::new(move || view! { <MatchDetailSurface /> }.into_any()) as PanelFn,
        ),
        // ... AnalyticsSurface, PostGameSurface, PersonalLearningsSurface
    ];
    view! { <RoomSurface panels=panels /> }
}
```

**CRITICAL ANTI-PATTERN** (never do this — causes the Phase 18.2 hydration panic):
```rust
// WRONG — reading context inside LibrarySurfaces or any *Surface:
#[component]
fn LibrarySurfaces(/* no region prop */) -> impl IntoView {
    let region = use_context::<InitialTheme>().unwrap_or_default().0; // PANIC at html/mod.rs:217
}
```

---

### `src/models/growth.rs` — add `LibraryGrowthCounts` + `LibraryGrowthState`

**Analog:** `src/models/growth.rs` lines 101-282 (`TeamGrowthCounts`/`TeamGrowthState` + tests)

**Struct + `from_counts` pattern** (lines 108-155):
```rust
// ADD after the TeamGrowthState impl (line 155)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryGrowthCounts {
    pub match_syncs: u32,        // match table, team_id-scoped  (records axis)
    pub post_game_learning: u32, // post_game_learning table, team-scoped
    pub team_note: u32,          // team_note table, team-scoped
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryGrowthState {
    pub counts: LibraryGrowthCounts,
    pub records_tier: u8,   // the archive grows as matches accumulate
    pub tomes_tier: u8,     // written knowledge (reviews + team notes)
    pub analysis_tier: u8,  // analytical depth (post-game reviews)
}

impl LibraryGrowthState {
    pub fn from_counts(counts: LibraryGrowthCounts) -> Self {
        // D-13 thresholds: hypotheses for beta tuning (same philosophy as D-09)
        let records  = tier_for(counts.match_syncs, [1, 10, 30]);
        let tomes    = tier_for(counts.post_game_learning + counts.team_note, [1, 5, 15]);
        let analysis = tier_for(counts.post_game_learning, [1, 5, 15]);
        LibraryGrowthState {
            counts,
            records_tier: records.level(),
            tomes_tier: tomes.level(),
            analysis_tier: analysis.level(),
        }
    }
}

impl Default for LibraryGrowthState {
    fn default() -> Self {
        LibraryGrowthState::from_counts(LibraryGrowthCounts::default())
    }
}
```

**Test pattern** (lines 229-282 — copy test shape for Library axes):
```rust
// In #[cfg(test)] mod tests — ADD after the team growth tests (line 282):
#[test]
fn library_empty_counts_are_all_seed() {
    let g = LibraryGrowthState::default();
    assert_eq!(g.records_tier, 0);
    assert_eq!(g.tomes_tier, 0);
    assert_eq!(g.analysis_tier, 0);
}

#[test]
fn library_records_tracks_match_syncs() {
    // 10 syncs → exactly the tier-2 bound (10).
    let g = LibraryGrowthState::from_counts(LibraryGrowthCounts {
        match_syncs: 10, ..Default::default()
    });
    assert_eq!(g.records_tier, 2);
}

#[test]
fn library_tomes_combines_reviews_and_notes() {
    // 3 reviews + 2 notes = 5 → tier-2 bound (5).
    let g = LibraryGrowthState::from_counts(LibraryGrowthCounts {
        post_game_learning: 3,
        team_note: 2,
        ..Default::default()
    });
    assert_eq!(g.tomes_tier, 2);
}

#[test]
fn library_analysis_tracks_post_game_reviews() {
    let g = LibraryGrowthState::from_counts(LibraryGrowthCounts {
        post_game_learning: 15, ..Default::default()
    });
    assert_eq!(g.analysis_tier, 3);
}
```

---

### `src/server/db.rs` — add `get_library_growth_counts`

**Analog:** `src/server/db.rs` lines 4164-4207 (`get_team_growth_counts`) — copy with three statements instead of six.

**Full function pattern** (lines 4164-4207):
```rust
/// Accumulation counts for a team, used to drive the Library room growth
/// atmosphere. Three `count() … GROUP ALL` in one round-trip.
///
/// Caller contract: `team_id` MUST be pre-authorized (same as `get_team_growth_counts`).
/// Note: field names differ per table — `match.team_id` vs `post_game_learning.team` (Pitfall 5).
pub async fn get_library_growth_counts(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<LibraryGrowthCounts> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    let mut result = db
        .query(
            "SELECT count() as n FROM match \
               WHERE team_id = type::record('team', $team_key) GROUP ALL; \
             SELECT count() as n FROM post_game_learning \
               WHERE team = type::record('team', $team_key) GROUP ALL; \
             SELECT count() as n FROM team_note \
               WHERE team = type::record('team', $team_key) GROUP ALL",
        )
        .bind(("team_key", team_key))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct CountResult { n: i64 }

    fn count_of(rows: Vec<CountResult>) -> u32 {
        rows.first().map(|r| r.n.max(0) as u32).unwrap_or(0)
    }

    let match_syncs       = count_of(result.take(0).unwrap_or_default());
    let post_game_learning = count_of(result.take(1).unwrap_or_default());
    let team_note         = count_of(result.take(2).unwrap_or_default());

    Ok(LibraryGrowthCounts { match_syncs, post_game_learning, team_note })
}
```

**CRITICAL NOTE — field name mismatch (Pitfall 5 from RESEARCH.md):**
- `match` table uses `team_id` → `WHERE team_id = type::record('team', $team_key)`
- `post_game_learning` + `team_note` tables use `team` → `WHERE team = type::record('team', $team_key)`
- Using the wrong field name silently returns 0.

---

### `*Surface` extractions — five page files

**Analog for the extraction pattern:** Phase 27 War Room surface extractions. The recipe is defined in RESEARCH.md Pattern 5:

**What to STRIP** from each page component to produce its `*Surface`:
- `let auth_user = Resource::new(...); Effect::new(...)` auth guard (examples at `stats.rs:282-290`, `personal_learnings.rs:192-200`, `post_game.rs:358-366`)
- `let theme = use_context::<InitialTheme>()...; let region = ...;` theme read (examples at `post_game.rs:369-370`, `match_detail.rs:559-560`)
- `view! { <div class="canvas-grain bg-base min-h-screen">` outer wrapper (examples at `stats.rs:331`, `personal_learnings.rs:213`, `analytics.rs:140`, `post_game.rs` equivalent)
- `<SectionHead ... title=... />` or `<h1>` page title block (examples at `personal_learnings.rs:217-228`, `analytics.rs:142-149`)
- `<div class="max-w-Xxxxxl mx-auto px-Y py-Z">` page padding wrapper

**What to KEEP**: all data Resources, CRUD signals, form handlers, table/list views.

**`StatsSurface` specifics** (`src/pages/stats.rs` — entry at line 280):
- Keep: `stats` Resource, `sync_result`/`syncing` signals, filters (`min_players`, `filter_player`, etc.), the full match table view
- Strip: auth guard (lines 282-290), `has_team` Resource, outer `<div class="canvas-grain bg-base min-h-screen">` (line 331), `data-feedback-label="Stats"` wrapper, `BattleLogHeadline` page header section
- **Match CTA replacement (Pitfall 2 from RESEARCH.md):** Replace `<a href="/match/{riot_id}">` link (stats.rs ~line 933) with shared signal dispatch:
```rust
let selected_match_id = use_context::<RwSignal<Option<String>>>().expect("LibrarySurfaces provides selected_match_id");
let navigate = use_navigate();
// on match row CTA:
on:click=move |_| {
    selected_match_id.set(Some(riot_id.clone()));
    navigate("?tab=match", Default::default());
}
```

**`MatchDetailSurface` specifics** (`src/pages/match_detail.rs` — entry at line 543):
- Keep: `params`/`match_id` extraction (but replace with shared signal + query param fallback), `detail` Resource, all comparison/filter signals, full scoreboard view
- Strip: auth guard (lines 548-556), `let theme = ...` / `let region = ...` (lines 559-560), outer wrapper
- **Shared signal + query param fallback**:
```rust
let selected_match_id = use_context::<RwSignal<Option<String>>>()
    .expect("LibrarySurfaces provides selected_match_id");
let query = use_query_map();
let match_id = Signal::derive(move || {
    selected_match_id.get()
        .or_else(|| query.read().get("id"))
        .unwrap_or_default()
});
let detail = Resource::new(match_id, |id| fetch_match_detail(id));
```
- Must also add a recent-match list in-tab for when `match_id` is empty (D-06) — fallback "pick a match" list.

**`AnalyticsSurface` specifics** (`src/pages/analytics.rs` — entry at line 116):
- Keep: `analytics` Resource (line 130), `sort_state` signal (line 133), `open_plan` signal (line 137), full Suspense view block
- Strip: auth guard (lines 118-128, which is `#[cfg(feature = "hydrate")]`-gated), outer `<div class="canvas-grain bg-base min-h-screen">` (line 140), `<main>` wrapper, Imperial header section (lines 142-149)

**`PostGameSurface` specifics** (`src/pages/post_game.rs` — entry at line 356):
- Keep: all Resources (`reviews`, `plans`, `drafts`, `match_ids`), all form signals, CRUD handlers, full review list view
- Strip: auth guard (lines 358-366), `let theme = use_context::<InitialTheme>()...; let region = ...` (lines 369-370), outer wrappers
- **No new `recursion_limit` needed**: `#![recursion_limit = "512"]` is already set project-wide in `lib.rs` and `main.rs` (rule #38).

**`PersonalLearningsSurface` specifics** (`src/pages/personal_learnings.rs` — entry at line 190):
- Keep: `learnings` Resource (line 202), all filter/sort signals (lines 204-208), `expanded_id` signal (line 208), full learning list/detail view
- Strip: auth guard (lines 192-200), outer `<div class="canvas-grain bg-base min-h-screen">` (line 213), page header including `<h1>` (lines 216-228)
- **Keep the `+ New learning` button but update href** — change `href="/personal-learnings/new"` (line 225) to continue pointing at `/personal-learnings/new` (D-11 mandate: that route stays functional)

---

### `src/app.rs` — add `/library` route + six redirect shells

**Analog:** `src/app.rs` lines 156-161 (existing room routes + current flat routes being replaced)

**New `/library` route** (after `/war-room` at line 158):
```rust
<Route path=path!("/journey") view=SoloJourneyRoom />
<Route path=path!("/war-room") view=WarRoomRoom />
<Route path=path!("/library") view=LibraryRoom />  // ADD
```

**Redirect shell shape** — two analogs: `roster.rs:9-23` (`leptos_axum::redirect` in `#[server]` + `create_team` action) and `profile.rs:28-36` (`logout` redirect). The stateless-page-redirect pattern requires a thin component + server fn. Six routes become redirect shells:

`/stats`, `/analytics`, `/post-game`, `/personal-learnings`, `/opponents` — replace `view=StatsPage` etc. with redirect components:
```rust
// In app.rs, replace:
<Route path=path!("/stats") view=StatsPage />
// With:
<Route path=path!("/stats") view=StatsRedirect />

// /opponents redirects to War Room:
// Replace:
<Route path=path!("/opponents") view=OpponentsPage />
// With:
<Route path=path!("/opponents") view=OpponentsRedirect />
```

`/match/:id` — replace with param-carrying redirect:
```rust
// Replace:
<Route path=path!("/match/:id") view=MatchDetailPage />
// With:
<Route path=path!("/match/:id") view=MatchRedirect />
```

**KEEP UNCHANGED** (D-11):
```rust
<Route path=path!("/personal-learnings/new") view=NewLearningPage />
```
This must remain above any redirect for `/personal-learnings` (declare more-specific routes first to prevent prefix matching).

**Redirect component shape** (mirrors `roster.rs` server fn + `profile.rs` redirect, RESEARCH.md Pattern 2 + Pattern 3):
```rust
// In the page file or a new redirects.rs — one per redirected route:
#[server]
pub async fn redirect_to_library_stats() -> Result<(), ServerFnError> {
    leptos_axum::redirect("/library?tab=stats");
    Ok(())
}

#[component]
pub fn StatsRedirect() -> impl IntoView {
    // SSR: leptos_axum::redirect fires on server render → 302 header.
    let action = Action::new(|_: &()| redirect_to_library_stats());
    action.dispatch(());

    // WASM/client-side path: hard-navigate when JS is running.
    #[cfg(feature = "hydrate")]
    {
        if let Some(win) = web_sys::window() {
            let _ = win.location().set_href("/library?tab=stats");
        }
    }

    view! { <></> }
}
```

**Param-carrying redirect for `/match/:id`** (RESEARCH.md Pattern 3, `match_detail.rs:563-565`):
```rust
#[component]
pub fn MatchRedirect() -> impl IntoView {
    use leptos_router::hooks::use_params_map;
    let params = use_params_map();
    let id = params.read().get("id").unwrap_or_default();
    let target = format!("/library?tab=match&id={id}");

    // SSR dispatch:
    let id_clone = id.clone();
    let action = Action::new(move |_: &()| {
        let t = format!("/library?tab=match&id={}", id_clone.clone());
        async move {
            leptos_axum::redirect(&t);
            Ok::<(), ServerFnError>(())
        }
    });
    action.dispatch(());

    // WASM path:
    #[cfg(feature = "hydrate")]
    {
        if let Some(win) = web_sys::window() {
            let _ = win.location().set_href(&target);
        }
    }

    view! { <></> }
}
```

---

### `src/components/room/mod.rs` — add Library re-exports

**Analog:** `src/components/room/mod.rs` lines 9-11 (current re-exports pattern)

```rust
// Current:
pub use parallax::{AtmosphereRoot, GrowthOverlay, Plane};
pub use quick_nav::QuickNav;
pub use room_surface::{PanelFn, RoomSurface, SurfaceTab};

// No change needed — LibraryRoom/LibrarySurfaces live in src/pages/rooms/library.rs,
// not in src/components/room/. This file needs NO modification.
// (WarRoomRoom/WarSurfaces are also not re-exported here — same pattern.)
```

**`src/pages/rooms/mod.rs` — add library module:**
```rust
// Current:
pub mod solo_journey;
pub mod war_room;

// ADD:
pub mod library;
```

---

### `e2e/tests/pages.spec.ts` — update AUTHED_PAGES

**Analog:** `e2e/tests/pages.spec.ts` lines 8-27 (AUTHED_PAGES array)

```typescript
// ADD new entry:
{ path: "/library", content: /stats|match|analytics|post.game|learnings/i },

// REMOVE (these routes now redirect; the /library entry above covers SC1/SC2 together):
// { path: "/stats", content: /stats|match|history/i },
// { path: "/analytics", content: /analytics|strategy|effectiveness/i },
// { path: "/post-game", content: /post|game|review/i },
// { path: "/personal-learnings", content: /learnings|personal|reflections/i },
// { path: "/opponents", ... },

// KEEP UNCHANGED (SC3 — D-11):
{ path: "/personal-learnings/new", content: /learning|new|happened|learned/i },

// KEEP UNCHANGED (War Room, Phase 27):
{ path: "/war-room", content: /draft|war|room|plan/i },
```

---

### `e2e/tests/hydration-no-panic.spec.ts` — add `/library` to SCOPED_ROUTES

**Analog:** `e2e/tests/hydration-no-panic.spec.ts` lines 72-85 (SCOPED_ROUTES array)

```typescript
// Current SCOPED_ROUTES (lines 72-85):
const SCOPED_ROUTES = [
  "/solo",
  "/draft",
  "/team/dashboard",
  "/tree-drafter",
  "/champion-pool",
  "/post-game",         // will now redirect → /library, still valid canary
  "/match/test-match-id", // will now redirect → /library?tab=match&id=test-match-id
  "/journey",
] as const;

// ADD "/library" to the array (SC5 — both regions):
const SCOPED_ROUTES = [
  "/solo",
  "/draft",
  "/team/dashboard",
  "/tree-drafter",
  "/champion-pool",
  "/post-game",
  "/match/test-match-id",
  "/journey",
  "/library",   // ADD — SC5: Library hydrates without tachys panic in both regions
] as const;

// DO NOT REMOVE /post-game or /match/test-match-id — they now test the
// redirect-then-library-load path, which is a valid additional canary.
```

---

## Shared Patterns

### Hydration Safety (rule #58) — apply to `LibraryRoom` ONLY
**Source:** `src/pages/rooms/war_room.rs` lines 64, `src/pages/rooms/solo_journey.rs:18`
**Apply to:** `LibraryRoom` (the one place `InitialTheme` is read for the Library module)
```rust
// At the TOP of LibraryRoom() body — never anywhere else in the module:
let region = use_context::<InitialTheme>().unwrap_or_default().0;
```
`LibrarySurfaces` and all five `*Surface` components MUST NOT call `use_context::<InitialTheme>()`.

### Server Function Auth + DB Context Pattern
**Source:** `src/pages/rooms/war_room.rs` lines 36-41
**Apply to:** `get_library_growth_state` in `library.rs`
```rust
let auth: AuthSession = leptos_axum::extract().await?;
let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
let surreal = use_context::<Arc<Surreal<Db>>>()
    .ok_or_else(|| ServerFnError::new("No DB context"))?;
```

### No-Team → Default Pattern (rule #44)
**Source:** `src/pages/rooms/war_room.rs` lines 44-47
**Apply to:** `get_library_growth_state`
```rust
let team_id = match db::get_user_team_id(&surreal, &user.id).await? {
    Some(id) => id,
    None => return Ok(LibraryGrowthState::default()),  // never Err on absent team
};
```

### Batch COUNT Query Shape (surreal-patterns rule #29)
**Source:** `src/server/db.rs` lines 4168-4206
**Apply to:** `get_library_growth_counts` in `db.rs`
- `strip_prefix("team:")` before binding, always pass owned `String` to `.bind()` (surreal rule #2 + #4)
- `#[derive(Debug, Deserialize, SurrealValue)] struct CountResult { n: i64 }` inner struct
- `count_of` helper fn: `.first().map(|r| r.n.max(0) as u32).unwrap_or(0)`
- `result.take(N).unwrap_or_default()` for each statement (surreal rule #28)

### provide_context Before view! (Pitfall 4 prevention)
**Source:** Leptos context owner-tree semantics; confirmed pattern in `src/pages/post_game.rs:385` (`use_context::<ToastContext>`)
**Apply to:** `LibrarySurfaces` — `selected_match_id` signal
```rust
// MUST be before view! { <RoomSurface ... /> }
let selected_match_id: RwSignal<Option<String>> = RwSignal::new(None);
provide_context(selected_match_id);
```

### WASM-only redirect guard (wasm-patterns rule #37)
**Source:** `src/pages/stats.rs` lines 285-289 (auth redirect), `src/pages/analytics.rs` lines 122-128
**Apply to:** All six redirect shell components
```rust
#[cfg(feature = "hydrate")]
{
    if let Some(win) = web_sys::window() {
        let _ = win.location().set_href("/library?tab=...");
    }
}
```

---

## No Analog Found

All files have analogs. No entries.

---

## Metadata

**Analog search scope:** `src/pages/rooms/`, `src/models/`, `src/server/db.rs`, `src/pages/{stats,match_detail,analytics,post_game,personal_learnings}.rs`, `src/app.rs`, `src/components/room/`, `e2e/tests/`
**Files scanned:** 14 source files read directly
**Pattern extraction date:** 2026-06-06

**Key structural facts verified from codebase:**
- `AtmosphereRoot` Library axis props (`records_tier`, `tomes_tier`, `analysis_tier`) already declared in `parallax.rs:71-73` — pass signals directly
- `RoomSurface` API: `panels: Vec<(SurfaceTab, PanelFn)>` — no changes needed
- `tier_for(count, [u32;3])` is a private fn in `growth.rs:66` — `LibraryGrowthState::from_counts` calls it directly (same file)
- `match.team_id` field name (not `team`) confirmed in `schema.surql:49`; `post_game_learning.team` + `team_note.team` use the `team` field name
- `#![recursion_limit = "512"]` already project-wide — no new recursion_limit for `PostGameSurface`
- `OpponentsSurface` already extracted in Phase 27 — Phase 28 only redirects `/opponents`, no re-extraction
- `src/components/room/mod.rs` needs NO changes — Library components live in `src/pages/rooms/library.rs`
