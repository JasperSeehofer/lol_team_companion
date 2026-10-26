# SPDX-License-Identifier: AGPL-3.0-only
"""vectorise on the synthetic grid only; output in temp dirs."""

from __future__ import annotations

import json

import numpy as np
import pytest

shapely = pytest.importorskip("shapely")
from shapely import box, hausdorff_distance, union_all  # noqa: E402
from shapely.geometry import MultiPolygon, shape  # noqa: E402

from navgrid import cli, ngrid, vectorise  # noqa: E402


@pytest.fixture(scope="module")
def out_dir(tmp_path_factory, synth_bytes):
    d = tmp_path_factory.mktemp("vec")
    grid = d / "g.aimesh_ngrid"
    grid.write_bytes(synth_bytes)
    return vectorise.run(grid, d / "out")


def load(out_dir, name):
    fc = json.loads((out_dir / f"{name}.geojson").read_text())
    return fc, [shape(f["geometry"]) for f in fc["features"]]


def raw_cells(g, mask):
    zz, xx = np.nonzero(mask)
    x0 = g.min_pos[0] + xx * g.cell_size
    z0 = g.min_pos[2] + zz * g.cell_size
    return union_all(box(x0, z0, x0 + g.cell_size, z0 + g.cell_size))


def test_layers_and_properties(out_dir):
    names = sorted(p.name for p in out_dir.glob("*.geojson"))
    layers = ["brush", "camps", "gates", "lanes", "pits", "river", "structures", "transparent", "walkable",
              "wall"]
    assert names == sorted(f"{lay}.{lev}.geojson" for lay in layers for lev in ("macro", "vision"))
    fc, _ = load(out_dir, "brush.vision")
    p = fc["properties"]
    assert p["derived"] == "riot-navgrid; do not ship before ticket 2"
    assert (p["layer"], p["level"], p["tolerance"]) == ("brush", "vision", 12.0)
    assert len(p["grid_sha256"]) == 64 and p["generator"].startswith("navgrid ")
    assert fc["features"][0]["properties"]["derived"] == p["derived"]


def test_polygons_valid_and_closed(out_dir):
    for f in out_dir.glob("*.geojson"):
        fc = json.loads(f.read_text())
        for feat in fc["features"]:
            rings = feat["geometry"]["coordinates"]
            for r in rings:
                assert r[0] == r[-1] and len(r) >= 4
            assert shape(feat["geometry"]).is_valid, f.name


def test_holes_preserved(out_dir):
    _, walk = load(out_dir, "walkable.vision")
    assert max(len(p.interiors) for p in walk) >= 28 + 3  # 28 structure blobs + terrain blocks
    _, wall = load(out_dir, "wall.macro")
    assert any(len(p.interiors) >= 1 for p in wall)      # the border wall rings the map


@pytest.mark.parametrize("layer", ["walkable", "wall", "brush", "river", "lanes"])
def test_vision_hausdorff_within_one_cell(out_dir, synth_bytes, layer):
    g = ngrid.parse(synth_bytes)
    raw = raw_cells(g, vectorise.rasters(g)[layer])
    _, polys = load(out_dir, f"{layer}.vision")
    vec = MultiPolygon(polys)
    assert hausdorff_distance(vec.boundary, raw.boundary) <= 50.0


def test_brush_area_within_five_percent(out_dir, synth_bytes):
    g = ngrid.parse(synth_bytes)
    cells = int(vectorise.rasters(g)["brush"].sum())
    for level in ("vision", "macro"):
        _, polys = load(out_dir, f"brush.{level}")
        area = sum(p.area for p in polys)
        assert abs(area - cells * 2500.0) <= 0.05 * cells * 2500.0


def test_deterministic(tmp_path, synth_bytes, out_dir):
    grid = tmp_path / "g.aimesh_ngrid"
    grid.write_bytes(synth_bytes)
    again = vectorise.run(grid, tmp_path / "again")
    for f in sorted(out_dir.iterdir()):
        assert (again / f.name).read_bytes() == f.read_bytes(), f.name


def test_refuses_out_inside_git_worktree(tmp_path, synth_file):
    (tmp_path / "repo" / ".git").mkdir(parents=True)
    with pytest.raises(vectorise.VectoriseRefused, match="git work tree"):
        vectorise.run(synth_file, tmp_path / "repo" / "layers")
    assert cli.main(["vectorise", str(synth_file), "--out", str(tmp_path / "repo" / "x")]) == 2


def test_default_out_under_home(synth_file, redirect_home, synth_bytes):
    out = vectorise.run(synth_file)
    assert out == redirect_home / ".local" / "share" / "navgrid" / "out" / ngrid.sha256(synth_bytes)[:8]
    assert (out / "vectorise.json").is_file()


def test_marching_squares_square_and_hole():
    F = np.zeros((8, 8))
    F[2:6, 2:6] = 1.0
    F[3, 3] = 0.0
    rings = vectorise.marching_squares(F)
    assert len(rings) == 2
    for r in rings:
        assert (r[0] == r[-1]).all()


def test_base_gates_are_walkable_not_walls(synth_bytes):
    """16.21 encodes base gates as 0x0400/0x0800 | 0x40 without 0x02 (M166 R2):
    0x40 alone must not make a cell a wall (R1's 0x46 rule did)."""
    g = ngrid.parse(synth_bytes)
    r = vectorise.rasters(g)
    gate = (g.flags & (ngrid.BLUE_ONLY | ngrid.RED_ONLY)) != 0
    assert gate.sum() > 0 and (g.flags[gate] & ngrid.TRANSPARENT_WALL).all()
    assert r["walkable"][gate].all() and not r["wall"][gate].any()
    # the settled rule equals "not 0x02" when every 0x04 cell also has 0x02
    assert np.array_equal(r["walkable"], (g.flags & ngrid.WALL) == 0)


def test_transparent_layer_excludes_structures_and_gates(synth_bytes):
    """16.21 sets 0x40 on structure and gate cells too (M166 R2): neither is a
    transparent wall; structures get their own layer."""
    g = ngrid.parse(synth_bytes)
    r = vectorise.rasters(g)
    struct = (g.flags & ngrid.STRUCTURE_WALL) != 0
    gate = (g.flags & (ngrid.BLUE_ONLY | ngrid.RED_ONLY)) != 0
    assert (g.flags[struct] & ngrid.TRANSPARENT_WALL).all() and (g.flags[gate] & ngrid.TRANSPARENT_WALL).all()
    assert r["transparent"].any()
    assert not r["transparent"][struct].any() and not r["transparent"][gate].any()
    assert np.array_equal(r["transparent"], g.regions["srx"] == 2)
    assert np.array_equal(r["structures"], struct)
