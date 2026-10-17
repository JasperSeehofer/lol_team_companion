from scout import fetch_timeline

KEY = "RGAPI-00000000-test-not-a-real-key"


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
            return FakeResponse({"info": {"frames": [{}]}})
        return FakeResponse({"info": {"participants": [{}] * 10}})

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
