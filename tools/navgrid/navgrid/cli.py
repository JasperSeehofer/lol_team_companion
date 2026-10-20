# SPDX-License-Identifier: AGPL-3.0-only
"""Command line: pc-run, validate, vectorise."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(prog="python -m navgrid", description=__doc__)
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("pc-run", help="read Map11.wad.client once and write %%LOCALAPPDATA%%\\navgrid\\latest")
    p.add_argument("--wad", help="path to Map11.wad.client (default: the install roots)")
    p.add_argument("--out", help="output root instead of %%LOCALAPPDATA%%\\navgrid")

    v = sub.add_parser("validate", help="numbers-only checks against public structure positions")
    v.add_argument("grid", type=Path)
    v.add_argument("--timeline", type=Path, help="a Match-V5 timeline JSON")

    z = sub.add_parser("vectorise", help="grid -> GeoJSON layers (needs the 'vector' extra)")
    z.add_argument("grid", type=Path)
    z.add_argument("--out", type=Path, help="output dir (default ~/.local/share/navgrid/out/<sha8>/)")

    args = ap.parse_args(argv)
    if args.cmd == "pc-run":
        from navgrid.pcrun import main_pc_run

        return main_pc_run(args)
    if args.cmd == "validate":
        from navgrid import ngrid, validate

        try:
            report = validate.validate_file(args.grid, args.timeline)
        except (OSError, ValueError, ngrid.NgridError) as e:
            print(f"navgrid validate: {e}", file=sys.stderr)
            return 2
        print(json.dumps(report, indent=2))
        return 0 if report["pass"] else 1
    if args.cmd == "vectorise":
        try:
            from navgrid.vectorise import main_vectorise
        except ImportError as e:
            print(f"navgrid vectorise needs the 'vector' extra (pip install -e tools/navgrid[vector]): {e}",
                  file=sys.stderr)
            return 2
        return main_vectorise(args)
    return 2
