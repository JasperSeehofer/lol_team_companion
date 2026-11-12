# M150 atelier recipe cards, round 1

Atelier, 2026-11-12. Written from the confirmed intake (garden `M150-game-plan-suggestion-chips.md`, "Atelier intake", confirmed by Jasper 2026-10-20) and the app's own tokens only (`input.css` `@theme`, `docs/DESIGN.md`). No render existed when these were written.

**Director/critic conflict (playbook §5.7).** This context drafted the cards, so the final critique of the live captures must come from a fresh-context atelier run.

## Shared ground (applies to every card)

- **Tokens that exist.** Surfaces `bg-page bg-ink-1 bg-ink-2 bg-ink-3`. Lines `border-rule` (hairline) and `border-outline` (3:1, anything interactive). Text `text-primary text-secondary text-muted`. Chalk `text-accent bg-accent text-accent-ink`. Type `text-caption` (12), `text-label` (13), `text-body` (14), `text-reading` (16), `font-display`, and the utilities `note` (Source Serif 4 italic), `eyebrow`, `figures`. Radius `rounded-ctl` (3 px). Motion `--dur-1` 100, `--dur-2` 160, `--dur-3` 220, `--dur-4` 300, plus `ease-out` (Tailwind's default timing, `cubic-bezier(0.23,1,0.32,1)`). Layers `z-(--z-dialog)`. `ButtonSize::Touch` (`h-11 … md:h-8`) already exists.
- **No keyframes exist.** `--animate-*` is `initial`, so all motion here is CSS transitions driven by class swaps (Leptos signal, then one `requestAnimationFrame`) or by Tailwind v4's `starting:` variant. Do not add `@keyframes` or animation tokens.
- **Reduced motion.** The global rule (`input.css`, `@media (prefers-reduced-motion: reduce)`) already cuts transitions to 0.01 ms. Every card below also states what the reduced variant shows, and every moving class also gets `motion-reduce:transition-none`.
- **Lint.** `scripts/lint_design.sh` must stay at zero. `rounded-full` is allowed only under `src/components/ui/`, so the pin-dot and the strength dots must live in a primitive (card 1).
- **Bans (mission "Never" list plus playbook §4).** No sparkle or AI glyph, no "Suggested by AI", no percentages or "confidence", no gradient pills, no pill shape at all, no drop shadow, no new colour token. No toast on accept. Nothing appears while typing and nothing writes without a tap. No Inter. No fade-up on scroll. Chalk is never resting on a chip or a card.
- **Feedback hooks.** Use `data-feedback-label="Suggestion chip"` on each chip root and `data-feedback-label="Suggestion card"` on the card or sheet body.

---

## Card 1: Chip, a paper tag in ink

**Technique.** A stationer's paper tag pinned to a board: a small rectangle of slightly lighter paper, a pin-dot near its left edge, set down by hand at a fraction of a degree off true. It is a tag, not a pill. It never uses chalk (chalk means *yours*, and the chip is the app's).

**References.**
- Intake row "Chip: a paper tag, not a pill — slightly rotated label with a pin-dot".
- DESIGN §3.5: 3 px for controls; round only for avatars, toggles and markers, so the dot is a marker inside a primitive.
- DESIGN §3.6: elevation is tone, so the tag is one tone lighter than the page.

**New primitive.** `src/components/ui/suggestion.rs`, `SuggestionChip`, re-exported from `components::ui`, with a gallery section "Suggestion chip" (add it to `SECTIONS` in `design.rs` and in `gallery.spec.ts`). A primitive is required because of `rounded-full` (lint) and because the component contract wants a `force` hook.

Props:
- `claim: String` (60 characters or fewer, rendered as text, never as HTML)
- `source: ReasonSource`
- `index: usize` (picks the tilt)
- `expanded: Signal<bool>`
- `controls: String` (the id of the card or sheet)
- `on_open: Callback<()>`
- `force: Option<Force>` (reuse `button::Force`: Hover, Focus, Active)

**Markup.**

```
<li class="shrink-0 snap-start">
  <button type="button" aria-expanded aria-controls={controls} data-force=…
          data-feedback-label="Suggestion chip" class=CHIP + TILT[index % 3]>
    <span aria-hidden="true" class="size-1.5 shrink-0 rounded-full bg-muted"></span>   // pin-dot
    <span class="min-w-0 line-clamp-2 text-left text-label font-medium text-primary md:line-clamp-1">{claim}</span>
    <span aria-hidden="true" class="figures shrink-0 text-caption text-muted">{glyph}</span>
    <span class="sr-only">{", from " + source.label()}</span>
  </button>
</li>
```

Classes:
- `CHIP` = `inline-flex min-h-11 max-w-[16rem] items-center gap-2 rounded-ctl border border-outline bg-ink-3 py-1.5 pl-2.5 pr-2 transition-colors duration-[160ms] ease-out hover:bg-ink-2 hover:border-secondary active:translate-y-px aria-expanded:border-primary aria-expanded:bg-ink-2 data-[force=hover]:bg-ink-2 data-[force=hover]:border-secondary data-[force=active]:translate-y-px motion-reduce:transition-none md:min-h-8 md:max-w-[24rem]`
- `TILT` is a `match` over three literal arms: `rotate-[-0.6deg]`, `rotate-[0.4deg]`, `rotate-[-0.3deg]`. The tilt is deterministic by index and never random per render, so SSR and hydration match.

**Evidence glyph** (mono, `aria-hidden`; the source is spoken by the sr-only text). `Rule` is "§", `TeamNote` "¶", `MatchData` "Σ", `Pool` "◇", `Scouting` "※". Every one is outside the emoji lint range. M150's path emits `Rule` only; the others are mapped so the primitive is total.

**Sizes.**
- Pin-dot: 6 px.
- Height: 44 px below `md`, 32 px from `md`.
- Width: up to 256 px on phones (2 lines) and 384 px on desktop (1 line). Sixty characters at 13 px is about 360 px.
- Gap between the pin-dot, the claim and the glyph: 8 px.

**States.**
- Default: `bg-ink-3`, `border-outline`.
- Hover: `bg-ink-2` with a `border-secondary` edge (the tag sinks into the board; the tilt does not change).
- Focus-visible: the global 2 px chalk outline, 2 px offset. Do not add focus classes; the outline is the only chalk on a chip.
- Pressed: `translate-y-px` for 100 ms.
- Expanded (card open): `bg-ink-2`, `border-primary`, `aria-expanded="true"`.
- Disabled: there is none. A chip that cannot act is not rendered.
- Reduced motion: no transition. The tilt stays, because it is static.

**Motion.**
- None at rest.
- Retire on accept or dismiss: `opacity-0` over 160 ms `ease-out` (`transition-opacity`), then the element is removed. The row re-flows instantly, with no layout animation.

**A11y.**
- Native button; its name is the claim plus ", from Trait rules".
- `aria-expanded` and `aria-controls` point at the card or sheet.
- Each chip is one Tab stop (three at most, so no roving tabindex). Enter or Space opens the card.
- The doc comment gets the `A11y:` line: "A native `<button>` named by its claim and source; `aria-expanded`/`aria-controls` tie it to its card; tilt, pin-dot and glyph are decorative (`aria-hidden`); 44 px tall below `md`."

**Bans that apply.** No pill (`rounded-full` on the chip body), no chalk at rest, no sparkle glyph, no "AI", no percentage, no shadow, no gradient. A tilt larger than 1° reads as a gimmick.

**Pass/fail (frames at 390 and 1280).**
- [ ] The chip reads as a rectangular tag at 3 px corners, never a capsule.
- [ ] The tilt is visible at 1280 when you look for it and invisible at a glance (at most 0.6°). Three chips have three different tilts.
- [ ] Text is crisp at 100% zoom (no smeared sub-pixel text from the rotation; if it smears at 1280, reduce to ±0.3°).
- [ ] No chalk pixels on any chip except the focus outline.
- [ ] Text contrast is at least 4.5:1 and the chip edge at least 3:1 against `bg-page`, recomputed from pixels.
- [ ] Chip height is 44 px or more at 390 and 32 px at 1280.

---

## Card 2: Card, a margin note

**Technique.** A pencilled note in the margin of a printed brief. One hairline above it, the reasoning set in the serif italic the app reserves for notes, and the facts in small sans beneath. It is a note, not a panel: no box, no fill, no shadow.

**References.**
- Intake row "card: margin-note language from proposal 8 (serif italic, hairline rule)".
- DESIGN §3.3: `note` is for annotations.
- DESIGN §6.7, the Draft Graph Why panel's per-source grouping and "rule id · weight" caption.
- `ReasonSource::label()` (`src/models/graph.rs:526`).

**Placement.**
- From `md` up: inline under the chip row, in page flow. It pushes content down and never covers it; it is not a popover.
- Below `md`: the same body inside the bottom sheet (card 5).
- One card open at a time. Opening another chip swaps the body.

**Markup (desktop).**

```
<div id="plan-suggestion-card" role="region" aria-labelledby="plan-suggestion-claim"
     data-feedback-label="Suggestion card"
     class="grid grid-rows-[0fr] transition-[grid-template-rows] duration-[180ms] ease-out motion-reduce:transition-none
            data-[open=true]:grid-rows-[1fr]" data-open=… inert=!open>
  <div class="min-h-0 overflow-hidden">
    <div class="mt-3 flex max-w-[36rem] flex-col gap-2 border-t border-rule pt-3">
      <p id="plan-suggestion-claim" class="text-body font-medium text-primary">{claim}</p>
      <p class="note text-reading text-secondary">{reason.text}</p>
      <p class="flex flex-wrap items-center gap-x-2 gap-y-1 text-caption text-muted">
        <span>{source.label()}</span><span aria-hidden="true">·</span>
        <span>{pair}</span><span aria-hidden="true">·</span>
        <StrengthDots value=1..=3 />
      </p>
      <div class="mt-1 flex flex-wrap gap-2">
        <Button variant=Secondary size=ButtonSize::Touch on_click=add>"Add to plan"</Button>
        <Button variant=Quiet size=ButtonSize::Touch on_click=dismiss>"Not for us"</Button>
      </div>
    </div>
  </div>
</div>
```

**Pair.**
- Pair rules: "Leona and Orianna", the two names as text. Optionally precede them with two `ChampionAvatar` at 24 px (existing primitive, square, monogram fallback).
- Comp claims: "Our picks" or "Their picks".

**`StrengthDots`** (same file as the chip; it needs `rounded-full`):
- `<span role="img" aria-label="Strength 2 of 3" class="inline-flex items-center gap-1">` around three spans of `size-1.5 rounded-full`.
- Filled dots: `bg-secondary`. Empty dots: `border border-muted`.
- No numbers and no word. The dots-versus-word choice is a react question on the round card.

**Sizes.**
- Body at most 576 px wide (`max-w-[36rem]`, 60 to 75 characters of `note` at 16 px).
- Hairline 1 px `border-rule`; 12 px space above and below the rule.
- Buttons 32 px from `md`, 44 px below.

**States.**
- Closed: zero height and `inert` (in the DOM, so SSR-safe).
- Open: 1fr.
- Buttons inherit the `Button` states, including focus, hover, `loading` (while the autosave call is in flight, it keeps its width via `reserve`) and disabled.
- "Add to plan" is **Secondary**, not Primary. The view's one chalk action stays the header's save (DESIGN §4, `:463`).
- Reduced motion: opens and closes instantly.

**Motion.** Expand and collapse use `grid-template-rows` over 180 ms `ease-out`. Nothing else moves.

**A11y.**
- The region is named by the claim.
- Focus moves to the claim `<p tabindex="-1">` when the card opens from a chip.
- Escape closes it and returns focus to the chip.
- After **Add to plan** or **Not for us**, focus moves to the next remaining chip, else to the collapsed line's button (card 3). It never goes to the textarea, because that would raise the phone keyboard.
- One persistent `<p class="sr-only" aria-live="polite">`, mounted before any text (rule 62), says "Added to Win conditions." or "Dismissed for this plan." This is not a toast.

**Bans that apply.** No box, fill or shadow around the card. No coloured left stripe. No "AI", no percentages, no explanation prompt on "Not for us". Exactly two actions: no "Learn more" and no close X (the chip toggles it).

**Pass/fail.**
- [ ] At 1280 the open card pushes Strategy down and overlaps nothing.
- [ ] The reason is serif italic and is the only serif on the card.
- [ ] The source label is one of the five `label()` strings, verbatim.
- [ ] The pair names both champions.
- [ ] There are exactly two buttons, and neither is chalk-filled.
- [ ] The strength reads as three dots with an accessible name and no number.
- [ ] Line length is 75 characters or fewer.

---

## Card 3: Collapsed line and empty-side line

**Technique.** The plan's own quiet sub-label (the "Strategy" label in `StrategySection`, `text-label font-medium text-secondary`) turned into a disclosure. It is not a section heading: it belongs to Matchup.

**References.**
- Intake row "collapsed line: the plan's own section-label style".
- `game_plan.rs` `StrategySection` sub-label.
- The caption lines `plan-thread*` (`text-caption text-muted`).

**Placement.** Wrap `<MatchupSection>` and the suggestions block in one `<div class="flex flex-col gap-4">`, so the line sits 16 px under Matchup and not in the 48 px gap that separates sections.

**Markup.**

```
<section aria-label="Suggestions" class="flex flex-col border-t border-rule" data-testid="plan-suggestions">
  <button type="button" aria-expanded aria-controls="plan-suggestion-row"
          class="flex min-h-11 w-full items-center gap-2 text-left text-label font-medium text-secondary
                 transition-colors duration-[160ms] ease-out hover:text-primary motion-reduce:transition-none md:min-h-8">
    <span aria-hidden="true" class="figures text-caption text-muted">{"▸" | "▾"}</span>
    <span>{"3 suggestions" | "1 suggestion"}</span>
  </button>
  <div id="plan-suggestion-row" class="grid grid-rows-[0fr] transition-[grid-template-rows] duration-[180ms] ease-out
       motion-reduce:transition-none data-[open=true]:grid-rows-[1fr]" data-open=… inert=!open>
    <div class="min-h-0 overflow-hidden"> …chip row (card 5)… card (card 2)… </div>
  </div>
</section>
```

**Variants (literal strings).**
- Both drafts in, N ≥ 1: the button reads "N suggestions" (singular for 1). **Collapsed by default on every load**; open state is not persisted.
- A side incomplete: no button. `<p class="py-2 text-caption text-muted border-t border-rule" data-testid="plan-suggestions-wait">"Suggestions appear when both drafts are in."</p>`
- Both in, none left (all dismissed or no rule fires): `<p class="py-2 text-caption text-muted border-t border-rule">"No suggestions for these picks."</p>` (the honesty rule). The executor records the wording in REPORT.
- No trait table loaded: nothing at all.
- Loading: nothing. The line appears when the resource resolves, so no skeleton flicker sits under Matchup.

**States.**
- Hover: `text-primary`.
- Focus-visible: the global chalk outline.
- Expanded: the caret becomes ▾. The count does not change while open; it drops after Add or Dismiss.
- Reduced motion: instant.

**Motion.** Expand over 180 ms `ease-out` (grid rows). Nothing else.

**A11y.**
- A real button with `aria-expanded` and `aria-controls`. The section is named "Suggestions". The caret is `aria-hidden`.
- Nothing inside the row takes text input.
- The count is part of the button name, so it needs no live region.

**Bans that apply.** It is not an `h2`. No badge or count bubble, no chalk count, no animation on load, no auto-expand.

**Pass/fail.**
- [ ] Reading-flow frame (nothing expanded) at 390 and 1280: one 13 px line under Matchup, 44 or 32 px tall. Strategy starts directly after it, with nothing covered.
- [ ] The wait line shows with one side empty, in 12 px muted text.
- [ ] The count matches the chips.

---

## Card 4: The written line, the signature moment

**Technique.** Chalk writing that dries to ink. The claim is *written* into the field left to right as a reveal (a pen stroke, not a slide-in object). It is chalk only while it is being written, then it settles into the field's own ink. A textarea cannot colour one line, so the colour lives on a transient **echo overlay**: a `div` laid exactly over the field's last line, which then removes itself.

**Chosen mechanism: the echo overlay, not an edge flash.** An edge flash on `border-accent` is indistinguishable from focus and says nothing about *which* line is new.

**References.**
- Intake row "chalk line: proposal 6's accent, hand-written feel for the first 300 ms only".
- Intake row "Signature effect".
- `garden-visualization-plan` §6–7, the "ink drying" motion precedent.
- DESIGN §3.7: 100 to 300 ms, `ease-out`.

**Steps.**
1. The Win-conditions `TextField` (or the Notes one) gets a wrapper `<div class="relative">` from the page. The `TextField` itself is unchanged.
2. On **Add to plan**, append `"\n" + claim` to `ed.win` (no leading newline if the field is empty).
3. Set the textarea's `scrollTop = scrollHeight`. This is its internal scroll only; never scroll the page.
4. Show the echo only if the textarea's bottom edge is inside the viewport (`getBoundingClientRect().bottom <= innerHeight`). Otherwise skip it; the line is still written and the live region still speaks.
5. Echo element, absolutely placed over the last line with the textarea's exact text metrics:
   `<div aria-hidden="true" class="pointer-events-none absolute inset-x-px bottom-px rounded-ctl bg-ink-2 px-3 pb-2 text-body leading-relaxed whitespace-pre-wrap break-words text-accent [clip-path:inset(0_100%_0_0)] transition-[clip-path,color] duration-[180ms] ease-out motion-reduce:transition-none">{claim}</div>`
   It mirrors `CONTROL` plus `py-2 text-body leading-relaxed` in `field.rs`. Being anchored to the bottom makes a claim that wraps to two lines still align.
6. Frame 0: mount with `[clip-path:inset(0_100%_0_0)]`. On the next `requestAnimationFrame`, swap to `[clip-path:inset(0_0_0_0)]`. This is the write, 180 ms.
7. At 180 ms, swap `text-accent` to `text-primary` (with `duration-[120ms]` on the colour). The chalk dries to ink, which is the field's own `text-primary`.
8. At 300 ms, remove the echo. The identical text underneath is now visible, so the removal is invisible.
9. The chip retires at the same moment (card 1, 160 ms opacity).

**Parameters.** Total 300 ms or less. Write 180 ms, dry 120 ms, curve `ease-out`, horizontal reveal 0 to 100%. No translate, no scale, no glow. Chalk is on screen for 300 ms at most and never at rest.

**Reduced motion.** No reveal. The echo mounts already fully shown in `text-accent`, stays 300 ms, and is removed. The state change is kept and the movement is gone, per the global rule. If even that is unwanted, skip the echo.

**Desktop or phone.** On phones the sheet closes first (Dialog close), then the echo runs if the field is visible. Usually it is not at 390; that is accepted and noted in REPORT.

**A11y.**
- The echo is `aria-hidden`. The live region says "Added to Win conditions." (or "…to Notes.").
- The line is plain text in the field with no marker, so it is fully editable at once and stays editable during the echo (`pointer-events-none`).

**Bans that apply.** No toast. No auto-focus into the field. No stored marker. No highlight left behind. No sparkle. No chalk after 300 ms.

**Pass/fail (three-frame strip: chip, mid-write about 90 ms, settled about 350 ms; plus the reduced variant).**
- [ ] Mid-write: the chalk text is partly revealed left to right, exactly over the new line (no doubled or offset glyphs; offset of 1 px or less).
- [ ] Settled: the line is the same ink as its neighbours, with no chalk pixels, and the echo is gone from the DOM.
- [ ] The page scroll position is unchanged.
- [ ] Reduced: no intermediate frame, the chalk is static for 300 ms or less, then ink.
- [ ] A typed edit right after Add works (the field is not blocked).

---

## Card 5: Phone row and bottom sheet

**Technique.** Tags laid out along a rail you can slide (one row, edge-bleed so a third tag peeks), and a sheet pulled up from the bottom edge like a drawer, holding the same margin note.

**References.**
- Mission "On the phone the chips are one horizontally scrolling row with 44 px targets and the card is a bottom sheet".
- DESIGN §3.4: 16 px phone gutters.
- DESIGN §6.2 `Dialog`.

**Row (all widths, one list).**
`<ul role="list" class="-mx-4 flex gap-2 overflow-x-auto overscroll-x-contain snap-x snap-mandatory px-4 py-2 md:mx-0 md:flex-wrap md:overflow-visible md:px-0 md:snap-none">`
- The 8 px vertical padding keeps the tilt corners and the 2 px focus outline from clipping.
- The negative margin bleeds to the screen edge inside `PageShell`'s 16 px gutter.
- The scrollbar is left as is (no hiding hacks).
- Chips are 44 px tall below `md` (card 1). With three chips at 256 px or less, the second or third peeks at 390, which is the scroll cue.

**Sheet: recommendation.** Add a **`Dialog` variant `DialogSize::BottomSheet`**, not a new primitive.
- It inherits `showModal()`, the focus trap, Escape, the inert page, the scrim, scroll lock and focus return, all already tested.
- The only new things are a size class and the `starting:` entry.

Size class (literal, in the `match`): `"mx-0 mb-0 mt-auto max-h-[85dvh] w-screen max-w-none rounded-none border-0 border-t border-outline shadow-none pb-[env(safe-area-inset-bottom)] transition-transform duration-[220ms] ease-out starting:open:translate-y-8 motion-reduce:transition-none"`
- `shadow-none` honours the mission's no-drop-shadow ban; the opaque `scrim` does the separation.
- Only a 32 px rise on entry, at `--dur-3` (tray). Close is instant.

The page renders the sheet only below `md` (the card's open signal drives `Dialog open` when `!matches_md`; the inline card from `md`). Body: card 2's inner block without its `border-t`. `title` = the claim (the Dialog's `h2`), no description, `footer` = the two buttons at `ButtonSize::Touch` with `full_width`, "Add to plan" first. `dismiss_on_scrim` true.

Extend `Dialog`'s `A11y:` line: "…`BottomSheet` is the same modal anchored to the bottom edge for phones; it rises 32 px in 220 ms (instant under reduced motion), and Escape, the scrim and the chip all close it."

**Force hook.**
- `SuggestionChip` carries `force` (card 1).
- The sheet uses the Dialog gallery pattern: a gallery button "Open bottom sheet" in the Dialog section, which `gallery.spec.ts` clicks and captures. `Dialog` has no `force` today, and this keeps parity.

**States.**
- Row: scroll, snap.
- Sheet: closed, or open with an entry transition.
- Buttons: `Button` states, including `loading` on Add.
- Reduced motion: the sheet appears in place, and the row scroll has no smooth behaviour (global rule).

**A11y.**
- The row is `role="list"` and each chip is an `li`.
- The sheet is a modal dialog named by the claim. Focus goes to the first button. Escape closes it.
- After Add or Dismiss the sheet closes and focus goes to the next chip or the collapsed-line button (card 2 rule; Dialog's own return target may be gone, rule 62).

**Bans that apply.** No drag handle pretending to be draggable (there is no drag gesture, so draw no grabber). No blur on the scrim. No rounded top corners beyond 0 (structure). No shadow.

**Pass/fail (390 by 844 and 1280 by 800).**
- [ ] At 390 the chips are one row, 44 px or taller, with a peek of the next chip and no page-level horizontal scroll (`document.scrollingElement.scrollWidth <= 390`).
- [ ] At 1280 the chips wrap, with no scroll container.
- [ ] At 390 the sheet sits on the bottom edge, at 85% of the viewport height or less, with a hairline top and no shadow. Both buttons are full width and 44 px tall.
- [ ] At 1280 no sheet ever opens; the inline card is used.
- [ ] Escape and a scrim tap close the sheet. Focus lands on a live element.

---

## Critique protocol to run later (fresh context)

- Captures at 390 by 852 and 1280 by 800: the reading-flow frame, the row opened, the card (desktop), the sheet (phone), the wait line, and an accepted line.
- The three-frame accept strip, plus a run with `prefers-reduced-motion: reduce` forced.
- Contrast from pixels.
- `node wiki/assets/ux-rubric/rubric-check.mjs --app-dir <repo>/app <render-path>` (garden; A1–A4, I1–I8, V1–V3).
- `scripts/lint_design.sh` at zero.

## Tokens and gaps found

- **No "paper" colour.** The nearest is `bg-ink-3` (one tone above the field's `bg-ink-2`). Use it; do not add one.
- **No animation or keyframe tokens.** Use transitions only (above).
- **No resting "yours" token.** Chalk (`text-accent`) is used transiently only, as ruled.
- **Bottom-sheet size.** `DialogSize` has no bottom variant; `Sheet` is full-viewport (the phone menu), so add `BottomSheet` as a variant.
- Nothing else is missing. `ButtonSize::Touch`, `sr-only`, `figures`, `note` and `line-clamp-*` all exist.
