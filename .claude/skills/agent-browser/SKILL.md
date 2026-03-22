# agent-browser Skill

Interactive browser verification for Claude Code sessions using Vercel's `agent-browser` CLI.

## What It Is

`agent-browser` is a headless browser automation CLI designed for AI agents (npm: `agent-browser`, Apache-2.0).
It provides navigate/screenshot/click/type capabilities for quick one-off browser checks during development.

GitHub: https://github.com/vercel-labs/agent-browser

## Setup

Install globally once:
```bash
npm install -g agent-browser
```

Or invoke via npx without installation:
```bash
npx agent-browser <command>
```

## Common Commands

### Navigate and screenshot
```bash
npx agent-browser screenshot http://127.0.0.1:3002/
npx agent-browser screenshot http://127.0.0.1:3002/draft
```

### Navigate to a URL
```bash
npx agent-browser navigate http://127.0.0.1:3002/game-plan
```

### Click an element (CSS selector)
```bash
npx agent-browser click "button:has-text('Save Draft')"
```

### Type into an input
```bash
npx agent-browser type "input[name=username]" "myuser"
```

### Get page text (accessibility snapshot)
```bash
npx agent-browser snapshot http://127.0.0.1:3002/team/dashboard
```

## Integration With This Project

The dev server runs at `http://127.0.0.1:3002` (start with `cargo leptos watch`).

### Auth Pattern

The dev server uses session cookies. To get an authenticated browser session,
register a test user first via the e2e auth fixture or manually:

```bash
# Navigate to register
npx agent-browser navigate http://127.0.0.1:3002/auth/register

# Fill registration form
npx agent-browser type "input[name=username]" "devuser"
npx agent-browser type "input[name=email]" "dev@test.invalid"
npx agent-browser type "input[name=password]" "Test1234!"
npx agent-browser click "button[type=submit]"
```

After registration, the session is persisted for the browser instance.

### Verification Patterns

**New page — confirm it renders:**
```bash
npx agent-browser screenshot http://127.0.0.1:3002/my-new-page
npx agent-browser snapshot http://127.0.0.1:3002/my-new-page
```

**Form submission — check result:**
```bash
npx agent-browser type "input[name=draft_name]" "MyDraft"
npx agent-browser click "button:has-text('Save Draft')"
npx agent-browser snapshot http://127.0.0.1:3002/draft  # confirm success state
```

**Error handling — trigger error condition:**
```bash
npx agent-browser navigate "http://127.0.0.1:3002/game-plan?draft_id=nonexistent"
npx agent-browser snapshot  # confirm graceful empty state, not error banner
```

## When to Use agent-browser vs e2e Tests

| Use Case | Tool |
|----------|------|
| Quick one-off check during development | agent-browser |
| Regression coverage before commit | `just e2e` or `cd e2e && npx playwright test` |
| Verify a specific page renders | agent-browser screenshot |
| Confirm a flow works end-to-end | `cd e2e && npx playwright test pipeline.spec.ts` |
| Check all pages for errors | `cd e2e && npx playwright test smoke.spec.ts` |

## Notes

- WASM hydration takes ~500ms after page load (CLAUDE.md rule 56). Add `--wait 500` or screenshot after a brief delay.
- The app uses session cookies — a new agent-browser session starts unauthenticated.
- Protected pages redirect to `/auth/login` if not authenticated (CLAUDE.md rule 50).
