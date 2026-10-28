"""Kalman tracking with an identity constraint: ten known champions, one each.

Each champion owns exactly one track for the whole game. Per sample the
detections (already in world units) are assigned to tracks with the Hungarian
algorithm on cost = distance to the track's prediction + a penalty when the
detector's label differs from the track's champion. Only the best-scoring
detection of each label takes part (finding K): each champion has one icon,
and a second same-label peak (a dark portrait matching a static background
spot) would otherwise become an absorbing state of the track. A detection carries at
most one track and a track takes at most one detection. Same-label detections
are accepted at any distance (recall, teleport, respawn); a different label
only within `mismatch_gate`, so an overlap cannot swap identities.

Possessions (Viego's passive): after a takedown Viego may take the victim's
soul and then wears the victim's portrait in his own team's ring until he
casts his ultimate, the possession ends, or he takes another soul. The
template detector ignores the ring, so that icon carries the victim's label.
While a victim is dead its own icon is off the map, so for `window` seconds
after the takedown a detection with the victim's label is handed to the
possessor; one inside the victim's fountain means the victim has respawned and
closes that window. Pickup delay, an early end and a switch to another soul
need no timing: they only change which label the possessor's icon has. The
window also ends when the possessor dies, and a victim-labelled detection
farther from the takedown than the possessor could have walked (a background
patch matching the dead victim) is dropped.
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np
from scipy.optimize import linear_sum_assignment

BIG = 1e9


class KalmanCV:
    """Constant-velocity Kalman filter on state [x, y, vx, vy] (numpy only)."""

    def __init__(self, x: float, y: float, q: float = 200.0, r: float = 60.0):
        self.s = np.array([x, y, 0.0, 0.0])
        self.P = np.diag([r * r, r * r, 500.0 ** 2, 500.0 ** 2])
        self.q = q
        self.R = np.eye(2) * r * r

    def predict(self, dt: float) -> np.ndarray:
        F = np.eye(4)
        F[0, 2] = F[1, 3] = dt
        G = np.array([[dt * dt / 2, 0], [0, dt * dt / 2], [dt, 0], [0, dt]])
        self.s = F @ self.s
        self.P = F @ self.P @ F.T + G @ G.T * self.q * self.q
        return self.s[:2].copy()

    def update(self, z) -> None:
        Hm = np.zeros((2, 4))
        Hm[0, 0] = Hm[1, 1] = 1.0
        y = np.asarray(z, dtype=float) - Hm @ self.s
        S = Hm @ self.P @ Hm.T + self.R
        K = self.P @ Hm.T @ np.linalg.inv(S)
        self.s = self.s + K @ y
        self.P = (np.eye(4) - K @ Hm) @ self.P

    @property
    def pos(self) -> tuple[float, float]:
        return float(self.s[0]), float(self.s[1])


@dataclass
class Possession:
    """A takedown after which `possessor` may wear `victim`'s icon (world units)."""
    possessor: str
    victim: str
    t: float                         # takedown, game seconds
    fountain: tuple[float, float]    # the victim's fountain: it respawns there
    at: tuple[float, float] | None = None   # takedown position; None skips the reach check
    until: float | None = None       # the possessor's next death, if before the window ends


POSSESS_WINDOW_S = 25.0      # soul pickup delay plus the longest possession: verify on run
POSSESS_GRACE_S = 1.0        # the frame of the takedown may still show the victim alive
POSSESS_FOUNTAIN_RADIUS = 1000.0
POSSESS_REACH = 2000.0       # possessor's distance from the takedown (assists happen at range)
POSSESS_SPEED = 450.0        # world units per second the possessor can walk after it


@dataclass
class WorldDet:
    label: str
    x: float
    y: float
    score: float = 1.0


class Tracker:
    def __init__(self, champions: list[str], mismatch_gate: float = 600.0,
                 mismatch_penalty: float = 2000.0, jump_reset: float = 3000.0,
                 possessions: list[Possession] | None = None,
                 possess_window: float = POSSESS_WINDOW_S):
        if len(set(champions)) != len(champions):
            raise ValueError("champions must be distinct")
        self.champions = list(champions)
        self.mismatch_gate = mismatch_gate
        self.mismatch_penalty = mismatch_penalty
        self.jump_reset = jump_reset
        self.filters: dict[str, KalmanCV | None] = {c: None for c in champions}
        self.last_t: float | None = None
        self.samples: list[dict] = []
        self.possessions = sorted(possessions or [], key=lambda p: p.t)
        self.possess_window = possess_window
        self.respawned: set[int] = set()   # indexes into possessions closed at the fountain
        self.relabelled: dict[str, int] = {}   # victim -> detections handed to the possessor
        self.dropped: dict[str, int] = {}      # victim -> detections out of the possessor's reach

    def _relabel_possessed(self, t: float, dets: list[WorldDet]) -> list[WorldDet]:
        live: dict[str, tuple[int, Possession]] = {}   # victim -> latest open takedown
        for i, p in enumerate(self.possessions):
            end = p.t + self.possess_window if p.until is None else min(p.until, p.t + self.possess_window)
            if p.t + POSSESS_GRACE_S <= t <= end and i not in self.respawned:
                live[p.victim] = (i, p)
        if not live:
            return dets
        out = []
        for d in dets:
            hit = live.get(d.label)
            if hit is None:
                out.append(d)
                continue
            i, p = hit
            if np.hypot(d.x - p.fountain[0], d.y - p.fountain[1]) <= POSSESS_FOUNTAIN_RADIUS:
                self.respawned.add(i)
                out.append(d)
            elif (p.at is not None and np.hypot(d.x - p.at[0], d.y - p.at[1])
                  > POSSESS_REACH + POSSESS_SPEED * (t - p.t)):
                self.dropped[p.victim] = self.dropped.get(p.victim, 0) + 1
            else:
                self.relabelled[p.victim] = self.relabelled.get(p.victim, 0) + 1
                out.append(WorldDet(p.possessor, d.x, d.y, d.score))
        return out

    def _cost(self, champ: str, pred, det: WorldDet) -> float:
        same = det.label == champ
        if pred is None:  # not seen yet: only its own icon can start the track
            return 0.0 if same else BIG
        d = float(np.hypot(pred[0] - det.x, pred[1] - det.y))
        if same:
            return d
        return d + self.mismatch_penalty if d <= self.mismatch_gate else BIG

    def step(self, t: float, dets: list[WorldDet]) -> dict[str, list[float] | None]:
        dt = 1.0 if self.last_t is None else max(1e-3, t - self.last_t)
        self.last_t = t
        preds = {c: (f.predict(dt) if f is not None else None) for c, f in self.filters.items()}
        pos: dict[str, list[float] | None] = {c: None for c in self.champions}
        dets = best_per_label(self._relabel_possessed(t, dets))
        if dets:
            cost = np.array([[self._cost(c, preds[c], d) for d in dets] for c in self.champions])
            rows, cols = linear_sum_assignment(cost)
            for r, k in zip(rows, cols):
                if cost[r, k] >= BIG:
                    continue
                champ, det = self.champions[r], dets[k]
                f = self.filters[champ]
                p = preds[champ]
                if f is None or np.hypot(p[0] - det.x, p[1] - det.y) > self.jump_reset:
                    self.filters[champ] = KalmanCV(det.x, det.y)
                else:
                    f.update((det.x, det.y))
                pos[champ] = [round(det.x, 1), round(det.y, 1)]
        self.samples.append({"t": t, "pos": pos})
        return pos


def best_per_label(dets: list[WorldDet]) -> list[WorldDet]:
    """Keep only the highest-scoring detection of each label (finding K).

    Same-label cost is plain distance, accepted at any distance; offered two
    peaks of one label, the Hungarian step takes the one nearest the
    prediction, so once a track touches a static false spot (a first-frame
    tie, or a frame where the real icon is hidden) it never leaves."""
    best: dict[str, WorldDet] = {}
    for d in dets:
        if d.label not in best or d.score > best[d.label].score:
            best[d.label] = d
    return list(best.values())


class StaticSpots:
    """Finding F: a champion's recurring static spots, in pixel coordinates.

    A dark portrait matches a fixed patch of background (the off-map strip,
    a jungle wall) in nearly every frame. After K the track still lands there
    while the champion is dead or hidden. Score threshold and margin rules
    both fail on Jasper's v2 frames, so the rule is about recurrence instead:
    per label, a spot (detections within `radius` px of each other) that is
    among that label's detections in >= `frac` of the frames is static, and
    detections there are rejected. A spot is counted at most once per frame,
    over every peak the detector reports for the label (the static patch is
    usually the second peak while the real icon is visible, so counting only
    the best would see it only while the champion is gone).

    Offline (two passes): observe() every frame, then filter() each frame
    against the final counts. Live (one pass): observe() then filter() frame
    by frame with the running counts. Either way nothing is static before
    `warmup` frames have been observed, so a short clip is left alone.
    """

    def __init__(self, radius: float = 4.0, frac: float = 0.2, warmup: int = 60):
        self.radius, self.frac, self.warmup = radius, frac, warmup
        self.frames = 0
        self.spots: dict[str, list[dict]] = {}   # label -> [{"x", "y", "n"}]
        self.rejected: dict[str, int] = {}

    def observe(self, dets) -> None:
        """One frame of detections with .label, .x, .y (pixels)."""
        self.frames += 1
        hit: set[int] = set()
        for d in dets:
            spots = self.spots.setdefault(d.label, [])
            near = min(((np.hypot(sp["x"] - d.x, sp["y"] - d.y), i) for i, sp in enumerate(spots)),
                       default=None)
            if near is not None and near[0] <= self.radius:
                sp = spots[near[1]]
                if id(sp) not in hit:
                    hit.add(id(sp))
                    sp["n"] += 1
                    sp["x"] += (d.x - sp["x"]) / sp["n"]   # running mean of the spot centre
                    sp["y"] += (d.y - sp["y"]) / sp["n"]
            else:
                sp = {"x": float(d.x), "y": float(d.y), "n": 1}
                spots.append(sp)
                hit.add(id(sp))

    def static(self) -> dict[str, list[dict]]:
        if self.frames < self.warmup:
            return {}
        need = self.frac * self.frames
        out = {}
        for label, spots in self.spots.items():
            keep = [sp for sp in spots if sp["n"] >= need]
            if keep:
                out[label] = keep
        return out

    def is_static(self, label: str, x: float, y: float, static: dict | None = None) -> bool:
        static = self.static() if static is None else static
        return any(np.hypot(sp["x"] - x, sp["y"] - y) <= self.radius for sp in static.get(label, []))

    def filter(self, dets: list, static: dict | None = None) -> list:
        static = self.static() if static is None else static
        out = []
        for d in dets:
            if self.is_static(d.label, d.x, d.y, static):
                self.rejected[d.label] = self.rejected.get(d.label, 0) + 1
            else:
                out.append(d)
        return out

    def describe(self, to_world=None) -> dict:
        """The static spots for the track JSON (`static_spots`)."""
        spots = []
        for label, sps in sorted(self.static().items()):
            for sp in sps:
                row = {"champion": label, "px": round(sp["x"], 1), "py": round(sp["y"], 1),
                       "frames": sp["n"], "frac": round(sp["n"] / self.frames, 3)}
                if to_world:
                    wx, wy = to_world(sp["x"], sp["y"])
                    row.update(wx=round(wx, 1), wy=round(wy, 1))
                spots.append(row)
        return {"rule": {"radius_px": self.radius, "min_frac": self.frac, "warmup": self.warmup},
                "frames": self.frames, "spots": spots, "rejected": dict(sorted(self.rejected.items()))}


def find_gaps(samples: list[dict], champions: list[str]) -> list[dict]:
    """Runs of consecutive samples without a position, per champion."""
    gaps = []
    for c in champions:
        start = prev = None
        for s in samples:
            missing = s["pos"].get(c) is None
            if missing and start is None:
                start = s["t"]
            if not missing and start is not None:
                gaps.append({"champion": c, "start": start, "end": prev,
                             "length_s": round(s["t"] - start, 3)})
                start = None
            prev = s["t"]
        if start is not None:
            # open gap at the end: length counts up to the last sample, inclusive
            step = samples[1]["t"] - samples[0]["t"] if len(samples) > 1 else 1.0
            gaps.append({"champion": c, "start": start, "end": prev,
                         "length_s": round(prev - start + step, 3)})
    return gaps
