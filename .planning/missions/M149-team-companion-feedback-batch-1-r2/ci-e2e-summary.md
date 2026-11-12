# CI e2e summary

- run: 43299035974
- sha: 39a1d5e6c3303ec5753b60b49d04c4468f686072

## Playwright

expected 202, unexpected 1, flaky 0, skipped 0

### [unexpected] feedback-attachment.spec.ts:195 a refused screen capture reopens the dialog with the message
attempt 1: failed (9249 ms)
```
Error: expect(locator).toContainText(expected) failed

Locator: getByRole('dialog', { name: 'Give Feedback' })
Expected substring: "The browser did not allow screen capture. You can still attach an image or paste one."
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toContainText" with timeout 5000ms
  - waiting for getByRole('dialog', { name: 'Give Feedback' })
```
attempt 2: failed (10053 ms)
```
Error: expect(locator).toBeVisible() failed

Locator: getByRole('dialog', { name: 'Give Feedback' })
Expected: visible
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for getByRole('dialog', { name: 'Give Feedback' })
```

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
