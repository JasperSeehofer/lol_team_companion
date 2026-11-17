Status: R3 done — validation run scored, findings final, ready to land

# M156 — Scout spike — REPORT-R3 (validation run, final)

Branch `mission/M156-scout-replay-spike`: commits ae034b3, 395ee78 and d54fad5 after 130a9ad, plus this report.
Run data stays on brain (the inbox and the job's temp dir: `tmp/run-1028`, `tmp/halfc-1028`, `tmp/halfc-v2-r3`)
and none of it is in git. The committed text carries aggregate numbers only, with no match ID.

## Budget

- Declared: 6.0 M, cap 8.0 M (raised in earlier rounds).
- This round: one Opus executing subagent, about 0.17 M tokens (scoring both runs with the same scorer, two
  scorer/probe fixes with tests, the findings and the checklists), plus the orchestrator.
- **The overseer's meter is the figure.**

## Least-confident decisions

1. **Finding G is withdrawn on Jasper's report, not on log evidence.** The logs never record a sign-in. They show
   two successful recordings on one day (2026-10-26, 19:39 and 20:31), with no record of a sign-out between them.
   The 0-frame recordings of 2026-10-22 were most likely a counting bug (fixed in 395ee78). That is not proven,
   because the PC's scratch folders were not inspected.
2. **Two scorer fixes landed on the run they serve.**
   - `halfc` now refits a top-down fit that found no pairs (finding N). The validation run's B verdict is GO only
     because of this; without it B is PENDING.
   - The recording rate now comes from the top-down pass when the probe has none, so the rule now prints B-first
     for v2 as well.
   - Both have synthetic tests; the refit test fails on the old code.
3. **The hand-check share per team.** Jasper re-marked 5 ward rows: 3 blue, 2 red. halfc counts the 5/5 against
   each team, as the thresholds allow below 5 rows, so blue's GO rests on 3 blue rows.
4. **The mock's default changed.** `one_recording` is now off and frames go into a client subfolder; the probe test
   now expects the second recording to write frames.
5. **The route reasons were rewritten.** Jasper's ruling (A+B, B on key windows) stands. The R2 reason "one B
   window per sign-in" fell with G. The reasons that remain: B needs A's identities, and B is measured on one
   window only.
6. **Findings M and N were added** to the A–L table, in both the doc and the template. They are run facts, not
   spike fixes; M has no code fix.

## Per-signal comparison (same R3 scorer)

| Signal | v2 (2026-10-26, before K) | Validation run (2026-10-28, after K+F) | Verdict (validation run) |
|---|---|---|---|
| A: median, n | 172.3 (n=246) | **112.5** (n=219) | |
| A: detection raw / alive-adjusted | 0.765 / 0.778 (false locks counted as seen) | 0.672 / **0.686** | |
| A: longest gap, gaps | 116 s, 599 | **120 s**, 760 | **PARTIAL** (median meets GO; detection and gap do not) |
| A: Pyke / Jax / Zed / Viego median | 8287 / 5538 / 3609 / 108.6 (false locks) | **146 / 163 / 75 / 112.5**, no `false_lock` | |
| A: static spots rejected | n/a (old tracker) | 7 (Jax 2, Pyke 2, Zed 3); none for the other seven | |
| B: LOFO median, in-sample | 127.8, 88.0 (32 pairs) | **124.7**, 113.6 (40 pairs), after the halfc refit | **GO** |
| B: bar hit rate | 0.824 | **0.827** | |
| Wards blue: P / R | 0.889 / 0.889 | 0.889 / 0.889 (identical) | **GO** (hand check 5/5; 3/3 on blue rows) |
| Wards red: P / R | 0.80 / 0.571 | 0.80 / 0.571 (identical) | **NO-GO** (recall) |
| Recalls: P / R | 0.857 (6/7) / 0.086 (6/70) | 0.875 (7/8) / 0.100 (7/70) | **NO-GO** (hand check 2/5) |
| Hand check | 2/9 (v1 verdicts carried over) | **7/10**, re-marked: wards 5/5, recalls 2/5 | |
| HDR | off (no record) | `hdr.enabled false`, read on real Windows | |
| Route | rule: B-first | rule: **B-first** (A PARTIAL, B GO, ≈9× real time) | **Ruled by Jasper 2026-11-08: A+B, B on key windows** |

**Did K fix route A?** It fixed identity and accuracy on a full 1 Hz run: there are no false locks, and the median
fell from 172 to 112.5, inside the GO bound. It did not fix coverage: detection is 0.686 (GO needs 0.85) and the
longest gap is 120 s (GO needs ≤15 s). So A reaches PARTIAL, not GO. The drop in detection from 0.778 is honest:
v2 counted the false locks as seen. Against v2 with its static samples dropped (0.585), the validation run is
higher.

**Which code Jasper ran:** the run shows K and F were included. `track_minimap.json` carries `static_spots` (F,
committed after K), `icon_sources` (J), `hdr` and the monitor record. `run.log` shows L removing 79 samples left
by an earlier pass. 99eba90 (parallel detection, Viego's possessed icon) came later and was not in the run.

**The ward results repeat:** two independent recordings of the same window gave the identical 14 new ward events.

## Finding G: the evidence

Every recording in the run logs:

- **2026-10-22, 20:57:** the probe's two recordings report 0 of 40 frames. The client read the request back and
  reported a path inside its own `<patch>_<game>_01` subfolder; the probe counted `*.png` only at the top of the
  given path.
- **2026-10-22, 21:11 and 21:14:** two top-down recordings with "0 png frames", counted the same way. The subfolder
  fix 944b0dc came at 21:18 that evening; the probe kept the old count until 395ee78.
- **2026-10-26, 19:39 and 20:31:** two top-down recordings of 601 frames each. The log does not show whether
  Windows was signed out between them.
- **2026-10-28, 09:35:** one recording of 601 frames in 33 s.

The corrected G: `endTime` is still ignored; "one recording per sign-in" is withdrawn on Jasper's report; the
2026-10-22 zeros were most likely the counting bug, cause unproven. With it, the time-cost paragraph now gives
about 30 s per 5-minute top-down window with no sign-out, and both checklists and the `session.py` docstring drop
the sign-out step.

## What changed in the findings (`docs/scout-findings.md`, d54fad5)

- **Status banner:** final, validated by the 2026-10-28 run, route A+B with B on key windows ruled by Jasper on
  2026-11-08.
- **Findings table:** G corrected; I and J confirmed on the PC (HDR read as off; all ten icons from CommunityDragon
  circle icons); K confirmed. Two new findings:
  - **M:** the first minimap pass used the automatic 341 px box against a 679 px minimap.
  - **N:** the top-down fit ran against that first pass and found no pairs.
- **(a) Facts:** fact 3 rewritten (about 9× real time, frames in a client subfolder, several recordings per
  sign-in); the HDR row carries the real-Windows record.
- **(b) Go/no-go:** v2 vs validation-run table, with the reason for each verdict.
- **(c) Route:** as ruled, with the rule's B-first output and the reasons kept for the record.
- **(e) Licence note:** the CommunityDragon URL answered on the PC.
- **(f) Privacy:** v1 and the quarantined PNGs are deleted, per Jasper's ruling.
- **(g) What M156b covers:** updated (see the last section).

## Commits this round

- ae034b3 fix(scout): halfc refits a failed top-down fit and reads the recording rate from the run.
  Tests: `test_refresh_topdown_refits_an_r2_track_whose_fit_found_no_pairs` (fails on the old code) and
  `test_recording_rate_falls_back_to_the_topdown_recording`.
- 395ee78 fix(scout): probe counts recorded frames in the client's subfolder.
  Test: `test_probe_counts_frames_in_the_client_subfolder`.
- d54fad5 docs(scout): validation run scored, finding G corrected, route as ruled (M156 R3).

## pytest

```
tools/scout/.venv/bin/python -m pytest tools/scout -q
161 passed in 84.52s (0:01:24)
```

## What M156b / the product round inherits

- **Route A's coverage:** detection 0.686 and gaps up to 120 s, measured on one game only. Get more games before
  relaxing any threshold.
- **The minimap box (M):** detect it on the first frame, or remember it per PC.
- **Red wards' recall** (0.571), and a ward stamp that lags placement by a few seconds.
- **Recall coverage at the fountain.**
- **B on more windows:** cheap now. The limits are disk on the PC (601 full-screen PNGs per window) and the fit,
  which needs a timeline frame each minute and A's identities. The top-down pass sizes the camera from its
  requested 1920×1080, not the recorded screen: harmless at 16:9, wrong on other aspects.
- **Still open from R2:** the YOLOv12 weights and their licence, nexus anchors, the unused live static-spot count,
  the HDR warning never seen firing on a real PC, a larger fact-6 ward crop, and `/liveclientdata/playerlist` in
  replay mode.
- **L-1** stays a prerequisite for any upload of tracks into Team Companion.
- **The real match ID is in branch history** from 944b0dc: squash when landing.
