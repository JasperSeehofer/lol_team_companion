# M150 R1 — Game-plan suggestion chips: REPORT

Branch `mission/M150-game-plan-suggestion-chips` (worktree `lol_team_companion-wt/M150-game-plan-suggestion-chips`). Tip: `eeec922` plus the CI frames commit and this report. No VERIFY file is written by this session; the overseer spawns the fresh Opus verifier.

## Budget

- Declared 7.0 M, cap 10.5 M, wall 16 h.
- Used: about 3.4 M. That is the overseer's meter (2.8 M at the 2026-11-17 resume) plus my estimate of about 0.6 M for the resume (merge check, the critique, the fix pass and this report). The meter's figure is the one that counts.
- The STOP RULE never applied. All items 0–6 shipped, including the phone row, the bottom sheet and the signature slide; nothing moves to R2.
- Wall: the work ran 2026-11-12 (about 3 h active). The session sat idle from 2026-11-12 until the overseer's resume on 2026-11-17 (care rule R-03 stopped it), and the resume took about 2 h, mostly waiting for CI.

## Least-confident decisions (read these first)

1. **Suggestions need five known champions on each side.** The server returns nothing unless both sides resolve to five distinct trait-table rows.
   - The wait line ("Suggestions appear when both drafts are in.") shows while either side has fewer than five non-empty picks.
   - If five are filled but one is unknown free text, the line reads "No suggestions for these picks." It does not say which pick is unknown.
2. **When their champion counters ours**, the claim uses the same template as our counters (e.g. "Janna undoes Amumu's engage") and goes to Notes. The claim text does not say which side each champion is on; the card's pair line and the reason do.
3. **A chip retires only while the claim is an exact line.** The match is trimmed and case-folded, with no marker stored in the text. If the coach rewords the accepted line, the chip can come back. The mission ruled out a stored marker, so a reworded claim reads as "no longer the app's claim".
4. **The picks come from the client, not from a plan id.**
   - `plan_suggestions(ours, theirs)` checks the signed-in user and their team, limits per IP (`Bucket::Graph`), and caps each side at 5 picks of at most 64 characters.
   - It reads nothing stored: no plan, no team data, no Riot. So it works for an unsaved plan, and no other team's plan or picks can be reached.
   - The client asks 400 ms after the last pick change.
5. **The written-line echo** is an overlay that mirrors the whole field. It is not the recipe's bottom-anchored overlay, which misaligned on short text. The CI frames show 0 px drift at 1280. On phones the echo is skipped when the field is off screen, which is usual after the sheet closes; the line still lands.
6. **B2 makes every `Dialog` close X 44 px below `md`**, not only the sheet's. It is a global touch-target fix and is unchanged from `md` up.
7. **The page shifts after Add (atelier W1, not fixed).** The card collapses as the line is written, so the Win field rises by about 150 px. A fast double click could land on a Strategy tag. This is on the card as a watch item.
8. **Strength of composition claims** is my call: front line and one-sided damage are 2, the others 1. Pair rules of weight 3 therefore lead, and composition claims mostly surface after dismissals.

## What shipped, per item

| # | Item | State | Where |
|---|---|---|---|
| 0 | Atelier first | Recipe cards before any render; amended once after the critique (the sheet keeps the Dialog's close X) | `atelier-recipes-r1.md`, `acb646f` |
| 1 | Suggestion path | `suggest()` pure over `PairRelations::between` + `CompProfile::sum`, never `fit::rank`, no Riot; ≤12 ranked server-side, ≤3 shown; deterministic; rule variety (`vary_rules`) so three chips teach three facts | `src/server/graph/suggest.rs`, `src/models/suggestion.rs`, `plan_suggestions` in `src/pages/game_plan.rs`; `085fd46`, `080a1eb` |
| 2 | Dismiss per plan | `dismissed_suggestions` baseline field, `option<array<string>>`, no migration (latest stays v16); bounded server-side twice | `schema.surql`, `src/server/db.rs`, `src/models/game_plan.rs`; `d6a58d8` |
| 3 | Accept writes a line | Appends the claim as a plain line to Win conditions or Notes, no marker; the chip retires while the line exists; one log line per accept/dismiss | `fd37dc9` |
| 4 | Surface | Collapsed line between Matchup and Strategy; chip row; inline margin-note card (desktop); horizontal row + `DialogSize::BottomSheet` (phone); signature slide; `data-feedback-label` on chips and card | `03f202f`, `fd37dc9`, `945aa08` |
| 5 | Proof | `e2e/tests/suggestion-chips.spec.ts` (7 tests) in the CI list; 16 frames; atelier critique | `96cf2eb` |
| 6 | Round dir | `publish-round-r1/entry.json`, frames, critique | this commit |

`fit.rs` now calls four small `CompProfile` threshold helpers (`src/models/graph.rs`) that `suggest` shares. `rank`'s results and tests are unchanged.

## Claim templates

The suggestion path can emit the following claims. `{a}` and `{b}` follow the rule's own order (for a counter, `{a}` beats `{b}`); the reason is the rule's existing `sentence()`. A unit test checks that every `RULES` id has a template and that each fits in 60 characters with two 14-character names.

**Synergies:**

| id | claim |
|---|---|
| syn.engage_burst | {a} engages, {b} bursts |
| syn.engage_wombo | {a}'s engage sets up {b} |
| syn.wombo | {a} and {b} combine ultimates |
| syn.protect_scale | {a} keeps {b} alive to scale |
| syn.protect_style | {a} plays around {b} |
| syn.cc_chain | {a} and {b} chain crowd control |
| syn.dive_together | {a} and {b} dive together |
| syn.dive_followup | {b} follows {a}'s engage |
| syn.siege_range | {a} and {b} siege from range |
| syn.frontline_carry | {a} fronts for {b} |
| syn.catch_followup | {a} catches, {b} follows up |
| syn.disengage_poke | {a} peels while {b} pokes |
| syn.split_hold | {b} holds waves, {a} splits |
| syn.early_pressure | {a} and {b} press early |
| syn.skirmish | {a} and {b} win skirmishes |

**Counters:**

| id | claim |
|---|---|
| ctr.poke_outranges | {a} pokes {b} down from range |
| ctr.engage_denied | {a} undoes {b}'s engage |
| ctr.peel_dive | {a} peels {b}'s dive |
| ctr.side_lane | {a} splits where {b} can't clear |
| ctr.burst_immobile | {a} reaches and bursts {b} |
| ctr.dive_backline | {a} dives {b}, who can't escape |
| ctr.cc_mobile_melee | {a}'s crowd control pins {b} |
| ctr.lane_range | {a} outranges {b} in lane |
| ctr.true_vs_tank | {a}'s true damage cuts {b} |
| ctr.sustained_vs_tank | {a} wears {b} down |
| ctr.engage_on_poke | {a} engages before {b} pokes |
| ctr.mobile_vs_poke | {a} dodges {b}'s poke |
| ctr.catch_immobile | {a} catches {b} |
| ctr.clear_siege | {a} clears {b}'s siege waves |

**Composition claims.** These reuse the existing `FIT_RULE_IDS` and the Draft Graph's own thresholds, with no new predicates. Claims about our side go to Notes; claims about their side go to Win conditions.

| id | claim | strength |
|---|---|---|
| fit.needs_engage | Our/Their comp has no hard engage | 1 |
| fit.needs_frontline | Our/Their comp has little front line | 2 |
| fit.needs_wave_clear | Our/Their comp clears waves slowly | 1 |
| fit.one_sided_damage | Our/Their damage is mostly physical / magic | 2 |

There is no "early pressure" composition claim; the page's example had no rule behind it. Early pressure appears only through `syn.early_pressure`.

## Ranking

1. Sort by strength (highest first), then Win conditions before Notes, then rule id, then key.
2. Keep one suggestion per key, and one per unordered champion pair (the strongest).
3. Variety: each rule's first suggestion comes before any repeat. This is applied again after the client filters out dismissed and written ones.
4. The server returns at most 12; the page shows 3.

For the e2e lineup (Malphite/Amumu/Orianna/Jinx/Leona vs Gnar/Nidalee/Xerath/Ezreal/Janna), the three chips are:
- "Amumu engages, Malphite bursts"
- "Amumu's engage sets up Orianna"
- "Amumu and Leona combine ultimates"

## Dismiss key and its bound

- **Key format:**
  - Synergy: `rule_id:idA:idB`, the two champion ids sorted.
  - Counter: `rule_id:winner:loser`.
  - Composition: `rule_id:ours|theirs`.
  - The ids come from `traits.champion`, never display names.
- **Bound** (`models::suggestion::sanitize_dismissed`): each key is trimmed, must be at most 96 characters, and must match the shape `[a-z_.]+:[A-Za-z0-9]+(:[A-Za-z0-9]+)?`. Duplicates are dropped (the first is kept) and the list is capped at 64.
- **Where it runs:** in `create_plan`/`update_plan`, and again in `db.rs` before the bind. An empty list is stored as NONE.
- **Tests:** a unit test covers the bound, and the integration test `dismissed_suggestions_round_trip_and_are_bounded` checks it end to end (it ran in CI's check job).

## Feedback log

`suggestion_feedback(rule_id, action)` checks the user and team, then limits per user with `Bucket::Community`. `action` must be accept or dismiss, and `rule_id` must be a known rule id. It then writes exactly `tracing::info!(rule_id, action, "plan suggestion")`, with no user, team or plan id. There is no table.

## Chalk resolution

- Chips and card are ink. "Add to plan" is Secondary, so "Save plan" stays the page's one chalk action.
- Chalk appears only in the echo overlay during the ≤300 ms accept moment: a 180 ms left-to-right reveal, then a 120 ms dry to ink, then the overlay is removed. With reduced motion, the chalk is static for ≤300 ms.
- The atelier measured 7 chalk-hue pixels in the settled frame, the same as the page at rest. `DESIGN.md:202` and `:463` are unchanged.

## Sheet form

The sheet is a `Dialog` variant, `DialogSize::BottomSheet`, not a new primitive. It keeps the dialog's focus trap and focus return, has a top hairline and no shadow, is at most 85dvh tall and handles the safe area. The `A11y:` line is extended, and the gallery's dialog section has "Open bottom sheet". To keep the sheet without a shadow, `border`/`shadow-overlay` moved from the base class into the Sm/Md/Lg/Sheet size classes; those sizes look the same as before.

## The one STORED sentence

`src/pages/legal/datenschutz.rs:43`:
- Before: "…game plans with checklists and checklist templates, reviews, …"
- After: "…game plans with checklists, checklist templates and the suggestions dismissed on a plan, reviews, …"

There is no `account.rs` row, because the field references no user.

## Gates

- **Local** (shared target, one cargo at a time) on the merged state, after the fixes:
  - `cargo fmt --check` OK
  - clippy ssr and wasm32 with `-D warnings` clean
  - `cargo test --features ssr --lib` 513 passed, 0 failed
  - `scripts/lint_design.sh` every rule at zero
- **CI on `080a1eb`** (run 43298644926): the check job passed (fmt, clippy on both targets, unit tests, DB integration tests incl. the new one), because e2e needs it. e2e: 199 expected, 0 unexpected, 0 flaky, including all 7 suggestion-chips tests. The server log had 0 slow-request, panic or ERROR lines.
- **CI on the merged and fixed tip:** run 44135227504 on `eeec922` (merged with main, atelier fixes in): e2e 200 expected, 0 unexpected, 0 flaky, including the 7 suggestion-chips tests; 0 slow-request, panic or ERROR lines in the server log; the check job passed (e2e needs it). I could not read the container-image job's conclusion: the host has no `gh`, and the public API rate-limited me after the first reads. The verifier or overseer should confirm it on the run page.
- **The CI fix this round needed (outside M150's scope, flagged):** since about 16:43 UTC on 2026-11-17, every branch failed at "Install e2e dependencies". That is my M150 runs `fe93d89`, `945aa08` and `8d989b8`, and M160's run `1c858e3`. GitHub's `ubuntu-latest` label is moving to Ubuntu 26, and `playwright install --with-deps` fails there. Commit `eeec922` pins all five jobs to `ubuntu-24.04`, and the run went green. **Main has the same problem until a branch carrying this pin lands**, and M160 will hit it too.
- **Rubric:** `rubric-check.mjs` in CI reaches public pages only (`publish-round-r1/rubric-check.md`). The atelier therefore judged A1–A4, I1–I8 and V1–V3 for the plan surface from the frames and the source (see the critique).

## Atelier

- **First critique (fresh context, `atelier-frames-critique.md`): FIX.**
  - Chip, card, the collapsed/wait/none lines and the written line passed.
  - The phone row and sheet got two blocking fixes: B1, the first chip was clipped and the bleed stopped at the gutter; B2, the close X was 32 px.
  - Three polish notes: N1 smaller tilt, N2 text-body reason at 390, N3 chip padding on scale.
  - All five were applied in `945aa08`.
- **Re-check of the re-captured 390 frames:** **PASS** on all five surfaces (fresh context, appended to `atelier-frames-critique.md`).
  - B1: the first chip sits at x=16 and the row bleeds to x=389.
  - B2: the close X is 44 px from the code (the glyph centre matches).
  - N1–N3 are visible.
  - No regression at 1280; the accept strip still drifts 0 px and the settled line is ink.
  - W1 stays a watch item.

## Merge and landing

- `origin/main` (M156 `1018903`, M165 `6923a6c`) was merged into the branch with no rebase.
- Conflicts were only in `ci.yml` (both new specs kept) and `CHANGELOG.md` (both milestone entries kept; M150 is dated 2026-11-17).
- The merged state compiles and passes the gates above.
- There is no migration: the first start of the new build only applies the baseline `DEFINE FIELD` line.

## R2

Nothing. Possible follow-ups, none of them half-built:
- W1: keep the card open, or anchor the scroll after Add.
- Show "ours"/"theirs" in counter claims.
- Tune composition strengths once the accept/dismiss log has counts.
