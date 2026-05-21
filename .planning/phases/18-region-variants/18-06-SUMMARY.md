---
phase: 18
plan: "06"
subsystem: team-dashboard-region
tags: [region-variants, team-dashboard, pandemonium, demacia, leptos, content-contract]
dependency-graph:
  requires: [18-01, 18-02, 18-04]
  provides: [region-aware-team-dashboard, pandemonium-data-surface-7-sections]
  affects: [src/pages/team/dashboard.rs]
tech-stack:
  added: []
  patterns:
    - "if is_pandemonium { view!{...}.into_any() } else { ... } dispatch at Ok(Some(...)) match arm"
    - "ChildrenFn-safe Card/Btn usage: wrap only static-content sections (no Vec::into_iter inside Card children)"
    - "Separate #[component] subcomponents (PandemoniumBansPanel, PandemoniumThreatsPanel) to avoid ChildrenFn + Vec iteration conflicts"
    - "MoodMeter with hardcoded placeholder value=0.7 + TODO comment for future vibe-check phase"
    - "RiotTape label must be &'static str literal — cannot use .to_string()"
key-files:
  created: []
  modified:
    - src/pages/team/dashboard.rs
decisions:
  - "Dispatch Pandemonium vs Demacia at the Ok(Some(...)) arm level — avoids passing Resources as component props"
  - "Card wraps only static sections (team info header); Vec-iterating sections (roster, bench, coaches) use plain divs to avoid ChildrenFn FnOnce conflict"
  - "PandemoniumTeamDashboard receives (team, members) from parent Suspense — no extra network round-trip"
  - "All 7 sections hardcoded placeholder data with TODO comments for future phases"
metrics:
  duration: "~90min (including context recovery from compaction + full reimplementation)"
  completed: "2026-05-21"
  tasks-completed: 2
  files-modified: 1
  loc-before: 1672
  loc-after: 1923
  loc-delta: "+251 lines"
---

# Phase 18 Plan 06: Team Dashboard Region Variants Summary

Team dashboard ported to region-aware dual-surface: Demacia uses gilt Card primitives with HeraldicDivider and "STRATEGY ROOM" eyebrow; Pandemonium contains a full 7-section data-surface rebuild (RiotTape header, 5-col roster with MoodMeter, captain note, reasoned bans, pool-ready indicator, their-pattern intel, threat ranking with "if you let it through" warnings).

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Port Demacia variant — InitialTheme context read, Card/Btn primitives, STRATEGY ROOM label | 92a6ee1 |
| 2 | Build Pandemonium 7-section data-surface rebuild | 92a6ee1 |
| 3 | Grep-based verification (orchestrator-overridden checkpoint) | n/a |

## Task 1: Demacia Variant

`src/pages/team/dashboard.rs`:
- Added `use crate::app::InitialTheme` + `use crate::components::region::*`
- Region read once at page entry: `let theme = use_context::<InitialTheme>().unwrap_or_default(); let region = theme.0.clone();`
- Mode stub: `let mode: String = "dashboard".to_string();` with `// 18-08 wires toggle` comment
- Dispatch in `Ok(Some(...))` match arm: `if is_pandemonium { PandemoniumTeamDashboard } else { /* original Demacia */ }`
- Team info card wrapped in `<Card region="demacia" variant="gilt">` — gilt corners + inset shadow from primitive
- Added `<HeraldicDivider>` + `<Btn region="demacia" variant="primary">Open Draft</Btn>` CTA in card footer
- "STRATEGY ROOM" eyebrow label replacing old "Roster sigil"
- All original business logic preserved: handle_join_request, role slots, drag-drop bench, edit modal, coach section, notebook, action items, post-game panel, pool gap warnings, leave team

## Task 2: Pandemonium 7-Section Rebuild

New components added to `src/pages/team/dashboard.rs`:

**`PandemoniumTeamDashboard`** — top-level Pandemonium view:
- Container: `space-y-3 bg-base bg-scanline p-4`
- Section 1: `<RiotTape width=1200 label="TEAM_BRIEF GAME_DAY" />` (static str — not String)
- Section 2: 5-col roster grid using bracket-corner divs (plain divs not Card, because roster iterates a Vec). Real member data from `team.members` partitioned by role; empty slots show "// EMPTY" glitch label. MoodMeter with hardcoded `value=0.7` per slot.
- Section 3: Captain's note with placeholder text and TODO comment for `Team.captain_note` field
- Section 5: Our Pool Ready — "4 / 5 PLAYERS · POOL FILLED" with 80% progress bar
- Section 6: Their Pattern — 3 intel lines (LAST_5_BANS, PICK_HABIT, EARLY_GAME) placeholders
- Footer: team name in `// SQUAD:` mono label

**`PandemoniumBansPanel`** (separate component, section 4):
- 4 ban entries: Yasuo, Yone, Zed, Akali — each with `<ChampTile name=... banned=true>` + reason text
- Danger badges rendered as inline spans (not Badge ChildrenFn component, to avoid FnOnce issues when inside view! alongside static data)

**`PandemoniumThreatsPanel`** (separate component, section 7):
- 3 threats: Azir (CRITICAL), Orianna (HIGH), Leona (MED)
- Each: numbered `<ChampTile>` + severity badge + "If you let it through:" warning line

## Pandemonium Data Section Status

| Section | Data Source | Status |
|---------|-------------|--------|
| 1. RiotTape header | Static label | Hardcoded |
| 2. 5-col roster | `Team` + `Vec<TeamMember>` from parent Suspense | Real data (role-partitioned) |
| 2. MoodMeter values | No team-vibe-check model yet | Hardcoded 0.7; TODO |
| 3. Captain's note | No `captain_note` field on `Team` model | Hardcoded placeholder; TODO |
| 4. Ban reasons | No ban-reasoning feature yet | Hardcoded placeholders; TODO |
| 5. Pool-ready count | No pool-fill aggregation server fn | Hardcoded 4/5; TODO |
| 6. Their Pattern | No opponent-intel resource on dashboard | Hardcoded scout lines; TODO |
| 7. Threat ranks | No threat-scoring feature yet | Hardcoded 3 entries; TODO |

## TODO List for Future Phases

- **captain_note field**: Add `captain_note: Option<String>` to `Team` model + schema migration; wire in Section 3
- **Mood / vibe-check**: Future phase adds per-player mood signal; wire MoodMeter Section 2
- **Ban reasoning**: Future ban-reasoning phase captures per-ban reason text; wire Section 4
- **Pool-ready computation**: Add `get_pool_readiness_summary()` server fn; wire Section 5
- **Opponent intel on dashboard**: Expose `get_dashboard_opponent_intel()` shortcut; wire Section 6
- **Threat scoring**: Future opponent-intel phase adds threat ranks; wire Section 7

## Orchestrator-Verified Checkpoint (Task 3)

Task 3 was a `checkpoint:human-verify` but the orchestrator authorized autonomous operation for this phase. Grep-based verification was performed instead of interactive browser approval.

All 13 grep assertions passed:

| Check | Result |
|-------|--------|
| `<MoodMeter` present (Section 2) | PASS |
| `<RiotTape` present (Section 1) | PASS |
| `FROM_THE_CAPTAIN` present (Section 3) | PASS |
| `REASONED BANS` present (Section 4) | PASS |
| `OUR POOL READY` present (Section 5) | PASS |
| `THEIR PATTERN` present (Section 6) | PASS |
| `THREATS` present (Section 7) | PASS |
| `If you let it through` present (Section 7 warnings) | PASS |
| `bg-scanline` present (Pandemonium grammar) | PASS |
| `<Card region=` present (Demacia primitive) | PASS |
| `<Btn region=` present (Demacia primitive) | PASS |
| `handle_join_request`/`requests` present (business logic) | PASS |
| No raw hex colors | PASS |

Visual browser inspection deferred to orchestrator post-merge per wave-3 checkpoint override.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Context recovery — previous session changes not committed**
- **Found during:** Session start
- **Issue:** Previous session (pre-compaction) had applied partial changes to `dashboard.rs` but did not commit them. The worktree showed a clean state at session start with 1672 lines (original).
- **Fix:** Full reimplementation of both tasks from scratch in this session.
- **Commit:** 92a6ee1

**2. [Rule 1 - Bug] ChildrenFn conflict: Card/Btn cannot wrap Vec-iterating sections**
- **Found during:** Task 1 implementation (architecture decision)
- **Issue:** `Card`, `Btn`, `Badge`, `Glitch` all use `ChildrenFn` requiring `Fn` children closures. Wrapping the roster/bench/coaches sections (which call `Vec::into_iter()`) inside Card's children would produce `FnOnce` closures — E0525 compile error.
- **Fix:** Only wrap static-content sections with `Card` (team info header); Vec-iterating sections use plain divs with equivalent CSS. Pandemonium sections use separate `#[component]` subcomponents so their Vec iterations are scoped inside their own view! blocks.
- **Files modified:** src/pages/team/dashboard.rs

**3. [Rule 1 - Bug] RiotTape label type: must be `&'static str` not `String`**
- **Found during:** Task 2 implementation (caught pre-compile via previous session analysis)
- **Issue:** Plan code sample showed `label="// TEAM_BRIEF · GAME_DAY".to_string()` but `RiotTape` prop is `#[prop(optional, default = "RIOT")] label: &'static str`. Passing `.to_string()` would E0308.
- **Fix:** Used `label="TEAM_BRIEF GAME_DAY"` (static str literal, no `.to_string()`; also removed `·` which is non-ASCII).
- **Files modified:** src/pages/team/dashboard.rs

**4. [Rule 1 - Architectural simplification] Dispatch at Ok(Some) arm vs TeamDashboardView component**
- **Found during:** Task 1 implementation
- **Issue:** Plan specified extracting a `TeamDashboardView` component that received Resources as props. Passing `Resource<T>` types as component props is verbose and requires type annotations. The Pandemonium variant doesn't need the Resources (it renders hardcoded data).
- **Fix:** Dispatch `if is_pandemonium { PandemoniumTeamDashboard } else { /* original */ }` directly inside the `Ok(Some(...))` match arm. This avoids prop threading entirely and is simpler.
- **Files modified:** src/pages/team/dashboard.rs

## Known Stubs

All stubs documented in "Pandemonium Data Section Status" table above. None block the plan's goal — all 7 required visual sections are present and the pixelDiffRatio assertion in 18-09 will fire correctly.

## Self-Check: PASSED

- src/pages/team/dashboard.rs: FOUND (1923 lines)
- commit 92a6ee1: FOUND
- `cargo check --features ssr`: 0 errors
- `cargo check --features hydrate --target wasm32-unknown-unknown`: 0 errors
- `cargo test --features ssr --lib`: 111/111 passed
- All 13 grep acceptance criteria: PASS
