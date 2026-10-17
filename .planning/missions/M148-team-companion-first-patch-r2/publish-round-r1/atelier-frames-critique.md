# M148 final atelier pass: frame critique of CI run 39000854398 (2026-10-17)

Mode: contact-sheet, run by the atelier agent itself. Frames only (no new capture); every PNG under `frames/after/` was opened, `before/` compared for C1, C2, C6. Tokens: `input.css` and `docs/DESIGN.md` only. Cards: `../atelier-critique-r2.md`. Process note: this run had no subagent tool, so the "reads to Sonnet" clause could not be followed; reads and the one file write were done here. No app file touched, nothing committed.

Method: widths read from the PNG headers (a 390 capture wider than 390 = horizontal scroll); contrast recomputed from rendered pixels (PIL); touch heights measured from the crops.

## Verdict table

| Surface | 390 | 1280 | Flow frame | Verdict |
|---|---|---|---|---|
| admin-resets (C9) | page 393 wide (3 px overflow) | good | good | FIX (minor) |
| dashboard-invites (C1) | page 400 wide (10 px overflow, team name) | good | good | FIX (minor) |
| match-day-phone (C2) | good, Add the next match chalk, 44 px | good | good | PASS |
| draft (C4) | good | good | good | PASS |
| draft-save-failed | state line, Retry, toast right; sticky-nav capture artefact | good | good | PASS (re-capture advised) |
| game-plan | good | good | good | PASS |
| game-plan-save-failed | as draft | good | good | PASS (re-capture advised) |
| tree-drafter | good | three toasts stacked over the left column in the flow frame | noisy | PASS |
| tree-save-failed (C4/C5) | page 455 wide: 65 px horizontal scroll | good | good | FIX (real) |
| register (C6) | good, checkbox label 44 px | good | good | PASS |
| login | good | good | good | PASS (one 4 px FIX) |
| login-next (C10) | identical to login, as intended | same | same | PASS |
| auth-reset | good | good | good | PASS |
| profile-password | PENDING | PENDING | PENDING | PENDING: HEAD c3b38b8 adds only the spec lines; no `after/profile-password-*` or `after/login-hint-*` PNG exists in the worktree |
| login-hint | PENDING (the hint is visible on the login frames above: "Forgot it? Ask the Team Companion admin for a reset code." under Password plus the link) | PENDING | PENDING | PENDING as a surface; content PASS from login |

Contrast from rendered pixels: failed save line (text-danger on bg-page) 7.02:1; login/register hints (text-muted) 6.96 to 7.04:1; reset callout title 8.72:1; Copy link label 16.21:1. All pass 4.5:1. No off-palette colour on any frame (chalk accent button, danger red, info blue-teal callout are all token hues). One primary action per view on every surface (the match-day welcome "Start" and checklist actions render Quiet/Secondary, so the chalk button is the single primary).

## FIX list (exact fixes)

### FIX 1 (real, highest): tree drafter failed state scrolls sideways at 390 (455 px)
Cause: `src/components/ui/layout.rs:187`, the `SectionHeader` actions wrapper is `flex items-center gap-2 shrink-0` with no wrap. With the failed state it holds "Could not save: the server did not answer" + Retry (Touch) + "Save node", wider than 390 and unable to shrink, so the page grows. The same row used in `src/pages/tree_drafter.rs:1048-1052`.
Fix: `layout.rs:187` change class to `flex min-w-0 max-w-full flex-wrap items-center gap-x-2 gap-y-1` (drop `shrink-0`; the outer row already wraps with `flex-wrap`, so the actions drop under the title). Check no other `SectionHeader` caller needs `shrink-0`; the actions are buttons, which wrap whole. Gate: Playwright `document.documentElement.scrollWidth <= 390` on tree-save-failed at 390 (add to `frames-m148.spec.ts`).

### FIX 2: dashboard overflows 10 px at 390 (400 wide) when the team name has no break opportunity
Cause: `src/components/ui/layout.rs:134`, the `PageHeader` `<h1>` has `text-balance` but nothing breaks an unbroken string ("FramesInv_179221034888 2" is clipped in `dashboard-invites-390.png`). Real team names can be long single words.
Fix: add `[overflow-wrap:anywhere]` to the h1 class at `layout.rs:134` (the parent div is already `min-w-0`). Not `break-all`: normal names keep word wraps.

### FIX 3: admin resets overflows 3 px at 390 (393 wide): long email in the Callout title
Cause: `src/components/ui/callout.rs:126`, the title `<p>` has no wrap rule, so "Reset code for frames_reset_...@test.invalid" is clipped at the right edge (`admin-resets-390.png`; the container is already `min-w-0`).
Fix: `callout.rs:126` add `[overflow-wrap:anywhere]` to the title class. Optional: the "Recent codes" table clips CREATED and BY inside its own `overflow-x-auto` (table.rs:57); that is an acceptable in-table scroll, but `Table` could show the status as the second column first; no change required.

### FIX 4 (4 px, optional): login "Have a reset code?" link is 40 px tall
Source `src/pages/auth/login.rs:134`, `py-2.5` + `text-label` = about 40 px, under the 44 px rule A1 on a key recovery path (the harness A1 PASS skips inline links).
Fix: `py-2.5` becomes `py-3` (44 px). No layout change at 1280 beyond 8 px.

### Capture advice (not app code)
- `draft-save-failed-390.png` and `game-plan-save-failed-390.png`: the sticky site header was painted mid-page (full-page shot after a scrolled focus), hiding the page header's Save button and a stripe of the title. Take the 390 frame after `window.scrollTo(0,0)` or with the header `position: static` so the Save button and Retry heights can be verified (Retry is `ButtonSize::Touch` in `status.rs:270`, so 44 px by construction).
- `tree-drafter-1280-flow.png`: three toasts (Tree created, Branch added, Node saved) are stacked over the page from the setup steps. Dismiss them (or wait 6 s) before the flow shot; the flow frame should show first-view state only. Separately, three toasts for one setup sequence is a toast-per-step pattern; not a defect of this patch.

## Per-surface notes

- **admin-resets (C9):** Recipe followed: code in the Info callout, `Copy code` Secondary Touch under the paragraph, left-aligned (43 to 44 px at 390, 32 px at 1280), one live region. Create reset code is the single chalk action. Flow frame reads: title, field, button, code, expiry copy, Copy, list. FIX 3 only.
- **dashboard-invites (C1):** Full URL shown, wrapped with `break-all`, status line below, `Copy link` Secondary Touch (44 px at 390, 32 at 1280) and a Quiet `Disable`; the status word is written (open, 0 of 1 used, expires date). No Share button on desktop, as specified. The row is the only action cluster in the panel; `Create invite link` stays secondary. The 1280 flow frame is balanced (roster left, invite right). FIX 2 only. At 1280 `Disable` sits slightly below `Copy link` baseline (items-start row), harmless.
- **match-day-phone (C2):** Before: "Read-only on the phone..." shown with no match and no action. After: caption gone, "No upcoming match" with chalk "Add the next match", 44 px at 390. Reading flow: the action is below welcome, a 7-step checklist and its heading, so it sits about 1,350 px down at 390 and 1,285 px at 1280 for a new account; that is the guidance design (welcome first) and not a regression, but a returning user with the checklist hidden gets it in the first screen. PASS.
- **draft / game-plan / tree (C4, C5):** Recipe followed to the letter: M141 `SaveStatus` copy "Could not save: the server did not answer", Retry, once-only toast "Your changes are not saved. Keep this tab open and use Retry." in `role=alert` danger style with a glyph and words (A4 satisfied, not hue-only). Success state "Saved"/"Draft saved" present. Toast in the bottom-left covers the board for a few seconds at 1280; it dismisses, accepted. The failed field keeps focus ring (chalk) so the user sees where the edit is. Tree FIX 1 only.
- **register (C6):** Before had no consent; after: unticked native checkbox, Jasper's label verbatim ("I am 16 or older and accept the Terms and have read the Privacy page"), links underlined in the app link style, sits between password hint and submit, same on both widths. The label is the 44 px target (`touch`); the 16 px box is small but the whole row is clickable. The "16" renders in a numeral style slightly different from the surrounding text (tabular figures); not worth a change. The header shows a "Request access" button on an invitation page (pre-existing, outside this patch).
- **login (C7/C8):** Hint sits under the Password field as the field hint, then a link; reads clearly, one chalk action. FIX 4. The two reset lines say partly the same thing; acceptable (hint for the case "no code yet", link for "have a code").
- **login-next (C10):** Pixel-identical to login, which is correct (no banner, return is silent). Verification that `?next=` works is the e2e's job, not a frame's.
- **auth-reset:** Four fields, example code under Reset code, hint "Use 10 or more characters." Focus ring on the first field in the flow frame (autofocus) is fine. One chalk action. Clean.

## Rubric harness (`rubric-check.md`, run 39000854398, sha 363613a): which rows are real

Pages covered: `/auth/login`, `/auth/login?next=%2Fdraft`, `/auth/register?invite=...`, `/auth/reset`, phone and desktop. Signed-in surfaces are not covered (known limitation).

| Row | Harness says | Real for these surfaces? |
|---|---|---|
| A1, A2, A3 | PASS | Real, and agrees with the frames (contrast 7:1; inline links are not measured, hence FIX 4) |
| I7, V1, V4 | PASS | Real |
| I1 No Settings path | FIX | GUESS: a signed-out public page has no Settings; not applicable |
| I2 progress indicators | FIX | GUESS: forms with one submit; not applicable |
| I6 notification policy | FIX | GUESS: not applicable |
| I8 undo affordance | FIX | GUESS: nothing destructive on these pages; not applicable |
| V2 off-scale spacing `[data-feedback-...` | FIX | Not a page defect: the offender is the in-app bug-report widget (`data-feedback-*` attribute), not the form. Ignore for these surfaces; if cared about, one sidecar fix in the widget, not in this patch |
| V3 CPL: login "p: 87 chars; p: 279 chars" | FIX | Mostly GUESS. It counts characters in the whole paragraph, not per rendered line. The 87-char paragraph is "No account yet? Registration needs an invite link..." and renders at about 50 characters per line; the 279-char one is the footer Riot disclaimer. Real part: at 1280 the footer disclaimer runs about 95 to 100 characters per line (`login-1280-flow.png`), over the 75 limit. Optional fix: footer disclaimer `<p>` in `src/components/footer.rs` gets `max-w-[65ch]`; low value, it is 12 px legal text on every page, and not a regression of this patch |
| V3 register/reset: "p: 24 chars; #register-password...; #reset-code-hint: 32 chars" | FIX | GUESS: hints of 24 and 32 characters are far below any CPL limit; this is the same heuristic misfiring (probably a minimum, "too short"). Ignore |
| Vo1-4, T1, T4 | PASS | GUESS rows (not implemented / heuristics). T1 to T4 are trust checks, not typography; typography is V2/V3 as already recorded |

No rubric row, real or guessed, found the three real defects above (the 455 px sideways scroll, the 400 px dashboard, the 393 px resets page), because those pages are signed-in and the harness only sees public pages. Add `scrollWidth <= viewport` to `frames-m148.spec.ts` for every surface at 390; it would have caught all three.

## Staleness flags

- STALE: .planning/missions/M148-team-companion-first-patch-r2/publish-round-r1/rubric-check.md "limitation" line vs the frames: the harness checks four public pages only; the signed-in surfaces of this patch (C1, C2, C4, C5, C9) rest on this frame critique alone.
- CONTRADICTION: wiki/meta/missions/M137-team-companion-first-patch.md (T1 to T4 named as typography) vs wiki/assets/ux-rubric/rubric.json (trust checks). Still open from item 0.

## Summary for the caller

PASS: match-day-phone, draft, game-plan, tree-drafter, register, login, login-next, auth-reset, draft-save-failed, game-plan-save-failed (re-capture the two failed 390 frames without the sticky-nav artefact).
FIX: tree-save-failed (real, `layout.rs:187`), dashboard-invites (`layout.rs:134`), admin-resets (`callout.rs:126`), login link height (`login.rs:134`, optional).
PENDING: profile-password, login-hint after frames (not in the worktree at HEAD c3b38b8).
