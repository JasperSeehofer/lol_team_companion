# TODOs

## Completed

### Section 1 – Teams & Existing Feature Gaps
- [x] Join request system: players request to join, leader accepts/declines
- [x] Nav badge: red dot on Team link shows pending request count (leaders only)
- [x] Substitute roster: members land on the bench after joining
- [x] Starter slots: 5 role slots (Top/Jungle/Mid/Bot/Support) with drag-and-drop assignment
- [x] Leader can kick members and edit team name/region
- [x] Role dropdown per bench member

### Section 2 – Draft Tree System (`/tree-drafter`)
- [x] New page at /tree-drafter (do NOT touch existing /draft)
- [x] Tree data model: DraftTree + DraftTreeNode (parent/child)
- [x] Create tree, add branches (child nodes)
- [x] Tree visualisation: indented list with expand/collapse
- [x] Node editor: full draft board per node + notes
- [x] Live game navigator: step-by-step branch selection
- [x] Improvisation mode: create branch mid-game, tag as improvised

### Section 3 – Stats & League API
- [x] Pull match history from Riot API (manual refresh only)
- [x] Filter for all-5-roster-player games
- [x] Stats dashboard with date/opponent filters
- [x] Flag clearly if RIOT_API_KEY is missing

### Section 4 – Game Plan System
- [x] Create plans linked to a specific matchup (your 5 champs vs enemy 5)
- [x] Macro strategy section (team-wide)
- [x] 5 role-specific sections
- [x] Link to a draft
- [x] Template-based auto-generation

### Earlier Work
- [x] Saved drafts display bans left/right of picks, phase groups separated
- [x] Draft rating (S+ to D tier picker)
- [x] Team selection on draft form
- [x] Sliding first-pick toggle with colour animation
- [x] Role filter icons via Community Dragon CDN
- [x] Champion pool on profile page (per role, add/remove)
- [x] Link Riot account (name#tag → PUUID)
- [x] Create/join team from roster page

---

## Next Up

### Section 5 – Postgame Analysis
- [ ] Link to actual match from stats
- [ ] Link to original game plan and draft
- [ ] Structured feedback fields
- [ ] Open-ended notes
- [ ] Pattern analysis

### Section 6 – Full Integration & UI Polish
- [ ] Consistent header with notifications dropdown
- [ ] Dashboard showing teams, recent drafts, alerts, stats summary
- [ ] Dark mode toggle
- [ ] Error handling and graceful API failures

---

## Known Bugs
*(none currently tracked)*
