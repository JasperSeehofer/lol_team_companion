# CI e2e summary

- run: 44915551610
- sha: b2106ba03838ac87985eacaff85da77880ea9d59

## Playwright

expected 203, unexpected 1, flaky 0, skipped 0

### [unexpected] guidance.spec.ts:161 seeded returning user > the returning-user line shows; a New mark appears and clears
attempt 1: failed (12772 ms)
```
Error: the seeded returning user sees the New mark

expect(received).toBe(expected) // Object.is equality

Expected: true
Received: false
```
attempt 2: failed (13008 ms)
```
Error: the seeded returning user sees the New mark

expect(received).toBe(expected) // Object.is equality

Expected: true
Received: false
```

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
