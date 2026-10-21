# CI e2e summary

- run: 39631700231
- sha: c69ee0c54b4e88064f175813cd5fe996ba62021e

## Playwright

expected 184, unexpected 1, flaky 6, skipped 0

### [flaky] bug-report.spec.ts:126 bug-report widget > widget DOM has no forbidden characters
attempt 1: failed (20659 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] frames-m148.spec.ts:127 M148 frames > team page invite links after a reload (dashboard-invites)
attempt 1: failed (20554 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hero-matchup.spec.ts:144 shared game plan: five champions per team never wrap at 320, 390 and 1280
attempt 1: failed (12936 ms)
```
Error: expect(locator).toHaveValue(expected) failed

Locator: locator('#plan-name')
Expected pattern: /Game 1 plan/
Received string:  ""
Timeout: 10000ms

Call log:
  - Expect "toHaveValue" with timeout 10000ms
  - waiting for locator('#plan-name')
    14 × locator resolved to <input type="text" required="" id="plan-name" maxlength="120" placeholder="Front-to-back vs Riverside" class="w-full rounded-ctl border border-outline bg-ink-2 px-3 text-primary placeholder:text-muted transition-colors hover:border-secondary data-[force=hover]:border-secondary disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-danger h-9 text-body"/>
       - unexpected value ""
```

### [flaky] hydration-no-panic.spec.ts:95 /team/dashboard hydrates without tachys panic
attempt 1: failed (20559 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] login-next.spec.ts:55 Return to the page after sign-in > next pointing back at an auth page lands on Home
attempt 1: failed (20721 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [unexpected] match-thread.spec.ts:134 match thread: the draft opens prefilled from the thread link
attempt 1: failed (30502 ms)
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
attempt 2: failed (32343 ms)
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

### [flaky] team-invites.spec.ts:177 a coach invites players; a plain member sees no invite panel
attempt 1: failed (28276 ms)
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
