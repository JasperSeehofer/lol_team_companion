# UX rubric check

- run: 44132722397
- sha: 1f13eb6c0cc68105621a308457065084f0b91fe1
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
