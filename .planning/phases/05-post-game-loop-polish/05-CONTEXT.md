# Phase 5: Post-Game Loop + Polish - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Auto-generate action items from post-game reviews. Add consistent empty states, loading skeletons, and mutation feedback (toast system) across all pages in the app. No new features or pages — polish and automation of existing surfaces.

</domain>

<decisions>
## Implementation Decisions

### Action Item Auto-Generation (PIPE-02)
- Trigger: Fully automatic on post-game review save — no confirmation step
- Source data: The `improvements` array from the review only (not strengths or patterns)
- Each improvement string becomes one ActionItem record with `source_review` set to the review ID
- Assignment: Created unassigned (team-wide) — coach or team assigns later on the action items page
- Feedback: Inline StatusMessage banner on post-game page showing "N action items created — View" with a clickable link to `/action-items`. Stays visible until dismissed or next save
- Deduplication: Claude's discretion on whether to skip creating items that match existing open action items

### Empty State Design (UX-01)
- Tone: Coaching/guiding — tell users what to do next. E.g. "No drafts yet — create your first draft to start planning picks"
- Visuals: Include a relevant icon above the message (Unicode or existing icon patterns). No illustrations or custom assets
- CTA: Primary accent-colored button with action label. E.g. "Create Your First Draft"
- No-team state: All team-scoped pages show consistent message — "You need a team to use this feature" with button linking to roster page
- Pages needing empty states: champion_pool, stats, opponents, profile (no Riot account linked), team_builder, team/roster, team/dashboard panels

### Loading Skeletons (UX-02)
- Type: Shape-matching skeletons that approximate actual content layout (card outlines, text line widths, stat boxes)
- Architecture: Reusable primitive components (SkeletonLine, SkeletonCard, SkeletonGrid) that pages compose in their Suspense fallbacks
- Animation: Tailwind `animate-pulse` (opacity fade) — already used in a few places, no custom CSS needed
- Pages needing skeletons: All data-fetching pages that currently show "Loading..." text

### Mutation Feedback — Toast System (UX-03)
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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements fully captured in decisions above and in:

### Requirements
- `.planning/REQUIREMENTS.md` — PIPE-02, UX-01, UX-02, UX-03 requirement definitions and acceptance criteria

### Prior Phase Context
- `.planning/phases/01-pipeline-ctas/01-CONTEXT.md` — Pipeline CTA patterns, back-reference badge design, flow-based labels
- `.planning/phases/04-inline-intel/04-CONTEXT.md` — Established patterns for Resource/Suspense loading, semantic token usage

### Existing Components
- `src/components/ui.rs` — Current ErrorBanner, StatusMessage, EmptyState components (to be replaced/enhanced)
- `src/pages/post_game.rs` — Post-game save flow, pattern analysis, improvements array
- `src/pages/action_items.rs` — ActionItem CRUD, model structure, source_review field
- `src/models/action_item.rs` — ActionItem struct definition

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `EmptyState` component in `ui.rs`: Generic centered text — needs enhancement with icon + CTA button
- `StatusMessage` / `ErrorBanner` in `ui.rs`: Current feedback components — will be replaced by toast system
- `animate-pulse` already used in post_game.rs, champion_pool.rs, draft.rs for inline loading — extend to skeleton primitives
- `ActionItem` model with `source_review: Option<String>` field — ready for linking to post-game reviews
- `create_action_item()` in db.rs — existing DB function for action item creation

### Established Patterns
- `Resource::new()` with `<Suspense fallback=...>` for async data loading — skeleton replaces fallback content
- `spawn_local` for async event handlers with match Ok/Err — toast calls replace local signal updates
- Semantic tokens: `bg-surface`, `bg-elevated`, `border-divider`, `text-muted`, `bg-accent`, `text-accent-contrast`
- `Callback::new()` for Copy-safe closures shared across reactive contexts

### Integration Points
- `src/pages/post_game.rs`: Wire improvements → ActionItem creation in save_post_game_learning server fn
- `src/server/db.rs`: Add batch action item creation function
- `src/components/ui.rs`: Add Toast, ToastProvider, skeleton primitive components
- Every page's `<Suspense fallback=...>`: Replace "Loading..." with composed skeleton
- Every page's mutation handlers: Replace local message signals with `show_toast()` calls
- `src/app.rs`: Wrap app in ToastProvider context

</code_context>

<specifics>
## Specific Ideas

- Error toasts with copy-to-clipboard are important for debugging — users can paste error messages when reporting issues
- The coaching tone for empty states fits the app's identity as a "coaching assistant"
- Toast migration means removing per-page `save_result` / `error_msg` signals — cleaner component code

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-post-game-loop-polish*
*Context gathered: 2026-03-17*
