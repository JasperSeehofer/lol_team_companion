# CI e2e summary

- run: 43492365236
- sha: 9015bfce37355b4d0d7ad4bacf43b6b472d65332

## Playwright

expected 191, unexpected 1, flaky 0, skipped 0

### [unexpected] review-together.spec.ts:318 review together: sealed, anonymous, decided and counted by five members
attempt 1: failed (28134 ms)
```
Error: expect(locator).toHaveCount(expected) failed

Locator:  locator('[data-prep] a[href^="/review/together?"]').filter({ hasText: 'Review together' })
Expected: 1
Received: 0
Timeout:  15000ms

Call log:
  - Expect "toHaveCount" with timeout 15000ms
  - waiting for locator('[data-prep] a[href^="/review/together?"]').filter({ hasText: 'Review together' })
    19 × locator resolved to 0 elements
       - unexpected value "0"
```
attempt 2: failed (28825 ms)
```
Error: expect(locator).toHaveCount(expected) failed

Locator:  locator('[data-prep] a[href^="/review/together?"]').filter({ hasText: 'Review together' })
Expected: 1
Received: 0
Timeout:  15000ms

Call log:
  - Expect "toHaveCount" with timeout 15000ms
  - waiting for locator('[data-prep] a[href^="/review/together?"]').filter({ hasText: 'Review together' })
    19 × locator resolved to 0 elements
       - unexpected value "0"
```

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
