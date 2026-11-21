# M149 — Team Companion feedback batch 1, round 2 — REPORT R1

Branch `mission/M149-team-companion-feedback-batch-1-r2` (pushed, branch only). Based on the identity land
f86e1dc; `origin/main` 7951e05 merged in at 3e02ca3 and has not moved since, so the green tip is the merged
state (Law 10). Gate tip **6f93871**, CI run **43309188443, all green**. The branch head after it, 1406177, is
CI's own commit of the e2e summary and frames (planning files only). Nothing merged, deployed or touched on the VM.

## Least-confident decisions

- **Inferred recalls have no position test.** A shop visit after 90 s that does not follow the buyer's death
  counts as a recall. Minute frames are too coarse for a fountain check, so a recall without a purchase is
  missed and a rare purchase away from the fountain can show as one. Labelled "Recall (inferred)" everywhere.
- **B was not checked by hand on a real desktop.** CI proves it with a stubbed `getDisplayMedia` only. No headed
  browser on brain; per the 2026-10-16 overseer note it is Jasper's act (card question 4).
- **The ORDER BY/LIMIT pattern may exist elsewhere.** SurrealDB 3 applied `ORDER BY match_id LIMIT` through the
  unique index before the `WHERE` in `db::matches_needing_reimport` (fixed in 97c65d8). Other queries of the same
  shape were not audited (R3).
- **CHANGELOG section date.** The entry is `### Feedback batch 1, round 2 (2026-10-18)` (CHANGELOG.md:11), the
  day the round started; B's fix, the A fixes and the land come in November. The date drives the 180-day
  "New" window for `Introduce:` lines, so it may want the land date.
- **Purchases are stored for all ten players**, not only the viewer's team, because the timeline is cached per
  game for everyone. They are never sent to the browser; only the viewer's team's inferred recalls are drawn.
  Privacy text is flagged in H, not changed.

## Budget

Declared 7.0 M, cap 9.5 M. Subagent meters add up to about 1.3 M tokens across 14 subagents; with the
orchestrator ≈1.6–1.8 M estimated. The metered figure is the overseer's. Wall time about 4 h on 2026-10-18/19
plus about 3 h on 2026-11-12, with a hold in between.

Host incidents: on 2026-10-19 a cold `surrealdb_core` compile ran 3 h+ on a swapping host and the orchestrator
killed it; from then on CI was the compile gate. The mission was held 2026-10-19 to 2026-11-12 (status log). On
2026-11-12 brain dropped to about 1.7–2 GB free disk; agents deleted `target/debug/incremental` in the shared
target dir.

## Items

**0 — atelier recipe cards** (90b6099). Cards for C, D, B and the timeline rail, before any code.

**F — CI write-back for every mission branch.** Already generalised on main by M148: the check-log step
(`.github/workflows/ci.yml:72`) and the e2e summary/frames step (`ci.yml:278`) fire for every `mission/*` push
and derive `MISSION_DIR`. No change needed; its output reached this branch (`ci-e2e-summary.md`,
`ci-check-log.md`, frames). Residue: one `mission/M141*` frames-copy case (`ci.yml:304`), kept as harmless.

**G — retention row** (f0bb138). `CLAUDE.md:132` now names `ranked_snapshot`. The code already purged it:
`src/main.rs:382` calls `db::purge_ranked_snapshots`; `src/server/account.rs:89` (`USER_LINKS`, Delete) and the
`DELETE ranked_snapshot` in the deletion SQL (`account.rs:205`) remove it with the account.

**C — 44 px event targets at 390** (0db55ed). A phone-only "Events near the playhead" row of up to 5 `size-11`
(44 px) buttons, built from the same list as Previous/Next (`near_window`, `src/pages/match_detail.rs:555`).
Track markers get `tabindex=-1` below `md`, set after hydration from `matchMedia` (`match_detail.rs:680`, `:794`).
1280 unchanged. Tests: lib `near_window_takes_the_events_nearest_the_playhead`; e2e `match-detail.spec.ts`
(390 sizes; Next/Previous visit every event).

**D — disabled primary contrast** (464744c). Primary disabled is `bg-ink-3` / `text-muted` / `border-outline`
at opacity 100 (`src/components/ui/button.rs`). The atelier measured 5.52:1 from pixels, up from 4.15:1.

**B — desktop area picker; screenshots in Download my data** (440933b, fix b36d862).
- Gate: `getDisplayMedia` exists, `md` width or wider, `pointer: fine`, decided after hydration.
- The dialog hides, one frame is grabbed, and every track is stopped right after the grab, also on failure.
- Overlay picker with keyboard support; crop maths in `src/models/area_select.rs` (9 unit tests). The crop goes
  into the existing upload path. `src/server/attachments.rs` is unchanged (re-encode, cap, sniff, admin-only
  read): `git diff f86e1dc..HEAD -- src/server/attachments.rs` is empty.
- Race fixed in b36d862: the Dialog's close event fired after a fast refusal had reopened it. One finish path
  now waits for `dialog_closed`.
- Export: `data.bug_report_screenshots = [{report_id, content_type "image/webp", data_base64}]`, read through
  `attachments::path_in` for the user's own report ids, 32 MB total (`EXPORT_SCREENSHOT_BUDGET`,
  `src/server/account.rs:502`), the rest counted in `bug_report_screenshots_omitted`. Documented in
  `docs/ARCHITECTURE.md`.
- e2e (`feedback-attachment.spec.ts`, 4 tests): phone hidden plus floor; desktop button present iff the API
  exists; a stubbed capture checks tracks ended, the overlay, Enter and Esc; a refused capture shows its message.

**A1 — importer keeps positions and purchases; inferred recalls** (9e2645a, fixes 8be9d28, 6f93871).
- New option fields on `match_timeline_event`: `participant_id`, `item_id`, `position_x`, `position_y`; on
  `match_detail`: `timeline_version` (current 2) and `timeline_reimport_failed` (`schema.surql:370–392`).
  No migration, no new table.
- Positions on champion, building and elite-monster kills. Match-V5 has none for `WARD_PLACED` or
  `ITEM_PURCHASED`. Wards keep their creator. Purchases are stored, never sent to the browser.
- `infer_recalls` (`src/models/match_data.rs:168`), computed at read time, never stored. See below.
- UI: the viewer's team's recalls, dashed outline, "Recall (inferred)". Caption: "Recalls are inferred from item
  purchases at the fountain; Riot reports no recall event." Plus "Map view comes with the Rift Board."
  (`match_detail.rs:887`). Without purchases: "Wards: yours only. This game was stored without item purchases,
  so recalls and positions are not drawn." (6f93871; the atelier rejected the earlier "Re-import this game…",
  since only admins can re-import.)
- Note: the caption goes by `has_purchase_data`, `/admin/reimport` by `timeline_version`. The seeded demo game
  (`SEED1_`, never re-imported) shows the no-purchases caption while the admin page says everything is current.
  Correct for demo ids, but the two views disagree on that game.

**A2 — admin re-import** (9a44025, fix 97c65d8).
- `/admin/reimport` (`AdminReimportPage`, `src/pages/admin/reimport.rs`) with a hub row. `require_admin` plus
  `Bucket::Reimport` per admin (6 batches, then 1 per 20 s; `src/server/rate_limit.rs:153`).
- Batches of ≤5 from `db::matches_needing_reimport` (`src/server/db.rs:6506`): version missing or <2, not failed,
  never `SEED1_`. Match ids come from the DB only; no URL input.
- `ensure_headroom(2)` on the global Riot budget before each game. `db::replace_match_timeline` (`db.rs:6567`):
  one transaction, idempotent, refuses a game purged meanwhile, keeps `fetched_at` (no retention extension).
- 404/missing marks the game failed. 429/5xx/key errors stop the batch without marking anything. Without a Riot
  key the page shows a disabled button and a callout.
- Tests: `tests/integration/timeline_reimport.rs`; a lib test on the riven fixture
  `tests/fixtures/riot_match_v5_timeline_riven.json`; `reimport_limit_applies_after_the_filter` (`db.rs:8244`).
  No test touches the live API.
- The live backfill is an admin act after deploy (Law 7: merged ≠ live).

**A3 — map rendering: deferred** to the Rift Board (Jasper, 2026-10-18).

**E — visual-regression baselines (optional): not done.** `visual-regression.spec.ts` is not in CI's spec list. R3.

**H — privacy flag only.** `git diff f86e1dc..HEAD -- src/pages/legal` is empty. Sentences below.

Merges: 8cee9a8 (A into the branch), 3e02ca3 (main). Frames spec 152e05e, docs 9f0cb34, atelier closing
critique 215586b.

## Gates

CI run **43309188443** on 6f93871, all five jobs green: Check, lint, unit tests; Container image; Style
guardrails; Ops scripts; e2e smoke. Playwright: 202 expected, 0 unexpected, 1 flaky (`account.spec.ts:15`, the
known sign-in `waitForURL` timeout seen on runs before B). Server log: 0 slow, panicked or ERROR lines
(`ci-e2e-summary.md`). Not in CI: a real desktop screen capture (B), `visual-regression.spec.ts` (E).

## Inferred-recall heuristic

1. Purchases ≤25 s apart form one shop visit (`RECALL_BURST_GAP_MS`).
2. A visit counts as a recall if it starts after 90 s (`RECALL_OPENING_SHOP_MS`) and the buyer was not the victim
   of a `CHAMPION_KILL` in the preceding window of 20 s + 2 s per game minute, capped at 90 s
   (`recall_death_window_ms`).
3. It is placed at the visit's first purchase.

Minute frames are not used for a fountain test: a champion leaves the fountain in under a minute, so the test
would drop real recalls. Known misses: a recall without a purchase, a visit soon after a death. Card question 3.

## B manual desktop check

Not done. Brain has no headed browser, and a real `getDisplayMedia` prompt cannot be accepted in headless CI.
The 2026-10-16 overseer note makes it Jasper's act (card question 4). CI-side evidence: the stubbed e2e above and
the frames `feedback-dialog-1280.png` and `feedback-picker-1280.png`.

## H — privacy sentences for Jasper's legal read

In `src/pages/legal/datenschutz.rs`, unchanged:

1. `:58`, STORED: "For a game you open: the public match data of all ten players and its timeline." The
   timeline now also holds kill and objective map positions, every player's item purchases, and who placed each
   ward.
2. `:121`, Riot API triggers. An admin re-import of stored games is a new trigger.
3. `:89`, `MATCH_CACHE_RETENTION`. Still accurate; optionally add that a re-import does not extend the 12 months.
4. `:224`, "Download my data". Screenshots are now included as base64 images (up to 32 MB; more is counted as
   omitted).
5. Optional: inferred recalls are derived, not reported by Riot.

No new table, so the legal-page↔schema test still passes.

## Frames

Written by CI from `e2e/tests/frames-m149.spec.ts` into `publish-round-r1/frames/after/`:
[match-detail-390](publish-round-r1/frames/after/match-detail-390.png),
[match-detail-390-flow](publish-round-r1/frames/after/match-detail-390-flow.png),
[match-detail-timeline-390](publish-round-r1/frames/after/match-detail-timeline-390.png),
[match-detail-1280](publish-round-r1/frames/after/match-detail-1280.png),
[feedback-dialog-390](publish-round-r1/frames/after/feedback-dialog-390.png),
[feedback-dialog-1280](publish-round-r1/frames/after/feedback-dialog-1280.png),
[feedback-picker-1280](publish-round-r1/frames/after/feedback-picker-1280.png),
[me-sync-390](publish-round-r1/frames/after/me-sync-390.png),
[me-sync-1280](publish-round-r1/frames/after/me-sync-1280.png),
[admin-reimport-390](publish-round-r1/frames/after/admin-reimport-390.png),
[admin-reimport-1280](publish-round-r1/frames/after/admin-reimport-1280.png).

No `before/` set: CI renders only this branch. Before strips are M141's committed
[match-detail-390](../M141-team-companion-feedback-batch-1/publish-round-r1/frames/after/match-detail-390.png) and
[me-390](../M141-team-companion-feedback-batch-1/publish-round-r1/frames/after/me-390.png).
`rubric-check.md` (CI) covers public pages only; the atelier applied the rubric ids by hand.
Atelier: recipe cards before (90b6099), closing critique `atelier-critique-r2.md` (215586b): all surfaces PASS
after the caption fix.

## R3

1. E: visual-regression baselines; add the spec to CI or retire it.
2. Audit queries with `ORDER BY … LIMIT` over a uniquely indexed field plus a `WHERE` (the A2 bug).
3. A1 phone carry-overs (not this branch's): timeline filter chips and rail labels (~28 px) and "Attach an image"
   (~32 px) are under 44 px at 390.
4. Missing frames: a 1280 timeline still, and `/admin/reimport` with games waiting.
5. Map view (A3) with the Rift Board.
6. `docs/DESIGN.md` Button row says "Targets at least 24px"; state the 44 px phone rule for timeline markers in 6.3.

## Security notes

- **Capture path.** Capture runs only after a user gesture in the browser; the frame stays in the page until
  Send and every track is stopped right after the grab, also on failure. The crop enters the M141 upload path;
  the server side (`attachments.rs`: magic-byte sniff, decode limits, WebP re-encode, size cap, admin-only read)
  is unchanged. The export reads files only for the session user's own report ids, paths from
  `attachments::path_in`, never from the client.
- **Backfill path.** `reimport_batch` takes no input: `require_admin` first, then `Bucket::Reimport` per admin,
  ids from the DB only (`SEED1_` excluded), the global Riot budget checked per game, one transaction per game,
  `fetched_at` kept so retention still counts from the first fetch.
- **CI write scope.** Both write-back steps run only on `push` to `mission/*` (`ci.yml:72`, `:278`), reject a
  name that is empty, contains `/` or starts with `.` (`ci.yml:76–78`, `:283–285`), write under
  `.planning/missions/<name>` and push only `HEAD:$BRANCH` (`ci.yml:99`, `:319`). They never push to main.
