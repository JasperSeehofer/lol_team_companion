# Pitfalls Research

**Domain:** Adding solo player mode and match detail/timeline intelligence to an existing Leptos 0.8 / SurrealDB 3.x team companion app
**Researched:** 2026-03-25
**Confidence:** HIGH (all pitfalls verified against codebase or official Riot API documentation)

---

## Critical Pitfalls

Mistakes that cause data corruption, team-feature regressions, or hard-to-recover state.

---

### Pitfall 1: The `match` Table's `team_id` Assumption Breaks Solo Mode

**What goes wrong:** Every `match` record stored today has `team_id TYPE option<record<team>>` (schema.surql line 39). The `store_matches()` function in `db.rs` does not set `team_id` at all — it is always NULL. However all existing query paths in `get_team_match_stats()`, `get_dashboard_summary()`, and the win-condition analytics functions filter by `team_id = type::record('team', $team_key)` or assume matches belong to a team context. When solo mode is added and users sync personal stats without a team, matches are stored with no `team_id` — which is already the current behaviour — but new solo queries and existing team queries must never mix results.

**Why it happens:** The schema was designed for team use but the `team_id` field was left optional without a clear invariant. "NULL team_id = solo match" and "NULL team_id = match not yet assigned" are indistinguishable without a separate `mode` discriminator field.

**How to avoid:** Add a `context` field (`'team'` or `'solo'`) to the `match` table at schema design time, or add a `solo_owner` field of type `option<record<user>>`. Use this discriminator in all queries instead of relying on `team_id IS NULL`. The solo stats fetch function must explicitly set `solo_owner = type::record('user', $user_key)` and all team stat queries must explicitly filter `team_id != NONE`.

**Warning signs:** The team stats page starts showing solo-queue games from a user who has no team. Or: a solo user syncs games but the team stats view counts them. Check by verifying that `get_team_match_stats()` returns 0 rows for a freshly-joined team member who has personally synced matches.

**Phase to address:** Data model phase — must be resolved before any solo sync endpoint is written. The schema change is additive (`IF NOT EXISTS`) so it does not require data migration.

---

### Pitfall 2: Sequential Riot API Fetch Loop Exhausts Rate Limit When Syncing 20 Matches Per User

**What goes wrong:** The current `fetch_match_history()` in `riot.rs` does a serial for-loop: fetch list of match IDs → loop over each ID → call `get_match()` per ID. For 20 matches that is 21 API calls (1 list + 20 match fetches). The `riven` crate handles rate limits internally and will delay calls until the window resets, but a personal development key has a limit of 20 requests per second and 100 requests per 2 minutes (source: hextechdocs.dev). A single full sync for one user consumes 21% of the 2-minute window. If `sync_solo_stats` is user-triggered and the same user also triggers `sync_team_stats` in the same session, the combined 42-call burst will hit the 2-minute limit and block subsequent requests for 60–90 seconds.

Adding the match timeline endpoint (`get_timeline()`) multiplies this: each timeline call is an additional API call per match. Fetching timelines for 20 matches = 21 (match list) + 20 (match detail) + 20 (timeline) = 61 API calls — 61% of the 2-minute window in a single operation. If a second user syncs concurrently, the shared per-key limit is breached immediately.

**Why it happens:** Timeline data is not fetched today. Adding it to the same sequential loop is the natural first implementation. The rate limit impact is not obvious because `riven` silently queues rather than errors.

**How to avoid:**
- Never fetch timelines eagerly during a bulk sync. Only fetch a timeline on-demand when the user navigates to a specific match detail view.
- Cache timeline data in the DB after first fetch. The `match` table should have a `timeline_json TYPE option<string>` field. Check it before calling the API; if present, return the cached value.
- Keep the on-demand timeline fetch as a separate server function (`get_match_timeline`) so it does not block the bulk sync flow.
- The bulk sync (`sync_solo_stats`) should fetch only match summaries (not timeline), mirroring the current `fetch_match_history` pattern.

**Warning signs:** `riven` debug logs show `sleeping X ms for rate limit`. Server function responses take 30+ seconds. Users see "timeout" errors on the stats page.

**Phase to address:** Match sync and match detail phases. The "fetch match summary only" vs "fetch timeline on demand" split must be established in the sync phase before the detail view is built.

---

### Pitfall 3: Timeline JSON Payload Is ~1–3 MB Per Match — Storing Raw in SurrealDB Will Bloat the Database

**What goes wrong:** A League of Legends match timeline response from the Riot API contains per-minute frames for the entire game duration (typically 25–45 frames), each including position data, CS, gold, XP, and damage for all 10 participants, plus a complete event log (kills, objectives, item purchases, ward placements, etc.). Raw timeline JSON is approximately 80,000 LLM tokens (source: research.mdpi.com paper on LoL match data summarization), which translates to roughly 300–500 KB of JSON per match. Storing this verbatim in a SurrealDB `string` field is possible but means each timeline record consumes ~500 KB. For a user with 100 matches, that is ~50 MB of raw timeline blobs in the DB, plus SurrealDB's internal overhead for SCHEMALESS-adjacent storage.

SurrealDB 3.x with SurrealKV stores data on-disk. A 50 MB blob store for one user's timelines is manageable, but the application only needs a small subset of the timeline: objective timestamps (dragon, baron, tower falls), team gold differential over time, and key kill events. Storing the full payload to query 5% of it is wasteful.

**Why it happens:** The Riot API returns timeline as a single JSON blob. Storing it raw is the path of least resistance. Developers underestimate the data volume.

**How to avoid:**
- Do not store the raw timeline JSON in SurrealDB. Process it in Rust immediately after fetching: extract only the events and frames needed for the UI (objectives, gold diff snapshots, key kills) and store a compact derived struct.
- Define a `MatchTimeline` model with only the fields the UI needs: `Vec<ObjectiveEvent>`, `Vec<GoldDiffSnapshot>`, and optionally `Vec<KillEvent>`. Serialize this compact struct to JSON (`serde_json::to_string`) and store it in `timeline_json`. This reduces storage by 80–90%.
- If full raw data is needed in future, store it separately in a dedicated `match_timeline_raw` table with a clear note that it is a cache and may be purged.

**Warning signs:** SurrealDB data directory grows by >1 MB per match fetched. Queries against the `match` table slow down as records grow (SurrealDB scans all fields for schemafull tables).

**Phase to address:** Match detail view phase — the data model for timeline storage must be decided before the first timeline fetch is implemented.

---

### Pitfall 4: Solo Toggle Scopes Team-Only Features Silently Instead of Hiding Them

**What goes wrong:** Features like draft planning, game plan creation, team notes, and opponent scouting are only meaningful in a team context. When a solo/team toggle is added to the nav or dashboard, the naive implementation hides the toggle state as a local signal. If the toggle state is not propagated globally (e.g. via a context provider), individual page components that have their own team-check logic (`get_current_user` → `get_user_team_id`) will still render their team-specific content for solo users who happen to also have a team.

The reverse is also a pitfall: if a user is in solo mode and navigates directly to `/draft`, the draft page's existing guard (`get_user_team_id` returns `None → empty state`) correctly shows an empty state — but the user sees "Create a team" CTA instead of a "Switch to team mode" prompt. The UX is broken even though the code is technically correct.

**Why it happens:** The existing codebase has no concept of "current mode". All pages are implicitly team-mode. Adding a toggle without a shared mode signal means each page independently decides what to show.

**How to avoid:**
- Store the current mode (`'solo'` or `'team'`) as a persistent user preference in the DB (a `mode` field on the `user` table, defaulting to `'team'`). The mode is not a local signal — it must survive page reload.
- Expose the mode via a server function (`get_user_mode`) that is fetched once in the shell/root component and stored in a Leptos context (`provide_context`). All pages read from this context rather than each re-deriving from `get_user_team_id`.
- Team-only nav links (Draft, Game Plan, Opponents, Team Notes) are conditionally rendered based on this context signal. They do not render in solo mode — do not just grey them out.
- Empty-state messages on team-only pages should detect solo mode and show "Switch to team mode in the nav to access this feature" rather than the generic "Join or create a team" CTA.

**Warning signs:** A solo-mode user navigates to `/draft` and sees a "Create a team" button. A team user switches to solo mode and still sees the Drafts nav link. The toggle state is lost on page reload.

**Phase to address:** Solo mode toggle phase (first phase). This is a foundational architectural decision that all other solo features depend on.

---

### Pitfall 5: Schema Migration for New Solo Tables Breaks Existing `db.rs` Struct Names

**What goes wrong:** The project adds new SurrealDB tables for solo features: `solo_goal`, `personal_learning`, `lp_history`, `practice_item`. Each new table requires a `Db*` struct in `db.rs` for deserialization. The existing `db.rs` already has 4,000+ lines and name collisions are a real risk. A `DbGoal` struct could conflict with a future team goal feature. A `DbLearning` struct would shadow the existing `PostGameLearning` model.

Additionally, `schema.surql` is re-applied on startup via `include_str!`. Any syntax error in the new schema definitions will prevent the server from starting at all — with no migration rollback mechanism.

**Why it happens:** `db.rs` grows by accretion. New developers add structs at the bottom without checking for name collisions. Schema errors only surface at startup.

**How to avoid:**
- Prefix all solo-mode DB structs with `DbSolo*` (e.g. `DbSoloGoal`, `DbSoloLearning`, `DbLpHistory`). This makes it trivially obvious they are solo-context structs and prevents collision with future team features.
- Test every new `schema.surql` addition by running `cargo leptos watch` and verifying the server starts cleanly before writing any query code.
- Add the new tables to the integration test DB setup in `tests/common/mod.rs` so integration tests catch schema errors in CI before they reach startup.
- Use `DEFINE TABLE IF NOT EXISTS` and `DEFINE FIELD IF NOT EXISTS` for all new solo tables — they are new additions, not modifications.

**Warning signs:** Compile error "struct `DbGoal` is defined multiple times". Server fails to start with a SurrealDB schema parse error. Integration tests pass but the server crashes at startup.

**Phase to address:** Data model phase for each solo feature. Any schema addition must be tested by starting the server before proceeding.

---

### Pitfall 6: The `match` Table's `UNIQUE` Index on `match_id` Breaks Multi-User Solo Sync

**What goes wrong:** `schema.surql` defines `DEFINE INDEX IF NOT EXISTS match_id_unique ON match FIELDS match_id UNIQUE`. This means there can only be one `match` record per Riot match ID, regardless of how many users played in that match. The current team sync loop already handles this correctly (it reuses the existing `match` record if the `match_id` is already stored). However, when solo mode is added, a common case is: user A and user B both played in the same ranked game. User A syncs first, creating the `match` record. User B syncs second — the existing record is found and reused, then a `player_match` for user B is created. This works correctly.

The pitfall occurs if a developer tries to add per-user data directly to the `match` table (e.g. `SET solo_owner = $user`) when creating it. On the second user's sync, the `SELECT id FROM match WHERE match_id = $match_id` finds the existing record but does not update `solo_owner`. User B's match record points to user A's `solo_owner`. Any query filtered by `solo_owner` will miss user B's matches.

**Why it happens:** The natural instinct when adding solo context is to "tag" the match record. But the match record is shared across players.

**How to avoid:**
- Never store per-user solo context on the shared `match` table. The `player_match` table is the per-user record — solo context (queue type, LP change, personal goal associations) must go on `player_match`, not `match`.
- Add a `solo_context_json` or explicit typed fields (`lp_before`, `lp_after`, `personal_notes`) to the `player_match` table where per-user solo data belongs.
- Add a `solo_owner` or `context` discriminator only to `player_match`, not `match`.

**Warning signs:** User B syncs their stats and sees fewer matches than expected. Queries filtered by `solo_owner` on the `match` table return only user A's first-synced matches.

**Phase to address:** Data model phase for solo sync — the `player_match` extension must be designed before solo sync is implemented.

---

## Moderate Pitfalls

---

### Pitfall 7: The Region Is Hardcoded to `EUROPE` and `EUW1` — Solo Mode Users May Be on Other Servers

**What goes wrong:** Every Riot API call in `riot.rs` hardcodes `RegionalRoute::EUROPE` and `PlatformRoute::EUW1`. This was acceptable for a private team tool where the team plays on EUW. Solo mode opens the app to individual players who may be on NA, KR, EUNE, or other servers. A NA player who links their Riot account and tries to sync solo stats will get `Account not found` errors because the PUUID lookup and match history fetch both hit the wrong regional route.

**Why it happens:** Hardcoding the region was a pragmatic decision for the team use case. Solo mode implicitly expands the user base beyond a single region.

**How to avoid:**
- Add a `region` field to the `user` table (default: `'EUW'`) and a region selector to the Riot account linking UI.
- Map the stored region string to the correct `RegionalRoute` and `PlatformRoute` in `riot.rs`. For `match-v5` and account lookups, use `RegionalRoute` (EUROPE/AMERICAS/ASIA). For champion mastery, use `PlatformRoute` (EUW1/NA1/KR etc).
- Do not add region-specific logic inside server functions — centralize it in `riot.rs` so there is one place to update when Riot changes routing.

**Warning signs:** A non-EUW user links their Riot ID and sees "Account not found". Match sync returns 0 matches for a user who has recent games.

**Phase to address:** Riot account linking phase — add region selection before any solo sync is implemented.

---

### Pitfall 8: `sync_team_stats` Requires a Team — `sync_solo_stats` Must Not

**What goes wrong:** The existing `sync_team_stats` server function returns an explicit `Err` if the user has no team (lines 107–111 in `stats.rs`). This is correct for team sync. A solo sync function copied from this template will replicate the error-on-no-team behaviour. But for solo mode, no team is the expected and valid state. Returning `Err("You need to join a team first")` from a solo sync function would be a regression.

**Why it happens:** Copy-paste from `sync_team_stats` without auditing the team guard.

**How to avoid:**
- Write `sync_solo_stats` from scratch rather than copying `sync_team_stats`. The solo function reads the current user's `riot_puuid` directly from the user record, bypasses any team lookup, and calls `riot::fetch_match_history` for just that user.
- Apply rule 44 (return `Ok(...)` not `Err` when an optional entity is absent): if `riot_puuid` is `None`, return `Ok("Link your Riot account first to enable sync.")` — not an error.

**Warning signs:** A solo-mode user without a team clicks "Sync Solo Stats" and sees a red error banner saying "You need to join a team first."

**Phase to address:** Solo sync phase.

---

### Pitfall 9: LP History Requires Calling a Separate Riot Endpoint That Is Rate-Limited Differently

**What goes wrong:** LP and rank data comes from `league-v4` (`getLeagueEntriesForSummoner`), not `match-v5`. This endpoint uses the platform route (EUW1) not the regional route (EUROPE). It returns the current rank/LP, not historical LP per game. There is no Riot API endpoint that returns LP change per match — it must be inferred by calling `getLeagueEntries` before and after each game, or by the user manually recording LP.

A common mistake is to assume the match data contains LP or rank tier information. It does not. The `MatchInfoDto` only contains game mode and queue type.

**Why it happens:** Developers assume the Riot match API contains full performance metadata including LP changes. The documentation is not prominent about this limitation.

**How to avoid:**
- LP tracking must be user-entered (record LP before and after each session) or estimated (snapshot rank at sync time, track changes between syncs).
- On-sync rank snapshot: call `league-v4` once at the start of each sync and store the current LP tier/rank in a `lp_snapshot` record. Over time, a series of snapshots provides a rough LP history.
- Be explicit in the UI that LP history shows rank snapshots at sync time, not per-game LP changes.
- Do not call `league-v4` in a loop per-match (it would exceed rate limits and the endpoint does not provide that data anyway).

**Warning signs:** Developer discovers that `MatchInfoDto` has no `lp_change` field and tries to work around it with per-match API calls. Rate limit errors appear when looping through matches.

**Phase to address:** Ranked climb tracker phase — set correct expectations in design before implementation begins.

---

### Pitfall 10: Match Detail View for 10-Player Data Requires Extracting All Participants, Not Just the Tracked User

**What goes wrong:** The current `fetch_match_history` only extracts the `participant` entry matching the user's `puuid`. The rest of the 10-player data is discarded. For a match detail view showing all participants (items, damage, gold), the full `participants` array must be preserved. If the detail view is implemented by re-fetching the match from the Riot API on demand, it costs another API call per detail view — and will burn rate limit if users scroll through many matches.

**Why it happens:** The current code was designed for personal stats only, not full-match detail. Extending it to show 10-player data requires either re-fetching or storing more data upfront.

**How to avoid:**
- Add a `full_match_json TYPE option<string>` field (or better: a structured `match_detail` table) to store all 10 participants' stats at sync time. The `MatchDto` from the Riot API already contains all participant data — extract and store it during the initial sync rather than discarding it.
- Define a compact `MatchParticipantSummary` struct with the fields needed for the detail view (champion, items, damage dealt/taken, gold, role, win) and store a `Vec<MatchParticipantSummary>` as a JSON column on `player_match` or as child records.
- The detail view then reads from the DB rather than re-fetching. This is important: do not add a per-match-detail Riot API call path, as it makes the detail view slow and burns rate limit on every navigation.

**Warning signs:** The match detail server function calls `riot::get_match()` directly. Network tab shows a Riot API call each time the user clicks a match. Rate limit warnings appear after viewing several match details.

**Phase to address:** Match detail view phase. The data stored during sync must be expanded before the detail view is built — backfilling for already-stored matches will require a one-time re-sync.

---

### Pitfall 11: Personal Learnings Journal Conflicts With Existing `post_game_learning` Team Table

**What goes wrong:** The app already has a `post_game_learning` table scoped to a team (`team ON post_game_learning TYPE record<team>`). The solo "personal learnings journal" is conceptually similar but is per-user, not per-team, and may not have an associated team game. If solo learnings are stored in the same `post_game_learning` table (with `team = NULL`), all existing team-scoped queries that do `WHERE team = type::record('team', $team_key)` will correctly exclude solo entries — but queries that list all of a user's learnings for cross-feature use (e.g. "surface relevant learnings when picking a champion") will need to query both team and solo contexts.

Alternatively, if a separate `solo_learning` table is created, the champion-pool notes integration must query both tables to surface matchup notes for draft.

**Why it happens:** The feature description uses similar language to the existing post-game review. Developers merge them to avoid a new table.

**How to avoid:**
- Create a separate `solo_learning` table (`user ON solo_learning TYPE record<user>`, `champion ON solo_learning TYPE option<string>`, `matchup ON solo_learning TYPE option<string>`, `tags ON solo_learning TYPE array<string>`). Keep it distinct from team post-game reviews.
- The champion pool / draft integration can query `solo_learning WHERE user = $user AND (champion = $champ OR matchup = $champ)` separately from team game plan notes. Union the results in Rust rather than adding a cross-table SurrealQL JOIN.
- This also prevents a future migration if team post-game reviews gain team-specific fields that should not appear in personal learnings.

**Warning signs:** A solo user's personal notes appear in the team's post-game review list. Queries for `post_game_learning` return `None` for solo users and show an error banner instead of an empty state.

**Phase to address:** Personal learnings journal phase.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Storing raw Riot timeline JSON as a string field | No Rust struct needed upfront | 300–500 KB per match, slow queries as data grows, no type safety | Never — process to compact struct before storing |
| Reusing `post_game_learning` for solo notes | One table instead of two | Team-scoped queries silently exclude solo data; cross-feature joins become complex | Never — scope is fundamentally different |
| Hardcoding `EUROPE`/`EUW1` in all Riot calls | Works for current EUW team | All non-EUW solo users get API errors; fix later requires touching every riot.rs call | Only if explicitly scoping to EUW-only in v1.2 |
| Fetching timelines eagerly in the bulk sync loop | Simpler code path | Exhausts rate limit for 20+ matches; blocks server for 60–120 seconds | Never — must be on-demand only |
| Using `team_id IS NULL` as the solo discriminator | No schema change needed | Ambiguous (NULL = solo OR NULL = unassigned team); team queries silently include orphan matches | Never — add an explicit `context` or `solo_owner` field |
| Copying `sync_team_stats` as the base for `sync_solo_stats` | Fast to write | Team guard error is copied verbatim; solo users without a team see error banners | Never — write solo sync from scratch |

---

## Integration Gotchas

Common mistakes when connecting to external services.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Riot match-v5 `get_match` | Loop 20 `get_match` calls sequentially in bulk sync path | Already the current pattern — acceptable for summaries; never extend to also call `get_timeline` in the same loop |
| Riot match-v5 `get_timeline` | Call on every match during bulk sync | Call on-demand only, cache in DB after first fetch |
| Riot league-v4 rank endpoint | Assume it returns LP per game | Returns current snapshot only; use for periodic snapshots, not per-game LP |
| Riot account-v1 PUUID lookup | Re-fetch PUUID on every sync | Fetch once on account linking, store in `user.riot_puuid`, never refetch unless user re-links |
| `riven` rate limit handling | Treat silently-blocked requests as success | Monitor server function response time; >10s for a 20-match sync means rate limit is being hit — reduce concurrency or fetch count |
| SurrealDB `match_id_unique` index | Try to create two `match` records for the same game (one per user) | Reuse the shared `match` record; put per-user data on `player_match` |

---

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| No-cache timeline fetch on detail view load | Each match detail view triggers a Riot API call; slow page, rate limit burns | Cache timeline in DB on first fetch; serve from DB on subsequent loads | After first use — every re-visit to a detail view costs an API call |
| Full `player_match SELECT *` for analytics | Works for 50 matches; scans all records as history grows | Add partial SELECTs with WHERE clauses filtering by user + date range | 200+ matches per user |
| Storing champion performance trends in Rust HashMap at query time | Fast at 50 matches; re-computed on every page load | Cache the aggregated trend as a periodic background compute or as a materialized field on `player_match` | 500+ matches per user |
| `db.rs` solo queries appended without grouping | Hard to find and audit; name collisions with team query structs | Group all solo functions under `// === SOLO MODE ===` section with `DbSolo*` struct naming | Any time — prevents future maintenance |

---

## Security Mistakes

Domain-specific security issues.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Exposing another user's match detail via match ID | Match records are shared by ID; a user who knows a match ID can view all 10 participants' data if the detail view is not user-scoped | Verify that the requesting user has a `player_match` record for that match before returning detail data |
| Storing Riot API key without rotation | Key compromise exposes all team members' Riot data | API key is already loaded from env var via `dotenvy`; document key rotation procedure in CLAUDE.md |
| Solo goals/LP data visible to teammates | Personal rank and goal data is sensitive; should not appear in team views | All solo tables use `record<user>` (not `record<team>`) as the scope; server functions must enforce user ownership |

---

## UX Pitfalls

Common user experience mistakes in dual-mode applications.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Mode toggle is a local signal — resets on page reload | User sets solo mode, navigates away, returns to team mode unexpectedly; confusing state | Persist mode in user DB record; read from server on mount |
| Hiding team nav links without explaining why | Solo user sees empty nav, thinks features are broken | Show team nav links as greyed-out with tooltip "Switch to team mode to use this feature" OR hide entirely but show a persistent mode indicator |
| "Sync Stats" button triggers both solo and team sync | Rate limit doubles; team sync includes solo games if discriminator is missing | Separate "Sync Solo Stats" and "Sync Team Stats" actions; in team mode, only show team sync |
| Match detail view shows all 10 participants with no user highlight | User cannot identify themselves in the scoreboard | Highlight the tracked user's row; sort by role (TOP/JGL/MID/BOT/SUP) for readability |
| Personal learnings journal surfaces globally in team view | Teammate sees another player's private notes about their ranked games | Personal learning records must never appear in team post-game review lists; enforce with `user = $me` clause in all personal learning queries |
| Solo dashboard shows team widgets when user has no team | "No team yet" empty states fill the solo dashboard | Solo dashboard should not render team-only widgets at all; design two distinct dashboard layouts |

---

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Solo mode toggle:** Does the mode persist across page reload? Verify by toggling, reloading, confirming mode unchanged.
- [ ] **Solo stats sync:** Does it work for a user with no team? Verify by creating a fresh user with no team, linking Riot account, clicking sync.
- [ ] **Match detail view:** Are all 10 participants shown, or just the tracked user? Navigate to a match detail and count rows.
- [ ] **Timeline fetch:** Does it serve from DB cache on second load? Call the detail server function twice for the same match and verify only one Riot API call fires.
- [ ] **LP history:** Does the UI clearly indicate these are snapshots, not per-game LP? Check for misleading axis labels.
- [ ] **Personal learnings:** Do they appear in the team post-game review list? Add a solo learning, navigate to team post-game page, verify it is absent.
- [ ] **Champion performance trends:** Are data points from team games mixed with solo queue games? Verify a team-game match does not inflate the solo champion trend numbers.
- [ ] **Region support:** Does a hypothetical NA user get an error on account linking? Check `riot.rs` for hardcoded `EUROPE`/`EUW1`.

---

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| `match` table has no context discriminator — solo and team matches are mixed | HIGH | Add `context` field with `DEFINE FIELD OVERWRITE`; write a one-time Rust migration script that sets `context = 'team'` for matches where any linked `player_match` user is a team member, else `context = 'solo'`; verify with integration test |
| Raw timeline JSON stored — DB bloated | MEDIUM | Add `DEFINE FIELD OVERWRITE timeline_json TYPE option<string>` (already a string); write a background migration that re-processes raw JSON to compact struct JSON; no schema downtime needed since it is an in-place update |
| Region hardcoded — non-EUW users can't sync | LOW | Add `region` field to user table; update `riot.rs` to accept region param; existing EUW users default to `'EUW'` — no data migration needed |
| `sync_solo_stats` copied from team version — errors for no-team users | LOW | Remove team guard from solo function; replace with user PUUID check |
| Solo learnings mixed into `post_game_learning` | MEDIUM | Add `is_solo TYPE bool DEFAULT false` field to `post_game_learning`; filter all team queries with `is_solo = false`; long-term migrate to separate `solo_learning` table |

---

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| `match` table context ambiguity (Pitfall 1) | Data model phase — first phase of v1.2 | Integration test: solo user match does not appear in team stats query |
| Rate limit exhaustion from timeline fetching (Pitfall 2) | Match sync phase — establish on-demand-only timeline pattern | Verify no timeline call in `sync_solo_stats`; verify timeline is fetched only in match detail server fn |
| Timeline JSON storage bloat (Pitfall 3) | Match detail view phase — before first timeline fetch | DB size check: single timeline stored < 5 KB (compact) vs 300+ KB (raw) |
| Solo toggle without global context (Pitfall 4) | Solo mode toggle phase — first phase | Page reload test: mode persists; team nav absent in solo mode |
| Schema migration naming collisions (Pitfall 5) | Data model phase and every subsequent phase | CI: `cargo check --features ssr` passes; server starts cleanly after schema re-apply |
| Per-user data on shared `match` table (Pitfall 6) | Data model phase — before solo sync | Multi-user test: two users play same game; both see correct solo stats |
| Hardcoded region (Pitfall 7) | Riot account linking phase | Manual test: verify `riot.rs` uses passed-in region, not hardcoded constant |
| Team guard copied to solo sync (Pitfall 8) | Solo sync phase | Smoke test: user with no team can sync solo stats successfully |
| LP history from match data (Pitfall 9) | Ranked climb tracker phase | Verify `league-v4` is used for rank snapshots, not `match-v5` metadata |
| Match detail re-fetch on view (Pitfall 10) | Match detail view phase | Network test: second visit to a match detail view shows 0 Riot API calls |
| Personal learnings in team table (Pitfall 11) | Personal learnings journal phase | Data isolation test: solo learning absent from team post-game review list |

---

## Sources

- Codebase analysis: `/home/jasper/Repositories/lol_team_companion/src/server/riot.rs` (all current Riot API calls, hardcoded routes, sequential loop pattern)
- Codebase analysis: `/home/jasper/Repositories/lol_team_companion/schema.surql` (table definitions, `team_id` on match table, `match_id_unique` index)
- Codebase analysis: `/home/jasper/Repositories/lol_team_companion/src/server/db.rs` lines 1454–1530 (`store_matches` implementation, team_id not set)
- Codebase analysis: `/home/jasper/Repositories/lol_team_companion/src/pages/stats.rs` (team guard pattern in `sync_team_stats`)
- Riot API rate limits: [hextechdocs.dev/rate-limiting](https://hextechdocs.dev/rate-limiting/) — development key: 20 req/s, 100 req/2-min; per-key limits shared across all endpoints
- Riot API data retention: [riot-api-libraries.readthedocs.io](https://riot-api-libraries.readthedocs.io/en/latest/specifics.html) — timelines retained 1 year (vs 2 years for matches)
- Timeline data size: [MDPI Applied Sciences paper on LoL match data summarization](https://www.mdpi.com/2076-3417/15/13/7190) — raw match JSON ~80,000 tokens, reducible to <2,000 tokens while retaining analytical value; estimated ~300–500 KB raw per match
- Timeline frame structure: [riot-api-libraries.readthedocs.io](https://riot-api-libraries.readthedocs.io/en/latest/specifics.html) — 1-minute frame intervals; events include kills, objectives, item purchases
- riven crate rate limiting: [crates.io/crates/riven](https://crates.io/crates/riven/2.24.0) — handles dynamic rate limits from response headers; silently delays rather than errors; can reach full rate limit throughput without 429s
- CLAUDE.md rules 1–5 (SurrealDB patterns), 7–11 (Leptos server fn patterns), 44 (Ok vs Err for empty state), 57 (server fn URL patterns)
- Previous PITFALLS.md for v1.1 milestone (cross-feature intelligence pitfalls, champion name normalization, db.rs growth)

---
*Pitfalls research for: solo player mode and match intelligence — v1.2 milestone*
*Researched: 2026-03-25*
