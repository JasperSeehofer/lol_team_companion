# CI e2e summary

- run: 39631112808
- sha: c04ed5729e0807747121dd3763f4dce60f0db4d0

## Playwright

expected 190, unexpected 1, flaky 0, skipped 0

### [unexpected] match-thread.spec.ts:134 match thread: the draft opens prefilled from the thread link
attempt 1: failed (31463 ms)
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
attempt 2: failed (33409 ms)
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
