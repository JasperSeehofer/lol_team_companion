# SPDX-License-Identifier: AGPL-3.0-only
"""ngrid parser and writer on the synthetic grid only."""

from __future__ import annotations

import struct

import numpy as np
import pytest

from navgrid import ngrid, structures, synth


def test_round_trip(synth_bytes):
    g = ngrid.parse(synth_bytes)
    assert g.version == "7.1"
    assert (g.count_x, g.count_z, g.cell_size) == (295, 296, 50.0)
    assert g.layers_parsed == ["header", "cells", "flags", "regions", "unknown_block",
                               "height_samples", "hint_nodes"]
    assert g.report == []
    assert g.trailing == b""
    assert g.hint_nodes.distances.shape == (900, 900)
    assert (g.height_samples.count_x, g.height_samples.count_z) == (591, 593)
    assert ngrid.write(g) == synth_bytes


def test_layers_hold_what_synth_drew(synth_bytes):
    g = ngrid.parse(synth_bytes)
    ix, iz = g.cell_of(6400, 2600)  # inside the brush patch
    assert g.flags[iz, ix] == ngrid.BRUSH
    ix, iz = g.cell_of(*structures.TURRETS["red_bot_outer"])
    assert g.flags[iz, ix] & ngrid.STRUCTURE_WALL
    assert g.regions["nearest_lane"][iz, ix] in ngrid.lane_code("red", "bot")
    ix, iz = g.cell_of(*synth.CAMP[:2])
    assert ngrid.POI[int(g.regions["poi"][iz, ix])] == "CampRedBuff"
    ix, iz = g.cell_of(7350, 7350)  # on the river line
    assert g.regions["river"][iz, ix] & 16
    assert (g.flags & ngrid.TRANSPARENT_WALL).any()


def test_axis_convention(synth_bytes):
    """Grid x is Match-V5 x (columns), grid z is Match-V5 y (rows)."""
    g = ngrid.parse(synth_bytes)
    # the second wall block is narrow on x (600) and tall on z (1400)
    x0, x1, z0, z1 = synth.WALL_BLOCKS[2]
    ix0, iz0 = g.cell_of(x0 + 1, z0 + 1)
    ix1, iz1 = g.cell_of(x1 - 1, z1 - 1)
    block = g.flags[iz0:iz1 + 1, ix0:ix1 + 1]
    assert block.shape == (28, 12)
    assert (block & ngrid.WALL).all()
    assert g.cell_center(0, 0) == (synth.MIN_X + 25, synth.MIN_Z + 25)
    # x and z are not interchangeable on this map
    assert not np.array_equal(g.flags[:295, :295], g.flags[:295, :295].T)


def test_wrong_version_refused(synth_bytes):
    with pytest.raises(ngrid.NgridError, match="version 5"):
        ngrid.parse(bytes([5]) + synth_bytes[1:])


@pytest.mark.parametrize("cut", [10, 39 + 48 * 100, 39 + 48 * 295 * 296 + 7])
def test_truncated_core_refused(synth_bytes, cut):
    with pytest.raises(ngrid.NgridError, match="truncated"):
        ngrid.parse(synth_bytes[:cut])


def test_missing_tail_reported(synth_bytes):
    n = 295 * 296
    core = 39 + 50 * n
    g = ngrid.parse(synth_bytes[:core])
    assert g.layers_parsed == ["header", "cells", "flags"]
    assert any("region layers missing" in r for r in g.report)
    g = ngrid.parse(synth_bytes[:core + 4 * n + 100])
    assert "regions" in g.layers_parsed and "unknown_block" not in g.layers_parsed
    assert len(g.trailing) == 100
    g = ngrid.parse(synth_bytes[:-5])  # hint block short
    assert "hint_nodes" not in g.layers_parsed
    assert any("not a square hint-node block" in r for r in g.report)


def test_unknown_trailing_bytes_kept(synth_bytes):
    g = ngrid.parse(synth_bytes + b"EXTRA-LAYER")
    # 11 bytes do not fit a square hint block, so the whole tail is kept raw
    assert g.trailing.endswith(b"EXTRA-LAYER")
    assert ngrid.write(g) == synth_bytes + b"EXTRA-LAYER"


def test_other_minor_parses_with_a_note(synth_bytes):
    data = bytearray(synth_bytes)
    struct.pack_into("<H", data, 1, 2)
    g = ngrid.parse(bytes(data))
    assert g.minor == 2 and any("minor version 2" in r for r in g.report)


def test_flag_table_carries_both_readings():
    bits = {row["bit"]: row for row in ngrid.FLAG_TABLE}
    assert bits[0x04]["plan"] == "structure wall" and bits[0x04]["mapgeo"] == "BUSY"
    assert bits[0x04]["converter"] == "StructureWall"
    assert bits[0x40]["plan"] == "transparent wall" and bits[0x40]["mapgeo"] == "SEE_THROUGH"
    assert set(ngrid.FLAG_SOURCES) == {"plan", "mapgeo", "converter", "settled"}
    # R2 settled some bits on the real file; the other readings stay listed
    for bit in (0x01, 0x02, 0x04, 0x40):
        assert "settled on 16.21 real file, R2" in bits[bit]["settled"]
    assert bits[0x80]["settled"] is None
    assert ngrid.BLOCKING == 0x06 and ngrid.BLOCKING_R1 == 0x46


def test_histogram(synth_bytes):
    h = ngrid.flag_histogram(ngrid.parse(synth_bytes).flags)
    assert h["cells"] == 295 * 296
    assert h["0x0004"] == 9 * 28
