# Phase 27: War Room - Context

**Gathered:** 2026-06-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 27 delivers the **War Room** — a new `/war-room` route (`WarRoomRoom` page +
`WarSurfaces` component) that hosts five existing features as ARIA-tabbed surfaces
over a single parallax `AtmosphereRoot`, whose growth overlays advance by tier from
real team data. It is the **first real consumer** of the Phase 26 `RoomSurface` tab
component. Scope (ROOM-04):

- `/war-room` route + `WarRoomRoom` page composing `AtmosphereRoot` over `WarSurfaces`.
- Five surfaces in a `RoomSurface` tab strip: **Draft / Tree-Drafter / Game Plan / Opponents / Ban Priority**.
- `get_team_growth_state` server fn (wraps Phase 26's `get_team_growth_counts` + `TeamGrowthState::from_counts`).
- Atmosphere axes `tactics` / `intel` / `command` drive `data-{axis}-tier` overlay opacity.
- Placeholder gradient WebP assets at the room's asset paths (real FLUX art is Phase 30).
- `mousemove` listener-accumulation guard in `AtmosphereRoot` (first time two rooms coexist in the router).
- e2e smoke coverage for `/war-room` + hydration-no-panic canary in **both** demacia and pandemonium themes + a `/draft`↔`/war-room` round-trip listener test.

**Out of scope:** IA-healing redirects / nav changes (ROOM-07, Phase 28); QuickNav (ROOM-08, Phase 29); real art generation (Phase 30); any new feature capability inside the five surfaces.

</domain>

<decisions>
## Implementation Decisions

### Surface extraction strategy (the central decision)
- **D-01:** **Full shared extraction now.** Extract each feature's content into a chrome-free,
  reusable `*Surface` component (e.g. `DraftSurface`, `TreeDrafterSurface`, `GamePlanSurface`,
  `OpponentsSurface`, `BanPrioritySurface`). The existing flat routes (`/draft`, `/tree-drafter`,
  `/game-plan`, `/opponents`) are refactored into **thin page shells that render the same `*Surface`
  component**, and `WarSurfaces` composes all five. Single source of truth from day one; pre-pays
  the Phase 28 IA-healing convergence.
- **D-02:** **Behavior of the flat routes must be preserved** (they still resolve and render the same
  UI/data). This supersedes Phase 26's "flat routes stay unchanged internally" note: routes are now
  thin shells over shared surfaces, but their externally observable behavior is unchanged. A
  regression check on each flat route is required (`draft.rs` is ~3000 lines — extraction is the main
  risk surface this phase).
- **D-03:** "Chrome-free" = the `*Surface` component renders the feature's working UI **without** its
  page-level `<h1>`/title block, outer page padding/`min-h-screen` wrapper, or anything that assumes
  it owns the viewport. The thin flat-route shell supplies whatever page chrome the standalone route
  still needs; the War Room tab supplies none (the tab bar + atmosphere are the chrome).

### Ban Priority surface (ROOM-04 fifth tab)
- **D-04:** **Extract the full ban-priority editor** out of `draft.rs` into its own `BanPrioritySurface`.
  This is the persistent team-strategic ranked ban list (`champion` + `rank` + `reason`, backed by
  `get_ban_priorities` / `save_ban_priorities`, table `ban_priority`). The Ban Priority **tab owns
  add/edit/save** of this list.
- **D-05:** The **Draft surface keeps its own in-draft ban handling** — `banned_champions` derived from
  the live draft board slots (draft.rs:1270) and the phase-by-phase ban labels are a *different*
  concept from the strategic ban-priority list and stay with the Draft surface. Only the
  `ban_priorities` panel (the `get_ban_priorities`/`set_ban_priorities`-backed editor, draft.rs ~188–236,
  1012–1019) moves to `BanPrioritySurface`.

### Tab-switch behavior (requires a RoomSurface change)
- **D-06:** **Keep all panels mounted** (CSS-hidden when inactive) instead of `<Show>` unmount/remount.
  In-tab state persists across tab switches; this is also the safest possible behavior for Success
  Criterion 1 (zero mount churn → no atmosphere flash). **This requires modifying the Phase 26
  `RoomSurface` component** (`room_surface.rs:137` currently uses `<Show when=active>` per panel).
- **D-07:** The mounted-but-hidden inactive panels MUST use `hidden` (or `display:none`) and keep their
  `role="tabpanel"` semantics; only the active panel is visible. Inactive panels must be removed from
  the a11y/tab order (`hidden` attribute handles this).
- **D-08:** Accepted consequence: **all five surfaces mount on room entry**, so each surface's data
  `Resource`s fetch eagerly (5 team-scoped round-trips on first paint). Acceptable per the user; if it
  becomes a problem it is a later optimization, not a Phase 27 blocker.
- **D-09 (RoomSurface API note):** The current `RoomSurface` takes a single `ChildrenFn` rendered per
  panel and expects children to branch on the active-tab context. Keep-mounted distinct panels need
  each surface to render its **own** content independently. The planner decides the exact new contract
  (e.g. a `Vec<(SurfaceTab, ViewFn)>` / per-tab render prop). War Room is `RoomSurface`'s first real
  consumer (Solo Journey has **no** tabs), so production-regression risk is low — but the existing
  dummy-surface render/unit test for `RoomSurface` must be updated to the new contract.

### Growth feedback presentation
- **D-10:** **Environmental art only.** Team growth is communicated purely through the `AtmosphereRoot`
  overlays fading in by tier — **no** explicit tier badges, counts, or growth panel in the UI. Keeps
  the room diegetic.
- **D-11:** Because the art is placeholder gradients this phase, Success Criterion 3 ("a richer team
  sees more atmospheric detail") is **verified via the `data-tactics-tier` / `data-intel-tier` /
  `data-command-tier` attributes and overlay opacity**, NOT via visible text. The e2e/UAT assertion for
  criterion 3 must read those attributes (seed a Tier-2 team vs a fresh team and compare), since with
  placeholder gradients the visual delta is subtle.

### Claude's Discretion
- Default tab = first surface (Draft); `?tab=` unknown/missing → first tab (already `RoomSurface`'s D-01 behavior).
- Tab order is fixed by the roadmap: Draft, Tree-Drafter, Game Plan, Opponents, Ban Priority.
- Asset path convention: mirror `/img/rooms/solo-journey/{region}/...` → `/img/rooms/war-room/{region}/...`; placeholder gradient WebPs at those paths.
- Exact plane/overlay count and parallax depths for the War Room atmosphere (mirror `solo_journey.rs` planes + the three axis overlays).
- Internal module layout for the five `*Surface` components (e.g. keep beside their page, or a `src/pages/.../surfaces/` grouping) — planner's call; keep parallel to the `solo_dashboard` → `SoloSurfaces` precedent.
- `get_team_growth_state` placement (likely beside the War Room page, mirroring `get_growth_state` for Solo Journey) and its empty/no-team handling (return `TeamGrowthState::default()` → all tier 0, never `Err`).
- Exact `mousemove`-guard mechanism (research suggests a `static AtomicBool` per Leptos #3665).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 26 contract & research (read first)
- `.planning/phases/26-room-system-foundations/26-CONTEXT.md` — locked room-system decisions (D-01..D-12); War Room axis→data mapping (D-06/07/08), RoomSurface contract, region-read-once rule.
- `.planning/research/SUMMARY.md` — extract-then-compose pattern, build sequence, axis table, Phase C (War Room) deliverables (§134-137), Pitfall 5 (mousemove accumulation, §111).
- `.planning/research/ARCHITECTURE.md` — component contracts, `<Room>Surfaces` composition.
- `.planning/research/PITFALLS.md` — mousemove accumulation, hydration panic, asset budget.

### Worked example (the template to follow)
- `src/pages/rooms/solo_journey.rs` — region-read-once (`:18`), `Resource` growth fetch + per-axis `Signal::derive`, plane/overlay composition, asset path convention, contrast scrim, `z-10` surface layer.
- `src/pages/solo_dashboard.rs` — the `SoloSurfaces` extraction precedent (extract-then-compose).

### Components to modify
- `src/components/room/room_surface.rs` — `RoomSurface`; change `<Show>` panels (`:137`) to keep-mounted CSS-hidden (D-06/07/09); update its dummy-surface test.
- `src/components/room/parallax.rs` — `AtmosphereRoot` (axis props, `data-*-tier` emission); add `mousemove` accumulation guard (D-08 / Pitfall 5).
- `src/components/room/mod.rs` — re-exports; add `WarRoom`/`WarSurfaces` if exported here.

### Surface sources to extract (D-01..D-05)
- `src/pages/draft.rs` — extract `DraftSurface`; extract the ban-priority editor → `BanPrioritySurface` (server fns `:188-236`, panel state `:1012-1019`); KEEP `banned_champions` (`:1270`) in Draft surface (D-05).
- `src/pages/tree_drafter.rs` — extract `TreeDrafterSurface`.
- `src/pages/game_plan.rs` — extract `GamePlanSurface`.
- `src/pages/opponents.rs` — extract `OpponentsSurface`.

### Growth data (Phase 26 — already built; wire only)
- `src/models/growth.rs:109-153` — `TeamGrowthCounts`, `TeamGrowthState` (`tactics_tier`/`intel_tier`/`command_tier`, `from_counts`, `Default`). Derivation is DONE; Phase 27 adds the server fn.
- `src/server/db.rs` — `get_team_growth_counts` (Phase 26 batched query); `get_ban_priorities` (`:2274`) / `set_ban_priorities` (`:2287`) for the Ban Priority surface.

### Routing & registration
- `src/app.rs:140-165` — `<Route>` block; add `/war-room` → `WarRoomRoom` (and refactor flat routes into shells per D-01/D-02). `/journey` (`:156`) is the Solo Journey precedent.

### Hydration safety (MUST read before writing the room)
- `CLAUDE.md` gotcha #58 — read region/tier ONCE at room top level, never inside reactive primitives.
- `.claude/rules/leptos-patterns.md` #58 — same rule + correct/wrong examples + Phase 18.2 reference.
- `.claude/rules/wasm-patterns.md` #35/#37/#42/#43 — WASM safety, hydrate-guarded timers/listeners, `#[allow(unused_variables)]`.
- `.planning/phases/18.2-pandemonium-hydration-reconciliation-gap-closure-for-ui-18-1/18.2-CONTEXT.md` — root cause of the structural-branching hydration panic the canary guards against.

### e2e
- `e2e/tests/pages.spec.ts` — add `/war-room` to `AUTHED_PAGES`.
- `e2e/tests/fixtures.ts` — auth fixture; reduced-motion CDP automation pattern for a11y.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `AtmosphereRoot` / `Plane` / `GrowthOverlay` (`parallax.rs`) — generalized axis props already shipped; War Room passes `tactics`/`intel`/`command` tier signals.
- `RoomSurface` (`room_surface.rs`) — ARIA tablist + `?tab=` deep-link + `resolve_active_tab` resolver already built; needs the keep-mounted change (D-06/09).
- `TeamGrowthState::from_counts` (`growth.rs:134`) — full War Room tier derivation already unit-tested; just needs a `#[server]` wrapper.
- `get_ban_priorities` / `set_ban_priorities` (`db.rs:2274/2287`) — ban-priority persistence, ready for `BanPrioritySurface`.
- `solo_journey.rs` — a complete, working room to copy structurally.

### Established Patterns
- Extract-then-compose: `solo_dashboard.rs` → `SoloSurfaces` → `solo_journey.rs`. Apply the same to all five War Room features.
- Region read ONCE at room top → plain `region: String` prop (rule #58); only used in `src/`/`href`/`data-*`/child props — never in a reactive closure or branch.
- Server fns: DB via `use_context::<Arc<Surreal<Db>>>()`; return `Ok(default/empty)` not `Err` for no-team (rule #11/#44).
- WASM: hydrate-only signals `#[allow(unused_variables)]`; MQL/mousemove listeners `#[cfg(feature = "hydrate")]`-guarded, handles in `StoredValue`, cleaned up in `on_cleanup` (rules #37/#42/#43).

### Integration Points
- `WarRoomRoom` (new, `src/pages/rooms/war_room.rs`) reads `InitialTheme` once → `region`, fetches `get_team_growth_state` Resource, derives three tier signals, composes `AtmosphereRoot` + `WarSurfaces`. Target ≤ 200 lines (research §78).
- `WarSurfaces` (new) holds the `RoomSurface` tab strip + the five extracted `*Surface` components, all mounted.
- Flat routes in `app.rs` become thin shells rendering the shared `*Surface` components (D-01/D-02).
- `mousemove` guard lives in `AtmosphereRoot` — the round-trip e2e (`/war-room`↔`/journey` or `/draft`) verifies `getEventListeners(window).mousemove.length === 1` post-round-trip.

</code_context>

<specifics>
## Specific Ideas

- The user explicitly chose the heavier-but-cleaner path on every axis: full shared extraction (not thin wrapping), full ban-priority editor (not read-only), keep-mounted panels (not lazy `<Show>`), and purely environmental growth (no badges/panels). The room should feel diegetic; correctness of growth is proven by `data-*-tier` attributes, not visible numbers.
- `BanPriority` and the in-draft `banned_champions` are deliberately kept as separate concepts (D-05) — do not merge them.

</specifics>

<deferred>
## Deferred Ideas

- **IA-healing redirects + nav changes** (opponents folded out of its silo, legacy routes redirect) — ROOM-07, Phase 28.
- **QuickNav command palette + sub-nav suppression on room paths** — ROOM-08, Phase 29.
- **Real FLUX environment art** (replaces placeholder gradients at same paths, no code change) — Phase 30.
- **Eager-fetch optimization** for the five mounted surfaces (D-08) — only if it proves to be a problem; not a Phase 27 task.
- **In-app "Pause atmosphere" toggle** — carried from Phase 26 deferred; later a11y/settings surface.
- **Explicit growth readout / tier badges** — explicitly rejected for Phase 27 (D-10); could be revisited if placeholder-art growth proves imperceptible to beta users.

None of the above were folded into Phase 27 scope.

</deferred>

---

*Phase: 27-war-room*
*Context gathered: 2026-06-05*
