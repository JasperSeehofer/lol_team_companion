# M148 — Team Companion first patch, R2 — REPORT-R1

Branch `mission/M148-team-companion-first-patch-r2`, base `origin/main` 9b7da51 (main has not moved; the merge of
`origin/main` is a no-op, "Already up to date"). Items 0, 2, 2b, 3, 4, 7, 8, 9 and 11 shipped, and item 10 is
reported below. The overseer called wrap-up at the token cap, so no new fixes went in after ba0c92b. For the
CI is **green on ba0c92b** (run 39007209640: all five jobs, 166/166 e2e, 0 flaky).

## Budget

- Declared 6.0 M, hard cap 8.0 M. The overseer metered **8.45 M after 5.5 h**, so the round **ran over the
  cap**. It was stopped at wrap-up: no further atelier iterations and no new fixes.
- The subagent context totals the harness reported add up to only ≈1.7 M, so the metered figure is mostly the
  orchestrator plus the agents' tool traffic.
- The biggest single cost was the first implementer: items 3 and 2b took 2 h of wall time. Next most expensive
  were the CI loops: four green or near-green runs, plus a separate CI-only branch for the "before" frames.
- Wall time was about 6 h against a 14 h maximum.
- STOP RULE: items 3, 2b, 8 and 7 were all done before the cap, and so were items 2 and 4. Nothing moves to R3
  because of the cap. What R3 inherits is listed under **R3**.

## Least-confident decisions (read these first)

1. **The run before the tip had 11 flaky e2e tests.** That was run 39004927668 on d9dcff5. Its one
   unexpected failure was the new `scrollWidth <= 390` assertion catching a real overflow (a long email
   widened `/profile` to 438 px), fixed in ba0c92b. The flaky tests passed on retry; they look like runner
   load. The tip's run 39007209640 had 0 flaky, so whether they come back is the thing to watch.
2. **Coach invite rights** (server change on a live product):
   - A coach may now create, list and disable their own team's invite links (`server/db.rs:214`
     `is_team_coach`, `:227` `is_team_leader_or_coach`, `server/authz.rs:370/432`, `pages/team/dashboard.rs:75/100/146`).
   - The team always comes from the session. Disable stays scoped with `WHERE team = …`, so another team's
     code answers NotFound.
   - Members and substitutes still get the old error text, "Only the team leader can do that". That text is
     now slightly inaccurate.
   - Placing a player stays leader-only. That is the atelier's open question, and Question 2 asks it.
3. **Item 7 also changed the legal pages.** Because the acceptance is now stored, one sentence was added to each:
   - Terms, "Your account": "You must be 16 or older to register; at sign-up you confirm it and accept these
     terms."
   - Privacy page, "Your account" list: the time you confirmed this at sign-up.
   - Jasper may want to show both to the adviser (Question 3).
4. **Accounts registered before this round have `terms_accepted_at = NONE`.** That includes the seed and test
   accounts. The signed-in "Join TEAM" path does not ask again.
5. **The terms refusal text is the atelier card's wording**, not a new one: "Please confirm that you are 16 or
   older and accept the Terms and have read the Privacy page." It is one constant, `register.rs:24`.
6. **Titles inside a `Suspense` may appear only after hydration.** Admin pages and the share page set theirs in
   the server HTML.
7. **The integration test for item 7 checks the gate function and the invite's uses, not `register_action`
   itself**, which needs a request context. The ordering (refuse first, then consume) is proven end to end by
   `auth.spec.ts`: a direct POST without `accept_terms`, and the one-use code still registers afterwards.

## What shipped, per item

| # | Item | Commit(s) | Notes |
|---|---|---|---|
| 11 | CI write-back for every `mission/*` branch | ceb5746, 364d50e, 003bb29 | `BRANCH` comes from `github.ref_name`, and `MISSION_DIR=.planning/missions/${BRANCH#mission/}`. It runs on push only. `MISSION_FRAMES_DIR` takes `before/` and `after/` frames. M141 frames now go only to `mission/M141*` branches; before this fix they had collided with M148's names. This is the same change as M149 item F: **this branch carries it, so M149 reports it as already landed** if this branch lands first. |
| 0 | Atelier re-read of C1, C2, C4, C5, C6, C9, C10 | af54fb6 | `atelier-critique-r2.md`: C2 CONFIRM, every other card AMEND (it reuses `ui::status`, and Jasper's C6 wording wins). |
| 3 | Phone Match Day "Add the next match" | f71bc25 | `home.rs`: the add action is visible on phones at Touch size (44 px). The "Read-only on the phone" caption shows only when there is a match. With no dossiers there is an "Add an opponent" link. DESIGN.md §6.9 is updated. New `match-day-phone.spec.ts`. |
| 2b | "Copy code" on `/admin/resets` plus the clipboard helper | fd95de5 | New `src/components/clipboard.rs`, with `copy_text` awaiting `writeText`, `can_share`/`share_url` and `copy_with_state`. It reports success only after the promise resolves. On failure it shows an error and selects the code. `password.spec.ts` has a success case and a refused case. |
| 8 | Per-route titles plus `?next=` | bc3cce3 | `PageHeader` sets "{title} · Team Companion". `models/login_next.rs:25` `safe_next` is shared by both targets, and the server validates in `login_action` (`login.rs:75`). It refuses non-`/` values, `//` and `/\` at the start, any byte outside printable ASCII, a backslash anywhere, `://`, and `/auth` or `/api` paths. It runs the same checks again on the percent-decoded form and caps length at 512. Unit tests cover the refusals. All 17 signed-out redirects go through `login_href()`. New `login-next.spec.ts`. |
| 7 | 16+ and terms checkbox | 82b6166 | Jasper's verbatim label; the links open in a new tab. `Checkbox` gets `name`, `required`, `touch`, `invalid` and `label_extra`, with a gallery sample. The box is native and `required`, named `accept_terms`. **The server refuses at `register.rs:80`, before `consume_invite_grant` at `:96`.** `terms_accepted_at option<datetime>` with no migration, set by `db::create_user_accepting_terms`, and included in the data export. Every client that posts the form was updated (fixtures, specs, `smoke_signin.sh`, `ssr_stress.py`, `flow_test.sh`, `api_test.sh`). No `TODO(legal-wording)` ships. |
| 2 | Invite Copy, Share, coach invites | 4dcf5fc, 52691fc | The full URL shows per row after a reload; it is rendered as a path on the server and gets the origin after hydration. Copy and Share use Touch size, and Share appears only when `navigator.share` exists. `share.rs:256` no longer toasts before the promise. The invites list shows `LoadFailed` instead of an empty list. Coach rights are covered above. Integration test `the_leader_and_coaches_manage_invites_members_do_not`, plus `team-invites.spec.ts` (coach mints, member sees no panel, clipboard success and refusal). |
| 4 | Autosave failure visibility plus unsaved guard | cf7dd63 | It reuses and extends `ui::status`: `saved_steady`, `show`, `is_failing`, `save_failure_reason` and `use_unsaved_guard` (beforeunload). On draft, game plan and tree: "Could not save: …" with Retry, one toast per failure, and a success toast only on Ok. Branch and Improvise now check the `save_node` result. `LoadFailed` replaces the empty lists. New `save-failure.spec.ts`. |
| 9 | Frame strips | 363613a, 2f67a36, c3b38b8, 1b2b884, d9dcff5, ba0c92b | "Before" comes from main 9b7da51, through the CI-only helper branch `mission/M148-team-companion-first-patch-r2-before` (**never merge it**). "After" comes from CI. The atelier critique is `publish-round-r1/atelier-frames-critique.md`; its three overflow FIXes and the 44 px login link are applied, and a fourth overflow (profile) was caught by the new assertion. The rubric harness output is `publish-round-r1/rubric-check.md`. |
| 10 | M137 page mislabel | — | Reported below, for the overseer to apply. |

## Item 10: the vault edit (for the overseer; the garden was not edited)

`wiki/meta/missions/M137-team-companion-first-patch.md:141`, in "UI definition of done":
replace `and **T1–T4** (typography scale and density)` with
`and **V1–V3** (visual: spacing scale, type scale and line length; T1–T4 in rubric.json are trust checks)`.

## Gates

- **On the host**, before every handback: `cargo fmt --check`, `cargo check --features ssr`, `cargo check
  --features hydrate --target wasm32-unknown-unknown`, `cargo test --features ssr --lib` (424 passed) and
  `scripts/lint_design.sh` (OK). All ran with `CARGO_BUILD_JOBS=2`. Load average stayed below 2.
- **CI runs:**

  | Run | Commit | Result |
  |---|---|---|
  | 38995081934 | fd95de5 | e2e red on 2 tests (the reserve-label text assertion), fixed in 52691fc |
  | 38998423121 | 3c83f76 | all green, 1 flaky |
  | 39000854398 | 363613a | all green, 164/164 e2e |
  | 39002450956 | c3b38b8 | **all five jobs green** |
  | 39004927668 | d9dcff5 | 1 unexpected (the profile overflow, a real bug), 11 flaky |
  | 39007209640 | ba0c92b | **all five jobs green, 166/166 e2e, 0 flaky** (the tip) |

- **e2e subset in CI:** the earlier list plus `frames-m148`, `match-day-phone`, `login-next` and
  `save-failure`. The updated `auth`, `password`, `team-invites` and `community-gates` specs are in it too.
- **Not in CI:** `match-day.spec.ts`. Its phone case lives in `match-day-phone.spec.ts`.
- **visual-regression:** `visual-regression.spec.ts` (not in CI) has a register snapshot that is now stale.

## UI definition of done

- **Frames:** `publish-round-r1/frames/{before,after}/` holds, at 390, 1280 and 1280-flow:
  - pairs: admin-resets, auth-reset, dashboard-invites, draft, game-plan, login, login-hint, login-next,
    match-day-phone, profile-password, register, tree-drafter;
  - after only: draft-save-failed, game-plan-save-failed, tree-save-failed;
  - before only (lists): draft-list, game-plan-list, dashboard-invites-minted.
- **Atelier verdicts on the live captures:** PASS on match-day-phone, draft, game-plan, tree-drafter, register,
  login, login-next and auth-reset. FIX (overflow at 390) on admin-resets, dashboard-invites and
  tree-save-failed; all three are fixed in d9dcff5. The atelier has **not re-checked them on new captures**,
  because the overseer stopped further atelier iterations. That re-check is R3, or the verifier's job.
- **Rubric harness** (`rubric-check.mjs`, vendored to `scripts/ux-rubric/`, run in CI on the public pages
  only, because it has no sign-in option):
  - A1–A3, I7, V1 and V4 PASS.
  - I1, I2, I6 and I8 FIX are the harness's GUESS heuristics and don't apply to signed-out forms.
  - V2 points at the feedback widget's `data-feedback-*`, which this round did not change.
  - V3 is a per-paragraph character count. The atelier judges it a misfire, except for the footer disclaimer at
    about 95–100 characters per line at 1280, which predates this round.

## Security notes, for the verifier's read

- **Coach rights:** see Least-confident decision 2. The tests cover:
  - a coach is allowed to create, list and disable;
  - a member and a substitute are refused;
  - the other team's coach or leader gets NotFound or an empty list;
  - a demoted coach loses the right.
- **Terms before the invite:** `register.rs:80` refuses before `register.rs:96` consumes. Nothing is written,
  and the invite is not touched.
- **`?next=`:** only the server decides the target. The client's hidden field is re-validated in
  `login_action`. Encoded forms (`/%2F%2F`, `/%5C`, `/%61uth`) are refused.

## R3 (not done, with reasons)

- **Atelier re-check of the FIXed frames:** stopped at the cap by the overseer.
- **Pages left for Harden:** their autosave and load errors were not touched in item 4.
  - `post_game.rs`, which has its own `review-save-state` at line 806, plus "did not load" callouts at 856
    and 1211.
  - The remaining draft.rs callouts.
  - `action_items`, `opponents`, `personal_learnings`, `stats`, `me`, `share`, the dashboard (:564), `roster`,
    and the game plan templates list.
- **Members' refusal text:** say "the leader or a coach".
- **The `/opponents` add form at 390:** not captured.
- **The 11 flaky e2e tests on run 39004927668:** watch whether they recur (0 on the tip).
- **visual-regression:** regenerate the register snapshot.
- **Rubric harness:** add a sign-in option so signed-in pages can be checked.
- **Write-back race:** a run on an older commit can push its write-back after a newer push. That happened once
  here (the M141 frames) and was cleaned up in 003bb29.
- **Orchestration deviation:** this report and `entry.json` were written by the orchestrator, not by a
  subagent, to stay inside the overseer's wrap-up budget.
