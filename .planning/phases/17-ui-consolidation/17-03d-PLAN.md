---
phase: 17-ui-consolidation
plan: 03d
type: execute
wave: 2
depends_on: [17-01, 17-02]
files_modified:
  - src/pages/opponents.rs
  - src/pages/action_items.rs
  - src/components/ui.rs
  - src/components/stat_card.rs
  - .planning/phases/17-ui-consolidation/17-OD-MAP.md
autonomous: false
decisions: [D-03, D-07, D-10, D-13]
requirements: [SC-2-claude-design-implementation, SC-3-open-design-seeding, SC-4-ui-review-pass]
must_haves:
  truths:
    - "User visits /opponents (utility tier) and sees Open-Design HTML port (Card.plain layout)"
    - "User visits /action-items (utility tier) and sees Open-Design HTML port"
    - "src/components/ui.rs ErrorBanner uses semantic token bg-danger (no raw bg-red-500)"
    - "src/components/stat_card.rs uses Card.elevated pattern with semantic tokens"
    - "Each visual gate in this plan passes per-page user review per D-10 before its commit"
  artifacts:
    - path: "src/pages/opponents.rs"
      provides: "Utility-tier opponents page (Open-Design port)"
    - path: "src/pages/action_items.rs"
      provides: "Utility-tier action items page (Open-Design port)"
    - path: "src/components/ui.rs"
      provides: "ErrorBanner / StatusMessage / EmptyState restyled with semantic tokens"
    - path: "src/components/stat_card.rs"
      provides: "Card.elevated pattern stat card"
  key_links:
    - from: "src/pages/opponents.rs"
      to: "/home/jasper/Repositories/open-design/.od/projects/{uuid}/opponents.html"
      via: "Open-Design HTML port (recorded in 17-OD-MAP.md)"
      pattern: "lol-companion|17-OD-MAP"
    - from: "src/components/ui.rs"
      to: "src/pages/*"
      via: "ErrorBanner imported across all pages"
      pattern: "ErrorBanner"
---

<objective>
Restyle the utility-tier Strategy hub pages (`/opponents`, `/action-items`) via Open-Design HTML, plus the shared `ui.rs` and `stat_card.rs` components used across the rest of the app. All page logic preserved; visual layer only.

Per D-13 utility-tier surfaces are generated via Open-Design first, then ported to Leptos. Per D-10 each page restyle is a per-page review gate.

This plan was split out from the original 17-03 to bound context/drift on the utility-tier + shared-components cluster.

Output: 2 restyled pages + 2 restyled shared components + per-page atomic commits + 17-OD-MAP.md updates.
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
@.planning/phases/17-ui-consolidation/17-OD-MAP.md
@/home/jasper/Repositories/open-design/design-systems/lol-companion/DESIGN.md
@.claude/rules/leptos-patterns.md
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Restyle utility-tier /opponents and /action-items via Open-Design HTML</name>
  <files>src/pages/opponents.rs, src/pages/action_items.rs, .planning/phases/17-ui-consolidation/17-OD-MAP.md</files>
  <read_first>
    - src/pages/opponents.rs (FULL FILE — preserve server fns)
    - src/pages/action_items.rs (FULL FILE — preserve server fns)
    - .planning/phases/17-ui-consolidation/17-OD-MAP.md (utility surface tracking — fill in OD UUID + path columns for opponents and action-items as you generate)
    - .planning/phases/17-ui-consolidation/17-UI-SPEC.md (lines 184-215 Route Inventory — utility tier per D-13)
    - /home/jasper/Repositories/open-design/design-systems/lol-companion/DESIGN.md (component patterns from plan 02)
  </read_first>
  <action>
    Per D-13 (utility tier → Open-Design):

    1. **Generate Open-Design HTML for opponents and action-items** following the conventions documented in 17-OD-MAP.md. This produces `.html` files in `/home/jasper/Repositories/open-design/.od/projects/{new-uuid}/...`. Use the seeded `lol-companion` design system. Update 17-OD-MAP.md rows for these surfaces with the resulting UUIDs and HTML paths.

    2. **Port the OD HTML output to Leptos**:
       - `src/pages/opponents.rs`: PRESERVE all server fns and existing data flow. Wrap in `canvas-grain bg-base min-h-screen`. Apply utility-tier visual: simpler than gilt cards — `Card.plain` (`bg-elevated border border-divider rounded-xl p-6`), restyled `Input.text`, primary/ghost buttons.
       - `src/pages/action_items.rs`: same treatment.

    3. NO raw hex; G-12 focus rings.
  </action>
  <verify>
    <automated>cargo check --features ssr 2>&1 | tail -3 && cargo check --features hydrate --target wasm32-unknown-unknown 2>&1 | tail -3 && grep -q "canvas-grain" src/pages/opponents.rs && grep -q "canvas-grain" src/pages/action_items.rs && ! grep -qE "#[0-9a-fA-F]{6}" src/pages/opponents.rs src/pages/action_items.rs && grep -q "opponents\|action-items" .planning/phases/17-ui-consolidation/17-OD-MAP.md && cd e2e && npx playwright test audit-misc-pages.spec.ts 2>&1 | tail -3</automated>
  </verify>
  <acceptance_criteria>
    - opponents.rs + action_items.rs use Open-Design utility tier patterns
    - 17-OD-MAP.md status updated for these 2 surfaces
    - Both compile targets green; no raw hex
    - Existing Playwright tests for these pages still pass
  </acceptance_criteria>
  <done>opponents + action-items restyled via Open-Design; OD-MAP updated.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Restyle src/components/ui.rs (ErrorBanner, StatusMessage, EmptyState, ToastOverlay)</name>
  <files>src/components/ui.rs</files>
  <read_first>
    - src/components/ui.rs (FULL FILE — ErrorBanner, StatusMessage, ToastOverlay, EmptyState, NoTeamState)
    - .planning/phases/17-ui-consolidation/17-PATTERNS.md (ui.rs analog at lines 609-630)
    - /home/jasper/Repositories/open-design/design-systems/lol-companion/DESIGN.md (ErrorBanner pattern)
  </read_first>
  <action>
    1. **Restyle src/components/ui.rs** (used across all pages):
       - PRESERVE: `ToastProvider` + `ToastOverlay` (4-second auto-dismiss, 3-toast cap, fixed-position).
       - PRESERVE: `EmptyState`, `NoTeamState`, `SkeletonLine`/`Card`/`Grid` API.
       - RESTYLE `ErrorBanner`: `bg-danger/10 border border-danger/30 text-danger rounded-lg px-4 py-3 text-sm` (replace existing `bg-red-500/10` raw color).
       - RESTYLE `StatusMessage` similarly with semantic tokens.
       - Add HeraldicDivider as a section divider primitive if needed.

    2. G-12 + no raw hex.
  </action>
  <verify>
    <automated>cargo check --features ssr 2>&1 | tail -3 && cargo check --features hydrate --target wasm32-unknown-unknown 2>&1 | tail -3 && ! grep -qE "bg-red-500" src/components/ui.rs && grep -q "bg-danger" src/components/ui.rs && ! grep -qE "#[0-9a-fA-F]{6}" src/components/ui.rs</automated>
  </verify>
  <acceptance_criteria>
    - ui.rs ErrorBanner uses `bg-danger` (semantic token), not `bg-red-500` (raw)
    - ToastProvider / ToastOverlay / EmptyState / NoTeamState / SkeletonLine API unchanged
    - Both compile targets green; no raw hex; G-12 compliant
  </acceptance_criteria>
  <done>ui.rs restyled; semantic tokens throughout.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Restyle src/components/stat_card.rs (Card.elevated pattern)</name>
  <files>src/components/stat_card.rs</files>
  <read_first>
    - src/components/stat_card.rs (FULL FILE — 19-line component)
    - /home/jasper/Repositories/open-design/design-systems/lol-companion/DESIGN.md (Card.elevated pattern)
  </read_first>
  <action>
    1. Apply `Card.elevated` pattern: `bg-surface border border-outline/50 rounded-xl p-4`.
    2. Stat number: `text-2xl font-display text-primary`.
    3. Label: `text-xs text-muted uppercase tracking-wider font-imperial`.
    4. Optional accent stripe.
    5. G-12 + no raw hex.
  </action>
  <verify>
    <automated>cargo check --features ssr 2>&1 | tail -3 && cargo check --features hydrate --target wasm32-unknown-unknown 2>&1 | tail -3 && grep -q "font-display\|font-imperial" src/components/stat_card.rs && ! grep -qE "#[0-9a-fA-F]{6}" src/components/stat_card.rs</automated>
  </verify>
  <acceptance_criteria>
    - stat_card.rs uses Card.elevated pattern + semantic tokens
    - Both compile targets green; no raw hex
  </acceptance_criteria>
  <done>stat_card.rs Card.elevated; semantic tokens.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 4: User reviews /opponents, /action-items, and shared ui.rs/stat_card.rs</name>
  <what-built>
    - /opponents + /action-items restyled to utility tier via Open-Design
    - ui.rs: ErrorBanner uses bg-danger; ToastOverlay preserved
    - stat_card.rs: Card.elevated pattern
    - 17-OD-MAP.md updated for these surfaces
  </what-built>
  <how-to-verify>
    1. Visit `/opponents`, `/action-items` — screenshot each.
    2. Compare to OD HTML output and DESIGN.md component patterns.
    3. Trigger an error condition on any page → ErrorBanner shows with semantic colors (red tint via bg-danger, not raw red).
    4. Trigger a toast (mutation feedback) → ToastOverlay still functions.
    5. Run `cd e2e && npx playwright test audit-misc-pages.spec.ts 2>&1 | tail -3`.

    APPROVAL CRITERIA: Visuals match utility-tier OD output; toast + banner still work; no regressions on other pages using ui.rs.
  </how-to-verify>
  <resume-signal>Type "approved" — Plan 03d complete; Strategy hub fully restyled.</resume-signal>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| client (browser) → opponents/action_items server fns | Persisted user content |
| OD HTML import → Leptos view! macro | Cross-repo content transcribed manually |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-17-14 | Tampering | OD HTML import may include unexpected scripts/styles | mitigate | Task 1 ports HTML manually to Leptos view! macros; no `<script>` or external `<link>` tags imported |
| T-17-12d | Information Disclosure | inline SVG data URLs in canvas-grain | accept | Decorative noise pattern; no PII or secrets |
</threat_model>

<verification>
- /opponents, /action-items restyled and individually approved per D-10
- audit-misc-pages.spec.ts passes
- ui.rs uses bg-danger not bg-red-500
- `cargo check --features ssr` and `--features hydrate` exit 0
- 17-OD-MAP.md updated for opponents + action-items
- No raw hex in any modified file
</verification>

<success_criteria>
1. /opponents + /action-items match Open-Design utility tier.
2. ui.rs uses semantic tokens only (no raw bg-red-500 etc.).
3. stat_card.rs uses Card.elevated pattern.
4. audit-misc-pages.spec.ts passes — no behavioral regression.
5. Each page committed atomically with user approval per D-10.
</success_criteria>

<output>
After completion, create `.planning/phases/17-ui-consolidation/17-03d-SUMMARY.md`.
</output>
