"""Replay API render payloads shared by the probe and the trackers.

Interface field names follow Riot's League Director (Apache-2.0) render
fields. Seen on Jasper's run (2026-10-22): ``interfaceAll=false`` hides the
minimap together with the rest of the HUD, whatever ``interfaceMinimap`` says.
The working minimap-only combination keeps ``interfaceAll=true`` and turns
every other ``interface*`` field off.
"""

from __future__ import annotations

INTERFACE_FIELDS = [
    "interfaceAll", "interfaceAnnounce", "interfaceChat", "interfaceFrames",
    "interfaceKillCallouts", "interfaceMinimap", "interfaceNeutralTimers", "interfaceQuests",
    "interfaceReplay", "interfaceScore", "interfaceScoreboard", "interfaceTarget",
    "interfaceTimeline",
]

# The combination R1 used; known to hide the minimap (kept only so the probe records it).
LEGACY_MINIMAP_ONLY = {"interfaceAll": False, "interfaceMinimap": True}
LEGACY_NOTE = ("interfaceAll=false + interfaceMinimap=true: known to hide the minimap "
               "(Jasper's run 2026-10-22)")


def minimap_only_render() -> dict:
    """interfaceAll and interfaceMinimap on, every other interface* field off.
    interfaceAll comes first: a client applying fields in order sees it before
    interfaceMinimap."""
    out = {"interfaceAll": True, "interfaceMinimap": True}
    out.update({f: False for f in INTERFACE_FIELDS if f not in out})
    return out
