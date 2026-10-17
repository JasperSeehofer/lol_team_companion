# Scout (spike, M156)

Per-second champion positions, wards and recalls from **your own** League replay, using only
Riot's local Replay API (`127.0.0.1:2999`) and vision on your own screen. No memory reading, no
injection, no replay-file decoding. A separate tool beside Team Companion, licensed
AGPL-3.0-or-later (`LICENSE`); `vendor/pyLoL/` is GPL-3.0 (see its `ATTRIBUTION.md`).

| Module | What it does | Output |
|---|---|---|
| `scout.probe` | Which Replay API values the client honours (spec, minimap-only, speed, recording, camera height, health bars, fog) | `probe.json`, `openapi.json`, `probe_*.png` |
| `scout.track_minimap` | Route A: minimap at 1 Hz game time, icon detector (YOLO weights or template fallback), Kalman + one-track-per-champion | `track_minimap.json`, `frames_minimap/` |
| `scout.track_topdown` | Route B: whole-map camera, health bars thresholded in HSV, identities from the minimap track, homography from `anchors.json` | `track_topdown.json`, `frames_topdown/` |
| `scout.fetch_timeline` | Match-V5 timeline and match (key from `RIOT_API_KEY` only, never stored) | `timeline.json`, `match.json` |
| `scout.score` | Median error, detection rate, gaps; wards vs `WARD_PLACED`; recalls vs purchases; `plots` | `score.json`, `handcheck.csv`, `plots/` |

Running it on a replay: `CHECKLIST.md` (Windows). Tests (Linux, no network, no client):

```bash
uv venv --python python3.12 --seed tools/scout/.venv   # or python3.12 -m venv
tools/scout/.venv/bin/pip install -e 'tools/scout[dev]'
tools/scout/.venv/bin/python -m pytest tools/scout -q
```

The fixtures are synthetic (`tests/fixtures/make_fixtures.py` regenerates them byte-identically);
the Replay API in tests is a mock on an ephemeral 127.0.0.1 port, and a guard in `conftest.py`
refuses any other connection.
