# CI e2e summary

- run: 39002450956
- sha: c3b38b8bd29f7ec8afb310c918a41d4fb53e1502

## Playwright

expected 145, unexpected 0, flaky 21, skipped 0

### [flaky] admin-invites.spec.ts:13 non-admin users get a 404 and no admin link
attempt 1: failed (20570 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] bug-report.spec.ts:44 bug-report widget > select mode shows the hint, captures a labelled element
attempt 1: failed (20572 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] bug-triage.spec.ts:96 another user sees only their own reports
attempt 1: failed (24400 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] champion-pool-moves.spec.ts:65 champion-pool moves: drag and drop keeps the scroll
attempt 1: failed (20570 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] champion-pool-moves.spec.ts:91 champion-pool moves: a failed save reverts and offers Retry
attempt 1: failed (20541 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] community-gates.spec.ts:25 admin pages are 404 for a signed-in non-admin
attempt 1: failed (529 ms)
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

### [flaky] cross-tenant.spec.ts:297 team invites: another team's leader cannot list or disable them, and the team is never taken from the client
attempt 1: failed (20536 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] feedback-attachment.spec.ts:80 posting bytes to a report that is not yours is refused
attempt 1: failed (20536 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] frames-m148.spec.ts:45 M148 frames > match day on a phone, no match
attempt 1: failed (20572 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] frames-m148.spec.ts:137 M148 frames > draft saved and failed (draft, draft-save-failed)
attempt 1: failed (541 ms)
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

### [flaky] guidance.spec.ts:56 the checklist ticks after creating a team and a draft, and hides on request
attempt 1: failed (20570 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 / hydrates without tachys panic
attempt 1: failed (20572 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /bugs hydrates without tachys panic
attempt 1: failed (20554 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /what-next hydrates without tachys panic
attempt 1: failed (20577 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /changelog hydrates without tachys panic
attempt 1: failed (20565 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] login-next.spec.ts:55 Return to the page after sign-in > next pointing back at an auth page lands on Home
attempt 1: failed (20648 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] match-day-phone.spec.ts:20 match day at 390 px: add the next match from a phone
attempt 1: failed (20583 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] password.spec.ts:207 /admin/resets is a 404 for a signed-in non-admin
attempt 1: failed (20551 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] save-failure.spec.ts:85 game plan: a failed autosave says so, Retry saves once the server answers
attempt 1: failed (20543 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] save-failure.spec.ts:183 lists that did not load say so and load again with Try again
attempt 1: failed (20585 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] team-invites.spec.ts:177 a coach invites players; a plain member sees no invite panel
attempt 1: failed (20519 ms)
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
