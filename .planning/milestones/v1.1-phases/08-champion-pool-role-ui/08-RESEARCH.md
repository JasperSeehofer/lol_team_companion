# Phase 08: Champion Pool & Role UI - Research

**Researched:** 2026-03-23
**Domain:** Leptos 0.8 UI — drag-and-drop, CSS grid, popover/overlay patterns, Rust model evolution
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Champion Pool Layout (UX-08)**
- D-01: Replace current 28px pill entries with full card grid — each card shows champion portrait, name, comfort stars, meta tag badge, and match stats line
- D-02: 3-column grid layout within each tier bucket (cards ~140px wide)
- D-03: Selected champion card gets accent border + subtle box-shadow glow (not just border)
- D-04: Remove button appears as corner × in top-right on card hover (same pattern as current pills but repositioned)

**Drag-and-Drop Between Tiers (UX-08)**
- D-05: Entire tier container is a drop zone — drag a champion card and drop it anywhere in the target tier bucket; card appends to end of that tier's grid
- D-06: Visual feedback: source card goes semi-transparent (opacity 50%) during drag; target tier bucket gets an accent border highlight on dragover
- D-07: No within-tier reordering — drag-and-drop is for tier changes only

**Matchup Entry Rework (UX-08)**
- D-08: For "matchup" note type: replace the free-text title field with a ChampionAutocomplete dropdown to pick the opponent champion; title auto-fills as "vs {opponent name}"
- D-09: Other note types keep existing free-text title — no changes
- D-10: Matchup notes in the notes list display a small opponent champion icon (24px) + name badge next to the "Matchup" label

**Draft Role Assignment (UX-10)**
- D-11: Each filled pick slot in the DraftBoard gets a small role icon badge (16-20px) in the bottom-right corner, overlaid on the champion portrait
- D-12: Clicking the badge opens a popover with 5 role icons (top/jg/mid/bot/sup) as clickable buttons using Community Dragon role icon URLs
- D-13: Auto-guess from `Champion.tags` (Marksman → bot, Assassin → mid, Tank/Fighter → top, Support → sup); jungle from multi-tag heuristics
- D-14: Auto-guessed role badge shows at ~50% opacity with dashed border; user-confirmed shows full opacity with solid border
- D-15: Add optional `role` field to `DraftAction` model and schema — role assignments persist with saved drafts. Phase 9 uses this for game plan autofill

### Claude's Discretion
- Exact card dimensions and spacing within the 3-column grid
- Champion class → role mapping heuristics (best-effort, doesn't need to be perfect)
- Popover positioning and dismiss behavior
- Drag ghost appearance (browser default vs custom)
- Whether to animate card tier transitions

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UX-08 | Champion pool has larger champion icons, drag-and-drop between tiers, and reworked matchup entry (no redundant type selection, opponent champion uses searchable champion list) | Card grid layout pattern, HTML5 drag-and-drop on tier containers, ChampionAutocomplete reuse for matchup opponent |
| UX-10 | Draft pick cards contain a clickable role icon for role assignment, defaulting to auto-guess from champion class (marksman → bot, etc.) | DraftAction model extension, role badge overlay in render_pick_slot, popover pattern, Champion.tags heuristics |
</phase_requirements>

---

## Summary

Phase 8 is a pure UI rework with one small data model change. All the building blocks already exist in the codebase: the champion pool page has tier buckets and card rendering, the draft board has drag-and-drop handlers, the ChampionAutocomplete component works correctly, and `role_icon_url()` is already defined in dashboard.rs. The work is about restructuring the layouts and wiring new interactions, not introducing new infrastructure.

The two hardest interaction patterns are (1) drag-and-drop between tier buckets using HTML5 drag events and (2) a click-triggered popover for role selection in draft pick slots. Both have clear precedents in the existing code. The only data model change needed is adding `role: Option<String>` to `DraftAction`, which flows through automatically into the existing `save_draft` JSON serialization path.

The primary complexity risk is Leptos 0.8 closure capture rules: the tier grid renders inside a deeply nested reactive closure (pool.get() → TIERS.iter() → entries.iter()), and the card drag handlers must capture champion/role/tier values correctly. The role popover in DraftBoard needs careful signal management because each of the 10 pick slots needs its own open/close state, and WASM panics must be avoided.

**Primary recommendation:** Implement in three focused waves: (1) champion pool card grid + drag-and-drop, (2) matchup ChampionAutocomplete, (3) draft role badges + popover + model change.

---

## Standard Stack

No new libraries needed. All features use existing dependencies.

### Core (Already in Cargo.toml)
| Library | Purpose | Notes |
|---------|---------|-------|
| Leptos 0.8 | Reactive UI, SSR + WASM | `signal`, `StoredValue`, `Callback`, `spawn_local`, `Effect` |
| web-sys | HTML5 drag events, DOM, CSS | `DragEvent`, `DataTransfer` already used in draft_board.rs |
| Tailwind CSS v4 | Styling | Semantic tokens (`bg-surface`, `border-accent`, etc.) |
| serde / serde_json | DraftAction JSON serialization | `role` field added to existing struct |
| SurrealDB 3.x | Schema field addition | `DEFINE FIELD IF NOT EXISTS role ON draft_action TYPE option<string>` |

### Reusable In-Project Components
| Component/Helper | Location | Reuse |
|-----------------|----------|-------|
| `ChampionAutocomplete` | `src/components/champion_autocomplete.rs` | Matchup opponent selection (D-08) |
| `role_icon_url(role)` | `src/pages/team/dashboard.rs` line 560-569 | Role badge and popover icons — extract to shared module |
| `tier_color()`, `tier_label_color()`, `tier_label()` | `src/pages/champion_pool.rs` lines 245-276 | Extend for drop-zone highlight |
| `slot_meta(idx)` | `src/components/draft_board.rs` | Already used — no changes |
| drag handlers pattern | `src/components/draft_board.rs` lines 67-101 | Template for tier container drag-and-drop |

---

## Architecture Patterns

### Recommended Change Structure
```
src/
├── components/
│   └── draft_board.rs     # Add role_slots signal prop + render_pick_slot role overlay
├── pages/
│   └── champion_pool.rs   # Replace pill layout with card grid + drag handlers
├── models/
│   └── draft.rs           # Add `role: Option<String>` to DraftAction
schema.surql               # Add DEFINE FIELD role ON draft_action
```

There is no new file needed. All changes are modifications to existing files.

### Pattern 1: CSS Grid Champion Cards (D-01, D-02)

Replace the `flex flex-wrap gap-2` pill container (champion_pool.rs line 554) with a 3-column CSS grid:

```rust
// Inside tier bucket, replacing <div class="flex flex-wrap gap-2">
view! {
    <div class="grid grid-cols-3 gap-2">
        {tier_entries.into_iter().map(|entry| {
            // card is ~140px wide by nature of 3-column grid
            view! {
                <div
                    class=move || {
                        let is_selected = ...;
                        // D-03: accent border + box-shadow glow on selection
                        if is_selected {
                            "relative bg-elevated border border-accent/60 rounded-xl overflow-hidden cursor-pointer group"
                            // box-shadow added via style attr: "box-shadow: 0 0 8px rgba(var(--color-accent-rgb), 0.3)"
                        } else {
                            "relative bg-elevated border border-divider rounded-xl overflow-hidden cursor-pointer group hover:border-accent/30"
                        }
                    }
                    draggable="true"
                    on:dragstart=...    // set "text/x-champion" + "text/x-source-tier"
                    on:click=...
                >
                    // Portrait — full-width, square, ~80-100px tall
                    <img src=img_url class="w-full aspect-square object-cover object-top" />
                    // D-04: hover × button top-right corner
                    <button class="absolute top-1 right-1 opacity-0 group-hover:opacity-100 ..."
                        on:click=move |ev| { ev.stop_propagation(); ... }
                    >"\u{00D7}"</button>
                    // Metadata below portrait
                    <div class="p-1.5 flex flex-col gap-0.5">
                        <span class="text-primary text-xs font-medium truncate">{display_name}</span>
                        // comfort stars, meta badge, stats — same as current pill but stacked
                    </div>
                </div>
            }
        }).collect_view()}
    </div>
}
```

**Key:** Use `object-top` on the portrait image so the champion face is visible (Data Dragon splash crops from top).

### Pattern 2: Tier Container as Drop Zone (D-05, D-06)

Each tier bucket `<div>` becomes a dragover/drop target. Add local drag state signal for the dragover highlight:

```rust
// Per-tier bucket — signals must be created outside the iterator or use RwSignal per tier
// Simplest approach: a single `dragging_over_tier: RwSignal<Option<&'static str>>`
// at the champion pool page level, shared across all tier containers
let dragging_over_tier: RwSignal<Option<&'static str>> = RwSignal::new(None);

// Inside TIERS.iter() for each tier:
let tier_key = tier; // &'static str from TIERS slice
view! {
    <div
        class=move || {
            let is_drag_target = dragging_over_tier.get() == Some(tier_key);
            let base = format!("border rounded-xl p-4 {}", tier_color(tier_key));
            if is_drag_target {
                format!("{} border-accent/60 bg-accent/10", base)  // D-06 highlight
            } else {
                base
            }
        }
        on:dragover=move |ev: web_sys::DragEvent| {
            ev.prevent_default();
            dragging_over_tier.set(Some(tier_key));
        }
        on:dragleave=move |_| dragging_over_tier.set(None)
        on:drop=move |ev: web_sys::DragEvent| {
            ev.prevent_default();
            dragging_over_tier.set(None);
            if let Some(dt) = ev.data_transfer() {
                let champ = dt.get_data("text/x-champion").ok().unwrap_or_default();
                let role = dt.get_data("text/x-role").ok().unwrap_or_default();
                let src_tier = dt.get_data("text/x-source-tier").ok().unwrap_or_default();
                if !champ.is_empty() && src_tier != tier_key {
                    // spawn_local → set_champion_tier(champ, role, tier_key.to_string())
                    // then pool.refetch()
                }
            }
        }
    >
        ...
    </div>
}
```

**D-06 card semi-transparency during drag:** Set `opacity-50` class on the card being dragged. In HTML5 drag, the browser renders a ghost automatically. The source element can be styled with `opacity-50` via a `dragging: RwSignal<Option<(String, String)>>` signal (champion, role). Set it on `on:dragstart`, clear on `dragend`.

### Pattern 3: Matchup ChampionAutocomplete (D-08, D-09)

The note form already has an `is_matchup` flag (champion_pool.rs line 1135). Currently it renders a plain `<input type="text">` for the title. Replace that input — only when `is_matchup` — with `ChampionAutocomplete`. Add a `matchup_opponent_signal: RwSignal<String>` alongside existing form signals:

```rust
let matchup_opponent_signal = RwSignal::new(String::new());

// In the note form, replace the title input with:
{if is_matchup {
    view! {
        <ChampionAutocomplete
            champions=champs_for_form   // from StoredValue
            value=matchup_opponent_signal
            placeholder="Search opponent champion..."
            on_select=Callback::new(move |champ_id: String| {
                // D-08: auto-fill title as "vs {opponent_name}"
                let display_name = champion_map.with_value(|m|
                    m.get(&champ_id).map(|c| c.name.clone()).unwrap_or(champ_id)
                );
                set_note_form_title.set(format!("vs {}", display_name));
            })
        />
    }.into_any()
} else {
    view! { <input type="text" ... /> }.into_any()
}}
```

**Requirement:** `champions_resource` is already loaded in the page. Pass a `StoredValue<Vec<Champion>>` (champion_map pattern) to avoid cloning the full Vec into each closure.

**D-10: Matchup note display with opponent icon.** When rendering matchup notes in the list, extract the opponent name from `note.title` (strip "vs " prefix, look up by name in champion_map):

```rust
// In the matchup notes list rendering:
let opponent_name = note_clone.title.trim_start_matches("vs ").to_string();
let opponent_icon = champion_name_to_id_map.with_value(|m|
    m.get(&opponent_name).and_then(|id| champion_map.get(id)).map(|c| c.image_full.clone())
);
// Render 24px icon before the note title if found
```

### Pattern 4: Draft Role Badge + Popover (D-11 through D-14)

`DraftBoard` component needs two new props:

```rust
#[component]
pub fn DraftBoard(
    // ...existing props...
    #[prop(optional)] role_slots: Option<ReadSignal<Vec<Option<String>>>>,
    #[prop(optional)] on_role_set: Option<Callback<(usize, String)>>,
) -> impl IntoView {
```

`role_slots` is a parallel `Vec<Option<String>>` to `draft_slots` — each pick slot index maps to an optional role string.

**Popover state:** Use a single `RwSignal<Option<usize>>` (`role_popover_open`) for which slot's popover is open. This avoids creating 10 separate signals. Clicking outside closes it via a document-level click handler:

```rust
let role_popover_open: RwSignal<Option<usize>> = RwSignal::new(None);

// In render_pick_slot, inside the filled slot view:
// Bottom-right role badge:
{move || {
    let role = role_slots.and_then(|rs| rs.get().get(slot_idx).cloned().flatten());
    let is_auto_guessed = ...; // based on separate signal or annotation
    // Render badge using role_icon_url()
    let icon_url = role.as_deref().map(role_icon_url_shared).unwrap_or("");
    view! {
        <button
            class=move || {
                // D-14: dashed border + 50% opacity for auto-guess
                if is_auto_guessed {
                    "absolute bottom-1 right-1 w-4 h-4 opacity-50 border border-dashed border-outline rounded cursor-pointer z-10"
                } else {
                    "absolute bottom-1 right-1 w-4 h-4 border border-outline/70 rounded cursor-pointer z-10"
                }
            }
            on:click=move |ev| {
                ev.stop_propagation();
                role_popover_open.update(|v| {
                    *v = if *v == Some(slot_idx) { None } else { Some(slot_idx) }
                });
            }
        >
            {if icon_url.is_empty() {
                view! { <span class="text-[8px] text-muted">"?"</span> }.into_any()
            } else {
                view! { <img src=icon_url class="w-full h-full" /> }.into_any()
            }}
        </button>
        // Popover
        {move || (role_popover_open.get() == Some(slot_idx)).then(|| view! {
            <div class="absolute bottom-6 right-0 z-50 bg-elevated border border-divider rounded-lg p-2 flex gap-1 shadow-xl">
                {["top", "jungle", "mid", "bot", "support"].iter().map(|&r| {
                    let icon = role_icon_url_shared(r);
                    view! {
                        <button
                            class="w-6 h-6 hover:bg-overlay rounded cursor-pointer transition-colors p-0.5"
                            title=r
                            on:click=move |ev| {
                                ev.stop_propagation();
                                if let Some(cb) = on_role_set {
                                    cb.run((slot_idx, r.to_string()));
                                }
                                role_popover_open.set(None);
                            }
                        >
                            <img src=icon class="w-full h-full" alt=r />
                        </button>
                    }
                }).collect_view()}
            </div>
        })}
    }
}}
```

**Only applies to pick slots, not ban slots.** The popover is only rendered inside `render_pick_slot`.

### Pattern 5: Auto-Guess from Champion.tags (D-13)

Pure Rust function, no external data needed. `Champion.tags` is a `Vec<String>` from Data Dragon. Known tag values from Data Dragon: "Assassin", "Fighter", "Mage", "Marksman", "Support", "Tank".

```rust
pub fn guess_role_from_tags(tags: &[String]) -> &'static str {
    // Priority order matters — check most specific first
    if tags.iter().any(|t| t == "Marksman") {
        return "bot";
    }
    if tags.iter().any(|t| t == "Support") {
        return "support";
    }
    if tags.iter().any(|t| t == "Assassin") {
        return "mid";
    }
    if tags.iter().any(|t| t == "Mage") {
        return "mid";
    }
    if tags.iter().any(|t| t == "Tank") || tags.iter().any(|t| t == "Fighter") {
        return "top";
    }
    // Jungle: champions tagged only as "Fighter" with no lane tag — best effort
    // This heuristic will misclassify some champions, which is acceptable (D-13)
    "mid" // fallback
}
```

**Jungle gap:** Data Dragon does not have a "Jungler" tag. The CONTEXT.md acknowledges this limitation: "best effort, doesn't need to be perfect". Accept that jungle champions with Fighter/Assassin tags will be mis-guessed and users will correct them manually.

**Where to call it:** In `draft.rs` (or `DraftBoard`), when a champion is dropped into a pick slot, auto-compute the role and set `role_slots` to `Some((slot_idx, guessed_role, is_auto=true))`.

### Pattern 6: DraftAction Model Extension (D-15)

`DraftAction` in `src/models/draft.rs` needs one new field:

```rust
pub struct DraftAction {
    // ...existing fields...
    pub role: Option<String>,
}
```

This field flows through automatically:
- `serde_json::to_string` on `Vec<DraftAction>` in `save_draft` includes it
- `serde_json::from_str` on load deserializes it
- The `#[derive(SurrealValue)]` on `DraftAction` is SSR-only but the struct is also used in WASM — the derive only affects DB serialization, which is fine
- Schema needs `DEFINE FIELD IF NOT EXISTS role ON draft_action TYPE option<string>`

**Test update required:** The round-trip test `draft_action_round_trips_json` in `draft.rs` must be updated to include `role: None` to avoid compilation failure.

### Pattern 7: Extracting role_icon_url to Shared Location

`role_icon_url()` is currently a private `fn` in `dashboard.rs`. Phase 8 needs it in `draft_board.rs` and/or `draft.rs`. Options:

1. **Copy the function** — simplest, acceptable for a small helper
2. **Move to `src/models/draft.rs`** or a new `src/models/role.rs` — cleaner, shared without re-export complexity
3. **Re-export from `dashboard.rs`** — requires `pub fn` + `use` in draft_board.rs; awkward cross-module dependency

**Recommendation (discretion area):** Move to `src/models/draft.rs` or a new small `src/models/role.rs` file and make it `pub`. Both files already import from models, so no import cycle issues.

### Anti-Patterns to Avoid

- **Reading `role_popover_open` lazily inside a `Closure::once` timer** — if the popover is closed via a document-level click, the timer may fire and read stale state. Keep popover open/close synchronous only (no timers needed for this).
- **Separate signal per slot for popover** — creates 10 signals. One `RwSignal<Option<usize>>` is correct.
- **Calling `champions_resource.get()` inside the tier iterator** — the iterator runs at render time inside a reactive closure; calling `.get()` inside adds reactive dependency. Use `StoredValue` for the champion map instead (same pattern as `DraftBoard`'s `champion_map`).
- **Using `dragend` to clear `dragging_over_tier`** — `dragend` fires on the source element in a different closure scope. Use `dragleave` + `drop` on the target instead.
- **Calling `tier_color()` with a runtime string** — these helpers take `&str` and return `&'static str`. Always pass a `&'static str` from the `TIERS` slice, not a `String` clone, to avoid lifetime issues.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Champion search in matchup form | Custom filtered list | `ChampionAutocomplete` component | Already handles fuzzy filter, blur delay, `on_select` callback |
| Role icon rendering | Custom SVG inline | `role_icon_url()` → Community Dragon CDN | URLs are already defined; CDN serves production SVGs |
| Tier drag data transfer | Custom JS interop | HTML5 `DataTransfer` via `web_sys` | Already used in `DraftBoard`; same `set_data`/`get_data` pattern |
| Champion tag lookup | External API call | `Champion.tags` from existing `champions_resource` | Data Dragon already loaded; tags are in the `Champion` struct |

**Key insight:** Every interaction pattern in this phase has a direct precedent in existing code. The work is composition and modification, not invention.

---

## Common Pitfalls

### Pitfall 1: `dragging_over_tier` not clearing on drag exit
**What goes wrong:** User drags a card out of a tier bucket via a path that doesn't cross the `dragleave` boundary of the inner cards, leaving the accent highlight stuck on the bucket.
**Why it happens:** `dragleave` fires for every child element exit, not just the container exit. When dragging between inner card elements, the container gets spurious `dragleave` events.
**How to avoid:** Use the `ev.related_target()` check pattern to ignore child-element leave events, OR use CSS `pointer-events: none` on children during drag, OR accept minor visual glitch (flicker during drag through children) since state resets on `drop` or `dragend`.
**Warning signs:** Tier bucket stays highlighted after dropping into another tier.

### Pitfall 2: Nested click handlers in role popover
**What goes wrong:** Clicking a role icon in the popover also fires the slot's `on:click` handler (which selects the slot or opens the champion picker), creating a double-action.
**Why it happens:** DOM event bubbling propagates from the role button up through the slot container.
**How to avoid:** Call `ev.stop_propagation()` in every `on:click` on the role badge button and all popover buttons. Already established pattern in `draft_board.rs` for the slot-clear button.

### Pitfall 3: `DraftAction` deserialization with new `role` field
**What goes wrong:** Old saved drafts stored in SurrealDB lack the `role` field; deserialization fails with a missing-field error.
**Why it happens:** `serde` requires all fields by default.
**How to avoid:** `role: Option<String>` with `#[serde(default)]` attribute, OR simply use `Option<String>` with serde's `missing_field` behavior. For `Option<T>`, serde's default behavior on missing JSON key is to produce `None` (NOT an error), because `Option` implies absence. Verify this is working by updating the round-trip test.

### Pitfall 4: `champions_resource` unavailable when rendering matchup form
**What goes wrong:** The ChampionAutocomplete inside the note form needs `Vec<Champion>`, but `champions_resource` may not be resolved when the form first opens.
**Why it happens:** The note form is conditionally rendered (`note_form_open.get().then(|| ...)`); `champions_resource` is a `Resource` that loads asynchronously.
**How to avoid:** Gate the ChampionAutocomplete rendering on `champions_resource.get().is_some()`, falling back to the plain text input when data isn't available. Or use `champions_resource.get().unwrap_or_default()` — an empty champion list just means the autocomplete shows nothing, which is acceptable.

### Pitfall 5: Role badge z-index conflict with slot clear button
**What goes wrong:** The role badge (bottom-right) and the clear button (top-right, shown on highlight) may stack incorrectly.
**Why it happens:** Both are `absolute` positioned within the `relative` slot container.
**How to avoid:** Role badge uses `z-10`, clear button already uses `z-10`. Since they're in different corners, no overlap, but the popover div needs `z-50` to appear above adjacent slots.

### Pitfall 6: WASM panic on `champions_resource` during drag event
**What goes wrong:** `on:dragstart` handler attempts to read champions_resource to get display name, but resource hasn't resolved; `.unwrap()` panics.
**Why it happens:** Drag events are synchronous; resource resolution is async.
**How to avoid:** Never call `.unwrap()` in drag/event handlers. Use `get().and_then(|r| r.ok()).and_then(|v| ...)` pattern, or store the champion map in a `StoredValue` before the tier rendering (same pattern as `DraftBoard`).

---

## Code Examples

### Existing Drag-and-Drop Pattern (from draft_board.rs)
```rust
// Source: src/components/draft_board.rs lines 67-101
on:dragstart=move |ev: web_sys::DragEvent| {
    let slots = draft_slots.get_untracked();
    if let Some(Some(champ_name)) = slots.get(slot_idx) {
        if let Some(dt) = ev.data_transfer() {
            let _ = dt.set_data("text/plain", champ_name);
            let _ = dt.set_data("text/x-source-slot", &slot_idx.to_string());
        }
    }
}
on:dragover=move |ev: web_sys::DragEvent| ev.prevent_default()
on:drop={
    move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        if let Some(dt) = ev.data_transfer() {
            if let Ok(name) = dt.get_data("text/plain") {
                if !name.is_empty() {
                    on_slot_drop.run((slot_idx, name));
                }
            }
        }
    }
}
```

### Existing role_icon_url Helper (from dashboard.rs)
```rust
// Source: src/pages/team/dashboard.rs lines 560-569
fn role_icon_url(role: &str) -> &'static str {
    match role {
        "top" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-top.svg",
        "jungle" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-jungle.svg",
        "mid" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-middle.svg",
        "bot" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-bottom.svg",
        "support" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-utility.svg",
        _ => "",
    }
}
```

### StoredValue Champion Map Pattern
```rust
// Source: src/components/draft_board.rs line 45
let champion_map = StoredValue::new(champion_map);
// Usage in closures:
let icon_url = champion_map.with_value(|m| {
    m.get(&champ_name).map(|c| c.image_full.clone()).unwrap_or_default()
});
```

### ChampionAutocomplete on_select Callback
```rust
// Source: src/components/champion_autocomplete.rs line 9, 34
// Component signature:
pub fn ChampionAutocomplete(
    champions: Vec<Champion>,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] on_select: Option<Callback<String>>,
) -> impl IntoView
// on_select receives the champion's canonical Data Dragon ID (not display name)
```

### SurrealDB Schema DEFINE FIELD Pattern
```surql
-- Source: schema.surql lines 72-78 (existing draft_action fields)
DEFINE FIELD IF NOT EXISTS role ON draft_action TYPE option<string>;
```

---

## State of the Art

No new patterns needed beyond what's already in the codebase. All patterns are established.

| Topic | Current Approach | Notes |
|-------|-----------------|-------|
| HTML5 drag-and-drop | `web_sys::DragEvent` + `DataTransfer` | Working in draft board. Same for champion pool tiers. |
| Click popover | `RwSignal<Option<usize>>` + conditional `{move \|\| condition.then(\|\| view!{...})}` | Established in dashboard join request handling. |
| Champion search | `ChampionAutocomplete` with `Callback<String>` | Used in champion_pool.rs for adding to pool; reuse for matchup. |

---

## Open Questions

1. **Role value for "jungle" vs "jg"**
   - What we know: `role_icon_url()` in dashboard.rs uses `"jungle"` as the key, returning the jungle SVG. `POOL_ROLES` in champion_pool.rs uses `"Jungle"` (capitalized). The draft board's `our_side` uses `"blue"`/`"red"` (lowercase).
   - What's unclear: Should DraftAction.role store `"jungle"` (matching `role_icon_url`) or `"jg"` or `"mid"` etc.?
   - Recommendation: Use `role_icon_url`'s key set exactly: `"top"`, `"jungle"`, `"mid"`, `"bot"`, `"support"`. Consistent with the existing helper and Community Dragon naming.

2. **Auto-guess confidence tracking**
   - What we know: D-14 requires visual distinction between auto-guessed and user-confirmed roles. The `DraftAction.role` field is `Option<String>` and doesn't store whether it was auto-guessed.
   - What's unclear: Should auto-guess state be persisted or only held in WASM signal memory during the session?
   - Recommendation: Track auto-guess in a separate client-side `RwSignal<Vec<bool>>` (`role_auto_guessed: RwSignal<Vec<bool>>`), one entry per slot index. Do NOT persist this to DB — when a draft is loaded, all roles are treated as user-confirmed (they were set by the user in a previous session). Auto-guess only applies to freshly placed champions in the current session.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is code/config changes only (Rust/Leptos UI + schema field + model addition). No external dependencies introduced.

---

## Validation Architecture

`nyquist_validation` is enabled (config.json has `"nyquist_validation": true`).

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Cargo test (unit) + Playwright (e2e) |
| Config file | `e2e/playwright.config.ts` |
| Quick run command | `cargo test --features ssr --lib` |
| Full suite command | `cd e2e && npx playwright test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UX-08 | DraftAction round-trip includes `role` field | unit | `cargo test --features ssr --lib draft_action_round_trips_json` | ✅ (update needed) |
| UX-08 | Champion pool page loads without errors | e2e | `cd e2e && npx playwright test audit-champion-pool.spec.ts` | ✅ |
| UX-08 | Champion cards visible (not pills) in tier buckets | e2e | `cd e2e && npx playwright test audit-champion-pool.spec.ts` | ✅ (extend test) |
| UX-08 | Matchup form uses champion autocomplete | e2e smoke | `cd e2e && npx playwright test audit-champion-pool.spec.ts` | ✅ (extend test) |
| UX-10 | Draft page loads without errors after model change | e2e | `cd e2e && npx playwright test audit-draft.spec.ts` | ✅ |
| UX-10 | Role badge visible on filled pick slot | e2e | `cd e2e && npx playwright test audit-draft.spec.ts` | ✅ (extend test) |
| UX-08 | `guess_role_from_tags` returns correct roles | unit | `cargo test --features ssr --lib guess_role_from_tags` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --features ssr --lib`
- **Per wave merge:** `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown && cargo test --features ssr --lib`
- **Phase gate:** Full Playwright e2e suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Unit test for `guess_role_from_tags()` function (new function, must be defined in models or a helper module with `#[cfg(test)]` block)
- [ ] Update `draft_action_round_trips_json` test in `src/models/draft.rs` to add `role: None` field — compilation will fail otherwise

*(Playwright tests for champion pool and draft already exist; they need extending, not creating from scratch)*

---

## Project Constraints (from CLAUDE.md)

These directives apply to all implementation in this phase:

| Category | Constraint |
|----------|-----------|
| Theming | Use semantic tokens ONLY: `bg-base`, `bg-surface`, `bg-elevated`, `text-primary`, `text-secondary`, `text-muted`, `text-dimmed`, `border-divider`, `border-outline`, `bg-accent`, `text-accent-contrast`. Exception: colored buttons with white text keep `text-white`. |
| SurrealDB | All new schema fields: `DEFINE FIELD IF NOT EXISTS` for idempotency |
| SurrealDB | Never use `type::thing()` — use `type::record()` |
| Leptos | Never `.unwrap()` in event handlers or WASM code — use `if let Some(...)` |
| Leptos | Clone before multiple closures (rules 18, 22) |
| Leptos | `into_any()` on divergent view branches |
| Leptos | `get_untracked()` in event handlers |
| Leptos | `collect_view()` for iterators in view! |
| Leptos | `Callback::new()` for closures shared across reactive contexts |
| Leptos | `spawn_local` for async event handlers |
| Leptos | `StoredValue::new()` for non-reactive data shared across closures |
| Server fns | `use` imports for SSR crates inside `#[server]` body, not at file top |
| Testing | `cargo test --features ssr --lib` only (integration tests OOM) |
| New page (n/a) | No new pages in this phase |

---

## Sources

### Primary (HIGH confidence)
- `src/components/draft_board.rs` — existing drag-and-drop implementation (dragstart, dragover, drop, DataTransfer pattern)
- `src/pages/champion_pool.rs` — existing tier rendering, pill layout, note form, all signals
- `src/components/champion_autocomplete.rs` — ChampionAutocomplete API surface and `on_select` callback behavior
- `src/models/draft.rs` — `DraftAction` struct, existing fields, round-trip test
- `src/models/champion.rs` — `Champion.tags: Vec<String>`, `ChampionPoolEntry`, `ChampionNote`
- `src/pages/team/dashboard.rs` — `role_icon_url()` helper (lines 560-569)
- `schema.surql` — existing `draft_action` table and field definitions (lines 72-79)
- `CLAUDE.md` — all coding rules, theming constraints, Leptos gotchas

### Secondary (MEDIUM confidence)
- Data Dragon API tag set ("Assassin", "Fighter", "Mage", "Marksman", "Support", "Tank") — from `Champion.tags` values observed in codebase; tag vocabulary is stable across Data Dragon versions

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, all existing
- Architecture: HIGH — based on direct codebase reading, not inference
- Pitfalls: HIGH — derived from existing CLAUDE.md documented patterns + code analysis
- Auto-guess heuristics: MEDIUM — Data Dragon tag set is stable but jungle detection is a best-effort approximation

**Research date:** 2026-03-23
**Valid until:** 2026-04-22 (stable stack, Leptos 0.8 API unlikely to change in 30 days)
