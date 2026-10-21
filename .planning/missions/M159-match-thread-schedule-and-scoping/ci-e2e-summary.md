# CI e2e summary

- run: 39617193146
- sha: ca4dcf8d9d950f5768bc31eb2a0b678c2825ed58

## Playwright

expected 186, unexpected 1, flaky 1, skipped 0

### [flaky] account.spec.ts:15 download my data returns the account as a JSON file
attempt 1: failed (20492 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [unexpected] match-thread.spec.ts:134 match thread: the draft opens prefilled from the thread link
attempt 1: failed (6813 ms)
```
Error: expect(locator).toBeVisible() failed

Locator: locator('[data-prep]').getByRole('link', { name: 'Start a draft' })
Expected: visible
Error: strict mode violation: locator('[data-prep]').getByRole('link', { name: 'Start a draft' }) resolved to 2 elements:
    1) <a href="/draft?match_id=scheduled_match:mskrr9qtxty400qnvfm0&opponent_id=opponent:2a7sn5m9s6lyye3ul509&game=1" class="col-start-2 inline-flex h-8 w-fit items-center gap-2 rounded-ctl border border-outline px-3 text-label font-medium text-primary transition-colors hover:bg-ink-3">…</a> aka getByText('Start a draft→')
    2) <a href="/draft" class="font-medium text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">…</a> aka getByLabel('Preparation', { exact: true }).getByText('Start a draft', { exact: true })

Call log:
  - Expect "toBeVisible" with timeout 10000ms
  - waiting for locator('[data-prep]').getByRole('link', { name: 'Start a draft' })
```
attempt 2: failed (7649 ms)
```
Error: expect(locator).toBeVisible() failed

Locator: locator('[data-prep]').getByRole('link', { name: 'Start a draft' })
Expected: visible
Error: strict mode violation: locator('[data-prep]').getByRole('link', { name: 'Start a draft' }) resolved to 2 elements:
    1) <a href="/draft?match_id=scheduled_match:q27bx11yfgahq40omfq5&opponent_id=opponent:zn8y8d1tsq8ug8t46io9&game=1" class="col-start-2 inline-flex h-8 w-fit items-center gap-2 rounded-ctl border border-outline px-3 text-label font-medium text-primary transition-colors hover:bg-ink-3">…</a> aka getByText('Start a draft→')
    2) <a href="/draft" class="font-medium text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">…</a> aka getByLabel('Preparation', { exact: true }).getByText('Start a draft', { exact: true })

Call log:
  - Expect "toBeVisible" with timeout 10000ms
  - waiting for locator('[data-prep]').getByRole('link', { name: 'Start a draft' })
```

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
