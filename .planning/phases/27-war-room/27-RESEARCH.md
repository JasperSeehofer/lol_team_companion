# Phase 27: War Room — Research

**Researched:** 2026-06-05
**Domain:** Leptos 0.8 SSR/hydrate room composition, extract-then-compose refactor, RoomSurface keep-mounted redesign, mousemove guard, team growth server fn
**Confidence:** HIGH — all findings grounded in the live codebase; no new external dependencies

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Surface extraction strategy:**
- D-01: Full shared extraction now — extract each feature into a chrome-free `*Surface` component; flat routes (`/draft`, `/tree-drafter`, `/game-plan`, `/opponents`) become thin shells rendering the same `*Surface`; `WarSurfaces` composes all five. Single source of truth.
- D-02: Flat-route behavior must be preserved (still resolve and render the same UI/data); regression check on each required.
- D-03: "Chrome-free" = no `<h1>`/title block, outer page padding, `min-h-screen`, or anything that assumes viewport ownership. Thin flat-route shell supplies page chrome; War Room tab supplies none.

**Ban Priority surface:**
- D-04: Extract the full ban-priority editor (`get_ban_priorities`/`save_ban_priorities`, table `ban_priority`) into its own `BanPrioritySurface`.
- D-05: Draft surface keeps its own in-draft ban handling (`banned_champions` derived from draft board slots at `draft.rs:1270`). Do NOT merge these two concepts.

**Tab-switch behavior (requires RoomSurface change):**
- D-06: Keep all panels mounted (CSS-hidden when inactive), not `<Show>` unmount/remount.
- D-07: Inactive panels use `hidden` attribute; keep `role="tabpanel"` semantics; removed from a11y/tab order.
- D-08: All five surfaces mount on room entry — 5 eager data round-trips accepted. Optimization deferred.
- D-09: RoomSurface API must change so each surface renders its own independent content (not a single `ChildrenFn` that branches). Planner decides exact contract.

**Growth feedback:**
- D-10: Environmental art only — no tier badges, counts, or growth panel.
- D-11: Success Criterion 3 verified via `data-tactics-tier`/`data-intel-tier`/`data-command-tier` attributes and overlay opacity, not visible text.

### Claude's Discretion

- Default tab = first surface (Draft); unknown/missing `?tab=` → first tab.
- Tab order fixed: Draft, Tree-Drafter, Game Plan, Opponents, Ban Priority.
- Asset path convention: `/img/rooms/war-room/{region}/...`; placeholder gradient WebPs at those paths.
- Exact plane/overlay count and parallax depths (mirror `solo_journey.rs` planes + three axis overlays).
- Internal module layout for five `*Surface` components (planner's call; parallel to `solo_dashboard.rs` → `SoloSurfaces` precedent).
- `get_team_growth_state` placement and empty/no-team handling (return `TeamGrowthState::default()`, never `Err`).
- Exact `mousemove`-guard mechanism (research suggests `static AtomicBool` per Leptos #3665).

### Deferred Ideas (OUT OF SCOPE)

- IA-healing redirects + nav changes (ROOM-07, Phase 28)
- QuickNav command palette + sub-nav suppression on room paths (ROOM-08, Phase 29)
- Real FLUX environment art (Phase 30)
- Eager-fetch optimization for the five mounted surfaces (D-08)
- In-app "Pause atmosphere" toggle
- Explicit growth readout / tier badges (rejected for Phase 27, D-10)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ROOM-04 | War Room hosts draft, tree-drafter, game-plan, opponents, and ban-priority as surfaces under one parallax atmosphere | All five extraction boundaries mapped; RoomSurface keep-mounted contract specified; `get_team_growth_state` wrapper pattern confirmed; asset paths defined; e2e canary protocol documented |
</phase_requirements>

---

## Summary

Phase 27 delivers the War Room as the first real consumer of the Phase 26 `RoomSurface` tab component. The work has four interlocking pillars: (1) refactor `RoomSurface` to keep all panels mounted and CSS-hidden, (2) extract five features into chrome-free `*Surface` components while keeping flat routes working as thin shells, (3) wrap Phase 26's already-built `get_team_growth_counts`/`TeamGrowthState::from_counts` in a `#[server]` fn called `get_team_growth_state`, and (4) add a `static AtomicBool` guard in `AtmosphereRoot` to prevent `mousemove` listener accumulation across room navigations.

All foundational infrastructure (AtmosphereRoot with all axis props, `TeamGrowthCounts`/`TeamGrowthState` with unit tests, `get_team_growth_counts` in db.rs) was shipped in Phase 26 and is verified present in the codebase. Phase 27 wires these together. The dominant risk is the ~4,500-line `draft.rs` extraction — the boundary is well-defined but the file is the largest in the project. The second risk is the `RoomSurface` API change from a single `ChildrenFn` to per-tab render props; a concrete contract is documented below.

**Primary recommendation:** Follow the `solo_dashboard.rs → SoloSurfaces → solo_journey.rs` extraction template exactly. The pattern is proven, the region-read-once rule is already codified (CLAUDE.md #58), and all data infrastructure is in place. The only genuinely new work is the `RoomSurface` keep-mounted refactor and the `AtomicBool` guard.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Parallax atmosphere + tier overlays | Frontend (WASM/SSR) | — | Pure CSS-var bridge; no server state |
| Growth tier derivation | API (server fn) | Frontend signals | Raw counts fetched server-side; pure math client-side |
| Tab switching | Frontend (signal) | — | `RwSignal<usize>` + CSS `hidden`; URL `?tab=` sync via navigate |
| Surface data (draft, game-plan, etc.) | API (#[server] fns) | Frontend Resources | Each surface has its own server fns and Resources |
| Ban priority persistence | API (db.rs) | Frontend | `get_ban_priorities`/`set_ban_priorities` already in db.rs |
| Region selection | Frontend (InitialTheme context) | SSR | Read once at room top level; never reactive |
| Asset loading | Browser (img src) | — | Active-region-only `format!("{dir}/...")` on render |
| mousemove listener lifecycle | Browser (WASM hydrate) | — | `#[cfg(feature = "hydrate")]` block in AtmosphereRoot |

---

## Standard Stack

### Core (no Cargo.toml changes)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `leptos` | 0.8 (existing) | SSR + WASM hydration, signals, Resources, server fns | Project stack |
| `surrealdb` | 3.x (existing) | DB queries for growth counts, ban priorities | Project stack |
| `web-sys` | (existing via leptos) | `AtomicBool`, `window`, `mousemove` listener | Already in Cargo.toml |
| `wasm-bindgen` | (existing) | `Closure` for mousemove/RAF | Already in Cargo.toml |

**Zero new Cargo dependencies.** All web-sys types needed are already declared. [VERIFIED: codebase grep + Phase 26 research SUMMARY.md]

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `std::sync::atomic::AtomicBool` (WASM) | std | Listener accumulation guard | Exactly one place: AtmosphereRoot |
| ImageMagick (build tool) | existing | Generate placeholder gradient WebPs | Offline, pre-commit |

**Installation:** None needed.

---

## Package Legitimacy Audit

> No new packages installed this phase — all work is within existing Cargo dependencies.

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
/war-room route
    │
    ▼
WarRoomRoom (src/pages/rooms/war_room.rs)
│  region = use_context::<InitialTheme>().0    ← read ONCE (rule #58)
│  growth = Resource::new(|| (), |_| get_team_growth_state())
│  tactics_tier, intel_tier, command_tier = Signal::derive(…)
│
└─► AtmosphereRoot(tactics_tier, intel_tier, command_tier)
       │
       ├── Plane (sky, depth=6, z=1)
       ├── Plane (mid, depth=16, z=2)
       ├── Plane (near, depth=32, z=3)
       ├── GrowthOverlay(axis="tactics", z=4)   ← opacity via data-tactics-tier CSS
       ├── GrowthOverlay(axis="intel", z=5)
       ├── GrowthOverlay(axis="command", z=6)
       ├── contrast scrim (z=8, aria-hidden)
       └── <div class="relative z-10">
              │
              └─► WarSurfaces(region)
                     │
                     ├── RoomSurface (refactored: keep-mounted per-tab ViewFn)
                     │     ├── tablist: Draft | Tree-Drafter | Game Plan | Opponents | Ban Priority
                     │     ├── tabpanel[draft]        → DraftSurface  (always mounted, hidden when inactive)
                     │     ├── tabpanel[tree-drafter] → TreeDrafterSurface
                     │     ├── tabpanel[game-plan]    → GamePlanSurface
                     │     ├── tabpanel[opponents]    → OpponentsSurface
                     │     └── tabpanel[ban-priority] → BanPrioritySurface
                     │
                     └── (each *Surface = extracted chrome-free content from existing page)

Flat routes (thin shells, existing behavior preserved):
  /draft         → DraftPage (renders DraftSurface + page chrome)
  /tree-drafter  → TreeDrafterPage (renders TreeDrafterSurface + page chrome)
  /game-plan     → GamePlanPage (renders GamePlanSurface + page chrome)
  /opponents     → OpponentsPage (renders OpponentsSurface + page chrome)

get_team_growth_state() [#[server] in war_room.rs]
    ↓
AuthSession → user.id
    ↓
db::get_user_team_id(db, user_id) → Some(team_id) or None → Ok(default)
    ↓
db::get_team_growth_counts(db, team_id)  ← ALREADY BUILT in db.rs:4164
    ↓
TeamGrowthState::from_counts(counts)     ← ALREADY BUILT in growth.rs:134
    ↓
Ok(TeamGrowthState)   [never Err for no-team; rule #44]
```

### Recommended Project Structure

```
src/
├── pages/
│   ├── rooms/
│   │   ├── mod.rs              # add: pub mod war_room;
│   │   ├── solo_journey.rs     # unchanged
│   │   └── war_room.rs         # NEW: WarRoomRoom + get_team_growth_state
│   ├── draft.rs                # extract DraftSurface, BanPrioritySurface; keep DraftPage thin shell
│   ├── tree_drafter.rs         # extract TreeDrafterSurface; keep TreeDrafterPage thin shell
│   ├── game_plan.rs            # extract GamePlanSurface; keep GamePlanPage thin shell
│   └── opponents.rs            # extract OpponentsSurface; keep OpponentsPage thin shell
├── components/
│   └── room/
│       ├── mod.rs              # unchanged (RoomSurface, SurfaceTab already exported)
│       ├── room_surface.rs     # MODIFY: keep-mounted contract (D-06/07/09)
│       └── parallax.rs         # MODIFY: add AtomicBool listener guard (D-08)
└── app.rs                      # add /war-room Route; flat routes stay unchanged
public/img/rooms/
└── war-room/
    ├── demacia/
    │   ├── plane-sky.webp      # placeholder gradient
    │   ├── plane-mid.webp
    │   ├── plane-near.webp
    │   ├── overlay-tactics.webp
    │   ├── overlay-intel.webp
    │   └── overlay-command.webp
    └── pandemonium/
        └── (same 6 files)
e2e/tests/
└── pages.spec.ts               # add /war-room to AUTHED_PAGES; add canary + round-trip tests
```

### Pattern 1: RoomSurface Keep-Mounted Contract (D-06/07/09)

**Current state:** `room_surface.rs:137` uses `<Show when=move || active_idx.get() == idx>` per panel — this unmounts/remounts the child on every tab switch, destroying surface state.

**The single `ChildrenFn` problem:** The existing API calls `children()` once per panel inside the `<Show>` closure. All panels receive the same `ChildrenFn` output and must branch internally on the active tab context. Keep-mounted distinct panels need each surface to render its **own** independent content — they cannot share one `ChildrenFn`.

**New API contract:** Replace `children: ChildrenFn` with `panels: Vec<(SurfaceTab, Box<dyn Fn() -> AnyView>)>` (or an equivalent `ViewFn` type). Each entry is a tab descriptor paired with a render function that returns that tab's complete content.

```rust
// src/components/room/room_surface.rs — new signature (D-09)

/// A render function for a single tab panel. `Box<dyn Fn() -> AnyView>` is
/// the simplest type that works across SSR and WASM without lifetime issues.
pub type PanelFn = Box<dyn Fn() -> AnyView>;

#[component]
pub fn RoomSurface(
    /// Ordered list of surfaces, each paired with its panel content factory.
    /// The factory is called ONCE at mount (all panels mount eagerly per D-08).
    panels: Vec<(SurfaceTab, PanelFn)>,
) -> impl IntoView {
    // … same tab-index resolution and URL sync as today …

    // Render ALL panels at mount; hide inactive ones via `hidden` attribute (D-07).
    // `hidden` removes the panel from a11y/tab order while preserving DOM mount.
    panels.into_iter().enumerate().map(|(idx, (tab, render_fn))| {
        let id = tab.id;
        let tab_label_id = format!("tab-{id}");
        let panel_id = format!("panel-{id}");
        view! {
            <div
                id=panel_id
                role="tabpanel"
                aria-labelledby=tab_label_id
                // D-07: `hidden` removes from a11y tree and tab order;
                // `display:none` from browser's built-in `hidden` handling.
                attr:hidden=move || if active_idx.get() != idx { Some("") } else { None }
            >
                {render_fn()}   // called once; result stays mounted
            </div>
        }
    }).collect_view()
}
```

**WASM/SSR note on `attr:hidden`:** Setting `attr:hidden=move || Some("")` on SSR outputs `hidden=""` on inactive panels, and `None` outputs nothing on the active panel. Leptos serializes `Option<&str>` as present/absent attribute correctly. This is structurally symmetric (always N panels in the DOM) so hydration is safe. [ASSUMED — `attr:hidden` with reactive `Option<&str>` works in Leptos 0.8; verify against Leptos docs at implementation time]

**Callers (WarSurfaces):**

```rust
// WarSurfaces calls RoomSurface with per-tab render closures:
RoomSurface(panels=vec![
    (SurfaceTab { id: "draft", label: "Draft" },
     Box::new(move || view! { <DraftSurface region=region.clone() /> }.into_any())),
    (SurfaceTab { id: "tree-drafter", label: "Tree Drafter" },
     Box::new(move || view! { <TreeDrafterSurface region=region.clone() /> }.into_any())),
    // … etc
])
```

**Unit test update:** The existing unit tests for `resolve_active_tab` are pure-function tests and require no change. The dummy-surface render test (which calls `RoomSurface` with the old `ChildrenFn`) must be replaced with a test that passes two dummy `PanelFn` closures and verifies the tablist renders correctly. [VERIFIED: codebase — `room_surface.rs:156-185` has `resolve_active_tab` unit tests that will survive; only the component-level render test needs updating]

### Pattern 2: Surface Extraction Boundaries

**Chrome-free means:** remove `class="canvas-grain bg-base min-h-screen"` wrappers, page-level `<h1>`/`<div class="max-w-... px-... py-...">` outermost wrappers, and standalone `use_context::<InitialTheme>()` calls at the page top. Keep all business logic, server fn calls, signals, Resources, sub-components, and internal reactive state.

**Extraction template (mirror `SoloDashboardPage` → `SoloSurfaces`):**

```rust
// BEFORE (thin shell keeps):
#[component]
pub fn DraftPage() -> impl IntoView {
    let region = use_context::<InitialTheme>().unwrap_or_default().0;  // ← stays in shell
    view! {
        <div class="canvas-grain bg-base min-h-screen">    // ← stays in shell
            <DraftSurface region=region />
        </div>
    }
}

// NEW extracted component:
#[component]
pub fn DraftSurface(region: String) -> impl IntoView {
    // all existing DraftPage content from after the outer <div class="canvas-grain ...">
    // onward, receiving `region` as a prop
}
```

**Per-file extraction summary:**

| File | Lines | Surface name | Chrome to remove | Complication |
|------|-------|--------------|------------------|--------------|
| `draft.rs` | 4,525 | `DraftSurface` | `canvas-grain bg-base min-h-screen` outer div (`:1579`), `DraftPage` fn wrapper | ~3000 lines of actual content below the page wrapper; ban-priority editor also extracts |
| `draft.rs` (ban panel) | ~200 lines of signals+view | `BanPrioritySurface` | Extract ban panel state (`:1012-1019`) and view from `DraftPage`; keep `DraftSurface`'s `banned_champions` (`:1270`) in place — distinct concept (D-05) |
| `tree_drafter.rs` | 1,679 | `TreeDrafterSurface` | Outer `bg-base min-h-screen` wrapper + `TreeDrafterPage` fn wrapper |  |
| `game_plan.rs` | 1,964 | `GamePlanSurface` | Outer wrapper + `GamePlanPage` fn wrapper |  |
| `opponents.rs` | 1,340 | `OpponentsSurface` | Outer wrapper + `OpponentsPage` fn wrapper |  |

**draft.rs: ban-priority boundary (D-04/D-05):**

The ban-priority *editor* is distinct from in-draft ban tracking:

- **BanPrioritySurface owns:** `ban_priorities` Resource (`:1013`), `ban_panel_open` signal (`:1014`), `editing_bans` (`:1015`), `ban_edit_list` (`:1016`), `ban_new_champ` (`:1017`), `ban_new_reason` (`:1018`), `ban_status` (`:1019`), `get_ban_priorities` server fn (`:188`), `save_ban_priorities` server fn (`:214`), and the view rendering the ban editor panel. These signals and their view sections move entirely to `BanPrioritySurface`.

- **DraftSurface keeps:** `banned_champions` closure (`:1270`) derived from `draft_slots` — this is the live-draft ban tracking for slot validation and champion picker filtering. This is a local computed function, not a Resource. It does NOT interact with the `ban_priority` DB table. (D-05)

**Region prop threading:** Each `*Surface` receives `region: String` as a prop (read once at `WarRoomRoom` top, passed through `WarSurfaces → *Surface`). `draft.rs`'s `DraftPage` already reads `InitialTheme` at its top level (`:930`) — after extraction the shell reads it and passes it down.

### Pattern 3: `get_team_growth_state` Server Fn

**All infrastructure is already built in Phase 26:** [VERIFIED: codebase — `growth.rs:134` has `TeamGrowthState::from_counts`; `db.rs:4164` has `get_team_growth_counts`]

The server fn is a thin wrapper:

```rust
// src/pages/rooms/war_room.rs
#[server]
pub async fn get_team_growth_state() -> Result<TeamGrowthState, ServerFnError> {
    use crate::models::growth::TeamGrowthState;
    use crate::server::{auth::AuthSession, db};
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&surreal, &user.id).await? {
        Some(id) => id,
        None => return Ok(TeamGrowthState::default()), // no team → all Seed (rule #44)
    };
    let counts = db::get_team_growth_counts(&surreal, &team_id)
        .await
        .unwrap_or_default();
    Ok(TeamGrowthState::from_counts(counts))
}
```

**Mirror:** `get_growth_state` in `solo_dashboard.rs:71` — identical structure, team-scoped. [VERIFIED: codebase — `solo_dashboard.rs:71-90`]

**No-team path:** `return Ok(TeamGrowthState::default())` renders all tier-0 overlays — the room starts empty. Never returns `Err` for a missing team. [VERIFIED: rule #44 + `growth.rs:150-155` `impl Default`]

### Pattern 4: WarRoomRoom Composition

```rust
// src/pages/rooms/war_room.rs  (target ≤ 200 lines)
#[component]
pub fn WarRoomRoom() -> impl IntoView {
    // Rule #58: read region ONCE here, never inside reactive primitives or children
    let region = use_context::<InitialTheme>().unwrap_or_default().0;

    let growth = Resource::new(|| (), |_| get_team_growth_state());
    let tactics_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.tactics_tier).unwrap_or(0)
    });
    let intel_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.intel_tier).unwrap_or(0)
    });
    let command_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.command_tier).unwrap_or(0)
    });

    let dir = format!("/img/rooms/war-room/{region}");
    let surfaces_region = region.clone();

    view! {
        <AtmosphereRoot
            class="min-h-screen".to_string()
            tactics_tier=tactics_tier
            intel_tier=intel_tier
            command_tier=command_tier
        >
            <Plane src=format!("{dir}/plane-sky.webp") depth=6 z=1 />
            <Plane src=format!("{dir}/plane-mid.webp") depth=16 z=2 />
            <Plane src=format!("{dir}/plane-near.webp") depth=32 z=3 />
            <GrowthOverlay axis="tactics" src=format!("{dir}/overlay-tactics.webp") z=4 />
            <GrowthOverlay axis="intel" src=format!("{dir}/overlay-intel.webp") z=5 />
            <GrowthOverlay axis="command" src=format!("{dir}/overlay-command.webp") z=6 />
            <div class="fixed inset-0 pointer-events-none" aria-hidden="true"
                 style="z-index: 8; background: radial-gradient(ellipse at center, transparent 0%, var(--color-base) 95%);"></div>
            <div class="relative z-10 px-8 py-6" data-feedback-label="War Room">
                <WarSurfaces region=surfaces_region />
            </div>
        </AtmosphereRoot>
    }
}
```

[VERIFIED: pattern — mirrors `solo_journey.rs:15-81` exactly, axis names confirmed in `parallax.rs:37-39`]

### Pattern 5: Mousemove Accumulation Guard (AtomicBool)

**Problem:** Leptos #3665 — `on_cleanup` may not fire atomically on first navigation under the same router parent. If a user navigates War Room → Solo Journey → War Room, the first War Room's cleanup may not fire until Solo Journey unmounts, leaving two simultaneous `mousemove` listeners. This doubles parallax amplitude. [VERIFIED: PITFALLS.md Pitfall 3, research SUMMARY.md §111]

**Current `AtmosphereRoot` state:** The `on_cleanup` at `parallax.rs:210` already correctly removes the listener — the guard is insurance against the edge case.

**Guard mechanism:**

```rust
// In parallax.rs, before the #[cfg(feature = "hydrate")] block:
use std::sync::atomic::{AtomicBool, Ordering};
static MOUSEMOVE_ACTIVE: AtomicBool = AtomicBool::new(false);
```

```rust
// Inside the Effect::new, before calling add_event_listener:
if MOUSEMOVE_ACTIVE.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
    // Another AtmosphereRoot instance already registered the listener.
    // Skip to avoid doubling. The existing on_cleanup will clear the flag
    // when the other room unmounts.
    return;
}
```

```rust
// In on_cleanup:
MOUSEMOVE_ACTIVE.store(false, Ordering::SeqCst);
// … existing listener.update_value removal …
```

**Why `AtomicBool` is sound in WASM:** WASM is single-threaded. `AtomicBool` operations are sequentially consistent in the single-thread model. `static` works in WASM because there is only one linear memory. [VERIFIED: research SUMMARY.md; `std::sync::atomic` is available in `wasm32-unknown-unknown`]

**Verification:** `getEventListeners(window).mousemove.length === 1` after `/journey` → `/war-room` → `/journey` round-trip. This requires a Playwright `evaluate()` call in the e2e test. [ASSUMED — Chrome DevTools `getEventListeners` API is available in Playwright's Chromium context; verify at test-writing time]

### Anti-Patterns to Avoid

- **`use_context::<InitialTheme>()` in any `*Surface` component.** Region must come as a `region: String` prop from `WarRoomRoom`. Violating this causes SSR/hydrate structural mismatch panic at `html/mod.rs:217` on non-default region (Phase 18.2 incident). [VERIFIED: CLAUDE.md #58, `leptos-patterns.md` #58]
- **Sub-routes inside the War Room.** `<Route path="/war-room/draft">` would remount `AtmosphereRoot` on every tab switch, destroying parallax continuity. Tab switching is `RwSignal<usize>` + CSS `hidden` only. [VERIFIED: architecture decision, CONTEXT.md D-02]
- **`bg-surface/50` on panels over parallax art.** Semi-transparent backgrounds fail WCAG 4.5:1 contrast at worst-case parallax position. Use `bg-surface` (fully opaque) inside `*Surface` components. [VERIFIED: PITFALLS.md Pitfall 6]
- **`growth_resource.refetch()` in mutation success handlers.** Growth overlay animation must not fire mid-task. Growth is only evaluated at room mount. [VERIFIED: PITFALLS.md Pitfall 11]
- **Merging `banned_champions` from DraftSurface into BanPrioritySurface.** D-05 explicitly separates these: `banned_champions` (`:1270`) is a local closure deriving from `draft_slots` signals; the ban-priority editor reads from the `ban_priority` DB table. Different data, different signals, different scope.
- **CSS-hiding the inactive region's asset set.** Only render the active region's `<img>` elements. Never add both regions' planes to the DOM and toggle `display:none`. [VERIFIED: parallax.rs doc comment line 163, PITFALLS.md Anti-Pattern 2]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Team growth tier derivation | Custom tier logic | `TeamGrowthState::from_counts` (`growth.rs:134`) | Already unit-tested with thresholds |
| Team-scoped DB count query | New query | `get_team_growth_counts` (`db.rs:4164`) | Already built in Phase 26 |
| Ban priority persistence | New DB layer | `get_ban_priorities`/`set_ban_priorities` (`db.rs:2274/2287`) | Already built |
| User team ID lookup | Inline query | `db::get_user_team_id` (`db.rs:572`) | Already handles stripping, error types |
| ARIA tablist structure | Hand-rolled tab logic | `RoomSurface` (modified) | ARIA contracts already implemented |
| Parallax CSS-var bridge | New mousemove handler | `AtmosphereRoot` (existing) | All 12 axis props already wired; just pass tier signals |
| Placeholder WebP assets | Complex art | `convert -size WxH gradient:...` (ImageMagick) | Matches solo-journey placeholder approach |

**Key insight:** The entire data layer for this phase was pre-built in Phase 26. Phase 27 is a composition and refactor phase, not a data layer phase. The only new code is: `WarRoomRoom`, `WarSurfaces`, `get_team_growth_state`, the `RoomSurface` API change, five `*Surface` extractions, the `AtomicBool` guard, and placeholder assets.

---

## Common Pitfalls

### Pitfall 1: Hydration Panic in pandemonium Theme (Phase 18.2 Redux)

**What goes wrong:** Any `*Surface` component that calls `use_context::<InitialTheme>()` internally, or any reactive closure in `WarRoomRoom` that reads `region`, causes a DOM structure mismatch between SSR and WASM. Tachys panics at `html/mod.rs:217` with `Option::unwrap()`. All WASM event handlers silently die — parallax stops, tabs stop working. The panic **only manifests in pandemonium** (non-default region), not in demacia. This makes it invisible in dev if you test only with the default theme.

**How to avoid:** Read `use_context::<InitialTheme>()` ONCE at `WarRoomRoom` top. Pass `region: String` prop to `WarSurfaces` and from there to each `*Surface`. NEVER read the context again inside any `*Surface`, signal, effect, or view branch. [VERIFIED: CLAUDE.md #58, phase 18.2 post-mortem]

**Warning signs:**
- Any `use_context::<InitialTheme>()` call outside `WarRoomRoom::` 
- A `region`-dependent `if/match` inside `view!` that conditionally includes/excludes elements
- Test passes in demacia but panics in pandemonium

### Pitfall 2: `<Show>` Left in RoomSurface After Refactor

**What goes wrong:** Leaving even one `<Show when=active>` in `RoomSurface` means that panel's content unmounts and remounts on every tab switch. State is lost, Resources re-fetch, and AtmosphereRoot may see unnecessary reactive updates.

**How to avoid:** Remove ALL `<Show>` from panel rendering. Replace entirely with `attr:hidden=move || if active_idx.get() != idx { Some("") } else { None }`. Verify by watching the browser's Elements panel: on tab switch, no element should be added/removed from inside `.room-surface` — only `hidden` attribute should toggle.

### Pitfall 3: DraftSurface `auth_user` Resource Conflict

**What goes wrong:** `DraftPage` (`:935-945`) creates its own `auth_user` Resource. `WarSurfaces` mounts all five surfaces eagerly (D-08), meaning five separate `get_current_user()` fetches fire on War Room entry. This is acceptable per D-08, but the planner should note it. Additionally, `DraftPage`'s auth-redirect Effect fires inside `WarRoomRoom` — the redirect to `/auth/login` should still work because it uses `window.location().set_href()` which is a hard nav.

**How to avoid:** Each `*Surface` keeps its own auth Resource and redirect Effect. No consolidation needed — D-08 accepts the eager fetch cost. Do not attempt to share a single auth Resource across five surfaces; the lifecycle coupling would be complex.

### Pitfall 4: `BanPrioritySurface` Missing `use_champions` for Autocomplete

**What goes wrong:** The ban-priority editor (draft.rs ~1012-1019 and its view section) uses a champion autocomplete input (`ban_new_champ`). The champion list comes from the `get_champions()` server fn which `DraftPage` already fetches. After extracting `BanPrioritySurface`, it will need its own `get_champions()` Resource for the autocomplete, unless the design chooses a plain text input. Check the ban-priority view code to determine whether champion autocomplete is used.

**How to avoid:** Confirm whether the ban-priority view uses `<ChampionAutocomplete>` or a plain `<input>`. If it uses autocomplete, `BanPrioritySurface` must declare its own `champions_resource`. If it's plain text, no issue.

### Pitfall 5: `WarSurfaces` Passing `region` into `DraftSurface` Creates Double-Clone

**What goes wrong:** `WarSurfaces` receives `region: String`. It must pass `region.clone()` to five `*Surface` components. Since `RoomSurface`'s `PanelFn` closures capture `region` by move, each closure needs its own clone. This is mechanical but easy to mis-count.

**How to avoid:** Clone upfront for each closure before building the `panels` vec:
```rust
let r1 = region.clone(); // for DraftSurface
let r2 = region.clone(); // for TreeDrafterSurface
// … etc
let panels = vec![
    (SurfaceTab { id: "draft", label: "Draft" }, Box::new(move || view! { <DraftSurface region=r1.clone() /> }.into_any()) as PanelFn),
    // …
];
```

### Pitfall 6: `draft.rs` `DraftPage` Already Reads `InitialTheme` — Not a Problem

**What happens:** `DraftPage` (`:930`) reads `use_context::<InitialTheme>().unwrap_or_default().0` at the top. After extraction, `DraftPage` becomes a thin shell that reads `InitialTheme` and passes it to `DraftSurface`. `DraftSurface` receives it as `region: String` prop. No hydration issue because the thin shell is the top-level component — it reads context at component entry, not inside a reactive primitive.

**Watch out:** If any sub-component of `DraftSurface` currently reads `use_context::<InitialTheme>()` directly (grep for this), those reads must be replaced with the prop-threaded `region`.

---

## Code Examples

### Existing: `get_growth_state` (solo) — template to mirror

```rust
// Source: src/pages/solo_dashboard.rs:71
#[server]
pub async fn get_growth_state() -> Result<crate::models::growth::GrowthState, ServerFnError> {
    use crate::models::growth::GrowthState;
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let counts = db::get_growth_counts(&surreal, &user.id)
        .await
        .unwrap_or_default();
    Ok(GrowthState::from_counts(counts))
}
```

### Existing: `SoloJourneyRoom` — template to mirror for `WarRoomRoom`

```rust
// Source: src/pages/rooms/solo_journey.rs:15
#[component]
pub fn SoloJourneyRoom() -> impl IntoView {
    let region = use_context::<InitialTheme>().unwrap_or_default().0;
    let growth = Resource::new(|| (), |_| get_growth_state());
    let lib_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.library_tier).unwrap_or(0)
    });
    // … (axis signals follow same pattern)
    let dir = format!("/img/rooms/solo-journey/{region}");
    view! {
        <AtmosphereRoot lib_tier=lib_tier armory_tier=armory_tier banner_tier=banner_tier>
            <Plane src=format!("{dir}/plane-sky.webp") depth=6 z=1 />
            // …
        </AtmosphereRoot>
    }
}
```

### Existing: `get_team_growth_counts` — already in db.rs

```rust
// Source: src/server/db.rs:4164
pub async fn get_team_growth_counts(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<TeamGrowthCounts> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut result = db.query(
        "SELECT count() as n FROM draft WHERE team = type::record('team', $team_key) GROUP ALL; \
         SELECT count() as n FROM draft_tree WHERE team = type::record('team', $team_key) GROUP ALL; \
         SELECT count() as n FROM opponent WHERE team = type::record('team', $team_key) GROUP ALL; \
         SELECT count() as n FROM ban_priority WHERE team = type::record('team', $team_key) GROUP ALL; \
         SELECT count() as n FROM post_game_learning WHERE team = type::record('team', $team_key) GROUP ALL; \
         SELECT count() as n FROM team_note WHERE team = type::record('team', $team_key) GROUP ALL",
    ).bind(("team_key", team_key)).await?;
    // … take(0..5) pattern …
}
```

### Existing: `AtmosphereRoot` axis props — already supports War Room axes

```rust
// Source: src/components/room/parallax.rs:37-39
#[prop(optional, into)] tactics_tier: Signal<u8>,
#[prop(optional, into)] intel_tier: Signal<u8>,
#[prop(optional, into)] command_tier: Signal<u8>,
// and data-*-tier written at parallax.rs:242-244
```

### Existing: `RoomSurface` unit tests — pure-function tests survive the API change

```rust
// Source: src/components/room/room_surface.rs:156
// These test resolve_active_tab() which is unchanged by the API refactor.
#[test]
fn resolve_known_tab_returns_its_index() { … }
#[test]
fn resolve_empty_falls_back_to_first() { … }
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `<Show>` panel mounting in `RoomSurface` | Keep-mounted `attr:hidden` panels (D-06) | Phase 27 | State persists across tab switches; no atmosphere flash |
| Single `ChildrenFn` per RoomSurface | Per-tab `PanelFn` vector (D-09) | Phase 27 | Each surface renders its own independent content |
| `DraftPage` / `GamePlanPage` etc. as monoliths | `*Surface` components + thin page shells (D-01) | Phase 27 | Single source of truth; War Room and flat routes share code |

**Deprecated/outdated:**
- `<Show when=active>` in `RoomSurface` panel rendering: replaced by `attr:hidden` on always-mounted divs.
- `children: ChildrenFn` in `RoomSurface`: replaced by `panels: Vec<(SurfaceTab, PanelFn)>`.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `attr:hidden=move \|\| Some("")` on inactive panels is structurally symmetric in SSR and WASM (N panels always rendered, only hidden attribute toggles) | Pattern 1: RoomSurface Keep-Mounted Contract | If Leptos emits different DOM structure for `hidden` vs not-hidden, hydration could panic. Verify with `cargo check --features hydrate --target wasm32-unknown-unknown` and a quick render test before committing the pattern. |
| A2 | Chrome DevTools `getEventListeners(window).mousemove.length` is accessible via Playwright `page.evaluate()` | Pitfall: mousemove guard verification | If not accessible in Playwright's Chromium context, the e2e test must use an alternative approach (e.g. expose a `window.__raf_count` debug counter) |
| A3 | `BanPrioritySurface` does not require champion autocomplete (i.e. the ban-priority editor uses plain text input, not `<ChampionAutocomplete>`) | Pattern 2: Surface Extraction Boundaries | If autocomplete is used, `BanPrioritySurface` needs its own `get_champions()` Resource — one additional server fn call on room entry |
| A4 | No `*Surface` source file currently calls `use_context::<InitialTheme>()` in a sub-component | Pitfall 1: Hydration Panic | If any does, that call must be refactored to accept a `region: String` prop. Verify with: `grep -rn "InitialTheme" src/pages/draft.rs src/pages/game_plan.rs src/pages/opponents.rs src/pages/tree_drafter.rs` |

---

## Open Questions (RESOLVED)

1. **`BanPrioritySurface`: champion autocomplete?** — **RESOLVED.**
   - What we know: The ban-priority editor at `draft.rs:1012-1019` has `ban_new_champ: RwSignal<String>`. The editor adds champion names to the ban list.
   - **RESOLVED:** The orchestrator grepped `draft.rs` directly — the ban-priority editor uses `<ChampionPicker>` (grid; `use crate::components::champion_picker::ChampionPicker`, draft.rs:2 / picker usage draft.rs:2851), NOT `<ChampionAutocomplete>`. `BanPrioritySurface` brings `ChampionPicker`; no `get_champions()` autocomplete Resource is required. Confirmed in `27-PATTERNS.md`.

2. **`AtomicBool` for `static` in WASM: `#[cfg(target_arch = "wasm32")]` scope needed?** — **RESOLVED.**
   - What we know: `static MOUSEMOVE_ACTIVE: AtomicBool` at module level is valid Rust. WASM is single-threaded.
   - **RESOLVED:** Declare the `static` unconditionally (SSR-harmless) and guard the `compare_exchange` inside `#[cfg(feature = "hydrate")]`. No special `wasm32` cfg needed. Plan 27-02 follows this verbatim.

---

## Environment Availability

> This phase is purely code/config changes with new placeholder WebP assets. No external services required beyond ImageMagick for placeholder generation.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| ImageMagick (`convert`) | Placeholder WebP generation | Likely ✓ | Check at build time | Hand-craft 1×1 gradient WebP; or re-use solo-journey placeholders temporarily |
| Playwright + running dev server | e2e tests | ✓ (existing) | npx playwright | — |

**Missing dependencies with no fallback:** none

---

## Validation Architecture

> `nyquist_validation: true` — include full test mapping.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Playwright (e2e) + `cargo test --features ssr --lib` (unit) |
| Config file | `e2e/playwright.config.ts` |
| Quick run command | `cd e2e && npx playwright test pages.spec.ts -g "war-room"` |
| Full suite command | `cd e2e && npx playwright test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| ROOM-04-SC1 | Navigate to `/war-room`, see parallax atmosphere; tab switch does NOT flash/remount atmosphere | e2e smoke | `cd e2e && npx playwright test pages.spec.ts -g "/war-room"` | ❌ Wave 0: add to `pages.spec.ts` |
| ROOM-04-SC2 | Tab bar uses `role="tablist"` / `role="tab"` / `aria-selected` / `aria-controls` | e2e ARIA | `cd e2e && npx playwright test war-room.spec.ts -g "aria tablist"` | ❌ Wave 0: create `war-room.spec.ts` |
| ROOM-04-SC3 | Team with saved drafts+opponents sees `data-tactics-tier` / `data-intel-tier` / `data-command-tier` ≥ 1; fresh team sees 0 | e2e data-attribute | `cd e2e && npx playwright test war-room.spec.ts -g "growth tiers"` | ❌ Wave 0 |
| ROOM-04-SC4 | `/draft` → `/war-room` → back: `getEventListeners(window).mousemove.length === 1` | e2e round-trip | `cd e2e && npx playwright test war-room.spec.ts -g "mousemove guard"` | ❌ Wave 0 |
| ROOM-04-SC5 | `/war-room` hydration-no-panic in **demacia** theme | e2e canary | `cd e2e && npx playwright test war-room.spec.ts -g "canary demacia"` | ❌ Wave 0 |
| ROOM-04-SC5 | `/war-room` hydration-no-panic in **pandemonium** theme | e2e canary | `cd e2e && npx playwright test war-room.spec.ts -g "canary pandemonium"` | ❌ Wave 0 |
| ROOM-04-RoomSurface-unit | `resolve_active_tab` pure-fn behavior preserved after API change | unit | `cargo test --features ssr --lib room_surface` | ✅ exists (needs dummy-surface test update) |
| ROOM-04-growth-unit | `TeamGrowthState::from_counts` tier derivation | unit | `cargo test --features ssr --lib growth` | ✅ `growth.rs:228-282` |
| ROOM-04-flat-routes | `/draft`, `/tree-drafter`, `/game-plan`, `/opponents` still load without JS errors | e2e regression | `cd e2e && npx playwright test pages.spec.ts` | ✅ (existing; must pass after extraction) |

### Observable Validation Signals

The following DOM/runtime signals are the ground truth for each success criterion:

**SC1 (no atmosphere flash):** AtmosphereRoot's `#room-atmosphere-root` div must remain in the DOM on tab switch. Verify in Playwright: click tab, assert `#room-atmosphere-root` count is still 1, assert `--px` CSS custom property is still readable.

**SC2 (ARIA tablist):** Assert `role="tablist"` present; assert each tab button has `role="tab"` and `aria-selected="true/false"`; assert `aria-controls` points to a valid `id`.

**SC3 (growth overlay tiers):** Seed test data with ≥ 5 drafts (tactics tier-2 threshold), then navigate to `/war-room`. Assert `data-tactics-tier` attribute on `#room-atmosphere-root` equals `"2"`. Verify with a fresh user (no team) that all three attributes equal `"0"`.

**SC4 (mousemove guard):** Navigate `/draft` → `/war-room` → `/journey` → `/war-room`. In Playwright:
```js
const count = await page.evaluate(() =>
  (window as any).getEventListeners?.(window)?.mousemove?.length ?? -1
);
expect(count).toBe(1);
```
(If `getEventListeners` is unavailable in Playwright, add `window.__mousemove_count` debug counter in `#[cfg(debug_assertions)]` block in `parallax.rs`.)

**SC5 (hydration-no-panic):** Register a `pageerror` listener before navigating to `/war-room`. After `waitForLoadState('networkidle')`, click a tab to trigger WASM interaction. Assert zero `pageerror` events AND zero console errors containing `"panicked"` or `"html/mod.rs"`. Run with pandemonium theme set in localStorage: `await page.evaluate(() => localStorage.setItem('theme', 'pandemonium'))` before navigating.

### Sampling Rate

- **Per task commit:** `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown`
- **Per wave merge:** `cargo test --features ssr --lib && cd e2e && npx playwright test pages.spec.ts`
- **Phase gate:** Full suite green (`cd e2e && npx playwright test`) before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `e2e/tests/war-room.spec.ts` — covers SC2, SC3, SC4, SC5 (canary demacia + pandemonium)
- [ ] Add `/war-room` to `AUTHED_PAGES` in `e2e/tests/pages.spec.ts` — covers SC1
- [ ] `e2e/tests/war-room.spec.ts` — Tier-2 seed fixture (drafts + opponents in DB before test run); may require a `before` hook seeding via direct server fn call or a dedicated seed endpoint

---

## Security Domain

> Reviewing applicable ASVS categories for this phase.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes — all 5 server fns require auth | `leptos_axum::extract::<AuthSession>()` + `auth.user.ok_or_else(…)` |
| V3 Session Management | no — inherited from existing auth layer | — |
| V4 Access Control | yes — team-scoped data | `get_user_team_id` gates all team queries; no-team → empty/default |
| V5 Input Validation | partial — `BanPrioritySurface` JSON deserializes `Vec<BanPriority>` | `serde_json::from_str` with typed struct (rejects malformed input); existing pattern from `save_ban_priorities` |
| V6 Cryptography | no | — |

### Known Threat Patterns for this Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Region path traversal in asset URLs | Tampering | `region` from server-set `InitialTheme` context; never from URL param |
| Team data leakage to teamless user | Information Disclosure | `get_user_team_id` → `None` → `Ok(TeamGrowthState::default())` / `Ok(Vec::new())` |
| Ban priority data for wrong team | Tampering | All ban priority fns resolve team via `get_user_team_id(db, user.id)` — user cannot specify team_id |

---

## Sources

### Primary (HIGH confidence)
- `src/components/room/room_surface.rs` — current API (lines 1-185), `<Show>` panel at :137, unit tests at :156
- `src/components/room/parallax.rs` — `AtmosphereRoot` props (lines 25-50), axis props (`:37-39`), `data-*-tier` emission (`:242-244`), `on_cleanup` pattern (`:210-231`), `ATMOSPHERE_ROOT_ID` (`:19`)
- `src/pages/rooms/solo_journey.rs` — region-read-once (`:18`), Resource pattern (`:21-42`), asset dir (`:46`), plane/overlay/scrim composition (`:49-79`); the exact template for `WarRoomRoom`
- `src/pages/solo_dashboard.rs` — `get_growth_state` server fn pattern (`:71-90`); `SoloSurfaces` extraction precedent (`:282-490`); `SoloDashboardPage` thin shell (`:493-503`)
- `src/models/growth.rs` — `TeamGrowthCounts` (`:109-116`), `TeamGrowthState` (`:121-129`), `from_counts` (`:134-147`), `Default` (`:150-155`), team unit tests (`:228-282`)
- `src/server/db.rs:4164` — `get_team_growth_counts` fully implemented; confirmed batched 6-statement query
- `src/server/db.rs:2274/2287` — `get_ban_priorities`/`set_ban_priorities` confirmed present
- `src/server/db.rs:572` — `get_user_team_id` confirmed
- `src/pages/draft.rs:930` — `DraftPage` region read (`:930`), outer div chrome (`:1579`), ban-priority signals (`:1012-1019`), server fns (`:188-239`), `banned_champions` closure (`:1270`)
- `src/app.rs:145-156` — Route table; `/journey` route at `:156` (Solo Journey precedent); flat routes to preserve
- `e2e/tests/pages.spec.ts` — `AUTHED_PAGES` array; filter pattern for 404 errors; test structure
- `.planning/phases/27-war-room/27-CONTEXT.md` — all locked decisions D-01 through D-11
- `.planning/phases/26-room-system-foundations/26-CONTEXT.md` — Phase 26 contract
- `.planning/research/SUMMARY.md` — AtmosphereRoot axis names, build sequence, Phase C deliverables
- `.planning/research/ARCHITECTURE.md` — component contracts, data flow diagrams, `get_team_growth_state` code pattern
- `.planning/research/PITFALLS.md` — Pitfall 3 (mousemove accumulation, AtomicBool), Pitfall 6 (contrast), CLAUDE.md #58 rule

### Secondary (MEDIUM confidence)
- [Leptos #3665](https://github.com/leptos-rs/leptos/issues/3665) — `on_cleanup` timing edge case motivating the `AtomicBool` guard [CITED: cited in PITFALLS.md; not re-verified this session]
- CLAUDE.md #58, `.claude/rules/leptos-patterns.md` #58 — region-read-once rule + Phase 18.2 root cause

---

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — zero new deps; all infrastructure confirmed in codebase
- Architecture: HIGH — all patterns have working precedents in the codebase (`solo_journey.rs`, `solo_dashboard.rs`)
- RoomSurface API change: MEDIUM-HIGH — `attr:hidden` behavior on SSR is assumed symmetric; verify with compile check
- Mousemove guard: HIGH — `AtomicBool` pattern is well-established; single assumption is Playwright `getEventListeners` API
- Surface extraction: HIGH (pattern) / MEDIUM (draft.rs complexity) — `draft.rs` is 4,525 lines; boundary is clear but implementation is risk-heavy

**Research date:** 2026-06-05
**Valid until:** 2026-07-05 (stable stack; no fast-moving deps)
