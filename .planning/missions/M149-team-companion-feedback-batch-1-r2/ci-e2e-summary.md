# CI e2e summary

- run: 43309188443
- sha: 6f93871daf6392df83ed46324edd5189c69187db

## Playwright

expected 202, unexpected 0, flaky 1, skipped 0

### [flaky] account.spec.ts:15 download my data returns the account as a JSON file
attempt 1: failed (21274 ms)
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
