# CI e2e summary

- run: 38995081934
- sha: fd95de59dfc7d26bc4183c9e890f500ab809b33e

## Playwright

expected 135, unexpected 2, flaky 0, skipped 0

### [unexpected] password.spec.ts:150 an admin copies the reset code to the clipboard
attempt 1: failed (8401 ms)
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
attempt 2: failed (8505 ms)
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

### [unexpected] password.spec.ts:177 a refused copy says so and never claims success
attempt 1: failed (8346 ms)
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
attempt 2: failed (8623 ms)
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

## Server log (slow request, inflight, panicked, ERROR)

0 matching lines
```
```
