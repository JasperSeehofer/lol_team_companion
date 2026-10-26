"""Top-down camera height: a default from the screen shape and a visual fit
check on screenshots (the API read-back only says what was stored, not what
is in view).

Reference (Jasper's run, 2026-10-22): camera y = 50000 at fieldOfView 19
shows the whole map at 2560x1440.

Scaling rule (``default_camera_y``). The map is square, so only the aspect
ratio a = width / height matters, not the pixel count; the visible extent at
height y is proportional to y * tan(fov / 2). We do not know whether the
client's fieldOfView is vertical or horizontal, so the default takes the
larger height of the two hypotheses, each calibrated to the reference:
  vertical fov:   y = Y0 * k / min(1, a)        (portrait screens need more)
  horizontal fov: y = Y0 * k * max(1, a) / A0   (screens wider than A0 need more)
with k = tan(fov0 / 2) / tan(fov / 2), Y0 = 50000, fov0 = 19, A0 = 16 / 9.
Together: y = Y0 * k * max(1, 1 / a, a / A0). Every 16:9 screen gets 50000 at
fov 19; 21:9 gets 65625; a 9:16 portrait screen gets 88889.

Fit metric (``map_fit``). With the HUD off, what lies outside the map is the
dark void. Pixels brighter than ``dark`` count as map. The map fits when the
frame border (a strip ``border`` of the short side wide) holds almost no map
pixels, i.e. all four map corners are inside the frame. ``fill`` (the map's
bounding box over the frame area) says how much of the frame it uses; among
the heights that fit, the lowest (largest map on screen) is chosen.
"""

from __future__ import annotations

import math

import numpy as np

REF_Y = 50000.0
REF_FOV = 19.0
REF_ASPECT = 16 / 9
REF_SIZE = (2560, 1440)
FIT_FACTORS = (0.85, 1.0, 1.15)   # the default and its neighbours tried by the probe
RULE = "y = 50000 * tan(9.5 deg) / tan(fov/2) * max(1, 1/a, a/(16/9)), a = width/height"


def default_camera_y(width: int, height: int, fov: float = REF_FOV) -> float:
    a = width / height
    k = math.tan(math.radians(REF_FOV) / 2) / math.tan(math.radians(fov) / 2)
    return float(round(REF_Y * k * max(1.0, 1.0 / a, a / REF_ASPECT)))


def map_fit(img: np.ndarray, dark: int = 24, border: float = 0.03,
            max_border_map: float = 0.01, min_map: float = 0.02) -> dict:
    """Fit metric of one BGR (or grey) frame; see the module docstring."""
    grey = img.mean(axis=2) if img.ndim == 3 else img.astype(float)
    mask = grey > dark
    h, w = mask.shape
    b = max(1, round(border * min(h, w)))
    edge = np.ones_like(mask)
    edge[b:h - b, b:w - b] = False
    border_map = float(mask[edge].mean())
    map_frac = float(mask.mean())
    found = map_frac >= min_map
    fill, bbox = 0.0, None
    if found:
        rows, cols = np.where(mask.any(axis=1))[0], np.where(mask.any(axis=0))[0]
        bbox = [int(cols[0]), int(rows[0]), int(cols[-1] - cols[0] + 1), int(rows[-1] - rows[0] + 1)]
        fill = bbox[2] * bbox[3] / (w * h)
    return {"map_found": found, "border_map_fraction": round(border_map, 4),
            "map_fraction": round(map_frac, 4), "fill": round(fill, 4), "bbox": bbox,
            "fits": bool(found and border_map <= max_border_map)}


def choose_height(steps: list[dict], default: float) -> tuple[float, str]:
    """steps: [{"y": ..., "fit": map_fit(...) or None}]. Lowest fitting height,
    else the default with the reason."""
    measured = [s for s in steps if s.get("fit")]
    fitting = [s for s in measured if s["fit"]["fits"]]
    if fitting:
        return min(s["y"] for s in fitting), "visual: lowest height whose frame holds the whole map"
    if not measured:
        return default, "default: no usable screenshot (none taken or all black)"
    return default, "default: no tested height showed the whole map"
