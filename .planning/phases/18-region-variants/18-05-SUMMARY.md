---
phase: 18-region-variants
plan: "05"
subsystem: pages/solo_dashboard
tags: [region, solo-constellation, medium-patch, content-contract, leptos]

dependency_graph:
  requires:
    - plan: 18-01
      provides: "RankBadge, LPProgress, Card, SectionHead, Btn, Badge, Glitch, Stat primitives"
    - plan: 18-02
      provides: "PageLoading, PageEmpty skeleton components"
  provides:
    - "src/pages/solo_dashboard.rs — solo-constellation with both region variants + 5 mismatch patches"
  affects:
    - "18-07 (solo-forge/solo-journal modes consume the same page entry pattern)"
    - "18-08 (mode toggle stub at the top of SoloDashboardPage is the hook point)"

tech_stack:
  added: []
  patterns:
    - "SoloConstellationContent extracted as sub-component to avoid FnOnce closure capture in Suspense reactive block"
    - "is_demacia / is_pandemonium bool flags computed once in sub-component; .then(|| ...) for conditional sections"
    - "StoredValue::new() wrapping collect_view() output for Last-10 pip row (avoids FnOnce on non-Copy collected view)"
    - "lp=current_lp.max(0) as u32 cast for LPProgress (ranked lp is i32 in model)"

key_files:
  modified:
    - src/pages/solo_dashboard.rs

decisions:
  - "SoloConstellationContent sub-component extracted (plan step 8) — FnOnce closure safety; all owned String data passed as props"
  - "Pandemonium stat cards (KDA/CS/DMG/Vision) hardcoded placeholder values — real aggregation deferred to future analytics phase (TODO comments in source)"
  - "Pool-gap warnings hardcoded placeholder strings — real pool-gap detection deferred to future analytics phase (TODO comments in source)"
  - "LPProgress max=100 is used as a visual stand-in; actual LP max depends on queue tier (e.g. 100 LP between divisions). Good enough for visual purposes"
  - "is_demacia.then(|| ...) replaced with if/else returning AnyView for the Last-10 section to avoid FnOnce issue with collected views"

metrics:
  duration: ~30min
  completed_date: "2026-05-21"
  tasks_completed: 1
  tasks_total: 1
  files_changed: 1
---

# Phase 18, Plan 05: Solo-Constellation Page Port Summary

Solo-constellation structure ported into `/solo` (`src/pages/solo_dashboard.rs`) with region-aware layout and all 5 CONTENT-CONTRACT-AUDIT mismatch patches applied: 3 Demacia patches (pool-gap warnings, last-10 W/L sequence, sort/filter controls) + 2 Pandemonium patches (tier crest + 4 deep stat cards).

## Performance

- **Duration:** ~30 min
- **Started:** 2026-05-21
- **Completed:** 2026-05-21 (commit `c4b56a0`)
- **Tasks:** 1/1
- **Files modified:** 1 (`src/pages/solo_dashboard.rs`)

## Accomplishments

- Read `InitialTheme` region ONCE at page entry (per SPEC constraints — no context reads inside primitives)
- Added mode stub clearly marked for 18-08: `let mode: String = "constellation".to_string();` with `// === Mode selection stub — replaced in 18-08 ===` comment
- Replaced header with region-aware `Card` + `SectionHead` (Demacia: "STARS ALIGN" eyebrow + "Constellation" title; Pandemonium: "// SOLO_PROFILE" + "FORGE")
- Replaced Suspense fallback with `PageLoading region=... variant="solo"` (from 18-02)
- Updated `MatchListSection`, `LpHistoryGraph`, `GoalCards` sub-components to accept `region: String` prop
- Replaced `EmptyState`/`SkeletonCard` with `PageEmpty`/`PageLoading` from `skeleton.rs` (18-02)

## Demacia Patches (3/3 applied)

All in `SoloConstellationContent` component (rendered when `region == "demacia"`):

| Patch | Description | Source Lines |
|-------|-------------|--------------|
| (a) Pool-gap warnings | `Card variant="gilt"` with 3 hardcoded `Badge tone="warning"` + italic gap descriptions | `~885–910` |
| (b) Last-10 W/L | `Card variant="gilt"` with pip row — `bg-accent` (win) / `bg-danger` (loss) circles | `~840–878` |
| (c) Sort/filter controls | Flex row with 3 ghost `Btn` components: "By Champion", "By Queue", "By Date" | `~880–888` |

## Pandemonium Patches (2/2 applied)

All in `SoloConstellationContent` component (rendered when `region == "pandemonium"`):

| Patch | Description | Source Lines |
|-------|-------------|--------------|
| (a) Tier crest | `Glitch` label showing `"// TIER · {TIER_UPPERCASE}"` inside the rank Card | `~810–817` |
| (b) 4 deep stat cards | 2×2 grid of `Card variant="zine"` + `Stat` for KDA, CS/min, DMG Share, Vision/min | `~820–838` |

## Real Data vs. Placeholders

| Data Point | Status | Notes |
|------------|--------|-------|
| Rank tier / division | Real — from `dashboard_resource` `ranked` field | Passed to `RankBadge` + `LPProgress` |
| Last-10 win/loss | Real — from `dashboard_resource` `matches` field | Uses `m.win: bool` field |
| KDA | Placeholder `"3.42"` | TODO: compute from match summaries in future analytics phase |
| CS/min | Placeholder `"7.1"` | TODO: same as above |
| DMG Share | Placeholder `"27.3%"` | TODO: same |
| Vision/min | Placeholder `"1.4"` | TODO: same |
| Pool gaps | Placeholder strings | TODO: wire real pool-gap detection from match history |

## Mode Stub Location (for 18-08)

`src/pages/solo_dashboard.rs` lines ~220–226:

```rust
// === Mode selection stub — replaced in 18-08 ===
// 18-08 wires the toggle UI + DB persistence + resolve_mode().
// Forge + Journal modes are built in 18-07. Until 18-08, default to constellation.
let mode: String = "constellation".to_string();
let _ = mode; // consumed by 18-08
// === End mode stub ===
```

18-08 should replace these lines with `resolve_mode(stored_mode, &region, "solo")` and add the mode toggle UI.

## Sub-View Extraction

`SoloConstellationContent` sub-component was extracted per plan step 8 (FnOnce closure safety):
- Accepts: `region, ranked, matches, lp_history_resource, lp_window, queue_filter, goal_progress_resource, lp_region, goals_region`
- Location: before `GoalCards` section in `solo_dashboard.rs`
- Reason: the `Ok(data) =>` arm of the `dashboard_resource` Suspense reactive closure would be `FnOnce` if it moved owned `String`/`Vec` values directly into `view!{}`

An additional `StoredValue::new()` wrapping was needed for the Last-10 pip row (`collect_view()` returns a non-Copy type that would cause `FnOnce` inside `.then(|| view!{...})`).

## Phase 16 WR-01 Hoist Preserved

The `lp_history_resource.refetch()` and `goal_progress_resource.refetch()` calls in both the auto-sync Effect and the manual `do_sync` handler are unchanged. Both resources are declared at `SoloDashboardPage` scope and passed down as props — not re-declared inside sub-components.

## Deviations from Plan

**1. [Rule 1 - Bug] FnOnce closure capture in Suspense reactive block**
- **Found during:** Task 1 (compile errors)
- **Issue:** The `Ok(data) =>` arm of `dashboard_resource.get().map(...)` needed `Fn` but captured `String`/`Vec` by move — making it `FnOnce`
- **Fix:** Extracted `SoloConstellationContent` sub-component; moved data extraction into a `#[component]` that receives owned props
- **Files modified:** `src/pages/solo_dashboard.rs` (new sub-component added in same file)
- **Commit:** `c4b56a0`

**2. [Rule 1 - Bug] Leptos `#[prop(optional, into)]` does not accept `Some(value)` syntax**
- **Found during:** Task 1 (compile errors: "From<Option<String>> not satisfied")
- **Issue:** Plan template used `eyebrow=Some("TEXT".to_string())` and `variant=Some("gilt".to_string())` but Leptos `#[prop(optional, into)]` expects the inner value directly (e.g., `eyebrow="TEXT"`, `variant="gilt"`)
- **Fix:** Changed all affected prop calls to pass the value without `Some(...)` wrapping
- **Files modified:** `src/pages/solo_dashboard.rs`
- **Commit:** `c4b56a0`

**3. [Rule 1 - Bug] `Stat::delta` is `#[prop(optional)] Option<f32>` — accepts `f32` not `Some(f32)`**
- **Found during:** Task 1 (same compile pass)
- **Issue:** `#[prop(optional)]` (without `into`) on `Option<f32>` wraps the passed value internally; passing `Some(0.18)` results in type error
- **Fix:** Changed `delta=Some(0.18_f32)` to `delta=0.18_f32` for all four stat cards
- **Commit:** `c4b56a0`

## Known Stubs

| Stub | File | Description |
|------|------|-------------|
| `value="3.42"` (KDA) | `src/pages/solo_dashboard.rs` | Placeholder — future analytics phase wires real KDA from match history |
| `value="7.1"` (CS/min) | `src/pages/solo_dashboard.rs` | Placeholder — future analytics phase |
| `value="27.3"` (DMG Share) | `src/pages/solo_dashboard.rs` | Placeholder — future analytics phase |
| `value="1.4"` (Vision/min) | `src/pages/solo_dashboard.rs` | Placeholder — future analytics phase |
| Pool gap strings | `src/pages/solo_dashboard.rs` | Hardcoded "No reliable engage support…" etc. — future pool-gap detection |

These stubs are intentional per the plan and CONTENT-CONTRACT-AUDIT.md — the visual elements MUST be present, and the plan explicitly calls for placeholders with TODOs until a future analytics phase is built.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All new content (stat cards, pool-gap strings, sort button labels) is hard-coded English copy with no PII. Existing server fn auth gates (`auth.user.ok_or_else(...)`) are preserved verbatim.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| `src/pages/solo_dashboard.rs` | FOUND |
| Commit `c4b56a0` | FOUND |
| `grep -q "let theme = use_context::<InitialTheme>"` | PASS |
| `grep -c "use_context::<InitialTheme>"` = 1 | PASS |
| `<RankBadge` present | PASS |
| `<LPProgress region=` present | PASS |
| Pool Gaps present | PASS |
| Last 10 present | PASS |
| By Champion / By Queue / By Date present | PASS |
| TIER present | PASS |
| KDA / CS/min / DMG Share / Vision/min present | PASS |
| lp_history_resource.refetch preserved | PASS |
| No raw hex | PASS |
| `cargo check --features ssr` | PASS |
| `cargo check --features hydrate --target wasm32-unknown-unknown` | PASS |
