# TODOs

## Open

### Priority 1 — Bugs & Blockers

#### Riot API (401 Unauthorized)
- [ ] **Riot API key expired/invalid**: Profile linking (`Lylaend#EUW`) and Stats sync both return 401. The key in `.env` needs to be refreshed — Riot development keys expire every 24h. Get a new key from [developer.riotgames.com](https://developer.riotgames.com). Once fixed, re-test:
  - Profile → Link Account → should succeed
  - Stats → Sync Matches → should pull match history
  - Dashboard → "Riot account not linked" notice should disappear after linking

#### CSS / Build
- [ ] **Tailwind v4 `@import "tailwindcss"` 404**: The `input.css` starts with `@import "tailwindcss"` (Tailwind v4 build-time syntax). The raw import leaks through to browser CSS, causing a 404 for `/pkg/tailwindcss` on every page load. Harmless but noisy in console. Fix: ensure `tailwindcss` standalone binary is present and `cargo-leptos` processes the import.

#### Team Dashboard
- [ ] **"Available players" not visible**: The bench section shows players with `roster_type = 'sub'`. If members joined but haven't been accepted via join requests, they won't appear. Verify: accept join requests first, then check bench. May also be a display issue if all members are assigned to starter/coach slots already.

---

### Priority 2 — High Value UX

#### Draft
- [x] **Per-pick rationale comments**: Click a pick slot to annotate with rationale; shown truncated on board, persisted via auto-save.
- [x] **Composition identity tags**: Toggle buttons for teamfight, split-push, poke, pick, scaling, early-game, protect-the-carry. Filter saved drafts by tag.
- [x] **Win condition notes**: "How We Win" and "Watch Out For" textareas per draft, saved alongside draft data.
- [ ] **Drag-and-drop picks/bans**: Champions in draft slots should be draggable between slots (swap on drop).

#### Tree Drafter
- [ ] **Branch edge redesign**: Edges should show the branching reason — a single champion icon (round, full color if open; greyed out if banned) with a small label. Node configuration (label + notes) sets the champion and ban/open state displayed on the inbound edge. Significant UX rework.
- [ ] **Node-level annotations**: Rich text notes per node explaining branching rationale ("if they pick Azir, we flex mid Tristana because...").
- [ ] **Conditional edge labels**: Text labels on edges ("enemy picks engage support", "enemy bans our ADC pool") making the tree self-documenting.
- [ ] **Priority branch markers**: Flag branches as "preferred" or "backup" with visual indicators (color, thickness, star).
- [ ] **Drag-and-drop picks/bans within node editor**: Same as draft — drag champions between slots.
- [ ] **Collapse/expand subtrees**: For large trees, collapse branches not being actively worked on.

#### Champion Pool
- [ ] **Drag-and-drop between tiers**: Champions draggable from one tier card to another. On drop, call tier-update server function.
- [ ] **Left-side champion grid**: Show all available champions in a grid with search bar. Drag directly from grid into any tier.
- [ ] **Larger champion icons**: Increase icon size to ~48–56 px so the image fills the full card height.

#### Team Section
- [ ] **Drag-and-drop roster management**: Players in bench/coach/starter slots draggable between slots. Show role selector on drop into starter slot.

#### Team Builder
- [ ] **Placeholder page**: `/team-builder` shows placeholder text — needs actual composition builder implementation or should be hidden from nav until ready.

---

### Priority 3 — Medium Features

#### Champion Pool — Structured Notes
Replace single textarea with structured fields per champion (informed by research — see `.planning/research/competitive_league_research.md` R3):
- [ ] **Comfort level** (1-5 scale) alongside existing tier
- [ ] **Matchup notes**: Per-opponent entries with difficulty rating, lane strategy, items to adjust, personal notes
- [ ] **Lessons learned journal**: Timestamped entries after games ("realized I need to save W for disengage")
- [ ] **Power spikes**: Repeatable entries (timing + description, e.g. "Level 6: ult enables tower dives")
- [ ] **Combos**: Named sequences with notes (e.g. "Basic trade: Q > AA > W > AA — use in short trades")
- [ ] **Teamfight role & positioning notes**

#### Draft Tools
- [ ] **Fearless draft mode**: Track previously picked/banned champions across a series (BO3/BO5), gray them out in subsequent games
- [ ] **Series-level draft grouping**: Group multiple drafts into a "series" with a single shareable view
- [ ] **Matchup context inline**: Show basic counter/synergy info when a champion is picked
- [x] **Ban priority list**: Collapsible panel with ranked champion list, add/remove entries with reason, save/cancel edit mode
- [ ] **Saved counter-picks**: Per champion, "our go-to answers" as a quick-reference overlay during draft phase

#### Stats
- [ ] **Per-champion stats breakdown**: Top 5 played champions per player with win rate, avg KDA, avg CS/min
- [ ] **Win/loss streak indicator**: Visual W/L streak on the match list
- [ ] **Aggregate graphs**: Line chart of team win rate over time; bar chart of most played champions together

#### Game Plan
- [ ] **Auto-save**: Same 2 s debounce + 30 s hard save as draft and tree sections
- [ ] **Template from champion pool**: Pre-fill role strategy fields from selected champion's notes

#### Scouting & Opponent Prep
- [ ] **Opponent team profile**: Fill in enemy roster (5 summoner names), auto-fetch their recent champion picks from Riot API. Link to game plan.
- [ ] **Champion matchup notes integration**: Surface champion pool matchup notes during draft when facing specific opponents
- [ ] **Patch tracking**: Record which patch each game was played on, show patch label on match cards, filter by patch

---

### Priority 4 — New Features

#### Champion Pool
- [ ] **Initial pool from Riot API**: Fetch champion mastery/recent ranked history on first link, suggest initial pool with confirmation dialog
- [ ] **Per-champion stats inline**: Show win rate, KDA, games played on each champion card
- [ ] **Role filter tab "All"**: Tab showing entire pool across all roles
- [ ] **Tier description tooltips**: Small (?) next to each tier name explaining its meaning

#### Team
- [ ] **Invite link**: Generate shareable UUID join link
- [ ] **PrimeLeague manual import**: No public API exists (IP-whitelisted internal API only). Build CSV/JSON roster import instead. See research: `.planning/research/competitive_league_research.md` R1

#### Tree Drafter
- [ ] **Opponent scouting integration**: Link tree to opponent team, pre-populate likely picks from their champion pool data
- [ ] **Tree comparison view**: Side-by-side two trees for different opponents to spot overlapping flex picks
- [ ] **Snapshot/version history**: Save tree versions before/after scrims

#### Practice & Scheduling
- [ ] **Scrim log**: Log scrimmage results (opponent, score, date, format) linked to drafts and post-game reviews
- [ ] **Pre-game checklist**: Customisable checklist before matches ("Reviewed ban pattern", "Agreed on win condition")
- [ ] **Practice priority queue**: Mark 1-3 champions per role as "focus this week", shown on dashboard

#### Draft Tools
- [ ] **Draft simulator**: Play both sides against yourself for practice
- [ ] **Side preference history**: Record which side was chosen and why per match

#### Communication
- [ ] **VoD link on post-game**: Attach YouTube/Twitch timestamp links with per-timestamp notes
- [ ] **Team announcements**: Pinboard on dashboard for leader to post short text announcements

---

### Priority 5 — Nice to Have
- [ ] **Draft versioning**: Last 5 auto-save snapshots per draft for rollback
- [ ] **Draft keyboard shortcuts**: `1`-`5` select pick slots, `b` toggles ban mode
- [ ] **Export draft as image**: Screenshot draft board as PNG for Discord sharing
- [ ] **Export tree as PDF/image**: For sharing scouting reports
- [ ] **Match timeline mini-graph**: Gold/XP lead chart per match (Riot timeline endpoint)
- [ ] **Post-game link from game plan**: Prompt to create linked post-game review after completing a game plan

---

## Research Findings (2026-03-12)

Full research document: `.planning/research/competitive_league_research.md`

### PrimeLeague Integration — Skip
No public API. The primeleague.gg internal API requires IP whitelisting. Freaks 4U Gaming (operator) filed for insolvency in 2024 (license renewed through 2027). **Recommendation**: Skip automated integration, build manual CSV/JSON roster import instead. Liquipedia scraping is a viable fallback if demand materializes.

### Draft Tool Landscape
Analyzed 10 tools (ProComps, DraftGap, iTero, Drafter.lol, ScoutAhead, etc.). Key gaps our app can fill: per-pick rationale annotations (most requested coaching feature), composition identity tags, and Fearless draft mode. See Priority 2-3 items above.

### Champion Learning Notes
Proposed structured `ChampionNote` schema with sub-records for power spikes, combos, matchups (with personal difficulty rating), teamfight role, synergies, and timestamped lessons journal. MVP: comfort level, matchup notes, lessons learned. Full schema in research doc.

---

## Recently Completed (Section 18 — Triage & Polish)

### UI Polish
- [x] **First pick toggle alignment**: Changed red-side indicator from `left-[1.375rem]` to `right-0.5` for symmetric padding
- [x] **Ban icon red bars removed**: Deleted red strikethrough overlay on draft board ban slots; grayscale + opacity-50 is sufficient
- [x] **Ban icons greyscale in tree graph**: Replaced red accent border + cross overlay with muted border + SVG grayscale filter + 50% opacity for ban icons on edges
- [x] **Tree graph canvas enlarged**: Increased `NODE_W` 150→180, `NODE_H` 36→42, `LEVEL_H` 100→120, `H_GAP` 16→24, `ICON_SIZE` 22→26. Container max-height 70vh→85vh. Proportional text/badge adjustments.
- [x] **Profile username pencil edit**: Replaced always-visible form with display + pencil icon toggle. Click pencil → inline edit with Save/Cancel.
- [x] **Recent games on dashboard**: 3 most recent matches shown as condensed cards (champion icon, KDA, W/L badge, date) with "View all stats →" link. Empty state for no matches.

### Triage Resolution
- [x] **Draft saving**: Already fixed in Section 17 (rule 44 + empty name guard)
- [x] **Tree drafter freeze**: Already fixed in Section 17 (eager capture + suppress_autosave)
- [x] **Riot API 401**: API key issue — needs fresh key from Riot developer portal (not a code bug)
- [x] **PrimeLeague**: Researched — no public API, skip automated integration

---

## Completed (Previous Sections)

### Section 17 — Team Polish, Interactive Tree Graph, Stats Overhaul
- [x] UI freeze after branching (auto-save eager capture)
- [x] Node switch bug (suppress_autosave + cancel timer)
- [x] Tree graph too many icons on edge (MAX_ICONS = 3)
- [x] Interactive tree graph (click-to-select, accent glow)
- [x] Auto-save for drafts and tree nodes (2s debounce)
- [x] Click-to-select-then-delete pattern (draft board)
- [x] Role icons deduplicated, team rename pencil modal
- [x] Protected page redirects, auth-aware nav
- [x] Stats: queue selector, OP.GG layout, expandable details, partial-team matches
- [x] Rule 44 violations fixed across 9 server functions

### Section 10 — WASM Panic Hardening
- [x] Safe `.unwrap()` replacements in nav, drag-and-drop handlers

### Section 9 — Tree Graph Visualization
- [x] SVG tree component, layout algorithm, champion icons on edges, clickable nodes

### Section 8 — Theming System
- [x] CSS custom properties, dark/light toggle, accent color picker, 285 token replacements

### Section 7 — Bug Fixes & Feature Enhancements
- [x] Nav dropdowns, game plan save fix, tree drafter UX, champion autocomplete, champion pool page

### Sections 1-6 — Core Features
- [x] Teams, join requests, roster management, draft tree system, stats & Riot API, game plan, postgame analysis, dashboard integration, landing page, theming

---

## Known Bugs

- **Tailwind CSS 404**: Every page loads `/pkg/tailwindcss` → 404. Harmless, filtered in e2e tests.

## E2E Test Status (2026-03-11)
- 21/21 passing
- Auth fixture registers then logs in
- Tailwind 404 filtered from error assertions
