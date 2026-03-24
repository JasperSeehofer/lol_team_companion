---
phase: 10-opponents-redesign
verified: 2026-03-23T15:00:00Z
status: human_needed
score: 14/14 must-haves verified
re_verification: false
human_verification:
  - test: "Navigate to /opponents while logged in with a team. Click '+ New Opponent'. Verify 5 role rows (Top, Jungle, Mid, Bot, Support) appear with Name#Tag inputs."
    expected: "Form shows team name input, 5 labeled role rows, Save & Fetch and Discard Form buttons."
    why_human: "WASM hydration renders the form; compile-time checks cannot confirm DOM rendering."
  - test: "Enter a team name and at least one Riot ID (e.g. 'Faker#KR1'). Click 'Save & Fetch'. Verify the opponent appears in the left list and a detail panel opens showing 5 player cards."
    expected: "Opponent saved toast appears; detail panel shows 5 PlayerCard components."
    why_human: "Requires a running server and valid DB round-trip; cannot be verified statically."
  - test: "If a Riot API key is configured, verify champion pills appear with mastery suffixes (e.g. 'Ahri M7') sorted by mastery points, and the recency badge shows 'Last fetched: just now'."
    expected: "Pills render with M-level suffixes; recency badge is green/muted (not orange)."
    why_human: "Requires live Riot API key and actual network call."
  - test: "Click 'Pool Analysis' toggle on any player card that has been fetched. Verify the section expands to show pool size, role distribution percentages, and top 3 comfort picks."
    expected: "Section expands inline; shows 'Pool: N champions', role distribution, comfort picks."
    why_human: "Collapsible expand/collapse behavior requires browser interaction."
  - test: "On a player card for a player who has >60% of games on a single champion, verify the OTP badge (orange, '⚠ OTP: {ChampionName}') appears."
    expected: "Orange OTP badge visible in row 1 of the card."
    why_human: "Requires real fetched data with an OTP player profile."
  - test: "Click 'Refresh All' and an individual player refresh icon button. Verify spinner appears during fetch and success check-mark appears after."
    expected: "FetchState spinner shown during fetch; ✓ shown on success; auto-clears after ~3s."
    why_human: "Animation and timing behavior requires live browser observation."
---

# Phase 10: Opponents Redesign Verification Report

**Phase Goal:** Opponent profiles are rebuilt with a structured 5-role form, auto-fetched Riot data, OTP detection, mastery badges, and pool analysis
**Verified:** 2026-03-23T15:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | OpponentPlayer model carries last_fetched, mastery_data_json, and role_distribution_json fields | VERIFIED | Lines 25, 28, 31 of `src/models/opponent.rs` — all three `Option<String>` fields with `#[serde(default)]` |
| 2 | compute_intel() produces correct OTP detection at the >60% threshold | VERIFIED | Line 70: `if *count as f64 / total as f64 > 0.6` (strictly greater than). 13 unit tests all pass, including boundary cases: 7/10 = OTP, 6/10 = not OTP, 5/10 = not OTP. |
| 3 | compute_intel() produces champion frequencies, comfort picks, and role distribution | VERIFIED | `champion_frequencies` sorted desc by count; `comfort_picks()` returns top 3 with %; `role_distribution()` deserializes JSON field |
| 4 | create_opponent_with_players DB function creates 1 opponent + 5 player records atomically | VERIFIED | `db.rs` line 2455: creates opponent first, then uses `BEGIN TRANSACTION`/`COMMIT TRANSACTION` for all player slots. Integration test `test_create_opponent_with_players` passes. |
| 5 | update_opponent_player_intel DB function persists enriched data including last_fetched | VERIFIED | `db.rs` line 2531: query sets `last_fetched = time::now()` server-side. Integration test `test_update_opponent_player_intel` passes. |
| 6 | fetch_player_intel returns champion names, per-match role data, and mastery data in a single call | VERIFIED | `src/server/riot.rs` line 181: function collects `champion_with_role` per match with `p.team_position`, then calls `fetch_champion_masteries`. Returns `PlayerIntelData { recent_champions, champion_with_role, mastery_data }`. |
| 7 | Server functions for creating opponents and fetching player intel exist and are callable from the UI | VERIFIED | `create_opponent_with_players_fn` (line 231) and `fetch_player_intel_fn` (line 271) in `src/pages/opponents.rs`. Both compile for SSR and hydrate targets. |
| 8 | New opponent form shows 5 pre-labeled role rows with Name#Tag inputs | VERIFIED (automated) | `src/pages/opponents.rs` contains `ROLES` constant with 5 entries; `Name#Tag (e.g. Faker#KR1)` placeholder on each row input; e2e test confirms all 5 role labels visible after clicking `+ New Opponent`. |
| 9 | Save & Fetch creates opponent + players and kicks off per-player Riot fetches | VERIFIED | Line 641: `spawn_local` calls `create_opponent_with_players_fn`, destructures `(opponent_id, player_ids)`, then loops `player_ids` calling `fetch_player_intel_fn` per player with riot ID. |
| 10 | Player cards show OTP warning badge when >60% games on one champion | VERIFIED | Line 1110-1117: `otp_champion.as_ref().map(...)` renders `⚠ OTP: {champ}` with orange styling. `compute_intel()` supplies the value from real `recent_champions` data. |
| 11 | Champion pills display mastery level suffix sorted by mastery points descending | VERIFIED | Lines 1047-1055: `sorted_champions` built from `recent_champions` sorted by mastery points desc via `mastery_map`. Pills render `{name} M{level}` suffix via `mastery_map_stored.with_value()`. |
| 12 | Data recency badge shows relative time, goes orange after 7+ days | VERIFIED | Lines 1097-1129: `recency_display` calls `is_stale(lf)` to pick `text-orange-400` vs `text-muted`. "Never fetched" shown when `last_fetched` is None. |
| 13 | Collapsible pool analysis shows pool size, role distribution, and top 3 comfort picks | VERIFIED | Lines 1219-1272: `pool_expanded` toggle; expands to show `pool_sz`, role distribution with position-name mapping (TOP→Top, JUNGLE→Jungle, MIDDLE→Mid, BOTTOM→Bot, UTILITY→Support), comfort picks as `{name} {pct:.0}%`. |
| 14 | Individual refresh button and Refresh All button both trigger fetch_player_intel_fn | VERIFIED | Individual refresh: line 1069 calls `fetch_player_intel_fn`. Refresh All: lines 812-841 loops all 5 players calling `fetch_player_intel_fn` sequentially. |

**Score:** 14/14 truths verified (automated checks)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `schema.surql` | last_fetched, mastery_data_json, role_distribution_json fields | VERIFIED | Lines 188-190: all 3 fields with `IF NOT EXISTS`, correct types (`option<datetime>`, `option<string>`) |
| `src/models/opponent.rs` | Extended OpponentPlayer with new fields, compute_intel() | VERIFIED | 289 lines; fields at lines 25/28/31; `compute_intel` at line 52; all helper methods present |
| `src/server/db.rs` | create_opponent_with_players, update_opponent_player_intel | VERIFIED | `create_opponent_with_players` at line 2455; `update_opponent_player_intel` at line 2519; `DbOpponentPlayer` updated with 3 new fields + `From` impl at lines 2239-2254 |
| `tests/db_opponents.rs` | Integration tests for batch creation and intel persistence | VERIFIED | 3 tests: `test_create_opponent_with_players`, `test_update_opponent_player_intel`, `test_create_opponent_with_empty_summoner_names`. All pass. |
| `src/server/riot.rs` | fetch_player_intel function returning PlayerIntelData | VERIFIED | `PlayerIntelData` struct at line 175; `fetch_player_intel` at line 181 — captures `p.team_position` per match, calls `fetch_champion_masteries` |
| `src/pages/opponents.rs` | Redesigned OpponentsPage, new server fns, PlayerCard | VERIFIED | `create_opponent_with_players_fn` (line 231), `fetch_player_intel_fn` (line 271), `FetchState` enum (line 345), `PlayerCard` component with OTP/mastery/pool analysis |
| `e2e/tests/opponents.spec.ts` | E2e smoke test for opponents page | VERIFIED | 3 tests: page load, 5-role form, discard form. Uses `teamPage` fixture. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/models/opponent.rs` | `src/server/db.rs` | `DbOpponentPlayer -> OpponentPlayer` includes new fields | VERIFIED | `From<DbOpponentPlayer>` at lines 2239-2254 maps `last_fetched`, `mastery_data_json`, `role_distribution_json` |
| `schema.surql` | `src/server/db.rs` | `DbOpponentPlayer` struct matches schema fields | VERIFIED | `last_fetched: Option<String>` at line 2232 with `#[serde(default)]`; pattern uses `<string>last_fetched AS last_fetched` cast in SELECT to handle datetime deserialization |
| `src/server/riot.rs` | `src/pages/opponents.rs` | `riot::fetch_player_intel` called from `fetch_player_intel_fn` | VERIFIED | Line 309: `let intel = riot::fetch_player_intel(&puuid, 20)` |
| `src/pages/opponents.rs` | `src/server/db.rs` | Server fns call `db::create_opponent_with_players` and `db::update_opponent_player_intel` | VERIFIED | Line 259: `db::create_opponent_with_players`; line 326: `db::update_opponent_player_intel` |
| `src/pages/opponents.rs` | `create_opponent_with_players_fn` | `spawn_local` destructures `(opponent_id, player_ids)` | VERIFIED | Lines 641-654: `Ok((opponent_id, player_ids)) =>` then loops `player_ids` |
| `src/pages/opponents.rs` | `fetch_player_intel_fn` | Called per player using player_ids from creation, and on refresh | VERIFIED | Line 653 (post-create loop); line 821 (Refresh All); line 1069 (individual refresh) |
| `src/pages/opponents.rs` | `OpponentPlayer::compute_intel` | Called in PlayerCard to derive OTP, frequencies, mastery | VERIFIED | Line 1031: `let intel = player.compute_intel()` |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| `PlayerCard` (opponents.rs) | `mastery_data` | `player.mastery_data()` → deserializes `mastery_data_json` from DB | `update_opponent_player_intel` writes Riot API data to DB; `get_opponent` SELECT retrieves it | FLOWING |
| `PlayerCard` (opponents.rs) | `role_distribution` | `player.role_distribution()` → deserializes `role_distribution_json` from DB | Computed from `champion_with_role` in `fetch_player_intel_fn`, serialized to JSON, written by `update_opponent_player_intel` | FLOWING |
| `PlayerCard` (opponents.rs) | `otp_champion` | `intel.otp_champion` from `compute_intel()` on `player.recent_champions` | `recent_champions` written by `update_opponent_player_intel` from Riot API match data | FLOWING |
| `PlayerCard` (opponents.rs) | `recency_display` | `player.last_fetched` | Set to `time::now()` server-side in `update_opponent_player_intel` | FLOWING |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Unit tests: OTP detection, frequencies, comfort picks, stale checks | `cargo test --features ssr --lib -- opponent` | 13 passed, 0 failed | PASS |
| Integration tests: batch create, intel persistence | `cargo test --features ssr --test db_opponents` | 3 passed, 0 failed | PASS |
| SSR compile target | `cargo check --features ssr` | 0 errors | PASS |
| WASM hydrate compile target | `cargo check --features hydrate --target wasm32-unknown-unknown` | 0 errors | PASS |
| E2e tests: page load, 5-role form, discard | Documented passing in 10-03-SUMMARY.md (3 tests) | 3 passed | PASS (previously run) |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| OPP-01 | 10-01, 10-02, 10-03 | New opponent form asks for team name and provides 5 role fields (top/jg/mid/bot/sup) with Riot ID input; on save, auto-fetches champion data from Riot API | SATISFIED | `CreationForm` renders 5 ROLES rows; `Save & Fetch` calls `create_opponent_with_players_fn` then loops `player_ids` calling `fetch_player_intel_fn` |
| OPP-02 | 10-01, 10-03 | Opponent players show OTP warning badge when >60% of recent games are on a single champion | SATISFIED | `compute_intel()` with strictly `> 0.6` threshold; OTP badge rendered at line 1110 |
| OPP-03 | 10-01, 10-02, 10-03 | Opponent player cards display champion mastery level badges and data recency indicator (last fetched date) | SATISFIED | Mastery pills with `M{level}` suffix (line 1197-1211); recency badge with `is_stale` color logic (line 1120-1129) |
| OPP-04 | 10-01, 10-03 | Each opponent player has a champion pool depth/breadth analysis showing pool size, role flexibility, and comfort picks | SATISFIED | Collapsible Pool Analysis showing `pool_sz` champions, role distribution %, comfort picks (lines 1219-1272) |

---

### Anti-Patterns Found

No blocker or warning anti-patterns found. Scan results:

- No TODO/FIXME/PLACEHOLDER comments in modified files
- No `return null` / `return {}` / `return []` stub patterns (empty state is handled with proper `EmptyRoleSlot` component and "Fetch player data to see pool analysis." message)
- `prop:value` used on all controlled inputs (10 occurrences)
- `collect_view()` used for all iterator-to-view conversions (10 occurrences)
- `spawn_local` used for all async event handlers

---

### Human Verification Required

All automated checks pass. The following behaviors require live browser verification:

#### 1. 5-Role Creation Form Rendering

**Test:** Navigate to `/opponents` while logged in with a team. Click `+ New Opponent`.
**Expected:** Form appears with team name input, 5 role rows (Top/Jungle/Mid/Bot/Support) each with a `Name#Tag` placeholder input, plus `Save & Fetch` and `Discard Form` buttons.
**Why human:** WASM hydration wires the `on:click` handler asynchronously — the e2e tests confirmed this works but a final visual check is the contract gate.

#### 2. Save & Fetch End-to-End Flow

**Test:** Enter a team name and at least one Riot ID (e.g. `Faker#KR1`). Click `Save & Fetch`.
**Expected:** Toast "Opponent saved and players fetched" appears; opponent appears in left list; clicking it shows detail panel with 5 player cards.
**Why human:** Requires running server and live DB round-trip.

#### 3. Mastery Pills and Recency Badge (Riot API key required)

**Test:** After a successful fetch with a live API key, view a player card.
**Expected:** Champion pills show `{Name} M{level}` suffixes sorted by mastery points descending. Recency badge shows `Last fetched: just now` in muted color.
**Why human:** Requires valid `RIOT_API_KEY` in `.env` and actual Riot API network call.

#### 4. Pool Analysis Collapsible

**Test:** Click `Pool Analysis` toggle on a fetched player card.
**Expected:** Section expands showing pool size, role distribution (e.g. `Mid 60% / Bot 40%`), and top 3 comfort picks with percentages.
**Why human:** Expand/collapse animation and correct percentage display requires browser interaction.

#### 5. OTP Badge Visibility

**Test:** View a player card where a fetched player has >60% of games on one champion.
**Expected:** Orange badge `⚠ OTP: {ChampionName}` visible in card header row.
**Why human:** Requires real fetched data where an OTP profile is present.

#### 6. Refresh Buttons with Spinner/Status

**Test:** Click individual player refresh icon and `Refresh All` button.
**Expected:** Spinner (`animate-spin`) appears during fetch; `✓` appears on success; auto-clears after ~3 seconds.
**Why human:** Animation timing and FetchState transitions require live observation.

---

### Gaps Summary

No automated gaps found. All 14 truths verified, all 7 required artifacts substantive and wired, all 4 requirements satisfied, both compile targets pass, 16 tests pass (13 unit + 3 integration).

The phase is pending human visual verification of the complete UI flow — a `checkpoint:human-verify` gate was explicitly included in Plan 03. Task 3 of Plan 03 remains pending per the SUMMARY.

---

_Verified: 2026-03-23T15:00:00Z_
_Verifier: Claude (gsd-verifier)_
