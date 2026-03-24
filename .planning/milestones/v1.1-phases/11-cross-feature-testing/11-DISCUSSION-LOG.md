# Phase 11: Cross-Feature & Testing - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-24
**Phase:** 11-cross-feature-testing
**Areas discussed:** Plan effectiveness metrics, Effectiveness view layout, XFEAT-02 scope check, Test data seeding

---

## Plan Effectiveness Metrics

| Option | Description | Selected |
|--------|-------------|----------|
| Add outcome field | Add win/loss/draw to PostGameLearning. Effectiveness = win rate per plan. | Partial |
| Derive from review sentiment | Score from what_went_well vs improvements ratio. | |
| User-rated effectiveness | 1-5 star rating on reviews. | Partial |

**User's choice:** Combination — basic win/loss tracking (automated via Riot API match detection) + 1-5 star user rating
**Notes:** User wants auto-detection: on review creation, fetch recent Riot API matches, match against draft champions to identify the correct game, auto-fill win/loss. "Fetch result" button as fallback for pre-game reviews.

| Option | Description | Selected |
|--------|-------------|----------|
| 1-5 stars | Familiar, good granularity | ✓ |
| Thumbs up/down | Binary, less nuanced | |
| 1-10 scale | More precision, overkill | |

**User's choice:** 1-5 stars

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-fill on save | Auto-fetch win/loss from Riot API on review save | ✓ |
| Fetch outcome button | Manual "Fetch result" button | Fallback |
| Always manual | User picks every time | |

**User's choice:** Auto-fill on creation + match against draft champions + fetch button fallback for pre-game reviews

| Option | Description | Selected |
|--------|-------------|----------|
| Both (tag + plan) | Strategy tag cards + per-plan breakdown | ✓ |
| Per game plan only | Individual plan stats only | |
| By strategy tag only | Aggregate by tag only | |

**User's choice:** Both — strategy tag aggregation AND individual game plan stats

---

## Effectiveness View Layout

| Option | Description | Selected |
|--------|-------------|----------|
| New /effectiveness page | Dedicated analytics page in nav | ✓ |
| Section on game plan page | Stats at top of existing page | |
| Dashboard panel | Effectiveness panel on dashboard | |

**User's choice:** New dedicated page

| Option | Description | Selected |
|--------|-------------|----------|
| Strategy cards + plan table | Top row cards per tag, table below | ✓ |
| Single table with filters | One filterable table | |
| Chart-first view | Bar/pie charts | |

**User's choice:** Strategy cards + plan table layout

| Option | Description | Selected |
|--------|-------------|----------|
| Expand inline | Accordion row expansion | ✓ |
| Navigate to game plan | Link to /game-plan | |
| Side panel | Slide-out detail panel | |

**User's choice:** Expand inline (accordion)

| Option | Description | Selected |
|--------|-------------|----------|
| No filters | Show all data, keep simple | ✓ |
| Date range filter | Last 7/30/90 days | |
| Strategy tag filter | Click card to filter table | |

**User's choice:** No filters for v1.1

| Option | Description | Selected |
|--------|-------------|----------|
| Effectiveness | Matches feature name | |
| Analytics | Broader label | ✓ |
| Plan Stats | Casual/concrete | |

**User's choice:** Analytics

---

## XFEAT-02 Scope Check

| Option | Description | Selected |
|--------|-------------|----------|
| Already done — mark complete | Phase 9 built exactly this (DRFT-05) | ✓ |
| Different location needed | Notes should appear elsewhere too | |
| Enhanced version needed | Sidebar needs improvements | |

**User's choice:** Already done — DRFT-05 satisfies XFEAT-02. Exclude from Phase 11.

---

## Test Data Seeding

| Option | Description | Selected |
|--------|-------------|----------|
| Real accounts | Real Riot IDs, actual data | ✓ |
| Fabricated data | Fake summoners, mock PUUIDs | |
| Mix | Real accounts, seeded app data | |

**User's choice:** Real accounts

| Option | Description | Selected |
|--------|-------------|----------|
| Rust binary + clean slate | Idempotent Rust seed script | ✓ |
| SurQL seed file | Static .surql INSERT statements | |
| Server endpoint | /api/seed dev-only endpoint | |

**User's choice:** Rust binary + clean slate

| Option | Description | Selected |
|--------|-------------|----------|
| Champion pools with tiers + notes | Full pool data per player | ✓ |
| Drafts + game plans | 2-3 drafts with linked plans | ✓ |
| Post-game reviews with outcomes | Reviews with win/loss + ratings | ✓ |
| Opponent scouting profiles | 1-2 opponent teams with 5-role data | ✓ |

**User's choice:** All four — full data population

---

## Claude's Discretion

- Star rating UI component implementation
- Strategy card visual design
- Accordion animation for plan row expansion
- Seed script CLI interface
- Riot API rate limiting strategy during seed
- Schema migration approach for new PostGameLearning fields

## Deferred Ideas

- Date range / time filters on analytics page
- Chart visualizations (bar charts, trend lines)
- Click strategy card to filter table by tag
