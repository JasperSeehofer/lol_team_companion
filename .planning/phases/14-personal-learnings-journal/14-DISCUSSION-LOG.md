# Phase 14: Personal Learnings Journal - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-27
**Phase:** 14-personal-learnings-journal
**Areas discussed:** Entry structure, Tag system, Browse & filter UX, Match-to-learning flow

---

## Entry Structure

### Q1: How structured should a learning entry be?

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal free-text | Title + single free-text body + champion/matchup/tags. Lightweight notebook feel. | |
| Guided sections | Title + structured fields (What happened, What I learned, Next time I will...) + champion/matchup/tags | ✓ |
| You decide | Claude picks | |

**User's choice:** Guided sections
**Notes:** None

### Q2: Should learnings capture match context when created from a match?

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-attach match context | Win/loss + match reference auto-filled from match detail. Shows as context badge. | ✓ |
| Win/loss only | Just capture win or loss toggle. No match reference. | |
| No match context | Purely reflective, no game result data. | |

**User's choice:** Auto-attach match context
**Notes:** None

### Q3: Should the guided section fields be required or optional?

| Option | Description | Selected |
|--------|-------------|----------|
| All optional | Any combination valid. Low friction. | |
| At least one required | Must fill at least one section. | |
| All required | Must fill all three. Ensures thorough reflection. | ✓ |

**User's choice:** All required
**Notes:** None

### Q4: Should learnings be editable after creation?

| Option | Description | Selected |
|--------|-------------|----------|
| Full edit + delete | Standard CRUD. Revisit, update, or delete. | ✓ |
| Append-only | Original locked. Can add follow-up notes. | |
| You decide | Claude picks | |

**User's choice:** Full edit + delete
**Notes:** None

### Q5: Should the title field be required or auto-generated?

| Option | Description | Selected |
|--------|-------------|----------|
| Required manual title | User must type a title. | |
| Auto-generated, editable | Default from champion + matchup + date. User can override. | ✓ |
| No title | Identified by champion/matchup/date only. | |

**User's choice:** Auto-generated, editable
**Notes:** None

### Q6: Is champion required on a learning?

| Option | Description | Selected |
|--------|-------------|----------|
| Champion required | Every learning tied to a champion. | |
| Champion optional | Champion and matchup fields optional. | |

**User's choice:** Other — Learning type selector
**Notes:** User proposed a learning type selector as the first decision: General (no champion), Champion (champion only), or Matchup (champion + opponent). This enables more specific filtering and reuse in Phase 16 (draft Intel sidebar).

---

## Tag System

### Q1: How should tags work on learnings?

| Option | Description | Selected |
|--------|-------------|----------|
| Free-text tags | User types freely. Autocomplete suggests previous tags. | |
| Predefined categories | Fixed set: Laning, Teamfighting, Macro/Rotations, Vision, Trading, Wave Management, Objective Control, Mental/Tilt | ✓ |
| Both: presets + custom | Predefined quick-select chips plus custom tag input. | |

**User's choice:** Predefined categories
**Notes:** None

### Q2: Can users select multiple tags per learning?

| Option | Description | Selected |
|--------|-------------|----------|
| Multiple tags | Multi-select from predefined list. | ✓ |
| Single category | Exactly one category per learning. | |

**User's choice:** Multiple tags
**Notes:** None

---

## Browse & Filter UX

### Q1: How should the learnings list be laid out?

| Option | Description | Selected |
|--------|-------------|----------|
| Card grid | Cards showing title, type badge, champion icon(s), tags, win/loss, preview. | ✓ |
| Compact list | Dense table-like rows. | |
| You decide | Claude picks | |

**User's choice:** Card grid
**Notes:** None

### Q2: How should filtering work?

| Option | Description | Selected |
|--------|-------------|----------|
| Filter bar above cards | Horizontal bar with dropdowns for Type, Champion, Tag. Instant filtering. | ✓ |
| Sidebar filters | Left sidebar with filter sections. | |
| Tag chips + search | Clickable tag chips plus text search bar. | |

**User's choice:** Filter bar above cards
**Notes:** None

### Q3: What should the default sort order be?

| Option | Description | Selected |
|--------|-------------|----------|
| Newest first | Most recent at top. | |
| Grouped by champion | Under champion headers, then by date. | |
| You decide | Claude picks | |

**User's choice:** Other — Both options available via sort control
**Notes:** User wants both "Newest first" and "By champion" as sort options in the filter bar. Default newest first.

### Q4: What should clicking a learning card do?

| Option | Description | Selected |
|--------|-------------|----------|
| Navigate to detail page | Opens /personal-learnings/:id with full content and edit/delete. | |
| Expand inline | Card expands in place with full content and edit/delete buttons. | ✓ |
| You decide | Claude picks | |

**User's choice:** Expand inline
**Notes:** None

---

## Match-to-Learning Flow

### Q1: How much match context should be pre-filled?

| Option | Description | Selected |
|--------|-------------|----------|
| Champion + matchup + result | Pre-fill type as Matchup, champion, opponent, win/loss, match reference. | ✓ |
| Champion + matchup only | No win/loss or match reference auto-attached. | |
| Full context + KDA | Everything plus a match summary card displayed during creation. | |

**User's choice:** Champion + matchup + result
**Notes:** User also requested "Add Learning" buttons on timeline event details — clicking an event marker's detail expands it and shows an "Add Learning" CTA that pre-fills all the above PLUS event timestamp and event name. Event type should auto-map to a tag suggestion (e.g. Dragon kill → Objective Control).

### Q2: Should event-to-learning store timestamp?

| Option | Description | Selected |
|--------|-------------|----------|
| Store timestamp + event name | Learning includes game timestamp and event name for context. | ✓ |
| Just use for tag mapping | Event maps to tag suggestion only. No timestamp stored. | |
| You decide | Claude picks | |

**User's choice:** Store timestamp + event name
**Notes:** None

### Q3: Where should learnings page live in nav?

| Option | Description | Selected |
|--------|-------------|----------|
| Top-level nav link | New "Learnings" link in main nav bar. | ✓ |
| Under solo dashboard | Embedded section within solo dashboard. | |
| You decide | Claude picks | |

**User's choice:** Top-level nav link
**Notes:** None

---

## Claude's Discretion

- Event-type-to-tag mapping logic
- Card grid responsive breakpoints
- Inline expand animation style
- Empty state messaging and CTA
- Sort-by-champion visual grouping presentation

## Deferred Ideas

None — discussion stayed within phase scope
