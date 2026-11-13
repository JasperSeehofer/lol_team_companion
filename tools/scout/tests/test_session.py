"""The session driver (M166 A4): every verb on temp dirs, the mock Replay API
and the synthetic fixtures. No network, no Riot key, HOME and TEMP redirected."""

import shutil
import types

import pytest

import make_fixtures as mf
from scout import fetch_timeline, probe, session
from scout.util import read_json, write_json

KEY = "RG" + "API-00000000-test-not-a-real-key"
MID = "EUW1_7100000099"


@pytest.fixture
def env(tmp_path, monkeypatch):
    monkeypatch.setenv("HOME", str(tmp_path / "home"))
    monkeypatch.setenv("USERPROFILE", str(tmp_path / "home"))
    monkeypatch.setenv("TEMP", str(tmp_path / "temp"))
    monkeypatch.delenv("RIOT_API_KEY", raising=False)
    monkeypatch.delenv("SCOUT_RUNS_DIR", raising=False)
    runs, models = tmp_path / "runs", tmp_path / "models"
    base = ["--runs-dir", str(runs), "--models-dir", str(models)]

    def call(*argv):
        return session.main([*base, *argv])

    return types.SimpleNamespace(call=call, runs=runs, models=models, tmp=tmp_path)


def started(env):
    assert env.call("start", MID, "--date", "2026-10-21") == 0
    return env.runs / f"2026-10-21-{MID}"


def test_start_creates_run_current_and_models(env, capsys):
    run = started(env)
    assert run.is_dir() and env.models.is_dir()
    assert (env.runs / ".current").read_text().strip() == run.name
    assert session.match_id_of(run) == MID
    assert str(run) in capsys.readouterr().out
    assert env.call("start", MID, "--date", "2026-10-21") == 0       # idempotent
    assert env.call("start", "not-a-match") == 2


def test_verbs_need_a_current_run(env, capsys):
    assert env.call("probe") == 2
    assert "session start" in capsys.readouterr().err


def test_unknown_arguments_refused_where_nothing_is_passed_on(env):
    started(env)
    assert env.call("check", "--bogus") == 2


CFG = "[General]\r\nWindowMode=2\r\nEnableReplayApi=0\r\n[HUD]\r\nEnableReplayApi=1\r\n"


def test_enable_api_sets_general_idempotent_with_backup(env, tmp_path):
    cfg = tmp_path / "game.cfg"
    cfg.write_bytes(CFG.encode())
    assert env.call("enable-api", "--cfg", str(cfg)) == 0
    after = cfg.read_bytes().decode()
    assert after == "[General]\r\nWindowMode=2\r\nEnableReplayApi=1\r\n[HUD]\r\nEnableReplayApi=1\r\n"
    assert (tmp_path / "game.cfg.bak").read_bytes() == CFG.encode()
    assert env.call("enable-api", "--cfg", str(cfg)) == 0             # second run: no change
    assert cfg.read_bytes().decode() == after
    assert (tmp_path / "game.cfg.bak").read_bytes() == CFG.encode()


def test_enable_api_section_respected(tmp_path):
    cfg = tmp_path / "game.cfg"
    cfg.write_text("[HUD]\nEnableReplayApi=1\n[General]\nWindowMode=2\n[Sound]\nVolume=1\n")
    assert "added EnableReplayApi=1 under [General]" in session.enable_replay_api(cfg)
    assert cfg.read_text() == ("[HUD]\nEnableReplayApi=1\n[General]\nEnableReplayApi=1\n"
                               "WindowMode=2\n[Sound]\nVolume=1\n")
    assert session.enable_replay_api(cfg).startswith("already enabled")
    cfg2 = tmp_path / "other.cfg"
    cfg2.write_text("[HUD]\nScale=1\n")
    session.enable_replay_api(cfg2)
    assert cfg2.read_text() == "[HUD]\nScale=1\n\n[General]\nEnableReplayApi=1\n"


def test_enable_api_missing_file(env, tmp_path, capsys):
    assert env.call("enable-api", "--cfg", str(tmp_path / "nope.cfg")) == 2
    assert "--cfg" in capsys.readouterr().err


def test_probe_against_the_mock(env, mock_api, monkeypatch):
    run = started(env)
    import time as real_time

    monkeypatch.setattr(probe, "time", types.SimpleNamespace(sleep=lambda s: None, time=real_time.time))
    monkeypatch.setattr(probe, "screen_grabber", lambda *a, **k: None)   # never the real screen
    assert env.call("probe", "--base", mock_api.base, "--skip-recording") == 0
    res = read_json(run / "probe.json")
    assert res["spec"]["answered"] == "/swagger/v3/openapi.json"
    assert (run / "openapi.json").is_file() and (run / "run.log").is_file()


def test_minimap_offline_frames(env, fixtures):
    run = started(env)
    assert env.call("minimap", "--frames", str(fixtures / "frames"), "--icons", str(fixtures / "icons"),
                    "--champions", ",".join(mf.CHAMPIONS)) == 0
    track = read_json(run / "track_minimap.json")
    assert track["source"] == "minimap" and len(track["samples"]) == 10
    assert (run / "frames_minimap").is_dir()


def test_topdown_offline_frames(env, tmp_path):
    run = started(env)
    frames = mf.write_topdown_series(tmp_path / "series")
    write_json(tmp_path / "anchors.json", mf.td_anchors())
    assert env.call("topdown", "--frames", str(frames), "--anchors",
                    str(tmp_path / "anchors.json"), "--champions", ",".join(mf.CHAMPIONS)) == 0
    assert read_json(run / "track_topdown.json")["source"] == "topdown"


class FakeResponse:
    def __init__(self, data, status=200):
        self.data, self.status_code = data, status

    def json(self):
        return self.data


def test_timeline_key_from_environment_only(env, fixtures, monkeypatch, capsys):
    run = started(env)
    assert env.call("timeline") == 2                                # no key in the environment
    monkeypatch.setenv("RIOT_API_KEY", KEY)
    seen = []

    def fake_get(url, headers, timeout):
        seen.append(url)
        if url.endswith("/timeline"):
            return FakeResponse(read_json(fixtures / "timeline_synth.json"))
        return FakeResponse(read_json(fixtures / "match_synth.json"))

    monkeypatch.setattr(fetch_timeline.requests, "get", fake_get)
    assert env.call("timeline") == 0
    assert seen[0].endswith(f"/matches/{MID}/timeline")
    out = capsys.readouterr()
    assert KEY not in out.out + out.err
    for p in run.rglob("*"):
        if p.is_file():
            assert KEY not in p.read_text(encoding="utf-8", errors="replace")


def copy_inputs(fixtures, run):
    for name in ("track_minimap.json", "track_topdown.json", "timeline.json", "match.json"):
        shutil.copy(fixtures / "run_synth" / name, run / name)


def test_score_writes_handcheck_ward_frames_and_optional_plots(env, fixtures):
    run = started(env)
    copy_inputs(fixtures, run)
    scratch = env.tmp / "scratch"
    scratch.mkdir()
    for p in (fixtures / "run_synth" / "frames_minimap").iterdir():
        shutil.copy(p, scratch / p.name)
    assert env.call("score", "--scratch", str(scratch)) == 0
    assert (run / "score.json").is_file() and (run / "handcheck.csv").is_file()
    assert len(list((run / "frames_minimap").glob("t*.png"))) == 20    # 4 events x 5 seconds
    assert not (run / "plots").exists()
    assert env.call("score", "--plots", "--scratch", str(scratch)) == 0
    assert (run / "plots" / "error_per_champion.png").is_file()


def test_score_needs_inputs(env):
    started(env)
    assert env.call("score") == 2


def test_handcheck_opens_the_csv(env, fixtures, monkeypatch):
    run = started(env)
    assert env.call("handcheck") == 2
    shutil.copy(fixtures / "run_synth" / "handcheck.csv", run / "handcheck.csv")
    opened = []
    monkeypatch.setattr(session, "open_in_editor", opened.append)
    assert env.call("handcheck") == 0
    assert opened == [run / "handcheck.csv"]


def test_check_is_the_manifest_check(env, fixtures):
    run = started(env)
    shutil.rmtree(run)
    shutil.copytree(fixtures / "run_synth", run)
    assert env.call("check") == 0
    (run / "notes.txt").write_text("key " + KEY)
    assert env.call("check") == 1


def test_send_dry_run_prints_the_exact_scp(env, fixtures, monkeypatch, capsys):
    run = started(env)
    shutil.rmtree(run)
    shutil.copytree(fixtures / "run_synth", run)
    calls = []
    monkeypatch.setattr(session.subprocess, "run", lambda cmd, check: calls.append(cmd))
    assert env.call("send", "--dry-run") == 0
    out = capsys.readouterr().out
    assert f"scp -r {run} ops@10.44.0.1:.local/share/scout/inbox/" in out
    assert calls == []
    monkeypatch.setattr(session.subprocess, "run",
                        lambda cmd, check: calls.append(cmd) or types.SimpleNamespace(returncode=0))
    assert env.call("send") == 0
    assert calls == [["scp", "-r", str(run), "ops@10.44.0.1:.local/share/scout/inbox/"]]


def test_send_refuses_a_git_work_tree_and_unsafe_runs(env, fixtures, monkeypatch, capsys):
    run = started(env)
    shutil.rmtree(run)
    shutil.copytree(fixtures / "run_synth", run)
    monkeypatch.setattr(session.subprocess, "run", lambda *a, **k: pytest.fail("must not copy"))
    repo = env.tmp / "repo"
    (repo / ".git").mkdir(parents=True)
    assert env.call("send", "--dest", str(repo / "tools" / "scout" / "runs")) == 2
    assert "git work tree" in capsys.readouterr().err
    (run / "rec.webm").write_bytes(b"\0")
    assert env.call("send", "--dry-run") == 2
    assert "video" in capsys.readouterr().err


def test_dest_classification():
    assert session.is_remote("ops@10.44.0.1:.local/share/scout/inbox/")
    assert session.is_remote("brain:inbox/")
    assert not session.is_remote("C:\\Users\\x\\inbox")
    assert not session.is_remote("/home/ops/inbox")
