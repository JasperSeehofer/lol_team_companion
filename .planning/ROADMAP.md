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
- [ ] **Phase 17: UI Consolidation** - Run `/gsd-ui-phase` for UI-SPEC; Claude Design primary pass; Open-Design fills missing surfaces; retroactive `/gsd-ui-review` after implementation.
- [ ] **Phase 18: Bug-Report Widget** - Element-click → modal (URL + semantic label + text + bug/wishlist); SurrealDB `bug_report` table; auto-export task writes `.planning/INBOX/bug-reports.md` on every server start.
- [ ] **Phase 19: Production Hardening** - Externalize site-addr (env-driven); secure cookies behind HTTPS; prod log config + rotation; Riot API timeout/retry/cache; dynamic Data Dragon patch; graceful shutdown; nightly SurrealKV snapshot.
- [ ] **Phase 19.1: Closed-Beta Access Gate (INSERTED 2026-05-07)** - Gate registration behind an invite mechanism so the deployed binary is URL-reachable but only the named-friends list can sign up. Login + legal pages remain public.
- [ ] **Phase 20: Deploy Infra (shared CAX11)** - Cross-compile via `cargo zigbuild --target aarch64-unknown-linux-gnu.2.36`; new systemd unit at port 3001; Caddyfile stanza; `just deploy` recipe; pre-deploy smoke + post-deploy WASM-200 + server-fn check (per feynman 2026-04-16 incident).
- [ ] **Phase 21: Compliance & Transparency** - Pick + register TLD (flag values charter for non-EU); DSE + Impressum (§5 DDG, no Steuernummer); Tier-A transparency in `[[lol-team-companion]]` vault entity; G-01..G-13 CI sweep; Riot Developer Portal application.
- [ ] **Phase 22: Pre-Launch Full Review** - `/gsd-audit-milestone v1.3` + full `/gsd-code-review` + `/gsd-secure-phase` + `/gsd-ui-review` + `/ultrareview` + `/consult security`; fix HIGH findings; produce `.planning/LAUNCH-GATE.md`.
- [ ] **Phase 23: Soft Launch + Feedback Loop** - Deploy; named-friends invite list; monitor `bug_report` for 1 week; weekly `/gsd-inbox` triage; backlog grooming feeds v1.4.

### 🔜 v1.4 Draft Integration & Post-Launch Backlog (Deferred)

**Milestone Goal:** Once v1.3 is shipped and producing real-user match data, ship the deferred Draft Integration alongside the highest-priority items surfaced from the launch bug-report inbox.

- [ ] **Phase 24: Draft Integration** (was old Phase 16) - Personal matchup notes surface in draft Intel sidebar; champion pool cards display solo-history win rate badges. Depends on Phase 14 + Phase 15 — both already complete.
- [ ] Phase 25+: Inbox-driven backlog from v1.3 launch (defined post-launch).

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
**Plans**: TBD
**UI hint**: yes (entire phase is UI work)

### Phase 18: Bug-Report Widget
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

### Phase 19: Production Hardening
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

### Phase 19.1: Closed-Beta Access Gate (INSERTED 2026-05-07)
**Goal**: Public registration is gated by an invite mechanism so that the deployed v1.3 binary is reachable by URL but only the named-friends list can complete account creation
**Depends on**: Phase 19
**Decisions to lock in plan-phase**:
  - **Gate model**: choose one of (A) single shared `BETA_INVITE_CODE` env var, (B) per-user `invite_code` SurrealDB table minted by an admin, or (C) email allow-list. Leaning (B) because it gives per-friend revocation, tracks who consumed which code, and surfaces a real "X of N invites used" signal during the beta. Plan-phase locks the choice.
  - **Gate location**: registration server fn only. Login still works for already-registered users; homepage and `/legal/impressum` + `/legal/datenschutz` (per Phase 21) stay reachable without a code so the legal pages can be cited externally.
  - **Admin role**: a single admin user (first registered, or flagged via `ADMIN_USER_EMAIL` env) sees an admin-only "Mint invite" page listing issued codes and their consumed status. Non-admin users get a 404 on the route, not a 403, to avoid leaking the route's existence.
  - **Removal path**: when the gate is dropped (likely v1.4 or post-beta), the `invite_code` table can stay (audit trail) but the registration fn check becomes feature-flag-controlled. No migrations needed to drop the gate.
  - **No anti-bot beyond the code**: a 6-12 char alphanumeric code from a CSPRNG is sufficient for ≤ 30-account beta. No CAPTCHA, no rate-limiting work in this phase (Phase 19 already adds Riot-side timeouts; abuse handling for registration deferred to v1.4 if needed).
**Success Criteria**:
  1. Registration server fn rejects sign-ups without a valid, unconsumed invite code and returns a clean form-level error
  2. Admin can mint invite codes via an admin-only route; on consumption each code is marked with the consuming `user_id` and timestamp
  3. Login, homepage, and `/legal/*` routes remain reachable without an invite code
  4. New `invite_code` table appears in `schema.surql` (or chosen alternative is documented) with appropriate field constraints
  5. e2e smoke test covers: invalid code → reject; valid code → registration succeeds and code is marked consumed; reuse of consumed code → reject
  6. Bug-report widget (Phase 18) still functions for invited users — no regression on already-shipped surfaces
**Plans**: TBD
**UI hint**: yes (registration form change + minimal admin-mint page)

### Phase 20: Deploy Infrastructure (Shared CAX11)
**Goal**: Get the production binary running on feynman-lookup's existing Hetzner CAX11 alongside the existing service, served via Caddy at the new domain
**Depends on**: Phase 19, Phase 19.1
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

### Phase 21: Compliance & Transparency
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
  4. Riot Developer Portal application status documented in `.planning/phases/21-compliance-transparency/21-RIOT-STATUS.md`
**Plans**: TBD
**UI hint**: minimal (legal pages only)

### Phase 22: Pre-Launch Full Review
**Goal**: A comprehensive multi-tool review of the entire v1.3 milestone before any public-facing deploy, producing a single LAUNCH-GATE.md that is the go/no-go document
**Depends on**: Phase 17, 18, 19, 20, 21
**Review chain**:
  1. `/gsd-audit-milestone v1.3` — does v1.3 deliver on its goal?
  2. `/gsd-code-review` (full repo, not just diff) — bugs, security, quality
  3. `/gsd-secure-phase` — threat model verification
  4. `/gsd-ui-review` — 6-pillar visual audit on production build
  5. `/ultrareview` — multi-agent cloud review of the launch branch
  6. `/consult security "v1.3 launch checklist"` — vault security advisor pass
**Success Criteria**:
  1. All HIGH findings from any tool are fixed before Phase 23
  2. `.planning/LAUNCH-GATE.md` exists with one row per check (PASS/FAIL/WAIVED with rationale)
  3. Post-deploy smoke commands documented and dry-run on staging
**Plans**: TBD
**UI hint**: no

### Phase 23: Soft Launch + Feedback Loop
**Goal**: Real users on real traffic, with the bug-report inbox driving the v1.4 backlog
**Depends on**: Phase 22
**Success Criteria**:
  1. Production deploy live at chosen domain
  2. Named-friends invite list (5-15 people) onboarded with personal invite links
  3. Bug-report widget exercised by at least 3 users in the first week
  4. `.planning/INBOX/bug-reports.md` populated and read by the next Claude session
  5. Weekly `/gsd-inbox` triage produces a graded backlog seeding v1.4 phases
  6. No HIGH-severity production incidents in the first 7 days
**Plans**: TBD
**UI hint**: no

### Phase 24: Draft Integration (deferred from old Phase 16, now in v1.4)
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
Phases execute in numeric order. v1.3: 16 → 17 → 18 → 19 → 19.1 → 20 → 21 → 22 → 23. v1.4 starts at 24 post-launch.

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
| 17. UI Consolidation | v1.3 | 0/? | Not started | - |
| 18. Bug-Report Widget | v1.3 | 0/? | Not started | - |
| 19. Production Hardening | v1.3 | 0/? | Not started | - |
| 19.1 Closed-Beta Access Gate | v1.3 | 0/? | Not started (INSERTED 2026-05-07) | - |
| 20. Deploy Infra (CAX11) | v1.3 | 0/? | Not started | - |
| 21. Compliance & Transparency | v1.3 | 0/? | Not started | - |
| 22. Pre-Launch Full Review | v1.3 | 0/? | Not started | - |
| 23. Soft Launch + Feedback Loop | v1.3 | 0/? | Not started | - |
| 24. Draft Integration (deferred) | v1.4 | 0/? | Deferred | - |
