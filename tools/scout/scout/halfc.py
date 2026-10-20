"""M156 half C in one command: check, score, plot and draft the findings.

  python -m scout.halfc RUN_DIR [--out DIR] [--findings PATH] [--thresholds PATH]
      [--enemy-team blue|red|both]

Local only: it reads RUN_DIR, writes OUT and the findings file, and opens no
socket. Steps:

1. Manifest check: expected files present (or missing with the reason), at most
   50 MB, no video, no API key, no PUUID / Riot ID / summoner name.
2. facts.md: the six unknowns from probe.json and openapi.json, each with its
   evidence (file + key, or frame).
3. score.py's `run` on the minimap track, and on the top-down track if present
   (OUT/score/score_minimap.json, score_topdown.json).
4. Wards and recalls; Jasper's handcheck.csv `jasper_says` merged beside
   `auto_says` (OUT/handcheck_merged.csv, agreement n/rows).
5. Plots (OUT/plots/): score.py's error bars and overlays per track, ward
   precision/recall per team, the top-down frame strip and the fact-6 eye-check
   strip (minimap frames +-2 s around each WARD_PLACED).
6. OUT/halfc-summary.json: machine-readable, no timestamps, deterministic.
7. The findings rendered from templates/scout-findings.md (also OUT/scout-findings.md).

Defaults: --out .planning/missions/M156/publish-round-r2/ and --findings
docs/scout-findings.md. When --out is given and --findings is not, the
findings go to OUT/scout-findings.md only, so a trial run never overwrites the
committed template. Missing inputs never crash the run: they are reported as
"not reached: <reason>" and the verdicts say NOT REACHED.
"""

from __future__ import annotations

import argparse
import contextlib
import csv
import hashlib
import io
import json
import shutil
from pathlib import Path

import cv2
import numpy as np

from . import findings, rules, score
from .track import find_gaps
from .util import list_frames, mmss, read_json, sample_evenly

REPO = Path(__file__).resolve().parents[3]
DEFAULT_OUT = REPO / ".planning" / "missions" / "M156" / "publish-round-r2"
DEFAULT_FINDINGS = REPO / "docs" / "scout-findings.md"
MAX_RUN_MB = 50.0
VIDEO_EXT = {".webm", ".mp4", ".avi", ".mkv", ".mov", ".flv", ".wmv"}
KEY_MARK = ("RG" + "API").encode()     # never written out literally
ID_KEYS = {"puuid", "summonername", "summonerid", "gamename", "tagline", "accountid"}
DEATH_GAP_S = 3.0                       # a gap starting this soon after a death is a death gap
BURST_S = 15.0                          # purchases this close belong to one shop visit
EARLY_S = 60.0                          # the opening shop visit is not a recall
EYE_WINDOW = (-2, -1, 0, 1, 2)          # seconds around a WARD_PLACED
EYE_EVENTS_PER_TEAM = 6
CELL = 128

# (name, kind, reason when missing)
EXPECTED = [
    ("probe.json", "file", "probe not run (session probe)"),
    ("openapi.json", "file", "the client served no spec (see probe.json spec.tried)"),
    ("probe_*.png", "glob", "no probe screenshots (screen capture unavailable or probe not run)"),
    ("track_minimap.json", "file", "route A not reached (session minimap)"),
    ("frames_minimap", "dir", "no minimap frames kept (session minimap)"),
    ("track_topdown.json", "file", "route B not reached (session topdown)"),
    ("frames_topdown", "dir", "no top-down frames kept (session topdown)"),
    ("anchors_template.json", "file", "written only when anchors.json is absent"),
    ("timeline.json", "file", "timeline not fetched (session timeline)"),
    ("match.json", "file", "match not fetched (session timeline)"),
    ("handcheck.csv", "file", "hand check not marked (session score, then session handcheck)"),
    ("run.log", "file", "no module wrote a log"),
]


# ---------------------------------------------------------------- 1. manifest

def _walk_ids(node, path: str, hits: list[str], fname: str) -> None:
    if isinstance(node, dict):
        for k, v in node.items():
            kl = str(k).lower()
            if kl in ID_KEYS or kl.startswith("riotid"):
                hits.append(f"{fname}: {path}{k}")
            elif kl == "participants" and path.endswith("metadata.") and isinstance(v, list) \
                    and any(isinstance(x, str) for x in v):
                hits.append(f"{fname}: {path}{k} (PUUID list)")
            else:
                _walk_ids(v, f"{path}{k}.", hits, fname)
    elif isinstance(node, list):
        for item in node:
            _walk_ids(item, path, hits, fname)


def manifest_check(run_dir: Path) -> dict:
    run_dir = Path(run_dir)
    if not run_dir.is_dir():
        return {"ok": False, "present": [], "missing": {"run dir": "not a directory"},
                "size_mb": 0.0, "size_ok": False, "videos": [], "key_hits": [],
                "identifier_hits": [], "problems": ["run dir not found"]}
    files = sorted(p for p in run_dir.rglob("*") if p.is_file())
    present, missing = [], {}
    for name, kind, reason in EXPECTED:
        p = run_dir / name
        found = (kind == "file" and p.is_file()) or \
                (kind == "dir" and p.is_dir() and any(p.iterdir())) or \
                (kind == "glob" and any(run_dir.glob(name)))
        if found:
            present.append(name)
        else:
            missing[name] = reason
    size = sum(p.stat().st_size for p in files)
    videos = [p.relative_to(run_dir).as_posix() for p in files if p.suffix.lower() in VIDEO_EXT]
    key_hits, id_hits = [], []
    for p in files:
        rel = p.relative_to(run_dir).as_posix()
        data = p.read_bytes()
        if KEY_MARK in data:
            key_hits.append(rel)
        if p.suffix.lower() == ".json":
            try:
                _walk_ids(json.loads(data.decode("utf-8")), "", id_hits, rel)
            except (ValueError, UnicodeDecodeError):
                id_hits.append(f"{rel}: unreadable JSON, not checked")
        elif p.suffix.lower() in {".csv", ".log", ".txt", ".md"}:
            low = data.decode("utf-8", "replace").lower()
            for word in ("puuid", "riotid", "summonername"):
                if word in low:
                    id_hits.append(f"{rel}: '{word}'")
    size_mb = round(size / 2**20, 2)
    problems = []
    if size_mb > MAX_RUN_MB:
        problems.append(f"{size_mb} MB > {MAX_RUN_MB} MB")
    if videos:
        problems.append(f"video files: {', '.join(videos)}")
    if key_hits:
        problems.append(f"API key pattern in: {', '.join(key_hits)}")
    if id_hits:
        problems.append(f"player identifiers: {'; '.join(id_hits)}")
    return {"ok": not problems, "present": present, "missing": missing, "size_mb": size_mb,
            "size_ok": size_mb <= MAX_RUN_MB, "videos": videos, "key_hits": key_hits,
            "identifier_hits": id_hits, "problems": problems}


def manifest_line(m: dict) -> str:
    head = "PASS" if m["ok"] else "FAIL (" + "; ".join(m["problems"]) + ")"
    miss = "; ".join(f"`{k}` missing: {v}" for k, v in m["missing"].items()) or "none missing"
    return f"{head}; {m['size_mb']} MB; {len(m['present'])} of {len(EXPECTED)} expected entries; {miss}"


# ---------------------------------------------------------------- 2. facts

def _load(run_dir: Path, name: str):
    p = run_dir / name
    if not p.is_file():
        return None
    try:
        return read_json(p)
    except (ValueError, OSError):
        return None


def _spec_mentions(spec, field: str) -> str:
    if spec is None:
        return "no openapi.json"
    return "yes" if field in json.dumps(spec) else "no"


def _yn(v) -> str:
    return {True: "yes", False: "no"}.get(v, "unknown")


def _sub(probe: dict | None, key: str):
    """(sub-probe result, reason it is missing)."""
    if probe is None:
        return None, "not reached: probe.json missing"
    if key in probe and probe[key] is not None:
        return probe[key], None
    err = (probe.get("errors") or {}).get(key)
    return None, f"not reached: {err}" if err else "not reached: sub-probe did not run"


def _shot(s) -> str:
    if not s:
        return "no screenshot"
    return f"{s.get('file')} (mean brightness {s.get('mean_brightness')})"


def facts(run_dir: Path, probe, spec, topdown, eye: dict) -> list[dict]:
    rows = []
    iface, why = _sub(probe, "interface")
    if why:
        rows.append({"n": 1, "value": why, "evidence": "probe.json"})
    else:
        a, m = iface.get("interfaceAll", {}), iface.get("interfaceMinimap", {})
        ok = bool(a.get("honoured")) and bool(m.get("honoured"))
        rows.append({"n": 1, "value": f"{'honoured' if ok else 'not honoured'} (interfaceAll read "
                                      f"{a.get('read')}, interfaceMinimap read {m.get('read')})",
                     "evidence": f"probe.json interface; {_shot(iface.get('screenshot'))}; "
                                 f"openapi.json mentions interfaceMinimap: "
                                 f"{_spec_mentions(spec, 'interfaceMinimap')}"})
    sp, why = _sub(probe, "speed")
    if why:
        rows.append({"n": 2, "value": why, "evidence": "probe.json"})
    else:
        steps = ", ".join(f"{s['sent']}→{s.get('read')} ({s.get('game_s_per_wall_s')})"
                          for s in sp.get("steps", []))
        rows.append({"n": 2, "value": f"{sp.get('max_honoured')}x",
                     "evidence": f"probe.json speed.steps (sent→read, game-s per wall-s): {steps}"})
    rec, why = _sub(probe, "recording")
    if why:
        rows.append({"n": 3, "value": why, "evidence": "probe.json"})
    else:
        def one(r):
            return (f"{r.get('game_s_per_wall_s')} game-s per wall-s, frames "
                    f"{r.get('frames_written')}/{r.get('frames_expected')}, size {r.get('frame_size')}")
        acc, rt = rec.get("accelerated", {}), rec.get("realtime", {})
        fast = realtime_only(probe)
        lossless = (acc.get("read_back") or {}).get("lossless")
        rows.append({"n": 3, "value": f"faster than real time: {_yn(None if fast is None else not fast)} "
                                      f"(accelerated {one(acc)}; real time {one(rt)}); lossless read "
                                      f"back: {_yn(lossless)}",
                     "evidence": "probe.json recording.accelerated, recording.realtime (codec png)"})
    cam, why = _sub(probe, "camera")
    if why:
        rows.append({"n": 4, "value": why, "evidence": "probe.json"})
    else:
        top = cam.get("max_honoured_y")
        far = next((s.get("read_farClip") for s in cam.get("steps", []) if s.get("sent_y") == top), None)
        mode = ((topdown or {}).get("recording") or {}).get("camera", {}).get("mode") \
            if topdown else None
        shot = f"probe_camera_{top}.png" if top else "no honoured height"
        rows.append({"n": 4, "value": f"camera height honoured up to {top}; farClip read {far}; "
                                      f"top-down ran with camera mode "
                                      f"{mode or ('not recorded' if topdown else 'not reached')}",
                     "evidence": f"probe.json camera.steps; {shot}; openapi.json mentions farClip: "
                                 f"{_spec_mentions(spec, 'farClip')}"})
    if probe is None:
        rows.append({"n": 5, "value": "not reached: probe.json missing", "evidence": "probe.json"})
    else:
        shots = [(iface or {}).get("screenshot")] + \
                [s.get("screenshot") for s in (probe.get("camera") or {}).get("steps", [])]
        shots = [s for s in shots if s]
        black = [s["file"] for s in shots if s.get("black")]
        recs = probe.get("recording") or {}
        written = [f"{k} {v.get('frames_written')}/{v.get('frames_expected')}"
                   for k, v in sorted(recs.items())]
        if not shots and not recs:
            rows.append({"n": 5, "value": "not reached: no screenshots or recordings in probe.json",
                         "evidence": "probe.json"})
        else:
            rows.append({"n": 5, "value": f"{len(black)} of {len(shots)} screenshots black; "
                                          f"recordings wrote {', '.join(written) or 'nothing'}",
                         "evidence": "probe.json *.screenshot.black, recording.*.frames_written"
                                     + (f"; black: {', '.join(black)}" if black else "")})
    fog, why = _sub(probe, "fog")
    fog_txt = why if why else f"fogOfWar=false honoured: {_yn((fog.get('fogOfWar') or {}).get('honoured'))}"
    if eye.get("events") is None:
        eye_txt = f"eye-check strip not reached: {eye.get('reason')}"
    else:
        eye_txt = (f"{eye['events']} WARD_PLACED events with minimap frames within ±2 s in "
                   f"plots/fact6_eye_check.png ({eye['without_frames']} without a frame)")
    rows.append({"n": 6, "value": f"{fog_txt}; {eye_txt}; visibility read by eye (agent judgment below)",
                 "evidence": "probe.json fog; frames_minimap/tNNNNN.png around timeline.json WARD_PLACED"})
    return rows


FACT_TITLES = {
    1: "Minimap-only mode honoured (`interfaceAll=false` + `interfaceMinimap=true`)",
    2: "Highest honoured replay speed",
    3: "Accelerated recording (real time or faster; lossless?)",
    4: "Camera height and whole-map view (API camera vs Ctrl+Shift+Y, `farClip`)",
    5: "Vanguard or capture black frames",
    6: "Enemy wards on the minimap with fog off",
}


def facts_md(rows: list[dict], run_name: str) -> str:
    out = [f"# Facts table: {run_name}", "", "| # | Unknown | Measured | Evidence |", "|---|---|---|---|"]
    for r in rows:
        out.append(f"| {r['n']} | {FACT_TITLES[r['n']]} | {_cell(r['value'])} | {_cell(r['evidence'])} |")
    return "\n".join(out) + "\n"


def _cell(s: str) -> str:
    return str(s).replace("|", "\\|").replace("\n", " ")


def realtime_only(probe) -> bool | None:
    """True when no accelerated recording was shown to run faster than real time."""
    rec = (probe or {}).get("recording")
    if not rec:
        return None
    th = rules.load_thresholds().get("realtime_only_below", 1.5)
    rate = (rec.get("accelerated") or {}).get("game_s_per_wall_s")
    if rate is None or not (rec.get("accelerated") or {}).get("frames_written"):
        return True
    return rate < th


# ---------------------------------------------------------------- 3. scoring

def _pmap_args(run_dir: Path, track: dict) -> list[str]:
    if (run_dir / "match.json").is_file():
        return ["--match", str(run_dir / "match.json")]
    return ["--participants", ",".join(f"{i + 1}={c}" for i, c in enumerate(track["champions"]))]


def run_score(run_dir: Path, out: Path, track_name: str, tag: str, wards_track: str | None) -> dict | None:
    track_p, tl_p = run_dir / track_name, run_dir / "timeline.json"
    if not (track_p.is_file() and tl_p.is_file()):
        return None
    track = read_json(track_p)
    argv = ["run", "--track", str(track_p), "--timeline", str(tl_p), *_pmap_args(run_dir, track),
            "--tag", tag, "--out", str(out / "score")]
    if wards_track and (run_dir / wards_track).is_file():
        argv += ["--wards-track", str(run_dir / wards_track)]
    with contextlib.redirect_stdout(io.StringIO()):
        score.main(argv)
    return read_json(out / "score" / f"score_{tag}.json")


def _pid_map(run_dir: Path, track: dict) -> dict[int, str]:
    match = _load(run_dir, "match.json")
    if match:
        return score.participant_map(match)
    return {i + 1: c for i, c in enumerate(track["champions"])}


def alive_adjusted(track: dict, timeline: dict, pmap: dict[int, str]) -> dict:
    """Detection rate and longest gap with death gaps taken out: a gap that
    starts within DEATH_GAP_S after that champion's CHAMPION_KILL."""
    deaths: dict[str, list[float]] = {}
    for e in score.events(timeline, "CHAMPION_KILL"):
        c = pmap.get(e.get("victimId"))
        if c:
            deaths.setdefault(c, []).append(e["timestamp"] / 1000)
    samples, champs = track["samples"], track["champions"]
    gaps = find_gaps(samples, champs)
    death_gaps, other = [], []
    for g in gaps:
        if any(0 <= g["start"] - d <= DEATH_GAP_S for d in deaths.get(g["champion"], [])):
            death_gaps.append(g)
        else:
            other.append(g)
    excluded = sum(1 for g in death_gaps for s in samples if g["start"] <= s["t"] <= g["end"])
    seen = sum(1 for s in samples for c in champs if s["pos"].get(c))
    total = len(samples) * len(champs) - excluded
    return {"detection_rate": round(seen / total, 4) if total else None,
            "longest_gap_s": max((g["length_s"] for g in other), default=0.0),
            "gaps_count": len(other), "death_gaps": len(death_gaps)}


def bar_hit_rate(topdown: dict) -> float | None:
    frames = topdown.get("detections_px") or []
    if not frames:
        return None
    n = len(topdown.get("champions") or []) or 10
    hits = [min(n, sum(1 for d in f["dets"] if d.get("label") == "champion")) for f in frames]
    return round(sum(hits) / (n * len(frames)), 4)


def purchase_bursts(timeline: dict) -> list[dict]:
    """Shop visits from ITEM_PURCHASED: per participant, purchases closer than
    BURST_S form one burst; the opening visit and visits within 120 s after a
    death (respawn shopping) are left out. These are the recalls to find."""
    buys = sorted(score.events(timeline, "ITEM_PURCHASED"),
                  key=lambda e: (e["participantId"], e["timestamp"]))
    deaths = [(e.get("victimId"), e["timestamp"] / 1000) for e in score.events(timeline, "CHAMPION_KILL")]
    bursts: list[dict] = []
    for e in buys:
        t, pid = e["timestamp"] / 1000, e["participantId"]
        last = bursts[-1] if bursts else None
        if last and last["participantId"] == pid and t - last["end"] <= BURST_S:
            last["end"] = t
            last["n"] += 1
        else:
            bursts.append({"participantId": pid, "start": t, "end": t, "n": 1})
    return [b for b in bursts if b["start"] >= EARLY_S and not any(
        v == b["participantId"] and 0 <= b["start"] - d <= score.RESPAWN_WINDOW_S for v, d in deaths)]


def recall_numbers(recalls: list[dict], timeline: dict, pmap: dict[int, str]) -> dict:
    bursts = purchase_bursts(timeline)
    champ_pid = {c: p for p, c in pmap.items()}
    found = sum(1 for b in bursts if any(
        champ_pid.get(r["champion"], r.get("participantId")) == b["participantId"]
        and abs(b["start"] - r["t"]) <= score.PURCHASE_WINDOW_S for r in recalls))
    confirmed = sum(1 for r in recalls if r.get("confirmed"))
    return {"detected": len(recalls), "confirmed": confirmed, "bursts": len(bursts), "found": found,
            "precision": round(confirmed / len(recalls), 4) if recalls else None,
            "recall": round(found / len(bursts), 4) if bursts else None}


# ---------------------------------------------------------------- 4. hand check

YES = {"yes", "y", "ja", "j", "true", "1", "x"}
NO = {"no", "n", "nein", "false", "0"}


def merge_handcheck(run_dir: Path, out: Path) -> dict:
    p = run_dir / "handcheck.csv"
    if not p.is_file():
        return {"marked": False, "reason": "not marked: handcheck.csv missing"}
    with open(p, newline="", encoding="utf-8-sig") as fh:
        rows = list(csv.DictReader(fh))
    merged, per = [], {}
    for r in rows:
        said = (r.get("jasper_says") or "").strip().lower()
        mark = "yes" if said in YES else "no" if said in NO else ""
        merged.append({"event_id": r.get("event_id", ""), "kind": r.get("kind", ""),
                       "game_time_mmss": r.get("game_time_mmss", ""), "champion": r.get("champion", ""),
                       "auto_says": r.get("auto_says", ""), "jasper_says": r.get("jasper_says", ""),
                       "agree": "" if not mark else ("yes" if mark == "yes" else "no"),
                       "note": r.get("note", "")})
        if mark:
            k = per.setdefault(r.get("kind", ""), {"rows": 0, "agree": 0})
            k["rows"] += 1
            k["agree"] += mark == "yes"
    with open(out / "handcheck_merged.csv", "w", newline="", encoding="utf-8") as fh:
        w = csv.DictWriter(fh, fieldnames=list(merged[0]) if merged else
                           ["event_id", "kind", "game_time_mmss", "champion", "auto_says",
                            "jasper_says", "agree", "note"])
        w.writeheader()
        w.writerows(merged)
    marked = sum(k["rows"] for k in per.values())
    if not marked:
        return {"marked": False, "reason": "not marked: no row has jasper_says yes or no",
                "rows": len(rows)}
    agree = sum(k["agree"] for k in per.values())
    return {"marked": True, "rows": len(rows), "marked_rows": marked, "agree": agree,
            "per_kind": {k: per[k] for k in sorted(per)}}


def handcheck_line(hc: dict) -> str:
    if not hc.get("marked"):
        return hc["reason"]
    kinds = ", ".join(f"{k} {v['agree']}/{v['rows']}" for k, v in hc["per_kind"].items())
    return f"agreement {hc['agree']}/{hc['marked_rows']} marked rows of {hc['rows']} ({kinds}); " \
           f"`handcheck_merged.csv`"


# ---------------------------------------------------------------- 5. plots

def _png(path: Path, img: np.ndarray) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    cv2.imwrite(str(path), img, [cv2.IMWRITE_PNG_COMPRESSION, 9])


def _label(img: np.ndarray, text: str, y: int = 14) -> None:
    cv2.putText(img, text, (4, y), cv2.FONT_HERSHEY_SIMPLEX, 0.4, (255, 255, 255), 1, cv2.LINE_AA)


def eye_check_strip(run_dir: Path, timeline: dict | None, out: Path, enemy_team: str) -> dict:
    frames_dir = run_dir / "frames_minimap"
    if timeline is None:
        return {"events": None, "reason": "timeline.json missing"}
    if not frames_dir.is_dir():
        return {"events": None, "reason": "frames_minimap/ missing"}
    have = {int(t): p for t, p in list_frames(frames_dir)}
    if not have:
        return {"events": None, "reason": "no frames in frames_minimap/"}
    teams = ("blue", "red") if enemy_team == "both" else (enemy_team,)
    evs = [e for e in score.events(timeline, "WARD_PLACED")
           if e.get("wardType") not in score.SKIP_WARD_TYPES and e.get("creatorId", 0) > 0]
    rows, without = [], 0
    for team in teams:
        team_evs = sorted((e for e in evs if score.team_of(e["creatorId"]) == team),
                          key=lambda e: e["timestamp"])
        with_frames = []
        for e in team_evs:
            t0 = int(round(e["timestamp"] / 1000))
            if any(t0 + d in have for d in EYE_WINDOW):
                with_frames.append((e, t0))
            else:
                without += 1
        for e, t0 in sample_evenly(with_frames, EYE_EVENTS_PER_TEAM):
            cells = []
            for d in EYE_WINDOW:
                p = have.get(t0 + d)
                if p is None:
                    cell = np.full((CELL, CELL, 3), 60, np.uint8)
                    _label(cell, "no frame", CELL // 2)
                else:
                    cell = cv2.resize(cv2.imread(str(p)), (CELL, CELL), interpolation=cv2.INTER_AREA)
                _label(cell, f"{mmss(t0 + d)} ({d:+d}s)")
                cells.append(cell)
            head = np.full((CELL, 150, 3), 30, np.uint8)
            _label(head, f"{team} ward", 20)
            _label(head, f"{e.get('wardType', '')[:16]}", 40)
            _label(head, f"at {mmss(e['timestamp'] / 1000)}", 60)
            _label(head, f"creator {e['creatorId']}", 80)
            rows.append(np.hstack([head, *cells]))
    if rows:
        img = np.vstack(rows)
    else:
        img = np.full((CELL, 150 + CELL * len(EYE_WINDOW), 3), 30, np.uint8)
        _label(img, "no minimap frame within +-2 s of any WARD_PLACED", CELL // 2)
    _png(out / "plots" / "fact6_eye_check.png", img)
    return {"events": len(rows), "without_frames": without, "teams": list(teams)}


def topdown_strip(run_dir: Path, out: Path) -> int:
    d = run_dir / "frames_topdown"
    frames = list_frames(d) if d.is_dir() else []
    if not frames:
        return 0
    cells = []
    for t, p in frames:
        img = cv2.imread(str(p))
        h = 180
        cell = cv2.resize(img, (round(img.shape[1] * h / img.shape[0]), h), interpolation=cv2.INTER_AREA)
        _label(cell, mmss(t))
        cells.append(cell)
    _png(out / "plots" / "topdown_strip.png", np.hstack(cells))
    return len(cells)


def wards_plot(wards: dict | None, out: Path) -> bool:
    if not wards:
        return False
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    teams = ["blue", "red"]
    p = [wards.get(t, {}).get("precision") or 0 for t in teams]
    r = [wards.get(t, {}).get("recall") or 0 for t in teams]
    x = np.arange(len(teams))
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.bar(x - 0.18, p, 0.36, label="precision", color="#3b6fb6")
    ax.bar(x + 0.18, r, 0.36, label="recall", color="#b6893b")
    ax.set_xticks(x, [f"{t} (placed {wards.get(t, {}).get('placed')})" for t in teams])
    ax.set_ylim(0, 1.05)
    ax.set_title(f"Wards vs WARD_PLACED (±{wards.get('tolerance_s')} s), source {wards.get('source')}")
    ax.legend()
    fig.tight_layout()
    path = out / "plots" / "wards_precision_recall.png"
    path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(path, dpi=100, metadata={"Software": None})
    plt.close(fig)
    return True


def score_plots(run_dir: Path, out: Path, tag: str) -> bool:
    sp = out / "score" / f"score_{tag}.json"
    if not sp.is_file():
        return False
    argv = ["plots", "--score", str(sp), "--out", str(out / "plots" / tag)]
    frames = list_frames(run_dir / "frames_minimap") if (run_dir / "frames_minimap").is_dir() else []
    if tag == "minimap" and frames:
        argv += ["--minimap", str(frames[0][1])]
    with contextlib.redirect_stdout(io.StringIO()):
        score.main(argv)
    return True


# ---------------------------------------------------------------- the run

def _sha(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def time_cost(probe, match, timeline) -> str:
    dur = (match or {}).get("info", {}).get("gameDuration")
    if dur is None and timeline:
        frames = timeline["info"]["frames"]
        dur = frames[-1]["timestamp"] / 1000 if frames else None
    speed = ((probe or {}).get("speed") or {}).get("max_honoured")
    rec = (probe or {}).get("recording") or {}
    acc = (rec.get("accelerated") or {}).get("game_s_per_wall_s")
    rt = (rec.get("realtime") or {}).get("game_s_per_wall_s")
    if dur is None:
        return "not reached: no game length (match.json and timeline.json missing)"
    parts = [f"game length {dur / 60:.1f} min"]
    parts.append(f"minimap pass {dur / speed / 60:.1f} min at {speed}x" if speed
                 else "minimap pass: not reached (no honoured speed)")
    rate = acc if acc and not realtime_only(probe) else rt
    if rate:
        parts.append(f"top-down full game {dur / rate / 60:.1f} min at {rate} game-s per wall-s "
                     f"(the spike's 5-min window {300 / rate / 60:.1f} min)")
    else:
        parts.append("top-down: not reached (no recording rate)")
    return "; ".join(parts)


def _thr_text(th: dict, key: str) -> str:
    t = th[key]["go"]
    if key == "positions_minimap":
        p = th[key]["partial"]
        return (f"GO: median ≤{t['median_error_max']}, detection ≥{t['detection_rate_min']}, "
                f"longest gap ≤{t['longest_gap_s_max']} s; PARTIAL: median ≤{p['median_error_max']}, "
                f"detection ≥{p['detection_rate_min']}")
    if key == "positions_topdown":
        return f"GO: median ≤{t['median_error_max']}, bar hit rate ≥{t['bar_hit_rate_min']}"
    return (f"GO: precision ≥{t['precision_min']}, recall ≥{t['recall_min']}, hand check "
            f"≥{t['handcheck_agree_min']} of {t['handcheck_of']}")


def _nums(v: dict) -> str:
    return ", ".join(f"{k.replace('_', ' ')} {'n/a' if x is None else x}" for k, x in v["numbers"].items()) \
        or "n/a"


def halfc(run_dir: Path, out: Path, findings_path: Path | None, thresholds: Path | None = None,
          enemy_team: str = "both") -> dict:
    run_dir, out = Path(run_dir), Path(out)
    if out.exists():
        for sub in ("plots", "score"):
            shutil.rmtree(out / sub, ignore_errors=True)
    out.mkdir(parents=True, exist_ok=True)
    th = rules.load_thresholds(thresholds)
    manifest = manifest_check(run_dir)
    probe, spec = _load(run_dir, "probe.json"), _load(run_dir, "openapi.json")
    timeline, match = _load(run_dir, "timeline.json"), _load(run_dir, "match.json")
    mm, td = _load(run_dir, "track_minimap.json"), _load(run_dir, "track_topdown.json")
    not_reached = [f"`{k}`: {v}" for k, v in manifest["missing"].items()
                   if k not in ("anchors_template.json", "openapi.json")]

    eye = eye_check_strip(run_dir, timeline, out, enemy_team)
    fact_rows = facts(run_dir, probe, spec, td, eye)
    (out / "facts.md").write_text(facts_md(fact_rows, run_dir.name), encoding="utf-8")

    wards_src = "track_topdown.json" if td else None
    s_mm = run_score(run_dir, out, "track_minimap.json", "minimap", wards_src) if mm else None
    s_td = run_score(run_dir, out, "track_topdown.json", "topdown", None) if td else None

    a_in = None
    if s_mm:
        adj = alive_adjusted(mm, timeline, _pid_map(run_dir, mm))
        a_in = {"median_error": s_mm["median_error"], "detection_rate": adj["detection_rate"],
                "longest_gap_s": adj["longest_gap_s"]}
        a_raw = {k: s_mm[k] for k in ("median_error", "n_scored", "detection_rate", "gaps_count",
                                      "longest_gap_s")}
    b_in = None
    if td:
        b_in = {"median_error": (s_td or {}).get("median_error") if td.get("calibration") else None,
                "bar_hit_rate": bar_hit_rate(td), "calibrated": td.get("calibration") is not None}
    hc = merge_handcheck(run_dir, out)
    per_kind = hc.get("per_kind", {}) if hc.get("marked") else {}

    def hc_of(kind):
        k = per_kind.get(kind)
        return (k["agree"], k["rows"]) if k else (None, None)

    wards = s_mm["wards"] if s_mm else None
    w_verdicts = {}
    for team in ("blue", "red"):
        if not td or not wards:
            w_verdicts[team] = rules.verdict(rules.NOT_REACHED, "no top-down ward track (route B not "
                                             "reached); the minimap has no ward detector"
                                             if not td else "no timeline or minimap track")
        else:
            agree, rows_ = hc_of("ward")
            w_verdicts[team] = rules.precision_recall_handcheck(
                {"precision": wards[team]["precision"], "recall": wards[team]["recall"],
                 "handcheck_agree": agree, "handcheck_rows": rows_}, th["wards"], f"{team} wards")
    rec_nums = recall_numbers(s_mm["recalls"], timeline, _pid_map(run_dir, mm)) if s_mm else None
    if rec_nums:
        agree, rows_ = hc_of("recall")
        r_verdict = rules.precision_recall_handcheck(
            {"precision": rec_nums["precision"], "recall": rec_nums["recall"],
             "handcheck_agree": agree, "handcheck_rows": rows_}, th["recalls"], "recall")
    else:
        r_verdict = rules.verdict(rules.NOT_REACHED, "no minimap track or no timeline")
    a_v = rules.positions_minimap(a_in, th["positions_minimap"])
    b_v = rules.positions_topdown(b_in, th["positions_topdown"])
    rt_only = realtime_only(probe)
    route = rules.route(a_v["verdict"], b_v["verdict"], rt_only)
    cost = time_cost(probe, match, timeline)

    plots = {"minimap": score_plots(run_dir, out, "minimap"),
             "topdown": score_plots(run_dir, out, "topdown"),
             "wards": wards_plot(wards if td else None, out),
             "topdown_strip_frames": topdown_strip(run_dir, out)}
    plot_files = sorted(p.relative_to(out).as_posix() for p in (out / "plots").rglob("*.png")) \
        if (out / "plots").is_dir() else []

    summary = {
        "run": run_dir.name,
        "generator": "scout.halfc",
        "manifest": manifest,
        "inputs_sha256": {p.relative_to(run_dir).as_posix(): _sha(p)
                          for p in sorted(run_dir.rglob("*")) if p.is_file()},
        "facts": fact_rows,
        "positions_minimap": {"raw": a_raw if s_mm else None, "alive_adjusted": a_in,
                              "verdict": a_v},
        "positions_topdown": {"input": b_in, "verdict": b_v},
        "wards": {"numbers": wards, "verdicts": w_verdicts},
        "recalls": {"numbers": rec_nums, "verdict": r_verdict},
        "handcheck": hc,
        "realtime_only": rt_only,
        "route": route,
        "time_cost": cost,
        "eye_check": eye,
        "plots": plot_files,
        "thresholds": th,
        "not_reached": not_reached,
    }
    (out / "halfc-summary.json").write_text(json.dumps(summary, indent=1, sort_keys=True,
                                                       ensure_ascii=False) + "\n", encoding="utf-8")

    def wnums(team):
        if not wards:
            return "n/a"
        w = wards[team]
        return (f"precision {w['precision']}, recall {w['recall']} (tp {w['tp']}, detected "
                f"{w['detected']}, placed {w['placed']})")

    values = {
        "status": f"Status: draft — computed by scout.halfc from run {run_dir.name}; the agent "
                  f"judgment lines are half C's to write",
        "run": f"`{run_dir.name}`",
        "thresholds_status": th.get("status", "unknown"),
        "manifest": manifest_line(manifest),
        "a_numbers": (f"median {a_in['median_error']} (n={a_raw['n_scored']}), detection "
                      f"{a_in['detection_rate']} (raw {a_raw['detection_rate']}), longest gap "
                      f"{a_in['longest_gap_s']} s (raw {a_raw['longest_gap_s']} s)") if a_in else "n/a",
        "a_threshold": _thr_text(th, "positions_minimap"),
        "a_verdict": a_v["verdict"], "a_reason": a_v["reason"],
        "b_numbers": _nums(b_v) if b_in else "n/a",
        "b_threshold": _thr_text(th, "positions_topdown"),
        "b_verdict": b_v["verdict"], "b_reason": b_v["reason"],
        "wards_blue_numbers": wnums("blue") if td else "n/a",
        "wards_red_numbers": wnums("red") if td else "n/a",
        "wards_threshold": _thr_text(th, "wards"),
        "wards_blue_verdict": w_verdicts["blue"]["verdict"],
        "wards_red_verdict": w_verdicts["red"]["verdict"],
        "wards_blue_reason": w_verdicts["blue"]["reason"],
        "wards_red_reason": w_verdicts["red"]["reason"],
        "recalls_numbers": (f"precision {rec_nums['precision']} ({rec_nums['confirmed']}/"
                            f"{rec_nums['detected']} confirmed), recall {rec_nums['recall']} "
                            f"({rec_nums['found']}/{rec_nums['bursts']} purchase bursts)")
        if rec_nums else "n/a",
        "recalls_threshold": _thr_text(th, "recalls"),
        "recalls_verdict": r_verdict["verdict"], "recalls_reason": r_verdict["reason"],
        "handcheck": handcheck_line(hc),
        "route": route["route"], "route_reason": route["reason"],
        "time_cost": cost,
        "not_reached": "; ".join(not_reached) or "nothing",
    }
    for r in fact_rows:
        values[f"fact{r['n']}_value"] = _cell(r["value"])
        values[f"fact{r['n']}_evidence"] = _cell(r["evidence"])
    text = findings.render(values)
    (out / "scout-findings.md").write_text(text, encoding="utf-8")
    if findings_path:
        Path(findings_path).parent.mkdir(parents=True, exist_ok=True)
        Path(findings_path).write_text(text, encoding="utf-8")
    return summary


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(prog="scout.halfc", description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("run_dir", type=Path)
    ap.add_argument("--out", type=Path, help=f"default {DEFAULT_OUT.relative_to(REPO)}")
    ap.add_argument("--findings", type=Path, help="default docs/scout-findings.md (only when --out "
                                                  "is not given; else OUT/scout-findings.md only)")
    ap.add_argument("--thresholds", type=Path, help="default scout/thresholds.json")
    ap.add_argument("--enemy-team", choices=["blue", "red", "both"], default="both",
                    help="whose WARD_PLACED events the fact-6 eye-check strip shows")
    args = ap.parse_args(argv)
    out = args.out or DEFAULT_OUT
    fpath = args.findings or (None if args.out else DEFAULT_FINDINGS)
    s = halfc(args.run_dir, out, fpath, args.thresholds, args.enemy_team)
    print(f"run {s['run']}: manifest {'PASS' if s['manifest']['ok'] else 'FAIL'}; "
          f"A {s['positions_minimap']['verdict']['verdict']}; "
          f"B {s['positions_topdown']['verdict']['verdict']}; "
          f"wards blue {s['wards']['verdicts']['blue']['verdict']}, red "
          f"{s['wards']['verdicts']['red']['verdict']}; recalls {s['recalls']['verdict']['verdict']}; "
          f"route {s['route']['route']}")
    print(f"outputs in {out}" + (f"; findings {fpath}" if fpath else ""))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
