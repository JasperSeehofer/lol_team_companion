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
| `scout.session` | The PC driver: one-line verbs (`start`, `enable-api`, `probe`, `minimap`, `topdown`, `timeline`, `score`, `handcheck`, `check`, `send`); current run in `runs/.current` | the run dir; `send` copies it to brain by `scp` |
| `scout.halfc` | M156 half C on a run dir: manifest check, facts table, scores, hand check, plots, go/no-go and route | `facts.md`, `halfc-summary.json`, `plots/`, `scout-findings.md` |

Minimap icon templates: the replay minimap draws each champion's base circle icon, so
`scout.track_minimap` first fetches CommunityDragon's `<champion>_circle.png` / `_circle_0.png`
and falls back to the Data Dragon square portrait (`--icons DIR` overrides both; the source per
champion is logged and recorded as `icon_sources` in `track_minimap.json`). CommunityDragon is not
on Riot's approved asset list: these are local detection templates fetched on the player's PC
into the git-ignored `models/icons/`, never committed, shipped or served; Data Dragon is the fallback.

Running it on a replay: `CHECKLIST.md` (Windows). Run dirs (`runs/`) are git-ignored and never
go to GitHub; `python -m scout.session send` copies one to brain. Tests (Linux, no network, no client):

```bash
uv venv --python python3.12 --seed tools/scout/.venv   # or python3.12 -m venv
tools/scout/.venv/bin/pip install -e 'tools/scout[dev]'
tools/scout/.venv/bin/python -m pytest tools/scout -q
```

The fixtures are synthetic (`tests/fixtures/make_fixtures.py` regenerates them byte-identically);
the Replay API in tests is a mock on an ephemeral 127.0.0.1 port, and a guard in `conftest.py`
refuses any other connection.
