# TODOs
## Open
### General
- [ ] it seems that often if a button click did not work, all subsequent clicks or links to different routes dont do anything. Please investigate and solve this issue in general.
## Completed

### Section 9 – Tree Graph Visualization
- [x] SVG-based tree graph component (`src/components/tree_graph.rs`)
- [x] Top-down layout algorithm with automatic node positioning
- [x] Champion icons on connection edges showing picks/bans diff between parent and child
- [x] Ban indicators (red border + cross overlay) vs pick indicators (green border)
- [x] List/graph view toggle in tree structure panel header
- [x] Graph panel auto-expands to fill width; list view stays as fixed sidebar
- [x] Clickable nodes to select for editing, hover + button to add branches

### Section 8 – Theming System
- [x] CSS custom property-based theme system with semantic color tokens
- [x] Dark/light mode toggle (moon/sun icon) with localStorage persistence
- [x] Anti-FOUC inline script in HTML head
- [x] Accent color picker (yellow, blue, purple, emerald, rose)
- [x] Replaced ~285 hardcoded color references with semantic tokens across all pages and components

### Section 7 – Bug Fixes & Feature Enhancements
- [x] Nav dropdowns close on outside click (transparent backdrop), Escape key, and link click
- [x] Removed duplicate "Team Settings" from profile menu
- [x] Fixed game plan save "Connection uninitialised" error (missing `.check()`)
- [x] Tree drafter: fixed Live Game button not activating immediately
- [x] Tree drafter: enlarged node +/x buttons
- [x] Notification dropdown: inline accept/decline for join requests
- [x] Team dashboard: coach role slots, leave team, leader badge
- [x] Drafts: blue/red side toggle, auto-populate game plan champions from draft
- [x] Champion autocomplete component with icons (game plan champion inputs)
- [x] Champion pool: standalone page (`/champion-pool`) with tiers (comfort, match ready, scrim ready, practicing, should be practiced) and notes
- [x] Profile: champion pool summary with link to full pool page
- [x] Tree drafter: "Branch from here" button to create branch from a selected draft position

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

### Section 5 – Postgame Analysis
- [x] Link to actual match from stats
- [x] Link to original game plan and draft
- [x] Structured feedback fields
- [x] Open-ended notes
- [x] Pattern analysis

### Earlier Work
- [x] Saved drafts display bans left/right of picks, phase groups separated
- [x] Draft rating (S+ to D tier picker)
- [x] Team selection on draft form
- [x] Sliding first-pick toggle with colour animation
- [x] Role filter icons via Community Dragon CDN
- [x] Champion pool on profile page (per role, add/remove)
- [x] Link Riot account (name#tag → PUUID)
- [x] Create/join team from roster page

### Section 6 – Full Integration & UI Polish
- [x] Dashboard: team summary, draft/plan/review counts, recent game stats, win rate
- [x] Landing page for unauthenticated users with CTA
- [x] Alert banners: pending join requests, no team, missing API key
- [x] Consistent header: sticky nav, backdrop blur, notifications dropdown, user avatar menu
- [x] Mobile-responsive nav with hamburger menu
- [x] Reusable ErrorBanner and StatusMessage UI components
- [x] Consistent error display across all pages (standardized to ErrorBanner)
- [x] Dark theme by default (app-wide `bg-gray-950`)

---

## Known Bugs
*(none currently tracked)*
