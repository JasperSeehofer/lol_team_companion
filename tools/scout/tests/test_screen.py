"""Fix A (M156 R2): capture the primary monitor, never mss monitors[1] blindly."""

import logging
import sys
import types

import numpy as np
import pytest

from scout import probe, screen, track_minimap
from scout.replay_api import ReplayClient

VIRTUAL = {"left": -1440, "top": -600, "width": 4000, "height": 2560}
PORTRAIT = {"left": -1440, "top": -600, "width": 1440, "height": 2560}   # secondary, listed first
PRIMARY = {"left": 0, "top": 0, "width": 2560, "height": 1440}


def with_primary():
    return [VIRTUAL, dict(PORTRAIT, is_primary=False), dict(PRIMARY, is_primary=True)]


def without_key():
    return [VIRTUAL, dict(PORTRAIT), dict(PRIMARY)]


def fake_mss(monitors, grabs):
    class Sct:
        def __init__(self):
            self.monitors = monitors

        def __enter__(self):
            return self

        def __exit__(self, *a):
            return False

        def grab(self, box):
            grabs.append(dict(box))
            return np.zeros((box["height"], box["width"], 4), np.uint8)

    return types.SimpleNamespace(mss=Sct)


@pytest.mark.parametrize("monitors,how", [(with_primary(), "is_primary"),
                                          (without_key(), "at origin (no is_primary key)")])
def test_pick_monitor_finds_primary_not_index_1(monitors, how):
    index, mon, chosen_by = screen.pick_monitor(monitors)
    assert index == 2 and mon["width"] == 2560 and chosen_by == how


def test_pick_monitor_override_and_bounds():
    assert screen.pick_monitor(with_primary(), 1)[0] == 1
    with pytest.raises(SystemExit):
        screen.pick_monitor(with_primary(), 3)
    # no is_primary, nothing at the origin: the first physical monitor
    odd = [VIRTUAL, dict(PRIMARY, left=10), dict(PORTRAIT)]
    assert screen.pick_monitor(odd)[0] == 1


def test_probe_grabber_captures_the_primary(monkeypatch, tmp_path):
    grabs = []
    monkeypatch.setitem(sys.modules, "mss", fake_mss(with_primary(), grabs))
    grab = probe.screen_grabber(log=logging.getLogger("t"))
    grab(tmp_path / "a.png")
    assert grabs == [PRIMARY]
    assert grab.monitor == {"index": 2, **PRIMARY, "chosen_by": "is_primary"}
    p = probe.Probe(None, tmp_path, grab=grab)
    assert p.result["monitor"]["index"] == 2


def test_minimap_box_relative_to_chosen_monitor(monkeypatch, mock_api, tmp_path):
    grabs = []
    second = {"left": 2560, "top": 200, "width": 1920, "height": 1080}
    monitors = [VIRTUAL, dict(PORTRAIT), dict(PRIMARY), dict(second)]
    monkeypatch.setitem(sys.modules, "mss", fake_mss(monitors, grabs))
    client = ReplayClient(mock_api.base)
    _, cap = track_minimap.capture_live(client, None, tmp_path / "f", 8.0, 0.0, 1.0,
                                        logging.getLogger("t"))
    expect = track_minimap.auto_minimap_box(2560, 1440)
    assert cap["monitor"]["index"] == 2 and cap["box"] == expect      # origin 0,0
    _, cap = track_minimap.capture_live(client, None, tmp_path / "g", 8.0, 0.0, 11.0,
                                        logging.getLogger("t"), monitor=3)
    box = track_minimap.auto_minimap_box(1920, 1080)
    assert cap["box"] == dict(box, left=box["left"] + 2560, top=box["top"] + 200)
    assert grabs and grabs[-1] == cap["box"]          # the frames come from that box


# Finding I: Windows HDR tone-maps captures; the capture tools warn.
from scout import winhdr  # noqa: E402

D_PRIMARY = {"gdi_name": r"\\.\DISPLAY1", **PRIMARY, "supported": True, "enabled": True}
D_PORTRAIT = {"gdi_name": r"\\.\DISPLAY2", **PORTRAIT, "supported": False, "enabled": False}


def test_hdr_query_is_none_off_windows(monkeypatch):
    monkeypatch.setattr(winhdr.sys, "platform", "linux")
    assert winhdr.query_displays() is None
    assert winhdr.check(PRIMARY, logging.getLogger("t")) is None


def test_hdr_on_for_captured_monitor_warns(caplog):
    caplog.set_level(logging.INFO)
    res = winhdr.check(dict(PRIMARY), logging.getLogger("t"), query=lambda: [D_PORTRAIT, D_PRIMARY])
    assert res["enabled"] is True and res["matched_by"] == "rectangle"
    assert res["display"] == r"\\.\DISPLAY1"
    assert any(r.levelno == logging.WARNING and "HDR is ON" in r.message for r in caplog.records)


def test_hdr_off_for_captured_monitor_does_not_warn(caplog):
    caplog.set_level(logging.INFO)
    res = winhdr.check(dict(PORTRAIT), logging.getLogger("t"), query=lambda: [D_PORTRAIT, D_PRIMARY])
    assert res["enabled"] is False
    assert not [r for r in caplog.records if r.levelno >= logging.WARNING]


def test_hdr_matching_fallbacks():
    scaled = dict(D_PRIMARY, width=1707, height=960)   # same origin, other size (DPI)
    assert winhdr.hdr_for_monitor([D_PORTRAIT, scaled], PRIMARY)["matched_by"] == "origin"
    assert winhdr.hdr_for_monitor([dict(D_PRIMARY, left=5)], PRIMARY)["matched_by"] == "only display"
    far = {"left": 9000, "top": 0, "width": 10, "height": 10}
    res = winhdr.hdr_for_monitor([dict(D_PORTRAIT), dict(D_PRIMARY, left=5)], far)
    assert res["enabled"] is True and res["display"] is None   # some display has HDR on


def test_hdr_query_failure_is_recorded_not_raised(caplog):
    def boom():
        raise OSError("QueryDisplayConfig returned 87")

    res = winhdr.check(PRIMARY, logging.getLogger("t"), query=boom)
    assert res == {"enabled": None, "error": "QueryDisplayConfig returned 87"}
    assert any(r.levelno == logging.WARNING for r in caplog.records)


def test_probe_and_capture_record_hdr(monkeypatch, mock_api, tmp_path):
    grabs = []
    monkeypatch.setitem(sys.modules, "mss", fake_mss(with_primary(), grabs))
    monkeypatch.setattr(winhdr, "query_displays", lambda: [D_PORTRAIT, D_PRIMARY])
    grab = probe.screen_grabber(log=logging.getLogger("t"))
    assert probe.Probe(None, tmp_path, grab=grab).result["hdr"]["enabled"] is True
    _, cap = track_minimap.capture_live(ReplayClient(mock_api.base), None, tmp_path / "f", 8.0,
                                        0.0, 1.0, logging.getLogger("t"))
    assert cap["hdr"]["enabled"] is True and cap["hdr"]["matched_by"] == "rectangle"
