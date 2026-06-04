# Phase 26: Room System Foundations - Pattern Map

**Mapped:** 2026-06-04
**Files analyzed:** 7 (new/modified files for this phase)
**Analogs found:** 7 / 7

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/models/growth.rs` (modify) | model | transform | `src/models/growth.rs` (existing half) | exact |
| `src/server/db.rs` (modify) | service | CRUD/batch | `src/server/db.rs:4109` (`get_growth_counts`) | exact |
| `src/components/room/parallax.rs` (modify) | component | event-driven | same file (existing `AtmosphereRoot`) | exact |
| `src/components/room/room_surface.rs` (new) | component | request-response | `src/components/room/quick_nav.rs` + `src/pages/opponents.rs:390` | role-match |
| `CLAUDE.md` (modify — ROOM-03 rule) | config/doc | n/a | existing numbered rules in `CLAUDE.md:125-173` | exact format |
| `.claude/rules/leptos-patterns.md` (modify — ROOM-03 rule) | config/doc | n/a | existing rules in `leptos-patterns.md` | exact format |
| `src/pages/rooms/solo_journey.rs` (regression target, no change) | component | request-response | itself | exact |

---

## Pattern Assignments

### `src/models/growth.rs` (modify — add `TeamGrowthCounts` + `TeamGrowthState`)

**Analog:** `src/models/growth.rs` — existing `GrowthCounts` / `GrowthState` block (lines 40–171)

**Imports pattern** (lines 1–9 — no change needed):
```rust
use serde::{Deserialize, Serialize};
```

**Core struct pattern** — mirror `GrowthCounts` (lines 40–48) for the team-scoped variant:
```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrowthCounts {
    pub personal_learning: u32,
    pub champion_pool: u32,
    // … six fields …
}
```
Team variant substitutes team-scoped field names (`draft`, `draft_tree`, `opponent`, `ban_priority`, `post_game_learning`, `team_note`) — same derive list, same `Default`.

**`tier_for` reuse** (lines 66–76 — no change, already `pub(crate)` accessible):
```rust
fn tier_for(count: u32, thresholds: [u32; 3]) -> Tier {
    if count >= thresholds[2] { Tier::Flourishing }
    else if count >= thresholds[1] { Tier::Established }
    else if count >= thresholds[0] { Tier::Sprout }
    else { Tier::Seed }
}
```
`TeamGrowthState::from_counts` calls `tier_for` exactly as `GrowthState::from_counts` does (lines 81–91): compute each tier, call `.level()`, collect into struct.

**`Default` impl pattern** (lines 94–98):
```rust
impl Default for GrowthState {
    fn default() -> Self {
        GrowthState::from_counts(GrowthCounts::default())
    }
}
```
Mirror verbatim for `TeamGrowthState`.

**War Room axis thresholds** (Phase 26 owns D-06/D-07/D-08 only — Library/Throne deferred):
- `tactics_tier` ← `draft + draft_tree`, thresholds `[1, 5, 20]`
- `intel_tier` ← `opponent`, thresholds `[1, 3, 10]`
- `command_tier` ← `ban_priority + post_game_learning`, thresholds `[1, 5, 15]`

**Unit test pattern** (lines 101–171 — mirror entire suite for TeamGrowthState):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_counts_are_all_seed() {
        let g = TeamGrowthState::default();
        assert_eq!(g.tactics_tier, 0);
        assert_eq!(g.intel_tier, 0);
        assert_eq!(g.command_tier, 0);
    }

    #[test]
    fn tier_for_respects_lower_bounds() { /* copy verbatim */ }

    #[test]
    fn growth_state_round_trips_json() {
        let g = TeamGrowthState::from_counts(TeamGrowthCounts { draft: 3, .. Default::default() });
        let json = serde_json::to_string(&g).unwrap();
        let back: TeamGrowthState = serde_json::from_str(&json).unwrap();
        assert_eq!(g, back);
    }
}
```

---

### `src/server/db.rs` (modify — add `get_team_growth_counts`)

**Analog:** `src/server/db.rs:4109-4149` (`get_growth_counts`)

**Function signature pattern** (line 4109):
```rust
pub async fn get_growth_counts(db: &Surreal<Db>, user_id: &str) -> DbResult<GrowthCounts> {
```
Team variant: `pub async fn get_team_growth_counts(db: &Surreal<Db>, team_id: &str) -> DbResult<TeamGrowthCounts>`

**Prefix-strip pattern** (line 4110 — surreal-patterns rule 2):
```rust
let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
```
Team variant: `let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();`

**Batched query pattern** (lines 4112–4122 — surreal-patterns rules 1, 29):
```rust
let mut result = db
    .query(
        "SELECT count() as n FROM personal_learning WHERE user = type::record('user', $user_key) GROUP ALL; \
         SELECT count() as n FROM champion_pool WHERE user = type::record('user', $user_key) GROUP ALL; \
         /* … 6 statements … */",
    )
    .bind(("user_key", user_key))
    .await?;
```
Team variant uses `type::record('team', $team_key)` and queries `draft`, `draft_tree`, `opponent`, `ban_priority`, `post_game_learning`, `team_note` (6 statements, one per field, D-04).

**`count_of` helper + `take(n).unwrap_or_default()` pattern** (lines 4124–4148 — surreal-patterns rule 28):
```rust
#[derive(Debug, Deserialize, SurrealValue)]
struct CountResult { n: i64 }

fn count_of(rows: Vec<CountResult>) -> u32 {
    rows.first().map(|r| r.n.max(0) as u32).unwrap_or(0)
}

let personal_learning = count_of(result.take(0).unwrap_or_default());
// … indices 0–5 …
Ok(GrowthCounts { personal_learning, champion_pool, … })
```
Team variant: copy `count_of` helper verbatim (it is local — define it again inside the new function body), index `take(0)` through `take(5)`.

**Import** (already present at top of `db.rs`): `use crate::models::growth::GrowthCounts;` — add `TeamGrowthCounts` to the same use path.

---

### `src/components/room/parallax.rs` (modify — axis generalization + MQL change-listener)

**Analog:** same file, existing `AtmosphereRoot` component (lines 24–157)

**Current fixed prop block** (lines 32–35) — replace with generalized optional props:
```rust
#[prop(optional, into)]
lib_tier: Signal<u8>,
#[prop(optional, into)] armory_tier: Signal<u8>,
#[prop(optional, into)] banner_tier: Signal<u8>,
```
Generalized: add twelve more `#[prop(optional, into)] <axis>_tier: Signal<u8>` fields covering all four rooms' axes (SUMMARY.md axis table). All default via `Signal::derive(|| 0u8)` — Leptos uses `Default::default()` for optional props, so each field must implement `Default`. Wrap via `#[prop(optional, into)]` which uses `Into<Signal<u8>>` — `Signal::derive(|| 0u8)` satisfies this.

**Mount-time `match_media` check** (lines 60–68) — existing pattern to keep intact (do not remove):
```rust
let reduced = win
    .match_media("(prefers-reduced-motion: reduce)")
    .ok().flatten()
    .map(|m| m.matches())
    .unwrap_or(false);
if reduced { return; }
```

**New MQL change-listener pattern** (A11Y-01 — add after the existing reduced-motion check, before the mousemove closure, inside the `#[cfg(feature = "hydrate")]` block):

Copy `StoredValue<Option<Closure<...>>>` shape from lines 50–52 (existing `listener` slot pattern):
```rust
// Store MQL handle in LocalStorage StoredValue for on_cleanup teardown
// (wasm-patterns rules 37, 42; surreal — no, wasm rule 43 for unused_variables)
#[allow(unused_variables)]
let mql_listener: StoredValue<
    Option<(web_sys::MediaQueryList, Closure<dyn FnMut(web_sys::MediaQueryListEvent)>)>,
    LocalStorage,
> = StoredValue::new_local(None);
```
Then, after the `if reduced { return; }` early-exit, attach a `change` event listener on the `MediaQueryList` that sets `--px`/`--py` to `"0"` (freeze) or re-attaches the mousemove listener (restore). Store handle for cleanup in `mql_listener`.

`on_cleanup` mirror (lines 130–142): add `mql_listener.update_value(|slot| { if let Some((mql, cb)) = slot.take() { let _ = mql.remove_event_listener_with_callback("change", cb.as_ref().unchecked_ref()); } });`

**`data-*-tier` emission pattern** (lines 150–152):
```rust
data-lib-tier=move || lib_tier.get().to_string()
data-armory-tier=move || armory_tier.get().to_string()
data-banner-tier=move || banner_tier.get().to_string()
```
Add one `data-{axis}-tier=move || {axis}_tier.get().to_string()` per new axis (12 new props total covering war-room, library, throne axes). Solo Journey `lib`/`armory`/`banner` props remain unchanged — `solo_journey.rs:50-54` keeps working verbatim.

**`GrowthOverlay` axis comment** (line 185): update short axis key list from `lib | armory | banner` to include all axes.

---

### `src/components/room/room_surface.rs` (new — `RoomSurface` tab component)

**Best analog:** `src/components/room/quick_nav.rs` (lines 24–90) for `RwSignal<bool>` + `<Show>` + ARIA dialog pattern; `src/pages/opponents.rs:390-391` for `use_query_map` / URL sync pattern; `ARCHITECTURE.md` Pattern 2 for the full `WarSurfaces` reference.

**Module registration:** add `pub mod room_surface;` to `src/components/room/mod.rs`; add `pub use room_surface::RoomSurface;` to the re-export block.

**Imports pattern** (copy from `quick_nav.rs:9-12`):
```rust
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use leptos_router::hooks::use_navigate;
```

**Core generic shape** (ARCHITECTURE.md Pattern 2 reference):
```rust
/// Intra-room tab bar + `<Show>`-per-surface wrapper.
///
/// Contract:
/// - `tab_id` values must be stable ASCII slugs (used in `?tab=` URL param).
/// - Tab bar uses ARIA tablist/tab/tabpanel roles (A11Y A-5).
/// - URL sync: reading/writing `?tab=` via `use_query_map` + `use_navigate`;
///   invalid/missing value falls back to first tab (D-01).
/// - NEVER uses sub-routes — `AtmosphereRoot` must never remount (D-02).

#[derive(Clone, PartialEq)]
pub struct SurfaceTab {
    pub id: &'static str,     // URL slug, e.g. "draft"
    pub label: &'static str,  // Display label
}

#[component]
pub fn RoomSurface(
    tabs: Vec<SurfaceTab>,
    children: ChildrenFn,  // receives active tab id as context or prop
) -> impl IntoView { … }
```

**`?tab=` URL sync pattern** (mirror `opponents.rs:390-391`):
```rust
let params = use_query_map();
let navigate = use_navigate();
// Read active tab from URL; fallback to first tab slug
let active = Signal::derive(move || {
    let q = params.read().get("tab").unwrap_or_default();
    if tabs.iter().any(|t| t.id == q.as_str()) { q } else { tabs[0].id.to_string() }
});
let set_active = move |id: &'static str| {
    navigate(&format!("?tab={id}"), Default::default());
};
```

**`<Show>`-per-tab pattern** (mirror `quick_nav.rs:70`):
```rust
<Show when=move || open.get()>
    …
</Show>
```
For `RoomSurface` one `<Show>` per tab, keyed on `active.get() == tab.id`.

**ARIA tablist pattern** (from ARCHITECTURE.md Pattern 2 concrete shape):
```rust
view! {
    <nav role="tablist" aria-label={label} class="flex gap-2 …">
        {tabs.iter().map(|tab| {
            let id = tab.id;
            view! {
                <button
                    role="tab"
                    id=format!("tab-{id}")
                    aria-selected=move || (active.get() == id).to_string()
                    aria-controls=format!("panel-{id}")
                    on:click=move |_| set_active(id)
                    class=…
                >"…"</button>
            }
        }).collect_view()}
    </nav>
    // One tabpanel per surface — Show mounts/unmounts (safe; each surface is SSR-stable)
    {tabs.iter().map(|tab| {
        let id = tab.id;
        view! {
            <Show when=move || active.get() == id>
                <div
                    id=format!("panel-{id}")
                    role="tabpanel"
                    aria-labelledby=format!("tab-{id}")
                >
                    {children()}  // or: provide_context(active_tab); children()
                </div>
            </Show>
        }
    }).collect_view()}
}
```
Note: `collect_view()` per leptos-patterns rule 25.

**Error handling:** none — this component has no server calls; all state is local.

---

### `CLAUDE.md` (modify — add anti-structural-branching rule, ROOM-03)

**Analog:** existing numbered rule format at `CLAUDE.md` lines 171-173:
```
41. **Tree assembly: use `children_of` map, not reversal heuristic** — Building a `HashMap<String, Vec<String>>` of parent→child IDs and doing a recursive DFS is correct regardless of DB return order. The old reverse-child_ids heuristic failed when sibling nodes shared the same `sort_order`.
```

**Where to insert:** in `## Critical Patterns and Gotchas`, after rule 41, before `## Claude Code Dev Workflow`. New rule number: **58** (next after rule 57 in `leptos-patterns.md`).

**Rule format to produce:**
```
58. **Room components: read `InitialTheme`/region ONCE at room top level — never inside reactive primitives** — Reading `use_context::<InitialTheme>()` inside a `Signal::derive()`, `Effect`, `move || {}` view block, or child component causes an SSR/hydrate structural mismatch: the tachys reconciler panics at `html/mod.rs:217` with `Option::unwrap()`, silently killing all WASM event handlers (Phase 18.2 incident). The fix: read region ONCE at the room component's top level (`SoloJourneyRoom`, `WarRoomRoom`, etc.), bind it to a local `String`, and pass it as a `region: String` prop to every child. Children use the prop; they never call `use_context`.
    ```rust
    // CORRECT — region read once, passed as prop
    let region = use_context::<InitialTheme>().unwrap_or_default().0;
    view! { <SoloSurfaces region=region.clone() /> }

    // WRONG — panics on non-default region
    let region = Signal::derive(|| use_context::<InitialTheme>().unwrap_or_default().0);
    ```
```

---

### `.claude/rules/leptos-patterns.md` (modify — add anti-structural-branching rule, ROOM-03)

**Analog:** existing rule entry format throughout the file; e.g. rule 54 (lines 77–87):
```
54. **Auto-save Effects must capture values eagerly** — Capture ALL signal values in the Effect body (eagerly), not inside the timer callback (lazily). …
    ```rust
    Effect::new(move |_| {
        …
    });
    ```
```

**Where to insert:** in `## Reactivity` section, after rule 55, as new rule **58** (consistent with `CLAUDE.md` numbering).

**Rule format to produce:**
```markdown
58. **Room components: read context ONCE at room top level — never inside reactive primitives** — Reading `use_context::<InitialTheme>()` (or any context) inside a reactive closure, `Signal::derive()`, `Effect`, or child component during SSR causes the tachys reconciler to see different DOM structure on hydration, panicking at `html/mod.rs:217` with `Option::unwrap()` and silently killing all WASM event handlers. This is the **Phase 18.2 incident root cause** (see `.planning/phases/18.2-pandemonium-hydration-reconciliation-gap-closure-for-ui-18-1/18.2-CONTEXT.md`).

    **Rule:** In every `<Room>Room` component, read `InitialTheme` exactly once at the function top level. Pass the result as a `region: String` prop to all children. A `region` value may only be used for: string-interpolated `src`/`href` attributes, `data-*` attribute values, or passing to child component props. It must never appear in a reactive closure, `Signal::derive()`, `move || {}`, or structural branch.

    ```rust
    // CORRECT — read once, passed as plain prop
    pub fn SoloJourneyRoom() -> impl IntoView {
        let region = use_context::<InitialTheme>().unwrap_or_default().0; // line 18: solo_journey.rs
        view! { <SoloSurfaces region=region.clone() /> }
    }

    // WRONG — context read inside reactive closure → hydration panic on non-default region
    let region = Signal::derive(|| use_context::<InitialTheme>().unwrap_or_default().0);
    // WRONG — context read inside child component
    #[component] fn MyChild() -> impl IntoView {
        let region = use_context::<InitialTheme>().unwrap_or_default().0; // PANIC
    }
    ```
```

---

## Shared Patterns

### Auth + DB extraction in server functions
**Source:** `src/pages/solo_dashboard.rs:71-89` (`get_growth_state` server fn)
**Apply to:** future `get_team_growth_state` server fn (Phase 27) that calls `get_team_growth_counts`
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
Note: `unwrap_or_default()` on the DB result (not `?`) implements rule 44 — an empty room is valid, not a failure.

### `StoredValue<Option<Closure>>` + `on_cleanup` teardown
**Source:** `src/components/room/parallax.rs:50-52, 130-142`
**Apply to:** new MQL change-listener in same file (A11Y-01); any future WASM listener added to `AtmosphereRoot`
```rust
let listener: StoredValue<Option<Closure<dyn FnMut(web_sys::MouseEvent)>>, LocalStorage> =
    StoredValue::new_local(None);

// … attach listener, store in slot …
listener.set_value(Some(cb));

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
});
```

### Signal::derive for tier props from Resource
**Source:** `src/pages/rooms/solo_journey.rs:22-42`
**Apply to:** every future room page that derives tier signals from a growth resource
```rust
let growth = Resource::new(|| (), |_| get_growth_state());
let lib_tier = Signal::derive(move || {
    growth
        .get()
        .and_then(|r| r.ok())
        .map(|g| g.library_tier)
        .unwrap_or(0)
});
```

### `use_query_map` for URL param read
**Source:** `src/pages/opponents.rs:390-391`
**Apply to:** `RoomSurface` `?tab=` URL sync
```rust
let params = leptos_router::hooks::use_query_map();
let return_to = move || params.read().get("return_to").unwrap_or_default();
```

### Numeric-tier CSS attribute emission (reactive)
**Source:** `src/components/room/parallax.rs:150-152`
**Apply to:** all new axis props in generalized `AtmosphereRoot`
```rust
data-lib-tier=move || lib_tier.get().to_string()
data-armory-tier=move || armory_tier.get().to_string()
data-banner-tier=move || banner_tier.get().to_string()
```

---

## No Analog Found

All files have strong analogs in the codebase. The `RoomSurface` component is the weakest match (no existing ARIA tablist pattern in the codebase), but `quick_nav.rs` provides the `<Show>` + ARIA dialog skeleton and `ARCHITECTURE.md` provides a concrete reference implementation to copy directly.

| File | Note |
|---|---|
| `src/components/room/room_surface.rs` | No existing tablist component; closest is `quick_nav.rs` `<Show>` + `opponents.rs` URL-param read. ARCHITECTURE.md Pattern 2 provides the complete reference. |

---

## Analog Search Scope

**Directories searched:** `src/models/`, `src/server/db.rs`, `src/components/room/`, `src/pages/rooms/`, `src/pages/solo_dashboard.rs`, `src/pages/opponents.rs`, `src/pages/personal_learnings.rs`, `CLAUDE.md`, `.claude/rules/`

**Files scanned:** 12

**Key patterns confirmed:**
- `tier_for` is a free `fn` (not `pub`) in `growth.rs` — must be widened to `pub(crate)` when `TeamGrowthState` is added in the same file, OR `TeamGrowthState::from_counts` can call it directly since it is in the same module. No change needed if both structs are in the same `growth.rs` module.
- `count_of` helper in `db.rs` is a local `fn` inside `get_growth_counts` — must be copy-pasted (or hoisted to a shared helper) for `get_team_growth_counts`. Simplest: define identically inside the new function body.
- `#[prop(optional, into)] Signal<u8>` fields on `AtmosphereRoot` — Leptos 0.8 optional prop defaults to `Default::default()`. `Signal<u8>` does not implement `Default` directly; research note says use `Signal::derive(|| 0u8)`. Implementor must verify whether `#[prop(optional, into)]` with a custom `Default` impl on a newtype is needed, or whether `Option<Signal<u8>>` with `.unwrap_or_else(|| Signal::derive(|| 0u8))` is the correct Leptos 0.8 idiom. Read `parallax.rs:32-35` carefully — the existing `lib_tier: Signal<u8>` is `#[prop(optional, into)]` and already works, which means `Signal<u8>` does implement `Default` (it defaults to a signal yielding `0u8`). Safe to copy the identical `#[prop(optional, into)]` pattern for new axes.

**Pattern extraction date:** 2026-06-04
