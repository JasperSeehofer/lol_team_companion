# Scout findings (M156 half C)

Status: half C — final (M156 R3, 2026-11-12): validated by the 2026-10-28 run; route A+B with B on key windows, ruled by Jasper on 2026-11-08

- Runs: three runs of one of Jasper's own replays (a ranked game of about 32 minutes on the current patch).
  **v1** (2026-10-22): Windows HDR on, Data Dragon square portraits; only the HDR baseline row in (a), and
  deleted since (Jasper, 2026-11-08). **v2** (2026-10-26): HDR off, base circle icons, tracker before K.
  **Validation run** (2026-10-28, "VR" below): HDR off, circle icons, the R2 code with K and F, no probe
  (`tools/scout/CHECKLIST-VALIDATE.md`). The match ID, the frames, the positions and the plots stay off
  git (the repository is public; see (f)).
- Thresholds: proposed (`tools/scout/scout/thresholds.json`, M166 A3).
- Produced from `python -m scout.halfc <run_dir>` on brain (v2 and VR re-scored with the same R3 scorer),
  then written by hand from that output so the committed text carries aggregate numbers only (no match
  ID, no per-player data). Lines marked "agent judgment" are half C's reading of the plots and frames;
  every other number is computed.
- Code Jasper ran on 2026-10-28, read from the run itself: `track_minimap.json` carries `static_spots`
  (F, after K), `icon_sources` (J), `hdr` and the monitor record (I, A), and `run.log` shows L removing
  the earlier pass's samples; so the VR's route A numbers are **after K+F**. The parallel detection and
  Viego's possession rule (99eba90) came later and were not in it.
- Manifest check: PASS for both (v2 43.5 MB, VR 45.3 MB); no video, no key, no player identifiers. The
  probe's quarantined screenshots were deleted with v1 (Jasper, 2026-11-08).

## Findings from the Windows sessions

What Jasper's Windows sessions showed that the mock could not, and the fix for each. Scoring in this file
is on v2 and the validation run, both with the R3 scorer; v1 appears only as the HDR baseline row in (a).

| Letter | What was found | Fix |
|---|---|---|
| A | The capture took `mss` monitor 1, not the primary monitor; the probe's screenshots show the wrong screen (deleted). | 44b31cb: capture the primary monitor (`--monitor N` to choose) |
| B | `interfaceAll=false` + `interfaceMinimap=true` hides the minimap, although the read-back says honoured. | 7f52dc7: minimap-only keeps `interfaceAll=true`, every other `interface*` false |
| C | The API camera's height read-back does not say whether the whole map is in view. | 981baab: camera from a visual fit (50000 at 2560×1440, fov 19, scaled by aspect) |
| D | Top-down positions were off: split health-bar blobs, a guessed bar-to-unit offset, a stale first second. | d76df29: merged bar left edge, fitted offset and homography, first second dropped |
| E | Wards were mostly false (mana bars, champion pips); precision 0.007 blue, 0.093 red. | 1958842: wards from pip groups, mana excluded, stamped at first appearance, pre-existing left out |
| F | Some champions' tracks sat on one static background spot. | Superseded by K; the static-spot rule stays as a guard: 1fec258 (4 px, 20 %, two passes; `score --drop-static`, `false_lock`) |
| G | The recording ignores `endTime`. **Corrected in R3:** "one recording per Windows sign-in" is withdrawn. Jasper reports several recordings working in the same sign-in (2026-11-08); the 0-frame recordings of 2026-10-22 were counted by code that did not look into the client's own recording subfolder. | 944b0dc (top-down, 2026-10-22) and 395ee78 (probe, R3): count the frames below the client's subfolder; no sign-out step (see "Finding G" below) |
| H | The tool left the client's render settings changed after a run. | 0ad8e06: render state saved and restored |
| I | Windows HDR on the captured monitor tone-maps the capture (mean 137 vs 88). | 4aa2c5f: `winhdr.py` warns when HDR is on; on the VR it read HDR off for the captured monitor, matched by rectangle |
| J | The replay minimap draws each champion's base circle icon; Viego's square portrait scored 0.37 (threshold 0.55), his circle 0.63. | 12d55ea: CommunityDragon base circle icons, Data Dragon fallback; on the VR all ten came from CommunityDragon (five `_circle.png`, five `_circle_0.png`) |
| K | Identity bug: Pyke, Jax and Zed were found by the detector within 40 px in 45–79 % of minutes, yet tracked 3600–8300 units off at detection ~1.0; the same-label cost ignored the score, so a static dark spot captured the track. | 3f35c52: best detection per label; two synthetic repro tests that failed on the old code (`test_static_false_spot_does_not_capture_the_track`, `test_static_false_spot_offered_first_does_not_lock_the_track`). **Confirmed on the VR:** no false lock; Pyke, Jax and Zed 146, 163 and 75 units |
| L | Offline re-runs appended to `frames_minimap/`, so samples of several captures mixed. | 0be8d88: a fresh sample folder per run |
| M | VR: the first minimap pass ran with the automatic box (a square of 24 % of the screen height, 341 px); Jasper's minimap is drawn at 679 px, so the pass found 2859 positions in 1946 samples. The second pass with `--minimap-box` found 13083. | No code fix in the spike; the product round should find the box on the first frame (see (g)) |
| N | VR: the top-down pass was fitted on the PC against that first minimap pass, so the fit found no pairs and route B scored PENDING. | ae034b3: `halfc` refits a fit without pairs against the run's minimap track, and reads the recording rate from the top-down pass when the probe has none |
| Recall | Lucian's 08:03 recall, confirmed by Jasper in v1, was missed in v2: the arrival sample was missing and the next one sat just outside the 1000-unit fountain radius, so the stay counted 6 s, under 8 s. | 797d3a4: the stay is counted inclusively, gaps of ≤2 s are bridged, 5–8 s counts with a purchase; the recall is found again (arrival 8:04, stay 7 s, 3 purchases) |

**Finding G, the evidence.** Every recording in the run logs:

- 2026-10-22, 20:57: the probe's two recordings (real time, accelerated) report 0 of 40 frames each. The
  client read the request back (`recording: true`, `codec png`) and reported a path inside its own
  subfolder (`<given path>/<patch>_<game>_01`); the probe counted PNGs only at the top of the given
  path. (Whether v1's recordings earlier that day were in the same sign-in is not known; v1's log is
  deleted.)
- 2026-10-22, 21:11 and 21:14: two top-down recordings, "0 png frames", counted the same way. The fix that
  searches the subfolder (944b0dc) was committed at 21:18 that evening; the probe kept the old count
  until 395ee78 (R3).
- 2026-10-26, 19:39 and 20:31: two top-down recordings, 601 frames each (24 s and 35 s wall). The log
  does not say whether Windows was signed out between them.
- 2026-10-28, 09:35: one top-down recording, 601 frames in 33 s (the checklist asked for a sign-out first).

So the logs show at most two successful recordings on one day and never record a sign-in; they do not
prove several recordings per sign-in. Jasper's report does, and nothing in the logs contradicts it. The
2026-10-22 zeros are most likely the counting bug; their cause is not proven, because the PC's scratch
folders were not inspected. "One recording per sign-in" was an assumption carried from R1's checklist
(and the test mock's `one_recording` quirk) before any Windows evidence; it is withdrawn, the checklists
no longer ask for a sign-out, and the mock no longer ignores a second recording by default.

## (a) Facts table

One row per unknown that half A could not settle without the client. Evidence names the file and key, or
the frame. v1 is the HDR baseline; v2 and the VR are the runs of record.

| # | Unknown | Measured | Evidence |
|---|---|---|---|
| 1 | Minimap-only mode honoured (R2: `interfaceAll=true` + `interfaceMinimap=true`, every other `interface*` false; R1's `interfaceAll=false` + `interfaceMinimap=true` recorded as legacy) | `interfaceAll=false` + `interfaceMinimap=true` **hides** the minimap, although the read-back says both were honoured. The working combination is `interfaceAll=true` with every other `interface*` false (R2 default, fix B); the VR's two minimap passes used it. | Jasper's Windows session (finding B); `probe.json` `interface` (read-back alone is misleading); VR `frames_minimap/` |
| 2 | Highest honoured replay speed | **16×** read back and timed (16.0 game-s per wall-s over 3 s). The full minimap pass ran at 8×: **7.86** game-s per wall-s on v2 and on both VR passes (1946 frames in 248 s). | `probe.json` `speed.steps`; `run.log` (minimap pass); VR `track_minimap.json` `capture` |
| 3 | Accelerated recording (real time or faster; lossless?) | **About 9× real time**: 601 PNG frames (2 per game-s) for 300 game-s in 24 s and 35 s wall on v2, 33 s on the VR (8.7 and 9.0 game-s per wall-s by `halfc`). `endTime` is ignored (sent −1 back, recordings ran to 900.6–901.6). PNG, lossless read back true: **lossless yes**. The first frame is stale (from before the seek). The client writes into its own subfolder and records at the screen size (2560×1440), not the requested 1920×1080. **Several recordings per sign-in work** (Jasper, 2026-11-08; finding G corrected). | `run.log` "client recording state at stop" and "recording: 601 png frames"; `track_topdown.json` `recording`; `probe.json` `recording.*` (0/40: counted outside the client's subfolder) |
| 4 | Camera height and whole-map view (visual fit `camera_fit`, API camera vs Ctrl+Shift+Y, `farClip`) | API camera at **y = 50000, fov 19** shows the whole map with margin at 2560×1440 (map about 1290 px, 11.6 world units per px). No Ctrl+Shift+Y needed. Heights to 80000 read back (read-back only); `farClip` read 130000. | v2 and VR `frames_topdown/` (whole map in every frame); `probe.json` `camera.steps`; fix C scales 50000 by aspect |
| 5 | Vanguard or capture black frames | **None black.** Top-down frame mean 71.4–71.6, minimap frames 78.5–143.1. The probe's own black check is not evidence (its PNGs showed the wrong monitor). | `frames_topdown/`, `frames_minimap/` mean brightness |
| 6 | Enemy wards on the minimap with fog off | **Yes**: with `fogOfWar=false` both teams' ward glyphs are drawn on the minimap (seen at 10:00, 18:00 and 22:59 in v1). The per-event eye-check strip is too small to confirm each event. | v1 `frames_minimap/` at those times; `plots/fact6_eye_check.png` (12 events with frames, v2 and VR) |
| HDR | Windows HDR on the captured monitor (finding I) | HDR on **tone-maps** the screen capture (mean brightness 137 vs 88). HDR off alone moved v1's minimap median from 206.4 to 196.6. On the VR the check ran on real Windows and read HDR off for the captured monitor (matched by rectangle). | v1 vs v2 frames; v1 re-scored; VR `track_minimap.json` `hdr` |

- Agent judgment, fact 3 (lossless, from the recorded PNGs): lossless yes, and fast: a 5-minute window takes about 30 s of wall time, so a whole game is about 4 minutes. With several recordings per sign-in the binding limit is neither speed nor sign-ins but the fit (one timeline frame a minute) and disk (601 full-screen PNGs per window on the PC).
- Agent judgment, fact 4 (whole map in view, from `probe_camera_*.png`): the whole map is in view at y = 50000 in every top-down frame of v2 and the VR; the probe's own camera PNGs showed the wrong monitor and are deleted, so this rests on the top-down frames, not on the probe.
- Agent judgment, fact 6 (by eye from `plots/fact6_eye_check.png`): enemy ward glyphs are visible on the minimap with fog off; the per-event confirmation needs larger crops (M156b).

## (b) Go/no-go per signal

Thresholds from `thresholds.json` (proposed). v2 and the validation run (VR) scored by the same R3
`halfc`. The verdict column is the VR's. "False locks" are v2's Pyke, Jax and Zed (finding K).

| Signal | v2 (2026-10-26, before K) | Validation run (2026-10-28, after K+F) | Threshold (proposed) | Verdict |
|---|---|---|---|---|
| Positions via minimap (A) | median **172.3** (n=246), detection 0.765 raw / **0.778** alive-adjusted, longest gap **116 s**, 599 gaps; false locks in (Pyke 8287, Jax 5538, Zed 3609, detection ≈1.0); Viego 108.6. Static samples dropped: 119.2 (n=187), detection 0.585. | median **112.5** (n=219), detection 0.672 raw / **0.686** alive-adjusted, longest gap **120 s**, 760 gaps; **no false lock**: Pyke 146, Jax 163, Zed 75, Viego 112.5; static spots rejected: 7 (Jax 2, Pyke 2, Zed 3), none for the other seven | GO: median ≤350, detection ≥0.85, longest gap ≤15 s; PARTIAL: median ≤700, detection ≥0.60 | **PARTIAL** |
| Positions via top-down (B) | LOFO median **127.8** (in-sample 88; 32 pairs, 30 inliers), bar hit rate **0.824**; direct fit, minimap identities | LOFO median **124.7** (in-sample 113.6; 40 pairs, 38 inliers), bar hit rate **0.827**; direct fit, minimap identities (refitted by `halfc`, finding N) | GO: median ≤150, bar hit rate ≥0.80 | **GO** |
| Wards, blue team | precision **0.889**, recall **0.889** (9 detected, 8 of 9 placed); hand check 2/8 (v1's verdicts, carried over) | precision **0.889**, recall **0.889** (same 9 / 8 of 9); hand check **5/5** re-marked (3/3 on blue rows) | GO: precision ≥0.80, recall ≥0.70, hand check ≥4 of 5 | **GO** |
| Wards, red team | precision **0.80**, recall **0.571** (5 detected, 4 of 7); 1.0 / 0.714 at 4 s | precision **0.80**, recall **0.571** (same); hand check 5/5 (2/2 on red rows) | GO: precision ≥0.80, recall ≥0.70, hand check ≥4 of 5 | **NO-GO** |
| Recalls | precision **0.857** (6/7), recall **0.086** (6 of 70 bursts); hand check 0/1 | precision **0.875** (7/8), recall **0.100** (7 of 70 bursts); hand check **2/5** | GO: precision ≥0.80, recall ≥0.70, hand check ≥4 of 5 | **NO-GO** |

Why each verdict:

- Positions (A): K did what it was for. The false locks are gone and the median fell from 172.3 to 112.5,
  inside the GO bound. Detection fell from 0.778 to 0.686, because v2's figure counted the three false
  locks as seen in almost every second. Against v2 with its static samples dropped (0.585), the VR is
  higher. The longest gap is 120 s (Lucian), so detection and gaps miss GO by far. **K fixed accuracy, not
  coverage: A stays PARTIAL on a full 1 Hz run.**
- Positions (B): median 124.7 ≤ 150 and bar hit rate 0.827 ≥ 0.80, on a second, independent recording of
  the same window (600–900 s). The fit needs A's identities: on the PC it was fitted against the wrong-box
  minimap pass and found no pairs (finding N).
- Wards: two independent recordings gave the same 14 new ward events, so the detector is repeatable on
  one game. Jasper re-marked the hand check against the VR's rows (every row is in the R3 output) and
  agreed with all five ward rows; one note says a ward was placed at 11:48, not at the 11:51 the tool
  stamped. Blue meets every bound. Red misses recall (0.571 < 0.70 at 2 s).
- Recalls: precision passes, recall does not (0.10). Hand check 2/5: one "no" is a timing difference
  (Jasper puts the recall at 10:57, the tool at 10:59), one a fountain reset, and one the stay without a
  purchase (forced back by the enemy team). With the timing row read as found it would be 3/5, still under 4.

- Agent judgment, positions (A): PARTIAL on a full run after K+F. The accuracy is there (median 112.5); the coverage is not (detection 0.686, 120 s gaps). The gaps are icons hidden under other icons, the fountain and fights, not identity errors; this is the detector's limit, not a tracker bug left open.
- Agent judgment, positions (B): GO on accuracy on two recordings of one window; recording is cheap (about 30 s per 5 minutes, several per sign-in), so more windows cost wall time and disk, not sign-ins.
- Agent judgment, wards: blue GO; red short on recall with the same detections in both runs, so a rule change, not luck, would be needed; the ward stamp can lag placement by a few seconds.
- Agent judgment, recalls: the stay rule is precise, coverage is the problem (no minimap position at the fountain during most shop visits); NO-GO for this round.

## (c) Route decision for the Scout product round

- **Route as ruled: A+B with B on key windows** (Jasper, 2026-11-08). A runs for the whole game; B is
  recorded on the windows that matter (fights, objectives) and replaces A's positions there.
- What the rule gives, for the record: **B-first** (A PARTIAL, B GO; recording is accelerated, about 9×
  real time, so the real-time clause does not apply). `halfc` now prints B-first for both v2 and the VR;
  in R2 it printed "A+B (B on key windows)" for v2 only because the probe's 0-frame counts made recording
  look real time only (finding G, fixed in 395ee78 and ae034b3).
- Why the ruling departs from the rule: B's fit takes its identities from A's track (no A, no B), and its
  accuracy rests on one 5-minute window with five timeline frames. A is the full-game continuity source
  either way. R2's further reason, one B window per Windows sign-in, has fallen with finding G: the ruling
  stands on the first two reasons.
- Time cost per game (32.5 min, VR): minimap pass at 8× **4.1 min** (7.83 game-s per wall-s measured),
  then its detection (17 min serial for a 679 px minimap on Jasper's PC; 99eba90 runs it in parallel
  since). Top-down: **about 30 s per 5-minute window** (24–35 s measured, about 9×), **no sign-out**, plus
  about 35 s of processing per window; the whole game would be about 3.6 min of recording. B on more
  windows, up to the whole game, is cheap in time; disk (601 full-screen PNGs per window) and the fit are
  the limits.

The rule (thresholds are proposed; Jasper reacts on the M166 card):

1. A GO and B GO: A+B, with B as the primary source where recorded and A for full-game continuity.
2. B GO but recording is real time only: A+B with B on key windows.
3. B GO without A: B-first.
4. A GO without B, or B not reached: A-only.
5. Neither GO: no-go. The board stays on tier 0+1, and Scout is retired on evidence.
6. Half C may override the rule only with a written reason.

## (d) The portal question

Drafted on 2026-10-17 for product 834192 (`.planning/missions/M156/portal-question.md`); Jasper sent it as
written on 2026-10-17 ("Send as is — already submitted the ticket"). Quoted:

> **Subject:** Product 834192 (Team Companion): offline review of a player's own replays, and CommunityDragon icons
>
> Hello,
>
> Team Companion (product 834192) is a private coaching workspace for one amateur team. We have two questions before we build anything.
>
> 1. Offline replay review. We are considering a separate tool, free and open source (AGPL-3.0), that a player runs on their own Windows PC on a replay they downloaded themselves. It uses only the official local Replay API (127.0.0.1:2999, which the player switches on with EnableReplayApi=1) and screen capture of the replay's minimap or zoomed-out view. From these it extracts per-second champion positions, ward placements and recalls for post-game review. It does no memory reading, no injection and no .rofl decoding, and it never runs during a live match. The player would then upload the derived tracks to their own team's private workspace. The tracks include the opponents' positions, because those are in the player's replay. They would be visible only to that team, deletable, and kept no longer than our match cache (12 months). Is this acceptable? Should the tool be registered under this product or separately?
>
> 2. Icons. May Team Companion reuse CommunityDragon's position (role) and objective icons, served from our own domain alongside Data Dragon? Would a map derived from the map texture be allowed? Today we only measure coordinates and draw our own schematic.
>
> Thank you for your time.
>
> Kind regards,
> Jasper Seehofer, Team Companion

- Riot's answer: none recorded by 2026-11-12. Nothing in the product round uploads tracks before an answer (and before L-1, see (f)).

## (e) Licence note

Folded in from `.planning/missions/M156/licence-note.md` (checked 2026-10-17). Nothing here chooses a
licence for the app.

| Component | Licence | Note |
|---|---|---|
| Scout (`tools/scout/`) | AGPL-3.0-or-later (`pyproject.toml` `license`; `tools/scout/LICENSE` is the AGPLv3 text) | A separate tool beside the app, not linked into the Rust crate, sharing no code with it. |
| pyLoL reference copy (`vendor/pyLoL/`, SHA `19afccf`) | GPL-3.0 (its `LICENSE`; its `setup.py` says MIT, the `LICENSE` file governs) | Reference code only; nothing imports it. |
| GPL-3.0 combined with AGPL-3.0 | Allowed (GPLv3 §13, AGPLv3 §13) | The GPL parts keep their terms; the combined work carries AGPL §13's network clause. |
| ultralytics (detector runtime) | AGPL-3.0 (a paid Enterprise Licence is the alternative) | Not used on Jasper's runs (template detector only). |
| YOLOv12 | AGPL-3.0 | |
| pyLoL's `yolov12.pt` weights | No stated licence | Treat as AGPL-derived; a risk for the Scout product round. Not tried in this spike. |
| opencv-python, numpy, scipy, mss | Apache-2.0, BSD-3-Clause, BSD-3-Clause, MIT | |
| League Director | Apache-2.0 | Field reference for Replay API names only; no code copied. |
| The app (`lol_team_companion`) | Undecided | No `LICENSE` at the repo root; not chosen here. |

Detection templates (fix J): the replay minimap draws each champion's **base circle icon**, so Scout fetches
`<c>_circle.png` (then `<c>_circle_0.png`) from CommunityDragon
(`raw.communitydragon.org/latest/game/assets/characters/<c>/hud/`) on the player's PC. CommunityDragon is
**not on Riot's approved asset list**. The icon is a local detection template: it is fetched and cached on
the player's PC, never shipped in the tool and never served by the app. Data Dragon's square portrait is the
fallback. The URL pattern answered on Jasper's PC on 2026-10-28 (all ten champions; the second pass used
the cache); the tests on brain open no socket.

For the Scout product round:

- Bundling an ultralytics-derived model or runtime makes the helper AGPL-3.0, or needs the Ultralytics
  Enterprise Licence.
- ONNX export changes the format, not the licence question for the weights.
- AGPL §13 matters if a detector ever runs server-side in the app. As long as detection runs on the
  player's PC and the app receives plain tracks (data, not code), the app's own licence stays independent.
- pyLoL's GPL-3.0 adds no constraint beyond the AGPL, provided the helper is AGPL.

## (f) Privacy

The tracks Scout derives from a player's own replay contain all ten champions' positions, including the
opponents'. Those opponents are identifiable players, so the tracks are personal data. The legal-privacy
audit's F01 applies: Art. 6(1)(f), 14, 17 and 21 GDPR, with scouted third parties counted as personal data
(garden `analyses/team-companion-audit-legal-privacy-2026-10.md`, §2 F01, scouting section; the Art. 14
notice outline is §4b item 9). The rules:

- **No upload yet.** Derived tracks are not uploaded into Team Companion until **L-1** ships: the audit's
  Art. 14 notice plus the objection and purge route across all teams. L-1 is a product prerequisite of the
  Scout product round.
- **When uploaded**, they are visible only to that team. They are minimised to per-second positions, ward
  placements and recalls, with no video, frames or chat. The team can delete them, and account or team
  deletion removes them. They are kept **12 months**, through the same nightly retention as
  `MATCH_CACHE_RETENTION_DAYS`.
- **The spike's own run data never goes to git.** The repository is public. Runs travel by
  `python -m scout.session send` (one `scp`) to brain's `~/.local/share/scout/inbox/` and are kept local
  there; `tools/scout/runs/` is git-ignored, and so are the round's plot and frame-strip folders. This
  file holds aggregate numbers only: no match ID, frames, positions or per-player identifiers.
- **Deleted on Jasper's word (2026-11-08):** the v1 run and the quarantined wrong-monitor probe
  screenshots. v2 and the validation run stay on brain.

Jasper's adviser read of this paragraph stays open.

## (g) What M156b would cover

The Scout product round (or an M156b) picks up what this spike leaves open:

- **Route A's coverage**: detection 0.686 and gaps up to 120 s after K+F, on one game. Overlapping icons,
  the fountain and fights; more games before any threshold is relaxed.
- **The minimap box** (finding M): find it on the first frame instead of the 24 % guess, or remember it
  per PC; a wrong box cost the VR one minimap pass and broke the first top-down fit (N).
- **Red wards' recall** (0.571 at 2 s) and the stamp lag of a few seconds; more games for the ward rule.
- **Recall coverage**: positions at the fountain during shop visits (dead time, gaps, the fountain HUD).
- **B on more windows**: several recordings per sign-in work, so the product round can choose the windows
  freely; the limits are disk on the PC and the fit (timeline frames once a minute, A's identities). The
  top-down pass sizes the camera from its requested 1920×1080, not the screen it records (harmless at
  16:9, wrong on other aspects).
- **The YOLOv12 weights**: not tried (template detector only); class list, accuracy and a clean-licence
  detector.
- **Nexus anchors** for the top-down homography: still unverified on real frames (the fit uses turrets and
  timeline frames).
- **The static-spot rule's live running count**: written but unused (offline passes only); its 60-frame
  warm-up means it needs at least 60 frames.
- **HDR on**: the check reads HDR off correctly on Windows; the warning itself has not fired on a real PC.
- A larger minimap ward crop for fact 6's per-event check, and whether `/liveclientdata/playerlist` answers
  in replay mode.
