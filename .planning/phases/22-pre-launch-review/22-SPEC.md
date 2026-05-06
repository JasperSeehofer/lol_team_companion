# Phase 22 — Pre-Launch Full Review (SPEC seed)

**Status:** SEED — produced by the v1.2 → v1.3 pivot on 2026-05-06. Run `/gsd-spec-phase 22` to expand.

**Milestone:** v1.3 Launch Readiness

## Goal (one sentence)

A comprehensive multi-tool review of the entire v1.3 milestone before any public-facing deploy, producing a single LAUNCH-GATE.md that is the go/no-go document for Phase 23.

## Why this phase exists

Each prior phase has its own narrow review (`/gsd-code-review N`, `/gsd-ui-review N`). Before launch, the *whole milestone* needs a holistic pass that catches inter-phase issues, regressions, and blind spots that single-phase reviews miss. The `[[cross-project-incidents]]` log shows that feynman-lookup discovered 4 distinct production-only failures (placeholder Impressum, WASM 404, CSP block, binary-arch mismatch) that all individual phase reviews had missed. This phase exists to catch those.

## In-scope

### Review chain (run in order)
1. **`/gsd-audit-milestone v1.3`** — does v1.3 deliver on its goal? Cross-phase coherence check.
2. **`/gsd-code-review`** on the full repo (not just the latest phase diff). Bugs, security, quality.
3. **`/gsd-secure-phase`** retroactive on phases 16-21. Threat-model verification.
4. **`/gsd-ui-review`** on the production build. 6-pillar visual audit.
5. **`/ultrareview`** — multi-agent cloud review of the launch branch. (User-triggered, billed.)
6. **`/consult security "v1.3 launch checklist"`** — vault security advisor pass.
7. **`/consult legal "v1.3 launch readiness"`** — vault legal advisor pass on Impressum/DSE rendered output.

### LAUNCH-GATE.md production
8. After each review tool above, append findings to `.planning/LAUNCH-GATE.md`. Each row:
   - Check name
   - Status: PASS / FAIL / WAIVED
   - Evidence (link / hash / quote)
   - If FAIL → fix is required before Phase 23
   - If WAIVED → user must explicitly approve with rationale
9. Pre-flight smoke commands (used during deploy in Phase 23) documented and dry-run on staging or local prod-like build.

### Specific items to verify (curated from blind-spot list)
10. No `fonts.googleapis.com` / `fonts.gstatic.com` in deployed HTML (G-01)
11. No `§5 TMG` anywhere (G-03)
12. No Steuernummer line in Impressum (G-04)
13. No `outline:none` without `ring-*` replacement (G-12)
14. Vault Tier-A section exists in `wiki/entities/lol-team-companion.md` (G-13)
15. `/healthz`, server-fn endpoint, and `_bg.wasm` all return 200 in a staging-equivalent test
16. Cookie `Secure` flag confirmed on prod (curl `-I` to inspect Set-Cookie)
17. CSP headers present on all responses
18. SurrealKV backup script tested round-trip (Phase 19 deliverable; verify here)

## Out of scope

- New feature work (this is a review phase only)
- Performance benchmarking (deferred — closed beta load is small)
- Penetration testing (deferred to post-launch; the GSD security suite + vault security consult is the minimum)

## Success criteria (verify with `/gsd-verify-work 22`)

1. All 7 review steps run; outputs preserved in `.planning/phases/22-pre-launch-review/`
2. `.planning/LAUNCH-GATE.md` has every row populated; no UNKNOWN status remains
3. All FAIL findings have a fix commit referenced; all WAIVED findings have explicit user approval recorded
4. No HIGH-severity findings remain unfixed
5. Deploy smoke commands documented and verified working

## Required reading before discuss-phase

1. `.planning/ROADMAP.md` (the v1.3 milestone block — what we're auditing against)
2. The 6 prior-phase SPECs (16-21) — what each promised
3. `[[guardrails]]` — the canonical hard-NO list
4. `[[cross-project-incidents]]` — production-only failure modes to specifically check
5. `[[agent-weaknesses]]` — W-PRE (Premature closure) is the highest-risk failure mode for this phase: do not declare PASS without runtime verification on a running server

## Plans

TBD — produced by `/gsd-plan-phase 22`. Likely 2 plans:
- 22-01: Run the review chain + populate LAUNCH-GATE.md
- 22-02: Fix all FAIL findings; re-run failed checks until green

## Depends on

- Phase 17, 18, 19, 20, 21 all complete

---

This SPEC was seeded by the pivot.
