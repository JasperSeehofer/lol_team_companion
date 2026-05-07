# Phase 17: UI Consolidation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-07
**Phase:** 17-ui-consolidation
**Areas discussed:** Page triage, Claude Design flow, Closed-beta screens, Open-Design handoff

---

## Page triage

### Q1: How do we determine which pages need a full polish pass?

| Option | Description | Selected |
|--------|-------------|----------|
| Screenshot audit first (Recommended) | agent-browser screenshots all routes, visual scan, list inconsistencies | |
| Trust the SPEC list + new surfaces only | Accept SPEC's 4 named pages + closed-beta + bug-report widget | |
| All pages get the treatment | Every page goes through the polish pass | ✓ |

**User's choice:** All pages get the treatment.
**Notes:** Coverage is exhaustive; no triage subset.

---

### Q2: What does "polished" actually mean across all pages?

| Option | Description | Selected |
|--------|-------------|----------|
| Visual consistency | Unify card depth, spacing, type scale, empty states | |
| Component consolidation | Deduplicate components doing the same job | |
| Both — visual + component | Full pass | |

**User's choice:** Free-text — *"Claude Design has already prepared some designs which are ready. Missing sites will be identified and also created."*
**Notes:** Existing Claude Design handoff bundle is the source of truth; gap pages get filled.

---

### Q3: Where do the existing Claude Design designs live?

| Option | Description | Selected |
|--------|-------------|----------|
| In-conversation / session-based | Reference by description from Claude session | |
| Files on disk | Saved as files locally or in repo | |
| Open-Design artifacts | Open-Design is source of truth | |

**User's choice:** Free-text — *"You can find the designs here: https://api.anthropic.com/v1/design/h/TRpEQW8gpAzeoobJ8eBSWQ"* (later: zip downloaded to `~/Downloads/LoL Team Companion App-handoff.zip`)
**Notes:** URL is authenticated; agent extracted the downloaded zip to `/tmp/lol-design-handoff/` and inspected the bundle.

---

### Q4: How do we reconcile the design's 2-theme system with the existing 5-accent palette?

| Option | Description | Selected |
|--------|-------------|----------|
| Adopt demacia/pandemonium, retire accents (Recommended) | Design contract becomes source of truth | ✓ |
| Keep accents, ignore design themes | Mine demacia for tokens but keep accent picker | |
| Both | demacia/pandemonium top-level + accents within each | |

**User's choice:** Adopt demacia/pandemonium, retire accents.

---

### Q5: How do we produce designs for the gap pages?

| Option | Description | Selected |
|--------|-------------|----------|
| Open-Design fills all gaps (Recommended per SPEC) | OD generates HTML for every missing surface | |
| Extend the Claude Design handoff | Go back to claude.ai/design and add screens | |
| Mix: Open-Design for new, infer the rest | OD only for new surfaces; existing-but-undesigned = infer from foundations | |

**User's choice:** Free-text — *"smaller pages can be done with Open-Design. Real Heros and new region designs can be done with Claude Design."*

---

### Q6: Tool split — which page goes to which tool?

| Option | Description | Selected |
|--------|-------------|----------|
| Claude Design = hero pages; Open-Design = small/utility | game-plan, post-game, team-dashboard variants vs auth, roster, action-items, bug-report widget, closed-beta invite | ✓ |
| Claude Design = anything user-facing major | Broader CD coverage incl. opponents, personal-learnings, analytics | |
| Let me list them by name | Spell out each page | |

**User's choice:** Claude Design = hero pages; Open-Design = small/utility.

---

### Q7: Use case for AI-generated background images?

| Option | Description | Selected |
|--------|-------------|----------|
| Hero / atmospheric for landing & login | Splash-art for /, /auth/login, /auth/register | |
| Page backgrounds throughout the app | Subtle painterly behind every workflow page | |
| Per-region atmospheric | demacia/pandemonium per-screen background palette | |

**User's choice:** Free-text — *"undecided yet. Should reflect the region and make the app more immersive. Can be just for landing page, but I could also imagine it for other pages"*
**Notes:** Scope deferred to `/gsd-ui-phase 17`; minimum = closed-beta landing.

---

### Q8: Which AI image model for background art?

| Option | Description | Selected |
|--------|-------------|----------|
| FLUX.1 [pro/dev] (Recommended) | Black Forest Labs (German), top-tier quality, EU-aligned | ✓ |
| SDXL + LoRA fine-tunes | Open-source, runs locally, more setup | |
| Midjourney v6+ | Best fantasy aesthetic but US/closed | |
| Defer the model choice | Lock aesthetic, decide model later | |

**User's choice:** FLUX.1 [pro/dev].
**Notes:** Aligns with project values (EU/open-source preference per values-charter.md).

---

## Claude Design flow

### Q9: Fidelity bar for the Leptos port?

| Option | Description | Selected |
|--------|-------------|----------|
| Pixel-perfect visuals, idiomatic Leptos structure (Recommended) | Match visuals exactly; components stay in Leptos style | ✓ |
| Visual intent + Leptos liberties | Match intent, allow visual differences | |
| Strict 1:1 port including structure | Mirror React component structure as closely as possible | |

**User's choice:** Pixel-perfect visuals, idiomatic Leptos structure.

---

### Q10: How do we reconcile the design's 4-route nav with the app's 19 routes?

| Option | Description | Selected |
|--------|-------------|----------|
| Adopt design's nav verbatim, regroup existing routes (Recommended) | Top nav = 4 hubs; existing routes become sub-routes | ✓ |
| Keep existing flat nav, restyle only | Don't touch IA; design = styling guide only | |
| Hybrid: design's top nav + existing sub-nav | 4 hubs + contextual sub-nav inside each | |

**User's choice:** Adopt design's nav verbatim, regroup existing routes.
**Notes:** This is a significant IA restructure. CONTEXT.md D-09 captures the proposed grouping.

---

### Q11: Implementation staging — review per page or batched?

| Option | Description | Selected |
|--------|-------------|----------|
| Per-page review gate (Recommended) | Implement → screenshot → approve → commit → next | ✓ |
| Batch by hub (4 reviews) | All pages under one hub, then review | |
| Implement everything, review at end | One big review at gsd-ui-review time | |

**User's choice:** Per-page review gate.

---

### Q12: How do we handle Google Fonts CDN usage in `themes.css`?

| Option | Description | Selected |
|--------|-------------|----------|
| Self-host all 5 families locally (Recommended) | Download .woff2, /public/fonts/, @font-face | ✓ |
| Pick a subset — only families actually used | Audit then self-host only needed | |
| Substitute with system/license-friendly | Replace Cinzel/Cormorant with system fonts | |

**User's choice:** Self-host all 5 families locally.
**Notes:** Required by guardrail G-01; non-negotiable.

---

## Closed-beta screens

### Q13: What does a non-invited visitor see?

| Option | Description | Selected |
|--------|-------------|----------|
| Branded landing with "closed beta" message + login link (Recommended) | /, /auth/login, legal visible; register hidden | ✓ |
| Login-only for everyone, no landing | Visitors redirect straight to /auth/login | |
| Public landing + "Request Access" button | Anyone can request invite via form | |

**User's choice:** Branded landing with "closed beta" message + login link.

---

### Q14: What's the invite mechanism in the UI?

| Option | Description | Selected |
|--------|-------------|----------|
| Invite code in URL (e.g. /auth/register?invite=ABC123) (Recommended) | Share URL with friends; click opens pre-validated form | ✓ |
| Email/username allowlist — no code | /auth/register public; submit fails if not on list | |
| Magic-link style | Two screens; requires SMTP infra | |

**User's choice:** Invite code in URL.

---

### Q15: How polished should closed-beta UI surfaces be?

| Option | Description | Selected |
|--------|-------------|----------|
| Hero treatment (Claude Design) | Background image, branded typography, full theme | ✓ |
| Themed but minimal (Open-Design) | Inherits tokens but no hero image | |
| Functional only — boilerplate | Plain text + button | |

**User's choice:** Hero treatment (Claude Design).
**Notes:** Bumps closed-beta landing into Claude Design tier (D-15); supersedes the earlier "small/utility → Open-Design" rule for this surface.

---

## Open-Design handoff

### Q16: How do we initiate the Open-Design work?

| Option | Description | Selected |
|--------|-------------|----------|
| One Open-Design project per surface group (Recommended) | Separate OD projects per "auth", "team setup", etc. | |
| Single "lol-utility" project for everything | One OD project with all utility surfaces | |
| Open-Design seeded from Claude Design tokens | Manually seed OD design system from demacia/pandemonium first | ✓ |

**User's choice:** Open-Design seeded from Claude Design tokens/foundations.

---

### Q17: How is the seeding done — new design system or built-in?

| Option | Description | Selected |
|--------|-------------|----------|
| New custom design system in design-systems/ | Hand-author lol-companion DESIGN.md from demacia tokens | ✓ |
| Pick closest built-in (e.g. atelier-zero), override | Use built-in baseline, override per-prototype | |
| Skip seeding — generate then hand-tune | Use any default, manually edit | |

**User's choice:** New custom design system in `/home/jasper/Repositories/open-design/design-systems/`.

---

### Q18: Handoff format from Open-Design to Leptos?

| Option | Description | Selected |
|--------|-------------|----------|
| Export to .planning/design-handoff/, implement from there (Recommended) | OD exports HTML+CSS into the repo | |
| Reference OD project paths directly | Implementation reads from OD's project workspace | ✓ |
| Convert OD output to Leptos scaffolds in a sub-spike | Translate HTML → Leptos component scaffolds first | |

**User's choice:** Reference OD project paths directly during implementation.

---

## Claude's Discretion

- Final URL paths under each hub (flat vs nested) — gsd-planner decides.
- Exact prompt templates for FLUX background generation — UI-SPEC step.
- Choice of FLUX runtime (fal.ai vs replicate vs self-host) — implementation plan.
- Per-page review notes format — set during first review.
- Whether to use design's `data.jsx` as Leptos seed or keep `db_seed` binary — gsd-planner.

## Deferred Ideas

- Live Match overlay (designed but not yet a feature) — future phase.
- Mobile responsive redesign — explicitly out of scope.
- Per-user accent color customization — replaced by demacia/pandemonium.
- Dynamic background image rotation — defer to post-launch.
- Open-Design `lol-companion` upstream contribution — not a v1.3 concern.
- Magic-link / email-based invite flow — defer to v1.4.
- Public landing with "request access" — not for v1.3.

## Notable Process Observations

- **Guardrail flag during discussion:** themes.css uses Google Fonts CDN (line 7) — violates G-01. Captured as D-08 (self-host) and listed in success criteria #6 in `<spec_lock>`.
- **User correction during Open-Design discussion:** existing OD project `4335183a-...` is just a re-import of the Claude Design handoff zip — Open-Design is effectively empty of new work for this codebase. CONTEXT.md D-22 reflects this.
