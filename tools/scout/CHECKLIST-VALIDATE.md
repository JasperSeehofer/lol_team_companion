# Scout validation run (Windows, PowerShell, about 30 min)

Short re-run of the same game as before, to measure the R2 fixes (K+F tracker, circle icons, wards,
recalls). Repository root, PowerShell, WireGuard to brain up. First type once:
`function spy { .\tools\scout\.venv\Scripts\python -m scout.session @args }`. Run data never goes to GitHub.
If a step fails and its "If it fails" line does not help, note it in the run's `run.log` and go on.

1. **Get the branch.** `git switch mission/M156-scout-replay-spike; git pull --ff-only`, then
   `.\tools\scout\.venv\Scripts\pip install -e "tools/scout"` (picks up the R2 code).
   Output: none. If it fails: `git status`; commit or stash local changes first.
2. **HDR off** on the game monitor (Settings > System > Display > Use HDR: off). The minimap step writes a
   warning to `run.log` if it is still on. Output: none. If it fails: check that the game runs on that monitor.
3. **New run dir, same game.** `spy start EUW1_<gameId>` with the same match id as the first run (today's
   date makes it a fresh folder). Output: `tools\scout\runs\<date>-EUW1_<gameId>\`.
   If it fails: the id must look like `EUW1_7100000099`.
4. **Timeline first** (the minimap and top-down steps read it). Your development key only, never the
   production key: `$env:RIOT_API_KEY = Read-Host "Riot key"`, then `spy timeline`, then `Remove-Item Env:RIOT_API_KEY`.
   Output: `match.json`, `timeline.json`. If it fails: 403 = expired key; 404 = wrong id.
5. **Open the replay** of that game in borderless full screen on the main monitor; do not run the probe
   (its recording would use up this sign-in's one recording).
6. **Minimap pass (game length ÷ 8, hands off).** `spy minimap` (speed 8, base circle icons by default;
   add the same `--minimap-box L,T,W,H` as last time if you used one; if `tools\scout\models\yolov12.pt`
   exists, move it out of that folder first, so the template detector runs as before). Output: `track_minimap.json`,
   `frames_minimap\`, `run.log` (HDR warning, icon sources). If it fails: if `run.log` shows icons from
   Data Dragon only (`icon template` lines), CommunityDragon was unreachable; note it and go on.
7. **Sign out of Windows and back in** (one recording per sign-in), reopen the replay (step 5).
8. **Top-down window (about 2 min).** `spy topdown` (camera 50000 scaled for 16:9, window 600–900 s).
   Output: `track_topdown.json`, `frames_topdown\`. If it fails: 0 frames = a recording already ran in this
   sign-in; repeat step 7. Whole map not in view: add `--keystroke-unlock`.
9. **Score.** `spy score`. Output: `score.json`, `handcheck.csv` (wards and recalls of the NEW outputs).
   If it fails: the message names the missing file; redo that step.
10. **Re-mark the hand check.** `spy handcheck`; per row jump the replay to `game_time_mmss` and write
    `yes` or `no` in `jasper_says` (do not copy the old verdicts). Output: `handcheck.csv`, saved.
11. **Check and send to brain.** `spy check`, then `spy send` (one `scp` into brain's inbox).
    Output: `~/.local/share/scout/inbox/<run>/` on brain. If `check` fails: delete the largest PNGs in
    `frames_topdown`, run it again. Never `git add` or push the run.

Delete `%TEMP%\scout-rec` and `%TEMP%\scout-frames` when done. The validation decides: route A's real
error and detection after K+F, the ward and recall verdicts, and whether to merge (`docs/scout-findings.md`).
