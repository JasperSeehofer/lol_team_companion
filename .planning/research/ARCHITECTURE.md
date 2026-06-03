# Architecture Research

**Domain:** v1.4 Immersive Companion — multi-room extension (War Room, Library, Throne)
**Researched:** 2026-06-03
**Confidence:** HIGH (all findings grounded in the existing codebase; no external dependencies introduced)

---

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  app.rs — Router shell (AtmosphereRoot mounts here per /room)   │
├─────────────────────────────────────────────────────────────────┤
│  Room Pages  (src/pages/rooms/<room>.rs)                        │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │  SoloJourney │ │  WarRoom     │ │  Library     │ … Throne   │
│  │  (done)      │ │  (new)       │ │  (new)       │ (new)      │
│  └──────┬───────┘ └──────┬───────┘ └──────┬───────┘             │
│         │               │               │                        │
│         ▼               ▼               ▼                        │
│  src/components/room/parallax.rs                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  AtmosphereRoot  (mousemove → RAF → --px/--py on DOM id) │    │
│  │    Plane × 3   GrowthOverlay × N   ContrastScrim         │    │
│  │    <div z-10>  ← Surfaces component floats here          │    │
│  └─────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────┤
│  Surfaces Components  (extracted from existing pages)           │
│  ┌──────────┐ ┌────────────┐ ┌──────────────┐ ┌─────────────┐  │
│  │SoloSurf. │ │WarSurfaces │ │LibSurfaces   │ │ThroneSurf.  │  │
│  │(existing)│ │(new)       │ │(new)         │ │(new)        │  │
│  └────┬─────┘ └─────┬──────┘ └──────┬───────┘ └──────┬──────┘  │
│       │             │              │               │            │
├───────┴─────────────┴──────────────┴───────────────┴────────────┤
│  Growth Layer                                                   │
│  src/models/growth.rs: GrowthState + TeamGrowthState           │
│  src/server/db.rs: get_growth_counts + get_team_growth_counts  │
│  src/pages/<room>.rs: get_growth_state + get_team_growth_state │
├─────────────────────────────────────────────────────────────────┤
│  Navigation Layer                                               │
│  nav.rs: HUB_ROUTES (existing sub-nav strip, shows per path)   │
│  quick_nav.rs: QuickNav (Cmd+K palette, reuses HUB_ROUTES)     │
│  In-room mode toggle: RwSignal<ActiveSurface> inside Surfaces  │
└─────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| `AtmosphereRoot` | Owns pointer→CSS-var bridge; writes `--px`/`--py` to DOM; hosts planes, overlays, scrim, surface slot | `src/components/room/parallax.rs` |
| `Plane` | Single fixed full-bleed parallax layer; consumes `--px`/`--py` via CSS calc | `src/components/room/parallax.rs` |
| `GrowthOverlay` | Growth-driven atmosphere overlay; always DOM-present; opacity driven by `data-*-tier` CSS | `src/components/room/parallax.rs` |
| `QuickNav` | Cmd+K command palette; cross-room navigation; reuses `HUB_ROUTES` | `src/components/room/quick_nav.rs` |
| `<Room>Room` | Entry point; reads `InitialTheme` once; fetches growth resource; selects active region's art; composes `AtmosphereRoot` + surfaces | `src/pages/rooms/<room>.rs` |
| `<Room>Surfaces` | Extracted page body; receives `region` as prop; owns intra-room mode state; hosts existing page components | extracted from existing pages |
| `GrowthState` / `TeamGrowthState` | Pure tier math; `from_counts` maps raw counts to 0–3 tiers; no DB dependency | `src/models/growth.rs` |
| `get_growth_counts` / `get_team_growth_counts` | Batched `count() GROUP ALL` queries; user-scoped vs team-scoped variants | `src/server/db.rs` |

---

## Recommended Project Structure

```
src/
├── models/
│   └── growth.rs                    # Add TeamGrowthCounts + TeamGrowthState here
├── server/
│   └── db.rs                        # Add get_team_growth_counts here
├── components/
│   └── room/
│       ├── mod.rs                   # Re-export AtmosphereRoot, Plane, GrowthOverlay, QuickNav
│       ├── parallax.rs              # (existing, unchanged)
│       └── quick_nav.rs             # (existing, unchanged)
└── pages/
    ├── rooms/
    │   ├── mod.rs                   # Add war_room, library, throne
    │   ├── solo_journey.rs          # (existing)
    │   ├── war_room.rs              # (new)
    │   ├── library.rs               # (new)
    │   └── throne.rs                # (new)
    ├── draft.rs                     # Extract WarSurfaces component from here or war_room.rs
    ├── stats.rs                     # Extract LibSurfaces component or host in library.rs
    └── solo_dashboard.rs            # SoloSurfaces already extracted (done)

public/img/rooms/
├── solo-journey/                    # (existing: demacia/ + pandemonium/ each with 3 planes + 3 overlays)
│   ├── demacia/
│   │   ├── plane-sky.webp
│   │   ├── plane-mid.webp
│   │   ├── plane-near.webp
│   │   ├── overlay-lib.webp
│   │   ├── overlay-armory.webp
│   │   └── overlay-banner.webp
│   └── pandemonium/
│       └── (same 6 files)
├── war-room/                        # (new)
│   ├── demacia/
│   │   ├── plane-sky.webp           # horizon / fortress walls
│   │   ├── plane-mid.webp           # war table / strategic objects
│   │   ├── plane-near.webp          # foreground frame (torches, banners)
│   │   ├── overlay-tactics.webp     # grows with: drafts saved
│   │   ├── overlay-intel.webp       # grows with: opponents scouted
│   │   └── overlay-command.webp     # grows with: ban-priority entries
│   └── pandemonium/
│       └── (same 6 files, chaotic palette)
├── library/                         # (new)
│   ├── demacia/
│   │   ├── plane-sky.webp           # reading room ceiling / light
│   │   ├── plane-mid.webp           # shelves of tomes
│   │   ├── plane-near.webp          # foreground desk / quill
│   │   ├── overlay-records.webp     # grows with: post-game reviews
│   │   ├── overlay-tomes.webp       # grows with: team learnings
│   │   └── overlay-analysis.webp    # grows with: analytics depth (games synced)
│   └── pandemonium/
│       └── (same 6 files)
└── throne/                          # (new)
    ├── demacia/
    │   ├── plane-sky.webp           # throne hall ceiling / light shaft
    │   ├── plane-mid.webp           # throne dais / banners
    │   ├── plane-near.webp          # foreground pillars / floor
    │   ├── overlay-roster.webp      # grows with: team members
    │   ├── overlay-progression.webp # grows with: solo LP milestones
    │   └── overlay-crown.webp       # grows with: combined team record
    └── pandemonium/
        └── (same 6 files)
```

---

## Architectural Patterns

### Pattern 1: Extract-then-Compose (the SoloSurfaces template)

**What:** Each room's page body is extracted into a `<Room>Surfaces` component that compiles and runs independently. The room page (`<Room>Room`) composes it over `AtmosphereRoot`. The flat legacy route (`/draft`, `/stats`, etc.) can continue to host the same surfaces component over flat chrome — ensuring no regression while the room layer matures.

**When to use:** Every room. This is the established pattern from `solo_dashboard.rs` → `solo_journey.rs`. Never inline existing page logic directly inside the room file.

**Trade-offs:** Slight indirection (one extra component boundary), but it means the room layer is a pure composition with zero risk to the existing flat surfaces. The flat routes remain functional fallbacks during art production.

**Integration points:**
- `SoloDashboardPage` (`/solo`) hosts `SoloSurfaces` over flat chrome — already done.
- `SoloJourneyRoom` (`/journey`) hosts `SoloSurfaces` over `AtmosphereRoot` — already done.
- The same split must be repeated: `/draft`, `/tree-drafter`, `/game-plan`, `/opponents`, `/action-items` → extracted into `WarSurfaces`; `/war-room` hosts `WarSurfaces` over atmosphere. Existing flat routes remain in `app.rs` unchanged.
- `/stats`, `/match/:id`, `/analytics`, `/post-game`, `/personal-learnings` → extracted into `LibSurfaces`; `/library` hosts over atmosphere.
- `/team/dashboard`, `/solo`, `/profile` → extracted into `ThroneSurfaces`; `/throne` hosts over atmosphere.

**Region prop rule (Phase 18.1/18.2 hard-won lesson):** `InitialTheme` is read ONCE at the room entry (`<Room>Room`) and passed as a `region: String` prop to `<Room>Surfaces`. It must never be read inside `Plane`, `GrowthOverlay`, or any reactive closure — reading a context inside a reactive closure or during hydration caused SSR/hydrate hash mismatches (pandemonium hydration panic). Static markup + reactive `data-*` attrs only.

### Pattern 2: Intra-Room Mode Toggle via RwSignal (not sub-routes)

**What:** Multi-surface rooms (War Room, Library) use a local `RwSignal<ActiveSurface>` inside the `<Room>Surfaces` component to switch between their hosted pages. This is NOT sub-routing — the URL stays at `/war-room` regardless of which surface (Draft, Tree, Game Plan, Opponents, Action Items) is active.

**When to use:** War Room (5 surfaces), Library (5 surfaces). Throne may not need it if surfaces are presented as panels on one screen rather than tab-switched views.

**Why not sub-routes:** Sub-routes inside a room would require `<Route>` nesting and `<Outlet>` inside `AtmosphereRoot`, which adds Leptos router complexity. More critically, sub-routes re-mount the atmosphere (the parallax listener and all planes) on every navigation — destroying the immersive continuity the rooms exist to create. The existing flat sub-nav already handles cross-room navigation; within-room switching should be zero-URL-change.

**QuickNav integration:** QuickNav's `HUB_ROUTES` already lists `/draft`, `/tree-drafter`, etc. as individual destinations. These remain valid — they navigate to the flat surface at those URLs. The room destinations (`/war-room`, `/library`) should be added to `HUB_ROUTES` as hub-level entries so Cmd+K can also land the user in the room. Both coexist.

**Concrete shape for WarSurfaces:**

```rust
#[derive(Clone, Copy, PartialEq)]
enum WarSurface { Draft, Tree, GamePlan, Opponents, BanPriority }

#[component]
pub fn WarSurfaces(region: String) -> impl IntoView {
    let active = RwSignal::new(WarSurface::Draft);

    view! {
        // Intra-room tab bar (not the global sub-nav)
        <nav class="flex gap-2 mb-6" aria-label="War Room surfaces"
             role="tablist">
            <button role="tab" aria-selected=move || active.get() == WarSurface::Draft
                    on:click=move |_| active.set(WarSurface::Draft)>
                "Draft"
            </button>
            // … other tabs …
        </nav>

        // <Show> mounts/unmounts on surface switch — safe because each surface
        // is SSR-stable (no structural branching inside them on region).
        <Show when=move || active.get() == WarSurface::Draft>
            <DraftPageContent region=region.clone() />
        </Show>
        // … etc …
    }
}
```

**Show vs display:none trade-off:** `<Show>` mounts/unmounts the child when the predicate flips. This is safe (each surface is SSR-stable) and avoids loading all surface data up front. Use `<Show>` unless profiling shows mount-time cost is noticeable.

**Accessibility:** The intra-room tab bar must use `role="tablist"` / `role="tab"` / `aria-selected` / `aria-controls` so screen readers treat it as a tab panel group, distinct from the global nav.

### Pattern 3: Team-Scoped Growth Extension

**What:** A parallel `TeamGrowthCounts` / `TeamGrowthState` type and `get_team_growth_counts(db, team_id)` function, mirroring the existing user-scoped pair exactly. The pure tier math in `tier_for()` is reused with different threshold choices; a separate `TeamGrowthState::from_counts` function with team-appropriate thresholds is added.

**When to use:** War Room and Library rooms. Their growth axes are driven by team-scoped tables (`draft`, `game_plan`, `post_game_learning`, `opponent`, `action_item`), not user-scoped ones.

**Concrete additions to `src/models/growth.rs`:**

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamGrowthCounts {
    pub draft: u32,              // War Room: tactics axis
    pub opponent: u32,           // War Room: intel axis
    pub ban_priority: u32,       // War Room: command axis
    pub post_game_learning: u32, // Library: records axis
    pub team_note: u32,          // Library: tomes axis (team notebook entries)
    pub player_match: u32,       // Library: analysis axis (proxied across team members)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamGrowthState {
    pub counts: TeamGrowthCounts,
    pub tactics_tier: u8,    // draft count
    pub intel_tier: u8,      // opponent count
    pub command_tier: u8,    // ban_priority count
    pub records_tier: u8,    // post_game_learning count
    pub tomes_tier: u8,      // team_note count
    pub analysis_tier: u8,   // player_match count
}

impl TeamGrowthState {
    pub fn from_counts(counts: TeamGrowthCounts) -> Self {
        // War Room axes — teams produce fewer but higher-value items than solo
        let tactics  = tier_for(counts.draft,              [1,  5, 20]);
        let intel    = tier_for(counts.opponent,           [1,  3, 10]);
        let command  = tier_for(counts.ban_priority,       [1,  5, 15]);
        // Library axes
        let records  = tier_for(counts.post_game_learning, [1,  5, 20]);
        let tomes    = tier_for(counts.team_note,          [1,  8, 30]);
        let analysis = tier_for(counts.player_match,       [1, 10, 50]);
        TeamGrowthState {
            counts,
            tactics_tier:  tactics.level(),
            intel_tier:    intel.level(),
            command_tier:  command.level(),
            records_tier:  records.level(),
            tomes_tier:    tomes.level(),
            analysis_tier: analysis.level(),
        }
    }
}

impl Default for TeamGrowthState {
    fn default() -> Self {
        TeamGrowthState::from_counts(TeamGrowthCounts::default())
    }
}
```

**Concrete addition to `src/server/db.rs`:**

```rust
pub async fn get_team_growth_counts(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<TeamGrowthCounts> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    let mut result = db
        .query(
            "SELECT count() as n FROM draft
               WHERE team = type::record('team', $team_key) GROUP ALL; \
             SELECT count() as n FROM opponent
               WHERE team = type::record('team', $team_key) GROUP ALL; \
             SELECT count() as n FROM ban_priority
               WHERE team = type::record('team', $team_key) GROUP ALL; \
             SELECT count() as n FROM post_game_learning
               WHERE team = type::record('team', $team_key) GROUP ALL; \
             SELECT count() as n FROM team_note
               WHERE team = type::record('team', $team_key) GROUP ALL; \
             SELECT count() as n FROM player_match
               WHERE user IN (SELECT VALUE user FROM team_member
                 WHERE team = type::record('team', $team_key)) GROUP ALL",
        )
        .bind(("team_key", team_key))
        .await?;

    // … take(0..5) pattern identical to get_growth_counts …
    // fn count_of local helper same as in get_growth_counts
    Ok(TeamGrowthCounts {
        draft:              count_of(result.take(0).unwrap_or_default()),
        opponent:           count_of(result.take(1).unwrap_or_default()),
        ban_priority:       count_of(result.take(2).unwrap_or_default()),
        post_game_learning: count_of(result.take(3).unwrap_or_default()),
        team_note:          count_of(result.take(4).unwrap_or_default()),
        player_match:       count_of(result.take(5).unwrap_or_default()),
    })
}
```

Note: `ban_priority` table name must be confirmed against `schema.surql` before writing; adjust if the actual table name differs. The `player_match` proxy aggregates all team members' match counts — deliberately approximate. A future phase can replace this with a team-specific analytics proxy.

**Server function in the room page file (e.g. `src/pages/rooms/war_room.rs`):**

```rust
#[server]
pub async fn get_team_growth_state() -> Result<TeamGrowthState, ServerFnError> {
    use crate::models::growth::TeamGrowthState;
    use crate::server::{auth::AuthSession, db};
    // … extract auth + surreal …

    let team_id = match db::get_user_team_id(&surreal, &user.id).await? {
        Some(id) => id,
        None => return Ok(TeamGrowthState::default()), // no team → all Seed (rule 44)
    };
    let counts = db::get_team_growth_counts(&surreal, &team_id)
        .await
        .unwrap_or_default();
    Ok(TeamGrowthState::from_counts(counts))
}
```

**`AtmosphereRoot` prop additions:** The existing `AtmosphereRoot` takes `lib_tier`, `armory_tier`, `banner_tier` signals hardcoded. For new rooms, the axis names differ. The cleanest extension is to add new `#[prop(optional, into)]` fields with defaults of `Signal::derive(|| 0u8)` for each new axis, and write them as `data-*-tier` attributes on the root element. The CSS rules in `input.css` key off these attribute names. Document the three sets of axis names:

| Room | Axis A | Axis B | Axis C | data-* attributes written |
|------|--------|--------|--------|--------------------------|
| Solo Journey | lib | armory | banner | `data-lib-tier`, `data-armory-tier`, `data-banner-tier` |
| War Room | tactics | intel | command | `data-tactics-tier`, `data-intel-tier`, `data-command-tier` |
| Library | records | tomes | analysis | `data-records-tier`, `data-tomes-tier`, `data-analysis-tier` |
| Throne | roster | progression | crown | `data-roster-tier`, `data-progression-tier`, `data-crown-tier` |

### Pattern 4: Art Asset Pipeline (FLUX Layer Separation)

**What:** Each room's art is produced as layer-separated FLUX output: sky/horizon (depth 6), mid (depth 16), near (depth 32), plus N growth overlay layers. Each region produces its own set. Only the active region's assets are loaded.

**Naming convention** (confirmed from `public/img/rooms/solo-journey/`):

```
public/img/rooms/<room-slug>/<region>/plane-sky.webp
public/img/rooms/<room-slug>/<region>/plane-mid.webp
public/img/rooms/<room-slug>/<region>/plane-near.webp
public/img/rooms/<room-slug>/<region>/overlay-<axis>.webp   (one per growth axis)
```

- `<room-slug>`: `war-room`, `library`, `throne`
- `<region>`: `demacia`, `pandemonium`
- `<axis>`: matches the room's tier axis names (`tactics`, `intel`, `command` for War Room etc.)

**FLUX layer production brief for AI-IMAGES.md:**

Each plane must be produced with a transparent-edge webp so the layers composite correctly in CSS. The depth planes are full-bleed (`object-cover`); growth overlays are additive atmospheric elements. The key constraint: the mid plane, near plane, and overlays must have transparent backgrounds — they are composited over lower planes in CSS stacking order, not in a compositing tool. Art production must deliver:

1. `plane-sky.webp` — solid background (no transparency needed), full scene environment
2. `plane-mid.webp` — transparent background, mid-ground elements only
3. `plane-near.webp` — transparent background, foreground frame elements only
4. `overlay-<axis>.webp` × 3 — transparent background, atmosphere accent elements keyed to a growth axis

**Loading strategy — active-region-only:**

The room page constructs the asset dir as:

```rust
let dir = format!("/img/rooms/war-room/{region}");
// only demacia OR pandemonium assets ever rendered in the DOM
```

This matches `solo_journey.rs` line 46. The 400 KB budget is enforced by never rendering the inactive region's `<img>` tags at all (not CSS-hiding them — `loading="lazy"` does not prevent fetch for DOM-present elements, as the `parallax.rs` doc comment warns at line 163).

**Per-room budget targets:**

| Asset | Target size |
|-------|-------------|
| `plane-sky.webp` | ≤ 80 KB |
| `plane-mid.webp` | ≤ 80 KB |
| `plane-near.webp` | ≤ 60 KB |
| Each `overlay-*.webp` | ≤ 30 KB |
| Total per room × region | ≤ 400 KB |

All webp, browser-optimized. Art production must deliver optimized webp, not raw PNG exports.

---

## Data Flow

### Room Entry Flow

```
Route change to /war-room
    ↓
WarRoomRoom component mounts
    ↓
use_context::<InitialTheme>() → region: String (read ONCE, not reactive)
    ↓
Resource::new(|| (), |_| get_team_growth_state()) → TeamGrowthState
    ↓
Signal::derive() × 3 → tactics_tier, intel_tier, command_tier signals
    ↓
AtmosphereRoot(tier signals, class) {
    Plane × 3  [src = /img/rooms/war-room/{region}/plane-*.webp]
    GrowthOverlay × 3  [data-*-tier drives CSS opacity in input.css]
    ContrastScrim  [radial gradient at z-8]
    <div z-10>
        WarSurfaces(region=region)
    </div>
}
    ↓ (hydrate only)
Effect::new → mousemove listener → RAF → --px/--py on #room-atmosphere-root
    ↓ (CSS)
Planes: transform: translate3d(calc(var(--px) * var(--depth) * 1px), …)
Overlays: opacity driven by data-tactics-tier=… CSS rules in input.css
```

### Intra-Room Surface Switch (War Room)

```
User clicks "Opponents" tab in WarSurfaces
    ↓
active.set(WarSurface::Opponents)  [RwSignal<WarSurface>]
    ↓
<Show when=move || active.get() == WarSurface::Opponents> mounts OpponentsContent
<Show when=move || active.get() == WarSurface::Draft> unmounts DraftContent
    ↓
AtmosphereRoot stays mounted — parallax listener and CSS vars NOT interrupted
URL does not change — stays at /war-room
    ↓
OpponentsContent fetches its resource on first mount (lazy, via its own Resource)
```

### Team Growth State Flow

```
get_team_growth_state() [server fn, called once per room entry]
    ↓
AuthSession → user.id
    ↓
db::get_user_team_id(db, user_id) → Some(team_id) or None→Ok(default Seed)
    ↓
db::get_team_growth_counts(db, team_id)
    batched 6-statement query in single round-trip (mirrors get_growth_counts)
    ↓
TeamGrowthState::from_counts(counts)  [pure, no DB]
    ↓
Serialized across server fn boundary → WASM
    ↓
Signal::derive() per axis → AtmosphereRoot data-*-tier props → CSS
```

---

## Accessibility Integration Points (Per Room — Mandatory Checklist)

These must be applied for every new room. They are not DRY-able at the component level — each room component is a separate mount point that must be independently verified.

### A-1: Reduced-Motion Gate (double gate — inherited, verify not broken)

`AtmosphereRoot` already gates the mousemove listener (`prefers-reduced-motion` check in the `Effect`, `parallax.rs` lines 60–68) and `input.css` zeroes the transforms via `@media`. Both gates are inherited automatically because all rooms use the same `AtmosphereRoot`. No per-room code change needed.

Verify per room: `input.css` media rule covers `.room-plane` and `.room-overlay` shared class names (not room-specific selectors), so all new rooms inherit the CSS gate without new CSS.

### A-2: Focus Rings Over Art

Every interactive element within `<Room>Surfaces>` must carry `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none`. The contrast scrim (z-8 radial gradient) ensures the art background darkens enough for focus rings to meet AA. Check: no `outline:none` without replacement inside `<Room>Surfaces` content.

### A-3: AA Contrast Scrim

The `AtmosphereRoot` renders a fixed contrast scrim at z-8:

```html
<div style="z-index:8; background: radial-gradient(ellipse at center, transparent 0%, var(--color-base) 95%);" aria-hidden="true" />
```

Copied from `solo_journey.rs` lines 68–73. Each new room file must include the same scrim. Without it, body text over mid-tone parallax art can drop below 4.5:1 AA. The scrim value (95% `--color-base` at edges) is sufficient for text on the surface layer at z-10.

### A-4: Decorative Image Attributes

`Plane` and `GrowthOverlay` already set `alt=""` and `aria-hidden="true"`. Verify any additional decorative images added in `<Room>Surfaces` carry these attributes.

### A-5: Intra-Room Tab Accessibility

The intra-room tab bar must use `role="tablist"` on the container, `role="tab"` and `aria-selected` on each button, and `aria-controls` pointing to a `role="tabpanel"` wrapper for the active surface. This makes the within-room navigation distinct from the global hub nav for screen readers.

### A-6: Hydration-No-Panic Canary Test (per route)

The Phase 18.2 lesson: SSR/hydrate hash mismatches in structural branching cause `Option::unwrap()` panics in tachys at `html/mod.rs:217`, killing all WASM event handlers silently.

Canary test protocol per room route:
1. Load the room route in a browser with WASM active.
2. Attempt a region-reactive interaction (theme toggle, mode toggle, or intra-room tab switch).
3. Working interaction = clean hydration. Silent no-op = check browser console for `panicked at … Option::unwrap()`.
4. Run for BOTH `demacia` and `pandemonium` — mismatches only surface under the non-default region.

Prevention: never branch on `region` (or any context value) structurally inside any component rendered during SSR. `region` may only drive:
- String interpolation into `src` or `href` attributes (safe — attribute value, not structure)
- `data-*` attribute values (safe — same markup shape in both SSR and WASM)
- Being passed as a prop to a child component (safe — serializes across boundary)

Add to `e2e/tests/pages.spec.ts` `AUTHED_PAGES`: `/war-room`, `/library`, `/throne` for basic smoke coverage. Add per-room interaction tests that click a surface tab and verify the surface changes.

---

## Suggested Build Order (Dependency-Ordered)

```
PHASE A: Growth Model Extension  (1–2 plans)
─────────────────────────────────────────────
  + Add TeamGrowthCounts, TeamGrowthState to src/models/growth.rs
  + Add get_team_growth_counts to src/server/db.rs
  + Unit tests (mirror existing growth.rs test suite)
  + Confirm ban_priority / team_note table names in schema.surql
  No UI work — model + DB layer only.

        │
        │ (unblocks all room phases)
        ▼

PHASE B: AtmosphereRoot Axis Generalization  (1 plan)
──────────────────────────────────────────────────────
  + Add optional tier props to AtmosphereRoot for new axis names
  + Add new data-*-tier CSS rules in input.css per room × axis
  + Solo Journey room: regression test (no change expected)

        │
        │ (B completes; C, D, E can run in parallel)
        ▼

PHASE C: War Room         PHASE D: Library          PHASE E: Throne
─────────────────────     ─────────────────────     ──────────────────
  (3–4 plans)               (3–4 plans)               (2–3 plans)
  + Extract WarSurfaces     + Extract LibSurfaces     + Extract ThroneSurfaces
  + Intra-room tab toggle   + Intra-room tab toggle   + Compose over atmosphere
  + Compose over AtmRoot    + Compose over AtmRoot    + get_team_growth_state
  + get_team_growth_state   + get_team_growth_state   + Canary test A6
  + Placeholder art         + Placeholder art         + Placeholder art
  + Canary test A6          + Canary test A6
  + e2e smoke test          + e2e smoke test          + e2e smoke test

        │
        ▼

PHASE F: Art Production  (1 extended phase, overlaps with C/D/E in calendar time)
──────────────────────────────────────────────────────────────────────────────────
  + FLUX layer-separated production for 3 rooms × 2 regions × 6 assets = 36 files
  + Deliver optimized webp; verify ≤400 KB budget per room per region
  + Swap placeholder assets in; no code change needed (path convention)
  + Canary + e2e visual baselines per room

        │
        ▼

PHASE G: QuickNav + HUB_ROUTES Update  (1 thin plan)
─────────────────────────────────────────────────────
  + Add /war-room, /library, /throne to HUB_ROUTES
  + Update hub_for_path() for new room paths (suppress sub-nav inside rooms)
  + Optionally redirect hub primary buttons from /draft, /stats to rooms

        │
        ▼

Re-incorporated Launch Phases (existing, unchanged)
────────────────────────────────────────────────────
  Phase 20: Production Hardening
  Phase 20.1: Closed-Beta Access Gate
  Phase 21: Deploy Infra
  Phase 22: Compliance & Transparency
  Phase 23: Pre-Launch Full Review
  Phase 24: Soft Launch + Feedback Loop
```

**Dependency rationale:**

- Phase A (growth model) must precede all room phases because `get_team_growth_state()` calls `get_team_growth_counts()`.
- Phase B (`AtmosphereRoot` axis generalization) must precede room phases because room files pass room-specific tier signals to `AtmosphereRoot`; the current fixed prop names (`lib_tier`, `armory_tier`, `banner_tier`) must be generalized first.
- Phases C, D, E can run in parallel once A and B are done. War Room is the highest-value room (most-used features) and should be built first if serialized.
- Phase F (art production) is decoupled from coding. Rooms should ship with placeholder flat-color or gradient webp assets so the room system is verifiable before real art arrives. Art production can overlap with Phases C/D/E in calendar time, delivering to Phase F as a swap-in — no code change required.
- Phase G (QuickNav/nav update) is small and can happen alongside any room phase or as its own thin plan.
- Launch phases (20–24) depend on all room phases being functionally complete and art-integrated; do not start Phase 20 until Phase G closes.

---

## Anti-Patterns

### Anti-Pattern 1: Context Read Inside Reactive Primitive

**What people do:** `let region = use_context::<InitialTheme>()` inside a `Signal::derive()` closure, `move || …` view block, or inside a child component that `AtmosphereRoot` or `<Room>Surfaces` instantiates.

**Why it's wrong:** Causes SSR/hydrate structural mismatch (Phase 18.2 incident). SSR computes the value from the per-request context; WASM hydration may read a different value or panic on missing context. The mismatch produces a tachys `Option::unwrap()` panic at `html/mod.rs:217` that silently kills all WASM event handlers.

**Do this instead:** Read `InitialTheme` ONCE at the room component's top level (`SoloJourneyRoom`, `WarRoomRoom`, etc.) into a local `String`. Pass it as a `region: String` prop to all child components. Children use the prop, not a context lookup.

### Anti-Pattern 2: CSS-Hiding the Inactive Region

**What people do:** Render both regions' `<Plane>` and `<GrowthOverlay>` sets and toggle `display:none` on the inactive one.

**Why it's wrong:** `loading="lazy"` does not prevent fetch when the element is in the DOM with `display:none`. Both regions' 6-image sets (~800 KB) load, blowing the 400 KB budget. The parallax.rs doc comment (line 163) explicitly warns against this.

**Do this instead:** Only render the active region's `<img>` elements via the `format!("{dir}/plane-sky.webp")` pattern. Region is fixed at room entry — it never needs to change reactively within the room.

### Anti-Pattern 3: Sub-Routing Inside the Room

**What people do:** Define `<Route path="/war-room/draft">` and `<Route path="/war-room/opponents">` as child routes of a `/war-room` layout route, using `<Outlet>` inside `AtmosphereRoot`.

**Why it's wrong:** Every sub-route navigation re-evaluates the parent layout, which re-mounts `AtmosphereRoot` in practice (destroying the parallax listener). The atmosphere flashes on every intra-room navigation, destroying immersive continuity. Leptos nested routing with `<Outlet>` inside non-`<Router>` elements has also been a source of hydration instability.

**Do this instead:** Use `RwSignal<ActiveSurface>` + `<Show>` inside `<Room>Surfaces`. The URL stays at `/war-room`; the atmosphere never re-mounts. QuickNav's flat-route entries (`/draft`, `/opponents`) remain valid deep-link targets.

### Anti-Pattern 4: Mixing User and Team Growth in One Query

**What people do:** Add team-scoped table counts to the existing `get_growth_counts` function to avoid creating a new one.

**Why it's wrong:** `get_growth_counts` is user-scoped by design — called by Solo Journey for a single user. Team tables require a `team_id` from `get_user_team_id`. Mixing scopes in one function creates mixed concerns, complicates the signature, and forces the Solo Journey room to incidentally load team data it doesn't use.

**Do this instead:** Keep `get_growth_counts(db, user_id)` as-is. Add `get_team_growth_counts(db, team_id)` as a parallel function. Each room's server function resolves its own growth state independently.

---

## Integration Points

### Existing Room System (unchanged)

| Component | Integration Contract | Notes |
|-----------|---------------------|-------|
| `AtmosphereRoot` | Receives tier signals as `Signal<u8>` props; writes `data-*-tier` on root element | Add optional props for new axis names; default to `Signal::derive(\|\| 0u8)` |
| `Plane` | Receives `src: String`, `depth: i32`, `z: i32`; no behavior change | Unchanged |
| `GrowthOverlay` | Receives `axis: String`, `src: String`, `z: i32`; CSS class `room-overlay--{axis}` must exist in `input.css` | New axes need new CSS rules |
| `QuickNav` | Reads `HUB_ROUTES`; add room-level entries | `hub_label()` match must be extended for new hub keys |

### Flat Routes (preserved, not removed)

All existing flat routes (`/draft`, `/stats`, `/team/dashboard`, etc.) remain in `app.rs` and remain functional. Rooms are additive new routes. Users navigate to rooms via hub primary buttons or QuickNav; flat routes remain valid deep-link targets and fallbacks.

### nav.rs Changes

When a user is on `/war-room`, `/library`, or `/throne`, the existing sub-nav strip should suppress itself (`hub_for_path` returns `""` for these paths). The room provides its own intra-room tab navigation. Add these paths to `hub_for_path` explicitly, returning `""` or a new hub key with empty sub-routes.

Consider updating the hub primary button `<A href=...>` targets from `/draft` and `/stats` to `/war-room` and `/library` — the flat routes remain but the primary nav entry points to rooms.

### SurrealDB Schema (verify before Phase A)

Before adding `get_team_growth_counts`, confirm these table names in `schema.surql`:
- `ban_priority` — confirm exact name (may differ from this guess)
- `team_note` — for library tomes axis (team notebook)
- `post_game_learning` — confirmed via `get_dashboard_summary` usage
- `draft`, `opponent` — confirmed via existing team-scoped query patterns

The `player_match` proxy joins via `team_member`; confirm `team_member.user` and `player_match.user` use matching `RecordId` types (both confirmed from existing DB query patterns).

---

## Sources

- `src/components/room/parallax.rs` — AtmosphereRoot implementation, doc comment budget warning (lines 1–201)
- `src/components/room/quick_nav.rs` — QuickNav palette, HUB_ROUTES reuse (lines 1–124)
- `src/models/growth.rs` — GrowthState, tier math, unit tests (lines 1–171)
- `src/pages/rooms/solo_journey.rs` — composition pattern, region-read-once, asset dir convention (lines 1–81)
- `src/pages/solo_dashboard.rs` — SoloSurfaces extraction, get_growth_state server fn (lines 1–90)
- `src/server/db.rs` — get_growth_counts (lines 4105–4149), get_dashboard_summary (lines 4005–4103), team_id strip patterns throughout
- `src/components/nav.rs` — HUB_ROUTES (lines 160–193), hub_for_path (lines 198–225), sub-nav strip (lines 521–557)
- `src/app.rs` — Route table, InitialTheme context, QuickNav placement (lines 1–165)
- `public/img/rooms/` — confirmed naming convention from solo-journey asset tree (12 files)
- `.planning/PROJECT.md` — v1.4 milestone goals, room names, re-incorporated launch phases
- `.planning/ROADMAP.md` — Phase 18.1/18.2 hydration lesson, launch phase sequence (Phases 20–24)
- `.claude/rules/leptos-patterns.md` — rules 8, 19, 22, 24, 44 (auth, signals, server fns, empty state handling)
- `.claude/rules/wasm-patterns.md` — rules 35, 36, 37 (WASM safety, closure patterns)
- `.claude/rules/surreal-patterns.md` — rules 1, 2, 28, 29 (type::record, strip prefix, batched queries)

---

*Architecture research for: v1.4 Immersive Companion — multi-room extension*
*Researched: 2026-06-03*
