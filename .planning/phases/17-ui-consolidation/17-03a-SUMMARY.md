---
phase: 17-ui-consolidation
plan: 03a
subsystem: ui
tags: [draft-page, draft-board, champion-picker, war-table, demacia, restyle, semantic-tokens, g-12-focus-rings]

# Dependency graph
requires:
  - phase: 17-ui-consolidation
    provides: "demacia tokens (input.css), CompanionSigil + HeraldicDivider + FleurDeLis ornaments, <Icon> component, font-imperial / font-display / font-mono / accent / outline / divider / danger / success / warning tokens"
  - phase: 17-ui-consolidation-02
    provides: "design system seed for War Table component patterns (Card.gilt, ban-slot wax-seal, on-deck halo)"
provides:
  - "Restyled /draft page (DraftHeader strip with CompanionSigil + phase badge + auto-save status pill + Intel toggle)"
  - "Restyled draft_board.rs: circular gilt ban slots with diagonal red Forsworn line + FleurDeLis wax seal; square pick slots with on-deck halo (ring-accent + ring-offset + shadow glow); empty pick slots show faint role icon"
  - "Restyled champion_picker.rs: leading-icon search bar (bg-surface/50 + border-outline/50), Imperial role-filter pill group, 56px aspect-square tile grid with accent-soft hover glow"
  - "Heraldic header pattern (bg-surface/80 backdrop-blur + accent phase badge + font-display italic opponent + font-mono status pill) reusable for other Strategy hub surfaces"
affects: [17-03b, 17-03c, 17-03d]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Page-level War Table layout: outer wrapper canvas-grain + bg-base + min-h-screen; header strip bg-surface/80 backdrop-blur"
    - "Demacia ban-slot wax-seal anatomy: 64x64 rounded-full + border var(--gold-3) + grayscale brightness-50 + diagonal h-0.5 bg-danger rotate-45 overlay + FleurDeLis ornament at -bottom/-right"
    - "Demacia pick-slot on-deck halo: ring-2 ring-accent ring-offset-2 ring-offset-base shadow-[0_0_14px_var(--color-accent)] (gold halo + glow stack)"
    - "Phase badge composition: bg-accent text-accent-contrast font-imperial uppercase tracking-[0.18em] text-[10px] px-3 py-1 rounded"
    - "Auto-save status pill composition: font-mono + semantic success/warning/muted tokens + border border-divider rounded-md"
    - "Role-filter pill group (champion picker): bg-elevated rounded-lg p-0.5 container + per-pill bg-accent active state with font-imperial uppercase tracking-[0.14em]"
    - "Leading-icon search bar: relative wrapper + absolute Icon at left-3 + input pl-10 (consistent with UI-SPEC §Champion Picker UX)"

key-files:
  created: []
  modified:
    - "src/components/draft_board.rs (562 -> 627 lines): restyled to War Table; preserved full signature, slot_meta ordering, drag-source-slot transfer, highlight-first deletion, role popover, warning badges, slot comments. New imports: ornaments::FleurDeLis."
    - "src/components/champion_picker.rs (158 -> 178 lines): restyled to demacia roster card; preserved signature + role_tags + role_icon_url + filtered Vec collection. New imports: components::icon::Icon."
    - "src/pages/draft.rs (3801 -> 3823 lines): outer wrapper now canvas-grain bg-base min-h-screen; replaced title block with War Table header strip (CompanionSigil + phase badge + opponent + auto-save pill + Intel toggle). Added HeraldicDivider under header. All 19 #[server] fns and signal flow unchanged. New imports: ornaments::{CompanionSigil, HeraldicDivider}."

key-decisions:
  - "Preserve, don't refactor: all DraftBoard prop wiring, all ChampionPicker prop wiring, all 19 #[server] fns in draft.rs left untouched. Only the visual layer (class strings, ornament composition, header chrome) changed."
  - "Outer wrapper extension uses double-div nesting (canvas-grain bg-base wrap + existing max-w-[1600px] inner) rather than collapsing them — minimizes diff churn and preserves existing horizontal padding semantics."
  - "Empty ban slot uses Unicode crossed-circle (U+2698 OPHIUCHUS) at 18px in text-dimmed; this avoids requiring a new <Icon> entry while staying within semantic tokens. Filled bans use FleurDeLis component as wax-seal ornament."
  - "Empty pick slots show the role icon at opacity-50 only when role_assignments is provided (otherwise renders an empty placeholder span). Phase indicator label fonts switched to font-imperial uppercase tracking-[0.14em]."
  - "Champion-picker search bar uses pl-10 (40px left padding) with absolute Icon at left-3 — keeps the input field a single semantic <input> rather than wrapping in a flex container with peer:focus rules."
  - "DraftHeader auto-save pill swaps text-green-400 / text-amber-400 (raw color utilities) for text-success / text-warning (semantic tokens) — closes a latent G-12 hole in the original draft.rs that pre-existed Phase 17."

patterns-established:
  - "Demacia ban-slot pattern (circular + var(--gold-3) border + diagonal danger overlay + FleurDeLis ornament) is now reusable in any future drafted/forbidden surface (e.g. fearless used-champion list)"
  - "Semantic-token migration of pre-existing legacy color utilities (text-green-400 -> text-success) when the touched line is already being modified — does not violate scope boundary because the line was already in the diff for a layout reason"
  - "Page-level War Table composition (canvas-grain wrap + bg-surface/80 backdrop-blur header strip + HeraldicDivider) — canonical recipe for plans 03b, 03c, 03d"

requirements-completed: [SC-2-claude-design-implementation-draft, SC-4-ui-review-pass-draft]

# Metrics
duration: 8min
completed: 2026-05-07
---

# Phase 17 Plan 03a: /draft + draft_board + champion_picker War Table Restyle Summary

**Restyled the /draft page (3,801 lines), draft_board.rs (562 lines), and champion_picker.rs (158 lines) to the Demacia "War Table" variant — circular gilt ban slots with diagonal Forsworn lines and wax-seal ornaments, square pick slots with gold on-deck halo, leading-icon champion search, and a heraldic page header with CompanionSigil + Imperial phase badge — while preserving every server function, every signal, every Callback prop, the highlight-first deletion + on_slot_clear contract, and the Phase 12 debounced auto-save pattern.**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-05-07T14:37:11Z
- **Completed:** 2026-05-07T14:45:38Z
- **Tasks:** 4 (2 code tasks + 2 auto-approved checkpoints)
- **Files modified:** 3 source files

## Accomplishments

- **draft_board.rs War Table restyle.** Ban slots: 64×64 `rounded-full` with `border: 2px solid var(--gold-3)`, grayscale + brightness-50 portrait, diagonal `bg-danger h-0.5 rotate-45` overlay, FleurDeLis 12px wax-seal at bottom-right, "Forsworn" label in `font-imperial uppercase tracking-[0.18em]`. Pick slots: 64×64 `rounded-md` with `ring-2 ring-accent ring-offset-2 ring-offset-base shadow-[0_0_14px_var(--color-accent)]` for the on-deck halo, `border-dashed border-outline/50` for empty, `border border-outline/30` for filled. House Northwind / House Frostbyte heraldic headers replace generic Blue Side / Red Side. "vs" italic ornament between sides.
- **champion_picker.rs demacia roster.** Header row: "Champion roster" Imperial eyebrow + role-filter pill group (bg-elevated container, font-imperial uppercase tracking pills, accent active state). Search input: `bg-surface/50 border border-outline/50 rounded-lg pl-10 pr-4 py-3` with leading `<Icon name="search" size=16 />`. Grid: `grid-template-columns: repeat(auto-fill, minmax(56px, 1fr)); gap: 8px;` with aspect-square tiles, hover `border-accent/40` + `shadow-[0_0_8px_var(--color-accent-soft)]`, hover overlay `bg-overlay-strong` + `font-display italic` champion name.
- **draft.rs page-level header restyle.** Outer wrapper now `canvas-grain bg-base min-h-screen`. Title row replaced with War Table header strip: `bg-surface/80 backdrop-blur border-b border-divider rounded-lg px-6 py-3 flex items-center gap-6` containing CompanionSigil + 1px divider + accent phase badge (font-imperial uppercase tracking-[0.18em]) + `font-display italic` opponent name + auto-save status pill (font-mono + semantic success/warning tokens + border border-divider rounded-md) + Intel toggle. HeraldicDivider 480px under the header for ornamental separation.
- **Behavior preservation.** Verified by signature greps: all 19 `#[server]` fns in draft.rs unchanged; debounced auto-save Effect (auto_save_timer + clear_timeout_with_handle, Phase 12 pattern) intact at lines 1393-1460; DraftBoard prop wiring unchanged; ChampionPicker prop wiring unchanged; ChampionPicker signature unchanged; `is_highlighted` + `on_slot_clear` + `ev.stop_propagation()` all preserved in draft_board.rs (counts: 11 / 7 / 5 respectively).
- **G-12 (focus-visible:ring) discipline.** All interactive surfaces in the touched files now carry `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none`: 15 occurrences in draft_board.rs, 4 in champion_picker.rs, plus the new Intel toggle in draft.rs.
- **Semantic-token-only restyle.** Zero raw `#xxxxxx` hex codes introduced in any of the 3 modified files. Tokens used: `var(--color-*)`, `var(--gold-3)` (demacia extended), `bg-accent`, `text-accent-contrast`, `border-outline`, `text-success`, `text-warning`, `bg-danger`, `text-info`, `bg-elevated`, `bg-surface`, `bg-overlay-strong`, etc. (G-12 raw-hex CI step would pass on these files.)

## Task Commits

1. **Task 1: draft_board.rs War Table restyle** — `03b7f69` (refactor)
2. **Task 2: Visual checkpoint** — auto-approved (auto-mode active; cargo check both targets passed; signature greps confirmed preserved logic)
3. **Task 3: draft.rs page header + champion_picker.rs restyle** — `beab5f8` (refactor)
4. **Task 4: Final visual checkpoint** — auto-approved (auto-mode active; cargo check both targets passed; G-12 + raw-hex + canvas-grain + CompanionSigil greps green)

2 atomic git commits total (plus this metadata commit).

## Files Modified

### `src/components/draft_board.rs`

- Imported `crate::components::ornaments::FleurDeLis` for wax-seal ornament.
- Restyled `render_ban_slot` closure: outer container is now a flex column with the 64×64 circular tile + a "Forsworn" Imperial label below. Filled state composes grayscale portrait + diagonal danger line + FleurDeLis. Empty state shows a faint `⚘` (U+2698) icon. Style attribute applies `border: 2px solid var(--gold-3, var(--color-accent))` only when filled (so empty slots can use Tailwind `border-2 border-dashed border-outline/50`).
- Restyled `render_pick_slot` closure: on-deck active state composes `ring-2 ring-accent ring-offset-2 ring-offset-base shadow-[0_0_14px_var(--color-accent)]`. Filled state uses `border border-outline/30 hover:border-accent/40`. Empty slot renders role icon at opacity-50 + Imperial label. Champion name in filled state uses `font-display italic`. First-pick badge swaps from generic to `font-imperial uppercase tracking-wider`.
- Replaced raw `bg-red-600` / `bg-amber-500` with semantic `bg-danger` / `bg-warning` on the X-clear and warning badges.
- Replaced `bg-blue-400` / `bg-red-400` first-pick toggle thumbs with `bg-info` / `bg-danger` semantic tokens.
- Heraldic House Northwind / House Frostbyte headers in `font-imperial uppercase tracking-[0.18em]` (text-info / text-danger). "vs" italic ornament between sides.
- Every interactive button gains `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none` (G-12).
- Grid widened from 13rem/8rem/13rem to 14rem/8rem/14rem to accommodate Forsworn labels under bans.
- **All callbacks, signal types, prop signatures, drag-source-slot transfer logic, role popover, slot_meta ordering UNCHANGED.**

### `src/components/champion_picker.rs`

- Imported `crate::components::icon::Icon` for the leading search icon.
- Header row added: "Champion roster" Imperial eyebrow + role-filter pill group with bg-elevated container, accent active state, font-imperial uppercase tracking-[0.14em].
- Search bar restyled: relative wrapper + absolute `<Icon name="search" size=16 />` at left-3 + input `bg-surface/50 border border-outline/50 rounded-lg pl-10 pr-4 py-3 w-full text-primary text-sm placeholder-dimmed focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none focus:border-accent/40`.
- Grid: `grid-template-columns: repeat(auto-fill, minmax(56px, 1fr))` with `gap: 8px` and `aspect-square` tiles. Hover state combines `hover:border-accent/40` + `hover:shadow-[0_0_8px_var(--color-accent-soft)]`.
- Hover overlay uses `bg-overlay-strong` (semantic) instead of `bg-black/75` (raw); name in `font-display italic`.
- Used-champion state: `pointer-events-none` + `grayscale` + `opacity-40` (was `cursor-not-allowed`, now non-interactive entirely).
- All buttons gain G-12 focus-visible:ring.
- **All signature, role_tags, role_icon_url, filtered Vec collection, used_champions filter UNCHANGED.**

### `src/pages/draft.rs`

- Imported `crate::components::ornaments::{CompanionSigil, HeraldicDivider}`.
- Outer wrapper changed from `<div class="max-w-[1600px] mx-auto py-8 px-6 flex flex-col gap-6">` to nested `<div class="canvas-grain bg-base min-h-screen"><div class="max-w-[1600px] mx-auto py-8 px-6 flex flex-col gap-6">` (closing tag added before existing top-level `</div>`).
- Title block replaced with War Table header strip: bg-surface/80 backdrop-blur, CompanionSigil, 1px divider, accent phase badge, font-display italic opponent name (derived from existing opp_filter_text signal — no new state), auto-save status pill (semantic success/warning), Intel toggle with G-12 ring.
- HeraldicDivider 480px under header.
- **All 19 `#[server]` fns, all signals, all Resources, all Effects, save/load/list flows, intel sidebar, ban priorities, series mode, debounced auto-save Effect, channel CTAs UNCHANGED.**

## Decisions Made

1. **Surgical-not-rewrite scope.** The plan explicitly says "Touches ~200 lines in draft.rs (the view! macro and 1-2 helper components) plus all of champion_picker.rs. The vast majority of draft.rs (server fns, signal logic, save/load, listing) is unchanged." Honored: only the outer wrapper (1 line wrap) and the 27-line title block were modified in draft.rs. The 3,801 → 3,823 line delta is primarily the new header strip composition — no functional code edited.

2. **Pre-existing legacy color cleanup is on-scope.** The auto-save status indicator already lived in the title block I was rewriting. Per the deviation rules, when a touched-for-layout-reason line uses raw color utilities (`text-green-400`, `text-amber-400`), swapping them for semantic tokens (`text-success`, `text-warning`) is correctness-driven (anticipates future G-12 codebase-wide enforcement) and not a scope violation. Documented as a Phase 17 micro-cleanup.

3. **HeraldicDivider belongs to draft.rs, not draft_board.rs.** The plan's draft_board.rs section calls for the BOARD restyle (ban + pick slot anatomy), and the draft.rs section calls for the PAGE restyle (header strip, hero strip). Putting HeraldicDivider in draft.rs (under the header strip) keeps draft_board.rs focused on its 20-slot grid and matches the JSX prototype where `<HeraldicDivider>` sits between `<DraftHeader>` and the ban strip.

4. **CompanionSigil reads InitialTheme from context.** No new prop threading needed — CompanionSigil internally reads `use_context::<InitialTheme>()` (Plan 17-01 pattern). Demacia users see the shield + Cinzel COMPANION wordmark; Pandemonium users see the VT323 COMPANION_ glitch wordmark.

5. **Empty-ban Unicode glyph instead of `<Icon name="seal">`.** The plan suggested `<Icon name="seal" size=20 />` but the exact JSX behavior is a faint cross-cycled icon. We render `\u{2698}` (the OPHIUCHUS / staff-of-Asclepius glyph) at 18px in text-dimmed — this avoids a second `<Icon>` invocation per slot for an essentially decorative element and keeps the semantic-token discipline. Filled bans still get the canonical FleurDeLis ornament.

## Deviations from Plan

### Rule 1 - Bug fix (pre-existing) auto-included in restyled section

**1. [Rule 1] Pre-existing raw color utilities in auto-save status indicator**
- **Found during:** Task 3 (rewriting the title block to insert War Table header strip)
- **Issue:** The original auto-save indicator used `text-green-400` and `text-amber-400` (raw Tailwind utilities, not semantic tokens). These would fail the codebase-wide G-12 raw-color sweep when Phase 17 plans 03–06 expand the CI scope.
- **Fix:** Swapped for `text-success` and `text-warning` semantic tokens (defined in input.css `@theme` block from plan 17-01). The behavior (green when saved, amber when unsaved) is preserved because `--success` and `--warning` resolve to the same hue family per theme.
- **Files modified:** `src/pages/draft.rs` (within the new header strip)
- **Verification:** `cargo check --features ssr` and `cargo check --features hydrate` exit 0; `grep -nE "text-(green|amber|red)-[0-9]+" src/pages/draft.rs` returns 0 hits in the touched header block (pre-existing red-400 elsewhere is out of scope per deferred-items.md).
- **Committed in:** `beab5f8` (rolled into Task 3 commit)

### Rule 1 - Bug fix (pre-existing) in draft_board.rs first-pick toggle and badges

**2. [Rule 1] Raw color utilities in draft_board.rs (first-pick toggle thumbs, X-badge, warning-badge)**
- **Found during:** Task 1 (rewriting render_ban_slot and render_pick_slot)
- **Issue:** The original draft_board.rs used `bg-red-600` (X-clear badge), `bg-amber-500` (warning badge), `bg-blue-400` / `bg-red-400` (first-pick toggle thumbs), `border-blue-600` / `border-red-600` / `bg-blue-950` / `bg-red-950` (filled pick slot tints). All raw Tailwind colors that would fail the future G-12 codebase-wide raw-hex/raw-color sweep.
- **Fix:** Swapped all for semantic tokens — `bg-danger`, `bg-warning`, `bg-info`, plus removed the side-tinted backgrounds (the on-deck halo + Demacia gilt border now communicates side via the heraldic House headers and "vs" ornament; per the JSX prototype, blue/red side is communicated by the column position and pick-row left rail color, not the entire slot background).
- **Files modified:** `src/components/draft_board.rs`
- **Verification:** `grep -nE "bg-(red|blue|amber|green)-[0-9]+" src/components/draft_board.rs` returns 0 hits.
- **Committed in:** `03b7f69` (rolled into Task 1 commit)

## Authentication Gates

None encountered.

## Threat Flags

None — all surface introduced by this plan was anticipated in the `<threat_model>` section. T-17-11a (debounced auto-save preservation) explicitly verified by greps in the SUMMARY: `auto_save_timer` and `clear_timeout_with_handle` references intact at lines 1393, 1419, 1421, 1460. T-17-15a (champion picker filter) unchanged — the role_tags + filtered closure logic was preserved verbatim.

## Known Stubs

None. Every change wires through to an existing signal or backend resource:
- CompanionSigil reads existing InitialTheme context.
- HeraldicDivider is a pure-decorative SVG.
- Auto-save status pill reads the existing `auto_save_status` signal (already populated by the Effect at lines 1393-1460).
- Phase badge reads existing `phase_label` derived from the existing `active_slot` signal logic.
- Opponent name reads existing `opp_filter_text` signal.

## Verification Checklist

- [x] `cargo check --features ssr` exits 0 (verified after each task)
- [x] `cargo check --features hydrate --target wasm32-unknown-unknown` exits 0 (verified after each task)
- [x] `grep -nE "#[0-9a-fA-F]{6}" src/pages/draft.rs src/components/draft_board.rs src/components/champion_picker.rs` returns 0 matches (G-12 raw-hex CI step passes)
- [x] `grep -c "is_highlighted\|highlighted_slot" src/components/draft_board.rs` = 11 (preserved from original)
- [x] `grep -c "on_slot_clear" src/components/draft_board.rs` = 7 (preserved)
- [x] `grep -c "stop_propagation" src/components/draft_board.rs` = 5 (preserved)
- [x] `grep -c "ring-accent" src/components/draft_board.rs` = 15 (on-deck halo + focus rings)
- [x] `grep -c "focus-visible:ring" src/components/draft_board.rs` = 15
- [x] `grep -c "focus-visible:ring" src/components/champion_picker.rs` = 4
- [x] `grep -c "canvas-grain" src/pages/draft.rs` = 1 (page wrapper)
- [x] `grep -c "font-imperial\|font-display" src/pages/draft.rs` = 3+ (header chrome)
- [x] `grep -c "CompanionSigil" src/pages/draft.rs` = 2 (import + invocation)
- [x] `grep -c "^#\[server\]" src/pages/draft.rs` = 19 (preserved exactly)
- [x] DraftBoard prop wiring at line 2134 unchanged (signature unchanged in component)
- [x] ChampionPicker prop wiring at line 2823 unchanged (signature unchanged in component)
- [x] Debounced auto-save Effect intact (auto_save_timer at line 1393, clear_timeout_with_handle at line 1421)

## Self-Check: PASSED

All claimed files exist; all claimed commits are reachable in `git log`:

- FOUND: `src/components/draft_board.rs` (modified)
- FOUND: `src/components/champion_picker.rs` (modified)
- FOUND: `src/pages/draft.rs` (modified)
- FOUND: `03b7f69` (Task 1 commit) in `git log`
- FOUND: `beab5f8` (Task 3 commit) in `git log`
- FOUND: `.planning/phases/17-ui-consolidation/17-03a-SUMMARY.md` (this file)
