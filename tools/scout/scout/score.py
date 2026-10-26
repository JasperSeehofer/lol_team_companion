"""Score a track against the Match-V5 timeline.

  python -m scout.score run --track RUN/track_minimap.json --timeline RUN/timeline.json
      --match RUN/match.json --out RUN [--wards-track RUN/track_topdown.json] [--tag NAME]
  python -m scout.score plots --score RUN/score.json --out RUN/plots [--minimap FRAME.png]

Positions: the timeline's 60 s participantFrames are ground truth; the track
sample nearest each frame (within +-1 s) is compared, in world units. The
detection rate counts samples with a position over all samples: the spike
treats every champion as alive throughout (dead time is counted as missed).
Wards: track wards matched one-to-one to WARD_PLACED within +-2 s (--ward-tol), per team;
with the ward track's ward_window (R2) only WARD_PLACED inside it count and wards
marked pre_existing are left out
(creatorId 1-5 blue, 6-10 red). Recalls: a path that arrives inside the own
fountain radius and stays there; the stay is counted inclusively (last minus
first inside sample, plus one sample step); missing samples do not end it,
and neither do positions outside the radius spanning <= 2 s (RECALL_BRIDGE_S). A stay >= 8 s counts;
5 to 8 s counts only with a purchase (a short fountain visit is otherwise
indistinguishable from walking past). Confirmed = >= 1 ITEM_PURCHASED by that
participant within +-15 s of arrival; an arrival within 120 s after that
champion's death is a respawn, not a recall.
A champion whose positions all sit at one spot (within FALSE_LOCK_RADIUS) for
the whole run is a false lock on a static background spot (finding F): it
counts as not detected (per_champion false_lock). --drop-static removes, before
scoring, track positions at spots a champion's track recurs at in >= 20 % of
samples (the tracker's static-spot rule, in world units, for tracks made
before it).
"""

from __future__ import annotations

import argparse
import bisect
import csv
import statistics
from pathlib import Path

import numpy as np
from scipy.optimize import linear_sum_assignment

from .track import StaticSpots, find_gaps
from .util import mmss, read_json, sample_evenly, write_json

# Fountain centres (world units) and radius. The centres are the Match-V5
# spawn positions every participant has in frame 0 (blue 554,581; red
# 14302,14387), as in the repo fixture tests/fixtures/riot_match_v5_timeline.json
# (checked 2026-10-17; was an estimate from the map bounds). The radius covers
# the platform and shop: verify on run.
FOUNTAINS = {"blue": (554.0, 581.0), "red": (14302.0, 14387.0)}
FOUNTAIN_RADIUS = 1000.0
RECALL_MIN_STAY_S = 8.0          # counts with or without a purchase
RECALL_SHORT_STAY_S = 5.0        # 5..8 s counts only with a purchase
RECALL_BRIDGE_S = 2.0            # outside stretches this short do not end a stay
PURCHASE_WINDOW_S = 15.0
RESPAWN_WINDOW_S = 120.0
WARD_TOL_S = 2.0
FRAME_TOL_S = 1.0
FALSE_LOCK_RADIUS = 100.0       # world units, ~4.5 px on a 679 px minimap
FALSE_LOCK_MIN_SAMPLES = 10
SKIP_WARD_TYPES = {"UNDEFINED"}  # timeline entries that are not placed wards


def team_of(pid: int) -> str:
    return "blue" if pid <= 5 else "red"


def participant_map(match: dict | None = None, mapping: str | None = None) -> dict[int, str]:
    """participantId -> champion, from match.json or '1=Aatrox,2=LeeSin,...'."""
    if mapping:
        return {int(k): v for k, v in (pair.split("=") for pair in mapping.split(","))}
    return {p["participantId"]: p["championName"] for p in match["info"]["participants"]}


def events(timeline: dict, kind: str) -> list[dict]:
    return [e for f in timeline["info"]["frames"] for e in f.get("events", []) if e["type"] == kind]


class Lookup:
    """Nearest track sample to a time, within a tolerance."""

    def __init__(self, samples: list[dict]):
        self.samples = sorted(samples, key=lambda s: s["t"])
        self.ts = [s["t"] for s in self.samples]

    def at(self, t: float, tol: float = FRAME_TOL_S) -> dict | None:
        i = bisect.bisect_left(self.ts, t)
        best = None
        for j in (i - 1, i):
            if 0 <= j < len(self.ts) and abs(self.ts[j] - t) <= tol:
                if best is None or abs(self.ts[j] - t) < abs(best["t"] - t):
                    best = self.samples[j]
        return best


def false_locks(track: dict, radius: float = FALSE_LOCK_RADIUS,
                min_samples: int = FALSE_LOCK_MIN_SAMPLES) -> list[str]:
    """Champions whose every position sits within `radius` of their median."""
    out = []
    for c in track["champions"]:
        pts = np.array([s["pos"][c] for s in track["samples"] if s["pos"].get(c)], dtype=float)
        if len(pts) < min_samples:
            continue
        mid = np.median(pts, axis=0)
        if np.all(np.hypot(*(pts - mid).T) <= radius):
            out.append(c)
    return out


class _Pt:
    def __init__(self, label, x, y):
        self.label, self.x, self.y = label, x, y


def drop_static(track: dict, radius: float = FALSE_LOCK_RADIUS, frac: float = 0.2,
                warmup: int = 60) -> tuple[dict, dict]:
    """A copy of the track without positions at static spots (world units)."""
    spots = StaticSpots(radius=radius, frac=frac, warmup=warmup)
    for s in track["samples"]:
        spots.observe([_Pt(c, *p) for c, p in s["pos"].items() if p])
    static = spots.static()
    samples = []
    for s in track["samples"]:
        pos = {}
        for c, p in s["pos"].items():
            pos[c] = None if p and spots.is_static(c, p[0], p[1], static) else p
            if p and pos[c] is None:
                spots.rejected[c] = spots.rejected.get(c, 0) + 1
        samples.append({**s, "pos": pos})
    return {**track, "samples": samples}, spots.describe()


def score_positions(track: dict, timeline: dict, pmap: dict[int, str]) -> dict:
    locked = false_locks(track)
    if locked:   # a false lock counts as not detected
        track = {**track, "samples": [{**s, "pos": {c: (None if c in locked else p)
                                                    for c, p in s["pos"].items()}}
                                      for s in track["samples"]]}
    look = Lookup(track["samples"])
    errors = []
    for frame in timeline["info"]["frames"]:
        t = frame["timestamp"] / 1000.0
        s = look.at(t)
        if s is None:
            continue
        for pid_s, pf in frame["participantFrames"].items():
            champ = pmap.get(int(pid_s))
            got = s["pos"].get(champ) if champ else None
            if got is None or "position" not in pf:
                continue
            truth = (pf["position"]["x"], pf["position"]["y"])
            errors.append({"t": t, "champion": champ, "err": round(float(np.hypot(
                got[0] - truth[0], got[1] - truth[1])), 1), "track": got, "truth": list(truth)})
    n = len(track["samples"])
    gaps = find_gaps(track["samples"], track["champions"])
    per = {}
    for c in track["champions"]:
        errs = [e["err"] for e in errors if e["champion"] == c]
        seen = sum(1 for s in track["samples"] if s["pos"].get(c) is not None)
        cg = [g["length_s"] for g in gaps if g["champion"] == c]
        per[c] = {"median_error": statistics.median(errs) if errs else None, "n_scored": len(errs),
                  "detection_rate": round(seen / n, 4) if n else None,
                  "gaps_count": len(cg), "longest_gap_s": max(cg) if cg else 0.0,
                  "false_lock": c in locked}
    all_err = [e["err"] for e in errors]
    seen_all = sum(1 for s in track["samples"] for c in track["champions"] if s["pos"].get(c))
    return {"median_error": statistics.median(all_err) if all_err else None,
            "n_scored": len(all_err),
            "detection_rate": round(seen_all / (n * len(track["champions"])), 4) if n else None,
            "gaps_count": len(gaps),
            "longest_gap_s": max((g["length_s"] for g in gaps), default=0.0),
            "false_lock": locked, "per_champion": per, "errors": errors}


def match_wards(wards: list[dict], timeline: dict, pmap: dict[int, str],
                t_range: tuple[float, float] | None, tol: float = WARD_TOL_S,
                window: list[float] | None = None) -> dict:
    """window: the ward track's own ward_window (R2): WARD_PLACED after its
    start and up to its end count, without the tolerance on either side, and
    wards marked pre_existing (visible when the window opened) are left out."""
    truth = [e for e in events(timeline, "WARD_PLACED")
             if e.get("wardType") not in SKIP_WARD_TYPES and e.get("creatorId", 0) > 0]
    if window:
        truth = [e for e in truth if window[0] < e["timestamp"] / 1000 <= window[1]]
    elif t_range:
        truth = [e for e in truth if t_range[0] - tol <= e["timestamp"] / 1000 <= t_range[1] + tol]
    pre = [w for w in wards if w.get("pre_existing")]
    wards = [w for w in wards if not w.get("pre_existing")]
    out = {"pairs": [], "tolerance_s": tol, "pre_existing_excluded": len(pre), "window": window}
    for team in ("blue", "red"):
        det = [w for w in wards if w.get("team") == team]
        tru = [e for e in truth if team_of(e["creatorId"]) == team]
        tp = 0
        if det and tru:
            cost = np.array([[abs(w["t"] - e["timestamp"] / 1000) for e in tru] for w in det])
            for r, k in zip(*linear_sum_assignment(cost)):
                if cost[r, k] <= tol:
                    tp += 1
                    out["pairs"].append({"ward": det[r], "event": tru[k],
                                         "champion": pmap.get(tru[k]["creatorId"])})
        out[team] = {"detected": len(det), "placed": len(tru), "tp": tp,
                     "fp": len(det) - tp, "fn": len(tru) - tp,
                     "precision": round(tp / len(det), 4) if det else None,
                     "recall": round(tp / len(tru), 4) if tru else None}
    matched = {id(p["ward"]) for p in out["pairs"]}
    out["unmatched_wards"] = [w for w in wards if id(w) not in matched]
    hit = {id(p["event"]) for p in out["pairs"]}
    out["missed_events"] = [e for e in truth if id(e) not in hit]
    return out


def _sample_step(samples: list[dict]) -> float:
    ts = sorted(s["t"] for s in samples)
    diffs = [b - a for a, b in zip(ts, ts[1:]) if b > a]
    return float(statistics.median(diffs)) if diffs else 1.0


def detect_recalls(track: dict, timeline: dict, pmap: dict[int, str]) -> list[dict]:
    champ_pid = {c: p for p, c in pmap.items()}
    buys = events(timeline, "ITEM_PURCHASED")
    kills = events(timeline, "CHAMPION_KILL")
    out = []
    for idx, champ in enumerate(track["champions"]):
        pid = champ_pid.get(champ, idx + 1)
        fx, fy = FOUNTAINS[team_of(pid)]
        pts = [(s["t"], s["pos"].get(champ)) for s in track["samples"]]
        pts = [(t, p) for t, p in pts if p is not None]
        inside = [np.hypot(p[0] - fx, p[1] - fy) <= FOUNTAIN_RADIUS for _, p in pts]
        step = _sample_step(track["samples"])
        i = 1
        while i < len(pts):
            if inside[i] and not inside[i - 1]:
                j = i
                while True:
                    k = next((m for m in range(j + 1, len(pts)) if inside[m]), None)
                    # missing samples never end a stay (as before); positions
                    # outside the radius end it unless they span <= 2 s
                    if k is None or (k > j + 1 and pts[k - 1][0] - pts[j + 1][0] + step
                                     > RECALL_BRIDGE_S + 1e-9):
                        break
                    j = k
                arrive, stay = pts[i][0], pts[j][0] - pts[i][0] + step
                n_buy = sum(1 for e in buys if e["participantId"] == pid
                            and abs(e["timestamp"] / 1000 - arrive) <= PURCHASE_WINDOW_S)
                if stay >= RECALL_MIN_STAY_S or (stay >= RECALL_SHORT_STAY_S and n_buy >= 1):
                    died = any(e.get("victimId") == pid and
                               0 <= arrive - e["timestamp"] / 1000 <= RESPAWN_WINDOW_S for e in kills)
                    if not died:
                        out.append({"champion": champ, "participantId": pid, "t": arrive,
                                    "stay_s": round(stay, 3), "purchases": n_buy,
                                    "confirmed": n_buy >= 1, "x": pts[i][1][0], "y": pts[i][1][1]})
                i = j + 1
            else:
                i += 1
    return sorted(out, key=lambda r: r["t"])


def handcheck_rows(wards: dict, recalls: list[dict], pmap: dict[int, str], n: int = 10) -> list[dict]:
    ward_rows = []
    for p in wards["pairs"]:
        e = p["event"]
        ward_rows.append(dict(kind="ward", t=p["ward"]["t"], champion=p["champion"] or "",
                              x=p["ward"]["x"], y=p["ward"]["y"],
                              auto_says=f"ward; WARD_PLACED {e.get('wardType', '')} at "
                                        f"{mmss(e['timestamp'] / 1000)}"))
    for w in wards["unmatched_wards"]:
        ward_rows.append(dict(kind="ward", t=w["t"], champion="", x=w["x"], y=w["y"],
                              auto_says=f"{w['team']} ward; no WARD_PLACED within {wards['tolerance_s']:g} s"))
    rec_rows = [dict(kind="recall", t=r["t"], champion=r["champion"], x=r["x"], y=r["y"],
                     auto_says=(f"recall, {r['purchases']} purchase(s) within 15 s" if r["confirmed"]
                                else "recall, no purchase within 15 s")) for r in recalls]
    missed = [dict(kind="ward_missed", t=e["timestamp"] / 1000,
                   champion=pmap.get(e["creatorId"], ""), x="", y="",
                   auto_says="timeline WARD_PLACED; track found no ward")
              for e in wards["missed_events"]]
    ward_rows.sort(key=lambda r: r["t"])
    picked_w = sample_evenly(ward_rows, 5)
    picked_r = sample_evenly(rec_rows, 5)
    rest = [r for r in ward_rows + rec_rows if r not in picked_w and r not in picked_r]
    rows = picked_w + picked_r
    for pool in (rest, sorted(missed, key=lambda r: r["t"])):
        need = n - len(rows)
        if need > 0:
            rows += sample_evenly(pool, need)
    rows.sort(key=lambda r: r["t"])
    out = []
    for i, r in enumerate(rows[:n], 1):
        out.append({"event_id": i, "kind": r["kind"], "game_time_mmss": mmss(r["t"]),
                    "champion": r["champion"],
                    "x": r["x"] if r["x"] == "" else round(r["x"]),
                    "y": r["y"] if r["y"] == "" else round(r["y"]),
                    "auto_says": r["auto_says"], "jasper_says": "", "note": ""})
    return out


HANDCHECK_COLUMNS = ["event_id", "kind", "game_time_mmss", "champion", "x", "y",
                     "auto_says", "jasper_says", "note"]


def run(args) -> int:
    track = read_json(args.track)
    timeline = read_json(args.timeline)
    match = read_json(args.match) if args.match else None
    pmap = participant_map(match, args.participants)
    if args.wards_track and not Path(args.wards_track).exists():
        print(f"warning: {args.wards_track} not found; wards taken from {args.track}")
        args.wards_track = None
    wards_track = read_json(args.wards_track) if args.wards_track else track
    ts = [s["t"] for s in wards_track["samples"]]
    result = {"track": Path(args.track).name, "source": track.get("source"),
              "matchId": track.get("matchId"), "detector": track.get("detector"),
              "missing_classes": track.get("missing_classes", []),
              "notes": ["detection_rate counts all samples (alive approximated as always)",
                        f"fountain radius {FOUNTAIN_RADIUS}, centres {FOUNTAINS}"]}
    if args.drop_static:
        track, result["static_spots"] = drop_static(track)
        result["notes"].append("--drop-static: positions at static spots removed before scoring")
    result.update(score_positions(track, timeline, pmap))
    wards = match_wards(wards_track.get("wards", []), timeline, pmap,
                        (min(ts), max(ts)) if ts else None, args.ward_tol, wards_track.get("ward_window"))
    result["wards"] = {k: v for k, v in wards.items()
                       if k in ("blue", "red", "tolerance_s", "pre_existing_excluded", "window")}
    result["wards"]["source"] = Path(args.wards_track or args.track).name
    result["recalls"] = detect_recalls(track, timeline, pmap)
    tag = f"_{args.tag}" if args.tag else ""
    out = Path(args.out)
    write_json(out / f"score{tag}.json", result)
    with open(out / f"handcheck{tag}.csv", "w", newline="", encoding="utf-8") as fh:
        w = csv.DictWriter(fh, fieldnames=HANDCHECK_COLUMNS)
        w.writeheader()
        w.writerows(handcheck_rows(wards, result["recalls"], pmap))
    print(f"median error {result['median_error']} (n={result['n_scored']}), detection rate "
          f"{result['detection_rate']}, recalls {len(result['recalls'])} -> {out / f'score{tag}.json'}")
    return 0


def plots(args) -> int:
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    from .calibrate import MAP_X, MAP_Y

    score = read_json(args.score)
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    per = score["per_champion"]
    names = list(per)
    vals = [per[c]["median_error"] or 0 for c in names]
    fig, ax = plt.subplots(figsize=(8, 4))
    ax.bar(names, vals, color=["#3b6fb6"] * 5 + ["#b64a3b"] * (len(names) - 5))
    ax.set_ylabel("median error (world units)")
    ax.set_title(f"Median position error per champion ({score.get('source')})")
    ax.tick_params(axis="x", rotation=45)
    fig.tight_layout()
    fig.savefig(out / "error_per_champion.png", dpi=110)
    plt.close(fig)
    times = sorted({e["t"] for e in score["errors"]})
    bg = None
    if args.minimap:
        import cv2

        bg = cv2.cvtColor(cv2.imread(str(args.minimap)), cv2.COLOR_BGR2RGB)
    for t in sample_evenly(times, 3):
        fig, ax = plt.subplots(figsize=(6, 6))
        if bg is not None:
            ax.imshow(bg, extent=(MAP_X[0], MAP_X[1], MAP_Y[0], MAP_Y[1]))
        ax.set_xlim(*MAP_X)
        ax.set_ylim(*MAP_Y)
        for e in (e for e in score["errors"] if e["t"] == t):
            (tx, ty), (gx, gy) = e["truth"], e["track"]
            ax.plot([tx, gx], [ty, gy], color="#999999", lw=1)
            ax.scatter([tx], [ty], marker="x", color="white" if bg is not None else "black")
            ax.scatter([gx], [gy], marker="o", facecolors="none", edgecolors="#e0b000")
            ax.annotate(e["champion"], (gx, gy), fontsize=8, color="#e0b000")
        ax.set_title(f"t={mmss(t)}: x = timeline, o = track")
        fig.savefig(out / f"overlay_{int(t):05d}.png", dpi=100)
        plt.close(fig)
    print(f"plots written to {out}")
    return 0


def main(argv=None) -> int:
    import sys

    argv = list(sys.argv[1:] if argv is None else argv)
    if not argv or argv[0] not in ("run", "plots", "-h", "--help"):
        argv.insert(0, "run")
    ap = argparse.ArgumentParser(prog="scout.score", description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run", help="score a track")
    r.add_argument("--track", required=True, type=Path)
    r.add_argument("--timeline", required=True, type=Path)
    r.add_argument("--match", type=Path, help="match.json for participantId -> champion")
    r.add_argument("--participants", help="instead of --match: '1=Aatrox,2=LeeSin,...'")
    r.add_argument("--wards-track", type=Path, help="take wards from this track (e.g. top-down)")
    r.add_argument("--ward-tol", type=float, default=WARD_TOL_S,
                   help=f"ward time tolerance, s (default {WARD_TOL_S}; 4 recovered one more red ward "
                        "on Jasper's run)")
    r.add_argument("--drop-static", action="store_true",
                   help="remove positions at a champion's recurring static spots first (finding F)")
    r.add_argument("--tag", help="suffix for score_<tag>.json / handcheck_<tag>.csv")
    r.add_argument("--out", required=True, type=Path)
    p = sub.add_parser("plots", help="error bar chart and minimap overlays")
    p.add_argument("--score", required=True, type=Path)
    p.add_argument("--minimap", type=Path, help="a minimap frame as the overlay background")
    p.add_argument("--out", required=True, type=Path)
    args = ap.parse_args(argv)
    if args.cmd == "run" and not (args.match or args.participants):
        ap.error("run needs --match or --participants")
    return run(args) if args.cmd == "run" else plots(args)


if __name__ == "__main__":
    raise SystemExit(main())
