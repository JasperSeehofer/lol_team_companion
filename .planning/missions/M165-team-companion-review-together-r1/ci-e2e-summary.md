# CI e2e summary

- run: 43483309031
- sha: 9e31f829eb72353b1282f72dc06dafd97917900e

## Playwright

expected 191, unexpected 1, flaky 0, skipped 0

### [unexpected] review-together.spec.ts:318 review together: sealed, anonymous, decided and counted by five members
attempt 1: failed (20785 ms)
```
Error: expect(locator).toContainText(expected) failed

Locator: locator('[data-prep] a[href^="/review/together?"]')
Expected substring: "Review together"
Received string:    "After the game: review"
Timeout: 5000ms

Call log:
  - Expect "toContainText" with timeout 5000ms
  - waiting for locator('[data-prep] a[href^="/review/together?"]')
    9 × locator resolved to <a class="hover:underline underline-offset-4" href="/review/together?match_id=scheduled_match:0tqi5q4nec3d1pobf171&game=1">…</a>
      - unexpected value "After the game: review"
```
attempt 2: failed (22499 ms)
```
Error: expect(locator).toContainText(expected) failed

Locator: locator('[data-prep] a[href^="/review/together?"]')
Expected substring: "Review together"
Received string:    "After the game: review"
Timeout: 5000ms

Call log:
  - Expect "toContainText" with timeout 5000ms
  - waiting for locator('[data-prep] a[href^="/review/together?"]')
    9 × locator resolved to <a class="hover:underline underline-offset-4" href="/review/together?match_id=scheduled_match:6xw3cpkmmb8qfovax8a2&game=1">…</a>
      - unexpected value "After the game: review"
```

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
