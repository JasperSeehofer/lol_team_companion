# M159 atelier critique r1: recipe cards (item 0)

Scope: four new pieces inside Match Day's CURRENT layout (DESIGN 6.9). No redesign (L2). Tokens from `input.css`/DESIGN only.
Basis: DESIGN 3.4/3.5/3.7/6.6/6.9/6.11, `scripts/lint_design.sh`, `ui/{prep,badge,toggle,table,button}.rs`, M148 frame `match-day-phone-390.png` (reviewed: rows are ruled, no cards, mono caption statuses on the right, chalk button as the page's one action).
Process note: this card set was written by the atelier directly (no subagent tool was available in this run); no app code was touched and nothing was rendered, so these are recipes, not verdicts. Critique the built render in a fresh-context run.

## Hard constraints (all cards)
- No cards, no shadows (only `shadow-overlay` for floating), no gradients, no blur, no emoji, no `rounded-full` outside primitives, no text under 12px (`text-caption` is the floor), no `text-muted/60`, no `border-outline/N`, no raw palette classes, no `animate-*` (skeleton only), no ease-in. Radius is `rounded-ctl` (3px) or none.
- Every row/target >= 44px below `md` (`min-h-11`, or `ButtonSize::Touch`; `md:min-h-8` allowed on desktop controls). Hairlines `border-rule`; controls `border-outline`.
- Times via `LocalTime` (UTC + "UTC" mark server-side, viewer zone after hydration). Never format times by hand.
- One chalk (`bg-accent`) action per view: Match Day already has it (next prep step / Add the next match). None of the new pieces may be chalk.
- Honest emptiness: `EmptyState`/plain line saying what will appear; never invented counts. Load failure = `LoadFailed` callout with "Try again", never an empty state.
- Team scoping: no team id from the client; RSVP and match_id are resolved against the session's team server-side (cross-tenant case each in tests/integration).
- "G1 ✓": U+2713 is explicitly allowed by the lint emoji rule (exempt along with 2605/2606/2715/2717), so the glyph is lint-safe. BUT do not rely on glyph alone: render it in an element with visible text for AT: `<span aria-hidden>✓</span><span class="sr-only">done</span>`, or simply write words. Recommended compliant form: "G1 done · G2 to do · G3 to do" in `text-caption font-mono`, with the done games in `text-success` and the rest in `text-muted` (never colour alone: words carry it). Do NOT use ResultGlyph for this: it means win/loss, and a Bo3 step readout is reviewed/not reviewed, not won/lost.

---
## Card 1. UpcomingMatches (compact list under the next-match card)

**Composes:** `SectionHeader` (title "Upcoming matches", count in mono caption right), a plain ruled list (`<ul class="border-t border-rule">`, each `<li class="border-b border-rule">` containing a `RowLink`-style `<a>`), `Badge` (kind, format), `LocalTime`. Not a `Table` on phones (columns do not fit 390).
**Placement:** directly below the next-match header/prep block, above the right-margin items on phones; inside the left column under the `PrepSequence` from `lg`. Never overlays, never inside the match header. Collapsed by default when more than 3 rows: show 3 and a quiet `Button` "Show all (n)" (`ButtonSize::Touch`, Quiet, `aria-expanded`, `aria-controls`). The current match itself is not repeated in the list.
**Row (phone 390):** `min-h-11 py-2 px-0 flex flex-col gap-1`
- line 1: `LocalTime` in `text-label font-semibold text-primary` ("Sat 17 Oct, 18:30") + right-aligned `text-caption font-mono text-secondary` "4 of 5 confirmed".
- line 2: `Badge` Neutral "Scrim" or Accent-less "Official" (use `BadgeTone::Info` for official, `Neutral` for scrim; never Accent, which is the chalk), `Badge` Neutral "Bo3", opponent name `text-body text-secondary truncate min-w-0`.
- hover/focus: `hover:bg-ink-1 focus-visible:` default focus ring from the primitives (do not restyle); no translate or scale.
**Row (1280):** one line `grid grid-cols-[10rem_5rem_4rem_1fr_auto] items-center gap-3 md:min-h-8 md:py-2`; same content in columns; confirmed count right in `figures`/mono.
**Link:** `href="/?match_id=<id>"`, plain navigation (a real `<a>`, works before hydration). On open, the page's header becomes that match and a quiet "Back to next match" link (`/`) appears in the header meta; the upcoming list highlights the open row `bg-ink-2` + `aria-current="page"` (not colour alone: also semibold).
**States:** none upcoming beyond the current: render nothing (the current-match card covers it; no empty box). Loading: `SkeletonGroup` 2 rows of `h-11`. Error: `LoadFailed` "Could not load upcoming matches" + Try again. Unknown/foreign `match_id`: fall back to the next match with a one-line `Callout` info "That match is not on your team's schedule." (same text as missing; no leak).
**Count text:** "4 of 5 confirmed" counts Yes only; if no roster of five, "n confirmed". Never show "0 of 5" before anyone could answer; show "No answers yet" (`text-muted`).
**A11y:** `<nav aria-label="Upcoming matches">`; each link's name reads "Sat 17 Oct 18:30, scrim, best of three, versus Riverside Rooks, 4 of 5 confirmed" (put the full sentence in visually-hidden text, visible chunks `aria-hidden` only if you duplicate it).
**Do NOT:** make rows cards or add avatars; put dates as relative-only; use a horizontal scroller; truncate the time or count (truncate the opponent only); add a second chalk button; use a `<select>`.

---
## Card 2a. Review match picker rows (post_game.rs)

**Composes:** a listbox-style radio group of rows (use `role="radiogroup"` with native `<input type=radio>` visually integrated, so form/Enter semantics stay), `ChampionAvatar` (size sm, round, from `/img/champion/.../icon`), `ResultGlyph`, `LocalTime`, `Field` label "Which game is this review of?", `Badge` for "Near the scheduled match".
**Row:** `min-h-11 md:min-h-8 flex items-center gap-3 border-b border-rule px-2` ; selected: `bg-ink-2 font-semibold` + radio checked (not colour alone).
- date: `text-caption font-mono text-secondary w-24` (LocalTime, "17 Oct").
- our champions: up to 5 `ChampionAvatar` 24px in a row, `gap-1`; on 390 show the signed-in/our five as avatars only if room, else the review author's champion + "+4" `text-caption`. `aria-label` lists champion names.
- result: `ResultGlyph` (win/loss; the only place W/L shows) with visible "W"/"L" text beside, `text-label`.
**Order:** newest first; games within +-24h of the scheduled match are lifted to the top under an eyebrow "Near the scheduled match" (`eyebrow text-muted`), then eyebrow "Other recent games". Max 20 then "Show more" (Touch quiet button).
**Last row:** "Not in the list (scrim / custom)" as a radio row, `text-secondary`, same height; choosing it reveals the existing free fields (no Riot id). Default selection: the first near-match row if exactly one, else none (never preselect a guess silently; a guessed one is labelled "Closest to the match time").
**States:** empty (no synced games): `EmptyState` "No synced games yet. Link your Riot account on your profile, or choose Not in the list" with the custom radio still present. Loading: 3 `Skeleton` rows. Error: `LoadFailed`; custom choice stays usable so the review is never blocked.
**A11y:** radiogroup with arrow-key navigation (native radios), Space selects, focus stays on the group after load; selected row announced "selected, win, 17 Oct".
**Do NOT:** show a raw Riot match id anywhere; use a `<select>`; colour W/L with success/danger fills across the row; put avatars of 10 players.

## Card 2b. "Review as a team" action
- Match detail header: a `Button` Secondary in `PageHeader` actions, label "Review as a team", `ButtonSize::Touch` (44px phone). Never chalk (the chalk slot, if any, is the page's existing one). On Team games rows (`Table`): a quiet text `RowLink`-adjacent `Button` Quiet, `ButtonSize::Touch` below `md`, in its own last `Td` (`Align::Right`); the row's main `RowLink` to `/match/:id` stays; two separate targets, 44px each, `gap-2`. Links to `/post-game?match=<opaque id>` (server resolves; never a team id). If a review for that game already exists: label becomes "Open team review" (same slot).
- A11y: name includes the game: `aria-label="Review as a team: game of 17 Oct, win"` on table rows (many identical labels otherwise).
- Do NOT: put it in a `Menu` only; duplicate it as an icon; show it for games the team did not play.

## Card 2c. Bo3 step readout inside PrepSequence
- Uses existing `PrepStep` `status` slot (mono caption on the right) and `detail` line; no new primitive. For the review step of a Bo3/Bo5: `status` = "1 of 3 reviewed"; `detail` = "G1 done · G2 to do · G3 to do" (words, see hard constraints). 390: detail wraps under the title, `text-caption`, never truncated. Step state follows existing `PrepState` (Next until all games the match played have a review; games not yet played are `text-muted` "not played").
- A11y: the detail is plain text; the step keeps `PrepState::label()`.
- Do NOT: add a progress bar or pips (PrepSequence is "no progress art"); invent G-results before games exist; use W/L glyphs here.

## Card 2d. "Earlier work vs Rooks (n)" line
- A single ruled row in the opponent-brief region: left `text-label text-secondary` "Earlier work vs Riverside Rooks (3)", right a `Button` Quiet `ButtonSize::Touch` "Use for this match" (never chalk). Count n = team-scoped drafts+plans+reviews against that opponent, with the breakdown in muted caption "2 drafts, 1 plan". "Use for this match" attaches (links) the chosen items; with n > 1 it opens a `Dialog` Md listing items as checkbox rows (`Checkbox`, 44px) with Save/Cancel `ButtonSize::Touch`; with n = 1 it acts directly and toasts with Undo (reversible: per 6.6 Undo before confirmation).
- n = 0: render nothing (no empty box; the prep steps already say drafts are to do).
- Error: toast "Could not attach: {reason}" with the existing retry pattern.
- Do NOT: auto-attach; show another team's or another opponent's work; call it "history" (plain words).

---
## Card 3. RSVP chips + attendance readout

**Composes:** `SegmentedControl` (3 options Yes / Maybe / No, `manual` = true because selection writes to the server; arrows move focus, Enter/Space/click select), `SegmentSize` touch-height on phones (min 44px each, `min-w-0 flex-1` so the three share the width at 390), `StatReadout`-style text for the count, `Badge` not used for the chips.
**Placement:** in the current-match header block, under the meta line, `mt-4`; label above `eyebrow text-muted` "Will you play?" Only for the signed-in member, only on the current (opened) match, only while the match is in the future (after start: read-only readout, no chips).
**Selected state:** the primitive's own selected look (ink-3 + outline + semibold); do not tint Yes green / No red (tint would imply win/loss and fails "not colour alone"); the word is the signal. Optimistic update with the SaveStatus-style quiet "Saved" in meta; failed write: revert chip, `text-danger` "Could not save your answer: {reason}" + Retry (44px), one toast on the first failure only (6.6).
**Readout:** `text-label text-primary` "4 of 5 confirmed" (Yes only), under it `<dl>` of three ruled rows (`border-t border-rule py-2`): `dt` "Yes" / "Maybe" / "No" `eyebrow`, `dd` names `text-body text-secondary`, comma-separated, wrapping; own name in `text-primary font-semibold`. "No answer yet (n)" row only for roster members who have not answered, names `text-muted`. Collapsed on 390 behind a quiet "Who is coming" `Button` (`aria-expanded`), expanded at `lg`+. Names are team members only, for that match only; never shown on /s/:token.
**States:** no roster/starting five: "n confirmed", no "of 5". No answer yet by the viewer: no chip pre-selected; helper `text-caption text-muted` "Your team sees your answer." Coach/non-player viewer: chips shown only if on the roster; otherwise readout only. Loading: `Skeleton` h-11 for the chips. Error loading: `LoadFailed`.
**A11y:** SegmentedControl group `aria-label="Your answer for Sat 17 Oct"`; result announced via the existing `role="status"` save line (polite). Guidance: do not add a FirstUseHint on this surface unless the M9 hint table gets a key.
**Do NOT:** use emoji/thumbs, coloured chips, `rounded-full` pills, a modal; show other members' answers for other matches; let one member change another's answer; auto-save "Yes" on page load.

---
## Card 4. Public brief share page (/s/:token, logged out)

**Pattern:** reading-width page per 6.9 Sharing; `PageHeader` (title "Match brief", `document_title` generic "Match brief", never the opponent or team names in the tab title), caption "Read-only. Shared with a link that expires ..." (existing), no hub navigation, no RSVP, no names of players, no ids.
**Body, in order, ruled sections (`SectionHeader` + `border-t border-rule`, `gap-6` between groups, comfortable density 24-32px):**
1. Match facts: `<dl>` grid `grid-cols-[7rem_1fr] gap-y-2`: When (`LocalTime`, UTC mark server-side), Kind (`Badge` Neutral), Format (`Badge` Neutral), Opponent (team name only, `text-body text-primary`; no roster, notes, players or dossier text).
2. Our roles: five rows `RoleLabel` (TOP JGL MID BOT SUP, typographic) + role one-liner `text-body text-secondary`; roles only, no player names (honours "nothing else of the team"). One-liner is the plan's per-role line; missing = "Not written yet" `text-muted` (never invented).
3. Win conditions: linked plan's win conditions as a plain list in `text-body`, `max-w-prose`; team notes in `note` serif italic only for quoted annotations (as DESIGN allows), otherwise not used.
**1280:** same single column, `max-w-3xl mx-auto`; facts and roles may sit in two columns from `lg` (`lg:grid-cols-2`) only if each stays readable; default stays single.
**States:** no linked plan: win conditions section replaced by plain line "No game plan is linked to this match yet." (no empty box); invalid/expired/revoked: existing "This link does not open anything" (identical for all three). Rate-limited: existing message.
**A11y:** one `h1`, section `h2`s; `dl` for facts; roles as list; the page has `lang` and no autofocus; passes at 390 with no sideways scroll.
**Security (hard):** public page reads only through the team-scoped DB fns with the team from the link row; whitelist the fields in the shared model (struct with exactly: start, kind, format, opponent_name, roles[], win_conditions[], role_lines[]); a test asserts the serialised model contains no other keys (no user ids, emails, Riot ids, RSVP names, notes, dossier).
**Do NOT:** show RSVP, roster names, scouting notes, patch/ledger data, or the team name; add a hero, share buttons, CTA to sign up, or navigation; use the chalk accent.

## Cross-cutting checks for the builder's render (what a fresh-context critic will capture)
390x852 and 1280x800, seeded data (the seed scrim is at 18:00 tomorrow), reduced motion on (nothing here animates; if any transition is added it must be <= `--dur-2`, ease-out). Measure tap targets from the DOM (>= 44px at 390), contrast of `text-muted` captions on `bg-ink-1/2`, and run `./scripts/lint_design.sh`. Craft question to answer for each: does the list read as ruled typography rather than a widget kit, and is the chalk button still the only loud thing on Match Day?
