# Phase 12: Solo Mode Infrastructure - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Solo players can use the app without a team. This phase delivers: a team/solo mode toggle persisted in the DB, multi-region Riot account linking, solo queue match history sync, queue type filtering, a ranked badge display, and a personal solo dashboard with goal placeholders.

Requirements covered: SOLO-01, SOLO-02, SOLO-03, SOLO-04, SOLO-05, RANK-01, RANK-03

</domain>

<decisions>
## Implementation Decisions

### Mode Toggle & Navigation
- **D-01:** Mode toggle lives in the **nav bar** — persistent, one-click switch between team and solo mode
- **D-02:** In solo mode, all nav links remain visible and clickable. Team-only pages (Team Dashboard, Roster, Opponents) show a "Switch to team mode to use this feature" gate message on click — not hidden or greyed out
- **D-03:** New users default to **solo mode** and land on the solo dashboard after login/register. They switch to team mode when they create/join a team
- **D-04:** Mode is persisted in the DB `user` record (per STATE.md decision), not a WASM signal — survives hard navigation and login/logout

### Region Support
- **D-05:** **Multi-region from day one** — support all major Riot API regions (EUW, NA, KR, EUNE, OCE, BR, LAN, LAS, TR, RU, JP, PH, SG, TH, TW, VN). Parameterize all hardcoded `EUROPE`/`EUW1` routes in `riot.rs`
- **D-06:** Region selection happens on the **profile page during Riot account linking** — dropdown appears alongside game_name/tag_line inputs. Region stored on user record

### Solo Dashboard
- **D-07:** **Single column, stacked sections** layout — top: ranked badge, middle: recent matches, bottom: goal placeholders
- **D-08:** Ranked badge shows **tier icon image + text + LP** (e.g. [Gold emblem] Gold II - 47 LP). Tier icons from Riot ranked emblems (Data Dragon or static assets)
- **D-09:** Dashboard shows **solo/duo queue rank only** — flex rank not displayed (can be added later)
- **D-10:** Match sync is **manual button + auto on page load** — auto-sync when visiting solo dashboard if last sync > 10 minutes ago, plus a manual "Sync Matches" button
- **D-11:** Goal status section shows **greyed-out placeholder cards** with "Coming in Phase 15" label — 2-3 cards teasing Rank Target, CS/min, Deaths goals

### Queue Filtering
- **D-12:** Queue filter uses a **dropdown select** (All / Solo/Duo / Flex) — compact, two clicks to change
- **D-13:** Queue filter appears on **both the stats page and solo dashboard** — users can filter anywhere they see matches

### Claude's Discretion
- Auto-sync staleness threshold (10 min suggested, Claude can adjust based on Riot API rate limits)
- Exact region-to-route mapping implementation (lookup table, enum, etc.)
- How the "switch to team mode" gate message is styled on team-only pages
- Ranked emblem image source (Data Dragon CDN vs. bundled static assets)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Riot API & Regions
- `src/server/riot.rs` — All Riot API calls with hardcoded `EUROPE`/`EUW1` routes that need parameterization
- `src/server/data_dragon.rs` — Champion data CDN; may also source ranked emblems

### Data Model
- `schema.surql` — Current DB schema; needs `mode` and `region` fields on user table, `synced_by` on match table, ranked snapshot table
- `src/models/user.rs` — User model structs (AppUser, PublicUser, TeamMember, UserProfile)
- `src/server/auth.rs` — DbUser and AppUser definitions with auth fields

### Navigation & Routing
- `src/components/nav.rs` — Top nav bar; mode toggle will be added here
- `src/app.rs` — Route definitions; solo dashboard route needs adding
- `src/pages/auth/login.rs` — Login redirect (currently hardcoded to `/team/dashboard`)
- `src/pages/auth/register.rs` — Registration redirect (currently hardcoded to `/team/dashboard`)

### Existing Dashboard & Stats
- `src/pages/team/dashboard.rs` — Team dashboard pattern to reference (not reuse) for solo dashboard
- `src/pages/stats.rs` — Match history display; queue filter will be added here
- `src/pages/profile.rs` — Riot account linking UI; region dropdown will be added here

### Requirements
- `.planning/REQUIREMENTS.md` — SOLO-01 through SOLO-05, RANK-01, RANK-03

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ErrorBanner` component (`src/components/ui.rs`) — for gate messages on team-only pages in solo mode
- `StatCard` component (`src/components/stat_card.rs`) — possible use for ranked display cards
- Theme toggle in nav (`src/components/theme_toggle.rs`) — reference pattern for adding mode toggle to nav
- Protected page template pattern (auth check + redirect) — all protected pages follow same pattern

### Established Patterns
- **Server fn + Resource + Suspense** — standard data fetching pattern for dashboard data
- **Debounced auto-save** with `RwSignal<Option<i32>>` timer — exists in draft/tree pages
- **`get_team_dashboard()` batched query** — reference for building `get_solo_dashboard()` batched query
- **DbStruct → AppStruct conversion** — all DB types follow this pattern

### Integration Points
- `src/server/db.rs` — New query functions for solo sync, ranked snapshots, solo dashboard summary
- `src/server/riot.rs` — Parameterize region on all API calls; add ranked endpoint calls
- `src/main.rs` — No changes expected (schema auto-applied from `schema.surql`)
- Nav bar — Mode toggle added alongside existing theme toggle
- Login/register redirects — Change from `/team/dashboard` to mode-aware routing (solo dashboard if solo mode, team dashboard if team mode)

</code_context>

<specifics>
## Specific Ideas

- Match rows on solo dashboard should show LP delta per game (e.g. +18 LP, -15 LP) when available from ranked snapshots
- Goal placeholder cards should name the specific goals (Rank Target, CS/min, Deaths) to tease Phase 15
- Auto-sync on page load with staleness check avoids hammering the Riot API while keeping data fresh

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 12-solo-mode-infrastructure*
*Context gathered: 2026-03-25*
