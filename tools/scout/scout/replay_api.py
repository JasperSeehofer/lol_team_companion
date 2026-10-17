"""Client for the League client's local Replay API.

Field names follow Riot's League Director (Apache-2.0) api.py: GET reads a
resource, POST updates the fields sent. The live API serves HTTPS on
127.0.0.1:2999 with a self-signed certificate; certificate checks are off only
for a loopback host. Tests point the client at a plain-HTTP mock.
"""

from __future__ import annotations

from urllib.parse import urlparse

import requests

DEFAULT_BASE = "https://127.0.0.1:2999"
LOOPBACK = {"127.0.0.1", "localhost", "::1"}


class ReplayApiError(RuntimeError):
    pass


class ReplayClient:
    def __init__(self, base: str = DEFAULT_BASE, timeout: float = 5.0):
        self.base = base.rstrip("/")
        self.timeout = timeout
        host = urlparse(self.base).hostname or ""
        # Self-signed certificate: skip verification only on this machine.
        self.verify = host not in LOOPBACK
        if not self.verify:
            import urllib3

            urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)
        self.session = requests.Session()

    def _call(self, method: str, path: str, body: dict | None = None):
        url = self.base + path
        try:
            r = self.session.request(method, url, json=body, timeout=self.timeout,
                                     verify=self.verify)
        except requests.RequestException as e:
            raise ReplayApiError(f"{method} {path}: {e.__class__.__name__}: {e}") from e
        if r.status_code >= 400:
            raise ReplayApiError(f"{method} {path}: HTTP {r.status_code} {r.text[:200]}")
        try:
            return r.json()
        except ValueError:
            return {"_text": r.text}

    def get(self, path: str):
        return self._call("GET", path)

    def post(self, path: str, body: dict):
        return self._call("POST", path, body)

    def status(self, path: str) -> int:
        """HTTP status of a GET (0 when the request failed), for spec discovery."""
        try:
            r = self.session.get(self.base + path, timeout=self.timeout, verify=self.verify)
            return r.status_code
        except requests.RequestException:
            return 0

    # Convenience wrappers
    def game(self):
        return self.get("/replay/game")

    def playback(self):
        return self.get("/replay/playback")

    def set_playback(self, **fields):
        return self.post("/replay/playback", fields)

    def render(self):
        return self.get("/replay/render")

    def set_render(self, **fields):
        return self.post("/replay/render", fields)

    def recording(self):
        return self.get("/replay/recording")

    def set_recording(self, **fields):
        return self.post("/replay/recording", fields)
