# M159 — Match thread: schedule and scoping — REPORT-R1

Branch `mission/M159-match-thread-schedule-and-scoping` (pushed, branch only), base `origin/main` `f86e1dc` (the M152
merge). Nothing was merged to main, deployed or touched on the VM. Card: `publish-round-r1/entry.json`
(id `m159-r1-match-thread-schedule-and-scoping`). Items 0–5, 9 and 10 shipped. **Items 6, 7 and 8 are not built and
move to R2 under the STOP RULE.** CI is **green on the tip `cc88702`** (run 39638849658: all five jobs, 191 expected,
0 unexpected, 0 flaky, 0 skipped). Later commits only add CI write-backs under `.planning/`.

## Least-confident decisions (read these first)

1. **The Bo3 "played" rule.** A game counts as played when a draft or a review is stamped with it. If nothing is
   stamped, G1 is still required. Games with no work at all read "not played". So the readout looks like
   "G1 done · G2 to do · G3 not played", and the review step's "1 of 2 reviewed" counts only the played games. A team that
   plays G3 but never drafts or reviews it shows G3 as "not played". This is react question (d) on the card.
2. **"Review as a team" shows only for picker candidates**, meaning games with at least 3 current members of the team
   (`src/models/review_pick.rs`, `src/server/review_pick.rs`). That is narrower than "team games" on `/stats`. A game
   with only two members synced has no entry on its match page.
3. **Toasts persist across client route changes.** This was kept by design: `ToastProvider` sits in `App`, and editors
   push "Saved" or Undo toasts right before `replace_url`. Dismissing toasts on a route change would cut that feedback
   short. The atelier's stray-toast FIX was therefore treated as a capture fix: `match-thread.spec.ts` dismisses every
   toast before each frame.
4. **The seed tool changed.** The demo game is now stored for all five Northgate members, and a second game,
   `SEED1_1000000002`, is added: a lost rematch that ended 3 h before the seed ran. It has a scoreboard only and no
   timeline (`src/bin/seed.rs`). The picker frame needs this to show "Near the scheduled match" next to
   "Other recent games". `post-game-fill.spec.ts` selects the demo game by value and still finds it.
5. **"Use for this match" acts per item and has no Undo toast.** Earlier unstamped work against the same opponent
   is offered and never counted. One tap stamps that one draft or plan onto the match. Nothing is ever stamped
   automatically by date. This is react question (b).
6. **The opponent re-check covers `save_draft` and `db::update_draft` through `ensure_opponent_ref`
   (`src/server/db.rs:173`).** An `opponent_id` from the URL must be the caller's team's dossier, or the draft is refused.
   The free-text opponent name is still accepted, as before.
7. **The picker preselects nothing unless `riot_match_id` is valid.** A malformed or foreign id in the URL leaves
   every row unselected.
8. **Prep facts are computed for matches with no dossier too.** A match whose opponent has no dossier still gets
   match-scoped prep facts from the work stamped onto it.

## Budget

- **Declared 10.0 M, cap 13.0 M, max-wall 18 h.** The metered total is the overseer's ledger, which this report does
  not have. The overseer applied the STOP RULE early, so the round stopped before items 6–8.
- **Subagent usage seen in this run's notifications: about 1.28 M tokens** across the completed subagents:

  | Agent | M tokens |
  |---|---|
  | atelier critique (item 0) | 0.05 |
  | code map | 0.21 |
  | items 1–2 | 0.13 |
  | items 3–4 | 0.18 |
  | item 5 | 0.20 |
  | docs (items 9–10) | 0.12 |
  | fix (`ThreadLookup`) | 0.08 |
  | atelier frames critique | 0.07 |
  | before frames | 0.08 |
  | frame fixes | 0.15 |

- **Four implementer agents died with their usage unreported.** Three were reaped by the watchdog for stalling during
  cold builds on an overloaded host, and one hit an OAuth expiry. The orchestrator's own usage comes on top of all
  this.
- **Wall time: far past the 18 h max-wall.** The round was dispatched on 2026-10-18 at about 23:30 and finished on
  2026-10-21 at about 19:45. The reasons:
  - the host was starved (load up to about 100, memory pressure, and the CI watcher reaped);
  - the daemon lost its auth;
  - the session was stopped and later resumed.

  Active work time was much shorter than the elapsed time, but this report cannot give an exact figure for it.
- **Why the STOP RULE was applied:** the overseer's instruction, citing the account at 80 % of its weekly limit until
  Oct 22 21:00, together with the repeated host outages.

## What shipped, per item

| # | Item | Commit(s) | State and evidence |
|---|---|---|---|
| 0 | Atelier critique and recipe cards | b0152ea | `atelier-critique-r1.md` |
| 1 | Match stamps on drafts, plans and reviews | 877967a | Optional baseline fields in `schema.surql` (no migration). `scheduled_match.series` and `post_game_learning.series` also came in here; **they are unused until item 6** |
| 2 | Upcoming-match list on Match Day | 877967a | `src/pages/home.rs` (UpcomingMatches), `src/models/match_day.rs`; e2e "a second match shows in the upcoming list" |
| 3 | Carried links (the draft opens prefilled from the thread) | ca4dcf8 (WIP), 2fd6517 | `src/pages/draft.rs`, `src/pages/game_plan.rs`; `ensure_opponent_ref` at `src/server/db.rs:173`; e2e "the draft opens prefilled". The CI run on ca4dcf8 caught a real bug (a duplicate "Start a draft" link), fixed in 2fd6517 by guidance `repeats_step` |
| 4 | Prep facts scoped to the match; earlier work offered, not counted | ca4dcf8, 2fd6517 | `src/models/match_day.rs`, `src/pages/home.rs`; Bo3 readout; e2e "earlier work is offered, not counted" |
| 5 | Review from a real game (picker, match page entry) | c04ed57 | New `src/models/review_pick.rs` and `src/server/review_pick.rs`, `src/pages/post_game.rs`, `src/pages/match_detail.rs`; e2e "a scrim review saves through Not in the list" and "a team game is chosen from the match page" |
| — | Fix: `ThreadLookup` enum | caa250a | A nested `Option` collapsed to JSON `null` during hydration, so the unknown-match line never appeared on `/draft`, `/game-plan` or `/post-game`. CI caught it on 92e28bd and c69ee0c |
| 6 | Series on the scheduled match | — | **Not built (STOP RULE), R2** |
| 7 | RSVP | — | **Not built (STOP RULE), R2** |
| 8 | Brief share link and migration v17 | — | **Not built (STOP RULE), R2** |
| 9 | Privacy and account audit | c69ee0c | No STORED change was needed; see below |
| 10 | ARCHITECTURE and CHANGELOG | c69ee0c | `docs/ARCHITECTURE.md`; `CHANGELOG.md:11` "Milestone M159: Match thread (2026-10-18)" |
| — | Before frames and rubric check | 6e51456 | `publish-round-r1/frames/before/SOURCES.md`, `rubric-check.md` |
| — | Atelier frame fixes | cc88702 | See "UI definition of done" |

Nothing of items 6–8 is half-built on the branch. There is no migration v17, no new table and no new user field
this round.

**"One commit per item" was not fully met.** Items 1–2 and 3–4 share hunks (`home.rs`, `match_day.rs`, `db.rs`), so
each pair was committed together.

Integration tests: new `tests/integration/match_thread.rs` (stamps, stamp clearing on deletion, cross-team refusal of
match and opponent ids, picker filter), plus cases in `security.rs`, `match_day.rs`, `db_drafts.rs`,
`db_game_plan_pipeline.rs` and `transactions.rs`. The new e2e spec `e2e/tests/match-thread.spec.ts` has 5 tests and
is in CI's subset.

## Item 9: STORED audit (no change needed)

- **Account links already exist.** `scheduled_match.brief_read_by` (RemoveFromArray) and `scheduled_match.created_by`
  (Tombstone) are in `USER_LINKS` (`src/server/account.rs:97-103`) and in the deletion statements (`:225-226`). The new
  stamps are team records, not user references, so `USER_LINKS` gains nothing.
- **Team deletion:** `DELETE_TEAM` (`src/server/account.rs:155-189`) already deletes `scheduled_match`, `series`,
  `draft`, `game_plan` and `post_game_learning` for the team.
- **Deleting a match or a series clears its stamps:** `db::delete_scheduled_match` (`src/server/db.rs:8273`) and
  `db::delete_series` (`:4129`), covered in `tests/integration/match_thread.rs`.
- **The privacy page** already names the next match in STORED (`src/pages/legal/datenschutz.rs:43`), and the schema
  test's table lists already contain `series` and `scheduled_match` (`:257`, `:271`). No new table, so no change.
- **One wording point for R2:** STORED says "the next match", but a team can now keep several upcoming matches. R2
  should change this together with the RSVP and share-link sentences (react question (a)). No STORED sentence changed
  this round.

## RSVP storage and migration behaviour

- **RSVP storage:** not built this round. The R2 default proposed on the card is a new `match_rsvp` table, deleted with
  the match and with the account, and included in the export (react question (c)).
- **Migration:** none this round. Every new field is an optional baseline field in `schema.surql`, and existing rows
  load through serde defaults. **No migration runs at the next start from this round.** Migration v17 is R2.

## Gates

- **Local (brain, shared target dir):** `cargo check` on ssr and on hydrate/wasm32, `cargo clippy -D warnings` on
  both, `cargo test --features ssr --lib` with 462 passed, `cargo check` of the integration test target,
  `cargo fmt --check`, and `./scripts/lint_design.sh` at zero. The full integration run and e2e ran only in CI.
  **CI is authoritative.**
- **CI runs:**

  | Run | Commit | Result |
  |---|---|---|
  | 39409436969 | 877967a | green |
  | 39617193146 | ca4dcf8 | 1 failure: the duplicate "Start a draft" link, a real page bug fixed in 2fd6517 |
  | 39628884536 | 92e28bd | failed on the unknown-match line, a real bug fixed in caa250a |
  | 39631700231 | c69ee0c | the same failure; flaky: `hero-matchup.spec.ts:144` (plan name ""), timeouts in `bug-report` and `frames-m148` |
  | 39637088684 | 7d5e728 | green |
  | **39638849658** | **cc88702** | **all five jobs green**: check/fmt/clippy (both targets)/unit/integration, image, style guardrails, ops scripts, e2e 191 expected / 0 unexpected / 0 flaky / 0 skipped; 0 slow-request lines (`ci-e2e-summary.md`) |

- **e2e subset vs. the Done criterion:** smoke, account, legal and the new `match-thread.spec.ts` are in CI.
  `match-day.spec.ts` is still not in CI's list, as in M148; the Match Day cases M159 needs live in
  `match-thread.spec.ts` and `match-day-phone.spec.ts`. The share and RSVP specs belong to items 7–8 (R2).
- **Law 10:** `git fetch` confirmed that `origin/main` is still `f86e1dc`. The branch already contains it, so the merge
  is a no-op and the green tip is the post-merge state.

## UI definition of done

- **Frames** (`publish-round-r1/frames/after/`, from CI on cc88702): match-day-upcoming, draft-from-thread (with a
  390 flow frame), match-day-prep-bo3, post-game-picker and match-detail-review, each at 390 and 1280, plus flow
  frames. The other files in `after/` are the earlier missions' frame specs, which still run in CI.
- **Before frames** (`frames/before/`) are **proxies**: the closest M152 and M141 frames of the same pages, all from
  ancestors of `f86e1dc`. `SOURCES.md` lists each source and the gaps. The M159 states had no true before, and there
  is no full-page 1280 Match Day frame and no match-detail 1280-flow frame. None was fabricated.
- **Atelier** (`publish-round-r1/atelier-frames-critique.md`):

  | Surface | Verdict |
  |---|---|
  | match-day-upcoming | PASS |
  | post-game-picker | PASS (capture FIX: more than one candidate, done through the seed) |
  | match-detail-review | PASS |
  | draft-from-thread | FIX → resolved in cc88702 (the stray toast was a capture issue; 390 flow frame added) |
  | match-day-prep-bo3 | FIX → resolved in cc88702 (the frame now shows "G1 done · G2 to do · G3 not played" and "1 of 2 reviewed"; "Use for this match" aligned from `md`) |

  The executor's answers are in the critique's "Executor response" section. **Critique item 4 ("No answers yet" on the
  upcoming row) was skipped**: RSVP is R2, and showing the text would claim a feature that does not exist. The tip's
  frames are recaptured, but **the atelier has not re-read them**; the verifier can.
- **Rubric harness** (`rubric-check.md`): CI's `rubric-check.mjs` measured only the public auth pages, because it has
  no sign-in step. **The signed-in M159 surfaces were judged by the atelier alone, not by the harness.**
- `./scripts/lint_design.sh`: zero on every rule.

## Security notes, for the verifier's read

- `match_id`, `opponent_id` and game/stamp params from the URL are re-checked server-side as the caller's team records
  (`ensure_opponent_ref`, the team-scoped match lookup). Unknown, malformed or another team's ids resolve to
  `ThreadLookup::Unknown` (`src/models/match_day.rs:162`, caa250a) and show the neutral "unknown match" line. Cross-team cases: `tests/integration/match_thread.rs`,
  `tests/integration/security.rs`.
- The picker lists only games with at least 3 current members of the caller's team. A foreign `riot_match_id`
  preselects nothing (decision 7).
- The RSVP, brief share page and migration v17 checks from the review regime do not apply this round (R2).

## Landing

Landing is **tap-only** (D6): the merge chip is the landing, never an auto-merge. **Merged ≠ live** (Law 7): deploy
goes through `82-release.sh` behind its own cards. No migration runs at the next start from this round.

## R2 (not done, with reasons)

1. **Item 6, series on the scheduled match**, as the mission page specifies (the `scheduled_match.series` field
   already exists, unused). Reason: STOP RULE.
2. **Item 7, RSVP** (`match_rsvp` by default, per the card), with the "No answers yet" caption from the atelier.
   Reason: STOP RULE.
3. **Item 8, the brief share link and migration v17.** Reason: STOP RULE.
4. **The STORED wording:** "the next match" becomes "scheduled matches", together with the RSVP and share-link lines,
   shown to the adviser.
5. **An atelier re-read** of the recaptured draft-from-thread and match-day-prep-bo3 frames.
6. **A sign-in step for the rubric harness**, carried over from M148.
