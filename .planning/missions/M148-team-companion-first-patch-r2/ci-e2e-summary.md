# CI e2e summary

- run: 39004927668
- sha: d9dcff5687031527f6c6eea1e8513b674eb5562e

## Playwright

expected 154, unexpected 1, flaky 11, skipped 0

### [flaky] auth.spec.ts:128 16+ and terms on sign-up (M148 item 7) > a direct post without accept_terms leaves a one-use invite unused
attempt 1: failed (3342 ms)
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

### [flaky] bug-report.spec.ts:76 bug-report widget > sending shows the toast and the report appears under Your reports
attempt 1: failed (20562 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] champion-pool-moves.spec.ts:32 champion-pool moves: keyboard move keeps the scroll and shows the save state
attempt 1: failed (20662 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] frames-m148.spec.ts:62 M148 frames > match day on a phone, no match
attempt 1: failed (20554 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] frames-m148.spec.ts:167 M148 frames > game plan saved and failed (game-plan, game-plan-save-failed)
attempt 1: failed (20620 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [unexpected] frames-m148.spec.ts:242 M148 frames > profile password section (profile-password)
attempt 1: failed (3535 ms)
```
Error: profile-password at 390 is 438px wide

expect(received).toBeLessThanOrEqual(expected)

Expected: <= 390
Received:    438
```
attempt 2: failed (4057 ms)
```
Error: profile-password at 390 is 438px wide

expect(received).toBeLessThanOrEqual(expected)

Expected: <= 390
Received:    438
```

### [flaky] gallery.spec.ts:408 gallery access > the gallery is a 404 for visitors and non-admins outside dev
attempt 1: failed (20585 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] guidance.spec.ts:56 the checklist ticks after creating a team and a draft, and hides on request
attempt 1: failed (20598 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /tree-drafter hydrates without tachys panic
attempt 1: failed (20586 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /champion-pool hydrates without tachys panic
attempt 1: failed (568 ms)
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

### [flaky] hydration-no-panic.spec.ts:161 draft slot click-fill
attempt 1: failed (20590 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] login-next.spec.ts:78 Tab titles > signed-in pages each name themselves
attempt 1: failed (20582 ms)
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
