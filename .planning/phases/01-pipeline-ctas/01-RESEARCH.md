# Phase 1: Pipeline CTAs - Research

**Researched:** 2026-03-14
**Domain:** Leptos 0.8 URL query params, SurrealDB FK-driven prefill, cross-page navigation wiring
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**CTA Placement**
- "Prep for This Draft" button appears on both the draft list (quick action per draft) and the draft detail view (prominent)
- "Review This Game" button appears on both the game plan page (bottom of prep flow) and the linked draft detail view
- Buttons are accent/primary style — prominent and discoverable, not subtle
- Labels are flow-based: "Prep for This Draft", "Review This Game" (not action-based like "Create Game Plan")

**Prefill Behavior (Draft → Game Plan)**
- Prefill all available data: champions (our picks + their picks from draft actions), side (blue/red), opponent name, win conditions, and draft notes
- Prefilled champions are locked by default with an "Edit" toggle to override
- Other prefilled fields (opponent, win conditions, notes) are freely editable
- URL pattern: `/game-plan?draft_id=X` — direct navigation and bookmarkable

**Prefill Behavior (Game Plan → Post-Game Review)**
- Claude's discretion on what carries over — pick what's most useful for writing a review

**Back-References**
- Game plan shows source draft as a small info badge — clickable, links to draft detail
- Draft detail shows linked game plans as a badge (e.g., "1 game plan") — clickable
- Post-game review shows badges linking back to BOTH the source game plan AND the source draft
- Badge pattern is consistent across all three pages

**Draft Selection & Duplicates**
- Primary flow: "Prep for This Draft" button creates a new game plan linked to that draft
- Also support manual linking: game plan page has a "Link to draft" dropdown for retroactive linking
- When clicking "Prep for This Draft" and a game plan already exists for that draft: show a prompt — "A game plan already exists for this draft. View it or create another?"

### Claude's Discretion
- Exact badge/tag visual design (should use existing semantic tokens from theme system)
- How "Link to draft" dropdown is implemented (autocomplete, modal, inline select)
- Loading/error states for the prefill query
- Whether the "locked with override" champion UI uses a toggle, edit icon, or click-to-unlock pattern

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PIPE-01 | User can create a game plan from a completed draft, with picks/bans/side prefilled | URL query param pattern (`use_query_map`), new `get_draft_for_prefill` server fn, champion extraction from `DraftAction`, existing `GamePlan.draft_id` FK, duplicate-check query |
</phase_requirements>

---

## Summary

Phase 1 is a pure navigation-wiring phase. No new schema changes are needed — all required FK fields (`GamePlan.draft_id`, `PostGameLearning.game_plan_id`, `PostGameLearning.draft_id`) already exist in the data model. The work falls into four categories: (1) URL query parameter reading via Leptos 0.8's `use_query_map()`, (2) a new server function `get_draft_for_prefill` that fetches a draft by ID and maps its `DraftAction` records into champion lists, (3) UI changes to `draft.rs`, `game_plan.rs`, and `post_game.rs` for CTA buttons and back-reference badges, and (4) a new db query to check whether a game plan already exists for a given draft (duplicate guard).

The "locked champion" toggle and the "Link to draft" dropdown are new UI patterns for this codebase but are self-contained. The duplicate-check flow requires a small new db query but no schema change. All navigation uses the established `window.location().set_href()` hard-nav pattern already throughout the codebase.

The key technical fact to internalize: `use_query_map()` returns `Memo<ParamsMap>`. Reading `draft_id` from the URL and driving a conditional `Resource::new()` off it is the entire machinery for the bookmarkable prefill requirement.

**Primary recommendation:** Wire the pipeline using `use_query_map()` for param reading, a new `get_draft_for_prefill(draft_id)` server function for data, and the established `window.location().set_href()` pattern for CTA navigation. No new dependencies, no schema changes.

---

## Standard Stack

### Core (All Existing — No New Dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `leptos_router::hooks::use_query_map` | 0.8.12 | Read URL query params reactively | Built into leptos_router; confirmed in installed crate source |
| `leptos_router::components::A` | 0.8.12 | Render navigation links with `?draft_id=` query strings | Existing pattern in codebase |
| `surrealdb` | 3.x | New `get_draft_for_prefill` query | Existing db.rs batch query pattern |
| `web_sys` | existing | Hard-nav via `window.location().set_href()` | Established pattern for all auth/navigation in codebase |

**No new Cargo.toml entries required.**

---

## Architecture Patterns

### Pattern 1: URL Query Param Reading with `use_query_map()`

**What:** `use_query_map()` from `leptos_router::hooks` returns `Memo<ParamsMap>`. Call `.get()` inside a derived signal to extract a typed param. Pass the derived signal as the key to a `Resource::new()` so the resource re-fetches when the URL changes.

**When to use:** Any page that accepts a context param from another page. Here: `game_plan.rs` reads `draft_id`, `post_game.rs` reads `plan_id`.

**Import path (verified from installed crate):**
```rust
use leptos_router::hooks::use_query_map;
```

**Pattern:**
```rust
// In GamePlanPage component body:
let query = use_query_map();
let prefill_draft_id = move || query.read().get("draft_id");

// Resource that re-fetches if the URL param changes:
let prefill_data = Resource::new(
    move || query.read().get("draft_id"),
    |draft_id_opt| async move {
        match draft_id_opt {
            Some(id) if !id.is_empty() => get_draft_for_prefill(id).await,
            _ => Ok(None),
        }
    },
);
```

**Confidence:** HIGH — `use_query_map` confirmed in `leptos_router-0.8.12/src/hooks.rs`, `ParamsMap::get(&str) -> Option<String>` confirmed in `params.rs`.

### Pattern 2: Conditional Prefill Resource → Signal Seeding

**What:** When `prefill_data` resolves with `Some(draft)`, seed the game plan editor signals (champion slots, opponent, win conditions) in an `Effect`. Mark champion slots as "from prefill" with a separate `RwSignal<bool>` (the locked/edit-toggle state).

**Key implementation detail:** The `Effect` that seeds signals must check `prefill_data.get()` and only fire once (when value transitions from `None` to `Some`). Use a `RwSignal<bool>` `prefill_applied` guard to prevent re-seeding when the user edits fields.

```rust
let prefill_applied = RwSignal::new(false);

Effect::new(move |_| {
    if prefill_applied.get() { return; }
    if let Some(Ok(Some(draft))) = prefill_data.get() {
        // seed signals from draft
        set_opponent.set(draft.opponent.clone().unwrap_or_default());
        // ... etc
        prefill_applied.set(true);
        // Also set champs_locked = true
        set_champs_locked.set(true);
    }
});
```

**Confidence:** HIGH — follows existing signal seeding pattern (`load_plan` Callback in `game_plan.rs` lines 501-519).

### Pattern 3: CTA Navigation via `format!` URL Construction

**What:** "Prep for This Draft" builds a URL with `format!("/game-plan?draft_id={}", draft_id)` and navigates via `window.location().set_href()`. Same hard-nav pattern used for all navigation in the codebase.

**Why hard-nav over `<A>` link:** The button must first check for duplicates (see Pattern 5). Because the duplicate check is async (server call), it must be done in `spawn_local`, which precludes a static `<A>` link. The CTA is always a `<button>` with an `on:click` handler.

```rust
on:click=move |_| {
    let draft_id = d_id.clone();
    leptos::task::spawn_local(async move {
        match check_draft_has_game_plan(draft_id.clone()).await {
            Ok(Some(existing_plan_id)) => {
                // Show duplicate prompt (signal-driven modal)
                set_duplicate_plan_id.set(Some((draft_id, existing_plan_id)));
            }
            Ok(None) => {
                #[cfg(feature = "hydrate")]
                if let Some(win) = web_sys::window() {
                    let _ = win.location().set_href(
                        &format!("/game-plan?draft_id={draft_id}")
                    );
                }
            }
            Err(e) => set_save_result.set(Some(format!("Error: {e}"))),
        }
    });
}
```

**Confidence:** HIGH — direct application of established `spawn_local` + `set_href` pattern from `game_plan.rs:1168-1184`.

### Pattern 4: Back-Reference Badges

**What:** Small `<span>` or `<a>` badges using semantic tokens from the theme. For the game plan page: when `draft_id` signal is non-empty, render a badge linking back to `/draft`. For the draft list: when a draft has ≥1 linked game plan, render a badge.

**Badge design (using existing semantic tokens):**
```rust
// Source draft badge on game plan page
{move || {
    let did = draft_id.get();
    if did.is_empty() {
        view! { <span></span> }.into_any()
    } else {
        view! {
            <a
                href=format!("/draft?load={did}")
                class="inline-flex items-center gap-1 bg-surface border border-outline/50 \
                       text-muted text-xs rounded px-2 py-1 hover:text-primary hover:border-accent/50 \
                       transition-colors"
            >
                <span class="text-accent">"Draft"</span>
                {did}
            </a>
        }.into_any()
    }
}}
```

**Note on draft detail → game plan badges:** Requires knowing how many game plans reference a given draft ID. This needs the new `get_game_plans_for_draft` db query (see New DB Queries section).

**Confidence:** HIGH — pure Leptos view pattern, same tokens as rest of codebase.

### Pattern 5: Duplicate Game Plan Guard

**What:** Before navigating to `/game-plan?draft_id=X`, check if any game plan already has `draft_id = X`. If yes, show an inline prompt with two choices: "View existing" or "Create new". This is driven by a `RwSignal<Option<(String, String)>>` (draft_id, existing_plan_id) rather than a modal component.

**Implementation approach:** Inline conditional view in the draft list item. When `duplicate_plan_id` signal is `Some`, replace the "Prep for This Draft" button with two buttons ("View Game Plan" + "Create New Plan"). Setting the signal back to `None` dismisses.

**Confidence:** HIGH — pure signal pattern, no new component needed.

### Pattern 6: Game Plan → Post-Game "Review This Game" CTA

**What:** The existing `start_post_game_review(plan_id, draft_id)` server function in `game_plan.rs` (lines 296-334) already creates a `PostGameLearning` record and the CTA button already navigates to `/post-game`. The change needed: navigate to `/post-game?review_id={id}` instead of `/post-game`, so the post-game page opens the specific review.

Currently the button navigates to `/post-game` without an ID, and the post-game page shows a list. The improvement is passing the new review ID in the URL so the page opens in edit mode for that review directly.

**Confidence:** HIGH — existing server fn returns the new review ID (`Ok(_review_id)` at line 1173, currently discarded).

### Recommended Project Structure Changes

No new files required. Changes go into:
```
src/
├── pages/
│   ├── draft.rs          # Add CTA buttons to draft list items + draft detail loaded state
│   ├── game_plan.rs      # Add use_query_map, prefill Resource, champion lock UI, back-ref badges, "Link to draft" dropdown
│   └── post_game.rs      # Add use_query_map for ?review_id= and ?plan_id=, back-ref badges
└── server/
    └── db.rs             # Add get_draft_for_prefill(), get_game_plans_for_draft()
```

---

## New DB Queries Required

### `get_draft_for_prefill(db, draft_id) -> DbResult<Option<Draft>>`

Fetches a single draft by ID with its actions. The app already has `list_drafts()` which fetches all team drafts — but for prefill we need to fetch a single draft by ID without a team scope (the user is already authenticated, the draft_id came from their own draft list).

Pattern (following existing `list_drafts` structure):
```rust
pub async fn get_draft_for_prefill(db: &Surreal<Db>, draft_id: &str) -> DbResult<Option<Draft>> {
    let key = draft_id.strip_prefix("draft:").unwrap_or(draft_id).to_string();
    let mut r = db
        .query("SELECT * FROM type::record('draft', $key); \
                SELECT * FROM draft_action WHERE draft = type::record('draft', $key) ORDER BY `order` ASC")
        .bind(("key", key))
        .await?;
    let drafts: Vec<DbDraft> = r.take(0).unwrap_or_default();
    let actions: Vec<DbDraftAction> = r.take(1).unwrap_or_default();
    Ok(drafts.into_iter().next().map(|d| {
        let mut draft = Draft::from(d);
        draft.actions = actions.into_iter().map(DraftAction::from).collect();
        draft
    }))
}
```

### `get_game_plans_for_draft(db, draft_id) -> DbResult<Vec<GamePlan>>`

Needed for the back-reference badge on the draft list ("1 game plan") and the duplicate check. Simple SELECT with a draft FK filter:

```rust
pub async fn get_game_plans_for_draft(db: &Surreal<Db>, draft_id: &str) -> DbResult<Vec<GamePlan>> {
    let mut r = db
        .query("SELECT * FROM game_plan WHERE draft = $draft_id")
        .bind(("draft_id", draft_id.to_string()))
        .await?;
    let rows: Vec<DbGamePlan> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(GamePlan::from).collect())
}
```

**Note on `draft` field in `DbGamePlan`:** The `save_game_plan` query binds `draft = $draft_id` where `$draft_id` is the `Option<String>` from the model (e.g. `"draft:abc123"`). This means the `draft` field stores the raw string ID (not a `RecordId`). The WHERE clause `WHERE draft = $draft_id` with `bind(("draft_id", "draft:abc123".to_string()))` should work. Verify against `schema.surql` — `draft` field may be typed as `option<record<draft>>` (record reference) or `option<string>`. If it's a record type, the filter must use `type::record()`. **Check `schema.surql` before writing this query.**

**Confidence:** HIGH for structure, MEDIUM for exact `draft` field type — needs `schema.surql` verification during implementation.

---

## Champion Extraction from DraftAction

To prefill `our_champions` and `enemy_champions` on the game plan, extract from `Draft.actions` by filtering on `side` vs `our_side`:

```rust
// On the server side, in get_draft_prefill_data server fn:
let our_side = &draft.our_side; // "blue" or "red"
let their_side = if our_side == "blue" { "red" } else { "blue" };

let our_picks: Vec<String> = draft.actions.iter()
    .filter(|a| a.side == *our_side && a.phase.contains("pick"))
    .sorted_by_key(|a| a.order)
    .map(|a| a.champion.clone())
    .collect();

let their_picks: Vec<String> = draft.actions.iter()
    .filter(|a| a.side == their_side && a.phase.contains("pick"))
    .sorted_by_key(|a| a.order)
    .map(|a| a.champion.clone())
    .collect();
```

The `DraftAction.phase` field values in the codebase are `"ban1"`, `"pick1"`, `"ban2"`, `"pick2"`. The filter `.contains("pick")` is correct. No need for `itertools` — use `sort_unstable_by_key` on a `Vec` after collecting.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| URL query param parsing | Custom `window.location().search` parsing in WASM | `leptos_router::hooks::use_query_map()` | Built-in, reactive, SSR-compatible |
| Page-to-page navigation with params | `leptos_router::navigate()` or SPA transitions | `window.location().set_href()` with format URL | Established hard-nav pattern in this codebase; avoids stale signal issues from SPA navigation |
| Champion extract logic duplication | Copy logic into component | Server fn `get_draft_for_prefill` returns a `DraftPrefillData` struct with pre-extracted champion lists | Keeps extraction logic SSR-only, testable |

---

## Common Pitfalls

### Pitfall 1: Reading `use_query_map()` Outside Component Body

**What goes wrong:** `use_query_map()` panics or returns an empty map if called outside a reactive context (e.g., in an async block or event handler).

**Why it happens:** It calls `use_url_raw()` which requires the router context to be present on the call stack.

**How to avoid:** Call `use_query_map()` once in the component body, capture the `Memo<ParamsMap>` in a local binding, then use that binding inside closures and Resource key functions.

```rust
// CORRECT — called in component body
let query = use_query_map();
let draft_id_key = move || query.read().get("draft_id");
let prefill = Resource::new(draft_id_key, |id| async move { ... });

// WRONG — called inside async block
let prefill = Resource::new(|| (), |_| async move {
    let query = use_query_map(); // PANIC: no reactive context
});
```

**Confidence:** HIGH — standard Leptos reactive context rule.

### Pitfall 2: Prefill Effect Firing on Every Signal Change

**What goes wrong:** The Effect that seeds form signals from prefill data re-triggers every time a user edits a field (because editing a signal that was seeded changes it, which re-triggers the Effect if it tracked that signal).

**Why it happens:** `Effect::new` tracks every signal read inside it. If the Effect reads `draft_id` and `draft_id` changes when the user types, it will re-fire.

**How to avoid:** Use a `prefill_applied: RwSignal<bool>` guard. Set it to `true` after the first seed. The Effect short-circuits on `if prefill_applied.get() { return; }` at the top.

**Warning signs:** Champion slot resets to prefill value every time user tries to change it.

### Pitfall 3: `draft` Field Type in SurrealDB (String vs RecordId)

**What goes wrong:** `save_game_plan` in `db.rs` binds `draft = $draft_id` where `$draft_id` is `Option<String>`. If `schema.surql` defines the field as `option<record<draft>>`, SurrealDB 3.x may store it as a proper record reference. The `WHERE draft = $draft_id` filter with a plain string may not match.

**How to avoid:** Before writing `get_game_plans_for_draft`, check `schema.surql` for the `game_plan` table's `draft` field type. If it's `option<record<draft>>`, use `WHERE draft = type::record('draft', $key)` with the stripped key. If it's `option<string>`, the plain string filter works.

**Warning signs:** `get_game_plans_for_draft` returns empty results even when plans exist.

**Confidence:** MEDIUM — field type is ambiguous from code inspection alone; `schema.surql` is ground truth.

### Pitfall 4: Hard-Nav Inside `spawn_local` Under SSR Feature Flag

**What goes wrong:** `window.location().set_href()` calls must be behind `#[cfg(feature = "hydrate")]`. If not guarded, the SSR build fails because `web_sys::window()` is not available server-side.

**How to avoid:** Every `web_sys::window()` call must be inside `#[cfg(feature = "hydrate")]`. This is already the universal pattern in the codebase (verified across 19 instances in `grep` output).

### Pitfall 5: Duplicate Check Race (User Clicks Twice)

**What goes wrong:** If the user double-clicks "Prep for This Draft", two `spawn_local` tasks race to check duplicates and navigate. Both may navigate to `/game-plan?draft_id=X` simultaneously.

**How to avoid:** Use a `RwSignal<bool>` `cta_loading` guard. Set to `true` on click, reset on completion. Disable the button while loading: `disabled=move || cta_loading.get()`.

---

## Code Examples

### Reading URL Query Param (Verified from `leptos_router-0.8.12`)

```rust
// Source: ~/.cargo/registry/.../leptos_router-0.8.12/src/hooks.rs
use leptos_router::hooks::use_query_map;

#[component]
pub fn GamePlanPage() -> impl IntoView {
    let query = use_query_map();
    // query is Memo<ParamsMap>; .read() dereferences to ParamsMap
    // ParamsMap::get(&str) -> Option<String>

    let prefill_data = Resource::new(
        move || query.read().get("draft_id"),  // key: Option<String>
        |draft_id_opt| async move {
            match draft_id_opt {
                Some(id) if !id.is_empty() => get_draft_for_prefill(id).await,
                _ => Ok(None),
            }
        },
    );
    // ...
}
```

### Champion Lock Toggle Pattern

```rust
// Locked state: show champion name with lock icon + "Edit" button
// Edit state: show ChampionAutocomplete normally
let (champs_locked, set_champs_locked) = signal(false);

{move || {
    let slot_signal = our_champ_signals[i].clone();
    if champs_locked.get() {
        view! {
            <div class="flex items-center gap-2 bg-surface/50 border border-outline/50 rounded-lg px-3 py-2">
                <span class="text-primary text-sm flex-1">{slot_signal.get()}</span>
                <button
                    class="text-muted hover:text-accent text-xs"
                    on:click=move |_| set_champs_locked.set(false)
                >"Edit"</button>
            </div>
        }.into_any()
    } else {
        view! {
            <ChampionAutocomplete
                value=slot_signal
                on_change=Callback::new(move |v| slot_signal.set(v))
                champions=champions_stored
            />
        }.into_any()
    }
}}
```

### Existing `start_post_game_review` Pattern (from `game_plan.rs:1168-1184`)

The existing button navigates to `/post-game` without passing the new review ID. The fix is to use the returned ID:

```rust
// CURRENT (lines 1172-1176):
match start_post_game_review(pid, did).await {
    Ok(_review_id) => {  // review_id is discarded!
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/post-game");
        }
    }
}

// IMPROVED (Phase 1 change):
match start_post_game_review(pid, did).await {
    Ok(review_id) => {
        #[cfg(feature = "hydrate")]
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href(
                &format!("/post-game?review_id={review_id}")
            );
        }
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| SPA routing with Leptos `navigate()` | Hard-nav `window.location().set_href()` | Established pattern in this codebase | Prevents stale signal state after navigation; required for auth state refresh |
| Draft list "Open" button (loads in-place) | "Prep for This Draft" CTA navigates to `/game-plan?draft_id=X` | Phase 1 (this work) | Users no longer need to copy-paste draft IDs |
| `/post-game` navigate without ID | `/post-game?review_id=X` | Phase 1 (this work) | Post-game page opens immediately to the correct review |

---

## Open Questions

1. **`draft` field type in `game_plan` schema**
   - What we know: `save_game_plan` binds `draft = $draft_id` where `$draft_id` is `Option<String>`. The `DbGamePlan` struct has `draft: Option<String>` (line 1696 in `db.rs`).
   - What's unclear: Whether `schema.surql` defines the field as `option<string>` or `option<record<draft>>`. The `save_game_plan` query stores the raw full ID string (e.g., `"draft:abc123"`), not a cast `type::record(...)`. So filtering by string equality should work.
   - Recommendation: Check `schema.surql` first during Wave 0. If `draft` is typed as `option<record<draft>>`, the stored value may be a record reference — use `type::record('draft', $key)` in the WHERE clause.

2. **"Link to draft" dropdown for retroactive linking**
   - What we know: Needs to list all team drafts in a `<select>` or inline dropdown and update `game_plan.draft_id` via `update_plan`.
   - What's unclear: Whether to use a `<select>` element (simplest) or a `ChampionAutocomplete`-style text filter. The CONTEXT.md leaves this to Claude's discretion.
   - Recommendation: Use a plain `<select>` with draft names as options. The draft list is small (< 50 items typically). No need for autocomplete filtering.

3. **"Draft detail" view on the draft page**
   - What we know: The draft page is a single-panel editor. There's no separate "draft detail" route — "detail" means the loaded state (`loaded_draft_id.is_some()`).
   - What's unclear: Whether the "Prep for This Draft" CTA in the "detail view" (per CONTEXT.md) appears in the editor panel when a draft is loaded, or in the draft list item row.
   - Recommendation: Add the CTA to both locations: (a) each draft list row as a small secondary button next to "Open", and (b) the loaded editor header area when `loaded_draft_id.is_some()`.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust unit + integration tests (`cargo test --features ssr`) + Playwright E2E |
| Config file | `Cargo.toml` dev-dependencies + `e2e/playwright.config.ts` |
| Quick run command | `cargo test --features ssr --lib` |
| Full suite command | `cargo test --features ssr` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PIPE-01 | `get_draft_for_prefill` returns correct champions and side | integration | `cargo test --features ssr --test db_game_plans::test_get_draft_for_prefill` | Wave 0 |
| PIPE-01 | `get_game_plans_for_draft` returns plans filtered by draft FK | integration | `cargo test --features ssr --test db_game_plans::test_get_game_plans_for_draft` | Wave 0 |
| PIPE-01 | Navigating to `/game-plan?draft_id=X` pre-fills the editor | e2e (manual-verify) | `just e2e` — `pages.spec.ts` smoke + manual navigation | Existing smoke only |
| PIPE-01 | "Prep for This Draft" button exists on draft list | e2e | `just e2e` (update `pages.spec.ts` draft content check) | Existing file, needs update |
| PIPE-01 | Back-reference badge on game plan links to draft | e2e (manual-verify) | `just e2e` + MCP browser verification | New test case |
| PIPE-01 | Duplicate draft → game plan prompt appears | e2e (manual-verify) | MCP browser verification | Manual only |

### Sampling Rate

- **Per task commit:** `cargo test --features ssr --lib`
- **Per wave merge:** `cargo test --features ssr`
- **Phase gate:** Full suite green + MCP browser verification of the end-to-end flow before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/db_game_plans.rs` — new integration test file for `get_draft_for_prefill` and `get_game_plans_for_draft`
- [ ] Framework install: None required (tokio + surrealdb already in dev-dependencies)

---

## Sources

### Primary (HIGH confidence)

- Codebase inspection: `src/pages/game_plan.rs` — server functions, component signals, existing `start_post_game_review` pattern
- Codebase inspection: `src/pages/draft.rs` — draft list rendering, loaded draft state, `spawn_local` navigation pattern
- Codebase inspection: `src/pages/post_game.rs` — existing `game_plan_id` and `draft_id` linkage
- Codebase inspection: `src/server/db.rs` — `list_drafts` batch query pattern, `save_game_plan` FK binding, `DbGamePlan` struct shape
- Codebase inspection: `src/models/draft.rs` and `src/models/game_plan.rs` — FK fields confirmed
- `~/.cargo/registry/.../leptos_router-0.8.12/src/hooks.rs` — `use_query_map() -> Memo<ParamsMap>` verified
- `~/.cargo/registry/.../leptos_router-0.8.12/src/params.rs` — `ParamsMap::get(&str) -> Option<String>` verified
- `.planning/codebase/CONVENTIONS.md` — signal patterns, SSR import placement, error handling
- `.planning/research/ARCHITECTURE.md` — Layer 1 URL deep-link pattern documented (Phase 1 is this layer)

### Secondary (MEDIUM confidence)

- `.planning/research/STACK.md` — Pattern 6 (Draft → Game Plan pipeline, no schema change needed) — based on direct code inspection

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all patterns verified from installed crate source
- Architecture: HIGH — all patterns extend existing established patterns in the codebase
- New DB queries: HIGH (structure) / MEDIUM (`draft` field type — needs `schema.surql` verification)
- Pitfalls: HIGH — derived from existing codebase patterns and Leptos reactive rules

**Research date:** 2026-03-14
**Valid until:** 2026-05-14 (stable stack; Leptos 0.8 API confirmed from installed source)
