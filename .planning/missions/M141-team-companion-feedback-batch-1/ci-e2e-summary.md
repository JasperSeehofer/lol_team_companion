# CI e2e summary

- run: 38881767825
- sha: d969ee0fa9273bba5e8977bdc9bbf302d9838b2f

## Playwright

expected 126, unexpected 1, flaky 5, skipped 0

### [flaky] match-detail.spec.ts:78 match detail (seeded demo game) > hovering a row highlights the same role in both teams; a tap pins it
attempt 1: failed (6668 ms)
```
Error: expect(locator).toHaveClass(expected) failed

Locator: locator('[data-scoreboard=blue] tbody tr').nth(2)
Expected pattern: /bg-ink-3/
Received string:  "relative hover:bg-ink-2/70 transition-colors"
Timeout: 5000ms

Call log:
  - Expect "toHaveClass" with timeout 5000ms
  - waiting for locator('[data-scoreboard=blue] tbody tr').nth(2)
    9 × locator resolved to <tr class="relative hover:bg-ink-2/70 transition-colors">…</tr>
      - unexpected value "relative hover:bg-ink-2/70 transition-colors"
```

### [flaky] post-game-fill.spec.ts:33 post-game: filled from match > choosing a match fills the result, marks it and can be undone
attempt 1: failed (6627 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator:  locator('[data-segmented="review-outcome"]').getByRole('radio', { name: 'Win' })
Expected: "true"
Received: "false"
Timeout:  5000ms

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for locator('[data-segmented="review-outcome"]').getByRole('radio', { name: 'Win' })
    9 × locator resolved to <button role="radio" type="button" tabindex="-1" data-value="win" aria-checked="false" id="review-outcome-seg-1" class="cursor-pointer whitespace-nowrap rounded-[2px] text-muted transition-colors hover:text-primary data-[force=hover]:text-primary h-7 px-3 text-label">…</button>
      - unexpected value "false"
```

### [unexpected] post-game-fill.spec.ts:63 post-game: filled from match > a result someone set is never overwritten
attempt 1: failed (6628 ms)
```
Error: expect(locator).toBeVisible() failed

Locator: getByTestId('review-match-facts')
Expected: visible
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for getByTestId('review-match-facts')
```
attempt 2: failed (7249 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator:  locator('[data-segmented="review-outcome"]').getByRole('radio', { name: 'Loss' })
Expected: "true"
Received: "false"
Timeout:  5000ms

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for locator('[data-segmented="review-outcome"]').getByRole('radio', { name: 'Loss' })
    9 × locator resolved to <button role="radio" type="button" tabindex="-1" data-value="loss" aria-checked="false" id="review-outcome-seg-2" class="cursor-pointer whitespace-nowrap rounded-[2px] text-muted transition-colors hover:text-primary data-[force=hover]:text-primary h-7 px-3 text-label">…</button>
      - unexpected value "false"
```

### [flaky] sortable-tables.spec.ts:31 sortable tables > champion trends sort by any column, by click and keyboard
attempt 1: failed (6824 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator:  locator('[data-champion-trends]').locator('th').filter({ has: getByRole('button', { name: 'Sort by Win rate' }) })
Expected: "descending"
Received: ""
Timeout:  5000ms

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for locator('[data-champion-trends]').locator('th').filter({ has: getByRole('button', { name: 'Sort by Win rate' }) })
    9 × locator resolved to <th scope="col" class="eyebrow font-medium whitespace-nowrap border-b border-outline pr-4 last:pr-0 py-2 text-right">…</th>
      - unexpected value "null"
```

### [flaky] sortable-tables.spec.ts:48 sortable tables > recent games sort and return to newest first
attempt 1: failed (6582 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator:  locator('[data-my-recent]').locator('th').filter({ has: getByRole('button', { name: 'Sort by CS' }) })
Expected: "descending"
Received: ""
Timeout:  5000ms

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for locator('[data-my-recent]').locator('th').filter({ has: getByRole('button', { name: 'Sort by CS' }) })
    9 × locator resolved to <th scope="col" class="eyebrow font-medium whitespace-nowrap border-b border-outline pr-4 last:pr-0 py-2 text-right">…</th>
      - unexpected value "null"
```

### [flaky] sortable-tables.spec.ts:64 sortable tables > team games by member sort by column
attempt 1: failed (6720 ms)
```
Error: expect(locator).toHaveAttribute(expected) failed

Locator:  locator('[data-member-summary]').locator('th').filter({ has: getByRole('button', { name: 'Sort by KDA' }) })
Expected: "descending"
Received: ""
Timeout:  5000ms

Call log:
  - Expect "toHaveAttribute" with timeout 5000ms
  - waiting for locator('[data-member-summary]').locator('th').filter({ has: getByRole('button', { name: 'Sort by KDA' }) })
    9 × locator resolved to <th scope="col" class="eyebrow font-medium whitespace-nowrap border-b border-outline pr-4 last:pr-0 py-2 text-right">…</th>
      - unexpected value "null"
```

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
