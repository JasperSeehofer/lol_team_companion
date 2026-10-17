# CI e2e summary

- run: 39070475669
- sha: 2df32c387363762e9ad4b45ec0bc15ba8655c03f

## Playwright

expected 168, unexpected 1, flaky 17, skipped 0

### [flaky] account.spec.ts:40 the player-card switch persists
attempt 1: failed (20602 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] account.spec.ts:55 deleting the account needs the username and password, then signs out
attempt 1: failed (20585 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] bug-report.spec.ts:17 bug-report widget > docked on signed-in pages, absent on public ones
attempt 1: failed (20602 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] champion-pool-moves.spec.ts:32 champion-pool moves: keyboard move keeps the scroll and shows the save state
attempt 1: failed (20650 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] community-gates.spec.ts:25 admin pages are 404 for a signed-in non-admin
attempt 1: failed (20592 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] cross-tenant.spec.ts:40 a second team cannot open the first team's draft or game plan by ID
attempt 1: failed (20565 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] frames-m148.spec.ts:127 M148 frames > team page invite links after a reload (dashboard-invites)
attempt 1: failed (20585 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] gallery.spec.ts:449 gallery access > the gallery is a 404 for visitors and non-admins outside dev
attempt 1: failed (20562 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] guidance.spec.ts:102 the profile switch hides every piece of guidance
attempt 1: failed (20587 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hero-matchup.spec.ts:124 shared game plan: five champions per team never wrap at 320, 390 and 1280
attempt 1: failed (11915 ms)
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

### [unexpected] hero-matchup.spec.ts:188 Match Day at 390: the field card's button is the page's chalk (at most 2 accent items)
attempt 1: failed (2153 ms)
```
Error: page.goto: Navigation to "http://127.0.0.1:3020/" is interrupted by another navigation to "http://127.0.0.1:3020/"
Call log:
  - navigating to "http://127.0.0.1:3020/", waiting until "load"
```
attempt 2: failed (1725 ms)
```
Error: page.goto: Navigation to "http://127.0.0.1:3020/" is interrupted by another navigation to "http://127.0.0.1:3020/"
Call log:
  - navigating to "http://127.0.0.1:3020/", waiting until "load"
```

### [flaky] hydration-no-panic.spec.ts:95 /draft hydrates without tachys panic
attempt 1: failed (20612 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /champion-pool hydrates without tachys panic
attempt 1: failed (534 ms)
```
Error: page.check: Clicking the checkbox did not change its state
Call log:
  - waiting for locator('#register-terms')
    - locator resolved to <input required="" type="checkbox" id="register-terms" name="accept_terms" class="peer size-4 cursor-pointer appearance-none rounded-ctl border border-outline bg-ink-2 transition-colors checked:border-accent checked:bg-accent hover:border-secondary aria-invalid:border-danger disabled:cursor-not-allowed"/>
  - attempting click action
    - waiting for element to be visible, enabled and stable
    - element is visible, enabled and stable
    - scrolling into view if needed
    - done scrolling
    - performing click action
    - click action done
    - waiting for scheduled navigations to finish
    - navigations have finished
```

### [flaky] hydration-no-panic.spec.ts:95 /what-next hydrates without tachys panic
attempt 1: failed (20590 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /changelog hydrates without tachys panic
attempt 1: failed (20582 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:139 segmented control observable on /draft
attempt 1: failed (20582 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:161 draft slot click-fill
attempt 1: failed (20568 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] login-next.spec.ts:55 Return to the page after sign-in > next pointing back at an auth page lands on Home
attempt 1: failed (20635 ms)
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
