---
phase: 05-post-game-loop-polish
plan: "01"
subsystem: ui-components
tags: [toast, skeleton, empty-state, leptos, wasm, ui-primitives]
dependency_graph:
  requires: []
  provides: [ToastProvider, ToastContext, ToastKind, ToastEntry, SkeletonLine, SkeletonCard, SkeletonGrid, EmptyState, NoTeamState]
  affects: [src/components/ui.rs, src/app.rs]
tech_stack:
  added: [Navigator (web-sys feature), Clipboard (web-sys feature), AtomicU64 (std)]
  patterns: [leptos-context-provider, callback-copy, stored-value, atomic-id-gen, cfg-hydrate-guard]
key_files:
  created: []
  modified:
    - src/components/ui.rs
    - src/app.rs
    - Cargo.toml
decisions:
  - "Used std::sync::atomic::AtomicU64 in StoredValue instead of Cell<u64> because StoredValue requires T: Send + Sync"
  - "Added Navigator and Clipboard to web-sys features in Cargo.toml to support clipboard copy in error toasts"
  - "Placed #[allow(unused_variables)] on msg_for_copy to suppress SSR unused-variable warning (variable only used in hydrate cfg block)"
metrics:
  duration_minutes: 8
  completed_date: "2026-03-17"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 3
---

# Phase 05 Plan 01: UI Primitives — Toast, Skeleton, EmptyState Summary

**One-liner:** Toast system with AtomicU64 ID gen and auto-dismiss, skeleton loading primitives, and enhanced EmptyState/NoTeamState components wired into App via ToastProvider context.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Add toast system, skeleton primitives, and empty state components to ui.rs | 327b2c8 |
| 2 | Wrap App content in ToastProvider in app.rs | 6f4c8a0 |

## What Was Built

### Toast System (`src/components/ui.rs`)
- `ToastKind` enum: `Success` | `Error`
- `ToastEntry` struct: `id: u64`, `kind: ToastKind`, `message: String`
- `ToastContext` struct: `show: Callback<(ToastKind, String)>` — `Copy` so pages can share across closures
- `ToastProvider` component: owns `RwSignal<Vec<ToastEntry>>`, `StoredValue<AtomicU64>` for ID generation, provides `ToastContext` via `provide_context`, enforces max-3 stacking (removes oldest Success before pushing), auto-dismisses Success toasts after 4000ms via `set_timeout` (hydrate-only)
- `ToastOverlay` (internal): fixed `top-4 left-1/2 z-50` overlay, error toasts include "Copy" (clipboard) and "×" (dismiss) buttons with `into_any()` for divergent branches

### Skeleton Primitives (`src/components/ui.rs`)
- `SkeletonLine`: `animate-pulse bg-elevated rounded {width} {height}` with `&'static str` defaults
- `SkeletonCard`: `animate-pulse bg-elevated rounded-xl border border-divider/30 {height} w-full`
- `SkeletonGrid`: maps `cols` (2/3/4) to grid-cols class, creates `cols*rows` `SkeletonCard` items via `collect_view()`

### Empty States (`src/components/ui.rs`)
- `EmptyState` enhanced: optional `icon`, `cta_label`, `cta_href` props with `#[prop(optional)]` — existing `<EmptyState message="..." />` callsites remain backward-compatible
- `NoTeamState`: zero-prop wrapper rendering EmptyState with icon="👥", canonical no-team message, CTA to /team/roster

### App Wiring (`src/app.rs`)
- `ToastProvider` wraps `<Nav />` and `<main>` inside `<Router>` — unconditionally (not in cfg block) so SSR can call `use_context::<ToastContext>()`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Cell<u64> not Send+Sync for StoredValue**
- **Found during:** Task 1 — first SSR compile
- **Issue:** `StoredValue::new(std::cell::Cell::new(0u64))` fails because `StoredValue<T>` requires `T: Send + Sync` and `Cell<u64>` is not `Sync`
- **Fix:** Used `StoredValue::new(std::sync::atomic::AtomicU64::new(0u64))` with `fetch_add(1, Ordering::Relaxed)` — equivalent semantics, fully thread-safe
- **Files modified:** src/components/ui.rs
- **Commit:** 327b2c8

**2. [Rule 3 - Blocking] Missing web-sys Navigator and Clipboard features**
- **Found during:** Task 1 — WASM compile (after SSR passed)
- **Issue:** `win.navigator().clipboard().write_text()` failed with "no method named `navigator`" because web-sys `Navigator` and `Clipboard` features were not listed in Cargo.toml
- **Fix:** Added `"Navigator", "Clipboard"` to web-sys features array in Cargo.toml
- **Files modified:** Cargo.toml
- **Commit:** 327b2c8

## Verification

- `cargo check --features ssr` — PASS
- `cargo check --features hydrate --target wasm32-unknown-unknown` — PASS
- All 9 new public items present: `ToastKind`, `ToastContext`, `ToastProvider`, `SkeletonLine`, `SkeletonCard`, `SkeletonGrid`, `EmptyState`, `NoTeamState` (+ internal `ToastOverlay`)
- `ErrorBanner` and `StatusMessage` unchanged (backward-compatible)
- `<ToastProvider>` wraps Nav+Routes inside Router in app.rs

## Self-Check: PASSED

- `src/components/ui.rs` exists: FOUND
- `src/app.rs` contains `ToastProvider`: FOUND
- Commit 327b2c8: FOUND
- Commit 6f4c8a0: FOUND
