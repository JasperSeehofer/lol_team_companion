# Phase 14: Personal Learnings Journal - Research

**Researched:** 2026-03-27
**Domain:** Leptos 0.8 + SurrealDB 3.x — new user-scoped page with CRUD, filtering, inline expansion, multi-route form, and match-detail integration
**Confidence:** HIGH

## Summary

Phase 14 adds a Personal Learnings Journal — a fully user-scoped (not team-scoped) section of the app. The implementation is a clean extension of patterns already established in the codebase. The data model, DB query patterns, form interaction model, and UI components are all well-understood from prior phases.

The two new routes are `/personal-learnings` (browse + filter) and `/personal-learnings/new` (create + edit form). The form receives context via URL query params when launched from match detail, following the exact pattern already used in `post_game.rs` and `game_plan.rs`. Inline card expansion on the browse page uses a `RwSignal<Option<String>>` holding the expanded ID — the same toggle pattern used in `draft.rs` for saved-draft expansion.

The only meaningfully new design element is the multi-step form with a type selector (General / Champion / Matchup) controlling conditional field visibility. This is straightforward `RwSignal` + conditional rendering using `.into_any()` (per CLAUDE.md rule 19). No animation is required beyond `transition-all` on the class swap.

**Primary recommendation:** Implement as a single new page module (`src/pages/personal_learnings.rs`) with two internal components — the browse/filter page and the form page — sharing signals via props. Use the `post_game.rs` CRUD pattern verbatim and the `stats.rs` filter dropdown pattern verbatim.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Entry Structure**
- D-01: Learning type selector as first step — user picks General, Champion, or Matchup. This determines which fields appear.
- D-02: Guided sections: "What happened", "What I learned", "Next time I will..." — all three required to save.
- D-03: Auto-generated editable title from champion + matchup + date (e.g. "Zed vs Ahri — Mar 27"). User can override.
- D-04: Full CRUD — learnings are editable and deletable after creation.
- D-05: When created from a match, auto-attach win/loss result and match reference (riot match ID). Standalone entries skip this.

**Tag System**
- D-06: Predefined category tags (multi-select): Laning, Teamfighting, Macro / Rotations, Vision, Trading, Wave Management, Objective Control, Mental / Tilt.
- D-07: Users can select multiple tags per learning entry.

**Browse & Filter UX**
- D-08: Card grid layout — each card shows title, type badge, champion icon(s), tags as chips, win/loss badge, and a preview of the first section.
- D-09: Filter bar above cards with dropdowns: Type (All/General/Champion/Matchup), Champion, Tag. Filters apply instantly.
- D-10: Sort control with "Newest first" (default) and "By champion" options.
- D-11: Clicking a card expands inline to show full content with edit/delete actions. No separate detail page navigation.

**Match-to-Learning Flow**
- D-12: "Add Learning" button on match detail scoreboard pre-fills: type as Matchup, champion, opponent, win/loss, and match reference.
- D-13: "Add Learning" on timeline event detail panel pre-fills all of D-12 plus event timestamp and event name. Event type auto-maps to a tag suggestion.
- D-14: Learning entries store optional game timestamp and event name when created from timeline events.

**Navigation**
- D-15: Top-level "Learnings" link in the main nav bar (auth-required).

### Claude's Discretion
- Event-type-to-tag mapping logic (which timeline events map to which predefined tags)
- Card grid responsive breakpoints (2 vs 3 columns)
- Inline expand animation style
- Empty state messaging and CTA
- How the sort-by-champion grouping is visually presented (headers, separators)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LEARN-01 | User can write personal learnings tagged by champion and matchup | New `personal_learning` table + CRUD server fns + form page at `/personal-learnings/new` |
| LEARN-02 | User can create a learning from match detail with one click (pre-fills champion/matchup) | URL query param passing (`?champion=&opponent=&match_id=&result=&event_ts=&event_name=`) + update to match_detail.rs "Add Learning" CTA and timeline event detail panel |
| LEARN-03 | User can browse and filter learnings by champion, matchup, or tag | Browse page at `/personal-learnings` with client-side filter signals and inline card expansion |
</phase_requirements>

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Leptos | 0.8 | Reactive components, server functions | Project standard |
| SurrealDB | 3.x (SurrealKV) | Persistent storage for personal_learning table | Project standard |
| Tailwind CSS v4 | v4 | Styling with semantic tokens | Project standard |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| leptos_router::hooks::use_query_map | (bundled) | Read URL query params on form page | Form pre-fill from match detail |
| leptos_router::hooks::use_params_map | (bundled) | Read route params (e.g. edit ID) | Edit mode routing |
| surrealdb-types-derive SurrealValue | (bundled with surrealdb) | Derive on DB result structs | DbPersonalLearning struct |
| web_sys | (bundled) | Window navigation for auth redirect | Auth guard pattern |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Client-side signal filtering | Server-side filtered fetch per filter change | Client-side is simpler, already loads all learnings once — sufficient for a personal journal (rarely exceeds 200 entries) |
| Single page with conditional form panel | Two separate routes (/personal-learnings + /personal-learnings/new) | Two routes enables direct linking from match detail; matches UI-SPEC decision |
| Inline edit in expanded card | Navigate to /personal-learnings/new?edit={id} | UI-SPEC leaves this at discretion; navigate-to-edit is simpler to implement and consistent with other pages |

**No npm packages required.** Standalone Tailwind binary handles CSS compilation.

---

## Architecture Patterns

### Recommended Project Structure

```
src/
├── models/
│   ├── personal_learning.rs    # NEW — PersonalLearning struct (shared SSR+WASM)
│   └── mod.rs                  # Add: pub mod personal_learning;
├── pages/
│   ├── personal_learnings.rs   # NEW — browse page + form page components + server fns
│   └── mod.rs                  # Add: pub mod personal_learnings;
├── server/
│   └── db.rs                   # Add: CRUD fns for personal_learning table
├── components/
│   └── nav.rs                  # Update: add "Learnings" nav link
├── app.rs                      # Add: two Route entries
schema.surql                    # Add: personal_learning table definition
e2e/tests/pages.spec.ts         # Add: /personal-learnings to AUTHED_PAGES
```

### Pattern 1: User-Scoped DB CRUD (mirrors post_game_learning)

**What:** All DB queries filter by `user` RecordId, not `team`. Return `Ok(Vec::new())` when no records exist.
**When to use:** Every server function in this page.

```rust
// Source: src/server/db.rs — list_post_game_learnings pattern adapted for user scope
pub async fn list_personal_learnings(db: &Surreal<Db>, user_id: &str) -> DbResult<Vec<PersonalLearning>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut r = db
        .query("SELECT * FROM personal_learning WHERE user = type::record('user', $user_key) ORDER BY created_at DESC")
        .bind(("user_key", user_key))
        .await?;
    let rows: Vec<DbPersonalLearning> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(PersonalLearning::from).collect())
}
```

### Pattern 2: URL Query Param Pre-fill (mirrors post_game.rs + game_plan.rs)

**What:** `use_query_map()` in the form page reads pre-fill params on mount, initialises signals.
**When to use:** `/personal-learnings/new` component, on first render.

```rust
// Source: src/pages/post_game.rs:367 and game_plan.rs:535 — identical pattern
use leptos_router::hooks::use_query_map;
let query = use_query_map();
let url_champion = move || query.read().get("champion").unwrap_or_default();
let url_opponent = move || query.read().get("opponent").unwrap_or_default();
let url_match_id = move || query.read().get("match_id").unwrap_or_default();
let url_result   = move || query.read().get("result").unwrap_or_default();
let url_event_ts = move || query.read().get("event_ts").unwrap_or_default();
let url_event_name = move || query.read().get("event_name").unwrap_or_default();
```

Signals are populated in an `Effect::new` that runs once on mount (triggered by `query.read()` read).

### Pattern 3: Inline Card Expansion Toggle

**What:** `RwSignal<Option<String>>` stores the currently expanded learning ID. Card click toggles it.
**When to use:** Browse page (`/personal-learnings`).

```rust
// Source: established pattern (draft.rs uses similar for expanded draft details)
let expanded_id: RwSignal<Option<String>> = RwSignal::new(None);

// In card on:click handler:
on:click=move |_| {
    let id = card_id.clone();
    let current = expanded_id.get_untracked();
    if current.as_deref() == Some(&id) {
        expanded_id.set(None);
    } else {
        expanded_id.set(Some(id));
    }
}

// Card class toggle:
class=move || if expanded_id.get().as_deref() == Some(&id_for_class) {
    "bg-accent/10 border border-accent/30 rounded-xl p-4 transition-all cursor-pointer"
} else {
    "bg-elevated/30 border border-divider/30 rounded-xl p-4 hover:bg-overlay/30 transition-all cursor-pointer"
}
```

### Pattern 4: Client-Side Filter + Sort (mirrors stats.rs champion filter)

**What:** All learnings loaded once in a `Resource`. Filter/sort signals drive a `move ||` closure that derives the visible list. No server round-trip per filter change.
**When to use:** Browse page filter bar.

```rust
// Signals
let (filter_type, set_filter_type) = signal("all".to_string());
let (filter_champion, set_filter_champion) = signal("all".to_string());
let (filter_tag, set_filter_tag) = signal("all".to_string());
let (sort_mode, set_sort_mode) = signal("newest".to_string());

// Derived visible list inside Suspense/move ||
let visible: Vec<PersonalLearning> = learnings.iter()
    .filter(|l| filter_type.get() == "all" || l.learning_type == filter_type.get())
    .filter(|l| filter_champion.get() == "all" || l.champion.as_deref() == Some(filter_champion.get().as_str()))
    .filter(|l| filter_tag.get() == "all" || l.tags.contains(&filter_tag.get()))
    .collect();
```

Sort by champion uses `BTreeMap<String, Vec<_>>` collection (per UI-SPEC note 4 — General learnings group under "General" at top).

### Pattern 5: Conditional Form Fields via Type Signal

**What:** Learning type `RwSignal<String>` ("general" | "champion" | "matchup") controls whether champion/opponent autocomplete fields render.
**When to use:** Form page.

```rust
// Divergent view branches require .into_any() per CLAUDE.md rule 19
{move || match learning_type.get().as_str() {
    "matchup" => view! {
        <div>
            <ChampionAutocomplete ... />  // champion field
            <ChampionAutocomplete ... />  // opponent field
        </div>
    }.into_any(),
    "champion" => view! {
        <ChampionAutocomplete ... />  // champion field only
    }.into_any(),
    _ => view! { <span /> }.into_any(),
}}
```

### Pattern 6: DbPersonalLearning Struct (mirrors DbPostGameLearning)

**What:** `Db*` struct with `RecordId` fields for SurrealDB deserialization, then converted to app-facing struct with `String` IDs.
**When to use:** All DB query result deserialization.

```rust
// Source: src/server/db.rs:1913 — DbPostGameLearning pattern
#[derive(Debug, Deserialize, SurrealValue)]
struct DbPersonalLearning {
    id: RecordId,
    user: RecordId,
    title: String,
    learning_type: String,
    champion: Option<String>,
    opponent: Option<String>,
    what_happened: String,
    what_i_learned: String,
    next_time: String,
    tags: Vec<String>,
    win_loss: Option<String>,
    match_riot_id: Option<String>,
    game_timestamp_ms: Option<i64>,
    event_name: Option<String>,
    created_at: Option<String>,  // serialized as string for app model
}
```

### Anti-Patterns to Avoid
- **Team-scoping personal learnings:** Never filter by `team_id`. User ID is the only scope. No `get_user_team_id()` call needed.
- **Returning Err for empty list:** If user has no learnings, return `Ok(Vec::new())` — per CLAUDE.md rule 44.
- **Debounced auto-save on the form:** Per UI-SPEC note 7, this form uses explicit save only. Do NOT apply the auto-save Effect pattern from `tree_drafter.rs`.
- **Deserializing DB results as serde_json::Value:** Always use `DbPersonalLearning` struct — per CLAUDE.md rule 3.
- **`type::thing()` in SurQL:** Use `type::record('personal_learning', $key)` — per CLAUDE.md rule 1.
- **Lazy signal reads inside Closure::once:** Not applicable here (no auto-save timer), but note for general hygiene.
- **`.unwrap()` in WASM event handlers:** Per CLAUDE.md rule 35 — use `if let Some(...)` everywhere in client-side code.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Champion name autocomplete | Custom input+dropdown | `ChampionAutocomplete` from `src/components/champion_autocomplete.rs` | Already handles keyboard nav, champion list fetching, icon display |
| Error feedback | Custom error div | `ErrorBanner` from `src/components/ui.rs` | Consistent styling, already used on every page |
| Success toast | Custom toast | `ToastContext` / `ToastKind::Success` from `src/components/ui.rs` | Already wired into `ToastProvider` in app.rs |
| Filter dropdowns | Custom dropdown component | `<select>` with `bg-surface/50 border border-outline/50` class (stats.rs pattern) | Consistent styling, no component needed |
| Champion icon URL | Custom helper | `champion_icon_url()` fn from `match_detail.rs` or inline `format!("https://ddragon.leagueoflegends.com/cdn/15.6.1/img/champion/{name}.png")` | Established DDragon URL pattern |

**Key insight:** Every UI primitive for this page already exists in the codebase. No new shared components need to be built.

---

## Common Pitfalls

### Pitfall 1: `ORDER BY` on `created_at` field not included in partial SELECT

**What goes wrong:** SurrealDB 3.x rejects `ORDER BY created_at` if the field is not in a partial `SELECT` clause.
**Why it happens:** A `SELECT *` query avoids this, but if only specific fields are selected the `ORDER BY` field must be in the `SELECT` list.
**How to avoid:** Use `SELECT *` for `personal_learning` queries (no projection needed). Per CLAUDE.md rule 40.
**Warning signs:** SurrealDB parse error in server logs on startup.

### Pitfall 2: `into_any()` missing on divergent view branches in type selector

**What goes wrong:** Rust type error — Leptos view branches must return the same concrete type. The type selector's conditional rendering (General / Champion / Matchup) returns structurally different views.
**Why it happens:** CLAUDE.md rule 19 — when `if/else` or `match` arms return different view types, each arm must call `.into_any()`.
**How to avoid:** Always call `.into_any()` on every branch of the type selector `match`.
**Warning signs:** Compile error mentioning mismatched types in `view!` macro.

### Pitfall 3: `Clone before multiple closures` on `Vec<String>` tags

**What goes wrong:** Borrow checker error when the same `Vec<String>` (tags, champion list for filter dropdown) is captured by multiple `move` closures.
**Why it happens:** CLAUDE.md rule 18 — `Vec<T>` does not implement `Copy`.
**How to avoid:** Clone before each closure: `let tags_for_chips = tags.clone(); let tags_for_filter = tags.clone();`
**Warning signs:** "cannot move out of `tags`, as it is used here" compile error.

### Pitfall 4: User ID not stripped of table prefix before `type::record()`

**What goes wrong:** SurrealDB record not found or parse error.
**Why it happens:** CLAUDE.md rule 2 — `type::record('user', $key)` expects just the UUID part, not `"user:abc123"`.
**How to avoid:** Strip prefix: `user_id.strip_prefix("user:").unwrap_or(&user_id).to_string()` before binding.
**Warning signs:** Empty result set on queries that should return data.

### Pitfall 5: Missing `#[serde(default)]` on optional fields in PersonalLearning model

**What goes wrong:** Deserialization panics or fails for records written before a field was added.
**Why it happens:** `serde` requires `#[serde(default)]` on fields that may be absent in existing DB records.
**How to avoid:** Apply `#[serde(default)]` on all `Option<T>` fields and `Vec<T>` fields in `PersonalLearning` that might be missing from older records. See `PostGameLearning.win_loss` as reference.
**Warning signs:** "missing field" deserialization error in server logs.

### Pitfall 6: `show_recalls` / `show_teamfights` signals not present on browse page

**What goes wrong:** Forgetting that match_detail.rs defines its timeline filter signals (`show_recalls`, `show_teamfights`) locally — they are not available in scope on the learnings page.
**Why it happens:** N/A — these are unrelated pages. Mentioned to clarify that the D-13 "Add Learning" CTA in the timeline event detail panel is a small addition to `match_detail.rs`, not a shared state problem.
**How to avoid:** The timeline event "Add Learning" CTA needs `user_champion`, `opponent_champion`, `user_win`, and `match_id` variables — all already in scope within the `Ok(d) =>` branch of `match_detail.rs`.
**Warning signs:** Variables not in scope compile errors if the CTA is placed outside the `Ok(d)` branch.

### Pitfall 7: Title auto-generation depends on champion/opponent signal values — must update on signal change

**What goes wrong:** Title field shows stale auto-generated text when type or champion changes.
**Why it happens:** If title is pre-computed once and stored in a signal, it won't update when the user changes champion or type.
**How to avoid:** Derive the auto-title string inside a `move ||` closure (reactive) and use it as the `prop:value` of the title input. Only commit to a `title_signal` when the user explicitly edits it. Per CLAUDE.md rule 21 on `prop:value` for controlled inputs.
**Warning signs:** Title shows "— Mar 27" without champion name after selecting a champion.

---

## Code Examples

### SurrealDB Schema for personal_learning table

```sql
-- Source: schema.surql pattern — always use DEFINE FIELD IF NOT EXISTS (CLAUDE.md rule 30)
DEFINE TABLE IF NOT EXISTS personal_learning SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON personal_learning TYPE record<user>;
DEFINE FIELD IF NOT EXISTS title ON personal_learning TYPE string;
DEFINE FIELD IF NOT EXISTS learning_type ON personal_learning TYPE string;
DEFINE FIELD IF NOT EXISTS champion ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS opponent ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS what_happened ON personal_learning TYPE string;
DEFINE FIELD IF NOT EXISTS what_i_learned ON personal_learning TYPE string;
DEFINE FIELD IF NOT EXISTS next_time ON personal_learning TYPE string;
DEFINE FIELD IF NOT EXISTS tags ON personal_learning TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS win_loss ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS match_riot_id ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS game_timestamp_ms ON personal_learning TYPE option<int>;
DEFINE FIELD IF NOT EXISTS event_name ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON personal_learning TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS personal_learning_user ON personal_learning FIELDS user;
```

### PersonalLearning model struct (src/models/personal_learning.rs)

```rust
// Source: src/models/game_plan.rs PostGameLearning pattern
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PersonalLearning {
    pub id: Option<String>,
    pub user_id: String,
    pub title: String,
    pub learning_type: String,   // "general" | "champion" | "matchup"
    pub champion: Option<String>,
    pub opponent: Option<String>,
    pub what_happened: String,
    pub what_i_learned: String,
    pub next_time: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub win_loss: Option<String>,
    #[serde(default)]
    pub match_riot_id: Option<String>,
    #[serde(default)]
    pub game_timestamp_ms: Option<i64>,
    #[serde(default)]
    pub event_name: Option<String>,
    pub created_at: Option<String>,
}
```

### create_personal_learning in db.rs

```rust
// Source: src/server/db.rs save_post_game_learning pattern
pub async fn create_personal_learning(
    db: &Surreal<Db>,
    learning: PersonalLearning,
) -> DbResult<String> {
    let user_key = learning.user_id.strip_prefix("user:").unwrap_or(&learning.user_id).to_string();
    let mut response = db.query(
        "CREATE personal_learning SET \
         user = type::record('user', $user_key), \
         title = $title, learning_type = $learning_type, \
         champion = $champion, opponent = $opponent, \
         what_happened = $what_happened, what_i_learned = $what_i_learned, \
         next_time = $next_time, tags = $tags, win_loss = $win_loss, \
         match_riot_id = $match_riot_id, game_timestamp_ms = $game_timestamp_ms, \
         event_name = $event_name"
    )
    .bind(("user_key", user_key))
    .bind(("title", learning.title))
    // ... remaining binds
    .await?;
    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create personal learning".into())),
    }
}
```

### Timeline event "Add Learning" URL construction (match_detail.rs)

```rust
// Source: match_detail.rs line ~840 — event detail panel; insert after the "Involved:" text block
// event.event_type values and their tag mappings (D-13 / Claude's Discretion):
// "ELITE_MONSTER_KILL" (DRAGON/BARON) → "Objective Control"
// "BUILDING_KILL" → "Objective Control"
// "CHAMPION_KILL" → "Teamfighting"
// "TEAMFIGHT" → "Teamfighting"

let event_tag_suggestion = match event.event_type.as_str() {
    "ELITE_MONSTER_KILL" | "BUILDING_KILL" => "Objective+Control",
    "CHAMPION_KILL" | "TEAMFIGHT" => "Teamfighting",
    _ => "",
};
let add_learning_url = format!(
    "/personal-learnings/new?champion={}&opponent={}&match_id={}&result={}&event_ts={}&event_name={}",
    user_champion_enc, opponent_champion_enc, match_id_enc,
    if user_win { "win" } else { "loss" },
    event.timestamp_ms,
    url_encode(&event_tooltip_text),
);
```

### Predefined tags constant (to be placed in personal_learnings.rs or a constant in the model)

```rust
// Source: CONTEXT.md D-06 — exact list
pub const LEARNING_TAGS: &[&str] = &[
    "Laning",
    "Teamfighting",
    "Macro / Rotations",
    "Vision",
    "Trading",
    "Wave Management",
    "Objective Control",
    "Mental / Tilt",
];
```

### Event-type to tag mapping (Claude's Discretion — D-13)

| event_type | Suggested tag(s) |
|------------|-----------------|
| ELITE_MONSTER_KILL (DRAGON, BARON, RIFT_HERALD, HORDE) | Objective Control |
| BUILDING_KILL (TOWER, INHIBITOR) | Objective Control |
| CHAMPION_KILL (first blood or multi-kill) | Teamfighting |
| CHAMPION_KILL (single, involving user) | Trading |
| TEAMFIGHT | Teamfighting |
| WARD_PLACED / WARD_KILL | Vision |
| Other / unrecognised | (no pre-selection) |

Implementation: URL query param `?tag_hint=Objective+Control` — form reads it and pre-selects that chip on mount.

---

## Integration Points

### 1. match_detail.rs — Scoreboard "Add Learning" CTA (LEARN-02)

**Location:** Line ~505-511. The stub already exists:
```rust
<a href=format!("/personal-learnings/new?champion={}&opponent={}", user_champion, opponent_champion) ...>
    "Add Learning ->"
</a>
```

**Required update:** Add `match_id` and `result` query params. The variables `user_champion`, `opponent_champion`, and `user_win` are all in scope at that point. The match_id can be extracted from `d.match_id` (already in scope in the `Ok(d)` branch).

Change the `href` to:
```rust
href=format!(
    "/personal-learnings/new?champion={}&opponent={}&match_id={}&result={}",
    user_champion, opponent_champion, d.match_id,
    if user_win { "win" } else { "loss" }
)
```
Update button class and label per UI-SPEC.

### 2. match_detail.rs — Timeline Event Detail Panel (LEARN-02, D-13)

**Location:** Lines ~840-870. The event detail panel renders a `<div>` with detail text and optionally involved player names. Add the "Add Learning" link after the involved-players paragraph.

All needed variables are in scope: `user_champion`, `opponent_champion`, `user_win`, `d.match_id` (via `d_stored`), `event.timestamp_ms`, `event.event_type`.

`url_champion` and `url_opponent` must be passed through `format!()` since URL encoding is not available without `percent_encoding` crate. Check if already depended on; otherwise use simple `str::replace(" ", "+")` for champion names (no spaces in LoL champion names except "Aurelion Sol", "Master Yi" etc — use a simple space→`+` replace).

**Note on URL encoding:** Champion names like "Aurelion Sol", "Lee Sin", "Master Yi" contain spaces. Pass champion names to the URL as-is using `format!()` — the browser will percent-encode them when reading from the `href`. Or use Rust's `percent_encoding` crate if already available. Check `Cargo.toml`.

### 3. nav.rs — "Learnings" nav link (D-15)

**Location:** Inside the `Nav` component's authenticated links section. Pattern: `<A attr:class="..." href="/personal-learnings">` matching the auth-guarded links pattern (uses `is_authed` signal, same as Stats/Draft/etc links).

### 4. app.rs — Two new Route entries

```rust
// After the /match/:id route
<Route path=path!("/personal-learnings") view=PersonalLearningsPage />
<Route path=path!("/personal-learnings/new") view=NewLearningPage />
```

Import from `pages::personal_learnings::{PersonalLearningsPage, NewLearningPage}`.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `type::thing()` in SurQL | `type::record()` | SurrealDB 2.x | Must use record(); thing() removed |
| `string()` cast in SurQL | `.to_sql()` on RecordId in Rust | SurrealDB 2.x | No string casting in SurQL |
| Team-scoped learnings (post_game_learning) | User-scoped personal learnings (personal_learning — new table) | Phase 14 decision | Different table, different scope |

**Deprecated/outdated:**
- The stub link at match_detail.rs:505 currently uses only `champion` and `opponent` params and has `bg-accent hover:bg-accent-hover` but the label is "Add Learning ->" (arrow to be removed per UI-SPEC). This stub needs updating.

---

## Open Questions

1. **URL encoding of champion names with spaces**
   - What we know: Champion names like "Aurelion Sol", "Lee Sin", "Twisted Fate" contain spaces. The `format!()` macro puts the raw space in the URL. The browser handles `<a href="...name with spaces...">` by percent-encoding on click, but `window.location().set_href()` does not encode automatically.
   - What's unclear: Whether the form's `use_query_map()` will decode `%20` back to a space automatically (Leptos router likely does).
   - Recommendation: For `<a>` tags (not `set_href`), raw spaces in the href are fine — browser encodes. Verify that `query.read().get("champion")` returns the decoded name. If uncertain, use `replace(' ', "+")` in the `format!()` for the href. This is low-risk.

2. **Edit flow: inline vs navigate-to-form**
   - What we know: UI-SPEC says "Edit opens `/personal-learnings/new?edit={id}` (or switches to inline edit mode at discretion)".
   - What's unclear: The planner must choose. Navigate-to-form is simpler (one code path). Inline edit requires a second set of signals and more complex state.
   - Recommendation: Navigate to `/personal-learnings/new?edit={id}`. On the form page, detect the `edit` query param, fetch the record server-side, pre-populate all signals. This reuses the exact same form. Requires one additional server fn: `get_personal_learning(id)`.

---

## Environment Availability

Step 2.6: SKIPPED — phase is purely code/config changes within the existing Rust/Leptos/SurrealDB stack. No new external tools, runtimes, or services are required.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Playwright (e2e) + `cargo test --features ssr --lib` (unit) |
| Config file | `e2e/playwright.config.ts` |
| Quick run command | `cd /home/jasper/Repositories/lol_team_companion/e2e && npx playwright test pages.spec.ts` |
| Full suite command | `cd /home/jasper/Repositories/lol_team_companion/e2e && npx playwright test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LEARN-01 | `/personal-learnings/new` loads without JS errors | smoke | `npx playwright test pages.spec.ts -g "personal-learnings/new"` | ❌ Wave 0 |
| LEARN-01 | Form saves a new learning (all 3 sections filled) | e2e | `npx playwright test audit-personal-learnings.spec.ts -g "create learning"` | ❌ Wave 0 |
| LEARN-02 | `/personal-learnings` (browse) loads without JS errors | smoke | `npx playwright test pages.spec.ts -g "personal-learnings"` | ❌ Wave 0 |
| LEARN-02 | Match detail "Add Learning" button navigates to form with pre-filled champion | e2e | `npx playwright test match-detail.spec.ts -g "Add Learning"` | ❌ Wave 0 |
| LEARN-03 | Filter by type narrows card list | e2e | `npx playwright test audit-personal-learnings.spec.ts -g "filter"` | ❌ Wave 0 |
| LEARN-01 | PersonalLearning model round-trips through serde_json | unit | `cargo test --features ssr --lib personal_learning` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown`
- **Per wave merge:** `cargo test --features ssr --lib && cd e2e && npx playwright test pages.spec.ts`
- **Phase gate:** Full Playwright suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `e2e/tests/audit-personal-learnings.spec.ts` — covers LEARN-01 (create), LEARN-03 (filter), LEARN-01 (delete)
- [ ] Add `/personal-learnings` and `/personal-learnings/new` to `AUTHED_PAGES` array in `e2e/tests/pages.spec.ts`
- [ ] Add unit test for `PersonalLearning` model round-trip in `src/models/personal_learning.rs` (same pattern as `PostGameLearning` tests in `game_plan.rs`)
- [ ] Match detail "Add Learning" test extension — `e2e/tests/match-detail.spec.ts` already exists; add test case for the button's presence and href format

---

## Project Constraints (from CLAUDE.md)

All of the following directives apply to this phase:

| Directive | Applies To |
|-----------|-----------|
| Use `type::record('table', $key)` not `type::thing()` | All SurQL queries |
| Strip table prefix before binding to `type::record()` | DB functions for personal_learning |
| Never deserialize SurrealDB results as `serde_json::Value` — use `Db*` structs | `DbPersonalLearning` struct required |
| `.bind()` requires owned `String` values | All `.bind()` calls |
| No `string()` cast in SurQL | DB queries |
| Use `#[derive(SurrealValue)]` on DB result structs | `DbPersonalLearning` |
| `ActionForm` has no `class` prop — wrap in `<div>` | Not applicable (no ActionForm on this page) |
| After login/register use hard navigation | Not applicable |
| SSR imports inside `#[server]` body | All `#[server]` fns in personal_learnings.rs |
| `attr:class` on `<A>` | Nav link addition |
| DB access via `use_context::<Arc<Surreal<Db>>>()` | All server fns |
| Auth extraction with `mut auth: AuthSession` | All server fns |
| `Clone before multiple closures` for `Vec<T>` | Tags vec, champion list |
| `.into_any()` for divergent view branches | Type selector match, conditional fields |
| `get_untracked()` in event handlers | All on:click handlers |
| `prop:value` for controlled inputs | Title input, text areas |
| `resource.refetch()` after mutations | learnings resource after create/update/delete |
| `spawn_local` for async event handlers | Save and delete button handlers |
| `collect_view()` for iterators | Tag chips, card list |
| `<For>` key must be stable entity ID | Card list, tag chips |
| `.check()` on write queries | CREATE/UPDATE/DELETE personal_learning |
| `take(0).unwrap_or_default()` for list queries | `list_personal_learnings()` |
| `DEFINE FIELD IF NOT EXISTS` for all schema fields | personal_learning schema |
| Return `Ok(Vec::new())` not `Err` when no records | `list_personal_learnings()` |
| Never `.unwrap()` in WASM event handlers | All client-side code |
| `recursion_limit = "512"` in lib.rs and main.rs | Already present — do not change |
| Auth guard pattern — client-side redirect to `/auth/login` if `None` | PersonalLearningsPage and NewLearningPage |
| Add smoke test entry to `e2e/tests/pages.spec.ts` AUTHED_PAGES | Both new routes |
| `ORDER BY` only on selected fields in partial SELECTs | Use `SELECT *` for personal_learning queries |
| `#[allow(unused_variables)]` on hydrate-only signals | Not applicable (no auto-save timer) |

---

## Sources

### Primary (HIGH confidence)
- Direct codebase read — `src/server/db.rs` (lines 1851-1973): `save_post_game_learning`, `update_post_game_learning`, `list_post_game_learnings`, `delete_post_game_learning` patterns
- Direct codebase read — `src/pages/post_game.rs` (lines 366-420): `use_query_map()` pre-fill pattern
- Direct codebase read — `src/pages/match_detail.rs` (lines 505-511, 840-870): existing "Add Learning" CTA stub + timeline event detail panel structure
- Direct codebase read — `schema.surql`: all existing table definitions — IF NOT EXISTS pattern
- Direct codebase read — `src/models/game_plan.rs`: `PostGameLearning` struct + round-trip tests pattern
- Direct codebase read — `src/components/nav.rs`: Nav component + `is_authed` signal pattern
- Direct codebase read — `src/app.rs`: Router + Route registration pattern
- Direct codebase read — `src/pages/mod.rs`, `src/models/mod.rs`: module registration pattern
- Direct codebase read — `.planning/phases/14-personal-learnings-journal/14-UI-SPEC.md`: exact CSS classes, copywriting, interaction contracts
- Direct codebase read — `CLAUDE.md`: all project constraints
- Direct codebase read — `.planning/phases/14-personal-learnings-journal/14-CONTEXT.md`: all locked decisions

### Secondary (MEDIUM confidence)
- `e2e/tests/pages.spec.ts` reviewed: AUTHED_PAGES array structure confirmed for Wave 0 gap
- `src/components/ui.rs` grep: `ToastContext`, `ToastKind::Success` API confirmed

### Tertiary (LOW confidence)
- URL encoding of champion names in `<a href>` — browser behaviour assumed correct (Leptos router decode confirmed by pattern in use_query_map usage, but not tested against spaces specifically)

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — entirely within established project tech, no new dependencies
- Architecture: HIGH — all patterns directly sourced from existing codebase files
- Pitfalls: HIGH — sourced from CLAUDE.md gotcha list + direct codebase inspection
- Integration points: HIGH — exact line numbers in match_detail.rs verified by reading source

**Research date:** 2026-03-27
**Valid until:** 2026-04-27 (stable stack — Leptos 0.8, SurrealDB 3.x, Tailwind v4 are all locked in Cargo.toml)
