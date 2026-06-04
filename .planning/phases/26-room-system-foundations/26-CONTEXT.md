# Phase 26: Room System Foundations - Context

**Gathered:** 2026-06-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 26 delivers the data model, component contracts, and documented rules that all three new rooms (War Room, Library, Throne) compile against. **No new visible UI surfaces** — this is the gate phase. Scope:

- **ROOM-01** — Team-scoped growth state: `TeamGrowthCounts` + `get_team_growth_counts` (the shared raw query) + War Room axis-tier derivation, unit-tested.
- **ROOM-02** — `AtmosphereRoot` generalized to arbitrary named growth axes, AND a reusable `RoomSurface` tab component (built as a standalone contract here — see D-01).
- **ROOM-03** — Anti-structural-branching hydration rule codified in `CLAUDE.md` + `.claude/rules/leptos-patterns.md` before any room is built.
- **A11Y-01** — Runtime `prefers-reduced-motion` `MediaQueryList` change-listener in `AtmosphereRoot` (freeze/restore parallax without page reload).

Research is complete (`.planning/research/SUMMARY.md`) — research-phase is skipped for this phase.

</domain>

<decisions>
## Implementation Decisions

### RoomSurface tab component boundary (ROOM-02)
- **D-01:** Build `RoomSurface` as a **standalone reusable component in Phase 26**, satisfying ROOM-02 in full here. Contract: `RwSignal<ActiveSurface>` (or generic equivalent) + `<Show>` per tab; tab bar uses `role="tablist"` / `role="tab"` / `aria-selected` / `aria-controls`; deep-linkable via `?tab=` URL sync. Ship with a minimal unit/render test using **dummy surfaces** (no real feature content). War Room (Phase 27) is its first real consumer.
- **D-02:** Intra-room navigation is signal + `<Show>` only — **never sub-routes** (sub-routes remount `AtmosphereRoot` and destroy parallax continuity). Carried forward from Roadmap v1.4 decision; reaffirmed here as a `RoomSurface` constraint.

### Team growth model shape (ROOM-01)
- **D-03:** Phase 26 ships raw `TeamGrowthCounts` + `get_team_growth_counts` (the shared batched round-trip) **plus the War Room axis-tier derivation as the proven template.** Library and Throne axis derivations are deferred to Phases 28/29 — Phase 26 does NOT pre-derive all nine axes (those mappings/thresholds should calibrate against real data, per research).
- **D-04:** `get_team_growth_counts` queries the **directly team-scoped tables only**: `draft`, `draft_tree`, `opponent`, `ban_priority`, `post_game_learning`, `team_note` (all carry a `team: record<team>` field — confirmed against `schema.surql`). Mirror the existing user-scoped `get_growth_counts` query shape in `db.rs:4109` (batched `SELECT count() ... GROUP ALL` statements, `type::record('team', $team_key)`, strip `team:` prefix).
- **D-05:** `player_match` is **dropped from Phase 26**. It is user-scoped (no `team` field) and would require a `team_member → user → player_match` join that no Phase 26 derivation reads. Defer until a Library/Throne axis actually needs team match volume. Rule: raw counts = directly-team-scoped tables; join-requiring tables deferred until consumed.

### War Room axis → data mapping (the derivation Phase 26 owns)
- **D-06:** `tactics_tier` ← count of `draft` + `draft_tree` (planning depth across linear + tree drafts).
- **D-07:** `intel_tier` ← count of `opponent` (scouting maturity).
- **D-08:** `command_tier` ← count of `ban_priority` + `post_game_learning` (ban strategy + accumulated command experience).
- **D-09:** Tier thresholds are calibration hypotheses (mirror `growth.rs` `tier_for` with `[t1, t2, t3]` lower bounds). Pick reasonable starting values; flag for beta calibration. NOT a blocking decision.

### AtmosphereRoot axis generalization (ROOM-02)
- **D-10:** Generalize the current fixed `lib_tier` / `armory_tier` / `banner_tier` props (`parallax.rs:32-35`) so all four rooms' axis names work. Implement as `#[prop(optional, into)]` `Signal<u8>` fields with `Signal::derive(|| 0u8)` defaults; emit `data-{axis}-tier` attributes. Solo Journey (`lib`/`armory`/`banner`) must keep working unchanged — include a Solo Journey regression check. Axis names per room are fixed in research SUMMARY.md §"AtmosphereRoot axis names per room".

### Reduced-motion runtime listener (A11Y-01)
- **D-11:** Add a runtime `MediaQueryList.addEventListener('change', ...)` in the `#[cfg(feature = "hydrate")]` block of `AtmosphereRoot` (current code only checks `match_media` at mount, `parallax.rs:61`). Store the MQL/closure handle in a `StoredValue` for cleanup, following the existing `on_cleanup` pattern. An in-app "Pause atmosphere" UI toggle is **out of scope** (deferred — see below).

### Anti-structural-branching rule (ROOM-03)
- **D-12:** Document the rule in BOTH `CLAUDE.md` and `.claude/rules/leptos-patterns.md`: region (`InitialTheme`) and growth tier must be read ONCE at the room component top level and passed as plain props (`region: String`, tier signals) — never read inside a reactive closure, `Signal::derive()`, `view!` branch, or child component. Reference the Phase 18.2 incident (tachys `Option::unwrap()` panic at `html/mod.rs:217`) explicitly so an agent reading either file before building a room finds it.

### Claude's Discretion
- Exact tier threshold integers for War Room axes (D-09) — pick sane starting values.
- Internal naming of the `TeamGrowthState` / War-Room-specific derivation struct(s).
- Whether the War Room derivation lives in `models/growth.rs` alongside the user-scoped pair or a sibling module — planner's call, keep it parallel to the existing pattern.
- `?tab=` deep-link fallback behavior for unknown/invalid tab values (default to first surface is the obvious choice).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase research (read first)
- `.planning/research/SUMMARY.md` — full v1.4 architecture, build sequence, axis mapping table, pitfalls. Authoritative for this phase.
- `.planning/research/ARCHITECTURE.md` — extract-then-compose pattern, component contracts.
- `.planning/research/PITFALLS.md` — reduced-motion listener gap, mousemove accumulation, hydration panic, asset budget.

### Existing code to mirror / modify
- `src/models/growth.rs` — `GrowthCounts`, `GrowthState`, `Tier`, `tier_for`, `from_counts`, unit-test suite. Mirror this for the team-scoped pair.
- `src/server/db.rs:4109` (`get_growth_counts`) — exact batched-query shape, `type::record` + prefix-strip pattern to mirror for `get_team_growth_counts`.
- `src/components/room/parallax.rs` — `AtmosphereRoot` props (`:32-35`), mount-time `match_media` check (`:61`), `data-*-tier` emission (`:150-152`), `Plane`/`GrowthOverlay`.
- `src/pages/rooms/solo_journey.rs` — region-read-once pattern (`:18`), Resource fetch, asset path convention; the regression target for axis generalization.

### Rule / doc targets to edit (ROOM-03)
- `CLAUDE.md` — add anti-structural-branching hydration rule.
- `.claude/rules/leptos-patterns.md` — add the same rule with Phase 18.2 reference.

### Schema ground truth
- `schema.surql` — `draft` (`:67`), `draft_tree` (`:152`), `opponent` (`:187`), `ban_priority` (`:179`), `post_game_learning` (`:115`), `team_note` (`:229`) all carry `team: record<team>`. Confirmed during discussion.

### Prior incident
- `.planning/phases/18.2-pandemonium-hydration-reconciliation-gap-closure-for-ui-18-1/18.2-CONTEXT.md` — root cause of the structural-branching panic that ROOM-03 codifies.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `growth.rs` user-scoped pair (`GrowthCounts`/`GrowthState`/`Tier`/`tier_for`): the team-scoped model is a direct structural mirror — same `from_counts` + `tier_for([t1,t2,t3])` approach.
- `get_growth_counts` (db.rs:4109): copy the batched `SELECT count() ... GROUP ALL` + `count_of` helper + `type::record('team', $team_key)` shape; swap `user` → `team` filters and the table set per D-04.
- `AtmosphereRoot` already supports optional tier props via `#[prop(optional, into)] Signal<u8>` — generalization is widening/renaming, not new infrastructure.

### Established Patterns
- SurrealDB: strip `team:` prefix, pass owned `String` to `.bind()`, `type::record('team', $key)`, `take(n).unwrap_or_default()` per statement (`.claude/rules/surreal-patterns.md`).
- WASM: hydrate-only signals need `#[allow(unused_variables)]`; MQL listener + closure must be `#[cfg(feature = "hydrate")]`-guarded and stored in `StoredValue` for cleanup (`.claude/rules/wasm-patterns.md` #37, #42).
- Region read ONCE at top → prop (the very rule ROOM-03 documents); already practiced in `solo_journey.rs:18`.

### Integration Points
- `get_team_growth_counts` is a new `db.rs` function; the War Room `get_team_growth_state` server fn (Phase 27) will call it. Phase 26 provides the counts + War Room derivation; the server fn wiring is Phase 27.
- `AtmosphereRoot` axis generalization must not break the Solo Journey route (`/solo` journey room) — regression check required.

</code_context>

<specifics>
## Specific Ideas

- War Room axis semantics the user explicitly chose (broader than 1:1): tactics spans both draft tools (`draft` + `draft_tree`); command folds post-game learnings in as "command experience" alongside `ban_priority`.
- `RoomSurface` must be genuinely reusable from day one (tested with dummy surfaces), not extracted-after-the-fact — the user wants the contract locked in Phase 26.

</specifics>

<deferred>
## Deferred Ideas

- **In-app "Pause atmosphere" toggle** — a user-facing control to freeze parallax independent of OS settings. Out of scope for Phase 26 (which ships only the required runtime MQL listener). Belongs in a later a11y/settings surface phase. (Research floated it as a "should-have" alongside A11Y-01.)
- **`player_match` team aggregation** — sum match volume across team members (via `team_member → user → player_match` join) for a future Library/Throne growth axis. Add when an axis actually consumes it; not before.
- **Library axis derivation** (`records`/`tomes`/`analysis`) + **Throne axis derivation** (`roster`/`progression`/`crown`) — deferred to Phases 28/29 with their own data→axis mappings and thresholds (calibrate against real data).
- **Mousemove listener accumulation guard** (`AtomicBool` per Leptos #3665) — research assigns this to Phase 27 (first time two rooms coexist in the router), not Phase 26. Noted in STATE.md deferred items.

</deferred>

---

*Phase: 26-room-system-foundations*
*Context gathered: 2026-06-04*
