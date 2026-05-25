# Roadmap: LoL Team Companion

## Milestones

- ✅ **v1.0 Cross-Feature Intelligence** — Phases 1-5 (shipped 2026-03-18)
- ✅ **v1.1 Polish, Draft & Opponents Rework** — Phases 6-11 (shipped 2026-03-24)
- ✅ **v1.2 Solo Mode & Match Intelligence** — Phases 12-16 (shipped 2026-05-07)
- 🚧 **v1.3 Launch Readiness** — Phases 16-23 (in progress, started 2026-05-06)
- 🔜 **v1.4 Draft Integration & Post-Launch Backlog** — Phase 24+ (deferred Draft Integration + bug-report inbox graduates)

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

### 🚧 v1.3 Launch Readiness (In Progress, started 2026-05-06)

**Milestone Goal:** Get a polished, production-deployed v1 in front of a closed-beta of named friends, with a built-in bug-report widget that auto-exports to a Claude-readable inbox so reports flow into the next coding session without manual triage.

**Decisions baked in (from pivot plan, see `.planning/phases/16..23/`):**
- Phase 15 critical bugs (CR-01 WASM hover, CR-02 Rule 40) become Phase 16 close-out, not pre-pivot inline fixes.
- Hetzner deployment reuses feynman-lookup's CAX11 (same VPS, new Caddy stanza, port 3001).
- Domain TLD finalised in Phase 21 (e.g. `lol-companion.gg` placeholder; flag EU-tools value charter for non-EU TLD).
- Bug-report widget captures page URL + semantic element label + free text + bug/wishlist toggle (no screenshots, no CSS selectors — selectors don't survive recompile).
- Launch shape: closed beta via named-friends invite list (no public self-serve registration in v1.3).

- [x] **Phase 16: Phase 15 Close-out** - CR-01 + CR-02 already fixed in `5902a81` (15-REVIEW.md was not updated). Remaining: clear WR-01 (refetch lp_history + goal_progress resources after sync) + WR-02 (orphan get_personal_goals — wire or remove); secondary code review + verification; mark v1.2 fully closed. (completed 2026-05-07)
- [x] **Phase 17: UI Consolidation** - Claude Design primary pass + Open-Design utility surfaces; 22 visual-regression baselines; 6-pillar audit PASS-with-deferred. (completed 2026-05-11)
- [x] **Phase 18: Region Variants (INSERTED 2026-05-11)** - Demacia + Pandemonium structural component branches per Open-Design mockups: shared region-variant primitives (RiotTape, ToxicSticker, WaxSeal, Card with region+variant props, halftone/oil-spill backgrounds, glitch text-shadow util), then port hero pages (War Table, Solo Dashboard, Game Plan, Post Game, Team Dashboard, Profile, Champion Pool, Tree Drafter, Closed-Beta) to genuinely different per-region layouts (not color-only swap), then regenerate visual baselines per theme. (completed 2026-05-22 — but **structural branching never activates at runtime; blocked on UI-18-RUNTIME-01; gap-closure in Phase 18.1**)
- [x] **Phase 18.1: SSR Theme Injection (gap closure for UI-18-RUNTIME-01)** - Close the Phase-17-deferred SSR theme-injection gap that blocks every Phase 18 structural region branch from activating at runtime. Axum middleware extracts theme from session cookie + `auth.user.theme`; `theme_toggle.rs` writes the cookie alongside the existing DB persist (so unauthenticated reloads persist); `main.rs:88` `provide_context(InitialTheme::default())` replaced with the per-request value. Re-run all 11 scoped pages × 2 regions × N modes in agent-browser to confirm structural branches activate; re-baseline 18-09's region-diff threshold (expect 5–15% pixel diff now, vs 0.5–2.5% pre-fix). (completed 2026-05-25)
- [ ] **Phase 18.2: Pandemonium Hydration Reconciliation (gap closure for UI-18.1-HYDRATE-01)** - Diagnose and fix the SSR/hydrate hash mismatch in a Pandemonium structural primitive that causes `tachys-0.2.14/src/html/mod.rs:217` `Option::unwrap()` on `None` for every Pandemonium page. SSR HTML is correct (Phase 18.1 proved that); client-side hydration panics, killing all WASM event handlers (mode toggle, theme toggle, drag-drop). Bisect Pandemonium structural primitives (RiotTape, ToxicSticker, WaxSeal, Card, halftone/oil-spill backgrounds, glitch text-shadow util) and per-page Pandemonium views (DraftLedgerView, TeamGameDayBriefView, SoloForge, SoloJournal, 7-section team-dashboard) to find which one emits SSR markup that diverges from its WASM render; fix the SSR/hydrate divergence; re-run the 18.1-03 agent-browser sweep with WASM event handlers exercised to verify interactivity restored on Pandemonium.
- [ ] **Phase 19: Bug-Report Widget (renumbered from 18)** - Element-click → modal (URL + semantic label + text + bug/wishlist); SurrealDB `bug_report` table; auto-export task writes `.planning/INBOX/bug-reports.md` on every server start.
- [ ] **Phase 20: Production Hardening (renumbered from 19)** - Externalize site-addr (env-driven); secure cookies behind HTTPS; prod log config + rotation; Riot API timeout/retry/cache; dynamic Data Dragon patch; graceful shutdown; nightly SurrealKV snapshot.
- [ ] **Phase 20.1: Closed-Beta Access Gate (renumbered from 19.1)** - Gate registration behind an invite mechanism so the deployed binary is URL-reachable but only the named-friends list can sign up. Login + legal pages remain public.
- [ ] **Phase 21: Deploy Infra (shared CAX11) (renumbered from 20)** - Cross-compile via `cargo zigbuild --target aarch64-unknown-linux-gnu.2.36`; new systemd unit at port 3001; Caddyfile stanza; `just deploy` recipe; pre-deploy smoke + post-deploy WASM-200 + server-fn check (per feynman 2026-04-16 incident).
- [ ] **Phase 22: Compliance & Transparency (renumbered from 21)** - Pick + register TLD (flag values charter for non-EU); DSE + Impressum (§5 DDG, no Steuernummer); Tier-A transparency in `[[lol-team-companion]]` vault entity; G-01..G-13 CI sweep; Riot Developer Portal application.
- [ ] **Phase 23: Pre-Launch Full Review (renumbered from 22)** - `/gsd-audit-milestone v1.3` + full `/gsd-code-review` + `/gsd-secure-phase` + `/gsd-ui-review` + `/ultrareview` + `/consult security`; fix HIGH findings; produce `.planning/LAUNCH-GATE.md`.
- [ ] **Phase 24: Soft Launch + Feedback Loop (renumbered from 23)** - Deploy; named-friends invite list; monitor `bug_report` for 1 week; weekly `/gsd-inbox` triage; backlog grooming feeds v1.4.

### 🔜 v1.4 Draft Integration & Post-Launch Backlog (Deferred)

**Milestone Goal:** Once v1.3 is shipped and producing real-user match data, ship the deferred Draft Integration alongside the highest-priority items surfaced from the launch bug-report inbox.

- [ ] **Phase 25: Draft Integration** (was old Phase 16, then renumbered Phase 24, now Phase 25 after Region Variants insertion) - Personal matchup notes surface in draft Intel sidebar; champion pool cards display solo-history win rate badges. Depends on Phase 14 + Phase 15 — both already complete.
- [ ] Phase 26+: Inbox-driven backlog from v1.3 launch (defined post-launch).

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
**Likely scope**:
  - **Bisect** the Pandemonium structural primitives (RiotTape, ToxicSticker, WaxSeal, Card region+variant, halftone/oil-spill backgrounds, glitch text-shadow util) and per-page Pandemonium views (DraftLedgerView Pandemonium arm, 7-section team-dashboard Pandemonium rebuild, SoloForge, SoloJournal, TeamGameDayBriefView) to identify which component(s) emit SSR markup that diverges from the WASM render. Strategy: short-circuit each Pandemonium primitive in turn (return empty `view! { <></> }` or the Demacia branch), reload a representative Pandemonium page (`/solo` or `/draft` with `lol_companion_theme=pandemonium`), watch the console for the panic. The component whose removal eliminates the panic is the culprit (or one of several).
  - **Diagnose** the SSR/hydrate divergence in the culprit(s). Common Leptos 0.8 + tachys patterns that produce this panic: `{move || ...}` reactive blocks rendering different node-count branches between SSR (no JS, signals at initial value) and hydrate (signals at restored value); `Suspense` boundaries with stale state; conditional rendering using `if region == "pandemonium" { ... } else { ... }` where one branch has fewer children than the SSR generator placeholder anchors expected; mismatched `Show`/`For`/`Suspense` keying.
  - **Fix** the divergence. Likely shapes: (a) move conditional Pandemonium structural primitives behind `{move || ...}` so they hydrate identically; (b) ensure both region branches return the same node-count skeleton at SSR; (c) replace dynamic markup that only renders post-hydration with `Show fallback=...` so SSR placeholder anchors are emitted.
  - **Verify** with the Phase 18.1-03 agent-browser sweep extended to assert: (1) no `tachys` panic in console on any Pandemonium page; (2) mode-toggle click on `/draft`/`/solo`/`/team/dashboard` Pandemonium switches the visible view; (3) theme-toggle click on a Pandemonium page reaches Demacia and back without panic; (4) at least one drag-drop or other rich interaction (e.g. draft slot drag) fires on a Pandemonium page.
**Decisions to lock in plan-phase**:
  - **Bisection method**: leaning short-circuit-and-reload (manual agent-browser per-primitive). Alternative: WASM-side `console::log` instrumentation that prints the component being rendered at hydrate time, walk the tree to find where reconciliation fails. Plan-phase picks the cheaper one based on Wave-1 cost estimate.
  - **Reproduction harness**: leaning a new Playwright spec `e2e/tests/hydration-no-panic.spec.ts` that visits each scoped Pandemonium page, listens for `page.on('pageerror')` and console errors, and fails on any `Option::unwrap()` text. Becomes the regression gate (runs in CI from this phase forward).
  - **Fix-vs-defer trade-off if the divergent component is structurally hard to refactor**: as a last resort, gate the SSR markup of the culprit behind `cfg(feature = "ssr")` to emit an identical placeholder to what the WASM client first renders. Plan-phase locks the trade-off rule (preferred: real fix; fallback: SSR-side placeholder).
  - **Interactivity assertion list**: lock the exact set of "WASM event handlers must fire on Pandemonium" assertions in plan-phase — at minimum mode-toggle on `/draft`+`/solo`+`/team/dashboard`, theme-toggle on `/profile`, and one drag-drop on `/draft`. Each becomes a Playwright assertion.
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
- [ ] 18.2-01-PLAN.md — Bisect leaves-first cycles 1-4 (Card, SectionHead, Glitch, RiotTape) via short-circuit-and-reload; initialize 18.2-DIAGNOSIS.md with sanity baseline + per-cycle results
- [ ] 18.2-02-PLAN.md — Bisect remaining leaves cycles 5-8 (CompanionSigil, ModeToggle, LPProgress, Btn) + per-page Pandemonium views (cycles 9-13 if needed) + D-04 WASM render-tracing escalation if needed; finalize Final culprit list + Root cause analysis in DIAGNOSIS.md
- [ ] 18.2-03-PLAN.md — Apply real Leptos fix (Fix Pattern 1/2/3 per D-09) or D-10 cfg(ssr) fallback to identified culprit(s); regression check + Demacia parity sweep + Summary table in DIAGNOSIS.md
**Wave 2** *(depend on Wave 1; Plan 05 has CHECKPOINT)*
- [ ] 18.2-04-PLAN.md — Author e2e/tests/hydration-no-panic.spec.ts (14-row panic-sweep matrix + 5 D-13 interactivity triad tests); just verify + just e2e green
- [ ] 18.2-05-PLAN.md — Agent-browser live re-sweep on 7 Pandemonium scoped routes × 2 regions + 4 interactivity probes captured in 18.2-05-EVIDENCE.md; human-verify checkpoint; flip UI-18.1-HYDRATE-01 → RESOLVED in 18-HUMAN-UAT.md (preserve reported: block per Phase 18.1 close-out pattern); update Summary + Gaps blocks (CHECKPOINT)

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
**Plans**: TBD
**UI hint**: yes

### Phase 20: Production Hardening (renumbered from Phase 19)
**Goal**: Close all production-readiness gaps surfaced by the audit so the binary is safe to deploy behind HTTPS on Hetzner
**Depends on**: Phase 16
**Hard blockers to close**:
  1. Externalize `site-addr` from `Cargo.toml` metadata to runtime env (`LEPTOS_SITE_ADDR` or equivalent)
  2. Flip `cookies.with_secure(false)` to `with_secure(true)` (env-conditional for local dev)
  3. Riot API: add timeout, retry-with-backoff, in-memory cache for Data Dragon, graceful degradation if Riot API down
  4. Dynamic Data Dragon patch fetch (replaces hardcoded `15.6.1`)
  5. Graceful shutdown hook (flush SurrealDB, drain in-flight requests)
  6. Nightly SurrealKV snapshot script (cron + restic or simple tar+rotate)
  7. Production log config: structured fields, rotation, sane defaults at `info` level
**Success Criteria**:
  1. Same binary runs locally (`cargo leptos watch`) and on prod (different env vars only)
  2. Riot API outage produces graceful UI fallback, not a 500
  3. Data Dragon version is fetched at startup (cached for the process lifetime, refreshed on next boot)
  4. Backup script tested locally — restoring a snapshot rebuilds the SurrealKV directory cleanly
  5. Server processes SIGTERM and exits within 30s with no in-flight request loss
**Plans**: TBD
**UI hint**: no (backend-only)

### Phase 20.1: Closed-Beta Access Gate (renumbered from Phase 19.1)
**Goal**: Public registration is gated by an invite mechanism so that the deployed v1.3 binary is reachable by URL but only the named-friends list can complete account creation
**Depends on**: Phase 20
**Decisions to lock in plan-phase**:
  - **Gate model**: choose one of (A) single shared `BETA_INVITE_CODE` env var, (B) per-user `invite_code` SurrealDB table minted by an admin, or (C) email allow-list. Leaning (B) because it gives per-friend revocation, tracks who consumed which code, and surfaces a real "X of N invites used" signal during the beta. Plan-phase locks the choice.
  - **Gate location**: registration server fn only. Login still works for already-registered users; homepage and `/legal/impressum` + `/legal/datenschutz` (per Phase 22) stay reachable without a code so the legal pages can be cited externally.
  - **Admin role**: a single admin user (first registered, or flagged via `ADMIN_USER_EMAIL` env) sees an admin-only "Mint invite" page listing issued codes and their consumed status. Non-admin users get a 404 on the route, not a 403, to avoid leaking the route's existence.
  - **Removal path**: when the gate is dropped (likely v1.4 or post-beta), the `invite_code` table can stay (audit trail) but the registration fn check becomes feature-flag-controlled. No migrations needed to drop the gate.
  - **No anti-bot beyond the code**: a 6-12 char alphanumeric code from a CSPRNG is sufficient for ≤ 30-account beta. No CAPTCHA, no rate-limiting work in this phase (Phase 20 already adds Riot-side timeouts; abuse handling for registration deferred to v1.4 if needed).
**Success Criteria**:
  1. Registration server fn rejects sign-ups without a valid, unconsumed invite code and returns a clean form-level error
  2. Admin can mint invite codes via an admin-only route; on consumption each code is marked with the consuming `user_id` and timestamp
  3. Login, homepage, and `/legal/*` routes remain reachable without an invite code
  4. New `invite_code` table appears in `schema.surql` (or chosen alternative is documented) with appropriate field constraints
  5. e2e smoke test covers: invalid code → reject; valid code → registration succeeds and code is marked consumed; reuse of consumed code → reject
  6. Bug-report widget (Phase 19) still functions for invited users — no regression on already-shipped surfaces
**Plans**: TBD
**UI hint**: yes (registration form change + minimal admin-mint page)

### Phase 21: Deploy Infrastructure (Shared CAX11) (renumbered from Phase 20)
**Goal**: Get the production binary running on feynman-lookup's existing Hetzner CAX11 alongside the existing service, served via Caddy at the new domain
**Depends on**: Phase 20, Phase 20.1
**Decisions baked in**:
  - Same CAX11, no new VPS provisioning (`cloud-init.yml` already applied)
  - Cross-compile locally via `cargo zigbuild --target aarch64-unknown-linux-gnu.2.36` (Debian 12 glibc pin per `[[cross-project-memory]]`)
  - New systemd unit `lol_team_companion.service` (user `lol`); listens on `127.0.0.1:3001`
  - New Caddyfile stanza for `lol-companion.<tld>` → `127.0.0.1:3001`
  - Caddy on 80/443 already exposed — no firewall changes needed (two-layer firewall caveat applies if changed)
  - Manual deploy via `just deploy` (no CI/CD push; matches feynman-lookup pattern)
  - `EnvironmentFile=/etc/lol_team_companion.env` populated post-deploy via SSH heredoc (never in git, never in shell history)
**Success Criteria**:
  1. `just deploy` cross-compiles, rsyncs binary + WASM + assets, restarts service
  2. New domain serves over HTTPS with Let's Encrypt cert auto-provisioned by Caddy
  3. `cargo-leptos` `_bg.wasm` rename handled in deploy recipe (`cp crate.wasm crate_bg.wasm`)
  4. Pre-deploy smoke: `cargo check --features ssr` + `cargo check --features hydrate` both pass
  5. Post-deploy smoke (per feynman 2026-04-16 incident): hits `/healthz` AND a server-fn endpoint AND `curl -sI https://host/pkg/lol_team_companion_bg.wasm | grep "200 OK"` — pure SSR HTML 200 hides binary-arch mismatch and broken WASM hydrate
  6. CSP via Caddy permits Leptos 0.8 per-request script nonces with documented `'unsafe-inline'` TODO
**Plans**: TBD
**UI hint**: no

### Phase 22: Compliance & Transparency (renumbered from Phase 21)
**Goal**: All EU-compliance artifacts in place before any external user touches the deployed app — Impressum, Datenschutzerklärung, vault Tier-A section, Riot API approval status, and a CI sweep for the project's hard-NO guardrails
**Depends on**: Phase 16
**Hard blockers (from vault `[[guardrails]]` and `[[values-charter]]`)**:
  1. Pick TLD (e.g. `lol-companion.gg` placeholder) — flag `[[values-charter#2]]` "EU tools preferred" if non-EU TLD chosen; document the justification
  2. Register the domain
  3. **Impressum**: cite §5 DDG (NOT §5 TMG, repealed 2024-05-14 per `[[guardrails#G-03]]`); no Steuernummer (per `[[guardrails#G-04]]`)
  4. **Datenschutzerklärung** (4 sections per `[[cross-project-memory]]` "German Kleinunternehmer Impressum"):
     - Server-Logfiles → Art. 6 Abs. 1 lit. f
     - Registration / user account → Art. 6 Abs. 1 lit. b
     - Hosting → Hetzner named subprocessor + AVV reference
     - Betroffenenrechte → LfDI Baden-Württemberg link
  5. **Riot API personal-data flow**: Riot puuid → US-headquartered processor must appear in DSE Tier-A table with legal basis
  6. **Vault Tier-A transparency section** in `wiki/entities/lol-team-companion.md` (currently absent — `[[guardrails#G-13]]` hard NO)
  7. **G-01..G-13 CI sweep**: rendered HTML must NOT contain `fonts.googleapis.com`, `fonts.gstatic.com`, "TMG", "Musterstraße", `outline:none` without ring; CI fails on hits
  8. **Riot Developer Portal**: confirm application status (Approved/Acknowledged); submit if not done
**Success Criteria**:
  1. `/legal/impressum` and `/legal/datenschutz` routes live with all required clauses
  2. Vault `[[lol-team-companion]]` Tier-A section exists with data flows, legal basis, retention, third parties
  3. CI workflow includes a guardrails sweep that fails on any hit
  4. Riot Developer Portal application status documented in `.planning/phases/22-compliance-transparency/22-RIOT-STATUS.md`
**Plans**: TBD
**UI hint**: minimal (legal pages only)

### Phase 23: Pre-Launch Full Review (renumbered from Phase 22)
**Goal**: A comprehensive multi-tool review of the entire v1.3 milestone before any public-facing deploy, producing a single LAUNCH-GATE.md that is the go/no-go document
**Depends on**: Phase 17, 18, 19, 20, 21, 22
**Review chain**:
  1. `/gsd-audit-milestone v1.3` — does v1.3 deliver on its goal?
  2. `/gsd-code-review` (full repo, not just diff) — bugs, security, quality
  3. `/gsd-secure-phase` — threat model verification
  4. `/gsd-ui-review` — 6-pillar visual audit on production build
  5. `/ultrareview` — multi-agent cloud review of the launch branch
  6. `/consult security "v1.3 launch checklist"` — vault security advisor pass
**Success Criteria**:
  1. All HIGH findings from any tool are fixed before Phase 24
  2. `.planning/LAUNCH-GATE.md` exists with one row per check (PASS/FAIL/WAIVED with rationale)
  3. Post-deploy smoke commands documented and dry-run on staging
**Plans**: TBD
**UI hint**: no

### Phase 24: Soft Launch + Feedback Loop (renumbered from Phase 23)
**Goal**: Real users on real traffic, with the bug-report inbox driving the v1.4 backlog
**Depends on**: Phase 23
**Success Criteria**:
  1. Production deploy live at chosen domain
  2. Named-friends invite list (5-15 people) onboarded with personal invite links
  3. Bug-report widget exercised by at least 3 users in the first week
  4. `.planning/INBOX/bug-reports.md` populated and read by the next Claude session
  5. Weekly `/gsd-inbox` triage produces a graded backlog seeding v1.4 phases
  6. No HIGH-severity production incidents in the first 7 days
**Plans**: TBD
**UI hint**: no

### Phase 25: Draft Integration (deferred from old Phase 16, then Phase 24, now Phase 25 after Region Variants insertion)
**Goal**: Personal game experience feeds back into draft decisions — matchup notes surface in the draft Intel sidebar and solo match history win rates appear as badges on champion pool cards
**Depends on**: Phase 14, Phase 15
**Requirements**: XFEAT-03, XFEAT-04
**Success Criteria** (what must be TRUE):
  1. When a champion is highlighted in the draft Intel sidebar, any personal matchup notes for that champion are visible in the sidebar panel
  2. Champion pool cards display a win rate badge sourced from solo match history when the user has played that champion in recent matches
**Plans**: TBD (post-launch, scoped after real-user data is in)
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order. v1.3: 16 → 17 → 18 (Region Variants, inserted 2026-05-11) → 19 → 20 → 20.1 → 21 → 22 → 23 → 24. v1.4 starts at 25 post-launch.

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
| 12. Solo Mode Infrastructure | v1.2 | 3/3 | Complete    | 2026-03-25 |
| 13. Match Detail View | v1.2 | 3/3 | Complete    | 2026-03-27 |
| 14. Personal Learnings Journal | v1.2 | 3/3 | Complete   | 2026-03-27 |
| 15. Goals & LP History | v1.2 | 3/3 | Complete (close-out via P16) | 2026-05-06 |
| 16. Phase 15 Close-out | v1.3 | 3/3 | Complete    | 2026-05-07 |
| 17. UI Consolidation | v1.3 | 10/10 | Complete | 2026-05-11 |
| 18. Region Variants | v1.3 | 10/10 | Complete    | 2026-05-22 |
| 18.1 SSR Theme Injection (gap closure) | v1.3 | 3/3 | Complete    | 2026-05-25 |
| 19. Bug-Report Widget | v1.3 | 0/? | Not started (renumbered from 18) | - |
| 20. Production Hardening | v1.3 | 0/? | Not started (renumbered from 19) | - |
| 20.1 Closed-Beta Access Gate | v1.3 | 0/? | Not started (renumbered from 19.1) | - |
| 21. Deploy Infra (CAX11) | v1.3 | 0/? | Not started (renumbered from 20) | - |
| 22. Compliance & Transparency | v1.3 | 0/? | Not started (renumbered from 21) | - |
| 23. Pre-Launch Full Review | v1.3 | 0/? | Not started (renumbered from 22) | - |
| 24. Soft Launch + Feedback Loop | v1.3 | 0/? | Not started (renumbered from 23) | - |
| 25. Draft Integration (deferred) | v1.4 | 0/? | Deferred (renumbered from 24) | - |
