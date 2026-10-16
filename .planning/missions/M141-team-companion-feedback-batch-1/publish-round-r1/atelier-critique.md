# M141 group 0: atelier critique (source-only, no render)

Mode: critique from source and report text. No server was available, so no frame, strip or pixel contrast exists yet.
Every verdict below is a source verdict (FIX / REJECT) and the acceptance checks in `recipe-cards.md` say what the
after-frames at 390 and 1280 must show. Design source: `input.css` @theme, `docs/DESIGN.md`, `src/components/ui/*`.
Rubric ids: A1 target size, A2 contrast from pixels, A3 reduced motion, A4 tap/screen-reader equivalent of drag/hover,
I1 manual path survives failure, I4 correction path, I7 primary content readable with secondary controls collapsed,
V1 closed tokens, V2 4/8 spacing, V3 measure.

Craft question asked of each surface first (could a design director ship it?): none of the eight ships as is. The
common cause is not styling, it is that layout is decided by content (auto table widths, inline spinners, `items-end`
rows), so state changes shift geometry. Fix geometry first, then everything else.

## 1. Match detail `src/pages/match_detail.rs`

- L250 + `src/components/ui/table.rs:46-48`, FIX (high, A1/V2): every `Table` is wrapped in `relative overflow-x-auto`
  with `tabindex=0`. The seven-column scoreboard (Pos, Player, KDA, 6-7 item icons, Damage, Gold, Vision) needs about
  730 px; at `xl:grid-cols-2` (L661) inside the 80rem `PageWidth::Wide` page each half is about 588 px at any viewport
  above 1280 (the page caps at 80rem, so 1536 and 2536 do not help). So both tables overflow, the browser shows the
  overlay scrollbar on hover (the owner's report), and blue and red can differ because widths are content-sized.
  Verdict: REJECT the two-column layout; stack blue over red with one shared fixed column grid (R-A1).
- L273: `(you)` text inside the name span, FIX (med): the same app already says "chalk = you" (timeline legend L433
  "Chalk outline: you were involved", `stroke-accent` L390). Reuse that: a chalk outline on the player's champion
  avatar plus a visually hidden "you" so the name stays accessible (R-A3). `Tr selected` also fills `bg-ink-2`
  (table.rs:133-139), which will collide with the hover highlight; drop the fill for this table.
- L269 `Td numeric` renders `"{} / {} / {}"` with ordinary spaces; `Td` has no `whitespace-nowrap`; `Th` has it, `Td`
  does not. KDA wraps, FIX (high): non-breaking spaces and `whitespace-nowrap` on the cell (R-A3). The header meta KDA
  (L652) is a flex child and can wrap the same way.
- No cross-team relation exists: rows carry no role key. Hover highlight needs a shared `hover_role` signal in
  `MatchBody` (L586) and a tap/focus equivalent (A4: hover-only is not available on phones) (R-A2).
- L335-448 Timeline: one 48 px track holds six categories, up to ~100 markers at `size-3` with `p-0.5` (about 16 px
  hit targets): clustered, below A1 on phone, FIX (high). Six equal toggles above it do not express grouping.
  Verdict: REJECT the single rail, rebuild as three stacked rails from the same data (R-A4). Markers keep the
  existing shape language (`marker_shape`, one shape per category, side in colour), which is good and stays.
- `src/server/riot.rs:684` and `src/server/db.rs:6054` map `ITEM_UNDO` to `EventCategory::Recall`. This is a
  classification error (an item undo is the shop "undo purchase" button) and Riot's Match-V5 timeline has no event
  for pressing B at all. The same app's own importer (`src/server/sand_table.rs:188`) already lists `ITEM_UNDO` under
  "deliberately not imported". So the fix is not a remap, it is: stop emitting Recall from `ITEM_UNDO`; a real Recall
  can only be inferred (R-A5). Existing stored matches carry the wrong rows, so the fix must also filter on read
  (`event_type == "ITEM_UNDO"`) or the old data keeps lying. REJECT of the current primitive mapping.
- Wards are the user's own only (`match_data.rs:~56` comment), which the legend does not say; with a "Recalls +
  wards" rail this reads as all wards. FIX (low): rail caption "your wards".
- Map and scrubber ("we have a rift map, reuse it"): there is no live map component. What exists are two reserved
  gallery fixtures: `CanvasFrame` (+ `ScaleBar`, `Annotation`) in `src/components/ui/canvas.rs` (fixture at
  `src/pages/design_domain.rs:~292`, numeric 0-1000 frame, `aspect-square`) and `TimelineScrub`
  (`src/components/ui/timeline.rs:88`, slider, J/L moment jump; fixture at `design_domain.rs:~321`). The Sand
  Table (positions from replay frames) is storage only: `models/sand_table.rs`, `server/sand_table.rs`,
  `db.rs:6298+ store_replay`; there is no page. `TimelineEvent` (`match_data.rs:63-77`) has NO x/y, so markers
  cannot be placed on a map from the match's own events. Smallest honest version in R-A6: shared playhead across
  the rails (needs nothing new) and, only when a replay exists for the match, ten tokens from the per-minute frame at
  the playhead minute on a `CanvasFrame`; otherwise an `EmptyState`. Never invent positions.
- Severity summary: A1 high, A2 high (row hover), A3 high (KDA), A4 high, A5 high (wrong data), A6 stretch.

## 2. Game plan draft table `src/pages/game_plan.rs`

- L1088-1155 `MatchupSection`: the read-only branch (picks that came from a draft, L1117-1128) already shows
  `ChampionAvatar` + name. The editable branch (L1130-1134, `ChampionAutocomplete value=sig`) shows a bare text
  input, so by hand or after "Edit picks" there is no icon. The owner sees this on picks that are not from a
  draft. FIX (med, V2): leading icon slot in `ChampionAutocomplete` (`src/components/champion_autocomplete.rs`; the
  avatar is only in the dropdown rows, L206). Icon beside the name, never instead (the mission says so). R-B1.
- Unresolved or free-typed names must not render a broken avatar: show nothing until the value matches a champion.

## 3. Champion pool `src/pages/champion_pool.rs`

- Page does not reload on drop in source (no form, `ev.prevent_default()` L478). What jumps the scroll is
  `pool.refetch()` (L393, L413, L685, L709, L725): the inner `<Suspense>` at L446 re-shows its fallback
  skeleton for every refetch, the five-tier list is replaced by 5 skeleton lines, the document shrinks, the browser
  clamps scrollTop, and then the list re-renders at the top. FIX (high): optimistic local update of the entry's tier
  and `Transition` (keeps the old DOM while refetching) instead of Suspense; no layout change on move. R-C1.
- Keyboard move: after the drop `focus_id_later` (L616) refocuses, but `scroll` is lost the same way. Same fix.
- A4 holds already (Space/Arrows keyboard move + announce). Keep the live region (L306). Missing: failure path
  shows a toast only; no persistent "not saved" (I1). R-C2 gives the save state in `PageHeader` meta (DESIGN 6.6).
- Drag affordance: `draggable="true"` on the whole `li`, with a `Button` inside (L531): fine; add `cursor-grab` on
  the li and `data-dragging` dimming of the source chip (no animation).

## 4. Tree drafter "add branch" `src/pages/tree_drafter.rs:690-702`

- `TextField value=branch_label` is not in a form and has no `on_keydown`; the primary `Button ... on_click=do_add`
  is click-only. Enter does nothing. FIX (high for keyboard-heavy product). R-D1. The same shape exists at
  `src/pages/opponents.rs:880` (add by Riot ID), `src/pages/draft.rs:2748/2751` (champion + reason pair),
  `src/pages/team/dashboard.rs:~1158` (textarea: Ctrl+Enter). Already correct: `action_items.rs:746-752`,
  `game_plan.rs:1404`, `champion_autocomplete.rs`, `champion_picker.rs`. `admin/invites.rs` uses `ActionForm`
  (native Enter). Audit the rest with the grep in the card.

## 5. Post-game form `src/pages/post_game.rs`

- L788: the Match `Select` only sets `ed.match_id`. Nothing derives from it. The only fields a match can answer on
  this form are `outcome` (win/loss), the linked game plan and draft (if the team has one for that match) and, as
  text, nothing for rating or the written lessons. The report says "all fields"; honesty rule (CLAUDE.md): only fill
  what the data answers; rating, went-well, improve and actions stay human. FIX (med). R-E1: fill empty fields only,
  never overwrite typed text, show a persistent "Filled from match" line with an Undo, and say which fields.
- `PlayerMatchStats`/`MatchParticipant` carry result and side; draft picks and bans are not on the review model
  today, so "picks, bans, side" in the mission table need either a read-only "From the match" summary block
  (Surface with the facts, not editable form fields) or are out of scope. The card chooses the summary block.

## 6. My Games `src/pages/me.rs` and `src/components/ui/button.rs`

- Button width jump: `button.rs:212-217` renders the spinner as a first inline-flex child (`size-3.5` + `gap-2` =
  about 22 px) only while loading, so the button grows when pressed. FIX (high, owner's wish + mission F2): overlay
  the spinner, keep the label in flow (R-F1). It is a primitive bug, so every `loading` Button in the app has it:
  `me.rs:292`, `bug_report_widget.rs` "Send report", every `loading=` site. `Button` doc says "the spinner is
  decorative and the label stays" - true, but it also moves.
- Tables: eleven pages use `Table` (list in R-F2). `Th` supports `aria-sort` display (`table.rs:~112`) but nothing
  sorts and there is no header button, so headers are not operable. REJECT as is; add a `SortTh` (R-F2).
  The indicator glyph must be an SVG chevron: `scripts/lint_design.sh` emoji rule (line 49) allows only
  U+2605/2606/2713/2715/2717 from the Dingbats/Misc-Symbols range; an arrow in U+2191/2193 or a geometric triangle
  U+25B2 would pass that regex but renders from a fallback font; SVG avoids both.
- LP history L460-526: the chart has no source label (L507-518 only labels the first and last date), the empty
  copy at L476 ("Each sync stores your Solo/Duo standing...") is the only place the source is said, and once a line
  shows it is silent. Data confirm the owner's suspicion: `db.rs:5528 store_ranked_snapshot` writes one row per
  sync; `get_lp_history` (db.rs:5664) reads them. The Oct 13-16 span is the span of syncs, not of games. FIX
  (med, trust): R-F3 caption stating source, count, last sync, and that older games carry no LP. Zero snapshots
  shows an honest empty state; one snapshot shows the dot and its value (today <2 hides it).
- Segmented period control (L470) is a good pattern; keep. The 600x160 SVG scales with `h-auto w-full`; on 390
  the 4 px markers and 11 px (viewBox scale ~0.62) text render under 12 px - there is no text in the SVG, so OK.

## 7. Admin invites `src/pages/admin/invites.rs:140-170`

- Row is `flex flex-wrap items-end`. Fields with a hint ("Expires in days", "Team (optional)") are about 20 px taller
  than fields without, and `items-end` pins the bottoms, so the controls of hinted and unhinted fields sit at
  different heights. Exactly the owner's report. FIX (med, V2). R-G1: `items-start` grid, one label line height
  for all, hints hang below, button aligned by a label-height spacer. Verify equal control heights (TextField `h-9`
  vs Select) in the frame: if Select differs, that is a primitive bug to report, not to patch with margins.

## 8. Feedback widget `src/components/bug_report_widget.rs` and the /bugs and admin pages

- L396-400: the trigger is a docked `Sm` button (28 px) in the page flow above the footer. On phone it is under
  A1's 44 px. Name "Report a bug" excludes ideas and praise - the owner wants "Give Feedback". The aria-label
  "Report a bug or a wish" is used by three e2e files (`e2e/tests/bug-report.spec.ts:9`, `bug-triage.spec.ts:16`,
  `frames-m141.spec.ts:96`) and prose in `community/bugs.rs:199`, `community/what_next.rs:59`, the file header L3.
  Builders must update all of them in one change. R-H1.
- Type is a two-segment `SegmentedControl` (L444-451). Three labels (Bug, Idea, Praise) are within 2-5 options, fine;
  stored values remain `bug`/`wishlist` (mission H1); praise adds a body prefix. The server check at L72 rejects
  anything else, so the UI maps before the call; do not widen the server enum.
- Select mode (`data-feedback-label` click capture, L~280) is the right bones for a region picker, but it picks
  elements, not rectangles. Region capture is progressive enhancement; the floor (file / paste) must work with
  JavaScript capture absent (I1: manual path survives failure). R-H2 specifies the layout; the dialog must hide
  (not unmount) during capture and return with the attachment inline.
- `bugs.rs:208-245` "Your reports": one table row per report with a Status badge and a "Note from the team" cell; no
  timeline. The statuses already exist (`status_label`, `BugStatus`), so H3 is presentation: R-H3, a row-expand
  strip below the status. The table has five columns and wraps on phone; on phone each report should become a
  stacked `Surface` (Law: no horizontal scroll).
- `admin/proposals.rs:94` list + editor pattern is right for the import action. Draft proposals are invisible to
  members by construction, so the import UI has to say that and show "Not published" (the existing warning badge,
  L120). R-H4 puts the action in the page header actions and a result summary `Callout`.

## Identity findings → proposals

Not recipe cards. These are brand-direction observations the builders must not act on in this batch; they go to the
agent-proposals document (group I) for the owner to accept or decline.

1. **"Chalk means you."** The app already uses the chalk accent for the viewer's involvement on the timeline and
   (after R-A3) for the player's champion. Proposal: state it as a principle in DESIGN.md section 4 (accent = your
   thing: your champion, your involvement, your selected row), and audit the other places accent appears as
   decoration. Cheap, strengthens the "Survey" direction because a surveyor's mark is a personal station mark.
2. **The match page has no signature moment.** DESIGN principle 14 says the invention goes into the Draft Graph and
   the Sand Table. The match detail page is where a coach looks after every game and today it is three stacked
   tables and a strip. The map plus the shared playhead (R-A6) is the one signature moment this page could carry;
   the rest should stay quiet. Proposal: promote "replay map on the match page" from stretch to a Sand Table
   milestone with the replay import (`store_replay`) invoked on first open of a match.
3. **Voice of the feedback loop.** "Give Feedback" with Bug / Idea / Praise is correct but generic. The product's
   own voice is plain-product-language plus a serif-italic `note` for annotations. Proposal: the reporter's status
   timeline (R-H3) and the team's reply use the `note` style as a "margin note from the team", so replies feel
   handwritten into the survey. Praise deserves a counter on the admin hub ("what works") so the owner sees what
   to protect, not only what to fix.
4. **Suggestion chips (wish fy9o9cmh).** The visible language for system suggestions in this direction is the
   surveyor's annotation (serif italic on a leader line, `Annotation` in canvas.rs), never a chat bubble or
   sparkle. Accept / Dismiss as two `Quiet` buttons, a reason sentence naming the Draft Graph evidence, and
   dismissals remembered per plan. That keeps "every decision conscious" (rubric I4/I8) and avoids the generic AI
   chip look. Proposal only.
5. **Sorted-column glyph and other micro-marks.** The app has a vocabulary of survey marks (tick, cross, star
   U+2605/2606). Sort chevrons, "filled from match" and "inferred" should come from one 10 px stroke-1.5 SVG set so
   marks do not drift. Proposal: a `Mark` primitive in a later round.
6. **Dense data screens vs. the 80rem cap.** Two six-to-seven column team tables cannot sit side by side at the
   Wide cap. Proposal: allow `PageWidth::Full` for match detail once the map exists, with a mirrored matchup layout
   (blue left, red right, role rows aligned). Until then stack.
7. **Inferred events.** If recalls are inferred from purchase bursts, DESIGN needs one visual rule for "inferred,
   not reported" (`RoleLabel guessed` exists; the timeline would be the second user). Proposal: name it
   "inferred = dashed outline" in section 6.3.

## Staleness flags

- STALE: `src/components/ui/button.rs` doc ("the spinner is decorative and the label stays") - true but incomplete;
  update with R-F1.
- STALE: `docs/DESIGN.md` 6.6 "Save state in the header" says editors show it; the champion pool, game-plan
  checklist and others autosave with no header state (no shared helper exists; M137 item 4 was never built). R-C2.
- CONTRADICTION: `src/server/riot.rs:684` comment "Riot API uses ITEM_UNDO for recalls (per D-07)" vs
  `src/server/sand_table.rs:188` listing `ITEM_UNDO` as a deliberately ignored event type. One of them is wrong;
  the sand table importer is right.
- STALE: `src/components/ui/canvas.rs` and `timeline.rs` headers say "Reserved ... no behaviour yet" - accurate,
  but the mission and the owner both believe a rift map exists. Surface that in the Match detail card.
