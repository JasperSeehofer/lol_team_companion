# SPDX-License-Identifier: AGPL-3.0-only
"""`python -m navgrid pc-run`: the one command on the PC.

Finds Map11.wad.client in one's own install, refuses while the game runs,
reads the WAD once, and writes %LOCALAPPDATA%\\navgrid\\latest\\ (the grid,
validation.json, manifest.json). Offline: it opens no socket.
"""

from __future__ import annotations

import datetime as _dt
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path, PureWindowsPath

from navgrid import __version__, ngrid, validate, wad

GAME_EXE = "League of Legends.exe"
WAD_REL = PureWindowsPath("DATA") / "FINAL" / "Maps" / "Shipping" / "Map11.wad.client"
SCP_LINE = 'scp -r "$env:LOCALAPPDATA\\navgrid\\latest" ops@10.44.0.1:.local/share/navgrid/inbox/'


class RunRefused(RuntimeError):
    pass


def default_wad_paths() -> list[PureWindowsPath]:
    """Install roots tried in order: with and without the Game folder, drives C to F."""
    out = []
    for drive in "CDEF":
        root = PureWindowsPath(f"{drive}:\\") / "Riot Games" / "League of Legends"
        out.append(root / "Game" / WAD_REL)
        out.append(root / WAD_REL)
    return out


def find_wad(override: str | None) -> Path:
    if override:
        p = Path(override)
        if not p.is_file():
            raise RunRefused(f"--wad {p} is not a file")
        return p
    if os.name == "nt":
        for p in default_wad_paths():
            if Path(p).is_file():
                return Path(p)
    raise RunRefused("Map11.wad.client not found under the default install roots; pass --wad PATH")


def game_running() -> bool:
    """True when a process named 'League of Legends.exe' runs. No psutil."""
    if os.name == "nt":
        res = subprocess.run(
            ["tasklist", "/FI", f"IMAGENAME eq {GAME_EXE}", "/NH", "/FO", "CSV"],
            capture_output=True, text=True, check=True,
        )
        return f'"{GAME_EXE.lower()}"' in res.stdout.lower()
    proc = Path("/proc")
    if proc.is_dir():
        for d in proc.iterdir():
            try:
                if GAME_EXE.lower() in (d / "comm").read_text().lower() or \
                   GAME_EXE.lower() in (d / "cmdline").read_bytes().decode("utf-8", "replace").lower():
                    return True
            except OSError:
                continue
    return False


def output_root() -> Path:
    base = os.environ.get("LOCALAPPDATA")
    if base:
        return Path(base) / "navgrid"
    return Path.home() / ".local" / "share" / "navgrid"


def inside_git_worktree(path: Path) -> bool:
    p = Path(path).resolve()
    for q in (p, *p.parents):
        if (q / ".git").exists():
            return True
    return False


def client_version(wad_path: Path) -> str | None:
    """The game's file version on Windows, else content-metadata.json nearby, else None."""
    for q in wad_path.resolve().parents:
        exe = q / GAME_EXE
        if exe.is_file() and os.name == "nt":
            v = _pe_version(exe)
            if v:
                return v
        meta = q / "content-metadata.json"
        if meta.is_file():
            try:
                return str(json.loads(meta.read_text(encoding="utf-8")).get("version")) or None
            except (OSError, ValueError):
                return None
    return None


def _pe_version(exe: Path) -> str | None:
    try:
        import ctypes

        ver = ctypes.windll.version  # type: ignore[attr-defined]
        size = ver.GetFileVersionInfoSizeW(str(exe), None)
        if not size:
            return None
        buf = ctypes.create_string_buffer(size)
        if not ver.GetFileVersionInfoW(str(exe), 0, size, buf):
            return None
        ptr, ln = ctypes.c_void_p(), ctypes.c_uint()
        if not ver.VerQueryValueW(buf, "\\", ctypes.byref(ptr), ctypes.byref(ln)):
            return None
        words = ctypes.cast(ptr, ctypes.POINTER(ctypes.c_uint32 * 13)).contents
        ms, ls = words[2], words[3]
        return f"{ms >> 16}.{ms & 0xFFFF}.{ls >> 16}.{ls & 0xFFFF}"
    except Exception:  # noqa: BLE001 - a version is nice to have, never required
        return None


def run(wad_override: str | None = None, out_root: Path | None = None, *,
        is_running=None, echo=print) -> Path:
    """Do the PC run. Returns the output directory. Raises RunRefused."""
    try:
        running = (is_running or game_running)()
    except Exception as e:  # noqa: BLE001
        raise RunRefused(f"could not check whether {GAME_EXE} runs ({e}); close the game and retry") from e
    if running:
        raise RunRefused(f"{GAME_EXE} is running: close the game (and the client) first")
    wad_path = find_wad(wad_override)
    root = Path(out_root) if out_root else output_root()
    if inside_git_worktree(root):
        raise RunRefused(f"refusing to write inside a git work tree: {root}")

    arc, wad_sha, handle = wad.open_wad(wad_path)
    try:
        found = arc.find_navgrid()
        wad_version = arc.version
    finally:
        wad.close_wad(handle)
    g = ngrid.parse(found.data)
    report = validate.validate(g)

    latest = root / "latest"
    if latest.exists():
        prev = root / "previous"
        if prev.exists():
            shutil.rmtree(prev)
        latest.rename(prev)
    latest.mkdir(parents=True)
    grid_name = Path(found.path).name if found.path else "sniffed.aimesh_ngrid"
    (latest / grid_name).write_bytes(found.data)
    st = wad_path.stat()
    manifest = {
        "tool": f"navgrid {__version__}",
        "created_utc": _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds"),
        "wad": {
            "path": str(wad_path),
            "sha256": wad_sha,
            "size": st.st_size,
            "mtime_utc": _dt.datetime.fromtimestamp(st.st_mtime, _dt.timezone.utc).isoformat(timespec="seconds"),
            "version": wad_version,
        },
        "entry": {
            "path": found.path,
            "hash": f"{found.entry.path_hash:016x}",
            "type": found.entry.type_name,
            "compressed_size": found.entry.compressed_size,
            "uncompressed_size": found.entry.uncompressed_size,
        },
        "find_method": found.method,
        "sniffed_hashes": found.sniffed,
        "grid": {
            "file": grid_name,
            "sha256": ngrid.sha256(found.data),
            "size": len(found.data),
            "version": g.version,
            "counts": [g.count_x, g.count_z],
            "cell_size": g.cell_size,
            "layers_parsed": g.layers_parsed,
            "parse_report": g.report,
        },
        "client_version": client_version(wad_path),
        "derived": "riot-navgrid; do not ship before ticket 2",
    }
    (latest / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    (latest / "validation.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    h = report["structure_wall_hypothesis"]
    lines = [
        f"navgrid {g.version}: {g.count_x} x {g.count_z} cells of {g.cell_size:g}, found by {found.method}",
        f"entry: {found.path or 'hash ' + manifest['entry']['hash']} ({found.entry.type_name})",
        f"layers: {', '.join(g.layers_parsed)}",
        f"structures: {report['structures_pass']}/{report['structures_total']} within "
        f"{validate.MAX_CELLS:g} cells; fountains walkable: {sum(report['fountains_walkable'].values())}/2",
        f"bit-4 hypothesis: {'supported' if h['supported'] else 'not supported'} "
        f"({h['structures_with_bit4_within_2_cells']}/{h['structures']} structures)",
        f"validation: {'PASS' if report['pass'] else 'FAIL (send it anyway)'}",
        f"written to {latest}",
        "next: send the folder to brain with this line:",
    ]
    for line in lines[:8]:
        echo(line)
    echo(SCP_LINE)
    return latest


def main_pc_run(args) -> int:
    try:
        run(args.wad, Path(args.out) if args.out else None)
    except (RunRefused, wad.WadError, ngrid.NgridError) as e:
        print(f"navgrid pc-run: {e}", file=sys.stderr)
        return 2
    return 0
