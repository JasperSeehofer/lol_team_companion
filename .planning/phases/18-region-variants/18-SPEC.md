# Phase 18: Region Variants — Specification

**Created:** 2026-05-11
**Ambiguity score:** 0.12 (gate: ≤ 0.20)
**Requirements:** 8 locked

## Goal

Port the canonical Claude Design source (11 page pairs + ~24 shared primitives + per-region skeleton/empty states) into Leptos so that Demacia and Pandemonium render genuinely different visual languages on every designed page — same skeleton across regions, region prop on shared primitives flips the variant.

## Background

Phase 17 (UI Consolidation, completed 2026-05-11) ported color tokens for both regions into `input.css:193-265` and self-hosted both font families. It captured 22 visual-regression baselines — all Demacia variant only. The Phase-17 6-pillar audit (`17-UI-REVIEW.md`) flagged VC-01 as deferred: switching `[data-theme]` swaps the palette but the markup is byte-identical. The only structural region branch in the codebase is `CompanionSigil` (`src/components/ornaments.rs:128-167`).

The **canonical design source** is the Claude Design project bundle, extracted to `.local-design-source/lol-team-companion-app/` (gitignored, retrieved from `https://api.anthropic.com/v1/design/h/x-wdJ6IZxjXwAVSU-8ikoA`). Key documents within:

- **`README.md`** — design author's directive: "recreate them pixel-perfectly in whatever technology makes sense for the target codebase. Match the visual output; don't copy the prototype's internal structure unless it happens to fit."
- **`project/pages/<page>/README.md`** — per-page region grammar. Every README ends with: *"Both regions render the same skeleton — only paint, type, and ornament differ. The `region` prop on shared primitives (`Card`, `Btn`, `SectionHead`) flips the variant."*
- **`project/pages/_shared/components.jsx`** — the design-system kit (~24 region-aware primitives)
- **`project/pages/_shared/skeletons-{demacia,pandemonium}.jsx`** — per-region loading + empty states (`<PageLoading variant>`, `<PageEmpty kind>`)
- **`project/scraps/CONTENT-CONTRACT-AUDIT.md`** — falsifiable content contracts for 7 page pairs + mismatch tables identifying where each region is currently under-equipped
- **`project/RESTRUCTURE-PLAN.md`** — design author's roadmap including the 4 single-region pages that need their counterpart authored ("A for all four")

**Page-pair inventory** (11 page-pairs in canonical scope; each carries its own README + audit):

| Slug | Status in design source | Implementation cost |
|---|---|---|
| `draft-carousel` | Both regions designed; light patch (add confidence/sample-size to P, onDeck halo to D) | Port + patch |
| `draft-war-table` | Both regions designed; light patch (composition pillars + numeric score to D) | Port + patch |
| `solo-constellation` | Both regions designed; medium patch (D gains pool gaps + last-10 sequence + sort/filter; P gains tier crest + 4 stat cards) | Port + medium patch |
| `team-dashboard` | Both regions designed; HEAVY patch (Pandemonium variant has no fixtures — full rebuild of data surface) | Port + heavy patch |
| `match-report` | Both regions designed; no patch needed | Direct port |
| `tree-drafter` | Both regions designed; no patch needed | Direct port |
| `champion-pool` | Both regions designed; no patch needed | Direct port |
| `draft-ledger` | Pandemonium-only currently (brutalist ledger); Demacia (medieval double-entry ledger) to be DESIGNED + built | Design + build |
| `solo-journal` | Demacia-only currently (parchment diary); Pandemonium (photocopied fanzine) to be DESIGNED + built | Design + build |
| `solo-forge` | Pandemonium-only currently (locker/workbench); Demacia (smith's workbench) to be DESIGNED + built | Design + build |
| `team-game-day-brief` | Demacia-only currently (newspaper); Pandemonium (xeroxed match-day zine) to be DESIGNED + built | Design + build |

**Route mapping** (locked in interview Round 3):

| Existing Leptos route | Design page(s) it surfaces | Mode/variant mechanism |
|---|---|---|
| `/draft` | `draft-carousel`, `draft-war-table`, `draft-ledger` | In-page mode toggle (e.g., tabs or segmented control) |
| `/tree-drafter` | `tree-drafter` | Single design page |
| `/champion-pool` | `champion-pool` | Single design page |
| `/team/dashboard` | `team-dashboard`, `team-game-day-brief` | In-page variant selector |
| `/solo` | `solo-constellation`, `solo-forge`, `solo-journal` | Region- and user-mode-selected (constellation = default; forge = tactical mode; journal = reflection mode) |
| `/match/:id` | `match-report` | Single design page |
| `/post-game` | `match-report` (likely re-uses the same component) | Single design page; same component as `/match/:id` |
| `/game-plan` | (no design — color-only swap stays) | Color-only |
| `/profile` | (no design — color-only swap stays) | Color-only |
| `/closed-beta` | (no design — color-only swap stays; closed-beta hero shipped in 17-06) | Color-only |
| `/auth/login`, `/auth/register`, `/admin/invites`, `/legal/*`, `/stats`, `/analytics`, `/team/roster`, `/team-builder`, `/opponents`, `/action-items`, `/personal-learnings` | (no design — color-only swap stays) | Color-only |

## Requirements

1. **Shared design-system kit ported to Leptos**: ~24 region-aware primitives from `_shared/components.jsx` exist as Leptos components.
   - Current: project has `ornaments.rs` with `CompanionSigil` (region-branching), `HeraldicDivider`, `GiltCorner`, `Crown`, `FleurDeLis`, `RiotTape` (verify if present or stub). No unified `Card region+variant`, no type primitives (`Display`, `Imperial`, `Glitch`, etc.), no data-viz primitives (`Stat`, `Sparkline`, `MoodMeter`).
   - Target: ship as new Leptos components (mostly under `src/components/region/` or extended `src/components/ornaments.rs`): type primitives `Display`, `Imperial`, `H`, `Eyebrow`, `Mono`, `Glitch`; ornaments `GiltCorner`, `HeraldicDivider`, `RiotTape` (with `region` prop where applicable); layout `Card region={region} variant={…}`, `SectionHead region+tone`, `Themed` wrapper; controls `Btn region+tone`, `Badge region+tone`; data-viz `Stat`, `Sparkline`, `MoodMeter`; solo helpers `RankBadge`, `LPProgress`. Reuse existing `CompanionSigil`, role/champion icons.
   - Acceptance: each primitive listed in `_shared/components.jsx` has a Leptos counterpart that compiles cleanly; primitive that takes a `region` prop produces visibly different output for `demacia` vs `pandemonium` (verifiable in component-level snapshot tests OR by inclusion in page-level snapshots).

2. **Per-region skeleton + empty states**: `<PageLoading variant>` and `<PageEmpty kind>` Leptos components exist with per-region rendering.
   - Current: no skeleton components exist; pages either render directly or show no loading state
   - Target: ship `PageLoading` with `variant ∈ {draft, solo, team}` and `PageEmpty` with `kind ∈ {draft, matches, team, pool, scout}` — each branches per region per `_shared/skeletons-{demacia,pandemonium}.jsx` (parchment-tan shimmer + serif italic captions for Demacia; xerox grey scan-flicker + monospace `// LOADING_` caption for Pandemonium)
   - Acceptance: `<Suspense fallback=move || view! { <PageLoading variant="draft" /> }>` renders in both themes; manual visual inspection confirms regional grammar; snapshot test asserts `pixelDiffRatio(loading-demacia, loading-pandemonium) > 0.40`

3. **Port 7 ready page pairs**: the 7 fully-designed page pairs are ported into the existing Leptos routes per the route mapping above.
   - Current: `src/pages/draft.rs`, `tree_drafter.rs`, `champion_pool.rs`, `team/dashboard.rs`, `post_game.rs` (and probably `/match/:id`, `/post-game` share `match_detail.rs`) render single view trees with color-only theme swap
   - Target: each of the 7 pages reads `InitialTheme` context and composes the shared region-aware primitives such that Demacia and Pandemonium variants both render their content contract (per `CONTENT-CONTRACT-AUDIT.md`) using region-flipped paint/type/ornament. Apply the mismatch patches identified in the audit: add confidence + sample-size to `draft-carousel` Pandemonium, onDeck halo to its Demacia; add composition pillars + numeric score to `draft-war-table` Demacia; add pool-gaps + last-10 + sort/filter to `solo-constellation` Demacia, add tier crest + 4 stat cards to its Pandemonium; rebuild `team-dashboard` Pandemonium with the full data surface.
   - Acceptance: each of the 7 pages has BOTH Demacia and Pandemonium visual-regression baselines that pass; manual inspection confirms each region's content contract from `CONTENT-CONTRACT-AUDIT.md` is satisfied; snapshot pixelDiff > 0.40 between regions on each page.

4. **Design + port 4 sibling pairs**: the 4 currently-single-region pages get their missing sibling designed AND built.
   - Current: `draft-ledger/demacia.jsx` (medieval ledger), `solo-journal/pandemonium.jsx` (fanzine diary), `solo-forge/demacia.jsx` (smith's workbench), `team-game-day-brief/pandemonium.jsx` (xeroxed zine) do not exist in the design source
   - Target: design those 4 missing siblings (in the existing Claude Design project, or via inline mockup HTML/Leptos), derive their content contracts (Phase D step 1 in CONTENT-CONTRACT-AUDIT.md), then port both regions into Leptos
   - Acceptance: each of the 4 pages has BOTH Demacia and Pandemonium baselines that pass; manual inspection confirms the new sibling captures the intended grammar (medieval ledger / fanzine diary / smith's workbench / xeroxed match-day zine)

5. **In-page mode toggles for multi-design routes**: the `/draft`, `/team/dashboard`, and `/solo` routes surface multiple design pages via a user-selectable mode.
   - Current: each route renders one page; no mode mechanism
   - Target: `/draft` has a mode toggle for `carousel | war-table | ledger`; `/team/dashboard` has a variant selector for `dashboard | brief`; `/solo` has a mode selector for `constellation | forge | journal`. Selection is persisted (DB field on user, or localStorage — decided in plan-phase). Default modes per region: see RESTRUCTURE-PLAN.md.
   - Acceptance: e2e test for each route confirms mode toggle visible, click changes the rendered view, selection persists across navigation; default-mode selection documented per region.

6. **Visual-regression baseline doubling for scoped routes**: every Leptos route that surfaces a designed page has both Demacia and Pandemonium snapshot baselines committed.
   - Current: 22 Demacia-only baselines from Phase 17
   - Target: for each route mapping above (the ~8 distinct routes that surface designed pages + mode-toggle states), capture both-theme baselines; preserve all existing Phase-17 Demacia baselines verbatim. For multi-mode routes, capture one baseline per (mode × region) combination. Total estimate: ~20-25 new baselines (8 routes × 2 themes + extra mode-state combinations).
   - Acceptance: `ls e2e/tests/visual-regression.spec.ts-snapshots/ | wc -l` returns ≥ 42; `cd e2e && npx playwright test visual-regression.spec.ts` exits 0; new `region-diff.spec.ts` asserts `pixelDiffRatio(demacia, pandemonium) > 0.40` for every scoped route.

7. **Utility routes remain color-only swap (out of Phase-18 work)**: the 13+ utility routes not in the design source explicitly stay at Phase-17's color-only treatment.
   - Current: utility routes render single view trees with `[data-theme]` palette swap
   - Target: NO CHANGE to utility routes (`/auth/*`, `/admin/invites`, `/legal/*`, `/stats`, `/analytics`, `/team/roster`, `/team-builder`, `/opponents`, `/action-items`, `/personal-learnings`, `/profile`, `/closed-beta`, `/game-plan`) — they preserve Phase-17 implementation byte-for-byte except for any incidental fixes needed to keep them compiling
   - Acceptance: utility-route Demacia baselines from Phase 17 still match within `maxDiffPixelRatio: 0.02` after Phase-18 lands; `grep -rE "use_context::<InitialTheme>" src/pages/{auth,admin,team,opponents,action_items,personal_learnings,stats,analytics,profile,closed_beta,game_plan}.rs` returns ZERO hits (or only the pre-existing `CompanionSigil` indirection if it's in the nav).

8. **6-pillar audit re-run on both regions for scoped pages**: produce `18-UI-REVIEW.md` documenting Demacia + Pandemonium audit verdicts for each of the 11 scoped pages.
   - Current: `17-UI-REVIEW.md` audited Demacia only (Pandemonium VC-01 deferred)
   - Target: `.planning/phases/18-region-variants/18-UI-REVIEW.md` with per-page per-region verdicts on the 6 pillars (visual coherence, accessibility, responsiveness, information density, microinteractions, performance) for the 11 scoped page pairs. No HIGH/CRITICAL findings open. MEDIUM/LOW deferrals allowed with explicit justification.
   - Acceptance: file exists; for each of the 11 pages, both region verdicts are documented; no `Verdict.*FAIL` or `status:\s*open.*HIGH|CRITICAL` matches.

## Boundaries

**In scope:**
- ~24 region-aware shared primitives ported from `_shared/components.jsx`
- Per-region skeleton + empty-state components (3 loading variants × 2 regions + 5 empty kinds × 2 regions = 16 combinations)
- 7 ready page pairs ported into existing Leptos routes
- 4 new sibling pairs designed + built (`draft-ledger demacia`, `solo-journal pandemonium`, `solo-forge demacia`, `team-game-day-brief pandemonium`)
- Mismatch patches per `CONTENT-CONTRACT-AUDIT.md` (light/medium/heavy fixes to existing region variants)
- In-page mode toggles for `/draft`, `/team/dashboard`, `/solo` routes
- Mode persistence (DB or localStorage — decided in plan-phase)
- Visual-regression baseline doubling for scoped routes (~20-25 new baselines)
- New e2e spec `region-diff.spec.ts` enforcing `pixelDiffRatio > 0.40`
- 6-pillar audit re-run for both regions on scoped pages → `18-UI-REVIEW.md`

**Out of scope:**
- **Utility routes** (`/auth/*`, `/admin/*`, `/legal/*`, `/stats`, `/analytics`, `/team/roster`, `/team-builder`, `/opponents`, `/action-items`, `/personal-learnings`, `/profile`, `/closed-beta`, `/game-plan`) — no region structural variants here. They stay at Phase-17's color-only swap. No design exists for them; designing inferentially would be scope creep.
- **Mobile responsiveness** — Phase 18 stays desktop-first per `PROJECT.md`. Pandemonium layouts at <768px are not designed or implemented here; deferred to a post-launch milestone.
- **Animated theme transitions** — switching `[data-theme]` is instant; no cross-fade/morph animation between Demacia and Pandemonium.
- **New typeface assets** — uses only the 5 already-self-hosted families (Cinzel, Cormorant-Garamond, VT323, Inter, JetBrains-Mono). No new web-font subsets, no new ornament fonts.
- **Audio / sound design** — no region-specific sound effects.
- **Universal utility-component region branching** (Nav, ChampionPicker, DraftBoard, etc. branching internally) — the canonical design intent uses shared primitives (`Card`, `Btn`, `SectionHead`) that branch on a `region` prop; we do NOT region-branch every utility component. Universal branching was the maximalist Round-2 answer; superseded by the canonical-scope decision in Round 3.
- **Adding new Leptos routes** — route inventory stays at the Phase-17 set; multi-design routes use in-page mode toggles instead of new URLs.
- **Designing in-flight** — the 4 sibling pairs to design are scoped to fit within Phase 18, but if a sibling design requires more than one design-iteration session it falls back to a stub + deferral note.
- **Pixel-perfect 1:1 with the JSX prototypes** — per the design README, "match the visual output; don't copy the prototype's internal structure unless it happens to fit." Leptos idioms may differ from JSX while preserving the intended look.

## Constraints

- **Recursion limit 512** — `src/lib.rs` and `src/main.rs` carry `#![recursion_limit = "512"]`. Per Phase-17 plan 17-03c, a single nested `view! {}` macro can already approach the limit. Region-aware primitives must avoid nested-macro explosions; prefer top-level `move || if is_pandemonium { … }` returning `AnyView`, or push the branch INSIDE the shared primitive's body so callers don't need to branch.
- **Shared-skeleton constraint** — per design author intent: same Leptos view-tree shape on both regions; the `region` prop on primitives is the branch point. Acceptable deviation: a page may render an ENTIRELY DIFFERENT page-level component if the design source treats the pages as separate (e.g., `solo-constellation` vs `solo-forge` vs `solo-journal` — these are different design pages, not regional variants of one page).
- **All Pandemonium fonts already self-hosted** under `public/fonts/{cinzel,cormorant-garamond,inter,jetbrains-mono,vt323}/`. Do not add new font assets.
- **Inherit project guardrails**: no `outline:none` without `focus-visible:ring-*` replacement (G-12); no raw hex colors in `src/components/` or `src/pages/`; no Google Fonts CDN (G-01); no dark patterns (G-10).
- **Existing 22 Demacia baselines are sacrosanct for utility routes** — Phase-18 changes that regress any non-scoped Demacia baseline by more than 2% pixel ratio must be deliberately approved.
- **For SCOPED routes, the existing Demacia baseline may be regenerated** — porting from Claude Design source will produce structurally different markup; the Demacia baseline likely shifts even though it's the "same region". This is expected and a one-time regeneration is acceptable.
- **Dual compile target** — every change must pass both `cargo check --features ssr` AND `cargo check --features hydrate --target wasm32-unknown-unknown`.
- **`InitialTheme` context plumbing** — already set in `src/app.rs:45` and consumed by `CompanionSigil`. Phase 18 routes consume it once at page entry and pass `region: String` to primitives as a prop (avoid context calls inside primitives unless unavoidable, for SSR/hydration consistency).

## Acceptance Criteria

- [ ] All ~24 shared region-aware primitives from `_shared/components.jsx` have Leptos counterparts that compile cleanly under both `--features ssr` and `--features hydrate --target wasm32-unknown-unknown`
- [ ] `PageLoading variant` (3 variants × 2 regions = 6 combinations) and `PageEmpty kind` (5 kinds × 2 regions = 10 combinations) render; per-region `pixelDiffRatio > 0.40` in component-level snapshot tests
- [ ] Each of the 7 ready page pairs has Demacia + Pandemonium baselines committed; `pixelDiffRatio(demacia, pandemonium) > 0.40` per page
- [ ] Each of the 4 new sibling pairs has BOTH region variants designed + ported + baselined; `pixelDiffRatio > 0.40` per page
- [ ] Mismatch patches from `CONTENT-CONTRACT-AUDIT.md` applied: confidence + sample-size on `draft-carousel` Pandemonium; onDeck halo on its Demacia; composition pillars + numeric score on `draft-war-table` Demacia; pool-gaps + last-10 + sort/filter on `solo-constellation` Demacia; tier crest + 4 stat cards on its Pandemonium; full data surface rebuild on `team-dashboard` Pandemonium
- [ ] In-page mode toggle exists and works on `/draft` (carousel/war-table/ledger), `/team/dashboard` (dashboard/brief), `/solo` (constellation/forge/journal); selection persists across navigation
- [ ] `e2e/tests/visual-regression.spec.ts-snapshots/` contains baselines for both themes on every scoped route + mode-state combination; full visual-regression suite exits 0
- [ ] `e2e/tests/region-diff.spec.ts` exists; for every scoped route, asserts `pixelDiffRatio(demacia, pandemonium) > 0.40`; suite exits 0
- [ ] Utility routes (`/auth/*`, `/admin/*`, `/legal/*`, `/stats`, `/analytics`, `/team/roster`, `/team-builder`, `/opponents`, `/action-items`, `/personal-learnings`, `/profile`, `/closed-beta`, `/game-plan`) have ZERO new region conditionals — `grep -rE "is_pandemonium|theme == \"pandemonium\"" src/pages/{auth,admin,team_roster,team_builder,opponents,action_items,personal_learnings,stats,analytics,profile,closed_beta,game_plan,legal}.rs 2>/dev/null` returns no hits
- [ ] Existing Phase-17 Demacia baselines for utility routes still match within `maxDiffPixelRatio: 0.02` after Phase 18
- [ ] `.planning/phases/18-region-variants/18-UI-REVIEW.md` exists with per-page per-region verdicts for the 11 scoped pages; no `FAIL`; no open HIGH/CRITICAL
- [ ] `cargo check --features ssr` clean; `cargo check --features hydrate --target wasm32-unknown-unknown` clean; `cargo test --features ssr --lib` passes (≥ 111 tests)
- [ ] No new font files added under `public/fonts/`; no `md:` / `lg:` / `xl:` Tailwind breakpoints introduced under Pandemonium conditionals; no theme-switch animation CSS added
- [ ] User performs a manual visual review on each of the 11 scoped page pairs (the Task-5 human checkpoint pattern from Phase 17): user opens both regions side-by-side and approves "genuinely different" before phase closes

## Ambiguity Report

| Dimension          | Score | Min  | Status | Notes                                                                                                       |
|--------------------|-------|------|--------|-------------------------------------------------------------------------------------------------------------|
| Goal Clarity       | 0.92  | 0.75 | ✓      | 11 page pairs (7 ready + 4 to design), explicit per-page content contracts in CONTENT-CONTRACT-AUDIT.md     |
| Boundary Clarity   | 0.88  | 0.70 | ✓      | Utility routes explicitly excluded; mobile/animations/audio/new fonts excluded; pixel-perfect-prototype OK to deviate |
| Constraint Clarity | 0.82  | 0.65 | ✓      | Recursion-limit-512, shared-skeleton + region-prop pattern, baseline preservation for utility routes        |
| Acceptance Criteria| 0.85  | 0.70 | ✓      | 13 falsifiable checkboxes; per-page content contracts and pixelDiff > 0.40 thresholds                       |
| **Ambiguity**      | 0.12  | ≤0.20| ✓      | Three-round Socratic interview; gate passed                                                                 |

## Interview Log

| Round | Perspective         | Question summary                                                                          | Decision locked                                                                                                |
|-------|---------------------|------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------|
| 1     | Researcher          | Which routes get region variants?                                                        | Initial answer: all 22 routes (maximalist). Superseded in Round 3 after canonical source review.              |
| 1     | Researcher          | How deep should the branch go per page?                                                  | Initial answer: full structural branch. Superseded in Round 3.                                                |
| 1     | Researcher          | Should VR baselines double per page or stay Demacia-only?                                | Capture both themes — kept. Refined in Round 3 to apply only to scoped routes.                                |
| 2     | Simplifier          | Do shared utility components branch per region or stay region-neutral?                   | Initial answer: universal branching. Superseded in Round 3 — utility components stay region-neutral.          |
| 2     | Boundary Keeper     | What is explicitly OUT of scope?                                                         | Mobile responsiveness, animated theme transitions, new typefaces, audio — kept                                |
| 2     | Failure Analyst     | What's the minimum-pass acceptance criterion?                                            | grep + snapshot pixelDiff > 0.40 + final manual pass — kept                                                   |
| 3     | Researcher          | (After canonical Claude Design source surfaced) How should scope reconcile?              | Adopt canonical scope: 11 page pairs (7 ready + 4 to design) + ~24 shared primitives + per-region skeletons   |
| 3     | Boundary Keeper     | How to map design slugs to Leptos routes (existing vs new)?                              | Map design slugs to existing routes; multi-design routes use in-page mode toggles (no new URLs)               |

## Implementation hint (for plan-phase)

The canonical design source already maps cleanly to a wave-based plan:

- **18-01 Shared primitives** (Wave 1) — port `_shared/components.jsx` to Leptos: type primitives, ornaments, layout (`Card region+variant`, `SectionHead`), controls (`Btn`, `Badge`), data-viz (`Stat`, `Sparkline`, `MoodMeter`), solo helpers (`RankBadge`, `LPProgress`). Spawn `gsd-pattern-mapper` to map each existing analog (e.g., extend `ornaments.rs`).
- **18-02 Per-region skeleton + empty states** (Wave 1, parallel) — port `_shared/skeletons-{demacia,pandemonium}.jsx`. Three `PageLoading` variants + five `PageEmpty` kinds × 2 regions.
- **18-03 Port 3 no-patch pages** (Wave 2) — `match-report`, `tree-drafter`, `champion-pool`. Direct port; both regions already match content contract.
- **18-04 Port 2 light-patch pages** (Wave 2, parallel) — `draft-carousel`, `draft-war-table`. Apply small mismatch patches per audit.
- **18-05 Port 1 medium-patch page** (Wave 2, parallel) — `solo-constellation`. Apply pool-gap + sort/filter additions to D; tier crest + 4 stat cards to P.
- **18-06 Port 1 heavy-patch page** (Wave 3, blocking on 18-01/02) — `team-dashboard`. Pandemonium rebuild with full data surface (rosters with mood, captain's note, ranked bans with reasons, our-pool-ready, their pattern, threats ranking, "if you let it through" warnings).
- **18-07 Design + build 4 sibling pairs** (Wave 3, parallel) — `draft-ledger demacia`, `solo-journal pandemonium`, `solo-forge demacia`, `team-game-day-brief pandemonium`. Each sub-task: derive content contract → design (in Claude Design or inline) → port to Leptos.
- **18-08 Mode toggles + persistence** (Wave 4) — `/draft` (carousel/war-table/ledger), `/team/dashboard` (dashboard/brief), `/solo` (constellation/forge/journal). Decide DB vs localStorage in plan-phase.
- **18-09 Visual-regression baseline doubling + region-diff spec** (Wave 4, parallel) — capture all scoped Pandemonium baselines; write `region-diff.spec.ts`; preserve utility-route baselines.
- **18-10 6-pillar audit re-run + 18-UI-REVIEW.md + manual checkpoint** (Wave 5, blocking) — audit each region on each scoped page; user manual-review checkpoint.

Scale: 10 plans across 5 waves. Realistic estimate 3-4 weeks. Closed-beta launch (now Phase 24) shifts proportionally but less than the maximalist scope would have demanded.

---

*Phase: 18-region-variants*
*Spec created: 2026-05-11*
*Next step: /gsd-discuss-phase 18 — implementation decisions (e.g., where shared primitives live in `src/components/`, how mode persistence works, baseline filename convention, design workflow for the 4 new siblings)*
