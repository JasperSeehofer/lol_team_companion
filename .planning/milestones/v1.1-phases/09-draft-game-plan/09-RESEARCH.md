# Phase 9: Draft & Game Plan - Research

**Researched:** 2026-03-23
**Domain:** Leptos 0.8 reactive UI — draft page wiring, intel sidebar extension, game plan prefill
**Confidence:** HIGH (all findings from direct source reading of project code)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Opponent Selection (DRFT-01, DRFT-02)**
- D-01: Replace the free-text "Opponent" input in the draft header with a searchable dropdown of scouted opponents + a separate "Add New" button next to it
- D-02: Searchable dropdown (type-ahead filter) rather than a plain `<select>`, even though opponent lists are small — user preference
- D-03: "Add New Opponent" button navigates to `/opponents` page after auto-saving the current draft; includes a return link back to the draft
- D-04: Selecting an opponent auto-opens the intel sidebar to show their data immediately
- D-05: Store the opponent record ID in `Draft.opponent` (not the display name). Add a free-text fallback for unlinked opponents (backward compat with existing drafts that have name strings)
- D-06: Remove the duplicate opponent `<select>` from the intel sidebar's "Their Picks" tab — header dropdown is the single source of truth; intel sidebar auto-syncs from `selected_opponent_id`

**Intel Panel UX (DRFT-03, DRFT-04)**
- D-07: Banned champions are filtered out of the matchup champion selection only (not from "Our Pools" tab click-to-pick)
- D-08: Add a back button to the existing champion detail view in the intel panel to navigate back to the champion selection list
- D-09: No changes to the "Our Pools" tab ban filtering — pools tab remains informational

**Pool Notes Sidebar (DRFT-05)**
- D-10: Add a 4th tab "Notes" to the intel sidebar (tabs become: Our Pools, Their Picks, Matchups, Notes)
- D-11: Within the Notes tab, show a second row of sub-tabs — one per our-side picked champion (by champion name)
- D-12: Each champion sub-tab shows ALL note types (matchup, power_spike, combo, lesson, synergy, positioning) in collapsible sections grouped by type
- D-13: When a pooled champion is picked into our side, the intel sidebar auto-opens and auto-switches to the Notes tab + that champion's sub-tab
- D-14: Champions without pool notes still get a sub-tab with an empty state message ("No notes for {champion} — add notes in Champion Pool")
- D-15: Notes data comes from the existing `ChampionNote` model via a new server fn that fetches notes for specific champions by the current user (or team members' pools)

**Game Plan Autofill (PLAN-01, PLAN-03)**
- D-16: Game plan name pre-fills as `"<draft name> plan"` but is editable before saving
- D-17: Strategy tag (`win_condition_tag`) auto-fills from the most common composition tag in the draft. If all tags appear once, use the first tag. User can change it on the game plan page
- D-18: Our/enemy champions pre-fill from draft picks (existing behavior, confirmed working)
- D-19: Role assignments from `DraftAction.role` map to per-role strategy fields — e.g., if Jinx has `role="bot"`, pre-fill `bot_strategy` with `"Jinx: "` as a header prompt. User fills in the strategy after the champion name
- D-20: All prefilled fields remain editable — prefill is a convenience, not a constraint

### Claude's Discretion
- Searchable dropdown implementation approach (reuse ChampionAutocomplete pattern or build a new OpponentAutocomplete)
- How to determine "most common" tag when there's a tie (first in list is fine)
- Exact collapsible section animation/toggle behavior for note types
- Whether to show champion portraits in the Notes sub-tabs
- Auto-save mechanism before navigating to opponents page (reuse existing save_draft or create a lightweight auto-save)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DRFT-01 | User can select opponent from a dropdown of scouted opponents, with an "add new opponent" button | `opponents_list` Resource already loaded; `ChampionAutocomplete` pattern available; `selected_opponent_id` signal wired to intel sidebar |
| DRFT-02 | Draft intel panel has a link to opponents page that auto-saves the current draft before navigating | Existing `do_save` / `update_draft` + `save_draft` server fns; `window.location().set_href()` for navigation; need to pass `?return_to=draft` |
| DRFT-03 | Banned champions are filtered out of the matchup champion selection in intel panel | `draft_slots` signal; slots 0–5 and 12–15 are bans per `slot_meta()`; filter logic mirrors `used_champions` |
| DRFT-04 | User can navigate back from champion detail to champion selection via a back button in intel panel | `matchup_champion` signal already controls detail vs. list view; set to `None` to go back |
| DRFT-05 | Champion pool notes appear in a draft sidebar when a pooled champion is picked | Need new `get_pool_notes_for_champions` server fn; `ChampionNote` model + `NOTE_TYPES` array exist; `fill_slot` callback is the trigger point |
| PLAN-01 | Game plan auto-fills strategy tag from source draft's tags and name as "<draft name> plan" | `get_draft_for_prefill` server fn returns full `Draft` with `tags` and `name` fields; prefill `Effect` in `game_plan.rs` lines 598–646 needs extension |
| PLAN-03 | Creating game plan from draft pre-fills our champions and enemy champions from the draft | Already partially working (confirmed in PLAN-02); prefill Effect already populates `our_sigs_for_prefill` / `enemy_sigs_for_prefill`; needs role-to-strategy extension |
</phase_requirements>

---

## Summary

Phase 9 is a pure UI wiring and data-flow phase — no new tables, no new routes, no new models. All work is concentrated in two existing files: `src/pages/draft.rs` (the largest file in the project at 2000+ lines) and `src/pages/game_plan.rs`. A new DB query function and a new server fn are needed to fetch pool notes for a set of champions, but the `ChampionNote` model and all note DB infrastructure already exist.

The biggest design decisions are already locked: the `opponent` signal becomes a record-ID string (backed by the new searchable dropdown), the intel sidebar gains a 4th tab, and the game-plan prefill Effect gains 3 additional fields (name, win_condition_tag, and per-role strategy strings). All changes are additive.

**Primary recommendation:** Work in three independent vertical slices — (1) opponent dropdown + intel sync, (2) Notes tab in intel sidebar, (3) game-plan autofill extension. Each slice can be implemented and verified independently because they touch different parts of the component tree.

---

## Standard Stack

This phase uses only the existing project stack. No new dependencies.

### Core (already installed)
| Library | Purpose | Role in this phase |
|---------|---------|-------------------|
| Leptos 0.8 | UI framework | All reactive signals, Resources, Effects, Callbacks |
| SurrealDB 3.x (SurrealKV) | Persistence | New `get_pool_notes_for_champions` query |
| Axum 0.8 | Server | New `#[server]` fn for pool notes |
| Tailwind CSS v4 | Styling | Notes tab layout, collapsible sections |

### No new packages needed
All required primitives are already in the codebase: `StoredValue`, `Callback`, `spawn_local`, `Closure::once`, `collect_view`.

---

## Architecture Patterns

### Pattern 1: Opponent Autocomplete (reuse ChampionAutocomplete pattern)
**What:** Build an `OpponentAutocomplete` inline in `draft.rs` using the same structure as `ChampionAutocomplete` but operating on `Vec<Opponent>` instead of `Vec<Champion>`.

**Key difference from ChampionAutocomplete:** The value stored is the opponent record ID (`opp.id`), but displayed text is the opponent name. The `opponent` signal (`RwSignal<String>`) currently stores a free-text name — after this change it stores an ID string. Backward compat: when loading a draft, if the stored value doesn't match any known opponent ID, treat it as a free-text fallback (D-05).

```rust
// Pattern from champion_autocomplete.rs
let select_opp = move |opp: Opponent| {
    let id = opp.id.clone().unwrap_or_default();
    let name = opp.name.clone();
    set_opponent.set(id.clone());
    set_filter_text.set(name);
    set_open.set(false);
    // D-04: auto-open intel sidebar and set selected_opponent_id
    set_selected_opponent_id.set(id);
    set_intel_open.set(true);
};
```

**The `opponents_list` resource already exists** at line 905 in `draft.rs`. Feed it into the dropdown directly — no new Resource needed.

### Pattern 2: Auto-save before navigating to /opponents
**What:** When the "Add New" button is clicked, save the current draft (or update if `loaded_draft_id` is set), then navigate to `/opponents?return_to=draft` (optionally with `&draft_id=<id>` for return navigation).

**Use the existing `do_save` logic pattern** — capture all current signal values, call `save_draft` or `update_draft` via `spawn_local`, and in the `Ok` branch call `window.location().set_href()`.

```rust
on:click=move |_| {
    // capture all needed values eagerly (CLAUDE.md rule 54)
    let name = draft_name.get_untracked();
    let opp = opponent.get_untracked();
    // ... rest of captures
    let existing_id = loaded_draft_id.get_untracked();
    leptos::task::spawn_local(async move {
        let saved_id = if let Some(id) = existing_id {
            update_draft(id.clone(), /* ... */).await.ok();
            id
        } else {
            save_draft(/* ... */).await.unwrap_or_default()
        };
        #[cfg(feature = "hydrate")]
        if let Some(win) = web_sys::window() {
            let url = format!("/opponents?return_to=draft&draft_id={}", saved_id);
            let _ = win.location().set_href(&url);
        }
    });
}
```

### Pattern 3: Notes Tab — New Server Fn
**What:** A new `get_pool_notes_for_champions` server fn that takes a `Vec<String>` (champion names, serialized as JSON string per CLAUDE.md rule 33) and returns `Vec<(String, Vec<ChampionNote>)>` (champion → notes pairs for each champion the current user has notes for).

**DB query:** The existing `get_champion_notes` function queries by user + champion + role. For the draft sidebar we want all notes for a champion regardless of role. A new DB function is needed:

```sql
SELECT *, <string>created_at AS created_at
FROM champion_note
WHERE user = type::record('user', $user_key)
  AND champion IN $champions
ORDER BY champion, note_type, created_at DESC
```

This returns all notes across all roles for the given champions. Group by champion in Rust.

### Pattern 4: Notes Tab — Reactive Sub-tabs
**What:** A two-level tab within the "Notes" intel tab: outer tab selects champion, inner sections show note types collapsibly.

**Signals needed:**
```rust
let (notes_champion_tab, set_notes_champion_tab) = signal(String::new()); // which champion sub-tab
let (collapsed_note_types, set_collapsed_note_types) = signal(Vec::<String>::new()); // which sections are collapsed
```

**Pool notes resource** (keyed on our-side picks):
```rust
let our_picks_for_notes = move || {
    let slots = draft_slots.get();
    let our = if our_side.get() == "blue" { "blue" } else { "red" };
    // pick slots for our side: indices 6-11 (blue picks phase 1), 16-19 (blue picks phase 2), etc.
    // Use slot_meta() to filter
    slots.iter().enumerate()
        .filter(|(i, s)| {
            let (side, kind, _) = slot_meta(*i);
            side == our && kind == "pick" && s.is_some()
        })
        .filter_map(|(_, s)| s.clone())
        .collect::<Vec<String>>()
};

let pool_notes = Resource::new(
    our_picks_for_notes,
    move |picks| async move {
        if picks.is_empty() { return Ok(vec![]); }
        let json = serde_json::to_string(&picks).unwrap_or_default();
        get_pool_notes_for_champions(json).await
    },
);
```

### Pattern 5: Auto-open Notes tab when pick lands (D-13)
**What:** When a pooled champion is picked onto our side, auto-switch to Notes tab and select that champion's sub-tab.

**Hook into `fill_slot`** — after the pick is placed, check if the slot is a pick on our side and if the champion has pool notes:

```rust
// Inside fill_slot, after setting draft_slots:
let (_, kind, _) = slot_meta(slot_idx);
let slot_our_side = slot_meta(slot_idx).0;
let current_our_side = our_side.get_untracked();
if kind == "pick" && slot_our_side == current_our_side {
    // Check if champion has notes — peek at pool_notes resource
    if let Some(Ok(notes)) = pool_notes.get_untracked() {
        if notes.iter().any(|(c, ns)| c == &champion_name && !ns.is_empty()) {
            set_intel_open.set(true);
            set_intel_tab.set("notes".to_string());
            set_notes_champion_tab.set(champion_name.clone());
        }
    }
}
```

Note: `pool_notes` resource refetches when `our_picks_for_notes` changes (which happens when `draft_slots` changes). The refetch is async, so a short settle may be needed — alternatively, trigger the tab switch in a follow-up Effect that watches the resource.

### Pattern 6: Game Plan Prefill Extension
**What:** The existing prefill `Effect` in `game_plan.rs` (lines 598–646) already populates our/enemy champions and win conditions. Extend it to also populate:
1. `plan_name` ← `format!("{} plan", draft.name)`
2. `win_condition_tag` ← most common tag in `draft.tags`
3. Per-role strategy fields ← from `DraftAction.role` for our-side picks

**Most common tag algorithm:**
```rust
let win_tag = if !draft.tags.is_empty() {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for tag in &draft.tags {
        *counts.entry(tag.as_str()).or_insert(0) += 1;
    }
    draft.tags.iter()
        .max_by_key(|t| counts[t.as_str()])
        .cloned()
} else {
    None
};
```

**Role-to-strategy mapping:**
```rust
// roles: top/jungle/mid/bot/support → top_strategy/jungle_strategy/mid_strategy/bot_strategy/support_strategy
let mut role_strats: Vec<String> = vec![String::new(); 5]; // index = 0:top,1:jng,2:mid,3:bot,4:sup
let our = draft.our_side.clone();
for action in &draft.actions {
    if action.side == our && action.phase.contains("pick") {
        if let Some(role) = &action.role {
            let idx = match role.as_str() {
                "top" => 0, "jungle" => 1, "mid" => 2, "bot" => 3, "support" => 4, _ => continue,
            };
            if role_strats[idx].is_empty() {
                role_strats[idx] = format!("{}: ", action.champion);
            }
        }
    }
}
set_role_strats.set(role_strats);
```

The `role_strats` signal maps to `top_strategy` through `support_strategy` via index in the game plan editor (indices 0–4 already used in the `clear_editor` callback at line 665).

### Pattern 7: Ban filtering in Matchups tab (DRFT-03)
**What:** The matchups tab lets users pick a champion to look up notes for (`set_matchup_champion`). Filter out banned champions from the selection grid.

**Banned slots:** Per `slot_meta()`, bans are at indices 0–5 (phase 1 bans) and 12–15 (phase 2 bans). Extract banned champions:
```rust
let banned_champions = move || {
    draft_slots.get().iter().enumerate()
        .filter(|(i, _)| {
            let (_, kind, _) = slot_meta(*i);
            kind == "ban"
        })
        .filter_map(|(_, s)| s.clone())
        .collect::<Vec<String>>()
};
```
Pass this as a filter to the champion selection UI inside the Matchups tab — exclude any champion in `banned_champions()`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Searchable opponent dropdown | Custom filtered list | Reuse `ChampionAutocomplete` pattern | Pattern is battle-tested, includes blur delay, mousedown preventDefault, StoredValue |
| Champion notes lookup | New notes model | Existing `ChampionNote`, `NOTE_TYPES`, `note_type_label()`, DB functions | All infrastructure exists in `src/models/champion.rs` and `src/server/db.rs` |
| Auto-save before navigate | New save mechanism | Existing `save_draft` / `update_draft` server fns | Same fn used by auto-save timer already |
| Back navigation in Intel | Router push/pop | Set `matchup_champion` to `None` | `matchup_champion` is already the state driving detail vs. list view |
| Role-to-strategy index mapping | Struct with named fields | `Vec<String>` indexed 0–4 | `set_role_strats` / `role_strats` signal is already a 5-element Vec by convention |

---

## Common Pitfalls

### Pitfall 1: Backward Compat on `Draft.opponent` field
**What goes wrong:** After D-05, `Draft.opponent` stores an opponent record ID like `"opponent:abc123"`. Old drafts have a display name like `"Team Evil"`. When loading a draft, trying to match the stored value against `opponents_list` by ID will fail for old drafts.
**How to avoid:** When loading a draft and populating the opponent dropdown, check if the stored value matches any opponent ID in `opponents_list`. If it does, set the display text to that opponent's name. If not (legacy free-text), display the raw value as-is in the input field. The `selected_opponent_id` signal should only be set to the ID if a match is found.
**Warning signs:** Opponent dropdown shows a raw ID string like "opponent:abc123" instead of a name.

### Pitfall 2: `fill_slot` and pool_notes resource timing (D-13)
**What goes wrong:** When a champion is added to a pick slot, `draft_slots` changes → `our_picks_for_notes` derived signal updates → `pool_notes` Resource refetches. But the refetch is async. If we try to read `pool_notes.get_untracked()` immediately inside `fill_slot`, it still has the old value and no auto-switch happens.
**How to avoid:** Use an `Effect` that watches `pool_notes` and `draft_slots` together. When both are available and a newly added our-side pick has notes, switch the tab. Use `get_untracked()` on `draft_slots` inside the effect so it only re-tracks `pool_notes`:
```rust
Effect::new(move |_| {
    if let Some(Ok(notes_map)) = pool_notes.get() { // track pool_notes
        // Check if any our-side pick champion has notes
        // Only switch if user hasn't manually changed the tab
    }
});
```
**Warning signs:** Notes tab never auto-opens; or it opens but shows empty state because notes hadn't loaded yet.

### Pitfall 3: `intel_tab` non-Copy capture in multiple closures
**What goes wrong:** The tab button rendering loop uses `intel_tab` in both `class=move ||` and the tab label `match`. Since `intel_tab` is a `ReadSignal`, it IS Copy — but the label match also needs a reference.
**How to avoid:** Follow the existing pattern in draft.rs lines 1973–1997: clone `tab_str` into `tab_for_class` and `tab_for_click` before the view.

### Pitfall 4: `selected_opponent_id` vs. `opponent` signal divergence
**What goes wrong:** There are now two related signals: `opponent` (the stored value in the draft, an ID or legacy name) and `selected_opponent_id` (the intel sidebar's active opponent for loading intel data). If the header dropdown sets `opponent` but not `selected_opponent_id`, the intel sidebar won't update.
**How to avoid:** When the user selects an opponent from the dropdown, set BOTH signals. When loading a saved draft, if the stored `opponent` value matches a known ID, set `selected_opponent_id` to that ID as well.

### Pitfall 5: Game plan prefill Effect runs too early
**What goes wrong:** The prefill Effect fires when `prefill_data` resource resolves. If the game plan page is navigated to without a `?draft_id=` param, `prefill_data` returns `None` and the Effect exits early. But if `plan_name` is not initialized to empty, the pre-filled name from a previous visit may persist.
**How to avoid:** The `clear_editor` callback already clears `plan_name`. Just ensure the new `set_plan_name` call inside the prefill Effect correctly sets it, and `clear_editor` resets it. Already handled by existing patterns.

### Pitfall 6: `get_champion_notes` requires a `role` parameter
**What goes wrong:** The existing `get_champion_notes(db, user_id, champion, role)` function takes an explicit role. For the draft sidebar, the user's champion may be played in any role and we want notes from all roles.
**How to avoid:** Write a NEW `get_pool_notes_for_champions(db, user_id, champions)` DB function that queries without a role filter and accepts a list of champions. This is a distinct query from `get_champion_notes`.

### Pitfall 7: `pool_notes` Resource dependency on `our_side` signal
**What goes wrong:** Our-side picks change whenever `our_side` toggles (blue/red). The `pool_notes` Resource key must include both `draft_slots` AND `our_side` to correctly refetch when the side toggle changes which slots are "ours."
**How to avoid:** The `our_picks_for_notes` closure (Resource key) already references both `draft_slots.get()` and `our_side.get()`, so it will correctly be reactive to both signals.

---

## Code Examples

### New DB function: get_pool_notes_for_champions
```rust
// Source: project db.rs pattern for champion_note queries
pub async fn get_pool_notes_for_champions(
    db: &Surreal<Db>,
    user_id: &str,
    champions: &[String],
) -> DbResult<Vec<ChampionNote>> {
    if champions.is_empty() {
        return Ok(Vec::new());
    }
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut r = db
        .query("SELECT *, <string>created_at AS created_at FROM champion_note WHERE user = type::record('user', $user_key) AND champion IN $champions ORDER BY champion, note_type, created_at DESC")
        .bind(("user_key", user_key))
        .bind(("champions", champions.to_vec()))
        .await?;
    let entries: Vec<DbChampionNote> = r.take(0).unwrap_or_default();
    Ok(entries.into_iter().map(ChampionNote::from).collect())
}
```

### New server fn: get_pool_notes_for_champions
```rust
#[server]
pub async fn get_pool_notes_for_champions(
    champions_json: String,
) -> Result<Vec<(String, Vec<ChampionNote>)>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let champions: Vec<String> = serde_json::from_str(&champions_json)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;

    let notes = db::get_pool_notes_for_champions(&surreal, &user.id, &champions)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Group by champion
    let mut map: std::collections::HashMap<String, Vec<ChampionNote>> = std::collections::HashMap::new();
    for note in notes {
        map.entry(note.champion.clone()).or_default().push(note);
    }
    // Return in same order as input champions list
    Ok(champions.into_iter()
        .map(|c| {
            let ns = map.remove(&c).unwrap_or_default();
            (c, ns)
        })
        .collect())
}
```

### Determining most common tag
```rust
// In game_plan.rs prefill Effect
let win_tag = if !draft.tags.is_empty() {
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for tag in &draft.tags {
        *counts.entry(tag.as_str()).or_insert(0) += 1;
    }
    let max_count = *counts.values().max().unwrap_or(&0);
    // First tag with max count (preserves position tie-break per D-17)
    draft.tags.iter()
        .find(|t| counts[t.as_str()] == max_count)
        .cloned()
} else {
    None
};
if let Some(tag) = win_tag {
    set_win_condition_tag.set(tag);
}
```

### Collapsible note section toggle
```rust
// Use a RwSignal<Vec<String>> for collapsed note types (type name = collapsed)
let (collapsed_types, set_collapsed_types) = signal(Vec::<String>::new());

let toggle_type = Callback::new(move |nt: String| {
    set_collapsed_types.update(|v| {
        if v.contains(&nt) { v.retain(|t| t != &nt); }
        else { v.push(nt); }
    });
});
```

---

## Integration Points Reference

All integration points come from direct code reading (HIGH confidence):

| What to change | Location | Current state | Target state |
|----------------|----------|---------------|--------------|
| Opponent text input in header | `draft.rs:1404–1413` | `<input type="text" ... set_opponent>` | OpponentAutocomplete + "Add New" button |
| Intel tab array | `draft.rs:1973` | `["pools", "their_picks", "matchups"]` | `["pools", "their_picks", "matchups", "notes"]` |
| Intel sidebar opponent `<select>` | `draft.rs:2136–2159` | Renders a `<select>` for opponent | Remove; intel auto-syncs from `selected_opponent_id` |
| Load draft opponent handling | `draft.rs:972–985` | `set_opponent.set(d_opp)` where `d_opp` is raw string | Also set `selected_opponent_id` if value matches a known ID |
| `fill_slot` callback | `draft.rs:1041–1073` | Sets slot, auto-guesses role | Also trigger Notes tab switch when pooled pick detected |
| Game plan prefill Effect | `game_plan.rs:598–646` | Fills our/enemy champs, win_conditions, notes, draft_id | Also fill `plan_name`, `win_condition_tag`, `role_strats` |
| Matchup champion selection | `draft.rs` (Matchups tab) | Shows all champions | Filter out `banned_champions()` |
| Matchup champion detail | `draft.rs` (Matchups tab) | Shows detail, no back button | Add back button that sets `matchup_champion(None)` |

---

## Validation Architecture

`nyquist_validation` is enabled in `.planning/config.json`.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Playwright (e2e) + cargo test (unit) |
| Config file | `e2e/playwright.config.ts` |
| Quick run command | `cd /home/jasper/Repositories/lol_team_companion/e2e && npx playwright test audit-draft.spec.ts audit-game-plan.spec.ts` |
| Full suite command | `cd /home/jasper/Repositories/lol_team_companion/e2e && npx playwright test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DRFT-01 | Opponent dropdown renders + filters opponents | e2e smoke | `npx playwright test audit-draft.spec.ts` | ✅ (needs new test case) |
| DRFT-02 | "Add New" button saves draft before navigating | e2e interaction | `npx playwright test audit-draft.spec.ts` | ✅ (needs new test case) |
| DRFT-03 | Banned champions not in matchup selection | e2e interaction | `npx playwright test audit-draft.spec.ts` | ✅ (needs new test case) |
| DRFT-04 | Back button in matchup detail returns to list | e2e interaction | `npx playwright test audit-draft.spec.ts` | ✅ (needs new test case) |
| DRFT-05 | Notes tab appears; sub-tabs per our-side pick | e2e smoke | `npx playwright test audit-draft.spec.ts` | ✅ (needs new test case) |
| PLAN-01 | Game plan name + strategy tag pre-filled | e2e interaction | `npx playwright test audit-game-plan.spec.ts` | ✅ (needs new test case) |
| PLAN-03 | Our/enemy champions pre-filled + role strategies | e2e interaction | `npx playwright test audit-game-plan.spec.ts` | ✅ (needs new test case) |

### Unit Tests
| Req | Behavior | Test Type | Command | File |
|-----|----------|-----------|---------|------|
| PLAN-01 | `most_common_tag` helper returns correct tag | unit | `cargo test --features ssr --lib` | New test in `src/models/draft.rs` or inline in `game_plan.rs` |
| DRFT-05 | `get_pool_notes_for_champions` groups correctly | unit | `cargo test --features ssr --lib` | New test in `src/server/db.rs` `#[cfg(test)]` block |

### Sampling Rate
- **Per task commit:** `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown`
- **Per wave merge:** `cargo test --features ssr --lib`
- **Phase gate:** `cd e2e && npx playwright test` full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `e2e/tests/audit-draft.spec.ts` — needs new test cases for DRFT-01 through DRFT-05 (file exists, needs expansion)
- [ ] `e2e/tests/audit-game-plan.spec.ts` — needs new test cases for PLAN-01, PLAN-03 (file exists, needs expansion)

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies for this phase — all changes are in existing Rust/Leptos/SurrealDB code with no new CLI tools, services, or runtimes required).

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| Free-text opponent field | Structured opponent ID with fallback for legacy | DRFT-01: backward compat required on load |
| Intel tab list hardcoded | Array-driven tab loop | DRFT-05: just push "notes" to the array |
| Game plan prefill limited to champions + win_conditions | Extend to name + tag + role strategies | PLAN-01, PLAN-03 |

---

## Open Questions

1. **Where does `get_pool_notes_for_champions` server fn live?**
   - What we know: All server fns for data that originates in `draft.rs` (e.g. `get_team_pools`, `get_matchup_notes_for_champion`) are defined at the top of `draft.rs`.
   - Recommendation: Define the new server fn in `draft.rs` alongside the existing intel server fns. This keeps all draft-page data dependencies in one file.

2. **Should the "Add New" opponent button preserve a partial draft (unsaved state)?**
   - What we know: D-03 says "auto-saves the current draft before navigating." The auto-save only runs when `draft_name` is non-empty (existing guard at line 1159).
   - Recommendation: If name is empty, navigate immediately without saving (or show a toast like "Draft wasn't saved — give it a name first"). User is going to create an opponent and return, at which point they can save.

3. **`return_to` mechanism: URL param or localStorage?**
   - What we know: D-03 says "includes a return link back to the draft." The opponents page (`/opponents`) doesn't currently have return-link logic.
   - Recommendation: Use a `?return_to=draft&draft_id=<id>` URL param. The opponents page can read this param and show a "Back to Draft" link. This requires a small addition to `opponents.rs` to read the query param and conditionally render the link. Minimal scope.

---

## Sources

### Primary (HIGH confidence)
- `src/pages/draft.rs` — full component read; opponent signals (lines 854, 900), intel sidebar tabs (lines 1973–1997), intel opponent select (lines 2136–2159), auto-save pattern (lines 1229–1302), `fill_slot` (lines 1041–1073)
- `src/pages/game_plan.rs` — prefill Effect (lines 598–646), `get_draft_for_prefill` server fn (lines 147–165)
- `src/models/draft.rs` — `Draft`, `DraftAction` structs; `guess_role_from_tags`, `role_icon_url`
- `src/models/champion.rs` — `ChampionNote`, `NOTE_TYPES`, `note_type_label`
- `src/models/game_plan.rs` — `GamePlan` struct with all role strategy fields
- `src/models/opponent.rs` — `Opponent`, `OpponentPlayerIntel`
- `src/components/champion_autocomplete.rs` — full component read; pattern for opponent autocomplete
- `src/server/db.rs` — `get_champion_notes` function (lines 369–384); existing query pattern

### Secondary (MEDIUM confidence)
- `e2e/tests/` directory listing — confirms `audit-draft.spec.ts` and `audit-game-plan.spec.ts` exist

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, full code read
- Architecture patterns: HIGH — all patterns derived from direct source reading
- Pitfalls: HIGH — derived from CLAUDE.md rules + direct code analysis of signal dependencies
- Integration points: HIGH — exact line numbers from source read

**Research date:** 2026-03-23
**Valid until:** 2026-04-22 (stable Leptos 0.8 / SurrealDB 3.x stack, 30-day window)
