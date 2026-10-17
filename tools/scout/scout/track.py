"""Kalman tracking with an identity constraint: ten known champions, one each.

Each champion owns exactly one track for the whole game. Per sample the
detections (already in world units) are assigned to tracks with the Hungarian
algorithm on cost = distance to the track's prediction + a penalty when the
detector's label differs from the track's champion. A detection carries at
most one track and a track takes at most one detection. Same-label detections
are accepted at any distance (recall, teleport, respawn); a different label
only within `mismatch_gate`, so an overlap cannot swap identities.
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
class WorldDet:
    label: str
    x: float
    y: float
    score: float = 1.0


class Tracker:
    def __init__(self, champions: list[str], mismatch_gate: float = 600.0,
                 mismatch_penalty: float = 2000.0, jump_reset: float = 3000.0):
        if len(set(champions)) != len(champions):
            raise ValueError("champions must be distinct")
        self.champions = list(champions)
        self.mismatch_gate = mismatch_gate
        self.mismatch_penalty = mismatch_penalty
        self.jump_reset = jump_reset
        self.filters: dict[str, KalmanCV | None] = {c: None for c in champions}
        self.last_t: float | None = None
        self.samples: list[dict] = []

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
