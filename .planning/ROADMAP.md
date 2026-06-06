# Roadmap: LoL Team Companion

## Milestones

- ✅ **v1.0 Cross-Feature Intelligence** — Phases 1-5 (shipped 2026-03-18)
- ✅ **v1.1 Polish, Draft & Opponents Rework** — Phases 6-11 (shipped 2026-03-24)
- ✅ **v1.2 Solo Mode & Match Intelligence** — Phases 12-16 (shipped 2026-05-07)
- ✅ **v1.3 Launch Readiness (partial)** — Phases 16-19 shipped; Phases 20-24 SUPERSEDED by v1.4 (see note below)
- 🚧 **v1.4 Immersive Companion** — Phases 26-34 (in progress, started 2026-06-04)

> **v1.3 → v1.4 supersession note:** The v1.3 launch phases that were planned but never built (old Phases 20, 20.1, 21, 22, 23, 24) and the deferred Phase 25 (Draft Integration) are re-absorbed into v1.4 as requirements LNCH-01..07. Their ROADMAP detail sections below are preserved for audit history but are superseded — the canonical plan is in the v1.4 phase details (Phases 31-34). Phase numbers 20, 20.1, 21, 22, 23, 24, 25 are retired.

## Phases

<details>
<summary>✅ v1.0 Cross-Feature Intelligence (Phases 1-5) — SHIPPED 2026-03-18</summary>

- [x] Phase 1: Pipeline CTAs (3/3 plans) — completed 2026-03-14
- [x] Phase 1.1: Playwright Bug Audit (3/3 plans) — completed 2026-03-14 (INSERTED)
- [x] Phase 2: Aggregation Layer (3/3 plans) — completed 2026-03-15
- [x] Phase 3: Smart Dashboard (2/2 plans) — completed 2026-03-15
- [x] Phase 4: Inline Intel (3/3 plans) — completed 2026-03-17
- [x] Phase 5: Post-Game Loop + Polish (7/7 plans) — completed 2026-03-18

See: `.planning/milestones/v1.0-ROADMAP.md` for full details.

</details>

<details>
<summary>✅ v1.1 Polish, Draft & Opponents Rework (Phases 6-11) — SHIPPED 2026-03-24</summary>

- [x] Phase 6: Bug Fixes (3/3 plans) — completed 2026-03-22
- [x] Phase 7: UX Polish (2/2 plans) — completed 2026-03-22
- [x] Phase 8: Champion Pool & Role UI (2/2 plans) — completed 2026-03-23
- [x] Phase 9: Draft & Game Plan (3/3 plans) — completed 2026-03-23
- [x] Phase 10: Opponents Redesign (3/3 plans) — completed 2026-03-23
- [x] Phase 11: Cross-Feature & Testing (4/4 plans) — completed 2026-03-24

See: `.planning/milestones/v1.1-ROADMAP.md` for full details.

</details>

### ✅ v1.2 Solo Mode & Match Intelligence (Shipped: 2026-05-07)

**Milestone Goal:** Open the app to individual players — solo queue tracking, personal goals, and learnings — plus richer match detail from the Riot API that connects back into champion pool and draft decisions.

- [x] **Phase 12: Solo Mode Infrastructure** - Mode toggle, region selector, solo sync, ranked display, solo dashboard shell (completed 2026-03-25)
- [x] **Phase 13: Match Detail View** - Full 10-player scoreboard, timeline events, performance breakdown, on-demand lazy fetch with DB cache (completed 2026-03-27)
- [x] **Phase 14: Personal Learnings Journal** - Personal journal CRUD, one-click creation from match detail, champion/matchup/tag filtering (completed 2026-03-27)
- [x] **Phase 15: Goals & LP History** - LP trend graph, rank goal setting, goal auto-tracking against match history, champion performance trends (UAT passed 2026-05-06; close-out via Phase 16)
- ↪ **Phase 16 (old): Draft Integration** — DEFERRED to v1.4 (was: personal matchup notes in draft Intel sidebar, champion win rate badges on pool cards). Pivot rationale: launch-readiness takes priority over draft integration; the integration's value depends on having real users producing match data.

### 🚧 v1.3 Launch Readiness — Partially Shipped (started 2026-05-06)

**Milestone Goal:** Get a polished, production-deployed v1 in front of a closed-beta of named friends, with a built-in bug-report widget that auto-exports to a Claude-readable inbox so reports flow into the next coding session without manual triage.

**Shipped in v1.3:**
- [x] **Phase 16: Phase 15 Close-out** - CR-01 + CR-02 already fixed in `5902a81`. Clear WR-01 (lp_history + goal_progress refetch) + WR-02 (orphan get_personal_goals removed); secondary code review + verification; mark v1.2 fully closed. (completed 2026-05-07)
- [x] **Phase 17: UI Consolidation** - Claude Design primary pass + Open-Design utility surfaces; 22 visual-regression baselines; 6-pillar audit PASS-with-deferred. (completed 2026-05-11)
- [x] **Phase 18: Region Variants (INSERTED 2026-05-11)** - Demacia + Pandemonium structural component branches per Open-Design mockups: shared region-variant primitives (RiotTape, ToxicSticker, WaxSeal, Card with region+variant props, halftone/oil-spill backgrounds, glitch text-shadow util), then port hero pages to genuinely different per-region layouts, then regenerate visual baselines per theme. (completed 2026-05-22)
- [x] **Phase 18.1: SSR Theme Injection (gap closure for UI-18-RUNTIME-01)** - Axum middleware extracts theme from session cookie + auth.user.theme; theme_toggle.rs writes cookie; main.rs per-request InitialTheme injection. (completed 2026-05-25)
- [x] **Phase 18.2: Pandemonium Hydration Reconciliation (gap closure for UI-18.1-HYDRATE-01)** - Diagnose + fix SSR/hydrate structural mismatch; new hydration-no-panic.spec.ts regression gate; UI-18.1-HYDRATE-01 resolved. (completed 2026-05-26)
- [x] **Phase 19: Bug-Report Widget** - Floating Report button on every authed page → select-mode capture → modal; bug_report table; auto-export to .planning/INBOX/bug-reports.md on every server start. (completed 2026-05-31)

**Superseded by v1.4 (never built under v1.3 numbering):**
- ~~**Phase 20: Production Hardening**~~ → superseded by Phase 31 (LNCH-01)
- ~~**Phase 20.1: Closed-Beta Access Gate**~~ → superseded by Phase 31 (LNCH-02)
- ~~**Phase 21: Deploy Infrastructure**~~ → superseded by Phase 32 (LNCH-03)
- ~~**Phase 22: Compliance & Transparency**~~ → superseded by Phase 33 (LNCH-04)
- ~~**Phase 23: Pre-Launch Full Review**~~ → superseded by Phase 33 (LNCH-05)
- ~~**Phase 24: Soft Launch + Feedback Loop**~~ → superseded by Phase 34 (LNCH-06)
- ~~**Phase 25: Draft Integration**~~ → superseded by Phase 34 (LNCH-07)

### 🚧 v1.4 Immersive Companion (In Progress, started 2026-06-04)

**Milestone Goal:** Transform the flat themed UI into an explorable 2.5D "rooms" experience whose environment grows as the player accumulates work — then ship to closed beta. Solo Journey room is already built (proving spike). v1.4 builds the three remaining rooms, heals IA, ships original-world art, and completes the absorbed launch work.

**Note on build order:** Phases 26 (foundations) gates all rooms. Phases 27-29 (rooms) run after 26 and may be partially parallelized. Phase 30 (art) is calendar-parallel with 27-29 — rooms ship with placeholder assets; art swaps in at the same file paths. Phases 31-34 (launch) follow all room work, with Phase 31 (hardening + beta gate) mandatory before deploy.

- [x] **Phase 26: Room System Foundations** - Team-scoped growth model, AtmosphereRoot axis generalization, anti-structural-branching hydration rule, prefers-reduced-motion runtime listener (gates all room phases) (completed 2026-06-05)
- [x] **Phase 27: War Room** - Draft, tree-drafter, game-plan, opponents, and ban-priority under one parallax atmosphere; RoomSurface tab component established as the pattern (completed 2026-06-05)
- [ ] **Phase 28: Library & IA Healing** - Stats, match detail, analytics, post-game, and learnings under library atmosphere; opponents and learnings IA healed into their intent rooms; legacy flat routes preserved as deep links
- [ ] **Phase 29: Throne, QuickNav & Room A11Y** - Throne hosts identity + team dashboard + solo progression; QuickNav promoted app-wide with focus trap; every room passes hydration-no-panic canary in both regions
- [ ] **Phase 30: Art Production** - Original-world art identity established; FLUX layer-separated art for all rooms (first region) and second region; CI-enforced 400 KB/room/region budget
- [ ] **Phase 31: Production Hardening & Beta Gate** - Production binary hardened for HTTPS deploy; closed-beta invite gate on registration
- [ ] **Phase 32: Deploy Infrastructure** - Binary running on Hetzner CAX11 at chosen domain via Caddy; `just deploy` recipe
- [ ] **Phase 33: Compliance & Pre-Launch Review** - EU compliance artifacts in place; comprehensive multi-tool pre-launch review produces LAUNCH-GATE.md
- [ ] **Phase 34: Soft Launch & Draft Integration** - Named-friends beta live; match → draft → game-plan → review chain shipped

## Phase Details

### Phase 12: Solo Mode Infrastructure
**Goal**: Solo players can use the app without a team — they can toggle into solo mode, link their Riot account with a region, sync their solo queue match history, and see a personal dashboard with their current rank
**Depends on**: Phase 11
**Requirements**: SOLO-01, SOLO-02, SOLO-03, SOLO-04, SOLO-05, RANK-01, RANK-03
**Success Criteria** (what must be TRUE):
  1. User can toggle between team mode and solo mode; the selected mode persists after navigating away and refreshing the page
  2. User can select their region (EUW, NA, KR, etc.) when linking a Riot account on the profile page
  3. User without a team can sync their solo queue match history and see matches appear in the stats page
  4. User can filter match history by queue type (solo queue, flex, all)
  5. User sees a personal solo dashboard at a dedicated route showing recent matches, current ranked badge, and goal status placeholders
  6. User's current ranked tier, division, and LP are displayed; a ranked snapshot is recorded in the DB on each sync
**Plans**: 3 plans
Plans:
- [x] 12-01-PLAN.md — Schema, models, riot.rs region parameterization, ranked data infrastructure
- [x] 12-02-PLAN.md — Mode toggle in nav, auth redirects, team-only page gates, region dropdown
- [x] 12-03-PLAN.md — Solo dashboard page, ranked badge, queue filter, match sync, goal placeholders
**UI hint**: yes

### Phase 13: Match Detail View
**Goal**: Users can open any match from their history and see the full 10-player scoreboard with items, damage, gold, and vision — plus timeline events — without that data being fetched during the initial sync
**Depends on**: Phase 12
**Requirements**: MATCH-01, MATCH-02, MATCH-03, MATCH-04
**Success Criteria** (what must be TRUE):
  1. User can click any match in the history list and see all 10 participants with KDA, items, damage dealt, gold earned, and vision score
  2. User can view match timeline events (dragon kills, baron kills, tower kills, champion kills) for a given match
  3. User can see their own performance breakdown within the match — damage share percentage, vision score, and CS compared to game average
  4. Match detail data loads on-demand the first time a user opens a match, then loads instantly from the DB cache on subsequent opens without another Riot API call
**Plans**: 3 plans
Plans:
- [x] 13-01-PLAN.md — Schema, models, Riot API extraction, DB cache, server function, unit tests
- [x] 13-02-PLAN.md — Match detail page UI (scoreboard, performance), route wiring, click-to-navigate
- [x] 13-03-PLAN.md — Timeline visualization, filter bar, event markers, visual verification

**UI hint**: yes

### Phase 14: Personal Learnings Journal
**Goal**: Users can write personal post-game reflections tagged by champion and matchup, browse and filter their history, and start a new learning from a match detail with a single click
**Depends on**: Phase 13
**Requirements**: LEARN-01, LEARN-02, LEARN-03
**Success Criteria** (what must be TRUE):
  1. User can create a personal learning entry with free-text notes, tagged by champion played and opponent matchup
  2. User can click a "Add Learning" button on a match detail view and land on the learnings form with champion and matchup pre-filled from that match
  3. User can browse all their personal learnings and filter the list by champion, matchup, or tag
  4. Personal learnings are stored separately from team post-game reviews and are only visible to the user who created them
**Plans**: 3 plans
Plans:
- [x] 14-01-PLAN.md — Schema, model, DB CRUD functions, e2e smoke test scaffolding
- [x] 14-02-PLAN.md — Browse page, form page, nav/routing wiring, filter/sort, visual verification
- [x] 14-03-PLAN.md — Match detail integration (scoreboard + timeline "Add Learning" CTAs)
**UI hint**: yes

### Phase 15: Goals & LP History
**Goal**: Users can set personal improvement goals (rank target, CS/min, death limit), see their LP gain/loss over time as a graph, and view champion performance trends that auto-update from match history
**Depends on**: Phase 12
**Requirements**: RANK-02, LEARN-04, LEARN-05, LEARN-06
**Success Criteria** (what must be TRUE):
  1. User can view a line graph of LP history over time, derived from ranked snapshots captured at each sync
  2. User can set goals including a rank target tier, CS/min threshold, and per-game death limit
  3. User can see goal progress on the solo dashboard — each goal shows current average vs. the target derived from recent match history
  4. User can view champion performance trends showing win rate, KDA, and CS/min per champion over a selectable time window
**Plans**: 3 plans
Plans:
**Wave 1**
- [x] 15-01-PLAN.md — Schema, models, rank_score helper, 5 db functions, Wave 0 integration tests

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 15-02-PLAN.md — LP graph + goal cards on solo dashboard (server fns, LpHistoryGraph SVG, GoalCards with inline edit)
- [x] 15-03-PLAN.md — Champion Trends section on stats page (sortable table, time-window toggle, min-games filter)
**UI hint**: yes

### Phase 16: Phase 15 Close-out
**Goal**: Resolve the remaining open issues from the Phase 15 code review so v1.2 closes cleanly before launch-readiness work begins
**Depends on**: Phase 15 (delivered)
**State** (verified 2026-05-06 against main `4856140`):
  - ✅ **CR-01 already fixed** in `5902a81` — variable rename reverted; hydrate compiles; UAT confirms hover works
  - ✅ **CR-02 already fixed** in `5902a81` — `<string>snapshotted_at AS snapshotted_at` added to SELECT
  - ❌ **WR-01 still open** — `solo_dashboard.rs:249,276` only call `dashboard_resource.refetch()`; LP graph + goal cards stay stale after sync
  - ❌ **WR-02 still open** — `db.rs:4699` `get_personal_goals` has no production caller (only definition)
  - 15-REVIEW.md was not updated to reflect the criticals fixes — should also be reconciled
**Issues to resolve**:
  - **WR-01**: After sync, also refetch `lp_history_resource` and `goal_progress_resource`
  - **WR-02**: Wire `get_personal_goals` into a caller, or remove it (and remove its tests)
  - Reconcile `15-REVIEW.md` to mark CR-01/CR-02 as resolved-in-`5902a81`
  - Secondary `/gsd-code-review 15` + `/gsd-verify-work 15`
**Success Criteria**:
  1. After sync, LP graph and goal cards refetch automatically (no page reload required)
  2. `get_personal_goals` either has a production caller or is removed
  3. `15-REVIEW.md` reflects current state (no stale issues marked open)
  4. `/gsd-code-review 15` second pass produces no new HIGH/Critical findings
  5. `/gsd-verify-work 15` confirms ROADMAP success criteria 1-4 still pass
  6. STATE.md / MILESTONES.md mark v1.2 fully closed
**Plans**: 3 plans
Plans:
**Wave 1**
- [x] 16-01-PLAN.md — WR-01 hoist fix: hoist `lp_window` + `lp_history_resource` into `SoloDashboardPage`; add `goal_progress_resource.refetch()` and `lp_history_resource.refetch()` to both sync paths
- [x] 16-02-PLAN.md — WR-02 removal: delete dead `pub async fn get_personal_goals` from `src/server/db.rs`; delete `tests/db_personal_goal.rs`

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 16-03-PLAN.md — Run `/gsd-code-review 15` second pass + `/gsd-verify-work 15`; annotate `15-REVIEW.md` with Status: lines on all 7 findings; update `STATE.md` + `MILESTONES.md` to mark v1.2 shipped
**UI hint**: minimal

### Phase 17: UI Consolidation
**Goal**: A coherent, polished UI across all v1 pages — produced from a project-level UI-SPEC.md, designed primarily by Claude Design, with Open-Design filling missing surfaces, then audited via `/gsd-ui-review`
**Depends on**: Phase 16
**Success Criteria**:
  1. `.planning/phases/17-ui-consolidation/17-UI-SPEC.md` exists with route inventory, draft-board layout, tree-graph interactions, auth flows, champion picker UX, and bug-report widget placement (project-specific decisions only — tokens/colors/typography stay in vault per CLAUDE.md UI-SPEC scope rule)
  2. Claude Design has produced primary mockups for any page lacking final polish (verified by user review pass)
  3. Open-Design generates HTML prototypes for any new surfaces missing from primary pass (e.g. closed-beta-only screens, bug-report inbox widget)
  4. Implementation matches the UI-SPEC; `/gsd-ui-review` produces PASS verdict on the 6 quality dimensions
**Plans**: 10 plans (Phase 17 plan 03 split into 03a/03b/03c/03d for context bounding)
Plans:
**Wave 1** *(parallel — no inter-dependency)*
- [x] 17-01-PLAN.md — Foundations: theme port (demacia/pandemonium), font self-hosting (G-01), DB theme field, theme_toggle 2-state, app.rs SSR data-theme, nav.rs 4-hub IA, ornaments.rs + icon.rs, Wave 0 e2e specs, CI sweep job
- [x] 17-02-PLAN.md — Open-Design seeding: lol-companion DESIGN.md + tokens.css + 17-OD-MAP.md tracking table

**Wave 2** *(parallel — all depend on Wave 1)*
- [x] 17-03a-PLAN.md — Strategy hub draft: /draft + draft_board.rs + champion_picker.rs (heaviest single artifact)
- [x] 17-03b-PLAN.md — Strategy hub tree: /tree-drafter + tree_graph.rs + champion_autocomplete.rs
- [x] 17-03c-PLAN.md — Strategy hub strategy pages: /champion-pool + /game-plan + /post-game
- [x] 17-03d-PLAN.md — Strategy hub utility: /opponents + /action-items (Open-Design ports) + ui.rs + stat_card.rs
- [x] 17-04-PLAN.md — History hub: /stats, /match-detail, /personal-learnings (utility), /analytics (utility)
- [x] 17-05-PLAN.md — Profile hub: /profile (Captain's Folio), /team/dashboard (Strategy Room), /team/roster (utility), /team-builder (utility), /solo-dashboard (SoloConstellation), /home (auth-aware redirect to /closed-beta)

**Wave 3** *(depends on Wave 2)*
- [x] 17-06-PLAN.md — Closed-beta + auth + admin + legal + bug-widget: FLUX images + AI-IMAGES.md, /closed-beta hero, /auth/login + /auth/register (invite-token URL), /admin/invites stub, /legal/impressum + /legal/datenschutz stubs, bug_report_widget visual stub

**Wave 4** *(depends on Wave 3)*
- [x] 17-07-PLAN.md — Final audit: visual-regression baselines (22 routes), /gsd-ui-review 17 6-pillar audit (PASS-with-deferred), G-01/G-12/raw-hex grep sweeps, 17-UI-REVIEW.md, full-suite test pass
**UI hint**: yes (entire phase is UI work)

### Phase 18: Region Variants (INSERTED 2026-05-11)
**Goal**: Demacia and Pandemonium themes render genuinely different component compositions per region (not color-only swap), per Open-Design mockups
**Depends on**: Phase 17
**Why this phase exists**: Phase 17 ported color tokens for both regions but did NOT branch component structure. Open-Design mockups show structurally different visual languages (Demacia: heraldic crests, gilt cards, wax-letter dossiers, HeraldicDivider; Pandemonium: glitch type, riot-tape ticker, toxic-sticker bans, halftone backgrounds, SoloForge replacement for SoloConstellation). Phase 17-UI-REVIEW.md flagged the gap as MEDIUM/deferred under Visual coherence. Closed-beta launch (Phase 24) is the public moment for regional identity — must ship before then to avoid the region concept reading as cosmetic.
**Likely scope** (refined in SPEC):
  - Shared region-variant primitives: RiotTape, ToxicSticker, WaxSeal, Card with region+variant props, halftone + oil-spill background utilities, glitch text-shadow utility
  - Per-page region branches for ~9 hero pages (War Table, Solo Dashboard, Game Plan, Post Game, Team Dashboard, Profile, Champion Pool, Tree Drafter, Closed-Beta)
  - SoloConstellation → split into Demacia constellation + Pandemonium SoloForge
  - Regenerate visual-regression baselines per theme (~40 snapshots total)
**Plans**: 10 plans across 5 waves
Plans:
**Wave 1** *(parallel — no dependency)*
- [x] 18-01-PLAN.md — Port ~24 region primitives under src/components/region/ + delete ornaments.rs
- [x] 18-02-PLAN.md — PageLoading + PageEmpty per-region skeleton + empty states + animation keyframes

**Wave 2** *(parallel — depend on Wave 1)*
- [x] 18-03-PLAN.md — Port 3 no-patch pages: tree-drafter + champion-pool + match-report
- [x] 18-04-PLAN.md — Port draft-carousel + draft-war-table with light mismatch patches
- [x] 18-05-PLAN.md — Port solo-constellation with medium mismatch patches (D pool-gaps/last-10/sort + P tier-crest/4-cards)

**Wave 3** *(depend on Wave 1; 18-06 heaviest)*
- [x] 18-06-PLAN.md — Heavy port: team-dashboard with full Pandemonium 7-section data-surface rebuild (CHECKPOINT)
- [x] 18-07-PLAN.md — 4 sibling pairs port: draft-ledger, solo-journal, solo-forge, team-game-day-brief (gate pre-cleared per RESEARCH F-01)

**Wave 4** *(depends on Waves 2 + 3)*
- [x] 18-08-PLAN.md — Schema fields + DB getter/setter pairs + ModeToggle UI primitive + resolve_mode + server fns with allowlist

**Wave 5** *(parallel — depend on Wave 4)*
- [x] 18-09-PLAN.md — pixelmatch + pngjs install + 26 scoped baselines + region-diff.spec.ts (pixelDiffRatio > 0.40)
- [x] 18-10-PLAN.md — 6-pillar audit + REQ-7 utility-route grep gate + 18-UI-REVIEW.md + user manual review (CHECKPOINT)
**UI hint**: yes (entire phase is UI structural work)

### Phase 18.1: SSR Theme Injection (gap closure for UI-18-RUNTIME-01)
**Goal**: Close the architectural gap that prevents every Phase 18 structural region branch from activating at runtime — make SSR provide `InitialTheme` from the actual user's persisted theme (cookie + auth.user.theme) so the `region: String` prop captured at page entry is "demacia" or "pandemonium" per the user's actual preference, not always the hardcoded default
**Depends on**: Phase 18
**Why this phase exists**: Phase 17 plan 17-01 task 6 noted at `src/main.rs:82-87` that "SSR-authoritative theme requires axum middleware injection ... deferred to a follow-up plan if FOUC becomes visible." Phase 18 built 24+ region-branching primitives and 11 scoped pages on top of `region: String` props sourced from `use_context::<InitialTheme>()`, but `main.rs:88` still hardcodes `provide_context(InitialTheme::default())` (= `"demacia"`). Effect at runtime: token swap (gold↔pink) works via post-hydration `<html data-theme>` flip, but every page's `region` prop is permanently `"demacia"`, so the Pandemonium structural branches (DraftLedgerView Pandemonium arm, the 7-section team-dashboard Pandemonium rebuild, SoloForge, SoloJournal, TeamGameDayBriefView, etc.) never activate. Verifier passed 7/8 by source-grep; 18-UI-REVIEW.md passed 22/22 by source-audit; 18-09 region-diff passed at 0.5% threshold because token swap is real — none of them rendered a Pandemonium page in a real browser. Discovered 2026-05-25 via user-driven agent-browser sweep. Recorded as UI-18-RUNTIME-01 (severity: blocker) in `.planning/phases/18-region-variants/18-HUMAN-UAT.md`.
**Likely scope**:
  - **Axum middleware** to extract theme per request: precedence (a) `theme` cookie if present and valid; (b) `AuthSession.user.theme` if authenticated; (c) `"demacia"` default. Inject into request extensions; `leptos_routes_with_context` reads via `leptos_axum::extract` (sync workaround via per-route layer, OR async via a wrapping layer) → `provide_context(InitialTheme(theme))`.
  - **`theme_toggle.rs`** to also write a `theme=<value>; Path=/; Max-Age=31536000; SameSite=Lax` cookie on click (alongside the existing DB persist), so unauthenticated reloads (or first-page-after-login before AuthSession lands) persist the choice.
  - **`set_user_theme` server fn** unchanged (still persists to DB) but now also returns the new value so the cookie write can happen client-side after the server confirms (defense-in-depth against the cookie value being trusted as authoritative for authenticated users).
  - **Verification**: agent-browser sweep across all 11 scoped pages × 2 regions × N modes (29 distinct DOM configurations); grep the DOM for region-only markers (e.g., RiotTape "DRAFT_LEDGER · v0.1" on `/draft` ledger Pandemonium; "TEAM_BRIEF · GAME_DAY" RiotTape on `/team/dashboard` Pandemonium dashboard mode). Confirm token swap still works (no regression on Demacia rendering).
  - **18-09 re-baseline**: with structural branching activated, pixel diff per scoped route × region will jump from 0.5–2.5% to 5–15% (educated estimate; the bulk of layout actually changes shape now). Update `REGION_DIFF_THRESHOLD` and the per-route baselines accordingly. Documents the D-THRESHOLD deviation finally closed (SPEC's 0.40 is still too high for typographic/ornamental variants but >0.05 is achievable post-18.1).
**Decisions to lock in plan-phase**:
  - **Cookie name + scope**: leaning `lol_companion_theme=<value>` with `Path=/; Max-Age=1y; SameSite=Lax; HttpOnly=false` (client must read for optimistic update). Plan-phase locks the name.
  - **Sync vs async middleware extraction**: leaning sync via `tower::Layer` that runs before `leptos_routes_with_context`'s sync context closure, parses cookie + (optionally) reads `AuthSession` from extensions (already populated by `auth_layer`), and inserts the resolved theme into request extensions for the context closure to read synchronously. Avoids the async-extractor-in-sync-context problem documented at `main.rs:82-83`.
  - **Cookie precedence vs auth**: cookie wins over auth.user.theme when both present (lets a user override their account theme per-device for unauthenticated browsing). Plan-phase confirms.
  - **First-paint behavior**: when neither cookie nor auth is available (e.g., a fresh visit to the home page), SSR serves "demacia" and the anti-FOUC script in `<head>` (already present from Phase 17) does NOT need changes — it reads from the same data-theme attribute that SSR sets.
**Success Criteria**:
  1. `src/main.rs:88` no longer reads `InitialTheme::default()` — replaced with per-request injection
  2. Axum middleware extracts theme from cookie/session/default in that precedence
  3. `theme_toggle.rs` writes the `lol_companion_theme` cookie on every toggle click (in addition to the DB persist)
  4. Hard navigation (close tab, reopen, click toggle to non-default, reload) keeps the chosen region rendered structurally
  5. Agent-browser DOM grep on all 11 scoped pages × Pandemonium confirms Pandemonium-only markers present (RiotTape, Glitch, zine cards, etc. per content contracts)
  6. UI-18-RUNTIME-01 marked as resolved in 18-HUMAN-UAT.md
  7. 18-09 region-diff baselines re-captured at the post-18.1 runtime; new `REGION_DIFF_THRESHOLD` documented (expected ~0.05 lower bound, actual measurements ~0.05–0.15)
  8. Both compile targets clean; ≥111 unit tests pass; no regression on utility routes (REQ-7 holds — utility routes still have ZERO `is_pandemonium` references and still render identically across regions)
**UI hint**: yes (gap closure for UI phase; small implementation surface but requires the agent-browser verification arc)
**Plans**: 3 plans across 3 waves
Plans:
**Wave 1**
- [x] 18.1-01-PLAN.md — Axum middleware (`src/server/theme_layer.rs`) + main.rs per-request InitialTheme injection + theme_toggle.rs synchronous cookie write; pure-resolver unit tests; live curl precedence matrix (5 cases)
**Wave 2** *(depends on 18.1-01)*
- [x] 18.1-02-PLAN.md — Playwright `setRegion` fixture switched to cookie-injection + reload; re-capture 24 scoped visual-regression baselines under post-18.1 runtime; calibrate `REGION_DIFF_THRESHOLD` from measured floor (expected 0.03–0.10)
**Wave 3** *(depends on 18.1-02; CHECKPOINT)*
- [x] 18.1-03-PLAN.md — Agent-browser runtime sweep across all 11 scoped pages × 2 regions × ~29 configs; DOM-grep for per-page Pandemonium markers; utility-route REQ-7 spot-check; final human sign-off; flip UI-18-RUNTIME-01 → RESOLVED in 18-HUMAN-UAT.md

### Phase 18.2: Pandemonium Hydration Reconciliation (gap closure for UI-18.1-HYDRATE-01)
**Goal**: Restore client-side interactivity on every Pandemonium-themed page by eliminating the SSR/hydrate divergence in a Pandemonium structural primitive that causes the `tachys-0.2.14/src/html/mod.rs:217` `Option::unwrap()` on `None` during hydration. SSR HTML continues to render correctly post-fix; the WASM client successfully reconciles against it and attaches event handlers (mode toggle, theme toggle, drag-drop) on Pandemonium pages.
**Depends on**: Phase 18.1
**Why this phase exists**: Phase 18.1's SSR theme-injection fix made the Pandemonium structural branches reachable at runtime for the first time. The Phase 18.1-03 agent-browser sweep (2026-05-25) found that every Pandemonium-themed page panics during WASM hydration with `panicked at /home/jasper/.cargo/registry/.../tachys-0.2.14/src/html/mod.rs:217:14: called Option::unwrap() on a None value`. Demacia pages hydrate cleanly; the divergence is specific to a Pandemonium structural primitive (or per-page Pandemonium view) whose SSR-rendered markup tree does not match the WASM render of the same component. Pre-18.1 this was hidden because SSR always rendered the Demacia branch — both sides agreed. Recorded as UI-18.1-HYDRATE-01 (severity: bug) in `.planning/phases/18-region-variants/18-HUMAN-UAT.md`. Evidence in `.planning/phases/18.1-ssr-theme-injection-gap-closure-for-ui-18-runtime-01/18.1-03-EVIDENCE.md` (Investigation Flag 1). Without this fix, Phase 18's structural region work is visible but inert under Pandemonium — and Phase 19's Bug-Report Widget (which needs WASM event handlers on every authenticated page) cannot ship cleanly on Pandemonium until hydration is restored.
**Success Criteria**:
  1. No `tachys-0.2.14/src/html/mod.rs:217` `Option::unwrap()` on `None` panic in the JS console on any scoped page × Pandemonium configuration
  2. The culprit component(s) documented in a phase-local diagnosis file (which primitive, which kind of SSR/hydrate divergence, why the fix works)
  3. Mode-toggle click on `/draft`, `/solo`, `/team/dashboard` under Pandemonium switches the visible view (proves WASM event handlers attached)
  4. Theme-toggle click on a Pandemonium page reaches Demacia and back without panic and without losing client state
  5. At least one drag-drop or other rich interaction on a Pandemonium page (e.g. draft slot drag-drop) fires
  6. Regression gate: a new e2e spec asserts no hydration panic on any Pandemonium scoped page; runs in CI
  7. UI-18.1-HYDRATE-01 marked RESOLVED in `18-HUMAN-UAT.md` (preserve original `reported:` block for audit trail, matching the Phase 18.1 closure pattern)
  8. Both compile targets clean; ≥121 unit tests pass; 52/52 Wave-2 e2e tests still pass; Phase 18.1 SSR injection still works (regression check on UI-18-RUNTIME-01 markers)
  9. No regression on Demacia: Demacia pages continue to hydrate cleanly and interact correctly
**UI hint**: yes (gap closure for UI phase; small implementation surface but requires the agent-browser verification arc + new regression gate)
**Plans**: 5 plans across 2 waves
Plans:
**Wave 1** *(sequential — bisect → bisect → fix; each plan depends on prior)*
- [x] 18.2-01-PLAN.md — Bisect leaves-first cycles 1-4 (Card, SectionHead, Glitch, RiotTape) via short-circuit-and-reload; initialize 18.2-DIAGNOSIS.md with sanity baseline + per-cycle results
- [x] 18.2-02-PLAN.md — Bisect remaining leaves cycles 5-8 (CompanionSigil, ModeToggle, LPProgress, Btn) + per-page Pandemonium views (cycles 9-13 if needed) + D-04 WASM render-tracing escalation if needed; finalize Final culprit list + Root cause analysis in DIAGNOSIS.md
- [x] 18.2-03-PLAN.md — Apply real Leptos fix (Fix Pattern 1/2/3 per D-09) or D-10 cfg(ssr) fallback to identified culprit(s); regression check + Demacia parity sweep + Summary table in DIAGNOSIS.md
**Wave 2** *(depend on Wave 1; Plan 05 has CHECKPOINT)*
- [x] 18.2-04-PLAN.md — Author e2e/tests/hydration-no-panic.spec.ts (14-row panic-sweep matrix + 5 D-13 interactivity triad tests); just verify + just e2e green
- [x] 18.2-05-PLAN.md — Agent-browser live re-sweep on 7 Pandemonium scoped routes × 2 regions + 4 interactivity probes captured in 18.2-05-EVIDENCE.md; human-verify checkpoint; flip UI-18.1-HYDRATE-01 → RESOLVED in 18-HUMAN-UAT.md (preserve reported: block per Phase 18.1 close-out pattern); update Summary + Gaps blocks (CHECKPOINT)

### Phase 19: Bug-Report Widget (renumbered from Phase 18)
**Goal**: A floating in-app feedback button that lets the user click any page element, describe what went wrong or what they wish for, and have it auto-exported to a Claude-readable inbox file so the next coding session sees the queue without manual triage
**Depends on**: Phase 16
**Capture model** (decided in pivot plan):
  - Page URL (e.g. `/draft`)
  - Semantic element label (e.g. "Champion picker → search input"); not a CSS selector — selectors don't survive recompile per `.claude/rules/wasm-patterns.md` rule 56
  - Free-text user description
  - Category toggle: `bug` | `wishlist`
  - User attribution (user_id, timestamp)
  - No screenshot in v1 (deferred — html2canvas adds ~1 day of complexity)
**Success Criteria**:
  1. Floating "Report" button visible on every authenticated page (mounted in `app.rs`)
  2. Clicking the button enters "select" mode; clicking any element opens a modal pre-filled with that element's semantic label
  3. Submitting the modal persists a row in a new `bug_report` SurrealDB table
  4. On every server start, an auto-export task writes/updates `.planning/INBOX/bug-reports.md` with all open reports grouped by category and recency, formatted for direct ingestion by the next Claude session
  5. CLAUDE.md mentions the inbox location so future sessions discover it on context load
  6. No dark patterns (per `[[guardrails#G-10]]`) — neutral language, no pre-filled ratings, no confirmshaming
**UI hint**: yes (floating widget UX + 7-page label rollout + e2e regression coverage)
**Plans**: 4 plans across 3 waves
Plans:
**Wave 1** *(backend foundation)*
- [x] 19-01-PLAN.md — Schema + shared model + DB CRUD (create_bug_report, list_bug_reports, list_open_bug_reports) + inline integration tests (mitigates T-19-01 at DB layer via ASSERT clause; T-19-02 at DB layer via empty-description guard; T-19-05 deferred-with-Forbidden)
**Wave 2** *(parallel — UI wiring + auto-export task)*
- [x] 19-02-PLAN.md — Extend Phase 17 widget stub: WidgetState (Idle/Selecting/Editing), global capture-phase click listener via StoredValue<Option<Closure>>, esc-cancel, real submit_bug_report server-fn call replacing console.log stub, toast "Thanks. Your report is in." (period), CSS rule for data-feedback-selecting outline; defense-in-depth server-fn validation
- [x] 19-03-PLAN.md — src/server/bug_report_export.rs with pure render_inbox + impure export_open_reports (BUG_REPORT_INBOX_PATH env var with default ./.planning/INBOX/bug-reports.md), main.rs synchronous .await hook with tracing::warn! warn-and-continue, CLAUDE.md ## Bug-Report Inbox section with T-19-04 prompt-injection warning, .planning/INBOX/.gitkeep marker
**Wave 3** *(rollout + e2e regression — has manual checkpoint)*
- [x] 19-04-PLAN.md — data-feedback-label rollout to 7 first-priority pages (draft, solo_dashboard, team/dashboard, stats, champion_pool, game_plan, post_game) + e2e/tests/bug-report.spec.ts (5 tests via authedPage fixture) + human-verify checkpoint sweeping all 7 pages via agent-browser and inspecting inbox file after server restart; hydration-no-panic regression gate

---

## v1.4 Immersive Companion — Phase Details

> **Supersession note:** v1.4 Phases 31-34 supersede the v1.3 phases that were planned but never built (old Phases 20, 20.1, 21, 22, 23, 24, 25). The old phase detail sections above are preserved for audit history. The canonical launch work is here, re-scoped around the immersive rooms milestone.
>
> **Phase numbering:** v1.4 starts at Phase 26 (highest prior phase was 25). Phase numbers 20, 20.1, 21, 22, 23, 24, 25 are retired.

### Phase 26: Room System Foundations
**Goal**: The data model, component contracts, and documented rules that all three new rooms depend on are in place — team growth state is queryable, `AtmosphereRoot` accepts arbitrary axis names, the anti-structural-branching hydration rule is codified, and the prefers-reduced-motion runtime listener is fixed
**Depends on**: Phase 19 (Solo Journey room already built on branch `feat/immersive-rooms-solo-journey`)
**Requirements**: ROOM-01, ROOM-02, ROOM-03, A11Y-01
**Success Criteria** (what must be TRUE):
  1. A user with a team can load any new room and the atmosphere growth overlays reflect team activity (drafts, opponents, etc.) — `get_team_growth_counts` + `TeamGrowthState` round-trip is functional and unit-tested
  2. `AtmosphereRoot` accepts optional tier props for all four rooms' axis names (War Room: tactics/intel/command; Library: records/tomes/analysis; Throne: roster/progression/crown) without breaking Solo Journey
  3. A developer reading `CLAUDE.md` and `.claude/rules/leptos-patterns.md` can find the anti-structural-branching hydration rule before building any room — the rule is explicitly stated with a reference to the Phase 18.2 incident
  4. A user who enables "Reduce Motion" in their OS mid-session (without reloading) sees the parallax stop immediately — the runtime `MediaQueryList` change-listener is active in `AtmosphereRoot`
**Plans**: 4 plans (1 wave, fully parallel — zero file overlap)
Plans:
**Wave 1** *(parallel — no inter-dependency)*
- [x] 26-01-PLAN.md — ROOM-01: TeamGrowthCounts + TeamGrowthState (War Room axis derivation) + get_team_growth_counts query, unit-tested
- [x] 26-02-PLAN.md — ROOM-03: anti-structural-branching hydration rule codified in CLAUDE.md + leptos-patterns.md (18.2 / html/mod.rs:217 refs)
- [x] 26-03-PLAN.md — ROOM-02 + A11Y-01: AtmosphereRoot axis generalization (all 4 rooms) + runtime prefers-reduced-motion MQL change-listener (CHECKPOINT)
- [x] 26-04-PLAN.md — ROOM-02: standalone reusable RoomSurface ARIA tablist component with ?tab= sync + dummy-surface unit test
**UI hint**: no (model, DB, documentation, and component internals — no new visible UI surfaces)

### Phase 27: War Room
**Goal**: Users can reach the War Room at `/war-room` and access draft planning, tree-drafter, game-plan creation, opponent scouting, and ban-priority as tabbed surfaces under a single parallax atmosphere — the room's environment grows visibly as the team accumulates strategy work
**Depends on**: Phase 26
**Requirements**: ROOM-04
**Success Criteria** (what must be TRUE):
  1. User can navigate to `/war-room` and see a parallax atmosphere with placeholder art; switching between Draft, Tree-Drafter, Game Plan, Opponents, and Ban Priority tabs does not cause the atmosphere to flash or remount
  2. The tab bar uses `role="tablist"` / `role="tab"` / `aria-selected` / `aria-controls` so screen readers identify it as a tab panel group distinct from the global nav
  3. A team with saved drafts and scouted opponents sees more atmospheric detail than a fresh team (growth overlays advance through tiers based on actual team data)
  4. Round-trip from `/draft` (flat route) to `/war-room` and back does not accumulate duplicate `mousemove` listeners — parallax amplitude is unchanged after the round-trip
  5. `/war-room` appears in the e2e smoke suite and passes the hydration-no-panic canary in both demacia and pandemonium themes
**Plans**: 8 plans across 4 waves
- [x] 27-01-PLAN.md — RoomSurface keep-mounted per-tab PanelFn refactor (D-06/07/09)
- [x] 27-02-PLAN.md — AtmosphereRoot mousemove accumulation guard (AtomicBool)
- [x] 27-03-PLAN.md — Wave 0 e2e scaffolding: war-room.spec.ts + pages.spec entry + tier-1 seed fixture
- [x] 27-04-PLAN.md — Extract DraftSurface + BanPrioritySurface from draft.rs (thin DraftPage shell)
- [x] 27-05-PLAN.md — Extract TreeDrafterSurface from tree_drafter.rs (thin shell)
- [x] 27-06-PLAN.md — Extract GamePlanSurface + OpponentsSurface (thin shells)
- [x] 27-07-PLAN.md — Compose WarRoomRoom + WarSurfaces + get_team_growth_state + /war-room route + placeholder assets
- [x] 27-08-PLAN.md — e2e phase gate: SC1-SC5 green + flat-route regression (D-02)
**UI hint**: yes

### Phase 28: Library & IA Healing
**Goal**: Users can reach the Library at `/library` and access stats, match detail, analytics, post-game review, and team learnings as tabbed surfaces under a hall-of-records atmosphere; previously siloed features (personal learnings, opponents) are now reachable inside their intent rooms while their legacy flat routes remain valid deep-link targets
**Depends on**: Phase 26 (Phase 27 recommended but not strictly required)
**Requirements**: ROOM-05, ROOM-07
**Success Criteria** (what must be TRUE):
  1. User can navigate to `/library` and browse stats, match detail, analytics, post-game, and team learnings as surfaces without the atmosphere remounting between tabs
  2. A user who follows a link to `/personal-learnings` (the old flat route) arrives at the learnings surface within the Library — the route resolves rather than 404ing
  3. A user who follows a link to `/opponents` (the old flat route) arrives at the Opponents surface within the War Room — flat routes for IA-healed features redirect into their intent room
  4. A team with post-game reviews, team notebook entries, and match syncs sees more atmospheric detail than a fresh team (Library growth axes: records, tomes, analysis)
  5. `/library` appears in the e2e smoke suite and passes the hydration-no-panic canary in both regions
**Plans**: 7 plans across 3 waves
Plans:
**Wave 1** *(parallel — no inter-dependency; independent files)*
- [x] 28-01-PLAN.md — Library growth model (LibraryGrowthCounts/State + from_counts + tier tests) + get_library_growth_counts batch query
- [x] 28-02-PLAN.md — Wave 0 e2e wiring: /library smoke entry + redirected-entry removal in pages.spec; /library added to hydration-no-panic SCOPED_ROUTES
- [x] 28-03-PLAN.md — Extract StatsSurface + MatchDetailSurface; Stats→Match-Detail shared-signal bridge (D-07) + in-tab recent-match list (D-06)
- [x] 28-04-PLAN.md — Extract AnalyticsSurface + PostGameSurface + PersonalLearningsSurface (chrome-free; /personal-learnings/new href preserved)

**Wave 2** *(depend on Wave 1; 28-06 sequential after 28-05 — shared app.rs + library.rs)*
- [ ] 28-05-PLAN.md — Compose LibraryRoom + LibrarySurfaces + get_library_growth_state + /library route + placeholder assets (CHECKPOINT)
- [ ] 28-06-PLAN.md — IA-healing redirect shells (6 flat routes → intent room; /match/:id id preserved; /personal-learnings/new + /draft/tree-drafter/game-plan untouched)

**Wave 3** *(phase gate — depends on all)*
- [ ] 28-07-PLAN.md — e2e phase gate: SC1 smoke + SC2/SC3 redirect resolution + SC5 canary both regions + SC4 tier unit tests + full-suite regression
**UI hint**: yes

### Phase 29: Throne, QuickNav & Room A11Y
**Goal**: The Throne room hosts identity and progression at `/throne`; QuickNav (Cmd+K) works from every page in the app with a proper focus trap; and every room passes the per-route hydration-no-panic canary in both regions with WCAG AA text contrast and visible focus rings throughout
**Depends on**: Phase 27, Phase 28
**Requirements**: ROOM-06, ROOM-08, A11Y-02
**Success Criteria** (what must be TRUE):
  1. User can navigate to `/throne` and see team identity, team dashboard summary, and solo progression surfaces under a parallax atmosphere; the room's environment reflects roster size, LP milestones, and combined team record
  2. Pressing Cmd+K (or Ctrl+K) from any page — including `/war-room`, `/library`, `/throne`, and all flat routes — opens the QuickNav palette; pressing Escape closes it and returns focus to the element that triggered it
  3. While the QuickNav overlay is open, keyboard Tab does not reach parallax plane elements behind it — the focus trap is effective
  4. Every room (`/war-room`, `/library`, `/throne`) passes the hydration-no-panic canary in both demacia and pandemonium themes: no `tachys` `Option::unwrap()` panic appears in the console, and at least one interactive element (tab switch, growth state display) responds to user input
  5. Text on the surface layer of every room holds WCAG AA contrast (4.5:1 minimum) over the art planes — the contrast scrim at z-8 is verified per room
**Plans**: TBD
**UI hint**: yes

### Phase 30: Art Production
**Goal**: Original-world, legally-safe FLUX art is in place for all three new rooms in both regions — the project establishes its own visual identity (Arcane-inspired but not Riot IP), and every room's atmosphere looks like a real place rather than a placeholder gradient
**Depends on**: Phase 26 (art pipeline scripts need confirmed asset paths; rooms may be in progress or complete — art swaps in at existing file paths with no code changes)
**Requirements**: ART-01, ART-02, ART-03
**Success Criteria** (what must be TRUE):
  1. The Riot IP go/no-go decision is logged in PROJECT.md Key Decisions before the first FLUX generation run — original-world identity (Path A) or accepted-risk LoL-names (Path B) is explicit and the art prompts in `AI-IMAGES.md` reflect the chosen path; `/consult legal` was run before committing any generated outputs
  2. All three rooms have layer-separated WebP art for the first region (war-room, library, throne — 3 planes + 3 overlays each = 18 files); each file is within budget (sky/mid ≤ 80 KB, near ≤ 60 KB, each overlay ≤ 30 KB, total ≤ 400 KB/room/region); the CI `find public/img/rooms -name "*.webp" -size +100k` check returns 0
  3. BiRefNet layer separation was validated on one test plane for halo-fringe artifacts before the full 18-asset run; `AI-IMAGES.md` records the seed, compute path, and final file size for every asset
  4. All three rooms have second-region art for both regions (18 more files, same budget constraints); all rooms' atmosphere is visually distinct between regions
**Plans**: TBD
**UI hint**: yes (asset production; no code changes, but requires room routes to be reachable for visual verification)

### Phase 31: Production Hardening & Beta Gate
**Goal**: The binary is safe to deploy behind HTTPS — all production-readiness gaps closed — and registration is gated by an invite mechanism so only named friends can sign up during the closed beta
**Depends on**: Phase 29 (all rooms complete; art in Phase 30 can trail but rooms must be functional)
**Requirements**: LNCH-01, LNCH-02
**Note**: This phase supersedes old v1.3 Phases 20 and 20.1. All detail from those planning stubs is carried forward here.
**Success Criteria** (what must be TRUE):
  1. The same binary runs locally (via `cargo leptos watch`) and on production — the only difference is environment variables; `LEPTOS_SITE_ADDR`, cookie security, and log level are all env-driven
  2. A Riot API outage produces a graceful UI fallback (empty state or cached data), not a 500 error; Data Dragon patch version is fetched dynamically at startup rather than hardcoded
  3. A nightly SurrealKV snapshot script runs, and restoring a snapshot locally rebuilds the database directory cleanly
  4. Registration is rejected for any request that does not present a valid, unconsumed invite code — existing users can still log in; `/legal/*` and the homepage remain reachable without a code
  5. An admin user can mint invite codes via an admin-only route; each code records which user consumed it and when
**Plans**: TBD
**UI hint**: yes (registration form change + admin invite page — LNCH-02 surface)

### Phase 32: Deploy Infrastructure
**Goal**: The production binary is running on the shared Hetzner CAX11 at the chosen domain, served via Caddy with HTTPS — `just deploy` is the single command to ship a new version
**Depends on**: Phase 31
**Requirements**: LNCH-03
**Note**: This phase supersedes old v1.3 Phase 21.
**Success Criteria** (what must be TRUE):
  1. `just deploy` cross-compiles the binary for aarch64, rsyncs binary + WASM bundle + public assets, and restarts the systemd service
  2. The chosen domain serves over HTTPS with a Let's Encrypt certificate auto-provisioned by Caddy; the `_bg.wasm` rename is handled in the deploy recipe
  3. Post-deploy smoke passes three checks: `/healthz` returns 200, a server function endpoint responds, and `curl -sI https://<host>/pkg/lol_team_companion_bg.wasm` returns 200 OK — pure SSR HTML 200 is not sufficient to confirm WASM delivery
**Plans**: TBD
**UI hint**: no

### Phase 33: Compliance & Pre-Launch Review
**Goal**: All EU-compliance artifacts are in place and a comprehensive multi-tool review of the v1.4 milestone produces a go/no-go `LAUNCH-GATE.md` — no external user touches the app before both gates pass
**Depends on**: Phase 32
**Requirements**: LNCH-04, LNCH-05
**Note**: This phase supersedes old v1.3 Phases 22 and 23.
**Success Criteria** (what must be TRUE):
  1. `/legal/impressum` and `/legal/datenschutz` are live with all required clauses (§5 DDG Impressum, 4-section DSE including Hetzner as named subprocessor and Riot puuid as a US-processor data flow with legal basis)
  2. The vault `[[lol-team-companion]]` Tier-A transparency section exists with data flows, legal basis, retention, and third parties; AI-art disclosure is included per ART-01's original-world decision
  3. `.planning/LAUNCH-GATE.md` exists with one row per review check (PASS/FAIL/WAIVED with rationale); all HIGH findings from `/gsd-secure-phase`, `/gsd-ui-review`, and `/gsd-code-review` are fixed before Phase 34
  4. Post-deploy smoke commands are documented and dry-run on the production server before any beta invite is sent
**Plans**: TBD
**UI hint**: minimal (legal pages only)

### Phase 34: Soft Launch & Draft Integration
**Goal**: Named-friend beta is live with real users on real traffic; the match → draft → game-plan → review chain is shipped so the app's connected-features promise is complete; the bug-report inbox drives the post-beta backlog
**Depends on**: Phase 33
**Requirements**: LNCH-06, LNCH-07
**Note**: This phase supersedes old v1.3 Phase 24 (Soft Launch) and the deferred v1.3/v1.4 Phase 25 (Draft Integration / XFEAT-03/04).
**Success Criteria** (what must be TRUE):
  1. Named-friends invite list (5-15 people) is onboarded with personal invite links; production deploy is live at the chosen domain
  2. When a champion appears in the draft Intel sidebar, any personal matchup notes the user has for that champion are visible in the sidebar panel — matchup intelligence flows from personal learnings into draft decisions
  3. Champion pool cards display a win rate badge sourced from solo match history when the user has played that champion in recent matches
  4. The bug-report widget is exercised by at least 3 beta users in the first week and `.planning/INBOX/bug-reports.md` is populated; the next Claude session reads it for backlog seeding
  5. No HIGH-severity production incidents in the first 7 days; any incidents are triaged via the bug-report inbox
**Plans**: TBD
**UI hint**: yes (LNCH-07 draft intel integration)

---

## Progress

**Execution Order:**
v1.3 shipped: 16 → 17 → 18 → 18.1 → 18.2 → 19. v1.3 unbuilt phases (20–25) superseded by v1.4.
v1.4: 26 (foundations) → 27 (War Room) → 28 (Library + IA) → 29 (Throne + QuickNav + A11Y) → 30 (art, calendar-parallel with 27-29) → 31 (hardening + beta gate) → 32 (deploy) → 33 (compliance + review) → 34 (soft launch + draft integration).

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Pipeline CTAs | v1.0 | 3/3 | Complete | 2026-03-14 |
| 1.1 Playwright Bug Audit | v1.0 | 3/3 | Complete | 2026-03-14 |
| 2. Aggregation Layer | v1.0 | 3/3 | Complete | 2026-03-15 |
| 3. Smart Dashboard | v1.0 | 2/2 | Complete | 2026-03-15 |
| 4. Inline Intel | v1.0 | 3/3 | Complete | 2026-03-17 |
| 5. Post-Game Loop + Polish | v1.0 | 7/7 | Complete | 2026-03-18 |
| 6. Bug Fixes | v1.1 | 3/3 | Complete | 2026-03-22 |
| 7. UX Polish | v1.1 | 2/2 | Complete | 2026-03-22 |
| 8. Champion Pool & Role UI | v1.1 | 2/2 | Complete | 2026-03-23 |
| 9. Draft & Game Plan | v1.1 | 3/3 | Complete | 2026-03-23 |
| 10. Opponents Redesign | v1.1 | 3/3 | Complete | 2026-03-23 |
| 11. Cross-Feature & Testing | v1.1 | 4/4 | Complete | 2026-03-24 |
| 12. Solo Mode Infrastructure | v1.2 | 3/3 | Complete | 2026-03-25 |
| 13. Match Detail View | v1.2 | 3/3 | Complete | 2026-03-27 |
| 14. Personal Learnings Journal | v1.2 | 3/3 | Complete | 2026-03-27 |
| 15. Goals & LP History | v1.2 | 3/3 | Complete (close-out via P16) | 2026-05-06 |
| 16. Phase 15 Close-out | v1.3 | 3/3 | Complete | 2026-05-07 |
| 17. UI Consolidation | v1.3 | 10/10 | Complete | 2026-05-11 |
| 18. Region Variants | v1.3 | 10/10 | Complete | 2026-05-22 |
| 18.1 SSR Theme Injection (gap closure) | v1.3 | 3/3 | Complete | 2026-05-25 |
| 18.2 Pandemonium Hydration Reconciliation | v1.3 | 5/5 | Complete | 2026-05-26 |
| 19. Bug-Report Widget | v1.3 | 4/4 | Complete | 2026-05-31 |
| ~~20. Production Hardening~~ | ~~v1.3~~ | — | SUPERSEDED by Phase 31 | — |
| ~~20.1 Closed-Beta Access Gate~~ | ~~v1.3~~ | — | SUPERSEDED by Phase 31 | — |
| ~~21. Deploy Infrastructure~~ | ~~v1.3~~ | — | SUPERSEDED by Phase 32 | — |
| ~~22. Compliance & Transparency~~ | ~~v1.3~~ | — | SUPERSEDED by Phase 33 | — |
| ~~23. Pre-Launch Full Review~~ | ~~v1.3~~ | — | SUPERSEDED by Phase 33 | — |
| ~~24. Soft Launch + Feedback Loop~~ | ~~v1.3~~ | — | SUPERSEDED by Phase 34 | — |
| ~~25. Draft Integration~~ | ~~v1.4~~ | — | SUPERSEDED by Phase 34 | — |
| 26. Room System Foundations | v1.4 | 4/4 | Complete   | 2026-06-05 |
| 27. War Room | v1.4 | 8/8 | Complete    | 2026-06-05 |
| 28. Library & IA Healing | v1.4 | 4/7 | In Progress|  |
| 29. Throne, QuickNav & Room A11Y | v1.4 | 0/? | Not started | — |
| 30. Art Production | v1.4 | 0/? | Not started | — |
| 31. Production Hardening & Beta Gate | v1.4 | 0/? | Not started | — |
| 32. Deploy Infrastructure | v1.4 | 0/? | Not started | — |
| 33. Compliance & Pre-Launch Review | v1.4 | 0/? | Not started | — |
| 34. Soft Launch & Draft Integration | v1.4 | 0/? | Not started | — |
