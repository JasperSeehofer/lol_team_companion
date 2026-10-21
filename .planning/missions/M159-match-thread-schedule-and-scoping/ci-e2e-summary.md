# CI e2e summary

- run: 39628884536
- sha: 92e28bdb1296e450da949d0dd4ae325ac2b90f6d

## Playwright

expected 188, unexpected 1, flaky 0, skipped 0

### [unexpected] match-thread.spec.ts:134 match thread: the draft opens prefilled from the thread link
attempt 1: failed (31723 ms)
```
Error: expect(locator).toBeVisible() failed

Locator: getByTestId('draft-thread-unknown')
Expected: visible
Timeout: 15000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 15000ms
  - waiting for getByTestId('draft-thread-unknown')
```
attempt 2: failed (33625 ms)
```
Error: expect(locator).toBeVisible() failed

Locator: getByTestId('draft-thread-unknown')
Expected: visible
Timeout: 15000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 15000ms
  - waiting for getByTestId('draft-thread-unknown')
```

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
