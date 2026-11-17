import numpy as np
import pytest

from scout import calibrate
from scout.calibrate import MAP_X, MAP_Y, TURRETS


def test_bounds_map_corners_and_y_flip():
    m = calibrate.bounds_map(300, 300)
    assert m.to_world(0, 0) == pytest.approx((MAP_X[0], MAP_Y[1]))      # top-left = max world y
    assert m.to_world(300, 300) == pytest.approx((MAP_X[1], MAP_Y[0]))  # bottom-right = min y
    assert m.ay < 0 < m.ax


def test_linear_fit_round_trip_from_turrets():
    true = calibrate.bounds_map(256, 256)
    names = ["blue_top_outer", "blue_bot_outer", "red_mid_outer", "red_nexus", "blue_nexus"]
    worlds = [TURRETS[n] for n in names]
    rng = np.random.default_rng(1)
    pixels = [tuple(np.array(true.to_pixel(*w)) + rng.normal(0, 0.3, 2)) for w in worlds]
    fit = calibrate.fit_linear(pixels, worlds)
    assert fit.ay < 0  # pixel y down, world y up
    for w in worlds:
        px = fit.to_pixel(*w)
        assert fit.to_world(*px) == pytest.approx(w, abs=1e-6)
    assert max(calibrate.residuals(fit, pixels, worlds)) < 40  # 0.3 px noise * ~58 u/px


def test_linear_fit_needs_two_pairs():
    with pytest.raises(ValueError):
        calibrate.fit_linear([(1, 2)], [(3, 4)])


def test_homography_round_trip():
    H_true = np.array([[0.046, 0.004, 300.0], [0.002, -0.046, 700.0], [0.0, 1.5e-6, 1.0]])
    worlds = [TURRETS[n] for n in ("blue_nexus", "red_nexus", "blue_top_outer", "red_bot_outer",
                                   "blue_mid_inner", "red_mid_inner")]
    pixels = []
    for x, y in worlds:
        v = H_true @ np.array([x, y, 1.0])
        pixels.append((v[0] / v[2], v[1] / v[2]))
    h = calibrate.fit_homography(pixels, worlds)
    for p, w in zip(pixels, worlds):
        assert h.to_world(*p) == pytest.approx(w, abs=0.5)
        assert h.to_pixel(*w) == pytest.approx(p, abs=0.01)
    assert max(calibrate.residuals(h, pixels, worlds)) < 0.5


def test_load_anchors_by_name_and_world(tmp_path):
    p = tmp_path / "anchors.json"
    p.write_text('{"pairs": [{"name": "blue_nexus", "pixel": [10, 20]},'
                 ' {"world": [5, 6], "pixel": [1, 2]}, {"name": "red_nexus", "pixel": null}]}')
    pixels, worlds = calibrate.load_anchors(p)
    assert pixels == [(10, 20), (1, 2)]
    assert worlds == [TURRETS["blue_nexus"], (5, 6)]
