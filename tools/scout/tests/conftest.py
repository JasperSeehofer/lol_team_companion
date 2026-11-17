"""Shared fixtures. An autouse guard refuses every socket connection that is
not to the loopback address, so no test can reach the network."""

from __future__ import annotations

import socket
import sys
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "fixtures"))
FIXTURES = HERE / "fixtures"
LOOPBACK = {"127.0.0.1", "localhost", "::1"}


class NetworkBlocked(RuntimeError):
    pass


def _host_of(address) -> str | None:
    if isinstance(address, tuple) and address:
        return str(address[0])
    return None  # AF_UNIX paths and the like


@pytest.fixture(autouse=True)
def block_network(monkeypatch):
    real_connect = socket.socket.connect
    real_connect_ex = socket.socket.connect_ex
    real_create = socket.create_connection
    real_getaddrinfo = socket.getaddrinfo

    def check(address):
        host = _host_of(address)
        if host is not None and host not in LOOPBACK:
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

    def getaddrinfo(host, *a, **kw):   # no DNS lookups either
        if host is not None and str(host) not in LOOPBACK:
            raise NetworkBlocked(f"test tried to resolve {host}")
        return real_getaddrinfo(host, *a, **kw)

    monkeypatch.setattr(socket.socket, "connect", connect)
    monkeypatch.setattr(socket, "getaddrinfo", getaddrinfo)
    monkeypatch.setattr(socket.socket, "connect_ex", connect_ex)
    monkeypatch.setattr(socket, "create_connection", create_connection)
    yield


@pytest.fixture
def fixtures() -> Path:
    return FIXTURES


@pytest.fixture
def mock_api():
    from mock_replay_api import MockReplayApi

    with MockReplayApi() as api:
        yield api
