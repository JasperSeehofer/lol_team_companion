# CI e2e summary

- run: 39839018128
- sha: e3eae81b61b09f3b3af48c71c5a955f1cb298c8c

## Playwright

expected 147, unexpected 0, flaky 19, skipped 0

### [flaky] auth.spec.ts:128 16+ and terms on sign-up (M148 item 7) > a direct post without accept_terms leaves a one-use invite unused
attempt 1: failed (23468 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] bug-report.spec.ts:97 bug-report widget > Give Feedback offers Bug, Idea and Praise; praise is sent (M141, report y1abg605)
attempt 1: failed (20557 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] bug-report.spec.ts:117 bug-report widget > an empty report is refused in the dialog
attempt 1: failed (20564 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] champion-pool-moves.spec.ts:91 champion-pool moves: a failed save reverts and offers Retry
attempt 1: failed (20566 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] community-gates.spec.ts:25 admin pages are 404 for a signed-in non-admin
attempt 1: failed (20562 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] cross-tenant.spec.ts:218 users see only their own bug reports and cannot answer another user's survey
attempt 1: failed (24392 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] feedback-attachment.spec.ts:25 a file attached to feedback reaches the admin as WebP, and only the admin
attempt 1: failed (20531 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] frames-m148.spec.ts:154 M148 frames > draft saved and failed (draft, draft-save-failed)
attempt 1: failed (20573 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] frames-m148.spec.ts:181 M148 frames > tree drafter saved and failed (tree-drafter, tree-save-failed)
attempt 1: failed (20558 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] guidance.spec.ts:56 the checklist ticks after creating a team and a draft, and hides on request
attempt 1: failed (20572 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /draft hydrates without tachys panic
attempt 1: failed (20550 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /champion-pool hydrates without tachys panic
attempt 1: failed (20555 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /post-game hydrates without tachys panic
attempt 1: failed (20553 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /match/test-match-id hydrates without tachys panic
attempt 1: failed (20570 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:95 /transparency hydrates without tachys panic
attempt 1: failed (20580 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:114 stale theme cookie is ignored
attempt 1: failed (20582 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hydration-no-panic.spec.ts:161 draft slot click-fill
attempt 1: failed (20576 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] match-detail.spec.ts:7 match detail page > renders match detail page without errors
attempt 1: failed (20538 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] team-invites.spec.ts:39 a new player registers into the team with the leader's link
attempt 1: failed (20526 ms)
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
