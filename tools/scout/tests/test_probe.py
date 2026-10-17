from scout import probe
from scout.replay_api import ReplayClient
from scout.util import read_json


def test_probe_records_mock_quirks(mock_api, tmp_path):
    shots = []

    def grab(path, full=False):     # stands in for mss: a tiny grey PNG
        import cv2
        import numpy as np

        shots.append(path.name)
        cv2.imwrite(str(path), np.full((10, 10, 3), 80, np.uint8))

    p = probe.Probe(ReplayClient(mock_api.base), tmp_path, grab=grab, settle=0.0,
                    speed_window=0.3, rec_seconds=2.0, rec_fps=4, rec_timeout=5.0,
                    scratch=tmp_path / "rec")
    res = p.run()
    assert res["errors"] == {}
    assert res["spec"]["answered"] == "/swagger/v3/openapi.json"
    assert read_json(tmp_path / "openapi.json")["openapi"] == "3.0.0"
    assert res["interface"]["interfaceAll"]["honoured"] is True
    assert res["interface"]["interfaceMinimap"]["honoured"] is False   # quirk: minimap hidden
    assert res["interface"]["screenshot"]["file"] == "probe_minimap_only.png"
    speeds = {r["sent"]: r for r in res["speed"]["steps"]}
    assert speeds[16]["read"] == 8 and res["speed"]["max_honoured"] == 8
    assert 6 < speeds[16]["game_s_per_wall_s"] < 10
    assert res["camera"]["max_honoured_y"] == 20000                  # quirk: capped at 25000
    assert res["camera"]["steps"][-1]["read_y"] == 25000
    assert all(v["honoured"] for v in res["health_bars"].values())
    assert res["fog"]["fogOfWar"]["honoured"] is True
    rec = res["recording"]
    assert rec["realtime"]["frames_written"] == 8 and rec["realtime"]["frame_size"] == [64, 36]
    assert rec["accelerated"]["frames_written"] == 0                 # quirk: one per session
    # restored afterwards
    assert mock_api.state.render["interfaceAll"] is True
    assert "probe_camera_2000.png" in shots


def test_probe_isolates_failures(tmp_path):
    from mock_replay_api import MockReplayApi, Quirks

    with MockReplayApi(Quirks(spec_path=None)) as api:
        p = probe.Probe(ReplayClient(api.base), tmp_path, settle=0.0, speed_window=0.05)
        def recording():
            raise RuntimeError("boom")

        p.recording = recording
        res = p.run()
    assert res["spec"]["answered"] is None
    assert set(res["spec"]["tried"].values()) == {404}
    assert "RuntimeError('boom')" in res["errors"]["recording"]
    assert "camera" in res and "health_bars" in res


def test_probe_dry_posts_nothing(capsys, tmp_path):
    assert probe.main(["--out", str(tmp_path), "--dry", "--base", "http://127.0.0.1:9"]) == 0
    out = capsys.readouterr().out
    assert "interfaceMinimap" in out and "speed:16" in out
    assert not (tmp_path / "probe.json").exists()


def test_client_verify_only_off_for_loopback():
    assert ReplayClient("https://127.0.0.1:2999").verify is False
    assert ReplayClient("https://example.org").verify is True
