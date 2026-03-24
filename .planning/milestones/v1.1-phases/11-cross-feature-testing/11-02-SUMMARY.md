---
phase: 11-cross-feature-testing
plan: 02
subsystem: ui
tags: [leptos, post-game, riot-api, win-loss, star-rating, auto-detect]

requires:
  - phase: 11-cross-feature-testing
    plan: 01
    provides: PostGameLearning model with win_loss and rating fields

provides:
  - Win/Loss three-state outcome selector (none/win/loss) in post-game review form
  - 1-5 star Plan Rating input with Unicode star characters and accent fill color
  - auto_detect_outcome server function querying Riot API recent match history
  - Fetch Result button with loading/result state displayed inline

affects:
  - src/pages/post_game.rs

tech-stack:
  added: []
  patterns:
    - "Three-state selector: None/win/loss with active color states (emerald/red/neutral)"
    - "Star rating: (1u8..=5).map() with closure capturing n for each star button"
    - "Conditional Fetch Result button using into_any() on both branches (CLAUDE.md rule 19)"
    - "get_untracked() in all event handlers per CLAUDE.md rule 20"
    - "riot::fetch_match_history() for auto-detect; champion name normalized to lowercase for matching"

key-files:
  created: []
  modified:
    - src/pages/post_game.rs
    - src/pages/analytics.rs

key-decisions:
  - "auto_detect_outcome uses get_draft_for_prefill (existing fn) to load draft picks, not a new fn"
  - "Champion matching uses lowercase comparison to handle Data Dragon casing vs user input"
  - "Fetch button only shown when win_loss=None AND draft_id non-empty — avoids premature calls"
  - "Check first 5 recent matches only (not all 20) to minimize Riot API calls per request"

duration: 20min
completed: 2026-03-24
---

# Phase 11 Plan 02: Post-Game Win/Loss, Rating, and Auto-Detect Summary

Win/loss outcome selector, 1-5 star plan rating, and Riot API auto-detection added to the post-game review form. These fields feed into the analytics aggregation built in Plan 01.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Win/loss selector and star rating to post-game review form | 4a5b098 | src/pages/post_game.rs, src/pages/analytics.rs |
| 2 | Auto-detect outcome server function and Fetch Result button | 5bc1734 | src/pages/post_game.rs |

## What Was Built

**Task 1 — Win/Loss selector and Star Rating:**
- `win_loss: RwSignal<Option<String>>` and `rating: RwSignal<Option<u8>>` signals added to PostGamePage
- Signals populated in `load_review()` from existing review data and cleared in `clear_editor()`
- `build_review()` updated to capture `win_loss.get_untracked()` and `rating.get_untracked()`
- Game Outcome three-button selector: "---" (none), "Win" (emerald active state), "Loss" (red active state)
- Plan Rating 1-5 star row using Unicode U+2605 (filled) / U+2606 (empty), accent color when filled

**Task 2 — Auto-detect outcome:**
- `auto_detect_outcome(draft_id)` server function added before the `#[component]` (CLAUDE.md rule 34)
- Loads user's PUUID from `auth.user.riot_puuid`, returns `Ok(None)` if not linked
- Fetches draft via `db::get_draft_for_prefill` to extract our side's picks
- Calls `riot::fetch_match_history(puuid, None)` and checks first 5 matches for champion overlap
- Returns `Some("win")` / `Some("loss")` / `None` based on match
- Fetch Result button shown conditionally (win_loss=None AND draft non-empty), hides after selection
- Loading state "Fetching...", post-detect "Win detected"/"Loss detected" or fallback "Result not found — select manually"

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed analytics.rs WASM compile failure from moved value**
- **Found during:** Task 1 verification (`cargo check --features hydrate`)
- **Issue:** In `analytics.rs`, `plan_id` was moved into the chevron closure, then used again in the accordion expansion closure, causing E0382 in WASM target
- **Fix:** Added `let plan_id_accordion = plan_id.clone()` before the view! block, used `plan_id_accordion` inside the accordion expansion closure
- **Files modified:** src/pages/analytics.rs
- **Commit:** 4a5b098

### Implementation Adaptations

**2. [Plan note] Used `get_draft_for_prefill` instead of `get_draft`**
- The plan pseudocode referenced `db::get_draft()` which does not exist
- Used `db::get_draft_for_prefill()` which loads both the draft record and its actions in a single query

**3. [Plan note] Used `fetch_match_history` instead of placeholder functions**
- Plan pseudocode referenced `get_recent_match_ids_for_puuid` / `get_match_data_for_id` which don't exist
- Used `riot::fetch_match_history(puuid, None)` which returns full `Vec<MatchData>` with champion + win fields in one call

## Known Stubs

None — all form fields are wired to signals and persisted via `build_review()` to the save/update flow.

## Verification

- `cargo check --features ssr` — exits 0
- `cargo check --features hydrate --target wasm32-unknown-unknown` — exits 0
- `cargo test --features ssr --lib` — 88 passed, 0 failed
