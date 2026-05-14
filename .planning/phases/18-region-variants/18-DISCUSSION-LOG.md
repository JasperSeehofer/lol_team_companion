# Phase 18: Region Variants - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 18-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-14
**Phase:** 18-region-variants
**Areas discussed:** Primitive file layout, Mode persistence, Baseline filename convention, Design workflow for 4 siblings

---

## Primitive file layout

### Question 1: Where do the ~24 region-aware primitives live in `src/components/`?

| Option | Description | Selected |
|--------|-------------|----------|
| Per-category split under `region/` | 7 sibling modules (typography, ornaments, layout, controls, data_viz, solo, chrome). Cleanest. ~3-5 primitives per file. | ✓ |
| Single `region.rs` module | One file with all primitives. Simpler imports; gets large (~1000 LOC). | |
| Extend `ornaments.rs` in place | Add new primitives to existing file. Risks ballooning past 1500 LOC. | |

**User's choice:** Per-category split under `region/`.
**Notes:** None.

### Question 2: Typography module name (since `type` is a Rust reserved keyword)?

| Option | Description | Selected |
|--------|-------------|----------|
| `typography.rs` | Most descriptive — covers Display, Imperial, H, Eyebrow, Mono, Glitch. | ✓ |
| `text.rs` | Shorter; slightly less precise (Glitch is more treatment than text). | |
| `r#type.rs` | Rust raw identifier prefix; works but ugly. | |

**User's choice:** `typography.rs`.
**Notes:** None.

### Question 3: What happens to existing ornaments and the new region/ornaments.rs?

| Option | Description | Selected |
|--------|-------------|----------|
| Move ALL ornaments into `region/ornaments.rs` | `FleurDeLis`, `Crown`, `CompanionSigil` move alongside `GiltCorner`, `HeraldicDivider`, `RiotTape`. Old `src/components/ornaments.rs` deleted. ~7 import sites updated. | ✓ |
| Keep old `ornaments.rs`; add `region/ornaments.rs` | Two ornaments files coexist; existing imports unchanged. Slight cognitive overhead. | |
| Move existing 3 into `region/chrome.rs` | Group identity ornaments with champion/role chrome. | |

**User's choice:** Move ALL ornaments into `region/ornaments.rs`; delete old file.
**Notes:** None.

---

## Mode persistence

### Question 1: How should mode selection persist for /draft, /team/dashboard, /solo?

| Option | Description | Selected |
|--------|-------------|----------|
| 3 separate string columns on `user` | Matches existing `user.theme` + `user.mode` precedent. Clean queries, type-safe. | ✓ |
| Single JSON `mode_prefs` column | One `object<string, string>` field. Flexible; less type-safe. | |
| localStorage only | No DB plumbing; doesn't survive cross-device login. Conflicts with project precedent. | |
| URL query param | Stateless, shareable; no session persistence. | |

**User's choice:** 3 separate string columns on user (matches user.theme/user.mode precedent).
**Notes:** None.

### Question 2: How are mode DEFAULTS chosen when a user first hits one of these routes?

| Option | Description | Selected |
|--------|-------------|----------|
| Hard schema defaults; region-agnostic | `carousel` / `dashboard` / `constellation` regardless of region. | |
| Region-coupled defaults via `'auto'` sentinel + resolver | Store `'auto'`; resolver picks per-region. Explicit user pick overrides. | (implied) ✓ |
| Defer to plan-phase | Capture as Claude's Discretion. | |

**User's choice:** Region-coupled (option B implied by user clarification).
**Notes:** User responded: *"the region selection is global and should be an immersive experience overall. if that answers the question"* — confirming that region is the global immersive shell and that mode defaults follow region grammar. Locked as region-coupled via `'auto'` sentinel + page-entry resolver.

---

## Baseline filename convention

### Pre-question: Should baselines be gitignored given the PNG churn concern?

**User raised this as a meta question during the AskUserQuestion turn (selected no option; left a note: "should this be gitignored as ther will be many png files in the future?").**

Resolution discussed inline before re-asking:
- Baselines must be **committed**, not gitignored — CI compares against committed baselines as the reference. Without them the visual-regression spec has nothing to diff against.
- Phase 17 already committed 22 PNGs flat (~3-5 MB total) without LFS — pattern is established.
- Churn is bounded because baselines only change when intentionally updated (`--update-snapshots`).
- Future mitigations if churn grows: sibling repo referenced by hash; Git LFS scoped to `*-snapshots/**`; perceptual SSIM diff. All deferrable.

This rationale is captured in CONTEXT.md `<deferred>` and D-08.

### Question 1: Given baselines stay committed, which naming/folder scheme?

| Option | Description | Selected |
|--------|-------------|----------|
| Subfolders per scoped route | Scoped routes nest under route-named folders; utility routes stay flat at root. | ✓ |
| Flat with hyphens | All baselines flat at root; route-region-mode chained. Matches Phase 17 style. | |
| Separate spec files per route group | Split visual-regression.spec.ts into per-route spec files. Bigger refactor. | |

**User's choice:** Subfolders per scoped route.
**Notes:** None.

---

## Design workflow for 4 new siblings

### Question 1: How should the 4 missing region siblings be designed before porting to Leptos?

| Option | Description | Selected |
|--------|-------------|----------|
| Extend Claude Design source | Run another Claude Design pass; re-export bundle; port from there. Single source of truth maintained. | ✓ |
| /gsd-sketch HTML mockups | Throwaway HTML in `.planning/sketches/`. Project-native, fast iteration, captured as findings skill. | |
| Direct Leptos + agent-browser iteration | Skip intermediate mockup. Stub component, screenshot, iterate. Fastest if design eye is trusted. | |
| Inline HTML drafts in `.planning/design-drafts/` | Static HTML files under `.planning/`. Lighter than /gsd-sketch; reviewable in PRs. | |

**User's choice:** Extend Claude Design source.
**Notes:** User-driven manual Claude Design session is the prerequisite. 18-07 is GATED on the 4 sibling JSX files existing in `.local-design-source/.../{draft-ledger,solo-journal,solo-forge,team-game-day-brief}/{demacia,pandemonium}.jsx` before porting starts.

---

## Claude's Discretion

Captured in CONTEXT.md `<decisions>` "Claude's Discretion" subsection:

- Project-local skill for the region-aware-primitive pattern.
- Mode-toggle UI shape (segmented control / tabs / dropdown / region-aware variants).
- `/solo` Pandemonium default (`forge` vs `journal`) — confirm in 18-08.
- `Region` enum vs `String`-typed `region` props.
- `/match/:id` and `/post-game` sharing one component vs separate files.
- Whether to restyle the existing `theme_toggle.rs`.
- File location for `PageLoading` / `PageEmpty` components.

---

## Deferred Ideas

Captured in CONTEXT.md `<deferred>` section:

- `Region` enum cleanup (deferrable to post-launch if not adopted in 18-01).
- Mode-reset UI (`'auto'` re-set affordance) — v1.4+.
- Per-region mode-toggle component variants — gsd-planner discretion.
- Git LFS for baseline PNGs — only if churn exceeds ~100 MB.
- Perceptual SSIM diff in place of pixelDiff — v1.4+.
- Mobile responsiveness, animated theme transitions — out of scope.
- Sibling repo for visual-regression baselines.
- Live Match overlay restyle — inherits region primitives once the feature ships in v1.4/1.5.
