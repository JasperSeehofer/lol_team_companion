"""Guard (M166 A4): no run data and no weights are tracked, and both are ignored."""

import shutil
import subprocess
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[3]


def git(*args):
    return subprocess.run(["git", "-C", str(REPO), *args], capture_output=True, text=True, check=False)


@pytest.fixture(autouse=True)
def need_git():
    if shutil.which("git") is None or git("rev-parse", "--is-inside-work-tree").stdout.strip() != "true":
        pytest.skip("not a git work tree")


def test_no_run_data_or_weights_tracked():
    files = git("ls-files").stdout.splitlines()
    runs = [f for f in files if f.startswith("tools/scout/runs/") and f != "tools/scout/runs/.gitkeep"]
    assert runs == [], f"run data tracked: {runs[:5]}"
    assert [f for f in files if f.endswith(".pt")] == []
    assert [f for f in files if f.startswith("tools/scout/models/")] == []


def test_runs_and_models_are_ignored():
    paths = ["tools/scout/runs/2026-10-21-EUW1_1/match.json", "tools/scout/runs/.current",
             "tools/scout/models/yolov12.pt", "tools/scout/yolov12.pt"]
    res = git("check-ignore", "--no-index", *paths)
    assert res.stdout.split() == paths
