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


def test_bar_detector_on_topdown_frame():
    img = mf.td_frame(620)
    dets = BarDetector().detect(img)
    champs = [d for d in dets if d.label == "champion"]
    assert sorted(d.team for d in champs) == ["blue"] * 5 + ["red"] * 5   # split bars merged
    for c in mf.CHAMPIONS:
        left, y = mf.td_bar_left(c, 620)
        assert min(abs(d.left - left) + abs(d.y - y) for d in champs) <= 1.5, c
    pips = [d for d in dets if d.label == "ward" and d.sat >= 230]
    assert len(pips) == 3 + 3              # two 3-pip wards placed by 620
    assert {d.team for d in pips} == {"blue", "red"}


def test_bar_detector_ignores_round_blob():
    import numpy as np

    img = np.zeros((200, 200, 3), np.uint8)
    cv2.circle(img, (100, 100), 30, (230, 120, 30), -1)
    assert BarDetector().detect(img) == []


def test_yolo_detector_is_not_imported_by_tests():
    import sys

    assert "ultralytics" not in sys.modules


# Finding J: base circle icons (CommunityDragon) first, Data Dragon squares as fallback.
def _png(color=(10, 200, 30), alpha=True):
    import numpy as np

    img = np.zeros((128, 128, 4 if alpha else 3), np.uint8)
    cv2.circle(img, (64, 64), 60, (*color, 255) if alpha else color, -1)
    return cv2.imencode(".png", img)[1].tobytes()


class FakeHttp:
    def __init__(self, ok: dict):
        self.ok, self.calls = ok, []

    def __call__(self, url):
        self.calls.append(url)
        return (200, self.ok[url]) if url in self.ok else (404, b"")


def test_fetch_icons_takes_the_base_circle_first(tmp_path):
    from scout.detect import fetch_icons

    url = "https://raw.communitydragon.org/latest/game/assets/characters/viego/hud/viego_circle.png"
    http = FakeHttp({url: _png()})
    src = fetch_icons(["Viego"], tmp_path, get=http)
    assert src["Viego"]["source"] == "communitydragon_circle" and src["Viego"]["url"] == url
    assert (tmp_path / "cdragon" / "Viego.png").read_bytes() == http.ok[url]
    assert http.calls == [url]                       # Data Dragon never asked


def test_fetch_icons_tries_circle_0_then_falls_back_to_data_dragon(tmp_path):
    from scout.detect import fetch_icons

    base = "https://raw.communitydragon.org/latest/game/assets/characters/{c}/hud/{c}_circle{s}.png"
    http = FakeHttp({base.format(c="pyke", s="_0"): _png(),
                     "https://ddragon.leagueoflegends.com/api/versions.json": b'["16.21.1", "16.20.1"]',
                     "https://ddragon.leagueoflegends.com/cdn/16.21.1/img/champion/Jax.png":
                         _png(alpha=False)})
    src = fetch_icons(["Pyke", "Jax"], tmp_path, get=http)
    assert src["Pyke"]["source"] == "communitydragon_circle" and src["Pyke"]["url"].endswith("_circle_0.png")
    assert src["Jax"]["source"] == "ddragon_square 16.21.1"
    assert (tmp_path / "ddragon" / "16.21.1" / "Jax.png").exists()
    assert http.calls == [base.format(c="pyke", s=""), base.format(c="pyke", s="_0"),
                          base.format(c="jax", s=""), base.format(c="jax", s="_0"),
                          "https://ddragon.leagueoflegends.com/api/versions.json",
                          "https://ddragon.leagueoflegends.com/cdn/16.21.1/img/champion/Jax.png"]


def test_fetch_icons_uses_the_cache(tmp_path):
    from scout.detect import fetch_icons

    (tmp_path / "cdragon").mkdir()
    (tmp_path / "cdragon" / "Zed.png").write_bytes(_png())

    def no_network(url):
        raise AssertionError(f"fetched {url}")

    assert fetch_icons(["Zed"], tmp_path, get=no_network)["Zed"]["cached"] is True


def test_rgba_icon_alpha_becomes_the_template_mask():
    import numpy as np

    from scout.detect import circle_mask, load_icon_files

    icon = cv2.imdecode(np.frombuffer(_png((200, 40, 40)), np.uint8), cv2.IMREAD_UNCHANGED)
    for y in range(8, 128, 20):                       # texture, so the match has variance
        cv2.line(icon, (64, y), (128, y + 10), (250, 250, 250, 255), 5)
    icon[:, :64, 3] = 0                               # left half transparent
    icon[:, :64, :3] = (0, 0, 255)                    # whatever is under the alpha must not matter
    det = TemplateDetector({"A": icon}, icon_px=26)
    tmpl, mask = det.templates["A"][0]
    assert tmpl.shape == (26, 26, 3) and mask[:, :10].max() == 0 and mask[13, 20] == 255
    assert mask.sum() < circle_mask(26).sum()
    img = np.full((120, 120, 3), (40, 70, 35), np.uint8)
    small = cv2.resize(icon, (26, 26), interpolation=cv2.INTER_AREA)
    m = small[:, :, 3] >= 128
    img[50:76, 30:56][m] = small[:, :, :3][m]         # only the opaque half is drawn
    best = max(det.detect(img), key=lambda d: d.score)
    assert abs(best.x - 43) <= 1 and abs(best.y - 63) <= 1 and best.score > 0.9
