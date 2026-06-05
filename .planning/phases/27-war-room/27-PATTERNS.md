# Phase 27: War Room - Pattern Map

**Mapped:** 2026-06-05
**Files analyzed:** 13 new/modified files
**Analogs found:** 13 / 13

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/pages/rooms/war_room.rs` | page (room) | request-response | `src/pages/rooms/solo_journey.rs` | exact |
| `src/pages/rooms/mod.rs` | config | — | `src/pages/rooms/mod.rs` (self) | exact |
| `src/pages/draft.rs` (extraction) | page + component | CRUD | `src/pages/solo_dashboard.rs` (SoloDashboardPage→SoloSurfaces) | exact |
| `src/pages/tree_drafter.rs` (extraction) | page + component | CRUD | `src/pages/solo_dashboard.rs` (SoloDashboardPage→SoloSurfaces) | role-match |
| `src/pages/game_plan.rs` (extraction) | page + component | CRUD | `src/pages/solo_dashboard.rs` (SoloDashboardPage→SoloSurfaces) | role-match |
| `src/pages/opponents.rs` (extraction) | page + component | CRUD | `src/pages/solo_dashboard.rs` (SoloDashboardPage→SoloSurfaces) | role-match |
| `BanPrioritySurface` (new, in draft.rs) | component | CRUD | `draft.rs` ban-priority signals + server fns | exact |
| `src/components/room/room_surface.rs` | component | event-driven | self (current `<Show>` API is the before-state) | self-refactor |
| `src/components/room/parallax.rs` | component | event-driven | self (AtomicBool addition) | self-refactor |
| `src/app.rs` | config/routing | — | self `:145-156` + `/journey` precedent `:156` | exact |
| `src/server/db.rs` (no change needed) | service | CRUD | `src/pages/solo_dashboard.rs:71` get_growth_state | exact |
| `e2e/tests/war-room.spec.ts` | test | — | `e2e/tests/room-solo-journey.spec.ts` | exact |
| `e2e/tests/pages.spec.ts` | test | — | self (AUTHED_PAGES array) | exact |

---

## Pattern Assignments

### `src/pages/rooms/war_room.rs` (page, request-response)

**Analog:** `src/pages/rooms/solo_journey.rs` (lines 1–81)

**Imports pattern** (lines 1–12):
```rust
use leptos::prelude::*;

use crate::app::InitialTheme;
use crate::components::room::{AtmosphereRoot, GrowthOverlay, Plane};
use crate::pages::solo_dashboard::{get_growth_state, SoloSurfaces};
```
For War Room:
```rust
use leptos::prelude::*;

use crate::app::InitialTheme;
use crate::components::room::{AtmosphereRoot, GrowthOverlay, Plane};
use crate::models::growth::TeamGrowthState;
// WarSurfaces defined in this file (no external import)
```

**Region-read-once + Resource + Signal::derive pattern** (solo_journey.rs lines 15–46):
```rust
#[component]
pub fn SoloJourneyRoom() -> impl IntoView {
    // Rule #58: read ONCE here — never inside reactive primitives or children
    let region = use_context::<InitialTheme>().unwrap_or_default().0;

    let growth = Resource::new(|| (), |_| get_growth_state());
    let lib_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.library_tier).unwrap_or(0)
    });
    let armory_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.armory_tier).unwrap_or(0)
    });
    let banner_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.banner_tier).unwrap_or(0)
    });

    let dir = format!("/img/rooms/solo-journey/{region}");
    let surfaces_region = region.clone();
```
Apply exactly to War Room with axes `tactics_tier`, `intel_tier`, `command_tier` and `get_team_growth_state()`.

**Plane/overlay/scrim composition pattern** (solo_journey.rs lines 49–81):
```rust
    view! {
        <AtmosphereRoot
            class="min-h-screen".to_string()
            lib_tier=lib_tier
            armory_tier=armory_tier
            banner_tier=banner_tier
        >
            <Plane src=format!("{dir}/plane-sky.webp") depth=6 z=1 />
            <Plane src=format!("{dir}/plane-mid.webp") depth=16 z=2 />
            <Plane src=format!("{dir}/plane-near.webp") depth=32 z=3 />
            <GrowthOverlay axis="banner" src=format!("{dir}/overlay-banner.webp") z=4 />
            <GrowthOverlay axis="lib"    src=format!("{dir}/overlay-lib.webp")    z=5 />
            <GrowthOverlay axis="armory" src=format!("{dir}/overlay-armory.webp") z=6 />
            <div
                class="fixed inset-0 pointer-events-none"
                style="z-index: 8; background: radial-gradient(ellipse at center, transparent 0%, var(--color-base) 95%);"
                aria-hidden="true"
            ></div>
            <div class="relative z-10 px-8 py-6" data-feedback-label="Solo Journey room">
                <SoloSurfaces region=surfaces_region />
            </div>
        </AtmosphereRoot>
    }
```
Apply verbatim with axis names `tactics`/`intel`/`command` and `data-feedback-label="War Room"`.

**`get_team_growth_state` server fn** — copy/adapt from `src/pages/solo_dashboard.rs` lines 71–90:
```rust
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
War Room version differences: team lookup via `db::get_user_team_id` (rule #44 no-team → `Ok(TeamGrowthState::default())`), then `db::get_team_growth_counts`, then `TeamGrowthState::from_counts`. Place in `war_room.rs` before the `#[component]`.

---

### `src/pages/rooms/mod.rs` (config)

**Analog:** self

Add `pub mod war_room;` after line 5 (`pub mod solo_journey;`).

---

### Extract-then-compose pattern: thin-shell + `*Surface` (four pages + DraftPage)

**Analog:** `src/pages/solo_dashboard.rs` — `SoloDashboardPage` (lines 493–503) is the thin shell; `SoloSurfaces` (lines 282–491) is the extracted surface.

**Thin shell pattern** (solo_dashboard.rs lines 493–503):
```rust
#[component]
pub fn SoloDashboardPage() -> impl IntoView {
    // Region read ONCE at page entry — passed as prop to all sub-components.
    let region = use_context::<InitialTheme>().unwrap_or_default().0;
    view! {
        <div class="canvas-grain bg-base min-h-screen px-8 py-6" data-feedback-label="Solo dashboard">
            <SoloSurfaces region=region />
        </div>
    }
}
```

**Surface component signature** (solo_dashboard.rs line 282–283):
```rust
#[component]
pub fn SoloSurfaces(region: String) -> impl IntoView {
```

**Auth-redirect + resource pattern inside surface** (solo_dashboard.rs lines 284–293):
```rust
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });
```

---

### `DraftSurface` extraction (src/pages/draft.rs)

**Chrome to remove:** outer `<div class="canvas-grain bg-base min-h-screen" ...>` wrapper (draft.rs line 1579) and the `DraftPage` function wrapper itself.

**New thin shell** (mirror solo_dashboard.rs:493–503 pattern):
```rust
#[component]
pub fn DraftPage() -> impl IntoView {
    let region = use_context::<InitialTheme>().unwrap_or_default().0;
    // theme.0 clone pattern from draft.rs:931-932
    view! {
        <div class="canvas-grain bg-base min-h-screen" data-feedback-label="Draft">
            <DraftSurface region=region />
        </div>
    }
}
```

**InitialTheme read in old DraftPage** (draft.rs lines 929–932):
```rust
pub fn DraftPage() -> impl IntoView {
    // Region — read ONCE at page entry, passed to all sub-views (18-04)
    let theme = use_context::<InitialTheme>().unwrap_or_default();
    let region = theme.0.clone();
```
After extraction, `DraftSurface(region: String)` receives this as a prop.

**`TreeDrafterPage` InitialTheme read** (tree_drafter.rs lines 260–262):
```rust
    let theme = use_context::<InitialTheme>().unwrap_or_default();
    let region = theme.0.clone();
```
Same pattern — thin shell reads it, passes as `region: String` to `TreeDrafterSurface`.

**`GamePlanPage` and `OpponentsPage`** do NOT read `InitialTheme` (confirmed: zero grep hits). Their thin shells do not need `region` prop threading — only auth-redirect stays.

---

### `BanPrioritySurface` extraction (src/pages/draft.rs)

**Server fns to move** (draft.rs lines 187–239): `get_ban_priorities` and `save_ban_priorities` — both follow the standard auth+db pattern. Move their definitions before the `BanPrioritySurface` component.

**`get_ban_priorities` server fn** (draft.rs lines 187–212):
```rust
#[server]
pub async fn get_ban_priorities() -> Result<Vec<BanPriority>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),  // rule #44: no-team → empty list
    };

    db::get_ban_priorities(&db, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
```

**Ban-priority signals to move** (draft.rs lines 1012–1019):
```rust
    // Ban priorities (Phase 4)
    let ban_priorities = Resource::new(|| (), |_| get_ban_priorities());
    let (ban_panel_open, set_ban_panel_open) = signal(false);
    let (editing_bans, set_editing_bans) = signal(false);
    let (ban_edit_list, set_ban_edit_list) = signal(Vec::<BanPriority>::new());
    let (ban_new_champ, set_ban_new_champ) = signal(String::new());
    let (ban_new_reason, set_ban_new_reason) = signal(String::new());
    let (ban_status, set_ban_status) = signal(Option::<String>::None);
```
These move entirely to `BanPrioritySurface`. `ban_new_champ` uses a plain `<input>` (draft.rs line 3621–3622) — NOT `<ChampionAutocomplete>`. No `get_champions()` Resource needed.

**`banned_champions` stays in `DraftSurface`** (draft.rs lines 1270–1280) — it is a local closure derived from `draft_slots`, not a DB Resource. Do not move it.

---

### `src/components/room/room_surface.rs` (component, self-refactor)

**Current state to replace** (room_surface.rs lines 59–66 — signature):
```rust
#[component]
pub fn RoomSurface(
    tabs: Vec<SurfaceTab>,
    children: ChildrenFn,
) -> impl IntoView {
```

**Current panel render to replace** (room_surface.rs lines 126–150):
```rust
        {tabs_stored.with_value(|tabs| {
            tabs.iter()
                .enumerate()
                .map(|(idx, tab)| {
                    let id = tab.id;
                    let tab_label_id = format!("tab-{id}");
                    let panel_id = format!("panel-{id}");
                    let children = children.clone();
                    view! {
                        <Show when=move || active_idx.get() == idx>
                            <div id=panel_id.clone() role="tabpanel" aria-labelledby=tab_label_id.clone()>
                                {children()}
                            </div>
                        </Show>
                    }
                })
                .collect_view()
        })}
```

**New type alias and signature** (from RESEARCH.md Pattern 1):
```rust
pub type PanelFn = Box<dyn Fn() -> AnyView>;

#[component]
pub fn RoomSurface(
    panels: Vec<(SurfaceTab, PanelFn)>,
) -> impl IntoView {
```

**New panel render — keep-mounted with `attr:hidden`** (from RESEARCH.md Pattern 1):
```rust
    panels.into_iter().enumerate().map(|(idx, (tab, render_fn))| {
        let id = tab.id;
        let tab_label_id = format!("tab-{id}");
        let panel_id = format!("panel-{id}");
        view! {
            <div
                id=panel_id
                role="tabpanel"
                aria-labelledby=tab_label_id
                attr:hidden=move || if active_idx.get() != idx { Some("") } else { None }
            >
                {render_fn()}
            </div>
        }
    }).collect_view()
```

**Tab bar template survives unchanged** (room_surface.rs lines 96–124) — only the panel render block changes.

**Unit tests that survive** (room_surface.rs lines 156–184): all four `resolve_active_tab` tests are pure-function tests — they require no change. The old `ChildrenFn`-based component render test (if one existed) must be replaced with a dummy-`PanelFn` test that verifies the tablist renders. `SurfaceTab` struct and `resolve_active_tab` fn are unchanged.

---

### `src/components/room/parallax.rs` (component, self-refactor)

**Existing on_cleanup pattern** (parallax.rs lines 210–231):
```rust
        on_cleanup(move || {
            listener.update_value(|slot| {
                if let Some(cb) = slot.take() {
                    if let Some(w) = web_sys::window() {
                        let _ = w.remove_event_listener_with_callback(
                            "mousemove",
                            cb.as_ref().unchecked_ref(),
                        );
                    }
                }
            });
            mql_listener.update_value(|slot| {
                if let Some((mql, cb)) = slot.take() {
                    let _ = mql
                        .remove_event_listener_with_callback("change", cb.as_ref().unchecked_ref());
                }
            });
        });
```

**AtomicBool guard addition** (from RESEARCH.md Pattern 5) — add at module top level, before the `#[component]`:
```rust
use std::sync::atomic::{AtomicBool, Ordering};
static MOUSEMOVE_ACTIVE: AtomicBool = AtomicBool::new(false);
```

**Guard in the Effect** — insert inside `#[cfg(feature = "hydrate")]` block, at the start of `Effect::new`, before the existing `add_event_listener` call (currently at parallax.rs line ~202):
```rust
if MOUSEMOVE_ACTIVE.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
    // Another AtmosphereRoot instance already holds the listener; skip.
    return;
}
```

**Guard reset in on_cleanup** — add one line before the existing `listener.update_value` call:
```rust
MOUSEMOVE_ACTIVE.store(false, Ordering::SeqCst);
```

The `static` declaration is unconditional (SSR-harmless); the `compare_exchange` and `store` are inside the `#[cfg(feature = "hydrate")]` block.

---

### `src/app.rs` (routing, config)

**Analog:** self, lines 140–165. Precedent is `/journey` at line 156.

**Current import block to extend** (app.rs lines 29):
```rust
    rooms::solo_journey::SoloJourneyRoom,
```
Add: `rooms::war_room::WarRoomRoom,`

**Route block to extend** (app.rs lines 155–156):
```rust
                        <Route path=path!("/journey") view=SoloJourneyRoom />
```
Add after line 156:
```rust
                        <Route path=path!("/war-room") view=WarRoomRoom />
```
No existing flat routes need to move in `app.rs` — they keep their existing `view=DraftPage` etc. bindings pointing at the new thin shells (same component name, same path, behavior preserved per D-02).

---

### `e2e/tests/war-room.spec.ts` (test)

**Analog:** `e2e/tests/room-solo-journey.spec.ts` (165 lines) + `e2e/tests/hydration-no-panic.spec.ts` for the canary pattern.

**File/import header** (room-solo-journey.spec.ts lines 1–24):
```typescript
import { test, expect } from "./fixtures";
const ROOM_BG_RE = /\/img\/rooms\//;
test.describe("War Room", () => {
```

**Atmosphere + surface renders test** (mirror room-solo-journey.spec.ts lines 27–45):
```typescript
  test("renders the War surfaces over the parallax atmosphere", async ({ authedPage }) => {
    await authedPage.goto("/war-room");
    await authedPage.waitForLoadState("networkidle");
    await expect(authedPage.locator(".room-atmosphere")).toBeVisible();
    expect(await authedPage.locator(".room-plane").count()).toBeGreaterThanOrEqual(3);
    await expect(authedPage.locator('[data-feedback-label="War Room"]')).toBeVisible();
    expect(await authedPage.locator("text=/Failed to load/i").count()).toBe(0);
  });
```

**ARIA tablist test** (RESEARCH.md SC2):
```typescript
  test("tab bar has correct ARIA roles", async ({ authedPage }) => {
    await authedPage.goto("/war-room");
    await authedPage.waitForLoadState("networkidle");
    await expect(authedPage.locator('[role="tablist"]')).toBeVisible();
    const tabs = authedPage.locator('[role="tab"]');
    expect(await tabs.count()).toBe(5);
    const selectedTabs = authedPage.locator('[role="tab"][aria-selected="true"]');
    expect(await selectedTabs.count()).toBe(1);
  });
```

**Growth tiers test** (mirror room-solo-journey.spec.ts lines 103–119, adapt for War Room axes):
```typescript
  test("growth-state tier attributes are present and in range", async ({ authedPage }) => {
    await authedPage.goto("/war-room");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(800);
    const root = authedPage.locator(".room-atmosphere");
    for (const axis of ["tactics", "intel", "command"]) {
      const val = await root.getAttribute(`data-${axis}-tier`);
      expect(val, `data-${axis}-tier present`).not.toBeNull();
      const n = parseInt(val ?? "", 10);
      expect(Number.isNaN(n)).toBe(false);
      expect(n).toBeGreaterThanOrEqual(0);
      expect(n).toBeLessThanOrEqual(3);
    }
  });
```

**Hydration canary (demacia + pandemonium)** — copy `attachPanicListeners` helper from hydration-no-panic.spec.ts lines 101–119, then:
```typescript
const PANIC_REGEX = /panicked at|Option::unwrap.*None|tachys/i;

async function attachPanicListeners(page: Page): Promise<string[]> {
  const errors: string[] = [];
  page.on("pageerror", (err) => errors.push(`pageerror: ${err.message}`));
  page.on("console", (msg) => {
    if (msg.type() === "error" && PANIC_REGEX.test(msg.text())) {
      errors.push(`console.error: ${msg.text()}`);
    }
  });
  return errors;
}

for (const region of ["demacia", "pandemonium"] as const) {
  test(`/war-room hydrates without tachys panic [${region}]`, async ({ authedPage }) => {
    const errors = await attachPanicListeners(authedPage);
    await setRegion(authedPage, region);
    await authedPage.goto("/war-room");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(1500);
    expect(errors, `Panic on /war-room [${region}]: ${errors.join("\n")}`).toEqual([]);
  });
}
```

**Mousemove guard round-trip test** (RESEARCH.md SC4):
```typescript
  test("mousemove listener does not accumulate across room navigations", async ({ authedPage }) => {
    await authedPage.goto("/draft");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await authedPage.goto("/war-room");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await authedPage.goto("/journey");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);
    await authedPage.goto("/war-room");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);

    const count = await authedPage.evaluate(() =>
      (window as any).getEventListeners?.(window)?.mousemove?.length ?? -1
    );
    // If getEventListeners is unavailable (-1), the test is advisory-only.
    if (count !== -1) {
      expect(count).toBe(1);
    }
  });
```

---

### `e2e/tests/pages.spec.ts` (test, modification)

**Analog:** self, lines 8–25 (`AUTHED_PAGES` array).

Add to the `AUTHED_PAGES` array after the existing `/analytics` entry:
```typescript
  { path: "/war-room", content: /draft|war|room|plan/i },
```

---

## Shared Patterns

### Auth redirect (hydrate-only)
**Source:** `src/pages/solo_dashboard.rs` lines 284–293 (also draft.rs:935–943, tree_drafter.rs:250–258)
**Apply to:** `DraftSurface`, `TreeDrafterSurface`, `GamePlanSurface`, `OpponentsSurface`, `BanPrioritySurface` — each keeps its own `auth_user` Resource and redirect Effect.
```rust
let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
Effect::new(move || {
    if let Some(Ok(None)) = auth_user.get() {
        #[cfg(feature = "hydrate")]
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/auth/login");
        }
    }
});
```

### Server fn DB + auth access
**Source:** `src/pages/solo_dashboard.rs` lines 28–40 (any server fn)
**Apply to:** `get_team_growth_state` in `war_room.rs`, any new server fn in `BanPrioritySurface`
```rust
    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;
```

### No-team empty return (rule #44)
**Source:** `src/pages/draft.rs` lines 200–207
**Apply to:** `get_team_growth_state` (return `Ok(TeamGrowthState::default())`), any team-scoped server fn
```rust
    let team_id = match db::get_user_team_id(&db, &user.id)
        .await.map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()), // or Ok(TeamGrowthState::default())
    };
```

### Region-read-once (rule #58)
**Source:** `src/pages/rooms/solo_journey.rs` line 18; `src/pages/solo_dashboard.rs` line 497
**Apply to:** `WarRoomRoom` top (read once), `DraftPage` thin shell (read once), `TreeDrafterPage` thin shell (read once)
```rust
let region = use_context::<InitialTheme>().unwrap_or_default().0;
```
Never read inside `Signal::derive`, `Effect`, `move || {}`, or a child component.

### `#[allow(unused_variables)]` on hydrate-only signals (rule #43)
**Source:** wasm-patterns.md rule 43
**Apply to:** any `RwSignal` in `*Surface` files that is only read/written inside `#[cfg(feature = "hydrate")]` blocks
```rust
#[allow(unused_variables)]
let auto_save_timer: RwSignal<Option<i32>> = RwSignal::new(None);
```

### Region clone-per-closure for `WarSurfaces`
**Source:** RESEARCH.md Pitfall 5 + solo_journey.rs line 47 (`let surfaces_region = region.clone()`)
**Apply to:** `WarSurfaces` building the `panels` vec — clone `region` once per `PanelFn` closure before the `vec![]`:
```rust
let r1 = region.clone(); // DraftSurface
let r2 = region.clone(); // TreeDrafterSurface
// … etc (5 total, one per surface that takes region)
```

---

## No Analog Found

All files have matching analogs. The two genuinely new patterns (keep-mounted `attr:hidden` and `AtomicBool` guard) are documented directly in RESEARCH.md Pattern 1 and Pattern 5 with exact code, and the existing `RoomSurface` / `parallax.rs` files serve as the before-state.

---

## Key Notes for Planner

1. **`game_plan.rs` and `opponents.rs` do NOT use `InitialTheme`** — their thin shells need no `region` prop threading. Only `draft.rs` and `tree_drafter.rs` have `InitialTheme` reads to migrate.

2. **`ban_new_champ` uses a plain `<input>`** (draft.rs lines 3621–3622) — `BanPrioritySurface` does NOT need a `get_champions()` Resource. Assumption A3 from RESEARCH.md is confirmed.

3. **`RoomSurface` has no component-level render test** — only the four pure-function `resolve_active_tab` tests exist (lines 156–184). The planner only needs to add a new dummy-`PanelFn` smoke test; the existing four tests are untouched.

4. **`DraftPage` currently reads both `InitialTheme` (`:931`) and `theme.accent` (`:932` — `theme.0.clone()`)** — the thin shell keeps the full `theme` extraction; only the outer div chrome moves; the surface receives `region: String`.

5. **`war_room.rs` is ~200 lines target** — the `get_team_growth_state` server fn, `WarRoomRoom`, and `WarSurfaces` all live in this one file (parallel to `solo_dashboard.rs` → `SoloSurfaces` + `get_growth_state` + `SoloDashboardPage` in one file).

---

## Metadata

**Analog search scope:** `src/pages/rooms/`, `src/pages/solo_dashboard.rs`, `src/components/room/`, `src/pages/draft.rs`, `e2e/tests/`
**Files read:** 13 source files + 2 planning files
**Pattern extraction date:** 2026-06-05
