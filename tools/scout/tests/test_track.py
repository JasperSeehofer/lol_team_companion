import numpy as np
import pytest

import make_fixtures as mf
from scout import track_minimap
from scout.detect import TemplateDetector
from scout.track import KalmanCV, Tracker, WorldDet, find_gaps
from scout.util import read_json


def test_kalman_learns_constant_velocity():
    k = KalmanCV(0, 0)
    for t in range(1, 8):
        k.predict(1.0)
        k.update((100.0 * t, 50.0 * t))
    pred = k.predict(1.0)
    assert pred == pytest.approx((800, 400), abs=30)


def crossing(t):
    """A moves right, B moves left, both on y=0; they meet at t=4.5."""
    return (1000 + 300 * t, 0.0), (3700 - 300 * t, 0.0)


def test_identity_survives_crossing_with_ambiguous_labels():
    tr = Tracker(["A", "B"])
    for t in range(10):
        a, b = crossing(t)
        if t in (4, 5):   # overlap: the detector confuses them, both say "A"
            dets = [WorldDet("A", *a), WorldDet("A", *b)]
        else:
            dets = [WorldDet("A", *a), WorldDet("B", *b)]
        pos = tr.step(float(t), dets)
        assigned = [tuple(p) for p in pos.values() if p is not None]
        assert len(assigned) == len(set(assigned)), "two tracks on one detection"
    a, b = crossing(9)
    assert pos["A"] == pytest.approx(list(a), abs=1)
    assert pos["B"] == pytest.approx(list(b), abs=1)


def test_one_merged_detection_goes_to_one_track_only():
    tr = Tracker(["A", "B"])
    tr.step(0.0, [WorldDet("A", 0, 0), WorldDet("B", 400, 0)])
    pos = tr.step(1.0, [WorldDet("A", 200, 0)])   # B hidden under A
    assert pos["A"] == [200, 0] and pos["B"] is None


def test_far_wrong_label_is_rejected_but_own_label_teleports():
    tr = Tracker(["A", "B"], mismatch_gate=600)
    tr.step(0.0, [WorldDet("A", 0, 0), WorldDet("B", 5000, 5000)])
    pos = tr.step(1.0, [WorldDet("B", 2000, 0)])      # far from A, B's own label -> B
    assert pos["A"] is None and pos["B"] == [2000, 0]
    pos = tr.step(2.0, [WorldDet("A", 14000, 14000)])  # recall to fountain: own label, accepted
    assert pos["A"] == [14000, 14000]


def test_find_gaps_hand_case():
    samples = [{"t": float(t), "pos": {"A": None if t in (2, 3, 7, 8, 9) else [0, 0]}}
               for t in range(10)]
    gaps = find_gaps(samples, ["A"])
    assert [(g["start"], g["length_s"]) for g in gaps] == [(2.0, 2.0), (7.0, 3.0)]


def test_track_minimap_offline_on_synthetic_frames(fixtures, tmp_path):
    rc = track_minimap.main(["--out", str(tmp_path), "--frames", str(fixtures / "frames"),
                             "--icons", str(fixtures / "icons"), "--match",
                             str(fixtures / "match_synth.json")])
    assert rc == 0
    track = read_json(tmp_path / "track_minimap.json")
    assert track["matchId"] == mf.MATCH_ID and track["champions"] == mf.CHAMPIONS
    assert len(track["samples"]) == 10 and len(list((tmp_path / "frames_minimap").iterdir())) == 10
    assert (tmp_path / "run.log").exists()
    assert track["icon_sources"]["Ahri"] == {"source": "--icons", "file": "Ahri.png"}
    for s in track["samples"]:
        got = [tuple(p) for p in s["pos"].values() if p]
        assert len(got) == len(set(got))
        a, b = mf.position("Ahri", s["t"]), mf.position("Lux", s["t"])
        overlap = np.hypot(a[0] - b[0], a[1] - b[1]) < 1300   # icons touch (26 px)
        # Lux is drawn on top and always visible; Ahri may be lost or off while covered
        for champ in ("Lux",) if overlap else ("Ahri", "Lux"):
            p = s["pos"][champ]
            truth = mf.position(champ, s["t"])
            assert p is not None and np.hypot(p[0] - truth[0], p[1] - truth[1]) < 100, \
                (s["t"], champ, p)
    last = track["samples"][-1]["pos"]
    assert all(last[c] is not None for c in mf.CHAMPIONS)


def test_champions_from_client_and_match(mock_api, fixtures):
    from scout.replay_api import ReplayClient

    assert track_minimap.champions_from_client(ReplayClient(mock_api.base)) == ["LeeSin", "Lux"]
    assert track_minimap.champions_from_match(read_json(fixtures / "match_synth.json")) == mf.CHAMPIONS


def test_auto_minimap_box_is_bottom_right_square():
    box = track_minimap.auto_minimap_box(2560, 1440)
    assert box["width"] == box["height"] == 341
    assert box["left"] + box["width"] < 2560 and box["top"] + box["height"] < 1440


# Finding K: the template detector reports up to two peaks per label, and a dark
# portrait matches a static background spot (off-map strip, jungle wall) at a
# near-constant ~0.74 in every frame. That spot must not capture the track.
STATIC = (13390.9, 680.6)


def test_static_false_spot_does_not_capture_the_track():
    tr = Tracker(["A", "B"])
    # t=0: A's real icon (0.90) and the static spot (0.74)
    tr.step(0.0, [WorldDet("A", 1000, 1000, 0.90), WorldDet("A", *STATIC, 0.74),
                  WorldDet("B", 7000, 7000, 0.90)])
    # t=1: A is dead or hidden: only the static spot is reported for A
    tr.step(1.0, [WorldDet("A", *STATIC, 0.74), WorldDet("B", 7050, 7000, 0.90)])
    # t=2..20: A is back and clearly visible, the static spot is still there
    for k in range(2, 21):
        pos = tr.step(float(k), [WorldDet("A", 1000 + 100 * k, 1000, 0.90),
                                 WorldDet("A", *STATIC, 0.74),
                                 WorldDet("B", 7000 + 50 * k, 7000, 0.90)])
    assert pos["A"] == [3000.0, 1000.0], pos["A"]
    assert pos["B"] == [8000.0, 7000.0]


def test_static_false_spot_offered_first_does_not_lock_the_track():
    """The first frame offers the static spot first (Hungarian tie at cost 0)."""
    tr = Tracker(["A"])
    tr.step(0.0, [WorldDet("A", *STATIC, 0.74), WorldDet("A", 1000, 1000, 0.90)])
    pos = tr.step(1.0, [WorldDet("A", 1100, 1000, 0.90), WorldDet("A", *STATIC, 0.74)])
    assert pos["A"] == [1100.0, 1000.0], pos["A"]


# Finding F: after K the track still lands on a static spot while the
# champion is dead or hidden; a spot that recurs in >= 20 % of frames is static.
def _paste(img, icon, cx, cy, px=26):
    import cv2

    from scout.detect import circle_mask

    t = cv2.resize(icon, (px, px), interpolation=cv2.INTER_AREA)
    m = circle_mask(px) > 0
    y0, x0 = int(round(cy - px / 2)), int(round(cx - px / 2))
    img[y0:y0 + px, x0:x0 + px][m] = t[m]


def test_static_distractor_is_rejected_while_the_icon_is_hidden(tmp_path):
    import cv2

    from scout.calibrate import bounds_map
    from scout.detect import TemplateDetector

    ia, ib = mf.make_icon(0), mf.make_icon(1)
    rng = np.random.default_rng(1)
    # a dark, noisy copy of A's icon at a fixed spot: scores ~0.84 for "A" in every frame
    dark = np.clip(cv2.resize(ia, (26, 26)).astype(float) * 0.5 + rng.normal(0, 25, (26, 26, 3)),
                   0, 255).astype(np.uint8)
    spot = (280.0, 290.0)
    frames = tmp_path / "frames"
    frames.mkdir()
    hidden = set(range(30, 50))
    for t in range(80):
        img = mf.draw_minimap()
        _paste(img, dark, *spot)
        if t not in hidden:
            _paste(img, ia, 40 + 2 * t, 100)
        _paste(img, ib, 200, 40 + t)
        cv2.imwrite(str(frames / f"t{t:05d}.png"), img)
    det = TemplateDetector({"A": ia, "B": ib}, icon_px=26)
    track = track_minimap.run_offline(frames, ["A", "B"], det, tmp_path / "out")
    m = bounds_map(mf.MINIMAP_PX, mf.MINIMAP_PX)
    ss = track["static_spots"]
    assert [(s["champion"], round(s["px"]), round(s["py"])) for s in ss["spots"]] == [("A", 280, 290)]
    assert ss["rejected"]["A"] >= 80 and "B" not in ss["rejected"]
    sw = m.to_world(*spot)
    for s in track["samples"]:
        a = s["pos"]["A"]
        if int(s["t"]) in hidden:
            assert a is None, (s["t"], a)
        else:
            tx, ty = m.to_world(40 + 2 * s["t"], 100)
            assert a is not None and np.hypot(a[0] - tx, a[1] - ty) < 100, (s["t"], a)
        assert a is None or np.hypot(a[0] - sw[0], a[1] - sw[1]) > 500
        assert s["pos"]["B"] is not None


def test_static_spots_live_running_count_needs_warmup():
    from scout.track import StaticSpots

    sp = StaticSpots(radius=4, frac=0.2, warmup=10)
    kept = []
    for t in range(30):
        dets = [WorldDet("A", 50.0 + 0.5 * (t % 3), 60.0), WorldDet("A", 10.0 + 5 * t, 20.0)]
        sp.observe(dets)
        kept.append([(d.x, d.y) for d in sp.filter(dets)])
    assert all(len(k) == 2 for k in kept[:9])       # warm-up: nothing static yet
    assert all(len(k) == 1 and k[0][0] != 50.0 + 0.5 * (i % 3)
               for i, k in enumerate(kept[9:], 9))  # the moving icon survives
    desc = sp.describe()
    assert len(desc["spots"]) == 1 and desc["spots"][0]["champion"] == "A"
    assert desc["rejected"] == {"A": 21}


def test_static_spots_leave_a_moving_icon_alone():
    from scout.track import StaticSpots

    sp = StaticSpots(warmup=5)
    for t in range(50):
        sp.observe([WorldDet("A", 3.0 * t, 0.0)])
    assert sp.static() == {}


# Finding L: an offline re-run writes only its own samples into frames_minimap.
def test_offline_rerun_leaves_one_runs_samples(fixtures, tmp_path):
    from scout.detect import load_icons

    det = TemplateDetector(load_icons(fixtures / "icons", mf.CHAMPIONS), icon_px=mf.ICON_PX)
    keep = tmp_path / "frames_minimap"
    keep.mkdir()
    (keep / "t09999.png").write_bytes(b"stale sample of another capture")
    (keep / "notes.txt").write_text("not the tool's")
    track_minimap.run_offline(fixtures / "frames", mf.CHAMPIONS, det, tmp_path, max_frames=10)
    first = sorted(p.name for p in keep.iterdir())
    track_minimap.run_offline(fixtures / "frames", mf.CHAMPIONS, det, tmp_path, max_frames=4)
    names = sorted(p.name for p in keep.iterdir())
    frames = sorted(p.name for p in (fixtures / "frames").iterdir() if p.suffix == ".png")
    assert "t09999.png" not in first and "notes.txt" in first
    assert len([n for n in names if n.endswith(".png")]) == 4
    assert set(names) - {"notes.txt"} <= set(frames)


def test_offline_run_on_its_own_sample_folder_keeps_the_input(fixtures, tmp_path):
    import shutil

    from scout.detect import load_icons

    keep = tmp_path / "frames_minimap"
    shutil.copytree(fixtures / "frames", keep)
    before = sorted(p.name for p in keep.iterdir())
    det = TemplateDetector(load_icons(fixtures / "icons", mf.CHAMPIONS), icon_px=mf.ICON_PX)
    track_minimap.run_offline(keep, mf.CHAMPIONS, det, tmp_path, max_frames=3)
    assert sorted(p.name for p in keep.iterdir()) == before
