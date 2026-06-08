# Phase 29: Throne, QuickNav & Room A11Y - Pattern Map

**Mapped:** 2026-06-08
**Files analyzed:** 14 new/modified files
**Analogs found:** 13 / 14 (QuickNav focus-trap is net-new — no codebase analog)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/pages/rooms/throne.rs` (new) | page/room | request-response | `src/pages/rooms/library.rs` | exact |
| `src/models/growth.rs` (modify) | model | transform | `src/models/growth.rs` (LibraryGrowthState block) | exact |
| `src/server/db.rs` (modify) | service | CRUD | `src/server/db.rs:4200` (`get_team_growth_counts`) | exact |
| `src/pages/profile.rs` (modify) | component/page | request-response | `src/pages/stats.rs` (surface extraction in library) | role-match |
| `src/pages/team/dashboard.rs` (modify) | component/page | request-response | itself — split into 3 surfaces | role-match |
| `src/pages/action_items.rs` (modify) | component/page | CRUD | itself — fold into DevelopmentSurface | role-match |
| `src/components/room/quick_nav.rs` (modify) | component | event-driven | itself (current open/close pattern) + WCAG modal trap (no in-repo analog) | partial |
| `src/components/nav.rs` (modify) | component | request-response | itself — restructure `HUB_ROUTES` constant | exact |
| `src/app.rs` (modify) | config/route | request-response | `src/app.rs:147-161` (existing redirect routes) | exact |
| `input.css` (modify) | config/style | transform | `input.css:436-464` (War Room / Library tier selectors) | exact |
| `e2e/tests/hydration-no-panic.spec.ts` (modify) | test | event-driven | itself (existing SCOPED_ROUTES block at :72-89) | exact |
| `e2e/tests/pages.spec.ts` (modify) | test | request-response | itself (AUTHED_PAGES block at :8-25) | exact |
| `e2e/tests/throne-growth.spec.ts` (new) | test | CRUD | `e2e/tests/hydration-no-panic.spec.ts` (seed+assert shape) | role-match |
| `e2e/tests/quicknav-focus.spec.ts` (new) | test | event-driven | (no in-repo a11y focus-trap spec — pattern from WCAG 2.1 SC 2.1.2) | none |
| `e2e/tests/room-contrast.spec.ts` (new) | test | request-response | `e2e/tests/pages.spec.ts` (simple page load + DOM assert) | role-match |

---

## Pattern Assignments

### `src/pages/rooms/throne.rs` (new — page/room, request-response)

**Analog:** `src/pages/rooms/library.rs` (entire file, 330 lines) — closest sibling with both redirect shells and multi-surface room structure.

**Structural template note:** Copy `library.rs` nearly verbatim; the Throne has:
- Three redirect server fns + three redirect shell components instead of five.
- Three growth axes (`roster`/`progression`/`crown`) instead of three Library axes.
- Five surfaces (Identity/Dashboard/Roster/Development/Solo) instead of five Library surfaces.
- `get_throne_growth_state` that does NOT early-return on no-team (D-07 divergence).

**Imports pattern** (`library.rs:12-20`):
```rust
use leptos::prelude::*;

use crate::app::InitialTheme;
use crate::components::room::{AtmosphereRoot, GrowthOverlay, Plane, PanelFn, RoomSurface, SurfaceTab};
use crate::pages::profile::IdentitySurface;
use crate::pages::team::dashboard::{DashboardSurface, RosterSurface, DevelopmentSurface};
use crate::pages::solo_dashboard::SoloSurfaces;
```

**Redirect server fn pattern** (`library.rs:25-50`):
```rust
#[server]
pub async fn redirect_to_throne_identity() -> Result<(), ServerFnError> {
    leptos_axum::redirect("/throne?tab=identity");
    Ok(())
}
// ...analogous redirect_to_throne_dashboard, redirect_to_throne_development
```

**Redirect shell component pattern** (`library.rs:67-78` — `StatsRedirect` is the literal template):
```rust
#[component]
pub fn ProfileRedirect() -> impl IntoView {
    let action = ServerAction::<RedirectToThroneIdentity>::new();
    action.dispatch(RedirectToThroneIdentity {});

    #[cfg(feature = "hydrate")]
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/throne?tab=identity");
    }

    view! { <></> }
}
```

**Growth server fn — THE D-07 DIVERGENCE** (synthesized from `war_room.rs:27-53` + `solo_dashboard.rs:71-90`):
The Throne server fn computes `progression` FIRST, before the team check. Do NOT copy `library.rs:203-207`'s early-return pattern. Structure:
```rust
#[server]
pub async fn get_throne_growth_state(
) -> Result<crate::models::growth::ThroneGrowthState, ServerFnError> {
    // ... auth + db context extraction (copy war_room.rs:36-41 verbatim) ...

    // 1. PERSONAL axis FIRST — never fails on absent data (rule #44)
    let ranked = db::get_latest_ranked_snapshot(&surreal, &user.id, "RANKED_SOLO_5x5")
        .await.unwrap_or(None);

    // 2. TEAM axes — fall to 0 without a team (do NOT early-return the whole state)
    let team_counts = match db::get_user_team_id(&surreal, &user.id).await? {
        Some(team_id) => db::get_throne_growth_counts(&surreal, &team_id)
            .await.unwrap_or_default(),
        None => crate::models::growth::ThroneGrowthCounts::default(),
    };

    Ok(crate::models::growth::ThroneGrowthState::from_parts(team_counts, ranked))
}
```

**Room component pattern** (`library.rs:220-285` — `LibraryRoom` is the template):
```rust
#[component]
pub fn ThroneRoom() -> impl IntoView {
    let region = use_context::<InitialTheme>().unwrap_or_default().0; // ONCE, rule #58

    let growth = Resource::new(|| (), |_| get_throne_growth_state());
    let roster_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.roster_tier).unwrap_or(0)
    });
    let progression_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.progression_tier).unwrap_or(0)
    });
    let crown_tier = Signal::derive(move || {
        growth.get().and_then(|r| r.ok()).map(|g| g.crown_tier).unwrap_or(0)
    });

    let dir = format!("/img/rooms/throne/{region}");
    let surfaces_region = region.clone();

    view! {
        <AtmosphereRoot
            class="min-h-screen".to_string()
            roster_tier=roster_tier
            progression_tier=progression_tier
            crown_tier=crown_tier
        >
            <Plane src=format!("{dir}/plane-sky.webp") depth=6 z=1 />
            <Plane src=format!("{dir}/plane-mid.webp") depth=16 z=2 />
            <Plane src=format!("{dir}/plane-near.webp") depth=32 z=3 />
            <GrowthOverlay axis="roster" src=format!("{dir}/overlay-roster.webp") z=4 />
            <GrowthOverlay axis="progression" src=format!("{dir}/overlay-progression.webp") z=5 />
            <GrowthOverlay axis="crown" src=format!("{dir}/overlay-crown.webp") z=6 />
            // D-15: z-8 contrast scrim (verbatim from solo_journey.rs:69 / library.rs:272-277)
            <div class="fixed inset-0 pointer-events-none"
                style="z-index: 8; background: radial-gradient(ellipse at center, transparent 0%, var(--color-base) 95%);"
                aria-hidden="true"></div>
            <div class="relative z-10 px-8 py-6" data-feedback-label="Throne">
                <ThroneSurfaces region=surfaces_region />
            </div>
        </AtmosphereRoot>
    }
}
```

**Surface composer pattern** (`library.rs:295-329` — `LibrarySurfaces` is the template):
```rust
#[component]
fn ThroneSurfaces(region: String) -> impl IntoView {
    // Provide shared context BEFORE view! if any surfaces need to share state
    // (see library.rs:299-300 — provide_context before panels vec).

    let panels: Vec<(SurfaceTab, PanelFn)> = vec![
        (SurfaceTab { id: "identity",    label: "Identity"    },
         Box::new(move || view! { <IdentitySurface /> }.into_any()) as PanelFn),
        (SurfaceTab { id: "dashboard",   label: "Dashboard"   },
         Box::new(move || view! { <DashboardSurface /> }.into_any()) as PanelFn),
        (SurfaceTab { id: "roster",      label: "Roster"      },
         Box::new(move || view! { <RosterSurface /> }.into_any()) as PanelFn),
        (SurfaceTab { id: "development", label: "Development" },
         Box::new(move || view! { <DevelopmentSurface /> }.into_any()) as PanelFn),
        (SurfaceTab { id: "solo",        label: "Solo"        },
         Box::new(move || {
             let r = region.clone();
             view! { <SoloSurfaces region=r /> }.into_any()
         }) as PanelFn),
    ];

    view! { <RoomSurface panels=panels /> }
}
```

**Asset path divergence note:** Use `/img/rooms/throne/{region}/` — verify `public/img/rooms/library/demacia/` and `public/img/rooms/pandemonium/` for the exact filenames and format to mirror for placeholder WebPs.

---

### `src/models/growth.rs` — add `ThroneGrowthCounts` + `ThroneGrowthState` (model, transform)

**Analog:** `src/models/growth.rs:157-204` — the `LibraryGrowthCounts` + `LibraryGrowthState` block (last added; closest shape).

**Key divergence from Library:** `ThroneGrowthState` has no `counts` field (unlike `TeamGrowthState` and `LibraryGrowthState`). The blended constructor takes both team counts and an optional `RankedInfo` — it cannot be derived from a single `*Counts` struct. Add after `LibraryGrowthState`.

**Struct definitions to add** (after `growth.rs:204`, immediately following `LibraryGrowthState::Default` impl):
```rust
// --- Throne (blended team + personal) growth ---

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThroneGrowthCounts {
    /// team_member rows for the team (roster axis).
    pub team_members: u32,
    /// post_game_learning rows where win_loss = "win" (crown axis).
    pub team_wins: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThroneGrowthState {
    pub roster_tier: u8,
    pub progression_tier: u8,
    pub crown_tier: u8,
}

impl ThroneGrowthState {
    /// Blended derivation: team counts + the viewing user's optional ranked snapshot.
    /// progression derives independently so it advances with NO team (D-05/D-07).
    pub fn from_parts(
        team: ThroneGrowthCounts,
        ranked: Option<crate::models::user::RankedInfo>,
    ) -> Self {
        let roster = tier_for(team.team_members, [1, 3, 5]);
        let crown  = tier_for(team.team_wins, [1, 5, 15]);
        let games  = ranked.as_ref()
            .map(|r| (r.wins + r.losses).max(0) as u32)
            .unwrap_or(0);
        let progression = tier_for(games, [1, 20, 60]);
        ThroneGrowthState {
            roster_tier:     roster.level(),
            progression_tier: progression.level(),
            crown_tier:      crown.level(),
        }
    }
}

impl Default for ThroneGrowthState {
    fn default() -> Self { Self::from_parts(ThroneGrowthCounts::default(), None) }
}
```

**`tier_for` is private** (`growth.rs:66`) — already accessible within the same file; no visibility change needed.

**Test pattern** (`growth.rs:334-371` — Library tests block is the template):
Add five unit tests to `#[cfg(test)]` block:
- `throne_empty_all_seed` — `ThroneGrowthState::default()` → all 0
- `throne_roster_tracks_members`
- `throne_crown_tracks_wins`
- `throne_progression_grows_without_team` — `from_parts(ThroneGrowthCounts::default(), Some(ranked_60_games))` → `progression_tier == 3`, `roster_tier == 0`
- `throne_state_round_trips_json`

---

### `src/server/db.rs` — add `get_throne_growth_counts` (service, CRUD)

**Analog:** `src/server/db.rs:4200` — `get_team_growth_counts` (the batch `count() GROUP ALL` template).

**Pattern to copy** (db.rs:4200-4243 shape):
```rust
pub async fn get_throne_growth_counts(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<ThroneGrowthCounts> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut result = db.query(
        "SELECT count() as n FROM team_member WHERE team = type::record('team', $team_key) GROUP ALL; \
         SELECT count() as n FROM post_game_learning \
           WHERE team = type::record('team', $team_key) AND win_loss = 'win' GROUP ALL",
    )
    .bind(("team_key", team_key))
    .await?;
    // Use the count_of helper pattern from db.rs:4224 (or inline the take pattern)
    let team_members: u32 = result.take::<Option<CountRow>>(0)?
        .map(|r| r.n).unwrap_or(0);
    let team_wins: u32 = result.take::<Option<CountRow>>(1)?
        .map(|r| r.n).unwrap_or(0);
    Ok(ThroneGrowthCounts { team_members, team_wins })
}
```

**CRITICAL: verify before writing.** RESEARCH.md flags two field names as unverified:
- **A2:** The exact `win_loss` string value (verify by grepping `save_post_game_learning` at `db.rs:1890` for the written literal — "win", "Win", or another form).
- **A3:** The `team_member` table's team reference field name — grep `schema.surql` for `DEFINE FIELD` lines under `team_member` before using `WHERE team = ...`.

---

### `src/pages/profile.rs` — add `IdentitySurface` (component, request-response)

**Analog:** `src/pages/stats.rs` (surface extraction in Library — `StatsSurface` in `stats.rs`, same pattern where the surface lives in the originating page file and is imported into the room composer).

**Extraction rule** (`war_room.rs:15-18` comment): The surface lives in its originating file (`profile.rs`), NOT in `throne.rs`. `throne.rs` imports it.

**Pattern:** Extract the body of the existing `ProfilePage` view (Riot-account link, champion-pool summary, team crest/banner) into a `pub fn IdentitySurface()` component in `profile.rs`. The existing server fns (`get_current_user`, etc.) stay in `profile.rs`.

**Empty-state pattern:** Identity surface always renders (D-02) — no-team → show empty-state CTA inside the component, NOT a structural `<Show when=has_team>` wrapper.

---

### `src/pages/team/dashboard.rs` — add `DashboardSurface` / `RosterSurface` / `DevelopmentSurface` (component, request-response)

**Analog:** itself — the existing `TeamDashboard` component (`dashboard.rs:617-723`) contains all the source data.

**Resource ownership map (Pitfall 4 prevention — map BEFORE extracting):**

| Surface | Owns these Resources | Server fns to pull in |
|---|---|---|
| `DashboardSurface` | `get_team_dashboard` (summary panels), `get_open_action_items_summary`, `get_post_game_panel`, `get_pool_gap_panel`, `get_recent_team_matches` | all summary/panel fns |
| `RosterSurface` | `get_team_dashboard` (member list), `get_pending_requests`, `handle_join_request` | roster + request fns |
| `DevelopmentSurface` | Action Items fns (from `action_items.rs`), `get_all_team_notes`/`add_team_note`/`toggle_pin`/`edit`/`remove` | action items + notebook fns |

**`get_team_dashboard` is needed by BOTH Dashboard and Roster.** Either: (a) provide it once at `ThroneSurfaces` via `provide_context` (Library pattern, `library.rs:299-300`), or (b) accept a double-fetch (eager-fetch optimization explicitly deferred — CONTEXT Deferred). The planner should pick option (a) and document the decision.

**`TeamNotebook` to move** (`dashboard.rs:1525-1526`):
```rust
// Source: dashboard.rs:1525-1526 — self-contained component, move into DevelopmentSurface
#[component]
fn TeamNotebook(current_user_id: String, is_leader: bool) -> impl IntoView { ... }
// It already owns: notes_resource, add_team_note, toggle_pin, edit, remove
```

**CRITICAL: strip ModeToggle chrome.** The existing `TeamDashboard` (`dashboard.rs:676-686`) has a `ModeToggle` / `is_solo_mode` branch / `TeamGameDayBriefView` region switch. When extracting surfaces, these are stripped — each surface renders ONE consistent structure. Keep only the data Resources and the actual UI panels. The `set_team_dashboard_mode_pref` DB field is left untouched (no migration needed).

**Empty-state pattern (D-02):** No-team user gets the existing empty-state CTA (copied verbatim from `dashboard.rs:689-712`) inside each team-scoped surface. No `<Show when=has_team>` wrapper — rule #58 structural guard.

---

### `src/components/room/quick_nav.rs` — filter + focus trap (component, event-driven)

**Analog:** Itself (`quick_nav.rs:24-124`) for the existing open/close, keyboard handler, and `HUB_ROUTES` iteration patterns. The focus trap and filter are net-new.

**Existing open/close pattern to preserve** (`quick_nav.rs:26-54`):
```rust
let open = RwSignal::new(false);

#[cfg(feature = "hydrate")]
{
    Effect::new(move |_| {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;
        let cb = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
            move |ev: web_sys::KeyboardEvent| {
                let key = ev.key();
                if key == "Escape" {
                    open.set(false);
                } else if (ev.meta_key() || ev.ctrl_key()) && key.eq_ignore_ascii_case("k") {
                    ev.prevent_default();
                    open.update(|o| *o = !*o);
                }
            },
        );
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
        }
        cb.forget(); // app-lifetime mount — forget is acceptable (per comment at :51-53)
    });
}
```

**Existing focus-visible ring to preserve** (`quick_nav.rs:65,108`):
```
focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none
```
Do NOT remove these classes. Guardrail G-12 (no `outline:none` without replacement).

**Filter input (D-09 — new):**
```rust
let filter = RwSignal::new(String::new());
let input_ref = NodeRef::<leptos::html::Input>::new();

// Auto-focus on open — inside an Effect watching `open`:
#[cfg(feature = "hydrate")]
Effect::new(move |_| {
    if open.get() {
        filter.set(String::new()); // clear on each open
        if let Some(el) = input_ref.get() {
            let _ = el.focus();
        }
    }
});

// In view! — the filter input at top of dialog:
<input
    node_ref=input_ref
    type="text"
    prop:value=move || filter.get()
    on:input=move |ev| filter.set(event_target_value(&ev))
    placeholder="Jump to…"
    class="w-full bg-transparent px-5 py-3 text-sm text-primary placeholder:text-dimmed border-b border-divider focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
/>

// Filter the HUB_ROUTES iteration:
// label.to_lowercase().contains(&filter.get().to_lowercase())
```

**Focus trap + focus-restore (D-10 — net-new, no codebase analog):**

There is no existing focus-trap / `inert` / `activeElement` code anywhere in `src/` (confirmed by RESEARCH.md). This is the only genuinely new mechanism. Use the wrap-on-Tab approach (lower risk than `inert`-on-siblings):

```rust
// Capture trigger element on open:
#[allow(unused_variables)]
let trigger_el: RwSignal<Option<web_sys::Element>> = RwSignal::new(None);

// When open is toggled true (the Cmd+K handler or the button click):
#[cfg(feature = "hydrate")]
{
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        trigger_el.set(doc.active_element());
    }
}

// Centralized close fn — ALL close paths call this (Escape, backdrop, link click):
let close = move || {
    open.set(false);
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        if let Some(el) = trigger_el.get_untracked() {
            if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                let _ = html_el.focus();
            }
        }
    }
};

// Tab trap on the dialog container — intercept Tab/Shift+Tab:
// Add on:keydown to the <div class="relative w-full max-w-lg ..."> container
// that wraps the filter input + nav items. On Tab: move focus to next focusable;
// on last item + Tab → wrap to filter input. On Shift+Tab on filter input → wrap to last item.
// All existing on:click=move |_| open.set(false) become on:click=move |_| close()
```

**web-sys feature note:** `HtmlElement::focus()` uses the `HtmlElement` web-sys feature already enabled (`Cargo.toml:42`). `document().active_element()` returns `Option<Element>` — the `Element` feature is also enabled. If `.dyn_into::<HtmlElement>()` requires an additional feature at compile time, add it; expect none needed.

---

### `src/components/nav.rs` — restructure `HUB_ROUTES` + update consumers (component, request-response)

**Analog:** Itself — `nav.rs:160-225` (current `HUB_ROUTES` + `hub_for_path`) and `nav.rs:522-557` (sub-nav strip).

**Current HUB_ROUTES** (`nav.rs:160-193`) — REPLACE entirely:
```rust
pub const HUB_ROUTES: &[(&str, &[(&str, &str)])] = &[
    ("strategy", &[
        ("/draft", "Draft"),
        ("/tree-drafter", "Tree"),
        ("/champion-pool", "Pool"),
        ("/game-plan", "Game plan"),
        ("/post-game", "Post-game"),
        ("/opponents", "Opponents"),
        ("/action-items", "Action items"),
    ]),
    ("history", &[ ... ]),
    ("profile", &[ ... ]),
];
```

**Replace with room-centric structure (D-11):**
```rust
pub const HUB_ROUTES: &[(&str, &[(&str, &str)])] = &[
    ("war-room", &[
        ("/war-room?tab=draft", "Draft"),
        ("/war-room?tab=tree-drafter", "Tree"),
        ("/war-room?tab=game-plan", "Game Plan"),
        ("/war-room?tab=opponents", "Opponents"),
        ("/war-room?tab=ban-priority", "Ban Priority"),
    ]),
    ("library", &[
        ("/library?tab=stats", "Stats"),
        ("/library?tab=match", "Match"),
        ("/library?tab=analytics", "Analytics"),
        ("/library?tab=post-game", "Post-Game"),
        ("/library?tab=personal", "Learnings"),
    ]),
    ("throne", &[
        ("/throne?tab=identity", "Identity"),
        ("/throne?tab=dashboard", "Dashboard"),
        ("/throne?tab=roster", "Roster"),
        ("/throne?tab=development", "Development"),
        ("/throne?tab=solo", "Solo"),
    ]),
];
```

**ALL five touch-points that must change together (Pitfall 5):**

1. `HUB_ROUTES` constant (`nav.rs:160`) — restructure as above.
2. `hub_for_path` (`nav.rs:198-225`) — update from `"strategy"`/`"history"`/`"profile"` keys to `"war-room"`/`"library"`/`"throne"`, and update the path-prefix matching to include `/throne`, `/war-room`, `/library`.
3. Primary hub `<A>` buttons (`nav.rs:320-332`) — update href targets and labels:
   - "Strategy" `href="/draft"` → `href="/war-room"` (label stays or becomes "War Room")
   - "History" `href="/stats"` → `href="/library"` (label stays or becomes "Library")
   - "Profile" `href="/profile"` → `href="/throne"` (profile now redirects — must point at room)
4. `hub_btn_class` memo keys (`nav.rs:303-306`) — `"strategy"` → `"war-room"`, `"history"` → `"library"`, `"profile"` → `"throne"`.
5. `quick_nav.rs:15-22` `hub_label` function — update key mapping:
   ```rust
   fn hub_label(key: &str) -> &'static str {
       match key {
           "war-room" => "War Room",
           "library"  => "Hall of Records",
           "throne"   => "Throne",
           _          => "More",
       }
   }
   ```

**Sub-nav strip** (`nav.rs:522-557`) reads `HUB_ROUTES` via the `active_hub` signal — no code change needed there if `hub_for_path` is updated correctly. Verify `is_active` path-matching (`:538`) works with `?tab=` URLs (it uses `starts_with`, which will match `/war-room?tab=draft` against `/war-room?tab=draft` — confirm).

---

### `src/app.rs` — add `/throne` route + three redirects (config/route, request-response)

**Analog:** `src/app.rs:147,151,152,160` — the existing redirect route entries (`StatsRedirect`, `PostGameRedirect`, `OpponentsRedirect`, `PersonalLearningsRedirect`).

**Current routes to convert** (`app.rs:140,143,153`):
```rust
<Route path=path!("/profile")          view=ProfilePage />
<Route path=path!("/team/dashboard")   view=TeamDashboard />
<Route path=path!("/action-items")     view=ActionItemsPage />
```

**After Phase 29** — add `/throne`, replace three routes with redirect shells:
```rust
<Route path=path!("/throne")          view=ThroneRoom />
<Route path=path!("/profile")         view=ProfileRedirect />
<Route path=path!("/team/dashboard")  view=TeamDashboardRedirect />
<Route path=path!("/action-items")    view=ActionItemsRedirect />
```

**Sub-route audit (D-12 preservation — from RESEARCH.md Pattern 5):**
- `/profile` — no sub-routes in `app.rs`; safe to redirect.
- `/team/dashboard` — sibling `/team/roster` (`app.rs:144`) is a SEPARATE route; do NOT redirect it.
- `/action-items` — no sub-routes; safe to redirect.
- `/journey` — do NOT touch (D-13).

**Import changes needed in `app.rs`:** add `ThroneRoom`, `ProfileRedirect`, `TeamDashboardRedirect`, `ActionItemsRedirect` to the use declarations at the top of the file (mirror the existing Library imports pattern).

---

### `input.css` — add nine tier→opacity selectors (config/style, transform)

**Analog:** `input.css:436-464` — the existing War Room and Library tier selector blocks. Copy the exact same pattern.

**Current state** (confirmed by grep): `roster`/`progression`/`crown` selectors are ABSENT. The `.room-overlay` base rule defaults to `opacity: 0` (`input.css:431`). Without these nine lines, all Throne overlays stay fully transparent at every tier — Pitfall 1 (the Phase 27 "room growth visibility gap").

**Add immediately after `input.css:464`** (after the Library `analysis` block):
```css
/* Throne axes (roster/progression/crown) — same tier→opacity ramp. */
[data-roster-tier="1"]      .room-overlay--roster      { opacity: 0.34; }
[data-roster-tier="2"]      .room-overlay--roster      { opacity: 0.67; }
[data-roster-tier="3"]      .room-overlay--roster      { opacity: 1;    }
[data-progression-tier="1"] .room-overlay--progression { opacity: 0.34; }
[data-progression-tier="2"] .room-overlay--progression { opacity: 0.67; }
[data-progression-tier="3"] .room-overlay--progression { opacity: 1;    }
[data-crown-tier="1"]       .room-overlay--crown       { opacity: 0.34; }
[data-crown-tier="2"]       .room-overlay--crown       { opacity: 0.67; }
[data-crown-tier="3"]       .room-overlay--crown       { opacity: 1;    }
```

This is a Wave 0 BLOCKER — must land before any Throne e2e test that checks overlay opacity.

---

### `e2e/tests/hydration-no-panic.spec.ts` — add `/throne` to `SCOPED_ROUTES` (test, event-driven)

**Analog:** Itself — `hydration-no-panic.spec.ts:72-89` (`SCOPED_ROUTES` block).

**Add one entry** (`hydration-no-panic.spec.ts:88`, just after `"/library"`):
```typescript
const SCOPED_ROUTES = [
  // ... existing entries ...
  "/library",
  // Phase 29 Throne room (SC4 A11Y-02 gate) — exercises both regions;
  // region read ONCE at ThroneRoom top → rule #58 structural guard.
  "/throne",
] as const;
```

The existing `for (const region of ["demacia", "pandemonium"])` loop at `:145` auto-generates 2 new tests (one per region) — no other changes needed in this file.

---

### `e2e/tests/pages.spec.ts` — add `/throne` + update redirected entries (test, request-response)

**Analog:** Itself — `pages.spec.ts:8-25` (`AUTHED_PAGES` array).

**Add `/throne` entry:**
```typescript
{ path: "/throne", content: /identity|dashboard|roster|development|solo/i },
```

**Update redirected entries** — `/profile` and `/team/dashboard` now 302 to the Throne. Either:
- Remove their entries from `AUTHED_PAGES` (they no longer render standalone content), OR
- Update them to `expect(authedPage.url()).toContain("/throne")` after navigation.

**Verify** whether `/action-items` has an `AUTHED_PAGES` entry — the current file (`pages.spec.ts:8-25`) does NOT include `/action-items` (confirmed), so no removal needed there.

**Keep unchanged:** `/team/roster`, `/team-builder`, `/solo` — these are NOT redirected.

---

### `e2e/tests/throne-growth.spec.ts` (new — test, CRUD)

**Analog:** `e2e/tests/hydration-no-panic.spec.ts` (seed-and-assert shape); the Phase 27 lesson from MEMORY.md `project_room_growth_visibility_gap.md`.

**Key pattern — assert computed opacity, NOT just data-attr:**
```typescript
// After seeding a high-tier scenario:
const overlay = await page.locator('.room-overlay--roster');
const opacity = await overlay.evaluate(el =>
    getComputedStyle(el).opacity
);
expect(parseFloat(opacity)).toBeGreaterThan(0);
// Also assert the data-attr:
const root = await page.locator('[data-roster-tier]');
await expect(root).toHaveAttribute('data-roster-tier', /[1-3]/);
```

**SC1b — blended test:** Assert that a teamless user with ranked data gets `data-progression-tier > 0` and `data-roster-tier = 0`.

---

### `e2e/tests/quicknav-focus.spec.ts` (new — test, event-driven)

**No in-repo analog.** Use Playwright's `page.keyboard` and `page.evaluate(document.activeElement)` pattern.

**Key test cases (SC2 + SC3):**
```typescript
// SC3 — Tab stays within open overlay:
await page.keyboard.press('Meta+k'); // open
// Tab through all items and verify focus never leaves the dialog
for (let i = 0; i < 20; i++) {
    await page.keyboard.press('Tab');
    const focusedEl = await page.evaluate(() => document.activeElement);
    // assert focusedEl is inside [role="dialog"]
}

// SC2 — Escape restores focus to trigger:
const triggerHandle = await page.locator('button[aria-label*="quick navigation"]');
await triggerHandle.focus();
await page.keyboard.press('Meta+k'); // open
await page.keyboard.press('Escape'); // close
const focused = await page.evaluate(() => document.activeElement?.ariaLabel);
expect(focused).toMatch(/quick navigation/i);
```

---

### `e2e/tests/room-contrast.spec.ts` (new — test, request-response)

**Analog:** `e2e/tests/pages.spec.ts` (simple page-load + DOM assertion pattern).

**Key assertion (D-15 — SC5):**
```typescript
// Assert z-8 contrast scrim present on each room:
for (const room of ['/war-room', '/library', '/throne']) {
    await page.goto(room);
    await page.waitForLoadState('networkidle');
    const scrim = await page.locator('div[style*="radial-gradient"]').first();
    await expect(scrim).toBeVisible();
    // Optional: sample computed background to verify z-8 position
}
```

---

## Shared Patterns

### Region-Read-Once (rule #58 — apply to ALL room components)
**Source:** `src/pages/rooms/solo_journey.rs:18` (earliest example); replicated in `war_room.rs:64`, `library.rs:224`.
**Apply to:** `ThroneRoom` (the only new room component — all five `*Surface` components receive region as a plain `String` prop; they NEVER call `use_context::<InitialTheme>()`).
```rust
// CORRECT — at ThroneRoom top only:
let region = use_context::<InitialTheme>().unwrap_or_default().0;
// ...pass as: region=region.clone() prop to ThroneSurfaces
// WRONG — never inside Signal::derive, Effect, move || {}, view! branch, or child component
```

### Contrast Scrim (D-15 — apply to all three rooms for SC5 verification)
**Source:** `src/pages/rooms/solo_journey.rs:69-73` (canonical) / `war_room.rs:112-117` / `library.rs:272-277`.
**Apply to:** `ThroneRoom` view — copy verbatim between the GrowthOverlay children and the z-10 surface div.
```rust
<div
    class="fixed inset-0 pointer-events-none"
    style="z-index: 8; background: radial-gradient(ellipse at center, transparent 0%, var(--color-base) 95%);"
    aria-hidden="true"
></div>
```

### Auth Extraction in Server Fns
**Source:** `src/pages/rooms/war_room.rs:36-41` (most recent room example).
**Apply to:** `get_throne_growth_state`.
```rust
let auth: AuthSession = leptos_axum::extract().await?;
let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
let surreal = use_context::<Arc<Surreal<Db>>>()
    .ok_or_else(|| ServerFnError::new("No DB context"))?;
```

### Rule #44 — Return `Ok(default)` not `Err` for absent data
**Source:** `src/pages/rooms/war_room.rs:43-47`, `src/pages/rooms/library.rs:203-207`.
**Apply to:** Every server fn in all new surfaces. For `get_throne_growth_state`: the team axes fall to `ThroneGrowthCounts::default()` on no-team; the `progression` axis falls to `tier_for(0, ...)` on no ranked data. Never `Err` for absent optional data.

### WASM Safety Guards
**Source:** `.claude/rules/wasm-patterns.md` rules #35, #37, #42, #43.
**Apply to:** All new client-side code in `quick_nav.rs` (focus trap, filter Effect, auto-focus):
- `#[cfg(feature = "hydrate")]` on all DOM manipulation.
- `#[allow(unused_variables)]` on any `RwSignal` only read/written inside `#[cfg(hydrate)]` blocks.
- No bare `.unwrap()` on `web_sys::window()` or `.focus()` calls.

### `PanelFn` / `RoomSurface` composition
**Source:** `src/pages/rooms/library.rs:306-329` (the `LibrarySurfaces` composer — five panels as `Vec<(SurfaceTab, PanelFn)>`).
**Apply to:** `ThroneSurfaces`. Do NOT wrap panels in `<Show>` — `RoomSurface` already keep-mounts + CSS-hides via `attr:hidden` (`room_surface.rs:146-160`).

### Empty-State CTA inside surfaces (D-02 guard)
**Source:** `src/pages/team/dashboard.rs:689-712` (the existing no-team / solo-mode empty state CTA block).
**Apply to:** `DashboardSurface`, `RosterSurface`, `DevelopmentSurface`. Copy the existing empty-state block INTO the surface, showing it when the team data Resource returns `None`. Never use a structural `<Show when=has_team>` wrapper around the whole surface (hydration-branch risk, rule #58 class).

---

## No Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `e2e/tests/quicknav-focus.spec.ts` | test | event-driven | No existing focus-trap / `activeElement` capture pattern in any `e2e/` spec; WCAG modal trap is net-new to this codebase. Write from scratch using Playwright `page.keyboard` + `page.evaluate(document.activeElement)`. |
| `get_throne_growth_state` (blended D-07 logic) | service | request-response | The "compute personal axis BEFORE the team check" ordering is the inverse of both sibling server fns (`war_room.rs:44-47`, `library.rs:203-207`). No existing server fn blends user-scoped and team-scoped data. The personal axis portion mirrors `solo_dashboard.rs:71-90`; the team-gated portion mirrors `war_room.rs:27-53`. These must be combined in the new ordering. |

---

## dashboard.rs Split — Surface Responsibility Table

Extracted from `team/dashboard.rs` (2286 lines) to three surfaces. Each surface is defined as a `pub fn *Surface` in `dashboard.rs`, NOT in `throne.rs`.

| Block in dashboard.rs | Moves to | Notes |
|---|---|---|
| `get_team_dashboard` server fn | Stays in dashboard.rs; imported by both DashboardSurface and RosterSurface | Double-import OK; eager-fetch deferred |
| `get_pending_requests` server fn | Stays in dashboard.rs | RosterSurface owns |
| `get_open_action_items_summary` server fn | Stays in dashboard.rs | DashboardSurface owns (summary only) |
| `get_post_game_panel` / `get_pool_gap_panel` server fns | Stays in dashboard.rs | DashboardSurface owns |
| `get_recent_team_matches` server fn | Stays in dashboard.rs | DashboardSurface owns |
| Dashboard summary panels (prep priorities, at-a-glance) | `DashboardSurface` | Strip ModeToggle chrome; remove `is_solo_mode` / Brief-mode branches |
| Roster slots, coach slots, join requests | `RosterSurface` | Strip ModeToggle chrome |
| `TeamNotebook` component (`:1525`) | `DevelopmentSurface` | Move the entire component definition |
| Action Items content | `DevelopmentSurface` | Import Action Items server fns from `action_items.rs` |
| `ModeToggle` / `TeamGameDayBriefView` / `PandemoniumTeamDashboard` / `is_solo_mode` branch | DROPPED — not carried into any surface | Confirm: CONTEXT D-01 scopes Dashboard to "summary"; Game Day Brief is flagged in RESEARCH A5 — planner should flag to user before deleting |

---

## Metadata

**Analog search scope:** `src/pages/rooms/`, `src/components/room/`, `src/models/growth.rs`, `src/components/nav.rs`, `src/app.rs`, `input.css`, `e2e/tests/`
**Files read:** 13 source files + both context documents
**Key verified file:line references:**
- Room room structure template: `library.rs:220-285`
- Redirect shell template: `library.rs:67-78`
- Growth early-return (pattern to NOT copy for Throne): `war_room.rs:44-47`
- Contrast scrim: `solo_journey.rs:69-73`
- `tier_for` utility: `growth.rs:66-76`
- `LibraryGrowthState` as `ThroneGrowthState` shape analog: `growth.rs:157-204`
- `get_team_growth_counts` batch pattern: `db.rs:4200`
- `HUB_ROUTES` current structure: `nav.rs:160-193`
- `hub_for_path`: `nav.rs:198-225`
- Sub-nav strip: `nav.rs:522-557`
- `QuickNav` current implementation: `quick_nav.rs:24-124`
- `SCOPED_ROUTES` block: `hydration-no-panic.spec.ts:72-89`
- `AUTHED_PAGES` block: `pages.spec.ts:8-25`
- Existing tier→opacity selectors (gap confirmed): `input.css:436-464`
- `TeamNotebook` component: `dashboard.rs:1525`
- Current app.rs route block: `app.rs:139-167`
- `ranked_snapshot` source: `db.rs:5007` (`get_latest_ranked_snapshot`)
- `SoloSurfaces` pub fn: `solo_dashboard.rs:282-283`

**Pattern extraction date:** 2026-06-08
