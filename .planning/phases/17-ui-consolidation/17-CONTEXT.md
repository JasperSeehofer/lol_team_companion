# Phase 17: UI Consolidation - Context

**Gathered:** 2026-05-07
**Status:** Ready for `/gsd-ui-phase 17` (then plan/execute)

<domain>
## Phase Boundary

Bring the LoL Team Companion app to launch-ready visual polish across **all** existing pages (~14 routes) by:

1. Producing `17-UI-SPEC.md` via `/gsd-ui-phase 17` — project-specific UI decisions only (page/route inventory, draft-board layout, tree-graph interactions, auth flows, champion-picker UX, bug-report widget placement, closed-beta surfaces). Tokens/colors/typography stay in vault per CLAUDE.md UI-SPEC scope rule.
2. Implementing the **existing Claude Design handoff bundle** (downloaded ZIP at `~/Downloads/LoL Team Companion App-handoff.zip`) for hero pages and shared foundations. The bundle includes: Strategy Room dashboard, Live Match overlay, Match History, Profile (Captain's Folio), draft-boards, tree-drafter, champion-pool, match-detail, solo-dashboards, team-dashboards, onboarding, foundations, and `themes.css` (`demacia` default + `pandemonium` variant).
3. Filling **gap surfaces** with Open-Design HTML prototypes (small/utility surfaces only — see tool split in D-12).
4. Adopting the **demacia/pandemonium** theme system, retiring the existing 5-accent palette (yellow, blue, purple, emerald, rose).
5. Generating **AI background imagery** (FLUX.1) for region-reflective immersion — scope undecided beyond "at least the closed-beta landing".
6. Per-page review gate: implement → screenshot via agent-browser → user approves → atomic commit → next page.
7. Final `/gsd-ui-review` produces PASS verdict on all 6 pillars.

**Touches:** `src/pages/**`, `src/components/**`, `input.css` (`@theme` block + `@font-face`), `src/app.rs` (nav restructure to 4 hubs), `public/fonts/` (new), `public/img/` (new for AI backgrounds), `themes.css` content merged into `input.css`. Likely deletes the old accent picker in `src/components/theme_toggle.rs`.

**Does not include** (deferred to other phases or explicitly out of scope):
- Bug-report widget *behavior* (Phase 18) — only its visual placement/anatomy is in this phase.
- Closed-beta gate *logic* (Phase 19.1) — only the UI surfaces (landing + acceptance form) are in this phase.
- Real-time draft sync, mobile responsive redesign, new features.

</domain>

<spec_lock>
## Requirements (locked via SPEC.md seed)

`17-SPEC.md` is a **SEED** — produced by the v1.2 → v1.3 pivot on 2026-05-06. It frames the phase but does not enumerate locked numbered requirements. The actual UI design contract is produced by `/gsd-ui-phase 17` → `17-UI-SPEC.md` next.

Downstream agents MUST read `17-SPEC.md` before planning to understand the phase intent (Goal, Why, In-scope, Out-of-scope, Success criteria, Required reading list).

**In scope (from SPEC.md):**
- `/gsd-ui-phase 17` → `17-UI-SPEC.md` (project-specific UI decisions only).
- Claude Design primary mockups for pages lacking final polish — *the handoff zip already exists*; this phase implements it.
- Open-Design HTML prototypes for surfaces missing from the Claude Design bundle.
- Implementation: bring the codebase up to mockups (page-by-page).
- `/gsd-ui-review` retroactive 6-pillar audit → PASS.

**Out of scope (from SPEC.md):**
- Tokens/colors/typography/accessibility — already in vault `wiki/concepts/design-system.md`, `ui-guidelines.md`, `accessibility-standards.md`. Do NOT re-specify in `17-UI-SPEC.md`.
- New features (this is polish only). Bug-report widget behavior (Phase 18) and closed-beta gate logic (Phase 19.1) stay in their own phases — only their visual surfaces are in this phase.

**Success criteria carried into this CONTEXT.md (from SPEC.md):**
1. `17-UI-SPEC.md` exists, scoped per CLAUDE.md (project-specific only).
2. Implementation matches the design (visual diff or screenshot comparison).
3. `/gsd-ui-review` produces PASS on all 6 pillars.
4. No `outline:none` without ring replacement (`[[guardrails#G-12]]`).
5. No raw hex colors in components (`[[guardrails]]`); semantic tokens only.
6. **No Google Fonts CDN** in deployed HTML (`[[guardrails#G-01]]`) — themes.css currently violates this; fix in implementation (D-08).

</spec_lock>

<decisions>
## Implementation Decisions

### Page coverage & triage

- **D-01: All ~14 routes get the polish treatment** — no triage subset. Coverage is exhaustive: home, auth (login/register), profile, team-dashboard, team-roster, team-builder, draft, tree-drafter, stats, match-detail, champion-pool, game-plan, post-game, opponents, action-items, solo-dashboard, personal-learnings, analytics, plus new closed-beta landing and acceptance surfaces.
- **D-02: Source of truth for designs is the existing Claude Design handoff bundle** at `~/Downloads/LoL Team Companion App-handoff.zip` (extracted reference at `/tmp/lol-design-handoff/lol-team-companion-app/project/`). The implementation agent reads `index.html` first to understand the load order (`data.jsx` → `components.jsx` → `foundations.jsx` → component bundles → `screens/` → `app.jsx`), then implements per-page.
- **D-03: Gap pages without Claude Design coverage are filled by Open-Design** — see D-12 for the per-surface tool split.

### Theme system

- **D-04: Adopt `demacia` (default) + `pandemonium` from the design's `themes.css`. Retire the existing 5-accent palette** (yellow, blue, purple, emerald, rose). The accent picker in `src/components/theme_toggle.rs` becomes a 2-theme toggle. Existing user accent preferences are dropped — acceptable since pre-launch users are internal-only.
- **D-05: Theme tokens move into `input.css` `@theme` block** following the existing semantic token convention. The design's `themes.css` content (CSS custom properties under `[data-theme="demacia"]` / `[data-theme="pandemonium"]`) is ported into `input.css`. Procedural backgrounds (radial gradients + SVG fractal-noise filters from themes.css) are preserved.
- **D-06: Theme persists per-user on the DB `user` record**, mirroring the Phase 12 `mode` precedent (D-04 in 16-CONTEXT.md). Survives hard navigation. Not a localStorage-only signal.

### Visual fidelity & nav structure

- **D-07: Pixel-perfect visuals, idiomatic Leptos structure.** Match the prototype's colors, spacing, typography, layout, and interaction states exactly. But components stay in idiomatic Leptos style (server fns, `Resource::new`, `RwSignal`, semantic tokens, `<For>` with stable IDs, etc.). The prototype's React structure is reference, not template.
- **D-08: Self-host all 5 font families locally per `[[guardrails#G-01]]`.** Cinzel, Cormorant Garamond, Inter, JetBrains Mono, VT323. Download `.woff2` files at the weights/styles the design actually uses (audit `themes.css` and component files first), drop in `public/fonts/`, declare via `@font-face` in `input.css`. Remove the Google Fonts CDN import line from `themes.css` before merging into `input.css`.
- **D-09: Adopt the design's 4-route primary nav verbatim — Strategy / Live / History / Profile — and regroup the existing 19 routes underneath as sub-routes.** Information architecture restructure: top nav shows the 4 hubs from `app.jsx`'s `ROUTES` array; existing pages become contextual sub-nav tabs inside their hub.
  - **Strategy hub:** `/draft`, `/tree-drafter`, `/champion-pool`, `/game-plan`, `/post-game`, `/opponents`, `/action-items`
  - **Live hub:** Live match overlay (new — design's `screens/live.jsx`)
  - **History hub:** `/stats`, `/match-detail`, `/personal-learnings`, `/analytics`
  - **Profile hub:** `/profile`, `/team/dashboard`, `/team/roster`, `/team-builder`
  - **Routes preserved** (same paths, same auth gates) — only the nav grouping changes. Plan-phase decides whether the URL paths get cleaner names like `/strategy/draft` or stay flat.

### Implementation cadence

- **D-10: Per-page review gate.** Implement one page → run dev server → take agent-browser screenshot → user approves or requests revision → atomic commit → move to next. Slower velocity than batched but catches drift early. Matches the existing GSD per-task atomic commit pattern.
- **D-11: One plan per hub** (4 hubs = 4 plans) plus separate plans for: (a) shared foundations + theme port + font self-hosting, (b) closed-beta surfaces, (c) Open-Design seeding + utility surfaces. Final shape decided by gsd-planner; rough budget = ~7 plans.

### Tool split (Claude Design vs. Open-Design)

- **D-12: Hero pages → Claude Design (existing handoff bundle).** Game-plan, post-game, team-dashboard variants, solo-dashboard, draft, tree-drafter, champion-pool, profile, history (stats/match-detail), home/strategy dashboard, **closed-beta landing** (D-15 bumps this to hero tier).
- **D-13: Small/utility surfaces → Open-Design HTML prototypes.** Auth (login + register), team/roster, team-builder, action-items, opponents, personal-learnings, analytics, bug-report widget, **closed-beta acceptance form** (the `/auth/register?invite=ABC123` view).

### Closed-beta surfaces (visual only — gate logic in Phase 19.1)

- **D-14: Branded landing for non-invited visitors.** Visiting `/` (or `/auth/register` without invite token) shows a "closed beta with named friends" landing page with login link. `/auth/login` and legal pages (Impressum, DSE) remain public. `/auth/register` requires a valid invite token in the URL.
- **D-15: Closed-beta landing gets full hero treatment** (Claude Design tier, FLUX background image). The first surface most visitors will see — quality matters. The acceptance form (D-13) stays utility-tier.
- **D-16: Invite mechanism = URL query token** (`/auth/register?invite=ABC123`). One screen for the form with hidden invite token. Phase 19.1 implements the token validation + named-friends list lookup. This phase only specifies the registration form's visual + the "invalid invite" error state.

### AI background imagery

- **D-17: FLUX.1 [pro/dev] from Black Forest Labs (German company)** for AI-generated background images. Aligns with project values (EU/open-source preferred). Run via `fal.ai` API, `replicate.com`, or self-host on a GPU. Final compute path decided in implementation plan.
- **D-18: Aesthetic intent = painterly fantasy reflecting region.** Demacia = warm/regal/clean. Pandemonium = cold/intense/chaotic. Final prompt library + style guide produced during `/gsd-ui-phase 17`.
- **D-19: Background image scope is undecided beyond "at least the closed-beta landing".** Possible expansion: hero panels on each hub's dashboard, login page, auth pages, modal overlays. Final scope decided in `/gsd-ui-phase 17` based on performance budget (image size, lazy load, theme-conditional loading).
- **D-20: Generated assets versioned in `public/img/` and tracked in git.** Reproducibility (prompt + seed + model version) recorded alongside each image — record in `.planning/assets/AI-IMAGES.md`. Avoids re-generation on rebuild and keeps the build deterministic.

### Open-Design integration

- **D-21: Seed a custom `lol-companion` design system in `/home/jasper/Repositories/open-design/design-systems/lol-companion/`** (DESIGN.md + tokens). Source: the demacia/pandemonium tokens from the Claude Design handoff `themes.css`, plus typography from `foundations.jsx`, plus the chosen FLUX background palette. This guarantees Open-Design's HTML output is visually consistent with the Claude Design hero pages by construction.
- **D-22: One Open-Design project per surface group is currently empty for this codebase** — `.od/projects/4335183a-...` only contains a re-import of the Claude Design handoff. Real Open-Design work for utility surfaces starts after the design system in D-21 is seeded.
- **D-23: Implementation references Open-Design project paths directly** (e.g. `/home/jasper/Repositories/open-design/.od/projects/{uuid}/{surface}.html`). No copy into the repo — design artifacts live outside, only the resulting Leptos components are tracked. Trade-off: PR diff doesn't reflect design changes; rationale: avoids duplication and large binary artifact churn in this repo.

### Claude's Discretion

- Final URL paths under each hub (flat `/draft` vs nested `/strategy/draft`) — gsd-planner decides based on Leptos routing ergonomics.
- Exact prompt templates for FLUX background generation — UI-SPEC step.
- Choice of FLUX runtime (fal.ai vs replicate vs self-host) — implementation plan.
- Per-page review notes format (markdown checklist vs free text) — set during first per-page review.
- Whether to merge the design's `data.jsx` mock data into a Leptos seed file or keep using the existing `db_seed` binary — gsd-planner.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase-scoped

- `.planning/phases/17-ui-consolidation/17-SPEC.md` — Seed SPEC; defines goal, in-scope/out-of-scope, success criteria, required reading list. **Locked by SPEC.md** — MUST read before planning.
- `.planning/ROADMAP.md` §"Phase 17" — Goal statement and success criteria.

### Design handoff (Claude Design)

- `~/Downloads/LoL Team Companion App-handoff.zip` — Original ZIP delivered by Claude Design (claude.ai/design).
- `/tmp/lol-design-handoff/lol-team-companion-app/project/` — Extracted bundle. Implementation agent reads `index.html` first to learn load order, then walks `data.jsx`, `components.jsx`, `foundations.jsx`, `draft-boards.jsx`, `solo-dashboards.jsx`, `team-dashboards.jsx`, `match-detail.jsx`, `tree-drafter.jsx`, `champion-pool.jsx`, `pandemonium-variants.jsx`, `extra-variants.jsx`, `screens/{dashboard,live,history,profile,canonical,coming-soon}.jsx`, `app.jsx`, and `themes.css`.
- `/tmp/lol-design-handoff/lol-team-companion-app/README.md` — Handoff agent guide (read first).
- Claude Design project URL: `https://api.anthropic.com/v1/design/h/TRpEQW8gpAzeoobJ8eBSWQ` (authenticated; treat the local zip extraction as the source of truth).

### Open-Design (gap fill — to be created)

- `/home/jasper/Repositories/open-design/design-systems/lol-companion/DESIGN.md` — **TO BE CREATED** in D-21 before any utility surface work.
- `/home/jasper/Repositories/open-design/.od/projects/` — Open-Design project workspace. Currently has one project (`4335183a-...`) which is just a re-import of the Claude Design handoff; new projects per D-22 are needed.
- `/home/jasper/Repositories/open-design/AGENTS.md` — Open-Design root agent guide.

### Vault — design system (do NOT re-specify in 17-UI-SPEC.md)

- `../professional-vault/wiki/concepts/design-system.md` — Tokens, `@theme` block, color palette inheritance.
- `../professional-vault/wiki/concepts/ui-guidelines.md` — Component rules.
- `../professional-vault/wiki/concepts/accessibility-standards.md` — A11y standards.

### Project guardrails

- `../professional-vault/wiki/meta/guardrails.md`:
  - **G-01** — No Google Fonts CDN in deployed HTML (D-08 self-hosts).
  - **G-12** — No `outline:none` without ring replacement.
  - **G-13** — Tier-A transparency (relevant to Phase 21, mentioned for awareness).
- `../professional-vault/wiki/meta/values-charter.md` — EU/open-source preference (justifies FLUX.1 selection in D-17).

### Project root

- `CLAUDE.md` §"UI-SPEC.md scope" — UI-SPEC.md scope rule (project-specific decisions only; do not re-specify tokens).
- `CLAUDE.md` §"Code Style" — Semantic tokens (`bg-base`, `bg-surface`, etc.); never raw hex; `text-white` exception for colored buttons.
- `CLAUDE.md` §"Design System" — Design density = standard.

### Prior UI-SPEC examples (for `/gsd-ui-phase 17` to mimic structure)

- `.planning/phases/15-goals-lp-history/15-UI-SPEC.md` — Most recent UI-SPEC, scoped per CLAUDE.md rule.
- `.planning/phases/14-personal-learnings-journal/14-UI-SPEC.md` — Earlier reference.

### Future-phase coordination

- `.planning/ROADMAP.md` §"Phase 18" — Bug-report widget behavior (this phase only specifies its visual placement).
- `.planning/ROADMAP.md` §"Phase 19.1" — Closed-beta gate logic (this phase only specifies the visual surfaces).
- `.planning/ROADMAP.md` §"Phase 21" — Compliance & Transparency (this phase must leave room for Impressum/DSE pages in the public route set per D-14).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`src/components/champion_picker.rs`** — Grid-based champion selection. Will need restyle to match design's `champion-pool.jsx` patterns; signature stays.
- **`src/components/champion_autocomplete.rs`** — Text input with champion dropdown. Used in game-plan, champion-pool. Restyle, keep API.
- **`src/components/draft_board.rs`** (562 lines) — 20-slot draft board (picks + bans), already has highlight-first slot deletion + click-to-clear. Heavy restyle to match `draft-boards.jsx` design variants.
- **`src/components/tree_graph.rs`** (709 lines) — SVG tree visualization with champion edge icons. Needs restyle to match `tree-drafter.jsx`; the children_of HashMap traversal stays.
- **`src/components/nav.rs`** (748 lines) — Top nav, notifications, theme toggle. **Significant restructure for D-09 (4-hub nav)**. Will likely shrink as accent-color picker (D-04) and 19-route flat nav are removed.
- **`src/components/theme_toggle.rs`** — Becomes a 2-theme (demacia/pandemonium) toggle per D-04. Existing accent-color logic is deleted.
- **`src/components/ui.rs`** — `ErrorBanner`, `StatusMessage`. Keep, restyle.
- **`src/components/stat_card.rs`** — Stat display card. Keep, restyle to match design's stat patterns.
- **`input.css`** — `@theme` block for semantic tokens. Add demacia/pandemonium tokens here per D-05; add `@font-face` declarations per D-08.

### Established Patterns

- **Semantic tokens only** in components (`bg-base`, `bg-surface`, `bg-elevated`, `text-primary`, etc.) — never raw hex. Continues unchanged. Demacia/pandemonium themes redefine these tokens, components don't change.
- **Skeleton loading + empty states** on every data page (Phase 5/UX-02 work, validated). Continues; design's loading patterns are layered on top.
- **Toast system** (`Phase 5 UX-03`) for mutation feedback. Continues; design's notification anatomy is restyled-on-top.
- **Per-task atomic commits** (CLAUDE.md guidance + STATE.md decision precedent). D-10 per-page review gate respects this.
- **Page protection template** (`Resource::new(|| (), |_| get_current_user())` + client-side redirect Effect) used across all auth-required pages. Stays for D-14 closed-beta gate (Phase 19.1 implements the new redirect logic; this phase preserves the visual auth flow).

### Integration Points

- **`src/app.rs`** — Router. Likely needs sub-route grouping for D-09 (4 hubs); decision deferred to gsd-planner per "Claude's Discretion" above.
- **`src/main.rs`** — Theme assets need to be served (`public/fonts/`, `public/img/`). Axum static file serving may need adjustment.
- **`src/server/db.rs`** — `theme` field on `user` table for D-06 persistence. Schema migration in `schema.surql` + `Db*` struct + getter/setter functions.
- **`schema.surql`** — Add `theme` field to `user` table; default to `'demacia'`.
- **`Cargo.toml`** — No new Rust crates expected. FLUX runtime is out-of-band (CLI/API call), not embedded.

### Codebase scale (relevant for plan sizing)

- 13 page files in `src/pages/`, 8 components in `src/components/`, ~10,500 lines of page code.
- Heaviest pages by line count: `draft.rs` (3,801), `tree_drafter.rs` (1,610), `champion_pool.rs` (1,356), `team/dashboard.rs` (~2,235 historical), `game_plan.rs` (1,515).
- These heavy pages likely need the most restyle work — gsd-planner should weight plans accordingly.

</code_context>

<specifics>
## Specific Ideas

- **Demacia / Pandemonium = LoL lore regions.** Demacia (kingdom of light, regal, just); Pandemonium (the harrowing, dark, intense). The aesthetic split should evoke that lore split — not just light/dark, but warm/just vs cold/intense.
- **"High-end background images"** = AI-generated splash-art-quality imagery (FLUX.1). Reference style: League of Legends in-client splash art and login backgrounds. Painterly, atmospheric, region-coded.
- **Design's 4-hub nav (Strategy / Live / History / Profile)** evokes a "captain's folio" / strategy-room aesthetic. Reflect that in copy and microinteractions during implementation.
- **Closed-beta landing is the FIRST surface visitors see** — D-15 bumps it to hero tier specifically because of this. Quality of this single page sets the tone for the entire app.
- **The Claude Design handoff zip is the source of truth** — when in doubt, the implementation matches the prototype, not improvises. README in the bundle says: "If anything is ambiguous, ask the user to confirm before you start implementing."
- **Hidden one-off image** in the design bundle: `uploads/draw-92acceeb-9fd2-499d-84e4-12ff75b7ab5d.png`. Inspect during `/gsd-ui-phase 17` to determine its role (might be the existing background reference).

</specifics>

<deferred>
## Deferred Ideas

- **Live Match overlay (`screens/live.jsx` from design)** — designed but not yet a feature. Belongs in a future phase (post-launch, possibly v1.4 or v1.5). The design assets are preserved in the handoff bundle for later use.
- **Mobile responsive redesign** — explicitly out of scope per PROJECT.md. Desktop-first remains the v1 stance.
- **Per-user accent color customization** — replaced by demacia/pandemonium per D-04. If users miss accent personalization post-launch, it's a future feature in the inbox-driven backlog (Phase 25+).
- **Dynamic background image rotation** — could rotate FLUX images per session or per route. Defer to post-launch based on user feedback.
- **Open-Design design-system contribution upstream** — the custom `lol-companion` design system seeded in D-21 could potentially be contributed back to Open-Design. Defer; not a v1.3 launch concern.
- **Magic-link / email-based invite flow** for closed beta — considered, rejected in favor of URL-token (D-16) for v1.3. Could be added in v1.4 if friction emerges.
- **Public landing with "request access" button** — considered, rejected for v1.3 (D-14 gates non-invitees). Could be added if we want to gather a waitlist.

### Reviewed Todos (not folded)

None — no todo matches found for Phase 17.

</deferred>

---

*Phase: 17-ui-consolidation*
*Context gathered: 2026-05-07*
