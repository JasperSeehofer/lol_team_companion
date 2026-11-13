"""Fix H (M156 R2): every live tool leaves the client's render state as it found it."""

import copy
import logging
import sys

import pytest

from scout import probe, track_minimap, track_topdown
from scout.render import minimap_only_render, preserved_render
from scout.replay_api import ReplayClient

LOG = logging.getLogger("t")


def scramble(client):
    client.set_render(**dict(minimap_only_render(), interfaceAll=False), fogOfWar=False,
                      fieldOfView=19.0, cameraPosition={"x": 1.0, "y": 20000.0, "z": 2.0})


def test_restores_after_a_normal_body(mock_api):
    before = copy.deepcopy(mock_api.state.render)
    c = ReplayClient(mock_api.base)
    with preserved_render(c, LOG) as rec:
        scramble(c)
        assert mock_api.state.render != before
    assert mock_api.state.render == before
    assert rec["restored"] and rec["mismatch"] == [] and rec["error"] is None


@pytest.mark.parametrize("exc", [RuntimeError("boom"), KeyboardInterrupt()])
def test_restores_when_the_body_raises(mock_api, exc):
    before = copy.deepcopy(mock_api.state.render)
    c = ReplayClient(mock_api.base)
    with pytest.raises(type(exc)):
        with preserved_render(c, LOG):
            scramble(c)
            raise exc
    assert mock_api.state.render == before


def test_only_readable_fields_restored(mock_api):
    mock_api.state.render["interfaceQuests"] = None          # unreadable: never posted back
    c = ReplayClient(mock_api.base)
    with preserved_render(c, LOG) as rec:
        c.set_render(interfaceQuests=True)
    assert "interfaceQuests" not in rec["saved"]
    assert mock_api.state.render["interfaceQuests"] is True
    restore_posts = [b for p, b in mock_api.state.posts if p == "/replay/render"][-2:]
    assert restore_posts[0] == {"interfaceAll": True}           # interfaceAll first, on its own
    assert "interfaceQuests" not in restore_posts[1]


def test_unreadable_render_restores_nothing():
    class Broken:
        posts = []

        def render(self):
            raise RuntimeError("no replay")

        def set_render(self, **kw):
            self.posts.append(kw)

    b = Broken()
    with preserved_render(b, LOG) as rec:
        b.set_render(interfaceAll=False)
    assert b.posts == [{"interfaceAll": False}] and not rec["restored"]
    assert rec["error"].startswith("read:")


def test_probe_leaves_render_as_found(mock_api, tmp_path):
    before = copy.deepcopy(mock_api.state.render)
    p = probe.Probe(ReplayClient(mock_api.base), tmp_path, settle=0.0, speed_window=0.05)
    res = p.run(skip_recording=True)
    assert mock_api.state.render == before
    assert res["render_restore"]["restored"] and "restore" not in res["errors"]


def test_probe_restores_on_keyboard_interrupt(mock_api, tmp_path):
    before = copy.deepcopy(mock_api.state.render)
    p = probe.Probe(ReplayClient(mock_api.base), tmp_path, settle=0.0, speed_window=0.05)

    def health_bars():
        raise KeyboardInterrupt

    p.health_bars = health_bars
    with pytest.raises(KeyboardInterrupt):
        p.run(skip_recording=True)
    assert mock_api.state.render == before
    assert mock_api.state.paused is True                      # playback put back too


def _minimap_main(mock_api, tmp_path):
    return track_minimap.main(["--out", str(tmp_path / "run"), "--base", mock_api.base,
                               "--champions", ",".join("ABCDEFGHIJ"), "--end", "11",
                               "--scratch", str(tmp_path / "scratch")])


def test_track_minimap_restores(mock_api, tmp_path, monkeypatch):
    from test_screen import fake_mss, with_primary

    monkeypatch.setitem(sys.modules, "mss", fake_mss(with_primary(), []))
    seen = {}

    def offline(frames_dir, champions, detector, out_dir, *a, **k):
        seen["during"] = copy.deepcopy(mock_api.state.render)   # detection runs after the restore
        return {}

    monkeypatch.setattr(track_minimap, "build_detector", lambda *a: None)
    monkeypatch.setattr(track_minimap, "run_offline", offline)
    before = copy.deepcopy(mock_api.state.render)
    real_capture = track_minimap.capture_live

    def capture(*a, **k):
        out = real_capture(*a, **k)
        seen["changed"] = mock_api.state.render["interfaceReplay"] is False
        return out

    monkeypatch.setattr(track_minimap, "capture_live", capture)
    assert _minimap_main(mock_api, tmp_path) == 0
    assert seen["changed"] and seen["during"] == before        # restored before detection
    assert mock_api.state.render == before


@pytest.mark.parametrize("exc", [RuntimeError("capture died"), KeyboardInterrupt()])
def test_track_minimap_restores_on_error(mock_api, tmp_path, monkeypatch, exc):
    before = copy.deepcopy(mock_api.state.render)

    def capture(client, *a, **k):
        scramble(client)
        raise exc

    monkeypatch.setattr(track_minimap, "capture_live", capture)
    with pytest.raises(type(exc)):
        _minimap_main(mock_api, tmp_path)
    assert mock_api.state.render == before


def _topdown_main(mock_api, tmp_path):
    return track_topdown.main(["--out", str(tmp_path / "run"), "--base", mock_api.base,
                               "--champions", ",".join("ABCDEFGHIJ"), "--camera-y", "20000",
                               "--start", "60", "--duration", "2", "--fps", "1",
                               "--scratch", str(tmp_path / "rec")])


def test_track_topdown_restores(mock_api, tmp_path, monkeypatch):
    monkeypatch.setattr(track_topdown.time, "sleep", lambda s: None)
    monkeypatch.setattr(track_topdown, "run_offline", lambda *a, **k: {})
    before = copy.deepcopy(mock_api.state.render)
    assert _topdown_main(mock_api, tmp_path) == 0
    posted = [b for p, b in mock_api.state.posts if p == "/replay/render"]
    assert any(b.get("fogOfWar") is False for b in posted)     # it did change the render
    assert mock_api.state.render == before


def test_track_topdown_restores_on_error(mock_api, tmp_path, monkeypatch):
    before = copy.deepcopy(mock_api.state.render)

    def record(client, args, log):
        scramble(client)
        raise KeyboardInterrupt

    monkeypatch.setattr(track_topdown, "record_live", record)
    with pytest.raises(KeyboardInterrupt):
        _topdown_main(mock_api, tmp_path)
    assert mock_api.state.render == before
