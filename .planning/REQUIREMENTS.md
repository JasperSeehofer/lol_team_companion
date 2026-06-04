# Requirements: LoL Team Companion — v1.4 Immersive Companion

**Defined:** 2026-06-04
**Core Value:** Features talk to each other — and now the app is a *place* worth spending time in: an explorable environment that visibly grows as you put in the work.

> Prior-milestone requirements (v1.0–v1.3) are recorded as Validated in PROJECT.md
> and preserved in git history. This file scopes the **v1.4** milestone only.

## v1.4 Requirements

### Room System Foundations

- [ ] **ROOM-01**: Team-scoped growth state — `get_team_growth_counts` + `TeamGrowthState` (mirrors the user-scoped pair) drives team-room environments by tier
- [ ] **ROOM-02**: `AtmosphereRoot` generalized to arbitrary named growth axes, plus a reusable `RoomSurface` tab component (signal + `<Show>`, ARIA tablist, deep-linkable `?tab=`, instant switch with no atmosphere remount)
- [ ] **ROOM-03**: Anti-structural-branching hydration rule documented in `CLAUDE.md` + `.claude/rules/leptos-patterns.md` before any new room is built

### Rooms

- [ ] **ROOM-04**: War Room hosts draft, tree-drafter, game-plan, opponents, and ban-priority as surfaces under one parallax atmosphere
- [ ] **ROOM-05**: Hall of Records / Library hosts stats, match detail, analytics, post-game, and team learnings as surfaces
- [ ] **ROOM-06**: Throne / Home room hosts identity, team dashboard, and solo progression
- [ ] **ROOM-07**: IA healing — previously-siloed features are reachable in their intent room (opponents in War Room, learnings in Library); legacy flat routes still resolve as deep links
- [ ] **ROOM-08**: QuickNav command palette available app-wide (every room + flat route) with a proper focus trap; nav IA reflects the rooms

### Accessibility (closed-beta blockers)

- [ ] **A11Y-01**: A runtime `prefers-reduced-motion` change-listener freezes/restores parallax without a page reload (not just a mount-time check)
- [ ] **A11Y-02**: Every room passes the per-route hydration-no-panic canary in both regions, keeps `focus-visible` rings, and holds WCAG AA text contrast over the art (scrims)

### Art Production

- [ ] **ART-01**: Original world & art identity — Arcane-inspired and evocative of LoL's regions but our own, with NO trademarked place-names or "in the style of [Riot product]" in committed assets; `AI-IMAGES.md` prompts revised; `/consult legal` check gates the first real generation run
- [ ] **ART-02**: Layer-separated environment art produced per room (first region) via the Black Forest Labs EU FLUX endpoint + BiRefNet layer separation, under a CI-enforced ~400 KB/room/region budget
- [ ] **ART-03**: Second region art produced for all rooms once the first region is locked

### Re-incorporated Launch Work (absorbed from v1.3)

- [ ] **LNCH-01**: Production hardening — env-driven site address, secure cookies behind HTTPS, prod log config + rotation, Riot API timeout/retry/cache, dynamic Data Dragon patch resolution, graceful shutdown, nightly SurrealKV snapshot
- [ ] **LNCH-02**: Closed-beta access gate (invite-only)
- [ ] **LNCH-03**: Deploy infrastructure on the shared Hetzner CAX11
- [ ] **LNCH-04**: Compliance & transparency — data-processing + AI-art disclosure / transparency table (the deferred Phase-19 SC-8)
- [ ] **LNCH-05**: Pre-launch full review (security, a11y, performance, regression)
- [ ] **LNCH-06**: Soft launch + feedback loop
- [ ] **LNCH-07**: Draft integration — match → draft → game-plan → review chain (deferred XFEAT-03/04)

## Future Requirements (deferred beyond v1.4)

- **ROOM-F1**: Discrete per-tier art swaps (distinct images per growth tier) instead of single-overlay opacity ramps
- **ROOM-F2**: Room-to-room View Transitions and per-region ambient motion
- **ROOM-F3**: Mobile-responsive room variants (desktop-first holds for v1.4)
- **INTL-03–06 / FLOW-01**: ban recommendations, stats-informed pick scoring, opponent lesson recall, draft-outcome correlation, game-day wizard

## Out of Scope

| Feature | Reason |
|---------|--------|
| Literal Riot IP in AI art (faction names, "splash-art style" prompts) | Legal/IP risk; Riot stance hostile to AI art — original-but-inspired identity instead (ART-01) |
| Ambient audio / music in rooms | Scope; revisit post-launch |
| Forced spatial navigation (click art objects to move) | Anti-feature — kills power-user speed; QuickNav + tabs are the fast path |
| Explicit XP bars / streaks / login rewards on rooms | Gimmicky gamification; growth maps to real work only |
| 3D / WebGL engine (Three.js etc.) | CSS+web-sys parallax is sufficient; no heavy JS dep |

## Traceability

Filled during roadmap creation (2026-06-04).

| Requirement | Phase | Status |
|-------------|-------|--------|
| ROOM-01 | Phase 26 — Room System Foundations | Pending |
| ROOM-02 | Phase 26 — Room System Foundations | Pending |
| ROOM-03 | Phase 26 — Room System Foundations | Pending |
| A11Y-01 | Phase 26 — Room System Foundations | Pending |
| ROOM-04 | Phase 27 — War Room | Pending |
| ROOM-05 | Phase 28 — Library & IA Healing | Pending |
| ROOM-07 | Phase 28 — Library & IA Healing | Pending |
| ROOM-06 | Phase 29 — Throne, QuickNav & Room A11Y | Pending |
| ROOM-08 | Phase 29 — Throne, QuickNav & Room A11Y | Pending |
| A11Y-02 | Phase 29 — Throne, QuickNav & Room A11Y | Pending |
| ART-01 | Phase 30 — Art Production | Pending |
| ART-02 | Phase 30 — Art Production | Pending |
| ART-03 | Phase 30 — Art Production | Pending |
| LNCH-01 | Phase 31 — Production Hardening & Beta Gate | Pending |
| LNCH-02 | Phase 31 — Production Hardening & Beta Gate | Pending |
| LNCH-03 | Phase 32 — Deploy Infrastructure | Pending |
| LNCH-04 | Phase 33 — Compliance & Pre-Launch Review | Pending |
| LNCH-05 | Phase 33 — Compliance & Pre-Launch Review | Pending |
| LNCH-06 | Phase 34 — Soft Launch & Draft Integration | Pending |
| LNCH-07 | Phase 34 — Soft Launch & Draft Integration | Pending |

**Coverage:**
- v1.4 requirements: 20 total (8 ROOM + 2 A11Y + 3 ART + 7 LNCH)
- Mapped to phases: 20/20
- Unmapped: 0

> **Coverage note:** The requirements document header states "18 total" but the actual count is 20 (ROOM-01..08 = 8, A11Y-01..02 = 2, ART-01..03 = 3, LNCH-01..07 = 7). All 20 are mapped. The "18" figure should be corrected to "20" in a future update.

---
*Requirements defined: 2026-06-04*
*Last updated: 2026-06-04 — traceability filled during roadmap creation (Phases 26-34)*
