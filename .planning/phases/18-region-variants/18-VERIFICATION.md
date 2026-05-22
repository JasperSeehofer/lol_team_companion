---
phase: 18-region-variants
verified: 2026-05-22T12:00:00Z
status: human_needed
score: 7/8
overrides_applied: 0
human_verification:
  - test: "Manual side-by-side visual review of all 11 scoped page pairs"
    expected: >
      Each page pair renders STRUCTURALLY different markup between Demacia and
      Pandemonium (not just different colors). Specific checks per 18-UI-REVIEW.md:
      /draft carousel — confidence/sample-size on Pandemonium + onDeck halo on Demacia;
      /draft war-table — composition pillars on Demacia; /draft ledger — medieval
      double-entry (D) vs brutalist dual-column (P); /solo constellation — pool-gaps +
      last-10 + sort/filter (D) vs tier crest + 4 stat cards (P); /solo forge + journal —
      both sibling pairs render; /team/dashboard — Pandemonium renders 7-section layout
      (RiotTape, 5-col roster, MoodMeter, captain note, bans, pool-ready, threat);
      mode toggle works and persists across reload; utility routes (login, profile,
      opponents, etc.) look IDENTICAL to pre-Phase-18.
    why_human: >
      The autonomous orchestrator deferred the 18-10 Task 3 manual checkpoint per
      autonomous-mode directive. Automated pixelmatch confirms >0.5% pixel difference
      per route but cannot verify structural layout divergence, content contract
      satisfaction, or UX quality. 18-UI-REVIEW.md section "User Sign-off" is
      explicitly pending user approval.
  - test: "Verify pixelDiffRatio threshold deviation is acceptable"
    expected: >
      Confirm the REGION_DIFF_THRESHOLD = 0.005 (0.5%) in region-diff.spec.ts is
      sufficient to detect structural region differences. The SPEC (REQ-6) specified
      0.40 (40%); actual measured ratios are 1.5-2.5% per route. The 0.5% threshold
      reliably distinguishes "different" from "identical" (identical pages score ~0%).
      Decision: is 0.5% threshold acceptable given the nature of the region differences
      (font style, border style, accent color vs wholesale layout change)?
    why_human: >
      The threshold deviation from 0.40 to 0.005 was a plan-executor decision
      documented in D-THRESHOLD (18-09-SUMMARY.md). The rationale is sound —
      Phase 18 region differences are typographic/ornamental, not layout-level —
      but the original SPEC acceptance criterion literally says "> 0.40". A human
      must confirm this deviation is acceptable for the launch-readiness milestone.
gaps:
  - truth: "region-diff.spec.ts asserts pixelDiffRatio > 0.40 for every scoped (route x mode)"
    status: partial
    reason: >
      Threshold lowered from 0.40 (40%) to 0.005 (0.5%) by plan executor (documented
      deviation D-THRESHOLD in 18-09-SUMMARY.md). Actual measured pixel differences
      are 1.5-2.5% which exceeds 0.5% but not 0.40 (40%). The spec's ls|wc -l >= 42
      acceptance also fails: top-level ls returns 22 (15 flat utility PNGs + 7
      subdirectory entries), not >=42. However, total file+dir count via find is 46.
      The plan's "41" count was already an arithmetic error (6+6+4+2+2+2+2=24 not 26).
    artifacts:
      - path: "e2e/tests/region-diff.spec.ts"
        issue: "REGION_DIFF_THRESHOLD = 0.005 (0.5%) not 0.40 (40%) as SPEC requires"
    missing:
      - "Human confirmation that 0.5% threshold is acceptable for this milestone, OR update
         threshold to 0.02 (2%) which would still be met by measured 1.5-2.5% ratios"
---

# Phase 18: Region Variants — Verification Report

**Phase Goal:** Demacia and Pandemonium themes render genuinely different component compositions per region (not color-only swap), per Open-Design mockups
**Verified:** 2026-05-22
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | REQ-1: ~24 region primitives under `src/components/region/` (7 submodules) compile on both targets; old `ornaments.rs` deleted | VERIFIED | `ls src/components/region/` shows 8 files (mod.rs + 7 submodules); all key primitives found (CompanionSigil, Glitch, Card, Btn, Stat, LPProgress, ChampPortrait); `test ! -f src/components/ornaments.rs` passes; `cargo check --features ssr` and hydrate both exit 0 with only warnings |
| 2 | REQ-2: `PageLoading` + `PageEmpty` exist with per-region rendering; 4 animation keyframes in input.css | VERIFIED | `grep -q "pub fn PageLoading" src/components/skeleton.rs` passes; `is_pandemonium` branching confirmed; `@keyframes dem-shimmer`, `pan-flicker`, `pan-scan`, `pan-cursor` all found in input.css; `pub mod skeleton` in components/mod.rs |
| 3 | REQ-3: 7 ready page pairs ported (draft, solo, team/dashboard, tree_drafter, champion_pool, match_detail, post_game) — each reads `region` once at entry | VERIFIED | All 7 pages import `InitialTheme`, call `use_context::<InitialTheme>()` at page entry, and pass `region: String` to primitives; region branch counts: draft.rs (53 refs), solo_dashboard.rs (135 refs), team/dashboard.rs (65 refs), tree_drafter.rs (11 refs), champion_pool.rs (8 refs), match_detail.rs (9 refs), post_game.rs (9 refs) |
| 4 | REQ-4: 4 sibling pairs designed + ported (draft-ledger D, solo-journal P, solo-forge D, team-game-day-brief P) | VERIFIED | 18-UI-REVIEW.md documents PASS verdicts for /draft (ledger), /solo (forge), /solo (journal), /team/dashboard (brief) for both regions; baselines captured in `authed-draft/`, `authed-solo/`, `authed-team-dashboard/` subdirs; is_pandemonium branching confirmed in solo_dashboard.rs (lines 1377, 1513) and team/dashboard.rs (lines 621, 2021) |
| 5 | REQ-5: Mode toggles on /draft, /solo, /team/dashboard — schema fields, DB getters/setters, `ModeToggle` primitive, server fns with allowlist, `resolve_mode`, AppUser fields | VERIFIED | schema.surql: 3 DEFINE FIELD IF NOT EXISTS at lines 18-20; db.rs: 6 functions at lines 4527-4620; auth.rs: DbUser has Option fields, AppUser has String fields (lines 27-64); controls.rs: `pub fn ModeToggle` at line 133; server fns with CONST VALID allowlist verified in draft.rs line 895; `<ModeToggle` used in all 3 pages; `resolve_mode` at draft.rs:913 and solo_dashboard.rs:239 |
| 6 | REQ-6: Visual-regression baselines captured for all scoped routes; region-diff.spec.ts asserts pixelDiffRatio > 0.40 | PARTIAL | 39 baseline PNGs captured (15 utility flat + 24 scoped in subdirs); `region-diff.spec.ts` exists with 13 test cases covering all scoped routes + modes; BUT threshold is 0.005 (0.5%) not 0.40 (40%) — documented deviation D-THRESHOLD; SPEC `ls|wc -l >= 42` gives 22, not 42; `find` gives 46 total entries (exceeds 42 if directory entries counted) |
| 7 | REQ-7: Utility routes (15 files) have ZERO new `is_pandemonium` or `theme == "pandemonium"` references | VERIFIED | Independent grep across auth/login.rs, auth/register.rs, legal/impressum.rs, legal/datenschutz.rs, opponents.rs, stats.rs, profile.rs, home.rs returns 0 matches; 18-UI-REVIEW.md REQ-7 gate documents same for all 15 utility files; game_plan.rs confirmed clean |
| 8 | REQ-8: `18-UI-REVIEW.md` exists with 22 region-scoped 6-pillar verdicts, all PASS, 0 FAIL, 0 open HIGH/CRITICAL | VERIFIED | File exists (467 lines); 11 scoped pages × 2 regions = 22 verdicts; all 22 marked PASS; 0 FAIL, 0 HIGH/CRITICAL; 7 LOW findings (all intentional stubs documented in prior plan SUMMARYs), 1 INFO finding; User Sign-off section deferred per orchestrator |

**Score:** 7/8 truths verified (REQ-6 PARTIAL — threshold deviation + ls count deviation)

---

### Deferred Items

No items deferred to later phases. REQ-6's placeholder data stubs (MoodMeter values,
captain notes, ban reasons, etc.) in Pandemonium team-dashboard are intentional stubs
documented in 18-06-SUMMARY and 18-UI-REVIEW.md as LOW findings, awaiting "future
phases" — but those future phases are not yet numbered in ROADMAP.md, so they cannot
be matched to a specific phase for deferral tracking. They are noted as warnings below.

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/components/region/mod.rs` | Re-exports 7 submodules | VERIFIED | `pub use chrome::*` and 6 more pub use lines; `pub mod region` in components/mod.rs |
| `src/components/region/ornaments.rs` | CompanionSigil + 5 ornaments | VERIFIED | `pub fn CompanionSigil` found; HeraldicDivider, GiltCorner, RiotTape, FleurDeLis, Crown all migrated from deleted ornaments.rs |
| `src/components/region/typography.rs` | Display, Imperial, H, Eyebrow, Mono, Glitch | VERIFIED | `pub fn Glitch` found; ChildrenFn used for Glitch; into_any() branching confirmed |
| `src/components/region/layout.rs` | Card, SectionHead, Themed | VERIFIED | `pub fn Card` found; ChildrenFn + is_pandemonium + into_any() confirmed |
| `src/components/region/controls.rs` | Btn, Badge, ModeToggle | VERIFIED | `pub fn Btn`, `pub fn Badge`, `pub fn ModeToggle` all found; `focus-visible:ring-2` confirmed (G-12) |
| `src/components/region/data_viz.rs` | Stat, Sparkline, MoodMeter | VERIFIED | `pub fn Stat` found |
| `src/components/region/solo.rs` | RankBadge, LPProgress | VERIFIED | `pub fn LPProgress` found; `is_pandemonium` branching confirmed |
| `src/components/region/chrome.rs` | ChampPortrait, ChampTile, RoleIcon, Icon | VERIFIED | `pub fn ChampPortrait` found |
| `src/components/skeleton.rs` | PageLoading + PageEmpty | VERIFIED | Both fns found; `is_pandemonium` + `into_any()` + animation class refs confirmed |
| `input.css` | 4 skeleton keyframes + 2 utilities | VERIFIED | All 4 `@keyframes` and `bg-scanline` / `bg-parchment-shimmer` utilities confirmed |
| `schema.surql` | 3 mode fields on user | VERIFIED | `DEFINE FIELD IF NOT EXISTS draft_mode` + `team_dashboard_mode` + `solo_mode` at lines 18-20 |
| `src/server/db.rs` | 3 getter/setter pairs | VERIFIED | 6 functions at lines 4527-4620: get/set for draft_mode, team_dashboard_mode, solo_mode |
| `src/server/auth.rs` | DbUser + AppUser with 3 mode fields | VERIFIED | Option fields in DbUser (lines 27-29); String fields in AppUser (lines 62-64) with unwrap_or_else "auto" |
| `e2e/tests/region-diff.spec.ts` | 13 pixelmatch tests for all scoped routes | VERIFIED | File exists; 13 test cases covering /draft (3 modes), /solo (3 modes), /team/dashboard (2 modes), /tree-drafter, /champion-pool, /post-game, /match/:id, loading skeleton |
| `e2e/tests/visual-regression.spec.ts-snapshots/` | 39 PNG baselines (15 utility + 24 scoped) | VERIFIED | 39 PNGs confirmed: 15 flat utility + 24 in 7 subdirectories; total find entries = 46 |
| `.planning/phases/18-region-variants/18-UI-REVIEW.md` | 6-pillar audit with 22 verdicts | VERIFIED | 467-line file; 22 PASS verdicts, 0 FAIL, 0 HIGH/CRITICAL |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/components/mod.rs` | `src/components/region/` | `pub mod region;` | WIRED | Confirmed at line 7 of mod.rs |
| `src/pages/draft.rs` | `src/components/region::*` | `use crate::components::region::*` | WIRED | Line 1 of draft.rs imports region; `<ModeToggle` at line 1608 |
| `src/pages/solo_dashboard.rs` | `src/components/region::*` | `use crate::components::region::*` | WIRED | Line 1 of solo_dashboard.rs; `<ModeToggle` at line 403 |
| `src/pages/team/dashboard.rs` | `src/components/region::*` | `use crate::components::region::*` | WIRED | Line 1 of team/dashboard.rs; `<ModeToggle` at line 677 |
| `src/pages/draft.rs` | `src/server/db.rs::set_user_draft_mode` | `set_draft_mode_pref` server fn | WIRED | Server fn at line 888; calls `set_user_draft_mode`; uses CONST VALID allowlist |
| `src/pages/solo_dashboard.rs` | `src/server/db.rs::set_user_solo_mode` | `set_solo_mode_pref` server fn | WIRED | Server fn at line 214 |
| `src/pages/team/dashboard.rs` | `src/server/db.rs::set_user_team_dashboard_mode` | `set_team_dashboard_mode_pref` server fn | WIRED | Server fn at line 580 |
| No old import sites | `src/components/ornaments.rs` | removed imports | WIRED | `grep -rln "use crate::components::ornaments" src/` returns 0 results |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `src/pages/draft.rs` | `region` | `use_context::<InitialTheme>()` at page entry (line 931) | Yes — from SSR-set DB theme field | FLOWING |
| `src/pages/team/dashboard.rs` | `region` | `use_context::<InitialTheme>()` (line 619) | Yes | FLOWING |
| `src/pages/solo_dashboard.rs` | `region` | `use_context::<InitialTheme>()` (line 257) | Yes | FLOWING |
| `src/pages/team/dashboard.rs` | `PandemoniumTeamDashboard` — MoodMeter, captain_note, ban_reasons | Hardcoded placeholder literals | No — intentional stubs (documented) | STATIC (intentional) |
| `src/pages/solo_dashboard.rs` | Pandemonium 4-stat cards (KDA, CS-min, DMG-Share, Vision-min) | Hardcoded placeholder values | No — intentional stubs (documented) | STATIC (intentional) |

---

### Behavioral Spot-Checks

Step 7b: SKIPPED — server not running during verification. Region-diff e2e spec requires a running Playwright + server environment. Compile checks serve as the automated proxy.

---

### Probe Execution

No `scripts/*/tests/probe-*.sh` probes declared or conventional for this phase.

---

### Requirements Coverage

Phase 18 defines its own internal REQ-1 through REQ-8 in `18-SPEC.md`. These are NOT tracked in `.planning/REQUIREMENTS.md` (which covers v1.2 requirements only; Phase 18 is v1.3 and uses the SPEC's acceptance criteria as its requirement set). No orphaned requirements in REQUIREMENTS.md for Phase 18.

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| REQ-1 | 18-01 | ~24 region primitives under region/ module | SATISFIED | All 7 submodules + ~24 primitives verified |
| REQ-2 | 18-02 | PageLoading + PageEmpty per-region skeletons | SATISFIED | skeleton.rs confirmed |
| REQ-3 | 18-03, 18-04, 18-05 | 7 ready page pairs ported | SATISFIED | All 7 pages have region reads + structural branching |
| REQ-4 | 18-07 | 4 sibling pairs designed + ported | SATISFIED | All 4 siblings confirmed in UI-REVIEW verdicts |
| REQ-5 | 18-08 | Mode toggles + persistence + resolver | SATISFIED | Schema, DB, auth, ModeToggle, server fns, resolve_mode all verified |
| REQ-6 | 18-09 | Visual-regression baselines doubled; pixelDiffRatio > 0.40 | PARTIAL | Baselines exist (39 PNGs); threshold 0.005 not 0.40 — documented deviation |
| REQ-7 | 18-10 | Utility routes have zero new region conditionals | SATISFIED | 0 matches across 15+ utility files verified independently |
| REQ-8 | 18-10 | 18-UI-REVIEW.md with 22 verdicts, all PASS | SATISFIED | File confirmed; 22 PASS, 0 FAIL, 0 HIGH/CRITICAL |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/pages/team/dashboard.rs` | 908, 978, 1429, 1489, 1520, 1529, 1772, 1810, 1835, 1848, 1852, 1870, 1907, 1953, 2069, 2080 | `TODO(future phase)` markers — hardcoded placeholder data for Pandemonium 7-section (MoodMeter=0.7, captain note, ban reasons, pool-ready, opponent patterns, threats) | WARNING | Intentional placeholder stubs per 18-06-SUMMARY acceptance criteria; documented as LOW in 18-UI-REVIEW.md; no formal issue tracking numbers |
| `src/pages/solo_dashboard.rs` | Multiple | `TODO` markers for Pandemonium stat cards + pool-gap strings | WARNING | Intentional stubs per 18-05-SUMMARY; documented in 18-UI-REVIEW.md UI-18-03/04 |
| `src/pages/draft.rs` | 4403 | `TODO: populate from post-game analysis once /post-game phase lands` | INFO | No formal issue number but references a known planned feature |
| `e2e/tests/region-diff.spec.ts` | 12-17 | Threshold 0.005 vs SPEC 0.40 — comment documents this as "REQ-6 spec mismatch against implementation" | WARNING | This is the REQ-6 partial gap — needs human acceptance decision |

**Debt marker verdict:** No `TBD`, `FIXME`, or `XXX` markers found in any Phase 18 modified files. All markers are `TODO` with "future phase" references — WARNING level, not BLOCKER. The `TODO` items match the known-gaps documentation in the verification task prompt.

---

### Human Verification Required

#### 1. Manual Side-by-Side Regional Visual Review

**Test:** Start the dev server (`cargo leptos watch`), then visit each of the 11 scoped page pairs. For each, toggle between Demacia and Pandemonium using the nav region toggle.

**Pages to check:**
1. `/draft` — carousel, war-table, ledger modes in each region
2. `/solo` — constellation, forge, journal modes in each region
3. `/team/dashboard` — dashboard, brief modes in each region
4. `/tree-drafter` — both regions
5. `/champion-pool` — both regions
6. `/match/:id` (use any match ID) — both regions
7. `/post-game` — both regions

**Expected:** Each page pair renders STRUCTURALLY different (not just different colors). Specific check points:
- `/draft carousel` Pandemonium: "conf 0.71" + "1,400 similar comps" labels present
- `/draft carousel` Demacia: onDeck halo indicator (animated pulse ring) present
- `/draft war-table` Demacia: composition pillars (DPS/FRONT/POKE/UTIL with /100 unit) present
- `/solo constellation` Demacia: pool-gap warnings + last-10 W/L pip row + sort/filter controls
- `/solo constellation` Pandemonium: tier crest Glitch label + 4 stat cards (KDA/CS-min/DMG-Share/Vision-min)
- `/team/dashboard` Pandemonium: all 7 sections visible (RiotTape header, 5-col roster + MoodMeter, captain note, reasoned bans, pool-ready, their pattern, threats)
- Mode toggle works on /draft, /solo, /team/dashboard; selection persists after page reload
- Utility routes (login, profile, opponents, etc.) look identical to Phase 17

**Approval:** Type "approved" (or specify which route+mode combinations need revision) to close Phase 18.

**Why human:** The 18-10 manual checkpoint was deferred by the orchestrator per autonomous-mode directive. Automated pixelmatch confirms >0.5% pixel difference per route, but structural layout divergence and content contract satisfaction require human eyes.

#### 2. REQ-6 Threshold Deviation Acceptance

**Test:** Review the `region-diff.spec.ts` threshold decision (D-THRESHOLD in 18-09-SUMMARY.md).

**Expected:** Confirm that `REGION_DIFF_THRESHOLD = 0.005` (0.5%) is acceptable for the v1.3 launch milestone, given that Phase 18 region differences are typographic/ornamental (font, border, accent color) rather than wholesale layout changes. Alternatively, confirm that the measured 1.5-2.5% ratio still satisfies the spirit of REQ-6's "genuinely different component compositions."

**Why human:** The SPEC's acceptance criterion literally says "> 0.40"; the actual threshold is 0.005. The rationale is sound technically but the deviation from a SPEC requirement needs explicit sign-off before the phase is marked complete.

---

### Gaps Summary

**REQ-6 partial gap:** The pixel difference threshold is 0.005 (0.5%) not 0.40 (40%) as originally specified. This is a documented, intentional deviation made by the plan executor after measuring actual region differences (1.5-2.5% per route). The 40% target assumed wholesale layout swaps; Phase 18 delivers typographic/ornamental differences that are structurally real but pixel-ratio modest.

The SPEC's `ls | wc -l >= 42` acceptance is also not met as literally written (22 top-level entries). However, the `find` total of 46 entries (including directory nodes) exceeds 42, and the actual baseline matrix (24 scoped + 15 utility = 39 PNGs) satisfies the ROADMAP's "~40 snapshots total" goal. The plan's "26 scoped" count was an arithmetic error (6+6+4+2+2+2+2 = 24, not 26); the actual 24 matches the intended per-route matrix exactly.

Both gaps require human decision: (a) is the threshold acceptable, and (b) is the visual review of 11 scoped page pairs satisfactory?

The placeholder stub data (MoodMeter values, captain notes, etc.) in Pandemonium sections are pre-accepted LOW-severity findings documented in 18-06-SUMMARY and 18-UI-REVIEW.md. They do not block the phase goal — the goal is structural divergence (achieved), not data completeness.

---

_Verified: 2026-05-22_
_Verifier: Claude (gsd-verifier), model claude-sonnet-4-6_
