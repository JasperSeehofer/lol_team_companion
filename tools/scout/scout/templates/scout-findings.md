# Scout findings (M156 half C)

{{status}}

- Run: {{run}}
- Thresholds: {{thresholds_status}} (`tools/scout/scout/thresholds.json`)
- Produced by `python -m scout.halfc <run_dir>`; the machine-readable numbers are in `halfc-summary.json`
  beside the plots. Lines marked "agent judgment" are written by half C after it has looked at the plots
  and frames; every other value is computed.
- Manifest check: {{manifest}}

## (a) Facts table

One row per unknown that half A could not settle without the client. Evidence names the file and key, or
the frame.

| # | Unknown | Measured | Evidence |
|---|---|---|---|
| 1 | Minimap-only mode honoured (`interfaceAll=false` + `interfaceMinimap=true`) | {{fact1_value}} | {{fact1_evidence}} |
| 2 | Highest honoured replay speed | {{fact2_value}} | {{fact2_evidence}} |
| 3 | Accelerated recording (real time or faster; lossless?) | {{fact3_value}} | {{fact3_evidence}} |
| 4 | Camera height and whole-map view (API camera vs Ctrl+Shift+Y, `farClip`) | {{fact4_value}} | {{fact4_evidence}} |
| 5 | Vanguard or capture black frames | {{fact5_value}} | {{fact5_evidence}} |
| 6 | Enemy wards on the minimap with fog off | {{fact6_value}} | {{fact6_evidence}} |

- Agent judgment, fact 3 (lossless, from the recorded PNGs): _(agent judgment: half C writes this line)_
- Agent judgment, fact 4 (whole map in view, from `probe_camera_*.png`): _(agent judgment: half C writes this line)_
- Agent judgment, fact 6 (by eye from `plots/fact6_eye_check.png`): _(agent judgment: half C writes this line)_

## (b) Go/no-go per signal

| Signal | Measured | Threshold (proposed) | Verdict |
|---|---|---|---|
| Positions via minimap (A) | {{a_numbers}} | {{a_threshold}} | {{a_verdict}} |
| Positions via top-down (B) | {{b_numbers}} | {{b_threshold}} | {{b_verdict}} |
| Wards, blue team | {{wards_blue_numbers}} | {{wards_threshold}} | {{wards_blue_verdict}} |
| Wards, red team | {{wards_red_numbers}} | {{wards_threshold}} | {{wards_red_verdict}} |
| Recalls | {{recalls_numbers}} | {{recalls_threshold}} | {{recalls_verdict}} |

Why each verdict:

- Positions (A): {{a_reason}}
- Positions (B): {{b_reason}}
- Wards, blue: {{wards_blue_reason}}
- Wards, red: {{wards_red_reason}}
- Recalls: {{recalls_reason}}
- Hand check (`handcheck.csv`, `jasper_says` beside `auto_says`): {{handcheck}}

- Agent judgment, positions (A): _(agent judgment: half C writes this line)_
- Agent judgment, positions (B): _(agent judgment: half C writes this line)_
- Agent judgment, wards: _(agent judgment: half C writes this line)_
- Agent judgment, recalls: _(agent judgment: half C writes this line)_

## (c) Route decision for the Scout product round

- Route by the rule: **{{route}}**
- Reason: {{route_reason}}
- Time cost per game (facts 2 and 3): {{time_cost}}
- Agent judgment (the rule may be overridden only with a written reason): _(agent judgment: half C writes this line)_

The rule (thresholds are proposed; Jasper reacts on the M166 card):

1. A GO and B GO: A+B, with B as the primary source where recorded and A for full-game continuity.
2. B GO but recording is real time only: A+B with B on key windows.
3. B GO without A: B-first.
4. A GO without B, or B not reached: A-only.
5. Neither GO: no-go. The board stays on tier 0+1, and Scout is retired on evidence.
6. Half C may override the rule only with a written reason.

## (d) The portal question

Drafted on 2026-10-17 for product 834192 (`.planning/missions/M156/portal-question.md`); the M166 page
records it as sent that day. Quoted:

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

- Riot's answer: _(agent judgment: half C writes this line)_

## (e) Licence note

Folded in from `.planning/missions/M156/licence-note.md` (checked 2026-10-17). Nothing here chooses a
licence for the app.

| Component | Licence | Note |
|---|---|---|
| Scout (`tools/scout/`) | AGPL-3.0-or-later (`pyproject.toml` `license`; `tools/scout/LICENSE` is the AGPLv3 text) | A separate tool beside the app, not linked into the Rust crate, sharing no code with it. |
| pyLoL reference copy (`vendor/pyLoL/`, SHA `19afccf`) | GPL-3.0 (its `LICENSE`; its `setup.py` says MIT, the `LICENSE` file governs) | Reference code only; nothing imports it. |
| GPL-3.0 combined with AGPL-3.0 | Allowed (GPLv3 §13, AGPLv3 §13) | The GPL parts keep their terms; the combined work carries AGPL §13's network clause. |
| ultralytics (detector runtime) | AGPL-3.0 (a paid Enterprise Licence is the alternative) | |
| YOLOv12 | AGPL-3.0 | |
| pyLoL's `yolov12.pt` weights | No stated licence | Treat as AGPL-derived; a risk for the Scout product round. |
| opencv-python, numpy, scipy, mss | Apache-2.0, BSD-3-Clause, BSD-3-Clause, MIT | |
| League Director | Apache-2.0 | Field reference for Replay API names only; no code copied. |
| The app (`lol_team_companion`) | Undecided | No `LICENSE` at the repo root; not chosen here. |

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
audit's F01 applies: Art. 6(1)(f), 14, 17 and 21 GDPR, with scouted third parties counted as personal data.
The rules:

- **No upload yet.** Derived tracks are not uploaded into Team Companion until **L-1** ships: the audit's
  Art. 14 notice plus the objection and purge route across all teams.
- **When uploaded**, they are visible only to that team. They are minimised to per-second positions, ward
  placements and recalls, with no video, frames or chat. The team can delete them, and account or team
  deletion removes them. They are kept **12 months**, through the same nightly retention as
  `MATCH_CACHE_RETENTION_DAYS`.
- **The spike's own run dir.** It is minimised (no PUUID, Riot ID or summoner name in `match.json` or
  `timeline.json`). It never goes to GitHub (the repository is public): it travels by `scp` to brain's
  `~/.local/share/scout/inbox/` and `tools/scout/runs/` is git-ignored. M166 recommends that M156's landing
  does not merge any run data into main.

Jasper's adviser read of this paragraph stays open.

## (g) What M156b would cover

- A minimap ward detector (fact 6 is read by eye in this spike).
- HSV health-bar parameters and the bar-to-unit offset tuned on real top-down frames.
- The nexus centres and the fountain radius, measured on the run's frames (marked "verify on run").
- Whether `/liveclientdata/playerlist` answers in replay mode.
- The YOLOv12 weights' class list against the current champion list, and a clean-licence detector.
- Not reached on this run: {{not_reached}}
