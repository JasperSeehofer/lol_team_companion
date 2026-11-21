# UX rubric check: Review together R1 (atelier run, 2026-11-13)

- frames: CI run 43511191122, sha 951190a (`frames/after/*.html`, post-hydration snapshots, scripts stripped, `<base href="http://127.0.0.1:3020/">`)
- harness: `/home/ops/Repositories/garden/wiki/assets/ux-rubric/rubric-check.mjs` (generated 2026-11-13T22:08Z), node v25.9.0, Chromium 1243
- replaces: the earlier CI version of this file (public pages only, `/auth/*`), which is in git at `0f02015`

## How it was run (state it, so the rows can be weighed)

1. No app server was running and `target/site` did not exist. The snapshots load `/pkg/lol_team_companion.css` through the `<base>`, so the atelier built that file from this tree's `input.css` with the cached Tailwind CLI (`~/.cache/cargo-leptos/tailwindcss-v4.2.1`, the version cargo-leptos uses) into `/tmp/m165-site/pkg/`, copied `public/` beside it, and served it with `python3 -m http.server 3020`. No app code was touched. The CSS is a rebuild from source, not the CI artefact.
2. The snapshots were copied under the same origin (`http://127.0.0.1:3020/snap/*.html`). From `file://`, Chromium marks the stylesheet cross-origin and V1/V2 come back "unreadable stylesheet (SecurityError)". That first run was thrown away.
3. `--app-dir` is `/home/ops/Repositories/lol_team_companion-wt/M137-team-companion-first-patch/e2e`. This worktree's `e2e/node_modules` is not installed, and the harness only resolves `playwright-core` from it.
4. 12 unique snapshots (the `-flow` copies are byte-identical to their base file and were skipped), each run at phone and desktop. Total: 432 rows.
5. Command: `rubric-check.mjs --app-dir <M137>/e2e --report-json /tmp/m165-rubric.json http://127.0.0.1:3020/snap/{match-day-review-together-390, review-together-writing-390, review-together-writing-1280, review-together-facilitator-1280, review-together-facilitator-1280-expanded, review-together-facilitator-sealed-1280, review-together-nudge-1280, review-together-reveal-390, review-together-reveal-1280, review-together-decide-390, review-together-decide-1280, review-together-review-1280}.html`. Exit code 1.

## Summary by row (24 page×viewport runs each)

| row | result | what it actually measured | atelier reading |
|---|---|---|---|
| A1 | PASS 24/24 | target size/spacing | green |
| A2 | PASS 24/24 | contrast | green. Pixel cross-check on decide-1280: note words 16.2:1, author caption 7.0:1, figures 10.8:1, Decide caption 7.0:1 against `bg-page` (8,20,21) |
| A3 | PASS 24/24 | reduced motion on a static file | proves nothing on a snapshot (brief §7). Judged instead on the live sequence in the critique: PASS |
| A4 | not emitted | human row | judged by sight in the critique: PASS |
| I1 | FIX 24/24 | the string "settings" in the DOM | **harness misfire.** The app calls it "Profile", and it is reachable from the account menu in every frame |
| I2 | FIX 24/24 | a `[role=progressbar]` / `.spinner` exists | **misfire / n/a.** No action on this surface runs >10 s, and no agent acts |
| I3–I5 | not emitted | atelier/human rows | I3 and I5 are judged in the critique (PASS). I4 is n/a (nothing is agent-authored) |
| I6 | FIX 24/24 | a notification policy object | **n/a.** The surface sends nothing (brief §8: no notification, badge, sound or title change) |
| I7 | PASS 24/24 | | green |
| I8 | FIX 24/24 | the string "undo" in the static DOM | **misfire.** The Undo controls live in toasts that only exist after an action. The surface has no agent-initiated action, and its two irreversible paths sit behind a `ConfirmDialog` |
| V1 | PASS 24/24 | literal colours outside tokens | green. **The V1 misfire predicted in the intake did not happen:** once the bundle was readable, the `@theme` block counted as token definitions |
| V2 | FIX 24/24 | every off-scale px in the **whole stylesheet**, used or not | **bundle-wide, not this surface.** The same 14 declarations appear on every page: Tailwind preflight `optgroup option` 20px, `.sr-only` −1px ×4, and the 1 px hairline utilities `-inset-x-px`, `-bottom-px`, `-my-px`, `mt-px`, `-mb-px`, `pb-px`, plus `top-[-3px]`. The only one present in these snapshots is `-mb-px` on the shared hub sub-nav. None comes from `review_together.rs` or `pages/review_together.rs` |
| V3 | FIX 24/24 | `textContent` length of the first 20 `<p>`, not rendered line length | **misfire.** It fails any sentence shorter than 45 characters (the notes run 31–44) and hidden dialog copy (`#bug-report-file-hint` 128, `#share-dialog-desc` 122). The rendered measure of a serif note is about 50 CPL at 390 (PASS at the rubric's 393 px), but about 93 CPL at 1280 inside the 42 rem column. That is a card note, not a build FIX (see the critique) |
| V4 | PASS | (dropped in the brief) | not counted |
| Vo1–Vo4, T1, T4 | PASS (GUESS) | not implemented | dropped in the brief |

**Literal reading of the round-0 criterion:** "A1–A4, I1–I8 and V1–V3 green under rubric-check.mjs" is **not literally met**. I1, I2, I6, I8, V2 and V3 are FIX on all 24 runs. Each FIX is a heuristic misfire, a bundle-wide finding, or n/a for this surface, as the table explains. None points at Review together code. The atelier recommends recording them as harness false positives. Ratifying that reading is for the human or the calling session, not for the atelier.

## Design lint

```
$ ./scripts/lint_design.sh
Design lint: OK (every rule at zero).
exit 0
```

## Raw harness table

<details><summary>432 rows (page URLs shortened to the snapshot name)</summary>

| id | page | viewport | verdict | detail |
|----|------|----------|---------|--------|
| I1 | match-day-review-together-390.html | phone | FIX | No Settings reachable path found |
| I2 | match-day-review-together-390.html | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | match-day-review-together-390.html | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | match-day-review-together-390.html | phone | PASS |  |
| I8 | match-day-review-together-390.html | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | match-day-review-together-390.html | phone | PASS | exempt: token definitions (201 declarations) |
| V2 | match-day-review-together-390.html | phone | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | match-day-review-together-390.html | phone | FIX | CPL violations: p: 39 chars; p: 102 chars; p: 3... |
| V4 | match-day-review-together-390.html | phone | PASS |  |
| Vo1 | match-day-review-together-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | match-day-review-together-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | match-day-review-together-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | match-day-review-together-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | match-day-review-together-390.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | match-day-review-together-390.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | match-day-review-together-390.html | phone | PASS |  |
| A2 | match-day-review-together-390.html | phone | PASS |  |
| A3 | match-day-review-together-390.html | phone | PASS |  |
| I1 | match-day-review-together-390.html | desktop | FIX | No Settings reachable path found |
| I2 | match-day-review-together-390.html | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | match-day-review-together-390.html | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | match-day-review-together-390.html | desktop | PASS |  |
| I8 | match-day-review-together-390.html | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | match-day-review-together-390.html | desktop | PASS | exempt: token definitions (201 declarations) |
| V2 | match-day-review-together-390.html | desktop | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | match-day-review-together-390.html | desktop | FIX | CPL violations: p: 39 chars; p: 102 chars; p: 3... |
| V4 | match-day-review-together-390.html | desktop | PASS |  |
| Vo1 | match-day-review-together-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | match-day-review-together-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | match-day-review-together-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | match-day-review-together-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | match-day-review-together-390.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | match-day-review-together-390.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | match-day-review-together-390.html | desktop | PASS |  |
| A2 | match-day-review-together-390.html | desktop | PASS |  |
| A3 | match-day-review-together-390.html | desktop | PASS |  |
| I1 | review-together-writing-390.html | phone | FIX | No Settings reachable path found |
| I2 | review-together-writing-390.html | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-writing-390.html | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-writing-390.html | phone | PASS |  |
| I8 | review-together-writing-390.html | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-writing-390.html | phone | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-writing-390.html | phone | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-writing-390.html | phone | FIX | CPL violations: p: 31 chars; #bug-report-file-h... |
| V4 | review-together-writing-390.html | phone | PASS |  |
| Vo1 | review-together-writing-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-writing-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-writing-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-writing-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-writing-390.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-writing-390.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-writing-390.html | phone | PASS |  |
| A2 | review-together-writing-390.html | phone | PASS |  |
| A3 | review-together-writing-390.html | phone | PASS |  |
| I1 | review-together-writing-390.html | desktop | FIX | No Settings reachable path found |
| I2 | review-together-writing-390.html | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-writing-390.html | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-writing-390.html | desktop | PASS |  |
| I8 | review-together-writing-390.html | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-writing-390.html | desktop | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-writing-390.html | desktop | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-writing-390.html | desktop | FIX | CPL violations: p: 31 chars; #bug-report-file-h... |
| V4 | review-together-writing-390.html | desktop | PASS |  |
| Vo1 | review-together-writing-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-writing-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-writing-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-writing-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-writing-390.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-writing-390.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-writing-390.html | desktop | PASS |  |
| A2 | review-together-writing-390.html | desktop | PASS |  |
| A3 | review-together-writing-390.html | desktop | PASS |  |
| I1 | review-together-writing-1280.html | phone | FIX | No Settings reachable path found |
| I2 | review-together-writing-1280.html | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-writing-1280.html | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-writing-1280.html | phone | PASS |  |
| I8 | review-together-writing-1280.html | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-writing-1280.html | phone | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-writing-1280.html | phone | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-writing-1280.html | phone | FIX | CPL violations: p: 31 chars; #bug-report-file-h... |
| V4 | review-together-writing-1280.html | phone | PASS |  |
| Vo1 | review-together-writing-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-writing-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-writing-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-writing-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-writing-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-writing-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-writing-1280.html | phone | PASS |  |
| A2 | review-together-writing-1280.html | phone | PASS |  |
| A3 | review-together-writing-1280.html | phone | PASS |  |
| I1 | review-together-writing-1280.html | desktop | FIX | No Settings reachable path found |
| I2 | review-together-writing-1280.html | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-writing-1280.html | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-writing-1280.html | desktop | PASS |  |
| I8 | review-together-writing-1280.html | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-writing-1280.html | desktop | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-writing-1280.html | desktop | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-writing-1280.html | desktop | FIX | CPL violations: p: 31 chars; #bug-report-file-h... |
| V4 | review-together-writing-1280.html | desktop | PASS |  |
| Vo1 | review-together-writing-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-writing-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-writing-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-writing-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-writing-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-writing-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-writing-1280.html | desktop | PASS |  |
| A2 | review-together-writing-1280.html | desktop | PASS |  |
| A3 | review-together-writing-1280.html | desktop | PASS |  |
| I1 | review-together-facilitator-1280.html | phone | FIX | No Settings reachable path found |
| I2 | review-together-facilitator-1280.html | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-facilitator-1280.html | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-facilitator-1280.html | phone | PASS |  |
| I8 | review-together-facilitator-1280.html | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-facilitator-1280.html | phone | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-facilitator-1280.html | phone | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-facilitator-1280.html | phone | FIX | CPL violations: p: 37 chars; #bug-report-file-h... |
| V4 | review-together-facilitator-1280.html | phone | PASS |  |
| Vo1 | review-together-facilitator-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-facilitator-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-facilitator-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-facilitator-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-facilitator-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-facilitator-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-facilitator-1280.html | phone | PASS |  |
| A2 | review-together-facilitator-1280.html | phone | PASS |  |
| A3 | review-together-facilitator-1280.html | phone | PASS |  |
| I1 | review-together-facilitator-1280.html | desktop | FIX | No Settings reachable path found |
| I2 | review-together-facilitator-1280.html | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-facilitator-1280.html | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-facilitator-1280.html | desktop | PASS |  |
| I8 | review-together-facilitator-1280.html | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-facilitator-1280.html | desktop | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-facilitator-1280.html | desktop | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-facilitator-1280.html | desktop | FIX | CPL violations: p: 37 chars; #bug-report-file-h... |
| V4 | review-together-facilitator-1280.html | desktop | PASS |  |
| Vo1 | review-together-facilitator-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-facilitator-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-facilitator-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-facilitator-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-facilitator-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-facilitator-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-facilitator-1280.html | desktop | PASS |  |
| A2 | review-together-facilitator-1280.html | desktop | PASS |  |
| A3 | review-together-facilitator-1280.html | desktop | PASS |  |
| I1 | review-together-facilitator-1280-expanded.html | phone | FIX | No Settings reachable path found |
| I2 | review-together-facilitator-1280-expanded.html | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-facilitator-1280-expanded.html | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-facilitator-1280-expanded.html | phone | PASS |  |
| I8 | review-together-facilitator-1280-expanded.html | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-facilitator-1280-expanded.html | phone | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-facilitator-1280-expanded.html | phone | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-facilitator-1280-expanded.html | phone | FIX | CPL violations: p: 37 chars; #bug-report-file-h... |
| V4 | review-together-facilitator-1280-expanded.html | phone | PASS |  |
| Vo1 | review-together-facilitator-1280-expanded.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-facilitator-1280-expanded.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-facilitator-1280-expanded.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-facilitator-1280-expanded.html | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-facilitator-1280-expanded.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-facilitator-1280-expanded.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-facilitator-1280-expanded.html | phone | PASS |  |
| A2 | review-together-facilitator-1280-expanded.html | phone | PASS |  |
| A3 | review-together-facilitator-1280-expanded.html | phone | PASS |  |
| I1 | review-together-facilitator-1280-expanded.html | desktop | FIX | No Settings reachable path found |
| I2 | review-together-facilitator-1280-expanded.html | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-facilitator-1280-expanded.html | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-facilitator-1280-expanded.html | desktop | PASS |  |
| I8 | review-together-facilitator-1280-expanded.html | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-facilitator-1280-expanded.html | desktop | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-facilitator-1280-expanded.html | desktop | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-facilitator-1280-expanded.html | desktop | FIX | CPL violations: p: 37 chars; #bug-report-file-h... |
| V4 | review-together-facilitator-1280-expanded.html | desktop | PASS |  |
| Vo1 | review-together-facilitator-1280-expanded.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-facilitator-1280-expanded.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-facilitator-1280-expanded.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-facilitator-1280-expanded.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-facilitator-1280-expanded.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-facilitator-1280-expanded.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-facilitator-1280-expanded.html | desktop | PASS |  |
| A2 | review-together-facilitator-1280-expanded.html | desktop | PASS |  |
| A3 | review-together-facilitator-1280-expanded.html | desktop | PASS |  |
| I1 | review-together-facilitator-sealed-1280.html | phone | FIX | No Settings reachable path found |
| I2 | review-together-facilitator-sealed-1280.html | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-facilitator-sealed-1280.html | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-facilitator-sealed-1280.html | phone | PASS |  |
| I8 | review-together-facilitator-sealed-1280.html | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-facilitator-sealed-1280.html | phone | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-facilitator-sealed-1280.html | phone | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-facilitator-sealed-1280.html | phone | FIX | CPL violations: p: 37 chars; #bug-report-file-h... |
| V4 | review-together-facilitator-sealed-1280.html | phone | PASS |  |
| Vo1 | review-together-facilitator-sealed-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-facilitator-sealed-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-facilitator-sealed-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-facilitator-sealed-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-facilitator-sealed-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-facilitator-sealed-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-facilitator-sealed-1280.html | phone | PASS |  |
| A2 | review-together-facilitator-sealed-1280.html | phone | PASS |  |
| A3 | review-together-facilitator-sealed-1280.html | phone | PASS |  |
| I1 | review-together-facilitator-sealed-1280.html | desktop | FIX | No Settings reachable path found |
| I2 | review-together-facilitator-sealed-1280.html | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-facilitator-sealed-1280.html | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-facilitator-sealed-1280.html | desktop | PASS |  |
| I8 | review-together-facilitator-sealed-1280.html | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-facilitator-sealed-1280.html | desktop | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-facilitator-sealed-1280.html | desktop | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-facilitator-sealed-1280.html | desktop | FIX | CPL violations: p: 37 chars; #bug-report-file-h... |
| V4 | review-together-facilitator-sealed-1280.html | desktop | PASS |  |
| Vo1 | review-together-facilitator-sealed-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-facilitator-sealed-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-facilitator-sealed-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-facilitator-sealed-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-facilitator-sealed-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-facilitator-sealed-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-facilitator-sealed-1280.html | desktop | PASS |  |
| A2 | review-together-facilitator-sealed-1280.html | desktop | PASS |  |
| A3 | review-together-facilitator-sealed-1280.html | desktop | PASS |  |
| I1 | review-together-nudge-1280.html | phone | FIX | No Settings reachable path found |
| I2 | review-together-nudge-1280.html | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-nudge-1280.html | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-nudge-1280.html | phone | PASS |  |
| I8 | review-together-nudge-1280.html | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-nudge-1280.html | phone | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-nudge-1280.html | phone | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-nudge-1280.html | phone | FIX | CPL violations: p: 37 chars; #bug-report-file-h... |
| V4 | review-together-nudge-1280.html | phone | PASS |  |
| Vo1 | review-together-nudge-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-nudge-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-nudge-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-nudge-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-nudge-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-nudge-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-nudge-1280.html | phone | PASS |  |
| A2 | review-together-nudge-1280.html | phone | PASS |  |
| A3 | review-together-nudge-1280.html | phone | PASS |  |
| I1 | review-together-nudge-1280.html | desktop | FIX | No Settings reachable path found |
| I2 | review-together-nudge-1280.html | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-nudge-1280.html | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-nudge-1280.html | desktop | PASS |  |
| I8 | review-together-nudge-1280.html | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-nudge-1280.html | desktop | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-nudge-1280.html | desktop | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-nudge-1280.html | desktop | FIX | CPL violations: p: 37 chars; #bug-report-file-h... |
| V4 | review-together-nudge-1280.html | desktop | PASS |  |
| Vo1 | review-together-nudge-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-nudge-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-nudge-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-nudge-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-nudge-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-nudge-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-nudge-1280.html | desktop | PASS |  |
| A2 | review-together-nudge-1280.html | desktop | PASS |  |
| A3 | review-together-nudge-1280.html | desktop | PASS |  |
| I1 | review-together-reveal-390.html | phone | FIX | No Settings reachable path found |
| I2 | review-together-reveal-390.html | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-reveal-390.html | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-reveal-390.html | phone | PASS |  |
| I8 | review-together-reveal-390.html | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-reveal-390.html | phone | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-reveal-390.html | phone | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-reveal-390.html | phone | FIX | CPL violations: p: 31 chars; p: 44 chars; p: 41... |
| V4 | review-together-reveal-390.html | phone | PASS |  |
| Vo1 | review-together-reveal-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-reveal-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-reveal-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-reveal-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-reveal-390.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-reveal-390.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-reveal-390.html | phone | PASS |  |
| A2 | review-together-reveal-390.html | phone | PASS |  |
| A3 | review-together-reveal-390.html | phone | PASS |  |
| I1 | review-together-reveal-390.html | desktop | FIX | No Settings reachable path found |
| I2 | review-together-reveal-390.html | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-reveal-390.html | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-reveal-390.html | desktop | PASS |  |
| I8 | review-together-reveal-390.html | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-reveal-390.html | desktop | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-reveal-390.html | desktop | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-reveal-390.html | desktop | FIX | CPL violations: p: 31 chars; p: 44 chars; p: 41... |
| V4 | review-together-reveal-390.html | desktop | PASS |  |
| Vo1 | review-together-reveal-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-reveal-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-reveal-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-reveal-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-reveal-390.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-reveal-390.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-reveal-390.html | desktop | PASS |  |
| A2 | review-together-reveal-390.html | desktop | PASS |  |
| A3 | review-together-reveal-390.html | desktop | PASS |  |
| I1 | review-together-reveal-1280.html | phone | FIX | No Settings reachable path found |
| I2 | review-together-reveal-1280.html | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-reveal-1280.html | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-reveal-1280.html | phone | PASS |  |
| I8 | review-together-reveal-1280.html | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-reveal-1280.html | phone | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-reveal-1280.html | phone | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-reveal-1280.html | phone | FIX | CPL violations: p: 37 chars; p: 44 chars; p: 41... |
| V4 | review-together-reveal-1280.html | phone | PASS |  |
| Vo1 | review-together-reveal-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-reveal-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-reveal-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-reveal-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-reveal-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-reveal-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-reveal-1280.html | phone | PASS |  |
| A2 | review-together-reveal-1280.html | phone | PASS |  |
| A3 | review-together-reveal-1280.html | phone | PASS |  |
| I1 | review-together-reveal-1280.html | desktop | FIX | No Settings reachable path found |
| I2 | review-together-reveal-1280.html | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-reveal-1280.html | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-reveal-1280.html | desktop | PASS |  |
| I8 | review-together-reveal-1280.html | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-reveal-1280.html | desktop | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-reveal-1280.html | desktop | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-reveal-1280.html | desktop | FIX | CPL violations: p: 37 chars; p: 44 chars; p: 41... |
| V4 | review-together-reveal-1280.html | desktop | PASS |  |
| Vo1 | review-together-reveal-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-reveal-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-reveal-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-reveal-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-reveal-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-reveal-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-reveal-1280.html | desktop | PASS |  |
| A2 | review-together-reveal-1280.html | desktop | PASS |  |
| A3 | review-together-reveal-1280.html | desktop | PASS |  |
| I1 | review-together-decide-390.html | phone | FIX | No Settings reachable path found |
| I2 | review-together-decide-390.html | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-decide-390.html | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-decide-390.html | phone | PASS |  |
| I8 | review-together-decide-390.html | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-decide-390.html | phone | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-decide-390.html | phone | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-decide-390.html | phone | FIX | CPL violations: p: 31 chars; p: 44 chars; p: 41... |
| V4 | review-together-decide-390.html | phone | PASS |  |
| Vo1 | review-together-decide-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-decide-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-decide-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-decide-390.html | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-decide-390.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-decide-390.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-decide-390.html | phone | PASS |  |
| A2 | review-together-decide-390.html | phone | PASS |  |
| A3 | review-together-decide-390.html | phone | PASS |  |
| I1 | review-together-decide-390.html | desktop | FIX | No Settings reachable path found |
| I2 | review-together-decide-390.html | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-decide-390.html | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-decide-390.html | desktop | PASS |  |
| I8 | review-together-decide-390.html | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-decide-390.html | desktop | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-decide-390.html | desktop | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-decide-390.html | desktop | FIX | CPL violations: p: 31 chars; p: 44 chars; p: 41... |
| V4 | review-together-decide-390.html | desktop | PASS |  |
| Vo1 | review-together-decide-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-decide-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-decide-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-decide-390.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-decide-390.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-decide-390.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-decide-390.html | desktop | PASS |  |
| A2 | review-together-decide-390.html | desktop | PASS |  |
| A3 | review-together-decide-390.html | desktop | PASS |  |
| I1 | review-together-decide-1280.html | phone | FIX | No Settings reachable path found |
| I2 | review-together-decide-1280.html | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-decide-1280.html | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-decide-1280.html | phone | PASS |  |
| I8 | review-together-decide-1280.html | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-decide-1280.html | phone | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-decide-1280.html | phone | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-decide-1280.html | phone | FIX | CPL violations: p: 37 chars; p: 44 chars; p: 41... |
| V4 | review-together-decide-1280.html | phone | PASS |  |
| Vo1 | review-together-decide-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-decide-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-decide-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-decide-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-decide-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-decide-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-decide-1280.html | phone | PASS |  |
| A2 | review-together-decide-1280.html | phone | PASS |  |
| A3 | review-together-decide-1280.html | phone | PASS |  |
| I1 | review-together-decide-1280.html | desktop | FIX | No Settings reachable path found |
| I2 | review-together-decide-1280.html | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-decide-1280.html | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-decide-1280.html | desktop | PASS |  |
| I8 | review-together-decide-1280.html | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-decide-1280.html | desktop | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-decide-1280.html | desktop | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-decide-1280.html | desktop | FIX | CPL violations: p: 37 chars; p: 44 chars; p: 41... |
| V4 | review-together-decide-1280.html | desktop | PASS |  |
| Vo1 | review-together-decide-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-decide-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-decide-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-decide-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-decide-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-decide-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-decide-1280.html | desktop | PASS |  |
| A2 | review-together-decide-1280.html | desktop | PASS |  |
| A3 | review-together-decide-1280.html | desktop | PASS |  |
| I1 | review-together-review-1280.html | phone | FIX | No Settings reachable path found |
| I2 | review-together-review-1280.html | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-review-1280.html | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-review-1280.html | phone | PASS |  |
| I8 | review-together-review-1280.html | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-review-1280.html | phone | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-review-1280.html | phone | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-review-1280.html | phone | FIX | CPL violations: #share-dialog-desc: 122 chars; ... |
| V4 | review-together-review-1280.html | phone | PASS |  |
| Vo1 | review-together-review-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-review-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-review-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-review-1280.html | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-review-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-review-1280.html | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-review-1280.html | phone | PASS |  |
| A2 | review-together-review-1280.html | phone | PASS |  |
| A3 | review-together-review-1280.html | phone | PASS |  |
| I1 | review-together-review-1280.html | desktop | FIX | No Settings reachable path found |
| I2 | review-together-review-1280.html | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | review-together-review-1280.html | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | review-together-review-1280.html | desktop | PASS |  |
| I8 | review-together-review-1280.html | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | review-together-review-1280.html | desktop | PASS | exempt: token definitions (201 declarations) |
| V2 | review-together-review-1280.html | desktop | FIX | 9 rules, 14 declarations off the 4/8 scale: :wh... |
| V3 | review-together-review-1280.html | desktop | FIX | CPL violations: #share-dialog-desc: 122 chars; ... |
| V4 | review-together-review-1280.html | desktop | PASS |  |
| Vo1 | review-together-review-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | review-together-review-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | review-together-review-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | review-together-review-1280.html | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | review-together-review-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | review-together-review-1280.html | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | review-together-review-1280.html | desktop | PASS |  |
| A2 | review-together-review-1280.html | desktop | PASS |  |
| A3 | review-together-review-1280.html | desktop | PASS |  |

</details>
