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
