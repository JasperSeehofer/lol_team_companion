# Scout run checklist (Windows, PowerShell, about 2 h)

Run everything from the repository root in PowerShell. Every command appends to `$RUN\run.log`.
If a step fails and the "If it fails" line does not help, note it in `$RUN\run.log`
(`Add-Content $RUN\run.log "step N: what happened"`), skip the step and go on.

1. **Get the branch.** `git fetch origin; git switch mission/M156-scout-replay-spike; git pull`
   Output: `tools\scout\` on disk. If it fails: `git status`, stash or commit local changes first.
2. **Python 3.12 and the venv.** `winget install Python.Python.3.12`, then
   `py -3.12 -m venv tools\scout\.venv; tools\scout\.venv\Scripts\pip install -e "tools/scout[yolo]"`
   and `Set-Alias spy "$PWD\tools\scout\.venv\Scripts\python.exe"` (repeat in each new shell).
   Output: `tools\scout\.venv\`. If it fails: `py -0` must list 3.12; rerun pip with `-v`.
3. **Weights.** Download `yolov12.pt` (227 MB) from the Google Drive link in
   `tools\scout\vendor\pyLoL\README.md` to `tools\scout\models\yolov12.pt` (git-ignored).
   Output: that file. If it fails: skip it; step 7 then uses the template detector (drop `--weights`).
4. **Enable the Replay API.** In `C:\Riot Games\League of Legends\Config\game.cfg` under `[General]`
   add the line `EnableReplayApi=1`, save, restart the League client.
   Output: `game.cfg` with that line. If it fails: the file may be read-only; untick read-only in Properties.
5. **Pick the replay.** One recent game of your own on the current patch. Note its match id
   (`EUW1_` + the game id in the match history). Open the replay, set the window to borderless
   full screen on the main monitor, let it load. Then:
   `$MATCH = "EUW1_..."; $RUN = "tools\scout\runs\$(Get-Date -Format yyyy-MM-dd)-$MATCH"; mkdir $RUN`
   Output: the run directory. If it fails: replays of an older patch will not open; pick a newer game.
6. **Probe (about 3 min; do not touch mouse or keyboard).** `spy -m scout.probe --out $RUN`
   Output: `probe.json`, `openapi.json` (if the client serves a spec), `probe_*.png`.
   If it fails: open `https://127.0.0.1:2999/replay/game` in a browser (accept the certificate
   warning); no answer means the replay is not open or step 4 did not take. `--dry` lists the steps.
7. **Minimap track (game length / 8; hands off; it rewinds the replay itself).**
   `spy -m scout.track_minimap --out $RUN --weights tools\scout\models\yolov12.pt --speed 8`
   Output: `track_minimap.json`, `frames_minimap\t*.png` (24 frames).
   If it fails: open the first PNG in `frames_minimap`; if it is not exactly the minimap, read the
   minimap's left, top, width, height in `probe_minimap_only.png` with Paint and add
   `--minimap-box L,T,W,H`. If champions are not found, do step 9 first and add `--match $RUN\match.json`.
8. **Top-down track (about 8 min).** The probe's test recordings used this session's one
   recording, so **sign out of Windows and back in**, reopen the replay (step 5 window settings),
   recreate `$MATCH`, `$RUN` and the alias, then `spy -m scout.track_topdown --out $RUN`
   Output: `track_topdown.json`, `frames_topdown\t*.png` (6), `anchors_template.json`.
   If it fails: if `probe_camera_*.png` never shows the whole map, add `--keystroke-unlock` (press
   Ctrl+Shift+Y and zoom out when asked). If nothing was recorded, sign out and in again, run once.
9. **Timeline.** `$env:RIOT_API_KEY = "RGAPI-..."; spy -m scout.fetch_timeline --match-id $MATCH --out $RUN; Remove-Item Env:RIOT_API_KEY`
   Output: `timeline.json`, `match.json`. The key is never written to disk.
   If it fails: HTTP 403 = expired development key (regenerate it); 404 = wrong match id or region.
10. **Score.**
    `spy -m scout.score run --track $RUN\track_minimap.json --timeline $RUN\timeline.json --match $RUN\match.json --wards-track $RUN\track_topdown.json --out $RUN`
    then `spy -m scout.score plots --score $RUN\score.json --minimap (Get-ChildItem $RUN\frames_minimap)[0].FullName --out $RUN\plots`
    Output: `score.json`, `handcheck.csv`, `plots\*.png`. If it fails: go on; the agent rescoring
    in half C needs only the files from steps 6 to 9.
11. **Hand check.** Open `handcheck.csv` (Notepad or Excel). For each row jump the replay to
    `game_time_mmss`, look at the champion and place, and write `yes` or `no` in `jasper_says`
    (did a ward go down / did that champion recall there and then?); `note` is optional. Save as CSV.
    Output: `handcheck.csv` filled in. If it fails: fewer than 10 rows is fine; mark what is there.
12. **Size and content check.**
    `(Get-ChildItem $RUN -Recurse | Measure-Object Length -Sum).Sum / 1MB` must be at most 50;
    `Get-ChildItem $RUN -Recurse -Include *.webm,*.mp4,*.avi` and
    `Select-String -Path "$RUN\*" -Pattern "RGAPI"` must both print nothing.
    Output: the size noted in `run.log`. If it fails: delete the largest PNGs in `frames_topdown` until under 50 MB.
13. **Commit and push.** `git add $RUN; git commit -m "data(m156): scout run $MATCH"; git push`
    Output: the run directory on `mission/M156-scout-replay-spike`. If it fails: `git pull --rebase`, push again.

Recordings and captured frames stay in `%TEMP%\scout-rec` and `%TEMP%\scout-frames`; delete them when done.
