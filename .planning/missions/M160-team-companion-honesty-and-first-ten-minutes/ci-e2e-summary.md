# CI e2e summary

- run: 40407373608
- sha: 013f44c9e1fc2cac3a99539a90acf753e9297b3f

## Playwright

expected 187, unexpected 4, flaky 1, skipped 0

### [unexpected] gallery.spec.ts:318 command bar: Ctrl+K opens it, typing filters, Enter goes
attempt 1: failed (8134 ms)
```
Error: expect(locator).toHaveText(expected) failed

Locator: locator('dialog[data-dialog="command-bar"]').locator('[role="option"]').first()
Expected pattern: /Roster/
Received string:  "Find or create a teamTeam"
Timeout: 5000ms

Call log:
  - Expect "toHaveText" with timeout 5000ms
  - waiting for locator('dialog[data-dialog="command-bar"]').locator('[role="option"]').first()
    9 × locator resolved to <a role="option" tabindex="-1" aria-selected="true" id="command-bar-opt-5" href="/team/dashboard" class="flex items-center justify-between gap-4 px-4 py-2 text-body text-secondary no-underline hover:bg-ink-2 aria-selected:bg-ink-3 aria-selected:text-primary">…</a>
      - unexpected value "Find or create a teamTeam"
```
attempt 2: failed (8461 ms)
```
Error: expect(locator).toHaveText(expected) failed

Locator: locator('dialog[data-dialog="command-bar"]').locator('[role="option"]').first()
Expected pattern: /Roster/
Received string:  "Find or create a teamTeam"
Timeout: 5000ms

Call log:
  - Expect "toHaveText" with timeout 5000ms
  - waiting for locator('dialog[data-dialog="command-bar"]').locator('[role="option"]').first()
    9 × locator resolved to <a role="option" tabindex="-1" aria-selected="true" id="command-bar-opt-5" href="/team/dashboard" class="flex items-center justify-between gap-4 px-4 py-2 text-body text-secondary no-underline hover:bg-ink-2 aria-selected:bg-ink-3 aria-selected:text-primary">…</a>
      - unexpected value "Find or create a teamTeam"
```

### [unexpected] guidance.spec.ts:20 first sign-in shows the welcome once; Skip keeps the checklist
attempt 1: failed (7439 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator:  getByTestId('welcome-start')
Expected: "/team/roster"
Received: "/team/dashboard"
Timeout:  5000ms

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for getByTestId('welcome-start')
    9 × locator resolved to <a href="/team/dashboard" data-testid="welcome-start" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-8 px-3 text-label bg-accent border-accent text-accent-ink font-semibold hover:bg-accent-hover hov…>…</a>
      - unexpected value "/team/dashboard"
```
attempt 2: failed (7651 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator:  getByTestId('welcome-start')
Expected: "/team/roster"
Received: "/team/dashboard"
Timeout:  5000ms

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for getByTestId('welcome-start')
    9 × locator resolved to <a href="/team/dashboard" data-testid="welcome-start" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-8 px-3 text-label bg-accent border-accent text-accent-ink font-semibold hover:bg-accent-hover hov…>…</a>
      - unexpected value "/team/dashboard"
```

### [unexpected] guidance.spec.ts:44 Start records the welcome: it does not come back
attempt 1: failed (12542 ms)
```
TimeoutError: page.waitForURL: Timeout 10000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```
attempt 2: failed (12694 ms)
```
TimeoutError: page.waitForURL: Timeout 10000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [flaky] login-next.spec.ts:46 Return to the page after sign-in > a forged next to another site lands on Home
attempt 1: failed (20562 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [unexpected] team-invites.spec.ts:39 a new player registers into the team with the leader's link
attempt 1: failed (11748 ms)
```
Error: expect(locator).toContainText(expected) failed

Locator: locator('[data-roster-group="unplaced"]')
Expected substring: "invited_1793052693452_7"
Received string:    "Not placed yet1Ppagesuser_1793052530469_7x21No Riot account linked · youActionsStart as TopStart as JungleStart as MidStart as BotStart as SupportMove to the benchMake coach"
Timeout: 5000ms

Call log:
  - Expect "toContainText" with timeout 5000ms
  - waiting for locator('[data-roster-group="unplaced"]')
    9 × locator resolved to <div class="mt-8" data-roster-group="unplaced">…</div>
      - unexpected value "Not placed yet1Ppagesuser_1793052530469_7x21No Riot account linked · youActionsStart as TopStart as JungleStart as MidStart as BotStart as SupportMove to the benchMake coach"
```
attempt 2: failed (12152 ms)
```
Error: expect(locator).toContainText(expected) failed

Locator: locator('[data-roster-group="unplaced"]')
Expected substring: "invited_1793052705966_8"
Received string:    "Not placed yet1Ppagesuser_1793052705963_8x1No Riot account linked · youActionsStart as TopStart as JungleStart as MidStart as BotStart as SupportMove to the benchMake coach"
Timeout: 5000ms

Call log:
  - Expect "toContainText" with timeout 5000ms
  - waiting for locator('[data-roster-group="unplaced"]')
    9 × locator resolved to <div class="mt-8" data-roster-group="unplaced">…</div>
      - unexpected value "Not placed yet1Ppagesuser_1793052705963_8x1No Riot account linked · youActionsStart as TopStart as JungleStart as MidStart as BotStart as SupportMove to the benchMake coach"
```

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
