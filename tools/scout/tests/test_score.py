import csv


from scout import score
from scout.util import read_json, write_json


def tiny_timeline(frames, events=()):
    return {"info": {"frames": [
        {"timestamp": int(t * 1000), "events": [e for e in events if e.get("_frame") == i],
         "participantFrames": {str(pid): {"position": {"x": x, "y": y}}
                               for pid, (x, y) in pos.items()}}
        for i, (t, pos) in enumerate(frames)]}}


def test_position_maths_hand_computed():
    track = {"champions": ["A", "B"], "samples": [
        {"t": 0.0, "pos": {"A": [0, 0], "B": [100, 0]}},
        {"t": 1.0, "pos": {"A": None, "B": [100, 0]}},
        {"t": 2.0, "pos": {"A": None, "B": None}},
        {"t": 59.5, "pos": {"A": [30, 40], "B": None}},   # within +-1 s of the 60 s frame
    ]}
    tl = tiny_timeline([(0, {1: (0, 0), 2: (100, 10)}), (60, {1: (0, 0), 2: (0, 0)})])
    res = score.score_positions(track, tl, {1: "A", 2: "B"})
    # errors: t0 A 0, B 10; t60 A 50 (3-4-5), B missing
    assert sorted(e["err"] for e in res["errors"]) == [0, 10, 50]
    assert res["median_error"] == 10
    assert res["per_champion"]["A"]["median_error"] == 25
    assert res["per_champion"]["A"]["detection_rate"] == 0.5
    assert res["per_champion"]["B"]["detection_rate"] == 0.5
    assert res["detection_rate"] == 0.5
    assert res["per_champion"]["A"]["gaps_count"] == 1
    assert res["per_champion"]["A"]["longest_gap_s"] == 58.5   # t=1 until t=59.5


def test_nearest_sample_respects_tolerance():
    look = score.Lookup([{"t": 10.0, "pos": {}}, {"t": 12.0, "pos": {}}])
    assert look.at(11.2)["t"] == 12.0
    assert look.at(13.5) is None


def test_ward_matching_two_seconds_and_per_team():
    events = [
        {"type": "WARD_PLACED", "timestamp": 100000, "creatorId": 2, "wardType": "YELLOW_TRINKET", "_frame": 1},
        {"type": "WARD_PLACED", "timestamp": 110000, "creatorId": 3, "wardType": "SIGHT_WARD", "_frame": 1},
        {"type": "WARD_PLACED", "timestamp": 120000, "creatorId": 7, "wardType": "CONTROL_WARD", "_frame": 1},
        {"type": "WARD_PLACED", "timestamp": 125000, "creatorId": 8, "wardType": "UNDEFINED", "_frame": 1},
        {"type": "WARD_PLACED", "timestamp": 900000, "creatorId": 9, "wardType": "YELLOW_TRINKET", "_frame": 1},
    ]
    tl = tiny_timeline([(0, {}), (120, {})], events)
    wards = [{"t": 101.5, "x": 1, "y": 1, "team": "blue"},    # matches pid 2 (+1.5 s)
             {"t": 113.0, "x": 1, "y": 1, "team": "blue"},    # 3 s late: no match
             {"t": 119.0, "x": 1, "y": 1, "team": "red"},     # matches pid 7 (-1 s)
             {"t": 140.0, "x": 1, "y": 1, "team": "red"}]     # false ward
    res = score.match_wards(wards, tl, {2: "LeeSin", 3: "Ahri", 7: "Vi"}, (0, 200))
    assert res["blue"] == {"detected": 2, "placed": 2, "tp": 1, "fp": 1, "fn": 1,
                           "precision": 0.5, "recall": 0.5}
    # red: UNDEFINED skipped, the 900 s ward lies outside the track's time range
    assert res["red"] == {"detected": 2, "placed": 1, "tp": 1, "fp": 1, "fn": 0,
                          "precision": 0.5, "recall": 1.0}
    assert {p["champion"] for p in res["pairs"]} == {"LeeSin", "Vi"}


def test_recalls_on_synthetic_game(fixtures):
    track = read_json(fixtures / "track_synth.json")
    tl = read_json(fixtures / "timeline_synth.json")
    pmap = score.participant_map(read_json(fixtures / "match_synth.json"))
    recalls = score.detect_recalls(track, tl, pmap)
    # Jinx recalls (arrives 199, stays to 215, three purchases); Caitlyn's 4 s visit
    # is too short; Garen's fountain stay after his death is a respawn.
    assert [(r["champion"], r["t"], r["confirmed"], r["purchases"]) for r in recalls] == \
        [("Jinx", 199.0, True, 3)]
    assert recalls[0]["stay_s"] >= 8


def test_recall_negative_short_stay():
    track = {"champions": ["A"], "samples": [{"t": float(t), "pos": {"A": [5000, 5000] if t < 3 or t > 9 else [400, 420]}}
                                             for t in range(15)]}  # 7 s at the fountain
    assert score.detect_recalls(track, {"info": {"frames": []}}, {1: "A"}) == []


def test_score_cli_and_plots(fixtures, tmp_path):
    track = read_json(fixtures / "track_synth.json")
    track["wards"] = [{"t": 96.0, "x": 9000, "y": 2000, "team": "blue"},
                      {"t": 131.0, "x": 9000, "y": 9000, "team": "red"},
                      {"t": 200.0, "x": 5000, "y": 5000, "team": "red"}]
    write_json(tmp_path / "track.json", track)
    rc = score.main(["--track", str(tmp_path / "track.json"), "--timeline",
                     str(fixtures / "timeline_synth.json"), "--match",
                     str(fixtures / "match_synth.json"), "--out", str(tmp_path)])
    assert rc == 0
    res = read_json(tmp_path / "score.json")
    assert 20 < res["median_error"] < 100           # noise sigma 40 per axis
    assert res["n_scored"] == 70                    # 7 frames x 10 (Garen is alive at each frame)
    assert res["per_champion"]["Garen"]["gaps_count"] == 1
    assert res["per_champion"]["Garen"]["longest_gap_s"] == 19.0
    assert res["wards"]["blue"]["tp"] == 1 and res["wards"]["red"]["tp"] == 1
    rows = list(csv.DictReader(open(tmp_path / "handcheck.csv", encoding="utf-8")))
    assert [r["kind"] for r in rows].count("recall") == 1
    assert [r["kind"] for r in rows].count("ward") == 3
    assert all(r["jasper_says"] == "" for r in rows)
    assert len(rows) == 6   # 3 wards + 1 recall + 2 missed WARD_PLACED as padding
    assert list(rows[0]) == score.HANDCHECK_COLUMNS
    rc = score.main(["plots", "--score", str(tmp_path / "score.json"), "--minimap",
                     str(fixtures / "frames" / "t00060.png"), "--out", str(tmp_path / "plots")])
    assert rc == 0
    names = sorted(p.name for p in (tmp_path / "plots").iterdir())
    assert "error_per_champion.png" in names and len(names) == 4


def test_ward_window_pre_existing_and_tolerance():
    events = [
        {"type": "WARD_PLACED", "timestamp": 600500, "creatorId": 2, "wardType": "YELLOW_TRINKET", "_frame": 1},
        {"type": "WARD_PLACED", "timestamp": 610000, "creatorId": 2, "wardType": "YELLOW_TRINKET", "_frame": 1},
        {"type": "WARD_PLACED", "timestamp": 620000, "creatorId": 7, "wardType": "SIGHT_WARD", "_frame": 1},
        {"type": "WARD_PLACED", "timestamp": 700000, "creatorId": 7, "wardType": "SIGHT_WARD", "_frame": 1},
    ]
    tl = tiny_timeline([(600, {}), (660, {})], events)
    wards = [{"t": 600.85, "team": "blue", "pre_existing": True},    # visible when the window opened
             {"t": 610.15, "team": "blue"},
             {"t": 623.0, "team": "red"}]                          # 3 s late
    window = [601.0, 699.0]                                        # the 600.5 and 700 s wards are outside
    res = score.match_wards(wards, tl, {}, (600, 700), window=window)
    assert res["pre_existing_excluded"] == 1
    assert (res["blue"]["detected"], res["blue"]["placed"], res["blue"]["tp"]) == (1, 1, 1)
    assert (res["red"]["placed"], res["red"]["tp"]) == (1, 0)
    assert score.match_wards(wards, tl, {}, None, tol=4.0, window=window)["red"]["tp"] == 1


def _locked_track():
    samples = [{"t": float(t), "pos": {"A": [13390.9, 680.6], "B": [1000.0 + 50 * t, 1000.0]}}
               for t in range(0, 120)]
    return {"champions": ["A", "B"], "samples": samples}


def test_false_lock_counts_as_not_detected():
    tl = {"info": {"frames": [{"timestamp": t * 60000, "events": [], "participantFrames": {
        "1": {"position": {"x": 554, "y": 581}},
        "2": {"position": {"x": 1000 + 50 * 60 * t, "y": 1000}}}} for t in range(2)]}}
    res = score.score_positions(_locked_track(), tl, {1: "A", 2: "B"})
    assert res["false_lock"] == ["A"]
    assert res["per_champion"]["A"]["false_lock"] and res["per_champion"]["A"]["detection_rate"] == 0
    assert res["per_champion"]["A"]["median_error"] is None
    assert not res["per_champion"]["B"]["false_lock"] and res["detection_rate"] == 0.5
    assert all(e["champion"] == "B" for e in res["errors"])


def test_false_lock_needs_the_whole_run_at_one_spot():
    tr = _locked_track()
    tr["samples"][5]["pos"]["A"] = [5000.0, 5000.0]
    assert score.false_locks(tr) == []


def test_drop_static_removes_recurring_spot_samples():
    tr = _locked_track()
    for s in tr["samples"][::2]:          # A moves half the time, sits at the spot otherwise
        s["pos"]["A"] = [2000.0 + 30 * s["t"], 3000.0]
    clean, spots = score.drop_static(tr)
    assert [(s["champion"], s["px"]) for s in spots["spots"]] == [("A", 13390.9)]
    assert spots["rejected"] == {"A": 60}
    assert all(s["pos"]["A"] is None for s in clean["samples"][1::2])
    assert all(s["pos"]["A"] is not None for s in clean["samples"][::2])
    assert all(s["pos"]["B"] is not None for s in clean["samples"])
    assert tr["samples"][1]["pos"]["A"] is not None   # the input is left as it was


# Recall rule (R2): stay counted inclusively, short gaps bridged; stays of 5 to
# 8 s count only with a purchase. Lucian's 08:03 recall in Jasper's v2 track:
# arrival sample missing, the last sample at 1027 just outside the radius.
def _visit_track(inside: dict, n: int = 520, outside=(3000.0, 3000.0)):
    """Champion A (pid 1, blue); inside: {t: pos or None}."""
    return {"champions": ["A"], "samples": [
        {"t": float(t), "pos": {"A": inside[t] if t in inside else list(outside)}} for t in range(n)]}


def _buys(times, pid=1):
    return {"info": {"frames": [{"timestamp": 0, "participantFrames": {}, "events": [
        {"type": "ITEM_PURCHASED", "participantId": pid, "timestamp": int(t * 1000)} for t in times]}]}}


def test_recall_v2_shape_is_detected():
    inside = {483: None, **{t: [554.0 + 10 * (t % 3), 600.0] for t in range(484, 491)},
              491: [554.0 + 1027.0, 581.0]}
    recalls = score.detect_recalls(_visit_track(inside), _buys([486, 487, 489]), {1: "A"})
    assert [(r["t"], r["stay_s"], r["purchases"], r["confirmed"]) for r in recalls] == \
        [(484.0, 7.0, 3, True)]


def test_short_fountain_visit_without_purchases_is_not_a_recall():
    for length in (3, 6):
        inside = {t: [554.0, 581.0] for t in range(300, 300 + length)}
        assert score.detect_recalls(_visit_track(inside), _buys([]), {1: "A"}) == [], length
    inside = {t: [554.0, 581.0] for t in range(300, 303)}            # 3 s, even with a purchase
    assert score.detect_recalls(_visit_track(inside), _buys([301]), {1: "A"}) == []


def test_recall_bridges_a_two_second_blip_but_not_three():
    blip = {t: [554.0, 581.0] for t in range(100, 112)}
    blip[105] = blip[106] = [2000.0, 2000.0]                        # 2 s outside: bridged
    r = score.detect_recalls(_visit_track(blip), _buys([]), {1: "A"})
    assert [(x["t"], x["stay_s"]) for x in r] == [(100.0, 12.0)]
    split = {t: [554.0, 581.0] for t in range(100, 113)}
    split[105] = split[106] = split[107] = [2000.0, 2000.0]         # 3 s outside: two visits
    r = score.detect_recalls(_visit_track(split), _buys([]), {1: "A"})
    assert r == []                                                  # 5 s each, no purchase
    r = score.detect_recalls(_visit_track(split), _buys([101, 109]), {1: "A"})
    assert [(x["t"], x["stay_s"]) for x in r] == [(100.0, 5.0), (108.0, 5.0)]


def test_recall_stay_runs_through_missing_samples():
    """Viego 29:39 in v2: two samples inside, 62 s without a position, one more inside."""
    inside = {1779: [600.0, 600.0], 1780: [600.0, 600.0], **{t: None for t in range(1781, 1843)},
              1843: [554.0, 600.0]}
    r = score.detect_recalls(_visit_track(inside, n=1900), _buys([1778, 1790]), {1: "A"})
    assert [(x["t"], x["stay_s"]) for x in r] == [(1779.0, 65.0)]
