# UX rubric check

- run: 44130276511
- sha: 945aa089c0756f7ac9a4933ca9c25438109a9884
- harness: scripts/ux-rubric/rubric-check.mjs (v24.22.2)
- limitation: only public pages; signed-in surfaces need a sign-in step the harness does not have

| id | page | viewport | verdict | detail |
|----|------|----------|---------|--------|

exit code: 0

```
Error during checks: page.goto: net::ERR_CONNECTION_REFUSED at http://127.0.0.1:3020/auth/login
Call log:
  - navigating to "http://127.0.0.1:3020/auth/login", waiting until "load"

```
