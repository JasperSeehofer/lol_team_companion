# UX rubric check

- run: 44913848967
- sha: 25ac97edb5131f76aa8e61f7f14685c9188f978b
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
