# Phase 29: Throne, QuickNav & Room A11Y - Context

**Gathered:** 2026-06-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 29 delivers the **third and final room** plus the app-wide nav layer and the
closed-beta accessibility gate. It is the direct sibling of Phase 27 (War Room) and
Phase 28 (Library) and reuses their established pattern. Scope (ROOM-06, ROOM-08, A11Y-02):

- **Throne room** — a new `/throne` route (`ThroneRoom` page + `ThroneSurfaces`
  component) hosting **five** existing features as ARIA-tabbed surfaces over a single
  parallax `AtmosphereRoot`, whose growth overlays advance by tier from **blended team +
  personal** data: **Identity / Dashboard / Roster / Development / Solo**.
- `get_throne_growth_state` server fn deriving the `roster` / `progression` / `crown`
  axes (Phase 26 named them; derivation is this phase's job). **Unlike War Room/Library,
  it does NOT early-return default for no-team** — `progression` is the viewing user's
  personal climb and must compute regardless of team membership (see D-09/D-10).
- **IA-healing redirects** for the Throne's source features (mirrors Phase 28 D-08):
  `/profile`, `/team/dashboard`, `/action-items` 302-redirect into their Throne surface.
- **QuickNav promotion** (ROOM-08): the existing `QuickNav` component (already mounted
  app-wide, Cmd/Ctrl+K + Escape) gains a **type-to-filter command palette**, a **focus
  trap** (the WCAG 2.1 SC 2.1.2 blocker), and **focus-restore-on-close** (SC2). The
  shared `HUB_ROUTES` constant is restructured **room-centric** (3 rooms + surface
  `?tab=` deep-links), updating BOTH QuickNav and the top sub-nav bar.
- **A11Y-02 gate:** every room (`/war-room`, `/library`, `/throne`) passes the per-route
  hydration-no-panic canary in **both** demacia and pandemonium themes; `focus-visible`
  rings preserved; WCAG AA (4.5:1) text contrast over the art via the z-8 contrast scrim,
  verified per room.

**Out of scope:** real FLUX art generation (Phase 30 — placeholder gradient WebPs only);
any NEW feature capability inside the five surfaces; a structured player-development /
training tracker beyond the existing Action Items (deferred — see Deferred); reviving the
Phase 26-deferred `team_member→user→player_match` join (the `crown` axis uses cheap
team-scoped post-game win/loss instead); War Room flat-route cleanup (`/draft`,
`/tree-drafter`, `/game-plan` remain Phase 27 thin shells, per Phase 28 D-10).

</domain>

<decisions>
## Implementation Decisions

### Throne surface roster (the five Throne tabs)
- **D-01:** The Throne hosts **exactly five surfaces, in order: Identity / Dashboard /
  Roster / Development / Solo.** This splits today's single `team/dashboard.rs` page into
  three distinct surfaces (Dashboard, Roster, Development) and adds Identity + Solo.
  - **Identity** ← `/profile` content: Riot-account link, champion-pool summary, team
    crest/banner (the "team identity" of SC1).
  - **Dashboard** ← the team dashboard *summary*: prep priorities + at-a-glance panels
    (the "team dashboard summary" of SC1).
  - **Roster** ← roster slots, coach slots, join requests.
  - **Development** ← the existing **Action Items** feature + **Team Notebook** composed
    together (see D-03).
  - **Solo** ← the existing `SoloSurfaces` (LP graph, goals, champion trends) — the
    "solo progression" of SC1.
- **D-02:** **Always show all five tabs.** A user with no team gets the existing
  empty-state CTA (UX-01 pattern) inside the team-scoped tabs (Dashboard / Roster /
  Development) — NOT conditional tab hiding. This avoids the rule #58 structural-branch
  hydration trap (no per-tab conditional rendering keyed on team/mode). The Solo and
  Identity tabs always work.
- **D-03:** **Development tab = compose existing only.** Action Items (`/action-items`,
  team-scoped, assignable, status-tracked to-dos auto-generated from post-game reviews via
  PIPE-02) is the **substance** of the tab; the **Team Notebook** (`team_note`, pinnable
  freeform notes — low standalone value per the user) folds in as supporting notes
  beside it. **Team learnings stay in the Library** (Phase 28 D-01 healed
  `PostGameLearning` into the Library Post-Game surface) — Development **deep-links** to
  them, it does NOT re-host them. **No new data models or capability.**

### Throne growth axes (blended team + personal — the first such room)
- **D-04:** **Three axes mapped to data** (Phase 26 named `roster`/`progression`/`crown`):
  - `roster` ← **team_member count** (team-scoped) → roster size.
  - `progression` ← the **viewing user's** ranked LP / rank milestones (personal /
    user-scoped) → LP milestones.
  - `crown` ← **combined team record** from **post-game review win/loss** (team-scoped).
- **D-05:** **Independent axes — blended growth.** Each axis derives from its own source
  independently. This is the FIRST room that blends team-scoped and personal data and the
  FIRST that does **not** go fully tier-0 for a teamless user:
  - No-team user: `roster` and `crown` = tier 0, but `progression` STILL grows from their
    personal LP.
  - No-rank user: `progression` = tier 0, but the team axes still grow.
  - The Throne always reflects whatever the user has actually built (team or solo).
- **D-06:** **`crown` uses cheap team-scoped post-game win/loss**, NOT the
  Phase 26 D-05-deferred `team_member→user→player_match` join. Consistent with the other
  rooms' team-scoped queries and with the Library growth source; avoids re-opening a
  deliberately-deferred expensive query.
- **D-07:** **`get_throne_growth_state` must NOT early-return `TeamGrowthState::default()`
  on no-team** the way `get_team_growth_state` / `get_library_growth_state` do. It must
  still compute `progression` from the authenticated user's ranked data even with no team;
  only the team-scoped axes (`roster`/`crown`) fall to 0 without a team. (It still must
  never `Err` on absent data — rule #44.)
- **D-08:** **Environmental art only** (Phase 27 D-10 / Phase 28 D-15): growth is
  communicated purely through `AtmosphereRoot` overlays fading in by tier — no tier
  badges, counts, or growth panel. With placeholder gradients this phase, SC1's "the
  room's environment reflects roster size, LP milestones, and combined team record" is
  verified via `data-roster-tier` / `data-progression-tier` / `data-crown-tier`
  attributes + overlay opacity (seed a high-tier scenario vs a fresh one and compare),
  NOT via visible text.

### QuickNav — app-wide command palette (ROOM-08)
- **D-09:** **Filterable command palette.** Add a text input at the top of the QuickNav
  overlay; typing filters destinations by label (e.g. "draft", "opp", "roster").
  Auto-focus the input on open. Realizes the "command palette" intent and scales as
  destinations grow.
- **D-10:** **Focus trap + focus-restore (WCAG 2.1 SC 2.1.2 — the closed-beta blocker).**
  While the overlay is open, Tab must NOT reach parallax/plane or page elements behind it
  (SC3). On Escape (and on any close), focus returns to the element that triggered the
  open (SC2) — capture `document.activeElement` on open, restore on close. Existing
  `focus-visible` rings on every item are preserved (no `outline:none` without
  replacement — guardrail G-12).
- **D-11:** **Room-centric `HUB_ROUTES` restructure.** Restructure the shared
  `HUB_ROUTES` constant (`nav.rs:160`) to three room groups — **War Room / Library /
  Throne** — each listing its surfaces via `?tab=` deep-links (e.g. War Room → Draft /
  Tree / Game Plan / Opponents / Ban Priority; Library → Stats / Match / Analytics /
  Post-Game / Learnings; Throne → Identity / Dashboard / Roster / Development / Solo). This
  updates BOTH QuickNav AND the top sub-nav bar (both read `HUB_ROUTES`). No dead or
  duplicate entries — the old flat-route links (which now 302-redirect) are removed.

### IA-healing for the Throne (decided via the Phase 28 D-08 precedent)
- **D-12:** **302-redirect the Throne's source routes into their surface**, mirroring
  Phase 28's clean canonical-home approach:
  - `/profile` → `/throne?tab=identity`
  - `/team/dashboard` → `/throne?tab=dashboard`
  - `/action-items` → `/throne?tab=development`
  - Reuse `RoomSurface`'s existing `?tab=` resolver; unknown/missing tab → first surface
    (Identity).
  - **Preserve param-carrying / create sub-routes** (Phase 28 D-11 rule): any
    `/profile/...`, `/team/...` sub-routes or create flows must keep working — left
    standalone or healed into an in-room state; must not 404 or lose their entry point.
    Planner audits `app.rs` for these before redirecting.
- **D-13:** **`/journey` (Solo Journey room) and `/throne` coexist — do NOT redirect
  `/journey`.** `/journey` is the shipped proving spike (a standalone solo-only room with
  its own `SoloSurfaces`); the Throne's **Solo** tab surfaces the same solo-progression
  content within the blended team+solo room. Both remain reachable; the nav IA (D-11)
  points at the Throne as the primary home. (Reconciling/retiring `/journey` is a possible
  later cleanup, not Phase 29 scope.)

### Accessibility (A11Y-02 — spec-locked by success criteria)
- **D-14:** Every room (`/war-room`, `/library`, `/throne`) must pass the per-route
  **hydration-no-panic canary in both demacia and pandemonium themes** (SC4): no `tachys`
  `Option::unwrap()` panic in the console, and at least one interactive element (tab
  switch or growth-state display) responds to input. `/throne` is added to
  `e2e/tests/hydration-no-panic.spec.ts`. Region read ONCE at `ThroneRoom` top → plain
  `region: String` prop (rule #58) is the structural guard.
- **D-15:** **WCAG AA (4.5:1) text contrast** on the surface layer over the art planes,
  via the z-8 contrast scrim, **verified per room** (SC5). Mirror the
  `solo_journey.rs:69` radial-gradient scrim. `focus-visible` rings preserved throughout.

### Claude's Discretion
- Exact internal module layout for the five `*Surface` components (parallel to the War
  Room / Library `*Surface` precedent); how `team/dashboard.rs` splits into Dashboard /
  Roster / Development surfaces cleanly.
- Shared-state mechanism (if any) between Throne surfaces (e.g. mode/team context) —
  planner's call; hard constraint: no structural hydration branch (rule #58).
- Asset path convention: mirror `/img/rooms/war-room/{region}/...` →
  `/img/rooms/throne/{region}/...`; placeholder gradient WebPs at those paths.
- Plane/overlay count + parallax depths for the Throne atmosphere (mirror
  `war_room.rs` / `solo_journey.rs`: 3 planes + 3 axis overlays).
- Exact `roster`/`progression`/`crown` count sources + tier thresholds (D-04), reusing
  the Phase 26 `TeamGrowthCounts` / Phase 28 `from_counts` approach. Thresholds are
  calibration hypotheses, tunable post-beta without touching room UI.
- `get_throne_growth_state` placement (beside the Throne page) and the new no-team /
  no-rank handling shape (D-07).
- Exact filter-match algorithm for the QuickNav palette (substring vs fuzzy) and overlay
  layout (D-09) — substring on label is sufficient.
- The redirect mechanism details for D-12 (which `app.rs` sub-routes need preservation).
- Whether `team_note` (Team Notebook) needs any extraction beyond moving its existing
  embedded block (`team/dashboard.rs:1526`) into the Development surface.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Sibling phases — the template to copy (read FIRST)
- `.planning/phases/27-war-room/27-CONTEXT.md` — the original multi-surface room
  pattern: extract-then-compose (D-01..D-03), keep-mounted panels (D-06..D-09),
  environmental-only growth (D-10/D-11), `get_team_growth_state` wrapper, asset path +
  atmosphere conventions, `mousemove` guard.
- `.planning/phases/28-library-ia-healing/28-CONTEXT.md` — the IA-healing precedent
  (302-redirect to intent room, D-08..D-12; preserve params, D-11), three-axis growth
  derivation (D-13..D-15), and the second `RoomSurface` consumer. The Throne mirrors all
  of this.
- `.planning/phases/26-room-system-foundations/26-CONTEXT.md` — locked room-system
  decisions; Throne axes named `roster`/`progression`/`crown` (D-03, deferred derivation
  to this phase, §114-115); `player_match` team-aggregation deferral (D-05) the `crown`
  axis works around.

### Worked examples (structural templates)
- `src/pages/rooms/war_room.rs` — `WarRoomRoom` + `WarSurfaces`: region-read-once,
  `get_team_growth_state` `Resource` fetch, per-axis `Signal::derive`, plane/overlay
  composition, ≤200-line room target, `RoomSurface` `panels: Vec<(SurfaceTab, PanelFn)>`
  usage. **Direct structural template for `ThroneRoom`/`ThroneSurfaces`.**
- `src/pages/rooms/library.rs` — second multi-surface room; five `*Surface` composition,
  shared-context provision pattern (`provide_context` before the panels vec).
- `src/pages/rooms/solo_journey.rs` — region-read-once (`:18`), plane/overlay composition,
  asset path convention, contrast scrim (`:69`), `z-10` surface layer. **The Solo tab's
  source room — its `SoloSurfaces` is reused for the Throne Solo tab (D-01).**
- `src/pages/solo_dashboard.rs` — `get_growth_state` + `SoloSurfaces` extraction precedent
  (LP graph, goals, champion trends — the Solo tab content).

### Components to reuse / extend
- `src/components/room/quick_nav.rs` — **the component to extend** (D-09/D-10): add the
  filter input + auto-focus, focus trap, focus-restore. Currently uses `HUB_ROUTES` +
  Cmd/Ctrl+K + Escape; mounted once at `app.rs:136` (the keydown `forget()` is fine —
  app-lifetime sibling of Nav).
- `src/components/nav.rs:160` — `HUB_ROUTES` (the shared nav source) — **restructure
  room-centric** (D-11); the top sub-nav bar (`nav.rs:527`) reads it too.
- `src/components/room/room_surface.rs` — `RoomSurface` (ARIA tablist, keep-mounted
  CSS-hidden panels, `?tab=` resolver) — **reuse as-is**, Throne is its third consumer.
- `src/components/room/parallax.rs` — `AtmosphereRoot` / `Plane` / `GrowthOverlay` (axis
  props, `data-*-tier` emission, mousemove guard) — pass `roster`/`progression`/`crown`
  tier signals.
- `src/components/room/mod.rs` — re-exports; add `ThroneRoom`/`ThroneSurfaces`.

### Surface sources to extract (D-01)
- `src/pages/profile.rs` — extract `IdentitySurface` (Riot link, champion-pool summary,
  team crest/banner). Heal `/profile` → `/throne?tab=identity` (D-12).
- `src/pages/team/dashboard.rs` — **split into three surfaces**: `DashboardSurface` (prep
  priorities + summary), `RosterSurface` (roster/coach slots, join requests),
  `DevelopmentSurface` (Action Items + Team Notebook at `:1526`). Heal `/team/dashboard` →
  `/throne?tab=dashboard` (D-12). Largest extraction risk surface this phase.
- `src/pages/action_items.rs` — fold into `DevelopmentSurface` (D-03). Server fns:
  `get_action_items`, `create_action_item_fn`, status/update/delete. Heal `/action-items`
  → `/throne?tab=development` (D-12).
- `src/pages/solo_dashboard.rs` — reuse `SoloSurfaces` for the Solo tab (D-01); no
  re-extraction.

### Growth data (Phase 26/27/28 — pattern to mirror)
- `src/models/growth.rs` — `TeamGrowthCounts` / `TeamGrowthState` (`from_counts`, tier
  derivation, `Default`). Add Throne axes (`roster`/`progression`/`crown`) following the
  established tier pattern (D-04). **Note the blended shape (D-05/D-07)** — progression is
  user-scoped, not team-scoped, so the Throne state may need a distinct struct or a
  user-context parameter, unlike `TeamGrowthState`.
- `src/server/db.rs` — `get_team_growth_counts` (Phase 26 batched query) as the model;
  `list_action_items` (`:3005`) / `list_open_action_items` (`:3020`) for Development;
  `list_team_notes` (`:3301`) / `list_pinned_team_notes` (`:3316`) for the Notebook;
  team-scoped post-game win/loss for `crown` (D-06, reuse the Library growth source);
  ranked-snapshot / LP data for personal `progression` (D-04). `get_growth_state`
  (solo_dashboard) is the personal-axis precedent for `progression`.

### Routing & redirects (D-11..D-13)
- `src/app.rs:136` — `QuickNav` mount point; `:140-167` — `<Route>` block. Add `/throne`
  → `ThroneRoom`; convert `/profile`, `/team/dashboard`, `/action-items` to redirects
  (preserve param/sub-routes per D-12). `/war-room`, `/library`, `/journey` are room
  precedents; **do NOT redirect `/journey`** (D-13).

### Hydration safety & a11y (MUST read before writing the room)
- `CLAUDE.md` gotcha #58 — read region/tier ONCE at room top level, never inside reactive
  primitives.
- `.claude/rules/leptos-patterns.md` #58 (region-read-once + examples), #44 (return
  `Ok(default)` not `Err` for no-team — note D-07's nuance), #38 (`recursion_limit=512`).
- `.claude/rules/wasm-patterns.md` #35/#37/#42/#43 — WASM safety, hydrate-guarded
  timers/listeners, `#[allow(unused_variables)]`.
- `.planning/phases/18.2-pandemonium-hydration-reconciliation-gap-closure-for-ui-18-1/18.2-CONTEXT.md`
  — root cause of the structural-branching hydration panic the A11Y-02 canary guards
  against (D-14).

### e2e
- `e2e/tests/pages.spec.ts` — `AUTHED_PAGES`: add `/throne`; **update** entries for routes
  now redirecting (`/profile`, `/team/dashboard`, `/action-items`) so the suite expects
  the redirect, not a standalone render.
- `e2e/tests/hydration-no-panic.spec.ts` — add `/throne` to the canary; ensure
  `/war-room` + `/library` + `/throne` all assert no-panic in **both** regions (D-14).
- `e2e/tests/fixtures.ts` — auth fixture; reduced-motion CDP automation for a11y.
- New a11y coverage: QuickNav focus-trap test (Tab stays within the open overlay;
  Escape restores focus to the trigger — D-10) and a per-room contrast-scrim assertion
  (D-15).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `AtmosphereRoot` / `Plane` / `GrowthOverlay` (`parallax.rs`) — generalized axis props +
  mousemove guard already shipped; Throne passes `roster`/`progression`/`crown` tier
  signals.
- `RoomSurface` (`room_surface.rs`) — ARIA tablist + `?tab=` deep-link + keep-mounted
  panels already built; Throne is its **third** consumer — no `RoomSurface` change needed.
- `QuickNav` (`quick_nav.rs`) — already mounted app-wide with Cmd/Ctrl+K + Escape + the
  floating launcher; Phase 29 adds filter input, focus trap, focus-restore (D-09/D-10).
- `SoloSurfaces` (`solo_dashboard.rs`) — the Solo tab content, reused verbatim (D-01).
- `Action Items` (`action_items.rs` + `db.rs:2981-3130`) — full assignable/status-tracked
  team to-do feature, ready to fold into `DevelopmentSurface`.
- `TeamGrowthState::from_counts` (`growth.rs`) — tier-derivation pattern to extend with
  Throne axes (note blended user+team shape, D-05/D-07).
- `war_room.rs` / `library.rs` — two complete working sibling rooms to copy structurally.

### Established Patterns
- Extract-then-compose: `solo_dashboard.rs`→`SoloSurfaces`, Phase 27/28's `*Surface`
  extractions. Apply identically to Identity/Dashboard/Roster/Development.
- Region read ONCE at room top → plain `region: String` prop (rule #58); used only in
  `src`/`href`/`data-*`/child props — never in a reactive closure/branch.
- IA-healing 302-redirect to intent room (Phase 28 D-08): one canonical home per feature,
  retire the standalone page, preserve param/create sub-routes (D-11).
- Server fns: DB via `use_context::<Arc<Surreal<Db>>>()`; return `Ok(default/empty)` not
  `Err` for no-team (rules #11/#44) — **except** `get_throne_growth_state` still computes
  personal `progression` for no-team users (D-07).
- WASM: hydrate-only signals `#[allow(unused_variables)]`; keyboard/focus listeners
  `#[cfg(feature="hydrate")]`-guarded (rules #37/#42/#43).
- Single shared nav source: `HUB_ROUTES` feeds both QuickNav and the top sub-nav —
  restructure once, both update (D-11).

### Integration Points
- `ThroneRoom` (new, `src/pages/rooms/throne.rs`) reads `InitialTheme` once → `region`,
  fetches `get_throne_growth_state` Resource, derives `roster`/`progression`/`crown` tier
  signals, composes `AtmosphereRoot` + `ThroneSurfaces`. Target ≤200 lines.
- `ThroneSurfaces` (new) holds the `RoomSurface` tab strip + five surfaces
  (`IdentitySurface`, `DashboardSurface`, `RosterSurface`, `DevelopmentSurface`,
  reused `SoloSurfaces`), all mounted.
- `team/dashboard.rs` splits into three `*Surface` components (Dashboard/Roster/
  Development) — main extraction risk; Team Notebook block (`:1526`) moves into
  Development.
- Flat routes in `app.rs` (`/profile`, `/team/dashboard`, `/action-items`) become 302
  redirects (D-12), preserving sub-routes. `/journey` left untouched (D-13).
- `get_throne_growth_state` (new, beside the Throne page) blends team counts (`roster`,
  `crown` via post-game win/loss) with the viewing user's personal `progression`.
- `QuickNav` filter/focus-trap logic lives in `quick_nav.rs`; the room-centric
  `HUB_ROUTES` drives its (and the sub-nav's) destination list.

</code_context>

<specifics>
## Specific Ideas

- The user reframed the Team Notebook on the spot: "what is the notebook for? The team also
  wants to track learnings and next actions… pure text notes are not worth that much." This
  drove D-03 — the **Development** tab makes the existing **Action Items** feature the
  substance and demotes the freeform notebook to a supporting role; team learnings are NOT
  duplicated (they live in the Library, Phase 28 D-01) but deep-linked.
- The user chose the richer **5-tab** decomposition (Identity / Dashboard / Roster /
  Development / Solo) over the leaner 3-tab option — splitting today's monolithic team
  dashboard into intent-named surfaces, consistent with the Phase 27/28 "each feature is a
  surface" philosophy.
- The user wants the Throne to **never feel dead** for a solo grinder: independent blended
  axes (D-05) mean personal LP progression lights the room even with no team — a deliberate
  departure from War Room/Library's all-or-nothing team-scoping.
- "Command palette" was taken literally: the user chose a **filterable** QuickNav
  (type-to-filter), not just a static list with a focus trap (D-09).

</specifics>

<deferred>
## Deferred Ideas

- **Structured player-development / training tracker** — skill goals, practice plans, a
  real development model beyond today's Action Items. The user explicitly wanted this
  noted as a future capability; it is OUT of Phase 29 scope (Development composes existing
  features only, D-03). Candidate for its own phase.
- **Retire / reconcile `/journey` into the Throne Solo tab** — the Solo Journey proving
  spike (`/journey`) overlaps the Throne's Solo surface (D-13); deduplicating them is a
  later IA cleanup, not Phase 29 scope.
- **Per-member ranked-win team record** — reviving the Phase 26-deferred
  `team_member→user→player_match` join for a "real" combined record; the `crown` axis uses
  cheap post-game win/loss instead (D-06). Add only if the proxy proves insufficient.
- **Fuzzy / command-style QuickNav matching** (actions, not just destinations) — Phase 29
  ships substring label filtering (D-09); richer command-palette behavior is a later
  enhancement.
- **Eager-fetch optimization** for the five mounted Throne surfaces (keep-mounted
  consequence, Phase 27 D-08) — only if it proves a problem.
- **Real FLUX environment art** for the Throne (replaces placeholder gradients at the same
  paths, no code change) — Phase 30.
- **Conditional/adaptive tab visibility by solo/team mode** — considered for D-02 and
  rejected this phase (hydration-branch risk); always-show-all with empty-state CTAs
  instead. Could revisit if pure-solo users find team tabs noisy.

### Reviewed Todos (not folded)
None — no pending todos matched this phase.

</deferred>

---

*Phase: 29-throne-quicknav-room-a11y*
*Context gathered: 2026-06-06*
