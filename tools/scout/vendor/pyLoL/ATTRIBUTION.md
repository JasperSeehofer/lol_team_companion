# pyLoL: vendored reference copy

- **Upstream:** https://github.com/league-of-legends-replay-extractor/pyLoL
- **Commit:** `19afccfc6590e074567798598833d3a737724ba4` (2026-02-13), fetched 2026-10-17.
- **Licence:** GNU GPL-3.0, per the upstream `LICENSE` file (copied here unchanged). Upstream's
  `setup.py` says "MIT License" in `license=` and in its classifiers; the `LICENSE` file is the
  authoritative grant, so this copy is treated as GPL-3.0. Scout itself is AGPL-3.0-or-later
  (`tools/scout/LICENSE`); the AGPL may incorporate GPL-3.0 code (GPLv3 section 13).

## What was copied, unmodified

| File | Why |
|---|---|
| `LICENSE` | The licence of the copied files. |
| `README.md` | Names the YOLOv12 weights download (Google Drive, `yolov12.pt`, 227 MB) the checklist points to. |
| `autoLeague/replays/scraper.py` | Reference only, **not imported**: its Replay API playback/render payloads (lines 74-162), client launch arguments and the mss minimap capture with hard-coded 4K offsets (lines 306-326). |

## What was not copied, and why

- **No detector code, weights or minimap-to-world mapping exist in the repository.** The weights
  are a separate Google Drive download, and the class list is not published (Scout reads
  `model.names` after loading and reports the game's champions missing from it).
- The notebooks, dataset tooling, docs and `requirements.txt` are not needed. The requirements
  pin `ultralytics==8.0.124`, which cannot load YOLOv12 weights; Scout's `yolo` extra asks for a
  recent ultralytics. ultralytics is AGPL-3.0 (`License: AGPL-3.0` in the 8.4.180 wheel's metadata
  and its LICENSE file, checked 2026-10-17).

## Metrics caveat

The README's figures (mAP 92.2 %, precision 91.3 %, recall 90.2 %) belong to the **older
Roboflow model**, not to the YOLOv12 weights; no metrics are stated for those. Scout measures
its own numbers against the Match-V5 timeline (`scout.score`).

## Scout's own code

Scout's Replay API driver (`scout/replay_api.py`, `scout/probe.py`) is written fresh from the
field list in Riot's League Director (`api.py`, Apache-2.0), not derived from `scraper.py`.
