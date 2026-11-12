# SPDX-License-Identifier: AGPL-3.0-only
"""Shared fixtures.

Autouse guards for every test: no socket except to 127.0.0.1 (and no DNS),
and HOME / USERPROFILE / LOCALAPPDATA point at temporary directories so no
test can read or write the real user's files.
"""

from __future__ import annotations

import socket
import sys
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
LOOPBACK = "127.0.0.1"


class NetworkBlocked(RuntimeError):
    pass


def _host_of(address) -> str | None:
    if isinstance(address, tuple) and address:
        return str(address[0])
    return None  # AF_UNIX paths


@pytest.fixture(autouse=True)
def block_network(monkeypatch):
    real_connect = socket.socket.connect
    real_connect_ex = socket.socket.connect_ex
    real_create = socket.create_connection
    real_getaddrinfo = socket.getaddrinfo

    def check(address):
        host = _host_of(address)
        if host is not None and host != LOOPBACK:
            raise NetworkBlocked(f"test tried to reach {address}")

    def connect(self, address):
        check(address)
        return real_connect(self, address)

    def connect_ex(self, address):
        check(address)
        return real_connect_ex(self, address)

    def create_connection(address, *a, **kw):
        check(address)
        return real_create(address, *a, **kw)

    def getaddrinfo(host, *a, **kw):
        if host is not None and str(host) != LOOPBACK:
            raise NetworkBlocked(f"test tried to resolve {host}")
        return real_getaddrinfo(host, *a, **kw)

    monkeypatch.setattr(socket.socket, "connect", connect)
    monkeypatch.setattr(socket.socket, "connect_ex", connect_ex)
    monkeypatch.setattr(socket, "create_connection", create_connection)
    monkeypatch.setattr(socket, "getaddrinfo", getaddrinfo)
    yield


@pytest.fixture(autouse=True)
def redirect_home(monkeypatch, tmp_path_factory):
    home = tmp_path_factory.mktemp("home")
    local = tmp_path_factory.mktemp("localappdata")
    monkeypatch.setenv("HOME", str(home))
    monkeypatch.setenv("USERPROFILE", str(home))
    monkeypatch.setenv("LOCALAPPDATA", str(local))
    yield home


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
