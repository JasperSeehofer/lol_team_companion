"""Small shared helpers: run log, JSON files, time formatting."""

from __future__ import annotations

import json
import logging
import re
import sys
from pathlib import Path

FRAME_NAME = re.compile(r"t(\d+)\.png$")


def run_logger(run_dir: Path | str, name: str) -> logging.Logger:
    """Logger writing to RUN_DIR/run.log (appended) and to stderr."""
    run_dir = Path(run_dir)
    run_dir.mkdir(parents=True, exist_ok=True)
    log = logging.getLogger(f"scout.{name}")
    log.setLevel(logging.INFO)
    log.handlers.clear()
    fmt = logging.Formatter("%(asctime)s %(name)s %(levelname)s %(message)s")
    for handler in (logging.FileHandler(run_dir / "run.log", encoding="utf-8"),
                    logging.StreamHandler(sys.stderr)):
        handler.setFormatter(fmt)
        log.addHandler(handler)
    log.propagate = False
    return log


def write_json(path: Path | str, data) -> Path:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=1, sort_keys=False) + "\n", encoding="utf-8")
    return path


def read_json(path: Path | str):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def mmss(seconds: float) -> str:
    seconds = max(0, int(round(seconds)))
    return f"{seconds // 60:02d}:{seconds % 60:02d}"


def frame_name(t: float) -> str:
    """File name of a frame sampled at game time t (whole seconds)."""
    return f"t{int(round(t)):05d}.png"


def list_frames(frames_dir: Path | str) -> list[tuple[float, Path]]:
    """(game time, path) for every tNNNNN.png in a directory, sorted by time."""
    out = []
    for p in Path(frames_dir).iterdir():
        m = FRAME_NAME.search(p.name)
        if m:
            out.append((float(m.group(1)), p))
    return sorted(out)


def sample_evenly(items: list, k: int) -> list:
    """At most k items spread evenly over the list (first and last kept)."""
    if len(items) <= k:
        return list(items)
    if k <= 1:
        return items[:k]
    step = (len(items) - 1) / (k - 1)
    return [items[round(i * step)] for i in range(k)]


def parse_box(text: str) -> dict:
    """'left,top,width,height' -> mss monitor dict."""
    left, top, width, height = (int(v) for v in text.split(","))
    return {"left": left, "top": top, "width": width, "height": height}
