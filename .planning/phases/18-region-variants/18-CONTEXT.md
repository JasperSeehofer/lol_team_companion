# Phase 18: Region Variants - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Port the canonical Claude Design source (11 page-pairs + ~24 region-aware primitives + per-region skeletons/empty states) into Leptos so that Demacia and Pandemonium render genuinely different visual languages on every designed page. Same skeleton across regions; the `region` prop on shared primitives flips the variant. Region selection is a **global immersive shell** — the entire visual language of the app changes when the user toggles region; mode toggles on `/draft`, `/team/dashboard`, and `/solo` are per-route choices nested inside the region's grammar.

**Touches:** `src/components/region/**` (new), `src/components/ornaments.rs` (DELETED), `src/pages/{draft,tree_drafter,champion_pool,team/dashboard,post_game,match_detail}.rs` (heavy), `src/pages/solo.rs` (likely new mode-toggle wrapper), `schema.surql` (3 new user fields), `src/server/db.rs` (AppUser + 3 getter/setter pairs), `e2e/tests/visual-regression.spec.ts-snapshots/**` (subfolder layout for scoped routes), `e2e/tests/region-diff.spec.ts` (new), `.local-design-source/**` (4 new sibling files authored upstream).

**Does not include:** utility routes (`/auth/*`, `/admin/*`, `/legal/*`, `/stats`, `/analytics`, `/team/roster`, `/team-builder`, `/opponents`, `/action-items`, `/personal-learnings`, `/profile`, `/closed-beta`, `/game-plan`) — those stay at Phase-17's color-only `[data-theme]` swap. Also out: mobile responsiveness, animated theme transitions, new typefaces, audio, pixel-perfect 1:1 with JSX prototype.

</domain>

<spec_lock>
## Requirements (locked via SPEC.md)

**8 requirements are locked.** See `18-SPEC.md` for full requirements, boundaries, and acceptance criteria.

Downstream agents MUST read `18-SPEC.md` before planning or implementing. Requirements are not duplicated here.

**In scope (from SPEC.md):**
- ~24 region-aware shared primitives ported from `_shared/components.jsx`
- Per-region skeleton + empty-state components (16 combinations)
- 7 ready page pairs ported into existing Leptos routes
- 4 new sibling pairs designed + built
- Mismatch patches per `CONTENT-CONTRACT-AUDIT.md`
- In-page mode toggles for `/draft`, `/team/dashboard`, `/solo` routes + mode persistence
- Visual-regression baseline doubling for scoped routes (~20-25 new baselines)
- New e2e spec `region-diff.spec.ts` enforcing `pixelDiffRatio > 0.40`
- 6-pillar audit re-run for both regions → `18-UI-REVIEW.md`

**Out of scope (from SPEC.md):**
- Utility routes (color-only swap stays)
- Mobile responsiveness
- Animated theme transitions
- New typeface assets (5 already self-hosted families only)
- Audio / sound design
- Universal utility-component region branching (Nav, ChampionPicker, DraftBoard, etc.)
- Adding new Leptos routes (multi-design routes use in-page mode toggles)
- Pixel-perfect 1:1 with JSX prototype

</spec_lock>

<decisions>
## Implementation Decisions

### Primitive file layout

- **D-01: Per-category split under `src/components/region/`.** The ~24 region-aware primitives are organized by category into 7 sibling modules:
  - `region/typography.rs` — `Display`, `Imperial`, `H`, `Eyebrow`, `Mono`, `Glitch` (type primitives; `typography` because `type` is a Rust reserved keyword)
  - `region/ornaments.rs` — `GiltCorner`, `HeraldicDivider`, `RiotTape`, plus existing `FleurDeLis`, `Crown`, `CompanionSigil` (ALL ornaments consolidated here)
  - `region/layout.rs` — `Card region+variant`, `SectionHead region+tone`, `Themed`
  - `region/controls.rs` — `Btn region+tone`, `Badge region+tone`
  - `region/data_viz.rs` — `Stat`, `Sparkline`, `MoodMeter`
  - `region/solo.rs` — `RankBadge`, `LPProgress`
  - `region/chrome.rs` — `ChampPortrait`, `ChampTile`, `RoleIcon`, `Icon` (champion/role identity chrome)
  - `region/mod.rs` — `pub use` re-exports for each submodule
- **D-02: Old `src/components/ornaments.rs` is DELETED.** `FleurDeLis`, `Crown`, `CompanionSigil` move into `region/ornaments.rs` alongside the new primitives. All consuming imports update — approximately 7 import sites (nav, theme_toggle, app shell, pages that reference `CompanionSigil`).

### Mode persistence

- **D-03: Three new string columns on the `user` table** — matches the existing `user.theme` + `user.mode` precedent (Phase 17 D-06, Phase 12 D-04). Schema additions in `schema.surql`:
  ```surql
  DEFINE FIELD IF NOT EXISTS draft_mode ON user TYPE string DEFAULT 'auto';
  DEFINE FIELD IF NOT EXISTS team_dashboard_mode ON user TYPE string DEFAULT 'auto';
  DEFINE FIELD IF NOT EXISTS solo_mode ON user TYPE string DEFAULT 'auto';
  ```
- **D-04: Region-coupled defaults via `'auto'` sentinel + page-entry resolver.** Region selection is the global immersive shell; mode defaults follow region grammar. Resolver shape:
  ```rust
  fn resolve_mode(stored: &str, region: &str, route: Route) -> &str {
      if stored != "auto" { return stored; }
      match (route, region) {
          (Draft,         "demacia")     => "carousel",
          (Draft,         "pandemonium") => "ledger",
          (TeamDashboard, "demacia")     => "dashboard",
          (TeamDashboard, "pandemonium") => "brief",
          (Solo,          "demacia")     => "constellation",
          (Solo,          "pandemonium") => "forge", // or "journal" — confirm during 18-08
          _ => "carousel", // fallback
      }
  }
  ```
- **D-05: Explicit user pick overrides the region resolver.** Once the user clicks a mode toggle, the value is persisted in the DB and is no longer `'auto'`. Switching region thereafter does NOT change their explicit choice. Resetting to `'auto'` is not exposed in UI for v1.3 (deferred).
- **D-06: AppUser + getter/setter pairs** in `src/server/db.rs` follow the existing `set_user_theme` / `set_user_mode` pattern. Three pairs: `get_user_draft_mode` / `set_user_draft_mode`, etc.

### Visual-regression baselines

- **D-07: Subfolders per scoped route; utility routes stay flat at root.** Scoped routes nest under route-named folders inside `e2e/tests/visual-regression.spec.ts-snapshots/`:
  ```
  snapshots/
  ├── authed-action-items-chromium-linux.png      ← Phase 17, unchanged
  ├── authed-admin-invites-chromium-linux.png     ← Phase 17, unchanged
  ├── public-auth-login-chromium-linux.png        ← Phase 17, unchanged
  ├── (14 other utility baselines, unchanged)
  ├── authed-draft/
  │   ├── demacia-carousel-chromium-linux.png
  │   ├── demacia-war-table-chromium-linux.png
  │   ├── demacia-ledger-chromium-linux.png
  │   ├── pandemonium-carousel-chromium-linux.png
  │   ├── pandemonium-war-table-chromium-linux.png
  │   └── pandemonium-ledger-chromium-linux.png
  ├── authed-solo/        (6 files: 3 modes × 2 regions)
  ├── authed-team-dashboard/ (4 files: 2 modes × 2 regions)
  ├── authed-tree-drafter/   (2 files)
  ├── authed-champion-pool/  (2 files)
  ├── authed-match-detail/   (2 files)
  └── authed-post-game/      (2 files)
  ```
- **D-08: Baselines remain committed to git, NOT gitignored.** CI needs them as reference for diff comparison. Phase 17 precedent (22 PNGs flat, no LFS) continues. If post-launch churn becomes painful, options to revisit are: sibling repo referenced by hash, Git LFS scoped to `*-snapshots/**`, or perceptual diff (e.g. SSIM) instead of pixel diff. All deferred to v1.4+ if needed.
- **D-09: Baseline filename within a route subfolder = `{region}-{mode}-chromium-linux.png`** for multi-mode routes; `{region}-chromium-linux.png` for single-mode routes (tree-drafter, champion-pool, match-detail, post-game). Playwright snapshot path is passed via `toHaveScreenshot(path.join('authed-draft', 'demacia-carousel.png'))`.
- **D-10: Existing scoped-route Demacia baselines from Phase 17 (`authed-draft-chromium-linux.png`, `authed-solo-chromium-linux.png`, `authed-team-dashboard-chromium-linux.png`, `authed-tree-drafter-chromium-linux.png`, `authed-champion-pool-chromium-linux.png`, `authed-post-game-chromium-linux.png`, `authed-match-detail-chromium-linux.png`) are DELETED at the start of 18-09** — their replacements live in the new subfolder structure. This is the "one-time regeneration acceptable for scoped routes" allowance from SPEC Constraints.

### Design workflow for 4 new sibling pairs

- **D-11: Extend the Claude Design source upstream — single source of truth stays at Claude Design.** Run a Claude Design session to author the 4 missing siblings into the existing project bundle, then re-export. The 4 to author:
  - `project/pages/draft-ledger/demacia.jsx` (medieval double-entry ledger)
  - `project/pages/solo-journal/pandemonium.jsx` (photocopied fanzine diary)
  - `project/pages/solo-forge/demacia.jsx` (smith's workbench)
  - `project/pages/team-game-day-brief/pandemonium.jsx` (xeroxed match-day zine)
- **D-12: Per-sibling workflow:**
  1. Author the JSX in Claude Design
  2. Re-export the bundle (refresh `https://api.anthropic.com/v1/design/h/x-wdJ6IZxjXwAVSU-8ikoA` extraction)
  3. Re-extract to `.local-design-source/lol-team-companion-app/`
  4. Derive content contract for the new sibling (matches Phase D step 1 in `CONTENT-CONTRACT-AUDIT.md`) and append it to a new `project/scraps/CONTENT-CONTRACT-AUDIT-PHASE-D.md` or extend the existing audit
  5. Port to Leptos in 18-07 sub-tasks
- **D-13: 18-07 is GATED on `.local-design-source/lol-team-companion-app/project/pages/{draft-ledger,solo-journal,solo-forge,team-game-day-brief}/{demacia,pandemonium}.jsx` all existing.** gsd-planner sequences 18-07 after the user-driven design extension finishes. The plan-phase should include a preflight check (e.g., `ls .local-design-source/.../draft-ledger/demacia.jsx` etc.) and emit a clear error pointing to D-11 if missing.
- **D-14: User drives the Claude Design session manually.** Claude Code agents do NOT call the Claude Design API; the design author re-runs the design project in the Claude Design UI. Once siblings are authored and re-extracted, the user signals readiness (commit `.local-design-source/**` or note in STATE.md).

### Claude's Discretion

- Whether to write a project-local skill (`region-aware-primitive-pattern`) capturing the `move || if is_pandemonium { … }` pattern as a reusable snippet — gsd-planner decides during 18-01.
- Exact shape of the mode-toggle UI primitive (segmented control vs. tabs vs. dropdown) — gsd-planner decides during 18-08 based on which feels native to each region's grammar (mode toggle MAY itself be region-aware).
- Whether `/solo` Pandemonium default is `forge` or `journal` — confirm during 18-08 once siblings exist; the canonical signature for Pandemonium solo is least obvious of the three.
- Whether to add a `Region` enum (`Region::{Demacia, Pandemonium}`) for type safety vs. keep `String`-typed `region` props throughout — gsd-planner decides during 18-01. Existing `InitialTheme(String)` and `is_pandemonium = theme.0 == "pandemonium"` pattern in `src/components/ornaments.rs:128-167` is acceptable as-is; a `Region` enum would be an optional cleanup.
- Whether `/match/:id` and `/post-game` share one `match_report` component (both consume the same design page per SPEC route mapping) or stay as separate page files — gsd-planner decides during 18-03.
- Whether the existing `src/components/theme_toggle.rs` is restyled (so theme/region selection feels like an immersive ritual rather than a control) — gsd-planner / 18-08 decision.
- File location and naming for the `PageLoading` and `PageEmpty` components (under `region/` or its own `src/components/skeleton.rs`) — gsd-planner decides during 18-02.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase-scoped — LOCKED requirements (MUST read before planning)

- `.planning/phases/18-region-variants/18-SPEC.md` — **Locked requirements.** 8 numbered requirements, boundaries, constraints, acceptance criteria. **MUST read before planning.**
- `.planning/ROADMAP.md` §"Phase 18" — Goal statement and success criteria summary.

### Canonical design source (extracted bundle — primary source of truth)

- `.local-design-source/lol-team-companion-app/` — Local extraction of the Claude Design bundle (gitignored). Original source: `https://api.anthropic.com/v1/design/h/x-wdJ6IZxjXwAVSU-8ikoA` (authenticated).
- `.local-design-source/lol-team-companion-app/README.md` — Design author's directive: *"recreate them pixel-perfectly in whatever technology makes sense for the target codebase. Match the visual output; don't copy the prototype's internal structure unless it happens to fit."*
- `.local-design-source/lol-team-companion-app/project/pages/_shared/components.jsx` — The ~24 region-aware primitive kit. Source for 18-01 port.
- `.local-design-source/lol-team-companion-app/project/pages/_shared/skeletons-demacia.jsx` — Per-region loading + empty states (Demacia). Source for 18-02.
- `.local-design-source/lol-team-companion-app/project/pages/_shared/skeletons-pandemonium.jsx` — Per-region loading + empty states (Pandemonium). Source for 18-02.
- `.local-design-source/lol-team-companion-app/project/pages/{draft-carousel,draft-war-table,solo-constellation,team-dashboard,match-report,tree-drafter,champion-pool}/{demacia,pandemonium}.jsx` — 7 ready page pairs. Source for 18-03/04/05/06 ports.
- `.local-design-source/lol-team-companion-app/project/pages/{draft-ledger,solo-journal,solo-forge,team-game-day-brief}/` — currently single-region; the missing sibling for each is GATED prerequisite for 18-07 per D-13.
- `.local-design-source/lol-team-companion-app/project/pages/<slug>/README.md` (every page) — per-page region grammar definition; ends with the shared-skeleton + region-prop directive.
- `.local-design-source/lol-team-companion-app/project/scraps/CONTENT-CONTRACT-AUDIT.md` — falsifiable content contracts for 7 page pairs + mismatch tables. **Authoritative for the mismatch patches** required by SPEC Requirement 3 acceptance criteria.
- `.local-design-source/lol-team-companion-app/project/RESTRUCTURE-PLAN.md` — design author's roadmap including the 4 single-region pages that need siblings authored.

### Prior-phase context (decisions still in force)

- `.planning/phases/17-ui-consolidation/17-CONTEXT.md` — Phase 17 implementation decisions. **D-04 (demacia/pandemonium adopted, 5-accent retired), D-06 (theme persists on user.theme DB field), D-08 (self-hosted fonts), D-09 (4-hub nav adopted)** are all in force. Don't relitigate.
- `.planning/phases/17-ui-consolidation/17-UI-REVIEW.md` §VC-01 — Visual coherence finding that DEFERRED Pandemonium structural variants to this phase. Phase 18 is the closure of VC-01.
- `.planning/STATE.md` — Project state. Phase 18 inserted on 2026-05-11; subsequent phases renumbered (Bug-Report Widget → 19, Prod Hardening → 20, Access Gate → 20.1, Deploy → 21, Compliance → 22, Pre-Launch → 23, Soft Launch → 24, v1.4 Draft Integration → 25).

### Vault — design system (do NOT re-specify in Phase 18 work)

- `../professional-vault/wiki/concepts/design-system.md` — Tokens, `@theme` block, color palette. Demacia/Pandemonium tokens already added to `input.css:193-265` in Phase 17.
- `../professional-vault/wiki/concepts/ui-guidelines.md` — Component rules.
- `../professional-vault/wiki/concepts/accessibility-standards.md` — A11y standards.

### Project guardrails

- `../professional-vault/wiki/meta/guardrails.md`:
  - **G-01** — No Google Fonts CDN (already enforced; no new font assets allowed per SPEC Constraints).
  - **G-12** — No `outline:none` without `focus-visible:ring-*` replacement.
  - Raw hex banned in `src/components/` and `src/pages/` (semantic tokens only).
- `../professional-vault/wiki/meta/values-charter.md` — EU/open-source preference (relevant if any new tooling is added).

### Codebase reference points

- `src/app.rs:45` — `InitialTheme` context provider. Phase 18 routes consume this once at page entry.
- `src/app.rs` (top) — `InitialTheme(pub String)` struct definition.
- `src/components/ornaments.rs:128-167` — Existing `CompanionSigil` region-branching pattern. Reference implementation for the `move || if is_pandemonium { … } else { … }` Anyview pattern that Phase 18 primitives generalize.
- `src/components/theme_toggle.rs` — Existing 2-theme toggle (current_theme signal). Phase 17 D-04 reduced this from 5 accents to demacia+pandemonium.
- `input.css:193-265` — Demacia/Pandemonium token definitions.
- `schema.surql` — `user` table; precedent fields `mode` (DEFAULT 'solo') and `theme` (DEFAULT 'demacia'). Phase 18 adds 3 mode fields (D-03).
- `e2e/tests/visual-regression.spec.ts` — Visual-regression suite. Phase 18 extends with subfolder paths (D-07–D-09); new sibling `e2e/tests/region-diff.spec.ts` per SPEC Requirement 6.
- `e2e/tests/fixtures.ts` — `authenticatePage` helper. Phase 18 e2e tests reuse this.

### CLAUDE.md project rules (cross-cutting)

- `CLAUDE.md` §"Code Style" — Semantic tokens only; never raw hex; `text-white` exception for colored buttons; dual-target compile (SSR + WASM).
- `CLAUDE.md` §"Critical Patterns" rule 41 — Tree assembly uses `children_of` HashMap, not reversal heuristic (in case 18-06 team-dashboard rebuild touches related tree logic).
- `CLAUDE.md` §"Debugging reactive bugs without a browser" — Signal lifecycle reminder; relevant for any mode-toggle Effect work.
- `CLAUDE.md` §"Testing" rule 39 — Tests run with `--features ssr --lib`; integration tests OOM under BFD linker.

### Phase-specific outputs (will be created)

- `.planning/phases/18-region-variants/18-UI-REVIEW.md` — 6-pillar per-region audit for the 11 scoped pages. Created in 18-10. Required by SPEC Requirement 8.
- `e2e/tests/region-diff.spec.ts` — New spec asserting `pixelDiffRatio(demacia, pandemonium) > 0.40` per scoped route. Created in 18-09.
- (Optional, gsd-planner discretion) `.planning/phases/18-region-variants/18-PATTERNS.md` — pattern-mapper output from 18-01 spawning gsd-pattern-mapper.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`src/components/ornaments.rs`** — Contains `HeraldicDivider`, `GiltCorner`, `FleurDeLis`, `RiotTape`, `CompanionSigil`, `Crown`. Per D-02 this file is **DELETED** in 18-01; contents move to `src/components/region/ornaments.rs`. The existing `is_pandemonium` branch pattern in `CompanionSigil` (`src/components/ornaments.rs:128-167`) is the reference shape for all new region-aware primitives.
- **`src/app.rs`** — `InitialTheme(pub String)` context already provided. New primitives consume it ONCE at page entry; primitives themselves take `region: String` as a prop (per SPEC Constraints, for SSR/hydration consistency).
- **`src/components/champion_picker.rs`, `champion_autocomplete.rs`** — Champion identity chrome. NOT in the canonical primitive kit; these existing components stay where they are. The new `region/chrome.rs` adds `ChampPortrait`, `ChampTile`, `RoleIcon`, `Icon` as canonical-design-source counterparts; gsd-planner decides whether the existing picker/autocomplete continue alongside or are restyled using the new chrome primitives during 18-04/05.
- **`src/components/draft_board.rs`** (562 lines) — 20-slot draft board. Heavy restyle expected during 18-04 (draft-carousel) and 18-06 (team-dashboard); preserved signature, restyled internals using region primitives.
- **`src/components/tree_graph.rs`** (709 lines) — SVG tree visualization. Restyle during 18-03 (tree-drafter); `children_of` HashMap traversal stays per CLAUDE.md rule 41.
- **`src/components/theme_toggle.rs`** — 2-theme toggle. Stays; gsd-planner may restyle the toggle UI itself to feel more "ritual" per Claude's Discretion above.
- **`src/components/stat_card.rs`** — Existing stat card. May be retired or extended in favor of the new `region/data_viz.rs::Stat` primitive — gsd-planner decides during 18-01.
- **`schema.surql` `user` table** — Already has `theme` (default `'demacia'`) and `mode` (default `'solo'`). D-03 adds `draft_mode`, `team_dashboard_mode`, `solo_mode` with default `'auto'`.

### Established Patterns

- **`InitialTheme` consumed at page entry, NOT inside primitives.** Per SPEC Constraints, avoid `use_context::<InitialTheme>()` calls inside the primitive components for SSR/hydration consistency. The page reads context once and passes `region: String` down as a prop.
- **`move || if is_pandemonium { … } else { … }` returning `AnyView`.** Top-level branching in primitives to avoid nested `view! {}` macro recursion-limit-512 explosions. Reference: `src/components/ornaments.rs:128-167`.
- **Semantic tokens only** — `bg-base`, `bg-surface`, `bg-elevated`, `text-primary`, `text-secondary`, `border-divider`, `border-outline`, `bg-accent`, `text-accent-contrast`. Demacia and Pandemonium variants are `[data-theme]`-scoped redefinitions of the same semantic tokens; primitives consume tokens, never raw hex.
- **Dual-target compile** — every change must pass `cargo check --features ssr` AND `cargo check --features hydrate --target wasm32-unknown-unknown`. Unused-variable warnings under `#[cfg(feature = "hydrate")]` guards are a known pitfall (CLAUDE.md "Debugging reactive bugs").
- **DB getter/setter pair pattern** — see existing `set_user_theme` / `set_user_mode` in `src/server/db.rs`. D-06 follows this for the 3 new mode fields. Each pair: `fn get_user_<field>(db: &Db, user_id: &str) -> Result<String, DbError>` + `fn set_user_<field>(db: &Db, user_id: &str, value: &str) -> Result<(), DbError>`.
- **Per-task atomic commits** — CLAUDE.md guidance. Each 18-XX sub-task ends in an atomic commit.
- **Per-page review gate (Phase 17 D-10)** — implement page → screenshot via agent-browser → user approves → commit → next page. Continues into 18-03 through 18-07.

### Integration Points

- **`src/components/mod.rs`** — Add `pub mod region;` after the old `ornaments` line is removed.
- **`src/app.rs`** — Router stays; pages consume `InitialTheme` at entry and call the resolver (D-04) for mode selection.
- **`src/server/db.rs`** — `AppUser` struct gains 3 string fields; 3 new getter/setter pairs; existing `get_current_user` / similar return paths include the new fields.
- **`schema.surql`** — 3 new `DEFINE FIELD IF NOT EXISTS` statements (D-03). Loaded on startup via `include_str!`.
- **`e2e/tests/fixtures.ts`** — `authenticatePage` helper extended (likely) with a `setRegion(region)` and `setMode(route, mode)` helper so tests can drive both regions and all mode combinations for baseline capture.
- **`e2e/tests/visual-regression.spec.ts`** — Restructured per D-07 (subfolder paths for scoped routes); utility-route assertions unchanged.
- **`e2e/tests/region-diff.spec.ts`** — NEW spec; loads both regions, captures full-page screenshots, asserts `pixelDiffRatio > 0.40`. Per SPEC Requirement 6.

### Codebase scale (relevant for plan sizing)

- Phase 17 shipped 10 plans across 4-hub IA + foundations + closed-beta surfaces.
- Phase 18 SPEC's implementation hint estimates 10 plans across 5 waves; realistic 3-4 weeks.
- Heaviest restyle targets: `draft.rs` (3,801 LOC), `tree_drafter.rs` (1,610 LOC), `team/dashboard.rs` (~2,235 LOC heaviest historical), `champion_pool.rs` (1,356 LOC). 18-06 (team-dashboard heavy patch) is the single biggest risk.

</code_context>

<specifics>
## Specific Ideas

- **Region is a global immersive shell, not a control.** When the user switches Demacia ↔ Pandemonium, the *entire* visual language flips. Mode toggles within `/draft`, `/team/dashboard`, `/solo` are subordinate choices nested INSIDE the region's grammar. This frames the UX hierarchy: region > mode.
- **`'auto'` sentinel for mode defaults.** Stored value `'auto'` means "let the region grammar decide"; explicit user pick (anything else) sticks across region switches. Provides region-coupled defaults without making them invisible — the resolver is auditable in source.
- **One-time baseline regeneration is acceptable for scoped routes.** Phase 17 Demacia baselines for the 7 scoped routes (`authed-draft`, `authed-solo`, `authed-team-dashboard`, `authed-tree-drafter`, `authed-champion-pool`, `authed-post-game`, `authed-match-detail`) will be deleted and replaced by the new subfolder baselines during 18-09. The 14 utility-route baselines stay byte-for-byte.
- **Claude Design is the single source of truth.** The 4 missing siblings are authored in the Claude Design project (extending the existing bundle at `api.anthropic.com/v1/design/h/x-wdJ6IZxjXwAVSU-8ikoA`), not sketched in-repo. This keeps the design log unified and supports future iteration.
- **Mode-toggle UI is itself region-aware.** Per Claude's Discretion: the toggle component (segmented control on Demacia, perhaps a stamped/tab-pull on Pandemonium) should respect its region's grammar — gsd-planner decides the exact shape during 18-08.

</specifics>

<deferred>
## Deferred Ideas

- **Region enum (`Region::{Demacia, Pandemonium}`)** for type safety throughout the new primitives — captured as Claude's Discretion (D-12 in Decisions); gsd-planner can choose to add it during 18-01 if it removes string-comparison footguns. If not added now, deferrable to a post-launch cleanup phase.
- **Mode-reset UI (`'auto'` sentinel re-set)** — Once a user explicitly picks a mode, there's no UI to "go back to region default". Deferred to v1.4+; trivial to add later if users ask.
- **Per-region mode-toggle component variants** (Demacia segmented control vs Pandemonium stamped tab) — gsd-planner discretion during 18-08; can ship region-neutral first and add region-aware styling later if pain.
- **Git LFS for baseline PNGs** — Deferred unless post-launch churn exceeds ~100 MB or PR diff noise becomes a recurring complaint.
- **Perceptual SSIM diff in place of pixelDiff** — Could reduce baseline regeneration churn (small font-rendering variations wouldn't trip the threshold). Deferred to v1.4+ as a CI optimization.
- **Mobile responsiveness for Pandemonium layouts** — Out of scope per PROJECT.md and SPEC Boundaries. Stays deferred to a post-launch milestone.
- **Animated theme transitions** — Out of scope per SPEC Boundaries; theme switch stays instant.
- **Sibling repo for visual-regression baselines** — Considered as a churn mitigation; rejected for v1.3 (premature). Reconsider only if `*-snapshots/` exceeds ~100 MB.
- **Live Match overlay restyle for region grammar** — `screens/live.jsx` exists in the Phase 17 handoff bundle but is not a feature yet. Once the live-match feature ships (v1.4 or v1.5), it inherits the region primitives from Phase 18 automatically — no extra design work needed.

</deferred>

---

*Phase: 18-region-variants*
*Context gathered: 2026-05-14*
