# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — Cross-Feature Intelligence

**Shipped:** 2026-03-18
**Phases:** 6 | **Plans:** 21

### What Was Built
- Draft → game plan → post-game pipeline with FK prefill and auto-navigation
- Cross-table aggregation layer with champion name normalization
- Smart dashboard with independently-loading panels (action items, post-game, pool gaps)
- Inline intel on draft page (pool warnings, opponent tendencies) and game plan page (win condition tracker)
- Post-game auto-generates action items from tagged patterns with dedup
- Toast system, skeleton loading, and meaningful empty states across all pages

### What Worked
- **Gap closure via audit** — milestone audit at ~80% found 7/9 requirements unsatisfied, spawning targeted Phases 4-5 that systematically closed all gaps
- **Dependency-ordered phases** — strict phase ordering (pipeline → aggregation → dashboard → intel → polish) meant each phase built cleanly on the last
- **Decimal phase insertion** — Phase 1.1 (Playwright bug audit) slotted in without renumbering anything
- **Consistent patterns** — ToastContext, EmptyState, NoTeamState, SkeletonCard became reusable primitives used across all 13+ pages
- **4-day execution** — from first commit to shipped milestone in 4 days across 101 commits

### What Was Inefficient
- **SUMMARY frontmatter gaps** — no plan summaries included `requirements_completed` field, degrading audit cross-reference from 3-source to 2-source
- **Phase 2 integration tests skipped** — success criterion #5 unmet; async DB tests were declared too complex and deferred as tech debt
- **Dashboard redundant queries** — `DashboardSummary.recent_action_items` computed but bypassed in favor of separate query; wasted DB work
- **`post_game_champ_outcomes` dead code** — schema lacked win/loss field from the start, making the aggregation function return empty forever

### Patterns Established
- **ToastContext provider pattern** — AtomicU64 for ID gen, Callback::new for Copy closures, context provider at app root
- **EmptyState / NoTeamState** — two-tier empty state: "no team yet" vs "no data in this feature"
- **has_team Resource pattern** — call `get_team_dashboard` to distinguish no-team from no-data states
- **Eager signal capture in Effects** — capture all values in Effect body, not inside delayed Closure::once callbacks
- **suppress_autosave guard** — RwSignal<bool> prevents auto-save during batch signal updates

### Key Lessons
1. **Run milestone audit early** — the audit at 80% completion was the most valuable step; it systematically found gaps that manual review would have missed
2. **Schema design matters for aggregation** — missing fields (win/loss on post_game_learning) create dead-end queries that persist as tech debt
3. **Infrastructure phases enable everything** — Phase 2 (aggregation) had no direct requirements but was load-bearing for 3 subsequent phases
4. **Toast > StatusMessage** — centralized toast system is strictly better than per-page status signals; migrating all pages was worth the effort

### Cost Observations
- Model mix: primarily opus for execution, sonnet for research/planning
- Notable: gap closure phases (4-5) were the most efficient — small, targeted, high-impact

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v1.0 | 6 | 21 | First milestone; established audit-driven gap closure |

### Top Lessons (Verified Across Milestones)

1. Milestone audit before completion catches systematic gaps that per-phase verification misses
2. Infrastructure phases with no direct requirements are still critical path — never skip them
