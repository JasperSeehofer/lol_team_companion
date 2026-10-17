# M156: licence note (to fold into `docs/scout-findings.md`, half C)

Checked 2026-10-17. Each fact carries its source. Nothing here chooses a licence for the app.

## Facts

| Component | Licence | Source | Note |
|---|---|---|---|
| **Scout** (`tools/scout/`) | AGPL-3.0 (`tools/scout/LICENSE`) | Mission page M156, Scope item 1 | A separate tool that ships beside the app. It is not linked into the Rust crate and shares no code with it. |
| **pyLoL**, vendored at SHA `19afccfc6590e074567798598833d3a737724ba4` | **GPL-3.0**, per its `LICENSE` file | https://github.com/league-of-legends-replay-extractor/pyLoL/blob/19afccfc6590e074567798598833d3a737724ba4/LICENSE | Its `setup.py` says `MIT` (https://github.com/league-of-legends-replay-extractor/pyLoL/blob/19afccfc6590e074567798598833d3a737724ba4/setup.py). That is inconsistent; the `LICENSE` file governs. Only reference code is copied, with its LICENSE and `ATTRIBUTION.md`. The mission page's bound facts call pyLoL "AGPL-3.0"; the file says GPL-3.0. |
| GPL-3.0 code combined with AGPL-3.0 | Allowed | GPLv3 §13: https://www.gnu.org/licenses/gpl-3.0.html#section13 ; AGPLv3 §13: https://www.gnu.org/licenses/agpl-3.0.html#section13 | Each licence explicitly permits combining with the other. The GPL parts keep their GPL terms, and the combined work carries AGPL §13's network clause. |
| **ultralytics** (detector runtime) | **AGPL-3.0** | https://github.com/ultralytics/ultralytics/blob/main/LICENSE ; PyPI metadata "AGPL-3.0": https://pypi.org/project/ultralytics/ | A paid Enterprise Licence is offered as the alternative: https://www.ultralytics.com/license. The mission page's "AAGPL-3.0" was a typo. |
| **YOLOv12** | AGPL-3.0 | https://github.com/sunsmarterjie/yolov12 (LICENSE) | |
| pyLoL's **`yolov12.pt` weights** | **No stated licence** | Linked from pyLoL's README as a Google Drive download; no licence alongside | Treat them as AGPL-derived (trained with an AGPL toolchain on an AGPL architecture). Their status is unclear, and that is **a risk for M157**. |
| opencv-python | Apache-2.0 | https://github.com/opencv/opencv-python/blob/4.x/LICENSE.txt | |
| numpy | BSD-3-Clause | https://github.com/numpy/numpy/blob/main/LICENSE.txt | |
| scipy | BSD-3-Clause | https://github.com/scipy/scipy/blob/main/LICENSE.txt | |
| mss | MIT | https://github.com/BoboTiG/python-mss/blob/main/LICENSE.txt | |
| League Director | Apache-2.0 | https://github.com/RiotGames/leaguedirector/blob/master/LICENSE | Used only as a field reference for Replay API names. No code is copied. |
| **The app** (`lol_team_companion`) | **Undecided** | There is no `LICENSE` file at the repo root, and `Cargo.toml` has no `license` field (checked 2026-10-17) | Not chosen here (an M156 non-goal). |

## Implications for M157 (the product helper)

- **Bundling.** If M157 bundles an ultralytics-derived model or runtime, it must be AGPL-3.0 itself, or the owner buys the Ultralytics Enterprise Licence.
- **ONNX export does not settle it.** Exporting to ONNX changes the format, not the licence question for the weights. The weights stay AGPL-derived and of unclear licence until a clean source is found, or until weights are trained with a permissively licensed architecture and toolchain.
- **Network use.** AGPL §13 matters if a detector ever runs server-side in the app: users who interact with it over the network must be offered its source. As long as detection runs only on the player's PC and the app receives plain tracks (data, not code), the app's own licence stays independent.
- **pyLoL's GPL-3.0** sets no extra constraint beyond the AGPL constraint, provided the helper is AGPL. A permissive or closed helper could not include pyLoL code.

## Privacy: derived tracks (draft paragraph)

The tracks Scout derives from a player's own replay contain all ten champions' positions, including the opponents'. Those opponents are identifiable players. The legal-privacy audit's finding F01 treats scouted third parties as personal data (Art. 6(1)(f), 14, 17, 21 GDPR; garden `analyses/team-companion-audit-legal-privacy-2026-10.md`, §2, F01, with the Art. 14 notice outline in §4b item 9). Derived tracks therefore follow the same rules as scouting:

- they are visible only to the uploading player's team;
- they are minimised to per-second positions, ward placements and recalls, with no video, frames or chat;
- the team can delete them, and account or team deletion removes them;
- they are kept no longer than the match cache, 12 months after upload, through the same nightly retention as `MATCH_CACHE_RETENTION_DAYS`.

The audit's planned objection route and purge across all teams (F01 (a), (b)) must cover the tracks too.
