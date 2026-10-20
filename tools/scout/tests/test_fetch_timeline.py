import json
from pathlib import Path

from scout import fetch_timeline, score
from scout.util import read_json, write_json

REPO_FIXTURE = Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "riot_match_v5_timeline.json"

KEY = "RG" + "API-00000000-test-not-a-real-key"   # never the literal prefix


class FakeResponse:
    def __init__(self, data, status=200):
        self.data, self.status_code = data, status

    def json(self):
        return self.data


def test_refuses_without_key(monkeypatch, tmp_path, capsys):
    monkeypatch.delenv("RIOT_API_KEY", raising=False)
    rc = fetch_timeline.main(["--match-id", "EUW1_1", "--out", str(tmp_path / "run")])
    assert rc == 2
    assert "RIOT_API_KEY" in capsys.readouterr().err
    assert not (tmp_path / "run").exists()


def test_key_never_written(monkeypatch, tmp_path):
    monkeypatch.setenv("RIOT_API_KEY", KEY)
    seen = []

    def fake_get(url, headers, timeout):
        seen.append(url)
        assert headers == {"X-Riot-Token": KEY}
        if url.endswith("/timeline"):
            return FakeResponse({"info": {"frames": [{}], "participants": []}})
        return FakeResponse({"info": {"participants": [{"participantId": i} for i in range(1, 11)]}})

    monkeypatch.setattr(fetch_timeline.requests, "get", fake_get)
    assert fetch_timeline.main(["--match-id", "EUW1_1", "--out", str(tmp_path)]) == 0
    assert seen[0].startswith("https://europe.api.riotgames.com/lol/match/v5/matches/EUW1_1")
    assert {p.name for p in tmp_path.iterdir()} == {"timeline.json", "match.json", "run.log"}
    for p in tmp_path.iterdir():
        assert KEY not in p.read_text(encoding="utf-8")


def test_key_not_logged_on_error(monkeypatch, tmp_path):
    monkeypatch.setenv("RIOT_API_KEY", KEY)
    monkeypatch.setattr(fetch_timeline.requests, "get",
                        lambda url, headers, timeout: FakeResponse({}, status=403))
    assert fetch_timeline.main(["--match-id", "EUW1_1", "--out", str(tmp_path)]) == 1
    assert KEY not in (tmp_path / "run.log").read_text(encoding="utf-8")
    assert not (tmp_path / "timeline.json").exists()


def full_match(timeline: dict) -> dict:
    """A raw-shaped Match-V5 match for the repo fixture's timeline, with every
    identifier a real response carries."""
    champs = ["Aatrox", "LeeSin", "Ahri", "Jinx", "Thresh", "Garen", "Vi", "Lux", "Caitlyn", "Leona"]
    return {"metadata": {"dataVersion": "2", "matchId": timeline["metadata"]["matchId"],
                         "participants": list(timeline["metadata"]["participants"])},
            "info": {"gameVersion": "16.20.700.1234", "gameDuration": 1800, "gameMode": "CLASSIC",
                     "participants": [{
                         "participantId": i + 1, "championName": c, "teamId": 100 if i < 5 else 200,
                         "teamPosition": "TOP", "win": i < 5, "kills": 3,
                         "puuid": timeline["metadata"]["participants"][i],
                         "riotIdGameName": f"Player{i}", "riotIdTagline": "EUW",
                         "summonerName": f"Summoner{i}", "summonerId": f"sid{i}",
                         "challenges": {"kda": 2.0}} for i, c in enumerate(champs)]}}


def test_output_minimised_on_the_repo_fixture(monkeypatch, tmp_path):
    raw_tl = read_json(REPO_FIXTURE)
    raw_tl.pop("_fixture_notes")    # the fixture's prose notes, not part of a Riot response
    raw_match = full_match(raw_tl)
    monkeypatch.setenv("RIOT_API_KEY", KEY)
    monkeypatch.setattr(fetch_timeline.requests, "get", lambda url, headers, timeout: FakeResponse(
        json.loads(json.dumps(raw_tl if url.endswith("/timeline") else raw_match))))
    assert fetch_timeline.main(["--match-id", raw_tl["metadata"]["matchId"], "--out", str(tmp_path)]) == 0
    for name in ("timeline.json", "match.json", "run.log"):
        text = (tmp_path / name).read_text(encoding="utf-8")
        low = text.lower()
        for word in ("puuid", "riotid", "summonername", "summonerid"):
            assert word not in low, (name, word)
        for p in raw_tl["metadata"]["participants"]:
            assert p not in text
    match = read_json(tmp_path / "match.json")
    assert set(match["info"]) == {"gameVersion", "gameDuration", "participants"}
    assert match["metadata"] == {"matchId": raw_tl["metadata"]["matchId"]}
    for p in match["info"]["participants"]:
        assert set(p) == {"participantId", "championName", "teamId", "teamPosition", "win"}
    tl = read_json(tmp_path / "timeline.json")
    assert "participants" not in tl["metadata"]
    assert tl["info"]["participants"] == [{"participantId": i} for i in range(1, 11)]
    assert tl["info"]["frames"] == raw_tl["info"]["frames"]      # events and positions untouched
    # score still works on the minimised pair: a track at the timeline's own positions
    pmap = score.participant_map(match)
    samples = [{"t": f["timestamp"] / 1000, "pos": {
        pmap[int(pid)]: [pf["position"]["x"], pf["position"]["y"]]
        for pid, pf in f["participantFrames"].items()}} for f in tl["info"]["frames"]]
    write_json(tmp_path / "track.json", {"champions": list(pmap.values()), "samples": samples, "wards": []})
    assert score.main(["--track", str(tmp_path / "track.json"), "--timeline", str(tmp_path / "timeline.json"),
                       "--match", str(tmp_path / "match.json"), "--out", str(tmp_path / "s")]) == 0
    res = read_json(tmp_path / "s" / "score.json")
    assert res["median_error"] == 0 and res["n_scored"] > 0


def test_strip_identifiers_any_depth():
    node = {"a": [{"puuid": 1, "riotIdTagline": 2, "keep": {"summonerName": 3, "x": 4}}]}
    assert fetch_timeline.strip_identifiers(node) == {"a": [{"keep": {"x": 4}}]}
