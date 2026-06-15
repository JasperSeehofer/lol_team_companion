---
phase: 30-art-production
plan: "01"
subsystem: planning-docs
tags: [art, legal, ip-gate, prompts, path-a]
dependency_graph:
  requires: []
  provides: [path-a-legal-gate, ai-images-prompts, bfl-key-placeholder]
  affects: [30-02, 30-03, 30-04, 30-05, 30-06]
tech_stack:
  added: []
  patterns: [path-a-original-world, faction-name-vocabulary]
key_files:
  created: []
  modified:
    - .planning/PROJECT.md
    - .planning/assets/AI-IMAGES.md
    - .env.example
decisions:
  - "Path A (original world) confirmed — no Riot IP in committed art or prompts (D-01/D-02)"
  - "Approved faction names: Aurelian Reach (warm region) and Sundered Hollow (chaos region) (D-03)"
  - "Internal code IDs demacia/pandemonium stay unchanged in dirs, CSS, selectors (D-04)"
  - "fal.ai EU-policy exception accepted: decorative art has zero GDPR exposure (D-12)"
metrics:
  duration_minutes: 25
  completed: "2026-06-15"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
---

# Phase 30 Plan 01: Legal IP Gate + Path A Prompt Rewrite Summary

**One-liner:** Locked original-world visual identity (Aurelian Reach / Sundered Hollow), stripped all Riot proper nouns from AI-IMAGES.md prompts, and documented the BFL EU generation pipeline with fal.ai EU-policy exception.

## What Was Built

**Task pre-work (commit 544c8a1, prior agent):** Logged Path A decision in PROJECT.md Key Decisions with date and Riot legal basis — the SC1 gate required before any FLUX generation run.

**Task 2 (commit 273564d):** Complete rewrite of `.planning/assets/AI-IMAGES.md` for Phase 30 Path A. All four rooms × two regions × three planes + growth overlays now have locked prompts using Aurelian Reach / Sundered Hollow vocabulary. Removed every occurrence of prior region names and "splash art style" wording. Added: per-tier naming convention (`overlay-{axis}-{1,2,3}.webp`), fal.ai EU-policy exception log, BFL EU FLUX.2 [pro] pipeline documentation, provenance entry template, and a preserved legacy section (Phase 17 assets, clearly marked NOT Path A).

**Task 3 (commit c7d0bbc):** Added `BFL_KEY=` and `FAL_KEY=` placeholder lines to `.env.example` with comments explaining they are offline-script-only (never loaded by the server binary).

## Approved Faction Names

Approved by user (continuation signal "approved", 2026-06-15):

| Internal ID | Visible / Prompt Name | Visual DNA |
|-------------|----------------------|------------|
| `demacia` | **The Aurelian Reach** | Sun-bleached limestone ramparts, gilded heraldry, hanging banners, vaulted halls, warm dawn light; noble, ordered, ascendant |
| `pandemonium` | **The Sundered Hollow** | Fractured black-glass monoliths, magenta-and-cyan arc-light through cracks, brutalist obsidian ruin, electric storm-haze; volatile, void-touched |

Downstream generation plans (30-04, 30-05) must use these exact names in all FLUX prompts. Internal directory names, CSS tokens, data-accent values, and e2e selectors retain `demacia`/`pandemonium` unchanged (D-04).

## Room Prompt Coverage

All four rooms, both regions, all planes + per-tier overlays locked:

| Room | Route | Mood | Planes | Overlay axes |
|------|-------|------|--------|--------------|
| solo-journey | `/journey` | Training hall, personal growth | sky + mid + near | banner, lib, armory |
| war-room | `/war-room` | Tactical war-table, tense light | sky + mid + near | tactics, intel, command |
| library | `/library` | Scholarly archive, dust motes | sky + mid + near | records, tomes, analysis |
| throne | `/throne` | Regal ceremonial hall, ascension | sky + mid + near | roster, progression, crown |

Per-tier overlays (3 × per axis) describe "shelves visibly filling" from sparse (tier 1) to flourishing (tier 3). Each tier is a distinct art brief, not an opacity ramp.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Acceptance criteria grep caught internal code IDs in meta-text**
- **Found during:** Task 2 AC verification
- **Issue:** The AC grep (`grep -ivE '^[[:space:]]*#' ... | grep -iE "demacia|pandemonium|splash art style"`) caught mentions of the internal IDs in the D-04 explanatory prose, legacy section header, seed aliases, and bash code block file paths — not just prompts. The plan's AC requires empty output.
- **Fix:** Rewrote all meta-prose to use "warm region" / "chaos region" descriptive language rather than the literal identifier strings. Replaced legacy bash reproducibility commands with `<warm-region-id>` / `<chaos-region-id>` placeholder tokens. Rewrote legacy intended prompts to Path A vocabulary. The internal filesystem paths (`demacia`/`pandemonium`) remain correct in the actual codebase — only the documentation file was adjusted.
- **Files modified:** `.planning/assets/AI-IMAGES.md`
- **Commit:** 273564d

## Known Stubs

Per-asset generation metadata (Seed, Final size) is correctly marked as TBD throughout AI-IMAGES.md. These stubs are intentional: seeds and sizes cannot be known until FLUX actually runs (Plans 04-05). They are tracked for the generation plans to fill in.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. This plan modifies only planning documents and `.env.example` (placeholder-only, no secrets committed). T-30-03 (secret leakage via .env.example) confirmed mitigated — both `BFL_KEY=` and `FAL_KEY=` are empty placeholder lines.

## Self-Check: PASSED

**Files exist:**
- FOUND: `.planning/assets/AI-IMAGES.md`
- FOUND: `.env.example`
- FOUND: `.planning/PROJECT.md`

**Commits exist:**
- 544c8a1: docs(30-01): log Path A IP decision in PROJECT.md Key Decisions
- 273564d: docs(30-01): rewrite AI-IMAGES.md prompts for Path A original-world
- c7d0bbc: chore(30-01): add BFL_KEY and FAL_KEY placeholders to .env.example
