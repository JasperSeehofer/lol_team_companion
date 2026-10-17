import numpy as np
import pytest

import make_fixtures as mf
from scout import track_minimap
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
