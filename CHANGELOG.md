# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions will follow semantic versioning once releases start.

Every milestone heading carries its date: `### Milestone M7: Title (YYYY-MM-DD)`. Dates before M7 were filled in from the git history (the commit that first wrote each entry). The app shows this file on /changelog.

A line `Introduce: /route | Feature | How to try it.` inside an entry introduces a new feature where it is used: people who used the app before the entry's date see a one-time "New" note on that page and a "New" mark on it in the navigation (M9). It is not listed as an item.

## [Unreleased]

### First deploy (2026-10-13)

#### Fixed
- For operators: visitors over IPv6 all looked like one address to the rate limiter (Docker's proxy stood in front of them), so one person's sign-in lockout would have locked out everyone on IPv6. The production stack now gives its network IPv6 as well, and each visitor is seen as themselves.
- For operators: invite links no longer leave their code in the web server's access log.
- For operators: the restore drill run as root failed with "Permission denied" because the scratch directory stayed root-only; `restore.sh` now hands it to the image's user. The drill notes name the two path overrides needed with a Storage Box SSH key, the runbook says a changed Caddyfile needs the Caddy container recreated, and `docs/ALERTING.md` is the contract for whatever forwards the monitoring alerts.

### Launch follow-up (2026-10-12)

#### Added
- A "Source" link in the footer of every page: the code of Team Companion is public.
- For operators: backups to a Hetzner Storage Box over SFTP (still encrypted before they leave the server, still kept 30 days), with setup steps in the runbook; S3-compatible storage still works. Two scripts for a second server: an uptime check that mails once when the site goes down and once when it is back, and a backup watch that mails when the newest backup is more than 36 hours old.

#### Changed
- Games opened from the match history are cached for 12 months and then deleted (a nightly task); the privacy page says so.
- When you delete your account, your bug reports stay so the bug board keeps its history, shown as from a former member, without your text, the element you pointed at, your window size or an id in the page address (such as your own player card). The privacy page says so; "Download my data" still includes them in full before you delete.

#### Fixed
- A page now and then never finished loading (about one in several thousand, more often on a busy server): one section kept its loading placeholder although everything had loaded. The cause was in the web framework and is fixed by its update; the server also logs any request that takes longer than 15 seconds, with how far it got.

### Polish before launch (2026-10-12)

Introduce: /team/dashboard | Invite a player | Team leaders: create a link here that signs a new player up straight into your team.

#### Added
- Invite a player: the team leader creates an invite link on the team page (how many uses, when it expires, optionally a position). Whoever signs up with it is on the team at once, not placed until the leader places them, and the welcome says which team they are on. Someone already signed in without a team joins with one click; someone already on a team is told to leave it first. The leader sees each link's uses and can disable it.
- Admins can bind an invite code on the admin page to a team.

#### Changed
- Guidance wording: the welcome says "Welcome" once; one order for "Create or join your team"; steps that wait for a team say "Needs a team"; the checklist explains invite links; "nudges" is the one name for the Match Day lines; shorter captions.
- "Picking up again?" on Match Day counts visits too: someone who opens the app every day without writing anything no longer sees it. The app notes when you were last here, to the hour (listed on the privacy page and in your data download).
- The app has its name: Team Companion. It replaces the working title in the browser tab, the navigation, the footer's Riot disclaimer, the legal pages and every page that named the app.
- Impressum, privacy page and terms are filled in with the operator's details: who runs the app and how to reach them, the hosting (Helsinki) and backup storage (Falkenstein), access logs kept for 14 days, the data protection authority you can complain to, the price (using Team Companion costs nothing; support is voluntary), liability and governing law. The privacy page states the legal basis for each purpose; those bases and the terms' liability and law clauses are marked as drafts until the operator's adviser has confirmed them.
- Transparency: support is shown as what supporters paid, before the payment platform's fees, and the fees appear as their own running cost. The new column label needs an admin's approval like every money text.
- Draft Graph: early in a draft one strong link with the only pick no longer fills synergy or counters, so the top candidates stop tying; the canvas draws lines only for the champion in focus instead of a web of every relation; the ranking loads less data (full explanations for the top eight and the focused candidate, the rest when you focus them).

#### Fixed
- The server closes the database within a second of a stop even after a long busy day; before, requests whose page never finished loading could keep it open, and a stop then took the full wait and ended with an error code (no data was lost).

### Milestone M9: Welcome and guidance (2026-10-12)

Introduce: /profile | Guidance | Turn the welcome, the checklist, hints and nudges off here, or show the checklist, hints and nudges again.

#### Added
- A welcome on your first visit to Match Day: the loop in one line (scout, draft, plan, play, review), what to do first, and the optional Riot link. "Start" takes you to the first open step; "Skip, I'll find my way" hides it for good.
- Getting started: a checklist on Match Day (join or create your team, fill the starting five, add a first opponent dossier, make a draft, write a game plan, review a game, invite a teammate). Steps tick themselves from what your team has actually done; joining an established team ticks its steps from the start. "Hide this checklist" hides it; "Guide me" in the command bar (Ctrl K) shows it again any time.
- One-time hints where a feature is, each with "Got it": the draft board's keys, the Draft Graph, branching a draft tree, share links, Undo on team notes, champion pool tiers, and the command bar. They stop once you have used the feature.
- One quiet line on Match Day when something is waiting: picking up again after two weeks away, a match with a plan but no review, action items open for over a week, a draft without a game plan, a scouted opponent without a draft. Each kind can be turned off with one click.
- New features are introduced on the page where they live, once, with a "New" mark in the navigation until you have seen them. The Draft Graph and share links are introduced this way to everyone who used the app before they arrived.
- Guidance on your profile: one switch for all of it, "Show the checklist again", "Show hints again" and "Show nudges again". "What's new" in the command bar opens the changelog.
- For operators: a promote command makes an existing account an admin without editing `ADMIN_EMAILS` (`promote <email>`, `--revoke`, `--list`; see the runbook).

#### Changed
- The privacy page lists the guidance you have seen or turned off; "Download my data" includes it and deleting your account removes it.
- The old Solo/Team setting, unused since the navigation rebuild, is removed from accounts.

### Milestone M8: Ship-ready (2026-10-12)

#### Added
- Download my data: on your profile, one click saves a file with everything Companion stores about you (account, Riot link, team membership, champion pool and notes, learnings, goals, synced games, notes you wrote, bug reports, votes and survey answers, and the team records you created by name).
- Delete my account: on your profile, with your username and password. Your personal data goes at once and you are signed out everywhere; what you wrote for your team stays with the team as written by a former member. A leader hands leadership to a teammate first, or deletes the team with the account when they are its only member.
- Hand over leadership: the team leader can make another member the leader from the member's actions on the team page.
- Keep your player card out of share links: a switch on your profile; turning it on also ends the links that already exist. Teammates still see the card in the app.
- Terms of use (`/legal/terms`), linked in the footer, and a complete privacy page: what is stored, who else is involved, the one cookie, the logs, how long things are kept and how to use your rights. Facts only the operator can give are marked until they are filled in.
- The Riot verification file at `/riot.txt`.
- For operators: a container image, a Docker Compose stack with automatic HTTPS (Caddy) and nightly encrypted backups with 30-day retention, a restore drill, a runbook and a launch checklist. CI builds and runs the image on every change.

#### Changed
- Settings are checked when the server starts: a mistyped value stops it with a list of every problem instead of silently using a default.
- The server shuts down cleanly: requests in flight finish (for up to 10 seconds) and the database is closed before it exits, so restarts for updates and backups are quick.
- Match Day shows the patch line only for two weeks after a patch change.
- Share pages show visitors a bare header, without Sign in and Request access.

#### Fixed
- Opening a review with a link to a draft or plan that is not your team's starts the review without it, instead of failing when you save.
- The phone menu no longer shows old counts after a failed load.
- An older version of the app started on a database from a newer version now stops before it changes anything.
- Leaving a share page for Home brings back Sign in and Request access without a reload.
- The team page lists five recent games even when a large roster played the same games together.
- Deleting an account no longer clears action items or note editor names in other teams that use the same name.

### Milestone M7: Community and transparency loop (2026-10-12)

#### Added
- Bug board (`/bugs`) for everyone in the beta: reported problems and wishes the team has read, with a title and a note the team writes, their status (new, triaged, planned, fixed, won't fix) and the update that fixed them. Your own words stay private.
- Your reports: every report you sent with its status and the team's note, marked "Updated" when something changed; the account menu says how many changed.
- The bug-report button is rebuilt: point at the part of the page (or skip that step), choose Bug or Wishlist, and send. It sits above the footer and never covers the page.
- What next (`/what-next`): ideas the maintainer is considering, planning, building and has shipped, ordered by votes. One vote per idea, and you can take it back.
- Changelog (`/changelog`, public): this file, newest first, with the game patches the app detected.
- Transparency page (`/transparency`, public). It stays empty until the owner has approved its wording and published a month.
- Short survey questions, at most one a week and only right after something you did; dismiss any of them, or turn them off on your profile.
- Admin area (`/admin`): bug triage, surveys with approval of the exact wording, What next ideas with a publish step, the monthly ledger and its public texts, each recording who changed what and when.

#### Changed
- The account menu has one "Admin" entry for admins (it replaces the separate admin links) and "Your reports" for everyone. The footer, the command bar and the phone menu link the community pages.
- The privacy page lists bug-report triage data, votes and survey answers.

#### Fixed
- Clicking "Report a bug" twice, or following a menu link while choosing the part of the page, no longer leaves the page in select mode or breaks later clicks.
- "Add month" on the ledger admin page no longer overwrites a month that already exists; change it with Edit instead.
- A survey question shown again after its wording changed now records the answer against the wording the person actually saw.
- Screen readers now announce a new survey question and the thank-you line.
- Closing or sending a report returns focus to the Report a bug button.
- Every changelog entry now has a date.

### Milestone M5: Remaining pages, navigation, mobile (2026-10-12)

Introduce: /game-plan | Share links | Open a saved plan and choose Share: the read-only link opens without signing in and expires when you say.

#### Added
- Match Day: signed in with a team, Home walks the team through the next scrim or match. Set it with date and time (in your own time zone), opponent, format and notes; the page then shows a preparation sequence built from your own records (read the opponent brief, draft plans against them, a game plan from those drafts, open action items, and the review after the game), with the one step to do now and a link to where it is done. Beside it: the opponent brief with when the dossier was last surveyed and who on the team has read it, open action items, the last review's lesson, form over your last five reviewed games, and the current patch. Without a match it says how to start; on a phone it is a read-only summary in the same order.
- Substitutes: the leader can start a player in a position, move them to the bench, make them a coach or take them off the roster. New members start as "not placed yet". Substitutes see everything the team does.
- Team notes can be undone: after an edit or a delete, "Undo" in the notification puts the note back (for a day after a delete), and a note says who last edited it.
- Read-only share links for a game plan, a review and a player card: they open without signing in, show only that one thing, expire after a day, a week or a month, and can be revoked at any time.
- Player cards (from the roster): position and place, best pool champions, recent form and most played champions.
- My games: your synced games, ranked standing, goals (rank, CS per minute, deaths per game), LP history and champion trends with 7, 30 and 90 day filters, all in one place.
- A phone menu: every page grouped by when you need it (today, after the game, team, you), with counts such as "3 open" on the right.

#### Changed
- Navigation has five hubs: Match Day, Prep, Review, Team and You. The Solo / Team switch is gone: personal pages are under You and no page asks you to switch modes any more. Signing in and registering land on Home.
- The team page shows the roster as starting five, substitutes, coaches and not placed yet, with join requests, team notes, recent games and pool gaps; the roster page shows members their team and how players join, and offers creating or joining a team only to people without one.
- The profile shows your email and holds the Riot link with what a sync costs; the stats page is now "Team games" (grouped by game, with a summary per member); match detail, learnings, sign-in, registration, the legal pages and the invites page are on the design system.
- Match timeline markers have a shape per kind of event (objective, tower, kill, ward, recall, teamfight), with a legend.
- The command bar lists every page; the bug-report button sits above the footer at every width, so it never covers content.
- Dragging a champion onto a filled draft slot swaps the two instead of replacing one.
- "Overdue" on action items follows your own calendar day.
- Plans and reviews whose draft was deleted say "draft deleted" instead of pointing nowhere.
- The game plan's step list no longer competes with "Save plan" for attention.
- Champion pool positions are stored in one spelling, and duplicates that differed only in spelling were merged.
- Team pages load faster for larger teams.

#### Fixed
- Creating an account before the page had finished loading failed with "Registration failed" even with a valid invite link.
- Saving a game plan started from a draft before the draft had loaded saved it without the link to that draft; "Save plan" now waits for the draft.
- An empty state right under a page title no longer draws a second line under the title's own.
- After opening an old address that redirects (the solo dashboard, analytics, team builder), pages could intermittently fail to load their scripts.
- The phone menu stays a full-screen sheet over the page once its counts have loaded.
- Wide tables on phones (such as the invite list) scroll inside their frame instead of widening the page.
- The bug-report dialog has its dimmed backdrop again (it was transparent after the colour clean-up).
- Deleting an opponent's dossier clears it from the next match instead of leaving a match that could not be saved; choosing another opponent resets who has read the brief, and "Read by" counts only current members.
- A deleted team note can be restored for one day, as promised, and is then removed for good.
- The privacy page now lists everything stored, including series, ban priorities, checklist templates, Sand Table boards and imported replays.
- Shared pages ask search engines not to index them.

#### Removed
- The standalone solo dashboard (now My games and your profile; its address redirects), and the parked analytics and team builder pages (their addresses redirect to Team games and Draft).
- An invented "14 days" log retention on the privacy page; the operator's details and the legal sections are marked placeholders until they are filled in.

### Milestone M4: Draft Graph, stage B (the canvas) (2026-10-11)

#### Added
- The Draft Graph view now opens with a force-directed graph of the pick being decided: your locked picks pinned on the left and theirs on the right (one row per role, blue circles and red squares), the top candidates floating between them, pulled toward the picks they work with and pushed away from the ones that beat them. Fit is drawn as elevation: a spot height ("▲ 84") and one contour ring for every 10 fit from 50. Lines say what kind of relation it is (solid synergy, dashed counter with an arrow at the one that loses, dotted when weak or working against), their width how strong, their colour whose pick it is; a legend under the graph explains every mark, and the grid references (A to H, 1 to 5) let you say "Thresh, D3" on voice.
- The graph, the ranked list and the Why panel follow one focus: focus a candidate anywhere and its connections light up while the rest fades; click one of the locked picks to see every candidate related to it. A line under the graph says the same in words ("Thresh, fit 87, at D3: synergy with Jinx, Vi and Orianna; beats Sejuani").
- The graph works from the keyboard as one stop: arrow keys go through the candidates in rank order, Enter picks, Escape goes back to the board. With the mouse, click focuses, double-click picks, and you can drag a candidate (the layout settles again).
- The layout settles in well under a second and then stays still; with reduced motion turned on, the settled picture appears at once. On phones the graph is square and shows the top 12.

#### Fixed
- The side marker in the draft page's header now follows the side switch.
- When the graph ranks the other side's pick, your draft's composition tags (teamfight, scaling, ...) no longer count for their candidates.
- Two team members who met as opponents in the same game no longer count as a pair "together in your games".
- A quick second pick from the graph (before the ranking for the next slot arrived) no longer replaces the champion just picked.
- The ranked list says when a candidate's traits are low confidence, not only the Why panel.

### Milestone M4: Draft Graph, stage A (the model) (2026-10-11)

Introduce: /draft | Draft Graph | Switch the side panel from Picker to Draft Graph to see how each candidate fits your draft, and why.

#### Added
- The draft page has a Draft Graph view beside the board (switch "Picker | Draft Graph"). For the pick being decided it ranks the candidates by fit (0 to 100, from a neutral 50) and says why: synergy with your locked picks, counters against theirs, how the pick fits the draft's composition and what the comp still lacks, and whether its power curve matches the comp's plan, in that order of weight, plus the player's champion pool. Every point has a reason a coach can read and argue with; there is no win probability.
- A "Why this pick" panel for the focused candidate: fit by part, the reasons grouped by where they come from (the trait rules, your team's notes with their author, your own games with the number of games, scouting, the player's pool), and "Argue with it", which opens a team note prefilled with the pair.
- Team notes on pairs ("X works with Y because ...", "X beats / loses to Y because ...", slight, clear or strong): every member sees them, the ranking uses them with your name on the reason, and they can be removed with Undo.
- A comp profile per side (damage mix, engage, front line, crowd control, wave clear, when it is strongest) with the change the focused candidate would make.
- The ranked list works from the keyboard: arrow keys move the focus (the Why panel follows), Enter or a double click picks into the current slot, Escape goes back to the board.
- Admins can review and edit the champion trait table on `/admin/traits` (search, "defaults only", "edited", "low confidence"; who edited when; reset to the shipped default). Everyone else gets the 404 page.
- The trait table ships with a kit-based default row for every champion (what the abilities structurally do, nothing patch-specific), each marked as a default until reviewed; champions the defaults do not know yet get neutral values marked as low confidence.
- Your own games count as evidence for pairs and lane matchups once there are at least 5, always shown with the number of games, and only ever strengthening or weakening a reason that already exists.

#### Changed
- A role the page guesses for a pick is now stored as a guess and drawn as uncertain on the board (dashed underline and "?", read as "role guessed"); choosing a role confirms it. Guesses use the champion's roles from the trait table, preferring roles your side has not filled yet. Drafts saved earlier show their roles as confirmed.
- Seeded demo pools use the champion pool page's tiers.

#### Fixed
- Signing in or registering before the page has finished loading no longer leaves you signed in but stuck on the form.
- Closing a view that contains a form field no longer breaks the page (a field could read its own state after it was gone).

### Milestone M3: Strategy pages onto the system (2026-10-11)

#### Added
- Action items have an optional due day; overdue items are flagged, and the header counts what is open, in progress, done and overdue.
- Action items can be edited in place, deleted with Undo, and worked from the keyboard: arrow keys move between items, Space completes, E edits, Delete removes.
- Opponent dossiers show how much evidence each holds (players and champions seen) and when they were last surveyed, flagged after a week; empty role slots can be filled with a Riot ID or a name.
- Champion pool tiers can be rearranged from the keyboard: Space picks a champion up, the arrow keys choose a tier, Space drops it there.
- Drafts can be deleted (with a confirmation).
- Saved drafts, game plans and reviews keep their id in the address, so a reload or a shared link reopens them.

#### Changed
- The draft page is one compact tournament board: blue left, red right, five bans per side grouped by ban rotation, then the picks in pick order grouped by rotation, with only the current step lit and a counter such as "Pick 7 of 10 · Red to pick". It fits beside the champion search on a laptop screen and works entirely from the keyboard (arrows, Enter, Escape, Backspace). The earlier draft board showed three bans per side instead of five.
- The draft page shows the opponent's brief beside the board, and saving, comments and "next steps" (prep a game plan, review the game) sit around it; saved drafts open from their address.
- The tree drafter, champion pool, game plan, post-game review, opponents and action items pages were rebuilt on the design system, each with one page title, clear save states and empty states that say what will appear.
- The game plan checklist uses real checkboxes and shows its progress; a saved plan can start a review that is already linked to it.
- Faint control borders across the app now meet the 3:1 contrast minimum.
- Tooltips stay open while you move the pointer onto them; the command bar's search shows a focus ring; the Solo / Team switch in the navigation moves with the arrow keys and only switches on Enter, Space or a click.
- On phones the bug-report button sits in the footer area instead of covering the page.
- Simultaneous requests for the same champion picture share one download.
- Lines under "Action items" in a post-game review now become action items when you save, as the form says (lines under "To improve" still do too; open duplicates are skipped).
- Older drafts that name their opponent open with that opponent's scouting brief beside the board.

#### Fixed
- Deleting the open tree in the tree drafter no longer breaks the page.
- A game plan started from a draft no longer replaces a name you typed while the draft was loading.
- "Branch" in the tree drafter puts the cursor in the new branch's name field.

#### Removed
- The draft page's carousel, table and ledger views and its first-pick toggle (blue picks first in a tournament draft).
- The privacy page no longer lists a theme preference among the stored data.

### M2 review fixes (2026-10-11)

#### Fixed
- Sign-in and registration no longer clear an email or password that your browser filled in (or that you typed) while the page was still loading.
- The Solo / Team workspace switch is back in the navigation on phones.
- A few faint labels on the draft, post-game, stats, tree drafter and learnings pages now meet the contrast minimum.
- Champion pictures for multi-word names such as Nunu & Willump load again where the older picture component is still used.
- A short outage of the image source no longer leaves champion pictures missing until the next patch; the image routes are also rate limited per visitor.

### Milestone M2: Design system foundation (stage B) (2026-10-11)

#### Added
- A command bar: press Ctrl+K (Cmd+K on a Mac), or use "Go to" in the navigation, type a few letters and press Enter to jump to any page. Commands for drafting and Match Day arrive with those pages.
- The account menu, dialogs, menus, notifications, tooltips, switches and segmented controls are now one consistent, fully keyboard-operable set. Notifications appear bottom left, can carry an action such as "Undo", and pause while you point at them.
- New gallery sections for every interaction and domain component, including the coordinate frame, legend, surveyor's notes and replay timeline reserved for the Sand Table and the Draft Graph.

#### Changed
- Roles are shown as short labels (TOP, JGL, MID, BOT, SUP) instead of icons loaded from a third-party site; your browser now loads nothing from outside Companion.
- Champion pictures show a two-letter monogram while they load or when an image is missing.
- Section headings say what the section is ("Starting roster", "Coaches", "Bench", "Recent games", "Team notes", "Open action items", "Recent reviews", "Pool gaps") instead of fantasy labels such as "Order of battle" or "Battle log".
- Every page uses the design tokens: side colours only for sides (match timeline events now use the blue and red side colours instead of status colours), results in their own colours, statuses with words, one rounded shape only for avatars and switches, and loading placeholders that stop moving when you ask your system for reduced motion.
- The destructive button turns dark crimson on hover instead of a pale pink.
- The first-pick choice on the draft board and the Improvised flag in the tree drafter are now a segmented control and a switch.

#### Removed
- The per-user theme setting stored in the database (there is one theme); a migration removes the stored values.
- The "resource read outside Suspense" warning from the bug-report button on every page load.

### Milestone M2: Design system foundation (stage A) (2026-10-11)

#### Added
- A new visual language, "Survey": a dark green-slate ground in tonal layers, one chalk accent for focus, selection and the next action, matched blue and red side colours, win and loss on their own colours, and plain, precise type. Every text and control colour meets WCAG AA contrast.
- New typefaces, served by the app itself: Familjen Grotesk for titles, Public Sans for text, JetBrains Mono for numbers, Source Serif 4 italic for annotations. Numbers line up in every table and readout, and no text is smaller than 12px.
- A design-system gallery at `/design` for admins (and developers): every component in every state, the colour tokens with their measured contrast, the type scale, a worked tournament draft and the Match Day preparation checklist.
- A tournament draft layout that shows the real order (five bans and five picks per side; blue starts, red opens the second ban phase) with the current step highlighted, and a "what to do next" checklist for Match Day. Pages adopt them in M3 and M5.
- A "Skip to content" link for keyboard users and a visible focus outline everywhere.

#### Changed
- Champion and item pictures load from Companion's own server (cached per patch) instead of from Riot's image servers, so your browser no longer contacts a third party for them.
- Sign-in, registration, the closed-beta page, the 404 page, the navigation, the footer and the headers of the solo and team dashboards use the new components; every other page already wears the new colours and type and is rebuilt in M3 and M5.
- The solo dashboard's views are called Overview, Rank and Journal; the last ten results show W and L marks.

#### Removed
- The heraldic ornaments, the painted background images, gradients, glows and blur, emoji icons and the leftover second theme.
- The theme card on the profile page, which pointed at a switch that no longer exists.

### Milestone M6: Data layer and integrations (2026-10-11)

#### Added
- Riot routing follows each player's region from their profile (and the team's region for team syncs and opponent scouting) instead of always using EUW. The profile only accepts the listed regions.
- When the app's Riot API budget is used up, Riot features answer "Riot is busy, try again in N s." instead of stalling.
- Champion images and data follow the current patch automatically (checked every 6 hours), and champion lists still work after a restart without internet access. New patches are recorded for later display.
- Storage for the upcoming Sand Table: imported match timelines (per-minute positions and economy, positioned events) and versioned tactics boards. No screens yet.
- The seed tool creates fuller demo data (two complete teams with drafts, plans, reviews, action items, notes and scouting) and can be run repeatedly.

#### Changed
- Usernames are unique regardless of case at the database level; a few accounts from before M1 whose names differed only by case get a numeric suffix (for example `bob-2`).
- A player can only ever be in one team, now also enforced by the database; extra memberships from before M1 are removed (the team the player leads, or else their first team, is kept).
- Saving a draft, a review, an opponent with players, a draft tree, a team or a deletion with its children either completes fully or not at all.
- The team match sync can run once a minute per team (it was per member).

#### Fixed
- The team dashboard's "drafts without a game plan" and "game plans without a review" counts counted everything.
- The 7/30/90-day windows for LP history and champion trends did not filter by date.
- Several lists could come back empty when a stored date failed to load (the same cause as the personal learnings bug); dates now load the same way everywhere.
- Invite codes could show the wrong status within the second they expired.
- Two people registering the same username (or email) at the same moment could both be told it worked while only one account was created; a tactics board saved twice at once could lose one version.
- Linking a Riot account or scouting an opponent failed for the OCE, SG, TW, VN and ME regions.
- When Riot was busy, a match sync also started the one-minute pause, so the suggested retry was refused.
- Accounts from before M1 whose email addresses differed only by case could no longer sign in when none of them was stored in lowercase; the oldest of them can sign in again.

### Milestone M1: Security hardening (2026-10-11)

#### Added
- Invite codes are checked on the server. Registering needs a valid code; each code has a use limit, an optional expiry and can be disabled.
- Invite management page for admins (`/admin/invites`): list codes with their uses and expiry, mint a code, disable a code. Admins are set with `ADMIN_EMAILS`; everyone else gets "Page not found".
- The seed tool prints an invite code for local registrations.
- Rate limits on sign-in, registration and everything that calls the Riot API, and a one-minute pause between match syncs. Hitting a limit shows "Too many requests. Please slow down..." with the wait time.
- `/healthz` reports HTTP 503 when the database does not answer.

#### Changed
- Passwords must have at least 10 characters.
- Email addresses are case-insensitive for sign-in and registration; usernames must be unique regardless of case.
- Registration failures for an invalid invite or an email that already has an account show one generic message.
- Error messages no longer contain internal database or Riot API details.
- Sessions expire on the server and expired ones are removed every hour; the session cookie is marked `Secure` outside development. Existing sessions end once with this update (sign in again).
- A player can belong to one team at a time.
- The team list on the roster page needs a signed-in user.

#### Fixed
- Security: a signed-in user could read, change or delete other teams' drafts, draft trees, game plans, checklists, reviews, action items, team notes, opponents and private personal learnings by guessing IDs, and opponent scouting worked without signing in. Every request is now checked against the user's own team or account.
- Security: tree branches could be used to run arbitrary database queries (query injection). All values are now passed as parameters.
- Security: a team leader could accept join requests sent to other teams, and accept the same request twice.
- Personal learnings were never listed (a date field failed to load).

#### Removed
- The unused Leaguepedia pro-play scraper.

### Milestone M0: Reset and safety net (2026-10-11)

#### Added
- Site footer on every page with Riot Games' legal disclaimer and links to the legal pages.
- A real "Page not found" page for unknown addresses (HTTP 404).
- `MISSION.md`, `docs/ARCHITECTURE.md`, `docs/DESIGN.md` (skeleton for M2) and this changelog.
- SessionStart hook for cloud sessions (cargo-leptos, clippy/rustfmt, e2e dependencies, Playwright browsers, `.env`).
- CI now runs clippy for the WASM target and the Playwright smoke and hydration suites against a built server.

#### Changed
- Toolchain moved from nightly to stable Rust; the BFD linker workaround is gone.
- Debug builds carry line tables for this crate only (no dependency debug info), which shrinks test and server binaries enough to link in a small container.
- The browser tab title is now "Companion".
- Tree drafter: the empty tree list now tells team members how to create a tree instead of asking them to join a team.
- DB integration tests are one test binary (`tests/integration/`), so `cargo test --features ssr` links once.
- Bug-report export moved from `.planning/INBOX/` to `docs/inbox/` (git-ignored).
- Code formatted with `cargo fmt`; clippy is clean with `-D warnings` on both targets.

#### Removed
- Invented data and dead controls: solo dashboard pool-gap suggestions, placeholder KDA/CS stats, the Forge tab's sample champions and "Queue" buttons, the no-op sort buttons; the team dashboard's Game Day Brief (fake bans, threats and results) and the Pandemonium dashboard panels; the War Table's fake composition scores and the draft carousel's fake confidence line. Each now shows an honest empty state or is gone.
- The disabled "Live" hub and the dead "Match" link in the navigation.
- The Pandemonium theme option and theme toggle; every request now renders the single dark theme.
- Old planning tree (archived under the git tag `planning-archive-2026-10`), dangling GSD agent links and stray test output.

#### Fixed
- Login and closed-beta pages: background images no longer push the card off-screen.
- Every page requested `/pkg/tailwindcss` and got a 404; the stylesheet is now built only by the Tailwind pipeline.
- The tree drafter showed the literal text `&#x1f333;` instead of an icon.
- Warning, success and info badges rendered grey instead of their status colours.
- The draft ledger view assigned phase-two bans to the wrong side.
