# SPDX-License-Identifier: AGPL-3.0-only
"""Guards: no Riot-derived file tracked, no network, no real home directory."""

from __future__ import annotations

import os
import socket
import subprocess
from pathlib import Path

import pytest

TOOL = Path(__file__).resolve().parents[1]
SYNTHETIC = "tools/navgrid/tests/fixtures/synthetic/"


def _tracked() -> list[str]:
    top = subprocess.run(["git", "-C", str(TOOL), "rev-parse", "--show-toplevel"],
                         capture_output=True, text=True)
    if top.returncode != 0:
        pytest.skip("not inside a git checkout")
    res = subprocess.run(["git", "-C", top.stdout.strip(), "ls-files", "-z"],
                         capture_output=True, text=True, check=True)
    return [p for p in res.stdout.split("\0") if p]


def test_no_riot_derived_files():
    bad = []
    for p in _tracked():
        low = p.lower()
        name = low.rsplit("/", 1)[-1]
        if low.endswith(".aimesh_ngrid") or ".wad" in name and (name.endswith(".wad") or ".wad." in name):
            bad.append(p)
        elif low.startswith("tools/navgrid/") and low.endswith((".geojson", ".png")) \
                and not low.startswith(SYNTHETIC):
            bad.append(p)
    assert bad == [], f"Riot-derived or unvetted files tracked: {bad}"


def test_gitignore_covers_navgrid_artefacts():
    top = subprocess.run(["git", "-C", str(TOOL), "rev-parse", "--show-toplevel"],
                         capture_output=True, text=True)
    if top.returncode != 0:
        pytest.skip("not inside a git checkout")
    root = top.stdout.strip()
    probes = ["tools/navgrid/x.aimesh_ngrid", "a/Map11.wad.client", "tools/navgrid/out/a.json",
              "tools/navgrid/layer.geojson", "docs/layer.geojson", "tools/navgrid/.venv/x"]
    res = subprocess.run(["git", "-C", root, "check-ignore", "--no-index", *probes],
                         capture_output=True, text=True)
    assert sorted(res.stdout.split()) == sorted(probes)
    ok = subprocess.run(["git", "-C", root, "check-ignore", "--no-index", SYNTHETIC + "a.geojson"],
                        capture_output=True, text=True)
    assert ok.returncode == 1  # not ignored


def test_network_is_blocked():
    from conftest import NetworkBlocked

    with pytest.raises(NetworkBlocked):
        socket.create_connection(("192.0.2.1", 443), timeout=0.1)
    with pytest.raises(NetworkBlocked):
        socket.getaddrinfo("example.com", 443)


def test_home_and_localappdata_redirected(tmp_path_factory):
    base = str(tmp_path_factory.getbasetemp())
    assert os.environ["HOME"].startswith(base)
    assert os.environ["LOCALAPPDATA"].startswith(base)
    assert str(Path.home()).startswith(base)


def test_pc_run_needs_no_vector_extra():
    """pc-run and validate must work on a PC without shapely installed."""
    import sys

    code = ("import sys; import navgrid.cli, navgrid.pcrun, navgrid.validate; "
            "assert 'shapely' not in sys.modules, 'shapely imported'")
    res = subprocess.run([sys.executable, "-c", code], cwd=str(TOOL), capture_output=True, text=True)
    assert res.returncode == 0, res.stderr
