---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Launch Readiness
status: phase_complete
stopped_at: Phase 18.2 complete — UI-18.1-HYDRATE-01 resolved
last_updated: "2026-05-26T12:05:00.000Z"
progress:
  total_phases: 17
  completed_phases: 9
  total_plans: 48
  completed_plans: 43
  percent: 53
---

## Current Position

Milestone: v1.3 Launch Readiness
Phase: 18.2 complete (5/5 plans, verification passed 9/9 must-haves)
Plan: —

Next: Phase 19 (Bug-Report Widget) — now fully unblocked. UI-18-RUNTIME-01 resolved by Phase 18.1 (SSR theme injection); UI-18.1-HYDRATE-01 resolved by Phase 18.2 (cfg(hydrate) provide_context in App::App() reads <html data-theme> via web_sys before Routes — commit e67c7da; CompanionSigil prop refactor — commit 50c70a8; 19-test regression gate — commit ed82453). Resume command: `/gsd:plan-phase 19` (next milestone phase).

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-25)

**Core value:** Features talk to each other — stats inform drafts, drafts flow into game plans, post-game lessons surface when preparing the next game.
**Current focus:** Phase 18.2 complete — UI-18.1-HYDRATE-01 resolved; Phase 19 (Bug-Report Widget) unblocked
**v1.3 launch goal:** Closed-beta deploy on shared Hetzner CAX11 with in-app bug-report widget that auto-exports to a Claude-readable inbox.

## Decisions

- [Roadmap]: Phase 12 must establish `synced_by` on `match` table and `mode` on `user` table before any other solo work — data model correctness gates all subsequent phases
- [Roadmap]: Match timeline fetch is on-demand + DB-cached only (never eager during sync) — rate limit protection
- [Roadmap]: `personal_learning` is a separate table from `post_game_learning` — solo learnings must not mix with team-scoped records
- [Roadmap]: Mode toggle persisted in DB user record, not WASM signal — survives hard navigation on login/logout
- [Roadmap]: Phase 15 (Goals & LP History) depends on Phase 12 only, not Phase 13 or 14 — can be sequenced after either
- [Phase 12]: EUW fallback for opponent Riot API calls: opponent players have no associated user region, hardcode EUW as safe fallback for get_puuid/fetch_player_intel/fetch_player_champions
- [Phase 12]: store_matches_with_synced_by wrapper: preserved backward-compatible store_matches() API; solo sync calls store_matches_with_synced_by directly with user ID
- [Phase 12]: PlayerMatchStats lacks queue_id/game_duration: match rows show champion+KDA+CS only; queue filter works via DB WHERE clause
- [Phase 13-match-detail-view]: Store participants as single JSON string in match_detail table — avoids 10-row JOIN with no query benefits
- [Phase 13-match-detail-view]: Timeline event markers use index into full timeline_events vec as stable key to avoid index drift when filters toggle
- [Phase 14-personal-learnings-journal]: personal_learning is user-scoped (not team-scoped) — matches D-01 design decision; created_at deserialized as Option<String> consistent with other models
- [Phase 14]: Pre-clone Strings before view! macro when same String is moved into a closure AND also needed by a later component prop
- [Phase 14]: Resource<Result<Vec<...>>> type used for LearningCard prop to match Leptos resource typing
- [Phase 14]: current_date_short() returns static Today — auto-title is user-editable so approximate is acceptable
- [Phase 16]: v1.2 Solo Mode & Match Intelligence shipped 2026-05-07; close-out resolved WR-01 (refetch hoisting) and WR-02 (dead get_personal_goals removed); 15-REVIEW.md fully reconciled; second-pass review found 0 new HIGH/Critical
- [Phase 17]: UI consolidation complete 2026-05-11; demacia/pandemonium themes adopted (color-only swap — structural variants scoped as new Phase 18); self-hosted fonts (G-01 zero hits); 4-hub IA; 22 visual-regression baselines committed; 6-pillar audit PASS-with-deferred; FLUX placeholder backgrounds with AI-IMAGES.md reproducibility log; Open-Design seeded as cross-repo design system
- [Phase 18.1]: UI-18-RUNTIME-01 resolved 2026-05-25 by SSR theme injection middleware (axum middleware + cookie write + per-request InitialTheme context). Runtime sweep proved Pandemonium structural branches activate at SSR (28 markers across 7 routes; 18 Demacia negative-space markers; 3/3 utility REQ-7 PASS; 3/3 mode-toggle D-04 PASS). New finding UI-18.1-HYDRATE-01 (WASM hydration panic on Pandemonium pages, tachys mod.rs:217 unwrap on None) recommended for Phase 18.2.
- [Phase 18.2]: UI-18.1-HYDRATE-01 resolved 2026-05-26. Root cause: SSR provided `InitialTheme` via `leptos_routes_with_context` but the hydrate path had no equivalent — `use_context::<InitialTheme>()` returned `None` on WASM → demacia fallback → SSR/WASM structural prop divergence at every region-branching descendant. Fix: `cfg(feature="hydrate")` `provide_context(InitialTheme)` block in `App::App()` reading `<html data-theme>` via `web_sys` BEFORE `view!` instantiates `<Routes>` (commit e67c7da). Anti-pattern fix: CompanionSigil refactored to take `region: String` prop, removing internal `use_context` read (commit 50c70a8). Regression gate: `e2e/tests/hydration-no-panic.spec.ts` — 19 tests (14-row panic-sweep matrix × 7 routes × 2 regions + 5 D-13 interactivity triad), commit ed82453. Sweep: 0/14 panics post-fix; Demacia parity preserved; 121/121 unit tests pass. Tooling lesson: agent-browser text-mode `errors`/`console` silently truncates structured WASM panic entries — `--json` mode is mandatory for hydration panic capture (this masked the real symmetric-bug nature for two waves). Plan-structure deviation: fix actually landed in Wave 4 (expanded scope) rather than Wave 3 because Plan 03's deepened investigation revealed the Plan 01–02 baseline error.

## Blockers/Concerns

- [Phase 12]: Region support scope — decide EUW-only vs. multi-region before writing solo sync; retrofitting touches every Riot API call
- [Phase 15]: LP delta boundary logic across tier/division changes (promos, relegation, Master+ no-division) needs validation before implementing LP graph

## Session Continuity

Last session: 2026-05-26T12:05:00.000Z
Stopped at: Phase 18.2 complete — UI-18.1-HYDRATE-01 resolved; verification passed 9/9
Resume file: .planning/phases/18.2-pandemonium-hydration-reconciliation-gap-closure-for-ui-18-1/18.2-VERIFICATION.md
Next command: `/gsd:plan-phase 19` (next milestone phase — Bug-Report Widget; both UI-18-RUNTIME-01 + UI-18.1-HYDRATE-01 now resolved, so all Phase 18 dependencies are clear).

## v1.3 Decisions (baked in pivot 2026-05-06)

- Phase 15 critical bugs (CR-01 WASM hover, CR-02 Rule 40) → Phase 16 close-out, not pre-pivot inline fix
- Hetzner: reuse feynman-lookup's CAX11; new domain (e.g. `lol-companion.gg` placeholder); Caddy stanza → port 3001
- Bug-report widget captures: page URL + semantic element label + free text + bug/wishlist toggle (no screenshots, no CSS selectors)
- Launch shape: closed beta via named-friends invite list (no public self-serve registration in v1.3)
- Old Phase 16 (Draft Integration) → deferred to v1.4 (now Phase 25 after Region Variants insertion on 2026-05-11)
- [2026-05-11] Region Variants inserted as new Phase 18; subsequent v1.3 phases renumbered (18→19 Bug-Report Widget, 19→20 Prod Hardening, 19.1→20.1 Access Gate, 20→21 Deploy, 21→22 Compliance, 22→23 Pre-Launch, 23→24 Soft Launch)

## Blind-spot Inventory (carried into Phase 22 / Phase 23)

1. Riot Developer Portal application status
2. G-13 Tier-A transparency section missing in vault `[[lol-team-companion]]`
3. Impressum: §5 DDG (not §5 TMG); no Steuernummer
4. DSE 4-section structure (logfiles, registration, hosting, betroffenenrechte)
5. Riot puuid → US processor — needs DSE Tier-A entry
6. G-01 sweep — no Google Fonts CDN in deployed HTML
7. G-12 sweep — no `outline:none` without ring replacement
8. Post-deploy WASM-200 check (per feynman 2026-04-16 incident)
9. Post-deploy server-fn check (catches binary-arch mismatch)
10. Caddy CSP + Leptos 0.8 nonces — `'unsafe-inline'` with TODO for nonce middleware
11. Cross-compile glibc pin: `--target aarch64-unknown-linux-gnu.2.36`
12. `cargo-leptos` `_bg.wasm` rename in deploy recipe
13. Two-layer firewall (Hetzner Cloud + on-host ufw)
14. Vault staleness: `[[lol-team-companion]]`, `[[product-portfolio]]`, `[[self-employment-action-plan]]`
15. Backup: feynman has none — Phase 20 SurrealKV snapshot must work standalone
