# M160 item 0: atelier critique and recipe cards (r1)

Mode: critique + recipe, from code and `docs/DESIGN.md` only. Tokens: this repo's `input.css` / DESIGN.md (Survey). No render was made; nothing here is a visual PASS. Process note: this run had no subagent tool, so the "Opus edits / Sonnet reads" clause could not be followed; reads and this one file write were done here. No app file touched. Frames read: M148 r2 critique (dashboard-invites, match-day-phone) for the team page.

## Decisions up front
- Keep `/` as NoTeamHome for solo users. Do NOT redirect `/` to `/me`: `/` is where the guidance (welcome, checklist step 1) lives and where a returning "join link" user lands; a redirect hides the only screen that explains the product. NoTeamHome must carry the single primary action ("Create a team") and the quiet "Find a team" path.
- Region on checklist step 1: show it only when done, as the Done label ("Created, EUW"), never as a prompt. Region is chosen in the create form, so asking again is redundant; but a done step that says what was chosen is honest feedback and makes the wrong-region mistake discoverable (Edit team). Joiners get no region text (they did not choose it).

## Critique per surface

**Solo nav (390 / 1280).** `HUBS` always lists five hubs; a solo user sees Prep, Review, Team that all land on a "No team yet" wall. That is navigation that promises what it cannot deliver (honesty rule, DESIGN "one primary action per view"). Fix by filtering hubs on team membership, not by greying them (disabled links are controls that do nothing). The command bar's `PAGES` entries must filter the same way, or it becomes the back door to the wall.

**NoTeamState.** Message says "on the roster page" and the CTA says "Go to the roster": it names a page that is being merged away, and the sentence is 30 words for a state with one job. It also sends a joiner to a page whose first thing is a long create form. Make the copy name what is missing for THIS page (draft/plan/review) so four pages are not four identical walls. One chalk CTA only.

**Founder checklist.** Current order puts the starting five before the dossier but after create/join; the planned order adds "invite" between them. Problem: step 2 "Fill the starting five" says "The team leader places players", a dead end for a non-leader; steps are shown to everyone. Items for non-leaders should read as team facts, not instructions to them. Invite step needs a real done-rule (>= 2 members or an invite ever minted), otherwise it is a checkbox that ticks itself and lies.

**Roster + dashboard.** Two pages with one concept. The dashboard header button "Roster and joining" is a sideways link to the second half of the same job; "How players join" prose duplicates "Invite a player". Merging is right. Risk: the dashboard is already a 2-column grid of seven sections; merging by appending makes it a long scroll. Order by who is acting, not by data type (card b).

**Opponent PlayerRow.** The row already carries Riot ID, survey, intel, delete. A permanently visible textarea per player x 5 players x N dossiers = fifteen empty boxes, which is the "rounded-card kit" anti-pattern and violates Law 11 (opened = nothing expanded). The note field exists already in `save_player` (`pnotes` is carried through but not editable here).

---

## Card (a): solo nav
- Technique: filter `HUBS` and `PAGES` by `has_team` in one place (a pure fn, unit-tested), used by the desktop bar, phone menu and command bar.
- Primitives: existing nav hub links; a `Button variant=Quiet` styled as link or the existing link class; no new primitive.
- Tokens: `text-secondary` link, `text-primary` on hover/active; chalk `bg-accent` is NOT used in the bar.
- Copy: hub "You" (lands `/me`); link "Find or create a team" -> `/team/dashboard`. Nothing else. Do not say "Join" (the invite link is the join path; this is search + request).
- States: signed-in no-team: [Match Day?] see below, You, link. Member: unchanged five hubs. Loading (team unknown): render the member bar only after membership resolves; show the skeleton width, not a flash of five hubs that collapses to one.
- Match Day hub: keep it (it is `/`, NoTeamHome) so the logo and the first tab agree; hubs shown solo = Match Day, You + the link. (The brief said "only You": I argue Match Day stays because `/` remains the home; if the owner insists on You only, `/` must stay reachable via the wordmark.)
- 1280: bar reads `Match Day  You    Find or create a team` with the link right-aligned, 14px, same baseline; sub-nav for You as today. 390: phone menu `Sheet` lists Match Day, You group (My games, Profile ...) then one row "Find or create a team" at the end, 44 px row; no empty "Prep/Review/Team" group headings.
- Do NOT: grey out or lock-icon the hidden hubs; show a count/badge; add a banner explaining the hiding; make the link chalk (the page body owns the one primary action); leave stale `aria-current` on a hub that no longer exists; forget the join-request bell (hide when no team).

## Card (b): ONE team page `/team/dashboard`
Order is by what the viewer can act on first; every section has an id/anchor (`SectionHeader id=`) and `data-*` hooks kept from the old pages.
- No team: PageHeader "Team" meta "You are not on a team yet." Then `#create` (chalk "Create team", name + region Select), `#join` (find a team, "Ask to join"), and a one-line note "Have an invite link? Open it; it adds you to the team." At 1280 create | join side by side (as roster today, `lg:grid-cols-2`); at 390 stacked, create first. No NoTeamState here (this IS the state).
- Member (player/sub): `#roster` (read-only: starting five, subs, coaches, not placed), `#team-notes`, `#recent-games`, `#pool-gaps`. Header action: Quiet "Leave team". No invite/requests sections (not "disabled", absent).
- Coach: Member order + `#invite` (coaches mint invite links; placing stays leader). No `#requests`.
- Leader: `#roster` (with placement actions) first, then `#invite`, `#requests` (badge count in the meta if > 0, shown above the fold at 390), `#team-notes`, `#recent-games`, `#pool-gaps`. Header: Quiet "Edit team"; region/members/leader meta kept.
- 1280: two columns as today (left roster/games/gaps; right invite/requests/notes). Right column order must match the section order above so tab order = reading order. 390: single column in the leader order above; invite right after roster because step 2 of the checklist leads there.
- Redirect: `/team/roster` -> 307 in axum (never a Leptos `<Redirect>`, see CLAUDE.md) to `/team/dashboard#create` when no team is knowable... the server cannot know; so redirect to `/team/dashboard` and let the page scroll: anchors `#create`, `#join`, `#invite`, `#requests`, `#roster`. Old roster deep links: `/team/roster#joining` maps to `#invite`. Checklist hrefs use the same anchors (`#invite`, `#roster`). Anchored sections need `scroll-mt-20` for the sticky bar.
- Delete "How players join" prose; keep ONE sentence under Invite: "Players join with the link; you place them here."
- Tokens: `Surface` only where the page already uses it; section rules `border-rule`; no new card chrome.
- Do NOT: keep both routes rendering; show "Roster and joining" button; show a leader-only control to others as disabled; reorder by data type; put join requests below team notes (a waiting person is the most time-sensitive thing on the page).

## Card (c): per-player notes input
- Technique: native disclosure. Collapsed row shows a Quiet text button "Add note" (empty) or "Note" with a one-line truncated preview (`text-secondary`, `truncate`) when a note exists. Expanded: `Field` + `TextField rows=2`, label "Note on {display}", placeholder "What they do that matters", saves on blur and Ctrl+Enter (`ctrl_enter_submits`), via the existing `save_player` (already carries `pnotes`; replace the StoredValue with an RwSignal).
- Primitives: `Button variant=Quiet size=Touch` (`aria-expanded`, `aria-controls`), `Field`, `TextField`, `SaveStatus`-style inline "Saved" text (not a toast per player).
- Tokens: `text-secondary`, `text-muted` for placeholder only at >= 4.5:1, `border-outline` on the field; `note` serif italic is only for quoted team notes, so do NOT use it for these.
- States: default collapsed always (Law 11), including after a reload when a note exists (preview visible, field closed); expanded keeps focus in the field; saving: "Saving"; failed: danger line "Could not save the note. Retry" with Retry; empty-on-blur after text = clears the note (confirm nothing; it is a plain field).
- 390: the button sits under the name row, 44 px, field full width. 1280: button inline at the end of the row; field spans the row beneath it. Only one expanded at a time is NOT required; do not auto-collapse.
- Do NOT: render textareas up front; open on hover; add a modal; use a placeholder as the label; add a character counter; show invented text in the preview.

## Card (d): auto-place toast
- Technique: when a join places a player automatically (a role preference fills a free starter slot), show `ToastContext.push(Toast::success(..).action("Undo"))` to the leader on the next team-page load/poll; Undo returns the player to "Not placed yet". If no free slot, place nothing and say nothing here; the player sits in "Not placed yet".
- Copy (success): "Mo joined as Mid starter" + action "Undo". Substitute case: "Mo joined as substitute". Role names written out (Top, Jungle, Mid, Bottom, Support), not glyph codes. Undone: "Mo is not placed yet". Failed undo: "Could not undo. Move Mo from the roster."
- Name: the display name, truncated at ~24 chars with an ellipsis (hard word-wrap on the narrow toast); never an email.
- States: one toast per placement; if several joined while away, one summary toast "3 players joined and were placed" with action "Open roster" (-> `#roster`), not three stacked (M148 noted toast-per-step noise). Undo window = the toast lifetime plus the roster action afterwards; placement is always reversible from the roster.
- 390: toast bottom, full width minus margins, Undo 44 px; 1280: bottom-left as today. `role="status"` (polite), never `alert`.
- Do NOT: say "automatically" or "AI"/"smart"; toast to the joiner; chalk-fill the toast; hide the placement from the roster list (it must appear there with its role); claim a role the player did not choose.

## Staleness / contradictions
- CONTRADICTION: `CLAUDE.md` / DESIGN refer to "Law 11"; `docs/DESIGN.md` has no "Law 11" heading (grep found none). The mission file should cite it, or DESIGN 6.x should state "viewer opened, nothing expanded".
- STALE (after the merge): `src/components/ui/empty.rs` NoTeamState copy and cta "/team/roster"; `models/guidance.rs` checklist hrefs "/team/roster"; docs/DESIGN.md 6.9 team page text.
