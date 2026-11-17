"""Finding I: is Windows HDR (advanced colour) on for the captured monitor?

With HDR on, Windows tone-maps what mss/GDI captures (Jasper's v1 frames:
mean brightness 137 against 88 with HDR off; minimap median error 206.4 ->
196.6 from turning HDR off alone), and the champion icon templates no longer
match. The capture tools check it once and warn; they do not change it.

Windows only: QueryDisplayConfig(QDC_ONLY_ACTIVE_PATHS) lists the active
paths; for each, DisplayConfigGetDeviceInfo with
DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO (type 9) on the target
gives the advancedColorSupported / advancedColorEnabled bits, and the
source mode gives the desktop rectangle, which is matched to the mss monitor.
Elsewhere (tests, Linux) query_displays() returns None.
"""

from __future__ import annotations

import sys

QDC_ONLY_ACTIVE_PATHS = 0x2
ERROR_SUCCESS = 0
ERROR_INSUFFICIENT_BUFFER = 122
DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE = 1
DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME = 1
DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO = 9


def _query_windows() -> list[dict]:
    import ctypes
    from ctypes import wintypes as w

    class LUID(ctypes.Structure):
        _fields_ = [("LowPart", w.DWORD), ("HighPart", w.LONG)]

    class RATIONAL(ctypes.Structure):
        _fields_ = [("Numerator", ctypes.c_uint32), ("Denominator", ctypes.c_uint32)]

    class SOURCE_INFO(ctypes.Structure):
        _fields_ = [("adapterId", LUID), ("id", ctypes.c_uint32), ("modeInfoIdx", ctypes.c_uint32),
                    ("statusFlags", ctypes.c_uint32)]

    class TARGET_INFO(ctypes.Structure):
        _fields_ = [("adapterId", LUID), ("id", ctypes.c_uint32), ("modeInfoIdx", ctypes.c_uint32),
                    ("outputTechnology", ctypes.c_uint32), ("rotation", ctypes.c_uint32),
                    ("scaling", ctypes.c_uint32), ("refreshRate", RATIONAL),
                    ("scanLineOrdering", ctypes.c_uint32), ("targetAvailable", w.BOOL),
                    ("statusFlags", ctypes.c_uint32)]

    class PATH_INFO(ctypes.Structure):
        _fields_ = [("sourceInfo", SOURCE_INFO), ("targetInfo", TARGET_INFO),
                    ("flags", ctypes.c_uint32)]

    class REGION(ctypes.Structure):
        _fields_ = [("cx", ctypes.c_uint32), ("cy", ctypes.c_uint32)]

    class VIDEO_SIGNAL_INFO(ctypes.Structure):
        _fields_ = [("pixelRate", ctypes.c_uint64), ("hSyncFreq", RATIONAL), ("vSyncFreq", RATIONAL),
                    ("activeSize", REGION), ("totalSize", REGION), ("videoStandard", ctypes.c_uint32),
                    ("scanLineOrdering", ctypes.c_uint32)]

    class SOURCE_MODE(ctypes.Structure):
        _fields_ = [("width", ctypes.c_uint32), ("height", ctypes.c_uint32),
                    ("pixelFormat", ctypes.c_uint32), ("position", w.POINT)]

    class DESKTOP_IMAGE_INFO(ctypes.Structure):
        _fields_ = [("PathSourceSize", w.POINT), ("DesktopImageRegion", w.RECT),
                    ("DesktopImageClip", w.RECT)]

    class MODE_UNION(ctypes.Union):
        _fields_ = [("targetMode", VIDEO_SIGNAL_INFO), ("sourceMode", SOURCE_MODE),
                    ("desktopImageInfo", DESKTOP_IMAGE_INFO)]

    class MODE_INFO(ctypes.Structure):
        _fields_ = [("infoType", ctypes.c_uint32), ("id", ctypes.c_uint32), ("adapterId", LUID),
                    ("u", MODE_UNION)]

    class HEADER(ctypes.Structure):
        _fields_ = [("type", ctypes.c_uint32), ("size", ctypes.c_uint32), ("adapterId", LUID),
                    ("id", ctypes.c_uint32)]

    class ADVANCED_COLOR_INFO(ctypes.Structure):
        _fields_ = [("header", HEADER), ("value", ctypes.c_uint32),
                    ("colorEncoding", ctypes.c_uint32), ("bitsPerColorChannel", ctypes.c_uint32)]

    class SOURCE_NAME(ctypes.Structure):
        _fields_ = [("header", HEADER), ("viewGdiDeviceName", ctypes.c_wchar * 32)]

    user32 = ctypes.WinDLL("user32")
    n_paths, n_modes = ctypes.c_uint32(), ctypes.c_uint32()
    for _ in range(3):   # the configuration can change between the two calls
        rc = user32.GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, ctypes.byref(n_paths),
                                                ctypes.byref(n_modes))
        if rc != ERROR_SUCCESS:
            raise OSError(f"GetDisplayConfigBufferSizes returned {rc}")
        paths = (PATH_INFO * n_paths.value)()
        modes = (MODE_INFO * n_modes.value)()
        rc = user32.QueryDisplayConfig(QDC_ONLY_ACTIVE_PATHS, ctypes.byref(n_paths), paths,
                                       ctypes.byref(n_modes), modes, None)
        if rc != ERROR_INSUFFICIENT_BUFFER:
            break
    if rc != ERROR_SUCCESS:
        raise OSError(f"QueryDisplayConfig returned {rc}")
    out = []
    for p in paths[:n_paths.value]:
        info = ADVANCED_COLOR_INFO()
        info.header.type = DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO
        info.header.size = ctypes.sizeof(info)
        info.header.adapterId = p.targetInfo.adapterId
        info.header.id = p.targetInfo.id
        rc = user32.DisplayConfigGetDeviceInfo(ctypes.byref(info.header))
        name = SOURCE_NAME()
        name.header.type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME
        name.header.size = ctypes.sizeof(name)
        name.header.adapterId = p.sourceInfo.adapterId
        name.header.id = p.sourceInfo.id
        user32.DisplayConfigGetDeviceInfo(ctypes.byref(name.header))
        row = {"gdi_name": name.viewGdiDeviceName, "left": None, "top": None, "width": None,
               "height": None, "supported": None, "enabled": None}
        idx = p.sourceInfo.modeInfoIdx
        if idx < n_modes.value and modes[idx].infoType == DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE:
            sm = modes[idx].u.sourceMode
            row.update(left=sm.position.x, top=sm.position.y, width=sm.width, height=sm.height)
        if rc == ERROR_SUCCESS:
            row.update(supported=bool(info.value & 0x1), enabled=bool(info.value & 0x2))
        out.append(row)
    return out


def query_displays() -> list[dict] | None:
    """Active displays with their desktop rectangle and HDR bits; None when
    not on Windows."""
    if sys.platform != "win32":
        return None
    return _query_windows()


def hdr_for_monitor(displays: list[dict], mon: dict | None) -> dict:
    """Match the captured mss monitor to a display: same rectangle, else same
    origin, else the only display. Unmatched: enabled is True when any display
    has HDR on (it may be the one captured), else False."""
    rect = ("left", "top", "width", "height")
    found, how = None, None
    if mon:
        found = next((d for d in displays if all(d.get(k) == mon.get(k) for k in rect)), None)
        how = "rectangle" if found else None
        if not found:
            found = next((d for d in displays
                          if d.get("left") == mon.get("left") and d.get("top") == mon.get("top")), None)
            how = "origin" if found else None
    if not found and len(displays) == 1:
        found, how = displays[0], "only display"
    if found:
        return {"enabled": found.get("enabled"), "supported": found.get("supported"),
                "matched_by": how, "display": found.get("gdi_name"), "displays": displays}
    return {"enabled": any(d.get("enabled") for d in displays), "supported": None,
            "matched_by": "not matched (any display)", "display": None, "displays": displays}


def check(mon: dict | None, log=None, query=None) -> dict | None:
    """The `hdr` entry for a tool's JSON; logs a WARNING when HDR is on.
    None when the query is unavailable (not Windows); a failed query is
    recorded, never raised."""
    try:
        displays = (query or query_displays)()
    except Exception as e:  # noqa: BLE001
        if log:
            log.warning("could not read the Windows HDR state (%s); check it by hand: "
                        "Settings > System > Display > Use HDR must be off", e)
        return {"enabled": None, "error": str(e)}
    if displays is None:
        return None
    res = hdr_for_monitor(displays, mon)
    if log:
        if res["enabled"]:
            log.warning("Windows HDR is ON for the captured monitor (%s, matched by %s): captures "
                        "are tone-mapped and the minimap icons match worse. Turn it off in "
                        "Settings > System > Display > Use HDR, then run again.",
                        res["display"], res["matched_by"])
        else:
            log.info("Windows HDR off for the captured monitor (%s, matched by %s)",
                     res["display"], res["matched_by"])
    return res
