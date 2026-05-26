# Phase 19: Bug-Report Widget — Context

**Gathered:** 2026-05-26
**Status:** Ready for planning
**Source:** Synthesized from `19-SPEC.md` (v1.2→v1.3 pivot decision, 2026-05-06). Yolo mode — `/gsd:discuss-phase` skipped because SPEC already locks every contested decision.

<domain>
## Phase Boundary

Deliver a floating in-app feedback widget on every authenticated page that lets a user click any tagged element, write a short bug/wishlist note, and have it persisted to SurrealDB AND auto-exported to a Claude-readable inbox file (`.planning/INBOX/bug-reports.md`) on every server start.

**In:** widget UI, capture flow, `bug_report` table + server fns, server-start auto-export task, `data-feedback-label` rollout to first-priority pages, `CLAUDE.md` reference to the inbox.

**Out:** screenshots (deferred), email notifications, public bug tracker, voting/upvoting, mobile redesign.

**Cross-phase coupling:**
- Phase 17 (UI Consolidation) shares the `data-feedback-label` rollout — coordinate naming.
- Phase 18/18.1/18.2 (Region Variants + hydration fixes) precede — Pandemonium hydration must already be clean before mounting WASM event handlers globally. Phase 18.2 LANDED 2026-05-26 ✓.
- Phase 22 (Compliance) will fold the capture flow into the Tier-A transparency table; record what data we collect.

</domain>

<decisions>
## Implementation Decisions (LOCKED)

### Capture model (D-01 — decided in pivot, no debate)
- D-01.1: Capture **page URL** (e.g. `/draft`) — read via `web_sys::window().location().pathname()` (hydrate-only).
- D-01.2: Capture **semantic element label** via `data-feedback-label` attribute on the clicked element OR its nearest ancestor. NEVER a CSS selector. Reason: Leptos recompile changes class hashes / ordering, breaking any selector-based identity.
- D-01.3: Capture **free-text user description** (textarea, no length cap on client; soft cap 4000 chars server-side).
- D-01.4: Capture **category**: `bug | wishlist` — radio button (exactly one), not checkbox. Default unselected; submit blocked until chosen.
- D-01.5: Capture **user attribution**: `user_id` (from `AuthSession`), `created_at` (server-side `time::now()`).
- D-01.6: Capture **viewport_width / viewport_height** (optional, browser metadata) — useful for responsive-bug triage. Read via `window().inner_width()/inner_height()` and cast to `i32`.
- D-01.7: Do NOT capture screenshots, CSS selectors, IP address, or User-Agent in v1.

### Storage (D-02)
- D-02.1: New SurrealDB table `bug_report` defined in `schema.surql` with fields: `id`, `user: record(user)`, `page_url: string`, `element_label: string`, `description: string`, `category: string` (`'bug'|'wishlist'`), `viewport_w: option<int>`, `viewport_h: option<int>`, `created_at: datetime`, `status: string` (`'open'|'triaged'|'closed'`, default `'open'`).
- D-02.2: Index on `(status, created_at DESC)` to make the auto-export query cheap.
- D-02.3: Follow the canonical `DbBugReport` ↔ shared `BugReport` split (DB struct in `src/server/db.rs` with `surrealdb::RecordId`, shared model in `src/models/bug_report.rs` with `String` id) — see auto-memory pattern.

### Server functions (D-03)
- D-03.1: `submit_bug_report(payload: NewBugReport) -> Result<(), ServerFnError>` — extracts `AuthSession` via `leptos_axum::extract`, validates category in {bug,wishlist} and description non-empty, inserts row. Owns DB via `use_context::<Arc<Surreal<Db>>>()` (NOT `axum::extract::State`).
- D-03.2: `list_bug_reports(filter) -> Result<Vec<BugReport>, ServerFnError>` — admin-only (returns `Forbidden` unless `auth.user.id == project_owner`). Used by the auto-export task to assemble the inbox file.
- D-03.3: Use owned `String` everywhere with `.bind(("foo", value))` — `bind()` requires `'static`.

### Auto-export task (D-04)
- D-04.1: Runs **once on every server start** (during `main.rs` startup, after DB init, before `axum::serve`). NOT a recurring background task in v1 — explicit "on start" keeps it deterministic and lets the user observe the result immediately on reload.
- D-04.2: Writes to `.planning/INBOX/bug-reports.md` (path relative to the project root / CWD at server start). Directory created if missing.
- D-04.3: File format optimized for Claude ingestion:
  - YAML front-matter header with: `exported_at: <iso8601>`, `total_open: <N>`, `by_category: { bug: <X>, wishlist: <Y> }`.
  - One `## ` heading per report, formatted as `## [bug|wishlist] {first 60 chars of description} — {date}`.
  - Bullet list of fields (URL, element label, user, viewport).
  - Description as a blockquote (`> …`).
  - Reports grouped by category (bug first, wishlist second), each group sorted newest-first.
- D-04.4: Only reports with `status='open'` are exported. Triaged/closed reports are dropped from the inbox file (they live in the DB until manually pruned).
- D-04.5: On write failure (permission denied, disk full): **log a warning and continue** — never block server start. Inbox is an aid, not the source of truth.

### Widget UI (D-05)
- D-05.1: Floating "Report" button mounted in `src/app.rs` at the top level (inside the `<Router>` but outside `<Routes>`), wrapped in `<Show when=move || auth_user.get().is_some()>` so it appears only on authenticated pages.
- D-05.2: Position: `fixed bottom-4 right-4 z-50`. Semi-transparent (`opacity-60`) at rest, full opacity on `hover:`. Uses semantic tokens (`bg-surface`, `text-primary`, `border-outline`). No raw hex per `[[guardrails]]`.
- D-05.3: Three UI states tracked by a single `RwSignal<WidgetState>` where `WidgetState ∈ {Idle, Selecting, Editing(ElementInfo)}`:
  - **Idle**: just the floating button.
  - **Selecting**: cursor → `crosshair` (set on `<body>`), all elements with `data-feedback-label` get a subtle outline (`outline-2 outline-accent/40`). One global click handler captures the first click, reads the label from the nearest ancestor with `data-feedback-label`, sets state to `Editing`.
  - **Editing**: modal opens (centered overlay), pre-filled with the captured label. Contains a textarea, two radios (bug/wishlist), Submit + Cancel.
- D-05.4: After Submit: show toast "Thanks! Your report is in." for ~3 s, close modal, reset to Idle. Use the existing `StatusMessage` component from `src/components/ui.rs`.
- D-05.5: Cancel + Esc + click-outside all return to Idle without persisting.
- D-05.6: **WASM safety:** zero `.unwrap()` in event handlers (rule 35). Use `if let Some(_)` / early-return on `web_sys` calls.

### Element labels (D-06)
- D-06.1: Tag the highest-value surfaces in v1 (Phase 19 scope): `/draft` (DraftPage), `/solo` (SoloPage), `/team/dashboard`, `/stats`, `/champion-pool`, `/game-plan`, `/post-game`. Other pages can land later without blocking Phase 19.
- D-06.2: Label hierarchy follows existing nav structure: `"<Page> → <Section> → <Element>"`. Examples: `"Draft → Blue side → Pick 3 slot"`, `"Solo dashboard → LP history graph"`, `"Stats → Match list → Filter dropdown"`.
- D-06.3: Tags live on the existing JSX-style markup — no new wrapper components. Use the natural ancestor structure already present (the `<div>` representing a section already exists; just add the attribute).

### Documentation (D-07)
- D-07.1: Add a top-level section to `CLAUDE.md` (or a new `.claude/rules/inbox.md` referenced from CLAUDE.md) named `### Bug-Report Inbox` that says:
  - The file lives at `.planning/INBOX/bug-reports.md`.
  - Future sessions should read it on context load if the file exists and `total_open > 0`.
  - Each report is actionable on its own — no triage step required before fixing.
- D-07.2: Add `.planning/INBOX/.gitkeep` so the directory exists in git even when empty.

### Dark-pattern guardrail (D-08, [[guardrails#G-10]])
- D-08.1: Neutral language only. No "Help us improve!", no exclamation points, no emoji. Button label is literally "Report". Radio labels are literally "bug" and "wishlist" (lowercase, neutral).
- D-08.2: No pre-filled ratings, no star widgets, no NPS prompts.
- D-08.3: No confirmshaming on Cancel (no "Are you sure you want to give up?"). Cancel just closes.
- D-08.4: No forced sign-up nag — widget only mounts when the user is already authenticated (D-05.1 covers this).

### Transparency (D-09, [[guardrails#G-13]])
- D-09.1: Record what data the widget collects in Phase 22 (Compliance & Transparency)'s Tier-A transparency table. Phase 19 lands the technical capture; Phase 22 fills the user-facing disclosure. This handoff is explicitly tracked here so Phase 22 doesn't re-discover it.
- D-09.2: Until Phase 22 ships the public DSE update, the widget is closed-beta-only (matches the user attribution gate — only logged-in users can submit, and closed-beta is gated by Phase 20.1).

### Claude's Discretion

Areas not pre-decided — the planner / executor picks based on existing patterns:
- Exact YAML front-matter key names in the inbox file (as long as `exported_at` and `total_open` are present).
- Exact toast duration (2–4 s is fine).
- Whether the modal uses an existing modal component or inlines a `<dialog>` element (whatever matches existing UI patterns).
- Exact CSS for the floating button's visual treatment (semantic tokens, but design details).
- Whether `data-feedback-label` tagging in D-06.1 lands as one task or one task per page — planner judgment.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Source of truth — closest analogs
- `src/pages/action_items.rs` — closest CRUD analog (auth-extraction + insert + list pattern). Read FIRST.
- `src/server/db.rs` — every server-side DB query pattern, `RecordId`-vs-`String` split, `bind()` with owned strings.
- `src/server/auth.rs` — `AuthSession` extraction inside server fns (`leptos_axum::extract`, `let mut auth`).
- `src/main.rs` — server-start hooks (where the auto-export task plugs in, between DB init and `axum::serve`).
- `src/app.rs` — global mounts (where the floating button plugs in, top-level inside `<Router>`).
- `schema.surql` — schema additions (`DEFINE TABLE`, `DEFINE FIELD`, `DEFINE INDEX`).

### Project rules (must respect)
- `.claude/rules/leptos-patterns.md` — server-fn boilerplate (incl. `ActionForm`-class caveat), mutation+refetch, `Callback::new`.
- `.claude/rules/wasm-patterns.md` rule 35 — never `.unwrap()` in event handlers.
- `.claude/rules/wasm-patterns.md` (general) — selectors don't survive recompile → use `data-feedback-label`.
- `.claude/rules/surreal-patterns.md` — `type::record('table', $key)`, owned-string `bind`, typed structs (no `serde_json::Value`).

### Wiki / Guardrails
- `../professional-vault/wiki/meta/guardrails.md#G-10` — no dark patterns (severity: high).
- `../professional-vault/wiki/meta/guardrails.md#G-13` — transparency required for data processing.
- `../professional-vault/wiki/concepts/design-system.md` — semantic tokens, no raw hex.
- `../professional-vault/wiki/concepts/ui-guidelines.md` — modal + button + accessibility rules.
- `../professional-vault/wiki/concepts/accessibility-standards.md` — focus rings, ARIA, keyboard reachability.

### Spec seed (parent doc)
- `.planning/phases/19-bug-report-widget/19-SPEC.md` — the original locked SPEC. Any conflict between this CONTEXT.md and the SPEC is a bug in CONTEXT.md, not the SPEC.

</canonical_refs>

<specifics>
## Specific Ideas

- **First-priority tag pages** (D-06.1): `/draft`, `/solo`, `/team/dashboard`, `/stats`, `/champion-pool`, `/game-plan`, `/post-game`. Lower-priority pages (e.g. `/team/roster`, `/profile`, `/action-items`, `/opponents`, `/team-builder`, `/tree-drafter`) can be tagged in the same task or deferred without blocking Phase 19 success criteria.
- **Inbox header example**:
  ```yaml
  ---
  exported_at: 2026-05-27T03:14:00Z
  total_open: 4
  by_category:
    bug: 3
    wishlist: 1
  ---
  ```
- **Report entry example**:
  ```markdown
  ## [bug] Hover tooltip never appears on LP history graph — 2026-05-26
  - URL: `/solo`
  - Element: `Solo dashboard → LP history graph`
  - User: `jasperseehofermusic@gmail.com`
  - Viewport: 1920×1080
  - Submitted: 2026-05-26T18:42:11Z

  > Hover tooltip never appears even though I clearly saw it work last week.
  ```
- **Inbox path resolution**: use `std::env::current_dir()` + `"./.planning/INBOX/bug-reports.md"` so it works whether the server is launched from the repo root or a worktree. Create parent dirs with `fs::create_dir_all`.

</specifics>

<deferred>
## Deferred Ideas

- **Screenshots** (html2canvas, ~1 day complexity) — revisit post-launch if reports are unclear.
- **Email/Slack notifications on new report** — inbox-driven triage suffices for closed-beta.
- **Voting / upvoting on wishlist items** — v1.4 post-launch backlog.
- **Public bug tracker / GitHub Issues integration** — not in v1.3.
- **Recurring background auto-export** (e.g. every 5 min) — server-start-only in v1 is deterministic and cheap; revisit if needed.
- **Admin triage page** (`/admin/bug-reports`) — `list_bug_reports` server fn exists but no UI; the inbox file IS the UI in v1.
- **User-agent / browser metadata** — add later if a browser-specific bug pattern emerges.

</deferred>

---

*Phase: 19-bug-report-widget*
*Context synthesized from 19-SPEC.md: 2026-05-26 (yolo, /gsd:discuss-phase skipped)*
