# SPDX-License-Identifier: AGPL-3.0-only
"""A fabricated v7.1 navgrid: the only grid the tests ever read.

Nothing here comes from a Riot file. It is a made-up, deliberately asymmetric
map on Summoner's Rift's grid size (295 x 296 cells of 50 units), so that
structure positions from the public Bayes table (structures.py) land on
structure blobs and a swapped or shifted axis fails validation:

- a 2-cell border wall and two terrain blocks with no mirror image;
- a brush patch (bit 0x01) of 16 x 10 cells;
- a diagonal transparent wall (bits 0x02 | 0x40, as on the 16.21 real file);
- a river band along x + z = 14700 (river bit 16, river main regions);
- two lanes in the main-region layer (top: west and north edges, bot: south
  and east edges); the nearest-lane layer is the nearest structure's lane;
- one camp (POI CampRedBuff) and two pits: POI DragonPit (4) and the Baron
  pit coded 12, as on the 16.21 real file (the converter's 3 is unused there);
- structure blobs (bits 0x02 | 0x04 | 0x40, 3 x 3 cells, the 16.21 encoding) at
  every turret and inhibitor of the Bayes table, each with a walkable moat;
- base gates (0x0400 / 0x0800 with 0x40, half of them also 0x1000; never 0x02)
  near each fountain: walkable, fountains walkable.
"""

from __future__ import annotations

import numpy as np

from navgrid import ngrid, structures

COUNT_X, COUNT_Z = 295, 296
CELL = 50.0
MIN_X, MIN_Z = -100.0, -150.0   # asymmetric on purpose
BRUSH_RECT = (6000, 6800, 2400, 2900)          # x0, x1, z0, z1 in world units
WALL_BLOCKS = ((3000, 4500, 6000, 8000), (9500, 11500, 5000, 5600), (10900, 11500, 5000, 6400))
DIAGONAL = ((7500, 3500), (8500, 4500))
CAMP = (2600, 8400, 200)                      # x, z, radius
PIT = (9800, 4300, 400)
BARON_PIT = (5000, 10400, 400)
BLOB_HALF = 1                                  # 3 x 3 cells
MOAT = 4                                       # cells cleared around each blob
HINT_NODES = 900


def _world_grids():
    ix = np.arange(COUNT_X)
    iz = np.arange(COUNT_Z)
    wx = MIN_X + (ix + 0.5) * CELL
    wz = MIN_Z + (iz + 0.5) * CELL
    return np.meshgrid(wx, wz)  # each (COUNT_Z, COUNT_X)


def _cell(x, z):
    return int((x - MIN_X) // CELL), int((z - MIN_Z) // CELL)


def make(hint_nodes: int = HINT_NODES) -> ngrid.NavGrid:
    X, Z = _world_grids()
    flags = np.zeros((COUNT_Z, COUNT_X), np.uint16)

    # terrain
    flags[:2, :] = flags[-2:, :] = ngrid.WALL
    flags[:, :2] = flags[:, -2:] = ngrid.WALL
    for x0, x1, z0, z1 in WALL_BLOCKS:
        flags[(X >= x0) & (X < x1) & (Z >= z0) & (Z < z1)] |= ngrid.WALL
    (ax, az), (bx, bz) = DIAGONAL
    for t in np.linspace(0.0, 1.0, 200):
        cx, cz = _cell(ax + t * (bx - ax), az + t * (bz - az))
        flags[cz, cx] = ngrid.WALL | ngrid.TRANSPARENT_WALL
    x0, x1, z0, z1 = BRUSH_RECT
    flags[(X >= x0) & (X < x1) & (Z >= z0) & (Z < z1)] = ngrid.BRUSH

    # structures with a walkable moat
    for x, z in structures.all_structures().values():
        cx, cz = _cell(x, z)
        sl = (slice(max(cz - MOAT, 0), cz + MOAT + 1), slice(max(cx - MOAT, 0), cx + MOAT + 1))
        flags[sl] &= ~np.uint16(ngrid.BLOCKING_R1)
    for x, z in structures.all_structures().values():
        cx, cz = _cell(x, z)
        flags[cz - BLOB_HALF:cz + BLOB_HALF + 1, cx - BLOB_HALF:cx + BLOB_HALF + 1] = (
            ngrid.WALL | ngrid.STRUCTURE_WALL | ngrid.TRANSPARENT_WALL)

    # gates and fountains
    blue_gate = (X >= 1900) & (X < 2000) & (Z >= 300) & (Z < 900)
    red_gate = (X >= 12750) & (X < 12850) & (Z >= 13800) & (Z < 14400)
    flags[blue_gate] = ngrid.BLUE_ONLY | ngrid.TRANSPARENT_WALL
    flags[red_gate] = ngrid.RED_ONLY | ngrid.TRANSPARENT_WALL
    flags[(blue_gate | red_gate) & (Z < 600)] |= ngrid.NEUTRAL_VIS
    flags[(blue_gate | red_gate) & (Z > 14100)] |= ngrid.NEUTRAL_VIS
    for x, z in structures.FOUNTAINS.values():
        cx, cz = _cell(x, z)
        flags[cz - 3:cz + 4, cx - 3:cx + 4] &= ~np.uint16(ngrid.BLOCKING_R1)

    # region layers
    s = X + Z
    river_band = np.abs(s - 14700) < 600
    top_lane = ((X < 1700) & (Z > 3000)) | ((Z > 13000) & (X < 11700))
    bot_lane = ((Z < 1700) & (X > 3000)) | ((X > 13000) & (Z < 11700))
    main = np.where(Z > X, 5, 6).astype(np.uint8)            # top/bot side jungle
    main[river_band] = np.where(Z > X, 7, 8)[river_band]       # top/bot side river
    main[s < 7000] = 1
    main[s > 22400] = 1
    main[top_lane] = 2
    main[bot_lane] = 4
    for x, z in structures.FOUNTAINS.values():
        main[(X - x) ** 2 + (Z - z) ** 2 < 800 ** 2] = 0

    river = np.zeros_like(main)
    jungle = (main == 5) | (main == 6)
    river[jungle] = 1
    river[river_band] |= 16

    quadrant = np.zeros_like(main)
    quadrant[jungle] = (1 + (Z > X).astype(np.uint8) + 2 * (s > 14700).astype(np.uint8))[jungle]

    names = list(structures.all_structures())
    pos = np.array([structures.all_structures()[k] for k in names], float)
    d2 = (X[..., None] - pos[:, 0]) ** 2 + (Z[..., None] - pos[:, 1]) ** 2
    nearest = np.argmin(d2, axis=-1)
    lane_of = []
    for k in names:
        side, lane = structures.side_lane(k)
        lane_of.append({"top": 0, "mid": 1, "bot": 2}[lane or "mid"] + (0 if side == "blue" else 3))
    nearest_lane = np.array(lane_of, np.uint8)[nearest]
    nearest_lane[(main >= 5) & (main <= 8)] += 6  # jungle and river: the lane's neutral zone

    poi = np.zeros_like(main)
    near_turret = np.min(d2, axis=-1) < (3 * CELL) ** 2
    poi[near_turret] = 1
    poi[(X - CAMP[0]) ** 2 + (Z - CAMP[1]) ** 2 < CAMP[2] ** 2] = 5
    poi[(X - PIT[0]) ** 2 + (Z - PIT[1]) ** 2 < PIT[2] ** 2] = 4
    poi[(X - BARON_PIT[0]) ** 2 + (Z - BARON_PIT[1]) ** 2 < BARON_PIT[2] ** 2] = 12

    dist_blue = np.hypot(X - 554, Z - 581)
    dist_red = np.hypot(X - 14302, Z - 14387)
    blue_side = dist_blue < dist_red
    band = np.clip((np.minimum(dist_blue, dist_red) // 2000).astype(np.uint8), 0, 4)
    ring = np.where(blue_side, band, band + 5).astype(np.uint8)

    srx = np.zeros_like(main)
    srx[flags & ngrid.WALL != 0] = 1
    srx[flags & ngrid.TRANSPARENT_WALL != 0] = 2
    srx[flags & ngrid.BRUSH != 0] = 3
    srx[flags & ngrid.STRUCTURE_WALL != 0] = 0   # 16.21: structures are Walkable in SRX
    srx[flags & ngrid.BLUE_ONLY != 0] = 7
    srx[flags & ngrid.RED_ONLY != 0] = 8

    regions = {"river": river, "jungle_quadrant": quadrant, "main_region": main,
               "nearest_lane": nearest_lane, "poi": poi, "ring": ring, "srx": srx}

    # heights: a gentle slope plus a ripple; 0 under blocking cells
    height = (20.0 + 0.004 * X + 30.0 * np.sin(Z / 900.0)).astype(np.float32)
    cells = np.zeros((COUNT_Z, COUNT_X), ngrid.CELL_DTYPE)
    cells["center_height"] = np.where(flags & ngrid.BLOCKING, 0.0, height)
    iz, ix = np.indices((COUNT_Z, COUNT_X))
    cells["x"] = ix
    cells["z"] = iz
    cells["hint_weight"] = 1.0

    sx, sz = 2 * COUNT_X + 1, 2 * COUNT_Z + 1
    hx = MIN_X + np.arange(sx) * CELL / 2
    hz = MIN_Z + np.arange(sz) * CELL / 2
    HX, HZ = np.meshgrid(hx, hz)
    samples = (20.0 + 0.004 * HX + 30.0 * np.sin(HZ / 900.0)).astype(np.float32)

    unknown = bytes((i * 31 + 7) & 0xFF for i in range(ngrid.UNKNOWN_BLOCK))
    hn = hint_nodes
    k = np.arange(hn)
    hint_x = (k % 30) * 10
    hint_z = (k // 30) * 10
    dist = np.hypot(hint_x[:, None] - hint_x[None, :], hint_z[:, None] - hint_z[None, :]).round()
    hints = ngrid.HintNodes(dist.astype(np.float32), np.stack([hint_x, hint_z], 1).astype(np.int16))

    return ngrid.NavGrid(
        7, 1, (MIN_X, -50.0, MIN_Z), (MIN_X + COUNT_X * CELL, 200.0, MIN_Z + COUNT_Z * CELL),
        CELL, COUNT_X, COUNT_Z, cells, flags, regions, unknown,
        ngrid.HeightSamples(sx, sz, MIN_X, MIN_Z, samples), hints,
    )


def make_bytes(hint_nodes: int = HINT_NODES) -> bytes:
    return ngrid.write(make(hint_nodes))
