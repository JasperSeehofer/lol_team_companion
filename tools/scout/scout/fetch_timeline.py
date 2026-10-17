"""Fetch one match and its timeline from Riot's Match-V5 API.

  $env:RIOT_API_KEY = "RGAPI-..."   (PowerShell, this shell only)
  python -m scout.fetch_timeline --match-id EUW1_1234567890 --out RUN_DIR

The key is read from the environment only. It is never written to a file,
never logged and never printed; the run log records only the URLs.
Writes RUN_DIR/timeline.json and RUN_DIR/match.json.
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

import requests

from .util import run_logger, write_json

REGIONS = {"europe", "americas", "asia", "sea"}


def fetch(match_id: str, region: str, key: str, timeout: float = 20.0) -> tuple[dict, dict]:
    base = f"https://{region}.api.riotgames.com/lol/match/v5/matches/{match_id}"
    headers = {"X-Riot-Token": key}
    out = []
    for url in (base + "/timeline", base):
        r = requests.get(url, headers=headers, timeout=timeout)
        if r.status_code != 200:
            # Do not echo the response headers or the request: only status and URL.
            raise RuntimeError(f"GET {url}: HTTP {r.status_code}")
        out.append(r.json())
    return out[0], out[1]


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(prog="scout.fetch_timeline", description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--match-id", required=True, help="e.g. EUW1_1234567890")
    ap.add_argument("--region", default="europe", choices=sorted(REGIONS),
                    help="regional routing value (EUW/EUNE/TR/RU: europe)")
    ap.add_argument("--out", required=True, type=Path, help="run directory")
    args = ap.parse_args(argv)
    key = os.environ.get("RIOT_API_KEY", "").strip()
    if not key:
        print("RIOT_API_KEY is not set in this shell; set it for this command only "
              "($env:RIOT_API_KEY = '...') and run again.", file=sys.stderr)
        return 2
    log = run_logger(args.out, "fetch_timeline")
    log.info("fetching %s from %s (key from environment)", args.match_id, args.region)
    try:
        timeline, match = fetch(args.match_id, args.region, key)
    except (RuntimeError, requests.RequestException) as e:
        log.error("fetch failed: %s", str(e).replace(key, "<key>"))
        return 1
    write_json(args.out / "timeline.json", timeline)
    write_json(args.out / "match.json", match)
    log.info("timeline.json: %d frames; match.json: %d participants",
             len(timeline["info"]["frames"]), len(match["info"]["participants"]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
