"""Generate the synthetic test fixtures (deterministic). Run from anywhere:

  tools/scout/.venv/bin/python tools/scout/tests/fixtures/make_fixtures.py

Nothing here is Riot art or real game data: the "icons" are coloured circles
with glyphs, the minimap is drawn, and the timeline is invented in the shape
of the repository's tests/fixtures/riot_match_v5_timeline.json (Match-V5).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import cv2
import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1]))
from scout.calibrate import TURRETS, bounds_map  # noqa: E402

CHAMPIONS = ["Aatrox", "LeeSin", "Ahri", "Jinx", "Thresh",
             "Garen", "Vi", "Lux", "Caitlyn", "Leona"]
MINIMAP_PX = 320
ICON_PX = 26
MATCH_ID = "EUW1_7100000099"
BLUE_F, RED_F = (450.0, 450.0), (14300.0, 14350.0)

# Waypoints (t, x, y) per champion; positions interpolate linearly; None = dead.
LANE = {
    "Aatrox": (1500, 11000), "LeeSin": (3800, 8000), "Ahri": (6000, 7400),
    "Jinx": (11800, 1800), "Thresh": (10600, 1000), "Garen": (2500, 12300),
    "Vi": (11000, 7000), "Lux": (8400, 7400), "Caitlyn": (12800, 2600),
    "Leona": (13400, 4200),
}


def waypoints(champ: str) -> list[tuple]:
    i = CHAMPIONS.index(champ)
    f = BLUE_F if i < 5 else RED_F
    start = (0, f[0] + 40 * (i % 5), f[1] + 30 * (i % 5))
    lx, ly = LANE[champ]
    wps = [start, (50, *start[1:]), (60, lx, ly)]
    if champ == "Ahri":      # crosses Lux at t = 64.5 (the overlap frames)
        wps += [(69, 8400, 7400), (360, 8400, 7400)]
    elif champ == "Lux":
        wps += [(69, 6000, 7400), (360, 6000, 7400)]
    elif champ == "Jinx":    # recall: channel 190-198, at fountain 199-215, back to lane
        wps += [(198, lx, ly), (199, *BLUE_F), (215, *BLUE_F), (300, lx, ly), (360, lx, ly)]
    elif champ == "Caitlyn":  # near own fountain for 4 s only: not a recall
        wps += [(245, 12800, 12800), (250, 14000, 14000), (254, 14000, 14000),
                (258, 12800, 12800), (360, 12800, 12800)]
    elif champ == "Garen":   # dies at 280, respawns at fountain 300-315
        wps += [(280, lx, ly), (281, None, None), (299, None, None), (300, *RED_F),
                (315, *RED_F), (360, lx, ly)]
    else:
        wps += [(360, lx + 300, ly + 300)]
    return wps


def position(champ: str, t: float):
    wps = waypoints(champ)
    for (t0, x0, y0), (t1, x1, y1) in zip(wps, wps[1:]):
        if t0 <= t <= t1:
            if x0 is None or x1 is None:   # a segment touching a dead interval
                if t == t0 and x0 is not None:
                    return (x0, y0)
                if t == t1 and x1 is not None:
                    return (x1, y1)
                return None
            a = 0 if t1 == t0 else (t - t0) / (t1 - t0)
            return (x0 + a * (x1 - x0), y0 + a * (y1 - y0))
    return wps[-1][1:]


def draw_minimap(size: int = MINIMAP_PX) -> np.ndarray:
    img = np.full((size, size, 3), (40, 70, 35), np.uint8)
    lane, w = (60, 90, 110), 10
    s, e = 18, size - 18
    cv2.line(img, (s, e), (s, s), lane, w)
    cv2.line(img, (s, s), (e, s), lane, w)
    cv2.line(img, (s, e), (e, e), lane, w)
    cv2.line(img, (e, e), (e, s), lane, w)
    cv2.line(img, (s, e), (e, s), lane, w)
    cv2.line(img, (30, 30), (size - 30, size - 30), (110, 80, 40), 18)   # river
    cv2.circle(img, (20, size - 20), 26, (120, 70, 30), -1)
    cv2.circle(img, (size - 20, 20), 26, (40, 40, 120), -1)
    rng = np.random.default_rng(7)
    for x, y in rng.integers(40, size - 40, (14, 2)):
        cv2.circle(img, (int(x), int(y)), 4, (30, 50, 25), -1)
    return img


COLOURS = [(40, 40, 200), (200, 120, 30), (180, 60, 200), (30, 200, 230), (60, 180, 60),
           (200, 200, 60), (120, 40, 140), (230, 230, 230), (30, 120, 220), (90, 90, 90)]


def make_icon(i: int, size: int = 64) -> np.ndarray:
    base = COLOURS[i]
    ink = (20, 20, 20) if sum(base) > 380 else (245, 245, 245)
    img = np.full((size, size, 3), base, np.uint8)
    c, q = size // 2, size // 4
    if i == 0:
        cv2.line(img, (c, 8), (c, size - 8), ink, 8); cv2.line(img, (8, c), (size - 8, c), ink, 8)
    elif i == 1:
        for y in range(10, size, 16):
            cv2.line(img, (0, y), (size, y), ink, 6)
    elif i == 2:
        for x in range(10, size, 16):
            cv2.line(img, (x, 0), (x, size), ink, 6)
    elif i == 3:
        for dx in (-q, q):
            for dy in (-q, q):
                cv2.circle(img, (c + dx, c + dy), 7, ink, -1)
    elif i == 4:
        cv2.fillPoly(img, [np.array([[c, 10], [size - 10, size - 12], [10, size - 12]])], ink)
    elif i == 5:
        cv2.circle(img, (c, c), 16, ink, 7)
    elif i == 6:
        cv2.line(img, (10, 10), (size - 10, size - 10), ink, 8)
        cv2.line(img, (size - 10, 10), (10, size - 10), ink, 8)
    elif i == 7:
        img[:, : c] = ink
    elif i == 8:
        cv2.rectangle(img, (c - 12, c - 12), (c + 12, c + 12), ink, -1)
    else:
        for y in range(0, size, 16):
            for x in range(0, size, 16):
                if (x // 16 + y // 16) % 2:
                    img[y:y + 16, x:x + 16] = ink
    return img


def composite(img: np.ndarray, icon: np.ndarray, cx: float, cy: float, team: str) -> None:
    d = ICON_PX
    small = cv2.resize(icon, (d, d), interpolation=cv2.INTER_AREA)
    yy, xx = np.mgrid[:d, :d]
    r = np.hypot(xx - (d - 1) / 2, yy - (d - 1) / 2)
    x0, y0 = int(round(cx - d / 2)), int(round(cy - d / 2))
    ring = (230, 140, 40) if team == "blue" else (50, 50, 230)
    for yy_, xx_ in zip(*np.nonzero(r <= d / 2)):
        y, x = y0 + yy_, x0 + xx_
        if 0 <= y < img.shape[0] and 0 <= x < img.shape[1]:
            img[y, x] = ring if r[yy_, xx_] > d / 2 - 2 else small[yy_, xx_]


def render_frame(t: float, icons: list[np.ndarray]) -> np.ndarray:
    img = draw_minimap()
    m = bounds_map(MINIMAP_PX, MINIMAP_PX)
    for i, champ in enumerate(CHAMPIONS):   # later champions drawn on top (Lux over Ahri)
        p = position(champ, t)
        if p is not None:
            composite(img, icons[i], *m.to_pixel(*p), "blue" if i < 5 else "red")
    return img


def pframe(pid: int, p) -> dict:
    x, y = (round(p[0]), round(p[1])) if p else (round(RED_F[0]), round(RED_F[1]))
    return {"participantId": pid, "position": {"x": x, "y": y}, "currentGold": 500,
            "totalGold": 500 + 50 * pid, "xp": 0, "level": 1, "minionsKilled": 0,
            "jungleMinionsKilled": 0}


EVENTS = [
    {"type": "WARD_PLACED", "timestamp": 95000, "creatorId": 5, "wardType": "YELLOW_TRINKET"},
    {"type": "WARD_PLACED", "timestamp": 130500, "creatorId": 10, "wardType": "YELLOW_TRINKET"},
    {"type": "WARD_PLACED", "timestamp": 170200, "creatorId": 2, "wardType": "CONTROL_WARD"},
    {"type": "WARD_PLACED", "timestamp": 230000, "creatorId": 7, "wardType": "SIGHT_WARD"},
    {"type": "WARD_PLACED", "timestamp": 300000, "creatorId": 3, "wardType": "UNDEFINED"},
    {"type": "ITEM_PURCHASED", "timestamp": 201000, "participantId": 4, "itemId": 1055},
    {"type": "ITEM_PURCHASED", "timestamp": 202000, "participantId": 4, "itemId": 1036},
    {"type": "ITEM_PURCHASED", "timestamp": 203500, "participantId": 4, "itemId": 2003},
    {"type": "ITEM_PURCHASED", "timestamp": 302000, "participantId": 6, "itemId": 1037},
    {"type": "CHAMPION_KILL", "timestamp": 280500, "killerId": 2, "victimId": 6,
     "position": {"x": 2500, "y": 12300}, "assistingParticipantIds": []},
]


def timeline() -> dict:
    frames = []
    for k in range(7):
        t = 60 * k
        evs = [e for e in EVENTS if 60000 * (k - 1) < e["timestamp"] <= 60000 * k]
        frames.append({"timestamp": t * 1000, "events": evs, "participantFrames": {
            str(i + 1): pframe(i + 1, position(c, t)) for i, c in enumerate(CHAMPIONS)}})
    return {"_fixture_notes": {"source": "tools/scout/tests/fixtures/make_fixtures.py; synthetic, "
                               "shape of tests/fixtures/riot_match_v5_timeline.json"},
            "metadata": {"dataVersion": "2", "matchId": MATCH_ID,
                         "participants": [f"synth-puuid-{i:02d}" for i in range(1, 11)]},
            "info": {"frameInterval": 60000, "gameId": 7100000099, "frames": frames,
                     "participants": [{"participantId": i, "puuid": f"synth-puuid-{i:02d}"}
                                      for i in range(1, 11)]}}


def match() -> dict:
    return {"metadata": {"dataVersion": "2", "matchId": MATCH_ID},
            "info": {"gameDuration": 360, "participants": [
                {"participantId": i + 1, "championName": c, "teamId": 100 if i < 5 else 200,
                 "puuid": f"synth-puuid-{i + 1:02d}"} for i, c in enumerate(CHAMPIONS)]}}


def track_synth(noise: float = 40.0) -> dict:
    """A 1 Hz track as a perfect detector with Gaussian noise would give."""
    rng = np.random.default_rng(3)
    samples = []
    for t in range(0, 361):
        pos = {}
        for c in CHAMPIONS:
            p = position(c, t)
            pos[c] = None if p is None else [round(p[0] + rng.normal(0, noise), 1),
                                             round(p[1] + rng.normal(0, noise), 1)]
        samples.append({"t": float(t), "pos": pos})
    return {"matchId": MATCH_ID, "source": "synthetic", "fps": 1.0, "champions": CHAMPIONS,
            "samples": samples, "gaps": [], "detector": {"kind": "synthetic"},
            "missing_classes": [], "wards": []}


# Top-down frame: a slightly tilted camera, i.e. a homography world -> pixel.
TOPDOWN_SIZE = (1280, 720)
H_WORLD_TO_PX = np.array([[0.046, 0.004, 300.0], [0.002, -0.046, 700.0], [0.0, 0.0000015, 1.0]])
TOPDOWN_T = 600


def world_to_topdown(x: float, y: float) -> tuple[float, float]:
    v = H_WORLD_TO_PX @ np.array([x, y, 1.0])
    return float(v[0] / v[2]), float(v[1] / v[2])


TOPDOWN_CHAMPS = {c: (LANE[c][0] + 500, LANE[c][1] + 500) for c in CHAMPIONS}
TOPDOWN_WARDS = [("blue", 5200, 9000), ("red", 9800, 5200), ("blue", 7000, 3000)]


def topdown_frame() -> np.ndarray:
    rng = np.random.default_rng(11)
    w, h = TOPDOWN_SIZE
    noise = cv2.resize(rng.normal(0, 14, (h // 16, w // 16, 3)), (w, h), interpolation=cv2.INTER_NEAREST)
    img = np.clip(noise + (45, 60, 50), 0, 255).astype(np.uint8)
    cv2.circle(img, (200, 640), 30, (230, 120, 30), -1)   # saturated blue blob, not a bar
    cv2.line(img, (100, 600), (1200, 100), (110, 90, 60), 25)  # dull river
    colours = {"blue": (230, 120, 30), "red": (40, 40, 220)}
    for i, c in enumerate(CHAMPIONS):
        px, py = world_to_topdown(*TOPDOWN_CHAMPS[c])
        team = "blue" if i < 5 else "red"
        x0, y0 = int(px - 30), int(py - 4)
        cv2.rectangle(img, (x0 - 1, y0 - 1), (x0 + 60, y0 + 8), (0, 0, 0), -1)
        cv2.rectangle(img, (x0, y0), (x0 + 59, y0 + 7), colours[team], -1)
    for team, x, y in TOPDOWN_WARDS:
        px, py = world_to_topdown(x, y)
        x0, y0 = int(px - 10), int(py - 2)
        cv2.rectangle(img, (x0 - 1, y0 - 1), (x0 + 20, y0 + 4), (0, 0, 0), -1)
        cv2.rectangle(img, (x0, y0), (x0 + 19, y0 + 3), colours[team], -1)
    return img


def main() -> None:
    icons_dir = HERE / "icons"
    frames_dir = HERE / "frames"
    td_dir = HERE / "topdown"
    for d in (icons_dir, frames_dir, td_dir):
        d.mkdir(exist_ok=True)
    icons = [make_icon(i) for i in range(10)]
    for c, icon in zip(CHAMPIONS, icons):
        cv2.imwrite(str(icons_dir / f"{c}.png"), icon)
    for t in range(60, 70):
        cv2.imwrite(str(frames_dir / f"t{t:05d}.png"), render_frame(t, icons))
    cv2.imwrite(str(td_dir / f"t{TOPDOWN_T:05d}.png"), topdown_frame())
    anchors = {"pairs": [{"name": n, "pixel": [round(v, 2) for v in world_to_topdown(*w)]}
                         for n, w in TURRETS.items() if n.endswith(("outer", "nexus"))]}
    for name, data in (("timeline_synth.json", timeline()), ("match_synth.json", match()),
                       ("track_synth.json", track_synth()), ("anchors_topdown.json", anchors)):
        indent = None if name == "track_synth.json" else 1
        (HERE / name).write_text(json.dumps(data, indent=indent) + "\n", encoding="utf-8")
    print("fixtures written to", HERE)


if __name__ == "__main__":
    main()
