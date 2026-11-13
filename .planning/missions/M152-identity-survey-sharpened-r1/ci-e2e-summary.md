# CI e2e summary

- run: 39075603464
- sha: a83d96e7c13c81bf7d159a7fdc00c8d397dfff9d

## Playwright

expected 183, unexpected 0, flaky 3, skipped 0

### [flaky] account.spec.ts:55 deleting the account needs the username and password, then signs out
attempt 1: failed (20574 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hero-matchup.spec.ts:144 shared game plan: five champions per team never wrap at 320, 390 and 1280
attempt 1: failed (12888 ms)
```
Error: expect(locator).toHaveValue(expected) failed

Locator: locator('#plan-name')
Expected pattern: /Game 1 plan/
Received string:  ""
Timeout: 10000ms

Call log:
  - Expect "toHaveValue" with timeout 10000ms
  - waiting for locator('#plan-name')
    13 × locator resolved to <input type="text" required="" id="plan-name" maxlength="120" placeholder="Front-to-back vs Riverside" class="w-full rounded-ctl border border-outline bg-ink-2 px-3 text-primary placeholder:text-muted transition-colors hover:border-secondary data-[force=hover]:border-secondary disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-danger h-9 text-body"/>
       - unexpected value ""
```

### [flaky] login-next.spec.ts:55 Return to the page after sign-in > next pointing back at an auth page lands on Home
attempt 1: failed (20704 ms)
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
