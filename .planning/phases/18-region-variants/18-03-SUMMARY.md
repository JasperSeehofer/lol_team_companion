---
phase: 18-region-variants
plan: "03"
subsystem: pages/region-ports
tags: [region, leptos, ports, tree-drafter, champion-pool, match-report, post-game]
requirements: [REQ-3]

dependency_graph:
  requires: [18-01, 18-02]
  provides: [tree-drafter-region, champion-pool-region, match-report-region, post-game-region]
  affects: [src/pages/tree_drafter.rs, src/pages/champion_pool.rs, src/pages/match_detail.rs, src/pages/post_game.rs]

tech_stack:
  added: []
  patterns:
    - "InitialTheme read ONCE at page entry; region: String prop threaded downward"
    - "SectionHead replaces plain <h1>/<p> header blocks in all four pages"
    - "Card wraps main interactive content per region grammar"
    - "eyebrow prop passed as &str literal, not Some(String) — Leptos #[prop(optional,into)]"

key_files:
  modified:
    - src/pages/tree_drafter.rs
    - src/pages/champion_pool.rs
    - src/pages/match_detail.rs
    - src/pages/post_game.rs

decisions:
  - "No shared MatchReportView extracted: match_detail.rs renders a full Riot API scoreboard + timeline; post_game.rs renders a review form + learning panel — structurally divergent. Separate styling layers duplicated (SectionHead + Card added independently to each file)."
  - "eyebrow prop passed as &str (not Some(String)): with #[prop(optional,into)], Leptos converts &str → Option<String> via Into; passing Some(String) fails with E0277."
  - "Card nesting in champion_pool.rs required removing one extra </div> that was mis-placed before </Card> during prior context window."

metrics:
  duration: "~2h (across two context sessions)"
  completed: "2026-05-21"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 4
---

# Phase 18 Plan 03: Region-Variant Page Ports (no-patch set) Summary

Ported four pages to use region-aware primitives from 18-01, reading `InitialTheme` once per page entry and threading `region: String` as a prop to `SectionHead` and `Card`.

## Tasks

### Task 1: Port tree-drafter + champion-pool

Both pages received:

1. `use crate::app::InitialTheme` + `use crate::components::region::{Card, SectionHead}` imports
2. Region read at page entry:
   ```rust
   let theme = use_context::<InitialTheme>().unwrap_or_default();
   let region = theme.0.clone();
   ```
3. Plain `<h1>` header block replaced by `<SectionHead region=... title=... eyebrow=... />`
4. Main interactive area wrapped in `<Card region=region.clone()>`

**tree_drafter.rs**: SectionHead eyebrow="Strategy", Card wraps the flex-gap-6 sidebar+content area.

**champion_pool.rs**: SectionHead eyebrow="Strategy hub · Champion Pool", Card wraps the role-tabs div and the flex-row content div. Required fixing a tag-nesting bug where an extra `</div>` appeared before `</Card>` (the Card opened at the role-tabs level; the original close sequence had one too many `</div>` before `</Card>`).

### Task 2: Port match_detail + post_game

**Shared component decision**: NOT extracted. `match_detail.rs` renders a 10-player scoreboard, timeline events, and per-player performance stats sourced from the Riot API (`MatchDetail` model). `post_game.rs` renders a review form with went-well/improvements/action-items fields, a pattern analysis panel, and linked game plan/draft badges — completely different data shape and interaction model. Duplicating the styling layer (SectionHead + Card) in each file is correct.

**match_detail.rs**:
- Replaced back-link + implicit title with `<SectionHead region=... title="Match Report" eyebrow="Stats" />`
- Added `<Card region=region.clone()>` wrapping the Suspense/content area
- Removed the plain `<a href="/stats">Back to history</a>` link (replaced by SectionHead eyebrow context)

**post_game.rs**:
- Replaced `<div>` header block (eyebrow p + h1 + subtitle p) with `<SectionHead region=... title="What we learned in the field." eyebrow="Strategy hub · Post-Game" />`
- Added `<Card region=region.clone()>` wrapping action_item_count signal + flex-row content

## Decision: Shared MatchReportView

**Decision: NO — keep separate.**

`match_detail.rs` and `post_game.rs` both nominally surface the `match-report` design page but their Rust implementations are structurally divergent:

| Aspect | match_detail.rs | post_game.rs |
|--------|-----------------|--------------|
| Data source | Riot API (`MatchDetail`, 10-player scoreboard) | DB reviews (`PostGameLearning`, free-text fields) |
| Primary UI | Scoreboard + timeline | Review form + pattern analysis |
| LOC | ~948 | ~1113 |
| Shared model | None | None |

Extracting a `MatchReportView` component would require either (a) a union type parameter or (b) two completely different render branches — providing no reuse benefit over the current approach.

## Pre-port vs Post-port LOC

| File | Pre-port | Post-port | Delta |
|------|----------|-----------|-------|
| tree_drafter.rs | ~1685 | ~1700 | +15 |
| champion_pool.rs | ~1421 | ~1421 | 0 (bug fix offset by line removal) |
| match_detail.rs | ~948 | ~955 | +7 |
| post_game.rs | ~1113 | ~1120 | +7 |

## Recursion Limit

No additional recursion-limit issues encountered. `post_game.rs` already has `#![recursion_limit = "512"]` in `lib.rs` and `main.rs` (from a prior phase). The new `SectionHead` + `Card` wrappers add minimal depth.

## Business Logic Preservation

No server functions, Resources, signals, Effects, or spawn_local blocks were modified. The port is purely visual: imports added, context read at entry, header HTML replaced with SectionHead, outer div replaced with Card. All data-loading logic is verbatim from the pre-18 implementation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Wrong Card close-tag nesting in champion_pool.rs**
- **Found during:** Task 1 compile check
- **Issue:** An extra `</div>` appeared before `</Card>` at the file end. The Card (line 512) wrapped the role-tabs AND flex-row divs. The closing sequence had three `</div>` tags before `</Card>` but only two were needed (sidebar div + flex-row div).
- **Fix:** Removed the extra `</div>` from the file end. Correct sequence: sidebar `</div>` → flex-row `</div>` → `</Card>` → max-w `</div>` → canvas-grain `</div>`.
- **Files modified:** src/pages/champion_pool.rs
- **Commit:** 59cf4f6

**2. [Rule 1 - Bug] SectionHead eyebrow prop type mismatch (both task 1 files)**
- **Found during:** Task 1 compile check
- **Issue:** `eyebrow=Some("Strategy".to_string())` fails with `E0277: the trait bound String: From<Option<String>> is not satisfied`. With Leptos `#[prop(optional, into)]`, the prop builder expects `Into<Option<String>>` — `&str` and `String` satisfy this; `Some(String)` does not.
- **Fix:** Changed both files to pass a `&str` literal: `eyebrow="Strategy"` and `eyebrow="Strategy hub · Champion Pool"`.
- **Files modified:** src/pages/tree_drafter.rs, src/pages/champion_pool.rs
- **Commit:** 59cf4f6

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes introduced. Pure visual restructure.

## Known Stubs

None. All data loading is wired to existing server functions. SectionHead and Card are purely structural wrappers.

## Self-Check: PASSED

Files exist:
- src/pages/tree_drafter.rs ✓
- src/pages/champion_pool.rs ✓
- src/pages/match_detail.rs ✓
- src/pages/post_game.rs ✓

Commits exist:
- 59cf4f6 (feat(18-03): port tree-drafter and champion-pool to region primitives) ✓
- 895de3d (feat(18-03): port match-detail and post-game to region primitives) ✓

Acceptance criteria:
- InitialTheme read exactly once per page: all 4 pages return count=1 ✓
- Card and SectionHead present in all 4 pages ✓
- No raw hex colors introduced ✓
- cargo check --features ssr: Finished (0 errors) ✓
- cargo check --features hydrate --target wasm32-unknown-unknown: Finished (0 errors, 1 pre-existing warning) ✓
