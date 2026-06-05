# Phase 27: War Room - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-05
**Phase:** 27-war-room
**Areas discussed:** Surface extraction depth, Ban Priority surface, Tab-switch state behavior, Growth feedback visibility

---

## Surface extraction depth

| Option | Description | Selected |
|--------|-------------|----------|
| Room-only surface wrappers | Wrap each feature's content with chrome suppressed; leave flat *Page routes untouched; Phase 28 converges them. | |
| Full shared extraction now | Extract every feature into a shared *Surface and refactor the flat routes to render it too. Single source of truth now; bigger blast radius. | ✓ |
| Embed pages as-is | Render existing *Page components unchanged inside each tab; accept duplicated chrome. | |

**User's choice:** Full shared extraction now.
**Notes:** Pre-pays the Phase 28 IA-healing convergence. Flat-route behavior must be preserved (regression check); `draft.rs` (~3000 lines) is the main risk surface. → D-01, D-02, D-03.

---

## Ban Priority surface

| Option | Description | Selected |
|--------|-------------|----------|
| Extract the full editor | Pull draft.rs's ban-priority panel (ranked list + add/edit/save) into its own surface; tab owns the persistent list. | ✓ |
| Read-only priority view | Show the saved ranked list; editing stays in the Draft tab only. | |

**User's choice:** Extract the full editor.
**Notes:** The strategic ban-priority list (`ban_priority` table, get/save_ban_priorities) is a separate concept from the in-draft `banned_champions` derived from live board slots — the latter stays with the Draft surface. → D-04, D-05.

---

## Tab-switch state behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Lazy <Show> (unmount inactive) | Only the active surface mounts/fetches; transient unsaved edits reset on switch. Matches RoomSurface as built. | |
| Keep panels mounted (hidden) | All 5 surfaces mount on entry and stay (CSS-hidden); state persists; requires changing the Phase-26 RoomSurface component. | ✓ |

**User's choice:** Keep panels mounted (hidden).
**Notes:** Requires reworking `RoomSurface` from `<Show>`-per-panel to keep-mounted CSS-hidden panels, and a new children contract so each surface mounts independently. War Room is RoomSurface's first real consumer (Solo Journey has no tabs) → low regression risk; dummy-surface test must be updated. All 5 Resources fetch eagerly on entry — accepted. → D-06, D-07, D-08, D-09.

---

## Growth feedback visibility

| Option | Description | Selected |
|--------|-------------|----------|
| Art + subtle tier readout | Overlays fade in by tier AND a small per-axis tier badge/label. | |
| Environmental art only | Growth shown purely through atmosphere overlays; no numbers/labels. | ✓ |
| Explicit growth panel | Dedicated readout of all three axes, counts, next-tier thresholds. | |

**User's choice:** Environmental art only.
**Notes:** Keeps the room diegetic. Because Phase 27 ships placeholder gradient art, Success Criterion 3 is verified via the `data-tactics-tier` / `data-intel-tier` / `data-command-tier` attributes and overlay opacity (seed Tier-2 vs fresh team), not visible text. → D-10, D-11.

---

## Claude's Discretion

- Default tab = Draft (first); `?tab=` unknown/missing → first tab (existing RoomSurface behavior).
- Tab order fixed by roadmap: Draft, Tree-Drafter, Game Plan, Opponents, Ban Priority.
- Asset paths mirror `/img/rooms/solo-journey/{region}/` → `/img/rooms/war-room/{region}/`; placeholder gradient WebPs.
- Plane/overlay count + parallax depths (mirror solo_journey.rs).
- `*Surface` module layout; `get_team_growth_state` placement + no-team default (`TeamGrowthState::default()`).
- `mousemove`-guard mechanism (static AtomicBool per Leptos #3665).

## Deferred Ideas

- IA-healing redirects + nav changes — ROOM-07, Phase 28.
- QuickNav + room sub-nav suppression — ROOM-08, Phase 29.
- Real FLUX art — Phase 30.
- Eager-fetch optimization for mounted surfaces — only if it proves a problem.
- In-app "Pause atmosphere" toggle — later a11y/settings phase.
- Explicit growth readout / tier badges — rejected for Phase 27 (D-10), revisitable if placeholder-art growth proves imperceptible.
