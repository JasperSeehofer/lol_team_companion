# Phase 19 — Bug-Report Widget (SPEC seed)

**Status:** SEED — produced by the v1.2 → v1.3 pivot on 2026-05-06. Renumbered from Phase 18 → 19 on 2026-05-11 when Phase 18 (Region Variants) was inserted. Run `/gsd-spec-phase 19` (or `/gsd-discuss-phase 19`) to expand.

**Milestone:** v1.3 Launch Readiness

## Goal (one sentence)

A floating in-app feedback button that lets the user click any page element, describe what went wrong or what they wish for, and have it auto-exported to a Claude-readable inbox file so the next coding session sees the queue without manual triage.

## User story

> As a closed-beta user, I notice the LP graph hover doesn't work. I click the "Report" button at the bottom-right of the page, then click the LP graph. A modal opens with "Solo dashboard → LP history graph" pre-filled as the element label. I type "Hover tooltip never appears even though I clearly saw it work last week" and pick "bug". I submit. Two days later, in the next Claude Code session, my report is the first item in `.planning/INBOX/bug-reports.md` and gets fixed in the same hour.

## Capture model (DECIDED in pivot — no debate)

When the user clicks an element after entering "select" mode, capture:

- **Page URL** (e.g. `/draft`)
- **Semantic element label** (e.g. "Champion picker → search input"); derived from the nearest `data-feedback-label` attribute on the clicked element or its ancestors. NOT a CSS selector — selectors don't survive recompile (per `.claude/rules/wasm-patterns.md` rule 56)
- **Free-text user description**
- **Category toggle**: `bug` | `wishlist` (radio, not checkbox — exactly one)
- **User attribution**: `user_id`, `created_at`
- **Optional**: `viewport_width`, `viewport_height` (browser metadata — useful for responsive-bug triage)

**NOT captured (deferred or rejected):**
- Screenshots (deferred — html2canvas adds ~1 day; revisit post-launch if reports are unclear without them)
- CSS selectors (rejected — fragile across recompiles)
- IP address (rejected — DSE simplification)
- User-agent string (deferred — add later if a browser-specific bug pattern emerges)

## In-scope

### Server-side
1. New `bug_report` SurrealDB table with fields: `id`, `user`, `page_url`, `element_label`, `description`, `category`, `viewport_w`, `viewport_h`, `created_at`, `status` (`open`/`triaged`/`closed`).
2. Server fn `submit_bug_report(payload) -> Result<(), ServerFnError>`.
3. Server fn `list_bug_reports(filter) -> Result<Vec<BugReport>, ServerFnError>` — admin-only (matches the user_id of the project owner only, for now).
4. **Auto-export task**: on every server start, write `.planning/INBOX/bug-reports.md` with all open reports grouped by category (bug first, wishlist second) and recency (newest first). Format optimized for Claude ingestion: H2 per report, bullet-list of fields, then the description as a blockquote. Include a YAML front-matter footer with the latest export timestamp + total open count.

### Client-side
5. Floating "Report" button mounted globally in `src/app.rs` (bottom-right, semi-transparent until hovered).
6. Clicking the button enters "select" mode: cursor changes to crosshair, all elements with `data-feedback-label` get a subtle outline.
7. Clicking any element opens a modal pre-filled with that element's label.
8. Modal: textarea + radio (bug/wishlist) + Submit + Cancel.
9. After submit, toast confirms ("Thanks! Your report is in.") and modal closes.

### Documentation
10. Add a top-level section to `CLAUDE.md` (or `.claude/rules/`) telling future Claude sessions where the inbox lives and how to triage it (`/gsd-inbox` or manual read).
11. Tag UI elements throughout the app with `data-feedback-label="..."` attributes — phase 17 (UI Consolidation) and phase 19 share this work; coordinate.

## Out of scope

- Email notifications (deferred — closed-beta scope is small enough that inbox-driven triage is sufficient)
- Public bug tracker (deferred — no GitHub Issues integration in v1.3)
- Screenshot capture (deferred — see Capture model)
- Voting / upvoting on wishlist items (deferred to v1.4 post-launch backlog)

## Success criteria (verify with `/gsd-verify-work 19`)

1. `bug_report` table exists in `schema.surql` and a fresh DB applies it cleanly
2. Floating "Report" button visible on every authenticated page
3. Clicking the button enters select mode; clicking any tagged element opens a modal pre-filled with that element's label
4. Submitting persists a row and closes the modal
5. Restarting the server writes `.planning/INBOX/bug-reports.md` with all open reports formatted as specified
6. The inbox file is referenced from `CLAUDE.md` so the next Claude session discovers it on context load
7. No dark patterns (per `[[guardrails#G-10]]`) — neutral language, no pre-filled ratings, no confirmshaming
8. The capture flow is documented in the DSE / Tier-A transparency table (coordinates with Phase 22)

## Required reading before discuss-phase

1. `src/pages/action_items.rs` — closest analog (similar generic CRUD + auth pattern)
2. `src/server/auth.rs` — AuthSession extraction in server fns
3. `src/server/db.rs` — query patterns, RecordId handling
4. `.claude/rules/wasm-patterns.md` rule 35 (no `.unwrap()` in event handlers) and rule 56 (selectors don't survive recompile)
5. `.claude/rules/leptos-patterns.md` (server-fn boilerplate, mutation+refetch pattern)
6. `[[guardrails#G-10]]` (no dark patterns), `[[guardrails#G-13]]` (transparency requirement)

## Plans

TBD — produced by `/gsd-plan-phase 19`. Likely structure:
- 19-01: Schema + model + server fns + tests
- 19-02: UI widget (floating button, select mode, modal) + global mount in app.rs
- 19-03: Auto-export task + CLAUDE.md update + tag rollout to first-priority pages

---

This SPEC was seeded by the pivot. The user confirmed the capture model on 2026-05-06.
