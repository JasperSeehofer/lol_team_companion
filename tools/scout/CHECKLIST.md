# Scout run checklist (Windows, PowerShell, about 2 h)

Run everything from the repository root in PowerShell, WireGuard tunnel to brain up. Every command
is one line and uses the session driver: it remembers the current run in `tools\scout\runs\.current`,
so nothing is typed again after the sign-out in step 8. Each step is marked **PC-only** (you do it) or
**pre-done on brain** (the agent did it). If a step fails and its "If it fails" line does not help,
note it in the run's `run.log`, skip the step and go on. Run data never goes to GitHub (the repo is public).

1. **PC-only. Get the branch.** `git fetch origin; git switch mission/M156-scout-replay-spike; git pull --ff-only`
   If it fails: `git status`; commit or stash local changes first.
2. **PC-only. Venv and run dir.** `winget install Python.Python.3.12` (once), then
   `py -3.12 -m venv tools\scout\.venv; .\tools\scout\.venv\Scripts\pip install -e "tools/scout[yolo]"`, then
   `.\tools\scout\.venv\Scripts\python -m scout.session start EUW1_<gameId>` (the match id from your match history).
   If it fails: `py -0` must list 3.12; rerun pip with `-v`.
3. **PC-only. Weights.** Brain's class check (M166, 2026-10-20): **not checked** (free -g: 2 GB available,
   floor 3 GB; df 17 GB free; load 2.37). If the card says the weights were checked, copy them from brain:
   `scp ops@10.44.0.1:.cache/scout/models/yolov12.pt tools\scout\models\yolov12.pt`
   Otherwise download `yolov12.pt` (227 MB) from the Google Drive link in `tools\scout\vendor\pyLoL\README.md`
   into `tools\scout\models\`. If it fails: skip it; step 7 then uses the template detector by itself.
4. **PC-only. Replay API.** `.\tools\scout\.venv\Scripts\python -m scout.session enable-api`, then restart the
   League client. If it fails: pass `--cfg "<your League folder>\Config\game.cfg"`; untick read-only in Properties.
5. **PC-only. Open the replay** of that game (current patch) in borderless full screen on the main monitor.
6. **PC-only. Probe (about 3 min, hands off).** `.\tools\scout\.venv\Scripts\python -m scout.session probe`
   If it fails: open `https://127.0.0.1:2999/replay/game` in a browser; no answer = replay not open or step 4 did not take.
7. **PC-only. Minimap track (game length / 8, hands off).** `.\tools\scout\.venv\Scripts\python -m scout.session minimap`
   If it fails: if the first PNG in `frames_minimap` is not exactly the minimap, read its left, top, width,
   height in `probe_minimap_only.png` with Paint and add `--minimap-box L,T,W,H`.
8. **PC-only. Top-down track (about 8 min).** Sign out of Windows and back in, reopen the replay (step 5), then
   `.\tools\scout\.venv\Scripts\python -m scout.session topdown`
   If it fails: add `--keystroke-unlock` (press Ctrl+Shift+Y and zoom out when asked).
9. **PC-only. Timeline.** Your development key only, never product 834192's production key. Three lines:
   `$env:RIOT_API_KEY = Read-Host "Riot key"` (typed at the prompt, so it never lands in PowerShell history), then
   `.\tools\scout\.venv\Scripts\python -m scout.session timeline`, then `Remove-Item Env:RIOT_API_KEY`
   `match.json` is minimised (no PUUID, Riot ID or summoner name). If it fails: HTTP 403 = expired key; 404 = wrong id.
10. **PC-only. Score.** `.\tools\scout\.venv\Scripts\python -m scout.session score` (plots optional: add `--plots`;
    half C redraws them). Writes `score.json`, `handcheck.csv` and the minimap frames around ward placements.
11. **PC-only. Hand check.** `.\tools\scout\.venv\Scripts\python -m scout.session handcheck` opens it in Notepad. Per row,
    jump the replay to `game_time_mmss` and write `yes` or `no` in `jasper_says`; save. Fewer than 10 rows is fine.
12. **PC-only. Check.** `.\tools\scout\.venv\Scripts\python -m scout.session check` (at most 50 MB, no video, no key,
    no player identifiers). If it fails: delete the largest PNGs in `frames_topdown`, run it again.
13. **PC-only. Send to brain (never GitHub).** `.\tools\scout\.venv\Scripts\python -m scout.session send`
    (one `scp` into `~/.local/share/scout/inbox/<run>/` on brain; `--dry-run` only prints it).

Delete `%TEMP%\scout-rec` and `%TEMP%\scout-frames` when done.

**Without the driver** (`$RUN` = the run dir; `spy` = `.\tools\scout\.venv\Scripts\python`):
`spy -m scout.probe --out $RUN` · `spy -m scout.track_minimap --out $RUN --speed 8 [--weights tools\scout\models\yolov12.pt]` ·
`spy -m scout.track_topdown --out $RUN` · `spy -m scout.fetch_timeline --match-id $MATCH --out $RUN` ·
`spy -m scout.score run --track $RUN\track_minimap.json --timeline $RUN\timeline.json --match $RUN\match.json --wards-track $RUN\track_topdown.json --out $RUN` ·
`spy -m scout.halfc $RUN --out $env:TEMP\halfc` (the manifest check and more) · `scp -r $RUN ops@10.44.0.1:.local/share/scout/inbox/`
