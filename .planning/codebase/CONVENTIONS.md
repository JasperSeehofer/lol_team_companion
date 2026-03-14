# Code Conventions

## Naming

| Element | Convention | Example |
|---------|-----------|---------|
| Files | `snake_case.rs` | `champion_pool.rs` |
| Functions | `snake_case` | `get_current_user()` |
| Components | `PascalCase` with `#[component]` | `ChampionPoolPage` |
| Types/Structs | `PascalCase` | `DraftAction` |
| DB structs | `Db` prefix + `PascalCase` | `DbTeam`, `DbDraft` |
| Constants | `SCREAMING_SNAKE_CASE` | `DEFAULT_TIMEOUT` |
| CSS classes | Tailwind utilities inline | `class="bg-surface text-primary"` |

## Import Organization

```rust
// 1. Standard library
use std::collections::HashMap;
use std::sync::Arc;

// 2. External crates
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// 3. Internal modules
use crate::components::champion_picker::ChampionPicker;
use crate::models::draft::{Draft, DraftAction};
```

**SSR-only imports** go inside `#[server]` function bodies, not at module top:
```rust
#[server]
pub async fn save_draft(...) -> Result<String, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};
    // ...
}
```

## Error Handling

### Custom Error Types (`src/server/db.rs`)
```rust
#[derive(Debug, Error)]
pub enum DbError {
    #[error("SurrealDB error: {0}")]
    Surreal(#[from] surrealdb::types::Error),
    #[error("Record not found")]
    NotFound,
    #[error("{0}")]
    Other(String),
}
pub type DbResult<T> = Result<T, DbError>;
```

### Server Function Errors
Map to `ServerFnError` at the boundary:
```rust
db::some_query(args).await.map_err(|e| ServerFnError::new(e.to_string()))?;
```

### Empty Results for Optional Resources
Return empty `Vec` (not `Err`) when a scoped resource is absent:
```rust
let team_id = match db::get_user_team_id(...).await? {
    Some(id) => id,
    None => return Ok(Vec::new()),  // not Err
};
```

## SurrealDB Patterns

### DbStruct → AppStruct Conversion
```rust
#[derive(Debug, Deserialize, SurrealValue)]
struct DbTeam {
    id: RecordId,
    name: String,
    created_by: RecordId,
}

impl From<DbTeam> for Team {
    fn from(t: DbTeam) -> Self {
        Team {
            id: Some(t.id.to_sql()),         // RecordId → String via .to_sql()
            name: t.name,
            created_by: t.created_by.to_sql(),
        }
    }
}
```

### Query Patterns
```rust
// Write queries: use .check() to surface errors
db.query("CREATE ...").bind(("key", value)).await?.check()?;

// List queries: unwrap_or_default for empty results
let rows: Vec<DbTeam> = result.take(0).unwrap_or_default();

// Single lookups: use ? for Option<T>
let item: Option<DbTeam> = result.take(0)?;

// Batch queries: chain statements, index by position
let mut r = db.query("SELECT ...; SELECT ...;").await?;
let teams: Vec<DbTeam> = r.take(0).unwrap_or_default();
let members: Vec<DbMember> = r.take(1).unwrap_or_default();
```

### Bind and Record IDs
```rust
// Always strip table prefix before type::record()
let key = user_id.strip_prefix("user:").unwrap_or(&user_id).to_string();
// Always pass owned String to .bind()
db.query("... type::record('user', $key) ...")
    .bind(("key", key))
    .await?.check()?;
```

## Leptos Reactivity Patterns

### Server Functions in Components
```rust
#[server]
pub async fn get_data() -> Result<Vec<Item>, ServerFnError> { ... }

#[component]
pub fn MyPage() -> impl IntoView {
    let data = Resource::new(|| (), |_| get_data());
    view! {
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            {move || data.get().map(|result| match result {
                Ok(items) => view! { /* render items */ }.into_any(),
                Err(e) => view! { <ErrorBanner message=e.to_string()/> }.into_any(),
            })}
        </Suspense>
    }
}
```

### Signal Conventions
- `get_untracked()` in event handlers (avoid accidental tracking)
- `prop:value` for controlled inputs (not `attr:value`)
- `StoredValue::new()` for non-reactive data shared across closures
- `resource.refetch()` after mutations
- `spawn_local` for async in event handlers
- `collect_view()` for iterator rendering
- `<For>` with stable entity ID keys (not array index)

### Clone Before Closures
```rust
let val_for_class = role_val.clone();
view! {
    <button class=move || { ... val_for_class.clone() ... }
            on:click=move |_| set_x.set(role_val.clone())>
}
```

### Divergent View Branches
```rust
{move || if condition {
    view! { <div>"A"</div> }.into_any()
} else {
    view! { <span>"B"</span> }.into_any()
}}
```

## Component Patterns

### Protected Page Template
```rust
#[component]
pub fn MyPage() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());

    // Client-side auth redirect
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let Some(Ok(None)) = user.get() {
            if let Some(win) = web_sys::window() {
                let _ = win.location().set_href("/auth/login");
            }
        }
    });

    view! { /* page content */ }
}
```

### Debounced Auto-Save
```rust
#[allow(unused_variables)]
let timer: RwSignal<Option<i32>> = RwSignal::new(None);

Effect::new(move |_| {
    let val = signal.get();  // eagerly capture
    #[cfg(feature = "hydrate")]
    {
        // cancel pending timer, schedule new one
        // use Closure::once with captured values
    }
});
```

## Styling

- **Semantic tokens:** `bg-base`, `bg-surface`, `bg-elevated`, `text-primary`, `text-secondary`, `text-muted`, `border-divider`, `border-outline`, `bg-accent`
- **Dark theme default**, light via `[data-theme="light"]`
- **Accent palettes:** yellow (default), blue, purple, emerald, rose
- **Exception:** Colored buttons keep literal `text-white`
- **Forms:** `bg-surface/50` inputs with `border-outline/50`
- **`<A>` component:** Use `attr:class="..."` not `class="..."`
- **`ActionForm`:** No `class` prop — wrap in `<div>` for styling
