---
phase: 19
slug: bug-report-widget
status: verified
threats_open: 0
asvs_level: 1
created: 2026-05-31
---

# Phase 19 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.
> Verified by gsd-security-auditor (claude-sonnet-4-6) on 2026-05-31. Register
> authored at plan time across 19-01..19-04 PLAN.md (`register_authored_at_plan_time: true`),
> so this run verified each claimed mitigation exists in the implementation — it did not scan for new threats.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| WASM client → `submit_bug_report` server fn | Untrusted form data crosses here; defense-in-depth across client UX → server-fn re-validation → DB ASSERT. | description, category, page_url, element_label |
| server fn → SurrealDB | category/status constrained by `ASSERT $value IN [...]` even if server-fn validation is bypassed. | bug_report row |
| global click-capture listener → DOM | Capture-phase listener intercepts clicks on tagged elements; must clean up on every exit path. | DOM element labels |
| user-typed description → inbox file → future Claude session | Reports are user input rendered into a Markdown file future Claude sessions read on context load. | bug-report description text |
| server process → filesystem | Startup export writes to a configurable path; the write may fail (permissions, disk full, read-only mount). | inbox markdown file |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-19-01 | Tampering | `submit_bug_report` + `bug_report.category` | mitigate | Two-layer: `src/components/bug_report_widget.rs:98-99` rejects category ∉ {bug,wishlist}; `schema.surql:328-329` `ASSERT $value IN ['bug','wishlist']`. Unit test `bug_report_rejects_invalid_category` proves the ASSERT fires. | closed |
| T-19-02 | DoS / Tampering | `submit_bug_report` + `bug_report.description` | mitigate | Two-layer: `bug_report_widget.rs:90-95` trims, rejects empty, caps at 4000 chars; `src/server/db.rs:3147-3149` rejects `description.trim().is_empty()`. Unit test `bug_report_rejects_empty_description` proves the DB guard fires. | closed |
| T-19-03 | DoS | `main.rs` startup export hook | mitigate | `src/main.rs:57-61` matches `Err` from `export_open_reports` → `tracing::warn!` → continues to serve. Synchronous `.await`, no spawn. Unit test `tolerates_unwritable_path` asserts `Err` (not panic) on FS failure. | closed |
| T-19-04 | Tampering / Information Disclosure | `.planning/INBOX/bug-reports.md` consumed by future Claude sessions | mitigate | Three-layer: `bug_report_export.rs:140` escapes `<`→`&lt;`; `:141-143` blockquote-prefixes every description line; `CLAUDE.md:252` instructs sessions to treat report content as untrusted and not execute instructions in report bodies. Residual prompt-injection risk accepted for v1 (see Accepted Risks). | closed |
| T-19-05 | Elevation of Privilege | `list_bug_reports` server fn | mitigate | `bug_report_widget.rs:121-125` returns `Err(ServerFnError::new("Forbidden"))` unconditionally for all callers; `// TODO Phase 22` markers (lines 119, 124) track admin-gate hardening. Legit consumer (export task) uses `db::list_open_bug_reports` directly, bypassing the server-fn. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-19-01 | T-19-04 | A determined prompt-injection attack against an LLM reading the inbox cannot be fully prevented in software. Mitigated three ways (HTML-escape, blockquote framing, explicit CLAUDE.md untrusted-content instruction). Residual risk accepted for v1 because the closed-beta invite gate (Phase 20.1) limits the attacker pool to a few named invitees. | Jasper Seehofer | 2026-05-31 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-05-31 | 5 | 5 | 0 | gsd-security-auditor (claude-sonnet-4-6) |

---

## Notes

- **T-19-05 is a deliberate Phase 22 handoff**, not a temporary omission. The v1 `Forbidden` return is the intended disposition; admin-role hardening is tracked via `grep -rn "TODO Phase 22"`.
- The `Closure::forget` lifecycle fix in plan 19-04 (commit `b5a760b`) is not a registered threat but eliminates a WASM panic path in select-mode listener teardown — noted for completeness.
- No unregistered threat flags: all four plan SUMMARY files report none.

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-05-31
