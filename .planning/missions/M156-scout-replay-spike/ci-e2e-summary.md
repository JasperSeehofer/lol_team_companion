# CI e2e summary

- run: 40413092067
- sha: baccfd066c4168e697e00d535ef33d6e59736244

## Playwright

expected 149, unexpected 0, flaky 17, skipped 0

### [flaky] account.spec.ts:55 deleting the account needs the username and password, then signs out
attempt 1: failed (20628 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] admin-invites.spec.ts:13 non-admin users get a 404 and no admin link
attempt 1: failed (20536 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] auth.spec.ts:128 16+ and terms on sign-up (M148 item 7) > a direct post without accept_terms leaves a one-use invite unused
attempt 1: failed (23390 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] bug-report.spec.ts:17 bug-report widget > docked on signed-in pages, absent on public ones
attempt 1: failed (20555 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] bug-report.spec.ts:57 bug-report widget > Skip opens the dialog for the whole page; Escape cancels select mode
attempt 1: failed (20557 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] bug-report.spec.ts:97 bug-report widget > Give Feedback offers Bug, Idea and Praise; praise is sent (M141, report y1abg605)
attempt 1: failed (551 ms)
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

### [flaky] bug-report.spec.ts:117 bug-report widget > an empty report is refused in the dialog
attempt 1: failed (20584 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] champion-pool-moves.spec.ts:32 champion-pool moves: keyboard move keeps the scroll and shows the save state
attempt 1: failed (20631 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] cross-tenant.spec.ts:297 team invites: another team's leader cannot list or disable them, and the team is never taken from the client
attempt 1: failed (29273 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] feedback-attachment.spec.ts:67 a file that is not an image is refused in the dialog
attempt 1: failed (20522 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] gallery.spec.ts:408 gallery access > the gallery is a 404 for visitors and non-admins outside dev
attempt 1: failed (20553 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /team/dashboard hydrates without tachys panic
attempt 1: failed (20542 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /match/test-match-id hydrates without tachys panic
attempt 1: failed (20575 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:139 segmented control observable on /draft
attempt 1: failed (20547 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] login-next.spec.ts:46 Return to the page after sign-in > a forged next to another site lands on Home
attempt 1: failed (20586 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] login-next.spec.ts:78 Tab titles > signed-in pages each name themselves
attempt 1: failed (20551 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] team-invites.spec.ts:70 a signed-in player joins with the link; one already in a team cannot
attempt 1: failed (25589 ms)
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
