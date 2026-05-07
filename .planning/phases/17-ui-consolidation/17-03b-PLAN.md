---
phase: 17-ui-consolidation
plan: 03b
type: execute
wave: 2
depends_on: [17-01, 17-02]
files_modified:
  - src/pages/tree_drafter.rs
  - src/components/tree_graph.rs
  - src/components/champion_autocomplete.rs
autonomous: false
decisions: [D-01, D-02, D-03, D-07, D-10, D-12]
requirements: [SC-2-claude-design-implementation, SC-4-ui-review-pass]
must_haves:
  truths:
    - "User visits /tree-drafter and sees 5 node states (locked/selected/alternate/ghost/leaf) with animated edge dash"
    - "ChampionAutocomplete dropdown matches design (input, dropdown items with thumbnails, keyboard nav preserved)"
    - "tree_graph.rs preserves layout algo + children_of HashMap DFS (CLAUDE.md rule 41)"
    - "Each visual gate in this plan passes per-page user review per D-10 before its commit"
  artifacts:
    - path: "src/components/tree_graph.rs"
      provides: "5-state node visuals with animated edge dash"
    - path: "src/pages/tree_drafter.rs"
      provides: "Tree drafter page restyle with preserved auto-save + node logic"
    - path: "src/components/champion_autocomplete.rs"
      provides: "Restyled autocomplete input + dropdown"
  key_links:
    - from: "src/pages/tree_drafter.rs"
      to: "src/components/tree_graph.rs"
      via: "TreeGraph component invocation with 5-state styling"
      pattern: "<TreeGraph|tree_graph"
    - from: "src/pages/tree_drafter.rs"
      to: "src/components/champion_autocomplete.rs"
      via: "ChampionAutocomplete component invocation"
      pattern: "<ChampionAutocomplete|champion_autocomplete"
---

<objective>
Restyle the tree-drafter feature: `/tree-drafter` page (1,610 lines) plus its two shared components `tree_graph.rs` (709 lines) and `champion_autocomplete.rs`. All page logic preserved; visual layer only.

Per D-10 each page restyle is a per-page review gate. Per D-07 components stay idiomatic Leptos while matching the prototype's pixel-level visuals. CLAUDE.md rule 41 (children_of HashMap DFS) is preserved verbatim.

This plan was split out from the original 17-03 to bound context/drift on the tree-drafter cluster.

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
@.planning/phases/17-ui-consolidation/17-RESEARCH.md
@/tmp/lol-design-handoff/lol-team-companion-app/project/tree-drafter.jsx
@.claude/rules/leptos-patterns.md
@.claude/rules/wasm-patterns.md
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Restyle src/components/tree_graph.rs (5 node states + animated edges, preserve layout algo + children_of)</name>
  <files>src/components/tree_graph.rs</files>
  <read_first>
    - src/components/tree_graph.rs (FULL FILE 709 lines — preserve LayoutNode, compute_widths, assign_positions, children_of HashMap DFS)
    - /tmp/lol-design-handoff/lol-team-companion-app/project/tree-drafter.jsx (FULL FILE)
    - .planning/phases/17-ui-consolidation/17-UI-SPEC.md (lines 290-347 — Tree Graph Interactions)
    - .planning/phases/17-ui-consolidation/17-PATTERNS.md (tree_graph.rs analog at lines 491-531)
    - CLAUDE.md rule 41 (children_of HashMap DFS)
    - .planning/phases/17-ui-consolidation/17-RESEARCH.md "Pitfall 9" (SVG stroke via style attr; CSS variables ARE re-evaluated by browser on theme switch)
  </read_first>
  <behavior>
    - Existing: tree assembly via children_of HashMap DFS (CLAUDE.md rule 41) preserved exactly.
    - Existing: layout algo (compute_widths, assign_positions) preserved.
    - New visual: 5 node states (locked, selected, alternate, ghost, leaf) class-based.
    - New visual: edge animations (selected = animated stroke-dasharray; alternate = static; ghost = dashed faint).
  </behavior>
  <action>
    Per UI-SPEC §"Tree Graph Interactions":

    1. PRESERVE: layout algo (`compute_widths`, `assign_positions`), `children_of` HashMap DFS, debounced auto-save Effect with eager signal capture (CLAUDE.md rule 41 + wasm-patterns rule 42).

    2. ADD: 5 node states. Render each node with conditional class:
       - `locked`: full opacity, `border-outline`.
       - `selected`: `ring-2 ring-accent shadow-[0_0_14px_color-mix(in_oklab,var(--color-accent)_30%,transparent)]`.
       - `alternate`: `opacity-78 border-divider`.
       - `ghost`: `opacity-30 border-dashed border-divider/50`.
       - `leaf`: circular larger 64px, `ring-accent/60 shadow-accent/20`, big outcome% text + label.

    3. Node anatomy: 144×56px, ChampTile 44px inside, phase label `text-xs font-mono text-muted`, champion name `text-sm font-semibold text-primary`, win% badge + side dot.
       - kind=ban: ChampTile with diagonal ban overlay.

    4. SVG edge rendering: cubic bezier paths.
       - Selected path: `style="stroke: var(--color-accent); stroke-width: 2"` + `stroke-dasharray="6 4"` + CSS animation `@keyframes dashFlow` defined in input.css plan 01 OR added here in a `<style>` block scoped to tree_graph.
       - Alternate path: `style="stroke: var(--color-secondary); stroke-width: 1.5; opacity: 0.6"`.
       - Ghost path: `style="stroke: var(--color-muted); stroke-width: 1; opacity: 0.25" stroke-dasharray="4 4"`.
       - Side tint: us=info (lapis blue `var(--info)` or `var(--lapis-1)`), them=danger.

    5. Per Pitfall 9: SVG attrs use `style="stroke: var(...)"` because Tailwind utilities don't reach SVG `stroke`; CSS variables ARE re-evaluated on theme switch by the browser, so theme switching works automatically.

    6. NO raw hex; G-12 focus rings on any interactive overlay buttons.
  </action>
  <verify>
    <automated>cargo check --features ssr 2>&1 | tail -3 && cargo check --features hydrate --target wasm32-unknown-unknown 2>&1 | tail -3 && grep -q "children_of" src/server/db.rs && grep -q "LayoutNode\|compute_widths" src/components/tree_graph.rs && grep -q "stroke: var" src/components/tree_graph.rs && ! grep -qE "#[0-9a-fA-F]{6}" src/components/tree_graph.rs</automated>
  </verify>
  <acceptance_criteria>
    - tree_graph.rs preserves children_of HashMap, LayoutNode, layout algo (grep confirms function names)
    - 5 node states implemented (grep for class strings or state matchers)
    - SVG strokes use `style="stroke: var(...)"` per Pitfall 9
    - Both compile targets green; no raw hex
  </acceptance_criteria>
  <done>tree_graph.rs has 5 node states + animated edges; layout algo preserved.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Restyle src/pages/tree_drafter.rs + src/components/champion_autocomplete.rs</name>
  <files>src/pages/tree_drafter.rs, src/components/champion_autocomplete.rs</files>
  <read_first>
    - src/pages/tree_drafter.rs (FULL FILE 1,610 lines)
    - src/components/champion_autocomplete.rs (FULL FILE — preserve on_select callback)
    - /tmp/lol-design-handoff/lol-team-companion-app/project/tree-drafter.jsx
    - .planning/phases/17-ui-consolidation/17-UI-SPEC.md (lines 422-447 — ChampionAutocomplete)
    - .planning/phases/17-ui-consolidation/17-PATTERNS.md (champion_autocomplete.rs at lines 569-606)
    - .claude/rules/wasm-patterns.md (rule 42 — debounced auto-save eager signal capture)
  </read_first>
  <behavior>
    - Existing: click node → selects subtree; debounced auto-save (Phase 12 pattern) preserved.
    - Existing: ChampionAutocomplete `on_select` callback preserved; keyboard nav (↑↓ Esc) preserved.
    - New visual: tree_drafter page wrapped in canvas-grain; restyled headers, action buttons.
    - New visual: ChampionAutocomplete input + dropdown match design.
  </behavior>
  <action>
    Per UI-SPEC §"Tree Graph Interactions" + §"ChampionAutocomplete":

    1. **tree_drafter.rs**:
       - PRESERVE all server fns, debounced auto-save logic, node label inline edit (contenteditable or input overlay), add-child + delete-node interactions.
       - RESTYLE page wrapper: `canvas-grain bg-base min-h-screen px-8 py-6`.
       - Page header: tree name + actions (save, list, delete) using primary/ghost button patterns.
       - "+" button on hover of non-ghost nodes: opens champion picker modal (existing pattern).
       - "×" badge on non-root nodes: shows inline confirmation ("Remove this branch and all descendants?" [Remove] [Cancel]) — NOT a modal.
       - Zoom/pan: SVG viewBox manipulation (existing).

    2. **champion_autocomplete.rs**:
       - PRESERVE signature (lines 5-10) and `on_select` callback (lines 30-37).
       - PRESERVE Effect that syncs filter_text from value (lines :47-50).
       - RESTYLE: input `bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm w-full focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none`.
       - Dropdown: `absolute z-50 bg-surface border border-divider rounded-lg shadow-lg overflow-hidden w-full max-h-60 overflow-y-auto`.
       - Each item: `flex items-center gap-2 px-3 py-2 text-sm text-secondary hover:bg-elevated hover:text-primary cursor-pointer`.
       - 24×24px ChampTile thumbnail per item.
       - Active (keyboard `↑↓`): `bg-elevated text-primary`. KEEP keyboard nav + Esc.

    3. NO raw hex; G-12 focus rings.
  </action>
  <verify>
    <automated>cargo check --features ssr 2>&1 | tail -3 && cargo check --features hydrate --target wasm32-unknown-unknown 2>&1 | tail -3 && grep -q "canvas-grain" src/pages/tree_drafter.rs && grep -q "on_select" src/components/champion_autocomplete.rs && grep -q "focus-visible:ring" src/components/champion_autocomplete.rs && ! grep -qE "#[0-9a-fA-F]{6}" src/pages/tree_drafter.rs src/components/champion_autocomplete.rs && cd e2e && npx playwright test audit-tree-drafter.spec.ts 2>&1 | tail -3</automated>
  </verify>
  <acceptance_criteria>
    - tree_drafter.rs canvas-grain wrapper; auto-save logic preserved
    - champion_autocomplete.rs `on_select` callback preserved; signature unchanged
    - audit-tree-drafter.spec.ts passes
    - Both compile targets green; no raw hex
  </acceptance_criteria>
  <done>tree_drafter.rs page restyled; champion_autocomplete.rs restyled.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 3: User reviews /tree-drafter</name>
  <what-built>
    - tree_graph.rs with 5 node states + animated edges
    - tree_drafter.rs page-level layout restyle
    - champion_autocomplete.rs restyled
  </what-built>
  <how-to-verify>
    1. Visit `/tree-drafter`, screenshot.
    2. Compare to tree-drafter.jsx.
    3. Test:
       - Create a tree, add nodes, click a node → siblings become alternate, dead branches become ghost.
       - Add child via "+" button → champion autocomplete opens.
       - Inline edit node label → debounced auto-save fires after 2s (verify via DevTools or DB).
       - Delete node → inline confirmation appears, not a modal.
       - Switch theme to Pandemonium → SVG strokes recolor automatically (CSS variable re-evaluation).
    4. Run `npx playwright test audit-tree-drafter.spec.ts`.

    APPROVAL CRITERIA: Visual matches; tree assembly correct (children_of order); auto-save works; theme switch updates SVG.
  </how-to-verify>
  <resume-signal>Type "approved" — Plan 03b complete; proceed to Plan 03c.</resume-signal>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| client (browser) → tree_drafter server fns | Persisted user content (trees) |
| ChampionAutocomplete → on_select callback | Champion ID strings flow from typeahead UI to parent state |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-17-11b | Tampering | restyle may break debounced auto-save (Phase 12 pattern) | mitigate | Task 2 explicitly preserves eager signal capture (wasm-patterns rule 42); audit-tree-drafter.spec.ts gate enforces |
| T-17-15b | Tampering | tree assembly DFS could be rebroken if children_of replaced | mitigate | Task 1 read_first explicitly cites CLAUDE.md rule 41; restyle is class-only |
</threat_model>

<verification>
- /tree-drafter restyled and individually approved per D-10
- audit-tree-drafter.spec.ts passes
- `cargo check --features ssr` and `--features hydrate` exit 0
- No raw hex in any modified file
</verification>

<success_criteria>
1. /tree-drafter matches Claude Design tree-drafter.jsx prototype.
2. tree_graph.rs preserves layout algo + children_of HashMap DFS (CLAUDE.md rule 41).
3. champion_autocomplete.rs `on_select` callback + keyboard nav preserved.
4. audit-tree-drafter.spec.ts passes — no behavioral regression.
5. Each restyle committed atomically with user approval per D-10.
</success_criteria>

<output>
After completion, create `.planning/phases/17-ui-consolidation/17-03b-SUMMARY.md`.
</output>
