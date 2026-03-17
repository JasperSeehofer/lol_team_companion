---
phase: 05-post-game-loop-polish
plan: "02"
subsystem: post-game-pipeline
tags: [action-items, auto-generation, dedup, post-game, pipeline]
dependency_graph:
  requires: []
  provides: [batch_create_action_items_from_review, post-game-action-item-pipeline]
  affects: [src/server/db.rs, src/pages/post_game.rs]
tech_stack:
  added: []
  patterns: [batch-dedup-creation, server-fn-tuple-return, inline-feedback-banner]
key_files:
  created: []
  modified:
    - src/server/db.rs
    - src/pages/post_game.rs
decisions:
  - Used list_open_action_items (existing fn) for dedup query rather than a new DB query — avoids duplication and reuses existing index-optimized query
  - create_review return type changed from String to (String, usize) tuple — tuple is Serialize+Deserialize so no wire format issues
  - update_review return type changed from () to usize — simpler than a struct for a single count
  - Banner only shown when n_items > 0 — avoids noisy "0 action items created" message on updates with no new improvements
metrics:
  duration: "~30 minutes"
  completed_date: "2026-03-17T15:31:00Z"
  tasks_completed: 2
  files_modified: 2
---

# Phase 05 Plan 02: Post-Game Action Item Auto-Generation Summary

Post-game reviews now automatically create action items from the `improvements` array with case-insensitive deduplication and inline user feedback.

## What Was Built

### Task 1: batch_create_action_items_from_review (db.rs)

Added `pub async fn batch_create_action_items_from_review` to `src/server/db.rs` near the existing `create_action_item` and `list_open_action_items` functions.

The function:
- Returns `Ok(0)` immediately for empty improvements arrays
- Calls `list_open_action_items` to fetch existing open/in_progress items
- Builds a `HashSet<String>` of lowercased existing item texts for O(1) dedup lookup
- Skips empty/whitespace-only strings via `.trim().is_empty()`
- Skips improvements that case-insensitively match existing open items
- Calls `create_action_item` for each new improvement, setting `source_review` to the review ID

Two unit tests added to the existing `#[cfg(test)] mod tests` block:
- `dedup_filters_matching_open_items`: validates the HashSet dedup logic in isolation
- `empty_improvements_returns_zero`: documents the early-return contract for empty input

### Task 2: Wiring + Inline Banner (post_game.rs)

**Server fn changes:**
- `create_review` return type: `Result<String, ServerFnError>` → `Result<(String, usize), ServerFnError>`
- `update_review` return type: `Result<(), ServerFnError>` → `Result<usize, ServerFnError>`
- Both fns call `batch_create_action_items_from_review` after the review is persisted

**Client-side changes in `PostGamePage`:**
- Added `action_item_count: RwSignal<Option<usize>>` signal
- `do_save` handler restructured into separate `if is_update { ... } else { ... }` branches to properly destructure the new return types
- `create_review` match arm destructures `Ok((id, n_items))`
- `update_review` match arm destructures `Ok(n_items)`
- Both branches set `action_item_count` to `Some(n)` when `n > 0`, `None` otherwise

**Inline banner:**
- Placed between the `StatusMessage` and the main flex layout
- Shows "1 action item created" or "N action items created" with a "View" link to `/action-items`
- Only rendered when `action_item_count` is `Some(_)` (i.e., at least 1 new item was created)
- Styled with accent color: `bg-accent/10 border border-accent/30`

## Verification

- `cargo test --features ssr --lib` — both new tests pass
- `cargo check --features ssr` — clean (1 pre-existing unrelated warning)
- `cargo check --features hydrate --target wasm32-unknown-unknown` — clean

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- `src/server/db.rs` modified: confirmed (contains `batch_create_action_items_from_review`)
- `src/pages/post_game.rs` modified: confirmed (contains `action_item_count`, `action items created`, `href="/action-items"`)
- Commit 76a048e (Task 1): confirmed
- Commit 0495e5b (Task 2): confirmed
