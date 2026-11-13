# CI e2e summary

- run: 43505578382
- sha: cc8ff8a1d884c1753bbb6794464bff8227268ee7

## Playwright

expected 191, unexpected 1, flaky 0, skipped 0

### [unexpected] review-together.spec.ts:318 review together: sealed, anonymous, decided and counted by five members
attempt 1: failed (48246 ms)
```
Error: expect(received).toBeLessThan(expected)

Expected: < 0.95
Received:   0.965983
```
attempt 2: failed (52982 ms)
```
Error: expect(received).toBeLessThan(expected)

Expected: < 0.95
Received:   0.965983
```

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
