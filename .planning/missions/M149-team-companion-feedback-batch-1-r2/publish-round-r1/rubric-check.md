# UX rubric check

- run: 44910817988
- sha: 6f109aecbdc7f9c7ebb27347dca95a646c8c3c3b
- harness: scripts/ux-rubric/rubric-check.mjs (v24.22.3)
- limitation: only public pages; signed-in surfaces need a sign-in step the harness does not have

| id | page | viewport | verdict | detail |
|----|------|----------|---------|--------|

exit code: 0

```
Error during checks: page.goto: net::ERR_CONNECTION_REFUSED at http://127.0.0.1:3020/auth/login
Call log:
  - navigating to "http://127.0.0.1:3020/auth/login", waiting until "load"

```
