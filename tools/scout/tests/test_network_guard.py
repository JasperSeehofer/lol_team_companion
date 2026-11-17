import socket

import pytest

from conftest import NetworkBlocked


def test_outside_connections_are_blocked():
    with pytest.raises(NetworkBlocked):
        socket.create_connection(("ddragon.leagueoflegends.com", 443), timeout=1)
    s = socket.socket()
    try:
        with pytest.raises(NetworkBlocked):
            s.connect(("10.255.255.1", 80))
    finally:
        s.close()


def test_requests_to_riot_are_blocked():
    import requests

    with pytest.raises(Exception) as err:
        requests.get("https://europe.api.riotgames.com/", timeout=1)
    assert "NetworkBlocked" in repr(err.value) or "resolve" in str(err.value)
