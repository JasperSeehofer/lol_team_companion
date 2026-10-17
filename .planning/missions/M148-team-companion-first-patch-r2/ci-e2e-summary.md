# CI e2e summary

- run: 39008897600
- sha: e6648843110827d4e0d2eb634b1a00747d8c0a25

## Playwright

expected 165, unexpected 0, flaky 1, skipped 0

### [flaky] login-next.spec.ts:55 Return to the page after sign-in > next pointing back at an auth page lands on Home
attempt 1: failed (20630 ms)
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
