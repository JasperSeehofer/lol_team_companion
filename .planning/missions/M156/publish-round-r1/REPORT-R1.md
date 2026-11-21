Status: half A done, awaiting Jasper's run

# M156 Scout spike: REPORT-R1 (half A)

Branch `mission/M156-scout-replay-spike`, pushed to origin. Date 2026-10-17.

## Budget

- Declared 3.0 M tokens, hard cap 4.0 M.
- Subagent totals so far, roughly: fetch 85 k, build 191 k, drafting 81 k, finishing (this pass) about 120 k. The orchestrator's own use comes on top.
- These are estimates. The meter is the overseer's figure, not this one.
- Half A is well under the declared budget. Half C has most of it left.

## Least-confident decisions

1. **Vendor, not fork, and only a reference copy.** pyLoL has no detector code, no weights and no minimap-to-world mapping in its repository. So only `scraper.py`, `LICENSE` and `README.md` are copied, unmodified, and nothing imports them. "Vendor vs fork" became "vendor for reference". Scout's own Replay API driver is written fresh from the field list in League Director (`api.py`, Apache-2.0).
2. **YOLOv12 weights are unverified.** They are a 227 MB Google Drive file linked from pyLoL's README. The runtime version they need, their class list and their accuracy are not published. The README's 92.2 mAP belongs to an older hosted model. Scout reads `model.names` after loading and reports missing champions. Without the weights, a template detector (Data Dragon icons) is the fallback.
3. **No minimap ward detector.** Wards come from the ward health bars in the top-down view. Fact 6 (do enemy wards show on the minimap with fog off?) is read by eye from the saved minimap frames in half C.
4. **HSV bar parameters and the bar-to-unit offset are placeholders.** They pass on the synthetic frames only. Half C tunes them on Jasper's top-down frames.
5. **Top-down camera preset is unverified.** Straight down over the map centre, rotation y = 90, field of view 19, height from the probe (default 57447, pyLoL's value). The checklist has a keystroke fallback (Ctrl+Shift+Y) if the API camera cannot see the whole map.
6. **Map constants (checked in this pass).**
   - All 22 turret coordinates match the Bayes Esports tower table exactly. The live host did not resolve, so I read the Wayback Machine copy of 9 December 2024: https://web.archive.org/web/20241209153455/https://docs.bayesesports.com/docs-data-portal/misc/tower_coords
   - The repo fixture's Match-V5 positions match too: `BUILDING_KILL` (10504, 1029) and `TURRET_PLATE_DESTROYED` (13866, 4505).
   - I added the six inhibitor coordinates from the same table as extra anchors (`INHIBITORS` in `calibrate.py`, also in the anchors template).
   - The fountain centres were estimates, (400, 420) and (14340, 14390). They are now the Match-V5 frame-0 spawn positions from the fixture, (554, 581) and (14302, 14387).
   - The two nexus centres are not in any source I found. They keep the "verify on run" mark, as does the fountain radius (1000).
7. **Champion names from `/liveclientdata/playerlist` in a replay are unverified.** If the list is empty in replay mode, the checklist tells Jasper to fetch `match.json` first and pass `--match`.
8. **Frame caps were lowered** to stay under 50 MB per run: 24 minimap frames, 6 top-down frames, 8 probe screenshots. The mission page allows up to 40 PNGs.
9. **pyLoL's licence is GPL-3.0, not AGPL-3.0** as the mission page says. Its `LICENSE` file is GPL-3.0; its `setup.py` says MIT. The `LICENSE` file governs. AGPL-3.0 may include GPL-3.0 code (GPLv3 §13), so Scout stays AGPL-3.0.
10. **"AAGPL-3.0" was a typo for AGPL-3.0.** ultralytics is AGPL-3.0, checked in the wheel's metadata, on PyPI and in its GitHub LICENSE.
11. **The venv on brain needed `uv venv --seed`.** The system Python has no `ensurepip`. Jasper's PC uses the normal `py -3.12 -m venv`.
12. **Size.** The package is about 1,740 lines of code (2,470 with tests), against an estimate of about 600. Most of the extra is error handling and quirk recording in the probe. Half C needs that from a run we cannot repeat cheaply.
13. **Phone page not screenshotted.** Every headless browser on brain timed out (load average about 25 from other sessions). `checklist.html` uses inline CSS only, no external URL, a dark palette, code that wraps, and `overflow-x: hidden` on the page. It was not seen at 390 px.

## What shipped

Commits on the branch:

- `a0303ad` chore(scout): vendored pyLoL reference files and the `.gitignore` block.
- `d93e8b4` feat(scout): probe, trackers, scorer, tests (includes the map-constant check).
- `a874bd2` docs(m156): checklist, portal question, licence note.
- The commit that adds this report and the r1 card.

Files:

- `tools/scout/LICENSE` (AGPL-3.0), `README.md`, `pyproject.toml`, `CHECKLIST.md`.
- `tools/scout/scout/`:
  - `replay_api.py`: HTTPS client for 127.0.0.1:2999.
  - `probe.py`: asks the five unverified items; writes `probe.json` and `openapi.json`.
  - `track_minimap.py`, `track_topdown.py`, `track.py`: the two trackers and their shared parts.
  - `detect.py`: YOLO and template detectors.
  - `calibrate.py`: pixel to world.
  - `score.py`: median error, detection rate, wards, recalls, `handcheck.csv`, plots.
  - `fetch_timeline.py`: Match-V5; the key comes from the environment only.
  - `util.py`.
- `tools/scout/tests/`: 8 test modules, a mock Replay API on an ephemeral 127.0.0.1 port, a network guard, synthetic frames and icons, and a synthetic timeline built from the repo fixture.
- `tools/scout/vendor/pyLoL/`: `LICENSE`, `README.md`, `ATTRIBUTION.md` (upstream commit `19afccf`), `autoLeague/replays/scraper.py`.
- `.gitignore`: venv, weights, egg-info, caches, recordings.
- `.planning/missions/M156/`: `portal-question.md`, `licence-note.md`, this report, `publish-round-r1/` (`entry.json`, `checklist.html`, and copies of the linked documents).

## pytest output

Run from the repository root on brain after the map-constant change. Load average 3.91 before the run:

```
$ tools/scout/.venv/bin/python -m pytest tools/scout -q
..................................                                       [100%]
34 passed in 9.81s
```

A second run later, at load average about 25, also passed: `34 passed in 392.97s (0:06:32)`.

## Checklist

`tools/scout/CHECKLIST.md` is 60 lines with 13 numbered steps. Every step has an "Output:" line naming its file and an "If it fails:" line.

1. Get the branch.
2. Python 3.12 venv.
3. Weights (optional).
4. `EnableReplayApi=1` in `game.cfg`.
5. Pick a replay and create the run directory.
6. Probe.
7. Minimap track.
8. Top-down track, after signing out and in (one recording per desktop session).
9. Timeline, with his own key, never written to disk.
10. Score.
11. Hand check of `handcheck.csv`.
12. Size and content check: at most 50 MB, no video, no key.
13. Commit and push the run directory.

The phone version is `publish-round-r1/checklist.html`.

## Portal question draft (verbatim)

From `portal-question.md`. Jasper sends it himself. Nothing has been sent.

**Subject:** Product 834192 (Team Companion): offline review of a player's own replays, and CommunityDragon icons

Hello,

Team Companion (product 834192) is a private coaching workspace for one amateur team. We have two questions before we build anything.

1. Offline replay review. We are considering a separate tool, free and open source (AGPL-3.0), that a player runs on their own Windows PC on a replay they downloaded themselves. It uses only the official local Replay API (127.0.0.1:2999, which the player switches on with EnableReplayApi=1) and screen capture of the replay's minimap or zoomed-out view. From these it extracts per-second champion positions, ward placements and recalls for post-game review. It does no memory reading, no injection and no .rofl decoding, and it never runs during a live match. The player would then upload the derived tracks to their own team's private workspace. The tracks include the opponents' positions, because those are in the player's replay. They would be visible only to that team, deletable, and kept no longer than our match cache (12 months). Is this acceptable? Should the tool be registered under this product or separately?

2. Icons. May Team Companion reuse CommunityDragon's position (role) and objective icons, served from our own domain alongside Data Dragon? Would a map derived from the map texture be allowed? Today we only measure coordinates and draw our own schematic.

Thank you for your time.

Kind regards,
Jasper Seehofer, Team Companion

## What the mock could not settle

The mock only proves that the probe asks each question and records whatever comes back. None of the six items is answered yet. How each is measured on Jasper's run:

1. **Minimap with `interfaceAll=false` and `interfaceMinimap=true`.** The probe sets both, reads them back into `probe.json` and saves `probe_minimap_only.png`. Half C looks at the screenshot.
2. **Highest honoured `speed`.** The probe posts 1, 2, 4, 8, 10 and 16, reads back what the client keeps and times the game clock against the wall clock. The result is in `probe.json`.
3. **Accelerated recording.** The probe makes one short PNG recording in real time (`enforceFrameRate` on, `replaySpeed` 1) and one accelerated (`enforceFrameRate` off, `replaySpeed` 4). It records wall time, game seconds per wall second, frames written against frames expected, and frame size, in `probe.json`. Whether the frames are lossless is judged in half C from the PNGs.
4. **Camera height and the whole-map view.** The probe raises the camera step by step, reads back the honoured height and saves `probe_camera_*.png`. If no shot shows the whole map, step 8 uses the Ctrl+Shift+Y fallback, and `track_topdown.json` notes which mode ran.
5. **Vanguard and window capture.** Each probe screenshot (mss) records its mean brightness, and a black capture is flagged in `probe.json`. Half C also looks at the PNGs.
6. **Enemy wards on the minimap with fog off.** No code measures this. Half C reads it by eye from the `frames_minimap` PNGs and compares with the `WARD_PLACED` times in `timeline.json`.

## Licence note

See `.planning/missions/M156/licence-note.md`. In short:

- Scout is AGPL-3.0.
- pyLoL is GPL-3.0.
- ultralytics and YOLOv12 are AGPL-3.0.
- pyLoL's weights have no stated licence. That is a risk for M157.
- The app's own licence is undecided and is not chosen here.
- The note also has a privacy paragraph on derived tracks for the findings note.

## Questions for Jasper (on the r1 card)

1. **(react) Run the checklist now?** Chips: "Tonight", "This weekend", "Later — remind me". Recommended: **This weekend**, after focus rows 1 and 2 (Berufsgenossenschaft, ELSTER), which are overdue. Link: `/app/previews/m156-r1-scout-checklist/checklist.html`.
2. **(react) Portal question wording.** The draft above. Chips: "Send as is", "Edit (free text)". Recommended: **Send as is**. It rides focus row 7's portal text, due 2026-10-20.

## What half C does on resume

- **Start condition:** `git log origin/mission/M156-scout-replay-spike -- tools/scout/runs/` shows a run directory committed by Jasper. Without it, half C reports "no run yet" and stops.
- Then half C:
  - writes the facts table for the six items above;
  - scores the minimap track (median error, per-champion detection rate, gaps);
  - compares wards and recalls with Jasper's `handcheck.csv`;
  - tunes the top-down bar detector, fits the homography and checks the nexus and fountain constants against the frames;
  - writes `docs/scout-findings.md` with go or no-go per signal and the M157 route;
  - publishes `publish-round-r2/` with the merge question and "Route for M157";
  - writes REPORT-R2.
