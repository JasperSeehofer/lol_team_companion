"""Route B: the whole map in one top-down view, champions and wards found by
their health bars.

Live (player's PC; run in a fresh desktop session if a recording already ran):
  python -m scout.track_topdown --out RUN_DIR [--keystroke-unlock] [--start 600]
Offline (saved frames tNNNNN.png):
  python -m scout.track_topdown --out RUN_DIR --frames DIR --anchors anchors.json
      --minimap-track RUN_DIR/track_minimap.json

Live mode records 5 min through /replay/recording (png codec) into a scratch
directory outside the repo (default %TEMP%/scout-rec), keeps one frame per game
second for tracking and copies up to --max-frames of them into
RUN_DIR/frames_topdown/. Without anchors.json it writes a template to fill in
and keeps the detections in pixels (half C fits the homography from the frames).
"""

from __future__ import annotations

import argparse
import os
import shutil
import tempfile
import time
from pathlib import Path

import cv2
import numpy as np
from scipy.optimize import linear_sum_assignment

from . import calibrate
from .camera import default_camera_y
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
WARD_MERGE = 250.0       # same ward if seen again this close ...
WARD_MERGE_S = 5.0       # ... within this many seconds
WARD_MERGE_PX = 12.0     # the same in screen pixels when no homography is fitted yet


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


def assign_identities(t: float, bars: list[tuple[str, float, float]], minimap: dict | None,
                      champions: list[str]) -> dict:
    """Hungarian per team between world-space bars and the minimap track at t."""
    pos = {c: None for c in champions}
    ref = nearest_sample(minimap, t)
    if ref is None:
        return pos
    for team, members in (("blue", champions[:5]), ("red", champions[5:])):
        cands = [m for m in members if ref["pos"].get(m)]
        team_bars = [(x, y) for tm, x, y in bars if tm == team]
        if not cands or not team_bars:
            continue
        cost = np.array([[np.hypot(ref["pos"][c][0] - x, ref["pos"][c][1] - y)
                          for x, y in team_bars] for c in cands])
        for r, k in zip(*linear_sum_assignment(cost)):
            if cost[r, k] <= IDENTITY_GATE:
                x, y = team_bars[k]
                pos[cands[r]] = [round(x, 1), round(y, 1)]
    return pos


def run_offline(frames_dir: Path, out_dir: Path, champions: list[str], anchors: Path | None,
                minimap: dict | None, max_frames: int = 6, extra: dict | None = None,
                log=None) -> dict:
    frames = list_frames(frames_dir)
    if not frames:
        raise SystemExit(f"no tNNNNN.png frames in {frames_dir}")
    detector = BarDetector()
    mapping = None
    if anchors and anchors.exists():
        mapping = calibrate.fit_homography(*calibrate.load_anchors(anchors))
    else:
        write_json(out_dir / "anchors_template.json", calibrate.anchors_template())
        if log:
            log.warning("no anchors.json: detections kept in pixels; fill anchors_template.json")
    samples, det_px, wards = [], [], []
    first = cv2.imread(str(frames[0][1]))
    for t, path in frames:
        dets = detector.detect(cv2.imread(str(path)))
        det_px.append({"t": t, "dets": [d.as_dict() for d in dets]})
        if mapping is None:   # no positions, but wards still count for time matching
            samples.append({"t": t, "pos": {c: None for c in champions}})
        else:
            bars = [(d.team, *mapping.to_world(d.x, d.y)) for d in dets if d.label == "champion"]
            samples.append({"t": t, "pos": assign_identities(t, bars, minimap, champions)})
        for d in dets:
            if d.label != "ward":
                continue
            x, y = mapping.to_world(d.x, d.y) if mapping else (d.x, d.y)
            radius = WARD_MERGE if mapping else WARD_MERGE_PX
            known = next((w for w in wards if w["team"] == d.team
                          and t - w["last_seen"] <= WARD_MERGE_S
                          and np.hypot(w["x"] - x, w["y"] - y) <= radius), None)
            if known:
                known["last_seen"] = t
            else:
                wards.append({"t": t, "x": round(x, 1), "y": round(y, 1), "team": d.team,
                              "last_seen": t})
    keep = out_dir / "frames_topdown"
    keep.mkdir(parents=True, exist_ok=True)
    for _, path in sample_evenly(frames, max_frames):   # re-encoded: the client's PNGs are large
        if path.parent.resolve() != keep.resolve():
            cv2.imwrite(str(keep / path.name), cv2.imread(str(path)), [cv2.IMWRITE_PNG_COMPRESSION, 9])
    track = {
        "matchId": (minimap or {}).get("matchId"), "source": "topdown", "fps": 1.0,
        "champions": champions, "samples": samples, "gaps": find_gaps(samples, champions),
        "detector": detector.describe(), "missing_classes": [],
        "calibration": mapping.as_dict() if mapping else None,
        "frame_size": [first.shape[1], first.shape[0]], "wards": wards,
        "ward_units": "world" if mapping else "px",
        "detections_px": det_px, **(extra or {}),
    }
    write_json(out_dir / "track_topdown.json", track)
    if log:
        n_bars = sum(1 for f in det_px for d in f["dets"] if d["label"] == "champion")
        log.info("track_topdown.json: %d frames, %d champion bars, %d wards", len(frames),
                 n_bars, len(wards))
    return track


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
    args = ap.parse_args(argv)
    log = run_logger(args.out, "track_topdown")

    mm_path = args.minimap_track or args.out / "track_minimap.json"
    minimap = read_json(mm_path) if mm_path.exists() else None
    if args.champions:
        champions = [c.strip() for c in args.champions.split(",")]
    elif minimap:
        champions = minimap["champions"]
    else:
        raise SystemExit("need --champions or a track_minimap.json")
    frames_dir, extra = args.frames, {}
    if frames_dir is None:
        client = ReplayClient(args.base)
        with preserved_render(client, log) as restore:   # the client is left as found
            frames_dir, extra = record_live(client, args, log)
        extra = {"recording": extra, "render_restore": restore}
    run_offline(frames_dir, args.out, champions, args.anchors or args.out / "anchors.json",
                minimap, args.max_frames, extra, log)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
