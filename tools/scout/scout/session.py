"""Session driver for the Scout run on the player's PC (M166 A4).

  python -m scout.session start EUW1_<gameId>   create the run dir, remember it as current
  python -m scout.session enable-api [--cfg PATH]   EnableReplayApi=1 under [General] in game.cfg
  python -m scout.session probe        scout.probe --out RUN
  python -m scout.session minimap      scout.track_minimap --out RUN --speed 8 [--weights ...]
  python -m scout.session topdown      scout.track_topdown --out RUN
  python -m scout.session timeline     scout.fetch_timeline (RIOT_API_KEY from the environment)
  python -m scout.session score [--plots]   scout.score run (+ plots); writes handcheck.csv
  python -m scout.session handcheck    open handcheck.csv in Notepad
  python -m scout.session check        the half-C manifest check (size, video, key, identifiers)
  python -m scout.session send [--dry-run]   scp the run dir to brain's inbox (never GitHub)

The current run is kept in tools/scout/runs/.current, so nothing has to be
typed again after a Windows sign-out. Every verb wraps the existing module with
its file names unchanged; options after the verb that the driver does not know
are passed on to that module (e.g. `session minimap --minimap-box 1600,820,300,300`).
tools/scout/runs/ is git-ignored: run data never goes to GitHub.
"""

from __future__ import annotations

import argparse
import os
import re
import shlex
import subprocess
import sys
import tempfile
from datetime import date
from pathlib import Path

PKG_ROOT = Path(__file__).resolve().parents[1]          # tools/scout
RUNS_DIR = PKG_ROOT / "runs"
MODELS_DIR = PKG_ROOT / "models"
DEFAULT_CFG = Path(r"C:\Riot Games\League of Legends\Config\game.cfg")
DEFAULT_DEST = "ops@10.44.0.1:.local/share/scout/inbox/"
MATCH_ID = re.compile(r"^[A-Z]{2,4}\d?_\d{5,}$")
WARD_FRAMES_PER_TEAM = 6
WARD_FRAMES_BUDGET_MB = 45.0


class SessionError(RuntimeError):
    pass


# ---------------------------------------------------------------- current run

def runs_dir(args) -> Path:
    return Path(args.runs_dir or os.environ.get("SCOUT_RUNS_DIR") or RUNS_DIR)


def current_run(args) -> Path:
    marker = runs_dir(args) / ".current"
    if not marker.is_file():
        raise SessionError("no current run: start one with "
                           "python -m scout.session start EUW1_<gameId>")
    run = runs_dir(args) / marker.read_text(encoding="utf-8").strip()
    if not run.is_dir():
        raise SessionError(f"the current run {run} does not exist; run start again")
    return run


def match_id_of(run: Path) -> str:
    return run.name.split("-", 3)[-1]


def cmd_start(args, extra) -> int:
    mid = args.match_id.strip().upper()
    if not MATCH_ID.match(mid):
        raise SessionError(f"'{args.match_id}' is not a match id like EUW1_7100000099")
    day = args.date or date.today().isoformat()
    run = runs_dir(args) / f"{day}-{mid}"
    run.mkdir(parents=True, exist_ok=True)
    (runs_dir(args) / ".current").write_text(run.name + "\n", encoding="utf-8")
    Path(args.models_dir or MODELS_DIR).mkdir(parents=True, exist_ok=True)
    print(f"current run: {run}")
    return 0


# ---------------------------------------------------------------- game.cfg

def enable_replay_api(cfg: Path) -> str:
    """Set EnableReplayApi=1 in the [General] section. Idempotent; writes
    game.cfg.bak (the file as it was) before any change."""
    raw = cfg.read_bytes()
    text = raw.decode("utf-8", "surrogateescape")
    nl = "\r\n" if "\r\n" in text else "\n"
    trailing = text.endswith(("\n", "\r"))
    lines = text.splitlines()
    head = next((i for i, ln in enumerate(lines) if ln.strip().lower() == "[general]"), None)
    if head is None:
        if lines and lines[-1].strip():
            lines.append("")
        lines += ["[General]", "EnableReplayApi=1"]
        status = "added a [General] section with EnableReplayApi=1"
    else:
        end = next((i for i in range(head + 1, len(lines)) if lines[i].strip().startswith("[")),
                   len(lines))
        key = next((i for i in range(head + 1, end)
                    if re.match(r"\s*EnableReplayApi\s*=", lines[i], re.I)), None)
        if key is not None and lines[key].split("=", 1)[1].strip() == "1":
            return "already enabled (EnableReplayApi=1 under [General]); nothing changed"
        if key is not None:
            lines[key] = "EnableReplayApi=1"
            status = "set EnableReplayApi=1 under [General]"
        else:
            lines.insert(head + 1, "EnableReplayApi=1")
            status = "added EnableReplayApi=1 under [General]"
    cfg.with_name(cfg.name + ".bak").write_bytes(raw)
    out = nl.join(lines) + (nl if trailing or head is None else "")
    cfg.write_bytes(out.encode("utf-8", "surrogateescape"))
    return f"{status}; backup {cfg.name}.bak; restart the League client"


def cmd_enable_api(args, extra) -> int:
    cfg = Path(args.cfg or DEFAULT_CFG)
    if not cfg.is_file():
        raise SessionError(f"{cfg} not found; pass --cfg with the path of your game.cfg "
                           "(League of Legends\\Config\\game.cfg)")
    print(enable_replay_api(cfg))
    return 0


# ---------------------------------------------------------------- module verbs

def cmd_probe(args, extra) -> int:
    from . import probe

    return probe.main(["--out", str(current_run(args)), *extra])


def cmd_minimap(args, extra) -> int:
    from . import track_minimap

    run = current_run(args)
    argv = ["--out", str(run), "--speed", "8"]
    weights = Path(args.models_dir or MODELS_DIR) / "yolov12.pt"
    if weights.is_file() and "--weights" not in extra:
        argv += ["--weights", str(weights)]
    if (run / "match.json").is_file() and "--match" not in extra and "--champions" not in extra:
        argv += ["--match", str(run / "match.json")]
    return track_minimap.main(argv + extra)


def cmd_topdown(args, extra) -> int:
    from . import track_topdown

    return track_topdown.main(["--out", str(current_run(args)), *extra])


def cmd_timeline(args, extra) -> int:
    from . import fetch_timeline

    run = current_run(args)
    # The key is read by fetch_timeline from RIOT_API_KEY; the driver never touches it.
    return fetch_timeline.main(["--match-id", match_id_of(run), "--out", str(run), *extra])


def copy_ward_frames(run: Path, scratch: Path, per_team: int = WARD_FRAMES_PER_TEAM) -> int:
    """Copy minimap frames +-2 s around WARD_PLACED events (up to per_team
    events per team) from the capture scratch dir into RUN/frames_minimap/,
    for half C's fact-6 eye check. Stops before the run dir passes the budget."""
    import cv2

    from . import score
    from .util import list_frames, read_json, sample_evenly

    tl = run / "timeline.json"
    if not (tl.is_file() and scratch.is_dir()):
        return 0
    have = {int(t): p for t, p in list_frames(scratch)}
    evs = [e for e in score.events(read_json(tl), "WARD_PLACED")
           if e.get("wardType") not in score.SKIP_WARD_TYPES and e.get("creatorId", 0) > 0]
    keep = run / "frames_minimap"
    keep.mkdir(exist_ok=True)
    size = sum(p.stat().st_size for p in run.rglob("*") if p.is_file())
    n = 0
    for team in ("blue", "red"):
        team_evs = [e for e in evs if score.team_of(e["creatorId"]) == team]
        for e in sample_evenly(team_evs, per_team):
            t0 = int(round(e["timestamp"] / 1000))
            for d in (-2, -1, 0, 1, 2):
                src, dst = have.get(t0 + d), keep / f"t{t0 + d:05d}.png"
                if src is None or dst.exists():
                    continue
                if size / 2**20 > WARD_FRAMES_BUDGET_MB:
                    return n
                cv2.imwrite(str(dst), cv2.imread(str(src)), [cv2.IMWRITE_PNG_COMPRESSION, 9])
                size += dst.stat().st_size
                n += 1
    return n


def cmd_score(args, extra) -> int:
    from . import score
    from .util import list_frames

    run = current_run(args)
    for need in ("track_minimap.json", "timeline.json", "match.json"):
        if not (run / need).is_file():
            raise SessionError(f"{need} missing in {run}; run the earlier steps first")
    argv = ["run", "--track", str(run / "track_minimap.json"), "--timeline", str(run / "timeline.json"),
            "--match", str(run / "match.json"), "--out", str(run)]
    if (run / "track_topdown.json").is_file():
        argv += ["--wards-track", str(run / "track_topdown.json")]
    rc = score.main(argv + extra)
    scratch = Path(args.scratch or Path(os.environ.get("TEMP", tempfile.gettempdir())) / "scout-frames")
    n = copy_ward_frames(run, scratch, args.ward_frames)
    print(f"{n} minimap frames around WARD_PLACED copied into frames_minimap (fact 6)")
    if args.plots and rc == 0:
        frames = list_frames(run / "frames_minimap") if (run / "frames_minimap").is_dir() else []
        pargv = ["plots", "--score", str(run / "score.json"), "--out", str(run / "plots")]
        if frames:
            pargv += ["--minimap", str(frames[0][1])]
        rc = score.main(pargv)
    print(f"next: python -m scout.session handcheck  (mark jasper_says yes/no, save)")
    return rc


def open_in_editor(path: Path) -> None:
    if sys.platform.startswith("win"):
        subprocess.Popen(["notepad.exe", str(path)])
    else:
        print(f"open {path} in an editor")


def cmd_handcheck(args, extra) -> int:
    p = current_run(args) / "handcheck.csv"
    if not p.is_file():
        raise SessionError("handcheck.csv missing; run python -m scout.session score first")
    open_in_editor(p)
    print(f"{p}: write yes or no in jasper_says per row, then save as CSV")
    return 0


# ---------------------------------------------------------------- check and send

def cmd_check(args, extra) -> int:
    from .halfc import manifest_check, manifest_line

    run = current_run(args)
    m = manifest_check(run)
    print(f"{run.name}: {manifest_line(m)}")
    if m["ok"]:
        print("next: python -m scout.session send")
    return 0 if m["ok"] else 1


def is_remote(dest: str) -> bool:
    if re.match(r"^[A-Za-z]:[\\/]", dest):      # a Windows drive path
        return False
    return bool(re.match(r"^[^/\\:]+:", dest))


def inside_git_work_tree(path: Path) -> Path | None:
    p = path.resolve()
    for cand in (p, *p.parents):
        if (cand / ".git").exists():
            return cand
    return None


def cmd_send(args, extra) -> int:
    from .halfc import manifest_check

    run = current_run(args)
    dest = args.dest or DEFAULT_DEST
    if not is_remote(dest):
        repo = inside_git_work_tree(Path(dest))
        if repo is not None:
            raise SessionError(f"refusing to send run data into a git work tree ({repo}): "
                               "run data never goes to git")
    m = manifest_check(run)
    if not m["ok"]:
        raise SessionError("refusing to send: " + "; ".join(m["problems"]) +
                           " (python -m scout.session check)")
    cmd = ["scp", "-r", str(run), dest]
    print(" ".join(shlex.quote(c) for c in cmd))
    print(f"(lands as {dest.rstrip('/')}/{run.name}/)")
    if args.dry_run:
        return 0
    return subprocess.run(cmd, check=False).returncode


# ---------------------------------------------------------------- CLI

VERBS = {"start": cmd_start, "enable-api": cmd_enable_api, "probe": cmd_probe,
         "minimap": cmd_minimap, "topdown": cmd_topdown, "timeline": cmd_timeline,
         "score": cmd_score, "handcheck": cmd_handcheck, "check": cmd_check, "send": cmd_send}


NO_PASSTHROUGH = {"start", "enable-api", "handcheck", "check", "send"}


def parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(prog="scout.session", description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--runs-dir", help="default tools/scout/runs (or $SCOUT_RUNS_DIR)")
    ap.add_argument("--models-dir", help="default tools/scout/models")
    sub = ap.add_subparsers(dest="verb", required=True)
    s = sub.add_parser("start", help="create the run dir and make it current")
    s.add_argument("match_id", help="EUW1_<gameId>")
    s.add_argument("--date", help="YYYY-MM-DD (default today)")
    e = sub.add_parser("enable-api", help="EnableReplayApi=1 under [General] in game.cfg")
    e.add_argument("--cfg", help=f"default {DEFAULT_CFG}")
    for v in ("probe", "minimap", "topdown", "timeline", "handcheck", "check"):
        sub.add_parser(v)
    sc = sub.add_parser("score", help="score run, handcheck.csv, ward frames; --plots for plots")
    sc.add_argument("--plots", action="store_true", help="also draw the plots (optional)")
    sc.add_argument("--ward-frames", type=int, default=WARD_FRAMES_PER_TEAM,
                    help="WARD_PLACED events per team whose +-2 s minimap frames are kept")
    sc.add_argument("--scratch", help="minimap capture scratch dir (default %%TEMP%%/scout-frames)")
    se = sub.add_parser("send", help=f"scp the run dir to {DEFAULT_DEST}")
    se.add_argument("--dest", help=f"default {DEFAULT_DEST}")
    se.add_argument("--dry-run", action="store_true", help="print the command, copy nothing")
    return ap


def main(argv=None) -> int:
    args, extra = parser().parse_known_args(argv)
    if extra and extra[0] == "--":
        extra = extra[1:]
    if extra and args.verb in NO_PASSTHROUGH:
        print(f"scout.session {args.verb}: unknown arguments {' '.join(extra)}", file=sys.stderr)
        return 2
    try:
        return VERBS[args.verb](args, extra)
    except SessionError as e:
        print(f"scout.session {args.verb}: {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
