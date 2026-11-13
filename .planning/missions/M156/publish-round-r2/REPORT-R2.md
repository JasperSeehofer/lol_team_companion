Status: half C done, findings written; a short PC validation run is pending (2026-10-26)

# M156 — Scout spike — REPORT-R2 (half C)

Branch `mission/M156-scout-replay-spike`. Scored on the v2 run (HDR off, base circle icons); v1 appears only as the
HDR baseline row. Run data stays on brain under `~/.local/share/scout/inbox/` and in the job's temp dir; none of it
is in git.

## Budget

- Declared: raised from 3.0 M to **6.0 M** (cap **8.0 M**, max-wall **30 h**).
- Subagent totals this half, approximate: analysis ≈ 192 k; fixes A, B, C, H ≈ 138 k; fixes D, E ≈ 269 k;
  K investigation ≈ 132 k; fixes K, F, I, J, L and recall ≈ 187 k; finishing step (findings, checklist, card) on
  top; plus the orchestrator. Roughly 1.0–1.2 M for the subagents.
- **The overseer's meter is the figure.**

## Least-confident decisions

1. **Route override of the rule.** The rule gives B-first (fact 3: recording is accelerated); I recommend A+B
   with B on key windows, because B gives one window per Windows sign-in (`endTime` is ignored), so a full game
   would take about seven sign-ins, and A's PARTIAL is inflated by a bug that is now fixed (K+F). If the
   validation run leaves A PARTIAL with long gaps, B-first is the honest answer.
2. **Plots and strips kept out of git by an ignore rule** (`publish-round-r2/plots/`, `publish-round-r2/strips/`):
   they show ten players' positions and the repo is public. The overseer must serve the round from this worktree,
   not from origin.
3. **Ward and recall verdicts depend on re-marking.** v2's `handcheck.csv` carries v1's verdicts (2/8 agree), which
   describe R1's false wards. Blue wards pass both automatic bounds and may flip to GO on re-marking alone.
4. **Route A after K+F rests on only 57 mixed saved frames** (median 519 → 315 → 162, n=29–34; the frames mix
   several captures, finding L). Detection rate and the 116 s longest gap after the fix are unmeasured.
5. **F's 60-frame warm-up**: the static-spot rule needs at least 60 frames.
6. **The live running count is implemented but unused**; live capture saves frames and then runs the offline
   two-pass rule.
7. **CommunityDragon URL unchecked** against the network (`.../characters/<c>/hud/<c>_circle.png`, then
   `_circle_0.png`). Data Dragon is the fallback.
8. **Camera scaling by aspect only** (C) assumes fov semantics; 16:10 and ultrawide screens are untested.
9. **Fit pairs within 0.5 s, not ±1 s** (D): the frame at 601 s against the timeline frame at 600.249 s added
   about 250 units of error, so five timeline frames remain in the window.
10. **Recall stays of 5–8 s need a purchase**, trading recall for precision.
11. **HDR detection is untested on real Windows**; only mocked.
12. **The v2 hand check carries v1 verdicts** (see 3).
13. **Five react questions across both cards** (r1: two; r2: four including the merge), against the page's limit
    of three. The overseer's half-C order asked for "≤4 react questions" on this card.
14. **The real match ID is in branch history** from 944b0dc (a comment and a fixture name, pushed before this
    half). Scrubbed in 64e31fd; force-pushing is not allowed, so squash on landing keeps it off main.
15. **Subagents may not write report files**, so I (the orchestrator) wrote this file from the finishing agent's
    text.

## Findings from the Windows sessions

| | Finding | Fix |
|---|---|---|
| A | Probe and minimap capture used mss `monitors[1]`, the secondary portrait monitor | 44b31cb: primary monitor (`is_primary`), `--monitor` override |
| B | `interfaceAll=false` + `interfaceMinimap=true` hides the minimap | 7f52dc7: `interfaceAll=true`, every other `interface*` false |
| C | Camera read-back does not show whether the whole map is in view | 981baab: visual fit, 50000 at 2560×1440 fov 19, scaled by aspect |
| D | Top-down positions were bar positions (split bars, offset 0, stale first second) | d76df29: merged bar left edge, fitted offset + homography, first second dropped |
| E | False wards: mana bars and duplicate pips (precision 0.007 blue / 0.093 red) | 1958842: pip groups, mana excluded, first-appearance stamp, pre-existing excluded |
| F | Champions tracked on static background spots | superseded by K; static-spot rule stays as a guard, 1fec258 |
| G | Recording ignores `endTime`; one recording per sign-in | a fact, no code fix |
| H | Render state left changed (Alt+F4 needed) | 0ad8e06: save GET /replay/render, restore in `finally` |
| I | Windows HDR tone-maps captures (mean 137 vs 88) | 4aa2c5f: `advancedColorEnabled` check, warning, recorded in JSON |
| J | Replay minimap draws base circle icons (Viego 0.37 square → 0.63 circle) | 12d55ea: CommunityDragon circle icons, Data Dragon fallback |
| K | Identity bug: same-label cost ignored the score, so static dark spots captured Pyke, Jax and Zed | 3f35c52: best detection per label; repro tests `test_static_false_spot_does_not_capture_the_track` and `test_static_false_spot_offered_first_does_not_lock_the_track` failed on the old code |
| L | Offline re-runs mixed captures in `frames_minimap` | 0be8d88: removes only its own `tNNNNN.png` samples first |
| Recall | Lucian's 08:03 recall missed in v2 | 797d3a4: inclusive stay, gaps ≤2 s bridged, 5–8 s with a purchase |

**Lucian 08:03.** v1 found it with a stay of exactly 8 s (483–491). In v2 the arrival sample at 483 was missing
and the 491 sample sat at 1027 units from the fountain, just outside the 1000 radius, so the stay counted 6 s and
was rejected. With the new rule it is found (arrives 484, stay 7 s, 3 purchases).

**K, the root cause.** The template detector returns up to two peaks per champion. The dark portraits (Pyke, Jax,
Zed) match black areas (the off-map strip below the map, jungle walls) at 0.65–0.74 in every frame. The tracker's
same-champion cost was plain distance, accepted at any distance, so once a track touched a static spot (a first
frame tie, or a frame where the real icon was dead or hidden) it never left. The value 480.4 is the last template
row, not a clamp. The score's participant mapping was correct.

## Item 2: facts table

| # | Fact | Measured | Evidence |
|---|---|---|---|
| 1 | Minimap-only mode | `interfaceAll=false` + `interfaceMinimap=true` hides the minimap despite the read-back; `interfaceAll=true` with every other `interface*` false works | Windows session finding B; probe.json `interface` |
| 2 | Max honoured speed | 16× read back and timed (16.0 game-s per wall-s); the full minimap pass at 8× ran 7.86 game-s per wall-s | probe.json `speed`; run.log |
| 3 | Accelerated recording | about 10× real time (601 PNG frames for 300 game-s in 24 s and 35 s wall); lossless (png, read back); `endTime` ignored; first frame stale; one recording per sign-in | run.log recording state; probe.json `recording` |
| 4 | Camera / whole map | API camera y=50000 at fov 19 shows the whole map at 2560×1440 (≈11.6 world units per px); no Ctrl+Shift+Y needed; heights to 80000 read back | frames_topdown; probe.json `camera` |
| 5 | Vanguard / black frames | none black (top-down mean 71.4–71.6, minimap 78.5–143.1); the probe PNGs are quarantined (wrong monitor) | saved frames |
| 6 | Enemy wards on the minimap, fog off | yes, both teams' ward glyphs are drawn; the per-event strip is too small to confirm each event | v1 minimap frames t00600, t01080, t01379 |
| HDR | v1 baseline | HDR tone-maps the capture (mean 137 vs 88); HDR off alone moved the median 206.4 → 196.6 | Windows session finding I |

## Item 3: route A, minimap positions (v2)

- As recorded: median **172.3** (n=246), detection 0.765 raw / **0.778** alive-adjusted, longest gap **116 s** →
  **PARTIAL**. **These numbers include the false locks** Pyke 8287, Jax 5538, Zed 3609 at detection ≈1.0.
- **Without the false locks** (static samples dropped, `--drop-static`): 119.2 (n=187), detection 0.585. Jax stays
  wrong (8175), because post-processing cannot repair K damage in the old track.
- The fixed tracker replayed on the 57 saved frames: 519 (old) → 315 (K) → 162 (K+F), n=29–34.
- Viego is now tracked (108.6, detection 0.619) with the circle icons.
- The real post-fix numbers need the validation run.

## Item 4: wards and recalls (v2)

- **Wards, after E:** blue P 0.889 / R 0.889 (9 detected, 8 matched); red P 0.80 / R 0.571 (5 detected, 4 of 7);
  at 4 s tolerance red 1.0 / 0.714. Before E: blue 0.007, red 0.093. 16 `WARD_PLACED` in the 600–900 s window.
  Hand check 2/8 on v1 verdicts → **NO-GO until re-marked**.
- **Recalls, new rule:** P 0.857 (6/7), R 0.086 (6/70 purchase bursts); before P 0.5, R 0.014. **NO-GO**: the
  minimap track rarely has a position at the fountain during shop visits.

## Item 5: top-down (v2)

- The API camera at y=50000 shows the whole map.
- Bar hit rate **0.824**, median **127.8** (leave-one-frame-out; in-sample 88) → **GO** (≤150, ≥0.80).
- Turret-anchored oracle fit: 101.7 / LOFO 107.0; offset (51.8, 143.7) px. Window 600–900 s only.

## Item 6: docs/scout-findings.md

- Sections (a)–(g) plus the Windows-session table: facts, go/no-go per signal, the route decision with the time cost
  per game, the portal question (sent by Jasper 2026-10-17), the licence note with the CommunityDragon policy note
  (not on Riot's approved asset list; a local detection template fetched on the player's PC, never shipped or
  served; Data Dragon fallback), privacy, and what M156b covers.
- Aggregate numbers only: no match ID, no per-player data. The drift test checks the template's headings, that the
  doc is filled, and that it carries no match ID.
- **Route:** A+B with B on key windows, a written override of the rule's B-first, to be confirmed by the
  validation run.

## Commits this half

44b31cb A · 7f52dc7 B · 981baab C · 0ad8e06 H · d76df29 D · 1958842 E · 383d0e6 halfc words + R2 facts ·
3f35c52 K · 1fec258 F · 4aa2c5f I · 12d55ea J · 0be8d88 L · 797d3a4 recall · 64e31fd halfc defaults + match-ID
scrub · b56a7e3 findings · ca98bf3 validation checklist · 3e63617 card · baccfd0 A–L table

## pytest

```
tools/scout/.venv/bin/python -m pytest tools/scout -q
152 passed in 77.15s (0:01:17)
```

## The short PC checklist

`tools/scout/CHECKLIST-VALIDATE.md` (39 lines, about 30 min; phone page `publish-round-r2/checklist-validate.html`):

1. Pull the branch and reinstall.
2. Turn HDR off on the game monitor.
3. `spy start EUW1_<gameId>` for the same game.
4. Fetch the timeline first, with the development key.
5. Open the replay. No probe: its recording would use up the sign-in's one recording.
6. `spy minimap` at 8× with circle icons; move `yolov12.pt` out of the models folder first.
7. Sign out of Windows and back in.
8. `spy topdown` (camera 50000, window 600–900 s).
9. `spy score`.
10. Re-mark `handcheck.csv` against the new outputs.
11. `spy check`, then `spy send` to brain; never git.

`spy` is a PowerShell function for `python -m scout.session`.

## Questions on the card (`publish-round-r2/entry.json`, id `m156-r2-scout-findings`)

1. (react) Run the short PC validation? Tonight / This week / Later — remind me. Recommend: This week.
2. (react) Route for the Scout product round: A+B, B on key windows (recommended) / B-first / A-only.
3. (react, kind: merge) Yes, merge / Not yet — after validation. Recommend: Not yet — after validation, since the
   run may change scorer code.
4. (react) Delete the quarantined probe PNGs and the v1 run? Delete both / Keep v1, delete PNGs (recommended) /
   Keep all.

## What the validation run decides

- Route A's real error, detection and longest gap after K+F, and so whether A+B with B on key windows stands or
  B-first wins.
- The ward and recall verdicts, from the re-marked hand check.
- Whether the CommunityDragon URL answers on a real PC, and whether the HDR warning fires.
- When to merge.
