# SPDX-License-Identifier: AGPL-3.0-only
"""`python -m navgrid vectorise GRID [--out DIR]`: navgrid -> GeoJSON layers.

Steps: (1) boolean rasters at the grid's 50 units; (2) upsample 8x and smooth
(one 3-pixel box pass per axis: corners round off by under 10 units);
(3) marching squares at 0.5 (written here; saddles resolved by the centre
value; rings start at their min-x point so simplification keeps a corner);
(4) Douglas-Peucker (shapely `simplify`) at 25 units ("macro") and 12 units
("vision"); (5) one GeoJSON per layer and level,
coordinates in game units [x, z] (= Match-V5 x, y).

Every file carries "derived": "riot-navgrid; do not ship before ticket 2".
Output inside a git work tree is refused.
"""

from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path

import numpy as np

from navgrid import __version__, ngrid
from navgrid.pcrun import inside_git_worktree

UPSAMPLE = 8
LEVELS = {"macro": 25.0, "vision": 12.0}
DERIVED = "riot-navgrid; do not ship before ticket 2"


class VectoriseRefused(RuntimeError):
    pass


def rasters(g: ngrid.NavGrid) -> dict[str, np.ndarray]:
    f = g.flags
    out = {
        "walkable": (f & ngrid.BLOCKING) == 0,
        "brush": (f & ngrid.BRUSH) != 0,
        "wall": (f & ngrid.BLOCKING) != 0,
        # a transparent wall is 0x02 | 0x40 without 0x04: on 16.21 (M166 R2) 0x40 also
        # marks every structure and gate cell; this mask agrees with the SRX layer's
        # TransparentWall on 1,681 of its 1,683 cells (both sets hold 1,683)
        "transparent": ((f & (ngrid.WALL | ngrid.TRANSPARENT_WALL)) == (ngrid.WALL | ngrid.TRANSPARENT_WALL))
        & ((f & ngrid.STRUCTURE_WALL) == 0),
        "structures": (f & ngrid.STRUCTURE_WALL) != 0,
        "gates": (f & (ngrid.BLUE_ONLY | ngrid.RED_ONLY)) != 0,
    }
    if g.regions is not None:
        r = g.regions
        out["river"] = (r["river"] & 16) != 0
        out["lanes"] = np.isin(r["main_region"], (2, 3, 4))
        out["camps"] = np.isin(r["poi"], (5, 6, 7, 8, 9, 10))
        out["pits"] = np.isin(r["poi"], ngrid.PIT_POI)
    return out


def _box3(a: np.ndarray, axis: int) -> np.ndarray:
    p = np.pad(a, [(1, 1) if ax == axis else (0, 0) for ax in range(2)], mode="edge")
    n = a.shape[axis]
    s = [slice(None)] * 2
    acc = np.zeros_like(a)
    for k in range(3):
        s[axis] = slice(k, k + n)
        acc = acc + p[tuple(s)]
    return acc / 3.0


def smooth_field(mask: np.ndarray, k: int = UPSAMPLE) -> np.ndarray:
    """Pad one zero cell, upsample k x k, one 3-pixel box pass per axis."""
    m = np.pad(mask.astype(np.float64), 1)
    up = np.kron(m, np.ones((k, k)))
    return _box3(_box3(up, 0), 1)


# segment table: case -> list of (edge, edge); edges: 0 top, 1 right, 2 bottom, 3 left
_SEGS = {
    1: [(3, 2)], 2: [(2, 1)], 3: [(3, 1)], 4: [(0, 1)], 6: [(0, 2)], 7: [(3, 0)],
    8: [(3, 0)], 9: [(0, 2)], 11: [(0, 1)], 12: [(3, 1)], 13: [(2, 1)], 14: [(3, 2)],
}
_SADDLE = {  # (centre inside, centre outside)
    5: ([(3, 0), (2, 1)], [(0, 1), (3, 2)]),
    10: ([(0, 1), (3, 2)], [(3, 0), (2, 1)]),
}


def marching_squares(F: np.ndarray, t: float = 0.5) -> list[np.ndarray]:
    """Closed rings (pixel coords x=col, y=row) of the t-level set. F must be 0 on its border."""
    H, W = F.shape
    ins = F >= t
    case = (ins[:-1, :-1] * 8 + ins[:-1, 1:] * 4 + ins[1:, 1:] * 2 + ins[1:, :-1] * 1).astype(np.int8)
    ii, jj = np.nonzero((case != 0) & (case != 15))

    def edge_id(i, j, e):
        # horizontal edge (i, j)-(i, j+1): 2*(i*W+j); vertical (i, j)-(i+1, j): 2*(i*W+j)+1
        if e == 0:
            return 2 * (i * W + j)
        if e == 2:
            return 2 * ((i + 1) * W + j)
        if e == 3:
            return 2 * (i * W + j) + 1
        return 2 * (i * W + j + 1) + 1

    adj: dict[int, list[int]] = defaultdict(list)
    for i, j in zip(ii.tolist(), jj.tolist()):
        c = int(case[i, j])
        if c in _SADDLE:
            centre = (F[i, j] + F[i, j + 1] + F[i + 1, j] + F[i + 1, j + 1]) / 4.0 >= t
            segs = _SADDLE[c][0 if centre else 1]
        else:
            segs = _SEGS[c]
        for e1, e2 in segs:
            a, b = edge_id(i, j, e1), edge_id(i, j, e2)
            adj[a].append(b)
            adj[b].append(a)

    def point(eid):
        cell, vert = divmod(eid, 2)
        i, j = divmod(cell, W)
        if vert:
            v0, v1 = F[i, j], F[i + 1, j]
            return (j, i + (t - v0) / (v1 - v0))
        v0, v1 = F[i, j], F[i, j + 1]
        return (j + (t - v0) / (v1 - v0), i)

    rings = []
    seen = set()
    for start in sorted(adj):
        if start in seen:
            continue
        ring = [start]
        seen.add(start)
        prev, cur = None, start
        while True:
            nbrs = adj[cur]
            nxt = nbrs[0] if nbrs[0] != prev else nbrs[1]
            if nxt == start:
                break
            if nxt in seen:  # should not happen on a zero-bordered field
                break
            ring.append(nxt)
            seen.add(nxt)
            prev, cur = cur, nxt
        if len(ring) >= 3:
            pts = np.array([point(e) for e in ring])
            # start at an extreme point (min x, then min y): Douglas-Peucker keeps
            # the start, so a ring never starts on a rounded corner's arc
            k = int(np.lexsort((pts[:, 1], pts[:, 0]))[0])
            pts = np.roll(pts, -k, axis=0)
            rings.append(np.vstack([pts, pts[:1]]))
    return rings


def polygons_from_rings(rings, to_world):
    """Even-odd nesting: rings at even depth are shells, odd depth holes of their parent."""
    from shapely.geometry import Point, Polygon
    from shapely.prepared import prep

    polys = [Polygon(to_world(r)) for r in rings]
    order = sorted(range(len(polys)), key=lambda k: polys[k].area, reverse=True)
    parent = {}
    depth = {}
    prepared = {}
    for pos, k in enumerate(order):
        pt = Point(polys[k].exterior.coords[0])
        par = None
        for q in reversed(order[:pos]):  # smallest containing ring first
            if polys[q].area <= polys[k].area:
                continue
            if q not in prepared:
                prepared[q] = prep(polys[q])
            if prepared[q].contains(pt):
                par = q
                break
        parent[k] = par
        depth[k] = 0 if par is None else depth[par] + 1
    shells = {k: [] for k in order if depth[k] % 2 == 0}
    for k in order:
        if depth[k] % 2 == 1:
            shells[parent[k]].append(polys[k].exterior.coords)
    return [Polygon(polys[k].exterior.coords, holes) for k, holes in sorted(shells.items())]


def vectorise_layer(g: ngrid.NavGrid, mask: np.ndarray, tolerance: float):
    from shapely import make_valid
    from shapely.geometry import Polygon
    from shapely.geometry.polygon import orient

    if not mask.any():
        return []
    F = smooth_field(mask)
    px = g.cell_size / UPSAMPLE

    def to_world(r):
        # pixel centre (col, row) of the padded, upsampled field -> world
        x = g.min_pos[0] + (r[:, 0] + 0.5) * px - g.cell_size
        z = g.min_pos[2] + (r[:, 1] + 0.5) * px - g.cell_size
        return np.column_stack([x, z])

    out = []
    for p in polygons_from_rings(marching_squares(F), to_world):
        s = p.simplify(tolerance, preserve_topology=True)
        if not s.is_valid:
            s = make_valid(s)
        parts = [s] if isinstance(s, Polygon) else [q for q in getattr(s, "geoms", []) if isinstance(q, Polygon)]
        out.extend(orient(q) for q in parts if not q.is_empty and q.area > 0)
    return out


def _coords(ring):
    return [[round(float(x), 1), round(float(z), 1)] for x, z in ring.coords]


def to_geojson(polys, props: dict) -> dict:
    feats = []
    for k, p in enumerate(polys):
        feats.append({
            "type": "Feature",
            "id": k,
            "properties": dict(props, area=round(p.area, 1)),
            "geometry": {"type": "Polygon",
                         "coordinates": [_coords(p.exterior)] + [_coords(h) for h in p.interiors]},
        })
    return {"type": "FeatureCollection", "properties": props, "features": feats}


def default_out(sha: str) -> Path:
    return Path.home() / ".local" / "share" / "navgrid" / "out" / sha[:8]


def run(grid_path: Path, out: Path | None = None) -> Path:
    data = Path(grid_path).read_bytes()
    sha = ngrid.sha256(data)
    out = Path(out) if out else default_out(sha)
    if inside_git_worktree(out):
        raise VectoriseRefused(f"refusing to write inside a git work tree: {out}")
    g = ngrid.parse(data)
    out.mkdir(parents=True, exist_ok=True)
    summary = {"grid_sha256": sha, "generator": f"navgrid {__version__}", "derived": DERIVED, "layers": {}}
    for layer, mask in rasters(g).items():
        for level, tol in LEVELS.items():
            polys = vectorise_layer(g, mask, tol)
            props = {"layer": layer, "level": level, "tolerance": tol, "grid_sha256": sha,
                     "generator": f"navgrid {__version__}", "units": "game units [x, z]", "derived": DERIVED}
            fc = to_geojson(polys, props)
            (out / f"{layer}.{level}.geojson").write_text(
                json.dumps(fc, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")
            summary["layers"][f"{layer}.{level}"] = {
                "polygons": len(polys), "holes": sum(len(p.interiors) for p in polys),
                "cells": int(mask.sum()), "area": round(sum(p.area for p in polys), 1)}
    (out / "vectorise.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return out


def main_vectorise(args) -> int:
    try:
        out = run(args.grid, args.out)
    except (VectoriseRefused, OSError, ngrid.NgridError) as e:
        print(f"navgrid vectorise: {e}", file=sys.stderr)
        return 2
    print(f"wrote {out}")
    return 0
