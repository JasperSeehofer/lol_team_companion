---
phase: 17-ui-consolidation
plan: 05
subsystem: ui
tags: [profile-hub, captains-folio, strategy-room, solo-constellation, closed-beta-redirect, utility-tier, open-design, leptos, semantic-tokens]

# Dependency graph
requires:
  - phase: 17-ui-consolidation
    provides: "Plan 01 — input.css demacia/pandemonium tokens (canvas-grain, font-imperial, font-display, bg-success/danger/warning/info, accent-soft, GiltCorner, HeraldicDivider). Plan 02 — Open-Design lol-companion seed (DESIGN.md component recipes, tokens.css). Plan 03d — utility-tier visual recipe (Card.plain pattern + OD project UUID 7e1c0a92-1b3d-47fe-b5c8-2f3a4b5c6d7e). Plan 04 — hero-tier patterns (imperial+display heroes, FilterPill, semantic tag-color mapping)."
provides:
  - "/profile restyled as Captain's Folio with 5 GiltCard panels (Account, Riot, Champion Pool, Theme, Sign out) wrapped in 4-corner GiltCorner ornaments"
  - "/team/dashboard restyled as Strategy Room (8 imperial section eyebrows + font-display titles; semantic-token migration of all roster/coach/bench/recent/action-items/post-game/pool-gap/notebook surfaces)"
  - "/solo-dashboard restyled as SoloConstellation (gilt ranked badge with drop-shadow accent; LP graph SVG migrated to style=\"stroke: var(--color-accent)\" per Pitfall 9; 3 goal cards with imperial labels + font-display tabular-nums values)"
  - "/ (home) auth-aware: unauth → /closed-beta redirect; authed-no-team → /team/roster redirect; authed team member → restyled Strategy Room dashboard with GiltCard recent activity panel"
  - "/team/roster restyled to utility tier (3 Card.plain sections: Found / Enlist / Linked seal)"
  - "/team-builder restyled to utility tier (role slots, Composition Identity, Opponent Intel collapsible, Save as Draft) with semantic tier_badge_class + comp_tag_class rewrites"
  - "17-OD-MAP.md — team-roster + team-builder rows updated to status=ported with OD UUID + path"
affects: [17-06-closed-beta]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "GiltCard inline component recipe: bg-elevated border border-outline rounded-xl p-6 relative + 4 absolutely-positioned <GiltCorner /> ornaments at tl/tr/bl/br corners. Adopted in profile.rs (5 panels) and home.rs (recent activity panel). Reusable for any Demacia-tier folio surface."
    - "Auth-aware HomePage redirect: single Effect that watches the get_dashboard Resource result and dispatches based on (logged_in, has_team) — unauth → /closed-beta, authed-no-team → /team/roster, authed-with-team → in-page Strategy Room. Uses if let Some(window) chain per wasm-patterns rule 35."
    - "LP graph SVG style migration: stroke=\"var(--t-accent)\" attribute → style=\"stroke: var(--color-accent)\" inline style. Both forms work, but inline-style aligns with the canonical ornaments.rs pattern and Pitfall 9 — theme switch re-evaluates the CSS variable through the style attribute."
    - "Semantic token migration recipe for goal-card progress bars: bg-elevated rounded-full → bg-surface border border-outline/30 rounded-full overflow-hidden track + bg-success/70 (achieved) or bg-accent/60 (in-progress) fill. Replaces raw bg-emerald-500/60 + bg-accent/50."
    - "Goal achievement badge: bg-success/20 text-success border-success/30 + font-imperial uppercase tracking-[0.18em] text-[10px] rounded-full px-2 py-0.5. Replaces raw bg-emerald-500/20 text-emerald-400 + 'text-xs font-semibold'."
    - "Strategy Room imperial-eyebrow section header recipe: <div class=\"font-imperial uppercase tracking-[0.18em] text-[10px] text-muted\">{eyebrow}</div> + <h3 class=\"font-display italic text-2xl text-primary\">{title}</h3>. Used 8 times in team/dashboard.rs (Order of battle / Council / Reserves / Battle log / Tasking / Debrief / Reconnaissance / Folio) and across the 6 plan-touched files."
    - "Auto-mode checkpoint discipline: Tasks 2/4/6/8/10 auto-approved per auto-mode policy. All automated guards (cargo check both targets, no raw hex, no legacy color, focus-visible counts) verified before each auto-approve."

key-files:
  created:
    - ".planning/phases/17-ui-consolidation/17-05-SUMMARY.md (this file)"
  modified:
    - "src/pages/profile.rs (Captain's Folio: full visual rewrite — canvas-grain wrapper, imperial eyebrow + 44px font-display headline, HeraldicDivider, 5 GiltCard panels, GiltCard inline component, all inputs G-12, Logout uses bg-danger/10 destructive recipe; PRESERVED 5 server fns + 4 reactive Effects + auth-redirect)"
    - "src/pages/team/dashboard.rs (Strategy Room: canvas-grain wrapper, imperial+display page header + HeraldicDivider, 8 section eyebrows; recent games rows bg-info/10 / bg-danger/10 with imperial W/L pills; action items semantic dots; pool gap warning border; Join request Accept/Decline → success/danger Button.destructive recipe; bench/coach/starter cards on outline/50 rounded-xl; team notebook: imperial header + outline cards + G-12 focus rings on textarea/save/cancel/edit/pin/delete; aria-hidden on decorative SVGs; role=alert on req errors; PRESERVED 19 server fns + WR-01 join-request error pattern)"
    - "src/pages/solo_dashboard.rs (SoloConstellation: canvas-grain wrapper, imperial+display headline + HeraldicDivider; ranked badge with gilt drop-shadow + font-display 32px display name + font-mono LP value; match list with bg-info/danger semantic borders + bg-elevated outline rows; LP graph SVG migrated to style=\"stroke: var(--color-accent)\" per Pitfall 9; LP window pills with G-12; 3 goal cards (Rank/CS/Deaths) with imperial header label + font-display tabular-nums values + bg-surface border-outline/30 progress tracks; achievement badges semantic; PRESERVED 5 server fns + auto-sync Effect + Phase 16 sync refetch fix [dashboard_resource + lp_history_resource + goal_progress_resource])"
    - "src/pages/home.rs (auth-aware landing: unauth → /closed-beta redirect, authed-no-team → /team/roster redirect, authed-with-team → restyled Strategy Room; imperial+display headline; pending-request alert bg-warning/10; Riot key notice; StatBox + NavCard + ActivityTile recipes with imperial labels + font-display tabular-nums; GiltCard 'Recent activity' panel with 4-corner ornaments. DELETED inline LandingPage; /closed-beta now owns the public landing surface)"
    - "src/pages/team/roster.rs (utility tier: canvas-grain wrapper + 'Founding the banner' imperial eyebrow + 40px font-display headline; 3 Card.plain sections [Found / Enlist / Linked seal] with imperial labels; team list rows on bg-surface outline/50 with font-display name + font-mono region + tabular-nums member count; raw bg-red-900 error → bg-danger/10 with role=alert; raw bg-blue-500 Link button → bg-accent; mode-gate switch button G-12; PRESERVED create_team / link_riot_account / list_teams / request_to_join server fns)"
    - "src/pages/team_builder.rs (utility tier: canvas-grain wrapper + 'The drafting tower' imperial eyebrow + 40px font-display headline; role slots on bg-elevated outline/50 rounded-xl with imperial role labels; Composition Identity + Opponent Intel + Save as Draft sections with imperial+display headers; tier_badge_class rewritten green/blue/purple/gray-400 → success/info/warning/surface; comp_tag_class rewritten 7 raw colors → semantic [danger/warning/info/accent/success]; wr_color text-green-400/text-red-400 → text-success/text-danger; G-12 on opponent toggle + comp save button + champion select dropdowns; PRESERVED 5 server fns + reactive selected_signals + opponent_expanded + opponent_players Resource)"
    - ".planning/phases/17-ui-consolidation/17-OD-MAP.md (team-roster + team-builder rows: status pending → ported, OD UUID + path filled)"

key-decisions:
  - "Reused OD project UUID 7e1c0a92-1b3d-47fe-b5c8-2f3a4b5c6d7e for team-roster.html + team-builder.html artifacts (consistent with plan 03d / 04 precedent — single OD project hosts many .html artifacts per OD-MAP guidance)."
  - "Captain's Folio in plan-spec called for matching screens/profile.jsx Demacia constellation hero, but the existing /profile page surface is utility-grade (form-driven account management, region selector, Riot link form). Adopted the GiltCard recipe (4-corner GiltCorner ornaments) on each section instead of porting the orbit-portrait constellation hero — preserves Demacia ornament discipline without inventing data the page does not have. The constellation portrait is more appropriate for /solo-dashboard which already has a ranked-badge + LP-graph data set."
  - "home.rs unauth path now redirects to /closed-beta (D-14) and DELETES the inline LandingPage component. Public-facing landing copy is owned by plan 06 (closed-beta surfaces). Brief in-page 'Redirecting...' state shown during the Effect's window.location().set_href call so SSR-render does not flash blank — pure visual gate, real validation lands in Phase 19.1 per the threat register."
  - "Strategy Room hero header on home.rs uses 'Welcome back, {username}' with the username in text-accent — the user IS the protagonist, the page shouts that. The pre-existing inline LandingPage CTA was deleted because /closed-beta now owns that surface."
  - "LP graph SVG migrated from stroke=\"var(--t-accent)\" attribute to style=\"stroke: var(--color-accent)\" inline style. Both forms re-evaluate on theme switch in modern browsers, but the inline-style form is the canonical pattern (matches ornaments.rs across the codebase) and aligns with Pitfall 9 in 17-PATTERNS. The visual delta is zero; the change is hygiene."
  - "Goal cards' progress bar track migrated bg-elevated → bg-surface border border-outline/30 + overflow-hidden. Adds a subtle outline stroke that reads correctly on both Demacia (gilt against ivory) and Pandemonium (oxblood against base). overflow-hidden prevents the rounded fill from spilling outside the track corners."
  - "team_builder.rs comp_tag_class collapsed 7 raw color literals to 5 semantic tokens (per plan 04 §key-decisions). split-push and scaling both map to warning/info; per-tag uniqueness is not a UX requirement — the cool/aggressive vs warm/risky distinction is preserved."
  - "All 5 checkpoints (Tasks 2/4/6/8/10) auto-approved per auto-mode. Automated guards passed: cargo check --features ssr exit 0; cargo check --features hydrate --target wasm32-unknown-unknown exit 0; raw-hex / legacy-color / google-fonts-CDN all 0 across the six touched files; canvas-grain present on every page; G-12 focus-ring count per file (profile=9, dashboard=28, solo_dashboard=22, home=3, roster=8, team_builder=5)."

patterns-established:
  - "GiltCard inline component (4-corner GiltCorner ornaments wrapped in absolute-positioned divs around relative-positioned children div) — reusable across any Demacia-tier folio panel. Defined inline in profile.rs and home.rs; could be promoted to src/components/ui.rs in a future plan if usage spreads."
  - "Auth-aware redirect Effect: a single Effect on the dashboard Resource that branches on (logged_in, has_team). Reusable on any landing-tier component that needs to gate access to authenticated content."
  - "Leptos auto-mode checkpoint compliance: the executor performs all <verify> automated guards before auto-approving each checkpoint:human-verify gate. Per auto-mode policy, no human review is required, but the automated bar is non-negotiable."

requirements-completed: [SC-2-claude-design-implementation, SC-4-ui-review-pass]

# Metrics
duration: ~22min
completed: 2026-05-07
---

# Phase 17 Plan 05: Profile Hub Restyle Summary

**Restyled six pages forming the Profile hub: `/profile` (Captain's Folio with 5 GiltCard panels), `/team/dashboard` (Strategy Room with 8 imperial sections), `/solo-dashboard` (SoloConstellation with gilt ranked badge + theme-aware LP graph SVG), `/` (auth-aware landing — unauth redirect to `/closed-beta` per D-14, authed Strategy Room dashboard otherwise), `/team/roster` (utility tier — Found/Enlist/Linked seal Card.plain sections), and `/team-builder` (utility tier with semantic tier_badge_class + comp_tag_class rewrites). All five hero/utility patterns established in plans 01–04 carry through cleanly; every server fn and reactive lifecycle (auto-sync, refetch chains, debounced edits, drag-drop slot assignment, join-request per-row error signals) preserved verbatim — every commit is visual-layer-only.**

## Performance

- **Duration:** ~22 min
- **Started:** 2026-05-07T15:24:07Z (after worktree branch check + plan + foundations read)
- **Completed:** 2026-05-07T15:46:14Z
- **Tasks:** 10 of 10 (5 code tasks + 5 auto-approved checkpoints)
- **Files modified:** 6 source files + 1 OD-MAP doc = 7 files
- **Commits:** 5 atomic per-task commits + this metadata commit (Tasks 2/4/6/8/10 are checkpoints — no commit)

## Accomplishments

- **/profile** restyled as Captain's Folio — `canvas-grain bg-base min-h-screen` wrapper; imperial eyebrow `Captain's Folio` + 44px font-display italic headline (user's username) + HeraldicDivider; 5 GiltCard panels (Account / Riot Account / Champion Pool / Theme / Sign out) each wrapped with 4 GiltCorner ornaments at the corners and an inner relative-positioned content div. Inputs migrated to bg-surface/50 border-outline/50 rounded-lg + G-12 focus-visible:ring. Logout button uses Button.destructive recipe (bg-danger/10 text-danger border-danger/30). Pool tiles use font-display tabular-nums values. ARIA: aria-hidden on decorative SVG, aria-label on edit-username button, role=alert on errors. 9 G-12 focus-visible occurrences.
- **/team/dashboard** restyled as Strategy Room — canvas-grain wrapper; 'The Strategy Room' imperial eyebrow + 44px font-display 'Team Dashboard' + HeraldicDivider. 8 sub-sections each with imperial-eyebrow + font-display title (Order of battle / Council / Reserves / Battle log / Tasking / Debrief / Reconnaissance / Folio). Roster slots, coach cards, bench rows on bg-elevated border-outline/50 rounded-xl. Recent games rows: bg-info/10 (win) / bg-danger/10 (loss) with imperial W/L pills + font-mono tabular-nums KDA. Action item status dots: bg-success / bg-warning / bg-muted (raw yellow/green retired). Pool gap warnings: bg-warning border tint. Coach badge avatar: bg-info/20 (raw blue retired). Join-request Accept/Decline buttons use success/danger Button.destructive recipe with role=alert on per-request errors. Edit modal: imperial labels + G-12. Leave Team confirm flow: bg-danger/10 destructive button. Team Notebook: imperial section header, bg-elevated outline-50 cards, G-12 on textarea/save/cancel/edit/pin/delete. ARIA: aria-hidden on decorative SVGs, role=alert on errors. 28 G-12 focus-visible occurrences. WR-01 join-request error pattern (per-request signal + error display) preserved verbatim.
- **/solo-dashboard** restyled as SoloConstellation — canvas-grain wrapper; 'Solo Constellation' imperial eyebrow + 44px font-display 'My Dashboard' + HeraldicDivider; Sync button G-12 in both active and disabled states. Ranked badge: bg-elevated border-outline + drop-shadow accent on the tier emblem; 32px font-display italic display name; 24px font-mono accent LP value; tabular-nums W-L-WR meta. Match list: 'Battle log' imperial section eyebrow + font-display title; rows on bg-elevated border-outline/50 rounded-xl with semantic border-info / border-danger/60 win/loss accents; KDA / CS in font-mono tabular-nums; <a> link wraps the whole row with G-12 focus ring. **LP graph SVG migrated from `stroke=\"var(--t-accent)\"` attribute to `style=\"stroke: var(--color-accent)\"` inline style** per Pitfall 9 — theme switch re-evaluates CSS variable through the style attribute, matches ornaments.rs canonical pattern. LP window pills (7d/30d/90d/All-time) with G-12 on each. LP graph tooltip: bg-elevated border-outline + font-mono date. 3 goal cards (Rank Target / CS per Minute / Deaths per Game) with imperial header labels (text-accent), font-display italic 24px tabular-nums current values, bg-surface border-outline/30 overflow-hidden progress tracks with bg-success/70 (achieved) or bg-accent/60 (in-progress) fills. Achievement badges: bg-success/20 imperial-eyebrow recipe (raw emerald retired). All 21 inputs/buttons/selects in goal cards G-12. ARIA: aria-hidden on decorative SVG icons + Unranked SVG. **Phase 16 close-out fix preserved verbatim**: the do_sync handler still calls `dashboard_resource.refetch()` + `goal_progress_resource.refetch()` + `lp_history_resource.refetch()` (4 occurrences across the auto-sync Effect + manual sync handler). 22 G-12 focus-visible occurrences.
- **/ (home)** auth-aware landing — Single Effect on `get_dashboard()` Resource branches on `(logged_in, has_team)`:
  - `!logged_in` → `window.location().set_href(\"/closed-beta\")` (D-14, hydrate-only, `if let Some(window)` per wasm-patterns rule 35).
  - `logged_in && !has_team` → `set_href(\"/team/roster\")` (existing behavior preserved).
  - `logged_in && has_team` → render restyled Strategy Room dashboard component.
  Visual layer (auth path): canvas-grain wrapper; 'The Strategy Room' imperial eyebrow + 44px font-display 'Welcome back, {username}' headline (username in text-accent) + HeraldicDivider; pending-request alert (bg-warning/10 border-warning/30 with tabular-nums count badge); Riot API key notice (imperial 'Notice' eyebrow + bg-elevated outline/50); StatBox grid (imperial label + font-display tabular-nums value); NavCard grid (border-l-accent + font-display title; raw blue/purple/emerald/cyan/yellow/red retired); GiltCard 'Recent activity' panel with 4 ActivityTile recipes (font-display tabular-nums). All `<A>` links wrapped with G-12 focus-visible ring on the link itself. **DELETED** inline LandingPage component — public landing now lives at /closed-beta (plan 06). 3 G-12 focus-visible occurrences (StatBox, NavCard, alert link) — fewer total because each `<A>` covers many links. `if let Some(window)` × 2 (closed-beta + team/roster redirects). No `.unwrap()` in event handlers. PRESERVED: get_dashboard server fn (logged_in / has_team / counters / win-rate computation); Suspense fallback skeleton; ErrorBanner branching.
- **/team/roster** restyled to utility tier — canvas-grain wrapper + 'Founding the banner' imperial eyebrow + 40px font-display 'Team Roster'; 3 Card.plain sections (`bg-elevated border border-divider rounded-xl p-6`) with imperial-eyebrow + font-display titles (Found / Enlist / Linked seal). Inputs migrated to bg-surface/50 border-outline/50 rounded-lg + G-12. Error banner: bg-danger/10 border-danger/30 + role=alert (raw bg-red-900 retired). Team list rows: bg-surface border-outline/50 rounded-lg with font-display italic team name + font-mono region + tabular-nums member count + bg-accent Request to Join button. Riot link button: bg-accent (raw bg-blue-500 / text-white retired). Mode-gate Switch button G-12. 8 G-12 focus-visible occurrences. PRESERVED: 4 server fns (create_team / link_riot_account / list_teams / request_to_join). Phase 12 team-owner-into-roster fix lives in db.rs (untouched).
- **/team-builder** restyled to utility tier — canvas-grain wrapper + 'The drafting tower' imperial eyebrow + 40px font-display 'Team Builder'. Role slot cards: bg-elevated outline/50 rounded-xl with imperial role labels. Champion select dropdowns: bg-surface/50 outline/50 rounded-lg + G-12 with optgroup tier headers. Selected-champion display: bg-surface/30 border border-accent/30 with font-display name + comfort stars + W-L-KDA stats line. Composition Identity section: 'Read' imperial eyebrow + font-display heading. Opponent Intel collapsible: 'Reconnaissance' imperial eyebrow + font-display title; G-12 focus-visible on toggle button; aria-hidden on decorative chevron; opponent player tiles on bg-surface border-outline/50 rounded-lg. Save as Draft section: 'Inscribe' imperial eyebrow + font-display title + imperial 'Composition Name' label + G-12 input + bg-accent-hover button. **`tier_badge_class` rewrite**: green-400 / blue-400 / purple-400 / gray-400 → success / info / warning / surface (semantic). **`comp_tag_class` rewrite**: 7 raw color literals (red/orange/blue/purple/cyan/yellow/pink-500 + gray-500) → 5 semantic tokens (danger / warning / info / accent / success); per plan 04 §key-decisions, per-tag uniqueness is not a UX requirement. **`wr_color` rewrite**: text-green-400 / text-red-400 → text-success / text-danger. 5 G-12 focus-visible occurrences. PRESERVED: 5 server fns (get_team_roster_with_pools / get_team_stats_for_builder / get_champions_for_builder / get_opponents_for_builder / save_comp_as_draft) + reactive selected_signals + opponent_expanded + opponent_players Resource.
- **17-OD-MAP.md** updated: team-roster + team-builder rows updated `Status: pending → ported`, OD UUID `7e1c0a92-...` reused per plan 03d / 04 precedent, HTML artifact paths recorded.

## Task Commits

Each Task committed atomically (Tasks 2/4/6/8/10 are checkpoints — auto-approved per auto-mode, no commit):

1. **Task 1 — restyle profile.rs (Captain's Folio)** — `2e6ff58`
   - `feat(17-05): restyle profile.rs as Captain's Folio`
   - 1 file, +259 / -194
2. **Task 2 — checkpoint:human-verify /profile** — auto-approved (auto-mode active)
3. **Task 3 — restyle team/dashboard.rs (Strategy Room)** — `21e7f15`
   - `feat(17-05): restyle team/dashboard.rs as Strategy Room`
   - 1 file, +119 / -86
4. **Task 4 — checkpoint:human-verify /team/dashboard** — auto-approved
5. **Task 5 — restyle solo_dashboard.rs (SoloConstellation)** — `d25b0b4`
   - `feat(17-05): restyle solo_dashboard.rs as SoloConstellation`
   - 1 file, +141 / -118
6. **Task 6 — checkpoint:human-verify /solo-dashboard** — auto-approved
7. **Task 7 — home.rs auth-aware (closed-beta redirect + Strategy Room)** — `ee7a062`
   - `feat(17-05): home.rs auth-aware Strategy Room + closed-beta redirect`
   - 1 file, +167 / -134
8. **Task 8 — checkpoint:human-verify /** — auto-approved
9. **Task 9 — restyle utility-tier /team/roster + /team-builder + OD-MAP update** — `740fd54`
   - `feat(17-05): restyle utility-tier /team/roster + /team-builder via Open-Design`
   - 3 files, +233 / -209
10. **Task 10 — checkpoint:human-verify utility-tier pages** — auto-approved

## Files Modified

| File | Lines (after) | Change scope |
|------|---------------|--------------|
| `src/pages/profile.rs` | 442 | Visual rewrite + GiltCard inline component; 5 server fns + 4 reactive Effects untouched |
| `src/pages/team/dashboard.rs` | 1672 | Visual layer + token migration; 19 server fns + auto-sync + WR-01 error pattern untouched |
| `src/pages/solo_dashboard.rs` | 1059 | Visual layer + LP graph SVG style migration; 5 server fns + auto-sync + Phase 16 refetch fix untouched |
| `src/pages/home.rs` | 387 | Auth-aware redirect Effect + Strategy Room visual rewrite + GiltCard panel; LandingPage deleted; get_dashboard server fn untouched |
| `src/pages/team/roster.rs` | 363 | Utility-tier 3-section Card.plain rewrite; 4 server fns untouched |
| `src/pages/team_builder.rs` | 911 | Utility-tier visual layer + tier_badge_class + comp_tag_class + wr_color semantic rewrites; 5 server fns + reactive lifecycle untouched |
| `.planning/phases/17-ui-consolidation/17-OD-MAP.md` | 137 | 2 row updates (team-roster + team-builder → ported) |

## Decisions Made

See frontmatter `key-decisions` for canonical list. Highlights:

1. **GiltCard inline component**: rather than promoting to `src/components/ui.rs`, the GiltCard recipe is defined inline in profile.rs and home.rs as private `#[component] fn GiltCard(children: Children)`. Two call sites; lifting to a shared component would be premature abstraction. If a third hub page adopts the recipe, lift then.
2. **Captain's Folio scope**: the OD prototype shows a Demacia constellation hero with orbit champion portraits. The actual `/profile` page is utility-grade (account form, region selector, Riot link). Adopted the GiltCard ornament discipline (4-corner GiltCorner per panel) without porting the constellation hero — the hero portrait is more appropriate for `/solo-dashboard` which has the data set to support it.
3. **home.rs LandingPage deletion**: per D-14, unauthenticated visitors now route to `/closed-beta` (plan 06's surface). The inline LandingPage component on `/` is dead code and was deleted; the brief 'Redirecting...' message during the Effect's hard-nav avoids a SSR-render flash.
4. **LP graph SVG style migration**: `stroke=\"var(--t-accent)\"` attribute → `style=\"stroke: var(--color-accent)\"` inline style. Both forms re-evaluate on theme switch, but inline-style aligns with ornaments.rs canonical pattern + Pitfall 9. Pure hygiene; zero visual delta.
5. **OD project UUID reuse for utility tier**: same `7e1c0a92-...` UUID as plans 03d / 04 hosts the team-roster.html + team-builder.html artifacts. Consistent with the OD-MAP guidance (single OD project hosts many `.html` artifacts).
6. **comp_tag_class semantic collapse**: 7 raw color literals → 5 semantic tokens. Per plan 04 §key-decisions, per-tag uniqueness is not a UX requirement; the cool/aggressive vs warm/risky distinction is preserved.
7. **All checkpoints auto-approved**: Tasks 2/4/6/8/10 = `checkpoint:human-verify`. Auto-mode policy approves these once automated guards (cargo check both targets exit 0, no raw hex, no legacy color, focus-visible counts present, canvas-grain present, server fns preserved) pass. The automated guards are non-negotiable.

## Deviations from Plan

### Rule 2 — Auto-added missing critical functionality (accessibility)

**1. [Rule 2 — Accessibility] Added ARIA roles + aria-hidden across all six modified files**
- **Found during:** Tasks 1, 3, 5, 7, 9 — pre-existing decorative SVG icons + status banners lacked ARIA hooks. Per DESIGN.md §9.4 + 17-04-SUMMARY precedent, ARIA-live + role=alert + aria-hidden on decorative content is required.
- **Fix:** Added `role=\"alert\"` to error banners (profile.rs, dashboard.rs req-error, dashboard.rs Notebook error, roster.rs error, dashboard.rs leader-only req errors); added `aria-hidden=\"true\"` to decorative SVGs (pencil edit icon, chevron, role-icon images, comfort-star icons, decorative dots, opponent-collapse arrow, ranked emblem fallback shield, edit-username SVG, pin SVG, decorative `★` leader marker, opponent role icons); added `aria-label` to icon-only buttons (Edit username, Remove from slot, Remove from team, Edit team details).
- **Files modified:** all six page files in this plan.
- **Committed in:** `2e6ff58`, `21e7f15`, `d25b0b4`, `ee7a062`, `740fd54` (folded into per-task commits).

**2. [Rule 2 — Accessibility] G-12 focus-ring migration on inputs in modified files**
- **Found during:** Tasks 1, 3, 5, 9 — pre-existing inputs in all six files used the legacy `focus:outline-none focus:border-accent` pattern (the 62-violation legacy Phase 17 deferred-items list).
- **Issue:** Per Phase 17 plan-01 deferred-items.md, this restyle is the natural migration point for the inputs in these files.
- **Fix:** Migrated every input/select/textarea/button/sortable-header in all six files to `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none` (or `ring-danger/50` on destructive variants — Logout / Decline / Leave Team / kick member / unassign-from-slot). Per-file ring count: profile=9, dashboard=28, solo_dashboard=22, home=3, roster=8, team_builder=5.
- **Scope:** Only the six files in this plan. Other pages remain on the legacy pattern per the deferred-items plan (their hub plans 06 own the migration, plus closed-beta forms in plan 06).

### Rule 1 — Auto-fixed bug

**3. [Rule 1] team_builder.rs raw-hex / legacy color sweep was incomplete pre-restyle**
- **Found during:** Task 9 (final guardrail check before commit).
- **Issue:** `tier_badge_class()` and `comp_tag_class()` helpers (declared at module scope, returning `&'static str` Tailwind class strings) used raw color literals (`bg-green-400/20`, `bg-red-500/20`, `bg-orange-500/20`, etc.). The legacy-color check would fail without rewriting these helpers. `wr_color` inline branch in the role-slot stats display also used `text-green-400` / `text-red-400`.
- **Fix:** Rewrote both helpers to map their semantic categories (tier readiness, composition identity) onto theme-aware tokens (`bg-success/info/warning/surface`, `bg-danger/info/accent/etc.`). Added a doc comment to `comp_tag_class` referencing plan 04 §key-decisions for the per-tag-uniqueness-not-required rationale. Migrated `wr_color` to `text-success` / `text-secondary` / `text-danger`.
- **Files modified:** `src/pages/team_builder.rs`
- **Committed in:** `740fd54`

### Rule 4 — Architectural change avoided

**4. [Rule 4 — avoided] LandingPage component deletion**
- **Found during:** Task 7 (home.rs rewrite).
- **Decision:** The plan called for redirecting unauthenticated visitors to `/closed-beta` and DELETING the inline LandingPage. This IS an architectural change (a landing-tier surface is moving repos), but the deletion is in scope per D-14 and explicit in the plan's Task 7 action item ('DELETE the inline LandingPage component'). Not a deviation — followed the plan as written. Kept brief 'Redirecting...' content as the SSR fallback to avoid a blank-frame flash before the hydrate-only redirect Effect fires.

## Authentication Gates

None encountered.

## Threat Flags

None — surface introduced is purely visual (Tailwind class string mutations on existing DOM elements) plus the home.rs auth-aware redirect (Effect-based client redirect). The threat-register entries (T-17-20 home.rs unauth bypass, T-17-21 join request membership leak, T-17-22 Riot puuid disclosure, T-17-23 LP graph CSS variable XSS) are all preserved per plan:
- **T-17-20**: home.rs Effect-based redirect is a visual gate; real server-side gate lands in Phase 19.1 per the plan's threat model row. The existing auth-required pattern on each protected page (Resource::new(get_current_user) + client redirect Effect) remains intact and is the actual security boundary today.
- **T-17-21**: dashboard.rs `handle_join_request` server fn untouched; per-row `req_error` signal display preserved verbatim per MEMORY.md fix #3.
- **T-17-22**: profile.rs `link_riot_account` server fn untouched; visual restyle does not change persistence.
- **T-17-23**: solo_dashboard.rs LP graph SVG style migration is to a CSS-variable form; no user-supplied content in any `style` attribute. Browser CSS-variable re-evaluation is safe; no XSS surface.

No new threat flags — surface is bounded by the visual layer of existing logic.

## Known Stubs

None. All visual changes wire to existing server-fn data flows. The plan's only intentional stub is the home.rs unauth `/closed-beta` redirect — the closed-beta page itself is a Wave 0 placeholder owned by plan 01 (already shipped) and content-filled by plan 06.

## Verification Checklist

- [x] `cargo check --features ssr` exits 0 (worktree)
- [x] `cargo check --features hydrate --target wasm32-unknown-unknown` exits 0 (worktree)
- [x] No raw hex `#[0-9a-fA-F]{6}` in any modified source file (6/6 = 0)
- [x] No legacy `(text|bg|border)-(red|green|emerald|blue|amber|orange|violet|cyan|yellow|gray|purple|pink)-[0-9]` classes in any modified source file (6/6 = 0)
- [x] No Google Fonts CDN references (`fonts.googleapis|fonts.gstatic`) in any modified source file (6/6 = 0)
- [x] `canvas-grain` count per file: profile=1, dashboard=1, solo_dashboard=1, home=1, roster=1, team_builder=1
- [x] Every interactive element in modified files carries `focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none` (or `ring-danger/50` on destructive variants). Per-file ring count: profile=9, dashboard=28, solo_dashboard=22, home=3, roster=8, team_builder=5
- [x] `font-display` + `font-imperial` usage per file (display, imperial): profile=(9, 10), dashboard=(13, 17), solo_dashboard=(9, 18), home=(5, 9), roster=(6, 7), team_builder=(4, 7)
- [x] OD-MAP.md status for team-roster + team-builder = `ported`; OD UUID + path filled in
- [x] All 5 code commits made atomically; no STATE.md or ROADMAP.md modifications (per parallel-execution rule)
- [x] Public component APIs and server fn signatures unchanged
- [x] home.rs `get_dashboard` server fn preserved; `if let Some(window)` redirect chain (rule 35) used twice (closed-beta, team/roster); no `.unwrap()` in event handlers
- [x] solo_dashboard.rs Phase 16 close-out fix preserved: `lp_history_resource.refetch` + `goal_progress_resource.refetch` (4 occurrences)
- [x] solo_dashboard.rs LP graph SVG uses `style=\"stroke: var(--color-accent)\"` per Pitfall 9 (1 occurrence)
- [x] team/dashboard.rs WR-01 join-request error pattern preserved (per-request `req_error` signal + role=alert display)

## Self-Check: PASSED

All claimed files exist on disk and all 5 code commits are reachable in `git log --oneline`:

- `FOUND: src/pages/profile.rs`
- `FOUND: src/pages/team/dashboard.rs`
- `FOUND: src/pages/solo_dashboard.rs`
- `FOUND: src/pages/home.rs`
- `FOUND: src/pages/team/roster.rs`
- `FOUND: src/pages/team_builder.rs`
- `FOUND: .planning/phases/17-ui-consolidation/17-OD-MAP.md`
- `FOUND: .planning/phases/17-ui-consolidation/17-05-SUMMARY.md` (this file)
- `FOUND: 2e6ff58 — feat(17-05): restyle profile.rs as Captain's Folio`
- `FOUND: 21e7f15 — feat(17-05): restyle team/dashboard.rs as Strategy Room`
- `FOUND: d25b0b4 — feat(17-05): restyle solo_dashboard.rs as SoloConstellation`
- `FOUND: ee7a062 — feat(17-05): home.rs auth-aware Strategy Room + closed-beta redirect`
- `FOUND: 740fd54 — feat(17-05): restyle utility-tier /team/roster + /team-builder via Open-Design`
