# M160 item 12 — fix round 1 (from the Sonnet verifier; commit 9466d0b, e2e/tests/frames-m160.spec.ts)
1. Coach test (~:185-203): `coach` is never signed in, so `coach.goto(link)` shows the register form. Call `const { username: coachName } = await authenticatePage(coach)` (read fixtures.ts for the exact helper name and return shape) BEFORE `goto(link)`; use `coachName` wherever the coach's username is needed.
2. Coach test (~:211-219): delete the /profile detour, the `coach.url().split('=')` code and the unused `coachRow`; use `coachName` from point 1.
3. Guard every surface-specific step that does not exist on main, so a run on main still writes its frames and the test does not fail: wrap in `try { ... } catch {}` with short timeouts (`{ timeout: 3000 }`) — `#team-invite-position` selectOption (~:256; still mint the invite after), the Add note / Edit note clicks and fills (~:310-317, ~:331-333), `tree-edit-open` / `tree-edit-form` (~:377-379), and the `phone-menu-trigger` click (~:83).
4. solo-nav: don't write `solo-nav-390.png` twice — name the phone-menu capture `solo-nav-menu-390.png`.
5. In the shared `frames()` helper (~:45): change `expect(sw).toBeLessThanOrEqual(390)` to `expect.soft(...)`.
6. Remove unused imports (`signInAsAdmin`, `INVITE_CODE`, `Browser`, `BrowserContext`) if unused after the changes.
Only e2e/tests/frames-m160.spec.ts may change.
