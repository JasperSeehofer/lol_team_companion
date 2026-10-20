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
