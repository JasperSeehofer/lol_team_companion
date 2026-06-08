# Phase 29: Throne, QuickNav & Room A11Y - Research

**Researched:** 2026-06-08
**Domain:** Leptos 0.8 room system — third RoomSurface consumer (Throne), blended team+personal growth, WCAG-2.1 QuickNav focus trap, app-wide a11y gate
**Confidence:** HIGH (all structural findings verified directly from codebase with file:line)

---

## Summary

Phase 29 is the **third instance** of a proven pattern. War Room (Phase 27, `war_room.rs`) and Library (Phase 28, `library.rs`) shipped the same structure: read region once → fetch a growth `Resource` → derive per-axis `Signal<u8>` → compose `AtmosphereRoot` + `Plane`×3 + `GrowthOverlay`×3 + contrast scrim + a `RoomSurface` tab strip floating at `z-10`. The Throne (`/throne` → `ThroneRoom` + `ThroneSurfaces`) copies this verbatim. The design is LOCKED by CONTEXT.md (D-01..D-15); this research verifies ground truth so the planner can write precise tasks.

Three things make Phase 29 harder than Phase 28, and the planner must budget for them:

1. **`team/dashboard.rs` is 2286 lines** and is NOT a clean single-surface page like the Library's five sources. It contains a `ModeToggle` (Dashboard / Game Day Brief), a `is_solo_mode` branch, a full Demacia management UI (`TeamDashboard` inline), a region-branched `PandemoniumTeamDashboard`, a `TeamGameDayBriefView`, AND the `TeamNotebook` component (`:1526`). Splitting this into three surfaces (Dashboard / Roster / Development) is the single largest extraction risk of the milestone.

2. **Blended growth is genuinely new.** Unlike War Room/Library (which early-return `Default` for no-team), `get_throne_growth_state` must compute the personal `progression` axis from the user's `ranked_snapshot` even with no team (D-07). The `roster`/`progression`/`crown` axis types do NOT yet exist in `growth.rs` (only their names are reserved in a comment). The `progression` source is user-scoped (`ranked_snapshot`), the `roster`/`crown` sources are team-scoped — so `ThroneGrowthState` cannot be derived from a single `*Counts` struct the way the others are.

3. **QuickNav focus trap + filter is the only net-new UI mechanism this phase**, and there is **zero existing focus-trap/`inert`/`activeElement` code in the codebase to copy** (verified by grep). It must be written from scratch following WCAG 2.1 SC 2.1.2.

**Two latent blockers found that will silently fail SC1 if not addressed in Wave 0:**

- **`input.css` has NO `roster`/`progression`/`crown` tier→opacity selectors** (verified lines 436-464 — only lib/armory/banner/tactics/intel/command/records/tomes/analysis exist). `.room-overlay` defaults to `opacity:0` (input.css:431). Without the three new selectors the Throne overlays render at `opacity:0` at every tier — the room can never visibly grow, and SC1's data-tier assertion would pass while the visual silently fails. This is the exact "room growth visibility gap" that bit Phase 27 (see MEMORY.md `project_room_growth_visibility_gap.md`).
- **Placeholder gradient WebP assets** at `/img/rooms/throne/{region}/` do not exist yet (Phase 30 ships real art). The planner must create placeholder gradients at those paths or the `<Plane>`/`<GrowthOverlay>` `src` 404s.

**Primary recommendation:** Copy `library.rs` (the closer sibling — it already has the IA-healing redirect shells) as the structural template for `throne.rs`. Add `ThroneGrowthCounts` + `ThroneGrowthState` to `growth.rs` with a **user-scoped `progression` field** distinct from the team-scoped fields. Extract `IdentitySurface` from `profile.rs`; split `team/dashboard.rs` into `DashboardSurface`/`RosterSurface`/`DevelopmentSurface`; fold `action_items.rs` + `TeamNotebook` into `DevelopmentSurface`; reuse `SoloSurfaces` verbatim for the Solo tab. Add the three CSS selectors and the three new e2e canary/contrast/focus-trap tests in Wave 0.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Throne surfaces (D-01..D-03):** Exactly five surfaces, in order: **Identity / Dashboard / Roster / Development / Solo.**
- Identity ← `/profile` content (Riot link, champion-pool summary, team crest/banner).
- Dashboard ← team dashboard *summary* (prep priorities + at-a-glance panels).
- Roster ← roster slots, coach slots, join requests.
- Development ← **Action Items** (substance) + **Team Notebook** (supporting), composed together. Team learnings stay in the Library; Development **deep-links** to them, does NOT re-host. **No new data models or capability.**
- Solo ← the existing `SoloSurfaces`, reused verbatim.
- **D-02: Always show all five tabs.** No-team user gets the existing empty-state CTA inside the team-scoped tabs (Dashboard/Roster/Development) — NOT conditional tab hiding (avoids rule #58 hydration trap). No per-tab conditional rendering keyed on team/mode.

**Throne growth axes (D-04..D-08):** Three axes — `roster` (team_member count, team-scoped), `progression` (viewing user's ranked LP/rank, **user-scoped**), `crown` (team record from post-game win/loss, team-scoped).
- **D-05: Independent blended axes.** First room that blends team + personal data and does NOT go fully tier-0 for a teamless user. No-team → `roster`/`crown` tier 0 but `progression` still grows from personal LP. No-rank → `progression` tier 0 but team axes grow.
- **D-06:** `crown` uses cheap team-scoped post-game win/loss, NOT the Phase 26-deferred `team_member→user→player_match` join.
- **D-07:** `get_throne_growth_state` must NOT early-return `TeamGrowthState::default()` on no-team. Compute `progression` from authenticated user's ranked data regardless of team. Never `Err` on absent data (rule #44).
- **D-08:** Environmental art only — no tier badges/counts/panels. SC1 verified via `data-roster-tier`/`data-progression-tier`/`data-crown-tier` attributes + overlay opacity (seed high-tier vs fresh, compare).

**QuickNav (D-09..D-11):**
- **D-09:** Filterable command palette — text input at top, auto-focused on open, substring-on-label filtering. (substring is sufficient; fuzzy deferred.)
- **D-10:** Focus trap + focus-restore (WCAG 2.1 SC 2.1.2 — closed-beta blocker). Tab must NOT escape the open overlay. On Escape/any close, restore focus to the element that triggered open (capture `document.activeElement` on open). Preserve existing `focus-visible` rings (guardrail G-12, no `outline:none` without replacement).
- **D-11:** Restructure shared `HUB_ROUTES` (`nav.rs:160`) room-centric — three room groups (War Room / Library / Throne), each listing surfaces via `?tab=` deep-links. Updates BOTH QuickNav AND the top sub-nav bar. Remove old flat-route links that now 302-redirect. No dead/duplicate entries.

**IA-healing (D-12..D-13):**
- **D-12:** 302-redirect `/profile` → `/throne?tab=identity`, `/team/dashboard` → `/throne?tab=dashboard`, `/action-items` → `/throne?tab=development`. Reuse `RoomSurface`'s `?tab=` resolver; unknown/missing tab → first surface (Identity). **Preserve param-carrying / create sub-routes** (`/profile/...`, `/team/...`) — must not 404 or lose entry point. Planner audits `app.rs` first.
- **D-13:** `/journey` and `/throne` coexist — do NOT redirect `/journey`.

**Accessibility (D-14..D-15):**
- **D-14:** Every room (`/war-room`, `/library`, `/throne`) passes the per-route hydration-no-panic canary in **both** demacia and pandemonium. `/throne` added to `hydration-no-panic.spec.ts`. Region read ONCE at `ThroneRoom` top → plain `region: String` prop (rule #58).
- **D-15:** WCAG AA (4.5:1) text contrast over the art via the z-8 contrast scrim, verified per room. Mirror `solo_journey.rs:69` radial-gradient scrim. `focus-visible` rings preserved.

### Claude's Discretion
- Internal module layout for the five `*Surface` components; how `team/dashboard.rs` splits cleanly.
- Shared-state mechanism (if any) between Throne surfaces — hard constraint: no structural hydration branch (rule #58).
- Asset path: mirror `/img/rooms/throne/{region}/...`; placeholder gradient WebPs.
- Plane/overlay count + parallax depths (mirror war_room/solo_journey: 3 planes + 3 axis overlays).
- Exact `roster`/`progression`/`crown` count sources + tier thresholds (calibration hypotheses, tunable post-beta).
- `get_throne_growth_state` placement + no-team/no-rank handling shape.
- Filter-match algorithm (substring on label sufficient) + overlay layout.
- Redirect mechanism details for D-12 (which sub-routes need preservation).
- Whether `team_note` needs extraction beyond moving its embedded block (`team/dashboard.rs:1526`).

### Deferred Ideas (OUT OF SCOPE)
- Structured player-development / training tracker (Development composes existing only).
- Retire/reconcile `/journey` into the Throne Solo tab.
- Per-member ranked-win team record (revive `team_member→user→player_match`).
- Fuzzy/command-style QuickNav matching.
- Eager-fetch optimization for the five mounted surfaces.
- Real FLUX environment art (Phase 30).
- Conditional/adaptive tab visibility by solo/team mode (rejected — hydration-branch risk).
- War Room flat-route cleanup (`/draft`, `/tree-drafter`, `/game-plan` stay Phase 27 thin shells).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ROOM-06 | Throne / Home room hosts identity, team dashboard, and solo progression | Five extraction targets located (`profile.rs`, `team/dashboard.rs` ×3, reused `SoloSurfaces`); `RoomSurface` API confirmed unchanged; Throne axis props (`roster_tier`/`progression_tier`/`crown_tier`) already declared in `parallax.rs:75-77`; `GrowthOverlay` already documents the three axis keys (`parallax.rs:386-388`) |
| ROOM-08 | QuickNav command palette app-wide with proper focus trap; nav IA reflects rooms | `QuickNav` already mounted at `app.rs:136`, Cmd/Ctrl+K + Escape working (`quick_nav.rs:31-55`); `HUB_ROUTES` single source (`nav.rs:160`) feeds both QuickNav and sub-nav (`nav.rs:527`); focus-trap is net-new (no existing precedent — grep clean) |
| A11Y-02 | Every room passes hydration-no-panic canary both regions, keeps focus-visible rings, holds WCAG AA contrast over art | Canary spec `hydration-no-panic.spec.ts` has `SCOPED_ROUTES` (currently includes `/journey`, `/library`; add `/throne`); rule #58 structural guard documented; z-8 scrim pattern at `solo_journey.rs:69`; `focus-visible` global rule at `input.css:374-375` |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Throne room shell + parallax | Frontend Server (SSR) | Browser (mousemove RAF) | `AtmosphereRoot` renders static HTML on SSR; WASM attaches single `mousemove` listener at hydration (`parallax.rs:108`) |
| Growth axis tiers (3) | API / Backend (`#[server]`) | Frontend (`Signal::derive`) | `get_throne_growth_state` fetches counts; tier signals are client-side `Signal::derive` wraps over the `Resource` |
| `progression` axis source | API / Backend (`#[server]`) | — | **User-scoped** `ranked_snapshot` (`db.rs:5007`), NOT team-scoped — the divergence from War/Library |
| `roster` + `crown` axis sources | API / Backend (`#[server]`) | — | Team-scoped: `team_member` count + `post_game_learning.win_loss` |
| Tab routing (`?tab=`) | Browser / Client | — | `RoomSurface` uses `use_query_map` + `use_navigate` (`room_surface.rs:79-109`) |
| Surface data (profile, roster, action items, notes) | API / Backend (`#[server]`) | — | Each surface's existing server fn unchanged; fetched eagerly on mount |
| QuickNav open/close + Cmd+K | Browser / Client | — | `RwSignal<bool>` + global keydown listener (`quick_nav.rs:26-55`) |
| QuickNav filter input | Browser / Client | — | Local `RwSignal<String>` + `event_target_value`; substring filter over `HUB_ROUTES` labels |
| QuickNav focus trap + restore | Browser / Client | — | Pure DOM: capture `document.activeElement` on open, intercept Tab keydown, restore `.focus()` on close — net-new |
| IA-healing 302 redirects | API / Backend (server fn) | Browser (`window.location`) | `leptos_axum::redirect()` SSR + `#[cfg(hydrate)] set_href()` — exact pattern proven in `library.rs:25-178` |
| Sub-route preservation (D-12) | Router config | — | `app.rs` `<Route>` ordering; exact-path redirect vs prefix-carrying sub-routes |

---

## Standard Stack

No new external packages in this phase. All required components already in the codebase and `Cargo.toml`.

### Core (already shipped — wire only)

| Component | Location | Purpose | Confirmed |
|-----------|----------|---------|-----------|
| `AtmosphereRoot` | `parallax.rs:54` | Parallax root; emits all 12 `data-*-tier` attrs incl. Throne's | Throne props `roster_tier`/`progression_tier`/`crown_tier` declared `parallax.rs:75-77`; root emits `data-roster-tier`/`data-progression-tier`/`data-crown-tier` `parallax.rs:346-348` [VERIFIED: codebase] |
| `Plane` / `GrowthOverlay` | `parallax.rs:361` / `:383` | Depth plane / axis overlay; `GrowthOverlay` doc lists `roster`/`progression`/`crown` `parallax.rs:386-388` | Same API as War Room/Library [VERIFIED: codebase] |
| `RoomSurface` | `room_surface.rs:74` | ARIA tablist + `?tab=` resolver + keep-mounted CSS-hidden panels | `panels: Vec<(SurfaceTab, PanelFn)>`; no changes needed; Throne is 3rd consumer [VERIFIED: codebase] |
| `tier_for` + `Tier` | `growth.rs:66` / `:14` | Pure count→tier mapper (0..=3) | Private fn; Throne uses same `from_counts` pattern [VERIFIED: codebase] |
| `leptos_axum::redirect` | `library.rs:25-178` | SSR 302 + dual `#[cfg(hydrate)] set_href()` shells | Exact precedent for D-12 — copy `StatsRedirect`/`MatchRedirect` shape [VERIFIED: codebase] |
| `SoloSurfaces` | `solo_dashboard.rs:283` | The Solo tab content, `pub fn SoloSurfaces(region: String)` | Reused verbatim as 5th Throne tab [VERIFIED: codebase] |
| `get_growth_state` | `solo_dashboard.rs:71` | Personal-axis server-fn precedent | Template for the `progression` axis derivation [VERIFIED: codebase] |
| `get_latest_ranked_snapshot` | `db.rs:5007` | User-scoped ranked: `tier`/`division`/`lp`/`wins`/`losses` | The `progression` source (D-04) [VERIFIED: codebase] |
| `list_action_items` / `list_open_action_items` | `db.rs:3005` / `:3020` | Development tab content | CONTEXT cited `:3005`/`:3020` — exact [VERIFIED: codebase] |
| `list_team_notes` / `list_pinned_team_notes` | `db.rs:3301` / `:3316` | Team Notebook content | CONTEXT cited `:3301`/`:3316` — exact [VERIFIED: codebase] |
| `get_team_growth_counts` | `db.rs:4200` | Batch `count() GROUP ALL` template | Model for `roster`/`crown` team count query [VERIFIED: codebase] |
| `get_user_team_id` | `db.rs:608` | Team-membership gate | Used by all team-scoped axes [VERIFIED: codebase] |
| `TeamNotebook` | `team/dashboard.rs:1526` | `fn TeamNotebook(current_user_id: String, is_leader: bool)` | Moves into `DevelopmentSurface` (D-03) [VERIFIED: codebase] |
| Action Items server fns | `action_items.rs` + `dashboard.rs` | `get_action_items`/`create`/status/update/delete + `get_open_action_items_summary` (`dashboard.rs:473`) | Fold into `DevelopmentSurface` [VERIFIED: codebase] |

### New Types Required

| Type | Location | Pattern / Note |
|------|----------|----------------|
| `ThroneGrowthCounts` | `growth.rs` | Team-scoped counts only: `team_member: u32`, `post_game_wins: u32` (or wins+losses for record). **Does NOT hold `progression`** — that's user-scoped, passed separately. |
| `ThroneGrowthState` | `growth.rs` | Fields: `roster_tier: u8`, `progression_tier: u8`, `crown_tier: u8`. Constructor takes BOTH team counts AND a user ranked snapshot (or pre-computed progression count) — see "Blended Growth" below. **Cannot derive from a single `*Counts` struct** (the D-05/D-07 divergence). |
| `ThroneRoom` + `ThroneSurfaces` | `pages/rooms/throne.rs` | Mirrors `LibraryRoom`/`LibrarySurfaces` |
| 3 redirect server fns + 3 redirect shells | `pages/rooms/throne.rs` | Mirror `library.rs` redirect shells for `/profile`, `/team/dashboard`, `/action-items` |
| `IdentitySurface` | `profile.rs` (same-file extraction, like War Room pattern) | New chrome-free surface |
| `DashboardSurface`/`RosterSurface`/`DevelopmentSurface` | `team/dashboard.rs` | Three new surfaces from the 2286-line page |

**No `npm install` / `cargo add`.** Verify nothing new needed: `cargo check --features ssr` + `cargo check --features hydrate --target wasm32-unknown-unknown`.

---

## Package Legitimacy Audit

No external packages installed in this phase. All dependencies already in `Cargo.toml` (`leptos`/`leptos_router`/`leptos_axum`/`axum` `0.8`, `web-sys 0.3.91`, `wasm-bindgen 0.2`). slopcheck not applicable.

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

**web-sys feature note:** `KeyboardEvent` compiles today without an explicit `Cargo.toml` feature entry (used in `quick_nav.rs:36`, `nav.rs:265`, `draft.rs`, etc.). `HtmlElement` and `Element` features ARE enabled (`Cargo.toml:42`), so `HtmlElement::focus()` and `document.activeElement` (returns `Element`) are available for the focus trap. The filter input can read its value via Leptos `event_target_value(&ev)` (no `HtmlInputElement` feature needed). **If `.focus()` on the restored element requires casting to `HtmlElement`, that cast is already feature-enabled — no `Cargo.toml` change expected.** Flag for the planner to confirm at compile time; add `HtmlInputElement`/`FocusEvent` only if the compiler demands it.

---

## Architecture Patterns

### System Architecture Diagram

```
Browser request: GET /throne?tab=dashboard
        │
        ▼
  ThroneRoom (SSR renders static HTML)
        │  reads InitialTheme ONCE → region: String   (rule #58)
        │  fetches get_throne_growth_state → ThroneGrowthState
        │     ├─ roster_tier      ← team_member count        (team-scoped, 0 if no team)
        │     ├─ progression_tier ← user ranked_snapshot LP  (USER-scoped, grows w/o team)  ← D-07 divergence
        │     └─ crown_tier       ← team post_game win/loss   (team-scoped, 0 if no team)
        │
        ├─► AtmosphereRoot (data-roster-tier, data-progression-tier, data-crown-tier)
        │       ├─ Plane (sky, mid, near)          /img/rooms/throne/{region}/plane-*.webp
        │       ├─ GrowthOverlay (roster, progression, crown)  overlay-*.webp
        │       │     ⚠ REQUIRES new input.css selectors [data-{axis}-tier="N"] .room-overlay--{axis}
        │       └─ contrast scrim (z-8 radial-gradient)        ← D-15, mirror solo_journey.rs:69
        │
        └─► ThroneSurfaces (z-10, region prop)
                └─► RoomSurface (panels=[Identity, Dashboard, Roster, Development, Solo])
                        │  reads ?tab= → active_idx; unknown/missing → 0 (Identity)
                        ├─ IdentitySurface     (from profile.rs — Riot link, pool, crest)
                        ├─ DashboardSurface    (from team/dashboard.rs — prep + summary panels)
                        ├─ RosterSurface       (from team/dashboard.rs — slots, coaches, requests)
                        ├─ DevelopmentSurface  (action_items.rs + TeamNotebook @ dashboard.rs:1526)
                        └─ SoloSurfaces        (reused verbatim from solo_dashboard.rs:283)

Legacy routes in app.rs → redirect shells (mirror library.rs):
  /profile          → server fn redirect("/throne?tab=identity")    + #[cfg(hydrate)] set_href
  /team/dashboard   → server fn redirect("/throne?tab=dashboard")   + #[cfg(hydrate)] set_href
  /action-items     → server fn redirect("/throne?tab=development")  + #[cfg(hydrate)] set_href
  /journey          → UNCHANGED (D-13 — coexists)
  /profile/... , /team/... sub-routes → PRESERVE (D-12 — planner audits app.rs)

QuickNav (app.rs:136, mounted once) — ROOM-08 additions:
  + filter <input> at top, auto-focus on open, substring filter over HUB_ROUTES labels (D-09)
  + focus trap: capture document.activeElement on open; intercept Tab; restore .focus() on close (D-10)
  + HUB_ROUTES restructured room-centric (D-11) → drives QuickNav AND nav.rs:527 sub-nav
```

### Recommended Project Structure

```
src/
├── pages/rooms/
│   ├── mod.rs          # add: pub mod throne;
│   ├── throne.rs       # ThroneRoom + ThroneSurfaces + get_throne_growth_state + 3 redirect shells
│   ├── library.rs      # template (closest sibling — has redirect shells)
│   ├── war_room.rs     # template (closest sibling — growth Resource shape)
│   └── solo_journey.rs # unchanged
├── pages/
│   ├── profile.rs            # add IdentitySurface (same-file extraction)
│   ├── team/dashboard.rs     # add DashboardSurface / RosterSurface / DevelopmentSurface
│   ├── action_items.rs       # surface content folded into DevelopmentSurface
│   └── solo_dashboard.rs     # SoloSurfaces reused (no change)
├── models/growth.rs   # add ThroneGrowthCounts + ThroneGrowthState (blended)
├── server/db.rs       # add get_throne_growth_counts (roster + crown) — team-scoped batch
├── components/nav.rs  # HUB_ROUTES restructure (room-centric) + sub-nav reads it
├── components/room/quick_nav.rs  # filter input + focus trap + focus-restore
├── app.rs             # add /throne route; 3 flat routes → redirects; preserve sub-routes
└── input.css          # ⚠ add roster/progression/crown tier→opacity selectors (Wave 0)
```

Surface extraction follows the War Room rule: each `*Surface` lives in its **originating page file** (`IdentitySurface` in `profile.rs`, the three dashboard surfaces in `team/dashboard.rs`), NOT in `throne.rs`. `throne.rs` only imports and composes them. This matches `war_room.rs:15-18` and `library.rs:16-20`.

### Pattern 1: Throne Room Structure (copy of Library/War Room)

```rust
// Source: src/pages/rooms/library.rs:220-285 (direct template) + war_room.rs:60-125
#[component]
pub fn ThroneRoom() -> impl IntoView {
    // Region read ONCE — never inside primitives (rule #58). The structural
    // guard for the A11Y-02 canary (D-14).
    let region = use_context::<InitialTheme>().unwrap_or_default().0;

    let growth = Resource::new(|| (), |_| get_throne_growth_state());
    let roster_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.roster_tier).unwrap_or(0)
    });
    let progression_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.progression_tier).unwrap_or(0)
    });
    let crown_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.crown_tier).unwrap_or(0)
    });

    let dir = format!("/img/rooms/throne/{region}");
    let surfaces_region = region.clone();

    view! {
        <AtmosphereRoot
            class="min-h-screen".to_string()
            roster_tier=roster_tier
            progression_tier=progression_tier
            crown_tier=crown_tier
        >
            <Plane src=format!("{dir}/plane-sky.webp") depth=6 z=1 />
            <Plane src=format!("{dir}/plane-mid.webp") depth=16 z=2 />
            <Plane src=format!("{dir}/plane-near.webp") depth=32 z=3 />
            <GrowthOverlay axis="roster" src=format!("{dir}/overlay-roster.webp") z=4 />
            <GrowthOverlay axis="progression" src=format!("{dir}/overlay-progression.webp") z=5 />
            <GrowthOverlay axis="crown" src=format!("{dir}/overlay-crown.webp") z=6 />
            // D-15: z-8 contrast scrim — verbatim from solo_journey.rs:69 / library.rs:272
            <div class="fixed inset-0 pointer-events-none"
                style="z-index: 8; background: radial-gradient(ellipse at center, transparent 0%, var(--color-base) 95%);"
                aria-hidden="true"></div>
            <div class="relative z-10 px-8 py-6" data-feedback-label="Throne">
                <ThroneSurfaces region=surfaces_region />
            </div>
        </AtmosphereRoot>
    }
}
```

[VERIFIED: codebase — direct structural copy of `library.rs:220-285`]

### Pattern 2: Blended Growth — the D-05/D-07 divergence (the genuinely new bit)

Unlike `get_team_growth_state`/`get_library_growth_state` (which `return Ok(Default::default())` the moment `get_user_team_id` is `None`), `get_throne_growth_state` must **continue past the no-team check** to compute `progression` from the user's ranked snapshot.

```rust
// Source: synthesized from war_room.rs:27-53 (server fn shape) + solo_dashboard.rs:71-90
//         (personal axis precedent) + db.rs:5007 (ranked source)
#[server]
pub async fn get_throne_growth_state(
) -> Result<crate::models::growth::ThroneGrowthState, ServerFnError> {
    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    // PERSONAL axis — computed FIRST, regardless of team (D-07). Never Err (rule #44).
    let ranked = db::get_latest_ranked_snapshot(&surreal, &user.id, "RANKED_SOLO_5x5")
        .await.unwrap_or(None);

    // TEAM axes — 0 when no team (do NOT early-return the whole state).
    let team_counts = match db::get_user_team_id(&surreal, &user.id).await? {
        Some(team_id) => db::get_throne_growth_counts(&surreal, &team_id)
            .await.unwrap_or_default(),
        None => ThroneGrowthCounts::default(),   // roster=0, crown=0
    };

    Ok(ThroneGrowthState::from_parts(team_counts, ranked))
}
```

`ThroneGrowthState::from_parts` is the new shape — it takes the team counts AND the optional ranked snapshot, deriving `progression_tier` from LP/rank milestones independently. **Do NOT force `progression` through a `*Counts` struct** that only exists for a team — that would re-couple it to team membership and re-break D-07.

[VERIFIED: codebase — `db.rs:5007` returns `RankedInfo { tier, division, lp, wins, losses }`; `war_room.rs:44-47` shows the early-return pattern the Throne must NOT use]

### Pattern 3: ThroneGrowthState — proposed model

```rust
// src/models/growth.rs — add after LibraryGrowthState
// roster: team_member count → roster size milestones
// progression: USER ranked LP/rank → personal climb (the blended/personal axis)
// crown: combined team record from post_game_learning.win_loss → team record milestones

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThroneGrowthCounts {
    /// team_member rows for the team (team-scoped, roster axis).
    pub team_members: u32,
    /// team wins counted from post_game_learning.win_loss == "win" (team-scoped, crown axis).
    pub team_wins: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThroneGrowthState {
    pub roster_tier: u8,
    pub progression_tier: u8,
    pub crown_tier: u8,
}

impl ThroneGrowthState {
    /// Blended derivation: team counts + the viewing user's optional ranked snapshot.
    /// progression derives from a personal metric (e.g. total ranked games = wins+losses,
    /// or a tier/LP milestone) so it advances with NO team (D-05/D-07).
    pub fn from_parts(
        team: ThroneGrowthCounts,
        ranked: Option<crate::models::user::RankedInfo>,
    ) -> Self {
        let roster = tier_for(team.team_members, [1, 3, 5]);      // 1 = you, 3 = forming, 5 = full
        let crown  = tier_for(team.team_wins,     [1, 5, 15]);     // first win → consistent record
        // progression: ranked games played (wins+losses) is a robust, always-present proxy
        let games = ranked.as_ref().map(|r| (r.wins + r.losses).max(0) as u32).unwrap_or(0);
        let progression = tier_for(games, [1, 20, 60]);
        ThroneGrowthState {
            roster_tier: roster.level(),
            progression_tier: progression.level(),
            crown_tier: crown.level(),
        }
    }
}

impl Default for ThroneGrowthState {
    fn default() -> Self { Self::from_parts(ThroneGrowthCounts::default(), None) }
}
```

[ASSUMED — threshold values AND the exact progression metric (games-played vs LP-milestone vs tier-rank); calibration hypotheses per the Phase 26/27/28 precedent. `RankedInfo` shape VERIFIED at `db.rs:5032-5039`.]

### Pattern 4: `get_throne_growth_counts` (team-scoped batch — roster + crown)

```rust
// src/server/db.rs — model on get_team_growth_counts (db.rs:4200)
// roster:  count team_member rows for the team
// crown:   count post_game_learning rows where win_loss = "win" (D-06: cheap team-scoped proxy)
pub async fn get_throne_growth_counts(db: &Surreal<Db>, team_id: &str)
    -> DbResult<ThroneGrowthCounts> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut result = db.query(
        "SELECT count() as n FROM team_member WHERE team = type::record('team', $team_key) GROUP ALL; \
         SELECT count() as n FROM post_game_learning WHERE team = type::record('team', $team_key) AND win_loss = 'win' GROUP ALL",
    ).bind(("team_key", team_key)).await?;
    // take(0) → team_members, take(1) → team_wins (use the count_of helper pattern, db.rs:4224)
}
```

**Verify field names against `schema.surql` before writing:** `post_game_learning.win_loss` is `option<string>` (`schema.surql:126`) — confirmed it carries the win/loss result. `team_member.team` field name must be confirmed (the War Room query uses `WHERE team = ...` on draft/opponent/etc.; verify `team_member` uses the same `team` field — likely, but the planner MUST grep `schema.surql` for the `team_member` table definition, mirroring Library RESEARCH Pitfall 5).

[VERIFIED: `post_game_learning.win_loss` at `schema.surql:126`; `get_team_growth_counts` batch pattern at `db.rs:4200-4243`. `team_member.team` field name FLAGGED for planner verification.]

### Pattern 5: IA-Healing Redirect Shells (copy library.rs verbatim)

`library.rs:25-178` is the exact, working precedent. Three shells needed:

```rust
// Source: src/pages/rooms/library.rs:25-78 (StatsRedirect is the literal template)
#[server]
pub async fn redirect_to_throne_identity() -> Result<(), ServerFnError> {
    leptos_axum::redirect("/throne?tab=identity");
    Ok(())
}
#[component]
pub fn ProfileRedirect() -> impl IntoView {
    let action = ServerAction::<RedirectToThroneIdentity>::new();
    action.dispatch(RedirectToThroneIdentity {});
    #[cfg(feature = "hydrate")]
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/throne?tab=identity");
    }
    view! { <></> }
}
// ...analogous for /team/dashboard → ?tab=dashboard and /action-items → ?tab=development
```

**D-12 sub-route audit (planner MUST do before redirecting):** Current `app.rs` routes for these features:
- `/profile` (`:140`) → `ProfilePage`. No `/profile/...` sub-routes exist in `app.rs` (verified — only the exact path). Safe to redirect.
- `/team/dashboard` (`:143`) → `TeamDashboard`. Sibling `/team/roster` (`:144` → `RosterPage`) and `/team-builder` (`:149`) are SEPARATE routes — do NOT redirect them. No `/team/dashboard/...` sub-routes. Safe to redirect `/team/dashboard` exactly.
- `/action-items` (`:153`) → `ActionItemsPage`. No sub-routes. Safe to redirect.
- **`/team/roster` is NOT redirected** — it remains its own page (the Roster *surface* in the Throne is the dashboard's roster panel; `/team/roster` is the create/join flow). Confirm with the user/plan whether `/team/roster` should also heal — CONTEXT D-12 only names `/profile`, `/team/dashboard`, `/action-items`. Leave `/team/roster` standalone.

[VERIFIED: `app.rs:139-167` route block; `library.rs:25-178` redirect shell pattern]

### Pattern 6: QuickNav Filter + Focus Trap (NET-NEW — no precedent to copy)

There is **no existing focus-trap / `inert` / `activeElement` code in `src/`** (grep returned nothing). This is the only genuinely new mechanism. WCAG 2.1 SC 2.1.2 ("No Keyboard Trap" — paradoxically the requirement here is a *controlled* modal trap with a guaranteed escape via Escape).

**Filter (D-09):**
```rust
let filter = RwSignal::new(String::new());
// in the overlay, at the top:
<input
    type="text"
    prop:value=move || filter.get()
    on:input=move |ev| filter.set(event_target_value(&ev))
    placeholder="Jump to…"
    // auto-focus on open: a node_ref + Effect that calls .focus() when `open` flips true
    class="... focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
/>
// filter each (path, label) by label.to_lowercase().contains(&filter.get().to_lowercase())
```

**Focus trap + restore (D-10) — recommended approach for Leptos 0.8 / WASM:**
1. **Capture on open:** in the `on:click`/keydown that sets `open=true`, read `document.activeElement` and store it in a `StoredValue<Option<web_sys::Element>>` (or `RwSignal`). This is the element focus returns to.
2. **Auto-focus the filter input on open:** `NodeRef` on the input + an `Effect` that, when `open.get()` becomes true, calls `input_ref.get().focus()` (guard `#[cfg(feature = "hydrate")]`, no bare `.unwrap()` — rule #35).
3. **Trap Tab:** add an `on:keydown` on the dialog container (NOT a global listener) that, on `Tab`/`Shift+Tab`, computes the first/last focusable element inside the dialog and wraps focus (`ev.prevent_default()` + `.focus()` the wrap target). Simplest robust version: keep a `Vec` of the focusable items (the filter input + the visible `<A>` links) and cycle by index. Because all items are inside one container, an alternative is to set `inert`/`tabindex=-1` on everything outside — but the codebase has no `inert` usage and Leptos renders the rest of the app as siblings; the **wrap-on-Tab approach is lower-risk** and does not require touching sibling DOM.
4. **Restore on close:** the existing `open.set(false)` paths (Escape at `quick_nav.rs:39`, backdrop click `:80`, link click `:107`) must ALSO call `.focus()` on the captured trigger element. Centralize close logic into one `close()` closure so all three paths restore focus.

**Keep existing rings:** every item already has `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none` (`quick_nav.rs:65,108`) — preserve verbatim (G-12). The global `input.css:374-375` `:focus-visible` outline is the fallback.

[VERIFIED: `quick_nav.rs:24-124` current implementation; grep confirms NO existing focus-trap code. web-sys `HtmlElement::focus()` available via enabled `HtmlElement` feature (`Cargo.toml:42`).]

### Pattern 7: HUB_ROUTES Room-Centric Restructure (D-11)

Current `HUB_ROUTES` (`nav.rs:160-193`) is FLAT-route grouped by old hubs (`strategy`/`history`/`profile`) with links like `/draft`, `/stats`, `/profile` — **many of which now 302-redirect** (`/stats`, `/post-game`, `/opponents`, `/analytics` already redirect to Library/War Room from Phase 28; `/profile`, `/team/dashboard`, `/action-items` will redirect to Throne this phase). D-11 requires restructuring to three ROOM groups pointing at `?tab=` deep-links:

```rust
pub const HUB_ROUTES: &[(&str, &[(&str, &str)])] = &[
    ("war-room", &[
        ("/war-room?tab=draft", "Draft"),
        ("/war-room?tab=tree-drafter", "Tree"),
        ("/war-room?tab=game-plan", "Game Plan"),
        ("/war-room?tab=opponents", "Opponents"),
        ("/war-room?tab=ban-priority", "Ban Priority"),
    ]),
    ("library", &[
        ("/library?tab=stats", "Stats"),
        ("/library?tab=match", "Match"),
        ("/library?tab=analytics", "Analytics"),
        ("/library?tab=post-game", "Post-Game"),
        ("/library?tab=personal", "Learnings"),
    ]),
    ("throne", &[
        ("/throne?tab=identity", "Identity"),
        ("/throne?tab=dashboard", "Dashboard"),
        ("/throne?tab=roster", "Roster"),
        ("/throne?tab=development", "Development"),
        ("/throne?tab=solo", "Solo"),
    ]),
];
```

**Knock-on effects the planner MUST handle (both consumers read `HUB_ROUTES`):**
- `nav.rs:198 hub_for_path` + `:283-306` `active_hub` derivation + the four hub buttons (`:320-332`) and the sub-nav strip (`:522-557`) all key on the OLD hub names (`strategy`/`history`/`profile`/`live`). Restructuring `HUB_ROUTES` keys to `war-room`/`library`/`throne` REQUIRES updating `hub_for_path`, the primary hub `<A>` buttons, and `quick_nav.rs:15 hub_label` (which maps `strategy`→"War Room" etc.). This is a coordinated nav.rs + quick_nav.rs change, not just a constant edit.
- The top-nav primary buttons (`nav.rs:320` "Strategy" → `/draft`, `:327` "History" → `/stats`, `:330` "Profile" → `/profile`) should point at the room roots (`/war-room`, `/library`, `/throne`). `/draft`/`/stats` still work (War Room flat shells / Library redirect) but `/profile` now redirects — update to `/throne`.
- `quick_nav.rs:15-22 hub_label` and `:99` must map the new keys (`war-room`→"War Room", `library`→"Hall of Records", `throne`→"Throne").

[VERIFIED: `nav.rs:160-225` HUB_ROUTES + hub_for_path; `nav.rs:522-557` sub-nav; `quick_nav.rs:15-22,93-118` both consume HUB_ROUTES]

### Anti-Patterns to Avoid

- **Reading context inside reactive primitives (the Phase 18.2 panic):** `use_context::<InitialTheme>()` only at the top of `ThroneRoom`. Never inside `Signal::derive`, `Effect`, `move || {}`, a `view!` branch, or any `*Surface` child (rule #58). This is the structural guard the A11Y-02 canary (D-14) enforces.
- **Per-tab conditional rendering keyed on team/mode (D-02):** Do NOT add `<Show when=has_team>` around team-scoped surfaces. Always mount all five panels; show empty-state CTAs *inside* the surface content. A structural branch keyed on team membership is a region-independent variant of the same hydration-mismatch class.
- **Copying the dashboard's `is_solo_mode`/`mode_current`/`is_pandemonium` branching into the surfaces:** The current `TeamDashboard` (`:617-723`) branches on solo-mode, brief-mode, and region to pick a whole sub-component (`PandemoniumTeamDashboard`, `TeamGameDayBriefView`). When extracting `DashboardSurface`/`RosterSurface`/`DevelopmentSurface`, strip the `ModeToggle`/brief-mode/region-branch chrome — the surface should render ONE consistent structure. Keep the *data Resources* (`get_team_dashboard`, `get_pending_requests`, `get_open_action_items_summary`, `get_post_game_panel`, `get_pool_gap_panel`), drop the mode/region dispatch.
- **Early-returning `Default` for no-team in `get_throne_growth_state` (D-07):** the single most likely mistake — it's the pattern in BOTH sibling server fns (`war_room.rs:44-47`, `library.rs:204-207`). The Throne must NOT do this; `progression` must compute regardless.
- **Forgetting the input.css selectors:** see Pitfall 1 — the overlays render at `opacity:0` without them and SC1's visual silently fails while the data-attr assertion passes.
- **`<Show>` around surfaces inside `ThroneSurfaces`:** `RoomSurface` already keep-mounts + CSS-hides via `attr:hidden` (`room_surface.rs:146-160`). Do not wrap panels in `<Show>`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tab strip + `?tab=` deep-link | Custom tab component | `RoomSurface` (`room_surface.rs`) | Keep-mounted panels, ARIA tablist, unknown→first fallback all done; Throne is 3rd consumer |
| Growth tier derivation | Custom tier logic | `tier_for()` (`growth.rs:66`) + `ThroneGrowthState::from_parts` | Pure, unit-tested; same `Tier` enum 0..=3 |
| Parallax mouse bridge | Custom mousemove handler | `AtmosphereRoot` (`parallax.rs`) | Single-listener slot, RAF batching, runtime reduced-motion change-listener (A11Y-01) — non-trivial, already correct |
| Server-side 302 redirect | Axum middleware | `leptos_axum::redirect()` + dual `#[cfg(hydrate)] set_href()` | Exact working precedent in `library.rs:25-178` |
| Team-scoped count queries | Per-table round-trips | Batch `count() GROUP ALL` (`db.rs:4200`) | One round-trip; `count_of` helper handles absent rows |
| Solo tab content | Re-extract LP graph / goals / trends | `SoloSurfaces(region)` (`solo_dashboard.rs:283`) | Already a `pub` composer; reuse verbatim (D-01) |
| Contrast scrim | New gradient | `solo_journey.rs:69` z-8 radial-gradient `<div>` | Verbatim across all 3 rooms; the D-15 control |

**Key insight:** The Throne is 90% composition of shipped parts. The ONLY new logic is (a) the blended `progression` axis (≈30 lines in `growth.rs` + the server fn), (b) the QuickNav focus trap (≈60 lines in `quick_nav.rs`), and (c) the dashboard split (mechanical but large). Everything else is copy-adapt.

---

## Runtime State Inventory

Not a rename/refactor/migration phase — no stored strings, service config, OS-registered state, secrets, or build artifacts change identity. **None — verified: this phase adds a new route + surfaces and redirects three legacy routes; no datastore keys, env vars, or registered tasks are renamed.** The one persistence-adjacent note: `set_team_dashboard_mode_pref` (`dashboard.rs:580`) and the `team_dashboard_mode` user field continue to exist; the Dashboard surface no longer needs the mode toggle, but the DB field is harmless if left (no migration required).

---

## Common Pitfalls

### Pitfall 1: Throne overlays render at opacity:0 — missing CSS tier selectors (SC1 silent failure)
**What goes wrong:** `.room-overlay` defaults to `opacity:0` (`input.css:431`). Visibility is driven ENTIRELY by `[data-{axis}-tier="N"] .room-overlay--{axis} { opacity: … }` rules. Those rules exist for `lib`/`armory`/`banner`/`tactics`/`intel`/`command`/`records`/`tomes`/`analysis` (`input.css:436-464`) but **NOT for `roster`/`progression`/`crown`**. The Throne's three overlays will stay fully transparent at every tier — the room never visibly grows.
**Why it happens:** The data-tier attributes ARE emitted by `AtmosphereRoot` (`parallax.rs:346-348`), so the `data-roster-tier="3"` attribute assertion passes — the failure is invisible to attribute-based tests. Exactly the Phase 27 "room growth visibility gap" (MEMORY.md).
**How to avoid:** Wave 0 task — add nine selectors to `input.css` mirroring `:456-464`:
```css
[data-roster-tier="1"] .room-overlay--roster { opacity: 0.34; }
[data-roster-tier="2"] .room-overlay--roster { opacity: 0.67; }
[data-roster-tier="3"] .room-overlay--roster { opacity: 1; }
/* …progression, …crown analogous */
```
**Warning signs:** SC1 data-tier assertion green but the room looks identical fresh vs high-tier. The seed-and-assert-opacity e2e test (Phase 27 lesson) must assert computed `opacity > 0` on the overlay `<img>`, not just the data attr.

### Pitfall 2: `get_throne_growth_state` early-returns Default on no-team (breaks D-07)
**What goes wrong:** Copying `library.rs:204-207` / `war_room.rs:44-47` verbatim returns `ThroneGrowthState::default()` (all 0) the instant `get_user_team_id` is `None`, so a teamless solo grinder's `progression` axis never lights — the Throne feels dead for exactly the user D-05 is designed to serve.
**Why it happens:** The early-return is the established sibling idiom; it's the natural copy.
**How to avoid:** Compute `progression` BEFORE the team check (Pattern 2). Only the team-scoped `roster`/`crown` fall to 0 without a team.
**Warning signs:** Throne `data-progression-tier="0"` for a ranked-but-teamless user. Add a unit test: `ThroneGrowthState::from_parts(ThroneGrowthCounts::default(), Some(ranked_with_60_games))` → `progression_tier == 3`, `roster_tier == 0`.

### Pitfall 3: Placeholder art 404s break the room paint
**What goes wrong:** `<Plane src="/img/rooms/throne/{region}/plane-sky.webp">` and the three overlays 404 if the placeholder WebPs don't exist. `loading="eager"` (`parallax.rs:370`) means they fetch immediately on paint.
**How to avoid:** Wave 0 — create placeholder gradient WebPs at `/img/rooms/throne/demacia/` and `/img/rooms/throne/pandemonium/` (plane-sky/mid/near + overlay-roster/progression/crown). Mirror whatever Library/War Room used for their placeholders (check `public/img/rooms/library/` for the exact filenames/format).
**Warning signs:** Console 404s on `/img/rooms/throne/...` (but note e2e filters 404s — rule #47 — so these won't fail the panic canary; check manually).

### Pitfall 4: Dashboard split drops a data Resource or duplicates a server fn
**What goes wrong:** `team/dashboard.rs:663-668` declares six Resources (`get_team_dashboard`, `get_pending_requests`, `get_recent_team_matches`, `get_open_action_items_summary`, `get_post_game_panel`, `get_pool_gap_panel`). Splitting into three surfaces, each fetching only what it needs, risks (a) dropping one, or (b) re-declaring the same Resource in two surfaces (double-fetch).
**How to avoid:** Map each Resource to ONE owning surface before extracting:
- `DashboardSurface`: `get_team_dashboard` (summary), `get_open_action_items_summary`, `get_post_game_panel`, `get_pool_gap_panel`, `get_recent_team_matches` (prep/at-a-glance).
- `RosterSurface`: `get_team_dashboard` (members), `get_pending_requests` + `handle_join_request`.
- `DevelopmentSurface`: Action Items fns (`action_items.rs`) + `get_all_team_notes`/`add_team_note`/`toggle_pin`/`edit`/`remove` (`dashboard.rs:363-467`) via the moved `TeamNotebook`.
- Note `get_team_dashboard` is needed by BOTH Dashboard and Roster — provide it once at `ThroneSurfaces` via `provide_context` (or accept a mild double-fetch; eager-fetch optimization is explicitly deferred).
**Warning signs:** A surface shows empty/stale where the old dashboard showed data; or network tab shows the same server fn fired twice.

### Pitfall 5: HUB_ROUTES restructure breaks nav.rs hub derivation (cascade)
**What goes wrong:** Renaming `HUB_ROUTES` keys from `strategy`/`history`/`profile` to `war-room`/`library`/`throne` without updating `hub_for_path` (`nav.rs:198`), the `active_hub` consumers (`:283-306`), the four primary hub `<A>` buttons (`:320-332`), and `quick_nav.rs:15 hub_label` leaves the sub-nav strip and active-state highlighting reading stale keys → sub-nav renders empty or wrong.
**How to avoid:** Treat D-11 as a coordinated nav.rs + quick_nav.rs change (Pattern 7). Update all five touch-points in one task.
**Warning signs:** Sub-nav strip empty on `/throne`; primary hub buttons never highlight; QuickNav group headers show "More".

### Pitfall 6: `team_member.team` field-name assumption in the roster count
**What goes wrong:** The roster axis counts `team_member` rows. If the schema field is not `team` (the War Room counts use `WHERE team = …` on draft/opponent/etc.), the count silently returns 0 (Library RESEARCH Pitfall 5, same class).
**How to avoid:** grep `schema.surql` for the `team_member` table's team-reference field name before writing `get_throne_growth_counts`. Verify against an actual query.
**Warning signs:** `roster_tier` stays 0 for a full team.

---

## Code Examples

### Throne axis props already wired in AtmosphereRoot
```rust
// Source: src/components/room/parallax.rs:75-77 — props EXIST, just pass signals
#[prop(optional, into)] roster_tier: Signal<u8>,
#[prop(optional, into)] progression_tier: Signal<u8>,
#[prop(optional, into)] crown_tier: Signal<u8>,
// Source: parallax.rs:346-348 — root already emits the data attrs
data-roster-tier=move || roster_tier.get().to_string()
data-progression-tier=move || progression_tier.get().to_string()
data-crown-tier=move || crown_tier.get().to_string()
```

### GrowthOverlay already documents the Throne axes
```rust
// Source: src/components/room/parallax.rs:386-388
// "…Throne `roster` | `progression` | `crown`."  ← axis keys are first-class
<GrowthOverlay axis="roster" src=... z=4 />
```

### Ranked snapshot shape (the progression source)
```rust
// Source: src/server/db.rs:5032-5039 (get_latest_ranked_snapshot returns RankedInfo)
RankedInfo { queue_type, tier, division, lp: i32, wins: i32, losses: i32 }
// games played = wins + losses ; LP/tier available for richer milestone derivation
```

### SoloSurfaces is a pub composer — reuse directly
```rust
// Source: src/pages/solo_dashboard.rs:282-283
#[component]
pub fn SoloSurfaces(region: String) -> impl IntoView { ... }
// In ThroneSurfaces panel: Box::new(move || view!{ <SoloSurfaces region=r.clone() /> }.into_any())
```

### TeamNotebook is a self-contained component to move
```rust
// Source: src/pages/team/dashboard.rs:1525-1526
#[component]
fn TeamNotebook(current_user_id: String, is_leader: bool) -> impl IntoView { ... }
// Move into DevelopmentSurface; it already owns get_all_team_notes/add/toggle/edit/remove
```

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| `/profile`, `/team/dashboard`, `/action-items` standalone pages | 302-redirect to `/throne?tab=…` surfaces | One canonical home per feature; deep links preserved (D-12) |
| Team dashboard = one 2286-line mode-branched page | Three intent-named surfaces (Dashboard/Roster/Development) | Each feature is a surface (Phase 27/28 philosophy) |
| War Room/Library growth = all-or-nothing team scope | Throne blends team + personal; never fully tier-0 for solo | First blended room (D-05) |
| `HUB_ROUTES` grouped by old hubs with flat-route links | Room-centric groups with `?tab=` deep-links | Nav IA reflects the three rooms (D-11) |
| QuickNav = static list, Escape-only | Filterable command palette + focus trap + focus-restore | WCAG 2.1 SC 2.1.2 closed-beta blocker cleared (D-09/D-10) |

**Deprecated/outdated in this phase:**
- `ProfilePage` standalone → `IdentitySurface` + `ProfileRedirect`.
- `TeamDashboard` standalone → three surfaces + `TeamDashboardRedirect`.
- `ActionItemsPage` standalone → folded into `DevelopmentSurface` + `ActionItemsRedirect`.
- The Dashboard `ModeToggle` (Dashboard / Game Day Brief) — not carried into the surface (chrome stripped). Confirm with planner whether Game Day Brief content survives anywhere or is dropped this phase (CONTEXT D-01 says Dashboard ← "summary"; the Brief is a separate mode, likely out of scope — FLAG).

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `progression` axis derives from ranked games-played (`wins+losses`) with thresholds [1,20,60] | Pattern 3 | Low — calibration hypothesis; tunable post-beta. Could instead key on LP/tier milestones (D-04 says "LP / rank milestones"). Planner/discuss may pick LP-based. `RankedInfo` shape verified. |
| A2 | `crown` counts `post_game_learning WHERE win_loss='win'` | Pattern 4 | Medium — `win_loss` is `option<string>`; the exact string value ("win" vs "Win" vs "victory") must be confirmed against how post-game review writes it. grep `save_post_game_learning` (`db.rs:1890`) / the post-game form for the literal. |
| A3 | `team_member` table uses a `team` reference field (like draft/opponent) | Pattern 4 / Pitfall 6 | Medium — silently returns 0 if wrong. MUST verify in `schema.surql` before writing the query. |
| A4 | `roster`/`crown` thresholds [1,3,5] / [1,5,15] | Pattern 3 | Low — calibration hypotheses per Phase 26/27/28 precedent. |
| A5 | Game Day Brief mode + `TeamDashboard` mode toggle are dropped (not carried into a surface) | State of the Art | Medium — if the user wants the Brief preserved, it needs a home. CONTEXT D-01 scopes Dashboard to "summary"; Brief appears out of scope but is not explicitly deferred. FLAG for discuss/plan. |
| A6 | No `Cargo.toml` web-sys feature additions needed for the focus trap | Package Legitimacy / Pattern 6 | Low — `HtmlElement`/`Element`/`KeyboardEvent` available today. If `.focus()`/`HtmlInputElement` demands a feature at compile time, add it (one-line change). |
| A7 | `/team/roster` stays standalone (not healed into Throne) | Pattern 5 | Low — CONTEXT D-12 names only three routes; `/team/roster` is the create/join flow, distinct from the Roster surface (dashboard panel). |

**These assumptions are the discuss-phase / planner confirmation targets** — A2, A3, A5 in particular are verifiable by grep and should be resolved during planning, not deferred to execution.

---

## Open Questions (RESOLVED)

> All three resolved during planning (2026-06-08); resolutions are locked in the plan interfaces. Annotated below.

1. **`crown` win/loss literal value** — **RESOLVED: `win_loss = 'win'`** (confirmed `post_game.rs:828`, `db.rs:2051`). Locked in Plan 29-01 Task 2 interfaces/action.
   - What we knew: `post_game_learning.win_loss` is `option<string>` (`schema.surql:126`).

2. **`progression` metric: games-played vs LP-milestone vs tier-rank** — **RESOLVED: games-played** (`wins+losses`, tier thresholds `[1,20,60]`); pure `growth.rs` choice, no UI impact, tunable post-beta. Locked in Plan 29-01 action.
   - What we knew: `RankedInfo` carries `tier`/`division`/`lp`/`wins`/`losses` (`db.rs:5032`).

3. **Game Day Brief fate (A5)** — **RESOLVED: DROPPED this phase.** `TeamGameDayBriefView` was an alternate dashboard *mode* (a mode-branch that violates D-02's no-structural-branch rule); its summary content is preserved inside `DashboardSurface`. Decision locked in Plan 29-04 interfaces DECISION block.
   - What we knew: `TeamGameDayBriefView` (`dashboard.rs:2016`) is a whole alternate dashboard mode.

---

## Environment Availability

Step 2.6: SKIPPED — code/CSS/asset changes only, no new external tools, services, runtimes, or CLIs beyond the existing Rust/Leptos/cargo-leptos/Playwright stack. (Placeholder WebP creation needs only an image tool already available; mirror existing `public/img/rooms/library/` assets.)

---

## Validation Architecture

> nyquist_validation is not disabled in config (assumed enabled). Sibling phases 27/28 produced VALIDATION.md from this section.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Playwright (e2e) + Rust `#[cfg(test)]` unit tests |
| Config file | `e2e/playwright.config.ts` |
| Quick run command | `cd e2e && npx playwright test pages.spec.ts` |
| Full suite command | `cd e2e && npx playwright test` |
| Unit tests | `cargo test --features ssr --lib` (integration tests OOM — `--lib` only, rule #39) |

### SC1–SC5 → Test Map
The phase has five empirically-verifiable success criteria (per CONTEXT). Mapping:

| SC | Behavior | Test Type | Automated Command | File Exists? |
|----|----------|-----------|-------------------|-------------|
| SC1 | Throne environment reflects roster/LP/record — `data-{roster,progression,crown}-tier` advance AND overlay opacity > 0 by tier | e2e (seed + assert) | `cd e2e && npx playwright test throne-growth.spec.ts` | ❌ Wave 0 — new spec; assert computed `opacity` on `.room-overlay--roster` etc., NOT just data-attr (Pitfall 1) |
| SC1b | Blended: teamless ranked user → `data-progression-tier > 0`, `data-roster-tier = 0` | unit + e2e | `cargo test --features ssr --lib -- throne` | ❌ Wave 0 — `growth.rs` unit test `throne_progression_grows_without_team` |
| SC2 | QuickNav focus-restore: Escape returns focus to trigger; close paths restore | e2e a11y | `cd e2e && npx playwright test quicknav-focus.spec.ts -g "restore"` | ❌ Wave 0 — new spec |
| SC3 | QuickNav focus-trap: Tab stays within open overlay (does not reach page behind) | e2e a11y | `cd e2e && npx playwright test quicknav-focus.spec.ts -g "trap"` | ❌ Wave 0 — new spec |
| SC4 | `/war-room`, `/library`, `/throne` hydrate without tachys panic in BOTH regions | e2e canary | `cd e2e && npx playwright test hydration-no-panic.spec.ts -g "/throne"` | ❌ Wave 0 — add `/throne` to `SCOPED_ROUTES` (`hydration-no-panic.spec.ts:72-89`) |
| SC5 | WCAG AA (4.5:1) text contrast over art per room (z-8 scrim present + measured) | e2e contrast | `cd e2e && npx playwright test room-contrast.spec.ts` | ❌ Wave 0 — new spec (or extend existing); assert scrim `<div>` present + sample contrast |
| Redirects | `/profile`, `/team/dashboard`, `/action-items` resolve to Throne (302, not 404); sub-routes preserved | smoke | `cd e2e && npx playwright test pages.spec.ts` | ❌ Wave 0 — update `AUTHED_PAGES` (`pages.spec.ts:8-25`) |

### e2e Suite Changes Required (detailed)

**`e2e/tests/hydration-no-panic.spec.ts` `SCOPED_ROUTES` (`:72-89`):**
```typescript
// ADD:
"/throne",
// /war-room and /library already covered; /throne completes the 3-room A11Y-02 gate.
// Both regions auto-iterated by the existing for-loop (:145-166) → 2 new tests.
```

**`e2e/tests/pages.spec.ts` `AUTHED_PAGES` (`:8-25`):**
```typescript
// ADD:
{ path: "/throne", content: /identity|dashboard|roster|development|solo/i },
// UPDATE/REMOVE (now redirect to Throne — expect redirect destination, not standalone):
//   { path: "/profile", ... }        → remove or expect /throne content
//   { path: "/team/dashboard", ... } → remove or expect /throne content
//   (/action-items has no AUTHED_PAGES entry currently — verify)
// KEEP: /team/roster (standalone, D-12), /team-builder, /solo, /personal-learnings/new
```

**New specs (Wave 0):**
- `throne-growth.spec.ts` — seed a high-tier team+ranked scenario vs fresh; assert `data-*-tier` AND computed overlay `opacity > 0` (Phase 27 seed-and-assert-opacity lesson).
- `quicknav-focus.spec.ts` — open via Cmd+K; Tab N times, assert `document.activeElement` stays inside the dialog; Escape, assert focus on the trigger.
- `room-contrast.spec.ts` — assert the z-8 scrim `<div[style*="radial-gradient"]]>` present on each room; optionally sample foreground/background luminance for 4.5:1.

**`src/models/growth.rs` unit tests (Wave 0):** add `throne_empty_all_seed`, `throne_roster_tracks_members`, `throne_crown_tracks_wins`, `throne_progression_grows_without_team`, `throne_state_round_trips_json` — mirror the Library tests (`growth.rs:334-371`).

### Sampling Rate
- **Per task commit:** `cargo check --features ssr` + `cargo check --features hydrate --target wasm32-unknown-unknown`; `cargo test --features ssr --lib` for `growth.rs` edits.
- **Per wave merge:** `cd e2e && npx playwright test pages.spec.ts hydration-no-panic.spec.ts` + the new specs.
- **Phase gate:** full e2e suite green before `/gsd:verify-work`.

### Wave 0 Gaps
- [ ] `input.css` — add 9 tier→opacity selectors for `roster`/`progression`/`crown` (Pitfall 1, BLOCKER for SC1).
- [ ] `public/img/rooms/throne/{demacia,pandemonium}/` — placeholder gradient WebPs (plane-sky/mid/near + overlay-roster/progression/crown).
- [ ] `e2e/tests/hydration-no-panic.spec.ts` — add `/throne` to `SCOPED_ROUTES`.
- [ ] `e2e/tests/pages.spec.ts` — add `/throne`; update redirected entries.
- [ ] `src/models/growth.rs` — `ThroneGrowthCounts`/`ThroneGrowthState` + 5 unit tests.
- [ ] New e2e specs: `throne-growth.spec.ts`, `quicknav-focus.spec.ts`, `room-contrast.spec.ts`.
- [ ] Resolve A2 (`win_loss` literal) + A3 (`team_member.team` field) by grep before the DB query task.

---

## Security Domain

`security_enforcement` not set to `false`. This phase adds a new route, three redirects, three new surfaces (composing existing authorized features), and one new team-scoped growth query. No new auth, session, or crypto changes.

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | Redirects + surfaces preserve existing auth guards; `get_throne_growth_state` rejects unauthenticated (`ServerFnError`, mirrors `war_room.rs:37-39`) |
| V3 Session Management | No | No session changes |
| V4 Access Control | Yes | `get_throne_growth_counts` MUST use the pre-authorized `team_id` from `get_user_team_id` — never raw client input (same contract as `get_team_growth_counts`, `db.rs:4248-4250`). The user-scoped `progression` reads only the authenticated user's own `ranked_snapshot`. |
| V5 Input Validation | Yes (low) | QuickNav filter input is client-only display filtering — no server round-trip, no injection surface. Redirect targets are static literals (no user-controlled redirect path). |
| V6 Cryptography | No | None |

### Known Threat Patterns for Leptos/SurrealDB stack
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via `region` in asset URLs | Tampering | `region` is server-set from cookie, never a URL/query param (`war_room.rs:90-91` comment); Throne inherits this — only `format!("/img/rooms/throne/{region}/…")` with the resolved value |
| Team-id spoofing in growth count query | Elevation of Privilege | Use pre-authorized `team_id` from `get_user_team_id`; never accept client team id |
| Open redirect | Tampering | All three redirect targets are static literal strings (`/throne?tab=…`); no user-controlled redirect destination |
| Cross-user ranked read in `progression` | Information Disclosure | Query scopes to `auth.user.id` only; no user id param accepted |

---

## Sources

### Primary (HIGH confidence)
- `src/pages/rooms/war_room.rs` — room structure + early-return growth pattern (the one to NOT copy for Throne, D-07)
- `src/pages/rooms/library.rs:25-330` — IA-healing redirect shells (exact D-12 precedent) + room structure
- `src/pages/rooms/solo_journey.rs:18,69` — region-read-once + z-8 contrast scrim (D-15)
- `src/components/room/parallax.rs:54-353` — `AtmosphereRoot` Throne props (`:75-77`) + data attrs (`:346-348`) + `GrowthOverlay` axis doc (`:386-388`)
- `src/components/room/room_surface.rs` — `RoomSurface` API, `?tab=` resolver, keep-mounted panels
- `src/components/room/quick_nav.rs:24-124` — current QuickNav (Cmd+K/Escape); focus trap is net-new
- `src/components/nav.rs:160-225,522-557` — `HUB_ROUTES` + `hub_for_path` + sub-nav (D-11 cascade)
- `src/models/growth.rs` — `Tier`/`tier_for`/`from_counts` pattern; Library tests as template
- `src/server/db.rs:608,3005,3020,3301,3316,4200,5007` — all CONTEXT-cited helpers verified at/near cited lines
- `src/pages/team/dashboard.rs:617-723,1525-1526,2016` — the 2286-line split target; `TeamNotebook`; Brief view
- `src/pages/solo_dashboard.rs:71,283` — `get_growth_state` precedent + `SoloSurfaces` reuse target
- `src/pages/profile.rs:140` — `IdentitySurface` extraction source
- `src/app.rs:136,139-167` — QuickNav mount + route block (D-12 sub-route audit)
- `input.css:374-375,428-464` — `:focus-visible` rule + overlay tier selectors (Throne axes ABSENT — Pitfall 1)
- `schema.surql:115-127,263-264` — `post_game_learning.win_loss`; `ranked_snapshot.wins/losses`
- `e2e/tests/hydration-no-panic.spec.ts:64-89` — `PANIC_REGEX` + `SCOPED_ROUTES`
- `e2e/tests/pages.spec.ts:8-25` — `AUTHED_PAGES`
- `Cargo.toml:11-42` — leptos 0.8, web-sys 0.3.91 features (HtmlElement/Element enabled)
- `.claude/rules/leptos-patterns.md` #58/#44/#38; `.claude/rules/wasm-patterns.md` #35/#37/#42/#43; `.claude/rules/surreal-patterns.md` #2/#28/#29
- `.planning/phases/28-library-ia-healing/28-RESEARCH.md` — sibling depth/shape precedent

### Secondary (MEDIUM confidence)
- WCAG 2.1 SC 2.1.2 (No Keyboard Trap) + SC 2.4.3 (Focus Order) — focus-trap requirement framing [CITED: w3.org/WAI/WCAG21] (modal-trap-with-Escape is the accepted pattern; verified against codebase need, not a specific library)

### Tertiary (LOW confidence — flagged ASSUMED)
- Tier thresholds + `progression` metric choice (A1/A4) — calibration hypotheses
- `win_loss` literal (A2) + `team_member.team` field (A3) — resolvable by grep in Wave 0

---

## Metadata

**Confidence breakdown:**
- Standard stack / components: HIGH — every component verified in codebase; no new packages
- Room structure / redirects: HIGH — direct copy of `library.rs`/`war_room.rs`, read in full
- Blended growth (D-07): HIGH (mechanism + sources) / ASSUMED (thresholds + exact progression metric)
- Dashboard split: HIGH (located all parts) / MEDIUM (the extraction is large + the Brief-mode fate is open)
- QuickNav focus trap: MEDIUM — pattern is well-understood and codebase-compatible, but net-new (no in-repo precedent); the recommended wrap-on-Tab approach avoids the riskier `inert`-on-siblings path
- CSS/asset blockers: HIGH — confirmed absent by grep (Pitfall 1/3 are real, not speculative)
- Pitfalls: HIGH — Phase 27 growth-visibility-gap + Library Pitfall-5 evidence trail directly applicable

**Research date:** 2026-06-08
**Valid until:** 2026-07-08 (stable stack; 30-day window)

---

## RESEARCH COMPLETE
