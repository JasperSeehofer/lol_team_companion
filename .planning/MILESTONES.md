# Milestones

## v1.1 Polish, Draft & Opponents Rework (Shipped: 2026-03-24)

**Phases completed:** 6 phases, 17 plans, 20 tasks

**Key accomplishments:**

- 1. [Rule 3 - Blocking] .mcp.json is gitignored
- Fixed all 5 known regressions: tree drafter freeze, game plan nav, roster leader, hydration mismatch, and drag-drop source clearing, with 5 passing regression tests
- Replaced 7 sets of local captureErrors/filterRealErrors with shared helper imports and standardized navigation in all audit/pipeline specs; full e2e suite passes green with 60 tests
- One-liner:
- Champion pool overhauled from pill entries to 3-column card grid with portrait photos, drag-and-drop tier management via DataTransfer API, and ChampionAutocomplete-based matchup note entry with opponent icon display
- Role icon badges on draft pick slots with auto-guess from champion tags, Community Dragon SVGs, and popover selection — persisted via DraftAction.role field in SurrealDB
- Task 1 — Opponent Autocomplete + Add New + Intel Cleanup (DRFT-01, DRFT-02, D-01 through D-06):
- Task 1 — DB Query + Server Function (DRFT-05):
- Game plan editor auto-fills plan name as "<draft name> plan", strategy tag from most common draft tag, and per-role "<ChampionName>: " prompts from draft role assignments, wired via extended Leptos prefill Effect and new most_common_tag helper
- 1. [Rule 3 - Blocking] DbOpponentPlayer missing new fields blocked test compilation
- `create_opponent_with_players_fn(name, players_json) -> Result<(String, Vec<String>), ServerFnError>`
- 5-role creation form with Save & Fetch, enriched player cards showing OTP badges/mastery pills/recency indicators/collapsible pool analysis, individual and team-level refresh buttons — full opponents page redesign.
- PostGameLearning extended with win_loss/rating fields (backward-compat via serde default), shared analytics types added, and get_analytics DB function aggregating game plan effectiveness by strategy tag
- Task 1 — Win/Loss selector and Star Rating:
- Analytics page at /analytics with strategy tag cards, sortable effectiveness table, accordion reviews, route, nav link, and e2e smoke test entry
- Rust seed binary creating 2 teams x 5 users with champion pools, 20-action drafts, game plans, and post-game reviews (win_loss/rating) for analytics page demo data

---

## v1.0 Cross-Feature Intelligence (Shipped: 2026-03-18)

**Phases completed:** 6 phases, 21 plans
**Timeline:** 4 days (2026-03-14 → 2026-03-18)
**Git range:** 101 commits, 97 files changed (+16,313 / -544 lines)
**Codebase:** 22,986 lines Rust

**Delivered:** Transformed the app from siloed features into one connected tool — drafts flow into game plans, post-game reviews auto-generate action items, and the dashboard surfaces what matters.

**Key accomplishments:**

1. Pipeline CTAs — draft → game plan → post-game navigation with FK prefill
2. Cross-table aggregation layer with champion name normalization
3. Smart dashboard with action items, post-game reviews, and pool gap panels
4. Inline intel — pool warning badges, opponent tendency sidebar, win condition tracker
5. Post-game automation — auto-creates action items from tagged patterns with dedup
6. UX polish — toast system, skeleton loading, and meaningful empty states across all pages

**Known Gaps (accepted as tech debt):**

- Missing integration tests for DB aggregation functions (Phase 2 criterion #5)
- Dashboard doesn't surface incomplete workflow counts (non-critical)
- `post_game_champ_outcomes` always empty (schema lacks win/loss field)

---
