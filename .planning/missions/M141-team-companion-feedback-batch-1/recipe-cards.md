# M141 recipe cards (group 0)

App tokens only (`input.css` @theme). Classes named are existing utilities: surfaces `bg-page bg-ink-1 bg-ink-2 bg-ink-3
bg-canvas`, lines `border-rule border-outline`, text `text-primary text-secondary text-muted`, `bg-accent text-accent-ink
bg-accent-soft`, `text-side-blue text-side-red`, status `text-success text-warning text-danger text-info`, radius
`rounded-ctl` (3 px), type `text-caption(12) text-label(13) text-body(14) text-heading`, `figures`, `eyebrow`, `note`,
motion `--dur-1..4` + `ease-out`. No gradients, blur, emoji, `rounded-full` outside primitives, `animate-pulse`,
`text-muted/60`. Frames required for each card: 390x844 and 1280x800, plus reduced-motion where animation exists.
Every card says "reviewer sees" in its acceptance line. Rubric ids in brackets.

Gating decisions to record in REPORT, not to re-litigate: R-A5 (no real Recall event exists in Riot data) and R-A6
(map exists only as fixtures; no event positions).

---
## Group A: Match detail

### R-A1 Scoreboards: one fixed grid, no scroll, blue = red [A1, V2]
Goal: both team tables identical in width and height, never scrollable, no scrollbar on hover.
- Layout: replace `grid gap-x-10 gap-y-10 xl:grid-cols-2` (match_detail.rs:661) with `flex flex-col gap-8`: blue
  above red, each full container width. Same role order in both (sort by `Role::ALL`, unknown role last) so the same
  role lines up row for row.
- `Table` gets `fit: bool` (new optional prop): wrapper `relative overflow-visible` (no `overflow-x-auto`, no
  `tabindex`, no `role=region`), table `w-full table-fixed`. Only `fit` tables; the default stays scrollable for wide
  data tables elsewhere.
- Columns via `<colgroup>`: Pos `w-14`, Player `w-auto` (min-w-0, `truncate`), K/D/A `w-24`, Items `w-44`
  (7 slots at `size-6` + `gap-1`: 7x24+6x4=192 px -> use `w-48`), Damage `w-20`, Gold `w-20`, Vision `w-16`.
- Fixed row height: `Td` content `h-10` (compact density, avatar `AvatarSize::Sm` 24 px). Five rows per team, so both
  tables are exactly header + 5 x 40 px. Empty slot rows (fewer than five participants) render a blank row of the same
  height so the tables stay identical.
- Phone (390): `<md` hide Items, Damage, Gold, Vision (`hidden md:table-cell` on th and td, same classes both). Left:
  Pos 3.5rem, Player flexible with truncate, KDA 6rem (`whitespace-nowrap`). Damage/Gold/Vision/Items stay
  reachable: a `details` "More stats" is NOT used (second ladder); instead the match page keeps `PerformanceSection`
  below. No horizontal scroll at 390.
- Desktop (1280): all seven columns, table width = container (about 1216 px).
- States: loading = existing `Skeleton Block h-48` -> make the skeleton `h-[17rem]` (header 40 + 5x40+borders) so
  there is no layout jump.
- A11y: real `<table>`, caption stays `sr-only`. No scroll region means no extra tab stop.
- Acceptance (frame): at 390 and 1280 and 1536, two tables of the same pixel box, no scrollbar visible while the
  pointer is over a row, last column right edges flush.

### R-A2 Row hover highlights the same role in both teams [A4, A1]
- State: `hover_role: RwSignal<Option<&'static str>>` and `pinned_role: RwSignal<Option<&'static str>>` in `MatchBody`;
  active = pinned, else hover. Passed to both `TeamScoreboard`.
- `Tr` gets `highlight: Signal<bool>` (optional): when true `bg-ink-3`, else `hover:bg-ink-2/70` as today; transition
  `transition-colors` (already `--dur-2`). Remove `selected` (`bg-ink-2`) for these tables.
- Pointer: `on:mouseenter`/`on:mouseleave` on `<tr>` set `hover_role`. Also `on:focusin`/`focusout`.
- Tap equivalent: the Pos cell holds a `Button variant=Quiet size=ButtonSize::Touch pressed=...` wrapping `RoleLabel`;
  click toggles `pinned_role` (second click or Esc clears). One tab stop per row, 44 px on phone (`Touch`). `aria-label`
  "Highlight {role} in both teams".
- Highlight is a background tint (`bg-ink-3`). A pinned row also gets `border-l-2 border-accent` on its first cell so the
  pin is not colour-only. Hover is tint only. No shadows.
- Reduced motion: transition becomes instant (global rule); nothing animates otherwise.
- Acceptance: pointer on the Mid row of blue shows the Mid row of red tinted in the same frame; at 390 a tap shows
  both rows tinted and the pin edge.

### R-A3 "(you)" as chalk accent; KDA never wraps [A2, V1]
- Remove `{is_user.then_some(" (you)")}` (match_detail.rs:273). On the player's `ChampionAvatar` wrapper add
  `outline outline-2 outline-offset-1 outline-accent` (same chalk used by the timeline, `stroke-accent`). Only the
  avatar, not the row.
- Screen readers: `<span class="sr-only">", you"</span>` after the name; row `aria-current="true"`.
- Because the icon can be dim next to ink-1, verify accent vs `bg-page`: accent is oklch(0.91 0.10 108) on
  oklch(0.18..), well above 3:1 (non-text).
- KDA: helper `kda_text(k,d,a) -> String` using U+00A0 around the slashes; `Td` class `whitespace-nowrap`
  (add `nowrap: bool` prop, or always for `numeric` cells: numbers never wrap; recommended: add `whitespace-nowrap` to
  the `numeric` branch in `Td`). PageHeader meta KDA span (L652) gets `whitespace-nowrap`.
- Acceptance: the 390 frame shows the K/D/A in one line for a double-digit game (e.g. `12 / 10 / 24`).

### R-A4 Timeline as three stacked rails with toggles [A1, A4, I7, V2]
- Groups (const `RAILS`): `Objectives` {Objective, Tower}, `Fights` {Kill, Teamfight}, `Back & vision` {Recall (see
  R-A5), Ward}. Per-kind toggle state stays (`shown[6]`), group header toggles all kinds in the group.
- Structure per rail (a `<div role="group" aria-label="{group}">`):
  `grid grid-cols-1 gap-2 md:grid-cols-[11rem_minmax(0,1fr)] md:items-center`.
  Left: `eyebrow` group name + the kind toggles as `Button size=Sm variant=Secondary pressed` with the existing marker svg
  (kept from L370-376). Right: track `relative h-10 border border-rule bg-ink-1` (phone `h-11`), hairline
  `left-3 right-3 top-1/2 h-px bg-rule`, markers as today (`marker_shape`, side colour, chalk stroke when `mine`).
- Collision handling inside a rail: sort by time; a marker within 1.2% of the previous one shifts to a second row
  (`top-1/4` / `top-3/4`) instead of overlapping; track grows to `h-14` only when needed. Never hide events.
- Ruler above the first rail: `figures text-caption text-muted` labels every 5 min (phone every 10), `border-t border-rule`
  ticks. Shared across rails.
- Hit targets: markers `p-1.5` on desktop (24 px), `p-3` on phone (44 px effective with the rail at `h-11`); where
  dense, the detail strip offers `Previous event` / `Next event` `Button size=Touch` that step through visible events
  (A1/A4 equivalent to tapping a crowded marker). Selected marker `aria-pressed` as today; detail panel unchanged.
- Empty rail: centered `text-label text-muted` "None of these in this game" inside the track (not the whole timeline).
- Wards caption: rail caption `text-caption text-muted` "Wards: yours only".
- Legend (L420-433) shrinks to the side markers + chalk; kind shapes are on their toggles now.
- Acceptance: 1280 frame shows 3 rails, no overlapping markers, each rail's toggle off hides only its kind; 390 frame
  shows each rail full width under its toggles, 44 px marker targets, prev/next buttons.

### R-A5 Recall vs Item_undo [data honesty]
- Fact: `ITEM_UNDO` is the shop undo, not a recall. Riot Match-V5 timeline has no B-press event.
- Change: (1) `classify_event` (riot.rs:684) returns `None` for `ITEM_UNDO` (not an event worth a marker, matches
  sand_table.rs:188 "deliberately not imported"); (2) `db.rs:6054` read path ignores `event_type == "ITEM_UNDO"` so
  already-stored matches stop showing the wrong rows without re-import; (3) fix the stale comment; unit tests flip
  (riot.rs:926 asserts the old mapping).
- Inferred recall (optional, only if cheap): cluster `ITEM_PURCHASED` events of one participant within 10 s ->
  one `Recall` event flagged `inferred`. Needs the importer to keep `ITEM_PURCHASED` (currently dropped by
  `classify_event` `_ => None`), so existing rows have none until re-fetched. Draw inferred markers as the existing
  recall circle with `stroke-dasharray="3 2"` (dashed = inferred, mirrors `RoleLabel guessed`), label "Back
  (inferred from purchases)". If not built this batch: leave the `Recall` category in the enum, hide its toggle when
  the match has zero Recall events, and say why in the rail caption.
- Acceptance: a match that previously showed N "Recall" circles shows none unless inferred ones exist; no Recall
  tooltip names an item-undo.

### R-A6 Stretch: shared playhead + map [A3, A4, I1]
- Reuse: `TimelineScrub` (ui/timeline.rs:88; `value: RwSignal<u32>`, `moments: Vec<Moment>`) and `CanvasFrame`
  (ui/canvas.rs:283, `FrameGrid::Numeric {max:1000, step:250}`, `aspect-square`, `texture=true`). These are the "rift
  map" parts; nothing else exists.
- Smallest version: (1) `TimelineScrub` placed directly under the rail ruler; its `value` drives a 1 px `bg-accent`
  playhead line over all three rails (`absolute inset-y-0 w-px bg-accent`, `left: pct%`), `moments` = the visible
  events (label from `event_tooltip`). No play button (DESIGN: none until playback exists). (2) beside/below on
  `lg:` (`lg:grid-cols-[minmax(0,1fr)_22rem]`), a `CanvasFrame` with ten tokens from the replay's per-minute frame at
  `value/60`: blue tokens round/side-blue, red square/side-red, "you" token chalk-outlined. (3) If no replay exists
  for this match: `EmptyState Compact` "Positions need the match's replay import. Import it on the Sand Table page"
  with the manual path (rails still work). I1: the rails never depend on the map.
- Never place events on the map: `TimelineEvent` has no x/y.
- Phone: map below the scrubber, `aspect-square w-full`, scrubber one tab stop with J/L jumps; tap on the ruler seeks.
- Reduced motion: no playback, so nothing moves except the jump; transition none.
- Acceptance: frame at two scrub values (e.g. 8:00 and 17:05) shows the playhead and tokens moved; empty-state frame
  for a match without replay.

---
## Group B: Game plan

### R-B1 Icons beside names once picked [A2, V2]
- `ChampionAutocomplete` (components/champion_autocomplete.rs): resolve `value` against `champions`; when it matches
  exactly (id or name, case-insensitive) render `ChampionAvatar size=AvatarSize::Sm decorative=true` in a fixed 24 px
  slot at the left of the input (`pl-10` on the input, slot `absolute left-2 top-1/2 -translate-y-1/2`). No match ->
  slot shows a dashed `size-6 rounded-ctl border border-dashed border-rule` placeholder (same dimension: no shift).
  Name stays in the input (never icon-only).
- `MatchupSection`: both sides use the same row (RoleLabel | icon+input). The read-only branch (L1117) already
  shows icon + name; make both branches the same row geometry (24 px slot) so "Edit picks" does not shift lines.
- Optional pair read: highlight row pairs on hover like R-A2 (not required).
- A11y: icon `decorative`; combobox name unchanged.
- Acceptance: frame of a plan with ten typed (not draft-linked) picks shows ten icons; blank rows show the dashed slot.

---
## Group C: Champion pool and the shared save-state helper

### R-C1 Moves keep scroll and feel instant [A4, A3]
- Optimistic: keep `pool` entries in a local `RwSignal<Vec<ChampionPoolEntry>>` seeded from the resource; `move_to`
  updates the tier at once, then calls `set_champion_tier`; on error revert and set the save state to Failed (R-C2).
- Replace the inner `<Suspense>` (champion_pool.rs:446) with `<Transition>` (or read the signal directly) so a refetch
  never swaps the list for a skeleton. Do the same for the detail panel refetch (L685, L709, L725). No `refetch()`
  needed after a move; refetch silently in the background.
- Preserve scroll: do not re-key the `ol`; after the move keep focus (`focus_id_later`) with
  `focus` option `preventScroll: true`.
- Drag feel: source chip `opacity-60` with `data-dragging`; target tier row already `bg-accent-soft`; add `cursor-grab`
  on chips, `active:cursor-grabbing`. No animation on keyboard-repeated actions (DESIGN 3.7).
- A4: keyboard path (Space / arrows / Space, Esc) already exists; keep the live region.
- Acceptance: scrolled-down frame before and after a drop: same scrollY, same page height, chip in new tier.

### R-C2 `SaveState` + `LoadFailed` helper [I1, A3]
- Module `src/components/ui/status.rs` (A11y line required), exported in `ui/mod.rs`, gallery section + `gallery.spec.ts`.
- `pub enum SaveState { Saved, NotSaved, Unsaved, Saving, Failed(String) }`; labels (DESIGN 6.6): "Not saved yet",
  "Unsaved changes", "Saving...", "Saved", "Could not save".
- `#[component] SaveStatus(#[prop(into)] state: Signal<SaveState>, #[prop(optional)] on_retry: Option<Callback<()>>)`:
  renders `<span role="status" class="text-label text-muted">` (Saved: `text-secondary`; Failed: `text-danger` + a
  `Button size=Sm variant=Quiet` "Retry" if `on_retry`). Text always present (not colour-only). Lives in `PageHeader`
  `meta`. One `role=status` per page.
- Helper: `fn use_save_state() -> (RwSignal<SaveState>, impl Fn(..))` with `run(fut)` that sets Saving -> Saved or
  Failed(error_text(e)); "Saved" reverts to quiet after 4 s via timer only on hydrate; clears pending timer on drop.
- `#[component] LoadFailed(#[prop(into)] what: String, on_retry: Callback<()>)`: `Callout tone=Danger` with title
  "{what} did not load" and a `Button size=Sm` "Try again" (calls `resource.refetch()`). Replaces the three-line
  `Err(_) => Callout` patterns (me.rs:~299, others).
- Reuse: champion pool (R-C1), then M137 R2 items. Does not change layout: header meta slot only.
- Acceptance: pool header shows "Saving..." then "Saved" on a move; with the server action failing (e2e route
  abort) shows "Could not save" + Retry; `LoadFailed` frame in the gallery.

---
## Group D: Keyboard

### R-D1 Enter submits the attached primary action
DESIGN.md rule line (to add in 6.6, "Page patterns"):
> **Enter submits.** A single-line field with an attached primary button runs that button on Enter (a `<form>` whose submit is the button's action, or `on_keydown`); in a textarea Enter stays a newline and Ctrl+Enter submits.
- Implementation: wrap field + button in `<form on:submit=move |ev| { ev.prevent_default(); do_add.run(()) }>`; the button
  becomes `kind=ButtonType::Submit` (Cancel next to it stays type `button`). Do not set `method` or `action`;
  a pre-hydration Enter does nothing, same as a click before hydration (SSR renders the form without an `action`).
- `do_add` takes a `MouseEvent` today (tree_drafter.rs:560); change to `Callback<()>` and have the click path call it.
- Apply: tree_drafter.rs:690-702; opponents.rs:~880 (Riot ID add); draft.rs:~2748/2751 (champion + reason pair:
  Enter in either field submits); team/dashboard.rs:~1158 note (textarea -> Ctrl+Enter, show `Keycap` hint).
  Audit command for the builder: `grep -n "TextField" -r src/pages src/components | ` each result without `on_keydown`
  or an enclosing `<form>` whose neighbour is a `Button` primary/secondary; list in REPORT. Already fine: action_items,
  game_plan checklist, champion autocomplete/picker, `ActionForm` pages.
- Validation: empty label -> existing error toast stays; focus stays in the field. Success -> keep focus in the field
  for repeated adds (keyboard-heavy flow) except the tree drafter, where the form closes (existing behaviour).
- A11y: native form semantics; no `aria-keyshortcuts` needed. Optionally add `shortcut="Enter"` keycap to the button
  on desktop only.
- Acceptance: e2e: type in "New branch", press Enter -> branch appears; frame not required beyond a screenshot of the
  filled field with the button.

---
## Group E: Post-game

### R-E1 "Filled from match" pattern [I4, I1]
- On match select (`ed.match_id` effect, post_game.rs:788): fetch a `get_match_facts(match_id)` (team-scoped) returning
  result for the user's team, side, game duration, linked plan/draft ids if the team has them. Fill only fields that
  are empty: `outcome` (win/loss), `plan_id`, `draft_id`. Never touch went-well/improve/actions/notes/rating
  (humans decide). Never overwrite a non-empty field; a second selection replaces only fields that were still
  auto-filled (track `autofilled: RwSignal<HashSet<&'static str>>`).
- Facts without a field (side, duration, picks/bans) go in a read-only `Surface` "From the match" above the form: one
  row of `StatReadout`s (`Result`, `Side` via `SideMarker`, `Length`), no inputs, nothing invented (EmptyState if the
  match has no stored detail: "Open the match once to store its details").
- Note pattern (reusable): under the `Game` section header, `<p class="text-caption text-muted" role="status">` "Filled
  from match: Result, Game plan. Change any of them." with `Button size=Sm variant=Quiet` "Undo" that restores the
  previous values (I8: undo of an automated action). A field that was auto-filled carries a `Badge` "From match"
  (Neutral) inside its `Field` hint area; editing it removes the badge.
- Phone: Surface stacks `grid-cols-2 gap-3`; the note wraps under the Select. 
- Acceptance: choose a match -> outcome Select set, note visible naming fields; edit outcome -> badge disappears;
  Undo restores blank.

---
## Group F: My Games

### R-F1 Buttons keep one size in every state [A3, V2]
DESIGN.md rule line (to add in 6.1 Button row or 6.6):
> **A button keeps its size.** Width and height never change between idle, loading and done: the spinner overlays the label's box instead of joining the row, and a label that changes uses a reserved width (the longest label).
- Primitive change (button.rs:205-232, button branch only; the link branch has no loading):
  1. Root class adds `relative`.
  2. Wrap `{icon_view}{children()}{keycap}` in `<span class="inline-flex items-center justify-center gap-2" class:opacity-0=loading>`
     (opacity keeps the box and the accessible name; `invisible` would drop the name for screen readers).
  3. Spinner becomes `<span class="absolute inset-0 grid place-items-center" aria-hidden="true"><span class="size-3.5 rounded-full border-2 border-current border-r-transparent animate-spin motion-reduce:animate-none"></span></span>` (the existing spinner class set already satisfies the lint rules; `rounded-full` stays inside the primitive).
  4. Optional `#[prop(optional)] reserve: &'static [&'static str]`: renders the other labels as hidden twins in a grid stack
     (`grid [&>*]:col-start-1 [&>*]:row-start-1`, twins `invisible aria-hidden=true`), so "Sync my games" / "Syncing..."
     / "Synced" share the widest box. Use only where a label really changes.
- Loading still sets `disabled` + `aria-busy` (unchanged); focus ring unchanged.
- Reduced motion: spinner static ring (`motion-reduce:animate-none`) - still readable as a busy mark because the label
  dims to `opacity-0` and the ring sits alone; add `disabled:opacity-50` stays (cursor-progress via `aria-busy`).
- Sweep: me.rs:292, bug_report_widget (Send report), every `loading=` call site; gallery gets a width test row
  (idle vs loading vs done side by side, same box) and a unit/e2e check comparing `boundingBox().width` idle vs loading.
- Acceptance: frame pair of "Sync my games" idle and loading at 390 and 1280: identical rectangle (measure in px).

### R-F2 Sortable table header [A1, A4]
- Component `SortTh` in `ui/table.rs` (next to `Th`): props `label`, `key: &'static str`, `sort: RwSignal<SortState>`
  (`SortState {key: Option<&'static str>, dir: SortDir}`; `SortDir` exists), `numeric: bool`, `default_dir`.
  Renders `<th scope="col" aria-sort=...>` (`ascending` | `descending` | `none` omitted when unsorted) containing a
  `<button type="button" class="eyebrow inline-flex items-center gap-1 min-h-11 md:min-h-8 whitespace-nowrap text-secondary hover:text-primary" aria-label="Sort by {label}">`
  + 10 px SVG chevron (stroke 1.5, `currentColor`): up when ascending, down when descending, up-down pair `text-muted`
  when unsorted. No unicode arrow, no emoji (lint rule `emoji`); chalk `text-accent` is NOT used for sorted
  state (accent means "you"): sorted = `text-primary` + the glyph.
- Click: first click on a column -> `default_dir` (numeric columns descending, text ascending); same column again
  flips. Keyboard: the button is native (Enter/Space). Order is stable: sort by key then original index.
- Helper `pub fn sort_by_key<T>(rows: &mut [T], dir, key: impl Fn(&T)->K)` stable.
- Announce: one `role="status" class="sr-only"` per table "Sorted by {label}, {ascending|descending}".
- Apply in this batch (comparable rows): me.rs Champion trends (L549, default Games desc), Recent games (L382, default
  by date; sorting by Result/KDA/CS/Damage), stats.rs "Games by member" (L401), game_plan.rs "Win rate by strategy"
  (L1529), draft.rs "Champions by appearances" (L2839), team/roster.rs members, admin invites (L227), bugs board
  (community/bugs.rs:157). Not sorted: LP points (chronological), caption/summary tables, admin hub.
- Phone: header button `min-h-11`; tables stay inside their wrapper `overflow-x-auto` (data tables, not R-A1).
- Acceptance: frame with Win rate sorted descending (glyph + order), then ascending; keyboard Tab to header + Enter.

### R-F3 LP history states its source [trust, honest emptiness]
- Under the figure (replace the first/last date `figcaption`, me.rs:500-504) add:
  `<p class="mt-2 text-caption text-muted">` "Source: one point each time you press Sync my games (Solo/Duo ranked standing). {n} points, last sync {LocalTime}. Riot's game data carries no LP, so nothing is drawn before your first sync." Keep the first/last date line above it.
- x-axis: points are placed by time (lp_points); draw nothing between syncs (no interpolation claim): the line style
  becomes `stroke-dasharray` only for gaps > 7 days (optional). Marks are the existing circles.
- Zero snapshots: `EmptyState Compact` "No ranked standing stored yet. Press Sync my games; each sync adds one point."
  One snapshot: draw the single circle and its readout ("Gold II, 54 LP, 16 Oct"), caption says "A second sync draws the line."
- The window control (L470) stays; when the selected window holds fewer points than "All", mention it in the caption.
- No mini sync-cadence promise beyond the page's existing sentence about minutes.
- Acceptance: three frames (0, 1, many snapshots) at 1280; caption wraps cleanly at 390 without cutting.

---
## Group G: Admin invites

### R-G1 Fields on one baseline, hints below [V2]
- Replace `flex flex-wrap items-end gap-3` (invites.rs:141) with
  `grid grid-cols-1 items-start gap-x-3 gap-y-4 sm:grid-cols-2 lg:grid-cols-[8rem_10rem_minmax(12rem,1fr)_minmax(12rem,1.2fr)_auto]`.
  Order: Max uses, Expires in days (hint "Empty: never"), Team (hint "Joins the account to this team"), Note, button.
- `items-start` puts every label on the same line and every control on the same line (labels are one line of
  `text-label`), hints hang below the controls (`Field` already puts them last).
- Button column: `<div class="flex flex-col gap-1.5"><span class="text-label" aria-hidden="true">&nbsp;</span><Button kind=Submit variant=Primary size=ButtonSize::Md>...` so the button sits on the control line. Phone: button `full_width`, `Touch` size.
- Verify `TextField` (h-9) and `Select` (h-9) are the same height in the frame; if not, report as a primitive bug.
- Acceptance: 1261 and 1280 frames: horizontal line through the four controls, hints under their own fields only.
  Phone: single column, hint below each field.

---
## Group H: Feedback loop

### R-H1 "Give Feedback" and bug / idea / praise labels [A1, V2]
- Trigger: label "Give Feedback" exactly as the owner wrote it (the mission names it so; the product's other buttons are
  sentence case, so if the builder prefers "Give feedback" record the choice in REPORT). `aria_label="Give Feedback"`; size `ButtonSize::Touch` (44 on phone, 32 from md). Keep dock position.
- Dialog title "Give Feedback". `SegmentedControl` options: Bug, Idea, Praise (3 segments; labels only). Mapping at
  submit: bug -> `bug`; idea -> `wishlist`; praise -> `wishlist` + body prefix "[praise] ". Server enum untouched
  (bug_report_widget.rs:72). Placeholders per kind: bug "What did you do, what did you expect, what happened instead?";
  idea "What would help your team, and when would you use it?"; praise "What worked well for you?".
- Rename every user-visible and test string: widget header L3 comment, `community/bugs.rs:199`,
  `community/what_next.rs:59`, admin pages, CHANGELOG, docs; e2e `bug-report.spec.ts:9`, `bug-triage.spec.ts:16`,
  `frames-m141.spec.ts:96` use the new aria-label. The select-mode hint copy stays ("Click the part of the page your
  report is about").
- Admin: category column shows Bug / Idea / Praise by reading the body prefix; no migration.
- Acceptance: widget frame at 390 and 1280 with the three-segment control; button measured >= 44 px high at 390.

### R-H2 Attachment: file/paste floor, region picker enhancement [I1, A4]
Layout inside the existing `Dialog` (Md), below the text field:
- `Field id="bug-report-attach" label="Screenshot (optional)" hint="PNG, JPG or WebP, up to 5 MB. We convert it to WebP. It can show other people's names: leave out what you do not want to share."`
  Contents (`flex flex-col gap-2`):
  1. Row: `Button variant=Secondary size=Md` "Attach an image" (opens a hidden `<input type=file accept="image/png,image/jpeg,image/webp">`)
     and, from `md` only and only if `getDisplayMedia` exists (feature-detect, hide otherwise): `Button variant=Secondary` "Capture an area".
  2. Paste: the text field and dialog body listen for `paste` with an image `ClipboardItem`; a `text-caption text-muted`
     line "or paste an image (Ctrl+V)".
  3. Attached state: a `Surface` row `flex items-center gap-3 border border-rule p-2`: thumbnail 64x40 `object-cover rounded-ctl`
     (`<img alt="Screenshot preview">`), file name + size `figures text-caption`, `Button size=Sm variant=Quiet` "Remove".
     One attachment only in R1.
- Capture flow (enhancement): click "Capture an area" -> `dialog_open.set(false)` (hide via `hidden`, keep the form
  state; do NOT unmount) -> full-viewport overlay `fixed inset-0 z-(--z-toast) bg-scrim` with the drag rectangle drawn as
  `border-2 border-accent bg-transparent` (cut-out) and a `Callout`-like bar "Drag over the area. Esc to cancel" (reuses
  the select-mode hint style, `bg-ink-1 shadow-overlay`) -> on release, capture (display media stream frame, cropped
  via canvas to WebP) -> dialog returns with the attachment. Esc or failure returns with an inline `Callout Warning`
  "Capture did not work. Attach an image file instead." (I1: the file path is always visible).
- Overlay is outside the app's `Dialog` scrim; the Dialog hides before the overlay mounts so the grab does not contain it.
- Phone: no "Capture an area" (no `getDisplayMedia`); file picker opens the camera roll; the paste line is hidden (`hidden md:block`).
- A3: no animation on the overlay (instant); focus returns to the "Attach an image" button after capture/cancel.
- Server (builder note, not UI): size/type check, strip metadata, admin-only read; as in the mission H2.
- Acceptance: frames: form with no attachment, with attachment thumb, capture overlay mid-drag (1280), failure callout.

### R-H3 Reporter's status timeline [A1, V3]
- On `/bugs?tab=mine` replace the 5-column table with a list of `Surface` cards on phone and a table row + expandable
  detail on desktop? Keep one structure: each report is a row `border-b border-rule py-4` with:
  title line (page code + date), the report text (clamped as today, "Show all"), then the timeline strip.
- Timeline strip: `ol` of the five states `New -> Triaged -> Planned -> Fixed / Won't fix`; `flex flex-wrap gap-x-4 gap-y-1 text-caption`.
  Reached steps: `text-primary` with a 6 px square marker `bg-primary` (survey tick shape); current: marker `bg-accent`
  (accent = yours) plus the word in `font-semibold`; future: `text-muted` hollow marker `border border-outline`.
  Never colour-only: the current step also carries `aria-current="step"` and a visually hidden "(current)". Each reached
  step shows its date `figures` when known (`status_changed_at` for the current one only; others omitted).
- Under the strip, the team note in `note` style (serif italic) with `border-l border-rule pl-3` as "Note from the team";
  if fixed: the CHANGELOG link `text-primary underline decoration-rule underline-offset-4` "Fixed in {title}".
- Won't-fix replaces the last two steps with one step in `text-secondary`.
- Phone 390: strip wraps to two lines, no table; the "Updated" `Badge Accent` stays at the title line.
- Acceptance: frames of four reports in four states; the current step identifiable without colour.

### R-H4 Import proposals (JSON), draft-only [I3, I4, I8]
- `/admin/proposals`: header action `Button variant=Secondary` "Import proposals (JSON)" opens a `Dialog` Md: a `TextField rows=10`
  (monospace via `font-mono` class on the control) or a file input (`.json`), `Field` hint "Imported proposals are saved as drafts, invisible to members until you publish each one."
  Footer: `Button Quiet` Cancel, `Button Primary` "Import as drafts" (disabled until the JSON parses; show
  `{n} proposals found` in `text-label`).
- Result: `Callout tone=Success` "{n} drafts imported, proposed by the Team Companion agent on {date}." plus per-item
  errors in `Callout tone=Danger` (one list, no partial silent skips). Never auto-publishes, never votes.
- List rows (L113-123) get a `Badge Neutral` "Draft" and the line "Proposed by the Team Companion agent - {date}" in
  `text-caption text-muted`; the existing "Not published" badge stays. The editor keeps Publish as the one primary action.
- Admin only: the dialog trigger renders for admins (page is already 404 for others); server fn uses `authz::require_admin()`.
- Acceptance: frames: import dialog (valid JSON), result callout, list with two draft rows.
