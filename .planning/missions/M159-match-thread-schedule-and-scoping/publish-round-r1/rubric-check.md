# UX rubric check (M159 publish round r1)

## Summary

Source: CI's e2e job (run 39631700231, sha c69ee0c), step "UX rubric check
(public touched surfaces, report only)" in .github/workflows/ci.yml, using
scripts/ux-rubric/rubric-check.mjs (identical to the garden copy). The
harness drives live pages in Chromium; it cannot run on PNGs, so it was not
re-run locally (no server on this host). Verdicts are identical to M152's
table (no regression, no change).

Measured: four public pages (/auth/login, /auth/login?next=/draft,
/auth/register?invite=..., /auth/reset), phone and desktop each.

**Not measured: every M159 surface.** Match Day (upcoming, Bo3 prep),
/draft from a thread, /post-game picker and /match/:id review are signed-in
pages and the harness has no sign-in step. Their after frames exist under
frames/after/ for review by eye (atelier/human).

| id | result (4 pages x 2 viewports) | note |
|---|---|---|
| A1 target size | PASS 8/8 | |
| A2 contrast from pixels | PASS 8/8 | |
| A3 reduced motion | PASS 8/8 | |
| A4 single-pointer + screen reader | not measured | run_by: human |
| I1 manual path to Settings | FIX 8/8 | heuristic; signed-out pages have no Settings by design |
| I2 progress past 10 s | FIX 8/8 | GUESS; no agent actions on these pages |
| I3 disclosure depth | not measured | run_by: atelier |
| I4 correction path | not measured | run_by: human |
| I5 confirms stay rare | not measured | run_by: atelier |
| I6 interruption budget | FIX 8/8 | GUESS; no notifier policy object in this app |
| I7 primary content readable | PASS 8/8 | |
| I8 undo/consent for agent action | FIX 8/8 | GUESS; no agent-initiated actions on these pages |
| V1 closed token vocabulary | PASS 8/8 | |
| V2 4/8 spacing scale | FIX 8/8 | off-scale values on [data-feedback-...] (feedback widget), pre-existing |
| V3 measure 45-75 CPL | FIX 8/8 | short hints and footer paragraphs outside the range, pre-existing |

The FIX rows are the same as in M152 and come from surfaces M159 did not
change; the I-row FIXes are agent-product heuristics that do not fit a
signed-out form.

Design lint (scripts/lint_design.sh, 2026-10-21, this branch): exit 0,
"Design lint: OK (every rule at zero)".

## CI output (verbatim)

- run: 39631700231
- sha: c69ee0c54b4e88064f175813cd5fe996ba62021e
- harness: scripts/ux-rubric/rubric-check.mjs (v24.21.0)
- limitation: only public pages; signed-in surfaces need a sign-in step the harness does not have

| id | page | viewport | verdict | detail |
|----|------|----------|---------|--------|
| I1 | http://127.0.0.1:3020/auth/login | phone | FIX | No Settings reachable path found |
| I2 | http://127.0.0.1:3020/auth/login | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | http://127.0.0.1:3020/auth/login | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | http://127.0.0.1:3020/auth/login | phone | PASS |  |
| I8 | http://127.0.0.1:3020/auth/login | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | http://127.0.0.1:3020/auth/login | phone | PASS |  |
| V2 | http://127.0.0.1:3020/auth/login | phone | FIX | Off-scale spacing values found: [data-feedback-... |
| V3 | http://127.0.0.1:3020/auth/login | phone | FIX | CPL violations: p: 87 chars; p: 279 chars; p: 1... |
| V4 | http://127.0.0.1:3020/auth/login | phone | PASS |  |
| Vo1 | http://127.0.0.1:3020/auth/login | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | http://127.0.0.1:3020/auth/login | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | http://127.0.0.1:3020/auth/login | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | http://127.0.0.1:3020/auth/login | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | http://127.0.0.1:3020/auth/login | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | http://127.0.0.1:3020/auth/login | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | http://127.0.0.1:3020/auth/login | phone | PASS |  |
| A2 | http://127.0.0.1:3020/auth/login | phone | PASS |  |
| A3 | http://127.0.0.1:3020/auth/login | phone | PASS |  |
| I1 | http://127.0.0.1:3020/auth/login | desktop | FIX | No Settings reachable path found |
| I2 | http://127.0.0.1:3020/auth/login | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | http://127.0.0.1:3020/auth/login | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | http://127.0.0.1:3020/auth/login | desktop | PASS |  |
| I8 | http://127.0.0.1:3020/auth/login | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | http://127.0.0.1:3020/auth/login | desktop | PASS |  |
| V2 | http://127.0.0.1:3020/auth/login | desktop | FIX | Off-scale spacing values found: [data-feedback-... |
| V3 | http://127.0.0.1:3020/auth/login | desktop | FIX | CPL violations: p: 87 chars; p: 279 chars; p: 1... |
| V4 | http://127.0.0.1:3020/auth/login | desktop | PASS |  |
| Vo1 | http://127.0.0.1:3020/auth/login | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | http://127.0.0.1:3020/auth/login | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | http://127.0.0.1:3020/auth/login | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | http://127.0.0.1:3020/auth/login | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | http://127.0.0.1:3020/auth/login | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | http://127.0.0.1:3020/auth/login | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | http://127.0.0.1:3020/auth/login | desktop | PASS |  |
| A2 | http://127.0.0.1:3020/auth/login | desktop | PASS |  |
| A3 | http://127.0.0.1:3020/auth/login | desktop | PASS |  |
| I1 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | FIX | No Settings reachable path found |
| I2 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | PASS |  |
| I8 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | PASS |  |
| V2 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | FIX | Off-scale spacing values found: [data-feedback-... |
| V3 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | FIX | CPL violations: p: 87 chars; p: 279 chars; p: 1... |
| V4 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | PASS |  |
| Vo1 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | PASS |  |
| A2 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | PASS |  |
| A3 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | PASS |  |
| I1 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | FIX | No Settings reachable path found |
| I2 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | PASS |  |
| I8 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | PASS |  |
| V2 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | FIX | Off-scale spacing values found: [data-feedback-... |
| V3 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | FIX | CPL violations: p: 87 chars; p: 279 chars; p: 1... |
| V4 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | PASS |  |
| Vo1 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | PASS |  |
| A2 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | PASS |  |
| A3 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | PASS |  |
| I1 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | FIX | No Settings reachable path found |
| I2 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | PASS |  |
| I8 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | PASS |  |
| V2 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | FIX | Off-scale spacing values found: [data-feedback-... |
| V3 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | FIX | CPL violations: p: 24 chars; #register-password... |
| V4 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | PASS |  |
| Vo1 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | PASS |  |
| A2 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | PASS |  |
| A3 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | phone | PASS |  |
| I1 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | FIX | No Settings reachable path found |
| I2 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | PASS |  |
| I8 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | PASS |  |
| V2 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | FIX | Off-scale spacing values found: [data-feedback-... |
| V3 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | FIX | CPL violations: p: 24 chars; #register-password... |
| V4 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | PASS |  |
| Vo1 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | PASS |  |
| A2 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | PASS |  |
| A3 | http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE | desktop | PASS |  |
| I1 | http://127.0.0.1:3020/auth/reset | phone | FIX | No Settings reachable path found |
| I2 | http://127.0.0.1:3020/auth/reset | phone | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | http://127.0.0.1:3020/auth/reset | phone | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | http://127.0.0.1:3020/auth/reset | phone | PASS |  |
| I8 | http://127.0.0.1:3020/auth/reset | phone | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | http://127.0.0.1:3020/auth/reset | phone | PASS |  |
| V2 | http://127.0.0.1:3020/auth/reset | phone | FIX | Off-scale spacing values found: [data-feedback-... |
| V3 | http://127.0.0.1:3020/auth/reset | phone | FIX | CPL violations: #reset-code-hint: 32 chars; #re... |
| V4 | http://127.0.0.1:3020/auth/reset | phone | PASS |  |
| Vo1 | http://127.0.0.1:3020/auth/reset | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | http://127.0.0.1:3020/auth/reset | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | http://127.0.0.1:3020/auth/reset | phone | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | http://127.0.0.1:3020/auth/reset | phone | PASS | GUESS: voice checks not implemented in harness |
| T1 | http://127.0.0.1:3020/auth/reset | phone | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | http://127.0.0.1:3020/auth/reset | phone | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | http://127.0.0.1:3020/auth/reset | phone | PASS |  |
| A2 | http://127.0.0.1:3020/auth/reset | phone | PASS |  |
| A3 | http://127.0.0.1:3020/auth/reset | phone | PASS |  |
| I1 | http://127.0.0.1:3020/auth/reset | desktop | FIX | No Settings reachable path found |
| I2 | http://127.0.0.1:3020/auth/reset | desktop | FIX | No progress indicators found (GUESS: cannot tes... |
| I6 | http://127.0.0.1:3020/auth/reset | desktop | FIX | No notification policy found (GUESS: heuristic ... |
| I7 | http://127.0.0.1:3020/auth/reset | desktop | PASS |  |
| I8 | http://127.0.0.1:3020/auth/reset | desktop | FIX | No undo affordance found (GUESS: heuristic check) |
| V1 | http://127.0.0.1:3020/auth/reset | desktop | PASS |  |
| V2 | http://127.0.0.1:3020/auth/reset | desktop | FIX | Off-scale spacing values found: [data-feedback-... |
| V3 | http://127.0.0.1:3020/auth/reset | desktop | FIX | CPL violations: #reset-code-hint: 32 chars; #re... |
| V4 | http://127.0.0.1:3020/auth/reset | desktop | PASS |  |
| Vo1 | http://127.0.0.1:3020/auth/reset | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo2 | http://127.0.0.1:3020/auth/reset | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo3 | http://127.0.0.1:3020/auth/reset | desktop | PASS | GUESS: voice checks not implemented in harness |
| Vo4 | http://127.0.0.1:3020/auth/reset | desktop | PASS | GUESS: voice checks not implemented in harness |
| T1 | http://127.0.0.1:3020/auth/reset | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| T4 | http://127.0.0.1:3020/auth/reset | desktop | PASS | GUESS: trust checks heuristic - need manual review |
| A1 | http://127.0.0.1:3020/auth/reset | desktop | PASS |  |
| A2 | http://127.0.0.1:3020/auth/reset | desktop | PASS |  |
| A3 | http://127.0.0.1:3020/auth/reset | desktop | PASS |  |

exit code: 1
