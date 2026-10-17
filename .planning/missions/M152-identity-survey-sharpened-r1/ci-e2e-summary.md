# CI e2e summary

- run: 39077594160
- sha: 085924575cadab07a5a2c4fe8b9bd1727bc80ea4

## Playwright

expected 171, unexpected 1, flaky 14, skipped 0

### [flaky] bug-report.spec.ts:57 bug-report widget > Skip opens the dialog for the whole page; Escape cancels select mode
attempt 1: failed (20549 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] community-gates.spec.ts:25 admin pages are 404 for a signed-in non-admin
attempt 1: failed (20541 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] cross-tenant.spec.ts:297 team invites: another team's leader cannot list or disable them, and the team is never taken from the client
attempt 1: failed (4591 ms)
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

### [flaky] enter-submits.spec.ts:44 enter submits: team note textarea keeps Enter as a newline, Ctrl+Enter adds
attempt 1: failed (20564 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] frames-m148.spec.ts:62 M148 frames > match day on a phone, no match
attempt 1: failed (20556 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] frames-m152.spec.ts:112 M152 frames > share page (seeded game plan) and Match Day field card
attempt 1: failed (19143 ms)
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

### [flaky] gallery.spec.ts:449 gallery access > the gallery is a 404 for visitors and non-admins outside dev
attempt 1: failed (20539 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] guidance.spec.ts:102 the profile switch hides every piece of guidance
attempt 1: failed (20547 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /team/dashboard hydrates without tachys panic
attempt 1: failed (20547 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /champion-pool hydrates without tachys panic
attempt 1: failed (20582 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [unexpected] hydration-no-panic.spec.ts:95 /bugs hydrates without tachys panic
attempt 1: failed (20548 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```
attempt 2: failed (21042 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /transparency hydrates without tachys panic
attempt 1: failed (20561 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] login-next.spec.ts:55 Return to the page after sign-in > next pointing back at an auth page lands on Home
attempt 1: failed (20652 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] og-meta.spec.ts:75 share links: identical card for a valid, an unknown and a revoked token
attempt 1: failed (20571 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] save-failure.spec.ts:167 leaving a game plan whose save failed shows the leave dialog
attempt 1: failed (20565 ms)
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
