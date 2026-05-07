# Phase 17: UI Consolidation - Research

**Researched:** 2026-05-07
**Domain:** Multi-page UI port (Claude Design handoff → Leptos), theme system migration, font self-hosting, IA restructure, AI background imagery, validation strategy
**Confidence:** HIGH — primary sources are local files (handoff bundle, current codebase) verified directly; library references verified via official docs

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Page coverage & triage**
- D-01: All ~14 routes get the polish treatment — no triage subset. Coverage exhaustive across home, auth (login/register), profile, team-dashboard, team-roster, team-builder, draft, tree-drafter, stats, match-detail, champion-pool, game-plan, post-game, opponents, action-items, solo-dashboard, personal-learnings, analytics, plus new closed-beta landing and acceptance surfaces.
- D-02: Source of truth = Claude Design handoff bundle at `/tmp/lol-design-handoff/lol-team-companion-app/project/`. Implementation agent reads `index.html` first to learn load order (`data.jsx` → `components.jsx` → `foundations.jsx` → bundles → `screens/` → `app.jsx`).
- D-03: Gap pages without Claude Design coverage are filled by Open-Design — see D-12.

**Theme system**
- D-04: Adopt `demacia` (default) + `pandemonium` from design's `themes.css`. Retire existing 5-accent palette (yellow, blue, purple, emerald, rose). Theme toggle becomes 2-state.
- D-05: Theme tokens move into `input.css` `@theme` block following the existing semantic-token convention. Procedural backgrounds (radial gradients + SVG fractal-noise filters) preserved.
- D-06: Theme persists per-user on the DB `user` record (mirrors Phase 12 `mode` precedent). Survives hard navigation.

**Visual fidelity & nav structure**
- D-07: Pixel-perfect visuals, idiomatic Leptos structure. Match prototype's colors/spacing/typography/layout/interaction states exactly. Components stay in idiomatic Leptos (server fns, `Resource::new`, `RwSignal`, semantic tokens, `<For>` with stable IDs).
- D-08: Self-host all 5 font families locally per G-01: Cinzel, Cormorant Garamond, Inter, JetBrains Mono, VT323. Drop in `public/fonts/`, declare via `@font-face` in `input.css`. Remove Google Fonts CDN import from `themes.css` before merging.
- D-09: Adopt design's 4-route primary nav verbatim — Strategy / Live / History / Profile — and regroup existing 19 routes underneath as sub-routes. Routes preserved (same paths, same auth gates) — only nav grouping changes.

**Implementation cadence**
- D-10: Per-page review gate. Implement → run dev server → agent-browser screenshot → user approves or requests revision → atomic commit → next.
- D-11: One plan per hub (4 hubs = 4 plans) plus separate plans for: (a) shared foundations + theme port + font self-hosting, (b) closed-beta surfaces, (c) Open-Design seeding + utility surfaces. Final shape decided by gsd-planner; rough budget = ~7 plans.

**Tool split**
- D-12: Hero pages → Claude Design (existing handoff). Game-plan, post-game, team-dashboard variants, solo-dashboard, draft, tree-drafter, champion-pool, profile, history, home/strategy dashboard, closed-beta landing.
- D-13: Small/utility surfaces → Open-Design HTML prototypes. Auth, team/roster, team-builder, action-items, opponents, personal-learnings, analytics, bug-report widget, closed-beta acceptance form.

**Closed-beta surfaces**
- D-14: Branded landing for non-invited visitors. `/auth/login` and legal pages remain public. `/auth/register` requires valid invite token in URL.
- D-15: Closed-beta landing gets full hero treatment (Claude Design tier + FLUX bg).
- D-16: Invite mechanism = URL query token (`/auth/register?invite=ABC123`). Phase 19.1 implements token validation; this phase only specifies the form's visual + invalid-invite error state.

**AI background imagery**
- D-17: FLUX.1 [pro/dev] from Black Forest Labs. Run via fal.ai, replicate.com, or self-host. Final compute path decided in implementation plan.
- D-18: Aesthetic intent = painterly fantasy reflecting region. Demacia = warm/regal/clean. Pandemonium = cold/intense/chaotic.
- D-19: Background-image scope undecided beyond closed-beta landing. Final scope decided in `/gsd-ui-phase 17` based on performance budget — UI-SPEC has already chosen: closed-beta landing (mandatory) + login/auth (optional, performance-permitting).
- D-20: Generated assets versioned in `public/img/` and tracked in git. Reproducibility recorded in `.planning/assets/AI-IMAGES.md`.

**Open-Design integration**
- D-21: Seed custom `lol-companion` design system in `/home/jasper/Repositories/open-design/design-systems/lol-companion/` (DESIGN.md + tokens) before any utility surface work.
- D-22: One Open-Design project per surface group is currently empty for this codebase — only `4335183a-...` re-imports the Claude Design handoff. Real Open-Design work starts after D-21 seed.
- D-23: Implementation references Open-Design project paths directly (e.g. `/home/jasper/Repositories/open-design/.od/projects/{uuid}/{surface}.html`). No copy into repo.

### Claude's Discretion
- Final URL paths under each hub (flat `/draft` vs nested `/strategy/draft`) — gsd-planner decides.
- Exact prompt templates for FLUX background generation — UI-SPEC step (already produced in 17-UI-SPEC.md "AI Background Imagery" table).
- Choice of FLUX runtime (fal.ai vs replicate vs self-host) — implementation plan.
- Per-page review notes format (markdown checklist vs free text).
- Whether to merge design's `data.jsx` mock data into a Leptos seed file or keep `db_seed` binary.

### Deferred Ideas (OUT OF SCOPE)
- Live Match overlay (designed but not yet a feature) — future phase post-launch.
- Mobile responsive redesign — explicitly out of scope.
- Per-user accent color customization — replaced by demacia/pandemonium.
- Dynamic background image rotation — defer to post-launch.
- Open-Design `lol-companion` upstream contribution — not a v1.3 concern.
- Magic-link / email-based invite flow — defer to v1.4.
- Public landing with "request access" — not for v1.3.
</user_constraints>

<phase_requirements>
## Phase Requirements

No formal REQ-IDs map to Phase 17 in REQUIREMENTS.md. The phase contract is:

| Source | Requirement | Research Support |
|--------|-------------|------------------|
| ROADMAP §Phase 17 SC1 | `17-UI-SPEC.md` exists with route inventory, draft-board layout, tree-graph interactions, auth flows, champion picker UX, bug-report widget placement | ✅ Already produced (`17-UI-SPEC.md` exists, 762 lines). |
| ROADMAP §Phase 17 SC2 | Claude Design has produced primary mockups for any page lacking final polish | Handoff bundle confirmed at `/tmp/lol-design-handoff/...`; coverage map below identifies gaps. |
| ROADMAP §Phase 17 SC3 | Open-Design generates HTML prototypes for any new surfaces missing from primary pass | Open-Design infra exists at `/home/jasper/Repositories/open-design`; seeding work blocked on D-21 task. |
| ROADMAP §Phase 17 SC4 | Implementation matches the UI-SPEC; `/gsd-ui-review` produces PASS verdict on the 6 quality dimensions | Per-page review gate (D-10) + final `/gsd-ui-review` strategy detailed in Validation Architecture below. |
| SPEC §6 success criteria | No `outline:none` without ring replacement (G-12); no raw hex in components; no Google Fonts CDN (G-01) | Codebase grep + final-commit verification step. |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

These apply to every task in the phase regardless of which hub:

- **Stack:** Rust nightly · Leptos 0.8 · Axum 0.8 · SurrealDB 3.x · Tailwind CSS v4 (standalone binary, no npm)
- **Dual-target compile:** All shared types in `src/models/` must compile for both `ssr` and `hydrate` features. Server fns and DB code must be guarded `#[cfg(feature = "ssr")]`.
- **BFD linker:** `.cargo/config.toml` forces `link-arg=-fuse-ld=bfd` to work around an LLVM 22 lld crash on nightly. Don't touch.
- **`cargo test --features ssr --lib`** for unit tests — integration tests in `tests/` OOM during BFD linking.
- **Semantic tokens only** in components (`bg-base`, `text-primary`, `border-divider`, etc.) — never raw hex. Exception: literal `text-white` on colored buttons (`bg-red-700 text-white`).
- **Path-specific rule files load automatically:**
  - `.claude/rules/leptos-patterns.md` for `pages/**`, `components/**`
  - `.claude/rules/wasm-patterns.md` for browser/event-handler code
  - `.claude/rules/surreal-patterns.md` for `db.rs`, `*.surql`
- **Per-task atomic commits** (CLAUDE.md guidance). D-10 per-page review gate respects this.
- **UI-SPEC scope (CLAUDE.md):** Project-specific decisions only. Tokens/colors/typography/a11y already in vault `wiki/concepts/design-system.md`, `ui-guidelines.md`, `accessibility-standards.md` — do not re-specify.
- **Recursion limit 512** in `src/lib.rs` and `src/main.rs` (deeply nested view types in `post_game.rs`). Don't lower.
- **`#[server]` ordering** — server fns must be defined before the component that calls them (macro generates client stub that must be in scope).
- **Toast system + Skeleton-loading + Empty-state** patterns already established (Phase 5/UX). Continue using; design's notification anatomy is restyled-on-top.

## Summary

Phase 17 is a **visual port + IA restructure**, not a feature build. The Claude Design handoff at `/tmp/lol-design-handoff/lol-team-companion-app/project/` (15 source files totalling ~880 KB of JSX) covers ~10 of the 14 existing routes plus the new closed-beta landing; the remaining 8 utility surfaces (auth, team/roster, team-builder, opponents, action-items, personal-learnings, analytics, admin/invites) have **no Claude Design coverage** and must be filled with Open-Design HTML prototypes per D-13. The handoff is a self-contained React/Babel prototype using `<script type="text/babel">` — no build step, just CSS-custom-property-driven theming via `[data-theme="demacia"]` and `[data-theme="pandemonium"]` on `<html>`. **Components in this prototype are inline-styled, not Tailwind**, so the port becomes a full re-translate from `var(--surface)` etc. → `bg-surface` etc.

Three structural changes carry the most risk: (1) **theme port** — merging the design's `themes.css` into `input.css` `@theme` block, mapping `--accent`, `--surface`, `--line-soft` etc. onto the existing `--t-*` aliases, and dealing with the procedural `.canvas-grain` utility; (2) **font self-hosting** — five families, ~14 weight/style combinations, each requiring a `.woff2` file in `public/fonts/` (G-01 violation in the design CDN line MUST be fixed); (3) **4-hub IA restructure** — current `src/app.rs` has 19 flat routes and `src/components/nav.rs` is 510 lines built around them; UI-SPEC has already locked which routes belong to which hub but the planner must decide nested vs. flat URL paths.

**Primary recommendation:** Sequence the work as **foundations-first → hubs in parallel → closed-beta last**:
1. Plan A — Foundations + theme port + font self-host + nav restructure shell (blocks everything).
2. Plans B/C/D/E — One plan per hub (Strategy / History / Profile / hub-shared utility surfaces). Heavy hubs (Strategy contains draft + tree-drafter + champion-pool + game-plan + post-game) may need additional sub-plans to stay under the per-page atomic-commit budget.
3. Plan F — Open-Design seeding + utility surface ports.
4. Plan G — Closed-beta landing + acceptance form (last, because it depends on FLUX assets being generated externally and on the theme system being stable).

The per-page review gate (D-10) means the agent runs through ~22 review-commit cycles total. Sequencing to keep blocking dependencies inside Plan A means Plans B–E can run on independent waves.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Theme tokens (CSS custom props) | Static CSS (`input.css`) | Browser (CSS) | Pure design tokens; no runtime branching |
| Theme persistence | API (server fn writes `user.theme`) | Database (`user` table field) | Survives hard nav per D-06; mirrors `mode` precedent |
| Theme application (data-theme attr) | Frontend Server (SSR shell) | Browser (toggle handler) | SSR sets initial attr from session user; client toggles update DOM + persist |
| Font loading | Browser (CDN-style request to `/fonts/*.woff2`) | CDN/Static (Axum static serving via cargo-leptos `assets-dir`) | Self-hosted; `font-display: swap` for FOIT avoidance |
| Nav rendering (4-hub primary + sub-nav) | Browser (Leptos component) | Frontend Server (SSR) | `<Nav />` already renders top-level; sub-nav added per active hub |
| Routing (hub grouping) | Frontend Server (Leptos router) | Browser (client-side) | Decision deferred to planner — flat or nested |
| Champion picker (grid + autocomplete) | Browser (Leptos components) | API (via `champion_pool` server fn) | Existing components restyled; APIs unchanged |
| Draft board interactions | Browser (Leptos signals + WASM) | API (server fn save) | Existing `draft_board.rs` patterns preserved per D-07 |
| Tree graph (SVG render + edit) | Browser (SVG via Leptos view!) | API (debounced auto-save) | Existing children_of HashMap traversal preserved (CLAUDE.md rule 41) |
| FLUX background generation | External (one-off CLI/API call) | CDN/Static (committed `public/img/*.jpg`) | Out-of-band per D-20; not embedded in runtime |
| Closed-beta landing render | Frontend Server (SSR) | Browser | First surface visitors see; SSR for fast-paint hero |
| Invite token validation | API (server fn) | Frontend Server (route guard) | Phase 19.1 owns logic; this phase pre-stages visual surfaces |
| Bug-report widget visual placement | Browser (Leptos component) | — | Phase 18 owns behavior; this phase only stubs the visual position |

---

## Standard Stack

### Core (already in repo, no new deps expected)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| leptos | 0.8 | Reactive Rust UI framework, SSR + WASM hydration | Project foundation; routing, signals, server fns |
| leptos_router | 0.8 | Routing with `<Route>`, `<ParentRoute>`, nested `<Outlet>` | Standard Leptos pattern; supports flat or nested hub grouping |
| axum | 0.8 | Async server framework | Already wired with auth, sessions, static files |
| tower-http | 0.6 | Static file serving | Used for `public/` assets (fonts, images) via cargo-leptos `assets-dir` |
| surrealdb | 3.x | DB | `theme` field added to `user` table per D-06 |
| Tailwind CSS | v4 (standalone binary) | Utility classes + `@theme` block | Already wired via `tailwind-input-file = "input.css"` in Cargo.toml `[package.metadata.leptos]` |

`[VERIFIED: Cargo.toml]` All required dependencies are present. **No new Rust crates expected.** FLUX runtime is out-of-band (CLI/API call), not embedded in the binary.

### Tooling (external, project-level)

| Tool | Purpose | Notes |
|------|---------|-------|
| google-webfonts-helper | Fast download of `.woff2` files for self-hosting | `[CITED: gwfh.mranftl.com/fonts/{family}?subsets=latin]` Provides direct download links + ready-to-paste `@font-face` CSS. The pragmatic source for D-08. |
| fal.ai or replicate.com | FLUX.1 image generation API | `[VERIFIED: pricepertoken.com/image, getdeploying.com/fal-ai-vs-replicate]` See FLUX section below for cost comparison. |
| agent-browser (`.claude/skills/agent-browser/`) | Per-page screenshot for D-10 review gate | Already installed; documented in CLAUDE.md "Browser verification" |
| Playwright (`e2e/`) | Visual regression / smoke after restyle | Existing suite at `e2e/tests/pages.spec.ts`; can add visual-snapshot tests |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| google-webfonts-helper for font download | Direct download from `fonts.google.com` | gwfh provides woff2 + paste-ready CSS in one step; saving ~30 minutes of manual `@font-face` authoring |
| `fal.ai` for FLUX | `replicate.com` | fal.ai ~$0.05/image vs Replicate ~$0.055/image for FLUX.1 Pro; fal.ai is "consistently 30-50% cheaper" with sub-second latency (`[CITED: teamday.ai/blog/fal-ai-vs-replicate-comparison]`). Replicate's per-second GPU billing wins for batch jobs but our ~3-image budget makes fal.ai's flat per-image pricing more predictable. |
| `fal.ai` for FLUX | Self-host FLUX.1-dev on H100 (RunPod / vast.ai) | Self-host is cheapest at scale (~$0.005/image after warmup) but adds setup complexity for a one-off ~10-image generation pass. **Not worth it for v1.3.** |
| Nested routes (`/strategy/draft`) | Flat routes (`/draft`) | UI-SPEC defaults to **flat paths to preserve existing bookmarks** (UI-SPEC line 178-180). Planner can override only with strong reason; no compelling one identified. |

**Installation (none required for build):** All tooling is external (font download, FLUX API). The Rust crate dependencies stay unchanged.

---

## Architecture Patterns

### System Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│  External (one-off, pre-deploy)                                       │
│   ┌────────────────────┐                                             │
│   │ FLUX.1 (fal.ai)    │ ──prompts→ painterly bg images              │
│   └────────────────────┘                                             │
│            │                                                         │
│            ▼ committed JPEG (.planning/assets/AI-IMAGES.md tracks)   │
│      public/img/beta-landing-{demacia,pandemonium}.jpg               │
│      public/img/auth-bg-demacia.jpg (optional)                       │
│                                                                      │
│   ┌────────────────────┐                                             │
│   │ google-webfonts-   │ ──download→ .woff2 files                    │
│   │   helper           │                                             │
│   └────────────────────┘                                             │
│            │                                                         │
│            ▼                                                         │
│      public/fonts/{cinzel,cormorant-garamond,inter,                  │
│        jetbrains-mono,vt323}/*.woff2                                 │
└──────────────────────────────────────────────────────────────────────┘
                          │
                          ▼ (cargo-leptos copies public/ → target/site/)
┌──────────────────────────────────────────────────────────────────────┐
│  Server (Axum + Leptos SSR)                                           │
│                                                                      │
│   src/main.rs ─────────────► AppState{db, auth_backend}              │
│      │                                                               │
│      ├─ tower-http ─► serve target/site/* (incl. fonts/, img/)       │
│      │                                                               │
│      ├─ AuthSession layer ─► current_user.theme (read from DB)       │
│      │                                                               │
│      └─ leptos_routes_with_context                                   │
│             │                                                        │
│             ▼                                                        │
│         src/app.rs::shell()                                          │
│             │                                                        │
│             ├─ <html data-theme={user.theme | "demacia"}>            │
│             │                                                        │
│             └─ <App>                                                 │
│                  ├─ <Nav> (4 hubs + sub-nav based on route)          │
│                  ├─ <Routes>                                         │
│                  │    ├─ Strategy hub: /draft, /tree-drafter,        │
│                  │    │   /champion-pool, /game-plan, /post-game,    │
│                  │    │   /opponents, /action-items                  │
│                  │    ├─ History hub: /stats, /match/:id,            │
│                  │    │   /personal-learnings, /analytics            │
│                  │    ├─ Profile hub: /profile, /team/dashboard,     │
│                  │    │   /team/roster, /team-builder, /solo         │
│                  │    └─ Public: /, /closed-beta, /auth/{login,      │
│                  │         register}, /admin/invites,                │
│                  │         /legal/{impressum,datenschutz}            │
│                  └─ <BugReportWidget /> (auth-only, fixed btm-right) │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
                          │
                          ▼ (HTTP)
┌──────────────────────────────────────────────────────────────────────┐
│  Browser (WASM hydrate)                                               │
│                                                                      │
│   ┌─ on load: data-theme already set (no FOUC)                       │
│   ├─ ThemeToggle click → set_user_theme(...) server fn               │
│   │                       └─► document.documentElement.set-attr      │
│   │                       └─► localStorage (for offline cache)       │
│   ├─ Per-page interactions (existing patterns preserved)             │
│   │   - draft_board.rs: highlight-first slot deletion                │
│   │   - tree_graph.rs: children_of HashMap DFS                       │
│   │   - champion_autocomplete.rs: keyboard nav, on:select            │
│   │   - debounced auto-save w/ cancellable timer (.claude/rules/     │
│   │     wasm-patterns.md rule 42)                                    │
│   └─ Bug-report widget (Phase 18 wires behavior)                     │
└──────────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure (additions only — existing tree preserved)

```
public/                       # NEW — Cargo.toml already references assets-dir = "public"
├── fonts/
│   ├── cinzel/               # 4 weights × normal = 4 woff2
│   │   ├── cinzel-400.woff2
│   │   ├── cinzel-500.woff2
│   │   ├── cinzel-600.woff2
│   │   └── cinzel-700.woff2
│   ├── cormorant-garamond/   # 4 weights × {normal, italic} = up to 8 woff2 (italic only on 400-600 per UI-SPEC)
│   │   └── ...
│   ├── inter/                # 4 weights = 4 woff2
│   │   └── ...
│   ├── jetbrains-mono/       # 4 weights = 4 woff2
│   │   └── ...
│   └── vt323/                # 1 weight = 1 woff2
│       └── vt323-400.woff2
└── img/
    ├── beta-landing-demacia.jpg
    ├── beta-landing-pandemonium.jpg
    └── auth-bg-demacia.jpg   # optional per UI-SPEC

.planning/assets/             # NEW
└── AI-IMAGES.md              # prompt + seed + model + compute path per asset

src/components/
└── ornaments.rs              # NEW — HeraldicDivider, GiltCorner, FleurDeLis, RiotTape, CompanionSigil
                              #       (per UI-SPEC "Ornament Library")
```

`[VERIFIED: Cargo.toml]` `assets-dir = "public"` is already set, but the `public/` directory does NOT yet exist in the repo. Plan A must create it.

`[CITED: github.com/leptos-rs/cargo-leptos]` cargo-leptos copies whatever is in `assets-dir` to `target/site/`, where Axum serves it via tower-http. Files at `public/fonts/foo.woff2` become available at `/fonts/foo.woff2` at runtime.

### Pattern 1: Theme port — merge themes.css into input.css

**What:** The design's `themes.css` defines all tokens via CSS custom properties on `[data-theme="demacia"]` / `[data-theme="pandemonium"]`. Tailwind v4's `@theme` block in `input.css` reads from `var(--t-*)` aliases. We merge by mapping the design tokens onto the `--t-*` aliases per-theme.

**When to use:** Once at the start of Plan A, before any page restyle.

**Example:**
```css
/* input.css — after @import "tailwindcss"; */

/* G-01: NO @import url("https://fonts.googleapis.com/...") here. */
/* Self-host via @font-face (see Pattern 2). */

@font-face {
  font-family: "Cinzel";
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url("/fonts/cinzel/cinzel-400.woff2") format("woff2");
}
/* ... repeat for every (family, weight, style) combo ... */

@theme {
  /* Existing semantic aliases preserved */
  --color-base: var(--t-base);
  --color-surface: var(--t-surface);
  /* ... */

  /* Map font-family utilities to design's stack */
  --font-display: "Cormorant Garamond", "Cinzel", serif;
  --font-imperial: "Cinzel", "Trajan Pro", serif;
  --font-ui: "Inter", system-ui, sans-serif;
  --font-mono: "JetBrains Mono", monospace;
  --font-glitch: "VT323", "JetBrains Mono", monospace;
}

/* Default theme = demacia (replaces existing :root dark theme) */
:root,
[data-theme="demacia"] {
  --t-base: #0d0f1a;
  --t-surface: #14182a;
  --t-elevated: #1c2238;
  --t-primary: #f6efd9;        /* maps to --text-strong */
  --t-secondary: #e0d4b2;       /* --text */
  --t-muted: #a89773;           /* --text-soft */
  --t-dimmed: #6e6446;          /* --text-faint */
  --t-divider: rgba(212, 175, 90, 0.14);
  --t-outline: rgba(212, 175, 90, 0.32);
  --t-accent: #d4af5a;
  --t-accent-hover: #f1d985;
  --t-accent-contrast: #0d0f1a;

  /* Demacia-only extended tokens (used in component class lookups) */
  --gold-1: #f1d985;
  --gold-2: #d4af5a;
  --gold-3: #9b7c34;
  --lapis-1: #3a5fa8;
  --ivory: #f6efd9;
  --ink: #2b1c14;
  --success: #7ba35e;
  --warning: #d4974a;
  --danger: #a84436;
  --info: #6b8eb8;
}

[data-theme="pandemonium"] {
  --t-base: #06070b;
  --t-surface: #0e1018;
  --t-elevated: #14171f;
  --t-primary: #f3f0e8;
  --t-secondary: #c8c4ba;
  --t-muted: #7c7870;
  --t-dimmed: #4d4a44;
  --t-divider: rgba(255, 255, 255, 0.08);
  --t-outline: rgba(120, 220, 240, 0.32);
  --t-accent: #f73c8c;
  --t-accent-hover: #ff7ab8;
  --t-accent-contrast: #06070b;

  --accent-2: #6cf0e2;          /* decorative only — NOT for CTAs/focus rings */
  --accent-3: #fff157;          /* riot tape yellow — decorative only */
  --success: #6cf0a0;
  --warning: #fff157;
  --danger: #ff5560;
  --info: #6cf0e2;
}

/* canvas-grain utility — port from themes.css verbatim */
.canvas-grain { position: relative; }
.canvas-grain::before {
  content: ""; position: absolute; inset: 0; pointer-events: none;
  background:
    radial-gradient(ellipse at 30% 10%, rgba(255, 220, 160, 0.06), transparent 60%),
    radial-gradient(ellipse at 70% 90%, rgba(0, 0, 0, 0.25), transparent 50%);
  mix-blend-mode: overlay; z-index: 0;
}
.canvas-grain::after {
  content: ""; position: absolute; inset: 0; pointer-events: none; opacity: 0.5;
  background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='160' height='160'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' seed='3'/><feColorMatrix values='0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 0.18 0'/></filter><rect width='100%' height='100%' filter='url(%23n)'/></svg>");
  mix-blend-mode: overlay; z-index: 0;
}
.canvas-grain > * { position: relative; z-index: 1; }
[data-theme="demacia"].canvas-grain::after { /* demacia-tinted grain — copy from themes.css */ }
[data-theme="pandemonium"].canvas-grain::before { /* pandemonium-tinted gradients */ }
```

**Source:** `[VERIFIED: /tmp/lol-design-handoff/lol-team-companion-app/project/themes.css lines 60-100]` (demacia tokens), `[VERIFIED: same file lines 102-131]` (pandemonium tokens), `[VERIFIED: same file lines 133-159]` (canvas-grain).

**Tailwind v4 confirms** `[CITED: tailwindcss.com/docs/theme]`: "Theme variables defined in the `--font-*` namespace determine all of the font-family utilities that exist in a project." Setting `--font-display` etc. in `@theme` automatically generates `font-display` utility classes.

### Pattern 2: SSR-set data-theme attribute (no FOUC)

**What:** Set `data-theme` on `<html>` in the server-rendered shell before hydration, reading from the session user's `theme` field. Fallback to `"demacia"` when unauthenticated.

**When to use:** Plan A nav-shell task. Replaces the existing localStorage-only inline script in `app.rs:38`.

**Example:**
```rust
// src/app.rs — shell()
pub fn shell(options: LeptosOptions) -> impl IntoView {
    let theme = use_context::<UserContext>()
        .and_then(|ctx| ctx.theme.clone())
        .unwrap_or_else(|| "demacia".to_string());

    view! {
        <!DOCTYPE html>
        <html lang="en" data-theme=theme>
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                // Old localStorage script DELETED — server-rendered attr is authoritative.
                // Optional: keep a tiny script that reads localStorage as a fallback for users
                // who toggle theme before the next page load races the SSR. Discuss in plan.
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body class="bg-base min-h-screen text-primary">
                <App />
            </body>
        </html>
    }
}
```

**Caveat:** `use_context::<UserContext>()` requires plumbing the user's theme through `leptos_routes_with_context` in `main.rs`. Need to extract auth + DB read of `user.theme` BEFORE the route handler runs. Server fn for setting theme is straightforward — mirror `set_user_mode` in `nav.rs:10-24`.

**Source:** `[VERIFIED: src/components/nav.rs:10-24]` (set_user_mode pattern); `[VERIFIED: src/app.rs:38]` (existing localStorage script — to be replaced).

### Pattern 3: 4-Hub nav with sub-nav (flat URLs)

**What:** Top nav shows 4 primary hubs (Strategy / Live / History / Profile) plus 3 secondary buttons (Draft / Tree / Pool). Below the top nav, a sub-nav strip surfaces the active hub's children with the active route highlighted.

**When to use:** Plan A nav-restructure task.

**Recommended approach (flat URLs):**
- Keep `<Routes>` flat in `app.rs` (no `<ParentRoute>` for hubs). All routes stay at their current paths per UI-SPEC default.
- A helper function `current_hub() -> &'static str` derives from `use_location()` — maps each path to its hub. Returns `"strategy" | "live" | "history" | "profile" | ""` for public pages.
- `<Nav>` reads `current_hub` reactively and renders the matching `<SubNav>`.
- The active sub-nav button uses the same accent-soft styling as the primary hub button.

**Why flat over nested:** UI-SPEC line 178-180 explicitly chooses flat to preserve bookmarks. Nested routes via `<ParentRoute>` + `<Outlet>` would force route-path renames OR a "shadow" layout component, both of which add complexity for no UX gain. `[CITED: book.leptos.dev/router/17_nested_routing.html]` confirms `<ParentRoute>`/`<Outlet>` is the supported nested pattern but is not required for shared layout — `<Nav>` already wraps `<Routes>` in `app.rs:60-83`.

**Example:**
```rust
// src/components/nav.rs (sketch)
const HUB_ROUTES: &[(&str, &[(&str, &str)])] = &[
    ("strategy", &[
        ("/draft", "Draft"),
        ("/tree-drafter", "Tree"),
        ("/champion-pool", "Pool"),
        ("/game-plan", "Game plan"),
        ("/post-game", "Post-game"),
        ("/opponents", "Opponents"),
        ("/action-items", "Action items"),
    ]),
    ("history", &[
        ("/stats", "Stats"),
        ("/personal-learnings", "Learnings"),
        ("/analytics", "Analytics"),
    ]),
    ("profile", &[
        ("/profile", "Profile"),
        ("/team/dashboard", "Team"),
        ("/team/roster", "Roster"),
        ("/team-builder", "Team builder"),
        ("/solo", "Solo dashboard"),
    ]),
];

fn current_hub(path: &str) -> &'static str {
    if HUB_ROUTES.iter().find(|(_, routes)| {
        routes.iter().any(|(p, _)| path.starts_with(p))
    }).map(|(h, _)| *h).unwrap_or("")
    // ...special-case "/match/:id" → "history", "/personal-learnings/new" → "history"
}
```

`[CITED: book.leptos.dev/router/17_nested_routing.html]` — Leptos 0.8 nested routes via `<ParentRoute>` + `<Outlet>` work but require child-route paths to live under parent's path prefix. Flat is cleaner here.

### Pattern 4: Open-Design seeding via DESIGN.md

**What:** Create `/home/jasper/Repositories/open-design/design-systems/lol-companion/DESIGN.md` with the demacia/pandemonium tokens, font assignments, and core component patterns. Open-Design uses this as the source of truth when generating utility-surface HTML.

**When to use:** Plan F (utility surfaces) — must run before any utility-surface prototype is requested.

**Format (per Open-Design conventions):** `[ASSUMED]` Open-Design's exact DESIGN.md schema is project-internal — agent should read `/home/jasper/Repositories/open-design/AGENTS.md` and an existing `design-systems/*/DESIGN.md` (if any) before authoring. The seed must include at minimum:
- Token definitions (CSS custom properties matching `input.css`)
- Font family assignments (which family for headings/body/mono/etc.)
- Component patterns: card variants (gilt, plain, elevated), button variants (primary, ghost, destructive), input fields, badges
- Icon library reference (the inline SVG paths from `components.jsx`'s `Icon` component)

This claim is **assumed** because we have not inspected an existing `design-systems/*/DESIGN.md`. The plan must include a "read Open-Design AGENTS.md" sub-task before authoring.

### Pattern 5: Per-page atomic-commit cadence

**What:** D-10 mandates per-page review-commit. Each task in a hub plan is "restyle page X". The flow:
1. Implement page X (read design source, port to Leptos, use semantic tokens).
2. `cargo leptos watch` already running (CLAUDE.md dev workflow).
3. agent-browser screenshot the route.
4. Compare with design source mentally.
5. User approves OR requests revision.
6. Atomic commit (one page, one commit).
7. Move to next.

**When to use:** Every hub plan (B/C/D/E/G).

**Why this is critical:** Catches drift early. The Claude Design prototype is inline-styled React and the Leptos port is class-based Tailwind — direct visual diff against the prototype is the only reliable way to verify pixel parity.

### Anti-Patterns to Avoid

- **Batch implementing multiple pages then reviewing at the end** — D-10 explicitly forbids; experience from prior phases shows visual drift compounds.
- **Re-specifying tokens in 17-UI-SPEC.md** — already done correctly (UI-SPEC scope honored). Don't re-add to RESEARCH.md or PLAN.md.
- **Hand-rolling SVG icons inside every page** — the design's `Icon` component (`components.jsx:108-145`) has a path set; either inline once into a shared `<Icon>` Leptos component or use the existing `tree_graph.rs` SVG inline pattern. Don't duplicate.
- **Using the design's React inline-style approach in Leptos** — D-07 says "components stay in idiomatic Leptos style". Translate `style={{ background: "var(--surface)" }}` → `class="bg-surface"`.
- **Touching the BFD linker config** — `.cargo/config.toml` `link-arg=-fuse-ld=bfd` is a workaround for nightly's lld crash. Leave alone.
- **Lowering `recursion_limit = "512"`** — `post_game.rs` view types depend on it. CLAUDE.md rule 38.
- **Using `outline: none` without a `focus-visible:ring-*` companion** — G-12. UI-SPEC enforces; planner must add to per-task acceptance criteria.
- **Adding raw hex colors in components** — semantic tokens only. CLAUDE.md "Code Style".
- **Running integration tests during this phase** — they OOM with BFD linker. Use `cargo test --features ssr --lib` only.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Theme system with two regions | A toggleable JS state machine that mutates style props | CSS custom properties on `[data-theme="..."]` selector + `@theme` block | Already the design's pattern; works server-side without JS; survives hard nav. |
| Fonts | Linking Google Fonts CDN | Self-host via `@font-face` in `input.css` | G-01 hard NO; deployed HTML must not contain `fonts.googleapis.com`. |
| FLUX background generation | A Rust crate that calls FLUX | One-off CLI call to fal.ai or replicate, commit JPEG to `public/img/` | D-20: assets versioned in git; runtime is read-only. |
| Per-page screenshot diff | Custom Playwright pixel-diff config | agent-browser skill (`.claude/skills/agent-browser/`) for D-10; Playwright `expect(locator).toHaveScreenshot()` for regression-only | agent-browser already integrated; Playwright already in `e2e/` for regression. |
| Inline SVG icon paths repeated 50× | Per-page `<svg>...</svg>` blocks | Shared `<Icon name="shield" />` component mirroring `components.jsx:108-145` | Maintainability; single source of truth for icon paths. |
| 4-hub nav state derivation | Storing `current_hub` in a global signal | `use_location()` + a pure mapping function | Routing already provides location; no need for a redundant signal. |
| Theme toggle persistence dance | localStorage + onload script + reactive hook | Server fn `set_user_theme(theme: String)` mirrored on `set_user_mode` (`nav.rs:10-24`) | D-06 mandates DB-backed; localStorage was the OLD pattern (replaced). |
| Charting / sparkline | Bringing in a JS chart library | Inline SVG (already used in `tree_graph.rs`, `lp_history_graph.rs`) | Project pattern; consistent with existing visual language. |
| Champion picker grid | A generic grid component | Restyle `src/components/champion_picker.rs` (existing) | API stays; only Tailwind classes change. |
| Draft slot interactions | Build a new draft-board state machine | Restyle existing `draft_board.rs` (already has highlight-first deletion + `on_slot_clear` callback per Phase 12 patterns) | Logic is solid; only visuals change. |
| Tree graph traversal | Reverse-child_ids heuristic | `children_of: HashMap<String, Vec<String>>` recursive DFS (already in `db.rs::get_tree_nodes`, CLAUDE.md rule 41) | Heuristic was buggy; HashMap pattern is the correct fix from Phase 12. |

**Key insight:** This phase is largely a re-skin, not a re-build. Existing logic patterns (debounced auto-save, highlight-first slot deletion, children_of HashMap, page-protection Resource template, toast system, skeleton loaders) are all preserved. The risk surface is **visual port fidelity**, not behavioral regression.

---

## Common Pitfalls

### Pitfall 1: G-01 violation re-introduced via copy-paste from themes.css

**What goes wrong:** Implementer copies `themes.css` content into `input.css` and accidentally keeps the `@import url("https://fonts.googleapis.com/css2?...")` line at the top.

**Why it happens:** That import is line 7 of `themes.css` — easy to grab without noticing.

**How to avoid:** Per-page-commit pre-flight grep:
```bash
grep -n "fonts.googleapis\|fonts.gstatic" input.css
# must return 0 hits
```
Add this as the **last item in Plan A's foundations checklist** and as a CI sweep item for Phase 21.

**Warning signs:** Network tab shows requests to `fonts.googleapis.com` or `fonts.gstatic.com` on page load.

**Source:** `[VERIFIED: /tmp/lol-design-handoff/lol-team-companion-app/project/themes.css line 7]`.

### Pitfall 2: Flash of unstyled content (FOUC) on theme switch

**What goes wrong:** User toggles theme; for a frame the page shows base content with no theme attr (or wrong theme), then the new theme paints.

**Why it happens:** If `data-theme` is set client-side after hydration, there's a window where the SSR HTML and the client hydrated state disagree.

**How to avoid:**
1. SSR sets `data-theme` on `<html>` from the session user's theme (Pattern 2 above).
2. Theme toggle handler updates `data-theme` synchronously *before* awaiting the server fn — optimistic UI.
3. Server fn writes to DB; on success, also write to localStorage as a defensive cache so unauthenticated future visits get the right initial theme too.

**Warning signs:** Visible color flicker on toggle. Visible color flicker on hard nav between pages.

### Pitfall 3: WASM panic in event handlers (the existing rule 35 trap)

**What goes wrong:** A `.unwrap()` on `web_sys::window()` or `document_element()` in the new theme toggle / nav code panics in WASM, freezing the entire runtime.

**Why it happens:** `theme_toggle.rs:42-50` already has this pattern correctly handled with `if let Some(...)`, but rewriting the file might lose the discipline.

**How to avoid:** Mandatory `.claude/rules/wasm-patterns.md` rule 35 — never `.unwrap()` in event handlers. Always `if let Some(window) = web_sys::window() { ... }`.

**Warning signs:** Toggle works once, then the entire page becomes unresponsive (all interactions frozen).

### Pitfall 4: Sub-nav active-state false negative on partial path matches

**What goes wrong:** Sub-nav for `/match/12345` shows no active hub because string-equality match fails.

**Why it happens:** Naive `path == "/match"` doesn't handle `:id` parameter routes.

**How to avoid:** Use `starts_with("/match/")` for variable routes; explicit special-cases for `/personal-learnings/new` and similar nested paths.

**Warning signs:** Active state breaks on any page with route params.

### Pitfall 5: Theme migration leaves orphan `[data-accent]` selectors in input.css

**What goes wrong:** D-04 retires the 5-accent palette but the `[data-accent="blue"]`/`[data-accent="purple"]`/etc. blocks in `input.css:74-93` remain dead.

**Why it happens:** The migration task focuses on adding the new tokens and forgets to delete the old ones.

**How to avoid:** Plan A includes a "delete old accent blocks" task with a grep-verification:
```bash
grep -n 'data-accent\b' input.css src/components/theme_toggle.rs src/app.rs
# must return 0 hits after migration (except in comments documenting the migration)
```

**Warning signs:** Old `data-accent` state in localStorage from prior dev session causes weird color overrides on first load post-migration.

### Pitfall 6: Font weight/style omitted, browser falls back to system serif

**What goes wrong:** `font-display: italic` for Cormorant Garamond 600 italic isn't included in `@font-face` declarations — browser falls back to system Times Italic, which looks wrong.

**Why it happens:** UI-SPEC says Cormorant Garamond italic 400/500/600 are needed. If implementer ships only normal weights, italic styles silently degrade.

**How to avoid:** Plan A's font-self-host task enumerates each `(family, weight, style)` combo from `themes.css` and `foundations.jsx` actual usage. Final list (per UI-SPEC line 117-122):
- Cormorant Garamond: 400, 400i, 500, 500i, 600, 600i (italic on 400/500/600)
- Cinzel: 400, 500, 600, 700 (normal only)
- Inter: 400, 500, 600, 700 (normal only)
- JetBrains Mono: 400, 500, 600, 700 (normal only)
- VT323: 400 (only weight that exists)

Total `@font-face` blocks: ~20.

**Warning signs:** Page renders Times Italic where Cormorant Italic was intended. DevTools Network tab shows fewer than 20 woff2 requests.

### Pitfall 7: cargo-leptos doesn't auto-create public/

**What goes wrong:** `public/fonts/` doesn't exist; `cp` to it during deploy silently creates a regular file `public` with the font name appended.

**Why it happens:** `Cargo.toml` `assets-dir = "public"` references it but doesn't create it.

**How to avoid:** Plan A includes a literal `mkdir -p public/fonts/{cinzel,cormorant-garamond,inter,jetbrains-mono,vt323} public/img` step before any download.

**Warning signs:** Browser 404s on `/fonts/cinzel/cinzel-400.woff2`.

### Pitfall 8: Per-page review gate stalled on user availability

**What goes wrong:** D-10 blocks until user approves each page. If the user is unavailable for a stretch, the implementation agent stalls 22+ times.

**Why it happens:** D-10 is non-negotiable, but the per-page review pace might exceed user availability.

**How to avoid:** Discuss in `/gsd-discuss-phase` whether D-10 strictly means "each page commits separately and pauses" or "each page commits separately and the user can batch-review N pages later". The current CONTEXT.md reading is "pauses each page". This research flags it.

**Warning signs:** Long idle blocks during execution; agent consumes context window waiting; phase takes calendar weeks instead of days.

**Mitigation question for planner:** Consider a "review queue" pattern — implementer commits per page (atomic) but flags each commit with `[REVIEW-PENDING]` in the message; user reviews them in batches. This preserves D-10's atomicity without requiring real-time user presence.

### Pitfall 9: Tree graph SVG breaks on theme switch because tokens are inline-styled

**What goes wrong:** `tree_graph.rs` uses `style="stroke: var(--color-accent)"` for SVG attrs because Tailwind utilities don't reach SVG `stroke` attribute. Theme switch updates the CSS variable but the SVG might not re-render reactively.

**Why it happens:** SSR pre-renders the SVG with the demacia stroke; client doesn't re-paint it on toggle because Leptos doesn't track plain CSS variable changes.

**How to avoid:** Use `class="stroke-accent"` (Tailwind utility) where supported; use `style="stroke: var(--color-accent)"` for inline SVG and trust that browsers DO re-paint when the CSS variable changes (they do — CSS variables ARE reactive at the rendering layer; Leptos doesn't need to track them).

**Verification:** UI-SPEC line 720-724 says: `class="fill-current"` with parent `text-accent` where utilities work; `style="stroke: var(--color-accent)"` otherwise. This is correct.

**Warning signs:** Tree graph stays demacia-gold after switching to pandemonium-pink. (Probably won't happen — CSS variables ARE re-evaluated by the browser on attr change.)

### Pitfall 10: Per-page review gate (D-10) doesn't catch behavioral regressions

**What goes wrong:** Restyling visually looks correct but breaks `on_slot_clear` callback or debounced auto-save Effect.

**Why it happens:** Visual review focuses on the rendered output; runtime bugs (signal lifecycle, stale captures, missing teardown — the "reactive bugs" CLAUDE.md warns about) aren't visible in a screenshot.

**How to avoid:** Each per-page commit also runs:
1. `cargo check --features ssr` — type check.
2. `cargo check --features hydrate --target wasm32-unknown-unknown` — WASM compile.
3. The matching e2e audit test (`audit-{draft,tree-drafter,champion-pool,game-plan,post-game,team,misc-pages}.spec.ts`) — these already exist and target the heavy pages.

**Warning signs:** A draft slot that "looks right" but doesn't clear on × click. A tree node that auto-saves to the wrong tree.

---

## Code Examples

### Theme toggle (replaces existing 5-accent picker)

```rust
// src/components/theme_toggle.rs (after Plan A rewrite)

use leptos::prelude::*;

#[server]
pub async fn set_user_theme(theme: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    if theme != "demacia" && theme != "pandemonium" {
        return Err(ServerFnError::new("Invalid theme"));
    }
    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;
    db::set_user_theme(&db, &user.id, &theme)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[component]
pub fn ThemeToggle(initial_theme: String) -> impl IntoView {
    let theme = RwSignal::new(initial_theme);

    let toggle = move |new_theme: &'static str| {
        if theme.get_untracked() == new_theme { return; }
        theme.set(new_theme.to_string());

        // Optimistic DOM update — no FOUC
        #[cfg(feature = "hydrate")]
        {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(root) = doc.document_element() {
                    let _ = root.set_attribute("data-theme", new_theme);
                }
            }
        }

        // Persist to DB
        leptos::task::spawn_local(async move {
            let _ = set_user_theme(new_theme.to_string()).await;
        });
    };

    view! {
        <div class="inline-flex p-0.5 bg-surface border border-divider rounded-full">
            <button
                class=move || {
                    if theme.get() == "demacia" {
                        "px-4 py-1.5 rounded-full bg-accent text-accent-contrast font-imperial \
                         text-[10px] uppercase tracking-[0.18em] font-semibold cursor-pointer"
                    } else {
                        "px-4 py-1.5 rounded-full text-muted hover:text-secondary font-imperial \
                         text-[10px] uppercase tracking-[0.18em] font-semibold cursor-pointer"
                    }
                }
                on:click=move |_| toggle("demacia")
            >"Demacia"</button>
            <button
                class=move || {
                    if theme.get() == "pandemonium" {
                        "px-4 py-1.5 rounded-full bg-accent text-accent-contrast font-mono \
                         text-[10px] uppercase tracking-[0.16em] font-semibold cursor-pointer"
                    } else {
                        "px-4 py-1.5 rounded-full text-muted hover:text-secondary font-mono \
                         text-[10px] uppercase tracking-[0.16em] font-semibold cursor-pointer"
                    }
                }
                on:click=move |_| toggle("pandemonium")
            >"Pandemonium"</button>
        </div>
    }
}
```

**Source:** Existing `set_user_mode` pattern from `src/components/nav.rs:10-24`. WASM safety from `.claude/rules/wasm-patterns.md` rule 35.

### DB schema migration

```surql
-- schema.surql — additions for D-06

DEFINE FIELD IF NOT EXISTS theme ON user TYPE string DEFAULT 'demacia'
  ASSERT $value IN ['demacia', 'pandemonium'];
```

```rust
// src/server/db.rs — additions

pub async fn set_user_theme(db: &Surreal<Db>, user_id: &str, theme: &str) -> DbResult<()> {
    let key = strip_table_prefix(user_id, "user");
    db.query("UPDATE type::record('user', $key) SET theme = $theme")
        .bind(("key", key))
        .bind(("theme", theme.to_string()))
        .await?
        .check()?;
    Ok(())
}
```

`[VERIFIED: schema.surql lines 1-12]` `user` table is SCHEMAFULL; `mode` field already added with `DEFAULT 'solo'` — same pattern works for `theme`.

### Sub-nav derived from current path

```rust
// src/components/nav.rs (sketch — replaces flat link list)

use leptos_router::hooks::use_location;

#[component]
pub fn Nav() -> impl IntoView {
    let location = use_location();
    let active_hub = move || hub_for_path(&location.pathname.get());

    view! {
        <header class="sticky top-0 z-50 bg-surface/80 backdrop-blur-md border-b border-divider">
            <div class="px-16 py-4 grid grid-cols-[auto_1fr_auto_auto] gap-7 items-center">
                <CompanionSigil />
                <PrimaryHubButtons active_hub=Signal::derive(active_hub) />
                <MatchDayClock />
                <ThemeToggle initial_theme=...  />
            </div>
            // Sub-nav strip — only renders when a hub is active
            {move || {
                let hub = active_hub();
                if hub.is_empty() {
                    view! {}.into_any()
                } else {
                    view! { <SubNav hub=hub current_path=location.pathname.get() /> }.into_any()
                }
            }}
        </header>
    }
}

fn hub_for_path(path: &str) -> &'static str {
    if path.starts_with("/draft") || path.starts_with("/tree-drafter")
        || path.starts_with("/champion-pool") || path.starts_with("/game-plan")
        || path.starts_with("/post-game") || path.starts_with("/opponents")
        || path.starts_with("/action-items") {
        "strategy"
    } else if path.starts_with("/stats") || path.starts_with("/match/")
        || path.starts_with("/personal-learnings") || path.starts_with("/analytics") {
        "history"
    } else if path.starts_with("/profile") || path.starts_with("/team")
        || path.starts_with("/team-builder") || path.starts_with("/solo") {
        "profile"
    } else {
        ""
    }
}
```

**Source:** `[CITED: docs.rs/leptos_router]` `use_location()` returns a reactive location.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 5-accent palette (yellow/blue/purple/emerald/rose) | 2-region theme system (demacia/pandemonium) | Phase 17 (D-04) | Retire `[data-accent="*"]` blocks; theme_toggle becomes 2-state |
| Theme stored in localStorage | Theme stored on `user` record | Phase 17 (D-06) | Add `theme` field to `user` table; mirror Phase 12 `mode` precedent |
| Google Fonts CDN `@import` | Self-hosted woff2 in `public/fonts/` | Phase 17 (D-08) | G-01 compliance; no external font fetch in deployed HTML |
| 19-route flat nav | 4-hub primary nav + sub-nav | Phase 17 (D-09) | Major `nav.rs` refactor; `app.rs` route definitions unchanged (paths preserved) |
| Public registration | Closed-beta gate via URL invite token | Phase 19.1 | Phase 17 stages visual surfaces; Phase 19.1 wires logic |

**Deprecated/outdated:**
- The existing `<script>` tag in `app.rs:38` that reads localStorage for `theme` and `accent` — replaced by SSR-rendered `data-theme` attr (Pattern 2). Keep a minimal version OR delete entirely (planner decides).
- The `[data-accent="*"]` blocks in `input.css:74-93` — delete after migration (Pitfall 5).
- Existing `set_is_light` light-theme code in `theme_toggle.rs` — UI-SPEC line 86-87 leaves the light theme block "may be retired alongside [accent blocks] or kept for future use; decision deferred to executor." Recommend retire (light theme isn't in the design at all).

---

## Files Touched

Concrete file list with role classification (port = new content from design / restyle = visual change to existing logic / new = file did not exist before / delete = remove obsolete code).

### Configuration / build (Plan A: Foundations)

| File | Role | Change |
|------|------|--------|
| `input.css` | port + restyle | Add `@font-face` blocks (×~20); merge demacia/pandemonium token blocks; map design tokens onto `--t-*` aliases; port `.canvas-grain` utility; delete `[data-accent="*"]` blocks; optionally delete `[data-theme="light"]` |
| `schema.surql` | restyle | Add `DEFINE FIELD theme ON user TYPE string DEFAULT 'demacia'` |
| `Cargo.toml` | unchanged | `assets-dir = "public"` already correct |
| `.cargo/config.toml` | unchanged | BFD linker — don't touch |
| `public/` | new | Create directory tree; `public/fonts/{family}/` for woff2; `public/img/` for FLUX assets |
| `public/fonts/**/*.woff2` | new | Download via google-webfonts-helper (~14-20 woff2 files, ≤500 KB total) |
| `public/img/beta-landing-{demacia,pandemonium}.jpg` | new | Generate via FLUX.1; commit per D-20 |
| `public/img/auth-bg-demacia.jpg` (optional) | new | Optional per UI-SPEC perf budget |
| `.planning/assets/AI-IMAGES.md` | new | Reproducibility log (prompt + seed + model + compute path per asset) |

### Server / DB (Plan A: Foundations)

| File | Role | Change |
|------|------|--------|
| `src/server/db.rs` | restyle | Add `set_user_theme()` function (mirror `set_user_mode`); add `theme` to `Db*` user struct; add `get_user_theme()` if not auto-included in current_user fetch |
| `src/main.rs` | restyle | Plumb `user.theme` into context for SSR shell |
| `src/server/auth.rs` | unchanged (verify) | `AuthSession::user` already returns full user — `theme` field comes for free if struct is regenerated |

### Shell / nav (Plan A: Foundations)

| File | Role | Change |
|------|------|--------|
| `src/app.rs` | restyle | Replace localStorage script with SSR `data-theme={user.theme}`; route list unchanged (paths preserved per D-09); add `/closed-beta`, `/admin/invites`, `/legal/impressum`, `/legal/datenschutz` routes (visual stubs only) |
| `src/components/nav.rs` (510 lines) | restyle (heavy) | Replace 19-route flat nav with 4-hub primary + sub-nav; add `current_hub()` derivation from `use_location()`; preserve notifications + ModeToggle; replace ThemeToggle invocation |
| `src/components/theme_toggle.rs` (151 lines) | restyle (heavy) | Delete 5-accent picker; delete light/dark toggle; replace with 2-state demacia/pandemonium toggle; new `set_user_theme` server fn |
| `src/components/ornaments.rs` | new | `<HeraldicDivider />`, `<GiltCorner />`, `<FleurDeLis />`, `<RiotTape />`, `<CompanionSigil />`, `<Crown />` — inline SVG components matching `components.jsx` paths |
| `src/components/icon.rs` | new (recommended) | Shared `<Icon name="shield" size=18 />` component mirroring `components.jsx:108-145` icon path set; replaces ad-hoc SVG inline blocks |

### Hub: Strategy (Plan B)

| File | Role | Lines | Change |
|------|------|-------|--------|
| `src/pages/draft.rs` | restyle (heavy) | 3,801 | Apply Claude Design `draft-boards.jsx` War Table (Demacia) variant; preserve all existing logic |
| `src/components/draft_board.rs` | restyle | 562 | Match design's draft board variants; preserve highlight-first deletion + on_slot_clear callback |
| `src/pages/tree_drafter.rs` | restyle (heavy) | 1,610 | Apply Claude Design `tree-drafter.jsx`; preserve children_of HashMap traversal |
| `src/components/tree_graph.rs` | restyle | 709 | Apply 5-state node visuals (locked, selected, alternate, ghost, leaf); animated edge dash stroke; SVG attrs use `style="stroke: var(--color-accent)"` |
| `src/pages/champion_pool.rs` | restyle (heavy) | 1,356 | Apply Claude Design `champion-pool.jsx` tier list with deep-dive panel |
| `src/components/champion_picker.rs` | restyle | (unknown) | New tile sizing, search bar styling, banned/unavailable state |
| `src/pages/game_plan.rs` | restyle (heavy) | 1,515 | Apply game-plan screens from bundle (no specific source file — use `extra-variants.jsx` / `pandemonium-variants.jsx`) |
| `src/pages/post_game.rs` | restyle (heavy) | (unknown) | Apply post-game screens from bundle |
| `src/components/champion_autocomplete.rs` | restyle | (unknown) | New input styling, dropdown styling, keyboard nav preserved |
| `src/pages/opponents.rs` | restyle (utility tier — Open-Design) | (unknown) | Apply Open-Design utility prototype |
| `src/pages/action_items.rs` | restyle (utility tier — Open-Design) | (unknown) | Apply Open-Design utility prototype |

### Hub: History (Plan C)

| File | Role | Change |
|------|------|--------|
| `src/pages/stats.rs` | restyle (heavy) | Apply `screens/history.jsx` battle-log + folio panel |
| `src/pages/match_detail.rs` | restyle (heavy) | Apply `match-detail.jsx`; preserve timeline event marker logic |
| `src/pages/personal_learnings.rs` | restyle (utility — Open-Design) | Apply Open-Design utility prototype |
| `src/pages/analytics.rs` | restyle (utility — Open-Design) | Apply Open-Design utility prototype |

### Hub: Profile (Plan D)

| File | Role | Change |
|------|------|--------|
| `src/pages/profile.rs` | restyle (heavy) | Apply `screens/profile.jsx` (Captain's Folio) |
| `src/pages/team/dashboard.rs` | restyle (heavy) | Apply `team-dashboards.jsx` Strategy Room variant |
| `src/pages/team/roster.rs` | restyle (utility — Open-Design) | Apply Open-Design utility prototype |
| `src/pages/team_builder.rs` | restyle (utility — Open-Design) | Apply Open-Design utility prototype |
| `src/pages/solo_dashboard.rs` | restyle (heavy) | Apply `solo-dashboards.jsx` SoloConstellation variant |

### Hub: Public + Auth (Plan E or merged into Plan A shell)

| File | Role | Change |
|------|------|--------|
| `src/pages/home.rs` | restyle | Public landing OR redirect to `/closed-beta` per session state |
| `src/pages/auth/login.rs` | restyle (utility — Open-Design) | Apply Open-Design login prototype + optional FLUX bg |
| `src/pages/auth/register.rs` | restyle (utility — Open-Design) | Apply Open-Design register prototype with hidden invite-token field |
| `src/pages/closed_beta.rs` | new (Claude Design tier) | Hero landing with FLUX background + CompanionSigil + "The Strategy Room" copy per UI-SPEC §"Closed-Beta Surfaces" |
| `src/pages/admin/invites.rs` | new (utility — Open-Design) | Visual stub; Phase 19.1 wires logic |
| `src/pages/legal/impressum.rs` | new stub | Phase 21 fills content |
| `src/pages/legal/datenschutz.rs` | new stub | Phase 21 fills content |

### Closed-beta + bug-report (Plan G or merged)

| File | Role | Change |
|------|------|--------|
| `src/components/bug_report_widget.rs` | new (visual stub) | Floating button + tooltip + modal stub per UI-SPEC §"Bug-Report Widget Placement"; Phase 18 wires behavior |

### Open-Design seeding (Plan F)

| File | Role | Change |
|------|------|--------|
| `/home/jasper/Repositories/open-design/design-systems/lol-companion/DESIGN.md` | new | Token + font + component pattern seed |
| `/home/jasper/Repositories/open-design/design-systems/lol-companion/tokens.css` (or equivalent) | new | Token CSS file matching `input.css` |
| `/home/jasper/Repositories/open-design/.od/projects/{new-uuid}/...` | new | Open-Design project per utility surface group; produced by Open-Design tool, not authored by hand |

### E2E tests (Plan A foundations + per-hub plan additions)

| File | Role | Change |
|------|------|--------|
| `e2e/tests/pages.spec.ts` | restyle | Add `/solo`, `/closed-beta`, `/admin/invites`, `/legal/*`; update `AUTHED_PAGES` array |
| `e2e/tests/audit-*.spec.ts` | restyle | Update visual assertions to match new layouts; selectors that depend on text content stay if the copy is unchanged |
| `e2e/tests/visual-regression.spec.ts` | new (recommended) | Per-page visual snapshot suite (Playwright `toHaveScreenshot()`) — initial baseline captured at end of phase |

**Total file impact:** ~25 source files restyled, ~10 new files (components + new pages), ~20 new asset files (woff2 + img). Project skill rules (`.claude/rules/leptos-patterns.md`, `wasm-patterns.md`) all apply.

---

## Validation Architecture

> Phase 17 has measurable success criteria: visual fidelity (per-page review), G-01 compliance, G-12 compliance, no raw hex in components, `/gsd-ui-review` PASS on 6 pillars, no behavioral regression on existing pages. The validation strategy mixes per-task fast checks, per-page review-gate verification (D-10), and a final 6-pillar audit.

### Test Framework

| Property | Value |
|----------|-------|
| Framework (logic) | `cargo test --features ssr --lib` (unit + tests/) |
| Framework (UI) | Playwright (`cd e2e && npx playwright test`) — already wired |
| Framework (visual review) | agent-browser skill (`.claude/skills/agent-browser/`) — already wired |
| Framework (style guard) | grep-based CI sweep for G-01 / G-12 / raw-hex |
| Config files | `Cargo.toml`, `e2e/playwright.config.ts`, `tests/common/mod.rs`, `.cargo/config.toml` |
| Quick run command | `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown` |
| Per-task verify command | `just verify` if available, else `cargo check --features ssr && cargo test --features ssr --lib` |
| Full suite command | `just verify && just smoke && just e2e` (requires running dev server) |

### Phase Requirements → Test Map

> No formal REQ-IDs for Phase 17. Map ROADMAP success criteria + UI-SPEC enforcement to validation activities.

| Source | Behavior to validate | Test Type | Automated Command | File / Tool |
|--------|---------------------|-----------|-------------------|-------------|
| ROADMAP SC1 | `17-UI-SPEC.md` exists with route inventory | manual (artifact check) | `test -f .planning/phases/17-ui-consolidation/17-UI-SPEC.md` | shell |
| ROADMAP SC2 | Claude Design primary mockups produced for hero pages | per-page review (D-10) | agent-browser screenshot at each route + user approval | agent-browser skill |
| ROADMAP SC3 | Open-Design HTML for utility surfaces exists | artifact check | `find /home/jasper/Repositories/open-design/.od/projects -name "*.html" \| grep lol-companion` | find |
| ROADMAP SC4 | Implementation matches UI-SPEC | per-page review (D-10) + final `/gsd-ui-review` | agent-browser screenshot vs design source mental diff | agent-browser + UI-review |
| ROADMAP SC4 + 6 pillars | `/gsd-ui-review` PASS | retrospective audit | `/gsd-ui-review 17` | UI-review tool |
| SPEC §6 SC4 | No `outline:none` without ring replacement (G-12) | grep | `grep -nE "outline\s*:\s*none" src/ \| grep -v "focus-visible:ring"` | grep |
| SPEC §6 SC5 | No raw hex in components | grep | `grep -nE "#[0-9a-fA-F]{3,8}\b" src/components/ src/pages/ \| grep -v "//" \| grep -v 'text-white'` | grep |
| SPEC §6 SC6 | No Google Fonts CDN | grep | `grep -nE "fonts\.googleapis\|fonts\.gstatic" input.css src/` | grep |
| Behavioral regression | Each restyled page still loads, nav present, no console errors | smoke | `cd e2e && npx playwright test pages.spec.ts` | Playwright |
| Behavioral regression — heavy pages | Draft, tree-drafter, champion-pool, game-plan, post-game still functional | audit | `cd e2e && npx playwright test audit-*.spec.ts` | Playwright |
| Visual regression baseline | New layouts captured for ongoing diff detection | visual snapshot | `cd e2e && npx playwright test visual-regression.spec.ts` | Playwright `toHaveScreenshot()` |
| Theme persistence | Theme set on user A persists across hard nav and login/logout | e2e | new test in `e2e/tests/theme.spec.ts` (Wave 0 gap) | Playwright |
| Font loading | All 5 families load from `127.0.0.1`, none from `fonts.googleapis.com` | e2e | network-tab assertion in Playwright | Playwright |
| Closed-beta gate visual | Unauth visit to `/` shows landing not login | e2e | new test in `e2e/tests/closed-beta-visual.spec.ts` | Playwright |

### Sampling Rate

- **Per task commit** (every page restyle):
  ```
  cargo check --features ssr
  cargo check --features hydrate --target wasm32-unknown-unknown
  ```
  Plus the matching audit test if the page has one (`audit-{draft,tree-drafter,champion-pool,game-plan,post-game,team,misc-pages}.spec.ts`).

- **Per page review (D-10):**
  1. agent-browser screenshot of the route.
  2. Mental diff against the corresponding design source file (`/tmp/lol-design-handoff/.../*.jsx`).
  3. User approves or requests revision.
  4. Atomic commit on approval.

- **Per wave merge** (end of each plan):
  ```
  just verify          # cargo check + test + clippy + fmt
  cd e2e && npx playwright test  # full e2e suite
  ```

- **Phase gate** (before `/gsd-verify-work`):
  ```
  just verify
  just smoke
  cd e2e && npx playwright test
  /gsd-ui-review 17    # 6-pillar retroactive audit
  ```
  Plus the three grep sweeps (G-01, G-12, raw-hex).

### Wave 0 Gaps

The following test infrastructure does NOT yet exist and must be created in Plan A (Wave 0) before per-page restyle work begins:

- [ ] `e2e/tests/theme.spec.ts` — covers theme persistence across login/logout and hard nav (D-06)
- [ ] `e2e/tests/closed-beta-visual.spec.ts` — covers closed-beta landing visibility for unauth visitors (D-14, visual-only — logic is Phase 19.1)
- [ ] `e2e/tests/visual-regression.spec.ts` — Playwright `toHaveScreenshot()` baseline per route; runs on phase-gate to detect future drift
- [ ] `e2e/tests/fonts.spec.ts` — network-tab assertion that no `fonts.googleapis.com` request fires (G-01 enforcement)
- [ ] CI sweep in `.github/workflows/ci.yml` — three grep checks (G-01, G-12, raw hex). Phase 21 owns the full G-01..G-13 sweep but Phase 17 needs at least the G-01 + G-12 + raw-hex subset green by phase end.

**Existing infrastructure (already covers):** `cargo test --features ssr --lib` runs all unit + integration tests. `e2e/tests/pages.spec.ts` covers route smoke. `e2e/tests/audit-*.spec.ts` covers heavy-page interactions. `e2e/tests/fixtures.ts` provides `authedPage` helper.

**Framework install:** None needed. `cargo`, `playwright`, `agent-browser` already installed per CLAUDE.md.

---

## Plan Sizing Recommendation

Given the scale (~25 files restyled, ~10 new files, ~20 new assets, per-page review gate per D-10), the recommended plan structure mirrors the user's locked D-11 budget of "~7 plans":

### Wave 1 (must complete before any hub work — blocks everything)

**Plan A — Foundations + Theme Port + Font Self-Hosting + Nav Shell** (~10-12 tasks)

Tasks (rough):
1. Create `public/`, `public/fonts/{family}/`, `public/img/` directory tree
2. Download woff2 files via google-webfonts-helper into `public/fonts/`
3. Add `@font-face` declarations to `input.css`
4. Port `themes.css` content into `input.css` `@theme` block; merge demacia + pandemonium tokens; map onto `--t-*` aliases
5. Port `.canvas-grain` utility into `input.css`
6. Delete `[data-accent="*"]` blocks; optionally delete `[data-theme="light"]` block
7. Add `theme` field to `schema.surql` + `db.rs::set_user_theme()`
8. Refactor `theme_toggle.rs` to 2-state demacia/pandemonium toggle
9. Replace localStorage script in `app.rs` with SSR `data-theme` attr
10. Plumb `user.theme` into SSR context
11. Refactor `nav.rs` — 4-hub primary + sub-nav strip; preserve notifications + ModeToggle
12. Wave 0 e2e test files: `theme.spec.ts`, `closed-beta-visual.spec.ts`, `visual-regression.spec.ts` (baseline empty), `fonts.spec.ts`
13. CI sweep additions for G-01 + G-12 + raw-hex

Verification gates:
- `cargo check --features ssr` + `cargo check --features hydrate` both green
- All existing audit tests still green (no behavioral regression)
- Theme toggle works end-to-end (DB persistence verified)
- agent-browser screenshot of any existing page shows new fonts loaded, theme tokens applied
- G-01 grep returns zero hits

**Why this is one plan, not split:** Foundations changes are tightly coupled — splitting fonts from tokens from nav-shell creates partial-state commits where the app doesn't render correctly. One plan + one wave-end review.

### Wave 2 (hub plans — can run in parallel)

**Plan B — Strategy Hub** (~9 page commits + 2 component commits = ~11 tasks)
- Restyle `draft.rs` + `draft_board.rs`
- Restyle `tree_drafter.rs` + `tree_graph.rs`
- Restyle `champion_pool.rs` + `champion_picker.rs`
- Restyle `game_plan.rs` + `champion_autocomplete.rs`
- Restyle `post_game.rs`
- Restyle `opponents.rs` (utility tier — Open-Design)
- Restyle `action_items.rs` (utility tier — Open-Design)

**Heavy hub** — 8,278 lines of page code combined for the four big pages (draft/tree/pool/game-plan/post-game). If this plan exceeds practical size, split into **B1 (Strategy core: draft + tree + pool)** and **B2 (Strategy supporting: game-plan + post-game + opponents + action-items)**.

**Plan C — History Hub** (~4 page commits)
- Restyle `stats.rs`
- Restyle `match_detail.rs`
- Restyle `personal_learnings.rs` (Open-Design utility)
- Restyle `analytics.rs` (Open-Design utility)

**Plan D — Profile Hub** (~5 page commits)
- Restyle `profile.rs`
- Restyle `team/dashboard.rs`
- Restyle `team/roster.rs` (Open-Design utility)
- Restyle `team_builder.rs` (Open-Design utility)
- Restyle `solo_dashboard.rs`

### Wave 3 (depends on Wave 2 utility-surface work — Open-Design seeding must precede)

**Plan E — Open-Design Seeding** (~3-5 tasks)

Tasks:
1. Read `/home/jasper/Repositories/open-design/AGENTS.md` to learn DESIGN.md schema
2. Author `/home/jasper/Repositories/open-design/design-systems/lol-companion/DESIGN.md` with tokens + font assignments + component patterns + icon library
3. Author `tokens.css` (or equivalent) matching `input.css` tokens
4. Generate Open-Design HTML prototypes for each utility surface group (auth, team setup, action-items, opponents, personal-learnings, analytics, admin/invites, bug-report widget) — produces `.od/projects/{uuid}/*.html` files
5. Document the OD project paths in `.planning/phases/17-ui-consolidation/17-OD-MAP.md` for downstream reference

**Sequencing note:** Per D-21+D-22, Open-Design seeding must precede utility surface ports. **However**, the utility surfaces in Plans B/C/D depend on Plan E. Two options:
- **Option α:** Run Plan E in Wave 1 (right after Plan A) so Plans B/C/D can implement utility surfaces in Wave 2.
- **Option β:** Run Plans B/C/D in Wave 2 implementing **only the hero pages**; defer utility surfaces to Wave 3 after Plan E.

**Recommend Option α** — Plan E becomes part of Wave 1, smaller scope, parallelizable with Plan A foundations work.

### Wave 4 (closed-beta + final review — depends on FLUX assets)

**Plan F — Closed-Beta Surfaces + Public Routes + FLUX Assets** (~5-6 tasks)

Tasks:
1. Generate FLUX images via fal.ai (one-off CLI/API call); commit to `public/img/`
2. Author `.planning/assets/AI-IMAGES.md` reproducibility log
3. Restyle `home.rs` (public landing — redirect logic for unauth)
4. Build new `closed_beta.rs` page with FLUX hero background
5. Restyle `auth/login.rs` (Open-Design utility + optional FLUX bg)
6. Restyle `auth/register.rs` with invite-token URL handling (visual + invalid-invite error state — Phase 19.1 wires real validation)
7. Build new `admin/invites.rs` visual stub (Phase 19.1 wires logic)
8. Build new `legal/impressum.rs` and `legal/datenschutz.rs` route stubs (Phase 21 fills content)

**Plan G — Final Review + 6-Pillar Audit** (~3 tasks)

Tasks:
1. Capture visual-regression baselines via Playwright `toHaveScreenshot()`
2. Run `/gsd-ui-review 17` 6-pillar audit; fix any HIGH findings
3. Run grep sweeps (G-01, G-12, raw hex); fix any hits

### Wave dependency graph

```
Wave 1: [Plan A] [Plan E (OD seed)]   ← both run in parallel, no inter-dependency
            │           │
            └─────┬─────┘
                  │
Wave 2: [Plan B (Strategy)] [Plan C (History)] [Plan D (Profile)]   ← parallel, all depend on A+E
            │                    │                   │
            └────────────────────┼───────────────────┘
                                 │
Wave 3: [Plan F (closed-beta + public)]   ← depends on Plans A+E (foundations) and B+C+D (page styles)
            │
Wave 4: [Plan G (review + audit)]   ← depends on everything above
```

**Total plans: 7** (matches D-11 budget). If Plan B exceeds task budget, split into B1+B2 → 8 plans (still within tolerance).

**Pacing assumption:** D-10 per-page review gate makes each task linear (no batching). Estimate ~1 page per session if user is responsive, ~3 days for foundations + 1-2 weeks for hubs given user-availability dependency.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Open-Design's DESIGN.md schema is project-internal and must be discovered by reading `/home/jasper/Repositories/open-design/AGENTS.md` | Pattern 4 / Plan E | Plan E task 1 captures this via "read AGENTS.md first"; risk is low but if Open-Design has a strict schema we don't follow, prototypes may not regenerate correctly. |
| A2 | The hidden upload `draw-92acceeb-9fd2-499d-84e4-12ff75b7ab5d.png` (2576×1479, 118 KB, hand-drawn sketch judging by filename) is a composition reference for the closed-beta landing or a FLUX prompt anchor — not a usable asset | UI-SPEC §"Implementation Notes" item 10 | Confirm during Plan F task 1 by viewing the image. If it IS a usable asset, it might displace one of the FLUX-generated images; saves a generation but changes the prompt brief. |
| A3 | fal.ai is the recommended FLUX runtime over replicate.com for our ~10-image one-off | Standard Stack table | Cost-comparison data is from `[CITED: pricepertoken.com/image]`, dated 2026 — verified current. Risk: API key onboarding friction. Replicate is fully equivalent fallback. |
| A4 | Light theme block in `input.css:53-71` should be retired | UI-SPEC line 86-87 | UI-SPEC defers to executor; recommendation is retire because design has no light variant. Risk: a future v1.4 user-config feature requesting light might cost a re-add. |
| A5 | The flat-URL approach for the 4-hub IA is correct (preserve bookmarks per UI-SPEC line 178-180) | Pattern 3 | UI-SPEC explicitly chooses flat. Risk: if planner has a strong reason to nest (e.g., shared layout component reuse), the override decision is documented in UI-SPEC. |
| A6 | The per-page review gate (D-10) means each page commits separately AND pauses for user approval each time | Pitfall 8 | If "pause" interpretation is too strict, the phase calendar duration balloons. Recommend planner discusses this in `/gsd-discuss-phase` if not already settled. The "review queue" mitigation (commit per page, batch-review later) preserves D-10's atomicity. |
| A7 | All ~14 routes referenced in UI-SPEC Route Inventory exist OR are in scope to be created in this phase | Files Touched table | Verified by `grep` of `src/app.rs:50-87` for existing routes. New routes (`/closed-beta`, `/admin/invites`, `/legal/*`, `/solo`) are in UI-SPEC scope. Risk: low. |
| A8 | The existing audit e2e tests (`audit-{draft,tree-drafter,champion-pool,game-plan,post-game,team,misc-pages}.spec.ts`) primarily test interactions, not specific text/CSS, so they survive a visual restyle | Validation Architecture | Verified via `e2e/tests/pages.spec.ts` content (matches text generic enough to survive). If audit-tests are fragile, Plan A or per-hub plan must update them. Risk: medium — needs verification per hub. |
| A9 | Tailwind v4 `@theme { --font-display: ... }` directive automatically generates the `font-display` utility class | Pattern 1 | `[CITED: tailwindcss.com/docs/theme]` confirms the `--font-*` namespace generates utilities. |
| A10 | cargo-leptos copies `public/` → `target/site/` and Axum serves files at the same relative path | Architecture diagram | `[CITED: github.com/leptos-rs/cargo-leptos]` confirms; `Cargo.toml` already has `assets-dir = "public"`. Risk: low. |
| A11 | The `mystery image` (A2) is decorative-only and adds no requirements | (general) | It might be a reference for one of the AI-generated splash backgrounds, in which case Plan F's prompt should reference it. Resolved during Plan F task 1. |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed. **It is not empty** — see the 11 entries above. None block planning, but A1, A2, A6 are worth confirming during planning or the first task.

---

## Open Questions (RESOLVED)

1. **Per-page review gate cadence (D-10) — pause-per-page vs commit-per-page-batch-review?**
   - What we know: D-10 says "implement → screenshot → user approves → atomic commit → next."
   - What's unclear: Whether "user approves" must happen in real-time or whether the agent can commit per page (atomically) and the user batch-reviews later.
   - Recommendation: Surface to user during `/gsd-discuss-phase` re-pass. The "review queue" pattern (commit-per-page-with-`[REVIEW-PENDING]`-marker, batch-review later) preserves D-10's atomicity AND avoids agent stalls.
   - **RESOLVED:** Per-page atomic commit immediately after user approval, no batching. D-10 stands as written: each page restyle is its own `checkpoint:human-verify` gate; the executor pauses for "approved" before committing the page and proceeding to the next.

2. **Plan B (Strategy hub) split — single plan or B1+B2?**
   - What we know: Strategy hub has 7 routes, 8,278+ lines of page code across the heavy four (draft, tree-drafter, champion-pool, game-plan, post-game).
   - What's unclear: Whether one plan can stay coherent at 11 tasks or should split.
   - Recommendation: Default to single Plan B. If task estimation pushes >12, split.
   - **RESOLVED:** Split — Plan 03 is split into 03a (draft + draft_board + champion_picker), 03b (tree_drafter + tree_graph + champion_autocomplete), 03c (champion_pool + game_plan + post_game), 03d (opponents + action_items + ui.rs + stat_card.rs). Each sub-plan is ≤4 tasks. Wave 2 now executes 03a/03b/03c/03d (and 04, 05) in parallel against the Wave 1 foundation.

3. **`db_seed` mock data — replace with `data.jsx` content or keep current binary?**
   - What we know: CONTEXT.md "Claude's Discretion" leaves this open.
   - What's unclear: Whether the design's `data.jsx` (player names, team names, match data) better demos the new visual style than current seed.
   - Recommendation: Defer. The current seed works; switching mock data is orthogonal to visual port. Touch only if a per-page review surfaces "the mock data looks weird in the new style."
   - **RESOLVED:** Defer; keep existing `db_seed` binary. The current seed is sufficient to exercise the new visual style. Re-evaluation will only happen if a per-page review surfaces a concrete "the mock data looks weird in the new style" complaint, in which case a follow-up (post-Phase-17) task will port `data.jsx`. No work in this phase.

4. **FLUX runtime — fal.ai vs replicate vs self-host?**
   - What we know: D-17 leaves it open; CONTEXT.md "Claude's Discretion" defers to plan.
   - What's unclear: Cost vs latency vs sovereignty preference for ~10 images.
   - Recommendation: **fal.ai** — `[CITED: teamday.ai]` cheapest at our scale, sub-second latency, EU-flagged in this domain (acknowledged not strictly EU-sovereign — Black Forest Labs is German but fal.ai infra is US). If full EU sovereignty matters, self-host on a Hetzner GPU instance (~€1-2/hr for 30 min total = €1).
   - **RESOLVED:** Deferred to user decision in Plan 06 Task 1 (`checkpoint:decision`). The plan presents fal.ai (recommended), replicate.com (equivalent), and self-host on GPU rental as options; user selects at execution time. Default recommendation remains fal.ai. Total cost is ≤$0.30 for 2-3 images.

5. **Open-Design DESIGN.md schema details**
   - What we know: D-21 mandates the seed; AGENTS.md exists at `/home/jasper/Repositories/open-design/AGENTS.md` (not yet read in this research session).
   - What's unclear: Exact field structure expected by Open-Design tooling.
   - Recommendation: Plan E task 1 reads AGENTS.md and any existing `design-systems/*/DESIGN.md` example before authoring.
   - **RESOLVED:** Handed off to Plan 02 Task 1 (now Plan 02). The planner reads `/home/jasper/Repositories/open-design/AGENTS.md` first to discover schema conventions, then enumerates existing `design-systems/` for reference, then authors the seed in Task 2. No DESIGN.md schema is fixed at planning time — Task 1 discovers and Task 2 conforms.

6. **The hidden upload image's role**
   - What we know: `uploads/draw-92acceeb-9fd2-499d-84e4-12ff75b7ab5d.png` is 2576×1479, likely hand-drawn (`draw-` prefix suggests user sketch).
   - What's unclear: Whether it's a composition reference, a usable splash, or a test image.
   - Recommendation: Inspect during Plan F task 1 (closed-beta landing implementation).
   - **RESOLVED:** Handed off to Plan 06 Task 2 — inspect during FLUX prompt drafting. The executor opens the image, documents its role in `.planning/assets/AI-IMAGES.md` (composition reference / usable asset / disregard), and adjusts FLUX prompts accordingly. If usable as-is, it may displace one generation.

7. **CI sweep ownership boundary — Phase 17 vs Phase 21**
   - What we know: Phase 21 owns the full G-01..G-13 sweep; Phase 17 must satisfy G-01 + G-12 + no-raw-hex.
   - What's unclear: Whether Phase 17 should already commit the CI workflow (subset) or merely make the codebase pass at phase end.
   - Recommendation: Add a minimal CI sweep (G-01 + G-12 + hex) in Plan G. Phase 21 extends.
   - **RESOLVED:** Phase 17 ships a minimal G-01 + G-12 + raw-hex sweep in Plan 01 Task 9 (the `style_guardrails` job in `.github/workflows/ci.yml`). Phase 21 extends to the full G-01..G-13 set. The Phase 17 job runs on every PR and blocks if any of the three checks fail; Phase 21 adds the remaining ten guardrails alongside (does not replace) the Phase 17 subset.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust nightly | All compile | `[ASSUMED]` ✓ — already in use per Cargo.toml | nightly | none — required |
| cargo-leptos | dev/build | `[ASSUMED]` ✓ — referenced throughout CLAUDE.md | latest | none — required |
| Tailwind CSS standalone binary | dev/build | `[ASSUMED]` ✓ — `tailwindcss-linux-x64` per CLAUDE.md | v4 | none — required |
| Playwright | e2e | ✓ — `e2e/playwright.config.ts` exists | per package.json | n/a |
| agent-browser | per-page review (D-10) | ✓ — `.claude/skills/agent-browser/` exists per CLAUDE.md | n/a | manual screenshot via DevTools (slower) |
| google-webfonts-helper | font download | ✓ — public web service at gwfh.mranftl.com | live | direct download from fonts.google.com via curl |
| fal.ai or replicate.com API | FLUX image generation | ✗ — API key not yet provisioned | n/a | ~10 images can be generated by hand via the public FLUX.1 web demo if API isn't set up |
| FLUX.1 model access | image generation | (depends on runtime) | flux.1-pro or flux.1-dev | flux.1-schnell (cheaper, lower quality) |
| /home/jasper/Repositories/open-design | OD seeding + utility surfaces | ✓ — directory exists | n/a | none — D-21 requires it |
| Dev server (cargo leptos watch on :3020) | per-page review screenshots | ✓ — already a dev workflow per CLAUDE.md | n/a | n/a |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:**
- FLUX.1 API key — if not provisioned by Plan F, manual generation via web demo is acceptable for v1.3 launch (10 images, one-off).

---

## Security Domain

> Phase 17 is a UI port; security exposure is minimal. Most security concerns are deferred to Phase 19 (production hardening) and Phase 19.1 (closed-beta gate). However, this phase introduces or interacts with the following surfaces.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | axum-login + argon2 (existing); auth/login + auth/register restyle preserves form submission to existing server fns |
| V3 Session Management | yes | tower-sessions 0.14 + SurrealSessionStore (existing); no changes |
| V4 Access Control | yes | Page protection template (Resource + redirect Effect — see CLAUDE.md rule 50) preserved on every restyled page; closed-beta visual gate prepared for Phase 19.1 |
| V5 Input Validation | yes | `set_user_theme` server fn validates `theme IN ['demacia', 'pandemonium']`; invite-token form validates length/charset (Phase 19.1 fully implements) |
| V6 Cryptography | no | No crypto changes |
| V14 Configuration | yes | G-01 self-hosted fonts; G-12 focus rings; no third-party JS introduced |

### Known Threat Patterns for Leptos+Axum stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via theme attr injection | Tampering | Theme value validated server-side (`['demacia','pandemonium']` only); attr set via Leptos `view!` macro escaping |
| CSRF on theme/setting changes | Tampering | Existing axum-login session cookies + same-origin policy; no changes |
| Information disclosure via admin/invites route | Disclosure | Phase 19.1 returns 404 for non-admin (D-14: "does not reveal route existence"). Phase 17 stub follows same pattern |
| Bug-report widget element-label disclosure | Disclosure | Phase 18 owns the capture model — no PII in semantic labels per design |
| Font-loading mixed-content warnings | (operational) | Self-hosted (G-01) eliminates this entirely |
| Theme XSS via localStorage cache | Tampering | Server fn `set_user_theme` is the source of truth; localStorage is defense-in-depth cache only, validated on read |

---

## Sources

### Primary (HIGH confidence)

- `[VERIFIED: /tmp/lol-design-handoff/lol-team-companion-app/project/themes.css]` — Demacia/Pandemonium CSS custom-property tokens (lines 60-131), canvas-grain utility (lines 133-159), font CDN line that violates G-01 (line 7)
- `[VERIFIED: /tmp/lol-design-handoff/lol-team-companion-app/project/index.html]` — Load order: data.jsx → components.jsx → foundations.jsx → bundles → screens/ → app.jsx (lines 50-65)
- `[VERIFIED: /tmp/lol-design-handoff/lol-team-companion-app/project/app.jsx]` — Top-level routing (4 hub primary + 3 secondary), useTheme hook reading localStorage (lines 14-65)
- `[VERIFIED: /tmp/lol-design-handoff/lol-team-companion-app/project/foundations.jsx]` — Font usage (`var(--font-display)`, `var(--font-imperial)`, `var(--font-glitch)`, `var(--font-mono)`, `var(--font-ui)` — 817 references in handoff bundle)
- `[VERIFIED: /tmp/lol-design-handoff/lol-team-companion-app/project/components.jsx]` — Icon path set (lines 108-145), Card/Btn/Badge primitives, ChampTile sizing
- `[VERIFIED: /tmp/lol-design-handoff/lol-team-companion-app/project/screens/canonical.jsx]` — Canonical screen wrappers mapping route IDs to component variants per region
- `[VERIFIED: /home/jasper/Repositories/lol_team_companion/CLAUDE.md]` — Project conventions, semantic tokens, dev workflow, BFD linker, recursion limit, `cargo test --features ssr --lib`
- `[VERIFIED: /home/jasper/Repositories/lol_team_companion/.planning/phases/17-ui-consolidation/17-CONTEXT.md]` — All 23 locked decisions D-01..D-23
- `[VERIFIED: /home/jasper/Repositories/lol_team_companion/.planning/phases/17-ui-consolidation/17-UI-SPEC.md]` — 762-line UI design contract; route inventory, draft-board layout, tree-graph interactions, copywriting, etc.
- `[VERIFIED: /home/jasper/Repositories/lol_team_companion/Cargo.toml]` — `assets-dir = "public"`, no new crates needed, dual-target features (ssr + hydrate)
- `[VERIFIED: /home/jasper/Repositories/lol_team_companion/src/app.rs lines 1-89]` — Current 19-route flat structure, localStorage script for theme/accent (line 38)
- `[VERIFIED: /home/jasper/Repositories/lol_team_companion/src/components/nav.rs lines 1-100]` — `set_user_mode` precedent for D-06 mirror, ModeToggle pattern
- `[VERIFIED: /home/jasper/Repositories/lol_team_companion/src/components/theme_toggle.rs]` — Current 5-accent picker (151 lines, to be replaced)
- `[VERIFIED: /home/jasper/Repositories/lol_team_companion/input.css]` — Current `@theme` block, `--t-*` aliases, `[data-accent="*"]` blocks to delete
- `[VERIFIED: /home/jasper/Repositories/lol_team_companion/schema.surql lines 1-12]` — `user` table SCHEMAFULL with `mode` field as the migration precedent
- `[VERIFIED: .claude/rules/leptos-patterns.md]` — Project-specific rules 7-57 (server fn ordering, signal lifecycle, Auth/Routing patterns)
- `[VERIFIED: .claude/rules/wasm-patterns.md]` — Rules 35-56 (no `.unwrap()` in event handlers, debounced auto-save with cancellable timer, font CDN 404 trap)
- `[VERIFIED: e2e/tests/pages.spec.ts]` — Smoke test pattern; AUTHED_PAGES array — Phase 17 must extend with new routes

### Secondary (MEDIUM confidence)

- `[CITED: tailwindcss.com/docs/theme]` — `@theme` block + `--font-*` namespace auto-generates utilities (verified pattern matches v4 docs)
- `[CITED: book.leptos.dev/router/17_nested_routing.html]` — Leptos 0.8 nested routing via `<ParentRoute>` + `<Outlet>`
- `[CITED: github.com/leptos-rs/cargo-leptos]` — `assets-dir` copies to `target/site/`
- `[CITED: gwfh.mranftl.com]` — google-webfonts-helper for self-hosted woff2 + paste-ready `@font-face` CSS
- `[CITED: pricepertoken.com/image]` — FLUX.1 fal.ai vs replicate pricing comparison (2026 data)
- `[CITED: teamday.ai/blog/fal-ai-vs-replicate-comparison]` — fal.ai 30-50% cheaper, sub-second latency
- `[CITED: harrisonbroadbent.com/blog/tailwind-custom-fonts/]` — Tailwind v4 `@font-face` + `@theme` integration pattern

### Tertiary (LOW confidence — flagged in Assumptions Log)

- A1: Open-Design DESIGN.md schema — not directly inspected; mitigated by Plan E task 1 ("read AGENTS.md first")
- A2: Hidden upload image role — not visually inspected in this research session; resolved during Plan F task 1
- A8: Audit e2e test fragility post-restyle — needs spot-check during each hub plan; risk medium

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies already in repo, no new crates
- Architecture: HIGH — design source files inspected, theme/font patterns verified against Tailwind v4 docs
- Pitfalls: HIGH — drawn from project skill rules (`.claude/rules/`) and PROJECT MEMORY (auto-load context)
- Plan sizing: MEDIUM — depends on per-page review pacing (Open Question 1)
- Open-Design integration: LOW — A1 (DESIGN.md schema) not directly verified

**Research date:** 2026-05-07

**Valid until:** 2026-06-07 (1 month — design handoff is static; FLUX pricing may shift; Tailwind v4 is stable)
