# SPDX-License-Identifier: AGPL-3.0-only
"""Shared fixtures."""

from __future__ import annotations

import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

import pytest  # noqa: E402


@pytest.fixture(scope="session")
def synth_bytes() -> bytes:
    """The fabricated v7.1 grid (navgrid.synth), built once per session in memory."""
    from navgrid import synth

    return synth.make_bytes()


@pytest.fixture
def synth_file(tmp_path, synth_bytes):
    p = tmp_path / "synthetic.aimesh_ngrid"
    p.write_bytes(synth_bytes)
    return p
