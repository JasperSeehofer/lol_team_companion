# TODOs

## Open

### Priority 1 — Critical Bugs (breaks core flow)

#### Tree Drafter
- [ ] **UI freeze after branching**: After creating a branch and clicking a champion for the next pick, the pick does not update and all subsequent clicks become unresponsive. Root cause: likely stale signal capture when `selected_node` changes while a closure holds an old reference. Investigate `selected_node` signal and the champion picker `on:click` in `tree_drafter.rs`. Key fix: switching nodes must fully re-initialize draft slot signals (consider keying the editor on `node_id`).
- [ ] **Node switch bug**: Clicking node A then node B leaves the editor broken (stale closures). Full signal teardown on node change needed.
- [ ] **Tree graph — too many icons on single edge**: SVG tree stacks multiple champion icons on one edge. Each pick/ban diff should show one icon, spaced or summarized. Fix edge rendering in `tree_graph.rs`.
- [ ] **Cannot easily switch between drafts**: `selected_node` and `nodes_resource` go out of sync when switching trees. Ensure `nodes_resource` is keyed on `selected_tree_id`.

#### Team Section
- [x] **Accept join request — member does not appear**: Fixed by surfacing the error from `handle_join_request` and calling `dashboard.refetch()` + `requests.refetch()` on success.
- [x] **Team owner missing from roster/bench**: Fixed `create_team()` in `db.rs` — owner now inserted as `team_member` with `role = 'unassigned', roster_type = 'sub'`.

#### Draft & Game Plan
- [ ] **Draft not saving / not appearing in saved list**: Verify `save_draft()` / `update_draft()` return a proper ID, that the client receives it, and `drafts.refetch()` is actually triggered. Add explicit error display if save fails. Same issue observed from game plan's draft picker — ensure draft selected there is persisted, not only held in client state.

---

### Priority 2 — High Value UX

#### Tree Drafter
- [x] **Enter key creates tree**: Pressing Enter in tree name or opponent inputs triggers Create Tree.
- [x] **Root node name = tree name**: Root node is now automatically labelled with the tree name.
- [x] **Click-to-select slot (not delete)**: First click highlights slot with red border + × badge; clicking × removes the champion. Click elsewhere to deselect.
- [x] **Auto-save node on edit**: Debounced 2 s after last change. Shows "✓ Saved" / "● Unsaved changes" indicator.
- [ ] **Drag-and-drop picks/bans within node editor**: Champions in draft slots should be draggable between slots (swap on drop).
- [ ] **Interactive tree graph**: Clicking a node in the SVG tree should select it in the editor (two-way sync). Highlight selected node.

#### Draft
- [x] **Auto-save**: When a draft has a name and is an existing saved draft, auto-saves debounced 2 s after any pick/ban/comment change. Shows "✓ Saved" / "● Unsaved changes" status in header.
- [ ] **Drag-and-drop picks/bans**: Same as tree drafter — drag champions between slots.

#### Team Section
- [ ] **Role icons — remove duplicate text**: Slot rendering shows text ("Top", "Jungle", …) AND a broken partial-text icon. Show only the SVG role icon, remove the text label.
- [ ] **Remove "Link Riot account" from team dashboard**: Belongs in Profile. Replace with a notice + link: "Riot account not linked — link it in your profile" if not yet connected.
- [ ] **Team rename → pencil icon modal**: Remove the prominent rename form. Add a small pencil (✏) icon next to the team name that opens a modal to change name and region.

#### Champion Pool
- [x] **Click-to-add champion**: Selecting a champion in the autocomplete dropdown now immediately adds it to the pool.
- [x] **Tier list visible from start**: All tiers always render; empty tiers show "No champions yet" placeholder.

---

### Priority 3 — UI Polish & Medium Features

#### Champion Pool
- [ ] **Drag-and-drop between tiers**: Champions are draggable from one tier card to another. On drop, call the tier-update server function.
- [ ] **Left-side champion grid**: Show all available champions in a grid with a search bar on the left side. Drag directly from this grid into any tier.
- [ ] **Larger champion icons**: Increase icon size to ~48–56 px so the image fills the full card height.
- [ ] **Expanded notes section**: Replace single text area with structured fields per champion entry:
  - Short summary / strengths
  - Key mechanics & combos to learn
  - Important matchup-specific warnings
  - Win conditions
  - Common mistakes / things to watch out for
  - Learning resources (free-text link/notes field)

#### Team Section
- [ ] **Drag-and-drop roster management**: Players in bench/coach/starter slots should be draggable between slots. Show role selector on drop into a starter slot.

#### Stats
- [ ] **Show partial-team matches**: Display all matches where ≥ 2 linked team members played together, not only full-roster games. Add a "minimum players together" dropdown (2 / 3 / 4 / 5).
- [ ] **Solo Queue sync**: Add a queue type selector to `sync_team_stats()`:
  - 420 — Ranked Solo/Duo
  - 440 — Ranked Flex (current default)
  - 0 — All queues (Normal)
- [ ] **Standard match history layout**: Align with established OP.GG / League client conventions:
  - Left: champion icon, spells, runes
  - Centre: KDA, CS, vision score, items
  - Right: W/L badge, match duration, date
  - Row tint: blue for win, red for loss
- [ ] **Match detail expand**: Click a match row to expand into a full scoreboard (all 10 players, damage, vision, items).

---

### Priority 4 — New Features & Suggestions

#### Champion Pool
- [ ] **Initial pool from Riot API**: On first link or on-demand button, fetch champion mastery or recent ranked match history (last 100–200 matches) and suggest an initial pool:
  - Mastery ≥ 5 → suggest "comfort"; ≥ 4 → "practicing"; ≥ 3 → "to_practice"
  - ≥ 10 ranked games last season → suggest "match_ready"
  - Present as a confirmation dialog — user accepts or rejects each suggestion individually
- [ ] **Per-champion stats inline**: Show win rate, KDA, and games played (from synced match history) on each champion card in the pool
- [ ] **Role filter tab "All"**: A tab that shows the entire pool across all roles in one view
- [ ] **Tier description tooltips**: Small (?) next to each tier name explaining its meaning

#### Team
- [ ] **Invite link**: Generate a shareable UUID join link so the leader can invite players without them searching
- [ ] **Leave team button**: Non-owner members can leave the team

#### Stats
- [ ] **Per-champion stats breakdown**: Top 5 played champions per player with win rate, avg KDA, avg CS/min
- [ ] **Win/loss streak indicator**: Visual W/L streak on the match list
- [ ] **Aggregate graphs**: Line chart of team win rate over time; bar chart of most played champions together

#### Tree Drafter
- [ ] **Collapse/expand subtrees**: Collapse branches in the tree view to reduce clutter on deep trees
- [ ] **Branch labels on edges**: Show a short decision label (e.g., "Enemy picks Yasuo") on the connecting edge rather than just champion icons

#### Game Plan
- [ ] **Auto-save**: Same 2 s debounce + 30 s hard save as draft and tree sections
- [ ] **Template from champion pool**: Pre-fill role strategy fields from the selected champion's notes in the pool

---

### Priority 5 — Nice to Have
- [ ] **Draft versioning**: Keep last 5 auto-save snapshots per draft for rollback
- [ ] **Draft keyboard shortcuts**: `1`–`5` select pick slots, `b` toggles ban mode
- [ ] **Export draft as image**: Screenshot draft board as PNG for Discord sharing
- [ ] **Export tree as PDF/image**: For sharing scouting reports
- [ ] **Match timeline mini-graph**: Small gold/XP lead chart per match (if Riot timeline endpoint is used)
- [ ] **Post-game link from game plan**: After completing a game plan, prompt to create a linked post-game review

---

## Completed

### Section 10 – WASM Panic Hardening
- [x] Replaced `.unwrap()` in nav Escape key listener with safe `if let Some(window)` pattern
- [x] Replaced `.unwrap()` in drag-and-drop handlers (dashboard) with `let Some(dt) = ... else { return }`
- [x] Fixed potential WASM runtime crash that froze all subsequent clicks and navigation

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
