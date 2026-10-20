# M166 — REPORT-R1: Wednesday prep (Scout half-C runner, navgrid tool)

Date: 2026-10-20. Branches: `mission/M166-scout-half-c-and-navgrid-prep` (part B and this report) and the
local `scout/M166-halfc` (part A, worktree `/home/ops/Repositories/lol_team_companion-wt/M166-scout-halfc`).
Neither branch has been pushed by this item. No VERIFY file was written here.

## Budget

- Declared 4.0 M (cap 6.0 M, max-wall 10h). The estimate is **about 0.7 to 0.9 M in total**. The subagents
  metered roughly 0.12 M (Sonnet upstream fetch), 0.25 M (part A), 0.18 M (part B), plus this item and the
  orchestration. `76-meter.sh` is authoritative. This figure is an estimate, not a meter reading.
- The STOP RULE never triggered. A1 to A5 and B1 to B5 all shipped, including B4. Only the real-file run itself
  moves to R2.
- The wall clock suffered from host pressure. In this item, `pytest tools/scout` took 62 minutes (80 passed;
  it is seconds on an idle host). The `pytest tools/navgrid` re-run was stopped by the harness for low memory
  (about 1 GB available, load around 5), so the part B count below is the executor's run. The verifier
  re-runs both suites anyway.

## Least-confident decisions

**Part A (Scout)**

1. `--out` without `--findings` writes the findings only to `OUT/`. This avoids clobbering the committed
   template rendering `docs/scout-findings.md`.
2. The positions-A gap and detection rate are judged with death gaps removed: a gap that starts within 3 s of
   that champion's `CHAMPION_KILL`. The raw figures are reported beside them.
3. Edges of the route rule:
   - B GO with real-time-only recording, and A GO or PARTIAL, gives "A+B (B on key windows)".
   - A NO-GO with B GO in real time gives B-first, with a note.
   - A PARTIAL with B not GO gives no-go, with an override hint.
   - "Real time only" means under 1.5 game-s per wall-s, or no frames.
4. Enemy wards: the minimised data cannot tell which team owns a ward, so verdicts are given per blue/red, and
   `--enemy-team` narrows the eye-check strip.
5. Hand checks with fewer than 5 rows use the same 4/5 share. In the synthetic run, wards at 3/4 (75 %) are
   NO-GO. Recall truth is purchase bursts outside the first 60 s and not within 120 s after a death.
6. `session score` also copies minimap frames ±2 s around at most 6 `WARD_PLACED` per team, from
   `%TEMP%\scout-frames` into `frames_minimap/` (at most 45 MB). Stale scratch frames from an earlier run are a
   risk.
7. The A1 and A2 commits are not green on their own (they import A3 files). The tip is green.
8. `metadata.matchId` is kept in `match.json`, because `track_minimap --match` reads it. `send` refuses a run
   that fails the manifest check.
9. `portal-question.md` still says "not sent", while the mission page says it was sent on 2026-10-17. The
   template records both.

**Part B (navgrid)**

1. Structure check: the centroid of an isolated blocking cluster (at most 13×13 cells), or a bit-4 cluster
   within 2 cells.
2. Lane check: uses the converter's nearest-lane nibble. This catches a swapped axis, but the nibble values are
   unverified on 16.20, and the nexus turrets are not lane-checked.
3. Walkable means none of `0x46`. MapgeoAddon's reading is "not `0x02`".
4. Zstd-chunked entries are handled by a heuristic, without the subchunk table.
5. The hint-node count is inferred as N·(4N+4). The size of the height grid is only noted.
6. The bit-4 "supported" thresholds (80 %, and 50 % within 300 units) are my own choice. They are reported but
   do not gate.
7. B4 upsamples 8× with a 3-pixel box. At 4×, 5–6 % of the brush area was lost; at 8× the synthetic brush keeps
   97 %.
8. pc-run searches install roots on drives C to F, with and without `Game\`, on Windows only.

**Orchestrator**

- Licence: `tools/navgrid/` is **AGPL-3.0-only**, not the "or-later" of scope B5. This follows the pre-flight
  licence rule. The converter repo carries the plain AGPL-3.0 text, "or later" appears only in the appendix
  template, and there are no file headers. `pyproject.toml` says `license = "AGPL-3.0-only"`.

## Test output

Part B, `tools/navgrid/.venv/bin/python -m pytest tools/navgrid -q` in this worktree: **60 passed** (the
executor's run; this item's re-run was stopped for host memory, see Budget).

```
$ git log --oneline main..HEAD
70b1274 feat(navgrid): B4 vectorise to GeoJSON
08a0541 chore(navgrid): B5 licence, notices, guards
5d3e2f0 feat(navgrid): B3 pc-run and validate
63587be feat(navgrid): B2 v7.1 navgrid parser and synthetic grid
ce0f7a8 feat(navgrid): B1 WAD v3 reader
```

Part A, `tools/scout/.venv/bin/python -m pytest tools/scout -q` in `M166-scout-halfc`, re-run in this item:

```
80 passed in 3716.37s (1:01:56)
```

That is half A's 34 tests plus 46 new ones.

```
$ git log --oneline origin/mission/M156-scout-replay-spike..HEAD
27d2cae feat(scout): A5 minimised match.json and timeline; weights pre-check not run
6855205 feat(scout): A4 session driver and the one-line CHECKLIST
011ec5a feat(scout): A3 proposed go/no-go thresholds and the route rule
0aaf38b docs(scout): A2 findings template and its empty rendering in docs/scout-findings.md
04f01f9 feat(scout): A1 half-C runner (python -m scout.halfc) and the synthetic run dir
```

## Runner on the synthetic run

`python -m scout.halfc tools/scout/tests/fixtures/run_synth --out <tmp>`:

```
run run_synth: manifest PASS; A GO; B GO; wards blue NO-GO, red NO-GO; recalls GO; route A+B
facts.md
halfc-summary.json
handcheck_merged.csv
plots/fact6_eye_check.png
plots/minimap/error_per_champion.png
plots/minimap/overlay_00000.png
plots/minimap/overlay_00180.png
plots/minimap/overlay_00360.png
plots/topdown/error_per_champion.png
plots/topdown/overlay_00120.png
plots/topdown/overlay_00180.png
plots/topdown/overlay_00240.png
plots/topdown_strip.png
plots/wards_precision_recall.png
score/handcheck_minimap.csv
score/handcheck_topdown.csv
score/score_minimap.json
score/score_topdown.json
scout-findings.md
```

Key numbers in the summary:

- **Manifest**: 0.43 MB, and nothing is missing.
- **Positions A**: median 51.45, detection 1.0 alive-adjusted (raw 0.9947, with one 19 s gap at a death).
- **Positions B**: median 66.9, bar hit rate 0.9.
- **Wards**:
  - Blue: precision 1.0, recall 1.0, but the hand check is 3/4, so NO-GO.
  - Red: precision 0.667, so NO-GO.
- **Recalls**: 1.0 / 1.0, hand check 1/1.
- **Eye check**: 4 events, 0 without a frame.

## Navgrid file names and hashes

`map11.bin.json` (CommunityDragon latest, 16.20) says the following:

- The Default skin's `mNavigationMesh` is `ASSETS/Maps/NavGrid/Map11/AIPath_SRX_2.aimesh_ngrid`.
- Across the skins:
  - 21 skins point to `AIPath_SRX_2`;
  - 14 to `AIPath.aimesh_ngrid`;
  - 1 to `AIPath_SRX`;
  - 1 to `AIPath_NPE_1`.
- There are 8 `.ngrid_overlay` files, which are out of scope.

The candidate list, in order, with the xxhash64 of the lower-cased path (`navgrid/wad.py`):

| # | path | xxhash64 |
|---|---|---|
| 1 | `assets/maps/navgrid/map11/aipath_srx_2.aimesh_ngrid` | `0xE08D20957D2E2A3A` |
| 2 | `assets/maps/navgrid/map11/aipath_srx_3.aimesh_ngrid` | `0x9D67B1C0B806E23E` |
| 3 | `assets/maps/navgrid/map11/aipath_srx.aimesh_ngrid` | `0xB7B2B65287ED2102` |
| 4 | `assets/maps/navgrid/map11/aipath.aimesh_ngrid` | `0x59C8E0541C0960CE` |

`aipath_npe_1` (`0xF0F5EC0956387220`, the tutorial) is excluded. If no candidate matches, the reader falls back
to content sniffing (a v7 header, cell size 50, counts 200–400). `manifest.json` records which method found
the file.

Upstream commits (in `tools/navgrid/NOTICE.md`):

- MapgeoAddon: `8d2f8bd9f57d064c47a5b9c5c7e6e55597412d76` (MIT).
- LoL-NGRID-converter: `92943ed2b2d5e82c86d680e69f53f247c89aefee` (AGPL-3.0).

## Flag table (both readings)

| bit | plan | MapgeoAddon | converter |
|---|---|---|---|
| 0x1 | brush | HAS_GRASS | Brush |
| 0x2 | wall | NOT_PASSABLE | Wall |
| 0x4 | structure wall | BUSY | StructureWall |
| 0x8 | – | TARGETTED | Unobserved8 |
| 0x10 | – | MARKED | Unobserved16 |
| 0x20 | – | PATHED_ON | Unobserved32 |
| 0x40 | transparent wall | SEE_THROUGH | TransparentWall |
| 0x80 | – | vision-related | Unknown128 |
| 0x100 | always visible | – | AlwaysVisible |
| 0x200 | – | – | Unknown512 |
| 0x400 | gate blue | – | BlueTeamOnly |
| 0x800 | gate red | – | RedTeamOnly |
| 0x1000 | gate vision | – | NeutralZoneVisibility |

`BLOCKING = 0x46` (wall, structure wall, transparent wall). R2 settles the meanings on the real file.

## Weights pre-check

**Not checked.** `free -g` showed 2 GB available, below the 3 GB floor (df 17 GB free, load 2.37). No class
count, missing-champion list or sha256 was produced. CHECKLIST step 3 keeps the Google Drive download, and
the Wednesday sheet marks step 9 (the weights scp) as skipped.

## CHECKLIST marks (`tools/scout/CHECKLIST.md`, still one page)

All 13 steps are **PC-only**, because without the weights check nothing could be pre-done on brain:

1. Get the branch.
2. Venv and run dir.
3. Weights (Google Drive fallback).
4. Replay API.
5. Open the replay.
6. Probe.
7. Minimap track.
8. Top-down track.
9. Timeline (dev key at `Read-Host`).
10. Score (plots optional).
11. Hand check.
12. Check.
13. Send to brain.

The appendix "Without the driver" keeps the old commands.

## Other part A findings

- **Scout licence label**: AGPL-3.0-or-later is correct. The LICENSE is the AGPLv3 text, and `pyproject` and
  `__init__` say "or later". "GPL-3.0" is pyLoL's vendored licence, which the helper page mixed in. M156's
  files were not relabelled.
- **Run data on M156's branch**: none. `git ls-files` and `git log` for `tools/scout/runs` are both empty.
  `tools/scout/runs/` and `**/*.pt` are now gitignored, with a guard test.
- **Brain inbox**: `/home/ops/.local/share/scout/inbox/` was created.
- **Push guard**: in the part A worktree, upstream tracking was removed and `pushRemote` points at a remote
  that does not exist, so a stray push fails.
- **Path assumption**: the mission page's `../lol_team_companion-wt/` path assumes the main checkout as the
  working directory. The actual part A worktree is `/home/ops/Repositories/lol_team_companion-wt/M166-scout-halfc`.
- **`RGAPI-` strings**: dummy strings already exist in `.env.example`, `ci.yml` and
  `.claude/hooks/session-start.sh`. They are not this round's files. There are none in `tools/`.
- **Cargo**: no cargo command ran in this round.

## Wednesday command list

**Unchanged** from the mission page, steps 1–22 word for word. The sheet `publish-round-r1/report.html` adds:

- the host-key prompt on the first scp;
- `navgrid\previous\` on a second pc-run;
- step 9 is skipped this time;
- "Wait for Riot" skips steps 2–5;
- the dev key at `Read-Host`;
- scp only, never a git push;
- the CHECKLIST fallback.

## Moved to R2

The real-file run itself. R2 runs after Wednesday as a separate spawn (about 0.6 M):

1. Vectorise and validate the real grid on brain.
2. Add the run's `BUILDING_KILL` positions.
3. Report the numbers (numbers only, nothing derived committed).
4. Re-raise the merge chip.

## After PASS (the overseer, before Wednesday noon)

1. `git -C /home/ops/Repositories/lol_team_companion push origin scout/M166-halfc:mission/M156-scout-replay-spike`
   (non-forced). The part A worktree's own push is disabled on purpose.
2. `git -C /home/ops/Repositories/lol_team_companion-wt/M156-scout-replay-spike pull --ff-only`.
3. Point M156's half-C dispatch at `python -m scout.halfc`.
4. Push `mission/M166-scout-half-c-and-navgrid-prep` (branch only), so that Jasper can switch to it on the PC.
   This item did not push it.

## Open flags carried from the page

- `base-branch:` is not implemented in `71 spawn`.
- M156's meter wall reads 64h against a 12h max-wall. Check it before M156 resumes.
- M156's `tools/scout/runs/` is not in main (card question 3).
