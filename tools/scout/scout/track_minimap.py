"""Route A: champion positions from the minimap at 1 Hz game time.

Live (on the player's PC, replay open):
  python -m scout.track_minimap --out RUN_DIR [--speed 8] [--minimap-box L,T,W,H]
Offline (saved frames tNNNNN.png, used by the tests and by half C):
  python -m scout.track_minimap --out RUN_DIR --frames DIR --champions A,B,...

Writes RUN_DIR/track_minimap.json (world units) and up to --max-frames sampled
minimap PNGs into RUN_DIR/frames_minimap/. All captured frames stay in the
scratch directory (default %TEMP%/scout-frames) outside the repository.
"""

from __future__ import annotations

import argparse
import math
import os
import shutil
import tempfile
import time
from pathlib import Path

import cv2
import numpy as np

from . import calibrate
from .detect import TemplateDetector, YoloDetector, fetch_ddragon_icons, load_icons
from .render import minimap_only_render, preserved_render
from .replay_api import DEFAULT_BASE, ReplayClient
from .screen import all_monitors, describe, pick_monitor
from .track import StaticSpots, Tracker, WorldDet, find_gaps
from .util import frame_name, list_frames, parse_box, read_json, run_logger, sample_evenly, write_json

ICON_FRACTION = 0.08   # icon diameter / minimap width (pyLoL: ~40 px on a 512 px minimap)
MODELS_DIR = Path(__file__).resolve().parent.parent / "models"


def champions_from_match(match: dict) -> list[str]:
    """Ten championName values, blue team (100) first, in participantId order."""
    parts = sorted(match["info"]["participants"], key=lambda p: (p.get("teamId", 0), p["participantId"]))
    return [p["championName"] for p in parts]


def champions_from_client(client: ReplayClient) -> list[str]:
    """Live Client Data player list (the same port serves it during replays;
    verify on run). rawChampionName ends with the Data Dragon id."""
    players = client.get("/liveclientdata/playerlist")
    order = sorted(players, key=lambda p: 0 if p.get("team") == "ORDER" else 1)
    return [p.get("rawChampionName", "").rsplit("_", 1)[-1] or p["championName"] for p in order]


def auto_minimap_box(screen_w: int, screen_h: int) -> dict:
    """Guess: square bottom-right, side ~24 % of the screen height (default HUD
    and minimap scale). Check the first saved frame; override with --minimap-box."""
    side = round(screen_h * 0.237)
    margin = round(screen_h * 0.01)
    return {"left": screen_w - side - margin, "top": screen_h - side - margin,
            "width": side, "height": side}


def capture_live(client: ReplayClient, box: dict | None, scratch: Path, speed: float,
                 start: float, end: float | None, log, monitor: int | None = None) -> tuple[Path, dict]:
    import mss  # Windows capture; imported only in live mode

    client.set_render(**minimap_only_render(), fogOfWar=False)
    pb = client.playback()
    end = end or float(pb.get("length", 0)) or 3600.0
    client.set_playback(time=start, paused=False, speed=speed)
    time.sleep(1.0)
    scratch.mkdir(parents=True, exist_ok=True)
    n = 0
    wall0 = time.time()
    with mss.mss() as sct:
        index, mon, how = pick_monitor(sct.monitors, monitor)
        chosen = describe(index, mon, how)
        log.info("monitor %d (%s): %dx%d at %d,%d; mss lists %s", index, how, mon["width"],
                 mon["height"], mon["left"], mon["top"], all_monitors(sct.monitors))
        if box is None:   # relative to the chosen monitor's origin
            box = auto_minimap_box(mon["width"], mon["height"])
            box["left"] += mon["left"]
            box["top"] += mon["top"]
        log.info("minimap box %s, speed %s, %s..%s s", box, speed, start, end)
        next_t = math.floor(start)
        while True:
            t = float(client.playback()["time"])
            if t >= end:
                break
            if t >= next_t:
                img = np.asarray(sct.grab(box))[:, :, :3]
                cv2.imwrite(str(scratch / frame_name(math.floor(t))), img,
                            [cv2.IMWRITE_PNG_COMPRESSION, 1])
                n += 1
                next_t = math.floor(t) + 1
            else:
                time.sleep(0.01)
    client.set_playback(paused=True)
    wall = time.time() - wall0
    log.info("captured %d frames in %.0f s wall (%.2f game-s per wall-s)", n, wall,
             (end - start) / wall if wall else 0)
    return scratch, {"box": box, "monitor": chosen, "frames": n, "wall_s": round(wall, 1),
                     "speed": speed}


def run_offline(frames_dir: Path, champions: list[str], detector, out_dir: Path,
                anchors: Path | None = None, max_frames: int = 24, match_id: str | None = None,
                log=None) -> dict:
    frames = list_frames(frames_dir)
    if not frames:
        raise SystemExit(f"no tNNNNN.png frames in {frames_dir}")
    first = cv2.imread(str(frames[0][1]))
    if anchors:
        mapping = calibrate.fit_linear(*calibrate.load_anchors(anchors))
    else:
        mapping = calibrate.bounds_map(first.shape[1], first.shape[0])
    # Two passes (finding F): detect every frame and count each label's
    # recurring spots, then track with the static spots rejected. The live
    # path captures to the scratch directory first, so it runs this too.
    spots = StaticSpots()
    per_frame = []
    for t, path in frames:
        dets = detector.detect(cv2.imread(str(path)))
        spots.observe(dets)
        per_frame.append((t, dets))
    static = spots.static()
    tracker = Tracker(champions)
    for t, dets in per_frame:
        tracker.step(t, [WorldDet(d.label, *mapping.to_world(d.x, d.y), d.score)
                         for d in spots.filter(dets, static)])
    keep = out_dir / "frames_minimap"
    keep.mkdir(parents=True, exist_ok=True)
    for _, path in sample_evenly(frames, max_frames):
        if path.parent.resolve() != keep.resolve():
            shutil.copy2(path, keep / path.name)
    track = {
        "matchId": match_id, "source": "minimap", "fps": 1.0, "champions": champions,
        "samples": tracker.samples, "gaps": find_gaps(tracker.samples, champions),
        "detector": detector.describe(), "missing_classes": detector.missing_classes,
        "calibration": mapping.as_dict(), "frame_size": [first.shape[1], first.shape[0]],
        "wards": [], "static_spots": spots.describe(mapping.to_world),
    }
    write_json(out_dir / "track_minimap.json", track)
    if log:
        seen = sum(1 for s in tracker.samples for p in s["pos"].values() if p)
        log.info("track_minimap.json: %d samples, %d positions, %d gaps", len(tracker.samples),
                 seen, len(track["gaps"]))
        for sp in track["static_spots"]["spots"]:
            log.info("static spot rejected: %s at px %.0f,%.0f (%d of %d frames)", sp["champion"],
                     sp["px"], sp["py"], sp["frames"], spots.frames)
    return track


def build_detector(args, champions: list[str], icon_px: int, log):
    if args.weights:
        det = YoloDetector(args.weights, champions)
        if det.missing_classes:
            log.warning("weights lack classes for: %s", ", ".join(det.missing_classes))
        return det
    icon_dir = args.icons
    if icon_dir is None:
        icon_dir = MODELS_DIR / "icons"
        version = fetch_ddragon_icons(champions, icon_dir)
        log.info("Data Dragon icons %s in %s", version, icon_dir)
    return TemplateDetector(load_icons(icon_dir, champions), icon_px=icon_px,
                            threshold=args.threshold)


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(prog="scout.track_minimap", description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", required=True, type=Path, help="run directory")
    ap.add_argument("--frames", type=Path, help="offline: directory of tNNNNN.png minimap crops")
    ap.add_argument("--champions", help="ten champion ids, blue 1-5 then red 6-10, comma-separated")
    ap.add_argument("--match", type=Path, help="match.json (from fetch_timeline) for champion names")
    ap.add_argument("--icons", type=Path, help="directory of {champion}.png icons (default: Data Dragon)")
    ap.add_argument("--weights", type=Path, help="pyLoL yolov12.pt; without it the template detector runs")
    ap.add_argument("--threshold", type=float, default=0.55, help="template score threshold")
    ap.add_argument("--icon-px", type=int, help="icon diameter on the minimap in px")
    ap.add_argument("--anchors", type=Path, help="anchors.json of minimap pixel <-> world pairs")
    ap.add_argument("--minimap-box", help="screen region left,top,width,height")
    ap.add_argument("--monitor", type=int,
                    help="mss monitor index to capture (default: the primary monitor)")
    ap.add_argument("--speed", type=float, default=8.0)
    ap.add_argument("--start", type=float, default=0.0)
    ap.add_argument("--end", type=float, help="game seconds (default: replay length)")
    ap.add_argument("--max-frames", type=int, default=24)
    ap.add_argument("--scratch", type=Path,
                    default=Path(os.environ.get("TEMP", tempfile.gettempdir())) / "scout-frames")
    ap.add_argument("--base", default=DEFAULT_BASE, help="Replay API base URL")
    args = ap.parse_args(argv)
    log = run_logger(args.out, "track_minimap")

    client = None if args.frames else ReplayClient(args.base)
    match_id = None
    if args.champions:
        champions = [c.strip() for c in args.champions.split(",")]
    elif args.match:
        match = read_json(args.match)
        champions, match_id = champions_from_match(match), match["metadata"]["matchId"]
    elif client:
        try:
            champions = champions_from_client(client)
        except Exception as e:  # noqa: BLE001
            raise SystemExit(f"could not read the champions from the client ({e}); "
                             "run fetch_timeline first and pass --match RUN/match.json") from e
    else:
        raise SystemExit("offline mode needs --champions or --match")
    if len(champions) != 10:
        log.warning("expected 10 champions, got %d: %s", len(champions), champions)
    log.info("champions: %s", ", ".join(champions))

    frames_dir, capture = args.frames, None
    if client:
        box = parse_box(args.minimap_box) if args.minimap_box else None
        with preserved_render(client, log) as restore:   # the client is left as found
            frames_dir, capture = capture_live(client, box, args.scratch, args.speed, args.start,
                                               args.end, log, args.monitor)
        capture["render_restore"] = restore
    first = cv2.imread(str(list_frames(frames_dir)[0][1]))
    icon_px = args.icon_px or max(10, round(first.shape[1] * ICON_FRACTION))
    detector = build_detector(args, champions, icon_px, log)
    track = run_offline(frames_dir, champions, detector, args.out, args.anchors, args.max_frames,
                        match_id, log)
    if capture:
        track["capture"] = capture
        write_json(args.out / "track_minimap.json", track)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
