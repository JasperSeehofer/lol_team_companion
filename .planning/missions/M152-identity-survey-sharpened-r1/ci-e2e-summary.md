# CI e2e summary

- run: 39077606005
- sha: 8cfbd31aeb46c85914da49b23e65e0bddb124f44

## Playwright

expected 184, unexpected 0, flaky 2, skipped 0

### [flaky] champion-pool-moves.spec.ts:32 champion-pool moves: keyboard move keeps the scroll and shows the save state
attempt 1: failed (20572 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] hero-matchup.spec.ts:144 shared game plan: five champions per team never wrap at 320, 390 and 1280
attempt 1: failed (12503 ms)
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

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
