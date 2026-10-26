"""A mock of the League client's Replay API for tests: plain HTTP on
127.0.0.1, ephemeral port, in a thread. Quirks are configurable so the probe
has something to find: speed clamped, the minimap hidden with the rest of the
interface, camera height capped, one recording per session."""

from __future__ import annotations

import json
import threading
import time
from dataclasses import dataclass, field
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import numpy as np


@dataclass
class Quirks:
    max_speed: float = 8.0
    ignore_minimap: bool = True        # interfaceAll=false hides the minimap too
    max_camera_y: float = 25000.0
    spec_path: str | None = "/swagger/v3/openapi.json"
    one_recording: bool = True
    time_scale: float = 0.25           # wall seconds per game second while recording


@dataclass
class State:
    quirks: Quirks = field(default_factory=Quirks)
    render: dict = field(default_factory=lambda: {
        "interfaceAll": True, "interfaceMinimap": True, "interfaceAnnounce": True,
        "interfaceChat": True, "interfaceFrames": True, "interfaceKillCallouts": True,
        "interfaceNeutralTimers": True, "interfaceQuests": True, "interfaceReplay": True,
        "interfaceScore": True, "interfaceScoreboard": True, "interfaceTarget": True,
        "interfaceTimeline": True, "fogOfWar": True, "floatingText": True,
        "healthBarChampions": True, "healthBarStructures": True, "healthBarWards": True,
        "healthBarPets": True, "healthBarMinions": True, "minions": True, "particles": True,
        "cameraMode": "top", "cameraAttached": False, "fieldOfView": 40.0, "farClip": 30000.0,
        "cameraPosition": {"x": 7000.0, "y": 1900.0, "z": 7000.0},
        "cameraRotation": {"x": 0.0, "y": 60.0, "z": 0.0}})
    base_time: float = 0.0
    base_wall: float = field(default_factory=time.time)
    speed: float = 1.0
    paused: bool = True
    length: float = 1800.0
    recording: dict = field(default_factory=lambda: {"recording": False})
    recordings_started: int = 0
    posts: list = field(default_factory=list)
    lock: threading.Lock = field(default_factory=threading.Lock)

    def now(self) -> float:
        if self.paused:
            return self.base_time
        return min(self.length, self.base_time + (time.time() - self.base_wall) * self.speed)

    def playback(self) -> dict:
        return {"paused": self.paused, "seeking": False, "time": self.now(),
                "speed": self.speed, "length": self.length}

    def post_playback(self, body: dict) -> None:
        t = self.now()
        if "time" in body:
            t = float(body["time"])
        if "speed" in body:
            self.speed = min(float(body["speed"]), self.quirks.max_speed)
        if "paused" in body:
            self.paused = bool(body["paused"])
        self.base_time, self.base_wall = t, time.time()

    def post_render(self, body: dict) -> None:
        # interfaceAll first, whatever the order sent: the quirk below depends on it.
        items = sorted(body.items(), key=lambda kv: kv[0] != "interfaceAll")
        for k, v in items:
            if k == "cameraPosition":
                pos = dict(self.render["cameraPosition"], **v)
                pos["y"] = min(float(pos["y"]), self.quirks.max_camera_y)
                self.render[k] = pos
            elif k == "interfaceMinimap" and self.quirks.ignore_minimap \
                    and not self.render.get("interfaceAll", True):
                continue
            else:
                self.render[k] = v
        if self.quirks.ignore_minimap and body.get("interfaceAll") is False:
            self.render["interfaceMinimap"] = False

    def post_recording(self, body: dict) -> None:
        if not body.get("recording"):
            self.recording.update(body)
            return
        if self.quirks.one_recording and self.recordings_started >= 1:
            return  # silently ignored: nothing records
        self.recordings_started += 1
        self.recording = dict(body)
        game_s = float(body["endTime"]) - float(body["startTime"])
        rate = 1.0 if body.get("enforceFrameRate", True) else float(body.get("replaySpeed", 1.0))
        wall = game_s / rate * self.quirks.time_scale
        n = int(game_s * float(body.get("framesPerSecond", 1)))

        def work():
            import cv2

            out = Path(body["path"])
            out.mkdir(parents=True, exist_ok=True)
            for i in range(n):
                cv2.imwrite(str(out / f"{i:05d}.png"), np.zeros((36, 64, 3), np.uint8))
            time.sleep(wall)
            with self.lock:
                self.recording["recording"] = False

        threading.Thread(target=work, daemon=True).start()


SPEC = {"openapi": "3.0.0", "info": {"title": "mock replay api"},
        "paths": {"/replay/playback": {}, "/replay/render": {}, "/replay/recording": {},
                  "/replay/game": {}}}
PLAYERS = [{"team": "ORDER", "championName": "Lee Sin",
            "rawChampionName": "game_character_displayname_LeeSin"},
           {"team": "CHAOS", "championName": "Lux", "rawChampionName": "game_character_displayname_Lux"}]


def make_handler(state: State):
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *a):  # quiet
            pass

        def _send(self, code: int, data) -> None:
            raw = json.dumps(data).encode()
            self.send_response(code)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(raw)))
            self.end_headers()
            self.wfile.write(raw)

        def do_GET(self):
            with state.lock:
                p = self.path
                if p == state.quirks.spec_path:
                    return self._send(200, SPEC)
                routes = {"/replay/playback": state.playback, "/replay/render": lambda: state.render,
                          "/replay/recording": lambda: state.recording,
                          "/replay/game": lambda: {"processID": 4242},
                          "/liveclientdata/playerlist": lambda: PLAYERS}
                if p in routes:
                    return self._send(200, routes[p]())
                return self._send(404, {"error": "not found"})

        def do_POST(self):
            body = json.loads(self.rfile.read(int(self.headers.get("Content-Length", 0))) or b"{}")
            with state.lock:
                state.posts.append((self.path, body))
                handler = {"/replay/playback": state.post_playback,
                           "/replay/render": state.post_render,
                           "/replay/recording": state.post_recording}.get(self.path)
                if handler is None:
                    return self._send(404, {"error": "not found"})
                handler(body)
                current = {"/replay/playback": state.playback, "/replay/render": lambda: state.render,
                           "/replay/recording": lambda: state.recording}[self.path]()
                return self._send(200, current)

    return Handler


class MockReplayApi:
    def __init__(self, quirks: Quirks | None = None):
        self.state = State(quirks=quirks or Quirks())
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), make_handler(self.state))
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)

    @property
    def base(self) -> str:
        return f"http://127.0.0.1:{self.server.server_address[1]}"

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *exc):
        self.server.shutdown()
        self.server.server_close()
