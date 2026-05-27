# Phase 19 → Phase 22 Handoff: Bug-Report Widget Captured-Field Inventory

**Source:** Phase 19 D-09.1 (CONTEXT.md decision). Phase 19 ships the technical capture; Phase 22 (Compliance & Transparency) ingests this inventory verbatim into the Tier-A transparency table.

**Status:** Authoritative as of Phase 19 ship date. If new fields are added in later phases, update this file and notify Phase 22.

## Fields Captured by the Bug-Report Widget

| Field | Type | Source | Purpose | Tier-A note |
|-------|------|--------|---------|-------------|
| page_url | string | window.location.pathname | Locate the surface of the report | Disclose: collected for triage |
| element_label | string | data-feedback-label (closest ancestor) | Semantic anchor | Disclose: collected for triage |
| description | string | user textarea | Free-text report body | Disclose: user-authored content; treat as personal data if it contains identifying info |
| category | enum bug/wishlist | radio | Triage routing | Disclose: collected for triage |
| user_id | record(user) | AuthSession | Attribution + audit | Disclose: pseudonymous account ID |
| created_at | datetime | server time::now() | Recency | Disclose: collected for triage |
| viewport_w / viewport_h | int (optional) | window.innerWidth/Height | Responsive-bug context | Disclose: device-fingerprinting risk negligible at width-only resolution |

## Storage and Retention

Reports persist in SurrealDB (SurrealKV file-backed, default `./data`). Only reports with `status='open'` are exported to the Claude inbox file; `triaged` and `closed` reports remain in the DB until manually pruned. There is no automated retention policy in v1 — Phase 22 should propose one (e.g. delete `closed` reports older than 90 days) as part of the Tier-A disclosure work.

## Out of Scope (Not Captured in v1)

Phase 19 explicitly does NOT capture: screenshots, CSS selectors, IP address, User-Agent string, geolocation, click coordinates. Phase 22 should reflect this in the Tier-A "what we don't collect" companion section.
