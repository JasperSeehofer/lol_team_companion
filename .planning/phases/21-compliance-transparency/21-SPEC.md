# Phase 21 — Compliance & Transparency (SPEC seed)

**Status:** SEED — produced by the v1.2 → v1.3 pivot on 2026-05-06. Run `/gsd-spec-phase 21` to expand. Consult vault on every decision (`/consult legal "..."`).

**Milestone:** v1.3 Launch Readiness

## Goal (one sentence)

All EU-compliance artifacts in place before any external user touches the deployed app — Impressum, Datenschutzerklärung, vault Tier-A section, Riot API approval status, and a CI sweep for the project's hard-NO guardrails.

## Why this phase exists

`[[guardrails#G-13]]` is a hard NO: shipped production features must have a Tier-A transparency section in the wiki entity page. The current `wiki/entities/lol-team-companion.md` has none. A closed beta with real users is a "shipped production feature" under this guardrail. Phase 21 closes the gap.

## In-scope

### Domain & TLD
1. **Pick TLD**. Default candidate `lol-companion.gg` — `.gg` is Bailiwick of Guernsey, NOT EU. The `[[values-charter#2]]` "EU tools preferred" value applies; document the justification (e.g. ".gg has strong gaming-community recognition that increases discoverability for the target audience"). If user prefers EU, `.eu` / `.de` / `.io` (UK, gray-area) alternatives.
2. **Register the domain** through an EU registrar where possible (e.g. Hetzner DNS, INWX, Mittwald).

### Legal pages (`src/pages/legal/`)
3. **Impressum** at `/legal/impressum`:
   - Cite **§5 DDG** (NOT §5 TMG — repealed 2024-05-14 per `[[guardrails#G-03]]`)
   - Real address (per `[[cross-project-incidents]]` feynman 2026-04-16: do NOT ship placeholder address; CI must check)
   - **No Steuernummer line** (per `[[guardrails#G-04]]` — without USt-IdNr, the line is omitted entirely)
   - Contact email
   - V.i.S.d.P. for any user-generated content sections
4. **Datenschutzerklärung** at `/legal/datenschutz` — 4 sections per `[[cross-project-memory]]` "German Kleinunternehmer Impressum" pattern:
   - **Server-Logfiles** → Art. 6 Abs. 1 lit. f (legitimate interest); retention period must match actual log rotation window
   - **Registration / user accounts** → Art. 6 Abs. 1 lit. b (contract performance)
   - **Hosting** → Hetzner named subprocessor with AVV reference (link to Hetzner AVV page)
   - **Betroffenenrechte** → LfDI Baden-Württemberg link
   - **Plus** a Tier-A subsection for: Riot API (puuid → US-headquartered processor — Art. 6 Abs. 1 lit. b for contract performance); Bug-report widget (free-text user input — Art. 6 Abs. 1 lit. b for support response, lit. a if optional improvement feedback)

### Vault Tier-A
5. **Update `wiki/entities/lol-team-companion.md`** with a full Tier-A transparency section per `[[transparency]]` template: data flows, legal basis per flow, retention, third parties, sub-processors. (Done via `/consult legal` and `/wiki-debrief`.)
6. Refresh entity page metadata (last updated 2026-04-10) — note v1.3 launch milestone, current architecture state.

### CI guardrails sweep
7. **GitHub Actions job** `guardrails-sweep` that fails on any of:
   - HTML output containing `fonts.googleapis.com` or `fonts.gstatic.com` (G-01 hard NO)
   - HTML or markdown containing the literal `§5 TMG` or "TMG" as a legal cite (G-03)
   - `Musterstraße`, `Musterstadt`, `[ADDR]`, or other placeholder strings in legal pages (post-feynman incident)
   - `outline:none` / `outline: 0` in CSS without a sibling `ring-*` token (G-12)
   - Raw hex colors in `src/components/` (per project Code Style)
   - Any `Steuernummer:` line in the rendered Impressum (G-04)
8. Run sweep on every push and PR.

### Riot Developer Portal
9. **Confirm Riot Developer Portal application status**. If not approved, submit a Personal API Key application. Document status in `21-RIOT-STATUS.md` for the phase folder.
10. If charging for the app, "prior written approval" is also required — but v1.3 is closed beta with no monetization, so unlikely needed in this milestone.

## Out of scope

- Cookie banner (no third-party tracking; only first-party session cookie which is necessary for auth — exempt from consent under TTDSG § 25 (2) Nr. 2). Document the assessment in the DSE.
- DSE for non-implemented features (only document what actually ships)
- Multi-language legal pages (German + English; translate post-launch if needed)

## Success criteria (verify with `/gsd-verify-work 21`)

1. Domain registered and DNS pointing at CAX11
2. `/legal/impressum` and `/legal/datenschutz` routes live with all required clauses
3. Vault `wiki/entities/lol-team-companion.md` Tier-A section exists with data flows, legal basis, retention, third parties — verified by `/consult legal "audit lol-team-companion Tier-A section"`
4. CI guardrails sweep job exists and passes on a clean main; intentionally-broken test branches fail it
5. Riot Developer Portal application status documented in `21-RIOT-STATUS.md`
6. No `§5 TMG` anywhere in the deployed HTML or markdown
7. No Steuernummer line in the Impressum

## Required reading before discuss-phase

1. `[[guardrails]]` (full file)
2. `[[values-charter]]`
3. `[[transparency]]`
4. `[[cross-project-memory]]` — German Kleinunternehmer Impressum pattern
5. `[[cross-project-incidents]]` — feynman 2026-04-16 (placeholder Impressum incident)
6. `wiki/entities/feynman-lookup.md` (existing example of Tier-A section to mirror)

## Plans

TBD — produced by `/gsd-plan-phase 21`. Likely 3 plans:
- 21-01: Legal pages (Impressum + DSE) + routes + tests for guardrail strings
- 21-02: Vault Tier-A update + Riot Developer Portal application
- 21-03: CI guardrails sweep

## Depends on

- Phase 16 only (no dependency on Phase 19 / 20 at the spec level — can proceed in parallel; only the DNS step in Phase 20 depends on Phase 21 having registered the domain)

---

This SPEC was seeded by the pivot.
