# Phase 9: Draft & Game Plan - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 09-draft-game-plan
**Areas discussed:** Opponent Selection, Intel Panel UX, Pool Notes Sidebar, Game Plan Autofill

---

## Opponent Selection

| Option | Description | Selected |
|--------|-------------|----------|
| Replace header input | Replace free-text 'Opponent' input with scouted opponents dropdown + 'Add New' button | ✓ |
| Keep both | Keep free-text header AND dropdown on intel tab | |
| You decide | Claude picks | |

**User's choice:** Replace header input
**Notes:** Single source of truth for opponent selection

---

| Option | Description | Selected |
|--------|-------------|----------|
| Navigate to opponents page | Auto-save draft, navigate to /opponents with return link | ✓ |
| Inline mini-form | Modal/popover on draft page for quick opponent creation | |
| You decide | Claude picks | |

**User's choice:** Navigate to opponents page
**Notes:** Matches DRFT-02 requirement

---

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, auto-open intel | When opponent selected, intel sidebar opens automatically | ✓ |
| No, keep manual | Intel stays closed unless user clicks button | |

**User's choice:** Yes, auto-open intel

---

| Option | Description | Selected |
|--------|-------------|----------|
| Store opponent ID | Link to opponent record for cross-feature queries | ✓ |
| Keep as name string | Store display name (simpler but fragile) | |
| You decide | Claude picks | |

**User's choice:** Store opponent ID
**Notes:** Enables richer cross-feature queries like "all drafts vs this opponent"

---

| Option | Description | Selected |
|--------|-------------|----------|
| Plain dropdown | Simple `<select>` element | |
| Searchable dropdown | Type-ahead filter like ChampionAutocomplete | ✓ |
| You decide | Claude picks | |

**User's choice:** Searchable dropdown
**Notes:** User prefers type-ahead even for small lists

---

| Option | Description | Selected |
|--------|-------------|----------|
| Separate button next to dropdown | Always visible '+' or 'Add' button beside dropdown | ✓ |
| Inside dropdown results | 'Add new opponent...' entry at bottom of list | |
| You decide | Claude picks | |

**User's choice:** Separate button next to dropdown

---

## Intel Panel UX

| Option | Description | Selected |
|--------|-------------|----------|
| Matchup picker only | Filter bans from matchup selection only | ✓ |
| Both tabs | Filter bans from matchup AND Our Pools click-to-pick | |
| You decide | Claude picks | |

**User's choice:** Matchup picker only

---

| Option | Description | Selected |
|--------|-------------|----------|
| It exists already | Champion detail view exists, needs back button | ✓ |
| Needs to be created | No detail view yet | |
| You decide | Claude investigates | |

**User's choice:** It exists already

---

| Option | Description | Selected |
|--------|-------------|----------|
| Remove it | Remove duplicate opponent dropdown from intel sidebar | ✓ |
| Keep as read-only | Show selected opponent name as context label | |
| You decide | Claude picks | |

**User's choice:** Remove it
**Notes:** Header dropdown is single source of truth

---

## Pool Notes Sidebar

| Option | Description | Selected |
|--------|-------------|----------|
| New tab in intel sidebar | 4th "Notes" tab, reuses existing sidebar infrastructure | ✓ |
| Inline below draft board | Collapsible panel below board | |
| Popover on champion | Click picked champion for popover | |
| You decide | Claude picks | |

**User's choice:** New tab in intel sidebar

---

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-switch on pick | Intel opens to Notes tab when pooled champion picked | ✓ |
| Manual only | User must click Notes tab | |
| Auto-switch + stay | Auto-switch once, then respect manual tab choice | |

**User's choice:** Auto-switch on pick

---

| Option | Description | Selected |
|--------|-------------|----------|
| All note types | Show all note types grouped and collapsible | ✓ |
| Matchup + synergy only | Focus on draft-relevant notes | |
| You decide | Claude picks | |

**User's choice:** All note types, but make the sections collapsible
**Notes:** User specifically requested collapsible sections per note type

---

| Option | Description | Selected |
|--------|-------------|----------|
| All our picks | Section per our-side champion with notes | |
| Highlighted only | Notes for currently highlighted slot's champion | |
| Champion sub-tabs | Second row of tabs within Notes, one per champion | ✓ |

**User's choice:** Nested champion sub-tabs within the Notes tab
**Notes:** User wanted option 1 (all picks) but with sub-tab navigation per champion rather than a scrollable list

---

| Option | Description | Selected |
|--------|-------------|----------|
| Show with empty state | Sub-tab appears with "No notes" message | ✓ |
| Hide tab | Only show sub-tabs for champions with notes | |
| You decide | Claude picks | |

**User's choice:** Show with empty state
**Notes:** Makes it obvious which champions lack notes

---

## Game Plan Autofill

| Option | Description | Selected |
|--------|-------------|----------|
| First tag | Use first composition tag from draft | |
| Most common tag | Pick tag with highest count, break ties with position | ✓ |
| All tags as comma string | Concatenate all tags | |
| You decide | Claude picks | |

**User's choice:** Most common tag

---

| Option | Description | Selected |
|--------|-------------|----------|
| Pre-filled but editable | Sets name to '<draft name> plan', user can change | ✓ |
| Locked | Force name to match draft | |

**User's choice:** Pre-filled but editable

---

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, map roles | Use DraftAction.role to pre-fill per-role strategy fields with champion header | ✓ |
| Champions only | Just fill champion lists, leave role fields empty | |
| You decide | Claude picks | |

**User's choice:** Yes, map roles
**Notes:** Pre-fill per-role strategy field with "ChampionName: " as a prompt header

---

## Claude's Discretion

- Searchable dropdown implementation approach (reuse ChampionAutocomplete pattern or build new)
- Tie-breaking for "most common" tag
- Collapsible section animation/toggle behavior
- Whether to show champion portraits in Notes sub-tabs
- Auto-save mechanism before navigating to opponents page

## Deferred Ideas

None — discussion stayed within phase scope
