# SPDX-License-Identifier: AGPL-3.0-only
"""WAD v3 reader on synthetic archives written by the test."""

from __future__ import annotations

import struct

import pytest

from navgrid import wad


def fake_grid(cx=295, cz=296, size=2_100_000) -> bytes:
    """A v7.1 header with 50-unit cells, padded with a deterministic pattern."""
    head = struct.pack("<BH3f3ffii", 7, 1, -100.0, 0.0, -150.0, 14650.0, 0.0, 14650.0, 50.0, cx, cz)
    body = bytes((i * 7) & 0xFF for i in range(256)) * ((size - len(head)) // 256 + 1)
    return head + body[: size - len(head)]


DECOY = b"PROP" + bytes(range(256)) * 9000  # another entry, not a grid


def test_candidate_hashes_match_paths():
    for p, h in wad.CANDIDATE_PATHS:
        assert wad.path_hash(p) == h
        assert wad.path_hash(p.upper().replace("/", "\\")) == h


@pytest.mark.parametrize("t", [wad.TYPE_RAW, wad.TYPE_GZIP, wad.TYPE_ZSTD, wad.TYPE_ZSTD_CHUNKED])
def test_finds_grid_by_candidate_path(tmp_path, t):
    grid = fake_grid()
    h = wad.CANDIDATE_PATHS[0][1]
    blob = wad.write_wad([(wad.path_hash("assets/decoy.bin"), DECOY, wad.TYPE_ZSTD), (h, grid, t)])
    p = tmp_path / "Map11.wad.client"
    p.write_bytes(blob)
    arc, sha, handle = wad.open_wad(p)
    try:
        assert arc.version == "3.4"
        found = arc.find_navgrid()
        assert found.method == "candidate-path"
        assert found.path == wad.CANDIDATE_PATHS[0][0]
        assert found.data == grid
        assert found.entry.type_name == wad.TYPE_NAMES[t]
        decoy = arc.by_hash[wad.path_hash("assets/decoy.bin")]
        assert arc.read(decoy) == DECOY
    finally:
        wad.close_wad(handle)
    assert len(sha) == 64


def test_later_candidate_variant_is_found(tmp_path):
    grid = fake_grid()
    blob = wad.write_wad([(wad.path_hash("assets/maps/navgrid/map11/aipath_srx_3.aimesh_ngrid"), grid, wad.TYPE_ZSTD)])
    p = tmp_path / "w.wad.client"
    p.write_bytes(blob)
    arc, _, handle = wad.open_wad(p)
    try:
        f = arc.find_navgrid()
        assert f.path.endswith("aipath_srx_3.aimesh_ngrid")
    finally:
        wad.close_wad(handle)


@pytest.mark.parametrize("t", [wad.TYPE_RAW, wad.TYPE_GZIP, wad.TYPE_ZSTD, wad.TYPE_ZSTD_CHUNKED])
def test_content_sniff_fallback(tmp_path, t):
    grid = fake_grid()
    bad_cell = bytearray(fake_grid())
    struct.pack_into("<f", bad_cell, 27, 25.0)  # wrong cell size: a decoy
    small = fake_grid(cx=120, cz=120)            # counts out of range: a decoy
    blob = wad.write_wad([
        (0x1111, DECOY, wad.TYPE_RAW),
        (0x2222, bytes(bad_cell), t),
        (0x3333, small, t),
        (0x4444, grid, t),
    ])
    p = tmp_path / "w.wad.client"
    p.write_bytes(blob)
    arc, _, handle = wad.open_wad(p)
    try:
        f = arc.find_navgrid()
        assert f.method == "content-sniff"
        assert f.path is None
        assert f.entry.path_hash == 0x4444
        assert f.sniffed == ["0000000000004444"]
        assert f.data == grid
    finally:
        wad.close_wad(handle)


def test_no_grid_is_an_error(tmp_path):
    p = tmp_path / "w.wad.client"
    p.write_bytes(wad.write_wad([(0x1, DECOY, wad.TYPE_ZSTD)]))
    arc, _, handle = wad.open_wad(p)
    try:
        with pytest.raises(wad.WadError, match="no navgrid"):
            arc.find_navgrid()
    finally:
        wad.close_wad(handle)


def test_refuses_other_versions_and_truncation(tmp_path):
    good = wad.write_wad([(0x1, DECOY, wad.TYPE_RAW)])
    with pytest.raises(wad.WadError, match="magic"):
        wad.WadArchive(b"XX" + good[2:])
    with pytest.raises(wad.WadError, match="version 2"):
        wad.WadArchive(b"RW\x02\x00" + good[4:])
    with pytest.raises(wad.WadError, match="table of contents"):
        wad.WadArchive(good[:280])
    arc = wad.WadArchive(good[:400])
    with pytest.raises(wad.WadError, match="past the end"):
        arc.read(arc.entries[0])


def test_size_mismatch_is_an_error():
    blob = bytearray(wad.write_wad([(0x1, DECOY, wad.TYPE_RAW)]))
    struct.pack_into("<I", blob, wad.HEADER_V3 + 16, len(DECOY) + 1)  # uncompressed_size
    blob[wad.HEADER_V3 + 12:wad.HEADER_V3 + 16] = struct.pack("<I", len(DECOY))
    arc = wad.WadArchive(bytes(blob))
    with pytest.raises(wad.WadError, match="TOC says"):
        arc.read(arc.entries[0])
