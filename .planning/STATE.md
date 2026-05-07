---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Launch Readiness
status: ready_to_plan
stopped_at: Phase 16 close-out complete; v1.2 shipped
last_updated: "2026-05-07T10:08:57.000Z"
progress:
  total_phases: 12
  completed_phases: 5
  total_plans: 15
  completed_plans: 14
  percent: 42
---

## Current Position

Milestone: v1.3 Launch Readiness
Phase: 17
Plan: Not started

Next: `/gsd-spec-phase 16` (or `/gsd-discuss-phase 16` if SPEC unnecessary), then plan/execute the close-out

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-25)

**Core value:** Features talk to each other — stats inform drafts, drafts flow into game plans, post-game lessons surface when preparing the next game.
**Current focus:** Phase 16 — phase15-closeout
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

## Blockers/Concerns

- [Phase 12]: Region support scope — decide EUW-only vs. multi-region before writing solo sync; retrofitting touches every Riot API call
- [Phase 15]: LP delta boundary logic across tier/division changes (promos, relegation, Master+ no-division) needs validation before implementing LP graph

## Session Continuity

Last session: 2026-05-06T15:55:31.393Z
Stopped at: Phase 16 context gathered
Resume file: .planning/phases/16-phase15-closeout/16-CONTEXT.md
Next command: `/gsd-spec-phase 16` or `/gsd-discuss-phase 16` to start the Phase 15 close-out.

## v1.3 Decisions (baked in pivot 2026-05-06)

- Phase 15 critical bugs (CR-01 WASM hover, CR-02 Rule 40) → Phase 16 close-out, not pre-pivot inline fix
- Hetzner: reuse feynman-lookup's CAX11; new domain (e.g. `lol-companion.gg` placeholder); Caddy stanza → port 3001
- Bug-report widget captures: page URL + semantic element label + free text + bug/wishlist toggle (no screenshots, no CSS selectors)
- Launch shape: closed beta via named-friends invite list (no public self-serve registration in v1.3)
- Old Phase 16 (Draft Integration) → deferred to v1.4 Phase 24

## Blind-spot Inventory (carried into Phase 21 / Phase 22)

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
15. Backup: feynman has none — Phase 19 SurrealKV snapshot must work standalone
