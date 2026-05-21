---
phase: 18-region-variants
plan: "02"
subsystem: components/skeleton
tags: [region, skeleton, empty-state, animations, tailwind, leptos]
dependency_graph:
  requires: []
  provides:
    - "src/components/skeleton.rs::PageLoading"
    - "src/components/skeleton.rs::PageEmpty"
    - "input.css::@keyframes dem-shimmer"
    - "input.css::@keyframes pan-flicker"
    - "input.css::@keyframes pan-scan"
    - "input.css::@keyframes pan-cursor"
    - "input.css::@utility bg-scanline"
    - "input.css::@utility bg-parchment-shimmer"
  affects:
    - "Wave 2+ page-port plans (consume PageLoading as <Suspense fallback> and PageEmpty as empty-state branches)"
tech_stack:
  added: []
  patterns:
    - "move || if is_pandemonium { view!{}.into_any() } else { view!{}.into_any() } — canonical AnyView region branch"
    - "@keyframes nested inside @theme block for Tailwind v4 animate-* utilities"
    - "@utility declarations outside @theme for custom background patterns"
key_files:
  created:
    - src/components/skeleton.rs
  modified:
    - input.css
    - src/components/mod.rs
decisions:
  - "skeleton.rs placed at src/components/skeleton.rs (NOT under region/) — skeletons are page-level layout shells, not canonical design-system primitives"
  - "Unicode escape sequences used for ellipsis (\\u{2026}) and em-dash (\\u{2014}) in Rust string literals instead of literal characters"
  - "render_dem_skeleton_layout and render_pan_skeleton_layout implemented as free functions returning AnyView to avoid recursion-limit issues with nested match inside view!"
metrics:
  duration_seconds: 953
  completed_date: "2026-05-14"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 3
---

# Phase 18 Plan 02: Skeleton + Empty State Components Summary

Per-region skeleton loading states and empty-state components implemented: `PageLoading` (parchment shimmer for Demacia, xerox flicker for Pandemonium) and `PageEmpty` (heraldic prose copy vs terminal comment copy), driven by 4 new CSS `@keyframes` animations and 2 `@utility` background declarations.

## Tasks Completed

| Task | Commit | Description |
|------|--------|-------------|
| 1: Add animation keyframes + utilities to input.css | `cc7d36d` | 4 @keyframes + 4 --animate-* tokens + 2 @utility blocks |
| 2: Implement PageLoading + PageEmpty in skeleton.rs | `3731064` | 6 PageLoading + 10 PageEmpty combinations, module registered |

## Component API

### `PageLoading`

```rust
#[component]
pub fn PageLoading(
    region: String,          // "demacia" | "pandemonium"
    #[prop(into)] variant: String, // "draft" | "solo" | "team"
) -> impl IntoView
```

**Demacia grammar:** `bg-parchment-shimmer` tint + `animate-dem-shimmer` (2.4s ease-in-out opacity pulse) + serif italic caption at bottom-left.

**Pandemonium grammar:** `bg-scanline` repeating gradient + `animate-pan-flicker` (1.6s xerox flicker) + monospace caption with `animate-pan-cursor` blinking `_` terminal cursor.

### `PageEmpty`

```rust
#[component]
pub fn PageEmpty(
    region: String,          // "demacia" | "pandemonium"
    #[prop(into)] kind: String, // "draft" | "matches" | "team" | "pool" | "scout"
) -> impl IntoView
```

## Caption Strings

### PageLoading captions

| Variant | Demacia | Pandemonium |
|---------|---------|-------------|
| `draft` | "Awaiting word from the field…" | `// LOADING_DRAFT` |
| `solo` | "Reading the stars…" | `// LOADING_SOLO_PROFILE` |
| `team` | "Mustering the company…" | `// LOADING_TEAM_BRIEF` |

### PageEmpty copy strings

| Kind | Demacia | Pandemonium |
|------|---------|-------------|
| `draft` | "No drafts recorded. Begin your first campaign." | `// NO_DRAFTS_FOUND — start a session` |
| `matches` | "No matches in the ledger. Sync your history." | `// MATCH_HISTORY_EMPTY — run sync` |
| `team` | "Your company has not assembled. Join or form a team." | `// NO_TEAM — join_or_create()` |
| `pool` | "Your champion pool awaits its first entries." | `// POOL_EMPTY — add champions` |
| `scout` | "No opponent intelligence on file." | `// NO_INTEL — begin scouting` |

## Animations Added

| Name | Token | Timing | Effect |
|------|-------|--------|--------|
| `dem-shimmer` | `--animate-dem-shimmer` | 2.4s ease-in-out infinite | Opacity 0.4→0.85→0.4 (parchment breathe) |
| `pan-flicker` | `--animate-pan-flicker` | 1.6s ease-in-out infinite | Xerox scan flicker with single bright flash at 50% |
| `pan-scan` | `--animate-pan-scan` | 0.9s linear infinite | Scrolls `background-position` 0→12px (scanline drift) |
| `pan-cursor` | `--animate-pan-cursor` | 0.9s steps(1) infinite | Binary blink (opacity 1→0 at 51%) for terminal cursor `_` |

## @utility Declarations

```css
@utility bg-scanline {
  background-image: repeating-linear-gradient(
    0deg,
    color-mix(in oklab, var(--color-accent) 14%, transparent),
    color-mix(in oklab, var(--color-accent) 14%, transparent) 1px,
    transparent 1px,
    transparent 12px
  );
}

@utility bg-parchment-shimmer {
  background-color: color-mix(in oklab, var(--color-accent) 8%, var(--t-surface) 92%);
}
```

## Location Decision

`src/components/skeleton.rs` — NOT under `region/`. Rationale: skeletons are page-level layout shells consumed via `<Suspense fallback>` at the route level. They are not part of the canonical design-system primitive kit (~24 components in `region/`). Placing them at the components root keeps them visible alongside other shell-level components (`nav.rs`, `ui.rs`) and avoids polluting the primitive namespace.

## Deviations from Plan

None — plan executed exactly as written. The helper functions `render_dem_skeleton_layout` and `render_pan_skeleton_layout` match the plan template precisely. Unicode escapes were used for `…` and `—` in Rust string literals as a minor implementation detail not specified in the plan (correctness improvement, avoids source encoding issues).

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All content strings are hard-coded English labels with no PII. Fallback `_ =>` arms in all match blocks prevent panics on unexpected prop values (T-18-02-01 accepted).

## Self-Check: PASSED

| Item | Status |
|------|--------|
| `src/components/skeleton.rs` | FOUND |
| `input.css` | FOUND |
| `src/components/mod.rs` | FOUND |
| Commit `cc7d36d` | FOUND |
| Commit `3731064` | FOUND |
| `cargo check --features ssr` | PASS |
| `cargo check --features hydrate --target wasm32-unknown-unknown` | PASS |
