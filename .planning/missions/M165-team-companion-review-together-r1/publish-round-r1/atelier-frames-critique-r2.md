# Atelier critique, round 2 (the last allowed): Review together R1 frames (CI run 43518892716, sha 37a7936)

Mode: critique, fresh context, read-only on code. This run did not write design-brief-r1's cards or the round-2 fixes. It judges both against the cards. Note on the orchestration clause: this run had no subagent tool, so the reads, the harness run and the measurements below were done in this context. No code was edited and no app test was run.

**Round-0 criterion** (design-brief-r1.md, top, TC-J10/J11/J12). This critique covers its sight half: atelier PASS per recipe card, harness rows A1–A4, I1–I8, V1–V3, `lint_design.sh`, and sight rows V5, V7, V8. Parts (1)–(4) belong to `e2e/tests/review-together.spec.ts` (CI run 43518892716). Frames corroborate them where they can.

## Capture protocol (what was run)

1. Stills: CI's 390 and 1280 captures and `-flow` shots for Review together and Match Day (`frames/after/`). Read: writing 390/1280, facilitator 1280 and expanded, reveal 390 and 1280-flow, decide 390/1280, review 390 and 1280-flow, match-day 390, reveal-seq 390 0 ms. Dark only.
2. Motion: luminance of the note column as a fraction of the final frame, from `reveal-seq-*`. 1280: 0.00 / 0.72 / 0.95 / 0.98 / 1.00 at 0 / 50 / 110 / 170 ms / after. 390: 0.21 (the header and presence are already full at t=0; notes at 0) / 0.78 / 0.96 / 0.99 / 1.00. This is the round-1 curve: one 220 ms ease-out fade. No strip or trace was produced, so card 8 §7 stays n/a.
3. Reduced motion: `reveal-reduced-1280` is unchanged code. Round 1's PASS stands.
4. Measured in Chromium 1243 on the snapshots, with the CSS rebuilt from this tree (it now contains `max-w-prose`). Note `<p>` max-width 558 px = 65ch. Rendered measure is **72 CPL at 1280 and 46 at 390** (round 1: about 93 at 1280). The sealed sentence computes to `display: inline`, with **3 px** between its bottom (217) and the "HERE NOW" eyebrow (220) at 390.
5. Harness: `rubric-check.mjs --app-dir <M137>/e2e` on 14 unique snapshots × phone/desktop = 28 runs, served same-origin as in round 1. Exit 1.
6. Lint: `./scripts/lint_design.sh` printed "Design lint: OK (every rule at zero)."

### rubric-check table (28 runs per row)

| row | result | reading |
|---|---|---|
| A1, A2, A3 | PASS 28/28 | green (A3 proves nothing on a static file; judged live, PASS) |
| A4 | not emitted | sight: PASS |
| I1, I2, I6, I8 | FIX 28/28 | same misfires as round 1, see ruling note |
| I7, V1, V4 | PASS 28/28 | green |
| V2 | FIX 28/28 | identical bundle-wide list (preflight `optgroup option` 20px, `.sr-only` −1px…), none from review_together |
| V3 | FIX 28/28 | flags `<p>` under 45 chars (notes 31–41) and hidden dialog copy; rendered measure is in band (item 4) |

## UX rows (critique-checklist.json, UX1–UX10)

| id | answer | frames |
|---|---|---|
| UX1 | n/a: no stubbed in-flight frame | — |
| UX2 | n/a: no empty or failed submit frame | — |
| UX3 | PASS: notes and title lead; for the facilitator, chalk "Reveal now" leads | writing-1280, facilitator-1280-expanded |
| UX4 | PASS: chalk only on the primary action and checked boxes | all |
| UX5 | PASS: the owner is now written as words on both sides ("Accepted · owner …") | decide-1280, decide-390 |
| UX6 | n/a for hover; focus and pressed states unchanged from round 1 | decide-1280 |
| UX7 | PASS: phone controls 44 px; A1 28/28 | *-390 |
| UX8 | **PASS** (was FIX): "Round controls" stays at x≈798 when opened | facilitator-1280 vs -expanded |
| UX9 | **PASS** (was FIX): the section meta reads "Official match · G1 · 4 of 5 voices · closed Today 22:51" | review-1280-flow, review-390 |
| UX10 | PASS: you notice the words, not the fade | reveal-seq-1280-50ms |

## Round-1 items, re-checked

| item | status | evidence |
|---|---|---|
| F1 sealed sentence crowds presence (390) | **OPEN** | The diff dropped `-mt-3` and kept `mb-6`. But `Text` defaults to a `<span>`, which renders `display: inline`, so the vertical margin does nothing. 3 px measured. writing-390 still reads "…until the reveal. / HERE NOW" as one block. **Exact fix:** `tag=TextTag::P` on that `Text` in `pages/review_together.rs` (the `{SEALED}` line). Then `mb-6` gives 24 px. One attribute. |
| F2 idle SaveStatus takes a gap | CLOSED | "Open until Sat 21:50" and "Revealed" are flush with the title at x=32 (facilitator-1280, reveal, decide). The idle wrapper is `sr-only` and stays mounted for the announcement. |
| F3 Round controls jump | CLOSED | `items-start`; same x before and after. No chevron (that was optional). |
| F4 capture: markers encoded the author | CLOSED | Markers are now `ref <8 random consonants>` (`ttrcpktn`, `fxqwkktj`…). No `mk[a-e]` string is left in any snapshot. In reveal-390 and reveal-1280-flow the "What didn't go to plan" rows show nothing a human could map to a member. |
| F5 announcement lacks voices | CLOSED in code, not visible in frames | `announce` now formats "Revealed: N notes, X of Y voices." from `state.voices`, which the server fills after the reveal as a team-level count (never while writing). The sr-only region is empty in every snapshot, because the frames are taken after the transition, and the spec does not assert the string. Suggest a one-line spec assertion. This is not a FIX. |
| F6 owner only as a pressed chip | CLOSED | "Accepted · owner rtu_…_b" under the chips (decide-1280). The "No owner yet" caption moved into the same line. |
| F7 review hierarchy, closed time, match stamp | CLOSED | The groups are `h3 text-heading` under the h2 "Reviewed together" (visibly a step down in review-1280-flow). The meta carries the match, the voices and the closed `LocalTime`. |
| V3 measure note (93 CPL) | CLOSED | `max-w-prose` gives 72 CPL at 1280, inside 45–75. Resolves round 1's CONTRADICTION flag in the build. |

**New observations (not scored, no round left to carry them):**
- N1, decide-1280: the "Decide" block starts at x=752 and "HERE NOW" at x=785, so the right column has two left edges. It was the same in round 1.
- N2, review-390: the meta wraps with a dangling "·" at the end of line 1.
- N3: the post-game list still labels the generated review "Review without a plan". That is the editor's convention, outside card 6.

## Sight rows

- **V5: PASS.** The late-join, server-rendered and reduced frames show the final sheet at full opacity. The fade is the same single 220 ms change as before.
- **V7: PASS.** Ruled rows, no hero tile, gradient, blur or modal phase, and no per-person tally in any snapshot (only "N of 3 votes", "N of 4 answered" for your own answers, and team-level voices). The minority note looks like the rest.
- **V8: PASS, on the same footing as round 1.** The note uses `note text-reading text-primary`, the `draft_graph.rs` serif precedent, with a sans caption author. The references are still too weak to judge from directly (see flags).

## Verdict per surface (design-brief-r1 cards)

| surface (card) | shape | edge | contrast | speed | layering | tint | typography | verdict |
|---|---|---|---|---|---|---|---|---|
| 1 Thread step | ✓ | ✓ | ✓ | – | ✓ | ✓ | ✓ | **PASS** |
| 2 Writing | ✓ | ✓ | ✓ | – | ✗ (390 spacing) | ✓ | ✓ | **FIX**: F1 open, `tag=TextTag::P` |
| 3 Facilitator | ✓ | ✓ | ✓ | – | ✓ | ✓ | ✓ | **PASS** (F2, F3 closed) |
| 4 Reveal | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | **PASS** (F4, F5 closed) |
| 5 Decide | ✓ | ✓ | ✓ | – | ✓ | ✓ | ✓ | **PASS** (F6 closed) |
| 6 Generated review | ✓ | ✓ | ✓ | – | ✓ | ✓ | ✓ | **PASS** (F7 closed) |
| 7 Nudge | ✓ | ✓ | ✓ | – | ✓ | ✓ | ✓ | **PASS** (unchanged) |
| 8 Sequence | – | – | – | ✓ | ✓ | – | – | **PASS** (strip and trace n/a) |

**Overall: 7 of 8 cards PASS. Writing is FIX on one carried, non-blocking spacing defect (F1).** The fix shipped for F1 had no effect: an inline span ignores vertical margin. Per brief §9 there is no round 3. The calling session can take the one-attribute fix as a post-round correction verified by a single 390 still, or report card 2 as open. The atelier does not ratify either. The signature moment is intact and is still the thing to show Jasper first. The anonymous-group frames are now fit for his tap.

## Ruling note: harness staleness (not surface defects, not counted as FIX above)

I1 (looks for the string "settings"; the app says "Profile"), I2 (requires a progress indicator; nothing here runs over 10 s), I6 (requires a notification policy; the surface sends nothing by design, brief §8), I8 (looks for "undo" in the static DOM; Undo lives in toasts after actions), V2 (scans every rule in the stylesheet, used or not), V3 (`textContent` length of `<p>`, not rendered measure). All six fail 28/28 identically on every page, including the CI public-page run (`rubric-check.md`). The criterion's literal "rows green" is therefore unmeetable by any surface. Recording them as harness false positives is a decision for the human or the calling session.

## Staleness flags

- STALE: wiki/assets/ux-rubric/rubric-check.mjs V3 — measures `textContent` length, not rendered CPL (carried).
- STALE: wiki/assets/ux-rubric/rubric-check.mjs V2 — counts unused stylesheet rules (carried).
- STALE: wiki/assets/ux-rubric/rubric-check.mjs I1/I8 — string heuristics ("settings", "undo") that misfire on this app's wording and toast pattern.
- STALE: atelier prompt "nine questions UX1–UX9" — the checklist has UX10 (carried).
- BROKEN-LINK: references.md c3 (Fluent 2 AvatarGroup) — returns a 404 (carried).
- STALE: references.md a1/b1/b3 — bot-blocked (403) (carried).
