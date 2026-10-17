# CI e2e summary

- run: 39060799348
- sha: 177e05f2015f177d97425824f90f08bba58733a3

## Playwright

expected 174, unexpected 0, flaky 2, skipped 0

### [flaky] enter-submits.spec.ts:44 enter submits: team note textarea keeps Enter as a newline, Ctrl+Enter adds
attempt 1: failed (20540 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] login-next.spec.ts:46 Return to the page after sign-in > a forged next to another site lands on Home
attempt 1: failed (20584 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
