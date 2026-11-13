# CI e2e summary

- run: 43492196074
- sha: c26e5c78bea93850805da45c1bf192238272b91f

## Playwright

expected 191, unexpected 1, flaky 0, skipped 0

### [unexpected] review-together.spec.ts:318 review together: sealed, anonymous, decided and counted by five members
attempt 1: failed (30626 ms)
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
attempt 2: failed (32373 ms)
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
