import cv2

import make_fixtures as mf
from scout.calibrate import bounds_map
from scout.detect import BarDetector, TemplateDetector, load_icons


def test_template_detector_finds_all_ten(fixtures):
    img = cv2.imread(str(fixtures / "frames" / "t00060.png"))
    det = TemplateDetector(load_icons(fixtures / "icons", mf.CHAMPIONS), icon_px=mf.ICON_PX)
    found = det.detect(img)
    m = bounds_map(mf.MINIMAP_PX, mf.MINIMAP_PX)
    for champ in mf.CHAMPIONS:
        best = max((d for d in found if d.label == champ), key=lambda d: d.score, default=None)
        assert best is not None, champ
        tx, ty = m.to_pixel(*mf.position(champ, 60))
        assert abs(best.x - tx) <= 2 and abs(best.y - ty) <= 2, (champ, best, tx, ty)


def test_bar_detector_on_topdown_frame(fixtures):
    img = cv2.imread(str(fixtures / "topdown" / "t00600.png"))
    dets = BarDetector().detect(img)
    champs = [d for d in dets if d.label == "champion"]
    wards = [d for d in dets if d.label == "ward"]
    assert sorted(d.team for d in champs) == ["blue"] * 5 + ["red"] * 5
    assert sorted(d.team for d in wards) == ["blue", "blue", "red"]
    for c, (x, y) in mf.TOPDOWN_CHAMPS.items():
        px, py = mf.world_to_topdown(x, y)
        assert min(abs(d.x - px) + abs(d.y - py) for d in champs) <= 3, c


def test_bar_detector_ignores_round_blob():
    import numpy as np

    img = np.zeros((200, 200, 3), np.uint8)
    cv2.circle(img, (100, 100), 30, (230, 120, 30), -1)
    assert BarDetector().detect(img) == []


def test_yolo_detector_is_not_imported_by_tests():
    import sys

    assert "ultralytics" not in sys.modules
