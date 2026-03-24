# Phase 7: UX Polish - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Five independent UX improvements: toast positioning, timestamp formatting, profile link button dedup, team join search, and roster role indicators. All are small, self-contained changes that make the app feel polished. No new features or capabilities.

</domain>

<decisions>
## Implementation Decisions

### Timestamp Display (UX-05)
- **D-01:** Relative time for <24 hours ("just now", "23 minutes ago", "6 hours ago"), absolute after ("Mar 19, 14:30")
- **D-02:** 24-hour clock format
- **D-03:** Year only shown when different from current year ("Mar 19, 14:30" vs "Dec 5, 2025 14:30")
- **D-04:** Create a shared formatting helper used across all pages that display timestamps

### Team Search UX (UX-07)
- **D-05:** Client-side filter — fetch all teams upfront, filter as user types in search input
- **D-06:** Empty search state shows "Type to search for teams" prompt (no list until user types)
- **D-07:** Search results show: team name + region + member count + "Request to Join" button
- **D-08:** Search matches team name only (not region)

### Role Icon Treatment (UX-09)
- **D-09:** Watermark-style background icons — large (48-64px), faded (~10% opacity) Community Dragon role icons
- **D-10:** Positioned in bottom-right corner of the card, partially cropped
- **D-11:** No watermark for unassigned roles (blank = visual cue to assign role)
- **D-12:** All card types get watermarks: starters and bench get role icons, coaches get a generic coach icon

### Toast Positioning (UX-04)
- **D-13:** Change toast container from `top-4` to ~`top-16` (64px) to clear the sticky nav header
- **D-14:** Keep centered horizontal position, same z-index and animation behavior

### Profile Link Button (UX-06)
- **D-15:** Verify and ensure exactly one "Link Account" button in all profile states (linked, unlinked, loading)

### Claude's Discretion
- Exact watermark opacity value (8-12% range)
- Toast top offset pixel value (whatever clears the nav)
- Timestamp helper implementation (pure Rust function vs component)
- Whether timestamp formatting happens server-side or client-side
- Search input placeholder text and styling
- Coach watermark icon choice

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — UX-04, UX-05, UX-06, UX-07, UX-09 acceptance criteria
- `.planning/ROADMAP.md` — Phase 7 success criteria (5 items)

### Toast System
- `src/components/ui.rs` — ToastProvider, ToastOverlay, current `fixed top-4` positioning (line ~110)

### Timestamps
- `src/models/action_item.rs` — `created_at: Option<String>` field pattern
- `src/models/game_plan.rs` — `created_at: Option<String>` field pattern
- `src/pages/team/dashboard.rs` — Multiple timestamp displays (join requests, recent matches, team notes)

### Profile
- `src/pages/profile.rs` — LinkRiotAccount form, current button layout (line ~237)

### Team Search
- `src/pages/team/roster.rs` — `list_teams()` resource (lines 61-73), team list display (lines 179-226)
- `src/server/db.rs` — `list_all_teams()` query

### Role Icons
- `src/pages/team/dashboard.rs` — Starter slot role icons (lines 891-897), `role_icon_url()` helper (lines 559-568), bench card layout (lines 982-1050)
- Community Dragon CDN role icon URLs used by `role_icon_url()`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `role_icon_url(role)` function in `dashboard.rs` — returns Community Dragon SVG URL per role, reuse for watermarks
- `ToastProvider` + `ToastOverlay` in `ui.rs` — only need CSS tweak for positioning
- `ChampionAutocomplete` component — reference pattern for search input (though team search is simpler)
- All model structs already have `created_at: Option<String>` fields

### Established Patterns
- Semantic tokens for styling (`bg-surface`, `text-primary`, `border-divider`) — all new UI must use these
- `Resource::new` + `Suspense` for data loading — team search will filter the existing resource
- `StoredValue::new()` for non-reactive data shared across closures
- `collect_view()` for rendering filtered lists

### Integration Points
- Toast overlay: `ui.rs` ToastOverlay component — CSS-only change
- Timestamps: every page with `created_at`/`updated_at` display — add shared helper, call from each page
- Profile: `profile.rs` — verify rendered states
- Team search: `roster.rs` — add search input + filter logic
- Role icons: `dashboard.rs` — add watermark `<img>` with absolute positioning inside card containers

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-ux-polish*
*Context gathered: 2026-03-22*
