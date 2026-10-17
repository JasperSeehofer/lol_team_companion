# M152 — Identity R1: Survey, sharpened — REPORT R1

Branch `mission/M152-identity-survey-sharpened-r1` (pushed, branch only). It was forked from main `0c57f9a` (M148 landed),
merged with `origin/mission/M151-identity-survey-sharpened-r0` (merge `0c0add8`), and then merged with `origin/main` (already up
to date at the last merge). Nothing was merged to main, deployed or touched on the VM. Card:
`publish-round-r1/entry.json` (id `m152-r1-team-companion-identity`).

## Budget

- **12.2 M metered** at the overseer's stop call, against 8.5 M declared (1.44×). The cap was raised to 14.0 M. The
  overseer's meter read 7.8 M after 80 minutes; from then on the round ran **one subagent at a time** and kept to the STOP
  RULE: no new scope item started, so **items 7 and 8 were not started**. Wrap-up began on the overseer's call at 12.2 M.
- Subagent tokens reported by the harness add up to about 2.1 M (the Rift builder alone used about 0.41 M). The rest is
  orchestration context and cache reads.
- Parallel fan-out in the first hour (four agents at once: recipes, icons + legal, Rift, OG) is what pushed the early
  meter. That is the lesson for the next taste round on this repo.

## Least-confident decisions (read these first)

1. **How the Rift geometry was sourced** (`docs/rift-schematic.md`, `scripts/rift-geometry/rift-source.json`). The
   3708 px `map11.png` turned out to be a stylised art map: no river, and each base drawn as an emblem. It was used only
   for calibration and the two nexus positions. The walls and river were measured on the **Data Dragon 16.20.1 minimap**
   (512 px, which is on Riot's approved asset list), likewise by eye on gridded crops, private and never committed.
   - 20 towers and 6 inhibitors are labelled `reported`: Match-V5's fixed positions, entered from known values, and
     not measured, because neither reference shows them.
   - The 24 bushes are labelled `approximate`, because no reference shows brush.
   - 3 points come from fixtures. One turret-plate event says team 100 but sits on red's position; the position was used.
   - Nothing is derived from pixels. `test_build.py` asserts that no image library is imported.
2. **Sketch level has no wall hatch** (atelier ruling 4, `critique-r1-surfaces.md`). The measured walls are 4–7 units
   wide, so card A's hatch left 13 bar fragments that read as "text lines". At sketch level the Rift is now frame, lanes,
   river, bases, Baron/Dragon, bush dots and one chalk label. Hatch exists at survey level only, which departs from the
   mission page's wording ("impassable terrain filled with strokes"); it holds at survey level.
3. **Match Day facing names: built per Scope 4, then removed** after the atelier REJECTed them on the CI render. In a best
   of 3 they assert sides that do not exist, they were misaligned with the bases, they wrapped at 390, and they were a third
   print of the title's names. The title "Scrim vs X" carries the facing. This is a react question on the card.
4. **Share strip side** comes from the plan's linked draft (`SharedPlan.our_side`, read through the team-scoped
   `get_draft_for_prefill` with the team from the link row). With a side, the strip is a base crop with a chalk "Ours"
   label and leader. Without one, it is the bridge crop with no label. Unknown, expired and revoked links get no label,
   and the `<head>` stays static.
5. **OG surface is chosen by path in `App`** (`src/app.rs`), not per page, so it can never vary inside a Suspense. As a
   result, `/` gives the landing card to signed-in users as well. The origin comes from the `Host` header; anything that
   is not a plain host name falls back to `https://app.team-companion.eu`. There is no new env variable.
6. **Icon pin `16.20`, not `latest`.** `latest` already served different bytes for five PNGs. The sanitiser keeps
   `fill-rule` and `opacity`, which the two-tone icons need.
7. **Fallback copies moved to `assets/ui/`** (not `public/ui/` as the page says). `public/` is the static root, which would
   have served them at `/ui/...` without the sandbox CSP. A unit test and an e2e test keep `/ui/...` a 404.
8. **"Sponsor" in the money-words test is exempted too broadly (open, R1b).** The s.6 notice contains "sponsor", so
   `ui_strings_use_the_approved_money_words` (`src/pages/community/mod.rs:108-113`) skips any literal that the notice
   *contains*, a substring check. A bare UI literal "sponsor" in one of the listed files would therefore slip past the
   check. The fix (exact match plus a test) was in progress when the overseer stopped the round. It was discarded
   uncommitted, so the tested tip stays as it is. Product behaviour is unaffected: this only weakens a guard test.

## What shipped per item

| # | Item | State | Evidence |
|---|---|---|---|
| 1 | Icon proxy and real icons | shipped | `src/server/ui_icons.rs` (allow-list of 21, upstream URL only from constants, `include_bytes!` fallbacks, 5 s timeout and 15 min retry, ETag/304, `max-age=604800`, nosniff, and a sandbox CSP on SVG); route `src/main.rs:435`; `assets/ui/`; `scripts/ui-icons/sanitise.py`; RoleLabel mask icon `src/components/ui/markers.rs`; `e2e/tests/ui-icons.spec.ts`; the CommunityDragon allowance was removed from `e2e/tests/helpers.ts` and `smoke.spec.ts`; the draft-graph third-party checks are unchanged |
| 2 | The drawn Rift | shipped | `scripts/rift-geometry/{rift-source.json,build.py,test_build.py}`, `assets/rift/map11-geometry.json`, generated `src/components/ui/rift_geometry.rs`, `src/components/ui/rift.rs` (`RiftLevel {Sketch, Survey}`, crops, label/anchor), `/design` "Drawn Rift" section with keyboard zoom (`src/pages/design_rift.rs`, `gallery.spec.ts`), `docs/rift-schematic.md` (127-row source table, calibration, elevation finding: no contours) |
| 3 | Hero on landing, closed beta, share | shipped | `src/pages/closed_beta.rs` (shared `Landing`, used by `home.rs`), `src/pages/share.rs`, `src/models/share.rs`; match-up rows in an `@container`, Md → Sm, nowrap; `e2e/tests/hero-matchup.spec.ts` (320/390/1280 one row per team, landing chalk ≤ 2, one primary) |
| 4 | Match Day field card | shipped; inset size R1b | `src/pages/home.rs` `FieldCard`, `TimeStyle::{Clock,Day}` in `src/components/local_time.rs`, `PrepEmphasis::Quiet` without accent in `src/components/ui/prep.rs`; e2e asserts ≤ 2 accent items at 390 |
| 5 | Open Graph | shipped | `src/components/og.rs` (`OgSurface`, fixed strings, no page data), `src/app.rs` (theme-color `#081415`, favicon, `PageOgMeta`), `public/og/{landing,share,default}.png` (1200×630, 13–16 KB, rendered by `scripts/og/` from the app's own geometry), `public/favicon.svg`; `e2e/tests/og-meta.spec.ts` (share head block byte-identical for valid/unknown/revoked; og:image same host, 200, `image/png`) |
| 6 | Disclaimers and provenance | shipped | `RIOT_LJJ_NOTICE` in `src/components/footer.rs:11`, rendered in the footer (`data-testid="riot-notice"`), `src/pages/legal/terms.rs:101`, `datenschutz.rs`; unit test and `e2e/tests/legal.spec.ts`; `docs/ASSETS.md` |
| 7 | Lock-in stroke and static stamp | **not started** (stop rule) | |
| 8 | Paper theme on `/s/` | **not started** (stop rule) | Go/no-go is a react question |

The s.6 sentence was checked against https://www.riotgames.com/en/legal §6. It is verbatim, with the product name filled
in: "Team Companion was created under Riot Games' "Legal Jibber Jabber" policy using assets owned by Riot Games. Riot Games
does not endorse or sponsor this project." The page has a non-breaking space between the sentences; here it is a normal space.

R0's nine original marks (`public/marks/`, `/design` marks section) came in with the M151 merge. They stay **gallery
only**; the real icons replace them on product surfaces.

## Gates

- **Local** (each builder, last on the tip before the report): `cargo fmt --check` clean; `cargo check` and
  `cargo clippy -D warnings` on ssr and hydrate clean; `cargo test --features ssr --lib` 449 passed, 0 failed;
  `./scripts/lint_design.sh` "Design lint: OK (every rule at zero)"; `build.py --check` "rift geometry: up to date";
  generator tests 5 OK.
- **CI:**
  - `51be80f` (run 39061358433): all five jobs green.
  - `3717f89` (run 39067107644): all five green, 182 expected, 2 flaky.
  - `2df32c3`: e2e failed on a navigation race in the new Match Day spec, with 17 flaky; fixed in `18695d1`. No code
    cause was found for the flakes: all of them time out in the shared registration helper.
  - `18695d1` (run 39073124625): check, style, scripts and e2e green (181 expected, 0 unexpected, 5 flaky). The image job
    failed with a GitHub cache error, `buildx failed: error writing layer blob: not_found`, which is infrastructure, not code.
  - The final tip's run is recorded in the status line at the end of this report.
- **Rubric** (`rubric-check.mjs`): not runnable on this host. It needs a live page and playwright-core, so it is CI-only
  and has no table this round. Contrast was recomputed from pixels by the atelier: chalk label 14.5:1, ink headline
  16.2:1, OG subtitle 12.3:1.

## Atelier verdicts (`critique-r1-rift.md`, `critique-r1-surfaces.md` iteration 2)

| Surface | Verdict |
|---|---|
| Landing 1280, 390, reading-flow | PASS |
| Closed beta | PASS |
| Share 390 / 1280 | PASS |
| Match Day 390 / 1280 | FIX, R1b (the ~75 px inset reads as an X in a box) |
| RoleLabel (roster 390) | PASS |
| Rift survey level (`/design`) | PASS |
| OG landing, share, default | PASS |

Strips: `publish-round-r1/report.html`. Main has no frames of the landing, closed-beta or share pages, so those are
paired with R0's T1 target stills. Match Day is paired with main's M148 frames.

## Measuring log summary

The details are in `docs/rift-schematic.md`. The calibration comes from the two nexus positions on map11 and the
reported structure pairs, which agree within 60 units under point symmetry. Every wall vertex was typed by hand from
gridded crops of the Data Dragon minimap. The river was refit as two cubics, 2.2 units off the measured centreline at
worst (R1-a's curve was 3.8 off). Source counts: 46 walls, 1 river, 3 lanes, 2 pits, 14 camps, 2 nexuses, 6 inhibitors,
22 towers, 24 bushes.

## DESIGN.md amendments (proposals only, not applied)

- **§7 (`docs/DESIGN.md:500`), replace the sentence "Role icons are typographic (`RoleLabel`); the browser loads no image
  from a third party (the gallery spec checks it).":**
  > Role icons are the app's own `/img/ui/position/{name}` (CommunityDragon 16.20, sanitised, committed fallback; see
  > docs/ASSETS.md), drawn as a CSS mask in the label's ink beside the mono letters (`RoleLabel`), `aria-hidden`, never in
  > chalk; the browser loads no image from a third party (the gallery and ui-icons specs check it).
  CLAUDE.md lines 321 ("no role icons") and 337 ("role marks are typographic") need the same change.
- **Chalk count (`docs/DESIGN.md:202`, accent rule):**
  > - **Accent:** chalk is for focus, the selected tab, the step being decided ("selecting"), the playhead and the one
  >   next action per view. On the signed-out landing and the share hero a second chalk element, the surveyed team label,
  >   is allowed, because it is the same promise ("this sheet is for you"). Never more than two, and never a mark in chalk.
  >   Not for decoration, headings or icons.
  Principle 3 (`:13`) gains "and the surveyed team label on landing and share".
- **New, from the owner's R0 answer:** the lead-figure grammar (one figure, one sentence, one action) on landing, share
  and Match Day.
- **Rift (new §6 entry):** sketch level has no hatch; hatch, bushes and outlines belong to survey level.

## R1b (left for the next round)

1. Match Day inset: hide it below 120 px, or use the bridge crop (atelier).
2. The Match Day card leaves the right side empty at 1280 (capped at 640 px).
3. Items 7 (lock-in stroke and static stamp) and 8 (paper theme on `/s/`; not on the card, ask the owner when R1b is planned).
4. Answers to the card's rulings: Rift fidelity, the chalk-count amendment, Match Day facing.
5. Apply the DESIGN.md and CLAUDE.md amendments once ruled.
6. Tighten the money-words exemption to an exact match (least-confident item 8).
7. The image job on the final tip, if GitHub's cache error repeats (see Gates).

## Questions for the owner (on the card)

The merge question, plus three react rulings: Rift fidelity, the chalk-count amendment, and the removal of the Match Day
facing names. The Riot portal ask was dropped: the owner filed the ticket on 2026-10-17 (overseer's note).
