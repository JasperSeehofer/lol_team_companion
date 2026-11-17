# M165 Review together: server API for the UI builder

Server layer only; no page component exists yet. Code:

- Types and pure rules: `src/models/review_together.rs`
- DB logic, sealing, presence: `src/server/review_together.rs` (unit tests in `src/server/review_together/tests.rs`)
- `#[server]` fns: `src/pages/review_together.rs` (add the page component below them; `#[server]` fns must stay above it)
- Integration tests: `tests/integration/review_together.rs`

Every fn starts with `authz::authed()` + `ctx.require_team()`; the team never comes from the client. Session, match, card and proposal ids are checked as records of the caller's team: an unknown or foreign id is "Not found". Rule violations come back as `ServerFnError` with a message that is safe to show (toast it). Writes take `Bucket::ReviewWrite` per user (120 burst, 2/s); `get_session_state` is not rate limited.

Ids are full record ids as strings (`review_session:abc`, `review_entry:xyz`, `review_proposal:...`, `scheduled_match:...`); bare keys are accepted too. Prompt keys: `decided`, `to_plan`, `off_plan`, `next_time` (`Prompt::key()`, labels in `Prompt::label()`, anonymity caption `ANONYMOUS_CAPTION`).

## Server functions (`crate::pages::review_together`)

| Fn | Who | Phase | Returns |
|---|---|---|---|
| `get_session_state(session_id: String, rev: Option<i64>)` | member | any | `SessionPoll` |
| `open_session(match_id: String, game_number: Option<i32>, match_riot_id: Option<String>)` | member (opener facilitates) | n/a | `OpenedSession { session_id, created }` (existing session when `created == false`) |
| `list_match_sessions(match_id: String)` | member | n/a | `Vec<MatchSessionLink { game_number, session_id, phase }>` |
| `take_over_facilitation(session_id)` | leader or coach | not closed | `()` |
| `upsert_entry(session_id, prompt: String, entry_id: Option<String>, text: String)` | member, own cards only | writing | card id (`String`) |
| `delete_entry(session_id, entry_id)` | own card or pass | writing | `()` |
| `pass_prompt(session_id, prompt: String, passed: bool)` | member | writing | `()` |
| `start_timed_round(session_id, minutes: Option<u32>)` | facilitator | writing | `()` |
| `add_minute(session_id)` | facilitator, timed round running | writing | `()` |
| `reveal_now(session_id)` | facilitator | writing | `()` |
| `start_deciding(session_id)` | facilitator | revealed | `()` |
| `plus_one(session_id, entry_id)` | member (own cards too) | revealed, deciding | `bool` (on now) |
| `merge_entries(session_id, target_id, entry_ids_json: String)` | facilitator | revealed, deciding | `()` |
| `unmerge(session_id, entry_id)` | facilitator | revealed, deciding | `()` |
| `create_proposal(session_id, source_entry: Option<String>, text: String)` | member | revealed, deciding | proposal id |
| `decide_vote(session_id, proposal_id)` | member | revealed, deciding | `bool` (on now) |
| `set_accepted(session_id, proposal_id, accepted: bool)` | facilitator | revealed, deciding | `()` |
| `set_owner(session_id, proposal_id, owner: Option<String>)` | facilitator | revealed, deciding | `()` |
| `delete_proposal(session_id, proposal_id)` | facilitator | revealed, deciding | `()` |
| `close_session(session_id)` | facilitator | revealed, deciding | generated review id (go to `/post-game?review_id=`) |
| `get_review_session_for_review(review_id)` | member | any | `Option<ReviewSessionView>` (None: review written alone) |
| `get_session_pattern_texts()` | member (empty without team) | n/a | `Vec<SessionPatternTexts>` (already wired into Patterns on `/post-game`) |

Notes per fn:

- `open_session`: `game_number` is 1..=game count of the match's format (`None` = 1; a Bo1 is game 1). Allowed before the match starts. One session per (match, game) (unique index); a second open returns the first and does not change the facilitator. `match_riot_id` is optional (letters, digits, `_`, ≤ 40) and goes onto the generated review.
- `upsert_entry`: `entry_id = None` creates (≤ 3 cards per prompt; writing a card withdraws a pass on that prompt); `Some(id)` edits one of the caller's own cards (anyone else's id is "Not found"). Text: control characters become spaces, whitespace collapses, trimmed, 1..=280 characters. Use `maxlength=280` on the input. Autosave = call this; keep the returned id for later edits.
- `pass_prompt(passed=true)` on a prompt with cards is refused ("Delete your cards for this prompt first, then pass."); `passed=false` takes a pass back. Both are no-ops when already in that state.
- `start_timed_round`: minutes default 3, clamped 1..=30. It only moves `writing_closes_at` earlier (an async member's time is cut only when the round ends). `add_minute` adds 60 s, never past the async deadline (match start + 24 h).
- `merge_entries`: `entry_ids_json` is a JSON array of card ids, 1..=50. All cards (target included) must be cards of this session and of one prompt. The root is the target's cluster head (or the target); cards that headed other clusters bring their members. Every card stays visible with its words; members carry `cluster = Some(root id)`, the root `None`. `unmerge` on a member removes it; on a root it dissolves the cluster.
- `create_proposal`: `source_entry` is optional (a card of this session); the text is the member's edit (same cleaning, 1..=280). At most 30 proposals per session. No proposer is stored anywhere.
- `decide_vote`: 3 per member per session; the 4th is refused with a message telling them to take one back.
- `set_accepted`: at most 3 accepted. The default suggestion is `ProposalView.suggested` (top 3 by votes, at least one vote, ties by id); the facilitator still has to accept.
- `set_owner`: a current member's username (case-insensitive; stored as the member writes it) or blank/None for nobody. Anything else: "Choose an owner from the team." The member list for the chips: `crate::pages::action_items::get_action_items().members` already returns usernames, or add a call; the server re-checks.
- `close_session`: one transaction. Writes the `post_game_learning` (`created_by` = facilitator, `what_went_well` and `improvements` empty, `action_items` = accepted texts in vote order, stamps: `scheduled_match`, `series` when the match has one, `game_number` only for a Bo3/Bo5 as M159 does, `match_riot_id` when given), one `action_item` per accepted proposal with `source_review` = the review and `assigned_to` = owner (an owner who left meanwhile is dropped), and closes the session with `voices`/`members`. Re-saving that review in the editor creates no items (tested).
- The owner check now also guards `/action-items`: `create_action_item_fn` and `update_item` refuse a non-member (an edit may keep the stored owner even if they left); `restore_item` (Undo) drops a non-member owner.

## Poll and state payload

`SessionPoll` (serde tag `kind`):

- `Unchanged { rev, present }` when the `rev` sent is the session's current one. `present` = usernames that polled in the last 10 s, alphabetical. Presence is NOT part of `rev` (choice documented in the module): people coming and going never force a full payload, and the strip refreshes from every `Unchanged`.
- `Full(Box<SessionState>)` otherwise. Send `rev: None` on first load.

Polling plan from the mission: every 2 s while visible and not closed, 10 s while hidden, stop when `phase == Closed`. Use `server_now` vs `writing_closes_at` for the countdown (never the browser clock alone).

`SessionState` fields: `id`, `rev`, `phase` (`writing|revealed|deciding|closed`), `match_id: Option`, `match_deleted`, `game_number` (1 for Bo1), `series_id`, `writing_closes_at`, `server_now` (RFC 3339 UTC), `timed_round`, `facilitator_name`, `i_am_facilitator`, `can_take_over` (viewer leads/coaches, is not the facilitator, not closed), `review_id` (after close), `written_count` / `member_count`, `mine: Vec<OwnEntry {id, prompt, text, passed}>`, `cards: Option<RevealedCards>`, `proposals: Vec<ProposalView>`, `decide_votes_left`, `quiet: Option<u8>`, `voices: Option<Voices {voices, members}>` (`Voices::label()` = "5 of 5 voices"), `present`.

`RevealedCards { decided, to_plan, next_time: Vec<AttributedCard>, off_plan: Vec<AnonymousCard>, suggestions: Vec<ClusterSuggestion {prompt, entry_ids}> }`

- `AttributedCard { id, text, author_name, plus_ones, i_plus_oned, cluster }` (author resolved at read time; "Former member" after an account deletion).
- `AnonymousCard { id, text, plus_ones, i_plus_oned, cluster }`: no author field, no "mine" flag, no time. Render no name, no avatar, no "yours" marker, and no `data-*`/`aria-*`/`title` holding an author for these.
- Each group is in `order_key` order (random, written at the reveal). Render in the given order; never sort by anything else.
- `suggestions` are filled for the facilitator only.

`ProposalView { id, text, source: Option<entry id>, votes, i_voted, accepted, suggested, owner }`, sorted by votes desc then id. No proposer.

`ReviewSessionView { session_id, cards: RevealedCards (no suggestions), accepted: Vec<AcceptedItem {text, owner}>, voices }` for the read-only section on `/post-game?review_id=`.

## Phase rules

- Stored phases: `writing` -> `revealed` -> `deciding` -> `closed`. `writing` with `now > writing_closes_at` IS revealed (computed at read time). The first read or write after the deadline writes the reveal (random `order_key` per card, `phase`, `revealed_at`, `rev + 1`) in one transaction under `invariant_lock`; idempotent, survives restarts, no background task.
- Writing closes at match start + 24 h (or 24 h from opening when that has passed). A timed round moves it earlier; `reveal_now` sets it to now.
- Sealed while writing: a member's state holds only their own cards and passes (`mine`), plus `written_count` (members with at least one card or pass over all four prompts, never per prompt; the facilitator counts like any member) and `member_count` (current team members). No other text, no per-prompt counts. The facilitator sees no texts either.
- After the reveal: `mine` is empty for everybody, cards are locked (edit, delete and pass are refused with "The cards are revealed; they can no longer change."). +1, merges, proposals and decide votes are open in `revealed` and `deciding` alike; `deciding` is a facilitator-set step for the group's view.
- `quiet`: only for the facilitator, only while writing: the number of current members with no non-pass card in each of the team's last 3 closed sessions (accounts newer than the oldest of those sessions are left out). `None` with fewer than 3 closed sessions or nobody quiet. Text: `quiet_text(n)`. Show it as a muteable `NudgeLine`-style line (that needs a `NudgeKind`, UI work).
- `voices`: written at close: current members with at least one card (a pass is not a voice) of current members.
- `rev` bumps on every write that changes what any member sees (cards, passes, votes, proposals, accepts, owners, merges, facilitator, deadline, phase, review link, the match being deleted). A team member joining does not bump it (the count updates on the next change).
- Match deleted: the session keeps working with `match_id = None`, `match_deleted = true`; the review it closes into has no match stamp. Review deleted: the session's `review_id` becomes `None`.

## Privacy rows (done)

`STORED` "Your team" sentence and `LISTED` tables in `datenschutz.rs`; `USER_LINKS`: `review_session.facilitator` Tombstone, `review_entry.author` Tombstone, `review_vote.user` Delete (plus `review_proposal.owner` unassigned by name in the user's team); `DELETE_TEAM` deletes all four tables; export sections `review_cards_written` (prompt, text, passed, anonymous, session_date) and `review_votes` (kind, created_at), and facilitated sessions in `team_records_created`.

## Not done here (UI round)

Page component and route (`/review-together?session=` suggested: no `:param`, see code-digest), nav/Match Day entry, `NudgeKind` for the quiet line, CHANGELOG `Introduce:` line, ARCHITECTURE/DESIGN text, e2e spec.
