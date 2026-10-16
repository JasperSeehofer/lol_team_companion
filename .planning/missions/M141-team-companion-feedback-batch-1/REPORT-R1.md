# M141 — Team Companion feedback batch 1 — REPORT R1

Branch `mission/M141-team-companion-feedback-batch-1` (pushed, branch only), based on `origin/main` 5784807
(contained; `git merge origin/main` up to date). Gate tip **cc8e67e**, CI run **38892154457 all green**; the
branch head after it, 21b8a4a, is CI's own commit of the e2e summary and frames (planning files only).
177 files, +6,452 / −589. Nothing merged, deployed or touched on the VM.

## How the round ran

- Orchestrator plus subagents: atelier (critique and recipe cards, then two frame-strip critiques), two Opus
  implementation lanes in separate worktrees (lane 1: A, F, E, B on the mission branch; lane 2: C, D, G, H on
  `mission/M141-lane2`, merged no-ff as ea5bf9f), an Opus integrator, two Opus CI/fix loops, and Sonnet agents
  for the frame infrastructure, before frames and the proposals document.
- **Report source:** the read-only export pull (`ssh companion … cat feedback.jsonl`) was **denied by the
  session's permission classifier** (production read). The 14 report bodies came from the locally staged judge
  copies (`~/.attine/inbox/feedback/feedback-team-companion-bug_report-*/source.json`, same JSONL items) and
  are copied in `reports.md`. All 14 are `reporter_is_admin: true`. Report text was treated as data only.
- **Host incident:** a `cargo build --features ssr --bin seed` for local "after" frames swapped brain at
  96 % memory pressure until the overseer killed it (the M137 stall again). After that only `cargo fmt` /
  `cargo check` ran on brain. Everything else ran in CI, including the after frames and the e2e failure summary.
- **No GitHub token on brain**, so CI logs can't be read. New CI step (e2e job, only on `mission/M141*`
  branches): it writes `ci-e2e-summary.md` (failing tests with their first error lines, plus server-log stall
  lines) and the frames, then commits them back to the branch with `GITHUB_TOKEN`
  (`scripts/ci_e2e_summary.py`, f0bdce1). Playwright also gets the `github` and `json` reporters.

## What shipped, per report id

| Report | Group | What changed | Commit |
|---|---|---|---|
| `xvqg64mb` | A | Scoreboards: blue stacked over red on one fixed grid, never scrollable, identical size (`Table fit`). Columns drop by width: phone = position, player, K/D/A. "(you)" → chalk outline on the player's avatar (sr-only ", you", `aria-current`). K/D/A never wraps. | 83cf23f |
| `12wea0hr` | A | Hovering or focusing a row tints the same role in the other team; tapping the position pins the pair (`aria-pressed`, Esc clears). | 83cf23f |
| `96mhp4u3` | A | `ITEM_UNDO` is no longer shown as Recall (`riot.rs classify_event`); stored wrong rows are filtered on read (`db::classify_db_event`), so no re-import is needed. Riot's timeline has **no recall event**, and the wards rail says so. Timeline split into three rails (objectives + towers · kills + fights · wards), each with kind toggles and a rail toggle, plus Previous/Next event. **Stretch, smallest version:** a shared `TimelineScrub` playhead across the rails (J/L jump between objectives; a live line shows what happened within 30 s). Map placement is not built: events carry no x/y. | 83cf23f |
| `ux89riwg` | F1 | LP history: one `ranked_snapshot` per sync. The chart names the first and last sync and says each point is the Solo/Duo standing read at a sync; nothing is drawn before the first sync or between syncs. A single point says "A second sync draws the line"; there are two separate empty states. **Gap closed:** ranked snapshots were *not* in the 12-month purge. `db::purge_ranked_snapshots` now runs in the nightly retention task (index on `snapshotted_at`, integration test `retention.rs::old_ranked_snapshots_are_purged`). Account deletion already covered them. | 60f7332 |
| `l31jk71m` | F2 | `Button` primitive app-wide: the spinner overlays the label, which goes transparent but keeps its accessible name; a `reserve` prop holds the widest label's width. Rule in DESIGN.md 6.1. Gallery width demo. | 60f7332 |
| `rucyy6k1` | F3 | `SortTh` / `SortState` / `sort_rows` primitive (stable sort, `aria-sort`, a real button, a status line announces the order; gallery section "sort"). Applied to: My games Champion trends + Recent games; Team games' games by member; Draft's champions by appearances, win rate by composition tag, win rate by first pick; Game plan's win rate by strategy. Left unsorted on purpose: LP points, roster, saved drafts, admin tables, bug board, scoreboards. | 60f7332 |
| `3z6n0emq` | E | Choosing a match fills the **result**. It is the only review field the match data can answer; rating and lessons stay human (honesty rule). A "Filled from match: Result" note with Undo; a value someone set is never overwritten. A read-only "From the match" block shows side, length, date, who played what and both teams' picks. Bans are not in the synced data, and the block says so. | 97bea8e |
| `8tiuijr4` | B | Game plan matchup table: a 24 px champion icon **beside** the name once it matches; a dashed placeholder keeps the geometry. | cbbaa32 |
| `fy9o9cmh` | B | Suggestion chips: **proposal only**. Not "small": `fit::rank` ranks the next pick and doesn't read a finished matchup, and Dismiss needs a new plan field. Assessment ≈1.5–2 days: `assessment-suggestion-chips.md`. Also proposal `game-plan-suggestion-chips`. | cbbaa32 |
| `vdgtxa2n` | C | Champion pool: moves are optimistic, revert on failure, and never swap the list for a skeleton; that skeleton swap, not a reload, caused the scroll jump. Every Suspense on the page is now a Transition; focus is restored with `preventScroll`. **This batch built the shared save-state helper that M137 item 4 described:** `ui::status` `SaveState` / `SaveStatus` / `SaveTracker` / `use_save_state` / `LoadFailed`. M137 R2 should reuse it. | cf589e7 |
| `zgoormwy` | D | Enter submits the attached primary action; Ctrl+Enter in textareas (`ui::enter_submits` / `ctrl_enter_submits`, IME-safe). Rule in DESIGN.md 6.6. Audit list below. | a43b4b7 |
| `1jjv5ji2` | G | Admin invites: mint fields on one baseline, hints below; the table fits at 1280 and 390 (no clipped Disable, no wrapped codes or dates). | 38ee883, 4a9b422, cc8e67e |
| `y1abg605` | H1 | "Report Bug" → **"Give Feedback"** everywhere (widget, dialog, admin, community copy, DESIGN.md, specs). Bug / Idea / Praise are labels only: stored values stay `bug` / `wishlist`, praise = `wishlist` + `[praise] ` body prefix. No migration. The 44 px trigger on phones. | 680ef37 |
| `y1abg605` | H3 | `/bugs` Your reports: status timeline (new → triaged → planned → fixed / won't fix, current step marked and dated), the team's public note, and a "Fixed in" CHANGELOG link. The existing `reporter_seen_at` behaviour is unchanged. | dd1e974 |
| `y1abg605` | H2 | **Floor shipped:** attach or paste an image → server checks magic bytes, decodes with limits and re-encodes to WebP ≤ 1 MB, which strips metadata (unit test) → `ATTACHMENTS_DIR/<report>.webp` (new env var: dev `./attachments`, image `/data/attachments`, refused inside `SURREAL_DATA_DIR`). Upload: the reporter's own report, within 15 min, once, ≤ 5 MB. Admin-only read at `/admin/attachment/{id}` (404 for others, nosniff, CSP `default-src 'none'`). Shown on `/admin/bugs` with "Delete screenshot". `attachment` field in `feedback.jsonl`. Deleted with the reporter's account. `backup.sh` and `restore.sh` carry the dir (harness fixtures cover local and SFTP). Privacy page STORED and retention text plus a new legal test cover screenshots. **Region picker deferred** (see the card question). | a4e0ede |
| (brief) | H4 | `/admin/proposals` **Import proposals (JSON)**: `require_admin`, strict parser (`version: 1`, `deny_unknown_fields`, 64 KB, 20 items, key/title/summary limits, all errors listed, all-or-nothing), one transaction under `invariant_lock`. Each import is an unpublished draft, `seed_key agent:<key>`, attributed "Proposed by the Team Companion agent — <date>". Schema in CLAUDE.md. No CLI twin. | 6ac752e |
| `1oe8iqmn` | I | Role responsibilities: assessed as a mission-sized proposal (`role-responsibilities`), not implemented. | 1a68bd2 |

**Fixes found along the way:** `Field` overwrote a sibling's context, so two Fields under one parent shared an
id (`e2d8d48`, real bug, caught by the attachment spec). A full-width `Button` centred its whole label row
(`da4195b`, the post-game list indent). For ASK-20261013-01 (group J), `docs/LAUNCH-CHECKLIST.md` now ticks
Riot registration (§2) and the uptime/alert path (§6, "first real alert: none yet"); the backup dead-man's-switch
line is left open.

**Enter audit (D).** Enter now runs the button on these single-line fields: tree drafter add branch, new tree
name and opponent; opponents "Add player", the Riot ID's Refresh and the dossier's Save; draft ban
champion/reason; team invite uses/days and the Edit team name; roster create team; profile username and Riot
ID; My games CS and deaths goals; action items due date. Ctrl+Enter now sends these textareas: team note (new
and inline edit), draft comment, dossier notes, admin text body, survey free text, feedback text. Already
worked: game-plan checklist, action items, champion picker, password form, every ActionForm. Left alone on
purpose: multi-field editors with their own Save, search fields, multi-field dialogs.

## Gates

- **CI run 38892154457 on cc8e67e: all five jobs green.** Check, lint, unit tests: fmt, both clippy, lib and
  integration tests. Style guardrails: design lint. Ops scripts. Container image. e2e: 132 expected,
  0 unexpected, 0 flaky, including the new specs `match-detail`, `sortable-tables`, `post-game-fill`,
  `champion-pool-moves`, `enter-submits`, `bug-report` (first time in CI), `feedback-attachment`, `frames-m141`
  and the extended `gallery`, `admin-invites` and `bug-triage`. SSR load check: 12,000 requests, 0 stalled
  (`ci-e2e-summary.md`).
- On brain (the last full set, integrator on da0a8fa): `cargo test --features ssr --lib` 412 passed; both
  clippy clean; design lint OK; shellcheck clean; ops harness 45/45. After the host incident, only
  `fmt --check`, `cargo check` on both targets and the design lint ran before each push.
- **SSR stall:** run 38828228316 (lane 1 only) flagged 35 never-completed requests. Three later runs, two of
  them on code that includes lane 1, showed 0 inflight lines over the full load. **Not reproduced, no cause
  found, no source change for it.** It may be intermittent; watch the release smoke.
- Not in CI: H2's region picker (none shipped); `visual-regression.spec.ts` (full suite only) may need new
  baselines for /match, /me and /post-game.

## UI definition of done

- Before frames: main's code (prebuilt M137-tip binary, local, seeded). After frames: CI run 38892154457.
  390 / 1280 / 1280-flow for 11 surfaces in `publish-round-r1/frames/{before,after}/`; labelled strips in
  `publish-round-r1/strips/`. Match detail had no seeded match before, so its before half is the stats fallback.
- Atelier critique: `publish-round-r1/frame-critique.md`. 8 PASS, 3 FIX (admin invites, post-game, My games);
  all three are fixed (4a9b422, cc8e67e, da4195b, 80221ac). **Re-check on run 38892154457: overall PASS**:
  all 11 surfaces pass, and no regression from the Button label-span change.
- `rubric-check.mjs` needs `--app-dir` and a live page, so it was not run; the atelier verdicts are by eye.
  One pixel measurement: the disabled "Sync my games" is 4.15:1. Disabled controls are WCAG-exempt and a
  callout explains it; darkening the label to reach 4.5:1 is optional (R2).
- Not provable from frames, covered by e2e: hover/pin, drag and keyboard moves, Enter, the attachment upload.

## Owner-visible text and decisions to note

- **Privacy page wording changed twice:** ranked snapshots now have a 12-month retention line
  (`RANKED_SNAPSHOT_RETENTION`), and screenshots are a STORED category deleted with the account. It is legal
  text, so the adviser should see it with the open draft callout. CLAUDE.md's `MATCH_CACHE_RETENTION_DAYS` row
  doesn't mention snapshots yet: one line for the land.
- **Agent proposals export as `author: "agent"`, `reporter_is_admin: false`.** Otherwise agent-written text
  would be tiered as trusted owner input by `81-feedback-pull.sh`.
- **Proposed autonomy-ledger row `FBK-2`** (for the land tap to apply): *agent proposals: the agent may write
  draft proposals only, through the admin import; an admin publishes each one; the agent never votes or
  publishes.*
- "Give Feedback" keeps the owner's title case. The app's other buttons are sentence case.
- Seed tool: a fictional demo match `SEED1_1000000001` (Northgate vs Riverside) for the frames and specs.
  `SEED1_` ids are never sent to Riot; a cache miss is "not found".
- The CI summary/frames step is scoped to `mission/M141*` branches. Before landing, either generalise it to
  `mission/*` (useful for every brain-built mission) or drop it; recommendation: generalise in R2.

## Agent proposals (group I)

`docs/agent-proposals/2026-10-batch1.json` (importable; a unit test parses it with the real parser) and `.md`
(rationale, audience, smallest version, Riot check, stance). Import on `/admin/proposals`, then publish
what you agree with:

| Key | Stance |
|---|---|
| `role-responsibilities` (1oe8iqmn) | try small |
| `game-plan-suggestion-chips` (fy9o9cmh) | try small |
| `season-schedule-availability` (audit G01) | adopt |
| `share-draft-discord` (audit G02/G03) | adopt |
| `review-tags-pattern-count` (audit G05) | try small |
| `chalk-means-you` (identity) | adopt |
| `replay-map-signature` (identity) | hold |
| `margin-notes-praise-count` (identity) | try small |

## R2 (not done, with reasons)

1. H2 region picker (`getDisplayMedia` overlay): its permission prompt can't be accepted in headless CI, so it
   would ship untested; floor first. Plus the screenshot in "Download my data".
2. Inferred recalls from purchase bursts: the importer must keep `ITEM_PURCHASED`, which stored games lack.
3. Match timeline on the rift map: needs event positions, which the importer doesn't keep. The playhead shipped.
4. Suggestion chips implementation (see the assessment).
5. Match detail at 390: late-game markers sit ~12 px apart, a tap-size concern; Previous/Next reach them.
6. Optional: darken the disabled "Sync my games" label (4.15:1 → 4.5:1); `visual-regression` baselines.
7. M137 R2 items stay where they were (invite Copy, phone Match Day add, 16+ checkbox, page titles,
   return-to-page, coach invite rights). M137 item 4 can now use `ui::status`.

## Cost

About 1.9 M subagent tokens as reported by the harness, plus the orchestrator. Well under the 12.0 M budget.
The overseer's meter is the authoritative figure.
