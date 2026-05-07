---
phase: 17-ui-consolidation
plan: 03a
type: execute
wave: 2
depends_on: [17-01, 17-02]
files_modified:
  - src/pages/draft.rs
  - src/components/draft_board.rs
  - src/components/champion_picker.rs
autonomous: false
decisions: [D-01, D-02, D-03, D-07, D-10, D-12]
requirements: [SC-2-claude-design-implementation, SC-4-ui-review-pass]
must_haves:
  truths:
    - "User visits /draft and sees the War Table (Demacia) variant: heraldic header, ornamented ban strip with wax seals, gilt pick slots with on-deck halo, full-width champion picker"
    - "draft_board.rs preserves highlight-first deletion + on_slot_clear; circular bans, square gilt picks, on-deck halo"
    - "champion_picker.rs grid + role filter + search restyled to design"
    - "Each visual gate in this plan passes per-page user review per D-10 before its commit"
  artifacts:
    - path: "src/pages/draft.rs"
      provides: "War Table draft page restyle with preserved server fns"
    - path: "src/components/draft_board.rs"
      provides: "Restyled 20-slot board with circular bans, square picks, on-deck halo"
    - path: "src/components/champion_picker.rs"
      provides: "Restyled champion picker grid with role filter pills"
  key_links:
    - from: "src/pages/draft.rs"
      to: "src/components/draft_board.rs"
      via: "DraftBoard component invocation"
      pattern: "<DraftBoard"
    - from: "src/pages/draft.rs"
      to: "src/components/champion_picker.rs"
      via: "ChampionPicker component invocation"
      pattern: "<ChampionPicker|champion_picker"
---

<objective>
Restyle the heaviest single artifact in the Strategy hub: `/draft` (3,801 lines) plus its two shared components `draft_board.rs` and `champion_picker.rs`. All page logic preserved; visual layer only.

Per D-10 each page restyle is a per-page review gate: implement → screenshot → user approves → commit. Per D-07 components stay idiomatic Leptos (server fns, RwSignal, Resource, semantic tokens) while matching the prototype's pixel-level visuals.

This plan was split out from the original 17-03 to bound context/drift on the largest single page in the Strategy hub.

Output: 1 restyled page + 2 restyled shared components + per-page atomic commits.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/17-ui-consolidation/17-CONTEXT.md
@.planning/phases/17-ui-consolidation/17-UI-SPEC.md
@.planning/phases/17-ui-consolidation/17-PATTERNS.md
@.planning/phases/17-ui-consolidation/17-01-SUMMARY.md
@.planning/phases/17-ui-consolidation/17-02-SUMMARY.md
@/tmp/lol-design-handoff/lol-team-companion-app/project/draft-boards.jsx
@.claude/rules/leptos-patterns.md
@.claude/rules/wasm-patterns.md
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Restyle src/components/draft_board.rs (preserve highlight-first deletion + on_slot_clear)</name>
  <files>src/components/draft_board.rs</files>
  <read_first>
    - src/components/draft_board.rs (FULL FILE 562 lines)
    - /tmp/lol-design-handoff/lol-team-companion-app/project/draft-boards.jsx (FULL FILE — War Table Demacia variant primary, plus 2 other variants for fallback ideas)
    - .planning/phases/17-ui-consolidation/17-UI-SPEC.md (lines 219-287 — Draft Board Layout)
    - .planning/phases/17-ui-consolidation/17-PATTERNS.md (draft_board.rs analog block lines 460-489)
    - .claude/rules/wasm-patterns.md (rule 35 no .unwrap)
    - src/components/ornaments.rs (FleurDeLis for wax seals, GiltCorner if used)
  </read_first>
  <behavior>
    - Test 1 (preserved): clicking a filled slot once highlights it (ring); clicking the X badge clears it.
    - Test 2 (preserved): clicking a banned slot does NOT trigger picking.
    - Test 3 (preserved): on-deck slot has visual emphasis distinguishable from non-active slots.
    - Test 4 (new): visual — circular ban slots with diagonal red overlay; square gilt pick slots with on-deck halo.
  </behavior>
  <action>
    Per UI-SPEC §"Draft Board Layout":

    1. KEEP signature exactly (lines 36-50). All callbacks, optional props, signal types preserved.
    2. KEEP highlight-first deletion logic (lines 60-69) and `ev.stop_propagation()` on X badge.

    3. RESTYLE ban slots (3 per side):
       - Container: 64×64px circular tile.
       - `border-2` using `border-[var(--gold-3)]` (Demacia gilt) — fallback: define `border-gilt-deep` Tailwind class via input.css `@theme` if needed, OR use `style="border: 2px solid var(--gold-3)"`.
       - Banned state overlay: grayscale + brightness-50 + diagonal red line via absolutely positioned `<div class="absolute inset-0 bg-danger h-px top-1/2 rotate-45"></div>`.
       - Empty ban slot: `bg-elevated border-2 border-divider/50 rounded-full`.
       - Label below: `text-xs text-dimmed uppercase tracking-wider font-imperial` — "Forsworn" (Demacia) or "BAN" fallback.
       - Optional FleurDeLis wax seal ornament (Demacia): `<FleurDeLis />` overlaid bottom-right (small, 12px).

    4. RESTYLE pick slots (5 per side):
       - 64x64 ChampTile inside `grid grid-cols-[64px_20px_1fr] gap-3 items-center`.
       - On-deck (current turn): `ring-2 ring-accent ring-offset-2 ring-offset-base shadow-[0_0_14px_var(--color-accent)]`.
       - Empty slot: `bg-elevated border border-dashed border-outline/50 rounded-md` with role icon at 50% opacity.
       - Locked/filled: full opacity, `border border-outline/30`.
       - Player name: `text-sm font-semibold text-primary font-display italic`.
       - Role icon: 20×20 SVG mask (use existing role tag pattern from `champion_picker.rs:4-13`).

    5. KEEP slot click behavior:
       - First click on filled slot: highlights (existing `is_highlighted` state).
       - X badge click on highlighted: calls `on_slot_clear`, with `ev.stop_propagation()`.
       - Click on empty slot: opens champion picker via existing `on_slot_click` callback.

    6. G-12: every interactive button has `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none`.

    7. NO raw hex outside theme tokens. Use semantic tokens or `style="...: var(--color-*)"` for SVG attrs.
  </action>
  <verify>
    <automated>cargo check --features ssr 2>&1 | tail -3 && cargo check --features hydrate --target wasm32-unknown-unknown 2>&1 | tail -3 && grep -q "is_highlighted\|highlighted_slot" src/components/draft_board.rs && grep -q "on_slot_clear" src/components/draft_board.rs && grep -q "stop_propagation" src/components/draft_board.rs && grep -q "ring-2 ring-accent\|ring-accent" src/components/draft_board.rs && ! grep -qE "#[0-9a-fA-F]{6}" src/components/draft_board.rs && grep -q "focus-visible:ring" src/components/draft_board.rs && cd e2e && npx playwright test audit-draft.spec.ts 2>&1 | tail -3</automated>
  </verify>
  <acceptance_criteria>
    - draft_board.rs signature unchanged (highlighted_slot, on_slot_clear preserved)
    - `ev.stop_propagation()` still present
    - Both compile targets green
    - audit-draft.spec.ts passes (no behavioral regression)
    - No raw hex (semantic tokens only)
    - G-12 focus-visible:ring on every interactive button
  </acceptance_criteria>
  <done>draft_board.rs restyled to War Table variant; logic preserved; both targets green; audit test passes.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 2: User reviews /draft (and draft_board component)</name>
  <what-built>
    - src/components/draft_board.rs restyled to War Table (Demacia) variant
    - Circular gilt ban slots with diagonal red overlay
    - Square pick slots with on-deck halo (ring-accent + shadow)
    - Existing highlight-first deletion + on_slot_clear preserved
  </what-built>
  <how-to-verify>
    1. Confirm `cargo leptos watch` is running on :3020.
    2. Visit `http://127.0.0.1:3020/draft` (logged in).
    3. Run `npx agent-browser screenshot http://127.0.0.1:3020/draft`.
    4. Compare screenshot side-by-side with `/tmp/lol-design-handoff/lol-team-companion-app/project/draft-boards.jsx` War Table variant rendering.
    5. Test interactions:
       - Click an empty pick slot → champion picker opens.
       - Pick a champion → slot fills.
       - Click filled slot once → ring highlight appears.
       - Click X badge → slot clears.
       - Verify on-deck slot has the gold halo + glow.
       - Verify ban slots are circular with diagonal red overlay when filled.
    6. Verify accessibility: tab through slots; focus rings appear (G-12).
    7. Verify no raw hex appears in DevTools (only semantic CSS variables).

    APPROVAL CRITERIA: Visual matches War Table prototype within reasonable tolerance; all interactions still work; no console errors.
  </how-to-verify>
  <resume-signal>Type "approved" to proceed to Task 3 (restyle draft.rs page-level layout), or describe issues for revision.</resume-signal>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Restyle src/pages/draft.rs (preserve all server fns; apply War Table page layout) + restyle src/components/champion_picker.rs</name>
  <files>src/pages/draft.rs, src/components/champion_picker.rs</files>
  <read_first>
    - src/pages/draft.rs (FULL FILE 3,801 lines — read top 200 lines for structure, then grep for key sections: server fns, the main view! macro, ChampionPicker invocation, save/load handlers)
    - src/components/champion_picker.rs (FULL FILE)
    - /tmp/lol-design-handoff/lol-team-companion-app/project/draft-boards.jsx (page-level layout: DraftHeader, hero strip, ban strips, pick zones, full-width picker)
    - .planning/phases/17-ui-consolidation/17-UI-SPEC.md (lines 219-287)
    - .planning/phases/17-ui-consolidation/17-PATTERNS.md (champion_picker.rs analog at lines 534-568)
    - .claude/rules/leptos-patterns.md (debounced auto-save pattern, ActionForm no class prop)
  </read_first>
  <behavior>
    - All existing draft server fns work unchanged (save/load/list/delete drafts).
    - Side toggle (Blue/Red) preserved.
    - Champion picker filter + role filter preserved.
    - Auto-save debounce preserved (Phase 12 pattern, MEMORY.md).
  </behavior>
  <action>
    Per UI-SPEC §"Draft Board Layout":

    1. PRESERVE all `#[server]` fns and signal flow in draft.rs.

    2. RESTYLE the page-level view! macro:
       - Outer wrapper: `<div class="canvas-grain bg-base min-h-screen">` (Demacia painterly background).
       - DraftHeader strip (~56px tall): `bg-surface/80 backdrop-blur border-b border-divider px-8 py-3 flex items-center gap-6`. Contents: CompanionSigil, phase badge (`bg-accent text-accent-contrast font-imperial uppercase tracking-[0.18em] text-[10px] px-3 py-1 rounded`), opponent name (`font-display italic text-primary`), timer (`font-mono border border-outline rounded-md px-2 py-1 text-secondary`), "Save draft" outline btn, "To plan" primary btn.
       - Hero strip (under header): `flex items-center justify-center gap-6 py-4` with our team name, vs ornament (HeraldicDivider rotated or vertical), opponent name.
       - Ban strips: 3 ban slots per side, side-by-side.
       - Pick zones: 5 pick slots per side, side-by-side using DraftBoard component.
       - Champion picker: full-width below the board.

    3. RESTYLE `src/components/champion_picker.rs`:
       - KEEP signature (champions, used_champions, on_select, role filter at lines 26-31).
       - Search bar: `bg-surface/50 border border-outline/50 rounded-lg px-4 py-3 w-full text-primary text-sm` + search icon left (`<Icon name="search" size=16 class="text-muted" />`).
       - Grid: `grid grid-cols-[repeat(auto-fill,minmax(56px,1fr))] gap-2 p-4 overflow-y-auto max-h-80`.
       - Tile: 56×56px `rounded-md hover:border-accent/40 cursor-pointer transition-all border border-transparent`.
       - Selected: `ring-2 ring-accent border-accent`.
       - Banned/unavailable: `grayscale opacity-40 pointer-events-none`.
       - Champion name: `text-xs text-center text-muted mt-1` below tile.
       - Role filter pills: `bg-elevated rounded-lg p-0.5 flex gap-0.5` per UI-SPEC.

    4. KEEP debounced auto-save pattern (per Phase 12 MEMORY.md and leptos-patterns rule 54): `RwSignal<Option<i32>>` timer handle + `Effect::new` cancels via `clear_timeout_with_handle()` + new 2s `Closure::once`. Eager signal capture outside the closure.

    5. NO `outline:none` without ring. Every interactive element has `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none`.

    6. NO raw hex; only semantic tokens.

    Touches ~200 lines in draft.rs (the view! macro and 1-2 helper components) plus all of champion_picker.rs. The vast majority of draft.rs (server fns, signal logic, save/load, listing) is unchanged.
  </action>
  <verify>
    <automated>cargo check --features ssr 2>&1 | tail -3 && cargo check --features hydrate --target wasm32-unknown-unknown 2>&1 | tail -3 && grep -q "canvas-grain" src/pages/draft.rs && grep -q "font-imperial\|font-display" src/pages/draft.rs && grep -q "CompanionSigil\|companion_sigil" src/pages/draft.rs && ! grep -qE "#[0-9a-fA-F]{6}" src/pages/draft.rs src/components/champion_picker.rs && cd e2e && npx playwright test audit-draft.spec.ts 2>&1 | tail -3</automated>
  </verify>
  <acceptance_criteria>
    - All server fns in draft.rs unchanged (grep counts before/after match)
    - Page wraps in canvas-grain bg-base
    - Champion picker uses semantic tokens, focus-visible:ring
    - Both compile targets green
    - audit-draft.spec.ts passes
  </acceptance_criteria>
  <done>draft.rs page-level layout matches War Table; champion_picker.rs restyled; logic preserved.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 4: User reviews /draft full page + champion picker</name>
  <what-built>
    - /draft page-level War Table layout with DraftHeader, hero strip, ban strips, pick zones, full-width picker
    - champion_picker.rs restyled with new search bar, grid, tile sizing
  </what-built>
  <how-to-verify>
    1. Visit `/draft` and screenshot via agent-browser.
    2. Compare to draft-boards.jsx War Table variant.
    3. Test full draft flow: pick champions, ban champions, save draft, load saved draft.
    4. Test champion picker: search, role filter pills, tile click.
    5. Verify on-deck halo correctly tracks the active phase.
    6. Run `cd e2e && npx playwright test audit-draft.spec.ts` — must pass.

    APPROVAL CRITERIA: Visual matches; full draft flow works; e2e passes.
  </how-to-verify>
  <resume-signal>Type "approved" — Plan 03a complete; proceed to Plan 03b (tree-drafter).</resume-signal>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| client (browser) → draft server fns | Persisted user content (drafts) |
| ChampionPicker → on_select callback | Champion ID strings flow from picker UI to parent state |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-17-11a | Tampering | restyle may break debounced auto-save (Phase 12 pattern) | mitigate | Task 3 explicitly preserves eager signal capture (wasm-patterns rule 42); audit-draft.spec.ts gate enforces |
| T-17-15a | Spoofing | champion picker filter could match unauthorized champion IDs | accept | Champion list comes from backend Resource (server-validated); user input only filters client-side display |
</threat_model>

<verification>
- /draft restyled and individually approved per D-10
- audit-draft.spec.ts passes
- `cargo check --features ssr` and `--features hydrate` exit 0
- `grep -E "#[0-9a-fA-F]{6}" src/pages/draft.rs src/components/draft_board.rs src/components/champion_picker.rs` returns 0 matches
</verification>

<success_criteria>
1. /draft matches Claude Design War Table hero variant.
2. draft_board.rs preserves highlight-first deletion + on_slot_clear.
3. champion_picker.rs grid + search + role filters restyled with semantic tokens.
4. audit-draft.spec.ts passes — no behavioral regression.
5. Each restyle committed atomically with user approval per D-10.
</success_criteria>

<output>
After completion, create `.planning/phases/17-ui-consolidation/17-03a-SUMMARY.md`.
</output>
