"""Detectors: champion icons on the minimap, health bars on the top-down view.

Every detector returns pixel-space Detection objects. Three implementations:
- TemplateDetector: classical, circular-masked template matching of the game's
  ten known champion icons. Needs no weights; used by the tests and as the
  fallback when pyLoL's YOLO weights are absent.
- YoloDetector: pyLoL's published YOLOv12 weights through ultralytics
  (AGPL-3.0), imported lazily so nothing else needs it.
- BarDetector: HSV threshold of team-coloured health bars, blobbed with
  connected components; champions vs wards by bar size. No training.
"""

from __future__ import annotations

import re
from dataclasses import asdict, dataclass, field
from pathlib import Path

import cv2
import numpy as np


@dataclass
class Detection:
    label: str          # champion id, or "champion"/"ward" for bar detections
    x: float            # pixel centre
    y: float
    score: float = 1.0
    team: str | None = None   # "blue" / "red" when known
    w: float = 0.0
    h: float = 0.0
    left: float | None = None   # bar detections: bounding-box left edge, px
    sat: float | None = None    # bar detections: mean HSV saturation of the blob

    def as_dict(self) -> dict:
        return asdict(self)


def norm_name(name: str) -> str:
    """'Lee Sin', 'LeeSin', 'leesin' -> 'leesin' (Data Dragon ids vs YOLO names)."""
    return re.sub(r"[^a-z0-9]", "", name.lower())


def circle_mask(size: int, frac: float = 0.85) -> np.ndarray:
    yy, xx = np.mgrid[:size, :size]
    c = (size - 1) / 2
    return (((xx - c) ** 2 + (yy - c) ** 2) <= (frac * size / 2) ** 2).astype(np.uint8) * 255


def load_icons(icon_dir: Path | str, champions: list[str]) -> dict[str, np.ndarray]:
    """Read {champion}.png for each champion; missing files raise."""
    icons = {}
    for champ in champions:
        path = Path(icon_dir) / f"{champ}.png"
        img = cv2.imread(str(path), cv2.IMREAD_COLOR)
        if img is None:
            raise FileNotFoundError(f"icon missing: {path}")
        icons[champ] = img
    return icons


def fetch_ddragon_icons(champions: list[str], out_dir: Path | str,
                        version: str | None = None) -> str:
    """Download square champion portraits from Data Dragon (run time, on the
    player's PC only; tests use committed synthetic icons). Returns the version."""
    import requests

    if version is None:
        version = requests.get("https://ddragon.leagueoflegends.com/api/versions.json",
                               timeout=20).json()[0]
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    for champ in champions:
        path = out / f"{champ}.png"
        if path.exists():
            continue
        url = f"https://ddragon.leagueoflegends.com/cdn/{version}/img/champion/{champ}.png"
        r = requests.get(url, timeout=20)
        r.raise_for_status()
        path.write_bytes(r.content)
    return version


class TemplateDetector:
    kind = "template"

    def __init__(self, icons: dict[str, np.ndarray], icon_px: int, threshold: float = 0.55,
                 max_per_label: int = 2, scales: tuple[float, ...] = (1.0,)):
        self.icon_px = icon_px
        self.threshold = threshold
        self.max_per_label = max_per_label
        self.templates: dict[str, list[tuple[np.ndarray, np.ndarray]]] = {}
        for name, img in icons.items():
            self.templates[name] = []
            for s in scales:
                size = max(8, int(round(icon_px * s)))
                t = cv2.resize(img, (size, size), interpolation=cv2.INTER_AREA)
                self.templates[name].append((t, circle_mask(size)))
        self.missing_classes: list[str] = []

    def describe(self) -> dict:
        return {"kind": self.kind, "icon_px": self.icon_px, "threshold": self.threshold}

    def detect(self, img: np.ndarray) -> list[Detection]:
        out: list[Detection] = []
        for name, variants in self.templates.items():
            best = None
            for tmpl, mask in variants:
                if tmpl.shape[0] > img.shape[0] or tmpl.shape[1] > img.shape[1]:
                    continue
                res = cv2.matchTemplate(img, tmpl, cv2.TM_CCOEFF_NORMED, mask=mask)
                res = np.nan_to_num(res, nan=-1.0, posinf=-1.0, neginf=-1.0)
                if best is None or res.max() > best[0].max():
                    best = (res, tmpl.shape[0])
            if best is None:
                continue
            res, size = best
            for _ in range(self.max_per_label):
                _, score, _, (x, y) = cv2.minMaxLoc(res)
                if score < self.threshold:
                    break
                out.append(Detection(name, x + size / 2, y + size / 2, float(score)))
                r = int(size * 0.6)
                res[max(0, y - r):y + r + 1, max(0, x - r):x + r + 1] = -1.0
        return out


class YoloDetector:
    """pyLoL's YOLOv12 weights (input 512x512). Import of ultralytics is lazy."""

    kind = "yolo"

    def __init__(self, weights: Path | str, champions: list[str], conf: float = 0.25,
                 imgsz: int = 512):
        from ultralytics import YOLO  # AGPL-3.0; only on the player's PC

        self.weights = str(weights)
        self.model = YOLO(self.weights)
        self.conf = conf
        self.imgsz = imgsz
        names = self.model.names  # {class index: name}; pyLoL never published the list
        wanted = {norm_name(c): c for c in champions}
        self.class_to_champ = {i: wanted[norm_name(n)] for i, n in names.items()
                               if norm_name(n) in wanted}
        found = set(self.class_to_champ.values())
        self.missing_classes = [c for c in champions if c not in found]
        self.n_classes = len(names)

    def describe(self) -> dict:
        return {"kind": self.kind, "weights": Path(self.weights).name, "conf": self.conf,
                "imgsz": self.imgsz, "n_classes": self.n_classes}

    def detect(self, img: np.ndarray) -> list[Detection]:
        h, w = img.shape[:2]
        res = self.model.predict(cv2.resize(img, (self.imgsz, self.imgsz)), imgsz=self.imgsz,
                                 conf=self.conf, verbose=False)[0]
        out = []
        for (cx, cy, _, _), cls, score in zip(res.boxes.xywh.tolist(), res.boxes.cls.tolist(),
                                             res.boxes.conf.tolist()):
            champ = self.class_to_champ.get(int(cls))
            if champ:
                out.append(Detection(champ, cx * w / self.imgsz, cy * h / self.imgsz, float(score)))
        return out


@dataclass
class BarParams:
    """HSV ranges and sizes of health bars on a top-down frame. Defaults are
    guesses for 1080p-1440p; half C tunes them on the player's frames."""

    blue_lo: tuple = (95, 120, 120)
    blue_hi: tuple = (125, 255, 255)
    red_lo: tuple = (0, 120, 120)
    red_hi: tuple = (8, 255, 255)
    red2_lo: tuple = (172, 120, 120)
    red2_hi: tuple = (180, 255, 255)
    min_aspect: float = 2.5        # bars are wide and flat
    champ_min_h: int = 6           # champion bars are taller than ward bars
    ward_min_w: int = 8
    champ_min_w: int = 12
    max_w: int = 200
    offset_y: float = 0.0          # bar centre -> unit position, screen px (track_topdown fits it)
    merge_dy: float = 3.0          # blobs of one champion bar: rows this close (px at 1440) ...
    merge_gap: float = 8.0         # ... gaps this small (the 1000-HP separators)
    extra: dict = field(default_factory=dict)


class BarDetector:
    kind = "hsv_bars"

    def __init__(self, params: BarParams | None = None):
        self.p = params or BarParams()
        self.missing_classes: list[str] = []

    def describe(self) -> dict:
        return {"kind": self.kind, **{k: v for k, v in asdict(self.p).items() if k != "extra"}}

    def _blobs(self, mask: np.ndarray, team: str, sat: np.ndarray | None = None) -> list[Detection]:
        """Ward blobs as found; champion bars merged first. A champion bar
        splits at its 1000-HP separators into blobs that can be too short for
        the aspect test on their own (tanky champions), so blobs of champion
        height on one row (rows within merge_dy, gaps within merge_gap, both
        times the UI scale frame height / 1440) are joined, then tested."""
        p = self.p
        scale = mask.shape[0] / 1440.0
        n, labels, stats, cents = cv2.connectedComponentsWithStats(mask, connectivity=8)
        out, parts = [], []
        for i in range(1, n):
            x, y, w, h, area = stats[i]
            if w > p.max_w:
                continue
            s = None
            if sat is not None:
                box = labels[y:y + h, x:x + w] == i
                s = float(sat[y:y + h, x:x + w][box].mean())
            cx, cy = cents[i]
            if h >= p.champ_min_h and w >= p.ward_min_w / 2:
                parts.append({"l": float(x), "r": float(x + w), "top": int(y), "bot": int(y + h),
                              "cy": float(cy), "area": float(area), "s": s})
                continue
            if w < p.ward_min_w or w < p.min_aspect * h:
                continue
            out.append(Detection("ward", float(cx), float(cy) + p.offset_y, 1.0, team, float(w),
                                 float(h), float(x), None if s is None else round(s, 1)))
        groups: list[list[dict]] = []
        for part in sorted(parts, key=lambda q: q["l"]):
            g = next((g for g in groups if abs(g[0]["cy"] - part["cy"]) <= p.merge_dy * scale
                      and part["l"] - max(q["r"] for q in g) <= p.merge_gap * scale), None)
            if g:
                g.append(part)
            else:
                groups.append([part])
        for g in groups:
            left, right = g[0]["l"], max(q["r"] for q in g)
            w, h = right - left, max(q["bot"] for q in g) - min(q["top"] for q in g)
            if w < p.champ_min_w or w < p.min_aspect * h or w > p.max_w:
                continue
            area = sum(q["area"] for q in g)
            cy = sum(q["cy"] * q["area"] for q in g) / area
            s = None if g[0]["s"] is None else round(sum(q["s"] * q["area"] for q in g) / area, 1)
            out.append(Detection("champion", left + (w - 1) / 2, cy + p.offset_y, 1.0, team,
                                 float(w), float(h), left, s))
        return out

    def detect(self, img: np.ndarray) -> list[Detection]:
        p = self.p
        hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
        blue = cv2.inRange(hsv, np.array(p.blue_lo), np.array(p.blue_hi))
        red = cv2.inRange(hsv, np.array(p.red_lo), np.array(p.red_hi)) | \
            cv2.inRange(hsv, np.array(p.red2_lo), np.array(p.red2_hi))
        sat = hsv[:, :, 1]
        return self._blobs(blue, "blue", sat) + self._blobs(red, "red", sat)
