"""Route B (M156 R2): merged health bars, stale first second, offset + homography fit."""

import numpy as np
import pytest

import make_fixtures as mf
from scout import calibrate, track_topdown
from scout.util import read_json, write_json

PMAP = {i + 1: c for i, c in enumerate(mf.CHAMPIONS)}


@pytest.fixture(scope="module")
def series(tmp_path_factory):
    d = tmp_path_factory.mktemp("topdown")
    mf.write_topdown_series(d / "frames")
    write_json(d / "anchors.json", mf.td_anchors())
    return d


def errors(track):
    return [float(np.hypot(p[0] - mf.td_world(c, s["t"])[0], p[1] - mf.td_world(c, s["t"])[1]))
            for s in track["samples"] for c, p in s["pos"].items() if p]


def run(series, tmp_path, anchors=None, identity="auto", minimap=True):
    return track_topdown.run_offline(series / "frames", tmp_path, mf.CHAMPIONS, anchors,
                                     mf.td_minimap_track() if minimap else None, 6, None, None,
                                     mf.td_timeline(), PMAP, identity)


def test_merge_bars_joins_separator_blobs_and_anchors_left():
    dets = [{"label": "champion", "team": "red", "x": 1664.0, "y": 821.7, "w": 89.0, "h": 12.0},
            {"label": "champion", "team": "red", "x": 1727.5, "y": 821.6, "w": 34.0, "h": 12.0},
            {"label": "champion", "team": "blue", "x": 1710.8, "y": 830.0, "w": 40.0, "h": 12.0},
            {"label": "ward", "team": "red", "x": 1700.0, "y": 822.0, "w": 20.0, "h": 4.0}]
    bars = track_topdown.merge_bars(dets)
    red = [b for b in bars if b["team"] == "red"]
    assert len(red) == 1 and red[0]["blobs"] == 2
    assert red[0]["left"] == pytest.approx(1664.0 - 44.0) and red[0]["right"] == pytest.approx(1745.0)
    assert len(bars) == 2   # the blue bar is another team's, the ward is no champion


def test_bar_left_edge_does_not_move_with_hp(series):
    """Detector + merge on every frame: one bar per champion at the true left edge."""
    import cv2

    from scout.detect import BarDetector

    for t in (601, 617, 633):
        dets = [d.as_dict() for d in BarDetector().detect(cv2.imread(str(series / "frames" / f"t{t:05d}.png")))]
        bars = track_topdown.merge_bars(dets, mf.TD_SCALE)
        assert len(bars) == 10, t
        for c in mf.CHAMPIONS:
            left, y = mf.td_bar_left(c, t)
            assert min(abs(b["left"] - left) + abs(b["y"] - y) for b in bars) <= 1.5, (t, c)


def test_fit_with_turret_anchors_recovers_the_offset(series, tmp_path):
    track = run(series, tmp_path, series / "anchors.json")
    fit = track["fit"]
    assert fit["mode"] == "anchors" and fit["identity"] == "minimap"
    assert abs(fit["offset_px"][0] - mf.TD_OFFSET[0]) <= 2 and abs(fit["offset_px"][1] - mf.TD_OFFSET[1]) <= 2
    assert fit["lofo_median"] is not None and fit["lofo_median"] < 40
    assert fit["n_pairs"] >= 25
    assert np.median(errors(track)) < 40
    saved = read_json(tmp_path / "track_topdown.json")
    assert saved["processing"] == "r2" and saved["calibration"]["kind"] == "homography"


def test_fit_without_anchors_direct_and_oracle(series, tmp_path):
    direct = run(series, tmp_path / "a")
    assert direct["fit"]["mode"] == "direct" and direct["fit"]["lofo_median"] < 40
    assert np.median(errors(direct)) < 40
    oracle = run(series, tmp_path / "b", identity="auto", minimap=False)
    assert oracle["fit"]["identity"] == "oracle" and "oracle fit" in oracle["fit"]["identity_note"]
    assert oracle["fit"]["lofo_median"] < 40


def test_stale_first_second_dropped(series, tmp_path, monkeypatch):
    track = run(series, tmp_path / "a", series / "anchors.json")
    assert track["stale_dropped"] == [600.0]
    assert track["samples"][0]["t"] == 601.0 and 600.0 not in track["fit"]["frames"]
    assert [c["t"] for c in track["bar_counts"]][0] == 601.0
    monkeypatch.setattr(track_topdown, "STALE_S", 0.0)   # the stale frame (t=600) then joins the fit
    bad = run(series, tmp_path / "b", series / "anchors.json")
    assert 600.0 in bad["fit"]["frames"]
    assert bad["fit"]["per_frame"]["600"]["lofo"] > 200        # it shows the game 15 s earlier


def test_reprocess_from_stored_detections_matches(series, tmp_path):
    first = run(series, tmp_path / "a")
    legacy = {k: v for k, v in first.items() if k not in ("processing", "fit", "bar_counts")}
    legacy["calibration"] = None
    again = track_topdown.reprocess(legacy, mf.td_timeline(), PMAP, mf.td_minimap_track())
    assert again["processing"] == "r2" and again["reprocessed_from"] == "r1"
    assert again["fit"]["lofo_median"] == first["fit"]["lofo_median"]


def test_without_anchors_or_timeline_keeps_pixels_and_writes_template(series, tmp_path):
    rc = track_topdown.main(["--out", str(tmp_path), "--frames", str(series / "frames"),
                             "--champions", ",".join(mf.CHAMPIONS)])
    assert rc == 0
    track = read_json(tmp_path / "track_topdown.json")
    assert track["calibration"] is None and track["fit"] is None
    assert all(c["blue"] + c["red"] == 10 for c in track["bar_counts"])
    assert (tmp_path / "anchors_template.json").exists()
    assert track["ward_units"] == "px"


def test_cli_fits_from_run_dir_files(series, tmp_path):
    write_json(tmp_path / "timeline.json", mf.td_timeline())
    write_json(tmp_path / "track_minimap.json", mf.td_minimap_track())
    assert track_topdown.main(["--out", str(tmp_path), "--frames", str(series / "frames")]) == 0
    track = read_json(tmp_path / "track_topdown.json")
    assert track["fit"]["mode"] == "direct" and track["fit"]["lofo_median"] < 40


def test_wards_pips_grouped_mana_excluded_first_appearance(series, tmp_path):
    from scout import score

    track = run(series, tmp_path, series / "anchors.json")
    summ = track["ward_summary"]
    assert summ["excluded_blobs"]["mana"] >= 10 * 40          # every mana bar, every frame
    wards = sorted(track["wards"], key=lambda w: w["first_seen"])
    assert len(wards) == len(mf.TD_WARDS)                      # one detection per ward
    for w, (team, x, y, n, placed) in zip(wards, sorted(mf.TD_WARDS, key=lambda q: q[4])):
        assert (w["team"], w["pips"]) == (team, n)
        assert np.hypot(w["x"] - x, w["y"] - y) < 150             # bar centre, no ward offset
        assert w["pre_existing"] == (placed < 600)
        if not w["pre_existing"]:   # first frame after placement, back-dated 0.15 s
            assert w["first_seen"] == np.ceil(placed) and w["t"] == pytest.approx(np.ceil(placed) - 0.15)
            assert abs(w["t"] - placed) <= 0.8
    assert track["ward_window"] == [601.0, 639.0]
    res = score.match_wards(track["wards"], mf.td_timeline(), PMAP, None, window=track["ward_window"])
    assert res["pre_existing_excluded"] == 1
    for team in ("blue", "red"):
        assert (res[team]["precision"], res[team]["recall"]) == (1.0, 1.0), (team, res[team])
    assert res["blue"]["placed"] == 2 and res["red"]["placed"] == 2   # the 590 s ward is not in the window


def test_ward_needs_two_frames_and_three_or_four_pips():
    from scout import track_topdown as td

    def pip(x, y, team="red", w=21.0):
        return {"label": "ward", "team": team, "x": x + (w - 1) / 2, "y": y, "w": w, "h": 4.0, "sat": 255.0}

    three = [pip(100, 50), pip(124.5, 50), pip(149, 50)]
    two = [pip(400, 80), pip(424.5, 80)]
    mana = [{"label": "ward", "team": "blue", "x": 560.0, "y": 211.8, "w": 109.0, "h": 4.0, "sat": 174.0}]
    champ = [{"label": "champion", "team": "red", "x": 556.0, "y": 200.0, "w": 101.0, "h": 12.0, "left": 506.0}]
    dull = [pip(700, 300, "blue")]
    dull[0]["sat"] = 195.0
    frames = [{"t": 10.0, "dets": two + champ}, {"t": 11.0, "dets": three + two + mana + champ + dull},
              {"t": 12.0, "dets": three + two + mana + champ + dull}, {"t": 13.0, "dets": three}]
    merged = {f["t"]: td.merge_bars(f["dets"]) for f in frames}
    wards, summ = td.track_wards(frames, merged, 1.0, None)
    assert [(w["pips"], w["first_seen"], w["frames"], w["pre_existing"]) for w in wards] == [(3, 11.0, 3, False)]
    assert summ["excluded_blobs"]["mana"] == 2 and summ["excluded_blobs"]["low_saturation"] == 2


def test_default_offset_scales_with_screen_and_camera():
    assert track_topdown.default_bar_offset((2560, 1440), {"height": 50000, "fov": 19}) == \
        pytest.approx(track_topdown.REF_BAR_OFFSET)
    x, y = track_topdown.default_bar_offset((1920, 1080), {"height": 100000, "fov": 19})
    assert x == pytest.approx(51.5 * 0.75) and y == pytest.approx(144.4 * 0.75 / 2)


def test_record_live_finds_frames_in_client_subfolder(tmp_path, monkeypatch):
    """The real client writes into its own subfolder (seen 2026-10-22: 16-21_EUW1-…_01)."""
    import argparse
    import logging
    from pathlib import Path

    import cv2

    from scout import track_topdown as td

    monkeypatch.setattr(td.time, "sleep", lambda s: None)

    class StubClient:
        def __init__(self):
            self.rec = {"recording": False}

        def set_render(self, **kw):
            pass

        def set_playback(self, **kw):
            pass

        def set_recording(self, **kw):
            sub = Path(kw["path"]) / "16-21_EUW1-8021305006_01"
            sub.mkdir(parents=True)
            for i in range(4):
                cv2.imwrite(str(sub / f"frame{i:05d}.png"), np.zeros((4, 4, 3), np.uint8))
            self.rec = {"recording": False, "path": str(sub), "endTime": -1.0, "width": 2560}

        def recording(self):
            return self.rec

    args = argparse.Namespace(keystroke_unlock=False, camera_y=1000.0, fov=19.0, out=tmp_path,
                              scratch=tmp_path / "scratch", start=600.0, duration=2.0, fps=2.0,
                              width=1920, height=1080)
    per_s, extra = td.record_live(StubClient(), args, logging.getLogger("t"))
    assert extra["frames_written"] == 4
    assert sorted(p.name for p in per_s.glob("t*.png")) == ["t00600.png", "t00601.png"]


def test_topdown_rerun_leaves_one_runs_samples(series, tmp_path):
    keep = tmp_path / "frames_topdown"
    keep.mkdir()
    (keep / "t09999.png").write_bytes(b"stale")
    (keep / "keep.me").write_text("foreign")
    run(series, tmp_path)
    names = sorted(p.name for p in keep.iterdir())
    assert "t09999.png" not in names and "keep.me" in names
    assert len([n for n in names if n.endswith(".png")]) == 6
