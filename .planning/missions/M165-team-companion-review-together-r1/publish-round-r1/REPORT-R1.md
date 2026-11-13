# M165 — Team Companion Review together — REPORT-R1

**Branch.** `mission/M165-team-companion-review-together-r1`, pushed (branch only). Base: `origin/main` `7951e05`. `origin/main`
`1018903` (M156) has since been merged in. Nothing was merged to main or deployed, and the VM was not touched. **Card:**
`publish-round-r1/entry.json`.

**What shipped.** Items 0–11 are complete, nothing moved to R1b, and the STOP RULE was never reached. The full decide flow is
built: vote, clustering and merge, and the quiet nudge with TC-J12.

**CI.**

- **`37a7936`** (run 43518892716): green on all five jobs. e2e: 192 expected, 0 unexpected, 0 flaky, 0 skipped; the server
  log has 0 slow, inflight, panic or error lines.
- **Tip `1aab82c`** (the post-round F1 fix `977542c`, run 43524977997): check and e2e green, with the same counts.

## Least-confident decisions (read these first)

1. **Account deletion keeps a person's anonymous cards.** They move to the "Former member" tombstone, as signed cards and
   team notes do (the suggested answer the mission page gave). But the STORED sentence says the app keeps the author link "so that
   you can export or delete it". After the reveal a card is locked and cannot be deleted on its own. Account deletion only
   unlinks it; the text stays in the team's review. So the sentence is only half true about "delete". I did **not** change
   the prose beyond what item 10 asked. Card question 1 puts it to you with a suggestion: delete the person's anonymous cards on
   account deletion and tombstone the signed ones, so the sentence holds as written. That would be a small R1b change.
2. **Voices and the quiet nudge ignore the anonymous prompt.** An independent privacy review found that "5 of 5 voices"
   could unmask the anonymous cards. If A–D signed cards and E signed nothing, "5 of 5" says E wrote an anonymous card.
   Fix `234091d`: a voice is a member with at least one non-pass card in a **signed** prompt ("decided", "to plan",
   "next time"). A member counts as quiet when they have no such card. So a member who wrote only anonymous cards reads
   exactly like one who passed. The price: that member is not counted as a voice. The mission text said "≥1 non-pass
   entry", and I chose the anonymity promise over the letter of the text.
3. **The harness rows are not literally all green.** A1–A3 and V1 pass. I1, I2, I6, I8, V2 and V3 fail on every run, and the
   atelier judged each a harness misfire, a finding about the whole stylesheet, or not applicable to this surface:
   - V2 scans every rule in the stylesheet, so Tailwind's preflight rules and `.sr-only` fail it;
   - V3 counts the `textContent` length of the first 20 `<p>` elements instead of measuring line length, so every
     sentence under 45 characters fails.

   The atelier filed these as STALE notes against `rubric-check.mjs`. The round-0 criterion says the rows are "green", so
   this needs your ruling (card question 3). The real readability point behind V3 was fixed: notes now run to about 72
   characters per line at 1280, down from 93.
4. **Names: "Review together" means only the session.** I overruled the atelier here.
   - **The collision.** M159's "Review as a team" button on a match page opens the single-author editor on a synced Riot
     game. Such a game may have no scheduled match, and a session is keyed on (scheduled match, game).
   - **The rename.** That button is now "Write a review" / "Open the review" (`src/pages/match_detail.rs`).
   - **The review step.** On the match thread's review step, "Review together" is the chalk action and "Write it alone"
     is a quiet link (`[data-prep-alt]`).

   This favours the flagship over a solo coach's habit (card question 4).
5. **When a session closes depends on when it is opened.**
   - **Open by default:** writing closes 24 h after the match's start.
   - **Opened later than that:** writing closes 24 h after opening.
   - **"Reveal now":** offered to the facilitator even without a timed round, so an untimed session does not have to wait
     24 h.
6. **Finding 3 of the privacy review was left as it was (it predates this mission).** The generated review keeps the
   decided texts in `action_items`, and a plain re-save creates no items (tested). But if someone deletes one of those
   action items and then re-saves the review, the item comes back. That is how `new_action_item_texts` already worked
   before this round.
7. **A merged group's +1 count is the first card's count.** The server keeps no count of distinct members per group.

## Budget

- **Declared:** 13.0 M, cap 20.0 M, max-wall 24 h. The metered total is in the overseer's ledger; this report does not
  have it.
- **Subagent tokens from this run's completion notifications:** about **1.96 M**. These are the per-agent figures the
  harness reports, which may not equal billed tokens. The orchestrator's own context comes on top.

  | Agent | Model | M tokens |
  |---|---|---|
  | references researcher | Sonnet | 0.06 |
  | code digest | Sonnet | 0.21 |
  | atelier intake | atelier | 0.14 |
  | server layer (items 1, 2, 6–10) | Opus | 0.30 |
  | reference stills | Sonnet | 0.06 |
  | UI (items 3–8, 11) | Opus | 0.29 |
  | privacy/security review (read-only) | Opus | 0.13 |
  | e2e spec, frames, 5 CI rounds | Opus | 0.30 |
  | atelier critique r1 | atelier | 0.20 |
  | taste round 2 + CI | Opus | 0.13 |
  | atelier critique r2 | atelier | 0.09 |
  | post-round F1 fix + CI | Opus | 0.06 |

- **Wall time:** from the evening of 2026-11-13 to 2026-11-14, inside the 24 h. The e2e loop took 3.7 h of it (five CI
  rounds; rounds 1–4 failed only in the new spec).
- **Rule break:** the server-layer agent once ran `cargo clippy --features ssr --all-targets`, which compiles the
  integration crate. The load average was about 2, and it did not happen again. The e2e agent started one
  `cargo test --lib` at a load of 21.7, just over the 20 limit.

## What shipped, per item

| # | Item | Commits | Notes |
|---|---|---|---|
| 0 | Atelier intake | `60e1b60` | `references.md`, `design-brief-r1.md` (8 recipe cards), committed **before** the first render commit (`719a0ad`). Reference gap: the researcher worked from docs text; the stills (`reference-stills/`, untracked) are bot-blocked or do not show the effect, so V8 was judged against the recipe cards and the app's own serif precedent (`draft_graph.rs` team-note quotes). |
| 1 | Data model | `d77a228` | Four tables in `schema.surql` (no migration; latest is still v16): `review_session` (+ `rev`, `async_closes_at`, `round_started_at`, `scheduled_match_deleted`), `review_entry`, `review_vote` (+ `team`), `review_proposal` (no proposer field). Random record ids, never `ulid()`. Unique `(scheduled_match, game_number)` and `(target, user, kind)`. Models in `src/models/review_together.rs`. |
| 2 | Transport + presence | `d8c5922`, `4bae5ec` | Short poll `get_session_state(session, rev)`: 2 s visible, 10 s hidden, stopped when closed and on unmount, a re-armed `set_timeout` under `#[cfg(feature = "hydrate")]`, no new crate. `Unchanged` is one read by record id and carries the presence list, so arrivals and departures never force a full payload. Presence: in-memory map, 10 s expiry, purged like `rate_limit`, never persisted or logged. The reveal happens at read time, idempotently, under the invariant lock (it locks entries and writes random `order_key`s), so it survives restarts. **Why not SSE:** `server::inflight` logs any request open longer than 15 s as "no complete response", and CI's "No stalled requests" step greps for that. A stream would trip the gate, fight phone sleep and need the watchdog changed. |
| 3 | Open from the thread | `719a0ad`, `27c80ce` | "Review together" per game on Match Day's review step (`PrepStep` gets one quiet `alt_link`, the only primitive change). The page is `/review/together?session=`; `?match_id=&game=` opens the session and rewrites the URL. Any member opens it and becomes facilitator; the leader or a coach may take over. "Write it alone" keeps today's `/post-game`. Once the match has started, the review step is the next step (`27c80ce`). |
| 4 | Silent round, sync + async | `d8c5922`, `4bae5ec` | Four prompts; up to 3 cards of ≤280 characters per prompt, or "Nothing to add here". Autosave with `SaveStatus`; the sealed caption; a whole-minute deadline line; 44 px targets on phones. The facilitator's tools sit at `md`+: start a timed round (3 min), +1 min, reveal now, take over. "N of M written" counts across all prompts, never per prompt. |
| 5 | Reveal | `4bae5ec` | One wrapper fades in over 220 ms, every group at once with no stagger. It is instant under reduced motion and on reload. Cards are grouped by prompt and ordered by `order_key`. Signed words are in `note`, with the author in a sans caption. "What didn't go to plan" shows no name, no avatar and no "yours" marker, with the caption "Anonymous: nobody on the team sees who wrote these." There is a uniform +1 with counts only. Near-duplicates are found lexically (Jaccard ≥ 0.5 on normalised words, within one prompt), and the facilitator merges and separates. Minority cards stay. |
| 6 | Decide | `d8c5922`, `397c12c`, `4bae5ec` | Proposals from cards store no proposer. Each member has ≤3 decide votes, with the count left shown; "Start deciding" pre-ticks the top three by votes. The facilitator accepts ≤3 and sets owner chips. **Owners are checked server-side against current members**, here and on the existing action-item assignment path (new: no check existed). Close is a `ConfirmDialog`. |
| 7 | Generated review | `d8c5922`, `be8957f`, `f133197` | Close is one transaction:<br>• It writes `post_game_learning` with M159's stamps (`series` only when the match has one), `created_by` = the facilitator, `what_went_well` and `improvements` **empty**, and `action_items` = the accepted texts.<br>• It writes the action items through a dedicated path, with `source_review` and the owner.<br>• It writes `session.review`.<br>Re-saving creates no items (tested). The review page gets a read-only "Reviewed together" section: signed cards with names ("Former member" after deletion), anonymous cards unnamed, the decided items, and one "X of Y voices" line. `analyze_patterns` reads the session's to_plan/off_plan texts. |
| 8 | Voices + quiet nudge | `d8c5922`, `234091d`, `4bae5ec` | One team-level voices line; no per-player count anywhere. Nudge: facilitator only, on the session page while writing, a muteable `NudgeLine` "One player has been quiet in the last 3 reviews". The payload is `Option<u8>`, computed on request and never stored, and `None` with fewer than 3 closed sessions. It skips accounts created after the oldest of those sessions. Signed prompts only (decision 2). |
| 9 | Anonymity guarantee | `d8c5922`, `9e31f82` | `AnonymousCard { id, text, plus_ones, i_plus_oned, cluster }` has **no author field**. Tests are listed below. |
| 10 | Privacy rows | `d77a228` | The STORED and LISTED lines, USER_LINKS/DELETE_USER, DELETE_TEAM and the export sections, all listed below. |
| 11 | CHANGELOG + docs | `da292e9` | CHANGELOG entries with `Introduce: /review/together …`; `docs/ARCHITECTURE.md` (session model, poll transport, anonymity rule); one DESIGN sentence in §2/§3.3 allowing `note` for review-together cards (owner's ruling 2026-10-20); the CLAUDE.md routes table. §6.9 untouched. |

Taste round 2 (`37a7936`) fixed atelier F2–F7 and the V3 line length: notes are now `max-w-prose`, about 72 characters per
line at 1280. It also re-shot every frame with neutral marker text (F4), because the round-1 markers showed the author's
letter in the anonymous stills.

## The anonymity design and its tests

- **Data shape.**
  - The author link exists only in `review_entry.author`, kept so the author can export their cards.
  - Every team-visible path builds the off_plan group as `AnonymousCard`, which has no field that could carry an author.
  - No timestamps are returned for cards. Ids are random, and the order is the random `order_key` written at the reveal.
  - Proposals store no proposer, and their `source` is only the card id that is already shown.
  - Keys and `data-note-id` are card ids. No log line pairs a card with a user.
- **Sealing.** While writing, a member's state holds only their own cards plus team-level `written_count`/`member_count`.
  That includes the facilitator. Writes re-check the phase with a fresh clock under the invariant lock, so a write that
  races the deadline cannot land after the reveal.
- **Unit tests** (`cargo test --features ssr --lib`):
  - `src/server/review_together/tests.rs`:
    - `a_revealed_state_serializes_the_off_plan_group_without_authors` (the JSON is searched for every member id, username
      and avatar path);
    - `anonymous_cards_carry_no_author_for_anyone_the_author_included`;
    - `before_the_reveal_each_member_sees_only_their_own_cards`;
    - `the_reveal_happens_at_read_time_and_locks_the_cards`;
    - `caps_passes_and_own_cards_only`;
    - `votes_accepts_owners_and_close_write_a_review_that_resaves_clean`;
    - `the_quiet_nudge_needs_three_closed_sessions_and_is_a_count`;
    - `other_teams_cannot_reach_a_session`;
    - `owner_check_takes_current_members_only`;
    - `account_deletion_export_and_team_deletion_cover_sessions`;
    - `presence_expires_after_ten_seconds_and_is_purged`;
    - and 5 more.
  - `src/models/review_together.rs`: 11 pure tests for phases, text cleaning, the Jaccard threshold, the nudge count,
    accept suggestions, the minute timer and voices.
- **Integration tests** (`tests/integration/review_together.rs`, CI):
  - `cards_are_sealed_until_the_reveal_and_locked_after_it`;
  - `anonymous_cards_and_their_proposals_name_nobody`;
  - `closing_writes_a_stamped_review_that_resaves_without_new_items`;
  - `the_owner_check_refuses_non_members`;
  - `another_teams_session_and_match_ids_are_not_found`;
  - `deletion_and_export_cover_cards_and_votes`;
  - `tests/integration/account.rs` seed rows extended.
- **e2e** (`e2e/tests/review-together.spec.ts`, one test: "review together: sealed, anonymous, decided and counted by five
  members"). Five members registered by the spec, five browser contexts, a Bo5 match. Mapped to the round-0 criterion:
  1. **Sealed.** Every `/api/` body, document and page HTML in every context, the facilitator's included, is searched for
     the other members' markers. The facilitator gets one written count. An `upsert_entry` replayed after the reveal is
     refused.
  2. **Anonymous.**
     - Where it searches: the off_plan group's HTML, its `data-*`, `aria-*` and `title` attributes, the poll JSON,
       `__RESOLVED_RESOURCES` (the hydration payload), and a proposal made from an anonymous card.
     - What it finds: no username or user id, in all five contexts.
     - Identical markup: every anonymous row is the same.
     - Export: the authors' exports list their cards.
  3. **Decide.** The vote cap, then 3 accepted items with owners, then 3 action items. Both point fields stay empty, and a
     re-save adds no item.
  4. **Voices and nudge.**
     - One voices line, "4 of 5 voices" when one member passes everything, and "5 of 5" in game 4.
     - With two closed sessions there is no nudge.
     - With three, the facilitator gets the count 1 and a member gets null.
     - Mute works.

## The deletion treatment

- `review_entry.author`: **Tombstone** ("Former member"; anonymous cards stay unnamed).
- `review_vote.user`: **Delete** (as `proposal_vote`).
- `review_session.facilitator`: **Tombstone**.
- `review_proposal.owner` is a username string, cleared on account deletion as `action_item.assigned_to` is.
- `Treatment::Anonymise` is never used (it is reserved for bug reports, by test).
- `DELETE_TEAM` covers all four tables.
- Deleting a scheduled match keeps its sessions without the link (`scheduled_match_deleted`).
- Deleting a review clears `session.review`.
- See least-confident decision 1 for the mismatch with the STORED wording.

## Changed privacy-page sentences (for your adviser read)

`src/pages/legal/datenschutz.rs`. These are the only legal-prose changes.

- **STORED, "Your team", one sentence added:** "Review sessions: the prompts each member answered, their cards and passes,
  +1 and decide votes (others see only counts), and the action items the team decided. For the anonymous prompt, the app
  stores who wrote each card so that you can export or delete it; teammates never see it."
- **LISTED, entries added:** `review_session`, `review_entry`, `review_proposal`, `review_vote`.
- **Export:** new sections `review_cards_written` (prompt, text, passed, anonymous yes/no, session date) and `review_votes`
  (kind, date). The export holds only the user's own rows.

## Atelier verdicts

- **Round 1** (`publish-round-r1/atelier-frames-critique.md`): FIX. F4 blocked: a capture defect, since the markers showed
  the author's letter. F1–F3 and F5–F7 did not block.
- **Round 2** (`atelier-frames-critique-r2.md`): **7 of 8 cards PASS.** V5, V7 and V8 pass. The writing card (card 2)
  stayed FIX on F1: the sealed sentence was an inline span, so its margin did nothing, and at 390 it sat 3 px above "HERE
  NOW". This was the last taste round allowed, so F1 went out as a one-line post-round fix (below), not as a third round.
- **Reveal sequence.**
  - One wrapper, simultaneous: every group is at 0.73 opacity at 50 ms.
  - 220 ms, instant under reduced motion.
  - No per-person tally anywhere.
  - The nudge appears only in the facilitator frame.
- **Harness:** `publish-round-r1/rubric-check.md` (CI) and `rubric-check-atelier-r1.md` (the atelier's local run). See
  least-confident decision 3.

## Frames

`publish-round-r1/frames/after/`, each at 390 and 1280 with an `.html` snapshot beside it:

- `match-day-review-together-*`;
- `review-together-writing-*`, `-facilitator-*` (with `-sealed-`, `-expanded-`), `-nudge-*`, `-reveal-*`, `-decide-*` and
  `-review-*`;
- the reading-flow frames `*-flow`;
- the reveal sequence `review-together-reveal-seq-{1280,390}-{before,0ms,50ms,110ms,170ms,after}` and
  `review-together-reveal-reduced-1280`.

The directory also holds other specs' frames, because CI writes everything in `MISSION_FRAMES_DIR` back. The before proxies
are in `frames/before/`, from M159's after-frames; `SOURCES.md` lists them and their gaps.

## Post-round fix

`977542c` (CI write-back `1aab82c`) makes two changes:
- **F1:** the sealed sentence is now a block paragraph (`TextTag::P`).
- **F5:** the spec asserts that the reveal announcement reads "Revealed: N notes, X of 5 voices."

**CI run 43524977997.** The e2e job passed: 192 expected, 0 unexpected, 0 flaky, 0 skipped, and 0 slow or panic lines.
It needs the check job, so check/lint/unit/integration passed too. The image, ops-scripts and style jobs were green on
`37a7936`. Their results for `977542c` were not visible here, because there is no `gh`. The change is one Rust line and
one spec assertion. The writing-390 still (`frames/after/review-together-writing-390.png`) shows a gap of about 24 px
above "HERE NOW", so F1 is closed. With that, all 8 recipe cards pass.

## R1b / R2

- **R1b:** nothing from this round's scope. One candidate, if you rule for it on card question 1: delete a leaving member's
  anonymous cards on account deletion.
- **R2 (unchanged):** timeline moments (phase 3), "lanes, crossed" (phase 4) and the carry-forward ritual (P3). What
  `/s/` shows of a session (card question 2) is currently the decided action items only. The share page is unchanged and
  shows no cards.

## Gates

On `37a7936`, in CI:

- `cargo fmt --check`;
- clippy for ssr and for hydrate/wasm32, with `-D warnings`;
- `cargo test --features ssr --lib` (493 passed locally);
- the integration tests;
- `scripts/lint_design.sh` (zero);
- "No stalled requests";
- the full e2e list, with the new spec.

`origin/main` `1018903` is merged into the branch (Law 10).

The new tables are created by the baseline schema at the first start of the new build. No migration runs.
