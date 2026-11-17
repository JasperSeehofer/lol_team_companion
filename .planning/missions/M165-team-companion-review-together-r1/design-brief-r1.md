# M165 design brief R1: Review together (atelier intake, recipe mode, 2026-11-13)

## Round 0 criterion (verbatim from the mission frontmatter; jobs TC-J10, TC-J11, TC-J12)

> TC-J10, TC-J11 and TC-J12 of [[team-companion-ui-jobs]] pass their acceptance tests at 390 and 1280 px in a new multi-user e2e spec (five members registered by the spec, ≥3 browser contexts, temp SurrealKV, dummy Riot key), tightened on this surface: (1) sealed until reveal: no member's poll payload, hydration payload or DOM holds another member's text, the facilitator's included; the facilitator sees one written count across all four prompts, never per prompt; after the reveal an edit is refused server-side; (2) anonymous: no card under 'What didn't go to plan', and no proposal made from one, carries an author id, username, avatar, timestamp or write-order signal in any team-visible payload (server-fn JSON, hydration payload, HTML text, data-*, aria-*, title), by a string search for every fixture member's id and name in every member's context, the author's and the facilitator's included, while the author's export still lists the card; (3) decide ends with at most three action items, each owned by a current member's username and written with the generated review, whose what_went_well and improvements stay empty, and re-saving that review in the editor creates no action item; (4) the generated review, stamped with M159's scheduled_match and game_number (series only when the match has one), shows exactly one team-level voices line ('5 of 5 voices' when all five write, '4 of 5' when one passes every prompt) and no per-member count anywhere; the quiet nudge's payload is a count with no name or id, it appears on the facilitator's session page only, and not with two closed sessions. Each surface also needs atelier PASS against its recipe card, with rubric rows A1–A4, I1–I8 and V1–V3 green under rubric-check.mjs plus scripts/lint_design.sh against the app's tokens, and sight rows V5, V7 and V8 passed by the atelier. Under the STOP RULE the nudge moves to R1b with TC-J12, and (3) is met by the minimal decide (the facilitator picks ≤3, owners set, no vote).


Jobs served: **TC-J10** (review together, every voice heard), **TC-J11** (say what didn't go to plan with no name on it), **TC-J12** (notice a quiet player without naming them). Every critique of this round names this criterion. The per-screen done sentences at the end make it stricter in places. They never replace it.

**Signature moment:** *every voice lands at once.* At the reveal, the page changes in a single step. Before it, you see only your own sealed lines. After it, you see a ruled sheet of everyone's words in the team-note serif. It is one 220 ms opacity change on one element, with no stagger. Nothing else on this surface moves for its own sake.

**Phone writing** is the exception Jasper ruled on 2026-10-18 (PROP-20261018-01 is pending). DESIGN §6.9 is not amended by this round.

---

## 1. Token freeze (app tokens only; never manyfold's)

The source is `input.css` `@theme` and `docs/DESIGN.md` (direction "Survey"), frozen as of `7951e05`. **No new colour, size, radius, shadow or keyframe token.** The only allowed change to `docs/DESIGN.md` is the review-card serif sentence in §2/§3.3, which is mission item 11.

Allowed utilities on this surface (anything else needs an argument in REPORT):

| Kind | Allowed |
|---|---|
| Ground | `bg-page` (the page), `bg-ink-1` (the facilitator column and the generated-review section ground, optional), `bg-ink-2` (a pressed chip or toggle, through `Button pressed`), `bg-ink-3` (avatars, through `UserAvatar`) |
| Lines | `border-rule` and `divide-rule` (hairlines between notes and sections), `border-outline` (controls only, through primitives) |
| Text | `text-primary` (words), `text-secondary` (meta, presence names), `text-muted` (author caption, hints) |
| Accent | `bg-accent` + `text-accent-ink` **only** on the one primary action per view, through `ButtonVariant::Primary`; `text-accent` / the focus ring through the primitives. Never on a note, a count or the presence strip |
| Status | `text-danger` only inside `SaveStatus` failure and field errors; `text-warning` / `text-success` / `text-info` not used on this surface |
| Type | `note` (members' words only), `eyebrow`, `figures`, `font-display` (through `PageHeader`/`SectionHeader`/`Text`), `text-caption` `text-label` `text-body` `text-reading` `text-heading` `text-title` `text-headline` |
| Layout | the 4/8 scale: `gap-2 gap-3 gap-4 gap-6 gap-8 gap-12`, `py-3 py-4 pt-3 pt-6`, `px-4`, `max-w-2xl` (42 rem reading column), `w-80` (20 rem side column), `lg:grid lg:grid-cols-[minmax(0,42rem)_20rem] lg:gap-12` |
| Motion | `transition-opacity`, `transition-colors`, `duration-(--dur-1)` … `duration-(--dur-4)`, `ease-out`, `motion-reduce:transition-none` |

Banned on this surface: `side-blue`/`side-red`, `win`/`loss` (no sides or results here), `shadow-overlay` outside `Dialog`/`Menu`/`Toast`, any `rounded-*` written by the page, and any opacity modifier on a text or outline token (lints `faded-text`, `faint-outline`).

## 2. Motion class

**Feedback-only, plus one signature moment (the reveal).** DESIGN principle 11 and §3.7:

- Feedback: press and colour changes at the primitives' defaults (`--dur-1`/`--dur-2`, `ease-out`). The +1 count and the votes-left count change as text with no tween.
- **The reveal:** `opacity` 0 → 1, **220 ms (`--dur-3`), `ease-out`**, on **one wrapper element** around all four groups. Every note therefore sits at the same opacity at every frame by construction. There is no per-note stagger, no translate (a fade-up is the convergent default and is banned), no loop and no count-up.
- It plays **only when this tab saw the phase go from writing to revealed** through a poll. A reload, a late joiner or the server render arrives already revealed at full opacity (this protects V5's first frame).
- Reduced motion: no transition (the global rule in `input.css`). The builder also leaves out the `opacity-0` start class when `matchMedia('(prefers-reduced-motion: reduce)')` matches, so the change is instant and the DOM is final on the first frame.
- Nothing else animates. The timer, presence, counts and proposals re-render as text. List re-orders are not animated.

## 3. Shading

None. Flat Survey layers (`page` → `ink-1` → `ink-2` → `ink-3`) and hairlines. No shadow, gradient, blur, texture or glow. R1 (soft shading) and R2 (environmental motion) do not apply: this is not an atmospheric surface.

## 4. Serif rule

- **Card words** (all four prompts, at the reveal and in the generated review): `note text-reading text-primary` (Source Serif 4 italic, 16/1.6). This is the new DESIGN §2/§3.3 exception (owner's ruling 2026-10-20).
- **Author:** on its own line under the words, `text-caption text-muted`, sans, with the plain username only. No em-dash, no quotation marks, no avatar on the note.
- **Anonymous cards** ("What didn't go to plan"): `note text-reading text-primary`, with **no name, no avatar and no "yours" marker** for the author either. The row's markup is identical for every card in the group: the same elements, classes, `data-*`, `aria-*` and `title` for every viewer. The caption sits once under the group heading.
- **Not serif:** prompts (display face), the writer's own text fields (the field's sans while writing; it becomes serif at the reveal), proposals (sans, because they are the team's draft action items and carry no person), decided items, counts, and the nudge.

## 5. Anti-slop bans (V7; DESIGN principles 1, 11, 13; §6.11 "Never build")

No card soup: notes are ruled list rows, not boxed tiles. Boxes appear only on controls (fields, buttons, chips). No hero, no three-card row, no coloured left stripe, no gradient, glass, blur or emoji. No confetti or celebration at reveal or close. No streaks, no participation percentages, no per-member tallies. **No per-person "has written" check**, typing indicator, or ghost placeholder for other people's cards (a placeholder leaks a per-prompt count). No red dots or counters for guidance. No live re-sort under the pointer. **No modal for phases:** writing, revealed, deciding and closed are inline page states. `ConfirmDialog` is used only for "Reveal now" (early) and "Close and write the review". No highlight of the most-voted or the outlier card, and no dimming of minority cards.

---

## 6. Recipe cards (ui-craft-playbook §3 format)

Shared local components live in **`src/components/review_together.rs`** (not `ui/`). Each gets a doc comment with an `A11y:` line, as primitives do. None of them is promoted to a design-system primitive in R1: each has a single surface, and a primitive needs gallery cost and a second use. The one exception is **`PrepStep`'s second link** (card 1), because the step row's markup lives in the primitive.

Local components: `ReviewNote`, `NoteGroup`, `PlusOne`, `PresenceStrip`, `WrittenCount`, `OwnerChips`, `VotesLeft`, `QuietNudge`.

### Card 1: "Review together" on the match thread's review step

1. **Technique:** a second, quiet text link on the existing `PrepStep` row. It is not a new button block. Add `PrepStep::alt_link(href, label)`, which renders after the main action as `text-label text-secondary underline decoration-outline underline-offset-4 hover:text-primary` (the `NudgeLine` link grammar, never chalk). Update the gallery's `prep` section with one example.
2. **Parameters:** before a session exists and after the match started, the chalk action is **"Review together"** (`/review/together?match_id=…&game=N`, which opens or creates the session) and the alt link is **"Write it alone"** (today's `/post-game?…`). While a session is writing, the action is "Open the session" with status "Writing until 21:40" (a `LocalTime`). The step shows **no count** to anyone here. When revealed or deciding: "Join the reveal". When closed: the step is Done and links to the generated review. For a series, the link carries the game (`G2`), and the readout ("G1 done · G2 …") is unchanged.
3. **Name collision:** M159's "Review as a team" on match detail becomes **"Review together"** and opens the same session route, so that one name means one thing. The single-author editor stays reachable as "Write it alone". (Open taste question, see the summary.)
4. **Layout 390:** the step row as today. The alt link wraps under the chalk action on its own line, with a 44 px tall hit area (`inline-flex min-h-11 items-center`). **1280:** the alt link sits inline after the action with `gap-4`.
5. **References:** b1 (one explicit entry into a shared phase).
6. **Perf:** zero, server-rendered.
7. **Reduced motion:** nothing moves.
8. **Verify:** stills at 390 and 1280, before (`main`) and after. A1 for the alt link. A DOM probe that the review step shows no "written" count.

### Card 2: Silent-round writing view

1. **Technique:** a reading-width form of four prompt sections. Each is an `h2` prompt in `text-heading` (display face) with your own entries under it as `TextField rows=3 maxlength=280`. **Nothing of anyone else is in the DOM**: no placeholders and no counts per prompt.
2. **Header:** `PageHeader` with eyebrow "Review together", title "Scrim vs Riverside Rooks" (+ " · G2" for a series), and meta of `SaveStatus` · the deadline line. Below it, one sealed sentence as `Text Body Secondary`: "Sealed. Only you see what you write until the reveal." (a1).
3. **Deadline line (calm timer, a3):** `figures text-label text-secondary`. Async: "Open until Sat 21:40" (`LocalTime`, no countdown). Sync round: "Closes 21:40 · 3 min left", which re-renders **on the minute** ("under a minute left" for the last 60 s). No seconds, no colour change, no bar. A seconds ticker would break A3 and the calm.
4. **Prompts, in order:** "The moment that decided the game" · "What went to plan" · "What didn't go to plan" (with the caption "Anonymous: nobody on the team sees who wrote these." in `text-label text-muted` under the heading, shown **while writing too**, so the writer knows before writing) · "One thing for next time". Above the sections, a progress cue for **your own** answers only: `figures text-caption text-muted` "2 of 4 answered" (a2).
5. **Entries:** at most 3 per prompt. "Add another" is `Button Quiet Touch` and disappears at 3. Each entry has "Remove" (`Button Quiet Touch`) with a Toast Undo (text held client-side, which is reversible before the reveal). A character count `figures text-caption text-muted` "241/280" appears only from 240 characters. Autosave runs after 600 ms idle and on blur, through `SaveTracker`. A save refused after the reveal shows the toast "Your last edit arrived after the reveal and was not kept."
6. **Pass:** per prompt, `Checkbox touch` "Nothing to add here". It is enabled only when the prompt has no saved text. Checking it collapses the empty field. Unchecking restores it.
7. **Layout 390:** `PageShell` gutters 16 px, one column, sections separated by `border-t border-rule pt-6` with `gap-8` between them. Every control is ≥44 px (`ButtonSize::Touch`, `Checkbox touch`). The textarea is full width (3 rows ≈ 76 px). The presence strip (card 3) sits as one line under the header. No facilitator tools appear on phones.
8. **Layout 1280:** `lg:grid-cols-[minmax(0,42rem)_20rem] lg:gap-12`. The left column holds the prompts at `max-w-2xl`. The right column holds presence and facilitator tools (card 3) with `border-l border-rule pl-8`.
9. **References:** a1 (sealed sentence, absence of others), a2 (prompt type, progress cue), a3 (soft timer).
10. **Perf:** a 2 s poll returns `Unchanged` in most cycles. The page must not re-render the fields on a poll: fields are keyed by entry id, and poll data never touches field signals.
11. **Reduced motion:** identical (nothing moves).
12. **Verify:** stills at 390 (async member) and 1280 (sync member), plus a reading-flow frame. A DOM and payload probe that no other member's text is present. A1 on every control.

### Card 3: Presence strip and facilitator controls

1. **Presence (`PresenceStrip`, c1–c3):** eyebrow "Here now", then a flat pile: `UserAvatar Sm` + username in `text-label text-secondary`, `flex flex-wrap gap-x-4 gap-y-2`, **alphabetical** (stable, never arrival order), deduplicated per user. A present member is only listed. There is no dot, ring, check or typing cue, and absent members are not listed. When the last poll failed: `text-caption text-muted` "Reconnecting…" (c2). It is the same for every member, the facilitator included. `role="list"` with `aria-label="Here now"`. It is not a live region.
2. **Written count (`WrittenCount`, facilitator only):** `figures text-title text-primary` "4 of 5", with `text-label text-secondary` "have written or passed" under it. One number across all prompts. Members (not the facilitator) don't see it. It changes only as text.
3. **Round controls:** a collapsed disclosure (`Button Quiet` "Round controls" with `aria-expanded`, **collapsed by default**, open state kept for the session) containing, separated by `gap-3`:
   - "Start a 3-minute round" (`Secondary`), when no timed round is running;
   - while a round runs: "Add a minute" (`Secondary`) and **"Reveal now"** (`Primary`: the page's one chalk action for the facilitator). An early reveal opens a `ConfirmDialog` (Default): "Reveal now? Writing ends for everyone. 4 of 5 have written." with confirm "Reveal now";
   - "Take over facilitating" (`Quiet`), shown to the leader or a coach who is not the facilitator.
4. **Layout:** **md and wider only** (`hidden md:block`). At md–lg it sits above the prompts as one ruled band (`border-y border-rule py-3`, count and disclosure on one row). At lg it is the right column. It never overlays, is never sticky and never floats. On a phone, the facilitator sees `text-caption text-muted` "Round controls are on a wider screen." under the presence line. That is honest, and it is not a dead control.
5. **References:** c1, c2, c3 (presence). b1 (host reveal).
6. **Perf:** presence comes from the same poll and adds no request.
7. **Reduced motion:** identical.
8. **Verify:** 1280 still of the facilitator collapsed (reading flow) and expanded. A payload probe that `WrittenCount` is one integer pair with no per-prompt or per-user field. A probe that the presence payload carries usernames only, with no last-seen time.

### Card 4: The reveal

1. **Technique:** each prompt becomes a `NoteGroup`: a `SectionHeader` (`h2`, the prompt) with meta `figures text-caption text-muted` "6 notes", then `<ul class="divide-y divide-rule border-y border-rule">` of `ReviewNote` rows. These are ruled notes, not tiles (e1–e3).
2. **`ReviewNote` row:** `py-4 flex flex-col gap-2`. The words are `<p class="note text-reading text-primary">`, rendered as text and never as HTML. Next comes the author line `text-caption text-muted` (attributed prompts only). Then the actions row `flex items-center justify-end gap-2`: `PlusOne`, then "Propose" (`Button Quiet` `Sm` at md+, `Touch` on phones).
3. **Anonymous group:** heading "What didn't go to plan" + caption `text-label text-muted` "Anonymous: nobody on the team sees who wrote these.". Rows **omit the author line entirely**. It is not an empty element. Rows render from `AnonymousCard { id, text, plus_ones, cluster }`, so no author field can exist. `id` is the random record id. No `data-*` other than `data-note-id` (random) appears.
4. **`PlusOne`:** `Button Secondary` with `pressed` = your own +1. The label is "+1", the count sits beside it as `figures text-label text-secondary`, and the accessible name is "Agree, 3 so far". It is **enabled on every card, your own included** (uniform). There is no list of voters.
5. **Order:** by `order_key` (random) in every phase. Never by time, author or votes. There is no re-sort when +1s change.
6. **Minority card:** a note with 0 or 1 +1 renders exactly like the others. No label, no dimming, no "low" marker. The frame strip needs one in shot.
7. **Near-duplicates (facilitator only):** one line at the top of a group, `text-label text-secondary` "2 pairs read alike", with a disclosure listing each pair as two short quoted texts (sans, truncated at 80 characters) plus "Merge" (`Secondary Sm`) and "Keep apart" (`Quiet Sm`). Merge shows a Toast "Merged. Undo".
8. **Merged cluster:** one `<li>` holding the notes in `order_key` order, each with its own author line (none in the anonymous group), separated by `gap-3` with no inner rule. Above them: `eyebrow` "3 said much the same". **One** `PlusOne` for the cluster, counting distinct members. Every member's words stay.
9. **Layout 390:** one column, gutters 16 px, controls 44 px, at most 2 controls per row with `gap-2`. **1280:** left column `max-w-2xl` notes. The right column holds Decide (card 5) and the facilitator count.
10. **Live region:** one polite `role="status"`, present from first paint and empty until the reveal: "Revealed: 14 notes, 5 of 5 voices." **Focus is not moved.**
11. **References:** b1, b2, b3 (one state change), e1–e3 (type).
12. **Perf:** the reveal payload is one response. Render all groups in one reactive swap. Poll diffs replace counts only, keyed by note id.
13. **Reduced motion:** see card 8.
14. **Verify:** stills at 390 and 1280 (attributed group, anonymous group, a merged cluster, a minority card), the frame sequence (card 8), and the string search over the anonymous group's DOM, hydration payload and server-fn JSON in every context.

### Card 5: Decide

1. **Technique:** a "Decide" section (`SectionHeader` h2 "Decide", meta `VotesLeft`). It holds proposals as a ruled list (`divide-y divide-rule border-y border-rule`), proposal text in `text-body text-primary` (sans, with no proposer anywhere).
2. **Propose (from a note):** "Propose" expands an inline `TextField rows=2 maxlength=280` under that note, prefilled with its text, plus "Add proposal" (`Secondary`) and "Cancel" (`Quiet`). Only one is open at a time. It is never a dialog. A proposal made from an anonymous card carries `source` only server-side. Its client struct has **no** source author and no source id.
3. **Vote:** per proposal, `Button Secondary` toggle with `pressed`: "Vote" / "Voted". The count is `figures text-label text-secondary` "3 votes". 1 per member per proposal.
4. **`VotesLeft`:** `figures text-label text-secondary` "2 of 3 votes left", in a polite live region. **Cap reached:** the meta reads "No votes left" and unpressed Vote buttons are `disabled`, with one `text-caption text-muted` line under the heading: "Take a vote back to vote for something else." (d1).
5. **Order:** votes descending, ties by random key. A poll that changes order applies when no control in the list has focus. Otherwise it waits for blur. Focus follows the row (DESIGN §6.6).
6. **Accept (facilitator):** `Checkbox` "Accept" per proposal. The top three are pre-checked when Decide opens. At 3 accepted, the others are disabled, with the caption "Three accepted. Untick one to accept another."
7. **`OwnerChips`** (accepted proposals only, d3): a `role="group"` `aria-label="Owner of: <text>"` row of `Button Secondary` with `pressed`, one per **current member's username**, wrapping with `flex flex-wrap gap-2`. `Sm` at md+, `Touch` on phones. "No owner yet" is in `text-caption text-muted` until one is chosen. The server rejects a username that is not a current member.
8. **Close:** facilitator `Primary` "Close and write the review", enabled when 1–3 are accepted and each has an owner. It opens a `ConfirmDialog` (Default) whose message names the consequence: "This writes the review with 3 action items, owned by ahri_main, leesin_jg and thresh_sup. Notes and votes are locked."
9. **Layout 390:** after the four groups, with a header jump link "Decide (4)" in the meta. Chips at 44 px. **1280:** right column `w-80`, top-aligned, not sticky.
10. **References:** d1 (votes left, disable at zero), d2 (total + per-item cap), d3 (owner chip).
11. **Perf:** votes and accepts are single-row writes, reflected by the next poll.
12. **Reduced motion:** identical.
13. **Verify:** stills at 390 and 1280: proposals, a vote cast, cap reached, three accepted with owner chips. A payload probe for no proposer.

### Card 6: Generated review section, "5 of 5 voices"

1. **Technique:** on `/post-game?review_id=` of a generated review, a read-only section **above** the editable outcome, rating and notes. It uses `SectionHeader` h2 "Review together" with meta `figures` "5 of 5 voices" · closed `LocalTime`. That is the **only** voices line on the page.
2. **Body:** the four `NoteGroup`s, read-only, with the same `ReviewNote` minus actions. +1 counts become `figures text-caption text-muted` "3 agreed". Then "Decided" lists the ≤3 items as rows: text `text-body text-primary` + owner `text-label text-secondary` + link "Open in action items".
3. **Authors** are resolved at read time ("Former member" after deletion). The anonymous group has no names.
4. Optional ground `bg-ink-1` with `border-y border-rule`. No box, no radius.
5. **Layout 390 / 1280:** single column `max-w-2xl` at both widths, inside the existing editor page.
6. **References:** e1–e3.
7. **Perf:** server-rendered.
8. **Reduced motion:** identical.
9. **Verify:** stills at both widths. A DOM probe for exactly one "of N voices" string and no per-member count. "4 of 5" in the fixture where one member passes every prompt. A re-save test (no new action items).

### Card 7: The facilitator's quiet nudge

1. **Technique:** `QuietNudge`, the `NudgeLine` markup (`border-t border-rule pt-3 text-body text-secondary`) without an action link (there is no honest action to link to). It shows the text plus "Don't show this again" (mute key `nudge.quiet_player`, through the existing guidance mute path). It honours the profile's Guidance switch.
2. **Text:** "One player has been quiet in the last 3 reviews." / "Two players have been quiet in the last 3 reviews." The payload is `{ count: u8 }` only.
3. **Placement:** the facilitator column (lg) or band (md), **under the written count, collapsed tools below it**, while the phase is writing. Never on a phone, a member's view, the review page, the share page or Match Day. Not shown with fewer than 3 closed sessions. Not a live region.
4. **References:** DESIGN §6.11 nudges (the in-app precedent, seen in code at `components/guidance.rs:533`).
5. **Perf:** computed on request.
6. **Reduced motion:** identical.
7. **Verify:** 1280 facilitator still. A member-context DOM and payload probe for absence. The two-closed-sessions negative.

### Card 8: Reveal frame sequence spec

1. **Before (t < 0):** the writing view (card 2). The member's own entries are in fields. No one else's text is in the DOM.
2. **t = 0:** the poll returns `revealed`. In **one** render, the writing sections unmount and the reveal wrapper mounts with `opacity-0 transition-opacity duration-(--dur-3) ease-out`. On the next `requestAnimationFrame` the class becomes `opacity-100`.
3. **Mid (t ≈ 110 ms, must be ≤300 ms):** every note in every group is at the same partial opacity. Check: sample the mean luminance of 3 notes in 3 different groups. They must agree within ±2 %. Disagreement means a stagger, which is a FIX.
4. **After (t ≥ 300 ms):** full opacity. Layout is final and identical to a fresh server render of the revealed state, which is V5's first frame.
5. **Reduced motion:** at t = 0 the wrapper mounts at `opacity-100` with no transition. The first captured frame is final.
6. **Capture:** Playwright with `page.clock` or a 6-frame strip at 0/50/110/170/230/400 ms after the poll response, at 1280 and 390. One extra run with `emulateMedia({reducedMotion:'reduce'})`. The strip is assembled with ffmpeg into `publish-round-r1/reveal-strip-{1280,390,reduced}.png`.
7. **Budget:** the swap must not drop a frame on a mid-tier phone (≤ 1 long task over 50 ms during the swap, per a Performance trace).

---

## 7. Rubric

- **Harness rows A1–A4, I1–I8, V1–V3:** `node /home/ops/Repositories/garden/wiki/assets/ux-rubric/rubric-check.mjs --app-dir <repo>/e2e <targets>` (Playwright resolves from `e2e/node_modules`), plus `./scripts/lint_design.sh` (hard zero). **Harness caveat:** the harness opens pages without a session. So the multi-user spec saves each state's post-hydration HTML (`page.content()`, scripts stripped, `<base href="http://127.0.0.1:3020/">` injected so `/pkg/*.css` loads) per context and viewport, and the harness runs on those files. A3 on a static snapshot proves nothing, so A3 is judged on the live frame sequence (card 8) as well. **V1 caveat:** the harness exempts only stylesheets named `tokens.css`. The app's tokens ship inside the Tailwind bundle, so a V1 FIX that points at the `@theme`/preflight block is a harness false positive. Report it as such, and don't change CSS to satisfy it. A1: controls under 44 px at md+ (`Sm`/`Md`) must keep ≥8 px to their neighbour (`gap-2` minimum).
- **Sight rows (atelier):** **V5** (the revealed page's first frame reads as the reveal), **V7** (§5 bans), **V8** (render against the references and the recipe parameters; see the gap in `references.md`).
- **Dropped, one reason each:** Vo1–Vo4 (the surface has no voice path); T1–T4 (nothing is agent-authored: no LLM clustering or summary); V4 (its 150–500 ms band is the vault's, while the app's is DESIGN §3.7, 100–300 ms; card 8 checks the reveal instead); V6 (no high-density view).

## 8. Q11: interruption, confirm and undo

- **The surface can interrupt no one.** No notification, sound, badge, tab-title change or red dot. The reveal arrives on the next poll for those already on the page. Others see it when they open the session or the thread step.
- The nudge is inline, facilitator-only and muteable (card 7). The guidance switch turns it off.
- **Confirms (2 paths, ≤1 confirm each, I5):** an early "Reveal now" (it locks everyone's writing) and "Close and write the review" (it creates the review and ≤3 action items). Both are `ConfirmDialog` Default, and the message names the consequence.
- **Undo:** "Remove" on your own entry and "Merge" use Toast Undo. A +1, a vote or an accept is undone by pressing it again. **Close has no undo.** Its corrections go through existing paths: the action items are editable and deletable on `/action-items` (with their Undo), and outcome, rating and notes stay editable in the review. The confirm text states this.

## 9. Review regime

The builder renders. The atelier, in a **fresh context** (it wrote these cards and does not judge their first render in the same context), critiques the frame strips of all six surfaces plus the reveal sequence against these cards, with the capture protocol and the rubric table, **before** anything goes to Jasper. Then comes Jasper's tap on the round card. `taste-rounds-max: 2`. A round ends only on atelier PASS **and** the tap. If round 2 does not pass, it goes to REPORT as open, without a third round.

## 10. Per-screen done sentences (stricter than the criterion, never in its place)

1. **Thread step:** at 390 and 1280, a started match's review step offers "Review together" as its one chalk action and "Write it alone" as a quiet link, and shows no written count to anyone.
2. **Writing:** at 390 a member writes, passes and removes with 44 px controls under a minute-granular deadline line. The page holds nothing of anyone else, not even a placeholder.
3. **Facilitator:** at 1280 the facilitator sees "Here now" names with no state markers, one "4 of 5" across all prompts, and collapsed round controls that cover no note.
4. **Reveal:** every note becomes visible in one 220 ms opacity change on one element (instant under reduced motion). Notes are serif ruled rows with sans author captions. The anonymous group's rows are byte-identical in structure for every viewer, the author included. A minority note looks exactly like the rest.
5. **Decide:** a member sees "N of 3 votes left", cannot pass 3, and cannot vote twice on one proposal. The facilitator accepts at most 3, each with an owner chip from the current members, and closes through one confirm that names the owners.
6. **Generated review:** exactly one "N of M voices" line and no per-member number anywhere on the page. The four groups are read-only, in the same type. The decided items link to action items.
7. **Nudge:** at 1280, only the facilitator's writing view shows one muteable count-only line, and it is absent with two closed sessions.
