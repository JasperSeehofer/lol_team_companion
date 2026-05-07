# Phase 17 — Open-Design Surface Map

> Tracks which utility-tier surfaces are generated as Open-Design HTML
> prototypes against the `lol-companion` design system, and where the
> generated HTML lives in the Open-Design repo.
>
> **Per D-23:** Open-Design HTML never round-trips into this repo. Only
> the resulting Leptos components are tracked here. This map is the
> bridge — it tells utility-tier execution plans (03d / 04 / 05 / 06)
> which OD project to consult when porting to Leptos.

## How to use this map

When implementing a utility-tier surface in plan 03d / 04 / 05 / 06:

1. **Generate the OD prototype** against the `lol-companion` design
   system (see "How to generate" below). Capture the project UUID and
   the artifact path under `.od/projects/{uuid}/`.
2. **Update this table** — fill in the `OD UUID` and `OD HTML path`
   columns, change `Status` from `pending` to `generated`.
3. **Port to Leptos** — read the generated HTML, extract structure +
   class strings, write the matching `#[component]` in `src/pages/...`
   or `src/components/...`. Preserve the focus-ring, ARIA, and token
   discipline encoded in the prototype.
4. **No copies in this repo** — the OD HTML stays in
   `/home/jasper/Repositories/open-design/.od/projects/{uuid}/`. Only
   the Leptos port and a reference to the UUID get committed here.

When all utility surfaces in a wave are `generated`, the wave's
implementation plan can begin. Once a surface is implemented in
Leptos, set `Status = ported`.

## Surfaces

| Surface | Route | Plan | OD UUID | OD HTML path | Status |
|---------|-------|------|---------|--------------|--------|
| Login | `/auth/login` | 06 | TBD | TBD | pending |
| Register (invited) | `/auth/register?invite=...` | 06 | TBD | TBD | pending |
| Team roster | `/team/roster` | 05 | TBD | TBD | pending |
| Team builder | `/team-builder` | 05 | TBD | TBD | pending |
| Opponents | `/opponents` | 03 | TBD | TBD | pending |
| Action items | `/action-items` | 03 | TBD | TBD | pending |
| Personal learnings | `/personal-learnings` | 04 | TBD | TBD | pending |
| Analytics | `/analytics` | 04 | TBD | TBD | pending |
| Admin invites | `/admin/invites` | 06 | TBD | TBD | pending |
| Bug-report widget | (floating, mounted in auth shell) | 06 | TBD | TBD | pending |
| Closed-beta acceptance form | `/auth/register?invite=...` (sub-form) | 06 | TBD | TBD | pending |

**Status legend:**

- `pending` — surface listed; no OD project generated yet.
- `generated` — OD HTML exists at `.od/projects/{uuid}/{name}.html`;
  Leptos port not yet written.
- `ported` — Leptos `#[component]` lives in
  `src/pages/...` or `src/components/...`; surface ships.

## How to generate an OD prototype against `lol-companion`

The `lol-companion` design system is seeded at
`/home/jasper/Repositories/open-design/design-systems/lol-companion/`
(`DESIGN.md` + `tokens.css`).

### Conventions inherited from Open-Design

- **Workspace shape.** Open-Design organises every OD project under
  `.od/projects/{uuid}/` (see `AGENTS.md` "Where is data written?" FAQ).
  An app config picks the active design system via
  `.od/app-config.json` (`designSystemId`). Switch the active system
  to `"lol-companion"` before generating.
- **Project file layout.** Each `.od/projects/{uuid}/` directory holds
  the JSX/HTML artifacts the daemon emits. Reference shape from the
  existing project at
  `/home/jasper/Repositories/open-design/.od/projects/4335183a-273c-4f5e-bf37-8724afafe551/`:
  `app.jsx`, `components.jsx`, `data.jsx`, `index.html`, plus per-screen
  `.html` artifacts (`{name}.html` + `{name}.html.artifact.json`).
- **Generated HTML is the deliverable.** Each surface produces one
  `.html` artifact under `.od/projects/{uuid}/`. The companion
  `.artifact.json` records prompt / agent metadata.
- **Tokens.** Prototypes can either `@import "../../design-systems/lol-companion/tokens.css"`
  or paste the values inline. Either way the `data-theme="demacia"`
  attribute on `<html>` activates the gold palette by default.

### Step-by-step

1. **Activate the design system.** In the OD app
   (`.od/app-config.json`), set `designSystemId` to `lol-companion`.
   Or pass `--design-system lol-companion` to the CLI/MCP entry point
   if invoking headlessly.
2. **Open or create the project** for Phase 17 utility surfaces. A
   single OD project can host many `.html` artifacts (one per surface).
   Reuse the existing `4335183a-...` project, or create a new one
   reserved for Phase 17 utility-tier work — record the chosen UUID in
   the table below.
3. **Run the prototype task.** Provide the agent the surface brief
   (route, copy, anatomy from `17-UI-SPEC.md` + `DESIGN.md`). The
   agent emits `{surface-slug}.html` and `{surface-slug}.html.artifact.json`
   under `.od/projects/{uuid}/`.
4. **Verify the prototype.** Open the generated HTML in a browser
   inside the OD daemon (or just `xdg-open` it). Confirm:
   - `data-theme="demacia"` (or `pandemonium`) applied on the page.
   - All semantic tokens render — no raw hex outside theme blocks.
   - Every interactive element has a visible `focus-visible:ring`.
   - Touch targets reach 44×44px.
5. **Update this table.** Fill in `OD UUID` and `OD HTML path`, set
   `Status = generated`. Commit only this `17-OD-MAP.md` change to
   the lol_team_companion repo (the OD HTML lives in the OD repo's
   own git lifecycle).
6. **Port to Leptos.** When the consuming plan (03 / 04 / 05 / 06)
   reaches the surface, the executor reads the generated HTML and
   produces the matching `#[component]`. Update `Status = ported` on
   merge.

### Brief checklist for prototype generation

When asking the OD agent to generate a surface, include:

- Route + auth requirement (from `17-UI-SPEC.md` Route Inventory).
- Anatomy (form fields, table columns, button labels) from
  `17-UI-SPEC.md` per-surface section.
- Copy from `17-UI-SPEC.md` Copywriting Contract.
- Theme to render (default to Demacia; render Pandemonium variant only
  if surface includes a visible toggle).
- The `lol-companion` design system as the binding token source —
  remind the agent: "no raw hex outside theme blocks, every focusable
  element gets `focus-visible:ring-2 focus-visible:ring-accent/50
  focus-visible:outline-none`."

## Cross-references

- Design system seed:
  `/home/jasper/Repositories/open-design/design-systems/lol-companion/DESIGN.md`
- Token reference:
  `/home/jasper/Repositories/open-design/design-systems/lol-companion/tokens.css`
- Surface anatomies / copy: `.planning/phases/17-ui-consolidation/17-UI-SPEC.md`
- Decision rationale (D-13, D-21, D-22, D-23): `.planning/phases/17-ui-consolidation/17-CONTEXT.md`
