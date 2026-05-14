# Phase 18: Region Variants — Pattern Map

**Mapped:** 2026-05-14
**Files analyzed:** 22 (new/modified)
**Analogs found:** 22 / 22

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/components/region/mod.rs` | config | — | `src/components/mod.rs` | exact |
| `src/components/region/ornaments.rs` | component | request-response | `src/components/ornaments.rs` | exact |
| `src/components/region/typography.rs` | component | request-response | `src/components/ornaments.rs` (HeraldicDivider shape) | role-match |
| `src/components/region/layout.rs` | component | request-response | `src/components/ornaments.rs` (CompanionSigil AnyView) | exact |
| `src/components/region/controls.rs` | component | request-response | `src/components/theme_toggle.rs` | role-match |
| `src/components/region/data_viz.rs` | component | request-response | `src/components/stat_card.rs` | role-match |
| `src/components/region/solo.rs` | component | request-response | `src/components/ornaments.rs` (CompanionSigil AnyView) | role-match |
| `src/components/region/chrome.rs` | component | request-response | `src/components/ornaments.rs` (HeraldicDivider shape) | role-match |
| `src/components/mod.rs` | config | — | `src/components/mod.rs` | exact |
| `schema.surql` | migration | batch | `schema.surql` (lines 10-14) | exact |
| `src/server/db.rs` (3 getter/setter pairs) | service | CRUD | `src/server/db.rs:4464-4519` | exact |
| `src/server/auth.rs` (AppUser + DbUser) | model | CRUD | `src/server/auth.rs:17-56` | exact |
| `src/pages/draft.rs` | component | request-response | `src/pages/draft.rs` (existing) | exact |
| `src/pages/tree_drafter.rs` | component | request-response | `src/pages/draft.rs` | role-match |
| `src/pages/champion_pool.rs` | component | request-response | `src/pages/champion_pool.rs` (existing) | exact |
| `src/pages/team/dashboard.rs` | component | request-response | `src/pages/draft.rs` | role-match |
| `src/pages/post_game.rs` | component | request-response | `src/pages/post_game.rs` (existing) | exact |
| `src/pages/match_detail.rs` | component | request-response | `src/pages/post_game.rs` | role-match |
| `src/pages/solo.rs` | component | request-response | `src/components/theme_toggle.rs` (toggle persistence) | role-match |
| `e2e/tests/region-diff.spec.ts` | test | event-driven | `e2e/tests/theme.spec.ts` | role-match |
| `e2e/tests/visual-regression.spec.ts` | test | event-driven | `e2e/tests/visual-regression.spec.ts` (existing) | exact |
| `e2e/tests/fixtures.ts` | utility | event-driven | `e2e/tests/fixtures.ts` (existing) | exact |

---

## Pattern Assignments

### `src/components/region/mod.rs` (config)

**Analog:** `src/components/mod.rs` (lines 1-11)

**Pattern** (`src/components/mod.rs:1-11`):
```rust
pub mod bug_report_widget;
pub mod champion_autocomplete;
pub mod ornaments;
// ...
```

For `region/mod.rs`, replace with `pub use` re-exports per D-01:
```rust
pub mod chrome;
pub mod controls;
pub mod data_viz;
pub mod layout;
pub mod ornaments;
pub mod solo;
pub mod typography;

pub use chrome::*;
pub use controls::*;
pub use data_viz::*;
pub use layout::*;
pub use ornaments::*;
pub use solo::*;
pub use typography::*;
```

---

### `src/components/region/ornaments.rs` (component, request-response)

**Analog:** `src/components/ornaments.rs` (entire file, 191 lines)

**File header pattern** (`src/components/ornaments.rs:1-13`):
```rust
//! Shared SVG ornament primitives per UI-SPEC §"Ornament Library".
//! All stroke/fill colors use CSS custom properties (`var(--color-accent)`)
//! so theme switching automatically retints them. Per CLAUDE.md
//! "no raw hex in components" rule.

use leptos::prelude::*;

use crate::app::InitialTheme;
```

**Simple (non-branching) SVG primitive pattern** (`src/components/ornaments.rs:17-41`):
```rust
#[component]
pub fn HeraldicDivider(
    #[prop(optional, default = 200)] width: u32,
) -> impl IntoView {
    view! {
        <svg
            width=width
            height="16"
            viewBox="0 0 200 16"
            class="block"
            style="stroke: var(--color-accent)"
        >
            // ...SVG paths...
        </svg>
    }
}
```

**AnyView region-branching pattern — THE CORE PRIMITIVE PATTERN** (`src/components/ornaments.rs:128-167`):
```rust
#[component]
pub fn CompanionSigil() -> impl IntoView {
    let is_pandemonium = use_context::<InitialTheme>()
        .map(|t| t.0 == "pandemonium")
        .unwrap_or(false);

    if is_pandemonium {
        view! {
            <span
                class="font-glitch text-[18px] uppercase tracking-[0.18em]"
                style="color: var(--color-accent)"
            >
                "COMPANION_"
            </span>
        }
        .into_any()
    } else {
        view! {
            <div class="flex items-center gap-2.5">
                // ...demacia markup...
            </div>
        }
        .into_any()
    }
}
```

**CRITICAL note on `CompanionSigil` vs new region primitives:** `CompanionSigil` reads `InitialTheme` context internally (legacy pattern). Per SPEC Constraints and CONTEXT.md Established Patterns, new region primitives in `region/` MUST accept `region: String` as a prop instead, NOT call `use_context`. The only exception is `CompanionSigil` which is migrated verbatim into `region/ornaments.rs`.

**New primitive prop pattern** (from RESEARCH.md Pattern 2):
```rust
#[component]
pub fn GiltCorner(
    #[prop(optional, default = "tl")] corner: &'static str,
    #[prop(optional, default = 28)] size: u32,
    // NO region prop — ornament is Demacia-only by design
) -> impl IntoView { ... }

// For region-branching primitives:
#[component]
pub fn Card(
    region: String,
    #[prop(optional, into)] variant: Option<String>,
    children: Children,       // use ChildrenFn if both arms need children
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";
    if is_pandemonium {
        view! { /* pandemonium markup */ }.into_any()
    } else {
        view! { /* demacia markup */ }.into_any()
    }
}
```

---

### `src/components/region/typography.rs` (component, request-response)

**Analog:** `src/components/ornaments.rs` (simple component shape, lines 17-41)

**Imports pattern** (same as ornaments.rs):
```rust
use leptos::prelude::*;
```

**Simple typography component pattern** (no region branching needed for most — typography is font-family only, controlled by CSS tokens):
```rust
#[component]
pub fn Eyebrow(
    children: Children,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    view! {
        <div class=format!(
            "font-mono text-[10px] uppercase tracking-[0.16em] text-muted {}",
            class.unwrap_or_default()
        )>
            {children()}
        </div>
    }
}
```

**`Glitch` requires region-branching** (Pandemonium-only):
```rust
#[component]
pub fn Glitch(
    region: String,
    children: ChildrenFn,  // ChildrenFn because both arms call children()
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";
    if is_pandemonium {
        view! {
            <span class="font-glitch tracking-[0.18em]" style="...glitch text-shadow...">
                {children()}
            </span>
        }.into_any()
    } else {
        view! { <span>{children()}</span> }.into_any()
    }
}
```

---

### `src/components/region/layout.rs` (component, request-response)

**Analog:** `src/components/ornaments.rs:128-167` (CompanionSigil AnyView branch) + `.local-design-source/.../components.jsx:276-315` (Card reference)

**Card with both region variants** (design source `components.jsx:277-315`):
- `region="demacia"` + `variant="gilt"` → gradient background + inset border + `GiltCorner` decorations in each corner
- `region="pandemonium"` + `variant="zine"` → `border-radius: 0` + accent bracket corners
- default → plain surface card

**AnyView branch for layout** (copy from ornaments.rs:128 pattern, adapted):
```rust
#[component]
pub fn Card(
    region: String,
    #[prop(optional, into)] variant: Option<String>,
    children: ChildrenFn,  // ChildrenFn: both region arms call children()
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";
    let var = variant.unwrap_or_default();
    move || if is_pandemonium {
        view! {
            <div class="bg-surface border border-outline/30 rounded-none p-6 relative">
                // Pandemonium bracket corners via absolute positioned divs
                {children()}
            </div>
        }.into_any()
    } else {
        view! {
            <div class="bg-surface border border-outline/50 rounded-xl p-6 relative">
                // Demacia gilt corners via <GiltCorner> when var == "gilt"
                {children()}
            </div>
        }.into_any()
    }
}
```

---

### `src/components/region/controls.rs` (component, request-response)

**Analog:** `src/components/theme_toggle.rs` (lines 40-117)

**Reactive button with signal-derived class** (`src/components/theme_toggle.rs:83-98`):
```rust
<button
    type="button"
    class=move || {
        let active = current_theme.get() == "demacia";
        if active {
            "px-3 py-1 rounded-full font-imperial text-[10px] uppercase tracking-[0.18em] bg-accent text-accent-contrast font-semibold cursor-pointer transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
        } else {
            "px-3 py-1 rounded-full font-imperial text-[10px] uppercase tracking-[0.18em] text-muted hover:text-secondary cursor-pointer transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
        }
    }
    aria-pressed=move || (current_theme.get() == "demacia").to_string()
    on:click=move |_| set_theme.run(String::from("demacia"))
>
    "Demacia"
</button>
```

**`Btn` region-branching approach** (design source `components.jsx:210-254`):
- Demacia primary: gold gradient background, Cinzel `font-imperial`, uppercase, `text-accent-contrast`
- Pandemonium primary: flat accent bg, `font-mono`, box-shadow stagger, `border-radius: 0`
- Both use semantic tokens; no raw hex

```rust
#[component]
pub fn Btn(
    region: String,
    #[prop(optional, into)] variant: Option<String>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    children: ChildrenFn,
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";
    let var = variant.unwrap_or_else(|| "primary".to_string());
    move || if is_pandemonium && var == "primary" {
        view! {
            <button
                type="button"
                class="inline-flex items-center gap-2 px-4 py-2.5 font-mono text-[13px] uppercase tracking-[0.12em] bg-accent text-accent-contrast rounded-none cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                style="box-shadow: 3px 3px 0 var(--color-accent-2), 6px 6px 0 var(--accent-3)"
                on:click=move |_| { if let Some(cb) = on_click { cb.run(()); } }
            >
                {children()}
            </button>
        }.into_any()
    } else {
        view! {
            <button
                type="button"
                class="inline-flex items-center gap-2 px-4 py-2.5 font-imperial text-[13px] uppercase tracking-[0.14em] rounded cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                style="background: linear-gradient(180deg, var(--gold-1, var(--color-accent)) 0%, var(--gold-2, var(--color-accent)) 50%, var(--gold-3, var(--color-accent)) 100%); color: var(--ink, var(--t-accent-contrast)); border: 1px solid var(--gold-deep, var(--border-outline))"
                on:click=move |_| { if let Some(cb) = on_click { cb.run(()); } }
            >
                {children()}
            </button>
        }.into_any()
    }
}
```

---

### `src/components/region/data_viz.rs` (component, request-response)

**Analog:** `src/components/stat_card.rs` (entire file, 27 lines)

**Existing StatCard pattern** (`src/components/stat_card.rs:1-27`):
```rust
use leptos::prelude::*;

#[component]
pub fn StatCard(
    label: String,
    value: String,
    #[prop(optional)] sub: Option<String>,
) -> impl IntoView {
    view! {
        <div class="bg-surface border border-outline/50 rounded-xl p-4">
            <div class="font-imperial uppercase tracking-wider text-xs text-muted mb-1">{label}</div>
            <div class="font-display text-primary text-2xl tabular-nums">{value}</div>
            {sub.map(|s| view! {
                <div class="text-muted text-sm mt-1">{s}</div>
            })}
        </div>
    }
}
```

**New `Stat` primitive** (design source `components.jsx:317-330`) — no Card wrapper, just the number cluster:
```rust
#[component]
pub fn Stat(
    label: String,
    value: String,
    #[prop(optional)] unit: Option<String>,
    #[prop(optional)] delta: Option<f32>,
) -> impl IntoView {
    // No region branching — Stat uses font tokens that are region-agnostic
    // Eyebrow label + Mono value + optional delta color
    view! {
        <div class="flex flex-col gap-1">
            <div class="font-mono text-[10px] uppercase tracking-[0.16em] text-muted">{label}</div>
            <div class="flex items-baseline gap-2">
                <span class="font-mono text-[22px] font-semibold tabular-nums text-primary">{value}</span>
                // unit and delta spans...
            </div>
        </div>
    }
}
```

---

### `src/components/region/solo.rs` (component, request-response)

**Analog:** `src/components/ornaments.rs:128-167` (CompanionSigil AnyView branch — same structural shape)

Same AnyView branch pattern. `RankBadge` and `LPProgress` are region-branching: Demacia renders heraldic crest/shield framing; Pandemonium renders glitch-text rank with stagger border.

---

### `src/components/region/chrome.rs` (component, request-response)

**Analog:** `src/components/ornaments.rs:17-41` (HeraldicDivider — simple SVG component shape)

`ChampPortrait`, `ChampTile`, `RoleIcon`, `Icon` are mostly non-branching image/SVG wrappers. They use Data Dragon CDN URLs and mask-image for role icons. See design source (`components.jsx:34-143`) for prop shapes.

**ChampPortrait pattern** (design source `components.jsx:34-52`, adapted to Leptos):
```rust
#[component]
pub fn ChampPortrait(
    name: String,
    #[prop(optional, default = 64)] size: u32,
    #[prop(optional, default = "square")] kind: &'static str,
) -> impl IntoView {
    let key = champ_key(&name);
    let src = match kind {
        "loading" => format!("https://ddragon.leagueoflegends.com/cdn/img/champion/loading/{key}_0.jpg"),
        "splash"  => format!("https://ddragon.leagueoflegends.com/cdn/img/champion/centered/{key}_0.jpg"),
        _         => format!("https://ddragon.leagueoflegends.com/cdn/14.24.1/img/champion/{key}.png"),
    };
    view! {
        <img
            src=src
            alt=name
            loading="lazy"
            style=format!("display: block; width: {size}px; height: {size}px; object-fit: cover; object-position: center top;")
        />
    }
}
```

---

### `src/components/mod.rs` (config)

**Analog:** `src/components/mod.rs` (existing, lines 1-11)

**Modification:** Remove `pub mod ornaments;` line; add `pub mod region;`.

```rust
// Before (line 7):
pub mod ornaments;

// After:
pub mod region;
```

All ~7 import sites that currently reference `crate::components::ornaments::*` must be updated to `crate::components::region::ornaments::*` or to the re-exported path `crate::components::region::*`.

**Import sites to update** — grep confirms these files import from ornaments:
- `src/components/nav.rs`
- `src/components/theme_toggle.rs` (if it imports CompanionSigil)
- `src/pages/draft.rs` (line 3: `use crate::components::ornaments::{CompanionSigil, HeraldicDivider}`)
- Any other pages referencing `FleurDeLis`, `Crown`, `GiltCorner`, `RiotTape`

---

### `schema.surql` (migration, batch)

**Analog:** `schema.surql:10-14` (existing `mode` and `theme` field definitions)

**Existing precedent** (`schema.surql:10-14`):
```sql
DEFINE FIELD IF NOT EXISTS mode ON user TYPE string DEFAULT 'solo';
DEFINE FIELD IF NOT EXISTS theme ON user TYPE string DEFAULT 'demacia'
  ASSERT $value IN ['demacia', 'pandemonium'];
```

**New additions** (D-03 — NO ASSERT constraint, wider value set):
```sql
DEFINE FIELD IF NOT EXISTS draft_mode ON user TYPE string DEFAULT 'auto';
DEFINE FIELD IF NOT EXISTS team_dashboard_mode ON user TYPE string DEFAULT 'auto';
DEFINE FIELD IF NOT EXISTS solo_mode ON user TYPE string DEFAULT 'auto';
```

Append after line 14. The `IF NOT EXISTS` makes this idempotent on re-runs. No ASSERT — value validation happens at the application layer in the server functions.

---

### `src/server/db.rs` — 3 getter/setter pairs (service, CRUD)

**Analog:** `src/server/db.rs:4464-4519` (`get_user_mode`/`set_user_mode` and `get_user_theme`/`set_user_theme`)

**Getter pattern** (`src/server/db.rs:4464-4478`):
```rust
pub async fn get_user_mode(db: &Surreal<Db>, user_id: &str) -> DbResult<String> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    #[derive(Debug, Deserialize, SurrealValue)]
    struct ModeRecord {
        mode: Option<String>,  // Option<String> — handles NONE for pre-Phase records
    }
    let mut result = db
        .query("SELECT mode FROM type::record('user', $user_key)")
        .bind(("user_key", user_key))
        .await?;
    let row: Option<ModeRecord> = result.take(0)?;
    Ok(row
        .and_then(|r| r.mode)
        .unwrap_or_else(|| "solo".to_string()))  // fallback for legacy NONE
}
```

**Setter pattern** (`src/server/db.rs:4480-4488`):
```rust
pub async fn set_user_mode(db: &Surreal<Db>, user_id: &str, mode: &str) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("UPDATE type::record('user', $user_key) SET mode = $mode")
        .bind(("user_key", user_key))
        .bind(("mode", mode.to_string()))
        .await?
        .check()?;   // surreal-patterns rule 27: .check() required on writes
    Ok(())
}
```

**Three pairs to add** — copy mode getter/setter verbatim, change field names and fallback:
- `get_user_draft_mode` / `set_user_draft_mode` — fallback `"auto"`
- `get_user_team_dashboard_mode` / `set_user_team_dashboard_mode` — fallback `"auto"`
- `get_user_solo_mode` / `set_user_solo_mode` — fallback `"auto"`

---

### `src/server/auth.rs` — AppUser + DbUser extension (model, CRUD)

**Analog:** `src/server/auth.rs:17-56` (existing `DbUser` → `AppUser` pattern)

**DbUser struct** (`src/server/auth.rs:16-27`):
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
    theme: Option<String>,
}
```

**AppUser struct** (`src/server/auth.rs:45-56`):
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppUser {
    pub id: String,
    // ... existing fields ...
    pub mode: String,
    pub theme: String,
}
```

**From<DbUser> conversion** (`src/server/auth.rs:29-43`):
```rust
impl From<DbUser> for AppUser {
    fn from(u: DbUser) -> Self {
        AppUser {
            // ...
            mode: u.mode.unwrap_or_else(|| "solo".to_string()),
            theme: u.theme.unwrap_or_else(|| "demacia".to_string()),
        }
    }
}
```

**Add 3 new fields** — same pattern for all three:
```rust
// In DbUser struct — add after `theme: Option<String>`:
draft_mode: Option<String>,
team_dashboard_mode: Option<String>,
solo_mode: Option<String>,

// In AppUser struct — add after `theme: String`:
pub draft_mode: String,
pub team_dashboard_mode: String,
pub solo_mode: String,

// In From<DbUser> for AppUser — add after `theme:` conversion:
draft_mode: u.draft_mode.unwrap_or_else(|| "auto".to_string()),
team_dashboard_mode: u.team_dashboard_mode.unwrap_or_else(|| "auto".to_string()),
solo_mode: u.solo_mode.unwrap_or_else(|| "auto".to_string()),
```

---

### Page files: `src/pages/{draft,tree_drafter,champion_pool,team/dashboard,post_game,match_detail,solo}.rs` (component, request-response)

**Analog:** `src/pages/draft.rs` (heavy existing page) + `src/pages/post_game.rs` + `src/pages/champion_pool.rs`

**Region context read at page entry — THE CORRECT PATTERN** (from `src/app.rs:57-59` + RESEARCH.md Pattern 2):
```rust
// At the top of each page component function body:
let theme = use_context::<InitialTheme>().unwrap_or_default();
let region = theme.0.clone();
// Pass region as prop to all child view functions; do NOT call use_context inside child components
```

**Server fn auth pattern** (repeated throughout all pages, e.g. `src/pages/champion_pool.rs:18-26`):
```rust
#[server]
pub async fn my_server_fn() -> Result<SomeType, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;
    // ...
}
```

**"No team → empty list" pattern** (leptos-patterns rule 44, used in `src/pages/post_game.rs:25-31`):
```rust
let team_id = match db::get_user_team_id(&surreal, &user.id)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
{
    Some(id) => id,
    None => return Ok(Vec::new()),
};
```

**Mode-branching component structure** (RESEARCH.md Pattern 1 + CONTEXT.md D-04):
```rust
#[component]
pub fn DraftPage() -> impl IntoView {
    let theme = use_context::<InitialTheme>().unwrap_or_default();
    let region = theme.0.clone();
    // resolve_mode is a pure fn defined in the same file or shared module
    let mode = resolve_mode(&stored_mode, &region, "draft");

    view! {
        // Top-level branch keeps each view! macro tree shallow
        {move || match mode {
            "carousel" => view! { <DraftCarouselView region=region.clone() /> }.into_any(),
            "war-table" => view! { <DraftWarTableView region=region.clone() /> }.into_any(),
            "ledger"   => view! { <DraftLedgerView region=region.clone() /> }.into_any(),
            _ => view! { <DraftCarouselView region=region.clone() /> }.into_any(),
        }}
    }
}
```

**Mode toggle server fn pattern** (copy from `src/components/theme_toggle.rs:8-29`, adapted):
```rust
#[server]
pub async fn set_draft_mode_pref(mode: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    // Validate mode value at server fn layer (no DB ASSERT on mode fields)
    let valid = ["auto", "carousel", "war-table", "ledger"];
    if !valid.contains(&mode.as_str()) {
        return Err(ServerFnError::new("Invalid draft mode"));
    }

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;
    db::set_user_draft_mode(&db, &user.id, &mode)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}
```

---

### `e2e/tests/fixtures.ts` — `setRegion`/`setMode` helpers (utility, event-driven)

**Analog:** `e2e/tests/fixtures.ts` (existing file, lines 1-101) + `e2e/tests/theme.spec.ts:20-35` (toggle click pattern)

**Existing toggle click pattern** (`e2e/tests/theme.spec.ts:23-27`):
```typescript
await authedPage.click('button:has-text("Pandemonium")');
await authedPage.waitForTimeout(700); // WASM Effect settle (wasm-patterns rule 56)
const theme = await authedPage.getAttribute("html", "data-theme");
```

**New `setRegion` helper** (RESEARCH.md Pattern fixtures.ts code, verified against existing theme.spec.ts):
```typescript
export async function setRegion(
  page: Page,
  region: 'demacia' | 'pandemonium'
): Promise<void> {
  const btnText = region === 'pandemonium' ? 'Pandemonium' : 'Demacia';
  const themeAttr = await page.getAttribute('html', 'data-theme');
  if (themeAttr === region) return; // already correct
  await page.click(`button:has-text("${btnText}")`);
  await page.waitForTimeout(700); // wasm-patterns rule 56
  const newTheme = await page.getAttribute('html', 'data-theme');
  if (newTheme !== region) throw new Error(`setRegion failed: expected ${region}, got ${newTheme}`);
}

export async function setMode(
  page: Page,
  mode: string
): Promise<void> {
  await page.click(`[data-mode-toggle="${mode}"], button:has-text("${mode.toUpperCase()}"), button:has-text("${mode}")`);
  await page.waitForTimeout(500);
}
```

**Add import** to any spec that uses these helpers:
```typescript
import { test, expect, setRegion, setMode } from './fixtures';
```

---

### `e2e/tests/visual-regression.spec.ts` — subfolder path refactor (test, event-driven)

**Analog:** `e2e/tests/visual-regression.spec.ts` (existing, lines 1-205)

**Existing flat snapshot path pattern** (`e2e/tests/visual-regression.spec.ts:111-116`):
```typescript
test("/draft visual baseline", async ({ authedPage }) => {
  await authedPage.goto("/draft");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(500);
  await expect(authedPage).toHaveScreenshot("authed-draft.png", VR_OPTS);
});
```

**New subfolder path pattern** (RESEARCH.md Pattern 4 + D-09 naming convention):
```typescript
test("/draft demacia carousel baseline", async ({ authedPage }) => {
  await setRegion(authedPage, 'demacia');
  await authedPage.goto("/draft");
  await setMode(authedPage, 'carousel');
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(500);
  await expect(authedPage).toHaveScreenshot(
    ['authed-draft', 'demacia-carousel.png'],  // array syntax is cross-platform safe
    VR_OPTS
  );
});
```

**VR_OPTS stays unchanged** (lines 32-35):
```typescript
const VR_OPTS = {
  maxDiffPixelRatio: 0.02,
  fullPage: true,
} as const;
```

**Utility routes (15 existing baselines) stay flat** — only the 7 scoped routes get subfolder restructuring per D-07.

---

### `e2e/tests/region-diff.spec.ts` (test, event-driven)

**Analog:** `e2e/tests/theme.spec.ts` (structure) + RESEARCH.md Pattern 5 (pixelmatch implementation)

**File structure pattern** (from `e2e/tests/theme.spec.ts:1-35`):
```typescript
import { test, expect } from "./fixtures";

test("default theme is demacia for a new user", async ({ authedPage }) => {
  await authedPage.goto("/team/dashboard");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(500);
  // assertions...
});
```

**pixelDiffRatio helper** (RESEARCH.md Pattern 5 — viewport screenshots, not fullPage):
```typescript
import { test, expect } from './fixtures';
import { setRegion } from './fixtures';
import { PNG } from 'pngjs';
import pixelmatch from 'pixelmatch';

// Viewport-only (not fullPage) to ensure identical dimensions between regions
async function pixelDiffRatio(buf1: Buffer, buf2: Buffer): Promise<number> {
  const img1 = PNG.sync.read(buf1);
  const img2 = PNG.sync.read(buf2);
  if (img1.width !== img2.width || img1.height !== img2.height) {
    return 1.0; // dimension mismatch = completely different
  }
  const diff = pixelmatch(img1.data, img2.data, null, img1.width, img1.height, { threshold: 0.1 });
  return diff / (img1.width * img1.height);
}

test('draft carousel: demacia vs pandemonium differ by >40%', async ({ authedPage }) => {
  await setRegion(authedPage, 'demacia');
  await authedPage.goto('/draft');
  await authedPage.waitForLoadState('networkidle');
  await authedPage.waitForTimeout(500);
  const demBuf = await authedPage.screenshot(); // fullPage: false (default) = viewport-only

  await setRegion(authedPage, 'pandemonium');
  await authedPage.goto('/draft');
  await authedPage.waitForLoadState('networkidle');
  await authedPage.waitForTimeout(500);
  const panBuf = await authedPage.screenshot();

  const ratio = await pixelDiffRatio(demBuf, panBuf);
  expect(ratio).toBeGreaterThan(0.40);
});
```

**npm install required before running** (e2e/package.json addition):
```bash
cd e2e && npm install pixelmatch pngjs
cd e2e && npm install --save-dev @types/pixelmatch @types/pngjs
```

---

## Shared Patterns

### 1. AnyView Region Branch (applies to all 24 region primitives)

**Source:** `src/components/ornaments.rs:128-167`
**Apply to:** All files under `src/components/region/`

```rust
// Pattern: top-level if/else branch, NOT nested inside view! {}
// Both arms call .into_any() — leptos-patterns rule 19
let is_pandemonium = region == "pandemonium";
if is_pandemonium {
    view! { /* pandemonium markup */ }.into_any()
} else {
    view! { /* demacia markup */ }.into_any()
}
```

Key constraints (from RESEARCH.md anti-patterns):
- Never nest `view! {}` inside `view! {}` for region branching — both branches' types accumulate in the outer macro's type tree
- Use `ChildrenFn` (not `Children`) when BOTH branch arms render children
- Add `.into_any()` to BOTH arms without exception — omitting causes `E0308 if and else have incompatible types`

### 2. Region Prop at Page Entry (applies to all 7+ page files)

**Source:** `src/app.rs:57-59` + CONTEXT.md Established Patterns
**Apply to:** `src/pages/draft.rs`, `src/pages/tree_drafter.rs`, `src/pages/champion_pool.rs`, `src/pages/team/dashboard.rs`, `src/pages/post_game.rs`, `src/pages/match_detail.rs`, `src/pages/solo.rs`

```rust
// In the page component — read context ONCE at entry
let theme = use_context::<InitialTheme>().unwrap_or_default();
let region = theme.0.clone();
// Pass region as prop to all subcomponents — never call use_context inside region primitives
```

### 3. DB Getter/Setter Pair (applies to 3 new mode fields)

**Source:** `src/server/db.rs:4464-4488`
**Apply to:** `src/server/db.rs` additions for `draft_mode`, `team_dashboard_mode`, `solo_mode`

```rust
// Getter: Option<String> field + unwrap_or_else fallback
struct ModeRecord { field_name: Option<String> }
// ...take(0)? + and_then + unwrap_or_else(|| "auto".to_string())

// Setter: .check()? required after write
db.query("UPDATE type::record('user', $user_key) SET field_name = $value")
    .bind(("user_key", user_key))
    .bind(("value", value.to_string()))
    .await?.check()?;
```

### 4. Server Function Auth Guard (applies to all new server functions)

**Source:** `src/pages/champion_pool.rs:18-29` (canonical example)
**Apply to:** All `#[server]` functions in modified page files

```rust
let auth: AuthSession = leptos_axum::extract().await?;
let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
let db = use_context::<Arc<Surreal<Db>>>()
    .ok_or_else(|| ServerFnError::new("No DB context"))?;
```

### 5. Semantic Tokens Only (applies to all component files)

**Source:** CLAUDE.md §"Code Style" + `input.css:193-265`
**Apply to:** All `src/components/region/**` and modified `src/pages/**` files

Use semantic tokens: `bg-base`, `bg-surface`, `bg-elevated`, `text-primary`, `text-secondary`, `text-muted`, `text-dimmed`, `border-divider`, `border-outline`, `bg-accent`, `text-accent-contrast`. Never hardcode hex values. Exception: `text-white` on colored buttons (e.g. `bg-red-700 text-white`).

Pandemonium special tokens (from `input.css:193-265`): `var(--accent-3)`, `var(--accent-2)`, `var(--gold-1)`, `var(--gold-2)`, `var(--gold-3)`, `var(--gold-deep)`, `var(--ink)` — these are CSS custom properties, NOT Tailwind classes; use via `style` attributes.

### 6. Dual-Target Compile Check (applies to every change)

**Source:** CLAUDE.md §"Critical Patterns"
**Apply to:** Every file change in this phase

After each task, verify both:
```bash
cargo check --features ssr
cargo check --features hydrate --target wasm32-unknown-unknown
```

Unused-variable warnings under `#[cfg(feature = "hydrate")]` guards require `#[allow(unused_variables)]` on the `let` declaration (wasm-patterns rule 43).

---

## No Analog Found

All files have analogs. No entries in this section.

---

## Metadata

**Analog search scope:** `src/components/`, `src/pages/`, `src/server/`, `schema.surql`, `e2e/tests/`, `.local-design-source/lol-team-companion-app/project/pages/_shared/components.jsx`
**Files scanned:** 14 source files + 2 design source files
**Pattern extraction date:** 2026-05-14
