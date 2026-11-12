# CI e2e summary

- run: 43291681609
- sha: 9a19b64c76d2e2cbe8c5c619af7dafa4577c12c2

## Playwright

expected 204, unexpected 1, flaky 0, skipped 0

### [unexpected] tree-meta.spec.ts:17 tree meta: pick a dossier when creating, rename via Edit, title shows vs {dossier}
attempt 1: failed (12546 ms)
```
Error: expect(locator).toHaveText(expected) failed

Locator: getByTestId('tree-title')
Expected: "MetaTree_1794515279170"
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toHaveText" with timeout 5000ms
  - waiting for getByTestId('tree-title')
```
attempt 2: timedOut (60325 ms)
```
Test timeout of 60000ms exceeded.
```

## Server log (slow request, inflight, panicked, ERROR)

1 matching lines
```
thread 'tokio-rt-worker' (21735) panicked at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/leptos_router-0.8.12/src/link.rs:161:16:
```
