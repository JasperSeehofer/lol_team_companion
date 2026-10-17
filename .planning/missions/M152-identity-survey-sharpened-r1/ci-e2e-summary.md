# CI e2e summary

- run: 39073124625
- sha: 18695d10b90c3843d75d9d42006a564cf700051b

## Playwright

expected 181, unexpected 0, flaky 5, skipped 0

### [flaky] bug-report.spec.ts:97 bug-report widget > Give Feedback offers Bug, Idea and Praise; praise is sent (M141, report y1abg605)
attempt 1: failed (20618 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hero-matchup.spec.ts:144 shared game plan: five champions per team never wrap at 320, 390 and 1280
attempt 1: failed (12927 ms)
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

### [flaky] hydration-no-panic.spec.ts:150 segmented control observable on /me
attempt 1: failed (20573 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] match-detail.spec.ts:7 match detail page > renders match detail page without errors
attempt 1: failed (20564 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] team-invites.spec.ts:177 a coach invites players; a plain member sees no invite panel
attempt 1: failed (28307 ms)
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
