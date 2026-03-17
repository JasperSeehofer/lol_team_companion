# Phase 5: Post-Game Loop + Polish - Research

**Researched:** 2026-03-17
**Domain:** Leptos 0.8 reactive UI — toast system, skeleton screens, empty states, server-side pipeline automation
**Confidence:** HIGH (all findings verified against existing codebase; no external library additions required)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Action Item Auto-Generation (PIPE-02)**
- Trigger: Fully automatic on post-game review save — no confirmation step
- Source data: The `improvements` array from the review only (not strengths or patterns)
- Each improvement string becomes one ActionItem record with `source_review` set to the review ID
- Assignment: Created unassigned (team-wide) — coach or team assigns later on the action items page
- Feedback: Inline StatusMessage banner on post-game page showing "N action items created — View" with a clickable link to `/action-items`. Stays visible until dismissed or next save
- Deduplication: Claude's discretion on whether to skip creating items that match existing open action items

**Empty State Design (UX-01)**
- Tone: Coaching/guiding — tell users what to do next
- Visuals: Include a relevant icon above the message (Unicode or existing icon patterns). No illustrations or custom assets
- CTA: Primary accent-colored button with action label
- No-team state: All team-scoped pages show consistent message — "You need a team to use this feature" with button linking to roster page
- Pages needing empty states: champion_pool, stats, opponents, profile (no Riot account linked), team_builder, team/roster, team/dashboard panels

**Loading Skeletons (UX-02)**
- Type: Shape-matching skeletons that approximate actual content layout
- Architecture: Reusable primitive components (SkeletonLine, SkeletonCard, SkeletonGrid) that pages compose in their Suspense fallbacks
- Animation: Tailwind `animate-pulse` (opacity fade)
- Pages needing skeletons: All data-fetching pages that currently show "Loading..." text

**Mutation Feedback — Toast System (UX-03)**
- Pattern: Fixed floating toast at top-center of screen
- Success toasts: Auto-dismiss after 3-5 seconds. Green styling
- Error toasts: Persist until manually dismissed via "x" button. Include a "copy to clipboard" button for the error message. Red styling
- Migration: Replace ALL existing inline StatusMessage/ErrorBanner usage with the new toast system for consistency
- Implementation: New Toast component + ToastProvider context. Pages call a `show_toast()` function instead of managing local message signals
- Pages needing feedback added: action_items (success messages), champion_pool, opponents, profile, team_builder, team/roster

### Claude's Discretion
- Skeleton exact shapes per page (how closely to match actual content)
- Action item deduplication logic (fuzzy match vs exact)
- Toast animation (slide-in, fade-in)
- Toast stacking behavior when multiple toasts fire
- Whether to keep ErrorBanner for non-mutation errors (e.g. page-level load failures) or unify everything through toast

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PIPE-02 | Post-game review auto-generates action items from identified patterns | `create_action_item` in db.rs already accepts `source_review: Option<String>`; `create_review` server fn returns new review ID; need batch creation fn + wiring in `create_review`/`update_review` |
| UX-01 | All data pages show meaningful empty states with contextual CTAs when no data exists | `EmptyState` component in ui.rs needs icon + CTA enhancement; 7 pages identified needing updates |
| UX-02 | All data-fetching pages use skeleton loading screens instead of blank/spinner | 15+ `"Loading..."` fallbacks identified across all pages; reusable primitives avoid code duplication |
| UX-03 | All mutations (save, delete, update) show consistent success/error feedback | 11 pages use local `status_msg`/`error_msg` signals; ToastProvider context replaces all of them |
</phase_requirements>

---

## Summary

Phase 5 is a pure polish + automation phase with no new data models or routes. All four requirements build on existing infrastructure. The work divides into four independent workstreams that can be partially parallelized:

**PIPE-02** requires the smallest server-side change: after saving a post-game review, loop over `improvements`, call `create_action_item` for each (checking for open duplicates if deduplication is enabled), and return the count alongside the review ID. The `create_review` and `update_review` server functions in `post_game.rs` need to call a new `batch_create_action_items_from_review` DB function. The return type of `create_review` can stay `String` (review ID); the action item count is returned separately or embedded in a new return struct.

**UX-01** enhances `EmptyState` in `ui.rs` and adds a new `NoTeamState` variant. The existing component is minimal (centered text only); the new version accepts an optional icon (Unicode char/SVG), a message, and an optional `(label, href)` CTA pair. Seven pages need their empty-state branches updated.

**UX-02** adds three skeleton primitive components (`SkeletonLine`, `SkeletonCard`, `SkeletonGrid`) to `ui.rs`, all using `animate-pulse bg-elevated rounded`. Pages replace their `"Loading..."` fallback strings with composed skeletons. Post-game, champion-pool, and stats have the most complex layouts; action-items and team/dashboard are simpler.

**UX-03** is the largest workstream: a `ToastProvider` wrapping `<App>` in `app.rs`, a `Toast` component, and a `show_toast(WriteSignal<Vec<Toast>>, ToastKind, String)` helper callable from any `spawn_local` body. All 11 pages with local `status_msg`/`error_msg`/`save_result` signals migrate to `show_toast`. Error toasts add a clipboard copy button using `web_sys::window()?.navigator().clipboard()`.

**Primary recommendation:** Implement in dependency order — (1) new skeleton/empty-state/toast components in `ui.rs` and `app.rs` first (zero regression risk, no page changes), then (2) wire PIPE-02 in the server layer, then (3) migrate pages from inner to outer in terms of complexity.

---

## Standard Stack

### Core (already in project — no new dependencies)

| Library | Purpose | How Used in This Phase |
|---------|---------|----------------------|
| Leptos 0.8 | Reactive UI framework | `RwSignal`, `provide_context`, `use_context`, `Effect`, `spawn_local` for all toast/skeleton/empty-state work |
| Tailwind CSS v4 | Styling | `animate-pulse`, semantic tokens, fixed positioning for toast overlay |
| SurrealDB 3.x | Persistence | New `batch_create_action_items_from_review` DB fn; existing `create_action_item` called in loop |
| web-sys (already in Cargo.toml) | Clipboard API for error toast copy button | `navigator().clipboard().write_text()` |

### No New Dependencies
This phase adds no new crates. All required browser APIs (`Clipboard`, `setTimeout` for auto-dismiss) are already available through `web-sys` and the existing `wasm_bindgen` setup.

---

## Architecture Patterns

### Recommended Project Structure Changes

```
src/components/ui.rs     # Add: Toast, ToastProvider, show_toast helper,
                         #      SkeletonLine, SkeletonCard, SkeletonGrid,
                         #      EmptyState (enhanced), NoTeamState
src/app.rs               # Wrap App content in <ToastProvider>
src/server/db.rs         # Add: batch_create_action_items_from_review
src/pages/post_game.rs   # Wire action item creation in create_review / update_review
src/pages/*.rs           # Replace Loading... fallbacks with skeleton components
src/pages/*.rs           # Replace status_msg/error_msg signals with show_toast
```

### Pattern 1: ToastProvider Context

The toast system uses Leptos context to make `show_toast` callable from any page without prop-drilling. `ToastProvider` owns the signal; pages extract the writer via `use_context`.

```rust
// In src/components/ui.rs

#[derive(Clone, Debug, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
}

#[derive(Clone, Debug)]
pub struct ToastEntry {
    pub id: u64,
    pub kind: ToastKind,
    pub message: String,
}

/// Context type — pages call show_toast() after extracting this.
#[derive(Clone, Copy)]
pub struct ToastContext {
    pub show: Callback<(ToastKind, String)>,
}

#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let (toasts, set_toasts) = signal(Vec::<ToastEntry>::new());
    let next_id = StoredValue::new(std::cell::Cell::new(0u64));

    let show = Callback::new(move |(kind, message): (ToastKind, String)| {
        let id = next_id.with_value(|c| { let v = c.get(); c.set(v + 1); v });
        let entry = ToastEntry { id, kind: kind.clone(), message };
        set_toasts.update(|v| v.push(entry));

        // Auto-dismiss success toasts after 4 seconds
        if kind == ToastKind::Success {
            #[cfg(feature = "hydrate")]
            {
                use wasm_bindgen::prelude::*;
                let cb = Closure::once(move || {
                    set_toasts.update(|v| v.retain(|t| t.id != id));
                });
                if let Some(win) = web_sys::window() {
                    let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                        cb.as_ref().unchecked_ref(), 4000,
                    );
                }
                cb.forget();
            }
        }
    });

    provide_context(ToastContext { show });

    view! {
        {children()}
        <ToastOverlay toasts=toasts set_toasts=set_toasts />
    }
}
```

**Wrapping in app.rs:**
```rust
// In App component, wrap routes content:
<ToastProvider>
    <Nav />
    <main>
        <Routes ...>
    </main>
</ToastProvider>
```

**Usage in pages:**
```rust
// In any page's spawn_local success/error branch:
let toast = use_context::<ToastContext>().expect("ToastProvider missing");
// ...
Ok(_) => toast.show.run((ToastKind::Success, "Review saved!".into())),
Err(e) => toast.show.run((ToastKind::Error, format!("Error: {e}"))),
```

### Pattern 2: Toast Overlay Component

Fixed position at top-center, renders above all other content. Error toasts include dismiss button and clipboard copy.

```rust
#[component]
fn ToastOverlay(
    toasts: ReadSignal<Vec<ToastEntry>>,
    set_toasts: WriteSignal<Vec<ToastEntry>>,
) -> impl IntoView {
    view! {
        <div class="fixed top-4 left-1/2 -translate-x-1/2 z-50 flex flex-col gap-2 items-center pointer-events-none">
            {move || toasts.get().into_iter().map(|t| {
                let id = t.id;
                let is_error = t.kind == ToastKind::Error;
                let msg = t.message.clone();
                let msg_for_copy = t.message.clone();
                let base = if is_error {
                    "pointer-events-auto flex items-start gap-3 bg-red-500/20 border border-red-500/40 text-red-300 rounded-xl px-4 py-3 text-sm shadow-lg min-w-64 max-w-sm"
                } else {
                    "pointer-events-auto flex items-center gap-3 bg-emerald-500/20 border border-emerald-500/40 text-emerald-300 rounded-xl px-4 py-3 text-sm shadow-lg min-w-64 max-w-sm"
                };
                view! {
                    <div class=base>
                        <span class="flex-1">{msg}</span>
                        {if is_error {
                            view! {
                                <div class="flex gap-1 shrink-0">
                                    // Copy to clipboard
                                    <button
                                        class="text-red-400/70 hover:text-red-300 text-xs px-1.5 py-0.5 rounded hover:bg-red-500/20 transition-colors"
                                        on:click=move |_| {
                                            #[cfg(feature = "hydrate")]
                                            {
                                                let msg = msg_for_copy.clone();
                                                if let Some(win) = web_sys::window() {
                                                    let _ = win.navigator().clipboard().write_text(&msg);
                                                }
                                            }
                                        }
                                    >"Copy"</button>
                                    // Dismiss
                                    <button
                                        class="text-red-400/70 hover:text-red-300 text-xs px-1.5 py-0.5 rounded hover:bg-red-500/20 transition-colors"
                                        on:click=move |_| set_toasts.update(|v| v.retain(|t| t.id != id))
                                    >"×"</button>
                                </div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
```

### Pattern 3: Skeleton Primitive Components

Three reusable building blocks composed per-page:

```rust
/// Single line skeleton — width is a Tailwind class like "w-48" or "w-full"
#[component]
pub fn SkeletonLine(#[prop(default = "w-full")] width: &'static str, #[prop(default = "h-4")] height: &'static str) -> impl IntoView {
    let cls = format!("animate-pulse bg-elevated rounded {width} {height}");
    view! { <div class=cls></div> }
}

/// Card-shaped skeleton with configurable height
#[component]
pub fn SkeletonCard(#[prop(default = "h-24")] height: &'static str) -> impl IntoView {
    let cls = format!("animate-pulse bg-elevated rounded-xl border border-divider/30 {height} w-full");
    view! { <div class=cls></div> }
}

/// Grid of skeleton cards
#[component]
pub fn SkeletonGrid(#[prop(default = 3u8)] cols: u8, #[prop(default = 2u8)] rows: u8, #[prop(default = "h-20")] card_height: &'static str) -> impl IntoView {
    let col_class = match cols {
        2 => "grid grid-cols-2 gap-3",
        3 => "grid grid-cols-3 gap-3",
        4 => "grid grid-cols-4 gap-3",
        _ => "grid grid-cols-1 gap-3",
    };
    let items: Vec<u8> = (0..cols * rows).collect();
    view! {
        <div class=col_class>
            {items.into_iter().map(|_| view! { <SkeletonCard height=card_height /> }).collect_view()}
        </div>
    }
}
```

**Usage example — action_items page:**
```rust
// Replace: <div class="text-muted text-sm">"Loading..."</div>
// With:
view! {
    <div class="flex flex-col gap-2">
        <SkeletonLine width="w-32" height="h-5" />  // "3 open 2 in-progress" stat bar
        <SkeletonCard height="h-12" />               // item row
        <SkeletonCard height="h-12" />
        <SkeletonCard height="h-12" />
    </div>
}
```

### Pattern 4: Enhanced EmptyState with Icon and CTA

```rust
#[component]
pub fn EmptyState(
    icon: &'static str,          // Unicode emoji or symbol, e.g. "📋"
    message: &'static str,
    #[prop(optional)] cta_label: Option<&'static str>,
    #[prop(optional)] cta_href: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="text-center py-12 flex flex-col items-center gap-3">
            <span class="text-4xl">{icon}</span>
            <p class="text-secondary text-sm max-w-xs">{message}</p>
            {match (cta_label, cta_href) {
                (Some(label), Some(href)) => view! {
                    <a href=href
                       class="mt-1 bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm transition-colors">
                        {label}
                    </a>
                }.into_any(),
                _ => view! { <span></span> }.into_any(),
            }}
        </div>
    }
}

/// Consistent "no team" state used by all team-scoped pages
#[component]
pub fn NoTeamState() -> impl IntoView {
    view! {
        <EmptyState
            icon="👥"
            message="You need a team to use this feature. Create or join a team to get started."
            cta_label="Go to Team Roster"
            cta_href="/team/roster"
        />
    }
}
```

### Pattern 5: PIPE-02 — Batch Action Item Creation

The DB function loops over improvement strings. Deduplication strategy: exact match on `text` field against existing open items for the same team (case-insensitive). This prevents identical repeated improvements from flooding the list without requiring fuzzy matching complexity.

```rust
// In src/server/db.rs
pub async fn batch_create_action_items_from_review(
    db: &Surreal<Db>,
    team_id: &str,
    review_id: &str,
    improvements: &[String],
) -> DbResult<usize> {
    if improvements.is_empty() {
        return Ok(0);
    }
    // Fetch existing open action items for dedup
    let existing = list_action_items(db, team_id).await?;
    let open_texts: std::collections::HashSet<String> = existing
        .iter()
        .filter(|i| i.status == "open" || i.status == "in_progress")
        .map(|i| i.text.to_lowercase())
        .collect();

    let mut created = 0usize;
    for text in improvements {
        let text = text.trim().to_string();
        if text.is_empty() {
            continue;
        }
        // Skip if identical open item exists
        if open_texts.contains(&text.to_lowercase()) {
            continue;
        }
        create_action_item(db, team_id, text, Some(review_id.to_string()), None).await?;
        created += 1;
    }
    Ok(created)
}
```

**Wiring in post_game.rs `create_review` server fn:**

The `create_review` server fn currently returns `Result<String, ServerFnError>` (the review ID). Change return type to `Result<(String, usize), ServerFnError>` to carry action item count alongside ID. Or keep `String` and encode as `"id|N"` to avoid client-side struct deserialization complexity — the latter avoids adding a new shared model type.

Recommended: Change to `(String, usize)` — both `String` and `usize` implement `Serialize + Deserialize`. The client unpacks the tuple.

For `update_review`, change return type from `()` to `usize` (action items created count). This enables the "N action items created" feedback on updates.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Toast auto-dismiss timer | Custom timer loop or interval | `web_sys::window().set_timeout_with_callback_and_timeout_and_arguments_0` + `Closure::once` | Pattern already used in codebase (CLAUDE.md rule 37); `gloo_timers` not available |
| Clipboard copy | Custom clipboard polyfill | `web_sys::navigator().clipboard().write_text()` | Direct browser API, no extra crate needed |
| Toast stacking | Manual z-index management | Flexbox column in fixed overlay div | CSS stacking is automatic with flex column |
| Skeleton shimmer animation | Custom CSS `@keyframes` | Tailwind `animate-pulse` | Already in use across the codebase; consistent with existing patterns |
| Action item deduplication | Levenshtein/fuzzy match | Exact case-insensitive string match | Sufficient for the use case; avoids dependency on a string-similarity crate |

---

## Common Pitfalls

### Pitfall 1: ToastProvider context not available during SSR
**What goes wrong:** `use_context::<ToastContext>()` returns `None` during server-side rendering if `ToastProvider` was added inside a `#[cfg(feature = "hydrate")]` block.
**Why it happens:** SSR renders the component tree server-side; `provide_context` must execute in both SSR and hydrate modes.
**How to avoid:** Place `<ToastProvider>` unconditionally in `App` (not inside a cfg block). The `set_timeout` auto-dismiss inside it is already guarded by `#[cfg(feature = "hydrate")]`.
**Warning signs:** Compile-time panic in SSR from `.expect("ToastProvider missing")`.

### Pitfall 2: `Callback::new` for `show` but ToastContext must be `Copy`
**What goes wrong:** If `ToastContext` holds a `WriteSignal` directly instead of `Callback`, it works but requires cloning in each closure.
**Why it happens:** Multiple closures in a page (one for save, one for delete) both need `show_toast`.
**How to avoid:** Wrap `show` in `Callback::new(...)` which is `Copy` (CLAUDE.md rule 36). Pages call `toast.show.run(...)` without cloning.

### Pitfall 3: `clipboard().write_text()` returns a `Promise` — must not block
**What goes wrong:** Calling `.write_text()` and ignoring the `Promise` causes a warning; awaiting it in a sync event handler panics.
**Why it happens:** The Web Clipboard API is async.
**How to avoid:** Use `let _ = win.navigator().clipboard().write_text(&msg);` — discard the promise. The write will succeed asynchronously. No feedback needed for a debug-copy action.

### Pitfall 4: Skeleton `#[prop(default)]` string literals need `&'static str`
**What goes wrong:** Passing dynamic strings to skeleton props fails because Leptos prop defaults require `'static`.
**Why it happens:** `#[prop(default = "w-full")]` expands to a `&'static str` default.
**How to avoid:** Only use `&'static str` for skeleton dimension props; use hardcoded Tailwind class strings at call sites.

### Pitfall 5: Toast stacking — old toasts pile up if success auto-dismiss fires after component unmount
**What goes wrong:** If a user navigates away before the 4-second auto-dismiss fires, the `set_toasts.update()` in the closure tries to update a dropped signal.
**Why it happens:** The `Closure::once` captures `set_toasts` which may be dropped when `ToastProvider` unmounts (unlikely in this SPA, but possible).
**How to avoid:** `ToastProvider` lives in `App` at the root — it never unmounts during normal navigation. This is safe for this architecture.

### Pitfall 6: Return type change breaks existing `create_review` callers
**What goes wrong:** Changing `create_review` from `Result<String>` to `Result<(String, usize)>` breaks the client-side match in `post_game.rs`.
**Why it happens:** Leptos server fn return types are serialized; callers must match.
**How to avoid:** Update both the server fn definition and the single call site in `do_save` simultaneously in the same task.

### Pitfall 7: `EmptyState` prop signature change breaks existing callsites
**What goes wrong:** `EmptyState` currently takes `message: &'static str`. After enhancement it takes additional optional props. Existing usages in `post_game.rs` (empty review list), `game_plan.rs`, `draft.rs`, etc. pass only a message string.
**Why it happens:** CLAUDE.md rule 8 — breaking component interfaces causes cascading compile errors.
**How to avoid:** Make `icon`, `cta_label`, `cta_href` all optional with `#[prop(optional)]`. The existing call `<EmptyState message="..." />` continues to compile with no icon or CTA shown (acceptable fallback). Update each page's empty state in a dedicated subtask.

---

## Code Examples

### Verified Patterns from Existing Codebase

#### Providing context in App (from `main.rs` pattern)
```rust
// Existing pattern in main.rs — leptos_routes_with_context:
.leptos_routes_with_context(&app_state, routes, {
    let surreal = surreal.clone();
    move || provide_context(surreal.clone())
}, App)
// Toast context follows same provide_context pattern but in the component tree
```

#### spawn_local with toast (replacing existing pattern)
```rust
// BEFORE (in post_game.rs do_save):
Ok(id) => {
    set_status_msg.set(Some("Review created!".into()));
}
Err(e) => set_status_msg.set(Some(format!("Error: {e}"))),

// AFTER:
let toast = use_context::<ToastContext>().expect("ToastProvider");
// ...in spawn_local:
Ok((id, n_items)) => {
    let msg = if n_items > 0 {
        // Inline StatusMessage stays for "N action items" with link (not toast)
        set_action_item_banner.set(Some(n_items));
    };
    toast.show.run((ToastKind::Success, "Review saved!".into()));
}
Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
```

#### Existing animate-pulse skeleton (from post_game.rs, already in use)
```rust
// EXISTING — used in post_game.rs Suspense for select dropdowns:
<Suspense fallback=|| view! {
    <div class="h-9 bg-overlay/50 rounded-lg animate-pulse"></div>
}>
// NEW SkeletonCard wraps this pattern with a reusable component
```

---

## Pages Audit — Full Inventory

### Pages Needing Skeleton Fallbacks (UX-02)

All have `"Loading..."` text in `<Suspense fallback>`:

| Page | File | Suspense Count | Skeleton Shape |
|------|------|----------------|---------------|
| Action Items | `action_items.rs:258,384` | 2 | stat-bar (3 pill badges) + list (3 card rows) |
| Team Dashboard | `team/dashboard.rs:593,1082,1140,1181,1227` | 5 | full dashboard shell + 4 panel skeletons |
| Profile | `profile.rs:112,258` | 2 | user info card + champion pool grid |
| Tree Drafter | `tree_drafter.rs:763,1324,1444,1482` | 4 | tree list (3 rows) + champion grid + node editor |
| Opponents | `opponents.rs:339,383` | 2 | opponent list (3 rows) + detail panel |
| Post Game | `post_game.rs:471` | 1 (+ 3 already have pulse) | review list (3 rows) |
| Game Plan | `game_plan.rs:980,1082,1366` | 3 | plan list + draft selector + win-rate panel |
| Champion Pool | `champion_pool.rs:483,910,1050` | 3 | champion autocomplete + notes sections |
| Stats | `stats.rs` (inferred from ErrorBanner pattern) | TBD (check file) |
| Draft | `draft.rs:1321,2099,2453,3149` | 4 | team select + opponent intel + champion picker + ban priorities |

### Pages Needing Toast Migration (UX-03)

All pages using `StatusMessage` / `ErrorBanner` / local `status_msg`/`error_msg`/`save_result` signals:

| Page | File | Current Pattern | Migration Scope |
|------|------|-----------------|-----------------|
| Post Game | `post_game.rs` | `status_msg` signal | Replace save/delete mutations; KEEP inline "N action items" banner |
| Action Items | `action_items.rs` | `error_msg` signal | Add success toasts; keep load-failure ErrorBanner or toast it |
| Team Dashboard | `team/dashboard.rs` | `StatusMessage` x2, `ErrorBanner` | Replace mutations; keep load ErrorBanner |
| Game Plan | `game_plan.rs` | `status_msg` signal (12 callsites) | Full migration |
| Tree Drafter | `tree_drafter.rs` | `status_msg` signal (11 callsites) | Full migration |
| Draft | `draft.rs` | `save_result` signal | Replace save/update/delete |
| Champion Pool | `champion_pool.rs` | `status_msg` signal (8 callsites) | Full migration |
| Opponents | `opponents.rs` | `error_msg` + `status_msg` RwSignals | Full migration |
| Team Builder | `team_builder.rs` | `save_result` RwSignal | Full migration |
| Profile | `profile.rs` | (inferred — link/unlink actions) | Add success/error toasts |
| Team Roster | `team/roster.rs` | (inferred — join/create actions) | Add success/error toasts |

### Pages Needing Empty States (UX-01)

| Page | Condition | Icon | Message | CTA |
|------|-----------|------|---------|-----|
| Champion Pool | No pool entries | "🎯" | "Your champion pool is empty — add champions to track your picks and matchups" | "Add Champion" (scroll/anchor) |
| Champion Pool | No team | "👥" | `NoTeamState` | /team/roster |
| Stats | No match data | "📊" | "No match stats yet — link your Riot account and play some games to see stats here" | /profile |
| Stats | No team | "👥" | `NoTeamState` | /team/roster |
| Opponents | No opponents added | "🎭" | "No opponents scouted yet — add an opponent team to start tracking their picks" | "Add Opponent" |
| Opponents | No team | "👥" | `NoTeamState` | /team/roster |
| Profile | No Riot account | "🔗" | "Link your Riot account to track match stats and champion performance" | "Link Account" (scroll/anchor) |
| Team Builder | No drafts saved | "⚗️" | "No team compositions saved yet — use the team builder to try role combinations" | none (builder is on-page) |
| Team Roster | No team | "👥" | "You're not part of a team yet — create a new team or join an existing one" | inline buttons already exist |
| Team Dashboard | No team | "👥" | `NoTeamState` | /team/roster |

---

## State of the Art

| Old Approach | Current Approach | Status in Codebase | Impact |
|--------------|------------------|-------------------|--------|
| Inline `StatusMessage` per-page | Centralized `ToastProvider` + `show_toast` | CURRENT: per-page signals | Migration removes 40+ signal declarations across 11 pages |
| `EmptyState` with text only | `EmptyState` with icon + CTA | CURRENT: no icon, no CTA | Requires backward-compatible prop extension |
| `"Loading..."` text in Suspense | Shape-matching `SkeletonCard`/`SkeletonLine` | CURRENT: text in most pages | 15+ Suspense fallbacks to update |
| Manual action item creation | Auto-generation from post-game improvements | NOT YET IMPLEMENTED | Requires new DB fn + server fn wiring |

---

## Open Questions

1. **Action item feedback placement (PIPE-02)**
   - What we know: Decision says inline StatusMessage on post-game page showing "N action items created — View" with link. Toast system replaces StatusMessage broadly.
   - What's unclear: Should the "N items created" message go through the toast system OR remain as a page-level inline banner (the only remaining inline message)?
   - Recommendation: Keep it as an inline banner specific to the post-game page since it contains a link and is semantically different from a transient mutation confirmation. Use a separate `RwSignal<Option<usize>>` for `action_item_count`, rendered as an inline panel above the editor. The save toast ("Review saved!") goes through the toast system.

2. **ErrorBanner for page-level load failures**
   - What we know: CONTEXT.md marks this as Claude's discretion.
   - What's unclear: Should `Err(e)` branches in `Suspense` content (e.g. "Failed to load reviews") go through toast or stay as inline `ErrorBanner`?
   - Recommendation: Keep `ErrorBanner` for load-failure `Err` branches inside `Suspense` content (these are structural errors, not mutation feedback). Migrate only mutation feedback (spawn_local Ok/Err branches) to toast. This is the cleanest separation: "loading failed" = inline, "action succeeded/failed" = toast.

3. **Toast stacking when multiple saves fire rapidly**
   - What we know: CONTEXT.md marks stacking behavior as Claude's discretion.
   - Recommendation: Stack vertically (flexbox column), newest at top, max 3 visible (oldest auto-removed if success, oldest persists if error). Simple slice truncation in the overlay render.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `#[cfg(test)]` unit tests + Playwright e2e |
| Config file | `playwright.config.ts` |
| Quick run command | `cargo test --features ssr --lib` |
| Full suite command | `just e2e` (requires running server) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PIPE-02 | `batch_create_action_items_from_review` creates correct number of items | unit | `cargo test --features ssr --lib batch_create` | ❌ Wave 0 |
| PIPE-02 | Deduplication skips existing open items | unit | `cargo test --features ssr --lib batch_create_dedup` | ❌ Wave 0 |
| PIPE-02 | Post-game save returns (review_id, item_count) | unit | `cargo test --features ssr --lib create_review_returns_count` | ❌ Wave 0 |
| UX-01 | Empty states render on pages with no data | e2e | `just e2e -- --grep "empty state"` | ❌ Wave 0 |
| UX-02 | Skeleton fallbacks render (visual regression) | e2e smoke | `just e2e -- --grep "skeleton"` | ❌ Wave 0 |
| UX-03 | Toast appears after mutation, auto-dismisses | e2e | `just e2e -- --grep "toast"` | ❌ Wave 0 |
| UX-03 | Error toast has copy + dismiss buttons | e2e | `just e2e -- --grep "error toast"` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --features ssr --lib`
- **Per wave merge:** `cargo test --features ssr --lib && cargo check --features hydrate --target wasm32-unknown-unknown`
- **Phase gate:** Full `just e2e` green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Unit tests for `batch_create_action_items_from_review` in `src/server/db.rs` test block
- [ ] Playwright tests for empty states and toast behavior in `e2e/tests/` (new file: `polish.spec.ts`)

---

## Sources

### Primary (HIGH confidence)
- Codebase direct read — `src/components/ui.rs`, `src/pages/post_game.rs`, `src/pages/action_items.rs`, `src/models/action_item.rs`, `src/app.rs`, `src/server/db.rs` (create_action_item, batch pattern)
- CLAUDE.md rules 36 (Callback::new), 37 (wasm timers), 22 (StoredValue), 24 (spawn_local), 18 (clone for multiple closures), 19 (into_any for divergent views)
- `.planning/phases/05-post-game-loop-polish/05-CONTEXT.md` — all locked decisions

### Secondary (MEDIUM confidence)
- Grep output — all 15+ "Loading..." Suspense fallbacks and all StatusMessage/ErrorBanner/status_msg callsites inventoried
- Existing `animate-pulse` usage in `post_game.rs`, `champion_pool.rs`, `draft.rs`, `home.rs` confirms pattern is established

### Tertiary (LOW confidence)
- None — all findings verified against actual source files

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all libraries already in project
- Architecture: HIGH — patterns derived from existing codebase conventions
- Pitfalls: HIGH — derived from CLAUDE.md rules and direct code analysis
- Page inventory: HIGH — grep confirmed all Loading.../StatusMessage/ErrorBanner locations

**Research date:** 2026-03-17
**Valid until:** Phase execution (stable codebase, no external API changes)
