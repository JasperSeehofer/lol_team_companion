---
phase: 30-art-production
plan: "03"
subsystem: offline-pipeline
tags: [art-production, shell-scripts, ci, budget-enforcement, bfl-eu, fal-ai, imagemagick]
dependency_graph:
  requires: ["30-01", "30-02"]
  provides: ["30-05", "30-06"]
  affects: [".github/workflows/ci.yml", "scripts/"]
tech_stack:
  added: []
  patterns:
    - "BFL EU FLUX.2 [pro] submit+poll+download via curl/jq (x-key header, polling_url, result.sample)"
    - "fal.ai BiRefNet v2 with fal-storage upload (T-30-08 mitigation — no leaky public host)"
    - "ImageMagick alpha-preserving WebP encode (-quality 80 -define webp:alpha-quality=100)"
    - "FAIL=0 accumulator pattern for multi-violation budget gate scripts"
    - "GitHub Actions ::error:: annotation in budget CI job"
key_files:
  created:
    - scripts/gen-room-art.sh
    - scripts/remove-bg.sh
    - scripts/encode-webp.sh
    - scripts/verify-budget.sh
  modified:
    - .github/workflows/ci.yml
decisions:
  - "gen-room-art.sh accepts prompt via --prompt flag OR PROMPT env var (flag takes precedence); documented in header comment for Plans 05/06 invocation"
  - "remove-bg.sh uses fal storage REST upload endpoint (https://fal.run/storage/upload, Authorization: Key $FAL_KEY) before calling BiRefNet — avoids expiring/attacker-controllable public host (Pitfall 7 / T-30-08)"
  - "verify-budget.sh uses maxdepth 1 for per-region aggregate so only files directly in the region dir are counted (not nested subdirs), matching the flat asset layout from Plan 02"
  - "asset_budget CI job runs parallel to check and style_guardrails (no needs:), additive only — existing jobs unchanged"
metrics:
  duration: "~12 minutes"
  completed: "2026-06-15"
  tasks_completed: 3
  tasks_total: 3
  files_created: 4
  files_modified: 1
---

# Phase 30 Plan 03: Art Production Pipeline Scripts + CI Budget Gate Summary

Scripts and CI job that close the D-11/D-12 gap: a complete offline art-production pipeline (BFL EU FLUX generation, fal BiRefNet background removal, ImageMagick WebP encoding, budget verification) plus a CI `asset_budget` job that enforces the per-room/region 400 KB aggregate cap on every push.

## What Was Built

Four executable shell scripts in `scripts/` and one additive CI job in `.github/workflows/ci.yml`:

| Artifact | Purpose | Key implementation detail |
|----------|---------|--------------------------|
| `scripts/gen-room-art.sh` | BFL EU FLUX.2 [pro] submit + poll + download | POST `api.eu.bfl.ai/v1/flux-2-pro` with `x-key` header; polls `polling_url` until `status==Ready`; downloads `result.sample` PNG to `/tmp/<room>/<region>/` |
| `scripts/remove-bg.sh` | fal.ai BiRefNet v2 background removal | Uploads PNG to fal storage first (T-30-08 mitigation); then POSTs `fal-ai/birefnet/v2` with `model: heavy`; downloads transparent PNG |
| `scripts/encode-webp.sh` | ImageMagick PNG → alpha WebP | `magick -quality 80 -define webp:alpha-quality=100`; optional q68 second pass if over 120 KB cap; resize-to-1600x900 fallback documented (Pitfall 3) |
| `scripts/verify-budget.sh` | Per-file 120 KB + per-region 400 KB gate | `FAIL=0` accumulator; `find -size +120k` per-file; `du -b + awk` aggregate per region dir; exits 0 on current placeholder tree |
| `.github/workflows/ci.yml` | `asset_budget` CI job (D-11) | Parallel job (no `needs:`); same logic as verify-budget.sh; `::error::` annotations; checks all 8 room/region dirs |

## Script Invocation Reference (for Plans 05/06)

**gen-room-art.sh** — two prompt-passing mechanisms:
```bash
# Via --prompt flag (recommended for scripted invocation)
./scripts/gen-room-art.sh --prompt "painterly scene..." solo-journey demacia plane-sky 7421337

# Via PROMPT env var
PROMPT="painterly scene..." ./scripts/gen-room-art.sh solo-journey demacia plane-sky 7421337
```
Output: `/tmp/<room>/<region>/<plane-name>.png` — pass to remove-bg.sh or encode-webp.sh.

**remove-bg.sh** — fal storage upload endpoint:
```bash
./scripts/remove-bg.sh /tmp/solo-journey/demacia/plane-mid.png /tmp/solo-journey/demacia/plane-mid-transparent.png
```
Upload endpoint: `POST https://fal.run/storage/upload` with `Authorization: Key $FAL_KEY` and `Content-Type: image/png`. Returns `access_url` or `url` field containing the fal-storage URL passed to BiRefNet.

**Typical end-to-end pipeline call sequence (Plans 05/06):**
```bash
# 1. Generate
./scripts/gen-room-art.sh --prompt "$PROMPT" solo-journey demacia plane-mid $SEED

# 2. Remove background (skip for sky planes — they are opaque)
./scripts/remove-bg.sh /tmp/solo-journey/demacia/plane-mid.png /tmp/solo-journey/demacia/plane-mid-transparent.png

# 3. Encode to WebP
./scripts/encode-webp.sh /tmp/solo-journey/demacia/plane-mid-transparent.png \
    public/img/rooms/solo-journey/demacia/plane-mid.webp

# 4. Check budget
./scripts/verify-budget.sh public/img/rooms/solo-journey/demacia/
```

## Threat Mitigations Implemented

| Threat ID | Mitigation |
|-----------|-----------|
| T-30-06 | `${BFL_KEY:?}` / `${FAL_KEY:?}` guards; keys never echoed to stdout or logs |
| T-30-07 | EU-policy exception documented in remove-bg.sh header; decorative art, no personal data |
| T-30-08 | remove-bg.sh uploads to fal storage before BiRefNet call — no expiring/public host in input path |
| T-30-09 | verify-budget.sh + asset_budget CI job enforce 120 KB per-file + 409600-byte aggregate |

## Verification Results

```
verify-budget.sh on current placeholder tree:
  PASS: no individual file exceeds 120 KB
  OK: solo-journey/demacia/   (17194 bytes, 16 KB)
  OK: solo-journey/pandemonium/ (14312 bytes, 13 KB)
  OK: war-room/demacia/       (14446 bytes, 14 KB)
  OK: war-room/pandemonium/   (11664 bytes, 11 KB)
  OK: library/demacia/        (14036 bytes, 13 KB)
  OK: library/pandemonium/    (11330 bytes, 11 KB)
  OK: throne/demacia/         (18446 bytes, 18 KB)
  OK: throne/pandemonium/     (12942 bytes, 12 KB)
  All budget checks passed.

CI YAML parse: jobs = ['check', 'style_guardrails', 'asset_budget'] — all present
```

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. These are tooling scripts, not UI components; no placeholder data flows to rendering.

## Threat Flags

None. No new network endpoints, auth paths, or schema changes introduced in the server binary. All new surfaces are offline scripts (no server exposure).

## Self-Check: PASSED

- [x] `scripts/gen-room-art.sh` exists and is executable
- [x] `scripts/remove-bg.sh` exists and is executable
- [x] `scripts/encode-webp.sh` exists and is executable
- [x] `scripts/verify-budget.sh` exists and is executable
- [x] `.github/workflows/ci.yml` contains `asset_budget` job alongside `check` and `style_guardrails`
- [x] Commits: 4cd429d (Task 1), 54506ca (Task 2), 01a643f (Task 3)
