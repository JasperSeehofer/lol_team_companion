# Phase 24 — Soft Launch + Feedback Loop (SPEC seed)

**Status:** SEED — produced by the v1.2 → v1.3 pivot on 2026-05-06. Renumbered from Phase 23 → 24 on 2026-05-11 when Phase 18 (Region Variants) was inserted. Run `/gsd-spec-phase 24` to expand.

**Milestone:** v1.3 Launch Readiness (final phase)

## Goal (one sentence)

Real users on real traffic, with the bug-report inbox driving the v1.4 backlog.

## Launch shape (DECIDED in pivot)

**Closed beta via named-friends invite list.** Public URL but distributed only to a small list (~5-15 people). Soft entry; no marketing push. Riot Developer Portal "Personal API Key" approval required (handled in Phase 22). DSE + Impressum live (Phase 22). Vault Tier-A section live (Phase 22).

## In-scope

### Deploy
1. Production deploy to the chosen domain (handled by Phase 21 `just deploy`)
2. Final pre-flight smoke per Phase 23 LAUNCH-GATE.md commands
3. Verify all `/legal/*` routes serve correctly on the production domain (not just localhost)

### Onboarding
4. Curate named-friends invite list (5-15 people) — names recorded in `23-INVITE-LIST.md` (private, not committed)
5. Optional: build a lightweight invite-code mechanism if registration is to be gated (otherwise leave registration open and rely on the obscure URL — decide in discuss-phase)
6. Send personal invites with: domain, registration link, brief "what to look for" message, and a clear pointer to the bug-report button
7. Set expectation: weekly fix cadence; you'll see things change

### First-week monitoring
8. Daily check of `bug_report` table (or the auto-exported `.planning/INBOX/bug-reports.md` file) — log who has signed up, who has reported, any HIGH severity bugs
9. Hot-fix any HIGH-severity bug within 24 hours of report
10. Track via systemd journal / Caddy logs for any 5xx errors, slow requests, OOM kills

### Triage
11. End of week 1: run `/gsd-inbox` (or manual triage in CLAUDE.md-style memo) to grade reports:
   - **A**: HIGH severity / quick-win — fix in v1.3 patch release
   - **B**: feature-request worth seeding into v1.4 phases
   - **C**: nice-to-have / deferred
12. Update `.planning/ROADMAP.md` with v1.4 phase seeds based on the A/B grades
13. Mark v1.3 closed in MILESTONES.md once first-week stability is confirmed (no HIGH-severity production incidents in 7 consecutive days)

## Out of scope

- Public marketing launch (deferred — that's a future milestone, post-Riot approval and post-validation)
- Paid signups / payments integration (deferred)
- Multi-region (Riot regions other than EUW) (deferred)
- Translations (deferred)
- Analytics tooling (deferred — bug-report inbox + manual feedback is sufficient for closed beta)

## Success criteria (verify with `/gsd-verify-work 24`)

1. Production deploy live at chosen domain; HTTPS valid; pages render
2. Named-friends invite list onboarded (at least 5 people register)
3. Bug-report widget exercised by at least 3 users in the first week (verifies the feedback loop actually works end-to-end)
4. `.planning/INBOX/bug-reports.md` populated and read by the next Claude session — proven by a v1.4 phase seeded directly from an inbox item
5. Weekly `/gsd-inbox` triage produces a graded backlog
6. No HIGH-severity production incidents in 7 consecutive days
7. `MILESTONES.md` updated with v1.3 completion summary

## Required reading before discuss-phase

1. `.planning/LAUNCH-GATE.md` (Phase 23 output)
2. `infra/RUNBOOK.md` (Phase 21 output)
3. `[[charter]]`, `[[values-charter]]` — what kind of launch we want (no dark patterns, transparency, scientific integrity in claims)
4. `[[cross-project-memory]]` — feynman-lookup launch-week observations (if any)

## Plans

TBD — produced by `/gsd-plan-phase 24`. Likely 1-2 plans:
- 24-01: deploy + verify live + send invites
- 24-02: week-1 monitoring + triage + v1.4 seeding

## Depends on

- Phase 23 complete (LAUNCH-GATE.md all-green)

## Hard NOs during launch

- No `--no-verify` on git operations (per project guardrails — applies to launch hot-fixes too)
- No silent data collection beyond what's in the DSE (per `[[guardrails#G-13]]`)
- No dark patterns in the invite/registration flow (per `[[guardrails#G-10]]`)
- No public claim of "Riot endorsed" anywhere (Riot ToS — independent project)

---

This SPEC was seeded by the pivot.
