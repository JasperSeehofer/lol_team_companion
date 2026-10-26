# SPDX-License-Identifier: AGPL-3.0-only
#
# The header, 48-byte cell stride and u16 flag array follow TheKillerey/
# MapgeoAddon navgrid.py at commit 8d2f8bd9f57d064c47a5b9c5c7e6e55597412d76
# (Copyright (c) 2026 TheKillerey, MIT; full notice in NOTICE.md).
# The v7 cell field list, the region-layer block, the 8 x 132-byte block, the
# height samples, the hint nodes and every enum below follow the layout and
# names documented by FrankTheBoxMonster/LoL-NGRID-converter
# (NGridFileReader.cs, NavGridCell.cs) at commit
# 92943ed2b2d5e82c86d680e69f53f247c89aefee, AGPL-3.0; see NOTICE.md.
# This is an independent Python implementation of those facts.
"""Parse and write `.aimesh_ngrid` navgrid files, version 7.x.

Layout (little-endian):

    u8 major (7), u16 minor
    f32 min_x, min_y, min_z, f32 max_x, max_y, max_z    (y is height)
    f32 cell_size, i32 count_x, i32 count_z             -> 39 bytes
    cells[n], 48 bytes each, index = z * count_x + x    (n = count_x * count_z)
    u16 flags[n]                                        vision/pathing bitfield
    --- optional tail (missing or short: reported, never an error) ---
    u8[4] regions[n]   byte0 river, byte1 jungle quadrant (lo) | main region (hi),
                       byte2 nearest lane (lo) | POI (hi), byte3 ring (lo) | SRX (hi)
    8 x 132 bytes      unknown block (kept raw)
    i32 sx, i32 sz, f32 offset_x, f32 offset_z, f32 samples[sx * sz]   height samples
    hint nodes: N x (f32 distances[N], i16 x, i16 z), N = 900 on SR
    anything after that: kept raw and reported

Axis convention: grid x is Match-V5 position.x, grid z is Match-V5 position.y.
Cell (ix, iz) covers [min_x + ix*cell, min_x + (ix+1)*cell) on x, the same on z.
Arrays are indexed [iz, ix].
"""

from __future__ import annotations

import hashlib
import struct
from dataclasses import dataclass, field

import numpy as np

HEADER = struct.Struct("<BH3f3ffii")  # 39 bytes
UNKNOWN_BLOCK = 8 * 132
SUPPORTED_MAJOR = 7

CELL_DTYPE = np.dtype([
    ("center_height", "<f4"),      # 0 for unwalkable cells; height samples are better
    ("session_id", "<i4"),
    ("arrival_cost", "<f4"),
    ("is_open", "<i4"),
    ("heuristic", "<f4"),
    ("x", "<i2"),
    ("z", "<i2"),
    ("actor_list", "<i4"),
    ("unknown1", "<i4"),
    ("good_cell_session_id", "<i4"),
    ("hint_weight", "<f4"),
    ("unknown2", "<i2"),
    ("arrival_direction", "<i2"),
    ("hint_node1", "<i2"),
    ("hint_node2", "<i2"),
])
assert CELL_DTYPE.itemsize == 48

REGION_LAYERS = ("river", "jungle_quadrant", "main_region", "nearest_lane", "poi", "ring", "srx")

# --- flag table: every reading per bit, with sources ------------------------
# plan = the Rift Board plan (team-companion-next-advance); mapgeo = MapgeoAddon
# navgrid.py docstring; converter = NavGridCell.cs VisionPathingFlags.
# "settled" records what the 16.21 real file showed in M166 R2 (numbers only:
# 87,320 cells, the 28 Bayes structures, one Match-V5 timeline); None = open.
FLAG_TABLE: tuple[dict, ...] = (
    {"bit": 0x0001, "plan": "brush", "mapgeo": "HAS_GRASS (brush)", "converter": "Brush",
     "settled": "brush (all readings agree): 40 clusters of 20-124 cells, 0/28 structures within 3 cells, "
                "SRX layer Brush/BrushWall on 2,129/2,129 cells; settled on 16.21 real file, R2"},
    {"bit": 0x0002, "plan": "wall", "mapgeo": "NOT_PASSABLE (wall / out of bounds)", "converter": "Wall",
     "settled": "not passable, including structures (all readings agree): 96/96 in-game event positions "
                "on cells without it, 337/340 per-minute player positions within 1 cell; settled on 16.21 "
                "real file, R2"},
    {"bit": 0x0004, "plan": "structure wall", "mapgeo": "BUSY", "converter": "StructureWall",
     "settled": "structure footprint (plan/converter): always with 0x02|0x40; 32 clusters = 28/28 "
                "turrets+inhibitors (centroid within 15 units) + 2 nexus + 2 fountain; SRX says Walkable "
                "under it; settled on 16.21 real file, R2"},
    {"bit": 0x0008, "plan": None, "mapgeo": "TARGETTED", "converter": "Unobserved8", "settled": None},
    {"bit": 0x0010, "plan": None, "mapgeo": "MARKED", "converter": "Unobserved16", "settled": None},
    {"bit": 0x0020, "plan": None, "mapgeo": "PATHED_ON", "converter": "Unobserved32", "settled": None},
    {"bit": 0x0040, "plan": "transparent wall", "mapgeo": "SEE_THROUGH",
     "converter": "TransparentWall (see-through, not walkable)",
     "settled": "see-through (MapgeoAddon): never blocks walking on its own; a transparent wall is "
                "0x02|0x40 without 0x04 (= SRX TransparentWall on 1,683/1,683 cells), it also marks every "
                "structure cell and every gate cell; settled on 16.21 real file, R2"},
    {"bit": 0x0080, "plan": None, "mapgeo": "vision-related (many walkable jungle cells)",
     "converter": "Unknown128 (sporadic noise)",
     "settled": None},  # R2: spatially coherent (not noise), 80 % of river cells; meaning open
    {"bit": 0x0100, "plan": "always visible", "mapgeo": None, "converter": "AlwaysVisible (not on SR)",
     "settled": None},  # R2: 0 cells on 16.21
    {"bit": 0x0200, "plan": None, "mapgeo": None, "converter": "Unknown512 (first Nexus Blitz only)",
     "settled": None},
    {"bit": 0x0400, "plan": "gate (blue)", "mapgeo": None, "converter": "BlueTeamOnly (base gate)",
     "settled": "blue-only gate (plan/converter): 2 clusters on the blue side, always with 0x40, never "
                "0x02; settled on 16.21 real file, R2"},
    {"bit": 0x0800, "plan": "gate (red)", "mapgeo": None, "converter": "RedTeamOnly (base gate)",
     "settled": "red-only gate (plan/converter), the mirror of 0x0400; settled on 16.21 real file, R2"},
    {"bit": 0x1000, "plan": "gate vision", "mapgeo": None, "converter": "NeutralZoneVisibility",
     "settled": None},  # R2: only on gate cells (4 clusters); meaning not tested
)
FLAG_SOURCES = {
    "plan": "Rift Board plan (brush 1, wall 2, structure wall 4, transparent wall 64, always-visible 256, gates)",
    "mapgeo": "TheKillerey/MapgeoAddon navgrid.py @ 8d2f8bd9",
    "converter": "FrankTheBoxMonster/LoL-NGRID-converter NavGridCell.cs @ 92943ed2",
    "settled": "M166 R2 on the 16.21 real file (numbers only; the file stays off every repo)",
}
BRUSH = 0x0001
WALL = 0x0002
STRUCTURE_WALL = 0x0004
TRANSPARENT_WALL = 0x0040
ALWAYS_VISIBLE = 0x0100
BLUE_ONLY = 0x0400
RED_ONLY = 0x0800
NEUTRAL_VIS = 0x1000
# Blocking for walkability and clusters. Settled on 16.21 real file, R2: wall
# (0x02) and structure (0x04) block; 0x40 alone does not (the base-gate cells
# carry 0x40 without 0x02). On 16.21 every 0x04 cell also has 0x02, so this
# equals MapgeoAddon's "not 0x02". R1 used 0x46, which made the 136 gate
# cells walls; both rules put 96/96 in-game event positions on walkable cells.
BLOCKING = WALL | STRUCTURE_WALL
BLOCKING_R1 = WALL | STRUCTURE_WALL | TRANSPARENT_WALL

# --- region enums (converter names; values are nibbles or bits) --------------
RIVER_BITS = {1: "JungleQuadrant", 2: "BaronPit", 4: "Unobserved4", 8: "Unobserved8",
              16: "River", 32: "Unknown32", 64: "RiverEntrance"}
JUNGLE_QUADRANT = {0: "None", 1: "North", 2: "East", 3: "West", 4: "South", 8: "Unobserved8"}
MAIN_REGION = {0: "Spawn", 1: "Base", 2: "TopLane", 3: "MidLane", 4: "BotLane",
               5: "TopSideJungle", 6: "BotSideJungle", 7: "TopSideRiver", 8: "BotSideRiver",
               9: "TopSideBasePerimeter", 10: "BotSideBasePerimeter",
               11: "TopSideLaneAlcove", 12: "BotSideLaneAlcove"}
NEAREST_LANE = {0: "BlueSideTopLane", 1: "BlueSideMidLane", 2: "BlueSideBotLane",
                3: "RedSideTopLane", 4: "RedSideMidLane", 5: "RedSideBotLane",
                6: "BlueSideTopNeutralZone", 7: "BlueSideMidNeutralZone", 8: "BlueSideBotNeutralZone",
                9: "RedSideTopNeutralZone", 10: "RedSideMidNeutralZone", 11: "RedSideBotNeutralZone"}
# POI 11 and 12 are not in the converter. On the 16.21 real file (M166 R2) the
# converter's BaronPit (3) has no cells and 12 is one 326-cell patch whose
# centre lies 28 units from the game's BARON_NASHOR kill; 11 is four ~35-cell
# patches away from every objective, meaning unknown.
POI = {0: "None", 1: "NearTurret", 2: "CloudDrakeWindTunnel", 3: "BaronPit", 4: "DragonPit",
       5: "CampRedBuff", 6: "CampBlueBuff", 7: "CampGromp", 8: "CampKrugs", 9: "CampRaptors",
       10: "CampMurkWolves", 11: "Unknown11", 12: "BaronPit (16.21)"}
PIT_POI = (3, 4, 12)
RING = {0: "BlueSpawnToNexus", 1: "BlueNexusToInhib", 2: "BlueInhibToInner", 3: "BlueInnerToOuter",
        4: "BlueOuterToNeutral", 5: "RedSpawnToNexus", 6: "RedNexusToInhib", 7: "RedInhibToInner",
        8: "RedInnerToOuter", 9: "RedOuterToNeutral"}
SRX = {0: "Walkable", 1: "Wall", 2: "TransparentWall", 3: "Brush", 4: "Unobserved4",
       5: "TopSideOceanDrakePuddle", 6: "BotSideOceanDrakePuddle", 7: "BlueTeamOnly", 8: "RedTeamOnly",
       9: "Unobserved9", 10: "Unobserved10", 11: "BlueTeamOnlyNeutralZoneVisibility",
       12: "RedTeamOnlyNeutralZoneVisibility", 13: "BrushWall"}


def lane_code(side: str, lane: str) -> set[int]:
    """NEAREST_LANE values that count as (side, lane): the lane and its neutral zone."""
    base = {"top": 0, "mid": 1, "bot": 2}[lane] + (0 if side == "blue" else 3)
    return {base, base + 6}


class NgridError(ValueError):
    pass


@dataclass
class HeightSamples:
    count_x: int
    count_z: int
    offset_x: float
    offset_z: float
    values: np.ndarray  # (count_z, count_x) float32


@dataclass
class HintNodes:
    distances: np.ndarray  # (N, N) float32
    coords: np.ndarray     # (N, 2) int16: hint x, hint z


@dataclass
class NavGrid:
    major: int
    minor: int
    min_pos: tuple[float, float, float]
    max_pos: tuple[float, float, float]
    cell_size: float
    count_x: int
    count_z: int
    cells: np.ndarray                      # structured (count_z, count_x), CELL_DTYPE
    flags: np.ndarray                      # uint16 (count_z, count_x)
    regions: dict[str, np.ndarray] | None = None   # uint8 (count_z, count_x) per REGION_LAYERS
    unknown_block: bytes | None = None
    height_samples: HeightSamples | None = None
    hint_nodes: HintNodes | None = None
    trailing: bytes = b""
    layers_parsed: list[str] = field(default_factory=list)
    report: list[str] = field(default_factory=list)

    @property
    def version(self) -> str:
        return f"{self.major}.{self.minor}"

    @property
    def heights(self) -> np.ndarray:
        return self.cells["center_height"]

    def cell_of(self, x: float, y: float) -> tuple[int, int]:
        """Match-V5 (x, y) -> (ix, iz). May fall outside the grid."""
        return (int(np.floor((x - self.min_pos[0]) / self.cell_size)),
                int(np.floor((y - self.min_pos[2]) / self.cell_size)))

    def cell_center(self, ix: float, iz: float) -> tuple[float, float]:
        return (self.min_pos[0] + (ix + 0.5) * self.cell_size,
                self.min_pos[2] + (iz + 0.5) * self.cell_size)

    def inside(self, ix: int, iz: int) -> bool:
        return 0 <= ix < self.count_x and 0 <= iz < self.count_z


def parse(data: bytes) -> NavGrid:
    """Parse a v7.x navgrid. Truncation inside header, cells or flags is an error;
    a missing or short optional tail is reported in `report` and `layers_parsed`."""
    if len(data) < HEADER.size:
        raise NgridError(f"truncated: {len(data)} bytes, the header needs {HEADER.size}")
    major = data[0]
    if major != SUPPORTED_MAJOR:
        raise NgridError(f"unsupported navgrid version {major} (7.x expected)")
    (major, minor, x0, y0, z0, x1, y1, z1, cell, cx, cz) = HEADER.unpack_from(data, 0)
    if not (cell > 0 and 0 < cx <= 4096 and 0 < cz <= 4096):
        raise NgridError(f"implausible header: cell {cell}, counts {cx} x {cz}")
    n = cx * cz
    report: list[str] = []
    if minor != 1:
        report.append(f"minor version {minor} (7.1 expected); parsed with the 7.1 layout")
    off = HEADER.size
    need = off + n * 48 + n * 2
    if len(data) < need:
        raise NgridError(f"truncated: cells and flags need {need} bytes, file has {len(data)}")
    cells = np.frombuffer(data, dtype=CELL_DTYPE, count=n, offset=off).reshape(cz, cx).copy()
    off += n * 48
    flags = np.frombuffer(data, dtype="<u2", count=n, offset=off).reshape(cz, cx).astype(np.uint16)
    off += n * 2
    g = NavGrid(major, minor, (x0, y0, z0), (x1, y1, z1), cell, cx, cz, cells, flags,
                layers_parsed=["header", "cells", "flags"], report=report)

    rest = len(data) - off
    if rest >= n * 4:
        b = np.frombuffer(data, dtype=np.uint8, count=n * 4, offset=off).reshape(cz, cx, 4)
        g.regions = {
            "river": b[..., 0].copy(),
            "jungle_quadrant": b[..., 1] & 0x0F,
            "main_region": b[..., 1] >> 4,
            "nearest_lane": b[..., 2] & 0x0F,
            "poi": b[..., 2] >> 4,
            "ring": b[..., 3] & 0x0F,
            "srx": b[..., 3] >> 4,
        }
        off += n * 4
        g.layers_parsed.append("regions")
    else:
        report.append(f"region layers missing: {rest} bytes after the flags, {n * 4} needed")
        g.trailing = data[off:]
        return g

    if len(data) - off >= UNKNOWN_BLOCK:
        g.unknown_block = data[off:off + UNKNOWN_BLOCK]
        off += UNKNOWN_BLOCK
        g.layers_parsed.append("unknown_block")
    else:
        report.append("unknown 8x132 block missing")
        g.trailing = data[off:]
        return g

    if len(data) - off >= 16:
        sx, sz, ox, oz = struct.unpack_from("<iiff", data, off)
        if 0 < sx <= 8192 and 0 < sz <= 8192 and len(data) - off - 16 >= sx * sz * 4:
            vals = np.frombuffer(data, dtype="<f4", count=sx * sz, offset=off + 16).reshape(sz, sx).copy()
            g.height_samples = HeightSamples(sx, sz, ox, oz, vals)
            off += 16 + sx * sz * 4
            g.layers_parsed.append("height_samples")
            if (sx, sz) != (2 * cx + 1, 2 * cz + 1):
                report.append(f"height samples {sx} x {sz}, not (2*{cx}+1) x (2*{cz}+1)")
        else:
            report.append(f"height samples header implausible or short ({sx} x {sz})")
            g.trailing = data[off:]
            return g
    else:
        report.append("height samples missing")
        g.trailing = data[off:]
        return g

    rest = len(data) - off
    hn = _hint_count(rest)
    if hn:
        dt = np.dtype([("d", "<f4", (hn,)), ("x", "<i2"), ("z", "<i2")])
        h = np.frombuffer(data, dtype=dt, count=hn, offset=off)
        g.hint_nodes = HintNodes(h["d"].copy(), np.stack([h["x"], h["z"]], axis=1).astype(np.int16))
        off += hn * dt.itemsize
        g.layers_parsed.append("hint_nodes")
        if hn != 900:
            report.append(f"{hn} hint nodes (900 expected on SR)")
    elif rest:
        report.append(f"{rest} bytes after the height samples are not a square hint-node block; kept raw")
    else:
        report.append("hint nodes missing")
    g.trailing = data[off:]
    if g.trailing and hn:
        report.append(f"{len(g.trailing)} unknown trailing bytes kept raw")
    return g


def _hint_count(rest: int) -> int:
    """N with N * (4N + 4) == rest, else 0."""
    if rest <= 0:
        return 0
    n = int((-4 + (16 + 16 * rest) ** 0.5) / 8)
    for k in (n - 1, n, n + 1):
        if k > 0 and k * (4 * k + 4) == rest:
            return k
    return 0


def write(g: NavGrid) -> bytes:
    """Serialise a NavGrid (the inverse of parse). Used for synthetic grids."""
    out = bytearray(HEADER.pack(g.major, g.minor, *g.min_pos, *g.max_pos, g.cell_size, g.count_x, g.count_z))
    out += np.ascontiguousarray(g.cells, dtype=CELL_DTYPE).tobytes()
    out += np.ascontiguousarray(g.flags, dtype="<u2").tobytes()
    if g.regions is not None:
        r = g.regions
        b = np.zeros((g.count_z, g.count_x, 4), np.uint8)
        b[..., 0] = r["river"]
        b[..., 1] = (r["jungle_quadrant"] & 0x0F) | (r["main_region"] << 4)
        b[..., 2] = (r["nearest_lane"] & 0x0F) | (r["poi"] << 4)
        b[..., 3] = (r["ring"] & 0x0F) | (r["srx"] << 4)
        out += b.tobytes()
        if g.unknown_block is not None:
            out += g.unknown_block
            if g.height_samples is not None:
                hs = g.height_samples
                out += struct.pack("<iiff", hs.count_x, hs.count_z, hs.offset_x, hs.offset_z)
                out += np.ascontiguousarray(hs.values, dtype="<f4").tobytes()
                if g.hint_nodes is not None:
                    hn = g.hint_nodes.distances.shape[0]
                    dt = np.dtype([("d", "<f4", (hn,)), ("x", "<i2"), ("z", "<i2")])
                    h = np.zeros(hn, dt)
                    h["d"] = g.hint_nodes.distances
                    h["x"] = g.hint_nodes.coords[:, 0]
                    h["z"] = g.hint_nodes.coords[:, 1]
                    out += h.tobytes()
    out += g.trailing
    return bytes(out)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def flag_histogram(flags: np.ndarray) -> dict[str, int]:
    """Cells per bit (by hex value) plus walkable and blocking counts."""
    f = flags.astype(np.uint32)
    hist = {f"0x{1 << b:04x}": int(((f >> b) & 1).sum()) for b in range(16)}
    hist = {k: v for k, v in hist.items() if v}
    hist["no_bits"] = int((f == 0).sum())
    hist[f"blocking_0x{BLOCKING:04x}"] = int(((f & BLOCKING) != 0).sum())
    hist["cells"] = int(f.size)
    return hist
