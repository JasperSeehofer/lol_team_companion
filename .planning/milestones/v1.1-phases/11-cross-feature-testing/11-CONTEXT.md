# Phase 11: Cross-Feature & Testing - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Build a plan effectiveness analytics page that aggregates post-game review outcomes per game plan and strategy tag, add win/loss tracking and user ratings to post-game reviews with auto-detection from Riot API match history, and seed the app with real test data (2 teams of 5 Riot-linked accounts with full app data). XFEAT-02 is already satisfied by Phase 9's DRFT-05 implementation and is excluded from this phase's scope.

</domain>

<decisions>
## Implementation Decisions

### Plan Effectiveness Metrics (XFEAT-01)
- **D-01:** Add a `win_loss` outcome field (`win`/`loss`) to `PostGameLearning` model
- **D-02:** Add a `rating` field (1-5 integer stars) to `PostGameLearning` — "How well did the plan work?" independent of win/loss
- **D-03:** Auto-detect win/loss on post-game review creation: fetch recent matches from Riot API, match against draft champions to identify the correct game, and auto-fill the outcome
- **D-04:** Show a "Fetch result" button as fallback when the game hasn't finished yet or the match wasn't auto-detected
- **D-05:** Effectiveness aggregates by BOTH strategy tag (top-level cards) AND individual game plan (table below). Strategy tag grouping uses the `win_condition_tag` field on `GamePlan`
- **D-06:** Clicking a game plan row expands inline (accordion) to show linked post-game reviews with outcomes, ratings, and key takeaways

### Effectiveness View Layout
- **D-07:** New dedicated page at `/analytics` with "Analytics" nav link (auth-required)
- **D-08:** Top row: strategy tag summary cards showing win rate, average rating, and games played per tag
- **D-09:** Below cards: sortable table of individual game plans with columns for name, tag, W-L record, average rating
- **D-10:** No filters for v1.1 — show all data. Filters deferred to future iteration
- **D-11:** Inline accordion expansion on row click — shows linked reviews without leaving the page

### XFEAT-02 Scope
- **D-12:** XFEAT-02 (draft sidebar champion notes) is already satisfied by Phase 9's DRFT-05 implementation (Notes tab with champion sub-tabs, auto-opens on pick). Mark complete, exclude from Phase 11 work

### Test Data Seeding (TEST-01)
- **D-13:** Use real Riot accounts (user's own accounts + friends/alts) for realistic demo data
- **D-14:** Rust binary seed script that performs a clean slate: wipes test accounts, creates 2 teams with 5 users each, links Riot accounts, populates all app data. Idempotent — safe to re-run
- **D-15:** Seed scope includes: champion pools with tiers + notes, 2-3 drafts per team with linked game plans, post-game reviews with win/loss + ratings (to populate the effectiveness page), and 1-2 opponent scouting profiles with 5-role player data
- **D-16:** Seed script fetches real champion mastery and match history from Riot API during seeding

### Claude's Discretion
- Exact star rating UI component (CSS stars, SVG, or emoji)
- Strategy card visual design (colors, icons per tag)
- Accordion animation/transition for plan row expansion
- Seed script CLI interface (args, flags, output format)
- How to handle Riot API rate limiting during seed (backoff strategy)
- Schema migration approach for new PostGameLearning fields (add with defaults for backward compat)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — XFEAT-01, XFEAT-02, TEST-01 acceptance criteria
- `.planning/ROADMAP.md` — Phase 11 success criteria (3 items)

### Existing implementations (built in prior phases)
- `src/models/game_plan.rs` — `GamePlan` (has `win_condition_tag`) and `PostGameLearning` structs (need new fields)
- `src/pages/post_game.rs` — Post-game review creation/listing, `analyze_patterns()` function
- `src/pages/game_plan.rs` — Game plan page with draft linkage
- `src/pages/draft.rs` — Intel sidebar Notes tab (DRFT-05/XFEAT-02 implementation, lines ~943-1161)
- `src/server/db.rs` — `save_post_game_learning()`, `list_team_post_game_learnings()`, game plan DB functions
- `src/server/riot.rs` — Riot API client (match history, champion mastery)
- `schema.surql` — DB schema (needs PostGameLearning field additions)

### Prior phase context
- `.planning/phases/09-draft-game-plan/09-CONTEXT.md` — D-10 through D-15 (Notes tab decisions), D-16 through D-20 (game plan autofill)

### Codebase maps
- `.planning/codebase/ARCHITECTURE.md` — Overall architecture
- `.planning/codebase/CONVENTIONS.md` — Code patterns and conventions
- `.planning/codebase/TESTING.md` — Test infrastructure

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `PostGameLearning` model in `src/models/game_plan.rs` — needs `win_loss: Option<String>` and `rating: Option<u8>` fields added
- `analyze_patterns()` in `src/pages/post_game.rs` — existing pattern analysis logic, can inform effectiveness aggregation approach
- `GamePlan.win_condition_tag` — already exists, used for strategy tag grouping
- `PostGameLearning.game_plan_id: Option<String>` — linkage to game plans already exists
- `PostGameLearning.match_riot_id: Option<String>` — linkage to Riot matches exists for auto-detection
- Riot API client in `src/server/riot.rs` — match history and champion data fetching
- `stat_card.rs` component — reusable for strategy tag summary cards

### Established Patterns
- New pages follow: create file in `src/pages/`, add route in `app.rs`, add nav link in `nav.rs`, add e2e smoke test
- Server functions use `use_context::<Arc<Surreal<Db>>>()` for DB access
- DB structs use `Db*` prefix with `RecordId` fields, converted to app-facing structs with `String` IDs
- Schema fields use `DEFINE FIELD IF NOT EXISTS` for idempotent re-application
- Protected pages redirect to `/auth/login` via client-side check

### Integration Points
- `/analytics` route added to `app.rs` router
- Nav link added to `nav.rs` (auth-gated)
- `PostGameLearning` model changes affect `post_game.rs` (creation form), `db.rs` (save/list functions), `schema.surql`
- Seed script needs access to the same DB and Riot API client code used by the server

</code_context>

<specifics>
## Specific Ideas

- Auto-match detection: when creating a post-game review, fetch recent Riot API matches and match against the draft's champion composition to pinpoint the correct game — then auto-fill win/loss
- "Fetch result" button fallback for reviews created before the game finished
- Strategy cards should feel like the existing stat cards on the dashboard — consistent visual language
- Seed script should be a proper Rust binary, not a hacky script — idempotent, clean slate approach

</specifics>

<deferred>
## Deferred Ideas

- Date range / time filters on analytics page — add when enough data accumulates
- Chart visualizations (bar charts, trend lines) — future analytics enhancement
- Click strategy card to filter table to that tag — nice interaction but not needed for v1.1

</deferred>

---

*Phase: 11-cross-feature-testing*
*Context gathered: 2026-03-24*
