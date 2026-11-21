# Atelier critique: Review together R1 frames (CI run 43511191122, sha 951190a)

Mode: critique, fresh context. This run did not write design-brief-r1's cards; it judges against them. Read-only on code.

**Round-0 criterion** (design-brief-r1.md, top, TC-J10/J11/J12). This critique covers its sight half: atelier PASS per recipe card, harness rows A1–A4, I1–I8, V1–V3, `lint_design.sh`, and sight rows V5, V7, V8. Parts (1)–(4) are payload, DOM and server behaviour. They belong to `e2e/tests/review-together.spec.ts`, which ran green (192 expected, 0 unexpected, `ci-e2e-summary.md`). The frames corroborate those parts where they can, as noted below.

## Capture protocol (what was run)

1. Stills: CI's own Playwright captures at 390 and 1280, plus the `-flow` full-page shots. The surface is dark-only. All 52 Review together and before PNGs were read. The CI run's other surfaces (auth, draft, admin and so on) are out of scope.
2. Motion: `reveal-seq-{1280,390}-{before,0,50,110,170ms,after}`. The spec scrubs the paused Web Animation (`currentTime = t`), which makes the frames deterministic. Card 8's 230/400 ms frames and the ffmpeg `reveal-strip-*.png` were not produced, and no Performance trace exists, so card 8 §7 (long-task budget) is **n/a**.
3. Reduced motion: `reveal-reduced-1280` (the spec asserts opacity 1 and 0 animations).
4. Contrast measured from rendered pixels with PIL: notes 16.2:1, author caption 7.0:1, figures 10.8:1. The pressed chip's border against an unpressed one is about 2.5:1, with an `ink-3` fill and a semibold weight on top.
5. Harness and lint: see `rubric-check.md`. A1–A3 and V1 PASS. I1, I2, I6, I8, V2 and V3 FIX, every one a misfire, bundle-wide or n/a (reasons there). A4, I3 and I5 are judged below. Lint is OK at zero.
6. UX rows: below.
7. Reveal luminance, as text-ink fraction of final. 1280: group-1 heading 0.73, note words 0.78, author 0.79, +1 0.74, group-2 heading 0.73 at 50 ms; every sample ≥0.94 at 110 ms. 390: every band 0.73 at 50 ms and 0.95 at 110 ms. That is **one fade, simultaneous**: 220 ms `--dur-3` on `--ease-out`, one `[data-reveal]` wrapper in the live DOM (the second copy sits inside an inert SSR-streaming `<template>`).

## UX1–UX10 (critique-checklist.json, 10 checks today)

| id | answer | frames |
|---|---|---|
| UX1 | n/a: no in-flight frame from a stubbed endpoint was captured | — |
| UX2 | n/a: focus frame exists (chalk ring on "Remove", writing-1280-flow), but no empty or failed submit frame | writing-1280-flow |
| UX3 | PASS: blurred, the notes column and title lead. For the facilitator, chalk "Reveal now" leads, which is the place's job | /tmp/m165-blur.png from writing, facilitator-expanded, reveal and decide at 1280 |
| UX4 | PASS: chalk only on the one primary action and checked boxes (primitive); no side or result hues | all |
| UX5 | PASS: +1, Vote and Propose are the same component at both widths. Note: Match Day shows "Review together" twice, as chalk in the Next match card and outlined in step 5. That is the existing PrepSequence pattern (before frame: "Read the brief" twice) | match-day-1280-flow, before/match-day-prep-bo3-1280 |
| UX6 | n/a for hover. Focus and pressed are captured (+1, Voted, owner chips) | writing-1280-flow, decide-1280 |
| UX7 | PASS: phone controls 44 px (+1 52×44, Propose, Vote, Remove/Add another Touch, alt link `min-h-11`); A1 green | *-390 |
| UX8 | **FIX**: on expand, the "Round controls" button itself moves 47 px right (798 to 845) | facilitator-1280 vs -expanded |
| UX9 | **FIX**: the session is "Official match · G1", but the generated review is titled "Review" and listed as "Review without a plan", with no match name on the page | writing-1280, review-1280-flow |
| UX10 | PASS: the first thing you notice is the words, not the fade | reveal-1280, seq-50ms |

## Sight rows

- **V5 (the reveal's first frame): PASS.** The server-rendered and late-join frame (reveal-1280, seq "after", reduced) is the final ruled sheet at full opacity. The t=0 frame is the wrapper at 0 with the side column at full, gone by 50 ms (0.73).
- **V7 (DESIGN principle 13 and §5 bans): PASS.** Notes are ruled rows. Boxes appear only on controls and on the brief-allowed `bg-ink-1` + `border-y` review ground. There is no hero tile, stripe, gradient, blur, confetti or modal phase. The minority cards ("0 agreed" and "2" side by side) look identical. No per-person tally appears: text search finds no per-member count, presence has no markers, and the facilitator sees one "N of 5" only.
- **V8 (against references): PASS against the recipe cards and the app's own serif precedent. The references are too weak to judge from, and this says so.** The reference stills prove nothing (bot-blocked a1/b1, c3 a 404, e1 shows no reading-size serif). So V8 rests on the card parameters and on `draft_graph.rs:1364`, whose team-note quote is `note text-reading text-primary`. The note `<p>` here has the same classes (`components/review_together.rs:29`), drops the curly quotes per e2, and puts the sans `text-caption text-muted` author on its own line. That is the precedent, done right.

## Reveal checks (task 4)

One wrapper: PASS. Simultaneous: PASS (±3 % across groups, within sampling noise). ≤300 ms: PASS. Instant under reduced motion: PASS. **Anonymous group:** no name, avatar or "yours" marker, and the normalized row markup is a single structure in all 5 viewer snapshots. The only attributes are `data-note-id` (random record id), `data-plus-one` and `data-testid="propose"`, none of them per-viewer. **Per-person tally:** none. **Nudge:** present only in `nudge-1280` (`data-nudge="nudge.quiet_player"`, count text only). It is absent from every member, review and Match Day snapshot. The two-closed-sessions negative is not in the frames (spec-only).

## Verdict per surface

| surface (card) | shape | edge | contrast | speed | layering | tint | typography | verdict |
|---|---|---|---|---|---|---|---|---|
| 1 Thread step | ✓ | ✓ | ✓ | – | ✓ | ✓ | ✓ | **PASS** |
| 2 Writing | ✓ | ✓ | ✓ | – | ✓ | ✓ | ✓ | **FIX** (non-blocking: F1, F2) |
| 3 Facilitator | ✓ | ✓ | ✓ | – | ✓ | ✓ | ✓ | **FIX** (non-blocking: F2, F3) |
| 4 Reveal | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | **PASS** (+F4 capture, F5) |
| 5 Decide | ✓ | ✓ | ~ | – | ✓ | ✓ | ✓ | **FIX** (non-blocking: F6) |
| 6 Generated review | ✓ | ✓ | ✓ | – | ~ | ✓ | ~ | **FIX** (non-blocking: F7) |
| 7 Nudge | ✓ | ✓ | ✓ | – | ✓ | ✓ | ✓ | **PASS** |
| 8 Sequence | – | – | – | ✓ | ✓ | – | – | **PASS** (strip/trace n/a) |

**FIXes**

- **F1 (writing, 390, non-blocking).** The sealed sentence sits about 8 px above the "HERE NOW" eyebrow and reads as one block. The span `text-body text-secondary -mt-3 mb-6` carries a negative margin that eats the gap. Drop `-mt-3` so the parent's `gap-8` applies, or give the sentence at least `mb-4` clear of the presence strip below `lg`.
- **F2 (header meta, every session frame, non-blocking).** When nothing is saving, the `SaveStatus` span (`data-save-state="idle"`, `inline-flex … text-muted`) renders empty but still takes the `gap-x-3` slot. "Open until…", "Revealed" and "Revealed · Decide (0)" then start 12 px right of the title. Hide it when empty (`empty:hidden`) or don't render it.
- **F3 (facilitator, non-blocking).** The Round-controls disclosure stretches to the expanded row's width and its centred label jumps 47 px. Add `self-start` to the button (or `items-start` on its column). Optional: a chevron, so the control reads as a disclosure.
- **F4 (capture, BLOCKING for the round card only, not for code).** The fixture text encodes the author letter (`mka6068536280offplan`). Every anonymous-group still therefore *shows* who wrote each card, even though the markup is clean. A human looking at these frames cannot see anonymity. Re-shoot the anonymous-group stills with neutral marker strings before Jasper's tap. The string search can keep the ids and names.
- **F5 (reveal, non-blocking).** The live region says "Revealed: N notes." where card 4 says "Revealed: 14 notes, 5 of 5 voices." Add the voices clause.
- **F6 (decide, non-blocking).** On the facilitator's side the chosen owner shows only as a pressed chip (border about 2.5:1 against the others, plus fill and weight). Members and phones get the text "Accepted · owner X". Render that same line under the facilitator's chips too, so the choice reads as words (UX5).
- **F7 (generated review, non-blocking).** (a) The note-group headings are `h2 text-title`, the same as "Reviewed together", so the hierarchy is flat. Make them `h3 text-heading` inside the section. (b) The meta lacks card 6's "· closed `LocalTime`". (c) No match stamp is visible (UX9). Add "Official match · G1" to the section meta.

**Overall: FIX, no code blocker.** The signature moment, every voice landing at once, is right: one element, one ease, simultaneous, and instant under reduced motion. It is worth showing first. One capture blocker (F4) must be cleared before Jasper's tap. The criterion's literal "harness rows green" is not met (6 rows FIX, all misfires or bundle-wide). That needs a ruling, not a code change.

## Staleness flags

- BROKEN-LINK: references.md c3 (fluent2 AvatarGroup) — the URL returns the Fluent 404 page.
- STALE: references.md a1/b1/b3 — bot-blocked (403); they can never serve V8.
- STALE: wiki/assets/ux-rubric/rubric-check.mjs V3 — it measures `textContent` length of the first 20 `<p>`, not rendered CPL, so every sentence under 45 characters fails.
- STALE: wiki/assets/ux-rubric/rubric-check.mjs V2 — it scans every rule in the stylesheet, used or not, so Tailwind preflight and `.sr-only` fail every page.
- STALE: atelier prompt "nine questions UX1–UX9" — critique-checklist.json now has UX10.
- CONTRADICTION: design-brief-r1 cards 2/4 (`max-w-2xl` 42 rem note column) vs ui-guidelines / rubric V3 45–75 CPL — serif italic notes run about 93 CPL at 1280. Note for R2's card, not a build FIX.
