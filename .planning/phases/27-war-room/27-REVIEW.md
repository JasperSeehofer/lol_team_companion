---
phase: 27-war-room
reviewed: 2026-06-05T00:00:00Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - e2e/tests/fixtures.ts
  - e2e/tests/pages.spec.ts
  - e2e/tests/war-room.spec.ts
  - src/app.rs
  - src/components/room/mod.rs
  - src/components/room/parallax.rs
  - src/components/room/room_surface.rs
  - src/pages/draft.rs
  - src/pages/game_plan.rs
  - src/pages/opponents.rs
  - src/pages/rooms/mod.rs
  - src/pages/rooms/war_room.rs
  - src/pages/tree_drafter.rs
findings:
  critical: 0
  warning: 5
  info: 4
  total: 9
resolved:
  - WR-01  # parallax listener-loss race — fixed via generation-id single-listener slot
  - WR-02  # Ban Priority double-mount — fixed via DraftSurface show_ban_priority prop
  - IN-02  # MOUSEMOVE_ACTIVE SSR dead-code — removed incidentally with WR-01
remaining:
  - WR-03  # reduced-motion change-listener (out of scope this pass)
  - WR-04  # redundant per-surface auth/fetch on keep-mount
  - WR-05  # stale status-message lifetime
status: partially_resolved
---

# Phase 27: Code Review Report

> **Resolution (2026-06-05):** WR-01 and WR-02 (the two confirmed functional bugs) were
> fixed in-session before phase completion — see the `fix(27): ...` commits. Both compile
> targets pass, 148 unit tests pass, and the full e2e gate re-ran GREEN (6/6 war-room
> SC2–5, 21/21 smoke incl. flat-route D-02 regression + /war-room SC1). WR-03/04/05
> (advisory, lower severity) remain open for a future polish pass or `/gsd:code-review 27 --fix`.

**Reviewed:** 2026-06-05
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Reviewed the War Room refactor delta against `9be8c65`. The mechanical extraction
of `DraftPage`/`TreeDrafterPage`/`GamePlanPage`/`OpponentsPage` into chrome-free
`*Surface` components is clean, both compile targets type-check (`cargo check
--features ssr` and `--features hydrate --target wasm32-unknown-unknown` both
pass), and rule #58 (region read once at room top level, threaded as plain prop)
is correctly observed in `war_room.rs`, `draft.rs`, and `tree_drafter.rs`. The
growth model is well-tested and the server fn correctly returns the all-zero
default for teamless users (rule #44).

No BLOCKER-tier defects found. The notable issues are: (1) a real
listener-loss race in the parallax accumulation guard that can leave the
surviving room with **zero** mousemove listeners — the inverse of the bug the
guard was meant to fix, directly contradicting the SC4 contract; (2) the
`BanPrioritySurface` editor is mounted **twice** in the War Room (embedded in
the Draft tab AND as its own tab), producing duplicated UI, duplicated network
fetches, and a stale-view data-consistency hazard; plus several lower-severity
robustness gaps in the reduced-motion change-listener path.

## Warnings

### WR-01: Parallax accumulation guard can strand the surviving room with zero mousemove listeners

**File:** `src/components/room/parallax.rs:214-227, 229-233`
**Issue:** The guard is `compare_exchange(false → true)`; on failure the new
mount's `Effect` returns early and never attaches a listener. The
in-code comment itself acknowledges the race: "during a War Room → Solo Journey
→ War Room round-trip where on_cleanup has not yet fired." Consider the exact
ordering the comment describes — new mount runs *before* old cleanup:

1. New `AtmosphereRoot` `Effect` runs. `MOUSEMOVE_ACTIVE` is still `true` (old
   instance holds it) → `compare_exchange` fails → Effect returns, **no listener
   attached**.
2. Old `AtmosphereRoot` `on_cleanup` then fires: sets `MOUSEMOVE_ACTIVE = false`
   AND removes the old instance's listener.

End state: `MOUSEMOVE_ACTIVE == false`, **zero** mousemove listeners attached.
The new Effect has no reactive dependencies, so it never re-runs — parallax is
dead until the next full navigation. This is the inverse of the doubling bug the
guard targets, and it directly breaks the SC4 e2e contract
(`war-room.spec.ts:179` asserts `count === 1`; this path yields `0`).

Whether this fires depends on Leptos 0.8 mount/cleanup ordering during route
transitions, which the comment concedes is not guaranteed cleanup-first. Treat
as a latent correctness bug, not a style nit.
**Fix:** Don't make registration a one-shot gated on a value that a *later*
cleanup will flip. Either (a) have `on_cleanup` re-trigger registration on the
surviving instance, or (b) drop the boolean guard in favor of removing the
listener before re-adding it (idempotent attach), or (c) gate the `Effect` on a
reactive signal so it re-runs after the guard clears. Minimal robust version:

```rust
// In the Effect, after on_cleanup of the prior instance has run, ensure we hold
// the slot. Simplest: always remove-then-add so a second attach is a no-op,
// and drop MOUSEMOVE_ACTIVE entirely (a single id-targeted listener writing the
// same two CSS vars is idempotent — doubled amplitude only occurs if TWO
// distinct closures run; removing before adding guarantees one).
listener.with_value(|slot| {
    if let Some(cb) = slot.as_ref() {
        let _ = win.remove_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref());
        let _ = win.add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref());
    }
});
```

### WR-02: `BanPrioritySurface` is mounted twice in the War Room (embedded in Draft tab + standalone tab)

**File:** `src/pages/rooms/war_room.rs:140-165`, `src/pages/draft.rs:3755-3756`
**Issue:** `DraftSurface` still embeds `<BanPrioritySurface />` (draft.rs:3756),
and the War Room *also* adds a dedicated `ban-priority` tab rendering a second
`<BanPrioritySurface />` (war_room.rs:164). Because `RoomSurface` is keep-mounted
(panels are not unmounted, only `hidden`), **both instances mount simultaneously**
when the War Room loads. Consequences:

- Two independent `get_ban_priorities()` resource fetches fire on War Room entry.
- Two independent auth-redirect `Effect`s run (draft.rs:935).
- The two editors hold separate `ban_edit_list`/`editing_bans` signals: saving in
  one does NOT update the other's view until that other instance's resource is
  refetched (it won't be) — a stale-data hazard the user will hit by editing in
  the Draft tab and then opening the Ban Priority tab (or vice versa).
- The "5 tabs" SC2 contract passes, but the Draft tab visibly contains a
  redundant Ban Priorities panel.

**Fix:** Decide on a single home for the ban-priority editor in the War Room
context. Either remove the embedded `<BanPrioritySurface />` from `DraftSurface`
when it is composed into the War Room (e.g. gate it behind a `show_ban_priority:
bool` prop defaulting to `true` for the flat `/draft` route and `false` for the
War Room tab), or drop the standalone `ban-priority` tab. Keeping both means the
editor's state is silently forked.

### WR-03: Reduced-motion "OFF" re-attach bypasses the accumulation guard

**File:** `src/components/room/parallax.rs:183-194`
**Issue:** When Reduce Motion flips OFF at runtime, the change-listener
re-attaches the mousemove handler (lines 186-193) but never sets
`MOUSEMOVE_ACTIVE = true`. If a second `AtmosphereRoot` later mounts while this
one is alive, its `compare_exchange(false → true)` succeeds (the flag is still
`false`) and it attaches a *second* listener → the doubled-amplitude parallax
the guard exists to prevent. Symmetrically, the "ON" branch (lines 165-182)
removes the listener but does not clear `MOUSEMOVE_ACTIVE`, leaving the flag
`true` with no listener attached.
**Fix:** Keep `MOUSEMOVE_ACTIVE` in sync with actual listener state in both
branches: set it `true` after the OFF re-attach and `false` after the ON
detach (mirroring the initial-attach and `on_cleanup` paths). Better still,
fold this into the WR-01 fix by removing the boolean guard and relying on
remove-then-add idempotency.

### WR-04: Keep-mounted hidden panels run full auth + data-fetch lifecycles for surfaces the user never opens

**File:** `src/components/room/room_surface.rs:146-160`; consumers
`src/pages/rooms/war_room.rs:139-166`
**Issue:** The D-06 keep-mounted design calls every `PanelFn` once at room entry,
so all five War Room surfaces hydrate immediately — including each surface's auth
redirect `Effect` (`get_current_user()`) and every resource it owns
(`get_ban_priorities`, opponents list, game plans, draft series, tree list, etc.).
Five surfaces each independently fetching `get_current_user()` on the same page
means five identical auth round-trips on first paint. More concerning: each
surface's auth `Effect` independently calls
`window.location().set_href("/auth/login")` when it observes `Ok(None)` — five
surfaces racing to redirect is harmless but wasteful, and any surface whose
resource `Err`s will render an `ErrorBanner` inside a hidden panel that the user
can't see but that still counts against "No error banners" assertions if a panel
is later revealed.
**Fix:** Hoist the auth check to the room level (`WarRoomRoom`) so it runs once,
and pass authenticated state down — or accept the redundancy explicitly and
document it. At minimum, dedupe `get_current_user()` by providing a single
shared `Resource` via context rather than one per surface.

### WR-05: `save_ban_priorities` error surfaces only as transient text; no rollback of optimistic edit list

**File:** `src/pages/draft.rs:138-160` (Save handler)
**Issue:** On save failure the handler sets `ban_status` to `Error: {e}` but
leaves `editing_bans = true` and the local `ban_edit_list` unchanged, while the
server state is unmodified. That is acceptable, but the success branch calls
`ban_priorities.refetch()` and `set_editing_bans(false)` without clearing
`ban_status`, so the green "Saved!" message persists into the next edit session
(it is only cleared when the user clicks "Edit", draft.rs:202). A user who saves,
then re-opens edit via the panel toggle without clicking "Edit" may see a stale
"Saved!"/"Error:" message against unrelated state. Minor data-presentation bug.
**Fix:** Clear `set_ban_status.set(None)` at the start of the save handler and on
`editing_bans` transitions, or move the status message lifetime to be scoped to a
single save attempt.

## Info

### IN-01: `serde_json::to_string(&list).unwrap_or_default()` silently sends `""` on serialization failure

**File:** `src/pages/draft.rs:140`
**Issue:** If serialization ever failed, an empty string is sent to
`save_ban_priorities`, which would likely be interpreted as "clear all
priorities" rather than surfacing an error. `BanPriority` is a plain serde struct
so failure is effectively impossible in practice, hence Info not Warning.
**Fix:** Match on the `Result` and set an error status on `Err` instead of
`unwrap_or_default()`.

### IN-02: `MOUSEMOVE_ACTIVE` triggers a dead-code warning on the SSR target

**File:** `src/components/room/parallax.rs:22`
**Issue:** `cargo check --features ssr` emits `static MOUSEMOVE_ACTIVE is never
used` because it is read only inside the `#[cfg(feature = "hydrate")]` block.
Expected given the dual-target design, but it adds noise to SSR build output.
**Fix:** Gate the static behind `#[cfg(feature = "hydrate")]` (and the `use
std::sync::atomic` import alongside it) so it does not exist on the SSR target.

### IN-03: Duplicate `<BanPrioritySurface />` editor markup remains inline in `DraftSurface` rather than reusing the component

**File:** `src/pages/draft.rs:3755-3756`
**Issue:** The extraction is correct, but the design now mounts the same
component in two places (see WR-02). Worth a code comment at both mount sites
explaining the intended single-source-of-truth so a future editor does not
"fix" one and leave the other.
**Fix:** Add a cross-reference comment, or resolve via WR-02.

### IN-04: `panel_fn_contract_compiles_and_resolves` test uses `unreachable!()` panels — does not exercise render path

**File:** `src/components/room/room_surface.rs:202-220`
**Issue:** The test validates the `PanelFn` type alias and `resolve_active_tab`
mapping but the panels themselves can never be invoked (they `unreachable!()`).
The actual keep-mounted `hidden`-toggle render behavior is only covered by the
e2e suite, which the file comment notes. This is acceptable for a unit test but
the name oversells coverage ("contract").
**Fix:** Rename to `panel_fn_type_and_tab_resolution` or note in the docstring
that render behavior is e2e-only.

---

_Reviewed: 2026-06-05_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
