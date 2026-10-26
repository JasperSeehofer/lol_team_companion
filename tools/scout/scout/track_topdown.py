"""Route B: the whole map in one top-down view, champions and wards found by
their health bars.

Live (player's PC; run in a fresh desktop session if a recording already ran):
  python -m scout.track_topdown --out RUN_DIR [--keystroke-unlock] [--start 600]
Offline (saved frames tNNNNN.png):
  python -m scout.track_topdown --out RUN_DIR --frames DIR [--anchors anchors.json]
      [--minimap-track RUN_DIR/track_minimap.json] [--timeline RUN_DIR/timeline.json]
Rebuild a track from its stored detections (no frames):
  python -m scout.track_topdown --out DIR --from-detections RUN_DIR/track_topdown.json

Live mode records 5 min through /replay/recording (png codec) into a scratch
directory outside the repo (default %TEMP%/scout-rec), keeps one frame per game
second for tracking and copies up to --max-frames of them into
RUN_DIR/frames_topdown/.

Positions (R2): the first second of the recording is dropped (stale), each
champion's bar blobs are merged and anchored on the bar's left edge, and
unit = left edge + offset. With timeline.json the offset and the homography
are fitted against the timeline's participantFrames (frames within 0.5 s),
identities by Hungarian against the minimap track (or, without one, against
the timeline itself: an oracle fit, reported as such); the fit report gives
the in-sample and leave-one-frame-out median errors. With anchors.json (turret
pixels) the homography comes from the anchors and only the offset is fitted.
With neither, a template is written and the detections stay in pixels.
"""

from __future__ import annotations

import argparse
import math
import os
import shutil
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path

import cv2
import numpy as np
from scipy.optimize import linear_sum_assignment

from . import calibrate
from .camera import REF_FOV, REF_Y, default_camera_y
from .detect import BarDetector
from .render import preserved_render
from .replay_api import DEFAULT_BASE, ReplayClient
from .track import find_gaps
from .util import frame_name, list_frames, read_json, run_logger, sample_evenly, write_json

RENDER_TOPDOWN = {
    "interfaceAll": False, "fogOfWar": False, "floatingText": False,
    "healthBarStructures": False, "healthBarMinions": False, "healthBarPets": False,
    "healthBarChampions": True, "healthBarWards": True, "minions": False, "particles": False,
}
MAP_CENTRE = (7375.0, 7430.0)
IDENTITY_GATE = 2000.0   # world units between a bar and the minimap track


def camera_render(height: float, fov: float) -> dict:
    """Straight-down free camera over the map centre. pyLoL's preset used
    height 57447, fieldOfView 19, farClip 66010, rotation y 85 (verify on run)."""
    return {"cameraMode": "fps", "cameraAttached": False,
            "cameraPosition": {"x": MAP_CENTRE[0], "y": height, "z": MAP_CENTRE[1]},
            "cameraRotation": {"x": 0.0, "y": 90.0, "z": 0.0},
            "fieldOfView": fov, "farClip": height * 1.5 + 10000}


def camera_height(args) -> tuple[float, str]:
    """--camera-y, else the probe's visual fit (same fov), else the default
    scaled to --width/--height (scout.camera)."""
    if args.camera_y is not None:
        return float(args.camera_y), "--camera-y"
    probe = read_json(args.out / "probe.json") if (args.out / "probe.json").exists() else {}
    fit = probe.get("camera_fit") or {}
    if fit.get("chosen_y") and float(fit.get("fov", -1)) == float(args.fov):
        return float(fit["chosen_y"]), f"probe.json camera_fit ({fit.get('chosen_by')})"
    return default_camera_y(args.width, args.height, args.fov), \
        f"scaled default for {args.width}x{args.height}"


def record_live(client: ReplayClient, args, log) -> tuple[Path, dict]:
    client.set_render(**RENDER_TOPDOWN)
    camera = {"mode": "keystroke" if args.keystroke_unlock else "api"}
    if args.keystroke_unlock:
        input("In the replay window press Ctrl+Shift+Y, zoom out with the mouse wheel until the "
              "whole map fits, then come back here and press Enter... ")
    else:
        height, source = camera_height(args)
        client.set_render(**camera_render(height, args.fov))
        camera.update(height=height, fov=args.fov, height_from=source)
        log.info("camera height %.0f (%s), fov %s", height, source, args.fov)
    rec_dir = args.scratch / time.strftime("%Y%m%d-%H%M%S")
    rec_dir.mkdir(parents=True, exist_ok=True)
    end = args.start + args.duration
    client.set_playback(time=args.start, paused=True, speed=1.0)
    time.sleep(1.0)
    client.set_recording(recording=True, codec="png", path=str(rec_dir), startTime=args.start,
                         endTime=end, framesPerSecond=args.fps, enforceFrameRate=True,
                         replaySpeed=1.0, width=args.width, height=args.height, lossless=True)
    client.set_playback(paused=False)
    wall0 = time.time()
    deadline = wall0 + args.duration * 1.5 + 120
    while time.time() < deadline:
        time.sleep(2.0)
        if not client.recording().get("recording"):
            break
    wall = time.time() - wall0
    # The client writes into its own subfolder (e.g. "16-21_EUW1-8021305006_01") under the path it is
    # given, and may ignore width/height/endTime (seen 2026-10-22: 2560x1440, endTime -1). Search the
    # whole scratch dir, and log what the client reports so a failure explains itself.
    try:
        state = client.recording()
    except Exception as e:  # the replay may already be closed
        state = {"error": str(e)}
    log.info("client recording state at stop: %s", {k: state.get(k) for k in
             ("recording", "path", "startTime", "endTime", "currentTime", "width", "height")})
    pngs = sorted(p for p in rec_dir.rglob("*.png") if p.parent.name != "1hz")
    log.info("recording: %d png frames in %.0f s wall under %s", len(pngs), wall, rec_dir)
    per_s = rec_dir / "1hz"
    per_s.mkdir(exist_ok=True)
    step = max(1, int(round(args.fps)))
    for i in range(0, len(pngs), step):
        shutil.copy2(pngs[i], per_s / frame_name(args.start + i / args.fps))
    return per_s, {"camera": camera, "frames_written": len(pngs), "wall_s": round(wall, 1),
                   "fps": args.fps, "size": [args.width, args.height]}


def nearest_sample(track: dict | None, t: float, tol: float = 1.0) -> dict | None:
    if not track:
        return None
    best = min(track["samples"], key=lambda s: abs(s["t"] - t), default=None)
    return best if best and abs(best["t"] - t) <= tol else None


def assign_identities(t: float, bars: list[tuple[str, float, float]], ref: dict | None,
                      champions: list[str], gate: float = IDENTITY_GATE) -> dict:
    """Hungarian per team between world-space bars and reference positions
    ({champion: [x, y]}, e.g. the minimap track's sample at t)."""
    pos = {c: None for c in champions}
    if not ref:
        return pos
    for team, members in (("blue", champions[:5]), ("red", champions[5:])):
        cands = [m for m in members if ref.get(m)]
        team_bars = [(x, y) for tm, x, y in bars if tm == team]
        if not cands or not team_bars:
            continue
        cost = np.array([[np.hypot(ref[c][0] - x, ref[c][1] - y)
                          for x, y in team_bars] for c in cands])
        for r, k in zip(*linear_sum_assignment(cost)):
            if cost[r, k] <= gate:
                x, y = team_bars[k]
                pos[cands[r]] = [round(x, 1), round(y, 1)]
    return pos


# ---------------------------------------------------------------- bars -> unit positions (R2)
#
# Measured on Jasper's run (2026-10-22: 2560x1440, camera y 50000, fov 19):
# - the recording's first frame (at startTime) is stale, it still shows the
#   game before the seek: the first STALE_S seconds of every recording are dropped;
# - a champion health bar splits into several blobs at its 1000-HP separators
#   and its centroid moves with the HP left: blobs on one row are merged and the
#   bar is anchored on its LEFT edge, which does not move;
# - the unit stands REF_BAR_OFFSET px right of and below that left edge.
#   Screen-space sizes scale with the frame height (UI scale); the vertical
#   offset is a height above the unit, so it also scales with the camera
#   (height and fov). Fitted per run when the timeline is at hand.

PROCESSING = "r2"
STALE_S = 1.0
REF_FRAME_H = 1440.0
REF_BAR_OFFSET = (51.5, 144.4)    # merged bar left edge -> unit, px (reference screen and camera)
BAR_MERGE_DY = 3.0                # blobs of one bar: rows this close (px at the reference) ...
BAR_MERGE_GAP = 8.0               # ... and horizontal gaps this small
MAP_SIDE_PX = 1200.0              # map bounds' side on screen at the reference (initial guess)
# top-down frame <-> timeline participantFrames. Not the score's 1 s: champions
# move ~350 units/s, so on Jasper's run the frame at 601 against the frame
# stamped 600.249 alone cost ~250 units (fit LOFO 152 -> 128 without it).
FIT_FRAME_TOL_S = 0.5
FIT_GATES = (2500.0, 1500.0, 800.0, 600.0, 600.0)
RANSAC_WORLD = 300.0
MIN_FIT_PAIRS = 6


def ui_scale(frame_size) -> float:
    return float(frame_size[1]) / REF_FRAME_H


def camera_scale(frame_size, camera: dict | None) -> float:
    """Screen size of world distances relative to the reference camera."""
    fov = float((camera or {}).get("fov") or REF_FOV)
    y = (camera or {}).get("height") or default_camera_y(int(frame_size[0]), int(frame_size[1]), fov)
    return (REF_Y * math.tan(math.radians(REF_FOV) / 2)) / (float(y) * math.tan(math.radians(fov) / 2))


def default_bar_offset(frame_size, camera: dict | None = None) -> tuple[float, float]:
    s = ui_scale(frame_size)
    return (REF_BAR_OFFSET[0] * s, REF_BAR_OFFSET[1] * s * camera_scale(frame_size, camera))


def bar_left(d: dict) -> float:
    """Left edge of a bar blob: the bounding box when recorded, else from the
    centroid (pixels left..left+w-1)."""
    if d.get("left") is not None:
        return float(d["left"])
    return float(d["x"]) - (float(d.get("w") or 0.0) - 1.0) / 2.0


def merge_bars(dets: list[dict], scale: float = 1.0) -> list[dict]:
    """Champion bar blobs joined across the HP separators: same team, rows
    within BAR_MERGE_DY, gaps within BAR_MERGE_GAP (both times the UI scale)."""
    segs = sorted((d for d in dets if d.get("label") == "champion"), key=bar_left)
    groups: list[dict] = []
    for d in segs:
        left = bar_left(d)
        right = left + float(d.get("w") or 0.0)
        g = next((g for g in groups if g["team"] == d.get("team")
                  and abs(g["y"] - float(d["y"])) <= BAR_MERGE_DY * scale
                  and left - g["right"] <= BAR_MERGE_GAP * scale), None)
        if g:
            g["right"] = max(g["right"], right)
            g["blobs"] += 1
        else:
            groups.append({"team": d.get("team"), "left": left, "right": right,
                           "y": float(d["y"]), "blobs": 1})
    return groups


def _translate(dx: float, dy: float) -> np.ndarray:
    return np.array([[1.0, 0.0, dx], [0.0, 1.0, dy], [0.0, 0.0, 1.0]])


@dataclass
class BarMap:
    """Merged bar (left edge, row) -> unit pixel (+ offset) -> world (H)."""

    H: np.ndarray            # unit pixel -> world
    offset: tuple[float, float]

    def to_world(self, left: float, y: float) -> tuple[float, float]:
        v = self.H @ np.array([left + self.offset[0], y + self.offset[1], 1.0])
        return float(v[0] / v[2]), float(v[1] / v[2])

    def pixel_to_world(self, px: float, py: float) -> tuple[float, float]:
        v = self.H @ np.array([px, py, 1.0])
        return float(v[0] / v[2]), float(v[1] / v[2])

    def unit_pixel(self, wx: float, wy: float) -> tuple[float, float]:
        v = np.linalg.inv(self.H) @ np.array([wx, wy, 1.0])
        return float(v[0] / v[2]), float(v[1] / v[2])

    def as_dict(self) -> dict:
        return {"kind": "homography", "H": self.H.tolist(), "applies_to": "unit pixel = bar left + offset",
                "offset_px": [round(self.offset[0], 2), round(self.offset[1], 2)]}


def initial_map(frame_size, camera: dict | None, offset) -> BarMap:
    """Crude start: the map bounds as a square centred on the frame."""
    side = MAP_SIDE_PX * ui_scale(frame_size) * camera_scale(frame_size, camera)
    cx, cy = frame_size[0] / 2.0, frame_size[1] / 2.0
    src = np.float32([[cx - side / 2, cy - side / 2], [cx + side / 2, cy - side / 2],
                      [cx + side / 2, cy + side / 2], [cx - side / 2, cy + side / 2]])
    (x0, x1), (y0, y1) = calibrate.MAP_X, calibrate.MAP_Y
    dst = np.float32([[x0, y1], [x1, y1], [x1, y0], [x0, y0]])
    return BarMap(cv2.getPerspectiveTransform(src, dst).astype(float), tuple(offset))


def truth_frames(timeline: dict | None, pmap: dict[int, str]) -> list[tuple[float, dict]]:
    """(t, {champion: [x, y]}) per timeline participantFrames frame; a dead
    champion (frame position = the spot of a CHAMPION_KILL in the last 60 s)
    is left out: it has no bar."""
    frames = (timeline or {}).get("info", {}).get("frames", [])
    kills = [e for f in frames for e in f.get("events", []) if e.get("type") == "CHAMPION_KILL"]
    out = []
    for f in frames:
        pos = {}
        for pid, pf in (f.get("participantFrames") or {}).items():
            c = pmap.get(int(pid))
            if not c or "position" not in pf:
                continue
            x, y = float(pf["position"]["x"]), float(pf["position"]["y"])
            dead = any(e.get("victimId") == int(pid) and 0 <= f["timestamp"] - e["timestamp"] <= 60000
                       and abs((e.get("position") or {}).get("x", 1e9) - x) < 5
                       and abs((e.get("position") or {}).get("y", 1e9) - y) < 5 for e in kills)
            if not dead:
                pos[c] = [x, y]
        out.append((f["timestamp"] / 1000.0, pos))
    return out


def _pairs(mapping: BarMap, frames: list[dict], champions: list[str], gate: float) -> list[dict]:
    """Bars matched to champions (Hungarian against each frame's identity
    reference), each with the timeline's position as the target."""
    out = []
    for fr in frames:
        bars = [(b["team"], *mapping.to_world(b["left"], b["y"])) for b in fr["bars"]]
        ident = assign_identities(fr["t"], bars, fr["ident"], champions, gate)
        by_world = {(round(x, 1), round(y, 1)): b for b, (_, x, y) in zip(fr["bars"], bars)}
        for c, p in ident.items():
            if p is None or c not in fr["truth"]:
                continue
            b = by_world[tuple(p)]
            out.append({"t": fr["t"], "champion": c, "left": b["left"], "y": b["y"],
                        "world": fr["truth"][c]})
    return out


def _errors(mapping: BarMap, pairs: list[dict]) -> np.ndarray:
    return np.array([np.hypot(*(np.subtract(mapping.to_world(p["left"], p["y"]), p["world"])))
                     for p in pairs])


def _fit_once(frames, champions, start: BarMap, anchored: bool):
    mapping, pairs, inliers = start, [], None
    for gate in FIT_GATES:
        cand = _pairs(mapping, frames, champions, gate)
        if len(cand) < (3 if anchored else MIN_FIT_PAIRS):
            break
        pairs = cand
        if anchored:   # homography fixed by the turret anchors: fit the offset only
            res = np.array([np.subtract(mapping.unit_pixel(*p["world"]), (p["left"], p["y"]))
                            for p in pairs])
            mapping = BarMap(mapping.H, (float(np.median(res[:, 0])), float(np.median(res[:, 1]))))
        else:          # bar -> world directly; the default offset is folded into H
            src = np.float64([(p["left"], p["y"]) for p in pairs])
            dst = np.float64([p["world"] for p in pairs])
            Hb, mask = cv2.findHomography(src, dst, cv2.RANSAC, RANSAC_WORLD)
            if Hb is None:
                break
            inliers = mask.ravel().astype(bool)
            mapping = BarMap(Hb @ _translate(-start.offset[0], -start.offset[1]), start.offset)
    return mapping, pairs, inliers


def fit_topdown(frames: list[dict], champions: list[str], start: BarMap, anchored: bool,
                identity: str) -> tuple[BarMap | None, dict]:
    """frames: [{"t", "bars" (merged), "truth" {champ: [x, y]}, "ident" {champ: [x, y]}}]
    at the timeline's minute frames. Returns the mapping and a report with the
    in-sample and leave-one-frame-out (LOFO) median errors in world units."""
    mapping, pairs, inliers = _fit_once(frames, champions, start, anchored)
    if not pairs:
        return None, {"source": "timeline", "identity": identity, "frames": [f["t"] for f in frames],
                      "n_pairs": 0, "reason": "no bar matched a champion within the gates"}
    err = _errors(mapping, pairs)
    lofo, per_frame = [], {}
    for fr in frames:
        rest = [f for f in frames if f is not fr]
        if not rest:
            break
        m_i, p_i, _ = _fit_once(rest, champions, start, anchored)
        if not p_i:
            continue
        held = _pairs(m_i, [fr], champions, FIT_GATES[-1])
        e = _errors(m_i, held) if held else np.array([])
        lofo += e.tolist()
        per_frame[f"{fr['t']:g}"] = {"in_sample": round(float(np.median(err[[p["t"] == fr["t"] for p in pairs]])), 1)
                                     if any(p["t"] == fr["t"] for p in pairs) else None,
                                     "lofo": round(float(np.median(e)), 1) if len(e) else None,
                                     "n": int(len(e))}
    report = {
        "source": "timeline", "identity": identity,
        "identity_note": ("oracle fit: identities from the timeline positions themselves, not "
                          "available on a scouting run" if identity == "oracle"
                          else "identities from the minimap track at the same time"),
        "mode": "anchors" if anchored else "direct",
        "offset_px": [round(mapping.offset[0], 2), round(mapping.offset[1], 2)],
        "offset_from": "fitted (homography from the turret anchors)" if anchored
        else "default, folded into the fitted homography",
        "frames": [f["t"] for f in frames], "n_pairs": len(pairs),
        "n_inliers": int(inliers.sum()) if inliers is not None else len(pairs),
        "median_error": round(float(np.median(err)), 1),
        "p90_error": round(float(np.percentile(err, 90)), 1),
        "lofo_median": round(float(np.median(lofo)), 1) if lofo else None,
        "lofo_n": len(lofo), "per_frame": per_frame,
    }
    return mapping, report


# ---------------------------------------------------------------- wards (R2)
#
# Measured on Jasper's run (px at 2560x1440; scaled by the frame height):
# - mana bars are blue for both teams, wide (w > 40, median 109, h 4) and sit
#   ~11.8 px below the champion bar's centre row, left-aligned with it (HSV
#   ~100/174/222); they are not wards;
# - a ward's bar is 3 pips (sight ward or trinket, spacing 24-25 px) or 4 pips
#   (control ward, spacing 18 px), each 15-24 px wide; blue pips are fully
#   saturated (S 255; champion HP ~195, mana ~174);
# - the ward shows up 0.0-0.8 s (median 0.15) after its WARD_PLACED in the
#   1 Hz frames: a ward is stamped at its first appearance minus WARD_LAG_S.
# Pips on one row are one ward; the same ward is followed across frames in
# pixels (the camera does not move). A ward visible when the window opens was
# placed before it ("pre_existing", left out of the score).

MANA_MIN_W = 40.0
MANA_DY = (9.0, 15.0)
PIP_W = (12.0, 28.0)
PIP_MIN_SAT_BLUE = 230.0
PIP_DY = 2.0
PIP_GAP = 8.0
WARD_PIPS = (3, 4)
WARD_MERGE_PX = 10.0       # same ward if seen again this close (px at the reference) ...
WARD_MERGE_S = 30.0        # ... within this many seconds
WARD_MIN_FRAMES = 2
WARD_LAG_S = 0.15


def ward_pips(dets: list[dict], bars: list[dict], scale: float, why: dict) -> list[dict]:
    """Ward-labelled blobs that are pips: mana bars and other wide blobs out,
    pip width, and (when the saturation was recorded) blue pips fully saturated."""
    out = []
    for d in dets:
        if d.get("label") != "ward":
            continue
        w, left = float(d.get("w") or 0.0), bar_left(d)
        if w > MANA_MIN_W * scale:
            under = any(MANA_DY[0] * scale <= float(d["y"]) - b["y"] <= MANA_DY[1] * scale
                        and abs(left - b["left"]) <= PIP_GAP * scale for b in bars)
            why["mana" if under else "wide"] += 1
            continue
        if not PIP_W[0] * scale <= w <= PIP_W[1] * scale:
            why["pip_width"] += 1
            continue
        if d.get("team") == "blue" and d.get("sat") is not None and d["sat"] < PIP_MIN_SAT_BLUE:
            why["low_saturation"] += 1
            continue
        out.append({"team": d.get("team"), "left": left, "right": left + w, "y": float(d["y"])})
    return out


def group_pips(pips: list[dict], scale: float) -> list[dict]:
    groups: list[dict] = []
    for p in sorted(pips, key=lambda q: q["left"]):
        g = next((g for g in groups if g["team"] == p["team"] and abs(g["y"] - p["y"]) <= PIP_DY * scale
                  and p["left"] - g["right"] <= PIP_GAP * scale), None)
        if g:
            g["right"] = max(g["right"], p["right"])
            g["pips"] += 1
        else:
            groups.append({**p, "pips": 1})
    for g in groups:
        g["x"] = (g["left"] + g["right"]) / 2
    return groups


def track_wards(kept: list[dict], merged: dict, scale: float, mapping) -> tuple[list[dict], dict]:
    why = {"mana": 0, "wide": 0, "pip_width": 0, "low_saturation": 0}
    tracks: list[dict] = []
    n_groups = 0
    for f in kept:
        t = f["t"]
        groups = group_pips(ward_pips(f["dets"], merged[t], scale, why), scale)
        n_groups += len(groups)
        for g in groups:
            k = next((w for w in tracks if w["team"] == g["team"] and t - w["last_seen"] <= WARD_MERGE_S
                      and t > w["last_seen"]
                      and np.hypot(w["px"][0] - g["x"], w["px"][1] - g["y"]) <= WARD_MERGE_PX * scale), None)
            if k:
                k["last_seen"] = t
                k["frames"] += 1
                k["pip_counts"].append(g["pips"])
            else:
                tracks.append({"team": g["team"], "first_seen": t, "last_seen": t, "frames": 1,
                               "px": [round(g["x"], 1), round(g["y"], 1)], "pip_counts": [g["pips"]]})
    ws = kept[0]["t"] if kept else 0.0
    wards = []
    for w in tracks:
        pips = max(set(w["pip_counts"]), key=w["pip_counts"].count)
        if pips not in WARD_PIPS or w["frames"] < WARD_MIN_FRAMES:
            continue
        x, y = mapping.pixel_to_world(*w["px"]) if mapping else tuple(w["px"])
        wards.append({"t": round(w["first_seen"] - WARD_LAG_S, 2), "x": round(x, 1), "y": round(y, 1),
                      "team": w["team"], "pips": pips, "kind": "control" if pips == 4 else "sight",
                      "first_seen": w["first_seen"], "last_seen": w["last_seen"], "frames": w["frames"],
                      "px": w["px"], "pre_existing": w["first_seen"] <= ws})
    summary = {"excluded_blobs": why, "pip_groups": n_groups, "tracks": len(tracks), "valid": len(wards),
               "pre_existing": sum(w["pre_existing"] for w in wards),
               "rules": {"pips": list(WARD_PIPS), "min_frames": WARD_MIN_FRAMES,
                         "merge_px": WARD_MERGE_PX, "merge_s": WARD_MERGE_S, "lag_s": WARD_LAG_S},
               "position": "pip-bar centre through the homography, no offset (not measured)"}
    return wards, summary


def ward_window(kept: list[dict]) -> list[float] | None:
    """WARD_PLACED times a ward could be scored for: after the first kept frame
    (a ward seen then is pre-existing) and early enough to be seen
    WARD_MIN_FRAMES times."""
    if len(kept) < WARD_MIN_FRAMES:
        return None
    return [kept[0]["t"], kept[-WARD_MIN_FRAMES]["t"]]


def build_track(det_px: list[dict], champions: list[str], frame_size, minimap: dict | None = None,
                timeline: dict | None = None, pmap: dict[int, str] | None = None,
                anchors_H: np.ndarray | None = None, camera: dict | None = None,
                identity: str = "auto", log=None) -> dict:
    """Everything after detection, from per-frame bar detections (pixel dicts):
    stale first second dropped, bars merged, mapping fitted, positions and wards."""
    pmap = pmap or {i + 1: c for i, c in enumerate(champions)}
    det_px = sorted(det_px, key=lambda f: f["t"])
    t0 = det_px[0]["t"] if det_px else 0.0
    kept = [f for f in det_px if f["t"] >= t0 + STALE_S]
    stale = [f["t"] for f in det_px if f["t"] < t0 + STALE_S]
    scale = ui_scale(frame_size)
    merged = {f["t"]: merge_bars(f["dets"], scale) for f in kept}
    if identity == "auto":
        identity = "minimap" if minimap else "oracle"
    offset0 = default_bar_offset(frame_size, camera)
    truths = truth_frames(timeline, pmap)
    fit_frames = []
    for tf, pos in truths:
        near = min(kept, key=lambda f: abs(f["t"] - tf), default=None)
        if near is None or abs(near["t"] - tf) > FIT_FRAME_TOL_S:
            continue
        ref = pos if identity == "oracle" else (nearest_sample(minimap, near["t"]) or {}).get("pos")
        fit_frames.append({"t": near["t"], "timeline_t": tf, "bars": merged[near["t"]],
                           "truth": pos, "ident": ref})
    mapping, fit = None, None
    if anchors_H is not None:
        mapping = BarMap(np.asarray(anchors_H, float), offset0)
        fit = {"source": "anchors", "mode": "anchors", "offset_px": list(offset0),
               "offset_from": "default scaled to the frame and camera (no timeline to fit it)"}
    if fit_frames:
        start = mapping or initial_map(frame_size, camera, offset0)
        m, rep = fit_topdown(fit_frames, champions, start, anchors_H is not None, identity)
        if m is not None:
            mapping, fit = m, rep
        elif fit is None:
            fit = rep
    samples, counts = [], []
    fit_t = {f["t"]: f for f in fit_frames}
    for f in kept:
        t, bars = f["t"], merged[f["t"]]
        counts.append({"t": t, "blue": sum(b["team"] == "blue" for b in bars),
                       "red": sum(b["team"] == "red" for b in bars)})
        if mapping is None:
            samples.append({"t": t, "pos": {c: None for c in champions}})
            continue
        world = [(b["team"], *mapping.to_world(b["left"], b["y"])) for b in bars]
        ref = fit_t[t]["truth"] if identity == "oracle" and t in fit_t else \
            (nearest_sample(minimap, t) or {}).get("pos")
        samples.append({"t": t, "pos": assign_identities(t, world, ref, champions)})
    wards, ward_summary = track_wards(kept, merged, scale, mapping)
    track = {
        "source": "topdown", "processing": PROCESSING, "fps": 1.0, "champions": champions,
        "samples": samples, "gaps": find_gaps(samples, champions), "missing_classes": [],
        "calibration": mapping.as_dict() if mapping else None, "fit": fit,
        "frame_size": list(frame_size), "stale_dropped": stale, "bar_counts": counts,
        "wards": wards, "ward_units": "world" if mapping else "px", "ward_window": ward_window(kept),
        "ward_summary": ward_summary, "detections_px": det_px,
    }
    if log:
        log.info("top-down: %d frames (%d stale dropped), fit %s", len(kept), len(stale),
                 {k: (fit or {}).get(k) for k in ("mode", "identity", "median_error", "lofo_median",
                                                  "offset_px")})
    return track


def run_offline(frames_dir: Path, out_dir: Path, champions: list[str], anchors: Path | None,
                minimap: dict | None, max_frames: int = 6, extra: dict | None = None,
                log=None, timeline: dict | None = None, pmap: dict[int, str] | None = None,
                identity: str = "auto") -> dict:
    frames = list_frames(frames_dir)
    if not frames:
        raise SystemExit(f"no tNNNNN.png frames in {frames_dir}")
    detector = BarDetector()
    anchors_H = None
    if anchors and anchors.exists():
        anchors_H = calibrate.fit_homography(*calibrate.load_anchors(anchors)).H
    elif timeline is None:
        write_json(out_dir / "anchors_template.json", calibrate.anchors_template())
        if log:
            log.warning("no anchors.json and no timeline: detections kept in pixels; "
                        "fill anchors_template.json or pass --timeline")
    det_px = []
    first = cv2.imread(str(frames[0][1]))
    for t, path in frames:
        dets = detector.detect(cv2.imread(str(path)))
        det_px.append({"t": t, "dets": [d.as_dict() for d in dets]})
    camera = ((extra or {}).get("recording") or {}).get("camera")
    track = build_track(det_px, champions, (first.shape[1], first.shape[0]), minimap, timeline,
                        pmap, anchors_H, camera, identity, log)
    keep = out_dir / "frames_topdown"
    keep.mkdir(parents=True, exist_ok=True)
    for _, path in sample_evenly(frames, max_frames):   # re-encoded: the client's PNGs are large
        if path.parent.resolve() != keep.resolve():
            cv2.imwrite(str(keep / path.name), cv2.imread(str(path)), [cv2.IMWRITE_PNG_COMPRESSION, 9])
    track = {"matchId": (minimap or {}).get("matchId"), **track, "detector": detector.describe(),
             **(extra or {})}
    write_json(out_dir / "track_topdown.json", track)
    if log:
        n_bars = sum(c["blue"] + c["red"] for c in track["bar_counts"])
        log.info("track_topdown.json: %d frames, %d merged champion bars, %d wards", len(frames),
                 n_bars, len(track["wards"]))
    return track


def reprocess(track: dict, timeline: dict | None, pmap: dict[int, str] | None = None,
              minimap: dict | None = None, anchors_H=None, identity: str = "auto", log=None) -> dict:
    """Rebuild a track written before R2 (or by another version) from its
    stored detections_px: no frames needed."""
    camera = (track.get("recording") or {}).get("camera")
    new = build_track(track.get("detections_px") or [], track["champions"], track["frame_size"],
                      minimap, timeline, pmap, anchors_H, camera, identity, log)
    keep = {k: v for k, v in track.items() if k not in new}
    return {**keep, **new, "reprocessed_from": track.get("processing") or "r1"}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(prog="scout.track_topdown", description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", required=True, type=Path, help="run directory")
    ap.add_argument("--frames", type=Path, help="offline: directory of tNNNNN.png frames")
    ap.add_argument("--anchors", type=Path, help="anchors.json (default RUN_DIR/anchors.json)")
    ap.add_argument("--minimap-track", type=Path, help="track_minimap.json for identities")
    ap.add_argument("--champions", help="ten ids, blue then red (default: from the minimap track)")
    ap.add_argument("--keystroke-unlock", action="store_true",
                    help="ask the player to press Ctrl+Shift+Y instead of moving the API camera")
    ap.add_argument("--camera-y", type=float,
                    help="camera height (default: probe.json camera_fit, else scaled 50000)")
    ap.add_argument("--fov", type=float, default=19.0)
    ap.add_argument("--start", type=float, default=600.0, help="game seconds")
    ap.add_argument("--duration", type=float, default=300.0)
    ap.add_argument("--fps", type=float, default=2.0)
    ap.add_argument("--width", type=int, default=1920)
    ap.add_argument("--height", type=int, default=1080)
    ap.add_argument("--max-frames", type=int, default=6)
    ap.add_argument("--scratch", type=Path,
                    default=Path(os.environ.get("TEMP", tempfile.gettempdir())) / "scout-rec")
    ap.add_argument("--base", default=DEFAULT_BASE, help="Replay API base URL")
    ap.add_argument("--timeline", type=Path,
                    help="timeline.json to fit the bar offset and homography (default RUN_DIR/timeline.json)")
    ap.add_argument("--match", type=Path, help="match.json for participantId -> champion "
                                               "(default RUN_DIR/match.json, else blue 1-5, red 6-10)")
    ap.add_argument("--identity", choices=["auto", "minimap", "oracle"], default="auto",
                    help="who is who in the fit: the minimap track (default when present) or the "
                         "timeline positions (oracle fit, reported as such)")
    ap.add_argument("--from-detections", type=Path,
                    help="rebuild an existing track_topdown.json from its detections_px (no frames)")
    args = ap.parse_args(argv)
    log = run_logger(args.out, "track_topdown")

    mm_path = args.minimap_track or args.out / "track_minimap.json"
    minimap = read_json(mm_path) if mm_path.exists() else None
    if args.champions:
        champions = [c.strip() for c in args.champions.split(",")]
    elif minimap:
        champions = minimap["champions"]
    elif not args.from_detections:
        raise SystemExit("need --champions or a track_minimap.json")
    tl_path = args.timeline or args.out / "timeline.json"
    timeline = read_json(tl_path) if tl_path.exists() else None
    m_path = args.match or args.out / "match.json"
    pmap = None
    if m_path.exists():
        pmap = {p["participantId"]: p["championName"] for p in read_json(m_path)["info"]["participants"]}
    anchors = args.anchors or args.out / "anchors.json"
    if args.from_detections:
        old = read_json(args.from_detections)
        H = calibrate.fit_homography(*calibrate.load_anchors(anchors)).H if anchors.exists() else None
        track = reprocess(old, timeline, pmap, minimap, H, args.identity, log)
        write_json(args.out / "track_topdown.json", track)
        return 0
    frames_dir, extra = args.frames, {}
    if frames_dir is not None and args.camera_y is not None:
        extra = {"recording": {"camera": {"mode": "given", "height": args.camera_y, "fov": args.fov}}}
    if frames_dir is None:
        client = ReplayClient(args.base)
        with preserved_render(client, log) as restore:   # the client is left as found
            frames_dir, extra = record_live(client, args, log)
        extra = {"recording": extra, "render_restore": restore}
    run_offline(frames_dir, args.out, champions, anchors, minimap, args.max_frames, extra, log,
                timeline, pmap, args.identity)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
