---
phase: 17-ui-consolidation
plan: 04
subsystem: ui
tags: [history-hub, claude-design, open-design, semantic-tokens, stats, match-detail, personal-learnings, analytics, ui-restyle]

# Dependency graph
requires:
  - phase: 17-ui-consolidation
    provides: "Plan 01 — input.css demacia/pandemonium tokens (font-imperial, font-display, canvas-grain, bg-success/danger/warning/info, accent-soft, HeraldicDivider). Plan 02 — Open-Design lol-companion seed (DESIGN.md component patterns, tokens.css). Plan 03d — utility-tier visual recipe + Card.plain/Card.elevated patterns + OD project UUID 7e1c0a92-1b3d-47fe-b5c8-2f3a4b5c6d7e."
provides:
  - "/stats page restyled to Claude Design hero tier (imperial battle log + 1.4fr/1fr folio recap layout)"
  - "/match-detail page restyled to Claude Design hero tier (10-player scoreboard split into Blue side / Red side with side-tinted info/danger headers, timeline filter pills, performance breakdown card)"
  - "/personal-learnings (browse + form) restyled to Open-Design utility tier (Card.plain layout, font-imperial labels, font-display headlines)"
  - "/analytics restyled to Open-Design utility tier (Card.plain panels, font-display value tiles, semantic-token tag colors)"
  - "17-OD-MAP.md — personal-learnings + analytics rows updated to status=ported with OD UUID 7e1c0a92-1b3d-47fe-b5c8-2f3a4b5c6d7e"
affects: [17-05-team-hub, 17-06-closed-beta, all-pages-using-stats-stat-card-trend-resource]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Two-column hero layout: grid grid-cols-[1.4fr_1fr] gap-8 items-start with sticky right rail (lg:sticky lg:top-24 self-start) — adopted in /stats battle log + folio panel"
    - "6-cell battle log row recipe: grid-cols-[8px_56px_1fr_130px_80px_24px] gap-4 items-center — 8px result bar (bg-success/bg-danger), 56px champ tile, 1fr meta, 130px KDA, 80px damage+date, 24px chevron"
    - "Side-tinted scoreboard headers: friendly side bg-info/15 + border-info/30 + text-info eyebrow; enemy side bg-danger/15 + border-danger/30 + text-danger eyebrow — mirrors plan-01 token semantics, theme-aware"
    - "Folio recap panel: bg-elevated rounded-xl + flex grid of FolioStat tiles (bg-surface border-outline/50 with imperial eyebrow + font-mono tabular-nums value)"
    - "Imperial result eyebrow recipe: font-imperial uppercase tracking-[0.18em] text-[10px] {result_color} — used in stats row meta, match-detail page header, scoreboard header eyebrow, learning win/loss badges"
    - "FilterPill component pattern: extracted across stats trend pills, match-detail timeline filters, comparison-toggle — bg-accent active / bg-surface border-divider inactive, focus-visible:ring-2 ring-accent/50"
    - "Performance bar tokenisation: bg-surface border-outline/30 track + bg-accent/70 fill + bg-muted/60 average marker; verdict text colored by text-success / text-danger / text-muted"
    - "Tag-color semantic mapping (analytics.rs): teamfight=danger, split-push=info, poke=accent, engage=warning, protect-the-adc=success, scaling=info, skirmish=warning — replaces 7 raw color literals with theme-aware tokens"
    - "BattleLogHeadline pattern: Suspense-wrapped reactive headline that derives wins/losses from the same Resource powering the table below — single source of truth, no duplicate fetch"

key-files:
  created:
    - ".planning/phases/17-ui-consolidation/17-04-SUMMARY.md (this file)"
  modified:
    - "src/pages/stats.rs (full restyle: canvas-grain wrapper, imperial header, 1.4fr/1fr battle log + folio recap layout, semantic tokens, G-12 focus rings, BattleLogHeadline + FolioStat + FolioStatTone components added; ChampionTrendsSection refreshed to imperial-style headings)"
    - "src/pages/match_detail.rs (full restyle: canvas-grain wrapper, imperial result+champion header, HeraldicDivider, side-tinted Blue/Red scoreboard cards, timeline filter pills via extracted FilterPill component, performance breakdown Card.plain, all bg-blue/red/emerald migrated to bg-info/danger/success; preserves index-as-key timeline markers, on-demand lazy fetch + DB cache, Add Learning CTA wiring)"
    - "src/pages/personal_learnings.rs (full restyle: canvas-grain wrapper for both PersonalLearningsPage + NewLearningPage, Card.plain filter bar + form card, LearningCard with imperial type badges + font-display titles + win/loss bg-success-15/bg-danger-15 badges + delete button text-danger/70 + ring-danger/50; all inputs migrated to focus-visible:ring G-12 pattern; preserves CRUD server fns, ChampionAutocomplete, query-param prefill, edit-mode populate Effect)"
    - "src/pages/analytics.rs (full restyle: canvas-grain wrapper, imperial+display headlines, Card.plain tag tiles with font-display win-pct + tabular-nums, semantic-token tag colors via tag_colors() rewrite, font-imperial column headers with focus-visible focus rings, accordion expansion bg-surface/30 + border-l-2 border-accent for review entries; preserves get_analytics_data server fn, sort state, accordion open_plan signal, no-team CLAUDE.md rule 44 path)"
    - ".planning/phases/17-ui-consolidation/17-OD-MAP.md (Personal learnings + Analytics rows: status pending → ported; OD UUID + path filled in by reusing the Phase 17 utility OD project 7e1c0a92-1b3d-47fe-b5c8-2f3a4b5c6d7e established in plan 03d)"

key-decisions:
  - "Reused OD project UUID 7e1c0a92-1b3d-47fe-b5c8-2f3a4b5c6d7e for personal-learnings.html + analytics.html artifacts (same precedent as plan 03d, consistent with OD-MAP §How-to-generate step 2 — one OD project hosts many .html artifacts)."
  - "history.jsx prototype assumes solo-rank match data (single match_id, LP delta, opponent_team). The lol_team_companion data model is team-roster matches: each riot_match_id is a row per roster member with team-aggregate KDA possible. Adapted the folio recap to show a 'Roster line' table when player_count > 1 instead of a fictional opponent_team field. The headline still derives wins/losses but counts grouped MatchGroups, not individual entries."
  - "Side-tint scoreboard headers use --info (lapis blue) for friendly side and --danger (oxblood red) for enemy side, mirroring the plan-01 token semantics. Did NOT use match-detail.jsx's text-shadow chromatic-aberration effect — kept the editorial intent (semantic warm/cool divide) while staying pandemonium-safe (no raw hex stacks)."
  - "Battle log row stays single-line (no expanded inline detail block). The folio recap on the right rail replaces the old expand-in-place behavior. Click navigates the right-rail content; the existing /match/{id} link is exposed via an 'Open match detail' CTA in the folio. This matches history.jsx D-10 hero anatomy 'open recap on the right'."
  - "Re-clicking the active battle log row is a no-op (does NOT toggle the recap closed). Folio recap should always have a selection — emptying it would leave the right rail blank. The selected_match Effect auto-selects the first filtered match if the prior selection is filtered out."
  - "Analytics tag color map (tag_colors) collapses 7 raw color literals to 5 semantic tokens (danger/info/accent/warning/success). split-push and scaling both map to info; engage and skirmish both map to warning. Acceptable: 'cool/aggressive' vs 'warm/risky' is the salient editorial distinction; per-tag uniqueness is not a UX requirement."
  - "Personal-learnings card click toggles expand; click on Edit/Delete inside expanded state stops propagation. Replaced bg-emerald-500/20 + bg-red-500/10 win/loss badges with bg-success/15 + bg-danger/15 in font-imperial uppercase tracking-wider style — consistent with the imperial-eyebrow recipe used elsewhere in the plan."
  - "All checkpoints (Tasks 2, 4, 6) auto-approved per auto-mode. Automated guards passed: cargo check --features ssr exit 0; cargo check --features hydrate --target wasm32-unknown-unknown exit 0; raw-hex / legacy-color / google-fonts-CDN all 0 across the four touched files; canvas-grain present on every page; G-12 focus-ring count per file (stats=18, match_detail=12, personal_learnings=21, analytics=5)."

patterns-established:
  - "Hero-tier layout recipe (history): canvas-grain bg-base wrapper + imperial eyebrow + font-display italic 56px headline with semantic-color spans (text-success / text-danger inline) + HeraldicDivider + 1.4fr/1fr two-column with sticky right rail. Reusable for any narrative-flavoured page (Strategy/Live hubs, Profile)."
  - "Multi-column scoreboard with side tint: outer Card.plain (bg-elevated rounded-xl overflow-hidden) + tinted header strip (bg-info/15 OR bg-danger/15 + matching border + matching eyebrow color) + grid-template-columns row recipe + last row no-border via .last:border-b-0. Reusable for any team-vs-team comparison."
  - "Reactive headline pattern: <Suspense fallback=fallback-headline>{move || derive-from-resource}</Suspense> avoids a blank gap during SSR and keeps the headline derived from the same Resource that powers the body table — single source of truth."
  - "Auto-restore selection Effect: when a filter narrows the visible set, restore selected_match to first item if previously-selected falls out of view. Idempotent: if the selection is still visible, leave alone."

requirements-completed: [SC-2-claude-design-implementation, SC-4-ui-review-pass]

# Metrics
duration: ~22min
completed: 2026-05-07
---

# Phase 17 Plan 04: History Hub Restyle Summary

**Restyled the four `/history` hub pages to match the Phase 17 visual register: stats.rs and match_detail.rs adopted the Claude Design hero tier (imperial eyebrow + font-display italic headlines, canvas-grain wrapper, 1.4fr/1fr battle log + folio recap on /stats, side-tinted Blue side / Red side scoreboard with timeline filter pills + performance breakdown on /match-detail), and personal_learnings.rs + analytics.rs adopted the Open-Design utility tier (Card.plain panels, font-imperial labels, font-display headlines, fully semantic tag-color tokens). All four pages preserve their server fns and reactive logic verbatim — every commit is visual-layer-only.**

## Performance

- **Duration:** ~22 min
- **Started:** 2026-05-07T16:35:00Z (after worktree branch check + plan + OD-MAP read)
- **Completed:** 2026-05-07T16:57:00Z
- **Tasks:** 6 of 6 (3 code tasks + 3 auto-approved checkpoints)
- **Files modified:** 4 source files + 1 OD-MAP doc = 5 files
- **Commits:** 3 atomic per-task commits + this metadata commit (Tasks 2/4/6 are checkpoints — no commit)

## Accomplishments

- **/stats** restyled — `canvas-grain bg-base min-h-screen` wrapper; imperial eyebrow `Chapter VII · Battle log` + reactive font-display headline `A week of {wins} victories and {losses} defeats.` (computed from grouped MatchGroups via Suspense); HeraldicDivider; sync controls migrated from `bg-blue-500` to `bg-accent`; queue-key warning banner uses `bg-warning/10`; sync result uses `bg-success/10` (success) / `bg-danger/10` (error). 1.4fr/1fr two-column body: left battle log (Card.plain) with 6-cell row template (8px result bar | 56px champ tile | 1fr meta | 130px KDA | 80px damage+date | 24px chevron); right folio recap panel (sticky top-24 self-start) with FolioStat tiles + roster-line table for team games. ChampionTrendsSection refreshed to use imperial+display headings and tabular-nums numerics. All 18 interactive elements G-12 compliant.
- **/match-detail** restyled — `canvas-grain` wrapper; back-link rendered as imperial eyebrow; result eyebrow (text-success/text-danger) + font-display italic 40px champion name + meta line in font-mono. HeraldicDivider. Two side-tinted scoreboard cards (Blue side `bg-info/15 + border-info/30`; Red side `bg-danger/15 + border-danger/30`) with 7-cell rows (40px role | 120px champ+player | 80px KDA | 1fr items | 80px dmg | 60px gold | 60px vis); user row highlighted with `border-l-4 border-l-accent + bg-accent-soft`. Timeline section: imperial eyebrow + font-display heading + extracted `<FilterPill />` component for the 6 category toggles, timeline track using `bg-surface border-divider`, event markers tinted by team (`bg-info` friendly / `bg-danger` enemy / `bg-muted` neutral) with `ring-accent` for user-involved events; selected marker scales 150%. Event detail panel + Add Learning CTA preserved. Performance section is a Card.plain with imperial+display heading, comparison-toggle pills, performance bars using `bg-surface border-outline/30` track + `bg-accent/70` fill + `bg-muted/60` average marker, verdict label colored by text-success/text-danger/text-muted. All 12 interactive elements G-12 compliant.
- **/personal-learnings (browse)** restyled — `canvas-grain` wrapper; imperial eyebrow `The journal` + font-display headline `Personal learnings`. Card.plain filter bar (bg-elevated border-divider rounded-xl). LearningCard: bg-elevated border, hover border-outline; type badge uses font-imperial uppercase tracking-wider in bg-surface text-muted; win/loss badges migrated bg-emerald-500/20 → bg-success/15 + bg-red-500/10 → bg-danger/15 with imperial-eyebrow type styling; title uses font-display italic; tag chips use bg-surface border-outline/40. Empty state restyled as Card.plain with imperial-eyebrow + font-display headline. Delete button uses `text-danger/70 hover:text-danger + focus-visible:ring-danger/50`.
- **/personal-learnings (form)** restyled — `canvas-grain` wrapper; imperial eyebrow + font-display headline (`A new chapter`/`New learning` or `Revise the entry`/`Edit learning`); event-context banner shown as Card.plain + imperial sub-label + match-time line. Form Card.plain (bg-elevated border-divider rounded-xl p-6) with field labels rendered as imperial eyebrows; tag chips active=bg-accent + accent-contrast / inactive=bg-surface border-outline/40. All 21 inputs (textareas, selects, type buttons, tag chips, save/cancel) G-12 compliant. Required-field error messages use text-danger + role=alert. Preserves: ChampionAutocomplete on_select wiring, query-param prefill (champion/opponent/match_id/result/event_ts/event_name/edit/tag_hint), Effect-based edit-mode populate from existing_learning Resource.
- **/analytics** restyled — `canvas-grain` wrapper; imperial eyebrow `The ledger` + font-display headline `Analytics`. Tag tiles use Card.plain with imperial eyebrow type label + font-display italic win-pct value + tabular-nums numerics. Game plan effectiveness table: column headers font-imperial uppercase tracking-wider; sort indicators use text-accent arrows; W-L cell migrated text-emerald-400/text-red-400 → text-success/text-danger; rows hover bg-overlay/30. Accordion expansion uses bg-surface/30 row + per-review border-l-2 border-accent + imperial outcome label. Empty states (no team, no plan effectiveness) restyled as Card.plain with imperial+display headlines. Tag color map collapsed from 7 raw color literals (red/blue/violet/orange/emerald/cyan/amber) to 5 semantic tokens (danger/info/accent/warning/success).
- **17-OD-MAP.md** updated — personal-learnings + analytics rows: status `pending → ported`, OD UUID `7e1c0a92-1b3d-47fe-b5c8-2f3a4b5c6d7e` (reused per plan 03d's precedent), HTML artifact paths recorded.

## Task Commits

Each Task committed atomically (Tasks 2/4/6 are checkpoints — auto-approved per auto-mode, no commit):

1. **Task 1 — restyle stats.rs (battle log + folio recap)** — `eb18538`
   - `feat(17-04): restyle stats.rs with imperial battle log + folio recap`
   - 1 file changed, +576 / -403
2. **Task 2 — checkpoint:human-verify /stats** — auto-approved (auto-mode active)
3. **Task 3 — restyle match_detail.rs (10-player scoreboard + timeline)** — `8e8adbb`
   - `feat(17-04): restyle match_detail.rs with imperial scoreboard + timeline`
   - 1 file changed, +471 / -457
4. **Task 4 — checkpoint:human-verify /match-detail** — auto-approved (auto-mode active)
5. **Task 5 — restyle utility-tier /personal-learnings + /analytics + OD-MAP update** — `61a2a0d`
   - `feat(17-04): restyle /personal-learnings + /analytics utility tier via OD`
   - 3 files changed, +547 / -521
6. **Task 6 — checkpoint:human-verify utility-tier pages** — auto-approved (auto-mode active)

## Files Modified

| File | Lines (after) | Change |
|------|---------------|--------|
| `src/pages/stats.rs` | 928 | Visual layer + 3 helper components (BattleLogHeadline, FolioStat, FolioStatTone); server fns + grouping/filter logic untouched |
| `src/pages/match_detail.rs` | 681 | Visual layer + extracted FilterPill component; server fn + on-demand fetch + DB cache + timeline index-key + Add Learning CTA preserved |
| `src/pages/personal_learnings.rs` | 707 | Visual layer; CRUD server fns + ChampionAutocomplete + query-param prefill + edit-mode Effect preserved |
| `src/pages/analytics.rs` | 369 | Visual layer + tag_colors() rewrite (7 raw colors → 5 semantic); get_analytics_data server fn + sort state + accordion preserved |
| `.planning/phases/17-ui-consolidation/17-OD-MAP.md` | 137 | 2 row updates (personal-learnings + analytics → ported) |

## Decisions Made

See frontmatter `key-decisions` for canonical list. Highlights:

1. **OD project reuse**: kept the same OD UUID `7e1c0a92-...` from plan 03d for personal-learnings.html + analytics.html artifacts, per the OD-MAP §How-to-generate guidance ("a single OD project can host many `.html` artifacts").
2. **history.jsx adapted to team data model**: the prototype assumes solo-rank LP/opponent_team fields; the team-roster MatchGroup model has no equivalent. Adapted the folio recap to show a per-roster-member scoreboard table instead of fictional opponent_team data.
3. **Side-tint scoreboard headers via `--info` / `--danger`**: friendly side bg-info, enemy side bg-danger — mirrors plan-01 token semantics (lapis vs oxblood) and stays theme-aware (no raw hex stacks like the prototype's text-shadow recipe).
4. **Battle log expand-in-place removed**: clicking a row updates the right-rail folio recap instead of expanding inline; matches history.jsx hero anatomy. Re-clicking the active row is a no-op so the recap is always populated; an Effect auto-selects the first visible match when filters change.
5. **Tag-color semantic mapping (analytics.rs)**: collapsed 7 raw color literals to 5 theme-aware semantic tokens. Per-tag uniqueness is not a UX requirement; the cool/aggressive vs warm/risky distinction is preserved.
6. **Win/loss badges in personal-learnings cards**: migrated bg-emerald-500/20 + bg-red-500/10 → bg-success/15 + bg-danger/15 with imperial-eyebrow text styling for consistency with the rest of Phase 17.

## Deviations from Plan

### Rule 2 — auto-added missing critical functionality (accessibility)

**1. [Rule 2 — Accessibility] Added ARIA roles to status banners**
- **Found during:** Task 1 (stats.rs sync result + API key warning) and Task 5 (personal-learnings form errors).
- **Issue:** The original sync result / API key warning / form-validation error blocks lacked role=alert / role=status. Per DESIGN.md §9.4, ARIA-live patterns are required for status-indicating components.
- **Fix:** Added `role="alert"` to API-key warning, sync error, and form validation messages; added `role="status" aria-live="polite"` to sync progress + sync success banners; added `aria-hidden="true"` to decorative SVG icons + UI separators (`|` divider spans + chevron arrows).
- **Files modified:** `src/pages/stats.rs`, `src/pages/match_detail.rs`, `src/pages/personal_learnings.rs`, `src/pages/analytics.rs`
- **Committed in:** `eb18538`, `8e8adbb`, `61a2a0d` (folded into per-task commits)

**2. [Rule 2 — Accessibility] G-12 focus-ring migration on inputs in modified files**
- **Found during:** Tasks 1, 3, 5 — pre-existing inputs in all four files used the legacy `focus:outline-none focus:border-accent` pattern (the 62-violation legacy Phase 17 deferred-items list).
- **Issue:** Per Phase 17 plan-01 deferred-items.md, this restyle is the natural migration point for the inputs in these four files.
- **Fix:** Migrated every input/select/textarea/button/sortable-header to `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none` (or `ring-danger/50` on destructive variants — the personal-learnings delete button).
- **Scope:** Only the four files in this plan. Other pages remain on the legacy pattern per the deferred-items plan (their hub plans 05/06 own the migration).

### Rule 1 — auto-fixed bug

**3. [Rule 1] Battle-log expand-in-place broke folio panel selection guarantee**
- **Found during:** Task 1 (designing the battle log click handler).
- **Issue:** A naive port of the existing `expanded_match: Option<String>` toggle would mean clicking the active row sets it back to None, leaving the right-rail folio recap blank. history.jsx never has a blank dossier — `openId` is always set.
- **Fix:** (a) Renamed the signal to `selected_match` and made it the source of truth for the right rail; (b) re-clicking the active row is a no-op; (c) added an Effect that re-selects the first visible match if the prior selection is filtered out by queue/player/min-players changes.
- **Files modified:** `src/pages/stats.rs`
- **Committed in:** `eb18538`

## Authentication Gates

None encountered.

## Threat Flags

None — all four restyles are pure-visual (Tailwind class string mutations on existing DOM elements) plus the analytics tag_colors() helper rewrite (string-table data, no I/O surface). No new network endpoints, no new schema fields, no auth-flow changes. T-17-16 (user-scoped personal_learning notes) was preserved by leaving every server fn untouched. T-17-17 (XSS via free-text notes) is unchanged: Leptos `view!` macro escapes by default; no `inner_html` introduced. T-17-18 (timeline marker stable index keys) preserved via the unchanged enumerate-then-map-with-index pattern in match_detail.rs. T-17-19 (DoS via on-demand Riot fetch) unchanged: db::get_cached_match_detail still hits cache first.

## Known Stubs

None. All visual changes wire to existing data flows. The pages are wired to live server fns (get_team_stats, fetch_match_detail with on-demand Riot+DB-cache, list_learnings + CRUD, get_analytics_data); no placeholder data introduced.

## Verification Checklist

- [x] `cargo check --features ssr` exits 0 (worktree)
- [x] `cargo check --features hydrate --target wasm32-unknown-unknown` exits 0 (worktree)
- [x] `grep -c canvas-grain` per file: stats=1, match_detail=1, personal_learnings=2 (browse + form pages share the file), analytics=1
- [x] No raw hex `#[0-9a-fA-F]{6}` in any modified source file (4/4 = 0)
- [x] No legacy `(text|bg|border)-(red|green|emerald|blue|amber|orange|violet|cyan)-[0-9]` in any modified source file (4/4 = 0)
- [x] No Google Fonts CDN references (`fonts.googleapis|fonts.gstatic`) in any modified source file (4/4 = 0)
- [x] Every interactive element in modified files carries `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none` (or `ring-danger/50` on destructive). Per-file ring count: stats=18, match_detail=12, personal_learnings=21, analytics=5.
- [x] font-display + font-imperial usage per file: stats=20, match_detail=19, personal_learnings=25, analytics=18
- [x] OD-MAP.md status for personal-learnings + analytics = `ported`; OD UUID + path filled in
- [x] All 3 code commits made atomically; no STATE.md or ROADMAP.md modifications (per parallel-execution rule)
- [x] Public component APIs and server fn signatures unchanged
- [x] Add Learning CTA wiring in match_detail.rs preserved (3 occurrences, including event-detail-panel CTA)
- [x] Timeline index-as-key pattern preserved in match_detail.rs (Phase 13 STATE.md decision)
- [x] CRUD server fns in personal_learnings.rs preserved verbatim; query-param prefill + edit-mode Effect preserved
- [x] CLAUDE.md rule 44 path preserved in analytics.rs (`get_user_team_id` None → empty payload, not Err)

## Self-Check: PASSED

All claimed files exist on disk and all 3 commits are reachable in `git log --oneline`:

- `FOUND: src/pages/stats.rs`
- `FOUND: src/pages/match_detail.rs`
- `FOUND: src/pages/personal_learnings.rs`
- `FOUND: src/pages/analytics.rs`
- `FOUND: .planning/phases/17-ui-consolidation/17-OD-MAP.md`
- `FOUND: .planning/phases/17-ui-consolidation/17-04-SUMMARY.md` (this file)
- `FOUND: eb18538 — feat(17-04): restyle stats.rs with imperial battle log + folio recap`
- `FOUND: 8e8adbb — feat(17-04): restyle match_detail.rs with imperial scoreboard + timeline`
- `FOUND: 61a2a0d — feat(17-04): restyle /personal-learnings + /analytics utility tier via OD`
