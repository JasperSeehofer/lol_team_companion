# Phase 17 — UI Consolidation (SPEC seed)

**Status:** SEED — produced by the v1.2 → v1.3 pivot on 2026-05-06. Run `/gsd-ui-phase 17` to produce the actual UI-SPEC.md, then plan/execute.

**Milestone:** v1.3 Launch Readiness

## Goal (one sentence)

A coherent, polished UI across all v1 pages — produced from a project-level UI-SPEC.md, designed primarily by Claude Design, with Open-Design filling missing surfaces, then audited via `/gsd-ui-review`.

## Why this phase exists

The app has accumulated 14 pages across 5 milestones (v1.0–v1.2) with UI patterns that have been refined incrementally. Before launch we need a deliberate consolidation pass: identify inconsistencies, generate primary mockups via Claude Design, fill any missing surfaces with Open-Design HTML prototypes, then audit. This is the "polish" phase before users see anything.

## In-scope

1. **`/gsd-ui-phase 17`** produces `17-UI-SPEC.md` covering project-specific UI decisions (per CLAUDE.md "UI-SPEC.md scope" rule):
   - Page/route inventory across all 14 routes
   - Draft board layout (linear + tree drafter)
   - Tree graph interactions
   - Auth flows (register, login, logout, redirects)
   - Champion picker UX (grid + autocomplete variants)
   - Bug-report widget placement (coordinates with Phase 18)
   - Closed-beta-only screens (invite landing, named-friends acceptance flow)
2. **Claude Design primary pass**: invoke Claude Design for primary mockups of pages lacking final polish. User reviews and accepts.
3. **Open-Design gap fill**: for any surface not covered by the primary pass (e.g. closed-beta invite flow, bug-report inbox widget), use Open-Design at `/home/jasper/Repositories/open-design` to generate HTML prototypes.
4. **Implementation**: bring the codebase up to the agreed mockups (page-by-page).
5. **`/gsd-ui-review`** retroactive 6-pillar audit produces PASS verdict.

## Out of scope

- Tokens, colors, typography, accessibility — already in vault `wiki/concepts/design-system.md`, `ui-guidelines.md`, `accessibility-standards.md` per CLAUDE.md. Do NOT re-specify in 17-UI-SPEC.md.
- New features (this is polish only). New surfaces are limited to those required by Phase 18 (bug-report widget) and the closed-beta flow.

## Success criteria (verify with `/gsd-verify-work 17` + `/gsd-ui-review`)

1. `17-UI-SPEC.md` exists, scoped per CLAUDE.md (project-specific only, not re-specifying tokens)
2. Primary Claude Design pass produced for at least the v1.0 + v1.1 pages that have not been touched since v1.1 (likely candidates: `/draft`, `/tree-drafter`, `/team-builder`, `/profile`)
3. Open-Design prototypes exist for any new closed-beta surface
4. Implementation diff matches mockups (visual diff or screenshot comparison)
5. `/gsd-ui-review` produces PASS on all 6 pillars
6. No `outline:none` without ring replacement (per `[[guardrails#G-12]]`)
7. No raw hex colors in components (per `[[guardrails]]`); semantic tokens only

## Required reading before discuss-phase

1. `CLAUDE.md` — UI-SPEC scope rule, semantic token rules, Code Style section
2. `wiki/concepts/design-system.md`, `ui-guidelines.md`, `accessibility-standards.md` (vault)
3. `src/pages/` — current state of every page (skim, don't deep-read)
4. Past UI-SPEC examples: `.planning/phases/15-goals-lp-history/15-UI-SPEC.md`, `.planning/phases/14-personal-learnings-journal/14-UI-SPEC.md`

## Hetzner / deploy interactions

None directly. Phase 20 deploys whatever Phase 17 produces.

## Plans

TBD — produced by `/gsd-plan-phase 17`. Likely structure: 1 plan per page-group + 1 plan for closed-beta surfaces.

---

This SPEC was seeded by the pivot. The actual UI-SPEC.md content is produced by `/gsd-ui-phase 17`.
