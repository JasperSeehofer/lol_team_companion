---
phase: 18
plan: "04"
subsystem: draft-board-region
tags: [region-variants, draft-board, leptos, components]
dependency-graph:
  requires: [18-01, 18-02]
  provides: [region-aware-draft-board, draft-carousel-view, draft-war-table-view]
  affects: [src/components/draft_board.rs, src/pages/draft.rs, src/pages/tree_drafter.rs]
tech-stack:
  added: []
  patterns:
    - static region branching via `let card_variant = if is_pandemonium { ... } else { ... }` before view! macro
    - Resource<T> single-type-param annotation (not Resource<S, T>) for Leptos 0.8 async resources
    - Pre-clone pattern for String props consumed by multiple closures in view! macro
    - AnyView dispatch via if/else + .into_any() for mode-based view switching
key-files:
  created: []
  modified:
    - src/components/draft_board.rs
    - src/pages/draft.rs
    - src/pages/tree_drafter.rs
decisions:
  - "Resource<T> not Resource<(), T>: Leptos 0.8 Resource has single type param (the Future output)"
  - "Static card_variant variable before view! for region branching avoids closure ownership issues with Fn closures"
  - "Pre-clone each String consumed by a different closure rather than relying on clone() inside view! arms"
  - "mode stub as let mode: String = carousel.to_string() — replaced by 18-08 toggle wiring"
metrics:
  duration: "~90min (including context recovery from compaction)"
  completed: "2026-05-21"
  tasks-completed: 2
  files-modified: 3
---

# Phase 18 Plan 04: DraftBoard Region Props + Draft Sub-views Summary

Region-aware DraftBoard with ChampTile portrait rendering (gilt card for Demacia, zine for Pandemonium), plus DraftCarouselView and DraftWarTableView sub-components extracted from the draft page with all three mismatch patches wired.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Add region prop + ChampTile + Card variant to DraftBoard | 5059820 |
| 2 | Extract DraftCarouselView + DraftWarTableView with mismatch patches | 09b5a3a |

## Task 1: DraftBoard Region Prop

`src/components/draft_board.rs`:
- Added `region: String` as first prop
- Replaced raw `<img>` ban/pick slots with `<ChampTile>` primitives
- Ban filled: `<ChampTile name=champ_name size=56 banned=true />` + `<FleurDeLis>` (Demacia only)
- Ban empty: `<ChampTile size=56 />`
- Pick filled: `<ChampTile name=champ_name.clone() size=56 />`
- Ban label: `"// BAN"` (Pandemonium) vs `<Eyebrow>"Forsworn"</Eyebrow>` (Demacia)
- Outer container: `<Card region=... variant=card_variant.to_string()>` where card_variant is `"zine"` or `"gilt"`
- Column headers: monospace `// BLUE_SIDE / // RED_SIDE` (Pandemonium) vs imperial `House Northwind / HeraldicDivider / House Frostbyte` (Demacia)

`src/pages/tree_drafter.rs`:
- Added `region="demacia".to_string()` placeholder to both DraftBoard usages (wired by 18-05)

## Task 2: Draft Page Sub-views

`src/pages/draft.rs`:
- Region read from `InitialTheme` context at DraftPage top
- Mode stub `let mode: String = "carousel".to_string()` (replaced by 18-08)
- Mode dispatch: `if mode == "war-table" { DraftWarTableView } else { DraftCarouselView }` via AnyView
- `DraftCarouselView`: SectionHead with eyebrow "STRATEGY"; Pandemonium patch (conf 0.71 + 1,400 similar comps); Demacia patch (w-3 h-3 rounded-full ring-2 ring-accent/60 animate-pulse onDeck indicator); DraftBoard + role overrides + slot comments
- `DraftWarTableView`: SectionHead with eyebrow "WAR TABLE"; DraftBoard; Demacia composition pillars (DPS/FRONT/POKE/UTIL Stat components, /100 unit each); Composite Score display (value 81 in font-display); role overrides + slot comments

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Leptos 0.8 Resource<T> has single type parameter**
- **Found during:** Task 2 compilation
- **Issue:** Sub-component props declared `Resource<(), Result<Vec<Champion>, ServerFnError>>` — two type params. Leptos 0.8 `Resource<T>` has only one type param T (the Future output). The two-param form resolved to `Resource<()>` whose `.get()` returns `Option<()>`, causing match arm type mismatches.
- **Fix:** Changed to `Resource<Result<Vec<Champion>, ServerFnError>>`
- **Files modified:** src/pages/draft.rs (both DraftCarouselView and DraftWarTableView)
- **Commit:** 09b5a3a

**2. [Rule 1 - Bug] `#[prop(optional, into)]` props reject `Some(val)` — pass inner value directly**
- **Found during:** Task 2 compilation (same session as Task 1 — this pattern was already discovered and documented)
- **Issue:** `eyebrow=Some("STRATEGY".to_string())` and `unit=Some("/100".to_string())` caused E0277 because `#[prop(optional, into)]` on `Option<String>` makes the builder accept `impl Into<String>`, not `Option<String>`. The builder auto-wraps in `Some`.
- **Fix:** `eyebrow="STRATEGY".to_string()`, `unit="/100".to_string()`
- **Files modified:** src/pages/draft.rs
- **Commit:** 09b5a3a

**3. [Rule 1 - Bug] Region String moved into Suspense fallback closure, unavailable for later view nodes**
- **Found during:** Task 2 compilation
- **Issue:** `<Suspense fallback=move || view! { <PageLoading region=region.clone() ... /> }>` moves `region` into the closure. Any use of `region` after the Suspense tag caused E0382 (use of moved value).
- **Fix:** Pre-clone to `region_for_loading` and `region_for_pillars` before entering view! macro; pass named clones to each closure
- **Files modified:** src/pages/draft.rs
- **Commit:** 09b5a3a

## Self-Check: PASSED

- src/components/draft_board.rs: FOUND
- src/pages/draft.rs: FOUND
- src/pages/tree_drafter.rs: FOUND
- commit 5059820: FOUND
- commit 09b5a3a: FOUND
