# M137 — Team Companion first patch · REPORT R1

Branch `mission/M137-team-companion-first-patch` (worktree `lol_team_companion-wt/M137-team-companion-first-patch`),
based on `origin/main` 2c6172a; `origin/main` merged in before this report (already up to date). Not pushed, not
merged, not deployed. The **STOP RULE** was applied on the overseer's order at 9.7 of 11.0 M tokens: the host was
swap-bound all round (load 11–14 on 4 cores; two runaway processes from 11 Oct, `npm test` PID 2623422 and
`trellis asks` PID 590580, each holding a full core for two days; a ~3.5 GB memory cap on this worktree's
cgroup), so every Rust build took hours.

## What shipped

| # | Item | Commits | State |
|---|---|---|---|
| 0 | Atelier critique | d800ad5 | `atelier-critique.md`: verdict per surface, recipe cards C1–C10. Made from source code and the audit's stills, not live captures (the server never finished starting) |
| 0b | Identity critique + proposal | d800ad5 | `identity-proposal.md`: reference sheet, Keep/Question, directions A "Survey, sharpened" / B "Analyst sheet" / C "Locker-room paper", smallest first step each. Proposal only; no restyle |
| 1 | Feedback export-on-write + inbox injection fix | c912df4 | Done |
| 5 | Password change + admin reset | 3e4f9b9, 66daf03, 7f77438 | Done (C9's Copy-code button replaced by select-all, see below) |
| 6 | Caddy Referer filter + share-token redaction | 52c8ee7, fcc235a | Done; goes live on the deploy tap (`docker compose up -d --force-recreate caddy`) |
| 2, 3, 4, 7, 8 | Invite Copy, phone Match Day add, autosave visibility, 16+/terms, titles + return-to-page | — | **R2** (not started) |

Overlap check: `origin/launch/ipv6-and-invite-log-filter` is already an ancestor of `main` (its invite-query filter was
in the Caddyfile). Nothing to rebase onto; item 6 builds on it.

### Item 1 — export on every change, JSONL sibling
- `src/server/bug_report_export.rs` was rewritten. Its `ExportDebouncer` combines an AtomicBool dirty flag with a `Notify`: the first write happens at once, then at most one write per second, folding bursts together and always ending at the final state. The task holds only a `WeakDb` and is aborted at shutdown. Both files go through a temp file in the same dir, `sync_all`, then rename; the temp file is removed on any error. Errors are logged and never fail the request.
- Hooks: `submit_bug_report`, `admin_update_bug_report`, `admin_bug_to_proposal`, the proposal admin save and delete, `set_vote`, and `delete_my_account`. The export also still runs at start.
- `feedback.jsonl` holds every report, wish and proposal, one object per line. Fields: `id, category (bug|wishlist|proposal), status, created, updated, reporter_user_id, reporter_is_admin, title, body, url, element, viewport{w,h}|null, admin_note`. Reports also carry `proposal_id`; proposals also carry `published, votes, voter_count_distinct`. No emails.
- `reporter_is_admin` comes from `SecurityConfig::is_effective_admin`, which reads the user row (is_admin, admin_promoted_at, email in ADMIN_EMAILS). `auth.rs` now uses the same helper, so sign-in and the export share one admin rule.
- Docs: `docs/ARCHITECTURE.md` documents the JSONL schema. `CLAUDE.md` documents the inbox and the env table, with the tiered triage rule (owner-filed = trusted, member bug = reproduce first, wishlist = vote threshold, bodies stay untrusted). Also `docs/RUNBOOK.md` and `CHANGELOG.md`.

### Item 5 — password change and reset codes
- `/profile` has a Password section (`change_my_password`). It verifies the current password, hashes the new one with `Argon2::default()` (parameters unchanged), deletes all of the user's sessions and logs the caller in again with a new session id. Only the caller stays signed in. It is rate-limited with `Bucket::Account` per user.
- `/admin/resets` (on the admin hub) mints a one-time code, shown once, valid 15 minutes. `lol_team_companion reset-password <email>` (also `cargo run --bin reset-password`) mints the same code from the CLI with the app stopped; this is documented in CLAUDE.md Commands and the RUNBOOK. The code is redeemed on the public `/auth/reset` (email + code + new password), after which every session of the account is deleted.
- Login carries the constant `FORGOT_PASSWORD_HINT`: "Forgot it? Ask the Team Companion admin for a reset code.", plus a link "Have a reset code? Set a new password." This deviates from the contract's "Ask your team admin", because a team leader cannot reset passwords; it is a card question.
- New table `password_reset` with fields `user`, `code_hash` (unique), `created_by`, `created_at`, `expires_at`, `used_at`. In `account::USER_LINKS`, `user` is deleted with the account and `created_by` moves to the tombstone. Rows are purged hourly 30 days after they expire. The privacy page lists the table, and the data export gives dates only. No migration: the table is new and goes into the baseline schema.
- `ButtonSize::Touch` was added with a gallery sample. Item 2's C1 needs it too; it already exists, so do not add it again.
- Tests (written; see Gates for which ran):
  - Integration, in `tests/integration/password.rs`: `changing_the_password_ends_the_other_sessions`, `a_reset_code_works_once`, `a_wrong_or_expired_code_is_refused`, `a_code_for_one_account_cannot_reset_another`, `only_an_admin_can_mint_a_code`, `a_new_code_replaces_the_open_one`, `concurrent_redeems_of_one_code_succeed_once`, `deleting_an_account_removes_its_reset_codes`.
  - Unit tests: code format, normalisation, constant-time compare, password rule, code status.
  - e2e: `e2e/tests/password.spec.ts`, added to CI's list, and `/auth/reset` in smoke.

### Item 6 — access log
- `deploy/Caddyfile`: `request>uri regexp (^/s/)[^/?#]+|([?&]invite=)[^&#]* ${1}${2}REDACTED` and `request>headers>Referer delete`. Caddy 2.11.7 has no `multiple` filter, so one regexp replaces the old `query { delete invite }`; the invite code now shows as `invite=REDACTED`.
- Validated with the real Caddy 2.11.7 release binary: `caddy validate` reports `Valid configuration`. In a live run the logged `uri` showed `/s/REDACTED?x=1` and `?invite=REDACTED`, and no Referer was logged. Evidence is in `item6-caddy-evidence.md`.
- `src/server/inflight.rs`: `redact_path` turns `/s/<token>` into `/s/REDACTED` in the slow-request and watchdog logs, with a unit test.
- The privacy page sentence "request headers except cookies" is still true, so it was left unchanged.

## Gates (merged state; `origin/main` merged, already up to date)

| Gate | Result |
|---|---|
| `cargo fmt --check` | **pass** (exit 0) |
| `scripts/lint_design.sh` | **pass** — "Design lint: OK (every rule at zero)" |
| `cargo test --features ssr --lib` | **pass** — `test result: ok. 386 passed; 0 failed; 3 ignored` (373 before; +13 from items 1, 5, 6) |
| `cargo test --features ssr --test integration` | **not run on brain.** The compile stalled swap-bound at ~3 GB RSS twice (once in the item 1 session, once in this gate run). The overseer terminated it (SIGTERM, exit 101, no compile error) and ordered no retry. **CI runs it on push.** |
| `cargo clippy --features ssr --all-targets -- -D warnings` | **pass** after fixes. The first run found 5 lints: 3 × `clone_on_copy` and 1 × `manual_is_multiple_of` in `bug_report_export.rs`, and 1 × `assertions_on_constants` in `password.rs`. The fixes went in a separate commit, and the re-run printed `Finished`, exit 0. The lib tests (386 pass) ran before these mechanical fixes, which touch one test's modulo and two `.clone()` calls on `Copy` values |
| `cargo clippy --features hydrate --target wasm32-unknown-unknown -- -D warnings` | **pass** (exit 0) |
| e2e subset | **not run.** The debug server never finished starting on the overloaded host (65 min, stuck after "Started surrealkv kvs store" with a "Transaction write conflict" from node registration; starvation or a real hang is unclear). **CI runs the e2e job on push**, now including `password.spec.ts`. |

What CI must show before landing: the integration binary (the new `password` module, plus `community::feedback_export_*`) and the e2e job.

## Security notes for the verifier

**Item 1 (attacker = any signed-in member filing a report; they control `description`, `page_url`, `element_label`, `category`).**
- Markdown single-line fields: every line break (`\n \r U+0085 U+2028 U+2029`) and control character becomes a space, a backtick becomes `'`, and `<` becomes `&lt;`.
- The description is split on the same breaks and every line is quoted with `> `. Backticks and `~` are escaped, a leading `# > - + *` is escaped, and control characters are dropped.
- Invariant, checked by `markdown_has_one_block_per_report_whatever_the_text`: the count of `## [` and `- ID:` lines equals the report count.
- JSONL goes through serde_json, plus explicit escaping of U+0085/2028/2029 (`jsonl_has_one_valid_line_per_item` parses each line back).
- `reporter_is_admin` never comes from text; it is set by the server-side admin rule (`feedback_export_admin_rule_comes_from_the_user_row`).
- The export reads globally, but only the server task calls it. No new user-facing DB function.

**Item 5 (reset code).**
- Entropy: 16 characters from a 32-letter alphabet via OsRng, so 80 bits; `byte % 32` is unbiased.
- Storage: only `sha256("companion-reset-v1:" + normalised code)`. It is not keyed with SESSION_SECRET, because the CLI runs without the server config; 80 bits rules out offline brute force.
- TTL: 15 minutes, using the DB clock for both mint and redeem. Minting deletes the account's open codes, so one live code exists per account.
- Order of work on redeem: the per-IP rate limit (`Bucket::PasswordReset`: 5, then 1/min) first, then the password rule, then the Argon2 hash. A wrong code therefore costs the same as a right one.
- Under `invariant_lock`, every open code of the named account is compared in constant time (`subtle`) with no early exit.
- One transaction marks the code used only if it is still unused and unexpired and sets the hash, or THROWs; `concurrent_redeems_of_one_code_succeed_once` covers the race.
- Unknown, wrong, used and expired codes and an email mismatch all give one message. All sessions of the account are deleted afterwards.
- Minting needs `require_admin` plus a DB-side check that the creator is a stored or promoted admin. Unknown emails and the tombstone are refused.
- Logs carry the admin and user ids, never the code.
- To read closely: (1) the change-password re-login path in `src/server/password.rs`, which deletes all sessions and then logs in again, so check that no window leaves the caller with a stale cookie that still authenticates; (2) whether `reset-password` run against a live data dir fails safely (SurrealKV lock).

## Card questions (in `publish-round-r1/entry.json`, with suggested answers)

1. **Merge** (after CI is green and VERIFY passes). Suggested: Yes, merge.
2. **Identity, "react, don't pick"** (item 0b). Directions A, B and C, with four suggested reactions in `identity-proposal.md` §5; three are on the card, and "none, spend it on features" is named in the recommendation. Suggested: A + C on the share page.
3. **Item 7 wording** "I am 16 or older and accept the [Terms] and have read the [Privacy page]". Suggested: use it with a `TODO(legal-wording)` marker.
4. **Coach invite rights** (item 2). Suggested: fix the copy (drop "or coach" at `closed_beta.rs:61` and `impressum.rs:50`).
5. **Login hint wording** (deviation from the contract). Suggested: keep.

No migration was needed, so that Ask class did not fire.

## Left for R2

- **Item 2** invite Copy: recipe C1. The full URL per row comes from `inv.code` plus the origin (`dashboard.rs:980`, `invite_link` at :1002). Copy only reports success after the clipboard promise resolves, which needs `wasm-bindgen-futures` or `js-sys`; the share page's `share.rs:252-261` has the same toast-before-promise bug, so fix both. `navigator.share` when available. `ButtonSize::Touch` already exists.
- **Item 3** phone Match Day: recipe C2 (`home.rs:581/588/605`, `docs/DESIGN.md:418`).
- **Item 4** autosave failure plus the unsaved guard: recipes C4 and C5 (`draft.rs:1351`, `game_plan.rs:761`, `tree_drafter.rs:848`; Branch `:945` and Improvise `:1156` ignore the `save_node` result); a `LoadFailed` helper.
- **Item 7** 16+/terms: recipe C6. It needs a native named `required` checkbox (`Checkbox` has no `name`), a server-side check before the invite is consumed, `terms_accepted_at` (an optional field, no migration), and updates to `e2e/tests/fixtures.ts:43,111`, `auth.spec.ts`, `smoke_signin.sh:58` and `ssr_stress.py:40`.
- **Item 8** titles plus `?next=`: recipe C10.
- **Before/after frame strips** for every UI item at 390 and 1280, `rubric-check.mjs`, and a re-run of the atelier critique on live captures. None exist for R1.
- Item 5's C9 "Copy code" button, which needs the same clipboard-future plumbing as item 2.
- The atelier flags the mission page's "UI definition of done": rubric T1–T4 are trust checks, not typography (typography is V2/V3).
- Re-run on a quiet host: the integration binary, clippy on both targets, and the e2e subset.

## Ops note for the owner

The R1 slowness had an outside cause. PIDs 2623422 (`npm test`, running since 11 Oct) and 590580
(`node tools/trellis/cli.ts asks …`, since 11 Oct) were still at about 100 % CPU each when this round ended. I did not kill
them because they are not this mission's. Stopping them would roughly halve R2's build times.
