"""Replay API render payloads shared by the probe and the trackers.

Interface field names follow Riot's League Director (Apache-2.0) render
fields. Seen on Jasper's run (2026-10-22): ``interfaceAll=false`` hides the
minimap together with the rest of the HUD, whatever ``interfaceMinimap`` says.
The working minimap-only combination keeps ``interfaceAll=true`` and turns
every other ``interface*`` field off.
"""

from __future__ import annotations

from contextlib import contextmanager

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


@contextmanager
def preserved_render(client, log=None):
    """GET /replay/render on entry, POST it back on exit, also when the body
    raises or is interrupted (Ctrl+C), so the client is left as found. Only
    fields that were readable (present and not null) are restored;
    interfaceAll goes first, on its own. Yields a record of what happened
    (saved fields, restored, mismatches read back, error)."""
    rec: dict = {"saved": None, "restored": False, "mismatch": [], "error": None}

    def note(msg, *a):
        if log:
            log.info(msg, *a)

    saved: dict = {}
    try:
        got = client.render()
        if isinstance(got, dict):
            saved = {k: v for k, v in got.items() if v is not None and not k.startswith("_")}
        rec["saved"] = sorted(saved)
        note("render state saved (%d fields) for restore on exit", len(saved))
    except Exception as e:  # noqa: BLE001 (nothing to restore then; the body still runs)
        rec["error"] = f"read: {e!r}"
        note("render state not readable, nothing will be restored: %r", e)
    try:
        yield rec
    finally:
        if saved:
            try:
                if "interfaceAll" in saved:
                    client.set_render(interfaceAll=saved["interfaceAll"])
                client.set_render(**{k: v for k, v in saved.items() if k != "interfaceAll"})
                rec["restored"] = True
                back = client.render()
                rec["mismatch"] = sorted(k for k, v in saved.items()
                                         if isinstance(back, dict) and back.get(k) != v)
                note("render state restored (%d fields); differing on read-back: %s", len(saved),
                     ", ".join(rec["mismatch"]) or "none")
            except Exception as e:  # noqa: BLE001 (never mask the body's own exception)
                rec["error"] = f"restore: {e!r}"
                note("render state restore failed: %r", e)
