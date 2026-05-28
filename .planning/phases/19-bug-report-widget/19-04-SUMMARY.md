---
phase: 19
plan: 04
status: complete
type: execute
wave: 3
success_criteria_satisfied: [SC-3, SC-7]
threats_addressed: []
files_modified:
  - src/pages/draft.rs
  - src/pages/solo_dashboard.rs
  - src/pages/team/dashboard.rs
  - src/pages/stats.rs
  - src/pages/champion_pool.rs
  - src/pages/game_plan.rs
  - src/pages/post_game.rs
  - src/components/bug_report_widget.rs
  - e2e/tests/bug-report.spec.ts
key_files:
  created:
    - e2e/tests/bug-report.spec.ts
  modified:
    - src/pages/draft.rs
    - src/pages/solo_dashboard.rs
    - src/pages/team/dashboard.rs
    - src/pages/stats.rs
    - src/pages/champion_pool.rs
    - src/pages/game_plan.rs
    - src/pages/post_game.rs
    - src/components/bug_report_widget.rs
---

# Plan 19-04 — Page rollout + e2e regression

## What landed

**Task 1 — `data-feedback-label` rollout across 7 authed pages** (commit `24328ae`)

Page-level + section-level identifiers using the strict `<Page> → <Section> → <Element>` schema (U+2192 RIGHTWARDS ARROW, **not** ASCII `->`). Raw HTML elements get plain `data-feedback-label="..."`; Leptos components get the `attr:`-prefixed form per leptos rule 10. No new wrapper divs were added solely to host the attribute (D-06.3 honored).

| Page | Labels added (U+2192 in section + element) |
|---|---|
| `src/pages/draft.rs` | Draft / Draft → War Table header / Draft → Action ledger |
| `src/pages/solo_dashboard.rs` | Solo dashboard / Solo dashboard → Ranked snapshot / Solo dashboard → LP history graph |
| `src/pages/team/dashboard.rs` | Team dashboard / Team dashboard → Roster slots / Team dashboard → Join requests / Team dashboard → Game-day brief |
| `src/pages/stats.rs` | Stats / Stats → Filters / Stats → Battle log |
| `src/pages/champion_pool.rs` | Champion pool / Champion pool → Tier list / Champion pool → Champion grid |
| `src/pages/game_plan.rs` | Game plan / Game plan → Champion picker / Game plan → Role analysis |
| `src/pages/post_game.rs` | Post game / Post game → Learning list / Post game → Match summary |

**Task 2 — Playwright spec `e2e/tests/bug-report.spec.ts`** (commit `faead1c`)

6 tests using the `authedPage` fixture (per leptos rule 8 + wasm rule 56):
1. Floating Report button visible on `/draft` (authed)
2. Floating Report button hidden on `/auth/login` (bare `page`, no auth)
3. Select mode captures the nearest `data-feedback-label` via `closest()`
4. Esc cancels select mode without opening modal
5. Submit persists to DB, toast "Your report is in" appears, modal closes
6. **Negative-content gate (SC-7)** — modal `innerHTML` must not contain 🐛, 🎉, or "Thanks!" exclamation form. Belt-and-suspenders over 19-02's source-code grep.

**Task 3 — `Closure::forget` lifecycle fix on the merged 19-02 widget** (commit `b5a760b`)

The new e2e spec caught a regression latent in 19-02: `exit_select_mode` was synchronously dropping the click-capture and keydown `Closure`s while the click closure was still on the JS stack. The browser's bubble-phase / synthetic-click follow-ups re-invoked the JS shim of an already-dropped Closure → wasm_bindgen panic `closure invoked recursively or after being dropped`. Slow human clicks during the 19-02 manual checkpoint didn't trigger the race; the Playwright `.click()` driving the flow rapidly reproduced it deterministically.

Fix: `cb.forget()` after `removeEventListener`. JS shim is leaked for the page lifetime (≈60 bytes per select-mode session), but the Rust side never destabilises an in-flight invocation. Standard wasm_bindgen pattern for transient global event listeners.

## Acceptance gates (all green)

- `cargo check --features ssr` — clean (2 pre-existing unrelated dead-code warnings)
- `cargo check --features hydrate --target wasm32-unknown-unknown` — clean
- `cargo test --features ssr --lib` — **132/132 pass** (no regression)
- `npx playwright test tests/bug-report.spec.ts` — **6/6 pass**
- `npx playwright test tests/hydration-no-panic.spec.ts` — **19/19 pass** (Phase 18.2 regression suite holds)
- U+2192 codepoint present per page (positive grep per plan acceptance)
- No new wrapper divs introduced (D-06.3 grep on diff)

## Self-Check: PASSED

## Deviations from plan

**D-1 — Orchestrator-side rescue after API-529 mid-execution.** The executor agent terminated with provider-side `API Error: 529 Overloaded` at ~36 min, 94 tool uses in, with 0 commits and no SUMMARY. All file edits were sitting uncommitted in the agent's worktree. Following the execute-phase workflow's overload-recovery rule (do not retry from scratch; rescue committed/uncommitted state first), the orchestrator generated a patch from `git -C <worktree> diff`, applied it to `main` via `git apply --index`, copied the new e2e spec across, and committed atomically per the plan's task split. Saved ~36 min of redundant agent work and the associated tokens. Re-validation confirmed all acceptance gates green on the rescued state.

**D-2 — e2e exposed a 19-02 latent bug; fixed in this plan rather than spawning a 19.1 gap phase.** The closure use-after-drop is a direct consequence of how 19-02 implemented `exit_select_mode`. Fixing it here keeps the bug-report flow shippable as a single phase and avoids a phase-decimal split for a 2-line fix. Documented above and in the commit message so the trail is auditable.

## Reusable lessons (candidates for wiki-debrief at phase close)

1. **`Closure::forget` after `removeEventListener` for transient global event listeners.** Sync-dropping a Closure while its JS shim may still be on the browser call stack panics with "closure invoked recursively or after being dropped." Trade 60 B per session for safety.
2. **Playwright e2e finds WASM closure-lifetime bugs that slow human clicks hide.** The 19-02 manual checkpoint passed; the 19-04 spec failed deterministically on the same code path. Carry: lock down lifetime-critical WASM code with at least one machine-driven test that hits the timing race.
3. **Agent API-529 rescue protocol.** When a worktree executor terminates with an overload error mid-run: do NOT retry. Inspect `git -C <wt> status`, rescue uncommitted edits via `git apply --index` of the worktree diff onto main, commit atomically per the plan task split. Saves both wall-clock and tokens.
4. **cargo-leptos watch is not crash-resilient.** Killing the server child (PID) does not auto-restart it; cargo-leptos can also miss file changes immediately after a manual `cargo check` run. If the binary mtime stops advancing, restart cargo-leptos rather than re-touching.

## Awaiting

Manual G-10 dark-pattern review of the real labels (deferred from 19-02 per user choice). User to walk a sample of pages (`/draft`, `/solo`, `/stats`, `/champion-pool`) confirming the labels are descriptive, non-manipulative, and the toast string remains `"Thanks. Your report is in."` (period, not exclamation).
