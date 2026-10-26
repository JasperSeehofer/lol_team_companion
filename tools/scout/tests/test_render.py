"""Fix B (M156 R2): the minimap-only render keeps interfaceAll on."""

import logging

from scout import track_minimap
from scout.render import INTERFACE_FIELDS, LEGACY_MINIMAP_ONLY, minimap_only_render
from scout.replay_api import ReplayClient


def test_minimap_only_payload():
    r = minimap_only_render()
    assert list(r)[0] == "interfaceAll"
    assert set(r) == set(INTERFACE_FIELDS)
    for f in ["interfaceReplay", "interfaceScore", "interfaceScoreboard", "interfaceFrames",
              "interfaceTimeline", "interfaceChat", "interfaceTarget", "interfaceQuests",
              "interfaceAnnounce", "interfaceKillCallouts", "interfaceNeutralTimers"]:
        assert r[f] is False, f
    assert r["interfaceAll"] is True and r["interfaceMinimap"] is True
    assert LEGACY_MINIMAP_ONLY["interfaceAll"] is False


def test_mock_hides_minimap_only_with_interface_all_off(mock_api):
    c = ReplayClient(mock_api.base)
    c.set_render(**LEGACY_MINIMAP_ONLY)
    assert c.render()["interfaceMinimap"] is False
    c.set_render(**minimap_only_render())
    back = c.render()
    assert back["interfaceMinimap"] is True and back["interfaceScoreboard"] is False


def test_track_minimap_live_posts_the_minimap_only_render(mock_api, tmp_path, monkeypatch):
    import sys
    from test_screen import fake_mss, with_primary

    monkeypatch.setitem(sys.modules, "mss", fake_mss(with_primary(), []))
    track_minimap.capture_live(ReplayClient(mock_api.base), None, tmp_path / "f", 8.0, 0.0, 1.0,
                               logging.getLogger("t"))
    posted = [b for path, b in mock_api.state.posts if path == "/replay/render"]
    assert posted and posted[0]["interfaceAll"] is True and posted[0]["interfaceMinimap"] is True
    assert posted[0]["interfaceReplay"] is False and posted[0]["fogOfWar"] is False
