# CI e2e summary

- run: 38998423121
- sha: 3c83f7636a0332603665e53477fd67099f7c70c7

## Playwright

expected 161, unexpected 0, flaky 1, skipped 0

### [flaky] account.spec.ts:55 deleting the account needs the username and password, then signs out
attempt 1: failed (20535 ms)
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
