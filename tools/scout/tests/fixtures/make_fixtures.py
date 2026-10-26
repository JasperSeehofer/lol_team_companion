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


# ---------------------------------------------------------------- top-down series (M156 R2)
# A 1 Hz top-down recording at 1920x1080 (UI scale 0.75 of the 2560x1440
# reference, camera at the reference height): champion health bars split at
# the 1000-HP separators with varying HP, blue mana bars under every bar,
# 3- and 4-pip ward bars of both teams. The first frame (t=600) is stale: it
# shows the game 15 s earlier, as the client's first recorded frame does.
# Generated on demand (tests write it to tmp_path); nothing here is game data.

TD_SIZE = (1920, 1080)
TD_SCALE = TD_SIZE[1] / 1440
TD_OFFSET = (51.5 * TD_SCALE + 0.8, 144.4 * TD_SCALE - 1.3)   # true bar left -> unit, px
# world -> unit pixel: ~0.06 px per unit, slightly tilted (a homography)
TD_H = np.array([[0.0600, 0.0015, 960.0 - 0.0600 * 7375 - 0.0015 * 7430],
                 [0.0010, -0.0600, 540.0 + 0.0600 * 7430 - 0.0010 * 7375],
                 [0.0, 0.0000010, 1.0]])
TD_TIMES = list(range(600, 641))
TD_STALE_LAG = 15.0
TD_TIMELINE_T = [600.249, 615.27, 630.3, 640.11]
TD_BAR_W, TD_BAR_H = round(104 * TD_SCALE), round(12 * TD_SCALE)
HSV = {"hp_blue": (102, 195, 236), "mana": (100, 174, 222), "pip_blue": (100, 255, 195),
       "hp_red": (2, 210, 220), "pip_red": (2, 255, 200)}
# (team, world x, y, pips, placed at): the first one is placed before the recording
TD_WARDS = [("red", 9000, 4200, 3, 590.0), ("blue", 5000, 10600, 3, 609.85),
            ("red", 10400, 6200, 4, 621.9), ("blue", 3500, 6100, 4, 630.85),
            ("red", 8200, 11000, 3, 635.2)]
TD_BASE = {"Aatrox": (2200, 11800), "LeeSin": (4300, 7600), "Ahri": (6600, 6900),
           "Jinx": (10800, 2300), "Thresh": (11900, 1700), "Garen": (3200, 12900),
           "Vi": (9600, 8600), "Lux": (8400, 6300), "Caitlyn": (12600, 3400),
           "Leona": (13200, 5000)}


def bgr(name: str) -> tuple:
    px = cv2.cvtColor(np.uint8([[HSV[name]]]), cv2.COLOR_HSV2BGR)[0, 0]
    return tuple(int(v) for v in px)


def td_world(champ: str, t: float) -> tuple[float, float]:
    i = CHAMPIONS.index(champ)
    bx, by = TD_BASE[champ]
    a = (t - 600) / 40.0
    return (bx + 900 * a * np.cos(i), by + 900 * a * np.sin(i * 1.7))


def td_unit_px(x: float, y: float) -> tuple[float, float]:
    v = TD_H @ np.array([x, y, 1.0])
    return float(v[0] / v[2]), float(v[1] / v[2])


def td_bar_left(champ: str, t: float) -> tuple[float, float]:
    ux, uy = td_unit_px(*td_world(champ, t))
    return ux - TD_OFFSET[0], uy - TD_OFFSET[1]


def td_hp(champ: str, t: float) -> tuple[float, float]:
    """(HP fraction, max HP)."""
    i = CHAMPIONS.index(champ)
    return 0.45 + 0.5 * abs(np.sin(0.3 * t + i)), 2600.0 + 220 * i


def td_frame(t: float) -> np.ndarray:
    w, h = TD_SIZE
    img = np.full((h, w, 3), (50, 62, 48), np.uint8)
    cv2.line(img, (300, 1000), (1650, 80), (95, 85, 60), 40)            # dull river
    cv2.circle(img, (150, 950), 40, bgr("hp_blue"), -1)                  # round blob, not a bar
    shown = t - TD_STALE_LAG if t == TD_TIMES[0] else t
    for team, x, y, n, placed in TD_WARDS:
        if shown < placed:
            continue
        ux, uy = td_unit_px(x, y)
        widths, spacing = ([24, 21, 21], 24.5) if n == 3 else ([18, 15, 15, 15], 18.0)
        x0 = ux - 36 * TD_SCALE
        for k, pw in enumerate(widths):
            px0 = int(round(x0 + k * spacing * TD_SCALE))
            pw_ = int(round(pw * TD_SCALE))
            cv2.rectangle(img, (px0 - 1, int(uy) - 2), (px0 + pw_, int(uy) + 2), (0, 0, 0), -1)
            cv2.rectangle(img, (px0, int(uy) - 1), (px0 + pw_ - 1, int(uy) + 1), bgr(f"pip_{team}"), -1)
    for i, c in enumerate(CHAMPIONS):
        team = "blue" if i < 5 else "red"
        left, top = td_bar_left(c, shown)
        x0, y0 = int(round(left)), int(round(top - TD_BAR_H / 2))
        frac, max_hp = td_hp(c, shown)
        cv2.rectangle(img, (x0 - 1, y0 - 1), (x0 + TD_BAR_W, y0 + TD_BAR_H), (0, 0, 0), -1)
        cv2.rectangle(img, (x0, y0), (x0 + TD_BAR_W - 1, y0 + TD_BAR_H - 1), (40, 40, 40), -1)
        fill = int(round(TD_BAR_W * frac))
        cv2.rectangle(img, (x0, y0), (x0 + fill - 1, y0 + TD_BAR_H - 1), bgr(f"hp_{team}"), -1)
        hp = 1000.0
        while hp < max_hp * frac:                                          # the 1000-HP separators
            sx = x0 + int(round(TD_BAR_W * hp / max_hp))
            cv2.line(img, (sx, y0), (sx, y0 + TD_BAR_H - 1), (20, 20, 20), 2)
            hp += 1000.0
        mana = int(round(TD_BAR_W * (0.6 + 0.4 * abs(np.cos(0.2 * t + i)))))
        my = int(round(top + 11.8 * TD_SCALE))
        cv2.rectangle(img, (x0, my - 1), (x0 + mana - 1, my + 1), bgr("mana"), -1)
    return img


def td_timeline() -> dict:
    """Timeline for the series: participantFrames near the frames, WARD_PLACED."""
    frames = []
    for k, tf in enumerate(TD_TIMELINE_T):
        evs = [{"type": "WARD_PLACED", "timestamp": int(placed * 1000), "creatorId":
                (2 if team == "blue" else 7) + j % 3, "wardType":
                "CONTROL_WARD" if n == 4 else "YELLOW_TRINKET"}
               for j, (team, _, _, n, placed) in enumerate(TD_WARDS)
               if (TD_TIMELINE_T[k - 1] if k else 0) < placed <= tf]
        frames.append({"timestamp": int(round(tf * 1000)), "events": evs, "participantFrames": {
            str(i + 1): {"participantId": i + 1, "position": {
                "x": round(td_world(c, tf)[0]), "y": round(td_world(c, tf)[1])}}
            for i, c in enumerate(CHAMPIONS)}})
    return {"metadata": {"matchId": MATCH_ID}, "info": {"frameInterval": 15000, "frames": frames}}


def td_minimap_track(noise: float = 250.0) -> dict:
    rng = np.random.default_rng(17)
    samples = [{"t": float(t), "pos": {c: [round(td_world(c, t)[0] + rng.normal(0, noise), 1),
                                           round(td_world(c, t)[1] + rng.normal(0, noise), 1)]
                                       for c in CHAMPIONS}} for t in TD_TIMES]
    return {"matchId": MATCH_ID, "source": "minimap", "champions": CHAMPIONS, "samples": samples}


def td_anchors() -> dict:
    """Turret anchors: the structures' unit pixels under the true homography."""
    return {"pairs": [{"name": n, "pixel": [round(v, 2) for v in td_unit_px(*w)]}
                      for n, w in TURRETS.items() if not n.endswith("nexus")]}


def write_topdown_series(dest: Path) -> Path:
    dest.mkdir(parents=True, exist_ok=True)
    for t in TD_TIMES:
        cv2.imwrite(str(dest / f"t{t:05d}.png"), td_frame(t), [cv2.IMWRITE_PNG_COMPRESSION, 1])
    return dest


# ---------------------------------------------------------------- run_synth (M166 A1)
# A whole synthetic run directory in the run-dir contract of M156 (file names
# unchanged), already minimised as A5's fetch_timeline writes it. scout.halfc
# runs end to end on it. Expected outcome under the proposed thresholds:
# A GO, B GO, wards blue and red NO-GO (hand check 3/4), recalls GO, route A+B.

RUN_SYNTH = HERE / "run_synth"
WARD_TIMES = [95, 130, 170, 230]          # rounded WARD_PLACED times (EVENTS)
TOPDOWN_RANGE = (120, 240)
TOPDOWN_WARDS_T = [(131.0, "red", 9800, 9000), (171.0, "blue", 5200, 9000),
                   (200.0, "red", 9000, 5000), (231.0, "red", 9800, 5200)]


def synth_probe() -> dict:
    def shot(name, b):
        return {"file": name, "mean_brightness": b, "black": b < 3}

    heights = [2000, 5000, 10000, 20000, 40000, 60000, 80000]
    cam_cap = 60000
    return {
        "errors": {},
        "spec": {"answered": "/swagger/v3/openapi.json", "tried": {"/swagger/v3/openapi.json": 200},
                 "paths": ["/replay/game", "/replay/playback", "/replay/recording", "/replay/render"]},
        "game": {"processID": 4242},
        "interface": {"interfaceAll": {"sent": False, "read": False, "honoured": True},
                      "interfaceMinimap": {"sent": True, "read": True, "honoured": True},
                      "screenshot": shot("probe_minimap_only.png", 41.3)},
        "speed": {"steps": [{"sent": s, "read": float(min(s, 8)),
                             "game_s_per_wall_s": round(min(s, 8) * 0.98, 2)}
                            for s in [1, 2, 4, 8, 10, 16]],
                  "max_honoured": 8, "window_s": 3.0},
        "recording": {
            "realtime": {"read_back": {"recording": True, "codec": "png", "lossless": True,
                                       "enforceFrameRate": True, "replaySpeed": 1.0},
                         "wall_s": 10.2, "game_s": 10.0, "game_s_per_wall_s": 0.98,
                         "frames_written": 40, "frames_expected": 40, "frame_size": [1920, 1080],
                         "scratch": "scout-rec/probe-realtime"},
            "accelerated": {"read_back": {"recording": True, "codec": "png", "lossless": True,
                                          "enforceFrameRate": False, "replaySpeed": 4.0},
                            "wall_s": 2.7, "game_s": 10.0, "game_s_per_wall_s": 3.7,
                            "frames_written": 40, "frames_expected": 40, "frame_size": [1920, 1080],
                            "scratch": "scout-rec/probe-accelerated"}},
        "camera": {"steps": [{"sent_y": h, "read_y": float(min(h, cam_cap)),
                              "read_farClip": h * 1.5 + 10000, "honoured": h <= cam_cap,
                              "screenshot": shot(f"probe_camera_{h}.png", 30.0 + i)}
                             for i, h in enumerate(heights)],
                   "max_honoured_y": cam_cap,
                   "note": "whether the whole map is in view is read from the screenshots"},
        "health_bars": {b: {"off": {"sent": False, "read": False, "honoured": True},
                            "on": {"sent": True, "read": True, "honoured": True}, "honoured": True}
                        for b in ["healthBarChampions", "healthBarStructures", "healthBarWards",
                                  "healthBarPets", "healthBarMinions"]},
        "fog": {"fogOfWar": {"sent": False, "read": False, "honoured": True}},
        "base": "https://127.0.0.1:2999",
    }


SYNTH_SPEC = {"openapi": "3.0.0", "info": {"title": "synthetic replay api"},
              "paths": {"/replay/game": {}, "/replay/playback": {"post": {"speed": "number"}},
                        "/replay/recording": {"post": {"replaySpeed": "number",
                                                       "enforceFrameRate": "boolean"}},
                        "/replay/render": {"post": {"interfaceAll": "boolean",
                                                    "interfaceMinimap": "boolean",
                                                    "farClip": "number", "fogOfWar": "boolean"}}}}


def minimised_timeline() -> dict:
    tl = timeline()
    del tl["metadata"]["participants"]
    tl["info"]["participants"] = [{"participantId": i} for i in range(1, 11)]
    return tl


def minimised_match() -> dict:
    roles = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"] * 2
    return {"metadata": {"dataVersion": "2", "matchId": MATCH_ID},
            "info": {"gameVersion": "16.20.700.0000", "gameDuration": 360, "participants": [
                {"participantId": i + 1, "championName": c, "teamId": 100 if i < 5 else 200,
                 "teamPosition": roles[i], "win": i < 5} for i, c in enumerate(CHAMPIONS)]}}


def synth_minimap_track() -> dict:
    from scout.track import find_gaps

    tr = track_synth()
    tr["source"] = "minimap"
    tr["gaps"] = find_gaps(tr["samples"], CHAMPIONS)
    tr["calibration"] = bounds_map(MINIMAP_PX, MINIMAP_PX).as_dict()
    tr["frame_size"] = [MINIMAP_PX, MINIMAP_PX]
    return tr


def synth_topdown_track() -> dict:
    from scout.track import find_gaps

    rng = np.random.default_rng(21)
    samples, det_px = [], []
    for t in range(TOPDOWN_RANGE[0], TOPDOWN_RANGE[1] + 1):
        pos, dets = {}, []
        for i, c in enumerate(CHAMPIONS):
            p = position(c, t)
            if c == "Leona" or p is None:      # one bar never found: hit rate 0.9
                pos[c] = None
                continue
            pos[c] = [round(p[0] + rng.normal(0, 60), 1), round(p[1] + rng.normal(0, 60), 1)]
            px, py = world_to_topdown(*p)
            dets.append({"label": "champion", "team": "blue" if i < 5 else "red",
                         "x": round(px, 1), "y": round(py, 1), "score": 1.0})
        samples.append({"t": float(t), "pos": pos})
        det_px.append({"t": float(t), "dets": dets})
    wards = [{"t": t, "x": float(x), "y": float(y), "team": team, "last_seen": t + 2}
             for t, team, x, y in TOPDOWN_WARDS_T]
    return {"matchId": MATCH_ID, "source": "topdown", "processing": "r2", "fps": 1.0,
            "champions": CHAMPIONS, "samples": samples, "gaps": find_gaps(samples, CHAMPIONS),
            "detector": {"kind": "synthetic-bars"}, "missing_classes": [],
            "fit": {"source": "synthetic", "note": "positions written directly, nothing fitted"},
            "calibration": {"kind": "homography", "synthetic": True},
            "frame_size": list(TOPDOWN_SIZE), "wards": wards, "ward_units": "world",
            "detections_px": det_px,
            "recording": {"camera": {"mode": "api", "height": 60000}, "frames": 242}}


def synth_handcheck(mm: dict, td: dict, tl: dict, mt: dict) -> str:
    import csv
    import io

    from scout import score

    pmap = score.participant_map(mt)
    ts = [s["t"] for s in td["samples"]]
    wards = score.match_wards(td["wards"], tl, pmap, (min(ts), max(ts)))
    rows = score.handcheck_rows(wards, score.detect_recalls(mm, tl, pmap), pmap)
    for r in rows:   # Jasper's marks: every claimed event seen except the false ward
        r["jasper_says"] = "no" if "no WARD_PLACED" in r["auto_says"] else "yes"
    buf = io.StringIO()
    w = csv.DictWriter(buf, fieldnames=score.HANDCHECK_COLUMNS)
    w.writeheader()
    w.writerows(rows)
    return buf.getvalue()


def synth_minimap_frame(t: int, icons) -> np.ndarray:
    img = render_frame(t, icons)
    for i, wt in enumerate(WARD_TIMES):   # blue wards drawn once placed, red ones never
        if i % 2 == 0 and t >= wt:
            cv2.circle(img, (60 + 40 * i, 200), 4, (40, 220, 240), -1)
    return img


def make_run_synth(dest: Path = RUN_SYNTH) -> Path:
    import shutil

    from scout.calibrate import anchors_template

    if dest.exists():
        shutil.rmtree(dest)
    (dest / "frames_minimap").mkdir(parents=True)
    (dest / "frames_topdown").mkdir()
    png = [cv2.IMWRITE_PNG_COMPRESSION, 9]
    probe = synth_probe()
    for s in [probe["interface"]["screenshot"]] + [c["screenshot"] for c in probe["camera"]["steps"]]:
        img = np.full((36, 64, 3), int(s["mean_brightness"]), np.uint8)
        cv2.imwrite(str(dest / s["file"]), img, png)
    icons = [make_icon(i) for i in range(10)]
    times = sorted({0, 180, 360} | {t + d for t in WARD_TIMES for d in (-2, -1, 0, 1, 2)})
    for t in times:
        cv2.imwrite(str(dest / "frames_minimap" / f"t{t:05d}.png"), synth_minimap_frame(t, icons), png)
    td_img = cv2.resize(topdown_frame(), (480, 270), interpolation=cv2.INTER_AREA)
    for t in TOPDOWN_RANGE:
        cv2.imwrite(str(dest / "frames_topdown" / f"t{t:05d}.png"), td_img, png)
    mm, td = synth_minimap_track(), synth_topdown_track()
    tl, mt = minimised_timeline(), minimised_match()
    files = {"probe.json": (probe, 1), "openapi.json": (SYNTH_SPEC, 1),
             "track_minimap.json": (mm, None), "track_topdown.json": (td, None),
             "anchors_template.json": (anchors_template(), 1), "timeline.json": (tl, 1),
             "match.json": (mt, 1)}
    for name, (data, indent) in files.items():
        (dest / name).write_text(json.dumps(data, indent=indent) + "\n", encoding="utf-8")
    with open(dest / "handcheck.csv", "w", encoding="utf-8", newline="") as fh:
        fh.write(synth_handcheck(mm, td, tl, mt))
    (dest / "run.log").write_text(
        "2026-10-21 14:00:00,000 scout.probe INFO probe.json written; 0 sub-probes failed: none\n"
        "2026-10-21 14:20:00,000 scout.track_minimap INFO track_minimap.json: 361 samples\n"
        "2026-10-21 14:40:00,000 scout.track_topdown INFO track_topdown.json: 121 frames\n"
        "2026-10-21 14:45:00,000 scout.fetch_timeline INFO timeline.json: 7 frames; "
        "match.json: 10 participants (minimised)\n", encoding="utf-8")
    return dest


def main() -> None:
    icons_dir = HERE / "icons"
    frames_dir = HERE / "frames"
    for d in (icons_dir, frames_dir):
        d.mkdir(exist_ok=True)
    icons = [make_icon(i) for i in range(10)]
    for c, icon in zip(CHAMPIONS, icons):
        cv2.imwrite(str(icons_dir / f"{c}.png"), icon)
    for t in range(60, 70):
        cv2.imwrite(str(frames_dir / f"t{t:05d}.png"), render_frame(t, icons))
    for name, data in (("timeline_synth.json", timeline()), ("match_synth.json", match()),
                       ("track_synth.json", track_synth())):
        indent = None if name == "track_synth.json" else 1
        (HERE / name).write_text(json.dumps(data, indent=indent) + "\n", encoding="utf-8")
    make_run_synth()
    print("fixtures written to", HERE)


if __name__ == "__main__":
    main()
