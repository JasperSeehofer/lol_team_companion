# UX rubric check

- run: 39007209640
- sha: ba0c92b6590c138d703635e172c4236a775d2b6c
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
| V3 | http://127.0.0.1:3020/auth/login | phone | FIX | CPL violations: p: 87 chars; p: 279 chars |
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
| V3 | http://127.0.0.1:3020/auth/login | desktop | FIX | CPL violations: p: 87 chars; p: 279 chars |
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
| V3 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | phone | FIX | CPL violations: p: 87 chars; p: 279 chars |
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
| V3 | http://127.0.0.1:3020/auth/login?next=%2Fdraft | desktop | FIX | CPL violations: p: 87 chars; p: 279 chars |
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
