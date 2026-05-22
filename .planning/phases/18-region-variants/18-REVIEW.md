---
phase: 18-region-variants
reviewed: 2026-05-22T09:00:00Z
depth: standard
files_reviewed: 34
files_reviewed_list:
  - input.css
  - schema.surql
  - src/components/draft_board.rs
  - src/components/mod.rs
  - src/components/nav.rs
  - src/components/region/chrome.rs
  - src/components/region/controls.rs
  - src/components/region/data_viz.rs
  - src/components/region/layout.rs
  - src/components/region/mod.rs
  - src/components/region/ornaments.rs
  - src/components/region/solo.rs
  - src/components/region/typography.rs
  - src/components/skeleton.rs
  - src/models/user.rs
  - src/pages/admin/invites.rs
  - src/pages/auth/login.rs
  - src/pages/auth/register.rs
  - src/pages/champion_pool.rs
  - src/pages/closed_beta.rs
  - src/pages/draft.rs
  - src/pages/game_plan.rs
  - src/pages/home.rs
  - src/pages/legal/datenschutz.rs
  - src/pages/legal/impressum.rs
  - src/pages/match_detail.rs
  - src/pages/post_game.rs
  - src/pages/profile.rs
  - src/pages/solo_dashboard.rs
  - src/pages/stats.rs
  - src/pages/team/dashboard.rs
  - src/pages/tree_drafter.rs
  - src/server/auth.rs
  - src/server/db.rs
findings:
  critical: 2
  warning: 4
  info: 3
  total: 9
status: issues_found
---

# Phase 18: Code Review Report

**Reviewed:** 2026-05-22
**Depth:** standard
**Files Reviewed:** 34
**Status:** issues_found

## Summary

Phase 18 introduces ~24 region-aware Leptos primitives, per-region skeleton/empty-state components, mode-toggle persistence for three routes, and structural region variants for the seven designed page pairs. The overall architecture is sound.

Positive findings worth noting:

- The three new mode-setter server functions (`set_draft_mode_pref`, `set_solo_mode_pref`, `set_team_dashboard_mode_pref`) all perform allowlist validation before the async auth extraction — correct ordering that short-circuits invalid input cheaply and prevents arbitrary string injection into the user table.
- `ChildrenFn` vs `Children` is correctly applied throughout region primitives: every component that calls `children()` in both region arms uses `ChildrenFn`; single-arm callers use `Children`.
- `.into_any()` is present on both arms of every region branch, satisfying the `AnyView` pattern requirement.
- Utility routes (`/auth/*`, `/admin/*`, `/legal/*`, `/stats`, `/game-plan`, `/profile`, `/closed-beta`) contain zero `is_pandemonium` or `InitialTheme` conditionals — REQ-7 satisfied.
- `PublicUser` correctly exposes the three new mode fields to the client while excluding `password_hash`, `email`, and `riot_puuid`.
- `schema.surql` adds the three mode fields with `IF NOT EXISTS` and `DEFAULT 'auto'` — idempotent, safe for existing users.

Two critical issues follow: a hardcoded stale DDragon patch version in `ChampPortrait` that will 404 for any champion released after patch 14.24.1, and a missing ownership authorization check in `update_draft` allowing any authenticated user to overwrite any draft. Four warnings cover an `inner_html` XSS surface, a duplicate SVG gradient id, a raw `rgba()` shadow color, and a naming collision between the new `Icon` in `chrome.rs` and the existing `Icon` in `src/components/icon.rs`.

---

## Critical Issues

### CR-01: Hardcoded stale DDragon patch version in `ChampPortrait` — portraits 404 for modern champions

**File:** `src/components/region/chrome.rs:33`
**Issue:** The `"square"` kind of `ChampPortrait` is hardcoded to DDragon patch `14.24.1`. Every other champion image URL in the codebase uses `15.6.1` (see `match_detail.rs:102`, `stats.rs:268`, `team_builder.rs:246`, `personal_learnings.rs:182`). Any champion released after patch 14.24.1 — including Mel, Ambessa, Aurora, Smolder, Hwei, and others — will return a 404 when rendered via `<ChampTile>` (which calls `ChampPortrait kind="square"` internally). This breaks champion tiles in the draft carousel, champion pool, team dashboard Pandemonium roster section, and every other surface that uses `ChampTile` with a recently-released champion.

**Fix:** Update to match the version used elsewhere. Longer term, derive from `data_dragon::fetch_latest_version()` at startup:
```rust
// Minimum fix — matches existing codebase convention:
_ => format!(
    "https://ddragon.leagueoflegends.com/cdn/15.6.1/img/champion/{key}.png"
),
```
Note: the `"loading"` and `"splash"` kinds at lines 27 and 30 use version-independent CDN paths (`/cdn/img/champion/loading/`, `/cdn/img/champion/centered/`) and are not affected.

---

### CR-02: `update_draft` server function missing ownership authorization — any authed user can overwrite any draft

**File:** `src/pages/draft.rs:107-138`
**Issue:** `update_draft` authenticates the caller (`let _user = auth.user.ok_or_else(...)?`) but immediately discards the user identity (underscore prefix) without verifying team membership or draft ownership. `db::update_draft` (db.rs:1044) issues an unconditional `UPDATE type::record('draft', $draft_key) SET ...` with no `WHERE` predicate checking team membership. Any authenticated user who knows a draft's SurrealDB record ID can overwrite its name, picks, bans, win conditions, and tags for any other team's draft. The server function URL includes a hash suffix that changes on recompile, but the HTTP endpoint is publicly reachable to authenticated sessions.

**Fix:** Add a team-membership check before calling `db::update_draft`:
```rust
let auth: AuthSession = leptos_axum::extract().await?;
let user = auth
    .user
    .ok_or_else(|| ServerFnError::new("Not logged in"))?;  // remove underscore
let db = use_context::<Arc<Surreal<Db>>>()
    .ok_or_else(|| ServerFnError::new("No DB context"))?;

// Verify draft belongs to a team the user is a member of.
// Requires a new db helper: SELECT team FROM draft WHERE id = type::record('draft', $key)
let draft_team = db::get_draft_team_id(&db, &draft_id)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Draft not found"))?;

let user_team = db::get_user_team_id(&db, &user.id)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Not a team member"))?;

if draft_team != user_team {
    return Err(ServerFnError::new("Not authorized to edit this draft"));
}
```

---

## Warnings

### WR-01: `inner_html` in `Icon` component creates latent XSS surface

**File:** `src/components/region/chrome.rs:191-194`
**Issue:** The `Icon` component renders SVG markup via `inner_html=svg_html`. The `svg_html` value is built from a `match` on the `name: String` prop — today all call sites pass static string literals (`"search"` in `champion_picker.rs:113`, `"feather"` in `bug_report_widget.rs:106`), so there is no current injection risk. However, `inner_html` bypasses Leptos' automatic HTML escaping unconditionally. If a future caller passes a `name` value derived from user data, a URL query param, or a DB-stored icon key, the resulting SVG HTML is injected unescaped into the DOM — XSS. The pattern is high risk for drift because nothing in the type system prevents a `String` from user input reaching this prop.

**Fix:** Replace `inner_html` with native Leptos `view!` arms. All SVG paths are compile-time constants:
```rust
match name.as_str() {
    "home" => view! {
        <svg viewBox="0 0 24 24" width=size height=size fill="none"
             stroke="currentColor" stroke-width="1.5"
             stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 12L12 3l9 9"/>
            <path d="M9 21V12h6v9"/>
            <path d="M3 12v9h18v-9"/>
        </svg>
    }.into_any(),
    // ... remaining arms
    _ => view! { <span></span> }.into_any(),
}
```
This eliminates the `inner_html` surface entirely and makes the XSS risk structurally impossible.

---

### WR-02: Duplicate `id="sparkline-grad"` — DOM uniqueness violation when multiple `Sparkline` instances render

**File:** `src/components/region/data_viz.rs:96`
**Issue:** `<linearGradient id="sparkline-grad">` uses a static id. SVG `id` attributes must be unique per document. If multiple `<Sparkline>` components render simultaneously on a page — a likely scenario when solo dashboard adds LP sparkline alongside per-champion stat sparklines in a future phase — all emit `id="sparkline-grad"` into the DOM. `fill="url(#sparkline-grad)"` resolves to the first matching definition per SVG spec; subsequent instances may silently render with no gradient fill or the wrong gradient in browsers with strict id-uniqueness enforcement. Currently `Sparkline` has no call sites other than its definition, but the bug exists in the component and will surface when it gets used.

**Fix:** Accept a unique gradient id via prop, or derive one from the data to make it call-site unique:
```rust
#[component]
pub fn Sparkline(
    data: Vec<f64>,
    #[prop(optional, default = 120)] width: u32,
    #[prop(optional, default = 32)] height: u32,
    #[prop(optional, into)] gradient_id: Option<String>,
) -> impl IntoView {
    let grad_id = gradient_id.unwrap_or_else(|| "sparkline-grad".to_string());
    // Use grad_id in the <linearGradient id=...> and fill="url(#...)"
}
```
Call sites use stable ids: `<Sparkline data=lp_data gradient_id="sparkline-lp" />`.

---

### WR-03: Raw `rgba(0,0,0,0.4)` in `RiotTape` box-shadow violates no-raw-color rule

**File:** `src/components/region/ornaments.rs:109`
**Issue:** The `RiotTape` component inline style contains `box-shadow: 0 1px 0 rgba(0,0,0,0.4), ...`. Per CLAUDE.md: "no raw hex colors in `src/components/`". The no-raw-color rule covers raw rgba values used as design tokens, not just hex. All other color values in this file use `var(--color-accent)`, `var(--accent-2)`, `var(--t-accent-contrast)` etc. The hardcoded `rgba(0,0,0,0.4)` is non-semantic and will look wrong if a theme introduces a non-dark base (e.g., a future light-mode Pandemonium skin).

**Fix:** Define a CSS custom property in `input.css` and reference it:
```css
/* input.css — in the @theme block */
--shadow-tape-drop: rgba(0, 0, 0, 0.4);
```
```rust
// ornaments.rs RiotTape style string:
format!(
    "width: {}px; height: 22px; position: relative; \
     background: var(--accent-3); transform: rotate(-1.2deg); \
     box-shadow: 0 1px 0 var(--shadow-tape-drop, rgba(0,0,0,0.4)), \
     2px 2px 0 var(--color-accent), -2px -1px 0 var(--accent-2); \
     overflow: hidden;",
    width
)
```

---

### WR-04: `Icon` component in `chrome.rs` shadows existing `Icon` in `src/components/icon.rs` — silent rendering failures

**File:** `src/components/region/chrome.rs:155-198`
**Issue:** `src/components/icon.rs` already exports a `pub fn Icon` component handling icon names `"feather"` and `"search"` (via native `view!` arms, no `inner_html`). Phase 18 adds a second `Icon` component in `chrome.rs` handling a different set of names: `"home"`, `"user"`, `"swords"`, `"tree"`, `"chart"`, `"calendar"`, `"sword"`, `"shield"`. Both are re-exported through `pub use crate::components::region::*` and `pub use crate::components::*`. Code importing `Icon` from `crate::components::region::*` will get the `chrome.rs` version; code importing from `crate::components::*` will get `icon.rs`. A caller passing `name="search"` to the `chrome.rs` version gets an empty `<span>` (the `_ => None` fallthrough). A caller passing `name="home"` to `icon.rs` gets an empty span for the same reason. These failures are silent — no compile error, no panic, just missing icons.

**Fix:** Consolidate into a single `Icon` in `src/components/icon.rs`. Move the eight SVG paths from `chrome.rs` into `icon.rs` as additional match arms (rewritten as native `view!`, not `inner_html` — per WR-01 fix). Remove `Icon` from `chrome.rs`. If callers need to import it from the region path, add `pub use crate::components::icon::Icon;` to `chrome.rs` or `region/mod.rs`.

---

## Info

### IN-01: Pandemonium team dashboard sections 3-7 render hardcoded placeholder data

**File:** `src/pages/team/dashboard.rs:1835, 1848, 1852, 1870, 1907, 1953`
**Issue:** Seven `// TODO(future phase):` comments mark sections of `PandemoniumTeamDashboard`, `PandemoniumBansPanel`, and `PandemoniumThreatsPanel` that render hardcoded data rather than live DB values. Specifically: captain's note is hardcoded `"Win the early game and contest first dragon"`, reasoned bans are hardcoded Yasuo/Yone/Zed/Akali, pool-ready and opponent-intel sections are stubbed. Per the review scope instructions, these are Info-level as intentional plan 18-06 deferrals with explicit future-phase notes. Recorded for tracking.

**Fix:** Wire each section to its data source when the corresponding feature (ban-reasoning, opponent-intel, team notes) ships. The TODO comments correctly identify the data sources.

---

### IN-02: `MoodMeter` missing `aria-label` — ARIA `meter` role requires accessible name

**File:** `src/components/region/data_viz.rs:128`
**Issue:** `MoodMeter` sets `role="meter"` with `aria-valuenow`, `aria-valuemin`, `aria-valuemax` but provides no accessible name (`aria-label` or `aria-labelledby`). ARIA 1.2 requires meters to have an accessible name; screen readers announce "meter, value is 70" with no context about what is being measured. Currently `MoodMeter` is used in the Pandemonium team dashboard roster section where context is provided visually by the adjacent `"MOOD"` text, but that text is not associated via `aria-labelledby`.

**Fix:** Add an optional `label` prop defaulting to `"Team mood"`:
```rust
#[component]
pub fn MoodMeter(
    value: f64,
    #[prop(optional, into)] label: Option<String>,
) -> impl IntoView {
    let clamped = value.clamp(0.0, 1.0);
    let filled = (clamped * 5.0).round() as usize;
    let aria_label = label.unwrap_or_else(|| "Mood".to_string());
    view! {
        <div class="flex gap-0.5" role="meter"
             aria-label=aria_label
             aria-valuenow=format!("{:.0}", clamped * 100.0)
             aria-valuemin="0" aria-valuemax="100">
            // segments unchanged
        </div>
    }
}
```

---

### IN-03: `Badge` fallback for `"success"`, `"warning"`, `"info"` tones renders as indistinct `bg-elevated text-muted`

**File:** `src/components/region/controls.rs:231-233`
**Issue:** The `Badge` component comment says `"success"`, `"warning"`, `"info"` tones fall back to `"bg-elevated text-muted"` because those tokens are not defined in `input.css`. However, `input.css` at lines 177-179 and 268-305 does define `--color-success`, `--color-warning`, `--color-info` as CSS custom properties (via `--success`, `--warning`, `--danger` vars). The comment is stale and the badge is silently rendering the wrong colors — a `"warning"` badge on the solo dashboard pool-gap section (used at `solo_dashboard.rs:988-1000`) renders with the same muted style as a neutral badge instead of the amber/yellow warning color. Users get no visual distinction between warning and neutral severity.

**Fix:** Update the `Badge` tone-class mapping to use the defined tokens:
```rust
let tone_class = match tone.as_deref().unwrap_or("neutral") {
    "accent"  => "bg-accent text-accent-contrast",
    "danger"  => "bg-danger/15 text-danger",
    "success" => "bg-success/15 text-success",   // token IS defined
    "warning" => "bg-warning/15 text-warning",   // token IS defined
    "info"    => "bg-info/15 text-info",         // token IS defined
    _         => "bg-elevated text-muted",        // neutral
};
```
Remove the stale comments claiming these tokens are absent.

---

_Reviewed: 2026-05-22_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
