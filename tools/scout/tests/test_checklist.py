"""CHECKLIST.md (M166 A4): one page, every step marked, the Wednesday commands verbatim."""

import re
from pathlib import Path

CHECKLIST = Path(__file__).resolve().parents[1] / "CHECKLIST.md"
PY = r".\tools\scout\.venv\Scripts\python -m scout.session"
# The M166 page's "Jasper's Wednesday", steps 6-22 (the Scout run), as typed.
WEDNESDAY = [
    "git switch mission/M156-scout-replay-spike; git pull --ff-only",
    r'py -3.12 -m venv tools\scout\.venv; .\tools\scout\.venv\Scripts\pip install -e "tools/scout[yolo]"',
    f"{PY} start EUW1_<gameId>",
    r"scp ops@10.44.0.1:.cache/scout/models/yolov12.pt tools\scout\models\yolov12.pt",
    f"{PY} enable-api", f"{PY} probe", f"{PY} minimap", f"{PY} topdown",
    '$env:RIOT_API_KEY = Read-Host "Riot key"', f"{PY} timeline", "Remove-Item Env:RIOT_API_KEY",
    f"{PY} score", f"{PY} handcheck", f"{PY} check", f"{PY} send",
]


def test_one_page_every_step_marked():
    text = CHECKLIST.read_text(encoding="utf-8")
    assert len(text.splitlines()) <= 60
    steps = re.findall(r"^\s*(\d+)\. (.*)$", text, re.M)
    assert [int(n) for n, _ in steps] == list(range(1, 14))
    for n, line in steps:
        assert line.startswith(("**PC-only.", "**Pre-done on brain.")), f"step {n} unmarked"


def test_wednesday_commands_verbatim_and_safe():
    text = CHECKLIST.read_text(encoding="utf-8")
    for cmd in WEDNESDAY:
        assert cmd in text, cmd
    assert "RG" + "API" not in text
    assert "git push" not in text and "git add" not in text
    assert "Google Drive" in text and "production key" in text


# ---------------------------------------------------------------- CHECKLIST-VALIDATE.md (M156 half C)

VALIDATE = Path(__file__).resolve().parents[1] / "CHECKLIST-VALIDATE.md"
SPY = r"function spy { .\tools\scout\.venv\Scripts\python -m scout.session @args }"


def test_validate_checklist_short_every_step_has_output_and_failure():
    text = VALIDATE.read_text(encoding="utf-8")
    assert len(text.splitlines()) <= 40
    steps = re.split(r"^\s*\d+\. ", text, flags=re.M)[1:]
    nums = [int(n) for n in re.findall(r"^\s*(\d+)\. ", text, re.M)]
    assert nums == list(range(1, len(nums) + 1)) and len(nums) >= 8
    for n, body in zip(nums, steps):
        if "sign out" in body.lower() or "open the replay" in body.lower():
            continue   # pure hand steps
        assert "Output:" in body, f"step {n} names no output"
    assert text.count("If it fails") >= 6


def test_validate_checklist_commands_exist_and_safe():
    from scout import session

    text = VALIDATE.read_text(encoding="utf-8")
    assert SPY in text
    verbs = set(re.findall(r"`spy ([a-z-]+)", text))
    assert verbs and verbs <= set(session.VERBS), verbs - set(session.VERBS)
    for v in ("start", "timeline", "minimap", "topdown", "score", "handcheck", "check", "send"):
        assert v in verbs, v
    # flags named in the checklist exist on the module the verb wraps
    import contextlib
    import io

    from scout import track_minimap, track_topdown
    for mod, flag in ((track_minimap, "--minimap-box"), (track_topdown, "--keystroke-unlock")):
        assert flag in text
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf), contextlib.suppress(SystemExit):
            mod.main(["--help"])
        assert flag in buf.getvalue(), flag
    assert "RG" + "API" not in text and "production key" in text
    assert "git push" not in text and "HDR" in text
