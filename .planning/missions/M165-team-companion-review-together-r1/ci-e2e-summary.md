# CI e2e summary

- run: 43499305253
- sha: 67f214e693b2049e9e9357c35754cbe4568d8ccc

## Playwright

expected 191, unexpected 1, flaky 0, skipped 0

### [unexpected] review-together.spec.ts:318 review together: sealed, anonymous, decided and counted by five members
attempt 1: failed (48788 ms)
```
Error: expect(locator).toBeVisible() failed

Locator: locator('#confirm-reveal')
Expected: visible
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for locator('#confirm-reveal')
```
attempt 2: failed (51592 ms)
```
Error: expect(locator).toBeVisible() failed

Locator: locator('#confirm-reveal')
Expected: visible
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for locator('#confirm-reveal')
```

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
