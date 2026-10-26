# Scout findings (M156 half C)

Status: half C — R2 tool fixes in, short PC validation run pending (2026-10-26)

- Runs: two runs of one of Jasper's own replays (a ranked game of about 32 minutes on the current patch).
  **v1** (2026-10-22): Windows HDR on, Data Dragon square portraits as templates; kept only as the HDR
  baseline row. **v2**: HDR off, base circle icons; every other number here is from v2. The match ID, the
  frames, the positions and the plots stay off git (the repository is public; see (f)).
- Thresholds: proposed (`tools/scout/scout/thresholds.json`, M166 A3).
- Produced from `python -m scout.halfc <run_dir>` on brain, then written by hand from that output so the
  committed text carries aggregate numbers only (no match ID, no per-player data). Lines marked "agent
  judgment" are half C's reading of the plots and frames; every other number is computed.
- R2 fixes on the branch since the run: the findings A–L and the recall rule, each with its commit, in
  "Findings from the Windows sessions" below.
- Manifest check (v2): PASS; 43.5 MB; no video, no key, no player identifiers. The probe's own screenshots
  of both runs are quarantined (they show the wrong monitor, finding A).

## Findings from the Windows sessions

What Jasper's two Windows sessions showed that the mock could not, and the fix for each. All scoring in
this file is on the v2 run; v1 appears only as the HDR baseline row in (a).

| Letter | What was found | Fix |
|---|---|---|
| A | The capture took `mss` monitor 1, not the primary monitor; the probe's screenshots show the wrong screen (quarantined). | 44b31cb: capture the primary monitor (`--monitor N` to choose) |
| B | `interfaceAll=false` + `interfaceMinimap=true` hides the minimap, although the read-back says honoured. | 7f52dc7: minimap-only keeps `interfaceAll=true`, every other `interface*` false |
| C | The API camera's height read-back does not say whether the whole map is in view. | 981baab: camera from a visual fit (50000 at 2560×1440, fov 19, scaled by aspect) |
| D | Top-down positions were off: split health-bar blobs, a guessed bar-to-unit offset, a stale first second. | d76df29: merged bar left edge, fitted offset and homography, first second dropped |
| E | Wards were mostly false (mana bars, champion pips); precision 0.007 blue, 0.093 red. | 1958842: wards from pip groups, mana excluded, stamped at first appearance, pre-existing left out |
| F | Some champions' tracks sat on one static background spot. | Superseded by K; the static-spot rule stays as a guard: 1fec258 (4 px, 20 %, two passes; `score --drop-static`, `false_lock`) |
| G | The recording ignores `endTime` and runs once per Windows sign-in (later recordings write 0 frames). | A fact, no code fix: one top-down window per sign-in (route, (c)) |
| H | The tool left the client's render settings changed after a run. | 0ad8e06: render state saved and restored |
| I | Windows HDR on the captured monitor tone-maps the capture (mean 137 vs 88). | 4aa2c5f: `winhdr.py` warns when HDR is on |
| J | The replay minimap draws each champion's base circle icon; Viego's square portrait scored 0.37 (threshold 0.55), his circle 0.63. | 12d55ea: CommunityDragon base circle icons, Data Dragon fallback |
| K | Identity bug: Pyke, Jax and Zed were found by the detector within 40 px in 45–79 % of minutes, yet tracked 3600–8300 units off at detection ~1.0; the same-label cost ignored the score, so a static dark spot captured the track. | 3f35c52: best detection per label; two synthetic repro tests that failed on the old code (`test_static_false_spot_does_not_capture_the_track`, `test_static_false_spot_offered_first_does_not_lock_the_track`) |
| L | Offline re-runs appended to `frames_minimap/`, so samples of several captures mixed. | 0be8d88: a fresh sample folder per run |
| Recall | Lucian's 08:03 recall, confirmed by Jasper in v1, was missed in v2: the arrival sample was missing and the next one sat just outside the 1000-unit fountain radius, so the stay counted 6 s, under 8 s. | 797d3a4: the stay is counted inclusively, gaps of ≤2 s are bridged, 5–8 s counts with a purchase; the recall is found again (arrival 8:04, stay 7 s, 3 purchases) |

## (a) Facts table

One row per unknown that half A could not settle without the client. Evidence names the file and key, or
the frame. v1 is the HDR baseline; v2 is the run of record.

| # | Unknown | Measured | Evidence |
|---|---|---|---|
| 1 | Minimap-only mode honoured (R2: `interfaceAll=true` + `interfaceMinimap=true`, every other `interface*` false; R1's `interfaceAll=false` + `interfaceMinimap=true` recorded as legacy) | `interfaceAll=false` + `interfaceMinimap=true` **hides** the minimap, although the read-back says both were honoured. The working combination is `interfaceAll=true` with every other `interface*` false (R2 default, fix B). | Jasper's Windows session (finding B); `probe.json` `interface` (read-back alone is misleading); `probe_minimap_only.png` mean brightness 49.2 |
| 2 | Highest honoured replay speed | **16×** read back and timed (16.0 game-s per wall-s over 3 s). The full minimap pass ran at 8×: **7.86** game-s per wall-s. | `probe.json` `speed.steps`; `run.log` (minimap pass) |
| 3 | Accelerated recording (real time or faster; lossless?) | **About 10× real time**: 601 PNG frames for 300 game-s in 24 s and 35 s wall (12.5× and 8.6×). `endTime` is ignored (sent −1 back, recording ran to 901.6). PNG, lossless read back true: **lossless yes**. The first frame is stale (from before the seek). **One recording per Windows sign-in**: later recordings in the same session write 0 frames. | `track_topdown` recording log; `/replay/recording` read-back (`codec png`, `lossless true`, `endTime`, `currentTime`); `probe.json` `recording.*` (0/40 frames: the probe recorded after another recording) |
| 4 | Camera height and whole-map view (visual fit `camera_fit`, API camera vs Ctrl+Shift+Y, `farClip`) | API camera at **y = 50000, fov 19** shows the whole map with margin at 2560×1440 (map about 1290 px, 11.6 world units per px). No Ctrl+Shift+Y needed. Heights to 80000 read back (read-back only); `farClip` read 130000. | `frames_topdown/` (whole map in every frame); `probe.json` `camera.steps`; fix C scales 50000 by aspect |
| 5 | Vanguard or capture black frames | **None black.** Top-down frame mean 71.4–71.6, minimap frames 78.5–143.1. The probe's own black check is not evidence (its PNGs show the wrong monitor). | `frames_topdown/`, `frames_minimap/` mean brightness |
| 6 | Enemy wards on the minimap with fog off | **Yes**: with `fogOfWar=false` both teams' ward glyphs are drawn on the minimap (seen at 10:00, 18:00 and 22:59 in v1). The per-event eye-check strip is too small to confirm each event. | v1 `frames_minimap/` at those times; `plots/fact6_eye_check.png` (12 events with frames) |
| HDR | Windows HDR on the captured monitor (finding I) | HDR on **tone-maps** the screen capture (mean brightness 137 vs 88). HDR off alone moved v1's minimap median from 206.4 to 196.6. | v1 vs v2 frames; v1 re-scored; `winhdr.py` now warns |

- Agent judgment, fact 3 (lossless, from the recorded PNGs): lossless yes, and fast enough: a 5-minute window takes about 30 s of wall time. The binding limit is not speed but one recording per Windows sign-in.
- Agent judgment, fact 4 (whole map in view, from `probe_camera_*.png`): the whole map is in view at y = 50000 in every top-down frame; the probe's own camera PNGs are quarantined (wrong monitor), so this rests on the top-down frames, not on the probe.
- Agent judgment, fact 6 (by eye from `plots/fact6_eye_check.png`): enemy ward glyphs are visible on the minimap with fog off; the per-event confirmation needs larger crops (M156b).

## (b) Go/no-go per signal

Thresholds from `thresholds.json` (proposed). "False locks" are the three champions (Pyke, Jax, Zed) that
the old tracker locked onto a static background spot (finding K); the rows say whether they are in.

| Signal | Measured | Threshold (proposed) | Verdict |
|---|---|---|---|
| Positions via minimap (A) | v2 (HDR off, circle icons, old tracker): median **172.3** (n=246), detection 0.765 raw / **0.778** alive-adjusted, longest gap **116 s**, false locks in; with static samples dropped (`--drop-static`) 119.2 (n=187), detection 0.585. **Fixed tracker (K+F) replayed** on the 57 saved v2 frames: 519 → 315 (K) → **162** (K+F), n = 29–34. | GO: median ≤350, detection ≥0.85, longest gap ≤15 s; PARTIAL: median ≤700, detection ≥0.60 | **PARTIAL** |
| Positions via top-down (B) | v2, after fix D: median **127.8** (leave-one-frame-out; in-sample 88), bar hit rate **0.824**; direct fit with minimap identities. Turret-anchored oracle: 101.7 (LOFO 107.0), bar offset (51.8, 143.7) px. Window 600–900 s only. | GO: median ≤150, bar hit rate ≥0.80 | **GO** |
| Wards, blue team | v2, after fix E: precision **0.889**, recall **0.889** (9 detected, 8 matched of 9 placed). Before E: precision 0.007. | GO: precision ≥0.80, recall ≥0.70, hand check ≥4 of 5 | **NO-GO** (hand check not re-marked) |
| Wards, red team | v2, after fix E: precision **0.80**, recall **0.571** (5 detected, 4 of 7 matched); at 4 s tolerance 1.0 / 0.714. Before E: precision 0.093. | GO: precision ≥0.80, recall ≥0.70, hand check ≥4 of 5 | **NO-GO** |
| Recalls | v2, new rule: precision **0.857** (6/7), recall **0.086** (6 of 70 purchase bursts). Before: 0.5 / 0.014. | GO: precision ≥0.80, recall ≥0.70, hand check ≥4 of 5 | **NO-GO** |

Why each verdict:

- Positions (A): median 172.3 ≤ 700 and detection 0.778 ≥ 0.60, but detection is under 0.85 and the
  longest gap is 116 s, so PARTIAL, not GO. These v2 numbers **include** the three false locks (errors
  3609–8287 at detection 0.999–1.0). The detector found those champions within 40 px of truth in 45–79 %
  of minutes; the tracker took a static spot because its same-label cost ignored the score (K, fixed).
  `--drop-static` cannot repair it after the fact (Jax stays at 8175), so the post-fix number needs a new
  run.
- Positions (B): median 127.8 ≤ 150 and bar hit rate 0.824 ≥ 0.80. Only one 5-minute window (600–900 s)
  and five timeline frames to fit on.
- Wards, blue: precision and recall pass; the hand check does not. Jasper's verdicts in v2's
  `handcheck.csv` were carried over from v1 and describe R1's false wards (2 of 8 agree), so they do not
  judge the R2 output.
- Wards, red: recall 0.571 is under 0.70 at the 2 s tolerance (0.714 at 4 s); hand check as for blue.
- Recalls: precision passes, recall does not. The minimap track rarely has a position at the fountain
  during shop visits (dead, a gap, or hidden under the fountain HUD), so most purchase bursts have no
  arrival to match.
- Hand check (`handcheck.csv`, `jasper_says` beside `auto_says`): 2 of 9 marked rows agree (wards 2/8,
  recalls 0/1), but the verdicts are v1's, carried over; they must be re-marked on the R2 output.

- Agent judgment, positions (A): PARTIAL with a known, fixed bug inflating the error; the fixed tracker brings the 57-frame sample to 162, so GO on the median is plausible, but detection and the 116 s gap need the validation run.
- Agent judgment, positions (B): GO on accuracy; usable only for chosen windows, because each window needs its own Windows sign-in.
- Agent judgment, wards: the automatic numbers after fix E are close to GO (blue passes, red short on recall); the verdict waits on Jasper re-marking the hand check against the new outputs.
- Agent judgment, recalls: the stay rule is now precise (Lucian's 08:03 recall is found again, see "Recall" in the findings table), but recall coverage is a tracking problem at the fountain, not a rule problem; NO-GO for this round.

## (c) Route decision for the Scout product round

- Route by the rule: **B-first** (A not GO, B GO; recording is accelerated, so the real-time clause does
  not apply). `scout.halfc` printed "A+B (B on key windows)" because its probe saw no recording rate; with
  fact 3 measured, the rule gives B-first.
- Reason: B meets both GO bounds; A is PARTIAL.
- Time cost per game (facts 2 and 3): minimap pass at 8× ≈ game length ÷ 7.9 of wall time (about 4 min for
  a 32-minute game). Top-down: a 5-minute window ≈ 30 s of wall time (24–35 s measured), **plus a Windows
  sign-out and sign-in per window**, because a session records only once.
- Agent judgment (the rule may be overridden only with a written reason): **override to A+B, B on key windows.** B cannot be the primary source for a whole game: it covers one window per Windows sign-in (`endTime` is ignored), so a full game would take about seven sign-ins. A is the only full-game continuity source, and its PARTIAL is inflated by a bug that is now fixed (K+F). So: A for the whole game, B on the windows that matter (fights, objectives). The short validation run (`tools/scout/CHECKLIST-VALIDATE.md`) confirms or reverses this.

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

- Riot's answer: none recorded by 2026-10-26. Nothing in the product round uploads tracks before an answer (and before L-1, see (f)).

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
fallback. The CommunityDragon URL pattern has not been checked against the network from brain (the tests
open no socket); the validation run checks it.

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
- **Quarantined PNGs.** The wrong-monitor probe screenshots of both runs are quarantined on brain and wait
  for Jasper's word to delete them.

Jasper's adviser read of this paragraph stays open.

## (g) What M156b would cover

The Scout product round (or an M156b) picks up what this spike leaves open:

- **The validation numbers.** Route A's real error and detection after K+F come only from 57 mixed frames;
  the short PC run (`tools/scout/CHECKLIST-VALIDATE.md`) measures them on a full game.
- **The ward and recall hand check**, re-marked against the R2 outputs (the v2 file carries v1's verdicts).
- **Recall coverage**: positions at the fountain during shop visits (dead time, gaps, the fountain HUD).
- **The YOLOv12 weights**: not tried (template detector only); class list, accuracy and a clean-licence
  detector.
- **Nexus anchors** for the top-down homography: still unverified on real frames (the fit uses turrets and
  timeline frames).
- **The CommunityDragon circle-icon URL**: unchecked against the network.
- **The static-spot rule's live running count**: written but unused (offline passes only); its 60-frame
  warm-up means it needs at least 60 frames.
- A larger minimap ward crop for fact 6's per-event check, and whether `/liveclientdata/playerlist` answers
  in replay mode.
