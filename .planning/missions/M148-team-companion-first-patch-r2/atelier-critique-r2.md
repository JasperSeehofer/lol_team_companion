# M148 item 0: atelier re-read of the M137 recipe cards (2026-10-17)

Mode: recipe re-read, CODE-ONLY against tip 9b7da51. No render was captured (a re-read of cards, not a critique of a build). Frozen tokens: `input.css` `@theme` and `docs/DESIGN.md` ("Survey") only. Cards: `.planning/missions/M137-team-companion-first-patch/atelier-critique.md` (C1, C2, C4, C5, C6, C9, C10).
Process note: this run had no subagent tool, so the reads and this one file write were done by the atelier agent itself. No app file was touched, nothing committed.

Rubric correction (carry forward): in `rubric.json` T1 to T4 are trust checks (confidence, change notice). Typography is V2/V3. The M137 mission page's "T1 to T4 = typography" gate cannot be passed or failed for type; read it as V2/V3 plus `scripts/lint_design.sh`. Verdict checks used below: A1 (44px target), A2 (contrast), A3 (reduced motion), A4 (no hue-only state), I3, I5, I7, V2, V4, principle 12.

## What M141 and M137 R1 already changed (read before building)

- `src/components/ui/status.rs` now exists: `SaveState {Idle, NotSaved, Unsaved, Saving, Saved, Failed(String)}` (l.15-31), `SaveTracker` (`saving()`, `saved()` with a 4 s fade to Idle, `failed(what)`; l.69-114), `use_save_state()` (l.117), `SaveStatus` (always-mounted `role="status"` span, `data-save-state`, optional `on_retry` giving a `Retry` button; l.131-170), `LoadFailed {what, on_retry}` (danger `Callout`, "{what} did not load", "Try again"; l.172-189). Only `champion_pool.rs` uses it (l.306, 372, 515). draft, game_plan, tree_drafter still hand-roll states.
- `ButtonSize::Touch` exists (`button.rs:54,63`: `h-11 px-4 text-body md:h-8 md:px-3 md:text-label`). Already used by `profile.rs:581`, `admin/resets.rs:113`, `admin/invites.rs:201`, `match_detail.rs`. Card shared-primitive 1 is DONE.
- Login hint `FORGOT_PASSWORD_HINT` and `/auth/reset` link exist (`login.rs:21,118,121`); `/admin/resets` and `/auth/reset` routes exist (`app.rs:~131`). C7/C8 are landed; C9's page exists but has no Copy (see C9).
- NOT yet done: invite URL rows and Copy/Share (C1), phone "Add the next match" (C2), draft/plan/tree failure states (C4), checkbox on register (C6), titles and `?next=` (C10).
- Gap in the M141 helper that the cards need: `SaveStatus`'s Retry and `LoadFailed`'s Try again are `ButtonSize::Sm` (28px), which fails A1 on phones. Amend `status.rs` (two lines) to `ButtonSize::Touch`; the existing champion_pool caller inherits it (44px below `md`, 28 to 32px from `md`, no desktop change).

## Verdict table

| Card | Verdict | One line |
|---|---|---|
| C1 invite Copy | AMEND | Coaches may invite; clipboard-promise rule; `navigator.share`; full URL after reload (already derivable: code is stored); Touch |
| C2 phone Add the next match | CONFIRM (+1 sub-amend) | Still `hidden md:block` at home.rs:605 and caption unconditional at :588; opponents sentence at :979 still a dead end |
| C4 autosave failure | AMEND | Reuse `ui::status` exactly; drop the card's new enum/`SaveStatus`; copy becomes M141's "Could not save: ..."; retry is manual Retry plus the next edit |
| C5 load errors | AMEND | `LoadFailed` already exists in `status.rs` (not `empty.rs`); only apply it |
| C6 register checkbox | AMEND | Jasper's label verbatim, no TODO(legal-wording); Checkbox extension still needed |
| C9 reset-code Copy | AMEND | Page exists (resets.rs:116-131); add Copy to the minted Callout, shared copy helper with C1 |
| C10 titles + `?next=` | AMEND | Same-origin path validator per ruling; mechanism through one helper because 12 pages hard-code `/auth/login` |

## C1. Invite rows (AMEND)

Source now: `InvitePlayers`, `src/pages/team/dashboard.rs:886-1000`; row `li` at :979-993 (`ButtonSize::Sm` Disable at :988); transient link block :960-964; `invite_link()` :1003; the list is built from `Invite.code`, so the full URL is derivable after reload with no schema change. Server: `list_team_invites` (:66), `mint_team_invite` (:90, `require_leader` :99), `disable_team_invite` (:140, `require_leader` :145); the panel is gated by `is_leader.then(...)` at :798 (`is_leader = team.created_by == me`, :584). The copy bug to avoid also sits at `share.rs:252-258` (`let _ = write_text(..); toast.success("Link copied")`).

Amended recipe:
1. **Rights.** Add `Ctx::require_leader_or_coach()` next to `require_leader` (`authz.rs:405`): a team member who is the leader or whose roster place is coach. Use it in `list_team_invites`, `mint_team_invite`, `disable_team_invite` only (not the placement/team-edit functions). Panel gate at :798 becomes `can_invite = is_leader || my_place == Coach`; join requests stay leader-only. Cross-tenant and "plain member refused" cases in `tests/integration/security.rs`. Keep `Bucket::TeamInvite` per user. Update the copy "You lead this team" is untouched; the invite intro (:944) says "until you place them" and stays true for a coach only if coaches may place: say "until the leader places them" when `!is_leader`. Landing/closed-beta wording "from their captain or coach" (`closed_beta.rs:61`) is then correct; no copy change.
2. **Row** (replaces :979): `<li class="flex flex-col gap-3 border-t border-rule py-3 md:flex-row md:items-start md:justify-between" data-team-invite=code>`.
   - Status line unchanged (`figures text-caption text-muted`; status word always shown, A4).
   - Open rows: URL line `<code class="block break-all select-all font-mono text-label text-secondary" data-testid="team-invite-url">{invite_link(&code)}</code>`, never truncated. Hydration: `invite_link` returns the bare path on the server and the absolute URL after hydrate; render it inside the same Suspense so it is the absolute form after reload (it is, because the list renders client-side after hydrate; verify that the SSR text does not mismatch: use the path in SSR and let the hydrate re-render, or compute from `window.location` in an Effect).
   - Used, expired, disabled rows: bare code, no actions.
   - Actions (open rows) in `flex flex-wrap gap-2`: `Copy link` (Secondary, `ButtonSize::Touch`), `Share` (Secondary, Touch, only when `navigator.share` exists, so desktop shows none), `Disable` (Quiet, Touch). At 390: URL wraps (`break-all`), buttons 44px tall, 8px gap, wrap onto two lines if needed. At 1280: row is one line, actions right, 32px.
3. **Copy state** (one `RwSignal<CopyState>` per row; shared helper `copy_to_clipboard(text) -> Result<(), ()>` in `src/pages/navigate.rs` or `components/ui/a11y.rs`, hydrate-only, reused by C9 and by the `share.rs:252` fix):
   - Idle "Copy link". Success ONLY after `JsFuture::from(clipboard.write_text(&url))` resolves Ok: label "Copied" for 2 s, plus `<span class="sr-only" role="status">Link copied</span>` present only then. No toast before resolve; no toast needed at all.
   - Failure (rejected promise, no `clipboard`, insecure context): label stays "Copy link"; under the row `<p role="alert" class="text-label text-danger">Could not copy. The link is selected above: copy it by hand.</p>` and select the `<code>` text (`Selection`/`Range`; the `select-all` class is the fallback). No success state ever.
   - Share: `navigator.share({title: "Join {team} on Team Companion", url})`. Cancel (AbortError) silent; any other rejection falls through to the Copy failure message. Needs web-sys features `ShareData` plus `Navigator::share` (check `Cargo.toml:55`; `Clipboard` is there, `ShareData` is not) and the `web_sys_unstable_apis` cfg for `clipboard()` if not already set. Feature-detect with `Reflect::has(navigator, "share")`.
   - Tab order: Copy, Share, Disable. Focus stays on the pressed button. Reduced motion: the 2 s label swap is a timeout, nothing animates.
4. **Fresh link block** (:960): keep `data-testid="team-invite-link"`, `role="status"`, same `InviteLinkActions` component, so the minted link is announced once. Replace the "Invite link ready" success toast (:913) by nothing (the block is the confirmation) or keep it as is; either is fine, but do not add a second announcement.
5. Disable keeps `Toast::success` on Ok and `Toast::error` on Err (:921-928, already correct). No ConfirmDialog (I5: reversible by minting another).

## C2. Phone Match Day, no match (CONFIRM)

Still as the card found it: `src/pages/home.rs:605` wraps the only action in `hidden md:block`; :588 prints "Read-only on the phone. Edit the match on a larger screen." unconditionally (`md:hidden`), including with no match; :581 `Edit match` stays `hidden md:block`; editor `Dialog` Md at :955-996 (`w-[min(32rem,calc(100vw-2rem))]`, `max-h-[calc(100dvh-2rem)]`, so it fits 390 and scrolls inside).

Recipe unchanged, with these exact points:
1. :600-610: drop the `hidden md:block` div; `<Button variant=Primary size=ButtonSize::Touch ... attr:data-testid="add-next-match">"Add the next match"</Button>`; natural width, 44px at 390, 32px at 1280. It is the page's one chalk action (welcome and checklist already render Quiet, :592-596).
2. :588: render the caption only when `has_match`: `{has_match.then(|| view! { <p class="-mt-3 mb-6 text-caption text-muted md:hidden" data-testid="phone-read-only">...</p> })}`. DESIGN 6.9's "phones are read-only for match editing" stays for an existing match (Q1 of M137 is not reopened).
3. Editor footer (:962-966): Cancel and Save match become `ButtonSize::Touch` (Remove match stays Sm, destructive and rare; it is behind a ConfirmDialog). Inside the dialog `Field` controls are the standard control height; confirm at 390 with a bounding-box check that the date, time and select rows are at least 44px or their label+control wrapper is the tap area; if not, `min-h-11` on the wrapper below `md`.
4. Sub-amend: :979 hint "No dossiers yet: add the opponent on the Opponents page first." is a dead end on the same flow. When `no_dossiers`, render `Button href="/opponents" variant=Secondary size=Touch` "Add an opponent" under the select instead of the hint sentence (hint text can stay as `aria-describedby`). The dialog does not close; opening the link leaves the half-filled form: acceptable since a match without opponent is allowed ("No opponent yet" option, :896). Check `/opponents` add form at 390; report, do not fix.
5. States: idle, dialog open, saving (`loading` on Save, exists), failure: today an inline `<p role="alert" class="text-label text-danger">` (:988) which is correct and persistent, keep it (the card's Callout is not needed); success closes the dialog.

## C4. Autosave failure and visibility (AMEND: reuse M141, nothing new)

Binding: reuse `ui::status` (`SaveState`, `SaveTracker`, `use_save_state`, `SaveStatus`). Do NOT add `SaveStatus` to `status.rs` again, do not add `Failed{reason,attempts,gave_up}`, do not move `SaveState` out of draft.rs into a second type (the local `enum SaveState` at `draft.rs:1169-1177` is deleted and replaced by the shared one; its variants map New to NotSaved, Dirty to Unsaved).

Current failure sites (each is a silent drop or a success claim on failure):
- `draft.rs:1353` autosave `if persist(..).await.is_ok() {..}` no else; header state at :1673-1678; manual save Err arm :1449-1454 sets Dirty and prints raw `{e}`.
- `game_plan.rs:761` same pattern; `save_state: RwSignal<&str>` (:668); manual Err arm :~842 raw `{e}`; header span :896-902.
- `tree_drafter.rs:858` autosave `is_ok()` no else; manual `save` :889-896 raw `{e}`; `let _ = save_node(..)` at :955 (Branch) and :1166 (Improvise) before success toasts; header span :997-999 (`ned.status` `&str`).

Amended recipe:
1. In each page: `let save = use_save_state();` (as `champion_pool.rs:306`). Header meta: `<SaveStatus attr:data-testid="draft-save-state" state=save.state on_retry=retry />` (`attr:` on a component lands on its root span; keep `draft-save-state`, `plan-save-state`, `node-save-state` for the e2e specs). Keep the live region mounted from first render (it is).
2. Copy is M141's, not the card's: "Not saved yet", "Unsaved changes", "Saving\u{2026}", "Saved", "Could not save: {what}". `what` is a short phrase through `models::utils::error_text` (never `e.to_string()` or `{e}`): for network/5xx failures pass "the server did not answer"; a server message such as "Create or join your team first" is shown as written. Failed is `text-danger` with the words carrying the state (A4).
3. Transitions: the editor effects keep calling `save.state.set(SaveState::NotSaved | Unsaved | Saved)` directly (documents keep "Saved" visible; the 4 s fade of `tracker.saved()` is for the pool's action-style saves). Use `save.saving()` before a request and `save.failed(what)` in every Err arm; success sets `Saved` (not `saved()`). Add the missing Err arms at draft.rs:1353, game_plan.rs:761, tree_drafter.rs:858.
4. Retry: no timers and no attempt counter. `on_retry` re-runs the manual save callback (`do_save` / `save`); the next edit re-arms the existing 2 s autosave. This is the champion_pool precedent (`retry_action`, :307-313). One `Toast::error` on the transition into Failed only (check `save.state` was not already Failed), text "Your changes are not saved. Keep this tab open and use Retry." No toast per attempt.
5. Branch/Improvise (`tree_drafter.rs:955`, `:1166`): `match save_node(..).await { Ok(()) => { nodes.refetch(); success toast } Err(e) => { error toast "Could not save the branch: {error_text}"; do not add the node to the local tree } }`. The success toast moves into the Ok arm. (If `add_branch` succeeded and `save_node` failed, call `nodes.refetch()` anyway so the empty branch is not hidden.)
6. Save buttons (`draft.rs:~1690` "Save draft", game plan `plan-save`, tree "Save node" `tree_drafter.rs:1000`) become `ButtonSize::Touch`; the Save button stays enabled while Failed.
7. Tokens: `SaveStatus` uses `text-muted`, `text-secondary` (Saved), `text-danger`, all in the frozen set. Recompute `text-danger` on `bg-page` from rendered pixels at the end (A2).
8. Unsaved-work guard (`beforeunload`) from the card stays, unchanged: `use_unsaved_guard(dirty: Signal<bool>)` in `pages/navigate.rs`, hydrate-only, closure in a `StoredValue` (do not copy the `cb.forget()` leak). Dirty means Unsaved, Saving or Failed, or NotSaved with content.
9. Verification: Playwright route-intercept the save server fn with 503; expect `[data-testid=draft-save-state]` to contain "Could not save" within 3 s, no "Draft saved" toast, `Retry` visible; unroute, click Retry, expect "Saved". Same for tree Branch. Existing specs that read these testids for "Saved" must not wait more than needed (editor Saved does not fade, so they are safe). Run `scripts/lint_design.sh`.

## C5. Load error versus empty (AMEND: component exists)

`LoadFailed` is already in `src/components/ui/status.rs:172-189` (not `empty.rs`), signature `LoadFailed(what, on_retry: Callback<()>)`, exported through `ui`, used at `champion_pool.rs:515`. The card's new component is dropped. Work left:
1. Apply it, with `on_retry=Callback::new(move |_| resource.refetch())`, in place of:
   - `team/dashboard.rs` invites list: `let (now, list) = r.unwrap_or_default();` (~:965) becomes `match r { Ok((now, list)) => ..., Err(_) => LoadFailed what="The invite links" }`. A non-leader/coach never reaches it.
   - `tree_drafter.rs:~440`: `Err(_) => Callout "The trees did not load."` is the exact hand-written callout the helper replaces; use `LoadFailed what="The trees"`.
   - Remaining `unwrap_or_default()` on list resources (`draft.rs:~2314` series, `game_plan.rs:~1422` templates, and the draft/plan list resources the item touches) are optional-detail lists, not the page's main content: leave them on the Harden list unless the page's own list is the sole content.
2. Amend the helper text once: Try again button `ButtonSize::Touch` (see M141 note); body "The server did not answer. Nothing you saved is lost." stays (plain, true).
3. Gate: grep test that no `unwrap_or_default()` follows a `Resource<Result<..>>` `.get()`/`.await` in a file listed in an allow-list that shrinks per mission; not required to land C5.

## C6. Register checkbox (AMEND)

Current: `src/pages/auth/register.rs:296-312` (`RegisterForm`), fields username/email/password then `Button` Lg full-width submit at :306. `Checkbox` (`toggle.rs:260-301`) still has only `label, checked, strike, disabled, on_change, id`: no `name`, `required`, touch, or link slot, and its box is 16px (`size-4`).

Amended recipe:
1. `Checkbox` gains optional props: `name: Option<&'static str>`, `required: bool`, `touch: bool` (label gets `max-md:min-h-11 max-md:items-center`), `label_extra: Option<ViewFn>` for the content after the text. Keep the `A11y:` line current (a test enforces it). Add a gallery sample (Checkbox with links, touch) and keep the `SECTIONS` entry intact (`design.rs`, `gallery.spec.ts`). Existing callers are unaffected (all optional).
2. In `RegisterForm`, between the password `Field` (:303) and the submit (:306), same `gap-5`: `<Checkbox id="register-terms" name="accept_terms" required=true touch=true checked=... />`, unchecked by default, never pre-ticked. The input must carry `required` as a native attribute so it blocks submit before hydration. ActionForm posts `accept_terms=on`: the server fn parameter is `accept_terms: Option<String>` (a `bool` field fails to decode "on"); the server refuses `None`.
3. **Label, Jasper's verbatim:** "I am 16 or older and accept the [Terms] and have read the [Privacy page]". `[Terms]` links `/legal/terms`, `[Privacy page]` links `/legal/datenschutz` (route per CLAUDE.md), both `target="_blank" rel="noopener"` with an `sr-only` " (opens in a new tab)", class `font-medium text-primary underline decoration-outline underline-offset-4 hover:decoration-accent` (the app's inline link style at register.rs:255). No trailing full stop is added and there is NO `TODO(legal-wording)` marker. Opening in a new tab keeps the half-filled form.
4. Server rejection (missing value) in the existing Danger `Callout` at register.rs:~290: "Please confirm that you are 16 or older and accept the Terms and have read the Privacy page." (same words as the label, via `error_text`). Also mark the input `aria-invalid` and move focus to `register-terms` when this is the only error (`focus_id`).
5. States: unchecked, checked (accent fill + check mark, existing), error (callout + aria-invalid), disabled: none. The submit stays enabled. Layout: 390 and 1280 identical, label wraps; the whole `<label>` is the 44px tap target below `md` (A1); links inside keep their own native hit area, so keep them on a line with at least 8px between (the label's `gap-2.5` plus text wrapping).
6. Tab order: username, email, password, checkbox, Terms link, Privacy link, Create account. Record the acceptance server-side only if the data model (`MISSION`/privacy page) already says so; otherwise it is a gate, not a stored consent: that is a product question for the item, not for atelier.

## C9. Reset code Copy on /admin/resets (AMEND)

Current: `src/pages/admin/resets.rs:116-131` renders the minted code in a `Callout` Info (`<code class="block select-all font-mono text-title figures text-primary" data-testid="reset-code">`), inside a `role="status"` wrapper (:~118, good), expiry as `LocalTime`, create button already `Primary` + `Touch` (:113). There is NO Copy button, so the card's `Copy code` is still open.

Amended recipe: add inside the Callout after the paragraph a `Button Secondary size=Touch` "Copy code" using the same `copy_to_clipboard` helper and the same three-state rule as C1 (label becomes "Copied" only after the promise resolves; failure shows `<p role="alert" class="text-label text-danger">Could not copy. Select the code and copy it by hand.</p>` and selects the `<code>`; nothing is announced as copied before success; `sr-only role="status"` "Code copied" on success). Row layout: the button sits under the paragraph, left aligned, at 390 and 1280. Because the code is shown once and is single-use, no ConfirmDialog and no toast. The Callout wrapper must not be a second `role="status"` (it is nested in one already; keep one live region).

## C10. Titles and return to page (AMEND)

Current: one `<Title text="Team Companion" />` at `src/app.rs:81` for every route; no `?next=` anywhere in `src`; the login server fn always redirects to "/" (`login.rs:63-72`, `Ok(dest)` consumed at :79-87); signed-out pages hard-navigate with `window.location().set_href("/auth/login")` from 14 sites (`profile.rs:274`, `personal_learnings.rs:232,415`, `action_items.rs:256`, `share.rs:578`, `match_detail.rs:833`, `post_game.rs:581`, `team/dashboard.rs:530`, `me.rs:239`, `opponents.rs:440`, `game_plan.rs:656`, `stats.rs:298`, `community/mod.rs:57`, `champion_pool.rs:285`, `tree_drafter.rs:326`, `draft.rs:1251`).

Amended recipe:
1. **Titles:** `PageHeader` (`ui/layout.rs:86-112`) emits `<Title text=...>` from `leptos_meta` as "{title} \u{00b7} Team Companion", using the live `title` closure but collapsing when the title equals the eyebrow or is empty. For editors with a document name ("New draft", the plan name) the document name is the title, which is correct. Pages without `PageHeader` set their own: landing "Team Companion", 404 "Page not found \u{00b7} Team Companion", closed beta already has a header, share page "Shared game plan \u{00b7} Team Companion" (never the plan's name: public). Keep `app.rs:81` as the fallback. Verify with `curl -s` on login, register, /draft, /game-plan, /team/dashboard, /bugs for `<title>`, plus `toHaveTitle` after hydration (a `Title` under a `Suspense` can arrive late in streamed SSR; headers are outside Suspense on most pages, so acceptable).
2. **`?next=`:** one pure function `safe_next(raw: &str) -> String` in `src/models/` (shared SSR and WASM, unit tested). Accept only a same-origin path: starts with "/", not "//", not "/\", no ASCII control characters or whitespace, no "://" anywhere before the first "?" (this also rejects a scheme or host), not starting with "/auth/" or "/api/"; otherwise "/". Tests: `/draft?draft_id=a`, `/` pass; `//evil`, `/\evil`, `https://evil`, `javascript:alert(1)`, `/auth/login`, ` /x`, `/x\n`, empty, `%2F%2Fevil` fail to "/". Percent-decoding happens once (the query map decodes), validate after decoding.
3. **Producer:** one helper `login_href()` in `pages/navigate.rs` returning `/auth/login?next={urlencoded current path+search}` (hydrate-only, falls back to `/auth/login`), and replace the 14 `set_href("/auth/login")` calls with it. Do not add `next` when the current path is `/` or under `/auth/`.
4. **Consumer:** `LoginPage` reads `next` from `use_query_map`, puts it in a hidden input `next` (use `value=`, not `prop:value`, so a pre-hydration post carries it, same reasoning as register.rs:~281), `LoginAction` takes `next: String`, computes `dest = safe_next(&next)`, passes it to `leptos_axum::redirect` and returns it; the hydrated Effect already navigates to the returned `dest` (`login.rs:79-87`). The server validates; never trust the client-side value. Registration keeps landing on "/" (new accounts have no page to return to).
5. Verification: unit tests above; Playwright: signed out, open `/game-plan`, expect `/auth/login?next=%2Fgame-plan`, sign in, expect `/game-plan`; a forged `?next=https://evil.test` lands on "/".

## Cross-cutting checks the builder runs

- A1: `getBoundingClientRect().height >= 44` at 390 for invite actions, Add the next match, editor Save/Cancel, Save draft/plan/node, Retry, Try again, Copy code, the register checkbox label. Desktop (1280) heights unchanged from `Touch` (32px).
- A2: new strings use `text-primary|secondary|muted|danger` on `bg-page|bg-ink-1|bg-ink-2`; recompute from rendered pixels, not tokens.
- A3: nothing animates; "Copied" revert and the 4 s Saved fade are timeouts.
- A4: every state has words; live regions exist before they change.
- `scripts/lint_design.sh` stays at zero (no `rounded-full`, palette classes, sub-12px text, emoji). New `Checkbox` props and `ShareData` web-sys feature need a `cargo check` for both targets.
- Re-capture before merge: Playwright 393x852 and 1280x800 of the dashboard invites, Match Day with no match, draft failed state, register form, `/admin/resets` after minting; run `node wiki/assets/ux-rubric/rubric-check.mjs --app-dir <repo>/app <render-path>` on each. Atelier did not run these in this re-read.

## Staleness flags

- CONTRADICTION: wiki/meta/missions/M137-team-companion-first-patch.md (names T1 to T4 as typography) vs wiki/assets/ux-rubric/rubric.json (T1 to T4 are trust checks; typography is V2/V3). Still open.
- STALE: .planning/missions/M137-team-companion-first-patch/atelier-critique.md card C4/C5 (propose new `SaveStatus` and `LoadFailed` in `status.rs`/`empty.rs`) and shared-primitive items 1 to 3: superseded by M141 (`status.rs` exists) and M137 R1 (`ButtonSize::Touch` exists). This file supersedes those parts.
- CONTRADICTION: docs/DESIGN.md 6.9 ("phones are read-only for match editing") vs C2: no-match case is an oversight, with-match stays a decision (unchanged from M137).

## Open questions for Jasper (max 2)

1. C1: when a coach (not the leader) mints an invite, should the new player start "not placed yet" as today (the leader places them), or may a coach place them too? Atelier assumes the leader only places; the intro copy changes to "until the leader places them" for coaches.
2. C6: should the 16+/Terms acceptance be stored (timestamp on the user row, needs a migration and an `account::USER_LINKS`-style export line) or only gate the sign-up form? Atelier recommends only gating unless the privacy page promises a record.
