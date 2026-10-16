# CI e2e summary

- run: 38876857165
- sha: f0bdce1cbca491f96c98ad26f40cb680996b0b47

## Playwright

expected 123, unexpected 9, flaky 0, skipped 0

### [unexpected] champion-pool-moves.spec.ts:65 champion-pool moves: drag and drop keeps the scroll
attempt 1: failed (5899 ms)
```
Error: expect(received).toBeLessThanOrEqual(expected)

Expected: <= 2
Received:    295
```
attempt 2: failed (6501 ms)
```
Error: expect(received).toBeLessThanOrEqual(expected)

Expected: <= 2
Received:    295
```

### [unexpected] enter-submits.spec.ts:10 enter submits: tree drafter adds a branch on Enter
attempt 1: failed (6195 ms)
```
Error: locator.click: Error: strict mode violation: getByRole('treeitem').first().getByRole('button', { name: 'Branch', exact: true }) resolved to 2 elements:
    1) <button type="button" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-7 px-2.5 text-label bg-transparent border-transparent text-secondary hover:bg-ink-3 hover:text-primary data-[force=hover]:bg-ink-3 dat…>…</button> aka getByRole('button', { name: 'Branch' }).nth(1)
    2) <button type="button" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-7 px-2.5 text-label bg-transparent border-transparent text-secondary hover:bg-ink-3 hover:text-primary data-[force=hover]:bg-ink-3 dat…>…</button> aka getByRole('group').getByRole('button', { name: 'Branch' })

Call log:
  - waiting for getByRole('treeitem').first().getByRole('button', { name: 'Branch', exact: true })
```
attempt 2: failed (6851 ms)
```
Error: locator.click: Error: strict mode violation: getByRole('treeitem').first().getByRole('button', { name: 'Branch', exact: true }) resolved to 2 elements:
    1) <button type="button" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-7 px-2.5 text-label bg-transparent border-transparent text-secondary hover:bg-ink-3 hover:text-primary data-[force=hover]:bg-ink-3 dat…>…</button> aka getByRole('button', { name: 'Branch' }).nth(1)
    2) <button type="button" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-7 px-2.5 text-label bg-transparent border-transparent text-secondary hover:bg-ink-3 hover:text-primary data-[force=hover]:bg-ink-3 dat…>…</button> aka getByRole('group').getByRole('button', { name: 'Branch' })

Call log:
  - waiting for getByRole('treeitem').first().getByRole('button', { name: 'Branch', exact: true })
```

### [unexpected] feedback-attachment.spec.ts:25 a file attached to feedback reaches the admin as WebP, and only the admin
attempt 1: failed (3483 ms)
```
Error: locator.setInputFiles: Error: strict mode violation: getByRole('dialog', { name: 'Give Feedback' }).locator('#bug-report-file') resolved to 2 elements:
    1) <textarea rows="5" maxlength="4000" id="bug-report-file" aria-describedby="bug-report-file-hint" placeholder="What did you do, what did you expect, what happened instead?" class="w-full rounded-ctl border border-outline bg-ink-2 px-3 text-primary placeholder:text-muted transition-colors hover:border-secondary data-[force=hover]:border-secondary disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-danger py-2 text-body min-h-20 resize-y leading-relaxed"></textarea> aka getByRole('textbox', { name: 'Screenshot (optional)' })
    2) <input type="file" tabindex="-1" class="sr-only" id="bug-report-file" accept="image/png,image/jpeg,image/webp"/> aka getByRole('button', { name: 'Choose File' })

Call log:
  - waiting for getByRole('dialog', { name: 'Give Feedback' }).locator('#bug-report-file')
```
attempt 2: failed (3903 ms)
```
Error: locator.setInputFiles: Error: strict mode violation: getByRole('dialog', { name: 'Give Feedback' }).locator('#bug-report-file') resolved to 2 elements:
    1) <textarea rows="5" maxlength="4000" id="bug-report-file" aria-describedby="bug-report-file-hint" placeholder="What did you do, what did you expect, what happened instead?" class="w-full rounded-ctl border border-outline bg-ink-2 px-3 text-primary placeholder:text-muted transition-colors hover:border-secondary data-[force=hover]:border-secondary disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-danger py-2 text-body min-h-20 resize-y leading-relaxed"></textarea> aka getByRole('textbox', { name: 'Screenshot (optional)' })
    2) <input type="file" tabindex="-1" class="sr-only" id="bug-report-file" accept="image/png,image/jpeg,image/webp"/> aka getByRole('button', { name: 'Choose File' })

Call log:
  - waiting for getByRole('dialog', { name: 'Give Feedback' }).locator('#bug-report-file')
```

### [unexpected] feedback-attachment.spec.ts:67 a file that is not an image is refused in the dialog
attempt 1: failed (3508 ms)
```
Error: locator.setInputFiles: Error: strict mode violation: getByRole('dialog', { name: 'Give Feedback' }).locator('#bug-report-file') resolved to 2 elements:
    1) <textarea rows="5" maxlength="4000" id="bug-report-file" aria-describedby="bug-report-file-hint" placeholder="What did you do, what did you expect, what happened instead?" class="w-full rounded-ctl border border-outline bg-ink-2 px-3 text-primary placeholder:text-muted transition-colors hover:border-secondary data-[force=hover]:border-secondary disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-danger py-2 text-body min-h-20 resize-y leading-relaxed"></textarea> aka getByRole('textbox', { name: 'Screenshot (optional)' })
    2) <input type="file" tabindex="-1" class="sr-only" id="bug-report-file" accept="image/png,image/jpeg,image/webp"/> aka getByRole('button', { name: 'Choose File' })

Call log:
  - waiting for getByRole('dialog', { name: 'Give Feedback' }).locator('#bug-report-file')
```
attempt 2: failed (3815 ms)
```
Error: locator.setInputFiles: Error: strict mode violation: getByRole('dialog', { name: 'Give Feedback' }).locator('#bug-report-file') resolved to 2 elements:
    1) <textarea rows="5" maxlength="4000" id="bug-report-file" aria-describedby="bug-report-file-hint" placeholder="What did you do, what did you expect, what happened instead?" class="w-full rounded-ctl border border-outline bg-ink-2 px-3 text-primary placeholder:text-muted transition-colors hover:border-secondary data-[force=hover]:border-secondary disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-danger py-2 text-body min-h-20 resize-y leading-relaxed"></textarea> aka getByRole('textbox', { name: 'Screenshot (optional)' })
    2) <input type="file" tabindex="-1" class="sr-only" id="bug-report-file" accept="image/png,image/jpeg,image/webp"/> aka getByRole('button', { name: 'Choose File' })

Call log:
  - waiting for getByRole('dialog', { name: 'Give Feedback' }).locator('#bug-report-file')
```

### [unexpected] post-game-fill.spec.ts:33 post-game: filled from match > choosing a match fills the result, marks it and can be undone
attempt 1: failed (6662 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator: locator('#review-outcome').getByRole('radio', { name: 'Not set' })
Expected: "true"
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for locator('#review-outcome').getByRole('radio', { name: 'Not set' })
```
attempt 2: failed (6952 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator: locator('#review-outcome').getByRole('radio', { name: 'Not set' })
Expected: "true"
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for locator('#review-outcome').getByRole('radio', { name: 'Not set' })
```

### [unexpected] post-game-fill.spec.ts:63 post-game: filled from match > a result someone set is never overwritten
attempt 1: timedOut (60031 ms)
```
Test timeout of 60000ms exceeded.
```
attempt 2: timedOut (60096 ms)
```
Test timeout of 60000ms exceeded.
```

### [unexpected] sortable-tables.spec.ts:31 sortable tables > champion trends sort by any column, by click and keyboard
attempt 1: failed (6813 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator: locator('[data-champion-trends]').locator('th').filter({ has: locator('[data-champion-trends]').getByRole('button', { name: 'Sort by Win rate' }) })
Expected: "descending"
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for locator('[data-champion-trends]').locator('th').filter({ has: locator('[data-champion-trends]').getByRole('button', { name: 'Sort by Win rate' }) })
```
attempt 2: failed (7234 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator: locator('[data-champion-trends]').locator('th').filter({ has: locator('[data-champion-trends]').getByRole('button', { name: 'Sort by Win rate' }) })
Expected: "descending"
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for locator('[data-champion-trends]').locator('th').filter({ has: locator('[data-champion-trends]').getByRole('button', { name: 'Sort by Win rate' }) })
```

### [unexpected] sortable-tables.spec.ts:48 sortable tables > recent games sort and return to newest first
attempt 1: failed (6929 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator: locator('[data-my-recent]').locator('th').filter({ has: locator('[data-my-recent]').getByRole('button', { name: 'Sort by CS' }) })
Expected: "descending"
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for locator('[data-my-recent]').locator('th').filter({ has: locator('[data-my-recent]').getByRole('button', { name: 'Sort by CS' }) })
```
attempt 2: failed (7123 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator: locator('[data-my-recent]').locator('th').filter({ has: locator('[data-my-recent]').getByRole('button', { name: 'Sort by CS' }) })
Expected: "descending"
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for locator('[data-my-recent]').locator('th').filter({ has: locator('[data-my-recent]').getByRole('button', { name: 'Sort by CS' }) })
```

### [unexpected] sortable-tables.spec.ts:62 sortable tables > team games by member sort by column
attempt 1: failed (6867 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator: locator('[data-member-summary]').locator('th').filter({ has: locator('[data-member-summary]').getByRole('button', { name: 'Sort by KDA' }) })
Expected: "descending"
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for locator('[data-member-summary]').locator('th').filter({ has: locator('[data-member-summary]').getByRole('button', { name: 'Sort by KDA' }) })
```
attempt 2: failed (7077 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator: locator('[data-member-summary]').locator('th').filter({ has: locator('[data-member-summary]').getByRole('button', { name: 'Sort by KDA' }) })
Expected: "descending"
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for locator('[data-member-summary]').locator('th').filter({ has: locator('[data-member-summary]').getByRole('button', { name: 'Sort by KDA' }) })
```

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
