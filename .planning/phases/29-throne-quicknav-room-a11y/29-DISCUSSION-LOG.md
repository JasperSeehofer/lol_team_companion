# Phase 29: Throne, QuickNav & Room A11Y - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-06
**Phase:** 29-throne-quicknav-room-a11y
**Areas discussed:** Throne surface roster, Throne growth axes (blended), QuickNav: palette vs list

---

## Area selection

| Option | Description | Selected |
|--------|-------------|----------|
| Throne surface roster | Tabs, Team Notebook placement, solo-vs-team behavior | ✓ |
| Throne growth axes (blended) | roster/progression/crown; team + personal blend | ✓ |
| QuickNav: palette vs list | search, room-centric IA, focus trap | ✓ |
| Throne IA-healing scope | redirects; /journey reconciliation | (Claude decided via Phase 28 precedent) |

---

## Throne surface roster

### Q1 — Tab roster
| Option | Description | Selected |
|--------|-------------|----------|
| 3 tabs: Identity / Team / Solo | Identity=profile; Team=full dashboard incl. notebook; Solo=SoloSurfaces | |
| 2 tabs: Team / Solo | Fold identity into Team header | |
| You decide | — | |

**User's choice:** 3 tabs: Identity / Team / Solo (later revised to 5 — see Q3).

### Q2 — Solo/team behavior for no-team users
| Option | Description | Selected |
|--------|-------------|----------|
| Always show all tabs | Team tab shows empty-state CTA for no-team; no conditional hiding | ✓ |
| Adapt tabs to mode/team | Hide/disable Team tab when no team/solo mode | |
| You decide | — | |

**User's choice:** Always show all tabs.

### Q3 — Team Notebook placement
| Option | Description | Selected |
|--------|-------------|----------|
| Inside the Team tab | Keep embedded as today | |
| Its own Notebook tab | Promote to dedicated surface | |

**User's choice (free-text):** "In the end what is the notebook for? The team also wants
to track Learnings and next actions, etc. This can end up in a dedicated tab. There can be
a dashboard, a roster specific tab and a Learnings, Training actions, Development tab. pure
text notes are not worth that much I think."

**Notes:** This reframed the whole team-side structure. Claude reconciled against locked
prior decisions: team learnings already healed into the Library Post-Game surface (Phase 28
D-01) → must NOT be duplicated; Action Items (`/action-items`) already exists as a real
team-scoped assignable/status-tracked to-do feature = the "training/development" substance;
Team Notebook is low-value freeform notes. A genuinely new structured dev-tracker = new
capability → defer.

### Q3-revised — Team-side tab granularity
| Option | Description | Selected |
|--------|-------------|----------|
| 5 tabs: Identity/Dashboard/Roster/Development/Solo | Splits dashboard into 3; Development=Action Items + Notebook, learnings deep-link | ✓ |
| 4 tabs: Identity/Team/Development/Solo | Keep dashboard+roster together; Development separate | |
| 3 tabs: Identity/Team/Solo | Fold everything into one Team tab | |

**User's choice:** 5 tabs: Identity / Dashboard / Roster / Development / Solo.

### Q3-revised-b — Development tab scope
| Option | Description | Selected |
|--------|-------------|----------|
| Compose existing only | Action Items + Notebook + deep-link to Library learnings; no new models | |
| Also note a future dev-tracker | Same for Phase 29, plus record a deferred structured tracker | ✓ |

**User's choice:** Compose existing only for Phase 29 + note a future structured
player-development tracker as deferred.

---

## Throne growth axes (blended)

### Q1 — Axis → data mapping
| Option | Description | Selected |
|--------|-------------|----------|
| roster=members, progression=your LP, crown=team record | Matches SC1's three named things | ✓ |
| Adjust the mapping | — | |
| You decide | — | |

**User's choice:** roster=members, progression=your LP, crown=team record.

### Q2 — Blending behavior (no-team)
| Option | Description | Selected |
|--------|-------------|----------|
| Independent axes (team + personal blend) | No-team still grows progression; no-rank still grows team axes | ✓ |
| Team-scoped like other rooms | No team → all tiers 0 | |

**User's choice:** Independent axes (team + personal blend). First room that doesn't go
fully dark without a team.

### Q3 — "Combined team record" data source
| Option | Description | Selected |
|--------|-------------|----------|
| Post-game win/loss (cheap, team-scoped) | Reuses Library growth source; avoids deferred join | ✓ |
| Per-member ranked wins (full aggregation) | Revive P26-deferred player_match join | |
| You decide | — | |

**User's choice:** Post-game win/loss (cheap, team-scoped).

---

## QuickNav: palette vs list

### Q1 — Add type-to-filter search?
| Option | Description | Selected |
|--------|-------------|----------|
| Yes — filterable palette | Text input + auto-focus; filter destinations by label | ✓ |
| No — curated list + focus trap | Static grouped list, focus trap only | |
| You decide | — | |

**User's choice:** Yes — filterable palette.

### Q2 — Restructure HUB_ROUTES room-centric?
| Option | Description | Selected |
|--------|-------------|----------|
| Room-centric with surface deep-links | 3 rooms + ?tab= deep-links; updates QuickNav + sub-nav | ✓ |
| Add rooms, keep flat entries too | Duplicates destinations | |
| You decide | — | |

**User's choice:** Room-centric with surface deep-links.

---

## Claude's Discretion

- Throne IA-healing redirect set + `/journey` reconciliation (decided via Phase 28 D-08
  precedent: redirect `/profile`, `/team/dashboard`, `/action-items`; leave `/journey`).
- Internal `*Surface` module layout; how `team/dashboard.rs` splits cleanly into three.
- Asset paths, plane/overlay counts, parallax depths (mirror sibling rooms).
- Exact axis count sources + tier thresholds (calibration hypotheses).
- `get_throne_growth_state` placement + blended no-team/no-rank shape.
- QuickNav filter-match algorithm (substring sufficient) + overlay layout.
- A11Y-02 details (canary, focus-trap mechanics, contrast scrim) — spec-locked by SCs.

## Deferred Ideas

- Structured player-development / training tracker (new capability) — its own phase.
- Retire / reconcile `/journey` into the Throne Solo tab.
- Per-member ranked-win team record (revive deferred player_match join).
- Fuzzy / command-action QuickNav matching beyond substring filtering.
- Eager-fetch optimization for the five mounted Throne surfaces.
- Real FLUX environment art for the Throne (Phase 30).
- Conditional/adaptive tab visibility by solo/team mode (rejected this phase).
