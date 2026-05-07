# Phase 17: UI Consolidation - Pattern Map

**Mapped:** 2026-05-07
**Files analyzed:** ~45 files (heavy restyle of ~25 + ~10 new + ~20 asset files)
**Analogs found:** 38 / 45 (high coverage — phase is largely a re-skin of established surfaces)

---

## File Classification

### New files (no analog → use closest existing pattern)

| New file | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `public/fonts/cinzel/*.woff2` | static asset | file-I/O (CDN) | (none — first font self-hosting in repo) | n/a |
| `public/fonts/cormorant-garamond/*.woff2` | static asset | file-I/O | (none) | n/a |
| `public/fonts/inter/*.woff2` | static asset | file-I/O | (none) | n/a |
| `public/fonts/jetbrains-mono/*.woff2` | static asset | file-I/O | (none) | n/a |
| `public/fonts/vt323/*.woff2` | static asset | file-I/O | (none) | n/a |
| `public/img/beta-landing-{demacia,pandemonium}.jpg` | static asset | file-I/O | (none) | n/a |
| `public/img/auth-bg-demacia.jpg` (optional) | static asset | file-I/O | (none) | n/a |
| `.planning/assets/AI-IMAGES.md` | docs | text doc | `.planning/phases/*/RESEARCH.md` (markdown table style) | role-match |
| `src/pages/closed_beta.rs` | page (new — hero tier) | request-response | `src/pages/home.rs` (auth-aware page with CTA) | role-match |
| `src/pages/admin/invites.rs` | page (new — utility tier, visual stub) | CRUD | `src/pages/team/dashboard.rs` (table-driven mgmt page) | role-match |
| `src/pages/legal/impressum.rs` | page stub | static | `src/pages/home.rs::LandingPage` (static content w/ CTAs) | partial |
| `src/pages/legal/datenschutz.rs` | page stub | static | `src/pages/home.rs::LandingPage` | partial |
| `src/pages/solo_dashboard.rs` (already exists in router but spec calls for restyle) | page (heavy restyle) | request-response | `src/pages/home.rs::Dashboard` | exact |
| `src/components/ornaments.rs` | component (new — SVG primitives) | render-only | `src/components/theme_toggle.rs` (inline SVG paths) | partial |
| `src/components/icon.rs` | component (new — shared `<Icon>`) | render-only | `src/components/theme_toggle.rs:97-110` (inline SVG with viewBox + stroke) | partial |
| `src/components/bug_report_widget.rs` | component (visual stub) | event-driven | `src/components/ui.rs::ToastOverlay` (fixed-position floating UI) | role-match |
| `e2e/tests/theme.spec.ts` | test | request-response | `e2e/tests/pages.spec.ts` (smoke test pattern) | exact |
| `e2e/tests/closed-beta-visual.spec.ts` | test | request-response | `e2e/tests/smoke.spec.ts` (public-page test) | exact |
| `e2e/tests/visual-regression.spec.ts` | test | request-response | `e2e/tests/pages.spec.ts` (loops over routes) | exact |
| `e2e/tests/fonts.spec.ts` | test | request-response | `e2e/tests/smoke.spec.ts` (network-tab assertion pattern) | role-match |
| `/home/jasper/Repositories/open-design/design-systems/lol-companion/DESIGN.md` | external docs | n/a | (none in this repo — read AGENTS.md in target repo) | n/a (cross-repo) |

### Modified files (heavy restyle)

| Modified file | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `input.css` | config (CSS) | static | (self — current `@theme` block + `[data-theme]`/`[data-accent]` selectors) | self-pattern |
| `schema.surql` | config (DB) | CRUD | `schema.surql:10` `mode` field (Phase 12 precedent) | exact |
| `src/server/db.rs` | service | CRUD | `set_user_mode`/`get_user_mode` at `src/server/db.rs:4470-4488` | exact |
| `src/server/auth.rs` | model + service | CRUD | `DbUser`/`AppUser` at `src/server/auth.rs:17-53` (add `theme` field) | exact (additive) |
| `src/app.rs` | router/shell | request-response | (self — existing `shell()` + `<Routes>`) | self-pattern |
| `src/main.rs` | server bootstrap | request-response | (self — existing `leptos_routes_with_context` block) | self-pattern (additive: plumb `user.theme`) |
| `src/components/nav.rs` (748 lines) | component | event-driven | (self — existing `Nav` component preserves notifications + ModeToggle) | self-pattern (heavy restructure) |
| `src/components/theme_toggle.rs` | component | event-driven | (self — `set_user_mode` mirror at `src/components/nav.rs:10-24`) | exact |
| `src/components/draft_board.rs` (562 lines) | component | event-driven | (self — preserve `slot_meta`, `on_slot_clear`, `highlighted_slot`) | self-pattern |
| `src/components/tree_graph.rs` (709 lines) | component | event-driven | (self — preserve LayoutNode + DFS layout) | self-pattern (visual only) |
| `src/components/champion_picker.rs` | component | event-driven | (self — preserve `on_select` callback signature, role filter) | self-pattern |
| `src/components/champion_autocomplete.rs` | component | event-driven | (self — preserve `RwSignal<String>` value + `on_select`) | self-pattern |
| `src/components/ui.rs` | component | render-only | (self — restyle `ErrorBanner`, `StatusMessage`, skeletons, EmptyState) | self-pattern |
| `src/components/stat_card.rs` | component | render-only | (self — restyle 19-line component) | self-pattern |
| `src/pages/home.rs` (354 lines) | page | request-response | (self — auth-aware `LandingPage` + `Dashboard` branches) | self-pattern |
| `src/pages/auth/login.rs` | page | request-response | (self — `ActionForm` + redirect Effect) | self-pattern (utility restyle) |
| `src/pages/auth/register.rs` | page | request-response | (self — auto-login + redirect at `register.rs:25-32`) + new invite-token URL handling | self-pattern (extends with `?invite=CODE`) |
| `src/pages/draft.rs` (3,801 lines) | page (heaviest) | event-driven | (self — preserve all server fns) | self-pattern |
| `src/pages/tree_drafter.rs` (1,610) | page | event-driven | (self — children_of HashMap traversal preserved) | self-pattern |
| `src/pages/champion_pool.rs` (1,356) | page | CRUD | (self — preserve drag-rank logic) | self-pattern |
| `src/pages/game_plan.rs` (1,515) | page | CRUD | (self — preserve auto-save) | self-pattern |
| `src/pages/post_game.rs` | page | CRUD | (self — preserve nested view types — recursion_limit=512) | self-pattern |
| `src/pages/profile.rs` | page | CRUD | (self) | self-pattern |
| `src/pages/team/dashboard.rs` | page | CRUD | (self) | self-pattern |
| `src/pages/team/roster.rs` | page | CRUD | (self) | self-pattern (utility restyle) |
| `src/pages/team_builder.rs` | page | CRUD | (self) | self-pattern (utility restyle) |
| `src/pages/stats.rs` | page | request-response | (self) | self-pattern |
| `src/pages/match_detail.rs` | page | request-response | (self — preserve timeline event marker logic) | self-pattern |
| `src/pages/opponents.rs` | page | CRUD | (self) | self-pattern (utility restyle) |
| `src/pages/action_items.rs` | page | CRUD | (self) | self-pattern (utility restyle) |
| `src/pages/personal_learnings.rs` | page | CRUD | (self) | self-pattern (utility restyle) |
| `src/pages/analytics.rs` | page | request-response | (self) | self-pattern (utility restyle) |
| `e2e/tests/pages.spec.ts` | test | request-response | (self — extend `AUTHED_PAGES` array with `/closed-beta`, `/admin/invites`, `/legal/*`) | self-pattern |

---

## Pattern Assignments

### `src/components/theme_toggle.rs` (component, event-driven) — HEAVY RESTYLE

**Analog:** `src/components/nav.rs:10-24` (`set_user_mode` + `ModeToggle`) — exact pattern for DB-persisted toggle.

**Server fn pattern** (copy structure from `src/components/nav.rs:9-24`):
```rust
#[server]
pub async fn set_user_mode(mode: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;
    db::set_user_mode(&db, &user.id, &mode)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}
```

**For Phase 17:** Mirror identically as `set_user_theme(theme: String)` — but ADD validation (theme MUST be `'demacia'` or `'pandemonium'`):
```rust
if theme != "demacia" && theme != "pandemonium" {
    return Err(ServerFnError::new("Invalid theme"));
}
```

**Toggle component pattern** (copy structure from `src/components/nav.rs:26-88` `ModeToggle`):
```rust
#[component]
pub fn ModeToggle(mode: String) -> impl IntoView {
    let current_mode = RwSignal::new(mode);

    let on_click_solo = move |_| {
        let m = current_mode.get_untracked();
        if m == "solo" { return; }
        current_mode.set("solo".to_string());
        leptos::task::spawn_local(async move {
            let _ = set_user_mode("solo".to_string()).await;
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().reload();
            }
        });
    };
    // ... mirror for team
    view! {
        <div class="bg-elevated rounded-lg p-0.5 flex">
            <button
                class=move || {
                    if current_mode.get() == "solo" {
                        "bg-accent text-accent-contrast font-semibold px-3 py-1 text-sm rounded-md cursor-pointer"
                    } else {
                        "text-muted hover:text-secondary px-3 py-1 text-sm rounded-md cursor-pointer"
                    }
                }
                on:click=on_click_solo
            >"Solo"</button>
            // ... team button
        </div>
    }
}
```

**Phase 17 deviation:** Replace `window.location().reload()` with optimistic DOM update (no FOUC) — use the existing `theme_toggle.rs:42-50` document-element pattern:
```rust
#[cfg(feature = "hydrate")]
{
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(root) = doc.document_element() {
            let _ = root.set_attribute("data-theme", new_theme);
        }
    }
}
```

**Existing 5-accent picker to DELETE:** `src/components/theme_toggle.rs:1-10` (`ACCENTS` const), `:60-85` (`set_accent`), `:113-147` (accent-color picker UI).

**WASM safety pattern preserved:** Existing `if let Some(...)` chain at `theme_toggle.rs:42-50` is correct per `wasm-patterns.md` rule 35 — no `.unwrap()` in event handlers.

---

### `src/server/db.rs` additions (service, CRUD)

**Analog:** `src/server/db.rs:4480-4488` (`set_user_mode`) — exact pattern, mirror verbatim.

**Excerpt to copy** (`src/server/db.rs:4480-4488`):
```rust
pub async fn set_user_mode(db: &Surreal<Db>, user_id: &str, mode: &str) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("UPDATE type::record('user', $user_key) SET mode = $mode")
        .bind(("user_key", user_key))
        .bind(("mode", mode.to_string()))
        .await?
        .check()?;
    Ok(())
}
```

**For Phase 17:** Add `set_user_theme(db, user_id, theme)` mirroring identically with field name `theme` and validation either at server-fn boundary OR via SurrealDB `ASSERT $value IN ['demacia','pandemonium']` (defense-in-depth).

**Critical (per `surreal-patterns.md` rule 2):** strip `user:` prefix before binding key. Pattern: `let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();`.

**Critical (per `surreal-patterns.md` rule 4):** `.bind()` requires `'static` — pass owned `String`, never `&str`.

**Critical (per `surreal-patterns.md` rule 27):** `.check()?` after writes to surface errors.

---

### `schema.surql` additions (config, CRUD)

**Analog:** `schema.surql:10` (`mode` field on `user`) — exact pattern.

**Excerpt to copy** (`schema.surql:10`):
```surql
DEFINE FIELD IF NOT EXISTS mode ON user TYPE string DEFAULT 'solo';
```

**For Phase 17:** Add to `user` table:
```surql
DEFINE FIELD IF NOT EXISTS theme ON user TYPE string DEFAULT 'demacia'
  ASSERT $value IN ['demacia', 'pandemonium'];
```

Also add to admin/invite scaffold (visual stub only, full logic in Phase 19.1) — defer to planner per CONTEXT.md `Claude's Discretion`.

**Critical (per `surreal-patterns.md` rule 30):** `IF NOT EXISTS` is mandatory — schema re-applies on every startup.

---

### `src/server/auth.rs` additions (model, CRUD)

**Analog:** `src/server/auth.rs:17-53` (`DbUser`/`AppUser` structs) — exact pattern (additive).

**Excerpt to extend** (`src/server/auth.rs:17-53`):
```rust
#[derive(Clone, Debug, Deserialize, SurrealValue)]
struct DbUser {
    id: RecordId,
    username: String,
    email: String,
    password_hash: String,
    riot_puuid: Option<String>,
    riot_summoner_name: Option<String>,
    mode: Option<String>,
    riot_region: Option<String>,
    // ADD: theme: Option<String>,
}

impl From<DbUser> for AppUser {
    fn from(u: DbUser) -> Self {
        AppUser {
            id: u.id.to_sql(),
            username: u.username,
            // ... existing fields
            mode: u.mode.unwrap_or_else(|| "solo".to_string()),
            // ADD: theme: u.theme.unwrap_or_else(|| "demacia".to_string()),
            riot_region: u.riot_region,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub riot_puuid: Option<String>,
    pub riot_summoner_name: Option<String>,
    pub mode: String,
    // ADD: pub theme: String,
    pub riot_region: Option<String>,
}
```

---

### `src/app.rs` (router/shell) — RESTYLE

**Analog:** self — existing `shell()` at `src/app.rs:31-48` and `<Routes>` at `:58-83`.

**Existing pattern to RESTYLE** (`src/app.rs:31-48`):
```rust
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <script>{r#"(function(){var t=localStorage.getItem('theme');if(t==='light')document.documentElement.setAttribute('data-theme','light');var a=localStorage.getItem('accent');if(a)document.documentElement.setAttribute('data-accent',a);})()"#}</script>
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body class="bg-base min-h-screen text-primary">
                <App />
            </body>
        </html>
    }
}
```

**For Phase 17:**
1. DELETE the inline localStorage `<script>` (replaced by SSR-set `data-theme` per RESEARCH.md Pattern 2).
2. Set `data-theme={user.theme | "demacia"}` on `<html>` reading from `use_context::<UserContext>()` (or equivalent) — see Pitfall 2.
3. Existing `<Route>` list (`src/app.rs:62-82`) preserves all 19 routes per D-09 (URL paths preserved).
4. ADD new route entries for `/closed-beta`, `/admin/invites`, `/legal/impressum`, `/legal/datenschutz` (visual stubs).

---

### `src/components/nav.rs` (748 lines) — HEAVY RESTRUCTURE

**Analog:** self — preserve scaffold of `Nav` component at `src/components/nav.rs:148-`. Existing notifications + Escape-key handler + Suspense-wrapped auth links continue.

**Pattern to preserve** (`src/components/nav.rs:148-198`):
```rust
#[component]
pub fn Nav() -> impl IntoView {
    let logout_action = ServerAction::<Logout>::new();
    let menu_open = RwSignal::new(false);
    let notif_open = RwSignal::new(false);
    let mobile_open = RwSignal::new(false);

    let logout_version = logout_action.version();
    let user = Resource::new(move || logout_version.get(), |_| get_current_user());
    let notifications = Resource::new(|| (), |_| get_notifications());

    let close_all = move || {
        menu_open.set(false);
        notif_open.set(false);
        mobile_open.set(false);
    };

    Effect::new(move || {
        if let Some(Ok(())) = logout_action.value().get() {
            close_all();
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/");
            }
        }
    });
    // ... Escape key Effect, Suspense-wrapped auth links
}
```

**For Phase 17:**
1. KEEP: `Resource::new` for user/notifications, logout Effect, Escape-key listener, Suspense pattern (`leptos-patterns.md` rules 49, 50, 52).
2. REPLACE: 19-route flat link array (lines `:208-` Suspense block) with 4-hub primary nav: `Strategy / Live / History / Profile`.
3. ADD: `current_hub()` helper (pure function on path) using `use_location()` reactive hook (RESEARCH.md Pattern 3 example).
4. ADD: secondary 3-button strip `[Draft] [Tree] [Pool]` and active-hub sub-nav strip below the top nav.
5. PRESERVE: notifications dropdown + ModeToggle invocation + ThemeToggle invocation (now 2-state).

**Sub-nav active-state pattern** (per Pitfall 4): Use `path.starts_with("/match/")` for variable routes; explicit special-cases for nested paths like `/personal-learnings/new`.

---

### `src/pages/closed_beta.rs` (NEW page, request-response, hero tier)

**Analog:** `src/pages/home.rs:147-221` (`HomePage` + `LandingPage`) — auth-aware page with CTA.

**Excerpt to copy/adapt** (`src/pages/home.rs:147-180`):
```rust
#[component]
pub fn HomePage() -> impl IntoView {
    let dashboard = Resource::new(|| (), |_| get_dashboard());

    view! {
        <Suspense fallback=|| view! {
            <div class="max-w-5xl mx-auto py-16 px-6">
                <div class="animate-pulse flex flex-col gap-6">...</div>
            </div>
        }>
            {move || dashboard.get().map(|result| match result {
                Err(e) => view! { ... <ErrorBanner ... /> ... }.into_any(),
                Ok(data) if !data.logged_in => view! { <LandingPage /> }.into_any(),
                Ok(data) => view! { <Dashboard data=data /> }.into_any(),
            })}
        </Suspense>
    }
}
```

**Excerpt for landing layout** (`src/pages/home.rs:182-221` `LandingPage`):
```rust
#[component]
fn LandingPage() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto py-20 px-6 text-center">
            <h1 class="text-5xl font-bold text-primary mb-4 tracking-tight">
                "LoL Team Companion"
            </h1>
            <p class="text-muted text-lg mb-10 max-w-2xl mx-auto">
                "Draft planning, team stats, and strategic tools..."
            </p>
            <div class="flex gap-4 justify-center mb-16">
                <A href="/auth/register">
                    <div class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded-lg px-8 py-3 transition-colors">
                        "Get Started"
                    </div>
                </A>
                ...
            </div>
            ...
        </div>
    }
}
```

**For Phase 17:**
- Wrap content in `<div class="canvas-grain bg-base min-h-screen">` per UI-SPEC §"Closed-Beta Surfaces".
- Add full-viewport background `<img src="/img/beta-landing-demacia.jpg" loading="lazy" aria-hidden="true" class="fixed inset-0 -z-10 w-full h-full object-cover" />` — per UI-SPEC AI Imagery §performance budget.
- Replace "Get Started" CTA with **only** "Sign in" (no register CTA — D-14 requires invite token).
- Use `font-imperial` class for "Closed beta · by invitation" eyebrow + `font-display italic` for "The Strategy Room" headline.

---

### `src/pages/auth/register.rs` (page, request-response) — RESTYLE + invite-token URL

**Analog:** self — `register_action` at `src/pages/auth/register.rs:4-33` and form component `:35-100`.

**Existing pattern to preserve** (`src/pages/auth/register.rs:4-48`):
```rust
#[server]
pub async fn register_action(
    username: String,
    email: String,
    password: String,
) -> Result<String, ServerFnError> {
    use crate::server::auth::{hash_password, AuthSession, Credentials};
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let db = use_context::<Arc<Surreal<Db>>>()...;
    let password_hash = hash_password(&password).map_err(ServerFnError::new)?;
    db::create_user(&db, username, email.clone(), password_hash)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Auto-login after registration
    let mut auth: AuthSession = leptos_axum::extract().await?;
    let creds = Credentials { email, password };
    if let Ok(Some(user)) = auth.authenticate(creds).await {
        let _ = auth.login(&user).await;
    }
    Ok("/solo".to_string())
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register = ServerAction::<RegisterAction>::new();
    Effect::new(move || {
        if let Some(Ok(dest)) = register.value().get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href(&dest);
            }
        }
    });
    ...
}
```

**For Phase 17 (visual only — gate logic Phase 19.1):**
1. KEEP: `register_action` server fn, auto-login + redirect Effect (`leptos-patterns.md` rule 51).
2. ADD: read `?invite=CODE` query param from `use_query_map()` and inject as hidden input `<input type="hidden" name="invite_code" value=invite />`.
3. ADD: redirect to `/closed-beta` if no/invalid invite (visual error state — Phase 19.1 wires real validation).
4. RESTYLE: replace existing red-banner error (`auth/register.rs:62-66`) with `<ErrorBanner>` component (`src/components/ui.rs:5-15`).
5. RESTYLE: form fields per UI-SPEC §"Auth Flows" — `bg-surface/50 border border-outline/50 rounded-lg px-3 py-3` inputs.

**Critical (per `leptos-patterns.md` rule 7):** `ActionForm` has no `class` prop — wrap in `<div>` for styling.

---

### `src/components/draft_board.rs` (562 lines) — VISUAL RESTYLE

**Analog:** self — preserve all logic (`slot_meta` at `:7-31`, `on_slot_clear` callback prop at `:41`, highlight-first deletion at `:60-69`).

**Patterns to preserve** (`src/components/draft_board.rs:34-50`):
```rust
#[component]
pub fn DraftBoard(
    draft_slots: ReadSignal<Vec<Option<String>>>,
    champion_map: HashMap<String, Champion>,
    active_slot: ReadSignal<Option<usize>>,
    on_slot_click: Callback<usize>,
    on_slot_drop: Callback<(usize, String)>,
    highlighted_slot: ReadSignal<Option<usize>>,
    on_slot_clear: Callback<usize>,
    #[prop(optional)] slot_comments: Option<ReadSignal<Vec<Option<String>>>>,
    #[prop(optional)] warning_slots: Option<Signal<Vec<Option<(String, String)>>>>,
    #[prop(optional)] role_assignments: Option<ReadSignal<Vec<Option<String>>>>,
    #[prop(optional)] role_auto_guessed: Option<ReadSignal<Vec<bool>>>,
    #[prop(optional)] on_role_set: Option<Callback<(usize, String)>>,
) -> impl IntoView {
```

**For Phase 17 (per UI-SPEC §"Draft Board Layout"):**
- KEEP signature + drag/click logic + `slot_meta` ban/pick ordering.
- RESTYLE ban slots to 64×64 circular `border-2 border-[var(--gold-3)]` (Demacia) + diagonal red ban-line overlay.
- RESTYLE pick slots to 64×64 square `rounded-md` + on-deck halo `ring-2 ring-accent ring-offset-2 ring-offset-base shadow-[0_0_14px_var(--color-accent)]`.
- KEEP highlight-first deletion: existing `is_highlighted` state at `:60-69` already implements this (Phase 12 pattern, `MEMORY.md` bug-fix #8).
- KEEP `ev.stop_propagation()` on × badge click to prevent outer click re-firing (Phase 12 pattern).

---

### `src/components/tree_graph.rs` (709 lines) — VISUAL RESTYLE

**Analog:** self — preserve LayoutNode + recursive layout algorithm at `:46-80`.

**Patterns to preserve** (`src/components/tree_graph.rs:1-44`):
```rust
const NODE_W: f64 = 180.0;
const NODE_H: f64 = 42.0;
const LEVEL_H: f64 = 120.0;
const H_GAP: f64 = 24.0;
const ICON_SIZE: f64 = 26.0;

#[derive(Clone, Debug)]
struct LayoutNode {
    id: String,
    label: String,
    is_improvised: bool,
    is_root: bool,
    actions: Vec<DraftAction>,
    children: Vec<LayoutNode>,
    x: f64, y: f64, width: f64,
}

fn to_layout_nodes(nodes: &[DraftTreeNode]) -> Vec<LayoutNode> {
    nodes.iter().map(|n| LayoutNode {
        ...
        children: to_layout_nodes(&n.children),
        ...
    }).collect()
}
```

**For Phase 17 (per UI-SPEC §"Tree Graph Interactions"):**
- KEEP: layout algo (`compute_widths`, `assign_positions`), `children_of` HashMap DFS (CLAUDE.md rule 41), debounced auto-save Effect (`MEMORY.md` Phase 12 pattern, `wasm-patterns.md` rule 42).
- ADD: 5 node states (locked, selected, alternate, ghost, leaf) — class-based per UI-SPEC table.
- RESTYLE: edges with animated `stroke-dasharray` keyframe. Use `style="stroke: var(--color-accent)"` for SVG attrs (Tailwind utilities don't reach SVG `stroke` — see Pitfall 9).
- KEEP: side-color edge tint via `stroke-info` (lapis blue) for our team, `stroke-danger` for them.

**Critical (per `leptos-patterns.md` rule 54):** Auto-save Effects MUST capture signal values eagerly outside the timer callback — never lazily inside `Closure::once`. Existing pattern from Phase 12 (`MEMORY.md`) is correct.

---

### `src/components/champion_picker.rs` — VISUAL RESTYLE

**Analog:** self — preserve `ChampionPicker` signature at `:26-31` + role filter logic at `:54-94`.

**Pattern to preserve** (`src/components/champion_picker.rs:26-52`):
```rust
#[component]
pub fn ChampionPicker(
    champions: Vec<Champion>,
    used_champions: Vec<String>,
    on_select: Callback<Champion>,
) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (role_filter, set_role_filter) = signal(Option::<String>::None);

    let filtered = {
        let champions = champions.clone();
        move || {
            let q = query.get().to_lowercase();
            let role = role_filter.get();
            champions.iter()
                .filter(|c| c.name.to_lowercase().contains(&q))
                ...
        }
    };
    ...
}
```

**For Phase 17 (per UI-SPEC §"Champion Picker UX"):**
- KEEP: signature, `used_champions` filter, role-tag mapping at `:4-13`.
- RESTYLE: search bar to `bg-surface/50 border border-outline/50 rounded-lg px-4 py-3` + search icon left.
- RESTYLE: tile grid to `grid grid-cols-[repeat(auto-fill,minmax(56px,1fr))] gap-2 p-4 overflow-y-auto max-h-80`.
- RESTYLE: tiles to `56×56px rounded-md hover:border-accent/40` + selected `ring-2 ring-accent`.

---

### `src/components/champion_autocomplete.rs` — VISUAL RESTYLE

**Analog:** self — preserve `ChampionAutocomplete` signature at `:5-10` and `on_select` callback at `:30-37`.

**Pattern to preserve** (`src/components/champion_autocomplete.rs:5-50`):
```rust
#[component]
pub fn ChampionAutocomplete(
    champions: Vec<Champion>,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] on_select: Option<Callback<String>>,
) -> impl IntoView {
    let (open, set_open) = signal(false);
    let (filter_text, set_filter_text) = signal(String::new());
    let champions = StoredValue::new(champions);

    let filtered = move || { ... };

    let select_champion = move |champ: Champion| {
        value.set(champ.id.clone());
        set_filter_text.set(champ.name.clone());
        set_open.set(false);
        if let Some(cb) = on_select { cb.run(champ.id); }
    };

    Effect::new(move |_| {
        let v = value.get();
        if v != filter_text.get_untracked() { ... }
    });
    ...
}
```

**For Phase 17:** KEEP signature + `on_select` callback firing pattern (Phase 12 MEMORY.md). RESTYLE input + dropdown per UI-SPEC §"Champion Picker UX > Autocomplete dropdown".

---

### `src/components/ui.rs` — VISUAL RESTYLE (preserve API)

**Analog:** self — preserve `ErrorBanner`, `StatusMessage`, `ToastProvider`, `ToastOverlay`, `SkeletonLine`/`Card`/`Grid`, `EmptyState`, `NoTeamState`.

**Pattern to preserve** (`src/components/ui.rs:5-15` `ErrorBanner`):
```rust
#[component]
pub fn ErrorBanner(message: String) -> impl IntoView {
    view! {
        <div class="bg-red-500/10 border border-red-500/30 rounded-xl p-4 flex items-start gap-3">
            <svg ...>...</svg>
            <p class="text-red-400 text-sm">{message}</p>
        </div>
    }
}
```

**For Phase 17:**
- RESTYLE: replace `bg-red-500/10` with `bg-danger/10` (Demacia) — both themes inherit via `--danger` token.
- RESTYLE: rounded corners + heraldic icon ornament (Demacia) per UI-SPEC §"Auth Flows > Error state".
- KEEP: `ToastProvider` context + `ToastOverlay` fixed-position (`ui.rs:31-153`) — 4-second auto-dismiss + 3-toast cap.

---

### `src/pages/home.rs` — RESTYLE (preserve `get_dashboard` server fn)

**Analog:** self — preserve `get_dashboard` server fn at `:24-145`, the auth-branched view at `:147-180`, `LandingPage` and `Dashboard` components.

**Pattern to preserve** (`src/pages/home.rs:147-180`):
```rust
#[component]
pub fn HomePage() -> impl IntoView {
    let dashboard = Resource::new(|| (), |_| get_dashboard());

    view! {
        <Suspense fallback=|| view! { ... skeleton ... }>
            {move || dashboard.get().map(|result| match result {
                Err(e) => view! { ... <ErrorBanner ... /> ... }.into_any(),
                Ok(data) if !data.logged_in => view! { <LandingPage /> }.into_any(),
                Ok(data) => view! { <Dashboard data=data /> }.into_any(),
            })}
        </Suspense>
    }
}
```

**For Phase 17:**
- KEEP: `get_dashboard` server fn, Suspense fallback skeleton, error/landing/dashboard branching.
- BRANCH: when `!data.logged_in`, redirect to `/closed-beta` instead of rendering `LandingPage` inline (D-14 closed-beta gate).
- RESTYLE: `Dashboard` (`:223-323`) into Strategy Room hero per UI-SPEC §"Strategy hub" — gilt cards, HeraldicDivider sections.

---

### `e2e/tests/theme.spec.ts` (NEW test, request-response)

**Analog:** `e2e/tests/pages.spec.ts` — exact pattern (loop over auth pages).

**Excerpt to copy** (`e2e/tests/pages.spec.ts:1-54`):
```typescript
import { test, expect } from "./fixtures";

const AUTHED_PAGES = [
  { path: "/profile", content: /profile|account|riot/i },
  // ...
];

for (const { path, content } of AUTHED_PAGES) {
  test(`${path} loads without JS errors`, async ({ authedPage }) => {
    const errors: string[] = [];
    authedPage.on("pageerror", (e) => errors.push(e.message));
    authedPage.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });
    await authedPage.goto(path);
    await authedPage.waitForLoadState("networkidle");
    expect(authedPage.url()).not.toContain("/auth/login");
    const body = await authedPage.textContent("body");
    expect(body).toMatch(content);
    await expect(authedPage.locator("nav")).toBeVisible();
    const realErrors = errors.filter((e) =>
      !e.includes("favicon") && !e.includes("404 (Not Found)")
    );
    expect(realErrors).toHaveLength(0);
  });
}
```

**For Phase 17:** Use `authedPage` fixture from `e2e/tests/fixtures.ts:75-79`. Test sequence: register → assert default `data-theme="demacia"` on `<html>` → click `Pandemonium` toggle → assert `data-theme="pandemonium"` → reload → assert persistence → logout → re-login → assert persistence.

**Critical (per `wasm-patterns.md` rule 56):** After `waitForURL`, add `await page.waitForTimeout(500)` before subsequent navigations to let WASM Effects fire.

---

### `e2e/tests/closed-beta-visual.spec.ts` (NEW test, request-response)

**Analog:** `e2e/tests/smoke.spec.ts:7-35` — exact pattern (public-page test, no `authedPage`).

**Excerpt to copy** (`e2e/tests/smoke.spec.ts:7-35`):
```typescript
import { test, expect } from "@playwright/test";

const PUBLIC_PAGES = [
  { path: "/", title: /LoL Team Companion/i },
  { path: "/auth/login", title: /LoL Team Companion/i },
  { path: "/auth/register", title: /LoL Team Companion/i },
];

for (const { path, title } of PUBLIC_PAGES) {
  test(`${path} loads without errors`, async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (e) => errors.push(e.message));
    page.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });
    await page.goto(path);
    await expect(page).toHaveTitle(title);
    await expect(page.locator("nav")).toBeVisible();
    expect(errors.filter((e) =>
      !e.includes("favicon") && !e.includes("404 (Not Found)")
    )).toHaveLength(0);
  });
}
```

**For Phase 17:** Test that visiting `/` unauthenticated shows the closed-beta landing copy ("The Strategy Room", "Closed beta · by invitation"), not the existing `LandingPage` content.

---

### `e2e/tests/fonts.spec.ts` (NEW test, network-tab assertion)

**Analog:** `e2e/tests/smoke.spec.ts:14-34` (page-level network/error capture pattern).

**For Phase 17:**
```typescript
test("no Google Fonts CDN requests", async ({ page }) => {
  const fontReqs: string[] = [];
  page.on("request", (req) => {
    const url = req.url();
    if (url.includes("fonts.googleapis.com") || url.includes("fonts.gstatic.com")) {
      fontReqs.push(url);
    }
  });
  await page.goto("/");
  await page.waitForLoadState("networkidle");
  expect(fontReqs).toHaveLength(0);
});

test("self-hosted woff2 served from 127.0.0.1", async ({ page }) => {
  const fontReqs: string[] = [];
  page.on("response", (resp) => {
    if (resp.url().endsWith(".woff2")) {
      fontReqs.push(resp.url());
    }
  });
  await page.goto("/");
  await page.waitForLoadState("networkidle");
  for (const url of fontReqs) {
    expect(url).toContain("127.0.0.1");
  }
});
```

**Critical (per `wasm-patterns.md` rule 47):** Filter out Tailwind v4 `@import "tailwindcss"` 404 — harmless dev-mode artifact.

---

### `e2e/tests/visual-regression.spec.ts` (NEW test, Playwright `toHaveScreenshot`)

**Analog:** `e2e/tests/regression.spec.ts-snapshots/` (existing snapshot dir — confirm convention) + `e2e/tests/pages.spec.ts:25-54` (route-loop pattern).

**For Phase 17:** Loop over both `PUBLIC_PAGES` + `AUTHED_PAGES`, capture `expect(page).toHaveScreenshot()` baselines per route. Initial run captures baseline; future runs detect diff. See `regression.spec.ts-snapshots/` for naming convention.

---

## Shared Patterns

### Authentication / Page Protection

**Source:** `src/pages/profile.rs::get_current_user` + per-page Effect redirect (`leptos-patterns.md` rule 50).
**Apply to:** Every restyled auth-required page in Strategy/History/Profile hubs.

**Excerpt** (typical pattern from existing pages):
```rust
let user = Resource::new(|| (), |_| get_current_user());
Effect::new(move |_| {
    if let Some(Ok(None)) = user.get() {
        #[cfg(feature = "hydrate")]
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/auth/login");
        }
    }
});
```

**Phase 17 deviation:** Closed-beta gate (`/`, `/auth/register` w/o invite) redirects to `/closed-beta` instead of `/auth/login`. Phase 19.1 wires actual logic.

---

### Server fn → DB → ServerFnError mapping

**Source:** `src/components/nav.rs:9-24` `set_user_mode` (canonical pattern for DB-write server fns).
**Apply to:** All new server fns introduced in Phase 17 (`set_user_theme`, future invite mint/redeem).

```rust
#[server]
pub async fn set_user_X(x: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;
    db::set_user_X(&db, &user.id, &x)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}
```

**Critical (per `leptos-patterns.md` rules 9, 11, 12):** SSR imports inside `#[server]` body; DB via `use_context`, not `axum::extract::State`; `mut` on auth for `auth.login()`.

---

### Optimistic DOM update + spawn_local persistence

**Source:** `src/components/theme_toggle.rs:36-57` (theme toggle); `src/components/nav.rs:30-42` (mode toggle).
**Apply to:** Theme toggle, any future per-user UI preference.

```rust
let toggle = move |new_value: &'static str| {
    if signal.get_untracked() == new_value { return; }
    signal.set(new_value.to_string());

    // Optimistic DOM update — no FOUC
    #[cfg(feature = "hydrate")]
    {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Some(root) = doc.document_element() {
                let _ = root.set_attribute("data-theme", new_value);
            }
        }
    }

    leptos::task::spawn_local(async move {
        let _ = set_user_theme(new_value.to_string()).await;
    });
};
```

**Critical (per `wasm-patterns.md` rule 35):** Never `.unwrap()` in event handlers — use `if let Some(...)` chains.

---

### Suspense + ErrorBanner branching

**Source:** `src/pages/home.rs:147-180` (`HomePage` Suspense + match branches).
**Apply to:** All restyled pages with Resource-backed data.

```rust
view! {
    <Suspense fallback=|| view! { ... <SkeletonGrid cols=3 rows=2 /> ... }>
        {move || data.get().map(|result| match result {
            Err(e) => view! {
                <div class="max-w-4xl mx-auto py-16 px-6">
                    <ErrorBanner message=format!("Failed to load: {e}") />
                </div>
            }.into_any(),
            Ok(data) => view! { <Content data=data /> }.into_any(),
        })}
    </Suspense>
}
```

**Critical (per `leptos-patterns.md` rule 19):** Each `match` arm calls `.into_any()` for divergent view types.

---

### Debounced auto-save with cancellable timer (heavy pages)

**Source:** `wasm-patterns.md` rule 42; `MEMORY.md` Phase 12 patterns; existing `tree_drafter.rs` + `draft.rs`.
**Apply to:** Heavy restyle of `draft.rs`, `tree_drafter.rs`, `champion_pool.rs`, `game_plan.rs` (preserve unchanged).

```rust
#[allow(unused_variables)]
let auto_save_timer: RwSignal<Option<i32>> = RwSignal::new(None);

Effect::new(move |_| {
    let val = my_signal.get();      // eagerly tracked
    let id = selected_id.get();      // eagerly captured

    #[cfg(feature = "hydrate")]
    if let Some(id) = auto_save_timer.get_untracked() {
        if let Some(win) = web_sys::window() {
            win.clear_timeout_with_handle(id);
        }
    }

    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        let cb = Closure::once(move || {
            spawn_local(async move { save(id, val).await; });
        });
        if let Some(win) = web_sys::window() {
            if let Ok(t) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(), 2000,
            ) { auto_save_timer.set(Some(t)); }
        }
        cb.forget();
    }
});
```

**Critical (per `leptos-patterns.md` rule 54):** capture signals eagerly outside the Closure — never lazily inside it.

---

### Static asset serving (Axum + cargo-leptos)

**Source:** `Cargo.toml` `[package.metadata.leptos] assets-dir = "public"` (already configured) + `src/main.rs:70-89` (existing static serving via `leptos_axum::file_and_error_handler`).
**Apply to:** New `public/fonts/`, `public/img/` directories.

**Pattern:** Anything in `public/` is automatically copied to `target/site/` by cargo-leptos and served at the matching path by tower-http. No code changes to `main.rs` required — only `mkdir -p public/fonts public/img` and drop files in. See RESEARCH.md Pitfall 7.

---

### CSS theme system (`@theme` + `[data-theme]`)

**Source:** `input.css:7-29` (`@theme` block) + `:32-93` (existing `:root`/`[data-theme="light"]`/`[data-accent="*"]` blocks).
**Apply to:** `input.css` rewrite per RESEARCH.md Pattern 1.

**Existing pattern** (`input.css:7-50`):
```css
@theme {
  --color-base: var(--t-base);
  --color-surface: var(--t-surface);
  --color-elevated: var(--t-elevated);
  --color-primary: var(--t-primary);
  --color-secondary: var(--t-secondary);
  --color-muted: var(--t-muted);
  --color-divider: var(--t-divider);
  --color-outline: var(--t-outline);
  --color-accent: var(--t-accent);
  --color-accent-hover: var(--t-accent-hover);
  --color-accent-contrast: var(--t-accent-contrast);
}

:root {
  --t-base: oklch(0.13 0.004 264.665);
  --t-surface: oklch(0.21 0.006 264.665);
  --t-elevated: oklch(0.278 0.013 260.031);
  ...
}
```

**For Phase 17:**
- KEEP: `@theme` block alias structure (semantic tokens stay).
- ADD: `@font-face` blocks BEFORE `@theme` (~14-20 declarations per UI-SPEC §"Font Self-Hosting").
- REPLACE: existing `:root` block content with demacia tokens; add `[data-theme="pandemonium"]` block.
- DELETE: `[data-accent="blue"]`, `[data-accent="purple"]`, `[data-accent="emerald"]`, `[data-accent="rose"]` blocks (D-04 retired).
- DELETE (recommended, per A4): `[data-theme="light"]` block (no light variant in design).
- ADD: `.canvas-grain` utility class (RESEARCH.md Pattern 1).

**Critical (G-01):** No `@import url("https://fonts.googleapis.com/...")`. Verification: `grep -n "fonts.googleapis\|fonts.gstatic" input.css` must return zero hits.

---

### CSS custom properties for SVG (Tree graph, ornaments)

**Source:** `tree_graph.rs` existing pattern + UI-SPEC §"Implementation Notes for Executor" item 3.
**Apply to:** `tree_graph.rs` restyle, new `ornaments.rs` SVG components.

```rust
// Where Tailwind utilities reach (fill, stroke on currentColor):
view! { <svg class="fill-current text-accent">...</svg> }

// Where they don't (inline SVG attrs):
view! { <path style="stroke: var(--color-accent); stroke-width: 2" ... /> }
```

CSS variables ARE reactive at the rendering layer — no Leptos signal tracking needed for theme switch. See RESEARCH.md Pitfall 9.

---

### E2E auth fixture

**Source:** `e2e/tests/fixtures.ts:15-89` (`authedPage`, `teamPage`, `createTeam`).
**Apply to:** All new e2e tests in Phase 17.

```typescript
import { test, expect } from "./fixtures";

test("authed test", async ({ authedPage }) => { ... });
test("with-team test", async ({ teamPage }) => { ... });
```

**Critical (per `wasm-patterns.md` rule 56):** Add `await page.waitForTimeout(500)` after `waitForURL` to let WASM Effects fire before next navigation.

---

## No Analog Found

Files with no close match in the codebase (planner should use RESEARCH.md patterns + UI-SPEC excerpts):

| File | Role | Data Flow | Why no analog | Reference |
|------|------|-----------|---------------|-----------|
| `public/fonts/**/*.woff2` | static asset | file-I/O | First font self-hosting in repo | RESEARCH.md §"Font Self-Hosting"; google-webfonts-helper for download |
| `public/img/beta-landing-*.jpg` | static asset | file-I/O | First AI-generated imagery | UI-SPEC §"AI Background Imagery"; FLUX.1 via fal.ai |
| `.planning/assets/AI-IMAGES.md` | docs | text | First AI asset registry | RESEARCH.md §"AI background imagery" — record prompt/seed/model/path |
| `src/components/ornaments.rs` | component | render-only | First heraldic SVG primitives (HeraldicDivider, GiltCorner, FleurDeLis, RiotTape, CompanionSigil) | UI-SPEC §"Ornament Library"; SVG paths from `components.jsx:108-145` in design handoff |
| `/home/jasper/Repositories/open-design/design-systems/lol-companion/DESIGN.md` | external docs | n/a | Cross-repo (Open-Design workspace) | Read `/home/jasper/Repositories/open-design/AGENTS.md` first per RESEARCH.md Pattern 4 |
| `src/components/bug_report_widget.rs` | component | event-driven (Phase 18 wires) | No floating-button pattern in repo yet | UI-SPEC §"Bug-Report Widget Placement"; `ToastOverlay` (`ui.rs:103-153`) for fixed-position style guide |

---

## Metadata

**Analog search scope:**
- `src/components/` (8 files, all read)
- `src/pages/` (selective — home.rs, auth/register.rs read in full; others verified via Grep)
- `src/server/db.rs:4470-4488` (set_user_mode pattern)
- `src/server/auth.rs:1-90` (AppUser/DbUser model pattern)
- `src/main.rs` (Axum bootstrap)
- `src/app.rs` (router/shell)
- `schema.surql:1-12` (user table — mode field precedent)
- `input.css:1-93` (@theme + theme/accent blocks)
- `e2e/tests/` (fixtures.ts, smoke.spec.ts, pages.spec.ts, audit-misc-pages.spec.ts)
- `.claude/rules/{leptos,wasm,surreal}-patterns.md` (auto-loaded project rules)

**Files scanned:** ~30 files read in full or via targeted Grep
**Pattern extraction date:** 2026-05-07

**Key insight:** Phase 17 is **largely a self-pattern restyle** — the existing codebase already has high-quality patterns for every concern (debounced auto-save, highlight-first slot deletion, server-fn-with-DB-write, page-protection, Suspense+ErrorBanner branching, optimistic DOM updates, etc.). The risk surface is **visual port fidelity**, not behavioral regression. The single largest pattern transplant is `set_user_mode` → `set_user_theme` (literally line-for-line mirror).

The only fundamentally new patterns introduced are:
1. SVG ornaments (`ornaments.rs`) — new style language, no in-repo analog.
2. `.canvas-grain` CSS utility — port verbatim from `themes.css` in handoff bundle.
3. FLUX-generated background images + AI-IMAGES.md registry — first of its kind.
4. Open-Design DESIGN.md seed (cross-repo) — schema discovered by reading target repo's AGENTS.md.

Everything else maps cleanly to existing patterns.
