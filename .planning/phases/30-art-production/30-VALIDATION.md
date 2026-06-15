---
phase: 30
slug: art-production
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-15
---

# Phase 30 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `30-RESEARCH.md` § Validation Architecture (SC1–SC4).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Playwright (TypeScript) + `cargo test` (Rust unit) |
| **Config file** | `e2e/playwright.config.ts` |
| **Quick run command** | `cd e2e && npx playwright test room-solo-journey.spec.ts hydration-no-panic.spec.ts` |
| **Full suite command** | `cd e2e && npx playwright test` |
| **Estimated runtime** | ~90 seconds (targeted) / ~5 min (full e2e) |

Code-change quick check: `cargo check --features ssr` and
`cargo check --features hydrate --target wasm32-unknown-unknown`.
Art-commit quick check: `find public/img/rooms -name "*.webp" -size +120k` (must be empty).

---

## Sampling Rate

- **After every task commit:** Code tasks → `cargo check --features ssr`. Art tasks → per-file budget `find ... -size +120k` (empty) + per-room/region aggregate ≤ 400 KB.
- **After every plan wave:** `cd e2e && npx playwright test room-solo-journey.spec.ts hydration-no-panic.spec.ts`
- **Before `/gsd:verify-work`:** Full CI green (`check` + `style_guardrails` + new `asset_budget`) + Playwright full suite green
- **Max feedback latency:** ~90 seconds (targeted e2e)

---

## Per-Task Verification Map

> The planner populates concrete task IDs. This map records the SC→test contract the
> tasks must satisfy. (Pipeline/art tasks are human-in-loop offline; their "test" is the
> budget gate + AI-IMAGES.md provenance check, not a unit test.)

| SC | Requirement | Secure/Correct Behavior | Test Type | Automated Command | File Exists |
|----|-------------|-------------------------|-----------|-------------------|-------------|
| SC1 | ART-01 | PROJECT.md logs "Path A" before first generation | manual gate + grep | `grep -q "Path A" .planning/PROJECT.md` | ❌ W0 |
| SC1 | ART-01 | AI-IMAGES.md prompts carry no Riot proper nouns | shell grep (neg) | `! grep -iE "demacia\|pandemonium\|splash art style" .planning/assets/AI-IMAGES.md` | ❌ W0 |
| SC2 | ART-03 | Regions read visually distinct (not color-swap) | manual visual QA | side-by-side screenshots both regions | manual |
| SC2 | ART-02/03 | No hydration panic in either region | e2e (existing) | `cd e2e && npx playwright test hydration-no-panic.spec.ts` | ✅ |
| SC3 | ART-02 | BiRefNet halo-free (validated on 1 plane first) | manual visual QA | open vs `#070912`, parallax max amplitude | manual |
| SC3 | ART-02 | No file > 120 KB | CI (new) | `find public/img/rooms -name "*.webp" -size +120k` empty | ❌ W0 |
| SC3 | ART-02 | Per-room/region aggregate ≤ 400 KB | CI (new) | `du` aggregate per region ≤ 409600 | ❌ W0 |
| SC3 | ART-02 | Every asset logged (seed/compute/size) | shell count | `grep -c "Seed:" AI-IMAGES.md` ≥ webp count | manual |
| SC4 | ART-02 | Correct per-tier overlay visible, others hidden | e2e (new) | `npx playwright test -g "per-tier overlay"` | ❌ W0 |
| SC4 | ART-02 | Per-tier code change compiles both targets | unit/compile | `cargo check --features ssr` + hydrate target | ✅ |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Legal gate task: `/consult legal` run + `PROJECT.md` Key Decisions "Path A" entry (`checkpoint:human-verify`)
- [ ] `AI-IMAGES.md` prompt rewrite for Path A (Aurelian Reach / Sundered Hollow vocabulary; faction names surfaced for user sign-off)
- [ ] Per-tier overlay CSS block in `input.css` (12 axes × 3 tiers; only tier-1/2/3 overrides, tier-0 = base `opacity:0`)
- [ ] D-10 per-tier overlay markup in all 4 room files (3 stacked `<img>` per axis, ROOM-03 compliant)
- [ ] Placeholder WebP files for new `overlay-{axis}-{1,2,3}.webp` paths so code compiles + smoke passes before real art
- [ ] New `asset_budget` CI job step in `.github/workflows/ci.yml` (per-file + per-room/region aggregate)
- [ ] E2e per-tier overlay seed-and-assert test (at least Solo Journey) — closes the Phase 27 silent-fail lesson

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Region visual distinctness (SC2) | ART-03 | Aesthetic judgment — "not a color swap" cannot be asserted by code | Capture each room in both regions; a reviewer blind to prompts must correctly label region from visual content alone |
| BiRefNet halo absence (SC3) | ART-02 | Sub-pixel edge artifact only visible during parallax motion | Open plane vs `#070912`, move mouse to corner (max amplitude), inspect mid/near edges at 1× zoom — no white/grey ring |
| Faction name sign-off (D-03) | ART-01 | User approval decision | Surface "Aurelian Reach" / "Sundered Hollow" for explicit approval before prompts finalized |
| `/consult legal` run (SC1) | ART-01 | Human legal-review session | Run `/consult legal`; record basis + date in PROJECT.md Key Decisions |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies (offline pipeline tasks gated by budget + provenance checks)
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (CSS selectors, CI step, e2e test, placeholder assets)
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
