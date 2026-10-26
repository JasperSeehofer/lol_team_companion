"""The half-C runner (M166 A1) on the synthetic run dir tests/fixtures/run_synth."""

import csv
import json
import re
import shutil
import statistics

import numpy as np
import pytest

import make_fixtures as mf
from scout import findings, halfc, rules
from scout.util import read_json

EXPECTED_OUTPUTS = [
    "facts.md", "halfc-summary.json", "handcheck_merged.csv", "scout-findings.md",
    "plots/fact6_eye_check.png", "plots/topdown_strip.png", "plots/wards_precision_recall.png",
    "plots/minimap/error_per_champion.png", "plots/topdown/error_per_champion.png",
    "score/score_minimap.json", "score/score_topdown.json",
]


@pytest.fixture
def run(fixtures, tmp_path):
    dst = tmp_path / "2026-10-21-EUW1_7100000099"
    shutil.copytree(fixtures / "run_synth", dst)
    return dst


def tree(root):
    return {p.relative_to(root).as_posix(): p.read_bytes() for p in sorted(root.rglob("*")) if p.is_file()}


def test_run_synth_is_small_and_regenerates_byte_identical(fixtures, tmp_path):
    committed = fixtures / "run_synth"
    assert sum(len(b) for b in tree(committed).values()) <= 2 * 2**20
    mf.make_run_synth(tmp_path / "again")
    assert tree(tmp_path / "again") == tree(committed)


def test_end_to_end(run, tmp_path, capsys):
    out, fpath = tmp_path / "out", tmp_path / "docs" / "scout-findings.md"
    assert halfc.main([str(run), "--out", str(out), "--findings", str(fpath)]) == 0
    for name in EXPECTED_OUTPUTS:
        assert (out / name).is_file(), name
    assert len(list((out / "plots" / "minimap").glob("overlay_*.png"))) == 3
    text = fpath.read_text(encoding="utf-8")
    assert text == (out / "scout-findings.md").read_text(encoding="utf-8")
    assert findings.unfilled_lines(text) == []
    assert "agent judgment" in text
    s = read_json(out / "halfc-summary.json")
    assert s["manifest"]["ok"] and s["manifest"]["missing"] == {}
    assert [r["n"] for r in s["facts"]] == [1, 2, 3, 4, 5, 6]
    assert all("not reached" not in r["value"] for r in s["facts"])
    assert s["eye_check"]["events"] == 4
    assert s["handcheck"]["agree"] == 4 and s["handcheck"]["marked_rows"] == 5
    merged = list(csv.DictReader(open(out / "handcheck_merged.csv", encoding="utf-8")))
    assert [r["agree"] for r in merged] == ["yes", "yes", "yes", "no", "yes"]
    assert "route A+B" in capsys.readouterr().out
    raw = (out / "halfc-summary.json").read_text(encoding="utf-8")
    assert not re.search(r"\d{4}-\d\d-\d\d[T ]\d\d:\d\d", raw)   # no wall-clock timestamps


def test_idempotent_byte_identical(run, tmp_path):
    out = tmp_path / "out"
    halfc.main([str(run), "--out", str(out)])
    first = tree(out)
    halfc.main([str(run), "--out", str(out)])
    assert tree(out) == first
    halfc.main([str(run), "--out", str(tmp_path / "elsewhere")])
    assert tree(tmp_path / "elsewhere") == first


def test_findings_default_never_touches_docs_when_out_given(run, tmp_path, monkeypatch):
    monkeypatch.setattr(halfc, "DEFAULT_FINDINGS", tmp_path / "must-not-exist.md")
    halfc.main([str(run), "--out", str(tmp_path / "out")])
    assert not (tmp_path / "must-not-exist.md").exists()


def test_go_no_go_matches_the_synthetic_numbers(run, tmp_path):
    """Recompute the minimap median and detection rate from the inputs and check
    every verdict and the route against thresholds.json by hand."""
    out = tmp_path / "out"
    halfc.main([str(run), "--out", str(out)])
    s = read_json(out / "halfc-summary.json")
    th = rules.load_thresholds()
    track, tl = read_json(run / "track_minimap.json"), read_json(run / "timeline.json")
    pmap = {p["participantId"]: p["championName"] for p in read_json(run / "match.json")["info"]["participants"]}
    look = {smp["t"]: smp["pos"] for smp in track["samples"]}
    errs = []
    for f in tl["info"]["frames"]:
        pos = look[f["timestamp"] / 1000]
        for pid, pf in f["participantFrames"].items():
            got = pos[pmap[int(pid)]]
            if got is not None:
                errs.append(round(float(np.hypot(got[0] - pf["position"]["x"], got[1] - pf["position"]["y"])), 1))
    a = s["positions_minimap"]
    assert a["raw"]["median_error"] == statistics.median(errs)
    assert a["raw"]["n_scored"] == len(errs) == 70
    # Garen's 19 s death gap is the only gap; alive-adjusted it is excluded
    assert a["raw"]["longest_gap_s"] == 19.0 and a["alive_adjusted"]["longest_gap_s"] == 0.0
    assert a["alive_adjusted"]["detection_rate"] == 1.0
    go = th["positions_minimap"]["go"]
    assert a["raw"]["median_error"] <= go["median_error_max"]
    assert a["verdict"]["verdict"] == "GO"
    b = s["positions_topdown"]
    assert b["input"]["bar_hit_rate"] == 0.9 and b["input"]["median_error"] <= 150
    assert b["verdict"]["verdict"] == "GO"
    w = s["wards"]["numbers"]
    assert (w["blue"]["precision"], w["blue"]["recall"]) == (1.0, 1.0)
    assert (w["red"]["tp"], w["red"]["detected"], w["red"]["placed"]) == (2, 3, 2)
    # blue passes the numbers but the hand check is 3/4 < 4/5; red fails precision too
    assert s["wards"]["verdicts"]["blue"]["verdict"] == "NO-GO"
    assert "hand check disagrees" in s["wards"]["verdicts"]["blue"]["reason"]
    assert s["wards"]["verdicts"]["red"]["verdict"] == "NO-GO"
    r = s["recalls"]["numbers"]
    assert (r["detected"], r["confirmed"], r["bursts"], r["found"]) == (1, 1, 1, 1)
    assert s["recalls"]["verdict"]["verdict"] == "GO"
    assert s["realtime_only"] is False
    assert s["route"]["route"] == "A+B"


def test_degraded_no_topdown(run, tmp_path):
    (run / "track_topdown.json").unlink()
    shutil.rmtree(run / "frames_topdown")
    out = tmp_path / "out"
    assert halfc.main([str(run), "--out", str(out)]) == 0
    s = read_json(out / "halfc-summary.json")
    assert s["positions_topdown"]["verdict"]["verdict"] == "NOT REACHED"
    assert "route B not reached" in s["manifest"]["missing"]["track_topdown.json"]
    assert s["wards"]["verdicts"]["blue"]["verdict"] == "NOT REACHED"
    assert s["route"]["route"] == "A-only" and "B not reached" in s["route"]["reason"]
    text = (out / "scout-findings.md").read_text(encoding="utf-8")
    assert findings.unfilled_lines(text) == []
    assert "not reached" in text.lower()


def test_degraded_no_handcheck(run, tmp_path):
    (run / "handcheck.csv").unlink()
    out = tmp_path / "out"
    assert halfc.main([str(run), "--out", str(out)]) == 0
    s = read_json(out / "halfc-summary.json")
    assert s["handcheck"] == {"marked": False, "reason": "not marked: handcheck.csv missing"}
    assert s["wards"]["verdicts"]["blue"]["verdict"] == "PENDING"     # numbers pass, no hand check
    assert s["recalls"]["verdict"]["verdict"] == "PENDING"
    text = (out / "scout-findings.md").read_text(encoding="utf-8")
    assert "not marked" in text and findings.unfilled_lines(text) == []


def test_degraded_unmarked_handcheck(run, tmp_path):
    rows = list(csv.DictReader(open(run / "handcheck.csv", encoding="utf-8")))
    with open(run / "handcheck.csv", "w", newline="", encoding="utf-8") as fh:
        w = csv.DictWriter(fh, fieldnames=list(rows[0]))
        w.writeheader()
        w.writerows({**r, "jasper_says": ""} for r in rows)
    out = tmp_path / "out"
    halfc.main([str(run), "--out", str(out)])
    assert read_json(out / "halfc-summary.json")["handcheck"]["marked"] is False


def test_degraded_no_probe(run, tmp_path):
    for p in run.glob("probe*"):
        p.unlink()
    (run / "openapi.json").unlink()
    out = tmp_path / "out"
    assert halfc.main([str(run), "--out", str(out)]) == 0
    s = read_json(out / "halfc-summary.json")
    for row in s["facts"][:5]:
        assert row["value"].startswith("not reached: probe.json missing"), row
    assert "not reached: probe.json missing" in s["facts"][5]["value"]
    assert s["realtime_only"] is None
    assert "minimap pass: not reached" in s["time_cost"]
    assert findings.unfilled_lines((out / "scout-findings.md").read_text(encoding="utf-8")) == []


def test_degraded_failed_sub_probe(run, tmp_path):
    probe = read_json(run / "probe.json")
    del probe["recording"]
    probe["errors"]["recording"] = "RuntimeError('no recording')"
    (run / "probe.json").write_text(json.dumps(probe), encoding="utf-8")
    out = tmp_path / "out"
    halfc.main([str(run), "--out", str(out)])
    s = read_json(out / "halfc-summary.json")
    assert s["facts"][2]["value"] == "not reached: RuntimeError('no recording')"


def test_degraded_empty_run_dir(tmp_path):
    run = tmp_path / "empty-run"
    run.mkdir()
    out = tmp_path / "out"
    assert halfc.main([str(run), "--out", str(out)]) == 0
    s = read_json(out / "halfc-summary.json")
    assert s["route"]["route"] == "no-go"
    assert s["positions_minimap"]["verdict"]["verdict"] == "NOT REACHED"
    assert s["eye_check"]["events"] is None
    assert findings.unfilled_lines((out / "scout-findings.md").read_text(encoding="utf-8")) == []


def test_manifest_flags_key_identifiers_video_and_size(run):
    assert halfc.manifest_check(run)["ok"]
    (run / "notes.txt").write_text("key " + "RG" + "API-0000", encoding="utf-8")
    m = read_json(run / "match.json")
    m["info"]["participants"][0]["puuid"] = "x" * 78
    m["info"]["participants"][1]["riotIdGameName"] = "someone"
    (run / "match.json").write_text(json.dumps(m), encoding="utf-8")
    (run / "rec.webm").write_bytes(b"\0" * 10)
    res = halfc.manifest_check(run)
    assert not res["ok"]
    assert res["key_hits"] == ["notes.txt"]
    assert res["videos"] == ["rec.webm"]
    assert any("puuid" in h for h in res["identifier_hits"])
    assert any("riotIdGameName" in h for h in res["identifier_hits"])


def test_manifest_flags_puuid_list_and_size(run, monkeypatch):
    tl = read_json(run / "timeline.json")
    tl["metadata"]["participants"] = ["abc"] * 10
    (run / "timeline.json").write_text(json.dumps(tl), encoding="utf-8")
    monkeypatch.setattr(halfc, "MAX_RUN_MB", 0.1)
    res = halfc.manifest_check(run)
    assert any("PUUID list" in h for h in res["identifier_hits"])
    assert not res["size_ok"]


def test_refresh_topdown_rebuilds_a_pre_r2_track(tmp_path):
    """A track_topdown.json written before R2 (raw blobs, no fit) is rebuilt from
    its detections_px, and route B is read from the fit's leave-one-frame-out median."""
    from scout import track_topdown

    frames = mf.write_topdown_series(tmp_path / "frames")
    first = track_topdown.run_offline(frames, tmp_path / "rec", mf.CHAMPIONS, None, None, 1)
    legacy = {k: v for k, v in first.items() if k not in ("processing", "fit", "bar_counts")}
    run_dir, out = tmp_path / "run", tmp_path / "out"
    run_dir.mkdir()
    out.mkdir()
    td, path = halfc.refresh_topdown(run_dir, out, legacy, mf.td_timeline(), mf.td_minimap_track())
    assert path == out / "track_topdown_r2.json" and path.is_file()
    assert td["processing"] == "r2" and td["fit"]["identity"] == "minimap"
    b = halfc.topdown_input(td, {"median_error": 1.0})
    assert b["median_from"] == "fit leave-one-frame-out" and b["median_error"] == td["fit"]["lofo_median"] < 40
    assert b["bar_hit_rate"] == 1.0 and b["calibrated"]
    assert halfc.refresh_topdown(run_dir, out, td, None, None) == (td, None)   # already R2


def test_handcheck_reads_jaspers_words(run, tmp_path):
    rows = list(csv.DictReader(open(run / "handcheck.csv", encoding="utf-8")))
    words = ["agreed", "disagree", "exact detection", "Yes", "hmm, not sure"]
    with open(run / "handcheck.csv", "w", newline="", encoding="utf-8-sig") as fh:
        w = csv.DictWriter(fh, fieldnames=list(rows[0]))
        w.writeheader()
        w.writerows({**r, "jasper_says": words[i % len(words)]} for i, r in enumerate(rows))
    out = tmp_path / "out"
    out.mkdir()
    hc = halfc.merge_handcheck(run, out)
    assert hc["marked"] and hc["marked_rows"] == 4 and hc["agree"] == 3 and hc["unclear"] == 1
    merged = list(csv.DictReader(open(out / "handcheck_merged.csv", encoding="utf-8")))
    assert [r["agree"] for r in merged] == ["yes", "no", "yes", "yes", "unclear"]
    assert "1 unclear" in halfc.handcheck_line(hc)
    for said, mark in [("agree", "yes"), ("y", "yes"), ("correct", "yes"), ("Exactly", "yes"),
                       ("true", "yes"), ("no", "no"), ("n", "no"), ("wrong", "no"), ("false", "no"),
                       ("", ""), ("maybe", "unclear")]:
        assert halfc.hand_mark(said) == mark, said


def test_facts_r2_probe_shape(run, tmp_path):
    out = tmp_path / "out"
    halfc.main([str(run), "--out", str(out)])
    facts = {r["n"]: r for r in read_json(out / "halfc-summary.json")["facts"]}
    assert facts[1]["value"].startswith("R2 combination honoured")
    assert "11 of 11 other interface* fields read false" in facts[1]["value"]
    assert "known to hide the minimap" in facts[1]["value"] and "interface.legacy" in facts[1]["evidence"]
    assert facts[4]["value"].startswith("visual fit: camera y 50000 at fov 19.0")
    assert "probe_camera_fit_50000.png" in facts[4]["evidence"]
    assert "API read-back honoured up to 60000" in facts[4]["value"]


def test_facts_legacy_r1_probe_shape(run, tmp_path):
    """A probe.json written by R1 (Jasper's 2026-10-22 run): only the legacy combination, no camera_fit."""
    probe = read_json(run / "probe.json")
    probe["interface"] = {"interfaceAll": {"sent": False, "read": False, "honoured": True},
                          "interfaceMinimap": {"sent": True, "read": True, "honoured": True},
                          "screenshot": probe["interface"]["screenshot"]}
    del probe["camera_fit"]
    (run / "probe.json").write_text(json.dumps(probe), encoding="utf-8")
    out = tmp_path / "out"
    assert halfc.main([str(run), "--out", str(out)]) == 0
    facts = {r["n"]: r for r in read_json(out / "halfc-summary.json")["facts"]}
    assert facts[1]["value"].startswith("R1 probe: only the legacy combination")
    assert "hides the minimap" in facts[1]["value"] and "R1 shape" in facts[1]["evidence"]
    assert "read-back only: no camera_fit" in facts[4]["value"]
    assert findings.unfilled_lines((out / "scout-findings.md").read_text(encoding="utf-8")) == []
