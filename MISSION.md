# Mission: Companion (working name)

Agreed 2026-10-10 between the owner (Jasper) and the orchestrating agent, from the Companion Mission Contract and the Sand Table Proposal. This file is the contract every milestone is measured against. Change it only by agreement.

## 1. What we are building

A private coaching workspace for one League of Legends team, built together with its users. It connects scouting, drafting, game planning, review and lessons into one loop; it makes drafting visual and explainable through the Draft Graph; it gives the team a tactical map to plan on and to replay games on (the Sand Table); and it is run in the open: users report, vote, get asked, and see where the money goes.

Four pillars:

- **P1 The prep pipeline** — scouting → draft / draft tree → game plan → post-game review → action items and patterns. Exists; must be secured and made coherent.
- **P2 The Draft Graph** — a force-directed view that shows why a pick fits (synergies, counters, playstyle), with reasons for every suggestion. Coach knowledge encoded, not an AI pick bot.
- **P3 The community loop** — bug reports with a status, micro-surveys, a public "what next" board with votes, a changelog, and a transparency ledger for the money.
- **P4 The map (Sand Table)** — a drawing board for plays, a tactical replay of a game's macro from Riot's timeline, and cross-game patterns.

Never: a public stats site, an AI pick bot, a video-analysis tool, pay-to-win (money never buys an advantage in the app).

## 2. Release 1 goal

Closed beta for invited teams: a secure, coherent, team-first coaching companion that an amateur or semi-pro 5-stack plus coach can use for a full competitive week — scout the opponent, plan and branch the draft with the Draft Graph, write the game plan, review the game, and carry lessons into the next prep — on one consistent, distinctive, legible interface, with the community loop built in, deployable with backups and EU-compliant legal pages. The Sand Table's data foundation ships in Release 1; its first full version is the flagship of Release 2.

## 3. Long-term vision

"The team's shared brain, built with its community." Every scrim adds to a private, structured body of knowledge; the app gets smarter on the team's own data and on knowledge the community curates; the map becomes the team's shared language (brief → plan → play → replay → lesson); users steer the roadmap and see the finances.

## 4. Decisions (from the questionnaire)

Identity and audience
- Product name: pick a neutral name before inviting outside teams; working title "Companion". The name must not contain "League of Legends", "LoL" or "Riot", and should avoid Riot IP such as "Demacia". The footer carries Riot's exact legal disclaimer.
- Audience: DACH and global English-speaking amateur/semi-pro teams; UI in English for Release 1.
- Repository visibility: decide at M7.

UI/UX and design system
- Visual direction: explore directions as mockups first, then commit. The owner asked for a state-of-the-art, unique, visionary yet humble and aesthetic experience; avoid classic gamer styling and generic AI-made UI (box soup, everything rounded, purple gradients). Three directions are presented for the owner to choose.
- Density mixed (compact for boards and tables, comfortable elsewhere). Motion purposeful, reduced motion respected. Dark only in Release 1 (tokens allow light later).
- Components approved in three batches (controls, layout, data). Gallery page for developers and admins. WCAG AA across the app. Phones: menu plus read-only views of plans, drafts, reviews and briefs.

Draft Graph
- Data sources: curated trait table, team-entered relations, community pool with votes, own match data, aggregated data from consenting beta users. No scraping.
- Priority: synergy, then counters, then playstyle fit, then power-spike timing.
- Lives in the centre of the draft page as a switchable view. Ban suggestions in Release 2. Trait table edited by the maintainer in Release 1, community proposals in Release 2.

Community loop
- Bug reporting: in-app widget plus a public bug board. Micro-surveys at most one per user per week, context-triggered. Everyone votes, one vote each.
- Transparency ledger shows support received, cost breakdown (hosting, domain, AI tokens), maintainer share, per-month history, named supporters only on opt-in.
- Money policy: "costs first (hosting, domain, AI tokens), then the remainder to the maintainer". Wording "support" / "contribution to running costs", never "donation" or "Spende"; state that no donation receipt is issued. One external channel (Ko-fi or GitHub Sponsors), nothing in the app. No supporter perks of any kind.
- Legal: the agent prepares a written question to Riot Developer Relations and a summary for the owner's tax adviser (not legal or tax advice); the owner confirms before a support page goes live. Riot's production key requires a live site with terms, privacy policy, a riot.txt file and a working prototype, so the support page follows registration.
- Community home: Discord server and in-app; a Discord bot later.

Scope
- Feature ratings: Match Day flow (R1), shareable read-only links and player cards (R1), templates and keyboard-first drafting (R1, partly), series and Fearless draft (R2), opponent memory (R2), practice loop (R2), patch awareness (R2), VOD bookmarks (R2), prediction check (R2), Discord integration (R2/R3).
- Pages: Team builder parked, Analytics parked (merged into stats and review), Tree drafter kept but simplified, "Live" hub removed, Game Day Brief removed and rebuilt as Match Day with real data, standalone solo dashboard removed (solo becomes the personal layer: profile, Riot link, match history, personal learnings feeding the draft), Pandemonium structural theme removed.
- The Draft Graph ships in Release 1.

Sand Table
- Name: Sand Table. Foundation in Release 1 (store per-minute frames and positioned events; reserve map, token, timeline and overlay components in the design system). Full v1 as the Release 2 flagship: Board v1 with live cursors from the start, Replay v1 with overlays in this priority: spread and sync, objective setups, fights and picks, timers and tempo, vision rhythm, territory, economy, roams and jungle; the moments rail; annotate into review, learning or action item; VOD bookmarks; plan-versus-reality overlay pulled into Release 2.
- Games: start with API-available games (ranked, flex, normal, Clash); scrims by manual entry now, Riot Sign-On integration once production approval allows. 60-second frames accepted, with estimated paths shown as uncertain and exact events as anchors. The desktop Replay Helper is planned as R4 only after a written policy check. Sand Table is pillar P4.

Ways of working
- A short note after every milestone, plus strong gates at M2 (design direction and primitives), M4 (Draft Graph), M8 (launch).
- Work lands on the designated session branch; at each gate a pull request into main is opened. The agent merges after the owner's go; the owner deploys at first.
- Demos as pages with screenshots; a deployment preview from M8. Budget guardrails: none stated.
- Hosting: a new small VM with Docker Compose, Caddy and nightly backups to object storage (30-day retention). Domain: open. The owner holds the Riot key, domain and hosting; the agent uses environment variables only. The Riot production key application is prepared at M8 and filed by the owner.

## 5. Milestones

| # | Milestone | Size | Gate | Contents |
|---|---|---|---|---|
| M0 | Reset and safety net | S | light | Stable or pinned toolchain, SessionStart hook, clippy and fmt green, CI runs the e2e smoke suite, docs reset (MISSION, ARCHITECTURE, DESIGN, CHANGELOG; old planning archived under a tag), fake data and dead links removed, stray CSS and the login/closed-beta layout bug fixed, Riot disclaimer in the footer, Pandemonium toggle removed. |
| M1 | Security hardening | L | light | Authorization layer and every server function fixed, injection fixed, server-side invite gate, session expiry and purge, Secure cookie via config, rate limiting, password hashing off the async runtime, generic auth errors, cross-tenant tests, security review. |
| M2 | Design system foundation | M | STRONG | Three visual directions as mockups, owner picks; DESIGN.md; token pass (type scale, domain tokens, AA contrast, body font applied, radius, shadow, layers, motion); about 25 primitives with typed variants in three approved batches; gallery page; lint gates; one theme; map/token/timeline/overlay components reserved. |
| M3 | Strategy pages onto the system | L | medium | Draft (rebuilt shell), tree drafter, champion pool, game plan, post-game, opponents, action items; keyboard and accessibility; e2e for core flows. |
| M4 | Draft Graph v1 | L | STRONG | Trait table, relation model, team notes, force-directed view, comp profile, explanations, inside the draft page; e2e; performance budget. |
| M5 | Remaining pages, navigation, mobile | M | light | Home, team dashboard (real data only), roster, profile, stats, match detail, personal layer, legal pages, Match Day flow, shareable links and player cards; navigation fixes; phone menu and read-only views. |
| M6 | Data layer and integrations | M | light | Migrations, indexes and unique constraints, record links, transactions, shared Riot client with region, live Data Dragon version and champion keys, learnings bug, seed tool, patch-diff detection, Sand Table frame storage. |
| M7 | Community and transparency loop | M | medium | Bug reports v2, micro-surveys, what-next board and changelog, transparency ledger and admin form, support link, admin role. |
| M8 | Ship-ready and launch | M | STRONG | Container image and compose, typed config, health check, graceful shutdown, backups and restore drill, runbook, real Impressum and privacy page, account deletion and export, Riot production key application, launch checklist, first teams invited. |

Order: M0 → M1 strictly first. M2 before M3, M4 and M5. M4 after M3. M6 and M7 interleave after M1. M8 last. Release 2 opens with the Sand Table (Board v1 with live cursors, Replay v1, plan-versus-reality).

Verification cycle per milestone: compile both targets, unit tests, clippy, fmt; e2e suite; browser walkthrough with screenshots at desktop and phone width; code review by a separate agent; a demo page for the owner; the owner's gate where marked.

## 6. Definition of done for Release 1

1. Secure: authorization everywhere, invite gate, expiring sessions, rate limits, injection fixed, cross-tenant tests.
2. One design system: tokens, at least 25 primitives, gallery, lint gates, all pages migrated, one theme, AA contrast, keyboard-operable draft and dialogs, phone menu.
3. Honest: zero invented data, no dead controls, every empty state says what will appear.
4. Draft Graph v1 shipped and explainable.
5. Community loop live: bug status, surveys, what-next board, changelog, transparency page.
6. Tested: core journey end-to-end in CI, hydration gate, cross-tenant tests.
7. Solid data: migrations, indexes, one shared Riot client, live Data Dragon version, Sand Table frames stored.
8. Shippable: container, config, health check, shutdown, backups, runbook, legal pages, disclaimer, account deletion and export.
9. Documented: MISSION, ARCHITECTURE, DESIGN, CHANGELOG, SessionStart hook, an accurate CLAUDE.md.

## 7. Rules of engagement

Follow strictly: security requirements; legal and compliance content and the money policy; branding once chosen; the "never" list; privacy (no scraping; Riot API only within its terms); scope cuts the owner decided; the milestone order and gates.

Creative freedom (the agent decides and shows; the owner can veto at gates): visual design within the chosen direction; component APIs and naming; the Draft Graph's and Sand Table's models, visuals and interactions; micro-copy and empty states; survey wording (owner approves before it reaches users); information architecture details; test strategy; internal refactors.

Ask first: anything that changes the Release 1 goal; deleting user-facing features; anything touching money or legal wording; adding external services or costs; merging to main or deploying.
