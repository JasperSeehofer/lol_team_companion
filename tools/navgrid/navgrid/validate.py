# SPDX-License-Identifier: AGPL-3.0-only
"""Check a navgrid against public structure positions. Numbers only.

Checks:
- every Bayes turret and inhibitor (structures.py) lies within 2 cells of the
  centroid of a small blocking cluster (an island of blocking cells that fits
  inside a 13 x 13 window around it) or of a bit-0x04 cluster, and, when the
  region layers parsed, in the matching nearest-lane region (side and lane;
  nexus turrets count as mid);
- both fountain spawn points are on walkable cells;
- the plan's bit-0x04 structure-wall hypothesis (reported, not part of pass);
- the flag histogram;
- with a Match-V5 timeline: the BUILDING_KILL and TURRET_PLATE_DESTROYED
  positions get the same cluster distance, the distance to the nearest
  table structure and, from the event's laneType and teamId (the building's
  owner), the same lane check.

The output holds distances, counts and pass/fail only: no cell coordinates,
no geometry.
"""

from __future__ import annotations

import json
from collections import deque
from pathlib import Path

import numpy as np

from navgrid import ngrid, structures

MAX_CELLS = 2.0       # tolerance, in cells
WINDOW = 6            # half-size of the search window, in cells
HYPOTHESIS_RADIUS = 300.0  # world units: a bit-4 cell this close to a structure counts as "at" it


def _components(mask: np.ndarray):
    """4-connected components of a small boolean array: list of (cells, touches_edge)."""
    seen = np.zeros_like(mask, dtype=bool)
    h, w = mask.shape
    out = []
    for z0, x0 in zip(*np.nonzero(mask)):
        if seen[z0, x0]:
            continue
        q = deque([(z0, x0)])
        seen[z0, x0] = True
        cells, edge = [], False
        while q:
            z, x = q.popleft()
            cells.append((z, x))
            if z in (0, h - 1) or x in (0, w - 1):
                edge = True
            for dz, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                zz, xx = z + dz, x + dx
                if 0 <= zz < h and 0 <= xx < w and mask[zz, xx] and not seen[zz, xx]:
                    seen[zz, xx] = True
                    q.append((zz, xx))
        out.append((np.array(cells), edge))
    return out


def point_check(g: ngrid.NavGrid, x: float, y: float) -> dict:
    """Distances (in cells) from a world point to nearby blocking structure clusters."""
    ix, iz = g.cell_of(x, y)
    if not g.inside(ix, iz):
        return {"in_grid": False, "cluster_cells": None, "bit4_cells": None,
                "nearest_blocking_cells": None, "within": False}
    z0, z1 = max(iz - WINDOW, 0), min(iz + WINDOW + 1, g.count_z)
    x0, x1 = max(ix - WINDOW, 0), min(ix + WINDOW + 1, g.count_x)
    win = g.flags[z0:z1, x0:x1]
    px = (x - g.min_pos[0]) / g.cell_size - x0 - 0.5   # point in window cell units
    pz = (y - g.min_pos[2]) / g.cell_size - z0 - 0.5

    def best(mask, isolated_only):
        d = None
        for cells, edge in _components(mask):
            if isolated_only and edge:
                continue
            cz, cx = cells.mean(axis=0)
            dist = float(np.hypot(cx - px, cz - pz))
            d = dist if d is None else min(d, dist)
        return d

    blocking = (win & ngrid.BLOCKING) != 0
    cluster = best(blocking, True)
    bit4 = best((win & ngrid.STRUCTURE_WALL) != 0, False)
    nz, nx = np.nonzero(blocking)
    nearest = float(np.min(np.hypot(nx - px, nz - pz))) if len(nz) else None
    cands = [d for d in (cluster, bit4) if d is not None]
    return {
        "in_grid": True,
        "cluster_cells": None if cluster is None else round(cluster, 2),
        "bit4_cells": None if bit4 is None else round(bit4, 2),
        "nearest_blocking_cells": None if nearest is None else round(nearest, 2),
        "within": bool(cands) and min(cands) <= MAX_CELLS,
    }


def lane_match(g: ngrid.NavGrid, x: float, y: float, side: str | None, lane: str | None) -> bool | None:
    """True/False when the nearest-lane layer can be checked at (x, y), else None."""
    ix, iz = g.cell_of(x, y)
    if g.regions is None or not side or not lane or not g.inside(ix, iz):
        return None
    return int(g.regions["nearest_lane"][iz, ix]) in ngrid.lane_code(side, lane)


def validate(g: ngrid.NavGrid, timeline: dict | None = None) -> dict:
    lanes = g.regions is not None
    rows = {}
    for name, (x, y) in structures.all_structures().items():
        r = point_check(g, x, y)
        side, lane = structures.check_lane(name)
        r["lane_match"] = lane_match(g, x, y, side, lane)
        r["pass"] = bool(r["within"] and r["lane_match"] is not False)
        rows[name] = r

    fountains = {}
    for name, (x, y) in structures.FOUNTAINS.items():
        ix, iz = g.cell_of(x, y)
        fountains[name] = bool(g.inside(ix, iz) and not (g.flags[iz, ix] & ngrid.BLOCKING))

    # bit-4 hypothesis
    bit4 = (g.flags & ngrid.STRUCTURE_WALL) != 0
    total4 = int(bit4.sum())
    near = 0
    if total4:
        zz, xx = np.nonzero(bit4)
        wx = g.min_pos[0] + (xx + 0.5) * g.cell_size
        wz = g.min_pos[2] + (zz + 0.5) * g.cell_size
        pos = np.array(list(structures.all_structures().values()), float)
        d = np.min(np.hypot(wx[:, None] - pos[:, 0], wz[:, None] - pos[:, 1]), axis=1)
        near = int((d <= HYPOTHESIS_RADIUS).sum())
    with4 = sum(1 for r in rows.values() if r["bit4_cells"] is not None and r["bit4_cells"] <= MAX_CELLS)
    hypothesis = {
        "bit4_cells": total4,
        "bit4_cells_near_known_structures": near,
        "share_near": round(near / total4, 3) if total4 else None,
        "structures_with_bit4_within_2_cells": with4,
        "structures": len(rows),
        "supported": bool(total4 and with4 >= 0.8 * len(rows) and near / total4 >= 0.5),
        "note": "nexus centres are not in the table, so bit-4 cells at the nexus count as not near",
    }

    n_pass = sum(r["pass"] for r in rows.values())
    out = {
        "navgrid_version": g.version,
        "counts": [g.count_x, g.count_z],
        "cell_size": g.cell_size,
        "layers_parsed": g.layers_parsed,
        "parse_report": g.report,
        "tolerance_cells": MAX_CELLS,
        "structures_pass": n_pass,
        "structures_total": len(rows),
        "lane_regions_checked": lanes,
        "structures": rows,
        "fountains_walkable": fountains,
        "structure_wall_hypothesis": hypothesis,
        "flag_histogram": ngrid.flag_histogram(g.flags),
        "pass": bool(n_pass == len(rows) and all(fountains.values())),
    }
    if timeline is not None:
        out["timeline"] = timeline_check(g, timeline)
    return out


def timeline_events(timeline: dict) -> list[tuple[str, float, float, str | None, str | None]]:
    """(type, x, y, side, lane) for BUILDING_KILL and TURRET_PLATE_DESTROYED; side
    (the building's owner) and lane from teamId and laneType when present.
    Nothing else is read."""
    frames = (timeline.get("info") or {}).get("frames") or timeline.get("frames") or []
    out = []
    for fr in frames:
        for ev in fr.get("events") or []:
            t = ev.get("type")
            pos = ev.get("position")
            if t in ("BUILDING_KILL", "TURRET_PLATE_DESTROYED") and isinstance(pos, dict):
                out.append((t, float(pos["x"]), float(pos["y"]),
                            structures.TIMELINE_TEAMS.get(ev.get("teamId")),
                            structures.TIMELINE_LANES.get(ev.get("laneType"))))
    return out


def timeline_check(g: ngrid.NavGrid, timeline: dict) -> dict:
    table = np.array(list(structures.all_structures().values()), float)
    rows = []
    for t, x, y, side, lane in timeline_events(timeline):
        r = point_check(g, x, y)
        r["type"] = t
        r["lane_match"] = lane_match(g, x, y, side, lane)
        r["nearest_table_structure_units"] = round(float(np.min(np.hypot(table[:, 0] - x, table[:, 1] - y))), 1)
        rows.append(r)
    kills = [r for r in rows if r["type"] == "BUILDING_KILL"]
    return {
        "events": len(rows),
        "building_kills": len(kills),
        "building_kills_within": sum(r["within"] for r in kills),
        "plates": len(rows) - len(kills),
        "lane_checked": sum(r["lane_match"] is not None for r in rows),
        "lane_match": sum(r["lane_match"] is True for r in rows),
        "rows": rows,
    }


def validate_file(path: Path, timeline_path: Path | None = None) -> dict:
    g = ngrid.parse(Path(path).read_bytes())
    tl = json.loads(Path(timeline_path).read_text(encoding="utf-8")) if timeline_path else None
    return validate(g, tl)
