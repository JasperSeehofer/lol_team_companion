# CI e2e summary

- run: 39182895010
- sha: 464744c3e7e5434f2a110d77652404f82a21b9e6

## Playwright

expected 173, unexpected 1, flaky 14, skipped 0

### [flaky] account.spec.ts:40 the player-card switch persists
attempt 1: failed (20595 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] bug-report.spec.ts:44 bug-report widget > select mode shows the hint, captures a labelled element
attempt 1: failed (20561 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] community-gates.spec.ts:40 signed-in users see the member community links
attempt 1: failed (20588 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] cross-tenant.spec.ts:40 a second team cannot open the first team's draft or game plan by ID
attempt 1: failed (20525 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] enter-submits.spec.ts:10 enter submits: tree drafter adds a branch on Enter
attempt 1: failed (20581 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] feedback-attachment.spec.ts:25 a file attached to feedback reaches the admin as WebP, and only the admin
attempt 1: failed (20517 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] frames-m148.spec.ts:154 M148 frames > draft saved and failed (draft, draft-save-failed)
attempt 1: failed (20601 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] guidance.spec.ts:86 a hint shows once on the draft board and not after Got it
attempt 1: failed (20564 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hero-matchup.spec.ts:144 shared game plan: five champions per team never wrap at 320, 390 and 1280
attempt 1: failed (12746 ms)
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

### [flaky] hydration-no-panic.spec.ts:95 / hydrates without tachys panic
attempt 1: failed (20589 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /team/dashboard hydrates without tachys panic
attempt 1: failed (20570 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /tree-drafter hydrates without tachys panic
attempt 1: failed (20583 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] login-next.spec.ts:55 Return to the page after sign-in > next pointing back at an auth page lands on Home
attempt 1: failed (20670 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] save-failure.spec.ts:183 lists that did not load say so and load again with Try again
attempt 1: failed (538 ms)
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

### [unexpected] team-invites.spec.ts:177 a coach invites players; a plain member sees no invite panel
attempt 1: failed (8462 ms)
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
attempt 2: failed (29196 ms)
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
