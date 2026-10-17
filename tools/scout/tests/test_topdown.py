import numpy as np

import make_fixtures as mf
from scout import track_topdown
from scout.util import read_json, write_json


def test_topdown_offline_identity_and_homography(fixtures, tmp_path):
    rng = np.random.default_rng(5)
    t = float(mf.TOPDOWN_T)
    # a minimap track at the same time, a few hundred units off (minimap is coarse)
    minimap = {"matchId": mf.MATCH_ID, "champions": mf.CHAMPIONS, "samples": [{"t": t, "pos": {
        c: [x + rng.normal(0, 250), y + rng.normal(0, 250)]
        for c, (x, y) in mf.TOPDOWN_CHAMPS.items()}}]}
    write_json(tmp_path / "track_minimap.json", minimap)
    rc = track_topdown.main(["--out", str(tmp_path), "--frames", str(fixtures / "topdown"),
                             "--anchors", str(fixtures / "anchors_topdown.json")])
    assert rc == 0
    track = read_json(tmp_path / "track_topdown.json")
    assert track["source"] == "topdown" and track["calibration"]["kind"] == "homography"
    pos = track["samples"][0]["pos"]
    for c, (x, y) in mf.TOPDOWN_CHAMPS.items():
        assert pos[c] is not None, c
        assert np.hypot(pos[c][0] - x, pos[c][1] - y) < 80, (c, pos[c], (x, y))
    assert sorted(w["team"] for w in track["wards"]) == ["blue", "blue", "red"]


def test_topdown_without_anchors_keeps_pixels_and_writes_template(fixtures, tmp_path):
    rc = track_topdown.main(["--out", str(tmp_path), "--frames", str(fixtures / "topdown"),
                             "--champions", ",".join(mf.CHAMPIONS)])
    assert rc == 0
    track = read_json(tmp_path / "track_topdown.json")
    assert track["calibration"] is None
    assert len(track["detections_px"][0]["dets"]) == 13
    assert (tmp_path / "anchors_template.json").exists()
    assert track["ward_units"] == "px" and len(track["wards"]) == 3   # still usable for timing
