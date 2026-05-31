---
status: complete
phase: 19-bug-report-widget
source:
  - 19-01-SUMMARY.md
  - 19-02-SUMMARY.md
  - 19-03-SUMMARY.md
  - 19-04-SUMMARY.md
mode: automated-suite
started: "2026-05-31T10:48:40Z"
updated: "2026-05-31T10:48:40Z"
---

## Current Test

[testing complete]

## Tests

### 1. Cold Start Smoke Test
expected: |
  Fresh server boot (cargo leptos watch) applies schema.surql cleanly,
  /healthz returns 200, and the bug-report inbox export runs during startup.
result: pass
evidence: "Server ready after 89s; /healthz → 200. Startup log: 'Bug-report inbox exported: 0 open report(s) -> ./.planning/INBOX/bug-reports.md'. Schema applied without error."

### 2. SC-1 — bug_report table exists, fresh DB applies cleanly
expected: |
  schema.surql defines the bug_report table with category/status ASSERT
  guards; a fresh in-memory DB applies it and create/list round-trips.
result: pass
evidence: "schema.surql:323-336 (SCHEMAFULL table + status/created_at index). Unit tests: bug_report_create_and_list_round_trip, bug_report_rejects_invalid_category, bug_report_rejects_empty_description — all pass (132 passed / 0 failed)."

### 3. SC-2 — Floating Report button visible on every authenticated page
expected: |
  The floating 'Report' button is present on auth-required pages and
  absent on public pages (e.g. /auth/login).
result: pass
evidence: "e2e bug-report.spec.ts #1 (visible on auth-required pages) + #2 (hidden on public pages) — both pass."

### 4. SC-3 — Select mode captures the tagged element and pre-fills the modal
expected: |
  Clicking Report enters select mode; clicking a tagged element opens the
  modal pre-filled with the nearest data-feedback-label; Esc cancels cleanly.
result: pass
evidence: "e2e #3 (select mode captures nearest data-feedback-label via closest()) + #4 (Esc cancels select mode without opening modal) — both pass. Closure::forget lifecycle fix (commit b5a760b) prevents the listener-detach panic."

### 5. SC-4 — Submitting persists a row and closes the modal
expected: |
  Submitting the modal persists a bug_report row, shows the confirmation
  toast, and closes the modal.
result: pass
evidence: "e2e #5 (submit persists and shows toast) passes; DB persistence proved by bug_report_create_and_list_round_trip unit test."

### 6. SC-5 — Server restart writes .planning/INBOX/bug-reports.md
expected: |
  On every server start, all open reports are exported to the inbox file
  with the specified frontmatter format.
result: pass
evidence: "Startup log confirms export ran. File written at boot with frontmatter (exported_at, total_open: 0, by_category). Unit tests bug_report_export::writes_to_path + tolerates_unwritable_path pass. main.rs:54-58 wires BUG_REPORT_INBOX_PATH → export_open_reports."

### 7. SC-6 — Inbox referenced from CLAUDE.md
expected: |
  CLAUDE.md references the inbox file so the next Claude session discovers
  it on context load.
result: pass
evidence: "CLAUDE.md:248 — 'All open reports are auto-exported on every server start to .planning/INBOX/bug-reports.md (override via BUG_REPORT_INBOX_PATH env var).'"

### 8. SC-7 — No dark patterns (per guardrails G-10)
expected: |
  Neutral language, no pre-filled ratings, no confirmshaming, no emoji
  manipulation in the widget DOM.
result: pass
evidence: "e2e #6 (widget DOM has no forbidden characters — no 🐛/🎉/'Thanks!' exclamation form). Belt-and-suspenders over the 19-02 source-code grep."

### 9. SC-8 — Capture flow documented in DSE / Tier-A transparency table
expected: |
  The capture flow is documented in the Datenschutzerklärung / Tier-A
  transparency table.
result: skipped
reason: "Out of scope for Phase 19 execution — SPEC explicitly states this 'coordinates with Phase 22 (Compliance & Transparency)'. The DSE + Tier-A table are Phase 22 deliverables. Tracked via 19-HANDOFF-TO-22.md."

## Summary

total: 9
passed: 8
issues: 0
pending: 0
skipped: 1

## Gaps

[none — all in-scope success criteria pass; SC-8 deferred to Phase 22 by design]

## Notes

- Non-fatal hydrate warning at src/components/bug_report_widget.rs:158 ("reading a
  resource in hydrate mode outside a <Suspense/>"). This is the deliberate tradeoff
  from commit 1eb3f00, which removed the <Suspense> wrapper to eliminate a racy
  hydration *panic*. Stable-with-warning was chosen over panic-with-Suspense.
- Security gate: workflow.security_enforcement is on and no 19-SECURITY.md exists yet.
  Phase 19 has a defined threat model (T-19-01 category injection, T-19-02 empty
  description, T-19-03 export-failure isolation, T-19-04 prompt-injection in inbox,
  T-19-05 list_bug_reports access gating) with mitigations described and partially
  verified in the SUMMARYs. Run /gsd-secure-phase 19 to verify mitigations before
  formally advancing to Phase 20.
