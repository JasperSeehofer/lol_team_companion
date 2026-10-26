"""Live-client facts: which Replay API values the client actually honours.

  python -m scout.probe --out RUN_DIR [--dry] [--skip-recording]

Writes RUN_DIR/openapi.json (when a spec path answers), RUN_DIR/probe.json and
a few screenshots (probe_*.png, downscaled) as evidence. Every sub-probe is
isolated: a failure is written into probe.json and the next one runs. The
render and playback state found at the start are restored at the end.
"""

from __future__ import annotations

import argparse
import os
import tempfile
import time
from pathlib import Path

from . import camera as cam
from .render import LEGACY_MINIMAP_ONLY, LEGACY_NOTE, minimap_only_render, preserved_render
from .replay_api import DEFAULT_BASE, ReplayClient
from . import winhdr
from .screen import all_monitors, describe, pick_monitor
from .track_topdown import camera_render
from .util import run_logger, write_json

SPEC_PATHS = ["/swagger/v3/openapi.json", "/swagger/v2/swagger.json",
              "/swagger/v1/api-docs", "/openapi.json"]
SPEEDS = [1, 2, 4, 8, 10, 16]
CAMERA_HEIGHTS = [2000, 5000, 10000, 20000, 40000, 60000, 80000]
HEALTH_BARS = ["healthBarChampions", "healthBarStructures", "healthBarWards",
               "healthBarPets", "healthBarMinions"]
SEEK_TO = 120.0  # game seconds to start the speed probe from


def screen_grabber(max_width: int = 1280, monitor: int | None = None, log=None):
    """Returns grab(path) saving a PNG of the primary monitor (or --monitor N;
    see scout.screen), downscaled to max_width, or None when mss is unavailable
    (tests, non-Windows). grab.monitor describes the monitor captured, grab.hdr
    its Windows HDR state (scout.winhdr; None off Windows)."""
    try:
        import cv2
        import mss
        import numpy as np
    except ImportError:
        return None

    with mss.mss() as sct:
        index, mon, how = pick_monitor(sct.monitors, monitor)
        listed = all_monitors(sct.monitors)
    box = {k: mon[k] for k in ("left", "top", "width", "height")}
    if log:
        log.info("capturing monitor %d (%s): %dx%d at %d,%d; mss lists %s", index, how,
                 box["width"], box["height"], box["left"], box["top"], listed)

    def grab(path: Path, full: bool = False) -> None:
        with mss.mss() as sct:
            img = np.asarray(sct.grab(box))[:, :, :3]
        if not full and img.shape[1] > max_width:
            scale = max_width / img.shape[1]
            img = cv2.resize(img, (max_width, round(img.shape[0] * scale)),
                             interpolation=cv2.INTER_AREA)
        cv2.imwrite(str(path), img, [cv2.IMWRITE_PNG_COMPRESSION, 9])

    grab.monitor = describe(index, mon, how)
    grab.hdr = winhdr.check(box, log)   # finding I: warn when Windows HDR is on
    return grab


def plan() -> list[str]:
    steps = [f"GET {p}" for p in SPEC_PATHS] + ["GET /replay/game", "GET /replay/render",
                                                 "GET /replay/playback"]
    steps.append("POST /replay/render {interfaceAll:false, interfaceMinimap:true} + screenshot"
                 " (known to hide the minimap)")
    steps.append("POST /replay/render {interfaceAll:true, interfaceMinimap:true, other interface*:false}"
                 " + screenshot")
    steps += [f"POST /replay/playback {{speed:{s}}} then time over a window" for s in SPEEDS]
    steps += ["POST /replay/recording png realtime (enforceFrameRate:true, replaySpeed:1)",
              "POST /replay/recording png accelerated (enforceFrameRate:false, replaySpeed:4)"]
    steps += [f"POST /replay/render cameraPosition.y={h} farClip={h * 1.5 + 10000:.0f} + screenshot"
              for h in CAMERA_HEIGHTS]
    d = cam.default_camera_y(*cam.REF_SIZE)
    steps.append("POST /replay/render {interfaceAll:false}, then camera fit check: cameraPosition.y="
                 + ", ".join(f"{d * f:.0f}" for f in cam.FIT_FACTORS)
                 + " (default for the captured monitor; here 2560x1440) + screenshot each")
    steps += [f"POST /replay/render {{{b}:false}} then {{{b}:true}}" for b in HEALTH_BARS]
    steps += ["POST /replay/render {fogOfWar:false}", "restore render and playback"]
    return steps


class Probe:
    def __init__(self, client: ReplayClient, out: Path, grab=None, settle: float = 1.0,
                 speed_window: float = 3.0, rec_seconds: float = 10.0, rec_fps: int = 4,
                 rec_timeout: float = 90.0, scratch: Path | None = None, log=None):
        self.c, self.out, self.grab, self.settle = client, out, grab, settle
        self.speed_window, self.rec_seconds, self.rec_fps = speed_window, rec_seconds, rec_fps
        self.rec_timeout = rec_timeout
        self.scratch = scratch or Path(os.environ.get("TEMP", tempfile.gettempdir())) / "scout-rec"
        self.log = log
        self.result: dict = {"errors": {}}
        if getattr(grab, "monitor", None):
            self.result["monitor"] = grab.monitor
        if grab is not None and hasattr(grab, "hdr"):
            self.result["hdr"] = grab.hdr

    def _info(self, msg, *a):
        if self.log:
            self.log.info(msg, *a)

    def _shot(self, name: str, full: bool = False) -> dict | None:
        """Screenshot as evidence; mean brightness flags a black capture
        (e.g. if Vanguard or the GPU blocks screen capture)."""
        if self.grab is None:
            return None
        path = self.out / name
        self.grab(path, full)
        import cv2

        img = cv2.imread(str(path))
        mean = None if img is None else round(float(img.mean()), 1)
        return {"file": path.name, "mean_brightness": mean, "black": mean is not None and mean < 3}

    def run(self, skip_recording: bool = False) -> dict:
        subs = [self.spec, self.game, self.interface, self.speed]
        if not skip_recording:
            subs.append(self.recording)
        subs += [self.camera, self.camera_fit, self.health_bars, self.fog]
        playback = None
        try:
            playback = self.c.playback()
        except Exception as e:  # noqa: BLE001 (recorded, not fatal)
            self.result["errors"]["initial_read"] = repr(e)
        with preserved_render(self.c, self.log) as restore:
            try:
                for sub in subs:
                    try:
                        self._info("probe %s", sub.__name__)
                        self.result[sub.__name__] = sub()
                    except Exception as e:  # noqa: BLE001
                        self.result["errors"][sub.__name__] = repr(e)
                        self._info("probe %s failed: %r", sub.__name__, e)
            finally:
                try:
                    if playback:
                        self.c.set_playback(time=playback.get("time", 0), paused=True,
                                            speed=playback.get("speed", 1.0))
                except Exception as e:  # noqa: BLE001
                    self.result["errors"]["restore_playback"] = repr(e)
        self.result["render_restore"] = restore
        if restore["error"]:
            self.result["errors"]["restore"] = restore["error"]
        return self.result

    def _set_read(self, **fields) -> dict:
        """POST render fields, read back, report sent/read/honoured per field."""
        self.c.set_render(**fields)
        time.sleep(self.settle)
        back = self.c.render()
        return {k: {"sent": v, "read": back.get(k), "honoured": back.get(k) == v}
                for k, v in fields.items()}

    def spec(self) -> dict:
        tried = {}
        for path in SPEC_PATHS:
            tried[path] = self.c.status(path)
            if tried[path] == 200:
                spec = self.c.get(path)
                write_json(self.out / "openapi.json", spec)
                return {"answered": path, "tried": tried,
                        "paths": sorted((spec.get("paths") or {}).keys())}
        return {"answered": None, "tried": tried}

    def game(self) -> dict:
        return self.c.game()

    def interface(self) -> dict:
        """Both combinations: R1's (interfaceAll=false, known to hide the
        minimap) under "legacy", then the working one at the top level."""
        legacy = self._set_read(**LEGACY_MINIMAP_ONLY)
        legacy["screenshot"] = self._shot("probe_minimap_legacy.png", full=True)
        legacy["note"] = LEGACY_NOTE
        res = self._set_read(**minimap_only_render())
        res["screenshot"] = self._shot("probe_minimap_only.png", full=True)
        res["combo"] = "interfaceAll=true, interfaceMinimap=true, every other interface* false"
        res["legacy"] = legacy
        return res

    def speed(self) -> dict:
        rows = []
        for s in SPEEDS:
            self.c.set_playback(time=SEEK_TO, paused=False, speed=float(s))
            time.sleep(self.settle)
            t0, w0 = float(self.c.playback()["time"]), time.time()
            time.sleep(self.speed_window)
            pb = self.c.playback()
            t1, w1 = float(pb["time"]), time.time()
            rows.append({"sent": s, "read": pb.get("speed"),
                         "game_s_per_wall_s": round((t1 - t0) / (w1 - w0), 2)})
        self.c.set_playback(paused=True)
        honoured = [r["sent"] for r in rows if abs(r["game_s_per_wall_s"] - r["sent"]) <= 0.15 * r["sent"]]
        return {"steps": rows, "max_honoured": max(honoured) if honoured else None,
                "window_s": self.speed_window}

    def _record(self, name: str, enforce: bool, replay_speed: float) -> dict:
        path = self.scratch / f"probe-{name}-{int(time.time())}"
        path.mkdir(parents=True, exist_ok=True)
        start = SEEK_TO
        self.c.set_playback(time=start, paused=True, speed=1.0)
        time.sleep(self.settle)
        sent = {"recording": True, "codec": "png", "path": str(path), "startTime": start,
                "endTime": start + self.rec_seconds, "framesPerSecond": self.rec_fps,
                "enforceFrameRate": enforce, "replaySpeed": replay_speed, "lossless": True,
                "width": 1920, "height": 1080}
        self.c.set_recording(**sent)
        read = self.c.recording()
        self.c.set_playback(paused=False)
        w0 = time.time()
        while time.time() - w0 < self.rec_timeout:
            time.sleep(min(1.0, self.rec_seconds / 4))
            if not self.c.recording().get("recording"):
                break
        wall = time.time() - w0
        pngs = sorted(path.glob("*.png"))
        size = None
        if pngs:
            import cv2

            img = cv2.imread(str(pngs[0]))
            size = None if img is None else [img.shape[1], img.shape[0]]
        expected = int(self.rec_seconds * self.rec_fps)
        return {"read_back": {k: read.get(k) for k in sent}, "wall_s": round(wall, 2),
                "game_s": self.rec_seconds,
                "game_s_per_wall_s": round(self.rec_seconds / wall, 2) if pngs else None,
                "frames_written": len(pngs), "frames_expected": expected, "frame_size": size,
                "scratch": str(path)}

    def recording(self) -> dict:
        # Note: one recording per desktop session is reported; the second may write nothing.
        return {"realtime": self._record("realtime", True, 1.0),
                "accelerated": self._record("accelerated", False, 4.0)}

    def camera(self) -> dict:
        steps = []
        for h in CAMERA_HEIGHTS:
            fields = camera_render(h, 19.0)
            self.c.set_render(**fields)
            time.sleep(self.settle)
            back = self.c.render()
            y = (back.get("cameraPosition") or {}).get("y")
            steps.append({"sent_y": h, "read_y": y, "read_farClip": back.get("farClip"),
                          "honoured": y is not None and abs(float(y) - h) < 1.0,
                          "screenshot": self._shot(f"probe_camera_{h}.png")})
        ok = [s["sent_y"] for s in steps if s["honoured"]]
        return {"steps": steps, "max_honoured_y": max(ok) if ok else None,
                "note": "API read-back only; whether the whole map is in view: camera_fit"}

    def camera_fit(self) -> dict:
        """Visual fit check around the default height for this screen (see
        scout.camera): HUD off, a screenshot per height, map_fit on each."""
        mon = getattr(self.grab, "monitor", None) or {}
        size = (mon["width"], mon["height"]) if mon else cam.REF_SIZE
        fov = cam.REF_FOV
        default = cam.default_camera_y(*size, fov)
        self.c.set_render(interfaceAll=False)
        steps = []
        for f in cam.FIT_FACTORS:
            y = float(round(default * f))
            self.c.set_render(**camera_render(y, fov))
            time.sleep(self.settle)
            back = self.c.render()
            shot = self._shot(f"probe_camera_fit_{y:.0f}.png")
            fit = None
            if shot and not shot["black"]:
                import cv2

                img = cv2.imread(str(self.out / shot["file"]))
                fit = None if img is None else cam.map_fit(img)
            steps.append({"y": y, "read_y": (back.get("cameraPosition") or {}).get("y"),
                          "fit": fit, "screenshot": shot})
        chosen, how = cam.choose_height(steps, default)
        return {"screen": list(size), "screen_from": "monitor" if mon else "reference (no capture)",
                "fov": fov, "default_y": default, "rule": cam.RULE, "steps": steps,
                "chosen_y": chosen, "chosen_by": how}

    def health_bars(self) -> dict:
        out = {}
        for b in HEALTH_BARS:
            off = self._set_read(**{b: False})[b]
            on = self._set_read(**{b: True})[b]
            out[b] = {"off": off, "on": on, "honoured": off["honoured"] and on["honoured"]}
        return out

    def fog(self) -> dict:
        return self._set_read(fogOfWar=False)


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(prog="scout.probe", description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", required=True, type=Path, help="run directory")
    ap.add_argument("--base", default=DEFAULT_BASE, help="Replay API base URL")
    ap.add_argument("--dry", action="store_true", help="list what would be posted, post nothing")
    ap.add_argument("--skip-recording", action="store_true")
    ap.add_argument("--speed-window", type=float, default=3.0, help="wall seconds per speed step")
    ap.add_argument("--rec-seconds", type=float, default=10.0, help="game seconds per test recording")
    ap.add_argument("--monitor", type=int,
                    help="mss monitor index to capture (default: the primary monitor)")
    args = ap.parse_args(argv)
    if args.dry:
        print("\n".join(plan()))
        return 0
    log = run_logger(args.out, "probe")
    probe = Probe(ReplayClient(args.base), args.out, grab=screen_grabber(monitor=args.monitor, log=log),
                  speed_window=args.speed_window, rec_seconds=args.rec_seconds, log=log)
    result = probe.run(skip_recording=args.skip_recording)
    result["base"] = args.base
    write_json(args.out / "probe.json", result)
    log.info("probe.json written; %d sub-probes failed: %s", len(result["errors"]),
             ", ".join(result["errors"]) or "none")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
