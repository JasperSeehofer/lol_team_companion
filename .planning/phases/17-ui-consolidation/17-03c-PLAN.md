---
phase: 17-ui-consolidation
plan: 03c
type: execute
wave: 2
depends_on: [17-01, 17-02]
files_modified:
  - src/pages/champion_pool.rs
  - src/pages/game_plan.rs
  - src/pages/post_game.rs
autonomous: false
decisions: [D-01, D-02, D-03, D-07, D-10, D-12]
requirements: [SC-2-claude-design-implementation, SC-4-ui-review-pass]
must_haves:
  truths:
    - "User visits /champion-pool and sees tier list (S/A/B/C) with tier-color left-border, font-imperial tier letter, deep-dive panel sticky sidebar"
    - "User visits /game-plan and sees the design-spec layout with restyled inputs"
    - "User visits /post-game and sees the design-spec layout (recursion_limit=512 unchanged)"
    - "Each visual gate in this plan passes per-page user review per D-10 before its commit"
  artifacts:
    - path: "src/pages/champion_pool.rs"
      provides: "Tier list with tier-color borders + deep dive sidebar"
    - path: "src/pages/game_plan.rs"
      provides: "Game plan page restyled (visual layer only — UI-SPEC line 510)"
    - path: "src/pages/post_game.rs"
      provides: "Post-game page restyled; nested view types preserved (recursion_limit=512)"
  key_links:
    - from: "src/pages/champion_pool.rs"
      to: "src/components/champion_autocomplete.rs"
      via: "ChampionAutocomplete invocation with on_select callback"
      pattern: "ChampionAutocomplete|on_select"
---

<objective>
Restyle the three remaining heavy hero pages in the Strategy hub: champion-pool (1,356 lines), game-plan (1,515 lines), and post-game (1,109 lines). All page logic preserved; visual layer only.

Per D-10 each page restyle is a per-page review gate. CLAUDE.md `recursion_limit = "512"` is preserved verbatim for post-game.

This plan was split out from the original 17-03 to bound context/drift on this 3-page cluster.

Output: 3 restyled pages + per-page atomic commits.
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
@/tmp/lol-design-handoff/lol-team-companion-app/project/champion-pool.jsx
@/tmp/lol-design-handoff/lol-team-companion-app/project/extra-variants.jsx
@/tmp/lol-design-handoff/lol-team-companion-app/project/pandemonium-variants.jsx
@.claude/rules/leptos-patterns.md
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Restyle src/pages/champion_pool.rs (tier list with deep-dive sidebar)</name>
  <files>src/pages/champion_pool.rs</files>
  <read_first>
    - src/pages/champion_pool.rs (FULL FILE 1,356 lines — preserve drag-rank logic, tier mutation server fns)
    - /tmp/lol-design-handoff/lol-team-companion-app/project/champion-pool.jsx (FULL FILE)
    - .planning/phases/17-ui-consolidation/17-UI-SPEC.md (lines 450-481 — Champion Pool Page Layout)
    - .planning/phases/17-ui-consolidation/17-PATTERNS.md (champion_pool.rs noted as self-pattern at line 60)
  </read_first>
  <behavior>
    - Existing: drag champions between tiers (S/A/B/C) — order persists.
    - Existing: click-to-add via ChampionAutocomplete `on_select` (Phase 12 pattern).
    - New visual: tier band layout with tier-color left-border, font-imperial tier letter (56px), gradient background.
    - New visual: deep-dive panel (sticky 380px sidebar).
    - New visual: gap analysis strip with role have/need bars.
  </behavior>
  <action>
    Per UI-SPEC §"Champion Pool Page Layout":

    1. PRESERVE all `#[server]` fns, drag-rank logic (Phase 12 pattern), `on_select` callback wiring.

    2. RESTYLE page layout:
       - Outer: `canvas-grain bg-base min-h-screen px-8 py-6`.
       - Page header: imperial eyebrow (`font-imperial uppercase tracking-[0.18em] text-[10px] text-muted`) + display headline (`font-display italic text-4xl text-primary`) — "Your champions, by station." + actions on right ("Add champion" outline + "Save pool" primary).
       - Two-column grid: `grid grid-cols-[1fr_380px] gap-6`.

    3. Tier bands (left column):
       - Each tier (S/A/B/C) has its own band:
         - Tier label column: `width: 100px` with `border-r-2` using tier color (`border-r-[var(--gold-2)]` for S, `border-r-[var(--color-accent)]` for A, `border-r-[var(--warning)]` for B, `border-r-[var(--danger)]` for C). Letter in `font-imperial text-[56px] font-bold` with subtitle ("Sword arm" / "Standing" / "Reserve" / "Forsaken").
         - Background gradient: `linear-gradient(135deg, color-mix(in oklab, {tier-color} 30%, var(--color-surface)), var(--color-surface))`.
         - Tier content: `p-4` with champion tiles `flex flex-wrap gap-3`. Each tile: 64×64 ChampTile.
       - Drag interaction visual: dragged tile `scale-105 shadow-lg`.

    4. Deep-dive panel (right column):
       - `bg-elevated border border-divider rounded-xl p-6 self-start sticky top-24`.
       - Champion constellation visualization (re-use existing data shape).
       - Stat breakdown.

    5. Gap analysis strip (above tier list):
       - `grid grid-cols-5 gap-3` showing each role with have/need bar.
       - Role status warn = `text-warning`, danger = `text-danger`.

    6. G-12 + no raw hex everywhere. Tier colors defined in input.css `@theme` are accessed via Tailwind utilities or `var(...)` references.
  </action>
  <verify>
    <automated>cargo check --features ssr 2>&1 | tail -3 && cargo check --features hydrate --target wasm32-unknown-unknown 2>&1 | tail -3 && grep -q "canvas-grain" src/pages/champion_pool.rs && grep -q "font-imperial\|font-display" src/pages/champion_pool.rs && grep -q "sticky top-24\|self-start sticky" src/pages/champion_pool.rs && ! grep -qE "#[0-9a-fA-F]{6}" src/pages/champion_pool.rs && cd e2e && npx playwright test audit-champion-pool.spec.ts 2>&1 | tail -3</automated>
  </verify>
  <acceptance_criteria>
    - canvas-grain wrapper present
    - Tier band gradient + tier letter sizing
    - Deep-dive panel uses sticky positioning
    - Drag-rank logic preserved (grep for existing handler patterns)
    - audit-champion-pool.spec.ts passes
    - Both compile targets green
  </acceptance_criteria>
  <done>champion_pool.rs tier list + deep-dive sidebar; logic preserved.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Restyle src/pages/game_plan.rs (visual layer only)</name>
  <files>src/pages/game_plan.rs</files>
  <read_first>
    - src/pages/game_plan.rs (FULL FILE 1,515 lines — preserve auto-save, server fns)
    - /tmp/lol-design-handoff/lol-team-companion-app/project/extra-variants.jsx (game-plan related screens)
    - /tmp/lol-design-handoff/lol-team-companion-app/project/pandemonium-variants.jsx (game-plan in pandemonium theme)
    - .planning/phases/17-ui-consolidation/17-UI-SPEC.md (lines 510-516 — Game Plan Page Layout note: "no structural change to server functions; visual layer only")
    - .claude/rules/leptos-patterns.md
  </read_first>
  <action>
    Per UI-SPEC §"Game Plan Page Layout" (which says "no structural change to server functions; visual layer only"):

    1. PRESERVE all server fns, ChampionAutocomplete invocations, debounced auto-save logic, role assignment logic.

    2. RESTYLE:
       - Outer wrapper: `canvas-grain bg-base min-h-screen px-8 py-6`.
       - Page header: imperial eyebrow + display headline.
       - Cards use `gilt` variant (`bg-elevated border border-outline rounded-xl p-6 relative` with optional GiltCorner ornaments).
       - Inputs: standard utility-tier input pattern (`bg-surface/50 border border-outline/50 rounded-lg px-3 py-3 ...`).
       - Buttons: primary (accent), ghost (text-muted), destructive.
       - HeraldicDivider between major sections (per UI-SPEC §"Ornament Library" — "Section breaks on Strategy/History/Profile hub pages").

    3. NO `outline:none` without ring; no raw hex.

    Read the bundle's game-plan-related JSX files to identify the exact component structure (Strategy Room hero panel? card layouts? heading patterns?). The bundle says game-plan screens live across `extra-variants.jsx` and `pandemonium-variants.jsx` — find the relevant components by searching for "game" or "plan" identifiers.
  </action>
  <verify>
    <automated>cargo check --features ssr 2>&1 | tail -3 && cargo check --features hydrate --target wasm32-unknown-unknown 2>&1 | tail -3 && grep -q "canvas-grain" src/pages/game_plan.rs && grep -q "font-imperial\|font-display" src/pages/game_plan.rs && ! grep -qE "#[0-9a-fA-F]{6}" src/pages/game_plan.rs && cd e2e && npx playwright test audit-game-plan.spec.ts 2>&1 | tail -3</automated>
  </verify>
  <acceptance_criteria>
    - canvas-grain wrapper, font-display headlines, gilt cards
    - Server fns preserved; auto-save still works
    - audit-game-plan.spec.ts passes
    - Both compile targets green; no raw hex
  </acceptance_criteria>
  <done>game_plan.rs visual restyle; logic preserved.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Restyle src/pages/post_game.rs (CAUTION: recursion_limit=512)</name>
  <files>src/pages/post_game.rs</files>
  <read_first>
    - src/pages/post_game.rs (FULL FILE — heavy nested view types; do NOT lower recursion_limit per CLAUDE.md rule 38)
    - /tmp/lol-design-handoff/lol-team-companion-app/project/extra-variants.jsx + pandemonium-variants.jsx (post-game screens)
    - .planning/phases/17-ui-consolidation/17-UI-SPEC.md (lines 519-523 — Post-Game Page Layout: "no structural change; visual layer only")
    - src/lib.rs and src/main.rs (`#![recursion_limit = "512"]` — DO NOT TOUCH)
  </read_first>
  <action>
    Per UI-SPEC §"Post-Game Page Layout":

    1. PRESERVE: all server fns, nested view types (recursion_limit=512 stays), pattern analysis logic.

    2. RESTYLE same as game_plan: canvas-grain wrapper, imperial eyebrow + display headline, gilt cards, restyled inputs, HeraldicDivider section breaks.

    3. CRITICAL: Do NOT add additional view! macro nesting that pushes past recursion_limit. Each restyle change should be a class-string swap, not a structural addition. If a new wrapper div is needed, ensure it does not deepen the type tree (use intentional `.into_any()` boundaries if the compiler complains).

    4. Read post-game-related screens in the design bundle (extra-variants.jsx, pandemonium-variants.jsx) to identify component patterns — search for "post" or "review".
  </action>
  <verify>
    <automated>cargo check --features ssr 2>&1 | tail -3 && cargo check --features hydrate --target wasm32-unknown-unknown 2>&1 | tail -3 && grep -q "canvas-grain" src/pages/post_game.rs && ! grep -qE "#[0-9a-fA-F]{6}" src/pages/post_game.rs && grep -q "recursion_limit" src/lib.rs && cd e2e && npx playwright test audit-post-game.spec.ts 2>&1 | tail -3</automated>
  </verify>
  <acceptance_criteria>
    - recursion_limit unchanged in src/lib.rs and src/main.rs (still "512")
    - canvas-grain wrapper, semantic tokens
    - Both compile targets green
    - audit-post-game.spec.ts passes
  </acceptance_criteria>
  <done>post_game.rs visual restyle; nested view types compile; logic preserved.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 4: User reviews /champion-pool, /game-plan, /post-game</name>
  <what-built>
    - /champion-pool: tier list with deep-dive sidebar, drag-rank preserved
    - /game-plan: canvas-grain + gilt cards, server fns preserved
    - /post-game: canvas-grain restyle, recursion_limit=512 preserved
  </what-built>
  <how-to-verify>
    1. Visit `/champion-pool`, screenshot. Test drag-rank, click-to-add via autocomplete, deep-dive panel sticky scroll. Run `npx playwright test audit-champion-pool.spec.ts`.
    2. Visit `/game-plan`, screenshot. Test pick assignments via ChampionAutocomplete, save plan, verify auto-save works. Run `npx playwright test audit-game-plan.spec.ts`.
    3. Visit `/post-game`, screenshot. Test pattern analysis still works, server fns produce expected outputs. Run `npx playwright test audit-post-game.spec.ts`.

    APPROVAL CRITERIA: All three visuals match prototypes; all tests pass; no behavioral regressions.
  </how-to-verify>
  <resume-signal>Type "approved" — Plan 03c complete; proceed to Plan 03d.</resume-signal>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| client (browser) → champion-pool/game-plan/post-game server fns | Persisted user content (pools, plans, post-game notes) |
| ChampionAutocomplete → on_select callback | Champion ID strings flow from typeahead UI to parent state |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-17-13 | Denial of Service | recursion_limit=512 in post_game.rs view types | mitigate | Task 3 explicitly forbids deepening view tree; restyle is class-string swap only; CLAUDE.md rule 38 enforced |
| T-17-11c | Tampering | restyle may break debounced auto-save in game_plan.rs | mitigate | Task 2 preserves server fn signatures + auto-save Effect; audit-game-plan.spec.ts gate enforces |
</threat_model>

<verification>
- /champion-pool, /game-plan, /post-game restyled and individually approved per D-10
- All audit-*.spec.ts tests pass
- `cargo check --features ssr` and `--features hydrate` exit 0
- No raw hex in any modified file
</verification>

<success_criteria>
1. /champion-pool matches Claude Design tier list + sticky deep-dive panel.
2. /game-plan + /post-game match Claude Design hero pages; logic preserved.
3. recursion_limit=512 unchanged in src/lib.rs and src/main.rs.
4. All three audit-*.spec.ts tests pass — no behavioral regression.
5. Each restyle committed atomically with user approval per D-10.
</success_criteria>

<output>
After completion, create `.planning/phases/17-ui-consolidation/17-03c-SUMMARY.md`.
</output>
