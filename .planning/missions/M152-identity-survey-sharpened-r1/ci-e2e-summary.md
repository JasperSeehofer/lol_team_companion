# CI e2e summary

- run: 39067107644
- sha: 3717f8993072cb5759d59a236cb233c0e57ac69d

## Playwright

expected 182, unexpected 0, flaky 2, skipped 0

### [flaky] hydration-no-panic.spec.ts:95 /post-game hydrates without tachys panic
attempt 1: failed (20427 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /bugs hydrates without tachys panic
attempt 1: failed (20465 ms)
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
