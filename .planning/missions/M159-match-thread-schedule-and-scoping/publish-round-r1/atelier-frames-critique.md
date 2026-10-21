# M159 r1 atelier frame-strip critique (fresh read of CI after-frames)

Process: no subagent tool in this run, so the atelier read the frames and the few source lines cited itself; no code touched. Frames are stills from CI: layout is judged, motion is not (nothing here animates). Rubric ids A1-A4, I1-I8, V1-V3 read from rubric.json. Corrections applied: picker prefers +-6 h; Bo3 readout is "G1 done · G2 to do · G3 not played".

## Verdicts
| Surface | Verdict |
|---|---|
| match-day-upcoming (390 / 1280 / flow) | PASS (one small FIX on count text) |
| draft-from-thread (390 / 1280 / flow) | FIX (capture artefact plus one long-title note) |
| match-day-prep-bo3 (390 / 1280) | FIX (frame does not show the Bo3 readout) |
| post-game-picker (390 / 1280 / flow) | PASS with a capture FIX (picker shows one row only) |
| match-detail-review (390 / 1280) | PASS |

## match-day-upcoming: PASS
- Craft: ruled typography, hairlines, mono-free bold date, Badge pair, no cards, chalk "Open the dossiers" remains the only loud element. Matches card 1. Could you name a shape? No. Would a director ship it? Yes.
- Law 11 / I7: in the 1280-flow frame the list sits directly under the Next match card, collapsed (single row, nothing to expand), and does not cover the match or push the Welcome/Prep content out of reach. No inputs in the list. PASS.
- No redesign: Match Day layout unchanged (left column width, right margin untouched).
- Rows: 390 two-line row is >=44 px; 1280 single line OK (A1). Opponent text "No opponent yet" is honest.
- FIX (minor): the row has no attendance text. Card 1 says show "No answers yet" (`text-muted`, never "0 of 5") when nobody answered; the row is silent. Add the muted caption on the right of line 1 (390) / last column (1280), in the upcoming-list row in `src/pages/home.rs` (UpcomingMatches). If the roster is empty, silence is acceptable; state which in the code comment.
- RSVP chips are not in any supplied frame (not judged).
- FIX (capture, not app): two stacked "Match saved" toasts sit over the Welcome panel at 390 and 1280. In the "reading-flow" frames this covers content; Law 11 is about the list, so no ding on the feature, but re-shoot after the toasts auto-dismiss (or dismiss them) so the flow frame is clean.

## draft-from-thread: FIX
- 1280 flow: the "For the match: Scrim vs Rooks..., Tomorrow 19:00. Saving links the draft to it." line under the Draft name field is plain muted caption, honest and quiet. Good. Board unchanged. PASS on that element.
- FIX (stray toast): a "Match saved" success toast shows on /draft where nothing was saved (390: over the board legend text "Brackets group the picks..."; 1280: over the board's last two slots). Either it leaks from the preceding setup step in the capture, or the toast context persists across navigation. Source: the push is `toast.push.run(Toast::success("Match saved"))` at `src/pages/home.rs:1358`. If a client-side navigation from the match form keeps the toast alive onto /draft, scope it (dismiss on route change) or capture via a fresh load. As a frame it hides content (I7/misleading state).
- Note (not blocking): the auto-name "vs Rooks 1792600441124 · Thu 19:00 · G1" renders as a 3-line H1 at 390 and pushes the board down ~100 px. Fixture name is long; with a real opponent it is shorter. No change unless `PageHeader` title is not allowed to truncate; consider `text-headline` wrapping is acceptable.
- The 390 frame (viewport only) does not show the "For the match" linkage line; add a 390 flow frame or scroll shot for it.

## match-day-prep-bo3: FIX
- The frames are Bo3 and upcoming ("Tomorrow"). The review step reads "After the game: review / After the game / Write it right after the match..." The required readout "G1 done · G2 to do · G3 not played" is absent, because `src/pages/home.rs` (~line 509) shows it only once the match has started or a review exists. So the frame named "bo3" does not prove the feature. FIX: re-capture with the Bo3 started and G1 reviewed so the step reads "1 of 2 reviewed" (or similar) and the detail line reads in words, wrapping under the title at 390 in `text-caption`, uncut.
- Earlier work row: "Earlier work vs Rooks (1) / 1 draft", the explanatory caption, "Draft: Old plan vs them" link and "Use for this match" Quiet Touch button render as a ruled line, no card, no chalk. Matches card 2d. `ButtonSize::Touch` is set in source (min-h-11). PASS. Minor: at 1280 the button label sits ~11 px inside the right edge used by "1 draft" caption; align the right edges (same `justify-between` padding) if cheap.
- Chalk stays on "Read the brief" only (one primary). PASS.
- Capture: toast covers the "Getting started" heading at 1280; retake.

## post-game-picker: PASS (capture FIX)
- Craft: radiogroup of ruled rows, date in mono, five champion marks (monogram fallback in CI as no Data Dragon), result as Win badge on the right, "Not in the list (scrim / custom)" row. No raw Riot id visible. Selected row has ink-2 plus semibold plus checked radio (not colour alone). No select. Could you name a shape? No. Rows are `min-h-11 md:min-h-8` in source (A1 ok).
- The frame shows a single candidate, so the "Near the scheduled match" / "Other recent games" eyebrows (the +-6 h preference) are not visible. FIX (capture): seed two or three games with one inside +-6 h of the match and one outside, and re-shoot, so the lift and the eyebrows are proven. Source: `src/pages/post_game.rs` ~1190-1260.
- Monogram pills "He Ji Da Th Az" are 2-letter cues without names; ensure `aria-label` lists champion names (A4) since the visual cannot be checked here.
- The "From the match" block, Result segmented control and form are existing and unchanged. "Filled from match ... Undo" line is plain and quiet. Good.

## match-detail-review: PASS
- "Review as a team" is a Secondary button, 44 px at 390, beside the page's existing chalk "Write a learning" (still the single chalk). Wrapping at 390 fine: two buttons on one row, no overflow. At 1280 they right-align in the header actions slot. No icon duplicate. Good.
- No cross-viewport defects found; the table below is untouched.

## Rubric notes
- A1: nothing under 44 px found in the new pieces from source (min-h-11, Touch) and frames. A2: not recomputed here (needs rendered pixels from a harness run; captions in muted on ink-1/2 are the ones to check). A3/V1/V2: lint_design covers; no raw values visible. V3: prose lines are short; "Made before this match..." caption at 390 wraps inside ~40 CPL (fine as caption). I3: depth of 1 open for each new piece. I5: no confirms added. I7: PASS on Match Day; the stray toasts are the only covering UI.

## FIX list (smallest fixes)
1. Re-shoot match-day-prep-bo3 with a started Bo3 and G1 reviewed so the readout in words is visible (capture script, not app). `src/pages/home.rs` ~497-535 is correct.
2. Re-shoot post-game-picker with a game within +-6 h and one outside, to show both eyebrows (seed in capture script).
3. "Match saved" toast on /draft (and doubled on Match Day): dismiss on route change or fix capture order. `src/pages/home.rs:1358`; toast host is in `src/components/ui/toast.rs`.
4. Upcoming row: add muted "No answers yet" caption when nobody has answered (`src/pages/home.rs` upcoming-list row).
5. Add a 390 flow shot of draft-from-thread showing the "For the match" linkage line.
6. (cosmetic) align the right edge of "Use for this match" with the count caption above it, `src/pages/home.rs` ~965-980.

## Executor response
- **Toasts (FIX 3): capture fix, no app change.** The "doubled" toast is two real saves: match-day-upcoming saves two matches within the 5 s a success toast lives, one push each (`save` runs once per click; no Effect re-runs it). The one on /draft is the "Match saved" from the step before, carried over by the client-side "Start a draft" link. Toasts are app-wide by design (`ToastProvider` in `App`, DESIGN 6 table: success leaves after 5 s), and editors push "Saved"/Undo toasts right before `replace_url`, so dismissing on route change would cut that feedback short. Not changed. `match-thread.spec.ts` now dismisses every toast (and waits for the region to be empty) before each frame.
- **Bo3 readout (FIX 1): capture fix.** The editor accepts starts up to 12 h back, but an honest state with "done" in it needs a review, so the spec now drafts game 2 from the thread (`&game=2`) and writes the game 1 review from the thread (`/post-game?match_id=…&game=1`, Not in the list) while the match is still ahead. It asserts "G1 done · G2 to do · G3 not played" and "1 of 2 reviewed" before shooting; the earlier-work row is still in the same frame. App logic unchanged.
- **Picker lift (FIX 2): done through the seed.** `src/bin/seed.rs` stores a second Northgate game `SEED1_1000000002`, a lost rematch that ended 3 h before the seed ran (all five members synced; scoreboard only, no timeline). The spec, signed in as the seeded leader, records a match that started 2 h ago (the app allows 12 h back), opens `/post-game?match_id=…` and, in CI, asserts the rematch under "Near the scheduled match" and the day-old demo game under "Other recent games". The match is then removed in a `finally`, so the seeded Match Day is unchanged for later specs. post-game-fill and the other seed specs select the demo game by its value and still find it. Champion names: the avatars are `decorative` and the row carries the names as `sr-only` text inside the radio's label, so the accessible name lists them; the spec asserts "Darius" in the row.
- **390 flow frame (FIX 5):** `draft-from-thread-390-flow.png` (full page) added.
- **Alignment (FIX 6):** from `md`, "Use for this match" is pulled back by its padding (`md:-mr-3`) so its label ends on the edge of the count caption. Phones keep it as it is: the 16 px gutter has no room for the overhang without sideways scroll.
- **Not done: FIX 4 ("No answers yet").** RSVP is deferred to round 2 and no attendance data exists, so the row stays silent. Showing that text would claim a feature that is not there (honesty rule).
- **Not re-shot here:** frames come from CI. The re-shoot happens on the next CI run.
