# CI e2e summary

- run: 39061358433
- sha: 51be80fb08f18e7ab85ed8af7f999d6012257943

## Playwright

expected 158, unexpected 0, flaky 19, skipped 0

### [flaky] auth.spec.ts:20 Registration > can register a new account
attempt 1: failed (1057 ms)
```
Error: expect(received).not.toContain(expected) // indexOf

Expected substring: not "/auth/register"
Received string:        "http://127.0.0.1:3020/auth/register?invite=E2E-INVITE-CODE"
```

### [flaky] champion-pool-moves.spec.ts:91 champion-pool moves: a failed save reverts and offers Retry
attempt 1: failed (20540 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] community-gates.spec.ts:40 signed-in users see the member community links
attempt 1: failed (20559 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] frames-m148.spec.ts:154 M148 frames > draft saved and failed (draft, draft-save-failed)
attempt 1: failed (20576 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] frames-m148.spec.ts:242 M148 frames > profile password section (profile-password)
attempt 1: failed (20569 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] guidance.spec.ts:86 a hint shows once on the draft board and not after Got it
attempt 1: failed (20557 ms)
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

### [flaky] hydration-no-panic.spec.ts:95 /team/dashboard hydrates without tachys panic
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

### [flaky] hydration-no-panic.spec.ts:95 /tree-drafter hydrates without tachys panic
attempt 1: failed (20596 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /champion-pool hydrates without tachys panic
attempt 1: failed (20586 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /bugs hydrates without tachys panic
attempt 1: failed (20573 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /transparency hydrates without tachys panic
attempt 1: failed (20558 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] login-next.spec.ts:55 Return to the page after sign-in > next pointing back at an auth page lands on Home
attempt 1: failed (20637 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] match-detail.spec.ts:7 match detail page > renders match detail page without errors
attempt 1: failed (20569 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] save-failure.spec.ts:53 draft: a failed autosave says so, Retry saves once the server answers
attempt 1: failed (20580 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] save-failure.spec.ts:85 game plan: a failed autosave says so, Retry saves once the server answers
attempt 1: failed (20573 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] save-failure.spec.ts:167 leaving a game plan whose save failed shows the leave dialog
attempt 1: failed (20581 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] team-invites.spec.ts:70 a signed-in player joins with the link; one already in a team cannot
attempt 1: failed (10027 ms)
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

### [flaky] team-invites.spec.ts:177 a coach invites players; a plain member sees no invite panel
attempt 1: failed (28431 ms)
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
