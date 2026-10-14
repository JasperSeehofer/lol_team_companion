# M137 item 0: atelier critique (2026-10-14)

Mode: critique, from stills and code. Frozen tokens: `input.css` `@theme`. References: `docs/DESIGN.md` ("Survey"), principles 1, 7, 11, 12 and sections 3.3 to 3.7, 6.9, 6.11. Not manyfold tokens.

## What was and was not run

- Sighted: `d1280-dark-landing`, `p390-dark-login`, `p390-dark-register` (this is the closed-beta page, the register form needs an invite and was NOT seen), plus the audit's probe numbers (0 text under 4.5:1, 0 under 12px, no overflow at 390).
- NOT run (host overloaded): Playwright 393x852 / 1280x800 captures of the signed-in surfaces, the rubric script, any contrast-from-pixels. Every signed-in verdict below is CODE-ONLY. Re-run the capture on the r1 build before merge: `node wiki/assets/ux-rubric/rubric-check.mjs --app-dir <repo>/app <render-path>` at 390 and 1280, and recompute contrast of every new `text-danger` / `text-muted` string from rendered pixels (A2).
- Nothing here proposes motion, so no frame strip is needed. Reduced-motion run: only the "Saved/Copied" label swaps, which are instant by construction.
- Rubric note: in `rubric.json` I1 to I8 are agent-UX checks and T1 to T4 are trust checks (confidence, change notice), not typography. They mostly do not apply to these surfaces. Verdicts rest on A1 (target 44), A2 (contrast), A3 (reduced motion), A4 (no hue-only state, no drag-only), I3 (disclosure depth), I5 (confirms rare), I7 (no inputs in list rows), V2 (4/8 spacing), V4 (motion band) and the DESIGN honesty rule (principle 12). Typography is V-group, not T.

## Verdict table

| Surface | Verdict | Rests on | One line |
|---|---|---|---|
| Public pages (landing, login, closed beta) | PASS | A2, V2, DESIGN 1.1/1.3 | On-spec, calm, one chalk action; the identity is in the type alone (see identity-proposal) |
| Invite rows (dashboard) | FIX | A1, A4, I7, 1.12 | Bare code, no URL, no Copy/Share; Disable is 28px |
| Phone Match Day, no match | FIX | A1, 1.12 | Central action `hidden md:block`: dead end on the likeliest first device; caption lies |
| Phone Match Day, with match | PASS (scoped) | DESIGN 6.9 | Read-only caption is a deliberate owner decision; see Q1 |
| Draft save state | FIX | A4, 1.12 | No `Err` branch; header stays "Unsaved changes"; Save is 32px |
| Game plan save state | FIX | same | `update_plan(..).await.is_ok()` has no else |
| Tree save state (+ Branch/Improvise) | FIX | same | `let _ = save_node(..)` then success toast |
| Load error vs empty state | FIX | 1.12 | `unwrap_or_default()` paints failure as "No invite links yet" |
| Register checkbox (item 7) | FIX (new) | A1, A4 | Primitive lacks `name`/`required`/link label; 16px box |
| Login "Forgot it?" (item 5) | FIX | 1.12 | Copy points at someone who cannot reset (see card) |
| Password change on /profile, admin reset, /auth/reset | FIX (new) | A1, A4, I5 | Recipe below |
| Page titles (item 8) | FIX | WCAG 2.4.2 | One `<Title text="Team Companion" />` at app.rs:80 for every route |

## Shared primitives to touch first (so the cards stay short)

1. **`ButtonSize::Touch`** in `src/components/ui/button.rs`: `"h-11 px-4 text-body md:h-8 md:px-3 md:text-label"`. 44px below `md`, the normal Md from `md` up. Doc line: "44px on phones, 32px from md: row actions and the one action a phone user needs." Add to the gallery's Button sizes and its `SECTIONS` entry (a test checks it). Existing sizes stay as they are, so nothing else shifts. There is no `class` prop on `Button`; do not try `attr:class` for height (it is not guaranteed to merge with the primitive's own `class`).
2. **`SaveStatus`** (new, `src/components/ui/status.rs`, exported in `mod.rs`, gallery section): see card C4.
3. **`LoadFailed`** (new, next to `EmptyState` in `empty.rs`): see card C5.
4. **`Checkbox`** extra optional props: `name: Option<&'static str>`, `required: bool`, `touch: bool`, `label_extra: Option<ViewFn>` (rendered after the label text inside the same `<label>`, for links). `touch` adds `max-md:min-h-11 max-md:items-center` to the label. A1 is met by the label being the target, not the 16px box. Keep the `A11y:` line current.

## Recipe cards

### C1. Invite rows (item 2)

Placement: `InvitePlayers`, `src/pages/team/dashboard.rs` ~957 to 994. Extract `InviteLinkActions(code: String)` and use it in both the transient "Send this link" block and every open row.

Row layout: `<li class="flex flex-col gap-3 border-t border-rule py-3 md:flex-row md:items-center md:justify-between" data-team-invite=code>` (keep `data-team-invite`).
- Line 1: status line as today (`{status} · {uses} of {max} used · expires {date} · {Position}`, `figures text-caption text-muted`). The status word stays: status is never hue-only (A4).
- Line 2 (open rows only): the full URL from `invite_link(&code)` in `<code class="block break-all font-mono text-label text-secondary select-all" data-testid="team-invite-url">`. Not truncated, so it can be read aloud or selected by hand as a fallback. Used, expired and disabled rows show the bare code only and no actions.
- Actions (open rows): `Copy link` (Secondary, `ButtonSize::Touch`), `Share` (Secondary, Touch; render only when `navigator.share` exists, so desktop shows none), `Disable` (Quiet, Touch). On phones the three sit in a `flex flex-wrap gap-2` row under the URL; gap 8px (A1 spacing).

States for Copy (one `RwSignal<CopyState>` per row: Idle, Copied, Failed):
- idle: label "Copy link".
- success: only after `JsFuture::from(clipboard.write_text(&url))` resolves `Ok`. Label becomes "Copied" for 2 s (timeout, then idle). Toast not needed; add a visually hidden `<span class="sr-only" role="status">Link copied</span>` that is present only in Copied state, so a screen reader hears it (A4). Do NOT toast before the promise resolves (this is the share.rs:255 bug; apply the same fix there in the same change).
- failure (rejected promise, no `clipboard`, insecure context): label stays "Copy link"; show `<p role="alert" class="text-label text-danger">Could not copy. The link is selected above: copy it by hand.</p>` under the row and select the URL text (`Range` over the `<code>`). No success toast ever.
- share: call `navigator.share({title: "Join {team} on Team Companion", url})`. A user cancelling (AbortError) is silent. Any other rejection falls through to the Copy failure message.
- busy: not needed; both are instant. Disabled: none (a disabled Copy with no reason is worse than none); closed rows simply have no actions.
- Keyboard: Tab order URL (not a stop), Copy, Share, Disable. Focus stays on the button after activation (do not move it).
- Reduced motion: nothing animates.
- The new link block after "Create invite link": keep `data-testid="team-invite-link"`, add the same actions, and give the block `role="status"` so the minted link is announced once.

Coach wording (F07): pick the smaller change, drop "or coach" from `closed_beta.rs:61` copy and the landing copy ("from their captain or coach" becomes "from their team captain"). Coaches do not get invite rights in R1. This is a copy fix, not a design call; the permission question stays an ask card.

### C2. Phone Match Day, no match (item 3)

`src/pages/home.rs`: remove `hidden md:block` around the `EmptyState` action at ~606 (render the Button directly) and make the caption conditional: `{has_match.then(|| view! { <p class="-mt-3 mb-6 text-caption text-muted md:hidden" data-testid="phone-read-only">...</p> })}`. When there is no match the page shows no caption at all.
- Button: `variant=Primary size=ButtonSize::Touch`, label "Add the next match" (unchanged), `data-testid="add-next-match"`. It is the page's one chalk action (the welcome and checklist already render quiet there). Full width below `md` is not necessary: the EmptyState action slot is left-aligned; keep natural width, 44px tall.
- The `MatchEditor` stays a `Dialog` Md: it is `w-[min(32rem,calc(100vw-2rem))]` with `max-h-[calc(100dvh-2rem)]`, so it fits 393px and scrolls inside. Do not make it a `Sheet` (that is the nav menu's pattern). Its footer buttons use `ButtonSize::Touch`; fields stay the existing 40px Lg-height controls (verify with a bounding-box check; fields under 44px at 393px are acceptable only if their full label row is tappable, otherwise add `min-h-11` on the control wrapper).
- Empty opponents inside the editor (home.rs ~979 "add the opponent on the Opponents page first"): make that sentence a `Button href="/opponents"` Touch "Add an opponent". It is a second dead end on the same flow. Check that /opponents is usable at 393px; if its add form is not, say so in REPORT for Harden instead of fixing.
- States: idle (button), dialog open, saving (`loading` on the Save button), failure (inline `Callout` Danger inside the dialog, never only a toast), success (dialog closes, Match Day re-renders with the match; no toast needed).

With a match: PASS. Keep `Edit match` desktop-only per DESIGN 6.9 for this round.

### C3. Page headers on phone

`PageHeader` actions wrap (`flex-wrap`), so the Save/Retry buttons using `Touch` reflow under the title without code change. Check the draft header at 393px: eyebrow, title, meta (opponent, side, save status), then actions; the status line may wrap to two lines, that is fine.

### C4. Save status and failure (item 4), one component for draft, plan, tree

New `SaveStatus` plus a shared state enum `SaveState { New, Dirty, Saving, Saved, Failed { reason: String, attempts: u8, gave_up: bool } }` (move `SaveState` out of draft.rs; game_plan and tree currently use `&str`).

Placement: where the span sits today (PageHeader `meta` in draft.rs ~1671 and game_plan.rs ~896; tree: the Node section `SectionHeader` actions at tree_drafter.rs ~988). Keep the `data-testid`s (`draft-save-state`, `plan-save-state`, `node-save-state`).

Markup: `<span role="status" aria-live="polite" aria-atomic="true" class="text-label ...">`. The element exists from first render (a live region added later is not announced) and only its text changes.

Copy and tone (plain, no jargon):
- New: "Not saved yet" (text-muted)
- Dirty: "Unsaved changes" (text-muted)
- Saving: "Saving…" (text-muted, no spinner, nothing animates)
- Saved: "Saved" (text-muted)
- Failed, retrying: "Not saved: {reason}. Retrying…" (`text-danger`; the words carry the state, colour is secondary, A4)
- Failed, gave up (after 3 attempts): "Not saved: {reason}." followed by `Button Quiet Touch` "Try again" in the same header cell.
- reason mapping through `models::utils::error_text`, never `e.to_string()`: network failure or 502/503 becomes "the app could not be reached"; a server message such as "Create or join your team first" is shown as written and is NOT retried (retry cannot help): state goes straight to gave_up and an inline `Callout` Warning with a link to /team/roster sits above the editor.

Autosave retry: 2 s debounce as today; on `Err`, retry after 3 s, 8 s, 20 s (timeouts, no animation), reset on any success or new edit. After the third failure, `Failed{gave_up:true}` and push ONE `Toast::error("Your changes are not saved. Keep this tab open and try again.")` (errors persist until dismissed per `toast_duration`). Never one toast per retry.

Success toasts only on `Ok`:
- draft.rs ~1351: add the `else` branch (set Failed). Manual `do_save` already handles `Err`; its text `format!("Could not save the draft: {e}")` goes through `error_text`.
- tree_drafter.rs ~848, ~945, ~1156 (Branch, Improvise): replace `let _ = save_node(..)` with a match. On `Err`: do not add the node to the local tree (or roll it back), `Toast::error("Could not save the branch: {reason}")`; the success toast moves into the `Ok` arm.
- game_plan.rs ~761: add the `Err` branch; `save_state` becomes the enum.
- The header Save button (`Primary`, `Touch`) stays enabled in Failed (pressing it is the retry); `loading` only while Saving.

Unsaved-work guard: `use_unsaved_guard(dirty: Signal<bool>)` in `pages/navigate.rs` (hydrate-only). Registers `beforeunload` on `window` while `dirty` is true: `ev.prevent_default()` and set `returnValue = ""`. Dirty means Dirty, Saving, Failed, or New with any content (draft: a pick, ban or note; plan: any text). Remove the listener in `on_cleanup` and when dirty turns false (the `cb.forget()` leak in nav.rs, F28, is the pattern NOT to copy: keep the `Closure` in a `StoredValue`). The browser's own wording is shown; do not try to customise it. In-app link navigation is not covered by `beforeunload`; list "New plan / New review discard without Undo" (F04) in Harden.

Verification the builder runs: Playwright route-intercept `POST /api/...` returning 503 for the save server fn; assert `[data-testid=draft-save-state]` contains "Not saved" within 3 s, no "Draft saved" toast, then restore the route and assert "Saved". Add the same for the tree Branch case and one `beforeunload` dialog assertion (`page.on('dialog')`).

### C5. Load error vs empty state (item 4, "introduce one helper")

New `LoadFailed(what: &'static str, #[prop(optional)] retry: Option<Callback<()>>)` in `components/ui/empty.rs`: a `Callout` Danger with title "Couldn't load {what}" and body "Reload in a moment. If it keeps happening, tell us with the bug button." plus, when `retry` is set, `Button Secondary Touch` "Try again" in the Callout `action`. It is role="alert" through Callout. `EmptyState` stays for true emptiness only.

Use: in every touched page, replace `r.unwrap_or_default()` with `match r { Ok(v) => ..., Err(e) => LoadFailed(what, retry=resource.refetch()) }` (log `e` through the existing error path). Apply to: dashboard.rs:965 (invites: "invite links"), the draft/plan/tree list resources the item touches. Add a grep gate to `scripts/lint_design.sh` or a unit test: no `unwrap_or_default()` on a `Resource<Result<..>>` inside `src/pages/**` (the Harden list, F05, has the other 20 sites; the gate should have an allow-list so item 4 can land without them).

### C6. Register checkbox (item 7)

`src/pages/auth/register.rs` ~315: between the password `Field` and the submit `Button`, with the same `gap-5`.
- Component: extended `Checkbox` (card shared-primitive 4), `id="register-terms"`, `name="accept_terms"`, `required=true`, `touch=true`; unchecked by default (never pre-ticked).
- Copy (English; mark with `// TODO(legal-wording)` for Jasper or his adviser): label text "I am 16 or older, I accept the " + link `Terms` (`/legal/terms`) + " and I have read the " + link `Privacy page` (`/legal/datenschutz`) + "." Links `target="_blank" rel="noopener"`, class as the existing inline links (`font-medium text-primary underline decoration-outline underline-offset-4 hover:decoration-accent`) and an `sr-only` " (opens in a new tab)". Opening in a new tab keeps the half-filled form.
- Client: native `required` (message appears before submit and before hydration). Server (item 7) rejects a missing value with the same wording in the existing Danger `Callout`: "Please confirm you are 16 or older and accept the Terms and the Privacy page."
- States: idle (unchecked), checked, error (Callout above; also `aria-invalid` on the input), disabled (none; the submit button is NOT disabled while unchecked, it explains itself instead).
- Focus: when the server rejects only this, move focus to the checkbox (`focus_id("register-terms")`). Tab order: username, email, password, checkbox, links, Create account.
- Touch: the `<label>` is `min-h-11` below `md`, so the whole line is the 44px target (A1); the links inside keep their own native target.

### C7. Login "Forgot it?" line (item 5)

`login.rs` ~104: use the existing `Field` `hint` prop on the password field so the text is tied to the input by `aria-describedby` and sits under it, left aligned: `hint="Forgot it? Ask your team admin."` is the contract copy. Problem: in this app "admin" means the operator (`is_admin`), a team leader cannot reset anything, and the line sends a player to a captain who has no power. Recommended wording: "Forgot it? Ask the Team Companion admin for a reset code." with the second sentence linking, when item 5 lands, to `/auth/reset` as `Have a reset code? Set a new password.` (`text-label`, same link style, Touch height via `py-2.5 inline-block`). Needs Jasper's pick between the two wordings (Q2). When the login error Callout is shown the hint stays visible.

### C8. Password change on /profile (item 5)

`src/pages/profile.rs`, new section directly under "Account" (~461): `SectionHeader title="Password" id="password-h"`, then a `Surface` with three `Field`+`TextField kind=Password`: "Current password" (`autocomplete="current-password"`), "New password" (`new-password`, hint "Use 10 or more characters."), "Repeat new password" (`new-password`). One `Button` Secondary `Touch` "Change password" (not Primary: the page keeps no chalk action, and this is not the page's next step).
- Validation on submit, not by disabling: wrong current password: error on that field "That is not your current password."; mismatch: error on the repeat field "The two new passwords do not match."; too short: the hint turns into an error. Focus goes to the first invalid field. Rate limit: field-less `Callout` Warning "Too many attempts. Try again in a few minutes."
- busy: `loading` (button disabled, `aria-busy`); success: an inline `Callout` Success (role=status) "Password changed. Your other sessions were signed out." and the three fields cleared; no toast (toasts auto-dismiss and a password change deserves a persistent line).
- No ConfirmDialog (I5: reversible, and the user typed the current password).

### C9. Admin reset and /auth/reset (item 5)

- Admin action: new row group on `/admin` (`AdminHubPage` lists every admin area with its status, so add "Password resets" there; per CLAUDE.md "New page" also `PAGES`, `MENU_GROUPS`, a smoke entry). Page `/admin/resets`: `PageHeader` eyebrow "Admin", title "Password resets"; one `Field` "Email of the account" + `Button` Primary `Touch` "Create reset code" (the page's one chalk action).
- Result (shown once): `Callout` Info with title "Reset code for {email}" and body: code in `<code class="block font-mono text-title figures select-all">`, then "Valid for 15 minutes, works once. Send it to the player in a private message. It is not shown again." plus the expiry as `LocalTime`, plus `Copy code` (Secondary, Touch, same real-clipboard-result rule as C1). Navigating away drops it.
- Unknown email: admin-only page, so say "No account with that email." (no enumeration risk to the public). No ConfirmDialog (minting changes nothing until redeemed).
- `/auth/reset` (public, rate limited): `PageHeader` title "Set a new password", eyebrow "Closed beta"; fields "Reset code" (mono, `autocomplete="one-time-code"`), "New password" (hint "Use 10 or more characters."), "Repeat new password"; `Button` Primary `Lg` full width "Set new password" like login. Any failure shows ONE generic Danger Callout "That code did not work. It may have expired or been used. Ask the admin for a new one." (same text for wrong, used and expired: no oracle). Success: server redirect to `/auth/login` with a success `Callout` "Password changed. Sign in with the new one."
- Focus: on load, focus the code field. Keyboard: Enter submits.

### C10. Page titles (item 8)

Put the title in `PageHeader` itself so it can never drift from the `<h1>`: render `<Title text=...>` from `leptos_meta` with `"{title} · {eyebrow} · Team Companion"` (drop the eyebrow when absent or equal to the title, e.g. "Draft · Draft · ..." must collapse to "Draft · Team Companion"). Pages without a `PageHeader` (landing, 404, closed beta, share) set their own: "Team Companion", "Page not found · Team Companion", "Closed beta · Team Companion", "Shared game plan · Team Companion" (never the plan's name: the token page is public). Keep `app.rs:80` as the fallback.
- Risk: `Title` inside a `Suspense` may land late in streamed SSR. Verification without JS: `curl -s <page> | grep -o '<title>[^<]*'` for 6 routes (login, register, /draft, /game-plan, /team/dashboard, /bugs); and one Playwright `await expect(page).toHaveTitle(/Draft/)` after hydration.
- Return-to-page: carry `?next=` through login as a path only: must start with `/`, not `//`, not `/\`, no scheme or host, not under `/auth/`; anything else becomes `/`. Unit-test the validator with `https://evil`, `//evil`, `/\evil`, `/auth/login`, `javascript:`.

## Cross-cutting checks the builder runs (A1 to A4)

- A1: every new button measured with `getBoundingClientRect().height >= 44` at 393px (invite actions, Add the next match, Save/Retry, Change password, Create reset code, checkbox label).
- A2: new strings use only `text-primary`, `text-secondary`, `text-muted` and `text-danger` on `bg-page` / `bg-ink-1`; no new token pairs, so the DESIGN 3.2 test covers them, but recompute from pixels on the final render.
- A3: none of the cards animates; `Copied` revert and retry timers are timeouts, not transitions.
- A4: no state is colour-only (every one has words); no drag; live regions exist before they change.
- `scripts/lint_design.sh` must stay at zero: no `rounded-full`, no palette classes, no sub-12px text, no emoji in these cards' strings.

## Open questions for the implementer's card (not mine to decide)

- Q1. Should `Edit match` also show on phones now that the editor is phone-usable? Recommend yes (same Dialog, one `md:hidden` wrapper), but it reverses a DESIGN 6.9 owner decision, so it needs a DESIGN.md sentence and Jasper's yes. Not in R1.
- Q2. Login line wording: contract copy "ask your team admin" vs "ask the Team Companion admin for a reset code".

## Staleness flags

- CONTRADICTION: wiki/meta/missions/M137-team-companion-first-patch.md ("UI definition of done" names T1 to T4 as typography) vs wiki/assets/ux-rubric/rubric.json (T1 to T4 are trust/confidence checks; typography is under V). The gate as written cannot be passed or failed for type; read it as V2/V3 plus the app's own lint.
- CONTRADICTION: docs/DESIGN.md 6.9 ("phones are read-only for match editing") vs audit F02 and this critique: the no-match case was an oversight, the with-match case is still a decision.
