# Phase 7: UX Polish - Research

**Researched:** 2026-03-22
**Domain:** Leptos 0.8 UI polish — toast positioning, timestamp formatting, profile dedup, team search, role watermarks
**Confidence:** HIGH

## Summary

Phase 7 is five independent CSS/Leptos UX fixes. All changes are small and self-contained. No new server functions, no new DB tables, no new pages. The codebase already has the building blocks for every requirement:

- Toast container: one CSS class change in `ui.rs`
- Timestamps: chrono is already a dependency, `<string>created_at` is already how SurrealDB stores datetimes; need a shared pure-Rust formatting helper
- Profile duplicate button: `EmptyState` has a `cta_href="#link-account"` CTA button AND the `ActionForm` has its own "Link Account" submit button — only the form button should remain
- Team search: `list_teams()` already returns all teams; add a `RwSignal<String>` filter + search input; Team model needs a `member_count` field for D-07 display
- Role watermarks: `role_icon_url()` already exists in `dashboard.rs`; add `<img>` with absolute positioning inside card containers for starters, bench, and coaches

The only non-trivial decision is timestamp formatting: whether the helper lives in `src/models/utils.rs` (compiled for both targets) or only in SSR. Since timestamps are displayed in the WASM client after being passed from server functions as `Option<String>`, the formatter must run client-side in WASM — so it must compile for both targets.

**Primary recommendation:** Put the timestamp formatter in `src/models/utils.rs` as a pure `fn format_timestamp(s: &str) -> String`. Chrono already compiles for both targets without the `wasm-bindgen` feature (string parsing/formatting only; no `Utc::now()` needed in WASM). Use `chrono::Utc::now()` only in SSR for the relative-time comparison.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Relative time for <24 hours ("just now", "23 minutes ago", "6 hours ago"), absolute after ("Mar 19, 14:30")
- **D-02:** 24-hour clock format
- **D-03:** Year only shown when different from current year ("Mar 19, 14:30" vs "Dec 5, 2025 14:30")
- **D-04:** Create a shared formatting helper used across all pages that display timestamps
- **D-05:** Client-side filter — fetch all teams upfront, filter as user types in search input
- **D-06:** Empty search state shows "Type to search for teams" prompt (no list until user types)
- **D-07:** Search results show: team name + region + member count + "Request to Join" button
- **D-08:** Search matches team name only (not region)
- **D-09:** Watermark-style background icons — large (48-64px), faded (~10% opacity) Community Dragon role icons
- **D-10:** Positioned in bottom-right corner of the card, partially cropped
- **D-11:** No watermark for unassigned roles (blank = visual cue to assign role)
- **D-12:** All card types get watermarks: starters and bench get role icons, coaches get a generic coach icon
- **D-13:** Change toast container from `top-4` to ~`top-16` (64px) to clear the sticky nav header
- **D-14:** Keep centered horizontal position, same z-index and animation behavior
- **D-15:** Verify and ensure exactly one "Link Account" button in all profile states (linked, unlinked, loading)

### Claude's Discretion
- Exact watermark opacity value (8-12% range)
- Toast top offset pixel value (whatever clears the nav)
- Timestamp helper implementation (pure Rust function vs component)
- Whether timestamp formatting happens server-side or client-side
- Search input placeholder text and styling
- Coach watermark icon choice

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UX-04 | Toast notifications render below the header section, not overlapping it | Nav is `h-14` (56px) sticky; `top-16` (64px) clears it with margin |
| UX-05 | Timestamps display human-readable format without decimal digits or "Z" suffix | Chrono parses ISO-8601 strings; formatter function in `src/models/utils.rs` |
| UX-06 | Profile page shows a single "Link Account" button, not two | `EmptyState` CTA + `ActionForm` submit both render "Link Account"; remove CTA |
| UX-07 | Team join uses a search bar with autocomplete/suggested results instead of listing all teams | `list_teams()` fetches all; add `RwSignal<String>` + filter + member_count on Team |
| UX-09 | Team roster cards display role icons (top/jg/mid/bot/sup) as visual background indicators | `role_icon_url()` exists; add absolute `<img>` to starter/bench/coach cards |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| chrono | 0.4 (already in Cargo.toml) | Parse ISO-8601 datetimes, compute age, format output | Already a dependency; compiles for both ssr and hydrate targets |
| Leptos signals | 0.8 (project standard) | `RwSignal<String>` for search input | Standard reactive pattern already used throughout |
| Tailwind CSS v4 | v4 (project standard) | CSS class changes for toast and watermark positioning | Already used for all styling |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Community Dragon CDN | latest | Role icon SVGs | Already used by `role_icon_url()` — reuse same URLs |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Pure Rust chrono formatter | Server-side formatting + pass formatted string | Server-side is simpler but duplicates strings and makes relative time stale; client-side formatter is more accurate for "23 minutes ago" |
| `RwSignal<String>` filter | Dedicated search server fn | Server search is overkill for a small team list; client-side filter is instant and avoids round-trips (D-05 explicitly locks this) |

**Installation:** No new dependencies required. Chrono is already at version 0.4 in `Cargo.toml`.

## Architecture Patterns

### Recommended Project Structure
No new files required. Changes go into:
```
src/
├── models/utils.rs          # NEW: format_timestamp() shared helper
├── models/mod.rs            # add: pub mod utils;
├── models/team.rs           # add: member_count field
├── components/ui.rs         # change: top-4 → top-16 in ToastOverlay
├── pages/profile.rs         # change: remove duplicate EmptyState CTA button
├── pages/team/roster.rs     # change: add search input + filter logic
└── pages/team/dashboard.rs  # change: add watermark imgs to role/bench/coach cards
```

### Pattern 1: Shared Timestamp Formatter

**What:** Pure function `format_timestamp(s: &str) -> String` in `src/models/utils.rs`. Takes an ISO-8601 string (as stored in DB), returns a display string per D-01/D-02/D-03.

**When to use:** Call at every `created_at`/`updated_at` display site.

**Implementation approach:**
- Compile for both SSR and WASM — no `#[cfg]` needed because chrono parses/formats strings on both targets without `js_sys`
- `Utc::now()` is needed only for the relative-time comparison. Chrono provides this on both targets (uses `std::time::SystemTime` under the hood, which works in WASM since wasm32 targets have a clock)
- SurrealDB `<string>created_at` already outputs ISO-8601 with UTC suffix (e.g. `"2026-03-19T14:30:00.000000000Z"`)
- Parse with `chrono::DateTime::parse_from_rfc3339` or `DateTime::<Utc>::from_str`

**Example skeleton:**
```rust
// src/models/utils.rs
use chrono::{DateTime, Utc, Duration};

pub fn format_timestamp(s: &str) -> String {
    let Ok(dt) = s.parse::<DateTime<Utc>>() else {
        return s.to_string();
    };
    let now = Utc::now();
    let age = now.signed_duration_since(dt);

    if age < Duration::minutes(1) {
        "just now".to_string()
    } else if age < Duration::hours(24) {
        let mins = age.num_minutes();
        let hours = age.num_hours();
        if hours > 0 {
            format!("{hours} hour{} ago", if hours == 1 { "" } else { "s" })
        } else {
            format!("{mins} minute{} ago", if mins == 1 { "" } else { "s" })
        }
    } else {
        // Absolute format
        let current_year = now.format("%Y").to_string();
        let dt_year = dt.format("%Y").to_string();
        if current_year == dt_year {
            dt.format("%-d %b, %H:%M").to_string()  // "19 Mar, 14:30"
        } else {
            dt.format("%-d %b %Y, %H:%M").to_string() // "5 Dec 2025, 14:30"
        }
    }
}
```

**Gotcha — chrono in WASM:** Chrono's `Utc::now()` uses `std::time::SystemTime::now()`, which is supported in WASM since wasm32-unknown-unknown has a clock stub. However, to avoid potential issues with `wasm-bindgen`-less WASM environments, wrap `Utc::now()` calls in the formatter. If needed at test time, pass `now` as a parameter instead of computing it inside.

### Pattern 2: Search Filter in `roster.rs`

**What:** Replace the full team list render with a search input + filtered view.

**When to use:** Inside the "Join an Existing Team" section of `RosterPage`.

**Implementation approach:**
```rust
let (search_query, set_search_query) = signal(String::new());

// In render, replace teams list with:
let filtered: Vec<Team> = teams.iter()
    .filter(|t| {
        let q = search_query.get();
        q.is_empty() || t.name.to_lowercase().contains(&q.to_lowercase())
    })
    .cloned()
    .collect();
```

**State for empty search (D-06):** Show "Type to search for teams" when `search_query.get().is_empty()`. Show filtered results when non-empty. Show "No teams match" when filtered is empty but query is non-empty.

**Member count (D-07):** The `Team` model currently has no `member_count` field. Two options:
1. Add `member_count: u32` to the `Team` struct and update `list_all_teams()` to join/count
2. Omit member count from display

Decision D-07 requires it. The SurrealQL approach:
```surql
SELECT *, count(->team_member) AS member_count FROM team ORDER BY name ASC
```
This uses SurrealDB graph notation. Alternative — separate `COUNT()` in a subquery or a Rust-side aggregation. The cleanest approach is a new `TeamWithCount` struct or adding `member_count: Option<u32>` to the existing `Team` model.

### Pattern 3: Watermark Role Icons in `dashboard.rs`

**What:** Absolute-positioned `<img>` inside each role slot / bench card / coach card.

**CSS approach:** Card container needs `relative overflow-hidden`. The watermark img uses `absolute bottom-0 right-0 w-12 h-12 opacity-10` (or `opacity-[0.10]`). The `invert` class already used for the foreground icons works here too.

**Starter slot cards:** Already have the container div. Add `relative` to the container, insert a watermark `<img>` before the slot content.

**Bench cards:** Same — the `bg-elevated border border-divider rounded px-4 py-3 flex items-center justify-between` row needs `relative` added.

**Coach cards:** Use a generic coach icon. Community Dragon has no standard "coach" SVG. Options:
- Use a clipboard/whiteboard icon (inline SVG or heroicons)
- Use a generic person SVG
- Use the same support icon as a proxy

The bench card `role` field is already available per `m.role`; use it to conditionally render the watermark.

**For starters:** Role label is the slot role (already known from the 5-slot grid).
**For bench:** `m.role` field holds the assigned role (top/jungle/mid/bot/support/coach/unassigned). Apply D-11: skip watermark when role is "unassigned".

**Example watermark snippet:**
```rust
// Inside a card container div with class="relative overflow-hidden ..."
{if !role_icon_url(role).is_empty() {
    view! {
        <img
            src=role_icon_url(role)
            alt=""
            aria-hidden="true"
            class="absolute bottom-0 right-0 w-14 h-14 opacity-10 invert pointer-events-none select-none translate-x-2 translate-y-2"
        />
    }.into_any()
} else {
    view! { <span></span> }.into_any()
}}
```

### Pattern 4: Toast Positioning

**What:** One CSS class change in `ToastOverlay` in `src/components/ui.rs`.

**Current:** `class="fixed top-4 left-1/2 -translate-x-1/2 z-50 ..."`

**Target:** `top-4` → `top-16`. The nav is `h-14` (56px). `top-16` is 64px — clears the nav with 8px margin.

**Tailwind note:** `top-16` = 4rem = 64px in Tailwind v4 default scale. This is verified against the Tailwind spacing scale (4 * 4px per unit = 64px).

### Pattern 5: Profile Button Dedup

**What:** The profile page renders two "Link Account" buttons when the user has no Riot account linked:
1. `EmptyState cta_label="Link Account" cta_href="#link-account"` — scrolls to the form (line ~214-220)
2. `ActionForm`'s submit `<button>"Link Account"</button>` (line ~237-243)

**Fix:** Remove the `cta_label`/`cta_href` props from the `EmptyState` call in the unlinked state. Keep only the form's submit button. The `EmptyState` icon and message remain.

When riot is already linked: No "Link Account" button appears at all — only the green "Linked: ..." text and the form for re-linking. Verify that the submit button should remain available even when already linked (for re-linking). Based on D-15: "exactly one Link Account button in all profile states" — the form submit button is the one to keep.

### Anti-Patterns to Avoid
- **Do not use `js_sys::Date::now()` for timestamp relative comparison** — rely on chrono's `Utc::now()` which works in both targets
- **Do not add a `position: relative` override via `style=`** — add `relative` as a Tailwind class on the existing card container div
- **Do not re-fetch teams on every keystroke** — filter the already-loaded `teams_resource` client-side (locked by D-05)
- **Do not `.unwrap()` the chrono parse** — timestamps from DB are normally valid but may be empty strings on old records; fall back to raw string on parse failure

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Date/time formatting | Custom string parser | `chrono::DateTime::parse_from_rfc3339` | Handles timezone, leap seconds, fractional seconds correctly |
| Fuzzy search | Levenshtein or substring scoring | Plain `str::contains` case-insensitive | D-08 locks match to team name only, simple contains is sufficient |
| Role icon assets | Host custom SVG files | Community Dragon CDN (already used) | Already referenced in `role_icon_url()`, no hosting needed |

## Common Pitfalls

### Pitfall 1: Timestamp Format from SurrealDB Cast
**What goes wrong:** `<string>created_at AS created_at` in SurrealDB produces a string like `"2026-03-19T14:30:00.000000000Z"` with 9 decimal digit nanoseconds. `parse_from_rfc3339()` handles this format fine (RFC 3339 allows fractional seconds).
**Why it happens:** SurrealDB's datetime type stores nanosecond precision.
**How to avoid:** Use `s.parse::<DateTime<Utc>>()` which calls the `FromStr` impl from chrono; this accepts any RFC 3339 / ISO-8601 format including nanosecond precision.
**Warning signs:** Parse returning `Err` with "trailing characters" message — means the format doesn't match.

### Pitfall 2: `overflow-hidden` Clips More Than Expected
**What goes wrong:** Adding `overflow-hidden` to bench row cards clips dropdown selects (the role select) that extend below the card boundary.
**Why it happens:** The bench card row has an inline `<select>` for role assignment. `overflow-hidden` clips child elements outside the box.
**How to avoid:** On bench rows (which have the role `<select>`), use `overflow-hidden` with care — or only apply it at the card's inner background layer while leaving the select outside the clipped area. For the starter slots (no select), `overflow-hidden` is safe. Test that the role dropdown still opens after adding the class.

### Pitfall 3: `member_count` Requires DB or Model Change
**What goes wrong:** D-07 requires showing member count in search results, but `Team` struct has no such field and `list_all_teams()` returns `Vec<Team>` without count.
**Why it happens:** The team list query was built before this requirement existed.
**How to avoid:** Either (a) add `member_count: Option<u32>` to the `Team` model and update the query, or (b) create a separate `TeamListItem` model for the roster search display only. Option (a) is simpler and keeps the type used everywhere.

**SurrealDB query approach:**
```surql
SELECT *, array::len(->team_member) AS member_count FROM team ORDER BY name ASC
```
Or using `count()` in a subselect:
```surql
SELECT *, (SELECT count() FROM team_member WHERE team = $parent.id GROUP ALL)[0].count AS member_count FROM team ORDER BY name ASC
```
The `array::len` approach requires that `->team_member` graph edge resolution works correctly. The subselect approach is more explicit. Both are valid SurrealDB 3.x.

### Pitfall 4: Duplicate Link Button Re-appears After Riot Link
**What goes wrong:** After fixing UX-06, the form submit button persists in the "already linked" state, showing two buttons again (the linked confirmation + the re-link button).
**Why it happens:** The `ActionForm` with the submit button is always rendered regardless of link state.
**How to avoid:** Check the current profile state: when `riot_name` is `Some`, the user is already linked. Keep the form visible (for re-linking) but verify no second CTA appears. The `EmptyState` CTA is the one to remove — the form submit is the canonical single button.

### Pitfall 5: Bench Role Values Don't Match `role_icon_url()` Keys
**What goes wrong:** The bench member's `m.role` is compared against `role_icon_url()` which matches on "top", "jungle", "mid", "bot", "support". But the bench also uses values like "unassigned", "coach", "sub" — those return `""` and should show no watermark (D-11).
**Why it happens:** `role_icon_url` is defined for the 5 game roles only; other roles fall through to `_ => ""`.
**How to avoid:** The existing `_ => ""` branch already handles this. Just check `!role_icon_url(role).is_empty()` before rendering watermark — which is already the pattern used for foreground icons (line ~891-897 in dashboard.rs).

## Code Examples

### Timestamp Formatter (verified pattern from existing chrono usage in db.rs)
```rust
// src/models/utils.rs
// Source: Cargo.toml chrono 0.4 + existing usage in src/server/db.rs lines 3916-3929
use chrono::{DateTime, Duration, Utc};

pub fn format_timestamp(s: &str) -> String {
    let Ok(dt) = s.parse::<DateTime<Utc>>() else {
        return s.to_string(); // Graceful fallback for empty or malformed strings
    };
    let now = Utc::now();
    let age = now.signed_duration_since(dt);

    if age < Duration::zero() {
        // Future timestamp (clock skew) — show absolute
        return format_absolute(dt, &now);
    }
    if age < Duration::minutes(1) {
        "just now".to_string()
    } else if age < Duration::hours(24) {
        let hours = age.num_hours();
        let mins = age.num_minutes();
        if hours > 0 {
            format!("{hours} hour{} ago", if hours == 1 { "" } else { "s" })
        } else {
            format!("{mins} minute{} ago", if mins == 1 { "" } else { "s" })
        }
    } else {
        format_absolute(dt, &now)
    }
}

fn format_absolute(dt: DateTime<Utc>, now: &DateTime<Utc>) -> String {
    let dt_year = dt.format("%Y").to_string();
    let now_year = now.format("%Y").to_string();
    if dt_year == now_year {
        dt.format("%-d %b, %H:%M").to_string()  // "19 Mar, 14:30"
    } else {
        dt.format("%-d %b %Y, %H:%M").to_string() // "5 Dec 2025, 14:30"
    }
}
```

### Usage at call sites
```rust
// In dashboard.rs, roster.rs, champion_pool.rs:
use crate::models::utils::format_timestamp;

// Replace:
{note.created_at.clone().unwrap_or_default()}
// With:
{note.created_at.as_deref().map(format_timestamp).unwrap_or_default()}
```

### Team Search Filter (client-side, D-05)
```rust
// In roster.rs RosterPage component:
let (search_query, set_search_query) = signal(String::new());

// Inside the Suspense Ok(teams) branch:
let search_val = search_query.get();
if search_val.is_empty() {
    view! {
        <p class="text-muted text-sm">"Type to search for teams..."</p>
    }.into_any()
} else {
    let filtered: Vec<_> = teams.iter()
        .filter(|t| t.name.to_lowercase().contains(&search_val.to_lowercase()))
        .cloned()
        .collect();
    if filtered.is_empty() {
        view! { <p class="text-dimmed text-sm">"No teams match your search."</p> }.into_any()
    } else {
        view! {
            <div class="flex flex-col gap-2">
                {filtered.into_iter().map(|team| { /* existing card structure */ }).collect_view()}
            </div>
        }.into_any()
    }
}
```

### Toast CSS Fix
```rust
// src/components/ui.rs line 110 — change only top-4 to top-16:
// Before:
<div class="fixed top-4 left-1/2 -translate-x-1/2 z-50 flex flex-col gap-2 items-center pointer-events-none">
// After:
<div class="fixed top-16 left-1/2 -translate-x-1/2 z-50 flex flex-col gap-2 items-center pointer-events-none">
```

### Role Watermark in Card
```rust
// Starter slot container: add `relative overflow-hidden` to div class
// Then inside, before slot content:
{if !role_icon_url(role).is_empty() {
    view! {
        <img
            src=role_icon_url(role)
            alt=""
            aria-hidden="true"
            class="absolute bottom-0 right-0 w-14 h-14 opacity-10 invert pointer-events-none select-none translate-x-2 translate-y-2"
        />
    }.into_any()
} else {
    view! { <span></span> }.into_any()
}}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Raw ISO timestamp string in UI | Formatted relative/absolute display | Phase 7 | Users see "6 hours ago" instead of "2026-03-19T14:30:00.000000000Z" |
| Full team list | Search-filtered list | Phase 7 | Scales to large team counts; better UX |
| Toast at top-4 overlaps nav | Toast at top-16 clears nav | Phase 7 | Visual polish |

## Open Questions

1. **Member count in SurrealDB query**
   - What we know: `Team` model has no `member_count` field; `list_all_teams()` returns `Vec<Team>` via `DbTeam` struct
   - What's unclear: Whether `array::len(->team_member)` graph traversal works reliably in SurrealDB 3.x for this query pattern, or whether a subselect COUNT is safer
   - Recommendation: Use the subselect COUNT pattern (same pattern as `count_pending_join_requests()` in db.rs line 782-785 which already works); add `member_count: Option<u32>` to `Team` struct

2. **Chrono `Utc::now()` in WASM target**
   - What we know: Chrono 0.4 uses `std::time::SystemTime::now()` which is available in wasm32-unknown-unknown via `wasi-clock`; existing WASM targets in this project compile successfully with chrono (it's not feature-gated to `ssr`)
   - What's unclear: Whether `Utc::now()` specifically triggers any linker warnings in the WASM target
   - Recommendation: Put the formatter in `src/models/utils.rs` (compiled for both targets) and test with `cargo check --features hydrate --target wasm32-unknown-unknown` after adding the function

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Playwright (TypeScript) |
| Config file | `e2e/playwright.config.ts` |
| Quick run command | `cd /home/jasper/Repositories/lol_team_companion/e2e && npx playwright test audit-team.spec.ts audit-misc-pages.spec.ts --reporter=line` |
| Full suite command | `cd /home/jasper/Repositories/lol_team_companion/e2e && npx playwright test` |

Also: `cargo test --features ssr --lib` for Rust unit tests.

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UX-04 | Toast appears below nav header (not overlapping) | visual/smoke | `npx playwright test audit-misc-pages.spec.ts -g "toast"` | ❌ Wave 0 |
| UX-05 | Timestamps display without "Z" or decimals | unit | `cargo test --features ssr --lib format_timestamp` | ❌ Wave 0 |
| UX-06 | Profile page: exactly one "Link Account" button | e2e | `npx playwright test audit-misc-pages.spec.ts -g "profile.*link"` | ❌ Wave 0 |
| UX-07 | Team roster search filters teams as user types | e2e | `npx playwright test audit-team.spec.ts -g "search"` | ❌ Wave 0 |
| UX-09 | Roster cards show role watermark icons | visual/smoke | `npx playwright test audit-team.spec.ts -g "watermark\|role icon"` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown`
- **Per wave merge:** `cargo test --features ssr --lib` + targeted Playwright spec
- **Phase gate:** Full e2e suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/models/utils.rs` — new file with `format_timestamp()` + unit tests in `#[cfg(test)]` block
- [ ] Playwright test for UX-04: toast positioning (visual check with screenshot assertion or element bounding box check)
- [ ] Playwright test for UX-06: profile page single button assertion
- [ ] Playwright test for UX-07: roster search interaction
- [ ] Playwright test for UX-09: watermark presence assertion

*All existing test infrastructure covers other requirements; only these 5 UX-specific tests are missing.*

## Sources

### Primary (HIGH confidence)
- Direct code inspection of `src/components/ui.rs` — toast positioning at `top-4`, `ToastOverlay` structure
- Direct code inspection of `src/pages/profile.rs` — duplicate button at lines 214-220 and 237-243
- Direct code inspection of `src/pages/team/roster.rs` — current team list implementation
- Direct code inspection of `src/pages/team/dashboard.rs` — `role_icon_url()` at lines 559-568, existing role icon pattern at lines 891-897
- Direct code inspection of `src/server/db.rs` — `list_all_teams()` at line 602, chrono usage pattern at lines 3916-3929
- Direct code inspection of `Cargo.toml` — chrono 0.4 already present, no feature gate
- Direct code inspection of `schema.surql` — all `created_at` fields are `TYPE datetime DEFAULT time::now()`

### Secondary (MEDIUM confidence)
- `src/components/nav.rs` line 184-186 — nav is `sticky top-0 z-50` with `h-14` (56px); `top-16` (64px) clears it

### Tertiary (LOW confidence)
- Chrono WASM compatibility: based on knowledge that `std::time::SystemTime` is available in wasm32-unknown-unknown; requires verification with `cargo check --features hydrate --target wasm32-unknown-unknown` after implementation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies already present in Cargo.toml
- Architecture: HIGH — all integration points verified by direct code inspection
- Pitfalls: HIGH — identified from actual code patterns (overflow-hidden/select conflict, role value mismatch)
- Chrono WASM: MEDIUM — works by design but not verified empirically in this project

**Research date:** 2026-03-22
**Valid until:** 2026-04-22 (stable stack; only risk is SurrealDB 3.x COUNT query syntax)
