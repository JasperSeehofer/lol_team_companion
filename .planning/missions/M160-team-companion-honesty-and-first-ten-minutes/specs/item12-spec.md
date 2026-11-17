# M160 item 12 — frames-m160 capture spec (mechanical, follows an existing pattern)
Copy the structure of e2e/tests/frames-m148.spec.ts (and frames-m152.spec.ts): read both fully first, including how they pick the output dir from `MISSION_FRAMES_DIR`, viewport sizes 390 and 1280, the `-flow` reading-flow frame (viewer opened, nothing expanded), and how they skip when the env var is unset.
Write e2e/tests/frames-m160.spec.ts capturing, at 390 and 1280 (full-page where the M148 spec does), into `$MISSION_FRAMES_DIR/<sub>/` where `<sub>` = `process.env.MISSION_FRAMES_SUB || "after"`:
1. solo-nav: a fresh authedPage (no team) on `/` — desktop nav at 1280; at 390 open the phone menu (read match-day.spec.ts / solo-mode.spec.ts for the menu button) and capture it.
2. no-team-draft, no-team-game-plan, no-team-post-game: authedPage on /draft, /game-plan, /post-game.
3. checklist: teamPage on `/` (the getting-started checklist visible; read guidance.spec.ts for the selector `[data-checklist]`).
4. team-page-none: authedPage on /team/dashboard (create/join). team-page-leader: teamPage on /team/dashboard. team-page-coach: reuse coach-rights.spec.ts's setup to make a coach and capture the coach's /team/dashboard.
5. auto-place: reuse team-invites.spec.ts's flow (leader mints an invite with position Mid, a new account registers with it) and capture the leader's /team/dashboard with the `[data-auto-placed]` notice.
6. player-notes: reuse opponent-notes.spec.ts's setup (dossier + player); capture the row collapsed with a saved note preview (`-flow`), and once expanded.
7. tree-meta: reuse tree-meta.spec.ts's setup; capture /tree-drafter with a tree open (title "vs {dossier}") and with the Edit form open.
Each capture: `await page.waitForLoadState("networkidle")` like the M148 spec does, file names `<surface>-390.png`, `<surface>-1280.png`, `<surface>-1280-flow.png` for the reading-flow frame. Every surface in its own `test()` so one failure doesn't hide the rest. If a selector from another spec does not exist on main (for the before run), the test must not throw: wrap the surface-specific parts in try/catch and still capture the page as it is (before frames show the old state, e.g. /draft without NoTeamState).
Add frames-m160.spec.ts to the Playwright list in .github/workflows/ci.yml (the same step that lists frames-m148.spec.ts).
