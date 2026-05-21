---
phase: 18-region-variants
plan: 01
subsystem: ui
tags: [region, primitives, leptos, design-system, components, refactor]

# Dependency graph
requires:
  - phase: 17-ui-consolidation
    provides: Demacia + Pandemonium tokens, fonts, accent palettes wired into `[data-theme]`/`[data-accent]`
provides:
  - "src/components/region/ module (7 submodules) with ~26 region-aware primitives"
  - "Card, SectionHead, Themed, Btn, Badge, Glitch, LPProgress as region-branching primitives via AnyView/ChildrenFn"
  - "Display, Imperial, H, Eyebrow, Mono, Stat, Sparkline, MoodMeter, RankBadge, ChampPortrait, ChampTile, RoleIcon, Icon as region-neutral primitives"
  - "All ornaments (HeraldicDivider, GiltCorner, FleurDeLis, RiotTape, Crown, CompanionSigil) consolidated under region/ornaments.rs"
affects: [18-02, 18-03, 18-04, 18-05, 18-06, 18-07, 18-08]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "AnyView region-branching: move || if is_pandemonium { ... .into_any() } else { ... .into_any() }"
    - "ChildrenFn (not Children) when both region arms render children"
    - "region: String prop on every structurally branching primitive (legacy CompanionSigil keeps use_context::<InitialTheme>())"
    - "Semantic tokens only — zero raw hex in region/ submodules"
    - "G-12 focus-visible:ring-* on every interactive control (Btn)"

key-files:
  created:
    - "src/components/region/mod.rs (re-exports 7 submodules)"
    - "src/components/region/ornaments.rs (verbatim move + docstring update)"
    - "src/components/region/typography.rs (Display, Imperial, H, Eyebrow, Mono, Glitch)"
    - "src/components/region/layout.rs (Card, SectionHead, Themed)"
    - "src/components/region/controls.rs (Btn, Badge)"
    - "src/components/region/data_viz.rs (Stat, Sparkline, MoodMeter)"
    - "src/components/region/solo.rs (RankBadge, LPProgress)"
    - "src/components/region/chrome.rs (ChampPortrait, ChampTile, RoleIcon, Icon)"
  modified:
    - "src/components/mod.rs (pub mod ornaments -> pub mod region)"
    - "src/components/nav.rs, src/components/draft_board.rs, src/pages/*.rs (16 import-site updates)"
  deleted:
    - "src/components/ornaments.rs (legacy 191-line file consolidated into region/ornaments.rs)"

key-decisions:
  - "No `Region` enum added - plan left this at Claude's Discretion; `region: String` is consistent with the JSX source contract and avoids touching the existing `InitialTheme(String)` context shape"
  - "Tone palette fallbacks: tones beyond {neutral, accent} not implemented as separate semantic tokens; success/warning/danger/info Badge variants fall back to `bg-elevated text-muted` per plan acceptance guidance"
  - "Glitch and Card and SectionHead and Btn and LPProgress use ChildrenFn because both region arms call children()"
  - "PageLoading/PageEmpty location deferred to 18-02 (committed in 18-02 as src/components/skeleton.rs)"

patterns-established:
  - "Region-branching primitives: accept `region: String` prop, derive `is_pandemonium`, return `move || if is_pandemonium { view!{}.into_any() } else { view!{}.into_any() }`"
  - "ChildrenFn for any primitive where both region arms render children to avoid FnOnce move error"
  - "GiltCorner/HeraldicDivider/RiotTape reuse - Card variant=gilt composes 4 GiltCorner instances; SectionHead Demacia composes HeraldicDivider, Pandemonium composes RiotTape"
  - "Semantic-token-only constraint: any region-specific gold/ink/accent2 colors come from CSS vars via inline `style=`, never raw hex"

requirements-completed:
  - "REQ-1"

# Metrics
duration: ~70min
completed: 2026-05-14
---

# Phase 18, Plan 01: Region Primitive Kit Migration

**Established the design-system foundation for Phase 18: ~26 region-aware Leptos primitives across 7 submodules, replacing the legacy `src/components/ornaments.rs` and updating every consumer.**

## Performance

- **Duration:** ~70 min (agent walltime; SUMMARY committed retroactively after a session resume)
- **Started:** 2026-05-14 (worktree branch base `a82f0f9`)
- **Completed:** 2026-05-14 (last impl commit `e11f0d3`)
- **Tasks:** 3/3
- **Files modified:** 25 (8 created in region/, 17 import-site updates, 1 deleted, 1 mod.rs edit)

## Accomplishments

- Created `src/components/region/` module with 7 submodules (typography, layout, controls, data_viz, solo, chrome, ornaments) re-exported flat from `mod.rs`
- Ported ~26 primitives from the JSX design source: 6 typography, 4 chrome, 3 data_viz, 3 layout, 2 controls, 2 solo, 6 ornaments
- Migrated all 6 existing ornaments (HeraldicDivider, GiltCorner, FleurDeLis, RiotTape, Crown, CompanionSigil) verbatim into `region/ornaments.rs`; deleted legacy `src/components/ornaments.rs`
- Updated 16 import sites (verified via `grep -rln "use crate::components::ornaments"` returning empty) to `use crate::components::region::...`
- Both compile targets clean: `cargo check --features ssr` and `cargo check --features hydrate --target wasm32-unknown-unknown` (each task commit message confirms this)
- G-12 compliance: every interactive control has `focus-visible:ring-2 focus-visible:ring-accent/50`; no `outline:none` without ring replacement
- Zero raw hex colors in any `region/` submodule (semantic tokens + CSS vars only)

## Task Commits

Each task was committed atomically on the worktree branch:

1. **Task 1: Create region module skeleton + migrate ornaments + delete old file** — `a81a692` (refactor)
2. **Task 2: Port typography + chrome + data_viz + solo primitives** — `4b6caae` (feat)
3. **Task 3: Port Card + SectionHead + Themed + Btn + Badge** — `e11f0d3` (feat)

_Note: the SUMMARY.md commit was written by the orchestrator after a session resume. The original executor agent's session ended after the final implementation commit but before the SUMMARY write/commit step. All implementation work was already on the worktree branch and verified via spot-check before the orchestrator wrote this summary._

## Files Created

- `src/components/region/mod.rs` (15 lines) — declares 7 submodules + flat re-exports (`pub use chrome::*` etc.)
- `src/components/region/ornaments.rs` (194 lines) — verbatim move of 6 ornaments with updated docstring; `CompanionSigil` keeps its `use_context::<InitialTheme>()` pattern (legacy reference)
- `src/components/region/typography.rs` (128 lines) — Display, Imperial, H, Eyebrow, Mono, Glitch (Glitch is region-branching with `ChildrenFn`)
- `src/components/region/layout.rs` (144 lines) — Card (region+variant, ChildrenFn, gilt corners vs bracket corners), SectionHead (Eyebrow+HeraldicDivider vs Glitch+RiotTape), Themed (data-theme attribute wrapper)
- `src/components/region/controls.rs` (150 lines) — Btn (all 4 variants, G-12 focus-visible ring), Badge (tone palette with fallbacks for missing tone tokens)
- `src/components/region/data_viz.rs` (138 lines) — Stat (label+value+unit+delta), Sparkline (SVG polyline + gradient area), MoodMeter (5-segment bar)
- `src/components/region/solo.rs` (117 lines) — RankBadge (SVG shield crest), LPProgress (region-branching AnyView: Demacia gold gradient vs Pandemonium flat accent)
- `src/components/region/chrome.rs` (198 lines) — ChampPortrait (DDragon URL), ChampTile (ban/lock/dim overlays), RoleIcon (CommunityDragon mask), Icon (inline SVG set)

## Files Deleted

- `src/components/ornaments.rs` (191-line legacy file; all 6 ornaments moved verbatim into `region/ornaments.rs` per D-02)

## Files Modified (Import-Site Updates)

`src/components/mod.rs`, `src/components/nav.rs`, `src/components/draft_board.rs`,
`src/pages/draft.rs`, `src/pages/match_detail.rs`, `src/pages/profile.rs`, `src/pages/home.rs`,
`src/pages/auth/register.rs`, `src/pages/auth/login.rs`, `src/pages/stats.rs`,
`src/pages/solo_dashboard.rs`, `src/pages/team/dashboard.rs`, `src/pages/closed_beta.rs`,
`src/pages/admin/invites.rs`, `src/pages/game_plan.rs`, `src/pages/legal/impressum.rs`,
`src/pages/legal/datenschutz.rs` — each switched `use crate::components::ornaments::*` → `use crate::components::region::*` (flat re-export makes the rename a single-line change per file).

## Reachability Check

Every public primitive is reachable from `use crate::components::region::*;` via flat re-export. Subsequent Wave 2+ plans (18-03 through 18-07) can import any primitive without knowing which submodule it lives in.

## Open Items / Carry-Over

- `src/components/stat_card.rs` was intentionally left in place — to be cleaned up in 18-04 / 18-05 once the pages consuming it have migrated to `region::Stat`.
- `Region` enum was not added (Claude's Discretion call documented above). If a future plan needs compile-time region exhaustiveness (e.g., for `match region { Region::Demacia => ..., Region::Pandemonium => ... }`), that can be introduced incrementally without breaking the current `region: String` API.

## Self-Check: PASSED

- [x] All 3 tasks committed atomically
- [x] `src/components/region/` exists with all 7 submodules + mod.rs
- [x] Old `src/components/ornaments.rs` deleted
- [x] Zero remaining `use crate::components::ornaments` imports in src/
- [x] Both compile targets clean (per task commit messages, verified during execution)
- [x] G-12 focus-visible ring on Btn
- [x] No raw hex in any region/ submodule
- [x] SUMMARY.md committed before worktree removal (this commit, written by orchestrator after agent session ended)
