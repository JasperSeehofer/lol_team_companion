# CI e2e summary

- run: 38997162828
- sha: 633f125f5708a807b9107fbfecb9956b8f4b5b46

## Playwright

expected 147, unexpected 4, flaky 1, skipped 0

### [flaky] account.spec.ts:15 download my data returns the account as a JSON file
attempt 1: failed (20565 ms)
```
TimeoutError: page.waitForURL: Timeout 20000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================
```

### [unexpected] password.spec.ts:151 an admin copies the reset code to the clipboard
attempt 1: failed (8536 ms)
```
Error: expect(locator).toHaveText(expected) failed

Locator:  getByTestId('copy-reset-code')
Expected: "Copy code"
Received: "Copy codeCopy codeCopied"
Timeout:  5000ms

Call log:
  - Expect "toHaveText" with timeout 5000ms
  - waiting for getByTestId('copy-reset-code')
    9 × locator resolved to <button type="button" data-testid="copy-reset-code" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-11 px-4 text-body md:h-8 md:px-3 md:text-label bg-transparent border-outline text-primary hover:bg-ink-3…>…</button>
      - unexpected value "Copy codeCopy codeCopied"
```
attempt 2: failed (8467 ms)
```
Error: expect(locator).toHaveText(expected) failed

Locator:  getByTestId('copy-reset-code')
Expected: "Copy code"
Received: "Copy codeCopy codeCopied"
Timeout:  5000ms

Call log:
  - Expect "toHaveText" with timeout 5000ms
  - waiting for getByTestId('copy-reset-code')
    9 × locator resolved to <button type="button" data-testid="copy-reset-code" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-11 px-4 text-body md:h-8 md:px-3 md:text-label bg-transparent border-outline text-primary hover:bg-ink-3…>…</button>
      - unexpected value "Copy codeCopy codeCopied"
```

### [unexpected] password.spec.ts:178 a refused copy says so and never claims success
attempt 1: failed (8329 ms)
```
Error: expect(locator).toHaveText(expected) failed

Locator:  getByTestId('copy-reset-code')
Expected: "Copy code"
Received: "Copy codeCopy codeCopied"
Timeout:  5000ms

Call log:
  - Expect "toHaveText" with timeout 5000ms
  - waiting for getByTestId('copy-reset-code')
    9 × locator resolved to <button type="button" data-testid="copy-reset-code" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-11 px-4 text-body md:h-8 md:px-3 md:text-label bg-transparent border-outline text-primary hover:bg-ink-3…>…</button>
      - unexpected value "Copy codeCopy codeCopied"
```
attempt 2: failed (8478 ms)
```
Error: expect(locator).toHaveText(expected) failed

Locator:  getByTestId('copy-reset-code')
Expected: "Copy code"
Received: "Copy codeCopy codeCopied"
Timeout:  5000ms

Call log:
  - Expect "toHaveText" with timeout 5000ms
  - waiting for getByTestId('copy-reset-code')
    9 × locator resolved to <button type="button" data-testid="copy-reset-code" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-11 px-4 text-body md:h-8 md:px-3 md:text-label bg-transparent border-outline text-primary hover:bg-ink-3…>…</button>
      - unexpected value "Copy codeCopy codeCopied"
```

### [unexpected] team-invites.spec.ts:119 after a reload an open link shows in full and Copy copies it
attempt 1: failed (10083 ms)
```
Error: expect(locator).toHaveText(expected) failed

Locator:  locator('[data-team-invites] [data-team-invite]').first().getByTestId('copy-invite-link')
Expected: "Copy link"
Received: "Copy linkCopy linkCopied"
Timeout:  5000ms

Call log:
  - Expect "toHaveText" with timeout 5000ms
  - waiting for locator('[data-team-invites] [data-team-invite]').first().getByTestId('copy-invite-link')
    9 × locator resolved to <button type="button" data-testid="copy-invite-link" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-11 px-4 text-body md:h-8 md:px-3 md:text-label bg-transparent border-outline text-primary hover:bg-ink-…>…</button>
      - unexpected value "Copy linkCopy linkCopied"
```
attempt 2: failed (10399 ms)
```
Error: expect(locator).toHaveText(expected) failed

Locator:  locator('[data-team-invites] [data-team-invite]').first().getByTestId('copy-invite-link')
Expected: "Copy link"
Received: "Copy linkCopy linkCopied"
Timeout:  5000ms

Call log:
  - Expect "toHaveText" with timeout 5000ms
  - waiting for locator('[data-team-invites] [data-team-invite]').first().getByTestId('copy-invite-link')
    9 × locator resolved to <button type="button" data-testid="copy-invite-link" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-11 px-4 text-body md:h-8 md:px-3 md:text-label bg-transparent border-outline text-primary hover:bg-ink-…>…</button>
      - unexpected value "Copy linkCopy linkCopied"
```

### [unexpected] team-invites.spec.ts:148 a refused copy says so and never claims success
attempt 1: failed (10197 ms)
```
Error: expect(locator).toHaveText(expected) failed

Locator:  locator('[data-team-invites] [data-team-invite]').first().getByTestId('copy-invite-link')
Expected: "Copy link"
Received: "Copy linkCopy linkCopied"
Timeout:  5000ms

Call log:
  - Expect "toHaveText" with timeout 5000ms
  - waiting for locator('[data-team-invites] [data-team-invite]').first().getByTestId('copy-invite-link')
    9 × locator resolved to <button type="button" data-testid="copy-invite-link" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-11 px-4 text-body md:h-8 md:px-3 md:text-label bg-transparent border-outline text-primary hover:bg-ink-…>…</button>
      - unexpected value "Copy linkCopy linkCopied"
```
attempt 2: failed (10389 ms)
```
Error: expect(locator).toHaveText(expected) failed

Locator:  locator('[data-team-invites] [data-team-invite]').first().getByTestId('copy-invite-link')
Expected: "Copy link"
Received: "Copy linkCopy linkCopied"
Timeout:  5000ms

Call log:
  - Expect "toHaveText" with timeout 5000ms
  - waiting for locator('[data-team-invites] [data-team-invite]').first().getByTestId('copy-invite-link')
    9 × locator resolved to <button type="button" data-testid="copy-invite-link" class="relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress aria-pressed:border-secondary aria-pressed:bg-ink-3 aria-pressed:font-semibold aria-pressed:text-primary h-11 px-4 text-body md:h-8 md:px-3 md:text-label bg-transparent border-outline text-primary hover:bg-ink-…>…</button>
      - unexpected value "Copy linkCopy linkCopied"
```

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
