# SPDX-License-Identifier: AGPL-3.0-only
"""validate and pc-run on the synthetic grid inside a synthetic WAD."""

from __future__ import annotations

import dataclasses
import json
import struct

import pytest

from navgrid import cli, ngrid, pcrun, structures, validate, wad


def transposed(g: ngrid.NavGrid) -> ngrid.NavGrid:
    """What a parser that swapped x and z would produce."""
    return dataclasses.replace(
        g,
        min_pos=(g.min_pos[2], g.min_pos[1], g.min_pos[0]),
        max_pos=(g.max_pos[2], g.max_pos[1], g.max_pos[0]),
        count_x=g.count_z, count_z=g.count_x,
        cells=g.cells.T.copy(), flags=g.flags.T.copy(),
        regions={k: v.T.copy() for k, v in g.regions.items()},
    )


def test_passes_on_synthetic(synth_bytes):
    r = validate.validate(ngrid.parse(synth_bytes))
    assert r["pass"], {k: v for k, v in r["structures"].items() if not v["pass"]}
    assert r["structures_total"] == 28 and r["structures_pass"] == 28
    assert r["lane_regions_checked"] is True
    assert all(r["fountains_walkable"].values())
    assert r["structure_wall_hypothesis"]["supported"]
    assert r["structure_wall_hypothesis"]["share_near"] == 1.0
    assert r["flag_histogram"]["cells"] == 295 * 296


@pytest.mark.parametrize("dx,dz", [(3, 0), (0, 3), (-3, 0), (0, -3)])
def test_fails_when_shifted_three_cells(synth_bytes, dx, dz):
    data = bytearray(synth_bytes)
    x0, _, z0 = struct.unpack_from("<3f", data, 3)
    struct.pack_into("<f", data, 3, x0 + 50 * dx)
    struct.pack_into("<f", data, 11, z0 + 50 * dz)
    r = validate.validate(ngrid.parse(bytes(data)))
    assert not r["pass"]
    assert r["structures_pass"] < 28 // 2


def test_fails_on_flipped_axis(synth_bytes):
    r = validate.validate(transposed(ngrid.parse(synth_bytes)))
    assert not r["pass"]
    lanes = [v["lane_match"] for v in r["structures"].values() if v["lane_match"] is not None]
    assert lanes.count(False) >= 8


def test_lane_check_skipped_without_regions(synth_bytes):
    core = 39 + 50 * 295 * 296
    r = validate.validate(ngrid.parse(synth_bytes[:core]))
    assert r["lane_regions_checked"] is False
    assert r["pass"]


def test_output_is_numbers_only(synth_bytes):
    r = validate.validate(ngrid.parse(synth_bytes))
    text = json.dumps(r)
    assert "polygon" not in text.lower() and "coordinates" not in text
    for row in r["structures"].values():
        assert set(row) == {"in_grid", "cluster_cells", "bit4_cells", "nearest_blocking_cells",
                            "within", "lane_match", "pass"}


def test_timeline_positions(synth_file, tmp_path):
    x, y = structures.TURRETS["blue_bot_outer"]
    tl = {"metadata": {"participants": ["X"]}, "info": {"frames": [{"events": [
        {"type": "BUILDING_KILL", "position": {"x": x, "y": y}, "killerId": 3},
        {"type": "TURRET_PLATE_DESTROYED", "position": {"x": x, "y": y}},
        {"type": "WARD_PLACED", "creatorId": 1},
    ]}]}}
    p = tmp_path / "timeline.json"
    p.write_text(json.dumps(tl))
    r = validate.validate_file(synth_file, p)
    t = r["timeline"]
    assert (t["events"], t["building_kills"], t["building_kills_within"], t["plates"]) == (2, 1, 1, 1)
    assert t["rows"][0]["nearest_table_structure_units"] == 0.0


def test_validate_cli_exit_codes(synth_file, tmp_path, capsys):
    assert cli.main(["validate", str(synth_file)]) == 0
    assert json.loads(capsys.readouterr().out)["pass"] is True
    bad = tmp_path / "shifted.aimesh_ngrid"
    data = bytearray(synth_file.read_bytes())
    struct.pack_into("<f", data, 3, struct.unpack_from("<f", data, 3)[0] + 150)
    bad.write_bytes(bytes(data))
    assert cli.main(["validate", str(bad)]) == 1


# -- pc-run -------------------------------------------------------------------

@pytest.fixture
def synth_wad(tmp_path, synth_bytes):
    p = tmp_path / "Riot Games" / "Map11.wad.client"
    p.parent.mkdir()
    p.write_bytes(wad.write_wad([
        (wad.path_hash("assets/maps/info.bin"), b"decoy" * 1000, wad.TYPE_ZSTD),
        (wad.CANDIDATE_PATHS[0][1], synth_bytes, wad.TYPE_ZSTD_CHUNKED),
    ]))
    return p


def test_pc_run_writes_latest(synth_wad, synth_bytes, tmp_path, monkeypatch):
    local = tmp_path / "localappdata"
    monkeypatch.setenv("LOCALAPPDATA", str(local))
    lines = []
    out = pcrun.run(str(synth_wad), is_running=lambda: False, echo=lines.append)
    assert out == local / "navgrid" / "latest"
    assert sorted(p.name for p in out.iterdir()) == [
        "aipath_srx_2.aimesh_ngrid", "manifest.json", "validation.json"]
    assert (out / "aipath_srx_2.aimesh_ngrid").read_bytes() == synth_bytes
    m = json.loads((out / "manifest.json").read_text())
    assert m["find_method"] == "candidate-path"
    assert m["entry"]["path"] == wad.CANDIDATE_PATHS[0][0]
    assert m["entry"]["type"] == "zstd-chunked"
    assert m["grid"]["sha256"] == ngrid.sha256(synth_bytes)
    assert len(m["wad"]["sha256"]) == 64 and m["wad"]["mtime_utc"]
    assert json.loads((out / "validation.json").read_text())["pass"] is True
    assert len(lines) <= 9 and lines[-1] == pcrun.SCP_LINE
    assert lines[-1] == ('scp -r "$env:LOCALAPPDATA\\navgrid\\latest" '
                         'ops@10.44.0.1:.local/share/navgrid/inbox/')
    # a second run keeps one previous copy
    pcrun.run(str(synth_wad), is_running=lambda: False, echo=lambda s: None)
    assert (local / "navgrid" / "previous" / "manifest.json").is_file()


def test_pc_run_refuses_while_game_runs(synth_wad, tmp_path, monkeypatch):
    local = tmp_path / "localappdata"
    monkeypatch.setenv("LOCALAPPDATA", str(local))
    with pytest.raises(pcrun.RunRefused, match="League of Legends.exe is running"):
        pcrun.run(str(synth_wad), is_running=lambda: True)
    assert not local.exists()


def test_tasklist_is_parsed(monkeypatch):
    class Res:
        stdout = '"League of Legends.exe","1234","Console","1","2,000,000 K"\n'

    monkeypatch.setattr(pcrun.os, "name", "nt")
    monkeypatch.setattr(pcrun.subprocess, "run", lambda *a, **k: Res())
    assert pcrun.game_running() is True
    Res.stdout = "INFO: No tasks are running which match the specified criteria.\n"
    assert pcrun.game_running() is False


def test_cli_pc_run_refuses_while_game_runs(synth_wad, monkeypatch, capsys):
    monkeypatch.setattr(pcrun, "game_running", lambda: True)
    assert cli.main(["pc-run", "--wad", str(synth_wad)]) == 2
    assert "is running" in capsys.readouterr().err


def test_pc_run_refuses_out_inside_git_worktree(synth_wad, tmp_path):
    repo = tmp_path / "repo"
    (repo / ".git").mkdir(parents=True)
    with pytest.raises(pcrun.RunRefused, match="git work tree"):
        pcrun.run(str(synth_wad), repo / "out", is_running=lambda: False)


def test_default_roots_cover_both_layouts():
    paths = [str(p) for p in pcrun.default_wad_paths()]
    assert "C:\\Riot Games\\League of Legends\\Game\\DATA\\FINAL\\Maps\\Shipping\\Map11.wad.client" in paths
    assert "C:\\Riot Games\\League of Legends\\DATA\\FINAL\\Maps\\Shipping\\Map11.wad.client" in paths
    assert all(p.endswith("Map11.wad.client") for p in paths)


def test_missing_wad_refused(tmp_path):
    with pytest.raises(pcrun.RunRefused, match="not a file"):
        pcrun.run(str(tmp_path / "nope.wad.client"), tmp_path / "o", is_running=lambda: False)

