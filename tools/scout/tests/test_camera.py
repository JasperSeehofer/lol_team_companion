"""Fix C (M156 R2): the camera height comes from a visual fit, not the read-back."""

import argparse

import numpy as np
import pytest

from scout import camera as cam
from scout import probe, track_topdown
from scout.replay_api import ReplayClient
from scout.util import write_json


def frame(side_frac, w=1280, h=720):
    """A grey map square (side = side_frac * frame height) centred in a black frame."""
    img = np.zeros((h, w, 3), np.uint8)
    s = round(side_frac * h)
    x0, y0 = (w - s) // 2, (h - s) // 2
    img[max(0, y0):y0 + s, max(0, x0):x0 + s] = 90
    return img


def test_default_camera_y_scaling():
    assert cam.default_camera_y(2560, 1440) == 50000
    assert cam.default_camera_y(1920, 1080) == 50000          # pixels do not matter, shape does
    assert cam.default_camera_y(1920, 1200) == 50000          # 16:10: between 1 and 16/9
    assert cam.default_camera_y(3440, 1440) == pytest.approx(50000 * (3440 / 1440) / (16 / 9), abs=1)
    assert cam.default_camera_y(1440, 2560) == pytest.approx(50000 * 2560 / 1440, abs=1)
    narrow = cam.default_camera_y(2560, 1440, fov=38.0)       # wider lens, lower camera
    assert narrow == pytest.approx(50000 * np.tan(np.radians(9.5)) / np.tan(np.radians(19)), abs=1)


@pytest.mark.parametrize("side,fits", [(0.9, True), (0.98, False), (1.2, False), (0.5, True)])
def test_map_fit_on_drawn_map(side, fits):
    res = cam.map_fit(frame(side))
    assert res["map_found"] and res["fits"] is fits
    if fits:
        assert res["border_map_fraction"] == 0
        assert res["fill"] == pytest.approx(side * side * 720 / 1280, rel=0.02)
    else:
        assert res["border_map_fraction"] > 0.01


def test_map_fit_black_frame_and_choice():
    assert cam.map_fit(np.zeros((90, 160, 3), np.uint8))["map_found"] is False
    steps = [{"y": 42500.0, "fit": cam.map_fit(frame(1.1))},
             {"y": 50000.0, "fit": cam.map_fit(frame(0.93))},
             {"y": 57500.0, "fit": cam.map_fit(frame(0.81))}]
    assert cam.choose_height(steps, 50000.0)[0] == 50000.0
    y, how = cam.choose_height(steps[:1], 50000.0)
    assert y == 50000.0 and "no tested height" in how
    assert "no usable screenshot" in cam.choose_height([{"y": 1.0, "fit": None}], 9.0)[1]


def test_probe_camera_fit_against_the_mock(tmp_path):
    """The fake screen draws the map at a size inversely proportional to the camera
    height the mock holds: at 50000 it fills 93 % of the frame height."""
    import cv2
    from mock_replay_api import MockReplayApi, Quirks

    with MockReplayApi(Quirks(max_camera_y=1e6)) as api:
        def grab(path, full=False):
            y = api.state.render["cameraPosition"]["y"]
            cv2.imwrite(str(path), frame(0.93 * 50000 / y))

        grab.monitor = {"index": 2, "left": 0, "top": 0, "width": 2560, "height": 1440,
                        "chosen_by": "is_primary"}
        p = probe.Probe(ReplayClient(api.base), tmp_path, grab=grab, settle=0.0)
        fit = p.camera_fit()
        assert api.state.render["interfaceAll"] is False             # HUD off for the check
    assert fit["default_y"] == 50000 and fit["screen"] == [2560, 1440]
    by_y = {s["y"]: s["fit"]["fits"] for s in fit["steps"]}
    assert by_y == {42500.0: False, 50000.0: True, 57500.0: True}
    assert fit["chosen_y"] == 50000.0 and fit["chosen_by"].startswith("visual")


def test_topdown_height_precedence(tmp_path):
    args = argparse.Namespace(camera_y=None, out=tmp_path, fov=19.0, width=1920, height=1080)
    assert track_topdown.camera_height(args) == (50000.0, "scaled default for 1920x1080")
    write_json(tmp_path / "probe.json", {"camera": {"max_honoured_y": 80000},
                                         "camera_fit": {"chosen_y": 57500.0, "fov": 19.0,
                                                        "chosen_by": "visual"}})
    y, src = track_topdown.camera_height(args)
    assert y == 57500.0 and "camera_fit" in src
    args.fov = 25.0                                  # another lens: the probe's fit does not apply
    assert track_topdown.camera_height(args)[1].startswith("scaled default")
    args.camera_y = 12345.0
    assert track_topdown.camera_height(args) == (12345.0, "--camera-y")
