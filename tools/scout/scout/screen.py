"""Which monitor to capture.

mss lists monitors as ``sct.monitors``: index 0 is the virtual screen spanning
all of them, 1.. are the physical monitors in the order the OS reports them.
Index 1 is not necessarily the primary monitor (seen 2026-10-22: it was
Jasper's secondary portrait monitor), so the replay window, which opens on the
primary monitor, is found by ``is_primary`` (mss 10 reports it on Windows) or,
when the key is absent, by the monitor at the origin (left=0, top=0, which is
where Windows puts the primary monitor). ``--monitor N`` overrides both.
"""

from __future__ import annotations


def pick_monitor(monitors: list[dict], index: int | None = None) -> tuple[int, dict, str]:
    """(mss index, monitor dict, how it was chosen). ``index`` is an explicit
    override (1-based, as in ``sct.monitors``)."""
    if len(monitors) < 2:
        raise SystemExit("mss reports no physical monitor")
    if index is not None:
        if not 1 <= index < len(monitors):
            raise SystemExit(f"--monitor {index}: mss reports monitors 1..{len(monitors) - 1}")
        return index, monitors[index], "--monitor"
    physical = list(enumerate(monitors))[1:]
    for i, m in physical:
        if m.get("is_primary"):
            return i, m, "is_primary"
    for i, m in physical:
        if m.get("left") == 0 and m.get("top") == 0:
            return i, m, "at origin (no is_primary key)"
    return 1, monitors[1], "first listed (no is_primary, none at origin)"


def describe(index: int, mon: dict, how: str) -> dict:
    return {"index": index, "left": mon["left"], "top": mon["top"], "width": mon["width"],
            "height": mon["height"], "chosen_by": how}


def all_monitors(monitors: list[dict]) -> list[dict]:
    return [{"index": i, "left": m["left"], "top": m["top"], "width": m["width"],
             "height": m["height"], "is_primary": m.get("is_primary")}
            for i, m in list(enumerate(monitors))[1:]]
