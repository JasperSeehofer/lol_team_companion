"""Pixel <-> world mappings for Summoner's Rift.

World units: x grows to the right (blue base bottom-left), y grows upwards.
Images: pixel x grows to the right, pixel y grows DOWN, so the minimap's
y axis is flipped against world y.

Two models:
- LinearMap: per-axis scale and offset, for the minimap (an axis-aligned,
  top-down picture of the map). Least squares on turret pixel <-> world pairs.
- Homography: screen -> world for the high, narrow-FOV top-down camera, which
  is close to orthographic but may be slightly tilted (cv2.findHomography on
  >= 4 nexus/turret pairs).
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import numpy as np

# Summoner's Rift bounds (hextechdocs map data).
MAP_X = (-120.0, 14870.0)
MAP_Y = (-120.0, 14980.0)

# Turret world coordinates, checked 2026-10-17: all 22 turrets match the
# Bayes Esports "Tower Coordinates" table exactly
# (https://docs.bayesesports.com/docs-data-portal/misc/tower_coords, read via
# https://web.archive.org/web/20241209153455/https://docs.bayesesports.com/docs-data-portal/misc/tower_coords),
# and the Match-V5 BUILDING_KILL / TURRET_PLATE_DESTROYED positions in the
# repo fixture tests/fixtures/riot_match_v5_timeline.json (10504,1029 and
# 13866,4505). The two nexus centres ("blue_nexus", "red_nexus") are NOT in
# that table: verify on run (half C checks them against the committed frames).
TURRETS: dict[str, tuple[float, float]] = {
    "blue_top_outer": (981, 10441), "blue_top_inner": (1512, 6699),
    "blue_top_inhib": (1169, 4287), "blue_mid_outer": (5846, 6396),
    "blue_mid_inner": (5048, 4812), "blue_mid_inhib": (3651, 3696),
    "blue_bot_outer": (10504, 1029), "blue_bot_inner": (6919, 1483),
    "blue_bot_inhib": (4281, 1253), "blue_nexus_top": (1748, 2270),
    "blue_nexus_bot": (2177, 1807), "blue_nexus": (1551, 1659),
    "red_top_outer": (4318, 13875), "red_top_inner": (7943, 13411),
    "red_top_inhib": (10481, 13650), "red_mid_outer": (8955, 8510),
    "red_mid_inner": (9767, 10113), "red_mid_inhib": (11134, 11207),
    "red_bot_outer": (13866, 4505), "red_bot_inner": (13327, 8226),
    "red_bot_inhib": (13624, 10572), "red_nexus_top": (12611, 13084),
    "red_nexus_bot": (13052, 12612), "red_nexus": (13204, 12993),
}

# Inhibitor world coordinates, same Bayes table (verified). Extra anchors.
INHIBITORS: dict[str, tuple[float, float]] = {
    "blue_top_inhibitor": (1169, 3573), "blue_mid_inhibitor": (3203, 3208),
    "blue_bot_inhibitor": (3454, 1241), "red_top_inhibitor": (11261, 13659),
    "red_mid_inhibitor": (11603, 11667), "red_bot_inhibitor": (13598, 11316),
}


@dataclass
class LinearMap:
    """world_x = ax * px + bx ; world_y = ay * py + by (ay < 0: y flip)."""

    ax: float
    bx: float
    ay: float
    by: float

    def to_world(self, px: float, py: float) -> tuple[float, float]:
        return self.ax * px + self.bx, self.ay * py + self.by

    def to_pixel(self, wx: float, wy: float) -> tuple[float, float]:
        return (wx - self.bx) / self.ax, (wy - self.by) / self.ay

    def as_dict(self) -> dict:
        return {"kind": "linear", "ax": self.ax, "bx": self.bx, "ay": self.ay, "by": self.by}


def bounds_map(width_px: float, height_px: float) -> LinearMap:
    """Default minimap mapping: the image spans exactly the map bounds."""
    ax = (MAP_X[1] - MAP_X[0]) / width_px
    ay = -(MAP_Y[1] - MAP_Y[0]) / height_px
    return LinearMap(ax=ax, bx=MAP_X[0], ay=ay, by=MAP_Y[1])


def fit_linear(pixels, worlds) -> LinearMap:
    """Least-squares per-axis fit from >= 2 pixel <-> world pairs."""
    p = np.asarray(pixels, dtype=float)
    w = np.asarray(worlds, dtype=float)
    if len(p) < 2 or p.shape != w.shape:
        raise ValueError("need >= 2 matching pixel/world pairs")
    ax, bx = np.polyfit(p[:, 0], w[:, 0], 1)
    ay, by = np.polyfit(p[:, 1], w[:, 1], 1)
    return LinearMap(float(ax), float(bx), float(ay), float(by))


@dataclass
class Homography:
    H: np.ndarray  # 3x3, pixel -> world

    def _apply(self, M, x, y):
        v = M @ np.array([x, y, 1.0])
        return float(v[0] / v[2]), float(v[1] / v[2])

    def to_world(self, px: float, py: float) -> tuple[float, float]:
        return self._apply(self.H, px, py)

    def to_pixel(self, wx: float, wy: float) -> tuple[float, float]:
        return self._apply(np.linalg.inv(self.H), wx, wy)

    def as_dict(self) -> dict:
        return {"kind": "homography", "H": self.H.tolist()}


def fit_homography(pixels, worlds) -> Homography:
    """Screen -> world homography from >= 4 pairs (plain least squares)."""
    import cv2

    p = np.asarray(pixels, dtype=np.float64)
    w = np.asarray(worlds, dtype=np.float64)
    if len(p) < 4 or p.shape != w.shape:
        raise ValueError("need >= 4 matching pixel/world pairs")
    H, _ = cv2.findHomography(p, w, 0)
    if H is None:
        raise ValueError("homography fit failed (points collinear?)")
    return Homography(H)


def residuals(mapping, pixels, worlds) -> list[float]:
    """World-unit error of each anchor under a fitted mapping."""
    out = []
    for (px, py), (wx, wy) in zip(pixels, worlds):
        x, y = mapping.to_world(px, py)
        out.append(float(np.hypot(x - wx, y - wy)))
    return out


def load_anchors(path: Path | str) -> tuple[list, list]:
    """anchors.json: {"pairs": [{"name": "blue_nexus", "pixel": [px, py]},
    {"world": [x, y], "pixel": [px, py]}, ...]}. A name looks up TURRETS, then INHIBITORS."""
    from .util import read_json

    pixels, worlds = [], []
    for pair in read_json(path)["pairs"]:
        if not pair.get("pixel"):
            continue
        world = pair.get("world") or TURRETS.get(pair["name"]) or INHIBITORS[pair["name"]]
        pixels.append(tuple(pair["pixel"]))
        worlds.append(tuple(world))
    return pixels, worlds


def anchors_template() -> dict:
    """An anchors.json to fill in: pixel left empty, one entry per structure."""
    return {"_how": "Open a frame in Paint, hover each structure's centre, copy "
                    "the pixel shown bottom-left into 'pixel' as [x, y]. At least 4.",
            "pairs": [{"name": n, "pixel": None} for n in (*TURRETS, *INHIBITORS)]}
